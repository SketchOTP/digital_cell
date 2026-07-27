use chemistry_core::d096_allocation::AssayEnvironment;
use chemistry_core::d097_analysis::{
    b_specificity, classify_first_break, decompose_eight_pairs, reconstruct_pair,
    D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED,
};

#[test]
fn reconstructs_the_real_d096_path_and_finds_reserve_compatibility_break() {
    let pair = reconstruct_pair(1, AssayEnvironment::H, 1_000);

    assert!(pair.difference.processing_allocation > 0.0);
    assert!(pair.difference.processing_expression > 0.0);
    assert!(pair.difference.resource_encounter.abs() < 1e-12);
    assert!(pair.difference.activated_production > 0.0);
    assert_eq!(pair.difference.reserve_inflow, 0.0);
    assert_eq!(pair.difference.reserve_change, 0.0);
    assert_eq!(pair.difference.growth, 0.0);
    assert!(!pair.processing.reserve_schema_compatible);
    assert_eq!(
        classify_first_break(&pair),
        D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED
    );
}

#[test]
fn decomposition_separates_upstream_authority_from_the_downstream_schema_break() {
    let result = decompose_eight_pairs(1_000);

    assert!(result.processing_share_mean_h > 0.0);
    assert!(result.legacy_share_mean_h > 0.0);
    assert!(result.pulse_expression_overlap_fraction > 0.99);
    assert!(!result.resource_delivery_limited);
    assert!(
        result
            .h_pairs
            .iter()
            .all(|pair| pair.difference.activated_production > 0.0)
    );
    assert_eq!(result.h_minus_neutral.reserve_inflow, 0.0);
    assert_eq!(result.primary_classification, D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED);
    assert!(!result.mutation_run);
    assert!(!result.heredity_run);
    assert!(!result.selection_run);
    assert!(!result.adaptation_run);
    assert!(!result.reversal_run);
}

#[test]
fn sealed_b_minus_neutral_specificity_is_paired_and_stable() {
    let b = [
        2.066987456330068,
        2.06508890603628,
        2.068518237759065,
        2.066987456330068,
        2.06508890603628,
        2.068518237759065,
        2.066987456330068,
        2.06508890603628,
    ];
    let neutral = [
        1.876274989562404,
        1.876040753632438,
        1.8798691170678126,
        1.876274989562404,
        1.876040753632438,
        1.8798691170678126,
        1.876274989562404,
        1.876040753632438,
    ];
    let result = b_specificity(&b, &neutral);

    assert_eq!(result.positive_pairs, 8);
    assert!(result.ci95[0] > 0.0);
    assert!(result.leave_one_out_all_positive);
    assert_eq!(result.classification, "B_REPAIR_SPECIFICITY_PRESENT");
}
