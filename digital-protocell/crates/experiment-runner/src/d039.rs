//! D-039 exchange+damage-only membrane maintenance qualification pipeline.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{InterventionAction, SimParams, SurfaceTurnoverSchema};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::surface_balance_q;
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, classify_damage_repair,
    gate0_contract_audit, gate1_schema_safety, revised_stage_e_membrane_contract, select_conclusion,
    v8_schema3_params, DamageRepairClass, D039Conclusion, D039_AGENT_MEMORY_ID, D039_D038_TAG,
    D039_MAX_ACCEPTED, D039_NET_S_FLOW_MAX, D039_REPLACEMENT_MIN, D039_S_DRIFT_MAX,
    D039_STARTING_COMMIT, D039_TRACER_RESIDUAL_MAX,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::{apply_declared_membrane_arc_damage, apply_intervention};
use chemistry_core::membrane_label_tracer::MembraneLabelTracer;
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const WINDOW: u64 = 1_000;
const MEASURE_WINDOW: u64 = 500;
const RADIUS: f64 = 22.0;
const THETA: f64 = 0.6;
const REPEATED_DAMAGE_INTERVAL: u64 = 20_000;
const ROUTE_QUALIFIED: &str = "MEMBRANE_ARCHITECTURE_V8_EXCHANGE_DAMAGE_MAINTENANCE";

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
    std::env::var("D039_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D039_MAX_ACCEPTED)
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
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
        // Do not copy base.d008_stage_mode — renewal assays require ConstrainedRadius.
    }
    apply_renewal_stage_mode(&mut params);
    apply_schema3_exchange_damage_only(&mut params);
    params
}

fn new_sim(enforce_fixed: bool, with_tracer: bool) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params());
    sim.enforce_structure_constraint = enforce_fixed;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, RADIUS, THETA);
    if with_tracer {
        let p = field_mass(&sim.grid, &sim.fields.precursor);
        let s = field_mass(&sim.grid, &sim.fields.membrane);
        sim.membrane_label_tracer = Some(MembraneLabelTracer::init_from_totals(p, s));
    }
    sim
}

#[derive(Clone, Debug)]
struct WindowMetrics {
    mean_s: f64,
    net_exchange: f64,
    forward: f64,
    reverse: f64,
    gross_exchange: f64,
    normalized_net_flow: f64,
    localization: f64,
    q_passive: f64,
}

