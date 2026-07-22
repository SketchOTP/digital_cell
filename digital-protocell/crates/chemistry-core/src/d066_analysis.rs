//! D-066 smooth-membrane activation utilization and local substrate access audit.
//! Pure classifiers/helpers; shadow/observer-only. No production biology change.
//!
//! Preserves D-065 conclusion `D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED`.
//! Rate law under audit is schema-2 catalyst-saturating volume activation:
//! `r = V_A · H(φ) · q_C(C) · N̂ · F̂` with `q_C = C/(K_C+C)`, `N̂ = N/N_ref`,
//! `F̂ = F/F_ref`, `H(φ) = φ²(3−2φ)`.

use crate::d050_analysis::schema2_activation_rate;
use crate::d053_analysis::{D053_FITTED_K_C, D053_FITTED_V_A, D053_F_REF, D053_N_REF};
use crate::d064_analysis::{chi_ratio, D064_FROZEN_KT};
use crate::fields::interior_weight;
use serde::{Deserialize, Serialize};

pub const D066_PROJECT_ID: &str = "D-066";
pub const D066_AGENT_MEMORY_ID: &str =
    "D-20260721-d066-smooth-membrane-activation-utilization-audit";
pub const D066_STARTING_COMMIT: &str = "8def238";
pub const D066_STARTING_TAG: &str = "D-065-canonical-resource-topology-requalification";
pub const D066_D065_CONCLUSION: &str = "D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED";
pub const D066_RECORD_DELIVERY: &str = "SMOOTH_MEMBRANE_RESOURCE_DELIVERY_SUFFICIENT";
pub const D066_RECORD_CAUSE: &str = "ACTIVATION_UTILIZATION_CAUSE_UNRESOLVED";
pub const D066_FROZEN_KT: f64 = D064_FROZEN_KT;
pub const D066_A_RETENTION_TARGET: f64 = 0.80;
pub const D066_CHI_VIABLE: f64 = 1.05;
pub const D066_V_A: f64 = D053_FITTED_V_A;
pub const D066_K_C: f64 = D053_FITTED_K_C;
pub const D066_N_REF: f64 = D053_N_REF;
pub const D066_F_REF: f64 = D053_F_REF;
pub const D066_EQUATION_VERSION: &str =
    "membrane_metabolism_v13_catalyst_saturating_activation";
pub const D066_EPS: f64 = 1e-18;
pub const D066_LEDGER_TOL: f64 = 1e-4;

/// One primary conclusion per pipeline. Serialized SCREAMING_SNAKE via `as_str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D066PrimaryConclusion {
    ActivationAcceptanceExecutionDefect,
    ActivationSubstrateOverlapLimit,
    CatalystActivationSupportLimit,
    FrozenActivationCapacityLimit,
    ActivationSufficientADemandLimited,
    WasteRejectionMasksActivationResult,
    MultipleActivationABalanceLimits,
    ActivationUtilizationAuditInconclusive,
    D065ActivationRouteNotReproduced,
    ActivationLineageUnresolved,
    ActivationRuntimeParityFailure,
    InternalResourceFateAccountingFailure,
    ALedgerFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D066PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationAcceptanceExecutionDefect => {
                "D066_ACTIVATION_ACCEPTANCE_EXECUTION_DEFECT"
            }
            Self::ActivationSubstrateOverlapLimit => "D066_ACTIVATION_SUBSTRATE_OVERLAP_LIMIT",
            Self::CatalystActivationSupportLimit => "D066_CATALYST_ACTIVATION_SUPPORT_LIMIT",
            Self::FrozenActivationCapacityLimit => "D066_FROZEN_ACTIVATION_CAPACITY_LIMIT",
            Self::ActivationSufficientADemandLimited => {
                "D066_ACTIVATION_SUFFICIENT_A_DEMAND_LIMITED"
            }
            Self::WasteRejectionMasksActivationResult => {
                "D066_WASTE_REJECTION_MASKS_ACTIVATION_RESULT"
            }
            Self::MultipleActivationABalanceLimits => "D066_MULTIPLE_ACTIVATION_A_BALANCE_LIMITS",
            Self::ActivationUtilizationAuditInconclusive => {
                "D066_ACTIVATION_UTILIZATION_AUDIT_INCONCLUSIVE"
            }
            Self::D065ActivationRouteNotReproduced => "D066_D065_ACTIVATION_ROUTE_NOT_REPRODUCED",
            Self::ActivationLineageUnresolved => "D066_ACTIVATION_LINEAGE_UNRESOLVED",
            Self::ActivationRuntimeParityFailure => "D066_ACTIVATION_RUNTIME_PARITY_FAILURE",
            Self::InternalResourceFateAccountingFailure => {
                "D066_INTERNAL_RESOURCE_FATE_ACCOUNTING_FAILURE"
            }
            Self::ALedgerFailure => "D066_A_LEDGER_FAILURE",
            Self::WorkspaceScopeNotIsolated => "D066_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D066_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D066_NUMERICAL_FAILURE",
            Self::Fail => "D066_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D066Route {
    X,
    O,
    C,
    K,
    D,
    W,
    M,
    I,
}

