//! Focused D-086 material-mesh unit tests.

use chemistry_core::d086_analysis::*;
use chemistry_core::material_mesh::*;
use chemistry_core::mesh_mechanics::*;
use chemistry_core::mesh_reactions::*;
use chemistry_core::mesh_transport::*;

#[test]
fn schema_identity_isolated() {
    assert_eq!(EQUATION_VERSION_MATERIAL_MESH, "autopoietic_material_mesh_v1");
    assert_eq!(FIELD_SCHEMA_MATERIAL_MESH, "mesh_vertices_edges_v1");
    assert_eq!(MATERIAL_MESH_SCHEMA_VERSION, 1);
}

#[test]
fn stretch_bend_pressure_local_only() {
    let mesh = seed_organism(14.0, 1);
    let p = MechParams::default();
    let f = compute_forces(&mesh, &p);
    assert_eq!(f.len(), mesh.n());
    assert!(f.iter().all(|v| v[0].is_finite() && v[1].is_finite()));
    // Pressure uses lumped local chem, not area target.
    let pi = local_pressure(&mesh, 0);
    assert!(pi.is_finite());
}

#[test]
fn mechanics_conserves_material() {
    let mut mesh = seed_organism(14.0, 1);
    let m0 = mesh.total_structural_mass();
    let b0 = mesh.total_bound_membrane();
    let p = MechParams::default();
    for _ in 0..50 {
        assert!(mechanics_step(&mut mesh, &p));
    }
    assert!((mesh.total_structural_mass() - m0).abs() < 1e-9);
    assert!((mesh.total_bound_membrane() - b0).abs() < 1e-9);
}

#[test]
fn split_conserves_mass() {
    let mut mesh = seed_organism(14.0, 1);
    mesh.l_max = 1.5;
    let m0 = mesh.total_structural_mass();
    let b0 = mesh.total_bound_membrane();
    let n0 = mesh.n();
    let splits = remesh_split(&mut mesh);
    assert!(splits > 0 || mesh.edge_length(0) <= mesh.l_max);
    assert!(mesh.n() >= n0);
    assert!((mesh.total_structural_mass() - m0).abs() < 1e-9);
    assert!((mesh.total_bound_membrane() - b0).abs() < 1e-9);
}

#[test]
fn strain_build_increases_with_tension() {
    let mesh = seed_organism(14.0, 1);
    let p = ReactionParams::default();
    let g0 = g_strain(0.0, p.g0, p.k_eps);
    let g1 = g_strain(1.0, p.g0, p.k_eps);
    assert!(g1 > g0);
    let j = structural_build_flux(&mesh, 0, &p);
    assert!(j >= 0.0);
}

#[test]
fn permeability_targets_at_high_occupancy() {
    assert!(permeability_in_targets(0.9));
    let pc = permeability(0.9, "C");
    let pw = permeability(0.9, "W");
    assert!(pc <= 0.05 + 1e-12);
    assert!(pw >= 0.70);
}

#[test]
fn rupture_and_no_invisible_topology() {
    let mut mesh = seed_organism(14.0, 1);
    apply_local_rupture(&mut mesh, 0);
    assert!(mesh.edges[0].ruptured);
    assert!(!mesh.closed_intact());
}

#[test]
fn irreversible_death_no_respawn() {
    let mut mesh = seed_organism(14.0, 1);
    mesh.interior.c = 0.0;
    mesh.interior.a = 0.0;
    for e in &mut mesh.edges {
        e.m = 0.0;
        e.ruptured = true;
    }
    evaluate_death(&mut mesh);
    assert!(!mesh.alive);
    mesh.exterior.n = 1.0;
    mesh.exterior.f = 1.0;
    evaluate_death(&mut mesh);
    assert!(!mesh.alive);
}

#[test]
fn gate1_mechanics_passes() {
    let g = gate1_mechanics();
    assert!(g.pass, "{}", g.detail);
}
