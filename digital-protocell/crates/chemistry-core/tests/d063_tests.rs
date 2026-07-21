//! Focused D-063 coverage: connectivity, area, material, carrier, route selection.

use chemistry_core::d063_analysis::*;
use chemistry_core::grid::Grid;

#[test]
fn seal_and_frozen_constants() {
    assert_eq!(D063_STARTING_COMMIT, "47f2abb");
    assert_eq!(D063_STARTING_TAG, "D-062-structural-maintenance-decay-review");
    assert_eq!(D063_D062_CONCLUSION, "D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW");
    assert_eq!(
        D063_D061_EXECUTION,
        "D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED"
    );
    assert_eq!(
        D063_D059_CONCLUSION,
        "D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN"
    );
    assert_eq!(
        D063_D058_CONCLUSION,
        "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT"
    );
    assert!((D063_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    assert_eq!(
        D063_AGENT_MEMORY_ID,
        "D-20260721-d063-environmentally-connected-membrane-invagination-architecture"
    );
    assert_eq!(
        D063_RECORD_SMALL_SIZE_CLOSED,
        "EXTERNAL_CARRIER_SMALL_SIZE_ROUTE_CLOSED"
    );
}

#[test]
fn d062_route_n_reproduction() {
    assert!(d062_route_n_reproduced(
        "D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW",
        1.1794749787217675,
        1.2187999967526635,
        12.7,
        false
    ));
    assert!(!d062_route_n_reproduced(
        "D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW",
        1.18,
        1.22,
        12.7,
        true
    ));
    assert!(rejected_architectures_disabled(
        false, false, false, false, false
    ));
    assert!(!rejected_architectures_disabled(
        true, false, false, false, false
    ));
}

#[test]
fn topology_flood_fill_and_classes() {
    let grid = Grid::new();
    let connected = exterior_connected_mask(&grid, &vec![0.0; grid.width * grid.height], 0.5);
    // Reservoir ring must seed exterior connectivity.
    let some = connected
        .iter()
        .enumerate()
        .find(|&(_, &c)| c)
        .map(|(i, _)| i)
        .expect("reservoir-connected extracellular cell");

    assert_eq!(
        classify_membrane_face(some, &connected, true, true),
        MembraneFaceClass::ExternalBoundary
    );
    assert_eq!(
        classify_membrane_face(some, &connected, true, false),
        MembraneFaceClass::ExteriorConnectedInvagination
    );
    let mut sealed = connected.clone();
    sealed[some] = false;
    assert_eq!(
        classify_membrane_face(some, &sealed, true, false),
        MembraneFaceClass::ClosedInternal
    );
    assert_eq!(
        classify_membrane_face(some, &connected, false, true),
        MembraneFaceClass::InvalidOrAmbiguous
    );
    assert!(MembraneFaceClass::ExternalBoundary.carrier_active());
    assert!(!MembraneFaceClass::ClosedInternal.carrier_active());
}

#[test]
fn closed_vesicle_rejection_and_invagination_connected() {
    let grid = Grid::new();
    let vesicles = GeometrySpec::closed_vesicles(22.0, 3, 3.0);
    let phi_v = generate_phi(&grid, &vesicles);
    let conn_v = exterior_connected_mask(&grid, &phi_v, D063_PHI_INTERIOR);
    let s_v = seed_mature_s_on_interfaces(&grid, &phi_v, 1.0);
    let base = smooth_baseline_length(22.0);
    let acc_v = account_geometry(&grid, &phi_v, &s_v, base, 22.0);
    assert!(acc_v.closed_internal_interface_length > 0.0);
    // Closed cavities must not inflate connected invagination area as environmental.
    // Some outer boundary still exists.
    assert!(acc_v.external_boundary_length > 0.0);

    let radial = GeometrySpec::radial(22.0, 6, 0.5, 2.5);
    let phi_r = generate_phi(&grid, &radial);
    let conn_r = exterior_connected_mask(&grid, &phi_r, D063_PHI_INTERIOR);
    let s_r = seed_mature_s_on_interfaces(&grid, &phi_r, 1.0);
    let acc_r = account_geometry(&grid, &phi_r, &s_r, base, 22.0);
    assert!(acc_r.connected_invagination_length > 0.0);
    assert!(acc_r.alpha_gamma > 1.0);
    // Channel cells inside radius should be exterior-connected.
    let mut found_connected_channel = false;
    for idx in 0..phi_r.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        let r = grid.distance_from_center(i, j);
        if r < 18.0 && phi_r[idx] < D063_PHI_INTERIOR && conn_r[idx] {
            found_connected_channel = true;
            break;
        }
    }
    assert!(found_connected_channel);
    // Vesicle interior extracellular should not be connected.
    let mut found_sealed = false;
    for idx in 0..phi_v.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        let r = grid.distance_from_center(i, j);
        if r < 12.0 && phi_v[idx] < D063_PHI_INTERIOR && !conn_v[idx] {
            found_sealed = true;
            break;
        }
    }
    assert!(found_sealed);
}

