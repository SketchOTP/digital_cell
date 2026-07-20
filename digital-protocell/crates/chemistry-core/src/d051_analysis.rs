//! D-051 coupled activation throughput bottleneck audit (diagnostic only).
//!
//! No activation-law, stoichiometry, transport, or productive-rate changes.
//! Classifies why schema-2 V_A scaling fails to restore coupled free A / membrane.

use serde::{Deserialize, Serialize};

pub const D051_PROJECT_ID: &str = "D-051";
pub const D051_AGENT_MEMORY_ID: &str =
    "D-20260720-d051-coupled-activation-throughput-bottleneck-audit";
pub const D051_STARTING_COMMIT: &str = "0b0fb89";
pub const D051_STARTING_TAG: &str = "D-050-catalyst-saturating-activation-fail";
pub const D051_D050_RECORD: &str = "CATALYST_SATURATING_CAPACITY_REPAIR_REJECTED";
pub const D051_FROZEN_D049: &str = "D049_COUPLED_ACTIVATION_CAPACITY_FAILURE";
pub const D051_FROZEN_D050: &str = "D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED";

pub const D051_FITTED_V_A: f64 = 0.12544510052968755;
pub const D051_FITTED_K_C: f64 = 0.10;
pub const D051_N_REF: f64 = 1.0;
pub const D051_F_REF: f64 = 1.0;
pub const D051_RADIUS: f64 = 22.0;
pub const D051_THETA: f64 = 0.6;
pub const D051_DEFAULT_HORIZON: u64 = 10_000;
pub const D051_WINDOW: u64 = 1_000;
pub const D051_EPS: f64 = 1.0e-18;
pub const D051_LEDGER_REL_TOL: f64 = 0.05;
pub const D051_RETENTION_COLLAPSE: f64 = 0.10;
pub const D051_MATERIAL_RISE: f64 = 0.05;
pub const D051_EXTENT_FLAT_REL: f64 = 0.15;
pub const D051_HEALTHY_N: f64 = 1.0;
pub const D051_HEALTHY_F: f64 = 1.0;

/// Gate-0 reproduction multipliers (plus historical schema 1).
pub fn d051_v_a_multipliers() -> &'static [f64] {
    &[0.75, 1.0, 2.0, 4.0]
}

pub fn v_a_from_multiplier(mult: f64) -> f64 {
    D051_FITTED_V_A * mult
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationLimitClass {
    Unconstrained,
    NLimited,
    FLimited,
    JointlyNfLimited,
    TimestepLimited,
    ConcentrationSafetyLimited,
    NumericalRejectionLimited,
}

impl ActivationLimitClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unconstrained => "unconstrained",
            Self::NLimited => "N-limited",
            Self::FLimited => "F-limited",
            Self::JointlyNfLimited => "jointly N/F-limited",
            Self::TimestepLimited => "timestep-limited",
            Self::ConcentrationSafetyLimited => "concentration-safety-limited",
            Self::NumericalRejectionLimited => "numerical-rejection-limited",
        }
    }
}

/// Local activation application record (observer).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ActivationExtentRecord {
    pub xi_requested: f64,
    pub xi_accepted: f64,
    pub n_available: f64,
    pub f_available: f64,
    pub rejected: bool,
    pub timestep_capped: bool,
    pub concentration_safety: bool,
}

impl ActivationExtentRecord {
    pub fn f_accepted(self) -> f64 {
        self.xi_accepted / self.xi_requested.max(D051_EPS)
    }

