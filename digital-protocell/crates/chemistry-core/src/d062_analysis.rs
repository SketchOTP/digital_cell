//! D-062 long-horizon structural maintenance and decay review helpers.
//! Shadow/observer only — structural synthesis, carrier, and production defaults stay frozen.

use crate::config::SimParams;
use crate::d060_analysis::{
    integrate_existing_structural_rates, q_deficit, DriveSample, StructuralLedger,
};
use crate::d061_analysis::{
    classify_corrected_drive, classify_runaway_collapse, classify_runaway_growth,
    detect_restoring_crossing, CorrectedDriveClass, D061_FROZEN_KT,
};
use crate::structural_kinetics::{structure_decay_rate, STRUCTURAL_EXPOSURE_FLOOR};
use serde::{Deserialize, Serialize};

pub const D062_PROJECT_ID: &str = "D-062";
pub const D062_AGENT_MEMORY_ID: &str = "D-20260721-d062-long-horizon-structural-maintenance-decay";
pub const D062_STARTING_COMMIT: &str = "1d4e2bb";
pub const D062_STARTING_TAG: &str = "D-061-structural-execution-size-revalidation";
pub const D062_D061_SCIENTIFIC: &str = "D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH";
pub const D062_D061_EXECUTION: &str = "D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED";
pub const D062_FROZEN_KT: f64 = D061_FROZEN_KT;
pub const D062_DRIVE_RADII: &[f64] = &[
    4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0,
];
pub const D062_TRAINING_RADII: &[f64] = &[6.0, 10.0, 14.0, 18.0, 22.0];
pub const D062_HOLDOUT_RADII: &[f64] = &[4.0, 8.0, 12.0, 16.0, 20.0, 24.0];
pub const D062_LEDGER_TOL: f64 = 1e-6;
pub const D062_DRIVE_EPS: f64 = 1e-9;
pub const D062_UPDATE_PARITY_TOL: f64 = 1e-6;
pub const D062_A_RETENTION_TARGET: f64 = 0.80;
pub const D062_C_RETENTION_TARGET: f64 = 0.80;
pub const D062_CHI_VIABLE: f64 = 1.05;
pub const D062_SCALAR_SPAN_MAX: f64 = 3.0;
pub const D062_CROSSING_DOMAIN_LO: f64 = 6.0;
pub const D062_CROSSING_DOMAIN_HI: f64 = 14.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D062PrimaryConclusion {
    ExistingStructuralDelayedRestoringBasin,
    GlobalStructuralDecayCalibrationQualified,
    ResourceDependentStructuralMaintenanceQualified,
    StructuralDecayExecutionDefect,
    UnmodifiedStructuralDelayedCollapse,
    SizeRestoredMetabolismNotQualified,
    NoLocalStructuralMaintenanceLaw,
    StructuralMaintenanceReviewInconclusive,
    D061PositiveDriveNotReproduced,
    StructuralScalingInconclusive,
    StructuralMaintenanceCausalityFailure,
    FoundationalRegression,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D062PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExistingStructuralDelayedRestoringBasin => {
                "D062_EXISTING_STRUCTURAL_DELAYED_RESTORING_BASIN"
            }
            Self::GlobalStructuralDecayCalibrationQualified => {
                "D062_GLOBAL_STRUCTURAL_DECAY_CALIBRATION_QUALIFIED"
            }
            Self::ResourceDependentStructuralMaintenanceQualified => {
                "D062_RESOURCE_DEPENDENT_STRUCTURAL_MAINTENANCE_QUALIFIED"
            }
            Self::StructuralDecayExecutionDefect => "D062_STRUCTURAL_DECAY_EXECUTION_DEFECT",
            Self::UnmodifiedStructuralDelayedCollapse => {
                "D062_UNMODIFIED_STRUCTURAL_DELAYED_COLLAPSE"
            }
            Self::SizeRestoredMetabolismNotQualified => {
                "D062_SIZE_RESTORED_METABOLISM_NOT_QUALIFIED"
            }
            Self::NoLocalStructuralMaintenanceLaw => "D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW",
            Self::StructuralMaintenanceReviewInconclusive => {
                "D062_STRUCTURAL_MAINTENANCE_REVIEW_INCONCLUSIVE"
            }
            Self::D061PositiveDriveNotReproduced => "D062_D061_POSITIVE_DRIVE_NOT_REPRODUCED",
            Self::StructuralScalingInconclusive => "D062_STRUCTURAL_SCALING_INCONCLUSIVE",
            Self::StructuralMaintenanceCausalityFailure => {
                "D062_STRUCTURAL_MAINTENANCE_CAUSALITY_FAILURE"
            }
            Self::FoundationalRegression => "D062_FOUNDATIONAL_REGRESSION",
            Self::WorkspaceScopeNotIsolated => "D062_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D062_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D062_NUMERICAL_FAILURE",
            Self::Fail => "D062_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D062Route {
    E,
    K,
    M,
    X,
    C,
    Q,
    N,
    I,
}

