//! D-069 mature-membrane exchange equilibrium and desorption audit helpers.
//!
//! Observer/shadow-only diagnostics. Frozen precursor production and activation
//! are preserved. Production exchange defaults are not changed.

use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::surface_density::{
    precursor_activity, surface_occupancy_theta, EXCHANGE_BOUND_TOLERANCE,
};
use serde::{Deserialize, Serialize};

pub const D069_PROJECT_ID: &str = "D-069";
pub const D069_AGENT_MEMORY_ID: &str =
    "D-20260722-1114-d069-mature-membrane-exchange-desorption-audit";
pub const D069_STARTING_COMMIT: &str = "3b16792";
pub const D069_STARTING_TAG: &str = "D-068-precursor-demand-membrane-assembly-audit";
pub const D068_CONCLUSION: &str = "D068_MEMBRANE_DESORPTION_DOMINANT";
pub const PRECURSOR_SUPPLY_NOT_PRIMARY_MEMBRANE_LIMIT: &str =
    "PRECURSOR_SUPPLY_NOT_PRIMARY_MEMBRANE_LIMIT";
pub const REVERSE_MEMBRANE_EXCHANGE_CAUSE_UNRESOLVED: &str =
    "REVERSE_MEMBRANE_EXCHANGE_CAUSE_UNRESOLVED";

pub const D069_FROZEN_KT: f64 = 1.4346157818803311;
pub const D069_K_EXCHANGE: f64 = D031_BETA_FROZEN;
pub const D069_K_EQ: f64 = D031_ALPHA_FROZEN / D031_BETA_FROZEN;
pub const D069_P_REF: f64 = 1.0;
pub const D069_GAMMA_MAX: f64 = 1.0;

pub const S_RETENTION: f64 = 0.80;
pub const A_RETENTION: f64 = 0.80;
pub const C_RETENTION: f64 = 0.80;
pub const CHI_S_TARGET: f64 = 1.00;
pub const PORTABLE_SPAN_MAX: f64 = 3.0;
pub const BOOTSTRAP_SPREAD_MAX: f64 = 0.50;
pub const LOO_MAX: f64 = 2.0;
pub const HOLDOUT_MEDIAN_ERR: f64 = 0.20;
pub const HOLDOUT_MAX_ERR: f64 = 0.35;
pub const DIRECTION_ACC: f64 = 0.90;
pub const EQ_OCC_ERR_PP: f64 = 0.10;
pub const LEDGER_TOL: f64 = 1e-6;
pub const EPS: f64 = 1e-18;
pub const NEAR_EQ_TOL: f64 = 1e-3;

pub const EXCHANGE_EQUATION: &str =
    "dS/dt = δ · k_exchange · q(C) · Γ_max · (K_eq · p · (1−θ) − θ)";
pub const ADS_REQ_EQUATION: &str =
    "J_ads^req = δ · k_exchange · q(C) · Γ_max · K_eq · p · (1−θ)";
pub const DES_REQ_EQUATION: &str =
    "J_des^req = δ · k_exchange · q(C) · Γ_max · θ";
