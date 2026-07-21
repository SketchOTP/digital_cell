//! D-060 structural growth law and resource-coupled size feedback helpers.
//! Observer / shadow diagnostic only: no production biology, no V15, no size controller.

use crate::config::{SimParams, DX, GRID_HEIGHT, GRID_WIDTH};
use crate::d008_analysis::STAGE_E_INTERFACE_WIDTH;
use crate::reactions::interface_weight;
use crate::structural_kinetics::{
    structure_decay_rate, structure_production_rate, STRUCTURAL_EXPOSURE_FLOOR,
};
use serde::{Deserialize, Serialize};

pub const D060_PROJECT_ID: &str = "D-060";
pub const D060_AGENT_MEMORY_ID: &str =
    "D-20260721-d060-structural-growth-resource-size-feedback";
pub const D060_STARTING_COMMIT: &str = "17faa2e";
pub const D060_STARTING_TAG: &str = "D-059-size-membrane-area-architecture-review";
pub const D060_D059_CONCLUSION: &str = "D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN";
pub const D060_D059_PRESERVATION: &str = "D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN";
pub const D060_D059_RECORD: &str = "RESOURCE_SUPPORTED_SMALL_SIZE_BAND_PROVISIONAL";
pub const D060_D059_RESTORING: &str = "NEUTRAL_SIZE_MANIFOLD";
/// Exact sealed D-059 best global carrier rate (full precision).
pub const D060_FROZEN_KT: f64 = 1.4346157818803311;
pub const D060_CHI_VIABLE: f64 = 1.05;
pub const D060_A_RETENTION_TARGET: f64 = 0.80;
pub const D060_C_RETENTION_TARGET: f64 = 0.80;
pub const D060_LEDGER_TOL: f64 = 1e-6;
pub const D060_RADIUS_MAP_TOL: f64 = 1e-3;
pub const D060_DRIVE_EPS: f64 = 1e-8;
pub const D060_HOLDOUT_SIGN_ACC: f64 = 0.90;
pub const D060_HOLDOUT_MEDIAN_ERR: f64 = 0.20;
pub const D060_HOLDOUT_MAX_ERR: f64 = 0.35;
pub const D060_BOOTSTRAP_SPREAD_MAX: f64 = 0.50;
pub const D060_LOO_SPREAD_MAX: f64 = 2.0;

