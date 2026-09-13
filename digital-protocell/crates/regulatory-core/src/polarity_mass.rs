//! DC-M4-R3: an opt-in, pre-fission conserved polarity substrate.
//!
//! This module is intentionally not wired into any production transition.  It
//! owns only two edge-local material pools (`active_amount` and
//! `inactive_amount`) and their conservative reaction/transport ledger.  It
//! accepts edge control-volume lengths, not coordinates, forces, fission
//! decisions, or observer labels.  Consequently the R3 pattern qualification
//! cannot alter mechanics, growth placement, pressure, apposition, or
//! scission.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const POLARITY_MASS_SCHEMA_V1: &str = "digital_cell_polarity_mass_state_v1";
pub const POLARITY_MASS_PARAMS_SCHEMA_V1: &str = "digital_cell_polarity_mass_params_v1";
pub const POLARITY_SOURCE_LEDGER_SCHEMA_V1: &str = "digital_cell_polarity_source_ledger_v1";
pub const POLARITY_STEP_LEDGER_SCHEMA_V1: &str = "digital_cell_polarity_step_ledger_v1";

/// Prospective, dimensionally explicit parameters for the R3 sandbox.
///
/// `active_rate` and `inactive_rate` are concentration/time terms.  The
/// reaction is integrated as an amount rate by multiplying by the local edge
/// measure.  `active_diffusion` and `inactive_diffusion` have
/// length^2/time units.  `synthesis_a_per_amount` and
/// `conversion_a_per_amount` are source/energy amount per polarity amount.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityMassParamsV1 {
    pub schema: String,
    pub basal_activation_rate: f64,
    pub positive_feedback_rate: f64,
    pub basal_deactivation_rate: f64,
    pub quadratic_deactivation_rate: f64,
    pub active_diffusion: f64,
    pub inactive_diffusion: f64,
    pub synthesis_a_per_amount: f64,
    pub conversion_a_per_amount: f64,
    pub time_step: f64,
    pub integration_substeps: usize,
}

impl PolarityMassParamsV1 {
    /// The only R3 candidate parameterization.  These values are sealed by
    /// the R3 evidence workflow before any held-out Resource result is read.
    /// They are the R2 standalone MCRD values expressed in the amount-based
    /// edge-volume convention; no fission or apposition result selected them.
    pub fn candidate() -> Self {
        Self {
            schema: POLARITY_MASS_PARAMS_SCHEMA_V1.to_string(),
            basal_activation_rate: 0.01,
            positive_feedback_rate: 1.0,
            basal_deactivation_rate: 0.1,
            quadratic_deactivation_rate: 0.01,
            active_diffusion: 0.1,
            inactive_diffusion: 1.0,
            synthesis_a_per_amount: 1.0,
            conversion_a_per_amount: 0.5,
            time_step: 0.01,
            integration_substeps: 4,
        }
    }

    /// Matched null control: retain the same material, source, transport,
    /// integration and energy contract while disabling only the nonlinear
    /// positive-feedback term.
    pub fn polarity_null(&self) -> Self {
        let mut null = self.clone();
        null.positive_feedback_rate = 0.0;
        null
    }

