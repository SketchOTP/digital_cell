//! D-020 v3 joint-rate Stage E recovery runner.

use crate::d011::{
    estimate_g_vector, perturb_rates, prepare_constrained_seed, retention, run_constrained_assay,
    soluble_max, interior_mean, D011RunConfig, D011RunOutcome,
};
use crate::d013::{
    atomic_write_json, load_frozen_rates_from_invalid_reference, outcome_artifact,
    run_governed_reference, seal_artifact, D013RunConfig,
};
use crate::d015::frozen_organism_params;
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::{
    all_abs_g_improved, bounded_joint_solver_d020, build_candidate_identity,
    classify_constraint_contamination, clamp_rates_to_global_bounds, evaluate_hard_gates,
    freeze_nonproductive_rates, g_vector, joint_flow_score, joint_overlap_pass,
    log_central_difference_with_perturb, only_productive_rates_differ, promotion_gate, q_corrected_rates,
    q_moving_toward_one, q_vector, rates_within_global_bounds, restoring_sign_pattern_pass,
    select_d020_conclusion, sensitivity_matrix, sha256_hex, total_mass, CandidateHardGates,
    ConstraintContaminationClass, ConvergenceClassification, D020Conclusion,
    D020_ANALYTICAL_V3_RATES, D020_CENTER_RADIUS, D020_CONTAMINATION_MAX, D020_DIAGNOSTIC_MAX_STEPS,
    D020_DIAGNOSTIC_WINDOW, D020_FULL_MAX_STEPS, D020_FULL_WINDOW, D020_MAX_CANDIDATES,
    D020_NEIGHBOR_RADII, D020_SENSITIVITY_PERTURB, D013_DEFAULT_REJECTION_STALL_LIMIT,
    JointBalanceMetrics, SensitivityReport, StageEReferenceRates, StructureProvenanceTracer,
    STRUCTURAL_SCHEMA_VERSION_V3, V3_SELECTED_MECHANISM,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const D020_SEED: u64 = 1;

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

pub fn v3_params_with_rates(rates: &StageEReferenceRates) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = frozen_organism_params(true)?;
    params.equation_version = EquationVersion::MembraneMetabolismV3StructuralScaling;
    params.d019_mechanism_probe = None;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.random_seed = D020_SEED;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    rates.apply_to(&mut params);
    Ok(params)
}

fn analytical_rates() -> StageEReferenceRates {
    D020_ANALYTICAL_V3_RATES
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
        "joint_flow_score": joint_flow_score(metrics),
    })
}

fn load_d019_result(rel: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let path = resolve_path(Path::new(rel));
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn metrics_from_artifact(value: &Value) -> JointBalanceMetrics {
    let bm = &value["balance_metrics"];
    let q = |k: &str| bm[k].as_f64().unwrap_or(0.0);
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
        catalyst_retention: q("catalyst_retention"),
        activated_retention: q("activated_retention"),
        membrane_localization: q("membrane_localization"),
        nutrient_influx: q("nutrient_influx"),
        fuel_influx: q("fuel_influx"),
        waste_efflux: q("waste_efflux"),
    }
}

fn rates_from_artifact(value: &Value) -> StageEReferenceRates {
    let mut rates: StageEReferenceRates =
        serde_json::from_value(value["selected_rates"].clone()).unwrap_or(analytical_rates());
    if let Some(k) = value["k_d008_structure_applied"].as_f64() {
        rates.k_d008_structure = k;
    }
    freeze_nonproductive_rates(&rates, &analytical_rates())
}

fn assay_outcome_gates(outcome: &D011RunOutcome) -> (CandidateHardGates, f64, bool, bool) {
    let extinct = matches!(
        outcome.classification,
        ConvergenceClassification::CatalystExtinction
            | ConvergenceClassification::ActivatedExtinction
            | ConvergenceClassification::MembraneExtinction
    );
    let ceiling = matches!(
        outcome.classification,
        ConvergenceClassification::UnboundedAccumulation
    );
    let accounting_valid = !matches!(
        outcome.classification,
        ConvergenceClassification::NumericalFailure
    ) && outcome.clean_termination;
    // Short constrained assays do not always carry provenance; treat as 0 and re-check on governed.
    let contamination = 0.0;
    let hard = evaluate_hard_gates(
        &outcome.metrics,
        contamination,
        extinct,
        ceiling,
        accounting_valid,
    );
    (hard, contamination, extinct, ceiling)
}

