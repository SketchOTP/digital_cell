//! D-090 spatial shared dish — finite N/F/W field with diffusion and local uptake.
//!
//! Ecology-only: does not change organism biology. Organisms still transport against
//! a local exterior sample; the dish ledger owns environmental mass.

use crate::material_mesh::MaterialMesh;
use crate::mesh_transport::{transport_step, TransportLedger, TransportParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialDish {
    pub nx: usize,
    pub ny: usize,
    /// Cell edge length in mesh world units.
    pub dx: f64,
    /// World coordinate of cell (0,0) lower-left corner.
    pub origin: [f64; 2],
    /// Mass of N / F / W per cell.
    pub n: Vec<f64>,
    pub f: Vec<f64>,
    pub w: Vec<f64>,
    /// Total mass inflow rate (mass / time) distributed uniformly over the dish.
    pub supply_n: f64,
    pub supply_f: f64,
    /// Diffusion coefficient (length² / time) for N, F, W.
    pub diff: f64,
    /// Per-dish step counter (never a process-global static).
    pub tick: u64,
    /// Cumulative environmental ledger (observer).
    pub ledger_n_supplied: f64,
    pub ledger_f_supplied: f64,
    pub ledger_n_taken: f64,
    pub ledger_f_taken: f64,
    pub ledger_w_emitted: f64,
}

impl SpatialDish {
    pub fn new(
        nx: usize,
        ny: usize,
        dx: f64,
        origin: [f64; 2],
        n0_mass: f64,
        f0_mass: f64,
        supply_n: f64,
        supply_f: f64,
        diff: f64,
    ) -> Self {
        let cells = nx * ny;
        let n_each = n0_mass / cells as f64;
        let f_each = f0_mass / cells as f64;
        Self {
            nx,
            ny,
            dx,
            origin,
            n: vec![n_each; cells],
            f: vec![f_each; cells],
            w: vec![0.0; cells],
            supply_n,
            supply_f,
            diff,
            tick: 0,
            ledger_n_supplied: 0.0,
            ledger_f_supplied: 0.0,
            ledger_n_taken: 0.0,
            ledger_f_taken: 0.0,
            ledger_w_emitted: 0.0,
        }
    }

    pub fn cell_volume(&self) -> f64 {
        self.dx * self.dx
    }

    pub fn total_n(&self) -> f64 {
        self.n.iter().sum()
    }
    pub fn total_f(&self) -> f64 {
        self.f.iter().sum()
    }
    pub fn total_w(&self) -> f64 {
        self.w.iter().sum()
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        j * self.nx + i
    }

    pub fn world_to_ij(&self, x: f64, y: f64) -> (usize, usize) {
        let i = ((x - self.origin[0]) / self.dx).floor() as isize;
        let j = ((y - self.origin[1]) / self.dx).floor() as isize;
        let i = i.clamp(0, self.nx as isize - 1) as usize;
        let j = j.clamp(0, self.ny as isize - 1) as usize;
        (i, j)
    }

    pub fn conc_at(&self, x: f64, y: f64) -> (f64, f64, f64) {
        let (i, j) = self.world_to_ij(x, y);
        let k = self.idx(i, j);
        let v = self.cell_volume().max(1e-15);
        (self.n[k] / v, self.f[k] / v, self.w[k] / v)
    }

    /// Sample exterior chemistry at the mesh centroid.
    pub fn sync_mesh_exterior(&self, mesh: &mut MaterialMesh) {
        let c = mesh.centroid();
        let (n, f, w) = self.conc_at(c[0], c[1]);
        mesh.exterior.n = n;
        mesh.exterior.f = f;
        mesh.exterior.w = w;
        mesh.exterior.c = 0.0;
        mesh.exterior.a = 0.0;
        mesh.exterior.c_h = 0.0;
        mesh.exterior.c_b = 0.0;
    }

    pub fn supply_step(&mut self, dt: f64) {
        let cells = (self.nx * self.ny) as f64;
        let dn = self.supply_n * dt / cells;
        let df = self.supply_f * dt / cells;
        for i in 0..self.n.len() {
            self.n[i] = (self.n[i] + dn).max(0.0);
            self.f[i] = (self.f[i] + df).max(0.0);
        }
        self.ledger_n_supplied += self.supply_n * dt;
        self.ledger_f_supplied += self.supply_f * dt;
    }

    /// Explicit isotropic diffusion with no-flux boundaries (mass conserved).
    pub fn diffuse(&mut self, dt: f64) {
        if self.diff <= 0.0 {
            return;
        }
        let alpha = self.diff * dt / (self.dx * self.dx).max(1e-15);
        if alpha > 0.24 {
            // Sub-step for stability under large dt/dx².
            let nsub = ((alpha / 0.24).ceil() as usize).max(1);
            let dts = dt / nsub as f64;
            for _ in 0..nsub {
                self.diffuse_once(dts);
            }
        } else {
            self.diffuse_once(dt);
        }
    }

    fn diffuse_once(&mut self, dt: f64) {
        let alpha = self.diff * dt / (self.dx * self.dx).max(1e-15);
        let nx = self.nx;
        let ny = self.ny;
        let mut n2 = self.n.clone();
        let mut f2 = self.f.clone();
        let mut w2 = self.w.clone();
        for j in 0..ny {
            for i in 0..nx {
                let k = self.idx(i, j);
                let mut ln = 0.0;
                let mut lf = 0.0;
                let mut lw = 0.0;
                let mut nn = 0.0;
                for (di, dj) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                    let ii = i as isize + di;
                    let jj = j as isize + dj;
                    if ii < 0 || jj < 0 || ii >= nx as isize || jj >= ny as isize {
                        continue;
                    }
                    let kk = self.idx(ii as usize, jj as usize);
                    ln += self.n[kk] - self.n[k];
                    lf += self.f[kk] - self.f[k];
                    lw += self.w[kk] - self.w[k];
                    nn += 1.0;
                }
                if nn > 0.0 {
                    // No clamp here — clamping would destroy mass conservation.
                    n2[k] = self.n[k] + alpha * ln;
                    f2[k] = self.f[k] + alpha * lf;
                    w2[k] = self.w[k] + alpha * lw;
                }
            }
        }
        self.n = n2;
        self.f = f2;
        self.w = w2;
    }

    /// Remove uptake mass from a neighborhood of the organism centroid.
    pub fn apply_uptake(&mut self, mesh: &MaterialMesh, tled: &TransportLedger) {
        let c = mesh.centroid();
        let (ic, jc) = self.world_to_ij(c[0], c[1]);
        // 3×3 stencil weighted by inverse distance (local uptake).
        let mut weights = Vec::new();
        let mut wsum = 0.0;
        for dj in -1isize..=1 {
            for di in -1isize..=1 {
                let ii = ic as isize + di;
                let jj = jc as isize + dj;
                if ii < 0 || jj < 0 || ii >= self.nx as isize || jj >= self.ny as isize {
                    continue;
                }
                let w = if di == 0 && dj == 0 {
                    4.0
                } else if di.abs() + dj.abs() == 1 {
                    2.0
                } else {
                    1.0
                };
                weights.push((ii as usize, jj as usize, w));
                wsum += w;
            }
        }
        if wsum <= 0.0 {
            return;
        }
        let mut taken_n = 0.0;
        let mut taken_f = 0.0;
        let mut emitted_w = 0.0;
        for &(i, j, w) in &weights {
            let k = self.idx(i, j);
            let frac = w / wsum;
    // Positive uptake only depletes the dish.
        let dn_req = tled.n_in.max(0.0) * frac;
        let df_req = tled.f_in.max(0.0) * frac;
        let dn = dn_req.min(self.n[k]);
        let df = df_req.min(self.f[k]);
        self.n[k] -= dn;
        self.f[k] -= df;
        self.w[k] += tled.w_out.max(0.0) * frac;
            taken_n += dn;
            taken_f += df;
            emitted_w += tled.w_out * frac;
        }
        self.ledger_n_taken += taken_n;
        self.ledger_f_taken += taken_f;
        self.ledger_w_emitted += emitted_w;
    }

    /// Transport one mesh against the local dish field and deduct uptake.
    pub fn transport_organism(
        &mut self,
        mesh: &mut MaterialMesh,
        transport: &TransportParams,
        dt: f64,
    ) -> TransportLedger {
        self.sync_mesh_exterior(mesh);
        let tled = transport_step(mesh, transport, dt);
        self.apply_uptake(mesh, &tled);
        tled
    }
}

/// Observer: total environmental mass plus organism interior N/F inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DishMassSnapshot {
    pub dish_n: f64,
    pub dish_f: f64,
    pub dish_w: f64,
    pub organism_n: f64,
    pub organism_f: f64,
    pub organism_w: f64,
    pub supplied_n: f64,
    pub supplied_f: f64,
    pub taken_n: f64,
    pub taken_f: f64,
}
