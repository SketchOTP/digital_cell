//! D-075 cellwise exposure-gated membrane requalification helpers.
//!
//! Observer/diagnostic only. Frozen D-070…D-074 exchange kinetics and
//! `SEED_CAPACITY_CONTRACT_V1`. No biological equation or parameter changes.
//!
//! Authoritative qualification metric is exact effective exposure
//! `E_i = -Σ ln(c_{i,n})` from the production mild-FE / BE dispatch.
//! Continuous `Λ_i = Σ λ_{i,n} Δt_n` remains diagnostic.

use crate::d070_analysis::SEED_CAPACITY_CONTRACT_V1;
use crate::d072_analysis::D071_SELECTED_M_P;
use crate::d073_analysis::{D073_GAMMA_MAX, D073_K_EQ, D073_K_EXCHANGE, D073_P_REF};
use crate::d074_analysis::{
    exchange_lambda, explicit_euler_exchange_proposal, runtime_invariant_exchange_step, EPS as D074_EPS,
    PARITY_TOL_RELAXED, Q_INACTIVE_FLOOR,
};
use crate::surface_density::{validate_exchange_cell, SURFACE_CAPACITY_FLOOR};
use serde::{Deserialize, Serialize};

pub const D075_PROJECT_ID: &str = "D-075";
pub const D075_AGENT_MEMORY_ID: &str =
    "D-20260722-d075-cellwise-exposure-gated-membrane-requalification";
pub const D075_STARTING_COMMIT: &str = "b06254b";
pub const D075_STARTING_TAG: &str = "D-074-cellwise-exchange-parity-audit";
pub const D074_CONCLUSION: &str = "D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT";

pub const D075_K_EXCHANGE: f64 = D073_K_EXCHANGE;
pub const D075_K_EQ: f64 = D073_K_EQ;
pub const D075_GAMMA_MAX: f64 = D073_GAMMA_MAX;
pub const D075_P_REF: f64 = D073_P_REF;
pub const D075_SELECTED_M_P: f64 = D071_SELECTED_M_P;

pub const SEED_CONTRACT: &str = SEED_CAPACITY_CONTRACT_V1;
pub const EPS: f64 = D074_EPS;
pub const ACCOUNTING_TOL: f64 = 1e-9;
pub const PARITY_TOL: f64 = PARITY_TOL_RELAXED;
pub const EXPOSURE_GATE: f64 = 5.0;
pub const EXPOSURE_COVERAGE_GATE: f64 = 0.95;
pub const ZERO_EXPOSURE_CAP_FRAC_MAX: f64 = 0.01;
pub const FIXED_P_HOLD_TOL: f64 = 0.02;
pub const REPAIR_FRACTION_GATE: f64 = 0.95;
pub const A_RETENTION_GATE: f64 = 0.80;
pub const C_RETENTION_GATE: f64 = 0.80;
pub const OCC_GATE: f64 = 0.95;
pub const Q_INACTIVE: f64 = Q_INACTIVE_FLOOR;

/// Cross-check frozen constants remain identical to D-074 bindings.
pub fn frozen_kinetics_unchanged(k_eq: f64, k_exchange: f64, gamma_max: f64) -> bool {
    (k_eq - D075_K_EQ).abs() < 1e-15
        && (k_exchange - D075_K_EXCHANGE).abs() < 1e-15
        && (gamma_max - D075_GAMMA_MAX).abs() < 1e-15
}

/// λ = k_exchange · q(C) · (K_eq p + 1).
#[inline]
pub fn lambda_i(k_exchange: f64, q_c: f64, k_eq: f64, p: f64) -> f64 {
    exchange_lambda(k_exchange, q_c, k_eq, p)
}

/// Continuous diagnostic exposure increment: λ Δt.
#[inline]
pub fn continuous_exposure_increment(lambda: f64, dt: f64) -> f64 {
    lambda.max(0.0) * dt.max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntegratorKind {
    ExplicitEuler,
    BackwardEuler,
    Rejected,
}

impl IntegratorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitEuler => "EXPLICIT_EULER",
            Self::BackwardEuler => "BACKWARD_EULER",
            Self::Rejected => "REJECTED",
        }
    }
}

/// Contraction factor `c` for an accepted explicit step: `|1 − λ Δt|`.
#[inline]
pub fn explicit_contraction(lambda: f64, dt: f64) -> f64 {
    (1.0 - lambda.max(0.0) * dt.max(0.0)).abs()
}

