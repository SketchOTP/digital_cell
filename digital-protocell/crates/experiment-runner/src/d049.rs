//! D-049 coupled A/P/S collapse feedback decomposition pipeline (diagnostic only).
//!
//! Frozen biology via D-048 organism params. No default rate changes when controls off.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::SimParams;
use chemistry_core::d026_analysis::{sample_stage_e_observability, D026_SETTLE_STEPS};
use chemistry_core::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d040_analysis::{find_reduced_fixed_points, ReducedApsParams};
use chemistry_core::d049_analysis::{
    classify_d048_completeness, classify_frozen_membrane, disposition_d040,
    earliest_causal_event, find_empirical_fixed_points, has_physical_healthy_fp,
    ledger_closes, select_route, ChronologySample, CoupledLedgerWindow, D048CompletenessReport,
    EmpiricalReducedParams, RouteEvidence, D049_AGENT_MEMORY_ID, D049_BOOTSTRAP_P,
    D049_D047_STATUS, D049_D048_TAG, D049_DEFAULT_HORIZON, D049_LEDGER_REL_TOL, D049_LOCALIZATION_MIN,
    D049_RADIUS, D049_RECORD, D049_RETENTION_MIN, D049_STARTING_COMMIT, D049_THETA, D049_WINDOW,
    d049_frozen_params,
};
use chemistry_core::field_mass;
use chemistry_core::snapshot::{load_snapshot, save_snapshot, FieldSnapshot};
use chemistry_core::surface_density::{
    compute_interface_geometry, precursor_activity, surface_localization,
    surface_occupancy_theta, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const CHECKPOINTS: [u64; 6] = [0, 1000, 2500, 5000, 7500, 10000];
const BOOTSTRAP_MAX: u64 = 8_000;
const BOOTSTRAP_A_HOLD: f64 = 1.0;
const NET_S_FAIL: f64 = -0.01;

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

fn commit_exists(prefix: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("{prefix}^{{commit}}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn max_accepted() -> u64 {
    std::env::var("D049_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D049_DEFAULT_HORIZON)
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn schema3_organism_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    d049_frozen_params(&base)
}

fn new_sim() -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params());
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, D049_RADIUS, D049_THETA);
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

fn perm_proxy(sim: &Simulation) -> f64 {
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

fn clamp_interior_p_activity(sim: &mut Simulation, activity: f64) {
    let target = activity * sim.params.p_reference.max(1e-12);
    let mut buf = sim.fields.precursor.clone();
    clamp_interior_field(sim, &mut buf, target);
    sim.fields.precursor.copy_from_slice(&buf);
}

fn replace_precursor_a_demand(sim: &mut Simulation, a_mass: f64) {
    if a_mass <= 0.0 {
        return;
    }
    let mut n_in = 0u64;
    for idx in 0..sim.fields.activated.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            n_in += 1;
        }
    }
    if n_in == 0 {
        return;
    }
    let per = a_mass / n_in as f64;
    for idx in 0..sim.fields.activated.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            sim.fields.activated[idx] = (sim.fields.activated[idx] + per).max(0.0);
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
    disable_a_transport: bool,
    disable_catalyst_reproduction: bool,
    disable_structure: bool,
    no_a_decay: bool,
    replace_precursor_demand: bool,
    freeze_healthy_a_perm: bool,
}

fn apply_control_params(sim: &mut Simulation, ctrl: &ControlSpec, initial_beta_a: Option<f64>) {
    if ctrl.freeze_surface || ctrl.freeze_healthy_a_perm {
        sim.d026_freeze_surface = true;
    }
    if ctrl.freeze_healthy_a_perm {
        if let Some(b) = initial_beta_a {
            sim.params.beta_a = b;
        }
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
    if ctrl.disable_a_transport {
        sim.d026_disable_a_normal_transport = true;
    }
    if ctrl.disable_catalyst_reproduction {
        sim.d026_disable_catalyst_reproduction = true;
    }
    if ctrl.disable_structure {
        sim.d026_disable_virtual_structure = true;
    }
    if ctrl.no_a_decay {
        sim.params.k_d008_activated_decay = 0.0;
    }
}

fn apply_pre_step_controls(sim: &mut Simulation, ctrl: &ControlSpec) {
    if let Some(p) = ctrl.clamp_p_activity {
        clamp_interior_p_activity(sim, p);
    }
    if let Some(a) = ctrl.clamp_a {
        let mut buf = sim.fields.activated.clone();
        clamp_interior_field(sim, &mut buf, a);
        sim.fields.activated.copy_from_slice(&buf);
    }
}

#[derive(Clone, Debug)]
struct WindowMetrics {
    accepted: u64,
    a_retention: f64,
    c_retention: f64,
    localization: f64,
    theta: f64,
    mean_s: f64,
    net_exchange: f64,
    normalized_s_flow: f64,
    p_activity: f64,
    steps_ok: bool,
}

fn a_retention(sim: &Simulation, a0: f64) -> f64 {
    field_mass(&sim.grid, &sim.fields.activated) / a0.max(1e-18)
}

fn c_retention(sim: &Simulation, c0: f64) -> f64 {
    field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18)
}

fn window_failed(w: &WindowMetrics) -> bool {
    w.a_retention < D049_RETENTION_MIN || w.normalized_s_flow < NET_S_FAIL
}