impl D062Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E => "Route_E_existing_delayed_restoring_basin",
            Self::K => "Route_K_global_decay_calibration",
            Self::M => "Route_M_resource_dependent_maintenance",
            Self::X => "Route_X_decay_execution_defect",
            Self::C => "Route_C_delayed_collapse",
            Self::Q => "Route_Q_size_restored_metabolism_fails",
            Self::N => "Route_N_no_local_maintenance_law",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D062PrimaryConclusion {
        match self {
            Self::E => D062PrimaryConclusion::ExistingStructuralDelayedRestoringBasin,
            Self::K => D062PrimaryConclusion::GlobalStructuralDecayCalibrationQualified,
            Self::M => D062PrimaryConclusion::ResourceDependentStructuralMaintenanceQualified,
            Self::X => D062PrimaryConclusion::StructuralDecayExecutionDefect,
            Self::C => D062PrimaryConclusion::UnmodifiedStructuralDelayedCollapse,
            Self::Q => D062PrimaryConclusion::SizeRestoredMetabolismNotQualified,
            Self::N => D062PrimaryConclusion::NoLocalStructuralMaintenanceLaw,
            Self::I => D062PrimaryConclusion::StructuralMaintenanceReviewInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BaselineHorizonClass {
    ExistingStructuralDelayedRestoringBasin,
    ExistingStructuralPersistentRunawayGrowth,
    ExistingStructuralDelayedCollapse,
    NumericallyUnresolved,
}

impl BaselineHorizonClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExistingStructuralDelayedRestoringBasin => {
                "EXISTING_STRUCTURAL_DELAYED_RESTORING_BASIN"
            }
            Self::ExistingStructuralPersistentRunawayGrowth => {
                "EXISTING_STRUCTURAL_PERSISTENT_RUNAWAY_GROWTH"
            }
            Self::ExistingStructuralDelayedCollapse => "EXISTING_STRUCTURAL_DELAYED_COLLAPSE",
            Self::NumericallyUnresolved => "NUMERICALLY_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScalingClass {
    GainAndLossVolumeMatched,
    GainScalesFasterThanLoss,
    LossScalesFasterThanGain,
    StructuralSpatialBasisMismatch,
    StructuralScalingInconclusive,
}

impl ScalingClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GainAndLossVolumeMatched => "GAIN_AND_LOSS_VOLUME_MATCHED",
            Self::GainScalesFasterThanLoss => "GAIN_SCALES_FASTER_THAN_LOSS",
            Self::LossScalesFasterThanGain => "LOSS_SCALES_FASTER_THAN_GAIN",
            Self::StructuralSpatialBasisMismatch => "STRUCTURAL_SPATIAL_BASIS_MISMATCH",
            Self::StructuralScalingInconclusive => "STRUCTURAL_SCALING_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaintenanceCandidateId {
    AExisting,
    BGlobalDecayCalibration,
    CResourceDependentMaintenance,
}

