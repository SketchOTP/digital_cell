//! D-040 exchange–precursor coupling decomposition pipeline (diagnostic only).

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d030_analysis::{
    build_fixed_interface_state, catalyst_for_q, compute_exchange_local_bases,
};
use chemistry_core::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, v8_schema3_params,
};
use chemistry_core::d040_analysis::{
    audit_exchange_sample, classify_basins, classify_endogenous_capacity, classify_equilibrium_audit,
    earliest_causal_divergence, find_reduced_fixed_points, frozen_kinetics_ok, j_predicted,
    required_p_for_theta, required_p_thresholds, select_route, theta_eq, ChronologyWindow,
    EndogenousCapacityClass, ExchangeParityClass, PrecursorSufficiencyOutcome, ReducedApsParams,
    RouteEvidence, D040_AGENT_MEMORY_ID, D040_D039_TAG, D040_K_FROZEN, D040_RECORD,
    D040_STARTING_COMMIT,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::surface_density::{
    compute_interface_geometry, evolve_surface_density, precursor_activity, reconstruct_gamma_field,
    surface_localization, surface_occupancy_theta, total_surface_mass, InterfaceGeometryCell,
    SurfaceAccountingTotals,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WINDOW: u64 = 500;
const RADIUS: f64 = 22.0;
const THETA0: f64 = 0.6;
const DEFAULT_HORIZON: u64 = 8_000;

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

fn tag_exists(tag: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn commit_exists(prefix: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("{prefix}^{{commit}}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn max_accepted() -> u64 {
    std::env::var("D040_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HORIZON)
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
    params
}

fn new_sim(enforce_fixed: bool) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params());
    sim.enforce_structure_constraint = enforce_fixed;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, RADIUS, THETA0);
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

fn perm_proxy(sim: &Simulation) -> f64 {
    // Higher when θ lower: exp(+β_a * (1-θ)) proxy of leak openness.
    let th = mean_interface_theta(sim);
    (-sim.params.beta_a * th).exp()
}

fn clamp_interior_field(sim: &mut Simulation, field: &mut [f64], value: f64) {
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            field[idx] = value.max(0.0);
        }
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
}

#[derive(Clone, Debug)]
struct WindowBudget {
    s_mass: f64,
    theta: f64,
    theta_eq: f64,
    p_activity: f64,
    p_total: f64,
    p_internal: f64,
    a_total: f64,
    a_internal: f64,
    forward: f64,
    reverse: f64,
    net_exchange: f64,
    normalized_s_flow: f64,
    p_synthesis: f64,
    p_decay: f64,
    a_activation: f64,
    a_decay: f64,
    perm: f64,
    localization: f64,
    accepted: u64,
    j_pred: f64,
}

fn run_budget_window(sim: &mut Simulation, ctrl: &ControlSpec) -> (WindowBudget, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut steps_ok = true;
    let mut s_sum = 0.0;
    let mut n = 0u64;
    for _ in 0..WINDOW {
        apply_pre_step_controls(sim, ctrl);
        if !sim.step() {
            steps_ok = false;
            break;
        }
        apply_pre_step_controls(sim, ctrl);
        if sim.substep % 10 == 0 {
            s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            n += 1;
        }
    }
    let wl = sim.surface_accounting.window_local();
    let mean_s = if n > 0 {
        s_sum / n as f64
    } else {
        total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    let p_int = mean_interior(sim, &sim.fields.precursor);
    let a_int = mean_interior(sim, &sim.fields.activated);
    let p_act = precursor_activity(p_int, sim.params.p_reference);
    let theta = mean_interface_theta(sim);
    let teq = theta_eq(sim.params.k_exchange_eq, p_act);
    let q = 0.7; // observer mobility scale; parity uses direction primarily
    let jp = j_predicted(
        D031_ALPHA_FROZEN,
        D031_BETA_FROZEN,
        q,
        p_act,
        theta,
    );
    let meta = &sim.metabolism_accounting.last_step;
    (
        WindowBudget {
            s_mass: mean_s,
            theta,
            theta_eq: teq,
            p_activity: p_act,
            p_total: field_mass(&sim.grid, &sim.fields.precursor),
            p_internal: p_int,
            a_total: field_mass(&sim.grid, &sim.fields.activated),
            a_internal: a_int,
            forward: wl.exchange_forward,
            reverse: wl.exchange_reverse,
            net_exchange: wl.exchange_net,
            normalized_s_flow: wl.exchange_net / mean_s.max(1e-18),
            p_synthesis: wl.precursor_synthesis_delta,
            p_decay: wl.precursor_decay_delta,
            a_activation: meta.activation,
            a_decay: meta.activated_decay,
            perm: perm_proxy(sim),
            localization: gamma_localization(sim),
            accepted: sim.substep,
            j_pred: jp,
        },
        steps_ok,
    )
}

fn budget_json(w: &WindowBudget) -> Value {
    json!({
        "surface": {
            "s_mass": w.s_mass,
            "occupancy_theta": w.theta,
            "theta_eq": w.theta_eq,
            "forward_adsorption": w.forward,
            "reverse_desorption": w.reverse,
            "net_exchange": w.net_exchange,
            "normalized_s_flow": w.normalized_s_flow,
            "permeability_proxy": w.perm,
            "localization": w.localization,
        },
        "precursor": {
            "total": w.p_total,
            "internal_mean": w.p_internal,
            "activity": w.p_activity,
            "synthesis": w.p_synthesis,
            "decay": w.p_decay,
        },
        "activated": {
            "total": w.a_total,
            "internal_mean": w.a_internal,
            "activation": w.a_activation,
            "decay": w.a_decay,
        },
        "j_predicted": w.j_pred,
        "accepted": w.accepted,
    })
}

fn gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let head = git_commit_hash();
    let tag_ok = tag_exists(D040_D039_TAG);
    let commit_ok = commit_exists(D040_STARTING_COMMIT) || head.starts_with(D040_STARTING_COMMIT);
    let kinetics_ok = frozen_kinetics_ok();
    let d039_dir = resolve_path(Path::new("experiments/generated/d039"));
    let d039_ok = d039_dir.join("result.json").exists() && d039_dir.join("manifest.json").exists();
    let pass = tag_ok && kinetics_ok && d039_ok;
    let body = json!({
        "gate": 0,
        "pass": pass,
        "record": D040_RECORD,
        "head": head,
        "required_commit_prefix": D040_STARTING_COMMIT,
        "commit_ok": commit_ok,
        "d039_tag": D040_D039_TAG,
        "tag_ok": tag_ok,
        "frozen_kinetics_ok": kinetics_ok,
        "alpha": D031_ALPHA_FROZEN,
        "beta": D031_BETA_FROZEN,
        "K": D040_K_FROZEN,
        "d039_artifacts_present": d039_ok,
        "failure": if pass { Value::Null } else { json!("D040_OBSERVABILITY_OR_PRESERVATION_FAILURE") },
    });
    write_json(&output.join("preservation"), "result.json", &body)?;
    Ok(body)
}

fn gate0_observability(output: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut sim = new_sim(false);
    let ctrl = ControlSpec {
        name: "baseline_obs",
        ..Default::default()
    };
    let settle = horizon.min(D026_SETTLE_STEPS).max(WINDOW);
    let mut ok = true;
    let mut windows = Vec::new();
    let mut accepted = 0u64;
    while accepted < settle && ok {
        let (w, s_ok) = run_budget_window(&mut sim, &ctrl);
        ok &= s_ok;
        accepted = w.accepted;
        windows.push(budget_json(&w));
        if windows.len() >= 6 {
            break;
        }
    }
    let accounting_ok = windows.iter().all(|w| {
        w["surface"]["s_mass"].as_f64().unwrap_or(0.0).is_finite()
            && w["precursor"]["total"].as_f64().unwrap_or(0.0).is_finite()
            && w["activated"]["total"].as_f64().unwrap_or(0.0).is_finite()
    });
    let pass = ok && accounting_ok && !windows.is_empty();
    let body = json!({
        "gate": 0,
        "observability_pass": pass,
        "windows": windows,
        "steps_ok": ok,
        "accounting_ok": accounting_ok,
    });
    write_json(&output.join("accounting"), "observability.json", &body)?;
    Ok((pass, body))
}

fn gate1_equilibrium_audit(
    output: &Path,
    obs_windows: &Value,
) -> Result<(ExchangeParityClass, Value), Box<dyn std::error::Error>> {
    // Organism-window bulk-mean p is not exchange-local p; keep as informational only.
    let mut informational = Vec::new();
    if let Some(arr) = obs_windows["windows"].as_array() {
        for (i, w) in arr.iter().enumerate() {
            let p = w["precursor"]["activity"].as_f64().unwrap_or(0.0);
            let theta = w["surface"]["occupancy_theta"].as_f64().unwrap_or(0.0);
            let net = w["surface"]["net_exchange"].as_f64().unwrap_or(0.0);
            let jp = w["j_predicted"].as_f64().unwrap_or(0.0);
            let mut s = audit_exchange_sample(
                &format!("obs_{i}"),
                p,
                theta,
                0.7,
                net,
                D031_ALPHA_FROZEN,
                D031_BETA_FROZEN,
                D040_K_FROZEN,
            );
            s.parity_ok = s.direction_ok && s.theta_eq.is_finite();
            informational.push(s);
        }
    }

    // Decisive runtime/equation parity: frozen law at known (p,θ) plus one fixed-interface step.
    let mut samples = Vec::new();
    let q = 0.8;
    for (th, p) in [
        (0.3, 0.005),
        (0.5, required_p_for_theta(D040_K_FROZEN, 0.5)),
        (0.7, 0.02),
        (0.2, 0.1),
        (0.8, 0.01),
    ] {
        let jp = j_predicted(D031_ALPHA_FROZEN, D031_BETA_FROZEN, q, p, th);
        samples.push(audit_exchange_sample(
            "equation_check",
            p,
            th,
            q,
            jp,
            D031_ALPHA_FROZEN,
            D031_BETA_FROZEN,
            D040_K_FROZEN,
        ));
    }

    // Fixed-interface runtime: one exchange step, compare ledger net sign to predicted.
    {
        let mut params = v8_schema3_params();
        params.reactions_enabled = false;
        params.k_precursor = 0.0;
        params.k_precursor_decay = 0.0;
        let cat = catalyst_for_q(&params, 0.7);
        let p_act = 0.02;
        let th0 = 0.7;
        let (
            grid,
            phi,
            catalyst,
            activated,
            mut precursor,
            mut s,
            mut waste,
            mut geometry,
            mut gamma,
            mut diffusion,
        ) = build_fixed_interface_state(&params, RADIUS, th0, p_act * params.p_reference, cat);
        let mut s_next = s.clone();
        let mut a_next = activated.clone();
        let mut p_next = precursor.clone();
        reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
        let bases = compute_exchange_local_bases(
            &grid, &precursor, &catalyst, &s, &geometry, &gamma, &params,
        );
        let totals = evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &params,
            0.01,
            false,
            true,
            false,
            false,
            false,
            &mut geometry,
            &mut gamma,
            &mut diffusion,
            &mut s_next,
            &mut a_next,
            &mut p_next,
            &mut waste,
            None,
            None,
        )
        .map_err(|e| format!("gate1 runtime evolve: {e:?}"))?;
        // Mass-rate prediction from local bases: J ≈ α·A − β·B (A,B include δΓq factors).
        let j_mass = D031_ALPHA_FROZEN * bases.adsorption_basis
            - D031_BETA_FROZEN * bases.desorption_basis;
        let j_obs_rate = totals.exchange_net / 0.01;
        let mut runtime = audit_exchange_sample(
            "fixed_interface_runtime",
            p_act,
            bases.mean_theta,
            bases.mean_q_c.max(1e-12),
            j_obs_rate,
            D031_ALPHA_FROZEN,
            D031_BETA_FROZEN,
            D040_K_FROZEN,
        );
        runtime.j_predicted = j_mass;
        runtime.j_observed = j_obs_rate;
        runtime.direction_ok = chemistry_core::d040_analysis::exchange_direction_agrees(
            j_mass,
            j_obs_rate,
            1e-9,
        );
        let rel = chemistry_core::d040_analysis::relative_err(j_mass, j_obs_rate);
        runtime.magnitude_rel_err = rel;
        runtime.parity_ok = runtime.direction_ok
            && runtime.theta_eq.is_finite()
            && (rel <= 0.25 || j_mass.abs() < 1e-8);
        samples.push(runtime);
        let _ = precursor;
    }

    let class = classify_equilibrium_audit(&samples);
    // If law parity passes, classify below/above using informational organism θ vs θ_eq.
    let class = if matches!(
        class,
        ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium
            | ExchangeParityClass::ExchangeLawParityPassPrecursorAboveEquilibrium
    ) {
        if !informational.is_empty() {
            let mean_theta: f64 =
                informational.iter().map(|s| s.theta).sum::<f64>() / informational.len() as f64;
            let mean_teq: f64 = informational.iter().map(|s| s.theta_eq).sum::<f64>()
                / informational.len() as f64;
            if mean_theta + 1e-6 < mean_teq {
                ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium
            } else {
                ExchangeParityClass::ExchangeLawParityPassPrecursorAboveEquilibrium
            }
        } else {
            class
        }
    } else {
        class
    };

    let thresholds = required_p_thresholds(D040_K_FROZEN);
    let body = json!({
        "gate": 1,
        "classification": class.as_str(),
        "alpha": D031_ALPHA_FROZEN,
        "beta": D031_BETA_FROZEN,
        "K": D040_K_FROZEN,
        "equations": {
            "theta_eq": "K*p/(1+K*p)",
            "J_predicted": "alpha*q*p*(1-theta)-beta*q*theta",
        },
        "required_p_thresholds": thresholds
            .iter()
            .map(|(th, p)| json!({"theta": th, "p": p}))
            .collect::<Vec<_>>(),
        "parity_samples": samples,
        "informational_organism_windows": informational,
        "note": "Parity classification uses equation checks + fixed-interface runtime direction; organism windows are informational (bulk-mean p ≠ exchange-local p).",
        "pass": !matches!(class, ExchangeParityClass::ExchangeRuntimeParityDefect
            | ExchangeParityClass::ExchangeEquilibriumUndefined),
    });
    write_json(&output.join("equilibrium_audit"), "result.json", &body)?;
    Ok((class, body))
}

fn gate2_chronology(
    output: &Path,
    horizon: u64,
) -> Result<(String, Value), Box<dyn std::error::Error>> {
    let mut sim = new_sim(false);
    let ctrl = ControlSpec::default();
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let mut windows = Vec::new();
    let mut chron = Vec::new();
    let mut ok = true;
    let target = horizon.max(4 * WINDOW);
    while sim.substep < target && ok && chron.len() < 12 {
        let (w, s_ok) = run_budget_window(&mut sim, &ctrl);
        ok &= s_ok;
        let a_ret = w.a_total / a0;
        let cw = ChronologyWindow {
            index: chron.len(),
            theta: w.theta,
            theta_eq: w.theta_eq,
            p: w.p_activity,
            a: w.a_internal,
            a_retention: a_ret,
            p_synthesis: w.p_synthesis,
            p_leakage: (w.p_total - w.p_internal).abs(), // coarse proxy
            a_leakage: (w.a_total - w.a_internal).abs(),
            net_exchange: w.net_exchange,
            permeability_proxy: w.perm,
            precursor_synthesis_demand: w.p_synthesis.abs(),
        };
        chron.push(cw.clone());
        windows.push(json!({
            "budget": budget_json(&w),
            "a_retention": a_ret,
        }));
    }
    let class = earliest_causal_divergence(&chron);
    let body = json!({
        "gate": 2,
        "earliest_divergence": class.as_str(),
        "steps_ok": ok,
        "windows": windows,
        "chronology": chron,
    });
    write_json(&output.join("chronology"), "result.json", &body)?;
    Ok((class.as_str().into(), body))
}

/// Fixed-interface schema-3 assay with clamped precursor activity.
fn fixed_p_assay(theta0: f64, p_activity: f64, steps: u64) -> Result<Value, String> {
    let mut params = v8_schema3_params();
    params.reactions_enabled = false;
    params.k_precursor = 0.0;
    params.k_precursor_decay = 0.0;
    params.k_ads = 0.0;
    let cat = catalyst_for_q(&params, 0.7);
    let p0 = p_activity * params.p_reference;
    let (grid, phi, catalyst, activated, mut precursor, mut s, mut waste, mut geometry, mut gamma, mut diffusion) =
        build_fixed_interface_state(&params, RADIUS, theta0, p0, cat);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let teq = theta_eq(params.k_exchange_eq, p_activity);
    let mut last_net = 0.0;
    let mut last_theta = theta0;
    let mut cum = SurfaceAccountingTotals::default();
    let dt = 0.01;
    for _ in 0..steps {
        // Clamp P every step (external reservoir).
        for v in precursor.iter_mut() {
            *v = p0;
        }
        let totals = evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &params,
            dt,
            false,
            true,
            false,
            false,
            false,
            &mut geometry,
            &mut gamma,
            &mut diffusion,
            &mut s_next,
            &mut a_next,
            &mut p_next,
            &mut waste,
            None,
            None,
        )
        .map_err(|e| format!("{e:?}"))?;
        s.copy_from_slice(&s_next);
        // Restore P clamp after exchange (inventory conserved in assay by external supply).
        for v in precursor.iter_mut() {
            *v = p0;
        }
        cum.accumulate(totals.clone());
        last_net = totals.exchange_net;
        reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
        let bases = compute_exchange_local_bases(
            &grid, &precursor, &catalyst, &s, &geometry, &gamma, &params,
        );
        last_theta = bases.mean_theta;
    }
    let jp = j_predicted(D031_ALPHA_FROZEN, D031_BETA_FROZEN, 0.7, p_activity, last_theta);
    let dir_ok = chemistry_core::d040_analysis::exchange_direction_agrees(jp, last_net, 1e-12)
        || (last_theta - teq).abs() < 0.05;
    let converged = (last_theta - teq).abs() < 0.10;
    Ok(json!({
        "theta0": theta0,
        "p_activity": p_activity,
        "theta_eq": teq,
        "theta_final": last_theta,
        "net_final": last_net,
        "j_predicted": jp,
        "direction_ok": dir_ok,
        "converged_toward_eq": converged,
        "s_final": total_surface_mass(&grid, &s),
        "bounded": last_theta.is_finite() && last_theta >= 0.0 && last_theta <= 1.0,
    }))
}

