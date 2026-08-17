//! DC-DEV-018 Phase 0 source-feasibility audit.
//!
//! This executable is observer-only.  It reconstructs the live N/F -> A
//! source, irreversible A/R exits, and the exact DC-DEV-017 matched precursor
//! trace before any homeostat production code is introduced.

use chemistry_core::integral_homeostat::{MetabolicHomeostatParamsV1, MetabolicHomeostatStateV1};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, reactions_step_with_homeostat, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    name: String,
    steps: usize,
    initial: Snapshot,
    final_state: Snapshot,
    quarter_snapshots: Vec<Snapshot>,
    quarter_e_stored_slopes: Vec<f64>,
    quarter_e_stored_changes: Vec<f64>,
    quarter_source: Vec<f64>,
    quarter_demand: Vec<f64>,
    quarter_error_mean: Vec<f64>,
    quarter_capacity_mean: Vec<f64>,
    initial_capacity: f64,
    final_capacity: f64,
    max_capacity: f64,
    max_e_stored: f64,
    n_delivered: f64,
    f_delivered: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    max_resource_error: f64,
    resource_conservation: bool,
    finite_nonnegative: bool,
    alive: bool,
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
        LumpedChem {
            c: 0.8,
            a: 0.5,
            r: 0.6,
            ..Default::default()
        },
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
        - r.c_produced
        - r.a_consumed_build
        - r.l_produced
        - r.reserve.a_to_r;
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

#[derive(Clone, Copy)]
enum ArmMode {
    ResourceFree,
    CurrentResource,
    DerivedResource,
    SustainedPrecursor,
}

impl ArmMode {
    fn name(self) -> &'static str {
        match self {
            Self::ResourceFree => "M1_starvation_homeostat_on",
            Self::CurrentResource => "M2_current_resource_homeostat_on",
            Self::DerivedResource => "M3_derived_sufficient_resource_homeostat_on",
            Self::SustainedPrecursor => "M4_sustained_precursor_homeostat_on",
        }
    }

    fn inventory(self) -> Option<f64> {
        match self {
            Self::ResourceFree | Self::SustainedPrecursor => None,
            Self::CurrentResource => Some(3.0),
            Self::DerivedResource => Some(14.588954880632265),
        }
    }
}

