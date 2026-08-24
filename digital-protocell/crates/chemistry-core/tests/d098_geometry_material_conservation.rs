use chemistry_core::d096_allocation::{AllocationGenotype, AllocationState};
use chemistry_core::material_mesh::{
    conserve_interior_amount_across_area_change, LumpedChem, MaterialMesh, MeshContractVersion,
};
use chemistry_core::mesh_mechanics::{compute_forces, mechanics_step, remesh, MechParams};

fn fixture() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        8,
        2.0,
        0.0,
        0.0,
        1.0,
        0.4,
        LumpedChem {
            c: 0.8,
            a: 0.6,
            n: 0.5,
            f: 0.4,
            w: 0.3,
            tracer_c: 0.2,
            c_h: 0.35,
            c_b: 0.45,
            r: 0.25,
            u_h: 0.15,
            u_b: 0.16,
            k_h: 0.11,
            k_b: 0.12,
            q_k: 0.13,
            q_e: 0.14,
            k_a: 0.07,
            k_r: 0.08,
            k_node_b: 0.09,
        },
        LumpedChem {
            c: 0.2,
            a: 0.1,
            n: 1.0,
            f: 1.1,
            ..Default::default()
        },
        2.0,
    );
    mesh.finite_allocation = Some(AllocationState {
        genotype: AllocationGenotype::neutral(),
        catalysts: [0.31, 0.22, 0.13, 0.07],
    });
    mesh
}

fn concentration_amounts(mesh: &MaterialMesh) -> [f64; 18] {
    let area = mesh.area();
    let c = mesh.interior;
    [
        c.c, c.a, c.n, c.f, c.w, c.tracer_c, c.c_h, c.c_b, c.r, c.u_h, c.u_b, c.k_h, c.k_b, c.q_k,
        c.q_e, c.k_a, c.k_r, c.k_node_b,
    ]
    .map(|value| value * area)
}

fn assert_close(left: f64, right: f64) {
    assert!(
        (left - right).abs() <= 1e-10 * (1.0 + left.abs().max(right.abs())),
        "{left} != {right}"
    );
}

#[test]
fn geometry_contract_rescales_all_interior_concentration_amounts_only() {
    let mut mesh = fixture();
    mesh.stamp_geometry_conservative_schema();
    let before = concentration_amounts(&mesh);
    let exterior_before = mesh.exterior;
    let edges_before = mesh.edges.clone();
    let free_l_before = mesh.free_l;
    let allocation_before = mesh.finite_allocation;
    let area_before = mesh.area();
    let area_after = area_before * 1.7;
    let centroid = mesh.centroid();
    let scale = (area_after / area_before).sqrt();
    for vertex in &mut mesh.vertices {
        vertex[0] = centroid[0] + (vertex[0] - centroid[0]) * scale;
        vertex[1] = centroid[1] + (vertex[1] - centroid[1]) * scale;
    }
    assert_close(mesh.area(), area_after);

    assert!(conserve_interior_amount_across_area_change(
        &mut mesh,
        area_before,
        area_after
    ));

    // The helper is geometry-driven; the test supplies the accepted post-area
    // directly by verifying the same scale identity used by the integrator.
    for (value, expected) in concentration_amounts(&mesh)
        .into_iter()
        .zip(before.into_iter())
    {
        assert_close(value, expected);
    }
    assert_eq!(mesh.exterior.c, exterior_before.c);
    assert_eq!(mesh.exterior.n, exterior_before.n);
    assert_eq!(mesh.edges, edges_before);
    assert_eq!(mesh.free_l, free_l_before);
    assert_eq!(mesh.finite_allocation, allocation_before);
}

#[test]
fn mechanics_candidate_preserves_forces_geometry_and_non_interior_material() {
    let mut v2 = fixture();
    let mut v3 = fixture();
    v2.stamp_conservative_schema();
    v3.stamp_geometry_conservative_schema();
    let params = MechParams::default();
    let forces_v2 = compute_forces(&v2, &params);
    let forces_v3 = compute_forces(&v3, &params);
    assert_eq!(forces_v2, forces_v3);

    let edges_before = v3.edges.clone();
    let free_l_before = v3.free_l;
    let exterior_before = v3.exterior;
    let amount_before = concentration_amounts(&v3);
    assert!(mechanics_step(&mut v2, &params));
    assert!(mechanics_step(&mut v3, &params));

    assert_eq!(v2.vertices, v3.vertices);
    assert_eq!(v2.edges, v3.edges);
    assert_eq!(v2.free_l, v3.free_l);
    assert_eq!(v3.edges, edges_before);
    assert_eq!(v3.free_l, free_l_before);
    assert_eq!(v3.exterior, exterior_before);
    for (left, right) in concentration_amounts(&v3)
        .into_iter()
        .zip(amount_before.into_iter())
    {
        assert_close(left, right);
    }
}

#[test]
fn remesh_candidate_preserves_topology_decisions_and_interior_amounts() {
    let mut v2 = fixture();
    let mut v3 = fixture();
    v2.stamp_conservative_schema();
    v3.stamp_geometry_conservative_schema();
    v2.l_min = 1.6;
    v3.l_min = 1.6;
    let amount_before = concentration_amounts(&v3);
    let (v2_split, v2_merge) = remesh(&mut v2);
    let (v3_split, v3_merge) = remesh(&mut v3);

    assert_eq!((v2_split, v2_merge), (v3_split, v3_merge));
    assert_eq!(v2.vertices, v3.vertices);
    assert_eq!(v2.edges, v3.edges);
    assert_eq!(v2.area(), v3.area());
    for (left, right) in concentration_amounts(&v3)
        .into_iter()
        .zip(amount_before.into_iter())
    {
        assert_close(left, right);
    }
}

#[test]
fn no_geometry_change_is_inert_and_v2_remains_unchanged() {
    let mut v2 = fixture();
    let mut v3 = fixture();
    v2.stamp_conservative_schema();
    v3.stamp_geometry_conservative_schema();
    let v2_before = v2.interior;
    let v3_before = v3.interior;
    let area = v3.area();
    assert!(conserve_interior_amount_across_area_change(
        &mut v3, area, area
    ));
    assert_eq!(v2.interior, v2_before);
    assert_eq!(v3.interior, v3_before);
    assert!(!conserve_interior_amount_across_area_change(
        &mut v2,
        area,
        area * 1.2
    ));
    assert_eq!(v2.interior, v2_before);
    assert_eq!(v2.contract_version, MeshContractVersion::ConservativeV2);
}