pub const P_DEFINITION: &str = "p = P / P_reference (dimensionless precursor activity)";
pub const THETA_DEFINITION: &str = "θ = S / (δ · Γ_max) = Γ / Γ_max (dimensionless occupancy)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D069PrimaryConclusion {
    MembraneExchangeExecutionDefect,
    MembraneEquilibriumCalibrationQualified,
    ReversibleMembraneExchangeLawQualified,
    MembraneExchangeRepairedPrecursorOverproductionRemains,
    MembraneExchangeTimescaleCalibrationQualified,
    NoPortableMembraneExchangeLaw,
    WasteExecutionBlocksMembraneQualification,
    ExistingMembraneExchangeQualified,
    MembraneExchangeAuditInconclusive,
    D068DesorptionResultNotReproduced,
    MembraneExchangeLineageOrUnitsFailure,
    ExchangeDirectionOrRuntimeParityFailure,
    ExchangeEquilibriumRuntimeMismatch,
    ExchangeSurfaceNormalizationDefect,
    MembraneExchangeCausalityFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D069PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MembraneExchangeExecutionDefect => "D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT",
            Self::MembraneEquilibriumCalibrationQualified => {
                "D069_MEMBRANE_EQUILIBRIUM_CALIBRATION_QUALIFIED"
            }
            Self::ReversibleMembraneExchangeLawQualified => {
                "D069_REVERSIBLE_MEMBRANE_EXCHANGE_LAW_QUALIFIED"
            }
            Self::MembraneExchangeRepairedPrecursorOverproductionRemains => {
                "D069_MEMBRANE_EXCHANGE_REPAIRED_PRECURSOR_OVERPRODUCTION_REMAINS"
            }
            Self::MembraneExchangeTimescaleCalibrationQualified => {
                "D069_MEMBRANE_EXCHANGE_TIMESCALE_CALIBRATION_QUALIFIED"
            }
            Self::NoPortableMembraneExchangeLaw => "D069_NO_PORTABLE_MEMBRANE_EXCHANGE_LAW",
            Self::WasteExecutionBlocksMembraneQualification => {
                "D069_WASTE_EXECUTION_BLOCKS_MEMBRANE_QUALIFICATION"
            }
            Self::ExistingMembraneExchangeQualified => "D069_EXISTING_MEMBRANE_EXCHANGE_QUALIFIED",
            Self::MembraneExchangeAuditInconclusive => "D069_MEMBRANE_EXCHANGE_AUDIT_INCONCLUSIVE",
            Self::D068DesorptionResultNotReproduced => "D069_D068_DESORPTION_RESULT_NOT_REPRODUCED",
            Self::MembraneExchangeLineageOrUnitsFailure => {
                "D069_MEMBRANE_EXCHANGE_LINEAGE_OR_UNITS_FAILURE"
            }
            Self::ExchangeDirectionOrRuntimeParityFailure => {
                "D069_EXCHANGE_DIRECTION_OR_RUNTIME_PARITY_FAILURE"
            }
            Self::ExchangeEquilibriumRuntimeMismatch => {
                "D069_EXCHANGE_EQUILIBRIUM_RUNTIME_MISMATCH"
            }
            Self::ExchangeSurfaceNormalizationDefect => {
                "D069_EXCHANGE_SURFACE_NORMALIZATION_DEFECT"
            }
            Self::MembraneExchangeCausalityFailure => "D069_MEMBRANE_EXCHANGE_CAUSALITY_FAILURE",
            Self::WorkspaceScopeNotIsolated => "D069_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D069_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D069_NUMERICAL_FAILURE",
            Self::Fail => "D069_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D069Route {
    X,
    E,
    R,
    P,
    T,
    N,
    W,
    Q,
    I,
}

