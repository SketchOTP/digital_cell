//! D-015 waste throughput diagnosis and environmental W-sink repair runners.

use crate::d013::{
    atomic_write_json, load_frozen_rates_from_invalid_reference, outcome_artifact, run_governed_reference,
    seal_artifact, v2_frozen_params, D013RunConfig, GovernedRunOutcome,
};
use chemistry_core::{
    apply_d015_repaired_environment, build_waste_spatial_masks, build_candidate_identity,
    d015_preflight_requires_waste_budget, d015_repaired_waste_sink_inner_radius,
    diagnostic_membrane_bypass_waste, environment_configuration_hash, field_mass,
    linear_sink_clearance_rate, organism_frozen_hash, waste_sink_cell_count, GridConfiguration,
    SimParams, Simulation, WASTE_BUDGET_REL_TOL, D012_V2_CENTER_RADIUS, D012_V2_MAX_STEPS,
    D012_V2_WINDOW, D013_DEFAULT_REJECTION_STALL_LIMIT, D015_ENVIRONMENT_SCHEMA_VERSION,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const FROZEN_CANDIDATE: &str =
    "9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626";
pub const FROZEN_CONFIG: &str =
    "87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6";

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    Ok(chemistry_core::sha256_hex(&fs::read(std::env::current_exe()?)?))
}

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

pub fn frozen_organism_params(repaired: bool) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = v2_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    if repaired {
        apply_d015_repaired_environment(&mut params, D012_V2_CENTER_RADIUS);
    }
    Ok(params)
}

pub fn run_preserve(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let d014_manifest = resolve_path(Path::new("experiments/generated/d014/manifest.json"));
    let params = frozen_organism_params(false)?;
    let grid = GridConfiguration::default();
    let identity = build_candidate_identity(
        params.clone(),
        &git_commit_hash().unwrap_or_else(|_| "unknown".into()),
        Some("d015-preserve"),
        None,
        "D-015 preservation record",
        None,
        None,
    );
    let body = json!({
        "project_directive": "D-015",
        "source_commit": git_commit_hash().ok(),
        "binary_sha256": binary_hash().ok(),
        "frozen_candidate_hash": FROZEN_CANDIDATE,
        "frozen_configuration_hash": FROZEN_CONFIG,
        "observed_candidate_hash": identity.candidate_hash,
        "observed_configuration_hash": identity.configuration_hash,
        "organism_frozen_hash": organism_frozen_hash(&params, &grid),
        "environment_configuration_hash_baseline": environment_configuration_hash(&params),
        "d014_manifest_exists": d014_manifest.exists(),
        "primary_diagnosis": "D015_BULK_DIFFUSION_BOTTLENECK / TRANSPORT_TO_SINK_LIMITED",
        "repaired_waste_sink_inner_radius": d015_repaired_waste_sink_inner_radius(D012_V2_CENTER_RADIUS),
        "environment_schema_version": D015_ENVIRONMENT_SCHEMA_VERSION,
    });
    atomic_write_json(&output.join("preservation_record.json"), &body)?;
    Ok(body)
}

pub fn run_regression_summary(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let log_path = resolve_path(Path::new("experiments/generated/d015/regressions/gate.log"));
    let summary = if log_path.exists() {
        json!({
            "source": log_path.display().to_string(),
            "log_present": true,
            "log_bytes": fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0),
        })
    } else {
        json!({
            "source": log_path.display().to_string(),
            "log_present": false,
            "note": "Run chemistry-core release tests and append output to gate.log",
        })
    };
    atomic_write_json(&output.join("regression_summary.json"), &summary)?;
    Ok(summary)
}

fn disable_biology(params: &mut SimParams) {
    params.k_d008_activation = 0.0;
    params.k_d008_reproduction = 0.0;
    params.k_d008_structure = 0.0;
    params.k_membrane = 0.0;
}