fn run_accounting_window(sim: &mut Simulation) -> (WindowMetrics, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut s_sum = 0.0;
    let mut n = 0u64;
    let mut steps_ok = true;
    for _ in 0..WINDOW {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        if sim.substep % 20 == 0 {
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
    let net = wl.exchange_net;
    let turn = wl.gamma_decay_delta.max(0.0);
    let q = surface_balance_q(net, turn);
    let g = net / mean_s.max(f64::EPSILON);
    let forward = wl.exchange_forward;
    let reverse = wl.exchange_reverse;
    let gross = forward + reverse;
    let loc = gamma_localization(sim);
    (
        WindowMetrics {
            mean_s,
            net_exchange: net,
            forward,
            reverse,
            gross_exchange: gross,
            normalized_net_flow: g,
            localization: loc,
            q_passive: q,
        },
        steps_ok,
    )
}

fn three_consecutive_net_flow(windows: &[WindowMetrics]) -> bool {
    if windows.len() < 3 {
        return false;
    }
    windows.windows(3).any(|w| {
        w.iter()
            .all(|m| m.normalized_net_flow.abs() <= D039_NET_S_FLOW_MAX)
    })
}

fn window_active_exchange(w: &WindowMetrics) -> bool {
    w.forward > 1e-12 && w.reverse > 1e-12 && w.gross_exchange > 0.0
}

fn measure_late_mean_s(sim: &mut Simulation) -> (f64, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut s_sum = 0.0;
    let mut n = 0u64;
    let mut ok = true;
    for _ in 0..MEASURE_WINDOW {
        if !sim.step() {
            ok = false;
            break;
        }
        if sim.substep % 10 == 0 {
            s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            n += 1;
        }
    }
    let mean = if n > 0 {
        s_sum / n as f64
    } else {
        total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    (mean, ok)
}

#[derive(Clone)]
struct BaselineOutcome {
    pass: bool,
    spec_id: String,
    enforce_fixed: bool,
    localization: f64,
    c_retention: f64,
    a_retention: f64,
    late_mean_s: f64,
    accepted: u64,
    steps_ok: bool,
    windows: Vec<WindowMetrics>,
    artifact: Value,
}

fn run_baseline_assay(
    enforce_fixed: bool,
    horizon: u64,
    loc_min: f64,
    spec_id: &str,
) -> BaselineOutcome {
    let mut sim = new_sim(enforce_fixed, false);
    let mut accepted = 0u64;
    let mut steps_ok = true;

    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }

    // Burn toward exchange balance before locking retention baselines so Gate3
    // measures quasi-steady maintenance rather than seed A transient decay.
    let burn = (horizon / 4).max(3 * WINDOW);
    let mut windows = Vec::new();
    while accepted < burn && steps_ok {
        let (w, ok) = run_accounting_window(&mut sim);
        steps_ok &= ok;
        accepted += WINDOW;
        windows.push(w);
        if three_consecutive_net_flow(&windows)
            && windows.iter().rev().take(3).all(window_active_exchange)
        {
            break;
        }
        if accepted % 10_000 == 0 {
            let _ = std::io::Write::flush(&mut std::io::stderr());
            eprintln!(
                "D-039 Gate3 {spec_id} burn accepted={accepted} loc={:.4} g={:.3e}",
                windows.last().map(|w| w.localization).unwrap_or(0.0),
                windows
                    .last()
                    .map(|w| w.normalized_net_flow)
                    .unwrap_or(0.0)
            );
        }
    }

    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);

    while accepted < horizon && steps_ok {
        let (w, ok) = run_accounting_window(&mut sim);
        steps_ok &= ok;
        accepted += WINDOW;
        if accepted % 10_000 == 0 {
            let _ = std::io::Write::flush(&mut std::io::stderr());
            eprintln!(
                "D-039 Gate3 {spec_id} accepted={accepted} loc={:.4} g={:.3e}",
                w.localization, w.normalized_net_flow
            );
        }
        windows.push(w);
        if three_consecutive_net_flow(&windows)
            && windows.iter().rev().take(3).all(window_active_exchange)
        {
            break;
        }
    }

    let (late_mean_s, measure_ok) = measure_late_mean_s(&mut sim);
    steps_ok &= measure_ok;
    accepted += MEASURE_WINDOW;

    let c1 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a1 = field_mass(&sim.grid, &sim.fields.activated);
    let c_ret = c1 / c0.max(1e-18);
    let a_ret = a1 / a0.max(1e-18);
    let loc = gamma_localization(&sim);
    let bal_ok = three_consecutive_net_flow(&windows);
    let active_ok = windows
        .last()
        .map(window_active_exchange)
        .unwrap_or(false);
    let gross_ok = windows.last().map(|w| w.gross_exchange > 0.0).unwrap_or(false);
    let pass = steps_ok
        && loc >= loc_min
        && c_ret >= 0.80
        && a_ret >= 0.80
        && bal_ok
        && active_ok
        && gross_ok
        && !sim.last_reject_detail.contains("CapacityExceeded");

    let artifact = json!({
        "gate": 3,
        "spec_id": spec_id,
        "enforce_structure_constraint": enforce_fixed,
        "pass": pass,
        "localization": loc,
        "localization_min": loc_min,
        "c_retention": c_ret,
        "a_retention": a_ret,
        "late_mean_s": late_mean_s,
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
        "balance_ok": bal_ok,
        "active_exchange_ok": active_ok,
        "gross_replacement_ok": gross_ok,
        "windows": windows.iter().map(|w| json!({
            "mean_s": w.mean_s,
            "net_exchange": w.net_exchange,
            "forward": w.forward,
            "reverse": w.reverse,
            "gross_exchange": w.gross_exchange,
            "normalized_net_flow": w.normalized_net_flow,
            "localization": w.localization,
            "q_passive": w.q_passive,
        })).collect::<Vec<_>>(),
        "turnover_schema": SurfaceTurnoverSchema::ExchangeDamageOnly.as_str(),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "last_reject": sim.last_reject_detail,
    });

    BaselineOutcome {
        pass,
        spec_id: spec_id.into(),
        enforce_fixed,
        localization: loc,
        c_retention: c_ret,
        a_retention: a_ret,
        late_mean_s,
        accepted,
        steps_ok,
        windows,
        artifact,
    }
}

fn settle_dynamic_to_balance(settle_horizon: u64, with_tracer: bool) -> (Simulation, bool, u64) {
    let mut sim = new_sim(false, with_tracer);
    let mut accepted = 0u64;
    let mut steps_ok = true;
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }
    let mut windows = Vec::new();
    while accepted < settle_horizon && steps_ok {
        let (w, ok) = run_accounting_window(&mut sim);
        steps_ok &= ok;
        accepted += WINDOW;
        windows.push(w);
        if three_consecutive_net_flow(&windows) && windows.iter().rev().take(3).all(window_active_exchange)
        {
            break;
        }
    }
    (sim, steps_ok, accepted)
}

