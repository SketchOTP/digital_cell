//! D-091 seasonal / pulse-lean resource ecology (finite pulses, not steady inflow).

use crate::spatial_shared_dish::SpatialDish;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PulseLeanSchedule {
    /// Full cycle length (time units).
    pub cycle_period: f64,
    /// Fraction of cycle that is the resource pulse (directive: 0.20).
    pub pulse_fraction: f64,
    /// Total N+F mass delivered per cycle to the dish (split evenly across pulse steps).
    pub cycle_nf_budget: f64,
    /// Background lean supply rate (usually 0 — no hidden refill).
    pub lean_nf_rate: f64,
}

impl Default for PulseLeanSchedule {
    fn default() -> Self {
        Self {
            cycle_period: 100.0,
            pulse_fraction: 0.20,
            cycle_nf_budget: 0.0,
            lean_nf_rate: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseLeanState {
    pub schedule: PulseLeanSchedule,
    pub t: f64,
    pub cycles_completed: u32,
    pub nf_delivered: f64,
}

impl PulseLeanState {
    pub fn new(schedule: PulseLeanSchedule) -> Self {
        Self {
            schedule,
            t: 0.0,
            cycles_completed: 0,
            nf_delivered: 0.0,
        }
    }

    pub fn in_pulse(&self) -> bool {
        let p = self.schedule.cycle_period.max(1e-9);
        let phase = self.t.rem_euclid(p) / p;
        phase < self.schedule.pulse_fraction.clamp(0.0, 1.0)
    }

    pub fn lean_intervals_completed(&self) -> u32 {
        // A lean interval completes at the end of each full cycle.
        self.cycles_completed
    }

    /// Apply one dt of seasonal supply into the spatial dish (N and F equally).
    pub fn supply_step(&mut self, dish: &mut SpatialDish, dt: f64) {
        let p = self.schedule.cycle_period.max(1e-9);
        let pulse_len = (self.schedule.pulse_fraction.clamp(0.0, 1.0) * p).max(1e-9);
        let before_cycle = (self.t / p).floor() as u32;
        let rate = if self.in_pulse() {
            self.schedule.cycle_nf_budget / pulse_len
        } else {
            self.schedule.lean_nf_rate.max(0.0)
        };
        let add = rate * dt.max(0.0);
        if add > 0.0 {
            let half = 0.5 * add;
            let cells = (dish.nx * dish.ny).max(1) as f64;
            let dn = half / cells;
            let df = half / cells;
            for v in dish.n.iter_mut() {
                *v += dn;
            }
            for v in dish.f.iter_mut() {
                *v += df;
            }
            self.nf_delivered += add;
        }
        self.t += dt.max(0.0);
        let after_cycle = (self.t / p).floor() as u32;
        if after_cycle > before_cycle {
            self.cycles_completed = after_cycle;
        }
    }
}

/// Candidate pulse periods as multiples of a maintenance horizon.
pub const PULSE_PERIOD_MULTS: [f64; 3] = [0.5, 1.0, 2.0];
