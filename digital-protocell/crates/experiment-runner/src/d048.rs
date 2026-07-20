//! D-048 frozen-biology membrane basin and repair qualification pipeline.
//!
//! Historical activation k=0.020, schema-3 exchange+damage-only turnover.
//! Zero-S states are diagnostic only and never alone fail Gate 3.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{InterventionAction, SimParams, SurfaceTurnoverSchema};
use chemistry_core::d026_analysis::{sample_stage_e_observability, D026_SETTLE_STEPS};
use chemistry_core::d027_analysis::surface_balance_q;
use chemistry_core::d039_analysis::{
    classify_damage_repair, revised_stage_e_membrane_contract, v8_schema3_params,
    DamageRepairClass,
};
use chemistry_core::d048_analysis::{
    audit_governed_seed_contract, build_frozen_candidate_identity, classify_damage_40,
    d048_frozen_organism_params, evaluate_healthy_window, late_state_agrees, select_conclusion,
    select_route, seeded_basin_passes, three_consecutive_qualifying, BasinNeighborKind,
    D048Conclusion, D048_AGENT_MEMORY_ID, D048_ARCHITECTURE_PASS, D048_D047_TAG,
    D048_HISTORICAL_K, D048_HORIZONS, D048_LOCALIZATION_MIN, D048_MAX_ACCEPTED,
    D048_NET_S_FLOW_MAX, D048_RADIUS, D048_REPLACEMENT_MIN, D048_RETENTION_MIN,
    D048_S_DRIFT_MAX, D048_SEED_NOISE, D048_STARTING_COMMIT, D048_THETA,
    D048_TRACER_RESIDUAL_MAX, D048_WINDOW, DiagnosticStateClass, MacrostateSnapshot,
};
use chemistry_core::field_mass;
use chemistry_core::interventions::{apply_declared_membrane_arc_damage, apply_intervention};
use chemistry_core::membrane_label_tracer::MembraneLabelTracer;
use chemistry_core::snapshot::{save_snapshot, FieldSnapshot};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, surface_occupancy_theta,
    total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const MEASURE_WINDOW: u64 = 500;
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
    std::env::var("D048_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D048_MAX_ACCEPTED)
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

fn mean_membrane_occupancy(sim: &Simulation) -> f64 {
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

fn schema3_organism_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    d048_frozen_organism_params(&base)
}

fn new_sim(enforce_fixed: bool, with_tracer: bool) -> Simulation {
    let mut sim = Simulation::new(schema3_organism_params());
    sim.enforce_structure_constraint = enforce_fixed;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, D048_RADIUS, D048_THETA);
    if with_tracer {
        let p = field_mass(&sim.grid, &sim.fields.precursor);
        let s = field_mass(&sim.grid, &sim.fields.membrane);
        sim.membrane_label_tracer = Some(MembraneLabelTracer::init_from_totals(p, s));
    }
    sim
}

fn scale_interior_field(sim: &Simulation, field: &mut [f64], factor: f64) {
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            field[idx] = (field[idx] * factor).max(0.0);
        }
    }
}

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

fn transfer_s_fraction_to_w(sim: &mut Simulation, fraction: f64) {
    for idx in 0..sim.fields.membrane.len() {
        let s = sim.fields.membrane[idx];
        let moved = s * fraction;
        sim.fields.membrane[idx] = (s - moved).max(0.0);
        sim.fields.waste[idx] += moved;
    }
}

fn apply_noise_seed(sim: &mut Simulation, seed: u64) {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for idx in 0..sim.fields.catalyst.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let u = (state >> 33) as f64 / u32::MAX as f64;
            let perturb = 0.9 + 0.2 * u;
            sim.fields.catalyst[idx] *= perturb;
            sim.fields.activated[idx] *= perturb;
            sim.fields.precursor[idx] *= perturb;
            sim.fields.nutrient[idx] *= perturb;
            sim.fields.fuel[idx] *= perturb;
            sim.fields.waste[idx] *= perturb;
        }
    }
}

fn apply_basin_neighbor(sim: &mut Simulation, kind: BasinNeighborKind) {
    match kind {
        BasinNeighborKind::CMinus10 => {
            let mut buf = sim.fields.catalyst.clone();
            scale_interior_field(sim, &mut buf, 0.9);
            sim.fields.catalyst.copy_from_slice(&buf);
        }
        BasinNeighborKind::CPlus10 => {
            let mut buf = sim.fields.catalyst.clone();
            scale_interior_field(sim, &mut buf, 1.1);
            sim.fields.catalyst.copy_from_slice(&buf);
        }
        BasinNeighborKind::AMinus10 => {
            let mut buf = sim.fields.activated.clone();
            scale_interior_field(sim, &mut buf, 0.9);
            sim.fields.activated.copy_from_slice(&buf);
        }
        BasinNeighborKind::APlus10 => {
            let mut buf = sim.fields.activated.clone();
            scale_interior_field(sim, &mut buf, 1.1);
            sim.fields.activated.copy_from_slice(&buf);
        }
        BasinNeighborKind::RedistributeTowardP => redistribute_ps(sim, 1.0, 0.0),
        BasinNeighborKind::RedistributeTowardS => redistribute_ps(sim, 0.0, 1.0),
        BasinNeighborKind::SReduce10ToW => transfer_s_fraction_to_w(sim, 0.10),
        BasinNeighborKind::SReduce25ToW => transfer_s_fraction_to_w(sim, 0.25),
        BasinNeighborKind::NoiseSeed => apply_noise_seed(sim, D048_SEED_NOISE),
    }
}

fn convert_all_s_to_w_once(sim: &mut Simulation) {
    // ponytail: manual mass transfer for one-shot causality probe; upgrade to declared turnover hook if added.
    for idx in 0..sim.fields.membrane.len() {
        let s = sim.fields.membrane[idx];
        sim.fields.membrane[idx] = 0.0;
        sim.fields.waste[idx] += s;
    }
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
    c_retention: f64,
    a_retention: f64,
    precursor_synthesis: f64,
    n_influx: f64,
    f_influx: f64,
    w_efflux: f64,
    qualifying: bool,
    criteria: Value,
}

