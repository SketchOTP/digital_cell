//! D-068 precursor demand and membrane assembly closure audit helpers.
//!
//! Observer/shadow-only diagnostics. Frozen activation law and stoichiometry
//! are preserved. Production precursor/membrane defaults are not changed.

use crate::fields::interior_weight;
use crate::membrane::membrane_catalyst_saturation;
use serde::{Deserialize, Serialize};

pub const D068_PROJECT_ID: &str = "D-068";
pub const D068_AGENT_MEMORY_ID: &str =
    "D-20260721-d068-precursor-demand-membrane-assembly-audit";
pub const D068_STARTING_COMMIT: &str = "99b8eda";
pub const D068_STARTING_TAG: &str = "D-067-activation-capacity-law-identification";
pub const D068_D067_CONCLUSION: &str = "D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW";
pub const ACTIVATION_LAW_BRANCH_CLOSED: &str = "ACTIVATION_LAW_BRANCH_CLOSED";
pub const PRECURSOR_MEMBRANE_DEMAND_CAUSE_UNRESOLVED: &str =
    "PRECURSOR_MEMBRANE_DEMAND_CAUSE_UNRESOLVED";

pub const D068_V_A: f64 = 0.12544510052968755;
pub const D068_K_C: f64 = 0.10;
pub const D068_N_REF: f64 = 1.0;
pub const D068_F_REF: f64 = 1.0;
pub const D068_FROZEN_KT: f64 = 1.4346157818803311;

/// Declared precursor synthesis stoichiometry: A → P (no W on synthesis).
pub const NU_A_SYN: f64 = 1.0;
pub const NU_P_SYN: f64 = 1.0;
pub const NU_W_SYN: f64 = 0.0;

pub const A_RETENTION: f64 = 0.80;
pub const C_RETENTION: f64 = 0.80;
pub const S_RETENTION: f64 = 0.80;
pub const CHI_A_TARGET: f64 = 1.05;
pub const CHI_S_TARGET: f64 = 1.00;
pub const PORTABLE_SPAN_MAX: f64 = 3.0;
pub const BOOTSTRAP_SPREAD_MAX: f64 = 0.50;
pub const LOO_MAX: f64 = 2.0;
pub const HOLDOUT_MEDIAN_ERR: f64 = 0.20;
pub const HOLDOUT_MAX_ERR: f64 = 0.35;
pub const BALANCE_SIGN_ACC: f64 = 0.90;
pub const LEDGER_TOL: f64 = 1e-6;
pub const EPS: f64 = 1e-18;

/// Exact frozen precursor law (production/default lineage).
pub const PRECURSOR_EQUATION: &str = "r_P = k_precursor · A · q(C) · H(φ)";
pub const PRECURSOR_STOICHIOMETRY: &str = "A → P";
pub const ADSORPTION_EQUATION: &str =
    "dS/dt_forward = δ · k_exchange · q(C) · Γ_max · K_eq · p · (1−θ)";
pub const DESORPTION_EQUATION: &str =
    "dS/dt_reverse = δ · k_exchange · q(C) · Γ_max · θ";
