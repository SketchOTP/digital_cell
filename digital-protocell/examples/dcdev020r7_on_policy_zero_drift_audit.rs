//! DC-DEV-020-R7 observer-only on-policy zero-drift audit.
//!
//! Replays the accepted R6 finite-feed arm, solves the frozen R5 physical
//! zero-drift root on every induced state, replays the frozen R5 NF/NFA
//! observers without refitting, and executes one exact-root oracle control.
//! Production chemistry and behavior are unchanged.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{q_catalyst, reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

const ACCEPTED_R6_HEAD: &str = "f01b716d9051c9f0114f3c5c0d1b123e2df037cf";
const CLEAN_BASE: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const ACCEPTED_R6_HASH: &str = "97010613dc36e447";
const R5_LEDGER_SHA256: &str = "4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585";
const R5_EXTERNAL_LOCATION: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r5/6dbb8d45c520e2756a81b2cc1e81dff9c3878992/statewise_root_ledger.json";
const SETTLE_STEPS: usize = 5_000;
const WINDOW: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const M_SELECTED: f64 = 19.878372106390554;
const DT: f64 = 0.02;
const E_DEPRIVED: f64 = 60.82781514212436;
const R6_FINAL_E: f64 = 60.0620310117838;
const R6_FINAL_A: f64 = 0.3423623895976825;
const R6_FINAL_R: f64 = 0.5056416879564652;
const R6_FINAL_NF: f64 = 0.10185789865759344;
const R6_FINAL_C: f64 = 0.7722488011667238;
const K_PL: f64 = 0.017556661171593057;
const POWER_P: f64 = 0.0003277429681759396;
const SOURCE_EPS: f64 = 1e-12;
const MASS_TOL: f64 = 1e-10;
const ROOT_REL_TOL: f64 = 1e-6;
const MATERIAL_ERROR: f64 = 0.10;
const R5_MATERIAL_STATE_FRACTION: f64 = 0.01;
const K_NEIGHBORS: usize = 16;
const RMSE_LIMIT: f64 = 0.15;
const P95_LIMIT: f64 = 0.30;
const AMBIGUITY_LIMIT: f64 = 0.25;

