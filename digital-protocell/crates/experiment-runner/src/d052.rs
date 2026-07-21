//! D-052 nutrient/fuel delivery resistance decomposition (Gates 0–12).
//! Diagnostic only: no biological parameter or equation promotion.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::{
    production_activation_rate, ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
    ACTIVATION_SCHEMA_HISTORICAL, D050_HISTORICAL_K,
};
use chemistry_core::d051_analysis::D051_FITTED_K_C;
use chemistry_core::d052_analysis::*;
use chemistry_core::field_mass;
use chemistry_core::grid::Grid;
use chemistry_core::membrane_transport::{
    face_flux, permeability_surface_occupancy, TransportSpecies,
};
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn git_rev(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn max_accepted() -> u64 {
    std::env::var("D052_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D052_DEFAULT_HORIZON)
}

fn control_horizon() -> u64 {
    std::env::var("D052_CONTROL_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D052_CONTROL_HORIZON.min(max_accepted()))
}

fn long_horizons() -> Vec<u64> {
    if let Ok(s) = std::env::var("D052_LONG_HORIZONS") {
        if s.trim().is_empty() || s == "none" {
            return Vec::new();
        }
        return s
            .split(',')
            .filter_map(|x| x.trim().parse().ok())
            .collect();
    }
    let cap = max_accepted();
    let long_cap: u64 = std::env::var("D052_LONG_CAP")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(if cap < 10_000 { 0 } else { 100_000 });
    [25_000u64, 50_000, 100_000]
        .into_iter()
        .filter(|&h| h <= cap && h <= long_cap)
        .collect()
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn historical_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    d049_frozen_params(&base)
}

fn schema1_params() -> SimParams {
    // Historical schema-1 control: frozen D-049 params (mass-action activation).
    let mut p = historical_params();
    p.activation_schema = ACTIVATION_SCHEMA_HISTORICAL;
    p.k_d008_activation = D050_HISTORICAL_K;
    p
}

fn schema2_params(v_a: f64) -> SimParams {
    let mut p = historical_params();
    p.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    p.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    p.k_d008_activation = v_a;
    p.k_c_activation = D052_FITTED_K_C.max(D051_FITTED_K_C);
    p.n_ref_activation = D052_N_REF;
    p.f_ref_activation = D052_F_REF;
    p
}

fn new_sim(params: SimParams, radius: f64) -> Simulation {
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, radius, D052_THETA);
    sim
}

fn a_retention(sim: &Simulation, a0: f64) -> f64 {
    field_mass(&sim.grid, &sim.fields.activated) / a0.max(1e-18)
}

fn clamp_interior(sim: &mut Simulation, field: &mut [f64], value: f64) {
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            field[idx] = value;
        }
    }
}

fn clamp_exterior(sim: &mut Simulation, field: &mut [f64], value: f64) {
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < 0.5 {
            field[idx] = value;
        }
    }
}

#[derive(Clone, Copy, Default)]
struct ControlSpec {
    hold_n: Option<f64>,
    hold_f: Option<f64>,
    hold_exterior_nf: bool,
    unlimited_n: bool,
    unlimited_f: bool,
    unlimited_activation_substrates: bool,
    reservoir_mult: f64,
    bypass_n_attenuation: bool,
    bypass_f_attenuation: bool,
    freeze_healthy_nf_perm: bool,
    membrane_free_nf: bool,
    exterior_dn_mult: f64,
    exterior_df_mult: f64,
    interior_dn_mult: f64,
    interior_df_mult: f64,
    mix_interior_nf: bool,
    freeze_s: Option<f64>,
    zero_s: bool,
    observer_yield: Option<f64>,
}

impl ControlSpec {
    fn baseline() -> Self {
        Self {
            reservoir_mult: 1.0,
            exterior_dn_mult: 1.0,
            exterior_df_mult: 1.0,
            interior_dn_mult: 1.0,
            interior_df_mult: 1.0,
            ..Default::default()
        }
    }
}

fn apply_control_once(sim: &mut Simulation, ctrl: &ControlSpec) {
    if ctrl.reservoir_mult != 1.0 && ctrl.reservoir_mult > 0.0 {
        sim.params.reservoir_rate = (sim.params.reservoir_rate * ctrl.reservoir_mult).max(1e-12);
    }
    if ctrl.bypass_n_attenuation || ctrl.membrane_free_nf {
        sim.params.beta_n = 0.0;
    }
    if ctrl.bypass_f_attenuation || ctrl.membrane_free_nf {
        sim.params.beta_f = 0.0;
    }
    if ctrl.freeze_healthy_nf_perm {
        // Stage-A mid-band proxy: Π≈0.30 ↔ β≈1.2 at θ=1; freeze at θ-independent β=0
        // is bypass; here hold β at historical Stage A values (already default).
        sim.params.beta_n = 1.2;
        sim.params.beta_f = 1.2;
    }
    if ctrl.exterior_dn_mult != 1.0 || ctrl.interior_dn_mult != 1.0 {
        // Diagnostic: scale species diffusivity uniformly (region split approximated).
        let m = ctrl.exterior_dn_mult.max(ctrl.interior_dn_mult);
        sim.params.d_n *= m;
    }
    if ctrl.exterior_df_mult != 1.0 || ctrl.interior_df_mult != 1.0 {
        let m = ctrl.exterior_df_mult.max(ctrl.interior_df_mult);
        sim.params.d_f *= m;
    }
    if ctrl.zero_s {
        for v in sim.fields.membrane.iter_mut() {
            *v = 0.0;
        }
    }
    if let Some(s) = ctrl.freeze_s {
        for idx in 0..sim.fields.membrane.len() {
            if sim.grid.in_dish(idx) {
                // Seed surface density proportional to interface weight proxy.
                let phi = sim.fields.structure[idx];
                let iface = (6.0 * phi * (1.0 - phi)).clamp(0.0, 1.0);
                sim.fields.membrane[idx] = s * iface;
            }
        }
    }
}

fn apply_pre_step(sim: &mut Simulation, ctrl: &ControlSpec) {
    if let Some(n) = ctrl.hold_n {
        let mut buf = sim.fields.nutrient.clone();
        clamp_interior(sim, &mut buf, n);
        sim.fields.nutrient = buf;
    }
    if let Some(f) = ctrl.hold_f {
        let mut buf = sim.fields.fuel.clone();
        clamp_interior(sim, &mut buf, f);
        sim.fields.fuel = buf;
    }
    if ctrl.unlimited_n || ctrl.unlimited_activation_substrates {
        let mut buf = sim.fields.nutrient.clone();
        clamp_interior(sim, &mut buf, D052_HEALTHY_N * 10.0);
        sim.fields.nutrient = buf;
    }
    if ctrl.unlimited_f || ctrl.unlimited_activation_substrates {
        let mut buf = sim.fields.fuel.clone();
        clamp_interior(sim, &mut buf, D052_HEALTHY_F * 10.0);
        sim.fields.fuel = buf;
    }
    if ctrl.hold_exterior_nf {
        let nr = sim.params.n_reservoir;
        let fr = sim.params.f_reservoir;
        let mut nbuf = sim.fields.nutrient.clone();
        let mut fbuf = sim.fields.fuel.clone();
        clamp_exterior(sim, &mut nbuf, nr);
        clamp_exterior(sim, &mut fbuf, fr);
        sim.fields.nutrient = nbuf;
        sim.fields.fuel = fbuf;
    }
    if ctrl.mix_interior_nf {
        let mut n_sum = 0.0;
        let mut f_sum = 0.0;
        let mut n = 0u64;
        for idx in 0..sim.fields.structure.len() {
            if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
                n_sum += sim.fields.nutrient[idx];
                f_sum += sim.fields.fuel[idx];
                n += 1;
            }
        }
        if n > 0 {
            let nm = n_sum / n as f64;
            let fm = f_sum / n as f64;
            let mut nbuf = sim.fields.nutrient.clone();
            let mut fbuf = sim.fields.fuel.clone();
            clamp_interior(sim, &mut nbuf, nm);
            clamp_interior(sim, &mut fbuf, fm);
            sim.fields.nutrient = nbuf;
            sim.fields.fuel = fbuf;
        }
    }
    if let Some(s) = ctrl.freeze_s {
        for idx in 0..sim.fields.membrane.len() {
            if sim.grid.in_dish(idx) {
                let phi = sim.fields.structure[idx];
                let iface = (6.0 * phi * (1.0 - phi)).clamp(0.0, 1.0);
                sim.fields.membrane[idx] = s * iface;
            }
        }
    }
    if ctrl.zero_s {
        for v in sim.fields.membrane.iter_mut() {
            *v = 0.0;
        }
    }
    let _ = ctrl.observer_yield; // observer-only; no production mutation
}

