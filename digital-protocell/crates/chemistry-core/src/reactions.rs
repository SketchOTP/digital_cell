//! Artificial reaction network R1–R5.

use crate::config::SimParams;
use crate::fields::interior_weight;

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

    let h = interior_weight(phi);
    let r_rep = params.k_rep
        * c
        * n
        * f
        * h
        * (1.0 - c / params.c_max).max(0.0);

    let r_structure = params.k_structure * c * n * f * (1.0 - phi).max(0.0);

    let r_structure_decay = params.k_structure_decay * phi;

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
    *phi = phi.clamp(0.0, 1.0);
    *c = c.max(0.0);
    *n = n.max(0.0);
    *f = f.max(0.0);
    *w = w.max(0.0);
}

pub fn catalyst_diffusivity(phi: f64, params: &SimParams) -> f64 {
    let h = interior_weight(phi);
    params.d_c_outside + h * (params.d_c_inside - params.d_c_outside)
}
