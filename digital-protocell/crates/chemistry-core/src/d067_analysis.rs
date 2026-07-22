//! D-067 activation-capacity law identification helpers.
//!
//! Observer/shadow-only diagnostics. Schema 3 is diagnostic and opt-in; production
//! defaults remain schema 2.

use crate::d066_analysis::{D066_F_REF, D066_FROZEN_KT, D066_K_C, D066_N_REF, D066_V_A};
use crate::fields::interior_weight;
use serde::{Deserialize, Serialize};

pub const D067_PROJECT_ID: &str = "D-067";
pub const D067_AGENT_MEMORY_ID: &str = "D-20260721-d067-activation-capacity-law-identification";
pub const D067_STARTING_COMMIT: &str = "cffbe2b";
pub const D067_STARTING_TAG: &str = "D-066-activation-utilization-capacity-audit";
pub const D067_D066_CONCLUSION: &str = "D066_FROZEN_ACTIVATION_CAPACITY_LIMIT";
pub const ACTIVATION_HIGH_SUBSTRATE_CAPACITY_PRESENT: &str =
    "ACTIVATION_HIGH_SUBSTRATE_CAPACITY_PRESENT";
pub const ORDINARY_SUBSTRATE_ACTIVATION_RESPONSE_INSUFFICIENT: &str =
    "ORDINARY_SUBSTRATE_ACTIVATION_RESPONSE_INSUFFICIENT";
pub const D067_V_A: f64 = D066_V_A;
pub const D067_K_C: f64 = D066_K_C;
pub const D067_N_REF: f64 = D066_N_REF;
pub const D067_F_REF: f64 = D066_F_REF;
pub const D067_FROZEN_KT: f64 = D066_FROZEN_KT;
/// Diagnostic-only schema; the production default remains schema 2.
pub const ACTIVATION_SCHEMA_BOUNDED_NF: u32 = 3;
pub const A_RETENTION: f64 = 0.80;
pub const CHI_A_TARGET: f64 = 1.05;
pub const CHI_VIABLE: f64 = 1.05;
pub const PORTABLE_SPAN_MAX: f64 = 3.0;
pub const BOOTSTRAP_SPREAD_MAX: f64 = 0.50;
pub const LOO_MAX: f64 = 2.0;
pub const HOLDOUT_MEDIAN_ERR: f64 = 0.20;
pub const HOLDOUT_MAX_ERR: f64 = 0.35;
pub const BALANCE_SIGN_ACC: f64 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D067PrimaryConclusion {
    ExistingActivationLawQualified,
    GlobalActivationCapacityCalibrationQualified,
    LowSubstrateActivationResponseQualified,
    ActivationCapacityRepairedStageEStillBlocked,
    PrecursorDemandPrimaryNotActivation,
    WasteExecutionBlocksActivationQualification,
    NoPortableActivationCapacityLaw,
    ActivationCapacityIdentificationInconclusive,
    D066CapacityResultNotReproduced,
    SubstrateResponseLineageUnresolved,
    ActivationParameterIdentificationFailure,
    ActivationSafetyOrCausalityFailure,
    ActivationRuntimeParityFailure,
    AOrWAccountingFailure,
    WorkspaceScopeNotIsolated,
    NumericalFailure,
    Fail,
}

