//! D-074 cellwise exchange integration parity helpers.
//!
//! Observer/diagnostic only. Frozen D-070…D-073 exchange kinetics and
//! `SEED_CAPACITY_CONTRACT_V1`. No biological equation or parameter changes.

use crate::d069_analysis::{theta_eq, D069_P_REF};
use crate::d070_analysis::SEED_CAPACITY_CONTRACT_V1;
use crate::d072_analysis::{exchange_timescale, REPAIR_THRESHOLD};
use crate::d073_analysis::{
    equilibrium_occupancy, p_required, D073_GAMMA_MAX, D073_K_EQ, D073_K_EXCHANGE, D073_P_REF,
};
use crate::surface_density::{
    exchange_scalar_f, solve_exchange_backward_euler, ExchangeReject, SURFACE_CAPACITY_FLOOR,
};
use serde::{Deserialize, Serialize};

pub const D074_PROJECT_ID: &str = "D-074";
pub const D074_AGENT_MEMORY_ID: &str =
    "D-20260722-d074-cellwise-exchange-integration-parity-repair";
pub const D074_STARTING_COMMIT: &str = "de407ca";
pub const D074_STARTING_TAG: &str = "D-073-mature-membrane-equilibrium-audit";
pub const D073_CONCLUSION: &str = "D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT";
pub const D073_ROUTE_E_STATUS: &str = "PROVISIONAL_PENDING_CELLWISE_PARITY";

pub const D074_K_EXCHANGE: f64 = D073_K_EXCHANGE;
pub const D074_K_EQ: f64 = D073_K_EQ;
pub const D074_GAMMA_MAX: f64 = D073_GAMMA_MAX;
pub const D074_P_REF: f64 = D073_P_REF;

pub const REPAIR_OCC: f64 = REPAIR_THRESHOLD;
pub const ACCOUNTING_TOL: f64 = 1e-9;
pub const PARITY_TOL: f64 = 1e-9;
pub const PARITY_TOL_RELAXED: f64 = 1e-5;
pub const EPS: f64 = 1e-15;
pub const Q_INACTIVE_FLOOR: f64 = 1e-12;
pub const EXPOSURE_COVERAGE_GATE: f64 = 0.95;
pub const SEED_CONTRACT: &str = SEED_CAPACITY_CONTRACT_V1;

/// Cross-check frozen constants remain identical to D-073 bindings.
pub fn frozen_kinetics_unchanged(k_eq: f64, k_exchange: f64, gamma_max: f64) -> bool {
    (k_eq - D074_K_EQ).abs() < 1e-15
        && (k_exchange - D074_K_EXCHANGE).abs() < 1e-15
        && (gamma_max - D074_GAMMA_MAX).abs() < 1e-15
        && (D074_P_REF - D069_P_REF).abs() < 1e-15
}

/// λ = k_exchange · q(C) · (K_eq p + 1).
#[inline]
pub fn exchange_lambda(k_exchange: f64, q_c: f64, k_eq: f64, p: f64) -> f64 {
    k_exchange.max(0.0) * q_c.max(0.0) * (k_eq.max(0.0) * p.max(0.0) + 1.0)
}

/// θ_eq = K_eq p / (1 + K_eq p).
#[inline]
pub fn theta_eq_of(p: f64, k_eq: f64) -> f64 {
    theta_eq(p, k_eq)
}

/// Bath-fixed backward-Euler occupancy update (directive §4 reference when p is held):
/// θ_{n+1} = θ_eq + (θ_n − θ_eq) / (1 + λ Δt).
#[inline]
pub fn discrete_bath_be_theta(theta_n: f64, theta_eq: f64, lambda: f64, dt: f64) -> f64 {
    let denom = 1.0 + lambda.max(0.0) * dt.max(0.0);
    if denom <= EPS {
        theta_n
    } else {
        theta_eq + (theta_n - theta_eq) / denom
    }
}

