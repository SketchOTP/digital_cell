//! D-006 planar interface calibration and prescribed-radius diagnostics.

use crate::config::{SimParams, DX};
use crate::reactions::{
    catalyst_activation, interface_weight, is_surface_turnover, phase_hat, ReactionRates,
};

/// Analytic 1D tanh interface φ(n) with φ→0 as n→−∞ and φ→φ_in as n→+∞.
pub fn planar_phase_profile(n: f64, interface_width: f64, phi_in: f64) -> f64 {
    let w = interface_width.max(1e-9);
    let t = (n / w).tanh();
    0.5 * phi_in * (1.0 + t)
}

/// Integrate B_interface = ∫ N F act(C) I(φ) dn across a planar interface.
pub fn integrate_planar_b_interface(
    params: &SimParams,
    phi_in: f64,
    n_conc: f64,
    f_conc: f64,
    c_conc: f64,
    half_span: f64,
    dn: f64,
) -> f64 {
    assert!(is_surface_turnover(params), "planar B requires surface_turnover_v1");
    let mut b = 0.0;
    let mut n = -half_span;
    while n <= half_span {
        let phi = planar_phase_profile(n, params.seed_interface_width, phi_in);
        let act = catalyst_activation(c_conc, params.k_c_structure);
        b += n_conc * f_conc * act * interface_weight(phi) * dn;
        n += dn;
    }
    b
}

/// k_structure_interface from planar estimate for target reference radius scale.
pub fn derive_k_structure_interface(
    k_structure_decay: f64,
    phi_in: f64,
    r_reference: f64,
    b_interface: f64,
) -> f64 {
    (k_structure_decay * phi_in * r_reference) / (2.0 * b_interface.max(1e-30))
}

#[derive(Debug, Clone)]
pub struct PrescribedRadiusPoint {
    pub radius: f64,
    pub integrated_assembly: f64,
    pub integrated_decay: f64,
    pub d_m_phi_dt: f64,
    pub d_r_dt: f64,
}

/// Circular φ disk with diffuse interface; hold uniform dense-phase C,N,F for diagnostics.
pub fn prescribed_circular_rates(
    params: &SimParams,
    radius: f64,
    width: usize,
    height: usize,
    c_in: f64,
    n_in: f64,
    f_in: f64,
) -> PrescribedRadiusPoint {
    let cx = (width as f64) * 0.5;
    let cy = (height as f64) * 0.5;
    let w = params.seed_interface_width.max(1e-9);
    let mut assembly = 0.0;
    let mut decay = 0.0;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            // φ ≈ (1/2)(1 - tanh((r-R)/w))
            let phi = 0.5 * (1.0 - ((r - radius) / w).tanh());
            // Retain catalyst across the diffuse interface band (local chemistry).
            let c = c_in * phase_hat(phi);
            let n = n_in;
            let f = f_in;
            let rates: ReactionRates =
                crate::reactions::compute_reactions_at(phi, c, n, f, 0.0, params, true);
            assembly += rates.r_structure * DX * DX;
            decay += rates.r_structure_decay * DX * DX;
        }
    }
    let d_m = assembly - decay;
    // M ∝ π R² ⇒ dR/dt ≈ dM/dt / (2 π R) for φ_in≈1
    let d_r = d_m / (2.0 * std::f64::consts::PI * radius.max(1e-9));
    PrescribedRadiusPoint {
        radius,
        integrated_assembly: assembly,
        integrated_decay: decay,
        d_m_phi_dt: d_m,
        d_r_dt: d_r,
    }
}

pub fn has_stable_radius_crossing(points: &[PrescribedRadiusPoint]) -> bool {
    if points.len() < 2 {
        return false;
    }
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a.radius.partial_cmp(&b.radius).unwrap());
    let mut saw_pos = false;
    let mut saw_neg_after = false;
    for p in &sorted {
        if p.d_r_dt > 1e-8 {
            saw_pos = true;
        }
        if saw_pos && p.d_r_dt < -1e-8 {
            saw_neg_after = true;
        }
    }
    // also require not all same sign
    let all_pos = sorted.iter().all(|p| p.d_r_dt > 0.0);
    let all_neg = sorted.iter().all(|p| p.d_r_dt < 0.0);
    saw_neg_after && !all_pos && !all_neg
}
