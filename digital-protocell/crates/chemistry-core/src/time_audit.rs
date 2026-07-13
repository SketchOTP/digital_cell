//! Simulated-time and adaptive-dt telemetry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DtTelemetry {
    pub accepted_simulated_time: f64,
    pub accepted_dts: Vec<f64>,
    pub timestep_reductions: u64,
    pub timestep_recoveries: u64,
    pub max_dt_used: f64,
}

impl DtTelemetry {
    pub fn record_accept(&mut self, dt: f64) {
        self.accepted_simulated_time += dt;
        self.accepted_dts.push(dt);
        self.max_dt_used = self.max_dt_used.max(dt);
    }

    pub fn record_reduction(&mut self) {
        self.timestep_reductions += 1;
    }

    pub fn record_recovery(&mut self, prev: f64, next: f64) {
        if next > prev * 1.01 {
            self.timestep_recoveries += 1;
        }
    }

    pub fn mean_dt(&self) -> f64 {
        if self.accepted_dts.is_empty() {
            return 0.0;
        }
        self.accepted_dts.iter().sum::<f64>() / self.accepted_dts.len() as f64
    }

    pub fn median_dt(&self) -> f64 {
        percentile(&self.accepted_dts, 50.0)
    }

    pub fn min_dt(&self) -> f64 {
        self.accepted_dts.iter().copied().fold(f64::INFINITY, f64::min)
    }

    pub fn max_dt(&self) -> f64 {
        self.accepted_dts.iter().copied().fold(0.0, f64::max)
    }

    pub fn std_dt(&self) -> f64 {
        if self.accepted_dts.len() < 2 {
            return 0.0;
        }
        let mean = self.mean_dt();
        let var = self
            .accepted_dts
            .iter()
            .map(|d| (d - mean).powi(2))
            .sum::<f64>()
            / self.accepted_dts.len() as f64;
        var.sqrt()
    }

    pub fn percentile_dt(&self, p: f64) -> f64 {
        percentile(&self.accepted_dts, p)
    }

    pub fn summary(&self) -> DtSummary {
        DtSummary {
            accepted_simulated_time: self.accepted_simulated_time,
            mean_dt: self.mean_dt(),
            median_dt: self.median_dt(),
            min_dt: if self.accepted_dts.is_empty() {
                0.0
            } else {
                self.min_dt()
            },
            max_dt: self.max_dt(),
            std_dt: self.std_dt(),
            p01: self.percentile_dt(1.0),
            p05: self.percentile_dt(5.0),
            p50: self.percentile_dt(50.0),
            p95: self.percentile_dt(95.0),
            p99: self.percentile_dt(99.0),
            timestep_reductions: self.timestep_reductions,
            timestep_recoveries: self.timestep_recoveries,
            accepted_substeps: self.accepted_dts.len() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtSummary {
    pub accepted_simulated_time: f64,
    pub mean_dt: f64,
    pub median_dt: f64,
    pub min_dt: f64,
    pub max_dt: f64,
    pub std_dt: f64,
    pub p01: f64,
    pub p05: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub timestep_reductions: u64,
    pub timestep_recoveries: u64,
    pub accepted_substeps: u64,
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}