#[test]
fn physical_area_and_subdivision_invariance() {
    assert!((alpha_gamma(20.0, 10.0) - 2.0).abs() < 1e-15);
    assert!(subdivision_area_invariant(10.0, 4.0, 6.0, D063_AREA_TOL));
    assert!(!subdivision_area_invariant(10.0, 4.0, 5.0, D063_AREA_TOL));
    assert!(orientation_area_invariant(12.5, 12.5, D063_AREA_TOL));
}

#[test]
fn mature_s_material_and_ps_cost() {
    let b = material_budget_063(10.0, 5.0, 1.0, 2.0, 0.5, 1.0, 0.2, 0.01, 0.0);
    assert!((b.delta_m_s - 5.0).abs() < 1e-12);
    assert!((b.candidate_s_mass - 15.0).abs() < 1e-12);
    assert!((b.a_cost - 5.0).abs() < 1e-12);
    assert!((b.construction_time - 6.0).abs() < 1e-12);
    assert_eq!(
        b.feasibility,
        MaterialFeasibility::MaterialBuildableFromEndogenousP
    );
    let seed = material_budget_063(10.0, 1.0, 1.0, 5.0, 0.0, 1.0, 0.2, 0.01, 0.0);
    assert_eq!(
        seed.feasibility,
        MaterialFeasibility::MaterialAvailableFromInitialSeed
    );
    let bad = material_budget_063(10.0, 5.0, 1.0, 0.0, 0.0, 1.0, 0.2, 0.01, 3.0);
    assert_eq!(
        bad.feasibility,
        MaterialFeasibility::MaterialRequiresUnauthorizedSeed
    );
}

#[test]
fn carrier_face_selection_and_conservation() {
    assert!(carrier_face_selected(
        MembraneFaceClass::ExternalBoundary,
        1.0
    ));
    assert!(!carrier_face_selected(
        MembraneFaceClass::ClosedInternal,
        1.0
    ));
    assert!(!carrier_face_selected(
        MembraneFaceClass::ExteriorConnectedInvagination,
        0.0
    ));
    let xi = shadow_xi_connected(D063_FROZEN_KT, 1.0, 0.5, 0.01);
    assert!(xi > 0.0);
    assert!(nfw_conservation_ok(0.0, 0.0, 0.0, 1e-12));
    assert!(!nfw_conservation_ok(1e-6, 0.0, 0.0, 1e-12));
}

#[test]
fn area_throughput_scaling_and_depletion() {
    let areas = [10.0, 20.0, 40.0];
    let fluxes = [1.0, 2.0, 4.0];
    let p = fit_area_throughput_exponent(&areas, &fluxes).unwrap();
    assert!((p - 1.0).abs() < 1e-9);
    assert!(throughput_scales_with_area(p));
    assert!(!throughput_scales_with_area(2.0));

    let n = channel_concentration_profile(1.0, 10.0, 3.0, 11);
    let f = channel_concentration_profile(1.0, 10.0, 3.0, 11);
    assert!((n[0].1 - 1.0).abs() < 1e-12);
    assert!(n.last().unwrap().1 < 0.1);
    let fu = usable_connected_fraction(10.0, &n, &f, 0.2, 0.2);
    assert!(fu > 0.0 && fu < 1.0);
    assert_eq!(
        classify_channel_access(0.8, 2.0, 0.5),
        ChannelAccessClass::ConnectedAreaResourceAccessible
    );
    assert_eq!(
        classify_channel_access(0.2, 2.0, 0.5),
        ChannelAccessClass::ChannelDepletionLimit
    );
    assert_eq!(
        classify_channel_access(0.9, 0.2, 0.5),
        ChannelAccessClass::ChannelGeometryOversealed
    );
}

