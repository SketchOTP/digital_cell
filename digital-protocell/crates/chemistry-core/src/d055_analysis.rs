//! D-055 strict resource-gate replay and passive-architecture review helpers.
//! Diagnostic / governance only: no production biology change.

use crate::d053_analysis::{
    evaluate_gate5, evaluate_gate8, gate5_legacy_informal_admitted, Gate5Evidence, Gate5Verdict,
    Gate8Evidence, Gate8RadiusEvidence, Gate8Verdict, HorizonClass, D053_CHI_MIN,
};
use crate::d054_analysis::{
    critical_radius_from_chi_a, fixed_compartment_chi_meets_contract, scaling_exponent,
};
use serde::{Deserialize, Serialize};

pub const D055_PROJECT_ID: &str = "D-055";
pub const D055_AGENT_MEMORY_ID: &str =
    "D-20260721-d055-strict-resource-gate-passive-architecture-review";
pub const D055_D053_SOURCE_COMMIT: &str = "76c0898e297b0abf04362df3e848e32c9d228b15";
pub const D055_D053_RESULT_COMMIT: &str = "ff18c6e56daa85a4d4acb01ffd20957a0bbf1a93";
pub const D055_D053_TAG: &str = "D-053-combined-resource-delivery-fail";
pub const D055_D053_SEALED_PRIMARY: &str = "D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND";
pub const D055_D054_CONCLUSION: &str = "D054_D053_PROVENANCE_RERUN_DIVERGED";
pub const D055_V14: &str = "V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED";
pub const D055_INFORMAL_GATE_INVALID: &str = "D053_INFORMAL_GATE5_AND_GATE8_PASSES_INVALID";
pub const D055_FIXED_COMPARTMENT_REVOKED: &str = "D053_FIXED_COMPARTMENT_PASS_REVOKED";
pub const D055_FROZEN_M_EXT: f64 = 4.0;
pub const D055_FROZEN_M_BETA: f64 = 0.5776226504666211;
pub const D055_FROZEN_PI_NF: f64 = 0.50;
pub const D055_FROZEN_CHI_DYNAMIC: f64 = 0.47;
pub const D055_FROZEN_A_RET: f64 = 0.047;
/// Informal fixed-compartment χ (R16, R24, R32) — all < 1.05.
pub const D055_INFORMAL_GATE8_CHI: [f64; 3] = [0.53, 0.38, 0.29];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D055PrimaryConclusion {
    ResourceSurfaceVolumeLimit,
    StageANfUpperBandUnsupported,
    EnvironmentalResourceGeometryLimit,
    EnvironmentalResourceConcentrationLimit,
    DynamicProductiveDemandScalingFailure,
    PassiveResourceTransportArchitectureInsufficient,
    D053HarnessRepairedNoArchitectureRoute,
    ResourceArchitectureInconclusive,
    D053AdmissionPathUnresolved,
    D053EvaluatorInvarianceFailure,
    D053StrictReplayDiverged,
    D053StrictReplayConfirmedNotFound,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D055PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceSurfaceVolumeLimit => "D055_RESOURCE_SURFACE_VOLUME_LIMIT",
            Self::StageANfUpperBandUnsupported => "D055_STAGE_A_NF_UPPER_BAND_UNSUPPORTED",
            Self::EnvironmentalResourceGeometryLimit => {
                "D055_ENVIRONMENTAL_RESOURCE_GEOMETRY_LIMIT"
            }
            Self::EnvironmentalResourceConcentrationLimit => {
                "D055_ENVIRONMENTAL_RESOURCE_CONCENTRATION_LIMIT"
            }
            Self::DynamicProductiveDemandScalingFailure => {
                "D055_DYNAMIC_PRODUCTIVE_DEMAND_SCALING_FAILURE"
            }
            Self::PassiveResourceTransportArchitectureInsufficient => {
                "D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT"
            }
            Self::D053HarnessRepairedNoArchitectureRoute => {
                "D055_D053_HARNESS_REPAIRED_NO_ARCHITECTURE_ROUTE"
            }
            Self::ResourceArchitectureInconclusive => "D055_RESOURCE_ARCHITECTURE_INCONCLUSIVE",
            Self::D053AdmissionPathUnresolved => "D055_D053_ADMISSION_PATH_UNRESOLVED",
            Self::D053EvaluatorInvarianceFailure => "D055_D053_EVALUATOR_INVARIANCE_FAILURE",
            Self::D053StrictReplayDiverged => "D055_D053_STRICT_REPLAY_DIVERGED",
            Self::D053StrictReplayConfirmedNotFound => {
                "D055_D053_STRICT_REPLAY_CONFIRMED_NOT_FOUND"
            }
            Self::AccountingFailure => "D055_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D055_NUMERICAL_FAILURE",
            Self::Fail => "D055_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D055Route {
    G,
    R,
    B,
    E,
    C,
    D,
    P,
    I,
}

