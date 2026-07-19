//! D-040 exchange–precursor coupling decomposition helpers (diagnostic only).
//!
//! No chemistry defaults change. Pure equilibrium / budget / route classification.

use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d039_analysis::v8_schema3_params;
use crate::nullcline::{classify_jacobian, FixedPointClass};
use serde::{Deserialize, Serialize};

pub const D040_STARTING_COMMIT: &str = "0df0195";
pub const D040_D039_TAG: &str = "D-039-membrane-maintenance-qualification";
pub const D040_AGENT_MEMORY_ID: &str =
    "D-20260719-1536-d040-exchange-precursor-coupling-decomposition";
pub const D040_RECORD: &str = "SCHEMA3_V8_MAINTENANCE_COUPLING_FAILED";

pub const D040_K_FROZEN: f64 = D031_ALPHA_FROZEN / D031_BETA_FROZEN;
pub const D040_PARITY_REL_TOL: f64 = 0.05;
pub const D040_PARITY_ABS_TOL: f64 = 1e-6;

/// θ_eq = K p / (1 + K p)
#[inline]
pub fn theta_eq(k: f64, p: f64) -> f64 {
    if !(k.is_finite() && p.is_finite()) || k < 0.0 || p < 0.0 {
        return f64::NAN;
    }
    let kp = k * p;
    kp / (1.0 + kp)
}

/// Required dimensionless precursor activity for target occupancy:
/// p = θ / (K (1 − θ))
#[inline]
pub fn required_p_for_theta(k: f64, theta: f64) -> f64 {
    if !(k > 0.0) || !(theta.is_finite()) || theta < 0.0 || theta >= 1.0 {
        return f64::NAN;
    }
    theta / (k * (1.0 - theta))
}

/// Predicted net exchange activity (before mobility): α q p (1−θ) − β q θ.
#[inline]
pub fn j_predicted(alpha: f64, beta: f64, q_c: f64, p: f64, theta: f64) -> f64 {
    let sat = (1.0 - theta).max(0.0);
    q_c * (alpha * p * sat - beta * theta.max(0.0))
}

/// Sign agreement and relative magnitude between predicted and observed net exchange.
#[inline]
pub fn exchange_direction_agrees(predicted: f64, observed: f64, eps: f64) -> bool {
    if predicted.abs() <= eps && observed.abs() <= eps {
        return true;
    }
    predicted.signum() == observed.signum()
        || (predicted.abs() <= eps)
        || (observed.abs() <= eps && predicted.abs() <= 10.0 * eps)
}

#[inline]
pub fn relative_err(predicted: f64, observed: f64) -> f64 {
    let scale = predicted.abs().max(observed.abs()).max(D040_PARITY_ABS_TOL);
    (predicted - observed).abs() / scale
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExchangeParityClass {
    ExchangeLawParityPassPrecursorBelowEquilibrium,
    ExchangeLawParityPassPrecursorAboveEquilibrium,
    ExchangeRuntimeParityDefect,
    ExchangeEquilibriumUndefined,
}

impl ExchangeParityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExchangeLawParityPassPrecursorBelowEquilibrium => {
                "EXCHANGE_LAW_PARITY_PASS_PRECURSOR_BELOW_EQUILIBRIUM"
            }
            Self::ExchangeLawParityPassPrecursorAboveEquilibrium => {
                "EXCHANGE_LAW_PARITY_PASS_PRECURSOR_ABOVE_EQUILIBRIUM"
            }
            Self::ExchangeRuntimeParityDefect => "EXCHANGE_RUNTIME_PARITY_DEFECT",
            Self::ExchangeEquilibriumUndefined => "EXCHANGE_EQUILIBRIUM_UNDEFINED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquilibriumAuditSample {
    pub label: String,
    pub p: f64,
    pub theta: f64,
    pub theta_eq: f64,
    pub j_predicted: f64,
    pub j_observed: f64,
    pub direction_ok: bool,
    pub magnitude_rel_err: f64,
    pub parity_ok: bool,
}