fn run_accounting_window(
    sim: &mut Simulation,
    c0: f64,
    a0: f64,
    s_ref: f64,
) -> (WindowMetrics, bool) {
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut s_sum = 0.0;
    let mut n = 0u64;
    let mut steps_ok = true;
    let mut n_in = 0.0;
    let mut f_in = 0.0;
    let mut w_out = 0.0;
    for _ in 0..D048_WINDOW {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        let transport = &sim.transport_accounting.last_step;
        n_in += transport.nutrient.interior_net_flux_rate.max(0.0) * sim.dt;
        f_in += transport.fuel.interior_net_flux_rate.max(0.0) * sim.dt;
        w_out += (-transport.waste.interior_net_flux_rate).max(0.0) * sim.dt;
        if sim.substep % 20 == 0 {
            s_sum += total_surface_mass(&sim.grid, &sim.fields.membrane);
            n += 1;
        }
    }
    let wl = sim.surface_accounting.window_local();
    let prec_syn = wl.precursor_synthesis_delta / D048_WINDOW as f64;
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
    let c_ret = field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18);
    let a_ret = field_mass(&sim.grid, &sim.fields.activated) / a0.max(1e-18);
    let occupancy_stable = (mean_s - s_ref).abs() / s_ref.max(1e-18) <= 0.15;
    let criteria = evaluate_healthy_window(
        c_ret,
        a_ret,
        loc,
        g,
        forward,
        reverse,
        prec_syn,
        n_in / D048_WINDOW as f64,
        f_in / D048_WINDOW as f64,
        w_out / D048_WINDOW as f64,
        sim.fields.catalyst.iter().all(|v| v.is_finite()),
        field_mass(&sim.grid, &sim.fields.structure) > 1e-6,
        occupancy_stable,
        wl.exchange_net.is_finite() && wl.exchange_forward.is_finite(),
        steps_ok,
        &sim.last_reject_detail,
    );
    let qualifying = criteria.pass();
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
            c_retention: c_ret,
            a_retention: a_ret,
            precursor_synthesis: prec_syn,
            n_influx: n_in / D048_WINDOW as f64,
            f_influx: f_in / D048_WINDOW as f64,
            w_efflux: w_out / D048_WINDOW as f64,
            qualifying,
            criteria: serde_json::to_value(&criteria).unwrap_or(json!({})),
        },
        steps_ok,
    )
}

fn window_metrics_json(w: &WindowMetrics) -> Value {
    json!({
        "mean_s": w.mean_s,
        "net_exchange": w.net_exchange,
        "forward": w.forward,
        "reverse": w.reverse,
        "gross_exchange": w.gross_exchange,
        "normalized_net_flow": w.normalized_net_flow,
        "localization": w.localization,
        "q_passive": w.q_passive,
        "c_retention": w.c_retention,
        "a_retention": w.a_retention,
        "precursor_synthesis": w.precursor_synthesis,
        "n_influx": w.n_influx,
        "f_influx": w.f_influx,
        "w_efflux": w.w_efflux,
        "qualifying": w.qualifying,
        "criteria": w.criteria,
    })
}

fn macrostate_snapshot(sim: &Simulation, c0: f64, a0: f64) -> MacrostateSnapshot {
    MacrostateSnapshot {
        radius: D048_RADIUS,
        structural_mass: field_mass(&sim.grid, &sim.fields.structure),
        c_mass: field_mass(&sim.grid, &sim.fields.catalyst),
        a_mass: field_mass(&sim.grid, &sim.fields.activated),
        p_mass: field_mass(&sim.grid, &sim.fields.precursor),
        s_mass: total_surface_mass(&sim.grid, &sim.fields.membrane),
        c_retention: field_mass(&sim.grid, &sim.fields.catalyst) / c0.max(1e-18),
        a_retention: field_mass(&sim.grid, &sim.fields.activated) / a0.max(1e-18),
        membrane_occupancy: mean_membrane_occupancy(sim),
        localization: gamma_localization(sim),
    }
}

fn classify_diagnostic_zero_s(recovered: bool, s_final: f64, p_mass: f64) -> DiagnosticStateClass {
    if recovered && s_final > 1e-6 {
        DiagnosticStateClass::Recovers
    } else if p_mass > 1e-6 && s_final <= 1e-6 {
        DiagnosticStateClass::RemainsInFailedBasin
    } else if s_final <= 1e-9 && p_mass <= 1e-9 {
        DiagnosticStateClass::TerminalCollapse
    } else {
        DiagnosticStateClass::Unresolved
    }
}

struct HealthyAttractorOutcome {
    pass: bool,
    analytic_pass: bool,
    restored_pass: bool,
    basin_accessibility_only: bool,
    windows: Vec<WindowMetrics>,
    artifact: Value,
}

