use crate::{
    AdapterError, AdvanceOutcome, EnvironmentProtocolV1, EventLedger, EventType, EventV1,
    FailureClass, FounderIdentityV1, GenerationTracker, LineageTracker, OrganismAdapter,
    PopulationManager, ReplicateResultV1,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("protocol invalid: {0}")]
    Protocol(String),
    #[error("adapter error: {0}")]
    Adapter(#[from] AdapterError),
    #[error("population error: {0}")]
    Population(String),
    #[error("lineage error: {0}")]
    Lineage(String),
    #[error("event ledger error: {0}")]
    Events(String),
    #[error("founder has already been initialized")]
    FounderAlreadyInitialized,
    #[error("organism {0} is missing from the adapter state")]
    MissingOrganism(u64),
}

pub struct EvolutionHarness<A: OrganismAdapter> {
    pub adapter: A,
    pub protocol: crate::ExperimentProtocolV1,
    pub ledger: EventLedger,
    pub population: PopulationManager,
    pub lineage: LineageTracker,
    pub generation: GenerationTracker,
    pub organisms: BTreeMap<u64, A::Organism>,
    pub accepted_step: u64,
    pub accepted_simulated_time: u64,
}

pub struct ReplicateRunner;

impl ReplicateRunner {
    pub fn validate_protocol(protocol: &crate::ExperimentProtocolV1) -> Result<(), HarnessError> {
        protocol.validate().map_err(|e| HarnessError::Protocol(e.to_string()))
    }

    pub fn run<A: OrganismAdapter>(
        harness: &mut EvolutionHarness<A>,
        environment: &EnvironmentProtocolV1,
        maximum_steps: u64,
        seed: u64,
    ) -> Result<ReplicateResultV1, HarnessError> {
        for _ in 0..maximum_steps {
            if harness.population.living_count() == 0 {
                break;
            }
            harness.advance_one(environment)?;
        }
        Ok(harness.replicate_result(seed, 0))
    }
}

impl<A: OrganismAdapter> EvolutionHarness<A> {
    pub fn new(adapter: A, protocol: crate::ExperimentProtocolV1) -> Result<Self, HarnessError> {
        protocol.validate().map_err(|e| HarnessError::Protocol(e.to_string()))?;
        Ok(Self {
            adapter,
            protocol,
            ledger: EventLedger::default(),
            population: PopulationManager::default(),
            lineage: LineageTracker::default(),
            generation: GenerationTracker::default(),
            organisms: BTreeMap::new(),
            accepted_step: 0,
            accepted_simulated_time: 0,
        })
    }

    pub fn initialize_founder(&mut self, founder: FounderIdentityV1) -> Result<u64, HarnessError> {
        if !self.organisms.is_empty() {
            return Err(HarnessError::FounderAlreadyInitialized);
        }
        let organism = self.adapter.initialize_founder(&founder)?;
        let organism_id = founder.founder_id;
        self.ledger
            .append(EventV1::founder(0, 0, 0, &self.protocol.environment_protocol.environment_id, &self.protocol.protocol_id, &organism_id.to_string()))
            .map_err(|e| HarnessError::Events(e.to_string()))?;
        let birth_event_id = self.ledger.events.last().map(|e| e.event_id).unwrap_or(1);
        self.population.create_founder(organism_id, organism_id, birth_event_id, 0).map_err(|e| HarnessError::Population(e.to_string()))?;
        self.population.mark_living(organism_id).map_err(|e| HarnessError::Population(e.to_string()))?;
        self.lineage.register_founder(organism_id, organism_id, birth_event_id, 0).map_err(|e| HarnessError::Lineage(e.to_string()))?;
        self.organisms.insert(organism_id, organism);
        Ok(organism_id)
    }

    pub fn advance_one(&mut self, environment: &EnvironmentProtocolV1) -> Result<(), HarnessError> {
        self.accepted_step += 1;
        self.accepted_simulated_time += 1;
        let ids = self.population.living_ids();
        for organism_id in ids {
            if let Some(organism) = self.organisms.get_mut(&organism_id) {
                if let Some(intervention) = self.adapter.apply_declared_environment(organism, environment, self.accepted_step)? {
                    let mut event = EventV1::base_for_harness(0, self.accepted_simulated_time, self.accepted_step, EventType::DamageApplied, environment, &self.protocol.protocol_id);
                    event.organism_id = Some(organism_id);
                    event.metadata.insert("intervention".into(), intervention);
                    self.ledger.append(event).map_err(|e| HarnessError::Events(e.to_string()))?;
                }
            }
            let outcome = {
                let organism = self.organisms.get_mut(&organism_id).ok_or(HarnessError::MissingOrganism(organism_id))?;
                self.adapter.advance(organism, environment, self.accepted_step, self.accepted_simulated_time)?
            };
            self.process_outcome(organism_id, outcome, environment)?;
        }
        Ok(())
    }

