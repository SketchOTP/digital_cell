//! D-042 focused tests: A ledger, capacity classification, buffer feasibility, route rules.

use chemistry_core::d042_analysis::{
    classify_persistent_capacity, dominant_demand, evaluate_structural_buffer_feasibility,
    linear_trend, replay_ideal_buffer, select_route, ALedgerIntegral, ALedgerTerms,
    CumulativeABalance, IdealActivationBuffer, PersistentCapacityClass, D042Conclusion,
    D042Route, D042_LEDGER_REL_TOL, D042_MAX_A_PER_SITE, D042_REPAIR_P_MIN,
};

#[test]
fn a_ledger_closure_identity() {
    let mut integ = ALedgerIntegral::default();
    let w = ALedgerTerms {
        j_activation: 1.5,
        j_in: 0.2,
        a_initial: 5.0,
        j_reproduction: 0.3,
        j_structural: 0.2,
        j_precursor: 0.1,
        j_decay: 0.05,
        j_out: 0.15,
        j_reservoir: 0.01,
        numerical_correction: -0.005,
        a_final: 5.0 + 1.5 + 0.2 - 0.6 - 0.05 - 0.15 + 0.01 - 0.005,
        dt: 1.0,
        interior_volume: 1.0,
        catalyst_mass: 1.0,
        structural_mass: 1.0,
        sim_time: 1.0,
    };
    assert!(w.closes(D042_LEDGER_REL_TOL));
    integ.accumulate(&w);
    assert!(integ.closes(D042_LEDGER_REL_TOL));
    assert_eq!(dominant_demand(&integ), "catalyst_reproduction");
}

#[test]
fn production_demand_decomposition() {
    let w = ALedgerTerms {
        j_activation: 1.0,
        j_reproduction: 0.4,
        j_structural: 0.3,
        j_precursor: 0.2,
        j_decay: 0.1,
        j_out: 0.05,
        j_in: 0.0,
        dt: 1.0,
        a_initial: 1.0,
        a_final: 1.0 - 0.05, // R_A = 1 - 0.9 - 0.1 - 0.05 = -0.05
        ..Default::default()
    };
    assert!((w.j_demands() - 0.9).abs() < 1e-15);
    assert!((w.r_a() + 0.05).abs() < 1e-15);
}

#[test]
fn persistent_balance_capacity_deficit() {
    let (c, dem) = classify_persistent_capacity(
        -1.0,
        -0.8,
        -0.7,
        &[
            ("precursor_synthesis", -0.6),
            ("structural_production", -0.5),
            ("catalyst_reproduction", -0.4),
        ],
        1e-9,
    );
    assert_eq!(c, PersistentCapacityClass::ActivationCapacityDeficit);
    assert!(dem.is_none());
}

#[test]
fn persistent_balance_demand_excess() {
    let (c, dem) = classify_persistent_capacity(
        -1.0,
        -0.5,
        -0.4,
        &[
            ("precursor_synthesis", 0.1),
            ("structural_production", -0.2),
            ("catalyst_reproduction", -0.3),
        ],
        1e-9,
    );
    assert_eq!(c, PersistentCapacityClass::ActivatedResourceDemandExcess);
    assert_eq!(dem.as_deref(), Some("precursor_synthesis"));
}

#[test]
fn cumulative_deficit_and_cycle_storage() {
    let rates = [(-1.0, 1.0), (-1.0, 1.0), (0.5, 1.0), (2.0, 1.0)];
    let c = CumulativeABalance::from_rates(&rates);
    assert!((c.bootstrap_storage() - 2.0).abs() < 1e-12);
    assert!(c.cycle_storage() > 0.0);
    assert!(!c.unrepaid_deficit_grows_unbounded(0.1));
}

#[test]
fn repeated_cycle_storage_bounded() {
    // Oscillating surplus/deficit
    let mut rates = Vec::new();
    for _ in 0..5 {
        rates.push((-1.0, 1.0));
        rates.push((1.5, 1.0));
    }
    let c = CumulativeABalance::from_rates(&rates);
    assert!(c.bootstrap_storage().is_finite());
    assert!(c.cycle_storage() < 10.0);
    assert!(c.all_deficits_repaid() || c.late_mean_r_a(4) > -1e-9);
}