impl D066Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "Route_X_activation_acceptance_execution_defect",
            Self::O => "Route_O_activation_substrate_overlap_limit",
            Self::C => "Route_C_catalyst_activation_support_limit",
            Self::K => "Route_K_frozen_activation_capacity_limit",
            Self::D => "Route_D_activation_sufficient_a_demand_limited",
            Self::W => "Route_W_waste_rejection_masks_activation_result",
            Self::M => "Route_M_multiple_activation_a_balance_limits",
            Self::I => "Route_I_activation_utilization_audit_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D066PrimaryConclusion {
        match self {
            Self::X => D066PrimaryConclusion::ActivationAcceptanceExecutionDefect,
            Self::O => D066PrimaryConclusion::ActivationSubstrateOverlapLimit,
            Self::C => D066PrimaryConclusion::CatalystActivationSupportLimit,
            Self::K => D066PrimaryConclusion::FrozenActivationCapacityLimit,
            Self::D => D066PrimaryConclusion::ActivationSufficientADemandLimited,
            Self::W => D066PrimaryConclusion::WasteRejectionMasksActivationResult,
            Self::M => D066PrimaryConclusion::MultipleActivationABalanceLimits,
            Self::I => D066PrimaryConclusion::ActivationUtilizationAuditInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationLimiterClass {
    NLimited,
    FLimited,
    CLimitedRate,
    SupportLimited,
    TimestepLimited,
    ProductCeilingLimited,
    NoLimit,
    MultipleLimits,
}

impl ActivationLimiterClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NLimited => "N_LIMITED",
            Self::FLimited => "F_LIMITED",
            Self::CLimitedRate => "C_LIMITED_RATE",
            Self::SupportLimited => "SUPPORT_LIMITED",
            Self::TimestepLimited => "TIMESTEP_LIMITED",
            Self::ProductCeilingLimited => "PRODUCT_CEILING_LIMITED",
            Self::NoLimit => "NO_LIMIT",
            Self::MultipleLimits => "MULTIPLE_LIMITS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpatialOverlapClass {
    ActivationSubstrateOverlapAdequate,
    BoundaryResourceCoreCatalystSeparation,
    NPenetrationLimit,
    FPenetrationLimit,
    CatalystSupportMismatch,
    MultipleSpatialOverlapLimits,
}

impl SpatialOverlapClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationSubstrateOverlapAdequate => {
                "ACTIVATION_SUBSTRATE_OVERLAP_ADEQUATE"
            }
            Self::BoundaryResourceCoreCatalystSeparation => {
                "BOUNDARY_RESOURCE_CORE_CATALYST_SEPARATION"
            }
            Self::NPenetrationLimit => "N_PENETRATION_LIMIT",
            Self::FPenetrationLimit => "F_PENETRATION_LIMIT",
            Self::CatalystSupportMismatch => "CATALYST_SUPPORT_MISMATCH",
            Self::MultipleSpatialOverlapLimits => "MULTIPLE_SPATIAL_OVERLAP_LIMITS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceUtilizationClass {
    HighDeliveryHighUtilization,
    HighDeliveryLowUtilization,
    RapidReexport,
    NonproductiveInternalAccumulation,
    ResourceFateUnresolved,
}

impl ResourceUtilizationClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HighDeliveryHighUtilization => "HIGH_DELIVERY_HIGH_UTILIZATION",
            Self::HighDeliveryLowUtilization => "HIGH_DELIVERY_LOW_UTILIZATION",
            Self::RapidReexport => "RAPID_REEXPORT",
            Self::NonproductiveInternalAccumulation => "NONPRODUCTIVE_INTERNAL_ACCUMULATION",
            Self::ResourceFateUnresolved => "RESOURCE_FATE_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapacityClass {
    ActivationCatalystSaturated,
    ActivationResourceSaturated,
    ActivationSupportCapacityLimit,
    ActivationRateConstantLimit,
    ActivationCapacityAdequate,
    ActivationCapacityInconclusive,
}

