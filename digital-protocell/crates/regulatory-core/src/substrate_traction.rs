//! DC-DEV-010: one passive direction-dependent substrate reaction law.
//!
//! The substrate is a local dissipative resistance, not an actuator. It reads
//! only the pre-step attempted velocity and a fixed substrate axis. A forward
//! and reverse longitudinal resistance differ, while transverse resistance is
//! fixed. The resulting reaction is always opposite the attempted motion and
//! is bounded before it reaches the existing mechanics force hook.

use chemistry_core::mesh_mechanics::MechParams;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SUBSTRATE_TRACTION_SCHEMA_V1: &str = "digital_cell_passive_directional_substrate_v1";
pub const FROZEN_FORWARD_RESISTANCE_RATIO: f64 = 0.25;
pub const FROZEN_REVERSE_RESISTANCE_RATIO: f64 = 0.75;
pub const FROZEN_TRANSVERSE_RESISTANCE_RATIO: f64 = 0.50;
pub const FROZEN_MAX_REACTION_FORCE: f64 = 0.45;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubstrateTractionParamsV1 {
    pub schema: String,
    pub axis: [f64; 2],
    pub forward_resistance_ratio: f64,
    pub reverse_resistance_ratio: f64,
    pub transverse_resistance_ratio: f64,
    pub max_reaction_force: f64,
}