impl D067PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExistingActivationLawQualified => "D067_EXISTING_ACTIVATION_LAW_QUALIFIED",
            Self::GlobalActivationCapacityCalibrationQualified => {
                "D067_GLOBAL_ACTIVATION_CAPACITY_CALIBRATION_QUALIFIED"
            }
            Self::LowSubstrateActivationResponseQualified => {
                "D067_LOW_SUBSTRATE_ACTIVATION_RESPONSE_QUALIFIED"
            }
            Self::ActivationCapacityRepairedStageEStillBlocked => {
                "D067_ACTIVATION_CAPACITY_REPAIRED_STAGE_E_STILL_BLOCKED"
            }
            Self::PrecursorDemandPrimaryNotActivation => {
                "D067_PRECURSOR_DEMAND_PRIMARY_NOT_ACTIVATION"
            }
            Self::WasteExecutionBlocksActivationQualification => {
                "D067_WASTE_EXECUTION_BLOCKS_ACTIVATION_QUALIFICATION"
            }
            Self::NoPortableActivationCapacityLaw => "D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW",
            Self::ActivationCapacityIdentificationInconclusive => {
                "D067_ACTIVATION_CAPACITY_IDENTIFICATION_INCONCLUSIVE"
            }
            Self::D066CapacityResultNotReproduced => "D067_D066_CAPACITY_RESULT_NOT_REPRODUCED",
            Self::SubstrateResponseLineageUnresolved => {
                "D067_SUBSTRATE_RESPONSE_LINEAGE_UNRESOLVED"
            }
            Self::ActivationParameterIdentificationFailure => {
                "D067_ACTIVATION_PARAMETER_IDENTIFICATION_FAILURE"
            }
            Self::ActivationSafetyOrCausalityFailure => {
                "D067_ACTIVATION_SAFETY_OR_CAUSALITY_FAILURE"
            }
            Self::ActivationRuntimeParityFailure => "D067_ACTIVATION_RUNTIME_PARITY_FAILURE",
            Self::AOrWAccountingFailure => "D067_A_OR_W_ACCOUNTING_FAILURE",
            Self::WorkspaceScopeNotIsolated => "D067_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::NumericalFailure => "D067_NUMERICAL_FAILURE",
            Self::Fail => "D067_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D067Route {
    E,
    V,
    R,
    P,
    D,
    W,
    N,
    I,
}

impl D067Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E => "Route_E_existing_activation_law_qualified",
            Self::V => "Route_V_global_activation_capacity_calibration_qualified",
            Self::R => "Route_R_low_substrate_activation_response_qualified",
            Self::P => "Route_P_activation_capacity_repaired_stage_e_still_blocked",
            Self::D => "Route_D_precursor_demand_primary_not_activation",
            Self::W => "Route_W_waste_execution_blocks_activation_qualification",
            Self::N => "Route_N_no_portable_activation_capacity_law",
            Self::I => "Route_I_activation_capacity_identification_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D067PrimaryConclusion {
        match self {
            Self::E => D067PrimaryConclusion::ExistingActivationLawQualified,
            Self::V => D067PrimaryConclusion::GlobalActivationCapacityCalibrationQualified,
            Self::R => D067PrimaryConclusion::LowSubstrateActivationResponseQualified,
            Self::P => D067PrimaryConclusion::ActivationCapacityRepairedStageEStillBlocked,
            Self::D => D067PrimaryConclusion::PrecursorDemandPrimaryNotActivation,
            Self::W => D067PrimaryConclusion::WasteExecutionBlocksActivationQualification,
            Self::N => D067PrimaryConclusion::NoPortableActivationCapacityLaw,
            Self::I => D067PrimaryConclusion::ActivationCapacityIdentificationInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubstrateResponseClass {
    OrdinaryResponseLinearLow,
    OrdinaryResponseSaturated,
    OrdinaryResponseProductSuppressed,
    ResourceResponseAlreadyBounded,
    SubstrateResponseLineageUnresolved,
}

impl SubstrateResponseClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OrdinaryResponseLinearLow => "ORDINARY_RESPONSE_LINEAR_LOW",
            Self::OrdinaryResponseSaturated => "ORDINARY_RESPONSE_SATURATED",
            Self::OrdinaryResponseProductSuppressed => "ORDINARY_RESPONSE_PRODUCT_SUPPRESSED",
            Self::ResourceResponseAlreadyBounded => "RESOURCE_RESPONSE_ALREADY_BOUNDED",
            Self::SubstrateResponseLineageUnresolved => "SUBSTRATE_RESPONSE_LINEAGE_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HighResourceCeilingClass {
    HighResourceCeilingHasHeadroom,
    HighResourceCeilingAlreadyTight,
    HighResourceOverproductionRisk,
    HighResourceCeilingInconclusive,
}

