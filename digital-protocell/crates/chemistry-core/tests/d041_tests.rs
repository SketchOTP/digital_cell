//! D-041 focused tests: structural A retention transport and selection rules.

use chemistry_core::candidate_identity::{build_candidate_identity, canonical_params_bytes};
use chemistry_core::config::{
    EquationVersion, SimParams, MEMBRANE_TRANSPORT_SCHEMA_3_STRUCTURAL_A_RETENTION,
    TRANSPORT_SCHEMA_VERSION_V1, TRANSPORT_SCHEMA_VERSION_V3,
};
use chemistry_core::d041_analysis::{
    apply_structural_a_retention, bracket_intermediate, build_rho_candidates,
    mature_membrane_nonredundant, pi_a_healthy, retention_candidate_passes, select_weakest_passing_rho,
    transport_schema_name, RetentionCandidateMetrics, D041_MAX_RHO_CANDIDATES, D041_RHO_SCREEN,
};
use chemistry_core::grid::Grid;
use chemistry_core::membrane_transport::{
    face_flux, permeability_surface_occupancy, transport_field, TransportSpecies,
};
use chemistry_core::surface_density::reconstruct_gamma;

fn base_surface_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    p.beta_a = 4.6;
    p.beta_c = 4.6;
    p.beta_n = 1.2;
    p.beta_f = 1.2;
    p.beta_w = 0.2;
    p.gamma_reference = 1.0;
    p.delta_floor = 1e-6;
    p.d_a = 0.10;
    p.transport_schema_version = TRANSPORT_SCHEMA_VERSION_V1;
    p.rho_a = 1.0;
    p
}

#[test]
fn transport_schema_name_and_apply() {
    let mut p = base_surface_params();
    assert_eq!(transport_schema_name(&p), "historical");
    apply_structural_a_retention(&mut p, 0.4);
    assert_eq!(p.transport_schema_version, TRANSPORT_SCHEMA_VERSION_V3);
    assert!((p.rho_a - 0.4).abs() < 1e-15);
    assert_eq!(
        transport_schema_name(&p),
        MEMBRANE_TRANSPORT_SCHEMA_3_STRUCTURAL_A_RETENTION
    );
}

#[test]
fn historical_equivalence_at_rho_one() {
    let mut hist = base_surface_params();
    let mut schema = base_surface_params();
    apply_structural_a_retention(&mut schema, 1.0);
    let phi_in = 0.75;
    let phi_out = 0.25;
    let s = 0.5;
    for sp in [
        TransportSpecies::Activated,
        TransportSpecies::Catalyst,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ] {
        let a = permeability_surface_occupancy(sp, phi_in, phi_out, s, s, &hist);
        let b = permeability_surface_occupancy(sp, phi_in, phi_out, s, s, &schema);
        assert!(
            (a - b).abs() < 1e-15,
            "{sp:?}: hist={a} schema_rho1={b}"
        );
    }
}

#[test]
fn a_only_structural_attenuation() {
    let mut p = base_surface_params();
    apply_structural_a_retention(&mut p, 0.5);
    let phi_in = 0.8;
    let phi_out = 0.2;
    let s = 0.0; // θ≈0 → mature factor 1 → Π_A = ρ_A
    let a = permeability_surface_occupancy(
        TransportSpecies::Activated,
        phi_in,
        phi_out,
        s,
        s,
        &p,
    );
    assert!((a - 0.5).abs() < 1e-12, "A perm={a}");
    for sp in [
        TransportSpecies::Catalyst,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ] {
        let perm = permeability_surface_occupancy(sp, phi_in, phi_out, s, s, &p);
        assert!((perm - 1.0).abs() < 1e-15, "{sp:?}={perm}");
    }
}

#[test]
fn non_interface_faces_unaffected() {
    let mut p = base_surface_params();
    apply_structural_a_retention(&mut p, 0.2);
    let perm = permeability_surface_occupancy(
        TransportSpecies::Activated,
        0.8,
        0.7, // both inside
        1.0,
        1.0,
        &p,
    );
    assert!((perm - 1.0).abs() < 1e-15);
}

