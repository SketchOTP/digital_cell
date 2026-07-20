//! D-049 focused tests: D-048 completeness, ledgers, chronology, routes, empirical model.

use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d048_analysis::D048_HISTORICAL_K;
use chemistry_core::d049_analysis::{
    classify_d048_completeness, classify_frozen_membrane, disposition_d040, earliest_causal_event,
    empirical_rhs, find_empirical_fixed_points, has_physical_healthy_fp, ledger_closes,
    select_route, d049_frozen_params, ChronologySample, D048BranchClass, D040ModelDisposition,
    D049EarliestCause, D049Route, EmpiricalReducedParams, RouteEvidence,
    D049_BOOTSTRAP_P, D049_LEDGER_REL_TOL, D049_RECORD, D049_RETENTION_MIN, MEMBRANE_TO_A_FEEDBACK_REQUIRED,
    UPSTREAM_OF_MEMBRANE,
};

fn base_evidence() -> RouteEvidence {
    RouteEvidence {
        numerical_ok: true,
        accounting_ok: true,
        coupled_ledger_ok: true,
        d048_evidence_complete: true,
        analytic_collapses: false,
        restored_survives: false,
        healthy_perm_prevents_collapse: false,
        no_outward_a_prevents_collapse: false,
        precursor_demand_removal_prevents_a_collapse: false,
        p_production_ok: false,
        p_decay_or_leak_keeps_p_low: false,
        exchange_parity_ok: false,
        no_healthy_endogenous_fp: false,
        a_still_deficient_under_controlled_p: false,
        empirical_no_physical_healthy_fp: false,
    }
}

#[test]
fn d048_missing_restored_is_incomplete() {
    let r = classify_d048_completeness(true, true, true, false, false, false, false);
    assert_eq!(r.branch_class, D048BranchClass::RestoredBranchMissing);
    assert!(!r.pass_gate0_complete);
}

#[test]
fn d048_both_collapse_after_both_run() {
    let r = classify_d048_completeness(true, true, true, false, true, false, true);
    assert!(r.pass_gate0_complete);
    assert_eq!(r.branch_class, D048BranchClass::AnalyticSeedCollapse);
    assert!(r.both_branches_collapsed());
}

#[test]
fn d048_restored_survives_analytic_fails() {
    let r = classify_d048_completeness(true, true, true, false, true, true, true);
    assert_eq!(r.branch_class, D048BranchClass::RestoredHealthySurvives);
    assert!(!r.both_branches_collapsed());
}

#[test]
fn ledger_closes_tolerance() {
    assert!(ledger_closes(10.0, 10.4, D049_LEDGER_REL_TOL));
    assert!(ledger_closes(-2.0, -2.09, D049_LEDGER_REL_TOL));
    assert!(!ledger_closes(10.0, 11.0, D049_LEDGER_REL_TOL));
    assert!(ledger_closes(0.0, 0.0, D049_LEDGER_REL_TOL));
}

#[test]
fn earliest_event_a_leakage_precedes_others() {
    let samples = vec![
        ChronologySample {
            index: 0,
            a_retention: 0.95,
            a_production: 1.0,
            a_leakage: 0.1,
            a_productive_demand: 1.0,
            p_synthesis: 1.0,
            p_leakage: 0.1,
            p_decay: 0.01,
            adsorption: 1.0,
            desorption: 0.5,
            s_occupancy: 0.7,
            permeability_proxy: 1.0,
            c_retention: 0.95,
            n_influx: 1.0,
            f_influx: 1.0,
        },
        ChronologySample {
            index: 1,
            a_retention: 0.90,
            a_production: 1.0,
            a_leakage: 0.25,
            a_productive_demand: 1.0,
            p_synthesis: 1.0,
            p_leakage: 0.1,
            p_decay: 0.01,
            adsorption: 1.0,
            desorption: 0.5,
            s_occupancy: 0.69,
            permeability_proxy: 1.0,
            c_retention: 0.95,
            n_influx: 1.0,
            f_influx: 1.0,
        },
        ChronologySample {
            index: 2,
            a_retention: 0.50,
            a_production: 0.5,
            a_leakage: 0.5,
            a_productive_demand: 1.0,
            p_synthesis: 0.2,
            p_leakage: 0.5,
            p_decay: 0.5,
            adsorption: 0.5,
            desorption: 2.0,
            s_occupancy: 0.4,
            permeability_proxy: 3.0,
            c_retention: 0.5,
            n_influx: 1.0,
            f_influx: 1.0,
        },
    ];
    assert_eq!(
        earliest_causal_event(&samples),
        D049EarliestCause::ATransportLeakageOnset
    );
    assert_eq!(
        earliest_causal_event(&samples).as_str(),
        "A_TRANSPORT_LEAKAGE_ONSET"
    );
}

#[test]
fn frozen_membrane_classification() {
    assert_eq!(
        classify_frozen_membrane(0.70, true, true, true),
        MEMBRANE_TO_A_FEEDBACK_REQUIRED
    );
    assert_eq!(
        classify_frozen_membrane(0.90, true, true, true),
        UPSTREAM_OF_MEMBRANE
    );
    assert_eq!(
        classify_frozen_membrane(0.70, false, true, true),
        UPSTREAM_OF_MEMBRANE
    );
}

#[test]
fn route_basin_inaccessible() {
    let mut ev = base_evidence();
    ev.analytic_collapses = true;
    ev.restored_survives = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::B);
    assert_eq!(
        conclusion.as_str(),
        "D049_HEALTHY_ATTRACTOR_BASIN_INACCESSIBLE"
    );
}

