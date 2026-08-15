//! DC-DEV-003: observer-only regulatory-state continuity through local remeshing.
//!
//! This module does not decide whether growth or remeshing occurs.  It observes
//! immutable before/after material frames and transfers the existing local
//! activity field only through deterministic nearest-local correspondences.
//! Fission and unknown topology events are explicit fail-closed boundaries.

use crate::{
    stable_json_hash, LocalMaterialFrameV1, LocalPatchInputV1, RegulatoryError, RegulatoryParamsV1,
    RegulatoryStateV1, CLOSED_RING_TOPOLOGY_V1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONTINUITY_FRAME_SCHEMA_V1: &str = "digital_cell_continuity_material_frame_v1";
pub const TOPOLOGY_MAPPING_SCHEMA_V1: &str = "digital_cell_local_topology_mapping_v1";
pub const CONTINUITY_LEDGER_SCHEMA_V1: &str = "digital_cell_regulatory_continuity_ledger_v1";

/// A material vertex plus the local signal observed at that vertex.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityPatchV1 {
    pub patch_index: usize,
    pub position: [f64; 2],
    pub previous_neighbor_index: usize,
    pub next_neighbor_index: usize,
    pub raw_stimulus: f64,
    pub accepted_dt: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityMaterialFrameV1 {
    pub schema: String,
    pub topology_size: usize,
    pub topology_identity: String,
    pub patches: Vec<ContinuityPatchV1>,
}

impl ContinuityMaterialFrameV1 {
    pub fn from_positions_and_stimuli(positions: &[[f64; 2]], stimuli: &[f64], dt: f64) -> Self {
        let n = positions.len();
        let patches = positions
            .iter()
            .enumerate()
            .map(|(i, position)| ContinuityPatchV1 {
                patch_index: i,
                position: *position,
                previous_neighbor_index: (i + n.saturating_sub(1)) % n.max(1),
                next_neighbor_index: (i + 1) % n.max(1),
                raw_stimulus: stimuli.get(i).copied().unwrap_or(0.0),
                accepted_dt: dt,
            })
            .collect();
        Self {
            schema: CONTINUITY_FRAME_SCHEMA_V1.to_string(),
            topology_size: n,
            topology_identity: CLOSED_RING_TOPOLOGY_V1.to_string(),
            patches,
        }
    }

    pub fn local_frame(&self) -> LocalMaterialFrameV1 {
        LocalMaterialFrameV1 {
            schema: crate::LOCAL_MATERIAL_FRAME_SCHEMA_V1.to_string(),
            topology_size: self.topology_size,
            topology_identity: self.topology_identity.clone(),
            patches: self
                .patches
                .iter()
                .map(|patch| LocalPatchInputV1 {
                    patch_index: patch.patch_index,
                    previous_neighbor_index: patch.previous_neighbor_index,
                    next_neighbor_index: patch.next_neighbor_index,
                    raw_stimulus: patch.raw_stimulus,
                    accepted_dt: patch.accepted_dt,
                })
                .collect(),
        }
    }

    fn validate(&self, params: &RegulatoryParamsV1) -> Result<(), ContinuityError> {
        if self.schema != CONTINUITY_FRAME_SCHEMA_V1
            || self.topology_identity != CLOSED_RING_TOPOLOGY_V1
            || self.topology_size < 3
            || self.patches.len() != self.topology_size
        {
            return Err(ContinuityError::InvalidFrame(
                "continuity frame schema, topology, or patch count is invalid".to_string(),
            ));
        }
        for (i, patch) in self.patches.iter().enumerate() {
            if patch.patch_index != i
                || patch.previous_neighbor_index
                    != (i + self.topology_size - 1) % self.topology_size
                || patch.next_neighbor_index != (i + 1) % self.topology_size
                || patch.position.iter().any(|value| !value.is_finite())
                || !patch.raw_stimulus.is_finite()
                || !(0.0..=1.0).contains(&patch.raw_stimulus)
                || !patch.accepted_dt.is_finite()
                || (patch.accepted_dt - params.dt).abs() > 1e-12
            {
                return Err(ContinuityError::InvalidFrame(
                    "continuity frame contains invalid local data".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyEventV1 {
    Initial,
    Stable,
    Split,
    Merge,
    Fission,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyMappingV1 {
    pub schema: String,
    pub old_topology_size: usize,
    pub new_topology_size: usize,
    pub event: TopologyEventV1,
    /// Each new vertex receives state from one old vertex in its local region.
    pub new_to_old: Vec<usize>,
    pub maximum_mapping_distance: f64,
    pub mapping_rule: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityStepLedgerV1 {
    pub schema: String,
    pub step_index: u64,
    pub event: TopologyEventV1,
    pub old_topology_size: usize,
    pub new_topology_size: usize,
    pub mapping_hash: String,
    pub frame_hash: String,
    pub before_state_hash: String,
    pub after_state_hash: String,
}

#[derive(Debug, Error)]
pub enum ContinuityError {
    #[error("invalid continuity frame: {0}")]
    InvalidFrame(String),
    #[error("unsupported topology event: {0:?}")]
    UnsupportedTopologyEvent(TopologyEventV1),
    #[error("local topology mapping unavailable: {0}")]
    MappingUnavailable(String),
    #[error("regulatory state error: {0}")]
    Regulatory(#[from] RegulatoryError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityNetworkV1 {
    pub params: RegulatoryParamsV1,
    pub state: RegulatoryStateV1,
    pub previous_frame: ContinuityMaterialFrameV1,
    pub ledger: Vec<ContinuityStepLedgerV1>,
}

impl ContinuityNetworkV1 {
    pub fn new(
        initial_frame: ContinuityMaterialFrameV1,
        provenance_seed: Option<u64>,
    ) -> Result<Self, ContinuityError> {
        let params = RegulatoryParamsV1::default();
        initial_frame.validate(&params)?;
        Ok(Self {
            state: RegulatoryStateV1::new(initial_frame.topology_size, provenance_seed),
            params,
            previous_frame: initial_frame,
            ledger: Vec::new(),
        })
    }

    pub fn step(
        &mut self,
        frame: ContinuityMaterialFrameV1,
        event: TopologyEventV1,
    ) -> Result<TopologyMappingV1, ContinuityError> {
        frame.validate(&self.params)?;
        if matches!(
            event,
            TopologyEventV1::Initial | TopologyEventV1::Fission | TopologyEventV1::Unknown
        ) {
            return Err(ContinuityError::UnsupportedTopologyEvent(event));
        }
        validate_event_sizes(
            self.previous_frame.topology_size,
            frame.topology_size,
            event,
        )?;
        let mapping = derive_local_mapping(&self.previous_frame, &frame, event)?;
        if mapping.old_topology_size != self.state.activity.len() {
            return Err(ContinuityError::MappingUnavailable(
                "old frame and regulatory state lengths diverged".to_string(),
            ));
        }

        let before = stable_json_hash(&self.state)?;
        let mapped_activity: Vec<f64> = mapping
            .new_to_old
            .iter()
            .map(|old_index| self.state.activity[*old_index])
            .collect();
        let local = frame.local_frame();
        let frame_hash = stable_json_hash(&frame)?;
        let mapping_hash = stable_json_hash(&mapping)?;
        let mut next = vec![0.0; frame.topology_size];
        for i in 0..frame.topology_size {
            let patch = &local.patches[i];
            let neighbor_mean = 0.5
                * (mapped_activity[patch.previous_neighbor_index]
                    + mapped_activity[patch.next_neighbor_index]);
            let derivative = self.params.k_neighbor * (neighbor_mean - mapped_activity[i])
                + self.params.k_stimulus * patch.raw_stimulus * (1.0 - mapped_activity[i])
                - self.params.k_decay * mapped_activity[i];
            next[i] = (mapped_activity[i] + self.params.dt * derivative).clamp(0.0, 1.0);
        }
        self.state.activity = next;
        self.state.step_index = self.state.step_index.saturating_add(1);
        let after = stable_json_hash(&self.state)?;
        self.ledger.push(ContinuityStepLedgerV1 {
            schema: CONTINUITY_LEDGER_SCHEMA_V1.to_string(),
            step_index: self.state.step_index,
            event,
            old_topology_size: mapping.old_topology_size,
            new_topology_size: mapping.new_topology_size,
            mapping_hash,
            frame_hash,
            before_state_hash: before,
            after_state_hash: after,
        });
        self.previous_frame = frame;
        Ok(mapping)
    }

    pub fn state_hash(&self) -> Result<String, ContinuityError> {
        Ok(stable_json_hash(&self.state)?)
    }
}

/// Derive a local mapping independently for each new vertex.  There is no
/// global assignment or redistribution: a new vertex can inherit only from
/// its nearest old vertex, bounded by the adjacent local edge scale.
pub fn derive_local_mapping(
    old_frame: &ContinuityMaterialFrameV1,
    new_frame: &ContinuityMaterialFrameV1,
    event: TopologyEventV1,
) -> Result<TopologyMappingV1, ContinuityError> {
    if matches!(
        event,
        TopologyEventV1::Initial | TopologyEventV1::Fission | TopologyEventV1::Unknown
    ) {
        return Err(ContinuityError::UnsupportedTopologyEvent(event));
    }
    if old_frame.topology_size != old_frame.patches.len()
        || new_frame.topology_size != new_frame.patches.len()
        || old_frame.topology_size < 3
        || new_frame.topology_size < 3
    {
        return Err(ContinuityError::MappingUnavailable(
            "mapping requires two valid closed local frames".to_string(),
        ));
    }
    validate_event_sizes(old_frame.topology_size, new_frame.topology_size, event)?;
    let mut new_to_old = Vec::with_capacity(new_frame.topology_size);
    let mut maximum_mapping_distance: f64 = 0.0;
    for (new_index, new_patch) in new_frame.patches.iter().enumerate() {
        let mut best: Option<(usize, f64)> = None;
        for (old_index, old_patch) in old_frame.patches.iter().enumerate() {
            let distance = euclidean_distance(new_patch.position, old_patch.position);
            if best
                .map(|(_, best_distance)| distance < best_distance)
                .unwrap_or(true)
            {
                best = Some((old_index, distance));
            }
        }
        let Some((old_index, distance)) = best else {
            return Err(ContinuityError::MappingUnavailable(format!(
                "new patch {new_index} has no old local source"
            )));
        };
        let local_bound = 3.0
            * local_edge_scale(old_frame, old_index)
                .max(local_edge_scale(new_frame, new_index))
                .max(1e-9);
        if distance > local_bound {
            return Err(ContinuityError::MappingUnavailable(format!(
                "new patch {new_index} is outside its local mapping bound"
            )));
        }
        new_to_old.push(old_index);
        maximum_mapping_distance = maximum_mapping_distance.max(distance);
    }
    Ok(TopologyMappingV1 {
        schema: TOPOLOGY_MAPPING_SCHEMA_V1.to_string(),
        old_topology_size: old_frame.topology_size,
        new_topology_size: new_frame.topology_size,
        event,
        new_to_old,
        maximum_mapping_distance,
        mapping_rule: "independent_nearest_old_vertex_with_local_edge_bound".to_string(),
    })
}

fn validate_event_sizes(
    old_topology_size: usize,
    new_topology_size: usize,
    event: TopologyEventV1,
) -> Result<(), ContinuityError> {
    let valid = match event {
        TopologyEventV1::Stable => old_topology_size == new_topology_size,
        TopologyEventV1::Split => new_topology_size > old_topology_size,
        TopologyEventV1::Merge => new_topology_size < old_topology_size,
        TopologyEventV1::Initial | TopologyEventV1::Fission | TopologyEventV1::Unknown => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ContinuityError::MappingUnavailable(format!(
            "event {event:?} is inconsistent with topology sizes {old_topology_size}->{new_topology_size}"
        )))
    }
}

fn euclidean_distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn local_edge_scale(frame: &ContinuityMaterialFrameV1, index: usize) -> f64 {
    let patch = &frame.patches[index];
    let previous = &frame.patches[patch.previous_neighbor_index];
    let next = &frame.patches[patch.next_neighbor_index];
    0.5 * (euclidean_distance(patch.position, previous.position)
        + euclidean_distance(patch.position, next.position))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(n: usize, radius: f64) -> ContinuityMaterialFrameV1 {
        let positions: Vec<[f64; 2]> = (0..n)
            .map(|i| {
                let theta = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                [radius * theta.cos(), radius * theta.sin()]
            })
            .collect();
        ContinuityMaterialFrameV1::from_positions_and_stimuli(&positions, &vec![0.0; n], 0.02)
    }

    #[test]
    fn constant_field_is_preserved_through_local_split_and_merge() {
        let old = ring(8, 2.0);
        let mut split_positions = old
            .patches
            .iter()
            .map(|patch| patch.position)
            .collect::<Vec<_>>();
        let midpoint = [
            0.5 * (split_positions[0][0] + split_positions[1][0]),
            0.5 * (split_positions[0][1] + split_positions[1][1]),
        ];
        split_positions.insert(1, midpoint);
        let split = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &split_positions,
            &vec![0.0; 9],
            0.02,
        );
        let mut network = ContinuityNetworkV1::new(old.clone(), None).unwrap();
        network.state.activity.fill(0.37);
        network.step(split.clone(), TopologyEventV1::Split).unwrap();
        assert!(network
            .state
            .activity
            .iter()
            .all(|value| (*value - 0.37 * (1.0 - 0.02 * 0.5)).abs() < 1e-12));

        let mut merged_positions = split
            .patches
            .iter()
            .map(|patch| patch.position)
            .collect::<Vec<_>>();
        merged_positions.remove(1);
        let merged = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &merged_positions,
            &vec![0.0; 8],
            0.02,
        );
        network.step(merged, TopologyEventV1::Merge).unwrap();
        assert!(network
            .state
            .activity
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0));
    }

    #[test]
    fn local_pattern_maps_only_to_nearby_vertices() {
        let old = ring(8, 2.0);
        let mut new_positions = old
            .patches
            .iter()
            .map(|patch| patch.position)
            .collect::<Vec<_>>();
        new_positions.insert(1, [1.7, 1.0]);
        let new = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &new_positions,
            &vec![0.0; 9],
            0.02,
        );
        let mapping = derive_local_mapping(&old, &new, TopologyEventV1::Split).unwrap();
        assert_eq!(mapping.new_to_old[0], 0);
        assert!(mapping.new_to_old[1] <= 2);
        assert!(mapping.new_to_old[5] >= 3);
    }

    #[test]
    fn replay_is_deterministic_and_fission_is_fail_closed() {
        let old = ring(8, 2.0);
        let mut a = ContinuityNetworkV1::new(old.clone(), Some(1)).unwrap();
        let mut b = ContinuityNetworkV1::new(old.clone(), Some(1)).unwrap();
        for _ in 0..4 {
            a.step(old.clone(), TopologyEventV1::Stable).unwrap();
            b.step(old.clone(), TopologyEventV1::Stable).unwrap();
        }
        assert_eq!(a.state, b.state);
        assert_eq!(a.ledger, b.ledger);
        assert!(matches!(
            a.step(old, TopologyEventV1::Fission),
            Err(ContinuityError::UnsupportedTopologyEvent(
                TopologyEventV1::Fission
            ))
        ));
    }
}
