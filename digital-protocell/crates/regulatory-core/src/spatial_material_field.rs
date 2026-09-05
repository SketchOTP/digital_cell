//! Opt-in finite spatial material field for Route-B ecology.
//!
//! This module is deliberately separate from `FiniteWorldV1`.  The latter is
//! a finite hard-contact reservoir.  This field is a finite, local N/F/W
//! environment whose N/F transfer is evaluated at the actual material edges
//! and whose diffusion is conservative.  It does not provide replenishment,
//! targets, distances, or behavioral guidance.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::{transport_step, TransportLedger, TransportParams};
use serde::{Deserialize, Serialize};

pub const SPATIAL_MATERIAL_FIELD_SCHEMA_V1: &str = "digital_cell_spatial_material_field_v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialMaterialFieldV1 {
    pub schema: String,
    pub nx: usize,
    pub ny: usize,
    pub dx: f64,
    pub origin: [f64; 2],
    /// Environmental mass per control volume.
    pub n: Vec<f64>,
    pub f: Vec<f64>,
    pub w: Vec<f64>,
    /// Diffusion coefficient in world-length² / time.
    pub diff: f64,
    pub tick: u64,
    pub initial_n_mass: f64,
    pub initial_f_mass: f64,
    pub ledger_n_taken: f64,
    pub ledger_f_taken: f64,
    pub ledger_w_emitted: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpatialFieldDeliveryV1 {
    pub organism_index: usize,
    pub exposed_edges: usize,
    pub touched_cells: usize,
    pub n_requested: f64,
    pub f_requested: f64,
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub n_world_loss: f64,
    pub f_world_loss: f64,
    pub w_emitted: f64,
    pub allocation_scale: f64,
    pub conservation_error: f64,
    /// Existing membrane transport for species other than positive field N/F.
    pub nonfeeding_transport: TransportLedger,
}

#[derive(Debug, Clone, Copy)]
struct EdgeRequest {
    organism_index: usize,
    cell: usize,
    n: f64,
    f: f64,
}