impl D055Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::G => "Route_G",
            Self::R => "Route_R",
            Self::B => "Route_B",
            Self::E => "Route_E",
            Self::C => "Route_C",
            Self::D => "Route_D",
            Self::P => "Route_P",
            Self::I => "Route_I",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionDefectKind {
    Gate5ChiRiseOrARiseBypass,
    Gate8ShortHorizonRelaxed,
    ReportIndependentRecompute,
    StaleArtifactOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdmissionPathRecord {
    pub path_id: String,
    pub source_commit: String,
    pub defect: Option<AdmissionDefectKind>,
    pub notes: String,
}

/// Gate 0 inventory of known D-053 admission defects.
pub fn d053_admission_path_inventory() -> Vec<AdmissionPathRecord> {
    vec![
        AdmissionPathRecord {
            path_id: "d053.rs::gate5_screen".into(),
            source_commit: D055_D053_SOURCE_COMMIT.into(),
            defect: Some(AdmissionDefectKind::Gate5ChiRiseOrARiseBypass),
            notes: "pass = capacity || a_rise || (chi_rise && a_ret>=0.5); χ improvement alone could admit"
                .into(),
        },
        AdmissionPathRecord {
            path_id: "d053.rs::gate8_fixed".into(),
            source_commit: D055_D053_SOURCE_COMMIT.into(),
            defect: Some(AdmissionDefectKind::Gate8ShortHorizonRelaxed),
            notes: "h<10000 → chi>=0.20 and a_ret>=0.15; short_horizon_relaxed=true in artifacts"
                .into(),
        },
        AdmissionPathRecord {
            path_id: "d054_analysis::gate5_candidate_admitted".into(),
            source_commit: "d7e65e8".into(),
            defect: Some(AdmissionDefectKind::Gate5ChiRiseOrARiseBypass),
            notes: "documented informal OR-admission for audit; superseded by evaluate_gate5".into(),
        },
    ]
}

pub fn admission_paths_resolved(records: &[AdmissionPathRecord]) -> bool {
    !records.is_empty()
        && records.iter().any(|r| {
            matches!(
                r.defect,
                Some(AdmissionDefectKind::Gate5ChiRiseOrARiseBypass)
            )
        })
        && records.iter().any(|r| {
            matches!(
                r.defect,
                Some(AdmissionDefectKind::Gate8ShortHorizonRelaxed)
            )
        })
}

/// Confirm informal Gate 8 χ never met contract.
pub fn informal_gate8_fails_strict() -> bool {
    D055_INFORMAL_GATE8_CHI
        .iter()
        .any(|&chi| !fixed_compartment_chi_meets_contract(chi, chi))
}

pub fn classify_fixed_vs_dynamic(
    fixed_chi: &[f64],
    dynamic_chi: f64,
) -> &'static str {
    let fixed_ok = fixed_chi
        .iter()
        .all(|&c| fixed_compartment_chi_meets_contract(c, c));
    let dyn_ok = dynamic_chi >= D053_CHI_MIN;
    if !fixed_ok && !dyn_ok {
        "NO_FIXED_DYNAMIC_CONTRADICTION"
    } else if fixed_ok != dyn_ok {
        "FIXED_DYNAMIC_ASSAY_MISMATCH"
    } else {
        "DYNAMIC_RESOURCE_DIVERGENCE"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PassiveUpperBoundClass {
    PassiveResourceDeliveryHardBoundFail,
    PassiveResourceDeliveryHardBoundPass,
}

pub fn classify_passive_upper_bound(chi_n: f64, chi_f: f64) -> PassiveUpperBoundClass {
    if chi_n >= 1.0 && chi_f >= 1.0 {
        PassiveUpperBoundClass::PassiveResourceDeliveryHardBoundPass
    } else {
        PassiveUpperBoundClass::PassiveResourceDeliveryHardBoundFail
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentRescueClass {
    EnvironmentGeometryRescue,
    EnvironmentConcentrationRescue,
    NoEnvironmentalRescue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RadiusRouteClass {
    ResourceSurfaceVolumeLimit,
    NoViableRadiusInTestedDomain,
    RadiusRouteInconclusive,
}

pub fn classify_radius_route(
    small_pass_count: usize,
    large_fail_count: usize,
    scaling_consistent: bool,
) -> RadiusRouteClass {
    if small_pass_count >= 2 && large_fail_count >= 2 && scaling_consistent {
        RadiusRouteClass::ResourceSurfaceVolumeLimit
    } else if small_pass_count == 0 {
        RadiusRouteClass::NoViableRadiusInTestedDomain
    } else {
        RadiusRouteClass::RadiusRouteInconclusive
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemandScalingClass {
    DemandDensityStable,
    PrecursorDemandGrowth,
    StructuralDemandGrowth,
    ReproductionDemandGrowth,
    MixedProductiveDemandGrowth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StageABandProvenance {
    HardScientificRequirement,
    EmpiricalCalibration,
    DesignHeuristic,
    InitialSearchBound,
    UnsupportedInheritedThreshold,
}

/// Stage A N/F upper band 0.20–0.50 originated as Stage A planar selectivity calibration band.
pub fn stage_a_nf_upper_band_provenance() -> StageABandProvenance {
    StageABandProvenance::EmpiricalCalibration
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SelectivityFrontierClass {
    PassiveSelectivityThroughputIncompatibility,
    FrontierHasViableState,
}

pub fn classify_selectivity_frontier(any_viable: bool) -> SelectivityFrontierClass {
    if any_viable {
        SelectivityFrontierClass::FrontierHasViableState
    } else {
        SelectivityFrontierClass::PassiveSelectivityThroughputIncompatibility
    }
}

/// Route selection after Phase A confirms NOT_FOUND and Phase B diagnostics complete.
pub fn select_route(
    admission_unresolved: bool,
    evaluator_invariance_fail: bool,
    strict_replay_diverged: bool,
    strict_replay_not_found: bool,
    surface_volume: bool,
    stage_a_band_unsupported: bool,
    env_geometry: bool,
    env_concentration: bool,
    demand_scaling: bool,
    passive_insufficient: bool,
    phase_b_inconclusive: bool,
) -> (D055Route, D055PrimaryConclusion) {
    if admission_unresolved {
        return (
            D055Route::I,
            D055PrimaryConclusion::D053AdmissionPathUnresolved,
        );
    }
    if evaluator_invariance_fail {
        return (
            D055Route::I,
            D055PrimaryConclusion::D053EvaluatorInvarianceFailure,
        );
    }
    if strict_replay_diverged {
        return (
            D055Route::I,
            D055PrimaryConclusion::D053StrictReplayDiverged,
        );
    }
    if !strict_replay_not_found {
        return (D055Route::I, D055PrimaryConclusion::Fail);
    }
    // Phase B routes (strict replay confirmed NOT_FOUND).
    if surface_volume {
        return (
            D055Route::R,
            D055PrimaryConclusion::ResourceSurfaceVolumeLimit,
        );
    }
    if stage_a_band_unsupported {
        return (
            D055Route::B,
            D055PrimaryConclusion::StageANfUpperBandUnsupported,
        );
    }
    if env_geometry {
        return (
            D055Route::E,
            D055PrimaryConclusion::EnvironmentalResourceGeometryLimit,
        );
    }
    if env_concentration {
        return (
            D055Route::C,
            D055PrimaryConclusion::EnvironmentalResourceConcentrationLimit,
        );
    }
    if demand_scaling {
        return (
            D055Route::D,
            D055PrimaryConclusion::DynamicProductiveDemandScalingFailure,
        );
    }
    if passive_insufficient {
        return (
            D055Route::P,
            D055PrimaryConclusion::PassiveResourceTransportArchitectureInsufficient,
        );
    }
    if phase_b_inconclusive {
        return (
            D055Route::G,
            D055PrimaryConclusion::D053HarnessRepairedNoArchitectureRoute,
        );
    }
    (
        D055Route::I,
        D055PrimaryConclusion::ResourceArchitectureInconclusive,
    )
}

/// Re-export evaluator entry points so report/replay share one module surface.
pub fn classify_gate5(ev: &Gate5Evidence) -> Gate5Verdict {
    evaluate_gate5(ev)
}

pub fn classify_gate8(ev: &Gate8Evidence) -> Gate8Verdict {
    evaluate_gate8(ev)
}

pub fn evaluator_fixture_parity_ok() -> bool {
    use crate::d053_analysis::{
        gate5_fixture_a_pass, gate5_fixture_b_resource_fail, gate5_fixture_c_a_capacity_fail,
        gate5_fixture_d_incomplete, gate5_fixture_e_quick,
    };
    classify_gate5(&gate5_fixture_a_pass()) == Gate5Verdict::Pass
        && classify_gate5(&gate5_fixture_b_resource_fail())
            == Gate5Verdict::FailResourceSufficiency
        && classify_gate5(&gate5_fixture_c_a_capacity_fail()) == Gate5Verdict::FailACapacity
        && classify_gate5(&gate5_fixture_d_incomplete()) == Gate5Verdict::FailIncompleteEvidence
        && classify_gate5(&gate5_fixture_e_quick()) == Gate5Verdict::DiagnosticOnly
}

/// Build Gate 8 evidence from informal frozen χ table (diagnostic revocation).
pub fn informal_gate8_evidence() -> Gate8Evidence {
    let radii: Vec<Gate8RadiusEvidence> = [16.0, 24.0, 32.0]
        .into_iter()
        .zip(D055_INFORMAL_GATE8_CHI)
        .map(|(radius, chi)| Gate8RadiusEvidence {
            radius,
            chi_n: chi,
            chi_f: chi,
            c_retention: 0.85,
            a_retention: 0.20,
            n_enters: true,
            f_enters: true,
            w_exits: true,
            bounded_fields: true,
            accounting_closes: true,
            influx_per_area: 1.0 / radius,
        })
        .collect();
    Gate8Evidence {
        horizon_class: HorizonClass::Full,
        radii,
    }
}

pub fn estimate_critical_radius_from_informal() -> Option<f64> {
    // χ decreases with R; extrapolate where χ=1 using R16/R32 informal points.
    critical_radius_from_chi_a(16.0, D055_INFORMAL_GATE8_CHI[0], 32.0, D055_INFORMAL_GATE8_CHI[2])
}

pub fn flux_scaling_exponent_informal() -> Option<f64> {
    // Use influx ∝ 1/R proxy from informal table.
    scaling_exponent(16.0, 1.0 / 16.0, 32.0, 1.0 / 32.0)
}

/// Prove legacy informal OR-path is distinct from strict evaluator.
pub fn harness_defect_demonstrated() -> bool {
    gate5_legacy_informal_admitted(false, false, true, 0.50)
        && !evaluate_gate5(&{
            let mut ev = crate::d053_analysis::gate5_fixture_a_pass();
            if let Some(ref mut a) = ev.analytic {
                a.chi_n = 0.53;
                a.chi_f = 0.53;
                a.final_a_retention = 0.50;
            }
            if let Some(ref mut r) = ev.restored {
                r.chi_n = 0.53;
                r.chi_f = 0.53;
                r.final_a_retention = 0.50;
            }
            ev
        })
        .admits_candidate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_and_admission_inventory() {
        assert!(evaluator_fixture_parity_ok());
        assert!(admission_paths_resolved(&d053_admission_path_inventory()));
        assert!(informal_gate8_fails_strict());
        assert!(!classify_gate8(&informal_gate8_evidence()).is_pass());
        assert!(harness_defect_demonstrated());
    }
}
