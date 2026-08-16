//! DC-DEV-004: one local, energy-coupled contractile actuator.
//!
//! Regulatory activity selects only local edge tension. Existing chemistry
//! reserve `R` funds that tension and is converted to existing waste `W`.
//! Existing chemistry-core mechanics remain the sole authority for vertex
//! movement; this module never writes vertex coordinates.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_mechanics::{
    mechanics_step, mechanics_step_with_edge_tensions,
    mechanics_step_with_edge_tensions_and_external_forces, mechanics_step_with_external_forces,
    MechParams,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONTRACTILITY_SCHEMA_V1: &str = "digital_cell_local_contractility_v1";

/// Frozen before the DC-DEV-004 assays. This is a force scale, not a target
/// shape or displacement command.
pub const FROZEN_MAX_ACTIVE_TENSION: f64 = 2.0;
/// Frozen reserve-R conversion: reserve units spent per force-length-time.
pub const FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME: f64 = 0.05;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractilityParamsV1 {
    pub schema: String,
    pub max_active_tension: f64,
    pub reserve_cost_per_force_length_time: f64,
}

impl Default for ContractilityParamsV1 {
    fn default() -> Self {
        Self {
            schema: CONTRACTILITY_SCHEMA_V1.to_string(),
            max_active_tension: FROZEN_MAX_ACTIVE_TENSION,
            reserve_cost_per_force_length_time: FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractilityStepLedgerV1 {
    pub schema: String,
    pub active_edge_indices: Vec<usize>,
    pub maximum_activity: f64,
    pub maximum_tension: f64,
    pub requested_resource: f64,
    pub resource_spent: f64,
    pub reserve_before: f64,
    pub reserve_after: f64,
    pub zero_resource_no_actuation: bool,
    pub mechanics_accepted: bool,
}

#[derive(Debug, Error, PartialEq)]
pub enum ContractilityError {
    #[error("contractility parameter schema or bound is invalid")]
    InvalidParameters,
    #[error("regulatory activity length {observed} does not match mesh size {expected}")]
    ActivityLength { expected: usize, observed: usize },
    #[error("regulatory activity is not finite and bounded")]
    InvalidActivity,
    #[error("existing material mechanics rejected the contractile step")]
    MechanicsRejected,
}

fn validate_params(params: &ContractilityParamsV1) -> Result<(), ContractilityError> {
    if params.schema != CONTRACTILITY_SCHEMA_V1
        || !params.max_active_tension.is_finite()
        || params.max_active_tension < 0.0
        || !params.reserve_cost_per_force_length_time.is_finite()
        || params.reserve_cost_per_force_length_time <= 0.0
    {
        return Err(ContractilityError::InvalidParameters);
    }
    Ok(())
}

/// Apply one local contractility step from the current distributed activity.
///
/// The only actuator decision is endpoint-local activity averaged onto each
/// existing edge. Available reserve is a scalar material budget; if it cannot
/// fund the requested tension, all available reserve funds a proportionally
/// reduced tension. With zero activity or zero reserve the exact legacy
/// mechanics path is used.
pub fn apply_local_contractility(
    mesh: &mut MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    params: &ContractilityParamsV1,
) -> Result<ContractilityStepLedgerV1, ContractilityError> {
    apply_local_contractility_with_external_forces(mesh, activity, mechanics, params, None)
}

/// Apply local contractility while allowing one bounded local force vector from
/// an external physical geometry.  `None` is the exact DC-DEV-004 path.
pub fn apply_local_contractility_with_external_forces(
    mesh: &mut MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    params: &ContractilityParamsV1,
    external_forces: Option<&[[f64; 2]]>,
) -> Result<ContractilityStepLedgerV1, ContractilityError> {
    validate_params(params)?;
    if activity.len() != mesh.n() {
        return Err(ContractilityError::ActivityLength {
            expected: mesh.n(),
            observed: activity.len(),
        });
    }
    if activity
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(ContractilityError::InvalidActivity);
    }

    let maximum_activity = activity.iter().copied().fold(0.0_f64, f64::max);
    let reserve_before = mesh.interior.r.max(0.0);
    if maximum_activity <= f64::EPSILON || reserve_before <= f64::EPSILON {
        let accepted = match external_forces {
            Some(forces) => mechanics_step_with_external_forces(mesh, mechanics, forces),
            None => mechanics_step(mesh, mechanics),
        };
        if !accepted {
            return Err(ContractilityError::MechanicsRejected);
        }
        return Ok(ContractilityStepLedgerV1 {
            schema: CONTRACTILITY_SCHEMA_V1.to_string(),
            active_edge_indices: Vec::new(),
            maximum_activity,
            maximum_tension: 0.0,
            requested_resource: 0.0,
            resource_spent: 0.0,
            reserve_before,
            reserve_after: mesh.interior.r.max(0.0),
            zero_resource_no_actuation: reserve_before <= f64::EPSILON,
            mechanics_accepted: true,
        });
    }

    let dt = mechanics.dt.max(0.0);
    let mut requested_tensions = vec![0.0; mesh.n()];
    let mut active_edge_indices = Vec::new();
    let mut requested_resource = 0.0;
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let edge_activity = 0.5 * (activity[i] + activity[(i + 1) % mesh.n()]);
        if edge_activity <= f64::EPSILON {
            continue;
        }
        let tension = params.max_active_tension * edge_activity;
        requested_tensions[i] = tension;
        active_edge_indices.push(i);
        requested_resource +=
            params.reserve_cost_per_force_length_time * tension * mesh.edge_length(i) * dt;
    }

    let available_resource = reserve_before * mesh.area().max(1e-12);
    let funding_scale = if requested_resource <= f64::EPSILON {
        0.0
    } else {
        (available_resource / requested_resource).min(1.0)
    };
    let mut tensions = requested_tensions;
    for tension in &mut tensions {
        *tension *= funding_scale;
    }
    let resource_spent = requested_resource * funding_scale;
    let active_edge_indices: Vec<usize> = active_edge_indices
        .into_iter()
        .filter(|index| tensions[*index] > f64::EPSILON)
        .collect();
    let maximum_tension = tensions.iter().copied().fold(0.0_f64, f64::max);

    let accepted = match (maximum_tension <= f64::EPSILON, external_forces) {
        (true, Some(forces)) => mechanics_step_with_external_forces(mesh, mechanics, forces),
        (true, None) => mechanics_step(mesh, mechanics),
        (false, Some(forces)) => mechanics_step_with_edge_tensions_and_external_forces(
            mesh, mechanics, &tensions, forces,
        ),
        (false, None) => mechanics_step_with_edge_tensions(mesh, mechanics, &tensions),
    };
    if !accepted {
        return Err(ContractilityError::MechanicsRejected);
    }

    if resource_spent > 0.0 {
        let area = mesh.area().max(1e-12);
        mesh.interior.r = (reserve_before - resource_spent / area).max(0.0);
        mesh.interior.w += resource_spent / area;
    }
    Ok(ContractilityStepLedgerV1 {
        schema: CONTRACTILITY_SCHEMA_V1.to_string(),
        active_edge_indices,
        maximum_activity,
        maximum_tension,
        requested_resource,
        resource_spent,
        reserve_before,
        reserve_after: mesh.interior.r.max(0.0),
        zero_resource_no_actuation: false,
        mechanics_accepted: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};

    fn mesh(reserve: f64) -> MaterialMesh {
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
                r: reserve,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        )
    }

    #[test]
    fn zero_reserve_disables_actuation_and_spending() {
        let mut body = mesh(0.0);
        let before = body.vertices.clone();
        let ledger = apply_local_contractility(
            &mut body,
            &[1.0; 8],
            &MechParams::default(),
            &ContractilityParamsV1::default(),
        )
        .unwrap();
        assert!(ledger.zero_resource_no_actuation);
        assert_eq!(ledger.resource_spent, 0.0);
        assert!(ledger.active_edge_indices.is_empty());
        assert_ne!(body.vertices, before); // legacy mechanics still advances the body
    }

    #[test]
    fn zero_activity_uses_legacy_path_exactly() {
        let mut legacy = mesh(0.6);
        let mut actuator = legacy.clone();
        mechanics_step(&mut legacy, &MechParams::default());
        let ledger = apply_local_contractility(
            &mut actuator,
            &[0.0; 8],
            &MechParams::default(),
            &ContractilityParamsV1::default(),
        )
        .unwrap();
        assert_eq!(legacy.vertices, actuator.vertices);
        assert_eq!(legacy.interior.r, actuator.interior.r);
        assert_eq!(ledger.resource_spent, 0.0);
    }

    #[test]
    fn funded_activity_spends_existing_reserve() {
        let mut body = mesh(0.6);
        let ledger = apply_local_contractility(
            &mut body,
            &[1.0; 8],
            &MechParams::default(),
            &ContractilityParamsV1::default(),
        )
        .unwrap();
        assert!(!ledger.active_edge_indices.is_empty());
        assert!(ledger.resource_spent > 0.0);
        assert!(ledger.reserve_after < ledger.reserve_before);
        assert!(body.interior.w > 0.0);
    }
}
