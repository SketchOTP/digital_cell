//! Viability-basin mapping and macrostate flow (D-005).

use crate::attractor::TrajectoryPoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasinOutcome {
    RapidCollapse,
    SlowDecline,
    NearBalance,
    Growth,
    UnboundedGrowth,
    Fragmentation,
    NumericalFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacrostateVelocity {
    pub radius: f64,
    pub mean_c_inside: f64,
    pub v_r: f64,
    pub v_c: f64,
}

/// Robust linear regression slope: y vs sim_time over tail fraction.
pub fn robust_regression_slope(times: &[f64], values: &[f64], tail_fraction: f64) -> f64 {
    if times.len() < 2 || times.len() != values.len() {
        return 0.0;
    }
    let n = times.len();
    let start = ((1.0 - tail_fraction) * n as f64).floor() as usize;
    let start = start.min(n.saturating_sub(2));
    let (t, v): (Vec<_>, Vec<_>) = times[start..]
        .iter()
        .zip(values[start..].iter())
        .unzip();
    if t.len() < 2 {
        return 0.0;
    }
    let t_mean = t.iter().sum::<f64>() / t.len() as f64;
    let v_mean = v.iter().sum::<f64>() / v.len() as f64;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..t.len() {
        let dt = t[i] - t_mean;
        num += dt * (v[i] - v_mean);
        den += dt * dt;
    }
    if den.abs() < 1e-30 {
        0.0
    } else {
        num / den
    }
}

pub fn macrostate_velocity_from_trajectory(
    trajectory: &[TrajectoryPoint],
    tail_fraction: f64,
) -> Option<MacrostateVelocity> {
    if trajectory.is_empty() {
        return None;
    }
    let times: Vec<f64> = trajectory.iter().map(|p| p.sim_time).collect();
    let radii: Vec<f64> = trajectory.iter().map(|p| p.equivalent_radius).collect();
    let mean_c: Vec<f64> = trajectory
        .iter()
        .map(|p| if p.m_phi > 1e-9 { p.m_c / p.m_phi } else { 0.0 })
        .collect();
    let last = trajectory.last()?;
    Some(MacrostateVelocity {
        radius: last.equivalent_radius,
        mean_c_inside: last.m_c / last.m_phi.max(1e-9),
        v_r: robust_regression_slope(&times, &radii, tail_fraction),
        v_c: robust_regression_slope(&times, &mean_c, tail_fraction),
    })
}

pub fn classify_basin_outcome(
    initial_radius: f64,
    final_radius: f64,
    q_phi: f64,
    slope_phi: f64,
    retention: f64,
    connected_frac: f64,
    rejection_count: u64,
) -> BasinOutcome {
    if rejection_count > 100 {
        return BasinOutcome::NumericalFailure;
    }
    if connected_frac < 0.5 && final_radius > 5.0 {
        return BasinOutcome::Fragmentation;
    }
    if final_radius > 80.0 || q_phi > 2.0 {
        return BasinOutcome::UnboundedGrowth;
    }
    if final_radius < initial_radius * 0.3 && q_phi < 0.5 {
        return BasinOutcome::RapidCollapse;
    }
    if slope_phi.abs() <= 1e-4 && (0.98..=1.02).contains(&q_phi) && retention >= 0.8 {
        return BasinOutcome::NearBalance;
    }
    if slope_phi > 1e-4 && final_radius > initial_radius * 1.05 {
        return BasinOutcome::Growth;
    }
    if slope_phi < -1e-4 {
        return BasinOutcome::SlowDecline;
    }
    if final_radius > initial_radius * 1.1 {
        return BasinOutcome::Growth;
    }
    BasinOutcome::SlowDecline
}

/// Basin requires ≥3 neighboring (R,C) points each passing 4/5 seeds.
pub fn basin_requires_neighboring_points(pass_grid: &[Vec<bool>]) -> bool {
    if pass_grid.is_empty() || pass_grid[0].is_empty() {
        return false;
    }
    let rows = pass_grid.len();
    let cols = pass_grid[0].len();
    for i in 0..rows {
        for j in 0..cols {
            if !pass_grid[i][j] {
                continue;
            }
            let mut cluster = 0usize;
            for (di, dj) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                let ni = i as i32 + di;
                let nj = j as i32 + dj;
                if ni >= 0 && nj >= 0 && (ni as usize) < rows && (nj as usize) < cols {
                    if pass_grid[ni as usize][nj as usize] {
                        cluster += 1;
                    }
                }
            }
            if cluster >= 3 {
                return true;
            }
        }
    }
    false
}

pub fn seeds_pass_fraction(pass_count: u32, total: u32) -> bool {
    pass_count >= 4 && total >= 5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateCorrection {
    pub k_structure_factor: f64,
    pub k_rep_factor: f64,
}

pub fn bounded_rate_correction(median_q_phi: f64, median_q_c: f64) -> RateCorrection {
    let clamp = |f: f64| f.clamp(0.85, 1.15);
    RateCorrection {
        k_structure_factor: clamp((1.0 / median_q_phi.max(1e-9)).powf(0.25)),
        k_rep_factor: clamp((1.0 / median_q_c.max(1e-9)).powf(0.25)),
    }
}

pub fn apply_rate_correction(params: &mut crate::config::SimParams, corr: &RateCorrection) {
    params.k_structure *= corr.k_structure_factor;
    params.k_rep *= corr.k_rep_factor;
}