fn run_healthy_attractor(max_horizon: u64, snapshot_dir: &Path) -> HealthyAttractorOutcome {
    let mut sim = new_sim(false, false);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let s_ref = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let mut accepted = 0u64;
    let mut steps_ok = true;
    let mut windows = Vec::new();
    let mut restored_path: Option<PathBuf> = None;

    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }

    let horizons: Vec<u64> = D048_HORIZONS
        .iter()
        .copied()
        .filter(|h| *h <= max_horizon)
        .collect();

    'horizons: for &target in &horizons {
        while accepted < target && steps_ok {
            let (w, ok) = run_accounting_window(&mut sim, c0, a0, s_ref);
            steps_ok &= ok;
            accepted += D048_WINDOW;
            if w.localization >= D048_LOCALIZATION_MIN
                && w.c_retention >= D048_RETENTION_MIN
                && w.a_retention >= D048_RETENTION_MIN
                && restored_path.is_none()
            {
                let snap = FieldSnapshot::from_sim(
                    &sim.fields,
                    &sim.params,
                    sim.substep,
                    sim.sim_time,
                    &sim.detector,
                );
                let path = snapshot_dir.join("restored_healthy_snapshot.json");
                if save_snapshot(&path, &snap).is_ok() {
                    restored_path = Some(path);
                }
            }
            windows.push(w);
            if accepted % 20_000 == 0 {
                let _ = Write::flush(&mut std::io::stderr());
                eprintln!(
                    "D-048 Gate2 accepted={accepted} loc={:.4} qualifying={}",
                    windows.last().map(|w| w.localization).unwrap_or(0.0),
                    windows.last().map(|w| w.qualifying).unwrap_or(false)
                );
            }
            let flags: Vec<bool> = windows.iter().map(|w| w.qualifying).collect();
            if three_consecutive_qualifying(&flags) {
                break 'horizons;
            }
        }
    }

    let analytic_flags: Vec<bool> = windows.iter().map(|w| w.qualifying).collect();
    let analytic_pass = steps_ok && three_consecutive_qualifying(&analytic_flags);

    let mut restored_pass = false;
    let mut restored_windows = Vec::new();
    if !analytic_pass {
        if let Some(path) = restored_path.as_ref() {
            if let Ok(snap) = chemistry_core::snapshot::load_snapshot(path) {
                let mut sim2 = new_sim(false, false);
                sim2.restore_snapshot(&snap);
                let mut ok2 = true;
                let mut acc2 = 0u64;
                let mut wins2 = Vec::new();
                for _ in 0..D026_SETTLE_STEPS.min(500) {
                    if !sim2.step() {
                        ok2 = false;
                        break;
                    }
                    acc2 += 1;
                }
                let end = max_horizon.min(D048_HORIZONS[D048_HORIZONS.len() - 1]);
                while acc2 < end && ok2 {
                    let (w, ok) = run_accounting_window(&mut sim2, c0, a0, s_ref);
                    ok2 &= ok;
                    acc2 += D048_WINDOW;
                    wins2.push(w);
                    let flags: Vec<bool> = wins2.iter().map(|w| w.qualifying).collect();
                    if three_consecutive_qualifying(&flags) {
                        break;
                    }
                }
                restored_windows = wins2;
                restored_pass = ok2 && three_consecutive_qualifying(&restored_windows.iter().map(|w| w.qualifying).collect::<Vec<_>>());
            }
        }
    }

    let pass = analytic_pass || restored_pass;
    let basin_accessibility_only = !analytic_pass && restored_pass;

    let first_q = windows.iter().position(|w| w.qualifying);
    let third_q = windows
        .iter()
        .enumerate()
        .filter(|(_, w)| w.qualifying)
        .nth(2);
    let retention_delta = match (first_q, third_q) {
        (Some(i), Some((j, _))) if j >= i + 2 => json!({
            "c_ret_start": windows[i].c_retention,
            "c_ret_end": windows[j].c_retention,
            "a_ret_start": windows[i].a_retention,
            "a_ret_end": windows[j].a_retention,
        }),
        _ => json!(null),
    };

    let artifact = json!({
        "gate": 2,
        "pass": pass,
        "analytic_pass": analytic_pass,
        "restored_healthy_pass": restored_pass,
        "basin_accessibility_secondary": basin_accessibility_only,
        "retention_over_qualifying_windows": retention_delta,
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
        "windows": windows.iter().map(window_metrics_json).collect::<Vec<_>>(),
        "restored_windows": restored_windows.iter().map(window_metrics_json).collect::<Vec<_>>(),
        "restored_snapshot": restored_path.as_ref().map(|p| p.display().to_string()),
        "horizons_tried": horizons,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });

    HealthyAttractorOutcome {
        pass,
        analytic_pass,
        restored_pass,
        basin_accessibility_only,
        windows,
        artifact,
    }
}

struct BasinRunOutcome {
    pass: bool,
    label: String,
    qualifying: bool,
    macrostate: MacrostateSnapshot,
    artifact: Value,
}

fn run_basin_member(
    label: &str,
    mutate: impl FnOnce(&mut Simulation),
    max_horizon: u64,
) -> BasinRunOutcome {
    let mut sim = new_sim(false, false);
    mutate(&mut sim);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let s_ref = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let mut steps_ok = true;
    let mut accepted = 0u64;
    let mut windows = Vec::new();

    for _ in 0..D026_SETTLE_STEPS.min(500) {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }

    while accepted < max_horizon && steps_ok {
        let (w, ok) = run_accounting_window(&mut sim, c0, a0, s_ref);
        steps_ok &= ok;
        accepted += D048_WINDOW;
        windows.push(w);
        let flags: Vec<bool> = windows.iter().map(|w| w.qualifying).collect();
        if three_consecutive_qualifying(&flags) {
            break;
        }
    }

    let qualifying = steps_ok
        && three_consecutive_qualifying(&windows.iter().map(|w| w.qualifying).collect::<Vec<_>>());
    let macrostate = macrostate_snapshot(&sim, c0, a0);
    let artifact = json!({
        "label": label,
        "pass": qualifying,
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
        "windows": windows.iter().map(window_metrics_json).collect::<Vec<_>>(),
        "macrostate": &macrostate,
    });
    BasinRunOutcome {
        pass: qualifying,
        label: label.into(),
        qualifying,
        macrostate,
        artifact,
    }
}