impl CapacityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationCatalystSaturated => "ACTIVATION_CATALYST_SATURATED",
            Self::ActivationResourceSaturated => "ACTIVATION_RESOURCE_SATURATED",
            Self::ActivationSupportCapacityLimit => "ACTIVATION_SUPPORT_CAPACITY_LIMIT",
            Self::ActivationRateConstantLimit => "ACTIVATION_RATE_CONSTANT_LIMIT",
            Self::ActivationCapacityAdequate => "ACTIVATION_CAPACITY_ADEQUATE",
            Self::ActivationCapacityInconclusive => "ACTIVATION_CAPACITY_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CatalystSupportClass {
    TotalCLimit,
    CSpatialSupportLimit,
    CNotPrimaryActivationLimit,
    CatalystActivationRoleInconclusive,
}

impl CatalystSupportClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TotalCLimit => "TOTAL_C_LIMIT",
            Self::CSpatialSupportLimit => "C_SPATIAL_SUPPORT_LIMIT",
            Self::CNotPrimaryActivationLimit => "C_NOT_PRIMARY_ACTIVATION_LIMIT",
            Self::CatalystActivationRoleInconclusive => "CATALYST_ACTIVATION_ROLE_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ADemandClass {
    GrossActivationBelowTotalDemand,
    GrossActivationSufficientNetTransportLimit,
    PrecursorDemandDominant,
    StructuralDemandDominant,
    CatalystDemandDominant,
    ADecayDominant,
    MultipleADemands,
    ABalanceUnresolved,
}

impl ADemandClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GrossActivationBelowTotalDemand => "GROSS_ACTIVATION_BELOW_TOTAL_DEMAND",
            Self::GrossActivationSufficientNetTransportLimit => {
                "GROSS_ACTIVATION_SUFFICIENT_NET_TRANSPORT_LIMIT"
            }
            Self::PrecursorDemandDominant => "PRECURSOR_DEMAND_DOMINANT",
            Self::StructuralDemandDominant => "STRUCTURAL_DEMAND_DOMINANT",
            Self::CatalystDemandDominant => "CATALYST_DEMAND_DOMINANT",
            Self::ADecayDominant => "A_DECAY_DOMINANT",
            Self::MultipleADemands => "MULTIPLE_A_DEMANDS",
            Self::ABalanceUnresolved => "A_BALANCE_UNRESOLVED",
        }
    }
}

/// Activation-lineage report documenting the rate law and its inputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivationLineageReport {
    pub equation_version: String,
    pub rate_law: String,
    pub v_a: f64,
    pub k_c: f64,
    pub n_ref: f64,
    pub f_ref: f64,
    pub spatial_support: String,
    pub stoichiometry: String,
    pub d065_conclusion_preserved: String,
    pub zero_resource_controls_pass: bool,
    pub bounded_high_c_pass: bool,
    pub monotonic_c_n_f_pass: bool,
}

pub fn activation_lineage() -> ActivationLineageReport {
    let zero_ok = schema2_activation_rate(D066_V_A, 1.0, 0.0, 1.0, 1.0, D066_K_C, D066_N_REF, D066_F_REF)
        .abs()
        < 1e-15
        && schema2_activation_rate(D066_V_A, 1.0, 1.0, 0.0, 1.0, D066_K_C, D066_N_REF, D066_F_REF).abs()
            < 1e-15
        && schema2_activation_rate(D066_V_A, 1.0, 1.0, 1.0, 0.0, D066_K_C, D066_N_REF, D066_F_REF).abs()
            < 1e-15;
    let bounded_high = {
        let r1 = schema2_activation_rate(D066_V_A, 1.0, 1.0, 1.0, 1.0, D066_K_C, D066_N_REF, D066_F_REF);
        let r10 =
            schema2_activation_rate(D066_V_A, 1.0, 10.0, 1.0, 1.0, D066_K_C, D066_N_REF, D066_F_REF);
        let r100 =
            schema2_activation_rate(D066_V_A, 1.0, 100.0, 1.0, 1.0, D066_K_C, D066_N_REF, D066_F_REF);
        r10 > r1 && r100 > r10 && r100 < D066_V_A * 1.01
    };
    let monotonic = {
        let base =
            schema2_activation_rate(D066_V_A, 1.0, 0.5, 0.5, 0.5, D066_K_C, D066_N_REF, D066_F_REF);
        schema2_activation_rate(D066_V_A, 1.0, 0.8, 0.5, 0.5, D066_K_C, D066_N_REF, D066_F_REF) >= base
            && schema2_activation_rate(D066_V_A, 1.0, 0.5, 0.8, 0.5, D066_K_C, D066_N_REF, D066_F_REF)
                >= base
            && schema2_activation_rate(D066_V_A, 1.0, 0.5, 0.5, 0.8, D066_K_C, D066_N_REF, D066_F_REF)
                >= base
    };
    ActivationLineageReport {
        equation_version: D066_EQUATION_VERSION.to_string(),
        rate_law: "r = V_A · H(phi) · q_C(C) · N_hat · F_hat".to_string(),
        v_a: D066_V_A,
        k_c: D066_K_C,
        n_ref: D066_N_REF,
        f_ref: D066_F_REF,
        spatial_support: "interior H(phi)=phi^2*(3-2*phi); catalyst q_C=C/(K_C+C)".to_string(),
        stoichiometry: "N+F -> A+W (dN=dF=-xi, dA=dW=+xi per unit extent)".to_string(),
        d065_conclusion_preserved: D066_D065_CONCLUSION.to_string(),
        zero_resource_controls_pass: zero_ok,
        bounded_high_c_pass: bounded_high,
        monotonic_c_n_f_pass: monotonic,
    }
}

