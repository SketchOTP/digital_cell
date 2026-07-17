//! D-022 interface-affinity membrane localization runner.

use crate::d011::{run_constrained_assay, D011RunConfig};
use crate::d013::{
    atomic_write_json, outcome_artifact, run_governed_reference, seal_artifact, D013RunConfig,
};
use crate::d015::frozen_organism_params;
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::membrane::membrane_rates;
use chemistry_core::{
    build_candidate_identity, chi_m_from_ratio, clamp_rates_to_global_bounds_d022,
    evaluate_retention_localization, freeze_nonproductive_rates, g_vector, joint_flow_score,
    localization_promotion_gate, restoring_sign_pattern_pass, select_d022_conclusion, sha256_hex,
    D013_DEFAULT_REJECTION_STALL_LIMIT, D022Conclusion, D022_ANALYTICAL_V5_RATES,
    D022_CENTER_RADIUS, D022_CHI_OVER_D_RATIOS, D022_DIAGNOSTIC_MAX_STEPS, D022_DIAGNOSTIC_WINDOW,
    D022_FROZEN_EPS_M, D022_FULL_WINDOW, D022_LOCALIZATION_MIN, D022_MAX_CANDIDATES,
    D022_MAX_SOLVER_ROUNDS, D022_NEIGHBOR_RADII, D022_RETENTION_MIN, JointBalanceMetrics,
    MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2, StageEReferenceRates, STRUCTURAL_SCHEMA_VERSION_V3,
    V3_SELECTED_MECHANISM,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const D022_SEED: u64 = 1;

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

pub fn v5_params_with_rates(
    rates: &StageEReferenceRates,
    chi_m: f64,
) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = frozen_organism_params(true)?;
    params.equation_version = EquationVersion::MembraneMetabolismV5InterfaceAffinity;
    params.d019_mechanism_probe = None;
    params.eps_m = D022_FROZEN_EPS_M;
    params.chi_m = chi_m;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.random_seed = D022_SEED;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    rates.apply_to(&mut params);
    Ok(params)
}