impl D069Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "Route_X_exchange_execution_defect",
            Self::E => "Route_E_equilibrium_calibration_qualifies",
            Self::R => "Route_R_explicit_reversible_rates_qualify",
            Self::P => "Route_P_exchange_repaired_precursor_overproduction_remains",
            Self::T => "Route_T_exchange_timescale_only",
            Self::N => "Route_N_no_portable_exchange_law",
            Self::W => "Route_W_waste_execution_blocks_qualification",
            Self::Q => "Route_Q_existing_exchange_qualifies",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D069PrimaryConclusion {
        match self {
            Self::X => D069PrimaryConclusion::MembraneExchangeExecutionDefect,
            Self::E => D069PrimaryConclusion::MembraneEquilibriumCalibrationQualified,
            Self::R => D069PrimaryConclusion::ReversibleMembraneExchangeLawQualified,
            Self::P => D069PrimaryConclusion::MembraneExchangeRepairedPrecursorOverproductionRemains,
            Self::T => D069PrimaryConclusion::MembraneExchangeTimescaleCalibrationQualified,
            Self::N => D069PrimaryConclusion::NoPortableMembraneExchangeLaw,
            Self::W => D069PrimaryConclusion::WasteExecutionBlocksMembraneQualification,
            Self::Q => D069PrimaryConclusion::ExistingMembraneExchangeQualified,
            Self::I => D069PrimaryConclusion::MembraneExchangeAuditInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquilibriumManifoldClass {
    MembraneNearExchangeEquilibrium,
    MembraneSystematicallyBelowRequiredP,
    MembraneSystematicallyAboveEquilibriumOccupancy,
    MembraneExchangeStateHeterogeneous,
    MembraneEquilibriumUnresolved,
}

impl EquilibriumManifoldClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MembraneNearExchangeEquilibrium => "MEMBRANE_NEAR_EXCHANGE_EQUILIBRIUM",
            Self::MembraneSystematicallyBelowRequiredP => {
                "MEMBRANE_SYSTEMATICALLY_BELOW_REQUIRED_P"
            }
            Self::MembraneSystematicallyAboveEquilibriumOccupancy => {
                "MEMBRANE_SYSTEMATICALLY_ABOVE_EQUILIBRIUM_OCCUPANCY"
            }
            Self::MembraneExchangeStateHeterogeneous => "MEMBRANE_EXCHANGE_STATE_HETEROGENEOUS",
            Self::MembraneEquilibriumUnresolved => "MEMBRANE_EQUILIBRIUM_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimescaleClass {
    ExchangeTimescalePortable,
    ExchangeTooFast,
    ExchangeTooSlow,
    ExchangeTimescaleNonportable,
    ExchangeTimescaleNotPrimary,
}

impl TimescaleClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExchangeTimescalePortable => "EXCHANGE_TIMESCALE_PORTABLE",
            Self::ExchangeTooFast => "EXCHANGE_TOO_FAST",
            Self::ExchangeTooSlow => "EXCHANGE_TOO_SLOW",
            Self::ExchangeTimescaleNonportable => "EXCHANGE_TIMESCALE_NONPORTABLE",
            Self::ExchangeTimescaleNotPrimary => "EXCHANGE_TIMESCALE_NOT_PRIMARY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrecursorFeasibilityClass {
    CurrentEquilibriumPrecursorFeasible,
    CurrentEquilibriumRequiresExcessPrecursor,
    CurrentEquilibriumMateriallyImpossible,
    PrecursorThresholdInconclusive,
}

impl PrecursorFeasibilityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CurrentEquilibriumPrecursorFeasible => "CURRENT_EQUILIBRIUM_PRECURSOR_FEASIBLE",
            Self::CurrentEquilibriumRequiresExcessPrecursor => {
                "CURRENT_EQUILIBRIUM_REQUIRES_EXCESS_PRECURSOR"
            }
            Self::CurrentEquilibriumMateriallyImpossible => {
                "CURRENT_EQUILIBRIUM_MATERIALLY_IMPOSSIBLE"
            }
            Self::PrecursorThresholdInconclusive => "PRECURSOR_THRESHOLD_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExchangeCandidate {
    Baseline,
    GlobalKeq,
    ExplicitOnOff,
}

impl ExchangeCandidate {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "CANDIDATE_A_FROZEN_EXCHANGE",
            Self::GlobalKeq => "CANDIDATE_B_GLOBAL_KEQ",
            Self::ExplicitOnOff => "CANDIDATE_C_EXPLICIT_ON_OFF",
        }
    }
}

/// Local precursor activity p = P / P_ref.
#[inline]
pub fn p_activity(precursor: f64, p_reference: f64) -> f64 {
    precursor_activity(precursor, p_reference)
}

/// Occupancy θ = S / (δ · Γ_max).
#[inline]
pub fn theta_occupancy(surface: f64, delta: f64, gamma_max: f64) -> f64 {
    let c_surface = delta.max(0.0) * gamma_max.max(0.0);
    if c_surface <= 0.0 {
        0.0
    } else {
        surface_occupancy_theta(surface / delta.max(EPS), gamma_max)
            .min(1.0 + EXCHANGE_BOUND_TOLERANCE)
    }
}

/// Occupancy from surface density Γ.
#[inline]
pub fn theta_from_gamma(gamma: f64, gamma_max: f64) -> f64 {
    surface_occupancy_theta(gamma, gamma_max)
}

/// Analytical equilibrium precursor activity for given occupancy.
#[inline]
pub fn p_eq(theta: f64, k_eq: f64) -> f64 {
    let t = theta.clamp(0.0, 1.0 - 1e-15);
    if k_eq <= 0.0 || (1.0 - t) <= 0.0 {
        return f64::INFINITY;
    }
    t / (k_eq * (1.0 - t))
}

