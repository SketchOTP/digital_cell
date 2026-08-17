//! DC-DEV-018 stateful integral assimilation-capacity homeostat.
//!
//! This post-Phase-1 wrapper owns exactly one adaptive scalar and can only
//! change the capacity of the existing N/F -> A source. It never writes
//! material fields and is disabled unless the caller opts in.

use serde::{Deserialize, Serialize};

pub const VERSION: &str = "digital_cell_integral_metabolic_homeostat_v1";
const EPS: f64 = 1.0e-15;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetabolicHomeostatParamsV1 {
    pub enable: bool,
    pub e_target: f64,
    pub tau_integral: f64,
    pub capacity_max: f64,
    pub k_integral: f64,
}

impl Default for MetabolicHomeostatParamsV1 {
    fn default() -> Self {
        Self {
            enable: false,
            e_target: 1.0,
            tau_integral: 80.0,
            capacity_max: 0.0,
            k_integral: 0.0,
        }
    }
}

impl MetabolicHomeostatParamsV1 {
    pub fn valid(&self) -> bool {
        self.e_target.is_finite()
            && self.e_target > EPS
            && self.tau_integral.is_finite()
            && self.tau_integral > EPS
            && self.capacity_max.is_finite()
            && self.capacity_max >= 0.0
            && self.k_integral.is_finite()
            && self.k_integral >= 0.0
    }

    pub fn derived(e_target: f64, capacity_max: f64, tau_integral: f64) -> Self {
        let tau_integral = tau_integral.max(EPS);
        let capacity_max = capacity_max.max(0.0);
        Self {
            enable: true,
            e_target,
            tau_integral,
            capacity_max,
            k_integral: capacity_max / tau_integral,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetabolicHomeostatStateV1 {
    pub assimilation_capacity: f64,
}

impl Default for MetabolicHomeostatStateV1 {
    fn default() -> Self {
        Self {
            assimilation_capacity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetabolicHomeostatStepV1 {
    pub error: f64,
    pub capacity_before: f64,
    pub capacity_after: f64,
    pub activation_gain: f64,
}

impl MetabolicHomeostatStateV1 {
    pub fn step(
        &mut self,
        area: f64,
        a: f64,
        r: f64,
        params: &MetabolicHomeostatParamsV1,
        dt: f64,
    ) -> MetabolicHomeostatStepV1 {
        let stored = area.max(0.0) * (a.max(0.0) + r.max(0.0));
        let error = if params.valid() {
            ((params.e_target - stored) / params.e_target).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let before = self
            .assimilation_capacity
            .clamp(0.0, params.capacity_max.max(0.0));
        let after = if params.enable && params.valid() && dt.is_finite() && dt > 0.0 {
            (before + params.k_integral * error * dt).clamp(0.0, params.capacity_max)
        } else {
            before
        };
        self.assimilation_capacity = after;
        MetabolicHomeostatStepV1 {
            error,
            capacity_before: before,
            capacity_after: after,
            activation_gain: if params.enable && params.valid() {
                1.0 + after
            } else {
                1.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_integral_accumulates_and_unwinds() {
        let params = MetabolicHomeostatParamsV1::derived(10.0, 4.0, 80.0);
        let mut state = MetabolicHomeostatStateV1::default();
        let deficit = state.step(1.0, 0.0, 0.0, &params, 0.02);
        assert!(deficit.error > 0.0 && deficit.capacity_after > 0.0);
        let recovered = state.step(10.0, 2.0, 0.0, &params, 0.02);
        assert!(recovered.error < 0.0 && recovered.capacity_after < deficit.capacity_after);
    }

    #[test]
    fn capacity_is_bounded_and_need_cannot_create_substrate() {
        let params = MetabolicHomeostatParamsV1::derived(10.0, 4.0, 80.0);
        let mut state = MetabolicHomeostatStateV1::default();
        for _ in 0..10_000 {
            state.step(1.0, 0.0, 0.0, &params, 0.02);
        }
        assert!((state.assimilation_capacity - 4.0).abs() <= 1.0e-12);
    }

    #[test]
    fn disabled_or_invalid_parameters_are_identity() {
        let mut state = MetabolicHomeostatStateV1 {
            assimilation_capacity: 2.0,
        };
        let disabled = MetabolicHomeostatParamsV1::default();
        assert_eq!(
            state.step(1.0, 0.0, 0.0, &disabled, 0.02).activation_gain,
            1.0
        );
        let invalid = MetabolicHomeostatParamsV1 {
            enable: true,
            e_target: 0.0,
            ..disabled
        };
        assert_eq!(
            state.step(1.0, 0.0, 0.0, &invalid, 0.02).activation_gain,
            1.0
        );
    }
}