    pub fn classify(self) -> ActivationLimitClass {
        if self.rejected {
            return ActivationLimitClass::NumericalRejectionLimited;
        }
        if self.concentration_safety {
            return ActivationLimitClass::ConcentrationSafetyLimited;
        }
        if self.timestep_capped {
            return ActivationLimitClass::TimestepLimited;
        }
        let n_ok = self.n_available + D051_EPS >= self.xi_requested;
        let f_ok = self.f_available + D051_EPS >= self.xi_requested;
        match (n_ok, f_ok) {
            (true, true) => ActivationLimitClass::Unconstrained,
            (false, true) => ActivationLimitClass::NLimited,
            (true, false) => ActivationLimitClass::FLimited,
            (false, false) => ActivationLimitClass::JointlyNfLimited,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExtentWindowSummary {
    pub sites_with_request: u64,
    pub sites_capped: u64,
    pub mean_accepted_fraction: f64,
    pub requested_integrated: f64,
    pub accepted_integrated: f64,
    pub rejected_requested: f64,
    pub dominant_limit: String,
}

pub fn summarize_extent_records(recs: &[ActivationExtentRecord]) -> ExtentWindowSummary {
    let mut sites_with_request = 0u64;
    let mut sites_capped = 0u64;
    let mut frac_sum = 0.0;
    let mut frac_n = 0u64;
    let mut requested = 0.0;
    let mut accepted = 0.0;
    let mut rejected = 0.0;
    let mut counts = [
        (ActivationLimitClass::Unconstrained, 0u64),
        (ActivationLimitClass::NLimited, 0),
        (ActivationLimitClass::FLimited, 0),
        (ActivationLimitClass::JointlyNfLimited, 0),
        (ActivationLimitClass::TimestepLimited, 0),
        (ActivationLimitClass::ConcentrationSafetyLimited, 0),
        (ActivationLimitClass::NumericalRejectionLimited, 0),
    ];
    for r in recs {
        if r.xi_requested > D051_EPS {
            sites_with_request += 1;
            frac_sum += r.f_accepted();
            frac_n += 1;
        }
        requested += r.xi_requested.max(0.0);
        accepted += r.xi_accepted.max(0.0);
        if r.rejected {
            rejected += r.xi_requested.max(0.0);
        }
        let cls = r.classify();
        if !matches!(cls, ActivationLimitClass::Unconstrained) && r.xi_requested > D051_EPS {
            sites_capped += 1;
        }
        for (c, n) in counts.iter_mut() {
            if *c == cls {
                *n += 1;
                break;
            }
        }
    }
    let dominant = counts
        .iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c.as_str().to_string())
        .unwrap_or_else(|| "none".into());
    ExtentWindowSummary {
        sites_with_request,
        sites_capped,
        mean_accepted_fraction: if frac_n == 0 {
            1.0
        } else {
            frac_sum / frac_n as f64
        },
        requested_integrated: requested,
        accepted_integrated: accepted,
        rejected_requested: rejected,
        dominant_limit: dominant,
    }
}

/// Physical vs numerical capping labels.
pub fn classify_extent_cap_mode(
    requested_scales_with_v_a: bool,
    accepted_flat_across_v_a: bool,
    resource_bounds_dominate: bool,
    numerical_dominate: bool,
) -> &'static str {
    if requested_scales_with_v_a && accepted_flat_across_v_a && resource_bounds_dominate {
        "ACTIVATION_EXTENT_RESOURCE_CAPPED"
    } else if accepted_flat_across_v_a && numerical_dominate {
        "ACTIVATION_EXTENT_NUMERICALLY_CAPPED"
    } else if requested_scales_with_v_a && !accepted_flat_across_v_a {
        "ACTIVATION_EXTENT_SCALES_WITH_V_A"
    } else {
        "ACTIVATION_EXTENT_CAP_INCONCLUSIVE"
    }
}

/// Resource throughput ceiling χ_resource = R_act,max / R_demand.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ResourceCeiling {
    pub j_n_available: f64,
    pub j_f_available: f64,
    pub r_activation_max: f64,
    pub r_demand: f64,
    pub chi_resource: f64,
}

pub fn resource_available_flux(
    j_reservoir: f64,
    j_transport_in: f64,
    m_initial: f64,
    m_floor: f64,
    horizon_t: f64,
) -> f64 {
    j_reservoir + j_transport_in + (m_initial - m_floor).max(0.0) / horizon_t.max(D051_EPS)
}

pub fn compute_resource_ceiling(
    j_n: f64,
    j_f: f64,
    l_rep: f64,
    l_structure: f64,
    l_precursor: f64,
    l_decay: f64,
    l_transport: f64,
) -> ResourceCeiling {
    let r_max = j_n.min(j_f);
    let r_demand = l_rep + l_structure + l_precursor + l_decay + l_transport;
    ResourceCeiling {
        j_n_available: j_n,
        j_f_available: j_f,
        r_activation_max: r_max,
        r_demand,
        chi_resource: r_max / r_demand.max(D051_EPS),
    }
}

/// A cohort destination fractions (must sum ≈ 1).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct ACohortDestinations {
    pub free_remaining: f64,
    pub catalyst_reproduction: f64,
    pub structural: f64,
    pub precursor: f64,
    pub decay: f64,
    pub outward_transport: f64,
}

