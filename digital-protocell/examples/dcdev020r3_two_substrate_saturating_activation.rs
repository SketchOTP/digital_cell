//! DC-DEV-020-R3 observer-only two-substrate kinetic audit.
//!
//! Production chemistry is not changed. The existing reaction path is replayed
//! with observer-local source multipliers so the substrate factor can be
//! diagnosed and, only if identifiable, replaced counterfactually.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{q_catalyst, reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const CLEAN_BASE: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const ACCEPTED_R2_HEAD: &str = "e394aa675a4f44d91d1a8729736679fb4b7e7ab8";
const SETTLE_STEPS: usize = 5_000;
const DEPRIVATION_STEPS: usize = 480;
const FEED_STEPS: usize = 480;
const SUSTAINED_STEPS: usize = 8_000;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const SELECTED_MASS: f64 = 19.878372106390554;
const BREAK_EVEN_GAIN: f64 = 13.9482421875;
const DEPRIVED_REFERENCE: f64 = 60.82781514212436;
const DT: f64 = 0.02;
const MASS_EPS: f64 = 1e-10;
const SOURCE_EPS: f64 = 1e-12;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
enum Arm {
    Baseline,
    ConstantBreakEven,
    SourceSaturated,
    SaturatingCandidate,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_bilinear_source",
            Self::ConstantBreakEven => "constant_break_even_gain",
            Self::SourceSaturated => "source_saturated_upper_bound",
            Self::SaturatingCandidate => "two_substrate_saturating_candidate",
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
}