impl MaintenanceCandidateId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AExisting => "candidate_A_existing_law",
            Self::BGlobalDecayCalibration => "candidate_B_global_decay_calibration",
            Self::CResourceDependentMaintenance => "candidate_C_resource_dependent_maintenance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceParams {
    pub m_d: f64,
    pub k_a_m: f64,
    pub alpha_m: f64,
}

impl MaintenanceParams {
    pub fn existing() -> Self {
        Self {
            m_d: 1.0,
            k_a_m: 0.0,
            alpha_m: 0.0,
        }
    }

    pub fn global_md(m_d: f64) -> Self {
        Self {
            m_d,
            k_a_m: 0.0,
            alpha_m: 0.0,
        }
    }

    pub fn resource_dependent(k_a_m: f64, alpha_m: f64) -> Self {
        Self {
            m_d: 1.0,
            k_a_m,
            alpha_m,
        }
    }

    pub fn positive_finite(self) -> bool {
        self.m_d.is_finite()
            && self.m_d >= 1.0
            && self.k_a_m.is_finite()
            && self.k_a_m >= 0.0
            && self.alpha_m.is_finite()
            && self.alpha_m >= 0.0
    }
}

/// Existing interface-limited structural loss density.
pub fn existing_loss_density(phi: f64, params: &SimParams) -> f64 {
    structure_decay_rate(phi, 0.0, params)
}

/// Candidate C local loss: `L0 * (1 + α_m * q_def(A))`, bounded by `(1+α_m) L0`.
pub fn candidate_c_loss_density(
    phi: f64,
    activated: f64,
    params: &SimParams,
    k_a_m: f64,
    alpha_m: f64,
) -> f64 {
    let base = existing_loss_density(phi, params);
    let factor = (1.0 + alpha_m.max(0.0) * q_deficit(activated, k_a_m)).min(1.0 + alpha_m.max(0.0));
    base * factor
}

pub fn candidate_loss_density(
    candidate: MaintenanceCandidateId,
    phi: f64,
    activated: f64,
    params: &SimParams,
    cand: MaintenanceParams,
) -> f64 {
    match candidate {
        MaintenanceCandidateId::AExisting => existing_loss_density(phi, params),
        MaintenanceCandidateId::BGlobalDecayCalibration => {
            cand.m_d.max(1.0) * existing_loss_density(phi, params)
        }
        MaintenanceCandidateId::CResourceDependentMaintenance => {
            candidate_c_loss_density(phi, activated, params, cand.k_a_m, cand.alpha_m)
        }
    }
}

/// Required global decay multiplier for zero net structural flow: `m_d★ = G/L`.
pub fn required_decay_multiplier(g_phi: f64, l_phi: f64) -> Option<f64> {
    if !g_phi.is_finite() || !l_phi.is_finite() || l_phi <= 0.0 || g_phi < 0.0 {
        return None;
    }
    Some(g_phi / l_phi)
}

pub fn scalar_multiplier_span(multipliers: &[f64]) -> Option<f64> {
    let finite: Vec<f64> = multipliers
        .iter()
        .copied()
        .filter(|m| m.is_finite() && *m > 0.0)
        .collect();
    if finite.is_empty() {
        return None;
    }
    let min = finite.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = finite.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if min <= 0.0 {
        return None;
    }
    Some(max / min)
}

pub fn scalar_correction_identifiable(multipliers: &[f64]) -> bool {
    match scalar_multiplier_span(multipliers) {
        Some(span) => span <= D062_SCALAR_SPAN_MAX,
        None => false,
    }
}

/// A global \(m_d\) can create a restoring crossing only if required multipliers
/// increase with radius (small states need less amplification than large ones).
pub fn scalar_md_allows_restoring_crossing(radii_and_md: &[(f64, f64)]) -> bool {
    if radii_and_md.len() < 4 {
        return false;
    }
    let mut ordered = radii_and_md.to_vec();
    ordered.sort_by(|a, b| a.0.total_cmp(&b.0));
    if ordered.iter().any(|(r, m)| !r.is_finite() || !m.is_finite() || *m <= 0.0) {
        return false;
    }
    if !scalar_correction_identifiable(&ordered.iter().map(|(_, m)| *m).collect::<Vec<_>>()) {
        return false;
    }
    let radii: Vec<f64> = ordered.iter().map(|(r, _)| *r).collect();
    let mds: Vec<f64> = ordered.iter().map(|(_, m)| *m).collect();
    match fit_power_exponent(&radii, &mds) {
        // Weak but positive radius trend in m_d★ is required for a single global m_d
        // to sit between small-R and large-R zero-net values.
        Some(p) => p > 0.15,
        None => false,
    }
}

