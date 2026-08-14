//! A thin, Digital Cell-owned experiment layer.
//!
//! This crate owns protocol, population, event, generation, lineage and
//! analysis bookkeeping. It does not assign fitness or tell an organism to
//! reproduce, survive, die, heal, or change biology.

mod adapter;
mod analysis;
mod events;
mod harness;
mod historical;
mod lineage;
mod mesh_adapter;
mod population;
mod protocols;
mod selection;

pub use adapter::*;
pub use analysis::*;
pub use events::*;
pub use harness::*;
pub use historical::*;
pub use lineage::*;
pub use mesh_adapter::*;
pub use population::*;
pub use protocols::*;
pub use selection::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Clone, Debug)]
    struct SyntheticOrganism {
        generation: u32,
        alive: bool,
        reproduction_step: Option<u64>,
        phenotype: String,
        hereditary_state: String,
    }

    struct SyntheticAdapter;

    impl OrganismAdapter for SyntheticAdapter {
        type Organism = SyntheticOrganism;

        fn initialize_founder(
            &self,
            founder: &FounderIdentityV1,
        ) -> Result<Self::Organism, AdapterError> {
            Ok(SyntheticOrganism {
                generation: 0,
                alive: true,
                reproduction_step: founder.seed.checked_add(2),
                phenotype: "baseline".into(),
                hereditary_state: format!("h:{}", founder.heredity_hash),
            })
        }

        fn advance(
            &self,
            organism: &mut Self::Organism,
            _environment: &EnvironmentProtocolV1,
            accepted_step: u64,
            _accepted_simulated_time: u64,
        ) -> Result<AdvanceOutcome<Self::Organism>, AdapterError> {
            if !organism.alive {
                return Ok(AdvanceOutcome::Died {
                    reason: "already_dead".into(),
                });
            }
            if organism.reproduction_step == Some(accepted_step) {
                organism.alive = false;
                let mut a = organism.clone();
                let mut b = organism.clone();
                a.alive = true;
                b.alive = true;
                a.generation += 1;
                b.generation += 1;
                a.reproduction_step = None;
                b.reproduction_step = None;
                return Ok(AdvanceOutcome::Fission {
                    offspring: vec![a, b],
                    metadata: BTreeMap::new(),
                });
            }
            Ok(AdvanceOutcome::Continuing)
        }

        fn is_alive(&self, organism: &Self::Organism) -> bool {
            organism.alive
        }

        fn phenotype(&self, organism: &Self::Organism) -> String {
            organism.phenotype.clone()
        }

        fn hereditary_state(&self, organism: &Self::Organism) -> String {
            organism.hereditary_state.clone()
        }
    }

    fn protocol() -> ExperimentProtocolV1 {
        ExperimentProtocolV1::minimal("synthetic_v1", "seasonal_lean_v1", "mutation_none")
    }

    fn founder() -> FounderIdentityV1 {
        FounderIdentityV1::new(7, "mesh_v1", "heredity_v1", "baseline", "material", 0, "none")
    }

    #[test]
    fn event_ids_unique_and_time_monotonic() {
        let mut ledger = EventLedger::default();
        ledger.append(EventV1::founder(1, 0, 0, "e", "p", "seed")).unwrap();
        ledger.append(EventV1::experiment_end(2, 1, 0, "e", "p")).unwrap();
        assert!(ledger.validate().is_ok());
        assert_eq!(ledger.events[0].event_id, 1);
        assert_eq!(ledger.events[1].event_id, 2);
    }

    #[test]
    fn birth_requires_parent_and_no_double_birth() {
        let mut ledger = EventLedger::default();
        ledger.append(EventV1::birth(1, 1, 0, 10, Some(9), 1, "e", "p")).unwrap();
        assert!(ledger.validate().unwrap_err().to_string().contains("parent"));
        ledger = EventLedger::default();
        ledger.append(EventV1::founder(1, 0, 0, "e", "p", "seed")).unwrap();
        ledger.append(EventV1::birth(2, 1, 0, 10, Some(1), 1, "e", "p"))
            .unwrap();
        ledger.append(EventV1::birth(3, 2, 0, 10, Some(1), 1, "e", "p"))
            .unwrap();
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn zero_generation_is_untestable() {
        let mut harness = EvolutionHarness::new(SyntheticAdapter, protocol()).unwrap();
        harness.initialize_founder(founder()).unwrap();
        let result = harness.replicate_result(0, 0);
        assert_eq!(result.classification, FailureClass::SelectionUntestableZeroGeneration);
    }

    #[test]
    fn lineage_tree_has_causal_depth_and_descendants() {
        let mut lineage = LineageTracker::default();
        lineage.register_founder(1, 1, 1, 0).unwrap();
        lineage.register_offspring(2, 1, 1, 1, 1).unwrap();
        lineage.register_offspring(3, 1, 1, 1, 1).unwrap();
        lineage.register_offspring(4, 2, 1, 2, 2).unwrap();
        lineage.register_offspring(5, 2, 1, 2, 2).unwrap();
        lineage.register_offspring(6, 3, 1, 2, 2).unwrap();
        assert_eq!(lineage.generation(1), Some(0));
        assert_eq!(lineage.generation(2), Some(1));
        assert_eq!(lineage.generation(4), Some(2));
        assert_eq!(lineage.descendant_count(1), 5);
        assert_eq!(lineage.lineage_depth(1), 2);
    }

    #[test]
    fn same_seed_same_protocol_has_same_hash() {
        let a = protocol();
        let b = protocol();
        assert_eq!(a.hash(), b.hash());
        assert_eq!(a.environment_protocol.hash(), b.environment_protocol.hash());
    }

    #[test]
    fn pressure_after_reproduction_is_invalid_but_before_is_interpretable() {
        assert_eq!(classify_ecology_timing(10, 5), FailureClass::EcologyPressurePostReproduction);
        assert_eq!(classify_ecology_timing(2, 5), FailureClass::InterpretableTiming);
    }

    #[test]
    fn mutation_protocol_has_no_selection_input() {
        let json = serde_json::to_string(&MutationProtocolV1::default()).unwrap();
        assert!(!json.contains("fitness"));
        assert!(!json.contains("winner"));
        assert!(!json.contains("survive"));
    }

    #[test]
    fn observer_is_measurement_only() {
        let result = ReplicateResultV1::empty("neutral", 0);
        let observed = DefaultSelectionObserver.observe(&result);
        assert_eq!(observed.replicate_count, 1);
        assert_eq!(observed.relative_effect, 0.0);
    }

    #[test]
    fn generation_comes_from_completed_fission_and_birth_events() {
        let mut harness = EvolutionHarness::new(SyntheticAdapter, protocol()).unwrap();
        harness.initialize_founder(founder()).unwrap();
        let environment = harness.protocol.environment_protocol.clone();
        harness.advance_one(&environment).unwrap();
        harness.advance_one(&environment).unwrap();
        assert_eq!(harness.generation.max_generation, 1);
        assert_eq!(harness.generation.completed_births, 2);
        assert_eq!(harness.ledger.events.iter().filter(|event| matches!(&event.event_type, EventType::Birth)).count(), 2);
    }

    #[test]
    fn lineage_persists_after_death() {
        let mut lineage = LineageTracker::default();
        lineage.register_founder(1, 1, 1, 0).unwrap();
        lineage.register_offspring(2, 1, 1, 2, 1).unwrap();
        lineage.record_death(1, 3).unwrap();
        assert_eq!(lineage.parent(2), Some(1));
        assert_eq!(lineage.ancestry.get(&1).unwrap().death_time, Some(3));
    }

    #[test]
    fn event_integrity_rejects_double_death() {
        let mut ledger = EventLedger::default();
        ledger.append(EventV1::founder(1, 0, 0, "e", "p", "1")).unwrap();
        ledger.append(EventV1::death(2, 1, 1, 1, 0, "e", "p")).unwrap();
        ledger.append(EventV1::death(3, 2, 2, 1, 0, "e", "p")).unwrap();
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn same_seed_same_events() {
        let mut first = EvolutionHarness::new(SyntheticAdapter, protocol()).unwrap();
        let mut second = EvolutionHarness::new(SyntheticAdapter, protocol()).unwrap();
        first.initialize_founder(founder()).unwrap();
        second.initialize_founder(founder()).unwrap();
        let environment = first.protocol.environment_protocol.clone();
        first.advance_one(&environment).unwrap();
        second.advance_one(&environment).unwrap();
        assert_eq!(first.ledger, second.ledger);
    }

    #[test]
    fn treatment_neutral_parity_is_structural() {
        let treatment = protocol();
        let neutral = protocol();
        assert_eq!(treatment.placement_protocol, neutral.placement_protocol);
        assert_eq!(treatment.mutation_protocol, neutral.mutation_protocol);
        assert_eq!(treatment.replicates, neutral.replicates);
        assert_eq!(treatment.random_seeds, neutral.random_seeds);
    }

    #[test]
    fn valid_no_effect_is_distinct_from_invalid_zero_generation() {
        let mut result = ReplicateResultV1::empty("e", 0);
        assert_eq!(result.classification, FailureClass::SelectionUntestableZeroGeneration);
        result.max_generation = 2;
        result.classification = FailureClass::ValidNoSelectionEffect;
        assert_ne!(result.classification, FailureClass::SelectionUntestableZeroGeneration);
    }

    #[test]
    fn founder_identity_is_stable() {
        assert_eq!(founder(), founder());
    }

    #[test]
    fn real_mesh_adapter_is_narrow_and_uses_existing_mesh_state() {
        let adapter = DigitalCellMeshAdapter::default();
        let mesh = adapter.initialize_founder(&founder()).unwrap();
        assert!(adapter.is_alive(&mesh));
        assert!(adapter.phenotype(&mesh).contains("mesh_vertices"));
        assert!(adapter.hereditary_state(&mesh).contains("equation:"));
    }

    #[test]
    fn historical_protocols_are_declarative() {
        let protocols = historical_protocols();
        assert_eq!(protocols.len(), 4);
        assert!(protocols.iter().all(|p| p.validate().is_ok()));
        assert!(d094_requalified_protocol().validate().is_ok());
    }
}
