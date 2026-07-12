//! Environmental reservoir relaxation in dish annulus.

use crate::config::SimParams;
use crate::grid::Grid;

pub fn apply_reservoir(
    grid: &Grid,
    nutrient: &mut [f64],
    fuel: &mut [f64],
    waste: &mut [f64],
    dt: f64,
    params: &SimParams,
) {
    let rate = params.reservoir_rate * dt;
    for idx in 0..grid.width * grid.height {
        if !grid.reservoir_mask[idx] {
            continue;
        }
        nutrient[idx] += rate * (params.n_reservoir - nutrient[idx]);
        fuel[idx] += rate * (params.f_reservoir - fuel[idx]);
        waste[idx] += rate * (params.w_reservoir - waste[idx]);
    }
}
