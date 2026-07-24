//! Independent Phase 1 certification of the frozen D-086 center material-mesh candidate.
//!
//! **Must not** import `chemistry_core::d086_analysis` gate conclusions or route helpers.
//! May import mesh schemas and step kernels (material_mesh / mesh_*).

pub mod frozen;
pub mod metrics;
pub mod sim;
pub mod source_audit;
pub mod gates;
pub mod runtime;
pub mod campaign;

pub use campaign::{run_certification, CertificationReport};
pub use frozen::{FROZEN_CENTER, FROZEN_SCHEMA, FROZEN_STATE};