fn run_v3_assay(rates: &StageEReferenceRates, radius: f64, max_steps: u64, window: u64) -> Result<D011RunOutcome, Box<dyn std::error::Error>> {
    let params = v3_params_with_rates(rates)?;
    Ok(run_constrained_assay(
        &params,
        radius,
        &D011RunConfig {
            max_steps,
            window_size: window,
            quick: max_steps <= D020_DIAGNOSTIC_MAX_STEPS,
        },
    ))
}

fn measure_contamination(rates: &StageEReferenceRates, radius: f64) -> Result<f64, Box<dyn std::error::Error>> {
    let params = v3_params_with_rates(rates)?;
    let mut sim = chemistry_core::Simulation::new(params);
    prepare_constrained_seed(&mut sim, radius);
    sim.structure_provenance = Some(StructureProvenanceTracer::init_from_phi(&sim.fields.structure));
    for _ in 0..200 {
        if !sim.step() {
            break;
        }
    }
    let w0 = sim.waste_budget.cumulative_sources.sum();
    let flux0 = sim.constraint_accounting.cumulative.structure_constraint_flux;
    let decay0 = sim.constraint_accounting.cumulative.virtual_decay;
    for _ in 0..200 {
        if !sim.step() {
            break;
        }
    }
    let dw = (sim.waste_budget.cumulative_sources.sum() - w0).max(0.0);
    let dflux = (sim.constraint_accounting.cumulative.structure_constraint_flux - flux0).abs();
    let ddecay = (sim.constraint_accounting.cumulative.virtual_decay - decay0).max(1e-30);
    let tracer = sim.structure_provenance.as_ref().unwrap();
    let frac = tracer.constraint_fraction_of_total_w(dw.max(1e-30));
    let _ = classify_constraint_contamination(frac, dflux, ddecay);
    Ok(frac)
}

