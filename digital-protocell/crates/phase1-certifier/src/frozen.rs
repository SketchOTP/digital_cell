//! Frozen D-086 center candidate — no reselection.

use chemistry_core::material_mesh::{EQUATION_VERSION_MATERIAL_MESH, FIELD_SCHEMA_MATERIAL_MESH};
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_transport::TransportParams;
use serde::{Deserialize, Serialize};

pub const FROZEN_SCHEMA: &str = EQUATION_VERSION_MATERIAL_MESH;
pub const FROZEN_STATE: &str = FIELD_SCHEMA_MATERIAL_MESH;
pub const FROZEN_COMMIT: &str = "6f8a80a";
pub const FROZEN_TAG: &str = "D-086-mesh-protocell-phase1-pass";
pub const FROZEN_BRANCH: &str = "phase1-autopoietic-material-mesh";

pub const D087_AGENT_ID: &str = "D-20260724-d087-independent-phase1-certification-phase2-launch";
pub const D087_PROJECT_ID: &str = "D-087";

/// Exact D-086 center mechanical candidate.
pub const FROZEN_CENTER: MechParams = MechParams {
    gamma: 1.0,
    k_s: 14.0,
    kappa_b: 2.0,
    k_pi: 0.22,
    dt: 0.02,
};

pub fn frozen_reactions() -> ReactionParams {
    ReactionParams::default()
}

pub fn frozen_transport() -> TransportParams {
    TransportParams::default()
}

pub fn verify_frozen_center(m: &MechParams) -> bool {
    (m.gamma - FROZEN_CENTER.gamma).abs() < 1e-12
        && (m.k_s - FROZEN_CENTER.k_s).abs() < 1e-12
        && (m.kappa_b - FROZEN_CENTER.kappa_b).abs() < 1e-12
        && (m.k_pi - FROZEN_CENTER.k_pi).abs() < 1e-12
        && (m.dt - FROZEN_CENTER.dt).abs() < 1e-12
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenIdentity {
    pub schema: String,
    pub state: String,
    pub gamma: f64,
    pub k_s: f64,
    pub kappa_b: f64,
    pub k_pi: f64,
    pub dt: f64,
    pub alpha_approx: f64,
}

pub fn frozen_identity() -> FrozenIdentity {
    FrozenIdentity {
        schema: FROZEN_SCHEMA.into(),
        state: FROZEN_STATE.into(),
        gamma: FROZEN_CENTER.gamma,
        k_s: FROZEN_CENTER.k_s,
        kappa_b: FROZEN_CENTER.kappa_b,
        k_pi: FROZEN_CENTER.k_pi,
        dt: FROZEN_CENTER.dt,
        alpha_approx: FROZEN_CENTER.k_pi * 1.4 / FROZEN_CENTER.k_s,
    }
}
