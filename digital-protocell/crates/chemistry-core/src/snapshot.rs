//! Versioned simulation state serialization.

use crate::candidate_identity::build_candidate_identity;
use crate::config::{EquationVersion, SimParams};
use crate::diagnostics::{CellDetector, TurnoverTotals, ViabilityClass};
use crate::fields::FieldBuffers;
use serde::{Deserialize, Serialize};

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldSchemaVersion {
    #[serde(rename = "five_field_v1")]
    FiveFieldV1,
    #[serde(rename = "seven_field_v1")]
    SevenFieldV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiveFieldPayload {
    pub structure: Vec<f64>,
    pub catalyst: Vec<f64>,
    pub nutrient: Vec<f64>,
    pub fuel: Vec<f64>,
    pub waste: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenFieldPayload {
    pub structure: Vec<f64>,
    pub catalyst: Vec<f64>,
    pub nutrient: Vec<f64>,
    pub fuel: Vec<f64>,
    pub waste: Vec<f64>,
    pub activated: Vec<f64>,
    pub membrane: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "payload_schema", content = "values")]
pub enum SnapshotFields {
    #[serde(rename = "five_field_v1")]
    FiveField(FiveFieldPayload),
    #[serde(rename = "seven_field_v1")]
    SevenField(SevenFieldPayload),
}

impl SnapshotFields {
    pub fn structure(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.structure,
            Self::SevenField(fields) => &fields.structure,
        }
    }

    pub fn catalyst(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.catalyst,
            Self::SevenField(fields) => &fields.catalyst,
        }
    }

    pub fn nutrient(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.nutrient,
            Self::SevenField(fields) => &fields.nutrient,
        }
    }

    pub fn fuel(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.fuel,
            Self::SevenField(fields) => &fields.fuel,
        }
    }

    pub fn waste(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.waste,
            Self::SevenField(fields) => &fields.waste,
        }
    }

    pub fn activated(&self) -> Option<&[f64]> {
        match self {
            Self::FiveField(_) => None,
            Self::SevenField(fields) => Some(&fields.activated),
        }
    }

    pub fn membrane(&self) -> Option<&[f64]> {
        match self {
            Self::FiveField(_) => None,
            Self::SevenField(fields) => Some(&fields.membrane),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSnapshot {
    pub snapshot_schema_version: u32,
    pub field_schema_version: FieldSchemaVersion,
    pub equation_version: EquationVersion,
    pub version: String,
    pub candidate_id: String,
    pub candidate_hash: String,
    pub configuration_hash: String,
    pub source_commit: String,
    pub random_seed: u64,
    pub substep: u64,
    pub sim_time: f64,
    pub params: SimParams,
    pub fields: SnapshotFields,
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
        let source_commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
        let identity = build_candidate_identity(
            params.clone(),
            source_commit,
            None,
            None,
            "snapshot",
            None,
            None,
        );
        let field_schema_version = match params.equation_version {
            EquationVersion::MembraneMetabolismV1 => FieldSchemaVersion::SevenFieldV1,
            EquationVersion::D001BulkV1
            | EquationVersion::D003CrowdingV1
            | EquationVersion::SurfaceTurnoverV1 => FieldSchemaVersion::FiveFieldV1,
        };
        let snapshot_fields = match field_schema_version {
            FieldSchemaVersion::FiveFieldV1 => SnapshotFields::FiveField(FiveFieldPayload {
                structure: fields.structure.clone(),
                catalyst: fields.catalyst.clone(),
                nutrient: fields.nutrient.clone(),
                fuel: fields.fuel.clone(),
                waste: fields.waste.clone(),
            }),
            FieldSchemaVersion::SevenFieldV1 => SnapshotFields::SevenField(SevenFieldPayload {
                structure: fields.structure.clone(),
                catalyst: fields.catalyst.clone(),
                nutrient: fields.nutrient.clone(),
                fuel: fields.fuel.clone(),
                waste: fields.waste.clone(),
                activated: fields.activated.clone(),
                membrane: fields.membrane.clone(),
            }),
        };
        Self {
            snapshot_schema_version: SNAPSHOT_SCHEMA_VERSION,
            field_schema_version,
            equation_version: params.equation_version,
            version: env!("CARGO_PKG_VERSION").to_string(),
            candidate_id: identity.candidate_id,
            candidate_hash: identity.candidate_hash,
            configuration_hash: identity.configuration_hash,
            source_commit: source_commit.to_string(),
            random_seed: params.random_seed,
            substep,
            sim_time,
            params: params.clone(),
            fields: snapshot_fields,
            classification: detector.last_classification,
            turnover: detector.turnover.clone(),
        }
    }

    pub fn restore_fields(&self, fields: &mut FieldBuffers) {
        fields.structure.copy_from_slice(self.fields.structure());
        fields.catalyst.copy_from_slice(self.fields.catalyst());
        fields.nutrient.copy_from_slice(self.fields.nutrient());
        fields.fuel.copy_from_slice(self.fields.fuel());
        fields.waste.copy_from_slice(self.fields.waste());
        match &self.fields {
            SnapshotFields::FiveField(_) => {
                fields.activated.fill(0.0);
                fields.membrane.fill(0.0);
            }
            SnapshotFields::SevenField(payload) => {
                fields.activated.copy_from_slice(&payload.activated);
                fields.membrane.copy_from_slice(&payload.membrane);
            }
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(s) {
            Ok(snapshot) => snapshot.validate().map(|()| snapshot).map_err(json_error),
            Err(new_schema_error) => match serde_json::from_str::<LegacyFieldSnapshot>(s) {
                Ok(snapshot) => Self::from_legacy(snapshot).validate_and_return(),
                Err(_) => Err(new_schema_error),
            },
        }
    }

    fn from_legacy(snapshot: LegacyFieldSnapshot) -> Self {
        let identity = build_candidate_identity(
            snapshot.params.clone(),
            "legacy-unknown",
            None,
            None,
            "legacy snapshot",
            None,
            None,
        );
        Self {
            snapshot_schema_version: 1,
            field_schema_version: FieldSchemaVersion::FiveFieldV1,
            equation_version: snapshot.params.equation_version,
            version: snapshot.version,
            candidate_id: identity.candidate_id,
            candidate_hash: identity.candidate_hash,
            configuration_hash: identity.configuration_hash,
            source_commit: "legacy-unknown".to_string(),
            random_seed: snapshot.random_seed,
            substep: snapshot.substep,
            sim_time: snapshot.sim_time,
            params: snapshot.params,
            fields: SnapshotFields::FiveField(FiveFieldPayload {
                structure: snapshot.structure,
                catalyst: snapshot.catalyst,
                nutrient: snapshot.nutrient,
                fuel: snapshot.fuel,
                waste: snapshot.waste,
            }),
            classification: snapshot.classification,
            turnover: snapshot.turnover,
        }
    }

    fn validate_and_return(self) -> Result<Self, serde_json::Error> {
        self.validate().map(|()| self).map_err(json_error)
    }

    fn validate(&self) -> Result<(), String> {
        if self.equation_version != self.params.equation_version {
            return Err("snapshot and parameter equation versions differ".to_string());
        }
        match (
            self.field_schema_version,
            &self.fields,
            self.equation_version,
        ) {
            (
                FieldSchemaVersion::FiveFieldV1,
                SnapshotFields::FiveField(_),
                EquationVersion::D001BulkV1
                | EquationVersion::D003CrowdingV1
                | EquationVersion::SurfaceTurnoverV1,
            )
            | (
                FieldSchemaVersion::SevenFieldV1,
                SnapshotFields::SevenField(_),
                EquationVersion::MembraneMetabolismV1,
            ) => Ok(()),
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV1) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v1".to_string())
            }
            (FieldSchemaVersion::SevenFieldV1, _, _) => {
                Err("seven_field_v1 snapshot requires membrane_metabolism_v1".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, SnapshotFields::SevenField(_), _) => {
                Err("five_field_v1 envelope contains seven_field_v1 payload".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct LegacyFieldSnapshot {
    version: String,
    random_seed: u64,
    substep: u64,
    sim_time: f64,
    params: SimParams,
    structure: Vec<f64>,
    catalyst: Vec<f64>,
    nutrient: Vec<f64>,
    fuel: Vec<f64>,
    waste: Vec<f64>,
    classification: ViabilityClass,
    turnover: TurnoverTotals,
}

fn json_error(message: String) -> serde_json::Error {
    <serde_json::Error as serde::de::Error>::custom(message)
}

pub fn save_snapshot(path: &std::path::Path, snap: &FieldSnapshot) -> std::io::Result<()> {
    let json = snap.to_json().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

pub fn load_snapshot(path: &std::path::Path) -> std::io::Result<FieldSnapshot> {
    let data = std::fs::read_to_string(path)?;
    FieldSnapshot::from_json(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
