//! Runtime-local carrier for the accepted native-ring excitable-polarity path.
//!
//! This is orchestration state, not a change to the frozen chemistry or
//! regulatory cores.  It mirrors the accepted ENTRY-019..025 native-ring
//! equations and amount-preserving remap rules so a checkpointable runtime can
//! carry polarity through remesh and physical fission.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::FissionEvent;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

const DOMAIN_LENGTH: f64 = 2.0 * PI;
const DT_LIMIT_FACTOR: f64 = 0.08;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingGrid {
    pub ds: Vec<f64>,
    pub centers: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarityAmounts {
    pub u: Vec<f64>,
    pub v: Vec<f64>,
    pub f: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarityState {
    pub grid: RingGrid,
    pub amounts: PolarityAmounts,
}

impl RingGrid {
    pub fn from_mesh(mesh: &MaterialMesh) -> Self {
        let lengths: Vec<f64> = (0..mesh.n())
            .map(|i| mesh.edge_length(i).max(1e-15))
            .collect();
        Self::from_lengths(&lengths)
    }

    pub fn from_lengths(lengths: &[f64]) -> Self {
        assert!(
            !lengths.is_empty(),
            "native polarity ring must be non-empty"
        );
        let perimeter: f64 = lengths.iter().sum();
        assert!(perimeter.is_finite() && perimeter > 0.0);
        let ds: Vec<f64> = lengths
            .iter()
            .map(|length| DOMAIN_LENGTH * length / perimeter)
            .collect();
        let mut cursor = 0.0;
        let centers = ds
            .iter()
            .map(|width| {
                let center = cursor + 0.5 * width;
                cursor += *width;
                center
            })
            .collect();
        Self { ds, centers }
    }

    pub fn total_measure(&self) -> f64 {
        self.ds.iter().sum()
    }
}

impl PolarityState {
    pub fn homogeneous(mesh: &MaterialMesh) -> Self {
        let grid = RingGrid::from_mesh(mesh);
        let (u, v, f) = polar_equilibrium();
        Self {
            amounts: PolarityAmounts {
                u: vec![u; grid.ds.len()],
                v: vec![v; grid.ds.len()],
                f: vec![f; grid.ds.len()],
            },
            grid,
        }
    }

    pub fn motor_fraction(&self) -> Vec<f64> {
        self.amounts
            .u
            .iter()
            .zip(&self.amounts.v)
            .map(|(u, v)| {
                assert!(u.is_finite() && v.is_finite() && *u >= 0.0 && *v >= 0.0);
                let pool = u + v;
                assert!(pool > 0.0, "native polarity pool vanished");
                // Accepted ENTRY-025/CLOSURE route: inactive fraction is the
                // local contractile motor fraction.  This is local and has no
                // global controller, target, or observer input.
                v / pool
            })
            .collect()
    }

    pub fn remap_and_advance(&mut self, mesh: &MaterialMesh, origin: usize, dt: f64) {
        let new_grid = RingGrid::from_mesh(mesh);
        let old_grid = self.grid.clone();
        self.amounts = remap_state(&old_grid, &self.amounts, &new_grid, origin);
        advance(&mut self.amounts, &new_grid, dt);
        self.grid = new_grid;
    }

    pub fn split_after_fission(
        &self,
        event: &FissionEvent,
        daughter_a: &MaterialMesh,
        daughter_b: &MaterialMesh,
        dt: f64,
    ) -> (Self, Self) {
        let (i, j) = event.pinch;
        let a = split_one(&self.amounts, &self.grid, i, daughter_a);
        let b = split_one(&self.amounts, &self.grid, j, daughter_b);
        let mut a = Self {
            grid: RingGrid::from_mesh(daughter_a),
            amounts: a,
        };
        let mut b = Self {
            grid: RingGrid::from_mesh(daughter_b),
            amounts: b,
        };
        // The accepted fission contract creates a closing edge with no parent
        // predecessor.  Advance the unchanged equations once before the
        // children re-enter the actuator boundary, as in ENTRY-027/CLOSURE.
        advance(&mut a.amounts, &a.grid, dt);
        advance(&mut b.amounts, &b.grid, dt);
        (a, b)
    }

    pub fn weighted_pool(&self) -> f64 {
        weighted(&self.amounts.u, &self.grid) + weighted(&self.amounts.v, &self.grid)
    }

    pub fn nonconstant_amplitude(&self) -> f64 {
        let max_u = (1..=self.amounts.u.len() / 2)
            .map(|k| mode(&self.amounts.u, &self.grid, k))
            .fold(0.0, f64::max);
        let max_v = (1..=self.amounts.v.len() / 2)
            .map(|k| mode(&self.amounts.v, &self.grid, k))
            .fold(0.0, f64::max);
        let max_f = (1..=self.amounts.f.len() / 2)
            .map(|k| mode(&self.amounts.f, &self.grid, k))
            .fold(0.0, f64::max);
        max_u.max(max_v).max(max_f)
    }
}

fn polar_equilibrium() -> (f64, f64, f64) {
    let total = 2.0;
    let reaction = |u: f64| {
        let v = total - u;
        let f = 0.8 + 3.8 * u;
        (0.067 + 3.55 * u * u) * v - (1.0 + 0.41 * f + u * u) * u
    };
    let n = 100_000;
    let mut x = 0.0;
    let mut previous = reaction(x);
    for index in 1..=n {
        let y = total * index as f64 / n as f64;
        let current = reaction(y);
        if previous * current < 0.0 {
            let (mut lo, mut hi, mut flo) = (x, y, previous);
            for _ in 0..80 {
                let mid = 0.5 * (lo + hi);
                let fmid = reaction(mid);
                if flo * fmid <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    flo = fmid;
                }
            }
            let u = 0.5 * (lo + hi);
            return (u, total - u, 0.8 + 3.8 * u);
        }
        x = y;
        previous = current;
    }
    panic!("accepted native polarity homogeneous equilibrium was not found")
}

fn weighted(values: &[f64], grid: &RingGrid) -> f64 {
    values
        .iter()
        .zip(&grid.ds)
        .map(|(value, width)| value * width)
        .sum()
}

fn diffusion(values: &[f64], grid: &RingGrid, coefficient: f64, i: usize) -> f64 {
    let n = values.len();
    let previous = (i + n - 1) % n;
    let next = (i + 1) % n;
    let left = 0.5 * (grid.ds[previous] + grid.ds[i]);
    let right = 0.5 * (grid.ds[i] + grid.ds[next]);
    (coefficient * (values[next] - values[i]) / right
        - coefficient * (values[i] - values[previous]) / left)
        / grid.ds[i]
}

fn exchange(u: f64, v: f64, f: f64) -> f64 {
    (0.067 + 3.55 * u * u) * v - (1.0 + 0.41 * f + u * u) * u
}

fn rhs(state: &PolarityAmounts, grid: &RingGrid) -> PolarityAmounts {
    let mut u = vec![0.0; state.u.len()];
    let mut v = vec![0.0; state.v.len()];
    let mut f = vec![0.0; state.f.len()];
    for i in 0..state.u.len() {
        let reaction = exchange(state.u[i], state.v[i], state.f[i]);
        u[i] = reaction + diffusion(&state.u, grid, 0.1, i);
        v[i] = -reaction + diffusion(&state.v, grid, 1.0, i);
        f[i] = 0.6 * (0.8 + 3.8 * state.u[i] - state.f[i]) + diffusion(&state.f, grid, 0.001, i);
    }
    PolarityAmounts { u, v, f }
}

fn add(state: &PolarityAmounts, derivative: &PolarityAmounts, scale: f64) -> PolarityAmounts {
    PolarityAmounts {
        u: state
            .u
            .iter()
            .zip(&derivative.u)
            .map(|(x, y)| x + scale * y)
            .collect(),
        v: state
            .v
            .iter()
            .zip(&derivative.v)
            .map(|(x, y)| x + scale * y)
            .collect(),
        f: state
            .f
            .iter()
            .zip(&derivative.f)
            .map(|(x, y)| x + scale * y)
            .collect(),
    }
}

fn advance(state: &mut PolarityAmounts, grid: &RingGrid, total: f64) {
    let minimum = grid.ds.iter().copied().fold(f64::INFINITY, f64::min);
    let h0 = (DT_LIMIT_FACTOR * minimum * minimum).min(total);
    let count = (total / h0).ceil().max(1.0) as usize;
    let h = total / count as f64;
    for _ in 0..count {
        let a = rhs(state, grid);
        let b = rhs(&add(state, &a, h * 0.5), grid);
        let c = rhs(&add(state, &b, h * 0.5), grid);
        let d = rhs(&add(state, &c, h), grid);
        for i in 0..state.u.len() {
            state.u[i] += h * (a.u[i] + 2.0 * b.u[i] + 2.0 * c.u[i] + d.u[i]) / 6.0;
            state.v[i] += h * (a.v[i] + 2.0 * b.v[i] + 2.0 * c.v[i] + d.v[i]) / 6.0;
            state.f[i] += h * (a.f[i] + 2.0 * b.f[i] + 2.0 * c.f[i] + d.f[i]) / 6.0;
        }
    }
}

fn remap_state(
    old: &RingGrid,
    state: &PolarityAmounts,
    new: &RingGrid,
    origin: usize,
) -> PolarityAmounts {
    let map = |values: &[f64]| {
        let amounts: Vec<f64> = values
            .iter()
            .zip(&old.ds)
            .map(|(value, width)| value * width)
            .collect();
        let mapped = map_amounts(old, &amounts, new, origin);
        mapped
            .iter()
            .zip(&new.ds)
            .map(|(amount, width)| amount / width)
            .collect()
    };
    PolarityAmounts {
        u: map(&state.u),
        v: map(&state.v),
        f: map(&state.f),
    }
}

fn map_amounts(old: &RingGrid, amounts: &[f64], new: &RingGrid, origin: usize) -> Vec<f64> {
    let n = old.ds.len();
    let mut starts = vec![0.0; n];
    let mut cursor = 0.0;
    for i in 0..n {
        starts[i] = cursor;
        cursor += old.ds[i];
    }
    let total = cursor;
    let mut out = Vec::with_capacity(new.ds.len());
    let mut new_start = 0.0;
    for width in &new.ds {
        let end = new_start + width;
        let mut x = new_start;
        let mut amount = 0.0;
        while x < end - 1e-14 {
            let absolute = (x + starts[origin % n]).rem_euclid(total);
            let mut old_index = 0;
            while old_index + 1 < n && absolute >= starts[old_index] + old.ds[old_index] - 1e-14 {
                old_index += 1;
            }
            let boundary = (starts[old_index] + old.ds[old_index]).min(total);
            let take = (end - x).min(boundary - absolute).max(0.0);
            amount += amounts[old_index] * take / old.ds[old_index].max(1e-15);
            x += take.max(1e-15);
        }
        out.push(amount);
        new_start = end;
    }
    out
}

fn split_one(
    state: &PolarityAmounts,
    grid: &RingGrid,
    start: usize,
    daughter: &MaterialMesh,
) -> PolarityAmounts {
    let n = grid.ds.len();
    let destination = RingGrid::from_mesh(daughter);
    let take = |values: &[f64]| {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let source = (start + index) % n;
                if index + 1 == daughter.n() {
                    0.0
                } else {
                    value * grid.ds[source]
                }
            })
            .zip(&destination.ds)
            .map(|(amount, width)| amount / width)
            .collect()
    };
    PolarityAmounts {
        u: take(&state.u),
        v: take(&state.v),
        f: take(&state.f),
    }
}

