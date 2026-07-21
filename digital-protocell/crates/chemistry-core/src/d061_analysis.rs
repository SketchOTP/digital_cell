//! D-061 structural-constraint execution repair and size-basin analysis helpers.
//! Structural kinetics remain unchanged; this module classifies execution and outcomes.

use crate::config::StructureEvolutionMode;
use crate::d060_analysis::*;
use serde::{Deserialize, Serialize};

pub const D061_PROJECT_ID: &str = "D-061";
pub const D061_AGENT_MEMORY_ID: &str = "D-20260721-d061-structural-constraint-execution-repair";
pub const D061_STARTING_COMMIT: &str = "5e3abdf";
pub const D061_STARTING_TAG: &str = "D-060-structural-size-feedback-audit";
pub const D061_D060_CONCLUSION: &str = "D060_STRUCTURAL_GEOMETRY_EXECUTION_DEFECT";
pub const D061_FROZEN_KT: f64 = 1.4346157818803311;
pub const D061_DRIVE_RADII: &[f64] = &[
    4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0,
];
pub const D061_RADII: &[f64] = D061_DRIVE_RADII;
pub const D061_LEDGER_TOL: f64 = D060_LEDGER_TOL;
pub const D061_UPDATE_PARITY_TOL: f64 = 1e-6;
pub const D061_RADIUS_MAP_TOL: f64 = D060_RADIUS_MAP_TOL;
pub const D061_DRIVE_EPS: f64 = D060_DRIVE_EPS;
pub const D061_A_RETENTION_TARGET: f64 = 0.80;
pub const D061_C_RETENTION_TARGET: f64 = 0.80;
pub const D061_CHI_VIABLE: f64 = 1.05;
pub const D061_CHI_TARGET: f64 = D061_CHI_VIABLE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D061PrimaryConclusion {
    ExistingStructuralRestoringBasinQualified,
    UnmodifiedStructuralRunawayGrowth,
    UnmodifiedStructuralRunawayCollapse,
    SizeRestoredMetabolismNotQualified,
    NoExistingStructuralRestoringBasin,
    StructureExecutionRepairFailure,
    DynamicStructureRevalidationInconclusive,
    D060ExecutionDefectNotReproduced,
    StructureModeSemanticsUnresolved,
    StructureModeImplementationFailure,
    StructuralUpdateParityFailure,
    DynamicGeometryResponseFailure,
    FixedGeometryRegression,
    DynamicStructureCausalityFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D061PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExistingStructuralRestoringBasinQualified => {
                "D061_EXISTING_STRUCTURAL_RESTORING_BASIN_QUALIFIED"
            }
            Self::UnmodifiedStructuralRunawayGrowth => "D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH",
            Self::UnmodifiedStructuralRunawayCollapse => {
                "D061_UNMODIFIED_STRUCTURAL_RUNAWAY_COLLAPSE"
            }
            Self::SizeRestoredMetabolismNotQualified => {
                "D061_SIZE_RESTORED_METABOLISM_NOT_QUALIFIED"
            }
            Self::NoExistingStructuralRestoringBasin => {
                "D061_NO_EXISTING_STRUCTURAL_RESTORING_BASIN"
            }
            Self::StructureExecutionRepairFailure => "D061_STRUCTURE_EXECUTION_REPAIR_FAILURE",
            Self::DynamicStructureRevalidationInconclusive => {
                "D061_DYNAMIC_STRUCTURE_REVALIDATION_INCONCLUSIVE"
            }
            Self::D060ExecutionDefectNotReproduced => "D061_D060_EXECUTION_DEFECT_NOT_REPRODUCED",
            Self::StructureModeSemanticsUnresolved => "D061_STRUCTURE_MODE_SEMANTICS_UNRESOLVED",
            Self::StructureModeImplementationFailure => {
                "D061_STRUCTURE_MODE_IMPLEMENTATION_FAILURE"
            }
            Self::StructuralUpdateParityFailure => "D061_STRUCTURAL_UPDATE_PARITY_FAILURE",
            Self::DynamicGeometryResponseFailure => "D061_DYNAMIC_GEOMETRY_RESPONSE_FAILURE",
            Self::FixedGeometryRegression => "D061_FIXED_GEOMETRY_REGRESSION",
            Self::DynamicStructureCausalityFailure => "D061_DYNAMIC_STRUCTURE_CAUSALITY_FAILURE",
            Self::WorkspaceScopeNotIsolated => "D061_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D061_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D061_NUMERICAL_FAILURE",
            Self::Fail => "D061_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D061Route {
    E,
    G,
    C,
    M,
    N,
    X,
    I,
}

