//! Headless Phase 1 research runtime binary.

use chemistry_core::material_mesh::MaterialMesh;
use phase1_certifier::frozen::FROZEN_CENTER;
use phase1_certifier::sim::{
    contract_label_for_mesh, reserve_enabled, run_coupled, seed_production_mesh,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn usage() {
    eprintln!(
        "digital-protocell-phase1 — headless Phase 1 research runtime\n\
         Usage:\n\
           digital-protocell-phase1 --steps N [--radius R] [--seed S] [--out PATH]\n\
           digital-protocell-phase1 --resume SNAPSHOT [--steps N] [--out PATH]\n\
         Offline / no GPU. Fails closed on incompatible snapshots."
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
        process::exit(0);
    }
    let mut steps = 1000usize;
    let mut radius = 14.0f64;
    let mut seed = 1u64;
    let mut out = PathBuf::from("phase1_run.json");
    let mut resume: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--steps" => {
                i += 1;
                steps = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(steps);
            }
            "--radius" => {
                i += 1;
                radius = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(radius);
            }
            "--seed" => {
                i += 1;
                seed = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(seed);
            }
            "--out" => {
                i += 1;
                if let Some(p) = args.get(i) {
                    out = PathBuf::from(p);
                }
            }
            "--resume" => {
                i += 1;
                if let Some(p) = args.get(i) {
                    resume = Some(PathBuf::from(p));
                }
            }
            other => {
                eprintln!("unknown arg: {other}");
                usage();
                process::exit(2);
            }
        }
        i += 1;
    }

    let mut mesh = if let Some(p) = resume {
        let text = match fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("fail-closed: cannot read snapshot: {e}");
                process::exit(3);
            }
        };
        match serde_json::from_str::<MaterialMesh>(&text) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("fail-closed: incompatible snapshot: {e}");
                process::exit(4);
            }
        }
    } else {
        seed_production_mesh(radius, seed)
    };

    let _ = FROZEN_CENTER; // identity seal
    let ledger = run_coupled(&mut mesh, steps, true, true);
    let report = serde_json::json!({
        "package": "digital-protocell-phase1-v1",
        "mesh_contract": contract_label_for_mesh(&mesh),
        "reserve_enabled": reserve_enabled(),
        "steps": steps,
        "alive": mesh.alive,
        "area": mesh.area(),
        "c": mesh.interior.c,
        "a": mesh.interior.a,
        "m_total": mesh.total_structural_mass(),
        "b_total": mesh.total_bound_membrane(),
        "ledger": ledger,
        "offline": true,
        "gpu": false,
    });
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(
        &out,
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    ) {
        eprintln!("write failed: {e}");
        process::exit(5);
    }
    // Save resume snapshot beside out
    let snap = out.with_extension("snapshot.json");
    if let Ok(s) = serde_json::to_string(&mesh) {
        let _ = fs::write(snap, s);
    }
    println!("ok alive={} area={:.4}", mesh.alive, mesh.area());
}
