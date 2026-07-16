//! Environmental reservoir relaxation in dish annulus.

use crate::config::SimParams;
use crate::grid::Grid;

#[inline]
pub fn waste_sink_cell(grid: &Grid, idx: usize, params: &SimParams) -> bool {
    if !grid.in_dish(idx) {
        return false;
    }
    let i = idx % grid.width;
    let j = idx / grid.width;
    grid.distance_from_center(i, j) >= params.waste_sink_inner_radius
}

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
        if grid.reservoir_mask[idx] {
            nutrient[idx] += rate * (params.n_reservoir - nutrient[idx]);
            fuel[idx] += rate * (params.f_reservoir - fuel[idx]);
        }
        if waste_sink_cell(grid, idx, params) {
            waste[idx] += rate * (params.w_reservoir - waste[idx]);
        }
    }
}
