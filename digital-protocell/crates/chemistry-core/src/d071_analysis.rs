//! D-071 capacity-bounded precursor demand regulation helpers.
//!
//! Observer/diagnostic regulation evaluation under frozen D-070 exchange kinetics
//! and `SEED_CAPACITY_CONTRACT_V1`. Production defaults remain constitutive
//! (`m_P=1`, `K_I=0`) unless an experiment explicitly sets regulation params.

use crate::candidate_identity::sha256_hex;
use crate::d068_analysis::{candidate_b_rate, candidate_c_rate, q_p_inhibition};
use crate::d070_analysis::{
    A_RETENTION, C_RETENTION, D070_FROZEN_KT, D070_GAMMA_MAX, D070_K_EQ, D070_K_EXCHANGE,
    SEED_CAPACITY_CONTRACT_V1, STAGE_E_MIN_OCCUPANCY,
};
use crate::membrane::{precursor_synthesis_rate, precursor_synthesis_rate_regulated};
use crate::config::SimParams;
use serde::{Deserialize, Serialize};

pub const D071_PROJECT_ID: &str = "D-071";
pub const D071_AGENT_MEMORY_ID: &str =
    "D-20260722-d071-capacity-bounded-precursor-demand-regulation";
pub const D071_STARTING_COMMIT: &str = "0ac93bb";
pub const D071_STARTING_TAG: &str = "D-070-mature-membrane-seed-capacity-repair";
pub const D070_CONCLUSION: &str = "D070_SEED_REPAIR_QUALIFIES_EXCHANGE_PRECURSOR_LIMIT_REMAINS";

pub const D071_FROZEN_KT: f64 = D070_FROZEN_KT;
pub const D071_K_EXCHANGE: f64 = D070_K_EXCHANGE;
pub const D071_K_EQ: f64 = D070_K_EQ;
pub const D071_GAMMA_MAX: f64 = D070_GAMMA_MAX;

pub const PRECURSOR_REGULATION_SCHEMA_V1: &str = "PRECURSOR_REGULATION_SCHEMA_V1";
pub const CONSTITUTIVE_EQ: &str = "r_P,0 = k_precursor * A * q(C) * H(phi)";
pub const CANDIDATE_A_EQ: &str = "r_P = m_P * r_P,0";
pub const CANDIDATE_B_EQ: &str = "r_P = r_P,0 * K_I / (K_I + P)";
pub const REACTION_EQ: &str = "A -> P";

pub const OCC_FLOOR: f64 = 0.95;
pub const BOUNDARY_COVERAGE_TARGET: f64 = 1.0;
pub const P_SLOPE_BOUND: f64 = 1e-4;
pub const LEDGER_TOL: f64 = 1e-6;
pub const EPS: f64 = 1e-15;

/// D-070 R22 control envelope (Seed B / Policy D, 1200 accepted).
pub const D070_CTRL_A_RET_LO: f64 = 0.25;
pub const D070_CTRL_A_RET_HI: f64 = 0.50;
pub const D070_CTRL_OCC_LO: f64 = 0.95;
pub const D070_CTRL_P0_LO: f64 = 50.0;
pub const D070_CTRL_P1_LO: f64 = 300.0;
pub const D070_CTRL_COVERAGE: f64 = 0.999;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D071PrimaryConclusion {
    PrecursorDemandRegulationQualified,
    StageERecovered,
    ConstitutivePrecursorOverproductionNotReproduced,
    PrecursorRegulationNotIdentifiable,
    PrecursorRegulationStarvesMembraneRepair,
    PrecursorRegulationNotPortable,
    ADeficitPersistsAfterPrecursorControl,
    FoundationalRegression,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D071PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrecursorDemandRegulationQualified => {
                "D071_PRECURSOR_DEMAND_REGULATION_QUALIFIED"
            }
            Self::StageERecovered => "D071_STAGE_E_RECOVERED",
            Self::ConstitutivePrecursorOverproductionNotReproduced => {
                "D071_CONSTITUTIVE_PRECURSOR_OVERPRODUCTION_NOT_REPRODUCED"
            }
            Self::PrecursorRegulationNotIdentifiable => {
                "D071_PRECURSOR_REGULATION_NOT_IDENTIFIABLE"
            }
            Self::PrecursorRegulationStarvesMembraneRepair => {
                "D071_PRECURSOR_REGULATION_STARVES_MEMBRANE_REPAIR"
            }
            Self::PrecursorRegulationNotPortable => "D071_PRECURSOR_REGULATION_NOT_PORTABLE",
            Self::ADeficitPersistsAfterPrecursorControl => {
                "D071_A_DEFICIT_PERSISTS_AFTER_PRECURSOR_CONTROL"
            }
            Self::FoundationalRegression => "D071_FOUNDATIONAL_REGRESSION",
            Self::AccountingFailure => "D071_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D071_NUMERICAL_FAILURE",
            Self::Fail => "D071_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D071Route {
    Q,
    E,
    N,
    I,
    S,
    P,
    A,
    F,
    X,
    U,
}

