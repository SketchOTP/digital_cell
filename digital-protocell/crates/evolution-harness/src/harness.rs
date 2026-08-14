use crate::{
    AdapterError, AdvanceOutcome, EnvironmentCapability, EnvironmentContext, EnvironmentProtocolV1,
    EventLedger, EventType, EventV1, FailureClass, FounderIdentityV1, GenerationTracker,
    LineageTracker, MutationContext, OrganismAdapter, PopulationManager, ReplicateResultV1,
};
use std::collections::{BTreeMap, BTreeSet};
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
    #[error("adapter cannot execute required environment capability: {0:?}")]
    UnsupportedEnvironment(EnvironmentCapability),
    #[error("adapter returned invalid accepted dt {0}")]
    InvalidDt(f64),
    #[error("adapter accepted dt changed within a replicate: expected {expected}, got {actual}")]
    DtMismatch { expected: f64, actual: f64 },
    #[error("founder has already been initialized")]
    FounderAlreadyInitialized,
    #[error("organism {0} is missing from the adapter state")]
    MissingOrganism(u64),
}

pub struct EvolutionHarness<A: OrganismAdapter> {
    pub adapter: A,
    pub protocol: crate::ExperimentProtocolV1,
    pub replicate: u32,
    pub ledger: EventLedger,
    pub population: PopulationManager,
    pub lineage: LineageTracker,
    pub generation: GenerationTracker,
    pub organisms: BTreeMap<u64, A::Organism>,
    pub accepted_step: u64,
    pub accepted_simulated_time: f64,
    pub minimum_population_seen: u64,
    pub pressure_event_count: u64,
    pub first_pressure_time: Option<f64>,
    pub first_reproduction_time: Option<f64>,
    pub heredity_preserved: bool,
    pub phenotype_measurable: bool,
    pub environment_supported: bool,
    emitted_schedule_events: BTreeSet<String>,
    current_environment_id: String,
    active_environment: EnvironmentProtocolV1,
}

pub struct ReplicateRunner;

impl ReplicateRunner {
    pub fn validate_protocol(protocol: &crate::ExperimentProtocolV1) -> Result<(), HarnessError> {
        protocol.validate().map_err(|e| HarnessError::Protocol(e.to_string()))
    }

    pub fn run_campaign<A, AF, FF>(
        protocol: crate::ExperimentProtocolV1,
        mut adapter_factory: AF,
        mut founder_factory: FF,
    ) -> Result<crate::CampaignResultV1, HarnessError>
    where
        A: OrganismAdapter,
        AF: FnMut(u32, u64) -> A,
        FF: FnMut(u32, u64) -> FounderIdentityV1,
    {
        Self::validate_protocol(&protocol)?;
        let mut results = Vec::with_capacity(protocol.replicates as usize);
        for (replicate, seed) in protocol.random_seeds.iter().copied().enumerate() {
            let adapter = adapter_factory(replicate as u32, seed);
            let founder = founder_factory(replicate as u32, seed);
            let mut harness = EvolutionHarness::new(adapter, protocol.clone())?.with_replicate(replicate as u32);
            harness.initialize_founder(founder)?;
            harness.run_to_horizon()?;
            results.push(harness.replicate_result(seed));
        }
        let experiment_id = protocol.experiment_id.clone();
        let protocol_hash = protocol.hash();
        let control_signature = protocol.control_signature();
        Ok(crate::CampaignResultV1 {
            schema: "CampaignResultV1".into(),
            experiment_id,
            protocol_hash,
            control_signature,
            replicate_results: results,
        })
    }
}

