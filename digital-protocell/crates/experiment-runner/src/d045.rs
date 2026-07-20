//! D-045 fuel-charged catalyst activation — Phase A gates (−1, 0); stop before C_star.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::sample_stage_e_observability;
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, v8_schema3_params,
};
use chemistry_core::d042_analysis::ALedgerTerms;
use chemistry_core::d045_analysis::{
    d044_seal_consistent, evaluate_demand_scaling, DemandScalingRow, D045Conclusion,
    D045_AGENT_MEMORY_ID, D045_D044_TAG, D045_HISTORICAL_K, D045_RECORD_BRANCH_CLOSED,
};
use chemistry_core::field_mass;
use chemistry_core::surface_density::{
    compute_interface_geometry, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WINDOW: u64 = 500;
const THETA: f64 = 0.6;
const A_CLAMP: f64 = 0.5;
const DIAG_DEFAULT: u64 = 1_500;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn git_commit_hash() -> String {
    git_output(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into())
}

fn git_tag_target(tag: &str) -> Option<String> {
    git_output(&["rev-parse", &format!("{tag}^{{}}")])
}

fn tag_exists(tag: &str) -> bool {
    git_output(&["rev-parse", "--verify", &format!("refs/tags/{tag}")]).is_some()
}

fn binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| fs::read(p).ok())
        .map(|b| chemistry_core::sha256_hex(&b))
        .unwrap_or_else(|| "unknown".into())
}

fn diagnostic_horizon() -> u64 {
    std::env::var("D045_DIAGNOSTIC_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DIAG_DEFAULT)
        .max(2 * WINDOW)
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn schema3_organism_params(k_activation: f64) -> SimParams {
    let mut params = v8_schema3_params();
    if let Ok(base) = v7_base_params() {
        params.beta_c = base.beta_c;
        params.beta_a = base.beta_a;
        params.beta_n = base.beta_n;
        params.beta_f = base.beta_f;
        params.beta_w = base.beta_w;
        params.k_phi = base.k_phi;
        params.k_structure = base.k_structure;
        params.k_rep = base.k_rep;
        params.k_d008_activation = base.k_d008_activation;
        params.k_d008_reproduction = base.k_d008_reproduction;
        params.k_d008_activated_decay = base.k_d008_activated_decay;
        params.k_d008_catalyst_turnover = base.k_d008_catalyst_turnover;
        params.k_d008_structure = base.k_d008_structure;
        params.k_precursor = base.k_precursor;
        params.k_precursor_decay = base.k_precursor_decay;
        params.d_p = base.d_p;
    }
    params.k_d008_activation = k_activation;
    apply_renewal_stage_mode(&mut params);
    apply_schema3_exchange_damage_only(&mut params);
    params.rho_a = 1.0;
    params
}

fn new_sim_radius(k_activation: f64, radius: f64) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params(k_activation));
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, radius, THETA);
    sim
}

fn mean_interior(sim: &Simulation, field: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut n = 0u64;
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            sum += field[idx].max(0.0);
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

fn interior_volume(sim: &Simulation) -> f64 {
    let mut n = 0u64;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            n += 1;
        }
    }
    n as f64
}

fn membrane_area(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    geometry
        .iter()
        .map(|g| {
            if g.delta > sim.params.delta_floor {
                g.delta
            } else {
                0.0
            }
        })
        .sum()
}

fn clamp_interior_field(sim: &mut Simulation, field: &mut [f64], value: f64) {
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            field[idx] = value.max(0.0);
        }
    }
}

#[derive(Clone, Default)]
struct ControlSpec {
    clamp_a: Option<f64>,
    clamp_c: Option<f64>,
    clamp_n: Option<f64>,
    clamp_f: Option<f64>,
}

fn apply_pre_step_controls(sim: &mut Simulation, ctrl: &ControlSpec) {
    if let Some(a) = ctrl.clamp_a {
        let mut buf = sim.fields.activated.clone();
        clamp_interior_field(sim, &mut buf, a);
        sim.fields.activated.copy_from_slice(&buf);
    }
    if let Some(c) = ctrl.clamp_c {
        let mut buf = sim.fields.catalyst.clone();
        clamp_interior_field(sim, &mut buf, c);
        sim.fields.catalyst.copy_from_slice(&buf);
    }
    if let Some(n) = ctrl.clamp_n {
        let mut buf = sim.fields.nutrient.clone();
        clamp_interior_field(sim, &mut buf, n);
        sim.fields.nutrient.copy_from_slice(&buf);
    }
    if let Some(f) = ctrl.clamp_f {
        let mut buf = sim.fields.fuel.clone();
        clamp_interior_field(sim, &mut buf, f);
        sim.fields.fuel.copy_from_slice(&buf);
    }
}

