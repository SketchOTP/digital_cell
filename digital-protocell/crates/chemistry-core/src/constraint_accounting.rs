//! D-011 observer-only structure constraint ledger (virtual φ chemistry).

use serde::{Deserialize, Serialize};

pub const CONSTRAINT_RESIDUAL_TOL: f64 = 1e-8;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StructureConstraintStep {
    pub virtual_production: f64,
    pub virtual_decay: f64,
    pub virtual_net: f64,
    pub constraint_flux: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StructureConstraintCumulative {
    pub virtual_production: f64,
    pub virtual_decay: f64,
    pub virtual_structure_flow: f64,
    pub structure_constraint_flux: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructureConstraintAccounting {
    pub last_step: StructureConstraintStep,
    pub cumulative: StructureConstraintCumulative,
    pub accepted_steps: u64,
}

impl StructureConstraintAccounting {
    pub fn record_accepted(&mut self, step: StructureConstraintStep) {
        self.cumulative.virtual_production += step.virtual_production;
        self.cumulative.virtual_decay += step.virtual_decay;
        self.cumulative.virtual_structure_flow += step.virtual_net;
        self.cumulative.structure_constraint_flux += step.constraint_flux;
        self.cumulative.residual += step.residual;
        self.last_step = step;
        self.accepted_steps += 1;
    }

    pub fn closes(&self) -> bool {
        self.cumulative.residual.abs()
            <= CONSTRAINT_RESIDUAL_TOL * self.cumulative.virtual_structure_flow.abs().max(1.0)
    }
}

pub fn build_constraint_step(
    virtual_production: f64,
    virtual_decay: f64,
) -> StructureConstraintStep {
    let virtual_net = virtual_production - virtual_decay;
    let constraint_flux = -virtual_net;
    StructureConstraintStep {
        virtual_production,
        virtual_decay,
        virtual_net,
        constraint_flux,
        residual: virtual_net + constraint_flux,
    }
}