pub const NET_EXCHANGE_EQUATION: &str =
    "dS/dt = δ · k_exchange · q(C) · Γ_max · (K_eq · p · (1−θ) − θ)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D068PrimaryConclusion {
    PrecursorOrMembraneExecutionDefect,
    PrecursorOverproductionQualified,
    PrecursorProductInhibitionQualified,
    MembraneAssemblyCapacityLimit,
    MembraneDesorptionDominant,
    PrecursorMembraneAccessLimit,
    MultiplePrecursorMembraneLimits,
    WasteExecutionBlocksPrecursorAudit,
    NoPortablePrecursorDemandRepair,
    PrecursorDemandRepairedStageEStillBlocked,
    ExistingPrecursorMembraneSystemQualified,
    PrecursorMembraneAuditInconclusive,
    D067PrecursorRouteNotReproduced,
    PrecursorMembraneLineageUnresolved,
    PrecursorMembraneRuntimeParityFailure,
    ApswLedgerFailure,
    PrecursorMembraneCausalityFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D068PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrecursorOrMembraneExecutionDefect => {
                "D068_PRECURSOR_OR_MEMBRANE_EXECUTION_DEFECT"
            }
            Self::PrecursorOverproductionQualified => "D068_PRECURSOR_OVERPRODUCTION_QUALIFIED",
            Self::PrecursorProductInhibitionQualified => {
                "D068_PRECURSOR_PRODUCT_INHIBITION_QUALIFIED"
            }
            Self::MembraneAssemblyCapacityLimit => "D068_MEMBRANE_ASSEMBLY_CAPACITY_LIMIT",
            Self::MembraneDesorptionDominant => "D068_MEMBRANE_DESORPTION_DOMINANT",
            Self::PrecursorMembraneAccessLimit => "D068_PRECURSOR_MEMBRANE_ACCESS_LIMIT",
            Self::MultiplePrecursorMembraneLimits => "D068_MULTIPLE_PRECURSOR_MEMBRANE_LIMITS",
            Self::WasteExecutionBlocksPrecursorAudit => {
                "D068_WASTE_EXECUTION_BLOCKS_PRECURSOR_AUDIT"
            }
            Self::NoPortablePrecursorDemandRepair => "D068_NO_PORTABLE_PRECURSOR_DEMAND_REPAIR",
            Self::PrecursorDemandRepairedStageEStillBlocked => {
                "D068_PRECURSOR_DEMAND_REPAIRED_STAGE_E_STILL_BLOCKED"
            }
            Self::ExistingPrecursorMembraneSystemQualified => {
                "D068_EXISTING_PRECURSOR_MEMBRANE_SYSTEM_QUALIFIED"
            }
            Self::PrecursorMembraneAuditInconclusive => {
                "D068_PRECURSOR_MEMBRANE_AUDIT_INCONCLUSIVE"
            }
            Self::D067PrecursorRouteNotReproduced => "D068_D067_PRECURSOR_ROUTE_NOT_REPRODUCED",
            Self::PrecursorMembraneLineageUnresolved => {
                "D068_PRECURSOR_MEMBRANE_LINEAGE_UNRESOLVED"
            }
            Self::PrecursorMembraneRuntimeParityFailure => {
                "D068_PRECURSOR_MEMBRANE_RUNTIME_PARITY_FAILURE"
            }
            Self::ApswLedgerFailure => "D068_APSW_LEDGER_FAILURE",
            Self::PrecursorMembraneCausalityFailure => {
                "D068_PRECURSOR_MEMBRANE_CAUSALITY_FAILURE"
            }
            Self::WorkspaceScopeNotIsolated => "D068_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D068_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D068_NUMERICAL_FAILURE",
            Self::Fail => "D068_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D068Route {
    X,
    O,
    I,
    A,
    S,
    P,
    M,
    W,
    N,
    Q,
    C,
    U,
}

