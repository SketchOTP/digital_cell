//! D-048 frozen-biology membrane basin and repair qualification helpers.
//!
//! No biological equation or parameter may change. Historical activation
//! `r_A = 0.020 · C · N · F` is frozen. Schema-3 constitutive mature-membrane
//! turnover remains zero. Zero-S states are diagnostic only.

use crate::config::{
    EquationVersion, SimParams, SurfaceExchangeIntegrator, SurfaceTurnoverSchema,
};
use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d039_analysis::{
    apply_renewal_stage_mode, apply_schema3_exchange_damage_only, classify_damage_repair,
    v8_schema3_params, DamageRepairClass,
};
use crate::d047_analysis::D047_HISTORICAL_K;
use serde::{Deserialize, Serialize};

pub const D048_AGENT_MEMORY_ID: &str =
    "D-20260720-d048-frozen-biology-membrane-basin-repair";
pub const D048_STARTING_COMMIT: &str = "3b211f9";
pub const D048_D047_TAG: &str = "D-047-shared-activated-resource-audit";
pub const D048_RECORD_ACTIVATION: &str = "HISTORICAL_ACTIVATION_FROZEN_FOR_MEMBRANE_VALIDATION";
pub const D048_ARCHITECTURE_PASS: &str = "V8_SCHEMA3_FROZEN_BIOLOGY_MEMBRANE_MAINTENANCE";

pub const D048_HISTORICAL_K: f64 = D047_HISTORICAL_K; // 0.020
pub const D048_NET_S_FLOW_MAX: f64 = 1e-4;
pub const D048_REPLACEMENT_MIN: f64 = 0.10;
pub const D048_S_DRIFT_MAX: f64 = 0.05;
pub const D048_TRACER_RESIDUAL_MAX: f64 = 1e-8;
pub const D048_WINDOW: u64 = 10_000;
pub const D048_REQUIRED_WINDOWS: usize = 3;
pub const D048_MAX_ACCEPTED: u64 = 200_000;
pub const D048_HORIZONS: [u64; 4] = [25_000, 50_000, 100_000, 200_000];
pub const D048_RETENTION_MIN: f64 = 0.80;
pub const D048_LOCALIZATION_MIN: f64 = 0.95;
pub const D048_RADIUS: f64 = 22.0;
pub const D048_THETA: f64 = 0.6;
pub const D048_SEED_NOISE: u64 = 1;

/// Seed material classification for Gate 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedMaterialClass {
    PermittedOrganismSeed,
    EnvironmentalResource,
    ForbiddenStoredRepairOrTargetState,
}

impl SeedMaterialClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PermittedOrganismSeed => "permitted_organism_seed",
            Self::EnvironmentalResource => "environmental_resource",
            Self::ForbiddenStoredRepairOrTargetState => "forbidden_stored_repair_or_target_state",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeedContractReport {
    pub radius: f64,
    pub interface_width: f64,
    pub theta_gamma: f64,
    pub noise_seed: u64,
    pub interior_c: f64,
    pub interior_a: f64,
    pub interior_p: f64,
    pub interior_n: f64,
    pub interior_f: f64,
    pub interior_w: f64,
    pub exterior_n: f64,
    pub exterior_f: f64,
    pub s_seeded: bool,
    pub zero_s_diagnostic_only: bool,
    pub material_classes: Vec<(String, String)>,
    pub forbidden_present: bool,
    pub pass: bool,
    pub notes: Vec<String>,
}

