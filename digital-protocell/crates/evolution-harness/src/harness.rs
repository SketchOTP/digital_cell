use crate::{
    AdapterError, AdvanceOutcome, CampaignRole, EnvironmentCapability, EnvironmentContext,
    EnvironmentProtocolV1, EventLedger, EventType, EventV1, FailureClass, FounderIdentityV1,
    GenerationTracker, LineageTracker, MutationContext, OrganismAdapter, PopulationManager,
    ReplicateResultV1,
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
    #[error("population initialization failed: {0}")]
    PopulationInitialization(String),
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
    pub pressure_reached: bool,
    pub pressure_contract_valid: bool,
    pub first_pressure_time: Option<f64>,
    pub first_reproduction_time: Option<f64>,
    pub heredity_preserved: bool,
    pub phenotype_measurable: bool,
    pub environment_supported: bool,
    emitted_schedule_events: BTreeSet<String>,
    current_environment_id: String,
    active_environment: EnvironmentProtocolV1,
    campaign_seed: u64,
    mutation_stream_counter: u64,
}

pub struct ReplicateRunner;

impl ReplicateRunner {
    pub fn validate_protocol(protocol: &crate::ExperimentProtocolV1) -> Result<(), HarnessError> {
        protocol
            .validate_for_execution()
            .map_err(|e| HarnessError::Protocol(e.to_string()))
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
        protocol
            .validate_for_execution()
            .map_err(|e| HarnessError::Protocol(e.to_string()))?;
        let mut results = Vec::with_capacity(protocol.replicates as usize);
        for (replicate, seed) in protocol.random_seeds.iter().copied().enumerate() {
            let adapter = adapter_factory(replicate as u32, seed);
            let founder = founder_factory(replicate as u32, seed);
            let mut harness =
                EvolutionHarness::new(adapter, protocol.clone())?
                    .with_replicate_seed(replicate as u32, seed);
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
            selective_pressure: protocol.selective_pressure.clone(),
            replicate_results: results,
        })
    }
}