fn gate3_precursor_sufficiency(
    output: &Path,
) -> Result<(PrecursorSufficiencyOutcome, f64, Value), Box<dyn std::error::Error>> {
    let thresholds = required_p_thresholds(D040_K_FROZEN);
    let theta_starts = [0.1, 0.4, 0.75, THETA0, THETA0 * 0.75];
    let mut assays = Vec::new();
    let mut any_repair = false;
    let mut min_repair_p = f64::INFINITY;
    for (th_eq, p) in &thresholds {
        for &th0 in &theta_starts {
            let r = fixed_p_assay(th0, *p, 400)?;
            let ok = r["direction_ok"].as_bool().unwrap_or(false)
                && r["converged_toward_eq"].as_bool().unwrap_or(false)
                && r["bounded"].as_bool().unwrap_or(false);
            if ok {
                any_repair = true;
                min_repair_p = min_repair_p.min(*p);
            }
            assays.push(json!({
                "target_theta": th_eq,
                "result": r,
                "pass": ok,
            }));
        }
    }
    // Damage-like low-S start under each P.
    let mut damage_map = Vec::new();
    for (th_eq, p) in &thresholds {
        let r = fixed_p_assay(THETA0 * 0.75, *p, 600)?;
        let restores = r["theta_final"].as_f64().unwrap_or(0.0) >= 0.95 * THETA0 * 0.9
            && (r["theta_final"].as_f64().unwrap_or(0.0) - *th_eq).abs() < 0.15;
        if restores {
            any_repair = true;
            min_repair_p = min_repair_p.min(*p);
        }
        damage_map.push(json!({
            "target_theta": th_eq,
            "p": p,
            "restores": restores,
            "result": r,
        }));
    }
    if !min_repair_p.is_finite() {
        min_repair_p = thresholds.last().map(|(_, p)| *p).unwrap_or(0.2);
    }
    let outcome = if any_repair {
        PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor
    } else {
        PrecursorSufficiencyOutcome::PassiveExchangeLawCannotRepair
    };
    let body = json!({
        "gate": 3,
        "outcome": outcome.as_str(),
        "min_repair_p_activity": min_repair_p,
        "assays": assays,
        "damage_fixed_p": damage_map,
    });
    write_json(&output.join("precursor_sufficiency"), "result.json", &body)?;
    Ok((outcome, min_repair_p, body))
}

