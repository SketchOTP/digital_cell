//! DC-DEV-006: the smallest spatial external-world boundary.
//!
//! The world is deterministic inert geometry.  It observes local boundary
//! penetration, returns a bounded local force vector, and exposes one
//! non-semantic contact stimulus to the already-existing regulatory frame.
//! This module never writes organism coordinates; chemistry-core mechanics
//! remains the sole authority for movement.

use crate::ContinuityMaterialFrameV1;
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_mechanics::{MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SPATIAL_WORLD_SCHEMA_V1: &str = "digital_cell_static_spatial_world_v1";

/// Frozen contact force law: force magnitude per unit local penetration.
pub const CONTACT_STIFFNESS_PER_LENGTH: f64 = 0.5;
/// The contact signal is normalized by the bounded mechanics-hook force scale.
pub const CONTACT_FORCE_NORMALIZATION: f64 = MAX_EXTERNAL_FORCE_PER_VERTEX;
/// Alias kept explicit in evidence and preregistration.
pub const CONTACT_STIMULUS_NORMALIZATION: f64 = CONTACT_FORCE_NORMALIZATION;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticObstacleV1 {
    pub schema: String,
    pub center: [f64; 2],
    pub radius: f64,
}

impl StaticObstacleV1 {
    pub fn new(center: [f64; 2], radius: f64) -> Result<Self, SpatialError> {
        if center.iter().any(|value| !value.is_finite()) || !radius.is_finite() || radius <= 0.0 {
            return Err(SpatialError::InvalidObstacle);
        }
        Ok(Self {
            schema: SPATIAL_WORLD_SCHEMA_V1.to_string(),
            center,
            radius,
        })
    }

    /// Observe local contact between mesh boundary vertices and this inert
    /// circular obstacle.  No position or material field is modified.
    pub fn observe(
        &self,
        mesh: &MaterialMesh,
        mechanics: &MechParams,
    ) -> Result<ContactObservationV1, SpatialError> {
        if self.schema != SPATIAL_WORLD_SCHEMA_V1
            || self.center.iter().any(|value| !value.is_finite())
            || !self.radius.is_finite()
            || self.radius <= 0.0
            || !mechanics.dt.is_finite()
            || mechanics.dt < 0.0
            || mesh.n() < 3
        {
            return Err(SpatialError::InvalidObstacle);
        }

        let mut external_force = vec![[0.0, 0.0]; mesh.n()];
        let mut contact_stimulus = vec![0.0; mesh.n()];
        let mut penetration = vec![0.0; mesh.n()];
        for (index, vertex) in mesh.vertices.iter().copied().enumerate() {
            if vertex.iter().any(|value| !value.is_finite()) {
                return Err(SpatialError::InvalidMesh);
            }
            let dx = vertex[0] - self.center[0];
            let dy = vertex[1] - self.center[1];
            let distance = dx.hypot(dy);
            let local_penetration = (self.radius - distance).max(0.0);
            if local_penetration <= 0.0 {
                continue;
            }
            let (nx, ny) = if distance > 1e-12 {
                (dx / distance, dy / distance)
            } else {
                // Deterministic outward normal for exact center coincidence.
                (1.0, 0.0)
            };
            let magnitude = (CONTACT_STIFFNESS_PER_LENGTH * local_penetration)
                .clamp(0.0, MAX_EXTERNAL_FORCE_PER_VERTEX);
            external_force[index] = [magnitude * nx, magnitude * ny];
            penetration[index] = local_penetration;
            contact_stimulus[index] = (magnitude / CONTACT_STIMULUS_NORMALIZATION).clamp(0.0, 1.0);
        }

        Ok(ContactObservationV1 {
            schema: SPATIAL_WORLD_SCHEMA_V1.to_string(),
            external_force,
            contact_stimulus,
            penetration,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactObservationV1 {
    pub schema: String,
    pub external_force: Vec<[f64; 2]>,
    pub contact_stimulus: Vec<f64>,
    pub penetration: Vec<f64>,
}

impl ContactObservationV1 {
    pub fn validate(&self, vertex_count: usize) -> Result<(), SpatialError> {
        if self.schema != SPATIAL_WORLD_SCHEMA_V1
            || self.external_force.len() != vertex_count
            || self.contact_stimulus.len() != vertex_count
            || self.penetration.len() != vertex_count
            || self.external_force.iter().any(|force| {
                force.iter().any(|value| !value.is_finite())
                    || force[0].hypot(force[1]) > MAX_EXTERNAL_FORCE_PER_VERTEX
            })
            || self
                .contact_stimulus
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || self
                .penetration
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(SpatialError::InvalidObservation);
        }
        Ok(())
    }

    pub fn contacted_indices(&self) -> Vec<usize> {
        self.contact_stimulus
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value > 0.0).then_some(index))
            .collect()
    }
}

/// Add the one local external signal to the already-existing local regulatory
/// activation path.  With a zero contact vector the returned frame is an
/// exact clone of the input frame.
pub fn augment_frame_with_contact(
    frame: &ContinuityMaterialFrameV1,
    contact_stimulus: &[f64],
) -> Result<ContinuityMaterialFrameV1, SpatialError> {
    if frame.patches.len() != contact_stimulus.len()
        || contact_stimulus
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(SpatialError::InvalidObservation);
    }
    let mut augmented = frame.clone();
    for (patch, contact) in augmented.patches.iter_mut().zip(contact_stimulus) {
        patch.raw_stimulus = (patch.raw_stimulus + contact).clamp(0.0, 1.0);
    }
    Ok(augmented)
}

#[derive(Debug, Error, PartialEq)]
pub enum SpatialError {
    #[error("static obstacle geometry is invalid")]
    InvalidObstacle,
    #[error("organism mesh geometry is invalid")]
    InvalidMesh,
    #[error("contact observation is invalid or out of bounds")]
    InvalidObservation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};

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
    fn deterministic_local_contact_is_bounded_and_zero_elsewhere() {
        let body = mesh();
        let obstacle = StaticObstacleV1::new([5.0, 0.0], 0.9).unwrap();
        let first = obstacle.observe(&body, &MechParams::default()).unwrap();
        let second = obstacle.observe(&body, &MechParams::default()).unwrap();
        assert_eq!(first, second);
        first.validate(body.n()).unwrap();
        assert!(!first.contacted_indices().is_empty());
        assert!(first.contacted_indices().len() < body.n());
        assert!(first
            .contact_stimulus
            .iter()
            .all(|value| (0.0..=1.0).contains(value)));
        assert!(first
            .contact_stimulus
            .iter()
            .zip(&first.external_force)
            .all(|(stimulus, force)| (*stimulus == 0.0) == (force[0] == 0.0 && force[1] == 0.0)));
    }

    #[test]
    fn far_obstacle_is_exactly_zero_contact() {
        let body = mesh();
        let obstacle = StaticObstacleV1::new([100.0, 100.0], 1.0).unwrap();
        let observation = obstacle.observe(&body, &MechParams::default()).unwrap();
        assert!(observation
            .external_force
            .iter()
            .all(|force| *force == [0.0, 0.0]));
        assert!(observation
            .contact_stimulus
            .iter()
            .all(|value| *value == 0.0));
    }
}
