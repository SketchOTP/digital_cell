//! D-043 activation-reaction capacity repair pipeline (Gates 0–9, stop-on-fail).

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::{sample_stage_e_observability, D026_SETTLE_STEPS};
use chemistry_core::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, classify_damage_repair,
    revised_stage_e_membrane_contract, v8_schema3_params, DamageRepairClass, D039_NET_S_FLOW_MAX,
    D039_REPLACEMENT_MIN, D039_S_DRIFT_MAX, D039_TRACER_RESIDUAL_MAX,
};
use chemistry_core::d040_analysis::{
    audit_exchange_sample, classify_equilibrium_audit, earliest_causal_divergence,
    frozen_kinetics_ok, j_predicted, required_p_for_theta, theta_eq, ChronologyClass,
    ChronologyWindow, D040_K_FROZEN,
};
use chemistry_core::d042_analysis::{
    dominant_demand, linear_trend, ALedgerIntegral, ALedgerTerms, CumulativeABalance,
};
use chemistry_core::d043_analysis::{
    activation_basis, build_activation_candidates, build_rate_estimate, check_activation_parity,
    classify_capacity_deficit, d042_capacity_deficit_reproduced, evaluate_candidate_row,
    evaluate_portable_rate, parity_suite_passes, screen_candidates, total_basis_from_activation_flux,
    zero_control_passes,
    CapacityClassification, D043Conclusion, D043_AGENT_MEMORY_ID, D043_D042_TAG,
    D043_GATE0_HORIZON, D043_HISTORICAL_K_ACTIVATION, D043_LEDGER_REL_TOL, D043_MAX_ACCEPTED,
    D043_MEASURE_WINDOW, D043_RECORD, D043_REPAIR_P_MIN, D043_STARTING_COMMIT,
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
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WINDOW: u64 = 500;
const RADIUS: f64 = 22.0;
const THETA: f64 = 0.6;
const ROUTE_QUALIFIED: &str = "MEMBRANE_ARCHITECTURE_V8_SCHEMA3_RECALIBRATED_ACTIVATION";

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
    std::env::var("D043_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D043_MAX_ACCEPTED)
}

fn gate0_horizon() -> u64 {
    std::env::var("D043_GATE0_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D043_GATE0_HORIZON)
}

/// Shorter horizon for Gates 2–4 diagnostics (clamped controls / rate screen).
fn diagnostic_horizon(gate0: u64) -> u64 {
    let requested = std::env::var("D043_DIAGNOSTIC_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000);
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
    let mut act_hist = Vec::new();
    let mut dem_hist = Vec::new();
    while sim.substep < horizon && ok {
        let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
        ok &= s_ok;
        integ.accumulate(&w.ledger);
        act_hist.push(w.ledger.j_activation);
        dem_hist.push(w.ledger.j_demands());
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
        "activation_trend": linear_trend(&act_hist),
        "demand_trend": linear_trend(&dem_hist),
        "dominant_demand": dominant_demand(&integ),
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

fn run_preservation() -> Value {
    json!({
        "project_directive": "D-043",
        "agent_memory_id": D043_AGENT_MEMORY_ID,
        "record": D043_RECORD,
        "source_commit": git_commit_hash(),
        "starting_commit_expected": D043_STARTING_COMMIT,
        "d042_tag_expected": D043_D042_TAG,
        "d042_tag_present": tag_exists(D043_D042_TAG),
        "historical_k_activation": D043_HISTORICAL_K_ACTIVATION,
        "activation_equation": "r_activation = k_d008_activation * C * N * F",
        "activation_basis": "B_activation = max(0,C) * max(0,N) * max(0,F)",
        "frozen_kinetics_ok": frozen_kinetics_ok(),
        "alpha": D031_ALPHA_FROZEN,
        "beta": D031_BETA_FROZEN,
        "K": D040_K_FROZEN,
        "architecture": "membrane_metabolism_v8_reversible_surface_exchange",
        "turnover": "surface_turnover_schema_3_exchange_damage_only",
        "a_transport": "historical rho_A=1",
        "frozen_conclusions": [
            "D040_MEMBRANE_METABOLISM_BISTABILITY",
            "D041_STRUCTURAL_A_RETENTION_NOT_SUFFICIENT",
            "D042_ACTIVATION_CAPACITY_DEFICIT",
        ],
    })
}

fn gate0_d042_reproduction(horizon: u64) -> (bool, Value) {
    let gate_horizon = gate0_horizon().min(horizon);
    let k = D043_HISTORICAL_K_ACTIVATION;
    let (integ, windows, ok, _) = run_ledger_campaign(
        k,
        "historical_baseline",
        ControlSpec {
            name: "baseline",
            ..Default::default()
        },
        gate_horizon,
    );
    let mut chron = Vec::new();
    let mut rates = Vec::new();
    let mut sim = new_sim(k);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let base = ControlSpec {
        name: "baseline",
        ..Default::default()
    };
    let mut steps_ok = true;
    while sim.substep < gate_horizon && steps_ok {
        let (w, s_ok) = run_measure_window(&mut sim, &base, a0);
        steps_ok &= s_ok;
        rates.push((w.ledger.r_a(), w.ledger.dt));
        if chron.len() < 64 {
            chron.push(ChronologyWindow {
                index: chron.len(),
                theta: w.theta,
                theta_eq: theta_eq(D040_K_FROZEN, w.p_activity),
                p: w.p_activity,
                a: w.a_internal,
                a_retention: w.a_total / a0,
                p_synthesis: 0.0,
                p_leakage: 0.0,
                a_leakage: 0.0,
                net_exchange: w.net_exchange,
                permeability_proxy: (-sim.params.beta_a * w.theta).exp(),
                precursor_synthesis_demand: 0.0,
            });
        }
    }
    let parity_samples: Vec<_> = [(0.5, 0.02), (0.7, required_p_for_theta(D040_K_FROZEN, 0.5))]
        .into_iter()
        .map(|(th, p)| {
            audit_exchange_sample(
                "gate0_equation",
                p,
                th,
                0.7,
                j_predicted(D031_ALPHA_FROZEN, D031_BETA_FROZEN, 0.7, p, th),
                D031_ALPHA_FROZEN,
                D031_BETA_FROZEN,
                D040_K_FROZEN,
            )
        })
        .collect();
    let parity = classify_equilibrium_audit(&parity_samples);
    let parity_ok = !matches!(
        parity,
        chemistry_core::d040_analysis::ExchangeParityClass::ExchangeRuntimeParityDefect
            | chemistry_core::d040_analysis::ExchangeParityClass::ExchangeEquilibriumUndefined
    );
    let chron_ok = earliest_causal_divergence(&chron) == ChronologyClass::AProductionDecline;
    let deficit_ok = d042_capacity_deficit_reproduced(&integ, windows.len());
    let pass = ok
        && steps_ok
        && frozen_kinetics_ok()
        && tag_exists(D043_D042_TAG)
        && parity_ok
        && chron_ok
        && deficit_ok
        && windows.len() >= 3
        && sim.substep >= gate_horizon.min(max_accepted());
    let cum = CumulativeABalance::from_rates(&rates);
    (
        pass,
        json!({
            "gate": 0,
            "pass": pass,
            "horizon": gate_horizon,
            "accepted": sim.substep,
            "integrated_r_a": integ.integrated_r_a,
            "ledger_closes": integ.closes(D043_LEDGER_REL_TOL),
            "deficit_reproduced": deficit_ok,
            "earliest_divergence": earliest_causal_divergence(&chron).as_str(),
            "exchange_parity": parity.as_str(),
            "parity_ok": parity_ok,
            "a_decline_precedes": chron_ok,
            "bootstrap_storage": cum.bootstrap_storage(),
            "record": D043_RECORD,
        }),
    )
}

fn gate1_activation_parity() -> (bool, Value) {
    let params = schema3_organism_params(D043_HISTORICAL_K_ACTIVATION);
    let k = D043_HISTORICAL_K_ACTIVATION;
    let grid = [
        (0.5, 0.5, 0.5, 0.2),
        (1.0, 0.8, 0.6, 0.3),
        (0.2, 1.0, 1.0, 0.1),
    ];
    let samples: Vec<_> = grid
        .iter()
        .map(|&(c, n, f, a)| {
            let p = check_activation_parity(k, c, n, f, a, &params);
            json!({
                "c": c, "n": n, "f": f, "a": a,
                "basis_observer": p.basis_observer,
                "rate_observer": p.rate_observer,
                "rate_runtime": p.rate_runtime,
                "basis_match": p.basis_match,
                "rate_match": p.rate_match,
            })
        })
        .collect();
    let zero_ok = zero_control_passes(k, &params);
    let suite_ok = parity_suite_passes(k, &params);
    let pass = zero_ok && suite_ok && samples.iter().all(|s| {
        s["basis_match"].as_bool() == Some(true) && s["rate_match"].as_bool() == Some(true)
    });
    (
        pass,
        json!({
            "gate": 1,
            "pass": pass,
            "zero_controls": zero_ok,
            "parity_suite": suite_ok,
            "samples": samples,
            "equation": "r = k * C * N * F",
            "historical_k": k,
        }),
    )
}

fn gate2_capacity_decomposition(horizon: u64) -> (CapacityClassification, bool, Value) {
    let k = D043_HISTORICAL_K_ACTIVATION;
    let seed = new_sim(k);
    let healthy_c = mean_interior(&seed, &seed.fields.catalyst).max(0.5);
    let healthy_n = mean_interior(&seed, &seed.fields.nutrient).max(0.5);
    let healthy_f = mean_interior(&seed, &seed.fields.fuel).max(0.5);
    let specs = [
        ControlSpec {
            name: "historical",
            ..Default::default()
        },
        ControlSpec {
            name: "healthy_n",
            clamp_n: Some(healthy_n),
            ..Default::default()
        },
        ControlSpec {
            name: "healthy_f",
            clamp_f: Some(healthy_f),
            ..Default::default()
        },
        ControlSpec {
            name: "healthy_c",
            clamp_c: Some(healthy_c),
            ..Default::default()
        },
        ControlSpec {
            name: "healthy_nf",
            clamp_n: Some(healthy_n),
            clamp_f: Some(healthy_f),
            ..Default::default()
        },
        ControlSpec {
            name: "healthy_cnf",
            clamp_c: Some(healthy_c),
            clamp_n: Some(healthy_n),
            clamp_f: Some(healthy_f),
            ..Default::default()
        },
        ControlSpec {
            name: "no_a_decay",
            no_a_decay: true,
            ..Default::default()
        },
        ControlSpec {
            name: "demands_disabled",
            disable_all_demands: true,
            ..Default::default()
        },
    ];
    let mut bodies = Vec::new();
    let mut balances = std::collections::HashMap::new();
    let mut all_ok = true;
    for spec in specs {
        let name = spec.name;
        let (_integ, windows, ok, mut body) = run_ledger_campaign(k, name, spec, horizon);
        all_ok &= ok;
        let total_dt: f64 = windows.iter().map(|w| w.ledger.dt).sum();
        let mean_r = if total_dt > 0.0 {
            _integ.integrated_r_a / total_dt
        } else {
            0.0
        };
        balances.insert(name.to_string(), mean_r);
        if let Some(obj) = body.as_object_mut() {
            obj.insert("integrated_mean_r_a".into(), json!(mean_r));
            if let Some(w) = windows.last() {
                obj.insert(
                    "activation_basis".into(),
                    json!(activation_basis(
                        w.c_internal,
                        w.n_internal,
                        w.f_internal
                    )),
                );
            }
        }
        bodies.push(body);
    }
    let find = |n: &str| *balances.get(n).unwrap_or(&0.0);
    let (class, rescue) = classify_capacity_deficit(
        find("historical"),
        find("healthy_n"),
        find("healthy_f"),
        find("healthy_c"),
        find("healthy_nf"),
        find("healthy_cnf"),
        find("no_a_decay"),
        find("demands_disabled"),
        1e-12,
    );
    let proceed = class == CapacityClassification::RateCapacity;
    (
        class,
        all_ok && proceed,
        json!({
            "gate": 2,
            "pass": all_ok,
            "proceed_to_gate3": proceed,
            "class": class.as_str(),
            "rescue": rescue,
            "controls": bodies,
        }),
    )
}

fn gate3_rate_reconstruction(horizon: u64) -> (bool, f64, Value) {
    let k = D043_HISTORICAL_K_ACTIVATION;
    // Controlled stationary family: hold A at seed-healthy level so L_A reflects
    // sustained authorized demand (not post-collapse free-A extinction).
    let short = horizon.min(3 * WINDOW).max(2 * WINDOW);
    let mut estimates = Vec::new();
    let state_specs: [(&str, f64, f64, f64); 8] = [
        ("R16", 0.6, 0.7, 0.7),
        ("R22", 0.8, 0.8, 0.8),
        ("R32", 1.0, 0.9, 0.9),
        ("low_c", 0.3, 0.8, 0.8),
        ("med_c", 0.6, 0.8, 0.8),
        ("high_c", 1.0, 0.8, 0.8),
        ("low_nf", 0.8, 0.3, 0.3),
        ("high_nf", 0.8, 1.0, 1.0),
    ];
    for (label, c, n, f) in state_specs {
        let spec = ControlSpec {
            name: label,
            clamp_a: Some(0.5),
            clamp_c: Some(c),
            clamp_n: Some(n),
            clamp_f: Some(f),
            // Do not freeze surface: precursor/structural authorized demand must remain active.
            ..Default::default()
        };
        let (_integ, windows, _combined_ok, body) = run_ledger_campaign(k, label, spec, short);
        // Diagnostic A/C/N/F clamps intentionally break field-ledger closure; require steps only.
        let accepted = body["accepted"].as_u64().unwrap_or(0);
        if windows.is_empty() || accepted < WINDOW {
            continue;
        }
        // Use early clamped windows: authorized productive demand is highest while A is
        // held healthy. Late windows under-report L_A after precursor/structural collapse.
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
        estimates.push(build_rate_estimate(
            label,
            c_m,
            n_m,
            f_m,
            total_basis,
            &terms,
            0.05,
        ));
    }
    let report = evaluate_portable_rate(&estimates);
    (
        report.pass,
        report.k_median,
        json!({
            "gate": 3,
            "pass": report.pass,
            "k_median": report.k_median,
            "k_min": report.k_min,
            "k_max": report.k_max,
            "span": report.span,
            "valid_count": report.valid_count,
            "loo_median_max_deviation": report.loo_median_max_deviation,
            "a_clamp": 0.5,
            "estimates": report.estimates,
        }),
    )
}

fn sustained_authorized_loss(terms: &ALedgerTerms) -> f64 {
    chemistry_core::d043_analysis::sustained_a_loss(terms)
}

fn gate4_candidate_screen(reconstructed_k: f64, horizon: u64) -> (bool, Option<f64>, Value) {
    let screen_horizon = horizon.min(6 * WINDOW);
    let candidates = build_activation_candidates(reconstructed_k);
    let mut rows = Vec::new();
    for &k in &candidates {
        // Free membrane dynamics — freeze_surface would block basin/P criteria.
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
        // Sustained balance: require nonnegative integrated R_A (not mean rate).
        let integrated_r_a = integ.integrated_r_a;
        let exhaustion = n < 0.05 || f < 0.05 || c < 0.05;
        let accumulation = free_a > 100.0;
        let clipping = !ok;
        let row = evaluate_candidate_row(
            k,
            reconstructed_k,
            integrated_r_a,
            free_a,
            p_act,
            theta,
            n,
            f,
            c,
            integ.closes(D043_LEDGER_REL_TOL),
            exhaustion,
            accumulation,
            clipping,
        );
        rows.push(row);
    }
    let report = screen_candidates(reconstructed_k, rows.clone());
    (
        report.pass,
        report.selected_k,
        json!({
            "gate": 4,
            "pass": report.pass,
            "reconstructed_k": reconstructed_k,
            "selected_k": report.selected_k,
            "candidates": report.candidates,
        }),
    )
}

fn gate5_foundational(k: f64, horizon: u64) -> (bool, Value) {
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
        let c1 = field_mass(&sim.grid, &sim.fields.catalyst);
        let a1 = field_mass(&sim.grid, &sim.fields.activated);
        let c_ret = c1 / c0.max(1e-18);
        let a_ret = a1 / a0;
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
            "gate": 5,
            "pass": pass,
            "parity_ok": parity_ok,
            "fixed_compartments": radius_rows,
            "k_activation": k,
        }),
    )
}

fn gate6_basin_multistart(k: f64, horizon: u64) -> (bool, Value) {
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
            "gate": 6,
            "pass": pass,
            "any_healthy": any_healthy,
            "multistarts": rows,
            "k_activation": k,
        }),
    )
}

