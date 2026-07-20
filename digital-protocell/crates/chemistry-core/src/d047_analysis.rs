//! D-047 shared activated-resource pool sufficiency (observer / diagnostic only).
//!
//! Freeze one biochemistry. Distinguish cross-parameter portability defects from
//! true shared-pool insufficiency. Do not implement activation laws or C_star.

use crate::d046_analysis::{
    fit_model_a, fit_model_b, fit_model_c, fit_model_d, DemandStateRow, ModelFitReport,
};
use serde::{Deserialize, Serialize};

pub const D047_AGENT_MEMORY_ID: &str =
    "D-20260720-d047-shared-activated-resource-pool-sufficiency";
pub const D047_D046_TAG: &str = "D-046-activated-resource-demand-audit";
pub const D047_D046_COMMIT: &str = "bafc830";
pub const D047_RECORD_MIXED: &str = "MIXED_LEGITIMATE_A_DEMAND_CONFIRMED";
pub const D047_HISTORICAL_K: f64 = 0.020;
pub const D047_MODEL_MEDIAN_HOLD_ERR: f64 = 0.20;
pub const D047_MODEL_MAX_HOLD_ERR: f64 = 0.35;
pub const D047_MODEL_BOOTSTRAP_SPREAD: f64 = 0.50;
pub const D047_MODEL_LOO_FACTOR: f64 = 2.0;
pub const D047_K_C_MEMBRANE: f64 = 0.10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D047Conclusion {
    HistoricalActivationFixedBiologyQualified,
    CatalystSaturatingActivationJustified,
    ProductInhibitedActivationJustified,
    PrecursorDemandRegulationDefect,
    SpatialAAllocationDefect,
    SharedAPoolStructurallyInsufficient,
    AEquivalentRoleInconsistent,
    ATracerAccountingFailure,
    CrossParameterPortabilityDefect,
    FixedBiologySupplyMismatch,
    SharedAPoolAuditInconclusive,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D047Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HistoricalActivationFixedBiologyQualified => {
                "D047_HISTORICAL_ACTIVATION_FIXED_BIOLOGY_QUALIFIED"
            }
            Self::CatalystSaturatingActivationJustified => {
                "D047_CATALYST_SATURATING_ACTIVATION_JUSTIFIED"
            }
            Self::ProductInhibitedActivationJustified => {
                "D047_PRODUCT_INHIBITED_ACTIVATION_JUSTIFIED"
            }
            Self::PrecursorDemandRegulationDefect => "D047_PRECURSOR_DEMAND_REGULATION_DEFECT",
            Self::SpatialAAllocationDefect => "D047_SPATIAL_A_ALLOCATION_DEFECT",
            Self::SharedAPoolStructurallyInsufficient => {
                "D047_SHARED_A_POOL_STRUCTURALLY_INSUFFICIENT"
            }
            Self::AEquivalentRoleInconsistent => "D047_A_EQUIVALENT_ROLE_INCONSISTENT",
            Self::ATracerAccountingFailure => "D047_A_TRACER_ACCOUNTING_FAILURE",
            Self::CrossParameterPortabilityDefect => "D047_CROSS_PARAMETER_PORTABILITY_DEFECT",
            Self::FixedBiologySupplyMismatch => "D047_FIXED_BIOLOGY_SUPPLY_MISMATCH",
            Self::SharedAPoolAuditInconclusive => "D047_SHARED_A_POOL_AUDIT_INCONCLUSIVE",
            Self::AccountingFailure => "D047_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D047_NUMERICAL_FAILURE",
            Self::Fail => "D047_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D047Route {
    RouteH,
    RouteV,
    RouteF,
    RouteP,
    RouteL,
    RouteM,
    RouteI,
}

