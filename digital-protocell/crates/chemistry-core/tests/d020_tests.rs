//! D-020 v3 joint-rate Stage E recovery tests.

use chemistry_core::*;

fn analytical() -> StageEReferenceRates {
    D020_ANALYTICAL_V3_RATES
}

#[test]
fn test_four_rate_only_mutability() {
    assert_eq!(RATE_PARAM_NAMES.len(), 4);
    for name in RATE_PARAM_NAMES {
        assert!(is_productive_rate_name(name));
        assert!(!is_frozen_rate_name(name));
    }
    for name in D020_FROZEN_RATE_NAMES {
        assert!(is_frozen_rate_name(name));
        assert!(!is_productive_rate_name(name));
    }
    let a = analytical();
    let mut b = a;
    b.k_d008_structure *= 1.2;
    b.k_d008_reproduction *= 1.1;
    b.k_membrane *= 0.9;
    b.k_d008_activation *= 0.8;
    assert!(only_productive_rates_differ(&a, &b));
    b.k_structure_decay *= 2.0;
    assert!(!only_productive_rates_differ(&a, &b));
    let frozen = freeze_nonproductive_rates(&b, &a);
    assert!(only_productive_rates_differ(&a, &frozen));
    assert_eq!(frozen.k_structure_decay, a.k_structure_decay);
}

#[test]
fn test_candidate_bounds() {
    let a = analytical();
    let mut high = a;
    high.k_d008_structure = a.k_d008_structure * 10.0;
    let clamped = clamp_rates_to_global_bounds(&high, &a);
    assert!((clamped.k_d008_structure - a.k_d008_structure * D020_GLOBAL_RATE_MAX_FACTOR).abs() < 1e-12);
    assert!(rates_within_global_bounds(&clamped, &a));

    let mut low = a;
    low.k_membrane = a.k_membrane * 0.01;
    let clamped_low = clamp_rates_to_global_bounds(&low, &a);
    assert!((clamped_low.k_membrane - a.k_membrane * D020_GLOBAL_RATE_MIN_FACTOR).abs() < 1e-12);

    let mut next = a;
    next.k_d008_reproduction = a.k_d008_reproduction * 1.4;
    assert!(rates_within_round_factor(&a, &next));
    next.k_d008_reproduction = a.k_d008_reproduction * 2.0;
    assert!(!rates_within_round_factor(&a, &next));
}

#[test]
fn test_candidate_count_limit() {
    let a = analytical();
    let sens = sensitivity_matrix(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let g_history = [[1.0, 1.0, 1.0, 1.0]; 8];
    let sens_history = vec![sens; 8];
    let report = bounded_joint_solver_d020(&a, &a, &g_history, &sens_history);
    assert!(report.candidates.len() <= D020_MAX_CANDIDATES);
    assert!(report.bounded);
    assert!(report.rounds_attempted <= D020_MAX_SOLVER_ROUNDS);
}

#[test]
fn test_sensitivity_matrix_rank() {
    let full = sensitivity_matrix(&[
        [2.0, 0.1, 0.0, 0.0],
        [0.1, 1.5, 0.2, 0.0],
        [0.0, 0.2, 1.2, 0.1],
        [0.0, 0.0, 0.1, 0.9],
    ]);
    assert_eq!(full.rank, 4);
    assert!(!full.rank_deficient);
    assert!(full.condition_number.is_finite());

    let deficient = sensitivity_matrix(&[
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 4.0, 6.0, 8.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    assert!(deficient.rank < 4);
    assert!(deficient.rank_deficient);

    let d = log_central_difference_with_perturb(2.0, 0.0, D020_SENSITIVITY_PERTURB);
    assert!(d.is_finite() && d > 0.0);
}

#[test]
fn test_joint_flow_scoring() {
    let m = placeholder_metrics([-3.0, -4.0, 0.0, 0.0], [0.5, 0.5, 1.0, 1.0]);
    assert!((joint_flow_score(&m) - 5.0).abs() < 1e-12);
    let better = placeholder_metrics([-1.0, -1.0, 0.0, 0.0], [0.8, 0.8, 1.0, 1.0]);
    assert!(joint_flow_score(&better) < joint_flow_score(&m));
}

#[test]
fn test_promotion_gates() {
    let baseline = placeholder_metrics(
        [-5.0, -0.5, -0.6, -0.4],
        [0.12, 0.46, 0.51, 1.49],
    );
    let improved = placeholder_metrics(
        [-2.0, -0.2, -0.3, -0.2],
        [0.5, 0.7, 0.8, 1.2],
    );
    let hard = evaluate_hard_gates(&improved, 0.01, false, false, true);
    assert!(hard.all_pass());
    assert!(promotion_gate(&baseline, &improved, hard));

    let destabilized = placeholder_metrics(
        [-1.0, -2.0, -0.3, -0.2],
        [0.5, 0.2, 0.8, 1.2],
    );
    let hard2 = evaluate_hard_gates(&destabilized, 0.01, false, false, true);
    assert!(!promotion_gate(&baseline, &destabilized, hard2));

    let contam = evaluate_hard_gates(&improved, 0.10, false, false, true);
    assert!(!contam.all_pass());
}

#[test]
fn test_r18_r22_r26_restoring_signs() {
    let ok = restoring_radius_signs(0.4, 0.001, -0.35);
    assert!(ok.r18_positive && ok.r26_negative);
    assert!(restoring_sign_pattern_pass(0.4, 0.01, -0.35));
    assert!(!restoring_sign_pattern_pass(-0.1, 0.0, -0.2));
    assert!(!restoring_sign_pattern_pass(0.4, 0.5, -0.35));
}

#[test]
fn test_historical_v2_v3_equivalence() {
    assert!(productive_rate_layout_matches_historical());
    // Same productive vector layout and freeze semantics as D-011/D-012.
    let a = analytical();
    let q_metrics = placeholder_metrics(
        [-5.677, -0.530, -0.599, -0.416],
        [0.11523801608563161, 0.45865699908292534, 0.5130047242658399, 1.4944310610696545],
    );
    let corrected = q_corrected_rates(&a, &q_metrics);
    assert!(rates_within_global_bounds(&corrected, &a));
    assert!(only_productive_rates_differ(&a, &corrected));
    // Structure hits the 4× global ceiling under the Stage A Q deficit.
    assert!(
        (corrected.k_d008_structure - a.k_d008_structure * D020_GLOBAL_RATE_MAX_FACTOR).abs()
            < 1e-9
    );
    // Activation scales down toward Q=1.
    assert!(corrected.k_d008_activation < a.k_d008_activation);
    assert!(corrected.k_d008_reproduction > a.k_d008_reproduction);
    assert!(corrected.k_membrane > a.k_membrane);

    let conclusion = select_d020_conclusion(false, false, true, true, false);
    assert_eq!(
        conclusion.as_str(),
        "D020_V3_STAGE_E_REFERENCE_RECOVERED"
    );
}
