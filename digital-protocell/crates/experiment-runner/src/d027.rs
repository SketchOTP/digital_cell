//! D-027 coupled surface-renewal calibration runner (Gates 0–12).

use crate::d013::{
    atomic_write_json, build_checkpoint, load_governed_checkpoint, restore_governed_simulation,
    write_governed_checkpoint,
};
use crate::d025::{seed_v7_compartment, v7_base_params, D025_FROZEN_K_ADS};
use chemistry_core::build_candidate_identity;
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::{
    classify_adsorption_portability, compute_adsorption_basis_labeled, frozen_k_ads_d024,
    generate_analytical_candidates, surface_balance_q, surface_rates_parity,
    WindowLocalSurfaceRates, D027_CANDIDATE_SCALES,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, InterfaceGeometryCell,
};
use chemistry_core::{ActivationPotentialLedger, ConvergenceCounter, Simulation};
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

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn binary_hash() -> String {
    let path = std::env::current_exe().ok();
    path.and_then(|p| fs::read(p).ok())
        .map(|b| chemistry_core::sha256_hex(&b))
        .unwrap_or_else(|| "unknown".into())
}

fn stage_e_ckpt(step: u64) -> PathBuf {
    resolve_path(Path::new(&format!(
        "experiments/generated/d025/stage_e_reference/checkpoints/checkpoint_{:06}.json",
        step
    )))
}

fn restore_stage_e_checkpoint(step: u64) -> Result<Simulation, Box<dyn std::error::Error>> {
    let path = stage_e_ckpt(step);
    if !path.is_file() {
        return Err(format!("missing Stage E checkpoint {}", path.display()).into());
    }
    let ckpt = load_governed_checkpoint(&path)?;
    let mut sim = Simulation::new(v7_base_params()?);
    restore_governed_simulation(&mut sim, &ckpt)?;
    sim.enforce_structure_constraint = true;
    Ok(sim)
}

fn fixed_interface_r22_state() -> Result<Simulation, Box<dyn std::error::Error>> {
    let params = v7_base_params()?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    for _ in 0..2_000 {
        if !sim.step() {
            break;
        }
    }
    Ok(sim)
}

fn dynamic_r22_endpoint_state() -> Result<Simulation, Box<dyn std::error::Error>> {
    let params = v7_base_params()?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    for _ in 0..4_000 {
        if !sim.step() {
            break;
        }
    }
    Ok(sim)
}

fn gamma_localization(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    surface_localization(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        sim.params.delta_floor,
    )
}

/// Gate 0: checkpoint window-local surface ledger restore parity.
pub fn run_gate0_ledger_restore(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let source_commit = git_commit_hash();
    let binary_sha = binary_hash();

    let params = v7_base_params()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d027-gate0"),
        None,
        "D-027 Gate 0 ledger restore",
        None,
        None,
    );

    let mut uninterrupted = Simulation::new(params.clone());
    uninterrupted.enforce_structure_constraint = true;
    uninterrupted.dt_cap = 0.005;
    seed_v7_compartment(&mut uninterrupted, 22.0, 0.6);
    for _ in 0..200 {
        if !uninterrupted.step() {
            break;
        }
    }

    let ckpt_dir = output.join("checkpoints");
    fs::create_dir_all(&ckpt_dir)?;
    let ckpt_path = ckpt_dir.join("checkpoint_gate0.json");
    let activation = ActivationPotentialLedger::new(0.0);
    let convergence = ConvergenceCounter {
        consecutive_qualifying: 0,
        required: 3,
        windows: vec![],
    };
    let ckpt = build_checkpoint(
        &uninterrupted,
        uninterrupted.substep,
        &identity,
        &source_commit,
        &binary_sha,
        &activation,
        &convergence,
        None,
        &[],
    );
    write_governed_checkpoint(&ckpt_path, &ckpt)?;

    uninterrupted
        .surface_accounting
        .begin_window_local(uninterrupted.substep, uninterrupted.sim_time);
    for _ in 0..150 {
        if !uninterrupted.step() {
            break;
        }
    }
    let rates_unint = WindowLocalSurfaceRates::from_sim(&uninterrupted);

    let mut restored = Simulation::new(params);
    let loaded = load_governed_checkpoint(&ckpt_path)?;
    restore_governed_simulation(&mut restored, &loaded)?;
    restored.enforce_structure_constraint = true;
    // Match numerical ceiling used by the uninterrupted path (checkpoint may store dt > MAX_DT).
    restored.dt_cap = 0.005;
    for _ in 0..150 {
        if !restored.step() {
            break;
        }
    }
    let rates_rest = WindowLocalSurfaceRates::from_sim(&restored);
    let (max_abs, ok) = surface_rates_parity(&rates_unint, &rates_rest);

    let body = json!({
        "project_directive": "D-027",
        "gate": 0,
        "source_commit": source_commit,
        "binary_hash": binary_sha,
        "k_ads_frozen": D025_FROZEN_K_ADS,
        "pass": ok,
        "max_abs_diff": max_abs,
        "uninterrupted_rates": rates_unint,
        "restored_rates": rates_rest,
        "conclusion": if ok { "D027_CHECKPOINT_LEDGER_PASS" } else { "D027_CHECKPOINT_LEDGER_FAILURE" },
    });
    atomic_write_json(&output.join("ledger_restore.json"), &body)?;
    Ok(body)
}