/// Gate 0 / Gate 3 radius campaigns.
pub const D060_REPRO_RADII: &[f64] = &[6.0, 10.0, 14.0, 18.0, 22.0];
pub const D060_DRIVE_RADII: &[f64] = &[
    4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0,
];
pub const D060_TRAIN_RADII: &[f64] = &[6.0, 10.0, 14.0, 18.0, 22.0];
pub const D060_HOLDOUT_RADII: &[f64] = &[4.0, 8.0, 12.0, 16.0, 20.0, 24.0];
pub const D060_KT_LADDER: &[f64] = &[
    0.007383456464644695,
    0.027566313435942912,
    0.10291949848792324,
    0.3842524388913928,
    1.4346157818803311,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D060PrimaryConclusion {
    ExistingStructuralFeedbackQualified,
    StructuralGeometryExecutionDefect,
    ResourceCoupledStructuralSynthesisQualified,
    ResourceDependentStructuralMaintenanceQualified,
    CombinedStructuralFeedbackQualified,
    NoLocalStructuralRestoringLaw,
    StructuralSizeFeedbackInconclusive,
    SizeRestoredMetabolismNotQualified,
    StructuralLossStoichiometryUnresolved,
    D059RouteLNotReproduced,
    WorkspaceScopeNotIsolated,
    StructuralLedgerFailure,
    StructureGeometryMappingDefect,
    StructuralFeedbackCausalityFailure,
    FoundationalRegression,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D060PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExistingStructuralFeedbackQualified => {
                "D060_EXISTING_STRUCTURAL_FEEDBACK_QUALIFIED"
            }
            Self::StructuralGeometryExecutionDefect => {
                "D060_STRUCTURAL_GEOMETRY_EXECUTION_DEFECT"
            }
            Self::ResourceCoupledStructuralSynthesisQualified => {
                "D060_RESOURCE_COUPLED_STRUCTURAL_SYNTHESIS_QUALIFIED"
            }
            Self::ResourceDependentStructuralMaintenanceQualified => {
                "D060_RESOURCE_DEPENDENT_STRUCTURAL_MAINTENANCE_QUALIFIED"
            }
            Self::CombinedStructuralFeedbackQualified => {
                "D060_COMBINED_STRUCTURAL_FEEDBACK_QUALIFIED"
            }
            Self::NoLocalStructuralRestoringLaw => "D060_NO_LOCAL_STRUCTURAL_RESTORING_LAW",
            Self::StructuralSizeFeedbackInconclusive => {
                "D060_STRUCTURAL_SIZE_FEEDBACK_INCONCLUSIVE"
            }
            Self::SizeRestoredMetabolismNotQualified => {
                "D060_SIZE_RESTORED_METABOLISM_NOT_QUALIFIED"
            }
            Self::StructuralLossStoichiometryUnresolved => {
                "D060_STRUCTURAL_LOSS_STOICHIOMETRY_UNRESOLVED"
            }
            Self::D059RouteLNotReproduced => "D060_D059_ROUTE_L_NOT_REPRODUCED",
            Self::WorkspaceScopeNotIsolated => "D060_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::StructuralLedgerFailure => "D060_STRUCTURAL_LEDGER_FAILURE",
            Self::StructureGeometryMappingDefect => "D060_STRUCTURE_GEOMETRY_MAPPING_DEFECT",
            Self::StructuralFeedbackCausalityFailure => {
                "D060_STRUCTURAL_FEEDBACK_CAUSALITY_FAILURE"
            }
            Self::FoundationalRegression => "D060_FOUNDATIONAL_REGRESSION",
            Self::AccountingFailure => "D060_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D060_NUMERICAL_FAILURE",
            Self::Fail => "D060_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D060Route {
    E,
    G,
    S,
    M,
    C,
    N,
    I,
}

impl D060Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E => "Route_E_existing_structural_feedback",
            Self::G => "Route_G_geometry_or_execution_defect",
            Self::S => "Route_S_resource_coupled_structural_synthesis",
            Self::M => "Route_M_resource_dependent_structural_maintenance",
            Self::C => "Route_C_combined_structural_feedback",
            Self::N => "Route_N_no_local_restoring_law",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D060PrimaryConclusion {
        match self {
            Self::E => D060PrimaryConclusion::ExistingStructuralFeedbackQualified,
            Self::G => D060PrimaryConclusion::StructuralGeometryExecutionDefect,
            Self::S => D060PrimaryConclusion::ResourceCoupledStructuralSynthesisQualified,
            Self::M => D060PrimaryConclusion::ResourceDependentStructuralMaintenanceQualified,
            Self::C => D060PrimaryConclusion::CombinedStructuralFeedbackQualified,
            Self::N => D060PrimaryConclusion::NoLocalStructuralRestoringLaw,
            Self::I => D060PrimaryConclusion::StructuralSizeFeedbackInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveSurfaceClass {
    PositiveAllRadii,
    NegativeAllRadii,
    ZeroAllRadii,
    NeutralBand,
    UnstableZeroCrossing,
    RestoringZeroCrossing,
    Nonmonotonic,
}

impl DriveSurfaceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PositiveAllRadii => "POSITIVE_ALL_RADII",
            Self::NegativeAllRadii => "NEGATIVE_ALL_RADII",
            Self::ZeroAllRadii => "ZERO_ALL_RADII",
            Self::NeutralBand => "NEUTRAL_BAND",
            Self::UnstableZeroCrossing => "UNSTABLE_ZERO_CROSSING",
            Self::RestoringZeroCrossing => "RESTORING_ZERO_CROSSING",
            Self::Nonmonotonic => "NONMONOTONIC",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceCausalityClass {
    StructuralSynthesisResourceSensitive,
    StructuralSynthesisResourceInsensitive,
    StructuralLossResourceSensitive,
    NoStructuralMaintenanceLoss,
    StructuralResponseSaturated,
    StructuralResponseDisconnected,
}

impl ResourceCausalityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StructuralSynthesisResourceSensitive => {
                "STRUCTURAL_SYNTHESIS_RESOURCE_SENSITIVE"
            }
            Self::StructuralSynthesisResourceInsensitive => {
                "STRUCTURAL_SYNTHESIS_RESOURCE_INSENSITIVE"
            }
            Self::StructuralLossResourceSensitive => "STRUCTURAL_LOSS_RESOURCE_SENSITIVE",
            Self::NoStructuralMaintenanceLoss => "NO_STRUCTURAL_MAINTENANCE_LOSS",
            Self::StructuralResponseSaturated => "STRUCTURAL_RESPONSE_SATURATED",
            Self::StructuralResponseDisconnected => "STRUCTURAL_RESPONSE_DISCONNECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NeutralityCause {
    StructuralGainAndLossBothZero,
    StructuralGainLossEqualConstant,
    StructuralGainLossEqualScaling,
    StructuralSynthesisAInsensitive,
    StructuralLossMissing,
    StructuralResponseSaturated,
    StructuralGeometryCouplingDefect,
    NumericalRelaxationCancelsBiology,
    MultipleStructuralCauses,
    StructuralNeutralityUnresolved,
}

impl NeutralityCause {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StructuralGainAndLossBothZero => "STRUCTURAL_GAIN_AND_LOSS_BOTH_ZERO",
            Self::StructuralGainLossEqualConstant => "STRUCTURAL_GAIN_LOSS_EQUAL_CONSTANT",
            Self::StructuralGainLossEqualScaling => "STRUCTURAL_GAIN_LOSS_EQUAL_SCALING",
            Self::StructuralSynthesisAInsensitive => "STRUCTURAL_SYNTHESIS_A_INSENSITIVE",
            Self::StructuralLossMissing => "STRUCTURAL_LOSS_MISSING",
            Self::StructuralResponseSaturated => "STRUCTURAL_RESPONSE_SATURATED",
            Self::StructuralGeometryCouplingDefect => "STRUCTURAL_GEOMETRY_COUPLING_DEFECT",
            Self::NumericalRelaxationCancelsBiology => "NUMERICAL_RELAXATION_CANCELS_BIOLOGY",
            Self::MultipleStructuralCauses => "MULTIPLE_STRUCTURAL_CAUSES",
            Self::StructuralNeutralityUnresolved => "STRUCTURAL_NEUTRALITY_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StructuralCandidateId {
    AExisting,
    BCorrectedASynthesis,
    CLocalMaintenanceLoss,
    DBoundedSynthesisPlusMaintenance,
}

impl StructuralCandidateId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AExisting => "candidate_A_existing_structural_law",
            Self::BCorrectedASynthesis => "candidate_B_corrected_A_dependent_synthesis",
            Self::CLocalMaintenanceLoss => "candidate_C_local_maintenance_loss",
            Self::DBoundedSynthesisPlusMaintenance => {
                "candidate_D_bounded_synthesis_plus_maintenance"
            }
        }
    }
}

