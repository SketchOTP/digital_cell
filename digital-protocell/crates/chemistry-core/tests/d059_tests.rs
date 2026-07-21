//! Focused D-059 coverage: Route V repro, matched radius, global k_T, size/area routes.

use chemistry_core::d057_analysis::scaling_exponent;
use chemistry_core::d059_analysis::*;

#[test]
fn seal_and_preservation_constants() {
    assert_eq!(D059_STARTING_COMMIT, "482882d");
    assert_eq!(D059_STARTING_TAG, "D-058-corrected-carrier-normalization-audit");
    assert_eq!(D059_D056_TAG, "D-056-waste-coupled-resource-carrier-fail");
    assert_eq!(D059_D057_TAG, "D-057-carrier-geometry-driving-force-audit");
    assert_eq!(D059_D058_CONCLUSION, "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT");
    assert_eq!(
        D059_PRESERVATION_RECORD,
        "EXTERNAL_MEMBRANE_CARRIER_SURFACE_CAPACITY_LIMIT_CONFIRMED"
    );
}

#[test]
fn d058_route_v_reproduction_rules() {
    assert!(d058_route_v_reproduced(
        194.3,
        false,
        "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT"
    ));
    assert!(!d058_route_v_reproduced(2.0, false, "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT"));
    assert!(!d058_route_v_reproduced(194.3, true, "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT"));
}

#[test]
fn matched_state_radius_construction_and_exponents() {
    let states: Vec<_> = D059_MATCHED_RADII
        .iter()
        .map(|&r| matched_disk_state(r, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 0.02, 1.2, 0.4, 0.08))
        .collect();
    assert_eq!(states.len(), 12);
    let (p_m, p_t) = fit_matched_exponents(&states);
    let p_m = p_m.unwrap();
    let p_t = p_t.unwrap();
    // Ideal disk: demand∝R², capacity∝R
    assert!((p_m - 2.0).abs() < 0.05, "p_m={p_m}");
    assert!((p_t - 1.0).abs() < 0.05, "p_t={p_t}");
    let cls = classify_matched_scaling(p_m, p_t, 7.81, 1.07);
    assert_eq!(cls, MatchedScalingClass::D058RadiusExponentConfounded);
}

#[test]
fn global_k_t_enforcement_and_radius_specific_rejection() {
    let ladder = select_global_k_t_ladder(0.007, 1.36, 2.0).unwrap();
    assert!(ladder.len() <= D059_MAX_GLOBAL_KT);
    assert!(ladder.windows(2).all(|w| w[1] >= w[0]));
    // Same k across radii is required
    let ok = reject_radius_specific_k_t(&[(8.0, 0.1), (16.0, 0.1), (24.0, 0.1)]);
    assert!(ok);
    let bad = reject_radius_specific_k_t(&[(8.0, 0.1), (16.0, 0.5)]);
    assert!(!bad);
}

#[test]
fn contiguous_viable_range_classification() {
    assert_eq!(
        longest_contiguous_viable_radii(&[8.0, 10.0, 12.0, 20.0]),
        3
    );
    assert!(longest_contiguous_viable_radii(&[8.0, 10.0]) < D059_CONTIGUOUS_RADII_MIN);
    assert!(viable_frontier_region_ok(&[8.0, 10.0, 12.0], &[0.01, 0.02, 0.04]));
    assert!(!viable_frontier_region_ok(&[8.0], &[0.01, 0.02]));
}

#[test]
fn restoring_size_classification() {
    let basin = classify_restoring_size(
        &[(10.0, 0.2), (12.0, 0.05), (14.0, 0.0), (16.0, -0.05), (18.0, -0.2)],
        14.0,
        1e-3,
    );
    assert_eq!(basin, RestoringSizeClass::RestoringSizeBasin);
    let collapse = classify_restoring_size(
        &[(10.0, -0.2), (12.0, -0.1), (16.0, -0.05), (18.0, -0.2)],
        14.0,
        1e-3,
    );
    assert_eq!(collapse, RestoringSizeClass::RunawayCollapse);
}

#[test]
fn radius_perturbation_and_viability_predicates() {
    assert!(radius_provisionally_viable(
        1.1, 1.1, true, true, true, true, true, true, true, true, true
    ));
    assert!(!radius_provisionally_viable(
        1.0, 1.1, true, true, true, true, true, true, true, true, true
    ));
    // Perturbation neighbors returning toward basin: encoded via restoring classifier
    let pert = classify_restoring_size(
        &[
            (14.0 * 0.8, 0.3),
            (14.0 * 0.9, 0.15),
            (14.0 * 1.1, -0.12),
            (14.0 * 1.2, -0.25),
        ],
        14.0,
        1e-3,
    );
    assert_eq!(pert, RestoringSizeClass::RestoringSizeBasin);
}

#[test]
fn starvation_and_non_resurrection_rules() {
    // No import without exterior N or F → drive/chi collapse
    let chi_starved = predicted_chi(0.0, 10.0);
    assert!(chi_starved < D059_CHI_VIABLE);
    // Carrier disabled restores passive failure
    assert!(shadow_isolation_ok(false, false));
    assert!(!shadow_isolation_ok(true, false));
    // Closed vesicle topology rejected
    assert!(!topology_admissible(
        TopologyClass::CClosedInternalVesicles,
        true,
        true
    ));
}