impl SpatialMaterialFieldV1 {
    pub fn new(
        nx: usize,
        ny: usize,
        dx: f64,
        origin: [f64; 2],
        n: Vec<f64>,
        f: Vec<f64>,
        diff: f64,
    ) -> Result<Self, String> {
        let cells = nx.checked_mul(ny).ok_or("field dimensions overflow")?;
        if nx == 0 || ny == 0 || n.len() != cells || f.len() != cells {
            return Err("field dimensions and species arrays must agree".into());
        }
        if !dx.is_finite() || dx <= 0.0 || !diff.is_finite() || diff < 0.0 {
            return Err("field dx must be positive and diffusion nonnegative".into());
        }
        if n.iter().any(|value| !value.is_finite() || *value < 0.0)
            || f.iter().any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err("field N/F masses must be finite and nonnegative".into());
        }
        let initial_n_mass = n.iter().sum();
        let initial_f_mass = f.iter().sum();
        Ok(Self {
            schema: SPATIAL_MATERIAL_FIELD_SCHEMA_V1.to_string(),
            nx,
            ny,
            dx,
            origin,
            n,
            f,
            w: vec![0.0; cells],
            diff,
            tick: 0,
            initial_n_mass,
            initial_f_mass,
            ledger_n_taken: 0.0,
            ledger_f_taken: 0.0,
            ledger_w_emitted: 0.0,
        })
    }

    pub fn single_patch(
        nx: usize,
        ny: usize,
        dx: f64,
        origin: [f64; 2],
        patch: (usize, usize),
        n_mass: f64,
        f_mass: f64,
        diff: f64,
    ) -> Result<Self, String> {
        if patch.0 >= nx || patch.1 >= ny {
            return Err("single field patch is outside the field".into());
        }
        let mut n = vec![0.0; nx.saturating_mul(ny)];
        let f = vec![0.0; nx.saturating_mul(ny)];
        let index = patch.1 * nx + patch.0;
        n[index] = n_mass.max(0.0);
        let mut field = Self::new(nx, ny, dx, origin, n, f, diff)?;
        field.f[index] = f_mass.max(0.0);
        field.initial_f_mass = f_mass.max(0.0);
        Ok(field)
    }

    pub fn cell_volume(&self) -> f64 {
        self.dx * self.dx
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        j * self.nx + i
    }

    pub fn total_n_mass(&self) -> f64 {
        self.n.iter().sum()
    }

    pub fn total_f_mass(&self) -> f64 {
        self.f.iter().sum()
    }

    pub fn total_w_mass(&self) -> f64 {
        self.w.iter().sum()
    }

    pub fn world_to_cell(&self, point: [f64; 2]) -> Option<usize> {
        let x = (point[0] - self.origin[0]) / self.dx;
        let y = (point[1] - self.origin[1]) / self.dx;
        if x < 0.0 || y < 0.0 || x >= self.nx as f64 || y >= self.ny as f64 {
            return None;
        }
        Some(self.idx(x.floor() as usize, y.floor() as usize))
    }

    pub fn concentration(&self, cell: usize) -> (f64, f64, f64) {
        let volume = self.cell_volume().max(1e-15);
        (
            self.n[cell] / volume,
            self.f[cell] / volume,
            self.w[cell] / volume,
        )
    }

    /// Conservative no-flux diffusion. Pairwise exchanges are applied once
    /// per undirected neighboring cell pair, so every internal flux cancels.
    pub fn diffuse(&mut self, dt: f64) {
        if self.diff <= 0.0 || dt <= 0.0 {
            self.tick = self.tick.saturating_add(1);
            return;
        }
        let alpha = self.diff * dt / (self.dx * self.dx);
        let nsub = (alpha / 0.24).ceil().max(1.0) as usize;
        let sub_dt = dt / nsub as f64;
        for _ in 0..nsub {
            self.diffuse_once(sub_dt);
        }
        self.tick = self.tick.saturating_add(1);
    }

    fn diffuse_once(&mut self, dt: f64) {
        let coefficient = self.diff * dt / self.dx.powi(2);
        let mut delta_n = vec![0.0; self.n.len()];
        let mut delta_f = vec![0.0; self.f.len()];
        let mut delta_w = vec![0.0; self.w.len()];
        for j in 0..self.ny {
            for i in 0..self.nx {
                let cell = self.idx(i, j);
                if i + 1 < self.nx {
                    self.apply_pair(
                        cell,
                        self.idx(i + 1, j),
                        coefficient,
                        &mut delta_n,
                        &mut delta_f,
                        &mut delta_w,
                    );
                }
                if j + 1 < self.ny {
                    self.apply_pair(
                        cell,
                        self.idx(i, j + 1),
                        coefficient,
                        &mut delta_n,
                        &mut delta_f,
                        &mut delta_w,
                    );
                }
            }
        }
        for cell in 0..self.n.len() {
            self.n[cell] = (self.n[cell] + delta_n[cell]).max(0.0);
            self.f[cell] = (self.f[cell] + delta_f[cell]).max(0.0);
            self.w[cell] = (self.w[cell] + delta_w[cell]).max(0.0);
        }
    }

    fn apply_pair(
        &self,
        left: usize,
        right: usize,
        coefficient: f64,
        delta_n: &mut [f64],
        delta_f: &mut [f64],
        delta_w: &mut [f64],
    ) {
        let flux_n = coefficient * (self.n[right] - self.n[left]);
        let flux_f = coefficient * (self.f[right] - self.f[left]);
        let flux_w = coefficient * (self.w[right] - self.w[left]);
        delta_n[left] += flux_n;
        delta_n[right] -= flux_n;
        delta_f[left] += flux_f;
        delta_f[right] -= flux_f;
        delta_w[left] += flux_w;
        delta_w[right] -= flux_w;
    }

    /// Execute one simultaneous local exchange against one pre-step field.
    /// Positive N/F requests use the unchanged membrane permeability and
    /// k_flux law; allocation is proportional per cell, making shared access
    /// independent of organism iteration order.
    pub fn exchange(
        &mut self,
        meshes: &mut [MaterialMesh],
        transport: &TransportParams,
        dt: f64,
    ) -> Vec<SpatialFieldDeliveryV1> {
        let nonfeeding: Vec<TransportLedger> = meshes
            .iter_mut()
            .map(|mesh| {
                let exterior = mesh.exterior;
                mesh.exterior.n = 0.0;
                mesh.exterior.f = 0.0;
                let ledger = transport_step(mesh, transport, dt);
                mesh.exterior = exterior;
                ledger
            })
            .collect();

        let mut requests = Vec::new();
        let mut exposed = vec![0usize; meshes.len()];
        let mut cells = vec![Vec::<usize>::new(); self.n.len()];
        for (organism_index, mesh) in meshes.iter().enumerate() {
            if !mesh.can_advance_physics() || dt <= 0.0 {
                continue;
            }
            for edge in 0..mesh.n() {
                if mesh.edges[edge].ruptured {
                    continue;
                }
                let a = mesh.vertices[edge];
                let b = mesh.vertices[(edge + 1) % mesh.n()];
                let midpoint = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
                let Some(cell) = self.world_to_cell(midpoint) else {
                    continue;
                };
                let (boundary_n, boundary_f, _) = self.concentration(cell);
                let theta = mesh.occupancy(edge);
                let segment = mesh.edge_length(edge);
                let n = inward_request(
                    boundary_n,
                    mesh.interior.n,
                    chemistry_core::mesh_transport::permeability(theta, "N"),
                    transport.k_flux,
                    segment,
                    dt,
                );
                let f = inward_request(
                    boundary_f,
                    mesh.interior.f,
                    chemistry_core::mesh_transport::permeability(theta, "F"),
                    transport.k_flux,
                    segment,
                    dt,
                );
                if boundary_n > 0.0 || boundary_f > 0.0 {
                    exposed[organism_index] += 1;
                }
                let request_index = requests.len();
                requests.push(EdgeRequest {
                    organism_index,
                    cell,
                    n,
                    f,
                });
                cells[cell].push(request_index);
            }
        }

        let mut scales = vec![(1.0, 1.0); self.n.len()];
        for cell in 0..self.n.len() {
            let total_n: f64 = cells[cell].iter().map(|&index| requests[index].n).sum();
            let total_f: f64 = cells[cell].iter().map(|&index| requests[index].f).sum();
            scales[cell] = (
                if total_n > 0.0 {
                    (self.n[cell] / total_n).min(1.0)
                } else {
                    1.0
                },
                if total_f > 0.0 {
                    (self.f[cell] / total_f).min(1.0)
                } else {
                    1.0
                },
            );
        }

        let mut deliveries = vec![SpatialFieldDeliveryV1::default(); meshes.len()];
        for (organism_index, delivery) in deliveries.iter_mut().enumerate() {
            delivery.organism_index = organism_index;
            delivery.exposed_edges = exposed[organism_index];
            delivery.nonfeeding_transport = nonfeeding[organism_index].clone();
        }
        for request in requests {
            let (n_scale, f_scale) = scales[request.cell];
            let n = request.n * n_scale;
            let f = request.f * f_scale;
            self.n[request.cell] = (self.n[request.cell] - n).max(0.0);
            self.f[request.cell] = (self.f[request.cell] - f).max(0.0);
            if let Some(mesh) = meshes.get_mut(request.organism_index) {
                let area = mesh.area().max(1e-15);
                mesh.interior.n += n / area;
                mesh.interior.f += f / area;
            }
            let delivery = &mut deliveries[request.organism_index];
            delivery.touched_cells += 1;
            delivery.n_requested += request.n;
            delivery.f_requested += request.f;
            delivery.n_delivered += n;
            delivery.f_delivered += f;
        }
        for delivery in &mut deliveries {
            let n_scale = if delivery.n_requested > 0.0 {
                delivery.n_delivered / delivery.n_requested
            } else {
                1.0
            };
            let f_scale = if delivery.f_requested > 0.0 {
                delivery.f_delivered / delivery.f_requested
            } else {
                1.0
            };
            delivery.allocation_scale = n_scale.min(f_scale);
            delivery.n_world_loss = delivery.n_delivered;
            delivery.f_world_loss = delivery.f_delivered;
            delivery.conservation_error = (delivery.n_world_loss - delivery.n_delivered).abs()
                + (delivery.f_world_loss - delivery.f_delivered).abs();
        }
        self.ledger_n_taken += deliveries.iter().map(|item| item.n_delivered).sum::<f64>();
        self.ledger_f_taken += deliveries.iter().map(|item| item.f_delivered).sum::<f64>();
        deliveries
    }

    /// Return a post-reaction W amount through the same local edge partition
    /// used by the field exchange. No centroid shortcut is used.
    pub fn emit_w(&mut self, mesh: &MaterialMesh, amount: f64) -> f64 {
        let amount = amount.max(0.0);
        if amount <= 0.0 || mesh.n() == 0 {
            return 0.0;
        }
        let perimeter = mesh.perimeter().max(1e-15);
        for edge in 0..mesh.n() {
            let a = mesh.vertices[edge];
            let b = mesh.vertices[(edge + 1) % mesh.n()];
            let midpoint = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
            if let Some(cell) = self.world_to_cell(midpoint) {
                self.w[cell] += amount * mesh.edge_length(edge) / perimeter;
            }
        }
        self.ledger_w_emitted += amount;
        amount
    }
}

