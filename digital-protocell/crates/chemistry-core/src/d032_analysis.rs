//! D-032 metabolically activated surface assembly: rate reconstruction and candidates.

use crate::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator};
use crate::d029_analysis::{apply_exchange_candidate, ExchangeCandidate};
use crate::d031_analysis::{d030_identified_candidate, D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    active_assembly_basis_density, active_assembly_rate_j, apply_active_assembly_bounded,
    classify_exchange_invariant_field, compute_interface_geometry, exchange_rate_j,
    reconstruct_gamma, surface_occupancy_theta, InterfaceGeometryCell,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

pub use crate::d031_analysis::{D031_ALPHA_FROZEN as D032_ALPHA_FROZEN, D031_BETA_FROZEN as D032_BETA_FROZEN};

/// Analytical candidate scales around the robust median `k_active_required`.
pub const D032_CANDIDATE_SCALES: [f64; 3] = [0.5, 1.0, 2.0];
pub const D032_MAX_ACTIVE_CANDIDATES: usize = 5;
/// Portability: valid k_active_required estimates must span ≤ this factor.
pub const D032_PORTABILITY_SPAN_MAX: f64 = 3.0;
/// Leave-one-out medians must stay within this relative band of the full median.
pub const D032_LOO_MEDIAN_REL_MAX: f64 = 0.50;
pub const D032_MIN_VALID_STATES: usize = 5;
pub const D032_UNDERFLOW_EPS: f64 = 1e-18;

pub const PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT: &str =
    "PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT_FOR_MEMBRANE_MAINTENANCE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRateEstimate {
    pub state_id: String,
    pub accepted_steps: u64,
    pub biological_turnover: f64,
    pub passive_net_exchange: f64,
    pub r_required: f64,
    pub b_active: f64,
    pub k_active_required: f64,
    pub valid: bool,
    pub reject_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRateReconstruction {
    pub estimates: Vec<ActiveRateEstimate>,
    pub valid_count: usize,
    pub median_k_active: f64,
    pub span_factor: f64,
    pub loo_medians: Vec<f64>,
    pub loo_ok: bool,
    pub portable: bool,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCandidate {
    pub identity: String,
    pub k_active: f64,
    pub scale: f64,
}

/// Frozen D-030/D-031 exchange candidate.
pub fn frozen_exchange_candidate() -> ExchangeCandidate {
    d030_identified_candidate()
}

/// Build v9 params with frozen passive exchange and chosen `k_active`.
pub fn v9_params(k_active: f64) -> SimParams {
    let mut p = SimParams::default();
    apply_exchange_candidate(&mut p, &frozen_exchange_candidate());
    // apply_exchange_candidate forces v8; restore v9 identity afterward.
    p.equation_version = EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = k_active;
    p.reactions_enabled = true;
    p
}

/// Passive-only v8 reproduction (k_active = 0 on v9, or pure v8).
pub fn v8_passive_only_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    apply_exchange_candidate(&mut p, &frozen_exchange_candidate());
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.k_active = 0.0;
    p.reactions_enabled = true;
    p
}

/// Integrate active-assembly basis `B_active = ∫ δ q(C) a p (1−θ) dV`.
pub fn integrate_active_basis(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut b = 0.0;
    let dx2 = crate::config::DX * crate::config::DX;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let g = reconstruct_gamma(sim.fields.membrane[idx], d, sim.params.delta_floor);
        let basis = active_assembly_basis_density(
            sim.fields.precursor[idx],
            sim.fields.activated[idx],
            sim.fields.catalyst[idx],
            g,
            &sim.params,
        );
        b += d * basis * dx2;
    }
    b
}

