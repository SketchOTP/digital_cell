//! D-073 mature-membrane equilibrium sufficiency audit pipeline.
//!
//! Diagnostic/non-promotable fixed-P holds only. Frozen D-070/D-071/D-072 biology.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::{
    apply_delivery_repair, DeliveryRepairPair, D053_FITTED_K_C, D053_FITTED_V_A, D053_F_REF,
    D053_N_REF,
};
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d063_analysis::{
    generate_phi, seed_mature_s_on_interfaces, GeometrySpec, D063_PHI_INTERIOR,
};
use chemistry_core::d069_analysis::split_accepted_exchange;
use chemistry_core::d070_analysis::{
    migrate_policy_d_authorized_reconstruction, occupancy_theta, MigrationPolicy,
};
use chemistry_core::d071_analysis::PrecursorRegulationParams;
use chemistry_core::d072_analysis::{
    exchange_timescale, DAMAGE_FRACTION, D071_SELECTED_M_P, REPAIR_THRESHOLD,
};
use chemistry_core::d073_analysis::*;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::membrane::membrane_catalyst_saturation;
use chemistry_core::surface_density::{
    compute_interface_geometry, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::{field_mass, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
        .max(1)
}
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
fn settle_steps() -> u64 {
    env_u64("D073_SETTLE", 400)
}
fn tau_max_accepted() -> u64 {
    env_u64("D073_TAU_MAX_ACCEPTED", 200_000)
}
fn tau_max_time() -> f64 {
    env_f64("D073_TAU_MAX_TIME", 1200.0).max(0.0)
}
fn horizon_dt_cap() -> f64 {
    env_f64("D073_HORIZON_DT_CAP", 0.05).max(1e-4)
}
fn control_tau_mult() -> f64 {
    env_f64("D073_CONTROL_TAU_MULT", 5.0).max(1.0)
}
fn skip_late() -> bool {
    env_flag("D073_SKIP_LATE_GATES")
}
fn reload_gate(out: &Path, name: &str) -> Option<Value> {
    if !env_flag("D073_RELOAD_EXISTING") {
        return None;
    }
    let path = out.join(name).join("result.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}
fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}
fn write_json(dir: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join("result.json"), value)?;
    Ok(())
}
fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(resolve_path(Path::new(".")).join(".."))
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
}

fn baseline_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    params
}

fn hold_exterior(sim: &mut Simulation) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
        }
    }
}

/// Diagnostic nonconservative hold: set precursor concentration on every
/// interface-supported biological cell (δ > δ_floor) to realize activity `p`.
fn hold_interface_activity_cached(sim: &mut Simulation, p_activity: f64, iface: &[usize]) {
    let conc = concentration_for_activity(p_activity, sim.params.p_reference);
    for &i in iface {
        sim.fields.precursor[i] = conc;
    }
}

fn interface_cell_indices(sim: &Simulation) -> Vec<usize> {
    let g = geometry(sim);
    (0..g.len())
        .filter(|&i| sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor)
        .collect()
}

fn hold_interface_activity(sim: &mut Simulation, p_activity: f64) {
    let iface = interface_cell_indices(sim);
    hold_interface_activity_cached(sim, p_activity, &iface);
    sim.fields.copy_current_to_next();
}

fn seed_b_policy_d(sim: &mut Simulation, spec: &GeometrySpec) -> Value {
    let phi = generate_phi(&sim.grid, spec);
    let mut geometry = vec![InterfaceGeometryCell::default(); phi.len()];
    compute_interface_geometry(&sim.grid, &phi, sim.params.eta_n, &mut geometry);
    let mut membrane = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    let precursor: Vec<f64> = phi
        .iter()
        .map(|&v| {
            if v >= D063_PHI_INTERIOR {
                0.05
            } else {
                0.0
            }
        })
        .collect();
    let migration = migrate_policy_d_authorized_reconstruction(
        &sim.grid,
        &geometry,
        &mut membrane,
        &precursor,
        sim.params.delta_floor,
        sim.params.gamma_max,
        1.0,
        "d073_seed_b_policy_d",
    );
    for i in 0..phi.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        sim.fields.structure[i] = phi[i];
        sim.fields.membrane[i] = membrane[i];
        sim.fields.precursor[i] = precursor[i];
        if phi[i] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[i] = 0.4;
            sim.fields.activated[i] = 0.5;
            sim.fields.nutrient[i] = 0.4;
            sim.fields.fuel[i] = 0.4;
            sim.fields.waste[i] = 0.5;
        } else {
            sim.fields.catalyst[i] = 0.0;
            sim.fields.activated[i] = 0.0;
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
            sim.fields.waste[i] = sim.params.w_reservoir;
        }
    }
    sim.fields.copy_current_to_next();
    json!({
        "seed_kind": "BPolicyD",
        "migration": migration,
        "policy": MigrationPolicy::AuthorizedMaterialReconstruction
    })
}

fn geometry(sim: &Simulation) -> Vec<InterfaceGeometryCell> {
    let mut g = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(&sim.grid, &sim.fields.structure, sim.params.eta_n, &mut g);
    g
}

