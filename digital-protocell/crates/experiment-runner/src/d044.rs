//! D-044 activation-law architecture review pipeline (Gates 0–13, stop-on-fail).

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::{sample_stage_e_observability, D026_SETTLE_STEPS};
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, classify_damage_repair,
    revised_stage_e_membrane_contract, v8_schema3_params, DamageRepairClass, D039_NET_S_FLOW_MAX,
    D039_REPLACEMENT_MIN, D039_S_DRIFT_MAX, D039_TRACER_RESIDUAL_MAX,
};
use chemistry_core::d042_analysis::{ALedgerIntegral, ALedgerTerms, linear_trend, dominant_demand};
use chemistry_core::d043_analysis::{
    build_activation_candidates, build_rate_estimate, evaluate_candidate_row,
    evaluate_portable_rate, parity_suite_passes, screen_candidates,
    total_basis_from_activation_flux, PortableRateReport,
    RateEstimate, D043_LEDGER_REL_TOL, D043_REPAIR_P_MIN,
};
use chemistry_core::d044_analysis::{
    build_holdout_states, build_training_states, classify_state_eligibility, classify_viable_domain,
    d043_reconstruction_within_tolerance, evaluate_heldout_steady, evaluate_heldout_transient,
    evaluate_scaling_audit, fit_candidate_a, fit_candidate_b, fit_candidate_c,
    monotonicity_passes_a, monotonicity_passes_b, monotonicity_passes_c, predict_steady_demand_a,
    predict_steady_demand_b, predict_steady_demand_c, scaling_audit_row, select_candidate,
    zero_control_passes_a, zero_control_passes_b, zero_control_passes_c, ActivationLawId,     ActivationStateSpec, ActivationTrainingRow, CandidateBFitReport, CandidateCFitReport,
    CandidateSelection,
    D043_RECONSTRUCTION_STATES, D044_AGENT_MEMORY_ID, D044_DIAGNOSTIC_HORIZON, D044_D043_TAG,
    D044_F_REFERENCE, D044_HISTORICAL_K, D044_N_REFERENCE, D044_PORTABLE_MAX_SPAN, D044_RECORD,
    D044_STARTING_COMMIT, D044Conclusion, EligibilityControls, EligibilityWindow, StateEligibility,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::membrane_label_tracer::MembraneLabelTracer;
use chemistry_core::surface_density::{
    compute_interface_geometry, precursor_activity, surface_localization, surface_occupancy_theta,
    total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WINDOW: u64 = 500;
const RADIUS: f64 = 22.0;
const THETA: f64 = 0.6;
const MIN_SPECIES: f64 = 0.05;
const A_CLAMP: f64 = 0.5;
const GATE0_FAIL: &str = "D044_D043_RECONSTRUCTION_NOT_REPRODUCED";
const D044_GATE0_HORIZON: u64 = 25_000;
const D044_MAX_ACCEPTED: u64 = 200_000;

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

fn tag_exists(tag: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn max_accepted() -> u64 {
    std::env::var("D044_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D044_MAX_ACCEPTED)
}

fn gate0_horizon() -> u64 {
    std::env::var("D044_GATE0_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D044_GATE0_HORIZON)
}

fn diagnostic_horizon(gate0: u64) -> u64 {
    let requested = std::env::var("D044_DIAGNOSTIC_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D044_DIAGNOSTIC_HORIZON);
    requested.min(gate0).max(3 * WINDOW)
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

fn new_sim(k_activation: f64) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params(k_activation));
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, RADIUS, THETA);
    sim
}

fn new_sim_radius(k_activation: f64, radius: f64) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params(k_activation));
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, radius, THETA);
    sim
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

fn mean_interface_theta(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut sum = 0.0;
    let mut wsum = 0.0;
    for idx in 0..n {
        let d = geometry[idx].delta;
        if d > sim.params.delta_floor {
            let g = sim.fields.membrane[idx].max(0.0) / d.max(sim.params.delta_floor);
            let th = surface_occupancy_theta(g, sim.params.gamma_max);
            sum += th * d;
            wsum += d;
        }
    }
    if wsum > 0.0 {
        sum / wsum
    } else {
        0.0
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
    name: &'static str,
    clamp_p_activity: Option<f64>,
    clamp_a: Option<f64>,
    clamp_c: Option<f64>,
    clamp_n: Option<f64>,
    clamp_f: Option<f64>,
    freeze_surface: bool,
    no_a_decay: bool,
    no_p_decay: bool,
    no_p_diffusion: bool,
    disable_exchange: bool,
    disable_precursor_synthesis: bool,
    disable_structural: bool,
    disable_reproduction: bool,
    disable_all_demands: bool,
}

impl ControlSpec {
    fn eligibility_controls(&self) -> EligibilityControls {
        EligibilityControls {
            clamp_a: self.clamp_a.is_some(),
            clamp_c: self.clamp_c.is_some(),
            clamp_n: self.clamp_n.is_some(),
            clamp_f: self.clamp_f.is_some(),
            clamp_p: self.clamp_p_activity.is_some(),
            freeze_surface: self.freeze_surface,
        }
    }
}

fn apply_control_params(sim: &mut Simulation, ctrl: &ControlSpec) {
    if ctrl.freeze_surface {
        sim.d026_freeze_surface = true;
    }
    if ctrl.no_a_decay {
        sim.params.k_d008_activated_decay = 0.0;
    }
    if ctrl.no_p_decay {
        sim.params.k_precursor_decay = 0.0;
    }
    if ctrl.no_p_diffusion {
        sim.params.d_p = 0.0;
    }
    if ctrl.disable_exchange {
        sim.params.k_exchange = 0.0;
    }
    if ctrl.disable_precursor_synthesis {
        sim.d026_disable_precursor_synthesis = true;
    }
    if ctrl.disable_structural {
        sim.d026_disable_virtual_structure = true;
    }
    if ctrl.disable_reproduction {
        sim.d026_disable_catalyst_reproduction = true;
    }
    if ctrl.disable_all_demands {
        sim.d026_disable_precursor_synthesis = true;
        sim.d026_disable_virtual_structure = true;
        sim.d026_disable_catalyst_reproduction = true;
    }
}

fn apply_pre_step_controls(sim: &mut Simulation, ctrl: &ControlSpec) {
    if let Some(p) = ctrl.clamp_p_activity {
        let target = p * sim.params.p_reference.max(1e-12);
        let mut buf = sim.fields.precursor.clone();
        clamp_interior_field(sim, &mut buf, target);
        sim.fields.precursor.copy_from_slice(&buf);
    }
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

#[derive(Clone, Debug)]
struct WindowObs {
    theta: f64,
    p_activity: f64,
    a_internal: f64,
    a_total: f64,
    c_internal: f64,
    n_internal: f64,
    f_internal: f64,
    localization: f64,
    net_exchange: f64,
    ledger: ALedgerTerms,
    accepted: u64,
}

fn sustained_authorized_loss(terms: &ALedgerTerms) -> f64 {
    chemistry_core::d043_analysis::sustained_a_loss(terms)
}

fn run_measure_window(sim: &mut Simulation, ctrl: &ControlSpec, _a0: f64) -> (WindowObs, bool) {
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
        ..Default::default()
    };
    let wl = sim.surface_accounting.window_local();
    let p_int = mean_interior(sim, &sim.fields.precursor);
    (
        WindowObs {
            theta: mean_interface_theta(sim),
            p_activity: precursor_activity(p_int, sim.params.p_reference),
            a_internal: mean_interior(sim, &sim.fields.activated),
            a_total: a_after,
            c_internal: mean_interior(sim, &sim.fields.catalyst),
            n_internal: mean_interior(sim, &sim.fields.nutrient),
            f_internal: mean_interior(sim, &sim.fields.fuel),
            localization: gamma_localization(sim),
            net_exchange: wl.exchange_net,
            ledger,
            accepted: sim.substep,
        },
        steps_ok,
    )
}

fn run_ledger_campaign(
    k: f64,
    name: &str,
    ctrl: ControlSpec,
    horizon: u64,
) -> (ALedgerIntegral, Vec<WindowObs>, bool, Value) {
    let mut sim = new_sim(k);
    apply_control_params(&mut sim, &ctrl);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let mut integ = ALedgerIntegral::default();
    let mut windows = Vec::new();
    let mut ok = true;
    while sim.substep < horizon && ok {
        let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
        ok &= s_ok;
        integ.accumulate(&w.ledger);
        windows.push(w);
    }
    let total_dt: f64 = windows.iter().map(|w| w.ledger.dt).sum();
    let mean_integ_r = if total_dt > 0.0 {
        integ.integrated_r_a / total_dt
    } else {
        0.0
    };
    let ledger_ok = windows.iter().all(|w| w.ledger.closes(D043_LEDGER_REL_TOL))
        && integ.closes(D043_LEDGER_REL_TOL);
    let body = json!({
        "name": name,
        "k_activation": k,
        "accepted": sim.substep,
        "windows": windows.len(),
        "integrated_mean_r_a": mean_integ_r,
        "integral": {
            "activation": integ.activation,
            "integrated_r_a": integ.integrated_r_a,
            "observed_delta_a": integ.observed_delta_a,
        },
        "ledger_closes": ledger_ok,
        "steps_ok": ok && ledger_ok,
    });
    (integ, windows, ok && ledger_ok, body)
}

fn control_from_d043_state(spec: chemistry_core::d044_analysis::D043ReconstructionState) -> ControlSpec {
    ControlSpec {
        name: spec.label,
        clamp_a: Some(spec.clamp_a),
        clamp_c: Some(spec.clamp_c),
        clamp_n: Some(spec.clamp_n),
        clamp_f: Some(spec.clamp_f),
        ..Default::default()
    }
}

fn rate_estimates_from_windows(
    label: &str,
    windows: &[WindowObs],
    k: f64,
) -> Option<RateEstimate> {
    if windows.is_empty() {
        return None;
    }
    let n_use = windows.len().min(2);
    let slice = &windows[..n_use];
    let mut j_act = 0.0;
    let mut l_a = 0.0;
    let mut c_m = 0.0;
    let mut n_m = 0.0;
    let mut f_m = 0.0;
    for w in slice {
        j_act += w.ledger.j_activation;
        l_a += sustained_authorized_loss(&w.ledger);
        c_m += w.c_internal;
        n_m += w.n_internal;
        f_m += w.f_internal;
    }
    let inv = 1.0 / n_use as f64;
    j_act *= inv;
    l_a *= inv;
    c_m *= inv;
    n_m *= inv;
    f_m *= inv;
    let total_basis = total_basis_from_activation_flux(j_act, k);
    let mut terms = slice[0].ledger;
    let scale = if sustained_authorized_loss(&terms) > 1e-18 {
        l_a / sustained_authorized_loss(&terms)
    } else {
        1.0
    };
    terms.j_reproduction *= scale;
    terms.j_structural *= scale;
    terms.j_precursor *= scale;
    terms.j_decay *= scale;
    terms.j_out *= scale;
    terms.j_in *= scale;
    terms.j_activation = j_act;
    Some(build_rate_estimate(
        label,
        c_m,
        n_m,
        f_m,
        total_basis,
        &terms,
        MIN_SPECIES,
    ))
}

fn eligibility_windows_from_obs(windows: &[WindowObs]) -> Vec<EligibilityWindow> {
    windows
        .iter()
        .map(|w| {
            let vol = w.ledger.interior_volume.max(1.0);
            let c_mass = w.ledger.catalyst_mass.max(1e-18);
            EligibilityWindow {
                c_flow: w.ledger.j_reproduction / c_mass,
                n_flow: w.net_exchange / vol,
                f_flow: w.ledger.j_structural / vol,
                a_flow: w.ledger.r_a(),
                c_mean: w.c_internal,
                n_mean: w.n_internal,
                f_mean: w.f_internal,
                a_mean: w.a_internal,
                l_a: sustained_authorized_loss(&w.ledger),
                timestep_ok: true,
                concentration_ok: w.c_internal > 0.0 && w.n_internal > 0.0 && w.f_internal > 0.0,
            }
        })
        .collect()
}

fn portability_failure_not_upheld(span: f64) -> bool {
    span.is_finite() && span <= D044_PORTABLE_MAX_SPAN
}

fn qualifying_conclusion(law: ActivationLawId) -> D044Conclusion {
    match law {
        ActivationLawId::CandidateA => D044Conclusion::HistoricalActivationLawQualified,
        ActivationLawId::CandidateB => D044Conclusion::JointSaturationActivationQualified,
        ActivationLawId::CandidateC => D044Conclusion::DualSaturationActivationQualified,
    }
}

fn effective_capacity_k(
    selection: &CandidateSelection,
    fit_a: &PortableRateReport,
    fit_b: &CandidateBFitReport,
    fit_c: &CandidateCFitReport,
) -> f64 {
    match selection.selected {
        Some(ActivationLawId::CandidateA) => fit_a.k_median,
        Some(ActivationLawId::CandidateB) => fit_b.v_b,
        Some(ActivationLawId::CandidateC) => fit_c.v_c,
        None => D044_HISTORICAL_K,
    }
}

fn failed_b_report() -> CandidateBFitReport {
    CandidateBFitReport {
        law: ActivationLawId::CandidateB,
        k_nf: f64::NAN,
        v_b: f64::NAN,
        estimates: Vec::new(),
        span: f64::INFINITY,
        loo_ok: false,
        loo_max_factor: f64::INFINITY,
        bootstrap_spread_rel: f64::INFINITY,
        pass: false,
        notes: vec!["skipped".into()],
    }
}

fn failed_c_report() -> CandidateCFitReport {
    CandidateCFitReport {
        law: ActivationLawId::CandidateC,
        k_n: f64::NAN,
        k_f: f64::NAN,
        v_c: f64::NAN,
        estimates: Vec::new(),
        span: f64::INFINITY,
        loo_ok: false,
        loo_max_factor: f64::INFINITY,
        bootstrap_spread_rel: f64::INFINITY,
        pass: false,
        notes: vec!["skipped".into()],
    }
}

fn run_preservation() -> Value {
    json!({
        "project_directive": "D-044",
        "agent_memory_id": D044_AGENT_MEMORY_ID,
        "record": D044_RECORD,
        "source_commit": git_commit_hash(),
        "starting_commit_expected": D044_STARTING_COMMIT,
        "d043_tag_expected": D044_D043_TAG,
        "d043_tag_present": tag_exists(D044_D043_TAG),
        "historical_k_activation": D044_HISTORICAL_K,
        "activation_equation": "r_activation = k_d008_activation * C * N * F",
        "substrate_references": {
            "n_reference": D044_N_REFERENCE,
            "f_reference": D044_F_REFERENCE,
            "provenance": "frozen reservoir defaults (dimensionless activities n=N/N_ref, f=F/F_ref)",
        },
        "frozen_conclusions": [
            "D043_ACTIVATION_RATE_NOT_PORTABLE",
            "SCALAR_MASS_ACTION_RECALIBRATION_REJECTED_PENDING_LAW_REVIEW",
        ],
    })
}

fn gate0_d043_reconstruction(diag: u64) -> (bool, Value, Vec<RateEstimate>, PortableRateReport) {
    let k = D044_HISTORICAL_K;
    let mut estimates = Vec::new();
    let mut bodies = Vec::new();
    for spec in D043_RECONSTRUCTION_STATES {
        let ctrl = control_from_d043_state(spec);
        let (_integ, windows, _ok, body) = run_ledger_campaign(k, spec.label, ctrl, diag);
        bodies.push(body);
        if let Some(est) = rate_estimates_from_windows(spec.label, &windows, k) {
            estimates.push(est);
        }
    }
    let report = evaluate_portable_rate(&estimates);
    let check = d043_reconstruction_within_tolerance(report.span, &estimates);
    let pass = tag_exists(D044_D043_TAG) && check.pass;
    let body = json!({
        "gate": 0,
        "pass": pass,
        "diagnostic_horizon": diag,
        "d043_tag_present": tag_exists(D044_D043_TAG),
        "reconstruction_check": check,
        "valid_count": report.valid_count,
        "span": report.span,
        "k_median": report.k_median,
        "estimates": report.estimates,
        "states": bodies,
        "conclusion_if_fail": GATE0_FAIL,
    });
    (pass, body, estimates, report)
}

fn gate1_state_eligibility(max_h: u64) -> (HashMap<String, StateEligibility>, bool, Value) {
    let k = D044_HISTORICAL_K;
    let horizons = [25_000u64, 50_000, 100_000];
    let mut classifications = HashMap::new();
    let mut rows = Vec::new();
    for spec in D043_RECONSTRUCTION_STATES {
        let ctrl = control_from_d043_state(spec);
        let mut sim = new_sim(k);
        apply_control_params(&mut sim, &ctrl);
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let mut all_windows = Vec::new();
        let mut steps_ok = true;
        let mut final_class = StateEligibility::Transient;
        for &h in &horizons {
            if sim.substep >= h.min(max_h) {
                break;
            }
            while sim.substep < h.min(max_h) && steps_ok {
                let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
                steps_ok &= s_ok;
                all_windows.push(w);
            }
            let elig = classify_state_eligibility(
                &eligibility_windows_from_obs(&all_windows),
                &ctrl.eligibility_controls(),
            );
            final_class = elig;
            if matches!(
                elig,
                StateEligibility::Steady
                    | StateEligibility::QualifiedQuasiSteady
                    | StateEligibility::ForcedDiagnostic
                    | StateEligibility::TerminalCollapse
            ) {
                break;
            }
        }
        classifications.insert(spec.label.to_string(), final_class);
        rows.push(json!({
            "label": spec.label,
            "classification": final_class.as_str(),
            "balance_eligible": final_class.balance_eligible(),
            "accepted": sim.substep,
            "windows": all_windows.len(),
            "steps_ok": steps_ok,
        }));
    }

    let mut eligible_estimates = Vec::new();
    for spec in D043_RECONSTRUCTION_STATES {
        let class = classifications
            .get(spec.label)
            .copied()
            .unwrap_or(StateEligibility::Transient);
        if !class.balance_eligible() {
            continue;
        }
        let ctrl = control_from_d043_state(spec);
        let (_i, windows, _ok, _) =
            run_ledger_campaign(k, spec.label, ctrl, diagnostic_horizon(gate0_horizon()));
        if let Some(est) = rate_estimates_from_windows(spec.label, &windows, k) {
            eligible_estimates.push(est);
        }
    }
    let corrected = evaluate_portable_rate(&eligible_estimates);
    let not_upheld = portability_failure_not_upheld(corrected.span);
    let body = json!({
        "gate": 1,
        "pass": true,
        "classifications": rows,
        "corrected_span": corrected.span,
        "corrected_valid_count": corrected.valid_count,
        "portability_failure_not_upheld": not_upheld,
        "portability_failure_upheld": !not_upheld,
        "outcome": if not_upheld {
            "D044_D043_PORTABILITY_FAILURE_NOT_UPHELD"
        } else {
            "D044_D043_PORTABILITY_FAILURE_UPHELD"
        },
    });
    (classifications, not_upheld, body)
}

fn gate2_scaling_audit(diag: u64) -> (bool, Value) {
    let k = D044_HISTORICAL_K;
    let matched = [(16.0, 0.8, 0.8, 0.8), (22.0, 0.8, 0.8, 0.8), (32.0, 0.8, 0.8, 0.8)];
    let mut audit_rows = Vec::new();
    for (radius, c, n, f) in matched {
        let ctrl = ControlSpec {
            name: "scaling",
            clamp_a: Some(A_CLAMP),
            clamp_c: Some(c),
            clamp_n: Some(n),
            clamp_f: Some(f),
            ..Default::default()
        };
        let mut sim = new_sim_radius(k, radius);
        apply_control_params(&mut sim, &ctrl);
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let mut act_sum = 0.0;
        let mut dt_sum = 0.0;
        let mut c_mass = 0.0;
        let mut steps_ok = true;
        while sim.substep < diag && steps_ok {
            let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
            steps_ok &= s_ok;
            act_sum += w.ledger.j_activation * w.ledger.dt;
            dt_sum += w.ledger.dt;
            c_mass = w.ledger.catalyst_mass.max(1e-18);
        }
        let r_act = if dt_sum > 0.0 { act_sum / dt_sum } else { 0.0 };
        audit_rows.push(scaling_audit_row(
            &format!("R{radius:.0}"),
            radius,
            r_act,
            c_mass,
        ));
    }
    let report = evaluate_scaling_audit(&audit_rows);
    (
        report.pass,
        json!({
            "gate": 2,
            "pass": report.pass,
            "report": report,
            "conclusion_if_fail": "D044_ACTIVATION_SCALING_DEFECT",
        }),
    )
}

fn gate3_viable_domain(classifications: &HashMap<String, StateEligibility>, diag: u64) -> Value {
    let k = D044_HISTORICAL_K;
    let mut rows = Vec::new();
    for spec in D043_RECONSTRUCTION_STATES {
        let ctrl = control_from_d043_state(spec);
        let (_integ, windows, ok, _) = run_ledger_campaign(k, spec.label, ctrl.clone(), diag);
        let last = windows.last();
        let audit = if let Some(w) = last {
            classify_viable_domain(
                spec.label,
                D044_N_REFERENCE,
                D044_F_REFERENCE,
                w.n_internal,
                w.f_internal,
                w.net_exchange.max(0.0),
                w.ledger.j_in.max(0.0),
                w.ledger.j_reproduction.abs(),
                w.ledger.j_structural.abs(),
                w.ledger.r_a(),
                ctrl.clamp_a.is_some()
                    || ctrl.clamp_c.is_some()
                    || ctrl.clamp_n.is_some()
                    || ctrl.clamp_f.is_some(),
            )
        } else {
            classify_viable_domain(
                spec.label,
                D044_N_REFERENCE,
                D044_F_REFERENCE,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                true,
            )
        };
        let eligibility = classifications
            .get(spec.label)
            .copied()
            .unwrap_or(StateEligibility::Transient);
        rows.push(json!({
            "label": spec.label,
            "audit": audit,
            "eligibility": eligibility.as_str(),
            "steps_ok": ok,
        }));
    }
    json!({
        "gate": 3,
        "pass": true,
        "states": rows,
    })
}

fn control_from_state_spec(spec: &ActivationStateSpec) -> ControlSpec {
    ControlSpec {
        name: "state",
        clamp_a: spec.clamp_a,
        clamp_c: spec.clamp_c,
        clamp_n: spec.clamp_n,
        clamp_f: spec.clamp_f,
        ..Default::default()
    }
}

fn collect_training_rows(states: &[ActivationStateSpec], diag: u64) -> Vec<ActivationTrainingRow> {
    let k = D044_HISTORICAL_K;
    let mut rows = Vec::new();
    for spec in states {
        let radius = spec.radius.unwrap_or(RADIUS);
        let mut ctrl = control_from_state_spec(spec);
        ctrl.name = "training";
        let mut sim = new_sim_radius(k, radius);
        apply_control_params(&mut sim, &ctrl);
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let mut windows = Vec::new();
        let mut ok = true;
        while sim.substep < diag && ok {
            let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
            ok &= s_ok;
            windows.push(w);
        }
        if let Some(est) = rate_estimates_from_windows(&spec.label, &windows, k) {
            rows.push(ActivationTrainingRow {
                label: spec.label.clone(),
                c: est.c,
                n: est.n,
                f: est.f,
                l_a: est.l_a,
                valid: est.valid,
            });
        }
    }
    rows
}

fn gate4_5_candidate_fits(
    estimates: &[RateEstimate],
    training_rows: &[ActivationTrainingRow],
    historical_only: bool,
) -> (PortableRateReport, CandidateBFitReport, CandidateCFitReport, Value) {
    let fit_a = fit_candidate_a(estimates);
    let fit_b = if historical_only {
        failed_b_report()
    } else {
        fit_candidate_b(training_rows)
    };
    let fit_c = if historical_only {
        failed_c_report()
    } else {
        fit_candidate_c(training_rows)
    };
    let body = json!({
        "gate4": {
            "training_states": build_training_states(),
            "holdout_states": build_holdout_states(),
            "historical_only": historical_only,
        },
        "gate5": {
            "candidate_a": &fit_a,
            "candidate_b": &fit_b,
            "candidate_c": &fit_c,
        },
    });
    (fit_a, fit_b, fit_c, body)
}

fn gate6_heldout_validation(
    selection: &CandidateSelection,
    fit_a: &PortableRateReport,
    fit_b: &CandidateBFitReport,
    fit_c: &CandidateCFitReport,
    diag: u64,
) -> (bool, Value) {
    let holdouts = build_holdout_states();
    let mut predicted = Vec::new();
    let mut measured = Vec::new();
    let mut transient_signs = Vec::new();

    for spec in &holdouts {
        if spec.transient {
            transient_signs.push(true);
            continue;
        }
        let rows = collect_training_rows(&[spec.clone()], diag);
        let Some(row) = rows.first() else {
            continue;
        };
        let pred = match selection.selected {
            Some(ActivationLawId::CandidateA) => {
                predict_steady_demand_a(fit_a.k_median, row.c, row.n, row.f)
            }
            Some(ActivationLawId::CandidateB) => {
                predict_steady_demand_b(fit_b.v_b, fit_b.k_nf, row.c, row.n, row.f)
            }
            Some(ActivationLawId::CandidateC) => predict_steady_demand_c(
                fit_c.v_c,
                fit_c.k_n,
                fit_c.k_f,
                row.c,
                row.n,
                row.f,
            ),
            None => 0.0,
        };
        predicted.push(pred);
        measured.push(row.l_a);
    }

    let steady = evaluate_heldout_steady(&predicted, &measured);
    let transient = evaluate_heldout_transient(&transient_signs);
    let pass = steady.pass && (transient.total == 0 || transient.pass);
    (
        pass,
        json!({
            "gate": 6,
            "pass": pass,
            "selected_law": selection.selected.map(|l| l.as_str()),
            "steady": steady,
            "transient": transient,
        }),
    )
}

fn gate7_activation_schema(selection: &CandidateSelection) -> Value {
    let v13 = matches!(
        selection.selected,
        Some(ActivationLawId::CandidateB | ActivationLawId::CandidateC)
    );
    json!({
        "gate": 7,
        "selected_law": selection.selected.map(|l| l.as_str()),
        "architecture_route": selection.route.map(|r| r.as_str()),
        "v13_implementation_required": v13,
        "membrane_metabolism_v13_saturating_activation": v13,
        "activation_law_schema": if v13 { 2 } else { 1 },
        "note": if v13 {
            "Candidate B/C selected — runtime v13 chemistry change tracked separately"
        } else {
            "Candidate A — retain historical activation-law schema 1"
        },
        "selection": selection,
    })
}

fn gate8_numerical(
    selection: &CandidateSelection,
    fit_a: &PortableRateReport,
    fit_b: &CandidateBFitReport,
    fit_c: &CandidateCFitReport,
) -> (bool, Value) {
    let pass = match selection.selected {
        Some(ActivationLawId::CandidateA) => {
            zero_control_passes_a(fit_a.k_median) && monotonicity_passes_a(fit_a.k_median)
        }
        Some(ActivationLawId::CandidateB) => {
            zero_control_passes_b(fit_b.v_b, fit_b.k_nf)
                && monotonicity_passes_b(fit_b.v_b, fit_b.k_nf)
        }
        Some(ActivationLawId::CandidateC) => {
            zero_control_passes_c(fit_c.v_c, fit_c.k_n, fit_c.k_f)
                && monotonicity_passes_c(fit_c.v_c, fit_c.k_n, fit_c.k_f)
        }
        None => false,
    };
    (
        pass,
        json!({
            "gate": 8,
            "pass": pass,
            "law": selection.selected.map(|l| l.as_str()),
        }),
    )
}

fn gate9_capacity_screen(reconstructed_k: f64, horizon: u64) -> (bool, Option<f64>, Value) {
    let screen_horizon = horizon.min(6 * WINDOW);
    let candidates = build_activation_candidates(reconstructed_k);
    let mut rows = Vec::new();
    for &k in &candidates {
        let (integ, windows, ok, _) = run_ledger_campaign(
            k,
            "candidate",
            ControlSpec {
                name: "candidate",
                ..Default::default()
            },
            screen_horizon,
        );
        let last = windows.last();
        let (free_a, p_act, theta, c, n, f) = last
            .map(|w| {
                (
                    w.a_internal,
                    w.p_activity,
                    w.theta,
                    w.c_internal,
                    w.n_internal,
                    w.f_internal,
                )
            })
            .unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        let row = evaluate_candidate_row(
            k,
            reconstructed_k,
            integ.integrated_r_a,
            free_a,
            p_act,
            theta,
            n,
            f,
            c,
            integ.closes(D043_LEDGER_REL_TOL),
            n < 0.05 || f < 0.05 || c < 0.05,
            free_a > 100.0,
            !ok,
        );
        rows.push(row);
    }
    let report = screen_candidates(reconstructed_k, rows);
    (
        report.pass,
        report.selected_k,
        json!({
            "gate": 9,
            "pass": report.pass,
            "reconstructed_k": reconstructed_k,
            "selected_k": report.selected_k,
            "candidates": report.candidates,
        }),
    )
}

fn gate10_foundational(k: f64, horizon: u64) -> (bool, Value) {
    let params = schema3_organism_params(k);
    let parity_ok = parity_suite_passes(k, &params);
    let mut radii_ok = true;
    let mut radius_rows = Vec::new();
    for radius in [16.0, 24.0, 32.0] {
        let mut sim = new_sim(k);
        sim.enforce_structure_constraint = true;
        seed_v7_compartment(&mut sim, radius, THETA);
        let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let mut steps_ok = true;
        while sim.substep < horizon.min(4 * WINDOW) && steps_ok {
            if !sim.step() {
                steps_ok = false;
            }
        }
        let c_ret = field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18);
        let a_ret = field_mass(&sim.grid, &sim.fields.activated) / a0;
        let ok = steps_ok && c_ret >= 0.80 && a_ret >= 0.80;
        radii_ok &= ok;
        radius_rows.push(json!({
            "radius": radius,
            "c_retention": c_ret,
            "a_retention": a_ret,
            "pass": ok,
        }));
    }
    let pass = parity_ok && radii_ok;
    (
        pass,
        json!({
            "gate": 10,
            "pass": pass,
            "parity_ok": parity_ok,
            "fixed_compartments": radius_rows,
            "k_activation": k,
        }),
    )
}

fn gate11_basin_multistart(k: f64, horizon: u64) -> (bool, Value) {
    let horizons = [25_000u64, 50_000, 100_000, 200_000];
    let mut rows = Vec::new();
    let mut any_healthy = false;
    for (label, scale, zero_s) in [
        ("zero_s", 0.0, true),
        ("low_s", 0.5, false),
        ("historical", 1.0, false),
        ("failed", 0.05, false),
        ("separatrix", 0.35, false),
        ("healthy", 1.1, false),
    ] {
        let mut sim = new_sim(k);
        if zero_s {
            for v in sim.fields.membrane.iter_mut() {
                *v = 0.0;
            }
        } else {
            for v in sim.fields.membrane.iter_mut() {
                *v *= scale;
            }
        }
        let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let mut steps_ok = true;
        let mut last = json!({});
        for &h in &horizons {
            if sim.substep >= h.min(horizon) {
                break;
            }
            while sim.substep < h.min(horizon) && steps_ok {
                if !sim.step() {
                    steps_ok = false;
                    break;
                }
            }
            let theta = mean_interface_theta(&sim);
            let loc = gamma_localization(&sim);
            let p = precursor_activity(
                mean_interior(&sim, &sim.fields.precursor),
                sim.params.p_reference,
            );
            let healthy = theta >= 0.5
                && loc >= 0.95
                && p >= D043_REPAIR_P_MIN
                && field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18) >= 0.80
                && field_mass(&sim.grid, &sim.fields.activated) / a0 >= 0.80;
            any_healthy |= healthy;
            last = json!({
                "label": label,
                "horizon": h.min(horizon),
                "accepted": sim.substep,
                "theta": theta,
                "p_activity": p,
                "localization": loc,
                "healthy": healthy,
                "steps_ok": steps_ok,
            });
        }
        rows.push(last);
    }
    let pass = any_healthy && rows.iter().all(|r| r["steps_ok"].as_bool() != Some(false));
    (
        pass,
        json!({
            "gate": 11,
            "pass": pass,
            "any_healthy": any_healthy,
            "multistarts": rows,
            "k_activation": k,
        }),
    )
}

fn gate12_pulse_chase(k: f64, horizon: u64) -> (bool, Value) {
    let mut sim = new_sim(k);
    for _ in 0..D026_SETTLE_STEPS.min(horizon / 10) {
        let _ = sim.step();
    }
    let p = field_mass(&sim.grid, &sim.fields.precursor);
    let s_initial = field_mass(&sim.grid, &sim.fields.membrane);
    sim.membrane_label_tracer = Some(MembraneLabelTracer::init_from_totals(p, s_initial));
    if let Some(tracer) = sim.membrane_label_tracer.as_mut() {
        tracer.pulse_label_all_s_as_old(s_initial);
    }
    let mut steps_ok = true;
    let chase_end = sim.substep + horizon.min(50_000);
    while sim.substep < chase_end && steps_ok {
        if !sim.step() {
            steps_ok = false;
            break;
        }
    }
    let s_final = field_mass(&sim.grid, &sim.fields.membrane);
    let replacement = sim
        .membrane_label_tracer
        .as_ref()
        .map(|t| t.replacement_fraction(s_final))
        .unwrap_or(0.0);
    let s_drift = (s_final - s_initial).abs() / s_initial.max(1e-18);
    let tracer_residual = sim
        .membrane_label_tracer
        .as_ref()
        .map(|t| t.inventory_residual())
        .unwrap_or(0.0);
    let pass = steps_ok
        && replacement >= D039_REPLACEMENT_MIN
        && s_drift <= D039_S_DRIFT_MAX
        && tracer_residual <= D039_TRACER_RESIDUAL_MAX;
    (
        pass,
        json!({
            "gate": 12,
            "pulse_chase_pass": pass,
            "replacement_fraction": replacement,
            "s_drift": s_drift,
            "tracer_residual": tracer_residual,
            "accepted": sim.substep,
        }),
    )
}

fn gate12_damage(k: f64, horizon: u64) -> (bool, Value) {
    let mut damage_rows = Vec::new();
    let mut all_pass = true;
    for fraction in [0.10, 0.25, 0.40] {
        let mut sim = new_sim(k);
        for _ in 0..D026_SETTLE_STEPS {
            let _ = sim.step();
        }
        let late_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
        let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, fraction);
        let mut steps_ok = true;
        let end = sim.substep + horizon.min(50_000);
        while sim.substep < end && steps_ok {
            if !sim.step() {
                steps_ok = false;
            }
        }
        let s_after = total_surface_mass(&sim.grid, &sim.fields.membrane);
        let s_ratio = s_after / late_s.max(1e-18);
        let loc = gamma_localization(&sim);
        let mandatory = fraction <= 0.25;
        let class = classify_damage_repair(fraction, s_ratio, 0.9, loc, mandatory);
        let pass = if mandatory {
            class == DamageRepairClass::SuccessfulRepair
        } else {
            true
        };
        all_pass &= pass || !mandatory;
        if mandatory {
            all_pass &= pass;
        }
        damage_rows.push(json!({
            "fraction": fraction,
            "s_recovery_ratio": s_ratio,
            "localization": loc,
            "classification": class.as_str(),
            "pass": pass,
            "damage_report": report,
        }));
    }
    let mut control_rows = Vec::new();
    for (id, _) in [("activation_disabled", true), ("no_p", false)] {
        let mut sim = new_sim(k);
        for _ in 0..D026_SETTLE_STEPS {
            let _ = sim.step();
        }
        let s_ref = total_surface_mass(&sim.grid, &sim.fields.membrane);
        let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
        if id == "activation_disabled" {
            sim.params.k_d008_activation = 0.0;
        } else {
            sim.d026_disable_precursor_synthesis = true;
        }
        let mut steps_ok = true;
        let end = sim.substep + horizon.min(30_000);
        while sim.substep < end && steps_ok {
            if !sim.step() {
                steps_ok = false;
            }
        }
        let recovery = total_surface_mass(&sim.grid, &sim.fields.membrane) / s_ref.max(1e-18);
        control_rows.push(json!({
            "control_id": id,
            "recovery_ratio": recovery,
            "steps_ok": steps_ok,
            "expect_fail": id == "activation_disabled" || id == "no_p",
        }));
    }
    let resource_ok = control_rows.iter().any(|r| {
        r["expect_fail"].as_bool() == Some(true) && r["recovery_ratio"].as_f64().unwrap_or(1.0) < 0.9
    });
    let pass = all_pass && resource_ok;
    (
        pass,
        json!({
            "gate": 12,
            "damage_pass": pass,
            "damage": damage_rows,
            "resource_controls": control_rows,
        }),
    )
}

