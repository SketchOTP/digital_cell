//! D-071 capacity-bounded precursor demand regulation tests.

use chemistry_core::config::SimParams;
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d071_analysis::*;
use chemistry_core::membrane::{precursor_synthesis_rate, precursor_synthesis_rate_regulated};

#[test]
fn d070_control_predicate() {
    assert!(d070_control_reproduced(
        0.35, 0.992, 1.0, 76.0, 493.0, false, 1.0
    ));
    assert!(!d070_control_reproduced(
        0.90, 0.992, 1.0, 76.0, 80.0, false, 1.0
    ));
}

#[test]
fn product_inhibition_equation_and_old_state() {
    let k_i = 2.0;
    assert!((product_inhibition_factor(0.0, k_i) - 1.0).abs() < 1e-15);
    assert!((product_inhibition_factor(2.0, k_i) - 0.5).abs() < 1e-15);
    // Old-state evaluation: increasing P lowers rate factor.
    assert!(product_inhibition_factor(1.0, k_i) > product_inhibition_factor(4.0, k_i));
    assert!(regulated_rate_matches_equation(
        0.2, 0.5, 0.4, 1.0, 0.1, 1.0, 1.0, k_i
    ));
}

#[test]
fn a_to_p_conservation_and_defaults() {
    assert!(a_to_p_conservation(0.3, -0.3, 0.3));
    assert!(!a_to_p_conservation(0.3, -0.2, 0.3));
    let params = v8_schema3_params();
    assert!(defaults_preserve_constitutive(&params, 1.0, 0.4, 0.5, 1.0));
    let r0 = precursor_synthesis_rate(1.0, 0.4, 0.5, &params);
    let r = precursor_synthesis_rate_regulated(1.0, 0.4, 0.5, 1.0, &params);
    assert!((r - r0).abs() < 1e-15);
}

#[test]
fn candidate_identity_includes_schema() {
    let a = PrecursorRegulationParams::product_inhibition(1.5);
    let b = PrecursorRegulationParams::product_inhibition(1.5);
    let c = PrecursorRegulationParams::reduced(0.25);
    assert_eq!(a.identity_hash(), b.identity_hash());
    assert_ne!(a.identity_hash(), c.identity_hash());
    assert!(a.identity_hash().contains("a") || a.identity_hash().len() == 64);
}

#[test]
fn bounded_p_and_m_p_derivation() {
    let slope = normalized_p_slope(100.0, 100.5, 1000, 100.0);
    assert!(slope < P_SLOPE_BOUND);
    assert!(p_is_bounded(slope, 100.5, 100.0));
    assert!(!p_is_bounded(1e-3, 200.0, 100.0));
    let m = derive_m_p_candidates(10.0);
    assert!(m.len() <= 3);
    assert!(m.iter().all(|&x| x > 0.0 && x <= 1.0));
}

#[test]
fn zero_a_and_zero_production_controls() {
    let mut params = v8_schema3_params();
    PrecursorRegulationParams::product_inhibition(1.0).apply_to(&mut params);
    assert_eq!(
        precursor_synthesis_rate_regulated(1.0, 0.4, 0.0, 1.0, &params),
        0.0
    );
    params.k_precursor = 0.0;
    assert_eq!(
        precursor_synthesis_rate_regulated(1.0, 0.4, 0.5, 1.0, &params),
        0.0
    );
}

#[test]
fn maintenance_and_portability_predicates() {
    assert!(maintenance_windows_pass(
        &[0.85, 0.86, 0.87],
        &[0.96, 0.97, 0.98],
        &[1.0, 1.0, 1.0],
        &[1e-5, 1e-5, 1e-5]
    ));
    assert!(radius_portable_row(0.85, 0.9, 0.96, 1.0, true));
    assert!(!radius_portable_row(0.5, 0.9, 0.96, 1.0, true));
}

#[test]
fn capacity_contract_and_frozen_kinetics() {
    assert_eq!(
        chemistry_core::d070_analysis::SEED_CAPACITY_CONTRACT_V1,
        "SEED_CAPACITY_CONTRACT_V1"
    );
    assert!(frozen_kinetics_unchanged(D071_K_EQ, D071_K_EXCHANGE, D071_GAMMA_MAX));
}

#[test]
fn route_starves_and_not_portable() {
    let base = RouteEvidence071 {
        workspace_isolated: true,
        d070_control_ok: true,
        ledger_ok: true,
        precursor_dominant_avoidable: true,
        candidate_identifiable: true,
        conservation_ok: true,
        numerical_ok: true,
        r22_maintenance_ok: true,
        a_retained: true,
        p_bounded: true,
        repair_ok: false,
        repair_starved: true,
        causal_ok: true,
        portable: true,
        stage_e_ok: false,
        foundational_regression: false,
    };
    let (r, c) = select_route(base.clone());
    assert_eq!(r, D071Route::S);
    assert_eq!(
        c.as_str(),
        "D071_PRECURSOR_REGULATION_STARVES_MEMBRANE_REPAIR"
    );
    let mut not_port = base;
    not_port.repair_starved = false;
    not_port.repair_ok = true;
    not_port.portable = false;
    let (r2, _) = select_route(not_port);
    assert_eq!(r2, D071Route::P);
}

#[test]
fn apply_params_isolates_candidates() {
    let mut params = SimParams::default();
    PrecursorRegulationParams::reduced(0.2).apply_to(&mut params);
    assert!((params.precursor_m_p - 0.2).abs() < 1e-15);
    assert_eq!(params.precursor_product_inhibition_ki, 0.0);
    PrecursorRegulationParams::product_inhibition(3.0).apply_to(&mut params);
    assert!((params.precursor_m_p - 1.0).abs() < 1e-15);
    assert!((params.precursor_product_inhibition_ki - 3.0).abs() < 1e-15);
}