#[derive(Clone, Debug, Deserialize)]
struct R5Root {
    probe: String,
    area: f64,
    a: f64,
    n: f64,
    f: f64,
    q_c: f64,
    status: String,
    s_zero: Option<f64>,
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

#[derive(Clone, Debug, Serialize)]
struct SinkAccounting {
    a_produced: f64,
    a_decay: f64,
    catalyst_a_consumption: f64,
    structural_a_consumption: f64,
    membrane_a_consumption: f64,
    reserve_loss: f64,
    conservation_residual: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SourceEval {
    requested_extent: f64,
    accepted_extent: f64,
    drift: f64,
    accelerated_decay: bool,
    accounting: SinkAccounting,
}

#[derive(Clone, Debug, Serialize)]
struct RootAudit {
    status: String,
    saturated_source: f64,
    shape: Vec<SourceEval>,
    s_zero: Option<f64>,
    s_zero_over_saturated: Option<f64>,
    root_relative_interval: Option<f64>,
    root: Option<SourceEval>,
    accelerated_boundary_crossed: bool,
}

#[derive(Clone, Debug, Serialize)]
struct OnPolicyRecord {
    step: usize,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    c: f64,
    q_c: f64,
    structural_mass: f64,
    bound_membrane: f64,
    free_membrane: f64,
    perimeter: f64,
    max_strain: f64,
    pre_reaction_mesh_hash: String,
    e_stored_before: f64,
    r6_requested_source: f64,
    r6_accepted_source: f64,
    r6_drift: f64,
    root_audit: RootAudit,
}

#[derive(Clone, Debug, Serialize)]
struct Distribution {
    count: usize,
    min: Option<f64>,
    p05: Option<f64>,
    median: Option<f64>,
    p95: Option<f64>,
    max: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct ErrorReport {
    points: usize,
    relative_rmse: f64,
    p95_absolute_relative_error: f64,
    source_over_root: Distribution,
    fraction_below_root: f64,
    fraction_materially_below: f64,
    fraction_materially_above: f64,
}

#[derive(Clone, Debug, Serialize)]
struct WindowErrors {
    early: ErrorReport,
    middle: ErrorReport,
    late: ErrorReport,
}

#[derive(Clone, Debug, Serialize)]
struct SupportReport {
    coordinate: String,
    training_to_training: Distribution,
    r5_holdout_to_training: Distribution,
    r6_on_policy_to_training: Distribution,
}

#[derive(Clone, Debug, Serialize)]
struct ObserverReport {
    coordinate: String,
    train_points: usize,
    evaluation_points: usize,
    k: usize,
    relative_rmse: f64,
    p95_absolute_relative_error: f64,
    ambiguity: f64,
    early: ErrorReport,
    middle: ErrorReport,
    late: ErrorReport,
    sufficient: bool,
}

#[derive(Clone, Debug, Serialize)]
struct OracleSummary {
    initial: Snap,
    final_state: Snap,
    total_n_consumed: f64,
    total_f_consumed: f64,
    source_capacity_fraction: Distribution,
    accelerated_decay_steps: usize,
    max_accounting_residual: f64,
    settled_distance_initial: f64,
    settled_distance_final: f64,
    trajectory_hash: String,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
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
    for _ in 0..WINDOW {
        reactions_step(&mut mesh, &p, DT, true, true);
    }
    assert!((snap(&mesh, WINDOW).e_stored - E_DEPRIVED).abs() <= MASS_TOL);
    mesh
}

fn ordinary_requested(mesh: &MaterialMesh, p: &ReactionParams) -> f64 {
    p.k_act
        * q_catalyst(mesh.interior.c, p.q_c)
        * mesh.interior.n.max(0.0)
        * mesh.interior.f.max(0.0)
        * mesh.area().max(1e-6)
        * DT
}

fn r6_requested(mesh: &MaterialMesh, p: &ReactionParams) -> f64 {
    if mesh.interior.n <= 0.0 || mesh.interior.f <= 0.0 {
        0.0
    } else {
        q_catalyst(mesh.interior.c, p.q_c)
            * K_PL
            * mesh.interior.n.powf(POWER_P)
            * mesh.interior.f.powf(POWER_P)
            * mesh.area().max(1e-6)
            * DT
    }
}

fn inferred_a_decay(
    before: LumpedChem,
    after: LumpedChem,
    ledger: &ReactionLedger,
    area: f64,
) -> f64 {
    (before.a * area + ledger.a_produced
        - ledger.c_produced
        - after.a * area
        - ledger.a_consumed_build
        - ledger.l_produced
        - ledger.reserve.a_to_r
        + ledger.reserve.r_to_a)
        .max(0.0)
}

fn execute_extent(
    mesh: &mut MaterialMesh,
    p: &ReactionParams,
    extent: f64,
) -> (ReactionLedger, SourceEval) {
    let before = mesh.interior;
    let area = mesh.area().max(1e-6);
    let before_e = area * (before.a + before.r).max(0.0);
    let unit = ordinary_requested(mesh, p);
    let gain = if extent <= SOURCE_EPS {
        0.0
    } else {
        extent / unit.max(SOURCE_EPS)
    };
    let mut effective = *p;
    effective.k_act = p.k_act * gain;
    let ledger = reactions_step(mesh, &effective, DT, true, true);
    let after_e = area * (mesh.interior.a + mesh.interior.r).max(0.0);
    let decay = inferred_a_decay(before, mesh.interior, &ledger, area);
    let expected = ledger.a_produced
        - ledger.c_produced
        - decay
        - ledger.a_consumed_build
        - ledger.l_produced
        - ledger.reserve.r_to_w;
    let accepted = ledger.n_consumed;
    let after_source_n = (before.n - accepted / area).max(0.0);
    let after_source_f = (before.f - accepted / area).max(0.0);
    let eval = SourceEval {
        requested_extent: extent,
        accepted_extent: accepted,
        drift: after_e - before_e,
        accelerated_decay: after_source_n * after_source_f < 1e-8,
        accounting: SinkAccounting {
            a_produced: ledger.a_produced,
            a_decay: decay,
            catalyst_a_consumption: ledger.c_produced,
            structural_a_consumption: ledger.a_consumed_build,
            membrane_a_consumption: ledger.l_produced,
            reserve_loss: ledger.reserve.r_to_w,
            conservation_residual: (after_e - before_e) - expected,
        },
    };
    (ledger, eval)
}

fn evaluate_source(mesh: &MaterialMesh, p: &ReactionParams, extent: f64) -> SourceEval {
    let mut clone = mesh.clone();
    execute_extent(&mut clone, p, extent).1
}

fn materially_decreases(a: f64, b: f64, scale: f64) -> bool {
    b < a - 1e-10_f64.max(1e-6 * scale)
}

fn audit_root(mesh: &MaterialMesh, p: &ReactionParams) -> RootAudit {
    let area = mesh.area().max(1e-6);
    let saturated = (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area);
    let shape: Vec<SourceEval> = [0.0, 0.25, 0.50, 0.75, 1.0]
        .iter()
        .map(|f| evaluate_source(mesh, p, saturated * f))
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
    let (status, s_zero, root, interval) = if nonmonotonic {
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
            if evaluate_source(mesh, p, mid).drift >= 0.0 {
                high = mid;
            } else {
                low = mid;
            }
            if (high - low) / saturated.max(SOURCE_EPS) <= ROOT_REL_TOL {
                break;
            }
        }
        let eval = evaluate_source(mesh, p, high);
        (
            "FINITE_ZERO_DRIFT_ROOT",
            Some(eval.accepted_extent),
            Some(eval),
            Some((high - low) / saturated.max(SOURCE_EPS)),
        )
    };
    RootAudit {
        status: status.into(),
        saturated_source: shape.last().unwrap().accepted_extent,
        shape,
        s_zero,
        s_zero_over_saturated: s_zero.map(|s| s / saturated.max(SOURCE_EPS)),
        root_relative_interval: interval,
        root,
        accelerated_boundary_crossed,
    }
}

fn sorted(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    v
}

fn quantile(v: &[f64], q: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let x = q.clamp(0.0, 1.0) * (v.len() - 1) as f64;
    let lo = x.floor() as usize;
    let hi = x.ceil() as usize;
    Some(v[lo] + (v[hi] - v[lo]) * (x - lo as f64))
}

fn distribution(v: Vec<f64>) -> Distribution {
    let v = sorted(v);
    Distribution {
        count: v.len(),
        min: v.first().copied(),
        p05: quantile(&v, 0.05),
        median: quantile(&v, 0.50),
        p95: quantile(&v, 0.95),
        max: v.last().copied(),
    }
}

fn errors_from_pairs(pairs: &[(f64, f64)]) -> ErrorReport {
    let errors: Vec<f64> = pairs
        .iter()
        .map(|(predicted, actual)| (predicted - actual) / actual.max(SOURCE_EPS))
        .collect();
    let ratios: Vec<f64> = pairs
        .iter()
        .map(|(predicted, actual)| predicted / actual.max(SOURCE_EPS))
        .collect();
    let n = pairs.len().max(1) as f64;
    ErrorReport {
        points: pairs.len(),
        relative_rmse: (errors.iter().map(|x| x * x).sum::<f64>() / n).sqrt(),
        p95_absolute_relative_error: quantile(
            &sorted(errors.iter().map(|x| x.abs()).collect()),
            0.95,
        )
        .unwrap_or(f64::INFINITY),
        source_over_root: distribution(ratios),
        fraction_below_root: pairs.iter().filter(|(p, a)| p + SOURCE_EPS < *a).count() as f64 / n,
        fraction_materially_below: pairs
            .iter()
            .filter(|(p, a)| *p < (1.0 - MATERIAL_ERROR) * *a)
            .count() as f64
            / n,
        fraction_materially_above: pairs
            .iter()
            .filter(|(p, a)| *p > (1.0 + MATERIAL_ERROR) * *a)
            .count() as f64
            / n,
    }
}

fn run_r6(deprived: &MaterialMesh) -> (Vec<OnPolicyRecord>, Snap, String, f64) {
    let mut mesh = deprived.clone();
    let p = reaction_params(&mesh);
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED,
        M_SELECTED,
    );
    let mut hashes = vec![stable_json_hash(&snap(&mesh, 0)).unwrap()];
    let mut records = Vec::with_capacity(WINDOW);
    let mut uptake_error = 0.0_f64;
    for step in 1..=WINDOW {
        let uptake = region.uptake(&mut mesh, &transport, DT);
        uptake_error = uptake_error.max(uptake.conservation_error.abs());
        let area = mesh.area().max(1e-6);
        let before_e = area * (mesh.interior.a + mesh.interior.r).max(0.0);
        let requested = r6_requested(&mesh, &p);
        let root_audit = audit_root(&mesh, &p);
        let pre_hash = stable_json_hash(&snap(&mesh, step)).unwrap();
        let max_strain = (0..mesh.n())
            .map(|i| mesh.strain(i).abs())
            .fold(0.0_f64, f64::max);
        let state = (
            area,
            mesh.interior,
            mesh.total_structural_mass(),
            mesh.total_bound_membrane(),
            mesh.free_l,
            mesh.perimeter(),
            max_strain,
        );
        let (_, executed) = execute_extent(&mut mesh, &p, requested);
        records.push(OnPolicyRecord {
            step,
            area: state.0,
            a: state.1.a,
            r: state.1.r,
            n: state.1.n,
            f: state.1.f,
            c: state.1.c,
            q_c: q_catalyst(state.1.c, p.q_c),
            structural_mass: state.2,
            bound_membrane: state.3,
            free_membrane: state.4,
            perimeter: state.5,
            max_strain: state.6,
            pre_reaction_mesh_hash: pre_hash,
            e_stored_before: before_e,
            r6_requested_source: requested,
            r6_accepted_source: executed.accepted_extent,
            r6_drift: executed.drift,
            root_audit,
        });
        hashes.push(stable_json_hash(&snap(&mesh, step)).unwrap());
    }
    (
        records,
        snap(&mesh, WINDOW),
        stable_json_hash(&hashes).unwrap(),
        uptake_error,
    )
}

fn raw_r5(r: &R5Root, include_a: bool) -> Vec<f64> {
    if include_a {
        vec![r.n, r.f, r.a]
    } else {
        vec![r.n, r.f]
    }
}

fn raw_on_policy(r: &OnPolicyRecord, include_a: bool) -> Vec<f64> {
    if include_a {
        vec![r.n, r.f, r.a]
    } else {
        vec![r.n, r.f]
    }
}

fn eligible_r5(r: &R5Root) -> bool {
    r.status == "FINITE_ZERO_DRIFT_ROOT" && r.s_zero.unwrap_or(0.0) > SOURCE_EPS
}

fn scaling(train: &[&R5Root], include_a: bool) -> (Vec<f64>, Vec<f64>) {
    let dims = if include_a { 3 } else { 2 };
    let mut min = vec![f64::INFINITY; dims];
    let mut max = vec![f64::NEG_INFINITY; dims];
    for r in train {
        for (i, x) in raw_r5(r, include_a).iter().enumerate() {
            min[i] = min[i].min(*x);
            max[i] = max[i].max(*x);
        }
    }
    (min, max)
}

fn scale(raw: &[f64], min: &[f64], max: &[f64]) -> Vec<f64> {
    raw.iter()
        .enumerate()
        .map(|(i, x)| (x - min[i]) / (max[i] - min[i]).max(SOURCE_EPS))
        .collect()
}

fn distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn normalized_target(r: &R5Root) -> f64 {
    r.s_zero.unwrap() / (r.q_c * r.area * DT).max(SOURCE_EPS)
}

fn support_report(
    records: &[R5Root],
    on_policy: &[OnPolicyRecord],
    include_a: bool,
) -> SupportReport {
    let train: Vec<&R5Root> = records
        .iter()
        .filter(|r| eligible_r5(r) && matches!(r.probe.as_str(), "P0" | "P1" | "P2"))
        .collect();
    let holdout: Vec<&R5Root> = records
        .iter()
        .filter(|r| eligible_r5(r) && matches!(r.probe.as_str(), "P3" | "P4"))
        .collect();
    let (min, max) = scaling(&train, include_a);
    let train_features: Vec<Vec<f64>> = train
        .iter()
        .map(|r| scale(&raw_r5(r, include_a), &min, &max))
        .collect();
    let train_distance: Vec<f64> = train_features
        .iter()
        .enumerate()
        .map(|(i, x)| {
            train_features
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, y)| distance(x, y))
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    let nearest = |raw: Vec<f64>| {
        let x = scale(&raw, &min, &max);
        train_features
            .iter()
            .map(|y| distance(&x, y))
            .fold(f64::INFINITY, f64::min)
    };
    SupportReport {
        coordinate: if include_a {
            "C1=(N,F,A)".into()
        } else {
            "C0=(N,F)".into()
        },
        training_to_training: distribution(train_distance),
        r5_holdout_to_training: distribution(
            holdout
                .iter()
                .map(|r| nearest(raw_r5(r, include_a)))
                .collect(),
        ),
        r6_on_policy_to_training: distribution(
            on_policy
                .iter()
                .map(|r| nearest(raw_on_policy(r, include_a)))
                .collect(),
        ),
    }
}

fn observer_report(
    records: &[R5Root],
    on_policy: &[OnPolicyRecord],
    include_a: bool,
) -> ObserverReport {
    let train: Vec<&R5Root> = records
        .iter()
        .filter(|r| eligible_r5(r) && matches!(r.probe.as_str(), "P0" | "P1" | "P2"))
        .collect();
    let (min, max) = scaling(&train, include_a);
    let train_rows: Vec<(Vec<f64>, f64)> = train
        .iter()
        .map(|r| {
            (
                scale(&raw_r5(r, include_a), &min, &max),
                normalized_target(r),
            )
        })
        .collect();
    let k = K_NEIGHBORS.min(train_rows.len().max(1));
    let mut pairs = Vec::with_capacity(on_policy.len());
    let mut ambiguity = 0.0_f64;
    for row in on_policy {
        let x = scale(&raw_on_policy(row, include_a), &min, &max);
        let mut neighbors: Vec<(f64, f64)> = train_rows
            .iter()
            .map(|(f, target)| (distance(&x, f), *target))
            .collect();
        neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        let nearest = &neighbors[..k];
        let predicted_normalized = nearest.iter().map(|x| x.1).sum::<f64>() / k as f64;
        let local = sorted(nearest.iter().map(|x| x.1).collect());
        let spread = (local.last().unwrap() - local.first().unwrap())
            / quantile(&local, 0.5).unwrap().max(SOURCE_EPS);
        ambiguity = ambiguity.max(spread);
        let predicted = predicted_normalized * row.q_c * row.area * DT;
        pairs.push((predicted, row.root_audit.s_zero.unwrap()));
    }
    let all = errors_from_pairs(&pairs);
    let early = errors_from_pairs(&pairs[0..160]);
    let middle = errors_from_pairs(&pairs[160..320]);
    let late = errors_from_pairs(&pairs[320..480]);
    ObserverReport {
        coordinate: if include_a {
            "C1=(N,F,A)".into()
        } else {
            "C0=(N,F)".into()
        },
        train_points: train.len(),
        evaluation_points: pairs.len(),
        k,
        relative_rmse: all.relative_rmse,
        p95_absolute_relative_error: all.p95_absolute_relative_error,
        ambiguity,
        early,
        middle,
        late,
        sufficient: all.relative_rmse <= RMSE_LIMIT
            && all.p95_absolute_relative_error <= P95_LIMIT
            && ambiguity <= AMBIGUITY_LIMIT,
    }
}

fn settled_distance(mesh: &MaterialMesh, settled: &MaterialMesh) -> f64 {
    ((mesh.interior.a - settled.interior.a).powi(2)
        + (mesh.interior.r - settled.interior.r).powi(2))
    .sqrt()
}

fn run_oracle(deprived: &MaterialMesh, settled: &MaterialMesh) -> OracleSummary {
    let mut mesh = deprived.clone();
    let p = reaction_params(&mesh);
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED,
        M_SELECTED,
    );
    let transport = TransportParams::default();
    let initial = snap(&mesh, 0);
    let initial_distance = settled_distance(&mesh, settled);
    let mut n_consumed = 0.0;
    let mut f_consumed = 0.0;
    let mut fractions = Vec::with_capacity(WINDOW);
    let mut accelerated = 0;
    let mut max_residual = 0.0_f64;
    let mut hashes = vec![stable_json_hash(&initial).unwrap()];
    for step in 1..=WINDOW {
        let uptake = region.uptake(&mut mesh, &transport, DT);
        assert!(uptake.conservation_error.abs() <= MASS_TOL);
        let root = audit_root(&mesh, &p);
        assert_eq!(root.status, "FINITE_ZERO_DRIFT_ROOT");
        let extent = root.s_zero.unwrap();
        fractions.push(extent / root.saturated_source.max(SOURCE_EPS));
        let (ledger, eval) = execute_extent(&mut mesh, &p, extent);
        n_consumed += ledger.n_consumed;
        f_consumed += ledger.f_consumed;
        accelerated += usize::from(eval.accelerated_decay);
        max_residual = max_residual.max(eval.accounting.conservation_residual.abs());
        hashes.push(stable_json_hash(&snap(&mesh, step)).unwrap());
    }
    OracleSummary {
        initial,
        final_state: snap(&mesh, WINDOW),
        total_n_consumed: n_consumed,
        total_f_consumed: f_consumed,
        source_capacity_fraction: distribution(fractions),
        accelerated_decay_steps: accelerated,
        max_accounting_residual: max_residual,
        settled_distance_initial: initial_distance,
        settled_distance_final: settled_distance(&mesh, settled),
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    }
}