impl HighResourceCeilingClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HighResourceCeilingHasHeadroom => "HIGH_RESOURCE_CEILING_HAS_HEADROOM",
            Self::HighResourceCeilingAlreadyTight => "HIGH_RESOURCE_CEILING_ALREADY_TIGHT",
            Self::HighResourceOverproductionRisk => "HIGH_RESOURCE_OVERPRODUCTION_RISK",
            Self::HighResourceCeilingInconclusive => "HIGH_RESOURCE_CEILING_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationCandidate {
    Baseline,
    GlobalScale,
    BoundedNfResponse,
}

impl ActivationCandidate {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "BASELINE",
            Self::GlobalScale => "GLOBAL_SCALE",
            Self::BoundedNfResponse => "BOUNDED_NF_RESPONSE",
        }
    }
}

/// N̂=N/N_ref has no upper clip and can therefore exceed one.
#[inline]
pub fn n_hat(n: f64, n_ref: f64) -> f64 {
    n.max(0.0) / n_ref.max(1e-18)
}

/// F̂=F/F_ref has no upper clip and can therefore exceed one.
#[inline]
pub fn f_hat(f: f64, f_ref: f64) -> f64 {
    f.max(0.0) / f_ref.max(1e-18)
}

/// N̂F̂ is effectively quadratic in the ordinary domain when N and F both scale.
#[inline]
pub fn product_n_f_hat(n: f64, f: f64, n_ref: f64, f_ref: f64) -> f64 {
    n_hat(n, n_ref) * f_hat(f, f_ref)
}

pub fn classify_substrate_response(
    ordinary_n_median: f64,
    ordinary_f_median: f64,
    ordinary_product_median: f64,
    healthy_product_median: f64,
    can_exceed_one: bool,
) -> SubstrateResponseClass {
    if !ordinary_n_median.is_finite()
        || !ordinary_f_median.is_finite()
        || !ordinary_product_median.is_finite()
        || !healthy_product_median.is_finite()
    {
        return SubstrateResponseClass::SubstrateResponseLineageUnresolved;
    }
    if ordinary_product_median >= 0.95 {
        return SubstrateResponseClass::OrdinaryResponseSaturated;
    }
    if !can_exceed_one {
        return SubstrateResponseClass::ResourceResponseAlreadyBounded;
    }
    let expected_product = ordinary_n_median.max(0.0) * ordinary_f_median.max(0.0);
    if ordinary_product_median < 0.5 * healthy_product_median.max(1e-18)
        && ordinary_product_median < 0.5
    {
        if (ordinary_product_median - expected_product).abs()
            <= 0.15 * (1.0 + expected_product.abs())
        {
            return SubstrateResponseClass::OrdinaryResponseLinearLow;
        }
        return SubstrateResponseClass::OrdinaryResponseProductSuppressed;
    }
    SubstrateResponseClass::OrdinaryResponseProductSuppressed
}

/// A linear N/N_ref·F/F_ref baseline is not mathematically equivalent to q_N·q_F,
/// including the common N_ref=F_ref=1 case.
pub fn baseline_equivalent_to_michaelis(n_ref: f64, f_ref: f64) -> bool {
    let _ = (n_ref, f_ref);
    false
}

#[inline]
pub fn q_sat(x: f64, k: f64) -> f64 {
    let x = x.max(0.0);
    x / (k.max(1e-18) + x)
}

/// Schema 2: V_A H(phi) q_C(C) N̂ F̂.
#[inline]
pub fn candidate_a_rate(
    v_a: f64,
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    k_c: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    v_a.max(0.0) * interior_weight(phi) * q_sat(c, k_c) * product_n_f_hat(n, f, n_ref, f_ref)
}

/// Global multiplier applied to schema 2.
#[inline]
pub fn candidate_b_rate(
    m_v: f64,
    v_a: f64,
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    k_c: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    m_v.max(0.0) * candidate_a_rate(v_a, phi, c, n, f, k_c, n_ref, f_ref)
}

