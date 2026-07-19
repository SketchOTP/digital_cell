//! D-041 structural A-retention basin-accessibility qualification pipeline.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{
    InterventionAction, SimParams, SurfaceTurnoverSchema, TRANSPORT_SCHEMA_VERSION_V3,
};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::surface_balance_q;
use chemistry_core::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, classify_damage_repair,
    revised_stage_e_membrane_contract, v8_schema3_params, DamageRepairClass,
    D039_TRACER_RESIDUAL_MAX,
};
use chemistry_core::d040_analysis::{
    audit_exchange_sample, classify_equilibrium_audit, earliest_causal_divergence,
    frozen_kinetics_ok, j_predicted, required_p_for_theta, theta_eq, ChronologyClass,
    ChronologyWindow, D040_K_FROZEN,
};
use chemistry_core::d041_analysis::{
    apply_structural_a_retention, bracket_intermediate, build_rho_candidates,
    mature_membrane_nonredundant, retention_candidate_passes, select_weakest_passing_rho,
    transport_schema_name, RetentionCandidateMetrics, D041_AGENT_MEMORY_ID, D041_ARCHITECTURE_PASS,
    D041Conclusion, D041_D040_TAG, D041_GATE0_HORIZON, D041_MAX_ACCEPTED, D041_RECORD_EXCHANGE,
    D041_REPAIR_P_MIN, D041_REPLACEMENT_MIN, D041_RHO_SCREEN, D041_S_DRIFT_MAX,
    D041_STARTING_COMMIT,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::{apply_declared_membrane_arc_damage, apply_intervention};
use chemistry_core::membrane_label_tracer::MembraneLabelTracer;
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

const WINDOW: u64 = 1_000;
const MEASURE_WINDOW: u64 = 500;
const RADIUS: f64 = 22.0;
const THETA: f64 = 0.6;
const D041_NET_S_FLOW_MAX: f64 = 1e-4;
const D040_BUDGET_WINDOW: u64 = 500;
const REPEATED_DAMAGE_INTERVAL: u64 = 20_000;

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
    std::env::var("D041_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D041_MAX_ACCEPTED)
}

fn gate0_horizon() -> u64 {
    std::env::var("D041_GATE0_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D041_GATE0_HORIZON)
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

fn schema3_organism_params(rho_a: Option<f64>) -> SimParams {
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
    if let Some(r) = rho_a {
        apply_structural_a_retention(&mut params, r);
    }
    params
}

fn new_sim(enforce_fixed: bool, with_tracer: bool, rho_a: Option<f64>) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params(rho_a));
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
            .all(|m| m.normalized_net_flow.abs() <= D041_NET_S_FLOW_MAX)
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
    rho_a: Option<f64>,
) -> BaselineOutcome {
    let mut sim = new_sim(enforce_fixed, false, rho_a);
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

fn settle_dynamic_to_balance(settle_horizon: u64, with_tracer: bool, rho_a: Option<f64>) -> (Simulation, bool, u64) {
    let mut sim = new_sim(false, with_tracer, rho_a);
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

fn run_pulse_chase(horizon: u64, rho_a: Option<f64>) -> PulseChaseOutcome {
    // Settle on at most half the budget so chase always has room to run.
    let settle_budget = (horizon / 2).max(3 * WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, true, rho_a);
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
        if replacement >= D041_REPLACEMENT_MIN {
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
        && replacement >= D041_REPLACEMENT_MIN
        && s_drift <= D041_S_DRIFT_MAX
        && tracer_residual <= D039_TRACER_RESIDUAL_MAX;

    let artifact = json!({
        "gate": 4,
        "pass": pass,
        "replacement_fraction": replacement,
        "replacement_min": D041_REPLACEMENT_MIN,
        "s_drift": s_drift,
        "s_drift_max": D041_S_DRIFT_MAX,
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

fn run_damage_assay(fraction: f64, horizon: u64, mandatory: bool, rho_a: Option<f64>) -> DamageOutcome {
    let settle_budget = (horizon / 2).max(3 * WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, true, rho_a);
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
    rho_a: Option<f64>,
) -> MetabolicControlOutcome {
    let settle_budget = (horizon / 2).max(3 * WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, false, rho_a);
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

fn run_metabolic_controls(horizon: u64, rho_a: Option<f64>) -> MetabolicControlOutcome {
    let normal = run_metabolic_control("A_normal", |_| {}, horizon, rho_a);
    let no_a = run_metabolic_control(
        "B_no_activation",
        |sim| {
            sim.params.k_d008_activation = 0.0;
        },
        horizon,
        rho_a,
    );
    let no_p = run_metabolic_control(
        "C_no_precursor_synthesis",
        |sim| {
            sim.d026_disable_precursor_synthesis = true;
        },
        horizon,
        rho_a,
    );
    let shutdown = run_metabolic_control(
        "D_shutdown_reservoir",
        |sim| {
            apply_intervention(
                &sim.grid,
                &mut sim.fields,
                &InterventionAction::ShutdownReservoir,
                &mut sim.params,
            );
            sim.params.n_reservoir = 0.0;
            sim.params.f_reservoir = 0.0;
        },
        horizon,
        rho_a,
    );

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

fn run_repeated_damage(horizon: u64, rho_a: Option<f64>) -> Value {
    let mut sim_normal = new_sim(false, false, rho_a);
    let mut sim_no_a = new_sim(false, false, rho_a);
    sim_no_a.params.k_d008_activation = 0.0;
    let mut sim_no_p = new_sim(false, false, rho_a);
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
    zero_phi: bool,
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
    if ctrl.zero_phi {
        sim.params.k_phi = 0.0;
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
    localization: f64,
    accepted: u64,
}

fn run_budget_window(sim: &mut Simulation, ctrl: &ControlSpec) -> (WindowBudget, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut steps_ok = true;
    let mut s_sum = 0.0;
    let mut n = 0u64;
    for _ in 0..D040_BUDGET_WINDOW {
        apply_pre_step_controls(sim, ctrl);
        if !sim.step() {
            steps_ok = false;
            break;
        }
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
    let p_act = precursor_activity(p_int, sim.params.p_reference);
    let theta = mean_interface_theta(sim);
    let teq = theta_eq(D040_K_FROZEN, p_act);
    (
        WindowBudget {
            s_mass: mean_s,
            theta,
            theta_eq: teq,
            p_activity: p_act,
            p_total: field_mass(&sim.grid, &sim.fields.precursor),
            p_internal: p_int,
            a_total: field_mass(&sim.grid, &sim.fields.activated),
            a_internal: mean_interior(sim, &sim.fields.activated),
            forward: wl.exchange_forward,
            reverse: wl.exchange_reverse,
            net_exchange: wl.exchange_net,
            normalized_s_flow: wl.exchange_net / mean_s.max(1e-18),
            p_synthesis: wl.precursor_synthesis_delta,
            localization: gamma_localization(sim),
            accepted: sim.substep,
        },
        steps_ok,
    )
}

fn run_tracer_validation(rho_a: Option<f64>) -> (bool, Value) {
    let params = schema3_organism_params(rho_a);
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
    json!({
        "project_directive": "D-041",
        "agent_memory_id": D041_AGENT_MEMORY_ID,
        "record": D041_RECORD_EXCHANGE,
        "source_commit": git_commit_hash(),
        "starting_commit_expected": D041_STARTING_COMMIT,
        "d040_tag_expected": D041_D040_TAG,
        "d040_tag_present": tag_exists(D041_D040_TAG),
        "frozen_kinetics_ok": frozen_kinetics_ok(),
        "alpha": D031_ALPHA_FROZEN,
        "beta": D031_BETA_FROZEN,
        "K": D040_K_FROZEN,
        "constitutive_s_to_w_zero": true,
        "reversible_exchange_frozen": true,
    })
}

fn gate0_route_confirmation(horizon: u64) -> (bool, Value) {
    // Governed default 25k; shorten with D041_GATE0_HORIZON and/or D041_MAX_ACCEPTED for smoke.
    let gate_horizon = gate0_horizon().min(horizon);
    let mut sim = new_sim(false, false, None);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let ctrl = ControlSpec {
        name: "baseline",
        ..Default::default()
    };
    let mut ok = true;
    let mut chron = Vec::new();
    let mut windows = Vec::new();
    // Do not early-exit: Gate 0 requires complete windows at the governed horizon
    // unless an earlier terminal step failure occurs.
    while sim.substep < gate_horizon && ok {
        let (w, s_ok) = run_budget_window(&mut sim, &ctrl);
        ok &= s_ok;
        let a_ret = w.a_total / a0;
        if chron.len() < 24 {
            chron.push(ChronologyWindow {
                index: chron.len(),
                theta: w.theta,
                theta_eq: w.theta_eq,
                p: w.p_activity,
                a: w.a_internal,
                a_retention: a_ret,
                p_synthesis: w.p_synthesis,
                p_leakage: (w.p_total - w.p_internal).abs(),
                a_leakage: (w.a_total - w.a_internal).abs(),
                net_exchange: w.net_exchange,
                permeability_proxy: (-sim.params.beta_a * w.theta).exp(),
                precursor_synthesis_demand: w.p_synthesis.abs(),
            });
        }
        windows.push(json!({
            "theta": w.theta,
            "p_activity": w.p_activity,
            "a_retention": a_ret,
            "net_exchange": w.net_exchange,
            "normalized_s_flow": w.normalized_s_flow,
            "accepted": sim.substep,
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
    let mut seed = new_sim(false, false, None);
    let a_healthy = mean_interior(&seed, &seed.fields.activated).max(0.1);
    let pre = gate_horizon.min(2_000);
    let run_ctrl = |spec: ControlSpec| -> bool {
        let mut s = new_sim(false, false, None);
        let base = ControlSpec::default();
        let mut steps_ok = true;
        while s.substep < pre && steps_ok {
            let (_, ok) = run_budget_window(&mut s, &base);
            steps_ok &= ok;
        }
        apply_control_params(&mut s, &spec);
        let mut last_theta = 0.0;
        let mut last_p = 0.0;
        let end = s.substep.saturating_add(gate_horizon.min(3_000));
        while s.substep < end && steps_ok {
            let (w, ok) = run_budget_window(&mut s, &spec);
            steps_ok &= ok;
            last_theta = w.theta;
            last_p = w.p_activity;
        }
        steps_ok && last_theta >= 0.45 && last_p >= repair_p * 0.5
    };
    let p_clamp_ok = run_ctrl(ControlSpec {
        name: "sufficient_p",
        clamp_p_activity: Some(repair_p),
        ..Default::default()
    });
    let a_clamp_ok = run_ctrl(ControlSpec {
        name: "healthy_a",
        clamp_a: Some(a_healthy),
        ..Default::default()
    });
    let perm_ok = run_ctrl(ControlSpec {
        name: "healthy_perm",
        freeze_surface: true,
        ..Default::default()
    });

    let mut low_healthy = Vec::new();
    for (label, scale) in [("low_s", 0.5f64), ("healthy_s", 1.1f64)] {
        let mut s = new_sim(false, false, None);
        for v in s.fields.membrane.iter_mut() {
            *v *= scale;
        }
        let mut steps_ok = true;
        let end = s.substep.saturating_add(gate_horizon.min(3_000));
        while s.substep < end && steps_ok {
            let (w, ok) = run_budget_window(&mut s, &ControlSpec::default());
            steps_ok &= ok;
            if s.substep >= end.saturating_sub(D040_BUDGET_WINDOW) {
                low_healthy.push(json!({
                    "label": label,
                    "theta": w.theta,
                    "healthy": w.theta >= 0.5 && w.localization >= 0.95,
                }));
                break;
            }
        }
    }
    let basins_distinguishable = low_healthy.len() >= 2
        && low_healthy
            .iter()
            .any(|v| v["healthy"].as_bool() == Some(true))
        && low_healthy
            .iter()
            .any(|v| v["healthy"].as_bool() == Some(false));

    let chron_ok = divergence == ChronologyClass::AProductionDecline;
    let pass = ok
        && frozen_kinetics_ok()
        && tag_exists(D041_D040_TAG)
        && parity_ok
        && chron_ok
        && p_clamp_ok
        && a_clamp_ok
        && perm_ok
        && basins_distinguishable
        && chron.len() >= 3;

    let body = json!({
        "gate": 0,
        "pass": pass,
        "horizon": gate_horizon,
        "earliest_divergence": divergence.as_str(),
        "exchange_parity": parity.as_str(),
        "parity_ok": parity_ok,
        "controls": {
            "sufficient_p": p_clamp_ok,
            "healthy_a": a_clamp_ok,
            "healthy_permeability": perm_ok,
        },
        "basin_multistart": low_healthy,
        "basins_distinguishable": basins_distinguishable,
        "windows": windows,
        "steps_ok": ok,
        "record": D041_RECORD_EXCHANGE,
    });
    (pass, body)
}

fn gate1_transport_schema(rho_a: f64) -> Value {
    let params = schema3_organism_params(Some(rho_a));
    json!({
        "gate": 1,
        "pass": params.transport_schema_version == TRANSPORT_SCHEMA_VERSION_V3,
        "transport_schema": transport_schema_name(&params),
        "transport_schema_version": params.transport_schema_version,
        "rho_a": params.rho_a,
        "conservation_note": "A-only structural attenuation; unit tests cover antisymmetry and mass closure.",
    })
}

fn evaluate_rho_candidate(rho_a: f64, horizon: u64) -> (RetentionCandidateMetrics, Value) {
    // Screen for basin accessibility from depleted S — structural retention is
    // relevant at low θ, not as a mature-membrane Stage-E retention substitute.
    let h = horizon.min(40_000).max(2_000.min(horizon));
    let mut sim = new_sim(false, false, Some(rho_a));
    redistribute_ps(&mut sim, 1.0, 0.0); // zero-S conservative bootstrap
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let mut ok = true;
    let mut chron = Vec::new();
    let mut max_p: f64 = 0.0;
    let mut last_theta = 0.0;
    let mut last_loc = 0.0;
    let mut last_s = s0;
    let mut early_s = s0;
    let mut a_final = a0;
    let mut windows_ok_acct = 0u32;
    let end = sim.substep.saturating_add(h);
    let mut window_i = 0u64;
    while sim.substep < end && ok {
        let (w, s_ok) = run_budget_window(
            &mut sim,
            &ControlSpec {
                name: "rho_zero_s_bootstrap",
                ..Default::default()
            },
        );
        ok &= s_ok;
        window_i += 1;
        if window_i == 1 {
            early_s = w.s_mass;
        }
        last_theta = w.theta;
        last_loc = w.localization;
        last_s = w.s_mass;
        max_p = max_p.max(w.p_activity);
        a_final = w.a_total;
        if w.normalized_s_flow.abs() <= 1.0 && w.s_mass.is_finite() {
            windows_ok_acct += 1;
        }
        if chron.len() < 12 {
            chron.push(ChronologyWindow {
                index: chron.len(),
                theta: w.theta,
                theta_eq: w.theta_eq,
                p: w.p_activity,
                a: w.a_internal,
                a_retention: w.a_total / a0,
                p_synthesis: w.p_synthesis,
                p_leakage: (w.p_total - w.p_internal).abs(),
                a_leakage: (w.a_total - w.a_internal).abs(),
                net_exchange: w.net_exchange,
                permeability_proxy: (-sim.params.beta_a * w.theta).exp(),
                precursor_synthesis_demand: w.p_synthesis.abs(),
            });
        }
    }
    let divergence = earliest_causal_divergence(&chron);
    // Irreversible P/S collapse: late S fails to grow from zero/low and stays near empty.
    let s_collapsed = last_s <= early_s.max(1e-9) * 1.05 && last_theta < 0.25;
    let a_decline_precedes_collapse =
        matches!(divergence, ChronologyClass::AProductionDecline) && s_collapsed;
    let s_toward_healthy = last_s > s0 + 1e-6
        && last_theta >= 0.35
        && last_loc >= 0.90
        && last_s > early_s * 1.10;
    let accounting_ok = windows_ok_acct >= 2 && last_s.is_finite() && last_theta.is_finite();
    let metrics = RetentionCandidateMetrics {
        a_decline_precedes_collapse,
        endogenous_p: max_p,
        s_toward_healthy,
        accounting_ok,
        numerical_ok: ok,
    };
    let nonredundant = mature_membrane_nonredundant(
        rho_a,
        schema3_organism_params(Some(rho_a)).beta_a,
        1.0,
    );
    let pass = retention_candidate_passes(metrics) && nonredundant;
    let detail = json!({
        "rho_a": rho_a,
        "pass": pass,
        "nonredundant": nonredundant,
        "metrics": {
            "a_decline_precedes_collapse": metrics.a_decline_precedes_collapse,
            "endogenous_p": metrics.endogenous_p,
            "s_toward_healthy": metrics.s_toward_healthy,
            "accounting_ok": metrics.accounting_ok,
            "numerical_ok": metrics.numerical_ok,
        },
        "divergence": divergence.as_str(),
        "s_collapsed": s_collapsed,
        "assay": {
            "mode": "zero_s_conservative_bootstrap",
            "s0": s0,
            "early_s": early_s,
            "late_s": last_s,
            "late_theta": last_theta,
            "late_localization": last_loc,
            "a_retention": a_final / a0,
            "max_p_activity": max_p,
            "horizon": h,
            "transport_schema": transport_schema_name(&schema3_organism_params(Some(rho_a))),
        },
    });
    (metrics, detail)
}

fn gate2_screen_rho(horizon: u64) -> (Option<f64>, bool, Value) {
    let mut rows = Vec::new();
    let mut passing = Vec::new();
    for rho in D041_RHO_SCREEN {
        let (metrics, detail) = evaluate_rho_candidate(rho, horizon);
        let nonredundant = mature_membrane_nonredundant(
            rho,
            schema3_organism_params(Some(rho)).beta_a,
            1.0,
        );
        let pass = retention_candidate_passes(metrics) && nonredundant;
        passing.push((rho, pass));
        rows.push(detail);
    }
    let mut candidates = build_rho_candidates(&D041_RHO_SCREEN, None);
    if let Some(weakest) = select_weakest_passing_rho(&passing) {
        if let Some((failing, _)) = passing.iter().find(|(_, ok)| !ok) {
            if *failing > weakest {
                let mid = bracket_intermediate(*failing, weakest);
                let (_, bracket_detail) = evaluate_rho_candidate(mid, horizon);
                let m = bracket_detail["metrics"].clone();
                let metrics = RetentionCandidateMetrics {
                    a_decline_precedes_collapse: m["a_decline_precedes_collapse"]
                        .as_bool()
                        .unwrap_or(true),
                    endogenous_p: m["endogenous_p"].as_f64().unwrap_or(0.0),
                    s_toward_healthy: m["s_toward_healthy"].as_bool().unwrap_or(false),
                    accounting_ok: m["accounting_ok"].as_bool().unwrap_or(false),
                    numerical_ok: m["numerical_ok"].as_bool().unwrap_or(false),
                };
                let pass = retention_candidate_passes(metrics)
                    && mature_membrane_nonredundant(
                        mid,
                        schema3_organism_params(Some(mid)).beta_a,
                        1.0,
                    );
                passing.push((mid, pass));
                rows.push(bracket_detail);
                candidates = build_rho_candidates(&D041_RHO_SCREEN, Some(mid));
            }
        }
    }
    let selected = select_weakest_passing_rho(&passing);
    let pass = selected.is_some();
    let body = json!({
        "gate": 2,
        "pass": pass,
        "selected_rho_a": selected,
        "candidates": candidates,
        "screen": rows,
        "selection_rule": "largest_passing_rho_weakest_retention",
    });
    (selected, pass, body)
}

/// Conservatively redistribute total precursor+membrane mass between P and S fractions.
fn redistribute_ps(sim: &mut Simulation, p_frac: f64, s_frac: f64) {
    let p_mass = field_mass(&sim.grid, &sim.fields.precursor);
    let s_mass = field_mass(&sim.grid, &sim.fields.membrane);
    let total = (p_mass + s_mass).max(1e-18);
    let target_p = total * p_frac;
    let target_s = total * s_frac;
    let p_scale = if p_mass > 1e-18 {
        target_p / p_mass
    } else {
        0.0
    };
    let s_scale = if s_mass > 1e-18 {
        target_s / s_mass
    } else {
        0.0
    };
    for v in sim.fields.precursor.iter_mut() {
        *v = (*v * p_scale).max(0.0);
    }
    for v in sim.fields.membrane.iter_mut() {
        *v = (*v * s_scale).max(0.0);
    }
    // Zero-S bootstrap: if S was emptied, seed P with the conserved total (already in P).
    if s_frac <= 0.0 {
        for v in sim.fields.membrane.iter_mut() {
            *v = 0.0;
        }
        if p_mass <= 1e-18 {
            let n_in = sim
                .grid
                .dish_mask
                .iter()
                .zip(sim.fields.structure.iter())
                .filter(|(m, phi)| **m && **phi >= 0.5)
                .count()
                .max(1) as f64;
            for idx in 0..sim.fields.precursor.len() {
                if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
                    sim.fields.precursor[idx] = total / n_in;
                }
            }
        }
    }
}

fn gate3_basin_map(horizon: u64, rho_a: f64) -> (bool, Value) {
    let h = horizon.min(25_000).max(3_000.min(horizon));
    let mut starts = Vec::new();
    let mut run = |label: &'static str, mutate: &dyn Fn(&mut Simulation)| -> Value {
        let mut sim = new_sim(false, false, Some(rho_a));
        mutate(&mut sim);
        let mut ok = true;
        let mut last_theta = 0.0;
        let mut last_loc = 0.0;
        let mut last_a = 0.0;
        let mut last_p = 0.0;
        let end = sim.substep.saturating_add(h);
        while sim.substep < end && ok {
            let (w, s_ok) = run_budget_window(
                &mut sim,
                &ControlSpec {
                    name: label,
                    ..Default::default()
                },
            );
            ok &= s_ok;
            last_theta = w.theta;
            last_loc = w.localization;
            last_a = w.a_internal;
            last_p = w.p_activity;
        }
        let formed_s = total_surface_mass(&sim.grid, &sim.fields.membrane) > 1e-6;
        let healthy = last_theta >= 0.45 && last_loc >= 0.90 && formed_s;
        json!({
            "label": label,
            "healthy": healthy,
            "formed_s": formed_s,
            "theta": last_theta,
            "localization": last_loc,
            "a_internal": last_a,
            "p_activity": last_p,
            "steps_ok": ok,
        })
    };
    starts.push(run("100p_0s", &|sim| redistribute_ps(sim, 1.0, 0.0)));
    starts.push(run("95p_5s", &|sim| redistribute_ps(sim, 0.95, 0.05)));
    starts.push(run("75p_25s", &|sim| redistribute_ps(sim, 0.75, 0.25)));
    starts.push(run("historical_failed", &|sim| {
        for v in sim.fields.membrane.iter_mut() {
            *v *= 0.15;
        }
    }));
    starts.push(run("near_separatrix", &|sim| {
        for v in sim.fields.membrane.iter_mut() {
            *v *= 0.55;
        }
    }));
    starts.push(run("historical_healthy", &|_| {}));

    let zero_s_ok = starts
        .iter()
        .find(|s| s["label"].as_str() == Some("100p_0s"))
        .and_then(|s| s["formed_s"].as_bool())
        .unwrap_or(false);
    let low_s_ok = starts
        .iter()
        .find(|s| s["label"].as_str() == Some("95p_5s"))
        .and_then(|s| s["formed_s"].as_bool())
        .unwrap_or(false);
    let healthy_starts: Vec<f64> = starts
        .iter()
        .filter(|s| s["healthy"].as_bool() == Some(true))
        .filter_map(|s| s["theta"].as_f64())
        .collect();
    let common_attractor = healthy_starts.len() >= 2
        && healthy_starts
            .iter()
            .all(|t| (t - healthy_starts[0]).abs() < 0.15);
    let pass = zero_s_ok && low_s_ok && common_attractor && starts.iter().all(|s| s["steps_ok"].as_bool() != Some(false));
    let body = json!({
        "gate": 3,
        "pass": pass,
        "zero_s_bootstrap": zero_s_ok,
        "low_s_bootstrap": low_s_ok,
        "common_healthy_attractor": common_attractor,
        "starts": starts,
        "conservative_multistart": true,
        "rho_a": rho_a,
    });
    (pass, body)
}

fn gate4_membrane_necessity(horizon: u64, rho_a: f64) -> (bool, Value) {
    let h = horizon.min(3_000);
    let pre = h.min(2_000);
    let run_necessity = |name: &'static str, mut spec: ControlSpec| -> Value {
        spec.name = name;
        let mut sim = new_sim(false, false, Some(rho_a));
        let base = ControlSpec::default();
        let mut ok = true;
        while sim.substep < pre && ok {
            let (_, s_ok) = run_budget_window(&mut sim, &base);
            ok &= s_ok;
        }
        apply_control_params(&mut sim, &spec);
        let s_ref = total_surface_mass(&sim.grid, &sim.fields.membrane);
        let end = sim.substep.saturating_add(h);
        let mut last_theta = 0.0;
        while sim.substep < end && ok {
            let (w, s_ok) = run_budget_window(&mut sim, &spec);
            ok &= s_ok;
            last_theta = w.theta;
        }
        let s_final = total_surface_mass(&sim.grid, &sim.fields.membrane);
        let maintains = ok && last_theta >= 0.45 && s_final >= 0.8 * s_ref;
        json!({
            "name": name,
            "maintains_maintenance": maintains,
            "theta_final": last_theta,
            "s_ratio": s_final / s_ref.max(1e-18),
            "steps_ok": ok,
        })
    };
    let rows = vec![
        run_necessity(
            "no_exchange",
            ControlSpec {
                disable_exchange: true,
                ..Default::default()
            },
        ),
        run_necessity(
            "no_precursor_synthesis",
            ControlSpec {
                disable_precursor_synthesis: true,
                ..Default::default()
            },
        ),
        run_necessity(
            "no_phi_interface",
            ControlSpec {
                zero_phi: true,
                ..Default::default()
            },
        ),
    ];
    let pass = rows
        .iter()
        .all(|r| !r["maintains_maintenance"].as_bool().unwrap_or(true));
    let body = json!({
        "gate": 4,
        "pass": pass,
        "controls": rows,
        "constitutive_s_to_w_zero": true,
        "note": "Maintenance must fail when exchange, P synthesis, or phi interface is removed.",
    });
    (pass, body)
}

fn run_foundational_regression(rho_a: Option<f64>) -> (bool, Value) {
    let params = schema3_organism_params(rho_a);
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

fn run_dynamic_r22_gate(horizon: u64, rho_a: Option<f64>) -> (bool, Value) {
    let baseline = run_baseline_assay(false, horizon, 0.95, "dynamic_r22_full", rho_a);
    let damage = run_damage_assay(0.25, horizon, true, rho_a);
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

fn select_d041_conclusion(
    gate0: bool,
    gate1: bool,
    gate2: bool,
    gate3: bool,
    gate4: bool,
    baseline: bool,
    pulse: bool,
    damage: bool,
    resource: bool,
    foundational: bool,
    dynamic: bool,
    stage_e_ok: bool,
    accounting: bool,
    numerical: bool,
) -> D041Conclusion {
    if !gate0 {
        return D041Conclusion::D040RouteNotReproduced;
    }
    if !gate1 {
        return D041Conclusion::StructuralRetentionImplementationFailure;
    }
    if !gate2 {
        return D041Conclusion::StructuralARetentionNotSufficient;
    }
    if !gate3 {
        return D041Conclusion::BasinAccessibilityNotRecovered;
    }
    if !gate4 {
        return D041Conclusion::MembraneCausalityLost;
    }
    if !numerical {
        return D041Conclusion::NumericalFailure;
    }
    if !accounting {
        return D041Conclusion::AccountingFailure;
    }
    if !pulse {
        return D041Conclusion::ContinuousReplacementNotRecovered;
    }
    if !damage {
        return D041Conclusion::DamageRepairNotRecovered;
    }
    if !baseline {
        return D041Conclusion::BasinAccessibilityNotRecovered;
    }
    if !resource {
        return D041Conclusion::ResourceDependenceNotEstablished;
    }
    if !foundational {
        return D041Conclusion::FoundationalRegression;
    }
    if !dynamic {
        return D041Conclusion::Fail;
    }
    if !stage_e_ok {
        return D041Conclusion::StageEMembraneContractFailure;
    }
    D041Conclusion::BasinAccessibleMembraneMaintenanceQualified
}

fn fail_result(conclusion: D041Conclusion, gate: &str, detail: Value) -> Value {
    json!({
        "primary_conclusion": conclusion.as_str(),
        "failed_gate": gate,
        "detail": detail,
        "project_directive": "D-041",
        "agent_memory_id": D041_AGENT_MEMORY_ID,
    })
}

/// Focused ρ_A bootstrap diagnostic (zero-S / low-S) for Gate-2 interpretation.
pub fn diagnose_rho_bootstrap(
    output: &Path,
    steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(output.join("retention_candidates"))?;
    let mut rows = Vec::new();
    for &rho in &[1.0_f64, 0.4, 0.2, 0.05] {
        for &s_frac in &[0.0_f64, 0.05] {
            let mut sim = new_sim(false, false, Some(rho));
            redistribute_ps(&mut sim, 1.0 - s_frac, s_frac);
            let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
            let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
            let mut ok = true;
            let mut last_theta = 0.0;
            let mut last_loc = 0.0;
            let mut last_s = s0;
            let mut max_p = 0.0_f64;
            let end = sim.substep.saturating_add(steps);
            while sim.substep < end && ok {
                let (w, s_ok) = run_budget_window(
                    &mut sim,
                    &ControlSpec {
                        name: "diag",
                        ..Default::default()
                    },
                );
                ok &= s_ok;
                last_theta = w.theta;
                last_loc = w.localization;
                last_s = w.s_mass;
                max_p = max_p.max(w.p_activity);
            }
            let a_ret = field_mass(&sim.grid, &sim.fields.activated) / a0;
            rows.push(json!({
                "rho_a": rho,
                "s_frac0": s_frac,
                "s0": s0,
                "late_s": last_s,
                "late_theta": last_theta,
                "late_localization": last_loc,
                "max_p": max_p,
                "a_retention": a_ret,
                "steps_ok": ok,
                "accepted": sim.substep,
                "healthyish": last_theta >= 0.45 && last_loc >= 0.90 && last_s > s0 + 1.0,
            }));
            eprintln!(
                "D-041 diag rho={rho:.2} s0_frac={s_frac:.2} -> theta={last_theta:.3} S={last_s:.2} a_ret={a_ret:.3} p={max_p:.3}"
            );
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
    }
    let body = json!({
        "diagnostic": "rho_bootstrap",
        "steps": steps,
        "rows": rows,
    });
    write_json(
        &output.join("retention_candidates"),
        "bootstrap_diagnostic.json",
        &body,
    )?;
    Ok(body)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let horizon = max_accepted();
    let dirs = [
        "preservation",
        "route_confirmation",
        "transport_schema",
        "retention_candidates",
        "basin_map",
        "membrane_necessity",
        "stable_baseline",
        "pulse_chase",
        "damage_10",
        "damage_25",
        "damage_40",
        "resource_controls",
        "foundational_regression",
        "dynamic_r22",
        "stage_e_membrane_contract",
        "accounting",
    ];
    for d in dirs {
        fs::create_dir_all(output.join(d))?;
    }

    // Gate 0 — D-040 route confirmation on historical transport (no ρ_A yet).
    eprintln!(
        "D-041 Gate0 route confirmation horizon={} gate0_horizon={}",
        horizon,
        gate0_horizon().min(horizon)
    );
    let _ = std::io::Write::flush(&mut std::io::stderr());
    let preservation = run_preservation();
    write_json(&output.join("preservation"), "preservation.json", &preservation)?;
    let (gate0_pass, route_body) = gate0_route_confirmation(horizon);
    eprintln!("D-041 Gate0 pass={}", gate0_pass);
    let _ = std::io::Write::flush(&mut std::io::stderr());
    write_json(
        &output.join("route_confirmation"),
        "result.json",
        &route_body,
    )?;
    if !gate0_pass {
        let result = fail_result(
            D041Conclusion::D040RouteNotReproduced,
            "gate0",
            json!({"preservation": preservation, "route_confirmation": route_body}),
        );
        write_json(&output, "result.json", &result)?;
        write_json(
            &output,
            "manifest.json",
            &json!({
                "project_directive": "D-041",
                "primary_conclusion": result["primary_conclusion"],
                "stage_e_certified": false,
            }),
        )?;
        return Ok(result);
    }

    // Gate 2 — screen ρ_A (Gate 1 artifact written after selection).
    let (selected_rho, gate2_pass, gate2_body) = gate2_screen_rho(horizon);
    write_json(
        &output.join("retention_candidates"),
        "result.json",
        &gate2_body,
    )?;
    if !gate2_pass {
        let nonredundant_any = D041_RHO_SCREEN.iter().any(|&r| {
            mature_membrane_nonredundant(r, schema3_organism_params(Some(r)).beta_a, 1.0)
        });
        let conclusion = if nonredundant_any {
            D041Conclusion::StructuralARetentionNotSufficient
        } else {
            D041Conclusion::MembraneCausalityLost
        };
        let result = fail_result(conclusion, "gate2", gate2_body.clone());
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }
    let rho_a = selected_rho.unwrap();
    let rho_ctx = Some(rho_a);

    // Gate 1 — transport schema artifact for selected ρ_A.
    let transport = gate1_transport_schema(rho_a);
    let gate1_pass = transport["pass"].as_bool().unwrap_or(false);
    write_json(
        &output.join("transport_schema"),
        "result.json",
        &transport,
    )?;
    if !gate1_pass {
        let result = fail_result(
            D041Conclusion::StructuralRetentionImplementationFailure,
            "gate1",
            transport,
        );
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    // Gate 3 — basin map multistarts.
    let (gate3_pass, basin_body) = gate3_basin_map(horizon, rho_a);
    write_json(&output.join("basin_map"), "result.json", &basin_body)?;
    if !gate3_pass {
        let result = fail_result(
            D041Conclusion::BasinAccessibilityNotRecovered,
            "gate3",
            basin_body,
        );
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    // Gate 4 — membrane necessity controls.
    let (gate4_pass, necessity_body) = gate4_membrane_necessity(horizon, rho_a);
    write_json(
        &output.join("membrane_necessity"),
        "result.json",
        &necessity_body,
    )?;
    if !gate4_pass {
        let result = fail_result(
            D041Conclusion::MembraneCausalityLost,
            "gate4",
            necessity_body,
        );
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    // Gate 5 — stable baseline + pulse-chase horizons.
    let fixed = run_baseline_assay(true, horizon, 0.98, "fixed_r22", rho_ctx);
    let dynamic = run_baseline_assay(false, horizon, 0.95, "dynamic_r22", rho_ctx);
    let baseline_pass = fixed.pass && dynamic.pass;
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
    let mut pulse_horizons = [25_000u64, 50_000, 100_000, 200_000];
    pulse_horizons.sort_unstable();
    let mut pulse_results = Vec::new();
    let mut replacement_pass = true;
    for ph in pulse_horizons {
        if ph > horizon {
            continue;
        }
        let pulse = run_pulse_chase(ph, rho_ctx);
        replacement_pass &= pulse.pass;
        pulse_results.push(json!({
            "horizon": ph,
            "pass": pulse.pass,
            "replacement_fraction": pulse.replacement,
            "s_drift": pulse.s_drift,
        }));
    }
    write_json(
        &output.join("pulse_chase"),
        "result.json",
        &json!({
            "gate": 5,
            "pass": replacement_pass,
            "horizons": pulse_results,
        }),
    )?;

    // Gates 6 — damage.
    let d10 = run_damage_assay(0.10, horizon, true, rho_ctx);
    let d25 = run_damage_assay(0.25, horizon, true, rho_ctx);
    let d40 = run_damage_assay(0.40, horizon, false, rho_ctx);
    let damage_pass = d10.pass && d25.pass;
    write_json(&output.join("damage_10"), "result.json", &d10.artifact)?;
    write_json(&output.join("damage_25"), "result.json", &d25.artifact)?;
    write_json(&output.join("damage_40"), "result.json", &d40.artifact)?;
    let numerical_ok = fixed.steps_ok
        && dynamic.steps_ok
        && d10.artifact["steps_ok"].as_bool().unwrap_or(false)
        && d25.artifact["steps_ok"].as_bool().unwrap_or(false);

    // Gate 7 — resource dependence.
    let maintenance_falsified = !baseline_pass || !replacement_pass || !damage_pass;
    let (metabolic, repeated, resource_pass) = if maintenance_falsified {
        (
            MetabolicControlOutcome {
                pass: false,
                artifact: json!({"gate": 7, "pass": false, "skipped": true}),
            },
            json!({"gate": 7, "pass": false, "skipped": true}),
            false,
        )
    } else {
        let metabolic = run_metabolic_controls(horizon.min(80_000), rho_ctx);
        let repeated = run_repeated_damage(horizon.min(120_000), rho_ctx);
        let pass = metabolic.pass && repeated["pass"].as_bool().unwrap_or(false);
        (metabolic, repeated, pass)
    };
    write_json(
        &output.join("resource_controls"),
        "metabolic_controls.json",
        &metabolic.artifact,
    )?;
    write_json(
        &output.join("resource_controls"),
        "repeated_damage.json",
        &repeated,
    )?;

    // Gate 8 — foundational regression.
    let (foundational_pass, found_body) = if maintenance_falsified {
        (
            false,
            json!({"gate": 8, "pass": false, "skipped": true}),
        )
    } else {
        run_foundational_regression(rho_ctx)
    };
    write_json(
        &output.join("foundational_regression"),
        "result.json",
        &found_body,
    )?;

    // Gate 9 — autonomous R22.
    let (dynamic_pass, dyn_body) = if maintenance_falsified {
        (false, json!({"gate": 9, "pass": false, "skipped": true}))
    } else {
        run_dynamic_r22_gate(horizon, rho_ctx)
    };
    write_json(&output.join("dynamic_r22"), "result.json", &dyn_body)?;

    // Gate 10 — constrained Stage E contract (document only; not certified).
    let stage_e = revised_stage_e_membrane_contract();
    let stage_e_ok = !stage_e.note.is_empty();
    write_json(
        &output.join("stage_e_membrane_contract"),
        "revised_contract.json",
        &json!(stage_e),
    )?;

    let accounting_pass = replacement_pass
        && pulse_results
            .iter()
            .all(|p| p["pass"].as_bool().unwrap_or(false));
    let accounting = json!({
        "gate": "accounting",
        "material_closed": true,
        "transport_schema": transport_schema_name(&schema3_organism_params(rho_ctx)),
        "rho_a": rho_a,
        "accounting_pass": accounting_pass,
    });
    write_json(&output.join("accounting"), "result.json", &accounting)?;

    let conclusion = select_d041_conclusion(
        gate0_pass,
        gate1_pass,
        gate2_pass,
        gate3_pass,
        gate4_pass,
        baseline_pass,
        replacement_pass,
        damage_pass,
        resource_pass,
        foundational_pass,
        dynamic_pass,
        stage_e_ok,
        accounting_pass,
        numerical_ok,
    );

    let full_pass =
        conclusion == D041Conclusion::BasinAccessibleMembraneMaintenanceQualified;
    let route = if full_pass {
        D041_ARCHITECTURE_PASS
    } else {
        "none"
    };

    let manifest = json!({
        "project_directive": "D-041",
        "agent_memory_id": D041_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "starting_commit": D041_STARTING_COMMIT,
        "d040_tag": D041_D040_TAG,
        "max_accepted": horizon,
        "selected_rho_a": rho_a,
        "route_on_full_pass": D041_ARCHITECTURE_PASS,
        "route": route,
        "primary_conclusion": conclusion.as_str(),
        "stage_e_certified": false,
        "record": D041_RECORD_EXCHANGE,
        "artifacts": dirs,
    });
    write_json(&output, "manifest.json", &manifest)?;

    let result = json!({
        "primary_conclusion": conclusion.as_str(),
        "route": route,
        "selected_rho_a": rho_a,
        "stage_e_status": "not_certified",
        "gate0_route_confirmation": route_body,
        "gate1_transport_schema": transport,
        "gate2_retention_screen": gate2_body,
        "gate3_basin_map": basin_body,
        "gate4_membrane_necessity": necessity_body,
        "gate5_stable_baseline": {
            "pass": baseline_pass,
            "fixed": fixed.artifact,
            "dynamic": dynamic.artifact,
        },
        "gate5_pulse_chase": pulse_results,
        "gate6_damage": {
            "damage_10": d10.artifact,
            "damage_25": d25.artifact,
            "damage_40": d40.artifact,
        },
        "gate7_resource_controls": {
            "metabolic": metabolic.artifact,
            "repeated_damage": repeated,
        },
        "gate8_foundational": found_body,
        "gate9_dynamic_r22": dyn_body,
        "gate10_stage_e_contract": stage_e,
        "accounting": accounting,
        "preservation": preservation,
        "numerical_ok": numerical_ok,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    write_json(&output, "result.json", &result)?;
    Ok(result)
}
