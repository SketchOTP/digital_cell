//! D-044 activation-law architecture review (observer-only analysis helpers).
//!
//! Compares the historical mass-action law `r = k·C·N·F` against bounded
//! substrate-saturation candidates on the frozen D-043 reconstruction family.
//!
//! Dimensionless substrate activities use frozen reservoir reference concentrations:
//! `n = N / N_reference`, `f = F / F_reference` with
//! `N_reference = F_reference = 1.0` from `SimParams::default()` (`n_reservoir`,
//! `f_reservoir` in `config.rs` — historical governed reservoir defaults).

use crate::d043_analysis::{
    activation_basis, evaluate_portable_rate, RateEstimate, PortableRateReport, D043_BASIS_FLOOR,
    D043_PORTABLE_MIN_ESTIMATES,
};
use serde::{Deserialize, Serialize};

pub const D044_STARTING_COMMIT: &str = "ff35e0f";
pub const D044_D043_TAG: &str = "D-043-activation-capacity-fail";
pub const D044_AGENT_MEMORY_ID: &str = "D-20260719-d044-activation-law-architecture-review";
pub const D044_RECORD: &str = "SCALAR_MASS_ACTION_RECALIBRATION_REJECTED_PENDING_LAW_REVIEW";
pub const D044_HISTORICAL_K: f64 = 0.020;
pub const D044_DIAGNOSTIC_HORIZON: u64 = 3_000;
pub const D044_PORTABLE_MAX_SPAN: f64 = 3.0;
pub const D044_LOO_MEDIAN_TOL: f64 = 0.50;
pub const D044_HELDOUT_MEDIAN_ERR: f64 = 0.20;
pub const D044_HELDOUT_MAX_ERR: f64 = 0.40;
pub const D044_BOOTSTRAP_SPREAD_MAX: f64 = 0.50;
pub const D044_LOO_FACTOR: f64 = 2.0;

/// Frozen governed reservoir reference concentrations (provenance: `SimParams` defaults).
pub const D044_N_REFERENCE: f64 = 1.0;
pub const D044_F_REFERENCE: f64 = 1.0;

/// Sealed D-043 Gate 3 reference span for reconstruction tolerance (5%).
pub const D043_SEALED_SPAN: f64 = 3.38;
pub const D043_RECONSTRUCTION_SPAN_REL_TOL: f64 = 0.05;
pub const D043_RECONSTRUCTION_K_REL_TOL: f64 = 0.10;

/// Relative tolerance for normalized flow stability in eligibility audit.
pub const D044_FLOW_STABILITY_REL_TOL: f64 = 0.05;

/// Minimum consecutive windows for balance eligibility.
pub const D044_ELIGIBILITY_MIN_WINDOWS: usize = 3;

/// Per-catalyst scaling audit relative tolerance (Gate 2).
pub const D044_SCALING_REL_TOL: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D044Conclusion {
    HistoricalActivationLawQualified,
    JointSaturationActivationQualified,
    DualSaturationActivationQualified,
    D043PortabilityFailureNotUpheld,
    D043PortabilityFailureUpheld,
    ActivationStateEligibilityDefect,
    ActivationScalingDefect,
    ActivationLawArchitectureRejected,
    ActivationLawNumericalFailure,
    ActivationCapacityRepairNotFound,
    FoundationalActivationRegression,
    MembraneBasinNotRecovered,
    ContinuousReplacementNotRecovered,
    DamageRepairNotRecovered,
    ResourceDependenceNotEstablished,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D044Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HistoricalActivationLawQualified => "D044_HISTORICAL_ACTIVATION_LAW_QUALIFIED",
            Self::JointSaturationActivationQualified => "D044_JOINT_SATURATION_ACTIVATION_QUALIFIED",
            Self::DualSaturationActivationQualified => "D044_DUAL_SATURATION_ACTIVATION_QUALIFIED",
            Self::D043PortabilityFailureNotUpheld => "D044_D043_PORTABILITY_FAILURE_NOT_UPHELD",
            Self::D043PortabilityFailureUpheld => "D044_D043_PORTABILITY_FAILURE_UPHELD",
            Self::ActivationStateEligibilityDefect => "D044_ACTIVATION_STATE_ELIGIBILITY_DEFECT",
            Self::ActivationScalingDefect => "D044_ACTIVATION_SCALING_DEFECT",
            Self::ActivationLawArchitectureRejected => "D044_ACTIVATION_LAW_ARCHITECTURE_REJECTED",
            Self::ActivationLawNumericalFailure => "D044_ACTIVATION_LAW_NUMERICAL_FAILURE",
            Self::ActivationCapacityRepairNotFound => "D044_ACTIVATION_CAPACITY_REPAIR_NOT_FOUND",
            Self::FoundationalActivationRegression => "D044_FOUNDATIONAL_ACTIVATION_REGRESSION",
            Self::MembraneBasinNotRecovered => "D044_MEMBRANE_BASIN_NOT_RECOVERED",
            Self::ContinuousReplacementNotRecovered => "D044_CONTINUOUS_REPLACEMENT_NOT_RECOVERED",
            Self::DamageRepairNotRecovered => "D044_DAMAGE_REPAIR_NOT_RECOVERED",
            Self::ResourceDependenceNotEstablished => "D044_RESOURCE_DEPENDENCE_NOT_ESTABLISHED",
            Self::StageEMembraneContractFailure => "D044_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D044_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D044_NUMERICAL_FAILURE",
            Self::Fail => "D044_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StateEligibility {
    Steady,
    QualifiedQuasiSteady,
    Transient,
    StarvationTransition,
    ForcedDiagnostic,
    TerminalCollapse,
}