/// Classify which factor limited an accepted-step activation extent.
/// Inputs: requested extent, accepted extent, local N,F,C, whether spatial support
/// (H(phi)>0) is present, whether the step was dt-capped, whether the step was
/// rejected before completion.
pub fn classify_limiter(
    xi_req: f64,
    xi_acc: f64,
    n: f64,
    f: f64,
    c: f64,
    support_ok: bool,
    timestep_capped: bool,
    rejected: bool,
) -> ActivationLimiterClass {
    if rejected {
        return ActivationLimiterClass::TimestepLimited;
    }
    if !support_ok {
        return ActivationLimiterClass::SupportLimited;
    }
    let close_enough = xi_req > 0.0 && (xi_acc - xi_req).abs() <= 1e-9 * (1.0 + xi_req.abs());
    if close_enough && !timestep_capped {
        // Accepted matches requested → whichever intensive factor is smallest is the
        // proximate rate-slowing input.
        let q = c / (D066_K_C + c).max(1e-18);
        let mut hits = Vec::new();
        if n < 0.5 * D066_N_REF {
            hits.push(ActivationLimiterClass::NLimited);
        }
        if f < 0.5 * D066_F_REF {
            hits.push(ActivationLimiterClass::FLimited);
        }
        if q < 0.5 {
            hits.push(ActivationLimiterClass::CLimitedRate);
        }
        return match hits.as_slice() {
            [] => ActivationLimiterClass::NoLimit,
            [only] => *only,
            _ => ActivationLimiterClass::MultipleLimits,
        };
    }
    if timestep_capped {
        return ActivationLimiterClass::TimestepLimited;
    }
    if xi_req > 0.0 && xi_acc < 0.99 * xi_req {
        // Fell short — attribute to substrate near zero if visible.
        let mut hits = Vec::new();
        if n <= D066_EPS {
            hits.push(ActivationLimiterClass::NLimited);
        }
        if f <= D066_EPS {
            hits.push(ActivationLimiterClass::FLimited);
        }
        if c <= D066_EPS {
            hits.push(ActivationLimiterClass::CLimitedRate);
        }
        return match hits.as_slice() {
            [] => ActivationLimiterClass::ProductCeilingLimited,
            [only] => *only,
            _ => ActivationLimiterClass::MultipleLimits,
        };
    }
    ActivationLimiterClass::NoLimit
}

/// Overlap integral O_{CNF} = Σ_i H(phi_i) · min(N̂_i,F̂_i) · q_C(C_i).
/// Restricted to indices supplied by caller (typically interior cells).
pub fn overlap_integral_o_cnf(
    phi: &[f64],
    n: &[f64],
    f: &[f64],
    c: &[f64],
    indices: &[usize],
) -> f64 {
    let mut s = 0.0;
    for &i in indices {
        let hphi = interior_weight(phi[i]);
        let nhat = (n[i] / D066_N_REF.max(1e-18)).max(0.0);
        let fhat = (f[i] / D066_F_REF.max(1e-18)).max(0.0);
        let q = c[i].max(0.0) / (D066_K_C + c[i].max(0.0)).max(1e-18);
        s += hphi * nhat.min(fhat) * q;
    }
    s
}

/// Fraction of interior cells with H(phi_i)*q_C*min(N̂_i,F̂_i) > threshold.
pub fn f_active(
    phi: &[f64],
    n: &[f64],
    f: &[f64],
    c: &[f64],
    indices: &[usize],
    threshold: f64,
) -> f64 {
    if indices.is_empty() {
        return 0.0;
    }
    let mut hits = 0usize;
    for &i in indices {
        let hphi = interior_weight(phi[i]);
        let nhat = (n[i] / D066_N_REF.max(1e-18)).max(0.0);
        let fhat = (f[i] / D066_F_REF.max(1e-18)).max(0.0);
        let q = c[i].max(0.0) / (D066_K_C + c[i].max(0.0)).max(1e-18);
        if hphi * q * nhat.min(fhat) > threshold {
            hits += 1;
        }
    }
    hits as f64 / indices.len() as f64
}