impl ACohortDestinations {
    pub fn normalize(self) -> Self {
        let s = self.free_remaining
            + self.catalyst_reproduction
            + self.structural
            + self.precursor
            + self.decay
            + self.outward_transport;
        if s <= D051_EPS {
            return self;
        }
        Self {
            free_remaining: self.free_remaining / s,
            catalyst_reproduction: self.catalyst_reproduction / s,
            structural: self.structural / s,
            precursor: self.precursor / s,
            decay: self.decay / s,
            outward_transport: self.outward_transport / s,
        }
    }

    pub fn sum(self) -> f64 {
        self.free_remaining
            + self.catalyst_reproduction
            + self.structural
            + self.precursor
            + self.decay
            + self.outward_transport
    }

    pub fn productive_immediate_fraction(self) -> f64 {
        self.catalyst_reproduction + self.structural + self.precursor
    }
}

pub fn cohort_from_ledger(
    activation: f64,
    free_delta: f64,
    reproduction: f64,
    structural: f64,
    precursor: f64,
    decay: f64,
    outward: f64,
) -> ACohortDestinations {
    let act = activation.max(0.0);
    if act <= D051_EPS {
        return ACohortDestinations::default();
    }
    let free_remaining = free_delta.max(0.0);
    ACohortDestinations {
        free_remaining,
        catalyst_reproduction: reproduction.max(0.0),
        structural: structural.max(0.0),
        precursor: precursor.max(0.0),
        decay: decay.max(0.0),
        outward_transport: outward.max(0.0),
    }
    .normalize()
}

pub fn is_immediate_productive_capture(
    gross_activation_rises_with_v_a: bool,
    productive_fraction: f64,
    free_fraction: f64,
) -> bool {
    gross_activation_rises_with_v_a && productive_fraction >= 0.70 && free_fraction <= 0.15
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ProductYields {
    pub y_a_to_p: f64,
    pub y_a_to_s: f64,
    pub p_decay: f64,
    pub p_outward: f64,
    pub p_unused: f64,
    pub p_adsorbed: f64,
    pub s_desorbed: f64,
}

pub fn precursor_yields(
    delta_p_produced: f64,
    a_consumed_precursor: f64,
    precursor_derived_s_gain: f64,
    p_decay: f64,
    p_outward: f64,
    p_unused: f64,
    p_adsorbed: f64,
    s_desorbed: f64,
) -> ProductYields {
    let den = a_consumed_precursor.max(D051_EPS);
    ProductYields {
        y_a_to_p: delta_p_produced / den,
        y_a_to_s: precursor_derived_s_gain / den,
        p_decay,
        p_outward,
        p_unused,
        p_adsorbed,
        s_desorbed,
    }
}

pub fn spatial_overlap(production: &[f64], demand: &[f64]) -> f64 {
    assert_eq!(production.len(), demand.len());
    let mut num = 0.0;
    let mut den = 0.0;
    for (p, d) in production.iter().zip(demand.iter()) {
        let dd = d.max(0.0);
        den += dd;
        num += p.max(0.0).min(dd);
    }
    if den <= D051_EPS {
        1.0
    } else {
        num / den
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FreePoolClass {
    FreeAPoolCausallyDeficient,
    FreeARetentionMetricNoncausal,
    HighFluxActivationWastedDownstream,
}

impl FreePoolClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FreeAPoolCausallyDeficient => "FREE_A_POOL_CAUSALLY_DEFICIENT",
            Self::FreeARetentionMetricNoncausal => "FREE_A_RETENTION_METRIC_NONCAUSAL",
            Self::HighFluxActivationWastedDownstream => "HIGH_FLUX_ACTIVATION_WASTED_DOWNSTREAM",
        }
    }
}

