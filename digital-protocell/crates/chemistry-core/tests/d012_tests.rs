//! D-012 stoichiometric descriptor tests.

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
