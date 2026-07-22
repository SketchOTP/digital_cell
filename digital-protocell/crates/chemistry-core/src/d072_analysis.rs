//! D-072 mature-membrane damage refill causal audit helpers.
//!
//! Frozen D-070/D-071 exchange kinetics and `SEED_CAPACITY_CONTRACT_V1`.
//! No kinetic, activation, precursor-production, or carrier changes.

use crate::d069_analysis::{p_activity, theta_eq, D069_K_EQ, D069_K_EXCHANGE};
use crate::d070_analysis::{D070_GAMMA_MAX, D070_K_EQ, D070_K_EXCHANGE, SEED_CAPACITY_CONTRACT_V1};
use crate::d071_analysis::{D071_GAMMA_MAX, D071_K_EQ, D071_K_EXCHANGE};
use serde::{Deserialize, Serialize};

pub const D072_PROJECT_ID: &str = "D-072";
pub const D072_AGENT_MEMORY_ID: &str =
    "D-20260722-d072-mature-membrane-damage-refill-causal-audit";
pub const D072_STARTING_COMMIT: &str = "0611603";
pub const D072_STARTING_TAG: &str = "D-071-precursor-demand-regulation-fail";
pub const D070_TAG: &str = "D-070-mature-membrane-seed-capacity-repair";
pub const D071_TAG: &str = "D-071-precursor-demand-regulation-fail";
pub const D071_CONCLUSION: &str = "D071_FAIL";

pub const D072_K_EXCHANGE: f64 = D071_K_EXCHANGE;
pub const D072_K_EQ: f64 = D071_K_EQ;
pub const D072_GAMMA_MAX: f64 = D071_GAMMA_MAX;

/// D-071 selected reduced-constitutive regulation (opt-in diagnostic only).
pub const D071_SELECTED_M_P: f64 = 0.0013190570087785272;
pub const D071_SELECTED_IDENTITY: &str =
    "27c54fadf69d933bd2760cc3b6689edf5501804809dc523b89c8d35760aa5247";

pub const DAMAGE_FRACTION: f64 = 0.10;
pub const REPAIR_THRESHOLD: f64 = 0.95;
pub const D071_PRE_OCC_TARGET: f64 = 0.992;
pub const D071_REPAIR_LO: f64 = 0.894;
pub const D071_REPAIR_HI: f64 = 0.898;
pub const ACCOUNTING_TOL: f64 = 1e-9;
pub const PARITY_TOL: f64 = 1e-6;
pub const EPS: f64 = 1e-15;

pub const SEED_CONTRACT: &str = SEED_CAPACITY_CONTRACT_V1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D072PrimaryConclusion {
    DamageRefillExecutionDefect,
    DamageRefillHorizonQualified,
    LocalPrecursorDeliveryLimit,
    LocalCatalystSupportLimit,
    InterfaceSupportLimit,
    CoupledRepairCompetition,
    FrozenExchangeCannotRefillDamage,
    D071RepairResultNotReproduced,
    DamageInterventionAccountingDefect,
    DamageStateSynchronizationDefect,
    DamageDerivedStateInvalidationDefect,
    ExchangeRefillExecutionDefect,
    AccountingFailure,
    NumericalFailure,
}

impl D072PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DamageRefillExecutionDefect => "D072_DAMAGE_REFILL_EXECUTION_DEFECT",
            Self::DamageRefillHorizonQualified => "D072_DAMAGE_REFILL_HORIZON_QUALIFIED",
            Self::LocalPrecursorDeliveryLimit => "D072_LOCAL_PRECURSOR_DELIVERY_LIMIT",
            Self::LocalCatalystSupportLimit => "D072_LOCAL_CATALYST_SUPPORT_LIMIT",
            Self::InterfaceSupportLimit => "D072_INTERFACE_SUPPORT_LIMIT",
            Self::CoupledRepairCompetition => "D072_COUPLED_REPAIR_COMPETITION",
            Self::FrozenExchangeCannotRefillDamage => "D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE",
            Self::D071RepairResultNotReproduced => "D072_D071_REPAIR_RESULT_NOT_REPRODUCED",
            Self::DamageInterventionAccountingDefect => "D072_DAMAGE_INTERVENTION_ACCOUNTING_DEFECT",
            Self::DamageStateSynchronizationDefect => "D072_DAMAGE_STATE_SYNCHRONIZATION_DEFECT",
            Self::DamageDerivedStateInvalidationDefect => {
                "D072_DAMAGE_DERIVED_STATE_INVALIDATION_DEFECT"
            }
            Self::ExchangeRefillExecutionDefect => "D072_EXCHANGE_REFILL_EXECUTION_DEFECT",
            Self::AccountingFailure => "D072_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D072_NUMERICAL_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D072Route {
    E,
    H,
    P,
    C,
    I,
    B,
    X,
    StopD071NotReproduced,
    StopIntervention,
    StopSynthetic,
    StopAccounting,
    StopNumerical,
}

