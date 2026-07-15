//! D-012 stoichiometric descriptor, conservation analysis, and v1 audit tests.

use chemistry_core::stoichiometry::*;

#[test]
fn test_v1_descriptor_order_matches_governed_reaction_list() {
    let ids: Vec<_> = v1_internal_reactions()
        .iter()
        .map(|r| r.reaction)
        .collect();
    assert_eq!(
        ids,
        vec![
            ReactionId::Activation,
            ReactionId::CatalystProduction,
            ReactionId::StructureProduction,
            ReactionId::MembraneProduction,
            ReactionId::StructureDecay,
            ReactionId::CatalystDecay,
            ReactionId::ActivatedDecay,
            ReactionId::MembraneDecay,
            ReactionId::MembraneDetachment,
        ]
    );
}

#[test]
fn test_positive_conservation_vector_detection() {
    // A → W alone is conservative under all-ones material weight.
    let rx = [ReactionStoichiometry::new(
        ReactionId::ActivatedDecay,
        {
            let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
            d[SpeciesId::A.index()] = Rational::from_i64(-1);
            d[SpeciesId::W.index()] = Rational::from_i64(1);
            d
        },
    )];
    let matrix = stoichiometric_matrix(&rx);
    let m = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    assert!(verify_m_transpose_s_zero(&m, &matrix));
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::StrictlyConservative
    );
}

#[test]
fn test_nonconservative_reaction_is_identified() {
    // A → C + W creates material under all-ones weight.
    let rx = [ReactionStoichiometry::new(
        ReactionId::CatalystProduction,
        {
            let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
            d[SpeciesId::C.index()] = Rational::from_i64(1);
            d[SpeciesId::A.index()] = Rational::from_i64(-1);
            d[SpeciesId::W.index()] = Rational::from_i64(1);
            d
        },
    )];
    let matrix = stoichiometric_matrix(&rx);
    let positives = positive_conservation_vectors(&matrix);
    assert!(positives.is_empty());
    let all_ones = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    let bad = nonconservative_reactions_under_vector(&all_ones, &rx);
    assert_eq!(bad, vec![ReactionId::CatalystProduction]);
}

#[test]
fn test_v1_stoichiometric_matrix_matches_reactions() {
    let reactions = v1_internal_reactions();
    let matrix = stoichiometric_matrix(reactions);
    assert_eq!(matrix.len(), SEVEN_FIELD_COUNT);
    assert_eq!(matrix[0].len(), ReactionId::INTERNAL_COUNT);

    for (col, rx) in reactions.iter().enumerate() {
        for (row, &expected) in rx.delta.iter().enumerate() {
            assert_eq!(
                matrix[row][col], expected,
                "species {:?} reaction {:?}",
                SpeciesId::ALL[row], rx.reaction
            );
        }
    }

    // Spot-check governed v1 columns against runtime-isolated deltas.
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::Activation),
        v1_runtime_activation_delta(1.0)
    );
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::CatalystProduction),
        v1_runtime_catalyst_production_delta(1.0)
    );
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::StructureProduction),
        v1_runtime_structure_production_delta(1.0)
    );
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::MembraneProduction),
        v1_runtime_membrane_production_delta(1.0)
    );
}

#[test]
fn test_v1_positive_conservation_vector_search() {
    let matrix = stoichiometric_matrix(v1_internal_reactions());
    let positives = positive_conservation_vectors(&matrix);
    assert!(
        positives.is_empty(),
        "v1 must not admit a strictly positive conservation vector: {:?}",
        positives
    );
}

#[test]
fn test_v1_nonconservative_productive_reaction_detection() {
    let reactions = v1_internal_reactions();
    let all_ones = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    let bad = nonconservative_reactions_under_vector(&all_ones, reactions);
    assert!(bad.contains(&ReactionId::CatalystProduction));
    assert!(bad.contains(&ReactionId::MembraneProduction));
    assert!(bad.contains(&ReactionId::MembraneDecay));
    assert!(bad.contains(&ReactionId::MembraneDetachment));

    let audit = run_v1_stoichiometric_audit();
    assert_eq!(audit.primary_finding, "D012_NONCONSERVATIVE_V1_CONFIRMED");
    assert_eq!(
        audit.conservation_class,
        ConservationClass::NoPositiveConservationVector
    );
    assert_eq!(
        audit.d011_branch_recommendation,
        "SKIP_D011_EXPENSIVE_COMPLETION_SUPERSEDED_BY_INVALID_STOICHIOMETRY"
    );
}

#[test]
fn test_field_ledgers_can_close_while_total_stoichiometry_fails() {
    // Stage-C per-field ledgers close for activation (N,F,A,W balance) while
    // catalyst production creates net material under any strictly positive weight.
    let activation = v1_runtime_activation_delta(1.0);
    let sum_act: f64 = activation.iter().sum();
    assert!((sum_act).abs() < 1e-12, "activation conserves under sum check");

    let reproduction = v1_runtime_catalyst_production_delta(1.0);
    let sum_rep: f64 = reproduction.iter().sum();
    assert!((sum_rep - 1.0).abs() < 1e-12, "reproduction creates +1 net mass");

    let reactions = v1_internal_reactions();
    let matrix = stoichiometric_matrix(reactions);
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::NoPositiveConservationVector
    );
}

#[test]
fn test_v2_unit_yield_is_strictly_conservative() {
    let one = Rational::ONE;
    let reactions = v2_internal_reactions(one, one, one);
    let matrix = stoichiometric_matrix(&reactions);
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::StrictlyConservative
    );
    let m = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    assert!(verify_m_transpose_s_zero(&m, &matrix));
}

#[test]
fn write_v1_audit_json_artifact() {
    let json = v1_audit_json_pretty();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("audit json valid");
    assert_eq!(
        parsed["primary_finding"].as_str(),
        Some("D012_NONCONSERVATIVE_V1_CONFIRMED")
    );
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../experiments/generated/d012/v1_stoichiometric_audit"
    );
    std::fs::create_dir_all(dir).expect("audit dir");
    std::fs::write(format!("{dir}/audit.json"), json).expect("write audit.json");
}