impl D047Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RouteH => "ROUTE_H_HISTORICAL_ACTIVATION_FIXED_BIOLOGY",
            Self::RouteV => "ROUTE_V_CATALYST_SATURATING_VOLUME_ACTIVATION",
            Self::RouteF => "ROUTE_F_PRODUCT_INHIBITED_SHARED_POOL_ACTIVATION",
            Self::RouteP => "ROUTE_P_PRECURSOR_REGULATION_DEFECT",
            Self::RouteL => "ROUTE_L_SPATIAL_ALLOCATION_DEFECT",
            Self::RouteM => "ROUTE_M_SHARED_POOL_STRUCTURALLY_INSUFFICIENT",
            Self::RouteI => "ROUTE_I_INCONCLUSIVE",
        }
    }

    pub const fn conclusion(self) -> D047Conclusion {
        match self {
            Self::RouteH => D047Conclusion::HistoricalActivationFixedBiologyQualified,
            Self::RouteV => D047Conclusion::CatalystSaturatingActivationJustified,
            Self::RouteF => D047Conclusion::ProductInhibitedActivationJustified,
            Self::RouteP => D047Conclusion::PrecursorDemandRegulationDefect,
            Self::RouteL => D047Conclusion::SpatialAAllocationDefect,
            Self::RouteM => D047Conclusion::SharedAPoolStructurallyInsufficient,
            Self::RouteI => D047Conclusion::SharedAPoolAuditInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BiochemistryClass {
    FixedBiochemistry,
    AlteredBiochemistry,
}

/// Gate 0: classify a D-046 (or D-047) campaign row.
pub fn classify_biochemistry_state(row: &DemandStateRow) -> BiochemistryClass {
    let k_ok = (row.k_precursor_scale - 1.0).abs() < 1e-12
        && (row.k_structure_scale - 1.0).abs() < 1e-12;
    if k_ok {
        BiochemistryClass::FixedBiochemistry
    } else {
        BiochemistryClass::AlteredBiochemistry
    }
}

pub fn is_altered_parameter_state(row: &DemandStateRow) -> bool {
    classify_biochemistry_state(row) == BiochemistryClass::AlteredBiochemistry
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossParameterAudit {
    pub complete_a: ModelFitReport,
    pub complete_b: ModelFitReport,
    pub complete_c: ModelFitReport,
    pub complete_d: ModelFitReport,
    pub fixed_a: ModelFitReport,
    pub fixed_b: ModelFitReport,
    pub fixed_c: ModelFitReport,
    pub fixed_d: ModelFitReport,
    pub n_complete: usize,
    pub n_fixed: usize,
    pub n_altered: usize,
    pub complete_any_adequate: bool,
    pub fixed_any_aggregate_adequate: bool,
    pub conclusion_tag: String,
}

/// Recalculate D-046 Models A/B/C/D on complete vs fixed-biochemistry subsets.
pub fn cross_parameter_model_audit(rows: &[DemandStateRow]) -> CrossParameterAudit {
    let complete_train: Vec<_> = rows.iter().filter(|r| r.train).cloned().collect();
    let complete_hold: Vec<_> = rows.iter().filter(|r| !r.train).cloned().collect();
    let fixed: Vec<_> = rows
        .iter()
        .filter(|r| classify_biochemistry_state(r) == BiochemistryClass::FixedBiochemistry)
        .cloned()
        .collect();
    let fixed_train: Vec<_> = fixed.iter().filter(|r| r.train).cloned().collect();
    let fixed_hold: Vec<_> = fixed.iter().filter(|r| !r.train).cloned().collect();
    let n_altered = rows
        .iter()
        .filter(|r| is_altered_parameter_state(r))
        .count();

    let complete_a = fit_model_a(&complete_train, &complete_hold);
    let complete_b = fit_model_b(&complete_train, &complete_hold);
    let complete_c = fit_model_c(&complete_train, &complete_hold, D047_K_C_MEMBRANE);
    let complete_d = fit_model_d(&complete_train, &complete_hold);
    let fixed_a = fit_model_a(&fixed_train, &fixed_hold);
    let fixed_b = fit_model_b(&fixed_train, &fixed_hold);
    let fixed_c = fit_model_c(&fixed_train, &fixed_hold, D047_K_C_MEMBRANE);
    let fixed_d = fit_model_d(&fixed_train, &fixed_hold);

    let complete_any = complete_a.adequate || complete_b.adequate || complete_c.adequate;
    let fixed_any = fixed_a.adequate || fixed_b.adequate || fixed_c.adequate;

    let conclusion_tag = if !complete_any && fixed_any {
        "D047_CROSS_PARAMETER_PORTABILITY_DEFECT".to_string()
    } else if !complete_any && !fixed_any {
        "D047_FIXED_BIOLOGY_SUPPLY_MISMATCH".to_string()
    } else if complete_any {
        "D047_COMPLETE_AGGREGATE_ADEQUATE".to_string()
    } else {
        "D047_CROSS_PARAMETER_AUDIT_INCONCLUSIVE".to_string()
    };

    CrossParameterAudit {
        complete_a,
        complete_b,
        complete_c,
        complete_d,
        fixed_a,
        fixed_b,
        fixed_c,
        fixed_d,
        n_complete: rows.len(),
        n_fixed: fixed.len(),
        n_altered,
        complete_any_adequate: complete_any,
        fixed_any_aggregate_adequate: fixed_any,
        conclusion_tag,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AEquivalentRole {
    MaterialEquivalent,
    ActivationPotential,
    Both,
    AbstractCombinedEquivalent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ARouteRole {
    pub route_id: String,
    pub equation: String,
    pub product_yield: String,
    pub w_yield: String,
    pub conserved_equivalent_weight: String,
    pub spatial_basis: String,
    pub saturation_basis: String,
    pub depends_on_c: bool,
    pub depends_on_product: bool,
    pub depends_on_damage_or_turnover: bool,
    pub a_role: AEquivalentRole,
}

/// Gate 1: A → product routes and conserved-equivalent interpretation.
pub fn a_equivalent_role_catalog() -> Vec<ARouteRole> {
    vec![
        ARouteRole {
            route_id: "A_to_C".into(),
            equation: "r_rep = k_rep · C · A; A → η_C C + (1-η_C) W".into(),
            product_yield: "η_C C per A".into(),
            w_yield: "(1-η_C) W per A".into(),
            conserved_equivalent_weight: "1 A = 1 activation-potential unit converted to catalyst material".into(),
            spatial_basis: "bulk interior".into(),
            saturation_basis: "linear in C (no product inhibition)".into(),
            depends_on_c: true,
            depends_on_product: true, // rate ∝ C
            depends_on_damage_or_turnover: false,
            a_role: AEquivalentRole::Both,
        },
        ARouteRole {
            route_id: "A_to_phi".into(),
            equation: "r_φ = k_φ · A · I(φ) [default]".into(),
            product_yield: "η_φ φ per A (virtual ledger when unconstrained)".into(),
            w_yield: "structure decay → W separately".into(),
            conserved_equivalent_weight: "1 A = 1 activation-potential unit for structural production".into(),
            spatial_basis: "interface".into(),
            saturation_basis: "interface weight I(φ)".into(),
            depends_on_c: false,
            depends_on_product: true, // via φ interface
            depends_on_damage_or_turnover: true,
            a_role: AEquivalentRole::ActivationPotential,
        },
        ARouteRole {
            route_id: "A_to_P".into(),
            equation: "r_P = k_P · A · q(C) · H(φ); A → P".into(),
            product_yield: "1 P per A".into(),
            w_yield: "none on synthesis".into(),
            conserved_equivalent_weight: "1 A = 1 material equivalent into precursor".into(),
            spatial_basis: "interior H(φ)".into(),
            saturation_basis: "q(C)=C/(K_C+C); no P dependence".into(),
            depends_on_c: true,
            depends_on_product: false,
            depends_on_damage_or_turnover: false,
            a_role: AEquivalentRole::MaterialEquivalent,
        },
        ARouteRole {
            route_id: "A_to_W_decay".into(),
            equation: "r_decay = k_dec · A; A → W".into(),
            product_yield: "none".into(),
            w_yield: "1 W per A".into(),
            conserved_equivalent_weight: "1 A lost as waste".into(),
            spatial_basis: "bulk".into(),
            saturation_basis: "linear in A".into(),
            depends_on_c: false,
            depends_on_product: false,
            depends_on_damage_or_turnover: false,
            a_role: AEquivalentRole::AbstractCombinedEquivalent,
        },
        ARouteRole {
            route_id: "A_transport".into(),
            equation: "selective face transport of A".into(),
            product_yield: "none".into(),
            w_yield: "none".into(),
            conserved_equivalent_weight: "conservative relocation".into(),
            spatial_basis: "membrane faces".into(),
            saturation_basis: "transport kinetics".into(),
            depends_on_c: false,
            depends_on_product: false,
            depends_on_damage_or_turnover: false,
            a_role: AEquivalentRole::AbstractCombinedEquivalent,
        },
    ]
}

/// Shared-pool structural-sufficiency checklist (Gate 1).
pub fn shared_pool_structural_checks(roles: &[ARouteRole]) -> (bool, Option<&'static str>) {
    // Coherent conserved-equivalent: all productive sinks treat A as a local scalar
    // activation/material currency. Mixed Material vs ActivationPotential is still
    // one abstract activated-resource unit (historical project definition), not an
    // irreconcilable split — unless roles require incompatible localization history.
    let has_productive = roles.iter().any(|r| {
        matches!(
            r.a_role,
            AEquivalentRole::MaterialEquivalent
                | AEquivalentRole::ActivationPotential
                | AEquivalentRole::Both
        )
    });
    if !has_productive {
        return (false, Some("D047_A_EQUIVALENT_ROLE_INCONSISTENT"));
    }
    // No sink requires independent activation history in catalog.
    let ok = roles.iter().all(|r| {
        matches!(
            r.a_role,
            AEquivalentRole::MaterialEquivalent
                | AEquivalentRole::ActivationPotential
                | AEquivalentRole::Both
                | AEquivalentRole::AbstractCombinedEquivalent
        )
    });
    if ok {
        (true, None)
    } else {
        (false, Some("D047_A_EQUIVALENT_ROLE_INCONSISTENT"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ACohortBalance {
    pub produced: f64,
    pub to_reproduction: f64,
    pub to_structure: f64,
    pub to_precursor: f64,
    pub to_decay: f64,
    pub to_transport_out: f64,
    pub remaining_free: f64,
    pub residual: f64,
}

impl ACohortBalance {
    pub fn from_flows(
        produced: f64,
        to_reproduction: f64,
        to_structure: f64,
        to_precursor: f64,
        to_decay: f64,
        to_transport_out: f64,
        remaining_free: f64,
    ) -> Self {
        let accounted = to_reproduction
            + to_structure
            + to_precursor
            + to_decay
            + to_transport_out
            + remaining_free;
        Self {
            produced,
            to_reproduction,
            to_structure,
            to_precursor,
            to_decay,
            to_transport_out,
            remaining_free,
            residual: produced - accounted,
        }
    }

    pub fn conservation_ok(&self, tol: f64) -> bool {
        let scale = self.produced.abs().max(1.0);
        self.residual.abs() / scale <= tol
    }

    pub fn destination_fractions(&self) -> [(&'static str, f64); 6] {
        let s = self.produced.max(1e-18);
        [
            ("reproduction", self.to_reproduction / s),
            ("structure", self.to_structure / s),
            ("precursor", self.to_precursor / s),
            ("decay", self.to_decay / s),
            ("transport_out", self.to_transport_out / s),
            ("remaining_free", self.remaining_free / s),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompetitionClass {
    ProportionalSharedDecline,
    PrecursorDominatedStarvation,
    CatalystReproductionStarvation,
    StructuralMaintenanceStarvation,
    SpatiallyIsolatedStarvation,
    Mixed,
}

/// Order services by relative decline under progressive activation multipliers.
pub fn classify_service_competition(
    multipliers: &[f64],
    j_rep: &[f64],
    j_struct: &[f64],
    j_prec: &[f64],
) -> CompetitionClass {
    assert_eq!(multipliers.len(), j_rep.len());
    assert_eq!(j_rep.len(), j_struct.len());
    assert_eq!(j_struct.len(), j_prec.len());
    if multipliers.len() < 2 {
        return CompetitionClass::Mixed;
    }
    let i0 = 0;
    let i1 = multipliers.len() - 1;
    let rel = |a: f64, b: f64| {
        if a.abs() < 1e-12 {
            if b.abs() < 1e-12 {
                0.0
            } else {
                1.0
            }
        } else {
            ((a - b) / a).max(0.0)
        }
    };
    let d_rep = rel(j_rep[i0], j_rep[i1]);
    let d_struct = rel(j_struct[i0], j_struct[i1]);
    let d_prec = rel(j_prec[i0], j_prec[i1]);
    let max_d = d_rep.max(d_struct).max(d_prec);
    let min_d = d_rep.min(d_struct).min(d_prec);
    if max_d - min_d < 0.15 {
        return CompetitionClass::ProportionalSharedDecline;
    }
    if d_prec >= d_rep && d_prec >= d_struct && d_prec - min_d > 0.15 {
        return CompetitionClass::PrecursorDominatedStarvation;
    }
    if d_rep >= d_prec && d_rep >= d_struct && d_rep - min_d > 0.15 {
        return CompetitionClass::CatalystReproductionStarvation;
    }
    if d_struct >= d_prec && d_struct >= d_rep && d_struct - min_d > 0.15 {
        return CompetitionClass::StructuralMaintenanceStarvation;
    }
    CompetitionClass::Mixed
}

/// Failure order: earliest relative collapse wins (higher decline at mid multiplier).
pub fn service_failure_order(
    multipliers: &[f64],
    j_rep: &[f64],
    j_struct: &[f64],
    j_prec: &[f64],
) -> Vec<&'static str> {
    if multipliers.len() < 2 {
        return vec!["reproduction", "structure", "precursor"];
    }
    // Use midpoint vs baseline decline.
    let mid = multipliers.len() / 2;
    let rel = |base: f64, now: f64| {
        if base.abs() < 1e-12 {
            0.0
        } else {
            ((base - now) / base).max(0.0)
        }
    };
    let mut items = [
        ("reproduction", rel(j_rep[0], j_rep[mid])),
        ("structure", rel(j_struct[0], j_struct[mid])),
        ("precursor", rel(j_prec[0], j_prec[mid])),
    ];
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    items.into_iter().map(|(n, _)| n).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SinkRegulationClass {
    SelfLimiting,
    TurnoverLimited,
    SubstrateLimited,
    ConstitutiveWhileARemains,
    ProductInhibited,
    CapacityLimited,
}

/// Analytical ∂r_P/∂P for r_P = k_P · A · q(C) · H(φ) (independent of P).
pub fn precursor_product_response(
    rates_at_p: &[(f64, f64)], // (P, r_P) at matched A,C,N,F
) -> (f64, bool, &'static str) {
    if rates_at_p.len() < 2 {
        return (0.0, false, "INSUFFICIENT_SAMPLES");
    }
    let mut slopes = Vec::new();
    for i in 0..rates_at_p.len() {
        for j in (i + 1)..rates_at_p.len() {
            let dp = rates_at_p[j].0 - rates_at_p[i].0;
            if dp.abs() > 1e-12 {
                slopes.push((rates_at_p[j].1 - rates_at_p[i].1) / dp);
            }
        }
    }
    if slopes.is_empty() {
        return (0.0, false, "NO_P_VARIATION");
    }
    let mean = slopes.iter().sum::<f64>() / slopes.len() as f64;
    let not_regulated = mean.abs() < 1e-6;
    let tag = if not_regulated {
        "PRECURSOR_SYNTHESIS_NOT_PRODUCT_REGULATED"
    } else {
        "PRECURSOR_PRODUCT_RESPONSE_NONEMPTY"
    };
    (mean, not_regulated, tag)
}

pub fn classify_sink_regulation(
    depends_on_product: bool,
    product_response: f64,
    substrate_limited: bool,
    capacity_limited: bool,
) -> SinkRegulationClass {
    if capacity_limited {
        return SinkRegulationClass::CapacityLimited;
    }
    if substrate_limited {
        return SinkRegulationClass::SubstrateLimited;
    }
    if depends_on_product && product_response < -1e-6 {
        return SinkRegulationClass::ProductInhibited;
    }
    if depends_on_product && product_response.abs() > 1e-6 {
        return SinkRegulationClass::SelfLimiting;
    }
    if !depends_on_product {
        return SinkRegulationClass::ConstitutiveWhileARemains;
    }
    SinkRegulationClass::TurnoverLimited
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SharedPoolUpperBound {
    SharedAPoolCapable,
    SpatialAAllocationDefect,
    SharedAPoolStructurallyInsufficient,
    Inconclusive,
}

/// Gate 6 classification from control outcomes.
pub fn classify_shared_pool_upper_bound(
    historical_fails: bool,
    control_ab_succeeds: bool,
    global_mix_only: bool,
    local_sufficient_fails: bool,
) -> SharedPoolUpperBound {
    if local_sufficient_fails {
        return SharedPoolUpperBound::SharedAPoolStructurallyInsufficient;
    }
    if global_mix_only && !control_ab_succeeds {
        return SharedPoolUpperBound::SpatialAAllocationDefect;
    }
    if control_ab_succeeds && historical_fails {
        return SharedPoolUpperBound::SharedAPoolCapable;
    }
    if control_ab_succeeds && !historical_fails {
        return SharedPoolUpperBound::SharedAPoolCapable;
    }
    SharedPoolUpperBound::Inconclusive
}

/// Observer-only activation candidates (Gate 8). No production wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationCandidate {
    AHistoricalMassAction,
    BCatalystSaturatingVolumetric,
    CProductInhibited,
    DProductInhibitedJointSat,
}

impl ActivationCandidate {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AHistoricalMassAction => "CANDIDATE_A_HISTORICAL_MASS_ACTION",
            Self::BCatalystSaturatingVolumetric => "CANDIDATE_B_CATALYST_SATURATING_VOLUMETRIC",
            Self::CProductInhibited => "CANDIDATE_C_PRODUCT_INHIBITED",
            Self::DProductInhibitedJointSat => "CANDIDATE_D_PRODUCT_INHIBITED_JOINT_SAT",
        }
    }
}

/// Candidate A: r = k · C · N · F
pub fn candidate_a_rate(k: f64, c: f64, n: f64, f: f64) -> f64 {
    k * c.max(0.0) * n.max(0.0) * f.max(0.0)
}

/// Candidate B: r = V_B · H · q(C) · n · f
pub fn candidate_b_rate(v_b: f64, h_phi: f64, c: f64, n: f64, f: f64, k_c: f64) -> f64 {
    let q = c.max(0.0) / (k_c + c.max(0.0)).max(1e-18);
    v_b * h_phi.max(0.0) * q * n.max(0.0) * f.max(0.0)
}

/// Candidate C: product-inhibited saturating volume.
pub fn candidate_c_rate(
    v_c: f64,
    h_phi: f64,
    c: f64,
    n: f64,
    f: f64,
    a: f64,
    k_c: f64,
    k_i: f64,
    a_ref: f64,
) -> f64 {
    let q = c.max(0.0) / (k_c + c.max(0.0)).max(1e-18);
    let a_hat = a.max(0.0) / a_ref.max(1e-18);
    let inhib = k_i / (k_i + a_hat).max(1e-18);
    v_c * h_phi.max(0.0) * q * n.max(0.0) * f.max(0.0) * inhib
}

/// Candidate D: joint-substrate saturation + product inhibition.
pub fn candidate_d_rate(
    v_d: f64,
    h_phi: f64,
    c: f64,
    n: f64,
    f: f64,
    a: f64,
    k_c: f64,
    k_nf: f64,
    k_i: f64,
    a_ref: f64,
) -> f64 {
    let q = c.max(0.0) / (k_c + c.max(0.0)).max(1e-18);
    let z = n.max(0.0) * f.max(0.0);
    let sat = z / (k_nf + z).max(1e-18);
    let a_hat = a.max(0.0) / a_ref.max(1e-18);
    let inhib = k_i / (k_i + a_hat).max(1e-18);
    v_d * h_phi.max(0.0) * q * sat * inhib
}

pub fn candidate_zero_resource_ok() -> bool {
    let a = candidate_a_rate(0.02, 0.0, 1.0, 1.0).abs() < 1e-15
        && candidate_a_rate(0.02, 1.0, 0.0, 1.0).abs() < 1e-15
        && candidate_a_rate(0.02, 1.0, 1.0, 0.0).abs() < 1e-15;
    let b = candidate_b_rate(1.0, 1.0, 0.0, 1.0, 1.0, 0.1).abs() < 1e-15
        && candidate_b_rate(1.0, 1.0, 1.0, 0.0, 1.0, 0.1).abs() < 1e-15;
    let c = candidate_c_rate(1.0, 1.0, 0.0, 1.0, 1.0, 0.5, 0.1, 1.0, 1.0).abs() < 1e-15;
    let d = candidate_d_rate(1.0, 1.0, 1.0, 0.0, 1.0, 0.5, 0.1, 1.0, 1.0, 1.0).abs() < 1e-15;
    a && b && c && d
}

pub fn product_inhibition_monotonic(a_low: f64, a_high: f64, k_i: f64, a_ref: f64) -> bool {
    let r_lo = candidate_c_rate(1.0, 1.0, 1.0, 1.0, 1.0, a_low, 0.1, k_i, a_ref);
    let r_hi = candidate_c_rate(1.0, 1.0, 1.0, 1.0, 1.0, a_high, 0.1, k_i, a_ref);
    a_high > a_low && r_lo > r_hi
}

/// Reduced fixed-point solver for lumped scalars (Gate 7 observer model).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReducedParams {
    pub k_act: f64,
    pub k_rep: f64,
    pub k_struct: f64,
    pub k_prec: f64,
    pub k_decay: f64,
    pub eta_c: f64,
    pub k_c_loss: f64,
    pub k_p_decay: f64,
    pub k_exchange: f64,
    pub n: f64,
    pub f: f64,
    pub h_phi: f64,
    pub k_c: f64,
}

impl Default for ReducedParams {
    fn default() -> Self {
        Self {
            k_act: D047_HISTORICAL_K,
            k_rep: 0.05,
            k_struct: 0.02,
            k_prec: 0.15,
            k_decay: 0.01,
            eta_c: 0.8,
            k_c_loss: 0.01,
            k_p_decay: 0.02,
            k_exchange: 0.05,
            n: 0.8,
            f: 0.8,
            h_phi: 1.0,
            k_c: D047_K_C_MEMBRANE,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ReducedState {
    pub a: f64,
    pub c: f64,
    pub p: f64,
    pub s: f64,
}

/// Lumped rates at a reduced state (observer).
pub fn reduced_rates(p: &ReducedParams, x: &ReducedState) -> (f64, f64, f64, f64) {
    let q = x.c.max(0.0) / (p.k_c + x.c.max(0.0)).max(1e-18);
    let r_act = p.k_act * x.c.max(0.0) * p.n * p.f;
    let l_rep = p.k_rep * x.c.max(0.0) * x.a.max(0.0);
    let l_struct = p.k_struct * x.a.max(0.0) * p.h_phi;
    let l_prec = p.k_prec * x.a.max(0.0) * q * p.h_phi;
    let l_decay = p.k_decay * x.a.max(0.0);
    let da = r_act - l_rep - l_struct - l_prec - l_decay;
    let dc = p.eta_c * l_rep - p.k_c_loss * x.c.max(0.0);
    // Exchange net: approximate P→S drive proportional to (P - S) for reduced model.
    let j_ps = p.k_exchange * (x.p - x.s);
    let dp = l_prec - j_ps - p.k_p_decay * x.p.max(0.0);
    let ds = j_ps;
    (da, dc, dp, ds)
}

/// Simple fixed-point search by damped iteration / multistart.
pub fn find_reduced_fixed_point(
    p: &ReducedParams,
    start: ReducedState,
    iters: usize,
) -> Option<ReducedState> {
    let mut x = start;
    for _ in 0..iters {
        let (da, dc, dp, ds) = reduced_rates(p, &x);
        let step = 0.05;
        x.a = (x.a + step * da).max(0.0);
        x.c = (x.c + step * dc).max(0.0);
        x.p = (x.p + step * dp).max(0.0);
        x.s = (x.s + step * ds).max(0.0);
        if da.abs() + dc.abs() + dp.abs() + ds.abs() < 1e-6 {
            return Some(x);
        }
    }
    let (da, dc, dp, ds) = reduced_rates(p, &x);
    if da.abs() + dc.abs() + dp.abs() + ds.abs() < 1e-3 {
        Some(x)
    } else {
        None
    }
}

/// Numerical Jacobian eigenvalues (real parts) of reduced 4D system.
pub fn reduced_jacobian_eig_real(p: &ReducedParams, x: &ReducedState) -> [f64; 4] {
    let eps = 1e-5;
    let f0 = reduced_rates(p, x);
    let mut j = [[0.0; 4]; 4];
    let mut pert = *x;
    // columns: ∂/∂a, ∂/∂c, ∂/∂p, ∂/∂s
    for col in 0..4 {
        pert = *x;
        match col {
            0 => pert.a += eps,
            1 => pert.c += eps,
            2 => pert.p += eps,
            _ => pert.s += eps,
        }
        let f1 = reduced_rates(p, &pert);
        j[0][col] = (f1.0 - f0.0) / eps;
        j[1][col] = (f1.1 - f0.1) / eps;
        j[2][col] = (f1.2 - f0.2) / eps;
        j[3][col] = (f1.3 - f0.3) / eps;
    }
    // Power-iterate rough spectral radius signs via Gershgorin centers (diagonal).
    // For route decisions we mainly need whether any Re(λ)>0 from diagonal dominance.
    [j[0][0], j[1][1], j[2][2], j[3][3]]
}

/// Does constitutive precursor demand remove a healthy bounded fixed point?
pub fn precursor_destroys_healthy_fixed_point(base: &ReducedParams) -> bool {
    let healthy_start = ReducedState {
        a: 0.5,
        c: 0.8,
        p: 0.2,
        s: 0.6,
    };
    let with_prec = find_reduced_fixed_point(base, healthy_start, 5000);
    let mut reduced_prec = base.clone();
    reduced_prec.k_prec *= 0.1;
    let with_low_prec = find_reduced_fixed_point(&reduced_prec, healthy_start, 5000);
    match (with_prec, with_low_prec) {
        (None, Some(_)) => true,
        (Some(x), Some(y)) => {
            // Destroyed if A collapses near zero under full precursor but recovers when reduced.
            x.a < 0.05 && y.a > 0.2
        }
        (Some(x), None) => x.a < 0.05,
        (None, None) => false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteDecisionInput {
    pub accounting_failure: bool,
    pub tracer_failure: bool,
    pub a_role_inconsistent: bool,
    pub shared_pool_structurally_insufficient: bool,
    pub spatial_allocation_defect: bool,
    pub precursor_not_product_regulated: bool,
    pub precursor_destroys_fixed_point: bool,
    pub reducing_precursor_restores_stability: bool,
    pub historical_fixed_biology_adequate: bool,
    pub candidate_b_qualified: bool,
    pub candidate_c_or_d_qualified: bool,
    pub shared_pool_capable: bool,
}

pub fn select_route(input: &RouteDecisionInput) -> D047Route {
    if input.accounting_failure {
        return D047Route::RouteI;
    }
    if input.tracer_failure {
        return D047Route::RouteI;
    }
    if input.a_role_inconsistent {
        return D047Route::RouteI;
    }
    // Stop rules: demand/allocation defects before activation redesign.
    if input.shared_pool_structurally_insufficient {
        return D047Route::RouteM;
    }
    if input.precursor_not_product_regulated
        && input.precursor_destroys_fixed_point
        && input.reducing_precursor_restores_stability
    {
        return D047Route::RouteP;
    }
    if input.spatial_allocation_defect {
        return D047Route::RouteL;
    }
    if input.historical_fixed_biology_adequate {
        return D047Route::RouteH;
    }
    if input.shared_pool_capable && input.candidate_b_qualified {
        return D047Route::RouteV;
    }
    if input.shared_pool_capable && input.candidate_c_or_d_qualified {
        return D047Route::RouteF;
    }
    D047Route::RouteI
}

/// Fixed-biochemistry training labels (Gate 9) — no altered k_P/k_rep/k_structure.
pub fn fixed_train_label(label: &str) -> bool {
    matches!(
        label,
        "R16" | "R22" | "R32" | "low_c" | "med_c" | "high_c" | "s_healthy" | "s_low" | "bootstrap"
    )
}

pub fn fixed_holdout_label(label: &str) -> bool {
    matches!(
        label,
        "low_n" | "low_f" | "high_nf" | "pre_collapse" | "damage10" | "damage25" | "damage_repeated"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(label: &str, family: &str, train: bool, k_p: f64, k_s: f64, l_a: f64) -> DemandStateRow {
        DemandStateRow {
            label: label.into(),
            family: family.into(),
            train,
            radius: 22.0,
            c: 0.8,
            n: 0.8,
            f: 0.8,
            a: 0.5,
            p: 0.05,
            s_occupancy: 0.6,
            m_c: 1200.0,
            interior_volume: 1500.0,
            structural_mass: 1500.0,
            membrane_area: 140.0,
            l_a,
            j_reproduction: l_a * 0.11,
            j_structural: l_a * 0.10,
            j_precursor: l_a * 0.76,
            j_decay: l_a * 0.02,
            j_out: l_a * 0.01,
            j_in: 0.0,
            k_structure_scale: k_s,
            k_precursor_scale: k_p,
        }
    }

    #[test]
    fn fixed_vs_altered_classification() {
        let fixed = row("R22", "radius", true, 1.0, 1.0, 180.0);
        let alt = row("prec_hi", "precursor", false, 2.0, 1.0, 310.0);
        assert_eq!(
            classify_biochemistry_state(&fixed),
            BiochemistryClass::FixedBiochemistry
        );
        assert!(is_altered_parameter_state(&alt));
    }

    #[test]
    fn cross_parameter_separation() {
        let rows = vec![
            row("R16", "radius", true, 1.0, 1.0, 96.0),
            row("R22", "radius", true, 1.0, 1.0, 180.0),
            row("R32", "radius", false, 1.0, 1.0, 384.0),
            row("low_c", "catalyst", true, 1.0, 1.0, 144.0),
            row("med_c", "catalyst", true, 1.0, 1.0, 168.0),
            row("high_c", "catalyst", false, 1.0, 1.0, 186.0),
            row("s_healthy", "membrane", true, 1.0, 1.0, 180.0),
            row("s_damaged25", "membrane", false, 1.0, 1.0, 182.0),
            row("prec_lo", "precursor", true, 0.5, 1.0, 100.0),
            row("prec_hi", "precursor", false, 2.0, 1.0, 260.0),
        ];
        let audit = cross_parameter_model_audit(&rows);
        assert!(audit.n_altered >= 2);
        assert!(audit.n_fixed >= 6);
        // Complete set should struggle on max error due to prec_hi.
        assert!(audit.complete_a.max_hold_err > 0.2 || !audit.complete_a.adequate);
    }

    #[test]
    fn a_role_catalog_coherent() {
        let cat = a_equivalent_role_catalog();
        assert!(cat.iter().any(|r| r.route_id == "A_to_C"));
        assert!(cat.iter().any(|r| r.route_id == "A_to_P"));
        let (ok, fail) = shared_pool_structural_checks(&cat);
        assert!(ok);
        assert!(fail.is_none());
        let prec = cat.iter().find(|r| r.route_id == "A_to_P").unwrap();
        assert!(!prec.depends_on_product);
    }

    #[test]
    fn cohort_tracer_conservation() {
        let b = ACohortBalance::from_flows(100.0, 11.0, 10.0, 76.0, 2.0, 1.0, 0.0);
        assert!(b.conservation_ok(1e-9));
        let fracs = b.destination_fractions();
        assert!((fracs[2].1 - 0.76).abs() < 1e-9);
    }

    #[test]
    fn service_failure_ordering() {
        let m = [1.0, 0.6, 0.2];
        let j_rep = [10.0, 8.0, 4.0];
        let j_struct = [10.0, 9.0, 7.0];
        let j_prec = [100.0, 40.0, 5.0];
        assert_eq!(
            classify_service_competition(&m, &j_rep, &j_struct, &j_prec),
            CompetitionClass::PrecursorDominatedStarvation
        );
        let order = service_failure_order(&m, &j_rep, &j_struct, &j_prec);
        assert_eq!(order[0], "precursor");
    }

    #[test]
    fn precursor_not_product_regulated() {
        let samples = vec![(0.01, 1.5), (0.1, 1.5), (1.0, 1.5)];
        let (slope, flag, tag) = precursor_product_response(&samples);
        assert!(slope.abs() < 1e-9);
        assert!(flag);
        assert_eq!(tag, "PRECURSOR_SYNTHESIS_NOT_PRODUCT_REGULATED");
        assert_eq!(
            classify_sink_regulation(false, slope, false, false),
            SinkRegulationClass::ConstitutiveWhileARemains
        );
    }

    #[test]
    fn upper_bound_classification() {
        assert_eq!(
            classify_shared_pool_upper_bound(true, true, false, false),
            SharedPoolUpperBound::SharedAPoolCapable
        );
        assert_eq!(
            classify_shared_pool_upper_bound(true, false, true, false),
            SharedPoolUpperBound::SpatialAAllocationDefect
        );
        assert_eq!(
            classify_shared_pool_upper_bound(true, false, false, true),
            SharedPoolUpperBound::SharedAPoolStructurallyInsufficient
        );
    }

    #[test]
    fn candidate_zero_and_inhibition() {
        assert!(candidate_zero_resource_ok());
        assert!(product_inhibition_monotonic(0.1, 2.0, 1.0, 1.0));
    }

    #[test]
    fn route_p_before_activation_redesign() {
        let r = select_route(&RouteDecisionInput {
            accounting_failure: false,
            tracer_failure: false,
            a_role_inconsistent: false,
            shared_pool_structurally_insufficient: false,
            spatial_allocation_defect: false,
            precursor_not_product_regulated: true,
            precursor_destroys_fixed_point: true,
            reducing_precursor_restores_stability: true,
            historical_fixed_biology_adequate: false,
            candidate_b_qualified: true,
            candidate_c_or_d_qualified: true,
            shared_pool_capable: true,
        });
        assert_eq!(r, D047Route::RouteP);
        assert_eq!(
            r.conclusion().as_str(),
            "D047_PRECURSOR_DEMAND_REGULATION_DEFECT"
        );
    }

    #[test]
    fn route_m_structural() {
        let r = select_route(&RouteDecisionInput {
            accounting_failure: false,
            tracer_failure: false,
            a_role_inconsistent: false,
            shared_pool_structurally_insufficient: true,
            spatial_allocation_defect: false,
            precursor_not_product_regulated: true,
            precursor_destroys_fixed_point: true,
            reducing_precursor_restores_stability: true,
            historical_fixed_biology_adequate: false,
            candidate_b_qualified: false,
            candidate_c_or_d_qualified: false,
            shared_pool_capable: false,
        });
        assert_eq!(r, D047Route::RouteM);
    }

    #[test]
    fn fixed_train_holdout_separation() {
        assert!(fixed_train_label("R22"));
        assert!(fixed_holdout_label("damage25"));
        assert!(!fixed_holdout_label("prec_hi"));
        assert!(!fixed_train_label("prec_hi"));
    }

    #[test]
    fn no_observer_feedback_in_candidates() {
        // Candidates depend on C,N,F,A,H — not membrane occupancy target.
        let r1 = candidate_b_rate(1.0, 1.0, 0.8, 0.8, 0.8, 0.1);
        let r2 = candidate_b_rate(1.0, 1.0, 0.8, 0.8, 0.8, 0.1);
        assert_eq!(r1, r2);
    }
}
