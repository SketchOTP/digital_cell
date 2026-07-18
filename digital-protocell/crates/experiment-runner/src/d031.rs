//! D-031 invariant-domain reversible exchange recovery runner.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SurfaceExchangeIntegrator};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::{surface_balance_q, WindowLocalSurfaceRates};
use chemistry_core::d029_analysis::{apply_exchange_candidate, ExchangeCandidate};
use chemistry_core::d030_analysis::{
    adsorption_matrix_specs, desorption_matrix_specs, recover_exchange_parameters,
    run_orthogonal_assay,
};
use chemistry_core::d031_analysis::{
    d030_identified_candidate, reproduce_capacity_failure, seed_d030_isolated_compartment,
    D031_ALPHA_FROZEN, D031_BETA_FROZEN,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, InterfaceGeometryCell,
    SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260718-d031-invariant-domain-surface-exchange";
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

fn identified() -> ExchangeCandidate {
    d030_identified_candidate()
}

fn v8_params(integrator: SurfaceExchangeIntegrator) -> Result<chemistry_core::SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, &identified());
    p.surface_exchange_integrator = integrator;
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

pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-022-localization-not-recovered",
        "D-023-precursor-assembly-fail",
        "D-024-surface-density-pass-provenance-sealed",
        "D-025-surface-density-recovery-fail",
        "D-026-stage-e-recovery-fail",
        "D-027-surface-renewal-fail",
        "D-028-bracketed-renewal-fail",
        "D-029-reversible-exchange-fail",
        "D-030-exchange-identification-fail",
    ];
    let tag_status: Vec<Value> = tags
        .iter()
        .map(|t| json!({"tag": t, "present": tag_exists(t)}))
        .collect();
    let body = json!({
        "project_directive": "D-031",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 0,
        "preservation": {
            "d030_result_commit": "921bd42",
            "d030_fail_tag": "D-030-exchange-identification-fail",
            "d030_fail_tag_present": tag_exists("D-030-exchange-identification-fail"),
            "historical_conclusion": "D030_TURNOVER_EXCHANGE_INCOMPATIBILITY",
            "operative_qualification": "D030_TURNOVER_EXCHANGE_INCOMPATIBILITY_NOT_ESTABLISHED_DUE_TO_ZERO_ACCEPTED_STEPS",
            "operative_status": "D030_NUMERICAL_CAPACITY_INTEGRATION_FAILURE",
            "tags": tag_status,
        },
        "branch": Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string()),
        "starting_commit": git_commit_hash(),
        "disk": disk_status(),
        "equation_version": EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange.as_str(),
        "surface_exchange_schema_version": 2,
        "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
        "alpha_frozen": D031_ALPHA_FROZEN,
        "beta_frozen": D031_BETA_FROZEN,
        "candidate": identified(),
    });
    atomic_write_json(&output.join("preservation.json"), &body)?;
    Ok(body)
}