/// Structural mass ledger: ΔM_φ = G_φ − L_φ + J_φ + C_φ.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StructuralLedger {
    pub g_phi: f64,
    pub l_phi: f64,
    pub j_phi: f64,
    pub c_phi: f64,
    pub delta_observed: f64,
}

impl StructuralLedger {
    pub fn delta_ledger(self) -> f64 {
        self.g_phi - self.l_phi + self.j_phi + self.c_phi
    }

    pub fn closes(self, tol: f64) -> bool {
        (self.delta_observed - self.delta_ledger()).abs() <= tol
            * (1.0 + self.delta_observed.abs() + self.delta_ledger().abs())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DriveSample {
    pub radius: f64,
    pub g_phi: f64,
    pub l_phi: f64,
    pub net_phi: f64,
    pub g_phi_per_area: f64,
    pub g_r: f64,
    pub interior_area: f64,
    pub interface_length: f64,
    pub a_mean: f64,
    pub c_mean: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CandidateParams {
    pub k_a_phi: f64,
    pub k_a_m: f64,
    pub k_phi_m: f64,
}

impl CandidateParams {
    pub fn existing() -> Self {
        Self {
            k_a_phi: 0.0,
            k_a_m: 0.0,
            k_phi_m: 0.0,
        }
    }

    pub fn all_positive_finite(self) -> bool {
        [self.k_a_phi, self.k_a_m, self.k_phi_m]
            .iter()
            .all(|x| x.is_finite() && *x >= 0.0)
    }
}

fn circular_phi(x: f64, y: f64, cx: f64, cy: f64, radius: f64) -> f64 {
    let distance = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    0.5 * (1.0 - ((distance - radius) / STAGE_E_INTERFACE_WIDTH).tanh())
}

/// Governed equivalent radius from occupied interior area.
pub fn equivalent_radius_from_area(interior_area: f64) -> f64 {
    (interior_area.max(0.0) / std::f64::consts::PI).sqrt()
}

pub fn interior_area_from_phi(phi: &[f64], cell_area: f64) -> f64 {
    phi.iter()
        .map(|p| if *p >= 0.5 { cell_area } else { 0.0 })
        .sum()
}

pub fn structural_mass(phi: &[f64], cell_area: f64) -> f64 {
    phi.iter().map(|p| p.max(0.0) * cell_area).sum()
}

/// Integrate existing structural production/decay over a circular prescribed disk.
pub fn integrate_existing_structural_rates(
    radius: f64,
    activated: f64,
    catalyst: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64) {
    let cx = (GRID_WIDTH as f64) * 0.5;
    let cy = (GRID_HEIGHT as f64) * 0.5;
    let cell = DX * DX;
    let mut g = 0.0;
    let mut l = 0.0;
    let mut area = 0.0;
    let mut iface = 0.0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let phi = circular_phi(x as f64 + 0.5, y as f64 + 0.5, cx, cy, radius);
            let a = if phi >= 0.5 { activated.max(0.0) } else { 0.0 };
            let c = if phi >= 0.5 { catalyst.max(0.0) } else { 0.0 };
            let gp = structure_production_rate(phi, a, c, params);
            let lp = structure_decay_rate(phi, 0.0, params);
            g += gp * cell;
            l += lp * cell;
            if phi >= 0.5 {
                area += cell;
            }
            iface += interface_weight(phi) * cell;
        }
    }
    (g, l, area, iface)
}

/// Candidate B: Michaelis–Menten A factor on existing synthesis stoichiometry.
#[inline]
pub fn q_a(a: f64, k_a_phi: f64) -> f64 {
    let a = a.max(0.0);
    if k_a_phi <= 0.0 {
        return if a > 0.0 { 1.0 } else { 0.0 };
    }
    a / (k_a_phi + a)
}

/// Candidate C: deficit-dependent maintenance factor.
#[inline]
pub fn q_deficit(a: f64, k_a_m: f64) -> f64 {
    let a = a.max(0.0);
    if k_a_m <= 0.0 {
        return 0.0;
    }
    k_a_m / (k_a_m + a)
}

/// Candidate B/D production: `k_structure · q_A(A) · I(φ)` (local InterfaceLimitedTurnover support).
pub fn candidate_b_production_rate(
    phi: f64,
    activated: f64,
    params: &SimParams,
    k_a_phi: f64,
) -> f64 {
    params.k_d008_structure * q_a(activated, k_a_phi) * interface_weight(phi)
}

pub fn candidate_decay_rate(
    candidate: StructuralCandidateId,
    phi: f64,
    activated: f64,
    params: &SimParams,
    cand: CandidateParams,
) -> f64 {
    let base = structure_decay_rate(phi, 0.0, params);
    match candidate {
        StructuralCandidateId::AExisting | StructuralCandidateId::BCorrectedASynthesis => base,
        StructuralCandidateId::CLocalMaintenanceLoss
        | StructuralCandidateId::DBoundedSynthesisPlusMaintenance => {
            base + cand.k_phi_m * phi.max(0.0) * q_deficit(activated, cand.k_a_m)
        }
    }
}

pub fn integrate_candidate_rates(
    candidate: StructuralCandidateId,
    radius: f64,
    activated: f64,
    catalyst: f64,
    params: &SimParams,
    cand: CandidateParams,
) -> (f64, f64, f64) {
    let cx = (GRID_WIDTH as f64) * 0.5;
    let cy = (GRID_HEIGHT as f64) * 0.5;
    let cell = DX * DX;
    let mut g = 0.0;
    let mut l = 0.0;
    let mut area = 0.0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let phi = circular_phi(x as f64 + 0.5, y as f64 + 0.5, cx, cy, radius);
            let a = if phi >= 0.5 { activated.max(0.0) } else { 0.0 };
            let c = if phi >= 0.5 { catalyst.max(0.0) } else { 0.0 };
            let gp = match candidate {
                StructuralCandidateId::BCorrectedASynthesis
                | StructuralCandidateId::DBoundedSynthesisPlusMaintenance => {
                    candidate_b_production_rate(phi, a, params, cand.k_a_phi)
                        * if matches!(
                            candidate,
                            StructuralCandidateId::DBoundedSynthesisPlusMaintenance
                        ) {
                            // retain catalyst gate only when existing law required it;
                            // InterfaceLimitedTurnover does not — keep q_A only.
                            1.0
                        } else {
                            1.0
                        }
                }
                _ => structure_production_rate(phi, a, c, params),
            };
            let lp = candidate_decay_rate(candidate, phi, a, params, cand);
            g += gp * cell;
            l += lp * cell;
            if phi >= 0.5 {
                area += cell;
            }
        }
    }
    (g, l, area)
}