impl<A: OrganismAdapter> EvolutionHarness<A> {
    pub fn new(adapter: A, protocol: crate::ExperimentProtocolV1) -> Result<Self, HarnessError> {
        protocol.validate().map_err(|e| HarnessError::Protocol(e.to_string()))?;
        let capabilities = adapter.environment_capabilities();
        for required in protocol.environment_protocol.required_capabilities() {
            if !capabilities.contains(&required) {
                return Err(HarnessError::UnsupportedEnvironment(required));
            }
        }
        Ok(Self {
            adapter,
            current_environment_id: protocol.environment_protocol.environment_id.clone(),
            protocol,
            replicate: 0,
            ledger: EventLedger::default(),
            population: PopulationManager::default(),
            lineage: LineageTracker::default(),
            generation: GenerationTracker::default(),
            organisms: BTreeMap::new(),
            accepted_step: 0,
            accepted_simulated_time: 0.0,
            minimum_population_seen: 0,
            pressure_event_count: 0,
            first_pressure_time: None,
            first_reproduction_time: None,
            heredity_preserved: false,
            phenotype_measurable: false,
            environment_supported: true,
            emitted_schedule_events: BTreeSet::new(),
            active_environment: protocol.environment_protocol.clone(),
        })
    }

    pub fn with_replicate(mut self, replicate: u32) -> Self { self.replicate = replicate; self }

    pub fn initialize_founder(&mut self, founder: FounderIdentityV1) -> Result<u64, HarnessError> {
        if !self.organisms.is_empty() { return Err(HarnessError::FounderAlreadyInitialized); }
        let organism = self.adapter.initialize_founder(&founder)?;
        let organism_id = founder.founder_id;
        let event_id = self.ledger.append(EventV1::founder(0, 0.0, 0, self.replicate, &self.current_environment_id, &self.protocol.protocol_id, organism_id)).map_err(|e| HarnessError::Events(e.to_string()))?;
        self.population.create_founder(organism_id, organism_id, event_id, 0.0).map_err(|e| HarnessError::Population(e.to_string()))?;
        self.population.mark_living(organism_id).map_err(|e| HarnessError::Population(e.to_string()))?;
        self.lineage.register_founder(organism_id, organism_id, event_id, 0.0).map_err(|e| HarnessError::Lineage(e.to_string()))?;
        let phenotype = self.adapter.phenotype(&organism);
        let hereditary = self.adapter.hereditary_state(&organism);
        self.phenotype_measurable = !phenotype.is_empty();
        self.heredity_preserved = !hereditary.is_empty();
        self.lineage.record_observation(organism_id, phenotype, hereditary).map_err(|e| HarnessError::Lineage(e.to_string()))?;
        self.generation.record_founder(0.0);
        self.organisms.insert(organism_id, organism);
        self.minimum_population_seen = 1;
        Ok(organism_id)
    }

    fn emit_global_environment_events(&mut self) -> Result<(), HarnessError> {
        let env = self.active_environment.clone();
        if env.resource_mode == crate::ResourceMode::Continuous && self.accepted_simulated_time == 0.0 {
            self.append_environment_event(EventType::ResourceContinuous, "continuous_resource_flow")?;
        }
        for time in env.pulse_schedule.iter().chain(env.resource_ecology.pulse_schedule.iter()) {
            if *time <= self.accepted_simulated_time + f64::EPSILON {
                self.emit_once(EventType::ResourcePulse, *time, "resource_pulse")?;
            }
        }
        for window in env.scarcity_schedule.iter().chain(env.resource_ecology.scarcity_schedule.iter()) {
            if window.start <= self.accepted_simulated_time + f64::EPSILON {
                self.emit_once(EventType::ScarcityStarted, window.start, "scarcity_start")?;
            }
            if window.end <= self.accepted_simulated_time + f64::EPSILON {
                self.emit_once(EventType::ScarcityEnded, window.end, "scarcity_end")?;
            }
        }
        for transition in &env.transitions {
            if transition.time <= self.accepted_simulated_time + f64::EPSILON {
                let key = format!("transition:{}", transition.time);
                if self.emitted_schedule_events.insert(key) {
                    let mut event = EventV1::base(0, self.accepted_simulated_time, self.accepted_step, self.replicate, EventType::EnvironmentSwitch, &transition.environment_id, &self.protocol.protocol_id);
                    event.metadata.insert("label".into(), transition.label.clone());
                    event.metadata.insert("scheduled_time".into(), transition.time.to_string());
                    event.metadata.insert("from_environment".into(), self.current_environment_id.clone());
                    self.ledger.append(event).map_err(|e| HarnessError::Events(e.to_string()))?;
                    self.current_environment_id = transition.environment_id.clone();
                    self.active_environment.environment_id = transition.environment_id.clone();
                    if let Some(resource_mode) = &transition.resource_mode {
                        self.active_environment.resource_mode = resource_mode.clone();
                    }
                }
            }
        }
        Ok(())
    }