/// Analytical equilibrium occupancy for given precursor activity.
#[inline]
pub fn theta_eq(p: f64, k_eq: f64) -> f64 {
    let kp = k_eq.max(0.0) * p.max(0.0);
    kp / (1.0 + kp)
}

/// Required K_eq★ for zero net exchange at (p, θ).
#[inline]
pub fn k_eq_star(theta: f64, p: f64) -> f64 {
    let t = theta.clamp(0.0, 1.0 - 1e-15);
    let free = (1.0 - t).max(EPS);
    if p <= EPS {
        return f64::INFINITY;
    }
    t / (p * free)
}

/// q(C) catalyst saturation (same as production membrane exchange).
#[inline]
pub fn q_c(catalyst: f64, k_c: f64) -> f64 {
    membrane_catalyst_saturation_local(catalyst, k_c)
}

#[inline]
fn membrane_catalyst_saturation_local(c: f64, k_c: f64) -> f64 {
    let c = c.max(0.0);
    c / (k_c.max(EPS) + c)
}

/// Requested adsorption rate component (dS/dt units before accept bound).
#[inline]
pub fn j_ads_req(
    delta: f64,
    k_exchange: f64,
    q: f64,
    gamma_max: f64,
    k_eq: f64,
    p: f64,
    theta: f64,
) -> f64 {
    delta.max(0.0)
        * k_exchange.max(0.0)
        * q.max(0.0)
        * gamma_max.max(0.0)
        * k_eq.max(0.0)
        * p.max(0.0)
        * (1.0 - theta).max(0.0)
}

/// Requested desorption rate component.
#[inline]
pub fn j_des_req(delta: f64, k_exchange: f64, q: f64, gamma_max: f64, theta: f64) -> f64 {
    delta.max(0.0)
        * k_exchange.max(0.0)
        * q.max(0.0)
        * gamma_max.max(0.0)
        * theta.clamp(0.0, 1.0)
}

/// Requested net exchange dS/dt.
#[inline]
pub fn j_net_req(
    delta: f64,
    k_exchange: f64,
    q: f64,
    gamma_max: f64,
    k_eq: f64,
    p: f64,
    theta: f64,
) -> f64 {
    j_ads_req(delta, k_exchange, q, gamma_max, k_eq, p, theta)
        - j_des_req(delta, k_exchange, q, gamma_max, theta)
}

/// Candidate C: explicit on/off rates with detailed balance K = k_on/k_off.
#[inline]
pub fn j_net_on_off(
    delta: f64,
    q: f64,
    gamma_max: f64,
    k_on: f64,
    k_off: f64,
    p: f64,
    theta: f64,
) -> f64 {
    delta.max(0.0)
        * q.max(0.0)
        * gamma_max.max(0.0)
        * (k_on.max(0.0) * p.max(0.0) * (1.0 - theta).max(0.0)
            - k_off.max(0.0) * theta.clamp(0.0, 1.0))
}

/// Nested special case: k_on = k_exchange·K_eq, k_off = k_exchange.
#[inline]
pub fn nested_on_off(k_exchange: f64, k_eq: f64) -> (f64, f64) {
    (k_exchange.max(0.0) * k_eq.max(0.0), k_exchange.max(0.0))
}

/// Signed distance from equilibrium: K_eq p (1−θ) − θ. Positive ⇒ adsorption-favored.
#[inline]
pub fn signed_eq_distance(k_eq: f64, p: f64, theta: f64) -> f64 {
    k_eq.max(0.0) * p.max(0.0) * (1.0 - theta).max(0.0) - theta.clamp(0.0, 1.0)
}

/// Runtime accepted-direction accounting: ΔP = −ξ, ΔS = +ξ.
#[inline]
pub fn accepted_exchange_parity(xi_acc: f64, delta_p: f64, delta_s: f64) -> bool {
    (delta_p + xi_acc).abs() <= 1e-12 * (1.0 + xi_acc.abs())
        && (delta_s - xi_acc).abs() <= 1e-12 * (1.0 + xi_acc.abs())
}