pub fn audit_exchange_sample(
    label: &str,
    p: f64,
    theta: f64,
    q_c: f64,
    j_observed: f64,
    alpha: f64,
    beta: f64,
    k: f64,
) -> EquilibriumAuditSample {
    let teq = theta_eq(k, p);
    let jp = j_predicted(alpha, beta, q_c, p, theta);
    let dir_ok = exchange_direction_agrees(jp, j_observed, D040_PARITY_ABS_TOL);
    let rel = relative_err(jp, j_observed);
    // Magnitude parity is soft when |J| is tiny; direction + finite teq are decisive.
    let mag_ok = rel <= D040_PARITY_REL_TOL
        || (jp.abs() < 1e-4 && j_observed.abs() < 1e-3)
        || (jp.abs() < 1e-8 && j_observed.abs() < 1e-6);
    EquilibriumAuditSample {
        label: label.into(),
        p,
        theta,
        theta_eq: teq,
        j_predicted: jp,
        j_observed,
        direction_ok: dir_ok,
        magnitude_rel_err: rel,
        parity_ok: dir_ok && mag_ok && teq.is_finite(),
    }
}

pub fn classify_equilibrium_audit(samples: &[EquilibriumAuditSample]) -> ExchangeParityClass {
    if samples.is_empty() {
        return ExchangeParityClass::ExchangeEquilibriumUndefined;
    }
    if samples.iter().any(|s| !s.theta_eq.is_finite() || !s.p.is_finite()) {
        return ExchangeParityClass::ExchangeEquilibriumUndefined;
    }
    if samples.iter().any(|s| !s.parity_ok) {
        return ExchangeParityClass::ExchangeRuntimeParityDefect;
    }
    // Compare mean observed θ to mean θ_eq weighted by sample count.
    let mean_theta: f64 = samples.iter().map(|s| s.theta).sum::<f64>() / samples.len() as f64;
    let mean_teq: f64 = samples.iter().map(|s| s.theta_eq).sum::<f64>() / samples.len() as f64;
    if mean_theta + 1e-6 < mean_teq {
        ExchangeParityClass::ExchangeLawParityPassPrecursorBelowEquilibrium
    } else {
        ExchangeParityClass::ExchangeLawParityPassPrecursorAboveEquilibrium
    }
}