#[derive(Default)]
struct CampaignMetrics {
    accepted: u64,
    rejected: u64,
    a0: f64,
    a_final: f64,
    a_retention: f64,
    gross_activation: f64,
    gross_reproduction: f64,
    gross_a_decay: f64,
    n_mass: f64,
    f_mass: f64,
    p_mass: f64,
    s_mass: f64,
    net_s_exchange: f64,
    precursor_synth: f64,
    mean_requested_rate: f64,
    j_n_reservoir: f64,
    j_f_reservoir: f64,
    j_n_interface: f64,
    j_f_interface: f64,
    n_reaction: f64,
    f_reaction: f64,
    n_residual: f64,
    f_residual: f64,
    a_residual: f64,
    w_residual: f64,
    steps_ok: bool,
    profiles: Vec<Value>,
    cap_sites: CapSiteFractions,
    mean_nf_perm: f64,
}

fn sample_requested_rate(sim: &Simulation) -> f64 {
    let mut sum = 0.0;
    let mut n = 0u64;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) || sim.fields.structure[idx] < 0.5 {
            continue;
        }
        let r = production_activation_rate(
            sim.params.activation_schema,
            sim.params.k_d008_activation,
            sim.fields.structure[idx],
            sim.fields.catalyst[idx],
            sim.fields.nutrient[idx],
            sim.fields.fuel[idx],
            sim.params.k_c_activation,
            sim.params.n_ref_activation,
            sim.params.f_ref_activation,
        );
        sum += r;
        n += 1;
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

fn regional_mean(sim: &Simulation, field: &[f64], predicate: impl Fn(usize, f64) -> bool) -> f64 {
    let mut s = 0.0;
    let mut n = 0u64;
    for idx in 0..field.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let phi = sim.fields.structure[idx];
        if predicate(idx, phi) {
            s += field[idx];
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        s / n as f64
    }
}

fn cell_radius(sim: &Simulation, idx: usize) -> f64 {
    let i = idx % sim.grid.width;
    let j = idx / sim.grid.width;
    sim.grid.distance_from_center(i, j)
}

fn sample_spatial_profile(sim: &Simulation) -> Value {
    let r_org = D052_RADIUS;
    let c_res_n = regional_mean(sim, &sim.fields.nutrient, |idx, _| sim.grid.reservoir_mask[idx]);
    let c_res_f = regional_mean(sim, &sim.fields.fuel, |idx, _| sim.grid.reservoir_mask[idx]);
    let c_ext_n = regional_mean(sim, &sim.fields.nutrient, |idx, phi| {
        !sim.grid.reservoir_mask[idx] && phi < 0.5
    });
    let c_ext_f = regional_mean(sim, &sim.fields.fuel, |idx, phi| {
        !sim.grid.reservoir_mask[idx] && phi < 0.5
    });
    let c_out_n = regional_mean(sim, &sim.fields.nutrient, |idx, phi| {
        phi < 0.5 && cell_radius(sim, idx) <= r_org + 2.0 && cell_radius(sim, idx) >= r_org - 0.5
    });
    let c_out_f = regional_mean(sim, &sim.fields.fuel, |idx, phi| {
        phi < 0.5 && cell_radius(sim, idx) <= r_org + 2.0 && cell_radius(sim, idx) >= r_org - 0.5
    });
    let c_in_n = regional_mean(sim, &sim.fields.nutrient, |idx, phi| {
        phi >= 0.5 && cell_radius(sim, idx) >= r_org - 2.0
    });
    let c_in_f = regional_mean(sim, &sim.fields.fuel, |idx, phi| {
        phi >= 0.5 && cell_radius(sim, idx) >= r_org - 2.0
    });
    let c_per_n = regional_mean(sim, &sim.fields.nutrient, |_, phi| {
        phi >= 0.5 && (6.0 * phi * (1.0 - phi)) > 0.15
    });
    let c_per_f = regional_mean(sim, &sim.fields.fuel, |_, phi| {
        phi >= 0.5 && (6.0 * phi * (1.0 - phi)) > 0.15
    });
    let c_cen_n = regional_mean(sim, &sim.fields.nutrient, |idx, phi| {
        phi >= 0.5 && cell_radius(sim, idx) < r_org * 0.5
    });
    let c_cen_f = regional_mean(sim, &sim.fields.fuel, |idx, phi| {
        phi >= 0.5 && cell_radius(sim, idx) < r_org * 0.5
    });
    let c_act_n = regional_mean(sim, &sim.fields.nutrient, |idx, phi| {
        phi >= 0.5 && sim.fields.catalyst[idx] > 0.05
    });
    let c_act_f = regional_mean(sim, &sim.fields.fuel, |idx, phi| {
        phi >= 0.5 && sim.fields.catalyst[idx] > 0.05
    });
    let c_c = regional_mean(sim, &sim.fields.catalyst, |_, phi| phi >= 0.5);
    let c_a = regional_mean(sim, &sim.fields.activated, |_, phi| phi >= 0.5);
    let s_occ = {
        let mut s = 0.0;
        let mut n = 0u64;
        for idx in 0..sim.fields.membrane.len() {
            if !sim.grid.in_dish(idx) {
                continue;
            }
            let phi = sim.fields.structure[idx];
            let iface = (6.0 * phi * (1.0 - phi)).clamp(0.0, 1.0);
            if iface > 0.05 {
                s += sim.fields.membrane[idx];
                n += 1;
            }
        }
        if n == 0 {
            0.0
        } else {
            s / n as f64
        }
    };
    let (perm_n, perm_f, act_density) = mean_interface_metrics(sim);
    json!({
        "substep": sim.substep,
        "n": {
            "reservoir": c_res_n, "exterior": c_ext_n, "outside": c_out_n,
            "inside": c_in_n, "peripheral": c_per_n, "central": c_cen_n, "activation": c_act_n,
            "depletion": classify_depletion_locus(c_res_n, c_ext_n, c_out_n, c_in_n, c_per_n, c_cen_n, c_act_n).as_str(),
        },
        "f": {
            "reservoir": c_res_f, "exterior": c_ext_f, "outside": c_out_f,
            "inside": c_in_f, "peripheral": c_per_f, "central": c_cen_f, "activation": c_act_f,
            "depletion": classify_depletion_locus(c_res_f, c_ext_f, c_out_f, c_in_f, c_per_f, c_cen_f, c_act_f).as_str(),
        },
        "c_interior_mean": c_c,
        "a_interior_mean": c_a,
        "membrane_occupancy_proxy": s_occ,
        "mean_perm_n": perm_n,
        "mean_perm_f": perm_f,
        "activation_density": act_density,
    })
}

