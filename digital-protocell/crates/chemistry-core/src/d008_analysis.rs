//! D-008 Stage B calibration helpers.

use crate::config::SimParams;
use crate::membrane::{membrane_basis, membrane_losses};

pub const MEMBRANE_CANDIDATE_FACTORS: [f64; 3] = [0.75, 1.0, 1.25];

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct MembraneCalibration {
    pub production_basis: f64,
    pub loss: f64,
    pub k_required: f64,
}

pub fn membrane_calibration(
    phi: &[f64],
    catalyst: &[f64],
    activated: &[f64],
    membrane: &[f64],
    dish_mask: &[bool],
    params: &SimParams,
) -> MembraneCalibration {
    let mut production_basis = 0.0;
    let mut loss = 0.0;
    for idx in 0..membrane.len() {
        if dish_mask[idx] {
            production_basis += membrane_basis(
                phi[idx],
                catalyst[idx],
                activated[idx],
                membrane[idx],
                params,
            );
            loss += membrane_losses(phi[idx], membrane[idx], params);
        }
    }
    MembraneCalibration {
        production_basis,
        loss,
        k_required: loss / production_basis.max(f64::EPSILON),
    }
}

pub fn membrane_candidates(k_required: f64) -> [f64; 3] {
    MEMBRANE_CANDIDATE_FACTORS.map(|factor| factor * k_required)
}