pub fn required_p_thresholds(k: f64) -> Vec<(f64, f64)> {
    [0.25, 0.50, 0.75, 0.90]
        .into_iter()
        .map(|th| (th, required_p_for_theta(k, th)))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChronologyClass {
    AProductionDecline,
    ALeakageIncrease,
    PSynthesisDecline,
    PLeakageIncrease,
    PDecayExcess,
    SurfaceDesorptionOnset,
    PermeabilityFeedbackOnset,
    InitialStateOutsideBasin,
    Unknown,
}

impl ChronologyClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AProductionDecline => "A_PRODUCTION_DECLINE",
            Self::ALeakageIncrease => "A_LEAKAGE_INCREASE",
            Self::PSynthesisDecline => "P_SYNTHESIS_DECLINE",
            Self::PLeakageIncrease => "P_LEAKAGE_INCREASE",
            Self::PDecayExcess => "P_DECAY_EXCESS",
            Self::SurfaceDesorptionOnset => "SURFACE_DESORPTION_ONSET",
            Self::PermeabilityFeedbackOnset => "PERMEABILITY_FEEDBACK_ONSET",
            Self::InitialStateOutsideBasin => "INITIAL_STATE_OUTSIDE_BASIN",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronologyWindow {
    pub index: usize,
    pub theta: f64,
    pub theta_eq: f64,
    pub p: f64,
    pub a: f64,
    pub a_retention: f64,
    pub p_synthesis: f64,
    pub p_leakage: f64,
    pub a_leakage: f64,
    pub net_exchange: f64,
    pub permeability_proxy: f64,
    pub precursor_synthesis_demand: f64,
}

/// Earliest causal divergence: first window whose state predicts later membrane loss.
pub fn earliest_causal_divergence(windows: &[ChronologyWindow]) -> ChronologyClass {
    if windows.is_empty() {
        return ChronologyClass::Unknown;
    }
    let w0 = &windows[0];
    if w0.theta + 0.05 < w0.theta_eq && w0.p < required_p_for_theta(D040_K_FROZEN, 0.5) {
        // Already under-supplied at start relative to mid occupancy.
        if w0.a_retention < 0.9 && w0.a < windows.last().map(|w| w.a).unwrap_or(w0.a) + 1.0 {
            // Fall through to chronological scan.
        } else if w0.net_exchange < 0.0 && w0.theta < 0.4 {
            return ChronologyClass::InitialStateOutsideBasin;
        }
    }

    let mut first_a_prod = None;
    let mut first_a_leak = None;
    let mut first_p_syn = None;
    let mut first_p_leak = None;
    let mut first_p_decay = None;
    let mut first_desorb = None;
    let mut first_perm = None;

    for w in windows.windows(2) {
        let a = &w[0];
        let b = &w[1];
        let i = b.index;
        if first_a_prod.is_none() && b.a < a.a * 0.95 && b.a_retention < a.a_retention {
            first_a_prod = Some(i);
        }
        if first_a_leak.is_none() && b.a_leakage > a.a_leakage * 1.2 + 1e-9 {
            first_a_leak = Some(i);
        }
        if first_p_syn.is_none() && b.p_synthesis < a.p_synthesis * 0.95 {
            first_p_syn = Some(i);
        }
        if first_p_leak.is_none() && b.p_leakage > a.p_leakage * 1.2 + 1e-9 {
            first_p_leak = Some(i);
        }
        // Decay excess: leakage small but p falls while synthesis holds.
        if first_p_decay.is_none()
            && b.p < a.p * 0.95
            && b.p_synthesis >= a.p_synthesis * 0.98
            && b.p_leakage <= a.p_leakage * 1.05
        {
            first_p_decay = Some(i);
        }
        if first_desorb.is_none() && b.net_exchange < -1e-6 && a.net_exchange >= -1e-6 {
            first_desorb = Some(i);
        }
        if first_perm.is_none()
            && b.permeability_proxy > a.permeability_proxy * 1.15
            && b.theta < a.theta
        {
            first_perm = Some(i);
        }
    }

    // Choose earliest measured predictive event (not largest terminal delta).
    let mut candidates: Vec<(usize, ChronologyClass)> = Vec::new();
    if let Some(i) = first_a_prod {
        candidates.push((i, ChronologyClass::AProductionDecline));
    }
    if let Some(i) = first_a_leak {
        candidates.push((i, ChronologyClass::ALeakageIncrease));
    }
    if let Some(i) = first_p_syn {
        candidates.push((i, ChronologyClass::PSynthesisDecline));
    }
    if let Some(i) = first_p_leak {
        candidates.push((i, ChronologyClass::PLeakageIncrease));
    }
    if let Some(i) = first_p_decay {
        candidates.push((i, ChronologyClass::PDecayExcess));
    }
    if let Some(i) = first_desorb {
        candidates.push((i, ChronologyClass::SurfaceDesorptionOnset));
    }
    if let Some(i) = first_perm {
        candidates.push((i, ChronologyClass::PermeabilityFeedbackOnset));
    }
    candidates.sort_by_key(|(i, _)| *i);
    candidates
        .first()
        .map(|(_, c)| *c)
        .unwrap_or(ChronologyClass::Unknown)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrecursorSufficiencyOutcome {
    PassiveExchangeCanRepairWithSufficientPrecursor,
    PassiveExchangeLawCannotRepair,
}

impl PrecursorSufficiencyOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PassiveExchangeCanRepairWithSufficientPrecursor => {
                "PASSIVE_EXCHANGE_CAN_REPAIR_WITH_SUFFICIENT_PRECURSOR"
            }
            Self::PassiveExchangeLawCannotRepair => "PASSIVE_EXCHANGE_LAW_CANNOT_REPAIR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndogenousCapacityClass {
    SynthesisCapacitySufficient,
    SynthesisCapacityInsufficient,
    ProductionSufficientRetentionInsufficient,
    ASupplyInsufficient,
    MixedDeficit,
}

impl EndogenousCapacityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SynthesisCapacitySufficient => "synthesis_capacity_sufficient",
            Self::SynthesisCapacityInsufficient => "synthesis_capacity_insufficient",
            Self::ProductionSufficientRetentionInsufficient => {
                "production_sufficient_but_retention_insufficient"
            }
            Self::ASupplyInsufficient => "a_supply_insufficient",
            Self::MixedDeficit => "mixed_deficit",
        }
    }
}

