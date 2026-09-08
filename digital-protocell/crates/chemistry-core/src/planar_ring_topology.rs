//! Robust planar half-edge topology for one simple material boundary.
//!
//! This representation owns connectivity only.  Material, chemistry, and
//! geometry remain authoritative in `MaterialMesh`; rebuilding after a
//! conservative remesh is exact and introduces no interpolation.

use crate::material_mesh::MaterialMesh;
use crate::mesh_fission::{try_local_segment_fission, FissionEvent, FissionParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryHalfEdge {
    pub origin: usize,
    pub next: usize,
    pub prev: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanarRingTopology {
    pub half_edges: Vec<BoundaryHalfEdge>,
}

impl PlanarRingTopology {
    pub fn from_mesh(mesh: &MaterialMesh) -> Option<Self> {
        if mesh.n() < 3
            || mesh.edges.len() != mesh.n()
            || !crate::mesh_self_contact::polygon_simple(&mesh.vertices)
        {
            return None;
        }
        let n = mesh.n();
        let half_edges = (0..n)
            .map(|i| BoundaryHalfEdge {
                origin: i,
                next: (i + 1) % n,
                prev: (i + n - 1) % n,
            })
            .collect();
        Some(Self { half_edges })
    }

    pub fn valid_for(&self, mesh: &MaterialMesh) -> bool {
        self.half_edges.len() == mesh.n()
            && crate::mesh_self_contact::polygon_simple(&mesh.vertices)
            && self.half_edges.iter().enumerate().all(|(i, h)| {
                h.origin == i
                    && h.next == (i + 1) % mesh.n()
                    && h.prev == (i + mesh.n() - 1) % mesh.n()
            })
    }

    pub fn try_local_scission(
        &self,
        mesh: &MaterialMesh,
        params: &FissionParams,
    ) -> Option<(MaterialMesh, MaterialMesh, FissionEvent)> {
        self.valid_for(mesh)
            .then(|| try_local_segment_fission(mesh, params))
            .flatten()
    }
}

/// Half-edge fallback remesh: accept the frozen remesh result only when its
/// replacement chords preserve the simple boundary.  If a legacy merge would
/// cross the boundary, retain the pre-merge ring and apply only conservative
/// edge splits.  No geometric threshold is changed.
pub fn remesh_preserving_simple(mesh: &mut MaterialMesh) -> (usize, usize, bool) {
    let before = mesh.clone();
    let (splits, merges) = crate::mesh_mechanics::remesh(mesh);
    if crate::mesh_self_contact::polygon_simple(&mesh.vertices) {
        return (splits, merges, false);
    }
    *mesh = before.clone();
    let splits = crate::mesh_mechanics::remesh_split(mesh);
    if crate::mesh_self_contact::polygon_simple(&mesh.vertices) {
        (splits, 0, true)
    } else {
        *mesh = before;
        (0, 0, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_to_one_ring_survives_index_rotation() {
        let mut mesh = MaterialMesh::seed_regular(
            12,
            5.0,
            0.0,
            0.0,
            1.0,
            0.5,
            Default::default(),
            Default::default(),
            0.0,
        );
        let topology = PlanarRingTopology::from_mesh(&mesh).unwrap();
        assert!(topology.valid_for(&mesh));
        mesh.vertices.rotate_left(3);
        mesh.edges.rotate_left(3);
        let rotated = PlanarRingTopology::from_mesh(&mesh).unwrap();
        assert!(rotated.valid_for(&mesh));
    }
}