impl D072Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E => "Route_E_execution_or_harness_defect",
            Self::H => "Route_H_horizon_defect",
            Self::P => "Route_P_local_precursor_delivery_limit",
            Self::C => "Route_C_catalyst_support_limit",
            Self::I => "Route_I_interface_support_limit",
            Self::B => "Route_B_coupled_competition",
            Self::X => "Route_X_frozen_exchange_cannot_refill",
            Self::StopD071NotReproduced => "Stop_D071_repair_not_reproduced",
            Self::StopIntervention => "Stop_intervention_integrity",
            Self::StopSynthetic => "Stop_synthetic_refill_parity",
            Self::StopAccounting => "Stop_accounting",
            Self::StopNumerical => "Stop_numerical",
        }
    }

    pub const fn conclusion(self) -> D072PrimaryConclusion {
        match self {
            Self::E => D072PrimaryConclusion::DamageRefillExecutionDefect,
            Self::H => D072PrimaryConclusion::DamageRefillHorizonQualified,
            Self::P => D072PrimaryConclusion::LocalPrecursorDeliveryLimit,
            Self::C => D072PrimaryConclusion::LocalCatalystSupportLimit,
            Self::I => D072PrimaryConclusion::InterfaceSupportLimit,
            Self::B => D072PrimaryConclusion::CoupledRepairCompetition,
            Self::X => D072PrimaryConclusion::FrozenExchangeCannotRefillDamage,
            Self::StopD071NotReproduced => D072PrimaryConclusion::D071RepairResultNotReproduced,
            Self::StopIntervention => D072PrimaryConclusion::DamageInterventionAccountingDefect,
            Self::StopSynthetic => D072PrimaryConclusion::ExchangeRefillExecutionDefect,
            Self::StopAccounting => D072PrimaryConclusion::AccountingFailure,
            Self::StopNumerical => D072PrimaryConclusion::NumericalFailure,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefillBasisClass {
    RefillBasisPresent,
    LocalPInsufficient,
    LocalCatalystSupportInsufficient,
    InterfaceSupportMissing,
    NetExchangeNonpositive,
}

impl RefillBasisClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RefillBasisPresent => "REFILL_BASIS_PRESENT",
            Self::LocalPInsufficient => "LOCAL_P_INSUFFICIENT",
            Self::LocalCatalystSupportInsufficient => "LOCAL_CATALYST_SUPPORT_INSUFFICIENT",
            Self::InterfaceSupportMissing => "INTERFACE_SUPPORT_MISSING",
            Self::NetExchangeNonpositive => "NET_EXCHANGE_NONPOSITIVE",
        }
    }
}

/// Immediate no-repair floor after removing `fraction` of mature membrane at pre-damage occupancy.
#[inline]
pub fn no_repair_floor(pre_damage_occupancy: f64, damage_fraction: f64) -> f64 {
    pre_damage_occupancy * (1.0 - damage_fraction.clamp(0.0, 1.0))
}

/// Expected D-071 floor: ≈ 0.992 × 0.90 = 0.8928.
#[inline]
pub fn expected_d071_no_repair_floor() -> f64 {
    no_repair_floor(D071_PRE_OCC_TARGET, DAMAGE_FRACTION)
}

/// Linearized exchange timescale τ = 1 / (k_exchange · q(C) · (K_eq p + 1)).
#[inline]
pub fn exchange_timescale(k_exchange: f64, q_c: f64, k_eq: f64, p: f64) -> f64 {
    let denom = k_exchange.max(0.0) * q_c.max(0.0) * (k_eq.max(0.0) * p.max(0.0) + 1.0);
    if denom <= EPS {
        f64::INFINITY
    } else {
        1.0 / denom
    }
}