fn main() {
    let output = std::env::var_os("DCDEV020R7_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r7"));
    let r5_path = std::env::var_os("DCDEV020R5_EXTERNAL_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r5-statewise-ledger.json"));
    let dense_path = std::env::var_os("DCDEV020R7_EXTERNAL_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r7-on-policy-ledger.json"));
    let external_location = std::env::var("DCDEV020R7_EXTERNAL_LOCATION")
        .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
    let dense_sha256 =
        std::env::var("DCDEV020R7_LEDGER_SHA256").unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());
    let source_commit =
        std::env::var("DCDEV020R7_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let r5: Vec<R5Root> = serde_json::from_slice(&fs::read(&r5_path).unwrap()).unwrap();
    assert_eq!(r5.len(), 4_800);

    let settled = settle();
    let deprived = deprive(&settled);
    let (records, final_state, trajectory_hash, uptake_error) = run_r6(&deprived);
    if let Some(parent) = dense_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&dense_path, serde_json::to_vec(&records).unwrap()).unwrap();

    // R5 established that non-authoritative mechanics internals can create a
    // different byte hash across operating systems. Scientific parity is the
    // exact frozen path plus its explicit endpoint snapshot within tolerance;
    // the committed Windows realization hash is retained as a separate seal.
    let artifact_hash_match = trajectory_hash == ACCEPTED_R6_HASH;
    let parity = (final_state.e_stored - R6_FINAL_E).abs() <= MASS_TOL
        && (final_state.a - R6_FINAL_A).abs() <= MASS_TOL
        && (final_state.r - R6_FINAL_R).abs() <= MASS_TOL
        && (final_state.n - R6_FINAL_NF).abs() <= MASS_TOL
        && (final_state.f - R6_FINAL_NF).abs() <= MASS_TOL
        && (final_state.c - R6_FINAL_C).abs() <= MASS_TOL;
    let capacity = records
        .iter()
        .filter(|r| r.root_audit.status == "LOCAL_SOURCE_CAPACITY_INSUFFICIENT")
        .count();
    let nonmonotonic = records
        .iter()
        .filter(|r| r.root_audit.status == "LOCAL_SOURCE_RESPONSE_NONMONOTONIC")
        .count();
    let finite = records
        .iter()
        .filter(|r| r.root_audit.status == "FINITE_ZERO_DRIFT_ROOT")
        .count();
    let max_root_interval = records
        .iter()
        .filter_map(|r| r.root_audit.root_relative_interval)
        .fold(0.0_f64, f64::max);
    let max_root_residual = records
        .iter()
        .filter_map(|r| r.root_audit.root.as_ref())
        .map(|x| x.accounting.conservation_residual.abs())
        .fold(0.0_f64, f64::max);
    let valid_physics = capacity == 0
        && nonmonotonic == 0
        && finite == WINDOW
        && max_root_interval <= ROOT_REL_TOL
        && max_root_residual <= MASS_TOL;

    let r6_pairs: Vec<(f64, f64)> = records
        .iter()
        .map(|r| (r.r6_accepted_source, r.root_audit.s_zero.unwrap()))
        .collect();
    let r6_errors = errors_from_pairs(&r6_pairs);
    let r6_windows = WindowErrors {
        early: errors_from_pairs(&r6_pairs[0..160]),
        middle: errors_from_pairs(&r6_pairs[160..320]),
        late: errors_from_pairs(&r6_pairs[320..480]),
    };
    let summed_drift: f64 = records.iter().map(|r| r.r6_drift).sum();
    let observed_drift = final_state.e_stored - E_DEPRIVED;
    let drift_closure = summed_drift - observed_drift;

    let nf_support = support_report(&r5, &records, false);
    let nfa_support = support_report(&r5, &records, true);
    let nf = observer_report(&r5, &records, false);
    let nfa = observer_report(&r5, &records, true);
    let oracle = run_oracle(&deprived, &settled);
    let meaningful_oracle_gain = oracle.final_state.e_stored - oracle.initial.e_stored
        > R5_MATERIAL_STATE_FRACTION * oracle.initial.e_stored;
    let r6_tracks =
        r6_errors.relative_rmse <= RMSE_LIMIT && r6_errors.p95_absolute_relative_error <= P95_LIMIT;
    let systematic_undersupply = r6_errors.fraction_below_root > 0.5;

    let classification = if capacity > 0 {
        "DCDEV020R7_ON_POLICY_SOURCE_CAPACITY_INSUFFICIENT"
    } else if nonmonotonic > 0 {
        "DCDEV020R7_ON_POLICY_SOURCE_RESPONSE_NONMONOTONIC"
    } else if nf.sufficient && (!r6_tracks || systematic_undersupply) {
        "DCDEV020R7_POWER_LAW_FAMILY_MISSPECIFIED_ON_POLICY"
    } else if !nf.sufficient && nfa.sufficient {
        "DCDEV020R7_NFA_COORDINATE_REQUIRED_ON_POLICY"
    } else if !nf.sufficient && !nfa.sufficient {
        "DCDEV020R7_EXISTING_LOCAL_COORDINATES_INSUFFICIENT_ON_POLICY"
    } else {
        assert!(r6_tracks && nf.sufficient && !meaningful_oracle_gain);
        "DCDEV020R7_ZERO_DRIFT_MAINTENANCE_NOT_RESTORATION"
    };

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-020-R7", "accepted_r6_head":ACCEPTED_R6_HEAD,
            "clean_scientific_base":CLEAN_BASE, "source_commit":source_commit,
            "r5_ledger_sha256":R5_LEDGER_SHA256, "r5_external_location":R5_EXTERNAL_LOCATION,
            "r6":{"k_pl":K_PL,"p":POWER_P,"accepted_trajectory_hash":ACCEPTED_R6_HASH},
            "settle_steps":SETTLE_STEPS,"deprivation_steps":WINDOW,"feed_steps":WINDOW,
            "selected_patch_mass_each":M_SELECTED,"capture_boundary":"after passive uptake and before reaction execution",
            "root":{"shape_fractions":[0.0,0.25,0.5,0.75,1.0],"relative_interval_tolerance":ROOT_REL_TOL},
            "observer":{"training":["P0","P1","P2"],"holdout_reference":["P3","P4"],"k":K_NEIGHBORS,"unweighted":true,"training_min_max_scaling":true,"target":"S_zero/(q_c*area*dt)","limits":{"rmse":RMSE_LIMIT,"p95":P95_LIMIT,"ambiguity":AMBIGUITY_LIMIT}},
        "material_error_fraction":MATERIAL_ERROR,
        "meaningful_oracle_gain":{"fraction":R5_MATERIAL_STATE_FRACTION,"provenance":"frozen R5 material-state fraction"},
            "observer_only":true,"production_integration":false,"refit":false
        }),
    );
    write_json(
        &output,
        "summary.json",
        &json!({
        "r6":{"trajectory_parity":parity,"trajectory_hash":trajectory_hash,"committed_artifact_hash_match":artifact_hash_match,"final_state":final_state,"uptake_conservation_error":uptake_error},
            "roots":{"states":records.len(),"finite":finite,"capacity_insufficient":capacity,"nonmonotonic":nonmonotonic,"max_relative_interval":max_root_interval,"max_accounting_residual":max_root_residual,"s_zero_over_saturated":distribution(records.iter().filter_map(|r|r.root_audit.s_zero_over_saturated).collect())},
            "r6_versus_root":{"all":r6_errors,"windows":r6_windows,"summed_local_drift":summed_drift,"observed_endpoint_drift":observed_drift,"drift_accounting_closure":drift_closure},
            "support":{"nf":nf_support,"nfa":nfa_support},
            "frozen_observer":{"nf":nf,"nfa":nfa},
            "oracle":oracle,"meaningful_oracle_gain":meaningful_oracle_gain
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification":classification,"trajectory_parity":parity,"physics_valid":valid_physics,
            "drift_accounting_valid":drift_closure.abs() <= MASS_TOL,"r6_tracks_frozen_limits":r6_tracks,
            "systematic_undersupply":systematic_undersupply,"nf_on_policy_sufficient":nf.sufficient,
            "nfa_on_policy_sufficient":nfa.sufficient,"oracle_restorative_gain":meaningful_oracle_gain,
            "r6_accepted_negative":"DCDEV020R6_ACCEPTED_NEGATIVE","route_disposition":"NF_POWER_LAW_RESTORATION_ROUTE_CLOSED",
            "nf_coordinate_closed":false,"production_chemistry_changed":false,"production_behavior_changed":false,
            "implementation_authorized":false,"dc_dev_021_authorized":false,"next_execution_started":false
        }),
    );
    write_json(
        &output,
        "external_evidence_manifest.json",
        &json!({
            "r5_dense_input":{"sha256":R5_LEDGER_SHA256,"location":R5_EXTERNAL_LOCATION},
        "r7_dense_output":{"local_path":dense_path,"external_location":external_location,"sha256":dense_sha256},
            "git_evidence":"compact summaries only"
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "external_values_or_models_imported":false,
            "sources":[
                {"citation":"Piroddi 2008, Simulation error minimisation methods for NARX model identification","url":"https://www.inderscience.com/info/inarticle.php?artid=20548","disposition":"ADAPTABLE_VALIDATION_METHOD","use":"free-running validation rationale only"},
                {"citation":"Ross, Gordon, and Bagnell 2011, A Reduction of Imitation Learning and Structured Prediction to No-Regret Online Learning","url":"https://proceedings.mlr.press/v15/ross11a","disposition":"REFERENCE_ONLY","use":"on-policy distribution-shift analogy only"}
            ]
        }),
    );

    assert!(parity);
    assert!(valid_physics);
    assert!(drift_closure.abs() <= MASS_TOL);
    println!("DCDEV020R7_ON_POLICY_ZERO_DRIFT_AUDIT_COMPLETE");
    println!("classification={classification}");
    println!("NEXT_EXECUTION_STARTED:false");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r6_power_law_is_frozen_and_symmetric() {
        let mut mesh = seed();
        mesh.interior.n = 0.2;
        mesh.interior.f = 0.8;
        let p = reaction_params(&mesh);
        let nf = r6_requested(&mesh, &p);
        mesh.interior.n = 0.8;
        mesh.interior.f = 0.2;
        assert!((nf - r6_requested(&mesh, &p)).abs() <= 1e-15);
    }

    #[test]
    fn distribution_quantiles_are_deterministic() {
        let d = distribution(vec![4.0, 1.0, 3.0, 2.0]);
        assert_eq!(d.min, Some(1.0));
        assert_eq!(d.median, Some(2.5));
        assert_eq!(d.max, Some(4.0));
    }
}
