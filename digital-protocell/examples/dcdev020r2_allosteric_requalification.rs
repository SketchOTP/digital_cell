//! DC-DEV-020-R2 observer-only requalification.
//!
//! This example preserves the DC-DEV-020 provisional result and adds the
//! missing protocol layers: source-capacity reconstruction, the actual
//! DC-DEV-017 demand-coupled observer replay, reaction-sequencing accounting,
//! a deterministic A-only fit, and conditional sustained/cyclic assays.
//! Nothing here changes production chemistry.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const PRIOR_HEAD: &str = "876012f8888b074285c55167613471a59d4be25d";
const SETTLE_STEPS: usize = 5_000;
const DEPRIVATION_STEPS: usize = 480;
const FEED_STEPS: usize = 480;
const SUSTAINED_STEPS: usize = 8_000;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const SELECTED_MASS: f64 = 19.878372106390554;
const SUSTAINED_NF: f64 = 0.1476710565778127;
const E_TARGET: f64 = 77.91027880846893;
const DT: f64 = 0.02;
const MASS_EPS: f64 = 1e-10;
const SOURCE_EPS: f64 = 1e-12;

const PRIOR_K_I: f64 = 1.0;
const PRIOR_ADDITIONAL_CAPACITY: f64 = 1.0;

// Sealed August DC-DEV-017 observer constants.  The July D-017 comparison
// is a different historical project and is not used by this assay.
const DCDEV017_MAX_GAIN: f64 = 8.58379474604017;
const DCDEV017_DEMAND_REFERENCE: f64 = 0.9427183336627594;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
enum Arm {
    Baseline,
    PriorTwoX,
    DCDev017Observer,
    ConstantGain,
    SourceSaturated,
    DerivedAFeedback,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_existing_nf_to_a",
            Self::PriorTwoX => "dcdev020_prior_exact_two_x_candidate",
            Self::DCDev017Observer => "dcdev017_demand_coupled_activation_v1_observer",
            Self::ConstantGain => "constant_break_even_gain",
            Self::SourceSaturated => "source_saturated_upper_bound",
            Self::DerivedAFeedback => "dcdev020r2_derived_a_feedback_observer",
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
    alive: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Ledger {
    n_delivered: f64,
    f_delivered: f64,
    n_consumed: f64,
    f_consumed: f64,
    a_produced: f64,
    a_decay: f64,
    accelerated_a_decay: f64,
    accelerated_steps: usize,
    a_to_r: f64,
    r_to_a: f64,
    r_to_w: f64,
    catalyst_a_consumption: f64,
    structural_a_consumption: f64,
    membrane_a_consumption: f64,
    max_conservation_error: f64,
    world_n_loss: f64,
    world_f_loss: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SourceStep {
    step: usize,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    ordinary_requested_extent: f64,
    ordinary_accepted_extent: f64,
    source_saturated_accepted_extent: f64,
    effective_gain_required: f64,
    applied_gain: f64,
    a_decay: f64,
    accelerated_a_decay: f64,
    catalyst_a_consumption: f64,
    structural_a_consumption: f64,
    membrane_a_consumption: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_loss: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
    arm: String,
    constant_gain: Option<f64>,
    initial: Snap,
    final_state: Snap,
    ledger: Ledger,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
    alive: bool,
    finite_nonnegative: bool,
    trajectory_hash: String,
    final_mesh_hash: String,
    #[serde(skip_serializing)]
    envelope: Vec<SourceStep>,
}

#[derive(Clone, Debug, Serialize)]
struct Fit {
    g_max: f64,
    k_a: f64,
    n: f64,
    train_relative_rmse: f64,
    holdout_relative_rmse: f64,
    train_points: usize,
    holdout_points: usize,
    required_gain_decreases_with_a: bool,
    holdout_phase_bias: f64,
    bounded_positive: bool,
}

#[derive(Clone, Debug, Serialize)]
struct SustainedSummary {
    arm: String,
    initial: Snap,
    final_state: Snap,
    quarter_e_stored: [f64; 4],
    quarter_slopes: [f64; 4],
    peak_e_stored: f64,
    mean_gain_last_quarter: f64,
    max_gain: f64,
    ledger: Ledger,
    alive: bool,
    finite_nonnegative: bool,
}

#[derive(Clone, Debug, Serialize)]
struct CycleSummary {
    cycle: usize,
    deprived_start_e_stored: f64,
    recovered_e_stored: f64,
    alive: bool,
    conservation_error: f64,
    gain_first_feed: f64,
    gain_last_feed: f64,
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
        alive: mesh.alive,
    }
}

fn finite_nonnegative(mesh: &MaterialMesh) -> bool {
    [
        mesh.interior.a,
        mesh.interior.r,
        mesh.interior.n,
        mesh.interior.f,
        mesh.interior.c,
        mesh.interior.w,
    ]
    .iter()
    .all(|v| v.is_finite() && *v >= -MASS_EPS)
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

fn accumulate(
    ledger: &mut Ledger,
    before: LumpedChem,
    after: LumpedChem,
    reaction: &ReactionLedger,
    area: f64,
) {
    ledger.n_consumed += reaction.n_consumed;
    ledger.f_consumed += reaction.f_consumed;
    ledger.a_produced += reaction.a_produced;
    ledger.a_to_r += reaction.reserve.a_to_r;
    ledger.r_to_a += reaction.reserve.r_to_a;
    ledger.r_to_w += reaction.reserve.r_to_w;
    ledger.catalyst_a_consumption += reaction.c_produced;
    ledger.structural_a_consumption += reaction.a_consumed_build;
    ledger.membrane_a_consumption += reaction.l_produced;
    ledger.a_decay += inferred_a_decay(before, after, reaction, area);
}

fn inferred_a_decay(
    before: LumpedChem,
    after: LumpedChem,
    reaction: &ReactionLedger,
    area: f64,
) -> f64 {
    (before.a * area + reaction.a_produced
        - reaction.c_produced
        - after.a * area
        - reaction.a_consumed_build
        - reaction.l_produced
        - reaction.reserve.a_to_r
        + reaction.reserve.r_to_a)
        .max(0.0)
}

fn reaction_with_gain(
    mesh: &mut MaterialMesh,
    base: &ReactionParams,
    dt: f64,
    gain: f64,
) -> ReactionLedger {
    let mut params = *base;
    params.k_act = base.k_act * gain.max(0.0);
    reactions_step(mesh, &params, dt, true, true)
}

fn ordinary_requested_extent(mesh: &MaterialMesh, params: &ReactionParams, dt: f64) -> f64 {
    params.k_act * mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) * dt * mesh.area().max(1e-6)
}

fn source_saturated_preview(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    dt: f64,
) -> (ReactionLedger, f64, ReactionLedger) {
    let area = mesh.area().max(1e-15);
    let mut ordinary_mesh = mesh.clone();
    let ordinary = reaction_with_gain(&mut ordinary_mesh, params, dt, 1.0);
    let capacity = (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area);
    let gain = if ordinary.n_consumed > SOURCE_EPS {
        (capacity / ordinary.n_consumed).max(1.0)
    } else {
        1.0
    };
    let mut saturated_mesh = mesh.clone();
    let saturated = reaction_with_gain(&mut saturated_mesh, params, dt, gain);
    (ordinary, gain, saturated)
}

fn prior_two_x(a: f64) -> f64 {
    1.0 + PRIOR_ADDITIONAL_CAPACITY * PRIOR_K_I / (PRIOR_K_I + a.max(0.0))
}

fn dcdev017_gain(mesh: &MaterialMesh, params: &ReactionParams) -> f64 {
    let demand = params.reserve.k_low / (params.reserve.k_low + mesh.interior.a.max(0.0))
        * mesh.interior.r.max(0.0)
        / (params.reserve.k_r + mesh.interior.r.max(0.0));
    (1.0 + (DCDEV017_MAX_GAIN - 1.0) * demand / DCDEV017_DEMAND_REFERENCE)
        .clamp(1.0, DCDEV017_MAX_GAIN)
}

fn fit_gain(a: f64, fit: &Fit) -> f64 {
    1.0 + (fit.g_max - 1.0) / (1.0 + (a.max(SOURCE_EPS) / fit.k_a).powf(fit.n))
}

fn gain_for(arm: Arm, mesh: &MaterialMesh, params: &ReactionParams, fit: Option<&Fit>) -> f64 {
    match arm {
        Arm::Baseline | Arm::SourceSaturated => 1.0,
        Arm::PriorTwoX => prior_two_x(mesh.interior.a),
        Arm::DCDev017Observer => dcdev017_gain(mesh, params),
        Arm::ConstantGain => fit.map(|f| f.g_max).unwrap_or(1.0),
        Arm::DerivedAFeedback => fit.map(|f| fit_gain(mesh.interior.a, f)).unwrap_or(1.0),
    }
}

fn run_feed(
    initial: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    fit: Option<&Fit>,
) -> (MaterialMesh, RunSummary) {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let initial_snap = snap(&mesh, 0);
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        SELECTED_MASS,
        SELECTED_MASS,
    );
    let transport = TransportParams::default();
    let mut ledger = Ledger::default();
    let mut envelope = Vec::with_capacity(FEED_STEPS);
    let mut hashes = vec![stable_json_hash(&initial_snap).unwrap()];
    for step in 0..FEED_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        ledger.n_delivered += uptake.n_delivered;
        ledger.f_delivered += uptake.f_delivered;
        ledger.world_n_loss += uptake.n_world_loss;
        ledger.world_f_loss += uptake.f_world_loss;
        ledger.max_conservation_error =
            ledger.max_conservation_error.max(uptake.conservation_error);
        assert!(uptake.conservation_error <= MASS_EPS);