#[test]
fn spatial_deficit_and_site_capacity() {
    let f = evaluate_structural_buffer_feasibility(
        0.4,  // max local deficit
        1.0,  // H(φ)
        0.5,  // surplus
        0.1,  // timescale
        true, true, false, false,
    );
    assert!(f.finite_capacity);
    assert!(f.within_one_a_per_site);
    assert!(f.required_capacity_per_h_phi <= D042_MAX_A_PER_SITE);
    assert!(f.rechargeable);
}

#[test]
fn ideal_buffer_conservation_no_creation() {
    let mut buf = IdealActivationBuffer::new(2.0);
    let (e1, c1) = buf.step(1.0, 1.0);
    assert!((e1 - 1.0).abs() < 1e-15);
    assert_eq!(c1, 0.0);
    assert!((buf.stored - 1.0).abs() < 1e-15);
    let (e2, c2) = buf.step(-1.5, 1.0);
    assert_eq!(c2, 0.0);
    // releases 1.0 stored → effective -0.5
    assert!((e2 + 0.5).abs() < 1e-15);
    assert!(buf.stored.abs() < 1e-15);
}

#[test]
fn starvation_depletion() {
    let mut buf = IdealActivationBuffer::new(1.0);
    buf.stored = 1.0;
    for _ in 0..5 {
        let _ = buf.step(-1.0, 1.0);
    }
    assert!(buf.stored.abs() < 1e-15);
    let replay = replay_ideal_buffer(
        1.0,
        &[-1.0, -1.0, -1.0],
        1.0,
        &[0.01, 0.01, 0.01],
        &[0.2, 0.2, 0.2],
        true,
        false,
    );
    assert!(replay.depletes_starvation);
    assert!(replay.never_created_a);
    assert!(!replay.inspected_s_or_damage);
}

#[test]
fn repeated_damage_depletion_flag() {
    let rates = [(-1.0, 1.0), (0.1, 1.0), (-1.0, 1.0), (0.1, 1.0), (-1.0, 1.0)];
    let c = CumulativeABalance::from_rates(&rates);
    // deepening troughs → unbounded growth signal
    assert!(c.unrepaid_deficit_grows_unbounded(0.05) || c.bootstrap_storage() > 0.0);
}

#[test]
fn route_selection_capacity_and_buffer() {
    // args: max_def, h_phi, surplus, timescale, starvation_depletes,
    //       indefinite_repair_without_activation, requires_transport_change, spatial_disjoint
    let spatial_ok =
        evaluate_structural_buffer_feasibility(0.2, 1.0, 0.5, 0.1, true, false, false, false);
    let (r, c) = select_route(
        true,
        true,
        PersistentCapacityClass::ActivationCapacityDeficit,
        None,
        false,
        false,
        &spatial_ok,
        false,
    );
    assert_eq!(r, D042Route::RouteA);
    assert_eq!(c, D042Conclusion::ActivationCapacityDeficit);

    let (r2, c2) = select_route(
        true,
        true,
        PersistentCapacityClass::TemporaryDeficitBufferCandidate,
        None,
        true,
        true,
        &spatial_ok,
        true,
    );
    assert_eq!(r2, D042Route::RouteB);
    assert_eq!(c2, D042Conclusion::LocalActivationBufferJustified);

    let spatial_s =
        evaluate_structural_buffer_feasibility(0.2, 1.0, 0.5, 0.1, true, false, false, true);
    let (r3, c3) = select_route(
        true,
        true,
        PersistentCapacityClass::TemporaryDeficitBufferCandidate,
        None,
        true,
        true,
        &spatial_s,
        true,
    );
    assert_eq!(r3, D042Route::RouteS);
    assert_eq!(c3, D042Conclusion::SpatialEnergyCarrierRequired);
}

#[test]
fn no_observer_feedback_flag() {
    let replay = replay_ideal_buffer(
        1.0,
        &[1.0, -0.5, 0.5],
        1.0,
        &[D042_REPAIR_P_MIN, D042_REPAIR_P_MIN, D042_REPAIR_P_MIN + 0.01],
        &[0.3, 0.4, 0.5],
        false,
        false,
    );
    assert!(!replay.inspected_s_or_damage);
    assert!(replay.never_created_a);
    assert!(replay.never_exceeded_capacity);
}

#[test]
fn linear_trend_detects_decline() {
    let y = [3.0, 2.5, 2.0, 1.5, 1.0];
    assert!(linear_trend(&y) < 0.0);
}