impl D071Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Q => "Route_Q_precursor_demand_regulation_qualified",
            Self::E => "Route_E_stage_e_recovered",
            Self::N => "Route_N_constitutive_overproduction_not_reproduced",
            Self::I => "Route_I_regulation_not_identifiable",
            Self::S => "Route_S_regulation_starves_membrane_repair",
            Self::P => "Route_P_regulation_not_portable",
            Self::A => "Route_A_deficit_persists_after_precursor_control",
            Self::F => "Route_F_foundational_regression",
            Self::X => "Route_X_accounting_or_numerical_failure",
            Self::U => "Route_U_fail",
        }
    }

    pub const fn conclusion(self) -> D071PrimaryConclusion {
        match self {
            Self::Q => D071PrimaryConclusion::PrecursorDemandRegulationQualified,
            Self::E => D071PrimaryConclusion::StageERecovered,
            Self::N => D071PrimaryConclusion::ConstitutivePrecursorOverproductionNotReproduced,
            Self::I => D071PrimaryConclusion::PrecursorRegulationNotIdentifiable,
            Self::S => D071PrimaryConclusion::PrecursorRegulationStarvesMembraneRepair,
            Self::P => D071PrimaryConclusion::PrecursorRegulationNotPortable,
            Self::A => D071PrimaryConclusion::ADeficitPersistsAfterPrecursorControl,
            Self::F => D071PrimaryConclusion::FoundationalRegression,
            Self::X => D071PrimaryConclusion::AccountingFailure,
            Self::U => D071PrimaryConclusion::Fail,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateKind {
    Constitutive,
    ReducedConstitutive,
    ProductInhibition,
}

impl CandidateKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Constitutive => "CONSTITUTIVE",
            Self::ReducedConstitutive => "REDUCED_CONSTITUTIVE_M_P",
            Self::ProductInhibition => "PRODUCT_INHIBITION_K_I",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PrecursorRegulationParams {
    pub kind: CandidateKind,
    pub m_p: f64,
    pub k_i: f64,
}

impl PrecursorRegulationParams {
    pub fn constitutive() -> Self {
        Self {
            kind: CandidateKind::Constitutive,
            m_p: 1.0,
            k_i: 0.0,
        }
    }

    pub fn reduced(m_p: f64) -> Self {
        Self {
            kind: CandidateKind::ReducedConstitutive,
            m_p: m_p.max(0.0),
            k_i: 0.0,
        }
    }

    pub fn product_inhibition(k_i: f64) -> Self {
        Self {
            kind: CandidateKind::ProductInhibition,
            m_p: 1.0,
            k_i: k_i.max(0.0),
        }
    }

    pub fn apply_to(&self, params: &mut SimParams) {
        params.precursor_m_p = self.m_p;
        params.precursor_product_inhibition_ki = match self.kind {
            CandidateKind::ProductInhibition => self.k_i,
            _ => 0.0,
        };
    }

    pub fn identity_hash(&self) -> String {
        sha256_hex(
            format!(
                "{}|{}|m_p={:.17e}|k_i={:.17e}|schema={}",
                PRECURSOR_REGULATION_SCHEMA_V1,
                self.kind.as_str(),
                self.m_p,
                self.k_i,
                SEED_CAPACITY_CONTRACT_V1
            )
            .as_bytes(),
        )
    }
}

/// Gate 0: reproduce D-070 constitutive control signature.
pub fn d070_control_reproduced(
    a_ret: f64,
    occ: f64,
    coverage: f64,
    p0: f64,
    p1: f64,
    initial_over_capacity: bool,
    max_occ: f64,
) -> bool {
    !initial_over_capacity
        && max_occ <= 1.0 + 1e-9
        && a_ret >= D070_CTRL_A_RET_LO
        && a_ret <= D070_CTRL_A_RET_HI
        && occ >= D070_CTRL_OCC_LO
        && coverage >= D070_CTRL_COVERAGE
        && p0 >= D070_CTRL_P0_LO
        && p1 >= D070_CTRL_P1_LO
        && p1 > p0 * 1.5
}

/// Exact A→P stoichiometry for synthesis extent ξ.
pub fn a_to_p_conservation(xi: f64, da: f64, dp: f64) -> bool {
    (da + xi).abs() <= 1e-12 && (dp - xi).abs() <= 1e-12
}

/// Product-inhibition factor uses old-state P only.
pub fn product_inhibition_factor(p_old: f64, k_i: f64) -> f64 {
    q_p_inhibition(p_old, k_i)
}

pub fn regulated_rate_matches_equation(
    k_p: f64,
    a: f64,
    c: f64,
    phi: f64,
    k_c: f64,
    p: f64,
    m_p: f64,
    k_i: f64,
) -> bool {
    let constitutive = candidate_b_rate(1.0, k_p, a, c, phi, k_c);
    let got = if k_i > 0.0 {
        constitutive * m_p.max(0.0) * product_inhibition_factor(p, k_i)
    } else {
        candidate_b_rate(m_p, k_p, a, c, phi, k_c)
    };
    let expected = if k_i > 0.0 {
        candidate_c_rate(k_p, a, c, phi, k_c, p, k_i) * m_p.max(0.0)
    } else {
        candidate_b_rate(m_p, k_p, a, c, phi, k_c)
    };
    (got - expected).abs() <= 1e-12 * (1.0 + got.abs())
}

/// Normalized P slope over a measurement window: (P_end - P_start) / (P_ref * steps).
pub fn normalized_p_slope(p_start: f64, p_end: f64, steps: u64, p_ref: f64) -> f64 {
    if steps == 0 || p_ref.abs() <= EPS {
        return f64::INFINITY;
    }
    (p_end - p_start) / (p_ref.abs() * steps as f64)
}

pub fn p_is_bounded(slope: f64, p_end: f64, p_start: f64) -> bool {
    slope.is_finite() && slope.abs() <= P_SLOPE_BOUND && p_end <= p_start * 1.05 + 1.0
}

/// ≤3 analytically derived constitutive scales from measured overproduction ρ_P.
pub fn derive_m_p_candidates(rho_p: f64) -> Vec<f64> {
    let mut vals = Vec::new();
    if rho_p.is_finite() && rho_p > 1.0 {
        vals.push((1.0 / rho_p).clamp(1e-4, 0.95));
        vals.push((1.05 / rho_p).clamp(1e-4, 0.95));
        vals.push((0.5 / rho_p).clamp(1e-4, 0.95));
    } else {
        vals.extend_from_slice(&[0.5, 0.25, 0.1]);
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    vals.dedup_by(|a, b| (*a - *b).abs() <= 1e-12);
    vals.into_iter().take(3).collect()
}

/// Derive K_I so that at reference P the regulated rate matches required loss L.
/// `K_I = L * P / (r0 - L)` when r0 > L; otherwise fall back to P_ref.
pub fn derive_k_i(r0: f64, loss: f64, p_ref: f64) -> f64 {
    if !(r0.is_finite() && loss.is_finite() && p_ref.is_finite()) || r0 <= loss + EPS || p_ref <= EPS
    {
        return p_ref.max(1e-3);
    }
    let ki = loss * p_ref / (r0 - loss);
    ki.clamp(1e-4, 1e6)
}

pub fn derive_k_i_candidates(r0: f64, loss: f64, p_ref: f64) -> Vec<f64> {
    let base = derive_k_i(r0, loss, p_ref);
    let mut vals = vec![base, base * 0.5, base * 2.0, p_ref.max(1e-3)];
    vals.sort_by(|a, b| a.total_cmp(b));
    vals.dedup_by(|a, b| (*a - *b).abs() <= 1e-9 * (1.0 + a.abs()));
    vals.into_iter().take(2).collect() // leave room under max-5 with m_P trio
}

pub fn maintenance_windows_pass(
    a_rets: &[f64],
    occs: &[f64],
    coverages: &[f64],
    p_slopes: &[f64],
) -> bool {
    if a_rets.len() < 3 || occs.len() < 3 || coverages.len() < 3 || p_slopes.len() < 3 {
        return false;
    }
    let n = a_rets.len().min(occs.len()).min(coverages.len()).min(p_slopes.len());
    let start = n.saturating_sub(3);
    (start..n).all(|i| {
        a_rets[i] >= A_RETENTION
            && occs[i] >= OCC_FLOOR
            && (coverages[i] - BOUNDARY_COVERAGE_TARGET).abs() <= 1e-9
            && p_slopes[i].abs() <= P_SLOPE_BOUND
    })
}

pub fn radius_portable_row(a_ret: f64, c_ret: f64, occ: f64, coverage: f64, p_bounded: bool) -> bool {
    a_ret >= A_RETENTION
        && c_ret >= C_RETENTION
        && occ >= OCC_FLOOR
        && coverage >= STAGE_E_MIN_OCCUPANCY
        && p_bounded
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence071 {
    pub workspace_isolated: bool,
    pub d070_control_ok: bool,
    pub ledger_ok: bool,
    pub precursor_dominant_avoidable: bool,
    pub candidate_identifiable: bool,
    pub conservation_ok: bool,
    pub numerical_ok: bool,
    pub r22_maintenance_ok: bool,
    pub a_retained: bool,
    pub p_bounded: bool,
    pub repair_ok: bool,
    pub repair_starved: bool,
    pub causal_ok: bool,
    pub portable: bool,
    pub stage_e_ok: bool,
    pub foundational_regression: bool,
}

pub fn select_route(ev: RouteEvidence071) -> (D071Route, D071PrimaryConclusion) {
    if !ev.workspace_isolated || ev.foundational_regression {
        return (D071Route::F, D071PrimaryConclusion::FoundationalRegression);
    }
    if !ev.d070_control_ok {
        return (
            D071Route::N,
            D071PrimaryConclusion::ConstitutivePrecursorOverproductionNotReproduced,
        );
    }
    if !ev.ledger_ok || !ev.conservation_ok {
        return (D071Route::X, D071PrimaryConclusion::AccountingFailure);
    }
    if !ev.numerical_ok {
        return (D071Route::X, D071PrimaryConclusion::NumericalFailure);
    }
    if !ev.candidate_identifiable {
        return (
            D071Route::I,
            D071PrimaryConclusion::PrecursorRegulationNotIdentifiable,
        );
    }
    if ev.repair_starved {
        return (
            D071Route::S,
            D071PrimaryConclusion::PrecursorRegulationStarvesMembraneRepair,
        );
    }
    if !ev.portable {
        return (
            D071Route::P,
            D071PrimaryConclusion::PrecursorRegulationNotPortable,
        );
    }
    if ev.r22_maintenance_ok && ev.p_bounded && !ev.a_retained {
        return (
            D071Route::A,
            D071PrimaryConclusion::ADeficitPersistsAfterPrecursorControl,
        );
    }
    if !(ev.r22_maintenance_ok && ev.a_retained && ev.p_bounded && ev.repair_ok && ev.causal_ok) {
        return (D071Route::U, D071PrimaryConclusion::Fail);
    }
    if ev.stage_e_ok {
        return (D071Route::E, D071PrimaryConclusion::StageERecovered);
    }
    (
        D071Route::Q,
        D071PrimaryConclusion::PrecursorDemandRegulationQualified,
    )
}

/// Runtime parity: regulated helper equals constitutive when defaults set.
pub fn defaults_preserve_constitutive(params: &SimParams, phi: f64, c: f64, a: f64, p: f64) -> bool {
    let r0 = precursor_synthesis_rate(phi, c, a, params);
    let mut p2 = params.clone();
    p2.precursor_m_p = 1.0;
    p2.precursor_product_inhibition_ki = 0.0;
    let r = precursor_synthesis_rate_regulated(phi, c, a, p, &p2);
    (r - r0).abs() <= 1e-15 * (1.0 + r0.abs())
}

pub fn frozen_kinetics_unchanged(k_eq: f64, k_ex: f64, gmax: f64) -> bool {
    (k_eq - D071_K_EQ).abs() <= 1e-15
        && (k_ex - D071_K_EXCHANGE).abs() <= 1e-15
        && (gmax - D071_GAMMA_MAX).abs() <= 1e-15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_inhibition_decreases_with_p() {
        let lo = product_inhibition_factor(0.1, 1.0);
        let hi = product_inhibition_factor(10.0, 1.0);
        assert!(lo > hi);
        assert!((lo - 1.0 / 1.1).abs() < 1e-12);
    }

    #[test]
    fn route_qualified_when_gates_pass() {
        let (route, conc) = select_route(RouteEvidence071 {
            workspace_isolated: true,
            d070_control_ok: true,
            ledger_ok: true,
            precursor_dominant_avoidable: true,
            candidate_identifiable: true,
            conservation_ok: true,
            numerical_ok: true,
            r22_maintenance_ok: true,
            a_retained: true,
            p_bounded: true,
            repair_ok: true,
            repair_starved: false,
            causal_ok: true,
            portable: true,
            stage_e_ok: false,
            foundational_regression: false,
        });
        assert_eq!(route, D071Route::Q);
        assert_eq!(
            conc,
            D071PrimaryConclusion::PrecursorDemandRegulationQualified
        );
    }
}