/// Split signed accepted exchange into directional extents (not simultaneous transfers).
#[inline]
pub fn split_accepted_exchange(xi_acc: f64) -> (f64, f64) {
    if xi_acc >= 0.0 {
        (xi_acc, 0.0)
    } else {
        (0.0, -xi_acc)
    }
}

/// D-068 Gate0 reproduction: desorption ≫ adsorption, low η, S loss, fixed-P non-rescue.
/// True when accepted desorption is accounted for by initial S above Σδ·Γ_max.
#[inline]
pub fn desorption_explained_by_over_capacity(
    des: f64,
    over_capacity_mass: f64,
    s_over_capacity_ratio: f64,
) -> bool {
    s_over_capacity_ratio > 1.05
        && des > 1.0
        && (des - over_capacity_mass).abs() <= 0.05 * (1.0 + des.max(over_capacity_mass))
}

pub fn d068_desorption_reproduction(
    ads: f64,
    des: f64,
    syn_p: f64,
    s_ret: f64,
    fixed_p_s_ret: f64,
    eta: f64,
) -> bool {
    des > ads * 5.0
        && des > 20.0
        && syn_p > 50.0
        && eta < 0.05
        && s_ret < 0.70
        && fixed_p_s_ret < 0.70
}

#[inline]
pub fn eta_p_to_s(ads: f64, syn_p: f64) -> f64 {
    ads / syn_p.max(EPS)
}

/// Dimensional factor table for Gate 1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DimensionalFactor {
    pub name: String,
    pub symbol: String,
    pub units: String,
    pub notes: String,
}

pub fn dimensional_table() -> Vec<DimensionalFactor> {
    vec![
        DimensionalFactor {
            name: "interface_measure".into(),
            symbol: "δ".into(),
            units: "length (interface thickness measure)".into(),
            notes: "applied exactly once as prefactor on J".into(),
        },
        DimensionalFactor {
            name: "exchange_rate".into(),
            symbol: "k_exchange".into(),
            units: "1/time".into(),
            notes: "equals frozen β = k_off".into(),
        },
        DimensionalFactor {
            name: "catalyst_response".into(),
            symbol: "q(C)".into(),
            units: "dimensionless".into(),
            notes: "C/(K_C+C); scales rate, not equilibrium".into(),
        },
        DimensionalFactor {
            name: "surface_capacity_density".into(),
            symbol: "Γ_max".into(),
            units: "surface density (mass/area)".into(),
            notes: "local capacity density; C_surface=δ·Γ_max".into(),
        },
        DimensionalFactor {
            name: "equilibrium_constant".into(),
            symbol: "K_eq".into(),
            units: "dimensionless (1/activity)".into(),
            notes: "α/β with α=k_on, β=k_off".into(),
        },
        DimensionalFactor {
            name: "precursor_activity".into(),
            symbol: "p".into(),
            units: "dimensionless".into(),
            notes: "P/P_reference; not raw concentration in the driving affinity".into(),
        },
        DimensionalFactor {
            name: "occupancy".into(),
            symbol: "θ".into(),
            units: "dimensionless".into(),
            notes: "S/(δ·Γ_max) ∈ [0,1]".into(),
        },
        DimensionalFactor {
            name: "net_exchange_rate".into(),
            symbol: "dS/dt".into(),
            units: "mass/time (volumetric cell update)".into(),
            notes: "δ·k·q·Γ_max·(K p(1−θ)−θ)".into(),
        },
    ]
}

pub fn classify_equilibrium_manifold(
    frac_ads_favored: f64,
    frac_near: f64,
    frac_des_favored: f64,
    median_signed_distance: f64,
) -> EquilibriumManifoldClass {
    if !(frac_ads_favored.is_finite() && frac_des_favored.is_finite() && frac_near.is_finite()) {
        return EquilibriumManifoldClass::MembraneEquilibriumUnresolved;
    }
    if frac_near >= 0.60 {
        return EquilibriumManifoldClass::MembraneNearExchangeEquilibrium;
    }
    if frac_des_favored >= 0.70 && median_signed_distance < 0.0 {
        // Desorption-favored can be either low p or high θ; prefer p-deficit when distance is from p.
        if frac_ads_favored <= 0.15 {
            return EquilibriumManifoldClass::MembraneSystematicallyBelowRequiredP;
        }
        return EquilibriumManifoldClass::MembraneSystematicallyAboveEquilibriumOccupancy;
    }
    if frac_ads_favored >= 0.70 && median_signed_distance > 0.0 {
        return EquilibriumManifoldClass::MembraneNearExchangeEquilibrium;
    }
    if (frac_ads_favored - frac_des_favored).abs() < 0.25 && frac_near < 0.40 {
        return EquilibriumManifoldClass::MembraneExchangeStateHeterogeneous;
    }
    EquilibriumManifoldClass::MembraneEquilibriumUnresolved
}

