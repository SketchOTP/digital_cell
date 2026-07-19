//! Artificial reaction network R1–R5.

use crate::config::{EquationVersion, SimParams};
use crate::fields::interior_weight;

/// Historical D-001 bulk production (retained as identifier).
pub const EQUATION_VERSION_D001_BULK: EquationVersion = EquationVersion::D001BulkV1;
/// D-003 crowding production.
pub const EQUATION_VERSION_CROWDING: EquationVersion = EquationVersion::D003CrowdingV1;
/// D-006 surface-production / bulk-turnover.
pub const EQUATION_VERSION_SURFACE: EquationVersion = EquationVersion::SurfaceTurnoverV1;
/// D-008 membrane-metabolism scaffold (v1 historical).
pub const EQUATION_VERSION_MEMBRANE_METABOLISM: EquationVersion =
    EquationVersion::MembraneMetabolismV1;
/// D-012 conservative membrane metabolism (scientifically non-equivalent to v1).
pub const EQUATION_VERSION_MEMBRANE_METABOLISM_V2: EquationVersion =
    EquationVersion::MembraneMetabolismV2Conservative;

/// Default active equation version for greenfield sims (D-003 crowding).
pub const EQUATION_VERSION: EquationVersion = EQUATION_VERSION_CROWDING;

#[derive(Debug, Clone, Default)]
pub struct ReactionRates {
    pub r_rep: f64,
    pub r_structure: f64,
    pub r_structure_decay: f64,
    pub r_catalyst_decay: f64,
    pub r_phi: f64,
    pub r_c: f64,
    pub r_n: f64,
    pub r_f: f64,
    pub r_w: f64,
    /// Interface weight I(φ̂) used in surface assembly (0 for bulk kinetics).
    pub interface_weight: f64,
}

#[derive(Debug, Clone)]
pub struct ReactionScratch {
    pub rates: Vec<ReactionRates>,
}

impl ReactionScratch {
    pub fn new(size: usize) -> Self {
        Self {
            rates: vec![ReactionRates::default(); size],
        }
    }
}

/// Legacy D-002 vacancy factor: max(0, 1 − φ)
#[inline]
pub fn structure_availability(phi: f64) -> f64 {
    (1.0 - phi).max(0.0)
}

/// D-003 crowding attenuation: K / (K + max(φ, 0))
#[inline]
pub fn structure_crowding(phi: f64, k_phi: f64) -> f64 {
    let p = phi.max(0.0);
    k_phi / (k_phi + p)
}

pub fn structure_production_factor(phi: f64, params: &SimParams) -> f64 {
    if params.use_legacy_structure_kinetics {
        structure_availability(phi)
    } else {
        structure_crowding(phi, params.k_phi)
    }
}

/// Diagnostic clamp for interface function only — does not mutate φ.
#[inline]
pub fn phase_hat(phi: f64) -> f64 {
    phi.clamp(0.0, 1.0)
}

/// Local interface weight I(φ̂) = 16 φ̂² (1−φ̂)². Peaks at φ̂=0.5; zero at 0 and 1.
#[inline]
pub fn interface_weight(phi: f64) -> f64 {
    let x = phase_hat(phi);
    16.0 * x * x * (1.0 - x) * (1.0 - x)
}

/// Saturating catalyst activation for structural assembly.
#[inline]
pub fn catalyst_activation(c: f64, k_c_structure: f64) -> f64 {
    let k = k_c_structure.max(0.0);
    c / (k + c.max(0.0))
}

pub fn is_surface_turnover(params: &SimParams) -> bool {
    params.equation_version == EQUATION_VERSION_SURFACE
}

pub fn compute_reactions_at(
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    w: f64,
    params: &SimParams,
    enabled: bool,
) -> ReactionRates {
    let _ = w;
    if !enabled {
        return ReactionRates::default();
    }
    match params.equation_version {
        EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative | EquationVersion::MembraneMetabolismV3StructuralScaling | EquationVersion::MembraneMetabolismV4InterfaceProtected | EquationVersion::MembraneMetabolismV5InterfaceAffinity | EquationVersion::MembraneMetabolismV6PrecursorAssembly | EquationVersion::MembraneMetabolismV7SurfaceDensity
            | EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
                | EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly
                | EquationVersion::MembraneMetabolismV10ActivatedIntermediate
                | EquationVersion::MembraneMetabolismV11SurfaceMaturation | EquationVersion::MembraneMetabolismV12MembraneCatalyticAssembly => {
            return ReactionRates::default();
        }
        EquationVersion::D001BulkV1
        | EquationVersion::D003CrowdingV1
        | EquationVersion::SurfaceTurnoverV1 => {}
    }

    let h = interior_weight(phi);
    let r_rep = params.k_rep
        * c
        * n
        * f
        * h
        * (1.0 - c / params.c_max).max(0.0);

    let (r_structure, i_weight) = match params.equation_version {
        EquationVersion::SurfaceTurnoverV1 => {
            let i = interface_weight(phi);
            let act = catalyst_activation(c, params.k_c_structure);
            (params.k_structure_interface * n * f * act * i, i)
        }
        EquationVersion::D001BulkV1 | EquationVersion::D003CrowdingV1 => {
            let g = structure_production_factor(phi, params);
            (params.k_structure * c * n * f * g, 0.0)
        }
        EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative | EquationVersion::MembraneMetabolismV3StructuralScaling | EquationVersion::MembraneMetabolismV4InterfaceProtected | EquationVersion::MembraneMetabolismV5InterfaceAffinity | EquationVersion::MembraneMetabolismV6PrecursorAssembly | EquationVersion::MembraneMetabolismV7SurfaceDensity
            | EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
                | EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly
                | EquationVersion::MembraneMetabolismV10ActivatedIntermediate
                | EquationVersion::MembraneMetabolismV11SurfaceMaturation | EquationVersion::MembraneMetabolismV12MembraneCatalyticAssembly => {
            unreachable!("handled before legacy chemistry")
        }
    };

    let r_structure_decay = params.k_structure_decay * phi.max(0.0);

    let r_catalyst_decay = c
        * (params.k_catalyst_decay_inside * h
            + params.k_catalyst_decay_outside * (1.0 - h));

    let r_phi = r_structure - r_structure_decay;
    let r_c = r_rep - r_catalyst_decay;
    let r_n = -params.alpha_n_rep * r_rep - params.alpha_n_structure * r_structure;
    let r_f = -params.alpha_f_rep * r_rep - params.alpha_f_structure * r_structure;
    let r_w = params.alpha_w_rep * r_rep
        + params.alpha_w_structure * r_structure
        + r_structure_decay
        + r_catalyst_decay
        - params.k_waste_decay * w;

    ReactionRates {
        r_rep,
        r_structure,
        r_structure_decay,
        r_catalyst_decay,
        r_phi,
        r_c,
        r_n,
        r_f,
        r_w,
        interface_weight: i_weight,
    }
}

