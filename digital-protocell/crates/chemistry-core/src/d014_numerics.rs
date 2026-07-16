//! D-014 numerical diagnostics: timestep limiters and rejection attribution.

use serde::{Deserialize, Serialize};

pub const D014_NUMERICAL_METHOD_VERSION: u32 = 2;
pub const D014_ADAPTIVE_CONTROLLER_VERSION: u32 = 2;
pub const D014_DT_FLOOR: f64 = 1e-8;
pub const D014_DT_RECOVERY_GROWTH: f64 = 1.25;
pub const D014_ACTIVATION_STEP_REL_TOL: f64 = 1e-6;
/// Absolute overshoot above CONC_SAFETY_LIMIT treated as roundoff (Branch E).
pub const D014_CONC_CEILING_PROJECT_EPS: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DtLimiter {
    DiffusionC,
    DiffusionN,
    DiffusionF,
    DiffusionW,
    DiffusionA,
    DiffusionM,
    MembraneTransportC,
    MembraneTransportN,
    MembraneTransportF,
    MembraneTransportW,
    MembraneTransportA,
    ActivationReaction,
    CatalystReproduction,
    StructureReaction,
    MembraneReaction,
    StructureTurnover,
    CatalystTurnover,
    ActivatedTurnover,
    MembraneTurnover,
    MembraneDetachment,
    ReservoirRelaxation,
    PositivityLimit,
    FieldBoundValidation,
    NonfiniteValue,
    AdaptiveController,
    IncomingStateInvalid,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NumericalCauseClassification {
    AdaptiveControllerRatchet,
    ReactionStiffness,
    TransportStiffness,
    MembraneDiffusionStiffness,
    ReservoirStiffness,
    PositivityStiffness,
    FieldBoundStiffness,
    NonfiniteNumericalInstability,
    CheckpointContinuationDefect,
    MultipleCoupledStiffness,
    UnknownNumericalFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttemptTelemetry {
    pub accepted_substep: u64,
    pub attempt_index: u64,
    pub simulated_time: f64,
    pub dt_entering: f64,
    pub dt_attempted: f64,
    pub accepted: bool,
    pub limiter: DtLimiter,
    pub rejection_reason: Option<String>,
    pub failing_field: Option<String>,
    pub failing_index: Option<usize>,
    pub max_c: f64,
    pub max_a: f64,
    pub max_m: f64,
    pub min_c: f64,
    pub min_a: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LimiterTransition {
    pub previous_limiter: DtLimiter,
    pub new_limiter: DtLimiter,
    pub accepted_substep: u64,
    pub simulated_time: f64,
    pub previous_dt: f64,
    pub new_dt: f64,
    pub controlling_value: f64,
}

pub fn classify_cause_from_terminal_limiter(limiter: DtLimiter) -> NumericalCauseClassification {
    match limiter {
        DtLimiter::AdaptiveController => NumericalCauseClassification::AdaptiveControllerRatchet,
        DtLimiter::IncomingStateInvalid | DtLimiter::FieldBoundValidation => {
            NumericalCauseClassification::FieldBoundStiffness
        }
        DtLimiter::PositivityLimit => NumericalCauseClassification::PositivityStiffness,
        DtLimiter::NonfiniteValue => NumericalCauseClassification::NonfiniteNumericalInstability,
        DtLimiter::ReservoirRelaxation => NumericalCauseClassification::ReservoirStiffness,
        DtLimiter::DiffusionC
        | DtLimiter::DiffusionN
        | DtLimiter::DiffusionF
        | DtLimiter::DiffusionW
        | DtLimiter::DiffusionA
        | DtLimiter::DiffusionM
        | DtLimiter::MembraneTransportC
        | DtLimiter::MembraneTransportN
        | DtLimiter::MembraneTransportF
        | DtLimiter::MembraneTransportW
        | DtLimiter::MembraneTransportA => NumericalCauseClassification::TransportStiffness,
        DtLimiter::MembraneReaction
        | DtLimiter::MembraneTurnover
        | DtLimiter::MembraneDetachment => NumericalCauseClassification::MembraneDiffusionStiffness,
        DtLimiter::ActivationReaction
        | DtLimiter::CatalystReproduction
        | DtLimiter::StructureReaction
        | DtLimiter::StructureTurnover
        | DtLimiter::CatalystTurnover
        | DtLimiter::ActivatedTurnover => NumericalCauseClassification::ReactionStiffness,
        DtLimiter::Unknown => NumericalCauseClassification::UnknownNumericalFailure,
    }
}

/// Next dt after an accepted step: allow bounded recovery toward MAX_DT.
pub fn recovered_dt_after_accept(current_accepted_dt: f64, max_dt: f64) -> f64 {
    (current_accepted_dt * D014_DT_RECOVERY_GROWTH).min(max_dt)
}
