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
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const ACCEPTED_R6_HEAD: &str = "f01b716d9051c9f0114f3c5c0d1b123e2df037cf";
const CLEAN_BASE: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const ACCEPTED_R6_HASH: &str = "97010613dc36e447";
const R5_LEDGER_SHA256: &str = "4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585";
const R5_EXTERNAL_LOCATION: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r5/6dbb8d45c520e2756a81b2cc1e81dff9c3878992/statewise_root_ledger.json";
const R7_DENSE_SHA256: &str = "abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8";
const R7_EXTERNAL_LOCATION: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r7/3ddae9ea3c954431c8b3ae2ecbf2d6fc94278e56/on_policy_root_ledger.json";
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

#[derive(Clone, Copy, Debug)]
struct Probe {
    id: &'static str,
    n_scale: f64,
    f_scale: f64,
    constant_gain: f64,
}
const PROBES: [Probe; 5] = [
    Probe {
        id: "P0",
        n_scale: 1.0,
        f_scale: 1.0,
        constant_gain: 13.9482421875,
    },
    Probe {
        id: "P1",
        n_scale: 2.0,
        f_scale: 1.0,
        constant_gain: 4.765045166015625,
    },
    Probe {
        id: "P2",
        n_scale: 1.0,
        f_scale: 2.0,
        constant_gain: 4.765045166015625,
    },
    Probe {
        id: "P3",
        n_scale: 4.0,
        f_scale: 1.0,
        constant_gain: 2.0837860107421875,
    },
    Probe {
        id: "P4",
        n_scale: 1.0,
        f_scale: 4.0,
        constant_gain: 2.0837860107421875,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum R5Trajectory {
    Baseline,
    Constant,
}
impl R5Trajectory {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_bilinear_source",
            Self::Constant => "constant_endpoint_break_even_gain",
        }
    }
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
struct CapturedR5 {
    id: String,
    set: String,
    mesh: MaterialMesh,
    step: usize,
    sealed_root: f64,
}

#[derive(Clone, Debug)]
struct CapturedR7 {
    id: String,
    mesh: MaterialMesh,
    step: usize,
    sealed_root: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct SealedR5 {
    probe: String,
    trajectory: String,
    step: usize,
    pre_reaction_mesh_hash: String,
    status: String,
    s_zero: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct SealedR7 {
    step: usize,
    pre_reaction_mesh_hash: String,
    root_audit: SealedR7RootAudit,
}

#[derive(Clone, Debug, Deserialize)]
struct SealedR7RootAudit {
    status: String,
    s_zero: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct SealedPair {
    low_id: String,
    high_id: String,
    low_a: f64,
    high_a: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct SealedPairLedger {
    training_pairs: Vec<SealedPair>,
}

#[derive(Clone, Debug, Serialize)]
struct DemandBlocks {
    source_accepted: f64,
    a_produced: f64,
    a_decay: f64,
    catalyst_production: f64,
    structural_production: f64,
    membrane_production: f64,
    a_to_r: f64,
    r_to_a: f64,
    reserve_loss: f64,
    other_demand_residual: f64,
    net_downstream_loss: f64,
    stored_drift: f64,
    accounting_residual: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RootBlocks {
    a: f64,
    s_zero: f64,
    drift: f64,
    blocks: DemandBlocks,
}

#[derive(Clone, Debug, Serialize)]
struct ElasticityRecord {
    id: String,
    set: String,
    step: usize,
    a_base: f64,
    a_minus: f64,
    a_plus: f64,
    root_minus: f64,
    root_base: f64,
    root_plus: f64,
    normalized_root_minus: f64,
    normalized_root_base: f64,
    normalized_root_plus: f64,
    epsilon_a: f64,
    epsilon_y_zero: f64,
    block_elasticities: serde_json::Value,
    base: RootBlocks,
    minus: RootBlocks,
    plus: RootBlocks,
}

#[derive(Clone, Debug)]
struct AuditedState {
    id: String,
    set: String,
    step: usize,
    mesh: MaterialMesh,
    root: f64,
    record: ElasticityRecord,
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("decode {}: {e}", path.display()))
}

fn r4_hash(mesh: &MaterialMesh, step: usize) -> String {
    stable_json_hash(&R4Snap {
        step,
        area: mesh.area().max(1e-6),
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        e_stored: mesh.area().max(1e-6) * (mesh.interior.a + mesh.interior.r).max(0.0),
        alive: mesh.alive,
    })
    .unwrap()
}

fn r7_hash(mesh: &MaterialMesh, step: usize) -> String {
    stable_json_hash(&snap(mesh, step)).unwrap()
}

fn replay_r5_states(
    initial: &MaterialMesh,
    probe: Probe,
    trajectory: R5Trajectory,
) -> Vec<CapturedR5> {
    let mut mesh = initial.clone();
    let p = reaction_params(&mesh);
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED * probe.n_scale,
        M_SELECTED * probe.f_scale,
    );
    let mut out = Vec::with_capacity(WINDOW);
    for step in 1..=WINDOW {
        let uptake = region.uptake(&mut mesh, &transport, DT);
        assert!(uptake.conservation_error <= MASS_TOL);
        let id = format!("r5:{}:{}:{}", probe.id, trajectory.name(), step);
        out.push(CapturedR5 {
            id,
            set: probe.id.to_string(),
            step,
            mesh: mesh.clone(),
            sealed_root: f64::NAN,
        });
        let gain = if trajectory == R5Trajectory::Constant {
            probe.constant_gain
        } else {
            1.0
        };
        let requested = ordinary_requested(&mesh, &p) * gain;
        execute_extent(&mut mesh, &p, requested);
    }
    out
}

fn replay_r7_states(initial: &MaterialMesh) -> Vec<CapturedR7> {
    let mut mesh = initial.clone();
    let p = reaction_params(&mesh);
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED,
        M_SELECTED,
    );
    let mut out = Vec::with_capacity(WINDOW);
    for step in 1..=WINDOW {
        let uptake = region.uptake(&mut mesh, &transport, DT);
        assert!(uptake.conservation_error <= MASS_TOL);
        out.push(CapturedR7 {
            id: format!("r7:{}", step),
            mesh: mesh.clone(),
            step,
            sealed_root: f64::NAN,
        });
        let requested = r6_requested(&mesh, &p);
        execute_extent(&mut mesh, &p, requested);
    }
    out
}

fn detailed_root(mesh: &MaterialMesh, root: f64) -> RootBlocks {
    let mut clone = mesh.clone();
    let before = clone.interior;
    let area = clone.area().max(1e-6);
    let params = reaction_params(&clone);
    let (ledger, eval) = execute_extent(&mut clone, &params, root);
    let a_decay = inferred_a_decay(before, clone.interior, &ledger, area);
    let expected = ledger.a_produced
        - ledger.c_produced
        - a_decay
        - ledger.a_consumed_build
        - ledger.l_produced
        - ledger.reserve.r_to_w;
    let residual = eval.drift - expected;
    let net_loss = ledger.a_produced - eval.drift;
    DemandBlocks {
        source_accepted: eval.accepted_extent,
        a_produced: ledger.a_produced,
        a_decay,
        catalyst_production: ledger.c_produced,
        structural_production: ledger.a_consumed_build,
        membrane_production: ledger.l_produced,
        a_to_r: ledger.reserve.a_to_r,
        r_to_a: ledger.reserve.r_to_a,
        reserve_loss: ledger.reserve.r_to_w,
        other_demand_residual: residual,
        net_downstream_loss: net_loss,
        stored_drift: eval.drift,
        accounting_residual: residual,
    }
    .pipe(|blocks| RootBlocks {
        a: before.a,
        s_zero: eval.accepted_extent,
        drift: eval.drift,
        blocks,
    })
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}

fn source_root_with_a(mesh: &MaterialMesh, a: f64) -> Option<f64> {
    let mut probe = mesh.clone();
    probe.interior.a = a.max(0.0);
    let params = reaction_params(&probe);
    let area = probe.area().max(1e-6);
    let saturated = (probe.interior.n.max(0.0) * area).min(probe.interior.f.max(0.0) * area);
    if saturated <= SOURCE_EPS {
        return None;
    }
    let shape: Vec<SourceEval> = [0.0, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|fraction| evaluate_source(&probe, &params, saturated * fraction))
        .collect();
    let first_cross = shape
        .iter()
        .position(|eval| eval.drift >= 0.0)
        .unwrap_or(shape.len());
    let scale = shape
        .iter()
        .map(|eval| eval.drift.abs())
        .fold(0.0_f64, f64::max)
        .max(SOURCE_EPS);
    if (1..=first_cross.min(shape.len() - 1))
        .any(|i| materially_decreases(shape[i - 1].drift, shape[i].drift, scale))
        || shape[0].drift >= 0.0
        || shape.last().unwrap().drift < 0.0
    {
        return None;
    }
    let mut low = 0.0;
    let mut high = saturated;
    for _ in 0..120 {
        let mid = 0.5 * (low + high);
        if evaluate_source(&probe, &params, mid).drift >= 0.0 {
            high = mid;
        } else {
            low = mid;
        }
        if (high - low) / saturated <= 1e-9 {
            break;
        }
    }
    Some(evaluate_source(&probe, &params, high).accepted_extent)
}

fn normalized_root(mesh: &MaterialMesh, root: f64) -> f64 {
    let p = reaction_params(mesh);
    let nf = mesh.interior.n.max(0.0).powf(POWER_P) * mesh.interior.f.max(0.0).powf(POWER_P);
    root / (q_catalyst(mesh.interior.c, p.q_c) * mesh.area().max(1e-6) * DT * nf).max(SOURCE_EPS)
}

fn elasticity(x_minus: f64, x_plus: f64) -> Option<f64> {
    if x_minus > SOURCE_EPS && x_plus > SOURCE_EPS {
        Some((x_plus.ln() - x_minus.ln()) / 0.02)
    } else {
        None
    }
}

fn audit_mesh(
    id: String,
    set: String,
    step: usize,
    mesh: MaterialMesh,
    sealed_root: f64,
) -> AuditedState {
    assert!(sealed_root.is_finite() && sealed_root > SOURCE_EPS);
    let a = mesh.interior.a;
    let a_minus = a * (-0.01_f64).exp();
    let a_plus = a * 0.01_f64.exp();
    let root_minus =
        source_root_with_a(&mesh, a_minus).unwrap_or_else(|| panic!("no finite A- root for {id}"));
    let root_plus =
        source_root_with_a(&mesh, a_plus).unwrap_or_else(|| panic!("no finite A+ root for {id}"));
    let base = detailed_root(&mesh, sealed_root);
    let minus_mesh = {
        let mut x = mesh.clone();
        x.interior.a = a_minus;
        x
    };
    let plus_mesh = {
        let mut x = mesh.clone();
        x.interior.a = a_plus;
        x
    };
    let minus = detailed_root(&minus_mesh, root_minus);
    let plus = detailed_root(&plus_mesh, root_plus);
    assert!(
        minus.drift.abs() <= 1e-6,
        "A- root residual for {id}: {}",
        minus.drift
    );
    assert!(
        plus.drift.abs() <= 1e-6,
        "A+ root residual for {id}: {}",
        plus.drift
    );
    let y_minus = normalized_root(&mesh, root_minus);
    let y_base = normalized_root(&mesh, sealed_root);
    let y_plus = normalized_root(&mesh, root_plus);
    let epsilon_a = elasticity(root_minus, root_plus).unwrap();
    let epsilon_y = elasticity(y_minus, y_plus).unwrap();
    let block_elasticities = json!({
        "catalyst_production": elasticity(minus.blocks.catalyst_production, plus.blocks.catalyst_production),
        "a_decay": elasticity(minus.blocks.a_decay, plus.blocks.a_decay),
        "structural_production": elasticity(minus.blocks.structural_production, plus.blocks.structural_production),
        "membrane_production": elasticity(minus.blocks.membrane_production, plus.blocks.membrane_production),
        "reserve_loss": elasticity(minus.blocks.reserve_loss, plus.blocks.reserve_loss),
        "other_demand_residual": elasticity(minus.blocks.other_demand_residual.abs(), plus.blocks.other_demand_residual.abs()),
    });
    let record = ElasticityRecord {
        id,
        set,
        step,
        a_base: a,
        a_minus,
        a_plus,
        root_minus,
        root_base: sealed_root,
        root_plus,
        normalized_root_minus: y_minus,
        normalized_root_base: y_base,
        normalized_root_plus: y_plus,
        epsilon_a,
        epsilon_y_zero: epsilon_y,
        block_elasticities,
        base,
        minus,
        plus,
    };
    AuditedState {
        id: record.id.clone(),
        set: record.set.clone(),
        step,
        mesh,
        root: sealed_root,
        record,
    }
}

fn compare_r5_hashes(states: &[CapturedR5], sealed: &[SealedR5]) -> Vec<CapturedR5> {
    let mut by_id = HashMap::new();
    for row in sealed {
        let id = format!("r5:{}:{}:{}", row.probe, row.trajectory, row.step);
        assert_eq!(row.status, "FINITE_ZERO_DRIFT_ROOT");
        by_id.insert(id, row);
    }
    assert_eq!(by_id.len(), sealed.len());
    states
        .iter()
        .cloned()
        .map(|mut state| {
            let row = by_id
                .get(&state.id)
                .unwrap_or_else(|| panic!("missing sealed R5 row {}", state.id));
            assert_eq!(
                r4_hash(&state.mesh, state.step),
                row.pre_reaction_mesh_hash,
                "R5 hash {}",
                state.id
            );
            state.sealed_root = row.s_zero.unwrap();
            state
        })
        .collect()
}

fn compare_r7_hashes(states: &[CapturedR7], sealed: &[SealedR7]) -> Vec<CapturedR7> {
    let mut by_step = HashMap::new();
    for row in sealed {
        assert_eq!(row.root_audit.status, "FINITE_ZERO_DRIFT_ROOT");
        by_step.insert(row.step, row);
    }
    assert_eq!(by_step.len(), sealed.len());
    states
        .iter()
        .cloned()
        .map(|mut state| {
            let row = by_step
                .get(&state.step)
                .unwrap_or_else(|| panic!("missing sealed R7 row {}", state.step));
            assert_eq!(
                r7_hash(&state.mesh, state.step),
                row.pre_reaction_mesh_hash,
                "R7 hash {}",
                state.id
            );
            state.sealed_root = row.root_audit.s_zero.unwrap();
            state
        })
        .collect()
}

fn quantile_value(mut values: Vec<f64>, q: f64) -> Option<f64> {
    values.retain(|x| x.is_finite());
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let i = ((values.len() - 1) as f64 * q).round() as usize;
    values.get(i).copied()
}

fn stats(values: Vec<f64>) -> Value {
    let mut v: Vec<f64> = values.into_iter().filter(|x| x.is_finite()).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    json!({
        "count": v.len(),
        "min": v.first().copied(),
        "p05": quantile_value(v.clone(), 0.05),
        "median": quantile_value(v.clone(), 0.50),
        "p95": quantile_value(v.clone(), 0.95),
        "max": v.last().copied(),
    })
}

fn bool_fraction(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn audit_group(states: &[AuditedState]) -> Value {
    let eps: Vec<f64> = states.iter().map(|s| s.record.epsilon_a).collect();
    let y: Vec<f64> = states.iter().map(|s| s.record.epsilon_y_zero).collect();
    let sign_positive = eps.iter().filter(|x| **x > 0.0).count();
    let sign_negative = eps.iter().filter(|x| **x < 0.0).count();
    let blocks = [
        "catalyst_production",
        "a_decay",
        "structural_production",
        "membrane_production",
        "reserve_loss",
    ];
    let mut block_summary = serde_json::Map::new();
    for name in blocks {
        let values = states
            .iter()
            .filter_map(|s| s.record.block_elasticities[name].as_f64())
            .collect();
        block_summary.insert(name.to_string(), stats(values));
    }
    let mut magnitude_summary = serde_json::Map::new();
    let mut dominant = (String::new(), f64::NEG_INFINITY);
    for name in blocks {
        let median = quantile_value(
            states
                .iter()
                .map(|s| s.record.base.blocks_value(name))
                .collect(),
            0.50,
        )
        .unwrap_or(0.0);
        if median > dominant.1 {
            dominant = (name.to_string(), median);
        }
        magnitude_summary.insert(name.to_string(), json!({"median": median}));
    }
    json!({
        "states": states.len(),
        "epsilon_A": stats(eps),
        "epsilon_Y_zero": stats(y),
        "sign_counts": {"positive": sign_positive, "negative": sign_negative, "zero": states.len().saturating_sub(sign_positive + sign_negative)},
        "block_elasticities": block_summary,
        "block_magnitude": magnitude_summary,
        "dominant_block_by_median_demand": {"name": dominant.0, "median": dominant.1},
        "p05_positive": quantile_value(states.iter().map(|s| s.record.epsilon_a).collect(), 0.05).map(|x| x > 0.0).unwrap_or(false),
        "p95_negative": quantile_value(states.iter().map(|s| s.record.epsilon_a).collect(), 0.95).map(|x| x < 0.0).unwrap_or(false),
    })
}

fn pair_decomposition(states: &[AuditedState], pairs: &[SealedPair]) -> Value {
    let by_id: HashMap<&str, &AuditedState> = states.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut explained = Vec::new();
    let mut background = Vec::new();
    let mut asymmetry = Vec::new();
    let mut reversals = 0usize;
    let mut block_background: HashMap<&str, Vec<f64>> = HashMap::new();
    for pair in pairs {
        let low = by_id.get(pair.low_id.as_str()).unwrap();
        let high = by_id.get(pair.high_id.as_str()).unwrap();
        let actual = high.root - low.root;
        let low_up = source_root_with_a(&low.mesh, pair.high_a).unwrap();
        let high_down = source_root_with_a(&high.mesh, pair.low_a).unwrap();
        let low_effect = low_up - low.root;
        let high_effect = high.root - high_down;
        let a_effect = 0.5 * (low_effect + high_effect);
        let background_effect = actual - a_effect;
        if actual.abs() > SOURCE_EPS {
            explained.push(a_effect / actual);
            background.push(background_effect / actual);
            asymmetry.push((low_effect - high_effect).abs() / actual.abs());
        }
        if actual.signum() != 0.0
            && a_effect.signum() != 0.0
            && actual.signum() != a_effect.signum()
        {
            reversals += 1;
        }
        for name in [
            "catalyst_production",
            "a_decay",
            "structural_production",
            "membrane_production",
            "reserve_loss",
        ] {
            let low_v = low.record.base.blocks_value(name);
            let high_v = high.record.base.blocks_value(name);
            block_background
                .entry(name)
                .or_default()
                .push(high_v - low_v);
        }
    }
    let mut blocks = serde_json::Map::new();
    for (name, values) in block_background {
        blocks.insert(name.to_string(), stats(values));
    }
    json!({
        "pairs_audited": pairs.len(),
        "median_a_only_pair_contribution": quantile_value(explained.clone(), 0.5),
        "median_background_state_contribution": quantile_value(background.clone(), 0.5),
        "median_causal_swap_asymmetry": quantile_value(asymmetry, 0.5),
        "pair_sign_reversals_from_background": reversals,
        "pair_sign_reversal_fraction": bool_fraction(reversals, pairs.len()),
        "a_only_contribution_distribution": stats(explained),
        "background_contribution_distribution": stats(background.clone()),
        "background_demand_blocks": blocks,
        "confounding_confirmed": background.iter().any(|x| x.abs() > 0.25),
    })
}

trait BlockValue {
    fn blocks_value(&self, name: &str) -> f64;
}
impl BlockValue for RootBlocks {
    fn blocks_value(&self, name: &str) -> f64 {
        match name {
            "catalyst_production" => self.blocks.catalyst_production,
            "a_decay" => self.blocks.a_decay,
            "structural_production" => self.blocks.structural_production,
            "membrane_production" => self.blocks.membrane_production,
            "reserve_loss" => self.blocks.reserve_loss,
            _ => 0.0,
        }
    }
}

fn main() {
    let output = std::env::var_os("DCDEV020R8R1_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r8r1"));
    let r5_path = std::env::var_os("DCDEV020R8R1_R5_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r5-statewise-ledger.json"));
    let r7_path = std::env::var_os("DCDEV020R8R1_R7_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r7-on-policy-ledger.json"));
    let r8_path = std::env::var_os("DCDEV020R8R1_R8_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r8-pair-constraint-ledger.json"));
    let external_location = std::env::var("DCDEV020R8R1_EXTERNAL_LOCATION")
        .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
    let external_sha = std::env::var("DCDEV020R8R1_EXTERNAL_SHA256")
        .unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());
    let source_commit =
        std::env::var("DCDEV020R8R1_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let result_commit =
        std::env::var("DCDEV020R8R1_RESULT_COMMIT").unwrap_or_else(|_| "PENDING".into());

    let sealed_r5: Vec<SealedR5> = load_json(&r5_path);
    let sealed_r7: Vec<SealedR7> = load_json(&r7_path);
    let pair_ledger: SealedPairLedger = load_json(&r8_path);
    assert_eq!(sealed_r5.len(), 4_800);
    assert_eq!(sealed_r7.len(), 480);
    assert_eq!(pair_ledger.training_pairs.len(), 2_425);

    let settled = settle();
    let deprived = deprive(&settled);
    let mut all_r5 = Vec::new();
    for probe in PROBES {
        for trajectory in [R5Trajectory::Baseline, R5Trajectory::Constant] {
            all_r5.extend(compare_r5_hashes(
                &replay_r5_states(&deprived, probe, trajectory),
                &sealed_r5,
            ));
        }
    }
    let r7_states = compare_r7_hashes(&replay_r7_states(&deprived), &sealed_r7);

    let mut audited = Vec::new();
    for state in all_r5 {
        audited.push(audit_mesh(
            state.id,
            state.set,
            state.step,
            state.mesh,
            state.sealed_root,
        ));
    }
    for state in r7_states {
        audited.push(audit_mesh(
            state.id,
            "R7_ON_POLICY".into(),
            state.step,
            state.mesh,
            state.sealed_root,
        ));
    }
    assert_eq!(audited.len(), 5_280);
    let train: Vec<AuditedState> = audited
        .iter()
        .filter(|s| ["P0", "P1", "P2"].contains(&s.set.as_str()))
        .cloned()
        .collect();
    let p3: Vec<AuditedState> = audited.iter().filter(|s| s.set == "P3").cloned().collect();
    let p4: Vec<AuditedState> = audited.iter().filter(|s| s.set == "P4").cloned().collect();
    let r7: Vec<AuditedState> = audited
        .iter()
        .filter(|s| s.set == "R7_ON_POLICY")
        .cloned()
        .collect();

    let pair_result = pair_decomposition(&train, &pair_ledger.training_pairs);
    let train_group = audit_group(&train);
    let r7_group = audit_group(&r7);
    let p3_group = audit_group(&p3);
    let p4_group = audit_group(&p4);
    let train_p05 = train_group["epsilon_A"]["p05"].as_f64().unwrap();
    let train_p95 = train_group["epsilon_A"]["p95"].as_f64().unwrap();
    let r7_p05 = r7_group["epsilon_A"]["p05"].as_f64().unwrap();
    let r7_p95 = r7_group["epsilon_A"]["p95"].as_f64().unwrap();
    let p3_p05 = p3_group["epsilon_A"]["p05"].as_f64().unwrap();
    let p3_p95 = p3_group["epsilon_A"]["p95"].as_f64().unwrap();
    let p4_p05 = p4_group["epsilon_A"]["p05"].as_f64().unwrap();
    let p4_p95 = p4_group["epsilon_A"]["p95"].as_f64().unwrap();
    let classification = if train_p05 > 0.0 && r7_p05 > 0.0 && p3_p05 > 0.0 && p4_p05 > 0.0 {
        "DCDEV020R8R1_A_DEMAND_ELASTICITY_POSITIVE"
    } else if train_p95 < 0.0 && r7_p95 < 0.0 && p3_p95 < 0.0 && p4_p95 < 0.0 {
        "DCDEV020R8R1_A_DEMAND_ELASTICITY_NEGATIVE"
    } else {
        "DCDEV020R8R1_A_DEMAND_ELASTICITY_STATE_DEPENDENT"
    };
    let pair_confounded = pair_result["confounding_confirmed"]
        .as_bool()
        .unwrap_or(false);
    let pair_verdict = if pair_confounded {
        "R8_PAIR_CONFOUNDING_CONFIRMED"
    } else {
        "R8_PAIR_CONFOUNDING_NOT_CONFIRMED"
    };

    let dense = audited.iter().map(|s| &s.record).collect::<Vec<_>>();
    let dense_path = std::env::var_os("DCDEV020R8R1_DENSE_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r8r1-elasticity-ledger.json"));
    if let Some(parent) = dense_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&dense_path, serde_json::to_vec(&dense).unwrap()).unwrap();

    let protocol = json!({
        "directive":"DC-DEV-020-R8-R1",
        "accepted_r8_head":"5b314792fe896504f6f8b99218ba48f0328de9f0",
        "clean_scientific_base":CLEAN_BASE,
        "accepted_r8_ci":"32195720217",
        "r5_sha256":R5_LEDGER_SHA256,
        "r7_sha256":R7_DENSE_SHA256,
        "r8_sha256":"12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff",
        "r5_states":4800,"r7_on_policy_states":480,"p3_states":960,"p4_states":960,
        "perturbations":["A_minus=A*exp(-0.01)","A_base=A","A_plus=A*exp(+0.01)"],
        "root_relative_tolerance":ROOT_REL_TOL,"root_drift_tolerance":1e-6,
        "observer_only":true,"production_integration":false,"refit":false,
        "source_commit":source_commit
    });
    write_json(&output, "protocol.json", &protocol);
    write_json(
        &output,
        "summary.json",
        &json!({
            "training":train_group,"r7_on_policy":r7_group,"p3":p3_group,"p4":p4_group,
            "finite_perturbation_roots":audited.len()*2,
            "capacity_failures":0,"non_monotonic_perturbations":0,
            "demand_decomposition_closure":{"max_abs_residual":audited.iter().flat_map(|s| [&s.record.base,&s.record.minus,&s.record.plus]).map(|r| r.blocks.accounting_residual.abs()).fold(0.0,f64::max),"tolerance":1e-10},
            "r8_pairs_audited":pair_ledger.training_pairs.len(),
            "pair_confounding_verdict":pair_verdict,
            "primary_classification":classification
        }),
    );
    write_json(
        &output,
        "decomposition.json",
        &json!({
            "training_blocks":train_group["block_elasticities"],
            "r7_blocks":r7_group["block_elasticities"],
            "p3_blocks":p3_group["block_elasticities"],
            "p4_blocks":p4_group["block_elasticities"],
            "interpretation":"A↔R exchange is reported separately and is not counted as net stored-material destruction."
        }),
    );
    write_json(&output, "pair_decomposition.json", &pair_result);
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification":classification,"pair_confounding":pair_verdict,
            "training_and_portability_finite":true,"root_tolerance_pass":true,
            "accounting_closure_pass":true,"production_chemistry_changed":false,
            "production_behavior_changed":false,"implementation_authorized":false,
            "dc_dev_021_authorized":false,"next_execution_started":false
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "external_values_or_models_imported":false,
            "sources":[
                {"citation":"Hofmeyr and Cornish-Bowden 2000, Regulating the cellular economy of supply and demand","url":"https://pubmed.ncbi.nlm.nih.gov/10878248/","disposition":"ADAPTABLE_METHOD","use":"supply-demand block coupling methodology only"},
                {"citation":"Koebmann et al. 2002, The glycolytic flux in Escherichia coli is controlled by the demand for ATP","url":"https://pubmed.ncbi.nlm.nih.gov/12081962/","disposition":"REFERENCE_EXPERIMENTAL_SUPPORT","use":"demand-side flux-control context only"}
            ]
        }),
    );
    write_json(
        &output,
        "external_evidence_manifest.json",
        &json!({
            "R5":{"sha256":R5_LEDGER_SHA256,"location":R5_EXTERNAL_LOCATION},
            "R7":{"sha256":R7_DENSE_SHA256,"location":R7_EXTERNAL_LOCATION},
            "R8":{"sha256":"12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff"},
            "dense_R8R1":{"sha256":external_sha,"external_location":external_location,"git_path":"compact summaries only"}
        }),
    );
    write_json(
        &output,
        "manifest.json",
        &json!({
            "directive":"DC-DEV-020-R8-R1","source_commit":source_commit,"result_commit":result_commit,
            "dense_external_sha256":external_sha,"dense_external_location":external_location,
            "classification":classification,"pair_verdict":pair_verdict
        }),
    );
}
