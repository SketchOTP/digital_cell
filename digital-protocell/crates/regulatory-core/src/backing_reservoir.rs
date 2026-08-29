//! DC-DEV-020-M1-R5: finite backing capacity for the accepted R4 boundary.
//!
//! This is a world-side capacity contract. It fixes the R4 boundary
//! concentration while scaling only the finite inventory, performs no
//! replenishment, and delegates every local transport decision to the
//! unchanged V1 finite-resource region.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::TransportParams;
use serde::{Deserialize, Serialize};

use crate::coupled_resource::{apply_coupled_delivery, CoupledSpatialResourceStepLedgerV1};
use crate::spatial_resource::{FiniteSpatialResourceRegionV1, SpatialResourceStepLedgerV1};

pub const FINITE_SPATIAL_BACKING_RESERVOIR_SCHEMA_V1: &str = "FINITE_SPATIAL_BACKING_RESERVOIR_V1";

/// Finite environmental inventory with R4 boundary concentration held fixed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiniteSpatialBackingReservoirV1 {
    pub schema: String,
    pub region: FiniteSpatialResourceRegionV1,
    pub initial_n_mass: f64,
    pub initial_f_mass: f64,
    pub fixed_boundary_n_concentration: f64,
    pub fixed_boundary_f_concentration: f64,
    pub replenishment_events: u64,
}

impl FiniteSpatialBackingReservoirV1 {
    pub fn new(
        center: [f64; 2],
        radius: f64,
        n_mass: f64,
        f_mass: f64,
        boundary_n_concentration: f64,
        boundary_f_concentration: f64,
    ) -> Self {
        let n_mass = n_mass.max(0.0);
        let f_mass = f_mass.max(0.0);
        let fixed_boundary_n_concentration = boundary_n_concentration.max(0.0);
        let fixed_boundary_f_concentration = boundary_f_concentration.max(0.0);
        let mut region = FiniteSpatialResourceRegionV1::new(center, radius, n_mass, f_mass);
        region.boundary_n_concentration = fixed_boundary_n_concentration;
        region.boundary_f_concentration = fixed_boundary_f_concentration;
        Self {
            schema: FINITE_SPATIAL_BACKING_RESERVOIR_SCHEMA_V1.to_string(),
            region,
            initial_n_mass: n_mass,
            initial_f_mass: f_mass,
            fixed_boundary_n_concentration,
            fixed_boundary_f_concentration,
            replenishment_events: 0,
        }
    }

    pub fn uptake(
        &mut self,
        mesh: &mut MaterialMesh,
        transport: &TransportParams,
        dt: f64,
    ) -> SpatialResourceStepLedgerV1 {
        self.region.uptake(mesh, transport, dt)
    }

    pub fn coupled_uptake(
        &mut self,
        mesh: &mut MaterialMesh,
        transport: &TransportParams,
        dt: f64,
    ) -> CoupledSpatialResourceStepLedgerV1 {
        let v1 = self.uptake(mesh, transport, dt);
        apply_coupled_delivery(mesh, v1)
    }

    pub fn remove_remaining_inventory(&mut self) {
        self.region.n_mass = 0.0;
        self.region.f_mass = 0.0;
    }

    pub fn total_mass(&self) -> f64 {
        self.region.total_mass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, DEFAULT_RHO_S};

    const CENTER: [f64; 2] = [4.8, 0.0];
    const RADIUS: f64 = 1.5;
    const DT: f64 = 0.02;
    const R4_MASS: f64 = 14.588954880632265;
    const R4_CONCENTRATION: f64 = 2.063914918930895;

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

    #[test]
    fn backing_reservoir_scales_capacity_without_changing_v1_first_step() {
        let mut v1_mesh = mesh();
        let mut backing_mesh = v1_mesh.clone();
        let mut v1 = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, R4_MASS, R4_MASS);
        let mut backing = FiniteSpatialBackingReservoirV1::new(
            CENTER,
            RADIUS,
            R4_MASS * 50.0 / 3.0,
            R4_MASS * 50.0 / 3.0,
            R4_CONCENTRATION,
            R4_CONCENTRATION,
        );
        let a = v1.uptake(&mut v1_mesh, &TransportParams::default(), DT);
        let b = backing.uptake(&mut backing_mesh, &TransportParams::default(), DT);
        assert_eq!(a.exposed_edges, b.exposed_edges);
        assert_eq!(a.n_delivered, b.n_delivered);
        assert_eq!(a.f_delivered, b.f_delivered);
        assert_eq!(a.n_world_loss, b.n_world_loss);
        assert_eq!(a.f_world_loss, b.f_world_loss);
        assert_eq!(a.conservation_error, b.conservation_error);
        assert_eq!(backing.replenishment_events, 0);
    }

    #[test]
    fn backing_reservoir_never_replenishes_and_can_be_removed() {
        let mut backing = FiniteSpatialBackingReservoirV1::new(
            CENTER,
            RADIUS,
            R4_MASS * 50.0 / 3.0,
            R4_MASS * 50.0 / 3.0,
            R4_CONCENTRATION,
            R4_CONCENTRATION,
        );
        assert_eq!(backing.replenishment_events, 0);
        backing.remove_remaining_inventory();
        assert_eq!(backing.total_mass(), 0.0);
    }
}
