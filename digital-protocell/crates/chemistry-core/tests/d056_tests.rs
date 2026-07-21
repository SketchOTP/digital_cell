//! Focused D-056 unit coverage: carrier law, conservation, reversibility, capacity.

use chemistry_core::d056_analysis::*;

#[test]
fn preservation_constants() {
    assert_eq!(D056_STARTING_TAG, "D-055-strict-resource-architecture-review");
    assert_eq!(D056_FROZEN_D055, "D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT");
    assert_eq!(D056_ORDINARY_PASSIVE_CLOSED, "ORDINARY_PASSIVE_RESOURCE_IMPORT_BRANCH_CLOSED");
    assert!((D056_SEALED_CHI_E - 0.9039035176168589).abs() < 1e-12);
}

#[test]
fn gate1_thermodynamic_review_passes() {
    assert!(gate1_all_pass());
    for (name, ok) in gate1_thermodynamic_checklist() {
        assert!(ok, "failed check: {name}");
    }
}

#[test]
fn carrier_flux_no_rectification_and_detailed_balance() {
    let j_eq = carrier_flux_jt(1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 1.5, 0.8, 0.8, 2.0);
    assert!(j_eq.abs() < 1e-12);
    let j_rev = carrier_flux_jt(0.1, 0.1, 0.05, 2.0, 2.0, 3.0, 1.0, 1.0, 1.0, 1.0);
    assert!(j_rev < 0.0);
}

#[test]
fn zero_controls_kill_inward_flux() {
    assert!(carrier_flux_jt(1.0, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0).abs() < 1e-15);
    assert!(carrier_flux_jt(0.0, 1.0, 2.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0) <= 0.0);
    assert!(carrier_flux_jt(1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0) <= 0.0);
    assert!(carrier_flux_jt(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0) <= 0.0);
}

#[test]
fn discrete_extent_conserves_nfw() {
    let s0 = CarrierFaceState {
        n_out: 5.0,
        f_out: 4.0,
        w_out: 1.0,
        n_in: 0.5,
        f_in: 0.5,
        w_in: 6.0,
    };
    let s1 = s0.apply_extent(1.25).unwrap();
    assert!((s1.total_n() - s0.total_n()).abs() < 1e-12);
    assert!((s1.total_f() - s0.total_f()).abs() < 1e-12);
    assert!((s1.total_w() - s0.total_w()).abs() < 1e-12);
    let s2 = s1.apply_extent(-0.7).unwrap();
    assert!((s2.total_n() - s0.total_n()).abs() < 1e-12);
}

#[test]
fn waste_capacity_helpers() {
    let delta = required_additional_influx(D056_SEALED_LN_E, D056_SEALED_JN_E);
    assert!((delta - (D056_SEALED_LN_E - D056_SEALED_JN_E)).abs() < 1e-9);
    assert!(waste_capacity_ok(delta * 1.10, delta, delta));
    assert!(!waste_capacity_ok(delta, delta, delta));
    assert!(waste_export_budget_ok(100.0, 80.0, 30.0));
    assert!(!waste_export_budget_ok(100.0, 50.0, 40.0));
}

#[test]
fn chi_with_carrier_and_rate_span() {
    let chi = chi_with_carrier(D056_SEALED_JN_E, 200.0, D056_SEALED_LN_E);
    assert!(chi > 1.05);
    assert!(rate_span_ok(&[1.0, 2.0, 2.5]));
    assert!(!rate_span_ok(&[1.0, 4.0]));
}

#[test]
fn identify_k_t_and_half_sat() {
    let k = identify_k_t(10.0, 2.0, 0.5).unwrap();
    assert!((k - 10.0).abs() < 1e-12);
    let mid = half_sat_from_range(0.25, 4.0);
    assert!((mid - 1.0).abs() < 1e-12);
}

#[test]
fn conclusion_labels_stable() {
    assert_eq!(
        D056PrimaryConclusion::WasteGradientCapacityInsufficient.as_str(),
        "D056_WASTE_GRADIENT_CAPACITY_INSUFFICIENT"
    );
    assert_eq!(
        D056PrimaryConclusion::D055PassiveBoundNotReproduced.as_str(),
        "D056_D055_PASSIVE_BOUND_NOT_REPRODUCED"
    );
}