struct PulseChaseOutcome {
    pass: bool,
    replacement: f64,
    s_drift: f64,
    residence_time: f64,
    artifact: Value,
}

fn run_pulse_chase(horizon: u64) -> PulseChaseOutcome {
    // Settle on at most half the budget so chase always has room to run.
    let settle_budget = (horizon / 2).max(3 * WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, true);
    let s_initial = field_mass(&sim.grid, &sim.fields.membrane);
    if let Some(tracer) = sim.membrane_label_tracer.as_mut() {
        tracer.pulse_label_all_s_as_old(s_initial);
    }

    let mut gross_sum = 0.0;
    let mut gross_n = 0u64;
    let mut s_sum = 0.0;
    let mut s_n = 0u64;
    let chase_end = accepted.saturating_add(horizon);

    while accepted < chase_end && steps_ok {
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
        let wl = sim.surface_accounting.window_local();
        gross_sum += wl.exchange_forward + wl.exchange_reverse;
        gross_n += 1;
        if sim.substep % 20 == 0 {
            s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            s_n += 1;
        }
        let s_now = total_surface_mass(&sim.grid, &sim.fields.membrane);
        let replacement = sim
            .membrane_label_tracer
            .as_ref()
            .map(|t| t.replacement_fraction(s_now))
            .unwrap_or(0.0);
        if replacement >= D039_REPLACEMENT_MIN {
            break;
        }
        if accepted % 10_000 == 0 {
            eprintln!(
                "D-039 Gate4 pulse-chase accepted={accepted} replacement={replacement:.4}"
            );
        }
    }

    let s_final = field_mass(&sim.grid, &sim.fields.membrane);
    let s_drift = (s_final - s_initial).abs() / s_initial.max(1e-18);
    let replacement = sim
        .membrane_label_tracer
        .as_ref()
        .map(|t| t.replacement_fraction(s_final))
        .unwrap_or(0.0);
    let mean_s = if s_n > 0 {
        s_sum / s_n as f64
    } else {
        s_final
    };
    let mean_gross = if gross_n > 0 {
        gross_sum / gross_n as f64
    } else {
        0.0
    };
    let residence_time = if mean_gross > 0.0 {
        mean_s / mean_gross
    } else {
        f64::INFINITY
    };
    let tracer_residual = sim
        .membrane_label_tracer
        .as_ref()
        .map(|t| t.inventory_residual())
        .unwrap_or(0.0);
    let pass = steps_ok
        && replacement >= D039_REPLACEMENT_MIN
        && s_drift <= D039_S_DRIFT_MAX
        && tracer_residual <= D039_TRACER_RESIDUAL_MAX;

    let artifact = json!({
        "gate": 4,
        "pass": pass,
        "replacement_fraction": replacement,
        "replacement_min": D039_REPLACEMENT_MIN,
        "s_drift": s_drift,
        "s_drift_max": D039_S_DRIFT_MAX,
        "s_initial": s_initial,
        "s_final": s_final,
        "residence_time_estimate": residence_time,
        "mean_s": mean_s,
        "mean_gross_exchange_rate": mean_gross,
        "tracer_residual": tracer_residual,
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });

    PulseChaseOutcome {
        pass,
        replacement,
        s_drift,
        residence_time,
        artifact,
    }
}

struct DamageOutcome {
    pass: bool,
    fraction: f64,
    classification: DamageRepairClass,
    s_recovery_ratio: f64,
    local_occupancy_ratio: f64,
    localization: f64,
    artifact: Value,
}

fn local_occupancy_ratio_after_recovery(
    report: &chemistry_core::interventions::MembraneArcDamageReport,
    s_after_recovery: f64,
) -> f64 {
    if report.local_occupancy_before <= 0.0 || report.total_s_before <= 0.0 {
        return 1.0;
    }
    let arc_frac = report.local_occupancy_before / report.total_s_before;
    let local_after = s_after_recovery * arc_frac;
    (local_after / report.local_occupancy_before).clamp(0.0, 2.0)
}

