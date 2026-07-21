//! Focused D-057 unit coverage: dimensions, measures, drives, route rules.

use chemistry_core::d056_analysis::{activity, rate_span_ok, relative_flux_error};
use chemistry_core::d057_analysis::*;

#[test]
fn seal_constants() {
    assert_eq!(D057_D056_TAG, "D-056-waste-coupled-resource-carrier-fail");
    assert_eq!(D057_FROZEN_D056, "D056_CARRIER_KINETICS_NOT_IDENTIFIABLE");
    assert_eq!(D057_UNRESOLVED, "WASTE_COUPLED_CARRIER_ARCHITECTURE_UNRESOLVED");
    assert_eq!(
        D057_D056_COMMIT,
        "ed6de2cb0ce78202a665ddc4335ca198ac79b625"
    );
}

#[test]
fn dimensional_ledger_flags_observer_omissions() {
    let led = dimensional_ledger();
    assert!(led.accounting_ok);
    assert!(!led.omitted_or_duplicated.is_empty());
    assert_eq!(d056_observer_face_measure_count(), 0);
    assert!(!d056_delta_matches_production());
}

#[test]
fn carrier_measures_vanish_without_s() {
    let zero = LocalMeasureInputs {
        gamma_s: 0.0,
        delta: 0.5,
        theta_s: 0.0,
        s_face: 0.0,
    };
    for kind in [
        CarrierMeasureKind::AGammaS,
        CarrierMeasureKind::BDeltaGammaS,
        CarrierMeasureKind::CDeltaThetaS,
        CarrierMeasureKind::DFaceAssignedS,
    ] {
        assert!(measure_vanishes_without_s(kind));
        assert!(local_measure(kind, zero) <= 1e-15);
    }
    let pos = LocalMeasureInputs {
        gamma_s: 2.0,
        delta: 0.25,
        theta_s: 0.8,
        s_face: 0.5,
    };
    assert!((local_measure(CarrierMeasureKind::AGammaS, pos) - 2.0).abs() < 1e-12);
    assert!((local_measure(CarrierMeasureKind::BDeltaGammaS, pos) - 0.5).abs() < 1e-12);
    assert!((local_measure(CarrierMeasureKind::CDeltaThetaS, pos) - 0.2).abs() < 1e-12);
    assert!((local_measure(CarrierMeasureKind::DFaceAssignedS, pos) - 0.5).abs() < 1e-12);
}

#[test]
fn forward_reverse_cancellation_and_classify() {
    let (f, r, n) = drive_abc_model_a(2.0, 2.0, 3.0, 0.2, 0.2, 0.1, 1.0, 1.0);
    assert!(f > r);
    assert!(n > 0.0);
    let rho = cancellation_ratio(f, r, n);
    assert!(rho > 0.0 && rho <= 1.0 + 1e-12);
    let cls = classify_drive(f, r, n, activity(3.0, 1.0));
    assert_eq!(cls, DriveClass::StrongForwardDrive);

    let (f2, r2, n2) = drive_abc_model_a(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 0.5);
    assert!(n2.abs() < 1e-12);
    assert_eq!(
        classify_drive(f2, r2, n2, 0.5),
        DriveClass::NearEquilibriumCancellation
    );
}

#[test]
fn separate_nf_and_mass_action_reversible() {
    let (f, r, n) = drive_model_b(2.0, 2.0, 3.0, 0.2, 0.2, 0.1, 1.0, 1.0, 1.0);
    let (f2, r2, n2) = drive_model_b(0.2, 0.2, 0.1, 2.0, 2.0, 3.0, 1.0, 1.0, 1.0);
    assert!((f + f2 - (r + r2)).abs() < 1e-9 || (n + n2).abs() < 1e-12);
    assert!((n + n2).abs() < 1e-12);

    let (c_f, c_r, c_n) = drive_model_c(2.0, 2.0, 3.0, 0.2, 0.2, 0.1, 1.0, 1.0, 1.0);
    let (_, _, c_n2) = drive_model_c(0.2, 0.2, 0.1, 2.0, 2.0, 3.0, 1.0, 1.0, 1.0);
    assert!((c_n + c_n2).abs() < 1e-12);
    assert!(c_f > 0.0 && c_r > 0.0);

    let (d_f, d_r, d_n) = drive_model_d(2.0, 2.0, 3.0, 0.2, 0.2, 0.1, 1.0, 1.0, 1.0);
    assert!(d_f < 1.0 && d_r < 1.0);
    assert!(d_n > 0.0);
}

