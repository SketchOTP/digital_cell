//! D-006 surface-turnover candidate pipeline.

use chemistry_core::*;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn d006_root() -> PathBuf {
    PathBuf::from("experiments/generated/d006")
}

fn git_commit_hash() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

pub fn run_planar_calibration() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d006_root().join("planar_interface");
    fs::create_dir_all(&root)?;
    let mut base = surface_turnover_params_from_calibrated_kphi1();
    base.k_structure_interface = 1.0; // probe integral uses act×I; rate scaled after
    let phi_in = 1.0;
    let n = 1.0;
    let f = 1.0;
    let c = 0.35;
    let b = integrate_planar_b_interface(&base, phi_in, n, f, c, 40.0, 0.125);
    let k0 = derive_k_structure_interface(base.k_structure_decay, phi_in, 24.0, b);
    let mut profile = Vec::new();
    let mut x = -20.0;
    while x <= 20.0 {
        profile.push(json!({
            "n": x,
            "phi": planar_phase_profile(x, base.seed_interface_width, phi_in),
            "I": interface_weight(planar_phase_profile(x, base.seed_interface_width, phi_in)),
        }));
        x += 0.5;
    }
    let out = json!({
        "equation_version": EQUATION_VERSION_SURFACE,
        "phi_in": phi_in,
        "N": n,
        "F": f,
        "C": c,
        "k_structure_decay": base.k_structure_decay,
        "R_reference": 24.0,
        "B_interface": b,
        "k_structure_interface_initial": k0,
        "assumptions": [
            "translationally invariant planar tanh interface",
            "uniform N,F,C across interface for B measurement",
            "R_reference used only to set physical scale, not as runtime target",
        ],
        "profile": profile,
        "commit_hash": git_commit_hash(),
    });
    fs::write(root.join("calibration.json"), serde_json::to_string_pretty(&out)?)?;
    Ok(out)
}

pub fn write_candidates(k0: f64) -> Result<Vec<CandidateIdentity>, Box<dyn std::error::Error>> {
    let root = d006_root().join("candidates");
    fs::create_dir_all(&root)?;
    let factors = [0.60, 0.80, 1.00, 1.20, 1.40];
    let commit = git_commit_hash();
    let mut ids = Vec::new();
    for fac in factors {
        let mut p = surface_turnover_params_from_calibrated_kphi1();
        p.k_structure_interface = k0 * fac;
        let id = build_candidate_identity(
            p,
            &commit,
            Some("surface_turnover_v1"),
            None,
            &format!("planar-derived × {fac}"),
            None,
            None,
        );
        let dir = root.join(&id.candidate_id);
        fs::create_dir_all(&dir)?;
        fs::write(
            dir.join("identity.json"),
            serde_json::to_string_pretty(&id)?,
        )?;
        fs::write(
            dir.join("params.json"),
            serde_json::to_string_pretty(&id.params)?,
        )?;
        ids.push(id);
    }
    fs::write(
        root.join("index.json"),
        serde_json::to_string_pretty(&ids.iter().map(|i| {
            json!({
                "candidate_id": i.candidate_id,
                "candidate_hash": i.candidate_hash,
                "configuration_hash": i.configuration_hash,
                "k_structure_interface": i.params.k_structure_interface,
                "equation_version": i.equation_version,
            })
        }).collect::<Vec<_>>())?,
    )?;
    Ok(ids)
}

pub fn run_prescribed_radius(id: &CandidateIdentity) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d006_root().join("prescribed_radius").join(&id.candidate_id);
    fs::create_dir_all(&root)?;
    let radii = [12.0, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0];
    let points: Vec<_> = radii
        .iter()
        .map(|r| prescribed_circular_rates(&id.params, *r, 160, 160, 0.35, 1.0, 1.0))
        .collect();
    let crossing = has_stable_radius_crossing(&points);
    let out = json!({
        "candidate_id": id.candidate_id,
        "candidate_hash": id.candidate_hash,
        "equation_version": id.equation_version,
        "has_stable_crossing": crossing,
        "points": points.iter().map(|p| json!({
            "radius": p.radius,
            "integrated_assembly": p.integrated_assembly,
            "integrated_decay": p.integrated_decay,
            "d_m_phi_dt": p.d_m_phi_dt,
            "d_r_dt": p.d_r_dt,
        })).collect::<Vec<_>>(),
    });
    fs::write(root.join("result.json"), serde_json::to_string_pretty(&out)?)?;
    Ok(out)
}

pub fn run_one_public(
    id: &CandidateIdentity,
    r0: f64,
    c0: f64,
    seed: u64,
    substeps: u64,
    out_dir: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    run_one_coupled(id, r0, c0, seed, substeps, out_dir)
}