/// τ_A = M_A / R_act ; Q_A = R_act / R_demand
pub fn classify_free_pool(
    m_a: f64,
    r_activation: f64,
    r_demand: f64,
    services_active: bool,
    wasteful_downstream: bool,
) -> FreePoolClass {
    let q = r_activation / r_demand.max(D051_EPS);
    if wasteful_downstream && q >= 0.9 {
        return FreePoolClass::HighFluxActivationWastedDownstream;
    }
    if m_a < D051_RETENTION_COLLAPSE && q >= 0.9 && services_active {
        return FreePoolClass::FreeARetentionMetricNoncausal;
    }
    FreePoolClass::FreeAPoolCausallyDeficient
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaxActivationOutcome {
    CoupledActivationTopologyCapable,
    LowFreeAHighFluxMembraneCapable,
    ActivationNotPrimaryCoupledBlocker,
}

impl MaxActivationOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CoupledActivationTopologyCapable => "COUPLED_ACTIVATION_TOPOLOGY_CAPABLE",
            Self::LowFreeAHighFluxMembraneCapable => "LOW_FREE_A_HIGH_FLUX_MEMBRANE_CAPABLE",
            Self::ActivationNotPrimaryCoupledBlocker => "ACTIVATION_NOT_PRIMARY_COUPLED_BLOCKER",
        }
    }
}

pub fn classify_max_activation_control(
    a_restored: bool,
    membrane_stabilized: bool,
    a_still_low: bool,
) -> MaxActivationOutcome {
    if a_restored && membrane_stabilized {
        MaxActivationOutcome::CoupledActivationTopologyCapable
    } else if membrane_stabilized && a_still_low {
        MaxActivationOutcome::LowFreeAHighFluxMembraneCapable
    } else {
        MaxActivationOutcome::ActivationNotPrimaryCoupledBlocker
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D051PrimaryConclusion {
    ResourceThroughputLimit,
    ActivationExtentBoundingDefect,
    ActivationOperatorSplitDefect,
    ActivationSpatialAllocationFailure,
    PrecursorConversionBottleneck,
    FreeARetentionMetricNoncausal,
    CoupledActivationTopologyInsufficient,
    ActivationNotPrimaryCoupledBlocker,
    CoupledActivationThroughputInconclusive,
    D050EvidenceNotSealed,
    D050FailureNotReproduced,
    ActivationExtentAccountingFailure,
    ACohortAccountingFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D051PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceThroughputLimit => "D051_RESOURCE_THROUGHPUT_LIMIT",
            Self::ActivationExtentBoundingDefect => "D051_ACTIVATION_EXTENT_BOUNDING_DEFECT",
            Self::ActivationOperatorSplitDefect => "D051_ACTIVATION_OPERATOR_SPLIT_DEFECT",
            Self::ActivationSpatialAllocationFailure => "D051_ACTIVATION_SPATIAL_ALLOCATION_FAILURE",
            Self::PrecursorConversionBottleneck => "D051_PRECURSOR_CONVERSION_BOTTLENECK",
            Self::FreeARetentionMetricNoncausal => "D051_FREE_A_RETENTION_METRIC_NONCAUSAL",
            Self::CoupledActivationTopologyInsufficient => {
                "D051_COUPLED_ACTIVATION_TOPOLOGY_INSUFFICIENT"
            }
            Self::ActivationNotPrimaryCoupledBlocker => {
                "D051_ACTIVATION_NOT_PRIMARY_COUPLED_BLOCKER"
            }
            Self::CoupledActivationThroughputInconclusive => {
                "D051_COUPLED_ACTIVATION_THROUGHPUT_INCONCLUSIVE"
            }
            Self::D050EvidenceNotSealed => "D051_D050_EVIDENCE_NOT_SEALED",
            Self::D050FailureNotReproduced => "D051_D050_FAILURE_NOT_REPRODUCED",
            Self::ActivationExtentAccountingFailure => "D051_ACTIVATION_EXTENT_ACCOUNTING_FAILURE",
            Self::ACohortAccountingFailure => "D051_A_COHORT_ACCOUNTING_FAILURE",
            Self::AccountingFailure => "D051_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D051_NUMERICAL_FAILURE",
            Self::Fail => "D051_FAIL",
        }
    }
}

/// Inputs for Gate-10 primary route selection (exactly one).
#[derive(Debug, Clone, Copy, Default)]
pub struct RouteDecisionInput {
    pub d050_sealed: bool,
    pub d050_reproduced: bool,
    pub extent_accounting_ok: bool,
    pub cohort_accounting_ok: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub resource_throughput_limits: bool,
    pub extent_bounding_defect: bool,
    pub operator_split_defect: bool,
    pub spatial_allocation_failure: bool,
    pub precursor_conversion_bottleneck: bool,
    pub free_a_metric_noncausal: bool,
    pub topology_insufficient: bool,
    pub activation_not_primary: bool,
}

