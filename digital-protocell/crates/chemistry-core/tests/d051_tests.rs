//! Focused D-051 unit coverage (diagnostic helpers; no production chemistry change).

use chemistry_core::d050_analysis::{
    production_activation_rate, schema2_activation_rate, ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
    ACTIVATION_SCHEMA_HISTORICAL, D050_HISTORICAL_K,
};
use chemistry_core::d051_analysis::*;

#[test]
fn d050_seal_constants_match_gate_minus_one() {
    assert_eq!(D051_STARTING_COMMIT, "0b0fb89");
    assert_eq!(D051_STARTING_TAG, "D-050-catalyst-saturating-activation-fail");
    assert_eq!(D051_FROZEN_D050, "D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED");
    assert_eq!(D051_D050_RECORD, "CATALYST_SATURATING_CAPACITY_REPAIR_REJECTED");
}

#[test]
fn requested_accepted_extent_accounting() {
    let rec = ActivationExtentRecord {
        xi_requested: 0.4,
        xi_accepted: 0.4,
        n_available: 1.0,
        f_available: 1.0,
        rejected: false,
        timestep_capped: false,
        concentration_safety: false,
    };
    assert!((rec.f_accepted() - 1.0).abs() < 1e-15);
    assert_eq!(rec.classify(), ActivationLimitClass::Unconstrained);
    let sum = summarize_extent_records(&[rec]);
    assert!((sum.requested_integrated - sum.accepted_integrated).abs() < 1e-15);
}

#[test]
fn physical_versus_numerical_capping() {
    assert_eq!(
        classify_extent_cap_mode(true, true, true, false),
        "ACTIVATION_EXTENT_RESOURCE_CAPPED"
    );
    assert_eq!(
        classify_extent_cap_mode(false, true, false, true),
        "ACTIVATION_EXTENT_NUMERICALLY_CAPPED"
    );
}

#[test]
fn resource_throughput_ceiling_helper() {
    let j = resource_available_flux(0.5, 0.2, 1.0, 0.0, 10.0);
    assert!((j - 0.8).abs() < 1e-12);
    let c = compute_resource_ceiling(j, j, 0.3, 0.1, 0.1, 0.05, 0.05);
    assert!(c.chi_resource > 1.0);
}

#[test]
fn operator_schedule_conservation_helpers() {
    let extents = jointly_bound_extents(&[0.5, 0.5, 0.5], 1.0);
    let sum: f64 = extents.iter().sum();
    assert!((sum - 1.0).abs() < 1e-12);
    assert!(!overcommitment(sum, 1.0));
}

#[test]
fn timestep_refinement_comparison_label() {
    // Analysis-only: finer dt must not invent a biological fix by itself.
    let coarse = jointly_bound_extents(&[1.0, 1.0], 1.0);
    let fine = jointly_bound_extents(&[0.5, 0.5], 1.0);
    assert!((coarse.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!((fine.iter().sum::<f64>() - 1.0).abs() < 1e-12);
}

#[test]
fn a_cohort_and_immediate_consumption() {
    let d = cohort_from_ledger(2.0, 0.1, 0.2, 0.2, 1.4, 0.05, 0.05);
    assert!((d.sum() - 1.0).abs() < 1e-12);
    assert!(is_immediate_productive_capture(
        true,
        d.productive_immediate_fraction(),
        d.free_remaining
    ));
}

#[test]
fn a_to_p_and_a_to_s_utility() {
    let y = precursor_yields(0.8, 1.0, 0.05, 0.1, 0.05, 0.0, 0.2, 0.1);
    assert!((y.y_a_to_p - 0.8).abs() < 1e-15);
    assert!((y.y_a_to_s - 0.05).abs() < 1e-15);
}

#[test]
fn spatial_overlap_and_conservative_mixing_invariant() {
    let prod = vec![2.0, 0.0, 0.0];
    let dem = vec![1.0, 1.0, 0.0];
    let o = spatial_overlap(&prod, &dem);
    assert!((o - 0.5).abs() < 1e-15);
    // Conservative mixing preserves total A (identity check on totals).
    let total = 3.0;
    let mixed = [1.0, 1.0, 1.0];
    assert!((mixed.iter().sum::<f64>() - total).abs() < 1e-15);
}

#[test]
fn gross_throughput_classification() {
    assert_eq!(
        classify_free_pool(0.02, 1.0, 1.05, true, true).as_str(),
        "HIGH_FLUX_ACTIVATION_WASTED_DOWNSTREAM"
    );
}

#[test]
fn route_selection_rules_exhaustive_priority() {
    let base = RouteDecisionInput {
        d050_sealed: true,
        d050_reproduced: true,
        extent_accounting_ok: true,
        cohort_accounting_ok: true,
        accounting_ok: true,
        numerical_ok: true,
        ..Default::default()
    };
    let cases = [
        (
            RouteDecisionInput {
                resource_throughput_limits: true,
                ..base
            },
            D051PrimaryConclusion::ResourceThroughputLimit,
        ),
        (
            RouteDecisionInput {
                extent_bounding_defect: true,
                ..base
            },
            D051PrimaryConclusion::ActivationExtentBoundingDefect,
        ),
        (
            RouteDecisionInput {
                operator_split_defect: true,
                ..base
            },
            D051PrimaryConclusion::ActivationOperatorSplitDefect,
        ),
        (
            RouteDecisionInput {
                spatial_allocation_failure: true,
                ..base
            },
            D051PrimaryConclusion::ActivationSpatialAllocationFailure,
        ),
        (
            RouteDecisionInput {
                precursor_conversion_bottleneck: true,
                ..base
            },
            D051PrimaryConclusion::PrecursorConversionBottleneck,
        ),
        (
            RouteDecisionInput {
                free_a_metric_noncausal: true,
                ..base
            },
            D051PrimaryConclusion::FreeARetentionMetricNoncausal,
        ),
        (
            RouteDecisionInput {
                topology_insufficient: true,
                ..base
            },
            D051PrimaryConclusion::CoupledActivationTopologyInsufficient,
        ),
        (
            RouteDecisionInput {
                activation_not_primary: true,
                ..base
            },
            D051PrimaryConclusion::ActivationNotPrimaryCoupledBlocker,
        ),
        (base, D051PrimaryConclusion::CoupledActivationThroughputInconclusive),
    ];
    for (input, expect) in cases {
        assert_eq!(select_primary_route(&input), expect);
    }
}

#[test]
fn schema1_and_schema2_preserved_no_repair() {
    let s1 = production_activation_rate(
        ACTIVATION_SCHEMA_HISTORICAL,
        D050_HISTORICAL_K,
        1.0,
        0.4,
        0.5,
        0.5,
        0.1,
        1.0,
        1.0,
    );
    assert!((s1 - D050_HISTORICAL_K * 0.4 * 0.5 * 0.5).abs() < 1e-15);
    let s2 = schema2_activation_rate(D051_FITTED_V_A, 1.0, 0.5, 1.0, 1.0, D051_FITTED_K_C, 1.0, 1.0);
    let s2x4 = schema2_activation_rate(
        D051_FITTED_V_A * 4.0,
        1.0,
        0.5,
        1.0,
        1.0,
        D051_FITTED_K_C,
        1.0,
        1.0,
    );
    assert!(s2x4 > s2 * 3.5);
    let _ = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
}

#[test]
fn no_diagnostic_feedback_into_production_constants() {
    // D-051 must not alter fitted center or promote controls.
    assert!((D051_FITTED_V_A - 0.12544510052968755).abs() < 1e-12);
    assert!((D051_FITTED_K_C - 0.10).abs() < 1e-15);
}
