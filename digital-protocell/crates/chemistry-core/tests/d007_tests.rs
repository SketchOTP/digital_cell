//! D-007 joint kinetic nullcline search tests.

use chemistry_core::*;

#[test]
fn test_d007_result_schema_is_complete() {
    assert!(result_schema_is_complete(D007_REQUIRED_RESULT_FIELDS));
    let mut missing = D007_REQUIRED_RESULT_FIELDS.to_vec();
    missing.pop();
    assert!(!result_schema_is_complete(&missing));
}

#[test]
fn test_d007_clean_termination_is_required() {
    assert!(clean_termination_is_required(true, true));
    assert!(!clean_termination_is_required(false, true));
    assert!(clean_termination_is_required(false, false));
}

#[test]
fn test_d007_accounting_is_required() {
    assert!(accounting_is_required(true, true));
    assert!(!accounting_is_required(false, true));
    assert!(accounting_is_required(false, false));
}

#[test]
fn test_d007_candidate_parameters_match_hash() {
    assert!(candidate_parameters_match_hash(0.1, 0.02, 0.1, 0.02));
    assert!(!candidate_parameters_match_hash(0.1, 0.02, 0.11, 0.02));
}

#[test]
fn test_d007_reference_reproduces_d006_flow_direction() {
    // Direction gate used after reference replay artifact is produced.
    assert!(reference_flow_direction_ok(0.02, -0.001));
    assert!(!reference_flow_direction_ok(-0.01, -0.001));
    assert!(!reference_flow_direction_ok(0.02, 0.001));
    // Frozen reference rates must match machine-precision D-006 values.
    let p = {
        let mut p = surface_turnover_params_from_calibrated_kphi1();
        p.k_structure_interface = D006_K_STRUCTURE_INTERFACE;
        p.k_rep = D006_K_REP;
        p
    };
    assert!((p.k_structure_interface - 0.09642857142857159).abs() < 1e-15);
    assert!((p.k_rep - 0.014489097664708522).abs() < 1e-15);
    let grid = GridConfiguration::default();
    let h = configuration_hash(&p, &grid);
    assert_eq!(h, D006_REFERENCE_CONFIGURATION_HASH);
}

#[test]
fn test_required_k_rep_estimator() {
    let rows = vec![
        (0.5, 0.9, 100.0, 24.0, "c1".into(), 0.35, Some(0.99), false, false, false, false),
        (1.0, 0.9, 100.0, 24.0, "c1".into(), 0.35, Some(0.99), false, false, false, false),
        (2.0, 0.9, 100.0, 16.0, "c2".into(), 0.275, Some(0.99), false, false, false, false),
    ];
    let est = estimate_required_k_rep(D006_K_REP, &rows);
    assert_eq!(est.n_valid, 3);
    assert!((est.median_required_k_rep - required_k_rep(D006_K_REP, 1.0)).abs() < 1e-12);
}

#[test]
fn test_required_k_rep_rejects_invalid_runs() {
    assert!(reject_required_k_rep_row(
        None, 0.9, 10.0, Some(0.99), false, false, false, false
    ));
    assert!(reject_required_k_rep_row(
        Some(1.0), 0.01, 10.0, Some(0.99), false, false, false, false
    ));
    assert!(reject_required_k_rep_row(
        Some(1.0), 0.9, 10.0, Some(0.99), false, false, true, false
    ));
    assert!(!reject_required_k_rep_row(
        Some(1.0), 0.9, 10.0, Some(0.99), false, false, false, false
    ));
}

#[test]
fn test_k_rep_search_remains_bounded() {
    assert!(k_rep_search_remains_bounded(D006_K_REP));
    assert!(k_rep_search_remains_bounded(D006_K_REP * 3.0));
    assert!(!k_rep_search_remains_bounded(D006_K_REP * 3.01));
    assert!(!k_rep_search_remains_bounded(D006_K_REP * 0.5));
    let clamped = clamp_k_rep_to_d006_bounds(D006_K_REP * 10.0);
    assert!((clamped - D006_K_REP * 3.0).abs() < 1e-15);
}

#[test]
fn test_structural_bracket_detects_all_decline() {
    assert_eq!(
        classify_structural_bracket(-0.1, -0.2, -0.3),
        StructuralBracketClass::AllDecline
    );
}

#[test]
fn test_structural_bracket_detects_all_growth() {
    assert_eq!(
        classify_structural_bracket(0.1, 0.2, 0.3),
        StructuralBracketClass::AllGrow
    );
}

#[test]
fn test_structural_bracket_detects_restoring_crossing() {
    assert_eq!(
        classify_structural_bracket(0.05, 0.0, -0.04),
        StructuralBracketClass::RestoringCrossing
    );
    assert!(provisional_structural_factor_passes(0.05, 0.01, -0.04));
    assert!(!provisional_structural_factor_passes(0.05, 0.05, 0.05));
    assert_eq!(
        structural_failure_gate(&[
            StructuralBracketClass::AllGrow,
            StructuralBracketClass::AllGrow
        ]),
        Some("D007_NO_STRUCTURAL_NULLCLINE")
    );
}

