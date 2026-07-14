//! D-008 observer-only membrane localization diagnostics.

use crate::grid::Grid;
use crate::reactions::interface_weight;

pub const LOCALIZATION_INTERFACE_THRESHOLD: f64 = 0.25;

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct MembranePartition {
    pub total_mass: f64,
    pub interface_mass: f64,
    pub interior_mass: f64,
    pub exterior_mass: f64,
    pub localization_fraction: f64,
}

pub fn membrane_partition(grid: &Grid, phi: &[f64], membrane: &[f64]) -> MembranePartition {
    let mut partition = MembranePartition::default();
    for idx in 0..membrane.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let mass = membrane[idx];
        partition.total_mass += mass;
        if interface_weight(phi[idx]) >= LOCALIZATION_INTERFACE_THRESHOLD {
            partition.interface_mass += mass;
        } else if phi[idx] >= 0.5 {
            partition.interior_mass += mass;
        } else {
            partition.exterior_mass += mass;
        }
    }
    partition.localization_fraction =
        partition.interface_mass / partition.total_mass.max(f64::EPSILON);
    partition
}
