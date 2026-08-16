//! DC-DEV-002: a bounded, observer-only local regulatory substrate.
//!
//! The production update path operates only on [`LocalMaterialFrameV1`].  The
//! mesh adapter is intentionally a separate module and accepts an immutable
//! `MaterialMesh` reference; no regulator method accepts a mesh or mutable
//! organism state.  This crate therefore computes internal regulatory state
//! and provenance only.  The DC-DEV-004 contractility adapter is a separate,
//! explicitly authorized boundary that consumes this state through one local
//! edge-tension rule; it does not add a semantic action or central controller.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod continuity;
pub mod contractility;
pub mod endogenous_polarity;
pub mod plasticity;
pub mod spatial;
pub mod spatial_resource;
pub mod stick_slip_traction;

pub use contractility::{
    apply_local_contractility, apply_local_contractility_with_external_forces, ContractilityError,
    ContractilityParamsV1, ContractilityStepLedgerV1, CONTRACTILITY_SCHEMA_V1,
    FROZEN_MAX_ACTIVE_TENSION, FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
};

pub use continuity::{
    ContinuityMaterialFrameV1, ContinuityNetworkV1, ContinuityPatchV1, ContinuityStepLedgerV1,
    TopologyEventV1, TopologyMappingV1,
};

pub use endogenous_polarity::{
    EndogenousPolarityError, EndogenousPolarityV1, PolarityParametersV1, PolarityStepLedgerV1,
    ENDOGENOUS_POLARITY_SCHEMA_V1, FROZEN_DIFFUSION_COEFFICIENT, FROZEN_DISSOCIATION_RATE,
    FROZEN_FEEDBACK_RATE, FROZEN_POLARITY_DT, FROZEN_SPONTANEOUS_ASSOCIATION_RATE,
    POLARITY_TOKEN_COUNT, SUPPORTED_POLARITY_TOPOLOGY,
};

pub use plasticity::{
    apply_local_plasticity, apply_local_plasticity_with_external_forces, PlasticityError,
    PlasticityParamsV1, PlasticityStateV1, PlasticityStepLedgerV1,
    FROZEN_ADAPTATION_LOAD_RATE_PER_TIME, FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
    PLASTICITY_SCHEMA_V1,
};

pub use spatial::{
    augment_frame_with_contact, ContactObservationV1, SpatialError, StaticObstacleV1,
    CONTACT_FORCE_NORMALIZATION, CONTACT_STIFFNESS_PER_LENGTH, CONTACT_STIMULUS_NORMALIZATION,
    SPATIAL_WORLD_SCHEMA_V1,
};

pub use spatial_resource::{
    FiniteSpatialResourceRegionV1, SpatialResourceStepLedgerV1,
    FINITE_SPATIAL_RESOURCE_REGION_SCHEMA_V1, SPATIAL_RESOURCE_STEP_LEDGER_SCHEMA_V1,
};

pub use stick_slip_traction::{
    apply_local_contractility_with_stick_slip, apply_stick_slip_to_legacy_mechanics,
    evaluate_contact, ContactLedgerV1, ContactRegimeV1, StickSlipError, StickSlipStepLedgerV1,
    StickSlipTractionParamsV1, FROZEN_KINETIC_TRACTION, FROZEN_STATIC_TRACTION_LIMIT,
    FROZEN_ZERO_MOTION_TOLERANCE, STICK_SLIP_TRACTION_SCHEMA_V1,
};

pub const LOCAL_MATERIAL_FRAME_SCHEMA_V1: &str = "digital_cell_local_material_frame_v1";
pub const REGULATORY_STATE_SCHEMA_V1: &str = "digital_cell_regulatory_state_v1";
pub const REGULATORY_PARAMS_SCHEMA_V1: &str = "digital_cell_regulatory_params_v1";
pub const REGULATORY_LEDGER_SCHEMA_V1: &str = "digital_cell_regulatory_step_ledger_v1";
pub const REGULATORY_EVIDENCE_SCHEMA_V1: &str = "digital_cell_regulatory_evidence_v1";
pub const CLOSED_RING_TOPOLOGY_V1: &str = "closed_ring_vertices_v1";