/// Utilization u_X = U_activation / max(J_net, eps). Clamped to [0, +inf].
pub fn utilization(u_activation: f64, j_net: f64) -> f64 {
    let denom = j_net.max(D066_EPS);
    (u_activation / denom).max(0.0)
}

fn interior_totals(field: &[f64], indices: &[usize]) -> f64 {
    indices.iter().map(|&i| field[i].max(0.0)).sum()
}

/// Uniform redistribution over `indices`. Total mass conserved on the subset.
/// Values outside `indices` are untouched.
pub fn redistribute_nf_uniform(n: &mut [f64], f: &mut [f64], indices: &[usize]) {
    if indices.is_empty() {
        return;
    }
    let n_tot: f64 = interior_totals(n, indices);
    let f_tot: f64 = interior_totals(f, indices);
    let inv = 1.0 / indices.len() as f64;
    let n_val = n_tot * inv;
    let f_val = f_tot * inv;
    for &i in indices {
        n[i] = n_val;
        f[i] = f_val;
    }
}

/// Catalyst-weighted redistribution: mass placed proportional to C_i·H(phi_i).
/// Falls back to uniform on the subset if weights sum to zero.
pub fn redistribute_nf_catalyst_weighted(
    n: &mut [f64],
    f: &mut [f64],
    c: &[f64],
    phi: &[f64],
    indices: &[usize],
) {
    if indices.is_empty() {
        return;
    }
    let n_tot: f64 = interior_totals(n, indices);
    let f_tot: f64 = interior_totals(f, indices);
    let weights: Vec<f64> = indices
        .iter()
        .map(|&i| c[i].max(0.0) * interior_weight(phi[i]))
        .collect();
    let wsum: f64 = weights.iter().sum();
    if wsum <= D066_EPS {
        redistribute_nf_uniform(n, f, indices);
        return;
    }
    for (k, &i) in indices.iter().enumerate() {
        let w = weights[k] / wsum;
        n[i] = n_tot * w;
        f[i] = f_tot * w;
    }
}

/// Boundary-weighted redistribution over `indices`, using caller-provided
/// per-index weights (e.g. distance-to-membrane). Falls back to uniform on the
/// subset when the weight sum is zero.
pub fn redistribute_nf_boundary_weighted(
    n: &mut [f64],
    f: &mut [f64],
    indices: &[usize],
    weights: &[f64],
) {
    assert_eq!(indices.len(), weights.len(), "weights length mismatch");
    if indices.is_empty() {
        return;
    }
    let n_tot: f64 = interior_totals(n, indices);
    let f_tot: f64 = interior_totals(f, indices);
    let wsum: f64 = weights.iter().map(|w| w.max(0.0)).sum();
    if wsum <= D066_EPS {
        redistribute_nf_uniform(n, f, indices);
        return;
    }
    for (k, &i) in indices.iter().enumerate() {
        let w = weights[k].max(0.0) / wsum;
        n[i] = n_tot * w;
        f[i] = f_tot * w;
    }
}

/// Schema-2 rate evaluated at a single point.
#[inline]
pub fn capacity_rate_at(c: f64, n: f64, f: f64, phi: f64) -> f64 {
    schema2_activation_rate(D066_V_A, phi, c, n, f, D066_K_C, D066_N_REF, D066_F_REF)
}

/// Finite-difference elasticity: (∂r/∂x) · (x/r) evaluated by central-ish
/// difference (fwd if x_base==0). Returns 0 when the base rate is ~0.
pub fn elasticity_along(
    base_c: f64,
    base_n: f64,
    base_f: f64,
    base_phi: f64,
    axis: char,
    h_frac: f64,
) -> f64 {
    let r0 = capacity_rate_at(base_c, base_n, base_f, base_phi);
    if r0.abs() <= D066_EPS {
        return 0.0;
    }
    let (x0, mut c_hi, mut n_hi, mut f_hi) = (
        match axis {
            'C' | 'c' => base_c,
            'N' | 'n' => base_n,
            'F' | 'f' => base_f,
            _ => base_c,
        },
        base_c,
        base_n,
        base_f,
    );
    let h = (x0.abs().max(1e-6)) * h_frac.abs().max(1e-6);
    match axis {
        'C' | 'c' => c_hi = base_c + h,
        'N' | 'n' => n_hi = base_n + h,
        'F' | 'f' => f_hi = base_f + h,
        _ => {}
    }
    let r1 = capacity_rate_at(c_hi, n_hi, f_hi, base_phi);
    let dr = r1 - r0;
    (dr / h) * (x0 / r0)
}

