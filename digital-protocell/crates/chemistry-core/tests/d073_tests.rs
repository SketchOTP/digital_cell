//! D-073 mature-membrane equilibrium sufficiency audit tests.

use chemistry_core::d069_analysis::theta_eq;
use chemistry_core::d070_analysis::STAGE_E_MIN_OCCUPANCY;
use chemistry_core::d072_analysis::{D072PrimaryConclusion, REPAIR_THRESHOLD};
use chemistry_core::d073_analysis::*;

#[test]
fn equilibrium_inversion_and_target_activities() {
    // Directive Gate 0 independent check for idealized K_eq=50.
    assert!((p_required(0.90, 50.0) - 0.18).abs() < 1e-12);
    assert!((p_required(0.95, 50.0) - 0.38).abs() < 1e-12);
    // Frozen binding is α/β ≈ 50 (not a bit-exact integer).
    let k = D073_K_EQ;
    assert!((k - 50.0).abs() < 1e-9);
    assert!((p_required(0.90, k) - 0.18).abs() < 1e-9);
    assert!((p_required(0.95, k) - 0.38).abs() < 1e-9);
    assert!((p_required(0.75, k) - 0.06).abs() < 1e-9);
    assert!((p_required(STAGE_E_MIN_OCCUPANCY, k) - 0.02).abs() < 1e-9);
    let p_maint = p_required(D070_LAWFUL_MAINTENANCE_OCCUPANCY, k);
    assert!((p_maint - 2.48).abs() < 1e-6);
    for th in [0.75, 0.90, 0.95, D070_LAWFUL_MAINTENANCE_OCCUPANCY, STAGE_E_MIN_OCCUPANCY] {
        let p = p_required(th, k);
        let back = equilibrium_occupancy(p, k);
        assert!(
            (back - th).abs() < 1e-12,
            "inversion failed for θ={th}: got {back}"
        );
    }
    let rows = equilibrium_contract_rows(k);
    assert!(rows.iter().all(|r| r.inversion_ok));
    assert!(rows
        .iter()
        .any(|r| r.name == "theta_0_95" && (r.p_required - 0.38).abs() < 1e-9));
}

#[test]
fn d072_fixed_p_control_identity_not_actually_fixed() {
    let p_req = p_required(REPAIR_THRESHOLD, D073_K_EQ);
    let (conc, imposed_p, class) = d072_fixed_p_audit(D073_P_REF, p_req);
    assert!((conc - 1.0).abs() < 1e-15);
    assert!((imposed_p - 1.0).abs() < 1e-15);
    // Initial activity is analytically sufficient, but the control was not held.
    assert!(imposed_p >= p_req);
    assert_eq!(class, FixedPControlClass::NotActuallyFixed);
    assert_eq!(class.as_str(), "NOT_ACTUALLY_FIXED");
    // Explicit insufficient case.
    assert_eq!(
        classify_fixed_p_control(Some(0.05), p_req, Some(true), Some(true)),
        FixedPControlClass::TargetInsufficient
    );
    assert_eq!(
        classify_fixed_p_control(Some(0.40), p_req, Some(false), Some(true)),
        FixedPControlClass::SpatiallyIncomplete
    );
    assert_eq!(
        classify_fixed_p_control(Some(0.40), p_req, Some(true), Some(true)),
        FixedPControlClass::TargetSufficient
    );
}

#[test]
fn analytical_runtime_equilibrium_parity_helper() {
    let p = 0.38;
    let analytical = theta_eq(p, D073_K_EQ);
    assert!((analytical - 0.95).abs() < 1e-12);
    assert!(eq_parity_ok(analytical, 0.95, 1e-9));
    assert!(!eq_parity_ok(analytical, 0.74, 1e-3));
    assert!(interface_p_within_tol(0.38 * 1.01, 0.38));
    assert!(!interface_p_within_tol(0.38 * 1.05, 0.38));
}

