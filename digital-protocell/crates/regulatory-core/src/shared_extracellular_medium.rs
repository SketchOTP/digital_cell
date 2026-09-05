//! Opt-in finite shared extracellular N/F medium.
//!
//! This is the R11 organism/world boundary.  The world owns one finite,
//! spatially exposed extracellular compartment shared by all organisms in a
//! step.  Each intact membrane segment submits a local request using the
//! existing permeability, `k_flux`, segment length, and `dt` terms.  Requests
//! are evaluated from one pre-step medium state and receive one common
//! order-independent scale per species.  Delivered material goes directly to
//! the existing interior N/F concentrations; no assimilation or per-organism
//! buffer is involved.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::{permeability, transport_step, TransportLedger, TransportParams};
use serde::{Deserialize, Serialize};

pub const SHARED_FINITE_EXTRACELLULAR_MEDIUM_SCHEMA_V1: &str =
    "digital_cell_shared_finite_extracellular_medium_v1";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SharedMediumDeliveryV1 {
    pub organism_index: usize,
    pub exposed_edges: usize,
    pub n_requested: f64,
    pub f_requested: f64,
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub n_world_loss: f64,
    pub f_world_loss: f64,
    pub allocation_scale: f64,
    pub conservation_error: f64,
    #[serde(default)]
    pub nonfeeding_transport: TransportLedger,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedFiniteExtracellularMediumV1 {
    pub schema: String,
    pub center: [f64; 2],
    pub radius: f64,
    pub material_volume: f64,
    pub initial_n_mass: f64,
    pub initial_f_mass: f64,
    pub n_mass: f64,
    pub f_mass: f64,
    pub step: u64,
    pub transfer_enabled: bool,
    pub ledger_n_taken: f64,
    pub ledger_f_taken: f64,
}

impl SharedFiniteExtracellularMediumV1 {
    pub fn new(
        center: [f64; 2],
        radius: f64,
        n_mass: f64,
        f_mass: f64,
    ) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("shared extracellular medium radius must be positive".into());
        }
        let n_mass = n_mass.max(0.0);
        let f_mass = f_mass.max(0.0);
        let material_volume = std::f64::consts::PI * radius * radius;
        Ok(Self {
            schema: SHARED_FINITE_EXTRACELLULAR_MEDIUM_SCHEMA_V1.to_string(),
            center,
            radius,
            material_volume,
            initial_n_mass: n_mass,
            initial_f_mass: f_mass,
            n_mass,
            f_mass,
            step: 0,
            transfer_enabled: true,
            ledger_n_taken: 0.0,
            ledger_f_taken: 0.0,
        })
    }

    pub fn total_n_mass(&self) -> f64 {
        self.n_mass
    }

    pub fn total_f_mass(&self) -> f64 {
        self.f_mass
    }

    pub fn boundary_concentrations(&self) -> (f64, f64) {
        (
            self.n_mass / self.material_volume,
            self.f_mass / self.material_volume,
        )
    }

    /// Apply one shared-medium step to all meshes simultaneously.
    ///
    /// Local requests are formed from the same pre-step concentrations.  A
    /// single common scale per species debits the one finite shared medium,
    /// so reversing organism order cannot change delivery totals.  Material
    /// is written directly into existing interior N/F.
    pub fn exchange(
        &mut self,
        meshes: &mut [MaterialMesh],
        transport: &TransportParams,
        dt: f64,
    ) -> Vec<SharedMediumDeliveryV1> {
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

        let (boundary_n, boundary_f) = self.boundary_concentrations();
        let mut requests = Vec::new();
        let mut deliveries = vec![SharedMediumDeliveryV1::default(); meshes.len()];
        for (organism_index, mesh) in meshes.iter().enumerate() {
            deliveries[organism_index].organism_index = organism_index;
            deliveries[organism_index].nonfeeding_transport = nonfeeding
                .get(organism_index)
                .cloned()
                .unwrap_or_default();
            if !mesh.can_advance_physics() || dt <= 0.0 {
                continue;
            }
            for edge in 0..mesh.n() {
                if mesh.edges[edge].ruptured || !self.edge_exposed(mesh, edge) {
                    continue;
                }
                deliveries[organism_index].exposed_edges += 1;
                let theta = mesh.occupancy(edge);
                let segment = mesh.edge_length(edge);
                let n = inward_request(
                    boundary_n,
                    mesh.interior.n,
                    permeability(theta, "N"),
                    transport.k_flux,
                    segment,
                    dt,
                );
                let f = inward_request(
                    boundary_f,
                    mesh.interior.f,
                    permeability(theta, "F"),
                    transport.k_flux,
                    segment,
                    dt,
                );
                requests.push((organism_index, n, f));
                deliveries[organism_index].n_requested += n;
                deliveries[organism_index].f_requested += f;
            }
        }

        let total_n_requested: f64 = requests.iter().map(|(_, n, _)| *n).sum();
        let total_f_requested: f64 = requests.iter().map(|(_, _, f)| *f).sum();
        let n_scale = if total_n_requested > 0.0 {
            (self.n_mass / total_n_requested).min(1.0)
        } else {
            1.0
        };
        let f_scale = if total_f_requested > 0.0 {
            (self.f_mass / total_f_requested).min(1.0)
        } else {
            1.0
        };
        let scale = if self.transfer_enabled {
            n_scale.min(f_scale).max(0.0)
        } else {
            0.0
        };

        for (organism_index, requested_n, requested_f) in requests {
            let n = requested_n * scale;
            let f = requested_f * scale;
            if let Some(mesh) = meshes.get_mut(organism_index) {
                let area = mesh.area();
                if area.is_finite() && area > 0.0 {
                    mesh.interior.n += n / area;
                    mesh.interior.f += f / area;
                }
            }
            let delivery = &mut deliveries[organism_index];
            delivery.n_delivered += n;
            delivery.f_delivered += f;
        }

        let delivered_n: f64 = deliveries.iter().map(|d| d.n_delivered).sum();
        let delivered_f: f64 = deliveries.iter().map(|d| d.f_delivered).sum();
        self.n_mass = (self.n_mass - delivered_n).max(0.0);
        self.f_mass = (self.f_mass - delivered_f).max(0.0);
        self.ledger_n_taken += delivered_n;
        self.ledger_f_taken += delivered_f;
        for delivery in &mut deliveries {
            delivery.n_world_loss = delivery.n_delivered;
            delivery.f_world_loss = delivery.f_delivered;
            delivery.allocation_scale = if delivery.n_requested > 0.0 {
                (delivery.n_delivered / delivery.n_requested).min(
                    if delivery.f_requested > 0.0 {
                        delivery.f_delivered / delivery.f_requested
                    } else {
                        1.0
                    },
                )
            } else {
                1.0
            };
            delivery.conservation_error =
                (delivery.n_world_loss - delivery.n_delivered).abs()
                    + (delivery.f_world_loss - delivery.f_delivered).abs();
        }
        self.step = self.step.saturating_add(1);
        deliveries
    }

    fn edge_exposed(&self, mesh: &MaterialMesh, edge: usize) -> bool {
        let a = mesh.vertices[edge];
        let b = mesh.vertices[(edge + 1) % mesh.n()];
        let midpoint = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        (midpoint[0] - self.center[0]).hypot(midpoint[1] - self.center[1]) <= self.radius
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
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};

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
    fn shared_medium_is_finite_local_and_conservative() {
        let mut medium = SharedFiniteExtracellularMediumV1::new([0.0, 0.0], 1.5, 10.0, 10.0).unwrap();
        let mut meshes = vec![mesh([0.0, 0.0]), mesh([8.0, 0.0])];
        let before = (medium.n_mass, medium.f_mass);
        let deliveries = medium.exchange(&mut meshes, &TransportParams::default(), 0.02);
        assert!(deliveries[0].n_delivered > 0.0);
        assert_eq!(deliveries[1].n_delivered, 0.0);
        assert!((before.0 - medium.n_mass - deliveries.iter().map(|d| d.n_delivered).sum::<f64>()).abs() < 1e-12);
        assert!((before.1 - medium.f_mass - deliveries.iter().map(|d| d.f_delivered).sum::<f64>()).abs() < 1e-12);
        assert!(deliveries.iter().all(|d| d.conservation_error == 0.0));
    }

    #[test]
    fn shared_medium_allocation_is_order_independent() {
        let mut first = SharedFiniteExtracellularMediumV1::new([0.0, 0.0], 1.5, 0.01, 0.01).unwrap();
        let mut second = first.clone();
        let mut a = vec![mesh([0.0, 0.0]), mesh([0.0, 0.0])];
        let mut b = vec![a[1].clone(), a[0].clone()];
        let left = first.exchange(&mut a, &TransportParams::default(), 0.02);
        let right = second.exchange(&mut b, &TransportParams::default(), 0.02);
        assert!((first.n_mass - second.n_mass).abs() < 1e-12);
        assert!((left.iter().map(|d| d.n_delivered).sum::<f64>()
            - right.iter().map(|d| d.n_delivered).sum::<f64>()).abs() < 1e-12);
        assert!((left[0].n_delivered - right[1].n_delivered).abs() < 1e-12);
    }

    #[test]
    fn disabled_or_empty_medium_cannot_influx() {
        for (n, f, enabled) in [(0.0, 0.0, true), (10.0, 10.0, false)] {
            let mut medium = SharedFiniteExtracellularMediumV1::new([0.0, 0.0], 1.5, n, f).unwrap();
            medium.transfer_enabled = enabled;
            let mut bodies = vec![mesh([0.0, 0.0])];
            let before = bodies[0].interior;
            let deliveries = medium.exchange(&mut bodies, &TransportParams::default(), 0.02);
            assert_eq!(deliveries[0].n_delivered, 0.0);
            assert_eq!(deliveries[0].f_delivered, 0.0);
            assert_eq!(medium.n_mass, n);
            assert_eq!(medium.f_mass, f);
            assert!(bodies[0].interior.n <= before.n);
            assert!(bodies[0].interior.f <= before.f);
        }
    }
}
