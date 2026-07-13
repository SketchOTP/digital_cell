//! Flat scalar field buffers with double buffering.

use crate::config::{CONC_SAFETY_LIMIT, NEG_CLAMP, PHI_HARD_MAX, PHI_HARD_MIN};
use crate::grid::Grid;

#[derive(Debug, Clone)]
pub struct FieldBuffers {
    pub structure: Vec<f64>,
    pub catalyst: Vec<f64>,
    pub nutrient: Vec<f64>,
    pub fuel: Vec<f64>,
    pub waste: Vec<f64>,
    pub structure_next: Vec<f64>,
    pub catalyst_next: Vec<f64>,
    pub nutrient_next: Vec<f64>,
    pub fuel_next: Vec<f64>,
    pub waste_next: Vec<f64>,
    /// scratch: h(phi), laplacian, mu, laplacian_mu, reaction scratch
    pub scratch_h: Vec<f64>,
    pub scratch_lap: Vec<f64>,
    pub scratch_mu: Vec<f64>,
    pub scratch_lap_mu: Vec<f64>,
    pub scratch_flux_x: Vec<f64>,
    pub scratch_flux_y: Vec<f64>,
    pub scratch_fuel_diff: Vec<f64>,
    pub scratch_waste_diff: Vec<f64>,
}

impl FieldBuffers {
    pub fn new(size: usize) -> Self {
        let zero = || vec![0.0; size];
        Self {
            structure: zero(),
            catalyst: zero(),
            nutrient: zero(),
            fuel: zero(),
            waste: zero(),
            structure_next: zero(),
            catalyst_next: zero(),
            nutrient_next: zero(),
            fuel_next: zero(),
            waste_next: zero(),
            scratch_h: zero(),
            scratch_lap: zero(),
            scratch_mu: zero(),
            scratch_lap_mu: zero(),
            scratch_flux_x: zero(),
            scratch_flux_y: zero(),
            scratch_fuel_diff: zero(),
            scratch_waste_diff: zero(),
        }
    }

    pub fn for_grid(grid: &Grid) -> Self {
        Self::new(grid.width * grid.height)
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.structure, &mut self.structure_next);
        std::mem::swap(&mut self.catalyst, &mut self.catalyst_next);
        std::mem::swap(&mut self.nutrient, &mut self.nutrient_next);
        std::mem::swap(&mut self.fuel, &mut self.fuel_next);
        std::mem::swap(&mut self.waste, &mut self.waste_next);
    }

    pub fn copy_current_to_working(&self, working: &mut FieldBuffers) {
        working.structure.copy_from_slice(&self.structure);
        working.catalyst.copy_from_slice(&self.catalyst);
        working.nutrient.copy_from_slice(&self.nutrient);
        working.fuel.copy_from_slice(&self.fuel);
        working.waste.copy_from_slice(&self.waste);
    }
}

pub fn clamp_small_negative(v: f64) -> f64 {
    if v >= NEG_CLAMP {
        v.max(0.0)
    } else {
        v
    }
}

pub fn validate_structure_field(values: &[f64], dish_mask: &[bool]) -> Result<(), String> {
    for (idx, &v) in values.iter().enumerate() {
        if !dish_mask[idx] {
            continue;
        }
        if !v.is_finite() {
            return Err(format!("non-finite structure at {idx}: {v}"));
        }
        if v < PHI_HARD_MIN {
            return Err(format!("structure below hard min at {idx}: {v}"));
        }
        if v > PHI_HARD_MAX {
            return Err(format!("structure above hard max at {idx}: {v}"));
        }
    }
    Ok(())
}

pub fn validate_soluble_field(values: &[f64], dish_mask: &[bool]) -> Result<(), String> {
    for (idx, &v) in values.iter().enumerate() {
        if !dish_mask[idx] {
            continue;
        }
        if !v.is_finite() {
            return Err(format!("non-finite value at {idx}: {v}"));
        }
        if v < NEG_CLAMP {
            return Err(format!("excessive negative at {idx}: {v}"));
        }
        if v > CONC_SAFETY_LIMIT {
            return Err(format!("excessive concentration at {idx}: {v}"));
        }
    }
    Ok(())
}

pub fn validate_field(values: &[f64], dish_mask: &[bool]) -> Result<(), String> {
    validate_soluble_field(values, dish_mask)
}

pub fn field_sha256(field: &[f64]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for v in field {
        v.to_bits().hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

pub fn structure_field_stats(values: &[f64], dish_mask: &[bool]) -> (f64, f64, f64, f64) {
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    let mut above_one = 0u64;
    let mut below_zero = 0u64;
    let mut n = 0u64;
    for (idx, &v) in values.iter().enumerate() {
        if !dish_mask[idx] {
            continue;
        }
        n += 1;
        min_v = min_v.min(v);
        max_v = max_v.max(v);
        if v > 1.0 {
            above_one += 1;
        }
        if v < 0.0 {
            below_zero += 1;
        }
    }
    if n == 0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    (
        min_v,
        max_v,
        above_one as f64 / n as f64,
        below_zero as f64 / n as f64,
    )
}

/// Deterministic xorshift64 PRNG for seed noise.
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn next_f64(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x as f64) / (u64::MAX as f64)
    }

    pub fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }
}

pub fn initialize_seed(
    grid: &Grid,
    params: &crate::config::SimParams,
    fields: &mut FieldBuffers,
) {
    let n = grid.width * grid.height;
    assert_eq!(fields.structure.len(), n);
    let mut rng = DeterministicRng::new(params.random_seed);

    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let r = grid.distance_from_center(i, j);
            let phi = 0.5 * (1.0 - ((r - params.seed_r0) / params.seed_interface_width).tanh());
            let noise = rng.uniform(-params.noise_amplitude, params.noise_amplitude);
            let phi = (phi + noise).clamp(0.0, 1.0);
            fields.structure[idx] = phi;
            let h = interior_weight(phi);
            fields.catalyst[idx] = params.seed_catalyst_scale * h;
            fields.nutrient[idx] = 1.0;
            fields.fuel[idx] = 1.0;
            fields.waste[idx] = 0.0;
        }
    }
}

#[inline]
pub fn interior_weight(phi: f64) -> f64 {
    let p = phi.clamp(0.0, 1.0);
    p * p * (3.0 - 2.0 * p)
}

pub const FIELD_NAMES: [&str; 5] = ["structure", "catalyst", "nutrient", "fuel", "waste"];

pub fn field_slice<'a>(fields: &'a FieldBuffers, name: &str) -> Option<&'a [f64]> {
    match name {
        "structure" => Some(&fields.structure),
        "catalyst" => Some(&fields.catalyst),
        "nutrient" => Some(&fields.nutrient),
        "fuel" => Some(&fields.fuel),
        "waste" => Some(&fields.waste),
        _ => None,
    }
}

// ponytail: grid size fixed at compile-time constants; upgrade path is dynamic Grid allocation