impl StateEligibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steady => "STEADY",
            Self::QualifiedQuasiSteady => "QUALIFIED_QUASI_STEADY",
            Self::Transient => "TRANSIENT",
            Self::StarvationTransition => "STARVATION_TRANSITION",
            Self::ForcedDiagnostic => "FORCED_DIAGNOSTIC",
            Self::TerminalCollapse => "TERMINAL_COLLAPSE",
        }
    }

    pub const fn balance_eligible(self) -> bool {
        matches!(self, Self::Steady | Self::QualifiedQuasiSteady)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationLawId {
    CandidateA,
    CandidateB,
    CandidateC,
}

impl ActivationLawId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CandidateA => "CANDIDATE_A",
            Self::CandidateB => "CANDIDATE_B",
            Self::CandidateC => "CANDIDATE_C",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchitectureRoute {
    V8Schema3HistoricalActivation,
    V13Schema3JointSaturation,
    V13Schema3DualSaturation,
}

impl ArchitectureRoute {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V8Schema3HistoricalActivation => "V8_SCHEMA3_HISTORICAL_ACTIVATION",
            Self::V13Schema3JointSaturation => "V13_SCHEMA3_JOINT_SATURATION",
            Self::V13Schema3DualSaturation => "V13_SCHEMA3_DUAL_SATURATION",
        }
    }

    pub const fn for_law(law: ActivationLawId) -> Self {
        match law {
            ActivationLawId::CandidateA => Self::V8Schema3HistoricalActivation,
            ActivationLawId::CandidateB => Self::V13Schema3JointSaturation,
            ActivationLawId::CandidateC => Self::V13Schema3DualSaturation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ViableDomainClass {
    ViableResourceLimited,
    RecoverableStarvation,
    IrreversibleStarvation,
    SyntheticDiagnostic,
}

impl ViableDomainClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ViableResourceLimited => "VIABLE_RESOURCE_LIMITED",
            Self::RecoverableStarvation => "RECOVERABLE_STARVATION",
            Self::IrreversibleStarvation => "IRREVERSIBLE_STARVATION",
            Self::SyntheticDiagnostic => "SYNTHETIC_DIAGNOSTIC",
        }
    }

    pub const fn portable_fitting_eligible(self) -> bool {
        matches!(self, Self::ViableResourceLimited | Self::RecoverableStarvation)
    }
}

/// One D-043 Gate 3 reconstruction state with diagnostic clamps.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct D043ReconstructionState {
    pub label: &'static str,
    pub clamp_c: f64,
    pub clamp_n: f64,
    pub clamp_f: f64,
    pub clamp_a: f64,
}

/// Frozen D-043 Gate 3 state family (8 states).
pub const D043_RECONSTRUCTION_STATES: [D043ReconstructionState; 8] = [
    D043ReconstructionState {
        label: "R16",
        clamp_c: 0.6,
        clamp_n: 0.7,
        clamp_f: 0.7,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "R22",
        clamp_c: 0.8,
        clamp_n: 0.8,
        clamp_f: 0.8,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "R32",
        clamp_c: 1.0,
        clamp_n: 0.9,
        clamp_f: 0.9,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "low_c",
        clamp_c: 0.3,
        clamp_n: 0.8,
        clamp_f: 0.8,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "med_c",
        clamp_c: 0.6,
        clamp_n: 0.8,
        clamp_f: 0.8,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "high_c",
        clamp_c: 1.0,
        clamp_n: 0.8,
        clamp_f: 0.8,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "low_nf",
        clamp_c: 0.8,
        clamp_n: 0.3,
        clamp_f: 0.3,
        clamp_a: 0.5,
    },
    D043ReconstructionState {
        label: "high_nf",
        clamp_c: 0.8,
        clamp_n: 1.0,
        clamp_f: 1.0,
        clamp_a: 0.5,
    },
];

/// Sealed D-043 k_required values for Gate 0 reconstruction check.
pub const D043_SEALED_K_REQUIRED: [(&str, f64); 7] = [
    ("R16", 0.373),
    ("R22", 0.226),
    ("R32", 0.150),
    ("low_c", 0.491),
    ("med_c", 0.285),
    ("high_c", 0.189),
    ("high_nf", 0.145),
];

/// Preregistered fit/holdout state identity (Gate 4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivationStateSpec {
    pub label: String,
    pub clamp_c: Option<f64>,
    pub clamp_n: Option<f64>,
    pub clamp_f: Option<f64>,
    pub clamp_a: Option<f64>,
    pub radius: Option<f64>,
    pub role: String,
    pub transient: bool,
}

/// Per-window observables for eligibility classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EligibilityWindow {
    pub c_flow: f64,
    pub n_flow: f64,
    pub f_flow: f64,
    pub a_flow: f64,
    pub c_mean: f64,
    pub n_mean: f64,
    pub f_mean: f64,
    pub a_mean: f64,
    pub l_a: f64,
    pub timestep_ok: bool,
    pub concentration_ok: bool,
}

/// Control flags indicating forced diagnostic clamps.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct EligibilityControls {
    pub clamp_a: bool,
    pub clamp_c: bool,
    pub clamp_n: bool,
    pub clamp_f: bool,
    pub clamp_p: bool,
    pub freeze_surface: bool,
}

/// Training-row inputs for saturation-law fitting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivationTrainingRow {
    pub label: String,
    pub c: f64,
    pub n: f64,
    pub f: f64,
    pub l_a: f64,
    pub valid: bool,
}

