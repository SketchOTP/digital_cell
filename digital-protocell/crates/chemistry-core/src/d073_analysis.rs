//! D-073 mature-membrane equilibrium sufficiency audit helpers.
//!
//! Observer/diagnostic only. Frozen D-070/D-071/D-072 exchange kinetics and
//! `SEED_CAPACITY_CONTRACT_V1`. No biological equation or parameter changes.

use crate::d069_analysis::{p_eq, theta_eq, D069_K_EQ, D069_P_REF};
use crate::d070_analysis::{
    D070_GAMMA_MAX, D070_K_EQ, D070_K_EXCHANGE, SEED_CAPACITY_CONTRACT_V1, STAGE_E_MIN_OCCUPANCY,
};
use crate::d071_analysis::{D071_GAMMA_MAX, D071_K_EQ, D071_K_EXCHANGE, OCC_FLOOR};
use crate::d072_analysis::{
    D072PrimaryConclusion, D072_GAMMA_MAX, D072_K_EQ, D072_K_EXCHANGE, REPAIR_THRESHOLD,
};
use serde::{Deserialize, Serialize};

pub const D073_PROJECT_ID: &str = "D-073";
pub const D073_AGENT_MEMORY_ID: &str =
    "D-20260722-d073-mature-membrane-equilibrium-sufficiency-audit";
pub const D073_STARTING_COMMIT: &str = "28dcdc4";
pub const D073_STARTING_TAG: &str = "D-072-membrane-damage-refill-audit";
pub const D072_ORIGINAL_CONCLUSION: &str = "D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE";
pub const D072_ROUTE_STATUS: &str = "PROVISIONAL_PENDING_EQUILIBRIUM_SUFFICIENCY_AUDIT";

pub const D073_K_EXCHANGE: f64 = D072_K_EXCHANGE;
pub const D073_K_EQ: f64 = D072_K_EQ;
pub const D073_GAMMA_MAX: f64 = D072_GAMMA_MAX;
pub const D073_P_REF: f64 = D069_P_REF;

/// D-070 reported lawful maintenance occupancy (Seed B / Policy D absolute occupancy).
pub const D070_LAWFUL_MAINTENANCE_OCCUPANCY: f64 = 0.992;
pub const STAGE_E_MEMBRANE_THRESHOLD: f64 = STAGE_E_MIN_OCCUPANCY;
pub const REPAIR_OCC: f64 = REPAIR_THRESHOLD; // 0.95
pub const OCC_090: f64 = 0.90;
pub const OCC_075: f64 = 0.75;

pub const P_HOLD_TOL_FRAC: f64 = 0.02;
pub const EQ_PARITY_TOL: f64 = 1e-9;
pub const ACCOUNTING_TOL: f64 = 1e-9;
pub const EPS: f64 = 1e-15;
pub const SEED_CONTRACT: &str = SEED_CAPACITY_CONTRACT_V1;

/// Cross-check frozen constants remain identical across D-069…D-072 bindings.
pub fn frozen_kinetics_unchanged(k_eq: f64, k_exchange: f64, gamma_max: f64) -> bool {
    (k_eq - D073_K_EQ).abs() < 1e-15
        && (k_exchange - D073_K_EXCHANGE).abs() < 1e-15
        && (gamma_max - D073_GAMMA_MAX).abs() < 1e-15
        && (D069_K_EQ - D070_K_EQ).abs() < 1e-15
        && (D070_K_EQ - D071_K_EQ).abs() < 1e-15
        && (D071_K_EQ - D072_K_EQ).abs() < 1e-15
        && (D070_K_EXCHANGE - D071_K_EXCHANGE).abs() < 1e-15
        && (D071_K_EXCHANGE - D072_K_EXCHANGE).abs() < 1e-15
        && (D070_GAMMA_MAX - D071_GAMMA_MAX).abs() < 1e-15
        && (D071_GAMMA_MAX - D072_GAMMA_MAX).abs() < 1e-15
}

/// Exact equilibrium inversion: p*(θ*) = θ* / (K_eq (1 − θ*)).
#[inline]
pub fn p_required(theta_star: f64, k_eq: f64) -> f64 {
    p_eq(theta_star, k_eq)
}

/// θ_eq(p) = K_eq p / (1 + K_eq p).
#[inline]
pub fn equilibrium_occupancy(p: f64, k_eq: f64) -> f64 {
    theta_eq(p, k_eq)
}

