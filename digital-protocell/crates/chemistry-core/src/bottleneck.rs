//! Reaction bottleneck and balance diagnostics (D-003).

use crate::fields::interior_weight;
use crate::grid::Grid;
use crate::reactions::{structure_availability, structure_crowding, ReactionScratch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BottleneckDiagnostics {
    pub mean_n_inside: f64,
    pub mean_f_inside: f64,
    pub mean_c_inside: f64,
    pub mean_phi_inside: f64,
    pub min_n_inside: f64,
    pub min_f_inside: f64,
    pub median_n_inside: f64,
    pub median_f_inside: f64,
    pub p10_n_inside: f64,
    pub p10_f_inside: f64,
    pub mean_catalyst_capacity: f64,
    pub old_structure_availability: f64,
    pub availability_dense: f64,
    pub availability_interface: f64,
    pub availability_exterior: f64,
    pub synth_dense: f64,
    pub synth_interface: f64,
    pub synth_exterior: f64,
    pub decay_dense: f64,
    pub decay_interface: f64,
    pub decay_exterior: f64,
    pub catalyst_decay_inside: f64,
    pub catalyst_decay_interface: f64,
    pub catalyst_decay_outside: f64,
    pub fraction_catalyst_outside: f64,
    pub transport_limited: bool,
    pub retention_limited: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceDiagnostics {
    pub s_phi: f64,
    pub d_phi: f64,
    pub r_c: f64,
    pub d_c: f64,
    pub q_phi: f64,
    pub q_c: f64,
    pub slope_phi: f64,
    pub slope_catalyst: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceWindowSample {
    pub sim_time: f64,
    pub m_phi: f64,
    pub m_c: f64,
    pub s_phi: f64,
    pub d_phi: f64,
    pub r_c: f64,
    pub d_c: f64,
}

/// D-003 transport-limitation threshold (§14).
pub fn is_transport_limited(mean_n: f64, mean_f: f64, n_reservoir: f64, f_reservoir: f64) -> bool {
    mean_n < 0.10 * n_reservoir || mean_f < 0.10 * f_reservoir
}

/// D-003 catalyst-retention threshold (§15).
pub fn is_retention_limited(fraction_catalyst_outside: f64, outside_decay_frac: f64) -> bool {
    fraction_catalyst_outside > 0.25 || outside_decay_frac > 0.50
}

pub fn compute_bottleneck(
    grid: &Grid,
    phi: &[f64],
    c: &[f64],
    n: &[f64],
    f: &[f64],
    reaction: &ReactionScratch,
    c_max: f64,
    n_reservoir: f64,
    f_reservoir: f64,
) -> BottleneckDiagnostics {
    let mut h_sum = 0.0;
    let mut n_in = 0.0;
    let mut f_in = 0.0;
    let mut c_in = 0.0;
    let mut phi_in = 0.0;
    let mut cap_sum = 0.0;
    let mut old_avail_sum = 0.0;
    let mut n_vals = Vec::new();
    let mut f_vals = Vec::new();

    let mut synth = [0.0; 3];
    let mut decay = [0.0; 3];
    let mut cdec = [0.0; 3];
    let mut cat_outside_mass = 0.0;
    let mut cat_total = 0.0;

    for idx in 0..grid.width * grid.height {
        if !grid.in_dish(idx) {
            continue;
        }
        let p = phi[idx];
        let h = interior_weight(p);
        h_sum += h;
        n_in += n[idx] * h;
        f_in += f[idx] * h;
        c_in += c[idx] * h;
        phi_in += p * h;
        cap_sum += (1.0 - c[idx] / c_max).max(0.0) * h;
        old_avail_sum += structure_availability(p) * h;
        if h > 0.01 {
            n_vals.push(n[idx]);
            f_vals.push(f[idx]);
        }

        let zone = zone_index(p);
        let r = &reaction.rates[idx];
        synth[zone] += r.r_structure;
        decay[zone] += r.r_structure_decay;
        cdec[zone] += r.r_catalyst_decay;

        cat_total += c[idx];
        if p < 0.5 {
            cat_outside_mass += c[idx];
        }
    }

    n_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    f_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let inv_h = 1.0 / h_sum.max(1e-12);
    let mean_n = n_in * inv_h;
    let mean_f = f_in * inv_h;
    let transport_limited = is_transport_limited(mean_n, mean_f, n_reservoir, f_reservoir);
    let frac_out = cat_outside_mass / cat_total.max(1e-12);
    let outside_decay_frac = (cdec[2] + cdec[1] * 0.5) / cdec.iter().sum::<f64>().max(1e-12);
    let retention_limited = is_retention_limited(frac_out, outside_decay_frac);

    BottleneckDiagnostics {
        mean_n_inside: mean_n,
        mean_f_inside: mean_f,
        mean_c_inside: c_in * inv_h,
        mean_phi_inside: phi_in * inv_h,
        min_n_inside: n_vals.first().copied().unwrap_or(0.0),
        min_f_inside: f_vals.first().copied().unwrap_or(0.0),
        median_n_inside: percentile(&n_vals, 50.0),
        median_f_inside: percentile(&f_vals, 50.0),
        p10_n_inside: percentile(&n_vals, 10.0),
        p10_f_inside: percentile(&f_vals, 10.0),
        mean_catalyst_capacity: cap_sum * inv_h,
        old_structure_availability: old_avail_sum * inv_h,
        availability_dense: zone_mean(phi, |p| structure_crowding(p, 1.0), 0),
        availability_interface: zone_mean(phi, |p| structure_crowding(p, 1.0), 1),
        availability_exterior: zone_mean(phi, |p| structure_crowding(p, 1.0), 2),
        synth_dense: synth[0],
        synth_interface: synth[1],
        synth_exterior: synth[2],
        decay_dense: decay[0],
        decay_interface: decay[1],
        decay_exterior: decay[2],
        catalyst_decay_inside: cdec[0],
        catalyst_decay_interface: cdec[1],
        catalyst_decay_outside: cdec[2],
        fraction_catalyst_outside: frac_out,
        transport_limited,
        retention_limited,
    }
}

pub fn compute_balance(samples: &[BalanceWindowSample]) -> BalanceDiagnostics {
    if samples.is_empty() {
        return BalanceDiagnostics::default();
    }
    let s_phi: f64 = samples.iter().map(|s| s.s_phi).sum();
    let d_phi: f64 = samples.iter().map(|s| s.d_phi).sum();
    let r_c: f64 = samples.iter().map(|s| s.r_c).sum();
    let d_c: f64 = samples.iter().map(|s| s.d_c).sum();
    let eps = 1e-12;
    let q_phi = s_phi / d_phi.max(eps);
    let q_c = r_c / d_c.max(eps);
    let (slope_phi, slope_c) = mass_slopes(samples);
    BalanceDiagnostics {
        s_phi,
        d_phi,
        r_c,
        d_c,
        q_phi,
        q_c,
        slope_phi,
        slope_catalyst: slope_c,
    }
}

fn mass_slopes(samples: &[BalanceWindowSample]) -> (f64, f64) {
    if samples.len() < 2 {
        return (0.0, 0.0);
    }
    let t0 = samples.first().unwrap().sim_time;
    let t1 = samples.last().unwrap().sim_time;
    let dt = (t1 - t0).max(1e-12);
    let mean_phi: f64 = samples.iter().map(|s| s.m_phi).sum::<f64>() / samples.len() as f64;
    let mean_c: f64 = samples.iter().map(|s| s.m_c).sum::<f64>() / samples.len() as f64;
    let mphi0 = samples.first().unwrap().m_phi;
    let mphi1 = samples.last().unwrap().m_phi;
    let mc0 = samples.first().unwrap().m_c;
    let mc1 = samples.last().unwrap().m_c;
    let slope_phi = ((mphi1 - mphi0) / dt) / mean_phi.max(1e-12);
    let slope_c = ((mc1 - mc0) / dt) / mean_c.max(1e-12);
    (slope_phi, slope_c)
}

fn zone_index(phi: f64) -> usize {
    if phi >= 0.75 {
        0
    } else if phi > 0.25 {
        1
    } else {
        2
    }
}

fn zone_mean(phi: &[f64], f: impl Fn(f64) -> f64, zone: usize) -> f64 {
    let mut sum = 0.0;
    let mut n = 0u64;
    for &p in phi {
        let z = zone_index(p);
        if z == zone {
            sum += f(p);
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn balance_window_passes(b: &BalanceDiagnostics) -> bool {
    (0.95..=1.05).contains(&b.q_phi)
        && (0.95..=1.05).contains(&b.q_c)
        && b.slope_phi.abs() <= 1e-4
        && b.slope_catalyst.abs() <= 1e-4
}

/// Shared balance-window runner for calibration and screening (D-004 metric parity).
pub struct BalanceWindowResult {
    pub balance: BalanceDiagnostics,
    pub samples: Vec<BalanceWindowSample>,
    pub substeps: u64,
    pub sim_time: f64,
    pub final_m_phi: f64,
    pub final_m_c: f64,
}

pub fn run_balance_window(sim: &mut crate::simulation::Simulation, substeps: u64) -> BalanceWindowResult {
    let mut samples = Vec::with_capacity(substeps as usize);
    for _ in 0..substeps {
        if !sim.step() {
            break;
        }
        crate::reactions::compute_all_reactions(
            &sim.fields.structure,
            &sim.fields.catalyst,
            &sim.fields.nutrient,
            &sim.fields.fuel,
            &sim.fields.waste,
            &sim.params,
            true,
            &mut sim.reaction_scratch,
        );
        let s_phi: f64 = sim.reaction_scratch.rates.iter().map(|r| r.r_structure).sum();
        let d_phi: f64 = sim
            .reaction_scratch
            .rates
            .iter()
            .map(|r| r.r_structure_decay)
            .sum();
        let r_c: f64 = sim.reaction_scratch.rates.iter().map(|r| r.r_rep).sum();
        let d_c: f64 = sim
            .reaction_scratch
            .rates
            .iter()
            .map(|r| r.r_catalyst_decay)
            .sum();
        samples.push(BalanceWindowSample {
            sim_time: sim.sim_time,
            m_phi: crate::operators::total_mass(&sim.grid, &sim.fields.structure),
            m_c: crate::operators::total_mass(&sim.grid, &sim.fields.catalyst),
            s_phi,
            d_phi,
            r_c,
            d_c,
        });
    }
    let balance = compute_balance(&samples);
    let final_m_phi = samples.last().map(|s| s.m_phi).unwrap_or(0.0);
    let final_m_c = samples.last().map(|s| s.m_c).unwrap_or(0.0);
    BalanceWindowResult {
        balance,
        samples,
        substeps: sim.substep,
        sim_time: sim.sim_time,
        final_m_phi,
        final_m_c,
    }
}

pub const BALANCE_WINDOW_SUBSTEPS: u64 = 20_000;