#[test]
fn bootstrap_persistence_damage_and_routes() {
    assert!(incremental_metabolic_return(2.0, 1.0, 0.5) > 1.0);
    assert_eq!(
        classify_bootstrap(true, true, true, false, false),
        BootstrapClass::ConnectedAreaBootstrapFeasible
    );
    assert_eq!(
        classify_bootstrap(false, false, false, false, true),
        BootstrapClass::ConnectedAreaBootstrapMaterialBlocked
    );
    assert_eq!(
        classify_topology_persistence(0.9, false, false, false),
        TopologyPersistenceClass::TopologyPersistsPassively
    );
    assert_eq!(
        classify_topology_persistence(0.6, false, false, false),
        TopologyPersistenceClass::TopologyRequiresMorphogeneticMaintenance
    );
    assert!(damage_seals_stop_import(true, false, 0.0, 1e-12));
    assert!(!damage_seals_stop_import(true, false, 1.0, 1e-12));

    let fail_ws = RouteEvidence063 {
        workspace_isolated: false,
        prior_route_reproduced: true,
        connectivity_resolved: true,
        area_accounting_ok: true,
        material_accounting_ok: true,
        carrier_parity_ok: true,
        throughput_scales_with_area: true,
        usable_throughput_ok: true,
        channel_depletion_limit: false,
        shadow_repair_ok: true,
        topology_persists: true,
        topology_requires_morphogenesis: false,
        bootstrap_feasible: true,
        bootstrap_material_blocked: false,
        damage_connectivity_ok: true,
        invagination_sufficient: true,
        channel_required: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    assert_eq!(
        select_route(fail_ws).1,
        D063PrimaryConclusion::WorkspaceScopeNotIsolated
    );

    let route_a = RouteEvidence063 {
        workspace_isolated: true,
        prior_route_reproduced: true,
        connectivity_resolved: true,
        area_accounting_ok: true,
        material_accounting_ok: true,
        carrier_parity_ok: true,
        throughput_scales_with_area: true,
        usable_throughput_ok: true,
        channel_depletion_limit: false,
        shadow_repair_ok: true,
        topology_persists: true,
        topology_requires_morphogenesis: false,
        bootstrap_feasible: true,
        bootstrap_material_blocked: false,
        damage_connectivity_ok: true,
        invagination_sufficient: true,
        channel_required: false,
        accounting_ok: true,
        numerical_ok: true,
    };
    let (r, c) = select_route(route_a);
    assert_eq!(r, D063Route::A);
    assert_eq!(
        c,
        D063PrimaryConclusion::ExternalInvaginationArchitectureJustified
    );

    let route_c = RouteEvidence063 {
        usable_throughput_ok: false,
        channel_depletion_limit: true,
        ..route_a
    };
    assert_eq!(select_route(route_c).0, D063Route::C);

    let route_m = RouteEvidence063 {
        bootstrap_feasible: false,
        bootstrap_material_blocked: true,
        ..route_a
    };
    assert_eq!(select_route(route_m).0, D063Route::M);
}

#[test]
fn fixed_vs_dynamic_topology_classifier_labels() {
    assert_eq!(
        TopologyPersistenceClass::TopologyPersistsPassively.as_str(),
        "TOPOLOGY_PERSISTS_PASSIVELY"
    );
    assert_eq!(
        TopologyPersistenceClass::TopologyRequiresMorphogeneticMaintenance.as_str(),
        "TOPOLOGY_REQUIRES_MORPHOGENETIC_MAINTENANCE"
    );
}

#[test]
fn trapped_resource_rejection_rule() {
    // Closed faces never selected for carrier even with high local N/F.
    assert!(!carrier_face_selected(
        MembraneFaceClass::ClosedInternal,
        10.0
    ));
    assert!(!MembraneFaceClass::ClosedInternal.carrier_active());
}
