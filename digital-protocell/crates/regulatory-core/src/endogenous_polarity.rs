//! DC-DEV-012: finite endogenous stochastic polarity.
//!
//! This module owns a conserved abstract token population on a fixed local
//! material ring.  It has no access to coordinates, forces, centroids,
//! stimulus labels, or any actuator.  Its only output is a body-attached
//! bounded local drive for the already-accepted regulatory network.

use chemistry_core::template_polymer::{RngLike, XorShift64};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ENDOGENOUS_POLARITY_SCHEMA_V1: &str = "digital_cell_endogenous_stochastic_polarity_v1";
pub const POLARITY_TOKEN_COUNT: u64 = 1_000;
pub const SUPPORTED_POLARITY_TOPOLOGY: usize = 24;
pub const FROZEN_FEEDBACK_RATE: f64 = 10.0;
pub const FROZEN_DISSOCIATION_RATE: f64 = 9.0;
pub const FROZEN_SPONTANEOUS_ASSOCIATION_RATE: f64 = 1.0e-4;
pub const FROZEN_DIFFUSION_COEFFICIENT: f64 = 1.2;
pub const FROZEN_POLARITY_DT: f64 = 0.02;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityParametersV1 {
    pub schema: String,
    pub total_tokens: u64,
    pub feedback_rate: f64,
    pub dissociation_rate: f64,
    pub spontaneous_association_rate: f64,
    pub diffusion_coefficient: f64,
    pub accepted_dt: f64,
    pub ring_spacing: f64,
    pub membrane_hop_rate: f64,
}

impl PolarityParametersV1 {
    pub fn from_ring_spacing(ring_spacing: f64) -> Result<Self, EndogenousPolarityError> {
        if !ring_spacing.is_finite() || ring_spacing <= 0.0 {
            return Err(EndogenousPolarityError::InvalidParameters(
                "ring spacing must be finite and positive".to_string(),
            ));
        }
        Ok(Self {
            schema: ENDOGENOUS_POLARITY_SCHEMA_V1.to_string(),
            total_tokens: POLARITY_TOKEN_COUNT,
            feedback_rate: FROZEN_FEEDBACK_RATE,
            dissociation_rate: FROZEN_DISSOCIATION_RATE,
            spontaneous_association_rate: FROZEN_SPONTANEOUS_ASSOCIATION_RATE,
            diffusion_coefficient: FROZEN_DIFFUSION_COEFFICIENT,
            accepted_dt: FROZEN_POLARITY_DT,
            ring_spacing,
            // In one dimension, a symmetric nearest-neighbor walk with rate
            // q on each edge has D = q h^2.  Thus q = D / h^2.
            membrane_hop_rate: FROZEN_DIFFUSION_COEFFICIENT / ring_spacing.powi(2),
        })
    }

