//! D-049 coupled A/P/S collapse feedback decomposition (diagnostic only).
//!
//! Frozen biology: no equation or rate changes. Decomposes collapse chronology,
//! coupled ledgers, control routes, and empirical reduced fixed points.

use crate::config::SimParams;
use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d040_analysis::{j_predicted, required_p_for_theta, theta_eq};
use crate::d048_analysis::d048_frozen_organism_params;
use crate::nullcline::{classify_jacobian, FixedPointClass};
use serde::{Deserialize, Serialize};

pub const D049_AGENT_MEMORY_ID: &str =
    "D-20260720-1534-d049-coupled-aps-collapse-feedback-decomposition";
pub const D049_STARTING_COMMIT: &str = "bdcd6bf";
pub const D049_D048_TAG: &str = "D-048-frozen-biology-membrane-fail";
pub const D049_RECORD: &str = "FROZEN_COUPLED_ORGANISM_COLLAPSE_CONFIRMED";
pub const D049_D047_STATUS: &str = "DIAGNOSTIC_SUPPLY_ADEQUACY_NOT_COUPLED_ATTRACTOR_PROOF";
pub const D049_RETENTION_MIN: f64 = 0.80;
pub const D049_LOCALIZATION_MIN: f64 = 0.95;
pub const D049_LEDGER_REL_TOL: f64 = 0.05;
pub const D049_WINDOW: u64 = 1000;
pub const D049_DEFAULT_HORIZON: u64 = 12_000;
pub const D049_BOOTSTRAP_P: f64 = 0.060;
pub const D049_RADIUS: f64 = 22.0;
pub const D049_THETA: f64 = 0.6;

pub const MEMBRANE_TO_A_FEEDBACK_REQUIRED: &str = "MEMBRANE_TO_A_FEEDBACK_REQUIRED";
pub const UPSTREAM_OF_MEMBRANE: &str = "UPSTREAM_OF_MEMBRANE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D048BranchClass {
    AnalyticSeedCollapse,
    RestoredHealthyCollapse,
    RestoredHealthySurvives,
    RestoredBranchMissing,
    Incomplete,
}

impl D048BranchClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AnalyticSeedCollapse => "ANALYTIC_SEED_COLLAPSE",
            Self::RestoredHealthyCollapse => "RESTORED_HEALTHY_COLLAPSE",
            Self::RestoredHealthySurvives => "RESTORED_HEALTHY_SURVIVES",
            Self::RestoredBranchMissing => "RESTORED_BRANCH_MISSING",
            Self::Incomplete => "INCOMPLETE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D049EarliestCause {
    ActivationProductionDecline,
    ATransportLeakageOnset,
    PrecursorDemandSurge,
    PrecursorSynthesisDecline,
    PrecursorRetentionFailure,
    SurfaceDesorptionOnset,
    PermeabilityFeedbackOnset,
    CatalystRetentionDecline,
    InitialStateOutsideBasin,
    Unknown,
}

