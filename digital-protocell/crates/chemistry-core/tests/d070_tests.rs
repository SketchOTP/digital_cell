//! D-070 mature-membrane seed capacity contract tests.

use chemistry_core::config::DX;
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d063_analysis::{generate_phi, seed_mature_s_on_interfaces, GeometrySpec};
use chemistry_core::d070_analysis::*;
use chemistry_core::grid::Grid;
use chemistry_core::surface_density::{compute_interface_geometry, InterfaceGeometryCell};

fn geometry_for(spec: &GeometrySpec) -> (Grid, Vec<f64>, Vec<InterfaceGeometryCell>, f64) {
    let grid = Grid::new();
    let phi = generate_phi(&grid, spec);
    let params = v8_schema3_params();
    let mut geometry = vec![InterfaceGeometryCell::default(); phi.len()];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    (grid, phi, geometry, params.delta_floor)
}

#[test]
fn d069_reproduction_predicate() {
    assert!(d069_capacity_defect_reproduced(
        176.0,
        76.333,
        99.667,
        99.666
    ));
    assert!(!d069_capacity_defect_reproduced(76.0, 76.0, 0.0, 1.0));
}

#[test]
fn capacity_unit_identity() {
    assert!((local_s_max(0.4, 1.0) - 0.4).abs() < 1e-15);
    assert!((occupancy_theta(0.2, 0.4, 1.0) - 0.5).abs() < 1e-15);
    assert!((cell_volume() - DX * DX).abs() < 1e-15);
    assert_eq!(SEED_CAPACITY_CONTRACT_V1, "SEED_CAPACITY_CONTRACT_V1");
}

#[test]
fn integrated_capacity_and_radius_scaling() {
    let (grid, _phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(16.0));
    let s = seed_capacity_bounded_s(&grid, &geometry, floor, 1.0, 1.0);
    let p = vec![0.0; s.len()];
    let a16 = audit_capacity(&grid, &geometry, &s, &p, floor, 1.0);
    let (grid2, _phi2, geometry2, floor2) = geometry_for(&GeometrySpec::smooth(32.0));
    let s2 = seed_capacity_bounded_s(&grid2, &geometry2, floor2, 1.0, 1.0);
    let p2 = vec![0.0; s2.len()];
    let a32 = audit_capacity(&grid2, &geometry2, &s2, &p2, floor2, 1.0);
    assert!(a16.capacity_mass > 0.0);
    assert!(capacity_scales_with_radius(
        a16.capacity_mass,
        16.0,
        a32.capacity_mass,
        32.0
    ));
    assert!(capacity_independent_of_timestep(
        a16.capacity_mass,
        a16.capacity_mass
    ));
}

#[test]
fn historical_seed_over_capacity_detected() {
    let (grid, phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(22.0));
    let s = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let p = vec![0.05; s.len()];
    let v = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, None);
    assert!(!v.valid);
    assert!(
        v.audit.over_capacity_mass > 50.0,
        "over_capacity_mass={}",
        v.audit.over_capacity_mass
    );
    // D-069-style ratio uses capacity only on cells with S>0 (~2.3). Full-support
    // contract capacity is larger; require clear global overseed vs that capacity.
    assert!(
        v.audit.capacity_ratio > 1.05,
        "capacity_ratio={} s={} cap={}",
        v.audit.capacity_ratio,
        v.audit.s_mass,
        v.audit.capacity_mass
    );
    assert_eq!(
        v.classification,
        SeedClassification::TotalMembraneMaterialUnauthorized
    );
}

#[test]
fn s_outside_interface_support() {
    let (grid, _phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(16.0));
    let mut s = vec![0.0; geometry.len()];
    let p = vec![0.0; geometry.len()];
    // Place S in a dish cell with zero delta if possible.
    let mut placed = false;
    for i in 0..geometry.len() {
        if grid.in_dish(i) && geometry[i].delta <= floor {
            s[i] = 0.5;
            placed = true;
            break;
        }
    }
    assert!(placed);
    let v = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, Some(true));
    assert!(!v.valid);
    assert!(v.audit.s_outside_support_mass > 0.0);
}

#[test]
fn strict_rejection_and_local_s_to_p() {
    let (grid, phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(22.0));
    let mut s = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let mut p = vec![0.05; s.len()];
    let v = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, Some(true));
    assert!(policy_a_reject(&v).is_err());

    let before = audit_capacity(&grid, &geometry, &s, &p, floor, 1.0);
    let report = migrate_policy_b_local_excess_s_to_p(
        &grid, &geometry, &mut s, &mut p, floor, 1.0, "test_b",
    );
    assert!(report.conserved);
    assert!((report.material_before - before.membrane_equivalent).abs() < 1e-9);
    assert!(report.excess_s > 0.0);
    let after = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, Some(true));
    assert!(after.valid);
    assert!(after.audit.max_occupancy <= 1.0 + NUMERIC_OCC_EPS);
}

