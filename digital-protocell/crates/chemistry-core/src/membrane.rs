//! D-008 Stage B fixed-field membrane dynamics.

use crate::config::{SimParams, DX};
use crate::fields::interior_weight;
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
pub fn membrane_decay_factor(phi: f64, params: &SimParams) -> f64 {
    if params.equation_version.is_interface_protected_membrane() {
        // r_M_decay = k_M_decay × M × [ε_M + (1 − I(φ))]
        params.eps_m + (1.0 - interface_weight(phi))
    } else {
        1.0
    }
}

#[inline]
pub fn membrane_losses(phi: f64, membrane: f64, params: &SimParams) -> f64 {
    let membrane = membrane.max(0.0);
    params.k_membrane_decay * membrane * membrane_decay_factor(phi, params)
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
        decay: params.k_membrane_decay * membrane * membrane_decay_factor(phi, params),
        detachment: params.k_membrane_detach * membrane * (1.0 - interface_weight(phi)),
    }
}

/// Face flux from cell i → j (antisymmetric: J(j→i) = −J(i→j)).
///
/// `J = −D_M · (M_j − M_i) + χ_M · mean(M_i,M_j) · (I_j − I_i)`
#[inline]
pub fn membrane_face_flux(
    m_i: f64,
    m_j: f64,
    i_i: f64,
    i_j: f64,
    d_m: f64,
    chi_m: f64,
) -> f64 {
    let mean_m = 0.5 * (m_i + m_j);
    -d_m * (m_j - m_i) + chi_m * mean_m * (i_j - i_i)
}

/// Conservative membrane transport rate (diffusion ± optional interface affinity).
///
/// When `χ_M = 0`, identical to `D_M · ∇²M` (historical v4 path).
pub fn membrane_transport_rate(
    grid: &Grid,
    membrane: &[f64],
    phi: &[f64],
    params: &SimParams,
    scratch_lap: &mut [f64],
    out_rate: &mut [f64],
) {
    let d_m = params.d_m;
    let chi_m = if params.equation_version.is_interface_affinity_membrane() {
        params.chi_m
    } else {
        0.0
    };
    if chi_m.abs() <= 0.0 {
        diffuse_constant(grid, membrane, d_m, scratch_lap, out_rate);
        return;
    }
    // Face-based assembly: visit each interior edge once (right/down).
    let w = grid.width;
    let h = grid.height;
    let inv_dx2 = 1.0 / (DX * DX);
    out_rate.fill(0.0);
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let m_i = membrane[idx].max(0.0);
            let i_i = interface_weight(phi[idx]);
            // +x face
            if i + 1 < w {
                let jdx = Grid::index(w, i + 1, j);
                if grid.in_dish(jdx) {
                    let m_j = membrane[jdx].max(0.0);
                    let i_j = interface_weight(phi[jdx]);
                    let flux = membrane_face_flux(m_i, m_j, i_i, i_j, d_m, chi_m) * inv_dx2;
                    out_rate[idx] -= flux;
                    out_rate[jdx] += flux;
                }
            }
            // +y face
            if j + 1 < h {
                let jdx = Grid::index(w, i, j + 1);
                if grid.in_dish(jdx) {
                    let m_j = membrane[jdx].max(0.0);
                    let i_j = interface_weight(phi[jdx]);
                    let flux = membrane_face_flux(m_i, m_j, i_i, i_j, d_m, chi_m) * inv_dx2;
                    out_rate[idx] -= flux;
                    out_rate[jdx] += flux;
                }
            }
        }
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

// === D-023 soluble precursor (P) + interface assembly (v6) ===

/// D-023 field index order for eight-field isolated deltas (v6):
/// [structure, catalyst, nutrient, fuel, waste, activated, membrane, precursor].
pub const V6_A_INDEX: usize = 5;
pub const V6_W_INDEX: usize = 4;
pub const V6_M_INDEX: usize = 6;
pub const V6_P_INDEX: usize = 7;

/// Precursor synthesis A → P: `k_precursor · A · q(C) · H(φ)`.
/// Produced where the catalyst-bearing interior exists (interior weight H(φ)).
#[inline]
pub fn precursor_synthesis_rate(
    phi: f64,
    catalyst: f64,
    activated: f64,
    params: &SimParams,
) -> f64 {
    params.k_precursor
        * activated.max(0.0)
        * membrane_catalyst_saturation(catalyst, params)
        * interior_weight(phi)
}

/// Interface assembly P → M: `k_assembly · P · I(φ) · max(0, 1 − M/M_max)`.
/// Assembles only where the structural interface exists (interface weight I(φ)).
#[inline]
pub fn precursor_assembly_rate(phi: f64, precursor: f64, membrane: f64, params: &SimParams) -> f64 {
    params.k_assembly
        * precursor.max(0.0)
        * interface_weight(phi)
        * (1.0 - membrane / params.m_max).max(0.0)
}