/// Map net structural mass rate to approximate radius derivative for a disk.
/// dA/dt = 2π R dR/dt ⇒ dR/dt = (dM/dt) / (2π R) when φ≈1 inside (mass≈area).
pub fn g_r_from_net(net_phi: f64, radius: f64) -> f64 {
    let den = 2.0 * std::f64::consts::PI * radius.max(1e-9);
    net_phi / den
}

pub fn classify_drive_surface(samples: &[DriveSample], eps: f64) -> DriveSurfaceClass {
    if samples.len() < 3 {
        return DriveSurfaceClass::ZeroAllRadii;
    }
    let signs: Vec<i8> = samples
        .iter()
        .map(|s| {
            if s.g_r > eps {
                1
            } else if s.g_r < -eps {
                -1
            } else {
                0
            }
        })
        .collect();
    if signs.iter().all(|&s| s == 0) {
        return DriveSurfaceClass::ZeroAllRadii;
    }
    if signs.iter().all(|&s| s >= 0) && signs.iter().any(|&s| s > 0) {
        return DriveSurfaceClass::PositiveAllRadii;
    }
    if signs.iter().all(|&s| s <= 0) && signs.iter().any(|&s| s < 0) {
        return DriveSurfaceClass::NegativeAllRadii;
    }
    // Find zero crossings in g_r vs R (sorted).
    let mut ordered = samples.to_vec();
    ordered.sort_by(|a, b| a.radius.partial_cmp(&b.radius).unwrap());
    let mut crossings = Vec::new();
    for w in ordered.windows(2) {
        let g0 = w[0].g_r;
        let g1 = w[1].g_r;
        if g0.abs() <= eps && g1.abs() <= eps {
            continue;
        }
        if g0 * g1 < 0.0 || (g0.abs() <= eps) != (g1.abs() <= eps) && g0 * g1 <= 0.0 {
            let r_star = if (g1 - g0).abs() > 1e-18 {
                w[0].radius - g0 * (w[1].radius - w[0].radius) / (g1 - g0)
            } else {
                0.5 * (w[0].radius + w[1].radius)
            };
            let slope = (g1 - g0) / (w[1].radius - w[0].radius).max(1e-18);
            crossings.push((r_star, slope));
        }
    }
    if crossings.is_empty() {
        // Mixed but no clean crossing → neutral band or nonmonotonic
        let near_zero = signs.iter().filter(|&&s| s == 0).count();
        if near_zero * 2 >= signs.len() {
            return DriveSurfaceClass::NeutralBand;
        }
        return DriveSurfaceClass::Nonmonotonic;
    }
    let restoring = crossings
        .iter()
        .filter(|(_, slope)| *slope < 0.0)
        .count();
    let unstable = crossings
        .iter()
        .filter(|(_, slope)| *slope > 0.0)
        .count();
    if restoring == 1 && unstable == 0 {
        // Require at least two positive below and two negative above the crossing.
        let r_star = crossings[0].0;
        let below_pos = ordered
            .iter()
            .filter(|s| s.radius < r_star - eps && s.g_r > eps)
            .count();
        let above_neg = ordered
            .iter()
            .filter(|s| s.radius > r_star + eps && s.g_r < -eps)
            .count();
        if below_pos >= 2 && above_neg >= 2 {
            return DriveSurfaceClass::RestoringZeroCrossing;
        }
        return DriveSurfaceClass::NeutralBand;
    }
    if unstable >= 1 && restoring == 0 {
        return DriveSurfaceClass::UnstableZeroCrossing;
    }
    DriveSurfaceClass::Nonmonotonic
}

pub fn log_elasticity(y_hi: f64, y_lo: f64, x_hi: f64, x_lo: f64) -> f64 {
    let lx = (x_hi.max(1e-18)).ln() - (x_lo.max(1e-18)).ln();
    if lx.abs() < 1e-18 {
        return 0.0;
    }
    ((y_hi.max(1e-18)).ln() - (y_lo.max(1e-18)).ln()) / lx
}

