//! D-025 Gate 8: v7 surface-density constrained-radius Stage E reference and solver.

use crate::d013::{
    atomic_write_json, outcome_artifact, run_governed_reference, seal_artifact, D013RunConfig,
    GovernedRunOutcome,
};
use chemistry_core::D013_DEFAULT_REJECTION_STALL_LIMIT;
use crate::d025::{v7_base_params, D025_FROZEN_K_ADS};
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::d011_analysis::StageEReferenceRates;
use chemistry_core::d012_accounting::material_step_closes;
use chemistry_core::d020_analysis::restoring_sign_pattern_pass;
use chemistry_core::{
    bounded_joint_solver_d025, build_candidate_identity, converged_three_windows,
    d025_joint_balance_pass, is_biological_termination, is_numerical_termination,
    perturb_productive, clamp_productive_to_global, productive_rates_close,
    select_stage_e_conclusion, sensitivity_from_perturbations, stage_e_recovered,
    D025Conclusion, D025ProductiveRates, D025_CENTER_RADIUS, D025_DIAGNOSTIC_MAX_STEPS,
    D025_DIAGNOSTIC_WINDOW, D025_FULL_MAX_STEPS, D025_MAX_CANDIDATES, D025_MAX_SOLVER_ROUNDS,
    D025_NEIGHBOR_RADII, D025_REQUIRED_WINDOWS, D025_SENSITIVITY_PERTURB, D025_WINDOW, g_vector,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    Ok(chemistry_core::sha256_hex(&fs::read(std::env::current_exe()?)?))
}

fn analytical_productive(params: &SimParams) -> D025ProductiveRates {
    D025ProductiveRates {
        k_activation: params.k_d008_activation,
        k_rep: params.k_d008_reproduction,
        k_precursor: params.k_precursor,
        k_structure: params.k_d008_structure,
    }
}

fn apply_productive(params: &mut SimParams, productive: &D025ProductiveRates) {
    productive.apply_to_params(params);
    params.k_ads = D025_FROZEN_K_ADS;
}

fn legacy_rates(params: &SimParams) -> StageEReferenceRates {
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

fn balance_metrics_json(outcome: &GovernedRunOutcome) -> Value {
    let m = &outcome.metrics;
    json!({
        "Q_structure": m.structure.q,
        "Q_catalyst": m.catalyst.q,
        "Q_membrane": m.membrane.q,
        "Q_activated": m.activated.q,
        "g_structure": m.structure.g,
        "g_catalyst": m.catalyst.g,
        "g_membrane": m.membrane.g,
        "g_activated": m.activated.g,
        "catalyst_retention": m.catalyst_retention,
        "activated_retention": m.activated_retention,
        "membrane_localization": m.membrane_localization,
        "nutrient_influx": m.nutrient_influx,
        "fuel_influx": m.fuel_influx,
        "waste_efflux": m.waste_efflux,
    })
}

fn outcome_summary(outcome: &GovernedRunOutcome) -> Value {
    json!({
        "accepted_substeps": outcome.accepted_substeps,
        "simulated_time": outcome.simulated_time,
        "termination_reason": outcome.termination_reason,
        "scientific_classification": outcome.scientific_classification,
        "consecutive_qualifying": outcome.convergence.consecutive_qualifying,
        "required_windows": D025_REQUIRED_WINDOWS,
        "converged": converged_three_windows(outcome.convergence.consecutive_qualifying),
        "joint_balance_pass": d025_joint_balance_pass(&outcome.metrics),
        "balance_metrics": balance_metrics_json(outcome),
        "material_accounting_closed": material_step_closes(&outcome.material_accounting),
        "activation_relative_residual": outcome.activation_potential_accounting.relative_residual,
    })
}

fn run_governed_v7(
    productive: &D025ProductiveRates,
    radius: f64,
    max_steps: u64,
    window_size: u64,
    checkpoint_dir: Option<PathBuf>,
) -> Result<(GovernedRunOutcome, Value), Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    apply_productive(&mut params, productive);
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some(&format!("d025-stage-e-r{radius}")),
        None,
        "D-025 v7 governed Stage E reference",
        None,
        None,
    );
    let resume = checkpoint_dir.as_ref().and_then(|dir| {
        ["200000", "150000", "100000", "050000", "025000", "010000"]
            .iter()
            .map(|t| dir.join(format!("checkpoint_{t}.json")))
            .find(|p| p.exists())
    });
    let config = D013RunConfig {
        max_steps,
        window_size,
        radius,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir,
        resume_checkpoint: resume,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let rates = legacy_rates(&params);
    let mut artifact =
        outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
    artifact["project_directive"] = json!("D-025");
    artifact["equation_version"] = json!(EquationVersion::MembraneMetabolismV7SurfaceDensity);
    artifact["k_ads_frozen"] = json!(D025_FROZEN_K_ADS);
    artifact["enforce_structure_constraint"] = json!(true);
    artifact["productive_rates"] = json!(productive);
    artifact = seal_artifact(artifact)?;
    Ok((outcome, artifact))
}

