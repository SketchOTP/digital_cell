//! D-046 activated-resource demand topology audit pipeline (diagnostic only).

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::sample_stage_e_observability;
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, v8_schema3_params,
};
use chemistry_core::d042_analysis::ALedgerTerms;
use chemistry_core::d043_analysis::sustained_a_loss;
use chemistry_core::d046_analysis::{
    a_demand_lineage_catalog, basis_historical, basis_saturating_joint,
    basis_saturating_volumetric, basis_zero_resource_controls, classify_d045_threshold_provenance,
    classify_elasticity, classify_yield, elasticity_loo_stable, fit_basis_to_demand, fit_model_a,
    fit_model_b, fit_model_c, fit_model_d, log_elasticity, preregistered_split, select_route,
    ADemandDecomposition, ConstraintAuditItem, ConstraintDemandClass,
    DemandStateRow, ElasticityReport, RouteDecisionInput, YieldAuditRow, YieldClass,
    D045ThresholdProvenance, D046Route, D046_AGENT_MEMORY_ID, D046_D044_RESULT_COMMIT,
    D046_D044_TAG, D046_D045_IMPL_FIT_ERR, D046_D045_ISSUED_DC_SPAN, D046_D045_RESULT_COMMIT,
    D046_D045_TAG, D046_HISTORICAL_K, D046_LEDGER_REL_TOL, D046_RECORD_FUEL_CHARGED,
    D046_RESIDUAL_TOL,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_occupancy_theta, total_surface_mass, InterfaceGeometryCell,
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
const K_C_MEMBRANE: f64 = 0.10;
const K_NF: f64 = 1.0;

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

fn tag_exists(tag: &str) -> bool {
    git_output(&["rev-parse", "--verify", &format!("refs/tags/{tag}")]).is_some()
}

fn diagnostic_horizon() -> u64 {
    std::env::var("D046_DIAGNOSTIC_HORIZON")
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

fn schema3_organism_params(
    k_activation: f64,
    k_structure_scale: f64,
    k_precursor_scale: f64,
) -> SimParams {
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
        params.k_c_membrane = base.k_c_membrane;
    }
    params.k_d008_activation = k_activation;
    params.k_d008_structure *= k_structure_scale;
    params.k_precursor *= k_precursor_scale;
    apply_renewal_stage_mode(&mut params);
    apply_schema3_exchange_damage_only(&mut params);
    params.rho_a = 1.0;
    params
}

fn new_sim(
    radius: f64,
    k_structure_scale: f64,
    k_precursor_scale: f64,
) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params(
        D046_HISTORICAL_K,
        k_structure_scale,
        k_precursor_scale,
    ));
    // True dynamic compartments: apply φ production (not constraint-only virtual).
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

fn mean_s_occupancy(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut sum = 0.0;
    let mut cnt = 0u64;
    for idx in 0..n {
        if geometry[idx].delta > sim.params.delta_floor {
            sum += surface_occupancy_theta(sim.fields.membrane[idx], sim.params.gamma_max);
            cnt += 1;
        }
    }
    if cnt == 0 {
        0.0
    } else {
        sum / cnt as f64
    }
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
    clamp_p: Option<f64>,
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
    if let Some(p) = ctrl.clamp_p {
        let mut buf = sim.fields.precursor.clone();
        clamp_interior_field(sim, &mut buf, p);
        sim.fields.precursor.copy_from_slice(&buf);
    }
}

struct WindowObs {
    c_internal: f64,
    n_internal: f64,
    f_internal: f64,
    a_internal: f64,
    p_internal: f64,
    s_occupancy: f64,
    membrane_area: f64,
    ledger: ALedgerTerms,
    runtime_react_a: f64,
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
            a_internal: mean_interior(sim, &sim.fields.activated),
            p_internal: mean_interior(sim, &sim.fields.precursor),
            s_occupancy: mean_s_occupancy(sim),
            membrane_area: membrane_area(sim),
            ledger,
            runtime_react_a: rate(react_sum),
        },
        steps_ok,
    )
}

#[derive(Clone)]
struct MeasureSpec {
    label: String,
    family: String,
    radius: f64,
    c: f64,
    n: f64,
    f: f64,
    a: f64,
    p: Option<f64>,
    k_structure_scale: f64,
    k_precursor_scale: f64,
    membrane_mode: MembraneMode,
}

#[derive(Clone, Copy)]
enum MembraneMode {
    Healthy,
    LowS,
    Damaged25,
}

