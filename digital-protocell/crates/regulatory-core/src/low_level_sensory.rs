//! Low-level, non-semantic camera and microphone transduction.
//!
//! The transducer exposes measured luminance, frame-to-frame motion, audio
//! amplitude, and two coarse spectral bands as environmental fields.  It does
//! not classify content or select an organism action.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const LOW_LEVEL_SENSORY_SCHEMA_V1: &str = "digital_cell_low_level_sensory_environment_v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LowLevelSensoryEnvironmentV1 {
    pub schema: String,
    pub visual_luminance: Vec<f64>,
    pub visual_motion: Vec<f64>,
    pub audio_amplitude: f64,
    pub audio_low_band: f64,
    pub audio_high_band: f64,
}

#[derive(Debug, Error, PartialEq)]
pub enum LowLevelSensoryError {
    #[error("camera frame dimensions or byte counts are invalid")]
    InvalidCameraFrame,
    #[error("microphone PCM sample stream is empty")]
    EmptyAudio,
}

/// Convert an RGB8 camera frame into ring-local horizontal luminance and
/// motion fields. Horizontal bins are an environmental coordinate supplied by
/// the camera, not a target direction.
pub fn camera_rgb8_fields(
    width: usize,
    height: usize,
    current: &[u8],
    previous: Option<&[u8]>,
    ring_sites: usize,
) -> Result<(Vec<f64>, Vec<f64>), LowLevelSensoryError> {
    let expected = width.saturating_mul(height).saturating_mul(3);
    if width == 0
        || height == 0
        || ring_sites == 0
        || current.len() != expected
        || previous.is_some_and(|frame| frame.len() != expected)
    {
        return Err(LowLevelSensoryError::InvalidCameraFrame);
    }
    let mut luminance = vec![0.0; ring_sites];
    let mut motion = vec![0.0; ring_sites];
    let mut counts = vec![0usize; ring_sites];
    for y in 0..height {
        for x in 0..width {
            let pixel = 3 * (y * width + x);
            let bin = (x * ring_sites / width).min(ring_sites - 1);
            let luma = (0.2126 * current[pixel] as f64
                + 0.7152 * current[pixel + 1] as f64
                + 0.0722 * current[pixel + 2] as f64)
                / 255.0;
            luminance[bin] += luma;
            if let Some(old) = previous {
                let old_luma = (0.2126 * old[pixel] as f64
                    + 0.7152 * old[pixel + 1] as f64
                    + 0.0722 * old[pixel + 2] as f64)
                    / 255.0;
                motion[bin] += (luma - old_luma).abs();
            }
            counts[bin] += 1;
        }
    }
    for index in 0..ring_sites {
        let count = counts[index].max(1) as f64;
        luminance[index] /= count;
        motion[index] /= count;
    }
    Ok((luminance, motion))
}

/// Convert signed PCM16 microphone samples into amplitude and two normalized
/// frequency-band energies. The split is the Nyquist midpoint, so no imported
/// biological frequency constant is introduced.
pub fn microphone_pcm16_fields(samples: &[i16]) -> Result<(f64, f64, f64), LowLevelSensoryError> {
    if samples.is_empty() {
        return Err(LowLevelSensoryError::EmptyAudio);
    }
    let normalized: Vec<f64> = samples
        .iter()
        .map(|sample| *sample as f64 / i16::MAX as f64)
        .collect();
    let amplitude =
        normalized.iter().map(|value| value.abs()).sum::<f64>() / normalized.len() as f64;
    let nyquist_bins = normalized.len() / 2;
    if nyquist_bins < 2 {
        return Ok((amplitude, 0.0, 0.0));
    }
    let split = nyquist_bins / 2;
    let mut low = 0.0;
    let mut high = 0.0;
    for k in 1..nyquist_bins {
        let mut real = 0.0;
        let mut imaginary = 0.0;
        for (index, value) in normalized.iter().enumerate() {
            let angle =
                2.0 * std::f64::consts::PI * k as f64 * index as f64 / normalized.len() as f64;
            real += value * angle.cos();
            imaginary -= value * angle.sin();
        }
        let energy =
            (real * real + imaginary * imaginary) / (normalized.len() * normalized.len()) as f64;
        if k <= split {
            low += energy;
        } else {
            high += energy;
        }
    }
    let total = low + high;
    if total > 0.0 {
        low /= total;
        high /= total;
    }
    Ok((amplitude, low, high))
}

impl LowLevelSensoryEnvironmentV1 {
    pub fn from_samples(
        visual_luminance: Vec<f64>,
        visual_motion: Vec<f64>,
        audio_amplitude: f64,
        audio_low_band: f64,
        audio_high_band: f64,
    ) -> Result<Self, LowLevelSensoryError> {
        if visual_luminance.is_empty()
            || visual_luminance.len() != visual_motion.len()
            || visual_luminance
                .iter()
                .chain(&visual_motion)
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || [audio_amplitude, audio_low_band, audio_high_band]
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(LowLevelSensoryError::InvalidCameraFrame);
        }
        Ok(Self {
            schema: LOW_LEVEL_SENSORY_SCHEMA_V1.to_string(),
            visual_luminance,
            visual_motion,
            audio_amplitude,
            audio_low_band,
            audio_high_band,
        })
    }

    /// Local, bounded sensory exposure used by organism-owned plasticity.
    pub fn local_exposure(&self) -> Vec<f64> {
        self.visual_luminance
            .iter()
            .zip(&self.visual_motion)
            .map(|(luminance, motion)| {
                luminance
                    .max(*motion)
                    .max(self.audio_amplitude)
                    .max(self.audio_low_band)
                    .max(self.audio_high_band)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_reports_luminance_and_motion_without_semantics() {
        let dark = vec![0u8; 4 * 2 * 3];
        let mut current = dark.clone();
        for y in 0..2 {
            for x in 0..2 {
                current[3 * (y * 4 + x)..3 * (y * 4 + x + 1)].fill(255);
            }
        }
        let (luminance, motion) = camera_rgb8_fields(4, 2, &current, Some(&dark), 2).unwrap();
        assert!(luminance[0] > luminance[1]);
        assert!(motion[0] > motion[1]);
    }

    #[test]
    fn microphone_reports_amplitude_and_normalized_bands() {
        let samples: Vec<i16> = (0..32)
            .map(|index| {
                (10_000.0 * (2.0 * std::f64::consts::PI * index as f64 / 32.0).sin()) as i16
            })
            .collect();
        let (amplitude, low, high) = microphone_pcm16_fields(&samples).unwrap();
        assert!(amplitude > 0.0);
        assert!((low + high - 1.0).abs() < 1e-12);
    }
}