struct WindowObs {
    c_internal: f64,
    n_internal: f64,
    f_internal: f64,
    membrane_area: f64,
    resource_influx: f64,
    ledger: ALedgerTerms,
}

fn run_measure_window(sim: &mut Simulation, ctrl: &ControlSpec) -> (WindowObs, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let a_before = field_mass(&sim.grid, &sim.fields.activated);
    let t0 = sim.sim_time;
    let mut steps_ok = true;
    let mut act_sum = 0.0;
    let mut repro_sum = 0.0;
    let mut virt_sum = 0.0;
    let mut prec_sum = 0.0;
    let mut decay_sum = 0.0;
    let mut ain_sum = 0.0;
    let mut aout_sum = 0.0;
    let mut nf_in_sum = 0.0;
    let mut res_sum = 0.0;
    let mut num_sum = 0.0;
    let mut react_sum = 0.0;
    let mut diff_sum = 0.0;
    for _ in 0..WINDOW {
        apply_pre_step_controls(sim, ctrl);
        if !sim.step() {
            steps_ok = false;
            break;
        }
        let obs = sample_stage_e_observability(sim);
        act_sum += obs.a_production_activation;
        repro_sum += obs.a_consumption_catalyst_reproduction;
        virt_sum += obs.a_consumption_virtual_structural;
        prec_sum += obs.a_consumption_precursor_production.abs();
        decay_sum += obs.a_consumption_decay;
        ain_sum += obs.a_transport_in_flux;
        aout_sum += obs.a_transport_out_flux;
        let tr = &sim.transport_accounting.last_step;
        // Rates → extents over the accepted step.
        nf_in_sum += (tr.nutrient.interior_net_flux_rate.max(0.0)
            + tr.fuel.interior_net_flux_rate.max(0.0))
            * sim.dt.max(f64::EPSILON);
        let led = &sim.accounting.last_step.activated;
        res_sum += led.reservoir_delta;
        num_sum += led.numerical_correction_delta;
        react_sum += led.reaction_delta;
        diff_sum += led.diffusion_delta;
    }
    apply_pre_step_controls(sim, ctrl);
    let dt = (sim.sim_time - t0).max(f64::EPSILON);
    let a_after = field_mass(&sim.grid, &sim.fields.activated);
    let rate = |sum: f64| sum / dt;
    let field_predicted = react_sum + diff_sum + res_sum + num_sum;
    let decomp = act_sum + ain_sum - repro_sum - virt_sum - prec_sum - decay_sum - aout_sum;
    let decomp_residual = field_predicted - res_sum - num_sum - decomp;
    let ledger = ALedgerTerms {
        j_activation: rate(act_sum),
        j_in: rate(ain_sum),
        a_initial: a_before,
        j_reproduction: rate(repro_sum),
        j_structural: rate(virt_sum),
        j_precursor: rate(prec_sum),
        j_decay: rate(decay_sum),
        j_out: rate(aout_sum),
        j_reservoir: res_sum,
        numerical_correction: num_sum + decomp_residual,
        a_final: a_after,
        dt,
        interior_volume: interior_volume(sim),
        catalyst_mass: field_mass(&sim.grid, &sim.fields.catalyst),
        structural_mass: field_mass(&sim.grid, &sim.fields.structure),
        sim_time: sim.sim_time,
    };
    (
        WindowObs {
            c_internal: mean_interior(sim, &sim.fields.catalyst),
            n_internal: mean_interior(sim, &sim.fields.nutrient),
            f_internal: mean_interior(sim, &sim.fields.fuel),
            membrane_area: membrane_area(sim),
            resource_influx: rate(nf_in_sum),
            ledger,
        },
        steps_ok,
    )
}

