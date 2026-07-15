//! D-012 v2 transport-coupled Stage E reference, solver, yield, and robustness runners.

use crate::d011::{
    estimate_g_vector, perturb_rates, prepare_constrained_seed, retention, run_constrained_assay,
    window_snapshot, D011RunConfig, D011RunOutcome,
};
use crate::d012::v2_stage_options;
use crate::d008;
use chemistry_core::{
    balance_calibration_score, bounded_joint_solver_v2, build_balance_metrics,
    build_candidate_identity, build_material_equivalent_step, classify_v2_stage_e,
    count_yield_changes, estimate_rates_from_metrics, expansion_radii_after_center,
    joint_overlap_pass, log_central_difference, membrane_partition, quasi_steady_report,
    restoring_radius_from_g_structure, select_calibration_factor, sensitivity_matrix,
    v2_stage_e_pass, yield_adjustment_allowed, apply_yield_change, D012_CALIBRATION_FACTORS, D012_DIAGNOSTIC_MAX_STEPS,
    D012_DIAGNOSTIC_WINDOW, D012_INITIAL_CAM_PERTURB, D012_MAX_YIELD_CANDIDATES,
    D012_RATE_PERTURB, D012_V2_CENTER_RADIUS, D012_V2_MAX_STEPS, D012_V2_NEIGHBOR_RADII,
    D012_V2_REQUIRED_WINDOWS, D012_V2_STAGE_E_RADII, D012_V2_WINDOW, D012_YIELD_CANDIDATES,
    D012RadiusBalancePoint, D012StageEClassification, D012V2CalibrationParameter, EquationVersion,
    FieldSchemaVersion, JointBalanceMetrics, SensitivityReport, SimParams,
    Simulation, SNAPSHOT_SCHEMA_VERSION, StageEReferenceRates, YieldComponent,
    STOICHIOMETRIC_SCHEMA_VERSION_V2, D011_SENSITIVITY_PERTURB,
};
use chemistry_core::config::D008StageMode;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const D012_SEED: u64 = 1;

fn d012_sensitivity_dir(reference_root: &Path) -> PathBuf {
    reference_root
        .parent()
        .map(|p| p.join("v2_sensitivity"))
        .unwrap_or_else(|| reference_root.join("../v2_sensitivity"))
}

#[derive(Debug, Clone)]
pub struct D012StageEConfig {
    pub max_steps: u64,
    pub window_size: u64,
    pub diagnostic: bool,
}

impl Default for D012StageEConfig {
    fn default() -> Self {
        Self {
            max_steps: D012_V2_MAX_STEPS,
            window_size: D012_V2_WINDOW,
            diagnostic: false,
        }
    }
}

impl D012StageEConfig {
    pub fn diagnostic() -> Self {
        Self {
            max_steps: D012_DIAGNOSTIC_MAX_STEPS,
            window_size: D012_DIAGNOSTIC_WINDOW,
            diagnostic: true,
        }
    }

    fn assay_config(&self) -> D011RunConfig {
        D011RunConfig {
            max_steps: self.max_steps,
            window_size: self.window_size,
            quick: self.diagnostic,
        }
    }
}

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    let path = std::env::current_exe().map_err(|err| format!("binary_sha256 failed: {err}"))?;
    let bytes = fs::read(&path).map_err(|err| format!("binary_sha256 failed: {err}"))?;
    Ok(chemistry_core::sha256_hex(&bytes))
}

fn stage_d_selected_toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/d008/stage_c_selected.toml")
}

fn v2_base_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = d008::stage_d_params_for(&v2_stage_options())?;
    params.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    params.eta_c = 1.0;
    params.eta_phi = 1.0;
    params.eta_m = 1.0;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.random_seed = D012_SEED;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    Ok(params)
}

fn rates_from_params(params: &SimParams) -> StageEReferenceRates {
    StageEReferenceRates {
        k_membrane: params.k_membrane,
        k_d008_activation: params.k_d008_activation,
        k_d008_reproduction: params.k_d008_reproduction,
        k_d008_structure: params.k_d008_structure,
        k_d008_activated_decay: params.k_d008_activated_decay,
        k_d008_catalyst_turnover: params.k_d008_catalyst_turnover,
        k_structure_decay: params.k_structure_decay,
    }
}

fn apply_rates(params: &mut SimParams, rates: &StageEReferenceRates) {
    rates.apply_to(params);
}

