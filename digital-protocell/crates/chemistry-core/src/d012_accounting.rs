//! D-012 material-equivalent and activation-potential observer ledgers.

use crate::accounting::{FieldStepLedger, StepAccounting};
use crate::config::EquationVersion;
use crate::stoichiometry::{Rational, SEVEN_FIELD_COUNT, SpeciesId};
use serde::{Deserialize, Serialize};

pub const MATERIAL_ACCOUNTING_REL_TOL: f64 = 1e-6;
pub const ACTIVATION_POTENTIAL_REL_TOL: f64 = 1e-6;

/// Initial governed weights (exact): e_F = 1, e_A = 1; other component potentials start at 0.
pub const E_FUEL: f64 = 1.0;
pub const E_ACTIVATED: f64 = 1.0;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MaterialEquivalentStep {
    pub total_before: f64,
    pub total_after: f64,
    pub observed_change: f64,
    pub reservoir_input: f64,
    pub waste_clearance: f64,
    pub numerical_correction: f64,
    pub boundary_exchange: f64,
    pub residual: f64,
    pub relative_residual: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ActivationPotentialStep {
    pub potential_before: f64,
    pub potential_after: f64,
    pub observed_change: f64,
    pub fuel_import: f64,
    pub numerical_correction: f64,
    pub residual: f64,
    pub relative_residual: f64,
}

pub fn material_weight_vector() -> [Rational; SEVEN_FIELD_COUNT] {
    [Rational::ONE; SEVEN_FIELD_COUNT]
}

pub fn total_material_equivalent(ledgers: &[FieldStepLedger]) -> f64 {
    ledgers.iter().map(|l| l.mass_after).sum()
}

pub fn activation_potential(fuel_mass: f64, activated_mass: f64) -> f64 {
    E_FUEL * fuel_mass + E_ACTIVATED * activated_mass
}

pub fn build_material_equivalent_step(step: &StepAccounting) -> MaterialEquivalentStep {
    let ledgers = [
        &step.structure,
        &step.catalyst,
        &step.nutrient,
        &step.fuel,
        &step.waste,
        &step.activated,
        &step.membrane,
    ];
    let total_before: f64 = ledgers.iter().map(|l| l.mass_before).sum();
    let total_after: f64 = ledgers.iter().map(|l| l.mass_after).sum();
    let observed_change = total_after - total_before;
    let reservoir_input = step.nutrient.reservoir_delta.max(0.0)
        + step.fuel.reservoir_delta.max(0.0)
        + step.waste.reservoir_delta.max(0.0);
    let waste_clearance = (-step.waste.reservoir_delta).max(0.0);
    let numerical_correction: f64 = ledgers
        .iter()
        .map(|l| l.numerical_correction_delta)
        .sum();
    let boundary_exchange = reservoir_input - waste_clearance + numerical_correction;
    let residual = observed_change - boundary_exchange;
    let scale = total_before.abs().max(total_after.abs()).max(1.0);
    MaterialEquivalentStep {
        total_before,
        total_after,
        observed_change,
        reservoir_input,
        waste_clearance,
        numerical_correction,
        boundary_exchange,
        residual,
        relative_residual: residual.abs() / scale,
    }
}

pub fn build_activation_potential_step(step: &StepAccounting) -> ActivationPotentialStep {
    let potential_before = activation_potential(step.fuel.mass_before, step.activated.mass_before);
    let potential_after = activation_potential(step.fuel.mass_after, step.activated.mass_after);
    let observed_change = potential_after - potential_before;
    let fuel_import = step.fuel.reservoir_delta.max(0.0);
    let numerical_correction = step.fuel.numerical_correction_delta
        + step.activated.numerical_correction_delta;
    let residual = observed_change - fuel_import - numerical_correction;
    let scale = potential_before.abs().max(potential_after.abs()).max(1.0);
    ActivationPotentialStep {
        potential_before,
        potential_after,
        observed_change,
        fuel_import,
        numerical_correction,
        residual,
        relative_residual: residual.abs() / scale,
    }
}

pub fn material_step_closes(step: &MaterialEquivalentStep) -> bool {
    step.relative_residual <= MATERIAL_ACCOUNTING_REL_TOL
}

pub fn activation_step_closes(step: &ActivationPotentialStep) -> bool {
    step.relative_residual <= ACTIVATION_POTENTIAL_REL_TOL
}

/// Closed-reactor internal reaction extent contributes zero under v2 all-ones weight.
pub fn internal_extent_conserves_material(delta: &[Rational; SEVEN_FIELD_COUNT]) -> bool {
    let m = material_weight_vector();
    let mut sum = Rational::ZERO;
    for (i, &mi) in m.iter().enumerate() {
        sum = sum.add(mi.mul(delta[i]));
    }
    sum.is_zero()
}

pub fn reaction_delta_creates_activation_potential(delta: &[f64; SEVEN_FIELD_COUNT]) -> bool {
    let potential_change =
        E_FUEL * delta[SpeciesId::F.index()] + E_ACTIVATED * delta[SpeciesId::A.index()];
    potential_change > 1e-12
}

pub fn waste_is_consumed_as_reactant(delta: &[f64; SEVEN_FIELD_COUNT]) -> bool {
    delta[SpeciesId::W.index()] < -1e-12
}

pub fn requires_v2_accounting(equation: EquationVersion) -> bool {
    equation == EquationVersion::MembraneMetabolismV2Conservative
}