impl D061Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::E => "Route_E_existing_structural_restoring_basin",
            Self::G => "Route_G_runaway_growth",
            Self::C => "Route_C_runaway_collapse",
            Self::M => "Route_M_size_restored_metabolism_fails",
            Self::N => "Route_N_no_restoring_basin",
            Self::X => "Route_X_execution_repair_fails",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D061PrimaryConclusion {
        match self {
            Self::E => D061PrimaryConclusion::ExistingStructuralRestoringBasinQualified,
            Self::G => D061PrimaryConclusion::UnmodifiedStructuralRunawayGrowth,
            Self::C => D061PrimaryConclusion::UnmodifiedStructuralRunawayCollapse,
            Self::M => D061PrimaryConclusion::SizeRestoredMetabolismNotQualified,
            Self::N => D061PrimaryConclusion::NoExistingStructuralRestoringBasin,
            Self::X => D061PrimaryConclusion::StructureExecutionRepairFailure,
            Self::I => D061PrimaryConclusion::DynamicStructureRevalidationInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CorrectedDriveClass {
    PositiveAllRadii,
    NegativeAllRadii,
    RestoringZeroCrossing,
    UnstableZeroCrossing,
    Nonmonotonic,
    NeutralAfterRepair,
    NumericallyUnresolved,
}

impl CorrectedDriveClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PositiveAllRadii => "POSITIVE_ALL_RADII",
            Self::NegativeAllRadii => "NEGATIVE_ALL_RADII",
            Self::RestoringZeroCrossing => "RESTORING_ZERO_CROSSING",
            Self::UnstableZeroCrossing => "UNSTABLE_ZERO_CROSSING",
            Self::Nonmonotonic => "NONMONOTONIC",
            Self::NeutralAfterRepair => "NEUTRAL_AFTER_REPAIR",
            Self::NumericallyUnresolved => "NUMERICALLY_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionRepairDisposition {
    Qualified,
    Rejected,
}

impl ExecutionRepairDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Qualified => "D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED",
            Self::Rejected => "D061_STRUCTURE_EXECUTION_REPAIR_REJECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PathGeometryClass {
    FixedGeometry,
    DynamicOrganism,
}

impl PathGeometryClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixedGeometry => "FIXED_GEOMETRY",
            Self::DynamicOrganism => "DYNAMIC_ORGANISM",
        }
    }
}

/// Gate 1 inventory row for one caller/configuration path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintPathRecord {
    pub caller: String,
    pub experiment: String,
    pub expected_geometry_behavior: String,
    pub current_behavior: String,
    pub expected_dynamic: bool,
    pub apply_phi: bool,
    pub should_evolve_phi: bool,
    pub scientific_production: bool,
    pub geometry_class: PathGeometryClass,
}

/// Classify the behavior actually selected by a constraint path.
pub const fn classify_constraint_path(
    expected_dynamic: bool,
    apply_phi: bool,
) -> PathGeometryClass {
    match (expected_dynamic, apply_phi) {
        (true, true) | (false, true) => PathGeometryClass::DynamicOrganism,
        (true, false) | (false, false) => PathGeometryClass::FixedGeometry,
    }
}

