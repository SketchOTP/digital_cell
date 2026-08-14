use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ProtocolError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("replicate count must be positive")]
    NoReplicates,
    #[error("seed count must equal replicate count")]
    SeedCountMismatch,
    #[error("mutation rate must be between 0 and 1")]
    InvalidMutationRate,
    #[error("maximum generation must be at least the minimum generation requirement")]
    GenerationBounds,
    #[error("protocol hash serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceMode {
    Continuous,
    Pulsed,
    Scarcity,
    Shared,
    SpatialLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DamageMode {
    None,
    Abrasion,
    DeclaredExternal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceEcologyV1 {
    pub schema: String,
    pub continuous_supply: bool,
    pub shared_resource_competition: bool,
    pub spatial_local_availability: bool,
    pub pulse_schedule: Vec<u64>,
    pub scarcity_schedule: Vec<u64>,
    pub damage_mode: DamageMode,
}

impl Default for ResourceEcologyV1 {
    fn default() -> Self {
        Self {
            schema: "ResourceEcologyV1".into(),
            continuous_supply: true,
            shared_resource_competition: true,
            spatial_local_availability: false,
            pulse_schedule: Vec::new(),
            scarcity_schedule: Vec::new(),
            damage_mode: DamageMode::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentProtocolV1 {
    pub schema: String,
    pub environment_id: String,
    pub resource_mode: ResourceMode,
    pub resource_field: String,
    pub resource_ecology: ResourceEcologyV1,
    pub pulse_schedule: Vec<u64>,
    pub scarcity_schedule: Vec<u64>,
    pub rich_duration: u64,
    pub lean_duration: u64,
    pub damage_mode: DamageMode,
    pub damage_interval: Option<u64>,
    pub spatial_constraints: String,
    pub transitions: Vec<String>,
    pub duration: u64,
    pub termination_rules: Vec<String>,
}

impl EnvironmentProtocolV1 {
    pub fn new(environment_id: impl Into<String>) -> Self {
        Self {
            schema: "EnvironmentProtocolV1".into(),
            environment_id: environment_id.into(),
            resource_mode: ResourceMode::Continuous,
            resource_field: "existing_digital_cell_resources".into(),
            resource_ecology: ResourceEcologyV1::default(),
            pulse_schedule: Vec::new(),
            scarcity_schedule: Vec::new(),
            rich_duration: 0,
            lean_duration: 0,
            damage_mode: DamageMode::None,
            damage_interval: None,
            spatial_constraints: "declared_by_experiment".into(),
            transitions: Vec::new(),
            duration: 0,
            termination_rules: vec!["accepted_horizon".into()],
        }
    }

    pub fn hash(&self) -> String {
        stable_hash(self).expect("environment protocol is serializable")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MutationProtocolV1 {
    pub schema: String,
    pub mutation_protocol_id: String,
    pub mutation_rate: f64,
    pub magnitude_distribution: String,
    pub bounds: String,
    pub provenance: String,
}

impl Default for MutationProtocolV1 {
    fn default() -> Self {
        Self {
            schema: "MutationProtocolV1".into(),
            mutation_protocol_id: "mutation_none".into(),
            mutation_rate: 0.0,
            magnitude_distribution: "none".into(),
            bounds: "declared_by_heredity_adapter".into(),
            provenance: "SR003".into(),
        }
    }
}

impl MutationProtocolV1 {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.mutation_protocol_id.is_empty() {
            return Err(ProtocolError::EmptyField { field: "mutation_protocol_id" });
        }
        if !(0.0..=1.0).contains(&self.mutation_rate) {
            return Err(ProtocolError::InvalidMutationRate);
        }
        Ok(())
    }

    pub fn hash(&self) -> String {
        stable_hash(self).expect("mutation protocol is serializable")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlacementProtocolV1 {
    pub schema: String,
    pub initial_coordinates: Vec<[f64; 2]>,
    pub spacing: f64,
    pub founder_seed: u64,
    pub random_seed: Option<u64>,
    pub dish_geometry: String,
    pub resource_geometry: String,
}

impl Default for PlacementProtocolV1 {
    fn default() -> Self {
        Self {
            schema: "PlacementProtocolV1".into(),
            initial_coordinates: vec![[0.0, 0.0]],
            spacing: 0.0,
            founder_seed: 0,
            random_seed: None,
            dish_geometry: "declared_by_experiment".into(),
            resource_geometry: "declared_by_environment".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentProtocolV1 {
    pub schema: String,
    pub experiment_id: String,
    pub protocol_id: String,
    pub organism_schema: String,
    pub heredity_schema: String,
    pub mutation_protocol: MutationProtocolV1,
    pub environment_protocol: EnvironmentProtocolV1,
    pub placement_protocol: PlacementProtocolV1,
    pub replicates: u32,
    pub random_seeds: Vec<u64>,
    pub maximum_accepted_horizon: u64,
    pub maximum_generation: u32,
    pub minimum_generation_requirement: u32,
    pub termination_rules: Vec<String>,
    pub primary_endpoints: Vec<String>,
    pub secondary_endpoints: Vec<String>,
}

impl ExperimentProtocolV1 {
    pub fn minimal(experiment_id: &str, environment_id: &str, mutation_id: &str) -> Self {
        let mut mutation = MutationProtocolV1::default();
        mutation.mutation_protocol_id = mutation_id.into();
        Self {
            schema: "ExperimentProtocolV1".into(),
            experiment_id: experiment_id.into(),
            protocol_id: format!("{experiment_id}_protocol"),
            organism_schema: "synthetic_test_organism".into(),
            heredity_schema: "synthetic_test_heredity".into(),
            mutation_protocol: mutation,
            environment_protocol: EnvironmentProtocolV1::new(environment_id),
            placement_protocol: PlacementProtocolV1::default(),
            replicates: 1,
            random_seeds: vec![0],
            maximum_accepted_horizon: 100,
            maximum_generation: 8,
            minimum_generation_requirement: 1,
            termination_rules: vec!["accepted_horizon".into(), "extinction".into()],
            primary_endpoints: vec!["completed_births".into(), "descendant_count".into()],
            secondary_endpoints: vec!["resource_consumption".into()],
        }
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        for (name, value) in [
            ("experiment_id", self.experiment_id.as_str()),
            ("protocol_id", self.protocol_id.as_str()),
            ("organism_schema", self.organism_schema.as_str()),
            ("heredity_schema", self.heredity_schema.as_str()),
        ] {
            if value.is_empty() {
                return Err(ProtocolError::EmptyField { field: name });
            }
        }
        if self.replicates == 0 {
            return Err(ProtocolError::NoReplicates);
        }
        if self.random_seeds.len() != self.replicates as usize {
            return Err(ProtocolError::SeedCountMismatch);
        }
        if self.maximum_generation < self.minimum_generation_requirement {
            return Err(ProtocolError::GenerationBounds);
        }
        self.mutation_protocol.validate()
    }

    pub fn hash(&self) -> String {
        stable_hash(self).expect("experiment protocol is serializable")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FounderIdentityV1 {
    pub schema: String,
    pub founder_id: u64,
    pub organism_schema: String,
    pub heredity_hash: String,
    pub phenotype_baseline: String,
    pub material_state_hash: String,
    pub seed: u64,
    pub preconditioning_protocol: String,
}

impl FounderIdentityV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        founder_id: u64,
        organism_schema: &str,
        heredity_hash: &str,
        phenotype_baseline: &str,
        material_state_hash: &str,
        seed: u64,
        preconditioning_protocol: &str,
    ) -> Self {
        Self {
            schema: "FounderIdentityV1".into(),
            founder_id,
            organism_schema: organism_schema.into(),
            heredity_hash: heredity_hash.into(),
            phenotype_baseline: phenotype_baseline.into(),
            material_state_hash: material_state_hash.into(),
            seed,
            preconditioning_protocol: preconditioning_protocol.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureClass {
    HarnessInvalid,
    FounderInvalid,
    PopulationCollapsePreSelection,
    ZeroGeneration,
    SelectionUntestableZeroGeneration,
    InsufficientGenerations,
    EcologyPressureNotReached,
    EcologyPressurePostReproduction,
    NoReproduction,
    HeredityNotPreserved,
    PhenotypeNotExpressed,
    NumericalFailure,
    ResourceExhaustionGlobal,
    LineageDataIncomplete,
    ValidNoSelectionEffect,
    ValidSelectionEffect,
    InterpretableTiming,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplicateResultV1 {
    pub schema: String,
    pub environment: String,
    pub seed: u64,
    pub max_generation: u32,
    pub birth_count: u64,
    pub death_count: u64,
    pub population_final: u64,
    pub classification: FailureClass,
    pub protocol_hash: String,
    pub event_ledger_hash: String,
}

impl ReplicateResultV1 {
    pub fn empty(environment: &str, seed: u64) -> Self {
        Self {
            schema: "ReplicateResultV1".into(),
            environment: environment.into(),
            seed,
            max_generation: 0,
            birth_count: 0,
            death_count: 0,
            population_final: 0,
            classification: FailureClass::SelectionUntestableZeroGeneration,
            protocol_hash: String::new(),
            event_ledger_hash: String::new(),
        }
    }
}

pub fn stable_hash<T: Serialize>(value: &T) -> Result<String, ProtocolError> {
    let bytes = serde_json::to_vec(value).map_err(|e| ProtocolError::Serialization(e.to_string()))?;
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("fnv1a64:{hash:016x}"))
}

pub fn classify_ecology_timing(
    first_reproduction_time: u64,
    pressure_start_time: u64,
) -> FailureClass {
    if first_reproduction_time <= pressure_start_time {
        FailureClass::EcologyPressurePostReproduction
    } else {
        FailureClass::InterpretableTiming
    }
}

pub type Metadata = BTreeMap<String, String>;