pub fn classify_timescale(
    eq_placement_ok: bool,
    final_occ_ok: bool,
    approach_too_fast: bool,
    approach_too_slow: bool,
    one_global_k_explains: bool,
) -> TimescaleClass {
    if !eq_placement_ok {
        return TimescaleClass::ExchangeTimescaleNotPrimary;
    }
    if !final_occ_ok {
        return TimescaleClass::ExchangeTimescaleNotPrimary;
    }
    if !one_global_k_explains {
        return TimescaleClass::ExchangeTimescaleNonportable;
    }
    if approach_too_fast {
        return TimescaleClass::ExchangeTooFast;
    }
    if approach_too_slow {
        return TimescaleClass::ExchangeTooSlow;
    }
    TimescaleClass::ExchangeTimescalePortable
}

pub fn classify_precursor_feasibility(
    p_required: f64,
    p_current: f64,
    p_fixed_healthy: f64,
    p_from_available_a: f64,
    volume_ok: bool,
) -> PrecursorFeasibilityClass {
    if !volume_ok || p_required.is_nan() || p_required < 0.0 {
        return PrecursorFeasibilityClass::PrecursorThresholdInconclusive;
    }
    if p_required.is_infinite() || p_required > 1e6 {
        return PrecursorFeasibilityClass::CurrentEquilibriumMateriallyImpossible;
    }
    if p_current + EPS >= p_required || p_fixed_healthy + EPS >= p_required {
        return PrecursorFeasibilityClass::CurrentEquilibriumPrecursorFeasible;
    }
    if p_from_available_a + EPS >= p_required {
        return PrecursorFeasibilityClass::CurrentEquilibriumRequiresExcessPrecursor;
    }
    if p_required > p_from_available_a.max(p_fixed_healthy).max(p_current) * 10.0 {
        return PrecursorFeasibilityClass::CurrentEquilibriumMateriallyImpossible;
    }
    PrecursorFeasibilityClass::CurrentEquilibriumRequiresExcessPrecursor
}

/// Portability of K_eq★: span ≤3×, no systematic radius/catalyst dependence.
pub fn keq_star_portable(
    values: &[f64],
    radius_spread: f64,
    catalyst_spread: f64,
    bootstrap_spread: f64,
    loo_variation: f64,
) -> bool {
    let finite: Vec<f64> = values
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    if finite.len() < 2 {
        return false;
    }
    let min_v = finite.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = finite.iter().cloned().fold(0.0_f64, f64::max);
    let span = max_v / min_v.max(EPS);
    span <= PORTABLE_SPAN_MAX
        && radius_spread <= PORTABLE_SPAN_MAX
        && catalyst_spread <= PORTABLE_SPAN_MAX
        && bootstrap_spread <= BOOTSTRAP_SPREAD_MAX
        && loo_variation <= LOO_MAX
}

pub fn span_ratio(values: &[f64]) -> f64 {
    let finite: Vec<f64> = values
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    if finite.is_empty() {
        return f64::NAN;
    }
    let min_v = finite.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = finite.iter().cloned().fold(0.0_f64, f64::max);
    max_v / min_v.max(EPS)
}

/// Dose-response zero crossing should match p_eq(θ).
pub fn zero_crossing_matches(p_cross: f64, p_eq_analytical: f64, rel_tol: f64) -> bool {
    if !p_cross.is_finite() || !p_eq_analytical.is_finite() {
        return false;
    }
    (p_cross - p_eq_analytical).abs()
        <= rel_tol * (1.0 + p_eq_analytical.abs().max(p_cross.abs()))
}

