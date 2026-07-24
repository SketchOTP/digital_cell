//! D-085 decisive structural closure: finish D-084 dynamic basin; one mechanochemical fallback.

use crate::candidate_identity::sha256_hex;
use crate::config::SimParams;
use crate::d060_analysis::integrate_existing_structural_rates;
use crate::d084_analysis::{
    classify_restoring_nets, ledger_params, D084_LEDGER_A, D084_LEDGER_C, D084_R22_BALANCE_TOL,
    D084_SCREEN_RADII,
};
use crate::structural_kinetics::apply_mixed_turnover_params as apply_mixed;
use serde::{Deserialize, Serialize};
use std::env;

pub const D085_PROJECT_ID: &str = "D-085";
pub const D085_AGENT_MEMORY_ID: &str = "D-20260723-d085-decisive-structural-closure";
pub const D085_STARTING_COMMIT: &str = "ed7be5e";
pub const D085_STARTING_TAG: &str = "D-084-edge-structural-homeostasis-fail";
pub const D085_D083_TAG: &str = "D-083-edge-dynamic-migration-repaired";

/// Sealed D-084 fixed-radius candidate (smallest qualifying η).
pub const D085_D084_ETA: f64 = 0.07535296829558047;
pub const D085_D084_K_PHI_LOSS: f64 = 0.019629931075673273;
pub const D085_D084_HASH: &str =
    "57c250ef4b72c649146f66d0b5e96bedf5c173ed92942038d4ac9e26bf1eb168";

pub const D085_BASIN_RADII: [f64; 3] = [18.0, 22.0, 26.0];
pub const D085_NOISE_SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
pub const D085_RETENTION_MIN: f64 = 0.80;
pub const D085_COVERAGE_MIN: f64 = 0.90;
pub const D085_RADIUS_AGREE_FRAC: f64 = 0.10;
pub const D085_MASS_AGREE_FRAC: f64 = 0.15;
pub const D085_SEEDS_PASS_MIN: usize = 4;
pub const D085_REQUIRED_WINDOWS: u64 = 3;
pub const D085_DEFAULT_WINDOW: u64 = 5_000;
pub const D085_DEFAULT_MAX_ACCEPTED: u64 = 75_000;
pub const D085_RADIUS_VEL_CONV: f64 = 5e-4;
pub const D085_MECHANO_MAX_MOD: f64 = 2.0;

pub const D084_FIXED_RADIUS_PENDING: &str =
    "D084_FIXED_RADIUS_RESTORING_CROSSING_QUALIFIED_DYNAMIC_BASIN_PENDING";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D085Conclusion {
    EdgeMechanochemicalStageERecovered,
    DynamicStructuralBasinQualified,
    D084DynamicBasinQualified,
    PhaseFieldStructuralSubstrateRejected,
    StaticDynamicParityDefect,
    Fail,
}

impl D085Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EdgeMechanochemicalStageERecovered => {
                "D085_EDGE_MECHANOCHEMICAL_STAGE_E_RECOVERED"
            }
            Self::DynamicStructuralBasinQualified => "D085_DYNAMIC_STRUCTURAL_BASIN_QUALIFIED",
            Self::D084DynamicBasinQualified => "D085_D084_DYNAMIC_BASIN_QUALIFIED",
            Self::PhaseFieldStructuralSubstrateRejected => {
                "D085_PHASE_FIELD_STRUCTURAL_SUBSTRATE_REJECTED"
            }
            Self::StaticDynamicParityDefect => "D085_STATIC_DYNAMIC_PARITY_DEFECT",
            Self::Fail => "D085_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DynamicFailureClass {
    StaticDynamicParityDefect,
    UniversalDynamicGrowth,
    UniversalDynamicCollapse,
    OvershootOrOscillation,
    ResourceCouplingReversal,
    EdgeBoundaryRegression,
    NumericalFailure,
    None,
}

impl DynamicFailureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaticDynamicParityDefect => "STATIC_DYNAMIC_PARITY_DEFECT",
            Self::UniversalDynamicGrowth => "UNIVERSAL_DYNAMIC_GROWTH",
            Self::UniversalDynamicCollapse => "UNIVERSAL_DYNAMIC_COLLAPSE",
            Self::OvershootOrOscillation => "OVERSHOOT_OR_OSCILLATION",
            Self::ResourceCouplingReversal => "RESOURCE_COUPLING_REVERSAL",
            Self::EdgeBoundaryRegression => "EDGE_BOUNDARY_REGRESSION",
            Self::NumericalFailure => "NUMERICAL_FAILURE",
            Self::None => "NONE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TerminationKind {
    ThreeConvergedWindows,
    BiologicalTerminal,
    NumericalFailure,
    MaxHorizon,
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
        .max(1)
}