/// Gate 1: adsorption basis across governed states + portability.
pub fn run_gate1_adsorption_basis(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let source_commit = git_commit_hash();

    let labeled = vec![
        (
            "d024_fixed_interface_r22".to_string(),
            fixed_interface_r22_state()?,
        ),
        (
            "d025_dynamic_r22_endpoint".to_string(),
            dynamic_r22_endpoint_state()?,
        ),
        (
            "d026_stage_e_10000".to_string(),
            restore_stage_e_checkpoint(10_000)?,
        ),
        (
            "d026_stage_e_25000".to_string(),
            restore_stage_e_checkpoint(25_000)?,
        ),
        (
            "d026_stage_e_100000".to_string(),
            restore_stage_e_checkpoint(100_000)?,
        ),
        (
            "d026_stage_e_200000".to_string(),
            restore_stage_e_checkpoint(200_000)?,
        ),
    ];

    let mut reports = Vec::new();
    for (label, sim) in &labeled {
        let r = compute_adsorption_basis_labeled(sim, label);
        atomic_write_json(&output.join(format!("{label}.json")), &json!(r))?;
        reports.push(r);
    }
    let portability = classify_adsorption_portability(&reports);
    let body = json!({
        "project_directive": "D-027",
        "gate": 1,
        "source_commit": source_commit,
        "frozen_k_ads_d024": frozen_k_ads_d024(),
        "reports": reports,
        "portability": portability,
        "pass": portability.portable,
        "conclusion": portability.conclusion,
    });
    atomic_write_json(&output.join("adsorption_basis.json"), &body)?;
    Ok(body)
}

/// Gate 2: exactly three analytical candidates from median required rate.
pub fn run_gate2_candidates(
    output: &Path,
    gate1: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let portability: chemistry_core::d027_analysis::PortabilityResult =
        serde_json::from_value(gate1["portability"].clone())?;
    let windows: Vec<String> = gate1["reports"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|r| r["label"].as_str().map(|s| s.to_string()))
        .collect();
    let candidates = match generate_analytical_candidates(&portability, "d024-sealed", &windows) {
        Ok(c) => c,
        Err(concl) => {
            let body = json!({
                "project_directive": "D-027",
                "gate": 2,
                "pass": false,
                "conclusion": concl,
                "candidates": [],
            });
            atomic_write_json(&output.join("analytical_candidates.json"), &body)?;
            return Ok(body);
        }
    };
    assert_eq!(candidates.len(), D027_CANDIDATE_SCALES.len());
    let body = json!({
        "project_directive": "D-027",
        "gate": 2,
        "pass": true,
        "analytical_center": portability.median_k_ads_required,
        "ratio_to_d024_k_ads": portability.median_k_ads_required / frozen_k_ads_d024().max(f64::EPSILON),
        "candidates": candidates,
        "conclusion": "D027_CANDIDATES_GENERATED",
    });
    atomic_write_json(&output.join("analytical_candidates.json"), &body)?;
    Ok(body)
}

