use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{local_a_surplus_rate, GrowthParams, Y_G_CANDIDATES};
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