fn run_v2_assay(params: &SimParams, radius: f64, config: &D012StageEConfig) -> D011RunOutcome {
    run_constrained_assay(params, radius, &config.assay_config())
}

fn material_step_from_sim(sim: &Simulation) -> chemistry_core::MaterialEquivalentStep {
    build_material_equivalent_step(&sim.accounting.last_step)
}

fn outcome_to_json(
    radius: f64,
    rates: &StageEReferenceRates,
    params: &SimParams,
    identity: &chemistry_core::CandidateIdentity,
    source_commit: &str,
    binary_sha256: &str,
    config: &D012StageEConfig,
    outcome: &D011RunOutcome,
) -> Value {
    json!({
        "snapshot_schema_version": SNAPSHOT_SCHEMA_VERSION,
        "field_schema_version": FieldSchemaVersion::SevenFieldV1,
        "stoichiometric_schema_version": STOICHIOMETRIC_SCHEMA_VERSION_V2,
        "equation_version": EquationVersion::MembraneMetabolismV2Conservative,
        "d008_stage_mode": D008StageMode::ConstrainedRadius.as_str(),
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "source_commit": source_commit,
        "binary_sha256": binary_sha256,
        "seed": D012_SEED,
        "radius": radius,
        "selected_rates": rates,
        "accepted_substeps": outcome.accepted_substeps,
        "simulated_time": outcome.simulated_time,
        "max_steps": config.max_steps,
        "window_size": config.window_size,
        "diagnostic_mode": config.diagnostic,
        "convergence_classification": outcome.classification,
        "quasi_steady": outcome.quasi_steady,
        "balance_metrics": balance_metrics_json(&outcome.metrics),
        "clean_termination": outcome.clean_termination,
        "joint_overlap": joint_overlap_pass(&outcome.metrics),
    })
}

fn balance_metrics_json(metrics: &JointBalanceMetrics) -> Value {
    json!({
        "Q_structure": metrics.structure.q,
        "Q_catalyst": metrics.catalyst.q,
        "Q_membrane": metrics.membrane.q,
        "Q_activated": metrics.activated.q,
        "g_structure": metrics.structure.g,
        "g_catalyst": metrics.catalyst.g,
        "g_membrane": metrics.membrane.g,
        "g_activated": metrics.activated.g,
        "catalyst_retention": metrics.catalyst_retention,
        "activated_retention": metrics.activated_retention,
        "membrane_localization": metrics.membrane_localization,
        "nutrient_influx": metrics.nutrient_influx,
        "fuel_influx": metrics.fuel_influx,
        "waste_efflux": metrics.waste_efflux,
    })
}

fn atomic_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JobLedgerEntry {
    job_id: String,
    status: String,
    artifact_path: Option<String>,
    content_hash: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct JobLedger {
    entries: Vec<JobLedgerEntry>,
}

fn ledger_path(root: &Path) -> PathBuf {
    root.join("job_ledger.json")
}

fn load_ledger(root: &Path) -> JobLedger {
    let path = ledger_path(root);
    if path.exists() {
        serde_json::from_slice(&fs::read(&path).unwrap_or_default())
            .unwrap_or(JobLedger { entries: vec![] })
    } else {
        JobLedger { entries: vec![] }
    }
}

fn save_ledger(root: &Path, ledger: &JobLedger) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root)?;
    atomic_write_json(&ledger_path(root), &json!(ledger))
}

fn ledger_complete(ledger: &JobLedger, job_id: &str) -> bool {
    ledger.entries.iter().any(|e| e.job_id == job_id && e.status == "completed")
}

fn record_job(
    root: &Path,
    job_id: &str,
    artifact: &Path,
    value: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = load_ledger(root);
    let hash = chemistry_core::sha256_hex(&serde_json::to_vec(value)?);
    ledger.entries.retain(|e| e.job_id != job_id);
    ledger.entries.push(JobLedgerEntry {
        job_id: job_id.to_string(),
        status: "completed".to_string(),
        artifact_path: Some(artifact.display().to_string()),
        content_hash: Some(hash),
    });
    save_ledger(root, &ledger)
}