/// Isolated surface-renewal screen (Gate 4) for one candidate.
pub fn run_isolated_surface_candidate(
    k_ads: f64,
    steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    params.k_ads = k_ads;
    params.d008_stage_b_enabled = true;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    // Burn-in then measure a late window for sustained adsorption–turnover balance.
    let burn_in = steps.saturating_mul(2) / 3;
    let measure = steps.saturating_sub(burn_in).max(200);
    for _ in 0..burn_in {
        if !sim.step() {
            break;
        }
    }
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut theta_series = Vec::new();
    for _ in 0..measure {
        if !sim.step() {
            break;
        }
        if sim.substep % 20 == 0 {
            let sample = chemistry_core::sample_stage_e_observability(&sim);
            theta_series.push(sample.surface.mean_theta_gamma);
        }
    }
    let rates = WindowLocalSurfaceRates::from_sim(&sim);
    let q = surface_balance_q(rates.adsorption, rates.gamma_turnover);
    let loc = gamma_localization(&sim);
    let late_mean = if theta_series.is_empty() {
        0.0
    } else {
        theta_series.iter().sum::<f64>() / theta_series.len() as f64
    };
    let theta_span = if theta_series.len() >= 2 {
        let min_t = theta_series.iter().copied().fold(f64::INFINITY, f64::min);
        let max_t = theta_series.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        max_t - min_t
    } else {
        0.0
    };
    // Sustained balance: late-window Q near 1; occupancy not collapsing; localization held.
    let occupancy_stable = theta_span <= 0.10 * late_mean.max(0.1) || theta_span < 0.05;
    let pass = loc >= 0.98
        && rates.adsorption > 0.0
        && rates.gamma_turnover > 0.0
        && (0.98..=1.02).contains(&q)
        && occupancy_stable
        && late_mean > 0.05
        && sim.fields.precursor.iter().all(|v| v.is_finite())
        && sim.fields.membrane.iter().all(|v| v.is_finite() && *v >= 0.0);
    Ok(json!({
        "k_ads": k_ads,
        "steps": sim.substep,
        "burn_in_steps": burn_in,
        "measure_steps": measure,
        "gamma_localization": loc,
        "q_surface": q,
        "rates": rates,
        "late_mean_theta": late_mean,
        "theta_span": theta_span,
        "occupancy_stable": occupancy_stable,
        "pass": pass,
    }))
}

pub fn run_gate4_isolated_surface(
    output: &Path,
    candidates: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut results = Vec::new();
    let mut promoted = None;
    let mut sorted = candidates["candidates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    sorted.sort_by(|a, b| {
        a["scale"]
            .as_f64()
            .partial_cmp(&b["scale"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for c in sorted {
        let k = c["k_ads"].as_f64().unwrap_or(0.0);
        let id = c["candidate_id"].as_str().unwrap_or("unknown");
        let r = run_isolated_surface_candidate(k, 12_000)?;
        let pass = r["pass"].as_bool().unwrap_or(false);
        let entry = json!({
            "candidate_id": id,
            "result": r,
        });
        atomic_write_json(&output.join(format!("{id}.json")), &entry)?;
        results.push(entry);
        if pass && promoted.is_none() {
            promoted = Some(c);
        }
    }
    let pass = promoted.is_some();
    let body = json!({
        "project_directive": "D-027",
        "gate": 4,
        "pass": pass,
        "promoted": promoted,
        "results": results,
        "conclusion": if pass { "D027_ISOLATED_SURFACE_RENEWAL_PASS" } else { "D027_ISOLATED_SURFACE_RENEWAL_FAILURE" },
    });
    atomic_write_json(&output.join("isolated_surface.json"), &body)?;
    Ok(body)
}

/// Full pipeline through early stop gates (0–4 minimum).
pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let root = resolve_path(output_root);
    fs::create_dir_all(&root)?;

    let gate0 = run_gate0_ledger_restore(&root.join("ledger_restore"))?;
    if !gate0["pass"].as_bool().unwrap_or(false) {
        let conclusion = "D027_CHECKPOINT_LEDGER_FAILURE";
        let manifest = json!({
            "project_directive": "D-027",
            "conclusion": conclusion,
            "gate0": gate0,
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate1 = run_gate1_adsorption_basis(&root.join("adsorption_basis"))?;
    if !gate1["pass"].as_bool().unwrap_or(false) {
        let conclusion = "D027_ADSORPTION_LAW_NOT_PORTABLE";
        let manifest = json!({
            "project_directive": "D-027",
            "conclusion": conclusion,
            "gate0": gate0,
            "gate1": gate1,
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate2 = run_gate2_candidates(&root.join("analytical_candidates"), &gate1)?;
    if !gate2["pass"].as_bool().unwrap_or(false) {
        let manifest = json!({
            "project_directive": "D-027",
            "conclusion": gate2["conclusion"],
            "gate0": gate0,
            "gate1": gate1,
            "gate2": gate2,
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate4 = run_gate4_isolated_surface(&root.join("isolated_surface"), &gate2)?;
    let conclusion = if gate4["pass"].as_bool().unwrap_or(false) {
        "D027_PARTIAL_GATES_0_4_PASS"
    } else {
        "D027_ISOLATED_SURFACE_RENEWAL_FAILURE"
    };
    let manifest = json!({
        "project_directive": "D-027",
        "agent_memory_directive": "D-20260717-d027-coupled-surface-renewal",
        "source_commit": git_commit_hash(),
        "conclusion": conclusion,
        "gate0": gate0,
        "gate1": gate1,
        "gate2": gate2,
        "gate4": gate4,
        "note": "Gates 3/5–12 continue only after Gate 4 promotion",
    });
    atomic_write_json(&root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
