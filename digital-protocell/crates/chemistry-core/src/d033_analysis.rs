//! D-033 activated membrane intermediate: orthogonal rate ID and helpers.

use crate::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator};
use crate::d029_analysis::{apply_exchange_candidate, ExchangeCandidate};
use crate::d031_analysis::{d030_identified_candidate, D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::fields::interior_weight;
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    apply_activated_intermediate_bounded, apply_charge_bounded, apply_insert_bounded,
    apply_relax_bounded, charge_rate, insert_rate, relax_rate, surface_occupancy_theta,
};
use serde::{Deserialize, Serialize};

pub use crate::d031_analysis::{D031_ALPHA_FROZEN as D033_ALPHA_FROZEN, D031_BETA_FROZEN as D033_BETA_FROZEN};

/// Normalized estimate tolerance for charge/insert (Gate 2).
pub const D033_RATE_TOL_15: f64 = 0.15;
/// Normalized estimate tolerance for relaxation (Gate 2).
pub const D033_RATE_TOL_10: f64 = 0.10;

pub const PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT: &str =
    "PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT_FOR_MEMBRANE_MAINTENANCE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateCandidate {
    pub identity: String,
    pub k_charge: f64,
    pub k_insert: f64,
    pub k_relax: f64,
}

/// Frozen D-030/D-031 exchange candidate.
pub fn frozen_exchange_candidate() -> ExchangeCandidate {
    d030_identified_candidate()
}