fn screen_v2_parameter(
    params: &SimParams,
    rates: &StageEReferenceRates,
    parameter: D012V2CalibrationParameter,
    config: &D012StageEConfig,
) -> (StageEReferenceRates, Value) {
    let baseline = *rates;
    let mut scores = [0.0; 3];
    let mut trials = Vec::new();
    for (idx, factor) in D012_CALIBRATION_FACTORS.iter().enumerate() {
        let mut trial_rates = baseline;
        parameter.set_value(&mut trial_rates, parameter.baseline_value(&baseline) * factor);
        let mut trial_params = params.clone();
        apply_rates(&mut trial_params, &trial_rates);
        let outcome = run_v2_assay(&trial_params, D012_V2_CENTER_RADIUS, config);
        scores[idx] = balance_calibration_score(&outcome.metrics);
        trials.push(json!({
            "factor": factor,
            "rates": trial_rates,
            "balance_score": scores[idx],
            "metrics": balance_metrics_json(&outcome.metrics),
        }));
    }
    let selected_idx = select_calibration_factor(scores);
    let mut selected = baseline;
    parameter.set_value(
        &mut selected,
        parameter.baseline_value(&baseline) * D012_CALIBRATION_FACTORS[selected_idx],
    );
    (
        selected,
        json!({
            "parameter": format!("{parameter:?}"),
            "baseline_value": parameter.baseline_value(&baseline),
            "selected_factor": D012_CALIBRATION_FACTORS[selected_idx],
            "selected_value": parameter.baseline_value(&selected),
            "scores": scores,
            "trials": trials,
        }),
    )
}

fn compute_v2_sensitivity(
    params: &SimParams,
    rates: &StageEReferenceRates,
    config: &D012StageEConfig,
) -> SensitivityReport {
    let sens_config = D012StageEConfig {
        max_steps: config.max_steps.min(D012_DIAGNOSTIC_MAX_STEPS),
        window_size: config.window_size.min(D012_DIAGNOSTIC_WINDOW),
        diagnostic: true,
    };
    let base_outcome = run_v2_assay(params, D012_V2_CENTER_RADIUS, &sens_config);
    let g0 = estimate_g_vector(&base_outcome);
    let mut rows = [[0.0; 4]; 4];
    for idx in 0..4 {
        let up = perturb_rates(rates, idx, 1.0 + D011_SENSITIVITY_PERTURB);
        let down = perturb_rates(rates, idx, 1.0 - D011_SENSITIVITY_PERTURB);
        let mut up_params = params.clone();
        let mut down_params = params.clone();
        apply_rates(&mut up_params, &up);
        apply_rates(&mut down_params, &down);
        let up_outcome = run_v2_assay(&up_params, D012_V2_CENTER_RADIUS, &sens_config);
        let down_outcome = run_v2_assay(&down_params, D012_V2_CENTER_RADIUS, &sens_config);
        let g_up = estimate_g_vector(&up_outcome);
        let g_down = estimate_g_vector(&down_outcome);
        for row in 0..4 {
            rows[row][idx] = log_central_difference(g_up[row], g_down[row]);
        }
        let _ = g0;
    }
    sensitivity_matrix(&rows)
}

fn restoring_points_from_radii(
    radii_results: &[(f64, D011RunOutcome)],
) -> Vec<D012RadiusBalancePoint> {
    radii_results
        .iter()
        .map(|(radius, outcome)| D012RadiusBalancePoint {
            radius: *radius,
            g_structure: outcome.metrics.structure.g,
            joint_overlap: joint_overlap_pass(&outcome.metrics),
            quasi_steady: outcome.quasi_steady.converged,
        })
        .collect()
}

fn run_radii_batch(
    params: &SimParams,
    rates: &StageEReferenceRates,
    radii: &[f64],
    config: &D012StageEConfig,
    root: &Path,
    prefix: &str,
    identity: &chemistry_core::CandidateIdentity,
    source_commit: &str,
    binary_sha256: &str,
) -> Result<Vec<(f64, D011RunOutcome, Value)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for &radius in radii {
        let job_id = format!("{prefix}_R{radius}_{}_{}", config.max_steps, config.window_size);
        let artifact = root.join(format!("{prefix}/R{radius}/result.json"));
        if ledger_complete(&load_ledger(root), &job_id) && artifact.exists() {
            let cached: Value = serde_json::from_slice(&fs::read(&artifact)?)?;
            let outcome = D011RunOutcome {
                metrics: parse_metrics(&cached["balance_metrics"]),
                quasi_steady: serde_json::from_value(cached["quasi_steady"].clone())?,
                classification: serde_json::from_value(cached["convergence_classification"].clone())?,
                accepted_substeps: cached["accepted_substeps"].as_u64().unwrap_or(0),
                simulated_time: cached["simulated_time"].as_f64().unwrap_or(0.0),
                clean_termination: cached["clean_termination"].as_bool().unwrap_or(false),
                windows: vec![],
            };
            results.push((radius, outcome, cached));
            continue;
        }
        let outcome = run_v2_assay(params, radius, config);
        let row = outcome_to_json(
            radius,
            rates,
            params,
            identity,
            source_commit,
            binary_sha256,
            config,
            &outcome,
        );
        fs::create_dir_all(artifact.parent().unwrap())?;
        atomic_write_json(&artifact, &row)?;
        record_job(root, &job_id, &artifact, &row)?;
        results.push((radius, outcome, row));
    }
    Ok(results)
}