        let before = mesh.interior;
        let ordinary_requested = ordinary_requested_extent(&mesh, &params, mechanics.dt);
        let (ordinary, required_gain, saturated) =
            source_saturated_preview(&mesh, &params, mechanics.dt);
        let applied_gain = if arm == Arm::SourceSaturated {
            required_gain
        } else {
            gain_for(arm, &mesh, &params, fit)
        };
        let reaction = reaction_with_gain(&mut mesh, &params, mechanics.dt, applied_gain);
        let accelerated = mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) < 1e-8;
        let a_decay = inferred_a_decay(before, mesh.interior, &reaction, mesh.area().max(1e-6));
        let accelerated_decay = if accelerated { a_decay } else { 0.0 };
        ledger.accelerated_steps += usize::from(accelerated);
        ledger.accelerated_a_decay += accelerated_decay;
        accumulate(
            &mut ledger,
            before,
            mesh.interior,
            &reaction,
            mesh.area().max(1e-6),
        );
        let state = snap(&mesh, step + 1);
        envelope.push(SourceStep {
            step: step + 1,
            a: before.a,
            r: before.r,
            n: before.n,
            f: before.f,
            ordinary_requested_extent: ordinary_requested,
            ordinary_accepted_extent: ordinary.n_consumed,
            source_saturated_accepted_extent: saturated.n_consumed,
            effective_gain_required: required_gain,
            applied_gain,
            a_decay,
            accelerated_a_decay: accelerated_decay,
            catalyst_a_consumption: reaction.c_produced,
            structural_a_consumption: reaction.a_consumed_build,
            membrane_a_consumption: reaction.l_produced,
            a_to_r: reaction.reserve.a_to_r,
            r_to_a: reaction.reserve.r_to_a,
            r_loss: reaction.reserve.r_to_w,
        });
        hashes.push(stable_json_hash(&state).unwrap());
    }
    let final_state = snap(&mesh, FEED_STEPS);
    (
        mesh.clone(),
        RunSummary {
            arm: arm.name().into(),
            constant_gain: if arm == Arm::ConstantGain {
                fit.map(|f| f.g_max)
            } else {
                None
            },
            initial: initial_snap,
            final_state,
            ledger,
            resource_n_remaining: region.n_mass,
            resource_f_remaining: region.f_mass,
            alive: mesh.alive,
            finite_nonnegative: finite_nonnegative(&mesh),
            trajectory_hash: stable_json_hash(&hashes).unwrap(),
            final_mesh_hash: stable_json_hash(&mesh).unwrap(),
            envelope,
        },
    )
}

