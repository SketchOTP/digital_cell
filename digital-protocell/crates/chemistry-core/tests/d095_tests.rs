use chemistry_core::d095_analysis::{
    classify_d094_failure, evaluate_candidates, normalize_d094_attempt,
    observational_decomposition, NormalizedRow,
};
use std::path::PathBuf;

#[test]
fn weak_paired_effects_select_finite_budget_allocation() {
    let rows = vec![
        NormalizedRow::d094("H", 0, 0.444_444, 0.480_198, 202, 194),
        NormalizedRow::d094("H", 1, 0.5, 0.48, 200, 192),
        NormalizedRow::d094("B", 0, 0.5, 0.459_459, 296, 288),
        NormalizedRow::d094("B", 3, 0.533_333, 0.538_012, 342, 334),
        NormalizedRow::d094("N", 0, 0.0, 0.510_563, 284, 276),
        NormalizedRow::d094("N", 1, 0.0, 0.505_780, 346, 338),
    ];
    assert_eq!(
        classify_d094_failure(&rows),
        "PHENOTYPE_NOT_COUPLED_TO_CONSERVED_PHYSIOLOGY"
    );
    let candidates = evaluate_candidates();
    assert_eq!(
        candidates["selected_architecture"],
        "B_FINITE_BUDGET_CATALYTIC_ALLOCATION"
    );
    assert_eq!(candidates["automatic_implementation"], false);
}

#[test]
fn observational_decomposition_measures_opportunity_covariance_and_loo() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experiments/generated/d094r/gate6/attempt_001");
    let table = normalize_d094_attempt(&root).expect("sealed D-094 attempt");
    let result = observational_decomposition(&table.included);
    assert_eq!(result["rows"], 24);
    assert!(result["H"]["opportunity_for_selection"].as_f64().unwrap() > 0.0);
    assert!(
        result["B"]["trait_descendant_covariance"]
            .as_f64()
            .unwrap()
            .abs()
            < 0.01
    );
    assert_eq!(result["N"]["phenotype_variance"], 0.0);
    assert_eq!(result["N"]["selection_gradient"], 0.0);
    assert_eq!(result["leave_one_out"]["selection_stable"], true);
}

#[test]
fn sealed_attempt_normalizes_paired_rows_with_complete_checkpoints() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experiments/generated/d094r/gate6/attempt_001");
    let table = normalize_d094_attempt(&root).expect("sealed D-094 attempt");
    assert_eq!(table.included.len(), 24);
    assert_eq!(table.excluded.len(), 0);
    assert!(table
        .included
        .iter()
        .all(|row| row.completed_generation == 8 && row.checkpoint_complete));
    for rep in 0..8 {
        assert_eq!(
            table
                .included
                .iter()
                .filter(|row| row.replicate == rep)
                .count(),
            3
        );
    }
}