#[test]
fn five_timescale_and_long_horizon_classification() {
    // High seed occupancy that settles to θ_eq≈0.74 is equilibrium-below-contract.
    let class = classify_long_horizon(0.74, 0.74, 0.95, 0.992, true, false);
    assert_eq!(class, LongHorizonClass::EquilibriumBelowContract);
    let decay = classify_long_horizon(0.80, 0.90, 0.95, 0.992, true, false);
    assert_eq!(decay, LongHorizonClass::SlowTransientDecay);
    let maint = classify_long_horizon(0.96, 0.97, 0.95, 0.992, true, false);
    assert_eq!(maint, LongHorizonClass::TrueMaintenance);
    let collapse = classify_long_horizon(0.1, 0.74, 0.95, 0.992, true, true);
    assert_eq!(collapse, LongHorizonClass::BiologicalCollapse);
}

#[test]
fn route_c_when_invalid_control_and_sufficient_p_repairs() {
    let mut ev = RouteEvidence073::default();
    ev.d072_control_class = FixedPControlClass::NotActuallyFixed;
    ev.target_consistent_fixed_p_valid = true;
    ev.sufficient_fixed_p_repairs = true;
    ev.runtime_analytical_eq_agree = true;
    ev.endogenous_interface_p_sufficient_095 = false;
    assert_eq!(select_route(ev), D073Route::C);
    assert_eq!(
        select_route(ev).conclusion().as_str(),
        "D073_D072_CONTROL_INSUFFICIENT"
    );
}

#[test]
fn route_e_when_sufficient_hold_fails_to_repair() {
    let mut ev = RouteEvidence073::default();
    ev.d072_control_class = FixedPControlClass::NotActuallyFixed;
    ev.target_consistent_fixed_p_valid = true;
    ev.sufficient_fixed_p_repairs = false;
    ev.runtime_analytical_eq_agree = true;
    assert_eq!(select_route(ev), D073Route::E);
}

#[test]
fn route_l_redistribution_repairs() {
    let mut ev = RouteEvidence073::default();
    ev.d072_control_class = FixedPControlClass::TargetSufficient;
    ev.total_p_mass_large = true;
    ev.endogenous_interface_p_sufficient_095 = false;
    ev.redistribution_raises_interface_p = true;
    ev.redistribution_repairs = true;
    assert_eq!(select_route(ev), D073Route::L);
}

#[test]
fn route_m_when_fixed_p_repairs_but_endogenous_cannot() {
    let mut ev = RouteEvidence073::default();
    ev.d072_control_class = FixedPControlClass::TargetSufficient;
    ev.sufficient_fixed_p_repairs = true;
    ev.endogenous_interface_p_sufficient_095 = false;
    assert_eq!(select_route(ev), D073Route::M);
}

#[test]
fn route_t_short_horizon_decay() {
    let mut ev = RouteEvidence073::default();
    ev.d072_control_class = FixedPControlClass::TargetSufficient;
    ev.long_horizon_class = LongHorizonClass::SlowTransientDecay;
    ev.target_consistent_fixed_p_valid = false;
    assert_eq!(select_route(ev), D073Route::T);
}

#[test]
fn d070_d072_preservation_markers() {
    assert!(frozen_kinetics_unchanged(D073_K_EQ, D073_K_EXCHANGE, D073_GAMMA_MAX));
    assert!(d072_original_preserved(D072_ORIGINAL_CONCLUSION));
    assert!(d072_original_preserved(
        D072PrimaryConclusion::FrozenExchangeCannotRefillDamage.as_str()
    ));
    assert_eq!(D072_ROUTE_STATUS, "PROVISIONAL_PENDING_EQUILIBRIUM_SUFFICIENCY_AUDIT");
    assert_eq!(SEED_CONTRACT, "SEED_CAPACITY_CONTRACT_V1");
    assert!((REPAIR_OCC - 0.95).abs() < 1e-15);
    assert!((concentration_for_activity(0.38, 1.0) - 0.38).abs() < 1e-15);
}
