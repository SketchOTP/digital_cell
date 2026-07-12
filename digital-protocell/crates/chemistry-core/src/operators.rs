//! Spatial differential operators with no-flux dish boundary.

use crate::config::DX;
use crate::grid::Grid;

/// Five-point Laplacian with no-flux boundary: outside neighbors mirror center.
pub fn laplacian(grid: &Grid, field: &[f64], out: &mut [f64]) {
    let w = grid.width;
    let h = grid.height;
    let inv_dx2 = 1.0 / (DX * DX);

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                out[idx] = 0.0;
                continue;
            }
            let c = field[idx];
            let left = neighbor_value(grid, field, i.wrapping_sub(1), j, i, j);
            let right = neighbor_value(grid, field, i + 1, j, i, j);
            let down = neighbor_value(grid, field, i, j.wrapping_sub(1), i, j);
            let up = neighbor_value(grid, field, i, j + 1, i, j);
            out[idx] = (left + right + down + up - 4.0 * c) * inv_dx2;
        }
    }
}

#[inline]
fn neighbor_value(
    grid: &Grid,
    field: &[f64],
    ni: usize,
    nj: usize,
    ci: usize,
    cj: usize,
) -> f64 {
    let w = grid.width;
    let h = grid.height;
    if ni >= w || nj >= h {
        return field[Grid::index(w, ci, cj)];
    }
    let nidx = Grid::index(w, ni, nj);
    if !grid.in_dish(nidx) {
        field[Grid::index(w, ci, cj)]
    } else {
        field[nidx]
    }
}

/// Variable-diffusivity diffusion via face fluxes: D_face = 0.5*(D_i + D_j).
pub fn diffuse_variable(
    grid: &Grid,
    field: &[f64],
    diffusivity: &[f64],
    out_rate: &mut [f64],
) {
    let w = grid.width;
    let h = grid.height;
    let inv_dx = 1.0 / DX;
    out_rate.fill(0.0);

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let d_c = diffusivity[idx];
            let c = field[idx];
            let mut flux_sum = 0.0;

            // +x face
            if i + 1 < w {
                let nidx = Grid::index(w, i + 1, j);
                let (n_val, d_n) = if grid.in_dish(nidx) {
                    (field[nidx], diffusivity[nidx])
                } else {
                    (c, d_c)
                };
                let d_face = 0.5 * (d_c + d_n);
                flux_sum += d_face * (n_val - c) * inv_dx;
            }
            // -x face
            if i > 0 {
                let nidx = Grid::index(w, i - 1, j);
                let (n_val, d_n) = if grid.in_dish(nidx) {
                    (field[nidx], diffusivity[nidx])
                } else {
                    (c, d_c)
                };
                let d_face = 0.5 * (d_c + d_n);
                flux_sum += d_face * (c - n_val) * inv_dx;
            } else {
                // no-flux at wall: neighbor mirrors center, flux zero
            }
            // +y face
            if j + 1 < h {
                let nidx = Grid::index(w, i, j + 1);
                let (n_val, d_n) = if grid.in_dish(nidx) {
                    (field[nidx], diffusivity[nidx])
                } else {
                    (c, d_c)
                };
                let d_face = 0.5 * (d_c + d_n);
                flux_sum += d_face * (n_val - c) * inv_dx;
            }
            // -y face
            if j > 0 {
                let nidx = Grid::index(w, i, j - 1);
                let (n_val, d_n) = if grid.in_dish(nidx) {
                    (field[nidx], diffusivity[nidx])
                } else {
                    (c, d_c)
                };
                let d_face = 0.5 * (d_c + d_n);
                flux_sum += d_face * (c - n_val) * inv_dx;
            }

            out_rate[idx] = flux_sum * inv_dx;
        }
    }
}

/// Constant diffusivity diffusion: D * laplacian with no-flux BC.
pub fn diffuse_constant(grid: &Grid, field: &[f64], d: f64, scratch_lap: &mut [f64], out_rate: &mut [f64]) {
    laplacian(grid, field, scratch_lap);
    for (o, &lap) in out_rate.iter_mut().zip(scratch_lap.iter()) {
        *o = d * lap;
    }
}

/// Total mass inside dish for a field.
pub fn total_mass(grid: &Grid, field: &[f64]) -> f64 {
    grid.dish_mask
        .iter()
        .zip(field.iter())
        .filter(|(&m, _)| m)
        .map(|(_, &v)| v)
        .sum()
}

/// Variance of field inside dish.
pub fn variance(grid: &Grid, field: &[f64]) -> f64 {
    let cells: Vec<f64> = grid
        .dish_mask
        .iter()
        .zip(field.iter())
        .filter(|(&m, _)| m)
        .map(|(_, &v)| v)
        .collect();
    if cells.is_empty() {
        return 0.0;
    }
    let mean = cells.iter().sum::<f64>() / cells.len() as f64;
    cells.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / cells.len() as f64
}