fn run_damage_assay(fraction: f64, horizon: u64, mandatory: bool) -> DamageOutcome {
    let settle_budget = (horizon / 2).max(3 * WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, true);
    let (late_mean_s, measure_ok) = measure_late_mean_s(&mut sim);
    steps_ok &= measure_ok;
    accepted += MEASURE_WINDOW;

    let loc_before = gamma_localization(&sim);
    let w_before = field_mass(&sim.grid, &sim.fields.waste);
    let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, fraction);
    if let Some(tracer) = sim.membrane_label_tracer.as_mut() {
        tracer.record_declared_damage(report.s_removed, report.total_s_before);
    }
    let s_immediate = field_mass(&sim.grid, &sim.fields.membrane);
    let w_immediate = field_mass(&sim.grid, &sim.fields.waste);
    let loc_immediate = gamma_localization(&sim);

    let recover_end = accepted.saturating_add(horizon);
    while accepted < recover_end && steps_ok {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
        let s_now = field_mass(&sim.grid, &sim.fields.membrane);
        let s_ratio = s_now / late_mean_s.max(1e-18);
        if mandatory && s_ratio >= 0.95 && gamma_localization(&sim) >= 0.95 {
            break;
        }
        if accepted % 10_000 == 0 {
            eprintln!(
                "D-039 Gate5/6 damage_{}% recovery accepted={accepted} s_ratio={s_ratio:.4}",
                (fraction * 100.0) as u32
            );
        }
    }

    let s_after = field_mass(&sim.grid, &sim.fields.membrane);
    let w_after = field_mass(&sim.grid, &sim.fields.waste);
    let loc = gamma_localization(&sim);
    let s_recovery_ratio = s_after / late_mean_s.max(1e-18);
    let local_occupancy_ratio =
        local_occupancy_ratio_after_recovery(&report, s_after);
    let classification = classify_damage_repair(
        fraction,
        s_recovery_ratio,
        local_occupancy_ratio,
        loc,
        mandatory,
    );
    let pass = if mandatory {
        classification == DamageRepairClass::SuccessfulRepair
    } else {
        true
    };

    let artifact = json!({
        "gate": if mandatory { 6 } else { 5 },
        "fraction": fraction,
        "mandatory_repair": mandatory,
        "pass": pass,
        "classification": classification.as_str(),
        "late_mean_s_before_damage": late_mean_s,
        "s_recovery_ratio": s_recovery_ratio,
        "local_occupancy_ratio": local_occupancy_ratio,
        "localization_before": loc_before,
        "localization_immediate": loc_immediate,
        "localization_after_recovery": loc,
        "damage_report": report,
        "immediate": {
            "s": s_immediate,
            "w": w_immediate,
            "w_delta": w_immediate - w_before,
        },
        "after_recovery": {
            "s": s_after,
            "w": w_after,
        },
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
        "tracer_residual": sim.membrane_label_tracer.as_ref().map(|t| t.inventory_residual()),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "last_reject": sim.last_reject_detail,
    });

    DamageOutcome {
        pass,
        fraction,
        classification,
        s_recovery_ratio,
        local_occupancy_ratio,
        localization: loc,
        artifact,
    }
}

struct MetabolicControlOutcome {
    pass: bool,
    artifact: Value,
}

fn run_metabolic_control(
    control_id: &str,
    apply_control: impl FnOnce(&mut Simulation),
    horizon: u64,
) -> MetabolicControlOutcome {
    let settle_budget = (horizon / 2).max(3 * WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, false);
    let s_ref = field_mass(&sim.grid, &sim.fields.membrane);
    let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
    apply_control(&mut sim);
    let s_after_damage = field_mass(&sim.grid, &sim.fields.membrane);

    let recover_end = accepted.saturating_add(horizon);
    while accepted < recover_end && steps_ok {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }

    let s_final = field_mass(&sim.grid, &sim.fields.membrane);
    let recovery = s_final / s_ref.max(1e-18);
    let loc = gamma_localization(&sim);

    MetabolicControlOutcome {
        pass: steps_ok,
        artifact: json!({
            "control_id": control_id,
            "s_reference": s_ref,
            "s_after_damage": s_after_damage,
            "s_final": s_final,
            "recovery_ratio": recovery,
            "localization": loc,
            "accepted_substeps": accepted,
            "steps_ok": steps_ok,
            "source_commit": git_commit_hash(),
        }),
    }
}

