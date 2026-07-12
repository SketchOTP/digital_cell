//! Phase-field free energy and Cahn-Hilliard dynamics.

use crate::config::SimParams;
use crate::fields::interior_weight;

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

/// Structural evolution rate from Cahn-Hilliard: M * laplacian(mu) + R_phi
pub fn structure_rate(lap_mu: f64, r_phi: f64, params: &SimParams, phase_enabled: bool) -> f64 {
    if phase_enabled {
        params.mobility_m * lap_mu + r_phi
    } else {
        r_phi
    }
}
