//! D-021 interface-protected membrane retention/localization repair runner.

use crate::d011::{
    prepare_constrained_seed, run_constrained_assay, D011RunConfig, D011RunOutcome,
};
use crate::d013::{
    atomic_write_json, outcome_artifact, run_governed_reference, seal_artifact, D013RunConfig,
};
use crate::d015::frozen_organism_params;
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::membrane::membrane_rates;
use chemistry_core::{
    bounded_joint_solver_d021, build_candidate_identity, clamp_rates_to_global_bounds_d021,
    evaluate_local_mechanism_gate, evaluate_retention_localization, freeze_nonproductive_rates,
    g_vector, joint_flow_score, prebalance_promotion_gate, rates_within_global_bounds_d021,
    restoring_sign_pattern_pass, select_d021_conclusion, sensitivity_matrix, sha256_hex,
    D021Conclusion, D021_ANALYTICAL_V4_RATES, D021_CENTER_RADIUS, D021_DIAGNOSTIC_MAX_STEPS,
    D021_DIAGNOSTIC_WINDOW, D021_EPS_CANDIDATES, D021_FULL_WINDOW, D021_LOCALIZATION_MIN,
    D021_MAX_CANDIDATES, D021_MAX_SOLVER_ROUNDS, D021_NEIGHBOR_RADII, D021_RETENTION_MIN,
    D013_DEFAULT_REJECTION_STALL_LIMIT, JointBalanceMetrics, MEMBRANE_SCHEMA_VERSION_V2,
    StageEReferenceRates, STRUCTURAL_SCHEMA_VERSION_V3, V3_SELECTED_MECHANISM,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const D021_SEED: u64 = 1;

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
    Ok(sha256_hex(&fs::read(std::env::current_exe()?)?))
}

pub fn v4_params_with_rates(
    rates: &StageEReferenceRates,
    eps_m: f64,
) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = frozen_organism_params(true)?;
    params.equation_version = EquationVersion::MembraneMetabolismV4InterfaceProtected;
    params.d019_mechanism_probe = None;
    params.eps_m = eps_m;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.random_seed = D021_SEED;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    rates.apply_to(&mut params);
    Ok(params)
}

fn analytical_rates() -> StageEReferenceRates {
    D021_ANALYTICAL_V4_RATES
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
        "joint_flow_score": joint_flow_score(metrics),
    })
}

fn run_v4_assay(
    rates: &StageEReferenceRates,
    eps_m: f64,
    radius: f64,
    max_steps: u64,
    window: u64,
) -> Result<D011RunOutcome, Box<dyn std::error::Error>> {
    let params = v4_params_with_rates(rates, eps_m)?;
    Ok(run_constrained_assay(
        &params,
        radius,
        &D011RunConfig {
            max_steps,
            window_size: window,
            quick: max_steps <= D021_DIAGNOSTIC_MAX_STEPS,
        },
    ))
}