/// Residual attenuation factor A = ∏ 1/(1+λ_n Δt_n).
#[inline]
pub fn attenuation_factor(lambda: f64, dt: f64) -> f64 {
    let denom = 1.0 + lambda.max(0.0) * dt.max(0.0);
    if denom <= EPS {
        1.0
    } else {
        1.0 / denom
    }
}

/// One-step local exposure increment: Λ += k q (K_eq p + 1) Δt.
#[inline]
pub fn exposure_increment(k_exchange: f64, q_c: f64, k_eq: f64, p: f64, dt: f64) -> f64 {
    exchange_lambda(k_exchange, q_c, k_eq, p) * dt.max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExposureClass {
    ExposureGe5,
    Exposure1To5,
    ExposureLt1,
    ZeroExposure,
    InterfaceUnsupported,
}

impl ExposureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExposureGe5 => "EXPOSURE_GE_5",
            Self::Exposure1To5 => "EXPOSURE_1_TO_5",
            Self::ExposureLt1 => "EXPOSURE_LT_1",
            Self::ZeroExposure => "ZERO_EXPOSURE",
            Self::InterfaceUnsupported => "INTERFACE_UNSUPPORTED",
        }
    }
}

#[inline]
pub fn classify_exposure(lambda_cum: f64, capacity: f64, supported: bool) -> ExposureClass {
    if !supported || capacity <= SURFACE_CAPACITY_FLOOR {
        return ExposureClass::InterfaceUnsupported;
    }
    if lambda_cum <= EPS {
        ExposureClass::ZeroExposure
    } else if lambda_cum < 1.0 {
        ExposureClass::ExposureLt1
    } else if lambda_cum < 5.0 {
        ExposureClass::Exposure1To5
    } else {
        ExposureClass::ExposureGe5
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CellExchangeClass {
    ExchangeActive,
    ExchangeSlow,
    ExchangeInactiveQ0,
    UnsupportedCapacity,
    AlreadyAtOrAboveEq,
}

impl CellExchangeClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExchangeActive => "EXCHANGE_ACTIVE",
            Self::ExchangeSlow => "EXCHANGE_SLOW",
            Self::ExchangeInactiveQ0 => "EXCHANGE_INACTIVE_Q0",
            Self::UnsupportedCapacity => "UNSUPPORTED_CAPACITY",
            Self::AlreadyAtOrAboveEq => "ALREADY_AT_OR_ABOVE_EQ",
        }
    }
}