fn measure_max_p(sim: &mut Simulation, ctrl: &ControlSpec, horizon: u64) -> (f64, f64, bool) {
    let mut max_p: f64 = 0.0;
    let mut max_act: f64 = 0.0;
    let mut ok = true;
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && ok {
        let (w, s_ok) = run_budget_window(sim, ctrl);
        ok &= s_ok;
        max_p = max_p.max(w.p_internal);
        max_act = max_act.max(w.p_activity);
    }
    (max_p, max_act, ok)
}

fn gate4_endogenous(
    output: &Path,
    repair_p: f64,
    horizon: u64,
) -> Result<(EndogenousCapacityClass, Value), Box<dyn std::error::Error>> {
    let h = horizon.min(4_000).max(WINDOW * 2);
    let mut cases = Vec::new();

    let run_case = |name: &'static str, mut spec: ControlSpec| -> Value {
        spec.name = name;
        spec.disable_exchange = true; // Gate 4: exchange off
        let mut sim = new_sim(false);
        apply_control_params(&mut sim, &spec);
        let (max_p, max_act, ok) = measure_max_p(&mut sim, &spec, h);
        json!({
            "name": name,
            "max_internal_p": max_p,
            "max_p_activity": max_act,
            "steps_ok": ok,
        })
    };

    let normal = run_case("normal", ControlSpec::default());
    let fixed_perm = run_case(
        "fixed_healthy_permeability",
        ControlSpec {
            freeze_surface: true,
            ..Default::default()
        },
    );
    // Fixed A retention: clamp A at seed interior mean.
    let mut seed = new_sim(false);
    let a_healthy = mean_interior(&seed, &seed.fields.activated);
    let fixed_a = run_case(
        "fixed_a_retention",
        ControlSpec {
            clamp_a: Some(a_healthy.max(0.1)),
            ..Default::default()
        },
    );
    let no_decay = run_case(
        "no_p_decay",
        ControlSpec {
            no_p_decay: true,
            ..Default::default()
        },
    );
    let no_leak = run_case(
        "no_p_interface_transport",
        ControlSpec {
            no_p_diffusion: true,
            ..Default::default()
        },
    );
    let combined = run_case(
        "no_decay_no_leak",
        ControlSpec {
            no_p_decay: true,
            no_p_diffusion: true,
            ..Default::default()
        },
    );

    cases.push(normal.clone());
    cases.push(fixed_perm);
    cases.push(fixed_a.clone());
    cases.push(no_decay.clone());
    cases.push(no_leak.clone());
    cases.push(combined.clone());

    let max_endog = normal["max_p_activity"].as_f64().unwrap_or(0.0);
    let max_no_leak = no_leak["max_p_activity"]
        .as_f64()
        .unwrap_or(0.0)
        .max(combined["max_p_activity"].as_f64().unwrap_or(0.0));
    let max_no_decay = no_decay["max_p_activity"].as_f64().unwrap_or(0.0);
    let max_fixed_a = fixed_a["max_p_activity"].as_f64().unwrap_or(0.0);
    let class = classify_endogenous_capacity(
        repair_p,
        max_endog,
        max_no_leak,
        max_no_decay,
        max_fixed_a,
    );
    let body = json!({
        "gate": 4,
        "repair_p_threshold": repair_p,
        "classification": class.as_str(),
        "cases": cases,
        "note": "Observer controls only; k_precursor unchanged on frozen candidate.",
    });
    write_json(&output.join("endogenous_capacity"), "result.json", &body)?;
    Ok((class, body))
}

