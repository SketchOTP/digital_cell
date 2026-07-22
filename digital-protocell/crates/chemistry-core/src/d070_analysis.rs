//! D-070 mature-membrane seed and capacity contract repair helpers.
//!
//! Observer/diagnostic only. Frozen P↔S exchange kinetics and production biology
//! are never modified here. Excess mature S must be rejected or explicitly migrated.

use crate::candidate_identity::sha256_hex;
use crate::config::DX;
use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d069_analysis::{
    desorption_explained_by_over_capacity, D069_FROZEN_KT, D069_GAMMA_MAX, D069_K_EQ,
    D069_K_EXCHANGE, D069_P_REF, EPS, LEDGER_TOL, S_RETENTION,
};
use crate::grid::Grid;
use crate::surface_density::{
    reconstruct_gamma, InterfaceGeometryCell, EXCHANGE_BOUND_TOLERANCE,
};
use serde::{Deserialize, Serialize};

pub const D070_PROJECT_ID: &str = "D-070";
pub const D070_AGENT_MEMORY_ID: &str =
    "D-20260722-0832-d070-mature-membrane-seed-capacity-contract-repair";
pub const D070_STARTING_COMMIT: &str = "a4f1c59";
pub const D070_STARTING_TAG: &str = "D-069-mature-membrane-exchange-audit";
pub const D069_CONCLUSION: &str = "D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT";
pub const D069_RECORD: &str = "D069_DESORPTION_EXPLAINED_BY_OVER_CAPACITY_SEED";

pub const D070_FROZEN_KT: f64 = D069_FROZEN_KT;
pub const D070_K_EXCHANGE: f64 = D069_K_EXCHANGE;
pub const D070_K_EQ: f64 = D069_K_EQ;
pub const D070_P_REF: f64 = D069_P_REF;
pub const D070_GAMMA_MAX: f64 = D069_GAMMA_MAX;

/// Canonical seed-capacity contract version.
pub const SEED_CAPACITY_CONTRACT_V1: &str = "SEED_CAPACITY_CONTRACT_V1";

pub const A_RETENTION: f64 = 0.80;
pub const C_RETENTION: f64 = 0.80;
pub const CHI_S_TARGET: f64 = 1.00;
/// Frozen D-008/D-039 Stage E occupancy floor used for boundary coverage.
pub const STAGE_E_MIN_OCCUPANCY: f64 = 0.50;
pub const NUMERIC_OCC_EPS: f64 = EXCHANGE_BOUND_TOLERANCE;
pub const CAPACITY_RATIO_TOL: f64 = 0.05;
pub const D069_S0_REF: f64 = 176.0;
pub const D069_CAPACITY_REF: f64 = 76.33335819109088;
pub const D069_OVER_CAPACITY_REF: f64 = 99.66664180890905;
pub const D069_DES_REF: f64 = 99.666;