fn run_qualification(
    initial: &MaterialMesh,
    mode: ArmMode,
    steps: usize,
    mechanics: &MechParams,
    reaction: &ReactionParams,
    homeostat: &MetabolicHomeostatParamsV1,
) -> ArmResult {
    let mut mesh = initial.clone();
    let mut state = MetabolicHomeostatStateV1::default();
    let quarter = steps / 4;
    let mut q_snapshots = vec![snapshot(&mesh, 0)];
    let mut q_source = vec![0.0; 4];
    let mut q_demand = vec![0.0; 4];
    let mut q_error = vec![0.0; 4];
    let mut q_capacity = vec![0.0; 4];
    let mut hashes = vec![stable_json_hash(&snapshot(&mesh, 0)).unwrap()];
    let mut region = mode
        .inventory()
        .map(|n| FiniteSpatialResourceRegionV1::new([0.0, 0.0], 5.0, n, n));
    let transport = TransportParams::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut world_n_loss = 0.0;
    let mut world_f_loss = 0.0;
    let mut max_resource_error: f64 = 0.0;
    let mut finite_nonnegative = true;
    let mut max_e_stored = snapshot(&mesh, 0).e_stored;
    let initial_capacity = state.assimilation_capacity;
    for step in 0..steps {
        if matches!(mode, ArmMode::SustainedPrecursor) {
            mesh.interior.n = D016_CLAMP;
            mesh.interior.f = D016_CLAMP;
        }
        if let Some(resource) = region.as_mut() {
            let uptake = resource.uptake(&mut mesh, &transport, mechanics.dt);
            n_delivered += uptake.n_delivered;
            f_delivered += uptake.f_delivered;
            world_n_loss += uptake.n_world_loss;
            world_f_loss += uptake.f_world_loss;
            max_resource_error = max_resource_error.max(uptake.conservation_error.abs());
        }
        let before = mesh.interior;
        let before_capacity = state.assimilation_capacity;
        let ledger = if matches!(mode, ArmMode::ResourceFree) {
            reactions_step_with_homeostat(
                &mut mesh,
                reaction,
                mechanics.dt,
                true,
                true,
                homeostat,
                &mut state,
            )
        } else {
            reactions_step_with_homeostat(
                &mut mesh,
                reaction,
                mechanics.dt,
                true,
                true,
                homeostat,
                &mut state,
            )
        };
        let area = mesh.area().max(1.0e-15);
        let demand = inferred_decay(before, mesh.interior, &ledger, area)
            + ledger.a_consumed_build
            + ledger.c_produced
            + ledger.l_produced
            + ledger.reserve.r_to_w
            + ledger.reserve.r_to_m;
        let q = step / quarter;
        q_source[q] += ledger.a_produced;
        q_demand[q] += demand;
        let stored = snapshot(&mesh, step + 1);
        let error = ((homeostat.e_target - stored.e_stored) / homeostat.e_target).clamp(-1.0, 1.0);
        q_error[q] += error;
        q_capacity[q] += (before_capacity + state.assimilation_capacity) * 0.5;
        max_e_stored = max_e_stored.max(stored.e_stored);
        finite_nonnegative &= mesh.alive
            && [
                mesh.interior.a,
                mesh.interior.r,
                mesh.interior.n,
                mesh.interior.f,
                mesh.interior.c,
                mesh.interior.w,
                stored.e_stored,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= -EPS);
        hashes.push(stable_json_hash(&stored).unwrap());
        if (step + 1) % quarter == 0 {
            q_snapshots.push(stored);
        }
    }
    let q_slopes = q_snapshots
        .windows(2)
        .map(|w| (w[1].e_stored - w[0].e_stored) / (quarter as f64 * mechanics.dt))
        .collect::<Vec<_>>();
    let q_changes = q_snapshots
        .windows(2)
        .map(|w| w[1].e_stored - w[0].e_stored)
        .collect::<Vec<_>>();
    let n = quarter as f64;
    for i in 0..4 {
        q_error[i] /= n;
        q_capacity[i] /= n;
    }
    ArmResult {
        name: mode.name().into(),
        steps,
        initial: q_snapshots[0],
        final_state: *q_snapshots.last().unwrap(),
        quarter_snapshots: q_snapshots,
        quarter_e_stored_slopes: q_slopes,
        quarter_e_stored_changes: q_changes,
        quarter_source: q_source,
        quarter_demand: q_demand,
        quarter_error_mean: q_error,
        quarter_capacity_mean: q_capacity,
        initial_capacity,
        final_capacity: state.assimilation_capacity,
        max_capacity: state.assimilation_capacity.max(initial_capacity),
        max_e_stored,
        n_delivered,
        f_delivered,
        world_n_loss,
        world_f_loss,
        max_resource_error,
        resource_conservation: max_resource_error <= EPS
            && (world_n_loss - n_delivered).abs() <= EPS
            && (world_f_loss - f_delivered).abs() <= EPS,
        finite_nonnegative,
        alive: mesh.alive,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    }
}

fn run_legacy(
    initial: &MaterialMesh,
    steps: usize,
    reaction: &ReactionParams,
    mechanics: &MechParams,
) -> String {
    let mut mesh = initial.clone();
    let mut hashes = vec![stable_json_hash(&snapshot(&mesh, 0)).unwrap()];
    for step in 0..steps {
        reactions_step(&mut mesh, reaction, mechanics.dt, true, true);
        hashes.push(stable_json_hash(&snapshot(&mesh, step + 1)).unwrap());
    }
    stable_json_hash(&hashes).unwrap()
}