impl D049EarliestCause {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationProductionDecline => "ACTIVATION_PRODUCTION_DECLINE",
            Self::ATransportLeakageOnset => "A_TRANSPORT_LEAKAGE_ONSET",
            Self::PrecursorDemandSurge => "PRECURSOR_DEMAND_SURGE",
            Self::PrecursorSynthesisDecline => "PRECURSOR_SYNTHESIS_DECLINE",
            Self::PrecursorRetentionFailure => "PRECURSOR_RETENTION_FAILURE",
            Self::SurfaceDesorptionOnset => "SURFACE_DESORPTION_ONSET",
            Self::PermeabilityFeedbackOnset => "PERMEABILITY_FEEDBACK_ONSET",
            Self::CatalystRetentionDecline => "CATALYST_RETENTION_DECLINE",
            Self::InitialStateOutsideBasin => "INITIAL_STATE_OUTSIDE_BASIN",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D049Route {
    B,
    L,
    P,
    R,
    S,
    A,
    N,
    I,
    EvidenceIncomplete,
    CoupledLedgerFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D049Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::B => "ROUTE_B_BASIN_INACCESSIBLE",
            Self::L => "ROUTE_L_A_LEAKAGE_MEMBRANE_FEEDBACK",
            Self::P => "ROUTE_P_PRECURSOR_DEMAND_REGULATION",
            Self::R => "ROUTE_R_PRECURSOR_RETENTION",
            Self::S => "ROUTE_S_ENDOGENOUS_EXCHANGE_EQUILIBRIUM",
            Self::A => "ROUTE_A_COUPLED_ACTIVATION_CAPACITY",
            Self::N => "ROUTE_N_NO_PHYSICAL_FIXED_POINT",
            Self::I => "ROUTE_I_INCONCLUSIVE",
            Self::EvidenceIncomplete => "ROUTE_EVIDENCE_INCOMPLETE",
            Self::CoupledLedgerFailure => "ROUTE_COUPLED_LEDGER_FAILURE",
            Self::AccountingFailure => "ROUTE_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "ROUTE_NUMERICAL_FAILURE",
            Self::Fail => "ROUTE_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D049Conclusion {
    HealthyAttractorBasinInaccessible,
    ALeakageMembraneFeedback,
    PrecursorDemandRegulationFailure,
    PrecursorRetentionFailure,
    EndogenousExchangeEquilibriumFailure,
    CoupledActivationCapacityFailure,
    NoPhysicalMembraneMetabolismFixedPoint,
    ApsCollapseDecompositionInconclusive,
    D048AttractorEvidenceIncomplete,
    CoupledLedgerFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D049Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HealthyAttractorBasinInaccessible => {
                "D049_HEALTHY_ATTRACTOR_BASIN_INACCESSIBLE"
            }
            Self::ALeakageMembraneFeedback => "D049_A_LEAKAGE_MEMBRANE_FEEDBACK",
            Self::PrecursorDemandRegulationFailure => {
                "D049_PRECURSOR_DEMAND_REGULATION_FAILURE"
            }
            Self::PrecursorRetentionFailure => "D049_PRECURSOR_RETENTION_FAILURE",
            Self::EndogenousExchangeEquilibriumFailure => {
                "D049_ENDOGENOUS_EXCHANGE_EQUILIBRIUM_FAILURE"
            }
            Self::CoupledActivationCapacityFailure => {
                "D049_COUPLED_ACTIVATION_CAPACITY_FAILURE"
            }
            Self::NoPhysicalMembraneMetabolismFixedPoint => {
                "D049_NO_PHYSICAL_MEMBRANE_METABOLISM_FIXED_POINT"
            }
            Self::ApsCollapseDecompositionInconclusive => {
                "D049_APS_COLLAPSE_DECOMPOSITION_INCONCLUSIVE"
            }
            Self::D048AttractorEvidenceIncomplete => "D049_D048_ATTRACTOR_EVIDENCE_INCOMPLETE",
            Self::CoupledLedgerFailure => "D049_COUPLED_LEDGER_FAILURE",
            Self::AccountingFailure => "D049_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D049_NUMERICAL_FAILURE",
            Self::Fail => "D049_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D040ModelDisposition {
    ReducedModelValid,
    ReducedModelOmittedALeakage,
    ReducedModelOmittedPrecursorLoad,
    ReducedModelInvalidExtrapolation,
    HealthyFixedPointNotPhysical,
}

impl D040ModelDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReducedModelValid => "D040_REDUCED_MODEL_VALID",
            Self::ReducedModelOmittedALeakage => "D040_REDUCED_MODEL_OMITTED_A_LEAKAGE",
            Self::ReducedModelOmittedPrecursorLoad => "D040_REDUCED_MODEL_OMITTED_PRECURSOR_LOAD",
            Self::ReducedModelInvalidExtrapolation => "D040_REDUCED_MODEL_INVALID_EXTRAPOLATION",
            Self::HealthyFixedPointNotPhysical => "D040_HEALTHY_FIXED_POINT_NOT_PHYSICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct D048CompletenessReport {
    pub tag_ok: bool,
    pub commit_ok: bool,
    pub analytic_ran: bool,
    pub analytic_pass: bool,
    pub restored_ran: bool,
    pub restored_pass: bool,
    pub restored_snapshot_present: bool,
    pub branch_class: D048BranchClass,
    pub pass_gate0_complete: bool,
}