fn deprive(settled: &MaterialMesh, mechanics: &MechParams) -> (MaterialMesh, Snap) {
    let mut mesh = settled.clone();
    let params = reaction_params(&mesh);
    for _ in 0..DEPRIVATION_STEPS {
        reaction_with_gain(&mut mesh, &params, mechanics.dt, 1.0);
        if !mesh.alive {
            break;
        }
    }
    let state = snap(&mesh, DEPRIVATION_STEPS);
    (mesh, state)
}

fn relative_rmse(points: &[(&SourceStep, f64)], fit: &Fit) -> f64 {
    if points.is_empty() {
        return f64::INFINITY;
    }
    let mse = points
        .iter()
        .map(|(point, expected)| {
            let predicted = fit_gain(point.a, fit);
            ((predicted - *expected) / expected.max(1.0)).powi(2)
        })
        .sum::<f64>()
        / points.len() as f64;
    mse.sqrt()
}

fn fit_a_only(envelope: &[SourceStep]) -> Option<Fit> {
    let usable: Vec<&SourceStep> = envelope
        .iter()
        .filter(|p| p.n * p.f > 1e-8 && p.effective_gain_required.is_finite() && p.a > SOURCE_EPS)
        .collect();
    if usable.len() < 12 {
        return None;
    }
    let split = usable.len() / 2;
    let train = &usable[..split];
    let holdout = &usable[split..];
    let g_max = usable
        .iter()
        .map(|p| p.effective_gain_required)
        .fold(1.0_f64, f64::max);
    let candidates: Vec<&SourceStep> = train
        .iter()
        .copied()
        .filter(|p| {
            p.effective_gain_required > 1.0 + 1e-6
                && p.effective_gain_required < g_max * (1.0 - 1e-6)
        })
        .collect();
    if candidates.len() < 4 {
        return None;
    }
    let first = candidates[0];
    let last = candidates[candidates.len() - 1];
    let y1 = (g_max - 1.0) / (first.effective_gain_required - 1.0) - 1.0;
    let y2 = (g_max - 1.0) / (last.effective_gain_required - 1.0) - 1.0;
    let denominator = (last.a / first.a).ln();
    if !(y1 > 0.0 && y2 > 0.0 && denominator.abs() > 1e-12) {
        return None;
    }
    let n = (y2 / y1).ln() / denominator;
    let k_a = first.a / y1.powf(1.0 / n);
    if !(n.is_finite() && n > 0.0 && k_a.is_finite() && k_a > 0.0) {
        return None;
    }
    let prototype = Fit {
        g_max,
        k_a,
        n,
        train_relative_rmse: 0.0,
        holdout_relative_rmse: 0.0,
        train_points: train.len(),
        holdout_points: holdout.len(),
        required_gain_decreases_with_a: last.effective_gain_required
            <= first.effective_gain_required,
        holdout_phase_bias: 0.0,
        bounded_positive: true,
    };
    let train_points: Vec<(&SourceStep, f64)> = train
        .iter()
        .map(|p| (*p, p.effective_gain_required))
        .collect();
    let holdout_points: Vec<(&SourceStep, f64)> = holdout
        .iter()
        .map(|p| (*p, p.effective_gain_required))
        .collect();
    let train_rmse = relative_rmse(&train_points, &prototype);
    let holdout_rmse = relative_rmse(&holdout_points, &prototype);
    let train_mean =
        train.iter().map(|p| p.effective_gain_required).sum::<f64>() / train.len() as f64;
    let holdout_mean = holdout
        .iter()
        .map(|p| p.effective_gain_required)
        .sum::<f64>()
        / holdout.len() as f64;
    Some(Fit {
        train_relative_rmse: train_rmse,
        holdout_relative_rmse: holdout_rmse,
        holdout_phase_bias: (holdout_mean - train_mean) / train_mean.max(1.0),
        ..prototype
    })
}

