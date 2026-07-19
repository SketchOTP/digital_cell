//! Chemistry perturbation interventions — observer-only actions on fields.

use crate::config::{InterventionAction, SimParams};
use crate::fields::FieldBuffers;
use crate::grid::Grid;
use crate::reactions::interface_weight;
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

/// Result of a declared membrane S→W damage intervention (D-039).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembraneArcDamageReport {
    pub fraction_requested: f64,
    pub total_s_before: f64,
    pub s_removed: f64,
    pub w_gained: f64,
    pub cells_touched: usize,
    pub arc_half_angle_rad: f64,
    pub local_occupancy_before: f64,
    pub local_occupancy_after: f64,
}

/// Convert a contiguous interface-arc fraction of total S into W.
///
/// Geometry is selected only at the intervention moment. Does not reseed S,
/// normalize coverage, change rates, or invoke a repair controller.
pub fn apply_declared_membrane_arc_damage(
    grid: &Grid,
    fields: &mut FieldBuffers,
    fraction_of_total_s: f64,
) -> MembraneArcDamageReport {
    let fraction = fraction_of_total_s.clamp(0.0, 1.0);
    let n = grid.width * grid.height;
    let mut total_s = 0.0;
    let mut interface_cells: Vec<(usize, f64, f64)> = Vec::new(); // idx, s, theta
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let s = fields.membrane[idx].max(0.0);
            total_s += s;
            let phi = fields.structure[idx];
            let iw = interface_weight(phi);
            if iw >= 0.25 && s > 0.0 {
                let dx = i as f64 - grid.cx;
                let dy = j as f64 - grid.cy;
                let theta = dy.atan2(dx);
                interface_cells.push((idx, s, theta));
            }
        }
    }
    let target = fraction * total_s;
    // Contiguous arc about +x (θ≈0): grow by |θ| until enough S covered.
    interface_cells.sort_by(|a, b| {
        a.2.abs()
            .partial_cmp(&b.2.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut covered = 0.0_f64;
    let mut half_angle = 0.0_f64;
    let mut mask = vec![false; n];
    for &(idx, s, theta) in &interface_cells {
        if covered >= target && target > 0.0 {
            break;
        }
        mask[idx] = true;
        covered += s;
        half_angle = half_angle.max(theta.abs());
    }
    let arc_s: f64 = mask
        .iter()
        .enumerate()
        .filter(|(idx, &m)| m && grid.in_dish(*idx))
        .map(|(idx, _)| fields.membrane[idx].max(0.0))
        .sum();
    let remove_scale = if arc_s > 0.0 {
        (target / arc_s).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut local_before = 0.0;
    let mut cells_touched = 0usize;
    let mut s_removed = 0.0;
    for idx in 0..n {
        if !mask[idx] || !grid.in_dish(idx) {
            continue;
        }
        let s0 = fields.membrane[idx].max(0.0);
        local_before += s0;
        let rem = s0 * remove_scale;
        if rem > 0.0 {
            fields.membrane[idx] = (s0 - rem).max(0.0);
            fields.waste[idx] += rem;
            s_removed += rem;
            cells_touched += 1;
        }
    }
    let local_after = local_before - s_removed;

    MembraneArcDamageReport {
        fraction_requested: fraction,
        total_s_before: total_s,
        s_removed,
        w_gained: s_removed,
        cells_touched,
        arc_half_angle_rad: half_angle,
        local_occupancy_before: local_before,
        local_occupancy_after: local_after.max(0.0),
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