/// Instantaneous passive net exchange rate ∫ δ J_passive dV and turnover ∫ δ k_Γ Γ dV.
pub fn integrate_passive_and_turnover(sim: &Simulation) -> (f64, f64) {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut net = 0.0;
    let mut turn = 0.0;
    let dx2 = crate::config::DX * crate::config::DX;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let g = reconstruct_gamma(sim.fields.membrane[idx], d, sim.params.delta_floor);
        let (j_net, _, _, _, _) =
            exchange_rate_j(sim.fields.precursor[idx], sim.fields.catalyst[idx], g, &sim.params);
        net += d * j_net * dx2;
        turn += d * sim.params.k_gamma_decay * g.max(0.0) * dx2;
    }
    (net, turn)
}

/// Estimate `k_active_required = R_required / B_active` with `R = turnover − passive_net`.
pub fn estimate_k_active_required(
    state_id: &str,
    accepted_steps: u64,
    sim: &Simulation,
) -> ActiveRateEstimate {
    let (passive_net, turnover) = integrate_passive_and_turnover(sim);
    let r_required = turnover - passive_net;
    let b_active = integrate_active_basis(sim);
    let mut valid = true;
    let mut reject = String::new();
    if !(r_required > 0.0 && r_required.is_finite()) {
        valid = false;
        reject = "r_required_nonpositive".into();
    } else if !(b_active > D032_UNDERFLOW_EPS && b_active.is_finite()) {
        valid = false;
        reject = "b_active_underflow".into();
    }
    let k = if valid { r_required / b_active } else { f64::NAN };
    if valid && !(k.is_finite() && k > 0.0) {
        valid = false;
        reject = "k_nonfinite".into();
    }
    ActiveRateEstimate {
        state_id: state_id.into(),
        accepted_steps,
        biological_turnover: turnover,
        passive_net_exchange: passive_net,
        r_required,
        b_active,
        k_active_required: k,
        valid,
        reject_reason: reject,
    }
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

/// Portability gate over a set of state estimates.
pub fn reconstruct_active_rate(estimates: Vec<ActiveRateEstimate>) -> ActiveRateReconstruction {
    let valid: Vec<f64> = estimates
        .iter()
        .filter(|e| e.valid)
        .map(|e| e.k_active_required)
        .collect();
    let valid_count = valid.len();
    let mut sorted = valid.clone();
    let median = median_sorted(&mut sorted);
    let (span, loo, loo_ok, portable, conclusion) = if valid_count < D032_MIN_VALID_STATES {
        (
            f64::NAN,
            Vec::new(),
            false,
            false,
            "D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE".to_string(),
        )
    } else {
        let min_k = valid.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_k = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = if min_k > 0.0 { max_k / min_k } else { f64::INFINITY };
        let mut loo_medians = Vec::new();
        let mut loo_ok = true;
        for i in 0..valid.len() {
            let mut others: Vec<f64> = valid
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, v)| *v)
                .collect();
            let m = median_sorted(&mut others);
            loo_medians.push(m);
            if !m.is_finite()
                || median <= 0.0
                || ((m - median).abs() / median) > D032_LOO_MEDIAN_REL_MAX
            {
                loo_ok = false;
            }
        }
        let portable = span.is_finite()
            && span <= D032_PORTABILITY_SPAN_MAX
            && loo_ok
            && median.is_finite()
            && median > 0.0;
        let conclusion = if portable {
            "D032_ACTIVE_RATE_PORTABLE".to_string()
        } else {
            "D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE".to_string()
        };
        (span, loo_medians, loo_ok, portable, conclusion)
    };
    ActiveRateReconstruction {
        estimates,
        valid_count,
        median_k_active: median,
        span_factor: span,
        loo_medians: loo,
        loo_ok,
        portable,
        conclusion,
    }
}

/// Generate ≤5 candidates: 0.5× / 1.0× / 2.0× of median, optionally bracketed interpolate.
pub fn generate_active_candidates(median_k: f64) -> Vec<ActiveCandidate> {
    let mut out = Vec::new();
    for &scale in &D032_CANDIDATE_SCALES {
        out.push(ActiveCandidate {
            identity: format!("k_active_{scale}x"),
            k_active: median_k * scale,
            scale,
        });
    }
    out.truncate(D032_MAX_ACTIVE_CANDIDATES);
    out
}