impl Default for SubstrateTractionParamsV1 {
    fn default() -> Self {
        Self {
            schema: SUBSTRATE_TRACTION_SCHEMA_V1.to_string(),
            axis: [1.0, 0.0],
            forward_resistance_ratio: FROZEN_FORWARD_RESISTANCE_RATIO,
            reverse_resistance_ratio: FROZEN_REVERSE_RESISTANCE_RATIO,
            transverse_resistance_ratio: FROZEN_TRANSVERSE_RESISTANCE_RATIO,
            max_reaction_force: FROZEN_MAX_REACTION_FORCE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateMode {
    Directional,
    IsotropicControl,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubstrateReactionV1 {
    pub force: [f64; 2],
    pub attempted_velocity: [f64; 2],
    pub accepted_velocity: [f64; 2],
    pub work: f64,
    pub resistance_ratio: f64,
}

#[derive(Debug, Error, PartialEq)]
pub enum SubstrateTractionError {
    #[error("substrate parameter schema, axis, ratio, or force bound is invalid")]
    InvalidParameters,
    #[error("internal force vector is not finite")]
    InvalidForce,
    #[error("internal force length {observed} does not match mesh size {expected}")]
    ForceLength { expected: usize, observed: usize },
}

fn validate(
    params: &SubstrateTractionParamsV1,
    mechanics: &MechParams,
) -> Result<(), SubstrateTractionError> {
    let axis_norm = params.axis[0].hypot(params.axis[1]);
    if params.schema != SUBSTRATE_TRACTION_SCHEMA_V1
        || !axis_norm.is_finite()
        || (axis_norm - 1.0).abs() > 1e-12
        || !params.forward_resistance_ratio.is_finite()
        || !(0.0..=1.0).contains(&params.forward_resistance_ratio)
        || !params.reverse_resistance_ratio.is_finite()
        || !(0.0..=1.0).contains(&params.reverse_resistance_ratio)
        || !params.transverse_resistance_ratio.is_finite()
        || !(0.0..=1.0).contains(&params.transverse_resistance_ratio)
        || !params.max_reaction_force.is_finite()
        || params.max_reaction_force <= 0.0
        || !mechanics.gamma.is_finite()
        || mechanics.gamma <= 0.0
        || !mechanics.dt.is_finite()
        || mechanics.dt < 0.0
    {
        return Err(SubstrateTractionError::InvalidParameters);
    }
    Ok(())
}

/// Return the passive local reaction for one attempted velocity.
pub fn reaction_for_velocity(
    attempted_velocity: [f64; 2],
    mechanics: &MechParams,
    params: &SubstrateTractionParamsV1,
    mode: SubstrateMode,
) -> Result<SubstrateReactionV1, SubstrateTractionError> {
    validate(params, mechanics)?;
    if !attempted_velocity[0].is_finite() || !attempted_velocity[1].is_finite() {
        return Err(SubstrateTractionError::InvalidForce);
    }
    let axis = params.axis;
    let longitudinal = attempted_velocity[0] * axis[0] + attempted_velocity[1] * axis[1];
    let longitudinal_ratio = match mode {
        SubstrateMode::Directional if longitudinal > 0.0 => params.forward_resistance_ratio,
        SubstrateMode::Directional => params.reverse_resistance_ratio,
        SubstrateMode::IsotropicControl => params.transverse_resistance_ratio,
    };
    let parallel = [longitudinal * axis[0], longitudinal * axis[1]];
    let perpendicular = [
        attempted_velocity[0] - parallel[0],
        attempted_velocity[1] - parallel[1],
    ];
    let mut force = [
        -mechanics.gamma
            * (longitudinal_ratio * parallel[0]
                + params.transverse_resistance_ratio * perpendicular[0]),
        -mechanics.gamma
            * (longitudinal_ratio * parallel[1]
                + params.transverse_resistance_ratio * perpendicular[1]),
    ];
    let magnitude = force[0].hypot(force[1]);
    if magnitude > params.max_reaction_force {
        let scale = params.max_reaction_force / magnitude;
        force[0] *= scale;
        force[1] *= scale;
    }
    let accepted_velocity = [
        attempted_velocity[0] + force[0] / mechanics.gamma,
        attempted_velocity[1] + force[1] / mechanics.gamma,
    ];
    let work = (force[0] * accepted_velocity[0] + force[1] * accepted_velocity[1]) * mechanics.dt;
    Ok(SubstrateReactionV1 {
        force,
        attempted_velocity,
        accepted_velocity,
        work,
        resistance_ratio: longitudinal_ratio,
    })
}

/// Compute one local reaction per vertex from the pre-step internal force.
pub fn reactions_for_internal_forces(
    internal_forces: &[[f64; 2]],
    mechanics: &MechParams,
    params: &SubstrateTractionParamsV1,
    mode: SubstrateMode,
) -> Result<Vec<SubstrateReactionV1>, SubstrateTractionError> {
    validate(params, mechanics)?;
    internal_forces
        .iter()
        .copied()
        .map(|force| {
            if !force[0].is_finite() || !force[1].is_finite() {
                return Err(SubstrateTractionError::InvalidForce);
            }
            reaction_for_velocity(
                [force[0] / mechanics.gamma, force[1] / mechanics.gamma],
                mechanics,
                params,
                mode,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mechanics() -> MechParams {
        MechParams::default()
    }

    #[test]
    fn zero_motion_has_zero_reaction_and_work() {
        let result = reaction_for_velocity(
            [0.0, 0.0],
            &mechanics(),
            &SubstrateTractionParamsV1::default(),
            SubstrateMode::Directional,
        )
        .unwrap();
        assert_eq!(result.force, [0.0, 0.0]);
        assert_eq!(result.work, 0.0);
    }

    #[test]
    fn opposite_sliding_directions_have_different_reactions() {
        let params = SubstrateTractionParamsV1::default();
        let positive = reaction_for_velocity(
            [1.0, 0.0],
            &mechanics(),
            &params,
            SubstrateMode::Directional,
        )
        .unwrap();
        let negative = reaction_for_velocity(
            [-1.0, 0.0],
            &mechanics(),
            &params,
            SubstrateMode::Directional,
        )
        .unwrap();
        assert!(positive.force[0] < 0.0);
        assert!(negative.force[0] > 0.0);
        assert_ne!(positive.force[0].abs(), negative.force[0].abs());
    }

    #[test]
    fn reaction_is_dissipative_and_bounded() {
        let params = SubstrateTractionParamsV1::default();
        for velocity in [[1.0, 0.0], [-1.0, 0.0], [0.3, 0.8], [-0.4, -0.2]] {
            let result =
                reaction_for_velocity(velocity, &mechanics(), &params, SubstrateMode::Directional)
                    .unwrap();
            assert!(result.work <= 1e-12);
            assert!(result.force[0].hypot(result.force[1]) <= params.max_reaction_force + 1e-12);
        }
    }

    #[test]
    fn deterministic_replay_is_exact() {
        let params = SubstrateTractionParamsV1::default();
        let first = reactions_for_internal_forces(
            &[[0.2, 0.1], [-0.3, 0.4]],
            &mechanics(),
            &params,
            SubstrateMode::Directional,
        )
        .unwrap();
        let second = reactions_for_internal_forces(
            &[[0.2, 0.1], [-0.3, 0.4]],
            &mechanics(),
            &params,
            SubstrateMode::Directional,
        )
        .unwrap();
        assert_eq!(first, second);
    }
}
