//! D-008 Stage C zero-dimensional activated metabolism.

use crate::accounting::FieldStepLedger;
use crate::config::SimParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ActivatedMetabolismRates {
    pub activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
    pub d_catalyst: f64,
    pub d_nutrient: f64,
    pub d_fuel: f64,
    pub d_activated: f64,
    pub d_waste: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivatedMetabolismStepAccounting {
    pub activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
    pub catalyst: FieldStepLedger,
    pub nutrient: FieldStepLedger,
    pub fuel: FieldStepLedger,
    pub activated: FieldStepLedger,
    pub waste: FieldStepLedger,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ActivatedMetabolismCumulativeAccounting {
    pub activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
    pub catalyst_reaction_delta: f64,
    pub nutrient_reaction_delta: f64,
    pub fuel_reaction_delta: f64,
    pub activated_reaction_delta: f64,
    pub waste_reaction_delta: f64,
    pub catalyst_clamp_correction: f64,
    pub nutrient_clamp_correction: f64,
    pub fuel_clamp_correction: f64,
    pub activated_clamp_correction: f64,
    pub waste_clamp_correction: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivatedMetabolismAccountingState {
    pub last_step: ActivatedMetabolismStepAccounting,
    pub cumulative: ActivatedMetabolismCumulativeAccounting,
    pub accepted_steps: u64,
}

impl ActivatedMetabolismAccountingState {
    pub fn record_accepted(&mut self, step: ActivatedMetabolismStepAccounting) {
        self.cumulative.activation += step.activation;
        self.cumulative.reproduction += step.reproduction;
        self.cumulative.activated_decay += step.activated_decay;
        self.cumulative.catalyst_turnover += step.catalyst_turnover;
        accumulate_field(
            &mut self.cumulative.catalyst_reaction_delta,
            &mut self.cumulative.catalyst_clamp_correction,
            &mut self.cumulative.residual,
            &step.catalyst,
        );
        accumulate_field(
            &mut self.cumulative.nutrient_reaction_delta,
            &mut self.cumulative.nutrient_clamp_correction,
            &mut self.cumulative.residual,
            &step.nutrient,
        );
        accumulate_field(
            &mut self.cumulative.fuel_reaction_delta,
            &mut self.cumulative.fuel_clamp_correction,
            &mut self.cumulative.residual,
            &step.fuel,
        );
        accumulate_field(
            &mut self.cumulative.activated_reaction_delta,
            &mut self.cumulative.activated_clamp_correction,
            &mut self.cumulative.residual,
            &step.activated,
        );
        accumulate_field(
            &mut self.cumulative.waste_reaction_delta,
            &mut self.cumulative.waste_clamp_correction,
            &mut self.cumulative.residual,
            &step.waste,
        );
        self.last_step = step;
        self.accepted_steps += 1;
    }
}

fn accumulate_field(
    reaction: &mut f64,
    correction: &mut f64,
    residual: &mut f64,
    ledger: &FieldStepLedger,
) {
    *reaction += ledger.reaction_delta;
    *correction += ledger.numerical_correction_delta;
    *residual += ledger.accounting_residual.abs();
}

/// Rates for C+N+F→C+A+W and C+A→2C+W, with unit stoichiometry.
pub fn activated_metabolism_rates(
    catalyst: f64,
    nutrient: f64,
    fuel: f64,
    activated: f64,
    params: &SimParams,
) -> ActivatedMetabolismRates {
    let c = catalyst.max(0.0);
    let n = nutrient.max(0.0);
    let f = fuel.max(0.0);
    let a = activated.max(0.0);
    let activation = params.k_d008_activation * c * n * f;
    let reproduction = params.k_d008_reproduction * c * a;
    let activated_decay = params.k_d008_activated_decay * a;
    let catalyst_turnover = params.k_d008_catalyst_turnover * c;
    ActivatedMetabolismRates {
        activation,
        reproduction,
        activated_decay,
        catalyst_turnover,
        d_catalyst: reproduction - catalyst_turnover,
        d_nutrient: -activation,
        d_fuel: -activation,
        d_activated: activation - reproduction - activated_decay,
        d_waste: activation + reproduction + activated_decay + catalyst_turnover,
    }
}
