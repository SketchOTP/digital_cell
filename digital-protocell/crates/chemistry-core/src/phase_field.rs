//! Phase-field free energy and Cahn-Hilliard dynamics.

use crate::config::SimParams;
use crate::fields::interior_weight;
use crate::grid::Grid;

/// Double-well free energy density: f(phi) = A * phi^2 * (1 - phi)^2
#[inline]
pub fn free_energy_density(phi: f64, a: f64) -> f64 {
    a * phi * phi * (1.0 - phi) * (1.0 - phi)
}

/// Chemical potential: mu = 2A*phi*(1-phi)*(1-2*phi) - kappa * laplacian(phi)
#[inline]
pub fn chemical_potential_local(phi: f64, lap_phi: f64, params: &SimParams) -> f64 {
    2.0 * params.a * phi * (1.0 - phi) * (1.0 - 2.0 * phi) - params.kappa * lap_phi
}

/// Compute h(phi) for all cells.
pub fn compute_interior_weights(phi: &[f64], out_h: &mut [f64]) {
    for (o, &p) in out_h.iter_mut().zip(phi.iter()) {
        *o = interior_weight(p);
    }
}

/// Total free energy F = Σ[Aφ²(1-φ)² + 0.5κ|∇φ|²] over dish cells.
pub fn total_free_energy(grid: &Grid, phi: &[f64], params: &SimParams) -> f64 {
    let w = grid.width;
    let h = grid.height;
    let mut total = 0.0;
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let p = phi[idx];
            let bulk = free_energy_density(p, params.a);
            let mut grad_sq = 0.0;
            let inv2dx = 0.5 / crate::config::DX;
            if i + 1 < w {
                let nidx = Grid::index(w, i + 1, j);
                if grid.in_dish(nidx) {
                    grad_sq += ((phi[nidx] - p) * inv2dx).powi(2);
                }
            }
            if j + 1 < h {
                let nidx = Grid::index(w, i, j + 1);
                if grid.in_dish(nidx) {
                    grad_sq += ((phi[nidx] - p) * inv2dx).powi(2);
                }
            }
            total += bulk + 0.5 * params.kappa * grad_sq;
        }
    }
    total
}

/// Structural evolution rate from Cahn-Hilliard: M * laplacian(mu) + R_phi
pub fn structure_rate(lap_mu: f64, r_phi: f64, params: &SimParams, phase_enabled: bool) -> f64 {
    if phase_enabled {
        params.mobility_m * lap_mu + r_phi
    } else {
        r_phi
    }
}
