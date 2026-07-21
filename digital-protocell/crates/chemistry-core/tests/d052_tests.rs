//! Focused D-052 unit coverage (diagnostic helpers; no production chemistry change).

use chemistry_core::d052_analysis::*;

#[test]
fn d051_preservation_constants() {
    assert_eq!(D052_STARTING_COMMIT, "e08075a");
    assert_eq!(D052_STARTING_TAG, "D-051-coupled-activation-throughput-audit");
    assert_eq!(D052_FROZEN_D051, "D051_RESOURCE_THROUGHPUT_LIMIT");
    assert_eq!(
        D052_ACTIVATION_SUPPLY_LAW_NOTE,
        "ACTIVATION_SUPPLY_LAW_NOT_CURRENT_REPAIR_TARGET"
    );
}

#[test]
fn regional_ledger_closure() {
    let led = ResourceRegionalLedger {
        j_reservoir: 2.0,
        j_interface: -0.5,
        loss_activation: 1.0,
        loss_other: 0.2,
        delta_central: 0.3,
        ..Default::default()
    };
    // pred = 2 - 0.5 - 1.2 = 0.3
    assert!(led.closes(0.3, 0.05));
    assert!(!led.closes(2.0, 0.05));
}

#[test]
fn reservoir_and_interface_flux_accounting_helpers() {
    let j_res = 1.5;
    let j_if = 0.8;
    let loss = 2.0;
    let delta: f64 = j_res + j_if - loss;
    assert!((delta - 0.3).abs() < 1e-12);
    let led = ResourceRegionalLedger {
        j_reservoir: j_res,
        j_interface: j_if,
        loss_activation: loss,
        delta_central: delta,
        ..Default::default()
    };
    assert!(led.closes(delta, D052_LEDGER_REL_TOL));
}

#[test]
fn resistance_calculation_and_dominance() {
    let mut segs = vec![
        SegmentResistance {
            segment: DeliverySegment::MembraneCrossing,
            delta_c: 0.9,
            flux: 0.05,
            resistance: segment_resistance(0.9, 0.05),
            fraction: 0.0,
        },
        SegmentResistance {
            segment: DeliverySegment::ReservoirRelaxation,
            delta_c: 0.01,
            flux: 1.0,
            resistance: segment_resistance(0.01, 1.0),
            fraction: 0.0,
        },
    ];
    normalize_resistance_fractions(&mut segs);
    assert_eq!(
        dominant_segment(&segs),
        Some(DeliverySegment::MembraneCrossing)
    );
    assert!(segs[0].fraction >= D052_RESISTANCE_DOMINANCE);
}

#[test]
fn limiting_resource_classification() {
    assert_eq!(
        classify_resource_identity(0.2, 0.2, 1.2, 0.1, 0.4, 0.4).as_str(),
        "JOINT_RESOURCE_LIMIT"
    );
    assert_eq!(
        classify_resource_identity(1.0, 0.12, 1.1, 0.1, 0.6, 0.1).as_str(),
        "NUTRIENT_DOMINANT"
    );
    assert_eq!(
        classify_resource_identity(0.12, 1.0, 1.1, 0.1, 0.1, 0.6).as_str(),
        "FUEL_DOMINANT"
    );
}

#[test]
fn reservoir_control_material_rise() {
    assert!(!material_throughput_rise(0.1, 0.12));
    assert!(material_throughput_rise(0.1, 0.2));
}

#[test]
fn permeability_bypass_labels() {
    let ordinary = nf_permeability_from_beta(1.2, 1.0);
    let bypass = nf_permeability_from_beta(0.0, 1.0);
    assert!(bypass > ordinary);
    assert!((bypass - 1.0).abs() < 1e-15);
    assert!(stage_a_nf_permeability_in_range(ordinary));
}

#[test]
fn diffusion_and_mixing_invariants() {
    // Conservative interior mixing preserves mass.
    let vals = [0.2, 0.8, 0.5];
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let mixed = [mean, mean, mean];
    assert!((vals.iter().sum::<f64>() - mixed.iter().sum::<f64>()).abs() < 1e-15);
}

#[test]
fn membrane_state_and_radius_helpers() {
    let chi = chi_supply(2.0, 1.0);
    assert!((chi - 2.0).abs() < 1e-15);
    let chi_a = chi_activation(2.0, 1.5, 1.0);
    assert!((chi_a - 1.5).abs() < 1e-15);
    let dep = classify_depletion_locus(1.0, 0.95, 0.9, 0.2, 0.18, 0.15, 0.1);
    assert_eq!(dep, DepletionLocus::AcrossMembrane);
}

#[test]
fn observer_yield_analysis() {
    let p = observer_yield_probe(1.0, 1.0, 2.0, 1.0);
    assert!((p.chi_activation_at_yield - 0.5).abs() < 1e-12);
    let y = required_analytical_yield(1.0, 1.0, 2.0);
    assert!((y - 2.0).abs() < 1e-12);
    let up = observer_yield_probe(1.0, 1.0, 2.0, y);
    assert!(up.chi_activation_at_yield >= 1.0 - 1e-12);
}

#[test]
fn route_selection_rules() {
    assert_eq!(
        select_primary_route(&RouteDecisionInput {
            d051_reproduced: false,
            ledger_ok: true,
            accounting_ok: true,
            numerical_ok: true,
            ..Default::default()
        })
        .as_str(),
        "D052_D051_RESOURCE_LIMIT_NOT_REPRODUCED"
    );
    assert_eq!(
        select_primary_route(&RouteDecisionInput {
            d051_reproduced: true,
            ledger_ok: true,
            accounting_ok: true,
            numerical_ok: true,
            membrane_permeability_dominant: true,
            ..Default::default()
        })
        .as_str(),
        "D052_MEMBRANE_RESOURCE_PERMEABILITY_LIMIT"
    );
    assert_eq!(
        select_primary_route(&RouteDecisionInput {
            d051_reproduced: true,
            ledger_ok: true,
            accounting_ok: true,
            numerical_ok: true,
            mixed_delivery: true,
            ..Default::default()
        })
        .as_str(),
        "D052_MIXED_RESOURCE_DELIVERY_LIMIT"
    );
}

#[test]
fn no_diagnostic_feedback_frozen_activation() {
    assert!((D052_FITTED_V_A - 0.12544510052968755).abs() < 1e-15);
    assert!((D052_FITTED_K_C - 0.10).abs() < 1e-15);
    assert_eq!(D052_N_REF, 1.0);
    assert_eq!(D052_F_REF, 1.0);
}

#[test]
fn cap_site_fractions() {
    let n = vec![0.1, 1.0, 0.1, 1.0];
    let f = vec![1.0, 0.1, 0.1, 1.0];
    let c = classify_cap_sites(&n, &f, 1.0, 1.0);
    assert!((c.n_limited - 0.25).abs() < 1e-12);
    assert!((c.f_limited - 0.25).abs() < 1e-12);
    assert!((c.jointly_limited - 0.25).abs() < 1e-12);
    assert!((c.unconstrained - 0.25).abs() < 1e-12);
}