/// Schema 3: V_A H(phi) q_C(C) q_N(N) q_F(F).
#[inline]
pub fn candidate_c_rate(
    v_a: f64,
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    k_c: f64,
    k_n: f64,
    k_f: f64,
) -> f64 {
    v_a.max(0.0) * interior_weight(phi) * q_sat(c, k_c) * q_sat(n, k_n) * q_sat(f, k_f)
}

/// Required activation gain after net A transport: L_cat+L_struct+L_prec+L_decay−J_A_net.
#[inline]
pub fn g_a_required(
    l_cat: f64,
    l_struct: f64,
    l_prec: f64,
    l_decay: f64,
    j_a_net: f64,
) -> f64 {
    l_cat + l_struct + l_prec + l_decay - j_a_net
}

#[inline]
pub fn m_a_star(g_required: f64, g_activation0: f64) -> f64 {
    if !g_required.is_finite() || !g_activation0.is_finite() || g_activation0 <= 0.0 {
        return f64::INFINITY;
    }
    (g_required / g_activation0).max(0.0)
}

pub fn multiplier_portable(values: &[f64], max_span: f64) -> bool {
    let finite_positive: Vec<f64> = values
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    if finite_positive.is_empty() || finite_positive.len() != values.len() || !max_span.is_finite() {
        return false;
    }
    let min = finite_positive.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite_positive
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    max / min <= max_span
}

pub fn preregistered_m_v_from_median(median_m: f64) -> Vec<f64> {
    [0.5, 0.75, 1.0, 1.25, 1.5]
        .into_iter()
        .map(|factor| (median_m * factor).max(1.0))
        .collect()
}

pub fn classify_high_resource_ceiling(
    ordinary_a_ret: f64,
    high_nf_a_ret: f64,
    saturating_a_ret: f64,
    a_target: f64,
    unstable_high: bool,
) -> HighResourceCeilingClass {
    if !ordinary_a_ret.is_finite()
        || !high_nf_a_ret.is_finite()
        || !a_target.is_finite()
    {
        return HighResourceCeilingClass::HighResourceCeilingInconclusive;
    }
    if unstable_high {
        return HighResourceCeilingClass::HighResourceOverproductionRisk;
    }
    if ordinary_a_ret >= a_target {
        return HighResourceCeilingClass::HighResourceCeilingAlreadyTight;
    }
    // Fixed-N/F corroboration may be below the retention threshold despite
    // unlimited local substrate demonstrating the relevant capacity ceiling.
    let _ = saturating_a_ret;
    if high_nf_a_ret >= a_target {
        return HighResourceCeilingClass::HighResourceCeilingHasHeadroom;
    }
    HighResourceCeilingClass::HighResourceCeilingInconclusive
}