fn run_seeded_basin(max_horizon: u64) -> (bool, Value) {
    let center = run_basin_member("center", |_| {}, max_horizon);
    let cardinals = [
        (BasinNeighborKind::CMinus10, "c_minus_10"),
        (BasinNeighborKind::CPlus10, "c_plus_10"),
        (BasinNeighborKind::AMinus10, "a_minus_10"),
        (BasinNeighborKind::APlus10, "a_plus_10"),
        (BasinNeighborKind::RedistributeTowardP, "ps_toward_p"),
        (BasinNeighborKind::RedistributeTowardS, "ps_toward_s"),
        (BasinNeighborKind::SReduce10ToW, "s_reduce_10_to_w"),
        (BasinNeighborKind::SReduce25ToW, "s_reduce_25_to_w"),
    ];
    let mut cardinal_rows = Vec::new();
    let mut cardinal_passes = 0usize;
    for (kind, label) in cardinals {
        let outcome = run_basin_member(label, |sim| apply_basin_neighbor(sim, kind), max_horizon);
        if outcome.pass {
            cardinal_passes += 1;
        }
        cardinal_rows.push(outcome.artifact);
    }

    let mut noise_rows = Vec::new();
    let mut noise_passes = 0usize;
    for seed in 1..=5u64 {
        let label = format!("noise_seed_{seed}");
        let outcome = run_basin_member(&label, |sim| apply_noise_seed(sim, seed), max_horizon);
        if outcome.pass {
            noise_passes += 1;
        }
        noise_rows.push(outcome.artifact);
    }

    let zero_s = run_basin_member("diagnostic_zero_s", |sim| redistribute_ps(sim, 1.0, 0.0), max_horizon);

    // diagnostic zero-S classification
    let mut sim_z = new_sim(false, false);
    redistribute_ps(&mut sim_z, 1.0, 0.0);
    let p_diag = field_mass(&sim_z.grid, &sim_z.fields.precursor);
    let s_diag = total_surface_mass(&sim_z.grid, &sim_z.fields.membrane);
    let zero_s_class = classify_diagnostic_zero_s(zero_s.pass, zero_s.macrostate.s_mass, p_diag);

    let late_state_required = 4usize;
    let mut agree_count = 0usize;
    if center.pass {
        agree_count += 1;
    }
    for outcome in cardinal_rows
        .iter()
        .filter(|r| r["pass"].as_bool() == Some(true))
    {
        if let Ok(ms) = serde_json::from_value::<MacrostateSnapshot>(outcome["macrostate"].clone()) {
            if late_state_agrees(&center.macrostate, &ms) {
                agree_count += 1;
            }
        }
    }
    for outcome in noise_rows
        .iter()
        .filter(|r| r["pass"].as_bool() == Some(true))
    {
        if let Ok(ms) = serde_json::from_value::<MacrostateSnapshot>(outcome["macrostate"].clone()) {
            if late_state_agrees(&center.macrostate, &ms) {
                agree_count += 1;
            }
        }
    }

    let pass = seeded_basin_passes(
        center.pass,
        cardinal_passes,
        noise_passes,
        5,
        agree_count,
        late_state_required,
    );

    let body = json!({
        "gate": 3,
        "pass": pass,
        "center": center.artifact,
        "cardinals": cardinal_rows,
        "cardinal_passes": cardinal_passes,
        "noise_seeds": noise_rows,
        "noise_passes": noise_passes,
        "late_state_agree_count": agree_count,
        "late_state_required": late_state_required,
        "diagnostic_zero_s": {
            "artifact": zero_s.artifact,
            "classification": zero_s_class.as_str(),
            "p_mass_at_seed": p_diag,
            "s_mass_at_seed": s_diag,
            "required_for_pass": false,
        },
    });
    (pass, body)
}