    fn emit_once(&mut self, event_type: EventType, time: f64, label: &str) -> Result<(), HarnessError> {
        let key = format!("{label}:{time}");
        if !self.emitted_schedule_events.insert(key) { return Ok(()); }
        self.append_environment_event_at(event_type, self.accepted_simulated_time, &format!("{label};scheduled_time={time}"))
    }

    fn append_environment_event(&mut self, event_type: EventType, label: &str) -> Result<(), HarnessError> {
        self.append_environment_event_at(event_type, self.accepted_simulated_time, label)
    }

    fn append_environment_event_at(&mut self, event_type: EventType, time: f64, label: &str) -> Result<(), HarnessError> {
        let mut event = EventV1::base(0, time, self.accepted_step, self.replicate, event_type.clone(), &self.current_environment_id, &self.protocol.protocol_id);
        event.metadata.insert("source".into(), label.into());
        self.ledger.append(event).map_err(|e| HarnessError::Events(e.to_string()))?;
        if matches!(event_type, EventType::ResourcePulse | EventType::ScarcityStarted | EventType::ScarcityEnded | EventType::DamageApplied) {
            self.pressure_event_count += 1;
            self.first_pressure_time.get_or_insert(time);
        }
        Ok(())
    }

    pub fn advance_one(&mut self) -> Result<(), HarnessError> {
        self.emit_global_environment_events()?;
        let expected_dt = self.adapter.accepted_dt();
        if !expected_dt.is_finite() || expected_dt <= 0.0 { return Err(HarnessError::InvalidDt(expected_dt)); }
        let step_start = self.accepted_simulated_time;
        let step_end = step_start + expected_dt;
        self.accepted_step += 1;
        let ids = self.population.living_ids();
        let living_population = ids.len();
        for (organism_index, organism_id) in ids.into_iter().enumerate() {
            let environment = self.active_environment.clone();
            if let Some(organism) = self.organisms.get_mut(&organism_id) {
                let events = self.adapter.apply_declared_environment(organism, &environment, self.accepted_step, step_start, EnvironmentContext { living_population, organism_index, accepted_dt: expected_dt })?;
                for adapter_event in events {
                    let mut event = EventV1::base(0, step_start, self.accepted_step, self.replicate, adapter_event.event_type.clone(), &self.current_environment_id, &self.protocol.protocol_id);
                    event.organism_id = Some(organism_id);
                    event.metadata = adapter_event.metadata;
                    self.ledger.append(event).map_err(|e| HarnessError::Events(e.to_string()))?;
                    if matches!(adapter_event.event_type, EventType::DamageApplied | EventType::ResourcePulse | EventType::ScarcityStarted | EventType::ScarcityEnded) {
                        self.pressure_event_count += 1;
                        self.first_pressure_time.get_or_insert(step_start);
                    }
                }
            }
            let outcome = {
                let organism = self.organisms.get_mut(&organism_id).ok_or(HarnessError::MissingOrganism(organism_id))?;
                self.adapter.advance(organism, &environment, self.accepted_step, step_start)?
            };
            let actual_dt = outcome.accepted_dt();
            if (actual_dt - expected_dt).abs() > f64::EPSILON.max(expected_dt.abs() * 1e-12) {
                return Err(HarnessError::DtMismatch { expected: expected_dt, actual: actual_dt });
            }
            self.accepted_simulated_time = step_end;
            self.process_outcome(organism_id, outcome, &environment)?;
        }
        self.minimum_population_seen = if self.minimum_population_seen == 0 { self.population.living_count() as u64 } else { self.minimum_population_seen.min(self.population.living_count() as u64) };
        Ok(())
    }

