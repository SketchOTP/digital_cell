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
    #[serde(rename = "eight_field_v1")]
    EightFieldV1,
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
pub struct EightFieldPayload {
    pub structure: Vec<f64>,
    pub catalyst: Vec<f64>,
    pub nutrient: Vec<f64>,
    pub fuel: Vec<f64>,
    pub waste: Vec<f64>,
    pub activated: Vec<f64>,
    pub membrane: Vec<f64>,
    pub precursor: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "payload_schema", content = "values")]
pub enum SnapshotFields {
    #[serde(rename = "five_field_v1")]
    FiveField(FiveFieldPayload),
    #[serde(rename = "seven_field_v1")]
    SevenField(SevenFieldPayload),
    #[serde(rename = "eight_field_v1")]
    EightField(EightFieldPayload),
}

impl SnapshotFields {
    pub fn structure(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.structure,
            Self::SevenField(fields) => &fields.structure,
            Self::EightField(fields) => &fields.structure,
        }
    }

    pub fn catalyst(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.catalyst,
            Self::SevenField(fields) => &fields.catalyst,
            Self::EightField(fields) => &fields.catalyst,
        }
    }

    pub fn nutrient(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.nutrient,
            Self::SevenField(fields) => &fields.nutrient,
            Self::EightField(fields) => &fields.nutrient,
        }
    }

    pub fn fuel(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.fuel,
            Self::SevenField(fields) => &fields.fuel,
            Self::EightField(fields) => &fields.fuel,
        }
    }

    pub fn waste(&self) -> &[f64] {
        match self {
            Self::FiveField(fields) => &fields.waste,
            Self::SevenField(fields) => &fields.waste,
            Self::EightField(fields) => &fields.waste,
        }
    }

    pub fn activated(&self) -> Option<&[f64]> {
        match self {
            Self::FiveField(_) => None,
            Self::SevenField(fields) => Some(&fields.activated),
            Self::EightField(fields) => Some(&fields.activated),
        }
    }

    pub fn membrane(&self) -> Option<&[f64]> {
        match self {
            Self::FiveField(_) => None,
            Self::SevenField(fields) => Some(&fields.membrane),
            Self::EightField(fields) => Some(&fields.membrane),
        }
    }

    pub fn precursor(&self) -> Option<&[f64]> {
        match self {
            Self::FiveField(_) | Self::SevenField(_) => None,
            Self::EightField(fields) => Some(&fields.precursor),
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
    pub fn stoichiometric_schema_version(&self) -> u32 {
        self.equation_version.stoichiometric_schema_version()
    }

    /// V1 snapshots may be inspected but must not initialize governed v2 runs.
    pub fn can_resume_into(&self, target: &SimParams) -> Result<(), String> {
        let snap_schema = self.stoichiometric_schema_version();
        let target_schema = target.equation_version.stoichiometric_schema_version();
        if snap_schema != target_schema {
            return Err(format!(
                "snapshot stoichiometric_schema_version {snap_schema} incompatible with target {target_schema}; v1 snapshots cannot resume v2 runs"
            ));
        }
        if self.equation_version != target.equation_version {
            return Err(format!(
                "snapshot equation_version {} incompatible with target {}",
                self.equation_version, target.equation_version
            ));
        }
        Ok(())
    }

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
            EquationVersion::MembraneMetabolismV6PrecursorAssembly => {
                FieldSchemaVersion::EightFieldV1
            }
            EquationVersion::MembraneMetabolismV1
            | EquationVersion::MembraneMetabolismV2Conservative | EquationVersion::MembraneMetabolismV3StructuralScaling | EquationVersion::MembraneMetabolismV4InterfaceProtected | EquationVersion::MembraneMetabolismV5InterfaceAffinity => FieldSchemaVersion::SevenFieldV1,
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
            FieldSchemaVersion::EightFieldV1 => SnapshotFields::EightField(EightFieldPayload {
                structure: fields.structure.clone(),
                catalyst: fields.catalyst.clone(),
                nutrient: fields.nutrient.clone(),
                fuel: fields.fuel.clone(),
                waste: fields.waste.clone(),
                activated: fields.activated.clone(),
                membrane: fields.membrane.clone(),
                precursor: fields.precursor.clone(),
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

    /// Compatibility wrapper. Prefer [`Self::try_restore_fields`] at trust boundaries.
    pub fn restore_fields(&self, fields: &mut FieldBuffers) {
        self.try_restore_fields(fields)
            .expect("snapshot restore failed schema or length validation");
    }

    pub fn try_restore_fields(&self, fields: &mut FieldBuffers) -> Result<(), String> {
        self.validate()?;
        let expected = fields.structure.len();
        validate_destination_lengths(fields, expected)?;
        match &self.fields {
            SnapshotFields::FiveField(payload) => {
                validate_equal_lengths(
                    expected,
                    &[
                        ("structure", payload.structure.len()),
                        ("catalyst", payload.catalyst.len()),
                        ("nutrient", payload.nutrient.len()),
                        ("fuel", payload.fuel.len()),
                        ("waste", payload.waste.len()),
                    ],
                )?;
                fields.structure.copy_from_slice(&payload.structure);
                fields.catalyst.copy_from_slice(&payload.catalyst);
                fields.nutrient.copy_from_slice(&payload.nutrient);
                fields.fuel.copy_from_slice(&payload.fuel);
                fields.waste.copy_from_slice(&payload.waste);
                // Legacy five-field payloads do not carry A/M/P; clear only after acceptance.
                fields.activated.fill(0.0);
                fields.membrane.fill(0.0);
                fields.precursor.fill(0.0);
            }
            SnapshotFields::SevenField(payload) => {
                validate_equal_lengths(
                    expected,
                    &[
                        ("structure", payload.structure.len()),
                        ("catalyst", payload.catalyst.len()),
                        ("nutrient", payload.nutrient.len()),
                        ("fuel", payload.fuel.len()),
                        ("waste", payload.waste.len()),
                        ("activated", payload.activated.len()),
                        ("membrane", payload.membrane.len()),
                    ],
                )?;
                fields.structure.copy_from_slice(&payload.structure);
                fields.catalyst.copy_from_slice(&payload.catalyst);
                fields.nutrient.copy_from_slice(&payload.nutrient);
                fields.fuel.copy_from_slice(&payload.fuel);
                fields.waste.copy_from_slice(&payload.waste);
                fields.activated.copy_from_slice(&payload.activated);
                fields.membrane.copy_from_slice(&payload.membrane);
                // Seven-field payloads do not carry P; clear it.
                fields.precursor.fill(0.0);
            }
            SnapshotFields::EightField(payload) => {
                validate_equal_lengths(
                    expected,
                    &[
                        ("structure", payload.structure.len()),
                        ("catalyst", payload.catalyst.len()),
                        ("nutrient", payload.nutrient.len()),
                        ("fuel", payload.fuel.len()),
                        ("waste", payload.waste.len()),
                        ("activated", payload.activated.len()),
                        ("membrane", payload.membrane.len()),
                        ("precursor", payload.precursor.len()),
                    ],
                )?;
                fields.structure.copy_from_slice(&payload.structure);
                fields.catalyst.copy_from_slice(&payload.catalyst);
                fields.nutrient.copy_from_slice(&payload.nutrient);
                fields.fuel.copy_from_slice(&payload.fuel);
                fields.waste.copy_from_slice(&payload.waste);
                fields.activated.copy_from_slice(&payload.activated);
                fields.membrane.copy_from_slice(&payload.membrane);
                fields.precursor.copy_from_slice(&payload.precursor);
            }
        }
        Ok(())
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

    pub fn validate(&self) -> Result<(), String> {
        match self.snapshot_schema_version {
            1 | SNAPSHOT_SCHEMA_VERSION => {}
            other => {
                return Err(format!(
                    "unsupported snapshot_schema_version {other}; known versions are 1 and {SNAPSHOT_SCHEMA_VERSION}"
                ));
            }
        }
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
            |             (
                FieldSchemaVersion::SevenFieldV1,
                SnapshotFields::SevenField(_),
                EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative | EquationVersion::MembraneMetabolismV3StructuralScaling | EquationVersion::MembraneMetabolismV4InterfaceProtected | EquationVersion::MembraneMetabolismV5InterfaceAffinity,
            )
            | (
                FieldSchemaVersion::EightFieldV1,
                SnapshotFields::EightField(_),
                EquationVersion::MembraneMetabolismV6PrecursorAssembly,
            ) => Ok(()),
            // Eight-field payload requires v6; seven-field snapshots cannot resume as v6.
            (FieldSchemaVersion::EightFieldV1, _, v) if !v.is_precursor_assembly() => Err(format!(
                "eight_field_v1 snapshot requires membrane_metabolism_v6_precursor_assembly, got {v}"
            )),
            (_, SnapshotFields::EightField(_), v) if !v.is_precursor_assembly() => Err(format!(
                "eight_field_v1 payload is incompatible with {v}"
            )),
            (FieldSchemaVersion::SevenFieldV1, _, EquationVersion::MembraneMetabolismV6PrecursorAssembly) => {
                Err("seven_field_v1 snapshot cannot resume as membrane_metabolism_v6_precursor_assembly".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV6PrecursorAssembly) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v6_precursor_assembly".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV1) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v1".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV2Conservative) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v2_conservative".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV3StructuralScaling) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v3_structural_scaling".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV4InterfaceProtected) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v4_interface_protected".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, _, EquationVersion::MembraneMetabolismV5InterfaceAffinity) => {
                Err("five_field_v1 snapshot is incompatible with membrane_metabolism_v5_interface_affinity".to_string())
            }
            (FieldSchemaVersion::SevenFieldV1, _, EquationVersion::D001BulkV1
            | EquationVersion::D003CrowdingV1
            | EquationVersion::SurfaceTurnoverV1) => {
                Err("seven_field_v1 snapshot requires membrane metabolism equation".to_string())
            }
            (FieldSchemaVersion::FiveFieldV1, SnapshotFields::SevenField(_), _) => {
                Err("five_field_v1 envelope contains seven_field_v1 payload".to_string())
            }
            (FieldSchemaVersion::SevenFieldV1, SnapshotFields::FiveField(_), _) => {
                Err("seven_field_v1 envelope contains five_field_v1 payload".to_string())
            }
            (field_schema, _, equation) => Err(format!(
                "snapshot field schema {field_schema:?} incompatible with equation {equation}"
            )),
        }
    }
}

fn validate_destination_lengths(fields: &FieldBuffers, expected: usize) -> Result<(), String> {
    validate_equal_lengths(
        expected,
        &[
            ("structure", fields.structure.len()),
            ("catalyst", fields.catalyst.len()),
            ("nutrient", fields.nutrient.len()),
            ("fuel", fields.fuel.len()),
            ("waste", fields.waste.len()),
            ("activated", fields.activated.len()),
            ("membrane", fields.membrane.len()),
            ("precursor", fields.precursor.len()),
        ],
    )
}

fn validate_equal_lengths(expected: usize, lengths: &[(&str, usize)]) -> Result<(), String> {
    for (name, len) in lengths {
        if *len != expected {
            return Err(format!(
                "snapshot field length mismatch for {name}: got {len}, expected {expected}"
            ));
        }
    }
    Ok(())
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