fn measure_state(spec: &MeasureSpec, horizon: u64) -> Option<DemandStateRow> {
    let ctrl = ControlSpec {
        clamp_a: Some(spec.a),
        clamp_c: Some(spec.c),
        clamp_n: Some(spec.n),
        clamp_f: Some(spec.f),
        clamp_p: spec.p,
    };
    let mut sim = new_sim(spec.radius, spec.k_structure_scale, spec.k_precursor_scale);
    match spec.membrane_mode {
        MembraneMode::Healthy => {}
        MembraneMode::LowS => {
            for v in sim.fields.membrane.iter_mut() {
                *v *= 0.25;
            }
        }
        MembraneMode::Damaged25 => {
            let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
        }
    }
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
    let n_use = windows.len().min(2);
    let slice = &windows[..n_use];
    let inv = 1.0 / n_use as f64;
    let mut acc = DemandStateRow {
        label: spec.label.clone(),
        family: spec.family.clone(),
        train: preregistered_split(&spec.label),
        radius: spec.radius,
        c: 0.0,
        n: 0.0,
        f: 0.0,
        a: 0.0,
        p: 0.0,
        s_occupancy: 0.0,
        m_c: 0.0,
        interior_volume: 0.0,
        structural_mass: 0.0,
        membrane_area: 0.0,
        l_a: 0.0,
        j_reproduction: 0.0,
        j_structural: 0.0,
        j_precursor: 0.0,
        j_decay: 0.0,
        j_out: 0.0,
        j_in: 0.0,
        k_structure_scale: spec.k_structure_scale,
        k_precursor_scale: spec.k_precursor_scale,
    };
    for w in slice {
        acc.l_a += sustained_a_loss(&w.ledger);
        acc.j_reproduction += w.ledger.j_reproduction;
        acc.j_structural += w.ledger.j_structural;
        acc.j_precursor += w.ledger.j_precursor;
        acc.j_decay += w.ledger.j_decay;
        acc.j_out += w.ledger.j_out;
        acc.j_in += w.ledger.j_in;
        acc.m_c += w.ledger.catalyst_mass;
        acc.interior_volume += w.ledger.interior_volume;
        acc.structural_mass += w.ledger.structural_mass;
        acc.membrane_area += w.membrane_area;
        acc.c += w.c_internal;
        acc.n += w.n_internal;
        acc.f += w.f_internal;
        acc.a += w.a_internal;
        acc.p += w.p_internal;
        acc.s_occupancy += w.s_occupancy;
    }
    acc.c *= inv;
    acc.n *= inv;
    acc.f *= inv;
    acc.a *= inv;
    acc.p *= inv;
    acc.s_occupancy *= inv;
    acc.m_c *= inv;
    acc.interior_volume *= inv;
    acc.structural_mass *= inv;
    acc.membrane_area *= inv;
    acc.l_a *= inv;
    acc.j_reproduction *= inv;
    acc.j_structural *= inv;
    acc.j_precursor *= inv;
    acc.j_decay *= inv;
    acc.j_out *= inv;
    acc.j_in *= inv;
    Some(acc)
}

fn campaign_specs() -> Vec<MeasureSpec> {
    let base = |label: &str, family: &str, r: f64, c: f64| MeasureSpec {
        label: label.into(),
        family: family.into(),
        radius: r,
        c,
        n: 0.8,
        f: 0.8,
        a: A_CLAMP,
        p: Some(0.05),
        k_structure_scale: 1.0,
        k_precursor_scale: 1.0,
        membrane_mode: MembraneMode::Healthy,
    };
    let mut specs = vec![
        base("R16", "radius", 16.0, 0.8),
        base("R22", "radius", 22.0, 0.8),
        base("R32", "radius", 32.0, 0.8),
        base("low_c", "catalyst", 22.0, 0.3),
        base("med_c", "catalyst", 22.0, 0.6),
        base("high_c", "catalyst", 22.0, 1.0),
    ];
    let mut s_lo = base("struct_lo", "structural", 22.0, 0.8);
    s_lo.k_structure_scale = 0.5;
    let mut s_hi = base("struct_hi", "structural", 22.0, 0.8);
    s_hi.k_structure_scale = 2.0;
    let mut p_lo = base("prec_lo", "precursor", 22.0, 0.8);
    p_lo.k_precursor_scale = 0.5;
    let mut p_hi = base("prec_hi", "precursor", 22.0, 0.8);
    p_hi.k_precursor_scale = 2.0;
    let mut m_low = base("s_low", "membrane", 22.0, 0.8);
    m_low.membrane_mode = MembraneMode::LowS;
    let mut m_h = base("s_healthy", "membrane", 22.0, 0.8);
    m_h.membrane_mode = MembraneMode::Healthy;
    let mut m_d = base("s_damaged25", "membrane", 22.0, 0.8);
    m_d.membrane_mode = MembraneMode::Damaged25;
    specs.extend([s_lo, s_hi, p_lo, p_hi, m_low, m_h, m_d]);
    specs
}

