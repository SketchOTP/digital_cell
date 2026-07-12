//! Simulation state serialization.

use crate::config::SimParams;
use crate::diagnostics::{CellDetector, DiagnosticsSnapshot, TurnoverTotals, ViabilityClass};
use crate::fields::FieldBuffers;
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSnapshot {
    pub version: String,
    pub random_seed: u64,
    pub substep: u64,
    pub sim_time: f64,
    pub params: SimParams,
    pub structure: Vec<f64>,
    pub catalyst: Vec<f64>,
    pub nutrient: Vec<f64>,
    pub fuel: Vec<f64>,
    pub waste: Vec<f64>,
    pub classification: ViabilityClass,
    pub turnover: TurnoverTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub grid: GridSnapshot,
    pub fields: FieldSnapshot,
    pub detector: DetectorSnapshot,
    pub dt: f64,
    pub min_dt_seen: f64,
    pub rejection_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSnapshot {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorSnapshot {
    pub extinction_counter: u64,
    pub last_classification: ViabilityClass,
    pub turnover: TurnoverTotals,
}

impl FieldSnapshot {
    pub fn from_sim(
        fields: &FieldBuffers,
        params: &SimParams,
        substep: u64,
        sim_time: f64,
        detector: &CellDetector,
    ) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            random_seed: params.random_seed,
            substep,
            sim_time,
            params: params.clone(),
            structure: fields.structure.clone(),
            catalyst: fields.catalyst.clone(),
            nutrient: fields.nutrient.clone(),
            fuel: fields.fuel.clone(),
            waste: fields.waste.clone(),
            classification: detector.last_classification,
            turnover: detector.turnover.clone(),
        }
    }

    pub fn restore_fields(&self, fields: &mut FieldBuffers) {
        fields.structure.copy_from_slice(&self.structure);
        fields.catalyst.copy_from_slice(&self.catalyst);
        fields.nutrient.copy_from_slice(&self.nutrient);
        fields.fuel.copy_from_slice(&self.fuel);
        fields.waste.copy_from_slice(&self.waste);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

pub fn save_snapshot(path: &std::path::Path, snap: &FieldSnapshot) -> std::io::Result<()> {
    let json = snap.to_json().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

pub fn load_snapshot(path: &std::path::Path) -> std::io::Result<FieldSnapshot> {
    let data = std::fs::read_to_string(path)?;
    FieldSnapshot::from_json(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