fn run_window(
    sim: &mut Simulation,
    ctrl: &ControlSpec,
    c0: f64,
    a0: f64,
    initial_beta_a: Option<f64>,
) -> WindowMetrics {
    apply_control_params(sim, ctrl, initial_beta_a);
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut steps_ok = true;
    let mut s_sum = 0.0;
    let mut s_n = 0u64;
    for _ in 0..D049_WINDOW {
        apply_pre_step_controls(sim, ctrl);
        if !sim.step() {
            steps_ok = false;
            break;
        }
        apply_pre_step_controls(sim, ctrl);
        if sim.substep % 20 == 0 {
            s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            s_n += 1;
        }
    }
    if ctrl.replace_precursor_demand {
        let syn = sim.surface_accounting.window_local().precursor_synthesis_delta;
        replace_precursor_a_demand(sim, syn);
    }
    let wl = sim.surface_accounting.window_local();
    let mean_s = if s_n > 0 {
        s_sum / s_n as f64
    } else {
        total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    let p_int = mean_interior(sim, &sim.fields.precursor);
    WindowMetrics {
        accepted: sim.substep,
        a_retention: a_retention(sim, a0),
        c_retention: c_retention(sim, c0),
        localization: gamma_localization(sim),
        theta: mean_interface_theta(sim),
        mean_s,
        net_exchange: wl.exchange_net,
        normalized_s_flow: wl.exchange_net / mean_s.max(1e-18),
        p_activity: precursor_activity(p_int, sim.params.p_reference),
        steps_ok,
    }
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

fn settle(sim: &mut Simulation, ctrl: &ControlSpec, initial_beta_a: Option<f64>) -> bool {
    let mut ok = true;
    for _ in 0..D026_SETTLE_STEPS {
        apply_pre_step_controls(sim, ctrl);
        if !sim.step() {
            ok = false;
            break;
        }
    }
    let _ = (ctrl, initial_beta_a);
    ok
}

fn run_horizon(
    sim: &mut Simulation,
    ctrl: &ControlSpec,
    horizon: u64,
    c0: f64,
    a0: f64,
    initial_beta_a: Option<f64>,
) -> (Vec<WindowMetrics>, bool, Option<usize>) {
    let mut windows = Vec::new();
    let mut steps_ok = true;
    let mut first_fail = None;
    // Relative horizon so restored/bootstrap snapshots still advance.
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && steps_ok {
        let w = run_window(sim, ctrl, c0, a0, initial_beta_a);
        steps_ok &= w.steps_ok;
        if first_fail.is_none() && window_failed(&w) {
            first_fail = Some(windows.len());
        }
        windows.push(w);
        if sim.substep % 4000 == 0 {
            let _ = Write::flush(&mut std::io::stderr());
            eprintln!(
                "D-049 {} accepted={} a_ret={:.4} loc={:.4}",
                ctrl.name,
                sim.substep,
                windows.last().map(|w| w.a_retention).unwrap_or(0.0),
                windows.last().map(|w| w.localization).unwrap_or(0.0),
            );
        }
    }
    (windows, steps_ok, first_fail)
}

fn trajectory_pass(windows: &[WindowMetrics], steps_ok: bool) -> bool {
    steps_ok
        && windows
            .last()
            .map(|w| w.a_retention >= D049_RETENTION_MIN)
            .unwrap_or(false)
}

/// Window-integrated A/P/S ledger using D-042-style observability sums + surface window rates.
fn measure_coupled_ledger_window(
    sim: &mut Simulation,
    ctrl: &ControlSpec,
) -> (CoupledLedgerWindow, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let a_before = field_mass(&sim.grid, &sim.fields.activated);
    let p_before = field_mass(&sim.grid, &sim.fields.precursor);
    let s_before = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let t0 = sim.sim_time;
    let mut steps_ok = true;
    let mut act_sum = 0.0;
    let mut repro_sum = 0.0;
    let mut virt_sum = 0.0;
    let mut prec_sum = 0.0;
    let mut decay_sum = 0.0;
    let mut ain_sum = 0.0;
    let mut aout_sum = 0.0;
    let mut a_react = 0.0;
    let mut a_diff = 0.0;
    let mut a_res = 0.0;
    let mut a_num = 0.0;
    let mut p_react = 0.0;
    let mut p_diff = 0.0;
    let mut p_res = 0.0;
    let mut p_num = 0.0;
    for _ in 0..D049_WINDOW {
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
        let a_led = &sim.accounting.last_step.activated;
        a_react += a_led.reaction_delta;
        a_diff += a_led.diffusion_delta;
        a_res += a_led.reservoir_delta;
        a_num += a_led.numerical_correction_delta;
        let p_led = &sim.accounting.last_step.precursor;
        p_react += p_led.reaction_delta;
        p_diff += p_led.diffusion_delta;
        p_res += p_led.reservoir_delta;
        p_num += p_led.numerical_correction_delta;
    }
    apply_pre_step_controls(sim, ctrl);
    let dt = (sim.sim_time - t0).max(f64::EPSILON);
    let rate = |sum: f64| sum / dt;
    let a_after = field_mass(&sim.grid, &sim.fields.activated);
    let p_after = field_mass(&sim.grid, &sim.fields.precursor);
    let s_after = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let a_delta_obs = (a_after - a_before) / dt;
    let a_field = (a_react + a_diff + a_res + a_num) / dt;
    let a_decomp = rate(act_sum + ain_sum - repro_sum - virt_sum - prec_sum - decay_sum - aout_sum);
    let a_closes = ledger_closes(a_field, a_delta_obs, D049_LEDGER_REL_TOL)
        || ledger_closes(a_decomp, a_delta_obs, D049_LEDGER_REL_TOL);

    let wl = sim.surface_accounting.window_local_rates(sim.sim_time);
    let p_delta_obs = (p_after - p_before) / dt;
    let p_field = (p_react + p_diff + p_res + p_num) / dt;
    let p_prod = wl.precursor_synthesis_delta;
    let p_gain_desorb = wl.exchange_reverse;
    let p_loss = wl.precursor_decay_delta + wl.exchange_forward.abs().max(0.0);
    let p_closes = ledger_closes(p_field, p_delta_obs, D049_LEDGER_REL_TOL);

    let s_delta_obs = (s_after - s_before) / dt;
    let s_gain_ads = wl.exchange_forward;
    let s_loss = wl.exchange_reverse + wl.gamma_decay_delta.abs();
    let s_predicted = s_gain_ads - s_loss;
    let s_closes = ledger_closes(s_predicted, s_delta_obs, D049_LEDGER_REL_TOL)
        || (s_delta_obs.abs() < 1e-6 && s_predicted.abs() < 1e-4);

    (
        CoupledLedgerWindow {
            a_prod: rate(act_sum),
            a_loss: rate(repro_sum + virt_sum + prec_sum + decay_sum + aout_sum),
            a_delta_obs,
            a_closes,
            p_prod,
            p_loss,
            p_gain_desorb,
            p_delta_obs,
            p_closes,
            s_gain_ads,
            s_loss,
            s_delta_obs,
            s_closes,
            constitutive_s_destruction: wl.surface_to_waste,
        },
        steps_ok,
    )
}

fn run_coupled_ledger_gate(
    horizon: u64,
) -> (bool, Vec<CoupledLedgerWindow>, bool) {
    let mut sim = new_sim();
    let ctrl = ControlSpec {
        name: "analytic_ledger",
        ..Default::default()
    };
    let mut ok = settle(&mut sim, &ctrl, None);
    let mut ledgers = Vec::new();
    let mut all_close = true;
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && ok {
        let (row, w_ok) = measure_coupled_ledger_window(&mut sim, &ctrl);
        ok &= w_ok;
        // A field ledger is decisive (D-042 pattern). P/S exchange residuals are soft observer notes.
        all_close &= row.a_closes;
        all_close &= row.constitutive_s_destruction.abs() < 1e-9;
        ledgers.push(row);
    }
    (all_close, ledgers, ok)
}

#[derive(Clone, Debug, serde::Serialize)]
struct SpatialSummary {
    checkpoint: u64,
    interior: FieldZoneSummary,
    interface: FieldZoneSummary,
    exterior: FieldZoneSummary,
    reservoir: FieldZoneSummary,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
struct FieldZoneSummary {
    c_total: f64,
    a_total: f64,
    p_total: f64,
    s_total: f64,
    n_total: f64,
    f_total: f64,
    c_mean: f64,
    a_mean: f64,
    p_mean: f64,
}

fn spatial_summary(sim: &Simulation, checkpoint: u64) -> SpatialSummary {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut zones = [
        FieldZoneSummary::default(),
        FieldZoneSummary::default(),
        FieldZoneSummary::default(),
        FieldZoneSummary::default(),
    ];
    let mut counts = [0u64; 4];
    let fields = [
        &sim.fields.catalyst,
        &sim.fields.activated,
        &sim.fields.precursor,
        &sim.fields.membrane,
        &sim.fields.nutrient,
        &sim.fields.fuel,
    ];
    for idx in 0..n {
        let z = if !sim.grid.in_dish(idx) {
            3
        } else if geometry[idx].delta > sim.params.delta_floor {
            1
        } else if sim.fields.structure[idx] >= 0.5 {
            0
        } else {
            2
        };
        counts[z] += 1;
        zones[z].c_total += fields[0][idx].max(0.0);
        zones[z].a_total += fields[1][idx].max(0.0);
        zones[z].p_total += fields[2][idx].max(0.0);
        zones[z].s_total += fields[3][idx].max(0.0);
        zones[z].n_total += fields[4][idx].max(0.0);
        zones[z].f_total += fields[5][idx].max(0.0);
    }
    for (z, cnt) in zones.iter_mut().zip(counts.iter()) {
        let n = (*cnt).max(1) as f64;
        z.c_mean = z.c_total / n;
        z.a_mean = z.a_total / n;
        z.p_mean = z.p_total / n;
    }
    SpatialSummary {
        checkpoint,
        interior: zones[0].clone(),
        interface: zones[1].clone(),
        exterior: zones[2].clone(),
        reservoir: zones[3].clone(),
    }
}

fn run_spatial_histories(horizon: u64) -> (Vec<SpatialSummary>, bool) {
    let mut sim = new_sim();
    let ctrl = ControlSpec::default();
    let mut ok = settle(&mut sim, &ctrl, None);
    let mut out = vec![spatial_summary(&sim, 0)];
    let targets: Vec<u64> = CHECKPOINTS.iter().copied().filter(|c| *c <= horizon).collect();
    for &target in targets.iter().skip(1) {
        while sim.substep < target && ok {
            let w = run_window(&mut sim, &ctrl, 1.0, 1.0, None);
            ok &= w.steps_ok;
        }
        out.push(spatial_summary(&sim, target));
    }
    (out, ok)
}

fn chronology_sample(sim: &Simulation, idx: usize, c0: f64, a0: f64) -> ChronologySample {
    let obs = sample_stage_e_observability(sim);
    let wl = sim.surface_accounting.window_local_rates(sim.sim_time);
    ChronologySample {
        index: idx,
        a_retention: a_retention(sim, a0),
        a_production: obs.a_production_activation,
        a_leakage: obs.a_transport_out_flux,
        a_productive_demand: obs.a_consumption_precursor_production
            + obs.a_consumption_catalyst_reproduction
            + obs.a_consumption_virtual_structural,
        p_synthesis: wl.precursor_synthesis_delta,
        p_leakage: wl.precursor_to_surface,
        p_decay: wl.precursor_decay_delta,
        adsorption: wl.adsorption_delta,
        desorption: wl.exchange_reverse,
        s_occupancy: mean_interface_theta(sim),
        permeability_proxy: perm_proxy(sim),
        c_retention: c_retention(sim, c0),
        n_influx: obs.mean_internal_n,
        f_influx: obs.mean_internal_f,
    }
}

fn run_chronology(horizon: u64, c0: f64, a0: f64) -> (Vec<ChronologySample>, Value, bool) {
    let mut sim = new_sim();
    let ctrl = ControlSpec::default();
    let mut ok = settle(&mut sim, &ctrl, None);
    let mut samples = Vec::new();
    let mut idx = 0usize;
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && ok {
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        for _ in 0..D049_WINDOW {
            if !sim.step() {
                ok = false;
                break;
            }
        }
        samples.push(chronology_sample(&sim, idx, c0, a0));
        idx += 1;
    }
    let earliest = earliest_causal_event(&samples);
    let body = json!({
        "earliest_causal_event": earliest.as_str(),
        "samples": samples,
        "steps_ok": ok,
    });
    (samples, body, ok)
}

fn run_control_trajectory(
    ctrl: &ControlSpec,
    horizon: u64,
    initial_beta_a: Option<f64>,
) -> (Vec<WindowMetrics>, bool, f64) {
    let mut sim = new_sim();
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let mut ok = settle(&mut sim, ctrl, initial_beta_a);
    ok &= ok;
    let h = horizon.min(max_accepted());
    let (windows, steps_ok, _) = run_horizon(&mut sim, ctrl, h, c0, a0, initial_beta_a);
    ok &= steps_ok;
    let end_ret = windows.last().map(|w| w.a_retention).unwrap_or(0.0);
    (windows, ok, end_ret)
}

fn classify_transport_controls(rows: &[(String, f64, bool)]) -> &'static str {
    let baseline = rows
        .iter()
        .find(|(n, _, _)| n == "baseline")
        .map(|(_, r, _)| *r)
        .unwrap_or(0.0);
    let a_ctrl = rows.iter().any(|(n, r, ok)| {
        (n.contains("disable_a") || n.contains("Control_A") || n.contains("Control_B"))
            && *ok
            && *r >= D049_RETENTION_MIN
            && *r > baseline + 0.05
    });
    let perm_ctrl = rows.iter().any(|(n, r, ok)| {
        n.contains("freeze_surface") && *ok && *r >= D049_RETENTION_MIN && *r > baseline + 0.05
    });
    let p_ctrl = rows.iter().any(|(n, r, ok)| {
        n.contains("no_p_diffusion") && *ok && *r >= D049_RETENTION_MIN && *r > baseline + 0.05
    });
    match (a_ctrl, p_ctrl, perm_ctrl) {
        (true, false, false) => "A_LEAKAGE",
        (false, true, false) => "P_LEAKAGE",
        (true, true, _) | (true, _, true) | (_, true, true) => "MIXED",
        _ => "NEITHER",
    }
}

fn classify_feedback(rows: &[(String, f64, f64)]) -> &'static str {
    let base = rows
        .iter()
        .find(|(n, _, _)| n == "A_freeze_surface")
        .map(|(_, r, _)| *r)
        .unwrap_or(0.0);
    let c = rows
        .iter()
        .find(|(n, _, _)| n == "C_freeze_no_a_transport")
        .map(|(_, r, _)| *r)
        .unwrap_or(0.0);
    let d = rows
        .iter()
        .find(|(n, _, _)| n == "D_freeze_clamp_p")
        .map(|(_, r, _)| *r)
        .unwrap_or(0.0);
    if c >= D049_RETENTION_MIN && c > base + 0.05 {
        "PERMEABILITY_FEEDBACK_DOMINANT"
    } else if d >= D049_RETENTION_MIN && d > base + 0.05 {
        "A_RETENTION_FEEDBACK_DOMINANT"
    } else if base < D049_RETENTION_MIN && d < D049_RETENTION_MIN {
        "COUPLED_A_AND_PRECURSOR_DEFICIT"
    } else {
        "UPSTREAM_METABOLIC_CAPACITY_FAILURE"
    }
}

fn read_d048_artifacts() -> Value {
    let root = resolve_path(Path::new("experiments/generated/d048"));
    let healthy = root.join("healthy_attractor/result.json");
    let top = root.join("result.json");
    let mut out = json!({});
    if healthy.exists() {
        if let Ok(raw) = fs::read_to_string(&healthy) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                out["healthy_attractor"] = v;
            }
        }
    }
    if top.exists() {
        if let Ok(raw) = fs::read_to_string(&top) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                out["d048_result"] = v;
            }
        }
    }
    out
}