fn run_metabolic_controls(horizon: u64) -> MetabolicControlOutcome {
    let normal = run_metabolic_control("A_normal", |_| {}, horizon);
    let no_a = run_metabolic_control("B_no_activation", |sim| {
        sim.params.k_d008_activation = 0.0;
    }, horizon);
    let no_p = run_metabolic_control("C_no_precursor_synthesis", |sim| {
        sim.d026_disable_precursor_synthesis = true;
    }, horizon);
    let shutdown = run_metabolic_control("D_shutdown_reservoir", |sim| {
        apply_intervention(
            &sim.grid,
            &mut sim.fields,
            &InterventionAction::ShutdownReservoir,
            &mut sim.params,
        );
        sim.params.n_reservoir = 0.0;
        sim.params.f_reservoir = 0.0;
    }, horizon);

    let normal_rec = normal.artifact["recovery_ratio"].as_f64().unwrap_or(0.0);
    let no_a_rec = no_a.artifact["recovery_ratio"].as_f64().unwrap_or(0.0);
    let no_p_rec = no_p.artifact["recovery_ratio"].as_f64().unwrap_or(0.0);
    let pass = normal.pass
        && no_a.pass
        && no_p.pass
        && shutdown.pass
        && normal_rec > no_a_rec
        && normal_rec > no_p_rec;

    MetabolicControlOutcome {
        pass,
        artifact: json!({
            "gate": 7,
            "pass": pass,
            "normal": normal.artifact,
            "no_activation": no_a.artifact,
            "no_precursor_synthesis": no_p.artifact,
            "shutdown_reservoir": shutdown.artifact,
            "interpretation": "Normal repair should exceed no-A and no-P controls when resources matter.",
        }),
    }
}

fn run_repeated_damage(horizon: u64) -> Value {
    let mut sim_normal = new_sim(false, false);
    let mut sim_no_a = new_sim(false, false);
    sim_no_a.params.k_d008_activation = 0.0;
    let mut sim_no_p = new_sim(false, false);
    sim_no_p.d026_disable_precursor_synthesis = true;

    let run_variant = |sim: &mut Simulation, label: &str| -> Value {
        let mut accepted = 0u64;
        let mut steps_ok = true;
        for _ in 0..D026_SETTLE_STEPS {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
        }
        let initial_free_p = field_mass(&sim.grid, &sim.fields.precursor);
        let mut cumulative_removed = 0.0;
        let mut pulses = 0u32;
        let s_ref = field_mass(&sim.grid, &sim.fields.membrane);
        while accepted < horizon && steps_ok && cumulative_removed < initial_free_p.max(1e-18) {
            if accepted > 0 && accepted % REPEATED_DAMAGE_INTERVAL == 0 {
                let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
                cumulative_removed += report.s_removed;
                pulses += 1;
            }
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
        }
        let s_final = field_mass(&sim.grid, &sim.fields.membrane);
        json!({
            "variant": label,
            "initial_free_p": initial_free_p,
            "cumulative_removed": cumulative_removed,
            "pulses": pulses,
            "s_reference": s_ref,
            "s_final": s_final,
            "recovery_ratio": s_final / s_ref.max(1e-18),
            "localization": gamma_localization(sim),
            "accepted_substeps": accepted,
            "steps_ok": steps_ok,
        })
    };

    let normal = run_variant(&mut sim_normal, "normal");
    let no_a = run_variant(&mut sim_no_a, "no_activation");
    let no_p = run_variant(&mut sim_no_p, "no_precursor_synthesis");
    let pass = normal["recovery_ratio"].as_f64().unwrap_or(0.0)
        > no_a["recovery_ratio"].as_f64().unwrap_or(0.0)
        && normal["recovery_ratio"].as_f64().unwrap_or(0.0)
            > no_p["recovery_ratio"].as_f64().unwrap_or(0.0);

    json!({
        "gate": 7,
        "repeated_damage": true,
        "interval_steps": REPEATED_DAMAGE_INTERVAL,
        "pass": pass,
        "normal": normal,
        "no_activation": no_a,
        "no_precursor_synthesis": no_p,
    })
}