    fn process_outcome(&mut self, parent_id: u64, outcome: AdvanceOutcome<A::Organism>, environment: &EnvironmentProtocolV1) -> Result<(), HarnessError> {
        match outcome {
            AdvanceOutcome::Continuing => {
                if let Some(organism) = self.organisms.get(&parent_id) {
                    self.lineage.record_observation(parent_id, self.adapter.phenotype(organism), self.adapter.hereditary_state(organism)).map_err(|e| HarnessError::Lineage(e.to_string()))?;
                    if !self.adapter.is_alive(organism) {
                        self.record_death(parent_id, environment, "adapter_derived_dead")?;
                    }
                }
            }
            AdvanceOutcome::Died { reason } => self.record_death(parent_id, environment, &reason)?,
            AdvanceOutcome::Fission { offspring, metadata } => {
                if offspring.len() < 2 {
                    return Err(HarnessError::Adapter(AdapterError::Advance("fission must return at least two offspring".into())));
                }
                let parent_record = self.population.get(parent_id).ok_or(HarnessError::MissingOrganism(parent_id))?.clone();
                let mut started = EventV1::base_for_harness(0, self.accepted_simulated_time, self.accepted_step, EventType::FissionStarted, environment, &self.protocol.protocol_id);
                started.organism_id = Some(parent_id);
                started.parent_id = Some(parent_id);
                started.lineage_id = Some(parent_record.lineage_id);
                started.metadata = metadata.clone();
                self.ledger.append(started).map_err(|e| HarnessError::Events(e.to_string()))?;
                let mut completed = EventV1::base_for_harness(0, self.accepted_simulated_time, self.accepted_step, EventType::FissionCompleted, environment, &self.protocol.protocol_id);
                completed.organism_id = Some(parent_id);
                completed.lineage_id = Some(parent_record.lineage_id);
                completed.metadata = metadata;
                self.ledger.append(completed).map_err(|e| HarnessError::Events(e.to_string()))?;
                self.population.mark_dead(parent_id, self.accepted_simulated_time).map_err(|e| HarnessError::Population(e.to_string()))?;
                self.lineage.record_death(parent_id, self.accepted_simulated_time).map_err(|e| HarnessError::Lineage(e.to_string()))?;
                self.generation.record_completed_fission(parent_record.birth_generation + 1, self.accepted_simulated_time);
                let lineage_id = parent_record.lineage_id;
                for offspring in offspring {
                    let child_id = self.population.next_organism_id;
                    let mut birth = EventV1::birth(0, self.accepted_simulated_time, self.accepted_step, child_id, Some(parent_id), lineage_id, &environment.environment_id, &self.protocol.protocol_id);
                    birth.replicate = 0;
                    let birth_event_id = self.ledger.append(birth).map_err(|e| HarnessError::Events(e.to_string()))?;
                    self.population.register_offspring(child_id, parent_id, lineage_id, birth_event_id, self.accepted_simulated_time, parent_record.birth_generation + 1).map_err(|e| HarnessError::Population(e.to_string()))?;
                    self.population.mark_living(child_id).map_err(|e| HarnessError::Population(e.to_string()))?;
                    self.lineage.register_offspring(child_id, parent_id, lineage_id, birth_event_id, self.accepted_simulated_time).map_err(|e| HarnessError::Lineage(e.to_string()))?;
                    self.organisms.insert(child_id, offspring);
                }
            }
        }
        Ok(())
    }

    fn record_death(&mut self, organism_id: u64, environment: &EnvironmentProtocolV1, reason: &str) -> Result<(), HarnessError> {
        let event_id = self.ledger.append(EventV1::death(0, self.accepted_simulated_time, self.accepted_step, organism_id, 0, &environment.environment_id, &self.protocol.protocol_id)).map_err(|e| HarnessError::Events(e.to_string()))?;
        if let Some(event) = self.ledger.events.iter_mut().find(|event| event.event_id == event_id) {
            event.metadata.insert("reason".into(), reason.into());
        }
        self.population.mark_dead(organism_id, self.accepted_simulated_time).map_err(|e| HarnessError::Population(e.to_string()))?;
        self.lineage.record_death(organism_id, self.accepted_simulated_time).map_err(|e| HarnessError::Lineage(e.to_string()))?;
        Ok(())
    }

    pub fn replicate_result(&self, seed: u64, _replicate: u32) -> ReplicateResultV1 {
        let classification = if self.generation.max_generation == 0 {
            FailureClass::SelectionUntestableZeroGeneration
        } else if self.generation.max_generation < self.protocol.minimum_generation_requirement {
            FailureClass::InsufficientGenerations
        } else {
            FailureClass::ValidNoSelectionEffect
        };
        ReplicateResultV1 {
            schema: "ReplicateResultV1".into(),
            environment: self.protocol.environment_protocol.environment_id.clone(),
            seed,
            max_generation: self.generation.max_generation,
            birth_count: self.generation.completed_births,
            death_count: self.ledger.events.iter().filter(|event| event.event_type == EventType::Death).count() as u64,
            population_final: self.population.living_count() as u64,
            classification,
            protocol_hash: self.protocol.hash(),
            event_ledger_hash: self.ledger.hash().unwrap_or_default(),
        }
    }
}

impl EventV1 {
    pub(crate) fn base_for_harness(
        event_id: u64,
        time: u64,
        step: u64,
        event_type: EventType,
        environment: &EnvironmentProtocolV1,
        protocol_id: &str,
    ) -> Self {
        Self {
            schema: "EventV1".into(),
            event_id,
            accepted_simulated_time: time,
            accepted_step: step,
            replicate: 0,
            event_type,
            organism_id: None,
            parent_id: None,
            lineage_id: None,
            environment_id: environment.environment_id.clone(),
            protocol_id: protocol_id.into(),
            metadata: BTreeMap::new(),
        }
    }
}