pub fn zero_activation_when_starved(rate_fn: impl Fn(f64, f64, f64) -> f64) -> bool {
    const EPS: f64 = 1e-15;
    rate_fn(0.0, 1.0, 1.0).abs() <= EPS
        && rate_fn(1.0, 0.0, 1.0).abs() <= EPS
        && rate_fn(1.0, 1.0, 0.0).abs() <= EPS
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct IdentificationReport {
    pub params_positive_finite: bool,
    pub half_sats_in_domain: bool,
    pub bootstrap_spread: f64,
    pub loo_variation: f64,
    pub holdout_median_err: f64,
    pub holdout_max_err: f64,
    pub holdout_balance_sign_acc: f64,
    pub no_radius_params: bool,
    pub stoichiometry_ok: bool,
    pub accounting_ok: bool,
}

impl IdentificationReport {
    pub fn qualifies(&self) -> bool {
        self.params_positive_finite
            && self.half_sats_in_domain
            && self.bootstrap_spread.is_finite()
            && self.bootstrap_spread <= BOOTSTRAP_SPREAD_MAX
            && self.loo_variation.is_finite()
            && self.loo_variation <= LOO_MAX
            && self.holdout_median_err.is_finite()
            && self.holdout_median_err <= HOLDOUT_MEDIAN_ERR
            && self.holdout_max_err.is_finite()
            && self.holdout_max_err <= HOLDOUT_MAX_ERR
            && self.holdout_balance_sign_acc.is_finite()
            && self.holdout_balance_sign_acc >= BALANCE_SIGN_ACC
            && self.no_radius_params
            && self.stoichiometry_ok
            && self.accounting_ok
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemandCounterfactualClass {
    PrecursorDemandPrimary,
    ActivationCapacityPrimary,
    MixedDemandAndActivation,
    DemandCounterfactualInconclusive,
}

impl DemandCounterfactualClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrecursorDemandPrimary => "PRECURSOR_DEMAND_PRIMARY",
            Self::ActivationCapacityPrimary => "ACTIVATION_CAPACITY_PRIMARY",
            Self::MixedDemandAndActivation => "MIXED_DEMAND_AND_ACTIVATION",
            Self::DemandCounterfactualInconclusive => "DEMAND_COUNTERFACTUAL_INCONCLUSIVE",
        }
    }
}