#[test]
fn antisymmetric_a_face_flux() {
    let mut p = base_surface_params();
    apply_structural_a_retention(&mut p, 0.4);
    let f_ij = face_flux(
        TransportSpecies::Activated,
        1.0,
        0.0,
        0.8,
        0.2,
        0.0,
        0.0,
        &p,
    );
    let f_ji = face_flux(
        TransportSpecies::Activated,
        0.0,
        1.0,
        0.2,
        0.8,
        0.0,
        0.0,
        &p,
    );
    assert!((f_ij + f_ji).abs() < 1e-15, "f_ij={f_ij} f_ji={f_ji}");
}

#[test]
fn a_mass_conserved_without_reactions() {
    let mut p = base_surface_params();
    apply_structural_a_retention(&mut p, 0.35);
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut field = vec![0.0; n];
    let mut phi = vec![0.0; n];
    let membrane = vec![0.0; n];
    let mut rate = vec![0.0; n];
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let cx = i as f64 - grid.cx;
            let cy = j as f64 - grid.cy;
            let r = (cx * cx + cy * cy).sqrt();
            phi[idx] = if r < 4.0 { 1.0 } else { 0.0 };
            field[idx] = if r < 4.0 { 1.0 } else { 0.1 };
        }
    }
    let before: f64 = grid
        .dish_mask
        .iter()
        .zip(field.iter())
        .filter(|(m, _)| **m)
        .map(|(_, v)| *v)
        .sum();
    let acct = transport_field(
        &grid,
        TransportSpecies::Activated,
        &field,
        &phi,
        &membrane,
        &p,
        &mut rate,
    );
    assert!(acct.net_change_rate.abs() < 1e-12, "net={}", acct.net_change_rate);
    let after_rate: f64 = grid
        .dish_mask
        .iter()
        .zip(rate.iter())
        .filter(|(m, _)| **m)
        .map(|(_, v)| *v)
        .sum();
    assert!(after_rate.abs() < 1e-12, "sum_rate={after_rate}");
    let _ = before;
    let _ = reconstruct_gamma;
}

#[test]
fn candidate_hash_includes_schema_and_rho() {
    let mut p = base_surface_params();
    let id0 = build_candidate_identity(p.clone(), "probe", None, None, "probe", None, None);
    apply_structural_a_retention(&mut p, 0.55);
    let id1 = build_candidate_identity(p.clone(), "probe", None, None, "probe", None, None);
    assert_ne!(id0.candidate_hash, id1.candidate_hash);
    let canonical = String::from_utf8(canonical_params_bytes(&p)).expect("utf8");
    assert!(canonical.contains("rho_a=0.55"));
    assert!(canonical.contains(MEMBRANE_TRANSPORT_SCHEMA_3_STRUCTURAL_A_RETENTION));
}

#[test]
fn candidate_limits_and_weakest_selection() {
    let c = build_rho_candidates(&D041_RHO_SCREEN, Some(0.5));
    assert!(c.len() <= D041_MAX_RHO_CANDIDATES);
    assert!(c.contains(&0.5));
    let mid = bracket_intermediate(0.8, 0.6);
    assert!((mid - 0.7).abs() < 1e-15);
    let pass = retention_candidate_passes(RetentionCandidateMetrics {
        a_decline_precedes_collapse: false,
        endogenous_p: 0.03,
        s_toward_healthy: true,
        accounting_ok: true,
        numerical_ok: true,
    });
    assert!(pass);
    let sel = select_weakest_passing_rho(&[(1.0, false), (0.4, true), (0.2, true)]);
    assert!((sel.unwrap() - 0.4).abs() < 1e-15);
}

#[test]
fn mature_membrane_nonredundancy() {
    let beta = 4.6;
    let pi = pi_a_healthy(beta, 1.0);
    assert!(mature_membrane_nonredundant(0.2, beta, 1.0));
    assert!(!mature_membrane_nonredundant(pi, beta, 1.0));
}