fn inward_request(
    boundary: f64,
    interior: f64,
    permeability: f64,
    k_flux: f64,
    segment: f64,
    dt: f64,
) -> f64 {
    (k_flux * permeability * (boundary - interior.max(0.0)) * segment * dt).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{
        LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S,
    };

    fn mesh(center: [f64; 2]) -> MaterialMesh {
        let mut mesh = MaterialMesh::seed_regular(
            24,
            0.8,
            center[0],
            center[1],
            DEFAULT_RHO_S,
            0.7,
            LumpedChem::default(),
            LumpedChem::default(),
            5.0,
        );
        mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
        mesh
    }

    #[test]
    fn diffusion_preserves_mass_and_constant_fields() {
        let mut field =
            SpatialMaterialFieldV1::new(3, 2, 1.0, [0.0, 0.0], vec![1.0; 6], vec![2.0; 6], 0.4)
                .unwrap();
        let before = (field.total_n_mass(), field.total_f_mass());
        field.diffuse(0.02);
        assert_eq!(field.n, vec![1.0; 6]);
        assert_eq!(field.f, vec![2.0; 6]);
        assert!((field.total_n_mass() - before.0).abs() < 1e-12);
        assert!((field.total_f_mass() - before.1).abs() < 1e-12);
    }

    #[test]
    fn local_edge_exchange_is_finite_and_conservative() {
        let mut field =
            SpatialMaterialFieldV1::single_patch(4, 1, 2.0, [-4.0, -1.0], (1, 0), 10.0, 10.0, 0.0)
                .unwrap();
        let mut near = mesh([0.0, 0.0]);
        let mut far = mesh([6.0, 0.0]);
        let before = field.total_n_mass();
        let deliveries = field.exchange(
            &mut [near.clone(), far.clone()],
            &TransportParams::default(),
            0.02,
        );
        assert!(deliveries[0].n_delivered > 0.0);
        assert_eq!(deliveries[1].n_delivered, 0.0);
        assert!((before - field.total_n_mass() - deliveries[0].n_delivered).abs() < 1e-12);
        near.interior.n = 0.0;
        far.interior.n = 0.0;
    }

    #[test]
    fn shared_cell_allocation_is_order_independent() {
        let mut left =
            SpatialMaterialFieldV1::single_patch(1, 1, 2.0, [-1.0, -1.0], (0, 0), 0.01, 0.01, 0.0)
                .unwrap();
        let mut right = left.clone();
        let mut first = vec![mesh([0.0, 0.0]), mesh([0.0, 0.0])];
        let mut second = vec![first[1].clone(), first[0].clone()];
        let a = left.exchange(&mut first, &TransportParams::default(), 0.02);
        let b = right.exchange(&mut second, &TransportParams::default(), 0.02);
        let total_a: f64 = a.iter().map(|x| x.n_delivered).sum();
        let total_b: f64 = b.iter().map(|x| x.n_delivered).sum();
        assert!((total_a - total_b).abs() < 1e-12);
        assert!((left.total_n_mass() - right.total_n_mass()).abs() < 1e-12);
        assert!((a[0].n_delivered - b[1].n_delivered).abs() < 1e-12);
        assert!((a[1].n_delivered - b[0].n_delivered).abs() < 1e-12);
    }

    #[test]
    fn empty_field_cannot_create_positive_influx() {
        let mut field =
            SpatialMaterialFieldV1::new(2, 2, 1.0, [-1.0, -1.0], vec![0.0; 4], vec![0.0; 4], 0.0)
                .unwrap();
        let mut meshes = vec![mesh([0.0, 0.0])];
        let before = meshes[0].interior;
        let deliveries = field.exchange(&mut meshes, &TransportParams::default(), 0.02);
        assert_eq!(deliveries[0].n_delivered, 0.0);
        assert_eq!(deliveries[0].f_delivered, 0.0);
        assert!((meshes[0].interior.n - before.n).abs() < 1e-12);
        assert!((meshes[0].interior.f - before.f).abs() < 1e-12);
    }
}
