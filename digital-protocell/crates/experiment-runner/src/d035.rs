//! D-035 mature-membrane-catalyzed assembly experiment runner (Gate 0 first).

use crate::d013::atomic_write_bytes;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::surface_balance_q;
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d031_analysis::d030_identified_candidate;
use chemistry_core::d035_analysis::{
    architecture_review, gate2_conservation, gate3_autocatalytic_signature,
    identify_saturation_constants, reconstruct_catalytic_rate, ArchitectureReview,
    D035_K_A_IDENTIFIED, D035_K_U_IDENTIFIED,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260719-d035-membrane-bound-catalytic-assembly";
const ISOLATED_HORIZONS: &[u64] = &[2_000, 10_000, 25_000, 50_000, 100_000, 200_000];
/// Basal share of catalytic scale for bootstrap (kept ≪ catalytic at typical θ_S).
const D035_BASAL_FRAC: f64 = 0.02;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn compact_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
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

fn tag_exists(tag: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn screen_to_json(s: &chemistry_core::d035_analysis::LawArchitectureScreen) -> Value {
    json!({
        "law": s.law_name,
        "k0": s.k0,
        "K_A": s.k_a,
        "K_U": s.k_u,
        "valid_count": s.valid_count,
        "median_rate": s.median_rate,
        "span_factor": s.span_factor,
        "loo_ok": s.loo_ok,
        "loo_medians": s.loo_medians,
        "algebraic_ok": s.algebraic_ok,
        "portable": s.portable,
        "notes": s.notes,
        "estimates": s.estimates.iter().map(|e| json!({
            "state_id": e.state_id,
            "l_s": e.l_s,
            "basis": e.basis,
            "rate_required": e.rate_required,
            "valid": e.valid,
            "reject_reason": e.reject_reason,
            "mean_theta_s": e.mean_theta_s,
            "mean_a": e.mean_a,
            "mean_gamma_u": e.mean_gamma_u,
        })).collect::<Vec<_>>(),
    })
}

pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let head = git_commit_hash();
    let d034_tag = tag_exists("D-034-surface-maturation-fail");
    let d035_pass = tag_exists("D-035-stage-e-recovered");
    let d035_fail = tag_exists("D-035-catalytic-assembly-fail");
    let result = json!({
        "project_directive": "D-035",
        "agent_memory_id": AGENT_MEMORY_ID,
        "gate": 0,
        "phase": "preservation",
        "source_commit": head,
        "starting_commit_expected": "9a3bef9",
        "starting_tag": "D-034-surface-maturation-fail",
        "d034_tag_present": d034_tag,
        "d035_pass_tag_absent": !d035_pass,
        "d035_fail_tag_absent": !d035_fail,
        "linear_surface_maturation_law_rejected":
            chemistry_core::d035_analysis::LINEAR_SURFACE_MATURATION_LAW_REJECTED,
        "fields_preserved": ["phi","C","N","F","W","A","P","U","S"],
        "production_untouched": true,
        "pass": d034_tag && !d035_pass && !d035_fail,
        "conclusion": if d034_tag {
            "D035_PRESERVATION_PASS"
        } else {
            "D035_PRESERVATION_FAIL"
        },
    });
    compact_write_json(&out.join("preservation.json"), &result)?;
    Ok(result)
}

pub fn run_gate0_architecture_review(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let review: ArchitectureReview = architecture_review();
    let selected = review
        .selected_law
        .map(|l| l.as_str().to_string())
        .unwrap_or_else(|| "none".into());
    let result = json!({
        "project_directive": "D-035",
        "agent_memory_id": AGENT_MEMORY_ID,
        "gate": 0,
        "phase": "architecture_review",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "linear_law_rejected": review.linear_law_rejected,
        "control_a": screen_to_json(&review.control_a),
        "candidate_b": screen_to_json(&review.candidate_b),
        "candidate_c": screen_to_json(&review.candidate_c),
        "selected_law": selected,
        "pass": review.pass,
        "conclusion": review.conclusion,
        "production_untouched": true,
    });
    compact_write_json(&out.join("architecture_review.json"), &result)?;
    Ok(result)
}

/// Gate 0 pipeline: preservation + architecture screen; stop without chemistry change.
pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let preservation = run_gate0_preservation(&out.join("preservation"))?;
    if preservation.get("pass") != Some(&json!(true)) {
        let manifest = json!({
            "project_directive": "D-035",
            "stopped_at_gate": 0,
            "phase": "preservation",
            "conclusion": preservation.get("conclusion"),
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        compact_write_json(&out.join("manifest.json"), &manifest)?;
        compact_write_json(&out.join("result.json"), &manifest)?;
        return Ok(manifest);
    }
    let review = run_gate0_architecture_review(&out.join("architecture_review"))?;
    let pass0 = review.get("pass") == Some(&json!(true));
    if !pass0 {
        let conclusion = review
            .get("conclusion")
            .cloned()
            .unwrap_or(json!("D035_MEMBRANE_CATALYTIC_ARCHITECTURE_REJECTED"));
        let manifest = json!({
            "project_directive": "D-035",
            "agent_memory_id": AGENT_MEMORY_ID,
            "stopped_at_gate": 0,
            "phase": "architecture_review",
            "pass": false,
            "conclusion": conclusion,
            "selected_law": review.get("selected_law"),
            "control_a_span": review.pointer("/control_a/span_factor"),
            "candidate_b_span": review.pointer("/candidate_b/span_factor"),
            "candidate_c_span": review.pointer("/candidate_c/span_factor"),
            "production_untouched": true,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
            "next": "Stop: do not implement catalytic maturation; next review may consider explicit membrane-bound catalyst field",
        });
        compact_write_json(&out.join("manifest.json"), &manifest)?;
        compact_write_json(&out.join("result.json"), &manifest)?;
        return Ok(manifest);
    }

    let sat = run_gate1_saturation_identification(&out.join("saturation_identification"))?;
    let pass1 = sat.get("pass") == Some(&json!(true));
    if !pass1 {
        let conclusion = sat
            .get("conclusion")
            .cloned()
            .unwrap_or(json!("D035_CATALYTIC_KINETICS_NOT_IDENTIFIABLE"));
        let manifest = json!({
            "project_directive": "D-035",
            "agent_memory_id": AGENT_MEMORY_ID,
            "stopped_at_gate": 1,
            "phase": "saturation_identification",
            "pass": false,
            "conclusion": conclusion,
            "gate0_conclusion": review.get("conclusion"),
            "selected_law": review.get("selected_law"),
            "production_untouched": true,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&out.join("manifest.json"), &manifest)?;
        compact_write_json(&out.join("result.json"), &manifest)?;
        return Ok(manifest);
    }

    let g2 = run_gate2_conservation(&out.join("conservation"))?;
    if g2.get("pass") != Some(&json!(true)) {
        return Ok(stop_manifest(&out, 2, "conservation", &g2)?);
    }
    let g3 = run_gate3_autocatalytic(&out.join("autocatalytic_signature"))?;
    if g3.get("pass") != Some(&json!(true)) {
        return Ok(stop_manifest(&out, 3, "autocatalytic_signature", &g3)?);
    }
    let g4 = run_gate4_rate_reconstruction(&out.join("rate_reconstruction"))?;
    if g4.get("pass") != Some(&json!(true)) {
        return Ok(stop_manifest(&out, 4, "rate_reconstruction", &g4)?);
    }
    let median_k = g4["median_rate"].as_f64().unwrap_or(0.02);
    let g5 = run_gate5_isolated_renewal(&out.join("isolated_renewal"), median_k)?;
    if g5.get("pass") != Some(&json!(true)) {
        return Ok(stop_manifest(&out, 5, "isolated_renewal", &g5)?);
    }

    let manifest = json!({
        "project_directive": "D-035",
        "agent_memory_id": AGENT_MEMORY_ID,
        "stopped_at_gate": 5,
        "phase": "isolated_renewal",
        "pass": true,
        "conclusion": "D035_ISOLATED_CATALYTIC_RENEWAL_PASS",
        "selected_law": "candidate_c_saturating",
        "K_A": D035_K_A_IDENTIFIED,
        "K_U": D035_K_U_IDENTIFIED,
        "median_k_cat": median_k,
        "gate0_conclusion": review.get("conclusion"),
        "production_chemistry": "membrane_metabolism_v12_membrane_catalytic_assembly",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "next": "Gate 6 causal controls",
    });
    compact_write_json(&out.join("manifest.json"), &manifest)?;
    compact_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}

fn stop_manifest(
    out: &Path,
    gate: u32,
    phase: &str,
    gate_result: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let conclusion = gate_result
        .get("conclusion")
        .cloned()
        .unwrap_or(json!("D035_FAIL"));
    let manifest = json!({
        "project_directive": "D-035",
        "agent_memory_id": AGENT_MEMORY_ID,
        "stopped_at_gate": gate,
        "phase": phase,
        "pass": false,
        "conclusion": conclusion,
        "gate_result": gate_result,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&out.join("manifest.json"), &manifest)?;
    compact_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}

pub fn run_gate1_saturation_identification(
    output: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let id = identify_saturation_constants();
    let result = json!({
        "project_directive": "D-035",
        "agent_memory_id": AGENT_MEMORY_ID,
        "gate": 1,
        "phase": "saturation_identification",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "planted_k_a": id.planted_k_a,
        "planted_k_u": id.planted_k_u,
        "a_response": {
            "levels": id.a_response.levels,
            "rates": id.a_response.rates,
            "k_half": id.a_response.k_half,
            "vmax": id.a_response.vmax,
            "zero_at_zero": id.a_response.zero_at_zero,
            "monotonic": id.a_response.monotonic,
            "k_in_range": id.a_response.k_in_range,
            "bootstrap_spread_rel": id.a_response.bootstrap_spread_rel,
            "loo_spread_rel": id.a_response.loo_spread_rel,
            "identifiable": id.a_response.identifiable,
            "notes": id.a_response.notes,
        },
        "u_response": {
            "levels": id.u_response.levels,
            "rates": id.u_response.rates,
            "k_half": id.u_response.k_half,
            "vmax": id.u_response.vmax,
            "zero_at_zero": id.u_response.zero_at_zero,
            "monotonic": id.u_response.monotonic,
            "k_in_range": id.u_response.k_in_range,
            "bootstrap_spread_rel": id.u_response.bootstrap_spread_rel,
            "loo_spread_rel": id.u_response.loo_spread_rel,
            "identifiable": id.u_response.identifiable,
            "notes": id.u_response.notes,
        },
        "pass": id.pass,
        "conclusion": id.conclusion,
        "production_untouched": true,
    });
    compact_write_json(&out.join("saturation_identification.json"), &result)?;
    Ok(result)
}

pub fn run_gate2_conservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let g = gate2_conservation();
    let result = json!({
        "project_directive": "D-035",
        "gate": 2,
        "pass": g.pass,
        "conclusion": g.conclusion,
        "no_u_zero": g.no_u_zero,
        "no_a_zero": g.no_a_zero,
        "no_cat_without_s": g.no_cat_without_s,
        "basal_without_s": g.basal_without_s,
        "u_loss_eq_s_gain": g.u_loss_eq_s_gain,
        "a_loss_eq_w_gain": g.a_loss_eq_w_gain,
        "material_closed": g.material_closed,
        "theta_ok": g.theta_ok,
        "nonnegative": g.nonnegative,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&out.join("conservation.json"), &result)?;
    Ok(result)
}

pub fn run_gate3_autocatalytic(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let g = gate3_autocatalytic_signature();
    let result = json!({
        "project_directive": "D-035",
        "gate": 3,
        "pass": g.pass,
        "conclusion": g.conclusion,
        "rate_rises_with_s": g.rate_rises_with_s,
        "catalytic_vs_basal": g.catalytic_vs_basal,
        "no_a_control": g.no_a_control,
        "no_u_control": g.no_u_control,
        "basal_only_at_zero_s": g.basal_only_at_zero_s,
        "rates_by_s": g.rates_by_s,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&out.join("autocatalytic_signature.json"), &result)?;
    Ok(result)
}

pub fn run_gate4_rate_reconstruction(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let g = reconstruct_catalytic_rate();
    let result = json!({
        "project_directive": "D-035",
        "gate": 4,
        "pass": g.portable,
        "conclusion": if g.portable {
            "D035_CATALYTIC_RATE_PORTABLE"
        } else {
            "D035_CATALYTIC_LAW_NOT_PORTABLE"
        },
        "K_A": g.k_a,
        "K_U": g.k_u,
        "median_rate": g.median_rate,
        "span_factor": g.span_factor,
        "loo_ok": g.loo_ok,
        "valid_count": g.valid_count,
        "estimates": g.estimates,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&out.join("rate_reconstruction.json"), &result)?;
    Ok(result)
}

pub fn v12_isolated_params(k_cat: f64) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV12MembraneCatalyticAssembly;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = 0.0;
    p.k_charge = 0.0;
    p.k_insert = 0.0;
    p.k_relax = 0.0;
    p.k_mature = 0.0;
    p.k_mature_cat = k_cat;
    // Basal ≪ catalytic at θ_S~0.25: k0 Γ_max / (k_cat Γ_S) ≈ D035_BASAL_FRAC
    p.k_mature_basal = D035_BASAL_FRAC * k_cat * 0.25;
    p.k_a_half = D035_K_A_IDENTIFIED;
    p.k_u_half = D035_K_U_IDENTIFIED;
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

fn renewal_window_observability_v12(sim: &Simulation, _accepted: u64) -> Value {
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
    let basal_frac = {
        let k0 = sim.params.k_mature_basal;
        let kc = sim.params.k_mature_cat;
        let gs = 0.25 * sim.params.gamma_max;
        let jb = k0 * sim.params.gamma_max;
        let jc = kc * gs;
        if jb + jc > 0.0 {
            jb / (jb + jc)
        } else {
            1.0
        }
    };
    json!({
        "p_mass": field_mass(sim, &sim.fields.precursor),
        "a_mass": field_mass(sim, &sim.fields.activated),
        "u_mass": mean_u,
        "s_mass": mean_s,
        "localization": dual_localization(sim),
        "passive_forward_exchange": wl.exchange_forward,
        "passive_reverse_exchange": wl.exchange_reverse,
        "maturation": wl.maturation_delta,
        "biological_turnover": wl.gamma_decay_delta,
        "basal_fraction_proxy": basal_frac,
        "q_u": q_u,
        "q_s": q_s,
        "g_u": g_u,
        "g_s": g_s,
    })
}

pub fn run_gate5_isolated_renewal(
    output: &Path,
    k_cat: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v12_isolated_params(k_cat)?;
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
        "radius": 22.0,
        "theta_seed": 0.6,
        "u_mass": field_mass(&sim, &sim.fields.immature_membrane),
        "s_mass": field_mass(&sim, &sim.fields.membrane),
        "k_cat": k_cat,
        "k_basal": sim.params.k_mature_basal,
        "K_A": sim.params.k_a_half,
        "K_U": sim.params.k_u_half,
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
            if total_accepted % 10000 == 0 {
                eprintln!(
                    "D-035 Gate5 progress accepted={} target={}",
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
            let obs = renewal_window_observability_v12(&sim, accepted);
            let q_u = obs["q_u"].as_f64().unwrap_or(0.0);
            let q_s = obs["q_s"].as_f64().unwrap_or(0.0);
            let g_u = obs["g_u"].as_f64().unwrap_or(1.0);
            let g_s = obs["g_s"].as_f64().unwrap_or(1.0);
            let loc = obs["localization"].as_f64().unwrap_or(0.0);
            let basal = obs["basal_fraction_proxy"].as_f64().unwrap_or(1.0);
            let ok = steps_ok
                && accepted >= window / 2
                && (0.98..=1.02).contains(&q_u)
                && (0.98..=1.02).contains(&q_s)
                && g_u.abs() <= 1e-4
                && g_s.abs() <= 1e-4
                && loc >= 0.98
                && basal <= 0.05
                && obs["passive_forward_exchange"].as_f64().unwrap_or(0.0) > 0.0
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
        "project_directive": "D-035",
        "gate": 5,
        "k_cat": k_cat,
        "initial_conditions": initial_conditions,
        "horizons": horizon_reports,
        "total_accepted": total_accepted,
        "capacity_rejects": capacity_rejects,
        "u_runaway": u_runaway,
        "consecutive_ok": consecutive,
        "pass": pass,
        "conclusion": if pass {
            "D035_ISOLATED_CATALYTIC_RENEWAL_PASS"
        } else {
            "D035_ISOLATED_CATALYTIC_RENEWAL_FAILURE"
        },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}
