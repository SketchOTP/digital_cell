//! D-028 bracketed surface-renewal root recovery runner.

use crate::d013::{
    atomic_write_json, load_governed_checkpoint, restore_governed_simulation,
};
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::d014_numerics::D014_DT_FLOOR;
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::WindowLocalSurfaceRates;
use chemistry_core::d028_analysis::{
    adsorption_response_monotonic, frozen_d027_monotonicity_holds, gate0_endpoint_ok,
    regula_falsi_trial, solve_bracketed_root, BracketEndpoints, SurfaceBalanceMetrics,
    D028_K_ADS_0_5X, D028_K_ADS_1X, D028_K_ADS_2X, D028_MAX_NEW_CANDIDATES, D028_Q_0_5X,
    D028_Q_1X, D028_Q_2X,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ISOLATED_STEPS: u64 = 12_000;
const PORTABILITY_MEASURE_STEPS: u64 = 2_000;
const AGENT_MEMORY_ID: &str = "D-20260717-d028-bracketed-surface-renewal-root";

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
    std::env::current_exe()
        .ok()
        .and_then(|p| fs::read(p).ok())
        .map(|b| chemistry_core::sha256_hex(&b))
        .unwrap_or_else(|| "unknown".into())
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

/// Isolated fixed-interface assay with Q and g on a common late window.
pub fn run_isolated_qg(k_ads: f64, steps: u64) -> Result<Value, Box<dyn std::error::Error>> {
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
    let mut s_mass_sum = 0.0;
    let mut s_samples = 0u64;
    for _ in 0..measure {
        if !sim.step() {
            break;
        }
        if sim.substep % 20 == 0 {
            let sample = chemistry_core::sample_stage_e_observability(&sim);
            theta_series.push(sample.surface.mean_theta_gamma);
            s_mass_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            s_samples += 1;
        }
    }
    let rates = WindowLocalSurfaceRates::from_sim(&sim);
    let mean_s = if s_samples > 0 {
        s_mass_sum / s_samples as f64
    } else {
        total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    let metrics = SurfaceBalanceMetrics::from_rates(rates.adsorption, rates.gamma_turnover, mean_s);
    let loc = gamma_localization(&sim);
    let late_mean = if theta_series.is_empty() {
        0.0
    } else {
        theta_series.iter().sum::<f64>() / theta_series.len() as f64
    };
    let theta_span = if theta_series.len() >= 2 {
        let min_t = theta_series.iter().copied().fold(f64::INFINITY, f64::min);
        let max_t = theta_series
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        max_t - min_t
    } else {
        0.0
    };
    let occupancy_stable = theta_span <= 0.10 * late_mean.max(0.1) || theta_span < 0.05;
    let p_bounded = sim.fields.precursor.iter().all(|v| v.is_finite() && *v >= 0.0);
    let s_bounded = sim
        .fields
        .membrane
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0);
    let no_sat_lock = late_mean < 0.999;
    let pass = metrics.is_balanced()
        && loc >= 0.98
        && rates.adsorption > 0.0
        && rates.gamma_turnover > 0.0
        && occupancy_stable
        && late_mean > 0.05
        && late_mean < 1.0
        && p_bounded
        && s_bounded
        && no_sat_lock
        && sim.min_attempted_dt > D014_DT_FLOOR * 2.0;
    Ok(json!({
        "k_ads": k_ads,
        "steps": sim.substep,
        "burn_in_steps": burn_in,
        "measure_steps": measure,
        "common_window": {
            "baseline_substep": sim.surface_accounting.window_baseline_substep,
            "baseline_time": sim.surface_accounting.window_baseline_time,
            "window_dt": rates.window_dt,
            "accepted_steps_in_window": rates.accepted_steps_in_window,
        },
        "gamma_localization": loc,
        "q_surface": metrics.q_surface,
        "g_surface": metrics.g_surface,
        "f_balance": metrics.f_balance,
        "mean_s_mass": mean_s,
        "rates": rates,
        "late_mean_theta": late_mean,
        "theta_span": theta_span,
        "occupancy_stable": occupancy_stable,
        "p_bounded": p_bounded,
        "s_bounded": s_bounded,
        "no_saturation_lock": no_sat_lock,
        "timestep_floor": sim.min_attempted_dt <= D014_DT_FLOOR * 2.0,
        "accounting_closed": sim.accounting.cumulative_within_tolerance(),
        "pass": pass,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "candidate_hash": chemistry_core::sha256_hex(
            format!("{k_ads:.17e}|{}|{}", metrics.q_surface, metrics.g_surface).as_bytes()
        ),
    }))
}

fn load_d027_isolated() -> Result<Value, Box<dyn std::error::Error>> {
    let path = resolve_path(Path::new(
        "experiments/generated/d027/isolated_surface/isolated_surface.json",
    ));
    let text = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&text)?)
}

