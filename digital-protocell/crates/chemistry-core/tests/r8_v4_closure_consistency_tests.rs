use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_fission::{try_local_segment_fission, FissionParams};
use chemistry_core::mesh_growth::Y_G_CANDIDATES;
use chemistry_core::mesh_mechanics::{compute_forces, MechParams};
use chemistry_core::mesh_topology::{local_same_edge_rebond, tension_rupture_step, TopologyParams};

fn regular(contract: MeshContractVersion, young_fraction: f64) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        12,
        4.0,
        0.0,
        0.0,
        1.0,
        0.5,
        LumpedChem {
            c: 2.0,
            a: 100.0,
            ..LumpedChem::default()
        },
        LumpedChem::default(),
        0.0,
    );
    mesh.contract_version = contract;
    for i in 0..mesh.n() {
        mesh.edges[i].m = 0.5 * mesh.rho_s * mesh.edge_length(i);
        mesh.edges[i].m_young = young_fraction * mesh.edges[i].m;
    }
    mesh
}

fn stretch_only() -> MechParams {
    MechParams {
        gamma: 1.0,
        k_s: 14.0,
        kappa_b: 0.0,
        k_pi: 0.0,
        dt: 0.02,
    }
}

fn force_norm(forces: &[[f64; 2]]) -> f64 {
    forces
        .iter()
        .map(|force| force[0].hypot(force[1]))
        .sum::<f64>()
}

#[test]
fn fully_young_v4_has_zero_stretch_load_and_does_not_rupture_from_raw_strain() {
    let mut mesh = regular(MeshContractVersion::MaturationCoupledV4, 1.0);
    assert!(mesh.strain(0) > TopologyParams::default().strain_rupture);
    assert_eq!(mesh.mature_structural_fraction(0), 0.0);
    assert_eq!(mesh.load_bearing_strain(0), 0.0);
    assert_eq!(force_norm(&compute_forces(&mesh, &stretch_only())), 0.0);
    assert_eq!(
        tension_rupture_step(&mut mesh, &TopologyParams::default()),
        0
    );
}

#[test]
fn mixed_v4_load_changes_continuously_with_mature_fraction() {
    let young = regular(MeshContractVersion::MaturationCoupledV4, 1.0);
    let mixed = regular(MeshContractVersion::MaturationCoupledV4, 0.5);
    let nearby = regular(MeshContractVersion::MaturationCoupledV4, 0.500_001);
    let mature = regular(MeshContractVersion::MaturationCoupledV4, 0.0);
    let f0 = force_norm(&compute_forces(&young, &stretch_only()));
    let f1 = force_norm(&compute_forces(&mixed, &stretch_only()));
    let f_nearby = force_norm(&compute_forces(&nearby, &stretch_only()));
    let f2 = force_norm(&compute_forces(&mature, &stretch_only()));
    assert_eq!(f0, 0.0);
    assert!(f1 > f0);
    assert!(f2 > f0);
    assert!((f1 - f_nearby).abs() < 1e-3 * f1.max(1.0));
    assert_eq!(mixed.mature_structural_fraction(0), 0.5);
}

fn compressed_v4(young_mass_per_edge: f64) -> MaterialMesh {
    let mut mesh = regular(MeshContractVersion::MaturationCoupledV4, 0.0);
    for i in 0..mesh.n() {
        let mature = 2.0 * mesh.rho_s * mesh.edge_length(i);
        mesh.edges[i].m = mature + young_mass_per_edge;
        mesh.edges[i].m_young = young_mass_per_edge;
        assert!(mesh.strain(i) < 0.0);
    }
    mesh
}

#[test]
fn compressed_v4_scaffold_retains_raw_spring_force() {
    let v4 = compressed_v4(0.5);
    let mut v3 = v4.clone();
    v3.contract_version = MeshContractVersion::GeometryConservativeV3;
    for edge in &mut v3.edges {
        edge.m -= edge.m_young;
        edge.m_young = 0.0;
    }
    assert_eq!(
        compute_forces(&v4, &stretch_only()),
        compute_forces(&v3, &stretch_only())
    );
}

#[test]
fn adding_young_mass_does_not_weaken_existing_compressed_scaffold() {
    let base = compressed_v4(0.0);
    let added = compressed_v4(3.0);
    assert!(added.mature_structural_fraction(0) < 1.0);
    assert_eq!(base.rest_length(0), added.rest_length(0));
    assert_eq!(base.strain(0), added.strain(0));
    assert_eq!(
        compute_forces(&base, &stretch_only()),
        compute_forces(&added, &stretch_only())
    );
}

