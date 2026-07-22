//! D-075 cellwise exposure-gated membrane requalification tests.

use chemistry_core::d074_analysis::{exchange_lambda, D074_K_EQ, D074_K_EXCHANGE};
use chemistry_core::d075_analysis::*;

#[test]
fn explicit_and_backward_euler_contraction() {
    let lam = 0.25;
    let dt = 0.04;
    let c_ex = explicit_contraction(lam, dt);
    let c_be = backward_euler_contraction(lam, dt);
    assert!((c_ex - (1.0 - lam * dt).abs()).abs() < 1e-15);
    assert!((c_be - 1.0 / (1.0 + lam * dt)).abs() < 1e-15);
    assert!(frozen_kinetics_unchanged(D075_K_EQ, D075_K_EXCHANGE, D075_GAMMA_MAX));
}

#[test]
fn exact_effective_exposure_accumulates_minus_ln_c() {
    let mut st = CellExposureState::default();
    let lam = 0.2;
    let dt = 0.05;
    st.observe_attempt(IntegratorKind::BackwardEuler, lam, dt);
    st.observe_attempt(IntegratorKind::BackwardEuler, lam, dt);
    let c = backward_euler_contraction(lam, dt);
    let expected = 2.0 * exact_effective_exposure_increment(c);
    assert!((st.e_exact - expected).abs() < 1e-12);
    assert!((st.lambda_cum - 2.0 * lam * dt).abs() < 1e-15);
    assert!((st.backward_euler_e - expected).abs() < 1e-12);
    assert_eq!(st.accepted_steps, 2);
}

#[test]
fn rejected_steps_contribute_zero_exposure() {
    let mut st = CellExposureState::default();
    st.observe_attempt(IntegratorKind::Rejected, 1.0, 0.1);
    st.observe_attempt(IntegratorKind::Rejected, 1.0, 0.1);
    assert_eq!(st.e_exact, 0.0);
    assert_eq!(st.lambda_cum, 0.0);
    assert_eq!(st.accepted_steps, 0);
    assert_eq!(st.rejected_attempts_seen, 2);
}

#[test]
fn snapshot_resume_continuity() {
    let mut a = CellExposureState::default();
    a.observe_attempt(IntegratorKind::ExplicitEuler, 0.1, 0.05);
    let snap = ExposureObserverSnapshot {
        cells: vec![a.clone()],
        accepted_sim_time: 0.05,
        accepted_steps: 1,
        rejected_attempts: 0,
    };
    let mut b = snap.cells[0].clone();
    b.observe_attempt(IntegratorKind::BackwardEuler, 0.2, 0.05);
    assert!(b.e_exact > a.e_exact);
    assert_eq!(b.accepted_steps, 2);
}

#[test]
fn capacity_weighted_qualification_and_zero_unsupported() {
    let cells = vec![
        // capacity, E, supported, e_ex, e_be, lam
        (9.0, 5.2, true, 1.0, 4.2, 5.2),
        (0.9, 3.0, true, 0.0, 3.0, 3.0),
        (0.05, 0.0, true, 0.0, 0.0, 0.0),
        (2.0, 0.0, false, 0.0, 0.0, 0.0),
    ];
    let q = qualify_exposure_capacity(&cells);
    // lawful = 9.95; ge5 = 9.0 → 9/9.95 < 0.95
    assert!((q.fraction_e_ge5 - 9.0 / 9.95).abs() < 1e-12);
    assert!(!q.qualifies);
    assert!(q.capacity_unsupported > 0.0);
    assert!(q.zero_exposure_fraction < ZERO_EXPOSURE_CAP_FRAC_MAX);

    let cells_ok = vec![
        (9.6, 5.1, true, 0.0, 5.1, 5.1),
        (0.3, 5.0, true, 0.0, 5.0, 5.0),
        (0.05, 0.0, true, 0.0, 0.0, 0.0),
    ];
    let q2 = qualify_exposure_capacity(&cells_ok);
    assert!(q2.qualifies);
    assert!(q2.fraction_e_ge1 >= 0.95);
}

#[test]
fn damaged_region_exposure_uses_relevant_subset() {
    let damaged_only = vec![(1.0, 5.0, true, 0.0, 5.0, 5.0), (1.0, 5.0, true, 0.0, 5.0, 5.0)];
    let q = qualify_exposure_capacity(&damaged_only);
    assert!(q.qualifies);
    assert!((q.relevant_lawful_capacity - 2.0).abs() < 1e-15);
}