pub fn select_primary_route(input: &RouteDecisionInput) -> D051PrimaryConclusion {
    if !input.d050_sealed {
        return D051PrimaryConclusion::D050EvidenceNotSealed;
    }
    if !input.d050_reproduced {
        return D051PrimaryConclusion::D050FailureNotReproduced;
    }
    if !input.extent_accounting_ok {
        return D051PrimaryConclusion::ActivationExtentAccountingFailure;
    }
    if !input.cohort_accounting_ok {
        return D051PrimaryConclusion::ACohortAccountingFailure;
    }
    if !input.accounting_ok {
        return D051PrimaryConclusion::AccountingFailure;
    }
    if !input.numerical_ok {
        return D051PrimaryConclusion::NumericalFailure;
    }
    // Priority order mirrors directive route list R → B → O → L → P → M → T → N → I
    if input.resource_throughput_limits {
        return D051PrimaryConclusion::ResourceThroughputLimit;
    }
    if input.extent_bounding_defect {
        return D051PrimaryConclusion::ActivationExtentBoundingDefect;
    }
    if input.operator_split_defect {
        return D051PrimaryConclusion::ActivationOperatorSplitDefect;
    }
    if input.spatial_allocation_failure {
        return D051PrimaryConclusion::ActivationSpatialAllocationFailure;
    }
    if input.precursor_conversion_bottleneck {
        return D051PrimaryConclusion::PrecursorConversionBottleneck;
    }
    if input.free_a_metric_noncausal {
        return D051PrimaryConclusion::FreeARetentionMetricNoncausal;
    }
    if input.topology_insufficient {
        return D051PrimaryConclusion::CoupledActivationTopologyInsufficient;
    }
    if input.activation_not_primary {
        return D051PrimaryConclusion::ActivationNotPrimaryCoupledBlocker;
    }
    D051PrimaryConclusion::CoupledActivationThroughputInconclusive
}

/// Shadow operator schedules (analysis-only labels).
pub fn accepted_step_operator_order() -> &'static [&'static str] {
    &[
        "reservoir_update",
        "nf_ca_transport_rates",
        "structural_production",
        "activation_reproduction_a_decay_c_turnover",
        "precursor_diffusion",
        "surface_precursor_and_ps_exchange",
        "precursor_transport_apply",
        "positivity_reject",
    ]
}

pub fn shadow_schedules() -> &'static [&'static str] {
    &[
        "current_production_order",
        "activation_before_all_a_sinks",
        "a_sinks_before_activation",
        "symmetric_half_activation_demand_half_activation",
        "jointly_bounded_activation_and_demand",
    ]
}

/// Joint overcommitment test: Σ ξ_A,i > A_available before common bounding.
pub fn overcommitment(sum_xi_a: f64, a_available: f64) -> bool {
    sum_xi_a > a_available + D051_EPS
}

/// Conservative jointly-bounded extents share a common factor.
pub fn jointly_bound_extents(extents: &[f64], available: f64) -> Vec<f64> {
    let sum: f64 = extents.iter().copied().sum();
    if sum <= available + D051_EPS || sum <= D051_EPS {
        return extents.to_vec();
    }
    let scale = available / sum;
    extents.iter().map(|e| e * scale).collect()
}

pub fn material_rise(baseline: f64, treatment: f64) -> bool {
    treatment - baseline >= D051_MATERIAL_RISE
}