pub const fn constraint_path_semantics_match(expected_dynamic: bool, apply_phi: bool) -> bool {
    expected_dynamic == apply_phi
}

pub const fn d060_defect_reproduced(
    analytic_positive_all: bool,
    coupled_dr_all_near_zero: bool,
    apply_phi_false: bool,
) -> bool {
    analytic_positive_all && coupled_dr_all_near_zero && apply_phi_false
}

/// Classify measured `(radius, dR/dt)` samples after dynamic execution repair.
pub fn classify_corrected_drive(samples: &[(f64, f64)], eps: f64) -> CorrectedDriveClass {
    if samples.len() < 3
        || !eps.is_finite()
        || eps < 0.0
        || samples
            .iter()
            .any(|(radius, dr_dt)| !radius.is_finite() || !dr_dt.is_finite())
    {
        return CorrectedDriveClass::NumericallyUnresolved;
    }

    let mut ordered = samples.to_vec();
    ordered.sort_by(|a, b| a.0.total_cmp(&b.0));
    if ordered.windows(2).any(|w| w[1].0 <= w[0].0) {
        return CorrectedDriveClass::NumericallyUnresolved;
    }

    let signs: Vec<i8> = ordered
        .iter()
        .map(|(_, dr_dt)| {
            if *dr_dt > eps {
                1
            } else if *dr_dt < -eps {
                -1
            } else {
                0
            }
        })
        .collect();
    if signs.iter().all(|sign| *sign == 0) {
        return CorrectedDriveClass::NeutralAfterRepair;
    }
    if signs.iter().all(|sign| *sign >= 0) {
        return CorrectedDriveClass::PositiveAllRadii;
    }
    if signs.iter().all(|sign| *sign <= 0) {
        return CorrectedDriveClass::NegativeAllRadii;
    }
    if detect_restoring_crossing(&ordered, eps).is_some() {
        return CorrectedDriveClass::RestoringZeroCrossing;
    }

    let nonzero_signs: Vec<i8> = signs.into_iter().filter(|sign| *sign != 0).collect();
    let unstable = nonzero_signs
        .windows(2)
        .any(|window| window[0] < 0 && window[1] > 0);
    if unstable {
        CorrectedDriveClass::UnstableZeroCrossing
    } else {
        CorrectedDriveClass::Nonmonotonic
    }
}

pub fn detect_restoring_crossing(samples: &[(f64, f64)], eps: f64) -> Option<(f64, f64)> {
    find_restoring_crossing(samples, eps)
}

pub fn classify_runaway_growth(radius_deltas: &[f64], eps: f64) -> bool {
    if radius_deltas.len() < 3
        || !eps.is_finite()
        || eps < 0.0
        || radius_deltas.iter().any(|delta| !delta.is_finite())
    {
        return false;
    }
    let growing = radius_deltas.iter().filter(|delta| **delta > eps).count();
    growing * 5 >= radius_deltas.len() * 4
}