/// Contraction factor `c` for an accepted backward-Euler step: `1/(1 + λ Δt)`.
#[inline]
pub fn backward_euler_contraction(lambda: f64, dt: f64) -> f64 {
    let denom = 1.0 + lambda.max(0.0) * dt.max(0.0);
    if denom <= EPS {
        1.0
    } else {
        1.0 / denom
    }
}

/// Exact effective exposure increment: `−ln(c)` with floor on `c`.
#[inline]
pub fn exact_effective_exposure_increment(contraction: f64) -> f64 {
    let c = contraction.max(EPS);
    -c.ln()
}

/// Production-faithful per-cell integrator dispatch (observer only).
///
/// Mirrors `evolve_surface_density` mild-FE / invariant-domain BE selection.
pub fn classify_production_exchange_step(
    s_old: f64,
    p_old: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
    delta_floor: f64,
    dt: f64,
) -> IntegratorKind {
    if delta <= delta_floor || gamma_max <= 0.0 || k_exchange <= 0.0 || dt <= 0.0 {
        return IntegratorKind::Rejected;
    }
    let (s_e, p_e, _xfer) = explicit_euler_exchange_proposal(
        s_old, p_old, delta, q_c, k_exchange, k_eq, p_reference, gamma_max, dt,
    );
    let mild_ok = validate_exchange_cell(
        p_e,
        s_e,
        delta,
        gamma_max,
        delta_floor,
        1.0,
        1.0,
        0.0,
    )
    .is_ok();
    if mild_ok {
        IntegratorKind::ExplicitEuler
    } else {
        // Production would attempt BE; if BE itself fails the outer step rejects.
        match runtime_invariant_exchange_step(
            s_old, p_old, delta, q_c, k_exchange, k_eq, p_reference, gamma_max, dt,
        ) {
            Ok(_) => IntegratorKind::BackwardEuler,
            Err(_) => IntegratorKind::Rejected,
        }
    }
}

/// Contraction for a classified accepted step. Rejected → 1 (zero exposure).
#[inline]
pub fn contraction_for_kind(kind: IntegratorKind, lambda: f64, dt: f64) -> f64 {
    match kind {
        IntegratorKind::ExplicitEuler => explicit_contraction(lambda, dt),
        IntegratorKind::BackwardEuler => backward_euler_contraction(lambda, dt),
        IntegratorKind::Rejected => 1.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellExposureState {
    pub lambda_cum: f64,
    pub e_exact: f64,
    pub explicit_e: f64,
    pub backward_euler_e: f64,
    pub accepted_steps: u64,
    pub rejected_attempts_seen: u64,
}

impl Default for CellExposureState {
    fn default() -> Self {
        Self {
            lambda_cum: 0.0,
            e_exact: 0.0,
            explicit_e: 0.0,
            backward_euler_e: 0.0,
            accepted_steps: 0,
            rejected_attempts_seen: 0,
        }
    }
}

impl CellExposureState {
    /// Accumulate one attempt. Rejected attempts add no exposure / time / extent.
    pub fn observe_attempt(&mut self, kind: IntegratorKind, lambda: f64, dt: f64) {
        match kind {
            IntegratorKind::Rejected => {
                self.rejected_attempts_seen = self.rejected_attempts_seen.saturating_add(1);
            }
            IntegratorKind::ExplicitEuler | IntegratorKind::BackwardEuler => {
                let c = contraction_for_kind(kind, lambda, dt);
                let de = exact_effective_exposure_increment(c);
                self.lambda_cum += continuous_exposure_increment(lambda, dt);
                self.e_exact += de;
                match kind {
                    IntegratorKind::ExplicitEuler => self.explicit_e += de,
                    IntegratorKind::BackwardEuler => self.backward_euler_e += de,
                    IntegratorKind::Rejected => {}
                }
                self.accepted_steps = self.accepted_steps.saturating_add(1);
            }
        }
    }
}

/// Snapshot/resume payload for the shared observer (no simulation feedback).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExposureObserverSnapshot {
    pub cells: Vec<CellExposureState>,
    pub accepted_sim_time: f64,
    pub accepted_steps: u64,
    pub rejected_attempts: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExactExposureClass {
    EGe5,
    EGe3Lt5,
    EGe1Lt3,
    ELt1,
    ZeroExposure,
    Unsupported,
}

impl ExactExposureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EGe5 => "E_GE_5",
            Self::EGe3Lt5 => "E_GE_3_LT_5",
            Self::EGe1Lt3 => "E_GE_1_LT_3",
            Self::ELt1 => "E_LT_1",
            Self::ZeroExposure => "ZERO_EXPOSURE",
            Self::Unsupported => "UNSUPPORTED",
        }
    }
}

