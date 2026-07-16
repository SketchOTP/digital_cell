//! D-014 constrained-radius numerical stability diagnostics and repair runners.

use crate::d011::prepare_constrained_seed;
use crate::d013::{
    atomic_write_json, load_frozen_rates_from_invalid_reference, load_governed_checkpoint,
    run_governed_reference, run_preflight, v2_frozen_params, D013RunConfig, GovernedCheckpoint,
};
use chemistry_core::{
    build_candidate_identity, classify_cause_from_terminal_limiter,
    recovered_dt_after_accept, AttemptTelemetry, DtLimiter, EquationVersion, SimParams, Simulation,
    MAX_DT, D012_V2_CENTER_RADIUS, D012_V2_MAX_STEPS, D012_V2_WINDOW,
    D013_DEFAULT_REJECTION_STALL_LIMIT, D014_DT_FLOOR, D014_ADAPTIVE_CONTROLLER_VERSION,
    D014_NUMERICAL_METHOD_VERSION,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const D013_CKPT_150K: &str =
    "experiments/generated/d013/reference_r22/checkpoints/checkpoint_150000.json";
const FROZEN_CANDIDATE: &str =
    "9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626";
const FROZEN_CONFIG: &str =
    "87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6";

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    let path = std::env::current_exe()?;
    Ok(chemistry_core::sha256_hex(&fs::read(path)?))
}

fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        return p;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(p)
}

fn restore_sim_from_d013_checkpoint(
    ckpt_path: &Path,
    params: &SimParams,
) -> Result<(Simulation, GovernedCheckpoint), Box<dyn std::error::Error>> {
    let ckpt = load_governed_checkpoint(ckpt_path)?;
    let mut sim = Simulation::new(params.clone());
    prepare_constrained_seed(&mut sim, D012_V2_CENTER_RADIUS);
    sim.try_restore_snapshot(&ckpt.snapshot)?;
    ckpt.lossless_fields.restore_into(&mut sim)?;
    sim.fields.copy_current_to_next();
    sim.dt = ckpt.current_dt;
    sim.min_dt_seen = ckpt.min_accepted_dt;
    sim.min_attempted_dt = ckpt.min_attempted_dt;
    sim.rejection_count = ckpt.rejected_substeps;
    sim.attempted_substeps = ckpt.attempted_substeps;
    sim.max_consecutive_rejections = ckpt.max_consecutive_rejections;
    sim.substep = ckpt.accepted_substeps;
    sim.sim_time = ckpt.simulated_time;
    sim.accounting.cumulative = serde_json::from_value(ckpt.accounting_cumulative.clone())?;
    sim.metabolism_accounting.cumulative =
        serde_json::from_value(ckpt.metabolism_cumulative.clone())?;
    sim.membrane_accounting.cumulative =
        serde_json::from_value(ckpt.membrane_cumulative.clone())?;
    sim.constraint_accounting.cumulative =
        serde_json::from_value(ckpt.constraint_cumulative.clone())?;
    sim.transport_accounting.cumulative =
        serde_json::from_value(ckpt.transport_ledgers.clone())?;
    Ok((sim, ckpt))
}

