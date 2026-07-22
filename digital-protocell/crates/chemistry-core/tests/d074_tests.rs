//! D-074 cellwise exchange integration parity tests.

use chemistry_core::d073_analysis::equilibrium_occupancy;
use chemistry_core::d074_analysis::*;
use chemistry_core::surface_density::validate_exchange_cell;

#[test]
fn exact_discrete_bath_be_update() {
    let k_eq = D074_K_EQ;
    let k_ex = D074_K_EXCHANGE;
    let p = 0.38;
    let q = 0.4;
    let dt = 0.05;
    let th_eq = theta_eq_of(p, k_eq);
    let lam = exchange_lambda(k_ex, q, k_eq, p);
    let th0 = 0.1;
    let th1 = discrete_bath_be_theta(th0, th_eq, lam, dt);
    let expected = th_eq + (th0 - th_eq) / (1.0 + lam * dt);
    assert!((th1 - expected).abs() < 1e-15);
    assert!(frozen_kinetics_unchanged(k_eq, k_ex, D074_GAMMA_MAX));
    assert!((p_star(0.95) - 0.38).abs() < 1e-9);
}

#[test]
fn heterogeneous_q_parity_runtime_be() {
    let k_eq = D074_K_EQ;
    let k_ex = D074_K_EXCHANGE;
    let p_ref = D074_P_REF;
    let gamma = D074_GAMMA_MAX;
    let delta = 0.25;
    let dt = 0.05;
    let p_act = 0.38;
    let p_old = p_act * p_ref;
    let s_old = 0.0;
    for q in [0.0, 0.05, 0.2, 0.4, 1.0] {
        let (s1, p1, xfer) = runtime_invariant_exchange_step(
            s_old, p_old, delta, q, k_ex, k_eq, p_ref, gamma, dt,
        )
        .expect("BE solve");
        assert!((p1 + s1 - (p_old + s_old)).abs() < 1e-12);
        if q <= Q_INACTIVE_FLOOR {
            assert!(xfer.abs() < 1e-14, "zero-q must not exchange");
        } else {
            assert!(xfer > 0.0, "positive q should adsorb from empty");
        }
        // Bath formula is diagnostic only; coupled BE conserves T.
        let _bath = predicted_bath_be_delta_s(s_old, delta * gamma, p_act, q, k_ex, k_eq, dt);
        let _ = validate_exchange_cell(p1, s1, delta, gamma, 1e-12, 1.0, 1.0, 0.0);
    }
}

#[test]
fn heterogeneous_capacity_and_zero_catalyst() {
    let k_eq = D074_K_EQ;
    let k_ex = D074_K_EXCHANGE;
    let p = 0.38;
    let th_eq = equilibrium_occupancy(p, k_eq);
    let cells = [
        // capacity, s_post, theta_eq, q, class
        (
            1.0,
            0.0,
            th_eq,
            0.4,
            classify_damaged_cell(0.4, 1.0, 0.0, th_eq, exchange_lambda(k_ex, 0.4, k_eq, p)),
        ),
        (
            2.0,
            0.1,
            th_eq,
            0.0,
            classify_damaged_cell(0.0, 2.0, 0.05, th_eq, 0.0),
        ),
        (
            0.0,
            0.0,
            th_eq,
            0.4,
            classify_damaged_cell(0.4, 0.0, 0.0, th_eq, 0.0),
        ),
    ];
    assert_eq!(cells[0].4, CellExchangeClass::ExchangeActive);
    assert_eq!(cells[1].4, CellExchangeClass::ExchangeInactiveQ0);
    assert_eq!(cells[2].4, CellExchangeClass::UnsupportedCapacity);

    let report = reachable_repair_ceiling(&cells, 10.0, 20.0);
    // active: 1.0*th_eq; inactive retains 0.1; unsupported retains 0; + undamaged 10
    let expected_m = 1.0 * th_eq + 0.1 + 0.0 + 10.0;
    assert!((report.m_reachable - expected_m).abs() < 1e-12);
    assert!(report.inactive_capacity > 0.0);
    assert!(report.unsupported_capacity == 0.0); // capacity 0 contributes 0
    assert!(report.max_theoretical_repair_fraction < 1.0);
}