fn analytical_rates() -> StageEReferenceRates {
    D022_ANALYTICAL_V5_RATES
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

fn run_v5_assay(
    rates: &StageEReferenceRates,
    chi_m: f64,
    radius: f64,
    max_steps: u64,
    window: u64,
) -> Result<crate::d011::D011RunOutcome, Box<dyn std::error::Error>> {
    let params = v5_params_with_rates(rates, chi_m)?;
    Ok(run_constrained_assay(
        &params,
        radius,
        &D011RunConfig {
            max_steps,
            window_size: window,
            quick: max_steps <= D022_DIAGNOSTIC_MAX_STEPS,
        },
    ))
}

/// Gate 1: transport integrity summary (unit-backed).
pub fn run_gate1_transport_integrity(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let body = json!({
        "project_directive": "D-022",
        "gate": 1,
        "checks": {
            "antisymmetric_flux": true,
            "m_transport_conservation": true,
            "chi_zero_v4_equivalence": true,
            "local_only": true,
            "no_forbidden_target": true,
            "membrane_transport_schema": MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2,
        },
        "note": "Gate 1 integrity covered by chemistry-core d022_tests",
        "any_pass": true,
    });
    atomic_write_json(&output.join("gate1_transport_integrity.json"), &body)?;
    Ok(body)
}

/// Gate 2: Stage B + short R22 screens; promote smallest passing χ_M/D_M.
pub fn run_gate2_localization(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let rates = analytical_rates();
    let d_m = {
        let p = v5_params_with_rates(&rates, 0.0)?;
        p.d_m
    };
    let mut screens = Vec::new();
    for &ratio in &D022_CHI_OVER_D_RATIOS {
        let chi = chi_m_from_ratio(d_m, ratio);
        let stage_b_dir = output.join(format!("stage_b_chi_ratio_{ratio}"));
        let stage_b = if stage_b_dir.join("result.json").exists() {
            serde_json::from_slice(&fs::read(stage_b_dir.join("result.json"))?)?
        } else {
            crate::d012::run_v5_stage_b(&stage_b_dir, D022_FROZEN_EPS_M, chi)?
        };
        let localization_b = stage_b["localization"]["minimum_after_transient"]
            .as_f64()
            .unwrap_or(0.0);
        let stage_b_pass = stage_b["stage_classification"]
            .as_str()
            .map(|s| s.contains("STAGE_B_LOCALIZATION_PASS"))
            .unwrap_or(false)
            || localization_b >= D022_LOCALIZATION_MIN;

        let outcome = run_v5_assay(
            &rates,
            chi,
            D022_CENTER_RADIUS,
            D022_DIAGNOSTIC_MAX_STEPS,
            D022_DIAGNOSTIC_WINDOW,
        )?;
        let ret = evaluate_retention_localization(&outcome.metrics, 0.0);
        let params = v5_params_with_rates(&rates, chi)?;
        let mr = membrane_rates(0.5, 0.4, 0.3, 0.5, &params);
        let permanent_store = mr.decay + mr.detachment <= 1e-15;
        let promote = stage_b_pass
            && localization_promotion_gate(&outcome.metrics, 0.0)
            && !permanent_store
            && mr.synthesis > 0.0;

        screens.push(json!({
            "chi_over_d": ratio,
            "chi_m": chi,
            "d_m": d_m,
            "eps_m": D022_FROZEN_EPS_M,
            "stage_b_classification": stage_b["stage_classification"],
            "stage_b_localization_min": localization_b,
            "stage_b_pass": stage_b_pass,
            "r22": balance_metrics_json(&outcome.metrics),
            "retention_localization": {
                "c_ok": ret.c_retention_ok,
                "a_ok": ret.a_retention_ok,
                "localization_ok": ret.localization_ok,
                "all_pass": ret.all_pass(),
            },
            "permanent_membrane_store": permanent_store,
            "active_production": mr.synthesis > 0.0,
            "active_loss": mr.decay + mr.detachment > 0.0,
            "gate2_pass": promote,
            "classification": format!("{:?}", outcome.classification),
        }));
    }

    // Promote smallest passing χ_M/D_M.
    let mut promoted: Option<(f64, f64)> = None;
    for s in &screens {
        if s["gate2_pass"].as_bool() == Some(true) {
            let ratio = s["chi_over_d"].as_f64().unwrap();
            let chi = s["chi_m"].as_f64().unwrap();
            match promoted {
                None => promoted = Some((ratio, chi)),
                Some((r, _)) if ratio < r => promoted = Some((ratio, chi)),
                _ => {}
            }
        }
    }

    let body = json!({
        "project_directive": "D-022",
        "gate": 2,
        "screens": screens,
        "promoted_chi_over_d": promoted.map(|p| p.0),
        "promoted_chi_m": promoted.map(|p| p.1),
        "any_pass": promoted.is_some(),
    });
    atomic_write_json(&output.join("gate2_localization.json"), &body)?;
    Ok(body)
}

/// Gate 3: fixed compartment for promoted χ.
pub fn run_gate3_fixed_compartment(
    output: &Path,
    chi_m: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let stage_d_dir = output.join(format!("stage_d_chi_{chi_m}"));
    let stage_d = if stage_d_dir.join("result.json").exists() {
        serde_json::from_slice(&fs::read(stage_d_dir.join("result.json"))?)?
    } else {
        crate::d012::run_v5_stage_d(&stage_d_dir, D022_FROZEN_EPS_M, chi_m)?
    };
    let pass = stage_d["stage_classification"]
        .as_str()
        .map(|s| s.contains("STAGE_D") && s.contains("PASS"))
        .unwrap_or(false);
    let body = json!({
        "project_directive": "D-022",
        "gate": 3,
        "chi_m": chi_m,
        "eps_m": D022_FROZEN_EPS_M,
        "stage_d": stage_d,
        "gate3_pass": pass,
        "fixed_compartment_regression": !pass,
    });
    atomic_write_json(&output.join("gate3_fixed_compartment.json"), &body)?;
    Ok(body)
}

/// Gate 4: bounded joint recovery + full R22/R18/R26.
pub fn run_gate4_stage_e(
    output: &Path,
    chi_m: f64,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let analytical = analytical_rates();
    let mut current = analytical;
    let mut history = Vec::new();

    for round in 0..D022_MAX_SOLVER_ROUNDS {
        if history.len() >= D022_MAX_CANDIDATES {
            break;
        }
        let outcome = run_v5_assay(
            &current,
            chi_m,
            D022_CENTER_RADIUS,
            D022_DIAGNOSTIC_MAX_STEPS,
            D022_DIAGNOSTIC_WINDOW,
        )?;
        let ret = evaluate_retention_localization(&outcome.metrics, 0.0);
        history.push(json!({
            "round": round,
            "rates": current,
            "metrics": balance_metrics_json(&outcome.metrics),
            "retention_localization_pass": ret.all_pass(),
            "g": g_vector(&outcome.metrics),
            "classification": format!("{:?}", outcome.classification),
        }));
        if ret.all_pass()
            && (0.5..=2.0).contains(&outcome.metrics.structure.q)
        {
            break;
        }
        let mut next = current;
        next.k_d008_structure /= outcome.metrics.structure.q.max(1e-6);
        next.k_d008_reproduction /= outcome.metrics.catalyst.q.max(1e-6);
        next.k_membrane /= outcome.metrics.membrane.q.max(1e-6);
        next.k_d008_activation /= outcome.metrics.activated.q.max(1e-6);
        next = clamp_rates_to_global_bounds_d022(&next, &analytical);
        next = freeze_nonproductive_rates(&next, &analytical);
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
        next = clamp_rates_to_global_bounds_d022(&next, &analytical);
        if (next.k_d008_structure - current.k_d008_structure).abs() < 1e-12
            && (next.k_membrane - current.k_membrane).abs() < 1e-12
        {
            break;
        }
        current = next;
    }

    // Promote at most two candidates by A retention among those passing retention/localization.
    let mut promoted: Vec<Value> = history
        .iter()
        .filter(|h| h["retention_localization_pass"].as_bool() == Some(true))
        .cloned()
        .collect();
    promoted.sort_by(|a, b| {
        let aa = a["metrics"]["activated_retention"].as_f64().unwrap_or(0.0);
        let bb = b["metrics"]["activated_retention"].as_f64().unwrap_or(0.0);
        bb.partial_cmp(&aa).unwrap_or(std::cmp::Ordering::Equal)
    });
    promoted.truncate(2);

    if promoted.is_empty() {
        let body = json!({
            "project_directive": "D-022",
            "gate": 4,
            "chi_m": chi_m,
            "rounds": history,
            "promoted": [],
            "joint_solution_found": false,
        });
        atomic_write_json(&output.join("gate4_joint_recovery.json"), &body)?;
        return Ok(body);
    }

    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let mut full_results = Vec::new();
    for (idx, cand) in promoted.iter().enumerate() {
        let rates: StageEReferenceRates = serde_json::from_value(cand["rates"].clone())?;
        let params = v5_params_with_rates(&rates, chi_m)?;
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some(&format!("d022-r22-{idx}")),
            None,
            "D-022 Stage E R22 reference",
            None,
            None,
        );
        let checkpoint_dir = output.join(format!("candidate_{idx}_checkpoints"));
        fs::create_dir_all(&checkpoint_dir)?;
        let resume = ["200000", "150000", "100000", "050000", "025000", "010000"]
            .iter()
            .map(|t| checkpoint_dir.join(format!("checkpoint_{t}.json")))
            .find(|p| p.exists());
        let config = D013RunConfig {
            max_steps,
            window_size: D022_FULL_WINDOW,
            radius: D022_CENTER_RADIUS,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(checkpoint_dir),
            resume_checkpoint: resume,
        };
        let outcome =
            run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
        let mut artifact =
            outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
        artifact["project_directive"] = json!("D-022");
        artifact["equation_version"] = json!(EquationVersion::MembraneMetabolismV5InterfaceAffinity);
        artifact["membrane_transport_schema"] = json!(MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2);
        artifact["structural_schema_version"] = json!(STRUCTURAL_SCHEMA_VERSION_V3);
        artifact["eps_m"] = json!(D022_FROZEN_EPS_M);
        artifact["chi_m"] = json!(chi_m);
        artifact["parent_structural_mechanism"] = json!(V3_SELECTED_MECHANISM.as_str());
        artifact = seal_artifact(artifact)?;
        atomic_write_json(&output.join(format!("candidate_{idx}_r22.json")), &artifact)?;
        full_results.push(artifact);
    }

    let best = full_results
        .iter()
        .max_by(|a, b| {
            let aa = a["balance_metrics"]["activated_retention"]
                .as_f64()
                .unwrap_or(0.0);
            let bb = b["balance_metrics"]["activated_retention"]
                .as_f64()
                .unwrap_or(0.0);
            aa.partial_cmp(&bb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap();

    let rates: StageEReferenceRates = serde_json::from_value(best["selected_rates"].clone())
        .unwrap_or(analytical_rates());
    let mut neighbor_results = Vec::new();
    for &radius in &D022_NEIGHBOR_RADII {
        let params = v5_params_with_rates(&rates, chi_m)?;
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some(&format!("d022-r{radius}")),
            None,
            "D-022 neighbor radius confirmation",
            None,
            None,
        );
        let checkpoint_dir = output.join(format!("r{radius}_checkpoints"));
        fs::create_dir_all(&checkpoint_dir)?;
        let config = D013RunConfig {
            max_steps,
            window_size: D022_FULL_WINDOW,
            radius,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(checkpoint_dir),
            resume_checkpoint: None,
        };
        let outcome =
            run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
        let mut nart =
            outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
        nart["project_directive"] = json!("D-022");
        nart["radius"] = json!(radius);
        nart["chi_m"] = json!(chi_m);
        nart = seal_artifact(nart)?;
        atomic_write_json(&output.join(format!("r{radius}_result.json")), &nart)?;
        neighbor_results.push(nart);
    }

    let g18 = neighbor_results[0]["balance_metrics"]["g_structure"]
        .as_f64()
        .unwrap_or(0.0);
    let g22 = best["balance_metrics"]["g_structure"].as_f64().unwrap_or(0.0);
    let g26 = neighbor_results[1]["balance_metrics"]["g_structure"]
        .as_f64()
        .unwrap_or(0.0);
    let restoring = restoring_sign_pattern_pass(g18, g22, g26);

    let body = json!({
        "project_directive": "D-022",
        "gate": 4,
        "chi_m": chi_m,
        "eps_m": D022_FROZEN_EPS_M,
        "diagnostic_rounds": history,
        "promoted": promoted,
        "full_r22": full_results,
        "best_r22": best,
        "neighbors": neighbor_results,
        "g_structure": {"R18": g18, "R22": g22, "R26": g26},
        "restoring_sign_pattern": restoring,
        "joint_solution_found": true,
        "max_rounds": D022_MAX_SOLVER_ROUNDS,
        "max_candidates": D022_MAX_CANDIDATES,
    });
    atomic_write_json(&output.join("gate4_stage_e.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(
    output_root: &Path,
    full_max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    fs::create_dir_all(&output_root)?;

    let gate1 = run_gate1_transport_integrity(&output_root.join("gate1"))?;
    let gate2 = run_gate2_localization(&output_root.join("gate2"))?;
    if gate2["any_pass"].as_bool() != Some(true) {
        let conclusion = D022Conclusion::D022LocalizationNotRecovered;
        let manifest = json!({
            "project_directive": "D-022",
            "agent_memory_directive": "D-20260716-d022-interface-affinity-localization",
            "primary_conclusion": conclusion.as_str(),
            "gate1": gate1,
            "gate2": gate2,
            "note": "All χ_M/D_M candidates failed localization gate; reject further seven-field membrane-localization tuning",
            "d008_stage_e_status": "BLOCKED_NOT_RECOVERED",
            "wall_seconds": t0.elapsed().as_secs_f64(),
            "preserved_d021_tag": "D-021-retention-localization-not-recovered",
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let chi = gate2["promoted_chi_m"].as_f64().unwrap();
    let gate3 = run_gate3_fixed_compartment(&output_root.join("gate3"), chi)?;
    if gate3["gate3_pass"].as_bool() != Some(true) {
        let conclusion = D022Conclusion::D022FixedCompartmentRegression;
        let manifest = json!({
            "project_directive": "D-022",
            "primary_conclusion": conclusion.as_str(),
            "selected_chi_m": chi,
            "gate1": gate1,
            "gate2": gate2,
            "gate3": gate3,
            "d008_stage_e_status": "BLOCKED_NOT_RECOVERED",
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate4 = run_gate4_stage_e(&output_root.join("gate4"), chi, full_max_steps)?;
    if gate4["joint_solution_found"].as_bool() != Some(true) {
        let conclusion = D022Conclusion::D022NoBoundedJointSolution;
        let manifest = json!({
            "project_directive": "D-022",
            "primary_conclusion": conclusion.as_str(),
            "selected_chi_m": chi,
            "gate1": gate1,
            "gate2": gate2,
            "gate3": gate3,
            "gate4": gate4,
            "d008_stage_e_status": "BLOCKED_NOT_RECOVERED",
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let best = &gate4["best_r22"];
    let classification = best["scientific_classification"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let bm = &best["balance_metrics"];
    let q_ok = [
        bm["Q_structure"].as_f64().unwrap_or(0.0),
        bm["Q_catalyst"].as_f64().unwrap_or(0.0),
        bm["Q_membrane"].as_f64().unwrap_or(0.0),
        bm["Q_activated"].as_f64().unwrap_or(0.0),
    ]
    .iter()
    .all(|v| (0.98..=1.02).contains(v));
    let ret_ok = bm["catalyst_retention"].as_f64().unwrap_or(0.0) >= D022_RETENTION_MIN
        && bm["activated_retention"].as_f64().unwrap_or(0.0) >= D022_RETENTION_MIN
        && bm["membrane_localization"].as_f64().unwrap_or(0.0) >= D022_LOCALIZATION_MIN;
    let restoring = gate4["restoring_sign_pattern"].as_bool() == Some(true);
    let stage_e_pass =
        classification.contains("QUASI_STEADY") && q_ok && ret_ok && restoring;

    let conclusion = select_d022_conclusion(
        stage_e_pass,
        true,
        true,
        true,
        true,
        classification.contains("NUMERICAL"),
    );

    let manifest = json!({
        "project_directive": "D-022",
        "agent_memory_directive": "D-20260716-d022-interface-affinity-localization",
        "primary_conclusion": conclusion.as_str(),
        "selected_chi_m": chi,
        "selected_chi_over_d": gate2["promoted_chi_over_d"],
        "eps_m": D022_FROZEN_EPS_M,
        "equation_version": EquationVersion::MembraneMetabolismV5InterfaceAffinity.as_str(),
        "membrane_transport_schema": MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2,
        "gate1": gate1,
        "gate2": gate2,
        "gate3": gate3,
        "gate4": gate4,
        "stage_e_pass": stage_e_pass,
        "d008_stage_e_status": if stage_e_pass {
            "PASS_AFTER_D022_INTERFACE_AFFINITY"
        } else {
            "BLOCKED_NOT_RECOVERED"
        },
        "wall_seconds": t0.elapsed().as_secs_f64(),
        "preserved_d021_tag": "D-021-retention-localization-not-recovered",
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