pub fn classify_capacity(
    r0: f64,
    e_c: f64,
    e_n: f64,
    e_f: f64,
    demand_density: f64,
) -> CapacityClass {
    if r0 <= D066_EPS {
        return CapacityClass::ActivationSupportCapacityLimit;
    }
    let saturating_c = e_c < 0.10 && e_n + e_f > e_c;
    let saturating_nf = e_n < 0.10 && e_f < 0.10 && e_c > 0.20;
    let rate_ok = r0 >= demand_density;
    if saturating_c && !rate_ok {
        return CapacityClass::ActivationCatalystSaturated;
    }
    if saturating_nf && !rate_ok {
        return CapacityClass::ActivationResourceSaturated;
    }
    if !rate_ok && e_c.abs() + e_n.abs() + e_f.abs() < 0.30 {
        return CapacityClass::ActivationRateConstantLimit;
    }
    if rate_ok {
        return CapacityClass::ActivationCapacityAdequate;
    }
    CapacityClass::ActivationCapacityInconclusive
}

pub fn classify_utilization(u: f64, delta_inv: f64, j_net: f64) -> ResourceUtilizationClass {
    if j_net <= D066_EPS {
        return ResourceUtilizationClass::ResourceFateUnresolved;
    }
    let scale = j_net.abs().max(1e-9);
    if u >= 0.5 {
        return ResourceUtilizationClass::HighDeliveryHighUtilization;
    }
    if delta_inv.abs() / scale >= 0.4 {
        return ResourceUtilizationClass::NonproductiveInternalAccumulation;
    }
    if u < 0.05 && delta_inv.abs() / scale < 0.4 {
        return ResourceUtilizationClass::RapidReexport;
    }
    ResourceUtilizationClass::HighDeliveryLowUtilization
}

pub fn classify_catalyst_support(
    c_total_healthy: f64,
    c_total_baseline: f64,
    spatial_c_helps: bool,
    total_c_helps: bool,
    baseline_a_ret: f64,
) -> CatalystSupportClass {
    if baseline_a_ret >= D066_A_RETENTION_TARGET {
        return CatalystSupportClass::CNotPrimaryActivationLimit;
    }
    if total_c_helps && c_total_healthy > c_total_baseline * 1.5 {
        return CatalystSupportClass::TotalCLimit;
    }
    if spatial_c_helps && !total_c_helps {
        return CatalystSupportClass::CSpatialSupportLimit;
    }
    if !spatial_c_helps && !total_c_helps {
        return CatalystSupportClass::CNotPrimaryActivationLimit;
    }
    CatalystSupportClass::CatalystActivationRoleInconclusive
}

/// A ledger for D-066: like D-065 ALedger but with explicit demand share weights
/// and its own `dominant_sink` string labels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ALedger066 {
    pub g_activation: f64,
    pub l_catalyst: f64,
    pub l_structure: f64,
    pub l_precursor: f64,
    pub l_decay: f64,
    pub j_out: f64,
    pub j_in: f64,
    pub delta_a: f64,
    pub activation_requested: f64,
    pub activation_accepted: f64,
    pub j_n_net: f64,
    pub j_f_net: f64,
}

impl ALedger066 {
    pub fn residual(self) -> f64 {
        self.g_activation
            - self.l_catalyst
            - self.l_structure
            - self.l_precursor
            - self.l_decay
            - self.j_out
            + self.j_in
            - self.delta_a
    }
    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs()
            <= tol.max(D066_LEDGER_TOL) * (1.0 + self.g_activation.abs().max(self.delta_a.abs()))
    }
    pub fn total_demand(self) -> f64 {
        self.l_catalyst + self.l_structure + self.l_precursor + self.l_decay + self.j_out
    }
    /// χ_A = accepted activation / total demand.
    pub fn chi_a(self) -> f64 {
        chi_ratio(self.activation_accepted, self.total_demand())
    }
    pub fn dominant_sink(self) -> &'static str {
        let terms = [
            ("precursor", self.l_precursor),
            ("structure", self.l_structure),
            ("catalyst", self.l_catalyst),
            ("decay", self.l_decay),
            ("transport_out", self.j_out),
        ];
        terms
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _)| *n)
            .unwrap_or("none")
    }
    /// Classify which A-demand branch dominates when χ_A < 1.
    pub fn classify_demand(self) -> ADemandClass {
        if !self.closes(1e-2) {
            return ADemandClass::ABalanceUnresolved;
        }
        let td = self.total_demand();
        let g = self.g_activation.max(D066_EPS);
        let mut hits = 0usize;
        let mut primary = ADemandClass::ABalanceUnresolved;
        if g < td {
            hits += 1;
            primary = ADemandClass::GrossActivationBelowTotalDemand;
        }
        if g >= td && self.j_out > 0.4 * g {
            hits += 1;
            primary = ADemandClass::GrossActivationSufficientNetTransportLimit;
        }
        // Dominant sink categorization only when demand exceeds production.
        if g < td {
            let terms = [
                (ADemandClass::PrecursorDemandDominant, self.l_precursor),
                (ADemandClass::StructuralDemandDominant, self.l_structure),
                (ADemandClass::CatalystDemandDominant, self.l_catalyst),
                (ADemandClass::ADecayDominant, self.l_decay),
            ];
            if let Some(top) = terms
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                if top.1 > 0.4 * td {
                    primary = top.0;
                    hits = hits.max(1);
                }
            }
        }
        if hits >= 2 {
            ADemandClass::MultipleADemands
        } else if hits == 1 {
            primary
        } else {
            ADemandClass::ABalanceUnresolved
        }
    }
}