#[test]
fn fully_mature_v4_matches_prior_non_v4_stretch_and_rupture_semantics() {
    let mut v4 = regular(MeshContractVersion::MaturationCoupledV4, 0.0);
    let mut v3 = v4.clone();
    v3.contract_version = MeshContractVersion::GeometryConservativeV3;
    assert_eq!(
        compute_forces(&v4, &stretch_only()),
        compute_forces(&v3, &stretch_only())
    );
    assert_eq!(v4.load_bearing_strain(0), v4.strain(0));
    assert_eq!(
        tension_rupture_step(&mut v4, &TopologyParams::default()),
        tension_rupture_step(&mut v3, &TopologyParams::default())
    );
    assert_eq!(
        v4.edges
            .iter()
            .map(|edge| edge.ruptured)
            .collect::<Vec<_>>(),
        v3.edges
            .iter()
            .map(|edge| edge.ruptured)
            .collect::<Vec<_>>()
    );
}

#[test]
fn v4_same_edge_rebond_is_young_and_does_not_immediately_rerupt() {
    let mut mesh = regular(MeshContractVersion::MaturationCoupledV4, 0.0);
    mesh.edges[0].m = 0.0;
    mesh.edges[0].m_young = 0.0;
    mesh.edges[0].ruptured = true;
    assert_eq!(
        local_same_edge_rebond(&mut mesh, &TopologyParams::default()),
        1
    );
    assert_eq!(mesh.edges[0].m_young, mesh.edges[0].m);
    assert_eq!(mesh.mature_structural_fraction(0), 0.0);
    assert_eq!(
        tension_rupture_step(&mut mesh, &TopologyParams::default()),
        0
    );
    assert!(!mesh.edges[0].ruptured);
}

fn dumbbell(contract: MeshContractVersion) -> MaterialMesh {
    let points = vec![
        [-3.0, -2.0],
        [-1.0, -2.0],
        [-0.5, -0.2],
        [0.5, -0.2],
        [1.0, -2.0],
        [3.0, -2.0],
        [3.0, 2.0],
        [1.0, 2.0],
        [0.5, 0.2],
        [-0.5, 0.2],
        [-1.0, 2.0],
        [-3.0, 2.0],
    ];
    let mut mesh = MaterialMesh::seed_regular(
        points.len(),
        3.0,
        0.0,
        0.0,
        1.0,
        0.5,
        LumpedChem {
            a: 100.0,
            c: 1.0,
            ..Default::default()
        },
        LumpedChem::default(),
        0.0,
    );
    mesh.vertices = points;
    mesh.contract_version = contract;
    for i in 0..mesh.n() {
        mesh.edges[i].m = mesh.rho_s * mesh.edge_length(i);
        mesh.edges[i].m_young = 0.0;
    }
    mesh
}

#[test]
fn v4_fission_creates_two_full_density_young_edges_with_yield_closed_cost() {
    let parent = dumbbell(MeshContractVersion::MaturationCoupledV4);
    let pre_m = parent.total_structural_mass();
    let pre_a = parent.interior.a * parent.area();
    let pre_w = parent.interior.w * parent.area();
    let (a, b, event) = try_local_segment_fission(&parent, &FissionParams::default())
        .expect("lawful V4 segment fission");
    assert!(event.partition.ok);
    for daughter in [&a, &b] {
        let index = daughter.n() - 1;
        let edge = daughter.edges[index];
        let required = daughter.rho_s * daughter.edge_length(index);
        assert!((edge.m - required).abs() < 1e-12 * (1.0 + required));
        assert_eq!(edge.m_young, edge.m);
        assert_eq!(daughter.mature_structural_fraction(index), 0.0);
    }
    let post_m = a.total_structural_mass() + b.total_structural_mass();
    let post_a = a.interior.a * a.area() + b.interior.a * b.area();
    let post_w = a.interior.w * a.area() + b.interior.w * b.area();
    let made = post_m - pre_m;
    let spent = pre_a - post_a;
    assert!((spent - made / Y_G_CANDIDATES[0]).abs() < 1e-9 * (1.0 + spent));
    assert!((post_w - pre_w - (spent - made)).abs() < 1e-9 * (1.0 + spent));
    assert_eq!(event.leakage_w, 0.0);
}

#[test]
fn non_v4_fission_preserves_shared_half_density_closure_contract() {
    let parent = dumbbell(MeshContractVersion::GeometryConservativeV3);
    let pre_m = parent.total_structural_mass();
    let pre_a = parent.interior.a * parent.area();
    let (a, b, event) = try_local_segment_fission(&parent, &FissionParams::default())
        .expect("lawful V3 segment fission");
    let post_m = a.total_structural_mass() + b.total_structural_mass();
    let post_a = a.interior.a * a.area() + b.interior.a * b.area();
    let need = parent.rho_s * a.edge_length(a.n() - 1);
    assert!((post_m - pre_m - need).abs() < 1e-12 * (1.0 + need));
    assert!((pre_a - post_a - need).abs() < 1e-9 * (1.0 + need));
    assert!((a.edges[a.n() - 1].m - 0.5 * need).abs() < 1e-12);
    assert!((b.edges[b.n() - 1].m - 0.5 * need).abs() < 1e-12);
    assert!((event.leakage_w - need).abs() < 1e-12);
}