#[test]
fn synthetic_contraction_parity_e1_e3_e5() {
    for target_e in [1.0, 3.0, 5.0] {
        let ratio = synthetic_residual_ratio(target_e);
        assert!((ratio - (-target_e).exp()).abs() < 1e-15);
        let d0 = 0.8;
        let d1 = predict_distance_from_eq(d0, target_e);
        assert!((d1 - d0 * ratio).abs() < 1e-15);
    }
    // Multi-step BE ledger: E = n ln(1+λdt) matches product of contractions.
    let lam = 0.15;
    let dt = 0.05;
    let n = 40;
    let mut st = CellExposureState::default();
    let mut product = 1.0;
    for _ in 0..n {
        st.observe_attempt(IntegratorKind::BackwardEuler, lam, dt);
        product *= backward_euler_contraction(lam, dt);
    }
    assert!((product - (-st.e_exact).exp()).abs() < 1e-10);
    assert!((st.e_exact - (n as f64) * (1.0 + lam * dt).ln()).abs() < 1e-10);
}

#[test]
fn long_horizon_classification_matrix() {
    let maint = MaintenanceEvidence {
        exposure_qualified: true,
        numerical_terminal: false,
        biological_terminal: false,
        mature_occupancy: 0.97,
        a_retention: 0.85,
        c_retention: 0.9,
        p_bounded: true,
        zero_exposure_fraction: 0.0,
        catalytic_exposure_failure: false,
        eq_occ_from_local_p: 0.96,
        s_retention: 0.995,
    };
    assert_eq!(
        classify_long_horizon(maint),
        LongHorizonClass::TrueLongHorizonMaintenance
    );

    let not_q = MaintenanceEvidence {
        exposure_qualified: false,
        ..maint
    };
    assert_eq!(classify_long_horizon(not_q), LongHorizonClass::NotQualified);

    let cat = MaintenanceEvidence {
        exposure_qualified: false,
        catalytic_exposure_failure: true,
        ..maint
    };
    assert_eq!(
        classify_long_horizon(cat),
        LongHorizonClass::CatalyticExposureFailure
    );
}

#[test]
fn maintenance_before_damage_enforced_by_route_logic() {
    // Constitutive repair without maintenance must not yield Route Q.
    let mut ev = RouteEvidence075 {
        d074_reproduced: true,
        observer_ok: true,
        synthetic_calibration_ok: true,
        fixed_p_ok: true,
        constitutive_maintains: false,
        constitutive_repairs: true,
        ..RouteEvidence075::default()
    };
    assert_eq!(select_route(ev), D075Route::F);

    ev.constitutive_maintains = true;
    assert_eq!(select_route(ev), D075Route::Q);
}

#[test]
fn radius_portability_and_regulation_routes() {
    let mut ev = RouteEvidence075 {
        d074_reproduced: true,
        observer_ok: true,
        synthetic_calibration_ok: true,
        fixed_p_ok: true,
        regulated_maintains: true,
        regulated_repairs: true,
        regulated_a_ok: true,
        regulated_p_bounded: true,
        radius_portable: true,
        ..RouteEvidence075::default()
    };
    assert_eq!(select_route(ev), D075Route::R);
    assert_eq!(
        select_route(ev).conclusion().as_str(),
        "D075_PRECURSOR_REGULATION_QUALIFIED"
    );

    ev.regulated_repairs = false;
    assert_eq!(select_route(ev), D075Route::T);

    ev.regulated_maintains = false;
    ev.constitutive_maintains = true;
    ev.constitutive_repairs = true;
    ev.stage_e_ok = true;
    assert_eq!(select_route(ev), D075Route::StageE);
}

#[test]
fn d074_preservation_constants() {
    assert_eq!(D074_CONCLUSION, "D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT");
    assert_eq!(D075_STARTING_TAG, "D-074-cellwise-exchange-parity-audit");
    assert!((D075_SELECTED_M_P - 0.0013190570087785272).abs() < 1e-18);
    let lam = exchange_lambda(D074_K_EXCHANGE, 0.4, D074_K_EQ, 0.38);
    assert!(lam > 0.0);
    assert_eq!(
        select_route(RouteEvidence075::default()),
        D075Route::StopD074
    );
}

#[test]
fn production_dispatch_rejected_when_delta_below_floor() {
    let kind = classify_production_exchange_step(
        0.1, 0.2, 1e-20, 0.4, D075_K_EXCHANGE, D075_K_EQ, D075_P_REF, D075_GAMMA_MAX, 1e-12, 0.05,
    );
    assert_eq!(kind, IntegratorKind::Rejected);
}