pub fn classify_demand_counterfactual(
    activation_repair_restores_a: bool,
    precursor_demand_relief_restores_a: bool,
) -> DemandCounterfactualClass {
    match (activation_repair_restores_a, precursor_demand_relief_restores_a) {
        (false, true) => DemandCounterfactualClass::PrecursorDemandPrimary,
        (true, false) => DemandCounterfactualClass::ActivationCapacityPrimary,
        (true, true) => DemandCounterfactualClass::MixedDemandAndActivation,
        (false, false) => DemandCounterfactualClass::DemandCounterfactualInconclusive,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence067 {
    pub workspace_isolated: bool,
    pub d066_reproduced: bool,
    pub substrate_lineage_ok: bool,
    pub runtime_parity_ok: bool,
    pub a_w_accounting_ok: bool,
    pub safety_causality_ok: bool,
    pub identification: IdentificationReport,
    pub waste_blocks_qualification: bool,
    pub existing_law_qualified: bool,
    pub global_scale_qualified: bool,
    pub low_substrate_response_qualified: bool,
    pub activation_repaired_stage_e_blocked: bool,
    pub precursor_demand_primary: bool,
    pub no_portable_law: bool,
}

/// Directive order: hard failures, waste, qualified candidates, downstream
/// counterfactuals, no-portable finding, then inconclusive.
pub fn select_route(ev: RouteEvidence067) -> (D067Route, D067PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (D067Route::I, D067PrimaryConclusion::WorkspaceScopeNotIsolated);
    }
    if !ev.d066_reproduced {
        return (D067Route::I, D067PrimaryConclusion::D066CapacityResultNotReproduced);
    }
    if !ev.substrate_lineage_ok {
        return (
            D067Route::I,
            D067PrimaryConclusion::SubstrateResponseLineageUnresolved,
        );
    }
    if !ev.runtime_parity_ok {
        return (
            D067Route::I,
            D067PrimaryConclusion::ActivationRuntimeParityFailure,
        );
    }
    if !ev.a_w_accounting_ok {
        return (D067Route::I, D067PrimaryConclusion::AOrWAccountingFailure);
    }
    if !ev.safety_causality_ok {
        return (
            D067Route::I,
            D067PrimaryConclusion::ActivationSafetyOrCausalityFailure,
        );
    }
    if !ev.identification.qualifies() {
        return (
            D067Route::I,
            D067PrimaryConclusion::ActivationParameterIdentificationFailure,
        );
    }
    if ev.waste_blocks_qualification {
        return (D067Route::W, D067Route::W.conclusion());
    }
    if ev.existing_law_qualified {
        return (D067Route::E, D067Route::E.conclusion());
    }
    if ev.global_scale_qualified {
        return (D067Route::V, D067Route::V.conclusion());
    }
    if ev.low_substrate_response_qualified {
        return (D067Route::R, D067Route::R.conclusion());
    }
    if ev.activation_repaired_stage_e_blocked {
        return (D067Route::P, D067Route::P.conclusion());
    }
    if ev.no_portable_law {
        return (D067Route::N, D067Route::N.conclusion());
    }
    if ev.precursor_demand_primary {
        return (D067Route::D, D067Route::D.conclusion());
    }
    (D067Route::I, D067Route::I.conclusion())
}

/// D-066 frozen-capacity reproduction: smooth resources are viable but ordinary
/// activation is low, perfect exterior does not repair it, and unlimited local
/// substrate does; χ_A stays distinctly below target.
pub fn d066_reproduction_predicate(
    smooth_chi_min: f64,
    ordinary_a: f64,
    unlimited_a: f64,
    perfect_exterior_a: f64,
    chi_a: f64,
) -> bool {
    smooth_chi_min >= CHI_VIABLE
        && ordinary_a < A_RETENTION
        && unlimited_a >= A_RETENTION
        && perfect_exterior_a < A_RETENTION
        && chi_a.is_finite()
        && chi_a < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qualified_identification() -> IdentificationReport {
        IdentificationReport {
            params_positive_finite: true,
            half_sats_in_domain: true,
            bootstrap_spread: 0.1,
            loo_variation: 1.1,
            holdout_median_err: 0.1,
            holdout_max_err: 0.2,
            holdout_balance_sign_acc: 0.95,
            no_radius_params: true,
            stoichiometry_ok: true,
            accounting_ok: true,
        }
    }

    fn evidence() -> RouteEvidence067 {
        RouteEvidence067 {
            workspace_isolated: true,
            d066_reproduced: true,
            substrate_lineage_ok: true,
            runtime_parity_ok: true,
            a_w_accounting_ok: true,
            safety_causality_ok: true,
            identification: qualified_identification(),
            waste_blocks_qualification: false,
            existing_law_qualified: false,
            global_scale_qualified: false,
            low_substrate_response_qualified: false,
            activation_repaired_stage_e_blocked: false,
            precursor_demand_primary: false,
            no_portable_law: false,
        }
    }

    #[test]
    fn n_hat_can_exceed_one() {
        assert_eq!(n_hat(2.0, 1.0), 2.0);
    }

    #[test]
    fn candidate_c_zero_on_starvation() {
        assert!(zero_activation_when_starved(|c, n, f| {
            candidate_c_rate(1.0, 1.0, c, n, f, 0.1, 1.0, 1.0)
        }));
    }

    #[test]
    fn baseline_is_not_equivalent_to_michaelis() {
        assert!(!baseline_equivalent_to_michaelis(1.0, 1.0));
    }

    #[test]
    fn multiplier_portability_uses_positive_span() {
        assert!(multiplier_portable(&[1.0, 2.0, 3.0], PORTABLE_SPAN_MAX));
        assert!(!multiplier_portable(&[1.0, 3.1], PORTABLE_SPAN_MAX));
    }

    #[test]
    fn route_selection_w_v_r_e() {
        let mut ev = evidence();
        ev.waste_blocks_qualification = true;
        assert_eq!(select_route(ev).0, D067Route::W);
        ev.waste_blocks_qualification = false;
        ev.global_scale_qualified = true;
        assert_eq!(select_route(ev).0, D067Route::V);
        ev.global_scale_qualified = false;
        ev.low_substrate_response_qualified = true;
        assert_eq!(select_route(ev).0, D067Route::R);
        ev.low_substrate_response_qualified = false;
        ev.existing_law_qualified = true;
        assert_eq!(select_route(ev).0, D067Route::E);
    }

    #[test]
    fn workspace_failure_precedes_scientific_routes() {
        let mut ev = evidence();
        ev.workspace_isolated = false;
        ev.existing_law_qualified = true;
        assert_eq!(
            select_route(ev).1,
            D067PrimaryConclusion::WorkspaceScopeNotIsolated
        );
    }
}
