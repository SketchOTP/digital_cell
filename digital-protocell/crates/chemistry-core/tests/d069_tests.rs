//! D-069 exchange equilibrium / desorption audit gates stay observer-only.

use chemistry_core::d069_analysis::*;

fn qualified() -> IdentificationReport069 {
    IdentificationReport069 {
        params_positive_finite: true,
        bootstrap_spread: 0.25,
        loo_variation: 1.25,
        holdout_median_err: 0.15,
        holdout_max_err: 0.25,
        direction_accuracy: 0.95,
        eq_occupancy_err_pp: 0.05,
        no_radius_params: true,
        accounting_ok: true,
        predicts_damage_adsorption: true,
        predicts_zero_p_desorption: true,
    }
}

fn evidence() -> RouteEvidence069 {
    RouteEvidence069 {
        workspace_isolated: true,
        d068_reproduced: true,
        lineage_ok: true,
        direction_parity_ok: true,
        equilibrium_runtime_ok: true,
        surface_normalization_ok: true,
        accounting_ok: true,
        causality_ok: true,
        waste_blocks: false,
        identification: qualified(),
        existing_qualified: false,
        keq_calibration_qualified: false,
        on_off_qualified: false,
        timescale_only_qualified: false,
        s_repairs_a_fails: false,
        no_portable_law: false,
        execution_defect: false,
    }
}

#[test]
fn d068_reproduction_predicate() {
    assert!(d068_desorption_reproduction(2.77, 99.7, 425.6, 0.45, 0.48, 0.0065));
    assert!(!d068_desorption_reproduction(50.0, 40.0, 100.0, 0.9, 0.9, 0.5));
}

#[test]
fn p_and_theta_lineage() {
    let lin = frozen_exchange_lineage();
    assert!(lineage_resolved(&lin));
    assert!((p_activity(0.5, 1.0) - 0.5).abs() < 1e-15);
    // S = δ·Γ_max·θ ⇒ θ recovered
    let delta = 0.25;
    let gamma_max = 1.0;
    let theta = 0.4;
    let s = delta * gamma_max * theta;
    assert!((theta_occupancy(s, delta, gamma_max) - theta).abs() < 1e-12);
}

#[test]
fn dimensional_table_covers_factors() {
    let t = dimensional_table();
    assert!(t.iter().any(|f| f.symbol == "δ"));
    assert!(t.iter().any(|f| f.symbol == "K_eq"));
    assert!(t.iter().any(|f| f.symbol == "p"));
    assert!(t.iter().any(|f| f.symbol == "θ"));
}

#[test]
fn signed_accepted_exchange_and_parity() {
    assert!(accepted_exchange_parity(2.5, -2.5, 2.5));
    assert!(accepted_exchange_parity(-1.25, 1.25, -1.25));
    assert_eq!(split_accepted_exchange(3.0), (3.0, 0.0));
    assert_eq!(split_accepted_exchange(-4.0), (0.0, 4.0));
}

#[test]
fn analytical_equilibrium_and_runtime_zero_crossing() {
    let k = D069_K_EQ;
    let theta = 0.75;
    let pe = p_eq(theta, k);
    assert!(zero_crossing_matches(pe, pe, 1e-12));
    assert!((theta_eq(pe, k) - theta).abs() < 1e-12);
    let j0 = j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, k, pe, theta);
    assert!(j0.abs() < 1e-12);
}

#[test]
fn keq_star_calculation() {
    let k = k_eq_star(0.5, 0.02);
    assert!((k - 50.0).abs() < 1e-9);
    assert!(!keq_star_portable(&[1.0, 10.0], 1.0, 1.0, 0.1, 1.0));
    assert!(keq_star_portable(&[40.0, 50.0, 60.0], 1.2, 1.1, 0.2, 1.5));
}

#[test]
fn exchange_timescale_separation() {
    let tau = tau_exchange(D069_K_EXCHANGE, 1.0, D069_K_EQ, 0.1);
    assert!(tau.is_finite() && tau > 0.0);
    assert_eq!(
        classify_timescale(false, true, false, false, true),
        TimescaleClass::ExchangeTimescaleNotPrimary
    );
    assert_eq!(
        classify_timescale(true, true, false, false, true),
        TimescaleClass::ExchangeTimescalePortable
    );
}

#[test]
fn surface_normalization_helpers() {
    assert!(surface_scale_ok(1.0, 2.0, 1e-9));
    assert!(volume_scale_ok(1.0, 0.5, 1e-9));
    assert!(!surface_scale_ok(1.0, 3.0, 0.05));
}

#[test]
fn zero_p_desorption_and_zero_s() {
    let j_zero_p = j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, D069_K_EQ, 0.0, 0.5);
    assert!(j_zero_p < 0.0);
    let j_zero_s = j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, D069_K_EQ, 0.2, 0.0);
    assert!(j_zero_s >= 0.0);
    assert!(j_des_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, 0.0).abs() < 1e-15);
}