/// Surface normalization: integrated exchange scales with interface measure.
pub fn surface_scale_ok(j_at_d: f64, j_at_2d: f64, rel_tol: f64) -> bool {
    if j_at_d.abs() <= EPS {
        return j_at_2d.abs() <= EPS;
    }
    ((j_at_2d / j_at_d) - 2.0).abs() <= rel_tol
}

/// Concentration update inverse-volume scaling.
pub fn volume_scale_ok(dp_at_v: f64, dp_at_2v: f64, rel_tol: f64) -> bool {
    if dp_at_v.abs() <= EPS {
        return dp_at_2v.abs() <= EPS;
    }
    ((dp_at_2v / dp_at_v) - 0.5).abs() <= rel_tol
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentificationReport069 {
    pub params_positive_finite: bool,
    pub bootstrap_spread: f64,
    pub loo_variation: f64,
    pub holdout_median_err: f64,
    pub holdout_max_err: f64,
    pub direction_accuracy: f64,
    pub eq_occupancy_err_pp: f64,
    pub no_radius_params: bool,
    pub accounting_ok: bool,
    pub predicts_damage_adsorption: bool,
    pub predicts_zero_p_desorption: bool,
}

impl IdentificationReport069 {
    pub fn qualifies(&self) -> bool {
        self.params_positive_finite
            && self.bootstrap_spread <= BOOTSTRAP_SPREAD_MAX
            && self.loo_variation <= LOO_MAX
            && self.holdout_median_err <= HOLDOUT_MEDIAN_ERR
            && self.holdout_max_err <= HOLDOUT_MAX_ERR
            && self.direction_accuracy >= DIRECTION_ACC
            && self.eq_occupancy_err_pp <= EQ_OCC_ERR_PP
            && self.no_radius_params
            && self.accounting_ok
            && self.predicts_damage_adsorption
            && self.predicts_zero_p_desorption
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence069 {
    pub workspace_isolated: bool,
    pub d068_reproduced: bool,
    pub lineage_ok: bool,
    pub direction_parity_ok: bool,
    pub equilibrium_runtime_ok: bool,
    pub surface_normalization_ok: bool,
    pub accounting_ok: bool,
    pub causality_ok: bool,
    pub waste_blocks: bool,
    pub identification: IdentificationReport069,
    pub existing_qualified: bool,
    pub keq_calibration_qualified: bool,
    pub on_off_qualified: bool,
    pub timescale_only_qualified: bool,
    pub s_repairs_a_fails: bool,
    pub no_portable_law: bool,
    pub execution_defect: bool,
}

pub fn select_route(ev: RouteEvidence069) -> (D069Route, D069PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D069Route::I,
            D069PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d068_reproduced {
        return (
            D069Route::I,
            D069PrimaryConclusion::D068DesorptionResultNotReproduced,
        );
    }
    if !ev.lineage_ok {
        return (
            D069Route::I,
            D069PrimaryConclusion::MembraneExchangeLineageOrUnitsFailure,
        );
    }
    if !ev.direction_parity_ok || ev.execution_defect {
        return (
            D069Route::X,
            D069PrimaryConclusion::MembraneExchangeExecutionDefect,
        );
    }
    if !ev.equilibrium_runtime_ok {
        return (
            D069Route::I,
            D069PrimaryConclusion::ExchangeEquilibriumRuntimeMismatch,
        );
    }
    if !ev.surface_normalization_ok {
        return (
            D069Route::X,
            D069PrimaryConclusion::ExchangeSurfaceNormalizationDefect,
        );
    }
    if !ev.accounting_ok {
        return (D069Route::I, D069PrimaryConclusion::AccountingFailure);
    }
    if ev.waste_blocks {
        return (
            D069Route::W,
            D069PrimaryConclusion::WasteExecutionBlocksMembraneQualification,
        );
    }
    if !ev.causality_ok {
        return (
            D069Route::I,
            D069PrimaryConclusion::MembraneExchangeCausalityFailure,
        );
    }
    if ev.existing_qualified {
        return (
            D069Route::Q,
            D069PrimaryConclusion::ExistingMembraneExchangeQualified,
        );
    }
    if ev.keq_calibration_qualified && ev.identification.qualifies() {
        return (
            D069Route::E,
            D069PrimaryConclusion::MembraneEquilibriumCalibrationQualified,
        );
    }
    if ev.on_off_qualified && ev.identification.qualifies() {
        return (
            D069Route::R,
            D069PrimaryConclusion::ReversibleMembraneExchangeLawQualified,
        );
    }
    if ev.timescale_only_qualified && ev.identification.qualifies() {
        return (
            D069Route::T,
            D069PrimaryConclusion::MembraneExchangeTimescaleCalibrationQualified,
        );
    }
    if ev.s_repairs_a_fails {
        return (
            D069Route::P,
            D069PrimaryConclusion::MembraneExchangeRepairedPrecursorOverproductionRemains,
        );
    }
    if ev.no_portable_law {
        return (
            D069Route::N,
            D069PrimaryConclusion::NoPortableMembraneExchangeLaw,
        );
    }
    (
        D069Route::I,
        D069PrimaryConclusion::MembraneExchangeAuditInconclusive,
    )
}

/// Frozen lineage snapshot for Gate 1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeLineage069 {
    pub equation: String,
    pub p_definition: String,
    pub theta_definition: String,
    pub k_exchange: f64,
    pub k_eq: f64,
    pub p_reference: f64,
    pub gamma_max: f64,
    pub alpha: f64,
    pub beta: f64,
    pub p_is_normalized_activity: bool,
    pub theta_is_dimensionless: bool,
    pub gamma_max_is_density: bool,
    pub delta_applied_once: bool,
}

pub fn frozen_exchange_lineage() -> ExchangeLineage069 {
    ExchangeLineage069 {
        equation: EXCHANGE_EQUATION.into(),
        p_definition: P_DEFINITION.into(),
        theta_definition: THETA_DEFINITION.into(),
        k_exchange: D069_K_EXCHANGE,
        k_eq: D069_K_EQ,
        p_reference: D069_P_REF,
        gamma_max: D069_GAMMA_MAX,
        alpha: D031_ALPHA_FROZEN,
        beta: D031_BETA_FROZEN,
        p_is_normalized_activity: true,
        theta_is_dimensionless: true,
        gamma_max_is_density: true,
        delta_applied_once: true,
    }
}

pub fn lineage_resolved(lin: &ExchangeLineage069) -> bool {
    lin.k_exchange.is_finite()
        && lin.k_exchange > 0.0
        && lin.k_eq.is_finite()
        && lin.k_eq > 0.0
        && lin.p_is_normalized_activity
        && lin.theta_is_dimensionless
        && lin.gamma_max_is_density
        && lin.delta_applied_once
        && (lin.k_exchange - lin.beta).abs() <= 1e-15
        && (lin.k_eq - lin.alpha / lin.beta).abs() <= 1e-9
}

/// Linear relaxation time near equilibrium: τ ≈ 1 / [k_exchange q Γ_max δ · K_eq p / C_surface scale].
/// For local S with C_surface = δ Γ_max: dθ/dt = k q (K p (1−θ) − θ), so
/// τ = 1 / [k q (K p + 1)] at fixed p.
#[inline]
pub fn tau_exchange(k_exchange: f64, q: f64, k_eq: f64, p: f64) -> f64 {
    let rate = k_exchange.max(0.0) * q.max(0.0) * (k_eq.max(0.0) * p.max(0.0) + 1.0);
    if rate <= EPS {
        f64::INFINITY
    } else {
        1.0 / rate
    }
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn equilibrium_identities() {
        let k = 50.0;
        let p = 0.1;
        let t = theta_eq(p, k);
        assert!((p_eq(t, k) - p).abs() < 1e-12);
        let star = k_eq_star(t, p);
        assert!((star - k).abs() < 1e-9);
    }

    #[test]
    fn nested_on_off_matches_baseline() {
        let (k_on, k_off) = nested_on_off(0.01, 50.0);
        let j_a = j_net_req(0.5, 0.01, 1.0, 1.0, 50.0, 0.2, 0.4);
        let j_c = j_net_on_off(0.5, 1.0, 1.0, k_on, k_off, 0.2, 0.4);
        assert!((j_a - j_c).abs() < 1e-15);
    }
}
