// DC-DEV-020-R8-R3 observer-only catalyst-reserve horizon audit.
// The candidate is fitted only to the accepted R5 statewise zero-drift roots.
// It is then executed counterfactually through the frozen chemistry. Production
// chemistry, parameters, transport, resources, mechanics, and behavior remain
// unchanged.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{q_catalyst, reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ACCEPTED_R5_HEAD: &str = "d215cfc00ce70517e25fa7c3b51b13d85d9ce521";
const CLEAN_BASE: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const R5_LEDGER_SHA256: &str = "4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585";
const R5_EXTERNAL_LOCATION: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r5/6dbb8d45c520e2756a81b2cc1e81dff9c3878992/statewise_root_ledger.json";
const SETTLE_STEPS: usize = 5_000;
const WINDOW: usize = 480;
const SUSTAINED_STEPS: usize = 8_000;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const M_SELECTED: f64 = 19.878372106390554;
const SUSTAINED_NF: f64 = 0.1476710565778127;
const DT: f64 = 0.02;
const E_DEPRIVED: f64 = 60.82781514212436;
const E_TARGET: f64 = 77.91027880846893;
const REL_RMSE_LIMIT: f64 = 0.15;
const P95_LIMIT: f64 = 0.30;
const MASS_TOL: f64 = 1e-10;
const SOURCE_EPS: f64 = 1e-12;