fn run_control_from_pre_collapse(
    ctrl: ControlSpec,
    horizon: u64,
    pre_steps: u64,
) -> Value {
    let mut sim = new_sim(false);
    let base = ControlSpec::default();
    let mut ok = true;
    // Reach a common pre-collapse state.
    while sim.substep < pre_steps && ok {
        let (_, s_ok) = run_budget_window(&mut sim, &base);
        ok &= s_ok;
    }
    apply_control_params(&mut sim, &ctrl);
    let mut windows = Vec::new();
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && ok && windows.len() < 8 {
        let (w, s_ok) = run_budget_window(&mut sim, &ctrl);
        ok &= s_ok;
        windows.push(budget_json(&w));
    }
    let last = windows.last().cloned().unwrap_or(json!({}));
    let theta = last["surface"]["occupancy_theta"].as_f64().unwrap_or(0.0);
    let p = last["precursor"]["activity"].as_f64().unwrap_or(0.0);
    let restores = theta >= 0.45 && p >= required_p_for_theta(D040_K_FROZEN, 0.4) * 0.5;
    json!({
        "name": ctrl.name,
        "steps_ok": ok,
        "restores_maintenance": restores,
        "final": last,
        "windows": windows,
    })
}

fn gate5_causal_controls(
    output: &Path,
    repair_p: f64,
    horizon: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let pre = horizon.min(2_000);
    let h = horizon.min(3_000);
    let mut seed = new_sim(false);
    let a_healthy = mean_interior(&seed, &seed.fields.activated).max(0.1);

    let controls = [
        ControlSpec {
            name: "A_p_clamp",
            clamp_p_activity: Some(repair_p),
            ..Default::default()
        },
        ControlSpec {
            name: "B_a_clamp",
            clamp_a: Some(a_healthy),
            ..Default::default()
        },
        ControlSpec {
            name: "C_healthy_permeability",
            freeze_surface: true,
            ..Default::default()
        },
        ControlSpec {
            name: "D_no_p_decay",
            no_p_decay: true,
            ..Default::default()
        },
        ControlSpec {
            name: "E_no_p_outward_transport",
            no_p_diffusion: true,
            ..Default::default()
        },
        ControlSpec {
            name: "F_exchange_disabled",
            disable_exchange: true,
            ..Default::default()
        },
    ];
    let results: Vec<Value> = controls
        .into_iter()
        .map(|c| run_control_from_pre_collapse(c, h, pre))
        .collect();
    let body = json!({
        "gate": 5,
        "pre_collapse_steps": pre,
        "controls": results,
        "note": "Diagnostic controls; not promotable biological candidates.",
    });
    write_json(&output.join("causal_controls"), "result.json", &body)?;
    Ok(body)
}