pub fn run_controls(output: &Path, repaired: bool) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut params = frozen_organism_params(repaired)?;
    let grid_cfg = GridConfiguration::default();
    let org_hash = organism_frozen_hash(&params, &grid_cfg);
    let env_hash = environment_configuration_hash(&params);
    let sink_cells = {
        let sim = Simulation::new(params.clone());
        waste_sink_cell_count(&sim.grid, &params)
    };

    let mut ext = params.clone();
    disable_biology(&mut ext);
    let mut sim_a = Simulation::new(ext);
    for idx in 0..sim_a.fields.waste.len() {
        if chemistry_core::reservoir::waste_sink_cell(&sim_a.grid, idx, &sim_a.params) {
            sim_a.fields.waste[idx] = 1.0;
        }
    }
    let w_before = field_mass(&sim_a.grid, &sim_a.fields.waste);
    for _ in 0..500 {
        if !sim_a.step() {
            break;
        }
    }
    let w_after = field_mass(&sim_a.grid, &sim_a.fields.waste);

    let mut int = params.clone();
    disable_biology(&mut int);
    let mut sim_b = Simulation::new(int);
    let cx = sim_b.grid.cx as usize;
    let cy = sim_b.grid.cy as usize;
    let center = chemistry_core::Grid::index(sim_b.grid.width, cx, cy);
    sim_b.fields.waste[center] = 5.0;
    let center_before = sim_b.fields.waste[center];
    for _ in 0..300 {
        if !sim_b.step() {
            break;
        }
    }

    let bypass = diagnostic_membrane_bypass_waste(&params);

    let mut no_clear = params.clone();
    disable_biology(&mut no_clear);
    no_clear.reservoir_rate = 0.0;
    let mut sim_e = Simulation::new(no_clear);
    let masks = build_waste_spatial_masks(&sim_e.grid, &sim_e.fields.structure, D012_V2_CENTER_RADIUS);
    for idx in 0..sim_e.fields.waste.len() {
        if masks.bulk_exterior[idx]
            || chemistry_core::reservoir::waste_sink_cell(&sim_e.grid, idx, &sim_e.params)
        {
            sim_e.fields.waste[idx] = 0.5;
        }
    }
    let e_before = field_mass(&sim_e.grid, &sim_e.fields.waste);
    for _ in 0..50 {
        let _ = sim_e.step();
    }
    let e_after = field_mass(&sim_e.grid, &sim_e.fields.waste);

    let body = json!({
        "project_directive": "D-015",
        "repaired_environment": repaired,
        "organism_frozen_hash": org_hash,
        "environment_configuration_hash": env_hash,
        "waste_sink_inner_radius": params.waste_sink_inner_radius,
        "waste_sink_cell_count": sink_cells,
        "control_a_external_pulse": {
            "w_before": w_before,
            "w_after": w_after,
            "cleared": w_after < w_before,
        },
        "control_b_internal_pulse": {
            "center_before": center_before,
            "center_after": sim_b.fields.waste[center],
            "exported": sim_b.fields.waste[center] < center_before,
        },
        "control_c_membrane_bypass": {
            "baseline_beta_w": params.beta_w,
            "diagnostic_beta_w": bypass.beta_w,
        },
        "control_e_no_clearance": {
            "w_before": e_before,
            "w_after": e_after,
            "accumulated": e_after >= e_before,
        },
    });
    atomic_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_preflight(output: &Path, repaired: bool) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;
    let params = frozen_organism_params(repaired)?;
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let grid = GridConfiguration::default();
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d015-preflight"),
        None,
        "D-015 waste-throughput preflight",
        None,
        None,
    );
    let rates = load_frozen_rates_from_invalid_reference()?;
    let config = D013RunConfig {
        max_steps: 25_000,
        window_size: 1_000,
        radius: D012_V2_CENTER_RADIUS,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir.clone()),
        resume_checkpoint: None,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let mut artifact = outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
    artifact = seal_artifact(artifact)?;
    atomic_write_json(&output.join("artifact.json"), &artifact)?;

    let waste_budget_ok = outcome.waste_budget_accepted_steps > 0
        && outcome.waste_budget_max_relative_residual <= WASTE_BUDGET_REL_TOL;
    let has_10k = checkpoint_dir.join("checkpoint_010000.json").exists();
    let has_25k = checkpoint_dir.join("checkpoint_025000.json").exists();
    let req = chemistry_core::d015_preflight_requirements();
    let body = json!({
        "project_directive": "D-015",
        "repaired_environment": repaired,
        "organism_frozen_hash": organism_frozen_hash(&params, &grid),
        "environment_configuration_hash": environment_configuration_hash(&params),
        "waste_sink_inner_radius": params.waste_sink_inner_radius,
        "preflight_pass": has_10k && has_25k && waste_budget_ok && d015_preflight_requires_waste_budget(&req),
        "checkpoints": {"10k": has_10k, "25k": has_25k},
        "waste_budget_ok": waste_budget_ok,
        "waste_budget_max_relative_residual": outcome.waste_budget_max_relative_residual,
        "accepted_substeps": outcome.accepted_substeps,
        "termination_reason": outcome.termination_reason,
        "scientific_classification": outcome.scientific_classification,
    });
    atomic_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_fresh_r22(output: &Path, repaired: bool) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;
    let params = frozen_organism_params(repaired)?;
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let grid = GridConfiguration::default();
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d015-fresh-r22"),
        None,
        "D-015 fresh governed R22 reference",
        None,
        None,
    );
    let config = D013RunConfig {
        max_steps: D012_V2_MAX_STEPS,
        window_size: D012_V2_WINDOW,
        radius: D012_V2_CENTER_RADIUS,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir),
        resume_checkpoint: None,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let sim_probe = Simulation::new(params.clone());
    let sink_rate = linear_sink_clearance_rate(&sim_probe.grid, &sim_probe.fields.waste, &params);
    let body = fresh_r22_body(
        &outcome,
        &params,
        &identity,
        &source_commit,
        &binary_sha,
        sink_rate,
        repaired,
        &grid,
    );
    atomic_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