/// Route evidence collected by the D-066 pipeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence066 {
    pub workspace_isolated: bool,
    pub d065_reproduced: bool,
    pub lineage_ok: bool,
    pub runtime_parity_ok: bool,
    pub fate_ledger_ok: bool,
    pub a_ledger_ok: bool,
    // Route X — acceptance execution
    pub acceptance_execution_defect: bool,
    // Route W — waste rejection masks the answer
    pub waste_masks_activation: bool,
    pub usable_windows_available: bool,
    // Route O — redistribution restores under ordinary N/F
    pub redistribution_restores_a: bool,
    pub ordinary_delivery_fails_a: bool,
    // Route C — catalyst support control restores under ordinary N/F
    pub healthy_c_restores_a_under_ordinary_nf: bool,
    // Route K — local N/F+C sufficient but demand still uncovered
    pub local_nf_and_c_sufficient_still_insufficient: bool,
    // Route D — activation sufficient but demand causes net loss
    pub activation_sufficient_demand_net_loss: bool,
    // Route M — multiple independent limits
    pub multiple_limits_flagged: bool,
    // Numeric context (informative)
    pub a_retention: f64,
    pub chi_smooth_min: f64,
    pub chi_a: f64,
}

/// Directive route priority — see D-066 spec. Failures first; then one of X/W/O/C/K/D/M/I.
pub fn select_route(ev: RouteEvidence066) -> (D066Route, D066PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D066Route::I,
            D066PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d065_reproduced {
        return (
            D066Route::I,
            D066PrimaryConclusion::D065ActivationRouteNotReproduced,
        );
    }
    if !ev.lineage_ok {
        return (
            D066Route::I,
            D066PrimaryConclusion::ActivationLineageUnresolved,
        );
    }
    if !ev.runtime_parity_ok {
        return (
            D066Route::I,
            D066PrimaryConclusion::ActivationRuntimeParityFailure,
        );
    }
    if !ev.fate_ledger_ok {
        return (
            D066Route::I,
            D066PrimaryConclusion::InternalResourceFateAccountingFailure,
        );
    }
    if !ev.a_ledger_ok {
        return (D066Route::I, D066PrimaryConclusion::ALedgerFailure);
    }

    if ev.acceptance_execution_defect {
        return (D066Route::X, D066Route::X.conclusion());
    }
    if ev.waste_masks_activation && !ev.usable_windows_available {
        return (D066Route::W, D066Route::W.conclusion());
    }
    if ev.redistribution_restores_a && ev.ordinary_delivery_fails_a {
        return (D066Route::O, D066Route::O.conclusion());
    }
    if ev.healthy_c_restores_a_under_ordinary_nf {
        return (D066Route::C, D066Route::C.conclusion());
    }
    if ev.local_nf_and_c_sufficient_still_insufficient {
        return (D066Route::K, D066Route::K.conclusion());
    }
    if ev.activation_sufficient_demand_net_loss {
        return (D066Route::D, D066Route::D.conclusion());
    }
    if ev.multiple_limits_flagged {
        return (D066Route::M, D066Route::M.conclusion());
    }
    (D066Route::I, D066Route::I.conclusion())
}

pub fn shadow_isolation_ok(production_carrier: bool, v15: bool, morphogenesis: bool) -> bool {
    !production_carrier && !v15 && !morphogenesis
}

/// Predicate summarising D-065 Gate-0-style reproduction expectations.
pub fn d065_reproduction_predicate(
    smooth_chi_r16: f64,
    smooth_chi_r22: f64,
    smooth_chi_r32: f64,
    a_ret_baseline: f64,
    a_ret_control_c: f64,
    a_ret_perfect_exterior: f64,
) -> bool {
    smooth_chi_r16 >= 1.05
        && smooth_chi_r22 >= 1.05
        && smooth_chi_r32 >= 1.05
        && a_ret_baseline < D066_A_RETENTION_TARGET
        && a_ret_perfect_exterior < D066_A_RETENTION_TARGET
        && a_ret_control_c >= D066_A_RETENTION_TARGET
}