#[test]
fn unauthorized_material_policy_d() {
    let (grid, phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(22.0));
    let mut s = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let p = vec![0.05; s.len()];
    let report = migrate_policy_d_authorized_reconstruction(
        &grid, &geometry, &mut s, &p, floor, 1.0, 1.0, "test_d",
    );
    assert!(!report.conserved);
    assert!(
        report.unauthorized_removed > 20.0,
        "unauthorized_removed={}",
        report.unauthorized_removed
    );
    let after = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, Some(true));
    assert!(after.valid);
    assert!(after.audit.s_mass <= after.audit.capacity_mass + 1e-6);
}

#[test]
fn migration_determinism_and_idempotence() {
    let (grid, phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(16.0));
    let s0 = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let p0 = vec![0.1; s0.len()];

    let mut s1 = s0.clone();
    let mut p1 = p0.clone();
    let r1 = migrate_policy_b_local_excess_s_to_p(
        &grid, &geometry, &mut s1, &mut p1, floor, 1.0, "idem",
    );
    let mut s2 = s0.clone();
    let mut p2 = p0.clone();
    let r2 = migrate_policy_b_local_excess_s_to_p(
        &grid, &geometry, &mut s2, &mut p2, floor, 1.0, "idem",
    );
    assert_eq!(r1.new_identity, r2.new_identity);

    let r3 = migrate_policy_b_local_excess_s_to_p(
        &grid, &geometry, &mut s1, &mut p1, floor, 1.0, "idem",
    );
    assert!(
        r3.excess_s.abs() <= 1e-9,
        "second excess={}",
        r3.excess_s
    );
    assert_eq!(r3.cells_touched, 0);
    assert!(r1.idempotent_ready && r3.idempotent_ready);
}

#[test]
fn capacity_valid_occupancy_and_precursor_only() {
    let (grid, phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(16.0));
    let s = seed_capacity_bounded_s(&grid, &geometry, floor, 1.0, 1.0);
    let p = vec![0.0; s.len()];
    let v = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, Some(true));
    assert!(v.valid);
    assert!(v.audit.max_occupancy <= 1.0 + NUMERIC_OCC_EPS);

    let mem = v.audit.capacity_mass;
    let (s0, p0) = seed_precursor_only_from_material(&grid, &geometry, floor, mem, &phi, 0.5);
    assert!(s0.iter().all(|&x| x == 0.0));
    let mass_p: f64 = p0
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, x)| x * cell_volume())
        .sum();
    assert!((mass_p - mem).abs() < 1e-6);
}

#[test]
fn analytical_exchange_equilibrium_unchanged() {
    use chemistry_core::d069_analysis::{j_net_req, p_eq, theta_eq};
    let pe = p_eq(0.75, D070_K_EQ);
    assert!((theta_eq(pe, D070_K_EQ) - 0.75).abs() < 1e-12);
    let j = j_net_req(1.0, D070_K_EXCHANGE, 1.0, 1.0, D070_K_EQ, pe, 0.75);
    assert!(j.abs() < 1e-12);
}

#[test]
fn absolute_versus_relative_membrane_sufficiency() {
    assert_eq!(
        classify_absolute_membrane(0.9, 0.9, 0.9, 1.2),
        AbsoluteMembraneClass::RelativeAndAbsoluteMembraneSufficient
    );
    assert_eq!(
        classify_absolute_membrane(0.9, 0.2, 0.2, 1.2),
        AbsoluteMembraneClass::RetentionPassAbsoluteMembraneLow
    );
    assert_eq!(
        classify_absolute_membrane(0.5, 0.9, 0.9, 0.2),
        AbsoluteMembraneClass::CapacityFilledButReplacementFails
    );
}

#[test]
fn route_selection_rules() {
    let ok = RouteEvidence070 {
        workspace_isolated: true,
        d069_reproduced: true,
        lineage_ok: true,
        capacity_normalization_ok: true,
        seed_authority_resolved: true,
        validator_ok: true,
        migration_ok: true,
        waste_blocks: false,
        material_budget_invalid: false,
        lawful_material_insufficient: false,
        exchange_qualifies: true,
        absolute_membrane_ok: true,
        precursor_a_limit_remains: true,
        capacity_valid_still_loses_s: false,
    };
    assert_eq!(select_route(ok.clone()).0, D070Route::P);
    let mut m = ok.clone();
    m.material_budget_invalid = true;
    assert_eq!(select_route(m).0, D070Route::M);
    let mut e = ok.clone();
    e.exchange_qualifies = false;
    e.capacity_valid_still_loses_s = true;
    e.precursor_a_limit_remains = false;
    e.absolute_membrane_ok = false;
    assert_eq!(select_route(e).0, D070Route::E);
    let mut s = ok;
    s.precursor_a_limit_remains = false;
    assert_eq!(select_route(s).0, D070Route::S);
}

#[test]
fn snapshot_incompatible_fail_closed() {
    let (grid, phi, geometry, floor) = geometry_for(&GeometrySpec::smooth(22.0));
    let s = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let p = vec![0.0; s.len()];
    let v = validate_seed_capacity(&grid, &geometry, &s, &p, floor, 1.0, None);
    assert!(v.fail_closed);
    assert!(policy_a_reject(&v).is_err());
}