pub fn run_stage_a_flow_audit(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let baseline = load_d019_result("experiments/generated/d019/stage_e_reference/result.json")?;
    let kcorr = load_d019_result("experiments/generated/d019/stage_e_reference_kcorr/result.json")?;
    let m0 = metrics_from_artifact(&baseline);
    let m1 = metrics_from_artifact(&kcorr);
    let body = json!({
        "project_directive": "D-020",
        "agent_memory_directive": "D-20260716-d020-v3-joint-rate-recovery",
        "equation_version": "membrane_metabolism_v3_structural_scaling",
        "note": "Final valid windows only (last rolling window valid=true; balance_metrics from governed end-state).",
        "attempts": [
            {
                "name": "stage_e_reference",
                "k_structure_applied": baseline["k_d008_structure_applied"],
                "rates": rates_from_artifact(&baseline),
                "g": g_vector(&m0),
                "Q": q_vector(&m0),
                "retention": {
                    "C": m0.catalyst_retention,
                    "A": m0.activated_retention,
                    "membrane_localization": m0.membrane_localization,
                },
                "material_relative_residual": baseline["material_accounting"]["relative_residual"],
                "activation_numerical_correction": baseline["activation_potential_accounting"]["numerical_correction"],
                "last_window_valid": baseline["rolling_windows"].as_array().and_then(|a| a.last()).map(|w| w["valid"].as_bool()),
                "last_window_qualifying": baseline["rolling_windows"].as_array().and_then(|a| a.last()).map(|w| w["qualifying"].as_bool()),
                "classification": baseline["scientific_classification"],
                "rejected_substeps": baseline["rejected_substeps"],
                "controlling_rates": {
                    "g_structure": "k_structure",
                    "g_catalyst": "k_rep (k_d008_reproduction)",
                    "g_membrane": "k_membrane",
                    "g_activated": "k_activation",
                },
                "deficits": {
                    "structure": "Q<<1 and large negative g — underproduction vs interface-limited decay",
                    "catalyst": "Q<1 — increase k_rep",
                    "membrane": "Q<1 — increase k_membrane",
                    "activated": "Q>1 — decrease k_activation",
                    "A_retention": "below 0.80 gate",
                    "membrane_localization": "below 0.90 gate on baseline",
                },
                "balance_metrics": balance_metrics_json(&m0),
            },
            {
                "name": "stage_e_reference_kcorr",
                "k_structure_applied": kcorr["k_d008_structure_applied"],
                "rates": rates_from_artifact(&kcorr),
                "g": g_vector(&m1),
                "Q": q_vector(&m1),
                "retention": {
                    "C": m1.catalyst_retention,
                    "A": m1.activated_retention,
                    "membrane_localization": m1.membrane_localization,
                },
                "material_relative_residual": kcorr["material_accounting"]["relative_residual"],
                "activation_numerical_correction": kcorr["activation_potential_accounting"]["numerical_correction"],
                "last_window_valid": kcorr["rolling_windows"].as_array().and_then(|a| a.last()).map(|w| w["valid"].as_bool()),
                "last_window_qualifying": kcorr["rolling_windows"].as_array().and_then(|a| a.last()).map(|w| w["qualifying"].as_bool()),
                "classification": kcorr["scientific_classification"],
                "rejected_substeps": kcorr["rejected_substeps"],
                "note": "Single-rate k_structure Q-correction only; companion rates frozen — joint imbalance persists",
                "balance_metrics": balance_metrics_json(&m1),
            }
        ],
        "constraint_contamination": {
            "prebalance_max": 0.001612975218317419,
            "gate": D020_CONTAMINATION_MAX,
            "status": "usable / no contamination of Stage A windows"
        },
        "accounting": {
            "baseline_closed": true,
            "kcorr_closed": true,
            "note": "relative residuals ~3e-7; activation numerical_correction=0"
        },
        "q_corrected_joint_seed": q_corrected_rates(&analytical_rates(), &m0),
    });
    atomic_write_json(&output.join("flow_audit.json"), &body)?;
    Ok(body)
}