fn load_d027_exact_k(scale: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let path = resolve_path(Path::new(
        "experiments/generated/d027/analytical_candidates/analytical_candidates.json",
    ));
    let v: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let want = format!("d027-ads-{scale}");
    for c in v["candidates"].as_array().into_iter().flatten() {
        let id = c["candidate_id"].as_str().unwrap_or("");
        if id == want {
            return Ok(c["k_ads"].as_f64().unwrap());
        }
    }
    Err(format!("missing D-027 candidate {want}").into())
}

/// Gate 0: reproduce D-027 1× / 2× isolated bracket; verify monotonicity with 0.5×.
pub fn run_gate0_bracket_reproduction(
    output: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;

    let k_half = load_d027_exact_k("0.5x").unwrap_or(D028_K_ADS_0_5X);
    let k_low = load_d027_exact_k("1x").unwrap_or(D028_K_ADS_1X);
    let k_high = load_d027_exact_k("2x").unwrap_or(D028_K_ADS_2X);

    // Prefer artifact machine equality.
    assert!((k_low - D028_K_ADS_1X).abs() < 1e-15);
    assert!((k_high - D028_K_ADS_2X).abs() < 1e-15);

    let frozen = load_d027_isolated().ok();
    // Gate 0 requires live reproduction of 1× and 2× only; 0.5× uses frozen D-027 Q.
    let r_low = run_isolated_qg(k_low, ISOLATED_STEPS)?;
    let r_high = run_isolated_qg(k_high, ISOLATED_STEPS)?;

    let q_half = D028_Q_0_5X;
    let q_low = r_low["q_surface"].as_f64().unwrap_or(f64::NAN);
    let q_high = r_high["q_surface"].as_f64().unwrap_or(f64::NAN);

    let mono_frozen = frozen_d027_monotonicity_holds();
    let mono_live = adsorption_response_monotonic(&[q_half, q_low, q_high]);
    // Midpoint monotonicity check (one evaluation inside bracket).
    let k_mid = 0.5 * (k_low + k_high);
    let r_mid = run_isolated_qg(k_mid, ISOLATED_STEPS)?;
    let q_mid = r_mid["q_surface"].as_f64().unwrap_or(f64::NAN);
    let mono_with_mid = adsorption_response_monotonic(&[q_low, q_mid, q_high]);

    let lower_ok = gate0_endpoint_ok(k_low, q_low, true)
        && r_low["p_bounded"].as_bool().unwrap_or(false)
        && r_low["s_bounded"].as_bool().unwrap_or(false)
        && !r_low["timestep_floor"].as_bool().unwrap_or(true)
        && r_low["no_saturation_lock"].as_bool().unwrap_or(false);
    let upper_ok = gate0_endpoint_ok(k_high, q_high, false)
        && r_high["p_bounded"].as_bool().unwrap_or(false)
        && r_high["s_bounded"].as_bool().unwrap_or(false)
        && !r_high["timestep_floor"].as_bool().unwrap_or(true)
        && r_high["no_saturation_lock"].as_bool().unwrap_or(false);

    let bracket_ok = lower_ok && upper_ok && q_low < 1.0 && q_high > 1.0;
    let mono_ok = mono_frozen && mono_live && mono_with_mid;

    let conclusion = if !bracket_ok {
        "D028_ROOT_BRACKET_NOT_REPRODUCED"
    } else if !mono_ok {
        "D028_ADSORPTION_RESPONSE_NONMONOTONIC"
    } else {
        "D028_SURFACE_BALANCE_ROOT_BRACKETED"
    };
    let pass = bracket_ok && mono_ok;

    let body = json!({
        "project_directive": "D-028",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 0,
        "pass": pass,
        "conclusion": conclusion,
        "d027_historical_conclusion": "D027_ISOLATED_SURFACE_RENEWAL_FAILURE",
        "additional_record": "D027_SURFACE_BALANCE_ROOT_BRACKETED",
        "exact_machine_candidates": {
            "k_0_5x": k_half,
            "k_1x": k_low,
            "k_2x": k_high,
        },
        "frozen_d027_q": {
            "q_0_5x": D028_Q_0_5X,
            "q_1x": D028_Q_1X,
            "q_2x": D028_Q_2X,
        },
        "reproduced": {
            "1x": r_low,
            "2x": r_high,
            "midpoint": r_mid,
            "0_5x_frozen_q": q_half,
        },
        "monotonicity": {
            "frozen_points": mono_frozen,
            "live_three_points": mono_live,
            "live_with_midpoint": mono_with_mid,
            "q_sequence": [q_half, q_low, q_mid, q_high],
        },
        "bracket": {
            "k_low": k_low,
            "q_low": q_low,
            "k_high": k_high,
            "q_high": q_high,
            "straddles_unity": q_low < 1.0 && q_high > 1.0,
        },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "d027_frozen_artifact_present": frozen.is_some(),
        "seed_and_horizon": {
            "radius": 22.0,
            "steps": ISOLATED_STEPS,
            "identical_across_candidates": true,
        },
    });
    atomic_write_json(&output.join("bracket_reproduction.json"), &body)?;
    // Preserve note that D-027 conclusion is not revised.
    atomic_write_json(
        &output.join("d027_preservation_note.json"),
        &json!({
            "d027_conclusion_unchanged": "D027_ISOLATED_SURFACE_RENEWAL_FAILURE",
            "d028_additional_record": "D027_SURFACE_BALANCE_ROOT_BRACKETED",
            "starting_commit": "15d46f2",
            "failure_tag": "D-027-surface-renewal-fail",
        }),
    )?;
    Ok(body)
}