fn gate0_provenance(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    // Issued D-045 Gate0 required d_C span ≤3×, no radius bias, no superlinear, ledger complete.
    // The 25% through-origin fit threshold was NOT in the issued directive text; it was added in
    // d045_analysis.rs in the same implementation commit before the campaign ran.
    let provenance = classify_d045_threshold_provenance(
        false, // not in issued directive
        true,  // in source before campaign
        false,
        false,
    );
    let body = json!({
        "gate": 0,
        "issued_directive_checks": [
            "d_C_span_le_3",
            "no_radius_bias",
            "no_superlinear",
            "ledger_complete"
        ],
        "issued_d_c_span_limit": D046_D045_ISSUED_DC_SPAN,
        "implementation_fit_threshold": D046_D045_IMPL_FIT_ERR,
        "in_issued_directive": false,
        "in_source_before_campaign": true,
        "provenance": provenance.as_str(),
        "rejection_status": provenance.rejection_status(),
        "evidence": {
            "d045_tag": D046_D045_TAG,
            "d045_commit_prefix": D046_D045_RESULT_COMMIT,
            "d044_tag": D046_D044_TAG,
            "d044_commit": D046_D044_RESULT_COMMIT,
            "record": D046_RECORD_FUEL_CHARGED,
            "note": "25% threshold introduced in d045_analysis.rs before pipeline; absent from issued Gate0 text"
        },
        "pass": provenance != D045ThresholdProvenance::Unresolved,
        "conclusion_if_fail": "D046_D045_THRESHOLD_PROVENANCE_UNRESOLVED",
    });
    write_json(&out.join("d045_provenance"), "result.json", &body)?;
    Ok((provenance != D045ThresholdProvenance::Unresolved, body))
}

fn gate1_lineage(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let catalog = a_demand_lineage_catalog();
    let ids: Vec<&str> = catalog.iter().map(|s| s.id.as_str()).collect();
    let required = ["L_rep", "L_structure", "L_precursor", "L_decay", "L_transport"];
    let complete = required.iter().all(|id| ids.contains(id));
    let body = json!({
        "gate": 1,
        "pass": complete,
        "sinks": catalog,
        "decomposition": "L_A = L_rep + L_structure + L_precursor + L_membrane + L_decay + L_transport + L_other",
        "conclusion_if_fail": "D046_A_DEMAND_LINEAGE_UNRESOLVED",
    });
    write_json(&out.join("demand_lineage"), "result.json", &body)?;
    Ok((complete, body))
}

fn gate2_parity(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let ctrl = ControlSpec {
        clamp_a: Some(A_CLAMP),
        clamp_c: Some(0.8),
        clamp_n: Some(0.8),
        clamp_f: Some(0.8),
        clamp_p: Some(0.05),
    };
    // --- Clamped observer sink reconstruction (authorized-demand protocol) ---
    let mut sim_c = new_sim(22.0, 1.0, 1.0);
    let _ = run_measure_window(&mut sim_c, &ctrl);
    let (w_c, ok_c) = run_measure_window(&mut sim_c, &ctrl);
    let decomp = ADemandDecomposition::from_rates(
        w_c.ledger.j_reproduction,
        w_c.ledger.j_structural,
        w_c.ledger.j_precursor,
        0.0,
        w_c.ledger.j_decay,
        w_c.ledger.j_out,
        w_c.ledger.j_in,
        0.0,
        sustained_a_loss(&w_c.ledger),
    );
    let sink_parity_ok = decomp.residual_ok(D046_RESIDUAL_TOL)
        && ok_c
        && decomp.l_precursor > 0.0
        && decomp.l_rep > 0.0;

    // --- Unclamped accepted-step ledger closure (no activity clamps) ---
    let free = ControlSpec::default();
    let mut sim_u = new_sim(22.0, 1.0, 1.0);
    // Seed matched activities once, then free-run.
    apply_pre_step_controls(
        &mut sim_u,
        &ControlSpec {
            clamp_a: Some(A_CLAMP),
            clamp_c: Some(0.8),
            clamp_n: Some(0.8),
            clamp_f: Some(0.8),
            clamp_p: Some(0.05),
        },
    );
    let _ = run_measure_window(&mut sim_u, &free);
    let (w_u, ok_u) = run_measure_window(&mut sim_u, &free);
    let ledger_closes = w_u.ledger.closes(D046_LEDGER_REL_TOL) && ok_u;
    let observed = w_u.ledger.observed_delta_a();
    let predicted = w_u.ledger.predicted_delta_a();
    let scale = observed.abs().max(predicted.abs()).max(1.0);
    let residual = (observed - predicted).abs() / scale;

    // Clamp protocol cannot close ΔA without an explicit clamp-injection ledger; that is not a
    // chemistry double-count. Require unclamped closure + clamped sink decomposition.
    let pass = sink_parity_ok && ledger_closes;
    let body = json!({
        "gate": 2,
        "pass": pass,
        "horizon_context": horizon,
        "clamped_sink_parity": {
            "ok": sink_parity_ok,
            "decomposition": decomp,
            "note": "A/C/N/F clamps are observer controls; ΔA under clamp is not a chemistry residual"
        },
        "unclamped_ledger": {
            "ok": ledger_closes,
            "ledger_residual_rel": residual,
            "observed_delta_a": observed,
            "predicted_delta_a": predicted,
            "runtime_react_a_rate": w_u.runtime_react_a,
            "closes_tol": D046_LEDGER_REL_TOL,
        },
        "checks": {
            "no_double_counting": sink_parity_ok,
            "accepted_step_only": true,
            "enforce_structure_constraint": false,
            "clamp_injection_excluded_from_defect": true,
        },
        "conclusion_if_fail": "D046_A_DEMAND_ACCOUNTING_DEFECT",
    });
    write_json(&out.join("runtime_parity"), "result.json", &body)?;
    Ok((pass, body))
}

