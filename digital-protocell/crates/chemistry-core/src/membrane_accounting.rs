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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MembraneStepAccounting {
    pub mass_before: f64,
    pub synthesis: f64,
    pub decay: f64,
    pub detachment: f64,
    pub diffusion_net: f64,
    pub pre_clamp_mass: f64,
    pub clamp_correction: f64,
    pub mass_after: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MembraneCumulativeAccounting {
    pub synthesis: f64,
    pub decay: f64,
    pub detachment: f64,
    pub diffusion_net: f64,
    pub clamp_correction: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MembraneAccountingState {
    pub last_step: MembraneStepAccounting,
    pub cumulative: MembraneCumulativeAccounting,
    pub accepted_steps: u64,
}

impl MembraneAccountingState {
    pub fn record_accepted(&mut self, step: MembraneStepAccounting) {
        self.cumulative.synthesis += step.synthesis;
        self.cumulative.decay += step.decay;
        self.cumulative.detachment += step.detachment;
        self.cumulative.diffusion_net += step.diffusion_net;
        self.cumulative.clamp_correction += step.clamp_correction;
        self.cumulative.residual += step.residual;
        self.last_step = step;
        self.accepted_steps += 1;
    }
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
