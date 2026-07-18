//! D-029 reversible surface-exchange runner (Gates 0–6 core path).

use crate::d013::{
    atomic_write_json, load_governed_checkpoint, restore_governed_simulation,
};
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::EquationVersion;
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::WindowLocalSurfaceRates;
use chemistry_core::d029_analysis::{
    apply_exchange_candidate, candidate_log_distance, compute_exchange_basis_labeled,
    fit_exchange_nnls, generate_exchange_candidates, leave_one_out_stable, ExchangeCandidate,
    ExchangeFitResult,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260717-d029-reversible-surface-exchange";
const ISOLATED_STEPS: u64 = 12_000;
const PORTABILITY_MEASURE: u64 = 2_000;

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

fn six_states() -> Result<Vec<(String, Simulation)>, Box<dyn std::error::Error>> {
    Ok(vec![
        (
            "d024_fixed_interface_r22".into(),
            fixed_interface_r22_state()?,
        ),
        (
            "d025_dynamic_r22_endpoint".into(),
            dynamic_r22_endpoint_state()?,
        ),
        (
            "d026_stage_e_10000".into(),
            restore_stage_e_checkpoint(10_000)?,
        ),
        (
            "d026_stage_e_25000".into(),
            restore_stage_e_checkpoint(25_000)?,
        ),
        (
            "d026_stage_e_100000".into(),
            restore_stage_e_checkpoint(100_000)?,
        ),
        (
            "d026_stage_e_200000".into(),
            restore_stage_e_checkpoint(200_000)?,
        ),
    ])
}

/// Gate 2: two-parameter identification from six governed states.
pub fn run_gate2_identification(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let states = six_states()?;
    let mut rows = Vec::new();
    for (label, sim) in &states {
        let row = compute_exchange_basis_labeled(sim, label);
        atomic_write_json(&output.join(format!("{label}.json")), &json!(row))?;
        rows.push(row);
    }
    let fit = fit_exchange_nnls(&rows);
    let (loo_ok, loo_vals) = leave_one_out_stable(&rows, &fit);
    let candidates = if fit.identifiable && loo_ok {
        generate_exchange_candidates(&fit)
    } else {
        Vec::new()
    };
    let pass = fit.identifiable && loo_ok && fit.rank == 2;
    let conclusion = if pass {
        "D029_EXCHANGE_IDENTIFIABLE".to_string()
    } else if !fit.identifiable {
        fit.conclusion.clone()
    } else {
        "D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE".to_string()
    };
    let body = json!({
        "project_directive": "D-029",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 2,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange.as_str(),
        "surface_exchange_schema_version": 2,
        "matrix": rows,
        "fit": fit,
        "leave_one_out_ok": loo_ok,
        "leave_one_out": loo_vals.iter().map(|(k, keq)| json!({"k_exchange": k, "K_exchange": keq})).collect::<Vec<_>>(),
        "candidates": candidates,
        "pass": pass,
        "conclusion": conclusion,
        "record": "IRREVERSIBLE_ADSORPTION_LAW_REJECTED",
    });
    atomic_write_json(&output.join("parameter_identification.json"), &body)?;
    Ok(body)
}

fn v8_from_candidate(c: &ExchangeCandidate) -> Result<chemistry_core::SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, c);
    // Preserve frozen biological Γ turnover and transport from v7 base.
    Ok(p)
}