#[test]
fn starvation_directionality() {
    assert!(starvation_blocks_import(0.0, 1.0, DriveModelKind::AProductSaturation));
    assert!(starvation_blocks_import(1.0, 0.0, DriveModelKind::BSeparateNf));
    assert!(!starvation_blocks_import(1.0, 1.0, DriveModelKind::CNormalizedMassAction));
}

#[test]
fn family_partition_and_route_rules() {
    assert_eq!(
        classify_family_nonportability(10.0, 1.5, 1.2, 1.1),
        FamilyNonportability::RadiusFamilyNonportable
    );
    assert_eq!(
        classify_family_nonportability(10.0, 8.0, 1.2, 1.1),
        FamilyNonportability::MultipleFamiliesNonportable
    );
    let early = primary_for_gate_failure(false, true, true, true, true);
    assert_eq!(
        early,
        Some(D057PrimaryConclusion::D056EvidenceNotReproduced)
    );
    let route = select_route(RouteEvidence {
        d056_reproduced: true,
        parameter_span_reproduced: true,
        dimensional_ok: true,
        grid_or_interface_defect: false,
        measure_identity_defect: true,
        drive_model_portable: false,
        surface_volume_limit: false,
        architecture_rejected: false,
    });
    assert_eq!(route, D057Route::M);
    assert_eq!(
        route.conclusion().as_str(),
        "D057_CARRIER_MEASURE_IDENTITY_DEFECT"
    );
}

#[test]
fn portability_helpers() {
    assert!(rate_span_ok(&[1.0, 2.0, 2.9]));
    assert!(!rate_span_ok(&[1.0, 4.0]));
    assert!((bootstrap_spread(&[1.0, 1.0, 1.0])).abs() < 1e-12);
    assert!(bootstrap_spread(&[1.0, 3.0]) >= 0.5 - 1e-12);
    assert!((loo_factor(&[0.1, 0.2]) - 2.0).abs() < 1e-12);
    let exp = scaling_exponent(&[8.0, 16.0, 32.0], &[1.0, 4.0, 16.0]).unwrap();
    assert!((exp - 2.0).abs() < 1e-6);
    assert!(surface_volume_capacity_limit(2.1, 1.0));
    assert!(!surface_volume_capacity_limit(1.0, 1.5));
}

#[test]
fn identifiability_gates_and_observer_flux() {
    let good = IdentifiabilityReport {
        measure: "M_A".into(),
        drive_model: "A".into(),
        rate_span: Some(2.0),
        bootstrap_spread: 0.2,
        loo_factor: 1.5,
        hold_median_err: 0.1,
        hold_max_err: 0.2,
        direction_ok: true,
        starve_ok: true,
        portable: true,
    };
    assert!(identifiability_passes(&good));
    let bad = IdentifiabilityReport {
        rate_span: Some(10.0),
        ..good.clone()
    };
    assert!(!identifiability_passes(&bad));
    assert!((observer_flux(2.0, 3.0, 0.5) - 3.0).abs() < 1e-12);
    assert!((relative_flux_error(1.0, 2.0) - 0.5).abs() < 1e-12);
}

#[test]
fn no_observer_feedback_constants() {
    // Diagnostic module must not authorize production schema.
    assert!(!D057_EQUATION.contains("production"));
    assert_eq!(
        D057PrimaryConclusion::WasteCoupledCarrierArchitectureRejected.as_str(),
        "D057_WASTE_COUPLED_CARRIER_ARCHITECTURE_REJECTED"
    );
}