/// Per-state capacity estimate under a saturation law.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaturationCapacityEstimate {
    pub label: String,
    pub v_required: f64,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateBFitReport {
    pub law: ActivationLawId,
    pub k_nf: f64,
    pub v_b: f64,
    pub estimates: Vec<SaturationCapacityEstimate>,
    pub span: f64,
    pub loo_ok: bool,
    pub loo_max_factor: f64,
    pub bootstrap_spread_rel: f64,
    pub pass: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateCFitReport {
    pub law: ActivationLawId,
    pub k_n: f64,
    pub k_f: f64,
    pub v_c: f64,
    pub estimates: Vec<SaturationCapacityEstimate>,
    pub span: f64,
    pub loo_ok: bool,
    pub loo_max_factor: f64,
    pub bootstrap_spread_rel: f64,
    pub pass: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateSelection {
    pub selected: Option<ActivationLawId>,
    pub route: Option<ArchitectureRoute>,
    pub candidate_a_pass: bool,
    pub candidate_b_pass: bool,
    pub candidate_c_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeldoutSteadyEvaluation {
    pub median_rel_err: f64,
    pub max_rel_err: f64,
    pub pass: bool,
    pub errors: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeldoutTransientEvaluation {
    pub correct_count: usize,
    pub total: usize,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScalingAuditRow {
    pub label: String,
    pub radius: f64,
    pub r_activation: f64,
    pub catalyst_mass: f64,
    pub r_per_catalyst: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScalingAuditReport {
    pub rows: Vec<ScalingAuditRow>,
    pub reference_r_per_catalyst: f64,
    pub max_rel_deviation: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViableDomainAudit {
    pub label: String,
    pub n_reservoir: f64,
    pub f_reservoir: f64,
    pub n_internal: f64,
    pub f_internal: f64,
    pub n_influx: f64,
    pub f_influx: f64,
    pub n_consumption: f64,
    pub f_consumption: f64,
    pub a_balance: f64,
    pub survival_expected: bool,
    pub classification: ViableDomainClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct D043ReconstructionCheck {
    pub span: f64,
    pub span_ok: bool,
    pub k_checks: Vec<(String, f64, f64, bool)>,
    pub pass: bool,
}

// ─── Substrate activities and rate laws ───────────────────────────────────────

/// Dimensionless substrate activities `(n, f)`.
#[inline]
pub fn dimensionless_activities(n: f64, f: f64, n_ref: f64, f_ref: f64) -> (f64, f64) {
    let nr = n_ref.max(1e-30);
    let fr = f_ref.max(1e-30);
    ((n / nr).max(0.0), (f / fr).max(0.0))
}

/// Candidate A — historical mass action: `r = k·C·N·F`.
#[inline]
pub fn activation_rate_a(k: f64, c: f64, n: f64, f: f64) -> f64 {
    k * activation_basis(c, n, f)
}

/// Joint activity `z = n·f` with dimensionless activities.
#[inline]
pub fn joint_activity(n: f64, f: f64, n_ref: f64, f_ref: f64) -> f64 {
    let (na, fa) = dimensionless_activities(n, f, n_ref, f_ref);
    na * fa
}

/// Saturation factor for Candidate B: `z / (K_NF + z)`.
#[inline]
pub fn saturation_factor_b(n: f64, f: f64, k_nf: f64, n_ref: f64, f_ref: f64) -> f64 {
    let z = joint_activity(n, f, n_ref, f_ref);
    if z <= 0.0 || k_nf < 0.0 {
        return 0.0;
    }
    z / (k_nf + z)
}

/// Candidate B — joint substrate saturation: `r = V_B·C·z/(K_NF+z)`.
#[inline]
pub fn activation_rate_b(
    v_b: f64,
    c: f64,
    n: f64,
    f: f64,
    k_nf: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    if c <= 0.0 || v_b <= 0.0 {
        return 0.0;
    }
    v_b * c.max(0.0) * saturation_factor_b(n, f, k_nf, n_ref, f_ref)
}

/// Saturation factor for Candidate C: `[n/(K_N+n)]·[f/(K_F+f)]`.
#[inline]
pub fn saturation_factor_c(
    n: f64,
    f: f64,
    k_n: f64,
    k_f: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    let (na, fa) = dimensionless_activities(n, f, n_ref, f_ref);
    if na <= 0.0 || fa <= 0.0 {
        return 0.0;
    }
    let fn_ = if k_n < 0.0 { 0.0 } else { na / (k_n + na) };
    let ff = if k_f < 0.0 { 0.0 } else { fa / (k_f + fa) };
    fn_ * ff
}

/// Candidate C — independent substrate saturation.
#[inline]
pub fn activation_rate_c(
    v_c: f64,
    c: f64,
    n: f64,
    f: f64,
    k_n: f64,
    k_f: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    if c <= 0.0 || v_c <= 0.0 {
        return 0.0;
    }
    v_c * c.max(0.0) * saturation_factor_c(n, f, k_n, k_f, n_ref, f_ref)
}

/// Required `V_B` from sustained demand and local concentrations.
#[inline]
pub fn required_v_b(
    l_a: f64,
    c: f64,
    n: f64,
    f: f64,
    k_nf: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    let basis = c.max(0.0) * saturation_factor_b(n, f, k_nf, n_ref, f_ref);
    if basis <= D043_BASIS_FLOOR || l_a < 0.0 {
        return f64::INFINITY;
    }
    l_a / basis
}

/// Required `V_C` from sustained demand and local concentrations.
#[inline]
pub fn required_v_c(
    l_a: f64,
    c: f64,
    n: f64,
    f: f64,
    k_n: f64,
    k_f: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    let basis = c.max(0.0) * saturation_factor_c(n, f, k_n, k_f, n_ref, f_ref);
    if basis <= D043_BASIS_FLOOR || l_a < 0.0 {
        return f64::INFINITY;
    }
    l_a / basis
}

// ─── Algebraic controls ───────────────────────────────────────────────────────

pub fn zero_control_passes_a(k: f64) -> bool {
    [(0.0, 1.0, 1.0), (1.0, 0.0, 1.0), (1.0, 1.0, 0.0)]
        .iter()
        .all(|&(c, n, f)| activation_rate_a(k, c, n, f).abs() <= 1e-15)
}

pub fn zero_control_passes_b(v_b: f64, k_nf: f64) -> bool {
    [(0.0, 1.0, 1.0), (1.0, 0.0, 1.0), (1.0, 1.0, 0.0)]
        .iter()
        .all(|&(c, n, f)| {
            activation_rate_b(v_b, c, n, f, k_nf, D044_N_REFERENCE, D044_F_REFERENCE).abs()
                <= 1e-15
        })
}

pub fn zero_control_passes_c(v_c: f64, k_n: f64, k_f: f64) -> bool {
    [(0.0, 1.0, 1.0), (1.0, 0.0, 1.0), (1.0, 1.0, 0.0)]
        .iter()
        .all(|&(c, n, f)| {
            activation_rate_c(
                v_c,
                c,
                n,
                f,
                k_n,
                k_f,
                D044_N_REFERENCE,
                D044_F_REFERENCE,
            )
            .abs()
                <= 1e-15
        })
}

fn monotonic_in_one<F: Fn(f64) -> f64>(f: F, base: f64, deltas: &[f64]) -> bool {
    let mut prev = f(base);
    for &d in deltas {
        let next = f(base + d);
        if next + 1e-15 < prev {
            return false;
        }
        prev = next;
    }
    true
}

pub fn monotonicity_passes_a(k: f64) -> bool {
    let c0 = 0.5;
    let n0 = 0.6;
    let f0 = 0.7;
    let ds = [0.1, 0.2, 0.3];
    monotonic_in_one(|c| activation_rate_a(k, c, n0, f0), c0, &ds)
        && monotonic_in_one(|n| activation_rate_a(k, c0, n, f0), n0, &ds)
        && monotonic_in_one(|f| activation_rate_a(k, c0, n0, f), f0, &ds)
}

pub fn monotonicity_passes_b(v_b: f64, k_nf: f64) -> bool {
    let c0 = 0.5;
    let n0 = 0.6;
    let f0 = 0.7;
    let ds = [0.1, 0.2, 0.3];
    let rate = |c: f64, n: f64, f: f64| {
        activation_rate_b(v_b, c, n, f, k_nf, D044_N_REFERENCE, D044_F_REFERENCE)
    };
    monotonic_in_one(|c| rate(c, n0, f0), c0, &ds)
        && monotonic_in_one(|n| rate(c0, n, f0), n0, &ds)
        && monotonic_in_one(|f| rate(c0, n0, f), f0, &ds)
}

pub fn monotonicity_passes_c(v_c: f64, k_n: f64, k_f: f64) -> bool {
    let c0 = 0.5;
    let n0 = 0.6;
    let f0 = 0.7;
    let ds = [0.1, 0.2, 0.3];
    let rate = |c: f64, n: f64, f: f64| {
        activation_rate_c(
            v_c,
            c,
            n,
            f,
            k_n,
            k_f,
            D044_N_REFERENCE,
            D044_F_REFERENCE,
        )
    };
    monotonic_in_one(|c| rate(c, n0, f0), c0, &ds)
        && monotonic_in_one(|n| rate(c0, n, f0), n0, &ds)
        && monotonic_in_one(|f| rate(c0, n0, f), f0, &ds)
}

// ─── State eligibility ────────────────────────────────────────────────────────

fn relative_stable(a: f64, b: f64, rel_tol: f64) -> bool {
    let scale = a.abs().max(b.abs()).max(1e-12);
    (a - b).abs() / scale <= rel_tol
}

fn monotonic_depletion_dominates(windows: &[EligibilityWindow]) -> bool {
    if windows.len() < 3 {
        return false;
    }
    let n_drop = windows.windows(2).filter(|w| w[1].n_mean + 1e-12 < w[0].n_mean).count();
    let f_drop = windows.windows(2).filter(|w| w[1].f_mean + 1e-12 < w[0].f_mean).count();
    let c_drop = windows.windows(2).filter(|w| w[1].c_mean + 1e-12 < w[0].c_mean).count();
    let len = windows.len().saturating_sub(1);
    n_drop >= len || f_drop >= len || c_drop >= len
}

/// Classify balance-state eligibility from a window series and control flags.
pub fn classify_state_eligibility(
    windows: &[EligibilityWindow],
    controls: &EligibilityControls,
) -> StateEligibility {
    if controls.clamp_a
        || controls.clamp_c
        || controls.clamp_n
        || controls.clamp_f
        || controls.clamp_p
        || controls.freeze_surface
    {
        return StateEligibility::ForcedDiagnostic;
    }
    if windows.is_empty() {
        return StateEligibility::TerminalCollapse;
    }
    if windows.iter().any(|w| !w.timestep_ok || !w.concentration_ok) {
        return StateEligibility::TerminalCollapse;
    }
    if windows.len() < D044_ELIGIBILITY_MIN_WINDOWS {
        return StateEligibility::Transient;
    }
    for chunk in windows.windows(D044_ELIGIBILITY_MIN_WINDOWS) {
        let flows_stable = chunk.windows(2).all(|pair| {
            relative_stable(pair[0].c_flow, pair[1].c_flow, D044_FLOW_STABILITY_REL_TOL)
                && relative_stable(pair[0].n_flow, pair[1].n_flow, D044_FLOW_STABILITY_REL_TOL)
                && relative_stable(pair[0].f_flow, pair[1].f_flow, D044_FLOW_STABILITY_REL_TOL)
                && relative_stable(pair[0].a_flow, pair[1].a_flow, D044_FLOW_STABILITY_REL_TOL)
        });
        if !flows_stable {
            continue;
        }
        if monotonic_depletion_dominates(chunk) {
            return StateEligibility::StarvationTransition;
        }
        let quasi = chunk.iter().all(|w| w.l_a.is_finite() && w.l_a >= 0.0);
        if quasi {
            let means_flat = chunk.windows(2).all(|pair| {
                relative_stable(pair[0].c_mean, pair[1].c_mean, D044_FLOW_STABILITY_REL_TOL)
                    && relative_stable(pair[0].n_mean, pair[1].n_mean, D044_FLOW_STABILITY_REL_TOL)
                    && relative_stable(pair[0].f_mean, pair[1].f_mean, D044_FLOW_STABILITY_REL_TOL)
            });
            if means_flat {
                return StateEligibility::Steady;
            }
            return StateEligibility::QualifiedQuasiSteady;
        }
    }
    if monotonic_depletion_dominates(windows) {
        return StateEligibility::StarvationTransition;
    }
    StateEligibility::Transient
}

// ─── Gate 4 preregistered families ────────────────────────────────────────────

pub fn build_training_states() -> Vec<ActivationStateSpec> {
    vec![
        spec("R16", Some(0.6), Some(0.7), Some(0.7), Some(0.5), Some(16.0), "training", false),
        spec("R22", Some(0.8), Some(0.8), Some(0.8), Some(0.5), Some(22.0), "training", false),
        spec("R32", Some(1.0), Some(0.9), Some(0.9), Some(0.5), Some(32.0), "training", false),
        spec("low_c", Some(0.3), Some(0.8), Some(0.8), Some(0.5), Some(22.0), "training", false),
        spec("med_c", Some(0.6), Some(0.8), Some(0.8), Some(0.5), Some(22.0), "training", false),
        spec("high_c", Some(1.0), Some(0.8), Some(0.8), Some(0.5), Some(22.0), "training", false),
        spec(
            "med_nf",
            Some(0.8),
            Some(0.8),
            Some(0.8),
            Some(0.5),
            Some(22.0),
            "training",
            false,
        ),
    ]
}

pub fn build_holdout_states() -> Vec<ActivationStateSpec> {
    vec![
        spec(
            "low_n",
            Some(0.8),
            Some(0.35),
            Some(0.8),
            Some(0.5),
            Some(22.0),
            "holdout",
            false,
        ),
        spec(
            "low_f",
            Some(0.8),
            Some(0.8),
            Some(0.35),
            Some(0.5),
            Some(22.0),
            "holdout",
            false,
        ),
        spec(
            "high_nf",
            Some(0.8),
            Some(1.0),
            Some(1.0),
            Some(0.5),
            Some(22.0),
            "holdout",
            false,
        ),
        spec(
            "healthy_membrane",
            None,
            None,
            None,
            None,
            Some(22.0),
            "holdout",
            false,
        ),
        spec(
            "low_membrane_precollapse",
            None,
            None,
            None,
            None,
            Some(22.0),
            "holdout",
            false,
        ),
        spec(
            "damage_recovery_window",
            None,
            None,
            None,
            None,
            Some(22.0),
            "holdout",
            true,
        ),
    ]
}

fn spec(
    label: &str,
    c: Option<f64>,
    n: Option<f64>,
    f: Option<f64>,
    a: Option<f64>,
    radius: Option<f64>,
    role: &str,
    transient: bool,
) -> ActivationStateSpec {
    ActivationStateSpec {
        label: label.to_string(),
        clamp_c: c,
        clamp_n: n,
        clamp_f: f,
        clamp_a: a,
        radius,
        role: role.to_string(),
        transient,
    }
}

// ─── D-043 reconstruction tolerance ───────────────────────────────────────────

pub fn d043_reconstruction_within_tolerance(span: f64, estimates: &[RateEstimate]) -> D043ReconstructionCheck {
    let span_ok = span.is_finite()
        && ((span - D043_SEALED_SPAN).abs() / D043_SEALED_SPAN) <= D043_RECONSTRUCTION_SPAN_REL_TOL;
    let mut k_checks = Vec::new();
    for &(label, sealed) in &D043_SEALED_K_REQUIRED {
        let found = estimates.iter().find(|e| e.label == label);
        let (observed, ok) = match found {
            Some(e) if e.k_required.is_finite() && e.k_required > 0.0 => {
                let rel = (e.k_required - sealed).abs() / sealed;
                (e.k_required, rel <= D043_RECONSTRUCTION_K_REL_TOL)
            }
            _ => (f64::NAN, false),
        };
        k_checks.push((label.to_string(), sealed, observed, ok));
    }
    let pass = span_ok && k_checks.iter().all(|(_, _, _, ok)| *ok);
    D043ReconstructionCheck {
        span,
        span_ok,
        k_checks,
        pass,
    }
}

// ─── Candidate fitting ──────────────────────────────────────────────────────────

/// Candidate A portable fit — reuses D-043 `evaluate_portable_rate`.
pub fn fit_candidate_a(estimates: &[RateEstimate]) -> PortableRateReport {
    evaluate_portable_rate(estimates)
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mut s = values.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2]
    } else {
        0.5 * (s[n / 2 - 1] + s[n / 2])
    }
}

fn span_factor(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::INFINITY;
    }
    let min_v = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if min_v > 0.0 {
        max_v / min_v
    } else {
        f64::INFINITY
    }
}

fn loo_max_factor(values: &[f64]) -> (bool, f64) {
    if values.len() < 2 {
        return (true, 1.0);
    }
    let full = median(values);
    let mut max_factor = 1.0_f64;
    let mut ok = true;
    for i in 0..values.len() {
        let subset: Vec<f64> = values
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, &v)| v)
            .collect();
        let m = median(&subset);
        if full > 0.0 && m > 0.0 {
            let factor = (m / full).max(full / m);
            max_factor = max_factor.max(factor);
            if factor > D044_LOO_FACTOR {
                ok = false;
            }
        }
    }
    (ok, max_factor)
}

fn bootstrap_spread(values: &[f64], draws: usize, seed: u64) -> f64 {
    if values.len() < 3 || draws == 0 {
        return f64::INFINITY;
    }
    let center = median(values);
    if !(center > 0.0) {
        return f64::INFINITY;
    }
    let n = values.len();
    let mut state = seed;
    let mut samples = Vec::with_capacity(draws);
    for _ in 0..draws {
        let mut acc = Vec::with_capacity(n);
        for _ in 0..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            acc.push(values[(state as usize) % n]);
        }
        samples.push(median(&acc));
    }
    let min_v = samples.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (max_v - min_v).abs() / center
}

fn training_z_range(rows: &[ActivationTrainingRow]) -> (f64, f64) {
    let mut zs: Vec<f64> = rows
        .iter()
        .filter(|r| r.valid)
        .map(|r| joint_activity(r.n, r.f, D044_N_REFERENCE, D044_F_REFERENCE))
        .filter(|z| *z > 0.0)
        .collect();
    if zs.is_empty() {
        return (0.01, 1.0);
    }
    zs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo = zs.first().copied().unwrap_or(0.01).max(1e-6);
    let hi = zs.last().copied().unwrap_or(1.0).max(lo * 1.01);
    (lo * 0.5, hi * 2.0)
}

fn evaluate_b_at_k_nf(rows: &[ActivationTrainingRow], k_nf: f64) -> CandidateBFitReport {
    let mut estimates = Vec::new();
    for row in rows {
        if !row.valid {
            estimates.push(SaturationCapacityEstimate {
                label: row.label.clone(),
                v_required: f64::INFINITY,
                valid: false,
            });
            continue;
        }
        let v = required_v_b(
            row.l_a,
            row.c,
            row.n,
            row.f,
            k_nf,
            D044_N_REFERENCE,
            D044_F_REFERENCE,
        );
        estimates.push(SaturationCapacityEstimate {
            label: row.label.clone(),
            v_required: v,
            valid: v.is_finite() && v > 0.0,
        });
    }
    let valid_vs: Vec<f64> = estimates
        .iter()
        .filter(|e| e.valid)
        .map(|e| e.v_required)
        .collect();
    let span = span_factor(&valid_vs);
    let (loo_ok, loo_max_factor) = loo_max_factor(&valid_vs);
    let bootstrap_spread_rel = bootstrap_spread(&valid_vs, 24, 0xD044_B_u64);
    let v_b = median(&valid_vs);
    let z_lo = training_z_range(rows).0;
    let z_hi = training_z_range(rows).1;
    let k_in_range = k_nf >= z_lo && k_nf <= z_hi;
    let pass = valid_vs.len() >= D043_PORTABLE_MIN_ESTIMATES
        && span <= D044_PORTABLE_MAX_SPAN
        && loo_ok
        && bootstrap_spread_rel <= D044_BOOTSTRAP_SPREAD_MAX
        && k_in_range
        && zero_control_passes_b(v_b, k_nf)
        && monotonicity_passes_b(v_b, k_nf);
    let mut notes = Vec::new();
    if !k_in_range {
        notes.push(format!("k_nf_outside_range:{k_nf}"));
    }
    CandidateBFitReport {
        law: ActivationLawId::CandidateB,
        k_nf,
        v_b,
        estimates,
        span,
        loo_ok,
        loo_max_factor,
        bootstrap_spread_rel,
        pass,
        notes,
    }
}

/// Grid-search `K_NF` for Candidate B on training rows.
pub fn fit_candidate_b(rows: &[ActivationTrainingRow]) -> CandidateBFitReport {
    let (z_lo, z_hi) = training_z_range(rows);
    let grid_n = 24;
    let log_lo = z_lo.max(1e-6).ln();
    let log_hi = z_hi.max(z_lo * 1.01).ln();
    let mut best: Option<CandidateBFitReport> = None;
    for i in 0..grid_n {
        let t = if grid_n == 1 {
            0.5
        } else {
            i as f64 / (grid_n - 1) as f64
        };
        let k_nf = (log_lo + t * (log_hi - log_lo)).exp();
        let report = evaluate_b_at_k_nf(rows, k_nf);
        if !report.pass {
            continue;
        }
        best = Some(match best {
            None => report,
            Some(prev) if report.span < prev.span => report,
            Some(prev) if (report.span - prev.span).abs() < 1e-12 && report.v_b < prev.v_b => {
                report
            }
            Some(prev) => prev,
        });
    }
    best.unwrap_or_else(|| evaluate_b_at_k_nf(rows, median(&[z_lo, z_hi])))
}

fn evaluate_c_at_k(rows: &[ActivationTrainingRow], k_n: f64, k_f: f64) -> CandidateCFitReport {
    let mut estimates = Vec::new();
    for row in rows {
        if !row.valid {
            estimates.push(SaturationCapacityEstimate {
                label: row.label.clone(),
                v_required: f64::INFINITY,
                valid: false,
            });
            continue;
        }
        let v = required_v_c(
            row.l_a,
            row.c,
            row.n,
            row.f,
            k_n,
            k_f,
            D044_N_REFERENCE,
            D044_F_REFERENCE,
        );
        estimates.push(SaturationCapacityEstimate {
            label: row.label.clone(),
            v_required: v,
            valid: v.is_finite() && v > 0.0,
        });
    }
    let valid_vs: Vec<f64> = estimates
        .iter()
        .filter(|e| e.valid)
        .map(|e| e.v_required)
        .collect();
    let span = span_factor(&valid_vs);
    let (loo_ok, loo_max_factor) = loo_max_factor(&valid_vs);
    let bootstrap_spread_rel = bootstrap_spread(&valid_vs, 24, 0xD044_C_u64);
    let v_c = median(&valid_vs);
    let (na_lo, na_hi) = activity_range(rows, true);
    let (fa_lo, fa_hi) = activity_range(rows, false);
    let k_n_ok = k_n >= na_lo && k_n <= na_hi;
    let k_f_ok = k_f >= fa_lo && k_f <= fa_hi;
    let pass = valid_vs.len() >= D043_PORTABLE_MIN_ESTIMATES
        && span <= D044_PORTABLE_MAX_SPAN
        && loo_ok
        && bootstrap_spread_rel <= D044_BOOTSTRAP_SPREAD_MAX
        && k_n_ok
        && k_f_ok
        && zero_control_passes_c(v_c, k_n, k_f)
        && monotonicity_passes_c(v_c, k_n, k_f);
    CandidateCFitReport {
        law: ActivationLawId::CandidateC,
        k_n,
        k_f,
        v_c,
        estimates,
        span,
        loo_ok,
        loo_max_factor,
        bootstrap_spread_rel,
        pass,
        notes: Vec::new(),
    }
}

fn activity_range(rows: &[ActivationTrainingRow], nutrient: bool) -> (f64, f64) {
    let mut vals: Vec<f64> = rows
        .iter()
        .filter(|r| r.valid)
        .map(|r| {
            if nutrient {
                dimensionless_activities(r.n, r.f, D044_N_REFERENCE, D044_F_REFERENCE).0
            } else {
                dimensionless_activities(r.n, r.f, D044_N_REFERENCE, D044_F_REFERENCE).1
            }
        })
        .filter(|v| *v > 0.0)
        .collect();
    if vals.is_empty() {
        return (0.01, 1.0);
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo = vals.first().copied().unwrap_or(0.01).max(1e-6);
    let hi = vals.last().copied().unwrap_or(1.0).max(lo * 1.01);
    (lo * 0.5, hi * 2.0)
}

/// Grid-search `K_N`, `K_F` for Candidate C.
pub fn fit_candidate_c(rows: &[ActivationTrainingRow]) -> CandidateCFitReport {
    let (na_lo, na_hi) = activity_range(rows, true);
    let (fa_lo, fa_hi) = activity_range(rows, false);
    let grid_n = 12;
    let mut best: Option<CandidateCFitReport> = None;
    for i in 0..grid_n {
        for j in 0..grid_n {
            let ti = i as f64 / (grid_n - 1).max(1) as f64;
            let tj = j as f64 / (grid_n - 1).max(1) as f64;
            let k_n = na_lo + ti * (na_hi - na_lo);
            let k_f = fa_lo + tj * (fa_hi - fa_lo);
            let report = evaluate_c_at_k(rows, k_n, k_f);
            if !report.pass {
                continue;
            }
            best = Some(match best {
                None => report,
                Some(prev) if report.span < prev.span => report,
                Some(prev) if (report.span - prev.span).abs() < 1e-12 && report.v_c < prev.v_c => {
                    report
                }
                Some(prev) => prev,
            });
        }
    }
    best.unwrap_or_else(|| evaluate_c_at_k(rows, 0.5 * (na_lo + na_hi), 0.5 * (fa_lo + fa_hi)))
}

// ─── Held-out validation ──────────────────────────────────────────────────────

pub fn evaluate_heldout_steady(predicted: &[f64], measured: &[f64]) -> HeldoutSteadyEvaluation {
    assert_eq!(predicted.len(), measured.len());
    let mut errors = Vec::new();
    let mut rels = Vec::new();
    for (i, (&p, &m)) in predicted.iter().zip(measured.iter()).enumerate() {
        let scale = m.abs().max(1e-12);
        let rel = (p - m).abs() / scale;
        rels.push(rel);
        errors.push((format!("holdout_{i}"), rel));
    }
    let median_rel_err = median(&rels);
    let max_rel_err = rels.iter().cloned().fold(0.0_f64, f64::max);
    let pass = median_rel_err <= D044_HELDOUT_MEDIAN_ERR && max_rel_err <= D044_HELDOUT_MAX_ERR;
    HeldoutSteadyEvaluation {
        median_rel_err,
        max_rel_err,
        pass,
        errors,
    }
}

pub fn evaluate_heldout_transient(signs_correct: &[bool]) -> HeldoutTransientEvaluation {
    let total = signs_correct.len();
    let correct_count = signs_correct.iter().filter(|&&b| b).count();
    let pass = total >= 6 && correct_count >= 5;
    HeldoutTransientEvaluation {
        correct_count,
        total,
        pass,
    }
}

// ─── Candidate selection ───────────────────────────────────────────────────────

pub fn select_candidate(
    a_pass: bool,
    b_report: &CandidateBFitReport,
    c_report: &CandidateCFitReport,
) -> CandidateSelection {
    let candidate_b_pass = b_report.pass;
    let candidate_c_pass = c_report.pass;
    let (selected, route) = if a_pass {
        (
            Some(ActivationLawId::CandidateA),
            Some(ArchitectureRoute::V8Schema3HistoricalActivation),
        )
    } else if candidate_b_pass {
        (
            Some(ActivationLawId::CandidateB),
            Some(ArchitectureRoute::V13Schema3JointSaturation),
        )
    } else if candidate_c_pass {
        (
            Some(ActivationLawId::CandidateC),
            Some(ArchitectureRoute::V13Schema3DualSaturation),
        )
    } else {
        (None, None)
    };
    CandidateSelection {
        selected,
        route,
        candidate_a_pass: a_pass,
        candidate_b_pass,
        candidate_c_pass,
    }
}

// ─── Scaling audit ────────────────────────────────────────────────────────────

pub fn scaling_audit_row(
    label: &str,
    radius: f64,
    r_activation: f64,
    catalyst_mass: f64,
) -> ScalingAuditRow {
    let r_per_catalyst = if catalyst_mass > 0.0 {
        r_activation / catalyst_mass
    } else {
        f64::NAN
    };
    ScalingAuditRow {
        label: label.to_string(),
        radius,
        r_activation,
        catalyst_mass,
        r_per_catalyst,
    }
}

pub fn evaluate_scaling_audit(rows: &[ScalingAuditRow]) -> ScalingAuditReport {
    let valid: Vec<f64> = rows
        .iter()
        .filter_map(|r| {
            if r.r_per_catalyst.is_finite() && r.r_per_catalyst > 0.0 {
                Some(r.r_per_catalyst)
            } else {
                None
            }
        })
        .collect();
    let reference = median(&valid);
    let mut max_rel = 0.0_f64;
    for &v in &valid {
        if reference > 0.0 {
            max_rel = max_rel.max((v - reference).abs() / reference);
        }
    }
    ScalingAuditReport {
        rows: rows.to_vec(),
        reference_r_per_catalyst: reference,
        max_rel_deviation: max_rel,
        pass: max_rel <= D044_SCALING_REL_TOL,
    }
}

// ─── Viable domain classification ─────────────────────────────────────────────

/// Classify whether a reconstruction state belongs to the viable operating domain.
pub fn classify_viable_domain(
    label: &str,
    n_reservoir: f64,
    f_reservoir: f64,
    n_internal: f64,
    f_internal: f64,
    n_influx: f64,
    f_influx: f64,
    n_consumption: f64,
    f_consumption: f64,
    a_balance: f64,
    forced_clamps: bool,
) -> ViableDomainAudit {
    let n_starvation = n_internal < 0.10 * n_reservoir.max(1e-12);
    let f_starvation = f_internal < 0.10 * f_reservoir.max(1e-12);
    let influx_covers_n = n_influx + 1e-12 >= n_consumption;
    let influx_covers_f = f_influx + 1e-12 >= f_consumption;
    let survival_expected =
        influx_covers_n && influx_covers_f && a_balance >= -1e-6 && !forced_clamps;
    let classification = if forced_clamps || label == "low_nf" {
        if label == "low_nf" && !survival_expected {
            ViableDomainClass::IrreversibleStarvation
        } else if forced_clamps {
            ViableDomainClass::SyntheticDiagnostic
        } else if n_starvation || f_starvation {
            if influx_covers_n && influx_covers_f {
                ViableDomainClass::RecoverableStarvation
            } else {
                ViableDomainClass::IrreversibleStarvation
            }
        } else {
            ViableDomainClass::ViableResourceLimited
        }
    } else if n_starvation || f_starvation {
        if influx_covers_n && influx_covers_f {
            ViableDomainClass::RecoverableStarvation
        } else {
            ViableDomainClass::IrreversibleStarvation
        }
    } else {
        ViableDomainClass::ViableResourceLimited
    };
    ViableDomainAudit {
        label: label.to_string(),
        n_reservoir,
        f_reservoir,
        n_internal,
        f_internal,
        n_influx,
        f_influx,
        n_consumption,
        f_consumption,
        a_balance,
        survival_expected,
        classification,
    }
}

/// Build training rows from rate estimates (observer helper).
pub fn training_rows_from_estimates(estimates: &[RateEstimate]) -> Vec<ActivationTrainingRow> {
    estimates
        .iter()
        .filter(|e| e.valid)
        .map(|e| ActivationTrainingRow {
            label: e.label.clone(),
            c: e.c,
            n: e.n,
            f: e.f,
            l_a: e.l_a,
            valid: e.valid,
        })
        .collect()
}

/// Predict steady authorized demand under the selected law.
pub fn predict_steady_demand_a(k: f64, c: f64, n: f64, f: f64) -> f64 {
    activation_rate_a(k, c, n, f)
}

pub fn predict_steady_demand_b(
    v_b: f64,
    k_nf: f64,
    c: f64,
    n: f64,
    f: f64,
) -> f64 {
    activation_rate_b(v_b, c, n, f, k_nf, D044_N_REFERENCE, D044_F_REFERENCE)
}

pub fn predict_steady_demand_c(
    v_c: f64,
    k_n: f64,
    k_f: f64,
    c: f64,
    n: f64,
    f: f64,
) -> f64 {
    activation_rate_c(
        v_c,
        c,
        n,
        f,
        k_n,
        k_f,
        D044_N_REFERENCE,
        D044_F_REFERENCE,
    )
}

#[cfg(test)]
mod inline_tests {
    use super::*;
    use crate::d042_analysis::ALedgerTerms;
    use crate::d043_analysis::build_rate_estimate;

    #[test]
    fn dimensionless_activities_and_laws() {
        let (n, f) = dimensionless_activities(0.8, 0.6, 1.0, 1.0);
        assert!((n - 0.8).abs() < 1e-15);
        assert!((f - 0.6).abs() < 1e-15);
        assert!((activation_rate_a(0.02, 2.0, 3.0, 4.0) - 0.48).abs() < 1e-15);
        let rb = activation_rate_b(0.5, 1.0, 0.8, 0.6, 0.2, 1.0, 1.0);
        assert!(rb > 0.0 && rb < 0.5);
        let rc = activation_rate_c(0.5, 1.0, 0.8, 0.6, 0.3, 0.2, 1.0, 1.0);
        assert!(rc > 0.0 && rc < 0.5);
    }

    #[test]
    fn saturation_factors_bounded() {
        let sb = saturation_factor_b(1.0, 1.0, 0.5, 1.0, 1.0);
        assert!(sb > 0.0 && sb <= 1.0);
        let sc = saturation_factor_c(1.0, 1.0, 0.5, 0.5, 1.0, 1.0);
        assert!(sc > 0.0 && sc <= 1.0);
    }

    #[test]
    fn zero_controls_all_laws() {
        assert!(zero_control_passes_a(0.02));
        assert!(zero_control_passes_b(0.5, 0.3));
        assert!(zero_control_passes_c(0.5, 0.3, 0.2));
    }

    #[test]
    fn monotonicity_all_laws() {
        assert!(monotonicity_passes_a(0.02));
        assert!(monotonicity_passes_b(0.5, 0.25));
        assert!(monotonicity_passes_c(0.5, 0.3, 0.2));
    }

    #[test]
    fn forced_diagnostic_from_clamps() {
        let windows = vec![EligibilityWindow {
            c_flow: 1.0,
            n_flow: 1.0,
            f_flow: 1.0,
            a_flow: 0.0,
            c_mean: 0.8,
            n_mean: 0.8,
            f_mean: 0.8,
            a_mean: 0.5,
            l_a: 0.1,
            timestep_ok: true,
            concentration_ok: true,
        }; 3];
        let controls = EligibilityControls {
            clamp_a: true,
            ..Default::default()
        };
        assert_eq!(
            classify_state_eligibility(&windows, &controls),
            StateEligibility::ForcedDiagnostic
        );
    }

    #[test]
    fn steady_eligibility_from_stable_windows() {
        let mk = |c| EligibilityWindow {
            c_flow: c,
            n_flow: 1.0,
            f_flow: 1.0,
            a_flow: 0.0,
            c_mean: 0.8,
            n_mean: 0.8,
            f_mean: 0.8,
            a_mean: 0.5,
            l_a: 0.1,
            timestep_ok: true,
            concentration_ok: true,
        };
        let windows = vec![mk(1.0), mk(1.001), mk(0.999)];
        assert_eq!(
            classify_state_eligibility(&windows, &EligibilityControls::default()),
            StateEligibility::Steady
        );
    }

    #[test]
    fn d043_reconstruction_tolerance() {
        let estimates: Vec<RateEstimate> = D043_SEALED_K_REQUIRED
            .iter()
            .map(|(label, k)| RateEstimate {
                label: (*label).to_string(),
                basis: 100.0,
                l_a: k * 100.0,
                k_required: *k,
                c: 0.8,
                n: 0.8,
                f: 0.8,
                valid: true,
                dominated_by_near_zero: false,
            })
            .collect();
        let check = d043_reconstruction_within_tolerance(D043_SEALED_SPAN, &estimates);
        assert!(check.pass);
    }

    #[test]
    fn fit_candidate_a_portable_family() {
        let mut estimates = Vec::new();
        for (i, label) in ["R16", "R22", "R32", "med_c", "high_c", "high_nf"]
            .iter()
            .enumerate()
        {
            let total_basis = 500.0 + 20.0 * i as f64;
            let terms = ALedgerTerms {
                j_reproduction: 0.18,
                j_structural: 0.0,
                j_precursor: 0.0,
                j_decay: 0.0,
                j_out: 0.0,
                j_in: 0.0,
                ..Default::default()
            };
            estimates.push(build_rate_estimate(
                label,
                0.8,
                0.8,
                0.8,
                total_basis,
                &terms,
                D043_BASIS_FLOOR,
            ));
        }
        let report = fit_candidate_a(&estimates);
        assert!(report.valid_count >= D043_PORTABLE_MIN_ESTIMATES);
        assert!(report.span <= D044_PORTABLE_MAX_SPAN + 1e-9);
    }

    #[test]
    fn heldout_steady_and_transient_eval() {
        let steady = evaluate_heldout_steady(&[0.18, 0.19, 0.17], &[0.20, 0.20, 0.20]);
        assert!(steady.pass);
        let transient = evaluate_heldout_transient(&[true, true, true, true, true, false]);
        assert!(transient.pass);
    }

    #[test]
    fn select_candidate_prefers_a() {
        let b = CandidateBFitReport {
            law: ActivationLawId::CandidateB,
            k_nf: 0.2,
            v_b: 0.5,
            estimates: Vec::new(),
            span: 2.0,
            loo_ok: true,
            loo_max_factor: 1.2,
            bootstrap_spread_rel: 0.2,
            pass: true,
            notes: Vec::new(),
        };
        let c = CandidateCFitReport {
            law: ActivationLawId::CandidateC,
            k_n: 0.3,
            k_f: 0.2,
            v_c: 0.5,
            estimates: Vec::new(),
            span: 2.0,
            loo_ok: true,
            loo_max_factor: 1.2,
            bootstrap_spread_rel: 0.2,
            pass: true,
            notes: Vec::new(),
        };
        let sel = select_candidate(true, &b, &c);
        assert_eq!(sel.selected, Some(ActivationLawId::CandidateA));
    }

    #[test]
    fn scaling_audit_radius_independence() {
        let rows = vec![
            scaling_audit_row("R16", 16.0, 100.0, 10.0),
            scaling_audit_row("R22", 22.0, 150.0, 15.0),
            scaling_audit_row("R32", 32.0, 200.0, 20.0),
        ];
        let report = evaluate_scaling_audit(&rows);
        assert!(report.pass);
    }

    #[test]
    fn training_and_holdout_splits_disjoint_roles() {
        let train = build_training_states();
        let hold = build_holdout_states();
        assert!(train.iter().all(|s| s.role == "training"));
        assert!(hold.iter().all(|s| s.role == "holdout"));
        assert!(!train.iter().any(|t| hold.iter().any(|h| t.label == h.label)));
    }
}