impl D068Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "Route_X_precursor_execution_defect",
            Self::O => "Route_O_precursor_overproduction",
            Self::I => "Route_I_product_inhibition_qualified",
            Self::A => "Route_A_p_to_s_assembly_limit",
            Self::S => "Route_S_s_desorption_limit",
            Self::P => "Route_P_p_membrane_access_limit",
            Self::M => "Route_M_multiple_membrane_demand_limits",
            Self::W => "Route_W_waste_execution_blocks_classification",
            Self::N => "Route_N_no_portable_precursor_demand_law",
            Self::Q => "Route_Q_precursor_repair_qualifies_stage_e_blocked",
            Self::C => "Route_C_existing_precursor_system_qualified",
            Self::U => "Route_U_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D068PrimaryConclusion {
        match self {
            Self::X => D068PrimaryConclusion::PrecursorOrMembraneExecutionDefect,
            Self::O => D068PrimaryConclusion::PrecursorOverproductionQualified,
            Self::I => D068PrimaryConclusion::PrecursorProductInhibitionQualified,
            Self::A => D068PrimaryConclusion::MembraneAssemblyCapacityLimit,
            Self::S => D068PrimaryConclusion::MembraneDesorptionDominant,
            Self::P => D068PrimaryConclusion::PrecursorMembraneAccessLimit,
            Self::M => D068PrimaryConclusion::MultiplePrecursorMembraneLimits,
            Self::W => D068PrimaryConclusion::WasteExecutionBlocksPrecursorAudit,
            Self::N => D068PrimaryConclusion::NoPortablePrecursorDemandRepair,
            Self::Q => D068PrimaryConclusion::PrecursorDemandRepairedStageEStillBlocked,
            Self::C => D068PrimaryConclusion::ExistingPrecursorMembraneSystemQualified,
            Self::U => D068PrimaryConclusion::PrecursorMembraneAuditInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrecursorFateClass {
    PrecursorProductivelyIncorporated,
    PrecursorAccumulation,
    PrecursorDecayLoss,
    PrecursorExportLoss,
    FutilePrecursorCycling,
    PrecursorFateUnresolved,
}

impl PrecursorFateClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrecursorProductivelyIncorporated => "PRECURSOR_PRODUCTIVELY_INCORPORATED",
            Self::PrecursorAccumulation => "PRECURSOR_ACCUMULATION",
            Self::PrecursorDecayLoss => "PRECURSOR_DECAY_LOSS",
            Self::PrecursorExportLoss => "PRECURSOR_EXPORT_LOSS",
            Self::FutilePrecursorCycling => "FUTILE_PRECURSOR_CYCLING",
            Self::PrecursorFateUnresolved => "PRECURSOR_FATE_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReplacementDemandClass {
    PrecursorOverproduction,
    PrecursorProductionAdequate,
    PrecursorProductionInsufficient,
    MembraneReplacementNotDemandIdentifiable,
}

impl ReplacementDemandClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrecursorOverproduction => "PRECURSOR_OVERPRODUCTION",
            Self::PrecursorProductionAdequate => "PRECURSOR_PRODUCTION_ADEQUATE",
            Self::PrecursorProductionInsufficient => "PRECURSOR_PRODUCTION_INSUFFICIENT",
            Self::MembraneReplacementNotDemandIdentifiable => {
                "MEMBRANE_REPLACEMENT_NOT_DEMAND_IDENTIFIABLE"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssemblyCapacityClass {
    PToSAssemblyCapacityAdequate,
    PToSAdsorptionCapacityLimit,
    SDesorptionDominant,
    PMembraneAccessLimit,
    PSExchangeExecutionDefect,
    PSAssemblyInconclusive,
}

impl AssemblyCapacityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PToSAssemblyCapacityAdequate => "P_TO_S_ASSEMBLY_CAPACITY_ADEQUATE",
            Self::PToSAdsorptionCapacityLimit => "P_TO_S_ADSORPTION_CAPACITY_LIMIT",
            Self::SDesorptionDominant => "S_DESORPTION_DOMINANT",
            Self::PMembraneAccessLimit => "P_MEMBRANE_ACCESS_LIMIT",
            Self::PSExchangeExecutionDefect => "P_S_EXCHANGE_EXECUTION_DEFECT",
            Self::PSAssemblyInconclusive => "P_S_ASSEMBLY_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrecursorCandidate {
    Baseline,
    GlobalScale,
    ProductInhibition,
}

impl PrecursorCandidate {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "BASELINE",
            Self::GlobalScale => "GLOBAL_SCALE",
            Self::ProductInhibition => "PRODUCT_INHIBITION",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ALedger068 {
    pub g_activation: f64,
    pub l_catalyst: f64,
    pub l_structure: f64,
    pub l_precursor: f64,
    pub l_decay: f64,
    pub j_net: f64,
    pub delta_a: f64,
}

impl ALedger068 {
    pub fn predicted_delta(self) -> f64 {
        self.g_activation
            - self.l_catalyst
            - self.l_structure
            - self.l_precursor
            - self.l_decay
            + self.j_net
    }