fn parse_metrics(value: &Value) -> JointBalanceMetrics {
    let q = |key: &str| value[key].as_f64().unwrap_or(0.0);
    JointBalanceMetrics {
        structure: chemistry_core::ComponentBalance {
            q: q("Q_structure"),
            g: q("g_structure"),
            production: 0.0,
            loss: 0.0,
        },
        catalyst: chemistry_core::ComponentBalance {
            q: q("Q_catalyst"),
            g: q("g_catalyst"),
            production: 0.0,
            loss: 0.0,
        },
        membrane: chemistry_core::ComponentBalance {
            q: q("Q_membrane"),
            g: q("g_membrane"),
            production: 0.0,
            loss: 0.0,
        },
        activated: chemistry_core::ComponentBalance {
            q: q("Q_activated"),
            g: q("g_activated"),
            production: 0.0,
            loss: 0.0,
        },
        catalyst_retention: value["catalyst_retention"].as_f64().unwrap_or(0.0),
        activated_retention: value["activated_retention"].as_f64().unwrap_or(0.0),
        membrane_localization: value["membrane_localization"].as_f64().unwrap_or(0.0),
        nutrient_influx: value["nutrient_influx"].as_f64().unwrap_or(0.0),
        fuel_influx: value["fuel_influx"].as_f64().unwrap_or(0.0),
        waste_efflux: value["waste_efflux"].as_f64().unwrap_or(0.0),
    }
}

