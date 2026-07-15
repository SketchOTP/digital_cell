//! D-011 transport-coupled constrained-radius balance assay runner.

use chemistry_core::{
    build_balance_metrics, build_candidate_identity, classify_convergence,
    field_sha256_stable, interface_weight, joint_overlap_pass, membrane_partition,
    quasi_steady_report, scientific_conclusion, sensitivity_matrix, stage_e_can_revise_to_pass,
    bounded_joint_solver, log_central_difference, total_mass, CandidateIdentity,
    ConvergenceClassification, D008StageMode, D011_DEFAULT_WINDOW, D011_HORIZON_RADII,
    D011_HORIZONS, D011_MAX_CANDIDATES, D011_REPLAY_RADII, EquationVersion, FieldSchemaVersion,
    JointBalanceMetrics, JointSolverReport, QuasiSteadyReport, SensitivityReport,
    SimParams, Simulation, SNAPSHOT_SCHEMA_VERSION, STAGE_E_FAILED_RATES, SteadyWindowSnapshot,
    StageEReferenceRates,
};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const D011_SEED: u64 = 1;

#[derive(Debug, serde::Deserialize)]
struct StageAReference {
    equation_version: EquationVersion,
    d_a: f64,
    beta_c: f64,
    beta_a: f64,
    beta_n: f64,
    beta_f: f64,
    beta_w: f64,
    m_max: f64,
    d_m: f64,
    k_membrane_decay: f64,
    k_membrane_detach: f64,
    k_c_membrane: f64,
    d008_a_max: f64,
    d008_c_max: f64,
}

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("sha256sum")
        .arg(std::env::current_exe()?)
        .output()?;
    if !output.status.success() {
        return Err("sha256sum failed".into());
    }
    let line = String::from_utf8(output.stdout)?;
    Ok(line.split_whitespace().next().unwrap_or("").to_string())
}

fn reference_toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/d008/stage_a_reference.toml")
}

fn reference_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let reference: StageAReference = toml::from_str(&fs::read_to_string(reference_toml_path())?)?;
    let mut params = SimParams::default();
    params.equation_version = reference.equation_version;
    params.d_a = reference.d_a;
    params.beta_c = reference.beta_c;
    params.beta_a = reference.beta_a;
    params.beta_n = reference.beta_n;
    params.beta_f = reference.beta_f;
    params.beta_w = reference.beta_w;
    params.m_max = reference.m_max;
    params.d_m = reference.d_m;
    params.k_membrane_decay = reference.k_membrane_decay;
    params.k_membrane_detach = reference.k_membrane_detach;
    params.k_c_membrane = reference.k_c_membrane;
    params.d008_a_max = reference.d008_a_max;
    params.d008_c_max = reference.d008_c_max;
    params.reactions_enabled = true;
    params.phase_separation_enabled = false;
    params.diffusion_enabled = true;
    Ok(params)
}

fn d011_params(rates: &StageEReferenceRates) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = reference_params()?;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.random_seed = D011_SEED;
    rates.apply_to(&mut params);
    Ok(params)
}

fn prepare_constrained_seed(sim: &mut Simulation, radius: f64) {
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let x = (idx % sim.grid.width) as f64 - sim.grid.cx;
        let y = (idx / sim.grid.width) as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        sim.fields.membrane[idx] = interface_weight(phi);
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.2;
            sim.fields.nutrient[idx] = 0.2;
            sim.fields.fuel[idx] = 0.2;
            sim.fields.waste[idx] = 0.5;
        } else {
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
        }
    }
}

fn interior_mean(sim: &Simulation, field: &[f64]) -> f64 {
    let mut total = 0.0;
    let mut area = 0.0_f64;
    for (idx, value) in field.iter().enumerate() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            total += value;
            area += 1.0;
        }
    }
    total / area.max(1.0)
}

fn retention(sim: &Simulation, field: &[f64]) -> f64 {
    let mut inside = 0.0;
    for (idx, value) in field.iter().enumerate() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            inside += value;
        }
    }
    inside / chemistry_core::field_mass(&sim.grid, field).max(f64::EPSILON)
}

fn soluble_max(sim: &Simulation) -> f64 {
    [
        &sim.fields.catalyst,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.fields.waste,
        &sim.fields.activated,
        &sim.fields.membrane,
    ]
    .into_iter()
    .flat_map(|field| field.iter().copied())
    .fold(0.0, f64::max)
}