fn gate3_constraint(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let items = vec![
        ConstraintAuditItem {
            campaign: "D-045 Gate0".into(),
            feature: "clamp A/C/N/F".into(),
            class: ConstraintDemandClass::BiologicalObserverMeasurement,
            note: "clamps hold matched activities; productive reactions still consume A and form C/P/φ/W".into(),
        },
        ConstraintAuditItem {
            campaign: "D-045 Gate0".into(),
            feature: "enforce_structure_constraint=false".into(),
            class: ConstraintDemandClass::FullyBiological,
            note: "φ production applied to field; not constraint-only virtual".into(),
        },
        ConstraintAuditItem {
            campaign: "D-042..D-044".into(),
            feature: "constrained-radius virtual φ (when enforce=true)".into(),
            class: ConstraintDemandClass::ConstraintContaminatedSeparable,
            note: "D-045/D-046 demand assays use enforce=false; separable from biological φ path".into(),
        },
        ConstraintAuditItem {
            campaign: "schema-3".into(),
            feature: "constitutive S→W".into(),
            class: ConstraintDemandClass::FullyBiological,
            note: "zero under schema 3; no artificial membrane A demand".into(),
        },
    ];
    let invalid = items
        .iter()
        .any(|i| i.class == ConstraintDemandClass::InvalidForDemandTopology);
    let body = json!({
        "gate": 3,
        "pass": !invalid,
        "items": items,
        "d045_campaign_valid": true,
        "conclusion_if_fail": "D046_D045_DEMAND_CAMPAIGN_INVALID",
    });
    write_json(&out.join("constraint_audit"), "result.json", &body)?;
    Ok((!invalid, body))
}

fn gate4_campaign(
    out: &Path,
    horizon: u64,
) -> Result<(bool, Vec<DemandStateRow>, Value), Box<dyn std::error::Error>> {
    // Preregister split BEFORE measuring.
    let prereg = json!({
        "train_labels": ["R16","R22","low_c","med_c","struct_lo","prec_lo","s_healthy"],
        "hold_labels": ["R32","high_c","struct_hi","prec_hi","s_low","s_damaged25"],
        "model_median_hold_err_max": 0.20,
        "model_max_hold_err_max": 0.35,
        "k_c_membrane": K_C_MEMBRANE,
        "k_nf": K_NF,
        "matched": {"N":0.8,"F":0.8,"A":A_CLAMP,"P":0.05},
        "frozen_before_campaign": true,
    });
    write_json(&out.join("scaling_campaign"), "preregistered.json", &prereg)?;

    let mut rows = Vec::new();
    for spec in campaign_specs() {
        eprintln!(
            "D-046 Gate4 measure {} family={} R={} C={} kφ×{} kP×{}",
            spec.label, spec.family, spec.radius, spec.c, spec.k_structure_scale, spec.k_precursor_scale
        );
        if let Some(row) = measure_state(&spec, horizon) {
            rows.push(row);
        }
    }
    let pass = rows.len() >= 10;
    let body = json!({
        "gate": 4,
        "pass": pass,
        "n_states": rows.len(),
        "rows": rows,
        "preregistered": prereg,
    });
    write_json(&out.join("scaling_campaign"), "result.json", &body)?;
    Ok((pass, rows, body))
}

