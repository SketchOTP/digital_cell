//! Initial seed audit and field hashing.

use crate::fields::{field_sha256, FieldBuffers};
use crate::grid::Grid;
use crate::operators::total_mass;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedAuditReport {
    pub seed: u64,
    pub structure_hash: String,
    pub catalyst_hash: String,
    pub nutrient_hash: String,
    pub fuel_hash: String,
    pub waste_hash: String,
    pub structural_mass: f64,
    pub catalyst_mass: f64,
    pub nutrient_mass: f64,
    pub fuel_mass: f64,
    pub waste_mass: f64,
    pub center_of_mass: (f64, f64),
    pub radial_symmetry_error: f64,
    pub max_perturbation: f64,
    pub min_perturbation: f64,
    pub mean_perturbation: f64,
    pub perturbation_variance: f64,
}

pub fn audit_initial_seed(grid: &Grid, fields: &FieldBuffers, seed: u64, seed_r0: f64) -> SeedAuditReport {
    let mut com_x = 0.0;
    let mut com_y = 0.0;
    let mut mass_phi = 0.0;
    let mut perturbations = Vec::new();
    let mut symmetry_err = 0.0;
    let mut symmetry_n = 0u64;

    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let phi = fields.structure[idx];
            let r = grid.distance_from_center(i, j);
            let phi0 = 0.5 * (1.0 - ((r - seed_r0) / 3.0).tanh());
            perturbations.push(phi - phi0);
            com_x += i as f64 * phi;
            com_y += j as f64 * phi;
            mass_phi += phi;

            let mirror_i = (2.0 * grid.cx - i as f64).round() as usize;
            if mirror_i < grid.width {
                let midx = Grid::index(grid.width, mirror_i, j);
                if grid.in_dish(midx) {
                    symmetry_err += (phi - fields.structure[midx]).abs();
                    symmetry_n += 1;
                }
            }
        }
    }

    let (cx, cy) = if mass_phi > 0.0 {
        (com_x / mass_phi, com_y / mass_phi)
    } else {
        (grid.cx, grid.cy)
    };

    let mean_p = if perturbations.is_empty() {
        0.0
    } else {
        perturbations.iter().sum::<f64>() / perturbations.len() as f64
    };
    let var_p = if perturbations.is_empty() {
        0.0
    } else {
        perturbations.iter().map(|p| (p - mean_p).powi(2)).sum::<f64>() / perturbations.len() as f64
    };

    SeedAuditReport {
        seed,
        structure_hash: field_sha256(&fields.structure),
        catalyst_hash: field_sha256(&fields.catalyst),
        nutrient_hash: field_sha256(&fields.nutrient),
        fuel_hash: field_sha256(&fields.fuel),
        waste_hash: field_sha256(&fields.waste),
        structural_mass: total_mass(grid, &fields.structure),
        catalyst_mass: total_mass(grid, &fields.catalyst),
        nutrient_mass: total_mass(grid, &fields.nutrient),
        fuel_mass: total_mass(grid, &fields.fuel),
        waste_mass: total_mass(grid, &fields.waste),
        center_of_mass: (cx, cy),
        radial_symmetry_error: if symmetry_n > 0 {
            symmetry_err / symmetry_n as f64
        } else {
            0.0
        },
        max_perturbation: perturbations.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        min_perturbation: perturbations.iter().copied().fold(f64::INFINITY, f64::min),
        mean_perturbation: mean_p,
        perturbation_variance: var_p,
    }
}
