//! D-090 unit checks: spatial dish conservation, pairwise interference, reserve audit helpers.

use chemistry_core::catalyst_composition::SIGMA_TRADEOFF;
use chemistry_core::founder_preconditioning::{
    audit_founder, catalyst_turnover_horizon, measure_reserve_funded_growth,
};
use chemistry_core::mesh_growth::GrowthParams;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::MeshIndividual;
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::shared_dish_audit::audit_shared_dish_harness;
use chemistry_core::spatial_shared_dish::SpatialDish;

#[test]
fn spatial_dish_mass_conserved_under_diffusion() {
    let mut dish = SpatialDish::new(16, 16, 2.0, [0.0, 0.0], 100.0, 80.0, 0.0, 0.0, 5.0);
    // Perturb one cell then diffuse.
    dish.n[0] += 10.0;
    dish.n[1] -= 10.0;
    let n0 = dish.total_n();
    let f0 = dish.total_f();
    for _ in 0..50 {
        dish.diffuse(0.02);
    }
    assert!((dish.total_n() - n0).abs() < 1e-9);
    assert!((dish.total_f() - f0).abs() < 1e-9);
}

#[test]
fn harness_audit_runs() {
    let mech = MechParams::default();
    let mut react = ReactionParams::default();
    react.composition.enable = true;
    react.composition.sigma = SIGMA_TRADEOFF;
    react.composition.mu = 0.0;
    let transport = TransportParams::default();
    let audit = audit_shared_dish_harness(&react, &transport, &mech);
    assert!(audit.shared_fields);
    assert!(audit.spatial_diffusion);
    assert!(audit.no_population_cap);
    assert!(audit.ledger_closes, "ledger detail={:?}", audit.detail);
    assert!(audit.pairwise_interference, "interference detail={:?}", audit.detail);
    assert!(audit.distance_modulates_competition, "distance detail={:?}", audit.detail);
    assert!(audit.pass, "harness failed detail={:?}", audit.detail);
}

#[test]
fn turnover_horizon_positive() {
    let r = ReactionParams::default();
    assert!(catalyst_turnover_horizon(&r) > 1.0);
}

#[test]
fn reserve_growth_measure_bounded() {
    let mut pop = chemistry_core::mesh_population::MeshPopulation::seed_one(8.0, 1, 1.0);
    let ind = &pop.individuals[0];
    let react = ReactionParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let mech = MechParams::default();
    let g = measure_reserve_funded_growth(&ind.mesh, &react, &growth, &mech, 200);
    assert!(g >= 0.0);
    let audit = audit_founder(
        &MeshIndividual {
            mesh: ind.mesh.clone(),
            lineage_id: 1,
            generation: 0,
            birth_mass: ind.birth_mass,
            clade: 1,
        },
        1.0,
        0.9,
    );
    assert!(audit.structural_mass > 0.0);
}
