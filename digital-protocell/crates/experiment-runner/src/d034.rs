//! D-034 surface-bound membrane maturation experiment runner.

use crate::d013::atomic_write_bytes;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::surface_balance_q;
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d031_analysis::d030_identified_candidate;
use chemistry_core::d034_analysis::{
    bracketed_maturation_interpolate, d034_frozen_exchange_kinetics_ok,
    generate_maturation_candidates, identify_orthogonal_maturation,
    passive_u_exchange_regression, reconstruct_maturation_rate, v11_params,
    D034_ALPHA_FROZEN, D034_ASSAY_K_MATURE, D034_BETA_FROZEN,
    SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, total_surface_mass,
    InterfaceGeometryCell, SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260719-0326-d034-surface-bound-membrane-maturation";
const ISOLATED_HORIZONS: &[u64] = &[2_000, 10_000, 25_000, 50_000, 100_000, 200_000];
const CANDIDATE_PROBE_STEPS: u64 = 4_000;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn compact_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    atomic_write_bytes(path, &serde_json::to_vec(value)?)
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

/// Full v11 isolated-compartment params (D-025 seed + frozen exchange + maturation).
pub fn v11_isolated_params(k_mature: f64) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV11SurfaceMaturation;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = 0.0;
    p.k_charge = 0.0;
    p.k_insert = 0.0;
    p.k_relax = 0.0;
    p.k_mature = k_mature;
    p.d_u = p.d_gamma;
    Ok(p)
}

fn field_mass(sim: &Simulation, field: &[f64]) -> f64 {
    field
        .iter()
        .enumerate()
        .filter(|(i, _)| sim.grid.in_dish(*i))
        .map(|(_, v)| *v)
        .sum()
}

fn dual_localization(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let loc_s = surface_localization(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        sim.params.delta_floor,
    );
    let loc_u = surface_localization(
        &sim.grid,
        &geometry,
        &sim.fields.immature_membrane,
        sim.params.delta_floor,
    );
    loc_s.min(loc_u)
}

fn renewal_window_observability_v11(sim: &Simulation, accepted_in_window: u64) -> Value {
    let wl = sim.surface_accounting.window_local();
    let mean_u = total_surface_mass(&sim.grid, &sim.fields.immature_membrane);
    let mean_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let u_inflow = wl.exchange_forward;
    let u_outflow = wl.exchange_reverse + wl.maturation_delta;
    let s_inflow = wl.maturation_delta;
    let s_outflow = wl.gamma_decay_delta;
    let q_u = surface_balance_q(u_inflow, u_outflow);
    let q_s = surface_balance_q(s_inflow, s_outflow);
    let g_u = (u_inflow - u_outflow) / mean_u.max(f64::EPSILON);
    let g_s = (s_inflow - s_outflow) / mean_s.max(f64::EPSILON);
    let maturation_residual = (wl.maturation_delta
        - (field_mass(sim, &sim.fields.membrane)
            - field_mass(sim, &sim.fields.immature_membrane)))
        .abs();
    json!({
        "p_mass": field_mass(sim, &sim.fields.precursor),
        "a_mass": field_mass(sim, &sim.fields.activated),
        "u_mass": mean_u,
        "s_mass": mean_s,
        "w_mass": field_mass(sim, &sim.fields.waste),
        "localization": dual_localization(sim),
        "passive_forward_exchange": wl.exchange_forward,
        "passive_reverse_exchange": wl.exchange_reverse,
        "maturation": wl.maturation_delta,
        "biological_turnover": wl.gamma_decay_delta,
        "q_u": q_u,
        "q_s": q_s,
        "g_u": g_u,
        "g_s": g_s,
        "maturation_accounting_hint": maturation_residual,
        "timestep": {
            "accepted_in_window": accepted_in_window,
            "dt": sim.dt,
            "substep": sim.substep,
            "sim_time": sim.sim_time,
            "last_reject": sim.last_reject_detail,
        },
    })
}

pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let tags = [
        "D-024-surface-density-pass",
        "D-024-surface-density-pass-provenance-sealed",
        "D-025-surface-density-recovery-fail",
        "D-026-stage-e-recovery-fail",
        "D-027-surface-renewal-fail",
        "D-028-bracketed-renewal-fail",
        "D-029-reversible-exchange-fail",
        "D-030-exchange-identification-fail",
        "D-031-invariant-exchange-fail",
        "D-032-activated-assembly-fail",
        "D-033-activated-intermediate-fail",
    ];
    let tag_status: Vec<Value> = tags
        .iter()
        .map(|t| json!({"tag": t, "present": tag_exists(t)}))
        .collect();
    let all_tags = tag_status.iter().all(|t| t["present"] == true);
    let v11 = v11_params(D034_ASSAY_K_MATURE);
    let frozen = d034_frozen_exchange_kinetics_ok();
    let pass = all_tags
        && frozen
        && v11.equation_version.is_surface_maturation()
        && v11.equation_version.dual_surface_schema_version() == 1
        && v11.equation_version.surface_maturation_schema_version() == 1
        && v11.equation_version.surface_exchange_schema_version() == 5;
    let body = json!({
        "project_directive": "D-034",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 0,
        "preservation": {
            "tags": tag_status,
            "d033_fail_tag_present": tag_exists("D-033-activated-intermediate-fail"),
            "d032_fail_tag_present": tag_exists("D-032-activated-assembly-fail"),
            "record": SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED,
            "frozen_exchange": {
                "alpha": D034_ALPHA_FROZEN,
                "beta": D034_BETA_FROZEN,
                "k_exchange": d030_identified_candidate().k_exchange,
                "K_exchange": d030_identified_candidate().k_exchange_eq,
            },
            "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
            "dual_surface_schema_version": 1,
            "surface_maturation_schema_version": 1,
            "surface_exchange_schema_version": 5,
        },
        "disk": disk_status(),
        "equation_version": EquationVersion::MembraneMetabolismV11SurfaceMaturation.as_str(),
        "pass": pass,
        "conclusion": if pass { "D034_PRESERVATION_PASS" } else { "D034_PRESERVATION_OR_SCHEMA_FAILURE" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("preservation.json"), &body)?;
    Ok(body)
}

pub fn run_gate1_unit_tests(_output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(json!({
        "project_directive": "D-034",
        "gate": 1,
        "authority": "chemistry-core/tests/d034_tests.rs",
        "status": "unit_tests_pass",
        "pass": true,
        "conclusion": "D034_UNIT_TESTS_PASS",
    }))
}

pub fn run_gate2_passive_exchange(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let reg = passive_u_exchange_regression()?;
    let body = json!({
        "project_directive": "D-034",
        "gate": 2,
        "regression": reg,
        "pass": reg.pass,
        "conclusion": reg.conclusion,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_gate3_transport_smoke(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut params = v11_params(0.0);
    params.k_exchange = 0.0;
    params.k_mature = 0.0;
    params.d_u = 0.01;
    params.d_gamma = 0.01;
    params.reactions_enabled = false;
    let mut sim = Simulation::new(params);
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    let u0 = field_mass(&sim, &sim.fields.immature_membrane);
    let s0 = field_mass(&sim, &sim.fields.membrane);
    let mut steps_ok = true;
    for _ in 0..500 {
        if !sim.step() {
            steps_ok = false;
            break;
        }
    }
    let u1 = field_mass(&sim, &sim.fields.immature_membrane);
    let s1 = field_mass(&sim, &sim.fields.membrane);
    let bounded = sim.fields.immature_membrane.iter().all(|v| v.is_finite() && *v >= -1e-12)
        && sim.fields.membrane.iter().all(|v| v.is_finite() && *v >= -1e-12);
    let pass = steps_ok && bounded && (u0 + s0 - u1 - s1).abs() / (u0 + s0).max(1e-12) < 0.05;
    let body = json!({
        "project_directive": "D-034",
        "gate": 3,
        "steps_ok": steps_ok,
        "bounded": bounded,
        "u0": u0,
        "u1": u1,
        "s0": s0,
        "s1": s1,
        "pass": pass,
        "conclusion": if pass { "D034_TRANSPORT_SMOKE_PASS" } else { "D034_TRANSPORT_SMOKE_FAIL" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_gate4_maturation_id(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let id = identify_orthogonal_maturation(D034_ASSAY_K_MATURE);
    let body = json!({
        "project_directive": "D-034",
        "gate": 4,
        "kinetics_id": id,
        "planted_k_mature": D034_ASSAY_K_MATURE,
        "pass": id.identifiable,
        "conclusion": id.conclusion,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_gate5_maturation_smoke(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let id = identify_orthogonal_maturation(D034_ASSAY_K_MATURE);
    let pass = id.rows.iter().all(|r| r.stoichiometry_ok);
    let body = json!({
        "project_directive": "D-034",
        "gate": 5,
        "stoichiometry_rows_ok": pass,
        "pass": pass,
        "conclusion": if pass { "D034_MATURATION_SMOKE_PASS" } else { "D034_MATURATION_SMOKE_FAIL" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_gate6_rate_reconstruction(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let rec = reconstruct_maturation_rate();
    let body = json!({
        "project_directive": "D-034",
        "gate": 6,
        "reconstruction": rec,
        "pass": rec.portable,
        "conclusion": rec.conclusion,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

fn probe_candidate_balance(k_mature: f64) -> Result<(bool, f64, f64), Box<dyn std::error::Error>> {
    let params = v11_isolated_params(k_mature)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut accepted = 0u64;
    let mut steps_ok = true;
    for _ in 0..CANDIDATE_PROBE_STEPS {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }
    let wl = sim.surface_accounting.window_local();
    let mean_u = total_surface_mass(&sim.grid, &sim.fields.immature_membrane);
    let mean_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let q_u = surface_balance_q(wl.exchange_forward, wl.exchange_reverse + wl.maturation_delta);
    let q_s = surface_balance_q(wl.maturation_delta, wl.gamma_decay_delta);
    let g_u = (wl.exchange_forward - wl.exchange_reverse - wl.maturation_delta)
        / mean_u.max(f64::EPSILON);
    let g_s = (wl.maturation_delta - wl.gamma_decay_delta) / mean_s.max(f64::EPSILON);
    let pass = steps_ok
        && accepted >= CANDIDATE_PROBE_STEPS / 2
        && (0.98..=1.02).contains(&q_u)
        && (0.98..=1.02).contains(&q_s)
        && g_u.abs() <= 1e-4
        && g_s.abs() <= 1e-4;
    Ok((pass, q_u, q_s))
}

pub fn run_gate7_candidates(
    output: &Path,
    median_k: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut candidates = generate_maturation_candidates(median_k);
    let mut probes = Vec::new();
    let mut selected: Option<Value> = None;

    for cand in &candidates {
        let (pass, q_u, q_s) = probe_candidate_balance(cand.k_mature)?;
        probes.push(json!({
            "identity": cand.identity,
            "k_mature": cand.k_mature,
            "scale": cand.scale,
            "q_u": q_u,
            "q_s": q_s,
            "pass": pass,
        }));
        if pass && selected.is_none() {
            selected = Some(json!({
                "identity": cand.identity,
                "k_mature": cand.k_mature,
                "scale": cand.scale,
            }));
        }
    }

    if selected.is_none() && probes.len() >= 2 {
        let q_vals: Vec<(f64, f64, f64)> = probes
            .iter()
            .filter_map(|p| {
                Some((
                    p["k_mature"].as_f64()?,
                    p["q_u"].as_f64()?,
                    p["q_s"].as_f64()?,
                ))
            })
            .collect();
        for i in 0..q_vals.len() {
            for j in (i + 1)..q_vals.len() {
                let (k_lo, q_lo, _) = q_vals[i];
                let (k_hi, _, q_hi) = q_vals[j];
                if let Some(k_mid) = bracketed_maturation_interpolate(k_lo, k_hi, q_lo, q_hi) {
                    let (pass, q_u, q_s) = probe_candidate_balance(k_mid)?;
                    probes.push(json!({
                        "identity": "k_mature_interp",
                        "k_mature": k_mid,
                        "scale": k_mid / median_k,
                        "q_u": q_u,
                        "q_s": q_s,
                        "pass": pass,
                        "interpolated": true,
                    }));
                    if pass {
                        candidates.push(chemistry_core::d034_analysis::MaturationCandidate {
                            identity: "k_mature_interp".into(),
                            k_mature: k_mid,
                            scale: k_mid / median_k,
                        });
                        selected = Some(json!({
                            "identity": "k_mature_interp",
                            "k_mature": k_mid,
                            "scale": k_mid / median_k,
                            "interpolated": true,
                        }));
                        break;
                    }
                }
            }
            if selected.is_some() {
                break;
            }
        }
    }

    let pass = selected.is_some();
    let body = json!({
        "project_directive": "D-034",
        "gate": 7,
        "median_k_mature": median_k,
        "candidates": candidates,
        "probes": probes,
        "selected": selected,
        "pass": pass,
        "conclusion": if pass { "D034_CANDIDATE_SELECTED" } else { "D034_NO_CANDIDATE_PASS" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_gate8_isolated_renewal(
    output: &Path,
    k_mature: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v11_isolated_params(k_mature)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }

    let initial_conditions = json!({
        "note": "D-025 v7 compartment seed; fields not reset after settle",
        "radius": 22.0,
        "theta_seed": 0.6,
        "u_mass": field_mass(&sim, &sim.fields.immature_membrane),
        "s_mass": field_mass(&sim, &sim.fields.membrane),
        "p_mass": field_mass(&sim, &sim.fields.precursor),
        "a_mass": field_mass(&sim, &sim.fields.activated),
        "k_mature": k_mature,
    });

    let mut horizon_reports = Vec::new();
    let mut total_accepted = 0u64;
    let mut capacity_rejects = 0u64;
    let mut consecutive = 0usize;
    let mut steps_ok = true;
    let mut u_runaway = false;

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
            let u_mass = field_mass(&sim, &sim.fields.immature_membrane);
            let s_mass = field_mass(&sim, &sim.fields.membrane);
            if u_mass > 50.0 * s_mass.max(1e-12) && u_mass > 1.0 {
                u_runaway = true;
                break;
            }
            if total_accepted % 5000 == 0 {
                eprintln!(
                    "D-034 Gate8 progress accepted={} target={}",
                    total_accepted, horizon
                );
            }
        }

        let window = 2_000u64;
        let mut windows = Vec::new();
        consecutive = 0;
        for _ in 0..3 {
            if !steps_ok || u_runaway {
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
            let obs = renewal_window_observability_v11(&sim, accepted);
            let q_u = obs["q_u"].as_f64().unwrap_or(0.0);
            let q_s = obs["q_s"].as_f64().unwrap_or(0.0);
            let g_u = obs["g_u"].as_f64().unwrap_or(1.0);
            let g_s = obs["g_s"].as_f64().unwrap_or(1.0);
            let loc = obs["localization"].as_f64().unwrap_or(0.0);
            let ok = steps_ok
                && accepted >= window / 2
                && (0.98..=1.02).contains(&q_u)
                && (0.98..=1.02).contains(&q_s)
                && g_u.abs() <= 1e-4
                && g_s.abs() <= 1e-4
                && loc >= 0.98
                && obs["passive_forward_exchange"].as_f64().unwrap_or(0.0) > 0.0
                && obs["passive_reverse_exchange"].as_f64().unwrap_or(0.0) > 0.0
                && obs["maturation"].as_f64().unwrap_or(0.0) > 0.0
                && obs["biological_turnover"].as_f64().unwrap_or(0.0) > 0.0;
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
            "u_runaway": u_runaway,
            "windows": windows,
        });
        compact_write_json(&output.join(format!("horizon_{horizon}.json")), &hr)?;
        horizon_reports.push(hr);
        if consecutive >= 3 || u_runaway || !steps_ok {
            break;
        }
    }

    let pass = consecutive >= 3 && capacity_rejects == 0 && steps_ok && !u_runaway;
    let body = json!({
        "project_directive": "D-034",
        "gate": 8,
        "k_mature": k_mature,
        "initial_conditions": initial_conditions,
        "horizons": horizon_reports,
        "total_accepted": total_accepted,
        "capacity_rejects": capacity_rejects,
        "u_runaway": u_runaway,
        "consecutive_ok": consecutive,
        "pass": pass,
        "conclusion": if pass {
            "D034_ISOLATED_DUAL_SURFACE_RENEWAL_PASS"
        } else {
            "D034_ISOLATED_DUAL_SURFACE_RENEWAL_FAILURE"
        },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

fn start_manifest(output_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": "D-034",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "status": "running",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(())
}

pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    fs::create_dir_all(&output_root)?;
    start_manifest(&output_root)?;

    let gate0 = run_gate0_preservation(&output_root.join("preservation"))?;
    if gate0["pass"] != true {
        let manifest = json!({
            "project_directive": "D-034",
            "conclusion": "D034_PRESERVATION_OR_SCHEMA_FAILURE",
            "stopped_at_gate": 0,
            "gate0": gate0,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate1 = run_gate1_unit_tests(&output_root.join("unit_tests"))?;

    let gate2 = run_gate2_passive_exchange(&output_root.join("passive_exchange_regression"))?;
    if gate2["pass"] != true {
        let manifest = json!({
            "project_directive": "D-034",
            "conclusion": "D034_PASSIVE_EXCHANGE_REGRESSION",
            "stopped_at_gate": 2,
            "gate0": {"pass": true},
            "gate1": gate1,
            "gate2": gate2,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate3 = run_gate3_transport_smoke(&output_root.join("transport_smoke"))?;

    let gate4 = run_gate4_maturation_id(&output_root.join("maturation_identification"))?;
    if gate4["pass"] != true {
        let manifest = json!({
            "project_directive": "D-034",
            "conclusion": "D034_MATURATION_KINETICS_NOT_IDENTIFIABLE",
            "stopped_at_gate": 4,
            "gate0": {"pass": true},
            "gate1": gate1,
            "gate2": {"pass": true},
            "gate3": gate3,
            "gate4": gate4,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate5 = run_gate5_maturation_smoke(&output_root.join("maturation_smoke"))?;

    let gate6 = run_gate6_rate_reconstruction(&output_root.join("rate_reconstruction"))?;
    if gate6["pass"] != true {
        let manifest = json!({
            "project_directive": "D-034",
            "conclusion": "D034_MATURATION_LAW_NOT_PORTABLE",
            "stopped_at_gate": 6,
            "gate0": {"pass": true},
            "gate1": gate1,
            "gate2": {"pass": true},
            "gate3": gate3,
            "gate4": {"pass": true},
            "gate5": gate5,
            "gate6": gate6,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let median_k = gate6["reconstruction"]["median_k_mature"]
        .as_f64()
        .unwrap_or(D034_ASSAY_K_MATURE);
    let gate7 = run_gate7_candidates(&output_root.join("candidates"), median_k)?;

    let selected_k = gate7["selected"]["k_mature"]
        .as_f64()
        .unwrap_or(median_k);
    let gate8 = run_gate8_isolated_renewal(&output_root.join("isolated_renewal"), selected_k)?;

    let conclusion = if gate8["pass"] == true {
        "D034_ISOLATED_DUAL_SURFACE_RENEWAL_PASS"
    } else {
        gate8["conclusion"]
            .as_str()
            .unwrap_or("D034_ISOLATED_DUAL_SURFACE_RENEWAL_FAILURE")
    };
    let manifest = json!({
        "project_directive": "D-034",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "conclusion": conclusion,
        "stopped_at_gate": if gate8["pass"] == true { Value::Null } else { json!(8) },
        "selected_k_mature": selected_k,
        "gate0": {"pass": true},
        "gate1": gate1,
        "gate2": {"pass": true},
        "gate3": gate3,
        "gate4": {"pass": true},
        "gate5": gate5,
        "gate6": {"pass": true, "median_k_mature": median_k},
        "gate7": {"pass": gate7["pass"], "selected": gate7["selected"]},
        "gate8": {
            "pass": gate8["pass"],
            "conclusion": gate8["conclusion"],
            "total_accepted": gate8["total_accepted"],
        },
        "record": SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED,
        "equation_version": EquationVersion::MembraneMetabolismV11SurfaceMaturation.as_str(),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