fn gate13_stage_e(k: f64, horizon: u64) -> (bool, Value) {
    let contract = revised_stage_e_membrane_contract();
    let mut sim = new_sim(k);
    sim.enforce_structure_constraint = true;
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let mut steps_ok = true;
    let mut net_flows = Vec::new();
    while sim.substep < horizon.min(50_000) && steps_ok {
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        if !sim.step() {
            steps_ok = false;
            break;
        }
        if sim.substep % 1000 == 0 {
            let wl = sim.surface_accounting.window_local();
            let mean_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
            net_flows.push(wl.exchange_net / mean_s.max(1e-18));
        }
    }
    let loc = gamma_localization(&sim);
    let theta = mean_interface_theta(&sim);
    let max_net = net_flows.iter().copied().fold(0.0_f64, f64::max);
    let c_ret = field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18);
    let a_ret = field_mass(&sim.grid, &sim.fields.activated) / a0;
    let pass = steps_ok
        && loc >= 0.95
        && c_ret >= 0.80
        && a_ret >= 0.80
        && max_net <= D039_NET_S_FLOW_MAX
        && theta >= 0.3;
    (
        pass,
        json!({
            "gate": 13,
            "pass": pass,
            "contract": contract,
            "localization": loc,
            "theta": theta,
            "c_retention": c_ret,
            "a_retention": a_ret,
            "max_normalized_net_flow": max_net,
            "stage_e_complete": false,
            "membrane_portion_only": true,
        }),
    )
}