pub fn classify_runaway_collapse(radius_deltas: &[f64], eps: f64) -> bool {
    if radius_deltas.len() < 3
        || !eps.is_finite()
        || eps < 0.0
        || radius_deltas.iter().any(|delta| !delta.is_finite())
    {
        return false;
    }
    let collapsing = radius_deltas.iter().filter(|delta| **delta < -eps).count();
    collapsing * 5 >= radius_deltas.len() * 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEvidence061 {
    pub workspace_isolated: bool,
    pub d060_defect_reproduced: bool,
    pub mode_semantics_ok: bool,
    pub mode_implementation_ok: bool,
    pub update_parity_ok: bool,
    pub synthetic_geometry_ok: bool,
    pub fixed_geometry_regression_ok: bool,
    pub causality_ok: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub restoring_basin_qualified: bool,
    pub runaway_growth: bool,
    pub runaway_collapse: bool,
    pub size_restored_metabolism_fail: bool,
    pub no_existing_restoring_basin: bool,
}

pub fn select_route(ev: RouteEvidence061) -> (D061Route, D061PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D061Route::I,
            D061PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d060_defect_reproduced {
        return (
            D061Route::I,
            D061PrimaryConclusion::D060ExecutionDefectNotReproduced,
        );
    }
    if !ev.mode_semantics_ok {
        return (
            D061Route::X,
            D061PrimaryConclusion::StructureModeSemanticsUnresolved,
        );
    }
    if !ev.mode_implementation_ok {
        return (
            D061Route::X,
            D061PrimaryConclusion::StructureModeImplementationFailure,
        );
    }
    if !ev.update_parity_ok {
        return (
            D061Route::X,
            D061PrimaryConclusion::StructuralUpdateParityFailure,
        );
    }
    if !ev.synthetic_geometry_ok {
        return (
            D061Route::X,
            D061PrimaryConclusion::DynamicGeometryResponseFailure,
        );
    }
    if !ev.fixed_geometry_regression_ok {
        return (D061Route::X, D061PrimaryConclusion::FixedGeometryRegression);
    }
    if !ev.causality_ok {
        return (
            D061Route::I,
            D061PrimaryConclusion::DynamicStructureCausalityFailure,
        );
    }
    if !ev.accounting_ok {
        return (D061Route::I, D061PrimaryConclusion::AccountingFailure);
    }
    if !ev.numerical_ok {
        return (D061Route::I, D061PrimaryConclusion::NumericalFailure);
    }
    if ev.restoring_basin_qualified {
        return (D061Route::E, D061Route::E.conclusion());
    }
    if ev.runaway_growth {
        return (D061Route::G, D061Route::G.conclusion());
    }
    if ev.runaway_collapse {
        return (D061Route::C, D061Route::C.conclusion());
    }
    if ev.size_restored_metabolism_fail {
        return (D061Route::M, D061Route::M.conclusion());
    }
    if ev.no_existing_restoring_basin {
        return (D061Route::N, D061Route::N.conclusion());
    }
    (D061Route::I, D061Route::I.conclusion())
}

pub fn structure_mode_identity_differs(
    a: StructureEvolutionMode,
    b: StructureEvolutionMode,
) -> bool {
    a != b
}

pub fn resume_rejects_structure_mode_change(
    snapshot_mode: StructureEvolutionMode,
    target_mode: StructureEvolutionMode,
) -> bool {
    crate::candidate_identity::structure_mode_resume_compatible(snapshot_mode, target_mode).is_err()
}

pub fn expected_structural_mass_delta(
    eta_phi: f64,
    xi_synthesis: f64,
    xi_decay: f64,
    j_phi: f64,
    c_phi: f64,
) -> f64 {
    eta_phi * xi_synthesis - xi_decay + j_phi + c_phi
}

pub fn structural_update_parity_ok(
    observed_delta_mass: f64,
    eta_phi: f64,
    xi_synthesis: f64,
    xi_decay: f64,
    j_phi: f64,
    c_phi: f64,
    tol: f64,
) -> bool {
    let expected = expected_structural_mass_delta(eta_phi, xi_synthesis, xi_decay, j_phi, c_phi);
    observed_delta_mass.is_finite()
        && expected.is_finite()
        && tol.is_finite()
        && tol >= 0.0
        && (observed_delta_mass - expected).abs()
            <= tol * (1.0 + observed_delta_mass.abs() + expected.abs())
}

pub fn structural_update_ledger(
    observed_delta_mass: f64,
    eta_phi: f64,
    xi_synthesis: f64,
    xi_decay: f64,
    j_phi: f64,
    c_phi: f64,
) -> StructuralLedger {
    StructuralLedger {
        g_phi: eta_phi * xi_synthesis,
        l_phi: xi_decay,
        j_phi,
        c_phi,
        delta_observed: observed_delta_mass,
    }
}