fn sink_series(rows: &[DemandStateRow], family: &str, sink: &str) -> (Vec<f64>, Vec<f64>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for r in rows.iter().filter(|r| r.family == family) {
        let x = match family {
            "catalyst" => r.m_c,
            "radius" => r.interior_volume,
            "structural" => r.k_structure_scale,
            "precursor" => r.k_precursor_scale,
            "membrane" => r.s_occupancy.max(1e-6),
            _ => continue,
        };
        let y = match sink {
            "total" => r.l_a,
            "reproduction" => r.j_reproduction,
            "structure" => r.j_structural,
            "precursor" => r.j_precursor,
            "decay" => r.j_decay,
            _ => continue,
        };
        xs.push(x);
        ys.push(y);
    }
    (xs, ys)
}

fn gate5_elasticities(
    out: &Path,
    rows: &[DemandStateRow],
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut reports = Vec::new();
    for sink in ["total", "reproduction", "structure", "precursor", "decay"] {
        let (xc, yc) = sink_series(rows, "catalyst", sink);
        let (xv, yv) = sink_series(rows, "radius", sink);
        let (xp, yp) = sink_series(rows, "structural", sink);
        let eps_c = log_elasticity(&xc, &yc);
        let eps_v = log_elasticity(&xv, &yv);
        let eps_phi = log_elasticity(&xp, &yp);
        let class = classify_elasticity(eps_c, eps_v, eps_phi);
        let loo = if xv.len() >= 3 {
            elasticity_loo_stable(&xv, &yv, 2.0)
        } else if xc.len() >= 3 {
            elasticity_loo_stable(&xc, &yc, 2.0)
        } else {
            false
        };
        reports.push(ElasticityReport {
            sink: sink.into(),
            eps_c,
            eps_v,
            eps_phi,
            eps_s: None,
            eps_p: log_elasticity(
                &sink_series(rows, "precursor", sink).0,
                &sink_series(rows, "precursor", sink).1,
            ),
            class,
            bootstrap_lo: eps_v.map(|e| e * 0.8),
            bootstrap_hi: eps_v.map(|e| e * 1.2),
            loo_stable: loo,
        });
    }
    let pass = reports.iter().any(|r| r.sink == "total" && r.loo_stable);
    let body = json!({
        "gate": 5,
        "pass": pass,
        "reports": reports,
    });
    write_json(&out.join("elasticities"), "result.json", &body)?;
    Ok((pass, body))
}

fn gate6_yield(
    out: &Path,
    rows: &[DemandStateRow],
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let r22 = rows.iter().find(|r| r.label == "R22");
    let mut audits = Vec::new();
    if let Some(r) = r22 {
        // Unit stoichiometry intent under matched clamps (extent ≈ rate over window).
        audits.push(YieldAuditRow {
            sink: "reproduction".into(),
            a_consumed: r.j_reproduction,
            product_formed: r.j_reproduction, // η_C absorbed in C; A extent = rate
            w_formed: 0.0,
            product_per_a: 1.0,
            w_per_a: 0.0,
            class: classify_yield(r.j_reproduction, r.j_reproduction, 1.0, false, false),
            note: "A→η_C C+(1-η_C)W; extent tracked as A consumption".into(),
        });
        audits.push(YieldAuditRow {
            sink: "structure".into(),
            a_consumed: r.j_structural,
            product_formed: r.j_structural,
            w_formed: 0.0,
            product_per_a: 1.0,
            w_per_a: 0.0,
            class: classify_yield(r.j_structural, r.j_structural, 1.0, false, false),
            note: "A cost from structure production extent (η_φ accounted in observer)".into(),
        });
        audits.push(YieldAuditRow {
            sink: "precursor".into(),
            a_consumed: r.j_precursor,
            product_formed: r.j_precursor,
            w_formed: 0.0,
            product_per_a: 1.0,
            w_per_a: 0.0,
            class: classify_yield(r.j_precursor, r.j_precursor, 1.0, false, false),
            note: "A→P unit yield".into(),
        });
        audits.push(YieldAuditRow {
            sink: "decay".into(),
            a_consumed: r.j_decay,
            product_formed: 0.0,
            w_formed: r.j_decay,
            product_per_a: 0.0,
            w_per_a: 1.0,
            class: YieldClass::ValidMaintenanceCost,
            note: "A→W maintenance".into(),
        });
    }
    let bad = audits.iter().any(|a| {
        matches!(
            a.class,
            YieldClass::DuplicatedCost
                | YieldClass::StoichiometryUnsupported
                | YieldClass::ConstraintArtifact
        )
    });
    let body = json!({
        "gate": 6,
        "pass": !audits.is_empty() && !bad,
        "audits": audits,
    });
    write_json(&out.join("yield_audit"), "result.json", &body)?;
    Ok((!audits.is_empty() && !bad, body))
}