#[test]
fn area_requirement_and_amplification_bins() {
    let a_req = required_carrier_area(10.0, 0.1, 2.0).unwrap();
    assert!((a_req - 50.0).abs() < 1e-12);
    let alpha = area_amplification(50.0, 10.0).unwrap();
    assert!((alpha - 5.0).abs() < 1e-12);
    assert_eq!(classify_amplification(1.1), AmplificationBin::Leq125);
    assert_eq!(classify_amplification(1.5), AmplificationBin::From125To2);
    assert_eq!(classify_amplification(3.0), AmplificationBin::From2To5);
    assert_eq!(classify_amplification(7.0), AmplificationBin::From5To10);
    assert_eq!(classify_amplification(12.0), AmplificationBin::Gt10);
}

#[test]
fn material_budget_and_bootstrap_feasibility() {
    let ok = material_budget(20.0, 5.0, 4.0, 2.0, 1.0, 6.0, 0.5, 0.01);
    assert!(ok.bootstrap_possible);
    assert!((ok.delta_m_s - 6.0).abs() < 1e-12);
    let fail = material_budget(1.0, 5.0, 0.0, 0.0, 1.0, 10.0, 0.5, 0.01);
    assert!(!fail.bootstrap_possible);
    assert_eq!(
        fail.bootstrap_possible,
        false // INTERNAL_AREA_BOOTSTRAP_IMPOSSIBLE condition
    );
}

#[test]
fn environmental_connectivity_and_topologies() {
    assert!(environmentally_connected(true, true, false));
    assert!(!environmentally_connected(true, true, true));
    assert!(!environmentally_connected(true, false, false));
    assert!(topology_admissible(
        TopologyClass::AExternalInvaginations,
        true,
        true
    ));
    assert!(topology_admissible(
        TopologyClass::BExteriorConnectedChannels,
        true,
        true
    ));
    assert!(!topology_admissible(
        TopologyClass::DDistributedInternalCarrierMembrane,
        false,
        true
    ));
}

#[test]
fn explicit_area_accounting_and_shadow_isolation() {
    assert!(area_multiplier_valid(2.0, 1.0, false, false, true));
    assert!(!area_multiplier_valid(2.0, 1.0, true, false, true)); // free scalar
    assert!(!area_multiplier_valid(2.0, 0.0, false, false, true)); // no mature S
    let j = amplified_throughput(0.1, 50.0);
    assert!((j - 5.0).abs() < 1e-12);
    assert!(shadow_isolation_ok(false, false));
}

#[test]
fn route_selection_rules() {
    let (r, c) = select_route(RouteEvidence059 {
        workspace_isolated: true,
        d058_route_v_reproduced: true,
        accounting_ok: true,
        numerical_ok: true,
        contiguous_viable_radii: true,
        restoring_basin: true,
        size_limit_no_restore: false,
        starvation_ok: true,
        area_amplification_bounded: false,
        material_bootstrap_ok: false,
        topology_justified: false,
        area_architecture_not_justified: false,
        carrier_surface_rejected: false,
    });
    assert_eq!(r, D059Route::S);
    assert_eq!(c.as_str(), "D059_EXTERNAL_CARRIER_RESTORING_SIZE_BASIN");

    let (_, no_v) = select_route(RouteEvidence059 {
        workspace_isolated: true,
        d058_route_v_reproduced: false,
        accounting_ok: true,
        numerical_ok: true,
        contiguous_viable_radii: false,
        restoring_basin: false,
        size_limit_no_restore: false,
        starvation_ok: false,
        area_amplification_bounded: false,
        material_bootstrap_ok: false,
        topology_justified: false,
        area_architecture_not_justified: false,
        carrier_surface_rejected: false,
    });
    assert_eq!(no_v.as_str(), "D059_D058_ROUTE_V_NOT_REPRODUCED");

    let (rm, cm) = select_route(RouteEvidence059 {
        workspace_isolated: true,
        d058_route_v_reproduced: true,
        accounting_ok: true,
        numerical_ok: true,
        contiguous_viable_radii: false,
        restoring_basin: false,
        size_limit_no_restore: false,
        starvation_ok: false,
        area_amplification_bounded: true,
        material_bootstrap_ok: true,
        topology_justified: true,
        area_architecture_not_justified: false,
        carrier_surface_rejected: false,
    });
    assert_eq!(rm, D059Route::M);
    assert_eq!(cm.as_str(), "D059_INTERNAL_MEMBRANE_AREA_ARCHITECTURE_JUSTIFIED");
}

#[test]
fn frontier_region_and_scaling_helpers() {
    assert_eq!(
        classify_frontier_cell(1.1, 1.1, 0.5, false, false, false, false),
        FrontierRegion::ViableThroughput
    );
    assert_eq!(
        classify_frontier_cell(0.5, 0.5, 0.5, false, false, false, false),
        FrontierRegion::InsufficientImport
    );
    let exp = scaling_exponent(&[8.0, 16.0], &[64.0, 256.0]).unwrap();
    assert!((exp - 2.0).abs() < 1e-6);
    let _ = k_t_span(&[0.01, 1.0]);
    let _ = D058_RATE_SPAN_MAX;
}