/// Classify a damaged interface cell for reachable-ceiling accounting.
pub fn classify_damaged_cell(
    q_c: f64,
    capacity: f64,
    theta: f64,
    theta_eq: f64,
    lambda: f64,
) -> CellExchangeClass {
    if capacity <= SURFACE_CAPACITY_FLOOR {
        return CellExchangeClass::UnsupportedCapacity;
    }
    if q_c <= Q_INACTIVE_FLOOR || lambda <= EPS {
        return CellExchangeClass::ExchangeInactiveQ0;
    }
    if theta + 1e-9 >= theta_eq {
        return CellExchangeClass::AlreadyAtOrAboveEq;
    }
    // "Slow" if local τ exceeds a generous multiple of the active scale.
    let tau = if lambda > EPS {
        1.0 / lambda
    } else {
        f64::INFINITY
    };
    if tau > 1.0e3 {
        CellExchangeClass::ExchangeSlow
    } else {
        CellExchangeClass::ExchangeActive
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachableCeilingReport {
    pub total_damaged_capacity: f64,
    pub active_exchange_capacity: f64,
    pub inactive_capacity: f64,
    pub unsupported_capacity: f64,
    pub already_eq_capacity: f64,
    pub capacity_weighted_equilibrium: f64,
    pub m_reachable: f64,
    pub m_post_damage: f64,
    pub max_theoretical_repair_fraction: f64,
    pub below_repair_gate: bool,
}

/// Infinite-time reachable mature mass under cellwise discrete law.
///
/// For exchange-inactive / unsupported cells, retain post-damage S (do not
/// assume equilibration). For active cells, use δ Γ_max θ_eq.
pub fn reachable_repair_ceiling(
    cells: &[(f64, f64, f64, f64, CellExchangeClass)],
    // (capacity, s_post_damage, theta_eq, undamaged_or_other_s_contrib ignored)
    undamaged_s: f64,
    pre_damage_s: f64,
) -> ReachableCeilingReport {
    let mut total_damaged_capacity = 0.0;
    let mut active_exchange_capacity = 0.0;
    let mut inactive_capacity = 0.0;
    let mut unsupported_capacity = 0.0;
    let mut already_eq_capacity = 0.0;
    let mut m_reachable_damaged = 0.0;
    let mut weighted_eq_num = 0.0;
    let mut weighted_eq_den = 0.0;

    for &(capacity, s_post, theta_eq_i, _q, class) in cells {
        total_damaged_capacity += capacity.max(0.0);
        match class {
            CellExchangeClass::UnsupportedCapacity => {
                unsupported_capacity += capacity.max(0.0);
                m_reachable_damaged += s_post.max(0.0);
            }
            CellExchangeClass::ExchangeInactiveQ0 => {
                inactive_capacity += capacity.max(0.0);
                m_reachable_damaged += s_post.max(0.0);
            }
            CellExchangeClass::AlreadyAtOrAboveEq => {
                already_eq_capacity += capacity.max(0.0);
                m_reachable_damaged += s_post.max(0.0).min(capacity.max(0.0));
                weighted_eq_num += capacity.max(0.0) * theta_eq_i;
                weighted_eq_den += capacity.max(0.0);
            }
            CellExchangeClass::ExchangeActive | CellExchangeClass::ExchangeSlow => {
                active_exchange_capacity += capacity.max(0.0);
                let s_eq = capacity.max(0.0) * theta_eq_i.clamp(0.0, 1.0);
                m_reachable_damaged += s_eq;
                weighted_eq_num += capacity.max(0.0) * theta_eq_i;
                weighted_eq_den += capacity.max(0.0);
            }
        }
    }

    let m_post_damage: f64 = cells.iter().map(|c| c.1.max(0.0)).sum::<f64>() + undamaged_s.max(0.0);
    let m_reachable = m_reachable_damaged + undamaged_s.max(0.0);
    let max_theoretical_repair_fraction = if pre_damage_s > EPS {
        (m_reachable / pre_damage_s).min(1.0)
    } else {
        0.0
    };
    let capacity_weighted_equilibrium = if weighted_eq_den > EPS {
        weighted_eq_num / weighted_eq_den
    } else {
        0.0
    };

    ReachableCeilingReport {
        total_damaged_capacity,
        active_exchange_capacity,
        inactive_capacity,
        unsupported_capacity,
        already_eq_capacity,
        capacity_weighted_equilibrium,
        m_reachable,
        m_post_damage,
        max_theoretical_repair_fraction,
        below_repair_gate: max_theoretical_repair_fraction + 1e-12 < REPAIR_OCC,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureCoverageReport {
    pub damaged_lawful_capacity: f64,
    pub capacity_ge5: f64,
    pub capacity_1_to_5: f64,
    pub capacity_lt1: f64,
    pub capacity_zero: f64,
    pub capacity_unsupported: f64,
    pub fraction_ge5: f64,
    pub qualifies_five_timescale: bool,
}

/// Capacity-weighted exposure coverage over damaged lawful capacity.
pub fn exposure_coverage(cells: &[(f64, ExposureClass)]) -> ExposureCoverageReport {
    let mut damaged_lawful_capacity = 0.0;
    let mut capacity_ge5 = 0.0;
    let mut capacity_1_to_5 = 0.0;
    let mut capacity_lt1 = 0.0;
    let mut capacity_zero = 0.0;
    let mut capacity_unsupported = 0.0;

    for &(cap, class) in cells {
        let c = cap.max(0.0);
        match class {
            ExposureClass::InterfaceUnsupported => capacity_unsupported += c,
            ExposureClass::ExposureGe5 => {
                damaged_lawful_capacity += c;
                capacity_ge5 += c;
            }
            ExposureClass::Exposure1To5 => {
                damaged_lawful_capacity += c;
                capacity_1_to_5 += c;
            }
            ExposureClass::ExposureLt1 => {
                damaged_lawful_capacity += c;
                capacity_lt1 += c;
            }
            ExposureClass::ZeroExposure => {
                damaged_lawful_capacity += c;
                capacity_zero += c;
            }
        }
    }

    let fraction_ge5 = if damaged_lawful_capacity > EPS {
        capacity_ge5 / damaged_lawful_capacity
    } else {
        0.0
    };

    ExposureCoverageReport {
        damaged_lawful_capacity,
        capacity_ge5,
        capacity_1_to_5,
        capacity_lt1,
        capacity_zero,
        capacity_unsupported,
        fraction_ge5,
        qualifies_five_timescale: fraction_ge5 + 1e-15 >= EXPOSURE_COVERAGE_GATE,
    }
}

/// Runtime-faithful invariant-domain exchange step (coupled T = P+S).
///
/// This is the production BE operator used when the explicit Euler proposal
/// leaves the invariant domain. Bath closed-form is NOT used here.
pub fn runtime_invariant_exchange_step(
    s_old: f64,
    p_old: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
    dt: f64,
) -> Result<(f64, f64, f64), ExchangeReject> {
    let c_surface = delta * gamma_max.max(0.0);
    let t_inv = p_old.max(0.0) + s_old.max(0.0);
    let solved = solve_exchange_backward_euler(
        s_old.max(0.0),
        t_inv,
        c_surface,
        delta,
        q_c,
        k_exchange,
        k_eq,
        p_reference,
        gamma_max,
        dt,
    )?;
    let xfer = solved.s_next - s_old.max(0.0);
    Ok((solved.s_next, solved.p_next, xfer))
}

/// Explicit-Euler proposal used by the production mild path.
#[inline]
pub fn explicit_euler_exchange_proposal(
    s_old: f64,
    p_old: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
    dt: f64,
) -> (f64, f64, f64) {
    let c_surface = delta * gamma_max.max(0.0);
    let t_inv = p_old.max(0.0) + s_old.max(0.0);
    let ds_dt = exchange_scalar_f(
        s_old,
        t_inv,
        c_surface,
        delta,
        q_c,
        k_exchange,
        k_eq,
        p_reference,
        gamma_max,
    );
    let xfer = ds_dt * dt;
    let s_e = s_old + xfer;
    let p_e = p_old - xfer;
    (s_e, p_e, xfer)
}

/// Predicted bath-BE ΔS for fixed activity p (diagnostic comparison only).
#[inline]
pub fn predicted_bath_be_delta_s(
    s_old: f64,
    capacity: f64,
    p: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    dt: f64,
) -> f64 {
    if capacity <= SURFACE_CAPACITY_FLOOR {
        return 0.0;
    }
    let theta_n = (s_old / capacity).clamp(0.0, 1.0);
    let th_eq = equilibrium_occupancy(p, k_eq);
    let lam = exchange_lambda(k_exchange, q_c, k_eq, p);
    let theta_next = discrete_bath_be_theta(theta_n, th_eq, lam, dt);
    (theta_next * capacity) - s_old
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D074PrimaryConclusion {
    ExchangeIntegrationDefectRepaired,
    LocalCatalyticExposureLimit,
    InterfaceSupportCoverageLimit,
    ExchangeTimescaleClassificationDefect,
    MembraneRepairMetricDefect,
    ExchangeRuntimeParityUnresolved,
    D073ResultNotReproduced,
    AccountingFailure,
    NumericalFailure,
}

impl D074PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExchangeIntegrationDefectRepaired => "D074_EXCHANGE_INTEGRATION_DEFECT_REPAIRED",
            Self::LocalCatalyticExposureLimit => "D074_LOCAL_CATALYTIC_EXPOSURE_LIMIT",
            Self::InterfaceSupportCoverageLimit => "D074_INTERFACE_SUPPORT_COVERAGE_LIMIT",
            Self::ExchangeTimescaleClassificationDefect => {
                "D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT"
            }
            Self::MembraneRepairMetricDefect => "D074_MEMBRANE_REPAIR_METRIC_DEFECT",
            Self::ExchangeRuntimeParityUnresolved => "D074_EXCHANGE_RUNTIME_PARITY_UNRESOLVED",
            Self::D073ResultNotReproduced => "D074_D073_RESULT_NOT_REPRODUCED",
            Self::AccountingFailure => "D074_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D074_NUMERICAL_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D074Route {
    E,
    Q,
    I,
    T,
    M,
    X,
    StopD073,
    StopAccounting,
    StopNumerical,
}

impl D074Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E => "Route_E_integration_defect_repaired",
            Self::Q => "Route_Q_local_catalytic_exposure_limit",
            Self::I => "Route_I_interface_support_coverage_limit",
            Self::T => "Route_T_timescale_classification_defect",
            Self::M => "Route_M_membrane_repair_metric_defect",
            Self::X => "Route_X_exchange_runtime_parity_unresolved",
            Self::StopD073 => "Stop_d073_not_reproduced",
            Self::StopAccounting => "Stop_accounting",
            Self::StopNumerical => "Stop_numerical",
        }
    }

    pub const fn conclusion(self) -> D074PrimaryConclusion {
        match self {
            Self::E => D074PrimaryConclusion::ExchangeIntegrationDefectRepaired,
            Self::Q => D074PrimaryConclusion::LocalCatalyticExposureLimit,
            Self::I => D074PrimaryConclusion::InterfaceSupportCoverageLimit,
            Self::T => D074PrimaryConclusion::ExchangeTimescaleClassificationDefect,
            Self::M => D074PrimaryConclusion::MembraneRepairMetricDefect,
            Self::X => D074PrimaryConclusion::ExchangeRuntimeParityUnresolved,
            Self::StopD073 => D074PrimaryConclusion::D073ResultNotReproduced,
            Self::StopAccounting => D074PrimaryConclusion::AccountingFailure,
            Self::StopNumerical => D074PrimaryConclusion::NumericalFailure,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RouteEvidence074 {
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub d073_reproduced: bool,
    pub static_cellwise_parity_ok: bool,
    pub accepted_step_replay_ok: bool,
    pub runtime_matches_discrete_predictor: bool,
    pub repair_restored_parity: bool,
    pub reachable_ceiling_below_gate: bool,
    pub inactive_q0_capacity_fraction: f64,
    pub unsupported_capacity_fraction: f64,
    pub exposure_qualifies_five_tau: bool,
    pub mean_tau_overstated_exposure: bool,
    pub aggregate_matches_cellwise_prediction: bool,
    pub local_exchange_correct_but_metric_wrong: bool,
}

impl Default for RouteEvidence074 {
    fn default() -> Self {
        Self {
            accounting_ok: true,
            numerical_ok: true,
            d073_reproduced: false,
            static_cellwise_parity_ok: false,
            accepted_step_replay_ok: false,
            runtime_matches_discrete_predictor: false,
            repair_restored_parity: false,
            reachable_ceiling_below_gate: false,
            inactive_q0_capacity_fraction: 0.0,
            unsupported_capacity_fraction: 0.0,
            exposure_qualifies_five_tau: false,
            mean_tau_overstated_exposure: false,
            aggregate_matches_cellwise_prediction: false,
            local_exchange_correct_but_metric_wrong: false,
        }
    }
}

/// Gate-13 route selection (exactly one primary).
///
/// Priority: stops → E (repaired) → Q → I → T → M → X.
pub fn select_route(ev: RouteEvidence074) -> D074Route {
    if !ev.numerical_ok {
        return D074Route::StopNumerical;
    }
    if !ev.accounting_ok {
        return D074Route::StopAccounting;
    }
    if !ev.d073_reproduced {
        return D074Route::StopD073;
    }

    // Route E — proven integration defect repaired; parity restored.
    if !ev.runtime_matches_discrete_predictor && ev.repair_restored_parity {
        return D074Route::E;
    }

    // Runtime agrees with discrete predictor hereafter.
    let runtime_ok = ev.runtime_matches_discrete_predictor
        && ev.static_cellwise_parity_ok
        && ev.accepted_step_replay_ok;

    // Route Q — catalyst exposure / zero-q limit.
    if runtime_ok
        && (ev.inactive_q0_capacity_fraction > 0.05
            || (ev.reachable_ceiling_below_gate && ev.inactive_q0_capacity_fraction > 1e-6))
    {
        return D074Route::Q;
    }

    // Route I — interface support / capacity coverage.
    if runtime_ok && ev.unsupported_capacity_fraction > 0.05 {
        return D074Route::I;
    }

    // Route T — mean-τ overstated cellwise exposure.
    if runtime_ok && (!ev.exposure_qualifies_five_tau || ev.mean_tau_overstated_exposure) {
        return D074Route::T;
    }

    // Route M — local exchange correct; aggregate metric wrong.
    if runtime_ok
        && ev.local_exchange_correct_but_metric_wrong
        && !ev.aggregate_matches_cellwise_prediction
    {
        return D074Route::M;
    }

    // Route X — unresolved parity failure after bounded repair attempt.
    if !ev.runtime_matches_discrete_predictor && !ev.repair_restored_parity {
        return D074Route::X;
    }

    // Fallback: if ceiling blocks repair without large inactive/unsupported share,
    // prefer T when exposure fails, else Q.
    if ev.reachable_ceiling_below_gate {
        if !ev.exposure_qualifies_five_tau {
            return D074Route::T;
        }
        return D074Route::Q;
    }

    D074Route::X
}

/// D-073 Gate-0 expected recovery anchors (approx).
pub fn d073_expected_recoveries() -> &'static [(&'static str, f64, f64)] {
    // (label, intended_p, expected_recovery_ratio)
    &[
        ("0_9x_p095", 0.342, 0.931),
        ("1_0x_p095", 0.380, 0.941),
        ("1_1x_p095", 0.418, 0.948),
        ("d070_maintenance", 2.48, 0.979),
    ]
}

/// Absolute tolerance for D-073 reproduction of recovery ratios.
pub const D073_REPRO_TOL: f64 = 0.015;

#[inline]
pub fn recovery_matches_d073(observed: f64, expected: f64) -> bool {
    (observed - expected).abs() <= D073_REPRO_TOL
}

/// Preserve D-073 conclusion string identity.
pub fn d073_conclusion_preserved(reported: &str) -> bool {
    reported == D073_CONCLUSION
}

/// Helper: p_required re-export for assays.
#[inline]
pub fn p_star(theta: f64) -> f64 {
    p_required(theta, D074_K_EQ)
}

/// Linearized τ helper (mean-τ diagnostic; not cellwise exposure).
#[inline]
pub fn mean_tau_proxy(k_exchange: f64, q_c: f64, k_eq: f64, p: f64) -> f64 {
    exchange_timescale(k_exchange, q_c, k_eq, p)
}

#[cfg(test)]
mod local_checks {
    use super::*;

    #[test]
    fn bath_be_matches_closed_form_identity() {
        let k = D074_K_EQ;
        let p = 0.38;
        let th_eq = theta_eq_of(p, k);
        let lam = exchange_lambda(D074_K_EXCHANGE, 0.4, k, p);
        let th0 = 0.0;
        let dt = 0.05;
        let th1 = discrete_bath_be_theta(th0, th_eq, lam, dt);
        assert!((th1 - (th_eq + (th0 - th_eq) / (1.0 + lam * dt))).abs() < 1e-15);
        assert!(th1 > th0 && th1 < th_eq);
    }

    #[test]
    fn zero_q_gives_zero_exposure_and_inactive() {
        let lam = exchange_lambda(D074_K_EXCHANGE, 0.0, D074_K_EQ, 0.38);
        assert!(lam <= EPS);
        assert_eq!(
            classify_exposure(0.0, 1.0, true),
            ExposureClass::ZeroExposure
        );
        assert_eq!(
            classify_damaged_cell(0.0, 1.0, 0.0, 0.95, 0.0),
            CellExchangeClass::ExchangeInactiveQ0
        );
    }
}
