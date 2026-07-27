use chemistry_core::d095_analysis::{
    candidate_review, environmental_contrast, final_causal_classification,
    freeze_d096_contract, normalize_d094_attempt, observational_decomposition,
    reciprocal_interaction, select_route, Allocation, CausalReplaySummary, PartitionSummary,
};
use std::path::PathBuf;

#[test]
fn corrected_classification_identifies_specificity_before_demography() {
    assert_eq!(
        final_causal_classification(
            &PartitionSummary {
                destroys_phenotype: true,
                ..Default::default()
            },
            &CausalReplaySummary::default(),
        ),
        ("PARTITION_NOISE_ERASES_SELECTION", None)
    );
    assert_eq!(
        final_causal_classification(
            &PartitionSummary::default(),
            &CausalReplaySummary::default(),
        ),
        ("PHENOTYPE_NOT_COUPLED_TO_CONSERVED_PHYSIOLOGY", None)
    );
    assert_eq!(
        final_causal_classification(
            &PartitionSummary::default(),
            &CausalReplaySummary {
                physiology_differs: true,
                ..Default::default()
            },
        ),
        ("PHYSIOLOGICAL_EFFECT_BUFFERED_BEFORE_FITNESS", None)
    );
    assert_eq!(
        final_causal_classification(
            &PartitionSummary::default(),
            &CausalReplaySummary {
                physiology_differs: true,
                growth_or_survival_differs: true,
                environment_interaction_present: false,
            },
        ),
        (
            "ENVIRONMENT_PHENOTYPE_INTERACTION_ABSENT",
            Some("DEMOGRAPHIC_NOISE_DOMINATES_WEAK_DESCENDANT_DIFFERENCES")
        )
    );
}

#[test]
fn environmental_inputs_are_local_measurable_and_label_independent() {
    let contrast = environmental_contrast();
    assert!(contrast.mechanistically_selectable);
    assert!(contrast.h.resource_timing_variance > contrast.neutral.resource_timing_variance);
    assert!(contrast.b.damage_per_350_steps > contrast.neutral.damage_per_350_steps);
    assert!(!contrast.equations_contain_environment_labels);
}

#[test]
fn finite_allocation_and_reciprocal_interaction_reject_universal_superiority() {
    let h = Allocation::new([0.45, 0.25, 0.10, 0.20]).unwrap();
    let b = Allocation::new([0.20, 0.20, 0.45, 0.15]).unwrap();
    assert!(Allocation::new([0.6, 0.6, 0.0, 0.0]).is_err());

    let reciprocal = reciprocal_interaction([1.4, 0.7, 0.8], [0.6, 1.3, 0.8]);
    assert!(reciprocal.reciprocal);
    assert!(!reciprocal.universally_superior);

    let universal = reciprocal_interaction([1.4, 1.3, 1.2], [0.8, 0.7, 0.6]);
    assert!(!universal.reciprocal);
    assert!(universal.universally_superior);

    assert!((h.fractions.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!((b.fractions.iter().sum::<f64>() - 1.0).abs() < 1e-12);
}

#[test]
fn candidate_scoring_is_deterministic_selects_one_route_and_freezes_complete_contract() {
    let first = candidate_review();
    let second = candidate_review();
    assert_eq!(first, second);
    assert!(!first.iter().find(|c| c.candidate == "A").unwrap().eligible);
    assert!(first.iter().find(|c| c.candidate == "B").unwrap().eligible);
    assert!(!first.iter().find(|c| c.candidate == "C").unwrap().eligible);
    assert!(!first.iter().find(|c| c.candidate == "D").unwrap().eligible);

    let selected = select_route(&first);
    assert_eq!(selected.as_deref(), Some("B"));
    assert_eq!(first.iter().filter(|c| c.selected).count(), 1);

    let contract = freeze_d096_contract(selected.as_deref()).expect("selected route contract");
    for key in [
        "scientific_hypothesis",
        "equation_identity",
        "hereditary_representation",
        "conserved_expression",
        "mandatory_tradeoff",
        "environmental_coupling",
        "gates",
        "phase_authority",
    ] {
        assert!(!contract[key].is_null(), "missing {key}");
    }
    assert_eq!(contract["implementation_status"], "NOT_IMPLEMENTED");
    assert_eq!(contract["phase3_authorized"], false);
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