fn classify_reference_outcome(outcome: &GovernedRunOutcome, max_steps: u64) -> D025Conclusion {
    if is_numerical_termination(outcome.termination_reason) {
        return D025Conclusion::D025NumericalFailure;
    }
    if is_biological_termination(outcome.termination_reason) {
        return D025Conclusion::D025Fail;
    }
    if !material_step_closes(&outcome.material_accounting)
        || outcome.activation_potential_accounting.relative_residual > 0.05
    {
        return D025Conclusion::D025AccountingFailure;
    }
    let converged = converged_three_windows(outcome.convergence.consecutive_qualifying);
    let joint = d025_joint_balance_pass(&outcome.metrics);
    select_stage_e_conclusion(
        false,
        false,
        converged,
        joint,
        outcome.accepted_substeps >= max_steps && !converged,
        false,
        false,
    )
}

pub fn run_stage_e_reference(output: &Path, diagnostic_only: bool) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;

    let params = v7_base_params()?;
    let analytical = analytical_productive(&params);
    let skip_diagnostic = !diagnostic_only
        && output.join("checkpoints/checkpoint_100000.json").exists()
        && output.join("diagnostic_result.json").exists();

    let (diag_summary, diag_conclusion) = if skip_diagnostic {
        let prior: Value = serde_json::from_slice(&fs::read(output.join("reference_terminal_classification.json"))?)?;
        let conclusion = prior["conclusion"]
            .as_str()
            .map(|s| match s {
                "D025_NUMERICAL_FAILURE" => D025Conclusion::D025NumericalFailure,
                "D025_ACCOUNTING_FAILURE" => D025Conclusion::D025AccountingFailure,
                _ => D025Conclusion::D025StageELongTransientUnresolved,
            })
            .unwrap_or(D025Conclusion::D025StageELongTransientUnresolved);
        (
            prior
                .get("diagnostic")
                .cloned()
                .unwrap_or(json!({"reused": true})),
            conclusion,
        )
    } else {
        let (diag_outcome, diag_artifact) = run_governed_v7(
            &analytical,
            D025_CENTER_RADIUS,
            D025_DIAGNOSTIC_MAX_STEPS,
            D025_DIAGNOSTIC_WINDOW,
            Some(checkpoint_dir.join("diagnostic")),
        )?;
        atomic_write_json(&output.join("diagnostic_result.json"), &diag_artifact)?;
        let conclusion = classify_reference_outcome(&diag_outcome, D025_DIAGNOSTIC_MAX_STEPS);
        (outcome_summary(&diag_outcome), conclusion)
    };

    if matches!(
        diag_conclusion,
        D025Conclusion::D025NumericalFailure | D025Conclusion::D025AccountingFailure
    ) {
        let body = json!({
            "project_directive": "D-025",
            "gate": 8,
            "source_commit": git_commit_hash()?,
            "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
            "k_ads_frozen": D025_FROZEN_K_ADS,
            "enforce_structure_constraint": true,
            "diagnostic": diag_summary,
            "conclusion": diag_conclusion.as_str(),
            "stage_e_recovered": false,
            "solver_skipped": true,
        });
        atomic_write_json(&output.join("reference_terminal_classification.json"), &body)?;
        atomic_write_json(&output.join("result.json"), &body)?;
        return Ok(body);
    }

    if diagnostic_only {
        let body = json!({
            "project_directive": "D-025",
            "gate": 8,
            "diagnostic_only": true,
            "diagnostic": diag_summary,
            "conclusion": diag_conclusion.as_str(),
        });
        atomic_write_json(&output.join("reference_terminal_classification.json"), &body)?;
        return Ok(body);
    }

    let (full_outcome, full_artifact) = run_governed_v7(
        &analytical,
        D025_CENTER_RADIUS,
        D025_FULL_MAX_STEPS,
        D025_WINDOW,
        Some(checkpoint_dir.clone()),
    )?;
    atomic_write_json(&output.join("result.json"), &full_artifact)?;

    let converged = converged_three_windows(full_outcome.convergence.consecutive_qualifying);
    let joint = d025_joint_balance_pass(&full_outcome.metrics);
    let contamination = if material_step_closes(&full_outcome.material_accounting) {
        0.0
    } else {
        1.0
    };

    let mut restoring = false;
    let mut neighbor_results = Vec::new();
    if converged && joint {
        for &radius in &D025_NEIGHBOR_RADII {
            let (n_out, n_art) = run_governed_v7(
                &analytical,
                radius,
                D025_FULL_MAX_STEPS,
                D025_WINDOW,
                Some(checkpoint_dir.join(format!("r{radius}"))),
            )?;
            atomic_write_json(&output.join(format!("r{radius}_result.json")), &n_art)?;
            neighbor_results.push(outcome_summary(&n_out));
        }
        let g18 = neighbor_results[0]["balance_metrics"]["g_structure"]
            .as_f64()
            .unwrap_or(0.0);
        let g22 = full_outcome.metrics.structure.g;
        let g26 = neighbor_results[1]["balance_metrics"]["g_structure"]
            .as_f64()
            .unwrap_or(0.0);
        restoring = restoring_sign_pattern_pass(g18, g22, g26);
        let neighbors = json!({
            "g_structure": {"R18": g18, "R22": g22, "R26": g26},
            "restoring_sign_pattern": restoring,
            "results": neighbor_results,
        });
        atomic_write_json(&output.join("radius_validation.json"), &neighbors)?;
    }

    let recovered = stage_e_recovered(
        converged,
        &full_outcome.metrics,
        material_step_closes(&full_outcome.material_accounting),
        material_step_closes(&full_outcome.material_accounting),
        contamination,
        restoring,
    );

    let conclusion = if recovered {
        D025Conclusion::D025StageERecovered
    } else {
        classify_reference_outcome(&full_outcome, D025_FULL_MAX_STEPS)
    };

    let body = json!({
        "project_directive": "D-025",
        "gate": 8,
        "source_commit": git_commit_hash()?,
        "binary_sha256": binary_hash()?,
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "k_ads_frozen": D025_FROZEN_K_ADS,
        "enforce_structure_constraint": true,
        "window_size": D025_WINDOW,
        "required_windows": D025_REQUIRED_WINDOWS,
        "max_steps": D025_FULL_MAX_STEPS,
        "productive_rates": analytical,
        "diagnostic": diag_summary,
        "reference": outcome_summary(&full_outcome),
        "constraint_contamination": contamination,
        "restoring_sign_pattern": restoring,
        "stage_e_recovered": recovered,
        "conclusion": conclusion.as_str(),
        "solver_recommended": !recovered
            && !matches!(
                conclusion,
                D025Conclusion::D025NumericalFailure
                    | D025Conclusion::D025AccountingFailure
                    | D025Conclusion::D025StageELongTransientUnresolved
            ),
    });
    atomic_write_json(&output.join("reference_terminal_classification.json"), &body)?;
    Ok(body)
}