/// Concentration that realizes activity `p` under reference `p_ref`.
#[inline]
pub fn concentration_for_activity(p: f64, p_ref: f64) -> f64 {
    p.max(0.0) * p_ref.max(EPS)
}

#[inline]
pub fn activity_from_concentration(precursor: f64, p_ref: f64) -> f64 {
    if p_ref <= EPS {
        0.0
    } else {
        precursor.max(0.0) / p_ref
    }
}

/// Gate-0 contract rows for the frozen exchange law.
pub fn equilibrium_contract_rows(k_eq: f64) -> Vec<EquilibriumContractRow> {
    let targets = [
        ("theta_0_75", OCC_075),
        ("theta_0_90", OCC_090),
        ("theta_0_95", REPAIR_OCC),
        ("d070_lawful_maintenance", D070_LAWFUL_MAINTENANCE_OCCUPANCY),
        ("stage_e_membrane_threshold", STAGE_E_MEMBRANE_THRESHOLD),
        ("occ_floor_d071", OCC_FLOOR),
    ];
    targets
        .into_iter()
        .map(|(name, theta)| {
            let p = p_required(theta, k_eq);
            let th_back = equilibrium_occupancy(p, k_eq);
            EquilibriumContractRow {
                name: name.into(),
                theta_star: theta,
                p_required: p,
                theta_eq_from_p: th_back,
                inversion_ok: (th_back - theta).abs() < EQ_PARITY_TOL
                    || (!theta.is_finite() && !th_back.is_finite()),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquilibriumContractRow {
    pub name: String,
    pub theta_star: f64,
    pub p_required: f64,
    pub theta_eq_from_p: f64,
    pub inversion_ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FixedPControlClass {
    TargetSufficient,
    TargetInsufficient,
    SpatiallyIncomplete,
    NotActuallyFixed,
    Unknown,
}

impl FixedPControlClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TargetSufficient => "TARGET_SUFFICIENT",
            Self::TargetInsufficient => "TARGET_INSUFFICIENT",
            Self::SpatiallyIncomplete => "SPATIALLY_INCOMPLETE",
            Self::NotActuallyFixed => "NOT_ACTUALLY_FIXED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Classify a reported fixed-P control against the repair equilibrium target.
///
/// Priority: unknown → not actually fixed → spatially incomplete →
/// target insufficient → target sufficient.
pub fn classify_fixed_p_control(
    imposed_p: Option<f64>,
    p_required_target: f64,
    spatially_complete: Option<bool>,
    actually_fixed: Option<bool>,
) -> FixedPControlClass {
    match (imposed_p, spatially_complete, actually_fixed) {
        (None, _, _) | (_, None, _) | (_, _, None) => FixedPControlClass::Unknown,
        (_, Some(false), _) => FixedPControlClass::SpatiallyIncomplete,
        (_, _, Some(false)) => FixedPControlClass::NotActuallyFixed,
        (Some(p), Some(true), Some(true)) => {
            if p + 1e-15 < p_required_target {
                FixedPControlClass::TargetInsufficient
            } else {
                FixedPControlClass::TargetSufficient
            }
        }
    }
}

/// D-072 fixed_sufficient_p: set P = max(p_ref, 1) once for all dish cells; not reheld.
pub fn d072_fixed_p_audit(
    p_reference: f64,
    p_required_095: f64,
) -> (f64, f64, FixedPControlClass) {
    let imposed_concentration = p_reference.max(1.0);
    let imposed_p = activity_from_concentration(imposed_concentration, p_reference);
    let class = classify_fixed_p_control(
        Some(imposed_p),
        p_required_095,
        Some(true), // all in_dish cells covered at t0
        Some(false), // applied once; reactions remain enabled
    );
    (imposed_concentration, imposed_p, class)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LongHorizonClass {
    TrueMaintenance,
    SlowTransientDecay,
    EquilibriumBelowContract,
    BiologicalCollapse,
    NotConverged,
}

impl LongHorizonClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrueMaintenance => "TRUE_MAINTENANCE",
            Self::SlowTransientDecay => "SLOW_TRANSIENT_DECAY",
            Self::EquilibriumBelowContract => "EQUILIBRIUM_BELOW_CONTRACT",
            Self::BiologicalCollapse => "BIOLOGICAL_COLLAPSE",
            Self::NotConverged => "NOT_CONVERGED",
        }
    }
}

/// Classify undamaged long-horizon Seed B occupancy relative to predicted θ_eq and contract.
pub fn classify_long_horizon(
    final_occupancy: f64,
    predicted_theta_eq: f64,
    contract_occupancy: f64,
    initial_occupancy: f64,
    converged: bool,
    collapse: bool,
) -> LongHorizonClass {
    if collapse {
        return LongHorizonClass::BiologicalCollapse;
    }
    if !converged {
        return LongHorizonClass::NotConverged;
    }
    if predicted_theta_eq + 1e-6 < contract_occupancy
        && (final_occupancy - predicted_theta_eq).abs() <= 0.05
    {
        return LongHorizonClass::EquilibriumBelowContract;
    }
    if final_occupancy + 0.02 < initial_occupancy && final_occupancy + 0.02 < contract_occupancy {
        return LongHorizonClass::SlowTransientDecay;
    }
    if final_occupancy + 1e-6 >= contract_occupancy.min(predicted_theta_eq) {
        return LongHorizonClass::TrueMaintenance;
    }
    LongHorizonClass::SlowTransientDecay
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D073PrimaryConclusion {
    D072ControlInsufficient,
    ShortHorizonMembraneQualificationDefect,
    LocalPrecursorDeliveryLimit,
    ExchangeEquilibriumMetabolicallyUnreachable,
    OrganismExchangeIntegrationDefect,
    FrozenExchangeEquilibriumIncompatible,
    AccountingFailure,
    NumericalFailure,
}

impl D073PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::D072ControlInsufficient => "D073_D072_CONTROL_INSUFFICIENT",
            Self::ShortHorizonMembraneQualificationDefect => {
                "D073_SHORT_HORIZON_MEMBRANE_QUALIFICATION_DEFECT"
            }
            Self::LocalPrecursorDeliveryLimit => "D073_LOCAL_PRECURSOR_DELIVERY_LIMIT",
            Self::ExchangeEquilibriumMetabolicallyUnreachable => {
                "D073_EXCHANGE_EQUILIBRIUM_METABOLICALLY_UNREACHABLE"
            }
            Self::OrganismExchangeIntegrationDefect => "D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT",
            Self::FrozenExchangeEquilibriumIncompatible => {
                "D073_FROZEN_EXCHANGE_EQUILIBRIUM_INCOMPATIBLE"
            }
            Self::AccountingFailure => "D073_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D073_NUMERICAL_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D073Route {
    C,
    T,
    L,
    M,
    E,
    X,
    StopAccounting,
    StopNumerical,
}

impl D073Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::C => "Route_C_invalid_d072_control",
            Self::T => "Route_T_short_horizon_qualification_defect",
            Self::L => "Route_L_local_delivery_failure",
            Self::M => "Route_M_metabolic_precursor_insufficiency",
            Self::E => "Route_E_organism_integration_defect",
            Self::X => "Route_X_genuine_exchange_architecture_incompatibility",
            Self::StopAccounting => "Stop_accounting",
            Self::StopNumerical => "Stop_numerical",
        }
    }

    pub const fn conclusion(self) -> D073PrimaryConclusion {
        match self {
            Self::C => D073PrimaryConclusion::D072ControlInsufficient,
            Self::T => D073PrimaryConclusion::ShortHorizonMembraneQualificationDefect,
            Self::L => D073PrimaryConclusion::LocalPrecursorDeliveryLimit,
            Self::M => D073PrimaryConclusion::ExchangeEquilibriumMetabolicallyUnreachable,
            Self::E => D073PrimaryConclusion::OrganismExchangeIntegrationDefect,
            Self::X => D073PrimaryConclusion::FrozenExchangeEquilibriumIncompatible,
            Self::StopAccounting => D073PrimaryConclusion::AccountingFailure,
            Self::StopNumerical => D073PrimaryConclusion::NumericalFailure,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RouteEvidence073 {
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub d072_control_class: FixedPControlClass,
    pub target_consistent_fixed_p_valid: bool,
    pub sufficient_fixed_p_repairs: bool,
    pub exchange_only_sufficient_repairs: bool,
    pub long_horizon_class: LongHorizonClass,
    pub endogenous_interface_p_sufficient_095: bool,
    pub total_p_mass_large: bool,
    pub redistribution_raises_interface_p: bool,
    pub redistribution_repairs: bool,
    pub a_collapses_under_endogenous: bool,
    pub runtime_analytical_eq_agree: bool,
    pub d072_route_x_original: bool,
}

impl Default for RouteEvidence073 {
    fn default() -> Self {
        Self {
            accounting_ok: true,
            numerical_ok: true,
            d072_control_class: FixedPControlClass::Unknown,
            target_consistent_fixed_p_valid: false,
            sufficient_fixed_p_repairs: false,
            exchange_only_sufficient_repairs: false,
            long_horizon_class: LongHorizonClass::NotConverged,
            endogenous_interface_p_sufficient_095: false,
            total_p_mass_large: false,
            redistribution_raises_interface_p: false,
            redistribution_repairs: false,
            a_collapses_under_endogenous: false,
            runtime_analytical_eq_agree: false,
            d072_route_x_original: true,
        }
    }
}

/// Gate-7 causal route selection (exactly one primary).
///
/// Priority: stops → C → L → M → E → T → X.
/// Route C takes precedence when an invalid D-072 control is overturned by a
/// target-consistent sufficient fixed-P repair.
pub fn select_route(ev: RouteEvidence073) -> D073Route {
    if !ev.numerical_ok {
        return D073Route::StopNumerical;
    }
    if !ev.accounting_ok {
        return D073Route::StopAccounting;
    }

    let d072_control_invalid = matches!(
        ev.d072_control_class,
        FixedPControlClass::TargetInsufficient
            | FixedPControlClass::NotActuallyFixed
            | FixedPControlClass::SpatiallyIncomplete
    );

    // Route C — invalid D-072 sufficiency control overturned by true fixed-P repair.
    if d072_control_invalid && ev.target_consistent_fixed_p_valid && ev.sufficient_fixed_p_repairs {
        return D073Route::C;
    }

    // Route L — bulk P present; interface starved; conservative redistribution repairs.
    if ev.total_p_mass_large
        && !ev.endogenous_interface_p_sufficient_095
        && ev.redistribution_raises_interface_p
        && ev.redistribution_repairs
    {
        return D073Route::L;
    }

    // Route M — sufficient fixed P repairs, but endogenous interface p is unreachable
    // (with or without measured A collapse).
    if ev.sufficient_fixed_p_repairs && !ev.endogenous_interface_p_sufficient_095 {
        return D073Route::M;
    }

    // Route E — analytically sufficient interface hold is valid, yet organism fails.
    if ev.target_consistent_fixed_p_valid
        && ev.runtime_analytical_eq_agree
        && !ev.sufficient_fixed_p_repairs
    {
        return D073Route::E;
    }

    // Route T — short-horizon maintenance qualification defect.
    if matches!(
        ev.long_horizon_class,
        LongHorizonClass::SlowTransientDecay | LongHorizonClass::EquilibriumBelowContract
    ) {
        return D073Route::T;
    }

    // Route X — genuine frozen-exchange equilibrium incompatibility (last resort).
    let _ = ev.d072_route_x_original;
    D073Route::X
}

/// Whether a measured interface activity stays within 2% of the intended hold.
#[inline]
pub fn interface_p_within_tol(measured: f64, intended: f64) -> bool {
    let denom = intended.abs().max(EPS);
    ((measured - intended).abs() / denom) <= P_HOLD_TOL_FRAC
}

/// Analytical/runtime equilibrium parity helper.
#[inline]
pub fn eq_parity_ok(analytical: f64, runtime: f64, tol: f64) -> bool {
    (analytical - runtime).abs() <= tol
        || ((analytical - runtime).abs() / analytical.abs().max(EPS) <= tol)
}

/// Preserve D-072 original conclusion string identity.
pub fn d072_original_preserved(reported: &str) -> bool {
    reported == D072_ORIGINAL_CONCLUSION
        || reported == D072PrimaryConclusion::FrozenExchangeCannotRefillDamage.as_str()
}

#[cfg(test)]
mod local_checks {
    use super::*;

    #[test]
    fn p_required_identities_keq50() {
        assert!((p_required(0.90, 50.0) - 0.18).abs() < 1e-12);
        assert!((p_required(0.95, 50.0) - 0.38).abs() < 1e-12);
    }
}