pub fn compute_all_reactions(
    phi: &[f64],
    c: &[f64],
    n: &[f64],
    f: &[f64],
    w: &[f64],
    params: &SimParams,
    enabled: bool,
    scratch: &mut ReactionScratch,
) {
    for (idx, rate) in scratch.rates.iter_mut().enumerate() {
        *rate = compute_reactions_at(phi[idx], c[idx], n[idx], f[idx], w[idx], params, enabled);
    }
}

/// Zero-dimensional reactor step (no spatial terms).
pub fn reactor_step(
    phi: &mut f64,
    c: &mut f64,
    n: &mut f64,
    f: &mut f64,
    w: &mut f64,
    dt: f64,
    params: &SimParams,
) {
    let rates = compute_reactions_at(*phi, *c, *n, *f, *w, params, true);
    *phi += rates.r_phi * dt;
    *c += rates.r_c * dt;
    *n += rates.r_n * dt;
    *f += rates.r_f * dt;
    *w += rates.r_w * dt;
    *phi = phi.max(0.0);
    *c = c.max(0.0);
    *n = n.max(0.0);
    *f = f.max(0.0);
    *w = w.max(0.0);
}

pub fn catalyst_diffusivity(phi: f64, params: &SimParams) -> f64 {
    let h = interior_weight(phi);
    params.d_c_outside + h * (params.d_c_inside - params.d_c_outside)
}

/// Integrated structural synthesis prefactor B = Σ C N F g(φ) over dish (per unit time snapshot).
pub fn integrated_structure_prefactor(
    phi: &[f64],
    c: &[f64],
    n: &[f64],
    f: &[f64],
    dish_mask: &[bool],
    params: &SimParams,
) -> f64 {
    let mut b = 0.0;
    for idx in 0..phi.len() {
        if !dish_mask[idx] {
            continue;
        }
        match params.equation_version {
            EquationVersion::SurfaceTurnoverV1 => {
                let act = catalyst_activation(c[idx], params.k_c_structure);
                b += n[idx] * f[idx] * act * interface_weight(phi[idx]);
            }
            EquationVersion::D001BulkV1 | EquationVersion::D003CrowdingV1 => {
                let g = structure_production_factor(phi[idx], params);
                b += c[idx] * n[idx] * f[idx] * g;
            }
            EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative | EquationVersion::MembraneMetabolismV3StructuralScaling | EquationVersion::MembraneMetabolismV4InterfaceProtected | EquationVersion::MembraneMetabolismV5InterfaceAffinity | EquationVersion::MembraneMetabolismV6PrecursorAssembly | EquationVersion::MembraneMetabolismV7SurfaceDensity
            | EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
                | EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly
                | EquationVersion::MembraneMetabolismV10ActivatedIntermediate
                | EquationVersion::MembraneMetabolismV11SurfaceMaturation | EquationVersion::MembraneMetabolismV12MembraneCatalyticAssembly => {}
        }
    }
    b
}

/// Integrated catalyst reproduction prefactor.
pub fn integrated_rep_prefactor(
    phi: &[f64],
    c: &[f64],
    n: &[f64],
    f: &[f64],
    dish_mask: &[bool],
    c_max: f64,
) -> f64 {
    let mut b = 0.0;
    for idx in 0..phi.len() {
        if !dish_mask[idx] {
            continue;
        }
        let h = interior_weight(phi[idx]);
        b += c[idx] * n[idx] * f[idx] * h * (1.0 - c[idx] / c_max).max(0.0);
    }
    b
}

/// Fraction of structural assembly occurring where I(φ) ≥ threshold.
pub fn interface_assembly_localization_fraction(
    rates: &[ReactionRates],
    threshold: f64,
) -> f64 {
    let mut total = 0.0;
    let mut iface = 0.0;
    for r in rates {
        total += r.r_structure.max(0.0);
        if r.interface_weight >= threshold {
            iface += r.r_structure.max(0.0);
        }
    }
    if total <= 1e-30 {
        1.0
    } else {
        iface / total
    }
}