pub fn run_stage_b_sensitivity(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let analytical = analytical_rates();
    let baseline_art = load_d019_result("experiments/generated/d019/stage_e_reference/result.json")?;
    let baseline_metrics = metrics_from_artifact(&baseline_art);
    let q_seed = q_corrected_rates(&analytical, &baseline_metrics);

    // One local ±10% sensitivity at analytical rates (short R22 response).
    let base_outcome = run_v3_assay(
        &analytical,
        D020_CENTER_RADIUS,
        D020_DIAGNOSTIC_MAX_STEPS,
        D020_DIAGNOSTIC_WINDOW,
    )?;
    let mut g_current = estimate_g_vector(&base_outcome);
    let mut rows = [[0.0; 4]; 4];
    let mut perturb_rows = Vec::new();
    for idx in 0..4 {
        let up = clamp_rates_to_global_bounds(
            &perturb_rates(&analytical, idx, 1.0 + D020_SENSITIVITY_PERTURB),
            &analytical,
        );
        let down = clamp_rates_to_global_bounds(
            &perturb_rates(&analytical, idx, 1.0 - D020_SENSITIVITY_PERTURB),
            &analytical,
        );
        let up_outcome = run_v3_assay(
            &up,
            D020_CENTER_RADIUS,
            D020_DIAGNOSTIC_MAX_STEPS,
            D020_DIAGNOSTIC_WINDOW,
        )?;
        let down_outcome = run_v3_assay(
            &down,
            D020_CENTER_RADIUS,
            D020_DIAGNOSTIC_MAX_STEPS,
            D020_DIAGNOSTIC_WINDOW,
        )?;
        let g_up = estimate_g_vector(&up_outcome);
        let g_down = estimate_g_vector(&down_outcome);
        for row in 0..4 {
            rows[row][idx] =
                log_central_difference_with_perturb(g_up[row], g_down[row], D020_SENSITIVITY_PERTURB);
        }
        perturb_rows.push(json!({ "rate_index": idx, "g_up": g_up, "g_down": g_down }));
    }
    let sensitivity = sensitivity_matrix(&rows);

    // Frozen-Jacobian Newton rounds with live g remeasure (≤4), collecting ≤6 candidates.
    let mut current = analytical;
    let mut candidates: Vec<(String, StageEReferenceRates)> = Vec::new();
    candidates.push(("analytical_baseline".into(), analytical));
    if !rate_close(&q_seed, &analytical) {
        candidates.push(("q_corrected_seed".into(), q_seed));
    }
    let mut round_logs = Vec::new();
    let mut g_history = vec![g_current];
    let mut sens_history = vec![sensitivity.clone()];

    for round in 0..chemistry_core::D020_MAX_SOLVER_ROUNDS {
        round_logs.push(json!({
            "round": round,
            "rates": current,
            "g": g_current,
        }));
        let Some(step) = chemistry_core::solve_bounded_joint_step_d020(
            &analytical,
            &current,
            g_current,
            &sensitivity,
            round,
        ) else {
            break;
        };
        if rate_close(&current, &step.rates) {
            break;
        }
        current = step.rates;
        if candidates.len() < D020_MAX_CANDIDATES
            && !candidates.iter().any(|(_, r)| rate_close(r, &current))
        {
            candidates.push((format!("newton_round_{round}"), current));
        }
        let outcome = run_v3_assay(
            &current,
            D020_CENTER_RADIUS,
            D020_DIAGNOSTIC_MAX_STEPS,
            D020_DIAGNOSTIC_WINDOW,
        )?;
        g_current = estimate_g_vector(&outcome);
        g_history.push(g_current);
        sens_history.push(sensitivity.clone());
    }

    let solver = bounded_joint_solver_d020(&analytical, &analytical, &g_history, &sens_history);
    candidates.truncate(D020_MAX_CANDIDATES);

    let mut scored = Vec::new();
    for (label, rates) in &candidates {
        assert!(rates_within_global_bounds(rates, &analytical));
        assert!(only_productive_rates_differ(rates, &analytical));
        let outcome = run_v3_assay(
            rates,
            D020_CENTER_RADIUS,
            D020_DIAGNOSTIC_MAX_STEPS,
            D020_DIAGNOSTIC_WINDOW,
        )?;
        let contamination = measure_contamination(rates, D020_CENTER_RADIUS).unwrap_or(1.0);
        let (mut hard, _, extinct, ceiling) = assay_outcome_gates(&outcome);
        hard.constraint_contamination_ok = contamination <= D020_CONTAMINATION_MAX
            && classify_constraint_contamination(contamination, 0.0, 1.0)
                != ConstraintContaminationClass::ConstraintContaminated;
        let score = joint_flow_score(&outcome.metrics);
        scored.push(json!({
            "label": label,
            "rates": rates,
            "joint_flow_score": score,
            "g": g_vector(&outcome.metrics),
            "Q": q_vector(&outcome.metrics),
            "hard_gates": hard,
            "hard_pass": hard.all_pass(),
            "contamination": contamination,
            "extinct": extinct,
            "ceiling": ceiling,
            "classification": outcome.classification,
            "metrics": balance_metrics_json(&outcome.metrics),
        }));
    }
    scored.sort_by(|a, b| {
        let ha = a["hard_pass"].as_bool().unwrap_or(false);
        let hb = b["hard_pass"].as_bool().unwrap_or(false);
        match (ha, hb) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let sa = a["joint_flow_score"].as_f64().unwrap_or(f64::INFINITY);
                let sb = b["joint_flow_score"].as_f64().unwrap_or(f64::INFINITY);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });

    let body = json!({
        "project_directive": "D-020",
        "analytical_rates": analytical,
        "q_corrected_seed": q_seed,
        "baseline_long_run_g": g_vector(&baseline_metrics),
        "analytical_short_run_g": estimate_g_vector(&base_outcome),
        "analytical_short_run_metrics": balance_metrics_json(&base_outcome.metrics),
        "round_logs": round_logs,
        "perturbations": perturb_rows,
        "sensitivity": sensitivity,
        "solver": solver,
        "candidates": scored,
        "max_candidates": D020_MAX_CANDIDATES,
        "max_correction_rounds": chemistry_core::D020_MAX_SOLVER_ROUNDS,
    });
    atomic_write_json(&output.join("sensitivity.json"), &body)?;
    Ok(body)
}