#[inline]
pub fn frozen_kinetics_unchanged(k_eq: f64, k_exchange: f64, gamma_max: f64) -> bool {
    (k_eq - D072_K_EQ).abs() <= 1e-15
        && (k_exchange - D072_K_EXCHANGE).abs() <= 1e-15
        && (gamma_max - D072_GAMMA_MAX).abs() <= 1e-15
        && (D072_K_EQ - D070_K_EQ).abs() <= 1e-15
        && (D072_K_EXCHANGE - D070_K_EXCHANGE).abs() <= 1e-15
        && (D072_GAMMA_MAX - D070_GAMMA_MAX).abs() <= 1e-15
        && (D069_K_EQ - D070_K_EQ).abs() <= 1e-15
        && (D069_K_EXCHANGE - D070_K_EXCHANGE).abs() <= 1e-15
}

#[inline]
pub fn s_w_conservation(delta_s: f64, delta_w: f64, tol: f64) -> bool {
    (delta_s + delta_w).abs() <= tol * (1.0 + delta_s.abs().max(delta_w.abs()))
}

#[inline]
pub fn d071_repair_reproduced(ratio: f64) -> bool {
    ratio.is_finite() && ratio >= D071_REPAIR_LO - 0.005 && ratio <= D071_REPAIR_HI + 0.005
}

#[inline]
pub fn near_no_repair_floor(ratio: f64, floor: f64, tol: f64) -> bool {
    ratio.is_finite() && (ratio - floor).abs() <= tol
}

/// Adsorption / desorption bases matching frozen exchange law (δ·k·q·Γ_max·…).
#[inline]
pub fn adsorption_basis(
    delta: f64,
    k_exchange: f64,
    q_c: f64,
    gamma_max: f64,
    k_eq: f64,
    p: f64,
    theta: f64,
) -> f64 {
    delta.max(0.0)
        * k_exchange.max(0.0)
        * q_c.max(0.0)
        * gamma_max.max(0.0)
        * k_eq.max(0.0)
        * p.max(0.0)
        * (1.0 - theta).max(0.0)
}

#[inline]
pub fn desorption_basis(
    delta: f64,
    k_exchange: f64,
    q_c: f64,
    gamma_max: f64,
    theta: f64,
) -> f64 {
    delta.max(0.0)
        * k_exchange.max(0.0)
        * q_c.max(0.0)
        * gamma_max.max(0.0)
        * theta.clamp(0.0, 1.0)
}

#[inline]
pub fn net_exchange_basis(
    delta: f64,
    k_exchange: f64,
    q_c: f64,
    gamma_max: f64,
    k_eq: f64,
    p: f64,
    theta: f64,
) -> f64 {
    adsorption_basis(delta, k_exchange, q_c, gamma_max, k_eq, p, theta)
        - desorption_basis(delta, k_exchange, q_c, gamma_max, theta)
}

/// Classify local refill basis immediately after damage (exactly one class).
pub fn classify_refill_basis(
    delta: f64,
    capacity: f64,
    free_capacity: f64,
    p: f64,
    q_c: f64,
    net_exchange: f64,
    p_floor: f64,
    q_floor: f64,
) -> RefillBasisClass {
    classify_refill_basis_with_eq(delta, capacity, free_capacity, p, q_c, net_exchange, p_floor, q_floor, D072_K_EQ, REPAIR_THRESHOLD)
}

/// Same as [`classify_refill_basis`], but requires θ_eq(p) ≥ `occ_target` for PRESENT.
pub fn classify_refill_basis_with_eq(
    delta: f64,
    capacity: f64,
    free_capacity: f64,
    p: f64,
    q_c: f64,
    net_exchange: f64,
    p_floor: f64,
    q_floor: f64,
    k_eq: f64,
    occ_target: f64,
) -> RefillBasisClass {
    if delta <= EPS || capacity <= EPS {
        return RefillBasisClass::InterfaceSupportMissing;
    }
    if free_capacity <= EPS {
        return RefillBasisClass::InterfaceSupportMissing;
    }
    if p < p_floor {
        return RefillBasisClass::LocalPInsufficient;
    }
    if q_c < q_floor {
        return RefillBasisClass::LocalCatalystSupportInsufficient;
    }
    if net_exchange <= EPS {
        return RefillBasisClass::NetExchangeNonpositive;
    }
    // Positive local adsorption can still be insufficient to meet the recovery occupancy target.
    if equilibrium_occupancy(p, k_eq) + 1e-12 < occ_target {
        return RefillBasisClass::LocalPInsufficient;
    }
    RefillBasisClass::RefillBasisPresent
}

