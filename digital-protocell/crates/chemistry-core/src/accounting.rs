//! Per-field mass accounting ledgers for scientific validation.

use crate::grid::Grid;
use crate::operators::total_mass;
use serde::{Deserialize, Serialize};

pub const STEP_RESIDUAL_TOL: f64 = 1e-8;
pub const CUMULATIVE_RESIDUAL_TOL: f64 = 1e-5;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldStepLedger {
    pub mass_before: f64,
    pub reaction_delta: f64,
    pub diffusion_delta: f64,
    pub reservoir_delta: f64,
    pub numerical_correction_delta: f64,
    pub mass_after: f64,
    pub accounting_residual: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepAccounting {
    pub structure: FieldStepLedger,
    pub catalyst: FieldStepLedger,
    pub nutrient: FieldStepLedger,
    pub fuel: FieldStepLedger,
    pub waste: FieldStepLedger,
    pub activated: FieldStepLedger,
    pub membrane: FieldStepLedger,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CumulativeLedger {
    pub structural_synthesis: f64,
    pub structural_decay: f64,
    pub catalyst_reproduction: f64,
    pub catalyst_decay: f64,
    pub nutrient_consumed_r1: f64,
    pub nutrient_consumed_r2: f64,
    pub fuel_consumed_r1: f64,
    pub fuel_consumed_r2: f64,
    pub waste_from_r1: f64,
    pub waste_from_r2: f64,
    pub waste_from_decay: f64,
    pub waste_removed_reservoir: f64,
    pub nutrient_supplied_reservoir: f64,
    pub fuel_supplied_reservoir: f64,
    pub clamp_corrections: f64,
    pub rejected_steps: u64,
    pub cumulative_unexplained_residual: f64,
    pub cumulative_processed_mass: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountingState {
    pub last_step: StepAccounting,
    pub cumulative: CumulativeLedger,
    pub max_step_residual: f64,
    pub steps_within_tolerance: u64,
    pub steps_outside_tolerance: u64,
}

impl AccountingState {
    pub fn record_step(
        &mut self,
        step: StepAccounting,
        reaction_totals: &ReactionStepTotals,
        clamp_total: f64,
    ) {
        let total_mass = step.structure.mass_before
            + step.catalyst.mass_before
            + step.nutrient.mass_before
            + step.fuel.mass_before
            + step.waste.mass_before
            + step.activated.mass_before
            + step.membrane.mass_before;

        let step_residual = step.structure.accounting_residual.abs()
            + step.catalyst.accounting_residual.abs()
            + step.nutrient.accounting_residual.abs()
            + step.fuel.accounting_residual.abs()
            + step.waste.accounting_residual.abs()
            + step.activated.accounting_residual.abs()
            + step.membrane.accounting_residual.abs();

        self.max_step_residual = self.max_step_residual.max(step_residual);
        self.cumulative.cumulative_processed_mass += total_mass;
        self.cumulative.cumulative_unexplained_residual += step_residual;

        let tol = STEP_RESIDUAL_TOL * total_mass.max(1.0);
        if step_residual <= tol {
            self.steps_within_tolerance += 1;
        } else {
            self.steps_outside_tolerance += 1;
        }

        self.cumulative.structural_synthesis += reaction_totals.structural_synthesis;
        self.cumulative.structural_decay += reaction_totals.structural_decay;
        self.cumulative.catalyst_reproduction += reaction_totals.catalyst_reproduction;
        self.cumulative.catalyst_decay += reaction_totals.catalyst_decay;
        self.cumulative.nutrient_consumed_r1 += reaction_totals.nutrient_consumed_r1;
        self.cumulative.nutrient_consumed_r2 += reaction_totals.nutrient_consumed_r2;
        self.cumulative.fuel_consumed_r1 += reaction_totals.fuel_consumed_r1;
        self.cumulative.fuel_consumed_r2 += reaction_totals.fuel_consumed_r2;
        self.cumulative.waste_from_r1 += reaction_totals.waste_from_r1;
        self.cumulative.waste_from_r2 += reaction_totals.waste_from_r2;
        self.cumulative.waste_from_decay += reaction_totals.waste_from_decay;
        self.cumulative.waste_removed_reservoir += step.waste.reservoir_delta.min(0.0).abs();
        self.cumulative.nutrient_supplied_reservoir += step.nutrient.reservoir_delta.max(0.0);
        self.cumulative.fuel_supplied_reservoir += step.fuel.reservoir_delta.max(0.0);
        self.cumulative.clamp_corrections += clamp_total;

        self.last_step = step;
    }

    pub fn cumulative_within_tolerance(&self) -> bool {
        let processed = self.cumulative.cumulative_processed_mass.max(1.0);
        self.cumulative.cumulative_unexplained_residual
            <= CUMULATIVE_RESIDUAL_TOL * processed
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReactionStepTotals {
    pub structural_synthesis: f64,
    pub structural_decay: f64,
    pub catalyst_reproduction: f64,
    pub catalyst_decay: f64,
    pub nutrient_consumed_r1: f64,
    pub nutrient_consumed_r2: f64,
    pub fuel_consumed_r1: f64,
    pub fuel_consumed_r2: f64,
    pub waste_from_r1: f64,
    pub waste_from_r2: f64,
    pub waste_from_decay: f64,
}

pub fn field_mass(grid: &Grid, field: &[f64]) -> f64 {
    total_mass(grid, field)
}

pub fn build_field_ledger(
    mass_before: f64,
    reaction_delta: f64,
    diffusion_delta: f64,
    reservoir_delta: f64,
    pre_clamp_mass: f64,
    mass_after: f64,
) -> FieldStepLedger {
    let integrated = mass_before + reaction_delta + diffusion_delta + reservoir_delta;
    let numerical_correction_delta = mass_after - pre_clamp_mass;
    let accounting_residual =
        mass_after - (mass_before + reaction_delta + diffusion_delta + reservoir_delta + numerical_correction_delta);
    FieldStepLedger {
        mass_before,
        reaction_delta,
        diffusion_delta,
        reservoir_delta,
        numerical_correction_delta,
        mass_after,
        accounting_residual,
    }
}

pub fn sum_clamp_correction(before: &[f64], after: &[f64], grid: &Grid) -> f64 {
    let mut delta = 0.0;
    for idx in 0..before.len() {
        if grid.in_dish(idx) {
            delta += after[idx] - before[idx];
        }
    }
    delta
}