fn measure_demand_row(
    label: &str,
    radius: f64,
    c: f64,
    n: f64,
    f: f64,
    horizon: u64,
) -> Option<DemandScalingRow> {
    let ctrl = ControlSpec {
        clamp_a: Some(A_CLAMP),
        clamp_c: Some(c),
        clamp_n: Some(n),
        clamp_f: Some(f),
    };
    let mut sim = new_sim_radius(D045_HISTORICAL_K, radius);
    let mut windows = Vec::new();
    let mut ok = true;
    while sim.substep < horizon && ok {
        let (w, s_ok) = run_measure_window(&mut sim, &ctrl);
        ok &= s_ok;
        windows.push(w);
    }
    if windows.is_empty() || !ok {
        return None;
    }
    // Early clamped windows: authorized demand while A is held healthy.
    let n_use = windows.len().min(2);
    let slice = &windows[..n_use];
    let inv = 1.0 / n_use as f64;
    let mut l_a = 0.0;
    let mut j_repro = 0.0;
    let mut j_struct = 0.0;
    let mut j_prec = 0.0;
    let mut j_decay = 0.0;
    let mut j_out = 0.0;
    let mut j_in = 0.0;
    let mut m_c = 0.0;
    let mut vol = 0.0;
    let mut s_mass = 0.0;
    let mut area = 0.0;
    let mut influx = 0.0;
    let mut c_m = 0.0;
    let mut n_m = 0.0;
    let mut f_m = 0.0;
    for w in slice {
        l_a += chemistry_core::d043_analysis::sustained_a_loss(&w.ledger);
        j_repro += w.ledger.j_reproduction;
        j_struct += w.ledger.j_structural;
        j_prec += w.ledger.j_precursor;
        j_decay += w.ledger.j_decay;
        j_out += w.ledger.j_out;
        j_in += w.ledger.j_in;
        m_c += w.ledger.catalyst_mass;
        vol += w.ledger.interior_volume;
        s_mass += w.ledger.structural_mass;
        area += w.membrane_area;
        influx += w.resource_influx;
        c_m += w.c_internal;
        n_m += w.n_internal;
        f_m += w.f_internal;
    }
    Some(DemandScalingRow {
        label: label.to_string(),
        radius,
        c: c_m * inv,
        n: n_m * inv,
        f: f_m * inv,
        l_a: l_a * inv,
        m_c: m_c * inv,
        interior_volume: vol * inv,
        structural_mass: s_mass * inv,
        membrane_area: area * inv,
        resource_influx: influx * inv,
        j_reproduction: j_repro * inv,
        j_structural: j_struct * inv,
        j_precursor: j_prec * inv,
        j_decay: j_decay * inv,
        j_out: j_out * inv,
        j_in: j_in * inv,
    })
}

fn gate_minus1_seal(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let head = git_commit_hash();
    let tag_ok = tag_exists(D045_D044_TAG);
    let tag_target = git_tag_target(D045_D044_TAG).unwrap_or_default();
    let starting = std::env::var("D045_STARTING_COMMIT").unwrap_or_else(|_| head.clone());
    let consistent = d044_seal_consistent(&starting, &tag_target, D045_D044_TAG);
    // Also accept when HEAD equals tag target (fresh seal) even if env unset.
    let head_matches = !tag_target.is_empty() && head == tag_target;
    let pass = tag_ok && (consistent || head_matches);
    let body = json!({
        "gate": -1,
        "pass": pass,
        "d044_tag": D045_D044_TAG,
        "tag_present": tag_ok,
        "tag_target": tag_target,
        "head": head,
        "starting_commit": starting,
        "seal_consistent": consistent || head_matches,
        "record": D045_RECORD_BRANCH_CLOSED,
        "frozen_conclusions": [
            "D042_ACTIVATION_CAPACITY_DEFICIT",
            "D043_ACTIVATION_RATE_NOT_PORTABLE",
            "D044_ACTIVATION_LAW_ARCHITECTURE_REJECTED"
        ],
        "conclusion_if_fail": D045Conclusion::D044EvidenceNotSealed.as_str(),
    });
    write_json(&out.join("d044_seal"), "result.json", &body)?;
    write_json(
        &out.join("preservation"),
        "result.json",
        &json!({
            "record": D045_RECORD_BRANCH_CLOSED,
            "d044_commit": tag_target,
            "d044_tag": D045_D044_TAG,
            "historical_k_activation": D045_HISTORICAL_K,
            "membrane_turnover_schema": 3,
            "no_constitutive_mature_s_to_w": true,
        }),
    )?;
    Ok((pass, body))
}