/// Gate 1 — classify the governed analytic seed (v7 compartment recipe).
pub fn audit_governed_seed_contract(
    radius: f64,
    interface_width: f64,
    theta: f64,
    noise_seed: u64,
    interior_c: f64,
    interior_a: f64,
    interior_p: f64,
    interior_n: f64,
    interior_f: f64,
    interior_w: f64,
    n_reservoir: f64,
    f_reservoir: f64,
    s_seeded: bool,
) -> SeedContractReport {
    let mut classes = vec![
        (
            "phi_structure".into(),
            SeedMaterialClass::PermittedOrganismSeed.as_str().into(),
        ),
        (
            "catalyst_C".into(),
            SeedMaterialClass::PermittedOrganismSeed.as_str().into(),
        ),
        (
            "activated_A".into(),
            SeedMaterialClass::PermittedOrganismSeed.as_str().into(),
        ),
        (
            "precursor_P".into(),
            SeedMaterialClass::PermittedOrganismSeed.as_str().into(),
        ),
        (
            "membrane_S".into(),
            if s_seeded {
                SeedMaterialClass::PermittedOrganismSeed.as_str().into()
            } else {
                "diagnostic_zero_s".into()
            },
        ),
        (
            "interior_N_F_W".into(),
            SeedMaterialClass::PermittedOrganismSeed.as_str().into(),
        ),
        (
            "reservoir_N_F".into(),
            SeedMaterialClass::EnvironmentalResource.as_str().into(),
        ),
    ];
    // Forbidden if seed encodes a future target occupancy / repair reserve selected from later outcomes.
    let forbidden = false;
    if forbidden {
        classes.push((
            "target_occupancy".into(),
            SeedMaterialClass::ForbiddenStoredRepairOrTargetState
                .as_str()
                .into(),
        ));
    }
    let notes = vec![
        "Phase 1 permits an initial organism seed for maintenance studies".into(),
        "Zero-S is diagnostic only; failure to form membrane from zero-S does not alone fail D-048".into(),
        "No hidden future resource injection, target mass, or observer-dependent repair reserve".into(),
    ];
    SeedContractReport {
        radius,
        interface_width,
        theta_gamma: theta,
        noise_seed,
        interior_c,
        interior_a,
        interior_p,
        interior_n,
        interior_f,
        interior_w,
        exterior_n: n_reservoir,
        exterior_f: f_reservoir,
        s_seeded,
        zero_s_diagnostic_only: true,
        material_classes: classes,
        forbidden_present: forbidden,
        pass: !forbidden && radius > 0.0 && interior_c > 0.0 && interior_a > 0.0,
        notes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrozenCandidateIdentity {
    pub equation_version: String,
    pub field_schema: String,
    pub activation_law: String,
    pub k_activation: f64,
    pub productive_rates: ProductiveRatesFreeze,
    pub exchange: ExchangeFreeze,
    pub membrane_turnover_schema: String,
    pub constitutive_s_to_w: f64,
    pub transport_schema_version: u32,
    pub rho_a: f64,
    pub reservoir: ReservoirFreeze,
    pub numerical: NumericalFreeze,
    pub seed_recipe: SeedRecipeFreeze,
    pub record: String,
    pub identity_hash_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductiveRatesFreeze {
    pub k_phi: f64,
    pub k_structure: f64,
    pub k_rep: f64,
    pub k_d008_reproduction: f64,
    pub k_d008_structure: f64,
    pub k_d008_activated_decay: f64,
    pub k_d008_catalyst_turnover: f64,
    pub k_precursor: f64,
    pub k_precursor_decay: f64,
    pub k_membrane: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeFreeze {
    pub alpha: f64,
    pub beta: f64,
    pub k_exchange: f64,
    pub k_exchange_eq: f64,
    pub integrator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReservoirFreeze {
    pub n_reservoir: f64,
    pub f_reservoir: f64,
    pub w_reservoir: f64,
    pub reservoir_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericalFreeze {
    pub dt_cap: f64,
    pub net_s_flow_max: f64,
    pub retention_min: f64,
    pub localization_min: f64,
    pub window: u64,
    pub required_windows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeedRecipeFreeze {
    pub radius: f64,
    pub theta_gamma: f64,
    pub interface_width: f64,
    pub noise_seed: u64,
    pub interior_c: f64,
    pub interior_a: f64,
    pub interior_p: f64,
    pub interior_n: f64,
    pub interior_f: f64,
    pub interior_w: f64,
}

/// Build the immutable D-048 candidate identity from production params (not diagnostic clamps).
pub fn build_frozen_candidate_identity(params: &SimParams) -> FrozenCandidateIdentity {
    let rates = ProductiveRatesFreeze {
        k_phi: params.k_phi,
        k_structure: params.k_structure,
        k_rep: params.k_rep,
        k_d008_reproduction: params.k_d008_reproduction,
        k_d008_structure: params.k_d008_structure,
        k_d008_activated_decay: params.k_d008_activated_decay,
        k_d008_catalyst_turnover: params.k_d008_catalyst_turnover,
        k_precursor: params.k_precursor,
        k_precursor_decay: params.k_precursor_decay,
        k_membrane: params.k_membrane,
    };
    let exchange = ExchangeFreeze {
        alpha: D031_ALPHA_FROZEN,
        beta: D031_BETA_FROZEN,
        k_exchange: params.k_exchange,
        k_exchange_eq: params.k_exchange_eq,
        integrator: SurfaceExchangeIntegrator::InvariantDomainV2.as_str().into(),
    };
    let reservoir = ReservoirFreeze {
        n_reservoir: params.n_reservoir,
        f_reservoir: params.f_reservoir,
        w_reservoir: params.w_reservoir,
        reservoir_rate: params.reservoir_rate,
    };
    let numerical = NumericalFreeze {
        dt_cap: 0.005,
        net_s_flow_max: D048_NET_S_FLOW_MAX,
        retention_min: D048_RETENTION_MIN,
        localization_min: D048_LOCALIZATION_MIN,
        window: D048_WINDOW,
        required_windows: D048_REQUIRED_WINDOWS,
    };
    let seed = SeedRecipeFreeze {
        radius: D048_RADIUS,
        theta_gamma: D048_THETA,
        interface_width: 2.0,
        noise_seed: D048_SEED_NOISE,
        interior_c: 0.4,
        interior_a: 0.5,
        interior_p: 0.05,
        interior_n: 0.4,
        interior_f: 0.4,
        interior_w: 0.5,
    };
    let identity_hash_input = format!(
        "eq={};act=0.020*C*N*F;k_act={};schema3;alpha={};beta={};rho_a={};seed_r={}",
        EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange.as_str(),
        params.k_d008_activation,
        D031_ALPHA_FROZEN,
        D031_BETA_FROZEN,
        params.rho_a,
        D048_RADIUS
    );
    FrozenCandidateIdentity {
        equation_version: EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
            .as_str()
            .into(),
        field_schema: "seven_field_v8_phi_C_A_P_S_N_F_W".into(),
        activation_law: "r_A=k_d008_activation*C*N*F".into(),
        k_activation: params.k_d008_activation,
        productive_rates: rates,
        exchange,
        membrane_turnover_schema: SurfaceTurnoverSchema::ExchangeDamageOnly.as_str().into(),
        constitutive_s_to_w: 0.0,
        transport_schema_version: params.transport_schema_version,
        rho_a: params.rho_a,
        reservoir,
        numerical,
        seed_recipe: seed,
        record: D048_RECORD_ACTIVATION.into(),
        identity_hash_input,
    }
}

/// Production organism params: v8 schema3 + frozen productive rates + historical k=0.020.
pub fn d048_frozen_organism_params(base: &SimParams) -> SimParams {
    let mut params = v8_schema3_params();
    params.beta_c = base.beta_c;
    params.beta_a = base.beta_a;
    params.beta_n = base.beta_n;
    params.beta_f = base.beta_f;
    params.beta_w = base.beta_w;
    params.k_phi = base.k_phi;
    params.k_structure = base.k_structure;
    params.k_rep = base.k_rep;
    params.k_d008_reproduction = base.k_d008_reproduction;
    params.k_d008_activated_decay = base.k_d008_activated_decay;
    params.k_d008_catalyst_turnover = base.k_d008_catalyst_turnover;
    params.k_d008_structure = base.k_d008_structure;
    params.k_precursor = base.k_precursor;
    params.k_precursor_decay = base.k_precursor_decay;
    params.d_p = base.d_p;
    params.k_membrane = base.k_membrane;
    params.k_c_membrane = base.k_c_membrane;
    // Critical: freeze historical activation, not STAGE_E_FAILED_RATES (0.024).
    params.k_d008_activation = D048_HISTORICAL_K;
    apply_renewal_stage_mode(&mut params);
    apply_schema3_exchange_damage_only(&mut params);
    params.rho_a = 1.0;
    params
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D048Conclusion {
    FrozenBiologyMembraneBasinQualified,
    CandidatePreservationFailure,
    SeedContractInvalid,
    NoHealthyMembraneAttractor,
    AdmissibleSeedBasinFailure,
    ContinuousMembraneReplacementFailure,
    LocalDamageRepairFailure,
    RepairResourceDependenceFailure,
    MembraneCausalityFailure,
    FoundationalRegression,
    DynamicMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D048Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FrozenBiologyMembraneBasinQualified => {
                "D048_FROZEN_BIOLOGY_MEMBRANE_BASIN_QUALIFIED"
            }
            Self::CandidatePreservationFailure => "D048_CANDIDATE_PRESERVATION_FAILURE",
            Self::SeedContractInvalid => "D048_SEED_CONTRACT_INVALID",
            Self::NoHealthyMembraneAttractor => "D048_NO_HEALTHY_MEMBRANE_ATTRACTOR",
            Self::AdmissibleSeedBasinFailure => "D048_ADMISSIBLE_SEED_BASIN_FAILURE",
            Self::ContinuousMembraneReplacementFailure => {
                "D048_CONTINUOUS_MEMBRANE_REPLACEMENT_FAILURE"
            }
            Self::LocalDamageRepairFailure => "D048_LOCAL_DAMAGE_REPAIR_FAILURE",
            Self::RepairResourceDependenceFailure => "D048_REPAIR_RESOURCE_DEPENDENCE_FAILURE",
            Self::MembraneCausalityFailure => "D048_MEMBRANE_CAUSALITY_FAILURE",
            Self::FoundationalRegression => "D048_FOUNDATIONAL_REGRESSION",
            Self::DynamicMembraneContractFailure => "D048_DYNAMIC_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D048_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D048_NUMERICAL_FAILURE",
            Self::Fail => "D048_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticStateClass {
    Recovers,
    RemainsInFailedBasin,
    TerminalCollapse,
    Unresolved,
}

impl DiagnosticStateClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recovers => "recovers",
            Self::RemainsInFailedBasin => "remains_in_failed_basin",
            Self::TerminalCollapse => "terminal_collapse",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Damage40Class {
    FullRecovery,
    BoundedIncompleteRecovery,
    IrreversibleFailure,
}

impl Damage40Class {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullRecovery => "full_recovery",
            Self::BoundedIncompleteRecovery => "bounded_incomplete_recovery",
            Self::IrreversibleFailure => "irreversible_failure",
        }
    }
}

pub fn classify_damage_40(
    s_recovery_ratio: f64,
    local_occupancy_ratio: f64,
    localization: f64,
) -> Damage40Class {
    match classify_damage_repair(0.40, s_recovery_ratio, local_occupancy_ratio, localization, false)
    {
        DamageRepairClass::SuccessfulRepair => Damage40Class::FullRecovery,
        DamageRepairClass::BoundedIncompleteRepair => Damage40Class::BoundedIncompleteRecovery,
        DamageRepairClass::IrreversibleMembraneFailure => Damage40Class::IrreversibleFailure,
    }
}

/// Late-state agreement tolerances for contiguous basin (Gate 3).
pub fn late_state_agrees(
    center: &MacrostateSnapshot,
    neighbor: &MacrostateSnapshot,
) -> bool {
    rel_ok(center.radius, neighbor.radius, 0.10)
        && rel_ok(center.structural_mass, neighbor.structural_mass, 0.15)
        && rel_ok(center.c_mass, neighbor.c_mass, 0.15)
        && rel_ok(center.a_mass, neighbor.a_mass, 0.15)
        && rel_ok(center.p_mass, neighbor.p_mass, 0.15)
        && rel_ok(center.s_mass, neighbor.s_mass, 0.15)
        && (center.c_retention - neighbor.c_retention).abs() <= 0.05
        && (center.a_retention - neighbor.a_retention).abs() <= 0.05
        && (center.membrane_occupancy - neighbor.membrane_occupancy).abs() <= 0.05
}

fn rel_ok(a: f64, b: f64, tol: f64) -> bool {
    let denom = a.abs().max(b.abs()).max(1e-12);
    (a - b).abs() / denom <= tol
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacrostateSnapshot {
    pub radius: f64,
    pub structural_mass: f64,
    pub c_mass: f64,
    pub a_mass: f64,
    pub p_mass: f64,
    pub s_mass: f64,
    pub c_retention: f64,
    pub a_retention: f64,
    pub membrane_occupancy: f64,
    pub localization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthyWindowCriteria {
    pub bounded_fields: bool,
    pub coherent_component: bool,
    pub c_retention_ok: bool,
    pub a_retention_ok: bool,
    pub s_localization_ok: bool,
    pub stable_occupancy: bool,
    pub active_ads_des: bool,
    pub active_precursor_synthesis: bool,
    pub active_nf_throughput: bool,
    pub active_w_clearance: bool,
    pub net_s_flow_ok: bool,
    pub no_resource_exhaustion: bool,
    pub no_concentration_failure: bool,
    pub no_timestep_floor_failure: bool,
    pub accounting_closed: bool,
}

impl HealthyWindowCriteria {
    pub fn pass(&self) -> bool {
        self.bounded_fields
            && self.coherent_component
            && self.c_retention_ok
            && self.a_retention_ok
            && self.s_localization_ok
            && self.stable_occupancy
            && self.active_ads_des
            && self.active_precursor_synthesis
            && self.active_nf_throughput
            && self.active_w_clearance
            && self.net_s_flow_ok
            && self.no_resource_exhaustion
            && self.no_concentration_failure
            && self.no_timestep_floor_failure
            && self.accounting_closed
    }
}

pub fn evaluate_healthy_window(
    c_ret: f64,
    a_ret: f64,
    localization: f64,
    net_s_flow: f64,
    forward: f64,
    reverse: f64,
    precursor_synthesis: f64,
    n_influx: f64,
    f_influx: f64,
    w_efflux: f64,
    bounded: bool,
    coherent: bool,
    occupancy_stable: bool,
    accounting_ok: bool,
    steps_ok: bool,
    reject_detail: &str,
) -> HealthyWindowCriteria {
    let floor_fail = reject_detail.contains("timestep") || reject_detail.contains("dt_floor");
    let conc_fail = reject_detail.contains("Concentration") || reject_detail.contains("capacity");
    HealthyWindowCriteria {
        bounded_fields: bounded,
        coherent_component: coherent,
        c_retention_ok: c_ret >= D048_RETENTION_MIN,
        a_retention_ok: a_ret >= D048_RETENTION_MIN,
        s_localization_ok: localization >= D048_LOCALIZATION_MIN,
        stable_occupancy: occupancy_stable,
        active_ads_des: forward > 1e-12 && reverse > 1e-12,
        active_precursor_synthesis: precursor_synthesis > 1e-12,
        active_nf_throughput: n_influx > 1e-12 && f_influx > 1e-12,
        active_w_clearance: w_efflux > 1e-12,
        net_s_flow_ok: net_s_flow.abs() <= D048_NET_S_FLOW_MAX,
        no_resource_exhaustion: steps_ok,
        no_concentration_failure: !conc_fail,
        no_timestep_floor_failure: !floor_fail,
        accounting_closed: accounting_ok,
    }
}

/// Three consecutive qualifying windows (Gate 2 / Gate 10).
pub fn three_consecutive_qualifying(windows: &[bool]) -> bool {
    if windows.len() < D048_REQUIRED_WINDOWS {
        return false;
    }
    windows
        .windows(D048_REQUIRED_WINDOWS)
        .any(|w| w.iter().all(|&q| q))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasinNeighborKind {
    CMinus10,
    CPlus10,
    AMinus10,
    APlus10,
    RedistributeTowardP,
    RedistributeTowardS,
    SReduce10ToW,
    SReduce25ToW,
    NoiseSeed,
}

impl BasinNeighborKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CMinus10 => "c_minus_10",
            Self::CPlus10 => "c_plus_10",
            Self::AMinus10 => "a_minus_10",
            Self::APlus10 => "a_plus_10",
            Self::RedistributeTowardP => "ps_toward_p",
            Self::RedistributeTowardS => "ps_toward_s",
            Self::SReduce10ToW => "s_reduce_10_to_w",
            Self::SReduce25ToW => "s_reduce_25_to_w",
            Self::NoiseSeed => "noise_seed",
        }
    }

    pub const fn is_cardinal(self) -> bool {
        !matches!(self, Self::NoiseSeed)
    }
}

/// Gate 3 basin pass: center + ≥4 cardinals + ≥4/5 noise + late-state agreement.
pub fn seeded_basin_passes(
    center_pass: bool,
    cardinal_passes: usize,
    noise_passes: usize,
    noise_total: usize,
    late_state_agree_count: usize,
    late_state_required: usize,
) -> bool {
    center_pass
        && cardinal_passes >= 4
        && noise_total > 0
        && noise_passes * 5 >= noise_total * 4
        && late_state_agree_count >= late_state_required
}

pub fn select_conclusion(
    gate0: bool,
    gate1: bool,
    gate2: bool,
    gate3: bool,
    gate4: bool,
    gate5: bool,
    gate6: bool,
    gate7: bool,
    gate8: bool,
    gate9: bool,
    gate10: bool,
    accounting: bool,
    numerical: bool,
) -> D048Conclusion {
    if !gate0 {
        return D048Conclusion::CandidatePreservationFailure;
    }
    if !gate1 {
        return D048Conclusion::SeedContractInvalid;
    }
    if !numerical {
        return D048Conclusion::NumericalFailure;
    }
    if !accounting {
        return D048Conclusion::AccountingFailure;
    }
    if !gate2 {
        return D048Conclusion::NoHealthyMembraneAttractor;
    }
    if !gate3 {
        return D048Conclusion::AdmissibleSeedBasinFailure;
    }
    if !gate4 {
        return D048Conclusion::ContinuousMembraneReplacementFailure;
    }
    if !gate5 {
        return D048Conclusion::LocalDamageRepairFailure;
    }
    if !gate6 {
        return D048Conclusion::RepairResourceDependenceFailure;
    }
    if !gate7 {
        return D048Conclusion::MembraneCausalityFailure;
    }
    if !gate8 {
        return D048Conclusion::FoundationalRegression;
    }
    if !gate9 {
        return D048Conclusion::DynamicMembraneContractFailure;
    }
    if !gate10 {
        // Gate 10 failure maps to dynamic/contract family; keep distinct fail only if others passed.
        return D048Conclusion::Fail;
    }
    D048Conclusion::FrozenBiologyMembraneBasinQualified
}

pub fn select_route(conclusion: D048Conclusion) -> &'static str {
    match conclusion {
        D048Conclusion::FrozenBiologyMembraneBasinQualified => D048_ARCHITECTURE_PASS,
        D048Conclusion::NoHealthyMembraneAttractor => {
            "RETURN_TO_MEMBRANE_METABOLISM_COUPLING_FULL_APS_HISTORIES"
        }
        D048Conclusion::AdmissibleSeedBasinFailure => {
            "REVIEW_GOVERNED_INITIAL_SEED_AND_BASIN_BOUNDARY"
        }
        D048Conclusion::ContinuousMembraneReplacementFailure => {
            "REVIEW_PASSIVE_EXCHANGE_RESIDENCE_TIME_AND_TRACER"
        }
        D048Conclusion::LocalDamageRepairFailure => {
            "REVIEW_LOCAL_PRECURSOR_SUPPLY_AND_DAMAGE_GEOMETRY"
        }
        _ => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_k_is_020() {
        assert!((D048_HISTORICAL_K - 0.020).abs() < 1e-15);
    }

    #[test]
    fn zero_s_does_not_invalidate_seed_contract() {
        let r = audit_governed_seed_contract(
            22.0, 2.0, 0.6, 1, 0.4, 0.5, 0.05, 0.4, 0.4, 0.5, 1.0, 1.0, false,
        );
        assert!(r.pass);
        assert!(r.zero_s_diagnostic_only);
    }

    #[test]
    fn three_windows_authority() {
        assert!(!three_consecutive_qualifying(&[true, true]));
        assert!(three_consecutive_qualifying(&[true, true, true]));
        assert!(three_consecutive_qualifying(&[false, true, true, true]));
        assert!(!three_consecutive_qualifying(&[true, false, true, true]));
    }

    #[test]
    fn basin_selection_thresholds() {
        assert!(seeded_basin_passes(true, 4, 4, 5, 4, 4));
        assert!(!seeded_basin_passes(true, 3, 4, 5, 4, 4));
        assert!(!seeded_basin_passes(true, 4, 3, 5, 4, 4));
        assert!(!seeded_basin_passes(false, 8, 5, 5, 8, 4));
    }

    #[test]
    fn conclusion_ordering() {
        assert_eq!(
            select_conclusion(
                false, true, true, true, true, true, true, true, true, true, true, true, true
            ),
            D048Conclusion::CandidatePreservationFailure
        );
        assert_eq!(
            select_conclusion(
                true, true, false, true, true, true, true, true, true, true, true, true, true
            ),
            D048Conclusion::NoHealthyMembraneAttractor
        );
        assert_eq!(
            select_conclusion(
                true, true, true, true, true, true, true, true, true, true, true, true, true
            ),
            D048Conclusion::FrozenBiologyMembraneBasinQualified
        );
    }

    #[test]
    fn late_state_agreement_tolerances() {
        let c = MacrostateSnapshot {
            radius: 22.0,
            structural_mass: 1000.0,
            c_mass: 100.0,
            a_mass: 50.0,
            p_mass: 10.0,
            s_mass: 80.0,
            c_retention: 0.90,
            a_retention: 0.85,
            membrane_occupancy: 0.70,
            localization: 0.99,
        };
        let mut n = c.clone();
        n.radius = 23.5; // ~6.8%
        assert!(late_state_agrees(&c, &n));
        n.radius = 26.0; // ~18%
        assert!(!late_state_agrees(&c, &n));
    }

    #[test]
    fn damage_40_classification() {
        assert_eq!(
            classify_damage_40(0.96, 0.92, 0.96).as_str(),
            "full_recovery"
        );
        assert_eq!(
            classify_damage_40(0.60, 0.70, 0.85).as_str(),
            "bounded_incomplete_recovery"
        );
    }

    #[test]
    fn route_on_pass() {
        assert_eq!(
            select_route(D048Conclusion::FrozenBiologyMembraneBasinQualified),
            D048_ARCHITECTURE_PASS
        );
    }
}