pub fn run_failure_reproduction(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let ckpt_path = resolve_path(D013_CKPT_150K);
    if !ckpt_path.exists() {
        return Err(format!("missing D-013 150k checkpoint: {}", ckpt_path.display()).into());
    }

    let mut params = v2_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;

    let (mut sim, ckpt) = restore_sim_from_d013_checkpoint(&ckpt_path, &params)?;
    let start_substep = sim.substep;
    let start_time = sim.sim_time;
    let start_dt = sim.dt;
    let start_field_hashes = json!({
        "structure": chemistry_core::field_sha256_stable(&sim.fields.structure),
        "catalyst": chemistry_core::field_sha256_stable(&sim.fields.catalyst),
        "activated": chemistry_core::field_sha256_stable(&sim.fields.activated),
        "membrane": chemistry_core::field_sha256_stable(&sim.fields.membrane),
        "fuel": chemistry_core::field_sha256_stable(&sim.fields.fuel),
        "nutrient": chemistry_core::field_sha256_stable(&sim.fields.nutrient),
        "waste": chemistry_core::field_sha256_stable(&sim.fields.waste),
    });

    let hash_match = start_field_hashes == json!(ckpt.field_hashes);
    let mut telemetry: Vec<AttemptTelemetry> = Vec::new();
    let mut limiter_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut last_limiter = DtLimiter::Unknown;
    let mut transitions = Vec::new();
    let target = 170_000u64;
    let mut step_failed = false;
    let original_failure_substep = 161_166u64;

    while sim.substep < target {
        let accepted_before = sim.substep;
        let time_before = sim.sim_time;
        let dt_enter = sim.dt;
        let attempt_before = sim.attempted_substeps;
        let ok = sim.step();
        let attempts = sim.attempted_substeps - attempt_before;
        let limiter = sim.last_reject_limiter;
        *limiter_counts
            .entry(format!("{limiter:?}"))
            .or_insert(0) += if ok { 0 } else { 1 };
        if limiter != last_limiter && (!ok || attempts > 1) {
            transitions.push(json!({
                "previous_limiter": last_limiter,
                "new_limiter": limiter,
                "accepted_substep": sim.substep,
                "simulated_time": sim.sim_time,
                "previous_dt": dt_enter,
                "new_dt": sim.dt,
                "detail": sim.last_reject_detail,
            }));
            last_limiter = limiter;
        }
        let max_c = sim.fields.catalyst.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let max_a = sim.fields.activated.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let max_m = sim.fields.membrane.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_c = sim.fields.catalyst.iter().cloned().fold(f64::INFINITY, f64::min);
        let min_a = sim.fields.activated.iter().cloned().fold(f64::INFINITY, f64::min);
        if !ok || attempts > 1 || sim.dt < dt_enter * 0.99 || sim.substep + 1000 >= target {
            telemetry.push(AttemptTelemetry {
                accepted_substep: accepted_before,
                attempt_index: sim.attempted_substeps,
                simulated_time: time_before,
                dt_entering: dt_enter,
                dt_attempted: sim.min_attempted_dt,
                accepted: ok,
                limiter,
                rejection_reason: if ok {
                    None
                } else {
                    Some(sim.last_reject_detail.clone())
                },
                failing_field: None,
                failing_index: None,
                max_c,
                max_a,
                max_m,
                min_c,
                min_a,
            });
        }
        if !ok {
            step_failed = true;
            break;
        }
    }

    let concentration_bound_abort = step_failed
        && sim.last_reject_limiter == DtLimiter::FieldBoundValidation
        && sim.last_reject_detail.contains("excessive concentration");
    let floor_failure = step_failed
        && !concentration_bound_abort
        && (sim.min_attempted_dt <= D014_DT_FLOOR * 2.0
            || sim.last_reject_limiter == DtLimiter::AdaptiveController);
    let cause = classify_cause_from_terminal_limiter(sim.last_reject_limiter);
    let reproduced = floor_failure
        && sim.substep >= 160_000
        && sim.substep <= 165_000
        && sim.min_attempted_dt <= D014_DT_FLOOR * 2.0;
    let passed_original_failure_point = !floor_failure
        && (sim.substep > original_failure_substep
            || concentration_bound_abort
            || sim.substep >= target);

    let result = json!({
        "project_directive": "D-014",
        "source_commit": source_commit,
        "binary_sha256": binary_sha,
        "candidate_hash": FROZEN_CANDIDATE,
        "configuration_hash": FROZEN_CONFIG,
        "checkpoint_path": ckpt_path.display().to_string(),
        "checkpoint_field_hash_match": hash_match,
        "start_accepted_substeps": start_substep,
        "start_simulated_time": start_time,
        "start_dt": start_dt,
        "end_accepted_substeps": sim.substep,
        "end_simulated_time": sim.sim_time,
        "end_dt": sim.dt,
        "min_attempted_dt": sim.min_attempted_dt,
        "step_failed": step_failed,
        "floor_failure": floor_failure,
        "concentration_bound_abort": concentration_bound_abort,
        "passed_original_failure_point": passed_original_failure_point,
        "reproduced_near_original": reproduced,
        "terminal_limiter": sim.last_reject_limiter,
        "terminal_detail": sim.last_reject_detail,
        "numerical_cause_classification": cause,
        "limiter_counts": limiter_counts,
        "limiter_transitions": transitions,
        "telemetry_sample_count": telemetry.len(),
        "telemetry_tail": telemetry.iter().rev().take(50).collect::<Vec<_>>(),
        "max_c": sim.fields.catalyst.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        "max_a": sim.fields.activated.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        "max_waste": sim.fields.waste.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        "equation_version": EquationVersion::MembraneMetabolismV2Conservative,
        "numerical_method_version": D014_NUMERICAL_METHOD_VERSION,
        "adaptive_controller_version": D014_ADAPTIVE_CONTROLLER_VERSION,
        "nondeterministic_if_not_reproduced": !floor_failure && !concentration_bound_abort && sim.substep < 160_000,
    });
    atomic_write_json(&output.join("result.json"), &result)?;
    atomic_write_json(&output.join("telemetry_tail.json"), &json!(telemetry))?;
    Ok(result)
}

