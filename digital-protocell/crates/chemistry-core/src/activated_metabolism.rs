//! D-008 Stage C zero-dimensional activated metabolism.

use crate::accounting::{FieldStepLedger, CUMULATIVE_RESIDUAL_TOL};
use crate::config::{EquationVersion, SimParams};
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

/// Stage C boundedness: cumulative C/A numerical correction must stay within ledger tolerance.
pub fn stage_c_clamp_negligible(cumulative: &ActivatedMetabolismCumulativeAccounting) -> bool {
    cumulative.catalyst_clamp_correction.abs() <= CUMULATIVE_RESIDUAL_TOL
        && cumulative.activated_clamp_correction.abs() <= CUMULATIVE_RESIDUAL_TOL
}

/// Rates for activation and reproduction. V1: A→C+W; V2: A→η_C C + (1−η_C) W.
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

    if params.equation_version.is_conservative_membrane_metabolism() {
        let eta_c = params.eta_c;
        return ActivatedMetabolismRates {
            activation,
            reproduction,
            activated_decay,
            catalyst_turnover,
            d_catalyst: eta_c * reproduction - catalyst_turnover,
            d_nutrient: -activation,
            d_fuel: -activation,
            d_activated: activation - reproduction - activated_decay,
            d_waste: activation
                + (1.0 - eta_c) * reproduction
                + activated_decay
                + catalyst_turnover,
        };
    }

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

/// Isolated per-unit-extent activation delta (N+F→A+W); shared by v1 and v2.
pub fn activation_isolated_delta(extent: f64) -> [f64; 7] {
    let mut d = [0.0; 7];
    d[2] = -extent; // N
    d[3] = -extent; // F
    d[5] = extent; // A
    d[4] = extent; // W
    d
}

/// Isolated catalyst-production delta for governed equation version.
pub fn catalyst_production_isolated_delta(extent: f64, params: &SimParams) -> [f64; 7] {
    if params.equation_version.is_conservative_membrane_metabolism() {
        let eta = params.eta_c;
        let mut d = [0.0; 7];
        d[1] = eta * extent;
        d[5] = -extent;
        d[4] = (1.0 - eta) * extent;
        return d;
    }
    let mut d = [0.0; 7];
    d[1] = extent;
    d[5] = -extent;
    d[4] = extent;
    d
}

/// Isolated turnover/decay delta: source species → W.
pub fn turnover_isolated_delta(source: usize, extent: f64) -> [f64; 7] {
    let mut d = [0.0; 7];
    d[source] = -extent;
    d[4] = extent;
    d
}