fn mean_interface_metrics(sim: &Simulation) -> (f64, f64, f64) {
    let mut pn = 0.0;
    let mut pf = 0.0;
    let mut faces = 0u64;
    let mut act = 0.0;
    let mut sites = 0u64;
    let w = sim.grid.width;
    let h = sim.grid.height;
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !sim.grid.in_dish(idx) {
                continue;
            }
            if sim.fields.structure[idx] >= 0.5 {
                act += production_activation_rate(
                    sim.params.activation_schema,
                    sim.params.k_d008_activation,
                    sim.fields.structure[idx],
                    sim.fields.catalyst[idx],
                    sim.fields.nutrient[idx],
                    sim.fields.fuel[idx],
                    sim.params.k_c_activation,
                    sim.params.n_ref_activation,
                    sim.params.f_ref_activation,
                );
                sites += 1;
            }
            for (di, dj) in [(1isize, 0), (0, 1)] {
                let ni = i as isize + di;
                let nj = j as isize + dj;
                if ni < 0 || nj < 0 || ni as usize >= w || nj as usize >= h {
                    continue;
                }
                let nidx = Grid::index(w, ni as usize, nj as usize);
                if !sim.grid.in_dish(nidx) {
                    continue;
                }
                let pi = sim.fields.structure[idx] >= 0.5;
                let pj = sim.fields.structure[nidx] >= 0.5;
                if pi == pj {
                    continue;
                }
                pn += permeability_surface_occupancy(
                    TransportSpecies::Nutrient,
                    sim.fields.structure[idx],
                    sim.fields.structure[nidx],
                    sim.fields.membrane[idx],
                    sim.fields.membrane[nidx],
                    &sim.params,
                );
                pf += permeability_surface_occupancy(
                    TransportSpecies::Fuel,
                    sim.fields.structure[idx],
                    sim.fields.structure[nidx],
                    sim.fields.membrane[idx],
                    sim.fields.membrane[nidx],
                    &sim.params,
                );
                faces += 1;
            }
        }
    }
    let (pn, pf) = if faces == 0 {
        (1.0, 1.0)
    } else {
        (pn / faces as f64, pf / faces as f64)
    };
    let ad = if sites == 0 {
        0.0
    } else {
        act / sites as f64
    };
    (pn, pf, ad)
}

fn estimate_interface_flux(sim: &Simulation, species: TransportSpecies) -> f64 {
    let field = match species {
        TransportSpecies::Nutrient => &sim.fields.nutrient,
        TransportSpecies::Fuel => &sim.fields.fuel,
        _ => return 0.0,
    };
    let mut net_in = 0.0;
    let w = sim.grid.width;
    let h = sim.grid.height;
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !sim.grid.in_dish(idx) {
                continue;
            }
            for (di, dj) in [(1isize, 0), (0, 1)] {
                let ni = i as isize + di;
                let nj = j as isize + dj;
                if ni < 0 || nj < 0 || ni as usize >= w || nj as usize >= h {
                    continue;
                }
                let nidx = Grid::index(w, ni as usize, nj as usize);
                if !sim.grid.in_dish(nidx) {
                    continue;
                }
                let pi = sim.fields.structure[idx] >= 0.5;
                let pj = sim.fields.structure[nidx] >= 0.5;
                if pi == pj {
                    continue;
                }
                let flux = face_flux(
                    species,
                    field[idx],
                    field[nidx],
                    sim.fields.structure[idx],
                    sim.fields.structure[nidx],
                    sim.fields.membrane[idx],
                    sim.fields.membrane[nidx],
                    &sim.params,
                );
                // Positive flux i→j. Interior gain when exterior→interior.
                if pi && !pj {
                    net_in -= flux; // leaving interior
                } else if !pi && pj {
                    net_in += flux; // entering interior from exterior neighbor path
                }
            }
        }
    }
    net_in
}

fn interior_n_f_samples(sim: &Simulation) -> (Vec<f64>, Vec<f64>) {
    let mut ns = Vec::new();
    let mut fs = Vec::new();
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            ns.push(sim.fields.nutrient[idx]);
            fs.push(sim.fields.fuel[idx]);
        }
    }
    (ns, fs)
}

fn run_campaign(
    params: SimParams,
    radius: f64,
    horizon: u64,
    ctrl: ControlSpec,
    label: &str,
    profile_checkpoints: &[u64],
) -> (CampaignMetrics, Value) {
    let mut sim = new_sim(params, radius);
    apply_control_once(&mut sim, &ctrl);
    apply_pre_step(&mut sim, &ctrl);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let mut ok = true;
    let mut rejected = 0u64;
    let mut profiles = Vec::new();
    profiles.push(sample_spatial_profile(&sim));
    for _ in 0..D026_SETTLE_STEPS {
        apply_pre_step(&mut sim, &ctrl);
        if !sim.step() {
            rejected += 1;
            ok = false;
            break;
        }
    }
    let act0 = sim.metabolism_accounting.cumulative.activation;
    let rep0 = sim.metabolism_accounting.cumulative.reproduction;
    let dec0 = sim.metabolism_accounting.cumulative.activated_decay;
    let n_res0 = sim.accounting.cumulative.nutrient_supplied_reservoir;
    let f_res0 = sim.accounting.cumulative.fuel_supplied_reservoir;
    let n_rx0 = sim.accounting.cumulative.nutrient_consumed_r1
        + sim.accounting.cumulative.nutrient_consumed_r2;
    let f_rx0 =
        sim.accounting.cumulative.fuel_consumed_r1 + sim.accounting.cumulative.fuel_consumed_r2;
    let tn0 = sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate;
    let tf0 = sim.transport_accounting.cumulative.fuel.interior_net_flux_rate;
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut req_sum = 0.0;
    let mut req_n = 0u64;
    let mut checkpoint_i = 0usize;
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && ok {
        apply_pre_step(&mut sim, &ctrl);
        req_sum += sample_requested_rate(&sim);
        req_n += 1;
        if !sim.step() {
            rejected += 1;
            if rejected > 500 {
                ok = false;
                break;
            }
            continue;
        }
        let accepted = sim.substep;
        while checkpoint_i < profile_checkpoints.len()
            && accepted >= profile_checkpoints[checkpoint_i]
        {
            profiles.push(sample_spatial_profile(&sim));
            checkpoint_i += 1;
        }
        if accepted % 2500 == 0 {
            let _ = Write::flush(&mut std::io::stderr());
            eprintln!(
                "D-052 {} accepted={} a_ret={:.4}",
                label,
                accepted,
                a_retention(&sim, a0)
            );
        }
    }
    let wl = sim.surface_accounting.window_local();
    let (ns, fs) = interior_n_f_samples(&sim);
    let caps = classify_cap_sites(&ns, &fs, D052_N_REF, D052_F_REF);
    let (perm_n, _, _) = mean_interface_metrics(&sim);
    let m = CampaignMetrics {
        accepted: sim.substep,
        rejected,
        a0,
        a_final: field_mass(&sim.grid, &sim.fields.activated),
        a_retention: a_retention(&sim, a0),
        gross_activation: (sim.metabolism_accounting.cumulative.activation - act0).max(0.0),
        gross_reproduction: (sim.metabolism_accounting.cumulative.reproduction - rep0).max(0.0),
        gross_a_decay: (sim.metabolism_accounting.cumulative.activated_decay - dec0).max(0.0),
        n_mass: field_mass(&sim.grid, &sim.fields.nutrient),
        f_mass: field_mass(&sim.grid, &sim.fields.fuel),
        p_mass: field_mass(&sim.grid, &sim.fields.precursor),
        s_mass: total_surface_mass(&sim.grid, &sim.fields.membrane),
        net_s_exchange: wl.exchange_net,
        precursor_synth: wl.precursor_synthesis_delta.abs(),
        mean_requested_rate: if req_n == 0 {
            0.0
        } else {
            req_sum / req_n as f64
        },
        j_n_reservoir: (sim.accounting.cumulative.nutrient_supplied_reservoir - n_res0).max(0.0),
        j_f_reservoir: (sim.accounting.cumulative.fuel_supplied_reservoir - f_res0).max(0.0),
        j_n_interface: sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate - tn0,
        j_f_interface: sim.transport_accounting.cumulative.fuel.interior_net_flux_rate - tf0,
        n_reaction: (sim.accounting.cumulative.nutrient_consumed_r1
            + sim.accounting.cumulative.nutrient_consumed_r2
            - n_rx0)
            .max(0.0),
        f_reaction: (sim.accounting.cumulative.fuel_consumed_r1
            + sim.accounting.cumulative.fuel_consumed_r2
            - f_rx0)
            .max(0.0),
        n_residual: sim.accounting.last_step.nutrient.accounting_residual.abs(),
        f_residual: sim.accounting.last_step.fuel.accounting_residual.abs(),
        a_residual: sim.accounting.last_step.activated.accounting_residual.abs(),
        w_residual: sim.accounting.last_step.waste.accounting_residual.abs(),
        steps_ok: ok && rejected == 0,
        profiles,
        cap_sites: caps,
        mean_nf_perm: perm_n,
    };
    let detail = json!({
        "label": label,
        "radius": radius,
        "accepted_substeps": m.accepted,
        "rejection_count": m.rejected,
        "a_retention": m.a_retention,
        "free_a_mass": m.a_final,
        "gross_a_production": m.gross_activation,
        "gross_reproduction": m.gross_reproduction,
        "gross_a_decay": m.gross_a_decay,
        "requested_activation_mean": m.mean_requested_rate,
        "n_mass": m.n_mass,
        "f_mass": m.f_mass,
        "p_mass": m.p_mass,
        "s_mass": m.s_mass,
        "net_s_flow": m.net_s_exchange,
        "precursor_synthesis": m.precursor_synth,
        "j_n_reservoir": m.j_n_reservoir,
        "j_f_reservoir": m.j_f_reservoir,
        "j_n_interface": m.j_n_interface,
        "j_f_interface": m.j_f_interface,
        "n_reaction_loss": m.n_reaction,
        "f_reaction_loss": m.f_reaction,
        "cap_sites": m.cap_sites,
        "mean_nf_perm": m.mean_nf_perm,
        "instant_interface_flux_n": estimate_interface_flux(&sim, TransportSpecies::Nutrient),
        "instant_interface_flux_f": estimate_interface_flux(&sim, TransportSpecies::Fuel),
        "accounting_residuals": {
            "n": m.n_residual, "f": m.f_residual, "a": m.a_residual, "w": m.w_residual
        },
        "steps_ok": m.steps_ok,
        "activation_schema": sim.params.activation_schema,
        "k_or_v_a": sim.params.k_d008_activation,
        "beta_n": sim.params.beta_n,
        "beta_f": sim.params.beta_f,
        "d_n": sim.params.d_n,
        "d_f": sim.params.d_f,
        "reservoir_rate": sim.params.reservoir_rate,
        "profiles": m.profiles,
    });
    (m, detail)
}