fn mode(values: &[f64], grid: &RingGrid, k: usize) -> f64 {
    let mean = weighted(values, grid) / DOMAIN_LENGTH;
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for (i, value) in values.iter().enumerate() {
        let phase = 2.0 * PI * k as f64 * grid.centers[i] / DOMAIN_LENGTH;
        real += (value - mean) * grid.ds[i] * phase.cos();
        imaginary -= (value - mean) * grid.ds[i] * phase.sin();
    }
    real.hypot(imaginary) / DOMAIN_LENGTH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_grid_diffusion_is_conservative() {
        let grid = RingGrid::from_lengths(&[1.0; 24]);
        let state = PolarityAmounts {
            u: (0..24).map(|i| i as f64).collect(),
            v: vec![2.0; 24],
            f: vec![3.0; 24],
        };
        let derivative = rhs(&state, &grid);
        let residual: f64 = derivative
            .u
            .iter()
            .zip(&grid.ds)
            .map(|(x, d)| x * d)
            .sum::<f64>()
            + derivative
                .v
                .iter()
                .zip(&grid.ds)
                .map(|(x, d)| x * d)
                .sum::<f64>();
        assert!(residual.abs() < 1e-12);
    }

    #[test]
    fn homogeneous_state_has_live_pool_and_bounded_motor() {
        let mesh = MaterialMesh::seed_regular(
            24,
            5.0,
            40.0,
            40.0,
            chemistry_core::material_mesh::DEFAULT_RHO_S,
            0.7,
            Default::default(),
            Default::default(),
            5.0,
        );
        let state = PolarityState::homogeneous(&mesh);
        let motor = state.motor_fraction();
        assert!(motor.iter().all(|x| (0.0..=1.0).contains(x)));
        assert!(state.weighted_pool() > 0.0);
    }
}
