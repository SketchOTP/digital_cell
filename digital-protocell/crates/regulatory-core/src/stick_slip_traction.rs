//! DC-DEV-011: one passive, local, isotropic stick-slip substrate.
//!
//! The substrate reads only the local force/velocity presented by the
//! existing mechanics path.  It has no world axis, vertex identity, centroid,
//! stimulus, regulatory, or semantic organism input.  This module supplies
//! reactions only; the chemistry-core mechanics integrator remains the sole
//! authority over vertex coordinates.

use crate::{
    apply_local_contractility_with_external_forces, ContractilityError, ContractilityParamsV1,
    ContractilityStepLedgerV1,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_mechanics::{
    mechanics_step, mechanics_step_with_external_forces, MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const STICK_SLIP_TRACTION_SCHEMA_V1: &str = "digital_cell_passive_isotropic_stick_slip_v1";

/// Frozen before the first DC-DEV-011 qualification run.
///
/// The static limit is anchored to the settled DC-DEV-010-R2 force scale
/// (approximately 0.52 standalone late-window component norm) and remains
/// below the existing external-force interface bound.  The kinetic magnitude
/// is smaller than static traction and is large enough to remain observable
/// against the existing active contractility scale without becoming a new
/// propulsion force.
pub const FROZEN_STATIC_TRACTION_LIMIT: f64 = 0.45;
pub const FROZEN_KINETIC_TRACTION: f64 = 0.20;
pub const FROZEN_ZERO_MOTION_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StickSlipTractionParamsV1 {
    pub schema: String,
    pub static_traction_limit: f64,
    pub kinetic_traction: f64,
    pub zero_motion_tolerance: f64,
}

impl Default for StickSlipTractionParamsV1 {
    fn default() -> Self {
        Self {
            schema: STICK_SLIP_TRACTION_SCHEMA_V1.to_string(),
            static_traction_limit: FROZEN_STATIC_TRACTION_LIMIT,
            kinetic_traction: FROZEN_KINETIC_TRACTION,
            zero_motion_tolerance: FROZEN_ZERO_MOTION_TOLERANCE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactRegimeV1 {
    Stick,
    Slip,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactLedgerV1 {
    pub regime: ContactRegimeV1,
    pub required_force: f64,
    pub attempted_velocity: [f64; 2],
    pub reaction: [f64; 2],
    pub accepted_velocity: [f64; 2],
    pub work: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StickSlipStepLedgerV1 {
    pub schema: String,
    pub contacts: Vec<ContactLedgerV1>,
    pub maximum_stick_reaction: f64,
    pub maximum_slip_reaction: f64,
    pub stuck_contacts: usize,
    pub slipping_contacts: usize,
    pub substrate_work: f64,
    pub maximum_attempted_velocity: f64,
    pub maximum_accepted_velocity: f64,
    pub contractility: Option<ContractilityStepLedgerV1>,
}

#[derive(Debug, Error, PartialEq)]
pub enum StickSlipError {
    #[error("stick-slip parameter schema or bound is invalid")]
    InvalidParameters,
    #[error("local contact force or velocity is invalid")]
    InvalidContact,
    #[error("mechanics parameters are invalid for an accepted stick-slip step")]
    InvalidMechanics,
    #[error("existing mechanics rejected the stick-slip step")]
    MechanicsRejected,
    #[error("contractility failed: {0}")]
    Contractility(#[from] ContractilityError),
}

fn validate_params(params: &StickSlipTractionParamsV1) -> Result<(), StickSlipError> {
    if params.schema != STICK_SLIP_TRACTION_SCHEMA_V1
        || !params.static_traction_limit.is_finite()
        || params.static_traction_limit <= 0.0
        || params.static_traction_limit > MAX_EXTERNAL_FORCE_PER_VERTEX
        || !params.kinetic_traction.is_finite()
        || params.kinetic_traction <= 0.0
        || params.kinetic_traction >= params.static_traction_limit
        || !params.zero_motion_tolerance.is_finite()
        || params.zero_motion_tolerance < 0.0
    {
        return Err(StickSlipError::InvalidParameters);
    }
    Ok(())
}

fn validate_mechanics(mechanics: &MechParams) -> Result<(), StickSlipError> {
    if !mechanics.gamma.is_finite()
        || mechanics.gamma <= 0.0
        || !mechanics.dt.is_finite()
        || mechanics.dt <= 0.0
    {
        return Err(StickSlipError::InvalidMechanics);
    }
    Ok(())
}

fn norm(vector: [f64; 2]) -> f64 {
    vector[0].hypot(vector[1])
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// Evaluate one local isotropic contact without any mesh or semantic input.
pub fn evaluate_contact(
    required_force: [f64; 2],
    attempted_velocity: [f64; 2],
    params: &StickSlipTractionParamsV1,
) -> Result<(ContactRegimeV1, [f64; 2]), StickSlipError> {
    validate_params(params)?;
    if required_force.iter().any(|value| !value.is_finite())
        || attempted_velocity.iter().any(|value| !value.is_finite())
    {
        return Err(StickSlipError::InvalidContact);
    }
    let required_norm = norm(required_force);
    if required_norm <= params.static_traction_limit {
        return Ok((
            ContactRegimeV1::Stick,
            [-required_force[0], -required_force[1]],
        ));
    }
    let speed = norm(attempted_velocity);
    if speed <= params.zero_motion_tolerance {
        return Err(StickSlipError::InvalidContact);
    }
    Ok((
        ContactRegimeV1::Slip,
        [
            -params.kinetic_traction * attempted_velocity[0] / speed,
            -params.kinetic_traction * attempted_velocity[1] / speed,
        ],
    ))
}

fn contacts_from_free_step(
    before: &MaterialMesh,
    free_step: &MaterialMesh,
    mechanics: &MechParams,
    params: &StickSlipTractionParamsV1,
) -> Result<(Vec<ContactRegimeV1>, Vec<[f64; 2]>), StickSlipError> {
    if before.n() != free_step.n() {
        return Err(StickSlipError::MechanicsRejected);
    }
    let mut regimes = Vec::with_capacity(before.n());
    let mut reactions = Vec::with_capacity(before.n());
    for (before_vertex, after_vertex) in before.vertices.iter().zip(&free_step.vertices) {
        let attempted_velocity = [
            (after_vertex[0] - before_vertex[0]) * mechanics.gamma / mechanics.dt,
            (after_vertex[1] - before_vertex[1]) * mechanics.gamma / mechanics.dt,
        ];
        let required_force = [
            attempted_velocity[0] * mechanics.gamma,
            attempted_velocity[1] * mechanics.gamma,
        ];
        let (regime, reaction) = evaluate_contact(required_force, attempted_velocity, params)?;
        regimes.push(regime);
        reactions.push(reaction);
    }
    Ok((regimes, reactions))
}

fn finish_step(
    before: &MaterialMesh,
    after: &MaterialMesh,
    free_step: &MaterialMesh,
    mechanics: &MechParams,
    params: &StickSlipTractionParamsV1,
    regimes: &[ContactRegimeV1],
    reactions: &[[f64; 2]],
    contractility: Option<ContractilityStepLedgerV1>,
) -> Result<StickSlipStepLedgerV1, StickSlipError> {
    if before.n() != after.n()
        || before.n() != free_step.n()
        || regimes.len() != before.n()
        || reactions.len() != before.n()
    {
        return Err(StickSlipError::MechanicsRejected);
    }
    let mut contacts = Vec::with_capacity(before.n());
    let mut maximum_stick_reaction: f64 = 0.0;
    let mut maximum_slip_reaction: f64 = 0.0;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    let mut substrate_work = 0.0;
    let mut maximum_attempted_velocity: f64 = 0.0;
    let mut maximum_accepted_velocity: f64 = 0.0;
    for index in 0..before.n() {
        let attempted_velocity = [
            (free_step.vertices[index][0] - before.vertices[index][0]) * mechanics.gamma
                / mechanics.dt,
            (free_step.vertices[index][1] - before.vertices[index][1]) * mechanics.gamma
                / mechanics.dt,
        ];
        let accepted_velocity = [
            (after.vertices[index][0] - before.vertices[index][0]) * mechanics.gamma / mechanics.dt,
            (after.vertices[index][1] - before.vertices[index][1]) * mechanics.gamma / mechanics.dt,
        ];
        let required_force = [
            attempted_velocity[0] * mechanics.gamma,
            attempted_velocity[1] * mechanics.gamma,
        ];
        let work = dot(
            reactions[index],
            [
                after.vertices[index][0] - before.vertices[index][0],
                after.vertices[index][1] - before.vertices[index][1],
            ],
        );
        if !work.is_finite() || work > params.zero_motion_tolerance {
            return Err(StickSlipError::InvalidContact);
        }
        let reaction_norm = norm(reactions[index]);
        match regimes[index] {
            ContactRegimeV1::Stick => {
                stuck_contacts += 1;
                maximum_stick_reaction = maximum_stick_reaction.max(reaction_norm);
            }
            ContactRegimeV1::Slip => {
                slipping_contacts += 1;
                maximum_slip_reaction = maximum_slip_reaction.max(reaction_norm);
            }
        }
        maximum_attempted_velocity = maximum_attempted_velocity.max(norm(attempted_velocity));
        maximum_accepted_velocity = maximum_accepted_velocity.max(norm(accepted_velocity));
        substrate_work += work;
        contacts.push(ContactLedgerV1 {
            regime: regimes[index],
            required_force: norm(required_force),
            attempted_velocity,
            reaction: reactions[index],
            accepted_velocity,
            work,
        });
    }
    Ok(StickSlipStepLedgerV1 {
        schema: STICK_SLIP_TRACTION_SCHEMA_V1.to_string(),
        contacts,
        maximum_stick_reaction,
        maximum_slip_reaction,
        stuck_contacts,
        slipping_contacts,
        substrate_work,
        maximum_attempted_velocity,
        maximum_accepted_velocity,
        contractility,
    })
}

/// Apply stick-slip to the existing legacy mechanics path.
pub fn apply_stick_slip_to_legacy_mechanics(
    mesh: &mut MaterialMesh,
    mechanics: &MechParams,
    params: &StickSlipTractionParamsV1,
) -> Result<StickSlipStepLedgerV1, StickSlipError> {
    validate_params(params)?;
    validate_mechanics(mechanics)?;
    let before = mesh.clone();
    let mut free_step = before.clone();
    if !mechanics_step(&mut free_step, mechanics) {
        return Err(StickSlipError::MechanicsRejected);
    }
    let (regimes, reactions) = contacts_from_free_step(&before, &free_step, mechanics, params)?;
    if !mechanics_step_with_external_forces(mesh, mechanics, &reactions) {
        return Err(StickSlipError::MechanicsRejected);
    }
    finish_step(
        &before, mesh, &free_step, mechanics, params, &regimes, &reactions, None,
    )
}

/// Apply stick-slip around the already-qualified reserve-funded contractility
/// adapter. The free forecast uses that adapter on a clone; the real mesh is
/// advanced once through the same adapter with only the local substrate
/// reactions added.
pub fn apply_local_contractility_with_stick_slip(
    mesh: &mut MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    params: &StickSlipTractionParamsV1,
) -> Result<StickSlipStepLedgerV1, StickSlipError> {
    validate_params(params)?;
    validate_mechanics(mechanics)?;
    let before = mesh.clone();
    let mut free_step = before.clone();
    let zero_external = vec![[0.0, 0.0]; before.n()];
    let _free_contractility = apply_local_contractility_with_external_forces(
        &mut free_step,
        activity,
        mechanics,
        contractility,
        Some(&zero_external),
    )?;
    let (regimes, reactions) = contacts_from_free_step(&before, &free_step, mechanics, params)?;
    let accepted_contractility = apply_local_contractility_with_external_forces(
        mesh,
        activity,
        mechanics,
        contractility,
        Some(&reactions),
    )?;
    finish_step(
        &before,
        mesh,
        &free_step,
        mechanics,
        params,
        &regimes,
        &reactions,
        Some(accepted_contractility),
    )
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
                r: 0.6,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        )
    }

    fn rotate(vector: [f64; 2], angle: f64) -> [f64; 2] {
        let (sin, cos) = angle.sin_cos();
        [
            cos * vector[0] - sin * vector[1],
            sin * vector[0] + cos * vector[1],
        ]
    }

    #[test]
    fn frozen_parameters_are_one_static_kinetic_set() {
        let params = StickSlipTractionParamsV1::default();
        assert_eq!(params.static_traction_limit, 0.45);
        assert_eq!(params.kinetic_traction, 0.20);
        assert!(params.kinetic_traction < params.static_traction_limit);
    }

    #[test]
    fn stick_cancels_local_force_and_does_zero_work() {
        let params = StickSlipTractionParamsV1::default();
        let (regime, reaction) = evaluate_contact([0.2, -0.3], [0.2, -0.3], &params).unwrap();
        assert_eq!(regime, ContactRegimeV1::Stick);
        assert_eq!(reaction, [-0.2, 0.3]);
    }

    #[test]
    fn slip_opposes_attempted_motion_and_is_bounded() {
        let params = StickSlipTractionParamsV1::default();
        let (regime, reaction) = evaluate_contact([0.6, 0.8], [0.6, 0.8], &params).unwrap();
        assert_eq!(regime, ContactRegimeV1::Slip);
        assert!(dot(reaction, [0.6, 0.8]) < 0.0);
        assert!((norm(reaction) - params.kinetic_traction).abs() < 1e-12);
        assert!(norm(reaction) < params.static_traction_limit);
    }

    #[test]
    fn rotation_equivalence_holds_without_global_direction() {
        let params = StickSlipTractionParamsV1::default();
        let force = [0.7, -0.2];
        let velocity = [0.7, -0.2];
        let angle = 1.137;
        let (_, reaction) = evaluate_contact(force, velocity, &params).unwrap();
        let (_, rotated_reaction) =
            evaluate_contact(rotate(force, angle), rotate(velocity, angle), &params).unwrap();
        let expected = rotate(reaction, angle);
        assert!((rotated_reaction[0] - expected[0]).abs() < 1e-12);
        assert!((rotated_reaction[1] - expected[1]).abs() < 1e-12);
    }

    #[test]
    fn legacy_adapter_is_deterministic_and_passive() {
        let mechanics = MechParams::default();
        let params = StickSlipTractionParamsV1::default();
        let mut first = mesh();
        let mut second = mesh();
        let first_ledger =
            apply_stick_slip_to_legacy_mechanics(&mut first, &mechanics, &params).unwrap();
        let second_ledger =
            apply_stick_slip_to_legacy_mechanics(&mut second, &mechanics, &params).unwrap();
        assert_eq!(first.vertices, second.vertices);
        assert_eq!(first_ledger, second_ledger);
        assert!(first_ledger.substrate_work <= params.zero_motion_tolerance);
    }
}