#[derive(Clone, Debug, Deserialize)]
struct R5Root {
    probe: String,
    trajectory: String,
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    q_c: f64,
    status: String,
    s_zero: Option<f64>,
    saturated_source: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct PowerLaw {
    k_pl: f64,
    p: f64,
    g_h: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ResidualStructure {
    correlation_n: f64,
    correlation_f: f64,
    correlation_nf: f64,
    correlation_time: f64,
    mean_relative_error_by_probe: BTreeMap<String, f64>,
    mean_relative_error_by_trajectory: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Serialize)]
struct FitReport {
    model: PowerLaw,
    points: usize,
    relative_rmse: f64,
    p95_absolute_relative_error: f64,
    residuals: ResidualStructure,
}

#[derive(Clone, Debug, Serialize)]
struct PredictionReport {
    probes: Vec<String>,
    points: usize,
    relative_rmse: f64,
    p95_absolute_relative_error: f64,
    predicted_capacity_violations: usize,
    clipping_fraction: f64,
    residuals: ResidualStructure,
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum SourceLaw {
    Baseline,
    PowerLaw(PowerLaw),
    Saturated,
}

impl SourceLaw {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline_bilinear_source",
            Self::PowerLaw(_) => "r6_nf_power_law_source",
            Self::Saturated => "source_saturated_upper_bound",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct FluxTotals {
    n_delivered: f64,
    f_delivered: f64,
    n_consumed: f64,
    f_consumed: f64,
    a_produced: f64,
    a_decay: f64,
    catalyst_a_consumption: f64,
    structural_a_consumption: f64,
    membrane_a_consumption: f64,
    reserve_a_to_r: f64,
    reserve_r_to_a: f64,
    reserve_r_to_w: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ArmSummary {
    law: String,
    dose_scale: f64,
    steps: usize,
    initial: Snap,
    final_state: Snap,
    settled_distance_initial: f64,
    settled_distance_final: f64,
    a_toward_settled: bool,
    r_toward_settled: bool,
    alive_throughout: bool,
    finite_nonnegative: bool,
    max_resource_conservation_error: f64,
    max_stored_accounting_residual: f64,
    peak_e_stored: f64,
    accelerated_a_decay_steps: usize,
    clipping_steps: usize,
    capacity_violation_steps: usize,
    flux: FluxTotals,
    trajectory_hash: String,
}

#[derive(Clone, Debug, Serialize)]
struct SustainedSummary {
    law: String,
    steps: usize,
    initial: Snap,
    final_state: Snap,
    peak_e_stored: f64,
    final_quarter_min: f64,
    final_quarter_max: f64,
    final_quarter_slope: f64,
    final_quarter_accelerated_steps: usize,
    final_quarter_clipping_steps: usize,
    alive_throughout: bool,
    finite_nonnegative: bool,
    max_stored_accounting_residual: f64,
    trajectory_hash: String,
}

#[derive(Clone, Debug, Serialize)]
struct CycleSummary {
    cycle: usize,
    deprived: ArmSummary,
    fed: ArmSummary,
    recovery: f64,
    fed_final_r: f64,
}

#[derive(Clone, Copy, Debug)]
struct SourceStep {
    accelerated: bool,
    clipped: bool,
    capacity_violation: bool,
    accounting_residual: f64,
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
    let mut params = ReactionParams::default();
    params.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    params
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
    let params = reaction_params(&mesh);
    for _ in 0..WINDOW {
        reactions_step(&mut mesh, &params, DT, true, true);
    }
    assert!((snap(&mesh, WINDOW).e_stored - E_DEPRIVED).abs() <= 1e-10);
    mesh
}

fn finite_nonnegative(mesh: &MaterialMesh) -> bool {
    let values = [
        mesh.interior.c,
        mesh.interior.a,
        mesh.interior.n,
        mesh.interior.f,
        mesh.interior.r,
        mesh.free_l,
    ];
    mesh.alive && values.iter().all(|x| x.is_finite() && *x >= -SOURCE_EPS)
}

fn ordinary_requested(mesh: &MaterialMesh, params: &ReactionParams) -> f64 {
    params.k_act
        * q_catalyst(mesh.interior.c, params.q_c)
        * mesh.interior.n.max(0.0)
        * mesh.interior.f.max(0.0)
        * mesh.area().max(1e-6)
        * DT
}

fn power_requested_values(model: PowerLaw, q_c: f64, n: f64, f: f64, area: f64) -> f64 {
    if n <= 0.0 || f <= 0.0 {
        return 0.0;
    }
    q_c * model.g_h * model.k_pl * n.powf(model.p) * f.powf(model.p) * area * DT
}

fn requested(mesh: &MaterialMesh, params: &ReactionParams, law: SourceLaw) -> f64 {
    let area = mesh.area().max(1e-6);
    match law {
        SourceLaw::Baseline => ordinary_requested(mesh, params),
        SourceLaw::PowerLaw(model) => power_requested_values(
            model,
            q_catalyst(mesh.interior.c, params.q_c),
            mesh.interior.n,
            mesh.interior.f,
            area,
        ),
        SourceLaw::Saturated => {
            (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area)
        }
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

fn apply_source(
    mesh: &mut MaterialMesh,
    params: &ReactionParams,
    law: SourceLaw,
) -> (ReactionLedger, SourceStep) {
    let before = mesh.interior;
    let area = mesh.area().max(1e-6);
    let before_e = area * (before.a + before.r).max(0.0);
    let capacity = (before.n.max(0.0) * area).min(before.f.max(0.0) * area);
    let requested = requested(mesh, params, law);
    assert!(requested.is_finite() && requested >= 0.0);
    let ordinary = ordinary_requested(mesh, params);
    let gain = if requested <= SOURCE_EPS {
        0.0
    } else {
        requested / ordinary.max(SOURCE_EPS)
    };
    let mut effective = *params;
    effective.k_act = params.k_act * gain;
    let ledger = reactions_step(mesh, &effective, DT, true, true);
    let accepted = ledger.n_consumed;
    let after_e = area * (mesh.interior.a + mesh.interior.r).max(0.0);
    let decay = inferred_a_decay(before, mesh.interior, &ledger, area);
    let expected = ledger.a_produced
        - ledger.c_produced
        - decay
        - ledger.a_consumed_build
        - ledger.l_produced
        - ledger.reserve.r_to_w;
    let after_source_n = (before.n - accepted / area).max(0.0);
    let after_source_f = (before.f - accepted / area).max(0.0);
    (
        ledger,
        SourceStep {
            accelerated: after_source_n * after_source_f < 1e-8,
            clipped: requested > accepted + SOURCE_EPS,
            capacity_violation: requested > capacity + SOURCE_EPS,
            accounting_residual: (after_e - before_e) - expected,
        },
    )
}

fn settled_distance(mesh: &MaterialMesh, settled: &MaterialMesh) -> f64 {
    ((mesh.interior.a - settled.interior.a).powi(2)
        + (mesh.interior.r - settled.interior.r).powi(2))
    .sqrt()
}

fn fit_model(records: &[R5Root]) -> FitReport {
    let train: Vec<&R5Root> = records
        .iter()
        .filter(|r| matches!(r.probe.as_str(), "P0" | "P1" | "P2"))
        .filter(|r| r.status == "FINITE_ZERO_DRIFT_ROOT" && r.s_zero.unwrap_or(0.0) > SOURCE_EPS)
        .collect();
    let xs: Vec<f64> = train.iter().map(|r| r.n.ln() + r.f.ln()).collect();
    let ys: Vec<f64> = train
        .iter()
        .map(|r| (r.s_zero.unwrap() / (r.q_c * r.area * DT)).ln())
        .collect();
    let x_mean = mean(&xs);
    let y_mean = mean(&ys);
    let denominator = xs.iter().map(|x| (x - x_mean).powi(2)).sum::<f64>();
    let p = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| (x - x_mean) * (y - y_mean))
        .sum::<f64>()
        / denominator;
    let model = PowerLaw {
        k_pl: (y_mean - p * x_mean).exp(),
        p,
        g_h: 1.0,
    };
    let (rmse, p95, _, _, residuals) = prediction_metrics(&train, model);
    FitReport {
        model,
        points: train.len(),
        relative_rmse: rmse,
        p95_absolute_relative_error: p95,
        residuals,
    }
}

fn prediction_report(records: &[R5Root], probes: &[&str], model: PowerLaw) -> PredictionReport {
    let rows: Vec<&R5Root> = records
        .iter()
        .filter(|r| probes.contains(&r.probe.as_str()))
        .filter(|r| r.status == "FINITE_ZERO_DRIFT_ROOT" && r.s_zero.unwrap_or(0.0) > SOURCE_EPS)
        .collect();
    let (rmse, p95, violations, clipping_fraction, residuals) = prediction_metrics(&rows, model);
    PredictionReport {
        probes: probes.iter().map(|x| x.to_string()).collect(),
        points: rows.len(),
        relative_rmse: rmse,
        p95_absolute_relative_error: p95,
        predicted_capacity_violations: violations,
        clipping_fraction,
        residuals,
    }
}

fn prediction_metrics(
    rows: &[&R5Root],
    model: PowerLaw,
) -> (f64, f64, usize, f64, ResidualStructure) {
    let mut errors = Vec::with_capacity(rows.len());
    let mut violations = 0;
    let mut by_probe: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut by_trajectory: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for r in rows {
        let predicted = power_requested_values(model, r.q_c, r.n, r.f, r.area);
        let actual = r.s_zero.unwrap();
        let error = (predicted - actual) / actual.max(SOURCE_EPS);
        errors.push(error);
        violations += usize::from(predicted > r.saturated_source + SOURCE_EPS);
        by_probe.entry(r.probe.clone()).or_default().push(error);
        by_trajectory
            .entry(r.trajectory.clone())
            .or_default()
            .push(error);
    }
    let absolute = sorted(errors.iter().map(|x| x.abs()).collect());
    let rmse = (errors.iter().map(|x| x * x).sum::<f64>() / errors.len().max(1) as f64).sqrt();
    let p95 = quantile(&absolute, 0.95).unwrap_or(f64::INFINITY);
    let n_values: Vec<f64> = rows.iter().map(|r| r.n).collect();
    let f_values: Vec<f64> = rows.iter().map(|r| r.f).collect();
    let nf_values: Vec<f64> = rows.iter().map(|r| r.n * r.f).collect();
    let time_values: Vec<f64> = rows.iter().map(|r| r.step as f64).collect();
    let residuals = ResidualStructure {
        correlation_n: correlation(&n_values, &errors),
        correlation_f: correlation(&f_values, &errors),
        correlation_nf: correlation(&nf_values, &errors),
        correlation_time: correlation(&time_values, &errors),
        mean_relative_error_by_probe: group_means(by_probe),
        mean_relative_error_by_trajectory: group_means(by_trajectory),
    };
    (
        rmse,
        p95,
        violations,
        violations as f64 / rows.len().max(1) as f64,
        residuals,
    )
}

fn group_means(groups: BTreeMap<String, Vec<f64>>) -> BTreeMap<String, f64> {
    groups.into_iter().map(|(k, v)| (k, mean(&v))).collect()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let xm = mean(x);
    let ym = mean(y);
    let covariance = x
        .iter()
        .zip(y.iter())
        .map(|(a, b)| (a - xm) * (b - ym))
        .sum::<f64>();
    let xx = x.iter().map(|a| (a - xm).powi(2)).sum::<f64>();
    let yy = y.iter().map(|b| (b - ym).powi(2)).sum::<f64>();
    if xx <= SOURCE_EPS || yy <= SOURCE_EPS {
        0.0
    } else {
        covariance / (xx * yy).sqrt()
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

fn accumulate_flux(
    totals: &mut FluxTotals,
    before: LumpedChem,
    after: LumpedChem,
    ledger: &ReactionLedger,
    area: f64,
) {
    totals.n_consumed += ledger.n_consumed;
    totals.f_consumed += ledger.f_consumed;
    totals.a_produced += ledger.a_produced;
    totals.a_decay += inferred_a_decay(before, after, ledger, area);
    totals.catalyst_a_consumption += ledger.c_produced;
    totals.structural_a_consumption += ledger.a_consumed_build;
    totals.membrane_a_consumption += ledger.l_produced;
    totals.reserve_a_to_r += ledger.reserve.a_to_r;
    totals.reserve_r_to_a += ledger.reserve.r_to_a;
    totals.reserve_r_to_w += ledger.reserve.r_to_w;
}

fn run_window(
    mesh: &mut MaterialMesh,
    settled: &MaterialMesh,
    law: SourceLaw,
    steps: usize,
    patch_mass: Option<f64>,
    dose_scale: f64,
    start_step: usize,
) -> ArmSummary {
    let params = reaction_params(mesh);
    let mut region = patch_mass.map(|m| {
        FiniteSpatialResourceRegionV1::new(
            RESOURCE_CENTER,
            RESOURCE_RADIUS,
            m * dose_scale,
            m * dose_scale,
        )
    });
    let initial = snap(mesh, start_step);
    let initial_distance = settled_distance(mesh, settled);
    let mut flux = FluxTotals::default();
    let mut alive = true;
    let mut nonnegative = true;
    let mut max_resource_error = 0.0_f64;
    let mut max_accounting = 0.0_f64;
    let mut peak = initial.e_stored;
    let mut accelerated = 0;
    let mut clipping = 0;
    let mut capacity_violations = 0;
    let mut hashes = Vec::with_capacity(steps + 1);
    hashes.push(stable_json_hash(&initial).unwrap());
    for i in 0..steps {
        if let Some(resource) = region.as_mut() {
            let uptake = resource.uptake(mesh, &TransportParams::default(), DT);
            flux.n_delivered += uptake.n_world_loss;
            flux.f_delivered += uptake.f_world_loss;
            max_resource_error = max_resource_error.max(uptake.conservation_error.abs());
        }
        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let (ledger, source) = apply_source(mesh, &params, law);
        accumulate_flux(&mut flux, before, mesh.interior, &ledger, area);
        accelerated += usize::from(source.accelerated);
        clipping += usize::from(source.clipped);
        capacity_violations += usize::from(source.capacity_violation);
        max_accounting = max_accounting.max(source.accounting_residual.abs());
        alive &= mesh.alive;
        nonnegative &= finite_nonnegative(mesh);
        let state = snap(mesh, start_step + i + 1);
        peak = peak.max(state.e_stored);
        hashes.push(stable_json_hash(&state).unwrap());
    }
    let final_state = snap(mesh, start_step + steps);
    ArmSummary {
        law: law.name().into(),
        dose_scale,
        steps,
        initial,
        final_state,
        settled_distance_initial: initial_distance,
        settled_distance_final: settled_distance(mesh, settled),
        a_toward_settled: (settled.interior.a - mesh.interior.a).abs()
            < (settled.interior.a - initial.a).abs(),
        r_toward_settled: (settled.interior.r - mesh.interior.r).abs()
            < (settled.interior.r - initial.r).abs(),
        alive_throughout: alive,
        finite_nonnegative: nonnegative,
        max_resource_conservation_error: max_resource_error,
        max_stored_accounting_residual: max_accounting,
        peak_e_stored: peak,
        accelerated_a_decay_steps: accelerated,
        clipping_steps: clipping,
        capacity_violation_steps: capacity_violations,
        flux,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    }
}

fn run_sustained(mesh: &mut MaterialMesh, law: SourceLaw) -> SustainedSummary {
    let params = reaction_params(mesh);
    let initial = snap(mesh, 0);
    let mut values = Vec::with_capacity(SUSTAINED_STEPS);
    let mut hashes = Vec::with_capacity(SUSTAINED_STEPS);
    let mut alive = true;
    let mut nonnegative = true;
    let mut peak = initial.e_stored;
    let mut residual = 0.0_f64;
    let mut accelerated = Vec::with_capacity(SUSTAINED_STEPS);
    let mut clipping = Vec::with_capacity(SUSTAINED_STEPS);
    for step in 0..SUSTAINED_STEPS {
        mesh.interior.n = SUSTAINED_NF;
        mesh.interior.f = SUSTAINED_NF;
        let (_, source) = apply_source(mesh, &params, law);
        let state = snap(mesh, step + 1);
        peak = peak.max(state.e_stored);
        residual = residual.max(source.accounting_residual.abs());
        alive &= mesh.alive;
        nonnegative &= finite_nonnegative(mesh);
        values.push(state.e_stored);
        accelerated.push(source.accelerated);
        clipping.push(source.clipped);
        hashes.push(stable_json_hash(&state).unwrap());
    }
    let q4 = 3 * SUSTAINED_STEPS / 4;
    SustainedSummary {
        law: law.name().into(),
        steps: SUSTAINED_STEPS,
        initial,
        final_state: snap(mesh, SUSTAINED_STEPS),
        peak_e_stored: peak,
        final_quarter_min: values[q4..].iter().copied().fold(f64::INFINITY, f64::min),
        final_quarter_max: values[q4..]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max),
        final_quarter_slope: (values.last().unwrap() - values[q4]) / (values.len() - q4 - 1) as f64,
        final_quarter_accelerated_steps: accelerated[q4..].iter().filter(|x| **x).count(),
        final_quarter_clipping_steps: clipping[q4..].iter().filter(|x| **x).count(),
        alive_throughout: alive,
        finite_nonnegative: nonnegative,
        max_stored_accounting_residual: residual,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    }
}

const ACCEPTED_R8R1_HEAD_R8R2: &str = "d2c4f76a46f6baf7eab544847dd58c034adea156";
const ACCEPTED_R8R1_CI_R8R2: &str = "32203855916";
const ACCEPTED_R7_HEAD_R8R2: &str = "7d5f772f0db67b8d754d27c1182c933533f750fd";
const ACCEPTED_R8_HEAD_R8R2: &str = "f01b716d9051c9f0114f3c5c0d1b123e2df037cf";
const R7_DENSE_SHA256_R8R2: &str =
    "abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8";
const R7_EXTERNAL_LOCATION_R8R2: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r7/3ddae9ea3c954431c8b3ae2ecbf2d6fc94278e56/on_policy_root_ledger.json";
const R8_DENSE_SHA256_R8R2: &str =
    "12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff";
const R8_EXTERNAL_LOCATION_R8R2: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r8/6e2b03a7551409086c1a38d6cf5f62827fb91929/pair_constraint_ledger.json";
const R8R1_DENSE_SHA256_R8R2: &str =
    "f44e8f9fa441451ee40bcbfccac5f556131e4d26868868607e9507c29e7bcf90";
const R8R1_EXTERNAL_LOCATION_R8R2: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r8r1/d50037e53d041d8b06895553933c3b0a78c7a024/demand_elasticity_ledger.json";
const R6_FINAL_E_R8R2: f64 = 60.0620310117838;
const R6_FINAL_A_R8R2: f64 = 0.3423623895976825;
const R6_FINAL_R_R8R2: f64 = 0.5056416879564652;
const R6_FINAL_NF_R8R2: f64 = 0.10185789865759344;
const R6_FINAL_C_R8R2: f64 = 0.7722488011667238;
const R6_K_PL_R8R2: f64 = 0.017556661171593057;
const R6_POWER_P_R8R2: f64 = 0.0003277429681759396;
const SOURCE_EPS_R8R2: f64 = 1e-12;
const ROOT_DRIFT_TOL_R8R2: f64 = 1e-6;
const ROOT_REL_TOL_R8R2: f64 = 1e-9;
const CHECKPOINTS_R8R2: [usize; 12] = [1, 40, 80, 120, 160, 200, 240, 280, 320, 360, 400, 440];

#[derive(Clone, Debug, Serialize)]
struct RootPairR8R2 {
    step: usize,
    normal_root: Option<f64>,
    no_cprod_root: Option<f64>,
    r6_source: f64,
    delta_cprod: Option<f64>,
    r6_deficit: Option<f64>,
    normal_drift: f64,
    no_cprod_drift: f64,
    normal_valid: bool,
    no_cprod_valid: bool,
    capacity: f64,
    nonmonotonic: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
struct MaterialVectorR8R2 {
    ar_store: f64,
    catalyst: f64,
    structural: f64,
    membrane: f64,
    other_retained: f64,
    irreversible_w: f64,
    total_retained: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ShadowFrameR8R2 {
    step: usize,
    e_ar: f64,
    a: f64,
    r: f64,
    c: f64,
    q_c: f64,
    n: f64,
    f: f64,
    material: MaterialVectorR8R2,
    alive: bool,
}

#[derive(Clone, Debug, Serialize)]
struct PaybackResultR8R2 {
    context: String,
    checkpoint: usize,
    initial_a_cost: f64,
    initial_delta_c: f64,
    initial_delta_q_c: f64,
    payback_step: Option<usize>,
    no_payback: bool,
    final_delta_e_ar: f64,
    final_delta_c: f64,
    final_delta_q_c: f64,
    cumulative_extra_nf_a: f64,
    irreversible_loss_difference: f64,
    retained_material_difference: f64,
    retained_vector_difference: MaterialVectorR8R2,
    alive: bool,
    finite: bool,
    max_accounting_residual: f64,
    frames: Vec<ShadowFrameR8R2>,
}

#[derive(Clone, Debug)]
struct R7StateR8R2 {
    step: usize,
    mesh: MaterialMesh,
    region: FiniteSpatialResourceRegionV1,
    r6_source: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ShadowResultR8R2 {
    law: String,
    cprod_enabled: bool,
    initial: Snap,
    final_state: Snap,
    total_flux: FluxTotals,
    alive: bool,
    finite: bool,
    max_resource_error: f64,
    max_accounting_residual: f64,
    trajectory_hash: String,
    frames: Vec<ShadowFrameR8R2>,
}

#[derive(Clone, Debug, Serialize)]
struct RootStatsR8R2 {
    count: usize,
    valid_pairs: usize,
    normal_root_median: Option<f64>,
    no_cprod_root_median: Option<f64>,
    catalyst_production_burden_median: Option<f64>,
    burden_over_zero_median: Option<f64>,
    fraction_burden_at_least_r6_shortfall: f64,
    early_burden_median: Option<f64>,
    middle_burden_median: Option<f64>,
    late_burden_median: Option<f64>,
    early_burden_over_zero_median: Option<f64>,
    middle_burden_over_zero_median: Option<f64>,
    late_burden_over_zero_median: Option<f64>,
    capacity_failures: usize,
    nonmonotonic_failures: usize,
    invalid_pairs: usize,
}

fn material_vector_r8r2(mesh: &MaterialMesh) -> MaterialVectorR8R2 {
    let area = mesh.area().max(1e-6);
    let structural = mesh.total_structural_mass();
    let membrane = mesh.total_bound_membrane() + mesh.free_l;
    let ar_store = area * (mesh.interior.a + mesh.interior.r).max(0.0);
    let catalyst = area * mesh.interior.c.max(0.0);
    let other_retained = area * (mesh.interior.n + mesh.interior.f).max(0.0);
    let irreversible_w = area * mesh.interior.w.max(0.0);
    MaterialVectorR8R2 {
        ar_store,
        catalyst,
        structural,
        membrane,
        other_retained,
        irreversible_w,
        total_retained: ar_store + catalyst + structural + membrane + other_retained,
    }
}

fn subtract_material_r8r2(a: MaterialVectorR8R2, b: MaterialVectorR8R2) -> MaterialVectorR8R2 {
    MaterialVectorR8R2 {
        ar_store: a.ar_store - b.ar_store,
        catalyst: a.catalyst - b.catalyst,
        structural: a.structural - b.structural,
        membrane: a.membrane - b.membrane,
        other_retained: a.other_retained - b.other_retained,
        irreversible_w: a.irreversible_w - b.irreversible_w,
        total_retained: a.total_retained - b.total_retained,
    }
}

fn apply_source_mode_r8r2(
    mesh: &mut MaterialMesh,
    params: &ReactionParams,
    law: SourceLaw,
    cprod_enabled: bool,
) -> (ReactionLedger, SourceStep) {
    let requested_extent = requested(mesh, params, law);
    apply_source_extent_r8r2(mesh, params, requested_extent, cprod_enabled)
}

fn apply_source_extent_r8r2(
    mesh: &mut MaterialMesh,
    params: &ReactionParams,
    requested_extent: f64,
    cprod_enabled: bool,
) -> (ReactionLedger, SourceStep) {
    let before = mesh.interior;
    let area = mesh.area().max(1e-6);
    let before_e = area * (before.a + before.r).max(0.0);
    let capacity = (before.n.max(0.0) * area).min(before.f.max(0.0) * area);
    let ordinary = ordinary_requested(mesh, params);
    let gain = if requested_extent <= SOURCE_EPS_R8R2 {
        0.0
    } else {
        requested_extent / ordinary.max(SOURCE_EPS_R8R2)
    };
    let mut effective = *params;
    effective.k_act = params.k_act * gain;
    if !cprod_enabled {
        effective.k_c_prod = 0.0;
    }
    let ledger = reactions_step(mesh, &effective, DT, true, true);
    let accepted = ledger.n_consumed;
    let after_e = area * (mesh.interior.a + mesh.interior.r).max(0.0);
    let decay = inferred_a_decay(before, mesh.interior, &ledger, area);
    let expected = ledger.a_produced
        - ledger.c_produced
        - decay
        - ledger.a_consumed_build
        - ledger.l_produced
        - ledger.reserve.r_to_w;
    let after_source_n = (before.n - accepted / area).max(0.0);
    let after_source_f = (before.f - accepted / area).max(0.0);
    (
        ledger,
        SourceStep {
            accelerated: after_source_n * after_source_f < 1e-8,
            clipped: requested_extent > accepted + SOURCE_EPS_R8R2,
            capacity_violation: requested_extent > capacity + SOURCE_EPS_R8R2,
            accounting_residual: (after_e - before_e) - expected,
        },
    )
}

fn drift_r8r2(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    _law: SourceLaw,
    cprod: bool,
    extent: f64,
) -> (f64, SourceStep) {
    let mut shadow = mesh.clone();
    let before_e = snap(&shadow, 0).e_stored;
    let (_, source) = {
        let (ledger, source) = apply_source_extent_r8r2(&mut shadow, params, extent, cprod);
        (ledger, source)
    };
    let after_e = snap(&shadow, 1).e_stored;
    (after_e - before_e, source)
}

fn root_for_r8r2(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    law: SourceLaw,
    cprod: bool,
) -> (Option<f64>, f64, bool, bool, f64) {
    let area = mesh.area().max(1e-6);
    let capacity = (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area);
    if capacity <= SOURCE_EPS_R8R2 {
        let (drift, source) = drift_r8r2(mesh, params, law, cprod, 0.0);
        return (
            Some(0.0),
            drift,
            drift.abs() <= ROOT_DRIFT_TOL_R8R2,
            false,
            capacity,
        );
    }
    let fractions = [0.0, 0.25, 0.50, 0.75, 1.0];
    let samples: Vec<(f64, f64)> = fractions
        .iter()
        .map(|fraction| {
            let extent = capacity * fraction;
            let (drift, _) = drift_r8r2(mesh, params, law, cprod, extent);
            (extent, drift)
        })
        .collect();
    let first_cross = samples
        .iter()
        .position(|sample| sample.1 >= 0.0)
        .unwrap_or(samples.len());
    let scale = samples
        .iter()
        .map(|x| x.1.abs())
        .fold(0.0, f64::max)
        .max(SOURCE_EPS_R8R2);
    let nonmonotonic = samples[..=first_cross.min(samples.len() - 1)]
        .windows(2)
        .any(|w| w[1].1 < w[0].1 - 1e-10_f64.max(1e-6 * scale));
    let low_drift = samples[0].1;
    let high_drift = samples[4].1;
    if nonmonotonic || low_drift >= 0.0 {
        return (
            Some(0.0),
            low_drift,
            low_drift.abs() <= ROOT_DRIFT_TOL_R8R2,
            nonmonotonic,
            capacity,
        );
    }
    if high_drift < 0.0 {
        return (None, high_drift, false, nonmonotonic, capacity);
    }
    let mut low = 0.0;
    let mut high = capacity;
    for _ in 0..100 {
        let mid = (low + high) * 0.5;
        let (drift, _) = drift_r8r2(mesh, params, law, cprod, mid);
        if drift >= 0.0 {
            high = mid;
        } else {
            low = mid;
        }
        if (high - low) / capacity.max(SOURCE_EPS_R8R2) <= ROOT_REL_TOL_R8R2 {
            break;
        }
    }
    let (drift, _) = drift_r8r2(mesh, params, law, cprod, high);
    (
        Some(high),
        drift,
        drift.abs() <= ROOT_DRIFT_TOL_R8R2,
        false,
        capacity,
    )
}

fn r7_states_r8r2(deprived: &MaterialMesh) -> Vec<R7StateR8R2> {
    let mut mesh = deprived.clone();
    let params = reaction_params(&mesh);
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED,
        M_SELECTED,
    );
    let mut states = Vec::with_capacity(WINDOW);
    let mut uptake_error = 0.0_f64;
    for step in 1..=WINDOW {
        let uptake = region.uptake(&mut mesh, &TransportParams::default(), DT);
        uptake_error = uptake_error.max(uptake.conservation_error.abs());
        let r6_source = requested(
            &mesh,
            &params,
            SourceLaw::PowerLaw(PowerLaw {
                k_pl: R6_K_PL_R8R2,
                p: R6_POWER_P_R8R2,
                g_h: 1.0,
            }),
        );
        states.push(R7StateR8R2 {
            step,
            mesh: mesh.clone(),
            region: region.clone(),
            r6_source,
        });
        apply_source_mode_r8r2(
            &mut mesh,
            &params,
            SourceLaw::PowerLaw(PowerLaw {
                k_pl: R6_K_PL_R8R2,
                p: R6_POWER_P_R8R2,
                g_h: 1.0,
            }),
            true,
        );
    }
    assert!(uptake_error <= MASS_TOL);
    assert!((snap(&mesh, WINDOW).e_stored - R6_FINAL_E_R8R2).abs() <= MASS_TOL);
    assert!((mesh.interior.a - R6_FINAL_A_R8R2).abs() <= MASS_TOL);
    assert!((mesh.interior.r - R6_FINAL_R_R8R2).abs() <= MASS_TOL);
    assert!((mesh.interior.n - R6_FINAL_NF_R8R2).abs() <= MASS_TOL);
    assert!((mesh.interior.f - R6_FINAL_NF_R8R2).abs() <= MASS_TOL);
    assert!((mesh.interior.c - R6_FINAL_C_R8R2).abs() <= MASS_TOL);
    states
}

fn checkpoint_state_r8r2(
    deprived: &MaterialMesh,
    checkpoint: usize,
    law: SourceLaw,
) -> (MaterialMesh, FiniteSpatialResourceRegionV1) {
    let mut mesh = deprived.clone();
    let params = reaction_params(&mesh);
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED,
        M_SELECTED,
    );
    for step in 1..=checkpoint {
        region.uptake(&mut mesh, &TransportParams::default(), DT);
        if step < checkpoint {
            apply_source_mode_r8r2(&mut mesh, &params, law, true);
        }
    }
    (mesh, region)
}

fn frame_r8r2(step: usize, mesh: &MaterialMesh) -> ShadowFrameR8R2 {
    let chem = mesh.interior;
    ShadowFrameR8R2 {
        step,
        e_ar: snap(mesh, step).e_stored,
        a: chem.a,
        r: chem.r,
        c: chem.c,
        q_c: q_catalyst(chem.c, reaction_params(mesh).q_c),
        n: chem.n,
        f: chem.f,
        material: material_vector_r8r2(mesh),
        alive: mesh.alive,
    }
}

fn run_payback_r8r2(
    deprived: &MaterialMesh,
    checkpoint: usize,
    law: SourceLaw,
) -> PaybackResultR8R2 {
    let (base_mesh, base_region) = checkpoint_state_r8r2(deprived, checkpoint, law);
    let params = reaction_params(&base_mesh);
    let area = base_mesh.area().max(1e-6);
    let initial = frame_r8r2(checkpoint, &base_mesh);
    let initial_a_cost = params.k_c_prod * base_mesh.interior.a * DT * area;
    let mut invest = base_mesh.clone();
    let mut defer = base_mesh;
    let mut invest_region = base_region.clone();
    let mut defer_region = base_region;
    let mut frames = Vec::new();
    let mut payback_step = None;
    let mut max_accounting = 0.0_f64;
    let mut cumulative_extra_nf_a = 0.0_f64;
    let mut alive = true;
    let mut finite = true;
    for local in 0..=(WINDOW - checkpoint) {
        let step = checkpoint + local;
        if local > 0 {
            let params_i = reaction_params(&invest);
            let params_d = reaction_params(&defer);
            invest_region.uptake(&mut invest, &TransportParams::default(), DT);
            defer_region.uptake(&mut defer, &TransportParams::default(), DT);
            let before_i = invest.interior;
            let before_d = defer.interior;
            let (_, source_i) = apply_source_mode_r8r2(&mut invest, &params_i, law, true);
            let (_, source_d) = apply_source_mode_r8r2(&mut defer, &params_d, law, true);
            max_accounting = max_accounting.max(source_i.accounting_residual.abs());
            max_accounting = max_accounting.max(source_d.accounting_residual.abs());
            let area_i = invest.area().max(1e-6);
            let area_d = defer.area().max(1e-6);
            let nf_a_i = (before_i.n - invest.interior.n) * area_i
                + (before_i.f - invest.interior.f) * area_i
                + (invest.interior.a - before_i.a).max(0.0) * area_i;
            let nf_a_d = (before_d.n - defer.interior.n) * area_d
                + (before_d.f - defer.interior.f) * area_d
                + (defer.interior.a - before_d.a).max(0.0) * area_d;
            cumulative_extra_nf_a += nf_a_i - nf_a_d;
        } else {
            let params_i = reaction_params(&invest);
            let (_, source_i) = apply_source_mode_r8r2(&mut invest, &params_i, law, true);
            let params_d = reaction_params(&defer);
            let (_, source_d) = apply_source_mode_r8r2(&mut defer, &params_d, law, false);
            max_accounting = max_accounting.max(source_i.accounting_residual.abs());
            max_accounting = max_accounting.max(source_d.accounting_residual.abs());
        }
        alive &= invest.alive && defer.alive;
        finite &= finite_nonnegative(&invest) && finite_nonnegative(&defer);
        let fi = frame_r8r2(step + 1, &invest);
        let fd = frame_r8r2(step + 1, &defer);
        if local > 0 && payback_step.is_none() && fi.e_ar >= fd.e_ar {
            payback_step = Some(step);
        }
        frames.push(ShadowFrameR8R2 {
            step,
            e_ar: fi.e_ar - fd.e_ar,
            a: fi.a - fd.a,
            r: fi.r - fd.r,
            c: fi.c - fd.c,
            q_c: fi.q_c - fd.q_c,
            n: fi.n - fd.n,
            f: fi.f - fd.f,
            material: subtract_material_r8r2(fi.material, fd.material),
            alive: fi.alive && fd.alive,
        });
    }
    let last_frame = frames.last().unwrap();
    let final_material = last_frame.material;
    PaybackResultR8R2 {
        context: law.name().into(),
        checkpoint,
        initial_a_cost,
        initial_delta_c: frames.first().map(|frame| frame.c).unwrap_or(0.0),
        initial_delta_q_c: frames.first().map(|frame| frame.q_c).unwrap_or(0.0),
        payback_step,
        no_payback: payback_step.is_none(),
        final_delta_e_ar: last_frame.e_ar,
        final_delta_c: last_frame.c,
        final_delta_q_c: last_frame.q_c,
        cumulative_extra_nf_a,
        irreversible_loss_difference: final_material.irreversible_w,
        retained_material_difference: final_material.total_retained,
        retained_vector_difference: final_material,
        alive,
        finite,
        max_accounting_residual: max_accounting,
        frames,
    }
}

fn run_shadow_r8r2(
    deprived: &MaterialMesh,
    law: SourceLaw,
    cprod_enabled: bool,
) -> ShadowResultR8R2 {
    let mut mesh = deprived.clone();
    let params = reaction_params(&mesh);
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        M_SELECTED,
        M_SELECTED,
    );
    let initial = snap(&mesh, 0);
    let mut flux = FluxTotals::default();
    let mut frames = Vec::with_capacity(WINDOW);
    let mut hashes = vec![stable_json_hash(&initial).unwrap()];
    let mut alive = true;
    let mut finite = true;
    let mut max_resource_error = 0.0_f64;
    let mut max_accounting = 0.0_f64;
    for step in 1..=WINDOW {
        let uptake = region.uptake(&mut mesh, &TransportParams::default(), DT);
        max_resource_error = max_resource_error.max(uptake.conservation_error.abs());
        flux.n_delivered += uptake.n_world_loss;
        flux.f_delivered += uptake.f_world_loss;
        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let (ledger, source) = apply_source_mode_r8r2(&mut mesh, &params, law, cprod_enabled);
        accumulate_flux(&mut flux, before, mesh.interior, &ledger, area);
        max_accounting = max_accounting.max(source.accounting_residual.abs());
        alive &= mesh.alive;
        finite &= finite_nonnegative(&mesh);
        frames.push(frame_r8r2(step, &mesh));
        hashes.push(stable_json_hash(&snap(&mesh, step)).unwrap());
    }
    ShadowResultR8R2 {
        law: law.name().into(),
        cprod_enabled,
        initial,
        final_state: snap(&mesh, WINDOW),
        total_flux: flux,
        alive,
        finite,
        max_resource_error,
        max_accounting_residual: max_accounting,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
        frames,
    }
}

fn median_r8r2(mut values: Vec<f64>) -> Option<f64> {
    values.retain(|x| x.is_finite());
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    Some(values[values.len() / 2])
}

fn root_stats_r8r2(rows: &[RootPairR8R2]) -> RootStatsR8R2 {
    let valid: Vec<&RootPairR8R2> = rows
        .iter()
        .filter(|row| {
            row.normal_valid
                && row.no_cprod_valid
                && row.delta_cprod.is_some()
                && row.r6_deficit.is_some()
        })
        .collect();
    let burden: Vec<f64> = valid.iter().map(|row| row.delta_cprod.unwrap()).collect();
    let zero: Vec<f64> = valid.iter().map(|row| row.normal_root.unwrap()).collect();
    let deficits: Vec<f64> = valid.iter().map(|row| row.r6_deficit.unwrap()).collect();
    let ratio = valid
        .iter()
        .filter(|row| row.delta_cprod.unwrap() >= row.r6_deficit.unwrap())
        .count() as f64
        / valid.len().max(1) as f64;
    let window = |lo: usize, hi: usize| -> Vec<f64> {
        valid
            .iter()
            .filter(|row| row.step >= lo && row.step <= hi)
            .map(|row| row.delta_cprod.unwrap())
            .collect()
    };
    let ratio_window = |lo: usize, hi: usize| -> Option<f64> {
        median_r8r2(
            valid
                .iter()
                .filter(|row| row.step >= lo && row.step <= hi)
                .map(|row| row.delta_cprod.unwrap() / row.normal_root.unwrap().max(SOURCE_EPS_R8R2))
                .collect(),
        )
    };
    let _ = deficits;
    RootStatsR8R2 {
        count: rows.len(),
        valid_pairs: valid.len(),
        normal_root_median: median_r8r2(zero),
        no_cprod_root_median: median_r8r2(
            valid.iter().map(|row| row.no_cprod_root.unwrap()).collect(),
        ),
        catalyst_production_burden_median: median_r8r2(burden.clone()),
        burden_over_zero_median: median_r8r2(
            valid
                .iter()
                .map(|row| row.delta_cprod.unwrap() / row.normal_root.unwrap().max(SOURCE_EPS_R8R2))
                .collect(),
        ),
        fraction_burden_at_least_r6_shortfall: ratio,
        early_burden_median: median_r8r2(window(1, 160)),
        middle_burden_median: median_r8r2(window(161, 320)),
        late_burden_median: median_r8r2(window(321, 480)),
        early_burden_over_zero_median: ratio_window(1, 160),
        middle_burden_over_zero_median: ratio_window(161, 320),
        late_burden_over_zero_median: ratio_window(321, 480),
        capacity_failures: rows
            .iter()
            .filter(|row| row.normal_root.is_none() || row.no_cprod_root.is_none())
            .count(),
        nonmonotonic_failures: rows.iter().filter(|row| row.nonmonotonic).count(),
        invalid_pairs: rows.len() - valid.len(),
    }
}

fn payback_compact_r8r2(result: &PaybackResultR8R2) -> Value {
    json!({
        "context": result.context,
        "checkpoint": result.checkpoint,
        "initial_a_cost": result.initial_a_cost,
        "initial_delta_c": result.initial_delta_c,
        "initial_delta_q_c": result.initial_delta_q_c,
        "payback_step": result.payback_step,
        "no_payback": result.no_payback,
        "final_delta_e_ar": result.final_delta_e_ar,
        "final_delta_c": result.final_delta_c,
        "final_delta_q_c": result.final_delta_q_c,
        "cumulative_extra_nf_a": result.cumulative_extra_nf_a,
        "irreversible_loss_difference": result.irreversible_loss_difference,
        "retained_material_difference": result.retained_material_difference,
        "retained_vector_difference": result.retained_vector_difference,
        "alive": result.alive,
        "finite": result.finite,
        "max_accounting_residual": result.max_accounting_residual,
        "dense_frames_externalized": true
    })
}

fn shadow_compact_r8r2(result: &ShadowResultR8R2) -> Value {
    json!({
        "law": result.law,
        "cprod_enabled": result.cprod_enabled,
        "initial": result.initial,
        "final_state": result.final_state,
        "total_flux": result.total_flux,
        "alive": result.alive,
        "finite": result.finite,
        "max_resource_error": result.max_resource_error,
        "max_accounting_residual": result.max_accounting_residual,
        "trajectory_hash": result.trajectory_hash,
        "dense_frames_externalized": true
    })
}

fn legacy_r8r2_main() {
    let output = std::env::var_os("DCDEV020R8R2_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r8r2"));
    let dense_path = std::env::var_os("DCDEV020R8R2_DENSE_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| output.join("catalyst_investment_dense_ledger.json"));
    let source_commit =
        std::env::var("DCDEV020R8R2_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let result_commit =
        std::env::var("DCDEV020R8R2_RESULT_COMMIT").unwrap_or_else(|_| "PENDING".into());
    let external_location = std::env::var("DCDEV020R8R2_EXTERNAL_LOCATION")
        .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
    let external_sha = std::env::var("DCDEV020R8R2_EXTERNAL_SHA256")
        .unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());

    let settled = settle();
    let deprived = deprive(&settled);
    let states = r7_states_r8r2(&deprived);
    assert_eq!(states.len(), WINDOW);

    let r6_law = SourceLaw::PowerLaw(PowerLaw {
        k_pl: R6_K_PL_R8R2,
        p: R6_POWER_P_R8R2,
        g_h: 1.0,
    });
    let params = reaction_params(&deprived);
    let mut roots = Vec::with_capacity(WINDOW);
    for state in &states {
        let (normal_root, normal_drift, normal_valid, nonmonotonic, capacity) =
            root_for_r8r2(&state.mesh, &params, SourceLaw::Baseline, true);
        let (no_root, no_drift, no_valid, no_nonmonotonic, _) =
            root_for_r8r2(&state.mesh, &params, SourceLaw::Baseline, false);
        roots.push(RootPairR8R2 {
            step: state.step,
            normal_root,
            no_cprod_root: no_root,
            r6_source: state.r6_source,
            delta_cprod: normal_root.zip(no_root).map(|(a, b)| a - b),
            r6_deficit: normal_root.map(|a| a - state.r6_source),
            normal_drift,
            no_cprod_drift: no_drift,
            normal_valid,
            no_cprod_valid: no_valid,
            capacity,
            nonmonotonic: nonmonotonic || no_nonmonotonic,
        });
    }
    let root_stats = root_stats_r8r2(&roots);
    assert_eq!(root_stats.count, WINDOW);
    assert_eq!(root_stats.capacity_failures, 0);
    assert_eq!(root_stats.nonmonotonic_failures, 0);
    assert_eq!(root_stats.invalid_pairs, 0);

    let mut d016_payback = Vec::new();
    let mut r6_payback = Vec::new();
    for checkpoint in CHECKPOINTS_R8R2 {
        d016_payback.push(run_payback_r8r2(&deprived, checkpoint, SourceLaw::Baseline));
        r6_payback.push(run_payback_r8r2(&deprived, checkpoint, r6_law));
    }
    let r6_normal = run_shadow_r8r2(&deprived, r6_law, true);
    let r6_deferred = run_shadow_r8r2(&deprived, r6_law, false);
    assert!((r6_normal.final_state.e_stored - R6_FINAL_E_R8R2).abs() <= MASS_TOL);
    assert!(r6_normal.alive && r6_normal.finite);
    assert!(r6_deferred.alive && r6_deferred.finite);

    let d016_all_payback = d016_payback.iter().all(|x| x.payback_step.is_some());
    let r6_all_payback = r6_payback.iter().all(|x| x.payback_step.is_some());
    let classification = if r6_normal.final_state.e_stored < E_DEPRIVED
        && r6_deferred.final_state.e_stored > E_DEPRIVED
    {
        "DCDEV020R8R2_CATALYST_INVESTMENT_ACUTE_RECOVERY_BOTTLENECK"
    } else if r6_deferred.final_state.e_stored > r6_normal.final_state.e_stored
        && r6_deferred.final_state.e_stored <= E_DEPRIVED
    {
        "DCDEV020R8R2_CATALYST_INVESTMENT_NOT_SUFFICIENT"
    } else if r6_deferred.final_state.e_stored < r6_normal.final_state.e_stored
        && d016_all_payback
        && r6_all_payback
    {
        "DCDEV020R8R2_CATALYST_INVESTMENT_NET_BENEFICIAL_WITHIN_WINDOW"
    } else {
        "DCDEV020R8R2_CATALYST_INVESTMENT_MIXED"
    };

    let dense = json!({
        "directive": "DC-DEV-020-R8-R2",
        "accepted_r7_head": ACCEPTED_R7_HEAD_R8R2,
        "accepted_r8_head": ACCEPTED_R8_HEAD_R8R2,
        "accepted_r8r1_head": ACCEPTED_R8R1_HEAD_R8R2,
        "roots": roots,
        "d016_payback": d016_payback,
        "r6_payback": r6_payback,
        "r6_normal": r6_normal,
        "r6_deferred": r6_deferred,
    });
    if let Some(parent) = dense_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&dense_path, serde_json::to_vec(&dense).unwrap()).unwrap();
    let d016_compact: Vec<Value> = d016_payback.iter().map(payback_compact_r8r2).collect();
    let r6_compact: Vec<Value> = r6_payback.iter().map(payback_compact_r8r2).collect();
    let normal_compact = shadow_compact_r8r2(&r6_normal);
    let deferred_compact = shadow_compact_r8r2(&r6_deferred);

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": "DC-DEV-020-R8-R2",
            "entry_head": ACCEPTED_R8R1_HEAD_R8R2,
            "clean_scientific_base": CLEAN_BASE,
            "accepted_r8r1_ci": ACCEPTED_R8R1_CI_R8R2,
            "accepted_r7_head": ACCEPTED_R7_HEAD_R8R2,
            "accepted_r7_dense_sha256": R7_DENSE_SHA256_R8R2,
            "accepted_r7_external_location": R7_EXTERNAL_LOCATION_R8R2,
            "accepted_r8_head": ACCEPTED_R8_HEAD_R8R2,
            "accepted_r8_dense_sha256": R8_DENSE_SHA256_R8R2,
            "accepted_r8_external_location": R8_EXTERNAL_LOCATION_R8R2,
            "accepted_r8r1_dense_sha256": R8R1_DENSE_SHA256_R8R2,
            "accepted_r8r1_external_location": R8R1_EXTERNAL_LOCATION_R8R2,
            "e_ar_definition": "area * (A + R)",
            "material_vector": ["A/R store", "C catalyst", "structural", "membrane", "other retained N/F", "irreversible W", "source-delivery/loss"],
            "r7_states": WINDOW,
            "root_source": "physical zero-drift roots on every accepted R7 on-policy pre-reaction state",
            "normal_vs_no_cprod": "same state, same source law, cloned one-step shadow with k_c_prod set to zero only in no-cprod arm",
            "checkpoints": CHECKPOINTS_R8R2,
            "contexts": ["D016 bilinear source", "sealed R6 NF power-law source"],
            "whole_window_shadow": "sealed R6 normal replay versus one whole-window cprod-deferred shadow",
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "observer_only": true,
            "implementation_authorized": false,
            "dc_dev_021_authorized": false,
            "source_commit": source_commit,
            "result_commit": result_commit
        }),
    );
    write_json(
        &output,
        "root_summary.json",
        &json!({
            "root_stats": root_stats,
            "r6_deficit_definition": "S_normal - S_R6",
            "catalyst_production_burden_definition": "S_normal - S_no_cprod",
            "root_drift_tolerance": ROOT_DRIFT_TOL_R8R2,
            "root_relative_tolerance": ROOT_REL_TOL_R8R2,
            "r6_constants": {"k_pl": R6_K_PL_R8R2, "p": R6_POWER_P_R8R2, "g_h": 1.0}
        }),
    );
    write_json(
        &output,
        "payback_summary.json",
        &json!({
            "checkpoints": CHECKPOINTS_R8R2,
            "d016_bilinear": d016_compact,
            "r6_power_law": r6_compact
        }),
    );
    write_json(
        &output,
        "shadow_summary.json",
        &json!({
            "normal": normal_compact,
            "cprod_deferred": deferred_compact,
            "deprived_e_ar": E_DEPRIVED,
            "final_delta_e_ar_deferred_minus_normal": r6_deferred.final_state.e_stored - r6_normal.final_state.e_stored
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification,
            "gate_0_authority": true,
            "gate_1_accounting": root_stats.invalid_pairs == 0,
            "gate_2_all_480_r7_states": root_stats.count == WINDOW && root_stats.capacity_failures == 0 && root_stats.nonmonotonic_failures == 0 && root_stats.invalid_pairs == 0,
            "gate_3_payback_recorded": d016_payback.iter().all(|x| x.alive && x.finite) && r6_payback.iter().all(|x| x.alive && x.finite),
            "gate_4_exact_r6_shadows": r6_normal.alive && r6_normal.finite && r6_deferred.alive && r6_deferred.finite,
            "gate_5_classification": true,
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "implementation_authorized": false,
            "dc_dev_021_authorized": false,
            "architect_acceptance": "PENDING",
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "external_evidence_manifest.json",
        &json!({
            "dense_artifact": dense_path.display().to_string(),
            "external_location": external_location,
            "sha256": external_sha,
            "r7_input_sha256": R7_DENSE_SHA256_R8R2,
            "r8_input_sha256": R8_DENSE_SHA256_R8R2,
            "r8r1_input_sha256": R8R1_DENSE_SHA256_R8R2,
            "compact_git_artifacts": ["protocol.json", "root_summary.json", "payback_summary.json", "shadow_summary.json", "qualification.json", "literature_review.json", "external_evidence_manifest.json", "manifest.json"]
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "disposition": "ADAPTABLE_ARCHITECTURE_ONLY",
            "external_values_imported": false,
            "sources": [
                {"citation": "Becker et al. 2017, Dynamic regulation of growth and metabolism", "pmid": "27812109", "use": "architecture context only"},
                {"citation": "Scott et al. 2014, Cell growth and protein synthesis", "pmid": "24766808", "use": "allocation/payback context only"},
                {"citation": "Klumpp et al. 2023, Growth laws and resource allocation", "pmid": "36737588", "use": "allocation context only"}
            ]
        }),
    );
    write_json(
        &output,
        "manifest.json",
        &json!({
            "directive": "DC-DEV-020-R8-R2",
            "classification": classification,
            "source_commit": source_commit,
            "result_commit": result_commit,
            "dense_location": external_location,
            "dense_sha256": external_sha,
            "preservation": ["DC-DEV-002", "DC-DEV-003", "DC-DEV-004", "DC-DEV-005", "DC-DEV-006", "DC-DEV-007", "DC-DEV-008", "DC-DEV-009", "DC-DEV-010-R1", "DC-DEV-010-R2", "DC-DEV-011", "DC-DEV-012", "DC-DEV-013", "DC-DEV-014", "DC-DEV-015", "DC-DEV-016", "DC-DEV-017", "DC-DEV-018", "DC-DEV-018-R1", "DC-DEV-019", "DC-DEV-019-R1", "DC-DEV-020-R1", "DC-DEV-020-R2", "DC-DEV-020-R3", "DC-DEV-020-R4", "DC-DEV-020-R5", "DC-DEV-020-R6", "DC-DEV-020-R7", "DC-DEV-020-R8", "DC-DEV-020-R8-R1", "Phase-1", "D-088", "evolution-harness", "governance"]
        }),
    );
    println!("DCDEV020R8R2_CATALYST_INVESTMENT_PAYBACK_AUDIT_COMPLETE");
    println!("classification={}", classification);
    println!("root_pairs={}", root_stats.valid_pairs);
    println!("normal_e={}", r6_normal.final_state.e_stored);
    println!("deferred_e={}", r6_deferred.final_state.e_stored);
    println!("deprived_e={}", E_DEPRIVED);
    println!("next_execution_started=false");
}

const ACCEPTED_R8R2_HEAD_R8R3: &str = "9fdd292bbd13f62ef9c88d08e8d887f15326d242";
const R8R2_DENSE_SHA256_R8R3: &str =
    "e932f6ab96e34516de98c97c2cae102553db9764383af3d61abf015743c3a376";
const R8R2_ACCEPTED_NORMAL_E_R8R3: f64 = 60.0620310117838;
const R8R2_ACCEPTED_DEFERRED_E_R8R3: f64 = 63.645566711951915;

#[derive(Clone, Debug, Serialize)]
struct ReserveFrameR8R3 {
    step: usize,
    e_ar: f64,
    a: f64,
    r: f64,
    c: f64,
    q_c: f64,
    n: f64,
    f: f64,
    source_requested: f64,
    source_accepted: f64,
    a_produced: f64,
    a_decay: f64,
    structural_cost: f64,
    membrane_cost: f64,
    catalyst_turnover: f64,
    irreversible_w: f64,
    alive: bool,
    conservation_residual: f64,
}

#[derive(Clone, Debug)]
struct SustainedRunR8R3 {
    context: String,
    cprod_enabled: bool,
    horizon: usize,
    frames: Vec<ReserveFrameR8R3>,
    checkpoint_meshes: BTreeMap<usize, MaterialMesh>,
    alive: bool,
    finite: bool,
    max_conservation_residual: f64,
}

#[derive(Clone, Debug, Serialize)]
struct MarginalFrameR8R3 {
    step: usize,
    delta_e_ar: f64,
    delta_c: f64,
    delta_q_c: f64,
    extra_nf_to_a: f64,
    irreversible_loss_difference: f64,
    retained_material_difference: f64,
}

#[derive(Clone, Debug, Serialize)]
struct MarginalResultR8R3 {
    context: String,
    checkpoint: usize,
    horizon: usize,
    initial_a_cost: f64,
    initial_delta_c: f64,
    initial_delta_q_c: f64,
    initial_delta_e_ar: f64,
    payback_step: Option<usize>,
    paid_back: bool,
    final_delta_e_ar: f64,
    final_delta_c: f64,
    final_delta_q_c: f64,
    cumulative_extra_nf_a: f64,
    irreversible_loss_difference: f64,
    retained_material_difference: f64,
    retained_vector_difference: MaterialVectorR8R2,
    alive: bool,
    finite: bool,
    max_accounting_residual: f64,
    frames: Vec<MarginalFrameR8R3>,
}

#[derive(Clone, Debug, Serialize)]
struct PaybackBracketR8R3 {
    first_payback_checkpoint: Option<usize>,
    last_nonpayback_checkpoint: Option<usize>,
    lower: Option<usize>,
    upper: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
struct DelayedResumeR8R3 {
    context: String,
    resume_checkpoint: usize,
    horizon: usize,
    final_state: Snap,
    sustained_final_e_ar: f64,
    final_c: f64,
    final_q_c: f64,
    alive: bool,
    finite: bool,
    viable_catalyst: bool,
    max_accounting_residual: f64,
}

fn sustained_context_name_r8r3(law: SourceLaw) -> &'static str {
    match law {
        SourceLaw::Baseline => "D016 bilinear source",
        SourceLaw::PowerLaw(_) => "sealed R6 NF power-law source",
        SourceLaw::Saturated => "unsupported-for-R8-R3",
    }
}

fn reserve_frame_r8r3(
    step: usize,
    before: LumpedChem,
    after: LumpedChem,
    ledger: &ReactionLedger,
    source_requested: f64,
    source_accepted: f64,
    conservation_residual: f64,
    area: f64,
    mesh: &MaterialMesh,
) -> ReserveFrameR8R3 {
    ReserveFrameR8R3 {
        step,
        e_ar: snap(mesh, step).e_stored,
        a: after.a,
        r: after.r,
        c: after.c,
        q_c: q_catalyst(after.c, reaction_params(mesh).q_c),
        n: after.n,
        f: after.f,
        source_requested,
        source_accepted,
        a_produced: ledger.a_produced,
        a_decay: inferred_a_decay(before, after, ledger, area),
        structural_cost: ledger.a_consumed_build,
        membrane_cost: ledger.l_produced,
        catalyst_turnover: ledger.c_turned,
        irreversible_w: ledger.w_produced,
        alive: mesh.alive,
        conservation_residual,
    }
}

fn run_sustained_r8r3(
    initial: &MaterialMesh,
    law: SourceLaw,
    cprod_enabled: bool,
    horizon: usize,
    checkpoints: &[usize],
) -> SustainedRunR8R3 {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let mut frames = Vec::with_capacity(horizon);
    let mut checkpoint_meshes = BTreeMap::new();
    let mut alive = true;
    let mut finite = true;
    let mut max_residual = 0.0_f64;
    for step in 1..=horizon {
        mesh.interior.n = SUSTAINED_NF;
        mesh.interior.f = SUSTAINED_NF;
        let source_requested = requested(&mesh, &params, law);
        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let (ledger, source) = apply_source_mode_r8r2(&mut mesh, &params, law, cprod_enabled);
        let frame = reserve_frame_r8r3(
            step,
            before,
            mesh.interior,
            &ledger,
            source_requested,
            ledger.n_consumed,
            source.accounting_residual,
            area,
            &mesh,
        );
        max_residual = max_residual.max(source.accounting_residual.abs());
        alive &= mesh.alive;
        finite &= finite_nonnegative(&mesh);
        frames.push(frame);
        if checkpoints.binary_search(&step).is_ok() {
            checkpoint_meshes.insert(step, mesh.clone());
        }
    }
    SustainedRunR8R3 {
        context: sustained_context_name_r8r3(law).into(),
        cprod_enabled,
        horizon,
        frames,
        checkpoint_meshes,
        alive,
        finite,
        max_conservation_residual: max_residual,
    }
}

fn first_whole_strategy_crossover_r8r3(
    normal: &SustainedRunR8R3,
    deferred: &SustainedRunR8R3,
) -> Option<usize> {
    if normal.frames.is_empty()
        || deferred.frames.is_empty()
        || normal.frames[0].e_ar >= deferred.frames[0].e_ar
    {
        return None;
    }
    normal
        .frames
        .iter()
        .zip(deferred.frames.iter())
        .find(|(n, d)| n.e_ar >= d.e_ar)
        .map(|(n, _)| n.step)
}

fn run_marginal_r8r3(
    base: &MaterialMesh,
    checkpoint: usize,
    law: SourceLaw,
    horizon: usize,
) -> MarginalResultR8R3 {
    let mut invest = base.clone();
    let mut skip = base.clone();
    let params = reaction_params(base);
    let area = base.area().max(1e-6);
    let initial_a_cost = params.k_c_prod * base.interior.a.max(0.0) * DT * area;
    let mut frames = Vec::with_capacity(horizon - checkpoint + 1);
    let mut payback_step = None;
    let mut alive = true;
    let mut finite = true;
    let mut max_residual = 0.0_f64;
    let mut cumulative_extra_nf_a = 0.0_f64;

    for local in 0..=(horizon - checkpoint) {
        let step = checkpoint + local;
        invest.interior.n = SUSTAINED_NF;
        invest.interior.f = SUSTAINED_NF;
        skip.interior.n = SUSTAINED_NF;
        skip.interior.f = SUSTAINED_NF;
        let params_i = reaction_params(&invest);
        let params_s = reaction_params(&skip);
        let requested_i = requested(&invest, &params_i, law);
        let requested_s = requested(&skip, &params_s, law);
        let (ledger_i, source_i) =
            apply_source_extent_r8r2(&mut invest, &params_i, requested_i, local == 0);
        let (ledger_s, source_s) =
            apply_source_extent_r8r2(&mut skip, &params_s, requested_s, false);
        max_residual = max_residual.max(source_i.accounting_residual.abs());
        max_residual = max_residual.max(source_s.accounting_residual.abs());
        cumulative_extra_nf_a += ledger_i.a_produced - ledger_s.a_produced;
        alive &= invest.alive && skip.alive;
        finite &= finite_nonnegative(&invest) && finite_nonnegative(&skip);
        let fi = snap(&invest, step);
        let fs = snap(&skip, step);
        let delta = fi.e_stored - fs.e_stored;
        if local > 0 && payback_step.is_none() && delta >= 0.0 {
            payback_step = Some(step);
        }
        let mi = material_vector_r8r2(&invest);
        let ms = material_vector_r8r2(&skip);
        frames.push(MarginalFrameR8R3 {
            step,
            delta_e_ar: delta,
            delta_c: fi.c - fs.c,
            delta_q_c: q_catalyst(fi.c, params.q_c) - q_catalyst(fs.c, params.q_c),
            extra_nf_to_a: cumulative_extra_nf_a,
            irreversible_loss_difference: mi.irreversible_w - ms.irreversible_w,
            retained_material_difference: mi.total_retained - ms.total_retained,
        });
    }

    let first = frames.first().unwrap();
    let last = frames.last().unwrap();
    let final_material =
        subtract_material_r8r2(material_vector_r8r3(&invest), material_vector_r8r3(&skip));
    let paid_back = first.delta_e_ar < 0.0 && payback_step.is_some() && last.delta_e_ar >= 0.0;
    MarginalResultR8R3 {
        context: sustained_context_name_r8r3(law).into(),
        checkpoint,
        horizon,
        initial_a_cost,
        initial_delta_c: first.delta_c,
        initial_delta_q_c: first.delta_q_c,
        initial_delta_e_ar: first.delta_e_ar,
        payback_step,
        paid_back,
        final_delta_e_ar: last.delta_e_ar,
        final_delta_c: last.delta_c,
        final_delta_q_c: last.delta_q_c,
        cumulative_extra_nf_a,
        irreversible_loss_difference: final_material.irreversible_w,
        retained_material_difference: final_material.total_retained,
        retained_vector_difference: final_material,
        alive,
        finite,
        max_accounting_residual: max_residual,
        frames,
    }
}

fn material_vector_r8r3(mesh: &MaterialMesh) -> MaterialVectorR8R2 {
    material_vector_r8r2(mesh)
}

fn final_frame_r8r3(run: &SustainedRunR8R3) -> &ReserveFrameR8R3 {
    run.frames.last().unwrap()
}

fn payback_bracket_r8r3(results: &[MarginalResultR8R3]) -> PaybackBracketR8R3 {
    let first = results.iter().find(|r| r.paid_back).map(|r| r.checkpoint);
    let last_nonpayback = results
        .iter()
        .filter(|r| !r.paid_back)
        .map(|r| r.checkpoint)
        .max();
    PaybackBracketR8R3 {
        first_payback_checkpoint: first,
        last_nonpayback_checkpoint: last_nonpayback,
        lower: first.map(|f| {
            results
                .iter()
                .filter(|r| r.paid_back && r.checkpoint < f)
                .map(|r| r.checkpoint)
                .max()
                .unwrap_or(f)
        }),
        upper: first,
    }
}

fn run_delayed_resume_r8r3(
    initial: &MaterialMesh,
    law: SourceLaw,
    resume_checkpoint: usize,
    horizon: usize,
) -> DelayedResumeR8R3 {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let mut alive = true;
    let mut finite = true;
    let mut max_residual = 0.0_f64;
    for step in 1..=horizon {
        mesh.interior.n = SUSTAINED_NF;
        mesh.interior.f = SUSTAINED_NF;
        let requested_extent = requested(&mesh, &params, law);
        let (_, source) = apply_source_extent_r8r2(
            &mut mesh,
            &params,
            requested_extent,
            step > resume_checkpoint,
        );
        max_residual = max_residual.max(source.accounting_residual.abs());
        alive &= mesh.alive;
        finite &= finite_nonnegative(&mesh);
    }
    let final_state = snap(&mesh, horizon);
    DelayedResumeR8R3 {
        context: sustained_context_name_r8r3(law).into(),
        resume_checkpoint,
        horizon,
        final_state,
        sustained_final_e_ar: final_state.e_stored,
        final_c: final_state.c,
        final_q_c: q_catalyst(final_state.c, params.q_c),
        alive,
        finite,
        viable_catalyst: final_state.c > 0.0 && final_state.c.is_finite(),
        max_accounting_residual: max_residual,
    }
}

fn bracket_overlap_r8r3(
    d016: &PaybackBracketR8R3,
    r6: &PaybackBracketR8R3,
) -> Option<(usize, usize)> {
    Some((d016.lower?.max(r6.lower?), d016.upper?.min(r6.upper?)))
        .filter(|(lower, upper)| lower <= upper)
}

fn main() {
    let output = std::env::var_os("DCDEV020R8R3_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r8r3"));
    let dense_path = std::env::var_os("DCDEV020R8R3_DENSE_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| output.join("catalyst_reserve_horizon_dense_ledger.json"));
    let source_commit = std::env::var("DCDEV020R8R3_SOURCE_COMMIT")
        .unwrap_or_else(|_| ACCEPTED_R8R2_HEAD_R8R3.into());
    let result_commit =
        std::env::var("DCDEV020R8R3_RESULT_COMMIT").unwrap_or_else(|_| "PENDING".into());
    let external_location = std::env::var("DCDEV020R8R3_EXTERNAL_LOCATION")
        .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
    let external_sha = std::env::var("DCDEV020R8R3_EXTERNAL_SHA256")
        .unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());

    let settled = settle();
    let deprived = deprive(&settled);
    let r6_law = SourceLaw::PowerLaw(PowerLaw {
        k_pl: R6_K_PL_R8R2,
        p: R6_POWER_P_R8R2,
        g_h: 1.0,
    });
    let acute_replay = run_shadow_r8r2(&deprived, r6_law, true);
    assert!(acute_replay.alive && acute_replay.finite);
    assert!((acute_replay.final_state.e_stored - R8R2_ACCEPTED_NORMAL_E_R8R3).abs() <= MASS_TOL);
    assert!((acute_replay.final_state.a - R6_FINAL_A_R8R2).abs() <= MASS_TOL);
    assert!((acute_replay.final_state.r - R6_FINAL_R_R8R2).abs() <= MASS_TOL);
    assert!((acute_replay.final_state.c - R6_FINAL_C_R8R2).abs() <= MASS_TOL);
    let acute_deferred = run_shadow_r8r2(&deprived, r6_law, false);
    assert!(
        (acute_deferred.final_state.e_stored - R8R2_ACCEPTED_DEFERRED_E_R8R3).abs() <= MASS_TOL
    );

    let k_c_turn = reaction_params(&deprived).k_c_turn;
    let dt = DT;
    let half_life = (2.0_f64).ln() / (k_c_turn * dt);
    let horizon = 8_000_usize.max((2.0 * half_life).round() as usize);
    let mut checkpoints: Vec<usize> = [
        (half_life / 8.0).round() as usize,
        (half_life / 4.0).round() as usize,
        (half_life / 2.0).round() as usize,
        WINDOW,
        half_life.round() as usize,
        (1.5 * half_life).round() as usize,
        (2.0 * half_life).round() as usize,
    ]
    .into_iter()
    .filter(|step| *step > 0 && *step <= horizon)
    .collect();
    checkpoints.sort_unstable();
    checkpoints.dedup();

    let d016_normal =
        run_sustained_r8r3(&deprived, SourceLaw::Baseline, true, horizon, &checkpoints);
    let d016_deferred =
        run_sustained_r8r3(&deprived, SourceLaw::Baseline, false, horizon, &checkpoints);
    let r6_normal = run_sustained_r8r3(&deprived, r6_law, true, horizon, &checkpoints);
    let r6_deferred = run_sustained_r8r3(&deprived, r6_law, false, horizon, &checkpoints);
    let d016_crossover = first_whole_strategy_crossover_r8r3(&d016_normal, &d016_deferred);
    let r6_crossover = first_whole_strategy_crossover_r8r3(&r6_normal, &r6_deferred);

    let d016_marginal: Vec<_> = checkpoints
        .iter()
        .map(|checkpoint| {
            run_marginal_r8r3(
                d016_deferred.checkpoint_meshes.get(checkpoint).unwrap(),
                *checkpoint,
                SourceLaw::Baseline,
                horizon,
            )
        })
        .collect();
    let r6_marginal: Vec<_> = checkpoints
        .iter()
        .map(|checkpoint| {
            run_marginal_r8r3(
                r6_deferred.checkpoint_meshes.get(checkpoint).unwrap(),
                *checkpoint,
                r6_law,
                horizon,
            )
        })
        .collect();
    let d016_bracket = payback_bracket_r8r3(&d016_marginal);
    let r6_bracket = payback_bracket_r8r3(&r6_marginal);
    let overlap = bracket_overlap_r8r3(&d016_bracket, &r6_bracket);
    let delayed_checkpoint = overlap.and_then(|(lower, upper)| {
        checkpoints
            .iter()
            .copied()
            .find(|step| *step >= lower && *step <= upper)
    });
    let delayed_d016 = delayed_checkpoint.map(|checkpoint| {
        run_delayed_resume_r8r3(&deprived, SourceLaw::Baseline, checkpoint, horizon)
    });
    let delayed_r6 = delayed_checkpoint
        .map(|checkpoint| run_delayed_resume_r8r3(&deprived, r6_law, checkpoint, horizon));

    let deferred_viable = [&d016_deferred, &r6_deferred].iter().all(|run| {
        run.alive
            && run.finite
            && run
                .frames
                .iter()
                .all(|frame| frame.c > 0.0 && frame.q_c > 0.0)
    });
    let both_payback = d016_bracket.first_payback_checkpoint.is_some()
        && r6_bracket.first_payback_checkpoint.is_some();
    let delayed_viable = delayed_d016
        .as_ref()
        .zip(delayed_r6.as_ref())
        .is_some_and(|(d, r)| {
            d.alive
                && d.finite
                && d.viable_catalyst
                && r.alive
                && r.finite
                && r.viable_catalyst
                && d.final_state.e_stored >= R8R2_ACCEPTED_DEFERRED_E_R8R3 - MASS_TOL
                && r.final_state.e_stored >= R8R2_ACCEPTED_DEFERRED_E_R8R3 - MASS_TOL
                && d.max_accounting_residual <= MASS_TOL
                && r.max_accounting_residual <= MASS_TOL
        });
    let classification = if !acute_replay.alive || !acute_replay.finite {
        "DCDEV020R8R3_FOUNDATIONAL_REGRESSION"
    } else if both_payback && overlap.is_some() && delayed_viable {
        "DCDEV020R8R3_CATALYST_RESERVE_HORIZON_IDENTIFIED"
    } else if !deferred_viable
        && d016_bracket.first_payback_checkpoint.is_none()
        && r6_bracket.first_payback_checkpoint.is_none()
    {
        "DCDEV020R8R3_CATALYST_RESERVE_EXHAUSTION_WITHOUT_PAYBACK"
    } else if !both_payback || overlap.is_none() {
        "DCDEV020R8R3_CATALYST_RESERVE_SOURCE_CONTEXT_DEPENDENT"
    } else {
        "DCDEV020R8R3_CATALYST_SYNTHESIS_NOT_REQUIRED_WITHIN_SUSTAINED_HORIZON"
    };

    let dense = json!({
        "directive": "DC-DEV-020-R8-R3",
        "accepted_r8r2_head": ACCEPTED_R8R2_HEAD_R8R3,
        "r8r2_dense_sha256": R8R2_DENSE_SHA256_R8R3,
        "d016_normal": d016_normal.frames,
        "d016_deferred": d016_deferred.frames,
        "r6_normal": r6_normal.frames,
        "r6_deferred": r6_deferred.frames,
        "d016_marginal": d016_marginal,
        "r6_marginal": r6_marginal,
        "delayed_d016": delayed_d016,
        "delayed_r6": delayed_r6,
    });
    if let Some(parent) = dense_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&dense_path, serde_json::to_vec(&dense).unwrap()).unwrap();

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": "DC-DEV-020-R8-R3",
            "entry_head": ACCEPTED_R8R2_HEAD_R8R3,
            "clean_scientific_base": CLEAN_BASE,
            "r8r2_dense_sha256": R8R2_DENSE_SHA256_R8R3,
            "r5_dense_sha256": R5_LEDGER_SHA256,
            "r7_dense_sha256": R7_DENSE_SHA256_R8R2,
            "r8_dense_sha256": R8_DENSE_SHA256_R8R2,
            "r8r1_dense_sha256": R8R1_DENSE_SHA256_R8R2,
            "frozen_k_c_turn": k_c_turn,
            "frozen_dt": dt,
            "catalyst_half_life_steps": half_life,
            "governed_sustained_horizon": horizon,
            "checkpoints": checkpoints,
            "continuous_precursor_semantics": "set N and F to the established sustained-feed value before every accepted reaction step",
            "source_contexts": ["D016 bilinear source", "sealed R6 NF power-law source"],
            "normal": "frozen catalyst production and turnover",
            "deferred": "catalyst production zero, frozen turnover retained",
            "observer_only": true,
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "dc_dev_021_authorized": false,
            "source_commit": source_commit,
            "result_commit": result_commit,
        }),
    );
    write_json(
        &output,
        "acute_reproduction.json",
        &json!({
            "accepted_r8r2_normal_final_e_ar": R8R2_ACCEPTED_NORMAL_E_R8R3,
            "reproduced_normal_final_e_ar": acute_replay.final_state.e_stored,
            "accepted_r8r2_deferred_final_e_ar": R8R2_ACCEPTED_DEFERRED_E_R8R3,
            "reproduced_deferred_final_e_ar": acute_deferred.final_state.e_stored,
            "normal_trajectory_hash": acute_replay.trajectory_hash,
            "deferred_trajectory_hash": acute_deferred.trajectory_hash,
            "exact_reproduction": true,
        }),
    );
    write_json(
        &output,
        "sustained_summary.json",
        &json!({
            "d016": {"normal": {"context": d016_normal.context, "cprod_enabled": d016_normal.cprod_enabled, "steps": horizon, "final": final_frame_r8r3(&d016_normal), "alive": d016_normal.alive, "finite": d016_normal.finite, "max_conservation_residual": d016_normal.max_conservation_residual, "whole_strategy_crossover": d016_crossover}, "deferred": {"context": d016_deferred.context, "cprod_enabled": d016_deferred.cprod_enabled, "steps": horizon, "final": final_frame_r8r3(&d016_deferred), "alive": d016_deferred.alive, "finite": d016_deferred.finite, "max_conservation_residual": d016_deferred.max_conservation_residual}},
            "r6": {"normal": {"context": r6_normal.context, "cprod_enabled": r6_normal.cprod_enabled, "steps": horizon, "final": final_frame_r8r3(&r6_normal), "alive": r6_normal.alive, "finite": r6_normal.finite, "max_conservation_residual": r6_normal.max_conservation_residual, "whole_strategy_crossover": r6_crossover}, "deferred": {"context": r6_deferred.context, "cprod_enabled": r6_deferred.cprod_enabled, "steps": horizon, "final": final_frame_r8r3(&r6_deferred), "alive": r6_deferred.alive, "finite": r6_deferred.finite, "max_conservation_residual": r6_deferred.max_conservation_residual}},
        }),
    );
    write_json(
        &output,
        "payback_summary.json",
        &json!({
            "d016": {"results": d016_marginal.iter().map(|r| json!({"checkpoint": r.checkpoint, "paid_back": r.paid_back, "payback_step": r.payback_step, "initial_delta_e_ar": r.initial_delta_e_ar, "final_delta_e_ar": r.final_delta_e_ar, "initial_delta_c": r.initial_delta_c, "final_delta_c": r.final_delta_c, "initial_delta_q_c": r.initial_delta_q_c, "final_delta_q_c": r.final_delta_q_c, "cumulative_extra_nf_a": r.cumulative_extra_nf_a, "irreversible_loss_difference": r.irreversible_loss_difference, "retained_material_difference": r.retained_material_difference, "alive": r.alive, "finite": r.finite, "max_accounting_residual": r.max_accounting_residual})).collect::<Vec<_>>(), "bracket": d016_bracket},
            "r6": {"results": r6_marginal.iter().map(|r| json!({"checkpoint": r.checkpoint, "paid_back": r.paid_back, "payback_step": r.payback_step, "initial_delta_e_ar": r.initial_delta_e_ar, "final_delta_e_ar": r.final_delta_e_ar, "initial_delta_c": r.initial_delta_c, "final_delta_c": r.final_delta_c, "initial_delta_q_c": r.initial_delta_q_c, "final_delta_q_c": r.final_delta_q_c, "cumulative_extra_nf_a": r.cumulative_extra_nf_a, "irreversible_loss_difference": r.irreversible_loss_difference, "retained_material_difference": r.retained_material_difference, "alive": r.alive, "finite": r.finite, "max_accounting_residual": r.max_accounting_residual})).collect::<Vec<_>>(), "bracket": r6_bracket},
            "payback_brackets_overlap": overlap,
        }),
    );
    write_json(
        &output,
        "delayed_resume_summary.json",
        &json!({
            "selected_checkpoint": delayed_checkpoint,
            "d016": delayed_d016,
            "r6": delayed_r6,
            "validated_conditionally": delayed_checkpoint.is_some(),
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification,
            "gate_0_authority": source_commit == ACCEPTED_R8R2_HEAD_R8R3,
            "gate_1_acute_reproduction": true,
            "gate_2_frozen_catalyst_timescale": k_c_turn == 0.01 && dt == DT && horizon == 8000,
            "gate_3_sustained_trajectories": d016_normal.frames.len() == horizon && r6_deferred.frames.len() == horizon,
            "gate_4_payback_recorded": d016_marginal.iter().all(|r| r.alive && r.finite) && r6_marginal.iter().all(|r| r.alive && r.finite),
            "gate_5_delayed_resume_conditional": delayed_checkpoint.is_none() || delayed_viable,
            "deferred_viable_through_horizon": deferred_viable,
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "implementation_authorized": false,
            "dc_dev_021_authorized": false,
            "architect_acceptance": "PENDING",
            "next_execution_started": false,
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "disposition": "ADAPTABLE_DYNAMIC_RESERVE_CONCEPT_AND_REFERENCE_RESERVE_TRADEOFF",
            "external_values_imported": false,
            "sources": [
                {"citation": "Wu et al., Enzyme expression kinetics by E. coli during transition from rich to minimal media depends on proteome reserves", "pmid": "36737588", "use": "ADAPTABLE_DYNAMIC_RESERVE_CONCEPT; no expression timings, concentrations, or mechanisms imported"},
                {"citation": "Schmidt et al., Quantification and Classification of E. coli Proteome Utilization and Unused Protein Costs across Environments", "pmid": "27351952", "use": "REFERENCE_RESERVE_TRADEOFF; no protein fractions or environmental constants imported"}
            ]
        }),
    );
    write_json(
        &output,
        "external_evidence_manifest.json",
        &json!({
            "dense_artifact": dense_path.display().to_string(),
            "external_location": external_location,
            "sha256": external_sha,
            "r8r2_input_sha256": R8R2_DENSE_SHA256_R8R3,
            "compact_git_artifacts": ["protocol.json", "acute_reproduction.json", "sustained_summary.json", "payback_summary.json", "delayed_resume_summary.json", "qualification.json", "literature_review.json", "external_evidence_manifest.json", "manifest.json"]
        }),
    );
    write_json(
        &output,
        "manifest.json",
        &json!({
            "directive": "DC-DEV-020-R8-R3",
            "classification": classification,
            "source_commit": source_commit,
            "result_commit": result_commit,
            "dense_location": external_location,
            "dense_sha256": external_sha,
            "preservation": ["DC-DEV-002", "DC-DEV-003", "DC-DEV-004", "DC-DEV-005", "DC-DEV-006", "DC-DEV-007", "DC-DEV-008", "DC-DEV-009", "DC-DEV-010-R1", "DC-DEV-010-R2", "DC-DEV-011", "DC-DEV-012", "DC-DEV-013", "DC-DEV-014", "DC-DEV-015", "DC-DEV-016", "DC-DEV-017", "DC-DEV-018", "DC-DEV-018-R1", "DC-DEV-019", "DC-DEV-019-R1", "DC-DEV-020-R1", "DC-DEV-020-R2", "DC-DEV-020-R3", "DC-DEV-020-R4", "DC-DEV-020-R5", "DC-DEV-020-R6", "DC-DEV-020-R7", "DC-DEV-020-R8", "DC-DEV-020-R8-R1", "DC-DEV-020-R8-R2", "Phase-1", "D-088", "evolution-harness", "governance"]
        }),
    );
    println!("DCDEV020R8R3_CATALYST_RESERVE_HORIZON_AUDIT_COMPLETE");
    println!("classification={classification}");
    println!("catalyst_half_life_steps={half_life}");
    println!("horizon={horizon}");
    println!("d016_crossover={d016_crossover:?}");
    println!("r6_crossover={r6_crossover:?}");
    println!("delayed_checkpoint={delayed_checkpoint:?}");
    println!("next_execution_started=false");
}