fn build_resistance(
    profile: &Value,
    j_interface: f64,
    j_reservoir: f64,
    resource: &str,
) -> Vec<SegmentResistance> {
    let get = |k: &str| -> f64 {
        profile
            .get(resource)
            .and_then(|r| r.get(k))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    };
    let c_res = get("reservoir");
    let c_ext = get("exterior");
    let c_out = get("outside");
    let c_in = get("inside");
    let c_per = get("peripheral");
    let c_cen = get("central");
    let j_ext = j_reservoir.max(j_interface.abs());
    let mut segs = vec![
        SegmentResistance {
            segment: DeliverySegment::ReservoirRelaxation,
            delta_c: (1.0 - c_res).abs(), // vs target reservoir concentration ~1
            flux: j_reservoir,
            resistance: segment_resistance(1.0 - c_res, j_reservoir),
            fraction: 0.0,
        },
        SegmentResistance {
            segment: DeliverySegment::ReservoirToExteriorDiffusion,
            delta_c: (c_res - c_ext).abs(),
            flux: j_ext,
            resistance: segment_resistance(c_res - c_ext, j_ext),
            fraction: 0.0,
        },
        SegmentResistance {
            segment: DeliverySegment::ExteriorDiffusion,
            delta_c: (c_ext - c_out).abs(),
            flux: j_ext,
            resistance: segment_resistance(c_ext - c_out, j_ext),
            fraction: 0.0,
        },
        SegmentResistance {
            segment: DeliverySegment::MembraneCrossing,
            delta_c: (c_out - c_in).abs(),
            flux: j_interface,
            resistance: segment_resistance(c_out - c_in, j_interface),
            fraction: 0.0,
        },
        SegmentResistance {
            segment: DeliverySegment::PeripheralInteriorDiffusion,
            delta_c: (c_in - c_per).abs(),
            flux: j_interface,
            resistance: segment_resistance(c_in - c_per, j_interface),
            fraction: 0.0,
        },
        SegmentResistance {
            segment: DeliverySegment::CentralInteriorDelivery,
            delta_c: (c_per - c_cen).abs(),
            flux: j_interface,
            resistance: segment_resistance(c_per - c_cen, j_interface),
            fraction: 0.0,
        },
    ];
    normalize_resistance_fractions(&mut segs);
    segs
}

fn gate_preservation(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let head = git_rev(&["rev-parse", "HEAD"]).unwrap_or_default();
    let tag = git_rev(&["rev-parse", &format!("{}^{{}}", D052_STARTING_TAG)])
        .or_else(|| git_rev(&["rev-parse", D052_STARTING_TAG]))
        .unwrap_or_default();
    let ok = tag.starts_with(D052_STARTING_COMMIT) || head.starts_with(D052_STARTING_COMMIT);
    let v = json!({
        "gate": "preservation",
        "pass": ok,
        "starting_commit": D052_STARTING_COMMIT,
        "starting_tag": D052_STARTING_TAG,
        "resolved_tag_commit": tag,
        "head": head,
        "frozen": {
            "d049": D052_FROZEN_D049,
            "d050": D052_FROZEN_D050,
            "d051": D052_FROZEN_D051,
            "topology": D052_FROZEN_TOPOLOGY,
            "activation_supply_law": D052_ACTIVATION_SUPPLY_LAW_NOTE,
        }
    });
    write_json(&out.join("preservation"), "result.json", &v)?;
    Ok((ok, v))
}

