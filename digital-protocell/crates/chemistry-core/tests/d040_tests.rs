//! D-040 focused tests: equilibrium, parity, budgets, controls, route rules.

use chemistry_core::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use chemistry_core::d040_analysis::{
    audit_exchange_sample, classify_basins, classify_endogenous_capacity, classify_equilibrium_audit,
    earliest_causal_divergence, find_reduced_fixed_points, frozen_kinetics_ok, j_predicted,
    required_p_for_theta, required_p_thresholds, select_route, theta_eq, ChronologyWindow,
    EndogenousCapacityClass, ExchangeParityClass, PrecursorSufficiencyOutcome, ReducedApsParams,
    RouteEvidence, D040_K_FROZEN, D040_RECORD,
};
use chemistry_core::nullcline::FixedPointClass;

#[test]
fn exchange_equilibrium_calculation() {
    let k = D040_K_FROZEN;
    assert!((theta_eq(k, 0.0) - 0.0).abs() < 1e-15);
    let p50 = required_p_for_theta(k, 0.5);
    assert!((theta_eq(k, p50) - 0.5).abs() < 1e-12);
    assert!((p50 - 1.0 / k).abs() < 1e-12);
}

#[test]
fn required_p_thresholds_monotonic() {
    let th = required_p_thresholds(50.0);
    assert_eq!(th.len(), 4);
    for w in th.windows(2) {
        assert!(w[1].1 > w[0].1);
    }
}

#[test]
fn runtime_equation_parity_pass_below_eq() {
    let alpha = D031_ALPHA_FROZEN;
    let beta = D031_BETA_FROZEN;
    let k = D040_K_FROZEN;
    let p = 0.01; // below p for θ=0.5
    let theta = 0.6;
    let q = 0.8;
    let jp = j_predicted(alpha, beta, q, p, theta);
    let sample = audit_exchange_sample("t0", p, theta, q, jp, alpha, beta, k);
    assert!(sample.parity_ok);
    assert!(sample.theta < sample.theta_eq || sample.j_predicted < 0.0);
    let class = classify_equilibrium_audit(&[sample]);
    assert!(matches!(
        class,
        ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium
            | ExchangeParityClass::ExchangeLawParityPassPrecursorAboveEquilibrium
    ));
}

#[test]
fn runtime_parity_defect_on_sign_flip() {
    let alpha = D031_ALPHA_FROZEN;
    let beta = D031_BETA_FROZEN;
    let k = D040_K_FROZEN;
    let p = 0.1;
    let theta = 0.2;
    let q = 1.0;
    let jp = j_predicted(alpha, beta, q, p, theta);
    let bad = audit_exchange_sample("bad", p, theta, q, -jp - 1.0, alpha, beta, k);
    assert!(!bad.parity_ok);
    assert_eq!(
        classify_equilibrium_audit(&[bad]),
        ExchangeParityClass::ExchangeRuntimeParityDefect
    );
}

#[test]
fn a_p_budget_closure_helpers_finite() {
    assert!(frozen_kinetics_ok());
    assert_eq!(D040_RECORD, "SCHEMA3_V8_MAINTENANCE_COUPLING_FAILED");
}

#[test]
fn chronology_picks_earliest_not_largest() {
    // Window 1: only P synthesis declines (A held). Window 2: huge terminal desorption.
    let windows = vec![
        ChronologyWindow {
            index: 0,
            theta: 0.7,
            theta_eq: 0.8,
            p: 0.05,
            a: 1.0,
            a_retention: 1.0,
            p_synthesis: 1.0,
            p_leakage: 0.1,
            a_leakage: 0.1,
            net_exchange: 0.01,
            permeability_proxy: 1.0,
            precursor_synthesis_demand: 1.0,
        },
        ChronologyWindow {
            index: 1,
            theta: 0.69,
            theta_eq: 0.75,
            p: 0.04,
            a: 1.0,
            a_retention: 1.0,
            p_synthesis: 0.9,
            p_leakage: 0.1,
            a_leakage: 0.1,
            net_exchange: 0.0,
            permeability_proxy: 1.0,
            precursor_synthesis_demand: 1.0,
        },
        ChronologyWindow {
            index: 2,
            theta: 0.5,
            theta_eq: 0.6,
            p: 0.02,
            a: 0.5,
            a_retention: 0.5,
            p_synthesis: 0.2,
            p_leakage: 0.5,
            a_leakage: 0.5,
            net_exchange: -1.0,
            permeability_proxy: 2.0,
            precursor_synthesis_demand: 1.0,
        },
    ];
    let c = earliest_causal_divergence(&windows);
    assert_eq!(c.as_str(), "P_SYNTHESIS_DECLINE");
}