/// Safeguarded bracketed interpolation between a below-balance and above-balance candidate.
pub fn bracketed_interpolate(k_lo: f64, k_hi: f64, q_lo: f64, q_hi: f64) -> Option<f64> {
    if !(k_lo < k_hi && q_lo.is_finite() && q_hi.is_finite()) {
        return None;
    }
    // Target Q = 1.
    if (q_lo - 1.0) * (q_hi - 1.0) >= 0.0 {
        return None;
    }
    let denom = q_hi - q_lo;
    if denom.abs() < 1e-30 {
        return None;
    }
    let k = k_lo + (1.0 - q_lo) * (k_hi - k_lo) / denom;
    if k.is_finite() && k > k_lo && k < k_hi {
        Some(k)
    } else {
        None
    }
}

/// Continuous combined (passive + active) field remains inside the domain at physical corners.
pub fn combined_boundary_inward(
    p: f64,
    a: f64,
    s: f64,
    delta: f64,
    catalyst: f64,
    params: &SimParams,
) -> bool {
    let q_c = membrane_catalyst_saturation(catalyst, params);
    let g = if delta > params.delta_floor {
        s / delta
    } else {
        0.0
    };
    let j_act = active_assembly_rate_j(p, a, catalyst, g, params);
    let theta = surface_occupancy_theta(g, params.gamma_max);
    // Active assembly vanishes at every physical domain face that blocks it.
    if p <= 1e-14 || a <= 1e-14 || theta >= 1.0 - 1e-12 || q_c <= 0.0 {
        if j_act.abs() > 1e-14 {
            return false;
        }
    } else if j_act < -1e-14 {
        return false;
    }
    // Passive exchange continuous signs (independent of A).
    let signs = classify_exchange_invariant_field(p.max(0.0), s.max(0.0), delta, q_c, params);
    // At P=0 / S=0 / θ=1 corners, require the exchange classifier's corner derivatives.
    if p <= 1e-14 {
        return signs.dp_at_p0 >= -1e-14 && j_act.abs() <= 1e-14;
    }
    if s <= 1e-14 {
        return signs.ds_at_s0 >= -1e-14;
    }
    if theta >= 1.0 - 1e-12 {
        return signs.ds_at_theta1 <= 1e-14 && j_act.abs() <= 1e-14;
    }
    true
}

/// Gate 1 corner sweep: P=0, A=0, S=0, θ=1.
pub fn combined_domain_corners_ok(params: &SimParams) -> bool {
    let d = 0.5_f64;
    let c = 0.4_f64;
    combined_boundary_inward(0.0, 1.0, 0.2, d, c, params)
        && combined_boundary_inward(1.0, 0.0, 0.2, d, c, params)
        && combined_boundary_inward(1.0, 1.0, 0.0, d, c, params)
        && combined_boundary_inward(1.0, 1.0, d * params.gamma_max, d, c, params)
}

/// Material residual of one bounded active step: ΔP+ΔA+ΔS+ΔW.
pub fn active_material_residual(
    p0: f64,
    a0: f64,
    s0: f64,
    w0: f64,
    delta: f64,
    catalyst: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64) {
    let (p1, a1, s1, dw, r) =
        apply_active_assembly_bounded(p0, a0, s0, delta, catalyst, dt, params);
    let w1 = w0 + dw;
    let residual = (p1 - p0) + (a1 - a0) + (s1 - s0) + (w1 - w0);
    (residual, r)
}

/// Frozen α, β identity check.
pub fn frozen_exchange_kinetics_ok() -> bool {
    let c = frozen_exchange_candidate();
    let alpha = c.k_exchange * c.k_exchange_eq;
    let beta = c.k_exchange;
    ((alpha - D031_ALPHA_FROZEN) / D031_ALPHA_FROZEN).abs() < 1e-12
        && ((beta - D031_BETA_FROZEN) / D031_BETA_FROZEN).abs() < 1e-12
}
