//! Focused D-064 coverage: χ accounting, rejection, budgets, seeds, route selection.

use chemistry_core::d063_analysis::{
    generate_phi, seed_mature_s_on_interfaces, smooth_baseline_length, GeometrySpec,
    MembraneFaceClass, D063_PHI_INTERIOR,
};
use chemistry_core::d064_analysis::*;
use chemistry_core::grid::Grid;

#[test]
fn seal_and_frozen_constants() {
    assert_eq!(D064_STARTING_COMMIT, "3ab07cb");
    assert_eq!(D064_STARTING_TAG, "D-063-connected-membrane-architecture-review");
    assert_eq!(D064_D063_CONCLUSION, "D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE");
    assert!((D064_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    assert_eq!(
        D064_AGENT_MEMORY_ID,
        "D-20260721-d064-connected-geometry-coupled-rejection-decomposition"
    );
    assert_eq!(
        D064_RECORD_STATIC_CAPACITY,
        "CONNECTED_AREA_STATIC_CAPACITY_QUALIFIED_COUPLED_CAUSE_UNRESOLVED"
    );
}

#[test]
fn d063_failure_reproduction_predicate() {
    assert!(d063_failure_reproduced(
        true, 0.40, 368.0, 227.0, 1076, false
    ));
    assert!(!d063_failure_reproduced(
        true, 0.85, 368.0, 360.0, 2500, true
    ));
    let legacy = legacy_d063_chi_proxy(2791.0, 1364.0, 1076);
    assert!(legacy < 0.5);
    let canonical = chi_ratio(2791.0 / 2.0, productive_demand(1364.0, 5.376));
    assert!(canonical > 1.05);
}

#[test]
fn canonical_chi_requested_vs_accepted() {
    let requested = legacy_analytical_requested_capacity(368.0, 0.005);
    let accepted_w = ResourceSufficiencyWindow {
        j_n_passive_accepted: 0.0,
        j_n_carrier_accepted: requested * 0.1,
        j_f_passive_accepted: 0.0,
        j_f_carrier_accepted: requested * 0.1,
        l_n_required: productive_demand(1000.0, 0.005),
        l_f_required: productive_demand(1000.0, 0.005),
        accepted_steps: 1,
        window_time: 0.005,
    };
    assert!(requested > accepted_w.j_n_carrier_accepted);
    assert!(static_coupled_accounting_mismatch(true, false, false));
    assert!(!static_coupled_accounting_mismatch(false, false, false));
    // Rejected-step flux must not enter numerator: zero accepted → χ=0 when demand>0.
    let rejected_excluded = ResourceSufficiencyWindow {
        j_n_passive_accepted: 0.0,
        j_n_carrier_accepted: 0.0,
        j_f_passive_accepted: 0.0,
        j_f_carrier_accepted: 0.0,
        l_n_required: 1.0,
        l_f_required: 1.0,
        accepted_steps: 0,
        window_time: 0.0,
    };
    assert_eq!(rejected_excluded.chi_n(), 0.0);
}

#[test]
fn first_rejection_classification() {
    assert_eq!(
        classify_rejection_from_detail("POSITIVITY_LIMIT", "next nutrient[12]=-1e-5 < NEG_CLAMP", true, 2.0, 0.1, 0.1),
        RejectionClass::CarrierNOverdraw
    );
    assert_eq!(
        classify_rejection_from_detail("POSITIVITY_LIMIT", "surface_exchange_reject:CapacityExceeded", false, 0.0, 0.0, 0.0),
        RejectionClass::PSExchangeOverdraw
    );
    assert_eq!(
        classify_rejection_from_detail(
            "IncomingStateInvalid",
            "waste:excessive concentration at 20073: 10.007",
            true,
            0.0,
            0.0,
            0.0
        ),
        RejectionClass::CarrierWOverdraw
    );
}

#[test]
fn cell_budget_and_joint_allocator() {
    let reqs = vec![
        CarrierFaceRequest {
            inside: 0,
            outside: 1,
            face_id: 0,
            xi_req: 4.0,
            topology: MembraneFaceClass::ExternalBoundary,
        },
        CarrierFaceRequest {
            inside: 0,
            outside: 1,
            face_id: 1,
            xi_req: 4.0,
            topology: MembraneFaceClass::ExteriorConnectedInvagination,
        },
    ];
    let n = vec![0.0, 1.0];
    let f = vec![0.0, 1.0];
    let w = vec![10.0, 0.0];
    let audit = cell_budget_audit(&reqs, &n, &f, &w, &[0.0; 2], &[0.0; 2]);
    assert!(audit.multiface_defect);
    assert!(audit.max_omega_n > 1.0);
    let scaled = joint_allocate_faces(&reqs, &n, &f, &w);
    let mut rev = reqs.clone();
    rev.reverse();
    let scaled_rev = joint_allocate_faces(&rev, &n, &f, &w);
    // Order invariance after sorting by face_id amounts:
    let a = scaled.clone();
    let b: Vec<f64> = scaled_rev.into_iter().rev().collect();
    // face_id order restored by reversing the reversed list
    assert!(joint_allocator_order_invariant(&a, &b, 1e-12));
}

#[test]
fn channel_width_and_curvature_classification() {
    assert_eq!(
        classify_geometry_stiffness(1.0, 1.5, 2, 0.1),
        GeometryStiffnessClass::SubgridChannelStiffness
    );
    assert_eq!(
        classify_geometry_stiffness(2.5, 3.0, 5, 0.1),
        GeometryStiffnessClass::HighCurvatureFaceMultiplicity
    );
    assert_eq!(
        classify_geometry_stiffness(2.5, 3.0, 2, 0.1),
        GeometryStiffnessClass::GeometryDiscretizationAcceptable
    );
}

#[test]
fn ps_equilibrium_and_seed_families() {
    let teq = theta_eq(50.0, 0.05);
    assert!((teq - (50.0 * 0.05) / (1.0 + 50.0 * 0.05)).abs() < 1e-12);
    let e = exchange_imbalance(1.0, 0.02, 1.0, 0.05, 0.9);
    assert!(e < 0.0); // desorption loaded when theta high vs p
    assert_eq!(
        classify_seed_equilibrium(e, 0.2, false),
        SeedEquilibriumClass::PrebuiltSeedDesorptionLoaded
    );
    assert_eq!(
        classify_seed_equilibrium(0.0, 0.0, true),
        SeedEquilibriumClass::PrebuiltSeedMaterialInconsistent
    );

    let grid = Grid::new();
    let radial = sealed_radial_r22_spec();
    let phi = generate_phi(&grid, &radial);
    let smooth = GeometrySpec::smooth(22.0);
    let phi_s = generate_phi(&grid, &smooth);
    let s_smooth = seed_mature_s_on_interfaces(&grid, &phi_s, 1.0);
    let baseline: f64 = s_smooth
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, &v)| v)
        .sum();
    let redistributed = redistribute_s_conserve_total(&grid, &phi, baseline, 1.0);
    let total_r: f64 = redistributed
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, &v)| v)
        .sum();
    assert!((total_r - baseline).abs() < 1e-6 * (1.0 + baseline));
    let _ = (D063_PHI_INTERIOR, smooth_baseline_length(22.0));
}