pub fn classify_endogenous_capacity(
    repair_p_min: f64,
    max_endogenous_p: f64,
    max_no_leak_p: f64,
    max_no_decay_p: f64,
    max_fixed_a_p: f64,
) -> EndogenousCapacityClass {
    if !(repair_p_min.is_finite() && repair_p_min > 0.0) {
        return EndogenousCapacityClass::MixedDeficit;
    }
    if max_endogenous_p >= repair_p_min * 0.95 {
        return EndogenousCapacityClass::SynthesisCapacitySufficient;
    }
    if max_fixed_a_p >= repair_p_min * 0.95 && max_endogenous_p < repair_p_min * 0.8 {
        return EndogenousCapacityClass::ASupplyInsufficient;
    }
    let retention_helps = max_no_leak_p.max(max_no_decay_p) >= repair_p_min * 0.95;
    if retention_helps && max_endogenous_p < repair_p_min * 0.8 {
        return EndogenousCapacityClass::ProductionSufficientRetentionInsufficient;
    }
    if max_endogenous_p < repair_p_min * 0.5 {
        return EndogenousCapacityClass::SynthesisCapacityInsufficient;
    }
    EndogenousCapacityClass::MixedDeficit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D040Conclusion {
    PrecursorSynthesisCapacityDeficit,
    PrecursorRetentionDefect,
    ActivatedResourceSupplyDeficit,
    MembraneMetabolismBistability,
    PassiveExchangeLawInvalid,
    NoBoundedMembraneMaintenanceState,
    ExchangeRuntimeParityDefect,
    AuditInconclusive,
    AccountingFailure,
    NumericalFailure,
}

impl D040Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrecursorSynthesisCapacityDeficit => "D040_PRECURSOR_SYNTHESIS_CAPACITY_DEFICIT",
            Self::PrecursorRetentionDefect => "D040_PRECURSOR_RETENTION_DEFECT",
            Self::ActivatedResourceSupplyDeficit => "D040_ACTIVATED_RESOURCE_SUPPLY_DEFICIT",
            Self::MembraneMetabolismBistability => "D040_MEMBRANE_METABOLISM_BISTABILITY",
            Self::PassiveExchangeLawInvalid => "D040_PASSIVE_EXCHANGE_LAW_INVALID",
            Self::NoBoundedMembraneMaintenanceState => "D040_NO_BOUNDED_MEMBRANE_MAINTENANCE_STATE",
            Self::ExchangeRuntimeParityDefect => "D040_EXCHANGE_RUNTIME_PARITY_DEFECT",
            Self::AuditInconclusive => "D040_AUDIT_INCONCLUSIVE",
            Self::AccountingFailure => "D040_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D040_NUMERICAL_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEvidence {
    pub parity: ExchangeParityClass,
    pub sufficiency: Option<PrecursorSufficiencyOutcome>,
    pub endogenous: Option<EndogenousCapacityClass>,
    pub p_clamp_restores: bool,
    pub a_clamp_restores: bool,
    pub perm_freeze_restores: bool,
    pub no_decay_restores: bool,
    pub no_leak_restores: bool,
    pub healthy_fixed_point_exists: bool,
    pub bistable_basins: bool,
    pub damage_crosses_separatrix: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
}