pub fn geometric_median(values: &[f64]) -> Option<f64> {
    let mut vals: Vec<f64> = values
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    if vals.is_empty() {
        return None;
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    let n = vals.len();
    if n % 2 == 1 {
        Some(vals[n / 2])
    } else {
        Some(0.5 * (vals[n / 2 - 1] + vals[n / 2]))
    }
}

/// Log-log power fit `y ∝ R^p` via ordinary least squares on ln R, ln y.
pub fn fit_power_exponent(radii: &[f64], values: &[f64]) -> Option<f64> {
    if radii.len() != values.len() || radii.len() < 3 {
        return None;
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (&r, &v) in radii.iter().zip(values.iter()) {
        if r > 0.0 && v > 0.0 && r.is_finite() && v.is_finite() {
            xs.push(r.ln());
            ys.push(v.ln());
        }
    }
    if xs.len() < 3 {
        return None;
    }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        num += (x - mean_x) * (y - mean_y);
        den += (x - mean_x) * (x - mean_x);
    }
    if den.abs() < 1e-18 {
        return None;
    }
    Some(num / den)
}

pub fn classify_gain_loss_scaling(p_g: f64, p_l: f64, tol: f64) -> ScalingClass {
    if !p_g.is_finite() || !p_l.is_finite() || !tol.is_finite() || tol < 0.0 {
        return ScalingClass::StructuralScalingInconclusive;
    }
    let diff = p_g - p_l;
    if diff.abs() <= tol {
        ScalingClass::GainAndLossVolumeMatched
    } else if diff > tol {
        ScalingClass::GainScalesFasterThanLoss
    } else {
        ScalingClass::LossScalesFasterThanGain
    }
}

/// Late-drive restoring basin: ≥2 small radii g_R>0, ≥2 large radii g_R<0, stable crossing.
pub fn classify_delayed_restoring_basin(samples: &[(f64, f64)], eps: f64) -> bool {
    if samples.len() < 4 {
        return false;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_by(|a, b| a.0.total_cmp(&b.0));
    let crossing = detect_restoring_crossing(&ordered, eps);
    if crossing.is_none() {
        return false;
    }
    let Some((r_star, _)) = crossing else {
        return false;
    };
    let positives_below = ordered
        .iter()
        .filter(|(r, g)| *r < r_star && *g > eps)
        .count();
    let negatives_above = ordered
        .iter()
        .filter(|(r, g)| *r > r_star && *g < -eps)
        .count();
    positives_below >= 2 && negatives_above >= 2
}

pub fn classify_baseline_horizon(
    late_samples: &[(f64, f64)],
    radius_deltas: &[f64],
    eps: f64,
) -> BaselineHorizonClass {
    if late_samples
        .iter()
        .any(|(r, g)| !r.is_finite() || !g.is_finite())
        || radius_deltas.iter().any(|d| !d.is_finite())
    {
        return BaselineHorizonClass::NumericallyUnresolved;
    }
    if classify_delayed_restoring_basin(late_samples, eps) {
        return BaselineHorizonClass::ExistingStructuralDelayedRestoringBasin;
    }
    let drive = classify_corrected_drive(late_samples, eps);
    if matches!(
        drive,
        CorrectedDriveClass::NegativeAllRadii | CorrectedDriveClass::NeutralAfterRepair
    ) || classify_runaway_collapse(radius_deltas, eps)
    {
        return BaselineHorizonClass::ExistingStructuralDelayedCollapse;
    }
    if matches!(drive, CorrectedDriveClass::PositiveAllRadii)
        || classify_runaway_growth(radius_deltas, eps)
    {
        return BaselineHorizonClass::ExistingStructuralPersistentRunawayGrowth;
    }
    match drive {
        CorrectedDriveClass::RestoringZeroCrossing => {
            BaselineHorizonClass::ExistingStructuralDelayedRestoringBasin
        }
        CorrectedDriveClass::NumericallyUnresolved => BaselineHorizonClass::NumericallyUnresolved,
        _ => BaselineHorizonClass::ExistingStructuralPersistentRunawayGrowth,
    }
}

pub fn stable_crossing_qualified(samples: &[(f64, f64)], eps: f64) -> Option<f64> {
    let mut ordered = samples.to_vec();
    ordered.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (r_star, slope) = detect_restoring_crossing(&ordered, eps)?;
    if slope >= 0.0 {
        return None;
    }
    let positives_below = ordered
        .iter()
        .filter(|(r, g)| *r < r_star && *g > eps)
        .count();
    let negatives_above = ordered
        .iter()
        .filter(|(r, g)| *r > r_star && *g < -eps)
        .count();
    if positives_below >= 2 && negatives_above >= 2 {
        Some(r_star)
    } else {
        None
    }
}

pub fn crossing_in_supported_domain(r_star: f64) -> bool {
    r_star.is_finite()
        && r_star >= D062_CROSSING_DOMAIN_LO - 2.0
        && r_star <= D062_CROSSING_DOMAIN_HI + 2.0
}

pub fn a_deficit_monotonic(loss_low_a: f64, loss_high_a: f64, eps: f64) -> bool {
    loss_low_a.is_finite()
        && loss_high_a.is_finite()
        && eps.is_finite()
        && loss_low_a + eps >= loss_high_a
}

pub fn zero_a_no_positive_growth(net_at_zero_a: f64, eps: f64) -> bool {
    net_at_zero_a.is_finite() && net_at_zero_a <= eps
}

pub fn decay_to_w_parity(delta_m_phi_loss: f64, delta_m_w: f64, xi_decay: f64, tol: f64) -> bool {
    let phi_ok = (delta_m_phi_loss + xi_decay).abs() <= tol * (1.0 + xi_decay.abs());
    let w_ok = (delta_m_w - xi_decay).abs() <= tol * (1.0 + xi_decay.abs());
    phi_ok && w_ok
}

pub fn counterfactual_loss_equal(fixed_xi: f64, dynamic_xi: f64, tol: f64) -> bool {
    (fixed_xi - dynamic_xi).abs() <= tol * (1.0 + fixed_xi.abs() + dynamic_xi.abs())
}

pub fn exposure_floor() -> f64 {
    STRUCTURAL_EXPOSURE_FLOOR
}

pub fn existing_equation_string() -> &'static str {
    "L_phi,0 = k_structure_decay * phi * (epsilon + I(phi))"
}

pub fn gain_equation_string() -> &'static str {
    "G_phi = k_d008_structure * A * I(phi)"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEvidence062 {
    pub workspace_isolated: bool,
    pub d061_reproduced: bool,
    pub decay_parity_ok: bool,
    pub scaling_ok: bool,
    pub baseline_restoring: bool,
    pub baseline_runaway: bool,
    pub baseline_collapse: bool,
    pub candidate_b_qualified: bool,
    pub candidate_c_qualified: bool,
    pub basin_qualified: bool,
    pub metabolism_qualified: bool,
    pub causality_ok: bool,
    pub foundational_ok: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
}

pub fn select_route(ev: RouteEvidence062) -> (D062Route, D062PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D062Route::I,
            D062PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d061_reproduced {
        return (
            D062Route::I,
            D062PrimaryConclusion::D061PositiveDriveNotReproduced,
        );
    }
    if !ev.decay_parity_ok {
        return (
            D062Route::X,
            D062PrimaryConclusion::StructuralDecayExecutionDefect,
        );
    }
    if !ev.scaling_ok {
        return (
            D062Route::I,
            D062PrimaryConclusion::StructuralScalingInconclusive,
        );
    }
    if !ev.foundational_ok {
        return (D062Route::I, D062PrimaryConclusion::FoundationalRegression);
    }
    if !ev.accounting_ok {
        return (D062Route::I, D062PrimaryConclusion::AccountingFailure);
    }
    if !ev.numerical_ok {
        return (D062Route::I, D062PrimaryConclusion::NumericalFailure);
    }
    if ev.baseline_restoring && ev.basin_qualified && ev.metabolism_qualified {
        return (D062Route::E, D062Route::E.conclusion());
    }
    if ev.baseline_restoring && ev.basin_qualified && !ev.metabolism_qualified {
        return (D062Route::Q, D062Route::Q.conclusion());
    }
    if ev.baseline_collapse {
        return (D062Route::C, D062Route::C.conclusion());
    }
    if ev.candidate_b_qualified && ev.basin_qualified && ev.metabolism_qualified && ev.causality_ok
    {
        return (D062Route::K, D062Route::K.conclusion());
    }
    if ev.candidate_c_qualified && ev.basin_qualified && ev.metabolism_qualified && ev.causality_ok
    {
        return (D062Route::M, D062Route::M.conclusion());
    }
    if (ev.candidate_b_qualified || ev.candidate_c_qualified)
        && ev.basin_qualified
        && !ev.metabolism_qualified
    {
        return (D062Route::Q, D062Route::Q.conclusion());
    }
    if !ev.causality_ok
        && (ev.candidate_b_qualified || ev.candidate_c_qualified || ev.baseline_restoring)
    {
        return (
            D062Route::I,
            D062PrimaryConclusion::StructuralMaintenanceCausalityFailure,
        );
    }
    if ev.baseline_runaway {
        return (D062Route::N, D062Route::N.conclusion());
    }
    (D062Route::I, D062Route::I.conclusion())
}

pub fn integrate_candidate_loss(
    candidate: MaintenanceCandidateId,
    radius: f64,
    activated: f64,
    catalyst: f64,
    params: &SimParams,
    cand: MaintenanceParams,
) -> (f64, f64, f64, f64) {
    let (g, l0, area, iface) =
        integrate_existing_structural_rates(radius, activated, catalyst, params);
    let l = match candidate {
        MaintenanceCandidateId::AExisting => l0,
        MaintenanceCandidateId::BGlobalDecayCalibration => cand.m_d.max(1.0) * l0,
        MaintenanceCandidateId::CResourceDependentMaintenance => {
            let factor = (1.0 + cand.alpha_m.max(0.0) * q_deficit(activated, cand.k_a_m))
                .min(1.0 + cand.alpha_m.max(0.0));
            l0 * factor
        }
    };
    (g, l, area, iface)
}

pub fn drive_sample_from_rates(
    radius: f64,
    g_phi: f64,
    l_phi: f64,
    area: f64,
    interface: f64,
    a_mean: f64,
    c_mean: f64,
) -> DriveSample {
    let net = g_phi - l_phi;
    DriveSample {
        radius,
        g_phi,
        l_phi,
        net_phi: net,
        g_phi_per_area: g_phi / area.max(1e-18),
        g_r: net / (2.0 * std::f64::consts::PI * radius.max(1e-9)),
        interior_area: area,
        interface_length: interface,
        a_mean,
        c_mean,
    }
}

pub fn structural_ledger_closes(
    observed_delta: f64,
    g_phi: f64,
    l_phi: f64,
    j_phi: f64,
    c_phi: f64,
    tol: f64,
) -> bool {
    StructuralLedger {
        g_phi,
        l_phi,
        j_phi,
        c_phi,
        delta_observed: observed_delta,
    }
    .closes(tol)
}

#[cfg(test)]
mod inline_smoke {
    use super::*;

    #[test]
    fn required_multiplier_basic() {
        assert!((required_decay_multiplier(2.0, 1.0).unwrap() - 2.0).abs() < 1e-12);
        assert!(required_decay_multiplier(1.0, 0.0).is_none());
    }
}