impl<A: OrganismAdapter> EvolutionHarness<A> {
    pub fn new(adapter: A, protocol: crate::ExperimentProtocolV1) -> Result<Self, HarnessError> {
        protocol
            .validate_for_execution()
            .map_err(|e| HarnessError::Protocol(e.to_string()))?;
        let capabilities = adapter.environment_capabilities();
        for required in protocol.environment_protocol.required_capabilities() {
            if !capabilities.contains(&required) {
                return Err(HarnessError::UnsupportedEnvironment(required));
            }
        }
        let active_environment = protocol.environment_protocol.clone();
        let pressure_contract_valid = protocol
            .selective_pressure
            .as_ref()
            .map(|contract| contract.validate().is_ok())
            .unwrap_or(false);
        let default_campaign_seed = protocol
            .placement_protocol
            .random_seed
            .unwrap_or(protocol.placement_protocol.founder_seed);
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
            pressure_reached: false,
            pressure_contract_valid,
            first_pressure_time: None,
            first_reproduction_time: None,
            heredity_preserved: false,
            phenotype_measurable: false,
            environment_supported: true,
            emitted_schedule_events: BTreeSet::new(),
            active_environment,
            campaign_seed: default_campaign_seed,
            mutation_stream_counter: 0,
        })
    }

    pub fn with_replicate(mut self, replicate: u32) -> Self {
        self.replicate = replicate;
        self
    }

    pub fn with_replicate_seed(mut self, replicate: u32, seed: u64) -> Self {
        self.replicate = replicate;
        self.campaign_seed = seed;
        self
    }

    pub fn initialize_founder(&mut self, founder: FounderIdentityV1) -> Result<u64, HarnessError> {
        let mut ids = self.initialize_founders(vec![founder])?;
        ids.pop().ok_or_else(|| {
            HarnessError::PopulationInitialization(
                "founder initializer returned no organism".into(),
            )
        })
    }

    pub fn initialize_founders(
        &mut self,
        founders: Vec<FounderIdentityV1>,
    ) -> Result<Vec<u64>, HarnessError> {
        if !self.organisms.is_empty() {
            return Err(HarnessError::FounderAlreadyInitialized);
        }
        if founders.is_empty() {
            return Err(HarnessError::PopulationInitialization(
                "at least one founder is required".into(),
            ));
        }
        let placements = &self.protocol.placement_protocol.initial_coordinates;
        if placements.len() != founders.len() {
            return Err(HarnessError::PopulationInitialization(format!(
                "placement count {} does not match founder count {}",
                placements.len(),
                founders.len()
            )));
        }
        let population_size = founders.len();
        let mut seen = BTreeSet::new();
        let mut founder_ids = Vec::with_capacity(founders.len());
        let mut phenotype_ok = true;
        for (founder_index, founder) in founders.into_iter().enumerate() {
            if !seen.insert(founder.founder_id) {
                return Err(HarnessError::PopulationInitialization(format!(
                    "duplicate founder id {}",
                    founder.founder_id
                )));
            }
            let placement = placements[founder_index];
            let organism = self.adapter.initialize_founder(
                &founder,
                crate::FounderInitializationContext {
                    replicate: self.replicate,
                    founder_index,
                    population_size,
                    placement,
                },
            )?;
            let organism_id = founder.founder_id;
            let mut event = EventV1::founder(
                0,
                0.0,
                0,
                self.replicate,
                &self.current_environment_id,
                &self.protocol.protocol_id,
                organism_id,
            );
            event
                .metadata
                .insert("placement_x".into(), placement[0].to_string());
            event
                .metadata
                .insert("placement_y".into(), placement[1].to_string());
            let event_id = self
                .ledger
                .append(event)
                .map_err(|e| HarnessError::Events(e.to_string()))?;
            self.population
                .create_founder(organism_id, organism_id, event_id, 0.0, placement)
                .map_err(|e| HarnessError::Population(e.to_string()))?;
            self.population
                .mark_living(organism_id)
                .map_err(|e| HarnessError::Population(e.to_string()))?;
            self.lineage
                .register_founder(organism_id, organism_id, event_id, 0.0)
                .map_err(|e| HarnessError::Lineage(e.to_string()))?;
            let phenotype = self.adapter.phenotype(&organism);
            let hereditary = self.adapter.hereditary_state(&organism);
            let phenotype_evidence = self
                .adapter
                .phenotype_evidence(&self.active_environment, &organism);
            phenotype_ok &= phenotype_evidence.qualification;
            self.lineage
                .record_observation(organism_id, phenotype, hereditary)
                .map_err(|e| HarnessError::Lineage(e.to_string()))?;
            self.organisms.insert(organism_id, organism);
            founder_ids.push(organism_id);
        }
        self.phenotype_measurable = phenotype_ok;
        self.heredity_preserved = true;
        self.generation.record_founder(0.0);
        self.minimum_population_seen = founder_ids.len() as u64;
        Ok(founder_ids)
    }

    fn emit_global_environment_events(&mut self) -> Result<(), HarnessError> {
        let env = self.active_environment.clone();
        if env.resource_mode == crate::ResourceMode::Continuous
            && self.accepted_simulated_time == 0.0
        {
            self.append_environment_event(
                EventType::ResourceContinuous,
                "continuous_resource_flow",
            )?;
        }
        for time in env
            .pulse_schedule
            .iter()
            .chain(env.resource_ecology.pulse_schedule.iter())
        {
            if *time <= self.accepted_simulated_time + f64::EPSILON {
                self.emit_once(EventType::ResourcePulse, *time, "resource_pulse")?;
            }
        }
        for window in env
            .scarcity_schedule
            .iter()
            .chain(env.resource_ecology.scarcity_schedule.iter())
        {
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
                    let mut event = EventV1::base(
                        0,
                        self.accepted_simulated_time,
                        self.accepted_step,
                        self.replicate,
                        EventType::EnvironmentSwitch,
                        &transition.environment_id,
                        &self.protocol.protocol_id,
                    );
                    event
                        .metadata
                        .insert("label".into(), transition.label.clone());
                    event
                        .metadata
                        .insert("scheduled_time".into(), transition.time.to_string());
                    event.metadata.insert(
                        "from_environment".into(),
                        self.current_environment_id.clone(),
                    );
                    self.ledger
                        .append(event)
                        .map_err(|e| HarnessError::Events(e.to_string()))?;
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

    fn emit_once(
        &mut self,
        event_type: EventType,
        time: f64,
        label: &str,
    ) -> Result<(), HarnessError> {
        let key = format!("{label}:{time}");
        if !self.emitted_schedule_events.insert(key) {
            return Ok(());
        }
        self.append_environment_event_at(
            event_type,
            self.accepted_simulated_time,
            &format!("{label};scheduled_time={time}"),
        )
    }

    fn declared_pressure_name(event_type: &EventType) -> Option<&'static str> {
        match event_type {
            EventType::ResourcePulse => Some("resource_pulse"),
            EventType::ScarcityStarted => Some("scarcity_start"),
            EventType::DamageApplied => Some("damage"),
            EventType::EnvironmentSwitch => Some("environment_switch"),
            _ => None,
        }
    }

    fn record_declared_pressure(
        &mut self,
        event_type: &EventType,
        environment_id: &str,
        time: f64,
    ) {
        let Some(contract) = &self.protocol.selective_pressure else {
            return;
        };
        let Some(event_name) = Self::declared_pressure_name(event_type) else {
            return;
        };
        if contract.campaign_role == CampaignRole::Treatment
            && environment_id == contract.treatment_environment
            && event_name == contract.pressure_event_or_condition
            && time + f64::EPSILON >= contract.pressure_start
        {
            self.pressure_event_count += 1;
            self.pressure_reached = true;
            self.first_pressure_time.get_or_insert(time);
        }
    }

    fn append_environment_event(
        &mut self,
        event_type: EventType,
        label: &str,
    ) -> Result<(), HarnessError> {
        self.append_environment_event_at(event_type, self.accepted_simulated_time, label)
    }

    fn append_environment_event_at(
        &mut self,
        event_type: EventType,
        time: f64,
        label: &str,
    ) -> Result<(), HarnessError> {
        let mut event = EventV1::base(
            0,
            time,
            self.accepted_step,
            self.replicate,
            event_type.clone(),
            &self.current_environment_id,
            &self.protocol.protocol_id,
        );
        event.metadata.insert("source".into(), label.into());
        self.ledger
            .append(event)
            .map_err(|e| HarnessError::Events(e.to_string()))?;
        self.record_declared_pressure(&event_type, &self.current_environment_id.clone(), time);
        Ok(())
    }

    pub fn advance_one(&mut self) -> Result<(), HarnessError> {
        self.emit_global_environment_events()?;
        let expected_dt = self.adapter.accepted_dt();
        if !expected_dt.is_finite() || expected_dt <= 0.0 {
            return Err(HarnessError::InvalidDt(expected_dt));
        }
        let step_start = self.accepted_simulated_time;
        let step_end = step_start + expected_dt;
        self.accepted_step += 1;
        let ids = self.population.living_ids();
        let living_population = ids.len();
        let environment = self.active_environment.clone();

        // Phase 3: apply all start-of-step interventions without appending yet.
        let mut intervention_events = Vec::new();
        for (organism_index, organism_id) in ids.iter().copied().enumerate() {
            if let Some(organism) = self.organisms.get_mut(&organism_id) {
                let events = self.adapter.apply_declared_environment(
                    organism,
                    &environment,
                    self.accepted_step,
                    step_start,
                    EnvironmentContext {
                        living_population,
                        organism_index,
                        accepted_dt: expected_dt,
                    },
                )?;
                for adapter_event in events {
                    intervention_events.push((organism_id, adapter_event));
                }
            }
        }

        // Phase 4: append every intervention at step_start before any step_end event.
        for (organism_id, adapter_event) in intervention_events {
            let event_type = adapter_event.event_type.clone();
            let environment_id = self.current_environment_id.clone();
            let mut event = EventV1::base(
                0,
                step_start,
                self.accepted_step,
                self.replicate,
                event_type.clone(),
                &environment_id,
                &self.protocol.protocol_id,
            );
            event.organism_id = Some(organism_id);
            event.metadata = adapter_event.metadata;
            self.ledger
                .append(event)
                .map_err(|e| HarnessError::Events(e.to_string()))?;
            self.record_declared_pressure(&event_type, &environment_id, step_start);
        }

        // Phase 5: advance every organism through the same accepted dt, collecting outcomes.
        let mut outcomes = Vec::with_capacity(ids.len());
        for organism_id in ids {
            let outcome = {
                let organism = self
                    .organisms
                    .get_mut(&organism_id)
                    .ok_or(HarnessError::MissingOrganism(organism_id))?;
                self.adapter
                    .advance(organism, &environment, self.accepted_step, step_start)?
            };
            let actual_dt = outcome.accepted_dt();
            if (actual_dt - expected_dt).abs() > f64::EPSILON.max(expected_dt.abs() * 1e-12) {
                return Err(HarnessError::DtMismatch {
                    expected: expected_dt,
                    actual: actual_dt,
                });
            }
            outcomes.push((organism_id, outcome));
        }

        // Phase 6: accept the common step end once every organism has advanced.
        self.accepted_simulated_time = step_end;

        // Phase 7: process all deaths, fissions, offspring, heredity, and mutation at step_end.
        for (organism_id, outcome) in outcomes {
            self.process_outcome(organism_id, outcome, &environment)?;
        }
        // Phase 8: fail closed if any chronology or population invariant was violated.
        self.ledger
            .validate()
            .map_err(|e| HarnessError::Events(e.to_string()))?;
        self.minimum_population_seen = if self.minimum_population_seen == 0 {
            self.population.living_count() as u64
        } else {
            self.minimum_population_seen
                .min(self.population.living_count() as u64)
        };
        Ok(())
    }

    fn process_outcome(
        &mut self,
        parent_id: u64,
        outcome: AdvanceOutcome<A::Organism>,
        environment: &EnvironmentProtocolV1,
    ) -> Result<(), HarnessError> {
        match outcome {
            AdvanceOutcome::Continuing { .. } => {
                if let Some(organism) = self.organisms.get(&parent_id) {
                    self.lineage
                        .record_observation(
                            parent_id,
                            self.adapter.phenotype(organism),
                            self.adapter.hereditary_state(organism),
                        )
                        .map_err(|e| HarnessError::Lineage(e.to_string()))?;
                    if !self.adapter.is_alive(organism) {
                        self.record_death(parent_id, environment, "adapter_derived_dead")?;
                    }
                }
            }
            AdvanceOutcome::Died { reason, .. } => {
                self.record_death(parent_id, environment, &reason)?
            }
            AdvanceOutcome::Fission {
                offspring,
                metadata,
                ..
            } => {
                if offspring.len() < 2 {
                    return Err(HarnessError::Adapter(AdapterError::Advance(
                        "fission must return at least two offspring".into(),
                    )));
                }
                let parent_record = self
                    .population
                    .get(parent_id)
                    .ok_or(HarnessError::MissingOrganism(parent_id))?
                    .clone();
                let mut started = EventV1::base(
                    0,
                    self.accepted_simulated_time,
                    self.accepted_step,
                    self.replicate,
                    EventType::FissionStarted,
                    &self.current_environment_id,
                    &self.protocol.protocol_id,
                );
                started.organism_id = Some(parent_id);
                started.parent_id = Some(parent_id);
                started.lineage_id = Some(parent_record.lineage_id);
                started.metadata = metadata.clone();
                self.ledger
                    .append(started)
                    .map_err(|e| HarnessError::Events(e.to_string()))?;
                let mut completed = EventV1::base(
                    0,
                    self.accepted_simulated_time,
                    self.accepted_step,
                    self.replicate,
                    EventType::FissionCompleted,
                    &self.current_environment_id,
                    &self.protocol.protocol_id,
                );
                completed.organism_id = Some(parent_id);
                completed.lineage_id = Some(parent_record.lineage_id);
                completed.metadata = metadata;
                self.ledger
                    .append(completed)
                    .map_err(|e| HarnessError::Events(e.to_string()))?;
                self.population
                    .mark_dead(parent_id, self.accepted_simulated_time)
                    .map_err(|e| HarnessError::Population(e.to_string()))?;
                self.lineage
                    .record_death(parent_id, self.accepted_simulated_time)
                    .map_err(|e| HarnessError::Lineage(e.to_string()))?;
                self.generation.record_completed_fission(
                    parent_id,
                    parent_record.birth_generation,
                    parent_record.birth_time,
                    self.accepted_simulated_time,
                );
                self.first_reproduction_time
                    .get_or_insert(self.accepted_simulated_time);
                let lineage_id = parent_record.lineage_id;
                for (offspring_index, mut child) in offspring.into_iter().enumerate() {
                    let qualified_copy_ordinal = self.mutation_stream_counter;
                    self.mutation_stream_counter = self.mutation_stream_counter.wrapping_add(1);
                    let mutation_stream_seed =
                        d096_mutation_stream_seed(self.campaign_seed, qualified_copy_ordinal);
                    let child_id = self.population.next_organism_id;
                    let mut birth = EventV1::birth(
                        0,
                        self.accepted_simulated_time,
                        self.accepted_step,
                        self.replicate,
                        child_id,
                        Some(parent_id),
                        lineage_id,
                        &self.current_environment_id,
                        &self.protocol.protocol_id,
                    );
                    birth.metadata.insert(
                        "qualified_physical_copy".into(),
                        "true".into(),
                    );
                    birth.metadata.insert(
                        "qualified_copy_ordinal".into(),
                        qualified_copy_ordinal.to_string(),
                    );
                    birth.metadata.insert(
                        "mutation_stream_seed".into(),
                        mutation_stream_seed.to_string(),
                    );
                    let birth_event_id = self
                        .ledger
                        .append(birth)
                        .map_err(|e| HarnessError::Events(e.to_string()))?;
                    self.population
                        .register_offspring(
                            child_id,
                            parent_id,
                            lineage_id,
                            birth_event_id,
                            self.accepted_simulated_time,
                            parent_record.birth_generation + 1,
                            parent_record.placement,
                        )
                        .map_err(|e| HarnessError::Population(e.to_string()))?;
                    self.population
                        .mark_living(child_id)
                        .map_err(|e| HarnessError::Population(e.to_string()))?;
                    self.lineage
                        .register_offspring(
                            child_id,
                            parent_id,
                            lineage_id,
                            birth_event_id,
                            self.accepted_simulated_time,
                        )
                        .map_err(|e| HarnessError::Lineage(e.to_string()))?;
                    let parent_state = self
                        .organisms
                        .get(&parent_id)
                        .map(|parent| self.adapter.hereditary_state(parent))
                        .unwrap_or_default();
                    let parent = self
                        .organisms
                        .get(&parent_id)
                        .ok_or(HarnessError::MissingOrganism(parent_id))?;
                    let mutation = self.adapter.apply_heredity_and_mutation(
                        parent,
                        &mut child,
                        &self.protocol.mutation_protocol,
                        &MutationContext {
                            accepted_step: self.accepted_step,
                            accepted_simulated_time: self.accepted_simulated_time,
                            // Mutation randomness is declared by the protocol
                            // and a unique accepted-copy ordinal. It is not
                            // derived from organism, lineage, clade, fitness,
                            // or survival identity.
                            seed: mutation_stream_seed,
                            offspring_index: offspring_index as u32,
                            qualified_physical_copy: true,
                            qualified_copy_ordinal,
                            parent_hereditary_state: parent_state,
                        },
                    )?;
                    let mutation_provenance_valid = if self
                        .protocol
                        .mutation_protocol
                        .mutation_protocol_id
                        == "d096_allocation_mutation_v1"
                    {
                        mutation.as_ref().is_some_and(valid_d096_mutation_metadata)
                    } else {
                        true
                    };
                    if let Some(metadata) = mutation {
                        if mutation_metadata_reports_change(&metadata) {
                            let mut event = EventV1::base(
                                0,
                                self.accepted_simulated_time,
                                self.accepted_step,
                                self.replicate,
                                EventType::Mutation,
                                &self.current_environment_id,
                                &self.protocol.protocol_id,
                            );
                            event.organism_id = Some(child_id);
                            event.parent_id = Some(parent_id);
                            event.lineage_id = Some(lineage_id);
                            event.metadata = metadata;
                            self.ledger
                                .append(event)
                                .map_err(|e| HarnessError::Events(e.to_string()))?;
                        }
                    }
                    let heredity_evidence = self.adapter.heredity_evidence(Some(parent), &child);
                    let phenotype_evidence = self.adapter.phenotype_evidence(environment, &child);
                    self.heredity_preserved &=
                        heredity_evidence.qualification && mutation_provenance_valid;
                    self.phenotype_measurable &= phenotype_evidence.qualification;
                    self.organisms.insert(child_id, child);
                }
            }
        }
        Ok(())
    }

    fn record_death(
        &mut self,
        organism_id: u64,
        environment: &EnvironmentProtocolV1,
        reason: &str,
    ) -> Result<(), HarnessError> {
        let mut event = EventV1::death(
            0,
            self.accepted_simulated_time,
            self.accepted_step,
            self.replicate,
            organism_id,
            &self.current_environment_id,
            &self.protocol.protocol_id,
        );
        event.metadata.insert("reason".into(), reason.into());
        self.ledger
            .append(event)
            .map_err(|e| HarnessError::Events(e.to_string()))?;
        self.population
            .mark_dead(organism_id, self.accepted_simulated_time)
            .map_err(|e| HarnessError::Population(e.to_string()))?;
        self.lineage
            .record_death(organism_id, self.accepted_simulated_time)
            .map_err(|e| HarnessError::Lineage(e.to_string()))?;
        let _ = environment;
        Ok(())
    }

    pub fn run_to_horizon(&mut self) -> Result<(), HarnessError> {
        while self.accepted_simulated_time < self.protocol.maximum_accepted_horizon
            && self.population.living_count() > 0
        {
            self.advance_one()?;
        }
        let end = EventV1::experiment_end(
            0,
            self.accepted_simulated_time,
            self.accepted_step,
            self.replicate,
            &self.current_environment_id,
            &self.protocol.protocol_id,
        );
        self.ledger
            .append(end)
            .map_err(|e| HarnessError::Events(e.to_string()))?;
        Ok(())
    }

    pub fn replicate_result(&self, seed: u64) -> ReplicateResultV1 {
        let event_ledger_valid = self.ledger.validate().is_ok();
        let actual_reproduction = self.generation.completed_fissions > 0;
        let pressure_before_reproduction = self
            .first_pressure_time
            .zip(self.first_reproduction_time)
            .map(|(pressure, reproduction)| pressure < reproduction)
            .unwrap_or(false);
        let campaign_role = self
            .protocol
            .selective_pressure
            .as_ref()
            .map(|contract| contract.campaign_role.clone());
        let pressure_contract_valid = self
            .protocol
            .selective_pressure
            .as_ref()
            .map(|contract| {
                contract.validate().is_ok()
                    && match contract.campaign_role {
                        CampaignRole::Treatment => {
                            self.current_environment_id == contract.treatment_environment
                        }
                        CampaignRole::Neutral => {
                            self.current_environment_id == contract.neutral_environment
                        }
                    }
            })
            .unwrap_or(false);
        let neutral_comparator_valid =
            campaign_role == Some(CampaignRole::Neutral) && pressure_contract_valid;
        let classification = if !event_ledger_valid {
            FailureClass::EventLedgerIncomplete
        } else if !self.environment_supported {
            FailureClass::EnvironmentUnsupported
        } else if self.generation.max_generation == 0 {
            FailureClass::SelectionUntestableZeroGeneration
        } else if self.generation.max_generation < self.protocol.minimum_generation_requirement {
            FailureClass::InsufficientGenerations
        } else if !actual_reproduction {
            FailureClass::NoReproduction
        } else if self.minimum_population_seen < self.protocol.minimum_population_viability {
            FailureClass::PopulationCollapsePreSelection
        } else if !self.heredity_preserved {
            FailureClass::HeredityNotPreserved
        } else if !self.phenotype_measurable {
            FailureClass::PhenotypeNotExpressed
        } else if !pressure_contract_valid {
            FailureClass::EcologyPressureNotReached
        } else if campaign_role == Some(CampaignRole::Treatment) && !self.pressure_reached {
            FailureClass::EcologyPressureNotReached
        } else if campaign_role == Some(CampaignRole::Treatment) && !pressure_before_reproduction {
            FailureClass::EcologyPressurePostReproduction
        } else {
            FailureClass::ReplicateQualified
        };
        ReplicateResultV1 {
            schema: "ReplicateResultV1".into(),
            experiment_id: self.protocol.experiment_id.clone(),
            environment: self.current_environment_id.clone(),
            replicate: self.replicate,
            seed,
            max_generation: self.generation.max_generation,
            birth_count: self.generation.completed_births,
            death_count: self
                .ledger
                .events
                .iter()
                .filter(|event| matches!(event.event_type, EventType::Death))
                .count() as u64,
            population_final: self.population.living_count() as u64,
            minimum_population_seen: self.minimum_population_seen,
            accepted_simulated_time: self.accepted_simulated_time,
            pressure_event_count: self.pressure_event_count,
            pressure_reached: self.pressure_reached,
            pressure_contract_valid,
            pressure_before_reproduction,
            campaign_role,
            actual_reproduction,
            heredity_preserved: self.heredity_preserved,
            phenotype_measurable: self.phenotype_measurable,
            event_ledger_valid,
            environment_supported: self.environment_supported,
            neutral_comparator_valid,
            classification,
            protocol_hash: self.protocol.hash(),
            event_ledger_hash: self.ledger.hash().unwrap_or_default(),
        }
    }
}