pub fn run_v2_stage_e_reference(
    root: &Path,
    config: &D012StageEConfig,
) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(root)?;
    let source_commit = git_commit_hash()?;
    let binary_sha256 = binary_hash()?;
    let mut params = v2_base_params()?;
    let mut rates = rates_from_params(&params);

    // Initial ledger-based estimate at center R=22.
    let estimate_outcome = run_v2_assay(&params, D012_V2_CENTER_RADIUS, config);
    rates = estimate_rates_from_metrics(&rates, &estimate_outcome.metrics);

    let mut calibration_screens = Map::new();
    for parameter in D012V2CalibrationParameter::ORDER {
        let (selected, screen) = screen_v2_parameter(&params, &rates, parameter, config);
        rates = selected;
        calibration_screens.insert(format!("{parameter:?}"), screen);
    }
    apply_rates(&mut params, &rates);

    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d012-v2-stage-e-reference"),
        None,
        "D-012 v2 transport-coupled Stage E reference assay",
        None,
        None,
    );

    // Sequential expansion: center → neighbors → full grid if center promising.
    let center_outcome = run_v2_assay(&params, D012_V2_CENTER_RADIUS, config);
    let center_pass = joint_overlap_pass(&center_outcome.metrics) && center_outcome.quasi_steady.converged;
    let mut radii_to_run = vec![D012_V2_CENTER_RADIUS];
    radii_to_run.extend(expansion_radii_after_center(center_pass));
    if center_pass {
        for &r in &D012_V2_STAGE_E_RADII {
            if !radii_to_run.contains(&r) {
                radii_to_run.push(r);
            }
        }
    }
    radii_to_run.sort_by(|a, b| a.partial_cmp(b).unwrap());
    radii_to_run.dedup();

    let batch = run_radii_batch(
        &params,
        &rates,
        &radii_to_run,
        config,
        root,
        "reference",
        &identity,
        &source_commit,
        &binary_sha256,
    )?;
    let radii_pairs: Vec<(f64, D011RunOutcome)> =
        batch.iter().map(|(r, o, _)| (*r, o.clone())).collect();
    let restoring_points = restoring_points_from_radii(&radii_pairs);
    let center = batch
        .iter()
        .find(|(r, _, _)| (*r - D012_V2_CENTER_RADIUS).abs() < 1e-6)
        .map(|(_, o, _)| o)
        .unwrap_or(&center_outcome);

    let mut sim = Simulation::new(params.clone());
    prepare_constrained_seed(&mut sim, D012_V2_CENTER_RADIUS);
    for _ in 0..center.accepted_substeps.min(config.max_steps) {
        if !sim.step() {
            break;
        }
    }
    let material = material_step_from_sim(&sim);
    let stage_pass = v2_stage_e_pass(
        &center.quasi_steady,
        &center.metrics,
        &material,
        &restoring_points,
    );
    let classification = classify_v2_stage_e(
        &center.quasi_steady,
        &center.metrics,
        sim.accounting.cumulative_within_tolerance(),
        &restoring_points,
        center.classification,
        center.accepted_substeps >= config.max_steps,
    );

    let sensitivity = compute_v2_sensitivity(&params, &rates, config);
    let sens_dir = d012_sensitivity_dir(root);
    fs::create_dir_all(&sens_dir)?;
    atomic_write_json(
        &sens_dir.join("center_R22.json"),
        &json!(sensitivity),
    )?;

    let result = json!({
        "source_commit": source_commit,
        "binary_sha256": binary_sha256,
        "equation_version": EquationVersion::MembraneMetabolismV2Conservative,
        "stoichiometric_schema_version": STOICHIOMETRIC_SCHEMA_VERSION_V2,
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "selected_rates": rates,
        "calibration_screens": calibration_screens,
        "radii": radii_to_run,
        "radius_results": batch.iter().map(|(r, o, row)| json!({
            "radius": r,
            "result": row,
            "joint_overlap": joint_overlap_pass(&o.metrics),
            "quasi_steady": o.quasi_steady.converged,
        })).collect::<Vec<_>>(),
        "restoring_points": restoring_points,
        "restoring_radius_pass": restoring_radius_from_g_structure(&restoring_points),
        "center_balance_metrics": balance_metrics_json(&center.metrics),
        "material_accounting": material,
        "stage_e_pass": stage_pass,
        "stage_classification": format!("{classification:?}"),
        "max_steps": config.max_steps,
        "window_size": config.window_size,
        "diagnostic_mode": config.diagnostic,
        "sensitivity": sensitivity,
        "stage_c_selected_toml": stage_d_selected_toml_path().display().to_string(),
    });
    atomic_write_json(&root.join("result.json"), &result)?;
    Ok(result)
}

pub fn run_v2_joint_solver(
    root: &Path,
    reference_root: &Path,
    config: &D012StageEConfig,
) -> Result<Value, Box<dyn std::error::Error>> {
    let reference: Value =
        serde_json::from_slice(&fs::read(reference_root.join("result.json"))?)?;
    let mut params = v2_base_params()?;
    let mut rates: StageEReferenceRates = serde_json::from_value(reference["selected_rates"].clone())?;
    apply_rates(&mut params, &rates);
    let reference_rates = rates;

    let sensitivity = compute_v2_sensitivity(&params, &rates, config);
    let center_outcome = run_v2_assay(&params, D012_V2_CENTER_RADIUS, config);
    let g = estimate_g_vector(&center_outcome);
    let solver_report = bounded_joint_solver_v2(&reference_rates, &rates, &[g], &[sensitivity.clone()]);

    fs::create_dir_all(root)?;
    atomic_write_json(&root.join("solver_report.json"), &json!(solver_report))?;

    let source_commit = git_commit_hash()?;
    let binary_sha256 = binary_hash()?;
    let mut validation = Vec::new();
    let mut any_pass = false;

    for candidate in solver_report.candidates.iter().filter(|c| c.round > 0) {
        let mut candidate_params = params.clone();
        apply_rates(&mut candidate_params, &candidate.rates);
        let identity = build_candidate_identity(
            candidate_params.clone(),
            &source_commit,
            Some("d012-v2-joint-candidate"),
            Some(candidate.round as u32),
            "D-012 v2 bounded joint correction candidate",
            None,
            None,
        );
        let radii = [D012_V2_CENTER_RADIUS, D012_V2_NEIGHBOR_RADII[0], D012_V2_NEIGHBOR_RADII[1]];
        let batch = run_radii_batch(
            &candidate_params,
            &candidate.rates,
            &radii,
            config,
            root,
            &format!("candidate_round_{}", candidate.round),
            &identity,
            &source_commit,
            &binary_sha256,
        )?;
        let restoring = restoring_points_from_radii(
            &batch.iter().map(|(r, o, _)| (*r, o.clone())).collect::<Vec<_>>(),
        );
        let center = batch
            .iter()
            .find(|(r, _, _)| (*r - D012_V2_CENTER_RADIUS).abs() < 1e-6)
            .map(|(_, o, _)| o)
            .expect("center");
        let pass = joint_overlap_pass(&center.metrics) && center.quasi_steady.converged;
        any_pass |= pass;
        validation.push(json!({
            "round": candidate.round,
            "rates": candidate.rates,
            "log_change_norm": candidate.log_change_norm,
            "joint_overlap_at_center": pass,
            "restoring_radius": restoring_radius_from_g_structure(&restoring),
            "balance_metrics": balance_metrics_json(&center.metrics),
        }));
    }

    let result = json!({
        "reference_root": reference_root.display().to_string(),
        "solver_report": solver_report,
        "validation_results": validation,
        "any_joint_overlap_pass": any_pass,
        "sensitivity": sensitivity,
    });
    atomic_write_json(&root.join("result.json"), &result)?;
    Ok(result)
}