/// Gate 1: local mechanism + Stage B localization for each ε.
pub fn run_gate1_eps_screen(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let rates = analytical_rates();
    let mut candidates = Vec::new();
    for &eps in &D021_EPS_CANDIDATES {
        let params = v4_params_with_rates(&rates, eps)?;
        let local = evaluate_local_mechanism_gate(0.5, 0.0, 0.4, 0.3, 0.5, &params);
        let stage_b_dir = output.join(format!("stage_b_eps_{eps}"));
        let stage_b = if stage_b_dir.join("result.json").exists() {
            serde_json::from_slice(&fs::read(stage_b_dir.join("result.json"))?)?
        } else {
            crate::d012::run_v4_stage_b(&stage_b_dir, eps)?
        };
        let localization = stage_b["localization"]["minimum_after_transient"]
            .as_f64()
            .or_else(|| {
                stage_b["selected"]["minimum_after_transient"]
                    .as_f64()
                    .or_else(|| stage_b["localization_minimum_after_transient"].as_f64())
            })
            .unwrap_or(0.0);
        // Fallback: read from candidates if present.
        let localization = if localization > 0.0 {
            localization
        } else {
            stage_b["candidates"]
                .as_array()
                .and_then(|arr| {
                    arr.iter()
                        .filter_map(|c| c["minimum_after_transient"].as_f64())
                        .fold(None, |acc: Option<f64>, v| {
                            Some(acc.map_or(v, |a| a.min(v)))
                        })
                })
                .unwrap_or(0.0)
        };
        let stage_pass = stage_b["stage_classification"]
            .as_str()
            .map(|s| s.contains("STAGE_B_LOCALIZATION_PASS"))
            .unwrap_or(false)
            || localization >= D021_LOCALIZATION_MIN;
        let pass = local.all_pass() && stage_pass && localization >= D021_LOCALIZATION_MIN;
        candidates.push(json!({
            "eps_m": eps,
            "local_mechanism": {
                "production_positive": local.production_positive,
                "loss_positive": local.loss_positive,
                "interface_turnover_possible": local.interface_turnover_possible,
                "faster_off_interface": local.faster_off_interface,
                "local_only": local.local_only,
                "all_pass": local.all_pass(),
            },
            "stage_b_classification": stage_b["stage_classification"],
            "localization_minimum": localization,
            "gate1_pass": pass,
            "membrane_schema_version": MEMBRANE_SCHEMA_VERSION_V2,
            "equation_version": EquationVersion::MembraneMetabolismV4InterfaceProtected.as_str(),
        }));
    }
    let any_pass = candidates.iter().any(|c| c["gate1_pass"].as_bool() == Some(true));
    let body = json!({
        "project_directive": "D-021",
        "gate": 1,
        "eps_candidates": candidates,
        "any_pass": any_pass,
        "stop_if_none": !any_pass,
    });
    atomic_write_json(&output.join("gate1_eps_screen.json"), &body)?;
    Ok(body)
}

/// Gate 2: fixed-compartment Stage D for Gate-1 passers.
pub fn run_gate2_fixed_compartment(
    output: &Path,
    gate1: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut results = Vec::new();
    let empty = Vec::new();
    let cands = gate1["eps_candidates"].as_array().unwrap_or(&empty);
    for c in cands {
        if c["gate1_pass"].as_bool() != Some(true) {
            continue;
        }
        let eps = c["eps_m"].as_f64().unwrap_or(0.05);
        let stage_d_dir = output.join(format!("stage_d_eps_{eps}"));
        let stage_d = if stage_d_dir.join("result.json").exists() {
            serde_json::from_slice(&fs::read(stage_d_dir.join("result.json"))?)?
        } else {
            crate::d012::run_v4_stage_d(&stage_d_dir, eps)?
        };
        let pass = stage_d["stage_classification"]
            .as_str()
            .map(|s| s.contains("STAGE_D") && s.contains("PASS"))
            .unwrap_or(false);
        results.push(json!({
            "eps_m": eps,
            "stage_d": stage_d,
            "gate2_pass": pass,
        }));
    }
    let any_pass = results.iter().any(|r| r["gate2_pass"].as_bool() == Some(true));
    let regression = !results.is_empty() && !any_pass;
    let body = json!({
        "project_directive": "D-021",
        "gate": 2,
        "results": results,
        "any_pass": any_pass,
        "fixed_compartment_regression": regression,
    });
    atomic_write_json(&output.join("gate2_fixed_compartment.json"), &body)?;
    Ok(body)
}

