//! Focused D-055 unit coverage: strict Gate5/Gate8 evaluator and route helpers.

use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::*;

#[test]
fn preservation_constants() {
    assert_eq!(D055_D053_SOURCE_COMMIT, "76c0898e297b0abf04362df3e848e32c9d228b15");
    assert_eq!(D055_D053_SEALED_PRIMARY, "D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND");
    assert_eq!(D055_INFORMAL_GATE_INVALID, "D053_INFORMAL_GATE5_AND_GATE8_PASSES_INVALID");
    assert!((D055_FROZEN_M_EXT - 4.0).abs() < 1e-15);
    assert!((D055_FROZEN_M_BETA - 0.5776226504666211).abs() < 1e-15);
}

#[test]
fn admission_path_inventory_resolved() {
    let inv = d053_admission_path_inventory();
    assert!(admission_paths_resolved(&inv));
    assert!(harness_defect_demonstrated());
}

#[test]
fn shared_gate5_evaluator_fixtures() {
    assert_eq!(evaluate_gate5(&gate5_fixture_a_pass()).as_str(), "PASS");
    assert_eq!(
        evaluate_gate5(&gate5_fixture_b_resource_fail()).as_str(),
        "FAIL_RESOURCE_SUFFICIENCY"
    );
    assert_eq!(
        evaluate_gate5(&gate5_fixture_c_a_capacity_fail()).as_str(),
        "FAIL_A_CAPACITY"
    );
    assert_eq!(
        evaluate_gate5(&gate5_fixture_d_incomplete()).as_str(),
        "FAIL_INCOMPLETE_EVIDENCE"
    );
    assert_eq!(
        evaluate_gate5(&gate5_fixture_e_quick()).as_str(),
        "DIAGNOSTIC_ONLY"
    );
    assert!(evaluator_fixture_parity_ok());
}

#[test]
fn shared_gate8_rejects_informal_and_short_relax() {
    assert!(informal_gate8_fails_strict());
    let v = classify_gate8(&informal_gate8_evidence());
    assert_eq!(v, Gate8Verdict::FailResourceSufficiency);
    assert!(!v.is_pass());
    // Quick horizon never qualifies.
    let mut quick = informal_gate8_evidence();
    quick.horizon_class = HorizonClass::QuickDiagnostic;
    assert_eq!(evaluate_gate8(&quick), Gate8Verdict::DiagnosticOnly);
}

#[test]
fn fixed_dynamic_no_contradiction_on_frozen() {
    assert_eq!(
        classify_fixed_vs_dynamic(&D055_INFORMAL_GATE8_CHI, D055_FROZEN_CHI_DYNAMIC),
        "NO_FIXED_DYNAMIC_CONTRADICTION"
    );
}

#[test]
fn passive_upper_bound_and_radius_helpers() {
    assert_eq!(
        classify_passive_upper_bound(0.47, 0.47),
        PassiveUpperBoundClass::PassiveResourceDeliveryHardBoundFail
    );
    assert_eq!(
        classify_passive_upper_bound(1.05, 1.05),
        PassiveUpperBoundClass::PassiveResourceDeliveryHardBoundPass
    );
    assert_eq!(
        classify_radius_route(0, 8, true),
        RadiusRouteClass::NoViableRadiusInTestedDomain
    );
    assert_eq!(
        classify_radius_route(2, 2, true),
        RadiusRouteClass::ResourceSurfaceVolumeLimit
    );
    let _ = estimate_critical_radius_from_informal();
    let _ = flux_scaling_exponent_informal();
}

#[test]
fn stage_a_provenance_and_frontier() {
    assert_eq!(
        stage_a_nf_upper_band_provenance(),
        StageABandProvenance::EmpiricalCalibration
    );
    assert_eq!(
        classify_selectivity_frontier(false),
        SelectivityFrontierClass::PassiveSelectivityThroughputIncompatibility
    );
}

#[test]
fn route_selection_priority() {
    let (r, c) = select_route(
        false, false, false, true, false, false, false, false, false, true, false,
    );
    assert_eq!(r, D055Route::P);
    assert_eq!(
        c,
        D055PrimaryConclusion::PassiveResourceTransportArchitectureInsufficient
    );
    assert_eq!(
        c.as_str(),
        "D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT"
    );
}

#[test]
fn candidate_count_bound_still_six() {
    assert_eq!(D053_MAX_CANDIDATES, 6);
}

#[test]
fn no_production_biology_defaults_changed() {
    // Frozen diagnostic pair only; production defaults remain identity repair.
    assert_eq!(DeliveryRepairPair::BASELINE.m_ext, 1.0);
    assert_eq!(DeliveryRepairPair::BASELINE.m_beta, 1.0);
}
