//! DC-DEV-005: one local, slow experience-dependent plasticity trace.
//!
//! Each regulatory patch owns one bounded adaptation value.  The trace is
//! updated only from that patch's activity and the accepted mechanics time
//! increment.  It modulates the already-qualified DC-DEV-004 activity-to-edge
//! tension rule; it does not introduce a sensor, actuator, command, reward, or
//! global state.

use crate::{
    apply_local_contractility, ContractilityError, ContractilityParamsV1,
    ContractilityStepLedgerV1, TopologyEventV1, TopologyMappingV1,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_mechanics::MechParams;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PLASTICITY_SCHEMA_V1: &str = "digital_cell_local_plasticity_v1";

/// Fixed before the first DC-DEV-005 qualification run.
///
/// The existing regulatory decay rate is 0.5 per simulation time unit, giving
/// a fast activity timescale of 2.0 time units.  These rates give adaptation
/// load and recovery timescales of 10.0 and 20.0 time units respectively.
pub const FROZEN_ADAPTATION_LOAD_RATE_PER_TIME: f64 = 0.1;
pub const FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME: f64 = 0.05;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlasticityParamsV1 {
    pub schema: String,
    pub load_rate_per_time: f64,
    pub recovery_rate_per_time: f64,
}

impl Default for PlasticityParamsV1 {
    fn default() -> Self {
        Self {
            schema: PLASTICITY_SCHEMA_V1.to_string(),
            load_rate_per_time: FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
            recovery_rate_per_time: FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlasticityStateV1 {
    pub schema: String,
    pub enabled: bool,
    pub adaptation: Vec<f64>,
}

impl PlasticityStateV1 {
    pub fn new(vertex_count: usize) -> Self {
        Self {
            schema: PLASTICITY_SCHEMA_V1.to_string(),
            enabled: true,
            adaptation: vec![0.0; vertex_count],
        }
    }

    pub fn disabled(vertex_count: usize) -> Self {
        Self {
            schema: PLASTICITY_SCHEMA_V1.to_string(),
            enabled: false,
            adaptation: vec![0.0; vertex_count],
        }
    }

    /// Transfer the one local trace through an ordinary split or merge using
    /// the already-qualified DC-DEV-003 local correspondence. Fission and
    /// unknown topology are intentionally unsupported and fail closed.
    pub fn remap(&mut self, mapping: &TopologyMappingV1) -> Result<(), PlasticityError> {
        if matches!(
            mapping.event,
            TopologyEventV1::Initial | TopologyEventV1::Fission | TopologyEventV1::Unknown
        ) {
            return Err(PlasticityError::UnsupportedTopology(mapping.event));
        }
        if mapping.old_topology_size != self.adaptation.len()
            || mapping.new_topology_size != mapping.new_to_old.len()
            || mapping
                .new_to_old
                .iter()
                .any(|old_index| *old_index >= self.adaptation.len())
        {
            return Err(PlasticityError::InvalidMapping);
        }
        self.adaptation = mapping
            .new_to_old
            .iter()
            .map(|old_index| self.adaptation[*old_index].clamp(0.0, 1.0))
            .collect();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlasticityStepLedgerV1 {
    pub schema: String,
    pub enabled: bool,
    pub accepted_dt: f64,
    pub adaptation_before: Vec<f64>,
    pub adaptation_after: Vec<f64>,
    pub effective_activity: Vec<f64>,
    pub maximum_adaptation: f64,
    pub contractility: ContractilityStepLedgerV1,
}

#[derive(Debug, Error, PartialEq)]
pub enum PlasticityError {
    #[error("plasticity parameter schema or bound is invalid")]
    InvalidParameters,
    #[error("plasticity state schema or length is invalid")]
    InvalidState,
    #[error("regulatory activity length {observed} does not match mesh size {expected}")]
    ActivityLength { expected: usize, observed: usize },
    #[error("regulatory activity is not finite and bounded")]
    InvalidActivity,
    #[error("accepted simulation time is not finite and non-negative")]
    InvalidAcceptedTime,
    #[error("local plasticity topology mapping is invalid")]
    InvalidMapping,
    #[error("local plasticity does not support topology event {0:?}")]
    UnsupportedTopology(TopologyEventV1),
    #[error("contractility failed: {0}")]
    Contractility(#[from] ContractilityError),
}

fn validate_params(params: &PlasticityParamsV1) -> Result<(), PlasticityError> {
    if params.schema != PLASTICITY_SCHEMA_V1
        || !params.load_rate_per_time.is_finite()
        || params.load_rate_per_time <= 0.0
        || !params.recovery_rate_per_time.is_finite()
        || params.recovery_rate_per_time <= 0.0
    {
        return Err(PlasticityError::InvalidParameters);
    }
    Ok(())
}

fn validate_state(state: &PlasticityStateV1, vertex_count: usize) -> Result<(), PlasticityError> {
    if state.schema != PLASTICITY_SCHEMA_V1
        || state.adaptation.len() != vertex_count
        || state
            .adaptation
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(PlasticityError::InvalidState);
    }
    Ok(())
}

/// Apply one accepted local plasticity/mechanics step.
///
/// The current response uses the adaptation value from the start of the
/// accepted step.  The trace is committed only after the existing mechanics
/// and contractility path accepts the step.  Therefore an all-zero trace is
/// exactly the DC-DEV-004 response, while an unsuccessful mechanics step does
/// not advance simulated experience.
pub fn apply_local_plasticity(
    mesh: &mut MaterialMesh,
    activity: &[f64],
    state: &mut PlasticityStateV1,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    params: &PlasticityParamsV1,
) -> Result<PlasticityStepLedgerV1, PlasticityError> {
    validate_params(params)?;
    if activity.len() != mesh.n() {
        return Err(PlasticityError::ActivityLength {
            expected: mesh.n(),
            observed: activity.len(),
        });
    }
    if activity
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(PlasticityError::InvalidActivity);
    }
    validate_state(state, mesh.n())?;
    if !mechanics.dt.is_finite() || mechanics.dt < 0.0 {
        return Err(PlasticityError::InvalidAcceptedTime);
    }

    let adaptation_before = state.adaptation.clone();
    let effective_activity: Vec<f64> = if state.enabled {
        activity
            .iter()
            .zip(&adaptation_before)
            .map(|(current, adaptation)| current * (1.0 - adaptation))
            .collect()
    } else {
        activity.to_vec()
    };
    let contractility_ledger =
        apply_local_contractility(mesh, &effective_activity, mechanics, contractility)?;

    let mut adaptation_after = adaptation_before.clone();
    if state.enabled {
        for ((next, current), before) in adaptation_after
            .iter_mut()
            .zip(activity)
            .zip(&adaptation_before)
        {
            let load = params.load_rate_per_time * current * (1.0 - before);
            let recovery = params.recovery_rate_per_time * (1.0 - current) * before;
            *next = (*before + mechanics.dt * (load - recovery)).clamp(0.0, 1.0);
        }
        state.adaptation = adaptation_after.clone();
    }

    Ok(PlasticityStepLedgerV1 {
        schema: PLASTICITY_SCHEMA_V1.to_string(),
        enabled: state.enabled,
        accepted_dt: mechanics.dt,
        maximum_adaptation: adaptation_after.iter().copied().fold(0.0, f64::max),
        adaptation_before,
        adaptation_after,
        effective_activity,
        contractility: contractility_ledger,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};

    fn mesh() -> MaterialMesh {
        MaterialMesh::seed_regular(
            8,
            4.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                c: 0.8,
                a: 0.5,
                n: 0.4,
                f: 0.4,
                r: 5.0,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        )
    }

    #[test]
    fn zero_adaptation_preserves_dcdev004_response_exactly() {
        let activity = [1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mechanics = MechParams::default();
        let contractility = ContractilityParamsV1::default();
        let plasticity = PlasticityParamsV1::default();
        let mut baseline = mesh();
        let mut adapted = baseline.clone();
        let expected =
            apply_local_contractility(&mut baseline, &activity, &mechanics, &contractility)
                .unwrap();
        let mut state = PlasticityStateV1::new(adapted.n());
        let observed = apply_local_plasticity(
            &mut adapted,
            &activity,
            &mut state,
            &mechanics,
            &contractility,
            &plasticity,
        )
        .unwrap();
        assert_eq!(baseline.vertices, adapted.vertices);
        assert_eq!(
            crate::stable_json_hash(&baseline.interior).unwrap(),
            crate::stable_json_hash(&adapted.interior).unwrap()
        );
        assert_eq!(expected, observed.contractility);
    }

    #[test]
    fn disabled_plasticity_is_exact_dcdev004_control() {
        let activity = [0.8; 8];
        let mechanics = MechParams::default();
        let contractility = ContractilityParamsV1::default();
        let plasticity = PlasticityParamsV1::default();
        let mut baseline = mesh();
        let mut disabled = baseline.clone();
        let expected =
            apply_local_contractility(&mut baseline, &activity, &mechanics, &contractility)
                .unwrap();
        let mut state = PlasticityStateV1::disabled(disabled.n());
        let observed = apply_local_plasticity(
            &mut disabled,
            &activity,
            &mut state,
            &mechanics,
            &contractility,
            &plasticity,
        )
        .unwrap();
        assert_eq!(baseline.vertices, disabled.vertices);
        assert_eq!(
            crate::stable_json_hash(&baseline.interior).unwrap(),
            crate::stable_json_hash(&disabled.interior).unwrap()
        );
        assert_eq!(expected, observed.contractility);
        assert!(state.adaptation.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn local_load_is_bounded_and_zero_activity_recovers() {
        let mechanics = MechParams::default();
        let contractility = ContractilityParamsV1::default();
        let params = PlasticityParamsV1::default();
        let mut body = mesh();
        let mut state = PlasticityStateV1::new(body.n());
        let local = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        for _ in 0..100 {
            apply_local_plasticity(
                &mut body,
                &local,
                &mut state,
                &mechanics,
                &contractility,
                &params,
            )
            .unwrap();
        }
        let loaded = state.adaptation[0];
        assert!(loaded > 0.0 && loaded < 1.0);
        assert!(state.adaptation[4] == 0.0);
        for _ in 0..100 {
            apply_local_plasticity(
                &mut body,
                &[0.0; 8],
                &mut state,
                &mechanics,
                &contractility,
                &params,
            )
            .unwrap();
        }
        assert!(state.adaptation[0] < loaded);
        assert!(state
            .adaptation
            .iter()
            .all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn fission_mapping_fails_closed() {
        let mut state = PlasticityStateV1::new(3);
        let mapping = TopologyMappingV1 {
            schema: crate::continuity::TOPOLOGY_MAPPING_SCHEMA_V1.to_string(),
            old_topology_size: 3,
            new_topology_size: 3,
            event: TopologyEventV1::Fission,
            new_to_old: vec![0, 1, 2],
            maximum_mapping_distance: 0.0,
            mapping_rule: "unsupported".to_string(),
        };
        assert_eq!(
            state.remap(&mapping),
            Err(PlasticityError::UnsupportedTopology(
                TopologyEventV1::Fission
            ))
        );
    }
}