pub fn classify_resource_causality(
    eps_a_g: f64,
    eps_a_l: f64,
    eps_c_g: f64,
    l_baseline: f64,
    g_hi: f64,
    g_lo: f64,
) -> Vec<ResourceCausalityClass> {
    let mut out = Vec::new();
    if l_baseline.abs() <= D060_DRIVE_EPS {
        out.push(ResourceCausalityClass::NoStructuralMaintenanceLoss);
    }
    if eps_a_g.abs() < 0.05 && eps_c_g.abs() < 0.05 {
        out.push(ResourceCausalityClass::StructuralSynthesisResourceInsensitive);
    } else if eps_a_g > 0.2 || eps_c_g > 0.2 {
        out.push(ResourceCausalityClass::StructuralSynthesisResourceSensitive);
    }
    if eps_a_l.abs() > 0.2 {
        out.push(ResourceCausalityClass::StructuralLossResourceSensitive);
    }
    let span = (g_hi / g_lo.max(1e-18)).max(g_lo / g_hi.max(1e-18));
    if span < 1.05 && g_hi > D060_DRIVE_EPS {
        out.push(ResourceCausalityClass::StructuralResponseSaturated);
    }
    if out.is_empty() {
        out.push(ResourceCausalityClass::StructuralResponseDisconnected);
    }
    out
}

/// Diagnose the D-059 neutral size manifold.
///
/// `analytic_drive` uses measured chemistry on prescribed disks.
/// `coupled_neutral` is true when sealed/coupled dR/dt is flat (D-059 NEUTRAL_SIZE_MANIFOLD).
pub fn select_neutrality_cause(
    analytic_drive: DriveSurfaceClass,
    causality: &[ResourceCausalityClass],
    ledger_closes: bool,
    geometry_ok: bool,
    g_mean: f64,
    l_mean: f64,
    net_mean: f64,
    relaxation_cancels: bool,
    coupled_neutral: bool,
) -> NeutralityCause {
    if !ledger_closes {
        return NeutralityCause::StructuralNeutralityUnresolved;
    }
    if !geometry_ok {
        return NeutralityCause::StructuralGeometryCouplingDefect;
    }
    if relaxation_cancels {
        return NeutralityCause::NumericalRelaxationCancelsBiology;
    }
    // Analytic restoring with coupled neutral ⇒ productive flow fails to move radius.
    if coupled_neutral
        && matches!(
            analytic_drive,
            DriveSurfaceClass::RestoringZeroCrossing
                | DriveSurfaceClass::PositiveAllRadii
                | DriveSurfaceClass::NegativeAllRadii
                | DriveSurfaceClass::Nonmonotonic
        )
        && net_mean.abs() > D060_DRIVE_EPS
    {
        return NeutralityCause::StructuralGeometryCouplingDefect;
    }

    let synth_insensitive = causality
        .iter()
        .any(|c| *c == ResourceCausalityClass::StructuralSynthesisResourceInsensitive);
    let saturated = causality
        .iter()
        .any(|c| *c == ResourceCausalityClass::StructuralResponseSaturated);
    let loss_a_insensitive = !causality
        .iter()
        .any(|c| *c == ResourceCausalityClass::StructuralLossResourceSensitive)
        && l_mean > D060_DRIVE_EPS;
    let synth_sensitive = causality
        .iter()
        .any(|c| *c == ResourceCausalityClass::StructuralSynthesisResourceSensitive);

    let mut causes = Vec::new();
    if g_mean.abs() <= D060_DRIVE_EPS && l_mean.abs() <= D060_DRIVE_EPS {
        causes.push(NeutralityCause::StructuralGainAndLossBothZero);
    }
    if net_mean.abs() <= D060_DRIVE_EPS
        && g_mean > D060_DRIVE_EPS
        && l_mean > D060_DRIVE_EPS
        && (g_mean - l_mean).abs() / g_mean.max(1e-18) < 0.05
    {
        causes.push(NeutralityCause::StructuralGainLossEqualConstant);
    }
    if matches!(
        analytic_drive,
        DriveSurfaceClass::NeutralBand | DriveSurfaceClass::ZeroAllRadii
    ) && g_mean > D060_DRIVE_EPS
        && l_mean > D060_DRIVE_EPS
        && net_mean.abs() <= (g_mean * 0.1).max(D060_DRIVE_EPS)
    {
        causes.push(NeutralityCause::StructuralGainLossEqualScaling);
    }
    if synth_insensitive {
        causes.push(NeutralityCause::StructuralSynthesisAInsensitive);
    }
    // Existing decay is A-independent: missing resource-coupled maintenance when
    // synthesis responds to A but loss does not, and the coupled manifold is neutral.
    if loss_a_insensitive && synth_sensitive && (coupled_neutral || matches!(
        analytic_drive,
        DriveSurfaceClass::NeutralBand
            | DriveSurfaceClass::ZeroAllRadii
            | DriveSurfaceClass::PositiveAllRadii
            | DriveSurfaceClass::NegativeAllRadii
    )) {
        causes.push(NeutralityCause::StructuralLossMissing);
    }
    if saturated {
        causes.push(NeutralityCause::StructuralResponseSaturated);
    }
    causes.sort_by_key(|c| c.as_str());
    causes.dedup();
    match causes.len() {
        0 => {
            if coupled_neutral {
                NeutralityCause::StructuralNeutralityUnresolved
            } else if matches!(analytic_drive, DriveSurfaceClass::RestoringZeroCrossing) {
                // Analytic restoring and coupled not forced-neutral: no neutrality to fix.
                NeutralityCause::StructuralNeutralityUnresolved
            } else {
                NeutralityCause::StructuralNeutralityUnresolved
            }
        }
        1 => causes[0],
        _ => {
            if causes.contains(&NeutralityCause::StructuralLossMissing)
                && (causes.contains(&NeutralityCause::StructuralGainLossEqualScaling)
                    || causes.contains(&NeutralityCause::StructuralGainLossEqualConstant)
                    || synth_sensitive)
            {
                // Prefer single primary when loss missing is the actionable lever.
                NeutralityCause::StructuralLossMissing
            } else {
                NeutralityCause::MultipleStructuralCauses
            }
        }
    }
}