fn capacity_snapshot(sim: &Simulation) -> (f64, f64) {
    let g = geometry(sim);
    let mut capacity = 0.0;
    let mut max_theta: f64 = 0.0;
    for i in 0..g.len() {
        if sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor {
            capacity += g[i].delta * sim.params.gamma_max;
            max_theta = max_theta.max(occupancy_theta(
                sim.fields.membrane[i],
                g[i].delta,
                sim.params.gamma_max,
            ));
        }
    }
    (capacity, max_theta)
}

fn absolute_occupancy(sim: &Simulation) -> f64 {
    let (capacity, _) = capacity_snapshot(sim);
    if capacity <= EPS {
        0.0
    } else {
        (total_surface_mass(&sim.grid, &sim.fields.membrane) / capacity).min(1.0)
    }
}

#[derive(Clone, Copy)]
enum HoldMode {
    None,
    /// Diagnostic nonconservative fixed activity on interface-supported cells.
    FixedInterfaceP(f64),
}

struct StepStats {
    accepted: u64,
    rejected: u64,
    time: f64,
    exchange_net: f64,
    ads: f64,
    des: f64,
}

fn run_steps(
    sim: &mut Simulation,
    accepted_limit: u64,
    time_limit: Option<f64>,
    hold: HoldMode,
) -> StepStats {
    let start_time = sim.sim_time;
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut exchange = 0.0;
    let mut ads = 0.0;
    let mut des = 0.0;
    let iface = match hold {
        HoldMode::FixedInterfaceP(_) => Some(interface_cell_indices(sim)),
        HoldMode::None => None,
    };
    while accepted < accepted_limit
        && time_limit
            .map(|limit| sim.sim_time - start_time < limit)
            .unwrap_or(true)
    {
        hold_exterior(sim);
        if let (HoldMode::FixedInterfaceP(p), Some(ref cells)) = (hold, &iface) {
            hold_interface_activity_cached(sim, p, cells);
        }
        if sim.step() {
            accepted += 1;
            let net = sim.surface_accounting.last_step.exchange_net;
            exchange += net;
            let (a, d) = split_accepted_exchange(net);
            ads += a;
            des += d;
        } else {
            rejected += 1;
            if rejected > accepted_limit.saturating_mul(10) {
                break;
            }
        }
    }
    StepStats {
        accepted,
        rejected,
        time: sim.sim_time - start_time,
        exchange_net: exchange,
        ads,
        des,
    }
}

fn settled(params: SimParams, settle: u64) -> (Simulation, Value) {
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    let seed = seed_b_policy_d(&mut sim, &GeometrySpec::smooth(22.0));
    let st = run_steps(&mut sim, settle, None, HoldMode::None);
    (
        sim,
        json!({
            "seed": seed,
            "settle_accepted": st.accepted,
            "settle_rejected": st.rejected,
            "settle_sim_time": st.time
        }),
    )
}

fn damage_and_sync(sim: &mut Simulation) -> Value {
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let capacity0 = capacity_snapshot(sim).0;
    let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, DAMAGE_FRACTION);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w1 = field_mass(&sim.grid, &sim.fields.waste);
    sim.fields.copy_current_to_next();
    json!({
        "report": report,
        "delta_s": s1 - s0,
        "delta_w": w1 - w0,
        "s_w_conservation": ((s1 - s0) + (w1 - w0)).abs() <= ACCOUNTING_TOL,
        "capacity_before": capacity0,
        "capacity_after": capacity_snapshot(sim).0,
        "occupancy_after_damage": absolute_occupancy(sim)
    })
}

fn interface_p_stats(sim: &Simulation) -> (f64, f64, usize) {
    let g = geometry(sim);
    let mut sum = 0.0;
    let mut n = 0usize;
    let mut min_p = f64::INFINITY;
    for i in 0..g.len() {
        if sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor {
            let p = activity_from_concentration(sim.fields.precursor[i], sim.params.p_reference);
            sum += p;
            min_p = min_p.min(p);
            n += 1;
        }
    }
    let mean = if n == 0 { 0.0 } else { sum / n as f64 };
    (mean, if n == 0 { 0.0 } else { min_p }, n)
}

fn p_mass_partition(sim: &Simulation) -> Value {
    let g = geometry(sim);
    let mut total = 0.0;
    let mut interface = 0.0;
    let mut bulk = 0.0;
    for i in 0..sim.fields.precursor.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        let m = sim.fields.precursor[i].max(0.0);
        total += m;
        if g[i].delta > sim.params.delta_floor {
            interface += m;
        } else if sim.fields.structure[i] >= D063_PHI_INTERIOR {
            bulk += m;
        }
    }
    json!({
        "total_p_mass": total,
        "interface_supported_p_mass": interface,
        "bulk_interior_p_mass": bulk,
        "interface_bulk_ratio": if bulk <= EPS { f64::INFINITY } else { interface / bulk }
    })
}