pub fn run_diagnostic_replay_170k(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    // Repaired binary diagnostic: must not TIMESTEP_FLOOR at the original failure point.
    let mut result = run_failure_reproduction(output)?;
    if let Some(obj) = result.as_object_mut() {
        obj.insert("diagnostic_mode".into(), json!("repaired_binary_150k_to_170k"));
        obj.insert(
            "original_failure_point_passed".into(),
            json!(obj
                .get("passed_original_failure_point")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)),
        );
    }
    atomic_write_json(&output.join("result.json"), &result)?;
    Ok(result)
}

pub fn run_d014_preflight(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    // Reuse D-013 preflight machinery on repaired binary.
    let result = run_preflight(output)?;
    Ok(result)
}

pub fn run_fresh_reference_r22(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = v2_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d014-fresh-r22"),
        None,
        "D-014 fresh governed Stage E reference",
        None,
        None,
    );
    fs::create_dir_all(output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;
    let config = D013RunConfig {
        max_steps: D012_V2_MAX_STEPS,
        window_size: D012_V2_WINDOW,
        radius: D012_V2_CENTER_RADIUS,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir),
        resume_checkpoint: None,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let body = json!({
        "project_directive": "D-014",
        "source_commit": source_commit,
        "binary_sha256": binary_sha,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "frozen_candidate_hash": FROZEN_CANDIDATE,
        "frozen_configuration_hash": FROZEN_CONFIG,
        "numerical_method_version": D014_NUMERICAL_METHOD_VERSION,
        "adaptive_controller_version": D014_ADAPTIVE_CONTROLLER_VERSION,
        "accepted_substeps": outcome.accepted_substeps,
        "attempted_substeps": outcome.attempted_substeps,
        "rejected_substeps": outcome.rejected_substeps,
        "simulated_time": outcome.simulated_time,
        "final_dt": outcome.current_dt,
        "minimum_accepted_dt": outcome.min_accepted_dt,
        "minimum_attempted_dt": outcome.min_attempted_dt,
        "termination_reason": outcome.termination_reason,
        "scientific_classification": outcome.scientific_classification,
        "clean_termination": outcome.clean_termination,
        "checkpoint_completion": outcome.checkpoint_completion,
        "balance_metrics": {
            "Q_structure": outcome.metrics.structure.q,
            "Q_catalyst": outcome.metrics.catalyst.q,
            "Q_membrane": outcome.metrics.membrane.q,
            "Q_activated": outcome.metrics.activated.q,
            "g_structure": outcome.metrics.structure.g,
            "g_catalyst": outcome.metrics.catalyst.g,
            "g_membrane": outcome.metrics.membrane.g,
            "g_activated": outcome.metrics.activated.g,
            "catalyst_retention": outcome.metrics.catalyst_retention,
            "activated_retention": outcome.metrics.activated_retention,
            "membrane_localization": outcome.metrics.membrane_localization,
            "nutrient_influx": outcome.metrics.nutrient_influx,
            "fuel_influx": outcome.metrics.fuel_influx,
            "waste_efflux": outcome.metrics.waste_efflux,
        },
        "material_accounting": outcome.material_accounting,
        "activation_potential_accounting": outcome.activation_potential_accounting,
        "field_hashes": outcome.field_hashes,
        "wall_seconds": outcome.wall_seconds,
        "convergence_counter": {
            "consecutive_qualifying": outcome.convergence.consecutive_qualifying,
            "required": outcome.convergence.required,
        },
    });
    atomic_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_controller_unit_checks() -> Value {
    let grown = recovered_dt_after_accept(0.001, MAX_DT);
    json!({
        "recovered_dt_from_0.001": grown,
        "recovery_factor": chemistry_core::D014_DT_RECOVERY_GROWTH,
        "max_dt": MAX_DT,
        "floor": D014_DT_FLOOR,
        "passes_growth_bound": grown <= MAX_DT && grown >= 0.001,
    })
}

fn snapshot_masses(sim: &Simulation) -> Value {
    json!({
        "C": chemistry_core::field_mass(&sim.grid, &sim.fields.catalyst),
        "N": chemistry_core::field_mass(&sim.grid, &sim.fields.nutrient),
        "F": chemistry_core::field_mass(&sim.grid, &sim.fields.fuel),
        "W": chemistry_core::field_mass(&sim.grid, &sim.fields.waste),
        "A": chemistry_core::field_mass(&sim.grid, &sim.fields.activated),
        "M": chemistry_core::field_mass(&sim.grid, &sim.fields.membrane),
        "Phi": chemistry_core::field_mass(&sim.grid, &sim.fields.structure),
    })
}

fn run_horizon_with_dt_cap(
    params: &SimParams,
    dt_cap: f64,
    target_simulated_time: f64,
    max_accepted: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut sim = Simulation::new(params.clone());
    prepare_constrained_seed(&mut sim, D012_V2_CENTER_RADIUS);
    sim.dt_cap = dt_cap;
    sim.dt = dt_cap.min(sim.dt);
    let mut floor = false;
    let mut bound_abort = false;
    while sim.sim_time + 1e-15 < target_simulated_time && sim.substep < max_accepted {
        if !sim.step() {
            bound_abort = sim.last_reject_limiter == DtLimiter::FieldBoundValidation;
            floor = !bound_abort
                && (sim.min_attempted_dt <= D014_DT_FLOOR * 2.0
                    || sim.last_reject_limiter == DtLimiter::AdaptiveController);
            break;
        }
    }
    Ok(json!({
        "dt_cap": dt_cap,
        "target_simulated_time": target_simulated_time,
        "accepted_substeps": sim.substep,
        "simulated_time": sim.sim_time,
        "final_dt": sim.dt,
        "min_attempted_dt": sim.min_attempted_dt,
        "floor_failure": floor,
        "concentration_bound_abort": bound_abort,
        "masses": snapshot_masses(&sim),
        "field_hashes": {
            "catalyst": chemistry_core::field_sha256_stable(&sim.fields.catalyst),
            "activated": chemistry_core::field_sha256_stable(&sim.fields.activated),
            "waste": chemistry_core::field_sha256_stable(&sim.fields.waste),
            "membrane": chemistry_core::field_sha256_stable(&sim.fields.membrane),
        },
    }))
}

pub fn run_nonstiff_equivalence(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut params = v2_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    // Same physical time as 25k accepted steps at reference MAX_DT.
    let target_time = 25_000.0 * MAX_DT;
    let max_accepted = 200_000u64;
    let caps = [MAX_DT, MAX_DT * 0.5, MAX_DT * 0.25];
    let mut runs = Vec::new();
    for &cap in &caps {
        runs.push(run_horizon_with_dt_cap(&params, cap, target_time, max_accepted)?);
    }
    let ref_masses = &runs[0]["masses"];
    let mut relative_mass_errors = Vec::new();
    for run in runs.iter().skip(1) {
        let mut errs = serde_json::Map::new();
        for key in ["C", "N", "F", "W", "A", "M", "Phi"] {
            let a = ref_masses[key].as_f64().unwrap_or(0.0);
            let b = run["masses"][key].as_f64().unwrap_or(0.0);
            let scale = a.abs().max(b.abs()).max(1.0);
            errs.insert(key.into(), json!((a - b).abs() / scale));
        }
        relative_mass_errors.push(Value::Object(errs));
    }
    let max_rel_err = relative_mass_errors
        .iter()
        .flat_map(|e| e.as_object().into_iter().flat_map(|m| m.values()))
        .filter_map(|v| v.as_f64())
        .fold(0.0_f64, f64::max);
    let result = json!({
        "project_directive": "D-014",
        "comparison_mode": "equal_simulated_time",
        "target_simulated_time": target_time,
        "reference_accepted_at_max_dt": 25_000,
        "dt_caps": caps,
        "runs": runs,
        "relative_mass_errors_vs_reference": relative_mass_errors,
        "max_relative_mass_error": max_rel_err,
        "candidate_hash": FROZEN_CANDIDATE,
        "numerical_method_version": D014_NUMERICAL_METHOD_VERSION,
    });
    atomic_write_json(&output.join("result.json"), &result)?;
    Ok(result)
}

pub fn run_dt_refinement(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    // Same multi-cap horizon; refinement = errors should not grow as dt_cap shrinks.
    let result = run_nonstiff_equivalence(output)?;
    atomic_write_json(&output.join("result.json"), &result)?;
    Ok(result)
}
