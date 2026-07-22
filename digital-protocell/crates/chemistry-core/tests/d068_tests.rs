//! D-068 precursor/membrane audit gates stay observer-only.

use chemistry_core::d068_analysis::*;

fn qualified() -> IdentificationReport068 {
    IdentificationReport068 {
        params_positive_finite: true,
        half_sats_in_domain: true,
        bootstrap_spread: 0.25,
        loo_variation: 1.25,
        holdout_median_err: 0.15,
        holdout_max_err: 0.25,
        holdout_a_sign_acc: 0.95,
        holdout_s_sign_acc: 0.95,
        no_radius_params: true,
        stoichiometry_ok: true,
        accounting_ok: true,
    }
}

fn evidence() -> RouteEvidence068 {
    RouteEvidence068 {
        workspace_isolated: true,
        d067_reproduced: true,
        lineage_ok: true,
        runtime_parity_ok: true,
        ledger_ok: true,
        safety_causality_ok: true,
        waste_blocks: false,
        identification: qualified(),
        existing_qualified: false,
        overproduction_qualified: false,
        inhibition_qualified: false,
        assembly_limit: false,
        desorption_dominant: false,
        membrane_access_limit: false,
        multiple_limits: false,
        repair_but_stage_e_blocked: false,
        no_portable_repair: false,
    }
}

#[test]
fn d067_reproduction_requires_capacity_and_precursor_sink() {
    assert!(d067_reproduction_predicate(1.27, 0.355, 1.81, 0.117, 0.76));
    assert!(!d067_reproduction_predicate(1.04, 0.355, 1.81, 0.117, 0.76));
}

#[test]
fn precursor_stoichiometry_is_a_to_p_without_w() {
    assert!(precursor_synthesis_parity(1.0));
    assert_eq!(NU_W_SYN, 0.0);
    let lin = frozen_lineage();
    assert!(lineage_resolved(&lin));
    assert!(!baseline_has_product_inhibition());
}

#[test]
fn exchange_parity_is_mass_conserving() {
    assert!(adsorption_parity(2.5));
    assert!(desorption_parity(0.75));
}

#[test]
fn apsw_ledgers_close_when_balanced() {
    let a = ALedger068 {
        g_activation: 10.0,
        l_catalyst: 1.0,
        l_structure: 1.0,
        l_precursor: 7.0,
        l_decay: 1.0,
        j_net: 0.0,
        delta_a: 0.0,
    };
    assert!(a.closes(1e-12));
    let p = PLedger068 {
        g_synthesis: 8.0,
        g_desorption: 2.0,
        l_adsorption: 5.0,
        l_decay: 1.0,
        j_net: 0.0,
        delta_p: 4.0,
    };
    assert!(p.closes(1e-12));
    let s = SLedger068 {
        g_adsorption: 5.0,
        l_desorption: 2.0,
        l_damage: 0.0,
        j_net: 0.0,
        delta_s: 3.0,
    };
    assert!(s.closes(1e-12));
}

#[test]
fn precursor_utility_and_replacement_metrics() {
    assert!((eta_p_to_s(4.0, 8.0) - 0.5).abs() < 1e-12);
    assert!((futile_fraction(4.0, 8.0) - 0.5).abs() < 1e-12);
    assert!((chi_s(5.0, 4.0, 1.0) - 1.0).abs() < 1e-9);
    assert_eq!(
        classify_replacement_demand(2.0, 0.5, true),
        ReplacementDemandClass::PrecursorOverproduction
    );
    assert_eq!(
        classify_replacement_demand(0.5, 0.5, true),
        ReplacementDemandClass::PrecursorProductionInsufficient
    );
}

#[test]
fn assembly_and_access_classification() {
    assert_eq!(
        classify_assembly_capacity(true, 2.0, 1.0, false, true, true),
        AssemblyCapacityClass::PToSAssemblyCapacityAdequate
    );
    assert_eq!(
        classify_assembly_capacity(false, 0.5, 2.0, false, false, true),
        AssemblyCapacityClass::SDesorptionDominant
    );
    assert_eq!(
        classify_assembly_capacity(false, 1.0, 1.0, true, false, true),
        AssemblyCapacityClass::PMembraneAccessLimit
    );
}

#[test]
fn fate_classification_covers_incorporation_and_futile() {
    assert_eq!(
        classify_precursor_fate(10.0, 9.0, 0.0, 0.5, 0.0, 1.0),
        PrecursorFateClass::PrecursorProductivelyIncorporated
    );
    assert_eq!(
        classify_precursor_fate(10.0, 2.0, 5.0, 0.5, 0.0, 0.5),
        PrecursorFateClass::PrecursorAccumulation
    );
}

#[test]
fn zero_a_blocks_precursor_and_c_is_distinct() {
    assert!(zero_precursor_when_a_starved(|c, phi, p| {
        let _ = p;
        precursor_rate(0.2, 0.0, c, phi, 0.1)
    }));
    assert!(candidate_c_distinct_from_baseline(0.1, 1.0));
    assert!(candidate_c_rate(0.2, 1.0, 1.0, 1.0, 0.1, 10.0, 0.1) < precursor_rate(0.2, 1.0, 1.0, 1.0, 0.1));
}

#[test]
fn route_selection_priorities() {
    let mut ev = evidence();
    ev.workspace_isolated = false;
    assert_eq!(
        select_route(ev.clone()).1,
        D068PrimaryConclusion::WorkspaceScopeNotIsolated
    );
    ev = evidence();
    ev.waste_blocks = true;
    assert_eq!(select_route(ev.clone()).0, D068Route::W);
    ev.waste_blocks = false;
    ev.overproduction_qualified = true;
    assert_eq!(select_route(ev.clone()).0, D068Route::O);
    ev.overproduction_qualified = false;
    ev.desorption_dominant = true;
    assert_eq!(select_route(ev.clone()).0, D068Route::S);
    ev.desorption_dominant = false;
    ev.assembly_limit = true;
    assert_eq!(select_route(ev.clone()).0, D068Route::A);
    ev.assembly_limit = false;
    ev.membrane_access_limit = true;
    assert_eq!(select_route(ev.clone()).0, D068Route::P);
    ev.membrane_access_limit = false;
    ev.no_portable_repair = true;
    assert_eq!(select_route(ev).0, D068Route::N);
}

#[test]
fn identification_enforces_thresholds() {
    assert!(qualified().qualifies());
    let mut failed = qualified();
    failed.bootstrap_spread = BOOTSTRAP_SPREAD_MAX + 0.01;
    assert!(!failed.qualifies());
}

#[test]
fn preregistered_m_p_bounded() {
    let ms = preregistered_m_p(4.0);
    assert!(ms.len() <= 5);
    assert!(ms.iter().all(|m| *m >= 0.0 && *m <= 1.0));
}