pub fn candidates_justified_by_cause(cause: NeutralityCause) -> Vec<StructuralCandidateId> {
    let mut out = vec![StructuralCandidateId::AExisting];
    match cause {
        NeutralityCause::StructuralSynthesisAInsensitive
        | NeutralityCause::StructuralResponseSaturated => {
            out.push(StructuralCandidateId::BCorrectedASynthesis);
        }
        NeutralityCause::StructuralLossMissing => {
            out.push(StructuralCandidateId::CLocalMaintenanceLoss);
        }
        NeutralityCause::MultipleStructuralCauses => {
            out.push(StructuralCandidateId::BCorrectedASynthesis);
            out.push(StructuralCandidateId::CLocalMaintenanceLoss);
            out.push(StructuralCandidateId::DBoundedSynthesisPlusMaintenance);
        }
        NeutralityCause::StructuralGainLossEqualScaling
        | NeutralityCause::StructuralGainLossEqualConstant => {
            out.push(StructuralCandidateId::CLocalMaintenanceLoss);
        }
        NeutralityCause::StructuralGeometryCouplingDefect
        | NeutralityCause::NumericalRelaxationCancelsBiology => {
            // No new kinetic candidate — repair geometry/execution instead.
        }
        NeutralityCause::StructuralGainAndLossBothZero
        | NeutralityCause::StructuralNeutralityUnresolved => {}
    }
    // Directive: evaluate at most three candidates total.
    out.truncate(3);
    out
}

/// Qualify Candidate A from its endogenous drive surface (no artificial target fit).
pub fn qualify_existing_from_drive(drive: DriveSurfaceClass, crossing: Option<(f64, f64)>) -> bool {
    matches!(drive, DriveSurfaceClass::RestoringZeroCrossing) && crossing.is_some()
}

pub fn candidate_forbids_radius_variable(source: &str) -> bool {
    let banned = [
        "radius",
        "r_star",
        "r_eq",
        "target_size",
        "organism_size",
        "viability",
        "health",
        "chi_n",
        "chi_f",
        "resource_sufficiency",
    ];
    let lower = source.to_ascii_lowercase();
    !banned.iter().any(|b| lower.contains(b))
}

pub fn restoring_frontier_ok(
    samples: &[(f64, f64)],
    r_star: f64,
    slope: f64,
    eps: f64,
) -> bool {
    if !(slope < 0.0) || !r_star.is_finite() {
        return false;
    }
    let below_pos = samples
        .iter()
        .filter(|(r, g)| *r < r_star - eps && *g > eps)
        .count();
    let above_neg = samples
        .iter()
        .filter(|(r, g)| *r > r_star + eps && *g < -eps)
        .count();
    below_pos >= 2 && above_neg >= 2
}

pub fn find_restoring_crossing(samples: &[(f64, f64)], eps: f64) -> Option<(f64, f64)> {
    let mut ordered = samples.to_vec();
    ordered.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for w in ordered.windows(2) {
        let (r0, g0) = w[0];
        let (r1, g1) = w[1];
        if g0.abs() <= eps && g1.abs() <= eps {
            continue;
        }
        if g0 * g1 > 0.0 {
            continue;
        }
        let slope = (g1 - g0) / (r1 - r0).max(1e-18);
        if slope >= 0.0 {
            continue;
        }
        let r_star = if (g1 - g0).abs() > 1e-18 {
            r0 - g0 * (r1 - r0) / (g1 - g0)
        } else {
            0.5 * (r0 + r1)
        };
        if restoring_frontier_ok(&ordered, r_star, slope, eps) {
            return Some((r_star, slope));
        }
    }
    None
}

/// Fit Candidate C maintenance parameters so net drive approximates a restoring target.
/// Target: g_net(R) ≈ κ (R★ − R) with R★ in [6,14] using resource-proxy A(R)=A0*(R0/R)^p.
pub fn fit_candidate_c_params(
    radii: &[f64],
    a_of_r: &[f64],
    c_mean: f64,
    params: &SimParams,
    r_star_target: f64,
) -> Option<CandidateParams> {
    if radii.len() != a_of_r.len() || radii.len() < 3 {
        return None;
    }
    // Grid search over positive half-saturations / rates.
    let k_a_grid = [0.01, 0.05, 0.1, 0.2, 0.5, 1.0];
    let k_m_grid = [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25];
    let mut best: Option<(f64, CandidateParams)> = None;
    for &k_a_m in &k_a_grid {
        for &k_phi_m in &k_m_grid {
            let cand = CandidateParams {
                k_a_phi: 0.0,
                k_a_m,
                k_phi_m,
            };
            let mut err = 0.0;
            let mut sign_ok = 0usize;
            for (i, &r) in radii.iter().enumerate() {
                let (g, l, _) = integrate_candidate_rates(
                    StructuralCandidateId::CLocalMaintenanceLoss,
                    r,
                    a_of_r[i],
                    c_mean,
                    params,
                    cand,
                );
                let g_r = g_r_from_net(g - l, r);
                let target = 0.01 * (r_star_target - r);
                err += (g_r - target).powi(2);
                if (r < r_star_target && g_r > 0.0) || (r > r_star_target && g_r < 0.0) {
                    sign_ok += 1;
                }
            }
            if sign_ok * 2 < radii.len() {
                continue;
            }
            if best.map(|(e, _)| err < e).unwrap_or(true) {
                best = Some((err, cand));
            }
        }
    }
    best.map(|(_, p)| p)
}

