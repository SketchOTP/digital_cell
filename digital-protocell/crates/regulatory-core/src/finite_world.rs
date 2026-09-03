//! Opt-in M2 finite-world exchange.
//!
//! The world owns finite R5 backing reservoirs.  Organisms submit local V1
//! uptake requests against one pre-step world state; a common proportional
//! allocation is then applied to every request.  This keeps simultaneous
//! sharing independent of iteration order while leaving the accepted V1/R5
//! exposure and flux law untouched.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::{transport_step, TransportLedger, TransportParams};
use serde::{Deserialize, Serialize};

use crate::backing_reservoir::FiniteSpatialBackingReservoirV1;
use crate::spatial_resource::SpatialResourceStepLedgerV1;

pub const FINITE_WORLD_SCHEMA_V1: &str = "digital_cell_m2_finite_world_v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiniteWorldResourceV1 {
    pub id: String,
    pub backing: FiniteSpatialBackingReservoirV1,
}

impl FiniteWorldResourceV1 {
    pub fn new(
        id: impl Into<String>,
        center: [f64; 2],
        radius: f64,
        n_mass: f64,
        f_mass: f64,
        boundary_n: f64,
        boundary_f: f64,
    ) -> Self {
        Self {
            id: id.into(),
            backing: FiniteSpatialBackingReservoirV1::new(
                center, radius, n_mass, f_mass, boundary_n, boundary_f,
            ),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiniteWorldRequestV1 {
    pub organism_index: usize,
    pub resource_index: usize,
    pub exposed_edges: usize,
    pub requested_n: f64,
    pub requested_f: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiniteWorldDeliveryV1 {
    pub organism_index: usize,
    pub resource_index: usize,
    pub resource_id: String,
    pub exposed_edges: usize,
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub n_world_loss: f64,
    pub f_world_loss: f64,
    pub allocation_scale: f64,
    pub conservation_error: f64,
    /// The ordinary membrane exchange performed in the same step with the
    /// finite-world N/F bath disabled.  This keeps W export and C/A leakage
    /// on their frozen transport path without creating an unbacked positive
    /// N/F source.
    #[serde(default)]
    pub nonfeeding_transport: TransportLedger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiniteWorldV1 {
    pub schema: String,
    pub resources: Vec<FiniteWorldResourceV1>,
    pub step: u64,
    pub transfer_enabled: bool,
}

impl FiniteWorldV1 {
    pub fn new(resources: Vec<FiniteWorldResourceV1>) -> Self {
        Self {
            schema: FINITE_WORLD_SCHEMA_V1.to_string(),
            resources,
            step: 0,
            transfer_enabled: true,
        }
    }

    pub fn total_n_mass(&self) -> f64 {
        self.resources
            .iter()
            .map(|resource| resource.backing.region.n_mass)
            .sum()
    }

    pub fn total_f_mass(&self) -> f64 {
        self.resources
            .iter()
            .map(|resource| resource.backing.region.f_mass)
            .sum()
    }

    pub fn request(
        &self,
        organism_index: usize,
        resource_index: usize,
        mesh: &MaterialMesh,
        transport: &TransportParams,
        dt: f64,
    ) -> FiniteWorldRequestV1 {
        let Some(resource) = self.resources.get(resource_index) else {
            return FiniteWorldRequestV1 {
                organism_index,
                resource_index,
                ..Default::default()
            };
        };
        let mut preview_mesh = mesh.clone();
        let mut preview = resource.backing.clone();
        let ledger = preview.uptake(&mut preview_mesh, transport, dt);
        FiniteWorldRequestV1 {
            organism_index,
            resource_index,
            exposed_edges: ledger.exposed_edges,
            requested_n: ledger.n_delivered.max(0.0),
            requested_f: ledger.f_delivered.max(0.0),
        }
    }

    /// Apply one simultaneous world step. Requests are derived from the same
    /// pre-transfer world/organism states, then each resource uses one common
    /// N/F scale. Therefore reversing organism iteration cannot change totals.
    pub fn exchange(
        &mut self,
        meshes: &mut [MaterialMesh],
        transport: &TransportParams,
        dt: f64,
    ) -> Vec<FiniteWorldDeliveryV1> {
        // The finite world owns the only positive N/F source in this mode.
        // Preserve the ordinary membrane transport for every mesh, but make
        // its N/F exterior concentration zero for this pass.  Restoring the
        // serialized exterior afterwards preserves the mechanical exterior
        // reference and prevents a hidden bath from feeding the organism.
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
        let mut requests = Vec::with_capacity(meshes.len() * self.resources.len());
        for organism_index in 0..meshes.len() {
            for resource_index in 0..self.resources.len() {
                requests.push(self.request(
                    organism_index,
                    resource_index,
                    &meshes[organism_index],
                    transport,
                    dt,
                ));
            }
        }

        // All requests are allocated against one pre-step inventory snapshot.
        // A single common scale per resource makes the result independent of
        // organism iteration order, even when the resource is oversubscribed.
        let availability: Vec<_> = self
            .resources
            .iter()
            .map(|resource| {
                (
                    resource.backing.region.n_mass,
                    resource.backing.region.f_mass,
                )
            })
            .collect();
        let scales: Vec<_> = (0..self.resources.len())
            .map(|resource_index| {
                let total_n: f64 = requests
                    .iter()
                    .filter(|candidate| candidate.resource_index == resource_index)
                    .map(|candidate| candidate.requested_n)
                    .sum();
                let total_f: f64 = requests
                    .iter()
                    .filter(|candidate| candidate.resource_index == resource_index)
                    .map(|candidate| candidate.requested_f)
                    .sum();
                let (available_n, available_f) = availability[resource_index];
                let n_scale = if total_n > 0.0 {
                    (available_n / total_n).min(1.0)
                } else {
                    1.0
                };
                let f_scale = if total_f > 0.0 {
                    (available_f / total_f).min(1.0)
                } else {
                    1.0
                };
                if self.transfer_enabled {
                    n_scale.min(f_scale).max(0.0)
                } else {
                    0.0
                }
            })
            .collect();
        let mut deliveries = Vec::with_capacity(requests.len());
        for request in &requests {
            let Some(resource) = self.resources.get_mut(request.resource_index) else {
                continue;
            };
            let scale = scales[request.resource_index];
            let n = request.requested_n * scale;
            let f = request.requested_f * scale;
            if let Some(mesh) = meshes.get_mut(request.organism_index) {
                let area = mesh.area();
                if area > 0.0 {
                    mesh.interior.n += n / area;
                    mesh.interior.f += f / area;
                }
            }
            resource.backing.region.n_mass = (resource.backing.region.n_mass - n).max(0.0);
            resource.backing.region.f_mass = (resource.backing.region.f_mass - f).max(0.0);
            deliveries.push(FiniteWorldDeliveryV1 {
                organism_index: request.organism_index,
                resource_index: request.resource_index,
                resource_id: resource.id.clone(),
                exposed_edges: request.exposed_edges,
                n_delivered: n,
                f_delivered: f,
                n_world_loss: n,
                f_world_loss: f,
                allocation_scale: scale,
                conservation_error: 0.0,
                nonfeeding_transport: nonfeeding
                    .get(request.organism_index)
                    .cloned()
                    .unwrap_or_default(),
            });
        }
        self.step += 1;
        deliveries
    }

    pub fn remove_all_inventory(&mut self) {
        for resource in &mut self.resources {
            resource.backing.remove_remaining_inventory();
        }
    }
}

/// Convert a world delivery into the existing V1-shaped ledger when an assay
/// needs the established field names. No chemistry is performed here.
pub fn delivery_as_v1(delivery: &FiniteWorldDeliveryV1) -> SpatialResourceStepLedgerV1 {
    SpatialResourceStepLedgerV1 {
        schema: crate::spatial_resource::SPATIAL_RESOURCE_STEP_LEDGER_SCHEMA_V1.to_string(),
        exposed_edges: delivery.exposed_edges,
        n_world_loss: delivery.n_world_loss,
        f_world_loss: delivery.f_world_loss,
        n_delivered: delivery.n_delivered,
        f_delivered: delivery.f_delivered,
        conservation_error: delivery.conservation_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{
        LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S,
    };

    fn mesh() -> MaterialMesh {
        let mut mesh = MaterialMesh::seed_regular(
            24,
            5.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                n: 0.1,
                f: 0.1,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        );
        mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
        mesh
    }

    #[test]
    fn shared_allocation_is_order_independent_and_conservative() {
        let mut a = mesh();
        let mut b = mesh();
        let mut first = FiniteWorldV1::new(vec![FiniteWorldResourceV1::new(
            "shared",
            [5.0, 0.0],
            2.0,
            1.0,
            1.0,
            2.0,
            2.0,
        )]);
        let mut second = first.clone();
        let before = first.total_n_mass();
        let mut first_meshes = vec![a.clone(), b.clone()];
        let mut second_meshes = vec![b, a];
        let left = first.exchange(&mut first_meshes, &TransportParams::default(), 0.02);
        let right = second.exchange(&mut second_meshes, &TransportParams::default(), 0.02);
        assert!(
            (left.iter().map(|x| x.n_delivered).sum::<f64>()
                - right.iter().map(|x| x.n_delivered).sum::<f64>())
            .abs()
                < 1e-12
        );
        assert!(
            (before - first.total_n_mass() - left.iter().map(|x| x.n_delivered).sum::<f64>()).abs()
                < 1e-12
        );
        assert_eq!(first.step, 1);
        a = first_meshes.remove(0);
        assert!(a.interior.n >= 0.1);
    }

    #[test]
    fn disabled_transfer_has_no_world_or_organism_influx() {
        let mut body = mesh();
        let before = body.interior;
        let mut world = FiniteWorldV1::new(vec![FiniteWorldResourceV1::new(
            "finite",
            [5.0, 0.0],
            2.0,
            10.0,
            10.0,
            2.0,
            2.0,
        )]);
        world.transfer_enabled = false;
        let before_world = world.total_n_mass();
        let deliveries = world.exchange(
            std::slice::from_mut(&mut body),
            &TransportParams::default(),
            0.02,
        );
        assert!(deliveries
            .iter()
            .all(|x| x.n_delivered == 0.0 && x.f_delivered == 0.0));
        assert!(deliveries.iter().all(|x| {
            x.nonfeeding_transport.n_in == 0.0
                && x.nonfeeding_transport.f_in == 0.0
        }));
        assert_eq!(world.total_n_mass(), before_world);
        assert!(body.interior.n <= before.n);
        assert!(body.interior.f <= before.f);
    }
}