fn window_snapshot(sim: &Simulation, start_step: u64, start_time: f64) -> SteadyWindowSnapshot {
    SteadyWindowSnapshot {
        start_step,
        end_step: sim.substep,
        simulated_time_start: start_time,
        simulated_time_end: sim.sim_time,
        mass_c: chemistry_core::field_mass(&sim.grid, &sim.fields.catalyst),
        mass_a: chemistry_core::field_mass(&sim.grid, &sim.fields.activated),
        mass_m: chemistry_core::field_mass(&sim.grid, &sim.fields.membrane),
        mean_n_interior: interior_mean(sim, &sim.fields.nutrient),
        mean_f_interior: interior_mean(sim, &sim.fields.fuel),
        mean_w_interior: interior_mean(sim, &sim.fields.waste),
        structure_production: sim.constraint_accounting.cumulative.virtual_production,
        structure_decay: sim.constraint_accounting.cumulative.virtual_decay,
        catalyst_reproduction: sim.metabolism_accounting.cumulative.reproduction,
        catalyst_turnover: sim.metabolism_accounting.cumulative.catalyst_turnover,
        membrane_synthesis: sim.membrane_accounting.cumulative.synthesis,
        membrane_loss: sim.membrane_accounting.cumulative.decay
            + sim.membrane_accounting.cumulative.detachment,
        activation: sim.metabolism_accounting.cumulative.activation,
        activated_loss: sim.metabolism_accounting.cumulative.activated_decay
            + sim.constraint_accounting.cumulative.virtual_production,
        nutrient_transport_interior: sim
            .transport_accounting
            .cumulative
            .nutrient
            .interior_net_flux_rate
            * sim.sim_time.max(1.0),
        fuel_transport_interior: sim
            .transport_accounting
            .cumulative
            .fuel
            .interior_net_flux_rate
            * sim.sim_time.max(1.0),
        waste_transport_interior: sim
            .transport_accounting
            .cumulative
            .waste
            .interior_net_flux_rate
            * sim.sim_time.max(1.0),
    }
}

pub struct D011RunConfig {
    pub max_steps: u64,
    pub window_size: u64,
    pub quick: bool,
}

pub struct D011RunOutcome {
    pub metrics: JointBalanceMetrics,
    pub quasi_steady: QuasiSteadyReport,
    pub classification: ConvergenceClassification,
    pub accepted_substeps: u64,
    pub simulated_time: f64,
    pub clean_termination: bool,
    pub windows: Vec<SteadyWindowSnapshot>,
}

pub fn run_constrained_assay(
    params: &SimParams,
    radius: f64,
    config: &D011RunConfig,
) -> D011RunOutcome {
    let mut sim = Simulation::new(params.clone());
    prepare_constrained_seed(&mut sim, radius);
    let window_size = config.window_size.max(1);
    let mut windows = Vec::new();
    let mut start_step = 0;
    let mut start_time = 0.0;
    let mut steps_done = 0u64;
    while steps_done < config.max_steps {
        let chunk = window_size.min(config.max_steps - steps_done);
        for _ in 0..chunk {
            if !sim.step() {
                break;
            }
        }
        steps_done += chunk;
        windows.push(window_snapshot(&sim, start_step, start_time));
        start_step = sim.substep;
        start_time = sim.sim_time;
    }
    let quasi = quasi_steady_report(&windows, window_size, 3);
    let partition =
        membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
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
    let classification = classify_convergence(
        &quasi,
        &metrics,
        total_mass(&sim.grid, &sim.fields.catalyst),
        total_mass(&sim.grid, &sim.fields.activated),
        total_mass(&sim.grid, &sim.fields.membrane),
        interior_mean(&sim, &sim.fields.nutrient),
        interior_mean(&sim, &sim.fields.fuel),
        soluble_max(&sim),
        sim.accounting.cumulative_within_tolerance(),
        sim.rejection_count as f64 / sim.substep.max(1) as f64,
    );
    let clean_termination =
        sim.substep == config.max_steps && sim.rejection_count == 0;
    D011RunOutcome {
        metrics,
        quasi_steady: quasi,
        classification,
        accepted_substeps: sim.substep,
        simulated_time: sim.sim_time,
        clean_termination,
        windows,
    }
}

fn metrics_json(metrics: &JointBalanceMetrics) -> Value {
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
        "joint_overlap": joint_overlap_pass(metrics),
    })
}