#[test]
fn damage_driven_adsorption_when_p_available() {
    // Damaged low θ with lawful p should request net adsorption.
    let j = j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, D069_K_EQ, 0.5, 0.1);
    assert!(j > 0.0);
}

#[test]
fn global_equilibrium_calibration_and_on_off() {
    let (k_on, k_off) = nested_on_off(D069_K_EXCHANGE, D069_K_EQ);
    let j_a = j_net_req(0.3, D069_K_EXCHANGE, 0.8, 1.0, D069_K_EQ, 0.2, 0.55);
    let j_c = j_net_on_off(0.3, 0.8, 1.0, k_on, k_off, 0.2, 0.55);
    assert!((j_a - j_c).abs() < 1e-14);
    // Candidate B: only K_eq changes
    let j_b = j_net_req(0.3, D069_K_EXCHANGE, 0.8, 1.0, 200.0, 0.2, 0.55);
    assert!(j_b > j_a);
}

#[test]
fn starvation_membrane_loss_and_feasibility() {
    assert_eq!(
        classify_precursor_feasibility(0.05, 0.1, 0.5, 1.0, true),
        PrecursorFeasibilityClass::CurrentEquilibriumPrecursorFeasible
    );
    assert_eq!(
        classify_precursor_feasibility(2.0, 0.05, 0.5, 0.8, true),
        PrecursorFeasibilityClass::CurrentEquilibriumRequiresExcessPrecursor
    );
    assert_eq!(
        classify_precursor_feasibility(f64::INFINITY, 0.1, 0.5, 1.0, true),
        PrecursorFeasibilityClass::CurrentEquilibriumMateriallyImpossible
    );
}

#[test]
fn equilibrium_manifold_and_w_control_route() {
    assert_eq!(
        classify_equilibrium_manifold(0.05, 0.1, 0.85, -0.2),
        EquilibriumManifoldClass::MembraneSystematicallyBelowRequiredP
    );
    let mut ev = evidence();
    ev.waste_blocks = true;
    assert_eq!(select_route(ev.clone()).0, D069Route::W);
    ev.waste_blocks = false;
    ev.no_portable_law = true;
    assert_eq!(
        select_route(ev.clone()).1,
        D069PrimaryConclusion::NoPortableMembraneExchangeLaw
    );
    ev.no_portable_law = false;
    ev.keq_calibration_qualified = true;
    assert_eq!(select_route(ev).0, D069Route::E);
}

#[test]
fn route_selection_priorities() {
    let mut ev = evidence();
    ev.workspace_isolated = false;
    assert_eq!(
        select_route(ev.clone()).1,
        D069PrimaryConclusion::WorkspaceScopeNotIsolated
    );
    ev = evidence();
    ev.direction_parity_ok = false;
    assert_eq!(select_route(ev.clone()).0, D069Route::X);
    ev = evidence();
    ev.execution_defect = true;
    assert_eq!(
        select_route(ev.clone()).1,
        D069PrimaryConclusion::MembraneExchangeExecutionDefect
    );
    ev = evidence();
    ev.existing_qualified = true;
    assert_eq!(select_route(ev).0, D069Route::Q);
}

#[test]
fn identification_enforces_thresholds() {
    assert!(qualified().qualifies());
    let mut failed = qualified();
    failed.bootstrap_spread = BOOTSTRAP_SPREAD_MAX + 0.01;
    assert!(!failed.qualifies());
}

#[test]
fn q_c_changes_rate_not_equilibrium() {
    let pe = p_eq(0.6, D069_K_EQ);
    let j_low = j_net_req(1.0, D069_K_EXCHANGE, 0.2, 1.0, D069_K_EQ, pe, 0.6);
    let j_high = j_net_req(1.0, D069_K_EXCHANGE, 0.9, 1.0, D069_K_EQ, pe, 0.6);
    assert!(j_low.abs() < 1e-12 && j_high.abs() < 1e-12);
    let jn_low = j_net_req(1.0, D069_K_EXCHANGE, 0.2, 1.0, D069_K_EQ, 0.5, 0.3);
    let jn_high = j_net_req(1.0, D069_K_EXCHANGE, 0.9, 1.0, D069_K_EQ, 0.5, 0.3);
    assert!(jn_high.abs() > jn_low.abs());
    assert_eq!(jn_low.signum(), jn_high.signum());
}




#[test]
fn desorption_matches_seed_over_capacity() {
    assert!(desorption_explained_by_over_capacity(99.666, 99.667, 2.3));
    assert!(!desorption_explained_by_over_capacity(99.666, 10.0, 2.3));
    assert!(!desorption_explained_by_over_capacity(99.666, 99.667, 1.0));
}
