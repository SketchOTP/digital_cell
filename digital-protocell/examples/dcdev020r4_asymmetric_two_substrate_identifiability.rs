//! DC-DEV-020-R4 observer-only asymmetric two-substrate identifiability audit.
//!
//! Production chemistry is unchanged. Five preregistered finite-resource probes
//! are replayed through the existing uptake and reaction paths. The analysis
//! asks only whether independent N/F excitation identifies the bounded
//! symmetric two-substrate source family.

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
const ACCEPTED_R3_HEAD: &str = "2f32cd40e62c8874d14dfe5aa98d1837c890547f";
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
const ROOT_REL_TOL: f64 = 1e-5;
const DESIGN_CONDITION_LIMIT: f64 = 1e8;
const FAMILY_RELATIVE_TOL: f64 = 0.05;
const R3_LOW_SUBSTRATE_RATIO: f64 = 3.34757812500001;

#[derive(Clone, Copy, Debug, Serialize)]
struct Probe {
    id: &'static str,
    n_scale: f64,
    f_scale: f64,
}

const PROBES: [Probe; 5] = [
    Probe {
        id: "P0",
        n_scale: 1.0,
        f_scale: 1.0,
    },
    Probe {
        id: "P1",
        n_scale: 2.0,
        f_scale: 1.0,
    },
    Probe {
        id: "P2",
        n_scale: 1.0,
        f_scale: 2.0,
    },
    Probe {
        id: "P3",
        n_scale: 4.0,
        f_scale: 1.0,
    },
    Probe {
        id: "P4",
        n_scale: 1.0,
        f_scale: 4.0,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
enum Arm {
    Baseline,
    Constant,
    SourceSaturated,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_bilinear_source",
            Self::Constant => "constant_break_even_gain",
            Self::SourceSaturated => "source_saturated_upper_bound",
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
    world_n_loss: f64,
    world_f_loss: f64,
    max_conservation_error: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Point {
    probe: String,
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    n_times_f: f64,
    n_plus_f: f64,
    q_c: f64,
    g_h: f64,
    ordinary_requested_source: f64,
    ordinary_accepted_source: f64,
    constant_break_even_accepted_source: f64,
    source_saturated_source: f64,
    a_produced: f64,
    a_decay: f64,
    accelerated_decay_state: bool,
    a: f64,
    r: f64,
    e_stored: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
    probe: String,
    arm: String,
    gain: Option<f64>,
    initial: Snap,
    final_state: Snap,
    ledger: Ledger,
    resource_n_initial: f64,
    resource_f_initial: f64,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
    alive: bool,
    finite_nonnegative: bool,
    trajectory_hash: String,
    final_mesh_hash: String,
    #[serde(skip_serializing)]
    points: Vec<Point>,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeResult {
    probe: Probe,
    root_gain: Option<f64>,
    root_trials: Vec<Value>,
    baseline: RunSummary,
    constant: Option<RunSummary>,
    source_saturated: RunSummary,
    usable: bool,
    unusable_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Errors {
    relative_rmse: f64,
    points: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Witness {
    k_s_multiplier: f64,
    k_s: f64,
    v_max: f64,
    p3: Errors,
    p4: Errors,
    combined: Errors,
}

#[derive(Clone, Debug, Serialize)]
struct Identification {
    method: String,
    train_points: usize,
    p3_points: usize,
    p4_points: usize,
    design_rank: usize,
    design_condition: f64,
    excitation_sufficient: bool,
    alpha: Option<f64>,
    beta: Option<f64>,
    gamma: Option<f64>,
    family_consistency_relative_error: Option<f64>,
    v_max: Option<f64>,
    k_s: Option<f64>,
    training_bilinear: Errors,
    training_r3_asymptote: Errors,
    training_finite: Option<Errors>,
    p3_bilinear: Errors,
    p3_r3_asymptote: Errors,
    p3_finite: Option<Errors>,
    p4_bilinear: Errors,
    p4_r3_asymptote: Errors,
    p4_finite: Option<Errors>,
    witnesses: Vec<Witness>,
    classification: String,
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

fn ordinary_requested(mesh: &MaterialMesh, params: &ReactionParams, dt: f64) -> f64 {
    params.k_act
        * q_catalyst(mesh.interior.c, params.q_c)
        * mesh.interior.n.max(0.0)
        * mesh.interior.f.max(0.0)
        * dt
        * mesh.area().max(1e-6)
}

fn previews(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    dt: f64,
    constant_gain: f64,
) -> (ReactionLedger, ReactionLedger, f64, ReactionLedger) {
    let mut ordinary_mesh = mesh.clone();
    let ordinary = reaction_with_gain(&mut ordinary_mesh, params, dt, 1.0);
    let mut constant_mesh = mesh.clone();
    let constant = reaction_with_gain(&mut constant_mesh, params, dt, constant_gain);
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
    probe: Probe,
    arm: Arm,
    gain: f64,
    mechanics: &MechParams,
) -> RunSummary {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let initial_state = snap(&mesh, 0);
    let mass_n = M * probe.n_scale;
    let mass_f = M * probe.f_scale;
    let mut region =
        FiniteSpatialResourceRegionV1::new(RESOURCE_CENTER, RESOURCE_RADIUS, mass_n, mass_f);
    let transport = TransportParams::default();
    let mut ledger = Ledger::default();
    let mut points = Vec::with_capacity(FEED_STEPS);
    let mut hashes = vec![stable_json_hash(&initial_state).unwrap()];
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
        let area = mesh.area().max(1e-6);
        let requested = ordinary_requested(&mesh, &params, mechanics.dt);
        let (ordinary, constant, saturated_gain, saturated) =
            previews(&mesh, &params, mechanics.dt, gain);
        let applied_gain = match arm {
            Arm::Baseline => 1.0,
            Arm::Constant => gain,
            Arm::SourceSaturated => saturated_gain,
        };
        let reaction = reaction_with_gain(&mut mesh, &params, mechanics.dt, applied_gain);
        let a_decay = inferred_a_decay(before, mesh.interior, &reaction, area);
        let accelerated = mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) < 1e-8;
        ledger.accelerated_steps += usize::from(accelerated);
        ledger.accelerated_a_decay += if accelerated { a_decay } else { 0.0 };
        accumulate(&mut ledger, before, mesh.interior, &reaction, area);
        let state = snap(&mesh, step + 1);
        points.push(Point {
            probe: probe.id.into(),
            step: step + 1,
            area,
            n: before.n,
            f: before.f,
            n_times_f: before.n.max(0.0) * before.f.max(0.0),
            n_plus_f: before.n.max(0.0) + before.f.max(0.0),
            q_c: q_catalyst(before.c, params.q_c),
            g_h: 1.0,
            ordinary_requested_source: requested,
            ordinary_accepted_source: ordinary.n_consumed,
            constant_break_even_accepted_source: constant.n_consumed,
            source_saturated_source: saturated.n_consumed,
            a_produced: reaction.a_produced,
            a_decay,
            accelerated_decay_state: accelerated,
            a: mesh.interior.a,
            r: mesh.interior.r,
            e_stored: state.e_stored,
        });
        hashes.push(stable_json_hash(&state).unwrap());
    }
    let final_state = snap(&mesh, FEED_STEPS);
    RunSummary {
        probe: probe.id.into(),
        arm: arm.name().into(),
        gain: if arm == Arm::Constant {
            Some(gain)
        } else {
            None
        },
        initial: initial_state,
        final_state,
        ledger,
        resource_n_initial: mass_n,
        resource_f_initial: mass_f,
        resource_n_remaining: region.n_mass,
        resource_f_remaining: region.f_mass,
        alive: mesh.alive,
        finite_nonnegative: finite_nonnegative(&mesh),
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        points,
    }
}

fn solve_break_even(
    initial: &MaterialMesh,
    probe: Probe,
    target: f64,
    mechanics: &MechParams,
) -> (Option<f64>, Vec<Value>) {
    let mut trials = Vec::new();
    let mut low = 1.0;
    let low_run = run_feed(initial, probe, Arm::Constant, low, mechanics);
    trials.push(json!({"gain":low,"final_e_stored":low_run.final_state.e_stored}));
    if low_run.final_state.e_stored >= target {
        return (Some(low), trials);
    }
    let mut high = 2.0;
    let mut high_run = run_feed(initial, probe, Arm::Constant, high, mechanics);
    trials.push(json!({"gain":high,"final_e_stored":high_run.final_state.e_stored}));
    while high_run.final_state.e_stored < target && high < 256.0 {
        low = high;
        high *= 2.0;
        high_run = run_feed(initial, probe, Arm::Constant, high, mechanics);
        trials.push(json!({"gain":high,"final_e_stored":high_run.final_state.e_stored}));
    }
    if high_run.final_state.e_stored < target {
        return (None, trials);
    }
    for _ in 0..48 {
        let mid = 0.5 * (low + high);
        let run = run_feed(initial, probe, Arm::Constant, mid, mechanics);
        trials.push(json!({"gain":mid,"final_e_stored":run.final_state.e_stored}));
        if run.final_state.e_stored >= target {
            high = mid;
        } else {
            low = mid;
        }
        if (high - low) / high.max(1.0) <= ROOT_REL_TOL {
            break;
        }
    }
    (Some(high), trials)
}

fn run_probe(initial: &MaterialMesh, probe: Probe, mechanics: &MechParams) -> ProbeResult {
    let baseline = run_feed(initial, probe, Arm::Baseline, 1.0, mechanics);
    let source_saturated = run_feed(initial, probe, Arm::SourceSaturated, 1.0, mechanics);
    let (root_gain, root_trials) =
        solve_break_even(initial, probe, baseline.initial.e_stored, mechanics);
    let constant = root_gain.map(|gain| run_feed(initial, probe, Arm::Constant, gain, mechanics));
    let paired_delivered = baseline.ledger.n_delivered.min(baseline.ledger.f_delivered);
    let usable = baseline.alive
        && baseline.finite_nonnegative
        && source_saturated.alive
        && source_saturated.finite_nonnegative
        && baseline.ledger.max_conservation_error <= MASS_EPS
        && source_saturated.ledger.max_conservation_error <= MASS_EPS
        && paired_delivered > SOURCE_EPS
        && root_gain.is_some()
        && constant
            .as_ref()
            .map(|run| {
                run.points.iter().all(|point| {
                    point.constant_break_even_accepted_source
                        <= point.source_saturated_source + MASS_EPS
                })
            })
            .unwrap_or(false);
    let reason = if usable {
        None
    } else if root_gain.is_none() {
        Some("no finite constant break-even root".into())
    } else if paired_delivered <= SOURCE_EPS {
        Some("no convertible paired substrate entered the organism".into())
    } else if !baseline.finite_nonnegative || !source_saturated.finite_nonnegative {
        Some("non-finite or negative state".into())
    } else {
        Some("conservation or source-saturation comparability failed".into())
    };
    ProbeResult {
        probe,
        root_gain,
        root_trials,
        baseline,
        constant,
        source_saturated,
        usable,
        unusable_reason: reason,
    }
}

fn usable_points<'a>(probes: &'a [ProbeResult], ids: &[&str]) -> Vec<&'a Point> {
    probes
        .iter()
        .filter(|probe| ids.contains(&probe.probe.id))
        .filter_map(|probe| probe.constant.as_ref())
        .flat_map(|run| run.points.iter())
        .filter(|point| {
            point.n > SOURCE_EPS
                && point.f > SOURCE_EPS
                && point.constant_break_even_accepted_source > SOURCE_EPS
                && point.constant_break_even_accepted_source
                    < point.source_saturated_source - SOURCE_EPS
        })
        .collect()
}

fn solve_3x3(mut a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for pivot in 0..3 {
        let row = (pivot..3)
            .max_by(|&i, &j| a[i][pivot].abs().partial_cmp(&a[j][pivot].abs()).unwrap())
            .unwrap();
        if a[row][pivot].abs() <= 1e-14 {
            return None;
        }
        a.swap(pivot, row);
        b.swap(pivot, row);
        for i in (pivot + 1)..3 {
            let factor = a[i][pivot] / a[pivot][pivot];
            for j in pivot..3 {
                a[i][j] -= factor * a[pivot][j];
            }
            b[i] -= factor * b[pivot];
        }
    }
    let mut x = [0.0; 3];
    for i in (0..3).rev() {
        x[i] = (b[i] - ((i + 1)..3).map(|j| a[i][j] * x[j]).sum::<f64>()) / a[i][i];
    }
    Some(x)
}

fn eigenvalues_symmetric_3x3(mut a: [[f64; 3]; 3]) -> [f64; 3] {
    for _ in 0..32 {
        let mut p = 0;
        let mut q = 1;
        for i in 0..3 {
            for j in (i + 1)..3 {
                if a[i][j].abs() > a[p][q].abs() {
                    p = i;
                    q = j;
                }
            }
        }
        if a[p][q].abs() <= 1e-14 {
            break;
        }
        let phi = 0.5 * (2.0 * a[p][q]).atan2(a[q][q] - a[p][p]);
        let (c, s) = (phi.cos(), phi.sin());
        let app = c * c * a[p][p] - 2.0 * s * c * a[p][q] + s * s * a[q][q];
        let aqq = s * s * a[p][p] + 2.0 * s * c * a[p][q] + c * c * a[q][q];
        for k in 0..3 {
            if k != p && k != q {
                let akp = c * a[k][p] - s * a[k][q];
                let akq = s * a[k][p] + c * a[k][q];
                a[k][p] = akp;
                a[p][k] = akp;
                a[k][q] = akq;
                a[q][k] = akq;
            }
        }
        a[p][p] = app;
        a[q][q] = aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;
    }
    let mut values = [a[0][0], a[1][1], a[2][2]];
    values.sort_by(|x, y| x.partial_cmp(y).unwrap());
    values
}

fn design_diagnostics(points: &[&Point]) -> (usize, f64) {
    let mut scales = [0.0_f64; 3];
    for point in points {
        let row = [1.0, point.n_plus_f, point.n_times_f];
        for i in 0..3 {
            scales[i] += row[i] * row[i];
        }
    }
    for scale in &mut scales {
        *scale = (*scale / points.len().max(1) as f64).sqrt().max(1e-15);
    }
    let mut gram = [[0.0; 3]; 3];
    for point in points {
        let row = [
            1.0 / scales[0],
            point.n_plus_f / scales[1],
            point.n_times_f / scales[2],
        ];
        for i in 0..3 {
            for j in 0..3 {
                gram[i][j] += row[i] * row[j];
            }
        }
    }
    let eigen = eigenvalues_symmetric_3x3(gram);
    let max = eigen[2].max(0.0);
    let tolerance = max * 1e-12;
    let rank = eigen.iter().filter(|value| **value > tolerance).count();
    let condition = if rank == 3 {
        (eigen[2] / eigen[0]).sqrt()
    } else {
        f64::INFINITY
    };
    (rank, condition)
}

fn fit_reciprocal(points: &[&Point]) -> Option<[f64; 3]> {
    let mut normal = [[0.0; 3]; 3];
    let mut rhs = [0.0; 3];
    for point in points {
        let row = [1.0, point.n_plus_f, point.n_times_f];
        let z = point.q_c * point.g_h * point.n_times_f * point.area * DT
            / point.constant_break_even_accepted_source;
        for i in 0..3 {
            rhs[i] += row[i] * z;
            for j in 0..3 {
                normal[i][j] += row[i] * row[j];
            }
        }
    }
    solve_3x3(normal, rhs)
}

fn saturating_extent(point: &Point, v_max: f64, k_s: f64) -> f64 {
    let denominator = k_s * k_s + k_s * point.n_plus_f + point.n_times_f;
    point.q_c * point.g_h * v_max * point.n_times_f / denominator * DT * point.area
}

fn errors(points: &[&Point], model: Option<(f64, f64)>, r3: bool) -> Errors {
    let mut sum = 0.0;
    let mut count = 0;
    for point in points {
        let expected = point.constant_break_even_accepted_source;
        if expected <= SOURCE_EPS {
            continue;
        }
        let predicted = if let Some((v, k)) = model {
            saturating_extent(point, v, k)
        } else if r3 {
            point.q_c * point.g_h * R3_LOW_SUBSTRATE_RATIO * point.n_times_f * DT * point.area
        } else {
            point.ordinary_requested_source
        };
        sum += ((predicted - expected) / expected).powi(2);
        count += 1;
    }
    Errors {
        relative_rmse: (sum / count.max(1) as f64).sqrt(),
        points: count,
    }
}

fn identify(probes: &[ProbeResult]) -> Identification {
    let train = usable_points(probes, &["P0", "P1", "P2"]);
    let p3 = usable_points(probes, &["P3"]);
    let p4 = usable_points(probes, &["P4"]);
    let (rank, condition) = design_diagnostics(&train);
    let excitation = rank == 3 && condition.is_finite() && condition <= DESIGN_CONDITION_LIMIT;
    let coefficients = if excitation {
        fit_reciprocal(&train)
    } else {
        None
    };
    let positive = coefficients
        .map(|c| c.iter().all(|v| v.is_finite() && *v > 0.0))
        .unwrap_or(false);
    let consistency = coefficients.map(|c| {
        let lhs = c[0] * c[2];
        let rhs = c[1] * c[1];
        (lhs - rhs).abs() / lhs.abs().max(rhs.abs()).max(SOURCE_EPS)
    });
    let family_ok = positive && consistency.unwrap_or(f64::INFINITY) <= FAMILY_RELATIVE_TOL;
    let model = if family_ok {
        let c = coefficients.unwrap();
        Some((1.0 / c[2], c[1] / c[2]))
    } else {
        None
    };
    let mut witnesses = Vec::new();
    if let Some((_, k_s)) = model {
        for multiplier in [0.1, 10.0, 100.0] {
            let witness_k = k_s * multiplier;
            let witness_v = R3_LOW_SUBSTRATE_RATIO * witness_k * witness_k;
            let mut combined = p3.clone();
            combined.extend(p4.iter().copied());
            witnesses.push(Witness {
                k_s_multiplier: multiplier,
                k_s: witness_k,
                v_max: witness_v,
                p3: errors(&p3, Some((witness_v, witness_k)), false),
                p4: errors(&p4, Some((witness_v, witness_k)), false),
                combined: errors(&combined, Some((witness_v, witness_k)), false),
            });
        }
    }
    let finite_p3 = model.map(|m| errors(&p3, Some(m), false));
    let finite_p4 = model.map(|m| errors(&p4, Some(m), false));
    let finite_combined = model.map(|m| {
        let mut combined = p3.clone();
        combined.extend(p4.iter().copied());
        errors(&combined, Some(m), false)
    });
    let boundary_pass = finite_combined
        .map(|finite| {
            witnesses
                .iter()
                .all(|w| finite.relative_rmse < w.combined.relative_rmse)
        })
        .unwrap_or(false);
    let direction_pass = finite_p3
        .zip(finite_p4)
        .map(|(a, b)| {
            a.relative_rmse < errors(&p3, None, false).relative_rmse
                && b.relative_rmse < errors(&p4, None, false).relative_rmse
                && a.relative_rmse < errors(&p3, None, true).relative_rmse
                && b.relative_rmse < errors(&p4, None, true).relative_rmse
        })
        .unwrap_or(false);
    let classification = if !excitation {
        "DCDEV020R4_TWO_SUBSTRATE_EXCITATION_INSUFFICIENT"
    } else if !family_ok {
        "DCDEV020R4_SATURATING_FAMILY_STRUCTURAL_MISMATCH"
    } else if !direction_pass || !boundary_pass {
        "DCDEV020R4_SATURATING_KINETICS_STILL_NOT_IDENTIFIABLE"
    } else {
        "DCDEV020R4_TWO_SUBSTRATE_SATURATING_KINETICS_IDENTIFIED_PENDING_QUALIFICATION"
    };
    Identification {
        method: "deterministic reciprocal normal-equation least squares; no parameter sweep".into(),
        train_points: train.len(),
        p3_points: p3.len(),
        p4_points: p4.len(),
        design_rank: rank,
        design_condition: condition,
        excitation_sufficient: excitation,
        alpha: coefficients.map(|c| c[0]),
        beta: coefficients.map(|c| c[1]),
        gamma: coefficients.map(|c| c[2]),
        family_consistency_relative_error: consistency,
        v_max: model.map(|m| m.0),
        k_s: model.map(|m| m.1),
        training_bilinear: errors(&train, None, false),
        training_r3_asymptote: errors(&train, None, true),
        training_finite: model.map(|m| errors(&train, Some(m), false)),
        p3_bilinear: errors(&p3, None, false),
        p3_r3_asymptote: errors(&p3, None, true),
        p3_finite: finite_p3,
        p4_bilinear: errors(&p4, None, false),
        p4_r3_asymptote: errors(&p4, None, true),
        p4_finite: finite_p4,
        witnesses,
        classification: classification.into(),
    }
}

fn main() {
    let output = std::env::var_os("DCDEV020R4_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r4"));
    let source_commit =
        std::env::var("DCDEV020R4_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= 1e-12);
    let settled = settle(&mechanics);
    let settled_hash = stable_json_hash(&settled).unwrap();
    let (deprived, deprived_state) = deprive(&settled, &mechanics);
    assert!((deprived_state.e_stored - DEPRIVED_REFERENCE).abs() <= 1e-10);

    let probes: Vec<ProbeResult> = PROBES
        .iter()
        .copied()
        .map(|probe| run_probe(&deprived, probe, &mechanics))
        .collect();
    let all_usable = probes.iter().all(|probe| probe.usable);
    let identification = if all_usable {
        Some(identify(&probes))
    } else {
        None
    };
    let conclusion = if !all_usable {
        "DCDEV020R4_ASYMMETRIC_IDENTIFICATION_PROBE_INVALID"
    } else {
        identification.as_ref().unwrap().classification.as_str()
    };
    let points: Vec<&Point> = probes
        .iter()
        .filter_map(|p| p.constant.as_ref())
        .flat_map(|run| run.points.iter())
        .collect();

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-020-R4", "accepted_r3_head":ACCEPTED_R3_HEAD,
            "clean_scientific_base":CLEAN_BASE, "source_commit":source_commit,
            "settle_steps":SETTLE_STEPS, "deprivation_steps":DEPRIVATION_STEPS,
            "feed_steps":FEED_STEPS, "resource_center":RESOURCE_CENTER,
            "resource_radius":RESOURCE_RADIUS, "base_mass_m":M,
            "probes":PROBES, "training":["P0","P1","P2"], "holdout":["P3","P4"],
            "root_relative_tolerance":ROOT_REL_TOL,
            "design_condition_limit":DESIGN_CONDITION_LIMIT,
            "family_consistency_relative_tolerance":FAMILY_RELATIVE_TOL,
            "r3_low_substrate_ratio":R3_LOW_SUBSTRATE_RATIO,
            "candidate_family":"q_c * g_h * V_max * N*F/(K_S^2 + K_S*N + K_S*F + N*F)",
            "observer_only":true, "production_integration":false
        }),
    );
    write_json(
        &output,
        "results.json",
        &json!({
            "directive":"DC-DEV-020-R4", "accepted_r3_head":ACCEPTED_R3_HEAD,
            "clean_scientific_base":CLEAN_BASE, "source_commit":source_commit,
            "observer_only":true, "production_chemistry_changed":false,
            "production_behavior_changed":false, "settled_hash":settled_hash,
            "deprived":deprived_state, "probes":probes,
            "identification":identification, "conclusion":conclusion,
            "qualification_run":false, "implementation_authorized":false,
            "next_execution_started":false
        }),
    );
    write_json(
        &output,
        "kinetic_identification.json",
        &json!({
            "record_count":points.len(), "points":points
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification":conclusion, "all_probes_usable":all_usable,
            "identification":identification, "implementation_authorized":false,
            "qualification_run":false, "next_execution_started":false
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "status":"primary_literature_reviewed_for_experimental_structure",
            "disposition":"ADAPTABLE_ARCHITECTURE_ONLY",
            "external_constants_imported":false, "molecular_identities_imported":false,
            "experimental_concentrations_imported":false,
            "sources":[
                {"citation":"Cleland 1963, The kinetics of enzyme-catalyzed reactions with two or more substrates or products. I. Nomenclature and rate equations",
                 "url":"https://doi.org/10.1016/0926-6569(63)90211-6", "classification":"ADAPTABLE",
                 "reusable_finding":"Use an explicit multi-reactant rate equation and preserve its reciprocal coefficient constraints.", "imported_constants":false},
                {"citation":"Pettersson 1969, Relationships between rapid equilibrium conditions and linearization of the reciprocal rate equation for the sequential random two-substrate enzyme mechanism",
                 "url":"https://pubmed.ncbi.nlm.nih.gov/5381399/", "classification":"ADAPTABLE",
                 "reusable_finding":"Vary substrate conditions and test reciprocal-rate relationships rather than inferring two parameters from one coupled axis.", "imported_constants":false},
                {"citation":"Wang and Mittermaier 2021, Characterizing Bi-substrate Enzyme Kinetics at High Resolution by 2D-ITC",
                 "url":"https://pubmed.ncbi.nlm.nih.gov/34514786/", "classification":"ADAPTABLE",
                 "reusable_finding":"Characterize a bi-substrate response by independently varying both substrate concentrations.", "imported_constants":false}
            ]
        }),
    );
    println!("DCDEV020R4_ASYMMETRIC_IDENTIFIABILITY_AUDIT_COMPLETE");
    println!("all_probes_usable={all_usable}");
    println!("conclusion={conclusion}");
    println!("NEXT_EXECUTION_STARTED:false");
}