pub fn fit_candidate_b_params(
    radii: &[f64],
    a_of_r: &[f64],
    c_mean: f64,
    params: &SimParams,
    r_star_target: f64,
) -> Option<CandidateParams> {
    if radii.len() != a_of_r.len() || radii.len() < 3 {
        return None;
    }
    let k_a_grid = [0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0];
    let mut best: Option<(f64, CandidateParams)> = None;
    for &k_a_phi in &k_a_grid {
        let cand = CandidateParams {
            k_a_phi,
            k_a_m: 0.0,
            k_phi_m: 0.0,
        };
        let mut err = 0.0;
        let mut sign_ok = 0usize;
        for (i, &r) in radii.iter().enumerate() {
            let (g, l, _) = integrate_candidate_rates(
                StructuralCandidateId::BCorrectedASynthesis,
                r,
                a_of_r[i],
                c_mean,
                params,
                cand,
            );
            let g_r = g_r_from_net(g - l, r);
            let target = 0.01 * (r_star_target - r);
            err += (g_r - target).powi(2);
            if (r < r_star_target && g_r > 0.0) || (r > r_star_target && g_r < 0.0) {
                sign_ok += 1;
            }
        }
        if sign_ok * 2 < radii.len() {
            continue;
        }
        if best.map(|(e, _)| err < e).unwrap_or(true) {
            best = Some((err, cand));
        }
    }
    best.map(|(_, p)| p)
}

