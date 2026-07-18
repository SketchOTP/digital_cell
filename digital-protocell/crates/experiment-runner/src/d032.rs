//! D-032 activated nonequilibrium surface assembly runner.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SurfaceExchangeIntegrator};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::{surface_balance_q, WindowLocalSurfaceRates};
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d031_analysis::{d030_identified_candidate, D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d032_analysis::{
    bracketed_interpolate, estimate_k_active_required, generate_active_candidates,
    reconstruct_active_rate, v9_params, ActiveCandidate, PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, surface_occupancy_theta, total_surface_mass,
    InterfaceGeometryCell, SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260718-d032-activated-surface-assembly";
const REGEN_HORIZONS: &[u64] = &[10_000, 25_000, 50_000, 100_000, 150_000, 200_000];
const ISOLATED_HORIZONS: &[u64] = &[2_000, 10_000, 25_000, 50_000, 100_000, 200_000];

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

fn disk_status() -> Value {
    let out = Command::new("df").args(["-B1", "."]).output().ok();
    if let Some(o) = out {
        if let Ok(text) = String::from_utf8(o.stdout) {
            if let Some(line) = text.lines().nth(1) {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 4 {
                    let total: u64 = cols[1].parse().unwrap_or(0);
                    let used: u64 = cols[2].parse().unwrap_or(0);
                    let avail: u64 = cols[3].parse().unwrap_or(0);
                    return json!({
                        "total_bytes": total,
                        "used_bytes": used,
                        "available_bytes": avail,
                        "available_gb": avail as f64 / 1e9,
                    });
                }
            }
        }
    }
    json!({"available_bytes": null})
}

fn tag_exists(tag: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn commit_exists(rev: &str) -> bool {
    Command::new("git")
        .args(["cat-file", "-e", &format!("{rev}^{{commit}}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn v8_isolated_params() -> Result<chemistry_core::SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    Ok(p)
}

fn v9_isolated_params(k_active: f64) -> Result<chemistry_core::SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.k_active = k_active;
    Ok(p)
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

fn field_mass(sim: &Simulation, field: &[f64]) -> f64 {
    field
        .iter()
        .enumerate()
        .filter(|(i, _)| sim.grid.in_dish(*i))
        .map(|(_, v)| *v)
        .sum()
}

fn theta_stats(sim: &Simulation) -> Value {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut thetas = Vec::new();
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let g = sim.fields.membrane[idx] / d;
        thetas.push(surface_occupancy_theta(g, sim.params.gamma_max));
    }
    if thetas.is_empty() {
        return json!({
            "mean": 0.0, "min": 0.0, "max": 0.0,
            "q25": 0.0, "q50": 0.0, "q75": 0.0, "n": 0
        });
    }
    thetas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_t = thetas.len();
    let mean = thetas.iter().sum::<f64>() / n_t as f64;
    let q = |f: f64| thetas[((f * (n_t as f64 - 1.0)).round() as usize).min(n_t - 1)];
    json!({
        "mean": mean,
        "min": thetas[0],
        "max": thetas[n_t - 1],
        "q25": q(0.25),
        "q50": q(0.50),
        "q75": q(0.75),
        "n": n_t,
    })
}

/// Full renewal-window observability (repairs D-031 omission).
fn renewal_window_observability(sim: &Simulation, accepted_in_window: u64) -> Value {
    let rates = WindowLocalSurfaceRates::from_sim(sim);
    let wl = sim.surface_accounting.window_local();
    let mean_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let passive_net = wl.exchange_net;
    let active = wl.active_assembly;
    let turn = wl.gamma_decay_delta;
    let q_total = surface_balance_q(passive_net + active, turn);
    let g_surface = (passive_net + active - turn) / mean_s.max(f64::EPSILON);
    let material_residual = wl.active_assembly - wl.active_assembly_activation; // 1:1 ⇒ ~0 vs A
    // Activation residual: A consumed by assembly should equal active extent.
    let activation_residual = (wl.active_assembly - wl.active_assembly_activation).abs();
    json!({
        "p_mass": field_mass(sim, &sim.fields.precursor),
        "a_mass": field_mass(sim, &sim.fields.activated),
        "s_mass": mean_s,
        "w_mass": field_mass(sim, &sim.fields.waste),
        "theta": theta_stats(sim),
        "interface_measure": gamma_localization(sim), // localization as interface support proxy
        "localization": gamma_localization(sim),
        "passive_forward_exchange": wl.exchange_forward,
        "passive_reverse_exchange": wl.exchange_reverse,
        "passive_net_exchange": passive_net,
        "active_assembly": active,
        "biological_turnover": turn,
        "q_total": q_total,
        "g_surface": g_surface,
        "material_residual": material_residual,
        "activation_residual": activation_residual,
        "dissipation": wl.exchange_dissipation,
        "timestep": {
            "accepted_in_window": accepted_in_window,
            "dt": sim.dt,
            "substep": sim.substep,
            "sim_time": sim.sim_time,
            "last_reject": sim.last_reject_detail,
        },
        "rates": {
            "adsorption": rates.adsorption,
            "turnover": rates.gamma_turnover,
            "window_dt": rates.window_dt,
        }
    })
}

pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-022-localization-not-recovered",
        "D-023-precursor-assembly-fail",
        "D-024-surface-density-pass",
        "D-024-surface-density-pass-provenance-sealed",
        "D-025-surface-density-recovery-fail",
        "D-026-stage-e-recovery-fail",
        "D-027-surface-renewal-fail",
        "D-028-bracketed-renewal-fail",
        "D-029-reversible-exchange-fail",
        "D-030-exchange-identification-fail",
        "D-031-invariant-exchange-fail",
    ];
    let tag_status: Vec<Value> = tags
        .iter()
        .map(|t| json!({"tag": t, "present": tag_exists(t)}))
        .collect();
    let all_tags = tag_status.iter().all(|t| t["present"] == true);
    let commits = json!({
        "d031_source_run": "f7a3dca",
        "d031_source_present": commit_exists("f7a3dca"),
        "d031_result": "023378b",
        "d031_result_present": commit_exists("023378b"),
    });
    let disk = disk_status();
    let pass = all_tags
        && commits["d031_source_present"] == true
        && commits["d031_result_present"] == true;
    let body = json!({
        "project_directive": "D-032",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 0,
        "preservation": {
            "tags": tag_status,
            "commits": commits,
            "d031_binary_hash_expected": "6398bc2cd7aa0be386e6ac330864dee3df76ff07a07061a960268b488d272d39",
            "d031_binary_hash_current_runner": binary_hash(),
            "d031_conclusion": "D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED",
            "record": PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
            "frozen_exchange": {
                "alpha": D031_ALPHA_FROZEN,
                "beta": D031_BETA_FROZEN,
                "k_exchange": d030_identified_candidate().k_exchange,
                "K_exchange": d030_identified_candidate().k_exchange_eq,
            },
            "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
        },
        "disk": disk,
        "equation_version": EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly.as_str(),
        "surface_exchange_schema_version": 3,
        "active_assembly_schema_version": 1,
        "observability_repair": "renewal_window_records_p_a_s_w_theta_passive_active_turnover_q_g_residuals",
        "pass": pass,
        "conclusion": if pass { "D032_PRESERVATION_PASS" } else { "D032_PRESERVATION_OR_OBSERVABILITY_FAILURE" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("preservation.json"), &body)?;
    Ok(body)
}

/// Gate 2 — regenerate compact v8 isolated states and reconstruct k_active_required.
pub fn run_gate2_active_basis(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v8_isolated_params()?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }

    let mut estimates = Vec::new();
    let mut state_summaries = Vec::new();
    let mut accepted = 0u64;
    let mut steps_ok = true;

    for &horizon in REGEN_HORIZONS {
        while accepted < horizon && steps_ok {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
            if accepted % 5000 == 0 {
                eprintln!(
                    "D-032 Gate2 regen accepted={} target={}",
                    accepted, horizon
                );
            }
        }
        // Compact summary only — no full-field dump.
        let est = estimate_k_active_required(&format!("v8_isolated_{horizon}"), accepted, &sim);
        let summary = json!({
            "horizon": horizon,
            "accepted": accepted,
            "steps_ok": steps_ok,
            "p_mass": field_mass(&sim, &sim.fields.precursor),
            "a_mass": field_mass(&sim, &sim.fields.activated),
            "s_mass": total_surface_mass(&sim.grid, &sim.fields.membrane),
            "theta": theta_stats(&sim),
            "localization": gamma_localization(&sim),
            "r_required": est.r_required,
            "b_active": est.b_active,
            "k_active_required": est.k_active_required,
            "valid": est.valid,
            "reject_reason": est.reject_reason,
        });
        atomic_write_json(&output.join(format!("state_{horizon}.json")), &summary)?;
        state_summaries.push(summary);
        estimates.push(est);
        if !steps_ok {
            break;
        }
    }

    let rec = reconstruct_active_rate(estimates);
    let body = json!({
        "project_directive": "D-032",
        "gate": 2,
        "states": state_summaries,
        "reconstruction": rec,
        "pass": rec.portable,
        "conclusion": rec.conclusion,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    atomic_write_json(&output.join("active_basis.json"), &body)?;
    Ok(body)
}

/// Gate 3 — candidate identification from portable median.
pub fn run_gate3_candidates(
    output: &Path,
    median_k: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let cands = generate_active_candidates(median_k);
    let body = json!({
        "project_directive": "D-032",
        "gate": 3,
        "median_k_active": median_k,
        "candidates": cands,
        "max_candidates": 5,
        "analytical_bracket": [median_k * 0.5, median_k * 2.0],
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("candidates.json"), &body)?;
    Ok(body)
}

/// Gate 5 — isolated biological renewal under selected k_active (full observability).
pub fn run_gate5_isolated_renewal(
    output: &Path,
    k_active: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v9_isolated_params(k_active)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }

    let mut horizon_reports = Vec::new();
    let mut total_accepted = 0u64;
    let mut capacity_rejects = 0u64;
    let mut consecutive = 0usize;
    let mut steps_ok = true;

    for &horizon in ISOLATED_HORIZONS {
        while total_accepted < horizon && steps_ok {
            if !sim.step() {
                steps_ok = false;
                if sim.last_reject_detail.contains("CapacityExceeded") {
                    capacity_rejects += 1;
                }
                break;
            }
            total_accepted += 1;
            if total_accepted % 5000 == 0 {
                eprintln!(
                    "D-032 Gate5 progress accepted={} target={} k_active={}",
                    total_accepted, horizon, k_active
                );
            }
        }
        let window = 2_000u64;
        let mut windows = Vec::new();
        consecutive = 0;
        for _ in 0..3 {
            if !steps_ok {
                windows.push(json!({"ok": false, "accepted_in_window": 0}));
                continue;
            }
            sim.surface_accounting
                .begin_window_local(sim.substep, sim.sim_time);
            let mut accepted = 0u64;
            for _ in 0..window {
                if !sim.step() {
                    steps_ok = false;
                    if sim.last_reject_detail.contains("CapacityExceeded") {
                        capacity_rejects += 1;
                    }
                    break;
                }
                accepted += 1;
                total_accepted += 1;
            }
            let obs = renewal_window_observability(&sim, accepted);
            let q = obs["q_total"].as_f64().unwrap_or(0.0);
            let g = obs["g_surface"].as_f64().unwrap_or(0.0);
            let loc = obs["localization"].as_f64().unwrap_or(0.0);
            let ok = steps_ok
                && accepted >= window / 2
                && (0.98..=1.02).contains(&q)
                && g.abs() <= 1e-4
                && loc >= 0.98
                && obs["passive_forward_exchange"].as_f64().unwrap_or(0.0) > 0.0
                && obs["passive_reverse_exchange"].as_f64().unwrap_or(0.0) > 0.0
                && obs["active_assembly"].as_f64().unwrap_or(0.0) > 0.0
                && obs["biological_turnover"].as_f64().unwrap_or(0.0) > 0.0
                && obs["activation_residual"].as_f64().unwrap_or(1.0) < 1e-9;
            if ok {
                consecutive += 1;
            } else {
                consecutive = 0;
            }
            let mut row = obs;
            row.as_object_mut().unwrap().insert("ok".into(), json!(ok));
            windows.push(row);
        }
        let hr = json!({
            "horizon": horizon,
            "total_accepted": total_accepted,
            "steps_ok": steps_ok,
            "consecutive_ok": consecutive,
            "capacity_rejects": capacity_rejects,
            "windows": windows,
        });
        atomic_write_json(&output.join(format!("horizon_{horizon}.json")), &hr)?;
        horizon_reports.push(hr);
        eprintln!(
            "D-032 Gate5 horizon={} accepted={} consecutive_ok={}",
            horizon, total_accepted, consecutive
        );
        if consecutive >= 3 {
            break;
        }
        if !steps_ok {
            break;
        }
    }

    let pass = consecutive >= 3 && capacity_rejects == 0 && steps_ok;
    let conclusion = if pass {
        "D032_ISOLATED_RENEWAL_PASS"
    } else if !steps_ok && capacity_rejects > 0 {
        "D032_ACTIVE_ASSEMBLY_NUMERICAL_FAILURE"
    } else {
        "D032_ISOLATED_RENEWAL_FAILURE"
    };
    let body = json!({
        "project_directive": "D-032",
        "gate": 5,
        "k_active": k_active,
        "horizons": horizon_reports,
        "total_accepted": total_accepted,
        "capacity_rejects": capacity_rejects,
        "consecutive_ok": consecutive,
        "pass": pass,
        "conclusion": conclusion,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    atomic_write_json(&output.join("isolated_renewal.json"), &body)?;
    Ok(body)
}

/// Screen candidates: evaluate center first, then bracket; select smallest passing.
pub fn run_candidate_screen(
    output: &Path,
    median_k: f64,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut cands = generate_active_candidates(median_k);
    let mut results = Vec::new();
    let mut selected: Option<ActiveCandidate> = None;
    let mut q_by_k: Vec<(f64, f64)> = Vec::new();

    // Evaluate center (1.0×) first.
    cands.sort_by(|a, b| {
        (a.scale - 1.0)
            .abs()
            .partial_cmp(&(b.scale - 1.0).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for c in &cands {
        if results.len() >= 5 {
            break;
        }
        let out = output.join(&c.identity);
        // Short progressive screen: settle + burn + one late window.
        let params = v9_isolated_params(c.k_active)?;
        let mut sim = Simulation::new(params);
        sim.enforce_structure_constraint = true;
        sim.dt_cap = 0.005;
        seed_v7_compartment(&mut sim, 22.0, 0.6);
        let mut accepted = 0u64;
        let mut steps_ok = true;
        for _ in 0..D026_SETTLE_STEPS {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
        }
        let burn = max_steps.saturating_mul(2) / 3;
        while accepted < burn && steps_ok {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
        }
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        let window = 2_000u64;
        let mut win_acc = 0u64;
        for _ in 0..window {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            win_acc += 1;
            accepted += 1;
        }
        let obs = renewal_window_observability(&sim, win_acc);
        let q = obs["q_total"].as_f64().unwrap_or(0.0);
        let g = obs["g_surface"].as_f64().unwrap_or(0.0);
        let pass_window = steps_ok
            && (0.98..=1.02).contains(&q)
            && g.abs() <= 1e-4
            && obs["active_assembly"].as_f64().unwrap_or(0.0) > 0.0;
        q_by_k.push((c.k_active, q));
        let row = json!({
            "candidate": c,
            "accepted": accepted,
            "steps_ok": steps_ok,
            "q_total": q,
            "g_surface": g,
            "observability": obs,
            "pass_window": pass_window,
        });
        fs::create_dir_all(&out)?;
        atomic_write_json(&out.join("screen.json"), &row)?;
        results.push(row);
        if pass_window {
            selected = Some(c.clone());
            break;
        }
    }

    // Optional bracketed interpolation if one below and one above balance.
    if selected.is_none() && q_by_k.len() >= 2 {
        q_by_k.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for w in q_by_k.windows(2) {
            if let Some(k_star) = bracketed_interpolate(w[0].0, w[1].0, w[0].1, w[1].1) {
                if results.len() < 5 {
                    let c = ActiveCandidate {
                        identity: "k_active_bracket".into(),
                        k_active: k_star,
                        scale: k_star / median_k,
                    };
                    // Evaluate interpolated candidate once.
                    let out = run_gate5_isolated_renewal(
                        &output.join("bracket_eval"),
                        c.k_active,
                    )?;
                    results.push(json!({"candidate": c, "isolated": out}));
                    if out["pass"] == true {
                        selected = Some(ActiveCandidate {
                            identity: "k_active_bracket".into(),
                            k_active: k_star,
                            scale: k_star / median_k,
                        });
                    }
                }
                break;
            }
        }
    }

    let body = json!({
        "project_directive": "D-032",
        "gate": 3,
        "median_k_active": median_k,
        "results": results,
        "selected": selected,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("candidate_screen.json"), &body)?;
    Ok(body)
}

/// Pipeline: Gate0 → Gate2 → Gate3 screen → Gate5 (stop on fail).
pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    fs::create_dir_all(&output_root)?;

    let gate0 = run_gate0_preservation(&output_root.join("preservation"))?;
    if gate0["pass"] != true {
        let manifest = json!({
            "project_directive": "D-032",
            "conclusion": "D032_PRESERVATION_OR_OBSERVABILITY_FAILURE",
            "stopped_at_gate": 0,
            "gate0": gate0,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate2 = run_gate2_active_basis(&output_root.join("active_basis"))?;
    if gate2["pass"] != true {
        let manifest = json!({
            "project_directive": "D-032",
            "conclusion": gate2["conclusion"],
            "stopped_at_gate": 2,
            "gate0": gate0,
            "gate2": gate2,
            "record": PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let median = gate2["reconstruction"]["median_k_active"]
        .as_f64()
        .unwrap_or(f64::NAN);
    let _ = run_gate3_candidates(&output_root.join("candidates"), median)?;
    // Screen at 25k burn for candidate selection (ponytail: full Gate5 after select).
    let screen = run_candidate_screen(&output_root.join("candidates"), median, 25_000)?;
    let selected_k = screen["selected"]["k_active"].as_f64();
    let Some(k_active) = selected_k else {
        let manifest = json!({
            "project_directive": "D-032",
            "conclusion": "D032_ISOLATED_RENEWAL_FAILURE",
            "stopped_at_gate": 3,
            "gate0": gate0,
            "gate2": gate2,
            "screen": screen,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    };

    let gate5 = run_gate5_isolated_renewal(&output_root.join("isolated_renewal"), k_active)?;
    let conclusion = if gate5["pass"] == true {
        "D032_ISOLATED_RENEWAL_PASS"
    } else {
        gate5["conclusion"].as_str().unwrap_or("D032_FAIL")
    };
    let manifest = json!({
        "project_directive": "D-032",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "conclusion": conclusion,
        "stopped_at_gate": if gate5["pass"] == true { Value::Null } else { json!(5) },
        "selected_k_active": k_active,
        "gate0": {"pass": true},
        "gate2": {"pass": true, "median_k_active": median},
        "screen": screen["selected"],
        "gate5": {
            "pass": gate5["pass"],
            "conclusion": gate5["conclusion"],
            "total_accepted": gate5["total_accepted"],
        },
        "record": PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
        "v9": v9_params(k_active).equation_version.as_str(),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
        "next": if gate5["pass"] == true {
            "Gates 6–15 Stage B/C/D/E"
        } else {
            "stop — isolated renewal failed"
        },
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