pub const LOCAL_CAPACITY_EQ: &str = "S_max,i = δ_i · Γ_max";
pub const INTEGRATED_CAPACITY_EQ: &str = "M_S,max = Σ_i S_max,i · V_i  (V_i = DX²)";
pub const OCCUPANCY_EQ: &str = "θ_i = S_i / S_max,i = Γ_i / Γ_max";
pub const S_UNITS: &str = "S = δ·Γ  (Cartesian membrane mass density; mass = Σ S·V)";
pub const P_UNITS: &str = "P concentration; 1:1 exchange stoichiometry ΔP=−ξ, ΔS=+ξ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D070PrimaryConclusion {
    CapacityBoundedSeedAndExchangeQualified,
    SeedRepairQualifiesExchangePrecursorLimitRemains,
    CapacityValidSeedExchangeFailure,
    MembraneCapacityNormalizationDefect,
    MembraneSeedMaterialBudgetDefect,
    LawfulSeedMembraneCapacityInsufficient,
    WasteExecutionBlocksCapacityValidReplay,
    MembraneSeedCapacityRepairInconclusive,
    D069CapacityDefectNotReproduced,
    MembraneCapacityLineageOrUnitsFailure,
    SeedMaterialAuthorityUnresolved,
    SeedCapacityValidatorFailure,
    SeedMigrationConservationOrIdentityFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D070PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CapacityBoundedSeedAndExchangeQualified => {
                "D070_CAPACITY_BOUNDED_SEED_AND_EXCHANGE_QUALIFIED"
            }
            Self::SeedRepairQualifiesExchangePrecursorLimitRemains => {
                "D070_SEED_REPAIR_QUALIFIES_EXCHANGE_PRECURSOR_LIMIT_REMAINS"
            }
            Self::CapacityValidSeedExchangeFailure => "D070_CAPACITY_VALID_SEED_EXCHANGE_FAILURE",
            Self::MembraneCapacityNormalizationDefect => {
                "D070_MEMBRANE_CAPACITY_NORMALIZATION_DEFECT"
            }
            Self::MembraneSeedMaterialBudgetDefect => "D070_MEMBRANE_SEED_MATERIAL_BUDGET_DEFECT",
            Self::LawfulSeedMembraneCapacityInsufficient => {
                "D070_LAWFUL_SEED_MEMBRANE_CAPACITY_INSUFFICIENT"
            }
            Self::WasteExecutionBlocksCapacityValidReplay => {
                "D070_WASTE_EXECUTION_BLOCKS_CAPACITY_VALID_REPLAY"
            }
            Self::MembraneSeedCapacityRepairInconclusive => {
                "D070_MEMBRANE_SEED_CAPACITY_REPAIR_INCONCLUSIVE"
            }
            Self::D069CapacityDefectNotReproduced => "D070_D069_CAPACITY_DEFECT_NOT_REPRODUCED",
            Self::MembraneCapacityLineageOrUnitsFailure => {
                "D070_MEMBRANE_CAPACITY_LINEAGE_OR_UNITS_FAILURE"
            }
            Self::SeedMaterialAuthorityUnresolved => "D070_SEED_MATERIAL_AUTHORITY_UNRESOLVED",
            Self::SeedCapacityValidatorFailure => "D070_SEED_CAPACITY_VALIDATOR_FAILURE",
            Self::SeedMigrationConservationOrIdentityFailure => {
                "D070_SEED_MIGRATION_CONSERVATION_OR_IDENTITY_FAILURE"
            }
            Self::WorkspaceScopeNotIsolated => "D070_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D070_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D070_NUMERICAL_FAILURE",
            Self::Fail => "D070_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D070Route {
    X,
    S,
    P,
    E,
    M,
    B,
    W,
    I,
}

