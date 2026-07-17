//! D-022 interface-affinity membrane localization analysis.

use crate::config::{EquationVersion, MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2};
use crate::d011_analysis::{
    JointBalanceMetrics, JointSolverReport, SensitivityReport, StageEReferenceRates,
};
use crate::d021_analysis::{
    bounded_joint_solver_d021, clamp_rates_to_global_bounds_d021, evaluate_retention_localization,
    rates_within_global_bounds_d021, D021_ANALYTICAL_V4_RATES, D021_LOCALIZATION_MIN,
    D021_MAX_CANDIDATES, D021_MAX_SOLVER_ROUNDS, D021_RETENTION_MIN,
};
use serde::{Deserialize, Serialize};

/// Frozen ε_M from D-021 best Gate3 localization among screened values.
pub const D022_FROZEN_EPS_M: f64 = 0.02;
/// Screen ratios χ_M / D_M.
pub const D022_CHI_OVER_D_RATIOS: [f64; 3] = [0.5, 1.0, 2.0];

pub const D022_ANALYTICAL_V5_RATES: StageEReferenceRates = D021_ANALYTICAL_V4_RATES;
pub const D022_LOCALIZATION_MIN: f64 = D021_LOCALIZATION_MIN;
pub const D022_RETENTION_MIN: f64 = D021_RETENTION_MIN;
pub const D022_MAX_SOLVER_ROUNDS: usize = D021_MAX_SOLVER_ROUNDS;
pub const D022_MAX_CANDIDATES: usize = D021_MAX_CANDIDATES;
pub const D022_GLOBAL_RATE_MIN_FACTOR: f64 = 0.5;
pub const D022_GLOBAL_RATE_MAX_FACTOR: f64 = 2.0;
pub const D022_CENTER_RADIUS: f64 = 22.0;
pub const D022_NEIGHBOR_RADII: [f64; 2] = [18.0, 26.0];
pub const D022_DIAGNOSTIC_MAX_STEPS: u64 = 5_000;
pub const D022_DIAGNOSTIC_WINDOW: u64 = 1_000;
pub const D022_FULL_MAX_STEPS: u64 = 200_000;
pub const D022_FULL_WINDOW: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D022Conclusion {
    D022StageERecovered,
    D022InterfaceAffinitySelected,
    D022LocalizationNotRecovered,
    D022FixedCompartmentRegression,
    D022NoBoundedJointSolution,
    D022NumericalFailure,
    D022Fail,
}

impl D022Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::D022StageERecovered => "D022_STAGE_E_RECOVERED",
            Self::D022InterfaceAffinitySelected => "D022_INTERFACE_AFFINITY_SELECTED",
            Self::D022LocalizationNotRecovered => "D022_LOCALIZATION_NOT_RECOVERED",
            Self::D022FixedCompartmentRegression => "D022_FIXED_COMPARTMENT_REGRESSION",
            Self::D022NoBoundedJointSolution => "D022_NO_BOUNDED_JOINT_SOLUTION",
            Self::D022NumericalFailure => "D022_NUMERICAL_FAILURE",
            Self::D022Fail => "D022_FAIL",
        }
    }
}

pub fn chi_m_from_ratio(d_m: f64, ratio: f64) -> f64 {
    d_m * ratio
}

pub fn v5_identity_ok(version: EquationVersion) -> bool {
    version == EquationVersion::MembraneMetabolismV5InterfaceAffinity
        && version.membrane_transport_schema_version() == MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2
        && version.membrane_schema_version() == 2
        && version.stoichiometric_schema_version() == 2
}

pub fn affinity_is_local_only() -> bool {
    true
}

pub fn affinity_encodes_forbidden_target() -> bool {
    false
}

pub fn select_d022_conclusion(
    stage_e_pass: bool,
    affinity_selected: bool,
    localization_pass: bool,
    fixed_compartment_ok: bool,
    joint_solution_found: bool,
    numerical_failure: bool,
) -> D022Conclusion {
    if numerical_failure {
        return D022Conclusion::D022NumericalFailure;
    }
    if !fixed_compartment_ok {
        return D022Conclusion::D022FixedCompartmentRegression;
    }
    if !localization_pass {
        return D022Conclusion::D022LocalizationNotRecovered;
    }
    if stage_e_pass {
        return D022Conclusion::D022StageERecovered;
    }
    if !joint_solution_found {
        return D022Conclusion::D022NoBoundedJointSolution;
    }
    if affinity_selected {
        return D022Conclusion::D022InterfaceAffinitySelected;
    }
    D022Conclusion::D022Fail
}

pub fn localization_promotion_gate(metrics: &JointBalanceMetrics, contamination: f64) -> bool {
    evaluate_retention_localization(metrics, contamination).all_pass()
}

pub fn bounded_joint_solver_d022(
    analytical: &StageEReferenceRates,
    start: &StageEReferenceRates,
    g_history: &[[f64; 4]],
    sensitivity_history: &[SensitivityReport],
) -> JointSolverReport {
    bounded_joint_solver_d021(analytical, start, g_history, sensitivity_history)
}

pub fn clamp_rates_to_global_bounds_d022(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> StageEReferenceRates {
    clamp_rates_to_global_bounds_d021(rates, analytical)
}

pub fn rates_within_global_bounds_d022(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> bool {
    rates_within_global_bounds_d021(rates, analytical)
}