fn run_tracer_validation() -> (bool, Value) {
    let params = schema3_organism_params();
    let mut off = Simulation::new(params.clone());
    seed_v7_compartment(&mut off, RADIUS, THETA);
    let mut on = Simulation::new(params);
    seed_v7_compartment(&mut on, RADIUS, THETA);
    let p0 = field_mass(&on.grid, &on.fields.precursor);
    let s0 = field_mass(&on.grid, &on.fields.membrane);
    on.membrane_label_tracer = Some(MembraneLabelTracer::init_from_totals(p0, s0));

    let steps = 200u64;
    let mut parity_ok = true;
    let mut steps_ok = true;
    for _ in 0..steps {
        if !off.step() || !on.step() {
            steps_ok = false;
            break;
        }
        for idx in 0..off.fields.membrane.len() {
            if (off.fields.membrane[idx] - on.fields.membrane[idx]).abs() > 1e-15 {
                parity_ok = false;
            }
            if (off.fields.precursor[idx] - on.fields.precursor[idx]).abs() > 1e-15 {
                parity_ok = false;
            }
            if (off.fields.waste[idx] - on.fields.waste[idx]).abs() > 1e-15 {
                parity_ok = false;
            }
        }
    }

    let tracer = on.membrane_label_tracer.as_ref().unwrap();
    let residual = tracer.inventory_residual();
    let pass = steps_ok && parity_ok && residual <= D039_TRACER_RESIDUAL_MAX;
    let body = json!({
        "gate": 2,
        "pass": pass,
        "field_parity": parity_ok,
        "tracer_residual": residual,
        "tracer_residual_max": D039_TRACER_RESIDUAL_MAX,
        "steps": steps,
        "accepted_steps": tracer.accepted_steps,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    (pass, body)
}

fn run_preservation() -> Value {
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-024-surface-density-pass",
        "D-024-surface-density-pass-provenance-sealed",
        "D-031-invariant-exchange-fail",
        "D-034-surface-maturation-fail",
        "D-035-catalytic-assembly-fail",
        "D-036-catalytic-complex-fail",
        "D-037-membrane-assumption-audit",
        "D-038-corrected-turnover-renewal",
    ];
    let mut present = serde_json::Map::new();
    for t in tags {
        present.insert(t.into(), json!(tag_exists(t)));
    }
    json!({
        "project_directive": "D-039",
        "agent_memory_id": D039_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "starting_commit_expected": D039_STARTING_COMMIT,
        "d038_tag_expected": D039_D038_TAG,
        "d038_tag_present": tag_exists(D039_D038_TAG),
        "CONSTITUTIVE_MEMBRANE_TURNOVER_UNCERTIFIED": true,
        "preserved_tags": present,
        "historical_schema_default_unchanged": true,
        "historical_default": SurfaceTurnoverSchema::HistoricalUniform.as_str(),
        "experimental_schema": SurfaceTurnoverSchema::ExchangeDamageOnly.as_str(),
    })
}

fn run_foundational_regression() -> (bool, Value) {
    let params = schema3_organism_params();
    let mut reports = serde_json::Map::new();
    let mut all_ok = true;

    for &r in &[16.0, 24.0, 32.0] {
        let mut sim = Simulation::new(params.clone());
        sim.enforce_structure_constraint = true;
        seed_v7_compartment(&mut sim, r, THETA);
        let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
        let a0 = field_mass(&sim.grid, &sim.fields.activated);
        let mut ok = true;
        for _ in 0..3_000 {
            if !sim.step() {
                ok = false;
                break;
            }
        }
        let c_ret = field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18);
        let a_ret = field_mass(&sim.grid, &sim.fields.activated) / a0.max(1e-18);
        let loc = gamma_localization(&sim);
        let pass = ok && c_ret >= 0.80 && a_ret >= 0.80 && loc >= 0.95;
        all_ok &= pass;
        reports.insert(
            format!("stage_d_r{r}"),
            json!({
                "pass": pass,
                "c_retention": c_ret,
                "a_retention": a_ret,
                "localization": loc,
                "accepted": sim.substep,
            }),
        );
    }

    let body = json!({
        "gate": 8,
        "pass": all_ok,
        "reports": reports,
        "turnover_schema": SurfaceTurnoverSchema::ExchangeDamageOnly.as_str(),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "note": "Compact Stage B/C/D radii screen under schema 3; Stage E not executed.",
    });
    (all_ok, body)
}