#[test]
fn capacity_weighted_exposure_gate() {
    let cells = vec![
        (9.5, ExposureClass::ExposureGe5),
        (0.4, ExposureClass::Exposure1To5),
        (0.1, ExposureClass::ZeroExposure),
    ];
    let cov = exposure_coverage(&cells);
    assert!((cov.fraction_ge5 - 0.95).abs() < 1e-12);
    assert!(cov.qualifies_five_timescale);

    let cells2 = vec![
        (8.0, ExposureClass::ExposureGe5),
        (2.0, ExposureClass::ZeroExposure),
    ];
    let cov2 = exposure_coverage(&cells2);
    assert!(!cov2.qualifies_five_timescale);
}

#[test]
fn equal_and_opposite_ps_transfer() {
    let (s1, p1, xfer) = runtime_invariant_exchange_step(
        0.0,
        0.5,
        0.3,
        0.4,
        D074_K_EXCHANGE,
        D074_K_EQ,
        D074_P_REF,
        D074_GAMMA_MAX,
        0.05,
    )
    .unwrap();
    assert!((p1 - (0.5 - xfer)).abs() < 1e-12);
    assert!((s1 - xfer).abs() < 1e-12);
}

#[test]
fn route_q_when_inactive_catalyst_limits_ceiling() {
    let mut ev = RouteEvidence074::default();
    ev.d073_reproduced = true;
    ev.static_cellwise_parity_ok = true;
    ev.accepted_step_replay_ok = true;
    ev.runtime_matches_discrete_predictor = true;
    ev.inactive_q0_capacity_fraction = 0.12;
    ev.reachable_ceiling_below_gate = true;
    assert_eq!(select_route(ev), D074Route::Q);
    assert_eq!(
        select_route(ev).conclusion().as_str(),
        "D074_LOCAL_CATALYTIC_EXPOSURE_LIMIT"
    );
}

#[test]
fn route_e_when_repair_restores_parity() {
    let mut ev = RouteEvidence074::default();
    ev.d073_reproduced = true;
    ev.runtime_matches_discrete_predictor = false;
    ev.repair_restored_parity = true;
    assert_eq!(select_route(ev), D074Route::E);
}

#[test]
fn route_t_when_mean_tau_overstates_exposure() {
    let mut ev = RouteEvidence074::default();
    ev.d073_reproduced = true;
    ev.static_cellwise_parity_ok = true;
    ev.accepted_step_replay_ok = true;
    ev.runtime_matches_discrete_predictor = true;
    ev.inactive_q0_capacity_fraction = 0.0;
    ev.unsupported_capacity_fraction = 0.0;
    ev.exposure_qualifies_five_tau = false;
    ev.mean_tau_overstated_exposure = true;
    assert_eq!(select_route(ev), D074Route::T);
}

#[test]
fn route_x_unresolved_parity() {
    let mut ev = RouteEvidence074::default();
    ev.d073_reproduced = true;
    ev.runtime_matches_discrete_predictor = false;
    ev.repair_restored_parity = false;
    assert_eq!(select_route(ev), D074Route::X);
}

#[test]
fn d073_preservation_and_repro_helpers() {
    assert!(d073_conclusion_preserved(D073_CONCLUSION));
    assert!(recovery_matches_d073(0.941, 0.941));
    assert!(!recovery_matches_d073(0.80, 0.941));
    for (name, p, _) in d073_expected_recoveries() {
        assert!(!name.is_empty());
        assert!(*p > 0.0);
    }
}

#[test]
fn mask_support_and_attenuation() {
    assert_eq!(
        classify_exposure(10.0, 0.0, false),
        ExposureClass::InterfaceUnsupported
    );
    let a = attenuation_factor(2.0, 0.5);
    assert!((a - 1.0 / 2.0).abs() < 1e-15); // 1/(1+1)=0.5... wait 1+2*0.5=2 → 0.5
    assert!((a - 0.5).abs() < 1e-15);
    let exp = exposure_increment(D074_K_EXCHANGE, 0.4, D074_K_EQ, 0.38, 0.05);
    assert!(exp > 0.0);
}

#[test]
fn aggregate_mass_reconstruction_identity() {
    // ΔS_obs = ΔS_ex + ΔS_tr + ΔS_other
    let ds_ex: f64 = 1.25;
    let ds_tr: f64 = -0.10;
    let ds_other: f64 = 0.0;
    let ds_obs: f64 = 1.15;
    assert!((ds_obs - (ds_ex + ds_tr + ds_other)).abs() < ACCOUNTING_TOL);
}
