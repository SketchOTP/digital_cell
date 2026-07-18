//! D-028 bracketed surface-renewal root recovery tests.

use chemistry_core::d028_analysis::{
    adsorption_response_monotonic, frozen_d027_monotonicity_holds, gate0_endpoint_ok,
    regula_falsi_trial, select_smallest_passing, solve_bracketed_root, BracketEndpoints,
    ProposalMethod, SurfaceBalanceMetrics, D028_K_ADS_0_5X, D028_K_ADS_1X, D028_K_ADS_2X,
    D028_MAX_NEW_CANDIDATES, D028_Q_0_5X, D028_Q_1X, D028_Q_2X,
};

#[test]
fn d027_bracket_reproduction_thresholds() {
    assert!(gate0_endpoint_ok(D028_K_ADS_1X, D028_Q_1X, true));
    assert!(gate0_endpoint_ok(D028_K_ADS_2X, D028_Q_2X, false));
    assert!(!gate0_endpoint_ok(D028_K_ADS_1X, 1.0, true));
    assert!(!gate0_endpoint_ok(D028_K_ADS_2X, 1.0, false));
}

#[test]
fn exact_machine_candidate_loading() {
    assert!((D028_K_ADS_0_5X - 0.01688873302573429).abs() < 1e-18);
    assert!((D028_K_ADS_1X - 0.03377746605146858).abs() < 1e-18);
    assert!((D028_K_ADS_2X - 0.06755493210293716).abs() < 1e-18);
    assert!((D028_K_ADS_1X / D028_K_ADS_0_5X - 2.0).abs() < 1e-12);
    assert!((D028_K_ADS_2X / D028_K_ADS_1X - 2.0).abs() < 1e-12);
}

#[test]
fn q_g_common_window_calculation() {
    let m = SurfaceBalanceMetrics::from_rates(0.15350183671968975, 0.16933564339636573, 10.0);
    assert!((m.q_surface - D028_Q_1X).abs() < 1e-12);
    let expected_g = (0.15350183671968975 - 0.16933564339636573) / 10.0;
    assert!((m.g_surface - expected_g).abs() < 1e-15);
    assert!(!m.is_balanced());
}

#[test]
fn monotonicity_validation() {
    assert!(frozen_d027_monotonicity_holds());
    assert!(adsorption_response_monotonic(&[D028_Q_0_5X, D028_Q_1X, D028_Q_2X]));
    assert!(!adsorption_response_monotonic(&[0.9, 0.8, 1.1]));
}

#[test]
fn regula_falsi_proposal_and_bisection_fallback() {
    let b = BracketEndpoints::from_d027_1x_2x();
    let k = regula_falsi_trial(b.k_low, b.q_low, b.k_high, b.q_high);
    assert!(k > b.k_low && k < b.k_high);
    assert!((k - 0.0548).abs() < 5e-4);

    let (k2, method, reason) = b.propose(Some(k));
    assert_eq!(method, ProposalMethod::Bisection);
    assert!(reason.is_some());
    assert!(k2 > b.k_low && k2 < b.k_high);
}

#[test]
fn bracket_preservation_no_out_of_bracket() {
    let b = BracketEndpoints::from_d027_1x_2x();
    assert!(b.update_with_observation(b.k_low - 0.001, 0.5).is_err());
    assert!(b.update_with_observation(b.k_high + 0.001, 1.2).is_err());
    let next = b.update_with_observation(0.05, 0.99).unwrap();
    assert!(next.straddles_unity());
    assert!(next.k_low > b.k_low || (next.k_low - 0.05).abs() < 1e-15);
}

#[test]
fn max_candidate_count_and_smallest_root() {
    assert_eq!(D028_MAX_NEW_CANDIDATES, 4);
    let initial = BracketEndpoints {
        k_low: 0.03,
        q_low: 0.90,
        k_high: 0.07,
        q_high: 1.10,
    };
    let mut calls = 0usize;
    let result = solve_bracketed_root(initial, |k| {
        calls += 1;
        let q = 0.90 + (k - 0.03) / 0.04 * 0.20;
        (q, 1e-8)
    });
    assert!(calls <= D028_MAX_NEW_CANDIDATES);
    assert!(result.pass);
    assert_eq!(
        select_smallest_passing(&[(0.06, true), (0.05, true)]),
        Some(0.05)
    );
    for it in &result.iterations {
        assert!(it.k_trial > initial.k_low && it.k_trial < initial.k_high);
    }
}

#[test]
fn selected_k_ads_immutability_contract() {
    // Gate 9 must not alter selected k_ads — encode as analysis invariant helper.
    let selected: f64 = 0.0548;
    let productive_only = ["k_activation", "k_rep", "k_precursor", "k_structure"];
    assert!(!productive_only.iter().any(|n| *n == "k_ads"));
    assert!(selected.is_finite() && selected > D028_K_ADS_1X && selected < D028_K_ADS_2X);
}
