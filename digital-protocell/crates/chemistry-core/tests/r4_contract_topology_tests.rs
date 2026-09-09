use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_topology::{local_same_edge_rebond, tension_rupture_step, TopologyParams};

fn regular_mesh(contract: MeshContractVersion) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        16,
        5.0,
        0.0,
        0.0,
        1.0,
        0.5,
        LumpedChem {
            c: 2.0,
            a: 10.0,
            ..LumpedChem::default()
        },
        LumpedChem::default(),
        0.0,
    );
    mesh.contract_version = contract;
    mesh
}

#[test]
fn r4_contract_ownership_matrix_isolates_m_young_to_v4() {
    let contracts = [
        MeshContractVersion::HistoricalV1,
        MeshContractVersion::ConservativeV2,
        MeshContractVersion::GeometryConservativeV3,
        MeshContractVersion::MaturationCoupledV4,
    ];
    let states = [0.0, 0.4, 1.0, 1.2];

    for contract in contracts {
        for young in states {
            let mut mesh = regular_mesh(contract);
            for edge in &mut mesh.edges {
                edge.m = 1.0;
                edge.m_young = young;
            }
            let is_v4 = contract == MeshContractVersion::MaturationCoupledV4;
            let expected_valid = !is_v4 || young <= 1.0 + 1e-12;

            assert_eq!(mesh.physical_runtime_valid(), expected_valid);
            assert_eq!(mesh.can_advance_physics(), expected_valid);
            assert_eq!(mesh.lifecycle_invariants_hold(), expected_valid);
            if is_v4 {
                assert!((mesh.young_structural_mass(0) - young.min(1.0)).abs() < 1e-12);
                assert!((mesh.mature_structural_mass(0) - (1.0 - young.min(1.0))).abs() < 1e-12);
                assert!((mesh.rest_length(0) - (1.0 - young.min(1.0)).max(1e-15)).abs() < 1e-12);
            } else {
                assert_eq!(mesh.young_structural_mass(0), 0.0);
                assert_eq!(mesh.mature_structural_mass(0), 1.0);
                assert_eq!(mesh.rest_length(0), 1.0);
            }
        }
    }
}

#[test]
fn r4_non_v4_runtime_validation_does_not_mutate_inert_state() {
    for contract in [
        MeshContractVersion::HistoricalV1,
        MeshContractVersion::ConservativeV2,
        MeshContractVersion::GeometryConservativeV3,
    ] {
        let mut mesh = regular_mesh(contract);
        mesh.edges[0].m = 1.0;
        mesh.edges[0].m_young = 2.0;
        let before = serde_json::to_vec(&mesh).expect("serialize pre-validation state");
        assert!(mesh.physical_runtime_valid());
        assert!(mesh.can_advance_physics());
        let after = serde_json::to_vec(&mesh).expect("serialize post-validation state");
        assert_eq!(before, after);
    }
}

#[test]
fn r4_v4_tension_rupture_clears_removed_subpools_and_scales_membrane_tracer() {
    let mut mesh = regular_mesh(MeshContractVersion::MaturationCoupledV4);
    mesh.edges[0].m = 0.1;
    mesh.edges[0].m_young = 0.04;
    mesh.edges[0].tracer_m = 0.03;
    mesh.edges[0].b = 1.0;
    mesh.edges[0].tracer_b = 0.4;
    let w_before = mesh.interior.w * mesh.area();

    let ruptures = tension_rupture_step(&mut mesh, &TopologyParams::default());

    assert_eq!(ruptures, 1);
    assert_eq!(mesh.edges[0].m, 0.0);
    assert_eq!(mesh.edges[0].m_young, 0.0);
    assert_eq!(mesh.edges[0].tracer_m, 0.0);
    assert_eq!(mesh.edges[0].b, 0.5);
    assert_eq!(mesh.edges[0].tracer_b, 0.2);
    assert!(mesh.edges[0].ruptured);
    assert!((mesh.interior.w * mesh.area() - w_before - 0.1).abs() < 1e-12);
    assert!(mesh.lifecycle_invariants_hold());
    assert!(mesh.physical_runtime_valid());
}

#[test]
fn r4_v4_same_edge_rebond_creates_unlabelled_young_structure() {
    let mut mesh = MaterialMesh::seed_regular(
        16,
        0.5,
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
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh.edges[0].m = 0.0;
    mesh.edges[0].m_young = 0.0;
    mesh.edges[0].tracer_m = 0.25;
    mesh.edges[0].ruptured = true;

    let rebounds = local_same_edge_rebond(&mut mesh, &TopologyParams::default());

    assert_eq!(rebounds, 1);
    assert!(!mesh.edges[0].ruptured);
    assert!(mesh.edges[0].m > 0.0);
    assert_eq!(mesh.edges[0].m_young, mesh.edges[0].m);
    assert_eq!(mesh.edges[0].tracer_m, 0.0);
    assert!(mesh.lifecycle_invariants_hold());
    assert!(mesh.physical_runtime_valid());
}

#[test]
fn r4_non_v4_topology_preserves_inert_m_young_bytes() {
    let mut mesh = regular_mesh(MeshContractVersion::HistoricalV1);
    mesh.edges[0].m = 0.1;
    mesh.edges[0].m_young = 0.4;
    mesh.edges[0].tracer_m = 0.03;
    mesh.edges[0].b = 1.0;
    mesh.edges[0].tracer_b = 0.4;

    assert_eq!(
        tension_rupture_step(&mut mesh, &TopologyParams::default()),
        1
    );
    assert_eq!(mesh.edges[0].m_young, 0.4);
    assert_eq!(mesh.edges[0].tracer_m, 0.03);
    assert_eq!(mesh.edges[0].tracer_b, 0.4);
}