impl D070Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "Route_X_capacity_normalization_defect",
            Self::S => "Route_S_seed_allocation_repair_qualifies",
            Self::P => "Route_P_exchange_repaired_precursor_limit_remains",
            Self::E => "Route_E_capacity_valid_seed_still_loses_membrane",
            Self::M => "Route_M_seed_material_budget_invalid",
            Self::B => "Route_B_lawful_material_cannot_fill_required_membrane",
            Self::W => "Route_W_waste_execution_blocks_revalidation",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D070PrimaryConclusion {
        match self {
            Self::X => D070PrimaryConclusion::MembraneCapacityNormalizationDefect,
            Self::S => D070PrimaryConclusion::CapacityBoundedSeedAndExchangeQualified,
            Self::P => D070PrimaryConclusion::SeedRepairQualifiesExchangePrecursorLimitRemains,
            Self::E => D070PrimaryConclusion::CapacityValidSeedExchangeFailure,
            Self::M => D070PrimaryConclusion::MembraneSeedMaterialBudgetDefect,
            Self::B => D070PrimaryConclusion::LawfulSeedMembraneCapacityInsufficient,
            Self::W => D070PrimaryConclusion::WasteExecutionBlocksCapacityValidReplay,
            Self::I => D070PrimaryConclusion::MembraneSeedCapacityRepairInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeedClassification {
    CapacityValid,
    LocalAllocationOverCapacity,
    GlobalSOverCapacity,
    TotalMembraneMaterialUnauthorized,
    LegacySchemaUnknown,
    SeedAuthorityUnresolved,
}

impl SeedClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CapacityValid => "CAPACITY_VALID",
            Self::LocalAllocationOverCapacity => "LOCAL_ALLOCATION_OVER_CAPACITY",
            Self::GlobalSOverCapacity => "GLOBAL_S_OVER_CAPACITY",
            Self::TotalMembraneMaterialUnauthorized => "TOTAL_MEMBRANE_MATERIAL_UNAUTHORIZED",
            Self::LegacySchemaUnknown => "LEGACY_SCHEMA_UNKNOWN",
            Self::SeedAuthorityUnresolved => "SEED_AUTHORITY_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MigrationPolicy {
    StrictRejection,
    LocalExcessSToP,
    SupportCorrection,
    AuthorizedMaterialReconstruction,
    None,
}

impl MigrationPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StrictRejection => "POLICY_A_STRICT_REJECTION",
            Self::LocalExcessSToP => "POLICY_B_LOCAL_EXCESS_S_TO_P",
            Self::SupportCorrection => "POLICY_C_SUPPORT_CORRECTION",
            Self::AuthorizedMaterialReconstruction => "POLICY_D_AUTHORIZED_MATERIAL_RECONSTRUCTION",
            Self::None => "POLICY_NONE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbsoluteMembraneClass {
    RelativeAndAbsoluteMembraneSufficient,
    RetentionPassAbsoluteMembraneLow,
    CapacityFilledButReplacementFails,
    MembraneContractNotMet,
}

impl AbsoluteMembraneClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RelativeAndAbsoluteMembraneSufficient => {
                "RELATIVE_AND_ABSOLUTE_MEMBRANE_SUFFICIENT"
            }
            Self::RetentionPassAbsoluteMembraneLow => "RETENTION_PASS_ABSOLUTE_MEMBRANE_LOW",
            Self::CapacityFilledButReplacementFails => "CAPACITY_FILLED_BUT_REPLACEMENT_FAILS",
            Self::MembraneContractNotMet => "MEMBRANE_CONTRACT_NOT_MET",
        }
    }
}

/// Local mature-membrane capacity when S is stored as S=δ·Γ.
#[inline]
pub fn local_s_max(delta: f64, gamma_max: f64) -> f64 {
    delta.max(0.0) * gamma_max.max(0.0)
}

#[inline]
pub fn cell_volume() -> f64 {
    DX * DX
}

#[inline]
pub fn occupancy_theta(s: f64, delta: f64, gamma_max: f64) -> f64 {
    let cap = local_s_max(delta, gamma_max);
    if cap <= 0.0 {
        0.0
    } else {
        (s.max(0.0) / cap).max(0.0)
    }
}

