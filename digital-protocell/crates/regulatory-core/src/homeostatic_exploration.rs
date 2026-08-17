//! DC-DEV-014: direction-neutral homeostatic exploration nucleation.
//!
//! This module converts one already-selected internal material need signal
//! into rare, local regulatory stimulus pulses.  It has no access to body
//! geometry, world mass, goals, or motor state.  The existing
//! distributed regulator remains the only process that spreads and decays
//! the pulse, and the existing contractility/stick-slip path remains the only
//! process that can move the body.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const HOMEOSTATIC_EXPLORATION_SCHEMA_V1: &str = "digital_cell_homeostatic_exploration_v1";
pub const HOMEOSTATIC_EXPLORATION_DOMAIN_V1: u64 = 0x4443_3031_345f_4558;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeostaticExplorationParamsV1 {
    pub schema: String,
    pub k_decay: f64,
    pub dt: f64,
}

impl HomeostaticExplorationParamsV1 {
    pub fn from_regulator(k_decay: f64, dt: f64) -> Result<Self, HomeostaticExplorationError> {
        if !k_decay.is_finite() || k_decay <= 0.0 || !dt.is_finite() || dt <= 0.0 {
            return Err(HomeostaticExplorationError::InvalidParameters);
        }
        Ok(Self {
            schema: HOMEOSTATIC_EXPLORATION_SCHEMA_V1.to_string(),
            k_decay,
            dt,
        })
    }

    /// The event-rate scale is the reciprocal of the existing regulator
    /// decay timescale: at maximum need, total nucleation rate is k_decay.
    pub fn decay_timescale(&self) -> f64 {
        1.0 / self.k_decay
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeostaticExplorationStateV1 {
    pub schema: String,
    pub step_index: u64,
    pub rng_state: u64,
    pub event_count: u64,
    pub provenance_domain: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeostaticExplorationStepV1 {
    pub schema: String,
    pub step_index: u64,
    pub need_signal: f64,
    pub total_event_rate: f64,
    pub event_probability: f64,
    pub event_patch: Option<usize>,
    pub local_stimulus: Vec<f64>,
    pub provenance_domain: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HomeostaticExplorationError {
    #[error("homeostatic exploration parameters are invalid")]
    InvalidParameters,
    #[error("topology must contain at least three local patches")]
    InvalidTopology,
    #[error("need signal must be finite and in [0, 1]")]
    InvalidNeed,
}

/// A deterministic local nucleator.  The only runtime input is a normalized
/// internal material need and the local topology size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeostaticExplorationV1 {
    pub params: HomeostaticExplorationParamsV1,
    pub state: HomeostaticExplorationStateV1,
}

impl HomeostaticExplorationV1 {
    pub fn new(
        topology_size: usize,
        seed: u64,
        params: HomeostaticExplorationParamsV1,
    ) -> Result<Self, HomeostaticExplorationError> {
        if topology_size < 3
            || params.schema != HOMEOSTATIC_EXPLORATION_SCHEMA_V1
            || !params.k_decay.is_finite()
            || params.k_decay <= 0.0
            || !params.dt.is_finite()
            || params.dt <= 0.0
        {
            return Err(HomeostaticExplorationError::InvalidParameters);
        }
        Ok(Self {
            params,
            state: HomeostaticExplorationStateV1 {
                schema: HOMEOSTATIC_EXPLORATION_SCHEMA_V1.to_string(),
                step_index: 0,
                rng_state: seed ^ HOMEOSTATIC_EXPLORATION_DOMAIN_V1,
                event_count: 0,
                provenance_domain: HOMEOSTATIC_EXPLORATION_DOMAIN_V1,
            },
        })
    }

    pub fn step(
        &mut self,
        topology_size: usize,
        need_signal: f64,
    ) -> Result<HomeostaticExplorationStepV1, HomeostaticExplorationError> {
        if topology_size < 3 {
            return Err(HomeostaticExplorationError::InvalidTopology);
        }
        if !need_signal.is_finite() || !(0.0..=1.0).contains(&need_signal) {
            return Err(HomeostaticExplorationError::InvalidNeed);
        }

        let total_event_rate = self.params.k_decay * need_signal;
        let event_probability = 1.0 - (-total_event_rate * self.params.dt).exp();
        let mut local_stimulus = vec![0.0; topology_size];
        let event_patch = if self.next_unit() < event_probability {
            let patch = self.uniform_below(topology_size as u64) as usize;
            local_stimulus[patch] = 1.0;
            self.state.event_count = self.state.event_count.saturating_add(1);
            Some(patch)
        } else {
            None
        };
        let step_index = self.state.step_index;
        self.state.step_index = self.state.step_index.saturating_add(1);
        Ok(HomeostaticExplorationStepV1 {
            schema: HOMEOSTATIC_EXPLORATION_SCHEMA_V1.to_string(),
            step_index,
            need_signal,
            total_event_rate,
            event_probability,
            event_patch,
            local_stimulus,
            provenance_domain: self.state.provenance_domain,
        })
    }

    fn next_u64(&mut self) -> u64 {
        self.state.rng_state = self.state.rng_state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state.rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    fn uniform_below(&mut self, bound: u64) -> u64 {
        let limit = u64::MAX - (u64::MAX % bound);
        loop {
            let candidate = self.next_u64();
            if candidate < limit {
                return candidate % bound;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> HomeostaticExplorationParamsV1 {
        HomeostaticExplorationParamsV1::from_regulator(0.5, 0.02).unwrap()
    }

    #[test]
    fn maximum_need_uses_regulator_decay_timescale() {
        let p = params();
        assert!((p.decay_timescale() - 2.0).abs() < 1e-12);
        let expected = 1.0_f64 - (-0.5_f64 * 0.02_f64).exp();
        let mut x = HomeostaticExplorationV1::new(24, 9, p).unwrap();
        let step = x.step(24, 1.0).unwrap();
        assert!((step.total_event_rate - 0.5).abs() < 1e-12);
        assert!((step.event_probability - expected).abs() < 1e-12);
    }

    #[test]
    fn zero_need_cannot_nucleate() {
        let mut x = HomeostaticExplorationV1::new(24, 9, params()).unwrap();
        for _ in 0..128 {
            let step = x.step(24, 0.0).unwrap();
            assert_eq!(step.event_patch, None);
            assert!(step.local_stimulus.iter().all(|value| *value == 0.0));
        }
    }

    #[test]
    fn replay_is_deterministic_and_seed_is_provenant() {
        let mut a = HomeostaticExplorationV1::new(24, 123, params()).unwrap();
        let mut b = HomeostaticExplorationV1::new(24, 123, params()).unwrap();
        let mut c = HomeostaticExplorationV1::new(24, 124, params()).unwrap();
        let mut trace_a = Vec::new();
        let mut trace_b = Vec::new();
        let mut trace_c = Vec::new();
        for _ in 0..2048 {
            trace_a.push(a.step(24, 1.0).unwrap());
            trace_b.push(b.step(24, 1.0).unwrap());
            trace_c.push(c.step(24, 1.0).unwrap());
        }
        assert_eq!(trace_a, trace_b);
        assert_ne!(trace_a, trace_c);
    }
}