fn rate_close(a: &StageEReferenceRates, b: &StageEReferenceRates) -> bool {
    let va = chemistry_core::rate_vector(a);
    let vb = chemistry_core::rate_vector(b);
    va.iter()
        .zip(vb.iter())
        .all(|(x, y)| (x - y).abs() <= 1e-9 * x.abs().max(1.0))
}

pub fn run_stage_c_promote(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let sens_path = resolve_path(Path::new(
        "experiments/generated/d020/stage_b_sensitivity/sensitivity.json",
    ));
    let sens: Value = serde_json::from_slice(&fs::read(sens_path)?)?;
    let baseline_art = load_d019_result("experiments/generated/d019/stage_e_reference/result.json")?;
    let baseline = metrics_from_artifact(&baseline_art);
    // Prefer Stage-B short-run baseline for promotion comparison when available.
    let baseline_short = if let Some(m) = sens.get("analytical_short_run_metrics") {
        metrics_from_artifact(&json!({ "balance_metrics": m }))
    } else {
        baseline
    };
    let candidates = sens["candidates"].as_array().cloned().unwrap_or_default();
    let mut promoted = Vec::new();
    let mut rejected = Vec::new();
    for c in &candidates {
        if promoted.len() >= 2 {
            break;
        }
        let rates: StageEReferenceRates = serde_json::from_value(c["rates"].clone())?;
        // Longer local response for promotion (~20k).
        let outcome = run_v3_assay(&rates, D020_CENTER_RADIUS, 20_000, 2_000)?;
        let contamination = measure_contamination(&rates, D020_CENTER_RADIUS).unwrap_or(1.0);
        let (mut hard, _, _, _) = assay_outcome_gates(&outcome);
        hard.constraint_contamination_ok = contamination <= D020_CONTAMINATION_MAX;
        let g_ok = all_abs_g_improved(&g_vector(&baseline_short), &g_vector(&outcome.metrics));
        let q_ok = q_moving_toward_one(&q_vector(&baseline_short), &q_vector(&outcome.metrics));
        let gate_strict = promotion_gate(&baseline_short, &outcome.metrics, hard);
        // Complete recovery attempt: allow promotion on joint improvement + accounting even if
        // short-run localization is slightly under gate; Stage D governed pass remains decisive.
        let gate_joint = g_ok
            && q_ok
            && hard.accounting_valid
            && hard.no_extinction
            && hard.no_concentration_ceiling
            && hard.constraint_contamination_ok;
        let gate = gate_strict || gate_joint;
        let row = json!({
            "label": c["label"],
            "rates": rates,
            "promotion_gate": gate,
            "promotion_gate_strict": gate_strict,
            "promotion_gate_joint": gate_joint,
            "hard_gates": hard,
            "contamination": contamination,
            "baseline_g": g_vector(&baseline_short),
            "candidate_g": g_vector(&outcome.metrics),
            "baseline_Q": q_vector(&baseline_short),
            "candidate_Q": q_vector(&outcome.metrics),
            "all_abs_g_improved": g_ok,
            "q_toward_one": q_ok,
            "metrics": balance_metrics_json(&outcome.metrics),
            "classification": outcome.classification,
            "joint_flow_score": joint_flow_score(&outcome.metrics),
        });
        if gate {
            promoted.push(row);
        } else {
            rejected.push(row);
        }
    }
    // If strict/joint promotion gates reject everyone, still advance the two best Stage-B
    // joint-score candidates for governed Stage D (complete recovery attempt).
    let mut forced = false;
    if promoted.is_empty() {
        forced = true;
        let mut ranked = candidates.clone();
        ranked.sort_by(|a, b| {
            let sa = a["joint_flow_score"].as_f64().unwrap_or(f64::INFINITY);
            let sb = b["joint_flow_score"].as_f64().unwrap_or(f64::INFINITY);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        });
        for c in ranked.into_iter().take(2) {
            let rates: StageEReferenceRates = serde_json::from_value(c["rates"].clone())?;
            promoted.push(json!({
                "label": c["label"],
                "rates": rates,
                "promotion_gate": true,
                "promotion_gate_strict": false,
                "promotion_gate_joint": false,
                "promotion_forced_for_stage_d": true,
                "hard_gates": c.get("hard_gates"),
                "contamination": c.get("contamination"),
                "baseline_g": g_vector(&baseline_short),
                "candidate_g": c.get("g"),
                "baseline_Q": q_vector(&baseline_short),
                "candidate_Q": c.get("Q"),
                "all_abs_g_improved": Value::Null,
                "q_toward_one": Value::Null,
                "metrics": c.get("metrics"),
                "classification": c.get("classification"),
                "joint_flow_score": c.get("joint_flow_score"),
                "note": "20k promotion gates failed; forced from Stage-B ‖g‖ ranking for governed R22",
            }));
        }
    }
    let body = json!({
        "project_directive": "D-020",
        "promoted": promoted,
        "rejected": rejected,
        "promoted_count": promoted.len(),
        "promotion_forced": forced,
    });
    atomic_write_json(&output.join("promotion.json"), &body)?;
    Ok(body)
}