    fn process_outcome(&mut self, parent_id: u64, outcome: AdvanceOutcome<A::Organism>, environment: &EnvironmentProtocolV1) -> Result<(), HarnessError> {
        match outcome {
            AdvanceOutcome::Continuing { .. } => {
                if let Some(organism) = self.organisms.get(&parent_id) {
                    self.lineage.record_observation(parent_id, self.adapter.phenotype(organism), self.adapter.hereditary_state(organism)).map_err(|e| HarnessError::Lineage(e.to_string()))?;
                    if !self.adapter.is_alive(organism) { self.record_death(parent_id, environment, "adapter_derived_dead")?; }
                }
            }
            AdvanceOutcome::Died { reason, .. } => self.record_death(parent_id, environment, &reason)?,
            AdvanceOutcome::Fission { offspring, metadata, .. } => {
                if offspring.len() < 2 { return Err(HarnessError::Adapter(AdapterError::Advance("fission must return at least two offspring".into()))); }
                let parent_record = self.population.get(parent_id).ok_or(HarnessError::MissingOrganism(parent_id))?.clone();
                let mut started = EventV1::base(0, self.accepted_simulated_time, self.accepted_step, self.replicate, EventType::FissionStarted, &self.current_environment_id, &self.protocol.protocol_id);
                started.organism_id = Some(parent_id); started.parent_id = Some(parent_id); started.lineage_id = Some(parent_record.lineage_id); started.metadata = metadata.clone();
                self.ledger.append(started).map_err(|e| HarnessError::Events(e.to_string()))?;
                let mut completed = EventV1::base(0, self.accepted_simulated_time, self.accepted_step, self.replicate, EventType::FissionCompleted, &self.current_environment_id, &self.protocol.protocol_id);
                completed.organism_id = Some(parent_id); completed.lineage_id = Some(parent_record.lineage_id); completed.metadata = metadata;
                self.ledger.append(completed).map_err(|e| HarnessError::Events(e.to_string()))?;
                self.population.mark_dead(parent_id, self.accepted_simulated_time).map_err(|e| HarnessError::Population(e.to_string()))?;
                self.lineage.record_death(parent_id, self.accepted_simulated_time).map_err(|e| HarnessError::Lineage(e.to_string()))?;
                self.generation.record_completed_fission(parent_record.birth_generation + 1, self.accepted_simulated_time);
                self.first_reproduction_time.get_or_insert(self.accepted_simulated_time);
                let lineage_id = parent_record.lineage_id;
                for mut child in offspring {
                    let child_id = self.population.next_organism_id;
                    let birth = EventV1::birth(0, self.accepted_simulated_time, self.accepted_step, self.replicate, child_id, Some(parent_id), lineage_id, &self.current_environment_id, &self.protocol.protocol_id);
                    let birth_event_id = self.ledger.append(birth).map_err(|e| HarnessError::Events(e.to_string()))?;
                    self.population.register_offspring(child_id, parent_id, lineage_id, birth_event_id, self.accepted_simulated_time, parent_record.birth_generation + 1).map_err(|e| HarnessError::Population(e.to_string()))?;
                    self.population.mark_living(child_id).map_err(|e| HarnessError::Population(e.to_string()))?;
                    self.lineage.register_offspring(child_id, parent_id, lineage_id, birth_event_id, self.accepted_simulated_time).map_err(|e| HarnessError::Lineage(e.to_string()))?;
                    let parent_state = self.organisms.get(&parent_id).map(|parent| self.adapter.hereditary_state(parent)).unwrap_or_default();
                    let parent = self.organisms.get(&parent_id).ok_or(HarnessError::MissingOrganism(parent_id))?;
                    let mutation = self.adapter.apply_heredity_and_mutation(parent, &mut child, &self.protocol.mutation_protocol, &MutationContext { accepted_step: self.accepted_step, accepted_simulated_time: self.accepted_simulated_time, seed: child_id, parent_hereditary_state: parent_state })?;
                    if let Some(metadata) = mutation {
                        let mut event = EventV1::base(0, self.accepted_simulated_time, self.accepted_step, self.replicate, EventType::Mutation, &self.current_environment_id, &self.protocol.protocol_id);
                        event.organism_id = Some(child_id); event.parent_id = Some(parent_id); event.lineage_id = Some(lineage_id); event.metadata = metadata;
                        self.ledger.append(event).map_err(|e| HarnessError::Events(e.to_string()))?;
                    }
                    self.heredity_preserved &= !self.adapter.hereditary_state(&child).is_empty();
                    self.phenotype_measurable &= !self.adapter.phenotype(&child).is_empty();
                    self.organisms.insert(child_id, child);
                }
            }
        }
        Ok(())
    }