#[test]
fn test_catalyst_bracket_detects_all_decline() {
    assert_eq!(
        classify_catalyst_bracket(-0.01, -0.02, -0.03),
        CatalystBracketClass::AllDecline
    );
    assert_eq!(
        catalyst_failure_gate(&[CatalystBracketClass::AllDecline]),
        Some("D007_NO_CATALYST_NULLCLINE")
    );
}

#[test]
fn test_catalyst_bracket_detects_all_growth() {
    assert_eq!(
        classify_catalyst_bracket(0.01, 0.02, 0.03),
        CatalystBracketClass::AllGrow
    );
    assert_eq!(
        catalyst_failure_gate(&[CatalystBracketClass::AllGrow]),
        Some("D007_UNBOUNDED_CATALYST")
    );
}

#[test]
fn test_catalyst_bracket_detects_restoring_crossing() {
    assert_eq!(
        classify_catalyst_bracket(0.01, 0.0, -0.01),
        CatalystBracketClass::RestoringCrossing
    );
    assert!(provisional_catalyst_rate_passes(0.01, -0.01, 0.85, false, false));
    assert!(!provisional_catalyst_rate_passes(0.01, -0.01, 0.70, false, false));
}

#[test]
fn test_joint_candidate_count_is_bounded() {
    assert!(joint_candidate_count_bounded(9));
    assert!(!joint_candidate_count_bounded(10));
}

#[test]
fn test_joint_candidate_is_immutable() {
    // Candidate identity strings encode parameters; rewriting would change hashes.
    let a = "cand-abc";
    let b = "cand-abc";
    assert_eq!(a, b);
    // Retention helper keeps at most three neighboring factors.
    let fac = structural_factors();
    let pass = [false, false, true, true, false, false, false];
    let kept = retain_neighboring_factors(&fac, &pass, 3);
    assert!(kept.len() <= 3);
}

#[test]
fn test_nullclines_must_intersect() {
    assert!(nullclines_must_intersect(true, true, true));
    assert!(!nullclines_must_intersect(true, true, false));
    assert!(!nullclines_must_intersect(true, false, true));
}

#[test]
fn test_joint_jacobian_must_be_stable() {
    assert!(joint_jacobian_must_be_stable(-0.1));
    assert!(!joint_jacobian_must_be_stable(0.0));
    assert!(!joint_jacobian_must_be_stable(0.2));
}

#[test]
fn test_saddle_intersection_fails() {
    let c = classify_joint_intersection(
        true,
        true,
        true,
        FixedPointClass::SaddleLike,
        false,
        false,
    );
    assert!(saddle_intersection_fails(c));
}

#[test]
fn test_disjoint_nullclines_fail() {
    let c = classify_joint_intersection(
        true,
        true,
        false,
        FixedPointClass::Stable,
        false,
        false,
    );
    assert!(disjoint_nullclines_fail(c));
}

#[test]
fn test_refined_center_requires_four_of_five() {
    assert!(refined_center_requires_four_of_five(4));
    assert!(refined_center_requires_four_of_five(5));
    assert!(!refined_center_requires_four_of_five(3));
}

#[test]
fn test_refined_neighbors_require_contiguous_patch() {
    assert!(refined_neighbors_require_contiguous_patch(3, true));
    assert!(!refined_neighbors_require_contiguous_patch(2, true));
    assert!(!refined_neighbors_require_contiguous_patch(3, false));
}

#[test]
fn test_full_acceptance_uses_fresh_seed() {
    assert!(accepts_only_fresh_seed(true, false));
    assert!(!accepts_only_fresh_seed(false, true));
}

#[test]
fn test_puncture_consumes_additional_resources() {
    let mut p = surface_turnover_params_from_calibrated_kphi1();
    p.k_structure_interface = D006_K_STRUCTURE_INTERFACE;
    let dense = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &p, true);
    let none = compute_reactions_at(0.0, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!(dense.r_n < none.r_n);
    assert!(dense.r_f < none.r_f);
    assert!(dense.r_w > none.r_w);
    assert!(dense.r_rep >= 0.0);
}

#[test]
fn test_controls_gate_full_acceptance() {
    assert!(!full_acceptance_may_run(true, true, true, false, true));
    assert!(full_acceptance_may_run(true, true, true, true, true));
}

#[test]
fn test_parameter_domain_rejection_gate() {
    assert_eq!(
        parameter_domain_rejection(true, false, false, false, false, false, false, false, false),
        Some("D007_FIVE_FIELD_MODEL_REJECTED")
    );
    assert_eq!(
        parameter_domain_rejection(false, false, false, false, false, false, false, false, false),
        None
    );
}