fn run_one_coupled(
    id: &CandidateIdentity,
    r0: f64,
    c0: f64,
    seed: u64,
    substeps: u64,
    out_dir: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;
    let result_path = out_dir.join("result.json");
    if result_path.exists() {
        return Ok(serde_json::from_str(&fs::read_to_string(result_path)?)?);
    }
    let recipe = FreshSeedRecipe {
        r0,
        c0,
        noise_seed: seed,
        noise_amplitude: 0.005,
    };
    let cfg = crate::d005::SimRunConfig {
        substeps,
        record_every: 500,
        checkpoint_every: 5_000,
        trajectory_sample_every: 500,
    };
    let t0 = Instant::now();
    let mut sim = spawn_fresh_simulation(id.params.clone(), &recipe);
    sim.observer_enabled = true;
    let start_m_phi = total_mass(&sim.grid, &sim.fields.structure);
    let start_m_c = total_mass(&sim.grid, &sim.fields.catalyst);
    let start_r = (start_m_phi.max(1e-9) / std::f64::consts::PI).sqrt();
    let (balance, _traj, _wins) = crate::d005::advance_simulation(&mut sim, &cfg);
    let diag = sim.current_diagnostics();
    let end_r = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
    let wall = t0.elapsed().as_secs_f64();
    let out = json!({
        "equation_version": id.equation_version,
        "candidate_id": id.candidate_id,
        "candidate_hash": id.candidate_hash,
        "configuration_hash": id.configuration_hash,
        "seed_recipe": recipe.identity_key(),
        "noise_seed": seed,
        "r0": r0,
        "c0": c0,
        "initial_radius": start_r,
        "final_radius": end_r,
        "initial_m_phi": start_m_phi,
        "final_m_phi": diag.structural_mass,
        "initial_m_c": start_m_c,
        "final_m_c": diag.catalyst_mass,
        "radial_velocity": (end_r - start_r) / sim.sim_time.max(1e-12),
        "catalyst_velocity": (diag.catalyst_mass - start_m_c) / sim.sim_time.max(1e-12),
        "q_phi": balance.q_phi,
        "q_c": balance.q_c,
        "slope_phi": balance.slope_phi,
        "slope_c": balance.slope_catalyst,
        "retention": diag.catalyst_retention,
        "accepted_substeps": sim.substep,
        "simulated_time": sim.sim_time,
        "classification": format!("{:?}", sim.detector.last_classification),
        "wall_seconds": wall,
        "substeps_per_second": substeps as f64 / wall.max(1e-9),
    });
    fs::write(result_path, serde_json::to_string_pretty(&out)?)?;
    Ok(out)
}

pub fn run_coupled_screen(substeps: u64) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let index: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(d006_root().join("candidates/index.json"))?)?;
    let radii = [16.0, 20.0, 24.0, 28.0, 32.0];
    let cats = [0.275, 0.35, 0.425];
    let seeds = [1u64, 2, 3];
    let mut summary = Vec::new();
    for entry in index.as_array().unwrap() {
        let cid = entry["candidate_id"].as_str().unwrap();
        let id_path = d006_root().join("candidates").join(cid).join("identity.json");
        let id: CandidateIdentity = serde_json::from_str(&fs::read_to_string(id_path)?)?;
        let prescribed = run_prescribed_radius(&id)?;
        if prescribed["has_stable_crossing"] != true {
            summary.push(json!({
                "candidate_id": cid,
                "status": "rejected_prescribed_no_crossing",
            }));
            continue;
        }
        let mut vel_by_r: Vec<(f64, Vec<f64>)> = radii.iter().map(|r| (*r, Vec::new())).collect();
        for r0 in radii {
            for c0 in cats {
                for seed in seeds {
                    let out = d006_root().join("candidate_screen").join(cid).join(format!(
                        "R{}_C{}_s{}",
                        r0 as u32,
                        (c0 * 1000.0) as u32,
                        seed
                    ));
                    eprintln!("D-006 screen {cid} R={r0} C={c0} seed={seed}");
                    let rec = run_one_coupled(&id, r0, c0, seed, substeps, &out)?;
                    let vr = rec["radial_velocity"].as_f64().unwrap_or(0.0);
                    if let Some((_, v)) = vel_by_r.iter_mut().find(|(r, _)| (*r - r0).abs() < 1e-9) {
                        v.push(vr);
                    }
                }
            }
        }
        // median v_R by radius
        let mut medians = Vec::new();
        for (r, mut vs) in vel_by_r {
            vs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = if vs.is_empty() {
                0.0
            } else {
                vs[vs.len() / 2]
            };
            medians.push(json!({"r0": r, "median_v_r": med}));
        }
        let below = medians
            .iter()
            .filter(|m| m["r0"].as_f64().unwrap() < 24.0)
            .all(|m| m["median_v_r"].as_f64().unwrap() > 0.0);
        let above = medians
            .iter()
            .filter(|m| m["r0"].as_f64().unwrap() > 24.0)
            .all(|m| m["median_v_r"].as_f64().unwrap() < 0.0);
        summary.push(json!({
            "candidate_id": cid,
            "status": if below && above { "restoring_pass" } else { "no_restoring_region" },
            "median_v_r_by_radius": medians,
            "restoring_below_24": below,
            "restoring_above_24": above,
        }));
    }
    let out = json!({ "screen_summary": summary, "substeps": substeps });
    fs::create_dir_all(d006_root().join("restoring_radius"))?;
    fs::write(
        d006_root().join("restoring_radius/screen_summary.json"),
        serde_json::to_string_pretty(&out)?,
    )?;
    Ok(out)
}

pub fn bootstrap_and_screen(substeps: u64) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(d006_root())?;
    let planar = run_planar_calibration()?;
    let k0 = planar["k_structure_interface_initial"].as_f64().unwrap();
    let ids = write_candidates(k0)?;
    for id in &ids {
        let _ = run_prescribed_radius(id)?;
    }
    let screen = run_coupled_screen(substeps)?;
    let manifest = json!({
        "directive": "D-006",
        "equation_version": EQUATION_VERSION_SURFACE,
        "derived_k_structure_interface": k0,
        "candidate_count": ids.len(),
        "screen": screen,
        "commit_hash": git_commit_hash(),
    });
    fs::write(
        d006_root().join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(manifest)
}