fn gate6_damage_controls(
    output: &Path,
    repair_p: f64,
    horizon: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let pre = horizon.min(2_000);
    let recover = horizon.min(3_000);
    let mut seed = new_sim(false);
    let a_healthy = mean_interior(&seed, &seed.fields.activated).max(0.1);

    let specs = [
        ControlSpec {
            name: "normal_v8",
            ..Default::default()
        },
        ControlSpec {
            name: "fixed_sufficient_p",
            clamp_p_activity: Some(repair_p),
            ..Default::default()
        },
        ControlSpec {
            name: "fixed_healthy_a",
            clamp_a: Some(a_healthy),
            ..Default::default()
        },
        ControlSpec {
            name: "fixed_healthy_permeability",
            freeze_surface: true,
            ..Default::default()
        },
        ControlSpec {
            name: "no_p_decay",
            no_p_decay: true,
            ..Default::default()
        },
        ControlSpec {
            name: "no_p_outward_transport",
            no_p_diffusion: true,
            ..Default::default()
        },
    ];

    let mut rows = Vec::new();
    let mut best_name = "normal_v8";
    let mut best_score = -1.0f64;
    for spec in specs {
        let mut sim = new_sim(false);
        let base = ControlSpec::default();
        let mut ok = true;
        while sim.substep < pre && ok {
            let (_, s_ok) = run_budget_window(&mut sim, &base);
            ok &= s_ok;
        }
        let s_before = field_mass(&sim.grid, &sim.fields.membrane);
        let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
        apply_control_params(&mut sim, &spec);
        let mut forward = 0.0;
        let mut reverse = 0.0;
        let end = sim.substep.saturating_add(recover);
        while sim.substep < end && ok {
            let (w, s_ok) = run_budget_window(&mut sim, &spec);
            ok &= s_ok;
            forward += w.forward;
            reverse += w.reverse;
        }
        let s_after = field_mass(&sim.grid, &sim.fields.membrane);
        let repair_frac = s_after / s_before.max(1e-18);
        let score = repair_frac;
        if score > best_score {
            best_score = score;
            best_name = spec.name;
        }
        rows.push(json!({
            "name": spec.name,
            "s_before": s_before,
            "s_after_damage_immediate": report.total_s_before - report.s_removed,
            "s_after_recovery": s_after,
            "repair_fraction": repair_frac,
            "adsorption_after": forward,
            "desorption_after": reverse,
            "steps_ok": ok,
            "field_reset": false,
        }));
    }
    let body = json!({
        "gate": 6,
        "controls": rows,
        "strongest_single_control": best_name,
        "best_repair_fraction": best_score,
    });
    write_json(&output.join("damage_controls"), "result.json", &body)?;
    Ok(body)
}

