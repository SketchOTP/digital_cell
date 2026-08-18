//! DC-DEV-020-R5 observer-only local zero-drift source audit.
//!
//! Production chemistry is unchanged. Frozen R4 baseline and constant-gain
//! trajectories are replayed exactly. Each post-uptake, pre-reaction state is
//! cloned to determine the physically bounded source extent required for one
//! accepted chemistry step to have nonnegative stored-material drift.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{q_catalyst, reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

const CLEAN_BASE: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const ACCEPTED_R4_HEAD: &str = "669a511aacb227240bd7a4698efecfb564f481d4";
const SETTLE_STEPS: usize = 5_000;
const DEPRIVATION_STEPS: usize = 480;
const FEED_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const M: f64 = 19.878372106390554;
const DEPRIVED_REFERENCE: f64 = 60.82781514212436;
const DT: f64 = 0.02;
const MASS_EPS: f64 = 1e-10;
const SOURCE_EPS: f64 = 1e-12;
const ROOT_REL_TOL: f64 = 1e-6;
const MATERIAL_STATE_FRACTION: f64 = 0.01;
const SURROGATE_MATERIAL_OVER: f64 = 0.10;
const K_NEIGHBORS: usize = 16;
const COORD_RMSE_LIMIT: f64 = 0.15;
const COORD_P95_LIMIT: f64 = 0.30;
const COORD_AMBIGUITY_LIMIT: f64 = 0.25;

#[derive(Clone, Copy, Debug, Serialize)]
struct Probe {
    id: &'static str,
    n_scale: f64,
    f_scale: f64,
    constant_gain: f64,
    baseline_hash: &'static str,
    constant_hash: &'static str,
}

