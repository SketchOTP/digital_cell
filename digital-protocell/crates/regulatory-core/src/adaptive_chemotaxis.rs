//! Material-local adaptive directional sensing for DC-FINAL-001-R1.
//!
//! This module performs no mechanics and reads no world coordinates.  It
//! compares bounded local membrane material occupancy with its
//! perimeter-weighted membrane mean.  The resulting front/rear fields are
//! therefore rotation equivariant and exactly silent for uniform exposure.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ADAPTIVE_DIRECTIONAL_SENSOR_SCHEMA_V1: &str =
    "digital_cell_adaptive_directional_sensor_v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveDirectionalDriveV1 {
    pub schema: String,
    pub local_occupancy: Vec<f64>,
    pub perimeter_weighted_mean: f64,
    pub front_drive: Vec<f64>,
    pub rear_drive: Vec<f64>,
}

#[derive(Debug, Error, PartialEq)]
pub enum AdaptiveDirectionalSensorError {
    #[error("occupancy and perimeter measures must have the same nonzero length")]
    Length,
    #[error("occupancy must be finite and bounded in [0,1]")]
    Occupancy,
    #[error("perimeter measures must be finite and positive")]
    Measure,
}

/// Compute the zero-gain LEGI-like spatial comparator authorized by R1.
///
/// The global term is a perimeter-weighted mean, interpreted by the final
/// organism as a well-mixed intracellular inhibitory comparison.  No target,
/// bearing, observer normalization, fitted gain, or concentration threshold
/// enters this operation.
pub fn adaptive_directional_drive(
    occupancy: &[f64],
    perimeter_measures: &[f64],
) -> Result<AdaptiveDirectionalDriveV1, AdaptiveDirectionalSensorError> {
    if occupancy.is_empty() || occupancy.len() != perimeter_measures.len() {
        return Err(AdaptiveDirectionalSensorError::Length);
    }
    if occupancy
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(AdaptiveDirectionalSensorError::Occupancy);
    }
    if perimeter_measures
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(AdaptiveDirectionalSensorError::Measure);
    }

    // Preserve exact silence for bitwise-uniform exposure rather than
    // allowing summation roundoff to manufacture a directional state.
    let uniform = occupancy
        .iter()
        .all(|value| value.to_bits() == occupancy[0].to_bits());
    let total_measure: f64 = perimeter_measures.iter().sum();
    let mean = if uniform {
        occupancy[0]
    } else {
        occupancy
            .iter()
            .zip(perimeter_measures)
            .map(|(signal, measure)| signal * measure)
            .sum::<f64>()
            / total_measure
    };
    let (front_drive, rear_drive) = if uniform {
        (vec![0.0; occupancy.len()], vec![0.0; occupancy.len()])
    } else {
        (
            occupancy
                .iter()
                .map(|value| (value - mean).max(0.0))
                .collect(),
            occupancy
                .iter()
                .map(|value| (mean - value).max(0.0))
                .collect(),
        )
    };
    Ok(AdaptiveDirectionalDriveV1 {
        schema: ADAPTIVE_DIRECTIONAL_SENSOR_SCHEMA_V1.to_string(),
        local_occupancy: occupancy.to_vec(),
        perimeter_weighted_mean: mean,
        front_drive,
        rear_drive,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_exposure_is_exactly_silent() {
        let drive = adaptive_directional_drive(&[0.4; 8], &[1.0; 8]).unwrap();
        assert_eq!(drive.front_drive, vec![0.0; 8]);
        assert_eq!(drive.rear_drive, vec![0.0; 8]);
        assert_eq!(drive.perimeter_weighted_mean, 0.4);
    }

    #[test]
    fn weighted_comparator_separates_front_and_rear_without_gain() {
        let drive = adaptive_directional_drive(&[0.8, 0.2], &[1.0, 3.0]).unwrap();
        assert!((drive.perimeter_weighted_mean - 0.35).abs() < 1e-12);
        assert!((drive.front_drive[0] - 0.45).abs() < 1e-12);
        assert_eq!(drive.front_drive[1], 0.0);
        assert_eq!(drive.rear_drive[0], 0.0);
        assert!((drive.rear_drive[1] - 0.15).abs() < 1e-12);
    }

    #[test]
    fn index_rotation_rotates_only_the_local_fields() {
        let a = adaptive_directional_drive(&[0.8, 0.2, 0.4], &[1.0; 3]).unwrap();
        let b = adaptive_directional_drive(&[0.4, 0.8, 0.2], &[1.0; 3]).unwrap();
        assert!((a.perimeter_weighted_mean - b.perimeter_weighted_mean).abs() < 1e-12);
        for (left, right) in [a.front_drive[2], a.front_drive[0], a.front_drive[1]]
            .iter()
            .zip(&b.front_drive)
        {
            assert!((left - right).abs() < 1e-12);
        }
        for (left, right) in [a.rear_drive[2], a.rear_drive[0], a.rear_drive[1]]
            .iter()
            .zip(&b.rear_drive)
        {
            assert!((left - right).abs() < 1e-12);
        }
    }
}