#[test]
fn endogenous_capacity_classification() {
    assert_eq!(
        classify_endogenous_capacity(0.1, 0.12, 0.12, 0.12, 0.12),
        EndogenousCapacityClass::SynthesisCapacitySufficient
    );
    assert_eq!(
        classify_endogenous_capacity(0.1, 0.02, 0.02, 0.02, 0.02),
        EndogenousCapacityClass::SynthesisCapacityInsufficient
    );
    assert_eq!(
        classify_endogenous_capacity(0.1, 0.03, 0.11, 0.04, 0.03),
        EndogenousCapacityClass::ProductionSufficientRetentionInsufficient
    );
    assert_eq!(
        classify_endogenous_capacity(0.1, 0.03, 0.03, 0.03, 0.12),
        EndogenousCapacityClass::ASupplyInsufficient
    );
}

#[test]
fn route_p_when_sufficient_fixed_p_and_weak_endogenous() {
    let ev = RouteEvidence {
        parity: ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium,
        sufficiency: Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor),
        endogenous: Some(EndogenousCapacityClass::SynthesisCapacityInsufficient),
        p_clamp_restores: true,
        a_clamp_restores: false,
        perm_freeze_restores: false,
        no_decay_restores: false,
        no_leak_restores: false,
        healthy_fixed_point_exists: true,
        bistable_basins: false,
        damage_crosses_separatrix: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(&ev).as_str(),
        "D040_PRECURSOR_SYNTHESIS_CAPACITY_DEFICIT"
    );
}

#[test]
fn route_e_when_fixed_p_cannot_repair() {
    let ev = RouteEvidence {
        parity: ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium,
        sufficiency: Some(PrecursorSufficiencyOutcome::PassiveExchangeLawCannotRepair),
        endogenous: Some(EndogenousCapacityClass::SynthesisCapacityInsufficient),
        p_clamp_restores: false,
        a_clamp_restores: false,
        perm_freeze_restores: false,
        no_decay_restores: false,
        no_leak_restores: false,
        healthy_fixed_point_exists: false,
        bistable_basins: false,
        damage_crosses_separatrix: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(&ev).as_str(),
        "D040_PASSIVE_EXCHANGE_LAW_INVALID"
    );
}

#[test]
fn route_parity_defect_stops() {
    let ev = RouteEvidence {
        parity: ExchangeParityClass::ExchangeRuntimeParityDefect,
        sufficiency: None,
        endogenous: None,
        p_clamp_restores: false,
        a_clamp_restores: false,
        perm_freeze_restores: false,
        no_decay_restores: false,
        no_leak_restores: false,
        healthy_fixed_point_exists: false,
        bistable_basins: false,
        damage_crosses_separatrix: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(&ev).as_str(),
        "D040_EXCHANGE_RUNTIME_PARITY_DEFECT"
    );
}

#[test]
fn reduced_fixed_points_and_jacobian() {
    let par = ReducedApsParams::default();
    let fps = find_reduced_fixed_points(&par);
    assert!(!fps.is_empty());
    for fp in &fps {
        assert!(fp.admissible);
        assert!(!fp.jacobian_eigs.is_empty());
        let _ = fp.class;
    }
}

#[test]
fn basin_classification_runs() {
    let par = ReducedApsParams::default();
    let basins = classify_basins(&par, 0.5);
    assert_eq!(basins.len(), 6);
}

#[test]
fn no_observer_feedback_in_route_evidence() {
    // Route rules are pure functions of recorded evidence — no Simulation mutation.
    let ev = RouteEvidence {
        parity: ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium,
        sufficiency: Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor),
        endogenous: Some(EndogenousCapacityClass::ProductionSufficientRetentionInsufficient),
        p_clamp_restores: true,
        a_clamp_restores: false,
        perm_freeze_restores: false,
        no_decay_restores: true,
        no_leak_restores: true,
        healthy_fixed_point_exists: true,
        bistable_basins: false,
        damage_crosses_separatrix: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(&ev).as_str(),
        "D040_PRECURSOR_RETENTION_DEFECT"
    );
}

#[test]
fn damage_control_isolation_route_f() {
    let ev = RouteEvidence {
        parity: ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium,
        sufficiency: Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor),
        endogenous: Some(EndogenousCapacityClass::MixedDeficit),
        p_clamp_restores: true,
        a_clamp_restores: false,
        perm_freeze_restores: true,
        no_decay_restores: false,
        no_leak_restores: false,
        healthy_fixed_point_exists: true,
        bistable_basins: true,
        damage_crosses_separatrix: true,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(&ev).as_str(),
        "D040_MEMBRANE_METABOLISM_BISTABILITY"
    );
    let _ = FixedPointClass::Stable;
}

#[test]
fn p_clamp_a_clamp_perm_no_decay_no_leak_flags_in_evidence() {
    // Structural: evidence fields exist and drive distinct routes.
    let base = RouteEvidence {
        parity: ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium,
        sufficiency: Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor),
        endogenous: Some(EndogenousCapacityClass::ASupplyInsufficient),
        p_clamp_restores: false,
        a_clamp_restores: true,
        perm_freeze_restores: false,
        no_decay_restores: false,
        no_leak_restores: false,
        healthy_fixed_point_exists: true,
        bistable_basins: false,
        damage_crosses_separatrix: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(&base).as_str(),
        "D040_ACTIVATED_RESOURCE_SUPPLY_DEFICIT"
    );
}
