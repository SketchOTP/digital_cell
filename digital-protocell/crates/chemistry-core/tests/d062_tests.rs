//! Focused D-062 coverage: decay parity, scaling, long-horizon classes, candidates, routes.

use chemistry_core::config::StructureEvolutionMode;
use chemistry_core::d062_analysis::*;
use chemistry_core::SimParams;

#[test]
fn seal_and_frozen_constants() {
    assert_eq!(D062_STARTING_COMMIT, "1d4e2bb");
    assert_eq!(
        D062_STARTING_TAG,
        "D-061-structural-execution-size-revalidation"
    );
    assert_eq!(
        D062_D061_SCIENTIFIC,
        "D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH"
    );
    assert_eq!(
        D062_D061_EXECUTION,
        "D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED"
    );
    assert!((D062_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    assert_eq!(
        D062_AGENT_MEMORY_ID,
        "D-20260721-d062-long-horizon-structural-maintenance-decay"
    );
    assert!((exposure_floor() - 0.05).abs() < 1e-15);
    assert!(gain_equation_string().contains("k_d008_structure"));
    assert!(existing_equation_string().contains("k_structure_decay"));
}

#[test]
fn decay_to_w_and_counterfactual() {
    assert!(decay_to_w_parity(-1.2, 1.2, 1.2, D062_LEDGER_TOL));
    assert!(!decay_to_w_parity(-1.0, 1.2, 1.2, D062_LEDGER_TOL));
    assert!(counterfactual_loss_equal(0.5, 0.5, D062_LEDGER_TOL));
    assert!(!counterfactual_loss_equal(0.5, 0.7, D062_LEDGER_TOL));
}

#[test]
fn gain_loss_scaling_classes() {
    assert_eq!(
        classify_gain_loss_scaling(2.0, 2.05, 0.2),
        ScalingClass::GainAndLossVolumeMatched
    );
    assert_eq!(
        classify_gain_loss_scaling(2.5, 1.0, 0.2),
        ScalingClass::GainScalesFasterThanLoss
    );
    assert_eq!(
        classify_gain_loss_scaling(1.0, 2.0, 0.2),
        ScalingClass::LossScalesFasterThanGain
    );
    let radii = [4.0, 8.0, 12.0, 16.0];
    let values = [16.0, 64.0, 144.0, 256.0]; // ∝ R^2
    let p = fit_power_exponent(&radii, &values).unwrap();
    assert!((p - 2.0).abs() < 0.05);
}

#[test]
fn required_multiplier_and_scalar_bounds() {
    assert!((required_decay_multiplier(3.0, 1.5).unwrap() - 2.0).abs() < 1e-12);
    assert!(scalar_correction_identifiable(&[1.5, 2.0, 3.0]));
    assert!(!scalar_correction_identifiable(&[1.0, 4.0]));
    assert!((geometric_median(&[1.0, 2.0, 3.0]).unwrap() - 2.0).abs() < 1e-12);
    // Flat m_d★ across radius cannot produce a restoring sign change.
    let flat = [
        (4.0, 14.0),
        (8.0, 14.1),
        (12.0, 13.9),
        (16.0, 14.2),
        (20.0, 14.0),
        (24.0, 13.8),
    ];
    assert!(!scalar_md_allows_restoring_crossing(&flat));
    let trending = [
        (4.0, 5.0),
        (8.0, 7.0),
        (12.0, 9.0),
        (16.0, 11.0),
        (20.0, 13.0),
        (24.0, 14.5),
    ];
    assert!(scalar_md_allows_restoring_crossing(&trending));
}

#[test]
fn a_deficit_and_zero_a_behavior() {
    let params = SimParams::default();
    let loss_low = candidate_c_loss_density(1.0, 0.0, &params, 0.5, 2.0);
    let loss_high = candidate_c_loss_density(1.0, 1.0e6, &params, 0.5, 2.0);
    assert!(a_deficit_monotonic(loss_low, loss_high, 1e-12));
    let base = existing_loss_density(1.0, &params);
    // High-A approaches the irreducible baseline decay.
    assert!((loss_high - base).abs() < 1e-6 * (1.0 + base));
    assert!(loss_low <= (1.0 + 2.0) * base + 1e-12);
    assert!(zero_a_no_positive_growth(-0.1, D062_DRIVE_EPS));
    assert!(!zero_a_no_positive_growth(0.1, D062_DRIVE_EPS));
}

#[test]
fn global_md_scales_loss() {
    let params = SimParams::default();
    let base = candidate_loss_density(
        MaintenanceCandidateId::AExisting,
        0.8,
        1.0,
        &params,
        MaintenanceParams::existing(),
    );
    let scaled = candidate_loss_density(
        MaintenanceCandidateId::BGlobalDecayCalibration,
        0.8,
        1.0,
        &params,
        MaintenanceParams::global_md(2.5),
    );
    assert!((scaled - 2.5 * base).abs() < 1e-12);
}

#[test]
fn delayed_restoring_and_runaway_classification() {
    let restoring = [
        (4.0, 0.02),
        (6.0, 0.01),
        (10.0, -0.005),
        (14.0, -0.02),
        (18.0, -0.03),
    ];
    assert!(classify_delayed_restoring_basin(&restoring, D062_DRIVE_EPS));
    assert!(stable_crossing_qualified(&restoring, D062_DRIVE_EPS).is_some());
    assert!(crossing_in_supported_domain(10.0));

    let runaway = [
        (4.0, 0.02),
        (8.0, 0.015),
        (12.0, 0.01),
        (16.0, 0.008),
        (20.0, 0.005),
    ];
    let deltas = [0.5, 0.4, 0.3, 0.2, 0.1];
    assert_eq!(
        classify_baseline_horizon(&runaway, &deltas, D062_DRIVE_EPS),
        BaselineHorizonClass::ExistingStructuralPersistentRunawayGrowth
    );

    let collapse = [
        (4.0, -0.02),
        (8.0, -0.015),
        (12.0, -0.01),
        (16.0, -0.008),
        (20.0, -0.005),
    ];
    let c_deltas = [-0.5, -0.4, -0.3, -0.2, -0.1];
    assert_eq!(
        classify_baseline_horizon(&collapse, &c_deltas, D062_DRIVE_EPS),
        BaselineHorizonClass::ExistingStructuralDelayedCollapse
    );
}

#[test]
fn unstable_crossing_rejected() {
    let unstable = [(4.0, -0.02), (8.0, -0.01), (12.0, 0.01), (16.0, 0.02)];
    assert!(stable_crossing_qualified(&unstable, D062_DRIVE_EPS).is_none());
}

#[test]
fn radius_specific_parameter_rejected() {
    let params = MaintenanceParams::global_md(2.0);
    assert!(params.positive_finite());
    // Global parameter set must not encode radius.
    assert_eq!(params.m_d, 2.0);
    assert_eq!(params.k_a_m, 0.0);
}

#[test]
fn structure_mode_dispatch_preserved() {
    assert!(!StructureEvolutionMode::FixedGeometry.apply_phi());
    assert!(StructureEvolutionMode::DynamicStructure.apply_phi());
}

#[test]
fn route_selection_matrix() {
    let base = RouteEvidence062 {
        workspace_isolated: true,
        d061_reproduced: true,
        decay_parity_ok: true,
        scaling_ok: true,
        baseline_restoring: false,
        baseline_runaway: true,
        baseline_collapse: false,
        candidate_b_qualified: false,
        candidate_c_qualified: false,
        basin_qualified: false,
        metabolism_qualified: false,
        causality_ok: true,
        foundational_ok: true,
        accounting_ok: true,
        numerical_ok: true,
    };
    let (route, conclusion) = select_route(base);
    assert_eq!(route, D062Route::N);
    assert_eq!(
        conclusion.as_str(),
        "D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW"
    );

    let mut defect = base;
    defect.decay_parity_ok = false;
    assert_eq!(select_route(defect).0, D062Route::X);

    let mut restoring = base;
    restoring.baseline_runaway = false;
    restoring.baseline_restoring = true;
    restoring.basin_qualified = true;
    restoring.metabolism_qualified = true;
    assert_eq!(select_route(restoring).0, D062Route::E);

    let mut calib = base;
    calib.candidate_b_qualified = true;
    calib.basin_qualified = true;
    calib.metabolism_qualified = true;
    assert_eq!(select_route(calib).0, D062Route::K);

    let mut maint = base;
    maint.candidate_c_qualified = true;
    maint.basin_qualified = true;
    maint.metabolism_qualified = true;
    assert_eq!(select_route(maint).0, D062Route::M);

    let mut collapse = base;
    collapse.baseline_runaway = false;
    collapse.baseline_collapse = true;
    assert_eq!(select_route(collapse).0, D062Route::C);
}

#[test]
fn shadow_isolation_constants() {
    assert!((D062_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    assert_eq!(D062_DRIVE_RADII.len(), 11);
    assert_eq!(D062_TRAINING_RADII.len(), 5);
    assert_eq!(D062_HOLDOUT_RADII.len(), 6);
}