fn run_constant_gain(initial: &MaterialMesh, gain: f64, mechanics: &MechParams) -> RunSummary {
    let placeholder = Fit {
        g_max: gain,
        k_a: 1.0,
        n: 1.0,
        train_relative_rmse: 0.0,
        holdout_relative_rmse: 0.0,
        train_points: 0,
        holdout_points: 0,
        required_gain_decreases_with_a: true,
        holdout_phase_bias: 0.0,
        bounded_positive: true,
    };
    run_feed(initial, Arm::ConstantGain, mechanics, Some(&placeholder)).1
}

fn solve_break_even(
    initial: &MaterialMesh,
    target: f64,
    mechanics: &MechParams,
) -> (Option<f64>, Vec<Value>) {
    let mut trials = Vec::new();
    let mut low = 1.0;
    let low_run = run_constant_gain(initial, low, mechanics);
    trials.push(json!({"gain":low,"final_e_stored":low_run.final_state.e_stored}));
    if low_run.final_state.e_stored >= target {
        return (Some(low), trials);
    }
    let mut high = 2.0;
    let mut high_run = run_constant_gain(initial, high, mechanics);
    trials.push(json!({"gain":high,"final_e_stored":high_run.final_state.e_stored}));
    while high_run.final_state.e_stored < target && high < 256.0 {
        low = high;
        high *= 2.0;
        high_run = run_constant_gain(initial, high, mechanics);
        trials.push(json!({"gain":high,"final_e_stored":high_run.final_state.e_stored}));
    }
    if high_run.final_state.e_stored < target {
        return (None, trials);
    }
    for _ in 0..48 {
        let mid = 0.5 * (low + high);
        let mid_run = run_constant_gain(initial, mid, mechanics);
        trials.push(json!({"gain":mid,"final_e_stored":mid_run.final_state.e_stored}));
        if mid_run.final_state.e_stored >= target {
            high = mid;
        } else {
            low = mid;
        }
        if (high - low) / high.max(1.0) <= 1e-5 {
            break;
        }
    }
    (Some(high), trials)
}