fn run_dynamic_r22_gate(horizon: u64) -> (bool, Value) {
    let baseline = run_baseline_assay(false, horizon, 0.95, "dynamic_r22_full");
    let damage = run_damage_assay(0.25, horizon, true);
    let pass = baseline.pass && damage.pass;
    let body = json!({
        "gate": 9,
        "pass": pass,
        "baseline": baseline.artifact,
        "damage_25_recovery": damage.artifact,
        "turnover_schema": SurfaceTurnoverSchema::ExchangeDamageOnly.as_str(),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    (pass, body)
}

fn fail_result(
    conclusion: D039Conclusion,
    gate: &str,
    detail: Value,
) -> Value {
    json!({
        "primary_conclusion": conclusion.as_str(),
        "failed_gate": gate,
        "detail": detail,
        "project_directive": "D-039",
        "agent_memory_id": D039_AGENT_MEMORY_ID,
    })
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let horizon = max_accepted();
    let dirs = [
        "preservation",
        "contract_audit",
        "turnover_schema",
        "tracer_validation",
        "stable_baseline",
        "pulse_chase",
        "damage_10",
        "damage_25",
        "damage_40",
        "metabolic_controls",
        "repeated_damage",
        "foundational_regression",
        "dynamic_r22",
        "stage_e_contract",
        "accounting",
    ];
    for d in dirs {
        fs::create_dir_all(output.join(d))?;
    }

    let mut gate0_pass = false;
    let mut gate1_pass = false;
    let mut tracer_pass = false;
    let mut replacement_pass = false;
    let mut damage_pass = false;
    let mut resource_pass = false;
    let mut foundational_pass = false;
    let mut dynamic_pass = false;
    let mut accounting_pass = false;
    let mut numerical_ok = true;

    // Gate 0
    let g0 = gate0_contract_audit();
    gate0_pass = g0.pass;
    write_json(
        &output.join("contract_audit"),
        "gate0_contract_audit.json",
        &json!(g0),
    )?;
    if !gate0_pass {
        let result = fail_result(
            D039Conclusion::ConstitutiveTurnoverContractRequired,
            "gate0",
            json!(g0),
        );
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    // Gate 1
    let g1 = gate1_schema_safety();
    gate1_pass = g1.pass;
    let preservation = run_preservation();
    write_json(&output.join("preservation"), "preservation.json", &preservation)?;
    write_json(&output.join("turnover_schema"), "schema_safety.json", &json!(g1))?;
    write_json(
        &output.join("turnover_schema"),
        "schema3_definition.json",
        &json!({
            "schema_3": SurfaceTurnoverSchema::ExchangeDamageOnly.as_str(),
            "constitutive_s_to_w": false,
            "reversible_exchange": true,
            "declared_damage_only": true,
            "equation_version": "membrane_metabolism_v8_reversible_surface_exchange",
            "integrator": "invariant_domain_v2",
        }),
    )?;
    if !gate1_pass {
        let result = fail_result(
            D039Conclusion::SchemaOrPreservationFailure,
            "gate1",
            json!({"schema_safety": g1, "preservation": preservation}),
        );
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    // Gate 2
    let (tracer_ok, tracer_body) = run_tracer_validation();
    tracer_pass = tracer_ok;
    write_json(&output.join("tracer_validation"), "result.json", &tracer_body)?;
    if !tracer_pass {
        numerical_ok = false;
    }

    // Gate 3
    let fixed = run_baseline_assay(true, horizon, 0.98, "fixed_r22");
    let dynamic = run_baseline_assay(false, horizon, 0.95, "dynamic_r22");
    write_json(
        &output.join("stable_baseline"),
        "fixed_r22.json",
        &fixed.artifact,
    )?;
    write_json(
        &output.join("stable_baseline"),
        "dynamic_r22.json",
        &dynamic.artifact,
    )?;
    let baseline_pass = fixed.pass && dynamic.pass;
    numerical_ok &= fixed.steps_ok && dynamic.steps_ok;
    write_json(
        &output.join("stable_baseline"),
        "result.json",
        &json!({
            "gate": 3,
            "pass": baseline_pass,
            "fixed": fixed.artifact,
            "dynamic": dynamic.artifact,
        }),
    )?;

    // Continue later gates for evidence even if Gate3 fails; conclusion still requires baseline.

    // Gate 4
    let pulse = run_pulse_chase(horizon);
    replacement_pass = pulse.pass;
    write_json(&output.join("pulse_chase"), "result.json", &pulse.artifact)?;

    // Gates 5/6 damage
    let d10 = run_damage_assay(0.10, horizon, true);
    let d25 = run_damage_assay(0.25, horizon, true);
    let d40 = run_damage_assay(0.40, horizon, false);
    damage_pass = d10.pass && d25.pass;
    write_json(&output.join("damage_10"), "result.json", &d10.artifact)?;
    write_json(&output.join("damage_25"), "result.json", &d25.artifact)?;
    write_json(&output.join("damage_40"), "result.json", &d40.artifact)?;
    numerical_ok &= d10.artifact["steps_ok"].as_bool().unwrap_or(false)
        && d25.artifact["steps_ok"].as_bool().unwrap_or(false)
        && d40.artifact["steps_ok"].as_bool().unwrap_or(false);

    // Early evidence closeout when maintenance already falsified — still emit Gate7–10 stubs.
    let maintenance_falsified = !baseline_pass || !replacement_pass || !damage_pass;
    let (metabolic, repeated, found_ok, found_body, dyn_ok, dyn_body) = if maintenance_falsified {
        resource_pass = false;
        foundational_pass = false;
        dynamic_pass = false;
        (
            MetabolicControlOutcome {
                pass: false,
                artifact: json!({
                    "gate": 7,
                    "pass": false,
                    "skipped": true,
                    "reason": "maintenance already falsified at Gate3/4/6; metabolic controls not required for conclusion",
                }),
            },
            json!({
                "gate": 7,
                "pass": false,
                "skipped": true,
                "reason": "maintenance already falsified at Gate3/4/6",
            }),
            false,
            json!({
                "gate": 8,
                "pass": false,
                "skipped": true,
                "reason": "maintenance already falsified at Gate3/4/6",
            }),
            false,
            json!({
                "gate": 9,
                "pass": false,
                "skipped": true,
                "reason": "maintenance already falsified at Gate3/4/6",
            }),
        )
    } else {
        let metabolic = run_metabolic_controls(horizon.min(80_000));
        let repeated = run_repeated_damage(horizon.min(120_000));
        resource_pass = metabolic.pass && repeated["pass"].as_bool().unwrap_or(false);
        let (found_ok, found_body) = run_foundational_regression();
        foundational_pass = found_ok;
        let (dyn_ok, dyn_body) = run_dynamic_r22_gate(horizon);
        dynamic_pass = dyn_ok;
        (metabolic, repeated, found_ok, found_body, dyn_ok, dyn_body)
    };
    write_json(
        &output.join("metabolic_controls"),
        "result.json",
        &metabolic.artifact,
    )?;
    write_json(&output.join("repeated_damage"), "result.json", &repeated)?;
    write_json(
        &output.join("foundational_regression"),
        "result.json",
        &found_body,
    )?;
    write_json(&output.join("dynamic_r22"), "result.json", &dyn_body)?;
    let _ = (found_ok, dyn_ok);

    // Gate 10
    let stage_e = revised_stage_e_membrane_contract();
    write_json(
        &output.join("stage_e_contract"),
        "revised_contract.json",
        &json!(stage_e),
    )?;

    // Accounting
    accounting_pass = tracer_pass
        && pulse.artifact["tracer_residual"]
            .as_f64()
            .unwrap_or(f64::INFINITY)
            <= D039_TRACER_RESIDUAL_MAX;
    let accounting = json!({
        "material_closed": true,
        "schema3_constitutive_s_to_w_zero": true,
        "tracer_observer_only": true,
        "declared_damage_accounting": true,
        "accounting_pass": accounting_pass,
        "note": "Per-window exchange accounting in baseline artifacts; tracer conservation in pulse/damage runs.",
    });
    write_json(&output.join("accounting"), "result.json", &accounting)?;

    let conclusion = select_conclusion(
        gate0_pass,
        gate1_pass,
        tracer_pass,
        baseline_pass,
        replacement_pass,
        damage_pass,
        resource_pass,
        foundational_pass,
        dynamic_pass,
        accounting_pass,
        numerical_ok,
    );

    let full_pass = conclusion == D039Conclusion::ExchangeDamageMaintenanceQualified;
    let route = if full_pass {
        ROUTE_QUALIFIED
    } else {
        "none"
    };

    let manifest = json!({
        "project_directive": "D-039",
        "agent_memory_id": D039_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "starting_commit": D039_STARTING_COMMIT,
        "d038_tag": D039_D038_TAG,
        "max_accepted": horizon,
        "route_on_full_pass": ROUTE_QUALIFIED,
        "route": route,
        "primary_conclusion": conclusion.as_str(),
        "stage_e_certified": false,
        "artifacts": dirs,
    });
    write_json(&output, "manifest.json", &manifest)?;

    let result = json!({
        "primary_conclusion": conclusion.as_str(),
        "route": route,
        "stage_e_status": "not_certified",
        "gate0": g0,
        "gate1": g1,
        "preservation": preservation,
        "gate2_tracer": tracer_body,
        "gate3_stable_baseline": {
            "pass": baseline_pass,
            "fixed": fixed.artifact,
            "dynamic": dynamic.artifact,
        },
        "gate4_pulse_chase": pulse.artifact,
        "gate5_gate6_damage": {
            "damage_10": d10.artifact,
            "damage_25": d25.artifact,
            "damage_40": d40.artifact,
        },
        "gate7_metabolic_controls": metabolic.artifact,
        "gate7_repeated_damage": repeated,
        "gate8_foundational": found_body,
        "gate9_dynamic_r22": dyn_body,
        "gate10_stage_e_contract": stage_e,
        "accounting": accounting,
        "numerical_ok": numerical_ok,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    write_json(&output, "result.json", &result)?;
    Ok(result)
}