#[inline]
pub fn classify_exact_exposure(e: f64, capacity: f64, supported: bool) -> ExactExposureClass {
    if !supported || capacity <= SURFACE_CAPACITY_FLOOR {
        return ExactExposureClass::Unsupported;
    }
    if e <= EPS {
        ExactExposureClass::ZeroExposure
    } else if e < 1.0 {
        ExactExposureClass::ELt1
    } else if e < 3.0 {
        ExactExposureClass::EGe1Lt3
    } else if e < 5.0 {
        ExactExposureClass::EGe3Lt5
    } else {
        ExactExposureClass::EGe5
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureQualificationReport {
    pub relevant_lawful_capacity: f64,
    pub capacity_e_ge5: f64,
    pub capacity_e_ge3: f64,
    pub capacity_e_ge1: f64,
    pub capacity_zero: f64,
    pub capacity_unsupported: f64,
    pub fraction_e_ge5: f64,
    pub fraction_e_ge3: f64,
    pub fraction_e_ge1: f64,
    pub zero_exposure_fraction: f64,
    pub min_e: f64,
    pub median_e: f64,
    pub capacity_weighted_e: f64,
    pub explicit_e_total: f64,
    pub backward_euler_e_total: f64,
    pub lambda_diagnostic_weighted: f64,
    pub qualifies: bool,
    pub histogram: Vec<(String, f64)>,
}

/// Capacity-weighted exact-exposure qualification over a relevant cell set.
///
/// `cells`: (capacity, E_i, supported, explicit_e, be_e, lambda_cum)
pub fn qualify_exposure_capacity(
    cells: &[(f64, f64, bool, f64, f64, f64)],
) -> ExposureQualificationReport {
    let mut relevant = 0.0;
    let mut cap_ge5 = 0.0;
    let mut cap_ge3 = 0.0;
    let mut cap_ge1 = 0.0;
    let mut cap_zero = 0.0;
    let mut cap_unsup = 0.0;
    let mut weighted_e = 0.0;
    let mut weighted_lam = 0.0;
    let mut explicit_total = 0.0;
    let mut be_total = 0.0;
    let mut e_samples: Vec<(f64, f64)> = Vec::new();
    let mut hist = [
        ("E_GE_5".to_string(), 0.0),
        ("E_GE_3_LT_5".to_string(), 0.0),
        ("E_GE_1_LT_3".to_string(), 0.0),
        ("E_LT_1".to_string(), 0.0),
        ("ZERO_EXPOSURE".to_string(), 0.0),
        ("UNSUPPORTED".to_string(), 0.0),
    ];

    for &(cap, e, supported, e_ex, e_be, lam) in cells {
        let c = cap.max(0.0);
        let class = classify_exact_exposure(e, c, supported);
        match class {
            ExactExposureClass::Unsupported => {
                cap_unsup += c;
                hist[5].1 += c;
            }
            other => {
                relevant += c;
                weighted_e += c * e.max(0.0);
                weighted_lam += c * lam.max(0.0);
                explicit_total += e_ex.max(0.0);
                be_total += e_be.max(0.0);
                e_samples.push((c, e.max(0.0)));
                match other {
                    ExactExposureClass::EGe5 => {
                        cap_ge5 += c;
                        cap_ge3 += c;
                        cap_ge1 += c;
                        hist[0].1 += c;
                    }
                    ExactExposureClass::EGe3Lt5 => {
                        cap_ge3 += c;
                        cap_ge1 += c;
                        hist[1].1 += c;
                    }
                    ExactExposureClass::EGe1Lt3 => {
                        cap_ge1 += c;
                        hist[2].1 += c;
                    }
                    ExactExposureClass::ELt1 => {
                        hist[3].1 += c;
                    }
                    ExactExposureClass::ZeroExposure => {
                        cap_zero += c;
                        hist[4].1 += c;
                    }
                    ExactExposureClass::Unsupported => {}
                }
            }
        }
    }

    let fraction_e_ge5 = if relevant > EPS { cap_ge5 / relevant } else { 0.0 };
    let fraction_e_ge3 = if relevant > EPS { cap_ge3 / relevant } else { 0.0 };
    let fraction_e_ge1 = if relevant > EPS { cap_ge1 / relevant } else { 0.0 };
    let zero_frac = if relevant > EPS { cap_zero / relevant } else { 0.0 };
    let capacity_weighted_e = if relevant > EPS { weighted_e / relevant } else { 0.0 };
    let lambda_diagnostic_weighted = if relevant > EPS {
        weighted_lam / relevant
    } else {
        0.0
    };

    let (min_e, median_e) = if e_samples.is_empty() {
        (0.0, 0.0)
    } else {
        let min_e = e_samples
            .iter()
            .map(|(_, e)| *e)
            .fold(f64::INFINITY, f64::min);
        // Capacity-weighted median via sorted cumulative capacity.
        e_samples.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let half = relevant * 0.5;
        let mut acc = 0.0;
        let mut med = e_samples.last().map(|s| s.1).unwrap_or(0.0);
        for &(c, e) in &e_samples {
            acc += c;
            if acc >= half {
                med = e;
                break;
            }
        }
        (if min_e.is_finite() { min_e } else { 0.0 }, med)
    };

    let qualifies = relevant > EPS
        && fraction_e_ge5 + 1e-15 >= EXPOSURE_COVERAGE_GATE
        && zero_frac <= ZERO_EXPOSURE_CAP_FRAC_MAX + 1e-15;

    ExposureQualificationReport {
        relevant_lawful_capacity: relevant,
        capacity_e_ge5: cap_ge5,
        capacity_e_ge3: cap_ge3,
        capacity_e_ge1: cap_ge1,
        capacity_zero: cap_zero,
        capacity_unsupported: cap_unsup,
        fraction_e_ge5,
        fraction_e_ge3,
        fraction_e_ge1,
        zero_exposure_fraction: zero_frac,
        min_e,
        median_e,
        capacity_weighted_e,
        explicit_e_total: explicit_total,
        backward_euler_e_total: be_total,
        lambda_diagnostic_weighted,
        qualifies,
        histogram: hist.to_vec(),
    }
}

/// Synthetic discrete contraction parity: residual ∝ e^{−E}.
pub fn synthetic_residual_ratio(e_exact: f64) -> f64 {
    (-e_exact.max(0.0)).exp()
}

/// Predict residual after exact effective exposure under linear exchange.
#[inline]
pub fn predict_distance_from_eq(distance0: f64, e_exact: f64) -> f64 {
    distance0.abs() * synthetic_residual_ratio(e_exact)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LongHorizonClass {
    TrueLongHorizonMaintenance,
    SlowTransientDecay,
    EquilibriumBelowContract,
    ActivatedResourceCollapse,
    CatalyticExposureFailure,
    NumericalTerminal,
    NotQualified,
}

impl LongHorizonClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrueLongHorizonMaintenance => "TRUE_LONG_HORIZON_MAINTENANCE",
            Self::SlowTransientDecay => "SLOW_TRANSIENT_DECAY",
            Self::EquilibriumBelowContract => "EQUILIBRIUM_BELOW_CONTRACT",
            Self::ActivatedResourceCollapse => "ACTIVATED_RESOURCE_COLLAPSE",
            Self::CatalyticExposureFailure => "CATALYTIC_EXPOSURE_FAILURE",
            Self::NumericalTerminal => "NUMERICAL_TERMINAL",
            Self::NotQualified => "NOT_QUALIFIED",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaintenanceEvidence {
    pub exposure_qualified: bool,
    pub numerical_terminal: bool,
    pub biological_terminal: bool,
    pub mature_occupancy: f64,
    pub a_retention: f64,
    pub c_retention: f64,
    pub p_bounded: bool,
    pub zero_exposure_fraction: f64,
    pub catalytic_exposure_failure: bool,
    pub eq_occ_from_local_p: f64,
    pub s_retention: f64,
}

pub fn classify_long_horizon(ev: MaintenanceEvidence) -> LongHorizonClass {
    if ev.numerical_terminal {
        return LongHorizonClass::NumericalTerminal;
    }
    if !ev.exposure_qualified {
        if ev.catalytic_exposure_failure || ev.zero_exposure_fraction > ZERO_EXPOSURE_CAP_FRAC_MAX {
            return LongHorizonClass::CatalyticExposureFailure;
        }
        return LongHorizonClass::NotQualified;
    }
    if ev.biological_terminal || ev.a_retention < 0.05 {
        return LongHorizonClass::ActivatedResourceCollapse;
    }
    if ev.mature_occupancy + 1e-12 >= OCC_GATE
        && ev.a_retention + 1e-12 >= A_RETENTION_GATE
        && ev.c_retention + 1e-12 >= C_RETENTION_GATE
        && ev.p_bounded
    {
        return LongHorizonClass::TrueLongHorizonMaintenance;
    }
    if ev.eq_occ_from_local_p + 1e-12 < OCC_GATE && ev.mature_occupancy + 1e-12 < OCC_GATE {
        return LongHorizonClass::EquilibriumBelowContract;
    }
    if ev.s_retention + 1e-12 < 0.99 || ev.mature_occupancy + 1e-12 < OCC_GATE {
        return LongHorizonClass::SlowTransientDecay;
    }
    LongHorizonClass::NotQualified
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D075PrimaryConclusion {
    ExposureGatedMembraneRequalified,
    StageERecovered,
    PrecursorRegulationQualified,
    PrecursorRegulationRepairTradeoff,
    FrozenExchangeMetabolicallyUnreachable,
    LocalCatalyticExposureLimit,
    MembraneHorizonUnqualifiable,
    FrozenExchangeMaintenanceFailure,
    D074ResultNotReproduced,
    ExposureObserverDefect,
    FixedPExposureParityFailure,
    AccountingFailure,
    NumericalFailure,
}

impl D075PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExposureGatedMembraneRequalified => "D075_EXPOSURE_GATED_MEMBRANE_REQUALIFIED",
            Self::StageERecovered => "D075_STAGE_E_RECOVERED",
            Self::PrecursorRegulationQualified => "D075_PRECURSOR_REGULATION_QUALIFIED",
            Self::PrecursorRegulationRepairTradeoff => "D075_PRECURSOR_REGULATION_REPAIR_TRADEOFF",
            Self::FrozenExchangeMetabolicallyUnreachable => {
                "D075_FROZEN_EXCHANGE_METABOLICALLY_UNREACHABLE"
            }
            Self::LocalCatalyticExposureLimit => "D075_LOCAL_CATALYTIC_EXPOSURE_LIMIT",
            Self::MembraneHorizonUnqualifiable => "D075_MEMBRANE_HORIZON_UNQUALIFIABLE",
            Self::FrozenExchangeMaintenanceFailure => "D075_FROZEN_EXCHANGE_MAINTENANCE_FAILURE",
            Self::D074ResultNotReproduced => "D075_D074_RESULT_NOT_REPRODUCED",
            Self::ExposureObserverDefect => "D075_EXPOSURE_OBSERVER_DEFECT",
            Self::FixedPExposureParityFailure => "D075_FIXED_P_EXPOSURE_PARITY_FAILURE",
            Self::AccountingFailure => "D075_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D075_NUMERICAL_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D075Route {
    Q,
    R,
    T,
    M,
    C,
    H,
    F,
    StageE,
    StopD074,
    StopObserver,
    StopFixedP,
    StopAccounting,
    StopNumerical,
}

impl D075Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Q => "Route_Q_metric_repair_qualifies_existing_biology",
            Self::R => "Route_R_precursor_regulation_qualified",
            Self::T => "Route_T_precursor_regulation_repair_tradeoff",
            Self::M => "Route_M_endogenous_exchange_state_unreachable",
            Self::C => "Route_C_catalytic_exposure_failure",
            Self::H => "Route_H_horizon_cannot_qualify",
            Self::F => "Route_F_foundational_maintenance_failure",
            Self::StageE => "Route_Q_plus_stage_e_recovered",
            Self::StopD074 => "Stop_d074_not_reproduced",
            Self::StopObserver => "Stop_exposure_observer_defect",
            Self::StopFixedP => "Stop_fixed_p_exposure_parity_failure",
            Self::StopAccounting => "Stop_accounting",
            Self::StopNumerical => "Stop_numerical",
        }
    }

    pub const fn conclusion(self) -> D075PrimaryConclusion {
        match self {
            Self::Q => D075PrimaryConclusion::ExposureGatedMembraneRequalified,
            Self::R => D075PrimaryConclusion::PrecursorRegulationQualified,
            Self::T => D075PrimaryConclusion::PrecursorRegulationRepairTradeoff,
            Self::M => D075PrimaryConclusion::FrozenExchangeMetabolicallyUnreachable,
            Self::C => D075PrimaryConclusion::LocalCatalyticExposureLimit,
            Self::H => D075PrimaryConclusion::MembraneHorizonUnqualifiable,
            Self::F => D075PrimaryConclusion::FrozenExchangeMaintenanceFailure,
            Self::StageE => D075PrimaryConclusion::StageERecovered,
            Self::StopD074 => D075PrimaryConclusion::D074ResultNotReproduced,
            Self::StopObserver => D075PrimaryConclusion::ExposureObserverDefect,
            Self::StopFixedP => D075PrimaryConclusion::FixedPExposureParityFailure,
            Self::StopAccounting => D075PrimaryConclusion::AccountingFailure,
            Self::StopNumerical => D075PrimaryConclusion::NumericalFailure,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RouteEvidence075 {
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub d074_reproduced: bool,
    pub observer_ok: bool,
    pub synthetic_calibration_ok: bool,
    pub fixed_p_ok: bool,
    pub fixed_p_repairs: bool,
    pub constitutive_maintains: bool,
    pub constitutive_repairs: bool,
    pub regulated_maintains: bool,
    pub regulated_repairs: bool,
    pub regulated_a_ok: bool,
    pub regulated_p_bounded: bool,
    pub radius_portable: bool,
    pub catalytic_exposure_limit: bool,
    pub horizon_unqualifiable: bool,
    pub endogenous_p_insufficient: bool,
    pub stage_e_ok: bool,
}

impl Default for RouteEvidence075 {
    fn default() -> Self {
        Self {
            accounting_ok: true,
            numerical_ok: true,
            d074_reproduced: false,
            observer_ok: false,
            synthetic_calibration_ok: false,
            fixed_p_ok: false,
            fixed_p_repairs: false,
            constitutive_maintains: false,
            constitutive_repairs: false,
            regulated_maintains: false,
            regulated_repairs: false,
            regulated_a_ok: false,
            regulated_p_bounded: false,
            radius_portable: false,
            catalytic_exposure_limit: false,
            horizon_unqualifiable: false,
            endogenous_p_insufficient: false,
            stage_e_ok: false,
        }
    }
}

/// Select exactly one primary D-075 route.
pub fn select_route(ev: RouteEvidence075) -> D075Route {
    if !ev.numerical_ok {
        return D075Route::StopNumerical;
    }
    if !ev.accounting_ok {
        return D075Route::StopAccounting;
    }
    if !ev.d074_reproduced {
        return D075Route::StopD074;
    }
    if !ev.observer_ok || !ev.synthetic_calibration_ok {
        return D075Route::StopObserver;
    }
    if !ev.fixed_p_ok {
        return D075Route::StopFixedP;
    }
    if ev.horizon_unqualifiable {
        return D075Route::H;
    }
    if ev.catalytic_exposure_limit {
        return D075Route::C;
    }
    // Regulation qualified (Route R) before generic metric repair.
    if ev.regulated_maintains
        && ev.regulated_repairs
        && ev.regulated_a_ok
        && ev.regulated_p_bounded
        && ev.radius_portable
    {
        return D075Route::R;
    }
    // Tradeoff: regulation maintains A/P but cannot repair.
    if ev.regulated_maintains && ev.regulated_a_ok && ev.regulated_p_bounded && !ev.regulated_repairs
    {
        return D075Route::T;
    }
    // Metric repair qualifies constitutive biology.
    if ev.constitutive_maintains && ev.constitutive_repairs {
        if ev.stage_e_ok {
            return D075Route::StageE;
        }
        return D075Route::Q;
    }
    // Fixed-P repairs but endogenous chemistry cannot reach needed local P.
    if ev.fixed_p_repairs && ev.endogenous_p_insufficient {
        return D075Route::M;
    }
    // Foundational failure under sufficient endogenous P + catalytic exposure.
    if !ev.constitutive_maintains || !ev.constitutive_repairs {
        return D075Route::F;
    }
    D075Route::H
}

#[cfg(test)]
mod unit_smoke {
    use super::*;

    #[test]
    fn be_contraction_matches_formula() {
        let lam = 0.2;
        let dt = 0.05;
        let c = backward_euler_contraction(lam, dt);
        assert!((c - 1.0 / (1.0 + lam * dt)).abs() < 1e-15);
        let e = exact_effective_exposure_increment(c);
        assert!((e - (1.0 + lam * dt).ln()).abs() < 1e-15);
    }
}
