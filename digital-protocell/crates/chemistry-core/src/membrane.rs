//! D-008 Stage B fixed-field membrane dynamics.

use crate::config::{EquationVersion, SimParams};
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
    /// V2: activated consumed by membrane synthesis (extent basis).
    pub activated_reaction_delta: f64,
    /// V2: waste from membrane synthesis/decay/detachment.
    pub waste_reaction_delta: f64,
}

impl MembraneEvolutionTotals {
    pub fn membrane_mass_reaction_delta(&self, params: &SimParams) -> f64 {
        if params.equation_version.is_conservative_membrane_metabolism() {
            params.eta_m * self.synthesis_delta - self.decay_delta - self.detachment_delta
        } else {
            self.synthesis_delta - self.decay_delta - self.detachment_delta
        }
    }
}

/// Per-unit synthesis extent: A → η_M M + (1−η_M) W.
pub fn membrane_synthesis_isolated_delta(extent: f64, eta_m: f64) -> [f64; 7] {
    let mut d = [0.0; 7];
    d[6] = eta_m * extent;
    d[5] = -extent;
    d[4] = (1.0 - eta_m) * extent;
    d
}

/// Per-unit membrane loss extent: M → W (decay or detachment).
pub fn membrane_loss_isolated_delta(extent: f64) -> [f64; 7] {
    let mut d = [0.0; 7];
    d[6] = -extent;
    d[4] = extent;
    d
}

/// Structure production extent: A → η_φ φ + (1−η_φ) W (virtual φ in constrained-radius assay).
pub fn structure_production_isolated_delta(extent: f64, eta_phi: f64) -> [f64; 7] {
    let mut d = [0.0; 7];
    d[0] = eta_phi * extent;
    d[5] = -extent;
    d[4] = (1.0 - eta_phi) * extent;
    d
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

/// Evolves M from old-state fixed drivers; v2 also couples A and W stoichiometrically.
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
    mut activated_next: Option<&mut [f64]>,
    mut waste_next: Option<&mut [f64]>,
) -> MembraneEvolutionTotals {
    membrane_diffusion_rate(grid, membrane, params.d_m, scratch_lap, diffusion_rate);
    let v2 = params.equation_version.is_conservative_membrane_metabolism();
    let eta_m = if v2 { params.eta_m } else { 1.0 };
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
        let syn = rates.synthesis * dt;
        let dec = rates.decay * dt;
        let det = rates.detachment * dt;
        totals.synthesis_delta += syn;
        totals.decay_delta += dec;
        totals.detachment_delta += det;
        totals.diffusion_delta += diffusion_rate[idx] * dt;

        if v2 {
            totals.activated_reaction_delta -= syn;
            totals.waste_reaction_delta += (1.0 - eta_m) * syn + dec + det;
            if let Some(a_next) = activated_next.as_deref_mut() {
                a_next[idx] -= syn;
            }
            if let Some(w_next) = waste_next.as_deref_mut() {
                w_next[idx] += (1.0 - eta_m) * syn + dec + det;
            }
            next[idx] = membrane[idx]
                + dt * (eta_m * rates.synthesis - rates.decay - rates.detachment)
                + diffusion_rate[idx] * dt;
        } else {
            next[idx] = membrane[idx] + dt * (rates.net() + diffusion_rate[idx]);
        }
    }
    totals
}