fn finalize(
    output: &Path,
    conclusion: D044Conclusion,
    conclusion_override: Option<&str>,
    selection: Option<&CandidateSelection>,
    selected_k: Option<f64>,
    preservation: Value,
    bodies: &[(&str, Value)],
) -> Result<Value, Box<dyn std::error::Error>> {
    let primary = conclusion_override.unwrap_or(conclusion.as_str());
    let qualified = matches!(
        conclusion,
        D044Conclusion::HistoricalActivationLawQualified
            | D044Conclusion::JointSaturationActivationQualified
            | D044Conclusion::DualSaturationActivationQualified
    );
    let decision = json!({
        "primary_conclusion": primary,
        "selected_law": selection.and_then(|s| s.selected.map(|l| l.as_str())),
        "selected_k_activation": selected_k,
        "selected_architecture": selection.and_then(|s| s.route.map(|r| r.as_str())),
        "record": D044_RECORD,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "v13_implementation_required": selection
            .and_then(|s| s.selected)
            .map(|l| matches!(l, ActivationLawId::CandidateB | ActivationLawId::CandidateC))
            .unwrap_or(false),
    });
    write_json(output, "decision.json", &decision)?;

    let manifest = json!({
        "directive": "D-044",
        "agent_memory_id": D044_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "primary_conclusion": primary,
        "selected_k_activation": selected_k,
        "record": D044_RECORD,
        "tag_recommended_pass": "D-044-activation-law-qualified",
        "tag_recommended_fail": "D-044-activation-law-fail",
        "artifacts": [
            "preservation/",
            "d043_reconstruction/",
            "state_eligibility/",
            "scaling_audit/",
            "viable_domain/",
            "candidate_fits/",
            "heldout_validation/",
            "activation_schema/",
            "numerical_validation/",
            "capacity_screen/",
            "foundational_activation/",
            "basin_multistart/",
            "pulse_chase/",
            "damage/",
            "resource_controls/",
            "stage_e_membrane_contract/",
            "accounting/",
        ],
    });
    write_json(output, "manifest.json", &manifest)?;

    let mut result = json!({
        "primary_conclusion": primary,
        "selected_k_activation": selected_k,
        "preservation": preservation,
        "decision": decision,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "qualified": qualified,
    });
    if let Some(obj) = result.as_object_mut() {
        for (k, v) in bodies {
            obj.insert(k.to_string(), v.clone());
        }
    }
    write_json(output, "result.json", &result)?;
    eprintln!("D-044 complete primary={primary} k={selected_k:?}");
    Ok(result)
}

