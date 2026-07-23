//! D-081 edge-membrane reserve causality focused tests.

use chemistry_core::d079_analysis::SEED_DENSITY;
use chemistry_core::d080_analysis::frozen_d079_params;
use chemistry_core::d081_analysis::*;
use chemistry_core::edge_membrane::*;
use chemistry_core::edge_support::*;

#[test]
fn ids_and_preservation_anchors() {
    assert_eq!(D081_STARTING_TAG, "D-080-edge-network-requalification-fail");
    assert_eq!(D080_GATE7_PROVISIONAL, "PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT");
    assert_eq!(SEED_CONTRACT_V1, "EDGE_MEMBRANE_SEED_CONTRACT_V1");
    assert!(D081_AGENT_MEMORY_ID.contains("d081"));
    assert_eq!(D080_PRIMARY, "D080_EDGE_NETWORK_REPAIR_OR_CAUSALITY_FAILURE");
}

#[test]
fn membrane_ledger_sums_l_and_b() {
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    let led = ledger(&state, &support);
    assert!((led.m_l - state.total_l()).abs() < 1e-12);
    assert!((led.m_b - state.total_b()).abs() < 1e-12);
    assert!((led.m_mem - led.m_l - led.m_b).abs() < 1e-12);
    assert!(led.interface_measure > 0.0);
    assert!(led.m_b.abs() < 1e-12);
}

#[test]
fn seed_provenance_scales_with_interface() {
    let g1 = gate1_seed_provenance();
    assert_eq!(g1.contract, SEED_CONTRACT_V1);
    assert!(g1.density_consistent, "{g1:?}");
    assert!(g1.rows.iter().all(|r| r.initial_m_b.abs() < 1e-9));
    assert!(g1.rows.iter().all(|r| r.no_completed_b_ring));
    assert!(g1.rows.iter().all(|r| !r.hidden_material));
    assert!(g1.rows.iter().all(|r| r.identity.contains(SEED_CONTRACT_V1)));
    let dens: Vec<_> = g1.rows.iter().map(|r| r.density_per_measure).collect();
    let mean = dens.iter().sum::<f64>() / dens.len() as f64;
    for d in dens {
        assert!((d - mean).abs() / mean < 0.08, "d={d} mean={mean}");
    }
}

#[test]
fn seed_classification_is_finite_reserve_or_explicit_failure() {
    let g1 = gate1_seed_provenance();
    match g1.classification {
        SeedClassification::CapacityValidFiniteReserve => assert!(g1.pass),
        SeedClassification::ExcessReserve
        | SeedClassification::RadiusInconsistent
        | SeedClassification::UnauthorizedMaterial
        | SeedClassification::ProvenanceUnknown => assert!(!g1.pass),
    }
}

#[test]
fn bind_unbind_conserves_m_mem() {
    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    let m0 = state.total_membrane();
    for _ in 0..500 {
        let _ = accepted_step_supported(&mut state, &phi, &support, &params, 0.08, false, 1.0);
    }
    assert!((state.total_membrane() - m0).abs() < 1e-6 * (1.0 + m0));
}

#[test]
fn damage_mass_amount_moves_b_to_w() {
    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    for _ in 0..3000 {
        let _ = accepted_step_supported(&mut state, &phi, &support, &params, 0.08, false, 1.0);
    }
    let b0 = state.total_b();
    let w0 = state.waste;
    let removed = damage_mass_amount(&mut state, &support, 0.1 * b0, &params);
    assert!(removed > 0.0);
    assert!((b0 - state.total_b() - removed).abs() < 1e-9);
    assert!((state.waste - w0 - removed).abs() < 1e-9);
}

#[test]
fn a_to_l_stoichiometry_matches_yield() {
    let params = EdgeMembraneParams {
        k_produce: 0.5,
        yield_l_from_a: 1.0,
        ..frozen_d079_params()
    };
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    state.activated = 2.0;
    let a0 = state.activated;
    let l0 = state.total_l();
    let led = accepted_step_supported(&mut state, &phi, &support, &params, 0.08, true, 1.0);
    let da = a0 - state.activated;
    assert!((led.produce - da * params.yield_l_from_a).abs() < 1e-9);
    assert!((state.total_l() - l0 - led.produce).abs() < 1e-9);
}

#[test]
fn no_a_and_production_knockout_do_not_create_l() {
    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let support = build_cut_cell_support(&phi, w, h);

    let mut na = EdgeMembraneState::new(w, h);
    na.catalyst = 1.0;
    na.activated = 0.0;
    let mut p = params;
    p.k_produce = 0.5;
    let m0 = na.total_membrane();
    let led = accepted_step_supported(&mut na, &phi, &support, &p, 0.08, true, 1.0);
    assert!(led.produce.abs() < 1e-15);
    assert!((na.total_membrane() - m0).abs() < 1e-12);

    let mut ko = EdgeMembraneState::new(w, h);
    ko.catalyst = 1.0;
    ko.activated = 5.0;
    p.k_produce = 0.0;
    let m1 = ko.total_membrane();
    let led2 = accepted_step_supported(&mut ko, &phi, &support, &p, 0.08, true, 1.0);
    assert!(led2.produce.abs() < 1e-15);
    assert!((ko.total_membrane() - m1).abs() < 1e-12);
}

#[test]
fn route_prefixes() {
    assert!(D081Route::EdgeReserveCausalityQualified
        .conclusion()
        .starts_with("D081_"));
    assert!(D081Route::Fail.conclusion().starts_with("D081_"));
}

#[test]
fn gate0_report_fields_present_without_full_pipeline() {
    // Full Gate0 reproduction is exercised by the release pipeline (too heavy for debug unit tests).
    assert_eq!(D080_GATE7_PROVISIONAL, "PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT");
    assert!(D081Route::D080ResultNotReproduced
        .conclusion()
        .contains("D080_RESULT_NOT_REPRODUCED"));
}