pub const FROZEN_K_NEIGHBOR: f64 = 2.0;
pub const FROZEN_K_STIMULUS: f64 = 4.0;
pub const FROZEN_K_DECAY: f64 = 0.5;
pub const FROZEN_DT: f64 = 0.02;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalPatchInputV1 {
    pub patch_index: usize,
    pub previous_neighbor_index: usize,
    pub next_neighbor_index: usize,
    pub raw_stimulus: f64,
    pub accepted_dt: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalMaterialFrameV1 {
    pub schema: String,
    pub topology_size: usize,
    pub topology_identity: String,
    pub patches: Vec<LocalPatchInputV1>,
}

impl LocalMaterialFrameV1 {
    pub fn from_patch_stimuli(stimuli: &[f64]) -> Self {
        let n = stimuli.len();
        let patches = stimuli
            .iter()
            .enumerate()
            .map(|(i, stimulus)| LocalPatchInputV1 {
                patch_index: i,
                previous_neighbor_index: (i + n.saturating_sub(1)) % n.max(1),
                next_neighbor_index: (i + 1) % n.max(1),
                raw_stimulus: *stimulus,
                accepted_dt: FROZEN_DT,
            })
            .collect();
        Self {
            schema: LOCAL_MATERIAL_FRAME_SCHEMA_V1.to_string(),
            topology_size: n,
            topology_identity: CLOSED_RING_TOPOLOGY_V1.to_string(),
            patches,
        }
    }

    fn validate(&self, params: &RegulatoryParamsV1) -> Result<(), RegulatoryError> {
        if self.schema != LOCAL_MATERIAL_FRAME_SCHEMA_V1 {
            return Err(RegulatoryError::InvalidFrame(
                "unsupported local material frame schema".to_string(),
            ));
        }
        if self.topology_identity != CLOSED_RING_TOPOLOGY_V1 {
            return Err(RegulatoryError::InvalidFrame(
                "unsupported topology identity".to_string(),
            ));
        }
        if self.topology_size < 3 || self.patches.len() != self.topology_size {
            return Err(RegulatoryError::InvalidFrame(
                "frame patch count must equal a legal mesh topology size".to_string(),
            ));
        }
        for (i, patch) in self.patches.iter().enumerate() {
            if patch.patch_index != i
                || patch.previous_neighbor_index
                    != (i + self.topology_size - 1) % self.topology_size
                || patch.next_neighbor_index != (i + 1) % self.topology_size
            {
                return Err(RegulatoryError::InvalidFrame(
                    "patch indices must describe the immediate closed-ring neighbors".to_string(),
                ));
            }
            if !patch.raw_stimulus.is_finite()
                || !(0.0..=1.0).contains(&patch.raw_stimulus)
                || !patch.accepted_dt.is_finite()
                || (patch.accepted_dt - params.dt).abs() > 1e-12
            {
                return Err(RegulatoryError::InvalidFrame(
                    "stimulus and accepted dt must be finite and within the frozen bounds"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryParamsV1 {
    pub schema: String,
    pub k_neighbor: f64,
    pub k_stimulus: f64,
    pub k_decay: f64,
    pub dt: f64,
}

impl Default for RegulatoryParamsV1 {
    fn default() -> Self {
        Self {
            schema: REGULATORY_PARAMS_SCHEMA_V1.to_string(),
            k_neighbor: FROZEN_K_NEIGHBOR,
            k_stimulus: FROZEN_K_STIMULUS,
            k_decay: FROZEN_K_DECAY,
            dt: FROZEN_DT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryStateV1 {
    pub schema: String,
    pub initial_vertex_count: usize,
    pub step_index: u64,
    pub activity: Vec<f64>,
    pub provenance_seed: Option<u64>,
}

impl RegulatoryStateV1 {
    pub fn new(initial_vertex_count: usize, provenance_seed: Option<u64>) -> Self {
        Self {
            schema: REGULATORY_STATE_SCHEMA_V1.to_string(),
            initial_vertex_count,
            step_index: 0,
            activity: vec![0.0; initial_vertex_count],
            provenance_seed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateOrderV1 {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryStepLedgerV1 {
    pub schema: String,
    pub step_index: u64,
    pub update_order: UpdateOrderV1,
    pub frame_hash: String,
    pub before_state_hash: String,
    pub after_state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryEvidenceV1 {
    pub schema: String,
    pub initial_vertex_count: usize,
    pub steps: u64,
    pub maximum_activity: f64,
    pub final_state_hash: String,
    pub serialized_result_hash: String,
}

#[derive(Debug, Error)]
pub enum RegulatoryError {
    #[error("topology changed: expected {expected} patches, observed {observed}")]
    TopologyChangeUnsupported { expected: usize, observed: usize },
    #[error("invalid regulatory frame: {0}")]
    InvalidFrame(String),
    #[error("invalid regulatory state: {0}")]
    InvalidState(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryNetworkV1 {
    pub params: RegulatoryParamsV1,
    pub state: RegulatoryStateV1,
    pub ledger: Vec<RegulatoryStepLedgerV1>,
}

impl RegulatoryNetworkV1 {
    pub fn new(vertex_count: usize, provenance_seed: Option<u64>) -> Result<Self, RegulatoryError> {
        if vertex_count < 3 {
            return Err(RegulatoryError::InvalidState(
                "a regulatory ring requires at least three vertices".to_string(),
            ));
        }
        Ok(Self {
            params: RegulatoryParamsV1::default(),
            state: RegulatoryStateV1::new(vertex_count, provenance_seed),
            ledger: Vec::new(),
        })
    }

    pub fn step(&mut self, frame: &LocalMaterialFrameV1) -> Result<(), RegulatoryError> {
        self.step_with_order(frame, UpdateOrderV1::Forward)
    }

    pub fn step_with_order(
        &mut self,
        frame: &LocalMaterialFrameV1,
        order: UpdateOrderV1,
    ) -> Result<(), RegulatoryError> {
        if frame.topology_size != self.state.initial_vertex_count {
            return Err(RegulatoryError::TopologyChangeUnsupported {
                expected: self.state.initial_vertex_count,
                observed: frame.topology_size,
            });
        }
        frame.validate(&self.params)?;
        self.validate_state()?;

        let before = self.state_hash()?;
        let frame_hash = stable_json_hash(frame)?;
        let mut next = vec![0.0; self.state.activity.len()];
        let indices: Box<dyn Iterator<Item = usize>> = match order {
            UpdateOrderV1::Forward => Box::new(0..self.state.activity.len()),
            UpdateOrderV1::Reverse => Box::new((0..self.state.activity.len()).rev()),
        };
        for i in indices {
            let patch = &frame.patches[i];
            let neighbor_mean = 0.5
                * (self.state.activity[patch.previous_neighbor_index]
                    + self.state.activity[patch.next_neighbor_index]);
            let derivative = self.params.k_neighbor * (neighbor_mean - self.state.activity[i])
                + self.params.k_stimulus * patch.raw_stimulus * (1.0 - self.state.activity[i])
                - self.params.k_decay * self.state.activity[i];
            next[i] = (self.state.activity[i] + self.params.dt * derivative).clamp(0.0, 1.0);
        }
        self.state.activity = next;
        self.state.step_index = self.state.step_index.saturating_add(1);
        self.validate_state()?;
        let after = self.state_hash()?;
        self.ledger.push(RegulatoryStepLedgerV1 {
            schema: REGULATORY_LEDGER_SCHEMA_V1.to_string(),
            step_index: self.state.step_index,
            update_order: order,
            frame_hash,
            before_state_hash: before,
            after_state_hash: after,
        });
        Ok(())
    }

    pub fn state_hash(&self) -> Result<String, RegulatoryError> {
        stable_json_hash(&self.state)
    }

    pub fn evidence(&self) -> Result<RegulatoryEvidenceV1, RegulatoryError> {
        let maximum_activity = self.state.activity.iter().copied().fold(0.0_f64, f64::max);
        let final_state_hash = self.state_hash()?;
        let result = (&self.state, &self.ledger);
        let serialized_result_hash = stable_json_hash(&result)?;
        Ok(RegulatoryEvidenceV1 {
            schema: REGULATORY_EVIDENCE_SCHEMA_V1.to_string(),
            initial_vertex_count: self.state.initial_vertex_count,
            steps: self.state.step_index,
            maximum_activity,
            final_state_hash,
            serialized_result_hash,
        })
    }

    fn validate_state(&self) -> Result<(), RegulatoryError> {
        if self.state.schema != REGULATORY_STATE_SCHEMA_V1
            || self.state.activity.len() != self.state.initial_vertex_count
            || self
                .state
                .activity
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(RegulatoryError::InvalidState(
                "state schema, length, or bounded activity is invalid".to_string(),
            ));
        }
        Ok(())
    }
}

/// Immutable observation adapter for the DC-DEV-002/DC-DEV-003 material frame;
/// the regulator itself receives only a frame.
pub mod material_adapter {
    use super::{ContinuityMaterialFrameV1, LocalMaterialFrameV1, CLOSED_RING_TOPOLOGY_V1};
    use chemistry_core::material_mesh::MaterialMesh;
    use chemistry_core::mesh_mechanics::MechParams;

    pub fn observe_local_material_frame(
        mesh: &MaterialMesh,
        mechanics: &MechParams,
    ) -> LocalMaterialFrameV1 {
        let n = mesh.n();
        let edge_strain = |i: usize| {
            let rest = mesh.rest_length(i).max(1e-12);
            ((mesh.edge_length(i) - rest) / rest).max(0.0)
        };
        let patches = (0..n)
            .map(|i| {
                let previous = (i + n - 1) % n;
                let raw_stimulus: f64 =
                    (0.5_f64 * (edge_strain(previous) + edge_strain(i))).clamp(0.0, 1.0);
                super::LocalPatchInputV1 {
                    patch_index: i,
                    previous_neighbor_index: previous,
                    next_neighbor_index: (i + 1) % n,
                    raw_stimulus,
                    accepted_dt: mechanics.dt,
                }
            })
            .collect();
        LocalMaterialFrameV1 {
            schema: super::LOCAL_MATERIAL_FRAME_SCHEMA_V1.to_string(),
            topology_size: n,
            topology_identity: CLOSED_RING_TOPOLOGY_V1.to_string(),
            patches,
        }
    }

    /// Observe the same local tensile signal while retaining immutable vertex
    /// positions for DC-DEV-003 local topology correspondence.
    pub fn observe_continuity_material_frame(
        mesh: &MaterialMesh,
        mechanics: &MechParams,
    ) -> ContinuityMaterialFrameV1 {
        let n = mesh.n();
        let edge_strain = |i: usize| {
            let rest = mesh.rest_length(i).max(1e-12);
            ((mesh.edge_length(i) - rest) / rest).max(0.0)
        };
        let stimuli: Vec<f64> = (0..n)
            .map(|i| (0.5_f64 * (edge_strain((i + n - 1) % n) + edge_strain(i))).clamp(0.0, 1.0))
            .collect();
        ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            &stimuli,
            mechanics.dt,
        )
    }
}

/// Stable, dependency-free FNV-1a serialization hash for evidence identity.
pub fn stable_json_hash<T: Serialize>(value: &T) -> Result<String, RegulatoryError> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| RegulatoryError::Serialization(e.to_string()))?;
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
    use chemistry_core::mesh_mechanics::MechParams;

    fn frame(n: usize, stimulus_index: Option<usize>) -> LocalMaterialFrameV1 {
        let mut stimuli = vec![0.0; n];
        if let Some(i) = stimulus_index {
            stimuli[i] = 1.0;
        }
        LocalMaterialFrameV1::from_patch_stimuli(&stimuli)
    }

    fn mesh(n: usize) -> MaterialMesh {
        MaterialMesh::seed_regular(
            n,
            2.0,
            0.0,
            0.0,
            1.0,
            0.4,
            LumpedChem::default(),
            LumpedChem::default(),
            1.0,
        )
    }

    #[test]
    fn neutral_control_remains_exactly_zero_for_1000_steps() {
        let mut network = RegulatoryNetworkV1::new(8, Some(1)).unwrap();
        let neutral = frame(8, None);
        for _ in 0..1000 {
            network.step(&neutral).unwrap();
        }
        assert!(network.state.activity.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn locality_has_exactly_two_ring_neighbors_for_legal_sizes() {
        for n in [3, 6, 8, 17] {
            let f = frame(n, None);
            assert_eq!(f.patches.len(), n);
            for (i, patch) in f.patches.iter().enumerate() {
                assert_eq!(patch.previous_neighbor_index, (i + n - 1) % n);
                assert_eq!(patch.next_neighbor_index, (i + 1) % n);
            }
        }
    }

    #[test]
    fn synchronous_forward_and_reverse_orders_are_identical() {
        let f = frame(9, Some(4));
        let mut forward = RegulatoryNetworkV1::new(9, Some(7)).unwrap();
        let mut reverse = RegulatoryNetworkV1::new(9, Some(7)).unwrap();
        for _ in 0..30 {
            forward.step_with_order(&f, UpdateOrderV1::Forward).unwrap();
            reverse.step_with_order(&f, UpdateOrderV1::Reverse).unwrap();
        }
        assert_eq!(forward.state, reverse.state);
    }

    #[test]
    fn topology_change_fails_before_partial_update() {
        let mut network = RegulatoryNetworkV1::new(6, None).unwrap();
        let before = network.state.clone();
        let result = network.step(&frame(7, Some(0)));
        assert!(matches!(
            result,
            Err(RegulatoryError::TopologyChangeUnsupported {
                expected: 6,
                observed: 7
            })
        ));
        assert_eq!(network.state, before);
        assert!(network.ledger.is_empty());
    }

    #[test]
    fn mesh_adapter_is_nonsemantic_and_reads_positive_tensile_strain() {
        let mut stretched = mesh(8);
        let baseline = observe(&stretched);
        assert!(baseline
            .patches
            .iter()
            .all(|patch| patch.raw_stimulus == 0.0));
        stretched.vertices[0][0] += 0.5;
        let observed = observe(&stretched);
        assert!(observed
            .patches
            .iter()
            .any(|patch| patch.raw_stimulus > 0.0));
        assert!(observed
            .patches
            .iter()
            .all(|patch| (0.0..=1.0).contains(&patch.raw_stimulus)));
    }

    #[test]
    fn compression_does_not_create_positive_stimulus() {
        let mut compressed = mesh(8);
        compressed.vertices[0][0] -= 0.25;
        compressed.vertices[1][0] -= 0.25;
        let observed = observe(&compressed);
        assert!(observed
            .patches
            .iter()
            .all(|patch| patch.raw_stimulus == 0.0));
    }

    #[test]
    fn local_response_and_propagation_are_bounded_and_nonlocal() {
        let mut network = RegulatoryNetworkV1::new(11, None).unwrap();
        let pulse = frame(11, Some(0));
        network.step(&pulse).unwrap();
        assert!(network.state.activity[0] > 0.0);
        assert_eq!(network.state.activity[3], 0.0);
        let zero = frame(11, None);
        network.step(&zero).unwrap();
        assert!(network.state.activity[1] > 0.0);
        assert!(network.state.activity[10] > 0.0);
        assert_eq!(network.state.activity[3], 0.0);
        network.step(&zero).unwrap();
        assert!(network.state.activity[1] > network.state.activity[2]);
        assert!(network.state.activity[10] > network.state.activity[9]);
    }

    #[test]
    fn perturbation_on_then_off_has_no_stored_target_shape() {
        let mut network = RegulatoryNetworkV1::new(8, None).unwrap();
        let pulse = frame(8, Some(0));
        let zero = frame(8, None);
        network.step(&pulse).unwrap();
        let activated = network.state.activity[0];
        network.step(&zero).unwrap();
        assert!(activated > 0.0);
        assert!(network.state.activity[0] < activated);
    }

    #[test]
    fn uniform_pulse_persists_then_decays_monotonically() {
        let mut network = RegulatoryNetworkV1::new(8, None).unwrap();
        let pulse = LocalMaterialFrameV1::from_patch_stimuli(&vec![1.0; 8]);
        for _ in 0..20 {
            network.step(&pulse).unwrap();
        }
        let mut previous = network.state.activity[0];
        assert!(previous > 0.0);
        let zero = frame(8, None);
        for _ in 0..1000 {
            network.step(&zero).unwrap();
            let current = network.state.activity[0];
            assert!(current <= previous + 1e-12);
            previous = current;
        }
        assert!(previous < 0.01);
    }

    #[test]
    fn seed_is_provenance_only() {
        let f = frame(7, Some(2));
        let mut a = RegulatoryNetworkV1::new(7, Some(1)).unwrap();
        let mut b = RegulatoryNetworkV1::new(7, Some(99)).unwrap();
        for _ in 0..12 {
            a.step(&f).unwrap();
            b.step(&f).unwrap();
        }
        assert_eq!(a.state.activity, b.state.activity);
        assert_ne!(a.state.provenance_seed, b.state.provenance_seed);
    }

    #[test]
    fn serialized_evidence_replays_byte_identically() {
        let f = frame(6, Some(2));
        let run = || {
            let mut network = RegulatoryNetworkV1::new(6, Some(3)).unwrap();
            for _ in 0..20 {
                network.step(&f).unwrap();
            }
            serde_json::to_vec(&network.evidence().unwrap()).unwrap()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn observer_execution_does_not_change_material_mesh_state() {
        let mesh = mesh(8);
        let before = stable_json_hash(&mesh).unwrap();
        let observed = observe(&mesh);
        let mut network = RegulatoryNetworkV1::new(observed.topology_size, None).unwrap();
        network.step(&observed).unwrap();
        let after = stable_json_hash(&mesh).unwrap();
        assert_eq!(before, after);
    }

    fn observe(mesh: &MaterialMesh) -> LocalMaterialFrameV1 {
        material_adapter::observe_local_material_frame(mesh, &MechParams::default())
    }
}