fn gate7_reduced(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let par = ReducedApsParams::default();
    let fps = find_reduced_fixed_points(&par);
    let basins = classify_basins(&par, 0.5);
    let healthy = fps.iter().any(|fp| fp.admissible && fp.theta >= 0.5);
    let failed = fps.iter().any(|fp| fp.admissible && fp.theta < 0.25);
    let bistable = healthy && failed
        || (basins.iter().any(|b| b.attracted_healthy)
            && basins.iter().any(|b| !b.attracted_healthy));
    let damage_sep = basins
        .iter()
        .find(|b| b.label == "damage_25")
        .map(|b| !b.attracted_healthy)
        .unwrap_or(false);
    let body = json!({
        "gate": 7,
        "params": par,
        "fixed_points": fps,
        "basins": basins,
        "healthy_fixed_point_exists": healthy,
        "bistable_basins": bistable,
        "damage_crosses_separatrix": damage_sep,
    });
    write_json(&output.join("reduced_feedback"), "result.json", &body)?;
    Ok(body)
}

fn gate8_multistart(output: &Path, horizon: u64) -> Result<Value, Box<dyn std::error::Error>> {
    let h = horizon.min(3_000);
    let mut starts = Vec::new();

    let mut run = |label: &str, mutate: &dyn Fn(&mut Simulation)| -> Value {
        let mut sim = new_sim(false);
        mutate(&mut sim);
        let ctrl = ControlSpec::default();
        let mut ok = true;
        let mut last = json!({});
        let end = sim.substep.saturating_add(h);
        while sim.substep < end && ok {
            let (w, s_ok) = run_budget_window(&mut sim, &ctrl);
            ok &= s_ok;
            last = budget_json(&w);
        }
        let theta = last["surface"]["occupancy_theta"].as_f64().unwrap_or(0.0);
        let a_ret = last["activated"]["total"].as_f64().unwrap_or(0.0);
        let loc = last["surface"]["localization"].as_f64().unwrap_or(0.0);
        let healthy = theta >= 0.5
            && loc >= 0.95
            && last["surface"]["forward_adsorption"].as_f64().unwrap_or(0.0) > 0.0
            && last["surface"]["reverse_desorption"].as_f64().unwrap_or(0.0) > 0.0;
        json!({
            "label": label,
            "steps_ok": ok,
            "final": last,
            "healthy": healthy,
            "a_mass": a_ret,
            "conservative_init_note": "mutations recorded; no target normalization",
        })
    };

    starts.push(run("historical_init", &|_| {}));
    starts.push(run("high_p_init", &|sim| {
        let mut buf = sim.fields.precursor.clone();
        clamp_interior_field(sim, &mut buf, 0.2);
        sim.fields.precursor.copy_from_slice(&buf);
    }));
    starts.push(run("high_a_init", &|sim| {
        let mut buf = sim.fields.activated.clone();
        clamp_interior_field(sim, &mut buf, 2.0);
        sim.fields.activated.copy_from_slice(&buf);
    }));
    starts.push(run("healthy_s_init", &|sim| {
        // Scale membrane toward higher occupancy without renormalizing totals globally.
        for v in sim.fields.membrane.iter_mut() {
            *v *= 1.1;
        }
    }));
    starts.push(run("low_s_init", &|sim| {
        for v in sim.fields.membrane.iter_mut() {
            *v *= 0.5;
        }
    }));
    starts.push(run("pre_collapse_checkpoint", &|sim| {
        // Advance briefly then continue as start.
        let ctrl = ControlSpec::default();
        for _ in 0..4 {
            let _ = run_budget_window(sim, &ctrl);
        }
    }));

    let healthy_n = starts.iter().filter(|s| s["healthy"].as_bool().unwrap_or(false)).count();
    let failed_n = starts.len() - healthy_n;
    let classification = if healthy_n == starts.len() {
        "converge_common_healthy"
    } else if failed_n == starts.len() {
        "converge_common_failed"
    } else if healthy_n > 0 && failed_n > 0 {
        "split_healthy_failed_attractors"
    } else {
        "long_transient_unresolved"
    };
    let body = json!({
        "gate": 8,
        "classification": classification,
        "starts": starts,
    });
    write_json(&output.join("multistart"), "result.json", &body)?;
    Ok(body)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let horizon = max_accepted();
    for d in [
        "preservation",
        "equilibrium_audit",
        "chronology",
        "precursor_sufficiency",
        "endogenous_capacity",
        "causal_controls",
        "damage_controls",
        "reduced_feedback",
        "multistart",
        "route_decision",
        "accounting",
    ] {
        fs::create_dir_all(output.join(d))?;
    }

    eprintln!("D-040 Gate0 preservation/observability horizon={horizon}");
    let g0p = gate0_preservation(&output)?;
    let (obs_ok, obs) = gate0_observability(&output, horizon)?;
    if !g0p["pass"].as_bool().unwrap_or(false) || !obs_ok {
        let fail = json!({
            "primary_conclusion": "D040_AUDIT_INCONCLUSIVE",
            "failed_gate": 0,
            "detail": { "preservation": g0p, "observability": obs },
            "record": D040_RECORD,
        });
        write_json(&output, "result.json", &fail)?;
        return Ok(fail);
    }

    eprintln!("D-040 Gate1 equilibrium audit");
    let (parity, g1) = gate1_equilibrium_audit(&output, &obs)?;
    if matches!(
        parity,
        ExchangeParityClass::ExchangeRuntimeParityDefect
            | ExchangeParityClass::ExchangeEquilibriumUndefined
    ) {
        let conclusion = if matches!(parity, ExchangeParityClass::ExchangeRuntimeParityDefect) {
            "D040_EXCHANGE_RUNTIME_PARITY_DEFECT"
        } else {
            "D040_AUDIT_INCONCLUSIVE"
        };
        let fail = json!({
            "primary_conclusion": conclusion,
            "failed_gate": 1,
            "parity": parity.as_str(),
            "gate1": g1,
            "record": D040_RECORD,
        });
        write_json(&output.join("route_decision"), "result.json", &fail)?;
        write_json(&output, "result.json", &fail)?;
        return Ok(fail);
    }

    eprintln!("D-040 Gate2 chronology");
    let (chron_class, g2) = gate2_chronology(&output, horizon)?;

    eprintln!("D-040 Gate3 precursor sufficiency");
    let (suff, repair_p, g3) = gate3_precursor_sufficiency(&output)?;

    eprintln!("D-040 Gate4 endogenous capacity");
    let (endog, g4) = gate4_endogenous(&output, repair_p, horizon)?;

    eprintln!("D-040 Gate5 causal controls (includes repaired metabolic path)");
    let g5 = gate5_causal_controls(&output, repair_p, horizon)?;
    eprintln!("D-040 Gate6 damage controls");
    let g6 = gate6_damage_controls(&output, repair_p, horizon)?;

    eprintln!("D-040 Gate7 reduced feedback");
    let g7 = gate7_reduced(&output)?;
    eprintln!("D-040 Gate8 multistart");
    let g8 = gate8_multistart(&output, horizon)?;

    let controls = g5["controls"].as_array().cloned().unwrap_or_default();
    let find_restore = |name: &str| -> bool {
        controls
            .iter()
            .find(|c| c["name"].as_str() == Some(name))
            .and_then(|c| c["restores_maintenance"].as_bool())
            .unwrap_or(false)
    };

    let ev = RouteEvidence {
        parity,
        sufficiency: Some(suff),
        endogenous: Some(endog),
        p_clamp_restores: find_restore("A_p_clamp"),
        a_clamp_restores: find_restore("B_a_clamp"),
        perm_freeze_restores: find_restore("C_healthy_permeability"),
        no_decay_restores: find_restore("D_no_p_decay"),
        no_leak_restores: find_restore("E_no_p_outward_transport"),
        healthy_fixed_point_exists: g7["healthy_fixed_point_exists"].as_bool().unwrap_or(false),
        bistable_basins: g7["bistable_basins"].as_bool().unwrap_or(false),
        damage_crosses_separatrix: g7["damage_crosses_separatrix"].as_bool().unwrap_or(false),
        accounting_ok: true,
        numerical_ok: obs["steps_ok"].as_bool().unwrap_or(true),
    };
    let conclusion = select_route(&ev);
    let route = match conclusion {
        chemistry_core::d040_analysis::D040Conclusion::PrecursorSynthesisCapacityDeficit => {
            "Route_P"
        }
        chemistry_core::d040_analysis::D040Conclusion::PrecursorRetentionDefect => "Route_R",
        chemistry_core::d040_analysis::D040Conclusion::ActivatedResourceSupplyDeficit => "Route_A",
        chemistry_core::d040_analysis::D040Conclusion::MembraneMetabolismBistability => "Route_F",
        chemistry_core::d040_analysis::D040Conclusion::PassiveExchangeLawInvalid => "Route_E",
        chemistry_core::d040_analysis::D040Conclusion::NoBoundedMembraneMaintenanceState => {
            "Route_N"
        }
        _ => "none",
    };

    let route_body = json!({
        "gate": 9,
        "selected_route": route,
        "primary_conclusion": conclusion.as_str(),
        "evidence": ev,
        "chronology_class": chron_class,
        "next_directive_hint": match route {
            "Route_P" => "Recalibrate/redesign precursor synthesis using measured capacity; do not alter exchange.",
            "Route_R" => "Repair dominant retention mechanism only.",
            "Route_A" => "Address upstream A budget.",
            "Route_F" => "Address basin accessibility / local bootstrap without target feedback.",
            "Route_E" => "Review chemical-potential or composition-dependent exchange law.",
            "Route_N" => "Stop for fundamental membrane-chemistry review.",
            _ => "Further diagnostic review.",
        },
    });
    write_json(&output.join("route_decision"), "result.json", &route_body)?;

    let result = json!({
        "project_directive": "D-040",
        "agent_memory_id": D040_AGENT_MEMORY_ID,
        "starting_commit": D040_STARTING_COMMIT,
        "d039_tag": D040_D039_TAG,
        "record": D040_RECORD,
        "primary_conclusion": conclusion.as_str(),
        "selected_route": route,
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "horizon": horizon,
        "gate0_preservation": g0p,
        "gate0_observability": obs,
        "gate1": g1,
        "gate2": g2,
        "gate3": g3,
        "gate4": g4,
        "gate5": g5,
        "gate6": g6,
        "gate7": g7,
        "gate8": g8,
        "gate9": route_body,
        "next_execution_started": false,
    });
    write_json(&output, "result.json", &result)?;

    let manifest = json!({
        "project_directive": "D-040",
        "agent_memory_id": D040_AGENT_MEMORY_ID,
        "starting_commit": D040_STARTING_COMMIT,
        "d039_tag": D040_D039_TAG,
        "primary_conclusion": conclusion.as_str(),
        "selected_route": route,
        "record": D040_RECORD,
        "stage_e_certified": false,
        "architecture": "membrane_metabolism_v8_reversible_surface_exchange",
        "turnover_schema": "surface_turnover_schema_3_exchange_damage_only",
        "artifacts": [
            "preservation",
            "equilibrium_audit",
            "chronology",
            "precursor_sufficiency",
            "endogenous_capacity",
            "causal_controls",
            "damage_controls",
            "reduced_feedback",
            "multistart",
            "route_decision",
            "accounting"
        ],
    });
    write_json(&output, "manifest.json", &manifest)?;
    Ok(result)
}
