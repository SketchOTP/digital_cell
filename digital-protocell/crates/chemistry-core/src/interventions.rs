//! Chemistry perturbation interventions — observer-only actions on fields.

use crate::config::{InterventionAction, SimParams};
use crate::fields::FieldBuffers;
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WoundRegion {
    pub mask: Vec<bool>,
    pub reference_ring: Vec<bool>,
    pub local_structure_before: f64,
    pub local_catalyst_before: f64,
}

impl WoundRegion {
    pub fn local_structure_mass(&self, grid: &Grid, structure: &[f64]) -> f64 {
        local_mass(grid, structure, &self.mask)
    }

    pub fn local_catalyst_mass(&self, grid: &Grid, catalyst: &[f64]) -> f64 {
        local_mass(grid, catalyst, &self.mask)
    }

    pub fn recovery_ratios(&self, grid: &Grid, fields: &FieldBuffers) -> (f64, f64) {
        let s = self.local_structure_mass(grid, &fields.structure)
            / self.local_structure_before.max(1e-12);
        let c = self.local_catalyst_mass(grid, &fields.catalyst)
            / self.local_catalyst_before.max(1e-12);
        (s, c)
    }
}

pub fn apply_intervention(
    grid: &Grid,
    fields: &mut FieldBuffers,
    action: &InterventionAction,
    params: &mut SimParams,
) {
    match action {
        InterventionAction::RemoveNutrient => {
            params.n_reservoir = 0.0;
        }
        InterventionAction::RemoveFuel => {
            params.f_reservoir = 0.0;
        }
        InterventionAction::DisableCatalystReproduction => {
            params.k_rep = 0.0;
        }
        InterventionAction::DisableStructuralSynthesis => {
            params.k_structure = 0.0;
        }
        InterventionAction::RestoreReservoir => {
            params.n_reservoir = 1.0;
            params.f_reservoir = 1.0;
            params.w_reservoir = 0.0;
        }
        InterventionAction::ShutdownReservoir => {
            params.reservoir_rate = 0.0;
        }
        InterventionAction::DisableAllReactions => {
            params.reactions_enabled = false;
            params.k_rep = 0.0;
            params.k_structure = 0.0;
        }
        InterventionAction::PunctureRepair => {
            puncture_wedge(grid, fields, params.seed_r0, 25.0, 0.90);
        }
        InterventionAction::CatastrophicDamage => {
            remove_fraction_global(grid, &mut fields.structure, 0.70);
            remove_fraction_global(grid, &mut fields.catalyst, 0.70);
        }
        InterventionAction::DamageFraction { fraction } => {
            remove_fraction_wedge(grid, fields, params.seed_r0, fraction * 100.0);
        }
    }
}

pub fn define_wound_region(grid: &Grid, r0: f64, angle_deg: f64) -> WoundRegion {
    let half_angle = angle_deg.to_radians() / 2.0;
    let r_inner = r0 - 5.0;
    let r_outer = r0 + 8.0;
    let r_ref_inner = r_outer;
    let r_ref_outer = r_outer + 6.0;
    let n = grid.width * grid.height;
    let mut mask = vec![false; n];
    let mut reference_ring = vec![false; n];

    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx);
            if r >= r_inner && r <= r_outer && theta.abs() <= half_angle {
                mask[idx] = true;
            }
            if r >= r_ref_inner && r <= r_ref_outer && theta.abs() <= half_angle {
                reference_ring[idx] = true;
            }
        }
    }

    WoundRegion {
        mask,
        reference_ring,
        local_structure_before: 0.0,
        local_catalyst_before: 0.0,
    }
}

pub fn capture_wound_baseline(wound: &mut WoundRegion, grid: &Grid, fields: &FieldBuffers) {
    wound.local_structure_before = local_mass(grid, &fields.structure, &wound.mask);
    wound.local_catalyst_before = local_mass(grid, &fields.catalyst, &wound.mask);
}

fn puncture_wedge(
    grid: &Grid,
    fields: &mut FieldBuffers,
    r0: f64,
    angle_deg: f64,
    removal_fraction: f64,
) {
    let half_angle = angle_deg.to_radians() / 2.0;
    let r_inner = r0 - 5.0;
    let r_outer = r0 + 8.0;
    let keep = 1.0 - removal_fraction;

    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx);
            if r >= r_inner && r <= r_outer && theta.abs() <= half_angle {
                fields.structure[idx] *= keep;
                fields.catalyst[idx] *= keep;
            }
        }
    }
}

fn remove_fraction_global(grid: &Grid, field: &mut [f64], fraction: f64) {
    let keep = 1.0 - fraction;
    for idx in 0..grid.width * grid.height {
        if grid.in_dish(idx) {
            field[idx] *= keep;
        }
    }
}

fn remove_fraction_wedge(grid: &Grid, fields: &mut FieldBuffers, r0: f64, fraction_pct: f64) {
    let keep = 1.0 - (fraction_pct / 100.0).clamp(0.0, 1.0);
    let half_angle = std::f64::consts::PI / 4.0;
    let r_max = r0 + 12.0;

    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx);
            if r <= r_max && theta.abs() <= half_angle {
                fields.structure[idx] *= keep;
                fields.catalyst[idx] *= keep;
            }
        }
    }
}

fn local_mass(grid: &Grid, field: &[f64], mask: &[bool]) -> f64 {
    mask.iter()
        .enumerate()
        .filter(|(idx, &m)| m && grid.in_dish(*idx))
        .map(|(idx, _)| field[idx])
        .sum()
}

// ponytail: wedge aligned to +x axis; upgrade path is configurable intervention geometry