#[derive(Debug, Clone, Copy)]
pub struct RouteEvidence072 {
    pub d071_reproduced: bool,
    pub intervention_ok: bool,
    pub synthetic_parity_ok: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub execution_defect: bool,
    pub horizon_recovers: bool,
    pub exchange_only_recovers: bool,
    pub mixed_p_recovers: bool,
    pub fixed_p_recovers: bool,
    pub healthy_q_recovers: bool,
    pub preserved_interface_recovers: bool,
    pub refill_basis: RefillBasisClass,
    pub tau_checkpoints_tested: bool,
}

impl Default for RouteEvidence072 {
    fn default() -> Self {
        Self {
            d071_reproduced: false,
            intervention_ok: false,
            synthetic_parity_ok: false,
            accounting_ok: false,
            numerical_ok: true,
            execution_defect: false,
            horizon_recovers: false,
            exchange_only_recovers: false,
            mixed_p_recovers: false,
            fixed_p_recovers: false,
            healthy_q_recovers: false,
            preserved_interface_recovers: false,
            refill_basis: RefillBasisClass::NetExchangeNonpositive,
            tau_checkpoints_tested: false,
        }
    }
}

/// Priority: stops → E → synthetic → H → B → P → C → I → X.
pub fn select_route(ev: RouteEvidence072) -> D072Route {
    if !ev.numerical_ok {
        return D072Route::StopNumerical;
    }
    if !ev.d071_reproduced {
        return D072Route::StopD071NotReproduced;
    }
    if !ev.intervention_ok || ev.execution_defect {
        return D072Route::E;
    }
    if !ev.synthetic_parity_ok {
        return D072Route::StopSynthetic;
    }
    if !ev.accounting_ok {
        return D072Route::StopAccounting;
    }
    if ev.horizon_recovers {
        return D072Route::H;
    }
    // Exchange-only recovers while the full coupled system does not.
    if ev.exchange_only_recovers {
        return D072Route::B;
    }
    // Route P requires a positive control that actually restores refill.
    if ev.mixed_p_recovers || ev.fixed_p_recovers {
        return D072Route::P;
    }
    if ev.healthy_q_recovers {
        return D072Route::C;
    }
    if ev.preserved_interface_recovers {
        return D072Route::I;
    }
    if matches!(ev.refill_basis, RefillBasisClass::InterfaceSupportMissing) {
        return D072Route::I;
    }
    // Local P classified insufficient but fixed/mixed P did not restore → not Route P.
    // With synthetic parity + τ tested + no restoring control ⇒ frozen organism refill failure.
    if ev.tau_checkpoints_tested {
        return D072Route::X;
    }
    match ev.refill_basis {
        RefillBasisClass::LocalPInsufficient => D072Route::P,
        RefillBasisClass::LocalCatalystSupportInsufficient => D072Route::C,
        RefillBasisClass::InterfaceSupportMissing => D072Route::I,
        RefillBasisClass::NetExchangeNonpositive => D072Route::B,
        RefillBasisClass::RefillBasisPresent => D072Route::StopNumerical,
    }
}

/// Analytical expected S gain over dt for linearised near-empty cell (θ≈0).
#[inline]
pub fn analytical_s_gain_empty(
    delta: f64,
    k_exchange: f64,
    q_c: f64,
    gamma_max: f64,
    k_eq: f64,
    p: f64,
    dt: f64,
) -> f64 {
    adsorption_basis(delta, k_exchange, q_c, gamma_max, k_eq, p, 0.0) * dt.max(0.0)
}

#[inline]
pub fn equilibrium_occupancy(p: f64, k_eq: f64) -> f64 {
    theta_eq(p, k_eq)
}

#[inline]
pub fn normalized_p(precursor: f64, p_reference: f64) -> f64 {
    p_activity(precursor, p_reference)
}

#[inline]
pub fn free_capacity(capacity: f64, s: f64) -> f64 {
    (capacity - s.max(0.0)).max(0.0)
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn floor_matches_directive() {
        let f = expected_d071_no_repair_floor();
        assert!((f - 0.8928).abs() < 1e-12);
    }

    #[test]
    fn timescale_positive() {
        let tau = exchange_timescale(D072_K_EXCHANGE, 1.0, D072_K_EQ, 0.05);
        assert!(tau.is_finite() && tau > 1.0);
    }
}
