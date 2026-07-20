//! D-008 Stage B calibration and Stage E prescribed-radius balance analysis.

use crate::activated_metabolism::activated_metabolism_rates;
use crate::config::{SimParams, DX, GRID_HEIGHT, GRID_WIDTH};
use crate::membrane::{membrane_basis, membrane_losses, membrane_rates};
use crate::reactions::interface_weight;

pub const MEMBRANE_CANDIDATE_FACTORS: [f64; 3] = [0.75, 1.0, 1.25];
pub const STAGE_E_CALIBRATION_FACTORS: [f64; 3] = [0.8, 1.0, 1.2];
pub const STAGE_E_INTERFACE_WIDTH: f64 = 2.0;
pub const STAGE_E_ZERO_FLOW_TOL: f64 = 1e-3;
pub const STAGE_E_RADIUS_OVERLAP_MAX: f64 = 6.0;

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

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct StructureCalibration {
    pub production_basis: f64,
    pub loss: f64,
    pub k_required: f64,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PrescribedInterior {
    pub catalyst: f64,
    pub nutrient: f64,
    pub fuel: f64,
    pub activated: f64,
    pub waste: f64,
    pub membrane_scale: f64,
}

impl Default for PrescribedInterior {
    fn default() -> Self {
        Self {
            catalyst: 0.4,
            nutrient: 0.2,
            fuel: 0.2,
            activated: 0.2,
            waste: 0.5,
            membrane_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct PrescribedBalancePoint {
    pub radius: f64,
    pub d_structure: f64,
    pub d_catalyst: f64,
    pub d_membrane: f64,
    pub d_activated: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageECalibrationParameter {
    MembraneProduction,
    Activation,
    Reproduction,
    StructureProduction,
}

pub fn stage_e_default_radii() -> Vec<f64> {
    (12..=36).step_by(2).map(|r| r as f64).collect()
}

fn circular_phi(x: f64, y: f64, cx: f64, cy: f64, radius: f64) -> f64 {
    let distance = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    0.5 * (1.0 - ((distance - radius) / STAGE_E_INTERFACE_WIDTH).tanh())
}

pub fn structure_calibration(
    params: &SimParams,
    radius: f64,
    interior: &PrescribedInterior,
) -> StructureCalibration {
    let cx = (GRID_WIDTH as f64) * 0.5;
    let cy = (GRID_HEIGHT as f64) * 0.5;
    let mut production_basis = 0.0;
    let mut loss = 0.0;
    let cell_area = DX * DX;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let phi = circular_phi(x as f64 + 0.5, y as f64 + 0.5, cx, cy, radius);
            production_basis +=
                interior.activated * interface_weight(phi) * cell_area;
            loss += params.k_structure_decay * phi * cell_area;
        }
    }
    StructureCalibration {
        production_basis,
        loss,
        k_required: loss / production_basis.max(f64::EPSILON),
    }
}

pub fn structure_candidates(k_required: f64) -> [f64; 3] {
    STAGE_E_CALIBRATION_FACTORS.map(|factor| factor * k_required)
}

pub fn prescribed_balance_point(
    params: &SimParams,
    radius: f64,
    interior: &PrescribedInterior,
) -> PrescribedBalancePoint {
    let cx = (GRID_WIDTH as f64) * 0.5;
    let cy = (GRID_HEIGHT as f64) * 0.5;
    let cell_area = DX * DX;
    let mut d_structure = 0.0;
    let mut d_catalyst = 0.0;
    let mut d_membrane = 0.0;
    let mut d_activated = 0.0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let phi = circular_phi(x as f64 + 0.5, y as f64 + 0.5, cx, cy, radius);
            let i_face = interface_weight(phi);
            let (c, n, f, a, _w, m) = if phi >= 0.5 {
                (
                    interior.catalyst,
                    interior.nutrient,
                    interior.fuel,
                    interior.activated,
                    interior.waste,
                    i_face * interior.membrane_scale,
                )
            } else {
                (0.0, 0.0, 0.0, 0.0, 0.0, i_face * interior.membrane_scale)
            };
            d_structure += (params.k_d008_structure * a * i_face - params.k_structure_decay * phi)
                * cell_area;
            let rates = activated_metabolism_rates(1.0, c, n, f, a, params);
            if phi >= 0.5 {
                d_catalyst += rates.d_catalyst * cell_area;
                d_activated += rates.d_activated * cell_area;
            }
            let membrane = membrane_rates(phi, c, a, m, params);
            d_membrane += membrane.net() * cell_area;
        }
    }
    PrescribedBalancePoint {
        radius,
        d_structure,
        d_catalyst,
        d_membrane,
        d_activated,
    }
}

pub fn prescribed_radius_sweep(
    params: &SimParams,
    radii: &[f64],
    interior: &PrescribedInterior,
) -> Vec<PrescribedBalancePoint> {
    radii
        .iter()
        .copied()
        .map(|radius| prescribed_balance_point(params, radius, interior))
        .collect()
}

pub fn simultaneous_zero_flow_near(point: &PrescribedBalancePoint) -> bool {
    point.d_structure.abs() <= STAGE_E_ZERO_FLOW_TOL
        && point.d_catalyst.abs() <= STAGE_E_ZERO_FLOW_TOL
        && point.d_membrane.abs() <= STAGE_E_ZERO_FLOW_TOL
        && point.d_activated.abs() <= STAGE_E_ZERO_FLOW_TOL
}

pub fn zero_crossing_radii(radii: &[f64], values: &[f64]) -> Vec<f64> {
    let mut crossings = Vec::new();
    if radii.len() < 2 || radii.len() != values.len() {
        return crossings;
    }
    for idx in 1..radii.len() {
        let prev = values[idx - 1];
        let curr = values[idx];
        if prev.signum() != curr.signum() && prev.abs() > f64::EPSILON && curr.abs() > f64::EPSILON {
            let t = prev.abs() / (prev.abs() + curr.abs());
            crossings.push(radii[idx - 1] + t * (radii[idx] - radii[idx - 1]));
        }
    }
    crossings
}

pub fn crossing_overlap_max(crossings: &[Vec<f64>]) -> Option<f64> {
    if crossings.iter().any(|c| c.is_empty()) {
        return None;
    }
    let mut joint_lo = f64::NEG_INFINITY;
    let mut joint_hi = f64::INFINITY;
    for field_crossings in crossings {
        let lo = field_crossings.iter().copied().reduce(f64::min)?;
        let hi = field_crossings.iter().copied().reduce(f64::max)?;
        joint_lo = joint_lo.max(lo);
        joint_hi = joint_hi.min(hi);
    }
    if joint_lo <= joint_hi {
        Some(joint_hi - joint_lo)
    } else {
        None
    }
}

pub fn stage_e_default_activated_levels() -> Vec<f64> {
    (1..=10).map(|step| step as f64 * 0.05).collect()
}

pub fn prescribed_balance_grid(
    params: &SimParams,
    radii: &[f64],
    activated_levels: &[f64],
    interior: &PrescribedInterior,
) -> Vec<PrescribedBalancePoint> {
    let mut points = Vec::new();
    for &radius in radii {
        for &activated in activated_levels {
            let mut trial = *interior;
            trial.activated = activated;
            points.push(prescribed_balance_point(params, radius, &trial));
        }
    }
    points
}

pub fn joint_zero_flow_overlap_2d(
    params: &SimParams,
    radii: &[f64],
    activated_levels: &[f64],
    interior: &PrescribedInterior,
) -> bool {
    let grid = prescribed_balance_grid(params, radii, activated_levels, interior);
    if grid.iter().any(simultaneous_zero_flow_near) {
        return true;
    }
    for &activated in activated_levels {
        let mut trial = *interior;
        trial.activated = activated;
        let sweep = prescribed_radius_sweep(params, radii, &trial);
        if joint_zero_flow_overlap(&sweep) {
            return true;
        }
    }
    false
}

pub fn joint_zero_flow_overlap(points: &[PrescribedBalancePoint]) -> bool {
    if points.iter().any(simultaneous_zero_flow_near) {
        return true;
    }
    let radii: Vec<f64> = points.iter().map(|p| p.radius).collect();
    let crossings = [
        zero_crossing_radii(&radii, &points.iter().map(|p| p.d_structure).collect::<Vec<_>>()),
        zero_crossing_radii(&radii, &points.iter().map(|p| p.d_catalyst).collect::<Vec<_>>()),
        zero_crossing_radii(&radii, &points.iter().map(|p| p.d_membrane).collect::<Vec<_>>()),
        zero_crossing_radii(&radii, &points.iter().map(|p| p.d_activated).collect::<Vec<_>>()),
    ];
    if crossings.iter().any(|c| c.is_empty()) {
        return false;
    }
    crossing_overlap_max(&crossings.to_vec())
        .map(|span| span <= STAGE_E_RADIUS_OVERLAP_MAX)
        .unwrap_or(false)
}

pub fn balance_score(points: &[PrescribedBalancePoint]) -> f64 {
    points
        .iter()
        .map(|point| {
            [
                point.d_structure,
                point.d_catalyst,
                point.d_membrane,
                point.d_activated,
            ]
            .into_iter()
            .map(|v| v * v)
            .sum::<f64>()
        })
        .fold(f64::INFINITY, f64::min)
}

pub fn select_stage_e_factor(
    baseline: f64,
    factors: [f64; 3],
    scores: [f64; 3],
    overlaps: [bool; 3],
) -> (f64, f64) {
    let mut best_idx = 1;
    for idx in 0..3 {
        let best_overlap = overlaps[best_idx];
        let candidate_overlap = overlaps[idx];
        if candidate_overlap && !best_overlap {
            best_idx = idx;
            continue;
        }
        if candidate_overlap != best_overlap {
            continue;
        }
        if scores[idx] < scores[best_idx] - f64::EPSILON {
            best_idx = idx;
        } else if (scores[idx] - scores[best_idx]).abs() <= f64::EPSILON
            && (factors[idx] - 1.0).abs() < (factors[best_idx] - 1.0).abs()
        {
            best_idx = idx;
        }
    }
    (baseline * factors[best_idx], factors[best_idx])
}