/// Precursor turnover P → W: `k_precursor_decay · P`.
#[inline]
pub fn precursor_decay_rate(precursor: f64, params: &SimParams) -> f64 {
    params.k_precursor_decay * precursor.max(0.0)
}

/// Conservative isolated extent A → P (unit yield).
pub fn precursor_synthesis_isolated_delta(extent: f64) -> [f64; 8] {
    let mut d = [0.0; 8];
    d[V6_A_INDEX] = -extent;
    d[V6_P_INDEX] = extent;
    d
}

/// Conservative isolated extent P → M (unit yield).
pub fn precursor_assembly_isolated_delta(extent: f64) -> [f64; 8] {
    let mut d = [0.0; 8];
    d[V6_P_INDEX] = -extent;
    d[V6_M_INDEX] = extent;
    d
}

/// Conservative isolated extent P → W (unit yield).
pub fn precursor_decay_isolated_delta(extent: f64) -> [f64; 8] {
    let mut d = [0.0; 8];
    d[V6_P_INDEX] = -extent;
    d[V6_W_INDEX] = extent;
    d
}

/// Conservative isolated extent M → W (membrane loss).
pub fn membrane_loss_isolated_delta_v6(extent: f64) -> [f64; 8] {
    let mut d = [0.0; 8];
    d[V6_M_INDEX] = -extent;
    d[V6_W_INDEX] = extent;
    d
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PrecursorEvolutionTotals {
    pub synthesis_delta: f64,
    pub assembly_delta: f64,
    pub precursor_decay_delta: f64,
    pub membrane_loss_delta: f64,
    pub membrane_diffusion_delta: f64,
    /// Net A consumed by precursor synthesis (≤ 0).
    pub activated_reaction_delta: f64,
    /// Net P change from reactions only (synthesis − assembly − decay).
    pub precursor_reaction_delta: f64,
    /// Net M change from reactions (assembly − loss).
    pub membrane_reaction_delta: f64,
    /// Waste produced by precursor turnover + membrane loss.
    pub waste_reaction_delta: f64,
}

/// D-023 v6 evolution. Advances M (assembly − loss + diffusion) and writes the
/// per-cell reaction contributions for A (−synthesis), P (synthesis − assembly −
/// decay), and W (decay + loss). Precursor transport is applied by the caller.
///
/// `phi`, `catalyst`, `activated`, `precursor`, `membrane` are current-state drivers.
/// `membrane_next` must already hold current M; diffusion + reaction are added in.
#[allow(clippy::too_many_arguments)]
pub fn evolve_precursor_assembly(
    grid: &Grid,
    phi: &[f64],
    catalyst: &[f64],
    activated: &[f64],
    precursor: &[f64],
    membrane: &[f64],
    params: &SimParams,
    dt: f64,
    scratch_lap: &mut [f64],
    diffusion_rate: &mut [f64],
    membrane_next: &mut [f64],
    activated_next: &mut [f64],
    precursor_next: &mut [f64],
    waste_next: &mut [f64],
) -> PrecursorEvolutionTotals {
    // χ_M = 0 for v6: plain M diffusion.
    membrane_transport_rate(grid, membrane, phi, params, scratch_lap, diffusion_rate);
    let mut totals = PrecursorEvolutionTotals::default();
    for idx in 0..membrane.len() {
        if !grid.in_dish(idx) {
            membrane_next[idx] = 0.0;
            continue;
        }
        let r_syn = precursor_synthesis_rate(phi[idx], catalyst[idx], activated[idx], params);
        let r_asm = precursor_assembly_rate(phi[idx], precursor[idx], membrane[idx], params);
        let r_dec = precursor_decay_rate(precursor[idx], params);
        let loss = membrane_losses(phi[idx], membrane[idx], params);

        let syn = r_syn * dt;
        let asm = r_asm * dt;
        let dec = r_dec * dt;
        let los = loss * dt;
        let diff = diffusion_rate[idx] * dt;

        totals.synthesis_delta += syn;
        totals.assembly_delta += asm;
        totals.precursor_decay_delta += dec;
        totals.membrane_loss_delta += los;
        totals.membrane_diffusion_delta += diff;
        totals.activated_reaction_delta -= syn;
        totals.precursor_reaction_delta += syn - asm - dec;
        totals.membrane_reaction_delta += asm - los;
        totals.waste_reaction_delta += dec + los;

        activated_next[idx] -= syn;
        precursor_next[idx] += syn - asm - dec;
        membrane_next[idx] += (asm - los) + diff;
        waste_next[idx] += dec + los;
    }
    totals
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
    membrane_transport_rate(grid, membrane, phi, params, scratch_lap, diffusion_rate);
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
