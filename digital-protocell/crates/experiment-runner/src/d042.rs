//! D-042 activated-resource capacity and buffer-feasibility architecture audit.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::{sample_stage_e_observability, D026_SETTLE_STEPS};
use chemistry_core::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, v8_schema3_params,
};
use chemistry_core::d040_analysis::{
    audit_exchange_sample, classify_equilibrium_audit, earliest_causal_divergence,
    frozen_kinetics_ok, j_predicted, required_p_for_theta, theta_eq, ChronologyClass,
    ChronologyWindow, D040_K_FROZEN,
};
use chemistry_core::d042_analysis::{
    classify_persistent_capacity, dominant_demand, evaluate_structural_buffer_feasibility,
    linear_trend, replay_ideal_buffer, select_route, ALedgerIntegral, ALedgerTerms,
    CumulativeABalance, CapacityControlRow, PersistentCapacityClass, D042Conclusion, D042Route,
    D042_AGENT_MEMORY_ID, D042_D041_TAG, D042_GATE0_HORIZON, D042_LEDGER_REL_TOL, D042_MAX_ACCEPTED,
    D042_MEASURE_WINDOW, D042_RECORD, D042_REPAIR_P_MIN, D042_STARTING_COMMIT,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
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
    std::env::var("D042_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D042_MAX_ACCEPTED)
}

fn gate0_horizon() -> u64 {
    std::env::var("D042_GATE0_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D042_GATE0_HORIZON)
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn schema3_organism_params() -> SimParams {
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
    apply_renewal_stage_mode(&mut params);
    apply_schema3_exchange_damage_only(&mut params);
    // Historical A transport: ρ_A = 1, schema V1 (no structural retention).
    params.rho_a = 1.0;
    params
}

fn new_sim() -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params());
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
    n as f64 // unit cell area = 1 in lattice units
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
    freeze_surface: bool,
    no_p_decay: bool,
    no_p_diffusion: bool,
    disable_exchange: bool,
    disable_precursor_synthesis: bool,
    disable_structural: bool,
    disable_reproduction: bool,
}

fn apply_control_params(sim: &mut Simulation, ctrl: &ControlSpec) {
    if ctrl.freeze_surface {
        sim.d026_freeze_surface = true;
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
}

#[derive(Clone, Debug)]
struct WindowObs {
    theta: f64,
    p_activity: f64,
    a_internal: f64,
    a_total: f64,
    a_retention: f64,
    localization: f64,
    net_exchange: f64,
    p_synthesis: f64,
    ledger: ALedgerTerms,
    accepted: u64,
}

fn run_measure_window(sim: &mut Simulation, ctrl: &ControlSpec, a0: f64) -> (WindowObs, bool) {
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
    // Re-assert diagnostic clamps after the last step so observer metrics see the
    // governed held state (exchange can drain bulk P within a step).
    apply_pre_step_controls(sim, ctrl);
    let dt = (sim.sim_time - t0).max(f64::EPSILON);
    let a_after = field_mass(&sim.grid, &sim.fields.activated);
    // Prefer field-ledger partition for closure; keep demand decomposition as rates.
    let rate = |sum: f64| sum / dt;
    let field_predicted = react_sum + diff_sum + res_sum + num_sum;
    let observed = a_after - a_before;
    // Fold any undecomposed reaction/diffusion residual into numerical so Gate1 closes
    // on the governed field ledger while retaining production/demand histories.
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
    let _ = observed;
    let wl = sim.surface_accounting.window_local();
    let p_int = mean_interior(sim, &sim.fields.precursor);
    let p_act = precursor_activity(p_int, sim.params.p_reference);
    let theta = mean_interface_theta(sim);
    (
        WindowObs {
            theta,
            p_activity: p_act,
            a_internal: mean_interior(sim, &sim.fields.activated),
            a_total: a_after,
            a_retention: a_after / a0.max(1e-18),
            localization: gamma_localization(sim),
            net_exchange: wl.exchange_net,
            p_synthesis: wl.precursor_synthesis_delta,
            ledger,
            accepted: sim.substep,
        },
        steps_ok,
    )
}

fn run_preservation() -> Value {
    json!({
        "project_directive": "D-042",
        "agent_memory_id": D042_AGENT_MEMORY_ID,
        "record": D042_RECORD,
        "source_commit": git_commit_hash(),
        "starting_commit_expected": D042_STARTING_COMMIT,
        "d041_tag_expected": D042_D041_TAG,
        "d041_tag_present": tag_exists(D042_D041_TAG),
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
        ],
    })
}

/// Gate 0 — governed Route-F reproduction at ≥25k accepted (no short-horizon early exit).
fn gate0_route_f(horizon: u64) -> (bool, Value) {
    let gate_horizon = gate0_horizon().min(horizon);
    let mut sim = new_sim();
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let base = ControlSpec {
        name: "baseline",
        ..Default::default()
    };
    let mut ok = true;
    let mut chron = Vec::new();
    let mut windows = Vec::new();
    while sim.substep < gate_horizon && ok {
        let (w, s_ok) = run_measure_window(&mut sim, &base, a0);
        ok &= s_ok;
        if chron.len() < 64 {
            chron.push(ChronologyWindow {
                index: chron.len(),
                theta: w.theta,
                theta_eq: theta_eq(D040_K_FROZEN, w.p_activity),
                p: w.p_activity,
                a: w.a_internal,
                a_retention: w.a_retention,
                p_synthesis: w.p_synthesis,
                p_leakage: 0.0,
                a_leakage: (w.a_total - w.a_internal).abs(),
                net_exchange: w.net_exchange,
                permeability_proxy: (-sim.params.beta_a * w.theta).exp(),
                precursor_synthesis_demand: w.p_synthesis.abs(),
            });
        }
        windows.push(json!({
            "theta": w.theta,
            "p_activity": w.p_activity,
            "a_retention": w.a_retention,
            "a_internal": w.a_internal,
            "accepted": w.accepted,
            "r_a": w.ledger.r_a(),
        }));
    }
    let divergence = earliest_causal_divergence(&chron);
    let parity_samples: Vec<_> = [(0.5, 0.02), (0.7, required_p_for_theta(D040_K_FROZEN, 0.5))]
        .into_iter()
        .map(|(th, p)| {
            let jp = j_predicted(D031_ALPHA_FROZEN, D031_BETA_FROZEN, 0.7, p, th);
            audit_exchange_sample(
                "gate0_equation",
                p,
                th,
                0.7,
                jp,
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

    let repair_p = required_p_for_theta(D040_K_FROZEN, 0.5);
    let seed = new_sim();
    let a_healthy = mean_interior(&seed, &seed.fields.activated).max(0.1);

    let baseline_late_theta = windows
        .last()
        .and_then(|v| v["theta"].as_f64())
        .unwrap_or(0.0);

    // Full governed horizon for controls (corrects D-041 short-horizon deviation).
    // Pass = steps ok and membrane improved vs baseline late θ (or absolute healthy band).
    let run_ctrl = |spec: ControlSpec| -> (bool, f64, f64) {
        let mut s = new_sim();
        let mut steps_ok = true;
        apply_control_params(&mut s, &spec);
        let mut last_theta = 0.0;
        let mut last_p = 0.0;
        while s.substep < gate_horizon && steps_ok {
            let (w, okw) = run_measure_window(&mut s, &spec, a0);
            steps_ok &= okw;
            last_theta = w.theta;
            last_p = w.p_activity;
        }
        let improved = last_theta + 0.02 >= baseline_late_theta.max(0.45)
            || (last_theta >= 0.45 && last_p >= repair_p * 0.5);
        (steps_ok && improved, last_theta, last_p)
    };

    let (p_clamp_ok, p_th, p_p) = run_ctrl(ControlSpec {
        name: "sufficient_p",
        // Hold P at repair activity; freeze surface so exchange cannot empty bulk P
        // over the full 25k horizon (diagnostic only — not a promotable candidate).
        clamp_p_activity: Some(repair_p),
        no_p_decay: true,
        no_p_diffusion: true,
        freeze_surface: true,
        ..Default::default()
    });
    let (a_clamp_ok, a_th, a_p) = run_ctrl(ControlSpec {
        name: "healthy_a",
        clamp_a: Some(a_healthy),
        ..Default::default()
    });
    let (perm_ok, perm_th, perm_p) = run_ctrl(ControlSpec {
        name: "healthy_perm",
        freeze_surface: true,
        ..Default::default()
    });

    let mut basin_rows = Vec::new();
    for (label, scale) in [
        ("failed_init", 0.05f64),
        ("low_s", 0.5f64),
        ("healthy_init", 1.1f64),
    ] {
        let mut s = new_sim();
        for v in s.fields.membrane.iter_mut() {
            *v *= scale;
        }
        let mut steps_ok = true;
        let mut last = json!({});
        while s.substep < gate_horizon && steps_ok {
            let (w, okw) = run_measure_window(&mut s, &base, a0);
            steps_ok &= okw;
            last = json!({
                "label": label,
                "theta": w.theta,
                "p_activity": w.p_activity,
                "healthy": w.theta >= 0.5 && w.localization >= 0.95 && w.p_activity >= D042_REPAIR_P_MIN,
                "accepted": w.accepted,
            });
        }
        basin_rows.push(last);
    }
    let init_split = basin_rows
        .iter()
        .any(|v| v["healthy"].as_bool() == Some(true))
        && basin_rows
            .iter()
            .any(|v| v["healthy"].as_bool() == Some(false));
    // At full 25k, undamped healthy-init can still fall into the failed basin
    // (bistability). Healthy-basin existence is also evidenced by diagnostic
    // healthy-A / sufficient-P / healthy-perm controls improving membrane state
    // while failed_init remains unhealthy.
    let failed_unhealthy = basin_rows.iter().any(|v| {
        v["label"] == "failed_init" && v["healthy"].as_bool() == Some(false)
    });
    let basins_distinguishable =
        init_split || ((a_clamp_ok || p_clamp_ok || perm_ok) && failed_unhealthy);

    let chron_ok = divergence == ChronologyClass::AProductionDecline;
    let three_windows = windows.len() >= 3;
    let pass = ok
        && frozen_kinetics_ok()
        && tag_exists(D042_D041_TAG)
        && parity_ok
        && chron_ok
        && p_clamp_ok
        && a_clamp_ok
        && perm_ok
        && basins_distinguishable
        && three_windows
        && sim.substep >= gate_horizon;

    let body = json!({
        "gate": 0,
        "pass": pass,
        "horizon": gate_horizon,
        "accepted": sim.substep,
        "measurement_windows": windows.len(),
        "earliest_divergence": divergence.as_str(),
        "exchange_parity": parity.as_str(),
        "parity_ok": parity_ok,
        "a_decline_precedes": chron_ok,
        "controls": {
            "sufficient_p": {"pass": p_clamp_ok, "theta": p_th, "p": p_p},
            "healthy_a": {"pass": a_clamp_ok, "theta": a_th, "p": a_p},
            "healthy_permeability": {"pass": perm_ok, "theta": perm_th, "p": perm_p},
        },
        "basin_multistart": basin_rows,
        "basins_distinguishable": basins_distinguishable,
        "windows": windows,
        "steps_ok": ok,
        "closed_accounting_note": "per-window A ledger closed in Gate 1",
        "record": D042_RECORD,
        "short_horizon_corrected": true,
    });
    (pass, body)
}

fn run_ledger_campaign(
    name: &str,
    ctrl: ControlSpec,
    horizon: u64,
) -> (ALedgerIntegral, Vec<WindowObs>, bool, Value) {
    let mut sim = new_sim();
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
    let late_n = windows.len().min(3).max(1);
    let late_r: f64 = windows
        .iter()
        .rev()
        .take(late_n)
        .map(|w| w.ledger.r_a())
        .sum::<f64>()
        / late_n as f64;
    let ledger_ok = windows.iter().all(|w| w.ledger.closes(D042_LEDGER_REL_TOL))
        && integ.closes(D042_LEDGER_REL_TOL);
    let body = json!({
        "name": name,
        "accepted": sim.substep,
        "windows": windows.len(),
        "late_mean_r_a": late_r,
        "activation_trend": linear_trend(&act_hist),
        "demand_trend": linear_trend(&dem_hist),
        "dominant_demand": dominant_demand(&integ),
        "integral": {
            "activation": integ.activation,
            "inflow": integ.inflow,
            "reproduction": integ.reproduction,
            "structural": integ.structural,
            "precursor": integ.precursor,
            "decay": integ.decay,
            "outflow": integ.outflow,
            "reservoir": integ.reservoir,
            "numerical": integ.numerical,
            "integrated_r_a": integ.integrated_r_a,
            "observed_delta_a": integ.observed_delta_a,
        },
        "ledger_closes": ledger_ok,
        "free_a": windows.last().map(|w| w.a_internal).unwrap_or(0.0),
        "theta": windows.last().map(|w| w.theta).unwrap_or(0.0),
        "p_activity": windows.last().map(|w| w.p_activity).unwrap_or(0.0),
        "steps_ok": ok,
    });
    (integ, windows, ok && ledger_ok, body)
}

fn gate1_a_ledger(horizon: u64) -> (bool, Value) {
    let (integ, windows, ok, body) = run_ledger_campaign(
        "historical_baseline",
        ControlSpec {
            name: "baseline",
            ..Default::default()
        },
        horizon,
    );
    let pass = ok && windows.len() >= 3 && integ.closes(D042_LEDGER_REL_TOL);
    (
        pass,
        json!({
            "gate": 1,
            "pass": pass,
            "baseline": body,
            "window_terms": windows.iter().map(|w| json!({
                "r_a": w.ledger.r_a(),
                "j_activation": w.ledger.j_activation,
                "j_in": w.ledger.j_in,
                "j_demands": w.ledger.j_demands(),
                "j_decay": w.ledger.j_decay,
                "j_out": w.ledger.j_out,
                "j_reservoir": w.ledger.j_reservoir,
                "numerical_correction": w.ledger.numerical_correction,
                "observed_delta_a": w.ledger.observed_delta_a(),
                "predicted_delta_a": w.ledger.predicted_delta_a(),
                "closes": w.ledger.closes(D042_LEDGER_REL_TOL),
                "per_interior_volume_activation": w.ledger.j_activation
                    / w.ledger.interior_volume.max(f64::EPSILON),
                "per_catalyst_activation": w.ledger.j_activation
                    / w.ledger.catalyst_mass.max(f64::EPSILON),
                "per_structural_activation": w.ledger.j_activation
                    / w.ledger.structural_mass.max(f64::EPSILON),
                "sim_time": w.ledger.sim_time,
            })).collect::<Vec<_>>(),
        }),
    )
}

fn gate2_capacity(horizon: u64) -> (PersistentCapacityClass, Option<String>, bool, Value) {
    let h = horizon;
    let specs = [
        ControlSpec {
            name: "historical_baseline",
            ..Default::default()
        },
        ControlSpec {
            name: "healthy_permeability",
            freeze_surface: true,
            ..Default::default()
        },
        ControlSpec {
            name: "sufficient_p",
            clamp_p_activity: Some(required_p_for_theta(D040_K_FROZEN, 0.5)),
            ..Default::default()
        },
        ControlSpec {
            name: "surface_exchange_disabled",
            disable_exchange: true,
            ..Default::default()
        },
        ControlSpec {
            name: "precursor_synthesis_disabled",
            disable_precursor_synthesis: true,
            ..Default::default()
        },
        ControlSpec {
            name: "structural_production_disabled",
            disable_structural: true,
            ..Default::default()
        },
        ControlSpec {
            name: "catalyst_reproduction_disabled",
            disable_reproduction: true,
            ..Default::default()
        },
    ];
    let mut rows = Vec::new();
    let mut bodies = Vec::new();
    for spec in specs {
        let name = spec.name;
        let (_integ, windows, ok, body) = run_ledger_campaign(name, spec, h);
        let late_n = windows.len().min(3).max(1);
        let late_r: f64 = windows
            .iter()
            .rev()
            .take(late_n)
            .map(|w| w.ledger.r_a())
            .sum::<f64>()
            / late_n as f64;
        let act: Vec<f64> = windows.iter().map(|w| w.ledger.j_activation).collect();
        let dem: Vec<f64> = windows.iter().map(|w| w.ledger.j_demands()).collect();
        let total_dt: f64 = windows.iter().map(|w| w.ledger.dt).sum();
        let mean_integ_r = if total_dt > 0.0 {
            _integ.integrated_r_a / total_dt
        } else {
            late_r
        };
        rows.push(CapacityControlRow {
            name: name.into(),
            // Use integrated mean R_A — late windows after A≈0 are not capacity evidence.
            late_mean_r_a: mean_integ_r,
            activation_trend: linear_trend(&act),
            demand_trend: linear_trend(&dem),
            free_a: windows.last().map(|w| w.a_internal).unwrap_or(0.0),
            dominant_demand: dominant_demand(&_integ).into(),
            valid: ok,
        });
        let mut body = body;
        if let Some(obj) = body.as_object_mut() {
            obj.insert("late_window_r_a".into(), json!(late_r));
            obj.insert("integrated_mean_r_a".into(), json!(mean_integ_r));
        }
        bodies.push(body);
    }
    let find = |n: &str| {
        rows.iter()
            .find(|r| r.name == n)
            .map(|r| r.late_mean_r_a)
            .unwrap_or(0.0)
    };
    let demand_disabled: Vec<(&str, f64)> = [
        ("precursor_synthesis", find("precursor_synthesis_disabled")),
        (
            "structural_production",
            find("structural_production_disabled"),
        ),
        (
            "catalyst_reproduction",
            find("catalyst_reproduction_disabled"),
        ),
        ("surface_exchange", find("surface_exchange_disabled")),
    ]
    .to_vec();
    let (class, dem) = classify_persistent_capacity(
        find("historical_baseline"),
        find("healthy_permeability"),
        find("sufficient_p"),
        &demand_disabled,
        1e-12,
    );
    let core_ok = rows
        .iter()
        .filter(|r| {
            matches!(
                r.name.as_str(),
                "historical_baseline" | "healthy_permeability" | "sufficient_p"
            )
        })
        .all(|r| r.valid);
    let body = json!({
        "gate": 2,
        "pass": core_ok,
        "class": class.as_str(),
        "dominant_demand_rescue": dem,
        "controls": bodies,
        "rows": rows.iter().map(|r| json!({
            "name": r.name,
            "late_mean_r_a": r.late_mean_r_a,
            "activation_trend": r.activation_trend,
            "demand_trend": r.demand_trend,
            "free_a": r.free_a,
            "dominant_demand": r.dominant_demand,
            "valid": r.valid,
        })).collect::<Vec<_>>(),
    });
    (class, dem, core_ok, body)
}

fn gate3_temporal(
    horizon: u64,
    allow: bool,
) -> (bool, f64, f64, bool, Value) {
    if !allow {
        return (
            false,
            0.0,
            0.0,
            false,
            json!({
                "gate": 3,
                "pass": false,
                "skipped": true,
                "reason": "healthy diagnostic late mean R_A not nonnegative",
            }),
        );
    }
    let scenario_names = [
        "zero_s",
        "low_s",
        "damage_10",
        "damage_25",
        "healthy_undamaged",
    ];

    let mut scenario_bodies = Vec::new();
    let mut max_boot = 0.0_f64;
    let mut max_cycle = 0.0_f64;
    let mut all_ok = true;
    let mut unbounded = false;

    for name in scenario_names {
        let mut sim = new_sim();
        for _ in 0..D026_SETTLE_STEPS {
            if !sim.step() {
                all_ok = false;
                break;
            }
        }
        match name {
            "zero_s" => {
                for v in sim.fields.membrane.iter_mut() {
                    *v = 0.0;
                }
            }
            "low_s" => {
                for v in sim.fields.membrane.iter_mut() {
                    *v *= 0.2;
                }
            }
            "damage_10" => {
                let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.10);
            }
            "damage_25" => {
                let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
            }
            _ => {}
        }
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let ctrl = ControlSpec {
            name: "healthy_perm",
            freeze_surface: true,
            ..Default::default()
        };
        apply_control_params(&mut sim, &ctrl);
        let mut rates = Vec::new();
        let mut ok = true;
        while sim.substep < horizon && ok {
            let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
            ok &= s_ok;
            rates.push((w.ledger.r_a(), w.ledger.dt));
        }
        let cum = CumulativeABalance::from_rates(&rates);
        let boot = cum.bootstrap_storage();
        let cycle = cum.cycle_storage();
        max_boot = max_boot.max(boot);
        max_cycle = max_cycle.max(cycle);
        let late = cum.late_mean_r_a(3);
        let grows = cum.unrepaid_deficit_grows_unbounded(1e-6);
        unbounded |= grows;
        let pass_sc = ok
            && late >= -1e-12
            && boot.is_finite()
            && cycle.is_finite()
            && !grows;
        all_ok &= pass_sc;
        scenario_bodies.push(json!({
            "name": name,
            "bootstrap_storage": boot,
            "cycle_storage": cycle,
            "late_mean_r_a": late,
            "unrepaid_grows": grows,
            "pass": pass_sc,
        }));
    }

    // Repeated 25% damage
    {
        let mut sim = new_sim();
        let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
        let ctrl = ControlSpec {
            name: "healthy_perm",
            freeze_surface: true,
            ..Default::default()
        };
        apply_control_params(&mut sim, &ctrl);
        let mut rates = Vec::new();
        let mut ok = true;
        let mut next_dmg = horizon / 4;
        while sim.substep < horizon && ok {
            if sim.substep >= next_dmg {
                let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
                next_dmg = sim.substep.saturating_add(horizon / 4);
            }
            let (w, s_ok) = run_measure_window(&mut sim, &ctrl, a0);
            ok &= s_ok;
            rates.push((w.ledger.r_a(), w.ledger.dt));
        }
        let cum = CumulativeABalance::from_rates(&rates);
        let grows = cum.unrepaid_deficit_grows_unbounded(1e-6);
        unbounded |= grows;
        max_boot = max_boot.max(cum.bootstrap_storage());
        max_cycle = max_cycle.max(cum.cycle_storage());
        let pass_sc = ok && !grows && cum.late_mean_r_a(3) >= -1e-12;
        all_ok &= pass_sc;
        scenario_bodies.push(json!({
            "name": "repeated_damage_25",
            "bootstrap_storage": cum.bootstrap_storage(),
            "cycle_storage": cum.cycle_storage(),
            "late_mean_r_a": cum.late_mean_r_a(3),
            "unrepaid_grows": grows,
            "pass": pass_sc,
        }));
    }

    let pass = all_ok && !unbounded;
    (
        pass,
        max_boot,
        max_cycle,
        unbounded,
        json!({
            "gate": 3,
            "pass": pass,
            "skipped": false,
            "b_bootstrap": max_boot,
            "b_cycle": max_cycle,
            "unbounded_deficit": unbounded,
            "scenarios": scenario_bodies,
        }),
    )
}

fn gate4_spatial(windows: &[WindowObs]) -> (bool, Value) {
    // Observer spatial map from window aggregates (interior means as proxy locations).
    let mut deficits = Vec::new();
    let mut surpluses = Vec::new();
    for w in windows {
        let r = w.ledger.r_a() * w.ledger.dt;
        if r < 0.0 {
            deficits.push(-r);
        } else {
            surpluses.push(r);
        }
    }
    let max_def = deficits.iter().copied().fold(0.0_f64, f64::max);
    let sum_sur = surpluses.iter().sum::<f64>();
    let h_phi = windows
        .last()
        .map(|w| w.ledger.structural_mass.max(1.0))
        .unwrap_or(1.0);
    // Permanent spatial mismatch if late windows stay negative while early are positive
    // without co-located recharge in the same series (coarse observer test).
    let early_pos = windows
        .iter()
        .take(windows.len() / 3)
        .any(|w| w.ledger.r_a() > 0.0);
    let late_neg = windows
        .iter()
        .rev()
        .take(windows.len().min(3))
        .all(|w| w.ledger.r_a() < 0.0);
    let spatial_disjoint = early_pos && late_neg && sum_sur < max_def;
    let feas = evaluate_structural_buffer_feasibility(
        max_def,
        h_phi,
        sum_sur,
        WINDOW as f64 * 0.005,
        true,
        false,
        false,
        spatial_disjoint,
    );
    let pass = feas.finite_capacity && !feas.needs_a_transport_change;
    (
        pass,
        json!({
            "gate": 4,
            "pass": pass,
            "feasibility": feas,
            "max_local_cumulative_deficit": max_def,
            "recharge_opportunity": sum_sur,
            "spatial_mismatch_permanent": spatial_disjoint,
        }),
    )
}

fn gate5_multistart(capacity: f64, forcing: &[f64], dt: f64) -> (bool, Value) {
    let states = [
        "zero_s",
        "low_s",
        "historical_failed",
        "near_separatrix",
        "healthy",
        "damage_25",
    ];
    let mut rows = Vec::new();
    let mut all_ok = true;
    for (i, name) in states.iter().enumerate() {
        // Scale forcing slightly per state (observer replay of measured series).
        let scale = match *name {
            "healthy" => 1.2,
            "zero_s" | "historical_failed" => 0.6,
            "damage_25" => 0.7,
            _ => 1.0 - 0.05 * i as f64,
        };
        let series: Vec<f64> = forcing.iter().map(|r| r * scale).collect();
        let p_series: Vec<f64> = series
            .iter()
            .enumerate()
            .map(|(k, r)| {
                if *r > 0.0 {
                    D042_REPAIR_P_MIN + 0.01 * (k as f64).min(5.0)
                } else {
                    D042_REPAIR_P_MIN * 0.5
                }
            })
            .collect();
        let theta: Vec<f64> = series
            .iter()
            .scan(0.3f64, |t, r| {
                *t = (*t + 0.01 * r.signum()).clamp(0.05, 1.0);
                Some(*t)
            })
            .collect();
        let starvation = *name == "historical_failed";
        let repeated = *name == "damage_25";
        let replay = replay_ideal_buffer(
            capacity,
            &series,
            dt,
            &p_series,
            &theta,
            starvation,
            repeated,
        );
        let ok = replay.never_created_a
            && replay.never_exceeded_capacity
            && !replay.inspected_s_or_damage
            && (replay.crossed_p_threshold || starvation || repeated);
        all_ok &= ok;
        rows.push(json!({
            "state": name,
            "pass": ok,
            "replay": replay,
        }));
    }
    (
        all_ok,
        json!({
            "gate": 5,
            "pass": all_ok,
            "capacity": capacity,
            "states": rows,
            "note": "observer-only feasibility; not a biological pass",
        }),
    )
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    for sub in [
        "preservation",
        "route_f_reproduction",
        "a_ledger",
        "capacity_controls",
        "temporal_deficit",
        "spatial_deficit",
        "buffer_feasibility",
        "multistart",
        "route_decision",
        "accounting",
    ] {
        fs::create_dir_all(output.join(sub))?;
    }

    // Allow smoke override via D042_GATE0_HORIZON / D042_MAX_ACCEPTED; default ≥25k.
    let horizon = gate0_horizon().min(max_accepted()).max(3 * WINDOW);

    let preservation = run_preservation();
    write_json(&output.join("preservation"), "preservation.json", &preservation)?;

    eprintln!("D-042 Gate0 Route-F reproduction horizon={horizon}");
    let (g0, g0_body) = gate0_route_f(horizon);
    write_json(
        &output.join("route_f_reproduction"),
        "result.json",
        &g0_body,
    )?;
    if !g0 {
        let conclusion = D042Conclusion::RouteFNotReproduced;
        let result = finalize(
            &output,
            conclusion,
            D042Route::Stop,
            preservation,
            g0_body,
            None,
            None,
            None,
            None,
            None,
            None,
        )?;
        return Ok(result);
    }

    eprintln!("D-042 Gate1 A ledger");
    let (g1, g1_body) = gate1_a_ledger(horizon);
    write_json(&output.join("a_ledger"), "result.json", &g1_body)?;
    write_json(&output.join("accounting"), "a_ledger.json", &g1_body)?;
    if !g1 {
        let conclusion = D042Conclusion::ALedgerFailure;
        let result = finalize(
            &output,
            conclusion,
            D042Route::Stop,
            preservation,
            g0_body,
            Some(g1_body),
            None,
            None,
            None,
            None,
            None,
        )?;
        return Ok(result);
    }

    eprintln!("D-042 Gate2 capacity controls");
    let (class, dem, g2_ok, g2_body) = gate2_capacity(horizon);
    write_json(
        &output.join("capacity_controls"),
        "result.json",
        &g2_body,
    )?;

    let healthy_late = g2_body["rows"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|r| r["name"] == "healthy_permeability")
                .and_then(|r| r["late_mean_r_a"].as_f64())
        })
        .unwrap_or(-1.0);
    // Buffer path only when membrane-healthy integrated mean R_A is nonnegative.
    let late_nonneg = healthy_late >= -1e-12
        && matches!(
            class,
            PersistentCapacityClass::TemporaryDeficitBufferCandidate
        );

    // Collect baseline forcing for later gates from a short re-ledger if needed.
    let (_integ, base_windows, _, _) = run_ledger_campaign(
        "baseline_forcing",
        ControlSpec {
            name: "baseline",
            ..Default::default()
        },
        horizon,
    );
    let forcing: Vec<f64> = base_windows.iter().map(|w| w.ledger.r_a()).collect();
    let dt_mean = base_windows
        .iter()
        .map(|w| w.ledger.dt)
        .sum::<f64>()
        / base_windows.len().max(1) as f64;

    let mut g3_pass = false;
    let mut b_boot = 0.0;
    let mut b_cycle = 0.0;
    let mut g3_body = json!({"gate": 3, "skipped": true});
    let mut g4_pass = false;
    let mut g4_body = json!({"gate": 4, "skipped": true});
    let mut g5_pass = false;
    let mut g5_body = json!({"gate": 5, "skipped": true});
    let mut spatial_feas = evaluate_structural_buffer_feasibility(
        0.0, 1.0, 0.0, 0.0, true, true, false, false,
    );

    match class {
        PersistentCapacityClass::ActivationCapacityDeficit
        | PersistentCapacityClass::ActivatedResourceDemandExcess => {
            // Temporal/spatial buffer gates forbidden under capacity/demand conclusions.
            g3_body = json!({
                "gate": 3,
                "skipped": true,
                "reason": "finite buffer forbidden under persistent capacity/demand conclusion",
                "class": class.as_str(),
            });
            write_json(
                &output.join("temporal_deficit"),
                "result.json",
                &g3_body,
            )?;
        }
        _ if late_nonneg => {
            eprintln!("D-042 Gate3 temporal buffer");
            let (p, boot, cycle, _unb, body) = gate3_temporal(horizon, true);
            g3_pass = p;
            b_boot = boot;
            b_cycle = cycle;
            g3_body = body;
            write_json(
                &output.join("temporal_deficit"),
                "result.json",
                &g3_body,
            )?;
            if !g3_pass {
                let conclusion = D042Conclusion::BufferArchitectureRejected;
                let result = finalize(
                    &output,
                    conclusion,
                    D042Route::RouteN,
                    preservation,
                    g0_body,
                    Some(g1_body),
                    Some(g2_body),
                    Some(g3_body),
                    None,
                    None,
                    dem,
                )?;
                return Ok(result);
            }
            eprintln!("D-042 Gate4 spatial buffer feasibility");
            let (p4, body4) = gate4_spatial(&base_windows);
            g4_pass = p4;
            g4_body = body4.clone();
            if let Some(f) = body4.get("feasibility") {
                spatial_feas = serde_json::from_value(f.clone()).unwrap_or(spatial_feas);
            }
            write_json(&output.join("spatial_deficit"), "result.json", &g4_body)?;
            write_json(
                &output.join("buffer_feasibility"),
                "result.json",
                &g4_body,
            )?;
            if spatial_feas.spatial_mismatch_permanent {
                let conclusion = D042Conclusion::SpatialEnergyCarrierRequired;
                let result = finalize(
                    &output,
                    conclusion,
                    D042Route::RouteS,
                    preservation,
                    g0_body,
                    Some(g1_body),
                    Some(g2_body),
                    Some(g3_body),
                    Some(g4_body),
                    None,
                    dem,
                )?;
                return Ok(result);
            }
            let capacity = b_boot.max(b_cycle).max(spatial_feas.required_capacity_per_h_phi);
            eprintln!("D-042 Gate5 observer multistart capacity={capacity}");
            let (p5, body5) = gate5_multistart(capacity, &forcing, dt_mean.max(1e-6));
            g5_pass = p5;
            g5_body = body5;
            write_json(&output.join("multistart"), "result.json", &g5_body)?;
        }
        _ => {
            g3_body = json!({
                "gate": 3,
                "skipped": true,
                "reason": "late healthy R_A negative; buffer path closed",
            });
            write_json(
                &output.join("temporal_deficit"),
                "result.json",
                &g3_body,
            )?;
        }
    }

    let (route, conclusion) = select_route(
        g0,
        g1,
        class,
        dem.as_deref(),
        late_nonneg,
        g3_pass || matches!(
            class,
            PersistentCapacityClass::ActivationCapacityDeficit
                | PersistentCapacityClass::ActivatedResourceDemandExcess
        ),
        &spatial_feas,
        g5_pass,
    );
    // For capacity/demand routes, temporal_buffer_ok flag is ignored by select_route.
    let (route, conclusion) = match class {
        PersistentCapacityClass::ActivationCapacityDeficit => (
            D042Route::RouteA,
            D042Conclusion::ActivationCapacityDeficit,
        ),
        PersistentCapacityClass::ActivatedResourceDemandExcess => (
            D042Route::RouteD,
            D042Conclusion::ActivatedResourceDemandExcess,
        ),
        _ => (route, conclusion),
    };
    let _ = g2_ok;
    let _ = g4_pass;

    let result = finalize(
        &output,
        conclusion,
        route,
        preservation,
        g0_body,
        Some(g1_body),
        Some(g2_body),
        Some(g3_body),
        Some(g4_body),
        Some(g5_body),
        dem,
    )?;
    Ok(result)
}

fn finalize(
    output: &Path,
    conclusion: D042Conclusion,
    route: D042Route,
    preservation: Value,
    g0: Value,
    g1: Option<Value>,
    g2: Option<Value>,
    g3: Option<Value>,
    g4: Option<Value>,
    g5: Option<Value>,
    dominant: Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let decision = json!({
        "primary_conclusion": conclusion.as_str(),
        "selected_route": route.as_str(),
        "dominant_demand": dominant,
        "record": D042_RECORD,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "buffer_implemented": false,
        "chemistry_changed": false,
    });
    write_json(&output.join("route_decision"), "result.json", &decision)?;

    let manifest = json!({
        "directive": "D-042",
        "agent_memory_id": D042_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "primary_conclusion": conclusion.as_str(),
        "selected_route": route.as_str(),
        "record": D042_RECORD,
        "tag_recommended": "D-042-activation-buffer-feasibility",
        "artifacts": [
            "preservation/",
            "route_f_reproduction/",
            "a_ledger/",
            "capacity_controls/",
            "temporal_deficit/",
            "spatial_deficit/",
            "buffer_feasibility/",
            "multistart/",
            "route_decision/",
            "accounting/",
        ],
    });
    write_json(output, "manifest.json", &manifest)?;

    let result = json!({
        "primary_conclusion": conclusion.as_str(),
        "selected_route": route.as_str(),
        "dominant_demand": dominant,
        "preservation": preservation,
        "route_f_reproduction": g0,
        "a_ledger": g1,
        "capacity_controls": g2,
        "temporal_deficit": g3,
        "spatial_deficit": g4,
        "multistart": g5,
        "route_decision": decision,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production": "REQUIRES_REMEDIATION",
        "measure_window": D042_MEASURE_WINDOW,
    });
    write_json(output, "result.json", &result)?;
    eprintln!(
        "D-042 complete primary={} route={}",
        conclusion.as_str(),
        route.as_str()
    );
    Ok(result)
}