/// Gate 9 route selection — exactly one primary route.
pub fn select_route(ev: &RouteEvidence) -> D040Conclusion {
    if !ev.numerical_ok {
        return D040Conclusion::NumericalFailure;
    }
    if !ev.accounting_ok {
        return D040Conclusion::AccountingFailure;
    }
    if matches!(ev.parity, ExchangeParityClass::ExchangeRuntimeParityDefect) {
        return D040Conclusion::ExchangeRuntimeParityDefect;
    }
    if matches!(
        ev.sufficiency,
        Some(PrecursorSufficiencyOutcome::PassiveExchangeLawCannotRepair)
    ) {
        return D040Conclusion::PassiveExchangeLawInvalid;
    }
    if !ev.healthy_fixed_point_exists
        && matches!(
            ev.sufficiency,
            Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor)
        )
        && !ev.p_clamp_restores
    {
        return D040Conclusion::NoBoundedMembraneMaintenanceState;
    }

    // Route F — bistability
    if ev.healthy_fixed_point_exists
        && ev.bistable_basins
        && (ev.damage_crosses_separatrix || ev.perm_freeze_restores || ev.p_clamp_restores)
    {
        return D040Conclusion::MembraneMetabolismBistability;
    }

    // Route A — A supply
    if ev.a_clamp_restores
        && matches!(
            ev.endogenous,
            Some(EndogenousCapacityClass::ASupplyInsufficient)
        )
    {
        return D040Conclusion::ActivatedResourceSupplyDeficit;
    }

    // Route R — retention
    if (ev.no_leak_restores || ev.no_decay_restores)
        && matches!(
            ev.endogenous,
            Some(EndogenousCapacityClass::ProductionSufficientRetentionInsufficient)
        )
    {
        return D040Conclusion::PrecursorRetentionDefect;
    }

    // Route P — synthesis capacity
    if matches!(
        ev.sufficiency,
        Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor)
    ) && ev.p_clamp_restores
        && matches!(
            ev.endogenous,
            Some(EndogenousCapacityClass::SynthesisCapacityInsufficient)
                | Some(EndogenousCapacityClass::MixedDeficit)
                | Some(EndogenousCapacityClass::ASupplyInsufficient)
        )
        && !ev.no_leak_restores
        && !ev.no_decay_restores
    {
        // If A clamp uniquely restores P, prefer Route A when endogenous says A.
        if ev.a_clamp_restores
            && matches!(
                ev.endogenous,
                Some(EndogenousCapacityClass::ASupplyInsufficient)
            )
        {
            return D040Conclusion::ActivatedResourceSupplyDeficit;
        }
        return D040Conclusion::PrecursorSynthesisCapacityDeficit;
    }

    if matches!(
        ev.sufficiency,
        Some(PrecursorSufficiencyOutcome::PassiveExchangeCanRepairWithSufficientPrecursor)
    ) && ev.p_clamp_restores
        && !ev.no_leak_restores
        && !ev.no_decay_restores
    {
        return D040Conclusion::PrecursorSynthesisCapacityDeficit;
    }

    if !ev.healthy_fixed_point_exists {
        return D040Conclusion::NoBoundedMembraneMaintenanceState;
    }

    D040Conclusion::AuditInconclusive
}

// --- Reduced observer APS model (Gate 7) ------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedApsParams {
    pub r_activation: f64,
    pub d_a: f64,
    pub l_a0: f64,
    pub r_p0: f64,
    pub d_p: f64,
    pub l_p0: f64,
    pub alpha: f64,
    pub beta: f64,
    pub q_c: f64,
    pub k: f64,
}

impl Default for ReducedApsParams {
    fn default() -> Self {
        Self {
            r_activation: 0.02,
            d_a: 0.01,
            l_a0: 0.05,
            r_p0: 0.01,
            d_p: 0.002,
            l_p0: 0.02,
            alpha: D031_ALPHA_FROZEN,
            beta: D031_BETA_FROZEN,
            q_c: 0.7,
            k: D040_K_FROZEN,
        }
    }
}

/// Leakage grows as occupancy falls: L(θ) = L0 * (1 − θ)_+.
#[inline]
fn leak(l0: f64, theta: f64) -> f64 {
    l0 * (1.0 - theta).max(0.0)
}

