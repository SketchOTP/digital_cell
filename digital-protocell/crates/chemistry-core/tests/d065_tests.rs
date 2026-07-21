//! Focused D-065 coverage: canonical net flux, parity, topology necessity, route selection.

use chemistry_core::d065_analysis::*;

#[test]
fn seal_and_frozen_constants() {
    assert_eq!(D065_STARTING_COMMIT, "4260a64");
    assert_eq!(
        D065_STARTING_TAG,
        "D-064-connected-geometry-coupled-failure-audit"
    );
    assert_eq!(D065_D064_CONCLUSION, "D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT");
    assert_eq!(
        D065_D064_RECORD,
        "CONNECTED_GEOMETRY_STATIC_CAPACITY_QUALIFIED_COUPLED_CAUSE_UNRESOLVED"
    );
    assert_eq!(
        D065_D063_RANKING_INVALIDATED,
        "D063_TOPOLOGY_RANKING_INVALIDATED_BY_RESOURCE_METRIC_DEFECT"
    );
    assert!((D065_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    assert_eq!(
        D065_AGENT_MEMORY_ID,
        "D-20260721-d065-canonical-resource-sufficiency-topology-necessity"
    );
    assert!(legacy_metrics_unauthorized_for_ranking());
    assert!(shadow_isolation_ok(false, false, false));
    assert!(!shadow_isolation_ok(true, false, false));
}

#[test]
fn d064_metric_defect_reproduction() {
    // Frozen D-064 numbers.
    let legacy_static = legacy_static_chi(368.0, 1364.0, 0.005);
    assert!((legacy_static - 13.55).abs() < 0.5);
    let legacy_coupled = legacy_coupled_proxy_chi(2791.0, 1364.0, 1076);
    assert!(legacy_coupled < 0.5);
    let canonical = window_from_signed_nets(0.0, 1395.5, 0.0, 1395.5, 1364.0, 5.376, 1076);
    assert!(canonical.chi_min() > 1.05);
    assert!(d064_metric_defect_reproduced(
        legacy_static,
        legacy_coupled,
        canonical.chi_min(),
        0.40,
        true,
        true,
        true
    ));
    assert!(!d064_metric_defect_reproduced(
        legacy_static,
        legacy_coupled,
        canonical.chi_min(),
        0.90,
        false,
        false,
        false
    ));
}

#[test]
fn canonical_signed_net_flux_inward_outward() {
    let inward = vec![AcceptedEnvFluxEvent {
        resource_is_n: true,
        amount_signed: 4.0,
        direction_into_interior: 1.0,
        is_carrier: true,
        is_passive: false,
        exterior_connected: true,
        closed_vesicle: false,
        step_accepted: true,
    }];
    let w_in = evaluate_canonical_net_flux(&inward, 100.0, 1.0, 1);
    assert!(w_in.chi_n() > 0.0);
    assert!((w_in.j_n_net() - 4.0).abs() < 1e-12);

    let outward = vec![AcceptedEnvFluxEvent {
        resource_is_n: true,
        amount_signed: 4.0,
        direction_into_interior: -1.0,
        is_carrier: true,
        is_passive: false,
        exterior_connected: true,
        closed_vesicle: false,
        step_accepted: true,
    }];
    let w_out = evaluate_canonical_net_flux(&outward, 100.0, 1.0, 1);
    assert!(w_out.j_n_net() < 0.0);
}

#[test]
fn gross_versus_net_and_recirculation() {
    let events = vec![
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 5.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 5.0,
            direction_into_interior: -1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
    ];
    let w = evaluate_canonical_net_flux(&events, 200.0, 2.0, 2);
    assert!((w.j_n_in_accepted - 5.0).abs() < 1e-12);
    assert!((w.j_n_out_accepted - 5.0).abs() < 1e-12);
    assert!(w.j_n_net().abs() < 1e-12);
    assert_eq!(w.chi_n(), 0.0);
}

#[test]
fn rejected_step_and_closed_vesicle_exclusion() {
    let events = vec![
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 10.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: false,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: false,
            amount_signed: 10.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: false,
            closed_vesicle: true,
            step_accepted: true,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 1.0,
            direction_into_interior: 0.0,
            is_carrier: false,
            is_passive: true,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
    ];
    let w = evaluate_canonical_net_flux(&events, 50.0, 1.0, 0);
    assert!(w.j_n_net().abs() < 1e-12);
    assert!(w.j_f_net().abs() < 1e-12);
    assert!(w.j_n_rejected_excluded > 0.0);
    assert!(w.j_f_closed_vesicle_excluded > 0.0);
    assert!(w.j_n_recirculation_excluded > 0.0);
}

#[test]
fn static_coupled_parity_and_passive_carrier_split() {
    let s = window_from_signed_nets(1.0, 2.0, 1.0, 2.0, 100.0, 0.01, 1);
    let c = window_from_signed_nets(1.0, 2.0, 1.0, 2.0, 100.0, 0.01, 1);
    assert!(static_coupled_parity(s, c));
    assert!((s.chi_n() - c.chi_n()).abs() < 1e-12);
    assert!((s.j_n_passive_net - 1.0).abs() < 1e-12);
    assert!((s.j_n_carrier_net - 2.0).abs() < 1e-12);
    // Carrier disabled → only passive.
    let passive_only = window_from_signed_nets(1.0, 0.0, 1.0, 0.0, 100.0, 0.01, 1);
    assert!((passive_only.j_n_net() - 1.0).abs() < 1e-12);
    // Passiveive disabled → only carrier.
    let carrier_only = window_from_signed_nets(0.0, 2.0, 0.0, 2.0, 100.0, 0.01, 1);
    assert!((carrier_only.j_n_net() - 2.0).abs() < 1e-12);
}

#[test]
fn smooth_versus_connected_classification() {
    assert_eq!(
        classify_topology_necessity(2.0, 5.0),
        TopologyNecessityClass::SmoothSufficient
    );
    assert_eq!(
        classify_topology_necessity(0.5, 1.2),
        TopologyNecessityClass::ConnectedAreaNecessary
    );
    assert_eq!(
        classify_topology_necessity(0.5, 0.8),
        TopologyNecessityClass::ConnectedAreaInsufficient
    );
    assert_eq!(
        classify_topology_necessity(10.0, 20.0),
        TopologyNecessityClass::ResourceOversupply
    );
    assert!((delta_chi_topology(2.0, 1.0) - 1.0).abs() < 1e-12);
    assert!(connected_membrane_not_required(1.2));
    assert!(!connected_membrane_not_required(0.9));
    assert!(connected_area_delivery_not_causally_useful(2.0, 1.0, false));
    assert!(!connected_area_delivery_not_causally_useful(2.0, 1.0, true));
}

#[test]
fn resource_fate_ledger_and_classification() {
    let ok = ResourceFateLedger {
        j_net: 10.0,
        u_activation: 6.0,
        u_other: 1.0,
        delta_inventory: 3.0,
        reexport: 0.0,
        reverse_carrier: 0.0,
        numerical_correction: 0.0,
        rejected_excluded: 0.0,
    };
    assert!(ok.closes(1e-6));
    assert_eq!(
        classify_resource_fate(ok, true),
        ResourceFateClass::ActivationConsumed
    );
    let reexport = ResourceFateLedger {
        j_net: 10.0,
        u_activation: 1.0,
        u_other: 0.0,
        delta_inventory: 1.0,
        reexport: 8.0,
        reverse_carrier: 0.0,
        numerical_correction: 0.0,
        rejected_excluded: 0.0,
    };
    assert_eq!(
        classify_resource_fate(reexport, true),
        ResourceFateClass::RapidReexport
    );
    let broken = ResourceFateLedger {
        j_net: 10.0,
        u_activation: 1.0,
        u_other: 0.0,
        delta_inventory: 0.0,
        reexport: 0.0,
        reverse_carrier: 0.0,
        numerical_correction: 0.0,
        rejected_excluded: 0.0,
    };
    assert!(!broken.closes(1e-6));
    assert_eq!(
        classify_resource_fate(broken, false),
        ResourceFateClass::ResourceLedgerUnresolved
    );
}

#[test]
fn waste_destination_budgeting_and_sink() {
    let overcommit = WasteAuditEvidence {
        multiface_overcommit: true,
        perfect_sink_removes_rejection: false,
        carrier_disabled_removes_rejection: false,
        reduced_dt_removes_rejection: false,
        export_sign_inverted: false,
        exterior_w_rises_faster_than_dispersal: true,
        smooth_also_hits_ceiling: false,
        rejection_observed: true,
    };
    assert_eq!(
        classify_waste_rejection(overcommit),
        WasteRejectionClass::WDestinationOvercommit
    );
    let sink = WasteAuditEvidence {
        multiface_overcommit: false,
        perfect_sink_removes_rejection: true,
        carrier_disabled_removes_rejection: false,
        reduced_dt_removes_rejection: false,
        export_sign_inverted: false,
        exterior_w_rises_faster_than_dispersal: true,
        smooth_also_hits_ceiling: false,
        rejection_observed: true,
    };
    assert_eq!(
        classify_waste_rejection(sink),
        WasteRejectionClass::WExternalDispersalLimit
    );
}

#[test]
fn complete_a_ledger_and_eta() {
    let ledger = ALedger {
        g_activation: 5.0,
        l_catalyst: 1.0,
        l_structure: 0.5,
        l_precursor: 2.0,
        l_decay: 0.5,
        j_out: 1.0,
        j_in: 0.0,
        delta_a: 0.0,
        activation_requested: 12.0,
        activation_accepted: 5.0,
        j_n_net: 20.0,
        j_f_net: 20.0,
    };
    assert!(ledger.closes(1e-6));
    assert!((ledger.eta_delivery_to_a() - 0.25).abs() < 1e-12);
    assert_eq!(ledger.dominant_sink(), "precursor");
    assert_eq!(
        classify_a_balance(ledger, true, 0.4),
        ABalanceClass::ActivationCapacityLimit
    );
    let unused = ALedger {
        g_activation: 0.1,
        l_catalyst: 0.0,
        l_structure: 0.0,
        l_precursor: 0.0,
        l_decay: 0.0,
        j_out: 0.0,
        j_in: 0.0,
        delta_a: 0.1,
        activation_requested: 0.1,
        activation_accepted: 0.1,
        j_n_net: 50.0,
        j_f_net: 50.0,
    };
    assert_eq!(
        classify_a_balance(unused, true, 0.3),
        ABalanceClass::ResourceDeliveryNotUsedByActivation
    );
}

#[test]
fn topology_necessity_and_route_selection_rules() {
    // Smooth sufficient + activation limited → Route A
    let ev_a = RouteEvidence065 {
        workspace_isolated: true,
        d064_reproduced: true,
        evaluator_ok: true,
        parity_ok: true,
        fate_ledger_ok: true,
        waste_provenance_ok: true,
        a_ledger_ok: true,
        chi_smooth_min: 19.0,
        chi_connected_best: 25.0,
        connected_improves_a: false,
        a_retention: 0.40,
        activation_limited: true,
        a_demand_limited: false,
        waste_execution_defect: true,
        closed_vesicle_chi_near_zero: true,
    };
    let (r, c) = select_route(ev_a);
    assert_eq!(r, D065Route::A);
    assert_eq!(
        c.as_str(),
        "D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED"
    );

    // Smooth sufficient + A demand → Route D
    let mut ev_d = ev_a;
    ev_d.activation_limited = false;
    ev_d.a_demand_limited = true;
    let (r, c) = select_route(ev_d);
    assert_eq!(r, D065Route::D);
    assert_eq!(
        c.as_str(),
        "D065_RESOURCE_DELIVERY_SUFFICIENT_A_DEMAND_LIMITED"
    );

    // Smooth sufficient, no A bottleneck classified → Route S
    let mut ev_s = ev_a;
    ev_s.activation_limited = false;
    ev_s.a_demand_limited = false;
    ev_s.waste_execution_defect = false;
    ev_s.a_retention = 0.85;
    let (r, c) = select_route(ev_s);
    assert_eq!(r, D065Route::S);
    assert_eq!(
        c.as_str(),
        "D065_SMOOTH_MEMBRANE_RESOURCE_CAPACITY_SUFFICIENT"
    );

    // Connected adds χ but not A → Route U when smooth fails
    let ev_u = RouteEvidence065 {
        workspace_isolated: true,
        d064_reproduced: true,
        evaluator_ok: true,
        parity_ok: true,
        fate_ledger_ok: true,
        waste_provenance_ok: true,
        a_ledger_ok: true,
        chi_smooth_min: 0.5,
        chi_connected_best: 2.0,
        connected_improves_a: false,
        a_retention: 0.40,
        activation_limited: false,
        a_demand_limited: false,
        waste_execution_defect: false,
        closed_vesicle_chi_near_zero: true,
    };
    let (r, c) = select_route(ev_u);
    assert_eq!(r, D065Route::U);
    assert_eq!(c.as_str(), "D065_CONNECTED_MEMBRANE_NOT_PRIMARY_REPAIR");

    // Workspace failure
    let mut bad = ev_a;
    bad.workspace_isolated = false;
    assert_eq!(
        select_route(bad).1,
        D065PrimaryConclusion::WorkspaceScopeNotIsolated
    );
}

#[test]
fn traversal_order_invariance_of_evaluator() {
    let mut a = vec![
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 1.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 0.5,
            direction_into_interior: -1.0,
            is_carrier: false,
            is_passive: true,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: false,
            amount_signed: 2.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
    ];
    let w1 = evaluate_canonical_net_flux(&a, 80.0, 0.5, 3);
    a.reverse();
    let w2 = evaluate_canonical_net_flux(&a, 80.0, 0.5, 3);
    assert!((w1.j_n_net() - w2.j_n_net()).abs() < 1e-12);
    assert!((w1.j_f_net() - w2.j_f_net()).abs() < 1e-12);
    assert!((w1.chi_min() - w2.chi_min()).abs() < 1e-12);
}
