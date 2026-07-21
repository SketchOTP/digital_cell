//! Focused D-053 unit coverage for combined resource-delivery repair.

use chemistry_core::config::SimParams;
use chemistry_core::d053_analysis::*;
use chemistry_core::membrane_transport::{
    exterior_face_weight, face_diffusivity, face_flux, TransportSpecies,
};

#[test]
fn d052_preservation_constants() {
    assert_eq!(D053_STARTING_COMMIT, "20c5d50");
    assert_eq!(D053_STARTING_TAG, "D-052-resource-delivery-resistance-audit");
    assert_eq!(D053_FROZEN_D052, "D052_MIXED_RESOURCE_DELIVERY_LIMIT");
    assert_eq!(D053_AUTHORIZATION, "MIXED_NF_DELIVERY_REPAIR_AUTHORIZED");
    assert!((D053_FITTED_V_A - 0.12544510052968755).abs() < 1e-15);
}

#[test]
fn exterior_only_face_identification() {
    assert!(is_exterior_exterior_face(0.1, 0.2));
    assert!(!is_exterior_exterior_face(0.1, 0.8));
    assert!(!is_exterior_exterior_face(0.8, 0.9));
    assert!((exterior_face_weight(0.1, 0.2) - 1.0).abs() < 1e-15);
    assert!(exterior_face_weight(0.1, 0.8).abs() < 1e-15);
}

#[test]
fn membrane_only_and_nf_coscaling() {
    let mut p = SimParams::default();
    p.m_beta = 0.8;
    assert!((effective_beta_n(&p) - p.beta_n * 0.8).abs() < 1e-15);
    assert!((effective_beta_f(&p) - p.beta_f * 0.8).abs() < 1e-15);
    assert!((effective_beta_n(&p) - effective_beta_f(&p)).abs() < 1e-15);
}

#[test]
fn caw_invariance_under_repair() {
    let base = SimParams::default();
    let mut repaired = base.clone();
    apply_delivery_repair(
        &mut repaired,
        DeliveryRepairPair {
            m_ext: 2.5,
            m_beta: 0.7,
        },
    );
    for species in [
        TransportSpecies::Catalyst,
        TransportSpecies::Activated,
        TransportSpecies::Waste,
    ] {
        let d0 = face_diffusivity(species, 0.2, 0.8, 1.0, 1.0, &base);
        let d1 = face_diffusivity(species, 0.2, 0.8, 1.0, 1.0, &repaired);
        assert!(
            (d0 - d1).abs() < 1e-15,
            "{species:?} changed under N/F repair"
        );
    }
}

#[test]
fn symmetric_face_conductance() {
    let mut p = SimParams::default();
    p.m_ext = 3.0;
    p.m_beta = 0.8;
    let f = face_flux(TransportSpecies::Nutrient, 1.0, 0.0, 0.1, 0.2, 0.0, 0.0, &p);
    let r = face_flux(TransportSpecies::Nutrient, 0.0, 1.0, 0.2, 0.1, 0.0, 0.0, &p);
    assert!((f + r).abs() < 1e-15);
}

#[test]
fn transport_isolation_proof() {
    let report = prove_transport_isolation(&SimParams::default());
    assert!(report.pass(), "{report:?}");
}

#[test]
fn local_sensitivity_and_candidate_limits() {
    let y0 = [1.0, 1.0, 1.0, 1.0];
    let y_ep = [1.2, 1.2, 1.1, 1.05];
    let y_em = [0.85, 0.85, 0.92, 0.96];
    let y_bp = [1.15, 1.15, 1.2, 1.1];
    let y_bm = [0.9, 0.9, 0.88, 0.94];
    let sens = sensitivity_from_observations(y0, y_ep, y_em, y_bp, y_bm, 0.1, 0.1);
    assert!(sens.both_columns_measurable);
    assert!(sens.rank >= 1);
    let pred = predict_min_pair(&sens, 0.2_f64.ln().abs());
    let cands = build_candidate_set(pred, 1.2, 1.2);
    assert!(!cands.is_empty());
    assert!(cands.len() <= D053_MAX_CANDIDATES);
    for c in &cands {
        assert!(pair_stage_a_authorized(c.pair, 1.2, 1.2));
    }
}

#[test]
fn minimum_change_selection() {
    let cands = vec![
        RepairCandidate {
            name: "big".into(),
            pair: DeliveryRepairPair {
                m_ext: 3.0,
                m_beta: 0.7,
            },
            pi_n: 0.3,
            pi_f: 0.3,
            parent: "p".into(),
            justification: "j".into(),
        },
        RepairCandidate {
            name: "small".into(),
            pair: DeliveryRepairPair {
                m_ext: 1.5,
                m_beta: 0.9,
            },
            pi_n: 0.3,
            pi_f: 0.3,
            parent: "p".into(),
            justification: "j".into(),
        },
    ];
    let sel = select_minimum_change(&cands).unwrap();
    assert_eq!(sel.name, "small");
}

#[test]
fn stage_a_permeability_bands() {
    assert!(stage_a_nf_band_ok(nf_permeability_normalized(1.2, 1.0)));
    assert!(!stage_a_nf_band_ok(0.10));
    assert!(!stage_a_nf_band_ok(0.60));
}

#[test]
fn positivity_rejection_and_chi_helpers() {
    assert!(chi_supply(2.1, 2.0) >= D053_CHI_MIN);
    assert!(resistance_fractions_within_tol(0.43, 0.37, D053_RESISTANCE_TOL));
    assert!(!resistance_fractions_within_tol(0.10, 0.10, 0.05));
}

#[test]
fn conclusion_labels() {
    assert_eq!(
        D053PrimaryConclusion::StageERecovered.as_str(),
        "D053_STAGE_E_RECOVERED"
    );
    assert_eq!(
        D053PrimaryConclusion::BoundedDeliveryRepairNotFound.as_str(),
        "D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND"
    );
}

#[test]
fn interior_diffusion_unaffected_by_m_ext() {
    let base = SimParams::default();
    let mut p = base.clone();
    p.m_ext = 4.0;
    let d0 = face_diffusivity(TransportSpecies::Nutrient, 0.7, 0.9, 0.0, 0.0, &base);
    let d1 = face_diffusivity(TransportSpecies::Nutrient, 0.7, 0.9, 0.0, 0.0, &p);
    assert!((d0 - d1).abs() < 1e-15);
}