pub fn run_gate0_capacity_failure(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let rec = reproduce_capacity_failure(
        |sim| seed_d030_isolated_compartment(sim, 22.0, 0.6),
        50_000,
    );
    let body = json!({
        "project_directive": "D-031",
        "gate": 0,
        "capacity_failure": rec,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("capacity_failure.json"), &body)?;
    Ok(body)
}

/// Gate 3 — D-030 identification regression under V2 integrator.
pub fn run_gate3_identification_regression(
    output: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let c = identified();
    let mut params = chemistry_core::SimParams::default();
    apply_exchange_candidate(&mut params, &c);
    params.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    let ads = adsorption_matrix_specs(&params);
    let des = desorption_matrix_specs(&params);
    let mut alphas = Vec::new();
    let mut betas = Vec::new();
    let mut a_by_q = vec![Vec::new(); 3];
    let mut b_by_q = vec![Vec::new(); 3];
    for (i, spec) in ads.iter().enumerate() {
        let r = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, spec)?;
        alphas.push(r.first.alpha_estimate);
        a_by_q[i / 3].push(r.first.alpha_estimate);
    }
    for (i, spec) in des.iter().enumerate() {
        let r = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, spec)?;
        betas.push(r.first.beta_estimate);
        b_by_q[i / 3].push(r.first.beta_estimate);
    }
    let rec = recover_exchange_parameters(&alphas, &betas, &a_by_q, &b_by_q);
    let alpha = rec.alpha_direct;
    let beta = rec.beta_direct;
    let alpha_ok = ((alpha - D031_ALPHA_FROZEN) / D031_ALPHA_FROZEN).abs() <= 0.02;
    let beta_ok = ((beta - D031_BETA_FROZEN) / D031_BETA_FROZEN).abs() <= 0.02;
    let pass = rec.identifiable && alpha_ok && beta_ok && rec.loo_ok;
    let body = json!({
        "project_directive": "D-031",
        "gate": 3,
        "pass": pass,
        "alpha_direct": alpha,
        "beta_direct": beta,
        "alpha_frozen": D031_ALPHA_FROZEN,
        "beta_frozen": D031_BETA_FROZEN,
        "alpha_rel_err": ((alpha - D031_ALPHA_FROZEN) / D031_ALPHA_FROZEN).abs(),
        "beta_rel_err": ((beta - D031_BETA_FROZEN) / D031_BETA_FROZEN).abs(),
        "identifiable": rec.identifiable,
        "loo_ok": rec.loo_ok,
        "alpha_q_norm_spread": rec.alpha_q_norm_spread,
        "beta_q_norm_spread": rec.beta_q_norm_spread,
        "bootstrap_spread_factor_alpha": rec.bootstrap_spread_factor_alpha,
        "bootstrap_spread_factor_beta": rec.bootstrap_spread_factor_beta,
        "conclusion": if pass { "D031_EXCHANGE_IDENTIFICATION_REGRESSION_PASS" } else { "D031_EXCHANGE_IDENTIFICATION_REGRESSION" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("identification_regression.json"), &body)?;
    Ok(body)
}

/// Short diagnostic: settle + 4k burn + one 2k window (accepted-step evidence).
pub fn run_gate4_short_diagnostic(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v8_params(SurfaceExchangeIntegrator::InvariantDomainV2)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    let mut accepted = 0u64;
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
        accepted += 1;
    }
    for _ in 0..4_000 {
        if !sim.step() {
            break;
        }
        accepted += 1;
    }
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let window = 2_000u64;
    let mut win_accepted = 0u64;
    let mut s_sum = 0.0;
    let mut n = 0u64;
    let mut steps_ok = true;
    for _ in 0..window {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        win_accepted += 1;
        accepted += 1;
        if sim.substep % 20 == 0 {
            s_sum += chemistry_core::surface_density::total_surface_mass(
                &sim.grid,
                &sim.fields.membrane,
            );
            n += 1;
        }
    }
    let rates = WindowLocalSurfaceRates::from_sim(&sim);
    let mean_s = if n > 0 {
        s_sum / n as f64
    } else {
        chemistry_core::surface_density::total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    let net = rates.adsorption;
    let turn = rates.gamma_turnover;
    let q = surface_balance_q(net, turn);
    let g = (net - turn) / mean_s.max(f64::EPSILON);
    let loc = gamma_localization(&sim);
    let wl = sim.surface_accounting.window_local();
    let body = json!({
        "project_directive": "D-031",
        "gate": "4_short_diagnostic",
        "total_accepted": accepted,
        "window_accepted": win_accepted,
        "steps_ok": steps_ok,
        "q_renewal": q,
        "g_surface": g,
        "localization": loc,
        "forward": wl.exchange_forward,
        "reverse": wl.exchange_reverse,
        "net": net,
        "turnover": turn,
        "last_reject": sim.last_reject_detail,
        "capacity_reject": sim.last_reject_detail.contains("CapacityExceeded"),
        "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("short_diagnostic.json"), &body)?;
    Ok(body)
}

/// Gate 4 — isolated biological renewal under invariant integrator.
pub fn run_gate4_isolated_turnover(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v8_params(SurfaceExchangeIntegrator::InvariantDomainV2)?;
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
    let mut last_windows = Vec::new();
    let mut steps_ok = true;

    for &horizon in ISOLATED_HORIZONS {
        let target = horizon;
        while total_accepted < target && steps_ok {
            if !sim.step() {
                steps_ok = false;
                if sim.last_reject_detail.contains("CapacityExceeded") {
                    capacity_rejects += 1;
                }
                break;
            }
            total_accepted += 1;
            if total_accepted % 1000 == 0 {
                eprintln!(
                    "D-031 Gate4 progress: accepted={} horizon_target={} reject={}",
                    total_accepted, target, sim.last_reject_detail
                );
            }
        }
        // Three measurement windows at this horizon.
        let window = 2_000u64;
        let mut windows = Vec::new();
        consecutive = 0;
        for _ in 0..3 {
            if !steps_ok {
                windows.push(json!({
                    "ok": false,
                    "accepted_in_window": 0,
                    "last_reject": sim.last_reject_detail,
                }));
                continue;
            }
            sim.surface_accounting
                .begin_window_local(sim.substep, sim.sim_time);
            let mut s_sum = 0.0;
            let mut n = 0u64;
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
                if sim.substep % 20 == 0 {
                    s_sum += chemistry_core::surface_density::total_surface_mass(
                        &sim.grid,
                        &sim.fields.membrane,
                    );
                    n += 1;
                }
            }
            let rates = WindowLocalSurfaceRates::from_sim(&sim);
            let mean_s = if n > 0 {
                s_sum / n as f64
            } else {
                chemistry_core::surface_density::total_surface_mass(
                    &sim.grid,
                    &sim.fields.membrane,
                )
            };
            let net = rates.adsorption;
            let turn = rates.gamma_turnover;
            let q = surface_balance_q(net, turn);
            let g = (net - turn) / mean_s.max(f64::EPSILON);
            let loc = gamma_localization(&sim);
            let wl = sim.surface_accounting.window_local();
            let ok = steps_ok
                && accepted >= window / 2
                && (0.98..=1.02).contains(&q)
                && g.abs() <= 1e-4
                && loc >= 0.98
                && wl.exchange_forward > 0.0
                && wl.exchange_reverse > 0.0
                && turn > 0.0
                && sim.fields.precursor.iter().all(|v| v.is_finite() && *v >= 0.0)
                && sim.fields.membrane.iter().all(|v| v.is_finite() && *v >= 0.0);
            if ok {
                consecutive += 1;
            } else {
                consecutive = 0;
            }
            windows.push(json!({
                "q_renewal": q,
                "g_surface": g,
                "localization": loc,
                "forward": wl.exchange_forward,
                "reverse": wl.exchange_reverse,
                "net": net,
                "turnover": turn,
                "accepted_in_window": accepted,
                "last_reject": sim.last_reject_detail,
                "ok": ok,
            }));
        }
        last_windows = windows.clone();
        let hr = json!({
            "horizon": horizon,
            "total_accepted": total_accepted,
            "steps_ok": steps_ok,
            "consecutive_ok": consecutive,
            "capacity_rejects": capacity_rejects,
            "windows": windows,
        });
        horizon_reports.push(hr.clone());
        // Checkpoint after each horizon so long runs leave inspectable evidence.
        let _ = atomic_write_json(
            &output.join(format!("horizon_{horizon}.json")),
            &hr,
        );
        eprintln!(
            "D-031 Gate4 horizon={} accepted={} consecutive_ok={} steps_ok={}",
            horizon, total_accepted, consecutive, steps_ok
        );
        if consecutive >= 3 {
            break;
        }
        if !steps_ok {
            break;
        }
    }

    let pass = consecutive >= 3 && total_accepted > 0 && capacity_rejects == 0;
    let conclusion = if !steps_ok && capacity_rejects > 0 {
        "D031_NUMERICAL_FAILURE"
    } else if pass {
        "D031_ISOLATED_RENEWAL_PASS"
    } else if steps_ok && total_accepted > 0 && consecutive < 3 {
        "D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED"
    } else {
        "D031_FAIL"
    };

    let body = json!({
        "project_directive": "D-031",
        "gate": 4,
        "pass": pass,
        "conclusion": conclusion,
        "total_accepted": total_accepted,
        "capacity_rejects": capacity_rejects,
        "consecutive_ok": consecutive,
        "horizons": horizon_reports,
        "last_windows": last_windows,
        "candidate": identified(),
        "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("isolated_turnover.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let root = resolve_path(output);
    fs::create_dir_all(&root)?;
    let g0p = run_gate0_preservation(&root.join("preservation"))?;
    let g0c = run_gate0_capacity_failure(&root.join("capacity_failure"))?;
    let class = g0c["capacity_failure"]["classification"]
        .as_str()
        .unwrap_or("D031_FAIL");
    if class == "D031_EXCHANGE_LAW_INVARIANT_FAILURE" {
        let body = json!({
            "conclusion": class,
            "gate0_preservation": g0p,
            "gate0_capacity": g0c,
            "stopped_at": "gate0",
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }
    let g3 = run_gate3_identification_regression(&root.join("identification_regression"))?;
    if g3["pass"] != true {
        let body = json!({
            "conclusion": "D031_EXCHANGE_IDENTIFICATION_REGRESSION",
            "gate0_preservation": g0p,
            "gate0_capacity": g0c,
            "gate3": g3,
            "stopped_at": "gate3",
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }
    let g4 = run_gate4_isolated_turnover(&root.join("isolated_turnover"))?;
    let conclusion = g4["conclusion"].as_str().unwrap_or("D031_FAIL");
    let body = json!({
        "project_directive": "D-031",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "conclusion": conclusion,
        "gate0_preservation": g0p["preservation"],
        "gate0_capacity_classification": class,
        "gate3": g3["pass"],
        "gate4": g4["pass"],
        "gate4_detail": g4,
        "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production": "REQUIRES_REMEDIATION",
    });
    atomic_write_json(&root.join("manifest.json"), &body)?;
    Ok(body)
}
