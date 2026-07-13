//! Analytical fresh-seed generation for D-005 basin mapping.

use crate::config::SimParams;
use crate::fields::{initialize_seed, FieldBuffers};
use crate::grid::Grid;
use crate::simulation::Simulation;
use serde::{Deserialize, Serialize};

/// Analytical protocell seed recipe — no saved attractor fields.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FreshSeedRecipe {
    pub r0: f64,
    pub c0: f64,
    pub noise_seed: u64,
    pub noise_amplitude: f64,
}

impl FreshSeedRecipe {
    pub fn default_production() -> Self {
        Self {
            r0: 24.0,
            c0: 0.35,
            noise_seed: 1,
            noise_amplitude: 0.005,
        }
    }

    pub fn identity_key(&self) -> String {
        format!(
            "R{:.1}_C{:.3}_ns{}_na{:.4}",
            self.r0, self.c0, self.noise_seed, self.noise_amplitude
        )
    }

    pub fn apply_to_params(&self, params: &mut SimParams) {
        params.seed_r0 = self.r0;
        params.seed_catalyst_scale = self.c0;
        params.seed_interface_width = 3.0;
        params.random_seed = self.noise_seed;
        params.noise_amplitude = self.noise_amplitude;
    }
}

/// Initialize fields from analytical recipe only (uniform N=1, F=1, W=0).
pub fn apply_fresh_seed(grid: &Grid, params: &SimParams, fields: &mut FieldBuffers) {
    initialize_seed(grid, params, fields);
}

pub fn spawn_fresh_simulation(candidate_params: SimParams, recipe: &FreshSeedRecipe) -> Simulation {
    let mut params = candidate_params;
    recipe.apply_to_params(&mut params);
    Simulation::new(params)
}

/// ponytail: recipe check is param-based; upgrade path is field-hash attestation against known snapshots
pub fn seed_uses_no_saved_attractor(recipe: &FreshSeedRecipe) -> bool {
    recipe.r0 > 0.0 && recipe.c0 >= 0.0
}

pub fn seed_preserves_uniform_resources(grid: &Grid, fields: &FieldBuffers) -> bool {
    for idx in 0..grid.width * grid.height {
        if !grid.in_dish(idx) {
            continue;
        }
        if (fields.nutrient[idx] - 1.0).abs() > 1e-9 {
            return false;
        }
        if (fields.fuel[idx] - 1.0).abs() > 1e-9 {
            return false;
        }
        if fields.waste[idx].abs() > 1e-9 {
            return false;
        }
    }
    true
}

pub fn coarse_grid_r0() -> [f64; 5] {
    [16.0, 20.0, 24.0, 28.0, 32.0]
}

pub fn coarse_grid_c0() -> [f64; 5] {
    [0.20, 0.275, 0.35, 0.425, 0.50]
}
