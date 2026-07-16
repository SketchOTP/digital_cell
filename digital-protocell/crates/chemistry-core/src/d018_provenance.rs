//! D-018 observer-only structural provenance tracer (E endogenous / K constraint).
//!
//! Does not alter chemistry, fields, transport, timesteps, or classifications.
//! When hooked into the constrained-radius assay, inventories track which structural
//! mass originated from synthesis vs external constraint restoration.

use serde::{Deserialize, Serialize};

pub const PROVENANCE_INVENTORY_TOL: f64 = 1e-8;
pub const CONSTRAINT_CONTAMINATION_W_FRAC_MAX: f64 = 0.05;
pub const CONSTRAINT_NET_FLUX_FRAC_MAX: f64 = 0.05;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureProvenanceTracer {
    pub endogenous: Vec<f64>,
    pub constraint: Vec<f64>,
    pub cumulative_w_from_endogenous: f64,
    pub cumulative_w_from_constraint: f64,
    pub cumulative_constraint_addition: f64,
    pub cumulative_constraint_removal: f64,
    pub cumulative_endogenous_production: f64,
    pub accepted_steps: u64,
    pub max_inventory_residual: f64,
}

impl StructureProvenanceTracer {
    pub fn init_from_phi(phi: &[f64]) -> Self {
        Self {
            endogenous: phi.to_vec(),
            constraint: vec![0.0; phi.len()],
            cumulative_w_from_endogenous: 0.0,
            cumulative_w_from_constraint: 0.0,
            cumulative_constraint_addition: 0.0,
            cumulative_constraint_removal: 0.0,
            cumulative_endogenous_production: 0.0,
            accepted_steps: 0,
            max_inventory_residual: 0.0,
        }
    }

    pub fn sum_endogenous(&self) -> f64 {
        self.endogenous.iter().copied().sum()
    }

    pub fn sum_constraint(&self) -> f64 {
        self.constraint.iter().copied().sum()
    }

    pub fn sum_total(&self) -> f64 {
        self.sum_endogenous() + self.sum_constraint()
    }

    pub fn constraint_fraction_of_structure(&self) -> f64 {
        let total = self.sum_total();
        if total > 0.0 {
            self.sum_constraint() / total
        } else {
            0.0
        }
    }

    pub fn endogenous_fraction_of_structure(&self) -> f64 {
        1.0 - self.constraint_fraction_of_structure()
    }

    pub fn total_structure_turnover_w(&self) -> f64 {
        self.cumulative_w_from_endogenous + self.cumulative_w_from_constraint
    }

    pub fn constraint_fraction_of_structure_w(&self) -> f64 {
        let t = self.total_structure_turnover_w();
        if t > 0.0 {
            self.cumulative_w_from_constraint / t
        } else {
            0.0
        }
    }

    pub fn constraint_fraction_of_total_w(&self, total_w_production: f64) -> f64 {
        if total_w_production > 0.0 {
            self.cumulative_w_from_constraint / total_w_production
        } else {
            0.0
        }
    }

    pub fn net_constraint_material_input(&self) -> f64 {
        self.cumulative_constraint_addition - self.cumulative_constraint_removal
    }

    pub fn constraint_turnovers(&self, prescribed_structure_mass: f64) -> f64 {
        if prescribed_structure_mass > 0.0 {
            self.cumulative_constraint_addition / prescribed_structure_mass
        } else {
            0.0
        }
    }

    /// Constrained-radius cell update: synthesis → decay attribution → constraint restore.
    ///
    /// After the step, `E+K` equals the prescribed φ at this cell (held fixed by the assay).
    pub fn record_constrained_cell(&mut self, idx: usize, produced: f64, decayed: f64, prescribed_phi: f64) {
        debug_assert!(produced >= 0.0 && decayed >= 0.0);
        // 1. Structure synthesis is endogenous.
        self.endogenous[idx] += produced;
        self.cumulative_endogenous_production += produced;

        // 2. Decay attributed proportionally to current inventories.
        let e = self.endogenous[idx].max(0.0);
        let k = self.constraint[idx].max(0.0);
        let inv = e + k;
        let (frac_e, frac_k) = if inv > 0.0 {
            (e / inv, k / inv)
        } else {
            (1.0, 0.0)
        };
        let w_e = decayed * frac_e;
        let w_k = decayed * frac_k;
        self.cumulative_w_from_endogenous += w_e;
        self.cumulative_w_from_constraint += w_k;
        self.endogenous[idx] = (e - w_e).max(0.0);
        self.constraint[idx] = (k - w_k).max(0.0);

        // 3. Constraint restores prescribed φ: addition = φ − (E+K) = decayed − produced
        //    when starting from E+K=φ before synthesis (invariant preserved).
        let after = self.endogenous[idx] + self.constraint[idx];
        let delta = prescribed_phi - after;
        if delta > 0.0 {
            self.constraint[idx] += delta;
            self.cumulative_constraint_addition += delta;
        } else if delta < 0.0 {
            let rem = -delta;
            let e2 = self.endogenous[idx];
            let k2 = self.constraint[idx];
            let inv2 = e2 + k2;
            if inv2 > 0.0 {
                let fe = e2 / inv2;
                let fk = k2 / inv2;
                self.endogenous[idx] = (e2 - rem * fe).max(0.0);
                self.constraint[idx] = (k2 - rem * fk).max(0.0);
            }
            self.cumulative_constraint_removal += rem;
        }

        let residual = (self.endogenous[idx] + self.constraint[idx] - prescribed_phi).abs();
        self.max_inventory_residual = self.max_inventory_residual.max(residual);
    }