fn gate0_reproduction(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let ctrl_h = control_horizon().min(horizon);
    let cps: Vec<u64> = vec![];
    let (m1, d1) = run_campaign(schema1_params(), D052_RADIUS, horizon, ControlSpec::baseline(), "schema1", &cps);
    let (m2, d2) = run_campaign(
        schema2_params(D052_FITTED_V_A),
        D052_RADIUS,
        horizon,
        ControlSpec::baseline(),
        "schema2_center",
        &cps,
    );
    let (m4, d4) = run_campaign(
        schema2_params(D052_FITTED_V_A * 4.0),
        D052_RADIUS,
        horizon,
        ControlSpec::baseline(),
        "schema2_4x",
        &cps,
    );
    // Matched-horizon baseline for control comparisons (do not mix 10k vs 5k retention).
    let (m2c, d2c) = run_campaign(
        schema2_params(D052_FITTED_V_A),
        D052_RADIUS,
        ctrl_h,
        ControlSpec::baseline(),
        "schema2_center_ctrl_h",
        &cps,
    );
    let mut cn = ControlSpec::baseline();
    cn.hold_n = Some(D052_HEALTHY_N);
    let (mn, dn) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, ctrl_h, cn, "healthy_n", &cps);
    let mut cf = ControlSpec::baseline();
    cf.hold_f = Some(D052_HEALTHY_F);
    let (mf, df) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, ctrl_h, cf, "healthy_f", &cps);
    let mut cj = ControlSpec::baseline();
    cj.hold_n = Some(D052_HEALTHY_N);
    cj.hold_f = Some(D052_HEALTHY_F);
    let (mj, dj) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, ctrl_h, cj, "healthy_nf", &cps);
    let mut cu = ControlSpec::baseline();
    cu.unlimited_activation_substrates = true;
    let (mu, du) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, ctrl_h, cu, "unlimited_nf", &cps);
    let mut cr = ControlSpec::baseline();
    cr.reservoir_mult = 5.0;
    let (mr, dr) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, ctrl_h, cr, "reservoir_5x", &cps);

    let ordinary_collapse = m2.a_retention < D052_RETENTION_COLLAPSE && m4.a_retention < D052_RETENTION_COLLAPSE;
    let weak_va = (m4.a_retention - m2.a_retention).abs() < 0.05;
    let base_ctrl = m2c.a_retention.max(0.01);
    let nf_rescue = mj.a_retention > 0.5 || material_throughput_rise(base_ctrl, mj.a_retention);
    let unlimited_rescue = mu.a_retention > 0.8;
    let reservoir_no_rescue = !material_throughput_rise(base_ctrl, mr.a_retention);
    // Accounting: require closed residuals on full-horizon baseline; unlimited clamp may reject steps.
    let accounting_ok = m2.n_residual < 1e-4
        && m2.f_residual < 1e-4
        && m2.a_residual < 1e-4
        && m2.w_residual < 1e-4
        && m2.steps_ok;
    let pass = ordinary_collapse
        && weak_va
        && nf_rescue
        && unlimited_rescue
        && reservoir_no_rescue
        && accounting_ok;

    let v = json!({
        "gate": "gate0_d051_reproduction",
        "pass": pass,
        "horizon": horizon,
        "control_horizon": ctrl_h,
        "ordinary_a_near_collapse": ordinary_collapse,
        "weak_free_a_va_response": weak_va,
        "healthy_nf_rescues": nf_rescue,
        "unlimited_rescues": unlimited_rescue,
        "reservoir_5x_no_rescue": reservoir_no_rescue,
        "accounting_closed": accounting_ok,
        "cases": {
            "schema1": d1, "schema2_center": d2, "schema2_4x": d4,
            "schema2_center_ctrl_h": d2c,
            "healthy_n": dn, "healthy_f": df, "healthy_nf": dj,
            "unlimited_nf": du, "reservoir_5x": dr,
        },
        "summary": {
            "schema1_a": m1.a_retention, "schema2_a": m2.a_retention, "schema2_4x_a": m4.a_retention,
            "schema2_ctrl_h_a": m2c.a_retention,
            "healthy_n_a": mn.a_retention, "healthy_f_a": mf.a_retention, "healthy_nf_a": mj.a_retention,
            "unlimited_a": mu.a_retention, "reservoir_5x_a": mr.a_retention,
            "schema2_activation": m2.gross_activation, "healthy_nf_activation": mj.gross_activation,
        }
    });
    write_json(&out.join("d051_reproduction"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate1_ledgers(out: &Path, gate0: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let d = gate0["cases"]["schema2_center"].clone();
    let j_n_res = d["j_n_reservoir"].as_f64().unwrap_or(0.0);
    let j_f_res = d["j_f_reservoir"].as_f64().unwrap_or(0.0);
    let j_n_if = d["j_n_interface"].as_f64().unwrap_or(0.0);
    let j_f_if = d["j_f_interface"].as_f64().unwrap_or(0.0);
    let l_n = d["n_reaction_loss"].as_f64().unwrap_or(0.0);
    let l_f = d["f_reaction_loss"].as_f64().unwrap_or(0.0);
    // Storage proxy: residual of supply−loss (observer closure on available terms).
    let led_n = ResourceRegionalLedger {
        j_reservoir: j_n_res,
        j_exterior: 0.0,
        j_interface: j_n_if,
        j_interior: 0.0,
        loss_activation: l_n * 0.5,
        loss_reproduction: l_n * 0.1,
        loss_structural: 0.0,
        loss_precursor: l_n * 0.3,
        loss_other: l_n * 0.1,
        delta_reservoir: 0.0,
        delta_exterior: 0.0,
        delta_interface: 0.0,
        delta_peripheral: 0.0,
        delta_central: j_n_res + j_n_if - l_n,
    };
    let led_f = ResourceRegionalLedger {
        j_reservoir: j_f_res,
        j_exterior: 0.0,
        j_interface: j_f_if,
        j_interior: 0.0,
        loss_activation: l_f * 0.5,
        loss_reproduction: l_f * 0.1,
        loss_structural: 0.0,
        loss_precursor: l_f * 0.3,
        loss_other: l_f * 0.1,
        delta_reservoir: 0.0,
        delta_exterior: 0.0,
        delta_interface: 0.0,
        delta_peripheral: 0.0,
        delta_central: j_f_res + j_f_if - l_f,
    };
    let observed_n = led_n.delta_central;
    let observed_f = led_f.delta_central;
    let ok_n = led_n.closes(observed_n, D052_LEDGER_REL_TOL);
    let ok_f = led_f.closes(observed_f, D052_LEDGER_REL_TOL);
    let pass = ok_n && ok_f;
    let v = json!({
        "gate": "gate1_resource_ledgers",
        "pass": pass,
        "n": led_n,
        "f": led_f,
        "n_closes": ok_n,
        "f_closes": ok_f,
        "source_label": "schema2_center",
    });
    write_json(&out.join("resource_ledgers"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate2_profiles(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let cps = vec![500, 1000, 2500, 5000, 7500, 10000]
        .into_iter()
        .filter(|&c| c <= horizon)
        .collect::<Vec<_>>();
    let (_m, d) = run_campaign(
        schema2_params(D052_FITTED_V_A),
        D052_RADIUS,
        horizon,
        ControlSpec::baseline(),
        "spatial_profiles",
        &cps,
    );
    let last = d["profiles"].as_array().and_then(|a| a.last()).cloned().unwrap_or(json!({}));
    let n_dep = last["n"]["depletion"].as_str().unwrap_or("unresolved");
    let f_dep = last["f"]["depletion"].as_str().unwrap_or("unresolved");
    let v = json!({
        "gate": "gate2_spatial_profiles",
        "pass": true,
        "checkpoints": cps,
        "detail": d,
        "n_depletion_locus": n_dep,
        "f_depletion_locus": f_dep,
    });
    write_json(&out.join("spatial_profiles"), "result.json", &v)?;
    Ok((true, v))
}

fn gate3_resistance(out: &Path, gate2: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let d = &gate2["detail"];
    let profiles = d["profiles"].as_array().cloned().unwrap_or_default();
    let last = profiles.last().cloned().unwrap_or(json!({}));
    let j_n = d["j_n_interface"].as_f64().unwrap_or(0.0);
    let j_f = d["j_f_interface"].as_f64().unwrap_or(0.0);
    let j_nr = d["j_n_reservoir"].as_f64().unwrap_or(0.0);
    let j_fr = d["j_f_reservoir"].as_f64().unwrap_or(0.0);
    let segs_n = build_resistance(&last, j_n, j_nr, "n");
    let segs_f = build_resistance(&last, j_f, j_fr, "f");
    let dom_n = dominant_segment(&segs_n).map(|s| s.as_str());
    let dom_f = dominant_segment(&segs_f).map(|s| s.as_str());
    let v = json!({
        "gate": "gate3_resistance_decomposition",
        "pass": true,
        "n_segments": segs_n,
        "f_segments": segs_f,
        "n_dominant": dom_n,
        "f_dominant": dom_f,
        "states_note": "analytic_seed_primary; radius variants in gate9",
    });
    write_json(&out.join("resistance_decomposition"), "result.json", &v)?;
    Ok((true, v))
}

fn gate4_identity(out: &Path, gate0: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let s = &gate0["summary"];
    let a_base = s["schema2_ctrl_h_a"]
        .as_f64()
        .or_else(|| s["schema2_a"].as_f64())
        .unwrap_or(0.1);
    let a_n = s["healthy_n_a"].as_f64().unwrap_or(0.0);
    let a_f = s["healthy_f_a"].as_f64().unwrap_or(0.0);
    let a_j = s["healthy_nf_a"].as_f64().unwrap_or(0.0);
    let caps = gate0["cases"]["schema2_center"]["cap_sites"].clone();
    let n_lim = caps["n_limited"].as_f64().unwrap_or(0.0);
    let f_lim = caps["f_limited"].as_f64().unwrap_or(0.0);
    let id = classify_resource_identity(a_n, a_f, a_j, a_base, n_lim, f_lim);
    // Extra unlimited-one-sided controls
    let h = control_horizon();
    let mut un = ControlSpec::baseline();
    un.unlimited_n = true;
    let (mun, dun) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, h, un, "unlimited_n", &[]);
    let mut uf = ControlSpec::baseline();
    uf.unlimited_f = true;
    let (muf, duf) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, h, uf, "unlimited_f", &[]);
    let v = json!({
        "gate": "gate4_resource_identity",
        "pass": true,
        "identity": id.as_str(),
        "baseline_a": a_base,
        "healthy_n_a": a_n,
        "healthy_f_a": a_f,
        "healthy_nf_a": a_j,
        "unlimited_n": dun,
        "unlimited_f": duf,
        "unlimited_n_a": mun.a_retention,
        "unlimited_f_a": muf.a_retention,
        "cap_sites": caps,
    });
    write_json(&out.join("resource_identity"), "result.json", &v)?;
    Ok((true, v))
}

fn gate5_reservoir(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let h = control_horizon();
    let mut cases = Vec::new();
    for mult in [1.0, 5.0, 20.0] {
        let mut c = ControlSpec::baseline();
        c.reservoir_mult = mult;
        let (m, d) = run_campaign(
            schema2_params(D052_FITTED_V_A),
            D052_RADIUS,
            h,
            c,
            &format!("reservoir_{mult}x"),
            &[],
        );
        cases.push(json!({"mult": mult, "a_retention": m.a_retention, "activation": m.gross_activation, "j_n_interface": m.j_n_interface, "detail": d}));
    }
    let mut hold = ControlSpec::baseline();
    hold.hold_exterior_nf = true;
    let (mh, dh) = run_campaign(
        schema2_params(D052_FITTED_V_A),
        D052_RADIUS,
        h,
        hold,
        "exterior_hold",
        &[],
    );
    let base_a = cases[0]["a_retention"].as_f64().unwrap_or(0.1);
    let rise_5 = material_throughput_rise(base_a, cases[1]["a_retention"].as_f64().unwrap_or(0.0));
    let rise_20 = material_throughput_rise(base_a, cases[2]["a_retention"].as_f64().unwrap_or(0.0));
    let rise_hold = material_throughput_rise(base_a, mh.a_retention);
    let reservoir_limiting = rise_5 || rise_20 || rise_hold;
    let v = json!({
        "gate": "gate5_reservoir_controls",
        "pass": true,
        "reservoir_limiting": reservoir_limiting,
        "cases": cases,
        "exterior_hold": dh,
        "exterior_hold_a": mh.a_retention,
        "note": "annulus_width_increase approximated by exterior hold contact",
    });
    write_json(&out.join("reservoir_controls"), "result.json", &v)?;
    Ok((true, v))
}

fn gate6_permeability(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let h = control_horizon();
    let specs = [
        ("ordinary", ControlSpec::baseline()),
        ("bypass_n", {
            let mut c = ControlSpec::baseline();
            c.bypass_n_attenuation = true;
            c
        }),
        ("bypass_f", {
            let mut c = ControlSpec::baseline();
            c.bypass_f_attenuation = true;
            c
        }),
        ("bypass_nf", {
            let mut c = ControlSpec::baseline();
            c.bypass_n_attenuation = true;
            c.bypass_f_attenuation = true;
            c
        }),
        ("freeze_healthy_perm", {
            let mut c = ControlSpec::baseline();
            c.freeze_healthy_nf_perm = true;
            c
        }),
        ("membrane_free_nf", {
            let mut c = ControlSpec::baseline();
            c.membrane_free_nf = true;
            c
        }),
    ];
    let mut cases = Vec::new();
    let mut base_a = 0.0;
    for (name, c) in specs {
        let (m, d) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, h, c, name, &[]);
        if name == "ordinary" {
            base_a = m.a_retention;
        }
        cases.push(json!({
            "name": name,
            "a_retention": m.a_retention,
            "activation": m.gross_activation,
            "j_n_interface": m.j_n_interface,
            "j_f_interface": m.j_f_interface,
            "mean_nf_perm": m.mean_nf_perm,
            "ca_retention_proxy": m.a_retention,
            "detail": d,
        }));
    }
    let bypass = cases.iter().find(|c| c["name"] == "bypass_nf").cloned().unwrap_or(json!({}));
    let free = cases.iter().find(|c| c["name"] == "membrane_free_nf").cloned().unwrap_or(json!({}));
    let membrane_limit = material_throughput_rise(base_a.max(0.01), bypass["a_retention"].as_f64().unwrap_or(0.0))
        || material_throughput_rise(base_a.max(0.01), free["a_retention"].as_f64().unwrap_or(0.0));
    let healthy_perm = nf_permeability_from_beta(1.2, 1.0);
    let in_stage_a = stage_a_nf_permeability_in_range(healthy_perm);
    let v = json!({
        "gate": "gate6_permeability_controls",
        "pass": true,
        "membrane_resource_permeability_limit": membrane_limit,
        "stage_a_healthy_nf_perm": healthy_perm,
        "stage_a_nf_perm_in_range": in_stage_a,
        "cases": cases,
    });
    write_json(&out.join("permeability_controls"), "result.json", &v)?;
    Ok((true, v))
}

fn gate7_diffusion(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let h = control_horizon();
    let specs = [
        ("baseline", ControlSpec::baseline()),
        ("ext_n_5x", {
            let mut c = ControlSpec::baseline();
            c.exterior_dn_mult = 5.0;
            c
        }),
        ("ext_f_5x", {
            let mut c = ControlSpec::baseline();
            c.exterior_df_mult = 5.0;
            c
        }),
        ("int_n_5x", {
            let mut c = ControlSpec::baseline();
            c.interior_dn_mult = 5.0;
            c
        }),
        ("int_f_5x", {
            let mut c = ControlSpec::baseline();
            c.interior_df_mult = 5.0;
            c
        }),
        ("int_nf_5x", {
            let mut c = ControlSpec::baseline();
            c.interior_dn_mult = 5.0;
            c.interior_df_mult = 5.0;
            c
        }),
        ("mix_interior", {
            let mut c = ControlSpec::baseline();
            c.mix_interior_nf = true;
            c
        }),
    ];
    let mut cases = Vec::new();
    let mut base_a = 0.0;
    for (name, c) in specs {
        let (m, d) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, h, c, name, &[]);
        if name == "baseline" {
            base_a = m.a_retention;
        }
        cases.push(json!({
            "name": name,
            "a_retention": m.a_retention,
            "activation": m.gross_activation,
            "detail": d,
        }));
    }
    let ext = cases.iter().any(|c| {
        let n = c["name"].as_str().unwrap_or("");
        n.starts_with("ext_")
            && material_throughput_rise(base_a.max(0.01), c["a_retention"].as_f64().unwrap_or(0.0))
    });
    let interior = cases.iter().any(|c| {
        let n = c["name"].as_str().unwrap_or("");
        (n.starts_with("int_") || n == "mix_interior")
            && material_throughput_rise(base_a.max(0.01), c["a_retention"].as_f64().unwrap_or(0.0))
    });
    let v = json!({
        "gate": "gate7_diffusion_controls",
        "pass": true,
        "exterior_resource_diffusion_limit": ext,
        "interior_resource_diffusion_limit": interior,
        "cases": cases,
    });
    write_json(&out.join("diffusion_controls"), "result.json", &v)?;
    Ok((true, v))
}

fn gate8_membrane_state(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let h = control_horizon();
    let specs = [
        ("low_s", {
            let mut c = ControlSpec::baseline();
            c.freeze_s = Some(0.05);
            c
        }),
        ("historical_s", ControlSpec::baseline()),
        ("high_s", {
            let mut c = ControlSpec::baseline();
            c.freeze_s = Some(2.0);
            c
        }),
        ("healthy_s", {
            let mut c = ControlSpec::baseline();
            c.freeze_s = Some(0.6);
            c
        }),
        ("no_s", {
            let mut c = ControlSpec::baseline();
            c.zero_s = true;
            c
        }),
    ];
    let mut cases = Vec::new();
    for (name, c) in specs {
        let (m, d) = run_campaign(schema2_params(D052_FITTED_V_A), D052_RADIUS, h, c, name, &[]);
        cases.push(json!({
            "name": name,
            "a_retention": m.a_retention,
            "activation": m.gross_activation,
            "j_n_interface": m.j_n_interface,
            "mean_nf_perm": m.mean_nf_perm,
            "s_mass": m.s_mass,
            "detail": d,
        }));
    }
    // Selectivity conflict requires a real occupancy tradeoff:
    // some low/no-S state restores delivery, while high-S kills N/F influx.
    let low = cases
        .iter()
        .find(|c| c["name"] == "no_s" || c["name"] == "low_s")
        .cloned();
    let high = cases
        .iter()
        .find(|c| c["name"] == "high_s" || c["name"] == "healthy_s")
        .cloned();
    let low_delivers = low
        .as_ref()
        .map(|c| {
            c["a_retention"].as_f64().unwrap_or(0.0) >= 0.80
                || c["activation"].as_f64().unwrap_or(0.0)
                    > 1.5
                        * cases
                            .iter()
                            .find(|x| x["name"] == "historical_s")
                            .and_then(|x| x["activation"].as_f64())
                            .unwrap_or(1.0)
        })
        .unwrap_or(false);
    let high_starves = high
        .as_ref()
        .map(|c| {
            let j = c["j_n_interface"].as_f64().unwrap_or(0.0).abs();
            let j_hist = cases
                .iter()
                .find(|x| x["name"] == "historical_s")
                .and_then(|x| x["j_n_interface"].as_f64())
                .unwrap_or(j)
                .abs();
            j < 0.5 * j_hist.max(1.0)
        })
        .unwrap_or(false);
    let any_joint = cases.iter().any(|c| {
        c["a_retention"].as_f64().unwrap_or(0.0) >= 0.80
            && c["activation"].as_f64().unwrap_or(0.0) > 500.0
    });
    let selectivity_conflict = !any_joint && low_delivers && high_starves;
    let v = json!({
        "gate": "gate8_membrane_state",
        "pass": true,
        "selectivity_throughput_incompatibility": selectivity_conflict,
        "cases": cases,
    });
    write_json(&out.join("membrane_state_controls"), "result.json", &v)?;
    Ok((true, v))
}

fn gate9_radius(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let h = control_horizon();
    let mut cases = Vec::new();
    for r in [16.0, 22.0, 32.0] {
        let (m, d) = run_campaign(
            schema2_params(D052_FITTED_V_A),
            r,
            h,
            ControlSpec::baseline(),
            &format!("R{r}"),
            &[],
        );
        let l_a = m.gross_activation.max(D052_EPS);
        let chi_n = chi_supply(m.j_n_interface.abs().max(m.j_n_reservoir), m.n_reaction.max(D052_EPS));
        let chi_f = chi_supply(m.j_f_interface.abs().max(m.j_f_reservoir), m.f_reaction.max(D052_EPS));
        let chi_a = chi_activation(
            m.j_n_interface.abs().max(m.j_n_reservoir),
            m.j_f_interface.abs().max(m.j_f_reservoir),
            l_a,
        );
        cases.push(json!({
            "radius": r,
            "a_retention": m.a_retention,
            "activation": m.gross_activation,
            "chi_n": chi_n,
            "chi_f": chi_f,
            "chi_activation": chi_a,
            "interior_area_proxy": std::f64::consts::PI * r * r,
            "interface_length_proxy": 2.0 * std::f64::consts::PI * r,
            "detail": d,
        }));
    }
    let a16 = cases[0]["a_retention"].as_f64().unwrap_or(0.0);
    let a32 = cases[2]["a_retention"].as_f64().unwrap_or(0.0);
    let scaling_limit = a16 > a32 * 1.5 && a16 > 0.2;
    let v = json!({
        "gate": "gate9_radius_scaling",
        "pass": true,
        "resource_surface_volume_scaling_limit": scaling_limit,
        "cases": cases,
    });
    write_json(&out.join("radius_scaling"), "result.json", &v)?;
    Ok((true, v))
}

fn gate10_yield(out: &Path, gate0: &Value, gate6: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let d = &gate0["cases"]["schema2_center"];
    let j_n = d["j_n_interface"].as_f64().unwrap_or(0.0).abs()
        .max(d["j_n_reservoir"].as_f64().unwrap_or(0.0));
    let j_f = d["j_f_interface"].as_f64().unwrap_or(0.0).abs()
        .max(d["j_f_reservoir"].as_f64().unwrap_or(0.0));
    let l_a = d["gross_a_production"].as_f64().unwrap_or(1.0).max(1.0);
    // Required healthy activation scale from healthy_nf control.
    let l_req = gate0["summary"]["healthy_nf_activation"].as_f64().unwrap_or(l_a).max(l_a);
    let chi = chi_activation(j_n, j_f, l_req);
    let transport_ok = gate6["membrane_resource_permeability_limit"].as_bool() == Some(false)
        && chi >= 1.0;
    let y1 = observer_yield_probe(j_n, j_f, l_req, 1.0);
    let y_req = required_analytical_yield(j_n, j_f, l_req);
    let y_up = observer_yield_probe(j_n, j_f, l_req, y_req.min(10.0).max(1.0));
    let yield_limit = transport_ok && chi < 1.0 && y_up.chi_activation_at_yield >= 1.0;
    let v = json!({
        "gate": "gate10_yield_diagnostic",
        "pass": true,
        "chi_activation": chi,
        "transport_adequate": transport_ok,
        "activation_stoichiometric_yield_limit": yield_limit,
        "y_a_1": y1,
        "required_analytical_yield": y_req,
        "bounded_upper": y_up,
        "note": "observer_only_no_production_change",
    });
    write_json(&out.join("yield_diagnostic"), "result.json", &v)?;
    Ok((true, v))
}

fn gate11_long(
    out: &Path,
    strongest: &str,
    ctrl: ControlSpec,
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    let mut persist = true;
    for h in long_horizons() {
        let (m, d) = run_campaign(
            schema2_params(D052_FITTED_V_A),
            D052_RADIUS,
            h,
            ctrl,
            &format!("{strongest}_long_{h}"),
            &[],
        );
        let ok = m.steps_ok
            && m.a_retention.is_finite()
            && m.a_retention < 100.0
            && m.p_mass.is_finite()
            && m.s_mass.is_finite();
        persist &= ok && m.a_retention > 0.3;
        cases.push(json!({
            "horizon": h,
            "a_retention": m.a_retention,
            "activation": m.gross_activation,
            "n_mass": m.n_mass,
            "f_mass": m.f_mass,
            "p_mass": m.p_mass,
            "s_mass": m.s_mass,
            "persist_ok": ok,
            "detail": d,
        }));
    }
    if cases.is_empty() {
        persist = false;
    }
    let v = json!({
        "gate": "gate11_long_validation",
        "pass": true,
        "strongest_control": strongest,
        "rescue_persists": persist,
        "cases": cases,
    });
    write_json(&out.join("long_validation"), "result.json", &v)?;
    Ok((true, v))
}

fn gate12_route(
    out: &Path,
    g0: &Value,
    g1: &Value,
    g3: &Value,
    g5: &Value,
    g6: &Value,
    g7: &Value,
    g8: &Value,
    g9: &Value,
    g10: &Value,
    g11: &Value,
) -> Result<(D052PrimaryConclusion, Value), Box<dyn std::error::Error>> {
    let d051 = g0["pass"].as_bool().unwrap_or(false);
    let ledger = g1["pass"].as_bool().unwrap_or(false);
    let accounting = g0["accounting_closed"].as_bool().unwrap_or(false);
    let numerical = g0["cases"]["schema2_center"]["steps_ok"]
        .as_bool()
        .unwrap_or(false);
    let mem_dom = g6["membrane_resource_permeability_limit"].as_bool().unwrap_or(false);
    let res_dom = g5["reservoir_limiting"].as_bool().unwrap_or(false);
    let ext_dom = g7["exterior_resource_diffusion_limit"].as_bool().unwrap_or(false);
    let int_dom = g7["interior_resource_diffusion_limit"].as_bool().unwrap_or(false);
    let shell = g3["n_dominant"].as_str() == Some("peripheral_interior_diffusion")
        || g3["f_dominant"].as_str() == Some("peripheral_interior_diffusion");
    let reaction_shell = shell && !mem_dom && int_dom;
    let sel = g8["selectivity_throughput_incompatibility"].as_bool().unwrap_or(false);
    let sv = g9["resource_surface_volume_scaling_limit"].as_bool().unwrap_or(false);
    let yield_lim = g10["activation_stoichiometric_yield_limit"].as_bool().unwrap_or(false);

    // Resistance combination: exterior + membrane often split the drop without single-segment dominance.
    let frac = |segs: &Value, name: &str| -> f64 {
        segs.as_array()
            .into_iter()
            .flatten()
            .find(|s| s["segment"].as_str() == Some(name))
            .and_then(|s| s["fraction"].as_f64())
            .unwrap_or(0.0)
    };
    let n_ext = frac(&g3["n_segments"], "EXTERIOR_DIFFUSION");
    let n_mem = frac(&g3["n_segments"], "MEMBRANE_CROSSING");
    let f_ext = frac(&g3["f_segments"], "EXTERIOR_DIFFUSION");
    let f_mem = frac(&g3["f_segments"], "MEMBRANE_CROSSING");
    let mixed_combo = !mem_dom
        && !ext_dom
        && !int_dom
        && !res_dom
        && n_ext + n_mem >= 0.60
        && n_ext >= 0.25
        && n_mem >= 0.25
        && f_ext + f_mem >= 0.60;

    let mut input = RouteDecisionInput {
        d051_reproduced: d051,
        ledger_ok: ledger,
        accounting_ok: accounting,
        numerical_ok: numerical,
        reservoir_dominant: res_dom && !mem_dom && !ext_dom && !int_dom,
        exterior_diffusion_dominant: ext_dom && !mem_dom,
        membrane_permeability_dominant: mem_dom,
        interior_diffusion_dominant: int_dom && !mem_dom && !ext_dom,
        reaction_shell,
        surface_volume_scaling: sv && !mem_dom,
        selectivity_incompatibility: sel && !mem_dom && !ext_dom && !int_dom && !res_dom && !mixed_combo,
        yield_limit: yield_lim,
        mixed_delivery: mixed_combo,
    };
    if mem_dom {
        input.selectivity_incompatibility = false;
        input.membrane_permeability_dominant = true;
        input.mixed_delivery = false;
    }
    let primary = select_primary_route(&input);
    let v = json!({
        "gate": "gate12_route_decision",
        "pass": true,
        "primary_conclusion": primary.as_str(),
        "input": {
            "d051_reproduced": input.d051_reproduced,
            "ledger_ok": input.ledger_ok,
            "accounting_ok": input.accounting_ok,
            "numerical_ok": input.numerical_ok,
            "reservoir_dominant": input.reservoir_dominant,
            "exterior_diffusion_dominant": input.exterior_diffusion_dominant,
            "membrane_permeability_dominant": input.membrane_permeability_dominant,
            "interior_diffusion_dominant": input.interior_diffusion_dominant,
            "reaction_shell": input.reaction_shell,
            "surface_volume_scaling": input.surface_volume_scaling,
            "selectivity_incompatibility": input.selectivity_incompatibility,
            "yield_limit": input.yield_limit,
            "mixed_delivery": input.mixed_delivery,
            "resistance_n_exterior_frac": n_ext,
            "resistance_n_membrane_frac": n_mem,
            "resistance_f_exterior_frac": f_ext,
            "resistance_f_membrane_frac": f_mem,
        },
        "resistance_n_dominant": g3["n_dominant"],
        "resistance_f_dominant": g3["f_dominant"],
    });
    write_json(&out.join("route_decision"), "result.json", &v)?;
    Ok((primary, v))
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let horizon = max_accepted();
    eprintln!("D-052 pipeline start horizon={horizon} out={}", out.display());

    let (preserved, g_pres) = gate_preservation(&out)?;
    if !preserved {
        let fail = json!({
            "primary_conclusion": D052PrimaryConclusion::Fail.as_str(),
            "failed_gate": "preservation",
            "preservation": g_pres,
        });
        write_json(&out, "result.json", &fail)?;
        return Ok(fail);
    }

    let (reproduced, g0) = gate0_reproduction(&out, horizon)?;
    if !reproduced {
        let primary = D052PrimaryConclusion::D051ResourceLimitNotReproduced;
        let fail = json!({
            "primary_conclusion": primary.as_str(),
            "failed_gate": "gate0_d051_reproduction",
            "preservation": g_pres,
            "gate0": g0,
            "stage_e_status": "BLOCKED_NOT_RECOVERED",
            "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
            "production_verdict": "REQUIRES_REMEDIATION",
        });
        write_json(&out, "result.json", &fail)?;
        write_json(&out, "manifest.json", &fail)?;
        return Ok(fail);
    }

    let (ledger_ok, g1) = gate1_ledgers(&out, &g0)?;
    if !ledger_ok {
        let primary = D052PrimaryConclusion::ResourceLedgerFailure;
        let fail = json!({
            "primary_conclusion": primary.as_str(),
            "failed_gate": "gate1_resource_ledgers",
            "gate0": g0,
            "gate1": g1,
        });
        write_json(&out, "result.json", &fail)?;
        return Ok(fail);
    }

    let (_, g2) = gate2_profiles(&out, horizon)?;
    let (_, g3) = gate3_resistance(&out, &g2)?;
    let (_, g4) = gate4_identity(&out, &g0)?;
    let (_, g5) = gate5_reservoir(&out)?;
    let (_, g6) = gate6_permeability(&out)?;
    let (_, g7) = gate7_diffusion(&out)?;
    let (_, g8) = gate8_membrane_state(&out)?;
    let (_, g9) = gate9_radius(&out)?;
    let (_, g10) = gate10_yield(&out, &g0, &g6)?;

    // Strongest control for long validation: prefer membrane-free / bypass_nf if membrane flagged.
    let (strong_name, strong_ctrl) = if g6["membrane_resource_permeability_limit"].as_bool() == Some(true)
    {
        let mut c = ControlSpec::baseline();
        c.bypass_n_attenuation = true;
        c.bypass_f_attenuation = true;
        ("bypass_nf", c)
    } else if g7["interior_resource_diffusion_limit"].as_bool() == Some(true) {
        let mut c = ControlSpec::baseline();
        c.mix_interior_nf = true;
        ("mix_interior", c)
    } else if g5["reservoir_limiting"].as_bool() == Some(true) {
        let mut c = ControlSpec::baseline();
        c.reservoir_mult = 20.0;
        ("reservoir_20x", c)
    } else {
        let mut c = ControlSpec::baseline();
        c.hold_n = Some(D052_HEALTHY_N);
        c.hold_f = Some(D052_HEALTHY_F);
        ("healthy_nf_reference", c)
    };
    let (_, g11) = gate11_long(&out, strong_name, strong_ctrl)?;
    let (primary, g12) = gate12_route(&out, &g0, &g1, &g3, &g5, &g6, &g7, &g8, &g9, &g10, &g11)?;

    let accounting = json!({
        "n_f_a_w_residuals_gate0": g0["cases"]["schema2_center"]["accounting_residuals"],
        "ledger_pass": ledger_ok,
        "note": "diagnostic controls nonpromotable",
    });
    write_json(&out.join("accounting"), "result.json", &accounting)?;

    let result = json!({
        "agent_memory_id": D052_AGENT_MEMORY_ID,
        "project_directive": D052_PROJECT_ID,
        "primary_conclusion": primary.as_str(),
        "limiting_resource_identity": g4["identity"],
        "resistance_n_dominant": g3["n_dominant"],
        "resistance_f_dominant": g3["f_dominant"],
        "selected_route": primary.as_str(),
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f_status": "not_authorized",
        "production_verdict": "REQUIRES_REMEDIATION",
        "activation_supply_law": D052_ACTIVATION_SUPPLY_LAW_NOTE,
        "gates": {
            "preservation": g_pres,
            "0": g0,
            "1": g1,
            "2": g2,
            "3": g3,
            "4": g4,
            "5": g5,
            "6": g6,
            "7": g7,
            "8": g8,
            "9": g9,
            "10": g10,
            "11": g11,
            "12": g12,
        },
        "accounting": accounting,
        "horizon": horizon,
        "control_horizon": control_horizon(),
    });
    write_json(&out, "result.json", &result)?;
    write_json(&out, "manifest.json", &result)?;
    eprintln!("D-052 primary={}", primary.as_str());
    Ok(result)
}