fn run_outcome_json(
    radius: f64,
    rates: &StageEReferenceRates,
    identity: &CandidateIdentity,
    source_commit: &str,
    binary_sha256: &str,
    config: &D011RunConfig,
    outcome: &D011RunOutcome,
    sim_fields: Option<&Simulation>,
) -> Value {
    let mut out = json!({
        "snapshot_schema_version": SNAPSHOT_SCHEMA_VERSION,
        "field_schema_version": FieldSchemaVersion::SevenFieldV1,
        "equation_version": EquationVersion::MembraneMetabolismV1,
        "d008_stage_mode": D008StageMode::ConstrainedRadius.as_str(),
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "source_commit": source_commit,
        "binary_sha256": binary_sha256,
        "seed": D011_SEED,
        "radius": radius,
        "selected_rates": rates,
        "accepted_substeps": outcome.accepted_substeps,
        "simulated_time": outcome.simulated_time,
        "max_steps": config.max_steps,
        "window_size": config.window_size,
        "quick_mode": config.quick,
        "convergence_classification": outcome.classification,
        "quasi_steady": outcome.quasi_steady,
        "balance_metrics": metrics_json(&outcome.metrics),
        "clean_termination": outcome.clean_termination,
    });
    if let Some(sim) = sim_fields {
        out["field_accounting"] = json!(sim.accounting);
        out["transport_accounting"] = json!(sim.transport_accounting);
        out["metabolism_accounting"] = json!(sim.metabolism_accounting);
        out["membrane_accounting"] = json!(sim.membrane_accounting);
        out["constraint_ledger"] = json!(sim.constraint_accounting);
        out["field_hashes"] = json!({
            "structure": field_sha256_stable(&sim.fields.structure),
            "catalyst": field_sha256_stable(&sim.fields.catalyst),
            "membrane": field_sha256_stable(&sim.fields.membrane),
        });
    }
    out
}

fn next_attempt(root: &Path, prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(root)?;
    for attempt in 1..=999 {
        let path = root.join(format!("{prefix}_{attempt:03}"));
        if !path.exists() {
            fs::create_dir_all(&path)?;
            return Ok(path);
        }
    }
    Err(format!("attempt namespace exhausted under {}", root.display()).into())
}

fn estimate_g_vector(outcome: &D011RunOutcome) -> [f64; 4] {
    [
        outcome.metrics.structure.g,
        outcome.metrics.catalyst.g,
        outcome.metrics.membrane.g,
        outcome.metrics.activated.g,
    ]
}

fn perturb_rates(base: &StageEReferenceRates, idx: usize, factor: f64) -> StageEReferenceRates {
    let mut rates = *base;
    let mut values = chemistry_core::rate_vector(&rates);
    values[idx] *= factor;
    rates.k_membrane = values[0];
    rates.k_d008_activation = values[1];
    rates.k_d008_reproduction = values[2];
    rates.k_d008_structure = values[3];
    rates.k_d008_activated_decay = values[4];
    rates.k_d008_catalyst_turnover = values[5];
    rates.k_structure_decay = values[6];
    rates
}

pub fn compute_sensitivity(
    base_rates: &StageEReferenceRates,
    radius: f64,
    config: &D011RunConfig,
) -> SensitivityReport {
    let base_params = d011_params(base_rates).expect("params");
    let base_outcome = run_constrained_assay(&base_params, radius, config);
    let g0 = estimate_g_vector(&base_outcome);
    let mut rows = [[0.0; 7]; 4];
    for idx in 0..7 {
        let up = perturb_rates(base_rates, idx, 1.0 + chemistry_core::D011_SENSITIVITY_PERTURB);
        let down = perturb_rates(base_rates, idx, 1.0 - chemistry_core::D011_SENSITIVITY_PERTURB);
        let up_outcome = run_constrained_assay(&d011_params(&up).expect("params"), radius, config);
        let down_outcome =
            run_constrained_assay(&d011_params(&down).expect("params"), radius, config);
        let g_up = estimate_g_vector(&up_outcome);
        let g_down = estimate_g_vector(&down_outcome);
        for row in 0..4 {
            rows[row][idx] = log_central_difference(g_up[row], g_down[row]);
        }
        let _ = g0;
    }
    sensitivity_matrix(&rows)
}