fn fresh_r22_body(
    outcome: &GovernedRunOutcome,
    params: &SimParams,
    identity: &chemistry_core::CandidateIdentity,
    source_commit: &str,
    binary_sha: &str,
    sink_rate: f64,
    repaired: bool,
    grid: &GridConfiguration,
) -> Value {
    json!({
        "project_directive": "D-015",
        "source_commit": source_commit,
        "binary_sha256": binary_sha,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "organism_frozen_hash": organism_frozen_hash(params, grid),
        "environment_configuration_hash": environment_configuration_hash(params),
        "frozen_candidate_hash": FROZEN_CANDIDATE,
        "frozen_configuration_hash": FROZEN_CONFIG,
        "repaired_environment": repaired,
        "waste_sink_inner_radius": params.waste_sink_inner_radius,
        "environment_schema_version": D015_ENVIRONMENT_SCHEMA_VERSION,
        "accepted_substeps": outcome.accepted_substeps,
        "simulated_time": outcome.simulated_time,
        "termination_reason": outcome.termination_reason,
        "scientific_classification": outcome.scientific_classification,
        "checkpoint_completion": outcome.checkpoint_completion,
        "waste_budget_max_relative_residual": outcome.waste_budget_max_relative_residual,
        "waste_budget_accepted_steps": outcome.waste_budget_accepted_steps,
        "linear_sink_clearance_rate_probe": sink_rate,
        "field_hashes": outcome.field_hashes,
        "wall_seconds": outcome.wall_seconds,
    })
}

pub fn run_analyze_d014_checkpoint(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let ckpt = resolve_path(Path::new(
        "experiments/generated/d013/reference_r22/checkpoints/checkpoint_150000.json",
    ));
    let body = json!({
        "checkpoint_path": ckpt.display().to_string(),
        "checkpoint_exists": ckpt.exists(),
        "note": "D-014 150k checkpoint available for replay diagnosis",
    });
    atomic_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}