    /// Unconstrained φ dynamics: synthesis endogenous, decay proportional, no constraint flux.
    pub fn record_unconstrained_cell(&mut self, idx: usize, produced: f64, decayed: f64) {
        debug_assert!(produced >= 0.0 && decayed >= 0.0);
        self.endogenous[idx] += produced;
        self.cumulative_endogenous_production += produced;

        let e = self.endogenous[idx].max(0.0);
        let k = self.constraint[idx].max(0.0);
        let inv = e + k;
        let (frac_e, frac_k) = if inv > 0.0 {
            (e / inv, k / inv)
        } else {
            (1.0, 0.0)
        };
        let w_e = decayed * frac_e;
        let w_k = decayed * frac_k;
        self.cumulative_w_from_endogenous += w_e;
        self.cumulative_w_from_constraint += w_k;
        self.endogenous[idx] = (e - w_e).max(0.0);
        self.constraint[idx] = (k - w_k).max(0.0);
    }

    pub fn mark_accepted_step(&mut self) {
        self.accepted_steps += 1;
    }

    pub fn inventory_closes_against_phi(&self, phi: &[f64]) -> bool {
        if phi.len() != self.endogenous.len() {
            return false;
        }
        let mut max_res = 0.0_f64;
        for idx in 0..phi.len() {
            let res = (self.endogenous[idx] + self.constraint[idx] - phi[idx]).abs();
            max_res = max_res.max(res);
        }
        max_res <= PROVENANCE_INVENTORY_TOL * (1.0 + phi.iter().copied().map(f64::abs).sum::<f64>())
    }

    pub fn metrics(&self, total_w_production: f64, prescribed_structure_mass: f64) -> ConstraintContaminationMetrics {
        ConstraintContaminationMetrics {
            constraint_fraction_of_structure: self.constraint_fraction_of_structure(),
            endogenous_fraction_of_structure: self.endogenous_fraction_of_structure(),
            constraint_fraction_of_structure_w: self.constraint_fraction_of_structure_w(),
            constraint_fraction_of_total_w: self.constraint_fraction_of_total_w(total_w_production),
            cumulative_constraint_addition: self.cumulative_constraint_addition,
            cumulative_constraint_removal: self.cumulative_constraint_removal,
            net_constraint_material_input: self.net_constraint_material_input(),
            constraint_turnovers: self.constraint_turnovers(prescribed_structure_mass),
            w_from_endogenous_structure: self.cumulative_w_from_endogenous,
            w_from_constraint_structure: self.cumulative_w_from_constraint,
            max_inventory_residual: self.max_inventory_residual,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct ConstraintContaminationMetrics {
    pub constraint_fraction_of_structure: f64,
    pub endogenous_fraction_of_structure: f64,
    pub constraint_fraction_of_structure_w: f64,
    pub constraint_fraction_of_total_w: f64,
    pub cumulative_constraint_addition: f64,
    pub cumulative_constraint_removal: f64,
    pub net_constraint_material_input: f64,
    pub constraint_turnovers: f64,
    pub w_from_endogenous_structure: f64,
    pub w_from_constraint_structure: f64,
    pub max_inventory_residual: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstraintContaminationClass {
    ConstraintUsable,
    ConstraintContaminated,
}

pub fn classify_constraint_contamination(
    constraint_fraction_of_total_w: f64,
    abs_net_constraint_flux: f64,
    total_structural_turnover: f64,
) -> ConstraintContaminationClass {
    let flux_ok = abs_net_constraint_flux
        <= CONSTRAINT_NET_FLUX_FRAC_MAX * total_structural_turnover.max(0.0);
    let w_ok = constraint_fraction_of_total_w <= CONSTRAINT_CONTAMINATION_W_FRAC_MAX;
    if flux_ok && w_ok {
        ConstraintContaminationClass::ConstraintUsable
    } else {
        ConstraintContaminationClass::ConstraintContaminated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HistoricalWasteOriginClass {
    ConstraintWasteDominant,
    EndogenousWasteDominant,
    MixedStructuralWaste,
    TracerInvalid,
}

pub fn classify_historical_waste_origin(
    constraint_fraction_of_total_w: f64,
    tracer_valid: bool,
) -> HistoricalWasteOriginClass {
    if !tracer_valid {
        return HistoricalWasteOriginClass::TracerInvalid;
    }
    if constraint_fraction_of_total_w >= 0.70 {
        HistoricalWasteOriginClass::ConstraintWasteDominant
    } else if constraint_fraction_of_total_w <= 0.30 {
        HistoricalWasteOriginClass::EndogenousWasteDominant
    } else {
        HistoricalWasteOriginClass::MixedStructuralWaste
    }
}