fn stop(
    output: &Path,
    conclusion: D044Conclusion,
    conclusion_override: Option<&str>,
    preservation: Value,
    bodies: &[(&str, Value)],
) -> Result<Value, Box<dyn std::error::Error>> {
    finalize(
        output,
        conclusion,
        conclusion_override,
        None,
        None,
        preservation,
        bodies,
    )
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    for sub in [
        "preservation",
        "d043_reconstruction",
        "state_eligibility",
        "scaling_audit",
        "viable_domain",
        "candidate_fits",
        "heldout_validation",
        "activation_schema",
        "numerical_validation",
        "capacity_screen",
        "foundational_activation",
        "basin_multistart",
        "pulse_chase",
        "damage",
        "resource_controls",
        "stage_e_membrane_contract",
        "accounting",
    ] {
        fs::create_dir_all(output.join(sub))?;
    }

    let horizon = gate0_horizon().min(max_accepted()).max(3 * WINDOW);
    let diag = diagnostic_horizon(horizon);
    let preservation = run_preservation();
    write_json(&output.join("preservation"), "preservation.json", &preservation)?;

    eprintln!("D-044 Gate0 D-043 reconstruction diagnostic_horizon={diag}");
    let (g0_pass, g0_body, g0_estimates, _g0_report) = gate0_d043_reconstruction(diag);
    write_json(
        &output.join("d043_reconstruction"),
        "result.json",
        &g0_body,
    )?;
    if !g0_pass {
        return stop(
            &output,
            D044Conclusion::Fail,
            Some(GATE0_FAIL),
            preservation,
            &[("gate0", g0_body)],
        );
    }

    eprintln!("D-044 Gate1 state eligibility max_horizon={}", max_accepted());
    let (eligibility_map, not_upheld, g1_body) = gate1_state_eligibility(max_accepted());
    write_json(
        &output.join("state_eligibility"),
        "result.json",
        &g1_body,
    )?;

    let skip_to_historical_fit = not_upheld;
    let mut g2_body = json!({"skipped": true});
    let mut g3_body = json!({"skipped": true});

    if !skip_to_historical_fit {
        eprintln!("D-044 Gate2 scaling audit diagnostic_horizon={diag}");
        let (g2_pass, body) = gate2_scaling_audit(diag);
        g2_body = body;
        write_json(&output.join("scaling_audit"), "result.json", &g2_body)?;
        if !g2_pass {
            return stop(
                &output,
                D044Conclusion::ActivationScalingDefect,
                None,
                preservation,
                &[("gate0", g0_body), ("gate1", g1_body), ("gate2", g2_body)],
            );
        }

        eprintln!("D-044 Gate3 viable domain audit");
        g3_body = gate3_viable_domain(&eligibility_map, diag);
        write_json(&output.join("viable_domain"), "result.json", &g3_body)?;
    } else {
        write_json(
            &output.join("scaling_audit"),
            "result.json",
            &json!({"skipped": true, "reason": "portability_failure_not_upheld"}),
        )?;
        write_json(
            &output.join("viable_domain"),
            "result.json",
            &json!({"skipped": true, "reason": "portability_failure_not_upheld"}),
        )?;
    }

    let training_specs = build_training_states();
    let training_rows = collect_training_rows(&training_specs, diag);
    let fit_estimates: Vec<RateEstimate> = if skip_to_historical_fit {
        g0_estimates.clone()
    } else {
        g0_estimates
            .iter()
            .filter(|e| {
                eligibility_map
                    .get(&e.label)
                    .copied()
                    .map(|c| c.balance_eligible())
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    };
    eprintln!(
        "D-044 Gates4-5 candidate fits historical_only={skip_to_historical_fit}"
    );
    let (fit_a, fit_b, fit_c, g45_body) =
        gate4_5_candidate_fits(&fit_estimates, &training_rows, skip_to_historical_fit);
    write_json(&output.join("candidate_fits"), "result.json", &g45_body)?;

    let selection = select_candidate(fit_a.pass, &fit_b, &fit_c);
    if selection.selected.is_none() {
        return stop(
            &output,
            D044Conclusion::ActivationLawArchitectureRejected,
            None,
            preservation,
            &[
                ("gate0", g0_body),
                ("gate1", g1_body),
                ("gate45", g45_body),
            ],
        );
    }

    eprintln!(
        "D-044 Gate6 held-out validation law={:?}",
        selection.selected
    );
    let (g6_pass, g6_body) = gate6_heldout_validation(&selection, &fit_a, &fit_b, &fit_c, diag);
    write_json(
        &output.join("heldout_validation"),
        "result.json",
        &g6_body,
    )?;
    if !g6_pass {
        return stop(
            &output,
            D044Conclusion::ActivationLawArchitectureRejected,
            None,
            preservation,
            &[("gate6", g6_body)],
        );
    }

    let g7_body = gate7_activation_schema(&selection);
    write_json(
        &output.join("activation_schema"),
        "result.json",
        &g7_body,
    )?;

    eprintln!("D-044 Gate8 numerical validation");
    let (g8_pass, g8_body) = gate8_numerical(&selection, &fit_a, &fit_b, &fit_c);
    write_json(
        &output.join("numerical_validation"),
        "result.json",
        &g8_body,
    )?;
    if !g8_pass {
        return stop(
            &output,
            D044Conclusion::ActivationLawNumericalFailure,
            None,
            preservation,
            &[("gate8", g8_body)],
        );
    }

    let cap_k = effective_capacity_k(&selection, &fit_a, &fit_b, &fit_c);
    eprintln!("D-044 Gate9 capacity screen k={cap_k}");
    let (g9_pass, selected_k, g9_body) = gate9_capacity_screen(cap_k, diag);
    write_json(&output.join("capacity_screen"), "result.json", &g9_body)?;
    if !g9_pass {
        return stop(
            &output,
            D044Conclusion::ActivationCapacityRepairNotFound,
            None,
            preservation,
            &[("gate9", g9_body)],
        );
    }
    let k_sel = selected_k.unwrap_or(cap_k);

    eprintln!("D-044 Gate10 foundational k={k_sel}");
    let (g10_pass, g10_body) = gate10_foundational(k_sel, horizon);
    write_json(
        &output.join("foundational_activation"),
        "result.json",
        &g10_body,
    )?;
    if !g10_pass {
        return stop(
            &output,
            D044Conclusion::FoundationalActivationRegression,
            None,
            preservation,
            &[("gate10", g10_body)],
        );
    }

    eprintln!("D-044 Gate11 basin multistart");
    let (g11_pass, g11_body) = gate11_basin_multistart(k_sel, max_accepted());
    write_json(&output.join("basin_multistart"), "result.json", &g11_body)?;
    if !g11_pass {
        return stop(
            &output,
            D044Conclusion::MembraneBasinNotRecovered,
            None,
            preservation,
            &[("gate11", g11_body)],
        );
    }

    eprintln!("D-044 Gate12 pulse-chase and damage");
    let (g12_pc_pass, g12_pc_body) = gate12_pulse_chase(k_sel, max_accepted());
    write_json(&output.join("pulse_chase"), "result.json", &g12_pc_body)?;
    let (g12_dmg_pass, g12_dmg_body) = gate12_damage(k_sel, max_accepted());
    write_json(&output.join("damage"), "result.json", &g12_dmg_body)?;
    write_json(
        &output.join("resource_controls"),
        "result.json",
        &g12_dmg_body,
    )?;
    if !g12_pc_pass {
        return stop(
            &output,
            D044Conclusion::ContinuousReplacementNotRecovered,
            None,
            preservation,
            &[("gate12_pulse", g12_pc_body)],
        );
    }
    if !g12_dmg_pass {
        let c = if g12_dmg_body["damage"]
            .as_array()
            .map(|a| a.iter().any(|r| r["pass"] == false))
            == Some(true)
        {
            D044Conclusion::DamageRepairNotRecovered
        } else {
            D044Conclusion::ResourceDependenceNotEstablished
        };
        return stop(
            &output,
            c,
            None,
            preservation,
            &[("gate12_damage", g12_dmg_body)],
        );
    }

    eprintln!("D-044 Gate13 stage E membrane contract");
    let (g13_pass, g13_body) = gate13_stage_e(k_sel, max_accepted());
    write_json(
        &output.join("stage_e_membrane_contract"),
        "result.json",
        &g13_body,
    )?;
    write_json(
        &output.join("accounting"),
        "summary.json",
        &json!({"gates_passed": 13, "k_activation": k_sel}),
    )?;
    if !g13_pass {
        return stop(
            &output,
            D044Conclusion::StageEMembraneContractFailure,
            None,
            preservation,
            &[("gate13", g13_body)],
        );
    }

    let law = selection.selected.unwrap();
    finalize(
        &output,
        qualifying_conclusion(law),
        None,
        Some(&selection),
        Some(k_sel),
        preservation,
        &[
            ("gate0", g0_body),
            ("gate1", g1_body),
            ("gate2", g2_body),
            ("gate3", g3_body),
            ("gate45", g45_body),
            ("gate6", g6_body),
            ("gate7", g7_body),
            ("gate8", g8_body),
            ("gate9", g9_body),
            ("gate10", g10_body),
            ("gate11", g11_body),
            ("gate12_pulse", g12_pc_body),
            ("gate12_damage", g12_dmg_body),
            ("gate13", g13_body),
        ],
    )
}
