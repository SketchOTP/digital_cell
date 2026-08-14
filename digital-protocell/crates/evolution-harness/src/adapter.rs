use crate::{EnvironmentProtocolV1, FounderIdentityV1, Metadata};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("founder could not be initialized: {0}")]
    Founder(String),
    #[error("organism advancement failed: {0}")]
    Advance(String),
    #[error("organism observation failed: {0}")]
    Observation(String),
    #[error("adapter unavailable for the requested historical system")]
    Unavailable,
}

#[derive(Debug, Clone)]
pub enum AdvanceOutcome<O> {
    Continuing,
    Fission { offspring: Vec<O>, metadata: Metadata },
    Died { reason: String },
}

/// The only causal surface exposed to the harness.
pub trait OrganismAdapter {
    type Organism;

    fn initialize_founder(&self, founder: &FounderIdentityV1) -> Result<Self::Organism, AdapterError>;
    fn advance(
        &self,
        organism: &mut Self::Organism,
        environment: &EnvironmentProtocolV1,
        accepted_step: u64,
        accepted_simulated_time: u64,
    ) -> Result<AdvanceOutcome<Self::Organism>, AdapterError>;
    fn is_alive(&self, organism: &Self::Organism) -> bool;
    fn phenotype(&self, organism: &Self::Organism) -> String;
    fn hereditary_state(&self, organism: &Self::Organism) -> String;
    fn apply_declared_environment(
        &self,
        _organism: &mut Self::Organism,
        _environment: &EnvironmentProtocolV1,
        _accepted_step: u64,
    ) -> Result<Option<String>, AdapterError> {
        Ok(None)
    }
    fn resource_state(&self, _organism: &Self::Organism) -> String {
        "adapter_resource_state_unreported".into()
    }
}

/// Mutation input deliberately contains no fitness, winner, survival, or
/// future-environment field. Mutation is blind to selection outcomes.
#[derive(Debug, Clone)]
pub struct MutationContext {
    pub accepted_step: u64,
    pub accepted_simulated_time: u64,
    pub seed: u64,
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