pub fn run_v2_yield_candidates(
    root: &Path,
    diagnosis_root: &Path,
    config: &D012StageEConfig,
) -> Result<Value, Box<dyn std::error::Error>> {
    let diagnosis: Value = serde_json::from_slice(&fs::read(diagnosis_root.join("result.json"))?)?;
    let metrics = &diagnosis["center_balance_metrics"];
    let mut params = v2_base_params()?;
    let rates: StageEReferenceRates = serde_json::from_value(diagnosis["selected_rates"].clone())?;
    apply_rates(&mut params, &rates);

    let overproduced = [
        (YieldComponent::Catalyst, metrics["Q_catalyst"].as_f64().unwrap_or(1.0), params.eta_c),
        (YieldComponent::Structure, metrics["Q_structure"].as_f64().unwrap_or(1.0), params.eta_phi),
        (YieldComponent::Membrane, metrics["Q_membrane"].as_f64().unwrap_or(1.0), params.eta_m),
    ]
    .into_iter()
    .filter(|(_, q, _)| *q > 1.02)
    .collect::<Vec<_>>();

    fs::create_dir_all(root)?;
    if overproduced.is_empty() {
        let result = json!({
            "skipped": true,
            "reason": "no_ledger_supported_overproduction",
        });
        atomic_write_json(&root.join("result.json"), &result)?;
        return Ok(result);
    }

    let mut candidates = Vec::new();
    for (component, q, current_eta) in overproduced.into_iter().take(D012_MAX_YIELD_CANDIDATES) {
        for &eta in &D012_YIELD_CANDIDATES {
            if eta >= current_eta {
                continue;
            }
            let balance = chemistry_core::ComponentBalance { q, g: 0.0, production: 0.0, loss: 0.0 };
            if !yield_adjustment_allowed(balance, current_eta, eta) {
                continue;
            }
            let before = (params.eta_c, params.eta_phi, params.eta_m);
            let mut trial = params.clone();
            chemistry_core::apply_yield_change(&mut trial, component, eta)?;
            let outcome = run_v2_assay(&trial, D012_V2_CENTER_RADIUS, config);
            candidates.push(json!({
                "component": format!("{component:?}"),
                "eta": eta,
                "previous_eta": current_eta,
                "yield_changes": count_yield_changes(before, (trial.eta_c, trial.eta_phi, trial.eta_m)),
                "Q_before": q,
                "balance_metrics": balance_metrics_json(&outcome.metrics),
                "joint_overlap": joint_overlap_pass(&outcome.metrics),
            }));
            if candidates.len() >= D012_MAX_YIELD_CANDIDATES {
                break;
            }
        }
        if candidates.len() >= D012_MAX_YIELD_CANDIDATES {
            break;
        }
    }

    let result = json!({
        "skipped": false,
        "candidates": candidates,
    });
    atomic_write_json(&root.join("result.json"), &result)?;
    Ok(result)
}