fn write_failure_diagnostic(
    out: &Path,
    m1: &ArmResult,
    m3: &ArmResult,
    m4: &ArmResult,
    homeostat: &MetabolicHomeostatParamsV1,
    control_q4: f64,
) {
    let capacity_saturated = m4.max_capacity >= 0.999999 * homeostat.capacity_max;
    let classification = if capacity_saturated {
        "DCDEV018_FAIL_CONTROLLER_CAPACITY_SATURATED"
    } else if m3.final_state.e_stored <= m3.initial.e_stored + EPS {
        "DCDEV018_FAIL_SOURCE_OUTPUT_INSUFFICIENT"
    } else if m4.quarter_e_stored_slopes[3].abs() > 0.01 * control_q4.abs() {
        "DCDEV018_FAIL_CONSTITUTIVE_DEMAND_DOMINANT"
    } else {
        "DCDEV018_FAIL_CAUSE_UNRESOLVED"
    };
    write(
        out,
        "failure_diagnostic.json",
        &json!({
            "finding":"DCDEV018_INTEGRAL_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED","classification":classification,
            "controller_capacity_max":homeostat.capacity_max,"m1_starvation":m1,"m3_derived_resource":m3,"m4_sustained_precursor":m4,
            "resource_free_control_q4_slope":control_q4,"quarter_matrix":{"source":"per-arm quarter_source","demand":"per-arm quarter_demand","e_stored":"per-arm quarter_snapshots"},
            "downstream_behavior_started":false,"parameter_tuning":false
        }),
    );
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
        && windows
            .iter()
            .all(|w| w.demand_max <= w.substrate_ceiling_min + EPS);
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
    write(
        &out,
        "source_feasibility.json",
        &serde_json::to_value(&result).unwrap(),
    );
    if !finite_capacity {
        write(
            &out,
            "protocol.json",
            &json!({"directive":"DC-DEV-018","phase":"0","entry_commit":ENTRY,"source_only_feasibility":feasibility,"downstream_started":false}),
        );
        write(
            &out,
            "artifact_manifest.json",
            &json!({"directive":"DC-DEV-018","phase":"0","entry_commit":ENTRY,"files":["protocol.json","source_feasibility.json","artifact_manifest.json"],"conclusion":feasibility,"next_execution_started":false}),
        );
        println!("{feasibility}\nG_cap_required={g_cap:.12e}");
        return;
    }

    let target = replete.e_stored;
    let homeostat = MetabolicHomeostatParamsV1::derived(target, g_cap - 1.0, 80.0);
    let legacy_hash = run_legacy(&deprived, STORAGE_STEPS, &p, &MechParams::default());
    let disabled = MetabolicHomeostatParamsV1::default();
    let feature_off = run_qualification(
        &deprived,
        ArmMode::ResourceFree,
        STORAGE_STEPS,
        &MechParams::default(),
        &p,
        &disabled,
    );
    let m1 = run_qualification(
        &deprived,
        ArmMode::ResourceFree,
        STORAGE_STEPS,
        &MechParams::default(),
        &p,
        &homeostat,
    );
    let m2 = run_qualification(
        &deprived,
        ArmMode::CurrentResource,
        480,
        &MechParams::default(),
        &p,
        &homeostat,
    );
    let m3 = run_qualification(
        &deprived,
        ArmMode::DerivedResource,
        480,
        &MechParams::default(),
        &p,
        &homeostat,
    );
    let m4 = run_qualification(
        &deprived,
        ArmMode::SustainedPrecursor,
        STORAGE_STEPS,
        &MechParams::default(),
        &p,
        &homeostat,
    );
    let feature_off_parity = legacy_hash == feature_off.trajectory_hash;
    let m1_zero_substrate = m1.quarter_source.iter().all(|v| v.abs() <= EPS);
    let m2_m3_conservation = m2.resource_conservation && m3.resource_conservation;
    let m3_distance_before = (target - m3.initial.e_stored).abs();
    let m3_distance_after = (target - m3.final_state.e_stored).abs();
    let finite_restore = m3.final_state.e_stored > deprived_snapshot.e_stored + EPS
        && m3_distance_after < m3_distance_before
        && ((m3.final_state.a - replete.a).abs() < (deprived_snapshot.a - replete.a).abs()
            || (m3.final_state.r - replete.r).abs() < (deprived_snapshot.r - replete.r).abs())
        && m3.alive
        && m3.finite_nonnegative
        && m3.max_e_stored.is_finite();
    let control_q4 = feature_off.quarter_e_stored_slopes[3];
    let sustained_homeostasis = m4.alive
        && m4.finite_nonnegative
        && m4.final_state.e_stored >= 0.95 * target
        && m4.final_state.e_stored <= 1.05 * target
        && m4.quarter_e_stored_slopes[3].abs() <= 0.01 * control_q4.abs()
        && m4.quarter_e_stored_changes[3] >= -EPS
        && m4.quarter_capacity_mean[3] < 0.95 * homeostat.capacity_max
        && m4.max_e_stored <= 1.10 * target;
    let metabolic_pass = feature_off_parity
        && m1_zero_substrate
        && m2_m3_conservation
        && finite_restore
        && sustained_homeostasis;
    if !metabolic_pass {
        write_failure_diagnostic(&out, &m1, &m3, &m4, &homeostat, control_q4);
    }
    let finding = if metabolic_pass {
        "DCDEV018_INTEGRAL_METABOLIC_HOMEOSTASIS_QUALIFIED"
    } else {
        "DCDEV018_INTEGRAL_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED"
    };
    write(
        &out,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-018","entry_commit":ENTRY,"source_branch":"strategy/dc-dev-016-metabolic-break-even",
            "phase":"0-2","observer_only":false,"settlement_steps":SETTLE,"deprivation_steps":480,"storage_horizon_steps":STORAGE_STEPS,
            "dt":DT,"target_e_stored":target,"target_precursor":D016_CLAMP,"capacity_max":homeostat.capacity_max,"k_integral":homeostat.k_integral,
            "arms":["M0_feature_off","M1_starvation_homeostat_on","M2_current_resource_homeostat_on","M3_derived_sufficient_resource_homeostat_on","M4_sustained_precursor_homeostat_on"],
            "parameter_sweep":false,"dcdev017_production_imported":false,"downstream_behavior_started":false
        }),
    );
    write(
        &out,
        "metabolic_results.json",
        &json!({
            "directive":"DC-DEV-018","entry_commit":ENTRY,"finding":finding,"target_e_stored":target,"homeostat_params":homeostat,
            "source_feasibility":result,"feature_off":{"legacy_hash":legacy_hash,"wrapper_hash":feature_off.trajectory_hash,"parity":feature_off_parity},
            "m1_starvation":m1,"m2_current_resource":m2,"m3_derived_resource":m3,"m4_sustained_precursor":m4,
            "gates":{"feature_off_parity":feature_off_parity,"zero_substrate_causality":m1_zero_substrate,"resource_conservation":m2_m3_conservation,"finite_resource_restoration":finite_restore,"sustained_homeostasis":sustained_homeostasis,"controller_unwinding":"not_run_until_metabolic_pass"},
            "downstream_phases_started":false,"next_execution_started":false
        }),
    );
    write(
        &out,
        "artifact_manifest.json",
        &json!({
            "directive":"DC-DEV-018","phase":"0-2","entry_commit":ENTRY,"files":["protocol.json","source_feasibility.json","metabolic_results.json","failure_diagnostic.json if metabolism fails","artifact_manifest.json"],"conclusion":"DCDEV018_INTEGRAL_HOMEOSTASIS_PERSISTENCE_BUNDLE_COMPLETE","overall_classification":if metabolic_pass {"DCDEV018_INTEGRAL_METABOLIC_HOMEOSTASIS_ONLY_QUALIFIED"} else {"DCDEV018_INTEGRAL_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED"},"production_behavior_changed":true,"chemistry_behavior_changed":true,"next_execution_started":false
        }),
    );
    println!("DCDEV018_INTEGRAL_HOMEOSTASIS_PERSISTENCE_BUNDLE_COMPLETE\n{finding}\nfeature_off_parity={feature_off_parity}\nfinite_resource_restoration={finite_restore}\nsustained_homeostasis={sustained_homeostasis}");
}