fn gate0_demand_scaling(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    // Matched N=F=0.8 across true radii and C levels.
    let specs: [(&str, f64, f64, f64, f64); 6] = [
        ("R16", 16.0, 0.8, 0.8, 0.8),
        ("R22", 22.0, 0.8, 0.8, 0.8),
        ("R32", 32.0, 0.8, 0.8, 0.8),
        ("low_c", 22.0, 0.3, 0.8, 0.8),
        ("med_c", 22.0, 0.6, 0.8, 0.8),
        ("high_c", 22.0, 1.0, 0.8, 0.8),
    ];
    let mut rows = Vec::new();
    for (label, radius, c, n, f) in specs {
        eprintln!("D-045 Gate0 measure {label} R={radius} C={c}");
        if let Some(row) = measure_demand_row(label, radius, c, n, f, horizon) {
            rows.push(row);
        }
    }
    let report = evaluate_demand_scaling(&rows);
    let body = json!({
        "gate": 0,
        "pass": report.pass,
        "report": report,
        "diagnostic_horizon": horizon,
        "note": "diagnostic states only; not organismal steady states",
        "conclusion_if_fail": D045Conclusion::CatalystLinearityRejected.as_str(),
    });
    write_json(&out.join("demand_scaling"), "result.json", &body)?;
    Ok((report.pass, body))
}

/// Run D-045 Phase A (Gate −1 seal, Gate 0 demand scaling). Stops before C_star.
pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let horizon = diagnostic_horizon();

    let (seal_ok, seal_body) = gate_minus1_seal(&out)?;
    if !seal_ok {
        let primary = D045Conclusion::D044EvidenceNotSealed;
        return finish(&out, primary, json!({
            "gate_minus1": seal_body,
            "stopped_at": "gate_minus1",
        }));
    }

    let (g0_ok, g0_body) = gate0_demand_scaling(&out, horizon)?;
    if !g0_ok {
        let primary = D045Conclusion::CatalystLinearityRejected;
        return finish(&out, primary, json!({
            "gate_minus1": seal_body,
            "gate0": g0_body,
            "stopped_at": "gate0",
            "c_star_implemented": false,
            "next_review": "A-demand topology rather than activation supply",
        }));
    }

    // Phase A Gate 0 passed — further gates require additional implementation.
    let primary = D045Conclusion::Fail;
    finish(
        &out,
        primary,
        json!({
            "gate_minus1": seal_body,
            "gate0": g0_body,
            "stopped_at": "gate0_pass_pending_gate1",
            "c_star_implemented": false,
            "note": "Gate 0 passed; Gates 1–11 not yet executed in this run",
        }),
    )
}

fn finish(
    out: &Path,
    primary: D045Conclusion,
    detail: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let qualified = primary == D045Conclusion::FuelChargedActivationQualified;
    let result = json!({
        "primary_conclusion": primary.as_str(),
        "qualified": qualified,
        "selected_architecture": if qualified {
            Value::String("V13_SCHEMA3_FUEL_CHARGED_CATALYST_ACTIVATION".into())
        } else {
            Value::Null
        },
        "c_star_implemented": false,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "detail": detail,
    });
    write_json(out, "result.json", &result)?;
    write_json(out, "decision.json", &json!({
        "primary_conclusion": primary.as_str(),
        "selected_architecture": result["selected_architecture"],
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production": "REQUIRES_REMEDIATION",
        "c_star_implemented": false,
    }))?;
    write_json(out, "manifest.json", &json!({
        "directive": "D-045",
        "agent_memory_id": D045_AGENT_MEMORY_ID,
        "primary_conclusion": primary.as_str(),
        "source_commit": git_commit_hash(),
        "d044_tag": D045_D044_TAG,
        "binary_hash": binary_hash(),
        "artifacts": [
            "d044_seal/",
            "preservation/",
            "demand_scaling/",
        ],
        "tag_recommended_fail": "D-045-fuel-charged-activation-fail",
        "tag_recommended_pass": "D-045-fuel-charged-activation-qualified",
    }))?;
    // Touch empty dirs required by the artifact contract for later gates.
    for d in [
        "qss_architecture",
        "catalyst_mapping",
        "activation_schema",
        "charge_identification",
        "transfer_identification",
        "pulse_order",
        "recycling",
        "portability",
        "activation_capacity",
        "foundational",
        "basin_multistart",
        "pulse_chase",
        "damage",
        "resource_controls",
        "stage_e_membrane_contract",
        "accounting",
    ] {
        fs::create_dir_all(out.join(d))?;
    }
    Ok(result)
}
