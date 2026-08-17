//! DC-DEV-018 Phase 0 source-feasibility audit.
//!
//! This executable is observer-only.  It reconstructs the live N/F -> A
//! source, irreversible A/R exits, and the exact DC-DEV-017 matched precursor
//! trace before any homeostat production code is introduced.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::{fs, path::{Path, PathBuf}};

const ENTRY: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const SETTLE: usize = 5_000;
const STORAGE_STEPS: usize = 4_000;
const D016_CLAMP: f64 = 0.1476710565778127;
const DT: f64 = 0.02;
const EPS: f64 = 1.0e-12;

#[derive(Debug, Clone, Copy, Serialize)]
struct Snapshot {
    step: usize,
    time: f64,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    e_stored: f64,
    alive: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct DemandStep {
    source: f64,
    irreversible_demand: f64,
    a_decay: f64,
    structural_a: f64,
    catalyst_a: f64,
    membrane_a: f64,
    reserve_to_w: f64,
    reserve_to_structural: f64,
    a_to_r: f64,
    r_to_a: f64,
    substrate_ceiling: f64,
    required_gain: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct Window {
    steps: usize,
    source_sum: f64,
    demand_sum: f64,
    source_max: f64,
    demand_max: f64,
    required_gain_max: f64,
    substrate_ceiling_min: f64,
    mean_a: f64,
    mean_r: f64,
    mean_e_stored: f64,
}

#[derive(Debug, Clone, Serialize)]
struct AuditResult {
    directive: String,
    entry_commit: String,
    settled_body_hash: String,
    deprived_body_hash: String,
    replete: Snapshot,
    deprived: Snapshot,
    target_precursor: f64,
    storage_horizon_steps: usize,
    storage_horizon_time: f64,
    source_equation: String,
    irreversible_demand_equation: String,
    e_stored_equation: String,
    windows: Vec<Window>,
    g_required_max: f64,
    hard_substrate_source_ceiling: f64,
    finite_capacity: bool,
    source_only_feasibility: String,
    observer_only: bool,
    production_behavior_changed: bool,
    prior_art_disposition: String,
    trajectory_hash: String,
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn snapshot(mesh: &MaterialMesh, step: usize) -> Snapshot {
    Snapshot {
        step,
        time: step as f64 * DT,
        area: mesh.area(),
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        e_stored: mesh.area() * (mesh.interior.a + mesh.interior.r),
        alive: mesh.alive,
    }
}

fn seed() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem { c: 0.8, a: 0.5, r: 0.6, ..Default::default() },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn params(mesh: &MaterialMesh) -> ReactionParams {
    let mut p = ReactionParams::default();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    p
}

fn settle() -> MaterialMesh {
    let mut mesh = seed();
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= EPS);
    for _ in 0..SETTLE {
        assert!(mechanics_step(&mut mesh, &mechanics));
    }
    mesh
}

fn deprive(settled: &MaterialMesh) -> MaterialMesh {
    let mut mesh = settled.clone();
    let p = params(&mesh);
    for _ in 0..480 {
        reactions_step(&mut mesh, &p, DT, true, true);
    }
    mesh
}

fn inferred_decay(before: LumpedChem, after: LumpedChem, r: &ReactionLedger, area: f64) -> f64 {
    let value = area * (before.a - after.a) + r.a_produced + r.reserve.r_to_a
        - r.c_produced - r.a_consumed_build - r.l_produced - r.reserve.a_to_r;
    assert!(value >= -1.0e-9, "negative inferred A decay {value}");
    value.max(0.0)
}

fn run_audit(deprived: &MaterialMesh, p: &ReactionParams) -> (Vec<Window>, Vec<String>, f64, f64) {
    let mut mesh = deprived.clone();
    let mut windows = vec![Window::default(); 4];
    let mut hashes = Vec::with_capacity(STORAGE_STEPS + 1);
    hashes.push(stable_json_hash(&snapshot(&mesh, 0)).unwrap());
    let mut g_cap: f64 = 0.0;
    let mut hard_ceiling = f64::INFINITY;
    for step in 0..STORAGE_STEPS {
        let area = mesh.area().max(1.0e-15);
        let n_delta = (D016_CLAMP - mesh.interior.n).max(0.0);
        let f_delta = (D016_CLAMP - mesh.interior.f).max(0.0);
        mesh.interior.n += n_delta;
        mesh.interior.f += f_delta;
        let before = mesh.interior;
        let reaction = reactions_step(&mut mesh, p, DT, true, true);
        let a_decay = inferred_decay(before, mesh.interior, &reaction, area);
        let demand = a_decay
            + reaction.a_consumed_build
            + reaction.c_produced
            + reaction.l_produced
            + reaction.reserve.r_to_w
            + reaction.reserve.r_to_m;
        let source = reaction.a_produced;
        let substrate_ceiling = D016_CLAMP * area;
        let required_gain = if demand <= EPS {
            0.0
        } else if source > EPS {
            demand / source
        } else {
            f64::INFINITY
        };
        let record = DemandStep {
            source,
            irreversible_demand: demand,
            a_decay,
            structural_a: reaction.a_consumed_build,
            catalyst_a: reaction.c_produced,
            membrane_a: reaction.l_produced,
            reserve_to_w: reaction.reserve.r_to_w,
            reserve_to_structural: reaction.reserve.r_to_m,
            a_to_r: reaction.reserve.a_to_r,
            r_to_a: reaction.reserve.r_to_a,
            substrate_ceiling,
            required_gain,
        };
        assert!(record.source.is_finite() && record.irreversible_demand.is_finite());
        let window = &mut windows[step / 1000];
        window.steps += 1;
        window.source_sum += record.source;
        window.demand_sum += record.irreversible_demand;
        window.source_max = window.source_max.max(record.source);
        window.demand_max = window.demand_max.max(record.irreversible_demand);
        window.required_gain_max = window.required_gain_max.max(record.required_gain);
        window.substrate_ceiling_min = if window.steps == 1 {
            record.substrate_ceiling
        } else {
            window.substrate_ceiling_min.min(record.substrate_ceiling)
        };
        window.mean_a += mesh.interior.a;
        window.mean_r += mesh.interior.r;
        window.mean_e_stored += mesh.area() * (mesh.interior.a + mesh.interior.r);
        g_cap = g_cap.max(record.required_gain);
        hard_ceiling = hard_ceiling.min(record.substrate_ceiling);
        hashes.push(stable_json_hash(&snapshot(&mesh, step + 1)).unwrap());
    }
    for window in &mut windows {
        if window.steps > 0 {
            let n = window.steps as f64;
            window.mean_a /= n;
            window.mean_r /= n;
            window.mean_e_stored /= n;
        }
    }
    (windows, hashes, g_cap, hard_ceiling)
}

fn main() {
    let out = std::env::var_os("DCDEV018_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev018"));
    let settled = settle();
    let deprived = deprive(&settled);
    let p = params(&deprived);
    let (windows, hashes, g_cap, hard_ceiling) = run_audit(&deprived, &p);
    let replete = snapshot(&settled, 0);
    let deprived_snapshot = snapshot(&deprived, 480);
    let finite_capacity = g_cap.is_finite()
        && g_cap >= 1.0
        && windows.iter().all(|w| w.demand_max <= w.substrate_ceiling_min + EPS);
    let feasibility = if finite_capacity {
        "DCDEV018_SOURCE_ONLY_HOMEOSTASIS_FEASIBLE"
    } else {
        "DCDEV018_SOURCE_ONLY_HOMEOSTASIS_INFEASIBLE"
    };
    let result = AuditResult {
        directive: "DC-DEV-018".into(),
        entry_commit: ENTRY.into(),
        settled_body_hash: stable_json_hash(&settled).unwrap(),
        deprived_body_hash: stable_json_hash(&deprived).unwrap(),
        replete,
        deprived: deprived_snapshot,
        target_precursor: D016_CLAMP,
        storage_horizon_steps: STORAGE_STEPS,
        storage_horizon_time: STORAGE_STEPS as f64 * DT,
        source_equation: "J_source = k_act*q_catalyst(C)*g_harvest*area*N*F*dt, with existing finite substrate clamps".into(),
        irreversible_demand_equation: "J_demand = inferred A decay + A->structure + A->catalyst + A->membrane + R->W + R->structure; A<->R excluded".into(),
        e_stored_equation: "E_stored = area*(A+R)".into(),
        windows,
        g_required_max: g_cap,
        hard_substrate_source_ceiling: hard_ceiling,
        finite_capacity,
        source_only_feasibility: feasibility.into(),
        observer_only: true,
        production_behavior_changed: false,
        prior_art_disposition: "REFERENCE primary literature; COMPOSE integral-feedback principle; BUILD native Digital Cell wrapper; REJECT external code, species, parameters, and ATP/ADP model imports".into(),
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    };
    write(&out, "protocol.json", &json!({
        "directive":"DC-DEV-018","entry_commit":ENTRY,"source_branch":"strategy/dc-dev-016-metabolic-break-even",
        "phase":"0","observer_only":true,"settlement_steps":SETTLE,"deprivation_steps":480,
        "source_feasibility_steps":STORAGE_STEPS,"dt":DT,"storage_horizon":80.0,
        "target_precursor":D016_CLAMP,"e_stored":"area*(A+R)","parameter_sweep":false,
        "production_behavior_changed":false,"dcdev017_production_imported":false
    }));
    write(&out, "source_feasibility.json", &serde_json::to_value(&result).unwrap());
    write(&out, "artifact_manifest.json", &json!({
        "directive":"DC-DEV-018","phase":"0","entry_commit":ENTRY,
        "files":["protocol.json","source_feasibility.json","artifact_manifest.json"],
        "conclusion":feasibility,"next_execution_started":false
    }));
    println!("{feasibility}\nG_cap_required={g_cap:.12e}\nhard_substrate_ceiling={hard_ceiling:.12e}");
}