pub fn run_stage_e_solve(
    output: &Path,
    reference_root: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let reference_root = resolve_path(reference_root);
    fs::create_dir_all(&output)?;

    let reference: Value =
        serde_json::from_slice(&fs::read(reference_root.join("reference_terminal_classification.json"))?)?;
    if reference["stage_e_recovered"].as_bool() == Some(true) {
        let body = json!({
            "project_directive": "D-025",
            "status": "ALREADY_RECOVERED",
            "conclusion": "D025_STAGE_E_RECOVERED",
        });
        atomic_write_json(&output.join("solver_report.json"), &body)?;
        return Ok(body);
    }

    let params = v7_base_params()?;
    let analytical = analytical_productive(&params);
    let mut current = analytical;

    let (base_outcome, _) = run_governed_v7(
        &current,
        D025_CENTER_RADIUS,
        D025_DIAGNOSTIC_MAX_STEPS,
        D025_DIAGNOSTIC_WINDOW,
        None,
    )?;
    let mut g_current = g_vector(&base_outcome.metrics);
    let mut g_up_rows = [[0.0; 4]; 4];
    let mut g_down_rows = [[0.0; 4]; 4];
    for idx in 0..4 {
        let up = clamp_productive_to_global(
            &perturb_productive(&analytical, idx, 1.0 + D025_SENSITIVITY_PERTURB),
            &analytical,
        );
        let down = clamp_productive_to_global(
            &perturb_productive(&analytical, idx, 1.0 - D025_SENSITIVITY_PERTURB),
            &analytical,
        );
        let (up_out, _) = run_governed_v7(
            &up,
            D025_CENTER_RADIUS,
            D025_DIAGNOSTIC_MAX_STEPS,
            D025_DIAGNOSTIC_WINDOW,
            None,
        )?;
        let (down_out, _) = run_governed_v7(
            &down,
            D025_CENTER_RADIUS,
            D025_DIAGNOSTIC_MAX_STEPS,
            D025_DIAGNOSTIC_WINDOW,
            None,
        )?;
        g_up_rows[idx] = g_vector(&up_out.metrics);
        g_down_rows[idx] = g_vector(&down_out.metrics);
    }
    let sensitivity = sensitivity_from_perturbations(&g_up_rows, &g_down_rows);
    let mut g_history = vec![g_current];
    let mut sens_history = vec![sensitivity.clone()];

    for round in 0..D025_MAX_SOLVER_ROUNDS {
        let Some(step) = chemistry_core::solve_bounded_joint_step_d025(
            &analytical,
            &current,
            g_current,
            &sensitivity,
            round,
        ) else {
            break;
        };
        let next = chemistry_core::apply_log_deltas(&analytical, &current, &step.rate_deltas_log);
        if productive_rates_close(&current, &next) {
            break;
        }
        current = next;
        let (out, _) = run_governed_v7(
            &current,
            D025_CENTER_RADIUS,
            D025_DIAGNOSTIC_MAX_STEPS,
            D025_DIAGNOSTIC_WINDOW,
            None,
        )?;
        g_current = g_vector(&out.metrics);
        g_history.push(g_current);
        sens_history.push(sensitivity.clone());
    }

    let solver = bounded_joint_solver_d025(&analytical, &analytical, &g_history, &sens_history);
    atomic_write_json(
        &output.join("solver_report.json"),
        &json!({
            "project_directive": "D-025",
            "analytical_rates": analytical,
            "sensitivity": sensitivity,
            "solver": solver,
        }),
    )?;

    let mut candidate_rows = Vec::new();
    let mut any_recovered = false;
    let mut productive = analytical;
    for (idx, cand) in solver.candidates.iter().enumerate().take(D025_MAX_CANDIDATES) {
        if cand.round > 0 {
            productive = chemistry_core::apply_log_deltas(&analytical, &productive, &cand.rate_deltas_log);
        }
        let cand_dir = output.join(format!("candidate_{idx}"));
        fs::create_dir_all(&cand_dir.join("checkpoints"))?;
        let (outcome, artifact) = run_governed_v7(
            &productive,
            D025_CENTER_RADIUS,
            D025_FULL_MAX_STEPS,
            D025_WINDOW,
            Some(cand_dir.join("checkpoints")),
        )?;
        atomic_write_json(&cand_dir.join("result.json"), &artifact)?;
        let converged = converged_three_windows(outcome.convergence.consecutive_qualifying);
        let joint = d025_joint_balance_pass(&outcome.metrics);
        let pass = stage_e_recovered(
            converged,
            &outcome.metrics,
            material_step_closes(&outcome.material_accounting),
            material_step_closes(&outcome.material_accounting),
            0.0,
            false,
        );
        any_recovered |= pass;
        candidate_rows.push(json!({
            "index": idx,
            "round": cand.round,
            "productive_rates": productive,
            "converged": converged,
            "joint_balance_pass": joint,
            "promotion_pass": pass,
            "summary": outcome_summary(&outcome),
        }));
    }
    atomic_write_json(&output.join("candidates.json"), &json!({ "candidates": candidate_rows }))?;

    let conclusion = if any_recovered {
        D025Conclusion::D025StageERecovered
    } else {
        D025Conclusion::D025StageENoJointFixedPoint
    };

    let body = json!({
        "project_directive": "D-025",
        "candidates_tested": candidate_rows.len(),
        "any_recovered": any_recovered,
        "conclusion": conclusion.as_str(),
    });
    atomic_write_json(&output.join("solve_terminal_classification.json"), &body)?;
    Ok(body)
}