/// Build v10 params with frozen passive exchange and chosen intermediate rates.
pub fn v10_params(k_charge: f64, k_insert: f64, k_relax: f64) -> SimParams {
    let mut p = SimParams::default();
    apply_exchange_candidate(&mut p, &frozen_exchange_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV10ActivatedIntermediate;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = 0.0;
    p.k_charge = k_charge;
    p.k_insert = k_insert;
    p.k_relax = k_relax;
    p.d_x = p.d_p;
    p.reactions_enabled = true;
    p
}

/// Frozen α, β identity check.
pub fn frozen_exchange_kinetics_ok() -> bool {
    let c = frozen_exchange_candidate();
    let alpha = c.k_exchange * c.k_exchange_eq;
    let beta = c.k_exchange;
    ((alpha - D031_ALPHA_FROZEN) / D031_ALPHA_FROZEN).abs() < 1e-12
        && ((beta - D031_BETA_FROZEN) / D031_BETA_FROZEN).abs() < 1e-12
}

/// Material residual of one bounded intermediate step: ΔP+ΔA+ΔX+ΔS+ΔW.
pub fn intermediate_material_residual(
    phi: f64,
    catalyst: f64,
    p0: f64,
    a0: f64,
    x0: f64,
    s0: f64,
    w0: f64,
    delta: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64) {
    let (p1, a1, x1, s1, dw, r_c, r_i, r_r) = apply_activated_intermediate_bounded(
        phi, catalyst, p0, a0, x0, s0, delta, dt, params,
    );
    let w1 = w0 + dw;
    let residual = (p1 - p0) + (a1 - a0) + (x1 - x0) + (s1 - s0) + (w1 - w0);
    (residual, r_c, r_i, r_r)
}

/// Activation-potential residual: production − work − dissipation − Δstorage.
pub fn activation_accounting_residual(
    r_charge: f64,
    r_insert: f64,
    r_relax: f64,
    x0: f64,
    x1: f64,
) -> f64 {
    let production = r_charge;
    let work = r_insert;
    let dissipation = r_relax;
    let storage_delta = x1 - x0;
    production - work - dissipation - storage_delta
}

fn median_sorted(vals: &mut [f64]) -> f64 {
    if vals.is_empty() {
        return f64::NAN;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    if n % 2 == 1 {
        vals[n / 2]
    } else {
        0.5 * (vals[n / 2 - 1] + vals[n / 2])
    }
}

/// Recover `k_charge` from initial X production with insertion/relaxation disabled.
pub fn estimate_k_charge_from_dx(
    true_k: f64,
    phi: f64,
    catalyst: f64,
    p0: f64,
    a0: f64,
    dt: f64,
) -> f64 {
    let mut params = v10_params(true_k, 0.0, 0.0);
    params.k_exchange = 0.0;
    params.k_gamma_decay = 0.0;
    let x0 = 0.0;
    let (_, _, x1, _, r) = apply_charge_bounded(phi, catalyst, p0, a0, x0, dt, &params);
    let h = interior_weight(phi);
    let q = membrane_catalyst_saturation(catalyst, &params);
    let denom = h * q * p0.max(0.0) * a0.max(0.0) * dt;
    if denom <= 1e-30 {
        return f64::NAN;
    }
    // Prefer extent; fall back to ΔX.
    let dx = (x1 - x0).max(r);
    dx / denom
}

/// Recover `k_insert` from X loss / S gain with charging and relaxation disabled.
pub fn estimate_k_insert_from_transfer(
    true_k: f64,
    x0: f64,
    s0: f64,
    delta: f64,
    dt: f64,
    gamma_max: f64,
) -> f64 {
    let mut params = v10_params(0.0, true_k, 0.0);
    params.gamma_max = gamma_max;
    params.k_exchange = 0.0;
    params.k_gamma_decay = 0.0;
    let (x1, s1, r) = apply_insert_bounded(x0, s0, delta, dt, &params);
    let gamma = s0 / delta.max(params.delta_floor);
    let theta = surface_occupancy_theta(gamma, params.gamma_max);
    let denom = delta * x0.max(0.0) * (1.0 - theta).max(0.0) * dt;
    if denom <= 1e-30 {
        return f64::NAN;
    }
    let extent = r.max(s1 - s0).max(x0 - x1);
    extent / denom
}

/// Recover `k_relax` from X→P with charging and insertion disabled.
pub fn estimate_k_relax_from_trajectory(true_k: f64, x0: f64, dt: f64) -> f64 {
    let mut params = v10_params(0.0, 0.0, true_k);
    params.k_exchange = 0.0;
    let (x1, _, r) = apply_relax_bounded(x0, 0.0, dt, &params);
    let denom = x0.max(0.0) * dt;
    if denom <= 1e-30 {
        return f64::NAN;
    }
    r.max(x0 - x1) / denom
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrthogonalRateId {
    pub k_charge_true: f64,
    pub k_insert_true: f64,
    pub k_relax_true: f64,
    pub k_charge_estimates: Vec<f64>,
    pub k_insert_estimates: Vec<f64>,
    pub k_relax_estimates: Vec<f64>,
    pub k_charge_median: f64,
    pub k_insert_median: f64,
    pub k_relax_median: f64,
    pub charge_ok: bool,
    pub insert_ok: bool,
    pub relax_ok: bool,
    pub identifiable: bool,
    pub conclusion: String,
}

/// Gate 2: identify each rate independently via robust medians.
pub fn identify_orthogonal_rates(
    k_charge: f64,
    k_insert: f64,
    k_relax: f64,
) -> OrthogonalRateId {
    // Charge assay: multiple P, A, q(C) levels.
    let mut charge_ests = Vec::new();
    for &(p, a, c) in &[
        (0.5, 0.5, 0.4),
        (1.0, 0.5, 0.4),
        (0.5, 1.0, 0.4),
        (0.8, 0.8, 0.2),
        (0.3, 0.7, 0.6),
    ] {
        let est = estimate_k_charge_from_dx(k_charge, 1.0, c, p, a, 1e-3);
        if est.is_finite() {
            charge_ests.push(est);
        }
    }
    // Insertion assay: multiple θ.
    let mut insert_ests = Vec::new();
    let delta = 0.5_f64;
    let gmax = 1.0_f64;
    for &theta in &[0.1, 0.3, 0.5, 0.7] {
        let s0 = theta * delta * gmax;
        let est = estimate_k_insert_from_transfer(k_insert, 0.4, s0, delta, 1e-3, gmax);
        if est.is_finite() {
            insert_ests.push(est);
        }
    }
    // Relaxation assay.
    let mut relax_ests = Vec::new();
    for &x0 in &[0.2, 0.5, 1.0, 0.05] {
        let est = estimate_k_relax_from_trajectory(k_relax, x0, 1e-3);
        if est.is_finite() {
            relax_ests.push(est);
        }
    }

    let mut c_sorted = charge_ests.clone();
    let mut i_sorted = insert_ests.clone();
    let mut r_sorted = relax_ests.clone();
    let c_med = median_sorted(&mut c_sorted);
    let i_med = median_sorted(&mut i_sorted);
    let r_med = median_sorted(&mut r_sorted);

    let charge_ok = c_med.is_finite()
        && k_charge > 0.0
        && ((c_med - k_charge).abs() / k_charge) <= D033_RATE_TOL_15;
    let insert_ok = i_med.is_finite()
        && k_insert > 0.0
        && ((i_med - k_insert).abs() / k_insert) <= D033_RATE_TOL_15;
    let relax_ok = r_med.is_finite()
        && k_relax > 0.0
        && ((r_med - k_relax).abs() / k_relax) <= D033_RATE_TOL_10;
    let identifiable = charge_ok && insert_ok && relax_ok;
    OrthogonalRateId {
        k_charge_true: k_charge,
        k_insert_true: k_insert,
        k_relax_true: k_relax,
        k_charge_estimates: charge_ests,
        k_insert_estimates: insert_ests,
        k_relax_estimates: relax_ests,
        k_charge_median: c_med,
        k_insert_median: i_med,
        k_relax_median: r_med,
        charge_ok,
        insert_ok,
        relax_ok,
        identifiable,
        conclusion: if identifiable {
            "D033_INTERMEDIATE_KINETICS_IDENTIFIABLE".into()
        } else {
            "D033_INTERMEDIATE_KINETICS_NOT_IDENTIFIABLE".into()
        },
    }
}

/// Continuity helpers for rate positivity under v10.
pub fn charge_zero_without_p_or_a_or_q(params: &SimParams) -> bool {
    charge_rate(1.0, 0.4, 0.0, 1.0, params) == 0.0
        && charge_rate(1.0, 0.4, 1.0, 0.0, params) == 0.0
        && charge_rate(1.0, 0.0, 1.0, 1.0, params) == 0.0
        && charge_rate(1.0, 0.4, 1.0, 1.0, params) > 0.0
}

pub fn insert_zero_without_x_or_capacity(params: &SimParams) -> bool {
    let d = 0.5;
    insert_rate(0.0, 0.1, d, params) == 0.0
        && insert_rate(1.0, d * params.gamma_max, d, params) == 0.0
        && insert_rate(1.0, 0.1, d, params) > 0.0
}

pub fn relax_returns_x_to_p(params: &SimParams) -> bool {
    let (x1, p1, r) = apply_relax_bounded(0.5, 0.1, 0.01, params);
    r > 0.0 && (x1 + p1 - 0.6).abs() < 1e-12 && (0.5 - x1 - r).abs() < 1e-12
}

/// Build candidate identity string components for hashing checks.
pub fn make_candidate(k_charge: f64, k_insert: f64, k_relax: f64) -> IntermediateCandidate {
    IntermediateCandidate {
        identity: format!("k_c={k_charge:.6e}_k_i={k_insert:.6e}_k_r={k_relax:.6e}"),
        k_charge,
        k_insert,
        k_relax,
    }
}

/// Instantaneous rate probes (used by Gate 2 diagnostics).
pub fn probe_rates(
    phi: f64,
    catalyst: f64,
    p: f64,
    a: f64,
    x: f64,
    s: f64,
    delta: f64,
    params: &SimParams,
) -> (f64, f64, f64) {
    (
        charge_rate(phi, catalyst, p, a, params),
        insert_rate(x, s, delta, params),
        relax_rate(x, params),
    )
}