#[inline]
pub fn membrane_material_equivalent(p_mass: f64, s_mass: f64) -> f64 {
    p_mass + s_mass
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapacityAudit {
    pub contract_version: String,
    pub s_mass: f64,
    pub p_mass: f64,
    pub membrane_equivalent: f64,
    pub capacity_mass: f64,
    pub max_occupancy: f64,
    pub over_capacity_cells: usize,
    pub over_capacity_mass: f64,
    pub support_cells: usize,
    pub s_outside_support_mass: f64,
    pub negative_p_cells: usize,
    pub negative_s_cells: usize,
    pub capacity_ratio: f64,
}

impl CapacityAudit {
    pub fn is_capacity_valid(&self) -> bool {
        self.over_capacity_cells == 0
            && self.over_capacity_mass <= LEDGER_TOL
            && self.s_outside_support_mass <= LEDGER_TOL
            && self.negative_p_cells == 0
            && self.negative_s_cells == 0
            && self.max_occupancy <= 1.0 + NUMERIC_OCC_EPS
            && self.s_mass.is_finite()
            && self.capacity_mass.is_finite()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeedCapacityValidation {
    pub valid: bool,
    pub fail_closed: bool,
    pub contract_version: String,
    pub audit: CapacityAudit,
    pub classification: SeedClassification,
    pub reason: String,
}

/// Shared capacity validator for seed construction, snapshot load, and diagnostics.
pub fn validate_seed_capacity(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &[f64],
    p: &[f64],
    delta_floor: f64,
    gamma_max: f64,
    material_authorized: Option<bool>,
) -> SeedCapacityValidation {
    let audit = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let classification = classify_seed(&audit, material_authorized);
    let valid = audit.is_capacity_valid()
        && !matches!(
            classification,
            SeedClassification::SeedAuthorityUnresolved
                | SeedClassification::LegacySchemaUnknown
        );
    let reason = if valid {
        "capacity_contract_satisfied".into()
    } else if matches!(classification, SeedClassification::SeedAuthorityUnresolved) {
        "seed_material_authority_unresolved".into()
    } else if audit.over_capacity_cells > 0 {
        format!(
            "over_capacity_cells={} mass={:.6}",
            audit.over_capacity_cells, audit.over_capacity_mass
        )
    } else {
        "capacity_contract_violation".into()
    };
    SeedCapacityValidation {
        valid,
        fail_closed: !valid,
        contract_version: SEED_CAPACITY_CONTRACT_V1.into(),
        audit,
        classification,
        reason,
    }
}

pub fn audit_capacity(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &[f64],
    p: &[f64],
    delta_floor: f64,
    gamma_max: f64,
) -> CapacityAudit {
    let v = cell_volume();
    let mut s_mass = 0.0;
    let mut p_mass = 0.0;
    let mut capacity_mass = 0.0;
    let mut max_occupancy = 0.0;
    let mut over_capacity_cells = 0usize;
    let mut over_capacity_mass = 0.0;
    let mut support_cells = 0usize;
    let mut s_outside_support_mass = 0.0;
    let mut negative_p_cells = 0usize;
    let mut negative_s_cells = 0usize;

    for i in 0..s.len() {
        if !grid.in_dish(i) {
            continue;
        }
        let si = s[i];
        let pi = p[i];
        if si < -LEDGER_TOL {
            negative_s_cells += 1;
        }
        if pi < -LEDGER_TOL {
            negative_p_cells += 1;
        }
        let s_pos = si.max(0.0);
        let p_pos = pi.max(0.0);
        s_mass += s_pos * v;
        p_mass += p_pos * v;
        let d = geometry[i].delta;
        if d > delta_floor {
            support_cells += 1;
            let cap = local_s_max(d, gamma_max);
            capacity_mass += cap * v;
            let theta = occupancy_theta(s_pos, d, gamma_max);
            if theta > max_occupancy {
                max_occupancy = theta;
            }
            if s_pos > cap + 1e-12 {
                over_capacity_cells += 1;
                over_capacity_mass += (s_pos - cap) * v;
            }
        } else if s_pos > LEDGER_TOL {
            s_outside_support_mass += s_pos * v;
            if s_pos > max_occupancy {
                // treat as infinite occupancy outside support for reporting
                max_occupancy = max_occupancy.max(f64::INFINITY);
            }
        }
    }

    CapacityAudit {
        contract_version: SEED_CAPACITY_CONTRACT_V1.into(),
        s_mass,
        p_mass,
        membrane_equivalent: membrane_material_equivalent(p_mass, s_mass),
        capacity_mass,
        max_occupancy: if max_occupancy.is_finite() {
            max_occupancy
        } else {
            f64::INFINITY
        },
        over_capacity_cells,
        over_capacity_mass,
        support_cells,
        s_outside_support_mass,
        negative_p_cells,
        negative_s_cells,
        capacity_ratio: s_mass / capacity_mass.max(EPS),
    }
}

pub fn classify_seed(audit: &CapacityAudit, material_authorized: Option<bool>) -> SeedClassification {
    if material_authorized == Some(false) {
        return SeedClassification::TotalMembraneMaterialUnauthorized;
    }
    if material_authorized.is_none()
        && (audit.over_capacity_cells > 0 || audit.capacity_ratio > 1.0 + CAPACITY_RATIO_TOL)
    {
        // Historical diagnostic seeds used face-length allocation without an explicit
        // membrane-material budget; authority cannot be claimed by silence.
        return SeedClassification::TotalMembraneMaterialUnauthorized;
    }
    if audit.s_outside_support_mass > LEDGER_TOL && audit.over_capacity_cells == 0 {
        return SeedClassification::LocalAllocationOverCapacity;
    }
    if audit.over_capacity_cells > 0 && audit.capacity_ratio > 1.0 + CAPACITY_RATIO_TOL {
        return SeedClassification::GlobalSOverCapacity;
    }
    if audit.over_capacity_cells > 0 {
        return SeedClassification::LocalAllocationOverCapacity;
    }
    if audit.is_capacity_valid() {
        SeedClassification::CapacityValid
    } else {
        SeedClassification::SeedAuthorityUnresolved
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationReport {
    pub policy: MigrationPolicy,
    pub contract_version: String,
    pub excess_s: f64,
    pub p_gained: f64,
    pub s_removed: f64,
    pub unauthorized_removed: f64,
    pub material_before: f64,
    pub material_after: f64,
    pub conserved: bool,
    pub idempotent_ready: bool,
    pub old_identity: String,
    pub new_identity: String,
    pub cells_touched: usize,
}

/// Deterministic content hash for seed identity.
pub fn seed_identity_hash(s: &[f64], p: &[f64], policy: MigrationPolicy, label: &str) -> String {
    let mut bytes = Vec::with_capacity(64 + 8 * (s.len() + p.len()));
    bytes.extend_from_slice(SEED_CAPACITY_CONTRACT_V1.as_bytes());
    bytes.extend_from_slice(policy.as_str().as_bytes());
    bytes.extend_from_slice(label.as_bytes());
    for &v in s {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    for &v in p {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    sha256_hex(&bytes)
}

/// Policy A: reject (no mutation). Returns Err-style report with conserved=false for apply path.
pub fn policy_a_reject(validation: &SeedCapacityValidation) -> Result<(), String> {
    if validation.valid {
        Ok(())
    } else {
        Err(format!(
            "POLICY_A_STRICT_REJECTION: {}",
            validation.reason
        ))
    }
}

/// Policy B: local excess S → P (authorized total material only).
pub fn migrate_policy_b_local_excess_s_to_p(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &mut [f64],
    p: &mut [f64],
    delta_floor: f64,
    gamma_max: f64,
    label: &str,
) -> MigrationReport {
    let before = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let old_id = seed_identity_hash(s, p, MigrationPolicy::LocalExcessSToP, label);
    let v = cell_volume();
    let mut excess_s = 0.0;
    let mut cells_touched = 0usize;
    for i in 0..s.len() {
        if !grid.in_dish(i) {
            continue;
        }
        let d = geometry[i].delta;
        if d <= delta_floor {
            continue;
        }
        let cap = local_s_max(d, gamma_max);
        let xi = (s[i] - cap).max(0.0);
        if xi > 1e-12 {
            s[i] = cap;
            p[i] += xi;
            excess_s += xi * v;
            cells_touched += 1;
        }
    }
    let after = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let new_id = seed_identity_hash(s, p, MigrationPolicy::LocalExcessSToP, label);
    MigrationReport {
        policy: MigrationPolicy::LocalExcessSToP,
        contract_version: SEED_CAPACITY_CONTRACT_V1.into(),
        excess_s,
        p_gained: excess_s,
        s_removed: excess_s,
        unauthorized_removed: 0.0,
        material_before: before.membrane_equivalent,
        material_after: after.membrane_equivalent,
        conserved: (before.membrane_equivalent - after.membrane_equivalent).abs() <= LEDGER_TOL,
        idempotent_ready: after.over_capacity_cells == 0,
        old_identity: old_id,
        new_identity: new_id,
        cells_touched,
    }
}

/// Policy C: S outside support → local P.
pub fn migrate_policy_c_support_correction(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &mut [f64],
    p: &mut [f64],
    delta_floor: f64,
    gamma_max: f64,
    label: &str,
) -> MigrationReport {
    let before = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let old_id = seed_identity_hash(s, p, MigrationPolicy::SupportCorrection, label);
    let v = cell_volume();
    let mut moved = 0.0;
    let mut cells_touched = 0usize;
    for i in 0..s.len() {
        if !grid.in_dish(i) {
            continue;
        }
        if geometry[i].delta > delta_floor {
            continue;
        }
        let xi = s[i].max(0.0);
        if xi > 0.0 {
            s[i] = 0.0;
            p[i] += xi;
            moved += xi * v;
            cells_touched += 1;
        }
    }
    // Also apply local capacity projection after support correction.
    let b = migrate_policy_b_local_excess_s_to_p(grid, geometry, s, p, delta_floor, gamma_max, label);
    let after = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let new_id = seed_identity_hash(s, p, MigrationPolicy::SupportCorrection, label);
    MigrationReport {
        policy: MigrationPolicy::SupportCorrection,
        contract_version: SEED_CAPACITY_CONTRACT_V1.into(),
        excess_s: moved + b.excess_s,
        p_gained: moved + b.p_gained,
        s_removed: moved + b.s_removed,
        unauthorized_removed: 0.0,
        material_before: before.membrane_equivalent,
        material_after: after.membrane_equivalent,
        conserved: (before.membrane_equivalent - after.membrane_equivalent).abs() <= LEDGER_TOL,
        idempotent_ready: after.over_capacity_cells == 0
            && after.s_outside_support_mass <= LEDGER_TOL,
        old_identity: old_id,
        new_identity: new_id,
        cells_touched: cells_touched + b.cells_touched,
    }
}

/// Policy D: reconstruct S from authorized capacity budget (θ_target on support).
/// Unauthorized excess S is removed (not converted to P) and reported separately.
pub fn migrate_policy_d_authorized_reconstruction(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &mut [f64],
    p: &[f64],
    delta_floor: f64,
    gamma_max: f64,
    theta_target: f64,
    label: &str,
) -> MigrationReport {
    let theta = theta_target.clamp(0.0, 1.0);
    let before = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let old_id = seed_identity_hash(s, p, MigrationPolicy::AuthorizedMaterialReconstruction, label);
    let v = cell_volume();
    let mut s_after_mass = 0.0;
    let mut cells_touched = 0usize;
    for i in 0..s.len() {
        if !grid.in_dish(i) {
            continue;
        }
        let d = geometry[i].delta;
        let next = if d > delta_floor {
            local_s_max(d, gamma_max) * theta
        } else {
            0.0
        };
        if (s[i] - next).abs() > 1e-15 {
            cells_touched += 1;
        }
        s[i] = next;
        s_after_mass += next * v;
    }
    let after = audit_capacity(grid, geometry, s, p, delta_floor, gamma_max);
    let unauthorized = (before.s_mass - s_after_mass).max(0.0);
    let new_id =
        seed_identity_hash(s, p, MigrationPolicy::AuthorizedMaterialReconstruction, label);
    MigrationReport {
        policy: MigrationPolicy::AuthorizedMaterialReconstruction,
        contract_version: SEED_CAPACITY_CONTRACT_V1.into(),
        excess_s: unauthorized,
        p_gained: 0.0,
        s_removed: unauthorized,
        unauthorized_removed: unauthorized,
        material_before: before.membrane_equivalent,
        material_after: after.membrane_equivalent,
        conserved: false, // intentional: unauthorized material is not preserved
        idempotent_ready: after.is_capacity_valid(),
        old_identity: old_id,
        new_identity: new_id,
        cells_touched,
    }
}

/// Capacity-bounded seed: S_i = δ_i · Γ_max · θ on interface support.
pub fn seed_capacity_bounded_s(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    delta_floor: f64,
    gamma_max: f64,
    theta: f64,
) -> Vec<f64> {
    let theta = theta.clamp(0.0, 1.0);
    let mut s = vec![0.0; geometry.len()];
    for i in 0..geometry.len() {
        if !grid.in_dish(i) {
            continue;
        }
        let d = geometry[i].delta;
        if d > delta_floor {
            s[i] = local_s_max(d, gamma_max) * theta;
        }
    }
    s
}

/// Precursor-only assembly control: S=0, preserve total membrane equivalent in P on support.
pub fn seed_precursor_only_from_material(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    delta_floor: f64,
    total_membrane_equivalent: f64,
    support_phi: &[f64],
    phi_interior: f64,
) -> (Vec<f64>, Vec<f64>) {
    let n = geometry.len();
    let s = vec![0.0; n];
    let mut p = vec![0.0; n];
    let v = cell_volume();
    let mut support = Vec::new();
    for i in 0..n {
        if grid.in_dish(i) && geometry[i].delta > delta_floor && support_phi[i] >= phi_interior {
            support.push(i);
        }
    }
    if support.is_empty() {
        // fall back to all interior cells
        for i in 0..n {
            if grid.in_dish(i) && support_phi[i] >= phi_interior {
                support.push(i);
            }
        }
    }
    if !support.is_empty() {
        let each = total_membrane_equivalent / (support.len() as f64 * v).max(EPS);
        for i in support {
            p[i] = each;
        }
    }
    (s, p)
}

pub fn d069_capacity_defect_reproduced(
    s0: f64,
    capacity: f64,
    over_capacity: f64,
    desorption: f64,
) -> bool {
    let ratio = s0 / capacity.max(EPS);
    (s0 - D069_S0_REF).abs() <= 1.0
        && (capacity - D069_CAPACITY_REF).abs() <= 1.0
        && (ratio - 2.306).abs() <= 0.05
        && (over_capacity - D069_OVER_CAPACITY_REF).abs() <= 1.0
        && desorption_explained_by_over_capacity(desorption, over_capacity, ratio)
}

pub fn capacity_scales_with_radius(cap_r: f64, r: f64, cap_r2: f64, r2: f64) -> bool {
    if r <= 0.0 || r2 <= 0.0 || cap_r <= 0.0 || cap_r2 <= 0.0 {
        return false;
    }
    let ratio = (cap_r2 / cap_r) / (r2 / r);
    (ratio - 1.0).abs() <= 0.25 // 2D smooth circle ≈ ∝ R; allow grid discreteness
}

pub fn capacity_independent_of_timestep(c1: f64, c2: f64) -> bool {
    (c1 - c2).abs() <= LEDGER_TOL * (1.0 + c1.abs())
}

pub fn relative_retention(s_t: f64, s0: f64) -> f64 {
    s_t / s0.max(EPS)
}

pub fn absolute_occupancy(s_t: f64, capacity_t: f64) -> f64 {
    s_t / capacity_t.max(EPS)
}

pub fn replacement_coverage(ads: f64, des: f64, damage: f64) -> f64 {
    let loss = des + damage;
    if loss <= EPS {
        if ads >= 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        ads / loss
    }
}

pub fn classify_absolute_membrane(
    s_ret: f64,
    abs_occ: f64,
    boundary_coverage: f64,
    chi_s: f64,
) -> AbsoluteMembraneClass {
    let abs_ok = abs_occ >= STAGE_E_MIN_OCCUPANCY && boundary_coverage >= STAGE_E_MIN_OCCUPANCY;
    let ret_ok = s_ret >= S_RETENTION;
    let repl_ok = chi_s >= CHI_S_TARGET || !chi_s.is_finite();
    if ret_ok && abs_ok && repl_ok {
        AbsoluteMembraneClass::RelativeAndAbsoluteMembraneSufficient
    } else if ret_ok && !abs_ok {
        AbsoluteMembraneClass::RetentionPassAbsoluteMembraneLow
    } else if abs_ok && !repl_ok {
        AbsoluteMembraneClass::CapacityFilledButReplacementFails
    } else {
        AbsoluteMembraneClass::MembraneContractNotMet
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence070 {
    pub workspace_isolated: bool,
    pub d069_reproduced: bool,
    pub lineage_ok: bool,
    pub capacity_normalization_ok: bool,
    pub seed_authority_resolved: bool,
    pub validator_ok: bool,
    pub migration_ok: bool,
    pub waste_blocks: bool,
    pub material_budget_invalid: bool,
    pub lawful_material_insufficient: bool,
    pub exchange_qualifies: bool,
    pub absolute_membrane_ok: bool,
    pub precursor_a_limit_remains: bool,
    pub capacity_valid_still_loses_s: bool,
}

pub fn select_route(ev: RouteEvidence070) -> (D070Route, D070PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D070Route::I,
            D070PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d069_reproduced {
        return (
            D070Route::I,
            D070PrimaryConclusion::D069CapacityDefectNotReproduced,
        );
    }
    if !ev.lineage_ok {
        return (
            D070Route::I,
            D070PrimaryConclusion::MembraneCapacityLineageOrUnitsFailure,
        );
    }
    if !ev.capacity_normalization_ok {
        return (D070Route::X, D070Route::X.conclusion());
    }
    if !ev.seed_authority_resolved {
        return (
            D070Route::I,
            D070PrimaryConclusion::SeedMaterialAuthorityUnresolved,
        );
    }
    if !ev.validator_ok {
        return (
            D070Route::I,
            D070PrimaryConclusion::SeedCapacityValidatorFailure,
        );
    }
    if !ev.migration_ok {
        return (
            D070Route::I,
            D070PrimaryConclusion::SeedMigrationConservationOrIdentityFailure,
        );
    }
    if ev.waste_blocks {
        return (D070Route::W, D070Route::W.conclusion());
    }
    if ev.material_budget_invalid {
        return (D070Route::M, D070Route::M.conclusion());
    }
    if ev.lawful_material_insufficient {
        return (D070Route::B, D070Route::B.conclusion());
    }
    if ev.capacity_valid_still_loses_s {
        return (D070Route::E, D070Route::E.conclusion());
    }
    if ev.exchange_qualifies && ev.absolute_membrane_ok && ev.precursor_a_limit_remains {
        return (D070Route::P, D070Route::P.conclusion());
    }
    if ev.exchange_qualifies && ev.absolute_membrane_ok {
        return (D070Route::S, D070Route::S.conclusion());
    }
    (D070Route::I, D070Route::I.conclusion())
}

pub fn frozen_params_table() -> Vec<(&'static str, f64)> {
    vec![
        ("k_exchange", D070_K_EXCHANGE),
        ("K_eq", D070_K_EQ),
        ("Gamma_max", D070_GAMMA_MAX),
        ("P_ref", D070_P_REF),
        ("k_T", D070_FROZEN_KT),
        ("alpha", D031_ALPHA_FROZEN),
        ("beta", D031_BETA_FROZEN),
    ]
}

pub fn gamma_from_s(s: f64, delta: f64, delta_floor: f64) -> f64 {
    reconstruct_gamma(s, delta, delta_floor)
}

pub fn migration_is_idempotent(report1: &MigrationReport, report2: &MigrationReport) -> bool {
    // Second apply must not move material; identities may differ only if label hashing
    // includes pre-state, so require stable post-state via zero excess/touch.
    report2.cells_touched == 0
        && report2.excess_s.abs() <= LEDGER_TOL
        && report1.idempotent_ready
        && report2.idempotent_ready
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn local_capacity_is_delta_gamma() {
        assert!((local_s_max(0.25, 1.0) - 0.25).abs() < 1e-15);
        assert!((occupancy_theta(0.125, 0.25, 1.0) - 0.5).abs() < 1e-15);
    }
}