#[test]
fn route_a_leakage_membrane_feedback() {
    let mut ev = base_evidence();
    ev.healthy_perm_prevents_collapse = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::L);
    assert_eq!(conclusion.as_str(), "D049_A_LEAKAGE_MEMBRANE_FEEDBACK");
}

#[test]
fn route_precursor_demand_regulation() {
    let mut ev = base_evidence();
    ev.precursor_demand_removal_prevents_a_collapse = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::P);
    assert_eq!(
        conclusion.as_str(),
        "D049_PRECURSOR_DEMAND_REGULATION_FAILURE"
    );
}

#[test]
fn route_precursor_retention() {
    let mut ev = base_evidence();
    ev.p_production_ok = true;
    ev.p_decay_or_leak_keeps_p_low = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::R);
    assert_eq!(conclusion.as_str(), "D049_PRECURSOR_RETENTION_FAILURE");
}

#[test]
fn route_coupled_activation_capacity() {
    let mut ev = base_evidence();
    ev.a_still_deficient_under_controlled_p = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::A);
    assert_eq!(
        conclusion.as_str(),
        "D049_COUPLED_ACTIVATION_CAPACITY_FAILURE"
    );
}

#[test]
fn route_no_physical_fixed_point() {
    let mut ev = base_evidence();
    ev.empirical_no_physical_healthy_fp = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::N);
    assert_eq!(
        conclusion.as_str(),
        "D049_NO_PHYSICAL_MEMBRANE_METABOLISM_FIXED_POINT"
    );
}

#[test]
fn route_inconclusive() {
    let ev = base_evidence();
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::I);
    assert_eq!(
        conclusion.as_str(),
        "D049_APS_COLLAPSE_DECOMPOSITION_INCONCLUSIVE"
    );
}

#[test]
fn route_endogenous_exchange_equilibrium() {
    let mut ev = base_evidence();
    ev.exchange_parity_ok = true;
    ev.no_healthy_endogenous_fp = true;
    let (route, conclusion) = select_route(&ev);
    assert_eq!(route, D049Route::S);
    assert_eq!(
        conclusion.as_str(),
        "D049_ENDOGENOUS_EXCHANGE_EQUILIBRIUM_FAILURE"
    );
}

#[test]
fn route_numerical_and_accounting_failures_first() {
    let mut ev = base_evidence();
    ev.numerical_ok = false;
    assert_eq!(
        select_route(&ev).1.as_str(),
        "D049_NUMERICAL_FAILURE"
    );
    ev.numerical_ok = true;
    ev.accounting_ok = false;
    assert_eq!(
        select_route(&ev).1.as_str(),
        "D049_ACCOUNTING_FAILURE"
    );
    ev.accounting_ok = true;
    ev.coupled_ledger_ok = false;
    assert_eq!(
        select_route(&ev).1.as_str(),
        "D049_COUPLED_LEDGER_FAILURE"
    );
}

#[test]
fn route_d048_evidence_incomplete() {
    let mut ev = base_evidence();
    ev.d048_evidence_complete = false;
    assert_eq!(
        select_route(&ev).1.as_str(),
        "D049_D048_ATTRACTOR_EVIDENCE_INCOMPLETE"
    );
}

#[test]
fn empirical_rhs_and_fixed_points() {
    let par = EmpiricalReducedParams::default();
    let (da, dp, dtheta) = empirical_rhs(0.5, 0.05, 0.6, &par);
    assert!(da.is_finite() && dp.is_finite() && dtheta.is_finite());
    let fps = find_empirical_fixed_points(&par);
    assert!(!fps.is_empty());
    let _ = has_physical_healthy_fp(&fps, 0.5);
}

#[test]
fn disposition_d040_cases() {
    assert_eq!(
        disposition_d040(false, false, true, false).as_str(),
        "D040_REDUCED_MODEL_VALID"
    );
    assert_eq!(
        disposition_d040(true, false, true, false).as_str(),
        "D040_REDUCED_MODEL_OMITTED_A_LEAKAGE"
    );
    assert_eq!(
        disposition_d040(false, true, true, false).as_str(),
        "D040_REDUCED_MODEL_OMITTED_PRECURSOR_LOAD"
    );
    assert_eq!(
        disposition_d040(false, false, false, false).as_str(),
        "D040_HEALTHY_FIXED_POINT_NOT_PHYSICAL"
    );
    assert_eq!(
        disposition_d040(false, false, true, true).as_str(),
        "D040_REDUCED_MODEL_INVALID_EXTRAPOLATION"
    );
}

#[test]
fn select_route_does_not_mutate_evidence() {
    let ev = base_evidence();
    let ev_clone = ev.clone();
    let _ = select_route(&ev);
    assert_eq!(ev, ev_clone);
}

#[test]
fn frozen_params_k_activation_is_020() {
    let base = v8_schema3_params();
    let frozen = d049_frozen_params(&base);
    assert!((frozen.k_d008_activation - D048_HISTORICAL_K).abs() < 1e-15);
    assert!((frozen.k_d008_activation - 0.020).abs() < 1e-15);
}

#[test]
fn constants_and_record() {
    assert_eq!(
        D049_RECORD,
        "FROZEN_COUPLED_ORGANISM_COLLAPSE_CONFIRMED"
    );
    assert!((D049_BOOTSTRAP_P - 0.060).abs() < 1e-15);
    assert!((D049_RETENTION_MIN - 0.80).abs() < 1e-15);
}

#[test]
fn d040_disposition_enum_matches_directive() {
    assert_eq!(
        D040ModelDisposition::ReducedModelValid.as_str(),
        "D040_REDUCED_MODEL_VALID"
    );
}