fn sustained(
    initial: &MaterialMesh,
    arm: Arm,
    fit: &Fit,
    mechanics: &MechParams,
) -> SustainedSummary {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let initial_snap = snap(&mesh, 0);
    let mut ledger = Ledger::default();
    let quarter = SUSTAINED_STEPS / 4;
    let mut q_values = [0.0; 4];
    let mut q_slopes = [0.0; 4];
    let mut previous = initial_snap.e_stored;
    let mut peak = previous;
    let mut late_gain_sum = 0.0;
    let mut late_count = 0usize;
    let mut max_gain: f64 = 0.0;
    for step in 0..SUSTAINED_STEPS {
        mesh.interior.n = SUSTAINED_NF;
        mesh.interior.f = SUSTAINED_NF;
        let before = mesh.interior;
        let gain = gain_for(arm, &mesh, &params, Some(fit));
        let reaction = reaction_with_gain(&mut mesh, &params, mechanics.dt, gain);
        let accelerated = mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) < 1e-8;
        let decay = inferred_a_decay(before, mesh.interior, &reaction, mesh.area().max(1e-6));
        ledger.accelerated_steps += usize::from(accelerated);
        ledger.accelerated_a_decay += if accelerated { decay } else { 0.0 };
        accumulate(
            &mut ledger,
            before,
            mesh.interior,
            &reaction,
            mesh.area().max(1e-6),
        );
        let current = snap(&mesh, step + 1).e_stored;
        peak = peak.max(current);
        if step >= SUSTAINED_STEPS - quarter {
            late_gain_sum += gain;
            late_count += 1;
        }
        max_gain = max_gain.max(gain);
        if (step + 1) % quarter == 0 {
            let index = (step + 1) / quarter - 1;
            q_values[index] = current;
            q_slopes[index] = (current - previous) / quarter as f64;
            previous = current;
        }
    }
    SustainedSummary {
        arm: arm.name().into(),
        initial: initial_snap,
        final_state: snap(&mesh, SUSTAINED_STEPS),
        quarter_e_stored: q_values,
        quarter_slopes: q_slopes,
        peak_e_stored: peak,
        mean_gain_last_quarter: late_gain_sum / late_count.max(1) as f64,
        max_gain,
        ledger,
        alive: mesh.alive,
        finite_nonnegative: finite_nonnegative(&mesh),
    }
}