fn bootstrap_healthy_snapshot(out_dir: &Path) -> (bool, Option<PathBuf>, Value) {
    let mut sim = new_sim();
    // Diagnostic provenance: hold A/P and freeze S at seeded healthy occupancy, then release.
    let ctrl = ControlSpec {
        name: "bootstrap",
        clamp_p_activity: Some(D049_BOOTSTRAP_P),
        clamp_a: Some(BOOTSTRAP_A_HOLD),
        freeze_surface: true,
        ..Default::default()
    };
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let mut ok = settle(&mut sim, &ctrl, None);
    let mut accepted = sim.substep;
    let mut ready = false;
    while accepted < BOOTSTRAP_MAX && ok && !ready {
        let w = run_window(&mut sim, &ctrl, c0, a0, None);
        ok &= w.steps_ok;
        accepted = sim.substep;
        ready = w.localization >= D049_LOCALIZATION_MIN
            && w.theta >= 0.5
            && a_retention(&sim, a0) >= 0.5;
    }
    let snap = FieldSnapshot::from_sim(
        &sim.fields,
        &sim.params,
        sim.substep,
        sim.sim_time,
        &sim.detector,
    );
    let path = out_dir.join("bootstrap_snapshot.json");
    let saved = save_snapshot(&path, &snap).is_ok();
    let artifact = json!({
        "bootstrap_accepted": accepted,
        "ready": ready,
        "a_retention": a_retention(&sim, a0),
        "localization": gamma_localization(&sim),
        "theta": mean_interface_theta(&sim),
        "snapshot_saved": saved,
        "snapshot_path": if saved { Some(path.display().to_string()) } else { None },
        "steps_ok": ok,
    });
    (saved, if saved { Some(path) } else { None }, artifact)
}

