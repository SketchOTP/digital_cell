//! D-048 focused tests: candidate identity, seed contract, basin, replacement, route.

use chemistry_core::config::SurfaceTurnoverSchema;
use chemistry_core::d039_analysis::{
    apply_schema3_exchange_damage_only, v8_schema3_params,
};
use chemistry_core::d048_analysis::{
    audit_governed_seed_contract, build_frozen_candidate_identity, classify_damage_40,
    d048_frozen_organism_params, evaluate_healthy_window, late_state_agrees, select_conclusion,
    select_route, seeded_basin_passes, three_consecutive_qualifying, MacrostateSnapshot,
    D048Conclusion, D048_ARCHITECTURE_PASS, D048_HISTORICAL_K, D048_RECORD_ACTIVATION,
    D048_WINDOW,
};
use chemistry_core::membrane_label_tracer::MembraneLabelTracer;

#[test]
fn immutable_candidate_identity_freezes_historical_activation() {
    let mut base = v8_schema3_params();
    base.k_d008_activation = 0.024; // must be overwritten
    base.k_phi = 0.5;
    let frozen = d048_frozen_organism_params(&base);
    assert!((frozen.k_d008_activation - D048_HISTORICAL_K).abs() < 1e-15);
    assert_eq!(
        frozen.surface_turnover_schema,
        SurfaceTurnoverSchema::ExchangeDamageOnly
    );
    assert!((frozen.rho_a - 1.0).abs() < 1e-15);
    let id = build_frozen_candidate_identity(&frozen);
    assert!((id.k_activation - 0.020).abs() < 1e-15);
    assert_eq!(id.record, D048_RECORD_ACTIVATION);
    assert_eq!(id.constitutive_s_to_w, 0.0);
}

#[test]
fn seed_contract_classifies_permitted_organism_seed() {
    let r = audit_governed_seed_contract(
        22.0, 2.0, 0.6, 1, 0.4, 0.5, 0.05, 0.4, 0.4, 0.5, 1.0, 1.0, true,
    );
    assert!(r.pass);
    assert!(r
        .material_classes
        .iter()
        .any(|(k, v)| k == "catalyst_C" && v == "permitted_organism_seed"));
    assert!(r
        .material_classes
        .iter()
        .any(|(k, v)| k == "reservoir_N_F" && v == "environmental_resource"));
    assert!(!r.forbidden_present);
}

#[test]
fn zero_s_diagnostic_status_does_not_fail_seed_contract() {
    let r = audit_governed_seed_contract(
        22.0, 2.0, 0.6, 1, 0.4, 0.5, 0.05, 0.4, 0.4, 0.5, 1.0, 1.0, false,
    );
    assert!(r.pass);
    assert!(r.zero_s_diagnostic_only);
}

#[test]
fn accepted_step_window_authority_is_10000() {
    assert_eq!(D048_WINDOW, 10_000);
    assert!(!three_consecutive_qualifying(&[true, true]));
    assert!(three_consecutive_qualifying(&[false, true, true, true]));
}

#[test]
fn healthy_attractor_classification_requires_retention_and_flow() {
    let ok = evaluate_healthy_window(
        0.85, 0.85, 0.96, 1e-5, 1.0, 1.0, 1e-3, 1e-3, 1e-3, 1e-3, true, true, true, true, true,
        "",
    );
    assert!(ok.pass());
    let bad_a = evaluate_healthy_window(
        0.85, 0.50, 0.96, 1e-5, 1.0, 1.0, 1e-3, 1e-3, 1e-3, 1e-3, true, true, true, true, true,
        "",
    );
    assert!(!bad_a.pass());
}

#[test]
fn contiguous_basin_selection_thresholds() {
    assert!(seeded_basin_passes(true, 4, 4, 5, 4, 4));
    assert!(!seeded_basin_passes(true, 3, 5, 5, 5, 4));
}

#[test]
fn tracer_conservation_observer_only() {
    let mut t = MembraneLabelTracer::init_from_totals(10.0, 20.0);
    t.pulse_label_all_s_as_old(20.0);
    assert!(t.inventory_residual().abs() <= 1e-12);
    assert!(t.replacement_fraction(20.0) < 1e-12);
}

#[test]
fn molecular_replacement_and_damage_classifiers() {
    assert_eq!(classify_damage_40(0.96, 0.91, 0.96).as_str(), "full_recovery");
    assert_eq!(
        classify_damage_40(0.40, 0.40, 0.70).as_str(),
        "irreversible_failure"
    );
}

#[test]
fn activation_disabled_and_exchange_knockout_route_semantics() {
    // Conclusion ordering: resource dependence after damage pass.
    assert_eq!(
        select_conclusion(
            true, true, true, true, true, true, false, true, true, true, true, true, true
        ),
        D048Conclusion::RepairResourceDependenceFailure
    );
    assert_eq!(
        select_conclusion(
            true, true, true, true, true, true, true, false, true, true, true, true, true
        ),
        D048Conclusion::MembraneCausalityFailure
    );
}

#[test]
fn stage_bcd_and_dynamic_r22_conclusion_mapping() {
    assert_eq!(
        select_conclusion(
            true, true, true, true, true, true, true, true, false, true, true, true, true
        ),
        D048Conclusion::FoundationalRegression
    );
    assert_eq!(
        select_conclusion(
            true, true, true, true, true, true, true, true, true, false, true, true, true
        ),
        D048Conclusion::DynamicMembraneContractFailure
    );
}

#[test]
fn constrained_membrane_contract_and_route_selection() {
    assert_eq!(
        select_route(D048Conclusion::FrozenBiologyMembraneBasinQualified),
        D048_ARCHITECTURE_PASS
    );
    assert_eq!(
        select_route(D048Conclusion::NoHealthyMembraneAttractor),
        "RETURN_TO_MEMBRANE_METABOLISM_COUPLING_FULL_APS_HISTORIES"
    );
    let mut p = v8_schema3_params();
    apply_schema3_exchange_damage_only(&mut p);
    p.k_d008_activation = D048_HISTORICAL_K;
    let id = build_frozen_candidate_identity(&p);
    assert!(id.identity_hash_input.contains("0.020"));
}

#[test]
fn late_state_agreement_for_basin_neighbors() {
    let c = MacrostateSnapshot {
        radius: 22.0,
        structural_mass: 1000.0,
        c_mass: 100.0,
        a_mass: 50.0,
        p_mass: 10.0,
        s_mass: 80.0,
        c_retention: 0.90,
        a_retention: 0.85,
        membrane_occupancy: 0.70,
        localization: 0.99,
    };
    let mut n = c.clone();
    n.s_mass = 85.0;
    assert!(late_state_agrees(&c, &n));
}
