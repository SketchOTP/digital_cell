//! DC-DEV-008: finite local spatial N/F material acquisition.
//!
//! This is an environment boundary, not a new chemistry.  It reuses the
//! certified membrane permeability law on exposed boundary segments and
//! transfers only finite material from the world inventory into the existing
//! `MaterialMesh::interior` pools.

use crate::material_mesh::MaterialMesh;
use crate::mesh_transport::{mean_occupancy, permeability, TransportParams};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SPATIAL_RESOURCE_SCHEMA_V1: &str = "digital_cell_finite_spatial_resource_v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiniteResourceRegionV1 {
    pub schema: String,
    pub center: [f64; 2],
    pub radius: f64,
    pub n_inventory: f64,
    pub f_inventory: f64,
}

impl FiniteResourceRegionV1 {
    pub fn new(
        center: [f64; 2],
        radius: f64,
        n_inventory: f64,
        f_inventory: f64,
    ) -> Result<Self, ResourceError> {
        if center.iter().any(|v| !v.is_finite())
            || !radius.is_finite()
            || radius <= 0.0
            || !n_inventory.is_finite()
            || !f_inventory.is_finite()
            || n_inventory < 0.0
            || f_inventory < 0.0
        {
            return Err(ResourceError::InvalidRegion);
        }
        Ok(Self {
            schema: SPATIAL_RESOURCE_SCHEMA_V1.to_string(),
            center,
            radius,
            n_inventory,
            f_inventory,
        })
    }

    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn contains(&self, point: [f64; 2]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        dx * dx + dy * dy <= self.radius * self.radius
    }

    pub fn uptake_step(
        &mut self,
        mesh: &mut MaterialMesh,
        params: &TransportParams,
        dt: f64,
    ) -> Result<ResourceUptakeLedger, ResourceError> {
        if self.schema != SPATIAL_RESOURCE_SCHEMA_V1
            || !dt.is_finite()
            || dt < 0.0
            || !mesh.alive
            || mesh.n() < 3
            || !params.k_flux.is_finite()
            || params.k_flux < 0.0
        {
            return Err(ResourceError::InvalidInput);
        }

        let area = mesh.area();
        if !area.is_finite() || area <= 0.0 {
            return Err(ResourceError::InvalidInput);
        }
        let exposed_length: f64 = (0..mesh.n())
            .filter(|&i| {
                let a = mesh.vertices[i];
                let b = mesh.vertices[(i + 1) % mesh.n()];
                self.contains([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5])
            })
            .map(|i| mesh.edge_length(i))
            .sum();
        if exposed_length <= 0.0 || dt == 0.0 {
            return Ok(ResourceUptakeLedger::default());
        }

        let theta = mean_occupancy(mesh);
        let uptake = |inventory: f64, interior: f64| {
            let world_concentration = inventory / self.area();
            let requested_mass = params.k_flux
                * permeability(theta, "N")
                * (world_concentration - interior).max(0.0)
                * exposed_length
                * dt;
            requested_mass.max(0.0).min(inventory)
        };
        let n_mass = uptake(self.n_inventory, mesh.interior.n);
        let f_mass = {
            let world_concentration = self.f_inventory / self.area();
            (params.k_flux
                * permeability(theta, "F")
                * (world_concentration - mesh.interior.f).max(0.0)
                * exposed_length
                * dt)
                .max(0.0)
                .min(self.f_inventory)
        };
        self.n_inventory -= n_mass;
        self.f_inventory -= f_mass;
        mesh.interior.n += n_mass / area;
        mesh.interior.f += f_mass / area;
        Ok(ResourceUptakeLedger {
            exposed_length,
            n_mass,
            f_mass,
            world_mass_loss: n_mass + f_mass,
            organism_mass_gain: n_mass + f_mass,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceUptakeLedger {
    pub exposed_length: f64,
    pub n_mass: f64,
    pub f_mass: f64,
    pub world_mass_loss: f64,
    pub organism_mass_gain: f64,
}

impl ResourceUptakeLedger {
    pub fn mass_conservative(self, tolerance: f64) -> bool {
        (self.world_mass_loss - self.organism_mass_gain).abs() <= tolerance
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ResourceError {
    #[error("resource region is invalid")]
    InvalidRegion,
    #[error("resource input is invalid")]
    InvalidInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material_mesh::{LumpedChem, DEFAULT_RHO_S};

    fn mesh() -> MaterialMesh {
        MaterialMesh::seed_regular(
            24,
            5.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                n: 0.0,
                f: 0.0,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        )
    }

    #[test]
    fn uptake_is_local_finite_and_mass_conservative() {
        let mut body = mesh();
        let mut region = FiniteResourceRegionV1::new([5.0, 0.0], 1.0, 2.0, 3.0).unwrap();
        let before = region.clone();
        let ledger = region
            .uptake_step(&mut body, &TransportParams::default(), 0.1)
            .unwrap();
        assert!(ledger.exposed_length > 0.0);
        assert!(ledger.n_mass > 0.0 && ledger.f_mass > 0.0);
        assert!(ledger.mass_conservative(1e-12));
        assert!(region.n_inventory < before.n_inventory);
        assert!(region.f_inventory < before.f_inventory);
    }

    #[test]
    fn noncontact_and_empty_regions_do_not_supply_material() {
        let mut body = mesh();
        let mut far = FiniteResourceRegionV1::new([100.0, 100.0], 1.0, 2.0, 3.0).unwrap();
        let far_ledger = far
            .uptake_step(&mut body, &TransportParams::default(), 0.1)
            .unwrap();
        assert_eq!(far_ledger.n_mass, 0.0);
        assert_eq!(far_ledger.f_mass, 0.0);
        let mut empty = FiniteResourceRegionV1::new([5.0, 0.0], 1.0, 0.0, 0.0).unwrap();
        let empty_ledger = empty
            .uptake_step(&mut body, &TransportParams::default(), 0.1)
            .unwrap();
        assert_eq!(empty_ledger.world_mass_loss, 0.0);
    }
}
