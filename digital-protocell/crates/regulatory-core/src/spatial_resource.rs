//! DC-DEV-008: finite local spatial N/F resource acquisition.
//!
//! This is the reusable post-Phase-1 world boundary for a finite static
//! resource region.  It exposes only local boundary segments, reuses the
//! existing chemistry-core permeability law, and transfers finite N/F mass
//! into the organism without changing chemistry-core equations or global
//! transport semantics.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::{permeability, TransportParams};
use serde::{Deserialize, Serialize};

pub const FINITE_SPATIAL_RESOURCE_REGION_SCHEMA_V1: &str = "dcdev008_finite_static_nf_region_v1";
pub const SPATIAL_RESOURCE_STEP_LEDGER_SCHEMA_V1: &str = "dcdev008_spatial_resource_step_ledger_v1";

/// A finite static circular region containing only N and F material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiniteSpatialResourceRegionV1 {
    pub schema: String,
    pub center: [f64; 2],
    pub radius: f64,
    pub material_volume: f64,
    pub boundary_n_concentration: f64,
    pub boundary_f_concentration: f64,
    pub n_mass: f64,
    pub f_mass: f64,
}

impl FiniteSpatialResourceRegionV1 {
    pub fn new(center: [f64; 2], radius: f64, n_mass: f64, f_mass: f64) -> Self {
        let material_volume = std::f64::consts::PI * radius * radius;
        let n_mass = n_mass.max(0.0);
        let f_mass = f_mass.max(0.0);
        Self {
            schema: FINITE_SPATIAL_RESOURCE_REGION_SCHEMA_V1.to_string(),
            center,
            radius,
            material_volume,
            boundary_n_concentration: if material_volume > 0.0 {
                n_mass / material_volume
            } else {
                0.0
            },
            boundary_f_concentration: if material_volume > 0.0 {
                f_mass / material_volume
            } else {
                0.0
            },
            n_mass,
            f_mass,
        }
    }

    /// Apply one accepted local acquisition step.
    ///
    /// Inward transfer is the positive part of the existing permeability law,
    /// capped by the finite source inventory.  Every delivered unit is
    /// removed from that same inventory, so the ledger exposes the
    /// world-loss/organism-delivery conservation boundary directly.
    pub fn uptake(
        &mut self,
        mesh: &mut MaterialMesh,
        transport: &TransportParams,
        dt: f64,
    ) -> SpatialResourceStepLedgerV1 {
        let mut ledger = SpatialResourceStepLedgerV1::default();
        if !mesh.alive || dt <= 0.0 {
            return ledger;
        }
        let area = mesh.area().max(1e-6);
        for edge in 0..mesh.n() {
            if !self.edge_exposed(mesh, edge) || mesh.edges[edge].ruptured {
                continue;
            }
            ledger.exposed_edges += 1;
            let theta = mesh.occupancy(edge);
            let segment = mesh.edge_length(edge);
            let n_delta = Self::inward_mass(
                self.n_mass,
                self.boundary_n_concentration,
                mesh.interior.n,
                permeability(theta, "N"),
                transport.k_flux,
                segment,
                dt,
            );
            let f_delta = Self::inward_mass(
                self.f_mass,
                self.boundary_f_concentration,
                mesh.interior.f,
                permeability(theta, "F"),
                transport.k_flux,
                segment,
                dt,
            );
            self.n_mass = (self.n_mass - n_delta).max(0.0);
            self.f_mass = (self.f_mass - f_delta).max(0.0);
            mesh.interior.n += n_delta / area;
            mesh.interior.f += f_delta / area;
            ledger.n_world_loss += n_delta;
            ledger.f_world_loss += f_delta;
            ledger.n_delivered += n_delta;
            ledger.f_delivered += f_delta;
        }
        ledger.conservation_error = (ledger.n_world_loss - ledger.n_delivered).abs()
            + (ledger.f_world_loss - ledger.f_delivered).abs();
        ledger
    }

    pub fn total_mass(&self) -> f64 {
        self.n_mass + self.f_mass
    }

    fn contains(&self, point: [f64; 2]) -> bool {
        (point[0] - self.center[0]).hypot(point[1] - self.center[1]) <= self.radius
    }

    fn edge_exposed(&self, mesh: &MaterialMesh, edge: usize) -> bool {
        let a = mesh.vertices[edge];
        let b = mesh.vertices[(edge + 1) % mesh.n()];
        self.contains([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5])
    }

