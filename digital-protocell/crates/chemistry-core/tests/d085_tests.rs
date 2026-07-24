//! Focused D-085 unit tests.

use chemistry_core::config::SimParams;
use chemistry_core::d085_analysis::*;
use chemistry_core::structural_kinetics::{
    apply_mechano_loss, apply_mechano_production, apply_mixed_turnover_params,
};

#[test]
fn preservation_candidate_identity_and_restoring() {
    assert!(verify_d084_candidate_identity());
    assert!(prescribed_restoring_ok());
    let p = SimParams::default();
    assert!(!p.use_mixed_structure_turnover);
    let report = gate_preservation(true);
    assert!(report.pass);
    assert_eq!(report.pending_record, D084_FIXED_RADIUS_PENDING);
}

#[test]
fn mixed_default_remains_off() {
    let p = SimParams::default();
    assert!(!p.use_mixed_structure_turnover);
    assert!(!p.use_mechanochemical_structure);
}

#[test]
fn mechano_bounded_response_and_zero_strain() {
    assert!((f_s(0.0, 1.0)).abs() < 1e-15);
    assert!((loss_multiplier(0.0, 0.8) - 1.0).abs() < 1e-15);
    let m = production_multiplier(1.0, 10.0, 5.0, 5.0);
    assert!(m >= 0.5 - 1e-12 && m <= 2.0 + 1e-12);
    let mut p = SimParams::default();
    apply_mixed_turnover_params(&mut p, D085_D084_ETA, D085_D084_K_PHI_LOSS);
    apply_mechano_params(
        &mut p,
        &MechanoCandidate {
            label: "center",
            g_kappa: 0.7,
            g_s: 0.7,
            k_kappa: 1.0,
            k_s: 1.0,
        },
    );
    let r0 = 1.0;
    let r_prod = apply_mechano_production(r0, 1.0, 0.0, &p);
    let r_loss = apply_mechano_loss(r0, 0.0, &p);
    assert!((r_loss - 1.0).abs() < 1e-12);
    assert!(r_prod > 1.0); // curvature supports rebuilding
}

#[test]
fn basin_cohort_requires_seed_majority_and_agreement() {
    let mut rows = Vec::new();
    for seed in 1..=5u64 {
        rows.push(DynamicRunRow {
            radius_seed: 22.0,
            noise_seed: seed,
            equivalent_radius: 22.0 + 0.1 * seed as f64,
            structural_mass: 1500.0,
            c_mass: 100.0,
            a_mass: 50.0,
            l_mass: 10.0,
            b_mass: 20.0,
            w_mass: 5.0,
            structural_production: 1.0,
            structural_loss: 1.0,
            radius_velocity: 1e-5,
            edge_coverage: 0.95,
            ghost_fraction: 0.05,
            trailing_ok: true,
            c_retention: 0.9,
            a_retention: 0.9,
            accepted: 15_000,
            accepted_time: 10.0,
            termination: TerminationKind::ThreeConvergedWindows,
            steps_ok: true,
            accounting_ok: true,
            fragmented: false,
            dish_contact: false,
            exhausted: false,
            clipped: false,
            window_converged: true,
            runtime_structural_net: 0.0,
            frozen_structural_net: 0.0,
            parity_ok: true,
        });
    }
    let cohort = classify_radius_cohort(22.0, &rows);
    assert!(cohort.pass);
    assert_eq!(cohort.seeds_pass, 5);
}

#[test]
fn failure_class_parity_first() {
    let rows = vec![DynamicRunRow {
        radius_seed: 22.0,
        noise_seed: 1,
        equivalent_radius: 30.0,
        structural_mass: 1.0,
        c_mass: 1.0,
        a_mass: 1.0,
        l_mass: 0.0,
        b_mass: 0.0,
        w_mass: 0.0,
        structural_production: 0.0,
        structural_loss: 0.0,
        radius_velocity: 0.01,
        edge_coverage: 0.95,
        ghost_fraction: 0.0,
        trailing_ok: true,
        c_retention: 0.9,
        a_retention: 0.9,
        accepted: 100,
        accepted_time: 1.0,
        termination: TerminationKind::MaxHorizon,
        steps_ok: true,
        accounting_ok: true,
        fragmented: false,
        dish_contact: false,
        exhausted: false,
        clipped: false,
        window_converged: false,
        runtime_structural_net: 0.1,
        frozen_structural_net: -0.1,
        parity_ok: false,
    }];
    let cohort = classify_radius_cohort(22.0, &rows);
    let class = classify_dynamic_failure(&[cohort], false);
    assert_eq!(class, DynamicFailureClass::StaticDynamicParityDefect);
}

#[test]
fn parity_direction_helper() {
    assert!(parity_direction_agrees(0.2, 0.1, 1e-3));
    assert!(!parity_direction_agrees(0.2, -0.1, 1e-3));
}