pub fn extent_nearly_flat(a: f64, b: f64) -> bool {
    let scale = a.abs().max(b.abs()).max(D051_EPS);
    (a - b).abs() / scale <= D051_EXTENT_FLAT_REL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_n_and_f_limits() {
        let n_lim = ActivationExtentRecord {
            xi_requested: 0.5,
            xi_accepted: 0.2,
            n_available: 0.2,
            f_available: 1.0,
            rejected: false,
            timestep_capped: false,
            concentration_safety: false,
        };
        assert_eq!(n_lim.classify(), ActivationLimitClass::NLimited);
        let f_lim = ActivationExtentRecord {
            xi_requested: 0.5,
            xi_accepted: 0.2,
            n_available: 1.0,
            f_available: 0.2,
            rejected: false,
            timestep_capped: false,
            concentration_safety: false,
        };
        assert_eq!(f_lim.classify(), ActivationLimitClass::FLimited);
        let joint = ActivationExtentRecord {
            xi_requested: 0.5,
            xi_accepted: 0.1,
            n_available: 0.1,
            f_available: 0.1,
            rejected: false,
            timestep_capped: false,
            concentration_safety: false,
        };
        assert_eq!(joint.classify(), ActivationLimitClass::JointlyNfLimited);
    }

    #[test]
    fn physical_vs_numerical_cap_labels() {
        assert_eq!(
            classify_extent_cap_mode(true, true, true, false),
            "ACTIVATION_EXTENT_RESOURCE_CAPPED"
        );
        assert_eq!(
            classify_extent_cap_mode(true, true, false, true),
            "ACTIVATION_EXTENT_NUMERICALLY_CAPPED"
        );
        assert_eq!(
            classify_extent_cap_mode(true, false, false, false),
            "ACTIVATION_EXTENT_SCALES_WITH_V_A"
        );
    }

    #[test]
    fn resource_ceiling_chi() {
        let c = compute_resource_ceiling(1.0, 2.0, 0.5, 0.2, 0.2, 0.05, 0.05);
        assert!((c.r_activation_max - 1.0).abs() < 1e-15);
        assert!((c.chi_resource - 1.0).abs() < 1e-12);
        let short = compute_resource_ceiling(0.1, 0.1, 1.0, 0.0, 0.0, 0.0, 0.0);
        assert!(short.chi_resource < 1.0);
    }

    #[test]
    fn cohort_conservation_and_immediate_capture() {
        let d = cohort_from_ledger(1.0, 0.05, 0.1, 0.1, 0.7, 0.03, 0.02);
        assert!((d.sum() - 1.0).abs() < 1e-12);
        assert!(is_immediate_productive_capture(true, d.productive_immediate_fraction(), d.free_remaining));
    }

    #[test]
    fn yields_and_spatial_overlap() {
        let y = precursor_yields(0.5, 1.0, 0.1, 0.0, 0.0, 0.4, 0.1, 0.0);
        assert!((y.y_a_to_p - 0.5).abs() < 1e-15);
        assert!((y.y_a_to_s - 0.1).abs() < 1e-15);
        let omega = spatial_overlap(&[1.0, 0.0], &[1.0, 1.0]);
        assert!((omega - 0.5).abs() < 1e-15);
    }

    #[test]
    fn free_pool_and_max_control() {
        assert_eq!(
            classify_free_pool(0.03, 1.0, 1.0, true, false),
            FreePoolClass::FreeARetentionMetricNoncausal
        );
        assert_eq!(
            classify_free_pool(0.03, 0.2, 1.0, false, false),
            FreePoolClass::FreeAPoolCausallyDeficient
        );
        assert_eq!(
            classify_max_activation_control(false, false, true),
            MaxActivationOutcome::ActivationNotPrimaryCoupledBlocker
        );
    }

    #[test]
    fn route_selection_priority() {
        let mut i = RouteDecisionInput {
            d050_sealed: true,
            d050_reproduced: true,
            extent_accounting_ok: true,
            cohort_accounting_ok: true,
            accounting_ok: true,
            numerical_ok: true,
            ..Default::default()
        };
        i.resource_throughput_limits = true;
        i.precursor_conversion_bottleneck = true;
        assert_eq!(
            select_primary_route(&i),
            D051PrimaryConclusion::ResourceThroughputLimit
        );
        i.resource_throughput_limits = false;
        assert_eq!(
            select_primary_route(&i),
            D051PrimaryConclusion::PrecursorConversionBottleneck
        );
        assert_eq!(
            select_primary_route(&RouteDecisionInput {
                d050_sealed: false,
                ..Default::default()
            }),
            D051PrimaryConclusion::D050EvidenceNotSealed
        );
    }

    #[test]
    fn jointly_bound_and_overcommit() {
        assert!(overcommitment(1.2, 1.0));
        let b = jointly_bound_extents(&[0.6, 0.6], 1.0);
        assert!((b[0] + b[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn no_diagnostic_feedback_constants() {
        // Schema2 remains experimental; historical schema1 identity preserved in constants.
        assert_eq!(D051_D050_RECORD, "CATALYST_SATURATING_CAPACITY_REPAIR_REJECTED");
        assert!(d051_v_a_multipliers().contains(&4.0));
        assert_eq!(accepted_step_operator_order().len(), 8);
        assert_eq!(shadow_schedules().len(), 5);
    }
}
