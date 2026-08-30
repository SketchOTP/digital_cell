//! DC-DEV-021 ENTRY-003: opt-in intrinsic local exploration.
//!
//! This module deliberately does not alter `RegulatoryNetworkV1` or
//! `PlasticityStateV1`.  It composes a separate, versioned activity state with
//! the existing slow local adaptation trace and the opt-in A-funded actuator.
//! No world, resource, observer, or target input is accepted.

use crate::{
    apply_local_activated_energy_contractility_with_stick_slip,
    ActivatedEnergyStickSlipStepLedgerV1, ContractilityParamsV1, PlasticityStateV1, StickSlipError,
    StickSlipTractionParamsV1, FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
    FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME, FROZEN_DT, FROZEN_K_DECAY, FROZEN_K_NEIGHBOR,
    FROZEN_K_STIMULUS, PLASTICITY_SCHEMA_V1,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_mechanics::MechParams;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1: &str =
    "digital_cell_intrinsic_exploration_regulator_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntrinsicExplorationDynamicsModeV1 {
    FullSelfExcitation,
    SeedOnlyControl,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntrinsicExplorationProposalV1 {
    pub schema: String,
    pub expected_step_index: u64,
    pub activity_after: Vec<f64>,
    pub adaptation_after: Vec<f64>,
    pub effective_activity: Vec<f64>,
    pub dominant_patch: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntrinsicExplorationStateV1 {
    pub schema: String,
    pub initial_vertex_count: usize,
    pub step_index: u64,
    pub provenance_seed: Option<u64>,
    pub activity: Vec<f64>,
    pub adaptation: PlasticityStateV1,
}

impl IntrinsicExplorationStateV1 {
    /// Initialise the only symmetry break.  Its amplitude is exactly the
    /// existing regulatory stimulus increment; the seed chooses a ring-local
    /// patch and never supplies a direction or a recurring impulse.
    pub fn new(
        vertex_count: usize,
        provenance_seed: Option<u64>,
    ) -> Result<Self, IntrinsicExplorationError> {
        if vertex_count < 3 {
            return Err(IntrinsicExplorationError::InvalidState);
        }
        let mut activity = vec![0.0; vertex_count];
        activity[seed_patch(vertex_count, provenance_seed)] =
            (FROZEN_K_STIMULUS * FROZEN_DT).min(1.0);
        Ok(Self {
            schema: INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1.to_string(),
            initial_vertex_count: vertex_count,
            step_index: 0,
            provenance_seed,
            activity,
            adaptation: PlasticityStateV1::new(vertex_count),
        })
    }

    pub fn seeded_patch_index(&self) -> usize {
        seed_patch(self.initial_vertex_count, self.provenance_seed)
    }

    fn validate(&self, vertex_count: usize, dt: f64) -> Result<(), IntrinsicExplorationError> {
        if self.schema != INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1
            || self.initial_vertex_count != vertex_count
            || self.activity.len() != vertex_count
            || self.adaptation.schema != PLASTICITY_SCHEMA_V1
            || self.adaptation.adaptation.len() != vertex_count
            || self
                .activity
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || self
                .adaptation
                .adaptation
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || !dt.is_finite()
            || (dt - FROZEN_DT).abs() > 1e-12
        {
            return Err(IntrinsicExplorationError::InvalidState);
        }
        Ok(())
    }
}

fn seed_patch(vertex_count: usize, provenance_seed: Option<u64>) -> usize {
    provenance_seed.unwrap_or(0) as usize % vertex_count
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntrinsicExplorationStepLedgerV1 {
    pub schema: String,
    pub step_index: u64,
    pub activity_before: Vec<f64>,
    pub activity_after: Vec<f64>,
    pub adaptation_before: Vec<f64>,
    pub adaptation_after: Vec<f64>,
    pub effective_activity: Vec<f64>,
    pub dominant_patch: usize,
    pub actuator: ActivatedEnergyStickSlipStepLedgerV1,
}

#[derive(Debug, Error, PartialEq)]
pub enum IntrinsicExplorationError {
    #[error("intrinsic exploration state is invalid")]
    InvalidState,
    #[error("intrinsic exploration requires a topology-preserving mesh")]
    TopologyChanged,
    #[error("A-funded stick-slip actuation failed: {0}")]
    Actuation(#[from] StickSlipError),
}

/// Compute one accepted-step candidate without mutating organism state.  The
/// caller must commit it only after its existing mechanics authority accepts.
pub fn propose_intrinsic_exploration_step(
    state: &IntrinsicExplorationStateV1,
    vertex_count: usize,
    dt: f64,
    mode: IntrinsicExplorationDynamicsModeV1,
) -> Result<IntrinsicExplorationProposalV1, IntrinsicExplorationError> {
    state.validate(vertex_count, dt)?;
    let activity_before = &state.activity;
    let adaptation_before = &state.adaptation.adaptation;
    let mut activity_after = vec![0.0; vertex_count];
    for i in 0..vertex_count {
        let neighbor_mean = 0.5
            * (activity_before[(i + vertex_count - 1) % vertex_count]
                + activity_before[(i + 1) % vertex_count]);
        let current = activity_before[i];
        let self_excitation = match mode {
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation => {
                FROZEN_K_STIMULUS * current * (1.0 - current) * (1.0 - adaptation_before[i])
            }
            IntrinsicExplorationDynamicsModeV1::SeedOnlyControl => 0.0,
        };
        let derivative = FROZEN_K_NEIGHBOR * (neighbor_mean - current) + self_excitation
            - FROZEN_K_DECAY * current;
        activity_after[i] = (current + dt * derivative).clamp(0.0, 1.0);
    }
    let effective_activity: Vec<f64> = activity_after
        .iter()
        .zip(adaptation_before)
        .map(|(activity, adaptation)| activity * (1.0 - adaptation))
        .collect();
    let adaptation_after: Vec<f64> = activity_after
        .iter()
        .zip(adaptation_before)
        .map(|(activity, before)| {
            let load = FROZEN_ADAPTATION_LOAD_RATE_PER_TIME * activity * (1.0 - before);
            let recovery = FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME * (1.0 - activity) * before;
            (before + dt * (load - recovery)).clamp(0.0, 1.0)
        })
        .collect();
    let dominant_patch = activity_after
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap())
        .map(|(index, _)| index)
        .unwrap_or(0);
    Ok(IntrinsicExplorationProposalV1 {
        schema: INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1.to_string(),
        expected_step_index: state.step_index,
        activity_after,
        adaptation_after,
        effective_activity,
        dominant_patch,
    })
}

pub fn commit_intrinsic_exploration_step(
    state: &mut IntrinsicExplorationStateV1,
    proposal: IntrinsicExplorationProposalV1,
) -> Result<(), IntrinsicExplorationError> {
    state.validate(state.initial_vertex_count, FROZEN_DT)?;
    if proposal.schema != INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1
        || proposal.expected_step_index != state.step_index
        || proposal.activity_after.len() != state.initial_vertex_count
        || proposal.adaptation_after.len() != state.initial_vertex_count
        || proposal.effective_activity.len() != state.initial_vertex_count
    {
        return Err(IntrinsicExplorationError::InvalidState);
    }
    state.activity = proposal.activity_after;
    state.adaptation.adaptation = proposal.adaptation_after;
    state.step_index = state.step_index.saturating_add(1);
    Ok(())
}

/// Advance the explicit intrinsic activity state and submit its local,
/// adaptation-limited output to the existing A-funded stick-slip path.  State
/// is committed only if the physical step is accepted by that path.
pub fn apply_intrinsic_exploration_with_stick_slip(
    mesh: &mut MaterialMesh,
    state: &mut IntrinsicExplorationStateV1,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Result<IntrinsicExplorationStepLedgerV1, IntrinsicExplorationError> {
    state.validate(mesh.n(), mechanics.dt)?;
    if mesh.n() != state.initial_vertex_count {
        return Err(IntrinsicExplorationError::TopologyChanged);
    }

    let activity_before = state.activity.clone();
    let adaptation_before = state.adaptation.adaptation.clone();
    let proposal = propose_intrinsic_exploration_step(
        state,
        mesh.n(),
        mechanics.dt,
        IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
    )?;
    let actuator = apply_local_activated_energy_contractility_with_stick_slip(
        mesh,
        &proposal.effective_activity,
        mechanics,
        contractility,
        traction,
    )?;

    let activity_after = proposal.activity_after.clone();
    let adaptation_after = proposal.adaptation_after.clone();
    let effective_activity = proposal.effective_activity.clone();
    let dominant_patch = proposal.dominant_patch;
    commit_intrinsic_exploration_step(state, proposal)?;
    Ok(IntrinsicExplorationStepLedgerV1 {
        schema: INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1.to_string(),
        step_index: state.step_index,
        activity_before,
        activity_after,
        adaptation_before,
        adaptation_after,
        effective_activity,
        dominant_patch,
        actuator,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{
        LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S,
    };

    fn mesh() -> MaterialMesh {
        let mut mesh = MaterialMesh::seed_regular(
            8,
            4.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                a: 0.5,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        );
        mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
        mesh
    }

    #[test]
    fn seed_is_one_time_ring_local_and_uses_only_existing_increment() {
        let one = IntrinsicExplorationStateV1::new(8, Some(1)).unwrap();
        let two = IntrinsicExplorationStateV1::new(8, Some(2)).unwrap();
        assert_eq!(one.seeded_patch_index(), 1);
        assert_eq!(two.seeded_patch_index(), 2);
        assert_eq!(one.activity.iter().filter(|value| **value > 0.0).count(), 1);
        assert_eq!(one.activity[1], FROZEN_K_STIMULUS * FROZEN_DT);
    }

    #[test]
    fn zero_a_keeps_intrinsic_dynamics_but_cannot_fund_motion() {
        let mut body = mesh();
        body.interior.a = 0.0;
        let mut state = IntrinsicExplorationStateV1::new(body.n(), Some(3)).unwrap();
        let before = body.clone();
        let ledger = apply_intrinsic_exploration_with_stick_slip(
            &mut body,
            &mut state,
            &MechParams::default(),
            &ContractilityParamsV1::default(),
            &StickSlipTractionParamsV1::default(),
        )
        .unwrap();
        assert!(ledger.actuator.contractility.unwrap().resource_spent == 0.0);
        assert!(state.activity.iter().any(|value| *value > 0.0));
        assert_eq!(before.interior.a, body.interior.a);
    }

    #[test]
    fn serialized_state_continues_without_reseeding() {
        let state = IntrinsicExplorationStateV1::new(8, Some(6)).unwrap();
        let restored: IntrinsicExplorationStateV1 =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert_eq!(state, restored);
        assert_eq!(restored.seeded_patch_index(), 6);
    }
}