pub fn max_accepted() -> u64 {
    env_u64("D085_MAX_ACCEPTED", D085_DEFAULT_MAX_ACCEPTED)
}

pub fn window_size() -> u64 {
    env_u64("D085_WINDOW", D085_DEFAULT_WINDOW)
}

pub fn smoke_mode() -> bool {
    matches!(
        env::var("D085_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    )
}

pub fn d084_candidate_params() -> SimParams {
    let mut p = ledger_params();
    apply_mixed(&mut p, D085_D084_ETA, D085_D084_K_PHI_LOSS);
    p
}

pub fn candidate_hash(eta: f64, k: f64) -> String {
    sha256_hex(format!("eta={:.12};k_phi_minus={:.12}", eta, k).as_bytes())
}

pub fn verify_d084_candidate_identity() -> bool {
    candidate_hash(D085_D084_ETA, D085_D084_K_PHI_LOSS) == D085_D084_HASH
}

/// Recompute prescribed-disk nets for the sealed D-084 candidate.
pub fn prescribed_nets_d084() -> (f64, f64, f64) {
    let p = d084_candidate_params();
    let mut nets = [0.0; 3];
    for (i, &r) in D084_SCREEN_RADII.iter().enumerate() {
        let (g, l, _, _) =
            integrate_existing_structural_rates(r, D084_LEDGER_A, D084_LEDGER_C, &p);
        nets[i] = g - l;
    }
    (nets[0], nets[1], nets[2])
}

pub fn prescribed_restoring_ok() -> bool {
    let (n18, n22, n26) = prescribed_nets_d084();
    classify_restoring_nets(n18, n22, n26, D084_R22_BALANCE_TOL)
}

// --- Mechanochemical response (Phase B) ---

#[inline]
pub fn f_kappa(kappa_abs: f64, k_kappa: f64) -> f64 {
    let k = k_kappa.max(1e-18);
    let a = kappa_abs.abs();
    a / (k + a)
}

#[inline]
pub fn f_s(strain: f64, k_s: f64) -> f64 {
    (strain / k_s.max(1e-18)).tanh()
}

/// Bounded production multiplier: `1 + g_κ f_κ − g_s f_s`, clamped to [1/M, M].
#[inline]
pub fn production_multiplier(f_k: f64, f_strain: f64, g_kappa: f64, g_s: f64) -> f64 {
    let raw = 1.0 + g_kappa * f_k - g_s * f_strain;
    raw.clamp(1.0 / D085_MECHANO_MAX_MOD, D085_MECHANO_MAX_MOD)
}

/// Bounded loss multiplier: `1 + g_s f_s`, clamped to [1/M, M].
#[inline]
pub fn loss_multiplier(f_strain: f64, g_s: f64) -> f64 {
    let raw = 1.0 + g_s * f_strain;
    raw.clamp(1.0 / D085_MECHANO_MAX_MOD, D085_MECHANO_MAX_MOD)
}

#[inline]
pub fn zero_strain_production_equals_base(g_kappa: f64, kappa: f64, k_kappa: f64) -> bool {
    let fk = f_kappa(kappa, k_kappa);
    let m = production_multiplier(fk, 0.0, g_kappa, 1.0);
    // With s=0, factor = 1 + g_κ f_κ (still may differ from 1 if kappa≠0).
    // Zero-strain *equivalence* means f_s=0 so loss factor≡1 and production loses only the −g_s f_s term.
    let m0 = production_multiplier(fk, 0.0, g_kappa, 0.0);
    (m - m0).abs() < 1e-15 && (loss_multiplier(0.0, 1.0) - 1.0).abs() < 1e-15
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MechanoCandidate {
    pub label: &'static str,
    pub g_kappa: f64,
    pub g_s: f64,
    pub k_kappa: f64,
    pub k_s: f64,
}

/// Build weak/center/strong from measured |κ| and |s| scales (not from a target radius).
pub fn mechano_candidates_from_scales(kappa_scale: f64, strain_scale: f64) -> [MechanoCandidate; 3] {
    let k_kappa = kappa_scale.max(1e-6);
    let k_s = strain_scale.max(1e-6);
    [
        MechanoCandidate {
            label: "weak",
            g_kappa: 0.35,
            g_s: 0.35,
            k_kappa,
            k_s,
        },
        MechanoCandidate {
            label: "center",
            g_kappa: 0.70,
            g_s: 0.70,
            k_kappa,
            k_s,
        },
        MechanoCandidate {
            label: "strong",
            g_kappa: 1.00,
            g_s: 1.00,
            k_kappa,
            k_s,
        },
    ]
}

pub fn apply_mechano_params(params: &mut SimParams, c: &MechanoCandidate) {
    params.use_mechanochemical_structure = true;
    params.mechano_g_kappa = c.g_kappa;
    params.mechano_g_s = c.g_s;
    params.mechano_k_kappa = c.k_kappa;
    params.mechano_k_s = c.k_s;
}

pub fn clear_mechano_params(params: &mut SimParams) {
    params.use_mechanochemical_structure = false;
    params.mechano_g_kappa = 0.0;
    params.mechano_g_s = 0.0;
    params.mechano_k_kappa = 0.0;
    params.mechano_k_s = 0.0;
}

// --- Basin classification ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRunRow {
    pub radius_seed: f64,
    pub noise_seed: u64,
    pub equivalent_radius: f64,
    pub structural_mass: f64,
    pub c_mass: f64,
    pub a_mass: f64,
    pub l_mass: f64,
    pub b_mass: f64,
    pub w_mass: f64,
    pub structural_production: f64,
    pub structural_loss: f64,
    pub radius_velocity: f64,
    pub edge_coverage: f64,
    pub ghost_fraction: f64,
    pub trailing_ok: bool,
    pub c_retention: f64,
    pub a_retention: f64,
    pub accepted: u64,
    pub accepted_time: f64,
    pub termination: TerminationKind,
    pub steps_ok: bool,
    pub accounting_ok: bool,
    pub fragmented: bool,
    pub dish_contact: bool,
    pub exhausted: bool,
    pub clipped: bool,
    pub window_converged: bool,
    /// Instantaneous structural net g−l on accepted state (runtime ledger).
    pub runtime_structural_net: f64,
    /// Same-state recomputed g−l with geometry frozen (parity).
    pub frozen_structural_net: f64,
    pub parity_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusCohortResult {
    pub radius_seed: f64,
    pub rows: Vec<DynamicRunRow>,
    pub seeds_pass: usize,
    pub pass: bool,
    pub mean_final_radius: f64,
    pub detail: String,
}

pub fn row_seed_passes(row: &DynamicRunRow) -> bool {
    row.steps_ok
        && row.accounting_ok
        && !row.fragmented
        && !row.dish_contact
        && !row.exhausted
        && !row.clipped
        && row.window_converged
        && row.c_retention + 1e-12 >= D085_RETENTION_MIN
        && row.a_retention + 1e-12 >= D085_RETENTION_MIN
        && row.edge_coverage + 1e-12 >= D085_COVERAGE_MIN
        && row.ghost_fraction <= 0.20
        && row.trailing_ok
        && matches!(
            row.termination,
            TerminationKind::ThreeConvergedWindows | TerminationKind::BiologicalTerminal
        )
}

fn relative_span(values: &[f64]) -> f64 {
    let Some(min) = values.iter().copied().filter(|v| v.is_finite()).reduce(f64::min) else {
        return f64::INFINITY;
    };
    let Some(max) = values.iter().copied().filter(|v| v.is_finite()).reduce(f64::max) else {
        return f64::INFINITY;
    };
    let mid = 0.5 * (min + max).abs().max(1e-9);
    (max - min).abs() / mid
}

pub fn classify_radius_cohort(radius_seed: f64, rows: &[DynamicRunRow]) -> RadiusCohortResult {
    let passers: Vec<&DynamicRunRow> = rows.iter().filter(|r| row_seed_passes(r)).collect();
    let seeds_pass = passers.len();
    let radii: Vec<f64> = passers.iter().map(|r| r.equivalent_radius).collect();
    let c_masses: Vec<f64> = passers.iter().map(|r| r.c_mass).collect();
    let a_masses: Vec<f64> = passers.iter().map(|r| r.a_mass).collect();
    let phi_masses: Vec<f64> = passers.iter().map(|r| r.structural_mass).collect();
    let agree = seeds_pass >= D085_SEEDS_PASS_MIN
        && relative_span(&radii) <= D085_RADIUS_AGREE_FRAC + 1e-12
        && relative_span(&c_masses) <= D085_MASS_AGREE_FRAC + 1e-12
        && relative_span(&a_masses) <= D085_MASS_AGREE_FRAC + 1e-12
        && relative_span(&phi_masses) <= D085_MASS_AGREE_FRAC + 1e-12;
    // Bounded: final radii stay finite and within a loose envelope of seed neighborhood.
    let bounded = passers.iter().all(|r| {
        r.equivalent_radius.is_finite()
            && r.equivalent_radius > 4.0
            && r.equivalent_radius < radius_seed.max(22.0) * 2.5
    });
    let mean_final_radius = if radii.is_empty() {
        0.0
    } else {
        radii.iter().sum::<f64>() / radii.len() as f64
    };
    let pass = agree && bounded;
    RadiusCohortResult {
        radius_seed,
        rows: rows.to_vec(),
        seeds_pass,
        pass,
        mean_final_radius,
        detail: format!(
            "seeds_pass={seeds_pass}/{} radius_span={:.4} c_span={:.4} a_span={:.4} phi_span={:.4} bounded={bounded}",
            rows.len(),
            relative_span(&radii),
            relative_span(&c_masses),
            relative_span(&a_masses),
            relative_span(&phi_masses),
        ),
    }
}

pub fn basin_matrix_passes(cohorts: &[RadiusCohortResult]) -> bool {
    cohorts.len() == D085_BASIN_RADII.len() && cohorts.iter().all(|c| c.pass)
}

/// Classify first causal dynamic failure from a completed Phase A matrix.
pub fn classify_dynamic_failure(
    cohorts: &[RadiusCohortResult],
    parity_ok: bool,
) -> DynamicFailureClass {
    if !parity_ok {
        return DynamicFailureClass::StaticDynamicParityDefect;
    }
    if cohorts.iter().any(|c| {
        c.rows
            .iter()
            .any(|r| !r.steps_ok || matches!(r.termination, TerminationKind::NumericalFailure))
    }) {
        return DynamicFailureClass::NumericalFailure;
    }
    if cohorts.iter().any(|c| {
        c.rows
            .iter()
            .any(|r| r.edge_coverage < D085_COVERAGE_MIN || r.ghost_fraction > 0.20 || !r.trailing_ok)
    }) {
        return DynamicFailureClass::EdgeBoundaryRegression;
    }
    if cohorts.iter().any(|c| {
        c.rows
            .iter()
            .any(|r| r.a_retention < D085_RETENTION_MIN || r.c_retention < D085_RETENTION_MIN)
    }) {
        return DynamicFailureClass::ResourceCouplingReversal;
    }
    let deltas: Vec<f64> = cohorts
        .iter()
        .flat_map(|c| {
            c.rows
                .iter()
                .map(|r| r.equivalent_radius - r.radius_seed)
                .collect::<Vec<_>>()
        })
        .collect();
    let all_grow = deltas.iter().all(|d| *d > 1.0);
    let all_collapse = deltas.iter().all(|d| *d < -1.0);
    if all_grow {
        return DynamicFailureClass::UniversalDynamicGrowth;
    }
    if all_collapse {
        return DynamicFailureClass::UniversalDynamicCollapse;
    }
    // Mixed signs / large |v_R| without cohort agreement → overshoot/oscillation class.
    DynamicFailureClass::OvershootOrOscillation
}

/// Runtime net structural flow sign vs frozen prescribed-disk flow at matched radius.
pub fn parity_direction_agrees(runtime_net: f64, frozen_net: f64, tol: f64) -> bool {
    if !runtime_net.is_finite() || !frozen_net.is_finite() {
        return false;
    }
    if runtime_net.abs() <= tol && frozen_net.abs() <= tol {
        return true;
    }
    runtime_net.signum() == frozen_net.signum()
}

pub fn frozen_net_at_radius(radius: f64) -> f64 {
    let p = d084_candidate_params();
    let (g, l, _, _) =
        integrate_existing_structural_rates(radius, D084_LEDGER_A, D084_LEDGER_C, &p);
    g - l
}

/// Integrate structural rates on a frozen accepted-state field (parity checkpoint).
pub fn integrate_frozen_field_rates(
    phi: &[f64],
    activated: &[f64],
    catalyst: &[f64],
    width: usize,
    height: usize,
    in_dish: impl Fn(usize) -> bool,
    params: &SimParams,
    cell_area: f64,
) -> (f64, f64) {
    use crate::structural_kinetics::{structure_decay_rate, structure_production_rate};
    let mut g = 0.0;
    let mut l = 0.0;
    for idx in 0..width * height {
        if !in_dish(idx) {
            continue;
        }
        let p = phi[idx];
        g += structure_production_rate(p, activated[idx], catalyst[idx], params) * cell_area;
        l += structure_decay_rate(p, 0.0, params) * cell_area;
    }
    (g, l)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationReport {
    pub starting_commit: &'static str,
    pub starting_tag: &'static str,
    pub d083_tag: &'static str,
    pub d084_candidate_hash_ok: bool,
    pub prescribed_restoring_ok: bool,
    pub nets: (f64, f64, f64),
    pub mixed_default_off: bool,
    pub pending_record: &'static str,
    pub pass: bool,
}

pub fn gate_preservation(mixed_default_off: bool) -> PreservationReport {
    let hash_ok = verify_d084_candidate_identity();
    let nets = prescribed_nets_d084();
    let restoring = classify_restoring_nets(nets.0, nets.1, nets.2, D084_R22_BALANCE_TOL);
    let pass = hash_ok && restoring && mixed_default_off;
    PreservationReport {
        starting_commit: D085_STARTING_COMMIT,
        starting_tag: D085_STARTING_TAG,
        d083_tag: D085_D083_TAG,
        d084_candidate_hash_ok: hash_ok,
        prescribed_restoring_ok: restoring,
        nets,
        mixed_default_off,
        pending_record: D084_FIXED_RADIUS_PENDING,
        pass,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub conclusion: String,
    pub failure_class: String,
    pub used_mechanochemical: bool,
    pub mechano_label: Option<String>,
    pub phase_a_pass: bool,
    pub phase_c_pass: bool,
    pub stage_e_pass: bool,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub scientific_conclusion: String,
    pub next_directive: String,
    pub next_execution_started: bool,
}

pub fn select_conclusion(
    phase_a_pass: bool,
    used_mechano: bool,
    phase_c_pass: bool,
    stage_e_pass: bool,
    energy_ok: bool,
    puncture_ok: bool,
    parity_ok: bool,
    failure: DynamicFailureClass,
) -> RouteReport {
    if !parity_ok || failure == DynamicFailureClass::StaticDynamicParityDefect {
        return RouteReport {
            conclusion: D085Conclusion::StaticDynamicParityDefect.as_str().into(),
            failure_class: DynamicFailureClass::StaticDynamicParityDefect.as_str().into(),
            used_mechanochemical: used_mechano,
            mechano_label: None,
            phase_a_pass,
            phase_c_pass,
            stage_e_pass: false,
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            scientific_conclusion: "Static/dynamic structural ledger parity defect; repair before scientific conclusion.".into(),
            next_directive: "Repair STATIC_DYNAMIC_PARITY_DEFECT and complete D-085.".into(),
            next_execution_started: false,
        };
    }
    if phase_a_pass && !used_mechano {
        if stage_e_pass {
            return RouteReport {
                conclusion: D085Conclusion::EdgeMechanochemicalStageERecovered.as_str().into(),
                failure_class: DynamicFailureClass::None.as_str().into(),
                used_mechanochemical: false,
                mechano_label: None,
                phase_a_pass: true,
                phase_c_pass: false,
                stage_e_pass: true,
                d008_status: "PASS_AFTER_D085".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "AUTHORIZED_FOR_D084_MIXED_TURNOVER".into(),
                scientific_conclusion: "D-084 candidate establishes dynamic basin; Stage E recovered.".into(),
                next_directive: "Proceed to D-008 Stage F.".into(),
                next_execution_started: false,
            };
        }
        // Note: Stage E pass uses EdgeMechanochemical tag only when mechano used;
        // for pure D-084 pass without Stage E component closure:
        if energy_ok && puncture_ok {
            return RouteReport {
                conclusion: D085Conclusion::D084DynamicBasinQualified.as_str().into(),
                failure_class: DynamicFailureClass::None.as_str().into(),
                used_mechanochemical: false,
                mechano_label: None,
                phase_a_pass: true,
                phase_c_pass: false,
                stage_e_pass: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
                scientific_conclusion: "D-084 mixed-turnover candidate qualifies dynamically; Stage E incomplete.".into(),
                next_directive: "Target only remaining Stage E ledger imbalance.".into(),
                next_execution_started: false,
            };
        }
        return RouteReport {
            conclusion: D085Conclusion::D084DynamicBasinQualified.as_str().into(),
            failure_class: DynamicFailureClass::None.as_str().into(),
            used_mechanochemical: false,
            mechano_label: None,
            phase_a_pass: true,
            phase_c_pass: false,
            stage_e_pass: false,
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            scientific_conclusion: "D-084 dynamic basin qualified; energy/puncture/Stage E incomplete.".into(),
            next_directive: "Complete energy/damage/Stage E under sealed D-084 candidate.".into(),
            next_execution_started: false,
        };
    }
    if used_mechano && phase_c_pass {
        if stage_e_pass {
            return RouteReport {
                conclusion: D085Conclusion::EdgeMechanochemicalStageERecovered.as_str().into(),
                failure_class: DynamicFailureClass::None.as_str().into(),
                used_mechanochemical: true,
                mechano_label: None,
                phase_a_pass,
                phase_c_pass: true,
                stage_e_pass: true,
                d008_status: "PASS_AFTER_D085".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "AUTHORIZED_FOR_MECHANOCHEMICAL_STRUCTURE".into(),
                scientific_conclusion: "Mechanochemical fallback recovers Stage E.".into(),
                next_directive: "Proceed to D-008 Stage F.".into(),
                next_execution_started: false,
            };
        }
        if energy_ok && puncture_ok {
            return RouteReport {
                conclusion: D085Conclusion::DynamicStructuralBasinQualified.as_str().into(),
                failure_class: DynamicFailureClass::None.as_str().into(),
                used_mechanochemical: true,
                mechano_label: None,
                phase_a_pass,
                phase_c_pass: true,
                stage_e_pass: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
                scientific_conclusion: "Mechanochemical basin + energy/repair pass; Stage E component remains.".into(),
                next_directive: "Target only remaining Stage E ledger imbalance.".into(),
                next_execution_started: false,
            };
        }
    }
    RouteReport {
        conclusion: D085Conclusion::PhaseFieldStructuralSubstrateRejected
            .as_str()
            .into(),
        failure_class: failure.as_str().into(),
        used_mechanochemical: used_mechano,
        mechano_label: None,
        phase_a_pass,
        phase_c_pass,
        stage_e_pass: false,
        d008_status: "BLOCKED_NOT_RECOVERED".into(),
        phase1_status: "PHASE1_STRUCTURAL_SUBSTRATE_CLOSED".into(),
        production_verdict: "REQUIRES_SUBSTRATE_REDESIGN".into(),
        scientific_conclusion: "Phase-field structural substrate rejected after D-084 basin and one mechanochemical fallback.".into(),
        next_directive: "Redesign organism body as conserved cellular/mesh material system.".into(),
        next_execution_started: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d084_hash_matches_sealed_candidate() {
        assert!(verify_d084_candidate_identity());
    }

    #[test]
    fn mechano_factors_bounded_and_zero_strain_loss_unity() {
        let fk = f_kappa(1.0, 1.0);
        assert!((fk - 0.5).abs() < 1e-12);
        assert!((f_s(0.0, 1.0)).abs() < 1e-15);
        let m = production_multiplier(1.0, 1.0, 10.0, 10.0);
        assert!(m <= D085_MECHANO_MAX_MOD + 1e-12);
        assert!(m >= 1.0 / D085_MECHANO_MAX_MOD - 1e-12);
        assert!((loss_multiplier(0.0, 0.7) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn parity_sign_agreement() {
        assert!(parity_direction_agrees(0.1, 0.2, 1e-3));
        assert!(parity_direction_agrees(-0.1, -0.2, 1e-3));
        assert!(!parity_direction_agrees(0.1, -0.2, 1e-3));
        assert!(parity_direction_agrees(1e-6, -1e-6, 1e-3));
    }
}