#[test]
fn operator_isolation_and_ledger_helpers() {
    assert_eq!(
        classify_coupled_load(false, true, false, false, true),
        CoupledLoadClass::MembraneExchangeLoad
    );
    assert_eq!(
        classify_coupled_load(true, true, false, false, true),
        CoupledLoadClass::MultipleCoupledLoads
    );
    assert!(ledger_closes(10.0, 10.0, 1e-6));
    assert!(!ledger_closes(10.0, 11.0, 1e-6));
    assert!(short_screen_admits(1.1, 1.1, 0.85, 0.9, false, false));
    assert!(!short_screen_admits(1.1, 1.1, 0.5, 0.9, false, false));
}

#[test]
fn closed_vesicle_non_rescue_identity() {
    let grid = Grid::new();
    let vesicles = GeometrySpec::closed_vesicles(22.0, 4, 3.0);
    let acc = measure_geometry(&vesicles);
    assert!(acc.connected_invagination_length < 1e-9 || acc.closed_internal_interface_length > 0.0);
    // Closed regions must not contribute environmental carrier area in D-063 contract.
    assert!(acc.closed_internal_interface_length > 0.0);
    let _ = grid;
}

#[test]
fn route_selection_rules() {
    let base = RouteEvidence064 {
        workspace_isolated: true,
        d063_reproduced: true,
        accounting_reconciled: true,
        static_used_requested_flux: false,
        rejection_provenance_resolved: true,
        multiface_budget_defect: false,
        joint_allocator_rescues: false,
        geometry_discretization_defect: false,
        seed_nonequilibrium: false,
        seed_material_inconsistent: false,
        exchange_load_dominant: false,
        precursor_demand_dominant: false,
        aps_ledger_ok: true,
        short_screen_pass: false,
        authoritative_pass: false,
        upper_bound_restores_aps: false,
        upper_bound_still_collapses: false,
    };
    let mut a = base;
    a.static_used_requested_flux = true;
    assert_eq!(select_route(a).1.as_str(), "D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT");

    let mut b = base;
    b.multiface_budget_defect = true;
    b.joint_allocator_rescues = true;
    assert_eq!(select_route(b).1.as_str(), "D064_MULTIFACE_CARRIER_BUDGETING_DEFECT");

    let mut s = base;
    s.seed_nonequilibrium = true;
    assert_eq!(select_route(s).1.as_str(), "D064_PREBUILT_CONNECTED_SEED_NONEQUILIBRIUM");

    let mut n = base;
    n.upper_bound_still_collapses = true;
    assert_eq!(
        select_route(n).1.as_str(),
        "D064_CONNECTED_MEMBRANE_NOT_PRIMARY_COUPLED_REPAIR"
    );

    let mut fail = base;
    fail.d063_reproduced = false;
    assert_eq!(
        select_route(fail).1.as_str(),
        "D064_D063_COUPLED_FAILURE_NOT_REPRODUCED"
    );
}

#[test]
fn shadow_isolation_contract() {
    assert!(shadow_isolation_ok(false, false, false));
    assert!(!shadow_isolation_ok(true, false, false));
    assert!(!shadow_isolation_ok(false, true, false));
    assert!(!shadow_isolation_ok(false, false, true));
}