pub fn run_d011_protocol(
    root: &Path,
    config: &D011RunConfig,
) -> Result<Value, Box<dyn std::error::Error>> {
    let source_commit = git_commit_hash()?;
    let binary_sha256 = binary_hash()?;
    let attempt = next_attempt(root, "attempt")?;
    let rates = STAGE_E_FAILED_RATES;
    let params = d011_params(&rates)?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d011-constrained-radius"),
        None,
        "D-011 failed Stage E rate replay under transport-coupled constrained-radius assay",
        None,
        None,
    );

    let mut replay_results = Vec::new();
    let mut any_pass = false;
    for &radius in &D011_REPLAY_RADII {
        let outcome = run_constrained_assay(&params, radius, config);
        any_pass |= joint_overlap_pass(&outcome.metrics) && outcome.quasi_steady.converged;
        let row = run_outcome_json(
            radius,
            &rates,
            &identity,
            &source_commit,
            &binary_sha256,
            config,
            &outcome,
            None,
        );
        let replay_dir = attempt.join("failed_candidate_replay").join(format!("R{radius}"));
        fs::create_dir_all(&replay_dir)?;
        fs::write(replay_dir.join("result.json"), serde_json::to_vec_pretty(&row)?)?;
        replay_results.push(row);
    }

    let mut horizon_results = Map::new();
    for &radius in &D011_HORIZON_RADII {
        let mut radius_map = Map::new();
        for &horizon in &D011_HORIZONS {
            if horizon > config.max_steps && config.quick {
                continue;
            }
            let horizon_config = D011RunConfig {
                max_steps: horizon.min(config.max_steps),
                window_size: config.window_size,
                quick: config.quick,
            };
            let outcome = run_constrained_assay(&params, radius, &horizon_config);
            radius_map.insert(
                horizon.to_string(),
                run_outcome_json(
                    radius,
                    &rates,
                    &identity,
                    &source_commit,
                    &binary_sha256,
                    &horizon_config,
                    &outcome,
                    None,
                ),
            );
        }
        horizon_results.insert(radius.to_string(), Value::Object(radius_map));
    }

    let mut solver_report = JointSolverReport {
        rounds_attempted: 0,
        candidates: Vec::new(),
        bounded: true,
    };
    let mut sensitivity_reports = Map::new();
    if !any_pass {
        for radius in [22.0_f64, 26.0_f64] {
            let sensitivity = compute_sensitivity(&rates, radius, config);
            sensitivity_reports.insert(radius.to_string(), json!(sensitivity));
            let g = replay_results
                .iter()
                .find(|row| row["radius"].as_f64() == Some(radius))
                .map(|row| {
                    [
                        row["balance_metrics"]["g_structure"].as_f64().unwrap_or(0.0),
                        row["balance_metrics"]["g_catalyst"].as_f64().unwrap_or(0.0),
                        row["balance_metrics"]["g_membrane"].as_f64().unwrap_or(0.0),
                        row["balance_metrics"]["g_activated"].as_f64().unwrap_or(0.0),
                    ]
                })
                .unwrap_or([0.0; 4]);
            solver_report = bounded_joint_solver(&rates, &rates, &[g], &[sensitivity]);
            let solver_dir = attempt.join("bounded_joint_solver");
            fs::create_dir_all(&solver_dir)?;
            fs::write(
                solver_dir.join(format!("radius_{radius}.json")),
                serde_json::to_vec_pretty(&solver_report)?,
            )?;
        }
    }

    let mut validation_results = Vec::new();
    for candidate in solver_report.candidates.iter().take(D011_MAX_CANDIDATES) {
        let candidate_params = d011_params(&candidate.rates)?;
        let candidate_identity = build_candidate_identity(
            candidate_params.clone(),
            &source_commit,
            Some("d011-constrained-radius"),
            Some(candidate.round as u32),
            "D-011 bounded joint correction candidate",
            None,
            None,
        );
        let mut radius_rows = Vec::new();
        for &radius in &D011_REPLAY_RADII {
            let outcome = run_constrained_assay(&candidate_params, radius, config);
            any_pass |= joint_overlap_pass(&outcome.metrics) && outcome.quasi_steady.converged;
            radius_rows.push(run_outcome_json(
                radius,
                &candidate.rates,
                &candidate_identity,
                &source_commit,
                &binary_sha256,
                config,
                &outcome,
                None,
            ));
        }
        validation_results.push(json!({
            "round": candidate.round,
            "rates": candidate.rates,
            "log_change_norm": candidate.log_change_norm,
            "radius_results": radius_rows,
        }));
    }

    let stage_e_revised = stage_e_can_revise_to_pass(any_pass);
    let conclusion = scientific_conclusion(any_pass, stage_e_revised);
    let tag = if any_pass {
        "D-011-transport-coupled-balance-pass"
    } else {
        "D-011-transport-coupled-balance-fail"
    };

    let result = json!({
        "attempt_directory": attempt.file_name().and_then(|s| s.to_str()),
        "source_commit": source_commit,
        "binary_sha256": binary_sha256,
        "equation_version": EquationVersion::MembraneMetabolismV1,
        "field_schema_version": FieldSchemaVersion::SevenFieldV1,
        "d008_stage_mode": D008StageMode::ConstrainedRadius.as_str(),
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "selected_rates": rates,
        "stage_e_reference_rates": rates,
        "replay_radii": D011_REPLAY_RADII,
        "failed_candidate_replay": replay_results,
        "horizon_sensitivity": horizon_results,
        "sensitivity_reports": sensitivity_reports,
        "solver_report": solver_report,
        "validation_results": validation_results,
        "any_joint_overlap_pass": any_pass,
        "stage_e_revised_to_pass_after_d011": stage_e_revised,
        "scientific_conclusion": conclusion,
        "stage_e_prior_conclusion": "D008_NO_JOINT_FIXED_POINT",
        "result_tag": tag,
        "max_steps": config.max_steps,
        "window_size": config.window_size,
        "quick_mode": config.quick,
    });
    fs::write(attempt.join("result.json"), serde_json::to_vec_pretty(&result)?)?;
    Ok(result)
}

pub fn d011_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../experiments/generated/d011")
}
