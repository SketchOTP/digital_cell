//! D-013 Stage E harness integrity: accepted-step windows, checkpoints, activation
//! ledgers, termination semantics, and governed artifact validation.

use crate::d011_analysis::{
    totals_within_tolerance, window_slope, window_slopes_converged, window_time, SteadyWindowSnapshot,
    WindowSlopes, D011_SLOPE_TOL,
};
use crate::d012_accounting::{
    activation_potential, ActivationPotentialStep, MaterialEquivalentStep, E_ACTIVATED, E_FUEL,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ACTIVATION_POTENTIAL_SCHEMA_VERSION: u32 = 1;
pub const D013_CHECKPOINT_THRESHOLDS: [u64; 6] = [10_000, 25_000, 50_000, 100_000, 150_000, 200_000];
pub const D013_DEFAULT_REJECTION_STALL_LIMIT: u64 = 64;
pub const D013_REQUIRED_WINDOWS: u64 = 3;
pub const D013_FP_EQ_TOL: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TerminationReason {
    QuasiSteadyConverged,
    MaxAcceptedSubstepsReached,
    ResourceExhaustion,
    CatalystExtinction,
    ActivatedExtinction,
    MembraneExtinction,
    UnboundedAccumulation,
    OscillatoryUnresolved,
    TimestepFloorFailure,
    NumericalFailure,
    OperatorInterrupt,
    InvalidArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactValidationStatus {
    ValidGovernedArtifact,
    InvalidArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScientificClassification {
    QuasiSteadyConverged,
    NotConvergedAt200k,
    ResourceExhaustion,
    CatalystExtinction,
    ActivatedExtinction,
    MembraneExtinction,
    UnboundedAccumulation,
    OscillatoryUnresolved,
    NumericalFailure,
    InvalidArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcceptedStateSample {
    pub accepted_substep: u64,
    pub simulated_time: f64,
    pub mass_c: f64,
    pub mass_a: f64,
    pub mass_m: f64,
    pub mean_n_interior: f64,
    pub mean_f_interior: f64,
    pub mean_w_interior: f64,
    pub structure_production: f64,
    pub structure_decay: f64,
    pub catalyst_reproduction: f64,
    pub catalyst_turnover: f64,
    pub membrane_synthesis: f64,
    pub membrane_loss: f64,
    pub activation: f64,
    pub activated_loss: f64,
    pub nutrient_transport_interior: f64,
    pub fuel_transport_interior: f64,
    pub waste_transport_interior: f64,
    pub material_equivalent_total: f64,
    pub activation_potential_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowRecord {
    pub start_accepted_substep: u64,
    pub end_accepted_substep: u64,
    pub start_simulated_time: f64,
    pub end_simulated_time: f64,
    pub sample_count: u64,
    pub slopes: Option<WindowSlopes>,
    pub reaction_total_change: f64,
    pub transport_total_change: f64,
    pub valid: bool,
    pub qualifying: bool,
    pub consecutive_count: u64,
    pub invalid_reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ConvergenceCounter {
    pub consecutive_qualifying: u64,
    pub required: u64,
    pub windows: Vec<WindowRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivationPotentialLedger {
    pub activation_potential_schema_version: u32,
    pub field_weights: ActivationFieldWeights,
    pub reaction_interpretation: String,
    pub initial_activation_potential: f64,
    pub fuel_reservoir_contribution: f64,
    pub activation_transfer: f64,
    pub productive_consumption: f64,
    pub turnover_dissipation: f64,
    pub waste_associated_potential: f64,
    pub numerical_correction: f64,
    pub final_activation_potential: f64,
    pub residual: f64,
    pub relative_residual: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivationFieldWeights {
    pub e_fuel: f64,
    pub e_activated: f64,
}

impl Default for ActivationFieldWeights {
    fn default() -> Self {
        Self {
            e_fuel: E_FUEL,
            e_activated: E_ACTIVATED,
        }
    }
}

impl ActivationPotentialLedger {
    pub fn new(initial: f64) -> Self {
        Self {
            activation_potential_schema_version: ACTIVATION_POTENTIAL_SCHEMA_VERSION,
            field_weights: ActivationFieldWeights::default(),
            reaction_interpretation: "v2: e_F=1, e_A=1; productive chemistry consumes A potential; turnover/waste do not create potential".to_string(),
            initial_activation_potential: initial,
            fuel_reservoir_contribution: 0.0,
            activation_transfer: 0.0,
            productive_consumption: 0.0,
            turnover_dissipation: 0.0,
            waste_associated_potential: 0.0,
            numerical_correction: 0.0,
            final_activation_potential: initial,
            residual: 0.0,
            relative_residual: 0.0,
        }
    }

    pub fn apply_accepted_step(
        &mut self,
        step: &ActivationPotentialStep,
        activation_extent: f64,
        productive_a_consumed: f64,
        turnover_a_to_w: f64,
    ) {
        self.fuel_reservoir_contribution += step.fuel_import;
        // Activation transfer moves potential F→A; net potential unchanged under e_F=e_A=1.
        self.activation_transfer += activation_extent.max(0.0);
        self.productive_consumption += productive_a_consumed.max(0.0);
        self.turnover_dissipation += turnover_a_to_w.max(0.0);
        self.numerical_correction += step.numerical_correction;
        self.final_activation_potential = step.potential_after;
        let expected = self.initial_activation_potential
            + self.fuel_reservoir_contribution
            + self.numerical_correction
            - self.productive_consumption
            - self.turnover_dissipation;
        self.residual = self.final_activation_potential - expected;
        let scale = self
            .final_activation_potential
            .abs()
            .max(self.initial_activation_potential.abs())
            .max(1.0);
        self.relative_residual = self.residual.abs() / scale;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointThresholdEvent {
    pub previous_accepted_substeps: u64,
    pub current_accepted_substeps: u64,
    pub threshold: u64,
}

/// Detect thresholds crossed by an accepted-step advance (handles batch jumps).
pub fn crossed_checkpoint_thresholds(
    previous_accepted: u64,
    current_accepted: u64,
) -> Vec<CheckpointThresholdEvent> {
    D013_CHECKPOINT_THRESHOLDS
        .iter()
        .copied()
        .filter(|&threshold| previous_accepted < threshold && current_accepted >= threshold)
        .map(|threshold| CheckpointThresholdEvent {
            previous_accepted_substeps: previous_accepted,
            current_accepted_substeps: current_accepted,
            threshold,
        })
        .collect()
}

pub fn sample_to_window_snapshot(sample: &AcceptedStateSample, start: &AcceptedStateSample) -> SteadyWindowSnapshot {
    SteadyWindowSnapshot {
        start_step: start.accepted_substep,
        end_step: sample.accepted_substep,
        simulated_time_start: start.simulated_time,
        simulated_time_end: sample.simulated_time,
        mass_c: sample.mass_c,
        mass_a: sample.mass_a,
        mass_m: sample.mass_m,
        mean_n_interior: sample.mean_n_interior,
        mean_f_interior: sample.mean_f_interior,
        mean_w_interior: sample.mean_w_interior,
        structure_production: sample.structure_production,
        structure_decay: sample.structure_decay,
        catalyst_reproduction: sample.catalyst_reproduction,
        catalyst_turnover: sample.catalyst_turnover,
        membrane_synthesis: sample.membrane_synthesis,
        membrane_loss: sample.membrane_loss,
        activation: sample.activation,
        activated_loss: sample.activated_loss,
        nutrient_transport_interior: sample.nutrient_transport_interior,
        fuel_transport_interior: sample.fuel_transport_interior,
        waste_transport_interior: sample.waste_transport_interior,
    }
}

pub fn validate_accepted_window(
    samples: &[AcceptedStateSample],
    required_size: u64,
) -> Result<(), Vec<String>> {
    let mut reasons = Vec::new();
    if samples.len() as u64 != required_size {
        reasons.push(format!(
            "sample_count {} != required {}",
            samples.len(),
            required_size
        ));
    }
    if samples.len() < 2 {
        reasons.push("window requires at least two accepted samples".into());
        return Err(reasons);
    }
    let first = &samples[0];
    let last = samples.last().unwrap();
    if !(last.simulated_time > first.simulated_time) {
        reasons.push("simulated time must increase strictly".into());
    }
    if first.accepted_substep == last.accepted_substep
        && (first.mass_c - last.mass_c).abs() < D013_FP_EQ_TOL
        && (first.mass_a - last.mass_a).abs() < D013_FP_EQ_TOL
        && (first.mass_m - last.mass_m).abs() < D013_FP_EQ_TOL
    {
        reasons.push("first and final samples are not distinct accepted states".into());
    }
    for (i, s) in samples.iter().enumerate() {
        if !s.material_equivalent_total.is_finite() || !s.activation_potential_total.is_finite() {
            reasons.push(format!("sample {i} missing finite ledger totals"));
        }
        if i > 0 && !(s.simulated_time > samples[i - 1].simulated_time) {
            reasons.push(format!("sample {i} does not advance simulated time"));
        }
        if i > 0 && !(s.accepted_substep > samples[i - 1].accepted_substep) {
            reasons.push(format!("sample {i} does not advance accepted_substep"));
        }
    }
    if reasons.is_empty() {
        Ok(())
    } else {
        Err(reasons)
    }
}

pub fn build_window_record(
    samples: &[AcceptedStateSample],
    required_size: u64,
    prev: Option<&SteadyWindowSnapshot>,
    consecutive_before: u64,
) -> WindowRecord {
    let invalid_reasons = match validate_accepted_window(samples, required_size) {
        Ok(()) => Vec::new(),
        Err(r) => r,
    };
    let valid = invalid_reasons.is_empty();
    let first = samples.first();
    let last = samples.last();
    let snapshot = if let (Some(f), Some(l)) = (first, last) {
        Some(sample_to_window_snapshot(l, f))
    } else {
        None
    };

    let mut slopes = None;
    let mut reaction_total_change = 0.0;
    let mut transport_total_change = 0.0;
    let mut qualifying = false;
    if valid {
        if let (Some(prev_snap), Some(curr)) = (prev, snapshot.as_ref()) {
            let dt = window_time(curr);
            let slope = WindowSlopes {
                slope_c: window_slope(curr.mass_c, prev_snap.mass_c, dt),
                slope_a: window_slope(curr.mass_a, prev_snap.mass_a, dt),
                slope_m: window_slope(curr.mass_m, prev_snap.mass_m, dt),
                slope_n: window_slope(curr.mean_n_interior, prev_snap.mean_n_interior, dt),
                slope_f: window_slope(curr.mean_f_interior, prev_snap.mean_f_interior, dt),
                slope_w: window_slope(curr.mean_w_interior, prev_snap.mean_w_interior, dt),
                totals_within_tolerance: totals_within_tolerance(prev_snap, curr),
            };
            reaction_total_change = (curr.structure_production + curr.catalyst_reproduction
                + curr.membrane_synthesis
                + curr.activation)
                - (prev_snap.structure_production
                    + prev_snap.catalyst_reproduction
                    + prev_snap.membrane_synthesis
                    + prev_snap.activation);
            transport_total_change = (curr.nutrient_transport_interior
                + curr.fuel_transport_interior
                + curr.waste_transport_interior)
                - (prev_snap.nutrient_transport_interior
                    + prev_snap.fuel_transport_interior
                    + prev_snap.waste_transport_interior);
            qualifying = window_slopes_converged(&slope);
            let _ = D011_SLOPE_TOL; // documented threshold used inside window_slopes_converged
            slopes = Some(slope);
        }
    }

    let consecutive_count = if valid && qualifying {
        consecutive_before + 1
    } else {
        0
    };

    WindowRecord {
        start_accepted_substep: first.map(|s| s.accepted_substep).unwrap_or(0),
        end_accepted_substep: last.map(|s| s.accepted_substep).unwrap_or(0),
        start_simulated_time: first.map(|s| s.simulated_time).unwrap_or(0.0),
        end_simulated_time: last.map(|s| s.simulated_time).unwrap_or(0.0),
        sample_count: samples.len() as u64,
        slopes,
        reaction_total_change,
        transport_total_change,
        valid,
        qualifying,
        consecutive_count,
        invalid_reasons,
    }
}

pub fn update_convergence_counter(
    counter: &mut ConvergenceCounter,
    record: WindowRecord,
) -> bool {
    let consecutive = record.consecutive_count;
    counter.windows.push(record);
    counter.consecutive_qualifying = consecutive;
    consecutive >= counter.required
}

/// Three consecutive windows must use non-overlapping terminal evidence samples.
pub fn windows_use_nonoverlapping_terminal_evidence(windows: &[WindowRecord]) -> bool {
    if windows.len() < 3 {
        return false;
    }
    let last3 = &windows[windows.len() - 3..];
    last3[0].end_accepted_substep < last3[1].start_accepted_substep
        && last3[1].end_accepted_substep < last3[2].start_accepted_substep
        || (last3[0].end_accepted_substep <= last3[1].start_accepted_substep
            && last3[1].end_accepted_substep <= last3[2].start_accepted_substep
            && last3[0].end_accepted_substep < last3[1].end_accepted_substep
            && last3[1].end_accepted_substep < last3[2].end_accepted_substep)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernedArtifactView {
    pub source_commit: Option<String>,
    pub binary_hash: Option<String>,
    pub candidate_hash: Option<String>,
    pub configuration_hash: Option<String>,
    pub equation_version: Option<String>,
    pub field_schema: Option<String>,
    pub stoichiometric_schema: Option<u32>,
    pub checkpoint_completion: BTreeMap<String, bool>,
    pub accepted_substeps: Option<u64>,
    pub attempted_substeps: Option<u64>,
    pub rejected_substeps: Option<u64>,
    pub material_accounting: Option<MaterialEquivalentStep>,
    pub activation_potential_accounting: Option<ActivationPotentialLedger>,
    pub rolling_windows: Option<Vec<WindowRecord>>,
    pub termination_reason: Option<TerminationReason>,
    pub clean_termination: Option<bool>,
    pub field_hashes: Option<BTreeMap<String, String>>,
    pub artifact_hash: Option<String>,
}

pub fn validate_governed_artifact(
    view: &GovernedArtifactView,
) -> (ArtifactValidationStatus, Vec<String>) {
    let mut missing = Vec::new();
    if view.source_commit.as_ref().filter(|s| !s.is_empty()).is_none() {
        missing.push("source_commit".into());
    }
    if view.binary_hash.as_ref().filter(|s| !s.is_empty()).is_none() {
        missing.push("binary_hash".into());
    }
    if view.candidate_hash.as_ref().filter(|s| !s.is_empty()).is_none() {
        missing.push("candidate_hash".into());
    }
    if view
        .configuration_hash
        .as_ref()
        .filter(|s| !s.is_empty())
        .is_none()
    {
        missing.push("configuration_hash".into());
    }
    if view.equation_version.as_ref().filter(|s| !s.is_empty()).is_none() {
        missing.push("equation_version".into());
    }
    if view.field_schema.as_ref().filter(|s| !s.is_empty()).is_none() {
        missing.push("field_schema".into());
    }
    if view.stoichiometric_schema.is_none() {
        missing.push("stoichiometric_schema".into());
    }
    for threshold in D013_CHECKPOINT_THRESHOLDS {
        let key = threshold.to_string();
        // Only require checkpoints that should exist for the accepted horizon.
        if let Some(accepted) = view.accepted_substeps {
            if accepted >= threshold {
                match view.checkpoint_completion.get(&key) {
                    Some(true) => {}
                    _ => missing.push(format!("checkpoint_{threshold}")),
                }
            }
        } else {
            missing.push("accepted_substeps".into());
            break;
        }
    }
    if view.accepted_substeps.is_none() {
        missing.push("accepted_substeps".into());
    }
    if view.attempted_substeps.is_none() {
        missing.push("attempted_substeps".into());
    }
    if view.rejected_substeps.is_none() {
        missing.push("rejected_substeps".into());
    }
    if view.material_accounting.is_none() {
        missing.push("material_accounting".into());
    }
    if view.activation_potential_accounting.is_none() {
        missing.push("activation_potential_accounting".into());
    }
    match &view.rolling_windows {
        None => missing.push("rolling_windows".into()),
        Some(windows) => {
            if windows.iter().any(|w| !w.valid && w.qualifying) {
                missing.push("invalid_qualifying_window".into());
            }
        }
    }
    if view.termination_reason.is_none() {
        missing.push("termination_reason".into());
    }
    if view.clean_termination.is_none() {
        missing.push("clean_termination".into());
    }
    if view.field_hashes.as_ref().filter(|m| !m.is_empty()).is_none() {
        missing.push("field_hashes".into());
    }
    if view.artifact_hash.as_ref().filter(|s| !s.is_empty()).is_none() {
        missing.push("artifact_hash".into());
    }

    if missing.is_empty() {
        (ArtifactValidationStatus::ValidGovernedArtifact, missing)
    } else {
        (ArtifactValidationStatus::InvalidArtifact, missing)
    }
}

pub fn map_termination_to_scientific(
    reason: TerminationReason,
    max_steps: u64,
    accepted: u64,
) -> ScientificClassification {
    match reason {
        TerminationReason::QuasiSteadyConverged => ScientificClassification::QuasiSteadyConverged,
        TerminationReason::MaxAcceptedSubstepsReached => {
            if accepted >= max_steps && max_steps >= 200_000 {
                ScientificClassification::NotConvergedAt200k
            } else if accepted >= max_steps {
                // Preflight / short-horizon max: not a Stage E scientific non-convergence claim.
                ScientificClassification::NotConvergedAt200k
            } else {
                ScientificClassification::NumericalFailure
            }
        }
        TerminationReason::ResourceExhaustion => ScientificClassification::ResourceExhaustion,
        TerminationReason::CatalystExtinction => ScientificClassification::CatalystExtinction,
        TerminationReason::ActivatedExtinction => ScientificClassification::ActivatedExtinction,
        TerminationReason::MembraneExtinction => ScientificClassification::MembraneExtinction,
        TerminationReason::UnboundedAccumulation => ScientificClassification::UnboundedAccumulation,
        TerminationReason::OscillatoryUnresolved => ScientificClassification::OscillatoryUnresolved,
        TerminationReason::TimestepFloorFailure | TerminationReason::NumericalFailure => {
            ScientificClassification::NumericalFailure
        }
        TerminationReason::OperatorInterrupt | TerminationReason::InvalidArtifact => {
            ScientificClassification::InvalidArtifact
        }
    }
}

pub fn potential_from_masses(fuel: f64, activated: f64) -> f64 {
    activation_potential(fuel, activated)
}

/// Solver entry remains closed until a valid converged governed reference exists.
pub fn solver_entry_allowed(
    artifact_status: ArtifactValidationStatus,
    scientific: ScientificClassification,
    activation_present: bool,
    material_present: bool,
) -> bool {
    artifact_status == ArtifactValidationStatus::ValidGovernedArtifact
        && scientific == ScientificClassification::QuasiSteadyConverged
        && activation_present
        && material_present
}
