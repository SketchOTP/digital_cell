//! D-039 observer-only membrane-material pulse-chase tracer.
//!
//! Tracks labeled mass in P and S without affecting chemistry, transport,
//! timestep selection, or candidate selection. Labels transfer proportionally
//! with underlying material during exchange and declared damage.
//!
//! Exact conservation: `label_p + label_s + label_removed_to_w == initial_inventory`
//! (within floating-point tolerance), except that only declared damage may move
//! label into `label_removed_to_w`.

use crate::surface_density::SurfaceAccountingTotals;
use serde::{Deserialize, Serialize};

/// Global (spatially aggregated) membrane-material identity tracer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembraneLabelTracer {
    /// Labeled precursor inventory.
    pub label_p: f64,
    /// Labeled mature-membrane inventory in S (pulse = "old" cohort).
    pub label_s: f64,
    /// Label removed into W by declared damage only.
    pub label_removed_to_w: f64,
    pub cumulative_adsorption: f64,
    pub cumulative_desorption: f64,
    pub accepted_steps: u64,
    pub initial_label_inventory: f64,
}

impl MembraneLabelTracer {
    /// Label all current P and S inventories.
    pub fn init_from_totals(total_p: f64, total_s: f64) -> Self {
        let label_p = total_p.max(0.0);
        let label_s = total_s.max(0.0);
        Self {
            label_p,
            label_s,
            label_removed_to_w: 0.0,
            cumulative_adsorption: 0.0,
            cumulative_desorption: 0.0,
            accepted_steps: 0,
            initial_label_inventory: label_p + label_s,
        }
    }

    /// Pulse-chase: label all existing S as the old cohort; clear P label so new
    /// precursor-derived membrane is unlabeled (old fraction falls as S is replaced).
    pub fn pulse_label_all_s_as_old(&mut self, total_s: f64) {
        let s = total_s.max(0.0);
        self.label_s = s;
        self.label_p = 0.0;
        self.label_removed_to_w = 0.0;
        self.initial_label_inventory = s;
        self.cumulative_adsorption = 0.0;
        self.cumulative_desorption = 0.0;
    }

    /// Old-label fraction among physical membrane mass.
    pub fn old_fraction_in_s(&self, total_s: f64) -> f64 {
        if total_s > 0.0 {
            (self.label_s / total_s).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn replacement_fraction(&self, total_s: f64) -> f64 {
        1.0 - self.old_fraction_in_s(total_s)
    }

    pub fn conserved_inventory(&self) -> f64 {
        self.label_p + self.label_s + self.label_removed_to_w
    }

    pub fn inventory_residual(&self) -> f64 {
        (self.conserved_inventory() - self.initial_label_inventory).abs()
    }

    /// Proportional label transfer from one accepted surface step's gross exchange.
    ///
    /// Requires physical totals *before* the step for correct fractions.
    pub fn record_accepted_exchange(
        &mut self,
        totals: &SurfaceAccountingTotals,
        total_p_before: f64,
        total_s_before: f64,
    ) {
        let ads = totals.exchange_forward.max(0.0);
        let des = totals.exchange_reverse.max(0.0);
        self.cumulative_adsorption += ads;
        self.cumulative_desorption += des;

        // Adsorption P→S: move labeled P proportional to ads / P.
        if ads > 0.0 && total_p_before > 0.0 && self.label_p > 0.0 {
            let frac = (ads / total_p_before).clamp(0.0, 1.0);
            let moved = (self.label_p * frac).min(self.label_p);
            self.label_p = (self.label_p - moved).max(0.0);
            self.label_s += moved;
        }

        // Desorption S→P: move labeled S proportional to des / S.
        if des > 0.0 && total_s_before > 0.0 && self.label_s > 0.0 {
            let frac = (des / total_s_before).clamp(0.0, 1.0);
            let moved = (self.label_s * frac).min(self.label_s);
            self.label_s = (self.label_s - moved).max(0.0);
            self.label_p += moved;
        }

        self.accepted_steps += 1;
    }

    /// Declared damage removes S mass and proportional label into W.
    pub fn record_declared_damage(&mut self, s_removed: f64, total_s_before: f64) {
        let removed = s_removed.max(0.0);
        if removed <= 0.0 || total_s_before <= 0.0 || self.label_s <= 0.0 {
            return;
        }
        let frac = (removed / total_s_before).clamp(0.0, 1.0);
        let take = (self.label_s * frac).min(self.label_s);
        self.label_s = (self.label_s - take).max(0.0);
        self.label_removed_to_w += take;
    }
}