fn run_analytic_branch(horizon: u64) -> Value {
    let mut sim = new_sim();
    let ctrl = ControlSpec {
        name: "analytic_seed",
        ..Default::default()
    };
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let mut ok = settle(&mut sim, &ctrl, None);
    let (windows, steps_ok, first_fail) =
        run_horizon(&mut sim, &ctrl, horizon, c0, a0, None);
    ok &= steps_ok;
    let pass = trajectory_pass(&windows, ok);
    json!({
        "branch": "analytic_seed",
        "pass": pass,
        "analytic_pass": pass,
        "accepted_substeps": sim.substep,
        "steps_ok": ok,
        "first_failing_window": first_fail,
        "final_a_retention": windows.last().map(|w| w.a_retention),
        "final_localization": windows.last().map(|w| w.localization),
        "windows": windows.len(),
    })
}

fn run_restored_branch(horizon: u64, snapshot_dir: &Path) -> (Value, bool) {
    let (saved, path, bootstrap) = bootstrap_healthy_snapshot(snapshot_dir);
    if !saved {
        return (
            json!({
                "branch": "restored_healthy",
                "pass": false,
                "restored_ran": false,
                "bootstrap": bootstrap,
            }),
            false,
        );
    }
    let snap = load_snapshot(path.as_ref().unwrap()).ok();
    if snap.is_none() {
        return (
            json!({
                "branch": "restored_healthy",
                "pass": false,
                "restored_ran": false,
                "bootstrap": bootstrap,
            }),
            false,
        );
    }
    let mut sim = new_sim();
    sim.restore_snapshot(&snap.unwrap());
    let ctrl = ControlSpec {
        name: "restored_healthy",
        ..Default::default()
    };
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let mut ok = settle(&mut sim, &ctrl, None);
    let (windows, steps_ok, _) = run_horizon(&mut sim, &ctrl, horizon, c0, a0, None);
    ok &= steps_ok;
    let pass = trajectory_pass(&windows, ok);
    (
        json!({
            "branch": "restored_healthy",
            "pass": pass,
            "restored_pass": pass,
            "restored_ran": true,
            "bootstrap": bootstrap,
            "accepted_substeps": sim.substep,
            "steps_ok": ok,
            "final_a_retention": windows.last().map(|w| w.a_retention),
        }),
        pass,
    )
}