/// Gate 1: safeguarded regula-falsi / bisection (max 4 new candidates).
pub fn run_gate1_root_solve(
    output: &Path,
    gate0: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;

    let k_low = gate0["bracket"]["k_low"].as_f64().unwrap_or(D028_K_ADS_1X);
    let q_low = gate0["bracket"]["q_low"].as_f64().unwrap_or(D028_Q_1X);
    let k_high = gate0["bracket"]["k_high"]
        .as_f64()
        .unwrap_or(D028_K_ADS_2X);
    let q_high = gate0["bracket"]["q_high"].as_f64().unwrap_or(D028_Q_2X);
    let initial = BracketEndpoints {
        k_low,
        q_low,
        k_high,
        q_high,
    };
    let first_trial = regula_falsi_trial(k_low, q_low, k_high, q_high);

    let mut eval_log: Vec<Value> = Vec::new();
    let solve = solve_bracketed_root(initial, |k| {
        let r = run_isolated_qg(k, ISOLATED_STEPS).expect("isolated assay");
        let q = r["q_surface"].as_f64().unwrap_or(f64::NAN);
        let g = r["g_surface"].as_f64().unwrap_or(f64::NAN);
        eval_log.push(r);
        (q, g)
    });

    for (i, r) in eval_log.iter().enumerate() {
        atomic_write_json(&output.join(format!("candidate_{:02}.json", i + 1)), r)?;
    }
    let solve_json = serde_json::to_value(&solve)?;
    atomic_write_json(&output.join("root_iterations.json"), &solve_json)?;

    let body = json!({
        "project_directive": "D-028",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 1,
        "pass": solve.pass,
        "conclusion": solve.conclusion,
        "max_new_candidates": D028_MAX_NEW_CANDIDATES,
        "first_regula_falsi_trial": first_trial,
        "initial_bracket": initial,
        "solve": solve_json,
        "evaluations": eval_log,
        "selected_k_ads": solve.selected_k_ads,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("root_solve.json"), &body)?;
    Ok(body)
}

/// Gate 2: local ±2% robustness around selected root.
pub fn run_gate2_local_robustness(
    output: &Path,
    selected_k: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let k_minus = selected_k * 0.98;
    let k_plus = selected_k * 1.02;
    let r_m = run_isolated_qg(k_minus, ISOLATED_STEPS)?;
    let r_c = run_isolated_qg(selected_k, ISOLATED_STEPS)?;
    let r_p = run_isolated_qg(k_plus, ISOLATED_STEPS)?;
    let q_m = r_m["q_surface"].as_f64().unwrap_or(f64::NAN);
    let q_c = r_c["q_surface"].as_f64().unwrap_or(f64::NAN);
    let q_p = r_p["q_surface"].as_f64().unwrap_or(f64::NAN);
    let ordered = q_m < q_c + 1e-6 && q_c < q_p + 1e-6;
    let center_pass = r_c["pass"].as_bool().unwrap_or(false);
    let bounded = |r: &Value| {
        r["p_bounded"].as_bool().unwrap_or(false)
            && r["s_bounded"].as_bool().unwrap_or(false)
            && r["no_saturation_lock"].as_bool().unwrap_or(false)
            && r["gamma_localization"].as_f64().unwrap_or(0.0) >= 0.98
            && !r["timestep_floor"].as_bool().unwrap_or(true)
    };
    let pass = center_pass && ordered && bounded(&r_m) && bounded(&r_c) && bounded(&r_p);
    let body = json!({
        "project_directive": "D-028",
        "gate": 2,
        "pass": pass,
        "selected_k_ads": selected_k,
        "minus": r_m,
        "center": r_c,
        "plus": r_p,
        "q_order": [q_m, q_c, q_p],
        "ordered": ordered,
        "conclusion": if pass { "D028_LOCAL_ROBUSTNESS_PASS" } else { "D028_FAIL" },
        "source_commit": git_commit_hash(),
    });
    atomic_write_json(&output.join("local_robustness.json"), &body)?;
    Ok(body)
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

/// Fixed-window renewal assay from a governed state with selected k_ads.
fn renewal_assay_from_state(
    mut sim: Simulation,
    selected_k: f64,
    label: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    sim.params.k_ads = selected_k;
    sim.dt_cap = 0.005;
    // Burn-in under selected k_ads so the late window reflects the new kinetics on this state.
    let settle = 4_000u64;
    for _ in 0..settle {
        if !sim.step() {
            break;
        }
    }
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut s_sum = 0.0;
    let mut n_s = 0u64;
    let mut theta_start = None;
    let mut theta_end = None;
    for _ in 0..PORTABILITY_MEASURE_STEPS {
        if !sim.step() {
            break;
        }
        if sim.substep % 20 == 0 {
            s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            n_s += 1;
            let sample = chemistry_core::sample_stage_e_observability(&sim);
            if theta_start.is_none() {
                theta_start = Some(sample.surface.mean_theta_gamma);
            }
            theta_end = Some(sample.surface.mean_theta_gamma);
        }
    }
    let rates = WindowLocalSurfaceRates::from_sim(&sim);
    let mean_s = if n_s > 0 {
        s_sum / n_s as f64
    } else {
        total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    let m = SurfaceBalanceMetrics::from_rates(rates.adsorption, rates.gamma_turnover, mean_s);
    let basis = chemistry_core::compute_adsorption_basis_labeled(&sim, label);
    let q_ok = (0.90..=1.10).contains(&m.q_surface);
    let ads_positive = rates.adsorption > 0.0 && basis.b_ads > 0.0;
    let gamma_positive = rates.gamma_turnover > 0.0;
    let p_available = basis.mean_p_near_interface > 0.0;
    let no_sat_lock = basis.mean_saturation_factor > 1e-6;
    let no_floor = sim.min_attempted_dt > D014_DT_FLOOR * 2.0;
    let toward_balance = {
        // Surface flow moves toward balance rather than away: |f| shrinking vs cold open.
        // Diagnostic: Q in [0.90,1.10] or g sign consistent with closing the Q gap.
        q_ok || (m.f_balance < 0.0 && m.g_surface > 0.0) || (m.f_balance > 0.0 && m.g_surface < 0.0)
    };
    let valid = ads_positive
        && gamma_positive
        && p_available
        && no_sat_lock
        && no_floor
        && rates.adsorption.is_finite()
        && rates.gamma_turnover.is_finite();
    let pass = valid && toward_balance && q_ok;
    Ok(json!({
        "label": label,
        "k_ads": selected_k,
        "q_surface": m.q_surface,
        "g_surface": m.g_surface,
        "f_balance": m.f_balance,
        "b_ads": basis.b_ads,
        "l_gamma": basis.l_gamma,
        "mean_p": basis.mean_p_near_interface,
        "mean_sat": basis.mean_saturation_factor,
        "theta_start": theta_start,
        "theta_end": theta_end,
        "ads_positive": ads_positive,
        "gamma_turnover_positive": gamma_positive,
        "p_available": p_available,
        "no_saturation_lock": no_sat_lock,
        "valid": valid,
        "toward_balance": toward_balance,
        "q_in_portability_band": q_ok,
        "pass": pass,
        "measure_steps": PORTABILITY_MEASURE_STEPS,
        "settle_steps": settle,
    }))
}

/// Gate 3: six-state adsorption-basis portability (fixed-window renewal assay).
pub fn run_gate3_portability(
    output: &Path,
    selected_k: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;

    let labeled = vec![
        ("d024_fixed_interface_r22", fixed_interface_r22_state()?),
        ("d025_dynamic_r22_endpoint", dynamic_r22_endpoint_state()?),
        ("d026_stage_e_10000", restore_stage_e_checkpoint(10_000)?),
        ("d026_stage_e_25000", restore_stage_e_checkpoint(25_000)?),
        ("d026_stage_e_100000", restore_stage_e_checkpoint(100_000)?),
        ("d026_stage_e_200000", restore_stage_e_checkpoint(200_000)?),
    ];

    let mut results = Vec::new();
    let mut pass_count = 0usize;
    for (label, sim) in labeled {
        let entry = renewal_assay_from_state(sim, selected_k, label)?;
        if entry["pass"].as_bool().unwrap_or(false) {
            pass_count += 1;
        }
        atomic_write_json(&output.join(format!("{label}.json")), &entry)?;
        results.push(entry);
    }
    let pass = pass_count >= 5;
    let body = json!({
        "project_directive": "D-028",
        "gate": 3,
        "pass": pass,
        "pass_count": pass_count,
        "required": 5,
        "selected_k_ads": selected_k,
        "results": results,
        "conclusion": if pass { "D028_ROOT_PORTABLE" } else { "D028_ROOT_NOT_PORTABLE" },
        "source_commit": git_commit_hash(),
        "note": "States match D-027 Gate1 adsorption-basis sources; k_ads overridden to selected root",
    });
    atomic_write_json(&output.join("portability.json"), &body)?;
    Ok(body)
}

fn write_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-022-localization-not-recovered",
        "D-023-precursor-assembly-fail",
        "D-024-surface-density-pass",
        "D-025-surface-density-recovery-fail",
        "D-026-stage-e-recovery-fail",
        "D-027-surface-renewal-fail",
    ];
    let mut present = Vec::new();
    for t in tags {
        let ok = Command::new("git")
            .args(["rev-parse", t])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        present.push(json!({"tag": t, "present": ok}));
    }
    let body = json!({
        "project_directive": "D-028",
        "branch": "d008-membrane-metabolic-closure",
        "starting_commit": "15d46f2",
        "starting_commit_full": git_commit_hash(),
        "d027_failure_tag": "D-027-surface-renewal-fail",
        "tags": present,
        "equation_version": "membrane_metabolism_v7_surface_density",
        "note": "D-021–D-027 commits/tags/artifacts preserved; not rewritten",
    });
    atomic_write_json(&output.join("preservation_manifest.json"), &body)?;
    Ok(body)
}

/// Full early pipeline: preservation → Gate0 → Gate1 → Gate2 → Gate3; stop on fail.
pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let root = resolve_path(output_root);
    fs::create_dir_all(&root)?;
    for sub in [
        "preservation",
        "bracket_reproduction",
        "root_iterations",
        "local_robustness",
        "portability",
        "dynamic_r22",
        "stage_b",
        "stage_d",
        "stage_e_surface",
        "stage_e_full",
        "productive_candidates",
        "radius_validation",
        "robustness",
        "accounting",
    ] {
        fs::create_dir_all(root.join(sub))?;
    }

    let preservation = write_preservation(&root.join("preservation"))?;

    let gate0 = run_gate0_bracket_reproduction(&root.join("bracket_reproduction"))?;
    if !gate0["pass"].as_bool().unwrap_or(false) {
        let conclusion = gate0["conclusion"].as_str().unwrap_or("D028_FAIL");
        let manifest = json!({
            "project_directive": "D-028",
            "agent_memory_directive": AGENT_MEMORY_ID,
            "conclusion": conclusion,
            "stopped_at_gate": 0,
            "preservation": preservation,
            "gate0": gate0,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate1 = run_gate1_root_solve(&root.join("root_iterations"), &gate0)?;
    if !gate1["pass"].as_bool().unwrap_or(false) {
        let conclusion = gate1["conclusion"]
            .as_str()
            .unwrap_or("D028_NO_ISOLATED_SURFACE_BALANCE_ROOT");
        let manifest = json!({
            "project_directive": "D-028",
            "agent_memory_directive": AGENT_MEMORY_ID,
            "conclusion": conclusion,
            "stopped_at_gate": 1,
            "preservation": preservation,
            "gate0": gate0,
            "gate1": gate1,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let selected = gate1["selected_k_ads"].as_f64().ok_or("missing selected k_ads")?;
    let gate2 = run_gate2_local_robustness(&root.join("local_robustness"), selected)?;
    if !gate2["pass"].as_bool().unwrap_or(false) {
        let manifest = json!({
            "project_directive": "D-028",
            "agent_memory_directive": AGENT_MEMORY_ID,
            "conclusion": "D028_FAIL",
            "stopped_at_gate": 2,
            "selected_k_ads": selected,
            "preservation": preservation,
            "gate0": gate0,
            "gate1": gate1,
            "gate2": gate2,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate3 = run_gate3_portability(&root.join("portability"), selected)?;
    if !gate3["pass"].as_bool().unwrap_or(false) {
        let manifest = json!({
            "project_directive": "D-028",
            "agent_memory_directive": AGENT_MEMORY_ID,
            "conclusion": "D028_ROOT_NOT_PORTABLE",
            "stopped_at_gate": 3,
            "selected_k_ads": selected,
            "preservation": preservation,
            "gate0": gate0,
            "gate1": gate1,
            "gate2": gate2,
            "gate3": gate3,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let manifest = json!({
        "project_directive": "D-028",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "conclusion": "D028_PARTIAL_GATES_0_3_PASS",
        "stopped_at_gate": null,
        "selected_k_ads": selected,
        "note": "Gates 4–11 continue only after Gate 3 portability; adsorption law unchanged",
        "preservation": preservation,
        "gate0": gate0,
        "gate1": gate1,
        "gate2": gate2,
        "gate3": gate3,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