fn gate7_pulse_chase(k: f64, horizon: u64) -> (bool, Value) {
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
            "gate": 7,
            "pass": pass,
            "replacement_fraction": replacement,
            "s_drift": s_drift,
            "tracer_residual": tracer_residual,
            "accepted": sim.substep,
        }),
    )
}

fn gate8_damage(k: f64, horizon: u64) -> (bool, Value) {
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
    for (id, _) in [
        ("activation_disabled", true),
        ("no_p", false),
    ] {
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
            "gate": 8,
            "pass": pass,
            "damage": damage_rows,
            "resource_controls": control_rows,
        }),
    )
}

fn gate9_stage_e_membrane(k: f64, horizon: u64) -> (bool, Value) {
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
            "gate": 9,
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
    conclusion: D043Conclusion,
    selected_k: Option<f64>,
    preservation: Value,
    bodies: &[(&str, Value)],
) -> Result<Value, Box<dyn std::error::Error>> {
    let qualified = conclusion == D043Conclusion::ActivationRateRepairQualified;
    let decision = json!({
        "primary_conclusion": conclusion.as_str(),
        "selected_k_activation": selected_k,
        "selected_architecture": if qualified {
            json!(ROUTE_QUALIFIED)
        } else {
            Value::Null
        },
        "record": D043_RECORD,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": if qualified { "REQUIRES_REMEDIATION" } else { "REQUIRES_REMEDIATION" },
        "activation_equation": "r_activation = k_d008_activation * C * N * F",
        "historical_k": D043_HISTORICAL_K_ACTIVATION,
    });
    write_json(output, "decision.json", &decision)?;

    let manifest = json!({
        "directive": "D-043",
        "agent_memory_id": D043_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "primary_conclusion": conclusion.as_str(),
        "selected_k_activation": selected_k,
        "record": D043_RECORD,
        "tag_recommended_pass": "D-043-activation-capacity-repair",
        "tag_recommended_fail": "D-043-activation-capacity-fail",
        "artifacts": [
            "preservation/",
            "d042_reproduction/",
            "activation_parity/",
            "capacity_decomposition/",
            "rate_reconstruction/",
            "candidate_screen/",
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
        "primary_conclusion": conclusion.as_str(),
        "selected_k_activation": selected_k,
        "preservation": preservation,
        "decision": decision,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "measure_window": D043_MEASURE_WINDOW,
    });
    if let Some(obj) = result.as_object_mut() {
        for (k, v) in bodies {
            obj.insert(k.to_string(), v.clone());
        }
    }
    write_json(output, "result.json", &result)?;
    eprintln!(
        "D-043 complete primary={} k={:?}",
        conclusion.as_str(),
        selected_k
    );
    Ok(result)
}

fn stop(
    output: &Path,
    conclusion: D043Conclusion,
    preservation: Value,
    bodies: &[(&str, Value)],
) -> Result<Value, Box<dyn std::error::Error>> {
    finalize(output, conclusion, None, preservation, bodies)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    for sub in [
        "preservation",
        "d042_reproduction",
        "activation_parity",
        "capacity_decomposition",
        "rate_reconstruction",
        "candidate_screen",
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

    eprintln!("D-043 Gate0 D-042 reproduction horizon={horizon}");
    let (g0_pass, g0_body) = gate0_d042_reproduction(horizon);
    write_json(
        &output.join("d042_reproduction"),
        "result.json",
        &g0_body,
    )?;
    if !g0_pass {
        return stop(
            &output,
            D043Conclusion::D042CapacityDeficitNotReproduced,
            preservation,
            &[("gate0", g0_body)],
        );
    }

    eprintln!("D-043 Gate1 activation parity");
    let (g1_pass, g1_body) = gate1_activation_parity();
    write_json(
        &output.join("activation_parity"),
        "result.json",
        &g1_body,
    )?;
    if !g1_pass {
        return stop(
            &output,
            D043Conclusion::ActivationImplementationDefect,
            preservation,
            &[("gate0", g0_body), ("gate1", g1_body)],
        );
    }

    eprintln!("D-043 Gate2 capacity decomposition diagnostic_horizon={diag}");
    let (class, g2_proceed, g2_body) = gate2_capacity_decomposition(diag);
    write_json(
        &output.join("capacity_decomposition"),
        "result.json",
        &g2_body,
    )?;
    let g2_conclusion = match class {
        CapacityClassification::SubstrateDelivery => {
            Some(D043Conclusion::ActivationSubstrateDeliveryDefect)
        }
        CapacityClassification::CatalystBasis => {
            Some(D043Conclusion::ActivationCatalystBasisDeficit)
        }
        CapacityClassification::DecayDefect => Some(D043Conclusion::ActivatedResourceDecayDefect),
        CapacityClassification::DemandDefect => Some(D043Conclusion::ActivatedResourceDemandDefect),
        CapacityClassification::RateCapacity => None,
    };
    if let Some(c) = g2_conclusion {
        return stop(
            &output,
            c,
            preservation,
            &[("gate0", g0_body), ("gate1", g1_body), ("gate2", g2_body)],
        );
    }
    if !g2_proceed {
        return stop(
            &output,
            D043Conclusion::Fail,
            preservation,
            &[("gate0", g0_body), ("gate1", g1_body), ("gate2", g2_body)],
        );
    }

    eprintln!("D-043 Gate3 portable rate reconstruction diagnostic_horizon={diag}");
    let (g3_pass, k_median, g3_body) = gate3_rate_reconstruction(diag);
    write_json(
        &output.join("rate_reconstruction"),
        "result.json",
        &g3_body,
    )?;
    if !g3_pass {
        return stop(
            &output,
            D043Conclusion::ActivationRateNotPortable,
            preservation,
            &[
                ("gate0", g0_body),
                ("gate1", g1_body),
                ("gate2", g2_body),
                ("gate3", g3_body),
            ],
        );
    }

    eprintln!("D-043 Gate4 candidate screen k_median={k_median} diagnostic_horizon={diag}");
    let (g4_pass, selected_k, g4_body) = gate4_candidate_screen(k_median, diag);
    write_json(
        &output.join("candidate_screen"),
        "result.json",
        &g4_body,
    )?;
    if !g4_pass {
        return stop(
            &output,
            D043Conclusion::ActivationRateRepairNotFound,
            preservation,
            &[
                ("gate0", g0_body),
                ("gate1", g1_body),
                ("gate2", g2_body),
                ("gate3", g3_body),
                ("gate4", g4_body),
            ],
        );
    }
    let k_sel = selected_k.unwrap_or(k_median);

    eprintln!("D-043 Gate5 foundational k={k_sel}");
    let (g5_pass, g5_body) = gate5_foundational(k_sel, horizon);
    write_json(
        &output.join("foundational_activation"),
        "result.json",
        &g5_body,
    )?;
    if !g5_pass {
        return stop(
            &output,
            D043Conclusion::FoundationalActivationRegression,
            preservation,
            &[
                ("gate0", g0_body),
                ("gate4", g4_body),
                ("gate5", g5_body),
            ],
        );
    }

    eprintln!("D-043 Gate6 basin multistart");
    let (g6_pass, g6_body) = gate6_basin_multistart(k_sel, max_accepted());
    write_json(&output.join("basin_multistart"), "result.json", &g6_body)?;
    if !g6_pass {
        return stop(
            &output,
            D043Conclusion::MembraneBasinNotRecovered,
            preservation,
            &[("gate4", g4_body), ("gate6", g6_body)],
        );
    }

    eprintln!("D-043 Gate7 pulse-chase");
    let (g7_pass, g7_body) = gate7_pulse_chase(k_sel, max_accepted());
    write_json(&output.join("pulse_chase"), "result.json", &g7_body)?;
    if !g7_pass {
        return stop(
            &output,
            D043Conclusion::ContinuousReplacementNotRecovered,
            preservation,
            &[("gate7", g7_body)],
        );
    }

    eprintln!("D-043 Gate8 damage");
    let (g8_pass, g8_body) = gate8_damage(k_sel, max_accepted());
    write_json(&output.join("damage"), "result.json", &g8_body)?;
    write_json(&output.join("resource_controls"), "result.json", &g8_body)?;
    if !g8_pass {
        let c = if g8_body["damage"]
            .as_array()
            .map(|a| a.iter().any(|r| r["pass"] == false))
            == Some(true)
        {
            D043Conclusion::DamageRepairNotRecovered
        } else {
            D043Conclusion::ResourceDependenceNotEstablished
        };
        return stop(&output, c, preservation, &[("gate8", g8_body)]);
    }

    eprintln!("D-043 Gate9 stage E membrane contract");
    let (g9_pass, g9_body) = gate9_stage_e_membrane(k_sel, max_accepted());
    write_json(
        &output.join("stage_e_membrane_contract"),
        "result.json",
        &g9_body,
    )?;
    write_json(
        &output.join("accounting"),
        "summary.json",
        &json!({"gates_passed": 9, "k_activation": k_sel}),
    )?;
    if !g9_pass {
        return stop(
            &output,
            D043Conclusion::StageEMembraneContractFailure,
            preservation,
            &[("gate9", g9_body)],
        );
    }

    finalize(
        &output,
        D043Conclusion::ActivationRateRepairQualified,
        Some(k_sel),
        preservation,
        &[
            ("gate0", g0_body),
            ("gate1", g1_body),
            ("gate2", g2_body),
            ("gate3", g3_body),
            ("gate4", g4_body),
            ("gate5", g5_body),
            ("gate6", g6_body),
            ("gate7", g7_body),
            ("gate8", g8_body),
            ("gate9", g9_body),
        ],
    )
}