    fn validate(&self) -> Result<(), EndogenousPolarityError> {
        let expected = Self::from_ring_spacing(self.ring_spacing)?;
        if self.schema != expected.schema
            || self.total_tokens != expected.total_tokens
            || (self.feedback_rate - expected.feedback_rate).abs() > 1e-15
            || (self.dissociation_rate - expected.dissociation_rate).abs() > 1e-15
            || (self.spontaneous_association_rate - expected.spontaneous_association_rate).abs()
                > 1e-15
            || (self.diffusion_coefficient - expected.diffusion_coefficient).abs() > 1e-15
            || (self.accepted_dt - expected.accepted_dt).abs() > 1e-15
            || (self.membrane_hop_rate - expected.membrane_hop_rate).abs() > 1e-12
        {
            return Err(EndogenousPolarityError::InvalidParameters(
                "parameters differ from the frozen DC-DEV-012 reference set".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityStateV1 {
    pub schema: String,
    pub topology_size: usize,
    pub total_tokens: u64,
    pub cytosolic_tokens: u64,
    pub membrane_bound_tokens: Vec<u64>,
    pub accepted_step: u64,
    pub accepted_time: f64,
    pub rng_state: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityStepLedgerV1 {
    pub schema: String,
    pub accepted_step: u64,
    pub accepted_time: f64,
    pub cytosolic_tokens: u64,
    pub membrane_bound_tokens: Vec<u64>,
    pub association_events: u64,
    pub recruitment_events: u64,
    pub diffusion_events: u64,
    pub dissociation_events: u64,
    pub state_hash: String,
}

#[derive(Debug, Error, PartialEq)]
pub enum EndogenousPolarityError {
    #[error("unsupported polarity topology: expected {expected}, observed {observed}")]
    UnsupportedTopology { expected: usize, observed: usize },
    #[error("invalid endogenous polarity parameters: {0}")]
    InvalidParameters(String),
    #[error("invalid endogenous polarity state: {0}")]
    InvalidState(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone)]
pub struct EndogenousPolarityV1 {
    params: PolarityParametersV1,
    state: PolarityStateV1,
}

impl EndogenousPolarityV1 {
    pub fn new(
        topology_size: usize,
        provenance_seed: u64,
        ring_spacing: f64,
    ) -> Result<Self, EndogenousPolarityError> {
        if topology_size != SUPPORTED_POLARITY_TOPOLOGY {
            return Err(EndogenousPolarityError::UnsupportedTopology {
                expected: SUPPORTED_POLARITY_TOPOLOGY,
                observed: topology_size,
            });
        }
        let params = PolarityParametersV1::from_ring_spacing(ring_spacing)?;
        let state = PolarityStateV1 {
            schema: ENDOGENOUS_POLARITY_SCHEMA_V1.to_string(),
            topology_size,
            total_tokens: POLARITY_TOKEN_COUNT,
            cytosolic_tokens: POLARITY_TOKEN_COUNT,
            membrane_bound_tokens: vec![0; topology_size],
            accepted_step: 0,
            accepted_time: 0.0,
            rng_state: provenance_seed.max(1),
        };
        let result = Self { params, state };
        result.validate()?;
        Ok(result)
    }

    pub fn parameters(&self) -> &PolarityParametersV1 {
        &self.params
    }

    pub fn state(&self) -> &PolarityStateV1 {
        &self.state
    }

    pub fn token_conserved(&self) -> bool {
        self.state.cytosolic_tokens + self.bound_total() == self.state.total_tokens
    }

    pub fn bound_total(&self) -> u64 {
        self.state.membrane_bound_tokens.iter().sum()
    }

    /// Convert body-attached membrane density to the bounded internal drive
    /// specified by DC-DEV-012.  This method reads only token state and the
    /// fixed local topology size.
    pub fn drive(&self) -> Vec<f64> {
        let topology_size = self.state.topology_size as f64;
        let total = self.state.total_tokens as f64;
        self.state
            .membrane_bound_tokens
            .iter()
            .map(|bound| (topology_size * *bound as f64 / total).clamp(0.0, 1.0))
            .collect()
    }

    pub fn state_hash(&self) -> Result<String, EndogenousPolarityError> {
        crate::stable_json_hash(&self.state)
            .map_err(|error| EndogenousPolarityError::Serialization(error.to_string()))
    }

    /// Advance exactly one accepted Digital Cell dt using a finite-event
    /// continuous-time Markov process.  All four allowed event types are
    /// sampled from the current propensities; no schedule or forced event is
    /// present.
    pub fn step(&mut self) -> Result<PolarityStepLedgerV1, EndogenousPolarityError> {
        self.validate()?;
        let mut rng = XorShift64::new(self.state.rng_state);
        let mut remaining = self.params.accepted_dt;
        let mut association_events = 0;
        let mut recruitment_events = 0;
        let mut diffusion_events = 0;
        let mut dissociation_events = 0;

        while remaining > 0.0 {
            let cytosolic = self.state.cytosolic_tokens as f64;
            let bound = self.bound_total() as f64;
            let association = self.params.spontaneous_association_rate * cytosolic;
            let recruitment =
                self.params.feedback_rate * bound * cytosolic / self.state.total_tokens as f64;
            let diffusion = 2.0 * self.params.membrane_hop_rate * bound;
            let dissociation = self.params.dissociation_rate * bound;
            let total_rate = association + recruitment + diffusion + dissociation;
            if total_rate <= 0.0 || !total_rate.is_finite() {
                break;
            }

            let uniform = rng.unit().clamp(f64::MIN_POSITIVE, 1.0 - f64::EPSILON);
            let wait = -uniform.ln() / total_rate;
            if wait >= remaining {
                break;
            }
            remaining -= wait;

            let choice = rng.unit() * total_rate;
            if choice < association {
                let patch = ((rng.unit() * self.state.topology_size as f64).floor() as usize)
                    .min(self.state.topology_size - 1);
                self.state.cytosolic_tokens -= 1;
                self.state.membrane_bound_tokens[patch] += 1;
                association_events += 1;
            } else if choice < association + recruitment {
                let patch = self.sample_bound_patch(&mut rng)?;
                self.state.cytosolic_tokens -= 1;
                self.state.membrane_bound_tokens[patch] += 1;
                recruitment_events += 1;
            } else if choice < association + recruitment + diffusion {
                let source = self.sample_bound_patch(&mut rng)?;
                let direction = if rng.unit() < 0.5 { -1_i32 } else { 1_i32 };
                let target = ((source as i32 + direction)
                    .rem_euclid(self.state.topology_size as i32))
                    as usize;
                self.state.membrane_bound_tokens[source] -= 1;
                self.state.membrane_bound_tokens[target] += 1;
                diffusion_events += 1;
            } else {
                let patch = self.sample_bound_patch(&mut rng)?;
                self.state.membrane_bound_tokens[patch] -= 1;
                self.state.cytosolic_tokens += 1;
                dissociation_events += 1;
            }
        }

        self.state.rng_state = rng.state();
        self.state.accepted_step = self.state.accepted_step.saturating_add(1);
        self.state.accepted_time += self.params.accepted_dt;
        self.validate()?;
        Ok(PolarityStepLedgerV1 {
            schema: ENDOGENOUS_POLARITY_SCHEMA_V1.to_string(),
            accepted_step: self.state.accepted_step,
            accepted_time: self.state.accepted_time,
            cytosolic_tokens: self.state.cytosolic_tokens,
            membrane_bound_tokens: self.state.membrane_bound_tokens.clone(),
            association_events,
            recruitment_events,
            diffusion_events,
            dissociation_events,
            state_hash: self.state_hash()?,
        })
    }

    fn sample_bound_patch(&self, rng: &mut XorShift64) -> Result<usize, EndogenousPolarityError> {
        let total = self.bound_total();
        if total == 0 {
            return Err(EndogenousPolarityError::InvalidState(
                "a bound-patch sample was requested with no membrane tokens".to_string(),
            ));
        }
        let target = rng.unit() * total as f64;
        let mut cumulative = 0.0;
        for (index, count) in self.state.membrane_bound_tokens.iter().enumerate() {
            cumulative += *count as f64;
            if target < cumulative {
                return Ok(index);
            }
        }
        Ok(self.state.topology_size - 1)
    }

    fn validate(&self) -> Result<(), EndogenousPolarityError> {
        self.params.validate()?;
        if self.state.schema != ENDOGENOUS_POLARITY_SCHEMA_V1
            || self.state.topology_size != SUPPORTED_POLARITY_TOPOLOGY
            || self.state.total_tokens != POLARITY_TOKEN_COUNT
            || self.state.membrane_bound_tokens.len() != self.state.topology_size
            || !self.state.accepted_time.is_finite()
            || self.state.accepted_time < 0.0
            || self.state.rng_state == 0
            || !self.token_conserved()
        {
            return Err(EndogenousPolarityError::InvalidState(
                "schema, topology, bounds, time, RNG, or token conservation is invalid".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RING_SPACING: f64 = 1.0;

    fn run(seed: u64, steps: usize) -> EndogenousPolarityV1 {
        let mut polarity =
            EndogenousPolarityV1::new(SUPPORTED_POLARITY_TOPOLOGY, seed, RING_SPACING).unwrap();
        for _ in 0..steps {
            polarity.step().unwrap();
            assert!(polarity.token_conserved());
            assert!(polarity
                .state()
                .membrane_bound_tokens
                .iter()
                .all(|value| *value <= POLARITY_TOKEN_COUNT));
        }
        polarity
    }

    #[test]
    fn exact_token_conservation_and_bounds_hold() {
        let polarity = run(12001, 500);
        assert_eq!(
            polarity.bound_total() + polarity.state().cytosolic_tokens,
            1000
        );
        assert!(polarity
            .drive()
            .iter()
            .all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn same_seed_replays_exactly_and_time_is_accepted_time() {
        let a = run(12001, 150);
        let b = run(12001, 150);
        assert_eq!(a.state(), b.state());
        assert!((a.state().accepted_time - 3.0).abs() < 1e-12);
    }

    #[test]
    fn different_seeds_are_allowed_to_diverge() {
        let a = run(12001, 150);
        let b = run(12002, 150);
        assert_ne!(a.state(), b.state());
    }

    #[test]
    fn diffusion_is_nearest_neighbor_and_recruitment_is_same_patch() {
        let mut polarity = run(12001, 1);
        polarity.state.cytosolic_tokens = 999;
        polarity.state.membrane_bound_tokens[7] = 1;
        let before = polarity.state.membrane_bound_tokens.clone();
        let _ = polarity.step().unwrap();
        for (index, count) in polarity.state.membrane_bound_tokens.iter().enumerate() {
            if index != 7 && index != 6 && index != 8 {
                assert_eq!(*count, before[index]);
            }
        }
    }

    #[test]
    fn unsupported_topology_fails_closed() {
        let result = EndogenousPolarityV1::new(23, 12001, RING_SPACING);
        assert!(matches!(
            result,
            Err(EndogenousPolarityError::UnsupportedTopology { .. })
        ));
    }

    #[test]
    fn diffusion_rate_is_derived_from_reference_coefficient_and_spacing() {
        let params = PolarityParametersV1::from_ring_spacing(2.0).unwrap();
        assert!((params.membrane_hop_rate - 0.3).abs() < 1e-12);
    }
}