#[derive(Clone, Debug, Serialize)]
struct KineticPoint {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    n_times_f: f64,
    q_c: f64,
    g_h: f64,
    ordinary_requested_extent: f64,
    ordinary_accepted_extent: f64,
    constant_break_even_accepted_extent: f64,
    source_saturated_accepted_extent: f64,
    effective_gain_required: f64,
    applied_gain: f64,
    accepted_extent: f64,
    a_produced: f64,
    a_decay: f64,
    accelerated_a_decay: f64,
    accelerated_decay_active: bool,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
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
    #[serde(skip_serializing)]
    points: Vec<KineticPoint>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct AsymptoticWitness {
    k_s: f64,
    v_max: f64,
    train_relative_rmse: f64,
    holdout_relative_rmse: f64,
    max_capacity_fraction: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Identification {
    method: String,
    train_points: usize,
    holdout_points: usize,
    linearized_slope: f64,
    linearized_intercept: f64,
    v_max: Option<f64>,
    k_s: Option<f64>,
    baseline_train_relative_rmse: f64,
    baseline_holdout_relative_rmse: f64,
    candidate_train_relative_rmse: Option<f64>,
    candidate_holdout_relative_rmse: Option<f64>,
    max_source_capacity_fraction: Option<f64>,
    witnesses: Vec<AsymptoticWitness>,
    identifiable: bool,
    reason: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct LogDiagnosis {
    points: usize,
    correlation_log_gain_log_nf: f64,
    log_linear_slope: f64,
    log_linear_r_squared: f64,
    fraction_variation_explained_by_nf: f64,
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

fn phenotype_gain(mesh: &MaterialMesh, params: &ReactionParams) -> f64 {
    assert!(!params.composition.enable);
    assert!(!params.autocatalytic.enable);
    assert!(!params.network.enable);
    assert!(!params.template.enable);
    assert!(mesh.finite_allocation.is_none());
    1.0
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
    let qc = q_catalyst(mesh.interior.c, params.q_c);
    let gh = phenotype_gain(mesh, params);
    params.k_act
        * qc
        * gh
        * mesh.interior.n.max(0.0)
        * mesh.interior.f.max(0.0)
        * dt
        * mesh.area().max(1e-6)
}

fn source_previews(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    dt: f64,
) -> (ReactionLedger, ReactionLedger, f64, ReactionLedger) {
    let mut ordinary_mesh = mesh.clone();
    let ordinary = reaction_with_gain(&mut ordinary_mesh, params, dt, 1.0);
    let mut constant_mesh = mesh.clone();
    let constant = reaction_with_gain(&mut constant_mesh, params, dt, BREAK_EVEN_GAIN);
    let area = mesh.area().max(1e-15);
    let capacity = (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area);
    let gain = if ordinary.n_consumed > SOURCE_EPS {
        (capacity / ordinary.n_consumed).max(1.0)
    } else {
        1.0
    };
    let mut saturated_mesh = mesh.clone();
    let saturated = reaction_with_gain(&mut saturated_mesh, params, dt, gain);
    (ordinary, constant, gain, saturated)
}

fn saturating_fraction(n: f64, f: f64, k_s: f64) -> f64 {
    let n = n.max(0.0);
    let f = f.max(0.0);
    let denominator = k_s * k_s + k_s * n + k_s * f + n * f;
    if n == 0.0 || f == 0.0 || denominator <= 0.0 {
        0.0
    } else {
        n * f / denominator
    }
}

fn candidate_extent(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    dt: f64,
    v_max: f64,
    k_s: f64,
) -> f64 {
    q_catalyst(mesh.interior.c, params.q_c)
        * phenotype_gain(mesh, params)
        * v_max
        * saturating_fraction(mesh.interior.n, mesh.interior.f, k_s)
        * dt
        * mesh.area().max(1e-6)
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

fn run_feed(
    initial: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    candidate: Option<(f64, f64)>,
    resource_scale: f64,
) -> (MaterialMesh, RunSummary) {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let initial_snap = snap(&mesh, 0);
    let mass = SELECTED_MASS * resource_scale;
    let mut region =
        FiniteSpatialResourceRegionV1::new(RESOURCE_CENTER, RESOURCE_RADIUS, mass, mass);
    let transport = TransportParams::default();
    let mut ledger = Ledger::default();
    let mut points = Vec::with_capacity(FEED_STEPS);
    let mut hashes = vec![stable_json_hash(&initial_snap).unwrap()];
    for step in 0..FEED_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        ledger.n_delivered += uptake.n_delivered;
        ledger.f_delivered += uptake.f_delivered;
        ledger.max_conservation_error =
            ledger.max_conservation_error.max(uptake.conservation_error);
        assert!(uptake.conservation_error <= MASS_EPS);

        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let qc = q_catalyst(before.c, params.q_c);
        let gh = phenotype_gain(&mesh, &params);
        let ordinary_requested = ordinary_requested_extent(&mesh, &params, mechanics.dt);
        let (ordinary, constant, required_gain, saturated) =
            source_previews(&mesh, &params, mechanics.dt);
        let applied_gain = match arm {
            Arm::Baseline => 1.0,
            Arm::ConstantBreakEven => BREAK_EVEN_GAIN,
            Arm::SourceSaturated => required_gain,
            Arm::SaturatingCandidate => {
                let (v_max, k_s) = candidate.expect("candidate parameters required");
                let predicted = candidate_extent(&mesh, &params, mechanics.dt, v_max, k_s);
                if ordinary_requested > SOURCE_EPS {
                    predicted / ordinary_requested
                } else {
                    0.0
                }
            }
        };
        let reaction = reaction_with_gain(&mut mesh, &params, mechanics.dt, applied_gain);
        let accelerated = mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) < 1e-8;
        let a_decay = inferred_a_decay(before, mesh.interior, &reaction, area);
        let accelerated_decay = if accelerated { a_decay } else { 0.0 };
        ledger.accelerated_steps += usize::from(accelerated);
        ledger.accelerated_a_decay += accelerated_decay;
        accumulate(&mut ledger, before, mesh.interior, &reaction, area);
        points.push(KineticPoint {
            step: step + 1,
            area,
            n: before.n,
            f: before.f,
            n_times_f: before.n.max(0.0) * before.f.max(0.0),
            q_c: qc,
            g_h: gh,
            ordinary_requested_extent: ordinary_requested,
            ordinary_accepted_extent: ordinary.n_consumed,
            constant_break_even_accepted_extent: constant.n_consumed,
            source_saturated_accepted_extent: saturated.n_consumed,
            effective_gain_required: required_gain,
            applied_gain,
            accepted_extent: reaction.n_consumed,
            a_produced: reaction.a_produced,
            a_decay,
            accelerated_a_decay: accelerated_decay,
            accelerated_decay_active: accelerated,
        });
        hashes.push(stable_json_hash(&snap(&mesh, step + 1)).unwrap());
    }
    let final_state = snap(&mesh, FEED_STEPS);
    (
        mesh.clone(),
        RunSummary {
            arm: arm.name().into(),
            initial: initial_snap,
            final_state,
            ledger,
            resource_n_remaining: region.n_mass,
            resource_f_remaining: region.f_mass,
            alive: mesh.alive,
            finite_nonnegative: finite_nonnegative(&mesh),
            trajectory_hash: stable_json_hash(&hashes).unwrap(),
            final_mesh_hash: stable_json_hash(&mesh).unwrap(),
            points,
        },
    )
}

fn correlation_diagnosis(points: &[KineticPoint]) -> LogDiagnosis {
    let pairs: Vec<(f64, f64)> = points
        .iter()
        .filter(|p| p.n_times_f > SOURCE_EPS && p.effective_gain_required > 0.0)
        .map(|p| (p.n_times_f.ln(), p.effective_gain_required.ln()))
        .collect();
    assert!(pairs.len() >= 12);
    let count = pairs.len() as f64;
    let mean_x = pairs.iter().map(|p| p.0).sum::<f64>() / count;
    let mean_y = pairs.iter().map(|p| p.1).sum::<f64>() / count;
    let covariance = pairs
        .iter()
        .map(|p| (p.0 - mean_x) * (p.1 - mean_y))
        .sum::<f64>();
    let variance_x = pairs.iter().map(|p| (p.0 - mean_x).powi(2)).sum::<f64>();
    let variance_y = pairs.iter().map(|p| (p.1 - mean_y).powi(2)).sum::<f64>();
    let slope = covariance / variance_x;
    let intercept = mean_y - slope * mean_x;
    let residual = pairs
        .iter()
        .map(|p| (p.1 - (intercept + slope * p.0)).powi(2))
        .sum::<f64>();
    let r_squared = 1.0 - residual / variance_y;
    LogDiagnosis {
        points: pairs.len(),
        correlation_log_gain_log_nf: covariance / (variance_x * variance_y).sqrt(),
        log_linear_slope: slope,
        log_linear_r_squared: r_squared,
        fraction_variation_explained_by_nf: r_squared.clamp(0.0, 1.0),
    }
}

fn relative_rmse(points: &[KineticPoint], v_max: Option<f64>, k_s: Option<f64>) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for point in points {
        if point.constant_break_even_accepted_extent <= SOURCE_EPS {
            continue;
        }
        let predicted = if let (Some(v_max), Some(k_s)) = (v_max, k_s) {
            point.q_c
                * point.g_h
                * v_max
                * saturating_fraction(point.n, point.f, k_s)
                * DT
                * point.area
        } else {
            point.ordinary_requested_extent
        };
        let error = (predicted - point.constant_break_even_accepted_extent)
            / point.constant_break_even_accepted_extent;
        sum += error * error;
        count += 1;
    }
    (sum / count.max(1) as f64).sqrt()
}

fn max_capacity_fraction(points: &[KineticPoint], v_max: f64, k_s: f64) -> f64 {
    points
        .iter()
        .filter_map(|point| {
            if point.source_saturated_accepted_extent <= SOURCE_EPS {
                None
            } else {
                let predicted = point.q_c
                    * point.g_h
                    * v_max
                    * saturating_fraction(point.n, point.f, k_s)
                    * DT
                    * point.area;
                Some(predicted / point.source_saturated_accepted_extent)
            }
        })
        .fold(0.0_f64, f64::max)
}

fn identify_saturating_law(reference: &[KineticPoint]) -> Identification {
    let usable: Vec<KineticPoint> = reference
        .iter()
        .filter(|p| {
            p.n > SOURCE_EPS
                && p.f > SOURCE_EPS
                && (p.n - p.f).abs() <= 1e-10
                && p.constant_break_even_accepted_extent > SOURCE_EPS
        })
        .cloned()
        .collect();
    assert!(usable.len() >= 30);
    let split = usable.len() * 2 / 3;
    let train = &usable[..split];
    let holdout = &usable[split..];

    // For N=F=s, sqrt(J/(q_c*g_h)) = sqrt(V_max)*s/(K_S+s).
    // Therefore s/sqrt(J/(q_c*g_h)) is affine in s. A zero slope means
    // V_max and K_S cannot be separately identified; only V_max/K_S^2 is.
    let transformed: Vec<(f64, f64)> = train
        .iter()
        .map(|point| {
            let s = (point.n * point.f).sqrt();
            let target_rate = point.constant_break_even_accepted_extent
                / (point.q_c * point.g_h * DT * point.area);
            (s, s / target_rate.sqrt())
        })
        .collect();
    let count = transformed.len() as f64;
    let mean_x = transformed.iter().map(|p| p.0).sum::<f64>() / count;
    let mean_y = transformed.iter().map(|p| p.1).sum::<f64>() / count;
    let covariance = transformed
        .iter()
        .map(|p| (p.0 - mean_x) * (p.1 - mean_y))
        .sum::<f64>();
    let variance_x = transformed
        .iter()
        .map(|p| (p.0 - mean_x).powi(2))
        .sum::<f64>();
    let slope = covariance / variance_x;
    let intercept = mean_y - slope * mean_x;

    let coefficient = train
        .iter()
        .map(|point| {
            let target_rate = point.constant_break_even_accepted_extent
                / (point.q_c * point.g_h * DT * point.area);
            target_rate / point.n_times_f
        })
        .sum::<f64>()
        / train.len() as f64;
    let max_s = usable
        .iter()
        .map(|point| (point.n * point.f).sqrt())
        .fold(0.0_f64, f64::max);
    let witnesses: Vec<AsymptoticWitness> = [10.0, 100.0, 1000.0]
        .iter()
        .map(|scale| {
            let k_s = max_s * scale;
            let v_max = coefficient * k_s * k_s;
            AsymptoticWitness {
                k_s,
                v_max,
                train_relative_rmse: relative_rmse(train, Some(v_max), Some(k_s)),
                holdout_relative_rmse: relative_rmse(holdout, Some(v_max), Some(k_s)),
                max_capacity_fraction: max_capacity_fraction(&usable, v_max, k_s),
            }
        })
        .collect();

    let finite_fit = if slope.is_finite() && slope > 1e-12 && intercept > 0.0 {
        let v_max = 1.0 / (slope * slope);
        let k_s = intercept / slope;
        if v_max.is_finite() && k_s.is_finite() && v_max > 0.0 && k_s > 0.0 {
            Some((v_max, k_s))
        } else {
            None
        }
    } else {
        None
    };
    let candidate_train = finite_fit.map(|p| relative_rmse(train, Some(p.0), Some(p.1)));
    let candidate_holdout = finite_fit.map(|p| relative_rmse(holdout, Some(p.0), Some(p.1)));
    let capacity = finite_fit.map(|p| max_capacity_fraction(&usable, p.0, p.1));
    let baseline_train = relative_rmse(train, None, None);
    let baseline_holdout = relative_rmse(holdout, None, None);
    let asymptotic_nonidentifiability = witnesses.windows(2).all(|pair| {
        pair[1].holdout_relative_rmse < pair[0].holdout_relative_rmse
            && pair[1].k_s > pair[0].k_s
            && pair[1].v_max > pair[0].v_max
    });
    let identifiable = finite_fit.is_some()
        && !asymptotic_nonidentifiability
        && candidate_holdout.unwrap_or(f64::INFINITY) < 0.5 * baseline_holdout
        && capacity.unwrap_or(f64::INFINITY) <= 1.0 + 1e-10;
    let reason = if asymptotic_nonidentifiability {
        "the profile error keeps falling as K_S and V_max grow together; only V_max/K_S^2 is constrained in the observed dilute regime"
    } else if finite_fit.is_none() {
        "the linearized reference does not yield finite positive V_max and K_S"
    } else if candidate_holdout.unwrap_or(f64::INFINITY) >= 0.5 * baseline_holdout {
        "held-out flux error is not materially below baseline bilinear kinetics"
    } else if capacity.unwrap_or(f64::INFINITY) > 1.0 + 1e-10 {
        "the fitted source exceeds the observed source-saturated capacity"
    } else {
        "identified"
    };
    Identification {
        method: "closed-form symmetric linearization plus three-point asymptotic profile; no parameter sweep"
            .into(),
        train_points: train.len(),
        holdout_points: holdout.len(),
        linearized_slope: slope,
        linearized_intercept: intercept,
        v_max: finite_fit.map(|p| p.0),
        k_s: finite_fit.map(|p| p.1),
        baseline_train_relative_rmse: baseline_train,
        baseline_holdout_relative_rmse: baseline_holdout,
        candidate_train_relative_rmse: candidate_train,
        candidate_holdout_relative_rmse: candidate_holdout,
        max_source_capacity_fraction: capacity,
        witnesses,
        identifiable,
        reason: reason.into(),
    }
}

fn validate_family_properties() {
    let k_s = 0.3;
    assert_eq!(saturating_fraction(0.0, 1.0, k_s), 0.0);
    assert_eq!(saturating_fraction(1.0, 0.0, k_s), 0.0);
    let a = saturating_fraction(0.2, 0.7, k_s);
    let b = saturating_fraction(0.7, 0.2, k_s);
    assert!((a - b).abs() <= 1e-15);
    assert!(saturating_fraction(0.4, 0.7, k_s) > a);
    assert!(saturating_fraction(0.2, 0.9, k_s) > a);
    assert!((0.0..=1.0).contains(&a));
}

fn main() {
    let output = std::env::var_os("DCDEV020R3_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r3"));
    let source_commit =
        std::env::var("DCDEV020R3_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= 1e-12);
    validate_family_properties();

    let settled = settle(&mechanics);
    let settled_hash = stable_json_hash(&settled).unwrap();
    let (deprived, deprived_snap) = deprive(&settled, &mechanics);
    assert!((deprived_snap.e_stored - DEPRIVED_REFERENCE).abs() <= 1e-10);
    let (_baseline_mesh, baseline) = run_feed(&deprived, Arm::Baseline, &mechanics, None, 1.0);
    let (_constant_mesh, constant_reference) =
        run_feed(&deprived, Arm::ConstantBreakEven, &mechanics, None, 1.0);
    let (_saturated_mesh, source_saturated) =
        run_feed(&deprived, Arm::SourceSaturated, &mechanics, None, 1.0);

    let diagnosis = correlation_diagnosis(&baseline.points);
    let gate2_supported = diagnosis.correlation_log_gain_log_nf <= -0.95
        && diagnosis.fraction_variation_explained_by_nf >= 0.95
        && constant_reference.final_state.e_stored >= deprived_snap.e_stored
        && constant_reference.ledger.a_produced > baseline.ledger.a_produced
        && source_saturated.ledger.a_produced > constant_reference.ledger.a_produced;
    let identification = if gate2_supported {
        Some(identify_saturating_law(&constant_reference.points))
    } else {
        None
    };
    let gate4_pass = identification
        .as_ref()
        .map(|fit| fit.identifiable)
        .unwrap_or(false);

    let mut candidate = None;
    let mut dose_results: Option<Value> = None;
    let mut sustained_result: Option<Value> = None;
    let mut cycle_result: Option<Value> = None;
    if gate4_pass {
        let fit = identification.as_ref().unwrap();
        let parameters = (fit.v_max.unwrap(), fit.k_s.unwrap());
        candidate = Some(
            run_feed(
                &deprived,
                Arm::SaturatingCandidate,
                &mechanics,
                Some(parameters),
                1.0,
            )
            .1,
        );
        // Later gates are intentionally not implemented unless identification passes.
        // This branch is unreachable for the frozen R3 reference trajectory.
        dose_results = Some(json!({"status":"NOT_RUN_IMPLEMENTATION_BOUNDARY"}));
        sustained_result =
            Some(json!({"status":"NOT_RUN_IMPLEMENTATION_BOUNDARY","steps":SUSTAINED_STEPS}));
        cycle_result = Some(json!({"status":"NOT_RUN_IMPLEMENTATION_BOUNDARY"}));
    }

    let conclusion = if !gate2_supported {
        "DCDEV020R3_BILINEAR_SUBSTRATE_KINETICS_NOT_PRIMARY"
    } else if !gate4_pass {
        "DCDEV020R3_SATURATING_KINETICS_NOT_IDENTIFIABLE"
    } else {
        "DCDEV020R3_FINITE_FEED_RESTORATION_FAILURE"
    };
    let baseline_points = baseline.points.clone();
    let constant_reference_points = constant_reference.points.clone();
    let results = json!({
        "directive": "DC-DEV-020-R3",
        "clean_scientific_base": CLEAN_BASE,
        "accepted_r2_head": ACCEPTED_R2_HEAD,
        "source_commit": source_commit,
        "observer_only": true,
        "production_chemistry_changed": false,
        "production_behavior_changed": false,
        "settled_hash": settled_hash,
        "deprived": deprived_snap,
        "baseline": baseline,
        "constant_break_even_reference": constant_reference,
        "source_saturated": source_saturated,
        "kinetic_diagnosis": diagnosis,
        "gate_2_supported": gate2_supported,
        "gate_2_classification": if gate2_supported {"BILINEAR_LOW_SUBSTRATE_SUPPRESSION_MATERIAL"} else {"DCDEV020R3_BILINEAR_SUBSTRATE_KINETICS_NOT_PRIMARY"},
        "identification": identification,
        "gate_4_pass": gate4_pass,
        "candidate": candidate,
        "dose_robustness": dose_results,
        "sustained_fed": sustained_result,
        "three_cycles": cycle_result,
        "later_gates": if gate4_pass {"NOT_RUN_IMPLEMENTATION_BOUNDARY"} else {"NOT_RUN_GATE_4_FAIL_CLOSED"},
        "conclusion": conclusion,
        "implementation_authorized": false,
        "next_execution_started": false
    });
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-020-R3",
            "clean_scientific_base":CLEAN_BASE,
            "accepted_r2_head":ACCEPTED_R2_HEAD,
            "source_commit":source_commit,
            "settle_steps":SETTLE_STEPS,
            "deprivation_steps":DEPRIVATION_STEPS,
            "feed_steps":FEED_STEPS,
            "sustained_steps":SUSTAINED_STEPS,
            "resource_mass_n":SELECTED_MASS,
            "resource_mass_f":SELECTED_MASS,
            "resource_center":RESOURCE_CENTER,
            "resource_radius":RESOURCE_RADIUS,
            "constant_break_even_gain":BREAK_EVEN_GAIN,
            "candidate_family":"q_c * g_h * V_max * N*F/(K_S^2 + K_S*N + K_S*F + N*F)",
            "identified_parameters":["V_max","K_S"],
            "observer_only":true,
            "production_integration":false
        }),
    );
    write_json(&output, "results.json", &results);
    write_json(
        &output,
        "kinetic_diagnosis.json",
        &json!({
            "diagnosis": diagnosis,
            "baseline_points": baseline_points,
            "constant_reference_points": constant_reference_points
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification":conclusion,
            "gate_2_supported":gate2_supported,
            "gate_4_pass":gate4_pass,
            "implementation_authorized":false,
            "next_execution_started":false
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "status":"primary_literature_reviewed_before_implementation",
            "disposition":"ADAPTABLE_ARCHITECTURE_ONLY",
            "external_constants_imported":false,
            "molecular_identities_imported":false,
            "sources":[
                {
                    "citation":"Cleland 1963, The kinetics of enzyme-catalyzed reactions with two or more substrates or products. I. Nomenclature and rate equations",
                    "url":"https://pubmed.ncbi.nlm.nih.gov/14021667/",
                    "classification":"ADAPTABLE",
                    "reusable_finding":"Two-or-more-substrate reactions admit explicit steady-state rate equations rather than requiring unrestricted bilinear mass action.",
                    "imported_constants":false
                },
                {
                    "citation":"Pettersson 1969, Relationships between rapid equilibrium conditions and linearization of the reciprocal rate equation for the sequential random two-substrate enzyme mechanism",
                    "url":"https://pubmed.ncbi.nlm.nih.gov/5381399/",
                    "classification":"REFERENCE_ONLY",
                    "reusable_finding":"Sequential-random two-substrate mechanisms have identifiable reciprocal-rate structure under stated equilibrium assumptions.",
                    "imported_constants":false
                },
                {
                    "citation":"Wang and Mittermaier 2021, Characterizing Bi-substrate Enzyme Kinetics at High Resolution by 2D-ITC",
                    "url":"https://pubmed.ncbi.nlm.nih.gov/34514786/",
                    "classification":"ADAPTABLE",
                    "reusable_finding":"Bi-substrate rate surfaces must be characterized across both substrate concentrations and can distinguish random sequential kinetic structure.",
                    "imported_constants":false
                },
                {
                    "citation":"Link, Kochanowski and Sauer 2013, Systematic identification of allosteric protein-metabolite interactions that control enzyme activity in vivo",
                    "url":"https://www.nature.com/articles/nbt.2489",
                    "classification":"REFERENCE_ONLY",
                    "reusable_finding":"Rapid nutrient switches can produce rapid metabolite and flux changes, supporting explicit transition and stability observations.",
                    "imported_constants":false
                }
            ]
        }),
    );
    println!("DCDEV020R3_TWO_SUBSTRATE_KINETIC_AUDIT_COMPLETE");
    println!("gate_2_supported={gate2_supported}");
    println!("gate_4_pass={gate4_pass}");
    println!("conclusion={conclusion}");
    println!("NEXT_EXECUTION_STARTED:false");
}