/// Accepted-step defect: xi_acc materially below xi_req despite step acceptance.
pub fn acceptance_execution_defect(xi_req: f64, xi_acc: f64, step_accepted: bool) -> bool {
    step_accepted && xi_req > D066_EPS && xi_acc + 1e-12 < xi_req * 0.999
}

/// Alias for tests/pipeline.
#[inline]
pub fn activation_stoichiometry_parity(extent: f64) -> bool {
    activation_isolated_stoichiometric(extent)
}

/// Predicate: activation-isolated per-unit-extent delta matches expected
/// stoichiometry ΔN=ΔF=−ξ, ΔA=ΔW=+ξ.
pub fn activation_isolated_stoichiometric(extent: f64) -> bool {
    let d = crate::activated_metabolism::activation_isolated_delta(extent);
    (d[2] + extent).abs() < 1e-15
        && (d[3] + extent).abs() < 1e-15
        && (d[5] - extent).abs() < 1e-15
        && (d[4] - extent).abs() < 1e-15
        && (d[0].abs() + d[1].abs() + d[6].abs()) < 1e-15
}

/// Exclude rejected steps from a window: caller passes `step_accepted` and any
/// per-step contribution; returns 0.0 when rejected.
#[inline]
pub fn accepted_contribution(step_accepted: bool, value: f64) -> f64 {
    if step_accepted {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn limiter_no_limit_when_accepted_matches_and_no_starvation() {
        let cls = classify_limiter(1.0, 1.0, 1.0, 1.0, 1.0, true, false, false);
        assert_eq!(cls, ActivationLimiterClass::NoLimit);
    }

    #[test]
    fn limiter_n_limited_when_low_n() {
        let cls = classify_limiter(1.0, 1.0, 0.1, 1.0, 1.0, true, false, false);
        assert_eq!(cls, ActivationLimiterClass::NLimited);
    }

    #[test]
    fn limiter_multiple_limits_when_both_starved() {
        let cls = classify_limiter(1.0, 1.0, 0.1, 0.1, 1.0, true, false, false);
        assert_eq!(cls, ActivationLimiterClass::MultipleLimits);
    }

    #[test]
    fn limiter_timestep_when_rejected() {
        let cls = classify_limiter(1.0, 0.0, 1.0, 1.0, 1.0, true, false, true);
        assert_eq!(cls, ActivationLimiterClass::TimestepLimited);
    }

    #[test]
    fn redistribute_mass_conservation_uniform() {
        let mut n = vec![0.0, 0.0, 1.0, 3.0, 0.0];
        let mut f = vec![0.0, 0.0, 2.0, 2.0, 0.0];
        let indices = vec![2, 3];
        let n_before: f64 = indices.iter().map(|&i| n[i]).sum();
        let f_before: f64 = indices.iter().map(|&i| f[i]).sum();
        redistribute_nf_uniform(&mut n, &mut f, &indices);
        let n_after: f64 = indices.iter().map(|&i| n[i]).sum();
        let f_after: f64 = indices.iter().map(|&i| f[i]).sum();
        assert!((n_before - n_after).abs() < 1e-12);
        assert!((f_before - f_after).abs() < 1e-12);
        // Untouched cells stay at zero.
        assert!(n[0] == 0.0 && n[1] == 0.0 && n[4] == 0.0);
    }

    #[test]
    fn route_priority_x_over_o() {
        let mut ev = RouteEvidence066 {
            workspace_isolated: true,
            d065_reproduced: true,
            lineage_ok: true,
            runtime_parity_ok: true,
            fate_ledger_ok: true,
            a_ledger_ok: true,
            acceptance_execution_defect: true,
            waste_masks_activation: false,
            usable_windows_available: true,
            redistribution_restores_a: true,
            ordinary_delivery_fails_a: true,
            healthy_c_restores_a_under_ordinary_nf: true,
            local_nf_and_c_sufficient_still_insufficient: false,
            activation_sufficient_demand_net_loss: false,
            multiple_limits_flagged: false,
            a_retention: 0.4,
            chi_smooth_min: 19.0,
            chi_a: 0.6,
        };
        assert_eq!(select_route(ev).0, D066Route::X);
        ev.acceptance_execution_defect = false;
        // With X off, next hit is O (redistribution restores).
        assert_eq!(select_route(ev).0, D066Route::O);
    }
}
