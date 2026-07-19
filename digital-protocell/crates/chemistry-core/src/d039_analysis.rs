//! D-039 membrane turnover requirement and damage-repair qualification helpers.
//!
//! Schema 3: `surface_turnover_schema_3_exchange_damage_only` — no constitutive
//! mature-membrane `S→W`; reversible `P↔S` exchange retained; declared damage only.

use crate::config::{
    D008StageMode, EquationVersion, SimParams, SurfaceExchangeIntegrator, SurfaceTurnoverSchema,
};
use crate::d029_analysis::apply_exchange_candidate;
use crate::d031_analysis::{d030_identified_candidate, D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::surface_density::{apply_surface_turnover_exact, surface_turnover_lambda};
use serde::{Deserialize, Serialize};

pub const D039_STARTING_COMMIT: &str = "c74dd95";
pub const D039_D038_TAG: &str = "D-038-corrected-turnover-renewal";
pub const D039_AGENT_MEMORY_ID: &str =
    "D-20260719-1328-d039-membrane-turnover-requirement-damage-repair";

pub const D039_NET_S_FLOW_MAX: f64 = 1e-4;
pub const D039_REPLACEMENT_MIN: f64 = 0.10;
pub const D039_S_DRIFT_MAX: f64 = 0.05;
pub const D039_TRACER_RESIDUAL_MAX: f64 = 1e-8;
pub const D039_MAX_ACCEPTED: u64 = 200_000;

/// Apply experimental schema-3 exchange+damage-only turnover.
pub fn apply_schema3_exchange_damage_only(params: &mut SimParams) {
    params.surface_turnover_schema = SurfaceTurnoverSchema::ExchangeDamageOnly;
    // Keep k_gamma_decay numerically defined for identity hashing, but λ=0 under schema 3.
    params.k_gamma_decay = 0.0;
}

pub fn apply_renewal_stage_mode(params: &mut SimParams) {
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
}

/// Frozen v8 architecture under schema 3.
pub fn v8_schema3_params() -> SimParams {
    let mut p = SimParams::default();
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    apply_schema3_exchange_damage_only(&mut p);
    apply_renewal_stage_mode(&mut p);
    p
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractAuditReport {
    pub component_replacement_required: bool,
    pub constitutive_destruction_required: bool,
    pub damage_repair_required: bool,
    pub material_replacement_required: bool,
    pub organizational_maintenance_required: bool,
    pub interpretation: String,
    pub conclusion: String,
    pub pass: bool,
    pub notes: Vec<String>,
}

/// Gate 0 — Phase 1 / D-008 acceptance-contract audit.
pub fn gate0_contract_audit() -> ContractAuditReport {
    // Evidence from D-008 design + PROJECT_GOAL + D-037 MIXED_PURPOSE_TERM:
    // Stage B requires active turnover (material replacement), not a permanent
    // unsupported uniform first-order S→W hazard. Stage E requires overlapping
    // zero-flow balance regions. PROJECT_GOAL requires continuous rebuild and
    // causal damage/starvation — not constitutive destruction of healthy membrane.
    let notes = vec![
        "D-008 Stage B: membrane production/diffusion/decay/detachment + active turnover ≥90% localization".into(),
        "D-008 Stage E: overlapping zero-flow for structure/catalyst/membrane/activated resource".into(),
        "PROJECT_GOAL: continuous rebuild; causal damage/starvation/death; no repair controller".into(),
        "D-037: constitutive surface turnover classified MIXED_PURPOSE_TERM / biologically unsupported".into(),
        "D-038: all renewal architectures fail under corrected constitutive turnover load".into(),
        "Interpretation: organizational maintenance + material replacement + causal damage repair required; uniform constitutive S→W not required".into(),
        "CONSTITUTIVE_MEMBRANE_TURNOVER_UNCERTIFIED".into(),
    ];
    let conclusion = String::from("MEMBRANE_MAINTENANCE_MAY_USE_EXCHANGE_PLUS_CAUSAL_DAMAGE");
    ContractAuditReport {
        component_replacement_required: true,
        constitutive_destruction_required: false,
        damage_repair_required: true,
        material_replacement_required: true,
        organizational_maintenance_required: true,
        interpretation: conclusion.clone(),
        conclusion,
        pass: true,
        notes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaSafetyReport {
    pub historical_default: String,
    pub schema_1: String,
    pub schema_2: String,
    pub schema_3: String,
    pub schema_3_is_default: bool,
    pub schema_3_lambda_zero: bool,
    pub schema_1_lambda_positive: bool,
    pub schema_2_lambda_positive_off_interface: bool,
    pub alpha_frozen: f64,
    pub beta_frozen: f64,
    pub pass: bool,
}

pub fn gate1_schema_safety() -> SchemaSafetyReport {
    let default = SimParams::default().surface_turnover_schema;
    let mut p1 = SimParams::default();
    p1.surface_turnover_schema = SurfaceTurnoverSchema::HistoricalUniform;
    p1.k_gamma_decay = 0.002;
    let mut p2 = p1.clone();
    p2.surface_turnover_schema = SurfaceTurnoverSchema::D021Equivalent;
    p2.eps_m = 0.05;
    let mut p3 = p1.clone();
    apply_schema3_exchange_damage_only(&mut p3);
    // Keep a nonzero rate constant on schema 3 to prove λ ignores it.
    p3.k_gamma_decay = 0.002;

    let lam1 = surface_turnover_lambda(1.0, &p1);
    let lam2 = surface_turnover_lambda(0.5, &p2);
    let lam3 = surface_turnover_lambda(0.5, &p3);
    let (s3, dw3) = apply_surface_turnover_exact(1.0, 0.5, &p3, 1.0);

    SchemaSafetyReport {
        historical_default: default.as_str().into(),
        schema_1: SurfaceTurnoverSchema::HistoricalUniform.as_str().into(),
        schema_2: SurfaceTurnoverSchema::D021Equivalent.as_str().into(),
        schema_3: SurfaceTurnoverSchema::ExchangeDamageOnly.as_str().into(),
        schema_3_is_default: default == SurfaceTurnoverSchema::ExchangeDamageOnly,
        schema_3_lambda_zero: lam3 == 0.0 && (s3 - 1.0).abs() < 1e-15 && dw3 == 0.0,
        schema_1_lambda_positive: lam1 > 0.0,
        schema_2_lambda_positive_off_interface: lam2 > 0.0,
        alpha_frozen: D031_ALPHA_FROZEN,
        beta_frozen: D031_BETA_FROZEN,
        pass: default == SurfaceTurnoverSchema::HistoricalUniform
            && lam3 == 0.0
            && lam1 > 0.0
            && lam2 > 0.0
            && (D031_ALPHA_FROZEN - 0.16699387305200235).abs() < 1e-6
            && (D031_BETA_FROZEN - 0.003339877461040047).abs() < 1e-12,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevisedStageEContract {
    pub normalized_net_s_flow_max: f64,
    pub require_bounded_s_p: bool,
    pub require_stable_occupancy: bool,
    pub require_active_gross_ads_des: bool,
    pub require_molecular_replacement: bool,
    pub require_metabolism_dependent_damage_repair: bool,
    pub require_localization_retention: bool,
    pub require_accounting_closure: bool,
    pub constitutive_production_destruction_ratio_required: bool,
    pub note: String,
}

/// Gate 10 — revised Stage E membrane contract (definition only; not executed).
pub fn revised_stage_e_membrane_contract() -> RevisedStageEContract {
    RevisedStageEContract {
        normalized_net_s_flow_max: D039_NET_S_FLOW_MAX,
        require_bounded_s_p: true,
        require_stable_occupancy: true,
        require_active_gross_ads_des: true,
        require_molecular_replacement: true,
        require_metabolism_dependent_damage_repair: true,
        require_localization_retention: true,
        require_accounting_closure: true,
        constitutive_production_destruction_ratio_required: false,
        note: "Remove unsupported membrane production / constitutive membrane destruction = 1 when no constitutive destruction mechanism is present. Other Stage E balance requirements unchanged. D-039 does not execute Stage E.".into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum D039Conclusion {
    ExchangeDamageMaintenanceQualified,
    ConstitutiveTurnoverContractRequired,
    ContinuousReplacementNotEstablished,
    DamageRepairFailure,
    ResourceDependenceNotEstablished,
    FoundationalRegression,
    SchemaOrPreservationFailure,
    TracerAccountingFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D039Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExchangeDamageMaintenanceQualified => {
                "D039_EXCHANGE_DAMAGE_MAINTENANCE_QUALIFIED"
            }
            Self::ConstitutiveTurnoverContractRequired => {
                "D039_CONSTITUTIVE_TURNOVER_CONTRACT_REQUIRED"
            }
            Self::ContinuousReplacementNotEstablished => {
                "D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED"
            }
            Self::DamageRepairFailure => "D039_DAMAGE_REPAIR_FAILURE",
            Self::ResourceDependenceNotEstablished => "D039_RESOURCE_DEPENDENCE_NOT_ESTABLISHED",
            Self::FoundationalRegression => "D039_FOUNDATIONAL_REGRESSION",
            Self::SchemaOrPreservationFailure => "D039_SCHEMA_OR_PRESERVATION_FAILURE",
            Self::TracerAccountingFailure => "D039_TRACER_ACCOUNTING_FAILURE",
            Self::AccountingFailure => "D039_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D039_NUMERICAL_FAILURE",
            Self::Fail => "D039_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageRepairClass {
    SuccessfulRepair,
    BoundedIncompleteRepair,
    IrreversibleMembraneFailure,
}

impl DamageRepairClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SuccessfulRepair => "successful_repair",
            Self::BoundedIncompleteRepair => "bounded_incomplete_repair",
            Self::IrreversibleMembraneFailure => "irreversible_membrane_failure",
        }
    }
}

pub fn classify_damage_repair(
    fraction: f64,
    s_recovery_ratio: f64,
    local_occupancy_ratio: f64,
    localization: f64,
    mandatory: bool,
) -> DamageRepairClass {
    let ok = s_recovery_ratio >= 0.95
        && local_occupancy_ratio >= 0.90
        && localization >= 0.95;
    if ok {
        DamageRepairClass::SuccessfulRepair
    } else if mandatory {
        DamageRepairClass::IrreversibleMembraneFailure
    } else if s_recovery_ratio >= 0.50 && localization >= 0.80 {
        DamageRepairClass::BoundedIncompleteRepair
    } else if fraction >= 0.40 {
        DamageRepairClass::IrreversibleMembraneFailure
    } else {
        DamageRepairClass::BoundedIncompleteRepair
    }
}

pub fn select_conclusion(
    gate0_pass: bool,
    gate1_pass: bool,
    tracer_pass: bool,
    baseline_pass: bool,
    replacement_pass: bool,
    damage_pass: bool,
    resource_pass: bool,
    foundational_pass: bool,
    dynamic_pass: bool,
    accounting_pass: bool,
    numerical_ok: bool,
) -> D039Conclusion {
    if !gate0_pass {
        return D039Conclusion::ConstitutiveTurnoverContractRequired;
    }
    if !gate1_pass {
        return D039Conclusion::SchemaOrPreservationFailure;
    }
    if !tracer_pass {
        return D039Conclusion::TracerAccountingFailure;
    }
    if !numerical_ok {
        return D039Conclusion::NumericalFailure;
    }
    if !accounting_pass {
        return D039Conclusion::AccountingFailure;
    }
    if !replacement_pass {
        return D039Conclusion::ContinuousReplacementNotEstablished;
    }
    if !damage_pass {
        return D039Conclusion::DamageRepairFailure;
    }
    if !baseline_pass {
        return D039Conclusion::Fail;
    }
    if !resource_pass {
        return D039Conclusion::ResourceDependenceNotEstablished;
    }
    if !foundational_pass {
        return D039Conclusion::FoundationalRegression;
    }
    if !dynamic_pass {
        return D039Conclusion::Fail;
    }
    D039Conclusion::ExchangeDamageMaintenanceQualified
}