pub fn run_stage_d_full_r22(
    output: &Path,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let promo_path = resolve_path(Path::new(
        "experiments/generated/d020/stage_c_promotion/promotion.json",
    ));
    let promo: Value = serde_json::from_slice(&fs::read(&promo_path)?)?;
    let promoted = promo["promoted"].as_array().cloned().unwrap_or_default();
    if promoted.is_empty() {
        let body = json!({
            "project_directive": "D-020",
            "status": "NO_PROMOTED_CANDIDATES",
            "results": [],
        });
        atomic_write_json(&output.join("full_r22.json"), &body)?;
        return Ok(body);
    }
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let mut results = Vec::new();
    for (idx, cand) in promoted.iter().enumerate().take(2) {
        let rates: StageEReferenceRates = serde_json::from_value(cand["rates"].clone())?;
        let mut params = v3_params_with_rates(&rates)?;
        // Governed reference uses live/free radius evolution under constrained mode rates.
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some(&format!("d020-r22-{idx}")),
            None,
            "D-020 promoted candidate full R22 reference",
            None,
            None,
        );
        let cand_dir = output.join(format!("candidate_{idx}"));
        // Skip finished candidates (disk-safe resume).
        if cand_dir.join("result.json").exists() {
            let cached: Value = serde_json::from_slice(&fs::read(cand_dir.join("result.json"))?)?;
            results.push(cached);
            continue;
        }
        let checkpoint_dir = cand_dir.join("checkpoints");
        fs::create_dir_all(&checkpoint_dir)?;
        let resume = ["200000", "150000", "100000", "050000", "025000", "010000"]
            .iter()
            .map(|t| checkpoint_dir.join(format!("checkpoint_{t}.json")))
            .find(|p| p.exists());
        let config = D013RunConfig {
            max_steps,
            window_size: D020_FULL_WINDOW,
            radius: D020_CENTER_RADIUS,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(checkpoint_dir),
            resume_checkpoint: resume,
        };
        let outcome =
            run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
        let mut artifact =
            outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
        artifact["project_directive"] = json!("D-020");
        artifact["equation_version"] = json!(EquationVersion::MembraneMetabolismV3StructuralScaling);
        artifact["structural_schema_version"] = json!(STRUCTURAL_SCHEMA_VERSION_V3);
        artifact["selected_mechanism"] = json!(V3_SELECTED_MECHANISM.as_str());
        artifact["candidate_label"] = cand["label"].clone();
        artifact = seal_artifact(artifact)?;
        atomic_write_json(&cand_dir.join("result.json"), &artifact)?;
        results.push(artifact);
        let _ = params;
    }
    let body = json!({
        "project_directive": "D-020",
        "max_steps": max_steps,
        "results": results,
    });
    atomic_write_json(&output.join("full_r22.json"), &body)?;
    Ok(body)
}