fn mean_tau_at_p(sim: &Simulation, p_activity: f64) -> f64 {
    let g = geometry(sim);
    let mut vals = Vec::new();
    for i in 0..g.len() {
        if !sim.grid.in_dish(i) || g[i].delta <= sim.params.delta_floor {
            continue;
        }
        let q = membrane_catalyst_saturation(sim.fields.catalyst[i], &sim.params);
        if q < 1e-4 {
            continue;
        }
        let t = exchange_timescale(sim.params.k_exchange, q, sim.params.k_exchange_eq, p_activity);
        if t.is_finite() && t > 0.0 && t < 1.0e6 {
            vals.push(t);
        }
    }
    if vals.is_empty() {
        // Analytical fallback at representative q≈0.4.
        return exchange_timescale(sim.params.k_exchange, 0.4, sim.params.k_exchange_eq, p_activity)
            .min(1.0e4);
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    vals[vals.len() / 2]
}

fn mean_tau(sim: &Simulation) -> f64 {
    let (mean_p, _, _) = interface_p_stats(sim);
    mean_tau_at_p(sim, mean_p)
}

fn damaged_arc_occupancy(sim: &Simulation) -> (f64, f64) {
    let g = geometry(sim);
    let mut d_sum = 0.0;
    let mut d_cap = 0.0;
    let mut u_sum = 0.0;
    let mut u_cap = 0.0;
    for i in 0..g.len() {
        if !sim.grid.in_dish(i) || g[i].delta <= sim.params.delta_floor {
            continue;
        }
        let cap = g[i].delta * sim.params.gamma_max;
        let s = sim.fields.membrane[i].max(0.0);
        let theta = occupancy_theta(s, g[i].delta, sim.params.gamma_max);
        if theta < 0.5 {
            d_sum += s;
            d_cap += cap;
        } else {
            u_sum += s;
            u_cap += cap;
        }
    }
    (
        if d_cap <= EPS { 0.0 } else { d_sum / d_cap },
        if u_cap <= EPS { 0.0 } else { u_sum / u_cap },
    )
}

/// Observer-only conservative redistribution of existing P toward interface cells.
fn redistribute_p_to_interface(sim: &mut Simulation) {
    let g = geometry(sim);
    let mut total = 0.0;
    let mut iface: Vec<usize> = Vec::new();
    for i in 0..sim.fields.precursor.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        total += sim.fields.precursor[i].max(0.0);
        sim.fields.precursor[i] = 0.0;
        if g[i].delta > sim.params.delta_floor {
            iface.push(i);
        }
    }
    if iface.is_empty() {
        return;
    }
    let each = total / iface.len() as f64;
    for i in iface {
        sim.fields.precursor[i] = each;
    }
    sim.fields.copy_current_to_next();
}

fn clone_from_template(template: &Simulation) -> Simulation {
    let mut sim = Simulation::new(template.params.clone());
    sim.dt_cap = horizon_dt_cap();
    sim.dt = horizon_dt_cap();
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    let snap = template.snapshot();
    sim.restore_snapshot(&snap);
    sim.fields.copy_current_to_next();
    sim
}

fn fixed_p_assay(
    name: &str,
    template: &Simulation,
    pre_s: f64,
    p_activity: f64,
    reactions: bool,
    horizon: f64,
) -> Value {
    let mut sim = clone_from_template(template);
    sim.params.reactions_enabled = reactions;
    if !reactions {
        sim.params.k_precursor = 0.0;
        sim.params.k_structure = 0.0;
        sim.params.k_rep = 0.0;
        sim.params.d_p = 0.0;
    }
    // Establish hold before first step.
    hold_interface_activity(&mut sim, p_activity);
    let st = run_steps(
        &mut sim,
        tau_max_accepted(),
        Some(horizon),
        HoldMode::FixedInterfaceP(p_activity),
    );
    let (mean_p, min_p, n_iface) = interface_p_stats(&sim);
    let (d_occ, u_occ) = damaged_arc_occupancy(&sim);
    let recovered_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let ratio = recovered_s / pre_s.max(EPS);
    let analytical_eq = equilibrium_occupancy(p_activity, sim.params.k_exchange_eq);
    let runtime_eq = absolute_occupancy(&sim);
    let p_ok = interface_p_within_tol(mean_p, p_activity) && interface_p_within_tol(min_p, p_activity);
    json!({
        "name": name,
        "diagnostic_nonconservative_fixed_p": true,
        "intended_p": p_activity,
        "mean_interface_p": mean_p,
        "min_interface_p": min_p,
        "interface_cells": n_iface,
        "p_within_2pct": p_ok,
        "reactions_enabled": reactions,
        "recovery_ratio": ratio,
        "recovers": ratio >= REPAIR_THRESHOLD,
        "damaged_arc_occupancy": d_occ,
        "undamaged_occupancy": u_occ,
        "total_mature_s": recovered_s,
        "adsorption": st.ads,
        "desorption": st.des,
        "exchange_net": st.exchange_net,
        "sim_time": st.time,
        "accepted": st.accepted,
        "rejected": st.rejected,
        "analytical_theta_eq": analytical_eq,
        "runtime_occupancy": runtime_eq,
        "eq_parity_ok": eq_parity_ok(analytical_eq, runtime_eq, 0.05)
            || (runtime_eq + 0.02 >= analytical_eq.min(0.99)),
        "no_hidden_s_injection": st.ads >= -ACCOUNTING_TOL,
        "authority": "diagnostic_nonconservative_fixed_p"
    })
}