    fn record_death(&mut self, organism_id: u64, environment: &EnvironmentProtocolV1, reason: &str) -> Result<(), HarnessError> {
        let mut event = EventV1::death(0, self.accepted_simulated_time, self.accepted_step, self.replicate, organism_id, &self.current_environment_id, &self.protocol.protocol_id);
        event.metadata.insert("reason".into(), reason.into());
        self.ledger.append(event).map_err(|e| HarnessError::Events(e.to_string()))?;
        self.population.mark_dead(organism_id, self.accepted_simulated_time).map_err(|e| HarnessError::Population(e.to_string()))?;
        self.lineage.record_death(organism_id, self.accepted_simulated_time).map_err(|e| HarnessError::Lineage(e.to_string()))?;
        let _ = environment;
        Ok(())
    }

    pub fn run_to_horizon(&mut self) -> Result<(), HarnessError> {
        while self.accepted_simulated_time < self.protocol.maximum_accepted_horizon && self.population.living_count() > 0 {
            self.advance_one()?;
        }
        let end = EventV1::experiment_end(0, self.accepted_simulated_time, self.accepted_step, self.replicate, &self.current_environment_id, &self.protocol.protocol_id);
        self.ledger.append(end).map_err(|e| HarnessError::Events(e.to_string()))?;
        Ok(())
    }

    pub fn replicate_result(&self, seed: u64) -> ReplicateResultV1 {
        let event_ledger_valid = self.ledger.validate().is_ok();
        let actual_reproduction = self.generation.completed_fissions > 0;
        let pressure_before_reproduction = self.first_pressure_time.zip(self.first_reproduction_time).map(|(pressure, reproduction)| pressure < reproduction).unwrap_or(false);
        let classification = if !event_ledger_valid { FailureClass::EventLedgerIncomplete }
        else if !self.environment_supported { FailureClass::EnvironmentUnsupported }
        else if self.generation.max_generation == 0 { FailureClass::SelectionUntestableZeroGeneration }
        else if self.generation.max_generation < self.protocol.minimum_generation_requirement { FailureClass::InsufficientGenerations }
        else if !actual_reproduction { FailureClass::NoReproduction }
        else if self.pressure_event_count == 0 { FailureClass::EcologyPressureNotReached }
        else if !pressure_before_reproduction { FailureClass::EcologyPressurePostReproduction }
        else if self.minimum_population_seen < self.protocol.minimum_population_viability { FailureClass::PopulationCollapsePreSelection }
        else if !self.heredity_preserved { FailureClass::HeredityNotPreserved }
        else if !self.phenotype_measurable { FailureClass::PhenotypeNotExpressed }
        else { FailureClass::ReplicateQualified };
        ReplicateResultV1 {
            schema: "ReplicateResultV1".into(), experiment_id: self.protocol.experiment_id.clone(), environment: self.current_environment_id.clone(), replicate: self.replicate,
            seed, max_generation: self.generation.max_generation, birth_count: self.generation.completed_births,
            death_count: self.ledger.events.iter().filter(|event| matches!(event.event_type, EventType::Death)).count() as u64,
            population_final: self.population.living_count() as u64, minimum_population_seen: self.minimum_population_seen,
            accepted_simulated_time: self.accepted_simulated_time, pressure_event_count: self.pressure_event_count,
            pressure_before_reproduction, actual_reproduction, heredity_preserved: self.heredity_preserved,
            phenotype_measurable: self.phenotype_measurable, event_ledger_valid, environment_supported: self.environment_supported,
            neutral_comparator_valid: false, classification, protocol_hash: self.protocol.hash(), event_ledger_hash: self.ledger.hash().unwrap_or_default(),
        }
    }
}