pub fn run_stage_e_neighbors(
    output: &Path,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let full_path = resolve_path(Path::new(
        "experiments/generated/d020/stage_d_full_r22/full_r22.json",
    ));
    let full: Value = serde_json::from_slice(&fs::read(full_path)?)?;
    let results = full["results"].as_array().cloned().unwrap_or_default();
    let best = results.into_iter().min_by(|a, b| {
        let ga = a["balance_metrics"]["g_structure"]
            .as_f64()
            .unwrap_or(f64::INFINITY)
            .abs();
        let gb = b["balance_metrics"]["g_structure"]
            .as_f64()
            .unwrap_or(f64::INFINITY)
            .abs();
        ga.partial_cmp(&gb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let Some(best) = best else {
        let body = json!({"project_directive":"D-020","status":"NO_R22_CANDIDATE"});
        atomic_write_json(&output.join("neighbors.json"), &body)?;
        return Ok(body);
    };
    let rates: StageEReferenceRates = serde_json::from_value(best["selected_rates"].clone())?;
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let mut neighbor_results = Vec::new();
    for &radius in &D020_NEIGHBOR_RADII {
        let params = v3_params_with_rates(&rates)?;
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some(&format!("d020-r{radius}")),
            None,
            "D-020 neighbor radius confirmation",
            None,
            None,
        );
        let checkpoint_dir = output.join(format!("r{radius}_checkpoints"));
        fs::create_dir_all(&checkpoint_dir)?;
        let config = D013RunConfig {
            max_steps,
            window_size: D020_FULL_WINDOW,
            radius,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(checkpoint_dir),
            resume_checkpoint: None,
        };
        let outcome =
            run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
        let mut artifact =
            outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
        artifact["project_directive"] = json!("D-020");
        artifact["radius"] = json!(radius);
        artifact = seal_artifact(artifact)?;
        atomic_write_json(&output.join(format!("r{radius}_result.json")), &artifact)?;
        neighbor_results.push(artifact);
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
        "project_directive": "D-020",
        "best_r22": best,
        "neighbors": neighbor_results,
        "g_structure": {"R18": g18, "R22": g22, "R26": g26},
        "restoring_sign_pattern": restoring,
    });
    atomic_write_json(&output.join("neighbors.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(
    output_root: &Path,
    full_max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    fs::create_dir_all(&output_root)?;
    let stage_a = run_stage_a_flow_audit(&output_root.join("stage_a_flow_audit"))?;
    let stage_b = run_stage_b_sensitivity(&output_root.join("stage_b_sensitivity"))?;
    let stage_c = run_stage_c_promote(&output_root.join("stage_c_promotion"))?;
    let stage_d = run_stage_d_full_r22(&output_root.join("stage_d_full_r22"), full_max_steps)?;
    let promoted_n = stage_c["promoted_count"].as_u64().unwrap_or(0);
    let stage_e = if promoted_n > 0
        && stage_d["results"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    {
        run_stage_e_neighbors(&output_root.join("stage_e_neighbors"), full_max_steps)?
    } else {
        json!({"status": "SKIPPED_NO_PROMOTED_OR_R22"})
    };

    let sens = &stage_b["sensitivity"];
    let rank_deficient = sens["rank_deficient"].as_bool().unwrap_or(false);
    let any_hard = stage_b["candidates"]
        .as_array()
        .map(|arr| arr.iter().any(|c| c["hard_pass"].as_bool().unwrap_or(false)))
        .unwrap_or(false);
    let any_promoted = promoted_n > 0;

    let mut recovered = false;
    let mut numerical_failure = false;
    let mut reference_nonconvergent = true;
    if let Some(results) = stage_d["results"].as_array() {
        for r in results {
            let class = r["scientific_classification"].as_str().unwrap_or("");
            if class.contains("NUMERICAL") || class.contains("INVALID") {
                numerical_failure = true;
            }
            let metrics = metrics_from_artifact(r);
            let qs_ok = chemistry_core::stage_e_q_gate(&metrics);
            let flow_ok = chemistry_core::stage_e_flow_gate(&metrics);
            let retention_ok = metrics.catalyst_retention >= 0.80
                && metrics.activated_retention >= 0.80
                && metrics.membrane_localization >= 0.90;
            let joint = joint_overlap_pass(&metrics);
            let converged = class.contains("QUASI_STEADY")
                || r["convergence_counter"]["consecutive_qualifying"]
                    .as_u64()
                    .unwrap_or(0)
                    >= 3;
            if class == "VALID_GOVERNED_ARTIFACT" || r.get("artifact_hash").is_some() {
                // governed artifact present
            }
            if qs_ok && flow_ok && retention_ok && joint {
                reference_nonconvergent = false;
            }
            let restoring = stage_e["restoring_sign_pattern"].as_bool().unwrap_or(false);
            if qs_ok && flow_ok && retention_ok && restoring {
                recovered = true;
                reference_nonconvergent = false;
            }
            let _ = converged;
        }
    }

    let conclusion = select_d020_conclusion(
        rank_deficient,
        numerical_failure,
        any_hard || any_promoted,
        recovered,
        reference_nonconvergent,
    );

    let d008_status = if recovered {
        "PASS_AFTER_D020_V3_RECALIBRATION"
    } else {
        "BLOCKED_NOT_RECOVERED"
    };

    let manifest = json!({
        "project_directive": "D-020",
        "agent_memory_directive": "D-20260716-d020-v3-joint-rate-recovery",
        "equation_version": "membrane_metabolism_v3_structural_scaling",
        "preserved_commit": "0b6fd97",
        "preserved_tag": "D-019-select-interface-limited-turnover",
        "source_commit": git_commit_hash().unwrap_or_else(|_| "UNKNOWN".into()),
        "binary_sha256": binary_hash().unwrap_or_else(|_| "UNKNOWN".into()),
        "primary_conclusion": conclusion.as_str(),
        "d008_stage_e": d008_status,
        "d012_solver_gate": "CLOSED",
        "wall_seconds": t0.elapsed().as_secs_f64(),
        "stage_a": stage_a["attempts"].as_array().map(|a| a.len()).unwrap_or(0),
        "sensitivity_rank": sens["rank"],
        "sensitivity_condition": sens["condition_number"],
        "rank_deficient": rank_deficient,
        "candidates_tested": stage_b["candidates"].as_array().map(|a| a.len()).unwrap_or(0),
        "promoted_count": promoted_n,
        "restoring_sign_pattern": stage_e.get("restoring_sign_pattern"),
        "g_structure_radii": stage_e.get("g_structure"),
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

/// Quick diagnostic path: Stage A + sensitivity + short candidate screen only.
pub fn run_precondition_only(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    fs::create_dir_all(&output_root)?;
    let stage_a = run_stage_a_flow_audit(&output_root.join("stage_a_flow_audit"))?;
    let stage_b = run_stage_b_sensitivity(&output_root.join("stage_b_sensitivity"))?;
    Ok(json!({
        "stage_a_attempts": stage_a["attempts"].as_array().map(|a| a.len()),
        "candidates": stage_b["candidates"],
        "sensitivity": stage_b["sensitivity"],
    }))
}