fn cycles(initial: &MaterialMesh, fit: &Fit, mechanics: &MechParams) -> Vec<CycleSummary> {
    let mut mesh = initial.clone();
    let mut summaries = Vec::new();
    for cycle in 1..=3 {
        let (deprived, deprived_snap) = deprive(&mesh, mechanics);
        let (fed, run) = run_feed(&deprived, Arm::DerivedAFeedback, mechanics, Some(fit));
        summaries.push(CycleSummary {
            cycle,
            deprived_start_e_stored: deprived_snap.e_stored,
            recovered_e_stored: run.final_state.e_stored,
            alive: run.alive,
            conservation_error: run.ledger.max_conservation_error,
            gain_first_feed: run.envelope.first().map(|p| p.applied_gain).unwrap_or(1.0),
            gain_last_feed: run.envelope.last().map(|p| p.applied_gain).unwrap_or(1.0),
        });
        mesh = fed;
    }
    summaries
}

fn main() {
    let output = std::env::var_os("DCDEV020R2_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r2"));
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() < 1e-12);
    let settled = settle(&mechanics);
    let settled_hash = stable_json_hash(&settled).unwrap();
    let (deprived, deprived_snap) = deprive(&settled, &mechanics);

    let (_baseline_mesh, baseline) = run_feed(&deprived, Arm::Baseline, &mechanics, None);
    let (_prior_mesh, prior_two_x) = run_feed(&deprived, Arm::PriorTwoX, &mechanics, None);
    let (_dcdev017_mesh, dcdev017) = run_feed(&deprived, Arm::DCDev017Observer, &mechanics, None);
    let (_source_mesh, source_saturated) =
        run_feed(&deprived, Arm::SourceSaturated, &mechanics, None);
    let source_fit = fit_a_only(&baseline.envelope);
    let (break_even_gain, root_trials) =
        solve_break_even(&deprived, deprived_snap.e_stored, &mechanics);

    let sequencing = json!({
        "ordinary": {
            "steps": baseline.envelope.len(),
            "accelerated_steps": baseline.ledger.accelerated_steps,
            "accelerated_fraction": baseline.ledger.accelerated_steps as f64 / baseline.envelope.len() as f64,
            "a_decay": baseline.ledger.a_decay,
            "accelerated_a_decay": baseline.ledger.accelerated_a_decay,
            "fraction_new_a_lost_through_accelerated_branch": baseline.ledger.accelerated_a_decay / baseline.ledger.a_produced.max(SOURCE_EPS)
        },
        "source_saturated": {
            "steps": source_saturated.envelope.len(),
            "accelerated_steps": source_saturated.ledger.accelerated_steps,
            "accelerated_fraction": source_saturated.ledger.accelerated_steps as f64 / source_saturated.envelope.len() as f64,
            "a_decay": source_saturated.ledger.a_decay,
            "accelerated_a_decay": source_saturated.ledger.accelerated_a_decay,
            "fraction_new_a_lost_through_accelerated_branch": source_saturated.ledger.accelerated_a_decay / source_saturated.ledger.a_produced.max(SOURCE_EPS)
        },
        "reaction_order_unchanged": true,
        "stop_classification": "not_triggered_by_observer_accounting"
    });

    let fit = source_fit.clone();
    let mut gate4_pass = false;
    let mut fit_value = None;
    if let Some(candidate) = fit {
        gate4_pass = candidate.required_gain_decreases_with_a
            && candidate.bounded_positive
            && candidate.holdout_relative_rmse <= 0.25
            && candidate.holdout_phase_bias.abs() <= 0.25;
        fit_value = Some(candidate);
    }

    let mut r2_candidate = None;
    let mut sustained_summary = None;
    let mut cycle_summary = None;
    let mut finite_feed_verdict = "NOT_RUN_GATE_4";
    let mut sustained_verdict = "NOT_RUN_GATE_6";
    let mut cycle_verdict = "NOT_RUN_GATE_7";
    if let Some(ref derived_fit) = fit_value {
        if gate4_pass {
            let (_candidate_mesh, candidate_run) = run_feed(
                &deprived,
                Arm::DerivedAFeedback,
                &mechanics,
                Some(derived_fit),
            );
            finite_feed_verdict = if candidate_run.alive
                && candidate_run.finite_nonnegative
                && candidate_run.ledger.max_conservation_error <= MASS_EPS
                && candidate_run.final_state.e_stored > deprived_snap.e_stored
                && candidate_run.final_state.e_stored > baseline.final_state.e_stored
            {
                "PASS"
            } else {
                "DCDEV020_A_PRODUCT_FEEDBACK_FINITE_RESTORATION_FAILURE"
            };
            r2_candidate = Some(candidate_run);
            if finite_feed_verdict == "PASS" {
                let sustained_run =
                    sustained(&deprived, Arm::DerivedAFeedback, derived_fit, &mechanics);
                sustained_verdict = if sustained_run.alive
                    && sustained_run.finite_nonnegative
                    && sustained_run.final_state.e_stored >= 0.95 * E_TARGET
                    && sustained_run.final_state.e_stored <= 1.05 * E_TARGET
                    && sustained_run.peak_e_stored <= 1.10 * E_TARGET
                    && sustained_run.ledger.max_conservation_error <= MASS_EPS
                {
                    "PASS"
                } else {
                    "DCDEV020_A_PRODUCT_FEEDBACK_NO_STABLE_FIXED_POINT"
                };
                sustained_summary = Some(sustained_run);
                if sustained_verdict == "PASS" {
                    let cycle_run = cycles(&deprived, derived_fit, &mechanics);
                    let first = cycle_run
                        .first()
                        .map(|c| c.recovered_e_stored)
                        .unwrap_or(0.0);
                    let last = cycle_run
                        .last()
                        .map(|c| c.recovered_e_stored)
                        .unwrap_or(0.0);
                    let all_pass = cycle_run.len() == 3
                        && cycle_run.iter().all(|c| {
                            c.alive
                                && c.conservation_error <= MASS_EPS
                                && c.recovered_e_stored > c.deprived_start_e_stored
                        })
                        && last >= 0.90 * first;
                    cycle_verdict = if all_pass {
                        "PASS"
                    } else {
                        "DCDEV020_A_PRODUCT_FEEDBACK_FINITE_RESTORATION_FAILURE"
                    };
                    cycle_summary = Some(cycle_run);
                }
            }
        }
    }

    let conclusion = if !gate4_pass {
        "DCDEV020_A_ONLY_ALLOSTERIC_COORDINATE_INSUFFICIENT"
    } else if finite_feed_verdict != "PASS" {
        "DCDEV020_A_PRODUCT_FEEDBACK_FINITE_RESTORATION_FAILURE"
    } else if sustained_verdict != "PASS" {
        sustained_verdict
    } else if cycle_verdict != "PASS" {
        cycle_verdict
    } else {
        "DCDEV020_A_ONLY_ALLOSTERIC_ASSIMILATION_OBSERVER_QUALIFIED"
    };

    let protocol_audit = json!({
        "classification": "DCDEV020_PROTOCOL_NONCONFORMANCE_CONFIRMED",
        "prior_head": PRIOR_HEAD,
        "prior_artifacts_immutable": true,
        "discrepancies": [
            "no derived source-actuation requirement",
            "arbitrary K_I=1",
            "arbitrary additional-capacity ceiling 1.0",
            "D-017 and DC-DEV-017 identity collision",
            "resource-free long-horizon assay",
            "incomplete three-cycle assay"
        ]
    });
    let results = json!({
        "directive": "DC-DEV-020-R2",
        "entry_commit": ENTRY,
        "prior_dcdev020_head": PRIOR_HEAD,
        "observer_only": true,
        "production_behavior_changed": false,
        "chemistry_behavior_changed": false,
        "protocol_audit": protocol_audit,
        "settled_hash": settled_hash,
        "deprived": deprived_snap,
        "source_saturated": source_saturated,
        "baseline": baseline,
        "prior_exact_two_x": prior_two_x,
        "dcdev017_correct_replay": dcdev017,
        "root_solve": {"target_e_stored": deprived_snap.e_stored, "minimum_constant_break_even_gain": break_even_gain, "trials": root_trials},
        "sequencing_audit": sequencing,
        "a_only_fit": fit_value,
        "gate_4_pass": gate4_pass,
        "r2_candidate": r2_candidate,
        "sustained_fed": sustained_summary,
        "three_cycles": cycle_summary,
        "verdicts": {"finite_feed": finite_feed_verdict, "sustained_fed": sustained_verdict, "three_cycles": cycle_verdict},
        "conclusion": conclusion,
        "implementation_authorized": false,
        "next_execution_started": false
    });
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": "DC-DEV-020-R2", "entry_commit": ENTRY, "prior_head": PRIOR_HEAD,
            "settle_steps": SETTLE_STEPS, "deprivation_steps": DEPRIVATION_STEPS, "feed_steps": FEED_STEPS,
            "sustained_steps": SUSTAINED_STEPS, "resource_mass": SELECTED_MASS,
            "resource_center": RESOURCE_CENTER, "resource_radius": RESOURCE_RADIUS, "sustained_nf": SUSTAINED_NF,
            "observer_only": true, "production_integration": false, "root_relative_tolerance": 1e-4,
            "fit_family": "1 + (G_max - 1)/(1 + (A/K_A)^n)", "historical_replay": "DC-DEV-017"
        }),
    );
    write_json(&output, "results.json", &results);
    write_json(
        &output,
        "qualification.json",
        &json!({
            "protocol_audit": "DCDEV020_PROTOCOL_NONCONFORMANCE_CONFIRMED",
            "classification": conclusion,
            "implementation_authorized": false,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "source_actuation_envelope.json",
        &json!({"steps": baseline.envelope}),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "status": "reviewed_for_observer_requalification",
            "external_constants_imported": false,
            "species_specific_identities_imported": false,
            "sources": [
                {
                    "citation": "Goyal et al. 2010, Achieving Optimal Growth through Product Feedback Inhibition in Metabolism",
                    "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC2880561/",
                    "classification": "ADAPTABLE",
                    "reusable_finding": "Product feedback can support homeostatic control, while simple feedback can produce large metabolite pools; ultrasensitivity can constrain those pools.",
                    "imported_into_assay": false
                },
                {
                    "citation": "Link, Kochanowski & Sauer 2013, Systematic identification of allosteric protein-metabolite interactions that control enzyme activity in vivo",
                    "url": "https://www.nature.com/articles/nbt.2489",
                    "classification": "ADAPTABLE",
                    "reusable_finding": "Rapid nutrient switches and allosteric interactions can control flux on short timescales; organism-specific interaction identities are not imported.",
                    "imported_into_assay": false
                },
                {
                    "citation": "Buffing et al. 2018, Capacity for instantaneous catabolism of preferred and non-preferred carbon sources in E. coli and B. subtilis",
                    "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC6079084/",
                    "classification": "ADAPTABLE",
                    "reusable_finding": "E. coli can reverse carbon flux rapidly through allosteric regulation, whereas B. subtilis requires additional interactions and transcriptional deregulation; species-specific details are not imported.",
                    "imported_into_assay": false
                },
                {
                    "citation": "Goyal & Wingreen 2007, Growth-Induced Instability in Metabolic Networks",
                    "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC1995071/",
                    "classification": "REFERENCE_ONLY",
                    "reusable_finding": "Coupled product-feedback networks can become oscillatory through a Hopf-like instability, supporting an explicit stability gate without importing model constants.",
                    "imported_into_assay": false
                }
            ]
        }),
    );
    println!("DCDEV020R2_OBSERVER_REQUALIFICATION_COMPLETE");
    println!("protocol_audit=DCDEV020_PROTOCOL_NONCONFORMANCE_CONFIRMED");
    println!("gate_4_a_only_pass={gate4_pass}");
    println!("minimum_constant_break_even_gain={break_even_gain:?}");
    println!("conclusion={conclusion}");
    println!("NEXT_EXECUTION_STARTED:false");
}
