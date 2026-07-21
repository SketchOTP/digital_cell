//! Focused D-060 coverage: D-059 Route L repro, structural ledger, drive surface, routes.

use chemistry_core::d060_analysis::*;

#[test]
fn seal_and_frozen_constants() {
    assert_eq!(D060_STARTING_COMMIT, "17faa2e");
    assert_eq!(D060_STARTING_TAG, "D-059-size-membrane-area-architecture-review");
    assert_eq!(D060_D059_CONCLUSION, "D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN");
    assert_eq!(D060_D059_RESTORING, "NEUTRAL_SIZE_MANIFOLD");
    assert!((D060_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
}

#[test]
fn d059_route_l_reproduction_predicate() {
    assert!(d059_route_l_reproduced(
        D060_D059_CONCLUSION,
        D060_D059_RESTORING,
        D060_FROZEN_KT,
        true,
        2.0,
        1.0,
    ));
    assert!(!d059_route_l_reproduced(
        "OTHER",
        D060_D059_RESTORING,
        D060_FROZEN_KT,
        true,
        2.0,
        1.0,
    ));
    assert!(!d059_route_l_reproduced(
        D060_D059_CONCLUSION,
        "RESTORING_SIZE_BASIN",
        D060_FROZEN_KT,
        true,
        2.0,
        1.0,
    ));
    assert!(!d059_route_l_reproduced(
        D060_D059_CONCLUSION,
        D060_D059_RESTORING,
        1.0,
        true,
        2.0,
        1.0,
    ));
}

#[test]
fn structural_ledger_closure() {
    let ledger = StructuralLedger {
        g_phi: 1.2,
        l_phi: 0.8,
        j_phi: 0.0,
        c_phi: 0.0,
        delta_observed: 0.4,
    };
    assert!(ledger.closes(D060_LEDGER_TOL));
    let bad = StructuralLedger {
        g_phi: 1.2,
        l_phi: 0.8,
        j_phi: 0.0,
        c_phi: 0.0,
        delta_observed: 0.9,
    };
    assert!(!bad.closes(D060_LEDGER_TOL));
}

#[test]
fn geometry_mapping_and_equivalent_radius() {
    assert!(geometry_mapping_synthetic_ok(D060_RADIUS_MAP_TOL));
    let r = equivalent_radius_from_area(std::f64::consts::PI * 100.0);
    assert!((r - 10.0).abs() < 1e-9);
}

#[test]
fn drive_surface_classification() {
    let positive: Vec<DriveSample> = [6.0, 10.0, 14.0]
        .iter()
        .map(|&r| DriveSample {
            radius: r,
            g_phi: 0.0,
            l_phi: 0.0,
            net_phi: 0.0,
            g_phi_per_area: 0.0,
            g_r: 0.05,
            interior_area: 1.0,
            interface_length: 1.0,
            a_mean: 0.05,
            c_mean: 0.5,
        })
        .collect();
    assert_eq!(
        classify_drive_surface(&positive, D060_DRIVE_EPS),
        DriveSurfaceClass::PositiveAllRadii
    );

    let negative: Vec<DriveSample> = [6.0, 10.0, 14.0]
        .iter()
        .map(|&r| DriveSample {
            radius: r,
            g_phi: 0.0,
            l_phi: 0.0,
            net_phi: 0.0,
            g_phi_per_area: 0.0,
            g_r: -0.05,
            interior_area: 1.0,
            interface_length: 1.0,
            a_mean: 0.05,
            c_mean: 0.5,
        })
        .collect();
    assert_eq!(
        classify_drive_surface(&negative, D060_DRIVE_EPS),
        DriveSurfaceClass::NegativeAllRadii
    );

    let neutral: Vec<DriveSample> = D060_DRIVE_RADII
        .iter()
        .map(|&r| DriveSample {
            radius: r,
            g_phi: 1.0,
            l_phi: 1.0,
            net_phi: 0.0,
            g_phi_per_area: 0.0,
            g_r: 0.0,
            interior_area: 1.0,
            interface_length: 1.0,
            a_mean: 0.05,
            c_mean: 0.5,
        })
        .collect();
    assert_eq!(
        classify_drive_surface(&neutral, D060_DRIVE_EPS),
        DriveSurfaceClass::ZeroAllRadii
    );

    let unstable: Vec<DriveSample> = [4.0, 8.0, 12.0, 16.0, 20.0]
        .iter()
        .map(|&r| DriveSample {
            radius: r,
            g_phi: 0.0,
            l_phi: 0.0,
            net_phi: 0.0,
            g_phi_per_area: 0.0,
            g_r: 0.01 * (r - 10.0),
            interior_area: 1.0,
            interface_length: 1.0,
            a_mean: 0.05,
            c_mean: 0.5,
        })
        .collect();
    assert_eq!(
        classify_drive_surface(&unstable, D060_DRIVE_EPS),
        DriveSurfaceClass::UnstableZeroCrossing
    );
}

#[test]
fn elasticity_and_resource_causality() {
    let eps = log_elasticity(2.0, 1.0, 2.0, 1.0);
    assert!((eps - 1.0).abs() < 1e-9);
    let classes = classify_resource_causality(0.5, 0.0, 0.0, 0.0, 2.0, 1.0);
    assert!(classes
        .iter()
        .any(|c| *c == ResourceCausalityClass::StructuralSynthesisResourceSensitive));
    assert!(classes
        .iter()
        .any(|c| *c == ResourceCausalityClass::NoStructuralMaintenanceLoss));
}

#[test]
fn neutrality_cause_missing_maintenance() {
    let causality = vec![
        ResourceCausalityClass::NoStructuralMaintenanceLoss,
        ResourceCausalityClass::StructuralSynthesisResourceSensitive,
    ];
    let cause = select_neutrality_cause(
        DriveSurfaceClass::NeutralBand,
        &causality,
        true,
        true,
        1.0,
        0.5,
        0.0,
        false,
        true,
    );
    assert!(matches!(
        cause,
        NeutralityCause::StructuralLossMissing | NeutralityCause::MultipleStructuralCauses
    ));

    let geometry = select_neutrality_cause(
        DriveSurfaceClass::RestoringZeroCrossing,
        &causality,
        true,
        true,
        1.0,
        0.5,
        0.5,
        false,
        true,
    );
    assert_eq!(geometry, NeutralityCause::StructuralGeometryCouplingDefect);
}

#[test]
fn candidate_locality_and_radius_prohibition() {
    let ok = "Local synthesis from activated resource with interface exposure floor.";
    assert!(candidate_forbids_radius_variable(ok));
    assert!(!candidate_forbids_radius_variable(
        "Forbidden radius target in law."
    ));
    let cands = candidates_justified_by_cause(NeutralityCause::MultipleStructuralCauses);
    assert!(cands.contains(&StructuralCandidateId::AExisting));
    assert!(cands.len() <= 3);
}

#[test]
fn qualify_thresholds_and_zero_crossing() {
    let cand = CandidateParams {
        k_a_phi: 0.1,
        k_a_m: 0.1,
        k_phi_m: 0.01,
    };
    assert!(qualify_candidate_params(cand, 0.95, 0.1, 0.2, 0.1, 0.5, true, true));
    assert!(!qualify_candidate_params(cand, 0.5, 0.1, 0.2, 0.1, 0.5, true, true));

    let samples = vec![
        (6.0, 0.05),
        (8.0, 0.02),
        (10.0, 0.0),
        (12.0, -0.02),
        (14.0, -0.05),
    ];
    let crossing = find_restoring_crossing(&samples, D060_DRIVE_EPS);
    assert!(crossing.is_some());
    let (r_star, slope) = crossing.unwrap();
    assert!(slope < 0.0);
    assert!((r_star - 10.0).abs() < 2.0);

    let unstable = vec![(6.0, -0.05), (10.0, 0.0), (14.0, 0.05)];
    assert!(find_restoring_crossing(&unstable, D060_DRIVE_EPS).is_none());
}

#[test]
fn route_selection_rules() {
    let base = RouteEvidence060 {
        workspace_isolated: true,
        d059_route_l_reproduced: true,
        ledger_ok: true,
        geometry_ok: true,
        accounting_ok: true,
        numerical_ok: true,
        foundational_ok: true,
        causality_ok: true,
        existing_restoring_qualified: false,
        geometry_execution_defect: false,
        synthesis_candidate_qualified: false,
        maintenance_candidate_qualified: false,
        combined_candidate_qualified: false,
        size_restored_metabolism_fail: false,
        loss_stoichiometry_unresolved: false,
        no_local_law: true,
    };
    let (r, c) = select_route(base);
    assert_eq!(r, D060Route::N);
    assert_eq!(c.as_str(), "D060_NO_LOCAL_STRUCTURAL_RESTORING_LAW");

    let (r_e, c_e) = select_route(RouteEvidence060 {
        existing_restoring_qualified: true,
        no_local_law: false,
        ..base
    });
    assert_eq!(r_e, D060Route::E);
    assert_eq!(c_e.as_str(), "D060_EXISTING_STRUCTURAL_FEEDBACK_QUALIFIED");

    let (r_g, _) = select_route(RouteEvidence060 {
        geometry_execution_defect: true,
        ..base
    });
    assert_eq!(r_g, D060Route::G);

    let (r_s, _) = select_route(RouteEvidence060 {
        synthesis_candidate_qualified: true,
        no_local_law: false,
        ..base
    });
    assert_eq!(r_s, D060Route::S);

    let (r_m, _) = select_route(RouteEvidence060 {
        maintenance_candidate_qualified: true,
        no_local_law: false,
        ..base
    });
    assert_eq!(r_m, D060Route::M);

    let (r_c, _) = select_route(RouteEvidence060 {
        combined_candidate_qualified: true,
        no_local_law: false,
        ..base
    });
    assert_eq!(r_c, D060Route::C);

    let (r_i, c_i) = select_route(RouteEvidence060 {
        workspace_isolated: false,
        ..base
    });
    assert_eq!(r_i, D060Route::I);
    assert_eq!(c_i.as_str(), "D060_WORKSPACE_SCOPE_NOT_ISOLATED");

    let (r_fail, c_fail) = select_route(RouteEvidence060 {
        d059_route_l_reproduced: false,
        ..base
    });
    assert_eq!(r_fail, D060Route::I);
    assert_eq!(c_fail.as_str(), "D060_D059_ROUTE_L_NOT_REPRODUCED");
}