impl D048CompletenessReport {
    /// Both branches ran and both collapsed — global attractor failure path.
    pub fn both_branches_collapsed(&self) -> bool {
        self.pass_gate0_complete && !self.analytic_pass && !self.restored_pass
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoupledLedgerWindow {
    pub a_prod: f64,
    pub a_loss: f64,
    pub a_delta_obs: f64,
    pub a_closes: bool,
    pub p_prod: f64,
    pub p_loss: f64,
    pub p_gain_desorb: f64,
    pub p_delta_obs: f64,
    pub p_closes: bool,
    pub s_gain_ads: f64,
    pub s_loss: f64,
    pub s_delta_obs: f64,
    pub s_closes: bool,
    pub constitutive_s_destruction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChronologySample {
    pub index: usize,
    pub a_retention: f64,
    pub a_production: f64,
    pub a_leakage: f64,
    pub a_productive_demand: f64,
    pub p_synthesis: f64,
    pub p_leakage: f64,
    pub p_decay: f64,
    pub adsorption: f64,
    pub desorption: f64,
    pub s_occupancy: f64,
    pub permeability_proxy: f64,
    pub c_retention: f64,
    pub n_influx: f64,
    pub f_influx: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence {
    pub numerical_ok: bool,
    pub accounting_ok: bool,
    pub coupled_ledger_ok: bool,
    pub d048_evidence_complete: bool,
    pub analytic_collapses: bool,
    pub restored_survives: bool,
    pub healthy_perm_prevents_collapse: bool,
    pub no_outward_a_prevents_collapse: bool,
    pub precursor_demand_removal_prevents_a_collapse: bool,
    pub p_production_ok: bool,
    pub p_decay_or_leak_keeps_p_low: bool,
    pub exchange_parity_ok: bool,
    pub no_healthy_endogenous_fp: bool,
    pub a_still_deficient_under_controlled_p: bool,
    pub empirical_no_physical_healthy_fp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmpiricalReducedParams {
    pub r_a: f64,
    pub l_rep: f64,
    pub l_structure: f64,
    pub l_precursor: f64,
    pub l_a_decay: f64,
    pub l_a_transport0: f64,
    pub r_p: f64,
    pub l_p_decay: f64,
    pub l_p_transport0: f64,
    pub alpha: f64,
    pub beta: f64,
    pub q_c: f64,
}

impl Default for EmpiricalReducedParams {
    fn default() -> Self {
        Self {
            r_a: 0.02,
            l_rep: 0.01,
            l_structure: 0.01,
            l_precursor: 0.005,
            l_a_decay: 0.01,
            l_a_transport0: 0.05,
            r_p: 0.01,
            l_p_decay: 0.002,
            l_p_transport0: 0.02,
            alpha: D031_ALPHA_FROZEN,
            beta: D031_BETA_FROZEN,
            q_c: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmpiricalFixedPoint {
    pub a: f64,
    pub p: f64,
    pub theta: f64,
    pub eigs: Vec<f64>,
    pub stable: bool,
    pub physical: bool,
}

/// Gate 0 — D-048 branch completeness before D-049 decomposition.
pub fn classify_d048_completeness(
    tag_ok: bool,
    commit_ok: bool,
    analytic_ran: bool,
    analytic_pass: bool,
    restored_ran: bool,
    restored_pass: bool,
    restored_snapshot_present: bool,
) -> D048CompletenessReport {
    let pass_gate0_complete = analytic_ran && restored_ran;
    let branch_class = if !restored_ran || !restored_snapshot_present {
        D048BranchClass::RestoredBranchMissing
    } else if !pass_gate0_complete || !analytic_ran {
        D048BranchClass::Incomplete
    } else if !analytic_pass && restored_pass {
        D048BranchClass::RestoredHealthySurvives
    } else if !analytic_pass && !restored_pass {
        // Both branches collapsed under frozen biology.
        D048BranchClass::AnalyticSeedCollapse
    } else if analytic_pass && !restored_pass {
        D048BranchClass::RestoredHealthyCollapse
    } else {
        D048BranchClass::RestoredHealthySurvives
    };
    D048CompletenessReport {
        tag_ok,
        commit_ok,
        analytic_ran,
        analytic_pass,
        restored_ran,
        restored_pass,
        restored_snapshot_present,
        branch_class,
        pass_gate0_complete,
    }
}

#[inline]
pub fn ledger_closes(predicted_delta: f64, observed_delta: f64, rel_tol: f64) -> bool {
    let scale = predicted_delta.abs().max(observed_delta.abs()).max(1e-12);
    (predicted_delta - observed_delta).abs() / scale <= rel_tol
}

/// Earliest supported causal event that persists across consecutive windows.
pub fn earliest_causal_event(samples: &[ChronologySample]) -> D049EarliestCause {
    if samples.is_empty() {
        return D049EarliestCause::Unknown;
    }
    let s0 = &samples[0];
    if s0.a_retention < D049_RETENTION_MIN * 0.9
        && s0.s_occupancy + 0.05 < D049_THETA
        && s0.p_synthesis < D049_BOOTSTRAP_P
    {
        return D049EarliestCause::InitialStateOutsideBasin;
    }

    let mut first_a_prod = None;
    let mut first_a_leak = None;
    let mut first_p_demand = None;
    let mut first_p_syn = None;
    let mut first_p_ret = None;
    let mut first_desorb = None;
    let mut first_perm = None;
    let mut first_c_ret = None;

    for w in samples.windows(2) {
        let a = &w[0];
        let b = &w[1];
        let i = b.index;
        if first_a_prod.is_none()
            && b.a_production < a.a_production * 0.95
            && b.a_retention < a.a_retention
        {
            first_a_prod = Some(i);
        }
        if first_a_leak.is_none() && b.a_leakage > a.a_leakage * 1.2 + 1e-9 {
            first_a_leak = Some(i);
        }
        if first_p_demand.is_none() && b.a_productive_demand > a.a_productive_demand * 1.15 + 1e-9 {
            first_p_demand = Some(i);
        }
        if first_p_syn.is_none() && b.p_synthesis < a.p_synthesis * 0.95 {
            first_p_syn = Some(i);
        }
        if first_p_ret.is_none()
            && (b.p_leakage > a.p_leakage * 1.2 + 1e-9 || b.p_decay > a.p_decay * 1.2 + 1e-9)
        {
            first_p_ret = Some(i);
        }
        if first_desorb.is_none() && b.desorption > a.desorption * 1.2 + 1e-9 && b.adsorption <= a.adsorption {
            first_desorb = Some(i);
        }
        if first_perm.is_none()
            && b.permeability_proxy > a.permeability_proxy * 1.15
            && b.s_occupancy < a.s_occupancy
        {
            first_perm = Some(i);
        }
        if first_c_ret.is_none() && b.c_retention < a.c_retention * 0.95 {
            first_c_ret = Some(i);
        }
    }

    let mut candidates: Vec<(usize, D049EarliestCause)> = Vec::new();
    if let Some(i) = first_a_prod {
        candidates.push((i, D049EarliestCause::ActivationProductionDecline));
    }
    if let Some(i) = first_a_leak {
        candidates.push((i, D049EarliestCause::ATransportLeakageOnset));
    }
    if let Some(i) = first_p_demand {
        candidates.push((i, D049EarliestCause::PrecursorDemandSurge));
    }
    if let Some(i) = first_p_syn {
        candidates.push((i, D049EarliestCause::PrecursorSynthesisDecline));
    }
    if let Some(i) = first_p_ret {
        candidates.push((i, D049EarliestCause::PrecursorRetentionFailure));
    }
    if let Some(i) = first_desorb {
        candidates.push((i, D049EarliestCause::SurfaceDesorptionOnset));
    }
    if let Some(i) = first_perm {
        candidates.push((i, D049EarliestCause::PermeabilityFeedbackOnset));
    }
    if let Some(i) = first_c_ret {
        candidates.push((i, D049EarliestCause::CatalystRetentionDecline));
    }
    candidates.sort_by_key(|(i, _)| *i);
    candidates
        .first()
        .map(|(_, c)| *c)
        .unwrap_or(D049EarliestCause::Unknown)
}

/// Classify whether membrane permeability/A-leak feedback is required vs upstream deficit.
pub fn classify_frozen_membrane(
    a_retention: f64,
    a_bounded: bool,
    p_bounded: bool,
    c_healthy: bool,
) -> &'static str {
    if c_healthy
        && a_retention < D049_RETENTION_MIN
        && a_bounded
        && p_bounded
    {
        MEMBRANE_TO_A_FEEDBACK_REQUIRED
    } else {
        UPSTREAM_OF_MEMBRANE
    }
}

/// Gate 11 route selection — exactly one primary route and conclusion.
pub fn select_route(ev: &RouteEvidence) -> (D049Route, D049Conclusion) {
    if !ev.numerical_ok {
        return (D049Route::NumericalFailure, D049Conclusion::NumericalFailure);
    }
    if !ev.accounting_ok {
        return (D049Route::AccountingFailure, D049Conclusion::AccountingFailure);
    }
    if !ev.coupled_ledger_ok {
        return (
            D049Route::CoupledLedgerFailure,
            D049Conclusion::CoupledLedgerFailure,
        );
    }
    if !ev.d048_evidence_complete {
        return (
            D049Route::EvidenceIncomplete,
            D049Conclusion::D048AttractorEvidenceIncomplete,
        );
    }
    if ev.restored_survives && ev.analytic_collapses {
        return (
            D049Route::B,
            D049Conclusion::HealthyAttractorBasinInaccessible,
        );
    }
    if ev.healthy_perm_prevents_collapse || ev.no_outward_a_prevents_collapse {
        return (
            D049Route::L,
            D049Conclusion::ALeakageMembraneFeedback,
        );
    }
    if ev.precursor_demand_removal_prevents_a_collapse {
        return (
            D049Route::P,
            D049Conclusion::PrecursorDemandRegulationFailure,
        );
    }
    // Route A before R when sufficient/fixed P still leaves A deficient — retention is not causal.
    if ev.a_still_deficient_under_controlled_p {
        return (
            D049Route::A,
            D049Conclusion::CoupledActivationCapacityFailure,
        );
    }
    if ev.p_production_ok && ev.p_decay_or_leak_keeps_p_low {
        return (
            D049Route::R,
            D049Conclusion::PrecursorRetentionFailure,
        );
    }
    if ev.exchange_parity_ok && ev.no_healthy_endogenous_fp {
        return (
            D049Route::S,
            D049Conclusion::EndogenousExchangeEquilibriumFailure,
        );
    }
    if ev.a_still_deficient_under_controlled_p {
        return (
            D049Route::A,
            D049Conclusion::CoupledActivationCapacityFailure,
        );
    }
    if ev.empirical_no_physical_healthy_fp {
        return (
            D049Route::N,
            D049Conclusion::NoPhysicalMembraneMetabolismFixedPoint,
        );
    }
    (
        D049Route::I,
        D049Conclusion::ApsCollapseDecompositionInconclusive,
    )
}

#[inline]
fn transport_leak(l0: f64, theta: f64, species: f64) -> f64 {
    l0 * (1.0 - theta).max(0.0) * species.max(0.0)
}

/// Empirical reduced APS RHS with occupancy-dependent transport leak.
pub fn empirical_rhs(a: f64, p: f64, theta: f64, par: &EmpiricalReducedParams) -> (f64, f64, f64) {
    let l_a = transport_leak(par.l_a_transport0, theta, a);
    let l_p = transport_leak(par.l_p_transport0, theta, p);
    let da = par.r_a - par.l_a_decay * a - l_a;
    let k = if par.beta > 0.0 {
        par.alpha / par.beta
    } else {
        0.0
    };
    let j = j_predicted(par.alpha, par.beta, par.q_c, p, theta);
    let dp = par.r_p * a.max(0.0) - par.l_p_decay * p - l_p - j;
    let dtheta = j;
    let _ = (par.l_rep, par.l_structure, par.l_precursor, k);
    (da, dp, dtheta)
}

fn jacobian_3(
    a: f64,
    p: f64,
    theta: f64,
    par: &EmpiricalReducedParams,
) -> [[f64; 3]; 3] {
    let eps = 1e-5;
    let f0 = empirical_rhs(a, p, theta, par);
    let fa = empirical_rhs(a + eps, p, theta, par);
    let fp = empirical_rhs(a, p + eps, theta, par);
    let ft = empirical_rhs(a, p, theta + eps, par);
    [
        [
            (fa.0 - f0.0) / eps,
            (fp.0 - f0.0) / eps,
            (ft.0 - f0.0) / eps,
        ],
        [
            (fa.1 - f0.1) / eps,
            (fp.1 - f0.1) / eps,
            (ft.1 - f0.1) / eps,
        ],
        [
            (fa.2 - f0.2) / eps,
            (fp.2 - f0.2) / eps,
            (ft.2 - f0.2) / eps,
        ],
    ]
}

fn push_fp_if_new(
    out: &mut Vec<EmpiricalFixedPoint>,
    a: f64,
    p: f64,
    theta: f64,
    par: &EmpiricalReducedParams,
) {
    let (da, dp, dt) = empirical_rhs(a, p, theta, par);
    let residual = da.abs() + dp.abs() + dt.abs();
    if residual >= 2e-2 || !a.is_finite() || !p.is_finite() || !theta.is_finite() {
        return;
    }
    let j3 = jacobian_3(a, p, theta, par);
    let j2 = [[j3[1][1], j3[1][2]], [j3[2][1], j3[2][2]]];
    let (class, _max_ev) = classify_jacobian(&j2);
    let eigs = {
        let tr = j2[0][0] + j2[1][1];
        let det = j2[0][0] * j2[1][1] - j2[0][1] * j2[1][0];
        let disc = tr * tr - 4.0 * det;
        if disc >= 0.0 {
            let s = disc.sqrt();
            vec![(tr + s) / 2.0, (tr - s) / 2.0]
        } else {
            vec![tr / 2.0, tr / 2.0]
        }
    };
    let stable = matches!(class, FixedPointClass::Stable);
    let physical = a >= 0.0 && p >= 0.0 && theta >= 0.0 && theta < 1.0;
    let dup = out.iter().any(|fp| {
        (fp.a - a).abs() < 0.05 && (fp.p - p).abs() < 0.01 && (fp.theta - theta).abs() < 0.05
    });
    if !dup {
        out.push(EmpiricalFixedPoint {
            a,
            p,
            theta,
            eigs,
            stable,
            physical,
        });
    }
}

fn analytical_exchange_seeds(par: &EmpiricalReducedParams) -> Vec<(f64, f64, f64)> {
    let k = if par.beta > 0.0 {
        par.alpha / par.beta
    } else {
        0.0
    };
    let mut pts = Vec::new();
    for mut p in [0.001, 0.01, 0.02, 0.05, 0.1, 0.2, D049_BOOTSTRAP_P] {
        let mut a = 0.0;
        for _ in 0..32 {
            let theta = theta_eq(k, p);
            let la = par.l_a_transport0 * (1.0 - theta).max(0.0);
            let lp = par.l_p_transport0 * (1.0 - theta).max(0.0);
            a = if par.l_a_decay + la > 0.0 {
                par.r_a / (par.l_a_decay + la)
            } else {
                0.0
            };
            let p_next = if par.l_p_decay + lp > 0.0 {
                (par.r_p * a / (par.l_p_decay + lp)).max(0.0)
            } else {
                p
            };
            if (p_next - p).abs() < 1e-12 {
                p = p_next;
                break;
            }
            p = p_next;
        }
        let theta = theta_eq(k, p);
        pts.push((a.max(0.0), p.max(0.0), theta));
    }
    pts.push((0.0, 0.0, 0.0));
    pts
}

/// Coarse grid + relaxation search for empirical reduced fixed points.
pub fn find_empirical_fixed_points(par: &EmpiricalReducedParams) -> Vec<EmpiricalFixedPoint> {
    let mut out = Vec::new();
    for (a, p, theta) in analytical_exchange_seeds(par) {
        push_fp_if_new(&mut out, a, p, theta, par);
    }
    let starts = [
        (0.1, 0.01, 0.1),
        (0.5, 0.05, 0.5),
        (1.0, 0.1, 0.8),
        (0.2, 0.2, 0.2),
        (0.8, 0.02, 0.7),
        (0.05, 0.001, 0.05),
    ];
    for (mut a, mut p, mut theta) in starts {
        for _ in 0..200 {
            let (da, dp, dt) = empirical_rhs(a, p, theta, par);
            a = (a + 0.1 * da).max(0.0);
            p = (p + 0.1 * dp).max(0.0);
            theta = (theta + 0.1 * dt).clamp(0.0, 0.999);
            if da.abs() + dp.abs() + dt.abs() < 1e-8 {
                break;
            }
        }
        push_fp_if_new(&mut out, a, p, theta, par);
    }
    for a in (1..=10).map(|i| i as f64 * 0.1) {
        for p in [0.01, 0.05, 0.1] {
            let k = if par.beta > 0.0 {
                par.alpha / par.beta
            } else {
                0.0
            };
            let theta = theta_eq(k, p);
            push_fp_if_new(&mut out, a, p, theta, par);
        }
    }
    out
}

pub fn disposition_d040(
    omitted_a_leak: bool,
    omitted_precursor_load: bool,
    fp_physical: bool,
    extrapolation_invalid: bool,
) -> D040ModelDisposition {
    if extrapolation_invalid {
        D040ModelDisposition::ReducedModelInvalidExtrapolation
    } else if !fp_physical {
        D040ModelDisposition::HealthyFixedPointNotPhysical
    } else if omitted_a_leak {
        D040ModelDisposition::ReducedModelOmittedALeakage
    } else if omitted_precursor_load {
        D040ModelDisposition::ReducedModelOmittedPrecursorLoad
    } else {
        D040ModelDisposition::ReducedModelValid
    }
}

/// Frozen D-049 organism params — delegates to D-048 freeze (k_activation = 0.020).
pub fn d049_frozen_params(base: &SimParams) -> SimParams {
    d048_frozen_organism_params(base)
}

/// Healthy empirical fixed point exists at or above occupancy threshold.
pub fn has_physical_healthy_fp(fps: &[EmpiricalFixedPoint], theta_min: f64) -> bool {
    let k = D031_ALPHA_FROZEN / D031_BETA_FROZEN;
    fps.iter().any(|fp| {
        fp.physical
            && fp.stable
            && fp.theta >= theta_min
            && fp.p >= required_p_for_theta(k, theta_min) * 0.5
    })
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn ledger_default_tol() {
        assert!(ledger_closes(1.0, 1.04, D049_LEDGER_REL_TOL));
        assert!(!ledger_closes(1.0, 1.10, D049_LEDGER_REL_TOL));
    }
}
