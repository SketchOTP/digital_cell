//! DC-DEV-019 one-state material homeostat.
//!
//! This module is deliberately narrower than a controller: it observes only
//! stored material, area, the frozen target, and accepted time, then returns
//! bounded gains for already-existing nutrient paths. It never reads a world,
//! contact signal, regulator, motor, target, or reward, and never writes mesh
//! material directly.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const METABOLIC_ACQUISITION_HOMEOSTAT_SCHEMA_V1: &str =
    "digital_cell_metabolic_acquisition_homeostat_v1";
pub const HOMEOSTAT_TAU: f64 = 80.0;
pub const HOMEOSTAT_SOURCE_GAIN_MAX: f64 = 6.97512279078733;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetabolicAcquisitionHomeostatV1 {
    pub schema: String,
    pub enabled: bool,
    pub h: f64,
    pub e_target: f64,
    pub tau: f64,
    pub k_h: f64,
    pub g_source_max: f64,
    pub g_transport_max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MetabolicHomeostatStepV1 {
    pub e_before: f64,
    pub e_after: f64,
    pub h_before: f64,
    pub h_after: f64,
    pub g_source: f64,
    pub g_transport: f64,
}

#[derive(Debug, Error, PartialEq)]
pub enum MetabolicHomeostatError {
    #[error("homeostat target, initial error, and tau must be finite and positive")]
    InvalidScale,
    #[error("homeostat gains must be finite and at least one")]
    InvalidGain,
}

impl MetabolicAcquisitionHomeostatV1 {
    pub fn try_new(
        enabled: bool,
        e_target: f64,
        e0: f64,
        tau: f64,
        g_source_max: f64,
        g_transport_max: f64,
    ) -> Result<Self, MetabolicHomeostatError> {
        if !e_target.is_finite()
            || e_target <= 0.0
            || !e0.is_finite()
            || e0 <= 0.0
            || !tau.is_finite()
            || tau <= 0.0
        {
            return Err(MetabolicHomeostatError::InvalidScale);
        }
        if !g_source_max.is_finite()
            || g_source_max < 1.0
            || !g_transport_max.is_finite()
            || g_transport_max < 1.0
        {
            return Err(MetabolicHomeostatError::InvalidGain);
        }
        Ok(Self {
            schema: METABOLIC_ACQUISITION_HOMEOSTAT_SCHEMA_V1.to_string(),
            enabled,
            h: 0.0,
            e_target,
            tau,
            k_h: 2.0 / (e0 * tau),
            g_source_max,
            g_transport_max,
        })
    }

    pub fn e_stored(area: f64, a: f64, r: f64) -> f64 {
        area.max(0.0) * (a.max(0.0) + r.max(0.0))
    }

    pub fn error(&self, area: f64, a: f64, r: f64) -> f64 {
        ((self.e_target - Self::e_stored(area, a, r)) / self.e_target).clamp(-1.0, 1.0)
    }

    pub fn advance(&mut self, area: f64, a: f64, r: f64, dt: f64) -> MetabolicHomeostatStepV1 {
        let e_before = self.error(area, a, r);
        let h_before = self.h;
        if self.enabled && dt.is_finite() && dt > 0.0 {
            self.h = (self.h + self.k_h * e_before * dt).clamp(0.0, 1.0);
        }
        let e_after = e_before;
        let g_source = if self.enabled {
            1.0 + self.h * (self.g_source_max - 1.0)
        } else {
            1.0
        };
        let g_transport = if self.enabled {
            1.0 + self.h * (self.g_transport_max - 1.0)
        } else {
            1.0
        };
        MetabolicHomeostatStepV1 {
            e_before,
            e_after,
            h_before,
            h_after: self.h,
            g_source,
            g_transport,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_frozen_integrator_gain() {
        let h = MetabolicAcquisitionHomeostatV1::try_new(
            true,
            100.0,
            0.2,
            HOMEOSTAT_TAU,
            HOMEOSTAT_SOURCE_GAIN_MAX,
            1.0,
        )
        .unwrap();
        assert!((h.k_h - 2.0 / (0.2 * 80.0)).abs() < 1e-15);
    }

    #[test]
    fn feature_off_is_neutral_and_does_not_accumulate() {
        let mut h = MetabolicAcquisitionHomeostatV1::try_new(false, 100.0, 0.2, 80.0, 6.0, 2.0)
            .unwrap();
        let step = h.advance(10.0, 0.0, 0.0, 0.02);
        assert_eq!(step.g_source, 1.0);
        assert_eq!(step.g_transport, 1.0);
        assert_eq!(h.h, 0.0);
    }

    #[test]
    fn starvation_accumulates_but_gains_remain_bounded() {
        let mut h = MetabolicAcquisitionHomeostatV1::try_new(true, 100.0, 0.2, 80.0, 6.0, 2.0)
            .unwrap();
        let step = h.advance(10.0, 0.0, 0.0, 80.0);
        assert_eq!(step.h_after, 1.0);
        assert_eq!(step.g_source, 6.0);
        assert_eq!(step.g_transport, 2.0);
    }

    #[test]
    fn invalid_parameters_fail_closed() {
        assert_eq!(
            MetabolicAcquisitionHomeostatV1::try_new(true, 0.0, 0.2, 80.0, 2.0, 1.0),
            Err(MetabolicHomeostatError::InvalidScale)
        );
        assert_eq!(
            MetabolicAcquisitionHomeostatV1::try_new(true, 100.0, 0.2, 80.0, 0.5, 1.0),
            Err(MetabolicHomeostatError::InvalidGain)
        );
    }
}