fn d096_mutation_stream_seed(campaign_seed: u64, qualified_copy_ordinal: u64) -> u64 {
    campaign_seed.wrapping_add(qualified_copy_ordinal)
}

fn mutation_metadata_reports_change(metadata: &crate::Metadata) -> bool {
    if metadata.get("mutation_occurred").map(String::as_str) != Some("true") {
        return false;
    }
    match (metadata.get("pre_genotype"), metadata.get("post_genotype")) {
        (Some(pre), Some(post)) => {
            let Ok(pre) = serde_json::from_str::<serde_json::Value>(pre) else {
                return false;
            };
            let Ok(post) = serde_json::from_str::<serde_json::Value>(post) else {
                return false;
            };
            pre != post
        }
        _ => true,
    }
}

fn valid_d096_mutation_metadata(metadata: &crate::Metadata) -> bool {
    metadata.get("operator").map(String::as_str)
        == Some("D096AllocationMutationOperator")
        && metadata
            .get("provenance")
            .is_some_and(|value| value.starts_with("DC-SR-004B;D-096_GATE6;"))
        && metadata.get("seed").is_some_and(|value| value.parse::<u64>().is_ok())
        && metadata
            .get("qualified_copy_ordinal")
            .is_some_and(|value| value.parse::<u64>().is_ok())
        && metadata.get("pre_genotype").is_some()
        && metadata.get("post_genotype").is_some()
}

#[cfg(test)]
mod tests {
    use super::d096_mutation_stream_seed;
    use chemistry_core::d096_allocation::{
        mutate_allocation_genotype, AllocationGenotype, AllocationParams,
    };

    #[test]
    fn d096_mutation_stream_repeats_within_replicate_and_diverges_across_seeds() {
        let params = AllocationParams::default();
        let trace = |campaign_seed| {
            (0..512_u64)
                .map(|qualified_copy_ordinal| {
                    let record = mutate_allocation_genotype(
                        AllocationGenotype::neutral(),
                        &params,
                        d096_mutation_stream_seed(campaign_seed, qualified_copy_ordinal),
                    )
                    .expect("frozen D-096 stream should remain valid");
                    (
                        record.mutation_occurred,
                        record.source,
                        record.target,
                        record.applied_delta.to_bits(),
                        record.post_genotype,
                    )
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(trace(17), trace(17));
        assert_ne!(trace(17), trace(18));
    }
}
