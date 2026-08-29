//! DC-DEV-020-R6 observer-only symmetric N/F power-law source audit.
//!
//! The candidate is fitted only to the accepted R5 statewise zero-drift roots.
//! It is then executed counterfactually through the frozen chemistry. Production
//! chemistry, parameters, transport, resources, mechanics, and behavior remain
//! unchanged.

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

fn main() {
    let output = std::env::var_os("DCDEV020R6_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r6"));
    let ledger_path = std::env::var_os("DCDEV020R5_EXTERNAL_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r5-statewise-ledger.json"));
    let source_commit =
        std::env::var("DCDEV020R6_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let records: Vec<R5Root> = serde_json::from_slice(&fs::read(&ledger_path).unwrap()).unwrap();
    assert_eq!(records.len(), 4_800);

    let fit = fit_model(&records);
    let p3 = prediction_report(&records, &["P3"], fit.model);
    let p4 = prediction_report(&records, &["P4"], fit.model);
    let holdout = prediction_report(&records, &["P3", "P4"], fit.model);
    let gate2 = fit.model.k_pl.is_finite()
        && fit.model.k_pl > 0.0
        && fit.model.p.is_finite()
        && (0.0..=1.0).contains(&fit.model.p);
    let gate3 = gate2
        && holdout.relative_rmse <= REL_RMSE_LIMIT
        && holdout.p95_absolute_relative_error <= P95_LIMIT
        && holdout.predicted_capacity_violations == 0;
    let all = prediction_report(&records, &["P0", "P1", "P2", "P3", "P4"], fit.model);
    let zero_control = power_requested_values(fit.model, 1.0, 0.0, 1.0, 1.0) == 0.0
        && power_requested_values(fit.model, 1.0, 1.0, 0.0, 1.0) == 0.0;
    let symmetry = (power_requested_values(fit.model, 1.0, 0.2, 0.8, 1.0)
        - power_requested_values(fit.model, 1.0, 0.8, 0.2, 1.0))
    .abs()
        <= SOURCE_EPS;
    let gate4 = gate3
        && zero_control
        && symmetry
        && all.predicted_capacity_violations == 0
        && all.clipping_fraction == 0.0;

    let settled = settle();
    let deprived = deprive(&settled);
    let mut baseline_mesh = deprived.clone();
    let baseline = run_window(
        &mut baseline_mesh,
        &settled,
        SourceLaw::Baseline,
        WINDOW,
        Some(M_SELECTED),
        1.0,
        0,
    );
    let mut r6_mesh = deprived.clone();
    let r6 = run_window(
        &mut r6_mesh,
        &settled,
        SourceLaw::PowerLaw(fit.model),
        WINDOW,
        Some(M_SELECTED),
        1.0,
        0,
    );
    let mut saturated_mesh = deprived.clone();
    let saturated = run_window(
        &mut saturated_mesh,
        &settled,
        SourceLaw::Saturated,
        WINDOW,
        Some(M_SELECTED),
        1.0,
        0,
    );
    let gate5 = gate4
        && r6.alive_throughout
        && r6.finite_nonnegative
        && r6.max_resource_conservation_error <= MASS_TOL
        && r6.max_stored_accounting_residual <= MASS_TOL
        && r6.final_state.e_stored > E_DEPRIVED
        && r6.settled_distance_final < r6.settled_distance_initial
        && (r6.a_toward_settled || r6.r_toward_settled)
        && r6.capacity_violation_steps == 0;

    let mut doses = Vec::new();
    let mut gate6 = false;
    if gate5 {
        for scale in [0.75, 1.0, 1.25] {
            let mut mesh = deprived.clone();
            doses.push(run_window(
                &mut mesh,
                &settled,
                SourceLaw::PowerLaw(fit.model),
                WINDOW,
                Some(M_SELECTED),
                scale,
                0,
            ));
        }
        gate6 = doses.iter().all(|d| {
            d.alive_throughout
                && d.finite_nonnegative
                && d.max_resource_conservation_error <= MASS_TOL
                && d.max_stored_accounting_residual <= MASS_TOL
                && d.capacity_violation_steps == 0
        }) && doses
            .windows(2)
            .all(|w| w[1].final_state.e_stored + MASS_TOL >= w[0].final_state.e_stored);
    }

    let mut sustained_r6 = None;
    let mut sustained_baseline = None;
    let mut gate7 = false;
    let mut oscillatory = false;
    if gate6 {
        let mut candidate_mesh = deprived.clone();
        let candidate = run_sustained(&mut candidate_mesh, SourceLaw::PowerLaw(fit.model));
        let mut reference_mesh = deprived.clone();
        let reference = run_sustained(&mut reference_mesh, SourceLaw::Baseline);
        let band_low = 0.95 * E_TARGET;
        let band_high = 1.05 * E_TARGET;
        oscillatory =
            candidate.final_quarter_min < band_low && candidate.final_quarter_max > band_high;
        gate7 = candidate.alive_throughout
            && candidate.finite_nonnegative
            && candidate.final_state.e_stored >= band_low
            && candidate.final_state.e_stored <= band_high
            && candidate.final_quarter_slope.abs() <= 0.01 * reference.final_quarter_slope.abs()
            && candidate.peak_e_stored <= 1.10 * E_TARGET
            && !oscillatory
            && candidate.final_quarter_clipping_steps == 0
            && candidate.final_quarter_accelerated_steps == 0
            && candidate.max_stored_accounting_residual <= MASS_TOL;
        sustained_r6 = Some((candidate_mesh, candidate));
        sustained_baseline = Some(reference);
    }

    let mut cycles = Vec::new();
    let mut gate8 = false;
    let mut reserve_monotonic_collapse = false;
    let mut clipping_escalation = false;
    if gate7 {
        let (mut mesh, _) = sustained_r6.as_ref().unwrap().clone();
        for cycle in 0..3 {
            let deprived_run = run_window(
                &mut mesh,
                &settled,
                SourceLaw::PowerLaw(fit.model),
                WINDOW,
                None,
                1.0,
                cycle * 2 * WINDOW,
            );
            let fed_run = run_window(
                &mut mesh,
                &settled,
                SourceLaw::PowerLaw(fit.model),
                WINDOW,
                Some(M_SELECTED),
                1.0,
                cycle * 2 * WINDOW + WINDOW,
            );
            cycles.push(CycleSummary {
                cycle: cycle + 1,
                recovery: fed_run.final_state.e_stored - deprived_run.final_state.e_stored,
                fed_final_r: fed_run.final_state.r,
                deprived: deprived_run,
                fed: fed_run,
            });
        }
        reserve_monotonic_collapse = cycles
            .windows(2)
            .all(|w| w[1].fed_final_r < w[0].fed_final_r);
        clipping_escalation = cycles
            .windows(2)
            .all(|w| w[1].fed.clipping_steps > w[0].fed.clipping_steps);
        gate8 = cycles.iter().all(|c| {
            c.deprived.alive_throughout
                && c.fed.alive_throughout
                && c.deprived.finite_nonnegative
                && c.fed.finite_nonnegative
                && c.recovery > 0.0
                && c.fed.max_resource_conservation_error <= MASS_TOL
                && c.fed.max_stored_accounting_residual <= MASS_TOL
                && c.fed.accelerated_a_decay_steps == 0
        }) && cycles[2].recovery >= 0.90 * cycles[0].recovery
            && !reserve_monotonic_collapse
            && !clipping_escalation;
    }

    let classification = if !gate2 {
        "DCDEV020R6_POWER_LAW_KINETIC_ORDER_REJECTED"
    } else if !gate3 {
        "DCDEV020R6_POWER_LAW_LOCAL_REQUIREMENT_NOT_ESTABLISHED"
    } else if !gate4 {
        "DCDEV020R6_POWER_LAW_CLIPPING_DEPENDENT"
    } else if !gate5 && r6.final_state.e_stored + MASS_TOL >= r6.initial.e_stored {
        "DCDEV020R6_NF_POWER_LAW_MAINTENANCE_WITHOUT_RESTORATION"
    } else if !gate5 {
        "DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE"
    } else if !gate6 {
        "DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE"
    } else if !gate7 && oscillatory {
        "DCDEV020R6_NF_POWER_LAW_OSCILLATORY"
    } else if !gate7 {
        "DCDEV020R6_NF_POWER_LAW_NO_STABLE_HOMEOSTASIS"
    } else if !gate8 {
        "DCDEV020R6_REPEATABILITY_FAILURE"
    } else {
        "DCDEV020R6_NF_POWER_LAW_OBSERVER_QUALIFIED"
    };

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-020-R6", "accepted_r5_head":ACCEPTED_R5_HEAD,
            "clean_scientific_base":CLEAN_BASE, "source_commit":source_commit,
            "r5_dense_ledger_sha256":R5_LEDGER_SHA256, "r5_external_location":R5_EXTERNAL_LOCATION,
            "training_probes":["P0","P1","P2"], "holdout_probes":["P3","P4"],
            "g_h":1.0, "fit":"closed-form OLS in log space; no search", "candidate":"q_c*g_h*K_PL*N^p*F^p",
            "constraints":{"k_pl_positive":true,"p_min":0.0,"p_max":1.0},
            "finite_feed_steps":WINDOW, "selected_mass":M_SELECTED,
            "dose_scales":[0.75,1.0,1.25], "sustained_steps":SUSTAINED_STEPS,
            "sustained_nf_clamp":SUSTAINED_NF, "cycles":3,
            "observer_only":true, "production_integration":false
        }),
    );
    write_json(
        &output,
        "identification.json",
        &json!({
            "fit":fit, "p3":p3, "p4":p4, "combined_holdout":holdout,
            "all_state_sanity":all, "zero_substrate_control":zero_control,
            "nf_symmetry":symmetry, "gates":{"gate2":gate2,"gate3":gate3,"gate4":gate4}
        }),
    );
    write_json(
        &output,
        "physiology.json",
        &json!({
            "baseline":baseline, "r6":r6, "source_saturated":saturated,
            "dose_arms":doses, "sustained_r6":sustained_r6.as_ref().map(|x|&x.1),
            "sustained_bilinear_reference":sustained_baseline, "oscillatory":oscillatory,
            "cycles":cycles, "reserve_monotonic_collapse":reserve_monotonic_collapse,
            "clipping_escalation":clipping_escalation,
            "gates":{"gate5":gate5,"gate6":gate6,"gate7":gate7,"gate8":gate8}
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification":classification, "gate2":gate2, "gate3":gate3, "gate4":gate4,
            "gate5":gate5, "gate6":gate6, "gate7":gate7, "gate8":gate8,
            "production_chemistry_changed":false, "production_behavior_changed":false,
            "implementation_authorized":false, "dc_dev_021_authorized":false,
            "next_execution_started":false
        }),
    );
    write_json(
        &output,
        "external_evidence_manifest.json",
        &json!({
            "dense_input":"accepted R5 statewise root ledger", "sha256":R5_LEDGER_SHA256,
            "external_location":R5_EXTERNAL_LOCATION,
            "r6_dense_output":"none; compact summaries and trajectory hashes only"
        }),
    );
    write_json(
        &output,
        "literature_review.json",
        &json!({
            "disposition":"ADAPTABLE_ARCHITECTURE_ONLY", "external_values_imported":false,
            "sources":[
                {"citation":"Savageau 1969, Biochemical systems analysis I", "pmid":"5387046", "doi":"10.1016/S0022-5193(69)80026-3", "use":"component enzymatic rate-law architecture precedent only"},
                {"citation":"Savageau 1969, Biochemical systems analysis II", "pmid":"5387047", "doi":"10.1016/S0022-5193(69)80027-5", "use":"power-law approximation architecture precedent only"},
                {"citation":"Muller and Regensburger 2012, Generalized Mass Action Systems", "doi":"10.1137/110847056", "use":"arbitrary kinetic-order reaction-network formalism precedent only"}
            ],
            "digital_cell_parameter_source":"K_PL and p fitted only from accepted R5 roots"
        }),
    );
    println!("DCDEV020R6_NF_POWER_LAW_SOURCE_AUDIT_COMPLETE");
    println!("K_PL={}", fit.model.k_pl);
    println!("p={}", fit.model.p);
    println!("classification={classification}");
    println!("NEXT_EXECUTION_STARTED:false");
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: PowerLaw = PowerLaw {
        k_pl: 0.017556661171593057,
        p: 0.0003277429681759396,
        g_h: 1.0,
    };

    #[test]
    fn power_law_is_exactly_zero_when_either_substrate_is_absent() {
        assert_eq!(power_requested_values(MODEL, 0.8, 0.0, 1.0, 70.0), 0.0);
        assert_eq!(power_requested_values(MODEL, 0.8, 1.0, 0.0, 70.0), 0.0);
    }

    #[test]
    fn power_law_is_symmetric_in_n_and_f() {
        let nf = power_requested_values(MODEL, 0.8, 0.2, 0.9, 70.0);
        let fn_ = power_requested_values(MODEL, 0.8, 0.9, 0.2, 70.0);
        assert!((nf - fn_).abs() <= 1e-15);
    }

    #[test]
    fn zero_order_limit_is_finite_for_positive_substrates() {
        let model = PowerLaw { p: 0.0, ..MODEL };
        let low = power_requested_values(model, 0.8, 1e-12, 1e-9, 70.0);
        let high = power_requested_values(model, 0.8, 10.0, 100.0, 70.0);
        assert!(low.is_finite() && low > 0.0);
        assert_eq!(low, high);
    }
}
