use crate::{
    EnvironmentCapability, EnvironmentProtocolV1, FounderIdentityV1, Metadata, MutationProtocolV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AdapterError {
    #[error("founder could not be initialized: {0}")]
    Founder(String),
    #[error("organism advancement failed: {0}")]
    Advance(String),
    #[error("organism observation failed: {0}")]
    Observation(String),
    #[error("HARNESS_ADAPTER_UNAVAILABLE: requested existing heredity or ecology mechanism is not adapted")]
    Unavailable,
}

#[derive(Debug, Clone)]
pub enum AdvanceOutcome<O> {
    Continuing {
        accepted_dt: f64,
        metadata: Metadata,
    },
    Fission {
        offspring: Vec<O>,
        accepted_dt: f64,
        metadata: Metadata,
    },
    Died {
        reason: String,
        accepted_dt: f64,
        metadata: Metadata,
    },
}

impl<O> AdvanceOutcome<O> {
    pub fn accepted_dt(&self) -> f64 {
        match self {
            Self::Continuing { accepted_dt, .. }
            | Self::Fission { accepted_dt, .. }
            | Self::Died { accepted_dt, .. } => *accepted_dt,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnvironmentContext {
    pub living_population: usize,
    pub organism_index: usize,
    pub accepted_dt: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FounderInitializationContext {
    pub replicate: u32,
    pub founder_index: usize,
    pub population_size: usize,
    pub placement: [f64; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeredityEvidenceV1 {
    pub observable: bool,
    pub preserved: bool,
    pub comparison_basis: String,
    pub metric: String,
    pub value: Option<f64>,
    pub qualification: bool,
    pub reason: String,
}

impl HeredityEvidenceV1 {
    pub fn unavailable(reason: &str) -> Self {
        Self {
            observable: false,
            preserved: false,
            comparison_basis: "none".into(),
            metric: "none".into(),
            value: None,
            qualification: false,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhenotypeEvidenceV1 {
    pub observable: bool,
    pub expressed: bool,
    pub comparison_basis: String,
    pub metric: String,
    pub value: Option<f64>,
    pub qualification: bool,
    pub reason: String,
}

impl PhenotypeEvidenceV1 {
    pub fn unavailable(reason: &str) -> Self {
        Self {
            observable: false,
            expressed: false,
            comparison_basis: "none".into(),
            metric: "none".into(),
            value: None,
            qualification: false,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdapterEnvironmentEvent {
    pub event_type: crate::EventType,
    pub metadata: Metadata,
}

/// The causal surface exposed to the harness. There is no force-reproduce,
/// set-fitness, heal, kill, growth, or survival-probability command.
pub trait OrganismAdapter {
    type Organism;

    fn initialize_founder(
        &mut self,
        founder: &FounderIdentityV1,
        context: FounderInitializationContext,
    ) -> Result<Self::Organism, AdapterError>;
    fn accepted_dt(&self) -> f64;
    fn environment_capabilities(&self) -> Vec<EnvironmentCapability>;
    fn apply_declared_environment(
        &mut self,
        organism: &mut Self::Organism,
        environment: &EnvironmentProtocolV1,
        accepted_step: u64,
        accepted_simulated_time: f64,
        context: EnvironmentContext,
    ) -> Result<Vec<AdapterEnvironmentEvent>, AdapterError>;
    fn advance(
        &mut self,
        organism: &mut Self::Organism,
        environment: &EnvironmentProtocolV1,
        accepted_step: u64,
        accepted_simulated_time: f64,
    ) -> Result<AdvanceOutcome<Self::Organism>, AdapterError>;
    fn is_alive(&self, organism: &Self::Organism) -> bool;
    fn phenotype(&self, organism: &Self::Organism) -> String;
    fn hereditary_state(&self, organism: &Self::Organism) -> String;
    fn heredity_evidence(
        &self,
        _parent: Option<&Self::Organism>,
        _organism: &Self::Organism,
    ) -> HeredityEvidenceV1 {
        HeredityEvidenceV1::unavailable(
            "HARNESS_ADAPTER_UNAVAILABLE: mechanism-specific heredity qualifier is not adapted",
        )
    }
    fn phenotype_evidence(
        &self,
        _environment: &EnvironmentProtocolV1,
        _organism: &Self::Organism,
    ) -> PhenotypeEvidenceV1 {
        PhenotypeEvidenceV1::unavailable(
            "HARNESS_ADAPTER_UNAVAILABLE: mechanism-specific phenotype qualifier is not adapted",
        )
    }
    fn apply_heredity_and_mutation(
        &mut self,
        _parent: &Self::Organism,
        _offspring: &mut Self::Organism,
        protocol: &MutationProtocolV1,
        _context: &MutationContext,
    ) -> Result<Option<Metadata>, AdapterError> {
        if protocol.mutation_rate == 0.0 && protocol.mutation_protocol_id == "mutation_none" {
            Ok(None)
        } else {
            Err(AdapterError::Unavailable)
        }
    }
}

/// Mutation input deliberately contains no fitness, winner, survival, or
/// future-environment field. Mutation is blind to selection outcomes.
#[derive(Debug, Clone)]
pub struct MutationContext {
    pub accepted_step: u64,
    pub accepted_simulated_time: f64,
    pub seed: u64,
    pub offspring_index: u32,
    pub qualified_physical_copy: bool,
    pub qualified_copy_ordinal: u64,
    pub parent_hereditary_state: String,
}

pub trait HeredityAdapter {
    type HereditaryState;

    fn encode(&self, state: &Self::HereditaryState) -> String;
    fn decode(&self, encoded: &str) -> Result<Self::HereditaryState, AdapterError>;
}

pub trait MutationOperator {
    type HereditaryState;

    fn mutate(
        &self,
        state: &Self::HereditaryState,
        context: &MutationContext,
    ) -> Result<Self::HereditaryState, AdapterError>;
}