    fn inward_mass(
        world_mass: f64,
        boundary_concentration: f64,
        interior_concentration: f64,
        permeability: f64,
        k_flux: f64,
        segment_length: f64,
        dt: f64,
    ) -> f64 {
        if world_mass <= 1e-12 || boundary_concentration <= 0.0 || dt <= 0.0 {
            return 0.0;
        }
        let requested = k_flux
            * permeability
            * (boundary_concentration - interior_concentration.max(0.0))
            * segment_length
            * dt;
        requested.max(0.0).min(world_mass)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialResourceStepLedgerV1 {
    pub schema: String,
    pub exposed_edges: usize,
    pub n_world_loss: f64,
    pub f_world_loss: f64,
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub conservation_error: f64,
}

impl Default for SpatialResourceStepLedgerV1 {
    fn default() -> Self {
        Self {
            schema: SPATIAL_RESOURCE_STEP_LEDGER_SCHEMA_V1.to_string(),
            exposed_edges: 0,
            n_world_loss: 0.0,
            f_world_loss: 0.0,
            n_delivered: 0.0,
            f_delivered: 0.0,
            conservation_error: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, DEFAULT_RHO_S};

    const RADIUS: f64 = 1.5;
    const CENTER: [f64; 2] = [4.8, 0.0];
    const N_MASS: f64 = 3.0;
    const F_MASS: f64 = 3.0;

    fn mesh() -> MaterialMesh {
        MaterialMesh::seed_regular(
            24,
            5.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem::default(),
            LumpedChem::default(),
            5.0,
        )
    }

    fn transport() -> TransportParams {
        TransportParams::default()
    }

    #[test]
    fn finite_inventory_never_becomes_negative() {
        let mut body = mesh();
        let mut region = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        for _ in 0..2_000 {
            region.uptake(&mut body, &transport(), 0.02);
        }
        assert!(region.n_mass >= 0.0);
        assert!(region.f_mass >= 0.0);
    }

    #[test]
    fn no_spatial_exposure_produces_zero_uptake() {
        let mut body = mesh();
        let before = body.interior;
        let mut region = FiniteSpatialResourceRegionV1::new([30.0, 30.0], RADIUS, N_MASS, F_MASS);
        let ledger = region.uptake(&mut body, &transport(), 0.02);
        assert_eq!(ledger.exposed_edges, 0);
        assert_eq!(ledger.n_delivered, 0.0);
        assert_eq!(ledger.f_delivered, 0.0);
        assert_eq!(body.interior.n, before.n);
        assert_eq!(body.interior.f, before.f);
    }

    #[test]
    fn local_exposure_transfers_both_resource_species() {
        let mut body = mesh();
        let mut region = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let ledger = region.uptake(&mut body, &transport(), 0.02);
        assert!(ledger.exposed_edges > 0);
        assert!(ledger.n_delivered > 0.0);
        assert!(ledger.f_delivered > 0.0);
        assert_eq!(ledger.conservation_error, 0.0);
    }

    #[test]
    fn world_loss_equals_organism_delivery() {
        let mut body = mesh();
        let mut region = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let ledger = region.uptake(&mut body, &transport(), 0.02);
        assert_eq!(ledger.n_world_loss, ledger.n_delivered);
        assert_eq!(ledger.f_world_loss, ledger.f_delivered);
        assert_eq!(ledger.conservation_error, 0.0);
    }

    #[test]
    fn exhaustion_permanently_stops_further_uptake() {
        let mut body = mesh();
        let mut region = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, 0.001, 0.001);
        for _ in 0..2_000 {
            region.uptake(&mut body, &transport(), 0.02);
        }
        assert_eq!(region.total_mass(), 0.0);
        let after_exhaustion = region.uptake(&mut body, &transport(), 0.02);
        assert_eq!(after_exhaustion.n_delivered, 0.0);
        assert_eq!(after_exhaustion.f_delivered, 0.0);
    }

    #[test]
    fn replay_is_deterministic() {
        let mut first_body = mesh();
        let mut second_body = mesh();
        let mut first_region = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let mut second_region = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let first = (0..120)
            .map(|_| first_region.uptake(&mut first_body, &transport(), 0.02))
            .collect::<Vec<_>>();
        let second = (0..120)
            .map(|_| second_region.uptake(&mut second_body, &transport(), 0.02))
            .collect::<Vec<_>>();
        assert_eq!(first, second);
        assert_eq!(first_region, second_region);
        assert_eq!(
            serde_json::to_value(&first_body).unwrap(),
            serde_json::to_value(&second_body).unwrap()
        );
    }
}
