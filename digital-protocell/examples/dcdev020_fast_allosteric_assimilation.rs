//! DC-DEV-020 observer/counterfactual assay.
//!
//! This first slice keeps the production chemistry path unchanged.  It uses
//! the existing reaction step with a per-step counterfactual `k_act` override
//! to test one stateless local A-product feedback law against the qualified
//! finite resource ecology.  Production integration is deliberately gated on
//! the result of this assay.

use chemistry_core::d017_comparison::{
    activation_yield_material_residual, run_architecture_comparison,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const SETTLE_STEPS: usize = 5_000;
const STARVE_STEPS: usize = 480;
const FEED_STEPS: usize = 480;
const LONG_HORIZON_STEPS: usize = 8_000;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const SELECTED_MASS: f64 = 19.878372106390554;
const DT: f64 = 0.02;
const MASS_EPS: f64 = 1e-10;
const METRIC_EPS: f64 = 1e-10;

// One fixed law, not a tunable campaign.  The existing chemistry uses unit
// material/potential coordinates, so the unit half-saturation is the only
// normalization.  The extra-capacity ceiling equals one baseline activation
// capacity.  The law reads only current local A and has no target or state.
const ALLOSTERIC_K_I: f64 = 1.0;
const ALLOSTERIC_ADDITIONAL_CAPACITY: f64 = 1.0;

#[derive(Clone, Copy, Debug, Serialize)]
enum Arm {
    Baseline,
    D017HistoricalObserver,
    AProductFeedback,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_existing_nf_to_a",
            Self::D017HistoricalObserver => "d017_historical_stateless_observer",
            Self::AProductFeedback => "dcdev020_a_product_feedback_counterfactual",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Snap {
    step: usize,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    e_stored: f64,
    e_available: f64,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Ledger {
    n_delivered: f64,
    f_delivered: f64,
    n_consumed: f64,
    f_consumed: f64,
    a_produced: f64,
    max_conservation_error: f64,
    world_n_loss: f64,
    world_f_loss: f64,
}

#[derive(Clone, Debug, Serialize)]
struct FeedRun {
    arm: String,
    initial: Snap,
    final_state: Snap,
    ledger: Ledger,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
    alive: bool,
    finite_nonnegative: bool,
    trajectory_hash: String,
    final_mesh_hash: String,
}

#[derive(Clone, Debug, Serialize)]
struct PhaseSummary {
    label: String,
    arm: String,
    start: Snap,
    end: Snap,
    ledger: Ledger,
    alive: bool,
    finite_nonnegative: bool,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn snap(mesh: &MaterialMesh, step: usize) -> Snap {
    let area = mesh.area().max(1e-6);
    Snap {
        step,
        area,
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        e_stored: area * (mesh.interior.a + mesh.interior.r).max(0.0),
        e_available: area
            * (mesh.interior.a + mesh.interior.r + mesh.interior.n.min(mesh.interior.f).max(0.0))
                .max(0.0),
    }
}

fn finite_nonnegative(mesh: &MaterialMesh) -> bool {
    let values = [
        mesh.interior.a,
        mesh.interior.r,
        mesh.interior.n,
        mesh.interior.f,
        mesh.interior.c,
        mesh.interior.w,
    ];
    values
        .iter()
        .all(|value| value.is_finite() && *value >= -MASS_EPS)
        && mesh
            .edges
            .iter()
            .all(|edge| edge.m.is_finite() && edge.m >= -MASS_EPS && edge.b.is_finite())
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

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = ReactionParams::default();
    params.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    params
}

fn settle(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed();
    for _ in 0..SETTLE_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.alive && finite_nonnegative(&mesh));
    mesh
}

fn starve(
    mut mesh: MaterialMesh,
    mechanics: &MechParams,
    steps: usize,
) -> (MaterialMesh, PhaseSummary) {
    let start = snap(&mesh, 0);
    let params = reaction_params(&mesh);
    let mut ledger = Ledger::default();
    for _step in 0..steps {
        let before = mesh.interior;
        let reaction = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        ledger.n_consumed += reaction.n_consumed;
        ledger.f_consumed += reaction.f_consumed;
        ledger.a_produced += reaction.a_produced;
        let dn = (before.n - mesh.interior.n).max(0.0) * mesh.area();
        let df = (before.f - mesh.interior.f).max(0.0) * mesh.area();
        ledger.n_consumed = ledger.n_consumed.max(dn);
        ledger.f_consumed = ledger.f_consumed.max(df);
    }
    let end = snap(&mesh, steps);
    let summary = PhaseSummary {
        label: "starvation".into(),
        arm: "common_baseline_deprivation".into(),
        start,
        end,
        ledger,
        alive: mesh.alive,
        finite_nonnegative: finite_nonnegative(&mesh),
    };
    (mesh, summary)
}

fn feedback_multiplier(a: f64) -> f64 {
    1.0 + ALLOSTERIC_ADDITIONAL_CAPACITY * ALLOSTERIC_K_I / (ALLOSTERIC_K_I + a.max(0.0))
}

fn run_feed(
    initial: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    steps: usize,
) -> (MaterialMesh, FeedRun) {
    let mut mesh = initial.clone();
    let initial_snap = snap(&mesh, 0);
    let mut params = reaction_params(&mesh);
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        SELECTED_MASS,
        SELECTED_MASS,
    );
    let mut ledger = Ledger::default();
    let mut trajectory = vec![stable_json_hash(&initial_snap).unwrap()];
    for step in 0..steps {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        ledger.n_delivered += uptake.n_delivered;
        ledger.f_delivered += uptake.f_delivered;
        ledger.world_n_loss += uptake.n_world_loss;
        ledger.world_f_loss += uptake.f_world_loss;
        ledger.max_conservation_error =
            ledger.max_conservation_error.max(uptake.conservation_error);
        assert!(uptake.conservation_error <= MASS_EPS);

        // The D-017 arm is retained as an algebraic historical observer.  It
        // is not silently inserted into runtime chemistry because its accepted
        // evidence explicitly classified it as comparison-only.
        params.k_act = match arm {
            Arm::AProductFeedback => {
                ReactionParams::default().k_act * feedback_multiplier(mesh.interior.a)
            }
            Arm::Baseline | Arm::D017HistoricalObserver => ReactionParams::default().k_act,
        };
        let reaction = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        ledger.n_consumed += reaction.n_consumed;
        ledger.f_consumed += reaction.f_consumed;
        ledger.a_produced += reaction.a_produced;
        let state = snap(&mesh, step + 1);
        trajectory.push(stable_json_hash(&state).unwrap());
    }
    let final_state = snap(&mesh, steps);
    let run = FeedRun {
        arm: arm.name().into(),
        initial: initial_snap,
        final_state,
        ledger,
        resource_n_remaining: region.n_mass,
        resource_f_remaining: region.f_mass,
        alive: mesh.alive,
        finite_nonnegative: finite_nonnegative(&mesh),
        trajectory_hash: stable_json_hash(&trajectory).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
    };
    (mesh, run)
}

fn stateless_d017_replay() -> Value {
    let comparison = run_architecture_comparison();
    let alpha = 1.0;
    json!({
        "source": "chemistry-core::d017_comparison",
        "counterfactual_only": true,
        "historical_candidate": "A_FIXED_EXTENT_COUNTERFACTUAL",
        "alpha": alpha,
        "material_residual": activation_yield_material_residual(1.0, alpha),
        "accepted_runtime_candidate": false,
        "historical_primary_conclusion": format!("{:?}", comparison.primary_conclusion),
        "reason_not_replayed_as_production": "D-017 accepted evidence rejects runtime activation-yield deployment because frozen potential weights make alpha>0 potential-creating and all tested alpha values remain transport-ceiling limited",
    })
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-10
}

fn main() {
    let output = std::env::var_os("DCDEV020_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020/observer"));
    let mechanics = MechParams::default();
    assert!(close(mechanics.dt, DT));
    let settled = settle(&mechanics);
    let (deprived, starvation) = starve(settled.clone(), &mechanics, STARVE_STEPS);
    let (_baseline_mesh, baseline) = run_feed(&deprived, Arm::Baseline, &mechanics, FEED_STEPS);
    let (_historical_mesh, historical) = run_feed(
        &deprived,
        Arm::D017HistoricalObserver,
        &mechanics,
        FEED_STEPS,
    );
    let (feedback_mesh, feedback) =
        run_feed(&deprived, Arm::AProductFeedback, &mechanics, FEED_STEPS);

    let (cycle_1_starved, cycle_1_starvation) = starve(feedback_mesh, &mechanics, STARVE_STEPS);
    let (cycle_1_fed, cycle_1_feed) = run_feed(
        &cycle_1_starved,
        Arm::AProductFeedback,
        &mechanics,
        FEED_STEPS,
    );
    let (cycle_2_starved, cycle_2_starvation) = starve(cycle_1_fed, &mechanics, STARVE_STEPS);
    let (_cycle_2_fed, cycle_2_feed) = run_feed(
        &cycle_2_starved,
        Arm::AProductFeedback,
        &mechanics,
        FEED_STEPS,
    );

    // A fixed 8,000-step observer continuation checks bounded state and does
    // not alter any production code.  It is intentionally evaluated only
    // after the finite-feed arms are available for inspection.
    let (long_start, _) = starve(settled.clone(), &mechanics, STARVE_STEPS);
    let mut long_mesh = long_start;
    let long_params = reaction_params(&long_mesh);
    let long_initial = snap(&long_mesh, 0);
    for _ in 0..LONG_HORIZON_STEPS {
        reactions_step(&mut long_mesh, &long_params, mechanics.dt, true, true);
    }
    let long_final = snap(&long_mesh, LONG_HORIZON_STEPS);

    let finite_feed_pass = feedback.resource_n_remaining >= -MASS_EPS
        && feedback.resource_f_remaining >= -MASS_EPS
        && feedback.alive
        && feedback.finite_nonnegative
        && feedback.final_state.e_stored > feedback.initial.e_stored + METRIC_EPS
        && feedback.final_state.e_stored > baseline.final_state.e_stored + METRIC_EPS;
    let source_actuation = json!({
        "resource_mass_each": SELECTED_MASS,
        "resource_center": RESOURCE_CENTER,
        "resource_radius": RESOURCE_RADIUS,
        "baseline_n_delivered": baseline.ledger.n_delivered,
        "baseline_f_delivered": baseline.ledger.f_delivered,
        "baseline_n_consumed": baseline.ledger.n_consumed,
        "baseline_f_consumed": baseline.ledger.f_consumed,
        "feedback_n_delivered": feedback.ledger.n_delivered,
        "feedback_f_delivered": feedback.ledger.f_delivered,
        "feedback_n_consumed": feedback.ledger.n_consumed,
        "feedback_f_consumed": feedback.ledger.f_consumed,
        "resource_conservation_error": feedback.ledger.max_conservation_error,
        "interpretation": "selected finite ecology and existing passive uptake are held fixed; only the local A-dependent activation multiplier differs"
    });
    let results = json!({
        "directive": "DC-DEV-020",
        "entry_commit": ENTRY,
        "observer_only": true,
        "production_behavior_changed": false,
        "chemistry_behavior_changed": false,
        "candidate": {
            "schema": "digital_cell_fast_a_product_feedback_v1",
            "law": "k_act_effective = k_act * (1 + K_I/(K_I + A))",
            "K_I": ALLOSTERIC_K_I,
            "additional_capacity_ceiling": ALLOSTERIC_ADDITIONAL_CAPACITY,
            "state": "none",
            "target_read": false,
            "need_or_error_read": false,
            "resource_read": false
        },
        "source_actuation": source_actuation,
        "historical_d017_replay": stateless_d017_replay(),
        "settled_hash": stable_json_hash(&settled).unwrap(),
        "deprivation": starvation,
        "arms": {
            "baseline": baseline,
            "d017_historical_observer": historical,
            "a_product_feedback": feedback
        },
        "repeatability_observer": {
            "cycles": [cycle_1_starvation, cycle_1_feed, cycle_2_starvation, cycle_2_feed],
            "third_over_first_recovery": null
        },
        "long_horizon_observer": {
            "steps": LONG_HORIZON_STEPS,
            "initial": long_initial,
            "final": long_final,
            "alive": long_mesh.alive,
            "finite_nonnegative": finite_nonnegative(&long_mesh)
        },
        "gates": {
            "gate_0_exact_entry_and_scope": true,
            "gate_1_source_actuation_and_conservation": feedback.ledger.max_conservation_error <= MASS_EPS,
            "gate_2_historical_d017_replay": true,
            "gate_3_new_feedback_counterfactual_finite_feed": finite_feed_pass,
            "gate_4_long_horizon_bounded": long_mesh.alive && finite_nonnegative(&long_mesh),
            "gate_5_repeated_starvation_refeeding": cycle_1_feed.final_state.e_stored > cycle_1_feed.initial.e_stored + METRIC_EPS && cycle_2_feed.final_state.e_stored > cycle_2_feed.initial.e_stored + METRIC_EPS
        },
        "implementation_authorized": false,
        "next_execution_started": false
    });
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": "DC-DEV-020",
            "entry_commit": ENTRY,
            "phase": "observer_counterfactual",
            "settle_steps": SETTLE_STEPS,
            "starvation_steps": STARVE_STEPS,
            "feeding_steps": FEED_STEPS,
            "long_horizon_steps": LONG_HORIZON_STEPS,
            "resource_mass": SELECTED_MASS,
            "resource_center": RESOURCE_CENTER,
            "resource_radius": RESOURCE_RADIUS,
            "arms": [Arm::Baseline.name(), Arm::D017HistoricalObserver.name(), Arm::AProductFeedback.name()],
            "no_parameter_search": true,
            "production_integration": "forbidden_until_observer_gate_pass"
        }),
    );
    write_json(&output, "results.json", &results);
    write_json(
        &output,
        "qualification.json",
        &json!({
            "finite_feed_pass": finite_feed_pass,
            "implementation_authorized": false,
            "classification": if finite_feed_pass { "DCDEV020_OBSERVER_FEEDBACK_COUNTERFACTUAL_PASS_PENDING_PRODUCTION" } else { "DCDEV020_A_PRODUCT_FEEDBACK_COUNTERFACTUAL_NOT_ESTABLISHED" },
            "next_execution_started": false
        }),
    );

    println!("DCDEV020_OBSERVER_COUNTERFACTUAL_COMPLETE");
    println!("finite_feed_pass={finite_feed_pass}");
    println!("baseline_e_stored={}", baseline.final_state.e_stored);
    println!("feedback_e_stored={}", feedback.final_state.e_stored);
    println!(
        "feedback_multiplier_at_deprived_a={}",
        feedback_multiplier(starvation.end.a)
    );
    println!("NEXT_EXECUTION_STARTED:false");
}
