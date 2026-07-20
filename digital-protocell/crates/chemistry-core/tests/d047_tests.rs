//! D-047 focused tests: classification, role, tracer, competition, route.

use chemistry_core::d046_analysis::DemandStateRow;
use chemistry_core::d047_analysis::{
    a_equivalent_role_catalog, candidate_zero_resource_ok, classify_biochemistry_state,
    classify_service_competition, classify_shared_pool_upper_bound, classify_sink_regulation,
    cross_parameter_model_audit, fixed_holdout_label, fixed_train_label, is_altered_parameter_state,
    precursor_product_response, product_inhibition_monotonic, select_route,
    shared_pool_structural_checks, ACohortBalance, BiochemistryClass, CompetitionClass,
    RouteDecisionInput, SharedPoolUpperBound, SinkRegulationClass, D047Route,
};

fn row(label: &str, family: &str, train: bool, k_p: f64, k_s: f64, l_a: f64) -> DemandStateRow {
    DemandStateRow {
        label: label.into(),
        family: family.into(),
        train,
        radius: 22.0,
        c: 0.8,
        n: 0.8,
        f: 0.8,
        a: 0.5,
        p: 0.05,
        s_occupancy: 0.6,
        m_c: 1200.0,
        interior_volume: 1500.0,
        structural_mass: 1500.0,
        membrane_area: 140.0,
        l_a,
        j_reproduction: l_a * 0.11,
        j_structural: l_a * 0.10,
        j_precursor: l_a * 0.76,
        j_decay: l_a * 0.02,
        j_out: l_a * 0.01,
        j_in: 0.0,
        k_structure_scale: k_s,
        k_precursor_scale: k_p,
    }
}

#[test]
fn fixed_biochemistry_state_classification() {
    let r = row("R22", "radius", true, 1.0, 1.0, 180.0);
    assert_eq!(
        classify_biochemistry_state(&r),
        BiochemistryClass::FixedBiochemistry
    );
}

#[test]
fn altered_parameter_state_classification() {
    let r = row("prec_hi", "precursor", false, 2.0, 1.0, 310.0);
    assert!(is_altered_parameter_state(&r));
    let s = row("struct_hi", "structural", false, 1.0, 2.0, 190.0);
    assert!(is_altered_parameter_state(&s));
}

#[test]
fn cross_parameter_error_separation() {
    let rows = vec![
        row("R16", "radius", true, 1.0, 1.0, 96.0),
        row("R22", "radius", true, 1.0, 1.0, 180.0),
        row("R32", "radius", false, 1.0, 1.0, 384.0),
        row("low_c", "catalyst", true, 1.0, 1.0, 144.0),
        row("med_c", "catalyst", true, 1.0, 1.0, 168.0),
        row("high_c", "catalyst", false, 1.0, 1.0, 186.0),
        row("s_healthy", "membrane", true, 1.0, 1.0, 180.0),
        row("s_damaged25", "membrane", false, 1.0, 1.0, 182.0),
        row("prec_lo", "precursor", true, 0.5, 1.0, 100.0),
        row("prec_hi", "precursor", false, 2.0, 1.0, 260.0),
    ];
    let audit = cross_parameter_model_audit(&rows);
    assert_eq!(audit.n_altered, 2);
    assert!(audit.n_fixed >= 8);
    assert!(!audit.complete_a.adequate || audit.complete_a.max_hold_err > 0.2);
}

#[test]
fn a_equivalent_role_tracing() {
    let cat = a_equivalent_role_catalog();
    assert!(cat.iter().any(|r| r.route_id == "A_to_C"));
    assert!(cat.iter().any(|r| r.route_id == "A_to_phi"));
    assert!(cat.iter().any(|r| r.route_id == "A_to_P"));
    let (ok, _) = shared_pool_structural_checks(&cat);
    assert!(ok);
}

#[test]
fn a_cohort_tracer_conservation() {
    let b = ACohortBalance::from_flows(100.0, 11.0, 10.0, 76.0, 2.0, 1.0, 0.0);
    assert!(b.conservation_ok(1e-12));
}

#[test]
fn sink_destination_accounting() {
    let b = ACohortBalance::from_flows(100.0, 11.0, 10.0, 76.0, 2.0, 1.0, 0.0);
    let f = b.destination_fractions();
    assert!((f[2].1 - 0.76).abs() < 1e-12);
}

#[test]
fn service_failure_ordering() {
    let m = [1.0, 0.6, 0.2];
    let j_rep = [10.0, 8.0, 4.0];
    let j_struct = [10.0, 9.0, 7.0];
    let j_prec = [100.0, 40.0, 5.0];
    assert_eq!(
        classify_service_competition(&m, &j_rep, &j_struct, &j_prec),
        CompetitionClass::PrecursorDominatedStarvation
    );
}

#[test]
fn precursor_product_response_assay() {
    let samples = [(0.01, 1.5), (0.5, 1.5), (1.0, 1.5)];
    let (slope, flag, tag) = precursor_product_response(&samples);
    assert!(slope.abs() < 1e-9);
    assert!(flag);
    assert_eq!(tag, "PRECURSOR_SYNTHESIS_NOT_PRODUCT_REGULATED");
    assert_eq!(
        classify_sink_regulation(false, slope, false, false),
        SinkRegulationClass::ConstitutiveWhileARemains
    );
}

#[test]
fn ideal_shared_pool_controls() {
    assert_eq!(
        classify_shared_pool_upper_bound(true, true, false, false),
        SharedPoolUpperBound::SharedAPoolCapable
    );
}

#[test]
fn local_versus_global_supply() {
    assert_eq!(
        classify_shared_pool_upper_bound(true, false, true, false),
        SharedPoolUpperBound::SpatialAAllocationDefect
    );
    assert_eq!(
        classify_shared_pool_upper_bound(true, false, false, true),
        SharedPoolUpperBound::SharedAPoolStructurallyInsufficient
    );
}

#[test]
fn candidate_zero_cnf_and_inhibition() {
    assert!(candidate_zero_resource_ok());
    assert!(product_inhibition_monotonic(0.2, 3.0, 1.0, 1.0));
}

#[test]
fn fixed_training_holdout_separation() {
    assert!(fixed_train_label("R16"));
    assert!(fixed_holdout_label("damage25"));
    assert!(!fixed_train_label("prec_hi"));
}

#[test]
fn route_selection_rules() {
    assert_eq!(
        select_route(&RouteDecisionInput {
            accounting_failure: false,
            tracer_failure: false,
            a_role_inconsistent: false,
            shared_pool_structurally_insufficient: false,
            spatial_allocation_defect: false,
            precursor_not_product_regulated: true,
            precursor_destroys_fixed_point: true,
            reducing_precursor_restores_stability: true,
            historical_fixed_biology_adequate: false,
            candidate_b_qualified: true,
            candidate_c_or_d_qualified: true,
            shared_pool_capable: true,
        }),
        D047Route::RouteP
    );
}

#[test]
fn no_observer_feedback() {
    // Occupancy is not an input to candidate_zero checks.
    assert!(candidate_zero_resource_ok());
}
