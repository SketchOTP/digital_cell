//! DC-M4-R4: polarity to the existing paid local actuator.
//!
//! This module owns only the local, dimensionless input adapter.  It does not
//! move coordinates, compute forces, spend A, or inspect reproduction state.
//! The caller must pass the returned activity to the already-authorized
//! activated-energy contractility operator, which remains the sole authority
//! for mechanics and mechanical A->W accounting.

use crate::polarity_mass::{
    homogeneous_active_concentration, PolarityMassError, PolarityMassParamsV1, PolarityMassStateV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const POLARITY_ACTUATION_SCHEMA_V1: &str = "digital_cell_polarity_to_paid_actuation_v1";
pub const R3_TOTAL_POLARITY_CONCENTRATION: f64 = 0.8;

/// The R4 coupling contains no fitted gain.  Its reference concentration is
/// derived from the sealed R3 candidate equations at the sealed R3 total
/// polarity concentration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityActuationParamsV1 {
    pub schema: String,
    pub reference_total_concentration: f64,
    pub reference_active_concentration: f64,
}

impl PolarityActuationParamsV1 {
    pub fn sealed_r3_candidate() -> Result<Self, PolarityActuationError> {
        let polarity = PolarityMassParamsV1::candidate();
        let reference_total_concentration = R3_TOTAL_POLARITY_CONCENTRATION;
        let reference_active_concentration =
            homogeneous_active_concentration(reference_total_concentration, &polarity)?;
        let params = Self {
            schema: POLARITY_ACTUATION_SCHEMA_V1.to_string(),
            reference_total_concentration,
            reference_active_concentration,
        };
        params.validate()?;
        Ok(params)
    }

    fn validate(&self) -> Result<(), PolarityActuationError> {
        if self.schema != POLARITY_ACTUATION_SCHEMA_V1
            || !self.reference_total_concentration.is_finite()
            || self.reference_total_concentration <= 0.0
            || !self.reference_active_concentration.is_finite()
            || self.reference_active_concentration <= 0.0
            || self.reference_active_concentration >= self.reference_total_concentration
        {
            return Err(PolarityActuationError::InvalidParameters);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityActuationProposalV1 {
    pub schema: String,
    pub reference_total_concentration: f64,
    pub reference_active_concentration: f64,
    pub edge_active_concentrations: Vec<f64>,
    pub vertex_activity: Vec<f64>,
    pub homogeneous_activity_zero: bool,
}

#[derive(Debug, Error, PartialEq)]
pub enum PolarityActuationError {
    #[error("polarity actuation parameters are invalid")]
    InvalidParameters,
    #[error("polarity state or edge measures are invalid: {0}")]
    InvalidPolarity(String),
    #[error("polarity reference concentration is not positive")]
    InvalidReference,
}

/// Derive the only R4 actuator input.  Edge concentrations are converted to
/// vertex-local concentrations by the existing nearest-edge average.  A
/// positive deviation above the sealed homogeneous active concentration is
/// mapped to the existing [0, 1] activity domain; all lower deviations map to
/// zero.  No organism-wide normalization or geometric target is used.
pub fn derive_local_activity(
    state: &PolarityMassStateV1,
    measures: &[f64],
    params: &PolarityActuationParamsV1,
) -> Result<PolarityActuationProposalV1, PolarityActuationError> {
    params.validate()?;
    state
        .validate(measures)
        .map_err(|error| PolarityActuationError::InvalidPolarity(error.to_string()))?;
    if params.reference_active_concentration <= 0.0 {
        return Err(PolarityActuationError::InvalidReference);
    }
    let edge_active_concentrations = state
        .active_amount
        .iter()
        .zip(measures)
        .map(|(amount, measure)| amount / measure)
        .collect::<Vec<_>>();
    let mut vertex_activity = Vec::with_capacity(measures.len());
    for vertex in 0..measures.len() {
        let previous = edge_active_concentrations[(vertex + measures.len() - 1) % measures.len()];
        let next = edge_active_concentrations[vertex];
        let local_concentration = 0.5 * (previous + next);
        let activity = ((local_concentration - params.reference_active_concentration)
            / params.reference_active_concentration)
            .clamp(0.0, 1.0);
        vertex_activity.push(activity);
    }
    let homogeneous_activity_zero = edge_active_concentrations
        .iter()
        .all(|value| (*value - params.reference_active_concentration).abs() <= 1e-14);
    Ok(PolarityActuationProposalV1 {
        schema: POLARITY_ACTUATION_SCHEMA_V1.to_string(),
        reference_total_concentration: params.reference_total_concentration,
        reference_active_concentration: params.reference_active_concentration,
        edge_active_concentrations,
        vertex_activity,
        homogeneous_activity_zero,
    })
}

impl From<PolarityMassError> for PolarityActuationError {
    fn from(error: PolarityMassError) -> Self {
        Self::InvalidPolarity(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> PolarityActuationParamsV1 {
        PolarityActuationParamsV1::sealed_r3_candidate().unwrap()
    }

    #[test]
    fn homogeneous_state_has_no_actuator_input() {
        let measures = vec![1.0; 8];
        let p = params();
        let state = PolarityMassStateV1 {
            schema: crate::polarity_mass::POLARITY_MASS_SCHEMA_V1.to_string(),
            active_amount: vec![p.reference_active_concentration; 8],
            inactive_amount: vec![1.0 - p.reference_active_concentration; 8],
            accepted_steps: 0,
        };
        let proposal = derive_local_activity(&state, &measures, &p).unwrap();
        assert!(proposal.homogeneous_activity_zero);
        assert!(proposal.vertex_activity.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn only_local_adjacent_polarity_changes_vertex_activity() {
        let measures = vec![1.0; 8];
        let p = params();
        let mut state = PolarityMassStateV1 {
            schema: crate::polarity_mass::POLARITY_MASS_SCHEMA_V1.to_string(),
            active_amount: vec![p.reference_active_concentration; 8],
            inactive_amount: vec![1.0 - p.reference_active_concentration; 8],
            accepted_steps: 0,
        };
        state.active_amount[3] = p.reference_active_concentration * 1.5;
        let proposal = derive_local_activity(&state, &measures, &p).unwrap();
        assert!(proposal.vertex_activity[3] > 0.0);
        assert!(proposal.vertex_activity[4] > 0.0);
        assert_eq!(proposal.vertex_activity[0], 0.0);
        assert_eq!(proposal.vertex_activity[1], 0.0);
        assert_eq!(proposal.vertex_activity[6], 0.0);
    }
}
