//! Chemistry perturbation interventions — observer-only actions on fields.

use crate::config::{InterventionAction, SimParams};
use crate::fields::FieldBuffers;
use crate::grid::Grid;

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
        InterventionAction::RestoreReservoir => {
            params.n_reservoir = 1.0;
            params.f_reservoir = 1.0;
            params.w_reservoir = 0.0;
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
            remove_fraction(grid, &mut fields.structure, 0.70);
            remove_fraction(grid, &mut fields.catalyst, 0.70);
        }
    }
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

fn remove_fraction(grid: &Grid, field: &mut [f64], fraction: f64) {
    let keep = 1.0 - fraction;
    for idx in 0..grid.width * grid.height {
        if grid.in_dish(idx) {
            field[idx] *= keep;
        }
    }
}

// ponytail: wedge aligned to +x axis; upgrade path is configurable intervention geometry
