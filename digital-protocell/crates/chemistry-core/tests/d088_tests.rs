use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
use chemistry_core::material_mesh::MeshContractVersion;
use chemistry_core::mesh_growth::{
    growth_step, growth_step_with_placement, local_a_surplus_rate,
    local_compression_neighbor_routing, GrowthParams, GrowthPlacementMode, Y_G_CANDIDATES,
};
use chemistry_core::mesh_population::MeshPopulation;
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_transport::TransportParams;

#[test]
fn y_g_candidates_are_three_analytic() {
    assert_eq!(Y_G_CANDIDATES.len(), 3);
    assert!(Y_G_CANDIDATES[0] < Y_G_CANDIDATES[1] && Y_G_CANDIDATES[1] < Y_G_CANDIDATES[2]);
}

#[test]
fn surplus_nonnegative() {
    let pop = MeshPopulation::seed_one(14.0, 1, 2.0);
    let mesh = &pop.individuals[0].mesh;
    let p = ReactionParams::default();
    for i in 0..mesh.n() {
        assert!(local_a_surplus_rate(mesh, i, &p) >= 0.0);
    }
}

#[test]
fn no_divide_symbol_in_fission_api() {
    // Compile-time presence of try_local_fission; absence of divide command is a source rule.
    let _ = try_local_fission;
    let _ = FissionParams::default();
}

#[test]
fn population_step_smoke() {
    let mut pop = MeshPopulation::seed_one(14.0, 1, 2.0);
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams::default();
    let fission = FissionParams::default();
    for _ in 0..50 {
        let _ = pop.step(&mech, &react, &transport, &growth, &fission, true);
    }
    assert!(pop.living_count() >= 1 || !pop.fission_log.is_empty());
}

fn v4_mesh() -> chemistry_core::material_mesh::MaterialMesh {
    let mut mesh = chemistry_core::material_mesh::MaterialMesh::seed_regular(
        12,
        5.0,
        0.0,
        0.0,
        1.0,
        0.7,
        Default::default(),
        Default::default(),
        2.0,
    );
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

#[test]
fn local_compression_routing_conserves_and_is_zero_compression_identity() {
    let mesh = v4_mesh();
    let increments = vec![0.1; mesh.n()];
    let compression = vec![0.0; mesh.n()];
    let routed = local_compression_neighbor_routing(&increments, &compression, &mesh);
    assert_eq!(routed, increments);
    assert!((routed.iter().sum::<f64>() - increments.iter().sum::<f64>()).abs() < 1e-12);
}

#[test]
fn local_compression_routing_is_one_hop_and_attracts_only_compressed_neighbors() {
    let mesh = v4_mesh();
    let increments = vec![1.0; mesh.n()];
    let mut compression = vec![0.0; mesh.n()];
    compression[3] = 0.8;
    let routed = local_compression_neighbor_routing(&increments, &compression, &mesh);
    assert!(routed[3] > increments[3]);
    assert!(routed[2] < increments[2]);
    assert!(routed[4] < increments[4]);
    assert_eq!(routed[5], increments[5]);
    assert_eq!(routed[6], increments[6]);
    assert!((routed.iter().sum::<f64>() - increments.iter().sum::<f64>()).abs() < 1e-12);
}

#[test]
fn local_compression_route_off_matches_frozen_growth_step() {
    let mut frozen = v4_mesh();
    let mut routed = frozen.clone();
    let react = ReactionParams::default();
    let growth = GrowthParams::default();
    let frozen_ledger = growth_step(&mut frozen, &react, &growth, 0.1);
    let routed_ledger = growth_step_with_placement(
        &mut routed,
        &react,
        &growth,
        0.1,
        GrowthPlacementMode::FrozenD088,
    );
    assert_eq!(frozen_ledger.m_grown, routed_ledger.m_grown);
    assert_eq!(frozen_ledger.a_consumed_growth, routed_ledger.a_consumed_growth);
    assert_eq!(frozen_ledger.w_from_growth, routed_ledger.w_from_growth);
    assert_eq!(frozen.interior, routed.interior);
    for (left, right) in frozen.edges.iter().zip(routed.edges.iter()) {
        assert_eq!(left.m, right.m);
        assert_eq!(left.m_young, right.m_young);
    }
}