const PROBES: [Probe; 5] = [
    Probe {
        id: "P0",
        n_scale: 1.0,
        f_scale: 1.0,
        constant_gain: 13.9482421875,
        baseline_hash: "ab6267772acc25ce",
        constant_hash: "309eeeed0d68d4da",
    },
    Probe {
        id: "P1",
        n_scale: 2.0,
        f_scale: 1.0,
        constant_gain: 4.765045166015625,
        baseline_hash: "12a913168f9a11d0",
        constant_hash: "b7b207568a0dd0f4",
    },
    Probe {
        id: "P2",
        n_scale: 1.0,
        f_scale: 2.0,
        constant_gain: 4.765045166015625,
        baseline_hash: "976cc99ad3d5cd7a",
        constant_hash: "12df7caa589d5338",
    },
    Probe {
        id: "P3",
        n_scale: 4.0,
        f_scale: 1.0,
        constant_gain: 2.0837860107421875,
        baseline_hash: "566e1e59bbb754d9",
        constant_hash: "6ac77b0b68033825",
    },
    Probe {
        id: "P4",
        n_scale: 1.0,
        f_scale: 4.0,
        constant_gain: 2.0837860107421875,
        baseline_hash: "7a44a8711de7f285",
        constant_hash: "ea54d37a6b3f2809",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
enum TrajectoryKind {
    Baseline,
    Constant,
}

impl TrajectoryKind {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_bilinear_source",
            Self::Constant => "constant_endpoint_break_even_gain",
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
    c: f64,
    e_stored: f64,
    alive: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct R4Snap {
    step: usize,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    e_stored: f64,
    alive: bool,
}

#[derive(Clone, Debug)]
struct CapturedState {
    probe: Probe,
    trajectory: TrajectoryKind,
    step: usize,
    mesh: MaterialMesh,
    constant_source: f64,
}

#[derive(Clone, Debug, Serialize)]
struct TrajectorySummary {
    probe: String,
    trajectory: String,
    gain: f64,
    initial: Snap,
    final_state: Snap,
    trajectory_hash: String,
    expected_r4_hash: String,
    parity: bool,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
    max_conservation_error: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SinkAccounting {
    a_produced: f64,
    a_decay: f64,
    catalyst_a_consumption: f64,
    structural_a_consumption: f64,
    membrane_a_consumption: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_loss: f64,
    conservation_residual: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SourceEval {
    requested_extent: f64,
    accepted_extent: f64,
    drift: f64,
    accelerated_decay: bool,
    after_n: f64,
    after_f: f64,
    accounting: SinkAccounting,
}

#[derive(Clone, Debug, Serialize)]
struct RootRecord {
    probe: String,
    trajectory: String,
    step: usize,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    c: f64,
    q_c: f64,
    free_l: f64,
    structural_mass: f64,
    bound_membrane: f64,
    perimeter: f64,
    max_strain: f64,
    pre_reaction_mesh_hash: String,
    e_stored_before: f64,
    constant_source: f64,
    saturated_source: f64,
    shape: Vec<SourceEval>,
    status: String,
    s_zero: Option<f64>,
    s_zero_over_saturated: Option<f64>,
    equivalent_legacy_gain: Option<f64>,
    root_relative_interval: Option<f64>,
    root: Option<SourceEval>,
    accelerated_boundary_crossed: bool,
}

#[derive(Clone, Debug, Serialize)]
struct Distribution {
    count: usize,
    median: Option<f64>,
    p05: Option<f64>,
    p95: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct SurrogateAudit {
    probe: String,
    ratios: Distribution,
    relative_rmse: Option<f64>,
    fraction_below_local_balance: Option<f64>,
    fraction_materially_above: Option<f64>,
    early_median_residual: Option<f64>,
    middle_median_residual: Option<f64>,
    late_median_residual: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct PredictorResult {
    coordinate: String,
    train_points: usize,
    holdout_points: usize,
    k: usize,
    relative_rmse: f64,
    p95_absolute_relative_error: f64,
    same_coordinate_ambiguity: f64,
    sufficient: bool,
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
        c: mesh.interior.c,
        e_stored: area * (mesh.interior.a + mesh.interior.r).max(0.0),
        alive: mesh.alive,
    }
}

fn r4_snap(mesh: &MaterialMesh, step: usize) -> R4Snap {
    let area = mesh.area().max(1e-6);
    R4Snap {
        step,
        area,
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        e_stored: area * (mesh.interior.a + mesh.interior.r).max(0.0),
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
    let mut p = ReactionParams::default();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    p
}

fn reaction_with_gain(mesh: &mut MaterialMesh, base: &ReactionParams, gain: f64) -> ReactionLedger {
    let mut p = *base;
    p.k_act = base.k_act * gain.max(0.0);
    reactions_step(mesh, &p, DT, true, true)
}

fn settle() -> MaterialMesh {
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= 1e-12);
    let mut mesh = seed();
    for _ in 0..SETTLE_STEPS {
        assert!(mechanics_step(&mut mesh, &mechanics));
    }
    mesh
}

fn deprive(settled: &MaterialMesh) -> MaterialMesh {
    let mut mesh = settled.clone();
    let p = reaction_params(&mesh);
    for _ in 0..DEPRIVATION_STEPS {
        reaction_with_gain(&mut mesh, &p, 1.0);
    }
    assert!((snap(&mesh, DEPRIVATION_STEPS).e_stored - DEPRIVED_REFERENCE).abs() <= 1e-10);
    mesh
}

fn ordinary_requested(mesh: &MaterialMesh, p: &ReactionParams) -> f64 {
    p.k_act
        * q_catalyst(mesh.interior.c, p.q_c)
        * mesh.interior.n.max(0.0)
        * mesh.interior.f.max(0.0)
        * DT
        * mesh.area().max(1e-6)
}

fn inferred_a_decay(before: LumpedChem, after: LumpedChem, led: &ReactionLedger, area: f64) -> f64 {
    (before.a * area + led.a_produced
        - led.c_produced
        - after.a * area
        - led.a_consumed_build
        - led.l_produced
        - led.reserve.a_to_r
        + led.reserve.r_to_a)
        .max(0.0)
}

fn source_preview(mesh: &MaterialMesh, p: &ReactionParams, gain: f64) -> f64 {
    let mut clone = mesh.clone();
    reaction_with_gain(&mut clone, p, gain).n_consumed
}

fn replay_trajectory(
    initial: &MaterialMesh,
    probe: Probe,
    trajectory: TrajectoryKind,
) -> (TrajectorySummary, Vec<CapturedState>) {
    let mut mesh = initial.clone();
    let p = reaction_params(&mesh);
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M * probe.n_scale,
        M * probe.f_scale,
    );
    let initial_snap = snap(&mesh, 0);
    let mut hashes = vec![stable_json_hash(&r4_snap(&mesh, 0)).unwrap()];
    let mut captures = Vec::with_capacity(FEED_STEPS);
    let mut max_conservation = 0.0_f64;
    for step in 0..FEED_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, DT);
        max_conservation = max_conservation.max(uptake.conservation_error);
        assert!(uptake.conservation_error <= MASS_EPS);
        let constant_source = source_preview(&mesh, &p, probe.constant_gain);
        captures.push(CapturedState {
            probe,
            trajectory,
            step: step + 1,
            mesh: mesh.clone(),
            constant_source,
        });
        let gain = if trajectory == TrajectoryKind::Constant {
            probe.constant_gain
        } else {
            1.0
        };
        reaction_with_gain(&mut mesh, &p, gain);
        hashes.push(stable_json_hash(&r4_snap(&mesh, step + 1)).unwrap());
    }
    let actual = stable_json_hash(&hashes).unwrap();
    let expected = if trajectory == TrajectoryKind::Constant {
        probe.constant_hash
    } else {
        probe.baseline_hash
    };
    (
        TrajectorySummary {
            probe: probe.id.into(),
            trajectory: trajectory.name().into(),
            gain: if trajectory == TrajectoryKind::Constant {
                probe.constant_gain
            } else {
                1.0
            },
            initial: initial_snap,
            final_state: snap(&mesh, FEED_STEPS),
            trajectory_hash: actual.clone(),
            expected_r4_hash: expected.into(),
            parity: actual == expected,
            resource_n_remaining: region.n_mass,
            resource_f_remaining: region.f_mass,
            max_conservation_error: max_conservation,
        },
        captures,
    )
}

fn evaluate_source(mesh: &MaterialMesh, p: &ReactionParams, extent: f64) -> SourceEval {
    let before = mesh.interior;
    let area = mesh.area().max(1e-6);
    let before_e = area * (before.a + before.r).max(0.0);
    let unit = ordinary_requested(mesh, p);
    let gain = if extent <= SOURCE_EPS {
        0.0
    } else {
        extent / unit.max(SOURCE_EPS)
    };
    let mut clone = mesh.clone();
    let led = reaction_with_gain(&mut clone, p, gain);
    let accepted = led.n_consumed;
    let after_e = area * (clone.interior.a + clone.interior.r).max(0.0);
    let drift = after_e - before_e;
    let a_decay = inferred_a_decay(before, clone.interior, &led, area);
    let after_source_n = (before.n - accepted / area).max(0.0);
    let after_source_f = (before.f - accepted / area).max(0.0);
    let accelerated = after_source_n * after_source_f < 1e-8;
    let expected = led.a_produced
        - led.c_produced
        - a_decay
        - led.a_consumed_build
        - led.l_produced
        - led.reserve.r_to_w;
    SourceEval {
        requested_extent: extent,
        accepted_extent: accepted,
        drift,
        accelerated_decay: accelerated,
        after_n: clone.interior.n,
        after_f: clone.interior.f,
        accounting: SinkAccounting {
            a_produced: led.a_produced,
            a_decay,
            catalyst_a_consumption: led.c_produced,
            structural_a_consumption: led.a_consumed_build,
            membrane_a_consumption: led.l_produced,
            a_to_r: led.reserve.a_to_r,
            r_to_a: led.reserve.r_to_a,
            r_loss: led.reserve.r_to_w,
            conservation_residual: drift - expected,
        },
    }
}

fn materially_decreases(a: f64, b: f64, scale: f64) -> bool {
    b < a - (1e-10_f64.max(1e-6 * scale))
}

fn audit_state(state: &CapturedState) -> RootRecord {
    let p = reaction_params(&state.mesh);
    let area = state.mesh.area().max(1e-6);
    let saturated =
        (state.mesh.interior.n.max(0.0) * area).min(state.mesh.interior.f.max(0.0) * area);
    let fractions = [0.0, 0.25, 0.5, 0.75, 1.0];
    let shape: Vec<SourceEval> = fractions
        .iter()
        .map(|f| evaluate_source(&state.mesh, &p, saturated * f))
        .collect();
    let first_cross = shape
        .iter()
        .position(|e| e.drift >= 0.0)
        .unwrap_or(shape.len());
    let scale = shape
        .iter()
        .map(|e| e.drift.abs())
        .fold(0.0_f64, f64::max)
        .max(SOURCE_EPS);
    let nonmonotonic = (1..=first_cross.min(shape.len() - 1))
        .any(|i| materially_decreases(shape[i - 1].drift, shape[i].drift, scale));
    let accelerated_boundary_crossed = shape
        .windows(2)
        .any(|w| w[0].accelerated_decay != w[1].accelerated_decay);
    let unit = ordinary_requested(&state.mesh, &p);
    let (status, s_zero, root, root_relative_interval) = if nonmonotonic {
        ("LOCAL_SOURCE_RESPONSE_NONMONOTONIC", None, None, None)
    } else if shape[0].drift >= 0.0 {
        (
            "ZERO_SOURCE_ALREADY_NONNEGATIVE",
            Some(0.0),
            Some(shape[0].clone()),
            Some(0.0),
        )
    } else if shape.last().unwrap().drift < 0.0 {
        ("LOCAL_SOURCE_CAPACITY_INSUFFICIENT", None, None, None)
    } else {
        let mut low = 0.0;
        let mut high = saturated;
        for _ in 0..80 {
            let mid = 0.5 * (low + high);
            if evaluate_source(&state.mesh, &p, mid).drift >= 0.0 {
                high = mid;
            } else {
                low = mid;
            }
            if (high - low) / saturated.max(SOURCE_EPS) <= ROOT_REL_TOL {
                break;
            }
        }
        let eval = evaluate_source(&state.mesh, &p, high);
        (
            "FINITE_ZERO_DRIFT_ROOT",
            Some(eval.accepted_extent),
            Some(eval),
            Some((high - low) / saturated.max(SOURCE_EPS)),
        )
    };
    let max_strain = (0..state.mesh.n())
        .map(|i| state.mesh.strain(i).abs())
        .fold(0.0_f64, f64::max);
    RootRecord {
        probe: state.probe.id.into(),
        trajectory: state.trajectory.name().into(),
        step: state.step,
        area,
        a: state.mesh.interior.a,
        r: state.mesh.interior.r,
        n: state.mesh.interior.n,
        f: state.mesh.interior.f,
        c: state.mesh.interior.c,
        q_c: q_catalyst(state.mesh.interior.c, p.q_c),
        free_l: state.mesh.free_l,
        structural_mass: state.mesh.total_structural_mass(),
        bound_membrane: state.mesh.total_bound_membrane(),
        perimeter: state.mesh.perimeter(),
        max_strain,
        pre_reaction_mesh_hash: stable_json_hash(&state.mesh).unwrap(),
        e_stored_before: area * (state.mesh.interior.a + state.mesh.interior.r).max(0.0),
        constant_source: state.constant_source,
        saturated_source: shape.last().unwrap().accepted_extent,
        shape,
        status: status.into(),
        s_zero,
        s_zero_over_saturated: s_zero.map(|s| {
            if saturated > SOURCE_EPS {
                s / saturated
            } else {
                0.0
            }
        }),
        equivalent_legacy_gain: s_zero.map(|s| if unit > SOURCE_EPS { s / unit } else { 0.0 }),
        root_relative_interval,
        root,
        accelerated_boundary_crossed,
    }
}

fn sorted(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    values
}
fn quantile(values: &[f64], q: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let x = q.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let lo = x.floor() as usize;
    let hi = x.ceil() as usize;
    Some(values[lo] + (values[hi] - values[lo]) * (x - lo as f64))
}
fn distribution(values: Vec<f64>) -> Distribution {
    let v = sorted(values);
    Distribution {
        count: v.len(),
        median: quantile(&v, 0.5),
        p05: quantile(&v, 0.05),
        p95: quantile(&v, 0.95),
    }
}

fn surrogate_audit(records: &[RootRecord], probe: &str) -> SurrogateAudit {
    let rows: Vec<&RootRecord> = records
        .iter()
        .filter(|r| {
            r.probe == probe
                && r.trajectory == TrajectoryKind::Constant.name()
                && r.s_zero.unwrap_or(0.0) > SOURCE_EPS
        })
        .collect();
    let ratios: Vec<f64> = rows
        .iter()
        .map(|r| r.constant_source / r.s_zero.unwrap())
        .collect();
    let rel: Vec<f64> = rows
        .iter()
        .map(|r| (r.constant_source - r.s_zero.unwrap()) / r.s_zero.unwrap())
        .collect();
    let n = rows.len().max(1) as f64;
    let window = |lo: usize, hi: usize| -> Option<f64> {
        let v: Vec<f64> = rows
            .iter()
            .filter(|r| r.step >= lo && r.step <= hi)
            .map(|r| (r.constant_source - r.s_zero.unwrap()) / r.s_zero.unwrap())
            .collect();
        quantile(&sorted(v), 0.5)
    };
    SurrogateAudit {
        probe: probe.into(),
        ratios: distribution(ratios),
        relative_rmse: (!rows.is_empty())
            .then(|| (rel.iter().map(|x| x * x).sum::<f64>() / n).sqrt()),
        fraction_below_local_balance: (!rows.is_empty()).then(|| {
            rows.iter()
                .filter(|r| r.constant_source + SOURCE_EPS < r.s_zero.unwrap())
                .count() as f64
                / n
        }),
        fraction_materially_above: (!rows.is_empty()).then(|| {
            rows.iter()
                .filter(|r| r.constant_source > (1.0 + SURROGATE_MATERIAL_OVER) * r.s_zero.unwrap())
                .count() as f64
                / n
        }),
        early_median_residual: window(1, 160),
        middle_median_residual: window(161, 320),
        late_median_residual: window(321, 480),
    }
}

fn normalized_target(r: &RootRecord) -> f64 {
    r.s_zero.unwrap() / (r.q_c * r.area * DT).max(SOURCE_EPS)
}

fn predictor(records: &[RootRecord], include_a: bool) -> PredictorResult {
    let eligible = |r: &&RootRecord| {
        r.s_zero.unwrap_or(0.0) > SOURCE_EPS && r.status == "FINITE_ZERO_DRIFT_ROOT"
    };
    let train: Vec<&RootRecord> = records
        .iter()
        .filter(eligible)
        .filter(|r| matches!(r.probe.as_str(), "P0" | "P1" | "P2"))
        .collect();
    let holdout: Vec<&RootRecord> = records
        .iter()
        .filter(eligible)
        .filter(|r| matches!(r.probe.as_str(), "P3" | "P4"))
        .collect();
    let dims = if include_a { 3 } else { 2 };
    let mut min = vec![f64::INFINITY; dims];
    let mut max = vec![f64::NEG_INFINITY; dims];
    let raw = |r: &RootRecord| {
        if include_a {
            vec![r.n, r.f, r.a]
        } else {
            vec![r.n, r.f]
        }
    };
    for r in &train {
        for (i, x) in raw(r).iter().enumerate() {
            min[i] = min[i].min(*x);
            max[i] = max[i].max(*x);
        }
    }
    let feat = |r: &RootRecord| {
        raw(r)
            .iter()
            .enumerate()
            .map(|(i, x)| (x - min[i]) / (max[i] - min[i]).max(SOURCE_EPS))
            .collect::<Vec<_>>()
    };
    let k = K_NEIGHBORS.min(train.len().max(1));
    let mut errors = Vec::new();
    let mut ambiguity = 0.0_f64;
    for h in &holdout {
        let hf = feat(h);
        let mut neighbors: Vec<(f64, f64)> = train
            .iter()
            .map(|t| {
                let tf = feat(t);
                let d = hf
                    .iter()
                    .zip(tf.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                (d, normalized_target(t))
            })
            .collect();
        neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        let nearest = &neighbors[..k];
        let predicted = nearest.iter().map(|x| x.1).sum::<f64>() / k as f64;
        let actual = normalized_target(h);
        errors.push((predicted - actual) / actual.max(SOURCE_EPS));
        let local: Vec<f64> = nearest.iter().map(|x| x.1).collect();
        let spread = (local.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            - local.iter().copied().fold(f64::INFINITY, f64::min))
            / quantile(&sorted(local), 0.5).unwrap().max(SOURCE_EPS);
        ambiguity = ambiguity.max(spread);
    }
    let abs = sorted(errors.iter().map(|x| x.abs()).collect());
    let rmse = (errors.iter().map(|x| x * x).sum::<f64>() / errors.len().max(1) as f64).sqrt();
    let p95 = quantile(&abs, 0.95).unwrap_or(f64::INFINITY);
    PredictorResult {
        coordinate: if include_a {
            "C1=(N,F,A)".into()
        } else {
            "C0=(N,F)".into()
        },
        train_points: train.len(),
        holdout_points: holdout.len(),
        k,
        relative_rmse: rmse,
        p95_absolute_relative_error: p95,
        same_coordinate_ambiguity: ambiguity,
        sufficient: !holdout.is_empty()
            && rmse <= COORD_RMSE_LIMIT
            && p95 <= COORD_P95_LIMIT
            && ambiguity <= COORD_AMBIGUITY_LIMIT,
    }
}

fn representative(records: &[RootRecord]) -> Vec<&RootRecord> {
    let mut out = Vec::new();
    for probe in ["P0", "P1", "P2", "P3", "P4"] {
        for trajectory in [
            TrajectoryKind::Baseline.name(),
            TrajectoryKind::Constant.name(),
        ] {
            for step in [1, 240, 480] {
                if let Some(r) = records
                    .iter()
                    .find(|r| r.probe == probe && r.trajectory == trajectory && r.step == step)
                {
                    out.push(r);
                }
            }
        }
    }
    out
}

fn main() {
    let output = std::env::var_os("DCDEV020R5_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r5"));
    let external = std::env::var_os("DCDEV020R5_EXTERNAL_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r5-statewise-ledger.json"));
    let source_commit =
        std::env::var("DCDEV020R5_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let settled = settle();
    let settled_hash = stable_json_hash(&settled).unwrap();
    assert_eq!(settled_hash, "c985c08ab226a061");
    let deprived = deprive(&settled);
    let mut trajectories = Vec::new();
    let mut captures = Vec::new();
    for probe in PROBES {
        for kind in [TrajectoryKind::Baseline, TrajectoryKind::Constant] {
            let (summary, states) = replay_trajectory(&deprived, probe, kind);
            trajectories.push(summary);
            captures.extend(states);
        }
    }
    for trajectory in trajectories.iter().filter(|t| !t.parity) {
        eprintln!(
            "R4 parity mismatch {} {}: actual={} expected={}",
            trajectory.probe,
            trajectory.trajectory,
            trajectory.trajectory_hash,
            trajectory.expected_r4_hash
        );
    }
    assert!(trajectories.iter().all(|t| t.parity));
    let records: Vec<RootRecord> = captures.iter().map(audit_state).collect();
    if let Some(parent) = external.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&external, serde_json::to_vec(&records).unwrap()).unwrap();

    let total = records.len();
    let zero = records.iter().filter(|r| r.s_zero == Some(0.0)).count();
    let finite = records
        .iter()
        .filter(|r| r.status == "FINITE_ZERO_DRIFT_ROOT")
        .count();
    let insufficient = records
        .iter()
        .filter(|r| r.status == "LOCAL_SOURCE_CAPACITY_INSUFFICIENT")
        .count();
    let nonmonotonic = records
        .iter()
        .filter(|r| r.status == "LOCAL_SOURCE_RESPONSE_NONMONOTONIC")
        .count();
    let accelerated = records
        .iter()
        .filter(|r| {
            r.root
                .as_ref()
                .map(|x| x.accelerated_decay)
                .unwrap_or(false)
        })
        .count();
    let accelerated_crossed = records
        .iter()
        .filter(|r| r.accelerated_boundary_crossed)
        .count();
    let max_conservation_residual = records
        .iter()
        .filter_map(|r| r.root.as_ref())
        .map(|r| r.accounting.conservation_residual.abs())
        .fold(0.0_f64, f64::max);
    let max_source_acceptance_error = records
        .iter()
        .filter_map(|r| r.root.as_ref())
        .map(|r| (r.requested_extent - r.accepted_extent).abs())
        .fold(0.0_f64, f64::max);
    let max_root_relative_interval = records
        .iter()
        .filter_map(|r| r.root_relative_interval)
        .fold(0.0_f64, f64::max);
    let ratios: Vec<f64> = records
        .iter()
        .filter_map(|r| r.s_zero_over_saturated)
        .collect();
    let surrogate: Vec<SurrogateAudit> = ["P0", "P1", "P2", "P3", "P4"]
        .iter()
        .map(|p| surrogate_audit(&records, p))
        .collect();
    let surrogate_consistent = surrogate.iter().all(|s| {
        s.ratios
            .median
            .map(|x| (0.9..=1.1).contains(&x))
            .unwrap_or(false)
            && s.ratios.p05.unwrap_or(0.0) >= 0.75
            && s.ratios.p95.unwrap_or(f64::INFINITY) <= 1.25
            && s.relative_rmse.unwrap_or(f64::INFINITY) <= 0.25
            && s.fraction_below_local_balance.unwrap_or(1.0) <= 0.10
            && s.fraction_materially_above.unwrap_or(1.0) <= 0.10
    });
    let surrogate_class = if surrogate_consistent {
        "LOCAL_REQUIREMENT_CONSISTENT"
    } else {
        "ENDPOINT_SURROGATE_NOT_LOCAL_REQUIREMENT"
    };
    let material_nonmonotonic = nonmonotonic as f64 / total as f64 >= MATERIAL_STATE_FRACTION;
    let material_insufficient = insufficient as f64 / total as f64 >= MATERIAL_STATE_FRACTION;
    let coordinate_ran = !material_nonmonotonic && !material_insufficient && finite + zero == total;
    let c0 = coordinate_ran.then(|| predictor(&records, false));
    let c1 = coordinate_ran.then(|| predictor(&records, true));
    let coordinate_class = if !coordinate_ran {
        None
    } else if c0.as_ref().unwrap().sufficient {
        Some("DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT")
    } else if c1.as_ref().unwrap().sufficient {
        Some("DCDEV020R5_NFA_LOCAL_COORDINATE_SUFFICIENT")
    } else {
        Some("DCDEV020R5_EXISTING_LOCAL_COORDINATES_INSUFFICIENT")
    };
    let conclusion = if material_nonmonotonic {
        "DCDEV020R5_LOCAL_SOURCE_RESPONSE_NONMONOTONIC"
    } else if material_insufficient {
        "DCDEV020R5_LOCAL_SOURCE_CAPACITY_INSUFFICIENT"
    } else {
        coordinate_class.unwrap_or("DCDEV020R5_LOCAL_SOURCE_REQUIREMENT_AUDIT_COMPLETE")
    };

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-020-R5", "accepted_r4_head":ACCEPTED_R4_HEAD, "clean_scientific_base":CLEAN_BASE, "source_commit":source_commit,
            "settle_steps":SETTLE_STEPS, "deprivation_steps":DEPRIVATION_STEPS, "feed_steps":FEED_STEPS,
            "resource_center":RESOURCE_CENTER, "resource_radius":RESOURCE_RADIUS, "base_mass_m":M, "probes":PROBES,
            "trajectories":["baseline_bilinear_source","constant_endpoint_break_even_gain"],
            "shape_fractions":[0.0,0.25,0.5,0.75,1.0], "root_relative_tolerance":ROOT_REL_TOL,
            "material_state_fraction":MATERIAL_STATE_FRACTION, "surrogate_material_over_fraction":SURROGATE_MATERIAL_OVER,
            "coordinate_predictor":{"type":"fixed unweighted Euclidean k-nearest-neighbor", "k":K_NEIGHBORS,
                "training":["P0","P1","P2"], "holdout":["P3","P4"], "feature_scaling":"training min-max",
                "target":"S_zero/(q_c*area*dt)", "rmse_limit":COORD_RMSE_LIMIT, "p95_limit":COORD_P95_LIMIT, "ambiguity_limit":COORD_AMBIGUITY_LIMIT},
            "observer_only":true, "production_integration":false
        }),
    );
    write_json(
        &output,
        "results.json",
        &json!({
            "directive":"DC-DEV-020-R5", "accepted_r4_head":ACCEPTED_R4_HEAD, "clean_scientific_base":CLEAN_BASE, "source_commit":source_commit,
            "settled_hash":settled_hash, "deprived":snap(&deprived,DEPRIVATION_STEPS), "trajectory_parity":trajectories,
            "states_audited":total, "states_with_s_zero_0":zero, "finite_zero_drift_roots":finite,
            "source_capacity_insufficient_states":insufficient, "nonmonotonic_states":nonmonotonic,
            "accelerated_decay_at_root_states":accelerated, "accelerated_decay_boundary_crossed_states":accelerated_crossed,
            "max_conservation_residual":max_conservation_residual, "max_source_acceptance_error":max_source_acceptance_error,
            "max_root_relative_interval":max_root_relative_interval, "s_zero_over_saturated":distribution(ratios),
            "surrogate_audit":surrogate, "surrogate_classification":surrogate_class,
            "coordinate_audit_ran":coordinate_ran, "nf_coordinate":c0, "nfa_coordinate":c1,
            "coordinate_classification":coordinate_class, "scientific_conclusion":conclusion,
            "production_chemistry_changed":false, "production_behavior_changed":false, "implementation_authorized":false,
            "external_ledger_path":external.to_string_lossy(), "next_execution_started":false
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification":conclusion, "surrogate_classification":surrogate_class,
            "trajectory_parity":trajectories.iter().all(|t|t.parity), "observer_only":true,
            "production_integration":false, "dc_dev_021_authorized":false, "next_execution_started":false
        }),
    );
    write_json(
        &output,
        "schema.json",
        &json!({
            "dense_ledger":"array of RootRecord", "capture_boundary":"after resource uptake and before reactions_step",
            "drift":"area*(A_after+R_after)-area*(A_before+R_before)",
            "source_interval":"0 through min(N*area,F*area)", "conservation_residual":"observed stored drift minus source and all recorded stored-material sinks"
        }),
    );
    write_json(
        &output,
        "representative_diagnostics.json",
        &json!({"records":representative(&records)}),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "status":"primary_literature_reviewed_for_transient_experimental_reasoning", "external_constants_imported":false,
            "sources":[
                {"citation":"Galvez, Varon, and Canovas 1981, Transient phase of two-substrate enzyme systems", "url":"https://pubmed.ncbi.nlm.nih.gov/7278306/", "classification":"ADAPTABLE", "use":"supports transient statewise analysis of two-substrate systems; no mechanism or constants imported"},
                {"citation":"Zechel et al. 1998, Pre-steady state kinetic analysis monitored by time-resolved ESI-MS", "url":"https://pubmed.ncbi.nlm.nih.gov/9601025/", "classification":"REFERENCE_ONLY", "use":"supports direct observation of transient catalytic states; no enzyme identity, intermediate, or constants imported"},
                {"citation":"Flach and Schnell 2006, Use and abuse of the quasi-steady-state approximation", "url":"https://pmc.ncbi.nlm.nih.gov/articles/PMC2265107/", "classification":"ADAPTABLE", "use":"supports preserving full coupled transient dynamics instead of reducing to a static source equation"}
            ]
        }),
    );
    println!("DCDEV020R5_LOCAL_SOURCE_REQUIREMENT_AUDIT_COMPLETE");
    println!("states_audited={total}");
    println!("conclusion={conclusion}");
    println!("surrogate_classification={surrogate_class}");
    println!("NEXT_EXECUTION_STARTED:false");
}