/// Observer RHS: (A, P, θ) with unit surface capacity.
pub fn reduced_aps_rhs(a: f64, p: f64, theta: f64, par: &ReducedApsParams) -> (f64, f64, f64) {
    let da = par.r_activation - par.d_a * a - leak(par.l_a0, theta) * a;
    let j = j_predicted(par.alpha, par.beta, par.q_c, p, theta);
    // R_P(A,C) ≈ r_p0 * A (catalyst folded into r_p0 scale)
    let dp = par.r_p0 * a.max(0.0) - par.d_p * p - leak(par.l_p0, theta) * p - j;
    let dtheta = j; // unit capacity observer
    (da, dp, dtheta)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedFixedPoint {
    pub a: f64,
    pub p: f64,
    pub theta: f64,
    pub jacobian_eigs: Vec<f64>,
    pub class: FixedPointClass,
    pub admissible: bool,
}

fn jacobian_3(
    a: f64,
    p: f64,
    theta: f64,
    par: &ReducedApsParams,
) -> [[f64; 3]; 3] {
    let eps = 1e-5;
    let f0 = reduced_aps_rhs(a, p, theta, par);
    let fa = reduced_aps_rhs(a + eps, p, theta, par);
    let fp = reduced_aps_rhs(a, p + eps, theta, par);
    let ft = reduced_aps_rhs(a, p, theta + eps, par);
    [
        [
            (fa.0 - f0.0) / eps,
            (fp.0 - f0.0) / eps,
            (ft.0 - f0.0) / eps,
        ],
        [
            (fa.1 - f0.1) / eps,
            (fp.1 - f0.1) / eps,
            (ft.1 - f0.1) / eps,
        ],
        [
            (fa.2 - f0.2) / eps,
            (fp.2 - f0.2) / eps,
            (ft.2 - f0.2) / eps,
        ],
    ]
}

fn max_real_eig_2x2(j: [[f64; 2]; 2]) -> (FixedPointClass, f64) {
    classify_jacobian(&j)
}

fn push_fp_if_new(out: &mut Vec<ReducedFixedPoint>, a: f64, p: f64, theta: f64, par: &ReducedApsParams) {
    let (da, dp, dt) = reduced_aps_rhs(a, p, theta, par);
    let residual = da.abs() + dp.abs() + dt.abs();
    if residual >= 2e-2 || !a.is_finite() || !p.is_finite() || !theta.is_finite() {
        return;
    }
    let j3 = jacobian_3(a, p, theta, par);
    let j2 = [[j3[1][1], j3[1][2]], [j3[2][1], j3[2][2]]];
    let (class, _max_ev) = max_real_eig_2x2(j2);
    let eigs = {
        let tr = j2[0][0] + j2[1][1];
        let det = j2[0][0] * j2[1][1] - j2[0][1] * j2[1][0];
        let disc = tr * tr - 4.0 * det;
        if disc >= 0.0 {
            let s = disc.sqrt();
            vec![(tr + s) / 2.0, (tr - s) / 2.0]
        } else {
            vec![tr / 2.0, tr / 2.0]
        }
    };
    let dup = out.iter().any(|fp: &ReducedFixedPoint| {
        (fp.a - a).abs() < 0.05 && (fp.p - p).abs() < 0.01 && (fp.theta - theta).abs() < 0.05
    });
    if !dup {
        out.push(ReducedFixedPoint {
            a,
            p,
            theta,
            jacobian_eigs: eigs,
            class,
            admissible: true,
        });
    }
}

/// Analytical exchange-nullcline seeds: J=0 ⇒ θ = θ_eq(K,p); solve A,P balance.
fn analytical_exchange_nullcline_points(par: &ReducedApsParams) -> Vec<(f64, f64, f64)> {
    let mut pts = Vec::new();
    for mut p in [0.001, 0.01, 0.02, 0.05, 0.1, 0.2] {
        let mut a = 0.0;
        let mut theta = 0.0;
        for _ in 0..32 {
            theta = theta_eq(par.k, p);
            let la = leak(par.l_a0, theta);
            let lp = leak(par.l_p0, theta);
            a = if par.d_a + la > 0.0 {
                par.r_activation / (par.d_a + la)
            } else {
                0.0
            };
            let p_next = if par.d_p + lp > 0.0 {
                (par.r_p0 * a / (par.d_p + lp)).max(0.0)
            } else {
                p
            };
            if (p_next - p).abs() < 1e-12 {
                p = p_next;
                break;
            }
            p = p_next;
        }
        theta = theta_eq(par.k, p);
        pts.push((a.max(0.0), p.max(0.0), theta));
    }
    // Collapsed / low branch under weak activation
    pts.push((0.0, 0.0, 0.0));
    pts
}

/// Find approximate fixed points by analytical seeds + multistart relaxation.
pub fn find_reduced_fixed_points(par: &ReducedApsParams) -> Vec<ReducedFixedPoint> {
    let mut out = Vec::new();
    for (a, p, theta) in analytical_exchange_nullcline_points(par) {
        push_fp_if_new(&mut out, a, p, theta, par);
    }
    let starts = [
        (0.1, 0.01, 0.1),
        (0.5, 0.05, 0.5),
        (1.0, 0.1, 0.8),
        (0.2, 0.2, 0.2),
        (0.8, 0.02, 0.7),
        (0.05, 0.001, 0.05),
    ];
    for (mut a, mut p, mut theta) in starts {
        for _ in 0..200 {
            let (da, dp, dt) = reduced_aps_rhs(a, p, theta, par);
            a = (a + 0.1 * da).max(0.0);
            p = (p + 0.1 * dp).max(0.0);
            theta = (theta + 0.1 * dt).clamp(0.0, 0.999);
            if da.abs() + dp.abs() + dt.abs() < 1e-8 {
                break;
            }
        }
        push_fp_if_new(&mut out, a, p, theta, par);
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasinOutcome {
    pub label: String,
    pub a0: f64,
    pub p0: f64,
    pub theta0: f64,
    pub a_final: f64,
    pub p_final: f64,
    pub theta_final: f64,
    pub attracted_healthy: bool,
}

pub fn integrate_reduced(
    a0: f64,
    p0: f64,
    theta0: f64,
    par: &ReducedApsParams,
    steps: usize,
    dt: f64,
) -> (f64, f64, f64) {
    let mut a = a0;
    let mut p = p0;
    let mut theta = theta0;
    for _ in 0..steps {
        let (da, dp, dt_th) = reduced_aps_rhs(a, p, theta, par);
        a = (a + dt * da).max(0.0);
        p = (p + dt * dp).max(0.0);
        theta = (theta + dt * dt_th).clamp(0.0, 0.999);
    }
    (a, p, theta)
}

pub fn classify_basins(par: &ReducedApsParams, healthy_theta_min: f64) -> Vec<BasinOutcome> {
    let starts = [
        ("low_aps", 0.05, 0.001, 0.05),
        ("healthy_aps", 0.8, 0.05, 0.7),
        ("high_p_low_s", 0.5, 0.2, 0.1),
        ("low_p_high_s", 0.5, 0.005, 0.85),
        ("damage_10", 0.6, 0.04, 0.63),
        ("damage_25", 0.6, 0.04, 0.525),
    ];
    starts
        .into_iter()
        .map(|(label, a0, p0, th0)| {
            let (a, p, th) = integrate_reduced(a0, p0, th0, par, 5_000, 0.05);
            BasinOutcome {
                label: label.into(),
                a0,
                p0,
                theta0: th0,
                a_final: a,
                p_final: p,
                theta_final: th,
                attracted_healthy: th >= healthy_theta_min && p >= required_p_for_theta(par.k, healthy_theta_min) * 0.5,
            }
        })
        .collect()
}

/// Frozen α, β, K sanity for preservation gate.
pub fn frozen_kinetics_ok() -> bool {
    let p = v8_schema3_params();
    (p.k_exchange - D031_BETA_FROZEN).abs() < 1e-12
        && (p.k_exchange_eq - D040_K_FROZEN).abs() < 1e-6
        && (D031_ALPHA_FROZEN - D031_BETA_FROZEN * D040_K_FROZEN).abs() < 1e-9
}

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn isotherm_roundtrip() {
        let k = 50.0;
        for th in [0.25, 0.5, 0.75, 0.9] {
            let p = required_p_for_theta(k, th);
            assert!((theta_eq(k, p) - th).abs() < 1e-12);
        }
    }
}