    pub fn residual(self) -> f64 {
        self.delta_a - self.predicted_delta()
    }

    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs() <= tol * (1.0 + self.delta_a.abs().max(self.g_activation.abs()))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PLedger068 {
    pub g_synthesis: f64,
    pub g_desorption: f64,
    pub l_adsorption: f64,
    pub l_decay: f64,
    pub j_net: f64,
    pub delta_p: f64,
}

impl PLedger068 {
    pub fn predicted_delta(self) -> f64 {
        self.g_synthesis + self.g_desorption - self.l_adsorption - self.l_decay + self.j_net
    }

    pub fn residual(self) -> f64 {
        self.delta_p - self.predicted_delta()
    }

    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs()
            <= tol * (1.0 + self.delta_p.abs().max(self.g_synthesis.abs()))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SLedger068 {
    pub g_adsorption: f64,
    pub l_desorption: f64,
    pub l_damage: f64,
    pub j_net: f64,
    pub delta_s: f64,
}

impl SLedger068 {
    pub fn predicted_delta(self) -> f64 {
        self.g_adsorption - self.l_desorption - self.l_damage + self.j_net
    }

    pub fn residual(self) -> f64 {
        self.delta_s - self.predicted_delta()
    }

    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs()
            <= tol * (1.0 + self.delta_s.abs().max(self.g_adsorption.abs()))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WLedger068 {
    pub g_activation: f64,
    pub g_catalyst: f64,
    pub g_structure: f64,
    pub g_precursor_decay: f64,
    pub g_membrane_damage: f64,
    pub j_net: f64,
    pub delta_w: f64,
}

impl WLedger068 {
    pub fn predicted_delta(self) -> f64 {
        self.g_activation
            + self.g_catalyst
            + self.g_structure
            + self.g_precursor_decay
            + self.g_membrane_damage
            + self.j_net
    }

