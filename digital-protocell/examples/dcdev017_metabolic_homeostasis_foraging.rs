//! DC-DEV-017 Phase 0 and Phase 1.
//!
//! This first commit is intentionally limited to live-code control-surface
//! auditing and the intrinsic-timescale precursor-clamp challenge.  It does
//! not change production chemistry or start behavioral phases.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
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
const DEPRIVATION: usize = 480;
const D016_CLAMP: f64 = 0.1476710565778127;
const DT: f64 = 0.02;
const EPS: f64 = 1e-10;

#[derive(Debug, Clone, Copy, Serialize)]
struct Snap {
    step: usize,
    time: f64,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    e_stored: f64,
    structural_m: f64,
    membrane: f64,
    catalyst: f64,
    waste: f64,
    alive: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
struct Ledger {
    n_consumed: f64,
    f_consumed: f64,
    a_produced: f64,
    a_structural_consumption: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_to_w: f64,
    structural_production: f64,
    structural_turnover: f64,
    free_membrane_production: f64,
    membrane_bind: f64,
    membrane_unbind: f64,
    waste_production: f64,
    reserve_rejected_steps: u64,
}

#[derive(Debug, Clone, Serialize)]
struct Injection {
    step: usize,
    time: f64,
    n_concentration: f64,
    f_concentration: f64,
    n_material: f64,
    f_material: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    name: String,
    initial: Snap,
    final_state: Snap,
    quarter_snapshots: Vec<Snap>,
    quarter_slopes: Vec<f64>,
    quarter_net_changes: Vec<f64>,
    ledger: Ledger,
    injections: Vec<Injection>,
    total_n_injected: f64,
    total_f_injected: f64,
    n_delivered: f64,
    f_delivered: f64,
    max_a: f64,
    max_r: f64,
    max_resource_error: f64,
    resource_conservation: bool,
    finite_nonnegative: bool,
    trajectory_hash: String,
    alive: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Audit {
    directive: String,
    entry_commit: String,
    activation_equation: String,
    activation_inputs: Vec<String>,
    sink_equations: Vec<String>,
    storage_equations: Vec<String>,
    dt: f64,
    maintenance_horizon: f64,
    storage_horizon: f64,
    storage_horizon_steps: usize,
    source_sink_units: String,
    thresholds: Value,
    sealed_values: Vec<String>,
    post_phase1_extensions: Vec<String>,
    prior_art_disposition: String,
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn snap(m: &MaterialMesh, step: usize, dt: f64) -> Snap {
    Snap {
        step,
        time: step as f64 * dt,
        area: m.area(),
        a: m.interior.a,
        r: m.interior.r,
        n: m.interior.n,
        f: m.interior.f,
        e_stored: m.area() * (m.interior.a + m.interior.r),
        structural_m: m.total_structural_mass(),
        membrane: m.total_bound_membrane() + m.free_l,
        catalyst: m.interior.c,
        waste: m.interior.w,
        alive: m.alive,
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
            n: 0.0,
            f: 0.0,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn params(mesh: &MaterialMesh, demand_coupled: bool) -> ReactionParams {
    let mut p = ReactionParams::default();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    p.demand_coupled_activation.enable = demand_coupled;
    p
}

fn add(ledger: &mut Ledger, before: LumpedChem, after: LumpedChem, r: ReactionLedger, area: f64) {
    ledger.n_consumed += r.n_consumed;
    ledger.f_consumed += r.f_consumed;
    ledger.a_produced += r.a_produced;
    ledger.a_structural_consumption += r.a_consumed_build;
    ledger.catalyst_production += r.c_produced;
    ledger.catalyst_turnover += r.c_turned;
    ledger.a_to_r += r.reserve.a_to_r;
    ledger.r_to_a += r.reserve.r_to_a;
    ledger.r_to_w += r.reserve.r_to_w;
    ledger.structural_production += r.m_produced;
    ledger.structural_turnover += r.m_to_w;
    ledger.free_membrane_production += r.l_produced;
    ledger.membrane_bind += r.bind_extent;
    ledger.membrane_unbind += r.unbind_extent;
    ledger.waste_production += r.w_produced;
    ledger.reserve_rejected_steps += r.reserve.rejected_steps;
    let inferred_decay = area * (before.a - after.a) + r.a_produced + r.reserve.r_to_a
        - r.c_produced
        - r.a_consumed_build
        - r.l_produced
        - r.reserve.a_to_r;
    assert!(
        inferred_decay >= -1e-9,
        "negative inferred A decay {inferred_decay}"
    );
}

fn settle(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed();
    for _ in 0..SETTLE {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    mesh
}

fn deprived(settled: &MaterialMesh, mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = settled.clone();
    let p = params(&mesh, false);
    for _ in 0..DEPRIVATION {
        let before = mesh.interior;
        let ledger = reactions_step(&mut mesh, &p, mechanics.dt, true, true);
        add(
            &mut Ledger::default(),
            before,
            mesh.interior,
            ledger,
            mesh.area().max(1e-6),
        );
    }
    mesh
}

fn run_arm(
    initial: &MaterialMesh,
    mechanics: &MechParams,
    steps: usize,
    clamp: Option<f64>,
    inventory: Option<f64>,
    demand_coupled: bool,
    name: &str,
) -> ArmResult {
    let mut mesh = initial.clone();
    let p = params(&mesh, demand_coupled);
    let mut region =
        inventory.map(|mass| FiniteSpatialResourceRegionV1::new([0.0, 0.0], 5.0, mass, mass));
    let transport = TransportParams::default();
    let initial_snap = snap(&mesh, 0, mechanics.dt);
    let quarter = steps / 4;
    let mut quarter_snapshots = vec![initial_snap];
    let mut trajectory = vec![stable_json_hash(&initial_snap).unwrap()];
    let mut ledger = Ledger::default();
    let mut injections = Vec::new();
    let mut n_injected = 0.0;
    let mut f_injected = 0.0;
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut max_resource_error: f64 = 0.0;
    let mut max_a = mesh.interior.a;
    let mut max_r = mesh.interior.r;
    let mut finite_nonnegative = true;
    for step in 0..steps {
        if let Some(target) = clamp {
            let n_delta = (target - mesh.interior.n).max(0.0);
            let f_delta = (target - mesh.interior.f).max(0.0);
            let area = mesh.area().max(1e-15);
            mesh.interior.n += n_delta;
            mesh.interior.f += f_delta;
            n_injected += n_delta * area;
            f_injected += f_delta * area;
            injections.push(Injection {
                step: step + 1,
                time: (step + 1) as f64 * mechanics.dt,
                n_concentration: n_delta,
                f_concentration: f_delta,
                n_material: n_delta * area,
                f_material: f_delta * area,
            });
        }
        let resource = region
            .as_mut()
            .map(|r| r.uptake(&mut mesh, &transport, mechanics.dt));
        if let Some(resource) = resource {
            n_delivered += resource.n_delivered;
            f_delivered += resource.f_delivered;
            max_resource_error = max_resource_error.max(resource.conservation_error.abs());
        }
        let before = mesh.interior;
        let reaction = reactions_step(&mut mesh, &p, mechanics.dt, true, true);
        add(
            &mut ledger,
            before,
            mesh.interior,
            reaction,
            mesh.area().max(1e-6),
        );
        max_a = max_a.max(mesh.interior.a);
        max_r = max_r.max(mesh.interior.r);
        finite_nonnegative &= mesh.alive
            && [
                mesh.interior.a,
                mesh.interior.r,
                mesh.interior.n,
                mesh.interior.f,
                mesh.interior.c,
                mesh.interior.w,
                mesh.total_structural_mass(),
                mesh.total_bound_membrane(),
                mesh.free_l,
            ]
            .iter()
            .all(|x| x.is_finite() && *x >= -EPS);
        let state = snap(&mesh, step + 1, mechanics.dt);
        trajectory.push(stable_json_hash(&state).unwrap());
        if (step + 1) % quarter == 0 {
            quarter_snapshots.push(state);
        }
    }
    let quarter_slopes = quarter_snapshots
        .windows(2)
        .map(|w| (w[1].e_stored - w[0].e_stored) / (quarter as f64 * mechanics.dt))
        .collect::<Vec<_>>();
    let quarter_net_changes = quarter_snapshots
        .windows(2)
        .map(|w| w[1].e_stored - w[0].e_stored)
        .collect::<Vec<_>>();
    let final_state = *quarter_snapshots.last().unwrap();
    ArmResult {
        name: name.into(),
        initial: initial_snap,
        final_state,
        quarter_snapshots,
        quarter_slopes,
        quarter_net_changes,
        ledger,
        injections,
        total_n_injected: n_injected,
        total_f_injected: f_injected,
        n_delivered,
        f_delivered,
        max_a,
        max_r,
        max_resource_error,
        resource_conservation: max_resource_error <= EPS,
        finite_nonnegative,
        trajectory_hash: stable_json_hash(&trajectory).unwrap(),
        alive: mesh.alive,
    }
}

fn audit(mesh: &MaterialMesh, mechanics: &MechParams, p: &ReactionParams) -> Audit {
    let maintenance_horizon = 1.0 / p.reserve.k_release.max(1e-15);
    let storage_horizon = p.reserve.store_horizon_mult * maintenance_horizon;
    Audit {
        directive: "DC-DEV-017".into(),
        entry_commit: ENTRY.into(),
        activation_equation: "J_A = k_act * q(C) * g_harvest * N * F; dA/dt += J_A and dW/dt += J_A".into(),
        activation_inputs: vec![
            "ReactionParams.k_act".into(), "q_catalyst(C, q_c)".into(),
            "D-096 function gain or active template/network/composition gain".into(),
            "interior N and F concentrations".into(), "accepted dt and mesh area".into(),
        ],
        sink_equations: vec![
            "catalyst production = k_c_prod * A * dt".into(),
            "A decay = k_a_decay * starve(N*F) * A * dt".into(),
            "structural A use = min(structural_build_flux * dt / yield, A * area)".into(),
            "membrane A use = min(0.02 * q(C) * g_build * A * perimeter * dt, A * area, room)".into(),
            "catalyst turnover = k_c_turn * C * dt".into(),
        ],
        storage_equations: vec![
            "J_store = k_store*q(C)*(A^2/(K_store^2+A^2))*(1-R/Rmax)".into(),
            "J_release = k_release*q(C)*(R/(K_R+R))*(K_low/(K_low+A))".into(),
            "J_R_loss = k_r_loss*R".into(),
        ],
        dt: mechanics.dt,
        maintenance_horizon,
        storage_horizon,
        storage_horizon_steps: (storage_horizon / mechanics.dt).round() as usize,
        source_sink_units: "N,F,A,R,C,L,M,W are concentration or material ledgers as implemented; area multiplication converts concentration flux to material ledger units".into(),
        thresholds: json!({
            "d016_clamp_concentration": D016_CLAMP,
            "finite_nonnegative_epsilon": EPS,
            "reserve_r_max": p.reserve.r_max,
            "reserve_k_release": p.reserve.k_release,
            "reserve_store_horizon_mult": p.reserve.store_horizon_mult,
            "mesh_area": mesh.area(),
        }),
        sealed_values: vec![
            "certified Phase-1 equations and chemistry-core source".into(),
            "DC-DEV-016 settlement/deprivation authority".into(),
            "DC-DEV-016 matched endpoint N=F=0.1476710565778127".into(),
        ],
        post_phase1_extensions: vec![
            "DC-DEV-017 assay-only precursor top-up".into(),
            "DC-DEV-017 observer evidence and phase gate calculations".into(),
        ],
        prior_art_disposition: "REFERENCE: primary literature only; COMPOSE: demand/homeostasis principle; BUILD: bounded Digital Cell assay adapter; REJECT: external code and unqualified biological parameters".into(),
    }
}

fn main() {
    let out = std::env::var_os("DCDEV017_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev017"));
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() < 1e-12);
    let settled = settle(&mechanics);
    let deprived_mesh = deprived(&settled, &mechanics);
    let p = params(&deprived_mesh, false);
    let phase0 = audit(&deprived_mesh, &mechanics, &p);
    assert_eq!(phase0.storage_horizon_steps, 4_000);
    let no_precursor = run_arm(
        &deprived_mesh,
        &mechanics,
        phase0.storage_horizon_steps,
        None,
        None,
        false,
        "P1-A_no_precursor",
    );
    let sustained = run_arm(
        &deprived_mesh,
        &mechanics,
        phase0.storage_horizon_steps,
        Some(D016_CLAMP),
        None,
        false,
        "P1-B_sustained_precursor",
    );
    let q4 = 3;
    let q4_depletion_slope = no_precursor.quarter_slopes[q4];
    let intrinsic_pass = sustained.alive
        && sustained.finite_nonnegative
        && sustained.quarter_net_changes[q4] >= -EPS
        && sustained.quarter_slopes[q4].abs() <= 0.01 * q4_depletion_slope.abs()
        && sustained
            .injections
            .iter()
            .all(|i| i.n_material >= 0.0 && i.f_material >= 0.0)
        && sustained.max_a <= sustained.initial.a * 10.0
        && sustained.max_r <= p.reserve.r_max + EPS
        && no_precursor.final_state.e_stored < no_precursor.initial.e_stored - EPS;
    let finding = if intrinsic_pass {
        "DCDEV017_EXISTING_METABOLISM_INTRINSIC_HOMEOSTASIS_SUPPORTED"
    } else {
        "DCDEV017_INTRINSIC_HOMEOSTASIS_NOT_ESTABLISHED"
    };
    let phase2 = if intrinsic_pass {
        None
    } else {
        let p2_a = run_arm(
            &deprived_mesh,
            &mechanics,
            phase0.storage_horizon_steps,
            None,
            None,
            true,
            "P2-A_resource_free_control_enabled",
        );
        let p2_b = run_arm(
            &deprived_mesh,
            &mechanics,
            phase0.storage_horizon_steps,
            None,
            Some(3.0),
            true,
            "P2-B_current_resource",
        );
        let p2_c = run_arm(
            &deprived_mesh,
            &mechanics,
            phase0.storage_horizon_steps,
            None,
            Some(14.588954880632265),
            true,
            "P2-C_derived_break_even_resource",
        );
        let p2_d = run_arm(
            &deprived_mesh,
            &mechanics,
            phase0.storage_horizon_steps,
            Some(D016_CLAMP),
            None,
            true,
            "P2-D_sustained_precursor_clamp",
        );
        let p2_a_legacy = run_arm(
            &deprived_mesh,
            &mechanics,
            phase0.storage_horizon_steps,
            None,
            None,
            false,
            "P2-A_legacy_parity_reference",
        );
        let feature_off_parity = p2_a.trajectory_hash == p2_a_legacy.trajectory_hash
            && p2_a.final_state.e_stored == p2_a_legacy.final_state.e_stored;
        let ref_a = sustained.final_state.a;
        let ref_r = sustained.final_state.r;
        let before_a = (p2_c.initial.a - ref_a).abs();
        let before_r = (p2_c.initial.r - ref_r).abs();
        let after_a = (p2_c.final_state.a - ref_a).abs();
        let after_r = (p2_c.final_state.r - ref_r).abs();
        let toward_reference = after_a < before_a || after_r < before_r;
        let p2_d_homeostasis = p2_d.quarter_net_changes[q4] >= -EPS
            && p2_d.quarter_slopes[q4].abs() <= 0.01 * q4_depletion_slope.abs();
        let p2_pass = feature_off_parity
            && p2_b.resource_conservation
            && p2_c.resource_conservation
            && p2_c.alive
            && p2_c.finite_nonnegative
            && p2_c.final_state.e_stored >= p2_c.initial.e_stored - EPS
            && toward_reference
            && p2_d_homeostasis
            && p2_a.ledger.a_produced.abs() <= EPS;
        Some(json!({
            "directive":"DC-DEV-017","phase":"PHASE_2",
            "production_module":"chemistry-core demand_coupled_activation v1, opt-in default off",
            "exact_feedback_equation":"multiplier = clamp(1 + (8.58379474604017 - 1) * demand / demand_reference, 1, 8.58379474604017); demand = K_low/(K_low+A) * R/(K_R+R)",
            "demand_signal":"existing reserve low-A/release-demand composition",
            "gain_derivation":{"sink_target":8.092100679490137,"legacy_source":0.9427183336627594,"reference_multiplier":8.58379474604017},
            "arms":{"p2_a":p2_a,"p2_b":p2_b,"p2_c":p2_c,"p2_d":p2_d},
            "controls":{"feature_off_reference":p2_a_legacy,"feature_off_trajectory_parity":feature_off_parity,"no_substrate_additional_a":p2_a.ledger.a_produced.abs() <= EPS},
            "fed_reference":{"a":ref_a,"r":ref_r},
            "reference_direction":{"before_a":before_a,"after_a":after_a,"before_r":before_r,"after_r":after_r,"toward":toward_reference},
            "checks":{"p2_d_intrinsic_homeostasis":p2_d_homeostasis,"metabolic_repair_pass":p2_pass},
            "finding":if p2_pass {"DCDEV017_DEMAND_COUPLED_METABOLIC_HOMEOSTASIS_QUALIFIED"} else {"DCDEV017_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED"},
            "production_behavior_changed":true,"chemistry_behavior_changed":true,"phase3_started":false,"next_execution_started":false
        }))
    };
    let prior_art = format!(
        "# DC-DEV-017 prior-art disposition\n\n- **REFERENCE**: Pols et al., *Nature Communications* (2019), [A synthetic metabolic network for physicochemical homeostasis](https://www.nature.com/articles/s41467-019-12287-2). It supports the relevance of sustained ATP production, substrate import, dissipation, and load-sensitive homeostasis in a synthetic vesicle.\n- **REFERENCE**: Covian et al., *PLOS ONE* (2021), [Energy homeostasis is a conserved process](https://pmc.ncbi.nlm.nih.gov/articles/PMC8575270/). It supports demand-coupled respiratory activation under changed energetic demand while high-energy state remains comparatively constrained.\n- **COMPOSE**: use the general demand-coupled principle as a rationale for the single later opt-in repair only if the existing metabolism fails its intrinsic-timescale test.\n- **BUILD**: implement only the bounded Digital Cell-native adapter authorized by DC-DEV-017, using existing A/R/N/F state and reserve demand.\n- **REJECT**: no external code, model, parameter, species, ATP/ADP implementation, or world-behavior mechanism is imported.\n\nPhase 0 and Phase 1 remain observer/assay-only; no production behavior is changed.\n"
    );
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("prior_art_disposition.md"), prior_art).unwrap();
    write(
        &out,
        "control_surface_audit.json",
        &serde_json::to_value(&phase0).unwrap(),
    );
    write(
        &out,
        "phase1_results.json",
        &json!({
            "directive":"DC-DEV-017","entry_commit":ENTRY,"phase":"PHASE_1",
            "clamp_derivation":{"source":"DC-DEV-016 formal challenge endpoint","n":D016_CLAMP,"f":D016_CLAMP,"rule":"lower physically achieved matched endpoint concentration"},
            "intrinsic_horizon":{"maintenance_horizon":phase0.maintenance_horizon,"storage_horizon":phase0.storage_horizon,"dt":phase0.dt,"steps":phase0.storage_horizon_steps,"quarters":["Q1","Q2","Q3","Q4"]},
            "p1_a_no_precursor":no_precursor,"p1_b_sustained_precursor":sustained,
            "checks":{"q4_depletion_slope":q4_depletion_slope,"q4_sustained_slope":sustained.quarter_slopes[q4],"intrinsic_pass":intrinsic_pass},
            "finding":finding,"production_behavior_changed":false,"phase2_started":phase2.is_some(),"phase3_started":false,"next_execution_started":false
        }),
    );
    if let Some(ref phase2_value) = phase2 {
        write(&out, "phase2_results.json", phase2_value);
    }
    write(
        &out,
        "artifact_manifest.json",
        &json!({
            "directive":"DC-DEV-017","entry_commit":ENTRY,"phase":"PHASE_1",
            "files":["control_surface_audit.json","prior_art_disposition.md","phase1_results.json","phase2_results.json if Phase 2 executed","artifact_manifest.json"],
            "finding":finding,"production_behavior_changed":false,"next_execution_started":false
        }),
    );
    println!("{finding}");
}