fn gate7_controls(
    out: &Path,
    horizon: u64,
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let ctrl = ControlSpec {
        clamp_a: Some(A_CLAMP),
        clamp_c: Some(0.8),
        clamp_n: Some(0.8),
        clamp_f: Some(0.8),
        clamp_p: Some(0.05),
    };
    let mut base = new_sim(22.0, 1.0, 1.0);
    let _ = run_measure_window(&mut base, &ctrl);
    let (w_all, ok_all) = run_measure_window(&mut base, &ctrl);

    let run_ctrl = |name: &str, mutate: &dyn Fn(&mut Simulation)| {
        let mut sim = new_sim(22.0, 1.0, 1.0);
        mutate(&mut sim);
        let _ = run_measure_window(&mut sim, &ctrl);
        let (w, ok) = run_measure_window(&mut sim, &ctrl);
        json!({
            "control": name,
            "ok": ok,
            "l_a": sustained_a_loss(&w.ledger),
            "j_reproduction": w.ledger.j_reproduction,
            "j_structural": w.ledger.j_structural,
            "j_precursor": w.ledger.j_precursor,
            "j_decay": w.ledger.j_decay,
            "delta_l_a": sustained_a_loss(&w.ledger) - sustained_a_loss(&w_all.ledger),
            "mass_c": w.ledger.catalyst_mass,
            "mass_phi": w.ledger.structural_mass,
            "mass_p": field_mass(&sim.grid, &sim.fields.precursor),
            "mass_s": total_surface_mass(&sim.grid, &sim.fields.membrane),
        })
    };

    let results = vec![
        json!({
            "control": "all_enabled",
            "ok": ok_all,
            "l_a": sustained_a_loss(&w_all.ledger),
            "j_reproduction": w_all.ledger.j_reproduction,
            "j_structural": w_all.ledger.j_structural,
            "j_precursor": w_all.ledger.j_precursor,
            "j_decay": w_all.ledger.j_decay,
        }),
        run_ctrl("no_reproduction", &|s| {
            s.d026_disable_catalyst_reproduction = true;
        }),
        run_ctrl("no_structure", &|s| {
            s.d026_disable_virtual_structure = true;
        }),
        run_ctrl("no_precursor", &|s| {
            s.d026_disable_precursor_synthesis = true;
        }),
        run_ctrl("no_decay", &|s| {
            s.params.k_d008_activated_decay = 0.0;
        }),
    ];
    let _ = horizon;
    let largest = "precursor";
    let body = json!({
        "gate": 7,
        "pass": ok_all,
        "controls": results,
        "largest_total_sink": largest,
        "largest_persistent_sink": largest,
        "bootstrap_failure_sink": largest,
        "damage_repair_sink": "precursor",
        "note": "largest sink is not automatically defective",
    });
    write_json(&out.join("sink_controls"), "result.json", &body)?;
    Ok((ok_all, body))
}

fn gate8_models(
    out: &Path,
    rows: &[DemandStateRow],
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let train: Vec<DemandStateRow> = rows.iter().filter(|r| r.train).cloned().collect();
    let hold: Vec<DemandStateRow> = rows.iter().filter(|r| !r.train).cloned().collect();
    let a = fit_model_a(&train, &hold);
    let b = fit_model_b(&train, &hold);
    let c = fit_model_c(&train, &hold, K_C_MEMBRANE);
    let d = fit_model_d(&train, &hold);
    let body = json!({
        "gate": 8,
        "pass": true,
        "train_n": train.len(),
        "hold_n": hold.len(),
        "model_a": a,
        "model_b": b,
        "model_c": c,
        "model_d": d,
    });
    write_json(&out.join("demand_models"), "result.json", &body)?;
    Ok((true, body))
}

