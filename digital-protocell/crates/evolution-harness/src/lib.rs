//! Digital Cell-owned protocol and bookkeeping harness.
//!
//! The crate owns experiment execution semantics, not organism science. The
//! adapter boundary contains no commands for fitness, reproduction, healing,
//! survival, growth, or scripted behavior.

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
        alive: bool,
        reproduction_step: Option<u64>,
        phenotype: String,
        hereditary_state: String,
    }

    struct SyntheticAdapter {
        pressure_on_second: bool,
        provide_evidence: bool,
    }

    impl OrganismAdapter for SyntheticAdapter {
        type Organism = SyntheticOrganism;

        fn initialize_founder(
            &mut self,
            founder: &FounderIdentityV1,
            _context: FounderInitializationContext,
        ) -> Result<Self::Organism, AdapterError> {
            Ok(SyntheticOrganism {
                alive: true,
                reproduction_step: Some(2),
                phenotype: "baseline".into(),
                hereditary_state: format!("h:{}", founder.heredity_hash),
            })
        }

        fn accepted_dt(&self) -> f64 {
            0.25
        }
        fn environment_capabilities(&self) -> Vec<EnvironmentCapability> {
            vec![
                EnvironmentCapability::ContinuousResources,
                EnvironmentCapability::PulsedResources,
                EnvironmentCapability::Scarcity,
                EnvironmentCapability::SharedCompetition,
                EnvironmentCapability::SpatialLocalResources,
                EnvironmentCapability::Damage,
                EnvironmentCapability::Transitions,
            ]
        }
        fn apply_declared_environment(
            &mut self,
            _organism: &mut Self::Organism,
            _environment: &EnvironmentProtocolV1,
            _accepted_step: u64,
            _accepted_simulated_time: f64,
            context: EnvironmentContext,
        ) -> Result<Vec<AdapterEnvironmentEvent>, AdapterError> {
            if self.pressure_on_second && context.organism_index == 1 {
                return Ok(vec![AdapterEnvironmentEvent {
                    event_type: EventType::ResourcePulse,
                    metadata: BTreeMap::new(),
                }]);
            }
            Ok(Vec::new())
        }

        fn advance(
            &mut self,
            organism: &mut Self::Organism,
            _environment: &EnvironmentProtocolV1,
            accepted_step: u64,
            _accepted_simulated_time: f64,
        ) -> Result<AdvanceOutcome<Self::Organism>, AdapterError> {
            if organism.reproduction_step == Some(accepted_step) {
                organism.alive = false;
                let mut a = organism.clone();
                let mut b = organism.clone();
                a.alive = true;
                b.alive = true;
                a.reproduction_step = None;
                b.reproduction_step = None;
                return Ok(AdvanceOutcome::Fission {
                    offspring: vec![a, b],
                    accepted_dt: 0.25,
                    metadata: BTreeMap::new(),
                });
            }
            Ok(AdvanceOutcome::Continuing {
                accepted_dt: 0.25,
                metadata: BTreeMap::new(),
            })
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
        fn heredity_evidence(
            &self,
            parent: Option<&Self::Organism>,
            organism: &Self::Organism,
        ) -> HeredityEvidenceV1 {
            if !self.provide_evidence {
                return HeredityEvidenceV1::unavailable(
                    "synthetic adapter deliberately omitted mechanism qualifier",
                );
            }
            let preserved = parent
                .map(|p| p.hereditary_state == organism.hereditary_state)
                .unwrap_or(false);
            HeredityEvidenceV1 {
                observable: true,
                preserved,
                comparison_basis: "exact_synthetic_state_match".into(),
                metric: "state_match".into(),
                value: Some(if preserved { 1.0 } else { 0.0 }),
                qualification: preserved,
                reason: "synthetic hereditary state compared to parent".into(),
            }
        }
        fn phenotype_evidence(
            &self,
            _environment: &EnvironmentProtocolV1,
            organism: &Self::Organism,
        ) -> PhenotypeEvidenceV1 {
            if !self.provide_evidence {
                return PhenotypeEvidenceV1::unavailable(
                    "synthetic adapter deliberately omitted mechanism qualifier",
                );
            }
            let expressed = organism.phenotype == "baseline";
            PhenotypeEvidenceV1 {
                observable: true,
                expressed,
                comparison_basis: "declared_synthetic_baseline".into(),
                metric: "baseline_present".into(),
                value: Some(if expressed { 1.0 } else { 0.0 }),
                qualification: expressed,
                reason: "synthetic phenotype is explicitly measurable".into(),
            }
        }
    }

    fn protocol() -> ExperimentProtocolV1 {
        let mut protocol =
            ExperimentProtocolV1::minimal("synthetic_v1", "seasonal_lean_v1", "mutation_none");
        protocol.environment_protocol.resource_mode = ResourceMode::Pulsed;
        protocol.environment_protocol.pulse_schedule = vec![0.0];
        protocol
            .environment_protocol
            .resource_ecology
            .pulse_schedule = vec![0.0];
        protocol.minimum_generation_requirement = 1;
        protocol.selective_pressure = Some(SelectivePressureContractV1 {
            schema: "SelectivePressureContractV1".into(),
            contrast_id: "synthetic_resource_pulse_contrast".into(),
            campaign_role: CampaignRole::Treatment,
            treatment_environment: "seasonal_lean_v1".into(),
            neutral_environment: "neutral_v1".into(),
            pressure_event_or_condition: "resource_pulse".into(),
            pressure_start: 0.0,
            expected_phenotype_dimension: "birth_count".into(),
        });
        protocol
    }

    fn founder(seed: u64) -> FounderIdentityV1 {
        FounderIdentityV1::new(
            7 + seed,
            "mesh_v1",
            "heredity_v1",
            "baseline",
            "material",
            seed,
            "none",
        )
    }

    fn synthetic_adapter() -> SyntheticAdapter {
        SyntheticAdapter {
            pressure_on_second: false,
            provide_evidence: true,
        }
    }

    fn opaque_adapter() -> SyntheticAdapter {
        SyntheticAdapter {
            pressure_on_second: false,
            provide_evidence: false,
        }
    }

    fn neutral_protocol(mut protocol: ExperimentProtocolV1) -> ExperimentProtocolV1 {
        protocol.environment_protocol.environment_id = "neutral_v1".into();
        protocol.selective_pressure.as_mut().unwrap().campaign_role = CampaignRole::Neutral;
        protocol
    }

    #[test]
    fn event_ids_and_real_time_are_monotonic() {
        let mut ledger = EventLedger::default();
        ledger
            .append(EventV1::founder(0, 0.0, 0, 0, "e", "p", 1))
            .unwrap();
        ledger
            .append(EventV1::experiment_end(0, 0.25, 1, 0, "e", "p"))
            .unwrap();
        assert!(ledger.validate().is_ok());
        assert_eq!(ledger.events[1].accepted_simulated_time, 0.25);
    }

    #[test]
    fn event_ledger_fails_closed_for_unknown_and_dead_parent() {
        let mut ledger = EventLedger::default();
        assert!(matches!(
            ledger.append(EventV1::death(0, 0.0, 0, 0, 99, "e", "p")),
            Err(EventLedgerError::DeathBeforeBirth(99))
        ));
        ledger
            .append(EventV1::founder(0, 0.0, 0, 0, "e", "p", 1))
            .unwrap();
        ledger
            .append(EventV1::death(0, 0.5, 1, 0, 1, "e", "p"))
            .unwrap();
        assert!(matches!(
            ledger.append(EventV1::birth(0, 0.5, 1, 0, 2, Some(1), 1, "e", "p")),
            Err(EventLedgerError::ParentDead { .. })
        ));
    }

    #[test]
    fn lineage_depth_is_descendant_depth_and_ancestor_depth_is_separate() {
        let mut lineage = LineageTracker::default();
        lineage.register_founder(1, 1, 1, 0.0).unwrap();
        lineage.register_offspring(2, 1, 1, 2, 0.1).unwrap();
        lineage.register_offspring(3, 1, 1, 3, 0.1).unwrap();
        lineage.register_offspring(4, 2, 1, 4, 0.2).unwrap();
        lineage.register_offspring(5, 2, 1, 5, 0.2).unwrap();
        lineage.register_offspring(6, 3, 1, 6, 0.2).unwrap();
        assert_eq!(lineage.descendant_depth(1), 2);
        assert_eq!(lineage.lineage_depth(1), 2);
        assert_eq!(lineage.ancestor_depth(6), 2);
    }

    #[test]
    fn zero_generation_is_not_a_valid_selection_result() {
        let mut harness = EvolutionHarness::new(synthetic_adapter(), {
            let mut p = protocol();
            p.environment_protocol.resource_mode = ResourceMode::Continuous;
            p.environment_protocol.pulse_schedule.clear();
            p.maximum_accepted_horizon = 0.0;
            p
        })
        .unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.run_to_horizon().unwrap();
        assert_eq!(
            harness.replicate_result(0).classification,
            FailureClass::SelectionUntestableZeroGeneration
        );
    }

    #[test]
    fn fission_births_use_actual_simulated_time_and_intervals() {
        let mut harness = EvolutionHarness::new(synthetic_adapter(), protocol()).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        harness.advance_one().unwrap();
        assert_eq!(harness.generation.max_generation, 1);
        assert_eq!(harness.generation.completed_births, 2);
        assert_eq!(harness.accepted_simulated_time, 0.5);
        assert_eq!(harness.generation.generation_times, vec![0.5]);
    }

    #[test]
    fn replicate_runner_executes_all_seeds_independently() {
        let mut p = protocol();
        p.replicates = 3;
        p.random_seeds = vec![0, 1, 2];
        p.maximum_accepted_horizon = 1.0;
        let campaign = ReplicateRunner::run_campaign::<SyntheticAdapter, _, _>(
            p,
            |_rep, _seed| synthetic_adapter(),
            |_rep, seed| founder(seed),
        )
        .unwrap();
        assert_eq!(campaign.replicate_results.len(), 3);
        assert_eq!(
            campaign
                .replicate_results
                .iter()
                .map(|r| r.replicate)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert!(campaign
            .replicate_results
            .iter()
            .all(|r| r.event_ledger_valid));
    }

    #[test]
    fn treatment_neutral_selection_uses_two_real_campaigns() {
        let mut p = protocol();
        p.replicates = 2;
        p.random_seeds = vec![0, 1];
        p.maximum_accepted_horizon = 1.0;
        let treatment = ReplicateRunner::run_campaign::<SyntheticAdapter, _, _>(
            p.clone(),
            |_rep, _seed| synthetic_adapter(),
            |_rep, seed| founder(seed),
        )
        .unwrap();
        let neutral = ReplicateRunner::run_campaign::<SyntheticAdapter, _, _>(
            neutral_protocol(p),
            |_rep, _seed| synthetic_adapter(),
            |_rep, seed| founder(seed),
        )
        .unwrap();
        let analysis = DefaultSelectionObserver.observe(&treatment, &neutral);
        assert_eq!(
            analysis.classification,
            FailureClass::ValidNoSelectionEffect
        );
        assert_eq!(analysis.replicate_count, 2);
        assert_eq!(analysis.absolute_effect, 0.0);
    }

    #[test]
    fn nonzero_mutation_requires_an_adapter_contract() {
        let mut p = protocol();
        p.mutation_protocol.mutation_protocol_id = "unknown_existing_mechanism".into();
        p.mutation_protocol.mutation_rate = 0.1;
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        assert!(matches!(
            harness.advance_one(),
            Err(HarnessError::Adapter(AdapterError::Unavailable))
        ));
    }

    #[test]
    fn historical_and_d094_fixtures_are_non_executable_until_provenance_is_complete() {
        assert_eq!(historical_protocols().len(), 4);
        assert!(historical_protocols()
            .iter()
            .all(|p| !p.provenance.execution_authorized
                && !p.provenance.unresolved_values.is_empty()));
        let d094 = d094_requalified_protocol();
        assert!(!d094.provenance.execution_authorized);
        assert!(d094.environment_protocol.pulse_schedule.is_empty());
        assert!(d094.environment_protocol.damage_interval.is_none());
    }

    #[test]
    fn execution_gate_rejects_unresolved_protocol() {
        let d094 = d094_requalified_protocol();
        assert!(
            matches!(EvolutionHarness::new(synthetic_adapter(), d094), Err(HarnessError::Protocol(message)) if message.contains("execution"))
        );
    }

    #[test]
    fn unresolved_d094_cannot_execute() {
        let d094 = d094_requalified_protocol();
        let result = ReplicateRunner::run_campaign::<SyntheticAdapter, _, _>(
            d094,
            |_rep, _seed| synthetic_adapter(),
            |_rep, seed| founder(seed),
        );
        assert!(
            matches!(result, Err(HarnessError::Protocol(message)) if message.contains("execution"))
        );
    }

    #[test]
    fn historical_fixture_cannot_execute() {
        for historical in historical_protocols() {
            assert!(
                matches!(EvolutionHarness::new(synthetic_adapter(), historical), Err(HarnessError::Protocol(message)) if message.contains("execution"))
            );
        }
    }

    #[test]
    fn execution_authorized_protocol_can_execute() {
        let result = ReplicateRunner::run_campaign::<SyntheticAdapter, _, _>(
            protocol(),
            |_rep, _seed| synthetic_adapter(),
            |_rep, seed| founder(seed),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn multi_organism_fission_plus_pressure_ordering() {
        let mut p = protocol();
        p.placement_protocol.initial_coordinates = vec![[1.0, 2.0], [3.0, 4.0]];
        p.maximum_accepted_horizon = 0.5;
        let mut harness = EvolutionHarness::new(
            SyntheticAdapter {
                pressure_on_second: true,
                provide_evidence: true,
            },
            p,
        )
        .unwrap();
        harness
            .initialize_founders(vec![founder(0), founder(1)])
            .unwrap();
        harness.advance_one().unwrap();
        harness.advance_one().unwrap();
        assert!(harness.ledger.validate().is_ok());
        assert!(harness
            .ledger
            .events
            .windows(2)
            .all(|pair| pair[0].accepted_simulated_time <= pair[1].accepted_simulated_time));
    }

    #[test]
    fn parent_causal_generation_duration() {
        let mut tracker = GenerationTracker::default();
        tracker.record_founder(0.0);
        tracker.record_completed_fission(1, 0, 0.0, 4.0);
        tracker.record_completed_fission(2, 1, 4.0, 7.0);
        tracker.record_completed_fission(3, 1, 4.0, 10.0);
        assert_eq!(tracker.generation_times, vec![4.0, 3.0, 6.0]);
        assert_eq!(tracker.median_generation_time(), Some(4.0));
    }

    #[test]
    fn independent_branch_generation_durations() {
        let mut tracker = GenerationTracker::default();
        tracker.record_founder(0.0);
        tracker.record_completed_fission(1, 0, 0.0, 4.0);
        tracker.record_completed_fission(2, 1, 4.0, 7.0);
        tracker.record_completed_fission(3, 1, 4.0, 10.0);
        assert_eq!(tracker.generation_intervals[1].generation_duration, 3.0);
        assert_eq!(tracker.generation_intervals[2].generation_duration, 6.0);
    }

    #[test]
    fn heredity_nonempty_string_is_not_sufficient() {
        let mut harness = EvolutionHarness::new(opaque_adapter(), protocol()).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        harness.advance_one().unwrap();
        assert!(!harness.replicate_result(0).heredity_preserved);
    }

    #[test]
    fn phenotype_nonempty_string_is_not_sufficient() {
        let mut harness = EvolutionHarness::new(opaque_adapter(), protocol()).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        harness.advance_one().unwrap();
        assert!(!harness.replicate_result(0).phenotype_measurable);
    }

    #[test]
    fn missing_mechanism_qualifier_fails_closed() {
        let mut harness = EvolutionHarness::new(opaque_adapter(), protocol()).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        harness.advance_one().unwrap();
        assert_eq!(
            harness.replicate_result(0).classification,
            FailureClass::HeredityNotPreserved
        );
    }

    #[test]
    fn declared_pressure_reached() {
        let mut harness = EvolutionHarness::new(synthetic_adapter(), protocol()).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        assert!(harness.replicate_result(0).pressure_reached);
    }

    #[test]
    fn unrelated_environment_event_does_not_count_as_pressure() {
        let mut p = protocol();
        p.environment_protocol.pulse_schedule.clear();
        p.environment_protocol
            .resource_ecology
            .pulse_schedule
            .clear();
        p.environment_protocol.resource_mode = ResourceMode::Continuous;
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        assert!(!harness.replicate_result(0).pressure_reached);
    }

    #[test]
    fn scarcity_end_does_not_start_pressure() {
        let mut p = protocol();
        p.environment_protocol.resource_mode = ResourceMode::Scarcity;
        p.environment_protocol.pulse_schedule.clear();
        p.environment_protocol
            .resource_ecology
            .pulse_schedule
            .clear();
        p.environment_protocol.scarcity_schedule = vec![TimeWindowV1 {
            start: 0.0,
            end: 0.25,
        }];
        p.selective_pressure
            .as_mut()
            .unwrap()
            .pressure_event_or_condition = "scarcity_end".into();
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        assert!(!harness.replicate_result(0).pressure_reached);
    }

    #[test]
    fn pressure_before_reproduction_uses_declared_contrast() {
        let mut harness = EvolutionHarness::new(synthetic_adapter(), protocol()).unwrap();
        harness.initialize_founder(founder(0)).unwrap();
        harness.advance_one().unwrap();
        harness.advance_one().unwrap();
        let result = harness.replicate_result(0);
        assert!(result.pressure_reached);
        assert!(result.pressure_before_reproduction);
    }

    #[test]
    fn two_founders_initialize() {
        let mut p = protocol();
        p.placement_protocol.initial_coordinates = vec![[1.0, 2.0], [3.0, 4.0]];
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        let ids = harness
            .initialize_founders(vec![founder(0), founder(1)])
            .unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn founder_ids_are_unique() {
        let mut p = protocol();
        p.placement_protocol.initial_coordinates = vec![[1.0, 2.0], [3.0, 4.0]];
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        assert!(
            matches!(harness.initialize_founders(vec![founder(0), founder(0)]), Err(HarnessError::PopulationInitialization(message)) if message.contains("duplicate"))
        );
    }

    #[test]
    fn founder_lineages_are_independent() {
        let mut p = protocol();
        p.placement_protocol.initial_coordinates = vec![[1.0, 2.0], [3.0, 4.0]];
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        let ids = harness
            .initialize_founders(vec![founder(0), founder(1)])
            .unwrap();
        assert_ne!(
            harness.lineage.ancestry[&ids[0]].lineage_id,
            harness.lineage.ancestry[&ids[1]].lineage_id
        );
    }

    #[test]
    fn population_placement_is_reproducible() {
        let mut p = protocol();
        p.placement_protocol.initial_coordinates = vec![[1.0, 2.0], [3.0, 4.0]];
        let mut harness = EvolutionHarness::new(synthetic_adapter(), p).unwrap();
        let ids = harness
            .initialize_founders(vec![founder(0), founder(1)])
            .unwrap();
        assert_eq!(harness.population.records[&ids[0]].placement, [1.0, 2.0]);
        assert_eq!(harness.population.records[&ids[1]].placement, [3.0, 4.0]);
    }
}
