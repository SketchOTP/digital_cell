//! D-033 Gate 1: activated intermediate conservation and causality.
use chemistry_core::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator, NINE_FIELD_COUNT};
use chemistry_core::d033_analysis::{
    activation_accounting_residual, charge_zero_without_p_or_a_or_q, frozen_exchange_kinetics_ok,
    identify_orthogonal_rates, insert_zero_without_x_or_capacity, intermediate_material_residual,
    relax_returns_x_to_p, v10_params, PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT, D033_ALPHA_FROZEN,
    D033_BETA_FROZEN,
};
use chemistry_core::fields::FIELD_NAMES_V10;
use chemistry_core::surface_density::{
    apply_activated_intermediate_bounded, apply_charge_bounded, apply_insert_bounded,
    apply_relax_bounded, SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use chemistry_core::FieldSchemaVersion;

fn v10_test_params(k_charge: f64, k_insert: f64, k_relax: f64) -> SimParams {
    let mut p = v10_params(k_charge, k_insert, k_relax);
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.k_exchange = 0.0;
    p.reactions_enabled = false;
    p
}

#[test]
fn v10_dispatch_and_schema() {
    let p = v10_params(1.0, 1.0, 0.1);
    assert_eq!(
        p.equation_version.as_str(),
        "membrane_metabolism_v10_activated_intermediate"
    );
    assert!(p.equation_version.is_activated_intermediate());
    assert!(!p.equation_version.is_activated_surface_assembly());
    assert!(p.equation_version.is_reversible_surface_exchange());
    assert!(p.equation_version.is_surface_density());
    assert!(p.equation_version.is_nine_field());
    assert!(!p.equation_version.is_eight_field());
    assert_eq!(p.equation_version.surface_exchange_schema_version(), 4);
    assert_eq!(p.equation_version.activated_intermediate_schema_version(), 1);
    assert_eq!(p.equation_version.active_assembly_schema_version(), 0);
    assert_eq!(NINE_FIELD_COUNT, 9);
    assert_eq!(FIELD_NAMES_V10.len(), 9);
    assert_eq!(
        SurfaceExchangeIntegrator::InvariantDomainV2.as_str(),
        SURFACE_EXCHANGE_INTEGRATOR_V2
    );
    assert!(frozen_exchange_kinetics_ok());
    assert!((D033_ALPHA_FROZEN - 0.167).abs() < 5e-3);
    assert!((D033_BETA_FROZEN - 0.00334).abs() < 5e-5);
    assert_eq!(
        PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
        "PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT_FOR_MEMBRANE_MAINTENANCE"
    );
    let id_a = chemistry_core::canonical_params_bytes(&p);
    let mut p2 = p.clone();
    p2.k_charge *= 1.01;
    let id_b = chemistry_core::canonical_params_bytes(&p2);
    assert_ne!(id_a, id_b, "X rate params must enter candidate hash");
}

#[test]
fn charge_causality_and_no_direct_a_to_s() {
    let params = v10_test_params(2.0, 0.0, 0.0);
    assert!(charge_zero_without_p_or_a_or_q(&params));
    // No insertion ⇒ S unchanged while charging.
    let (p1, a1, x1, s1, dw, r_c, r_i, r_r) = apply_activated_intermediate_bounded(
        1.0, 0.4, 0.5, 0.4, 0.0, 0.1, 0.5, 0.01, &params,
    );
    assert!(r_c > 0.0);
    assert_eq!(r_i, 0.0);
    assert_eq!(r_r, 0.0);
    assert!((s1 - 0.1).abs() < 1e-15, "no direct A→S");
    assert!((x1 - r_c).abs() < 1e-12);
    assert!((0.5 - p1 - r_c).abs() < 1e-12);
    assert!((0.4 - a1 - r_c).abs() < 1e-12);
    assert!((dw - r_c).abs() < 1e-12);
}

#[test]
fn insert_continues_without_a_when_x_remains() {
    let params = v10_test_params(0.0, 2.0, 0.0);
    assert!(insert_zero_without_x_or_capacity(&params));
    let (x1, s1, r) = apply_insert_bounded(0.3, 0.1, 0.5, 0.01, &params);
    assert!(r > 0.0);
    assert!((0.3 - x1 - r).abs() < 1e-12);
    assert!((s1 - 0.1 - r).abs() < 1e-12);
    // A is unused.
    let (_, a1, x2, s2, _, _, r_i, _) = apply_activated_intermediate_bounded(
        1.0, 0.4, 0.0, 0.0, 0.3, 0.1, 0.5, 0.01, &params,
    );
    assert_eq!(a1, 0.0);
    assert!(r_i > 0.0);
    assert!(s2 > 0.1);
    assert!(x2 < 0.3);
}

#[test]
fn relaxation_returns_x_to_p_no_waste() {
    let params = v10_test_params(0.0, 0.0, 1.0);
    assert!(relax_returns_x_to_p(&params));
    let (x1, p1, r) = apply_relax_bounded(0.4, 0.1, 0.01, &params);
    assert!(r > 0.0);
    assert!((x1 + p1 - 0.5).abs() < 1e-12);
}

#[test]
fn material_and_activation_accounting_close() {
    let params = v10_test_params(1.5, 1.0, 0.2);
    let (residual, r_c, r_i, r_r) = intermediate_material_residual(
        1.0, 0.4, 0.5, 0.4, 0.1, 0.05, 0.0, 0.5, 0.01, &params,
    );
    assert!(residual.abs() < 1e-12, "material residual={residual}");
    let (p1, a1, x1, s1, dw, _, _, _) = apply_activated_intermediate_bounded(
        1.0, 0.4, 0.5, 0.4, 0.1, 0.05, 0.5, 0.01, &params,
    );
    let act = activation_accounting_residual(r_c, r_i, r_r, 0.1, x1);
    assert!(act.abs() < 1e-12, "activation residual={act}");
    assert!(p1 >= -1e-15 && a1 >= -1e-15 && x1 >= -1e-15 && s1 >= -1e-15 && dw >= -1e-15);
}

#[test]
fn bounded_transfers_respect_capacity_and_inventory() {
    let params = v10_test_params(1000.0, 1000.0, 1000.0);
    let d = 0.5;
    let s0 = d * params.gamma_max - 1e-4;
    let (p1, a1, x1, s1, _, r_c, r_i, r_r) = apply_activated_intermediate_bounded(
        1.0, 0.4, 1e-5, 1e-5, 1e-5, s0, d, 1.0, &params,
    );
    assert!(p1 >= -1e-15 && a1 >= -1e-15 && x1 >= -1e-15);
    assert!(s1 <= d * params.gamma_max + 1e-12);
    assert!(r_c <= 1e-5 + 1e-18);
    assert!(r_i + r_r <= 1e-5 + r_c + 1e-18);
}

#[test]
fn no_charging_when_q_c_zero() {
    let params = v10_test_params(2.0, 0.0, 0.0);
    let (_, _, _, _, r) = apply_charge_bounded(1.0, 0.0, 1.0, 1.0, 0.0, 0.01, &params);
    assert_eq!(r, 0.0);
}

#[test]
fn snapshot_schema_is_nine_field() {
    assert_eq!(
        format!("{:?}", FieldSchemaVersion::NineFieldSurfaceDensityV1)
            .contains("NineField"),
        true
    );
    let _ = EquationVersion::MembraneMetabolismV10ActivatedIntermediate;
}

#[test]
fn orthogonal_rate_identification_recovers_truth() {
    let id = identify_orthogonal_rates(0.8, 1.2, 0.25);
    assert!(id.charge_ok, "charge {:?}", id);
    assert!(id.insert_ok, "insert {:?}", id);
    assert!(id.relax_ok, "relax {:?}", id);
    assert!(id.identifiable, "conclusion={}", id.conclusion);
}