pub fn run_v2_robust_overlap(
    root: &Path,
    candidate_root: &Path,
    config: &D012StageEConfig,
) -> Result<Value, Box<dyn std::error::Error>> {
    let candidate: Value = serde_json::from_slice(&fs::read(candidate_root.join("result.json"))?)?;
    let rates: StageEReferenceRates = serde_json::from_value(candidate["selected_rates"].clone())?;
    let mut params = v2_base_params()?;
    apply_rates(&mut params, &rates);

    fs::create_dir_all(root)?;
    let source_commit = git_commit_hash()?;
    let binary_sha256 = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d012-v2-robust"),
        None,
        "D-012 v2 Stage E robustness overlap",
        None,
        None,
    );

    let mut perturbations = Vec::new();
    for idx in 0..4 {
        for &sign in &[1.0, -1.0] {
            let factor = 1.0 + sign * D012_RATE_PERTURB;
            let perturbed = perturb_rates(&rates, idx, factor);
            let mut trial = params.clone();
            apply_rates(&mut trial, &perturbed);
            let outcome = run_v2_assay(&trial, D012_V2_CENTER_RADIUS, config);
            perturbations.push(json!({
                "kind": "rate",
                "rate_index": idx,
                "factor": factor,
                "joint_overlap": joint_overlap_pass(&outcome.metrics),
                "metrics": balance_metrics_json(&outcome.metrics),
            }));
        }
    }

    for &(label, c_scale, a_scale, m_scale) in &[
        ("cam_up", 1.0 + D012_INITIAL_CAM_PERTURB, 1.0 + D012_INITIAL_CAM_PERTURB, 1.0 + D012_INITIAL_CAM_PERTURB),
        ("cam_down", 1.0 - D012_INITIAL_CAM_PERTURB, 1.0 - D012_INITIAL_CAM_PERTURB, 1.0 - D012_INITIAL_CAM_PERTURB),
    ] {
        let mut sim = Simulation::new(params.clone());
        prepare_constrained_seed(&mut sim, D012_V2_CENTER_RADIUS);
        for idx in 0..sim.fields.structure.len() {
            if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
                sim.fields.catalyst[idx] *= c_scale;
                sim.fields.activated[idx] *= a_scale;
                sim.fields.membrane[idx] *= m_scale;
            }
        }
        let mut windows = Vec::new();
        let window = config.window_size.max(1);
        let mut steps = 0u64;
        while steps < config.max_steps {
            let chunk = window.min(config.max_steps - steps);
            for _ in 0..chunk {
                if !sim.step() {
                    break;
                }
            }
            steps += chunk;
            windows.push(window_snapshot(&sim, 0, 0.0));
        }
        let quasi = quasi_steady_report(&windows, window, D012_V2_REQUIRED_WINDOWS);
        let partition = membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
        let metrics = build_balance_metrics(
            sim.sim_time,
            &sim.constraint_accounting.cumulative,
            &sim.metabolism_accounting.cumulative,
            &sim.membrane_accounting.cumulative,
            &sim.transport_accounting.cumulative,
            retention(&sim, &sim.fields.catalyst),
            retention(&sim, &sim.fields.activated),
            partition.localization_fraction,
        );
        perturbations.push(json!({
            "kind": "initial_cam",
            "label": label,
            "joint_overlap": joint_overlap_pass(&metrics),
            "quasi_steady": quasi.converged,
            "metrics": balance_metrics_json(&metrics),
        }));
    }

    let radii_batch = run_radii_batch(
        &params,
        &rates,
        &D012_V2_NEIGHBOR_RADII.to_vec(),
        config,
        root,
        "robust_restoring",
        &identity,
        &source_commit,
        &binary_sha256,
    )?;
    let restoring = restoring_points_from_radii(
        &radii_batch.iter().map(|(r, o, _)| (*r, o.clone())).collect::<Vec<_>>(),
    );

    let all_rate_pass = perturbations
        .iter()
        .filter(|p| p["kind"] == "rate")
        .all(|p| p["joint_overlap"].as_bool() == Some(true));
    let result = json!({
        "candidate_root": candidate_root.display().to_string(),
        "perturbations": perturbations,
        "restoring_radius_pass": restoring_radius_from_g_structure(&restoring),
        "rate_robust_overlap": all_rate_pass,
        "restoring_points": restoring,
    });
    atomic_write_json(&root.join("result.json"), &result)?;
    Ok(result)
}