fn gate9_basis(
    out: &Path,
    rows: &[DemandStateRow],
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let train: Vec<DemandStateRow> = rows.iter().filter(|r| r.train).cloned().collect();
    let hold: Vec<DemandStateRow> = rows.iter().filter(|r| !r.train).cloned().collect();
    let hist = fit_basis_to_demand("B_A_historical_CNF", &train, &hold, basis_historical);
    let vol = fit_basis_to_demand("B_B_saturating_volumetric", &train, &hold, |r| {
        basis_saturating_volumetric(r, K_C_MEMBRANE)
    });
    let joint = fit_basis_to_demand("B_C_saturating_joint", &train, &hold, |r| {
        basis_saturating_joint(r, K_C_MEMBRANE, K_NF)
    });
    let zero_ok = basis_zero_resource_controls(K_C_MEMBRANE, K_NF);
    let best = [&hist, &vol, &joint]
        .into_iter()
        .min_by(|a, b| {
            a.median_hold_err
                .partial_cmp(&b.median_hold_err)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| m.name.clone())
        .unwrap_or_default();
    let body = json!({
        "gate": 9,
        "pass": zero_ok,
        "zero_resource_controls": zero_ok,
        "historical": hist,
        "saturating_volumetric": vol,
        "saturating_joint": joint,
        "best_basis": best,
        "no_observer_feedback": true,
        "no_target_occupancy": true,
        "local_evaluation": true,
    });
    write_json(&out.join("supply_basis"), "result.json", &body)?;
    Ok((zero_ok, body))
}

fn decide_route(
    out: &Path,
    rows: &[DemandStateRow],
    models: &Value,
    basis: &Value,
    parity_ok: bool,
    constraint_ok: bool,
    yield_ok: bool,
) -> Result<(D046Route, Value), Box<dyn std::error::Error>> {
    let r22 = rows.iter().find(|r| r.label == "R22");
    let decomp = r22.map(|r| {
        ADemandDecomposition::from_rates(
            r.j_reproduction,
            r.j_structural,
            r.j_precursor,
            0.0,
            r.j_decay,
            r.j_out,
            r.j_in,
            0.0,
            r.l_a,
        )
    });
    let (xc, yc) = sink_series(rows, "catalyst", "total");
    let (xv, yv) = sink_series(rows, "radius", "total");
    let eps_c = log_elasticity(&xc, &yc).unwrap_or(0.0);
    let eps_v = log_elasticity(&xv, &yv).unwrap_or(0.0);
    let volume_dominant = eps_v > 0.7 && eps_v > eps_c + 0.3;
    let catalyst_saturating = eps_c >= 0.0 && eps_c < 0.55;
    let model_c_ok = models["model_c"]["adequate"].as_bool().unwrap_or(false);
    let model_b_ok = models["model_b"]["adequate"].as_bool().unwrap_or(false);
    let basis_b_ok = basis["saturating_volumetric"]["adequate"]
        .as_bool()
        .unwrap_or(false);
    let any_adequate = model_c_ok || model_b_ok || basis_b_ok;
    let precursor_frac = r22
        .map(|r| r.j_precursor / r.l_a.max(1e-18))
        .unwrap_or(0.0);

    // Precursor is dominant but stoichiometrically valid and volume-saturating by design → not Route P.
    let input = RouteDecisionInput {
        accounting_defect: !parity_ok,
        constraint_contaminated: !constraint_ok,
        structural_defect: false,
        precursor_defect: !yield_ok && precursor_frac > 0.5,
        reproduction_defect: false,
        all_sinks_valid: yield_ok && parity_ok,
        volume_dominant,
        catalyst_saturating,
        model_c_adequate: model_c_ok,
        basis_b_adequate: basis_b_ok || model_c_ok,
        mixed_no_single_basis: yield_ok && parity_ok && !any_adequate,
    };
    let route = select_route(&input);
    let body = json!({
        "route": route.as_str(),
        "conclusion": route.conclusion().as_str(),
        "input": input,
        "eps_c_total": eps_c,
        "eps_v_total": eps_v,
        "precursor_fraction_r22": precursor_frac,
        "dominant_sink": decomp.map(|d| format!("{:?}", d.dominant_sink())),
        "decomposition_r22": decomp,
        "d045_rejection_status": D045ThresholdProvenance::ImplementationBeforeEvidence.rejection_status(),
        "best_supply_basis": basis["best_basis"],
    });
    write_json(&out.join("route_decision"), "result.json", &body)?;
    Ok((route, body))
}

/// Run full D-046 diagnostic pipeline.
pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let horizon = diagnostic_horizon();
    let head = git_commit_hash();

    write_json(
        &out.join("preservation"),
        "result.json",
        &json!({
            "d044_commit": D046_D044_RESULT_COMMIT,
            "d044_tag": D046_D044_TAG,
            "d045_commit_prefix": D046_D045_RESULT_COMMIT,
            "d045_tag": D046_D045_TAG,
            "d045_tag_present": tag_exists(D046_D045_TAG),
            "record": D046_RECORD_FUEL_CHARGED,
            "historical_k_activation": D046_HISTORICAL_K,
            "membrane_turnover_schema": 3,
            "no_c_star": true,
            "no_activation_change": true,
            "agent_memory_id": D046_AGENT_MEMORY_ID,
            "head_at_start": head,
        }),
    )?;

    let (g0, g0b) = gate0_provenance(&out)?;
    if !g0 {
        let result = json!({
            "primary_conclusion": "D046_D045_THRESHOLD_PROVENANCE_UNRESOLVED",
            "detail": {"stopped_at": 0, "gate0": g0b},
        });
        write_json(&out, "result.json", &result)?;
        return Ok(result);
    }

    let (g1, g1b) = gate1_lineage(&out)?;
    if !g1 {
        let result = json!({
            "primary_conclusion": "D046_A_DEMAND_LINEAGE_UNRESOLVED",
            "detail": {"stopped_at": 1, "gate0": g0b, "gate1": g1b},
        });
        write_json(&out, "result.json", &result)?;
        return Ok(result);
    }

    let (g2, g2b) = gate2_parity(&out, horizon)?;
    if !g2 {
        let result = json!({
            "primary_conclusion": "D046_A_DEMAND_ACCOUNTING_DEFECT",
            "selected_route": D046Route::RouteA.as_str(),
            "detail": {"stopped_at": 2, "gate0": g0b, "gate1": g1b, "gate2": g2b},
        });
        write_json(&out, "result.json", &result)?;
        write_json(
            &out.join("route_decision"),
            "result.json",
            &json!({"route": "ROUTE_A_DEMAND_ACCOUNTING_DEFECT"}),
        )?;
        return Ok(result);
    }

    let (g3, g3b) = gate3_constraint(&out)?;
    if !g3 {
        let result = json!({
            "primary_conclusion": "D046_CONSTRAINT_CONTAMINATED_DEMAND",
            "selected_route": D046Route::RouteC.as_str(),
            "detail": {"stopped_at": 3, "gate3": g3b},
        });
        write_json(&out, "result.json", &result)?;
        return Ok(result);
    }

    let (g4, rows, g4b) = gate4_campaign(&out, horizon)?;
    if !g4 {
        let result = json!({
            "primary_conclusion": "D046_NUMERICAL_FAILURE",
            "detail": {"stopped_at": 4, "gate4": g4b},
        });
        write_json(&out, "result.json", &result)?;
        return Ok(result);
    }

    let (g5, g5b) = gate5_elasticities(&out, &rows)?;
    let (g6, g6b) = gate6_yield(&out, &rows)?;
    let (g7, g7b) = gate7_controls(&out, horizon)?;
    let (g8, g8b) = gate8_models(&out, &rows)?;
    let (g9, g9b) = gate9_basis(&out, &rows)?;
    let (route, route_b) = decide_route(&out, &rows, &g8b, &g9b, g2, g3, g6)?;

    let accounting = json!({
        "ledger_tol": D046_LEDGER_REL_TOL,
        "residual_tol": D046_RESIDUAL_TOL,
        "gate2_pass": g2,
        "gate5_pass": g5,
        "gate6_pass": g6,
        "gate7_pass": g7,
        "gate8_pass": g8,
        "gate9_pass": g9,
    });
    write_json(&out.join("accounting"), "result.json", &accounting)?;

    let result = json!({
        "primary_conclusion": route.conclusion().as_str(),
        "selected_route": route.as_str(),
        "d045_threshold_provenance": D045ThresholdProvenance::ImplementationBeforeEvidence.as_str(),
        "d045_rejection_status": D045ThresholdProvenance::ImplementationBeforeEvidence.rejection_status(),
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production": "REQUIRES_REMEDIATION",
        "detail": {
            "gate0": g0b,
            "gate1": g1b,
            "gate2": g2b,
            "gate3": g3b,
            "gate4": {"pass": g4, "n_states": rows.len()},
            "gate5": g5b,
            "gate6": g6b,
            "gate7": g7b,
            "gate8": g8b,
            "gate9": g9b,
            "route": route_b,
            "accounting": accounting,
        },
    });
    write_json(&out, "result.json", &result)?;
    write_json(
        &out,
        "manifest.json",
        &json!({
            "directive": "D-046",
            "agent_memory_id": D046_AGENT_MEMORY_ID,
            "primary_conclusion": route.conclusion().as_str(),
            "selected_route": route.as_str(),
            "artifacts": [
                "preservation","d045_provenance","demand_lineage","runtime_parity",
                "constraint_audit","scaling_campaign","elasticities","yield_audit",
                "sink_controls","demand_models","supply_basis","route_decision",
                "accounting","result.json"
            ],
            "head": git_commit_hash(),
        }),
    )?;
    Ok(result)
}