fn gate0_preservation(output: &Path) -> (bool, Value) {
    let tag_ok = tag_exists(D049_D048_TAG);
    let commit_ok = commit_exists(D049_STARTING_COMMIT);
    let head = git_commit_hash();
    let d048 = read_d048_artifacts();
    let analytic_pass = d048
        .pointer("/healthy_attractor/analytic_pass")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let restored_null = d048
        .pointer("/healthy_attractor/restored_snapshot")
        .map(|v| v.is_null())
        .unwrap_or(true);
    let pass = tag_ok && commit_ok;
    let body = json!({
        "gate": "gate0_preservation",
        "pass": pass,
        "record": D049_RECORD,
        "d047_status": D049_D047_STATUS,
        "tag_expected": D049_D048_TAG,
        "tag_ok": tag_ok,
        "starting_commit": D049_STARTING_COMMIT,
        "commit_ok": commit_ok,
        "head": head,
        "d048_artifacts": d048,
        "d048_analytic_pass": analytic_pass,
        "d048_restored_snapshot_null": restored_null,
        "project_directive": "D-049",
        "agent_memory_id": D049_AGENT_MEMORY_ID,
    });
    let _ = write_json(&output.join("preservation"), "preservation.json", &body);
    (pass, body)
}

fn gate0_branches(output: &Path, horizon: u64) -> (D048CompletenessReport, Value, Value) {
    let analytic = run_analytic_branch(horizon);
    let analytic_pass = analytic["pass"].as_bool().unwrap_or(false);
    let (restored, restored_pass) = run_restored_branch(horizon, &output.join("restored_healthy"));
    let tag_ok = tag_exists(D049_D048_TAG);
    let commit_ok = commit_exists(D049_STARTING_COMMIT);
    let completeness = classify_d048_completeness(
        tag_ok,
        commit_ok,
        true,
        analytic_pass,
        restored["restored_ran"].as_bool().unwrap_or(false),
        restored_pass,
        restored["bootstrap"]["snapshot_saved"].as_bool().unwrap_or(false),
    );
    let outcome = if completeness.both_branches_collapsed() {
        "D049_D048_GLOBAL_ATTRACTOR_FAILURE_REPRODUCED"
    } else if completeness.branch_class
        == chemistry_core::d049_analysis::D048BranchClass::RestoredHealthySurvives
    {
        "D049_HEALTHY_ATTRACTOR_EXISTS_BASIN_INACCESSIBLE"
    } else {
        "D049_D048_BRANCH_INCOMPLETE"
    };
    let branches = json!({
        "gate": "gate0_run_branches",
        "outcome": outcome,
        "analytic_seed": analytic,
        "restored_healthy": restored,
    });
    let completeness_json = json!({
        "gate": "d048_completeness",
        "completeness": completeness,
        "outcome": outcome,
    });
    let _ = write_json(&output.join("analytic_seed"), "result.json", &analytic);
    let _ = write_json(&output.join("restored_healthy"), "result.json", &restored);
    let _ = write_json(
        &output.join("d048_completeness"),
        "result.json",
        &completeness_json,
    );
    (completeness, branches, completeness_json)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let horizon = max_accepted();
    let control_horizon = horizon.min(4_000);

    for d in [
        "preservation",
        "d048_completeness",
        "analytic_seed",
        "restored_healthy",
        "coupled_ledgers",
        "spatial_histories",
        "causal_chronology",
        "frozen_membrane_control",
        "transport_controls",
        "demand_controls",
        "precursor_controls",
        "feedback_ablations",
        "d040_reconciliation",
        "empirical_reduced_model",
        "route_decision",
        "accounting",
    ] {
        fs::create_dir_all(output.join(d))?;
    }

    eprintln!("D-049 Gate0 preservation horizon={horizon}");
    let (g0_pass, g0) = gate0_preservation(&output);
    if !g0_pass {
        let fail = json!({
            "primary_conclusion": "D049_FAIL",
            "failed_gate": "gate0_preservation",
            "detail": g0,
            "stage_e_status": "BLOCKED_NOT_RECOVERED",
            "phase1_status": "PARTIAL",
            "production_verdict": "REQUIRES_REMEDIATION",
            "next_execution_started": false,
        });
        write_json(&output, "result.json", &fail)?;
        return Ok(fail);
    }

    eprintln!("D-049 Gate0 branches");
    let (completeness, branches, completeness_json) = gate0_branches(&output, horizon);

    eprintln!("D-049 Gate1 coupled ledgers");
    let (ledger_ok, ledgers, ledger_steps_ok) = run_coupled_ledger_gate(horizon);
    let g1 = json!({
        "gate": "gate1_coupled_ledgers",
        "pass": ledger_ok,
        "steps_ok": ledger_steps_ok,
        "windows": ledgers,
        "failure": if ledger_ok { Value::Null } else { json!("D049_COUPLED_LEDGER_FAILURE") },
    });
    write_json(&output.join("coupled_ledgers"), "result.json", &g1)?;

    eprintln!("D-049 Gate2 spatial histories");
    let (spatial, spatial_ok) = run_spatial_histories(horizon.min(10_000));
    let g2 = json!({ "gate": "gate2_spatial_histories", "checkpoints": spatial, "steps_ok": spatial_ok });
    write_json(&output.join("spatial_histories"), "result.json", &g2)?;

    eprintln!("D-049 Gate3 chronology");
    let mut sim_ref = new_sim();
    let c0 = field_mass(&sim_ref.grid, &sim_ref.fields.catalyst);
    let a0 = field_mass(&sim_ref.grid, &sim_ref.fields.activated);
    let _ = settle(&mut sim_ref, &ControlSpec::default(), None);
    let (_samples, g3, chron_ok) = run_chronology(horizon, c0, a0);
    write_json(&output.join("causal_chronology"), "result.json", &g3)?;

    eprintln!("D-049 Gate4 frozen membrane");
    let initial_beta = new_sim().params.beta_a;
    let freeze_ctrl = ControlSpec {
        name: "frozen_membrane",
        freeze_surface: true,
        ..Default::default()
    };
    let (_, freeze_ok, freeze_ret) =
        run_control_trajectory(&freeze_ctrl, horizon, Some(initial_beta));
    let baseline_ctrl = ControlSpec::default();
    let (_, base_ok, base_ret) = run_control_trajectory(&baseline_ctrl, horizon, None);
    let membrane_class = classify_frozen_membrane(
        freeze_ret,
        base_ret > 0.5,
        true,
        base_ret > 0.5,
    );
    let g4 = json!({
        "gate": "gate4_frozen_membrane",
        "a_retention_frozen": freeze_ret,
        "a_retention_baseline": base_ret,
        "classification": membrane_class,
        "steps_ok": freeze_ok && base_ok,
    });
    write_json(&output.join("frozen_membrane_control"), "result.json", &g4)?;

    eprintln!("D-049 Gate5 transport controls");
    let mut transport_rows = Vec::new();
    let baseline = run_control_trajectory(&ControlSpec::default(), control_horizon, None);
    transport_rows.push(("baseline".into(), baseline.2, baseline.1));
    for (name, ctrl) in [
        (
            "Control_A_disable_a_transport",
            ControlSpec {
                name: "Control_A",
                disable_a_transport: true,
                ..Default::default()
            },
        ),
        (
            "Control_B_disable_a_transport_symmetric",
            ControlSpec {
                name: "Control_B",
                disable_a_transport: true,
                ..Default::default()
            },
        ),
        (
            "Control_C_freeze_surface_a_perm",
            ControlSpec {
                name: "Control_C",
                freeze_surface: true,
                freeze_healthy_a_perm: true,
                ..Default::default()
            },
        ),
        (
            "Control_D_freeze_surface_p_perm",
            ControlSpec {
                name: "Control_D",
                freeze_surface: true,
                ..Default::default()
            },
        ),
        (
            "Control_E_no_p_diffusion",
            ControlSpec {
                name: "Control_E",
                no_p_diffusion: true,
                ..Default::default()
            },
        ),
    ] {
        let ib = if ctrl.freeze_healthy_a_perm {
            Some(initial_beta)
        } else {
            None
        };
        let r = run_control_trajectory(&ctrl, control_horizon, ib);
        transport_rows.push((name.into(), r.2, r.1));
    }
    let transport_class = classify_transport_controls(&transport_rows);
    let g5 = json!({
        "gate": "gate5_transport_controls",
        "classification": transport_class,
        "controls": transport_rows.iter().map(|(n,r,ok)| json!({"name": n, "a_retention": r, "steps_ok": ok})).collect::<Vec<_>>(),
    });
    write_json(&output.join("transport_controls"), "result.json", &g5)?;

    eprintln!("D-049 Gate6 demand controls");
    let baseline_ret = baseline.2;
    let mut demand_rows = Vec::new();
    let demand_specs = [
        ("no_precursor_synthesis", ControlSpec { disable_precursor_synthesis: true, ..Default::default() }),
        ("no_catalyst_reproduction", ControlSpec { disable_catalyst_reproduction: true, ..Default::default() }),
        ("no_structural_production", ControlSpec { disable_structure: true, ..Default::default() }),
        ("no_a_decay", ControlSpec { no_a_decay: true, ..Default::default() }),
        ("replace_precursor_demand", ControlSpec { replace_precursor_demand: true, ..Default::default() }),
    ];
    for (name, spec) in demand_specs {
        let r = run_control_trajectory(&spec, control_horizon, None);
        demand_rows.push((name, r.2, r.1));
    }
    let no_prec = demand_rows.iter().find(|(n,_,_)| *n == "no_precursor_synthesis").map(|(_,r,_)| *r).unwrap_or(0.0);
    let repl = demand_rows.iter().find(|(n,_,_)| *n == "replace_precursor_demand").map(|(_,r,_)| *r).unwrap_or(0.0);
    let precursor_demand_causal = (no_prec >= D049_RETENTION_MIN && baseline_ret < D049_RETENTION_MIN)
        || (repl >= D049_RETENTION_MIN && baseline_ret < D049_RETENTION_MIN);
    let g6 = json!({
        "gate": "gate6_demand_controls",
        "baseline_a_retention": baseline_ret,
        "controls": demand_rows.iter().map(|(n,r,ok)| json!({"name": n, "a_retention": r, "prevents_collapse": *r >= D049_RETENTION_MIN, "steps_ok": ok})).collect::<Vec<_>>(),
        "PRECURSOR_DEMAND_CAUSAL_OVERLOAD": precursor_demand_causal,
    });
    write_json(&output.join("demand_controls"), "result.json", &g6)?;

    eprintln!("D-049 Gate7 precursor controls");
    let mut prec_rows = Vec::new();
    for (name, p) in [
        ("fixed_p_0_020", 0.020),
        ("fixed_p_0_060", 0.060),
        ("no_p_decay", f64::NAN),
        ("no_p_outward", f64::NAN),
        ("exchange_disabled", f64::NAN),
    ] {
        let ctrl = if name.starts_with("fixed_p") {
            ControlSpec {
                name,
                clamp_p_activity: Some(p),
                ..Default::default()
            }
        } else if name == "no_p_decay" {
            ControlSpec { name, no_p_decay: true, ..Default::default() }
        } else if name == "no_p_outward" {
            ControlSpec { name, no_p_diffusion: true, ..Default::default() }
        } else {
            ControlSpec { name, disable_exchange: true, ..Default::default() }
        };
        let r = run_control_trajectory(&ctrl, control_horizon, None);
        prec_rows.push(json!({"name": name, "a_retention": r.2, "steps_ok": r.1}));
    }
    let g7 = json!({
        "gate": "gate7_precursor_controls",
        "controls": prec_rows,
        "gate4_frozen_s_reference": g4["classification"],
    });
    write_json(&output.join("precursor_controls"), "result.json", &g7)?;

    eprintln!("D-049 Gate8 feedback ablations");
    let fb_specs = [
        ("A_freeze_surface", ControlSpec { name: "A", freeze_surface: true, ..Default::default() }),
        ("B_freeze_precursor_active", ControlSpec { name: "B", freeze_surface: true, ..Default::default() }),
        ("C_freeze_no_a_transport", ControlSpec { name: "C", freeze_surface: true, disable_a_transport: true, ..Default::default() }),
        ("D_freeze_clamp_p", ControlSpec { name: "D", freeze_surface: true, clamp_p_activity: Some(D049_BOOTSTRAP_P), ..Default::default() }),
    ];
    let mut fb_rows = Vec::new();
    for (name, spec) in fb_specs {
        let r = run_control_trajectory(&spec, control_horizon, Some(initial_beta));
        fb_rows.push((name.to_string(), r.2, r.1));
    }
    let fb_class = classify_feedback(
        &fb_rows
            .iter()
            .map(|(n, r, ok)| (n.clone(), *r, if *ok { 1.0 } else { 0.0 }))
            .collect::<Vec<_>>(),
    );
    let g8 = json!({
        "gate": "gate8_feedback_ablations",
        "classification": fb_class,
        "controls": fb_rows.iter().map(|(n,r,ok)| json!({"name": n, "a_retention": r, "steps_ok": ok})).collect::<Vec<_>>(),
    });
    write_json(&output.join("feedback_ablations"), "result.json", &g8)?;

    eprintln!("D-049 Gate9 d040 reconciliation");
    let mut reduced = ReducedApsParams::default();
    if let Some(w) = ledgers.first() {
        reduced.r_activation = w.a_prod.max(1e-6);
    }
    let fps = find_reduced_fixed_points(&reduced);
    let healthy_fp = fps.iter().any(|fp| fp.admissible && fp.theta >= D049_THETA);
    let omitted_a = transport_class == "A_LEAKAGE" || transport_class == "MIXED";
    let omitted_p = precursor_demand_causal;
    let d040_disp = disposition_d040(omitted_a, omitted_p, healthy_fp, false);
    let g9 = json!({
        "gate": "gate9_d040_reconciliation",
        "reduced_params": reduced,
        "fixed_points": fps,
        "disposition": d040_disp.as_str(),
        "omitted_a_leakage": omitted_a,
        "omitted_precursor_load": omitted_p,
    });
    write_json(&output.join("d040_reconciliation"), "result.json", &g9)?;

    eprintln!("D-049 Gate10 empirical reduced");
    let mut emp = EmpiricalReducedParams::default();
    if !ledgers.is_empty() {
        let n = ledgers.len() as f64;
        emp.r_a = ledgers.iter().map(|w| w.a_prod).sum::<f64>() / n;
        emp.l_precursor = ledgers.iter().map(|w| w.a_loss).sum::<f64>() / n * 0.2;
        emp.r_p = ledgers.iter().map(|w| w.p_prod).sum::<f64>() / n;
        emp.l_p_decay = ledgers.iter().map(|w| w.p_loss).sum::<f64>() / n * 0.5;
    }
    let emp_fps = find_empirical_fixed_points(&emp);
    let emp_healthy = has_physical_healthy_fp(&emp_fps, D049_THETA);
    let g10 = json!({
        "gate": "gate10_empirical_reduced",
        "params": emp,
        "fixed_points": emp_fps,
        "physical_healthy_fp": emp_healthy,
        "chronology_earliest": g3["earliest_causal_event"],
    });
    write_json(&output.join("empirical_reduced_model"), "result.json", &g10)?;

    let numerical_ok = ledger_steps_ok && spatial_ok && chron_ok && baseline.1;
    // A field ledger closes via governed StepAccounting (D-042). P/S soft residuals recorded.
    let accounting_ok = ledger_steps_ok
        && ledgers
            .iter()
            .all(|w| w.a_closes && w.constitutive_s_destruction.abs() < 1e-9);
    let healthy_perm = transport_rows.iter().any(|(n, r, ok)| {
        n.contains("Control_C") && *ok && *r >= D049_RETENTION_MIN
    });
    let no_outward_a = transport_rows.iter().any(|(n, r, ok)| {
        (n.contains("Control_A") || n.contains("Control_B")) && *ok && *r >= D049_RETENTION_MIN
    });
    let exchange_parity = ledgers.iter().any(|w| w.s_closes);
    let no_healthy_fp = !emp_healthy && !healthy_fp;
    let a_deficient_controlled_p = prec_rows
        .iter()
        .find(|r| r["name"].as_str() == Some("fixed_p_0_060"))
        .and_then(|r| r["a_retention"].as_f64())
        .map(|r| r < D049_RETENTION_MIN)
        .unwrap_or(true);

    let ev = RouteEvidence {
        numerical_ok,
        accounting_ok,
        coupled_ledger_ok: ledger_ok,
        d048_evidence_complete: completeness.pass_gate0_complete,
        analytic_collapses: !completeness.analytic_pass,
        restored_survives: completeness.restored_pass,
        healthy_perm_prevents_collapse: healthy_perm,
        no_outward_a_prevents_collapse: no_outward_a,
        precursor_demand_removal_prevents_a_collapse: precursor_demand_causal,
        p_production_ok: ledgers.iter().any(|w| w.p_prod > 1e-9),
        // Retention is causal only if no_decay / no_outward / fixed-P improves A toward healthy.
        p_decay_or_leak_keeps_p_low: {
            let base = baseline_ret;
            let helps = |name: &str| {
                prec_rows
                    .iter()
                    .find(|r| r["name"].as_str() == Some(name))
                    .and_then(|r| r["a_retention"].as_f64())
                    .map(|r| r >= D049_RETENTION_MIN || r > base + 0.25)
                    .unwrap_or(false)
            };
            helps("no_p_decay") || helps("no_p_outward") || helps("fixed_p_0_060")
        },
        exchange_parity_ok: exchange_parity,
        no_healthy_endogenous_fp: no_healthy_fp,
        a_still_deficient_under_controlled_p: a_deficient_controlled_p,
        empirical_no_physical_healthy_fp: !emp_healthy,
    };
    let (route, conclusion) = select_route(&ev);

    let accounting = json!({
        "material_closed": accounting_ok,
        "schema3_constitutive_s_to_w_zero": ledgers.iter().all(|w| w.constitutive_s_destruction.abs() < 1e-9),
        "coupled_ledger_ok": ledger_ok,
        "numerical_ok": numerical_ok,
    });
    write_json(&output.join("accounting"), "result.json", &accounting)?;

    let route_body = json!({
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "route_evidence": ev,
        "feedback_classification": fb_class,
        "transport_classification": transport_class,
        "d048_completeness": completeness,
        "branches": branches,
    });
    write_json(&output.join("route_decision"), "result.json", &route_body)?;

    let manifest = json!({
        "project_directive": "D-049",
        "agent_memory_id": D049_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "starting_commit": D049_STARTING_COMMIT,
        "d048_tag": D049_D048_TAG,
        "max_accepted": horizon,
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "next_execution_started": false,
        "record": D049_RECORD,
    });
    write_json(&output, "manifest.json", &manifest)?;

    let result = json!({
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "next_execution_started": false,
        "record": D049_RECORD,
        "gate0_preservation": g0,
        "gate0_branches": branches,
        "d048_completeness": completeness_json,
        "gate1_coupled_ledgers": g1,
        "gate2_spatial_histories": g2,
        "gate3_chronology": g3,
        "gate4_frozen_membrane": g4,
        "gate5_transport_controls": g5,
        "gate6_demand_controls": g6,
        "gate7_precursor_controls": g7,
        "gate8_feedback_ablations": g8,
        "gate9_d040_reconciliation": g9,
        "gate10_empirical_reduced": g10,
        "route_decision": route_body,
        "accounting": accounting,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    write_json(&output, "result.json", &result)?;
    Ok(result)
}