/// Isolated fixed-interface surface balance under reversible exchange.
pub fn run_isolated_renewal(
    output: &Path,
    c: &ExchangeCandidate,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v8_from_candidate(c)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    let burn = ISOLATED_STEPS * 2 / 3;
    let measure = ISOLATED_STEPS - burn;
    for _ in 0..burn {
        if !sim.step() {
            break;
        }
    }
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut windows_ok = 0usize;
    let mut last_q = 0.0;
    let mut last_g = 0.0;
    let window = measure / 3;
    for w in 0..3 {
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        let mut s_sum = 0.0;
        let mut n = 0u64;
        for _ in 0..window.max(200) {
            if !sim.step() {
                break;
            }
            if sim.substep % 20 == 0 {
                s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
                n += 1;
            }
        }
        let rates = WindowLocalSurfaceRates::from_sim(&sim);
        let mean_s = if n > 0 {
            s_sum / n as f64
        } else {
            total_surface_mass(&sim.grid, &sim.fields.membrane)
        };
        let net = rates.adsorption; // net exchange into S (v8 ledger)
        let turn = rates.gamma_turnover;
        let q = net / turn.max(f64::EPSILON);
        let g = (net - turn) / mean_s.max(f64::EPSILON);
        last_q = q;
        last_g = g;
        let loc = gamma_localization(&sim);
        let fwd = sim.surface_accounting.window_local().exchange_forward;
        let rev = sim.surface_accounting.window_local().exchange_reverse;
        let ok = (0.98..=1.02).contains(&q)
            && g.abs() <= 1e-4
            && loc >= 0.98
            && turn > 0.0
            && fwd > 0.0;
        if ok {
            windows_ok += 1;
        }
        atomic_write_json(
            &output.join(format!("window_{w}.json")),
            &json!({
                "Q_renewal": q,
                "g_surface": g,
                "localization": loc,
                "forward": fwd,
                "reverse": rev,
                "net": net,
                "turnover": turn,
                "pass": ok,
            }),
        )?;
    }
    let pass = windows_ok >= 3;
    let body = json!({
        "project_directive": "D-029",
        "gate": 5,
        "candidate": c,
        "windows_ok": windows_ok,
        "Q_renewal": last_q,
        "g_surface": last_g,
        "pass": pass,
        "conclusion": if pass { "D029_ISOLATED_RENEWAL_PASS" } else { "D029_ISOLATED_RENEWAL_FAILURE" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("isolated_renewal.json"), &body)?;
    Ok(body)
}

/// Six-state portability of selected candidate.
pub fn run_gate6_portability(
    output: &Path,
    c: &ExchangeCandidate,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let states = six_states()?;
    let mut results = Vec::new();
    let mut pass_count = 0usize;
    for (label, mut base) in states {
        // Transplant fields into a fresh v8 candidate simulation.
        let mut params = v8_from_candidate(c)?;
        params.d008_stage_b_enabled = base.params.d008_stage_b_enabled;
        let constrained = base.enforce_structure_constraint;
        let mut sim = Simulation::new(params);
        sim.fields = base.fields.clone();
        sim.enforce_structure_constraint = constrained;
        sim.dt_cap = 0.005;
        for _ in 0..D026_SETTLE_STEPS.min(500) {
            if !sim.step() {
                break;
            }
        }
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        let mut s_sum = 0.0;
        let mut n = 0u64;
        for _ in 0..PORTABILITY_MEASURE {
            if !sim.step() {
                break;
            }
            if sim.substep % 20 == 0 {
                s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
                n += 1;
            }
        }
        let rates = WindowLocalSurfaceRates::from_sim(&sim);
        let mean_s = if n > 0 {
            s_sum / n as f64
        } else {
            total_surface_mass(&sim.grid, &sim.fields.membrane)
        };
        let net = rates.adsorption;
        let turn = rates.gamma_turnover;
        let q = net / turn.max(f64::EPSILON);
        let g = (net - turn) / mean_s.max(f64::EPSILON);
        let p_ok = sim.fields.precursor.iter().all(|v| v.is_finite() && *v >= 0.0);
        let s_ok = sim.fields.membrane.iter().all(|v| v.is_finite() && *v >= 0.0);
        let pass = p_ok
            && s_ok
            && (0.90..=1.10).contains(&q)
            && turn > 0.0
            && g.is_finite();
        if pass {
            pass_count += 1;
        }
        results.push(json!({
            "label": label,
            "Q_renewal": q,
            "g_surface": g,
            "pass": pass,
            "forward": sim.surface_accounting.window_local().exchange_forward,
            "reverse": sim.surface_accounting.window_local().exchange_reverse,
            "net": net,
            "turnover": turn,
        }));
    }
    let pass = pass_count >= 5;
    let body = json!({
        "project_directive": "D-029",
        "gate": 6,
        "candidate": c,
        "pass_count": pass_count,
        "required": "5/6",
        "results": results,
        "pass": pass,
        "conclusion": if pass { "D029_PORTABILITY_PASS" } else { "D029_REVERSIBLE_EXCHANGE_NOT_PORTABLE" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("portability.json"), &body)?;
    Ok(body)
}

/// Full pipeline through Gate 6 (stop on first fail). Later gates deferred.
pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let root = resolve_path(output);
    fs::create_dir_all(&root)?;

    // Gate 0 already written to preservation/; reaffirm.
    let preservation = root.join("preservation/preservation_manifest.json");
    if !preservation.is_file() {
        return Ok(json!({
            "conclusion": "D029_FAIL",
            "reason": "missing Gate 0 preservation manifest",
            "pass": false,
        }));
    }

    let id = run_gate2_identification(&root.join("parameter_identification"))?;
    if id["pass"] != true {
        let conclusion = id["conclusion"].as_str().unwrap_or("D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE");
        let body = json!({
            "project_directive": "D-029",
            "conclusion": conclusion,
            "stopped_at": "gate2",
            "identification": id,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let fit: ExchangeFitResult = serde_json::from_value(id["fit"].clone())?;
    let candidates: Vec<ExchangeCandidate> = serde_json::from_value(id["candidates"].clone())?;
    let center = candidates
        .iter()
        .find(|c| c.identity == "fitted_center")
        .cloned()
        .ok_or("missing fitted_center")?;

    // Gate 5 first on center (Gate 4 static unit-covered; Gate 3 candidate set recorded).
    let isolated = run_isolated_renewal(&root.join("isolated_renewal"), &center)?;
    let mut selected = center.clone();
    if isolated["pass"] != true {
        // Try perturbations only when center misses and direction is supported.
        let mut best: Option<(ExchangeCandidate, Value, f64)> = None;
        for c in &candidates {
            if c.identity == "fitted_center" {
                continue;
            }
            let r = run_isolated_renewal(
                &root.join(format!("isolated_renewal_{}", c.identity)),
                c,
            )?;
            if r["pass"] == true {
                let dist = candidate_log_distance(c, &fit);
                if best
                    .as_ref()
                    .map(|(_, _, d)| dist < *d)
                    .unwrap_or(true)
                {
                    best = Some((c.clone(), r, dist));
                }
            }
        }
        if let Some((c, r, _)) = best {
            selected = c;
            atomic_write_json(&root.join("isolated_renewal/selected.json"), &r)?;
        } else {
            let body = json!({
                "project_directive": "D-029",
                "conclusion": "D029_ISOLATED_RENEWAL_FAILURE",
                "stopped_at": "gate5",
                "identification": id,
                "isolated": isolated,
                "pass": false,
                "source_commit": git_commit_hash(),
            });
            atomic_write_json(&root.join("manifest.json"), &body)?;
            return Ok(body);
        }
    }

    let port = run_gate6_portability(&root.join("portability"), &selected)?;
    if port["pass"] != true {
        let body = json!({
            "project_directive": "D-029",
            "conclusion": "D029_REVERSIBLE_EXCHANGE_NOT_PORTABLE",
            "stopped_at": "gate6",
            "identification": id,
            "isolated": isolated,
            "portability": port,
            "selected_candidate": selected,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let body = json!({
        "project_directive": "D-029",
        "conclusion": "D029_GATES_0_6_PASS_CONTINUE",
        "stopped_at": null,
        "identification": id,
        "isolated": isolated,
        "portability": port,
        "selected_candidate": selected,
        "pass": true,
        "note": "Gates 7–15 not yet executed in this pipeline revision",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&root.join("manifest.json"), &body)?;
    Ok(body)
}
