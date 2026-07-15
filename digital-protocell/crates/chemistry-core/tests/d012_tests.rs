//! D-012 stoichiometric descriptor and conservation analysis tests.

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
fn test_v2_unit_yield_is_strictly_conservative() {
    let one = Rational::ONE;
    let reactions = v2_internal_reactions(one, one, one);
    let matrix = stoichiometric_matrix(&reactions);
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::StrictlyConservative
    );
}
