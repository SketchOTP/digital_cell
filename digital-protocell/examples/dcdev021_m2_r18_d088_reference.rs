//! R18 observer-only trace of the already-qualified D-088 physical fission path.
//! This reproduces the historical fixture and records readiness on the clone
//! immediately before the existing fission call. It does not alter the kernel.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_topology::{find_local_pinch, local_rebond_range, TopologyLedger};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

const STEPS: usize = 12_000;

fn rotate(mesh: &mut MaterialMesh, angle: f64) {
    let c = mesh.centroid();
    let (s, co) = angle.sin_cos();
    for p in &mut mesh.vertices {
        let x = p[0] - c[0];
        let y = p[1] - c[1];
        p[0] = c[0] + co * x - s * y;
        p[1] = c[1] + s * x + co * y;
    }
}

fn perturb(mesh: &mut MaterialMesh) {
    for (i, p) in mesh.vertices.iter_mut().enumerate() {
        let f = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        p[0] += 0.35 * (f - 0.5);
        p[1] += 0.35 * ((f * 7.13).fract() - 0.5);
    }
}

fn historical_fixture(seed: u64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    rotate(&mut mesh, 0.3);
    perturb(&mut mesh);
    let c = mesh.centroid();
    for p in &mut mesh.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
    }
    mesh
}

fn best_nonadjacent_distance(mesh: &MaterialMesh) -> Option<f64> {
    let n = mesh.n();
    if n < 8 {
        return None;
    }
    let min_sep = (n / 4).max(3);
    let mut best = None;
    for i in 0..n {
        for d in min_sep..=(n - min_sep) {
            let j = (i + d) % n;
            if j <= i || (j - i).min(n - (j - i)) < min_sep {
                continue;
            }
            let a = mesh.vertices[i];
            let b = mesh.vertices[j];
            let distance = (b[0] - a[0]).hypot(b[1] - a[1]);
            best = Some(best.map_or(distance, |current: f64| current.min(distance)));
        }
    }
    best
}

fn readiness(mesh: &MaterialMesh, fission: &FissionParams, ledger: &TopologyLedger) -> Value {
    let range = local_rebond_range(mesh, &fission.topo);
    let pinch = find_local_pinch(mesh, &fission.topo);
    let distance = pinch.map(|(i, j)| {
        let a = mesh.vertices[i];
        let b = mesh.vertices[j];
        (b[0] - a[0]).hypot(b[1] - a[1])
    });
    let absolute_a = mesh.interior.a.max(0.0) * mesh.area().abs();
    let required = distance.map(|d| mesh.rho_s * d);
    let sufficient = required.map(|need| absolute_a >= need).unwrap_or(false);
    let shadow = try_local_fission(&mesh.clone(), fission).is_some();
    json!({
        "topology": {"vertices": mesh.n(), "area": mesh.area(), "perimeter": mesh.perimeter()},
        "structural_mass": mesh.total_structural_mass(),
        "shape_factor": 4.0 * std::f64::consts::PI * mesh.area().abs() / mesh.perimeter().powi(2),
        "max_edge_strain": (0..mesh.n()).map(|i| mesh.strain(i)).fold(f64::NEG_INFINITY, f64::max),
        "mean_edge_strain": (0..mesh.n()).map(|i| mesh.strain(i)).sum::<f64>() / mesh.n().max(1) as f64,
        "local_rebond_range": range,
        "best_nonadjacent_distance": best_nonadjacent_distance(mesh),
        "pinch_candidate": pinch.map(|(i,j)| json!({"i":i,"j":j,"distance":distance})),
        "absolute_a_mass": absolute_a,
        "cross_bond_mass_needed": required,
        "a_over_cross_bond_need": required.map(|need| absolute_a / need.max(1e-12)),
        "cross_bond_a_sufficient": sufficient,
        "shadow_try_local_fission": if shadow {"SUCCESS"} else {"FAIL"},
        "topology_ledger": {"tension_ruptures":ledger.tension_ruptures,"local_rebonds":ledger.local_rebonds,"cross_bonds":ledger.cross_bonds}
    })
}

fn physical_step(mesh: &mut MaterialMesh, mech: &MechParams, react: &ReactionParams,
                 transport: &TransportParams, growth: &GrowthParams,
                 fission: &FissionParams, topology: bool) -> TopologyLedger {
    let _ = transport_step(mesh, transport, mech.dt);
    let _ = chemistry_core::mesh_reactions::reactions_step(mesh, react, mech.dt, true, true);
    let _ = growth_step(mesh, react, growth, mech.dt);
    assert!(mechanics_step(mesh, mech));
    remesh(mesh);
    if topology { topology_step(mesh, fission) } else { TopologyLedger::default() }
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcdev021_m2_r18_d088_reference.json");
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--output" {
            output = PathBuf::from(args.get(i + 1).expect("--output path"));
            i += 2;
        } else { i += 1; }
    }
    let mut mesh = historical_fixture(1);
    let birth_mass = mesh.total_structural_mass();
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let mut rows = Vec::new();
    let mut event = None;
    for step in 0..STEPS {
        if !mesh.can_advance_physics() { break; }
        let ledger = physical_step(&mut mesh, &mech, &react, &transport, &growth, &fission, step % 10 == 0);
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            let before = mesh.clone();
            let row = readiness(&before, &fission, &ledger);
            rows.push(json!({"step":step + 1,"phase":"d088_fission_evaluation","mass_gate":true,"readiness":row}));
            if let Some((a,b,e)) = try_local_fission(&mesh, &fission) {
                event = Some(json!({"step":step + 1,"daughter_a_vertices":a.n(),"daughter_b_vertices":b.n(),"event":format!("{:?}", e)}));
                break;
            }
        }
    }
    let value = json!({
        "directive": "DC-DEV-021-M2-R18-PHYSICAL-FISSION-READINESS-AND-D088-EXECUTION-PARITY-AUDIT-001",
        "fixture": "accepted D-088 historical_fixture(1)",
        "execution_order": ["transport_step","reactions_step","growth_step","mechanics_step","remesh","topology_step_every_10","try_local_fission_every_25_after_mass_gate"],
        "birth_mass": birth_mass,
        "final_step": rows.last().and_then(|r| r["step"].as_u64()),
        "physical_fission": event.is_some(),
        "event": event,
        "readiness_trace": rows,
        "raw_final_mesh": "externalized; compact reference only"
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