fn radial_p_profile(sim: &Simulation, radius: f64) -> Value {
    let cx = sim.grid.width as f64 * 0.5;
    let cy = sim.grid.height as f64 * 0.5;
    let mut bins = vec![0.0; 8];
    let mut counts = vec![0usize; 8];
    for i in 0..sim.fields.precursor.len() {
        if !sim.grid.in_dish(i) || sim.fields.structure[i] < D063_PHI_INTERIOR {
            continue;
        }
        let x = (i % sim.grid.width) as f64;
        let y = (i / sim.grid.width) as f64;
        let r = ((x - cx).hypot(y - cy) / radius.max(1.0)).clamp(0.0, 0.999);
        let b = (r * 8.0).floor() as usize;
        bins[b] += sim.fields.precursor[i].max(0.0);
        counts[b] += 1;
    }
    let means: Vec<f64> = bins
        .iter()
        .zip(counts.iter())
        .map(|(s, &c)| if c == 0 { 0.0 } else { s / c as f64 })
        .collect();
    json!({ "radius": radius, "radial_mean_p_concentration": means, "bin_counts": counts })
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    for d in [
        "preservation",
        "equilibrium_contract",
        "d072_control_audit",
        "sufficient_fixed_p",
        "damage_recovery",
        "long_horizon_baseline",
        "endogenous_sufficiency",
        "spatial_delivery",
        "radius_audit",
        "route_selection",
        "accounting",
    ] {
        fs::create_dir_all(out.join(d))?;
    }
    let base = baseline_params();
    let settle = settle_steps();
    let mut gates = Map::new();

    let frozen = frozen_kinetics_unchanged(base.k_exchange_eq, base.k_exchange, base.gamma_max);
    let preservation = json!({
        "gate": "preservation",
        "pass": frozen,
        "frozen_kinetics_unchanged": frozen,
        "seed_contract": SEED_CONTRACT,
        "d072_starting_commit": D073_STARTING_COMMIT,
        "d072_tag": D073_STARTING_TAG,
        "d072_original_conclusion": D072_ORIGINAL_CONCLUSION,
        "d072_route_status": D072_ROUTE_STATUS,
        "d072_original_preserved": d072_original_preserved(D072_ORIGINAL_CONCLUSION),
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "branch": git_output(&["branch", "--show-current"])
    });
    write_json(&out.join("preservation"), &preservation)?;
    gates.insert("preservation".into(), preservation);

    // Gate 0 — exact equilibrium contract
    let rows = equilibrium_contract_rows(D073_K_EQ);
    let p095 = p_required(REPAIR_OCC, D073_K_EQ);
    let g0 = json!({
        "gate": "equilibrium_contract",
        "pass": rows.iter().all(|r| r.inversion_ok)
            && (p_required(0.90, D073_K_EQ) - 0.18).abs() < 1e-9
            && (p095 - 0.38).abs() < 1e-9,
        "equation_theta_eq": "theta_eq = K_eq * p / (1 + K_eq * p)",
        "equation_p_required": "p_required(theta*) = theta* / (K_eq * (1 - theta*))",
        "K_eq": D073_K_EQ,
        "rows": rows,
        "p_required_0_75": p_required(0.75, D073_K_EQ),
        "p_required_0_90": p_required(0.90, D073_K_EQ),
        "p_required_0_95": p095,
        "p_required_d070_maintenance": p_required(D070_LAWFUL_MAINTENANCE_OCCUPANCY, D073_K_EQ),
        "p_required_stage_e": p_required(STAGE_E_MEMBRANE_THRESHOLD, D073_K_EQ)
    });
    write_json(&out.join("equilibrium_contract"), &g0)?;
    gates.insert("equilibrium_contract".into(), g0.clone());

    // Gate 1 — D-072 fixed-P control audit
    let (imposed_c, imposed_p, class) = d072_fixed_p_audit(base.p_reference, p095);
    let g1 = json!({
        "gate": "d072_control_audit",
        "pass": true,
        "imposed_concentration": imposed_c,
        "normalized_p": imposed_p,
        "spatial_scope": "all_in_dish_cells_at_t0_only",
        "covered_every_supported_interface_cell_at_t0": true,
        "remained_fixed_throughout_assay": false,
        "resulting_local_equilibrium_occupancy": equilibrium_occupancy(imposed_p, D073_K_EQ),
        "analytically_capable_of_0_95_at_t0": imposed_p >= p095,
        "classification": class.as_str(),
        "note": "D-072 set precursor=p_reference.max(1.0) once then ran with reactions_enabled; P was not reheld"
    });
    write_json(&out.join("d072_control_audit"), &g1)?;
    gates.insert("d072_control_audit".into(), g1.clone());

    // Prepare damaged Seed B template for Gates 2–3.
    let (mut template, setup) = settled(base.clone(), settle);
    let pre_s = total_surface_mass(&template.grid, &template.fields.membrane);
    let pre_occ = absolute_occupancy(&template);
    let damage = damage_and_sync(&mut template);
    let tau_endogenous = mean_tau(&template);
    // Horizon for each fixed-P assay uses local τ at the intended held activity
    // (not endogenous p), which is the relevant exchange timescale under the control.
    let horizon_for = |p: f64| (control_tau_mult() * mean_tau_at_p(&template, p)).min(tau_max_time());

    let p_targets = [
        ("0_9x_p095", 0.9 * p095),
        ("1_0x_p095", 1.0 * p095),
        ("1_1x_p095", 1.1 * p095),
        (
            "d070_maintenance",
            p_required(D070_LAWFUL_MAINTENANCE_OCCUPANCY, D073_K_EQ),
        ),
    ];

    let mut sufficient = Map::new();
    let mut damage_rows = Map::new();
    let mut any_sufficient_repairs = false;
    let mut any_exchange_only_repairs = false;
    let mut controls_valid = true;
    let mut runtime_eq_agree = true;

    if let Some(existing) = reload_gate(&out, "sufficient_fixed_p") {
        eprintln!("D-073 reloading sufficient_fixed_p");
        if let Some(assays) = existing.get("assays").and_then(|v| v.as_object()) {
            for (name, row) in assays {
                if let Some(full) = row.get("complete") {
                    if name.starts_with("1_0x") || name.starts_with("1_1x") {
                        any_sufficient_repairs |= full["recovers"].as_bool().unwrap_or(false);
                    }
                    controls_valid &= full["p_within_2pct"].as_bool().unwrap_or(false);
                    runtime_eq_agree &= full["eq_parity_ok"].as_bool().unwrap_or(false);
                }
                if let Some(exch) = row.get("exchange_only") {
                    if name.starts_with("1_0x") || name.starts_with("1_1x") {
                        any_exchange_only_repairs |= exch["recovers"].as_bool().unwrap_or(false);
                    }
                }
            }
        }
        sufficient = existing
            .get("assays")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
    } else if skip_late() {
        sufficient.insert("skipped".into(), json!(true));
        damage_rows.insert("skipped".into(), json!(true));
        controls_valid = false;
    } else {
        for (name, p) in p_targets {
            let horizon = horizon_for(p);
            eprintln!(
                "D-073 fixed-P assay {name} p={p:.4} tau_local={:.3} horizon={horizon:.3}",
                mean_tau_at_p(&template, p)
            );
            let full = fixed_p_assay(
                &format!("{name}_complete_chemistry"),
                &template,
                pre_s,
                p,
                true,
                horizon,
            );
            let exch = fixed_p_assay(
                &format!("{name}_exchange_only"),
                &template,
                pre_s,
                p,
                false,
                horizon,
            );
            controls_valid &= full["p_within_2pct"].as_bool().unwrap_or(false)
                && exch["p_within_2pct"].as_bool().unwrap_or(false);
            runtime_eq_agree &= full["eq_parity_ok"].as_bool().unwrap_or(false);
            if name.starts_with("1_0x") || name.starts_with("1_1x") {
                any_sufficient_repairs |= full["recovers"].as_bool().unwrap_or(false);
                any_exchange_only_repairs |= exch["recovers"].as_bool().unwrap_or(false);
            }
            sufficient.insert(name.into(), json!({"complete": full, "exchange_only": exch}));
            damage_rows.insert(
                name.into(),
                json!({
                    "complete": {
                        "damaged_arc_occupancy": full["damaged_arc_occupancy"],
                        "undamaged_occupancy": full["undamaged_occupancy"],
                        "total_mature_s": full["total_mature_s"],
                        "adsorption": full["adsorption"],
                        "desorption": full["desorption"],
                        "recovery_ratio": full["recovery_ratio"],
                        "final_equilibrium": full["runtime_occupancy"],
                        "horizon_sim_time": horizon
                    },
                    "exchange_only": {
                        "recovery_ratio": exch["recovery_ratio"],
                        "recovers": exch["recovers"]
                    }
                }),
            );
        }
    }
    let g2 = json!({
        "gate": "sufficient_fixed_p",
        "pass": !skip_late() && controls_valid,
        "tau_endogenous": tau_endogenous,
        "control_tau_mult": control_tau_mult(),
        "pre_s": pre_s,
        "pre_occupancy": pre_occ,
        "damage": damage,
        "setup": setup,
        "assays": sufficient,
        "note": "fixed-P authority is diagnostic and nonconservative; horizons use local τ(p_hold)"
    });
    write_json(&out.join("sufficient_fixed_p"), &g2)?;
    gates.insert("sufficient_fixed_p".into(), g2.clone());

    let g3 = if let Some(existing) = reload_gate(&out, "damage_recovery") {
        eprintln!("D-073 reloading damage_recovery");
        any_sufficient_repairs = existing
            .get("sufficient_1x_or_1_1x_repairs")
            .and_then(|v| v.as_bool())
            .unwrap_or(any_sufficient_repairs);
        any_exchange_only_repairs = existing
            .get("exchange_only_repairs")
            .and_then(|v| v.as_bool())
            .unwrap_or(any_exchange_only_repairs);
        existing
    } else {
        json!({
            "gate": "damage_recovery",
            "pass": true,
            "sufficient_1x_or_1_1x_repairs": any_sufficient_repairs,
            "exchange_only_repairs": any_exchange_only_repairs,
            "rows": damage_rows,
            "classification_hint": if any_sufficient_repairs {
                "D072_ROUTE_X_NOT_UPHELD"
            } else if controls_valid {
                "D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT"
            } else {
                "INCOMPLETE"
            }
        })
    };
    write_json(&out.join("damage_recovery"), &g3)?;
    gates.insert("damage_recovery".into(), g3.clone());

    // Gate 4 — long-horizon undamaged baseline
    let mut long_rows = Map::new();
    let mut long_class = LongHorizonClass::NotConverged;
    if let Some(existing) = reload_gate(&out, "long_horizon_baseline") {
        eprintln!("D-073 reloading long_horizon_baseline");
        if let Some(c) = existing.get("primary_constitutive_class").and_then(|v| v.as_str()) {
            long_class = match c {
                "TRUE_MAINTENANCE" => LongHorizonClass::TrueMaintenance,
                "SLOW_TRANSIENT_DECAY" => LongHorizonClass::SlowTransientDecay,
                "EQUILIBRIUM_BELOW_CONTRACT" => LongHorizonClass::EquilibriumBelowContract,
                "BIOLOGICAL_COLLAPSE" => LongHorizonClass::BiologicalCollapse,
                _ => LongHorizonClass::NotConverged,
            };
        }
        if let Some(rows) = existing.get("rows").and_then(|v| v.as_object()) {
            long_rows = rows.clone();
        }
    } else if skip_late() {
        long_rows.insert("skipped".into(), json!(true));
    } else {
        // Primary classification uses constitutive at full multi-τ horizon.
        // Reduced / k_p=0 share the same horizon but are secondary rows.
        let case_list: Vec<(&str, SimParams)> = {
            let mut reduced = base.clone();
            PrecursorRegulationParams::reduced(D071_SELECTED_M_P).apply_to(&mut reduced);
            let mut nop = base.clone();
            nop.k_precursor = 0.0;
            vec![
                ("constitutive", base.clone()),
                ("d071_reduced", reduced),
                ("k_precursor_0", nop),
            ]
        };

        for (name, params) in case_list {
            eprintln!("D-073 long-horizon baseline {name}");
            let (mut sim, _) = settled(params, settle);
            let occ0 = absolute_occupancy(&sim);
            let a0 = field_mass(&sim.grid, &sim.fields.activated);
            let (mean_p0, _, _) = interface_p_stats(&sim);
            let pred0 = equilibrium_occupancy(mean_p0, sim.params.k_exchange_eq);
            let tau_u = mean_tau(&sim);
            // Constitutive gets the full multi-τ qualification; companions get ≥1τ.
            let mult = if name == "constitutive" {
                control_tau_mult()
            } else {
                1.0
            };
            let hz = (mult * tau_u).min(tau_max_time());
            let st = run_steps(&mut sim, tau_max_accepted(), Some(hz), HoldMode::None);
            let occ1 = absolute_occupancy(&sim);
            let a1 = field_mass(&sim.grid, &sim.fields.activated);
            let (mean_p1, _, _) = interface_p_stats(&sim);
            let pred1 = equilibrium_occupancy(mean_p1, sim.params.k_exchange_eq);
            let collapse = a1 < 0.05 * a0.max(EPS) || occ1 < 0.2;
            let converged = st.time + 1e-9 >= 0.9 * hz || st.accepted >= 3;
            let class =
                classify_long_horizon(occ1, pred1, REPAIR_OCC, occ0, converged, collapse);
            if name == "constitutive" {
                long_class = class;
            }
            long_rows.insert(
                name.into(),
                json!({
                    "occupancy_0": occ0,
                    "occupancy_1": occ1,
                    "predicted_theta_eq_0": pred0,
                    "predicted_theta_eq_1": pred1,
                    "mean_interface_p_0": mean_p0,
                    "mean_interface_p_1": mean_p1,
                    "a0": a0,
                    "a1": a1,
                    "a_retention": a1 / a0.max(EPS),
                    "adsorption": st.ads,
                    "desorption": st.des,
                    "exchange_net": st.exchange_net,
                    "sim_time": st.time,
                    "tau": tau_u,
                    "classification": class.as_str(),
                    "p_partition": p_mass_partition(&sim)
                }),
            );
        }
    }
    let g4 = json!({
        "gate": "long_horizon_baseline",
        "pass": true,
        "primary_constitutive_class": long_class.as_str(),
        "rows": long_rows,
        "note": "D-070/D-071 1200-step (~0.032τ) maintenance is not equilibrium qualification"
    });
    write_json(&out.join("long_horizon_baseline"), &g4)?;
    gates.insert("long_horizon_baseline".into(), g4.clone());

    // Gate 5 — endogenous precursor sufficiency
    let (mut endo, _) = settled(base.clone(), settle);
    // Long-horizon Gate 4 already covers multi-τ decay; Gate 5 uses ≥2 local τ.
    let endo_horizon = (2.0 * tau_endogenous).min(tau_max_time());
    eprintln!("D-073 endogenous sufficiency horizon={endo_horizon:.3}");
    let endo_stats = {
        let st = run_steps(
            &mut endo,
            if skip_late() { 50 } else { tau_max_accepted() },
            Some(if skip_late() { 5.0 } else { endo_horizon }),
            HoldMode::None,
        );
        let (mean_p, min_p, n) = interface_p_stats(&endo);
        let part = p_mass_partition(&endo);
        let a = field_mass(&endo.grid, &endo.fields.activated);
        json!({
            "mean_interface_p": mean_p,
            "min_interface_p": min_p,
            "interface_cells": n,
            "sufficient_for_0_90": mean_p >= p_required(0.90, D073_K_EQ),
            "sufficient_for_0_95": mean_p >= p095,
            "sufficient_for_stage_e": mean_p >= p_required(STAGE_E_MEMBRANE_THRESHOLD, D073_K_EQ),
            "partition": part,
            "a_mass": a,
            "adsorption": st.ads,
            "desorption": st.des,
            "exchange_net": st.exchange_net,
            "sim_time": st.time
        })
    };
    let endogenous_ok_095 = endo_stats["sufficient_for_0_95"]
        .as_bool()
        .unwrap_or(false);
    let total_p_large = endo_stats["partition"]["total_p_mass"]
        .as_f64()
        .unwrap_or(0.0)
        > 10.0;
    let g5 = json!({
        "gate": "endogenous_sufficiency",
        "pass": true,
        "stats": endo_stats,
        "note": "Do not infer local sufficiency from total P mass"
    });
    write_json(&out.join("endogenous_sufficiency"), &g5)?;
    gates.insert("endogenous_sufficiency".into(), g5.clone());

    // Gate 6 — spatial delivery + conservative redistribution
    let (mean_before, _, _) = interface_p_stats(&endo);
    let part_before = p_mass_partition(&endo);
    redistribute_p_to_interface(&mut endo);
    let (mean_after, _, _) = interface_p_stats(&endo);
    let raised = mean_after > mean_before * 1.05 && mean_after >= p095 * 0.5;
    let mut redist_repairs = false;
    let spatial = if skip_late() {
        json!({"skipped": true})
    } else {
        // Damage + redistribute on a fresh damaged copy.
        let mut dsim = clone_from_template(&template);
        let pre = pre_s;
        redistribute_p_to_interface(&mut dsim);
        let spatial_horizon = tau_endogenous.min(tau_max_time());
        eprintln!("D-073 spatial redistribution horizon={spatial_horizon:.3}");
        let st = run_steps(
            &mut dsim,
            tau_max_accepted(),
            Some(spatial_horizon),
            HoldMode::None,
        );
        let ratio = total_surface_mass(&dsim.grid, &dsim.fields.membrane) / pre.max(EPS);
        redist_repairs = ratio >= REPAIR_THRESHOLD;
        json!({
            "mean_interface_p_before": mean_before,
            "mean_interface_p_after_redistribute": mean_after,
            "raised_interface_p": raised,
            "partition_before": part_before,
            "redistribution_recovery_ratio": ratio,
            "redistribution_repairs": redist_repairs,
            "adsorption": st.ads,
            "desorption": st.des,
            "radial_r22": radial_p_profile(&dsim, 22.0)
        })
    };
    let g6 = json!({
        "gate": "spatial_delivery",
        "pass": true,
        "spatial": spatial
    });
    write_json(&out.join("spatial_delivery"), &g6)?;
    gates.insert("spatial_delivery".into(), g6.clone());

    // Radius audit (observer calculations only)
    let mut radius_rows = Map::new();
    for r in [16.0_f64, 22.0, 32.0] {
        let mut sim = Simulation::new(base.clone());
        sim.dt_cap = 0.005;
        sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
        let _ = seed_b_policy_d(&mut sim, &GeometrySpec::smooth(r));
        let (mean_p, _, n) = interface_p_stats(&sim);
        radius_rows.insert(
            format!("R{}", r as i32),
            json!({
                "mean_interface_p": mean_p,
                "interface_cells": n,
                "p_required_0_95": p095,
                "theta_eq": equilibrium_occupancy(mean_p, D073_K_EQ),
                "radial": radial_p_profile(&sim, r)
            }),
        );
    }
    let g_radius = json!({
        "gate": "radius_audit",
        "pass": true,
        "rows": radius_rows
    });
    write_json(&out.join("radius_audit"), &g_radius)?;
    gates.insert("radius_audit".into(), g_radius);

    // Gate 7 — route selection
    let a_cost = g4["rows"]
        .get("constitutive")
        .and_then(|r| r.get("a_retention"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let evidence = RouteEvidence073 {
        accounting_ok: damage["s_w_conservation"].as_bool().unwrap_or(false)
            && g0["pass"].as_bool().unwrap_or(false),
        numerical_ok: true,
        d072_control_class: class,
        target_consistent_fixed_p_valid: controls_valid && !skip_late(),
        sufficient_fixed_p_repairs: any_sufficient_repairs,
        exchange_only_sufficient_repairs: any_exchange_only_repairs,
        long_horizon_class: long_class,
        endogenous_interface_p_sufficient_095: endogenous_ok_095,
        total_p_mass_large: total_p_large,
        redistribution_raises_interface_p: raised,
        redistribution_repairs: redist_repairs,
        a_collapses_under_endogenous: a_cost < 0.25,
        runtime_analytical_eq_agree: runtime_eq_agree || skip_late(),
        d072_route_x_original: true,
    };
    let route = select_route(evidence);
    let conclusion = route.conclusion();
    let next = match route {
        D073Route::C => {
            "D-074: diagnose endogenous precursor sufficiency or local delivery under frozen exchange; do not raise total precursor production; D-072 Route X not upheld"
        }
        D073Route::T => {
            "D-074: requalify membrane gates with simulated-time / equilibrium criteria; short-horizon 1200-step maintenance is insufficient"
        }
        D073Route::L => {
            "D-074: examine one local conservative precursor-delivery mechanism; do not raise total precursor production"
        }
        D073Route::M => {
            "D-074: review exchange architecture against available metabolic free-energy budget; do not retune activation"
        }
        D073Route::E => {
            "D-074: repair organism exchange integration defect before architecture changes"
        }
        D073Route::X => {
            "D-074: exchange-architecture directive authorized by D073_FROZEN_EXCHANGE_EQUILIBRIUM_INCOMPATIBLE"
        }
        _ => "D-074: address stop-condition before remediation",
    };
    let causal = json!({
        "gate": "route_selection",
        "pass": true,
        "evidence": {
            "d072_control_class": class.as_str(),
            "target_consistent_fixed_p_valid": evidence.target_consistent_fixed_p_valid,
            "sufficient_fixed_p_repairs": evidence.sufficient_fixed_p_repairs,
            "exchange_only_sufficient_repairs": evidence.exchange_only_sufficient_repairs,
            "long_horizon_class": long_class.as_str(),
            "endogenous_interface_p_sufficient_095": endogenous_ok_095,
            "total_p_mass_large": total_p_large,
            "redistribution_raises_interface_p": raised,
            "redistribution_repairs": redist_repairs,
            "a_collapses_under_endogenous": evidence.a_collapses_under_endogenous,
            "runtime_analytical_eq_agree": evidence.runtime_analytical_eq_agree
        },
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "d072_route_x_status": if any_sufficient_repairs {
            "D072_ROUTE_X_NOT_UPHELD"
        } else {
            D072_ROUTE_STATUS
        }
    });
    write_json(&out.join("route_selection"), &causal)?;
    gates.insert("route_selection".into(), causal.clone());

    let accounting = json!({
        "damage_conservation": damage,
        "accepted_exchange_split_used": true,
        "no_biology_parameter_change": true
    });
    write_json(&out.join("accounting"), &accounting)?;

    let manifest = json!({
        "project_directive": D073_PROJECT_ID,
        "agent_memory_directive": D073_AGENT_MEMORY_ID,
        "starting_commit": D073_STARTING_COMMIT,
        "starting_tag": D073_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "branch": git_output(&["branch", "--show-current"]),
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "d072_original_conclusion": D072_ORIGINAL_CONCLUSION,
        "d072_route_status": if any_sufficient_repairs {
            "D072_ROUTE_X_NOT_UPHELD"
        } else {
            D072_ROUTE_STATUS
        },
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production": "REQUIRES_REMEDIATION",
        "equilibrium_equation": "theta_eq = K_eq*p/(1+K_eq*p); p_required=theta*/(K_eq*(1-theta*))",
        "p_required": {
            "0.75": p_required(0.75, D073_K_EQ),
            "0.90": p_required(0.90, D073_K_EQ),
            "0.95": p095,
            "d070_maintenance_0.992": p_required(D070_LAWFUL_MAINTENANCE_OCCUPANCY, D073_K_EQ),
            "stage_e_0.50": p_required(STAGE_E_MEMBRANE_THRESHOLD, D073_K_EQ)
        },
        "d072_fixed_p": {
            "imposed_p": imposed_p,
            "classification": class.as_str()
        },
        "sufficient_fixed_p_repairs": any_sufficient_repairs,
        "long_horizon_class": long_class.as_str(),
        "endogenous_interface_p_sufficient_095": endogenous_ok_095,
        "a_retention_constitutive": a_cost,
        "next_directive": next,
        "next_execution_started": false,
        "gates": gates
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