/// Gate 3: R22 pre-balance short screens with frozen productive rates.
pub fn run_gate3_prebalance(
    output: &Path,
    gate2: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let rates = analytical_rates();
    // v3 baseline for comparison (uniform membrane decay).
    let mut v3 = frozen_organism_params(true)?;
    v3.equation_version = EquationVersion::MembraneMetabolismV3StructuralScaling;
    v3.d008_stage_mode = D008StageMode::ConstrainedRadius;
    rates.apply_to(&mut v3);
    let baseline = run_constrained_assay(
        &v3,
        D021_CENTER_RADIUS,
        &D011RunConfig {
            max_steps: D021_DIAGNOSTIC_MAX_STEPS,
            window_size: D021_DIAGNOSTIC_WINDOW,
            quick: true,
        },
    );
    let mut screens = Vec::new();
    let empty = Vec::new();
    let results = gate2["results"].as_array().unwrap_or(&empty);
    for r in results {
        if r["gate2_pass"].as_bool() != Some(true) {
            continue;
        }
        let eps = r["eps_m"].as_f64().unwrap_or(0.05);
        let outcome = run_v4_assay(
            &rates,
            eps,
            D021_CENTER_RADIUS,
            D021_DIAGNOSTIC_MAX_STEPS,
            D021_DIAGNOSTIC_WINDOW,
        )?;
        let ret = evaluate_retention_localization(&outcome.metrics, 0.0);
        let promote = prebalance_promotion_gate(&baseline.metrics, &outcome.metrics, 0.0);
        // Reject permanent membrane storage: membrane production and loss both > 0.
        let params = v4_params_with_rates(&rates, eps)?;
        let mr = membrane_rates(0.5, 0.4, 0.3, 0.5, &params);
        let permanent_store = mr.decay + mr.detachment <= 1e-15;
        let pass = promote && ret.all_pass() && !permanent_store;
        screens.push(json!({
            "eps_m": eps,
            "baseline_v3": balance_metrics_json(&baseline.metrics),
            "candidate_v4": balance_metrics_json(&outcome.metrics),
            "retention_localization": {
                "c_ok": ret.c_retention_ok,
                "a_ok": ret.a_retention_ok,
                "localization_ok": ret.localization_ok,
                "all_pass": ret.all_pass(),
            },
            "promote": promote,
            "permanent_membrane_store": permanent_store,
            "gate3_pass": pass,
            "classification": format!("{:?}", outcome.classification),
        }));
    }
    // Promote at most one ε (best A retention among passers).
    let mut promoted_eps: Option<f64> = None;
    let mut best_a = -1.0;
    for s in &screens {
        if s["gate3_pass"].as_bool() == Some(true) {
            let a = s["candidate_v4"]["activated_retention"].as_f64().unwrap_or(0.0);
            if a > best_a {
                best_a = a;
                promoted_eps = s["eps_m"].as_f64();
            }
        }
    }
    let body = json!({
        "project_directive": "D-021",
        "gate": 3,
        "screens": screens,
        "promoted_eps_m": promoted_eps,
        "any_pass": promoted_eps.is_some(),
    });
    atomic_write_json(&output.join("gate3_prebalance.json"), &body)?;
    Ok(body)
}

