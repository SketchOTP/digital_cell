//! D-008 Stage B fixed-field membrane dynamics.

use crate::config::SimParams;
use crate::grid::Grid;
use crate::operators::diffuse_constant;
use crate::reactions::interface_weight;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MembraneRates {
    pub synthesis: f64,
    pub decay: f64,
    pub detachment: f64,
}

impl MembraneRates {
    pub fn net(self) -> f64 {
        self.synthesis - self.decay - self.detachment
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MembraneEvolutionTotals {
    pub synthesis_delta: f64,
    pub decay_delta: f64,
    pub detachment_delta: f64,
    pub diffusion_delta: f64,
}

#[inline]
pub fn membrane_catalyst_saturation(catalyst: f64, params: &SimParams) -> f64 {
    let catalyst = catalyst.max(0.0);
    catalyst / (params.k_c_membrane + catalyst)
}

#[inline]
pub fn membrane_basis(
    phi: f64,
    catalyst: f64,
    activated: f64,
    membrane: f64,
    params: &SimParams,
) -> f64 {
    activated.max(0.0)
        * membrane_catalyst_saturation(catalyst, params)
        * interface_weight(phi)
        * (1.0 - membrane / params.m_max).max(0.0)
}

#[inline]
pub fn membrane_losses(phi: f64, membrane: f64, params: &SimParams) -> f64 {
    let membrane = membrane.max(0.0);
    params.k_membrane_decay * membrane
        + params.k_membrane_detach * membrane * (1.0 - interface_weight(phi))
}

#[inline]
pub fn membrane_rates(
    phi: f64,
    catalyst: f64,
    activated: f64,
    membrane: f64,
    params: &SimParams,
) -> MembraneRates {
    let membrane = membrane.max(0.0);
    MembraneRates {
        synthesis: params.k_membrane * membrane_basis(phi, catalyst, activated, membrane, params),
        decay: params.k_membrane_decay * membrane,
        detachment: params.k_membrane_detach * membrane * (1.0 - interface_weight(phi)),
    }
}

pub fn membrane_diffusion_rate(
    grid: &Grid,
    membrane: &[f64],
    diffusivity: f64,
    scratch_lap: &mut [f64],
    out_rate: &mut [f64],
) {
    diffuse_constant(grid, membrane, diffusivity, scratch_lap, out_rate);
}

/// Evolves only M from old-state fixed drivers into a caller-owned next buffer.
pub fn evolve_fixed_membrane(
    grid: &Grid,
    phi: &[f64],
    catalyst: &[f64],
    activated: &[f64],
    membrane: &[f64],
    params: &SimParams,
    dt: f64,
    scratch_lap: &mut [f64],
    diffusion_rate: &mut [f64],
    next: &mut [f64],
) -> MembraneEvolutionTotals {
    membrane_diffusion_rate(grid, membrane, params.d_m, scratch_lap, diffusion_rate);
    let mut totals = MembraneEvolutionTotals::default();
    for idx in 0..membrane.len() {
        if !grid.in_dish(idx) {
            next[idx] = 0.0;
            continue;
        }
        let rates = membrane_rates(
            phi[idx],
            catalyst[idx],
            activated[idx],
            membrane[idx],
            params,
        );
        totals.synthesis_delta += rates.synthesis * dt;
        totals.decay_delta += rates.decay * dt;
        totals.detachment_delta += rates.detachment * dt;
        totals.diffusion_delta += diffusion_rate[idx] * dt;
        next[idx] = membrane[idx] + dt * (rates.net() + diffusion_rate[idx]);
    }
    totals
}
