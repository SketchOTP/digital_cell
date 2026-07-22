//! D-067 law-identification gates stay observer-only.

use chemistry_core::activated_metabolism::activation_isolated_delta;
use chemistry_core::d066_analysis::{activation_stoichiometry_parity, ALedger066};
use chemistry_core::d067_analysis::*;

fn qualified() -> IdentificationReport {
    IdentificationReport {
        params_positive_finite: true,
        half_sats_in_domain: true,
        bootstrap_spread: 0.25,
        loo_variation: 1.25,
        holdout_median_err: 0.15,
        holdout_max_err: 0.25,
        holdout_balance_sign_acc: 0.95,
        no_radius_params: true,
        stoichiometry_ok: true,
        accounting_ok: true,
    }
}

fn evidence() -> RouteEvidence067 {
    RouteEvidence067 {
        workspace_isolated: true,
        d066_reproduced: true,
        substrate_lineage_ok: true,
        runtime_parity_ok: true,
        a_w_accounting_ok: true,
        safety_causality_ok: true,
        identification: qualified(),
        waste_blocks_qualification: false,
        existing_law_qualified: false,
        global_scale_qualified: false,
        low_substrate_response_qualified: false,
        activation_repaired_stage_e_blocked: false,
        precursor_demand_primary: false,
        no_portable_law: false,
    }
}

#[test]
fn d066_reproduction_requires_capacity_pattern() {
    assert!(d066_reproduction_predicate(1.05, 0.36, 0.81, 0.36, 0.12));
    assert!(!d066_reproduction_predicate(1.04, 0.36, 0.81, 0.36, 0.12));
}

#[test]
fn normalized_lineage_is_unclipped_and_quadratic() {
    assert_eq!(n_hat(2.0, 1.0), 2.0);
    assert_eq!(f_hat(3.0, 1.0), 3.0);
    assert_eq!(product_n_f_hat(2.0, 3.0, 1.0, 1.0), 6.0);
    assert_eq!(
        classify_substrate_response(0.3, 0.3, 0.09, 1.0, true),
        SubstrateResponseClass::OrdinaryResponseLinearLow
    );
    assert!(!baseline_equivalent_to_michaelis(1.0, 1.0));
}

#[test]
fn required_multiplier_and_portability_are_explicit() {
    assert_eq!(g_a_required(2.0, 3.0, 4.0, 1.0, 1.0), 9.0);
    assert_eq!(m_a_star(9.0, 3.0), 3.0);
    assert!(multiplier_portable(&[1.0, 2.0, 3.0], PORTABLE_SPAN_MAX));
    assert!(!multiplier_portable(&[1.0, 3.01], PORTABLE_SPAN_MAX));
}

#[test]
fn high_resource_ceiling_detects_headroom_and_risk() {
    assert_eq!(
        classify_high_resource_ceiling(0.36, 0.90, 0.90, A_RETENTION, false),
        HighResourceCeilingClass::HighResourceCeilingHasHeadroom
    );
    assert_eq!(
        classify_high_resource_ceiling(0.36, 0.90, 0.90, A_RETENTION, true),
        HighResourceCeilingClass::HighResourceOverproductionRisk
    );
}

#[test]
fn bounded_nf_differs_from_linear_baseline_and_starves_exactly() {
    let baseline = candidate_a_rate(1.0, 1.0, 1.0, 5.0, 5.0, 0.1, 1.0, 1.0);
    let bounded = candidate_c_rate(1.0, 1.0, 1.0, 5.0, 5.0, 0.1, 0.2, 0.2);
    assert!(baseline > bounded);
    assert!(zero_activation_when_starved(|c, n, f| {
        candidate_c_rate(D067_V_A, 1.0, c, n, f, D067_K_C, 0.2, 0.2)
    }));
}

#[test]
fn activation_stoichiometry_and_a_ledger_close() {
    assert!(activation_stoichiometry_parity(1.0));
    assert_eq!(activation_isolated_delta(1.0)[4], 1.0);
    let ledger = ALedger066 {
        g_activation: 10.0, l_catalyst: 1.0, l_structure: 1.0, l_precursor: 6.0,
        l_decay: 1.0, j_out: 1.0, j_in: 0.0, delta_a: 0.0,
        activation_requested: 10.0, activation_accepted: 10.0, j_n_net: 1.0, j_f_net: 1.0,
    };
    assert!(ledger.closes(1e-12));
}

#[test]
fn identification_enforces_bootstrap_and_holdout_thresholds() {
    assert!(qualified().qualifies());
    let mut failed = qualified();
    failed.bootstrap_spread = BOOTSTRAP_SPREAD_MAX + 0.01;
    assert!(!failed.qualifies());
    failed = qualified();
    failed.loo_variation = LOO_MAX + 0.01;
    assert!(!failed.qualifies());
    failed = qualified();
    failed.holdout_max_err = HOLDOUT_MAX_ERR + 0.01;
    assert!(!failed.qualifies());
}

#[test]
fn routes_prioritize_workspace_waste_then_candidates() {
    let mut ev = evidence();
    ev.workspace_isolated = false;
    assert_eq!(select_route(ev).1, D067PrimaryConclusion::WorkspaceScopeNotIsolated);
    ev = evidence();
    ev.waste_blocks_qualification = true;
    assert_eq!(select_route(ev).0, D067Route::W);
    ev.waste_blocks_qualification = false;
    ev.global_scale_qualified = true;
    assert_eq!(select_route(ev).0, D067Route::V);
    ev.global_scale_qualified = false;
    ev.low_substrate_response_qualified = true;
    assert_eq!(select_route(ev).0, D067Route::R);
    ev.low_substrate_response_qualified = false;
    ev.existing_law_qualified = true;
    assert_eq!(select_route(ev).0, D067Route::E);
}

#[test]
fn no_portable_law_precedes_demand_counterfactual() {
    let mut ev = evidence();
    ev.no_portable_law = true;
    ev.precursor_demand_primary = true;
    assert_eq!(select_route(ev).0, D067Route::N);
}

#[test]
fn demand_and_safety_flags_remain_conservative() {
    assert_eq!(
        classify_demand_counterfactual(false, true),
        DemandCounterfactualClass::PrecursorDemandPrimary
    );
    assert!(zero_activation_when_starved(|c, n, f| {
        candidate_c_rate(1.0, 1.0, c, n, f, 0.1, 1.0, 1.0)
    }));
}