/// Gate 4: bounded four-rate joint recovery under selected ε.
pub fn run_gate4_joint_recovery(
    output: &Path,
    eps_m: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let analytical = analytical_rates();
    let mut current = analytical;
    let mut history = Vec::new();
    let mut g_history = Vec::new();
    let mut sens_history = Vec::new();

    for round in 0..D021_MAX_SOLVER_ROUNDS {
        if history.len() >= D021_MAX_CANDIDATES {
            break;
        }
        let outcome = run_v4_assay(
            &current,
            eps_m,
            D021_CENTER_RADIUS,
            D021_DIAGNOSTIC_MAX_STEPS,
            D021_DIAGNOSTIC_WINDOW,
        )?;
        let g = g_vector(&outcome.metrics);
        g_history.push(g);
        // Identity sensitivity for bounded step (ponytail: full finite-diff deferred to promotion).
        let sens = sensitivity_matrix(&[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        sens_history.push(sens);
        let ret = evaluate_retention_localization(&outcome.metrics, 0.0);
        history.push(json!({
            "round": round,
            "rates": current,
            "metrics": balance_metrics_json(&outcome.metrics),
            "retention_localization_pass": ret.all_pass(),
            "classification": format!("{:?}", outcome.classification),
        }));
        if ret.all_pass()
            && outcome.metrics.structure.q >= 0.5
            && outcome.metrics.structure.q <= 2.0
        {
            break;
        }
        // Q-correct toward balance within D-021 bounds.
        let mut next = current;
        next.k_d008_structure /= outcome.metrics.structure.q.max(1e-6);
        next.k_d008_reproduction /= outcome.metrics.catalyst.q.max(1e-6);
        next.k_membrane /= outcome.metrics.membrane.q.max(1e-6);
        next.k_d008_activation /= outcome.metrics.activated.q.max(1e-6);
        next = clamp_rates_to_global_bounds_d021(&next, &analytical);
        next = freeze_nonproductive_rates(&next, &analytical);
        // Round factor clamp.
        let cur_v = [
            current.k_d008_structure,
            current.k_d008_reproduction,
            current.k_membrane,
            current.k_d008_activation,
        ];
        let nxt_v = [
            next.k_d008_structure,
            next.k_d008_reproduction,
            next.k_membrane,
            next.k_d008_activation,
        ];
        for i in 0..4 {
            let ratio = (nxt_v[i] / cur_v[i].max(1e-30)).clamp(0.67, 1.50);
            let val = cur_v[i] * ratio;
            match i {
                0 => next.k_d008_structure = val,
                1 => next.k_d008_reproduction = val,
                2 => next.k_membrane = val,
                3 => next.k_d008_activation = val,
                _ => {}
            }
        }
        next = clamp_rates_to_global_bounds_d021(&next, &analytical);
        if (next.k_d008_structure - current.k_d008_structure).abs() < 1e-12
            && (next.k_membrane - current.k_membrane).abs() < 1e-12
        {
            break;
        }
        current = next;
    }

    let solver = bounded_joint_solver_d021(&analytical, &analytical, &g_history, &sens_history);
    let best = history
        .iter()
        .filter(|h| h["retention_localization_pass"].as_bool() == Some(true))
        .max_by(|a, b| {
            let sa = a["metrics"]["activated_retention"].as_f64().unwrap_or(0.0);
            let sb = b["metrics"]["activated_retention"].as_f64().unwrap_or(0.0);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();
    let body = json!({
        "project_directive": "D-021",
        "gate": 4,
        "eps_m": eps_m,
        "rounds": history,
        "solver_candidates": solver.candidates.len(),
        "solver_bounded": solver.bounded,
        "max_rounds": D021_MAX_SOLVER_ROUNDS,
        "max_candidates": D021_MAX_CANDIDATES,
        "selected": best,
        "joint_solution_found": best.is_some(),
    });
    atomic_write_json(&output.join("gate4_joint_recovery.json"), &body)?;
    Ok(body)
}

/// Gate 5: full R22 Stage E + R18/R26 restoring check.
pub fn run_gate5_stage_e(
    output: &Path,
    eps_m: f64,
    rates: &StageEReferenceRates,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let params = v4_params_with_rates(rates, eps_m)?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d021-r22"),
        None,
        "D-021 Stage E R22 reference",
        None,
        None,
    );
    let checkpoint_dir = output.join("r22_checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;
    let resume = ["200000", "150000", "100000", "050000", "025000", "010000"]
        .iter()
        .map(|t| checkpoint_dir.join(format!("checkpoint_{t}.json")))
        .find(|p| p.exists());
    let config = D013RunConfig {
        max_steps,
        window_size: D021_FULL_WINDOW,
        radius: D021_CENTER_RADIUS,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir),
        resume_checkpoint: resume,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let mut artifact =
        outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, rates);
    artifact["project_directive"] = json!("D-021");
    artifact["equation_version"] = json!(EquationVersion::MembraneMetabolismV4InterfaceProtected);
    artifact["membrane_schema_version"] = json!(MEMBRANE_SCHEMA_VERSION_V2);
    artifact["structural_schema_version"] = json!(STRUCTURAL_SCHEMA_VERSION_V3);
    artifact["eps_m"] = json!(eps_m);
    artifact["selected_mechanism"] = json!("membrane_metabolism_v4_interface_protected");
    artifact["parent_structural_mechanism"] = json!(V3_SELECTED_MECHANISM.as_str());
    artifact = seal_artifact(artifact)?;
    atomic_write_json(&output.join("r22_result.json"), &artifact)?;

    let mut neighbor_results = Vec::new();
    for &radius in &D021_NEIGHBOR_RADII {
        let params = v4_params_with_rates(rates, eps_m)?;
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some(&format!("d021-r{radius}")),
            None,
            "D-021 neighbor radius confirmation",
            None,
            None,
        );
        let checkpoint_dir = output.join(format!("r{radius}_checkpoints"));
        fs::create_dir_all(&checkpoint_dir)?;
        let config = D013RunConfig {
            max_steps,
            window_size: D021_FULL_WINDOW,
            radius,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(checkpoint_dir),
            resume_checkpoint: None,
        };
        let outcome =
            run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
        let mut nart =
            outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, rates);
        nart["project_directive"] = json!("D-021");
        nart["radius"] = json!(radius);
        nart["eps_m"] = json!(eps_m);
        nart = seal_artifact(nart)?;
        atomic_write_json(&output.join(format!("r{radius}_result.json")), &nart)?;
        neighbor_results.push(nart);
    }

    let g18 = neighbor_results[0]["balance_metrics"]["g_structure"]
        .as_f64()
        .unwrap_or(0.0);
    let g22 = artifact["balance_metrics"]["g_structure"]
        .as_f64()
        .unwrap_or(0.0);
    let g26 = neighbor_results[1]["balance_metrics"]["g_structure"]
        .as_f64()
        .unwrap_or(0.0);
    let restoring = restoring_sign_pattern_pass(g18, g22, g26);
    let body = json!({
        "project_directive": "D-021",
        "gate": 5,
        "eps_m": eps_m,
        "r22": artifact,
        "neighbors": neighbor_results,
        "g_structure": {"R18": g18, "R22": g22, "R26": g26},
        "restoring_sign_pattern": restoring,
    });
    atomic_write_json(&output.join("gate5_stage_e.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(
    output_root: &Path,
    full_max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    fs::create_dir_all(&output_root)?;

    let gate1 = run_gate1_eps_screen(&output_root.join("gate1"))?;
    if gate1["any_pass"].as_bool() != Some(true) {
        let conclusion = D021Conclusion::D021RetentionLocalizationNotRecovered;
        let manifest = json!({
            "project_directive": "D-021",
            "primary_conclusion": conclusion.as_str(),
            "gate1": gate1,
            "wall_seconds": t0.elapsed().as_secs_f64(),
            "note": "No ε candidate passed Gate 1 local mechanism / Stage B localization",
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate2 = run_gate2_fixed_compartment(&output_root.join("gate2"), &gate1)?;
    if gate2["fixed_compartment_regression"].as_bool() == Some(true) {
        let conclusion = D021Conclusion::D021FixedCompartmentRegression;
        let manifest = json!({
            "project_directive": "D-021",
            "primary_conclusion": conclusion.as_str(),
            "gate1": gate1,
            "gate2": gate2,
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }
    if gate2["any_pass"].as_bool() != Some(true) {
        let conclusion = D021Conclusion::D021RetentionLocalizationNotRecovered;
        let manifest = json!({
            "project_directive": "D-021",
            "primary_conclusion": conclusion.as_str(),
            "gate1": gate1,
            "gate2": gate2,
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate3 = run_gate3_prebalance(&output_root.join("gate3"), &gate2)?;
    let promoted_eps = gate3["promoted_eps_m"].as_f64();
    if promoted_eps.is_none() {
        let conclusion = D021Conclusion::D021RetentionLocalizationNotRecovered;
        let manifest = json!({
            "project_directive": "D-021",
            "primary_conclusion": conclusion.as_str(),
            "gate1": gate1,
            "gate2": gate2,
            "gate3": gate3,
            "wall_seconds": t0.elapsed().as_secs_f64(),
            "note": "Retention/localization not recovered; stop modifying rates; reject seven-field membrane bootstrap",
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }
    let eps = promoted_eps.unwrap();

    let gate4 = run_gate4_joint_recovery(&output_root.join("gate4"), eps)?;
    if gate4["joint_solution_found"].as_bool() != Some(true) {
        let conclusion = D021Conclusion::D021NoBoundedJointSolution;
        let manifest = json!({
            "project_directive": "D-021",
            "primary_conclusion": conclusion.as_str(),
            "selected_eps_m": eps,
            "gate1": gate1,
            "gate2": gate2,
            "gate3": gate3,
            "gate4": gate4,
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let selected_rates: StageEReferenceRates = if gate4["selected"]["rates"].is_null() {
        analytical_rates()
    } else {
        serde_json::from_value(gate4["selected"]["rates"].clone())?
    };
    let gate5 = run_gate5_stage_e(
        &output_root.join("gate5"),
        eps,
        &selected_rates,
        full_max_steps,
    )?;

    let r22 = &gate5["r22"];
    let classification = r22["scientific_classification"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let q_ok = {
        let bm = &r22["balance_metrics"];
        [
            bm["Q_structure"].as_f64().unwrap_or(0.0),
            bm["Q_catalyst"].as_f64().unwrap_or(0.0),
            bm["Q_membrane"].as_f64().unwrap_or(0.0),
            bm["Q_activated"].as_f64().unwrap_or(0.0),
        ]
        .iter()
        .all(|v| (0.98..=1.02).contains(v))
    };
    let ret_ok = {
        let bm = &r22["balance_metrics"];
        bm["catalyst_retention"].as_f64().unwrap_or(0.0) >= D021_RETENTION_MIN
            && bm["activated_retention"].as_f64().unwrap_or(0.0) >= D021_RETENTION_MIN
            && bm["membrane_localization"].as_f64().unwrap_or(0.0) >= D021_LOCALIZATION_MIN
    };
    let restoring = gate5["restoring_sign_pattern"].as_bool() == Some(true);
    let stage_e_pass = classification.contains("QUASI_STEADY")
        && q_ok
        && ret_ok
        && restoring;

    let conclusion = select_d021_conclusion(
        stage_e_pass,
        true,
        true,
        true,
        true,
        classification.contains("NUMERICAL"),
    );

    let manifest = json!({
        "project_directive": "D-021",
        "agent_memory_directive": "D-20260716-d021-retention-localization-repair",
        "primary_conclusion": conclusion.as_str(),
        "selected_eps_m": eps,
        "equation_version": EquationVersion::MembraneMetabolismV4InterfaceProtected.as_str(),
        "membrane_schema_version": MEMBRANE_SCHEMA_VERSION_V2,
        "gate1": gate1,
        "gate2": gate2,
        "gate3": gate3,
        "gate4": gate4,
        "gate5": gate5,
        "stage_e_pass": stage_e_pass,
        "d008_stage_e_status": if stage_e_pass {
            "PASS_AFTER_D021_RETENTION_REPAIR"
        } else {
            "BLOCKED_NOT_RECOVERED"
        },
        "wall_seconds": t0.elapsed().as_secs_f64(),
        "preserved_d020_tag": "D-020-v3-joint-rate-recovery-fail",
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
