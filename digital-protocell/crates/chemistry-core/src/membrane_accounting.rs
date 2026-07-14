//! D-008 conservative soluble-transport accounting.

use crate::membrane_transport::TransportSpecies;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SpeciesTransportAccounting {
    pub net_change_rate: f64,
    pub absolute_crossed_face_flux: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportStepAccounting {
    pub catalyst: SpeciesTransportAccounting,
    pub activated: SpeciesTransportAccounting,
    pub nutrient: SpeciesTransportAccounting,
    pub fuel: SpeciesTransportAccounting,
    pub waste: SpeciesTransportAccounting,
}

impl TransportStepAccounting {
    pub fn set(&mut self, species: TransportSpecies, value: SpeciesTransportAccounting) {
        match species {
            TransportSpecies::Catalyst => self.catalyst = value,
            TransportSpecies::Activated => self.activated = value,
            TransportSpecies::Nutrient => self.nutrient = value,
            TransportSpecies::Fuel => self.fuel = value,
            TransportSpecies::Waste => self.waste = value,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportAccountingState {
    pub last_step: TransportStepAccounting,
    pub cumulative: TransportStepAccounting,
    pub accepted_steps: u64,
}

impl TransportAccountingState {
    pub fn record_accepted(&mut self, step: TransportStepAccounting, dt: f64) {
        accumulate(&mut self.cumulative.catalyst, step.catalyst, dt);
        accumulate(&mut self.cumulative.activated, step.activated, dt);
        accumulate(&mut self.cumulative.nutrient, step.nutrient, dt);
        accumulate(&mut self.cumulative.fuel, step.fuel, dt);
        accumulate(&mut self.cumulative.waste, step.waste, dt);
        self.last_step = step;
        self.accepted_steps += 1;
    }
}

fn accumulate(
    cumulative: &mut SpeciesTransportAccounting,
    step: SpeciesTransportAccounting,
    dt: f64,
) {
    cumulative.net_change_rate += step.net_change_rate * dt;
    cumulative.absolute_crossed_face_flux += step.absolute_crossed_face_flux * dt;
}