pub fn fit_candidate_d_params(
    radii: &[f64],
    a_of_r: &[f64],
    c_mean: f64,
    params: &SimParams,
    r_star_target: f64,
) -> Option<CandidateParams> {
    if radii.len() != a_of_r.len() || radii.len() < 3 {
        return None;
    }
    let k_a_grid = [0.05, 0.1, 0.2, 0.5];
    let k_am_grid = [0.05, 0.1, 0.2, 0.5];
    let k_m_grid = [0.005, 0.01, 0.025, 0.05];
    let mut best: Option<(f64, CandidateParams)> = None;
    for &k_a_phi in &k_a_grid {
        for &k_a_m in &k_am_grid {
            for &k_phi_m in &k_m_grid {
                let cand = CandidateParams {
                    k_a_phi,
                    k_a_m,
                    k_phi_m,
                };
                let mut err = 0.0;
                let mut sign_ok = 0usize;
                for (i, &r) in radii.iter().enumerate() {
                    let (g, l, _) = integrate_candidate_rates(
                        StructuralCandidateId::DBoundedSynthesisPlusMaintenance,
                        r,
                        a_of_r[i],
                        c_mean,
                        params,
                        cand,
                    );
                    let g_r = g_r_from_net(g - l, r);
                    let target = 0.01 * (r_star_target - r);
                    err += (g_r - target).powi(2);
                    if (r < r_star_target && g_r > 0.0) || (r > r_star_target && g_r < 0.0) {
                        sign_ok += 1;
                    }
                }
                if sign_ok * 2 < radii.len() {
                    continue;
                }
                if best.map(|(e, _)| err < e).unwrap_or(true) {
                    best = Some((err, cand));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

pub fn holdout_metrics(
    candidate: StructuralCandidateId,
    params: &SimParams,
    cand: CandidateParams,
    holdout_r: &[f64],
    holdout_a: &[f64],
    c_mean: f64,
    r_star: f64,
) -> (f64, f64, f64) {
    // returns (sign_accuracy, median_rel_err, max_rel_err)
    let mut signs_ok = 0usize;
    let mut rel_errs = Vec::new();
    for (i, &r) in holdout_r.iter().enumerate() {
        let a = holdout_a.get(i).copied().unwrap_or(0.0);
        let (g, l, _) = integrate_candidate_rates(candidate, r, a, c_mean, params, cand);
        let g_r = g_r_from_net(g - l, r);
        let target = 0.01 * (r_star - r);
        let want_pos = r < r_star;
        if (want_pos && g_r > 0.0) || (!want_pos && g_r < 0.0) || target.abs() < D060_DRIVE_EPS {
            signs_ok += 1;
        }
        let denom = target.abs().max(1e-6);
        rel_errs.push(((g_r - target).abs()) / denom);
    }
    rel_errs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = holdout_r.len().max(1);
    let sign_acc = signs_ok as f64 / n as f64;
    let median = rel_errs.get(n / 2).copied().unwrap_or(f64::INFINITY);
    let max_e = rel_errs.last().copied().unwrap_or(f64::INFINITY);
    (sign_acc, median, max_e)
}

pub fn qualify_candidate_params(
    cand: CandidateParams,
    sign_acc: f64,
    median_err: f64,
    max_err: f64,
    bootstrap_spread: f64,
    loo_spread: f64,
    no_growth_without_a: bool,
    no_growth_without_c_when_required: bool,
) -> bool {
    cand.all_positive_finite()
        && sign_acc + 1e-12 >= D060_HOLDOUT_SIGN_ACC
        && median_err <= D060_HOLDOUT_MEDIAN_ERR
        && max_err <= D060_HOLDOUT_MAX_ERR
        && bootstrap_spread <= D060_BOOTSTRAP_SPREAD_MAX
        && loo_spread <= D060_LOO_SPREAD_MAX
        && no_growth_without_a
        && no_growth_without_c_when_required
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteEvidence060 {
    pub workspace_isolated: bool,
    pub d059_route_l_reproduced: bool,
    pub ledger_ok: bool,
    pub geometry_ok: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub foundational_ok: bool,
    pub causality_ok: bool,
    pub existing_restoring_qualified: bool,
    pub geometry_execution_defect: bool,
    pub synthesis_candidate_qualified: bool,
    pub maintenance_candidate_qualified: bool,
    pub combined_candidate_qualified: bool,
    pub size_restored_metabolism_fail: bool,
    pub loss_stoichiometry_unresolved: bool,
    pub no_local_law: bool,
}

pub fn select_route(ev: RouteEvidence060) -> (D060Route, D060PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D060Route::I,
            D060PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d059_route_l_reproduced {
        return (
            D060Route::I,
            D060PrimaryConclusion::D059RouteLNotReproduced,
        );
    }
    if !ev.ledger_ok {
        return (
            D060Route::I,
            D060PrimaryConclusion::StructuralLedgerFailure,
        );
    }
    if !ev.geometry_ok {
        return (
            D060Route::I,
            D060PrimaryConclusion::StructureGeometryMappingDefect,
        );
    }
    if ev.loss_stoichiometry_unresolved {
        return (
            D060Route::I,
            D060PrimaryConclusion::StructuralLossStoichiometryUnresolved,
        );
    }
    if !ev.accounting_ok {
        return (D060Route::I, D060PrimaryConclusion::AccountingFailure);
    }
    if !ev.numerical_ok {
        return (D060Route::I, D060PrimaryConclusion::NumericalFailure);
    }
    if !ev.foundational_ok {
        return (D060Route::I, D060PrimaryConclusion::FoundationalRegression);
    }
    if ev.geometry_execution_defect {
        return (D060Route::G, D060Route::G.conclusion());
    }
    if ev.existing_restoring_qualified && ev.causality_ok {
        return (D060Route::E, D060Route::E.conclusion());
    }
    if ev.size_restored_metabolism_fail {
        return (
            D060Route::I,
            D060PrimaryConclusion::SizeRestoredMetabolismNotQualified,
        );
    }
    if ev.combined_candidate_qualified && ev.causality_ok {
        return (D060Route::C, D060Route::C.conclusion());
    }
    if ev.maintenance_candidate_qualified && ev.causality_ok {
        return (D060Route::M, D060Route::M.conclusion());
    }
    if ev.synthesis_candidate_qualified && ev.causality_ok {
        return (D060Route::S, D060Route::S.conclusion());
    }
    if !ev.causality_ok
        && (ev.synthesis_candidate_qualified
            || ev.maintenance_candidate_qualified
            || ev.combined_candidate_qualified)
    {
        return (
            D060Route::I,
            D060PrimaryConclusion::StructuralFeedbackCausalityFailure,
        );
    }
    if ev.no_local_law {
        return (D060Route::N, D060Route::N.conclusion());
    }
    (D060Route::I, D060Route::I.conclusion())
}

pub fn d059_route_l_reproduced(
    primary: &str,
    restoring: &str,
    best_k_t: f64,
    contiguous_viable: bool,
    p_m: f64,
    p_t: f64,
) -> bool {
    primary == D060_D059_CONCLUSION
        && restoring == D060_D059_RESTORING
        && contiguous_viable
        && (best_k_t - D060_FROZEN_KT).abs() <= 1e-9
        && (p_m - 2.0).abs() < 0.05
        && (p_t - 1.0).abs() < 0.05
}

/// Synthetic geometry mapping checks: mass gain/loss moves equivalent radius.
pub fn geometry_mapping_synthetic_ok(eps: f64) -> bool {
    let cell = DX * DX;
    let mut phi = vec![0.0; GRID_WIDTH * GRID_HEIGHT];
    let cx = (GRID_WIDTH as f64) * 0.5;
    let cy = (GRID_HEIGHT as f64) * 0.5;
    let r0 = 10.0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            phi[idx] = circular_phi(x as f64 + 0.5, y as f64 + 0.5, cx, cy, r0);
        }
    }
    let a0 = interior_area_from_phi(&phi, cell);
    let req0 = equivalent_radius_from_area(a0);
    if (req0 - r0).abs() > 0.5 {
        return false;
    }
    // Add structural mass in a rim band just inside interface.
    let mut phi_gain = phi.clone();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if (dist - r0).abs() < 1.5 {
                phi_gain[idx] = (phi_gain[idx] + 0.2).min(1.0);
            }
        }
    }
    let a_gain = interior_area_from_phi(&phi_gain, cell);
    let r_gain = equivalent_radius_from_area(a_gain);
    // Remove mass.
    let mut phi_loss = phi.clone();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if (dist - r0).abs() < 1.5 {
                phi_loss[idx] = (phi_loss[idx] - 0.2).max(0.0);
            }
        }
    }
    let a_loss = interior_area_from_phi(&phi_loss, cell);
    let r_loss = equivalent_radius_from_area(a_loss);
    let up = r_gain > req0 + eps;
    let down = r_loss < req0 - eps;
    let sym = ((r_gain - req0) + (r_loss - req0)).abs() < 0.5 * (r_gain - r_loss).abs().max(eps);
    up && down && sym
}

/// Exposure floor used by InterfaceLimitedTurnover (documents lineage).
pub fn structural_exposure_floor() -> f64 {
    STRUCTURAL_EXPOSURE_FLOOR
}

#[cfg(test)]
mod inline_smoke {
    use super::*;

    #[test]
    fn frozen_kt_matches_sealed() {
        assert!((D060_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    }
}