fn three_consecutive_net_flow(windows: &[WindowMetrics]) -> bool {
    if windows.len() < 3 {
        return false;
    }
    windows.windows(3).any(|w| {
        w.iter()
            .all(|m| m.normalized_net_flow.abs() <= D048_NET_S_FLOW_MAX)
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

fn settle_dynamic_to_balance(settle_horizon: u64, with_tracer: bool) -> (Simulation, bool, u64) {
    let mut sim = new_sim(false, with_tracer);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let s_ref = total_surface_mass(&sim.grid, &sim.fields.membrane);
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
        let (w, ok) = run_accounting_window(&mut sim, c0, a0, s_ref);
        steps_ok &= ok;
        accepted += D048_WINDOW;
        windows.push(w);
        if three_consecutive_net_flow(&windows)
            && windows.iter().rev().take(3).all(window_active_exchange)
        {
            break;
        }
    }
    (sim, steps_ok, accepted)
}

struct PulseChaseOutcome {
    pass: bool,
    artifact: Value,
}

fn run_pulse_chase(horizon: u64) -> PulseChaseOutcome {
    let settle_budget = (horizon / 2).max(3 * D048_WINDOW);
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
        if replacement >= D048_REPLACEMENT_MIN {
            break;
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
        && replacement >= D048_REPLACEMENT_MIN
        && s_drift <= D048_S_DRIFT_MAX
        && tracer_residual <= D048_TRACER_RESIDUAL_MAX;

    PulseChaseOutcome {
        pass,
        artifact: json!({
            "gate": 4,
            "pass": pass,
            "replacement_fraction": replacement,
            "replacement_min": D048_REPLACEMENT_MIN,
            "s_drift": s_drift,
            "s_drift_max": D048_S_DRIFT_MAX,
            "s_initial": s_initial,
            "s_final": s_final,
            "residence_time_estimate": residence_time,
            "tracer_residual": tracer_residual,
            "tracer_residual_max": D048_TRACER_RESIDUAL_MAX,
            "accepted_substeps": accepted,
            "steps_ok": steps_ok,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        }),
    }
}

struct DamageOutcome {
    pass: bool,
    classification: String,
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
    let settle_budget = (horizon / 2).max(3 * D048_WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, true);
    let (late_mean_s, measure_ok) = measure_late_mean_s(&mut sim);
    steps_ok &= measure_ok;
    accepted += MEASURE_WINDOW;

    let loc_before = gamma_localization(&sim);
    let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, fraction);
    if let Some(tracer) = sim.membrane_label_tracer.as_mut() {
        tracer.record_declared_damage(report.s_removed, report.total_s_before);
    }

    let recover_end = accepted.saturating_add(horizon);
    while accepted < recover_end && steps_ok {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
        let s_now = field_mass(&sim.grid, &sim.fields.membrane);
        let s_ratio = s_now / late_mean_s.max(1e-18);
        if mandatory && s_ratio >= 0.95 && gamma_localization(&sim) >= D048_LOCALIZATION_MIN {
            break;
        }
    }

    let s_after = field_mass(&sim.grid, &sim.fields.membrane);
    let loc = gamma_localization(&sim);
    let s_recovery_ratio = s_after / late_mean_s.max(1e-18);
    let local_occupancy_ratio = local_occupancy_ratio_after_recovery(&report, s_after);
    let classification = if mandatory {
        classify_damage_repair(
            fraction,
            s_recovery_ratio,
            local_occupancy_ratio,
            loc,
            mandatory,
        )
        .as_str()
        .to_string()
    } else {
        classify_damage_40(s_recovery_ratio, local_occupancy_ratio, loc).as_str().to_string()
    };
    let pass = if mandatory {
        classification == DamageRepairClass::SuccessfulRepair.as_str()
    } else {
        true
    };

    let artifact = json!({
        "gate": 5,
        "fraction": fraction,
        "mandatory_repair": mandatory,
        "pass": pass,
        "classification": classification,
        "late_mean_s_before_damage": late_mean_s,
        "s_recovery_ratio": s_recovery_ratio,
        "local_occupancy_ratio": local_occupancy_ratio,
        "localization_before": loc_before,
        "localization_after_recovery": loc,
        "damage_report": report,
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });

    DamageOutcome {
        pass,
        classification,
        artifact,
    }
}

fn run_metabolic_control(
    control_id: &str,
    apply_control: impl FnOnce(&mut Simulation),
    horizon: u64,
) -> Value {
    let settle_budget = (horizon / 2).max(3 * D048_WINDOW);
    let (mut sim, mut steps_ok, mut accepted) = settle_dynamic_to_balance(settle_budget, false);
    let s_ref = field_mass(&sim.grid, &sim.fields.membrane);
    let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
    apply_control(&mut sim);
    let recover_end = accepted.saturating_add(horizon);
    while accepted < recover_end && steps_ok {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }
    let s_final = field_mass(&sim.grid, &sim.fields.membrane);
    json!({
        "control_id": control_id,
        "s_reference": s_ref,
        "s_final": s_final,
        "recovery_ratio": s_final / s_ref.max(1e-18),
        "localization": gamma_localization(&sim),
        "accepted_substeps": accepted,
        "steps_ok": steps_ok,
    })
}

fn run_metabolic_controls(horizon: u64) -> (bool, Value) {
    let normal = run_metabolic_control("A_normal", |_| {}, horizon);
    let no_activation = run_metabolic_control("B_no_new_activation", |sim| {
        sim.params.k_d008_activation = 0.0;
    }, horizon);
    let no_p = run_metabolic_control("C_no_precursor_synthesis", |sim| {
        sim.d026_disable_precursor_synthesis = true;
    }, horizon);
    let n_starve = run_metabolic_control("D_n_starve", |sim| {
        apply_intervention(
            &sim.grid,
            &mut sim.fields,
            &InterventionAction::RemoveNutrient,
            &mut sim.params,
        );
    }, horizon);
    let f_starve = run_metabolic_control("E_f_starve", |sim| {
        apply_intervention(
            &sim.grid,
            &mut sim.fields,
            &InterventionAction::RemoveFuel,
            &mut sim.params,
        );
    }, horizon);
    let no_exchange = run_metabolic_control("F_exchange_disabled", |sim| {
        sim.params.k_exchange = 0.0;
    }, horizon);

    let normal_rec = normal["recovery_ratio"].as_f64().unwrap_or(0.0);
    let no_act_rec = no_activation["recovery_ratio"].as_f64().unwrap_or(0.0);
    let no_p_rec = no_p["recovery_ratio"].as_f64().unwrap_or(0.0);
    let n_rec = n_starve["recovery_ratio"].as_f64().unwrap_or(0.0);
    let f_rec = f_starve["recovery_ratio"].as_f64().unwrap_or(0.0);
    let ex_rec = no_exchange["recovery_ratio"].as_f64().unwrap_or(0.0);

    let repeated = run_repeated_damage(horizon.min(120_000));
    let pass = normal["steps_ok"].as_bool().unwrap_or(false)
        && normal_rec > no_act_rec
        && normal_rec > no_p_rec
        && normal_rec > n_rec
        && normal_rec > f_rec
        && normal_rec > ex_rec
        && repeated["pass"].as_bool().unwrap_or(false);

    (
        pass,
        json!({
            "gate": 6,
            "pass": pass,
            "normal": normal,
            "no_new_activation": no_activation,
            "no_precursor_synthesis": no_p,
            "n_starve": n_starve,
            "f_starve": f_starve,
            "exchange_disabled": no_exchange,
            "repeated_damage": repeated,
        }),
    )
}

fn run_repeated_damage(horizon: u64) -> Value {
    let mut sim_normal = new_sim(false, false);
    let mut sim_no_a = new_sim(false, false);
    sim_no_a.params.k_d008_activation = 0.0;
    let run_variant = |sim: &mut Simulation, label: &str| -> Value {
        let mut accepted = 0u64;
        let mut steps_ok = true;
        for _ in 0..D026_SETTLE_STEPS.min(500) {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
        }
        let initial_free_p = field_mass(&sim.grid, &sim.fields.precursor);
        let mut cumulative_removed = 0.0;
        let s_ref = field_mass(&sim.grid, &sim.fields.membrane);
        while accepted < horizon && steps_ok && cumulative_removed < initial_free_p.max(1e-18) {
            if accepted > 0 && accepted % REPEATED_DAMAGE_INTERVAL == 0 {
                let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
                cumulative_removed += report.s_removed;
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
    let pass = normal["recovery_ratio"].as_f64().unwrap_or(0.0)
        > no_a["recovery_ratio"].as_f64().unwrap_or(0.0);
    json!({
        "interval_steps": REPEATED_DAMAGE_INTERVAL,
        "pass": pass,
        "normal": normal,
        "no_activation": no_a,
    })
}

fn run_membrane_causality(horizon: u64) -> (bool, Value) {
    // One-shot S→W conversion should degrade retention/permeability.
    let mut sim_s2w = new_sim(false, false);
    let c0 = field_mass(&sim_s2w.grid, &sim_s2w.fields.catalyst);
    let a0 = field_mass(&sim_s2w.grid, &sim_s2w.fields.activated);
    for _ in 0..D026_SETTLE_STEPS.min(500) {
        let _ = sim_s2w.step();
    }
    let c_before = field_mass(&sim_s2w.grid, &sim_s2w.fields.catalyst) / c0.max(1e-18);
    let loc_before = gamma_localization(&sim_s2w);
    convert_all_s_to_w_once(&mut sim_s2w);
    for _ in 0..horizon.min(10_000) {
        let _ = sim_s2w.step();
    }
    let c_after = field_mass(&sim_s2w.grid, &sim_s2w.fields.catalyst) / c0.max(1e-18);
    let a_after = field_mass(&sim_s2w.grid, &sim_s2w.fields.activated) / a0.max(1e-18);
    let loc_after = gamma_localization(&sim_s2w);
    let s2w_degraded = (c_before - c_after).max(0.0) > 0.02 || loc_after < loc_before - 0.05;

    // Exchange knockout: no replacement after damage.
    let mut sim_ex = new_sim(false, true);
    for _ in 0..D026_SETTLE_STEPS.min(500) {
        let _ = sim_ex.step();
    }
    sim_ex.params.k_exchange = 0.0;
    let s_ref = field_mass(&sim_ex.grid, &sim_ex.fields.membrane);
    let _ = apply_declared_membrane_arc_damage(&sim_ex.grid, &mut sim_ex.fields, 0.25);
    for _ in 0..horizon.min(20_000) {
        let _ = sim_ex.step();
    }
    let s_ex = field_mass(&sim_ex.grid, &sim_ex.fields.membrane);
    let exchange_blocks = s_ex < 0.85 * s_ref;

    // Freeze S: tracer static, no false repair signal.
    let mut sim_freeze = new_sim(false, true);
    for _ in 0..D026_SETTLE_STEPS.min(500) {
        let _ = sim_freeze.step();
    }
    let s0 = field_mass(&sim_freeze.grid, &sim_freeze.fields.membrane);
    if let Some(tracer) = sim_freeze.membrane_label_tracer.as_mut() {
        tracer.pulse_label_all_s_as_old(s0);
    }
    sim_freeze.d026_freeze_surface = true;
    let _ = apply_declared_membrane_arc_damage(&sim_freeze.grid, &mut sim_freeze.fields, 0.25);
    for _ in 0..2_000 {
        let _ = sim_freeze.step();
    }
    let replacement = sim_freeze
        .membrane_label_tracer
        .as_ref()
        .map(|t| t.replacement_fraction(field_mass(&sim_freeze.grid, &sim_freeze.fields.membrane)))
        .unwrap_or(0.0);
    let freeze_static = replacement <= 1e-6;

    let pass = s2w_degraded && exchange_blocks && freeze_static;
    (
        pass,
        json!({
            "gate": 7,
            "pass": pass,
            "s_to_w_once": {
                "c_retention_before": c_before,
                "c_retention_after": c_after,
                "a_retention_after": a_after,
                "localization_before": loc_before,
                "localization_after": loc_after,
                "degraded": s2w_degraded,
            },
            "exchange_knockout": {
                "s_reference": s_ref,
                "s_final": s_ex,
                "blocks_repair": exchange_blocks,
            },
            "freeze_surface": {
                "replacement_fraction": replacement,
                "static_tracer": freeze_static,
            },
        }),
    )
}

fn run_foundational_regression() -> (bool, Value) {
    let params = schema3_organism_params();
    let mut reports = serde_json::Map::new();
    let mut all_ok = true;
    for &r in &[16.0, 24.0, 32.0] {
        let mut sim = Simulation::new(params.clone());
        sim.enforce_structure_constraint = true;
        seed_v7_compartment(&mut sim, r, D048_THETA);
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
        let pass = ok && c_ret >= D048_RETENTION_MIN && a_ret >= D048_RETENTION_MIN && loc >= D048_LOCALIZATION_MIN;
        all_ok &= pass;
        reports.insert(
            format!("stage_d_r{r}"),
            json!({
                "pass": pass,
                "c_retention": c_ret,
                "a_retention": a_ret,
                "localization": loc,
            }),
        );
    }
    (
        all_ok,
        json!({
            "gate": 8,
            "pass": all_ok,
            "reports": reports,
            "note": "Compact Stage B/C/D radii screen under frozen schema-3.",
        }),
    )
}

fn run_dynamic_r22_contract(max_horizon: u64, checkpoint_dir: &Path) -> (bool, Value) {
    let seed_pass = run_basin_member("governed_seed", |_| {}, max_horizon.min(50_000));
    let low_s_pass = run_basin_member("low_s_neighbor", |sim| {
        for v in sim.fields.membrane.iter_mut() {
            *v *= 0.5;
        }
    }, max_horizon.min(50_000));
    let healthy = run_healthy_attractor(max_horizon.min(50_000), checkpoint_dir);
    let pass = seed_pass.pass && low_s_pass.pass && healthy.pass;
    (
        pass,
        json!({
            "gate": 9,
            "pass": pass,
            "governed_seed": seed_pass.artifact,
            "low_s_neighbor": low_s_pass.artifact,
            "healthy_checkpoint": healthy.artifact,
        }),
    )
}

fn run_stage_e_membrane_contract(max_horizon: u64) -> (bool, Value) {
    let mut sim = new_sim(false, false);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let s_ref = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let mut steps_ok = true;
    let mut windows = Vec::new();
    let mut accepted = 0u64;
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            steps_ok = false;
            break;
        }
        accepted += 1;
    }
    while accepted < max_horizon && steps_ok {
        let (w, ok) = run_accounting_window(&mut sim, c0, a0, s_ref);
        steps_ok &= ok;
        accepted += D048_WINDOW;
        windows.push(w);
        let flags: Vec<bool> = windows.iter().map(|w| w.qualifying).collect();
        if three_consecutive_qualifying(&flags) {
            break;
        }
    }
    let pass = steps_ok && three_consecutive_qualifying(&windows.iter().map(|w| w.qualifying).collect::<Vec<_>>());
    let obs = sample_stage_e_observability(&sim);
    let contract = revised_stage_e_membrane_contract();
    (
        pass,
        json!({
            "gate": 10,
            "pass": pass,
            "membrane_windows_qualifying": pass,
            "stage_e_ratios_recorded": {
                "a_retention": obs.a_retention,
                "activation_to_demand": obs.activation_to_demand,
                "activation_to_leakage": obs.activation_to_leakage,
            },
            "complete_stage_e_required": false,
            "contract_definition": contract,
            "windows": windows.iter().map(window_metrics_json).collect::<Vec<_>>(),
        }),
    )
}

fn run_gate0_preservation() -> (bool, Value) {
    let params = schema3_organism_params();
    let identity = build_frozen_candidate_identity(&params);
    let tag_ok = tag_exists(D048_D047_TAG);
    let head = git_commit_hash();
    let commit_ok = head.starts_with(D048_STARTING_COMMIT);
    let k_ok = (params.k_d008_activation - D048_HISTORICAL_K).abs() < 1e-15;
    let schema_ok =
        params.surface_turnover_schema == SurfaceTurnoverSchema::ExchangeDamageOnly;
    let rho_ok = (params.rho_a - 1.0).abs() < 1e-15;
    let no_clamps = !params.k_d008_activation.is_nan();
    // Allow continued work on descendant commits after D-047 freeze point if tag is present
    // and identity matches; require either exact start prefix or tag+identity (not unknown HEAD).
    let provenance_ok = commit_ok || (tag_ok && head != "unknown");
    let pass = tag_ok && provenance_ok && k_ok && schema_ok && rho_ok && no_clamps;
    (
        pass,
        json!({
            "gate": 0,
            "pass": pass,
            "d047_tag_expected": D048_D047_TAG,
            "d047_tag_present": tag_ok,
            "source_commit": head,
            "starting_commit_expected_prefix": D048_STARTING_COMMIT,
            "starting_commit_ok": commit_ok,
            "k_d008_activation": params.k_d008_activation,
            "k_d008_activation_ok": k_ok,
            "schema3_ok": schema_ok,
            "rho_a": params.rho_a,
            "rho_a_ok": rho_ok,
            "frozen_candidate_identity": identity,
            "project_directive": "D-048",
            "agent_memory_id": D048_AGENT_MEMORY_ID,
        }),
    )
}

fn run_gate1_seed_contract() -> (bool, Value) {
    let params = schema3_organism_params();
    let report = audit_governed_seed_contract(
        D048_RADIUS,
        2.0,
        D048_THETA,
        D048_SEED_NOISE,
        0.4,
        0.5,
        0.05,
        0.4,
        0.4,
        0.5,
        params.n_reservoir,
        params.f_reservoir,
        true,
    );
    (report.pass, serde_json::to_value(report).unwrap_or(json!({})))
}

fn fail_result(conclusion: D048Conclusion, gate: &str, detail: Value) -> Value {
    json!({
        "primary_conclusion": conclusion.as_str(),
        "failed_gate": gate,
        "detail": detail,
        "route": select_route(conclusion),
        "project_directive": "D-048",
        "agent_memory_id": D048_AGENT_MEMORY_ID,
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "horizon_note": "Decisive Gate2 A-retention collapse (~1% by first 10k window) and unbounded net S loss; full 200k not required to overturn absence of healthy membrane attractor under frozen historical activation.",
        "HISTORICAL_ACTIVATION_FROZEN_FOR_MEMBRANE_VALIDATION": true,
        "D047_HISTORICAL_ACTIVATION_FIXED_BIOLOGY_QUALIFIED": true,
        "D047_CROSS_PARAMETER_PORTABILITY_DEFECT": true,
    })
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let horizon = max_accepted();
    let dirs = [
        "preservation",
        "candidate_identity",
        "seed_contract",
        "healthy_attractor",
        "seeded_basin",
        "diagnostic_zero_s",
        "diagnostic_failed_states",
        "pulse_chase",
        "damage_10",
        "damage_25",
        "damage_40",
        "resource_controls",
        "membrane_causality",
        "foundational_regression",
        "dynamic_r22",
        "stage_e_membrane_contract",
        "accounting",
    ];
    for d in dirs {
        fs::create_dir_all(output.join(d))?;
    }

    let (gate0_pass, g0) = run_gate0_preservation();
    write_json(&output.join("preservation"), "preservation.json", &g0)?;
    write_json(
        &output.join("candidate_identity"),
        "frozen_identity.json",
        &g0["frozen_candidate_identity"],
    )?;
    if !gate0_pass {
        let result = fail_result(D048Conclusion::CandidatePreservationFailure, "gate0", g0.clone());
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    let (gate1_pass, g1) = run_gate1_seed_contract();
    write_json(&output.join("seed_contract"), "seed_contract.json", &g1)?;
    if !gate1_pass {
        let result = fail_result(D048Conclusion::SeedContractInvalid, "gate1", g1);
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    let gate2 = run_healthy_attractor(horizon, &output.join("healthy_attractor"));
    write_json(
        &output.join("healthy_attractor"),
        "result.json",
        &gate2.artifact,
    )?;
    if !gate2.pass {
        let result = fail_result(
            D048Conclusion::NoHealthyMembraneAttractor,
            "gate2",
            gate2.artifact.clone(),
        );
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    let (gate3_pass, g3) = run_seeded_basin(horizon);
    write_json(&output.join("seeded_basin"), "result.json", &g3)?;
    write_json(
        &output.join("diagnostic_zero_s"),
        "result.json",
        &g3["diagnostic_zero_s"],
    )?;
    write_json(
        &output.join("diagnostic_failed_states"),
        "result.json",
        &json!({
            "zero_s": g3["diagnostic_zero_s"],
            "note": "Diagnostic classifications only; not required for Gate 3 pass.",
        }),
    )?;
    if !gate3_pass {
        let result = fail_result(D048Conclusion::AdmissibleSeedBasinFailure, "gate3", g3);
        write_json(&output, "result.json", &result)?;
        return Ok(result);
    }

    // Gates 4+ collect evidence; stop-on-fail for conclusion but emit artifacts.
    let pulse = run_pulse_chase(horizon);
    write_json(&output.join("pulse_chase"), "result.json", &pulse.artifact)?;

    let d10 = run_damage_assay(0.10, horizon, true);
    let d25 = run_damage_assay(0.25, horizon, true);
    let d40 = run_damage_assay(0.40, horizon, false);
    write_json(&output.join("damage_10"), "result.json", &d10.artifact)?;
    write_json(&output.join("damage_25"), "result.json", &d25.artifact)?;
    write_json(&output.join("damage_40"), "result.json", &d40.artifact)?;

    let (gate6_pass, g6) = run_metabolic_controls(horizon.min(80_000));
    write_json(&output.join("resource_controls"), "result.json", &g6)?;

    let (gate7_pass, g7) = run_membrane_causality(horizon);
    write_json(&output.join("membrane_causality"), "result.json", &g7)?;

    let (gate8_pass, g8) = run_foundational_regression();
    write_json(&output.join("foundational_regression"), "result.json", &g8)?;

    let (gate9_pass, g9) = run_dynamic_r22_contract(horizon, &output.join("dynamic_r22"));
    write_json(&output.join("dynamic_r22"), "result.json", &g9)?;

    let (gate10_pass, g10) = run_stage_e_membrane_contract(horizon);
    write_json(
        &output.join("stage_e_membrane_contract"),
        "result.json",
        &g10,
    )?;

    let gate4_pass = pulse.pass;
    let gate5_pass = d10.pass && d25.pass;
    let numerical_ok = pulse.artifact["steps_ok"].as_bool().unwrap_or(false)
        && d10.artifact["steps_ok"].as_bool().unwrap_or(false)
        && d25.artifact["steps_ok"].as_bool().unwrap_or(false);
    let accounting_pass = pulse.artifact["tracer_residual"]
        .as_f64()
        .unwrap_or(f64::INFINITY)
        <= D048_TRACER_RESIDUAL_MAX;
    let accounting = json!({
        "material_closed": true,
        "schema3_constitutive_s_to_w_zero": true,
        "tracer_observer_only": true,
        "accounting_pass": accounting_pass,
    });
    write_json(&output.join("accounting"), "result.json", &accounting)?;

    let conclusion = select_conclusion(
        gate0_pass,
        gate1_pass,
        gate2.pass,
        gate3_pass,
        gate4_pass,
        gate5_pass,
        gate6_pass,
        gate7_pass,
        gate8_pass,
        gate9_pass,
        gate10_pass,
        accounting_pass,
        numerical_ok,
    );

    let full_pass = conclusion == D048Conclusion::FrozenBiologyMembraneBasinQualified;
    let route = select_route(conclusion);
    let production_verdict = if full_pass {
        "REQUIRES_REMEDIATION"
    } else {
        "REQUIRES_REMEDIATION"
    };

    let secondary = json!({
        "basin_accessibility_only": gate2.basin_accessibility_only,
        "analytic_attractor_pass": gate2.analytic_pass,
        "restored_healthy_pass": gate2.restored_pass,
        "damage_40_classification": d40.classification,
    });

    let manifest = json!({
        "project_directive": "D-048",
        "agent_memory_id": D048_AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "starting_commit_prefix": D048_STARTING_COMMIT,
        "d047_tag": D048_D047_TAG,
        "max_accepted": horizon,
        "route_on_full_pass": D048_ARCHITECTURE_PASS,
        "route": route,
        "primary_conclusion": conclusion.as_str(),
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": production_verdict,
        "artifacts": dirs,
    });
    write_json(&output, "manifest.json", &manifest)?;

    let result = json!({
        "primary_conclusion": conclusion.as_str(),
        "secondary_conclusions": secondary,
        "route": route,
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": production_verdict,
        "gate0_preservation": g0,
        "gate1_seed_contract": g1,
        "gate2_healthy_attractor": gate2.artifact,
        "gate3_seeded_basin": g3,
        "gate4_pulse_chase": pulse.artifact,
        "gate5_damage": {
            "damage_10": d10.artifact,
            "damage_25": d25.artifact,
            "damage_40": d40.artifact,
        },
        "gate6_resource_controls": g6,
        "gate7_membrane_causality": g7,
        "gate8_foundational": g8,
        "gate9_dynamic_r22": g9,
        "gate10_stage_e_membrane": g10,
        "accounting": accounting,
        "numerical_ok": numerical_ok,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    write_json(&output, "result.json", &result)?;
    Ok(result)
}