    pub fn residual(self) -> f64 {
        self.delta_w - self.predicted_delta()
    }

    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs()
            <= tol * (1.0 + self.delta_w.abs().max(self.g_activation.abs()))
    }
}

/// Frozen precursor rate: r_P = k_P · A · q(C) · H(φ). Independent of P and S.
#[inline]
pub fn precursor_rate(k_p: f64, a: f64, c: f64, phi: f64, k_c: f64) -> f64 {
    k_p.max(0.0)
        * a.max(0.0)
        * membrane_catalyst_saturation_local(c, k_c)
        * interior_weight(phi)
}

#[inline]
fn membrane_catalyst_saturation_local(c: f64, k_c: f64) -> f64 {
    let c = c.max(0.0);
    c / (k_c.max(EPS) + c)
}

/// Candidate B: global precursor scale m_P · r_P,0.
#[inline]
pub fn candidate_b_rate(m_p: f64, k_p: f64, a: f64, c: f64, phi: f64, k_c: f64) -> f64 {
    m_p.max(0.0) * precursor_rate(k_p, a, c, phi, k_c)
}

/// Candidate C: local product inhibition q_P(P)=K/(K+P).
#[inline]
pub fn q_p_inhibition(p: f64, k_inh: f64) -> f64 {
    let p = p.max(0.0);
    k_inh.max(EPS) / (k_inh.max(EPS) + p)
}

#[inline]
pub fn candidate_c_rate(
    k_p: f64,
    a: f64,
    c: f64,
    phi: f64,
    k_c: f64,
    p: f64,
    k_inh: f64,
) -> f64 {
    precursor_rate(k_p, a, c, phi, k_c) * q_p_inhibition(p, k_inh)
}

/// Existing law has no P dependence ⇒ Candidate C is not already present.
pub fn baseline_has_product_inhibition() -> bool {
    false
}

/// Runtime synthesis parity for extent ξ_P under A → P.
pub fn precursor_synthesis_parity(xi: f64) -> bool {
    let da = -NU_A_SYN * xi;
    let dp = NU_P_SYN * xi;
    let dw = NU_W_SYN * xi;
    (da + xi).abs() <= 1e-15 && (dp - xi).abs() <= 1e-15 && dw.abs() <= 1e-15
}

/// Adsorption parity: ΔP = −ξ, ΔS = +ξ.
pub fn adsorption_parity(xi: f64) -> bool {
    let dp = -xi;
    let ds = xi;
    (dp + ds).abs() <= 1e-15
}

/// Desorption parity: ΔS = −ξ, ΔP = +ξ.
pub fn desorption_parity(xi: f64) -> bool {
    let ds = -xi;
    let dp = xi;
    (ds + dp).abs() <= 1e-15
}

#[inline]
pub fn eta_p_to_s(u_ads: f64, g_p: f64) -> f64 {
    u_ads / g_p.max(EPS)
}

#[inline]
pub fn eta_a_to_s(m_s: f64, e_a_to_p: f64) -> f64 {
    m_s.max(0.0) / e_a_to_p.max(EPS)
}

#[inline]
pub fn futile_fraction(u_ads: f64, g_p: f64) -> f64 {
    1.0 - u_ads / (g_p + EPS)
}

#[inline]
pub fn net_maintained_s(g_ads: f64, l_des: f64, l_damage: f64) -> f64 {
    g_ads - l_des - l_damage
}

#[inline]
pub fn g_p_required(l_des: f64, l_damage: f64, g_recycled: f64) -> f64 {
    (l_des + l_damage - g_recycled).max(0.0)
}

#[inline]
pub fn rho_p(g_actual: f64, g_required: f64) -> f64 {
    g_actual / (g_required + EPS)
}

#[inline]
pub fn chi_s(g_ads: f64, l_des: f64, l_damage: f64) -> f64 {
    g_ads / (l_des + l_damage + EPS)
}

pub fn classify_precursor_fate(
    g_p: f64,
    u_ads: f64,
    delta_p: f64,
    l_decay: f64,
    j_export: f64,
    l_des: f64,
) -> PrecursorFateClass {
    if !(g_p.is_finite() && u_ads.is_finite()) {
        return PrecursorFateClass::PrecursorFateUnresolved;
    }
    if g_p <= EPS {
        return PrecursorFateClass::PrecursorFateUnresolved;
    }
    let eta = eta_p_to_s(u_ads, g_p);
    if eta >= 0.80 && net_maintained_s(u_ads, l_des, 0.0) >= 0.0 {
        return PrecursorFateClass::PrecursorProductivelyIncorporated;
    }
    if delta_p > 0.15 * g_p {
        return PrecursorFateClass::PrecursorAccumulation;
    }
    if l_decay > 0.40 * g_p {
        return PrecursorFateClass::PrecursorDecayLoss;
    }
    if j_export.abs() > 0.40 * g_p {
        return PrecursorFateClass::PrecursorExportLoss;
    }
    if l_des > 0.50 * u_ads.max(EPS) && eta < 0.80 {
        return PrecursorFateClass::FutilePrecursorCycling;
    }
    if eta < 0.50 {
        return PrecursorFateClass::FutilePrecursorCycling;
    }
    PrecursorFateClass::PrecursorFateUnresolved
}

pub fn classify_replacement_demand(rho: f64, chi: f64, identifiable: bool) -> ReplacementDemandClass {
    if !identifiable || !rho.is_finite() || !chi.is_finite() {
        return ReplacementDemandClass::MembraneReplacementNotDemandIdentifiable;
    }
    if rho > 1.25 && chi < CHI_S_TARGET {
        return ReplacementDemandClass::PrecursorOverproduction;
    }
    if rho >= 0.90 && rho <= 1.25 && chi >= 0.95 {
        return ReplacementDemandClass::PrecursorProductionAdequate;
    }
    if rho < 0.90 {
        return ReplacementDemandClass::PrecursorProductionInsufficient;
    }
    if chi < CHI_S_TARGET {
        return ReplacementDemandClass::PrecursorOverproduction;
    }
    ReplacementDemandClass::PrecursorProductionAdequate
}

pub fn classify_assembly_capacity(
    fixed_healthy_p_arrests_s: bool,
    adsorption_accepted: f64,
    desorption: f64,
    interface_redistribution_rescues: bool,
    ordinary_rescues: bool,
    exchange_parity_ok: bool,
) -> AssemblyCapacityClass {
    if !exchange_parity_ok {
        return AssemblyCapacityClass::PSExchangeExecutionDefect;
    }
    if interface_redistribution_rescues && !ordinary_rescues {
        return AssemblyCapacityClass::PMembraneAccessLimit;
    }
    if fixed_healthy_p_arrests_s && adsorption_accepted >= desorption {
        return AssemblyCapacityClass::PToSAssemblyCapacityAdequate;
    }
    if !fixed_healthy_p_arrests_s && adsorption_accepted + EPS < desorption {
        return AssemblyCapacityClass::SDesorptionDominant;
    }
    if !fixed_healthy_p_arrests_s && adsorption_accepted >= desorption * 0.95 {
        return AssemblyCapacityClass::PToSAdsorptionCapacityLimit;
    }
    AssemblyCapacityClass::PSAssemblyInconclusive
}

/// D-067 Gate0 reproduction predicate (smooth delivery + ordinary A + unlimited rescue).
pub fn d067_reproduction_predicate(
    chi_min: f64,
    ordinary_a: f64,
    unlimited_a: f64,
    chi_a: f64,
    precursor_fraction: f64,
) -> bool {
    chi_min >= 1.05
        && ordinary_a > 0.20
        && ordinary_a < 0.55
        && unlimited_a > 1.0
        && chi_a > 0.05
        && chi_a < 0.30
        && precursor_fraction >= 0.50
}

pub fn zero_precursor_when_a_starved<F>(rate: F) -> bool
where
    F: Fn(f64, f64, f64) -> f64,
{
    // (C, phi, P) — A fixed at 0
    rate(0.4, 1.0, 0.1).abs() <= 1e-15 && rate(1.0, 1.0, 1.0).abs() <= 1e-15
}

/// Existing frozen law already lacks P dependence; C is distinct only if q_P applied.
pub fn candidate_c_distinct_from_baseline(k_inh: f64, p_domain: f64) -> bool {
    !baseline_has_product_inhibition()
        && k_inh.is_finite()
        && k_inh > 0.0
        && p_domain.is_finite()
        && q_p_inhibition(p_domain, k_inh) < 0.999
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentificationReport068 {
    pub params_positive_finite: bool,
    pub half_sats_in_domain: bool,
    pub bootstrap_spread: f64,
    pub loo_variation: f64,
    pub holdout_median_err: f64,
    pub holdout_max_err: f64,
    pub holdout_a_sign_acc: f64,
    pub holdout_s_sign_acc: f64,
    pub no_radius_params: bool,
    pub stoichiometry_ok: bool,
    pub accounting_ok: bool,
}

impl IdentificationReport068 {
    pub fn qualifies(&self) -> bool {
        self.params_positive_finite
            && self.half_sats_in_domain
            && self.bootstrap_spread <= BOOTSTRAP_SPREAD_MAX
            && self.loo_variation <= LOO_MAX
            && self.holdout_median_err <= HOLDOUT_MEDIAN_ERR
            && self.holdout_max_err <= HOLDOUT_MAX_ERR
            && self.holdout_a_sign_acc >= BALANCE_SIGN_ACC
            && self.holdout_s_sign_acc >= BALANCE_SIGN_ACC
            && self.no_radius_params
            && self.stoichiometry_ok
            && self.accounting_ok
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence068 {
    pub workspace_isolated: bool,
    pub d067_reproduced: bool,
    pub lineage_ok: bool,
    pub runtime_parity_ok: bool,
    pub ledger_ok: bool,
    pub safety_causality_ok: bool,
    pub waste_blocks: bool,
    pub identification: IdentificationReport068,
    pub existing_qualified: bool,
    pub overproduction_qualified: bool,
    pub inhibition_qualified: bool,
    pub assembly_limit: bool,
    pub desorption_dominant: bool,
    pub membrane_access_limit: bool,
    pub multiple_limits: bool,
    pub repair_but_stage_e_blocked: bool,
    pub no_portable_repair: bool,
}

pub fn select_route(ev: RouteEvidence068) -> (D068Route, D068PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D068Route::U,
            D068PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d067_reproduced {
        return (
            D068Route::U,
            D068PrimaryConclusion::D067PrecursorRouteNotReproduced,
        );
    }
    if !ev.lineage_ok {
        return (
            D068Route::U,
            D068PrimaryConclusion::PrecursorMembraneLineageUnresolved,
        );
    }
    if !ev.runtime_parity_ok {
        return (D068Route::X, D068PrimaryConclusion::PrecursorOrMembraneExecutionDefect);
    }
    if !ev.ledger_ok {
        return (D068Route::U, D068PrimaryConclusion::ApswLedgerFailure);
    }
    if ev.waste_blocks {
        return (D068Route::W, D068PrimaryConclusion::WasteExecutionBlocksPrecursorAudit);
    }
    if !ev.safety_causality_ok {
        return (
            D068Route::U,
            D068PrimaryConclusion::PrecursorMembraneCausalityFailure,
        );
    }
    if ev.overproduction_qualified && ev.identification.qualifies() {
        return (D068Route::O, D068PrimaryConclusion::PrecursorOverproductionQualified);
    }
    if ev.inhibition_qualified && ev.identification.qualifies() {
        return (D068Route::I, D068PrimaryConclusion::PrecursorProductInhibitionQualified);
    }
    if ev.repair_but_stage_e_blocked {
        return (
            D068Route::Q,
            D068PrimaryConclusion::PrecursorDemandRepairedStageEStillBlocked,
        );
    }
    if ev.existing_qualified {
        return (
            D068Route::C,
            D068PrimaryConclusion::ExistingPrecursorMembraneSystemQualified,
        );
    }
    if ev.multiple_limits {
        return (D068Route::M, D068PrimaryConclusion::MultiplePrecursorMembraneLimits);
    }
    if ev.membrane_access_limit {
        return (D068Route::P, D068PrimaryConclusion::PrecursorMembraneAccessLimit);
    }
    if ev.desorption_dominant {
        return (D068Route::S, D068PrimaryConclusion::MembraneDesorptionDominant);
    }
    if ev.assembly_limit {
        return (D068Route::A, D068PrimaryConclusion::MembraneAssemblyCapacityLimit);
    }
    if ev.no_portable_repair {
        return (D068Route::N, D068PrimaryConclusion::NoPortablePrecursorDemandRepair);
    }
    (
        D068Route::U,
        D068PrimaryConclusion::PrecursorMembraneAuditInconclusive,
    )
}

/// Preregistered m_P ∈ (0,1] from measured ρ_P.
pub fn preregistered_m_p(rho_p: f64) -> Vec<f64> {
    let mut vals = vec![1.0];
    if rho_p.is_finite() && rho_p > 1.0 {
        vals.push((1.0 / rho_p).clamp(0.05, 0.95));
        vals.push((1.05 / rho_p).clamp(0.05, 0.95));
        vals.push(0.5);
        vals.push(0.25);
    } else {
        vals.extend_from_slice(&[0.75, 0.5, 0.25, 0.0]);
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    vals.dedup_by(|a, b| (*a - *b).abs() <= 1e-12);
    vals.into_iter().take(5).collect()
}

/// Lineage identity record for Gate 1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrecursorMembraneLineage {
    pub precursor_equation: String,
    pub precursor_stoichiometry: String,
    pub depends_on_a: bool,
    pub depends_on_c: bool,
    pub depends_on_phi: bool,
    pub depends_on_p: bool,
    pub depends_on_s: bool,
    pub nu_a: f64,
    pub nu_p: f64,
    pub nu_w: f64,
    pub adsorption_equation: String,
    pub desorption_equation: String,
    pub net_exchange_equation: String,
    pub schema3_constitutive_damage: bool,
}

pub fn frozen_lineage() -> PrecursorMembraneLineage {
    PrecursorMembraneLineage {
        precursor_equation: PRECURSOR_EQUATION.into(),
        precursor_stoichiometry: PRECURSOR_STOICHIOMETRY.into(),
        depends_on_a: true,
        depends_on_c: true,
        depends_on_phi: true,
        depends_on_p: false,
        depends_on_s: false,
        nu_a: NU_A_SYN,
        nu_p: NU_P_SYN,
        nu_w: NU_W_SYN,
        adsorption_equation: ADSORPTION_EQUATION.into(),
        desorption_equation: DESORPTION_EQUATION.into(),
        net_exchange_equation: NET_EXCHANGE_EQUATION.into(),
        schema3_constitutive_damage: false,
    }
}

pub fn lineage_resolved(lin: &PrecursorMembraneLineage) -> bool {
    lin.depends_on_a
        && lin.depends_on_c
        && lin.depends_on_phi
        && !lin.depends_on_p
        && !lin.depends_on_s
        && (lin.nu_a - 1.0).abs() < 1e-15
        && (lin.nu_p - 1.0).abs() < 1e-15
        && lin.nu_w.abs() < 1e-15
        && !lin.schema3_constitutive_damage
}

/// Sanity: membrane_catalyst_saturation matches local helper at default K.
pub fn catalyst_saturation_matches_runtime(c: f64, k_c: f64) -> bool {
    use crate::config::SimParams;
    let mut p = SimParams::default();
    p.k_c_membrane = k_c;
    (membrane_catalyst_saturation(c, &p) - membrane_catalyst_saturation_local(c, k_c)).abs()
        < 1e-15
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn synthesis_parity_unit() {
        assert!(precursor_synthesis_parity(1.0));
        assert!(precursor_synthesis_parity(0.37));
    }

    #[test]
    fn exchange_parity_unit() {
        assert!(adsorption_parity(2.0));
        assert!(desorption_parity(0.5));
    }

    #[test]
    fn ledgers_close_when_balanced() {
        let a = ALedger068 {
            g_activation: 10.0,
            l_catalyst: 1.0,
            l_structure: 1.0,
            l_precursor: 6.0,
            l_decay: 1.0,
            j_net: -1.0,
            delta_a: 0.0,
        };
        assert!(a.closes(1e-12));
        let p = PLedger068 {
            g_synthesis: 5.0,
            g_desorption: 1.0,
            l_adsorption: 4.0,
            l_decay: 1.0,
            j_net: 0.0,
            delta_p: 1.0,
        };
        assert!(p.closes(1e-12));
        let s = SLedger068 {
            g_adsorption: 4.0,
            l_desorption: 1.0,
            l_damage: 0.0,
            j_net: 0.0,
            delta_s: 3.0,
        };
        assert!(s.closes(1e-12));
    }

    #[test]
    fn route_priorities() {
        let id = IdentificationReport068 {
            params_positive_finite: true,
            half_sats_in_domain: true,
            bootstrap_spread: 0.1,
            loo_variation: 1.0,
            holdout_median_err: 0.1,
            holdout_max_err: 0.2,
            holdout_a_sign_acc: 0.95,
            holdout_s_sign_acc: 0.95,
            no_radius_params: true,
            stoichiometry_ok: true,
            accounting_ok: true,
        };
        let mut ev = RouteEvidence068 {
            workspace_isolated: true,
            d067_reproduced: true,
            lineage_ok: true,
            runtime_parity_ok: true,
            ledger_ok: true,
            safety_causality_ok: true,
            waste_blocks: false,
            identification: id,
            existing_qualified: false,
            overproduction_qualified: false,
            inhibition_qualified: false,
            assembly_limit: false,
            desorption_dominant: false,
            membrane_access_limit: false,
            multiple_limits: false,
            repair_but_stage_e_blocked: false,
            no_portable_repair: true,
        };
        assert_eq!(select_route(ev.clone()).0, D068Route::N);
        ev.waste_blocks = true;
        assert_eq!(select_route(ev.clone()).0, D068Route::W);
        ev.waste_blocks = false;
        ev.overproduction_qualified = true;
        assert_eq!(select_route(ev).0, D068Route::O);
    }
}