    fn validate(&self) -> Result<(), PolarityMassError> {
        if self.schema != POLARITY_MASS_PARAMS_SCHEMA_V1 {
            return Err(PolarityMassError::InvalidParameters(
                "unsupported parameter schema".to_string(),
            ));
        }
        let values = [
            self.basal_activation_rate,
            self.positive_feedback_rate,
            self.basal_deactivation_rate,
            self.quadratic_deactivation_rate,
            self.active_diffusion,
            self.inactive_diffusion,
            self.synthesis_a_per_amount,
            self.conversion_a_per_amount,
            self.time_step,
        ];
        if values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
            || self.time_step <= 0.0
            || self.integration_substeps == 0
        {
            return Err(PolarityMassError::InvalidParameters(
                "parameters must be finite, nonnegative, and have positive time step".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityMassStateV1 {
    pub schema: String,
    pub active_amount: Vec<f64>,
    pub inactive_amount: Vec<f64>,
    pub accepted_steps: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolaritySourceLedgerV1 {
    pub schema: String,
    pub source_a_before: f64,
    pub source_a_debited: f64,
    pub polarity_created: f64,
    pub source_a_after: f64,
    pub material_fate: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityStepLedgerV1 {
    pub schema: String,
    pub accepted: bool,
    pub accepted_dt: f64,
    pub total_before: f64,
    pub total_after: f64,
    pub diffusion_residual: f64,
    pub reaction_transfer: f64,
    pub a_consumed: f64,
    pub w_produced: f64,
    pub active_before: f64,
    pub active_after: f64,
    pub inactive_before: f64,
    pub inactive_after: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarityModeSummaryV1 {
    pub mode: usize,
    pub amplitude: f64,
    pub cosine: f64,
    pub sine: f64,
}

#[derive(Debug, Error)]
pub enum PolarityMassError {
    #[error("invalid polarity parameters: {0}")]
    InvalidParameters(String),
    #[error("invalid edge control volumes: {0}")]
    InvalidMeasures(String),
    #[error("invalid polarity state: {0}")]
    InvalidState(String),
    #[error("source A is insufficient: required {required}, available {available}")]
    InsufficientSource { required: f64, available: f64 },
    #[error(
        "polarity chemistry A budget is insufficient: required {required}, available {available}"
    )]
    InsufficientEnergy { required: f64, available: f64 },
    #[error("polarity transaction was not accepted: {0}")]
    TransactionRejected(String),
}

impl PolarityMassStateV1 {
    /// Construct a source-funded state from local active/inactive
    /// concentrations.  The caller supplies the finite A source explicitly;
    /// this function never creates polarity material implicitly.
    pub fn from_source_funded_concentrations(
        measures: &[f64],
        active_concentration: &[f64],
        inactive_concentration: &[f64],
        source_a_available: f64,
        params: &PolarityMassParamsV1,
    ) -> Result<(Self, PolaritySourceLedgerV1), PolarityMassError> {
        params.validate()?;
        validate_measures(measures)?;
        if active_concentration.len() != measures.len()
            || inactive_concentration.len() != measures.len()
        {
            return Err(PolarityMassError::InvalidState(
                "concentration and measure lengths differ".to_string(),
            ));
        }
        if !source_a_available.is_finite() || source_a_available < 0.0 {
            return Err(PolarityMassError::InvalidState(
                "source A must be finite and nonnegative".to_string(),
            ));
        }
        let mut active_amount = Vec::with_capacity(measures.len());
        let mut inactive_amount = Vec::with_capacity(measures.len());
        for ((measure, active), inactive) in measures
            .iter()
            .zip(active_concentration)
            .zip(inactive_concentration)
        {
            if !active.is_finite() || !inactive.is_finite() || *active < 0.0 || *inactive < 0.0 {
                return Err(PolarityMassError::InvalidState(
                    "concentrations must be finite and nonnegative".to_string(),
                ));
            }
            active_amount.push(active * measure);
            inactive_amount.push(inactive * measure);
        }
        let polarity_created =
            active_amount.iter().sum::<f64>() + inactive_amount.iter().sum::<f64>();
        let source_a_debited = polarity_created * params.synthesis_a_per_amount;
        if source_a_debited > source_a_available {
            return Err(PolarityMassError::InsufficientSource {
                required: source_a_debited,
                available: source_a_available,
            });
        }
        let state = Self {
            schema: POLARITY_MASS_SCHEMA_V1.to_string(),
            active_amount,
            inactive_amount,
            accepted_steps: 0,
        };
        let ledger = PolaritySourceLedgerV1 {
            schema: POLARITY_SOURCE_LEDGER_SCHEMA_V1.to_string(),
            source_a_before: source_a_available,
            source_a_debited,
            polarity_created,
            source_a_after: source_a_available - source_a_debited,
            material_fate: "finite_existing_A_to_polarity_material; no_hidden_creation".to_string(),
        };
        Ok((state, ledger))
    }

    pub fn validate(&self, measures: &[f64]) -> Result<(), PolarityMassError> {
        if self.schema != POLARITY_MASS_SCHEMA_V1 {
            return Err(PolarityMassError::InvalidState(
                "unsupported state schema".to_string(),
            ));
        }
        validate_measures(measures)?;
        if self.active_amount.len() != measures.len()
            || self.inactive_amount.len() != measures.len()
        {
            return Err(PolarityMassError::InvalidState(
                "state and measure lengths differ".to_string(),
            ));
        }
        if self
            .active_amount
            .iter()
            .chain(&self.inactive_amount)
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(PolarityMassError::InvalidState(
                "amounts must be finite and nonnegative".to_string(),
            ));
        }
        Ok(())
    }

    pub fn total_amount(&self) -> f64 {
        self.active_amount.iter().sum::<f64>() + self.inactive_amount.iter().sum::<f64>()
    }

    pub fn active_total(&self) -> f64 {
        self.active_amount.iter().sum()
    }

    pub fn inactive_total(&self) -> f64 {
        self.inactive_amount.iter().sum()
    }

    /// Advance one accepted R3 interval.  The transaction is computed in
    /// scratch arrays and committed only after positivity, finite-state and A
    /// funding checks pass.  On error, the state and accepted-step counter are
    /// unchanged.
    pub fn advance(
        &mut self,
        measures: &[f64],
        params: &PolarityMassParamsV1,
        available_a: f64,
    ) -> Result<PolarityStepLedgerV1, PolarityMassError> {
        params.validate()?;
        self.validate(measures)?;
        if !available_a.is_finite() || available_a < 0.0 {
            return Err(PolarityMassError::InsufficientEnergy {
                required: 0.0,
                available: available_a,
            });
        }
        let total_before = self.total_amount();
        let active_before = self.active_total();
        let inactive_before = self.inactive_total();
        let mut active = self.active_amount.clone();
        let mut inactive = self.inactive_amount.clone();
        let h = params.time_step / params.integration_substeps as f64;
        let mut total_a = 0.0;
        let mut total_transfer = 0.0;
        let mut diffusion_residual = 0.0;
        for _ in 0..params.integration_substeps {
            let (next_active, active_diffusion_residual) =
                diffuse_once(&active, measures, params.active_diffusion, h)?;
            let (next_inactive, inactive_diffusion_residual) =
                diffuse_once(&inactive, measures, params.inactive_diffusion, h)?;
            let mut reacted_active = next_active;
            let mut reacted_inactive = next_inactive;
            let mut reaction_a = 0.0;
            let mut reaction_transfer = 0.0;
            for i in 0..measures.len() {
                let active_concentration = reacted_active[i] / measures[i];
                let inactive_concentration = reacted_inactive[i] / measures[i];
                let rate = (params.basal_activation_rate
                    + params.positive_feedback_rate * active_concentration * active_concentration)
                    * inactive_concentration
                    - (params.basal_deactivation_rate
                        + params.quadratic_deactivation_rate
                            * active_concentration
                            * active_concentration)
                        * active_concentration;
                let proposed_transfer = rate * measures[i] * h;
                let transfer = proposed_transfer
                    .max(-reacted_active[i])
                    .min(reacted_inactive[i]);
                reacted_active[i] += transfer;
                reacted_inactive[i] -= transfer;
                reaction_transfer += transfer;
                reaction_a += transfer.abs() * params.conversion_a_per_amount;
            }
            if !reaction_a.is_finite() || total_a + reaction_a > available_a {
                return Err(PolarityMassError::InsufficientEnergy {
                    required: total_a + reaction_a,
                    available: available_a,
                });
            }
            if reacted_active
                .iter()
                .chain(&reacted_inactive)
                .any(|value| !value.is_finite() || *value < 0.0)
            {
                return Err(PolarityMassError::TransactionRejected(
                    "positivity or finite-state check failed".to_string(),
                ));
            }
            active = reacted_active;
            inactive = reacted_inactive;
            total_a += reaction_a;
            total_transfer += reaction_transfer;
            diffusion_residual += active_diffusion_residual + inactive_diffusion_residual;
            if !active_diffusion_residual.is_finite() || !inactive_diffusion_residual.is_finite() {
                return Err(PolarityMassError::TransactionRejected(
                    "diffusion residual is non-finite".to_string(),
                ));
            }
        }
        self.active_amount = active;
        self.inactive_amount = inactive;
        self.accepted_steps = self.accepted_steps.saturating_add(1);
        let total_after = self.total_amount();
        let active_after = self.active_total();
        let inactive_after = self.inactive_total();
        Ok(PolarityStepLedgerV1 {
            schema: POLARITY_STEP_LEDGER_SCHEMA_V1.to_string(),
            accepted: true,
            accepted_dt: params.time_step,
            total_before,
            total_after,
            diffusion_residual,
            reaction_transfer: total_transfer,
            a_consumed: total_a,
            w_produced: total_a,
            active_before,
            active_after,
            inactive_before,
            inactive_after,
        })
    }

    /// Conservative arclength remap for an accepted mesh reconfiguration.
    /// Amount density is transported over overlapping normalized arclength
    /// control volumes; no material is created or discarded.
    pub fn remap_conservative(
        &self,
        old_measures: &[f64],
        new_measures: &[f64],
    ) -> Result<Self, PolarityMassError> {
        self.validate(old_measures)?;
        validate_measures(new_measures)?;
        let active_amount = remap_channel(&self.active_amount, old_measures, new_measures)?;
        let inactive_amount = remap_channel(&self.inactive_amount, old_measures, new_measures)?;
        Ok(Self {
            schema: POLARITY_MASS_SCHEMA_V1.to_string(),
            active_amount,
            inactive_amount,
            accepted_steps: self.accepted_steps,
        })
    }

    /// Partition by explicit parent-edge correspondence.  A daughter edge
    /// with `Some(parent_index)` inherits the parent's local concentration; a
    /// closing edge with `None` has no parent predecessor and receives zero.
    /// Each parent amount must be represented by at least one daughter edge.
    pub fn split_by_correspondence(
        &self,
        parent_measures: &[f64],
        daughter_a_measures: &[f64],
        daughter_a_parent: &[Option<usize>],
        daughter_b_measures: &[f64],
        daughter_b_parent: &[Option<usize>],
    ) -> Result<(Self, Self), PolarityMassError> {
        self.validate(parent_measures)?;
        validate_measures(daughter_a_measures)?;
        validate_measures(daughter_b_measures)?;
        if daughter_a_measures.len() != daughter_a_parent.len()
            || daughter_b_measures.len() != daughter_b_parent.len()
        {
            return Err(PolarityMassError::InvalidState(
                "daughter correspondence lengths differ".to_string(),
            ));
        }
        let a = split_channel(
            &self.active_amount,
            parent_measures,
            daughter_a_measures,
            daughter_a_parent,
            daughter_b_measures,
            daughter_b_parent,
        )?;
        let b = split_channel(
            &self.inactive_amount,
            parent_measures,
            daughter_a_measures,
            daughter_a_parent,
            daughter_b_measures,
            daughter_b_parent,
        )?;
        Ok((
            Self {
                schema: POLARITY_MASS_SCHEMA_V1.to_string(),
                active_amount: a,
                inactive_amount: b,
                accepted_steps: self.accepted_steps,
            },
            Self {
                schema: POLARITY_MASS_SCHEMA_V1.to_string(),
                active_amount: split_channel(
                    &self.active_amount,
                    parent_measures,
                    daughter_b_measures,
                    daughter_b_parent,
                    daughter_a_measures,
                    daughter_a_parent,
                )?,
                inactive_amount: split_channel(
                    &self.inactive_amount,
                    parent_measures,
                    daughter_b_measures,
                    daughter_b_parent,
                    daughter_a_measures,
                    daughter_a_parent,
                )?,
                accepted_steps: self.accepted_steps,
            },
        ))
    }

    /// Observer-only arclength Fourier summaries.  This method is not used by
    /// the transition and therefore cannot feed a mode result back into the
    /// substrate.
    pub fn mode_summaries(
        &self,
        measures: &[f64],
        max_mode: usize,
    ) -> Result<Vec<PolarityModeSummaryV1>, PolarityMassError> {
        self.validate(measures)?;
        let perimeter: f64 = measures.iter().sum();
        let mut centers = Vec::with_capacity(measures.len());
        let mut cursor = 0.0;
        for measure in measures {
            centers.push(cursor + 0.5 * measure);
            cursor += *measure;
        }
        let mean = self.active_total() / perimeter;
        let mut result = Vec::new();
        for mode in 1..=max_mode.min(measures.len() / 2) {
            let mut cosine = 0.0;
            let mut sine = 0.0;
            for i in 0..measures.len() {
                let phase = 2.0 * std::f64::consts::PI * mode as f64 * centers[i] / perimeter;
                let density = self.active_amount[i] / measures[i] - mean;
                cosine += density * phase.cos() * measures[i];
                sine += density * phase.sin() * measures[i];
            }
            cosine /= perimeter;
            sine /= perimeter;
            result.push(PolarityModeSummaryV1 {
                mode,
                amplitude: (cosine * cosine + sine * sine).sqrt(),
                cosine,
                sine,
            });
        }
        Ok(result)
    }
}

fn validate_measures(measures: &[f64]) -> Result<(), PolarityMassError> {
    if measures.len() < 3 {
        return Err(PolarityMassError::InvalidMeasures(
            "at least three positive edge control volumes are required".to_string(),
        ));
    }
    if measures
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(PolarityMassError::InvalidMeasures(
            "edge control volumes must be finite and positive".to_string(),
        ));
    }
    Ok(())
}

fn diffuse_once(
    amount: &[f64],
    measures: &[f64],
    coefficient: f64,
    dt: f64,
) -> Result<(Vec<f64>, f64), PolarityMassError> {
    let n = amount.len();
    let mut raw_flux = vec![0.0; n];
    let mut outgoing = vec![0.0; n];
    for i in 0..n {
        let j = (i + 1) % n;
        let interface = 0.5 * (measures[i] + measures[j]);
        let distance = amount[i] / measures[i] - amount[j] / measures[j];
        let flux = coefficient * distance / interface * dt;
        raw_flux[i] = flux;
        if flux > 0.0 {
            outgoing[i] += flux;
        } else {
            outgoing[j] += -flux;
        }
    }
    let mut scale = vec![1.0; n];
    for i in 0..n {
        if outgoing[i] > amount[i] {
            scale[i] = amount[i] / outgoing[i];
        }
    }
    let mut next = amount.to_vec();
    for i in 0..n {
        let j = (i + 1) % n;
        let flux = if raw_flux[i] >= 0.0 {
            raw_flux[i] * scale[i]
        } else {
            raw_flux[i] * scale[j]
        };
        next[i] -= flux;
        next[j] += flux;
    }
    if next.iter().any(|value| !value.is_finite() || *value < 0.0) {
        return Err(PolarityMassError::TransactionRejected(
            "diffusion generated an invalid amount".to_string(),
        ));
    }
    let residual = next.iter().sum::<f64>() - amount.iter().sum::<f64>();
    Ok((next, residual))
}

fn remap_channel(
    old_amount: &[f64],
    old_measures: &[f64],
    new_measures: &[f64],
) -> Result<Vec<f64>, PolarityMassError> {
    let old_total: f64 = old_measures.iter().sum();
    let new_total: f64 = new_measures.iter().sum();
    let mut old_starts = Vec::with_capacity(old_measures.len());
    let mut cursor = 0.0;
    for measure in old_measures {
        old_starts.push(cursor / old_total);
        cursor += *measure;
    }
    let mut new_starts = Vec::with_capacity(new_measures.len());
    cursor = 0.0;
    for measure in new_measures {
        new_starts.push(cursor / new_total);
        cursor += *measure;
    }
    let mut result = vec![0.0; new_measures.len()];
    for (new_i, new_measure) in new_measures.iter().enumerate() {
        let new_start = new_starts[new_i];
        let new_end = new_start + new_measure / new_total;
        for (old_i, old_measure) in old_measures.iter().enumerate() {
            let old_start = old_starts[old_i];
            let old_end = old_start + old_measure / old_total;
            let overlap = interval_overlap_periodic(new_start, new_end, old_start, old_end);
            result[new_i] += old_amount[old_i] * overlap / (old_measure / old_total);
        }
    }
    if result
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(PolarityMassError::TransactionRejected(
            "remap generated an invalid amount".to_string(),
        ));
    }
    Ok(result)
}

fn interval_overlap_periodic(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> f64 {
    let mut overlap = 0.0;
    for shift in [-1.0, 0.0, 1.0] {
        overlap += (a_end.min(b_end + shift) - a_start.max(b_start + shift)).max(0.0);
    }
    overlap
}

fn split_channel(
    parent_amount: &[f64],
    parent_measures: &[f64],
    primary_measures: &[f64],
    primary_parent: &[Option<usize>],
    secondary_measures: &[f64],
    secondary_parent: &[Option<usize>],
) -> Result<Vec<f64>, PolarityMassError> {
    let mut assigned_measure = vec![0.0; parent_measures.len()];
    for (measure, parent) in primary_measures.iter().zip(primary_parent) {
        if let Some(index) = parent {
            if *index >= parent_measures.len() {
                return Err(PolarityMassError::InvalidState(
                    "daughter parent index is out of range".to_string(),
                ));
            }
            assigned_measure[*index] += *measure;
        }
    }
    for (measure, parent) in secondary_measures.iter().zip(secondary_parent) {
        if let Some(index) = parent {
            if *index >= parent_measures.len() {
                return Err(PolarityMassError::InvalidState(
                    "daughter parent index is out of range".to_string(),
                ));
            }
            assigned_measure[*index] += *measure;
        }
    }
    for (index, (assigned, parent)) in assigned_measure.iter().zip(parent_amount).enumerate() {
        if *parent > 0.0 && *assigned <= 0.0 {
            return Err(PolarityMassError::InvalidState(format!(
                "parent edge {index} has no daughter correspondence"
            )));
        }
    }
    let mut result = vec![0.0; primary_measures.len()];
    for (i, (measure, parent)) in primary_measures.iter().zip(primary_parent).enumerate() {
        if let Some(index) = parent {
            result[i] = parent_amount[*index] * *measure / assigned_measure[*index];
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measures() -> Vec<f64> {
        vec![1.0, 1.1, 0.9, 1.2, 0.8, 1.05, 0.95, 1.15]
    }

    fn state() -> (PolarityMassStateV1, PolarityMassParamsV1) {
        let params = PolarityMassParamsV1::candidate();
        let m = measures();
        let active = vec![0.641; m.len()];
        let inactive = vec![0.159; m.len()];
        let (state, ledger) = PolarityMassStateV1::from_source_funded_concentrations(
            &m, &active, &inactive, 10_000.0, &params,
        )
        .expect("source-funded state");
        assert!((ledger.polarity_created - state.total_amount()).abs() < 1.0e-12);
        (state, params)
    }

    #[test]
    fn advance_is_positive_conservative_and_energy_accounted() {
        let (mut state, params) = state();
        let before = state.total_amount();
        let ledger = state
            .advance(&measures(), &params, 10_000.0)
            .expect("advance");
        assert!(ledger.accepted);
        assert!(state
            .active_amount
            .iter()
            .chain(&state.inactive_amount)
            .all(|value| *value >= 0.0 && value.is_finite()));
        assert!((state.total_amount() - before).abs() < 1.0e-10);
        assert!((ledger.a_consumed - ledger.w_produced).abs() < 1.0e-12);
        assert!((ledger.diffusion_residual).abs() < 1.0e-10);
    }

    #[test]
    fn insufficient_energy_is_atomic() {
        let (mut state, params) = state();
        let original = state.clone();
        let error = state
            .advance(&measures(), &params, 0.0)
            .expect_err("must reject");
        assert!(matches!(
            error,
            PolarityMassError::InsufficientEnergy { .. }
        ));
        assert_eq!(state, original);
    }

    #[test]
    fn remap_and_fission_preserve_both_pools() {
        let (state, _) = state();
        let remapped = state
            .remap_conservative(&measures(), &[0.7, 1.4, 0.8, 1.1, 1.0, 1.2, 0.9, 1.0])
            .expect("remap");
        assert!((remapped.total_amount() - state.total_amount()).abs() < 1.0e-10);
        let parent = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let da = vec![1.0, 1.0, 1.0, 1.0];
        let db = vec![1.0, 1.0, 1.0, 1.0];
        let map_a = vec![Some(0), Some(1), Some(2), Some(3)];
        let map_b = vec![Some(4), Some(5), Some(6), Some(7)];
        let (a, b) = state
            .split_by_correspondence(&parent, &da, &map_a, &db, &map_b)
            .expect("partition");
        assert!((a.total_amount() + b.total_amount() - state.total_amount()).abs() < 1.0e-10);
    }

    #[test]
    fn serialization_round_trip_preserves_restart_state() {
        let (mut state, params) = state();
        state
            .advance(&measures(), &params, 10_000.0)
            .expect("advance");
        let encoded = serde_json::to_vec(&state).expect("serialize state");
        let restored: PolarityMassStateV1 =
            serde_json::from_slice(&encoded).expect("deserialize state");
        assert_eq!(restored.schema, state.schema);
        assert_eq!(restored.accepted_steps, state.accepted_steps);
        assert_eq!(restored.active_amount.len(), state.active_amount.len());
        assert_eq!(restored.inactive_amount.len(), state.inactive_amount.len());
        for (actual, expected) in restored
            .active_amount
            .iter()
            .zip(state.active_amount.iter())
            .chain(
                restored
                    .inactive_amount
                    .iter()
                    .zip(state.inactive_amount.iter()),
            )
        {
            assert!((actual - expected).abs() <= 1.0e-15);
        }
    }

    #[test]
    fn rotation_of_ring_is_equivariant() {
        let (mut original, params) = state();
        let m = measures();
        let shift = 3;
        let rotate = |values: &[f64]| {
            (0..values.len())
                .map(|i| values[(i + shift) % values.len()])
                .collect::<Vec<_>>()
        };
        let mut rotated = PolarityMassStateV1 {
            schema: original.schema.clone(),
            active_amount: rotate(&original.active_amount),
            inactive_amount: rotate(&original.inactive_amount),
            accepted_steps: 0,
        };
        original.advance(&m, &params, 10_000.0).expect("original");
        rotated
            .advance(&rotate(&m), &params, 10_000.0)
            .expect("rotated");
        let unrotate = |values: &[f64]| {
            (0..values.len())
                .map(|i| values[(i + values.len() - shift) % values.len()])
                .collect::<Vec<_>>()
        };
        let back_active = unrotate(&rotated.active_amount);
        let back_inactive = unrotate(&rotated.inactive_amount);
        for (left, right) in original.active_amount.iter().zip(back_active) {
            assert!((left - right).abs() < 1.0e-10);
        }
        for (left, right) in original.inactive_amount.iter().zip(back_inactive) {
            assert!((left - right).abs() < 1.0e-10);
        }
    }
}
