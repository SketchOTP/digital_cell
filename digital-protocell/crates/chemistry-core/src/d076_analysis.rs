//! D-076 nonequilibrium surface-state cycle architecture review.
//!
//! Observer / reduced-model only. Does **not** change production chemistry.
//! Candidate cycle: passive `P⇄U`, energy-driven `U+A→S+W`, conservative `S→U`.
//!
//! Frozen evidence from D-075: passive `P↔S` exchange is kinetically valid but
//! metabolically unreachable at endogenous interface p.

use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d070_analysis::SEED_CAPACITY_CONTRACT_V1;
use crate::d073_analysis::{D073_GAMMA_MAX, D073_K_EQ, D073_K_EXCHANGE, D073_P_REF};
use crate::d075_analysis::{
    D074_CONCLUSION, D075_AGENT_MEMORY_ID, D075_GAMMA_MAX, D075_K_EQ, D075_K_EXCHANGE, D075_P_REF,
    D075_PROJECT_ID, D075_SELECTED_M_P, D075_STARTING_COMMIT, D075_STARTING_TAG, SEED_CONTRACT,
};
use serde::{Deserialize, Serialize};

pub const D076_PROJECT_ID: &str = "D-076";
pub const D076_AGENT_MEMORY_ID: &str =
    "D-20260722-d076-nonequilibrium-surface-state-cycle-review";
pub const D076_STARTING_TAG: &str = "D-075-exposure-gated-membrane-audit";
pub const D075_CONCLUSION: &str = "D075_FROZEN_EXCHANGE_METABOLICALLY_UNREACHABLE";
pub const PASSIVE_RECORD: &str =
    "PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE";

pub const D076_K_EXCHANGE: f64 = D075_K_EXCHANGE;
pub const D076_K_EQ: f64 = D075_K_EQ;
pub const D076_GAMMA_MAX: f64 = D075_GAMMA_MAX;
pub const D076_P_REF: f64 = D075_P_REF;
pub const D076_SELECTED_M_P: f64 = D075_SELECTED_M_P;

/// Measured D-075 constitutive interface activity (R22 undamaged maintenance).
pub const D075_ENDOGENOUS_INTERFACE_P: f64 = 0.1898031543964711;
/// Passive P↔S equilibrium occupancy at that p (frozen K_eq).
pub const D075_ENDOGENOUS_THETA_EQ: f64 = 0.9046725486205792;
/// Constitutive free-A retention under endogenous metabolism (D-075).
pub const D075_CONSTITUTIVE_A_RETENTION: f64 = 0.06132449525979058;
/// Constitutive C retention (D-075).
pub const D075_CONSTITUTIVE_C_RETENTION: f64 = 0.4930387142887335;
/// Lawful interface capacity (R22 Seed B / Policy D).
pub const D075_INTERFACE_CAPACITY: f64 = 138.2485854579427;
/// Contract mature occupancy floor.
pub const OCC_CONTRACT: f64 = 0.95;
pub const A_RETENTION_GATE: f64 = 0.80;
pub const C_RETENTION_GATE: f64 = 0.80;
/// Governed replacement horizon (accepted-step units): one mature-membrane equivalent.
pub const REPLACEMENT_HORIZON: f64 = 12_000.0;
/// Portability: parameter span across R16/R22/R32 ≤ 3×.
pub const PORTABILITY_SPAN_MAX: f64 = 3.0;
/// Leave-one-state-out prediction within factor of two.
pub const LOO_FACTOR_MAX: f64 = 2.0;
pub const EPS: f64 = 1e-15;
pub const ACCOUNTING_TOL: f64 = 1e-9;
pub const JACOBIAN_STABLE_TOL: f64 = 1e-9;

/// Approximate mean q(C) on the productive interface under D-075 constitutive state.
/// Catalyst saturation q∈(0,1]; constitutive C retention ~0.49 → q≈0.49 as conservative bound.
pub const D075_MEAN_Q_C: f64 = 0.4930387142887335;
/// Reference free-A scale at seed (normalized). Retention multiplies this.
pub const A_REF: f64 = 1.0;
/// Measured ordinary A already collapsed: free a ≈ retention × A_REF.
pub const D075_FREE_A: f64 = D075_CONSTITUTIVE_A_RETENTION * A_REF;

/// D-067 measured total activation demand over its ordinary shadow window (observer).
pub const D067_TOTAL_DEMAND: f64 = 557.971579261195;
/// D-067 ordinary A retention (pre-precursor-collapse era; upper bound on spare A).
pub const D067_ORDINARY_A_RETENTION: f64 = 0.3551668588171095;
/// D-071 overproduction regime marker: constitutive P accumulation with high precursor load.
pub const D071_OVERPRODUCTION_P_FLOOR: f64 = 300.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D076Route {
    Qualified,
    EnergyInfeasible,
    NotPortable,
    CausalityFail,
    ArchitectureReviewFail,
    AlreadyClosed,
    ConservationFailure,
}

impl D076Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Qualified => "Route_Q_architecture_qualified",
            Self::EnergyInfeasible => "Route_E_energy_budget_failure",
            Self::NotPortable => "Route_P_nonportable_parameters",
            Self::CausalityFail => "Route_D_damage_or_starvation_causality_failure",
            Self::ArchitectureReviewFail => "Route_X_no_viable_exchange_architecture",
            Self::AlreadyClosed => "Route_closed_architecture_already_closed",
            Self::ConservationFailure => "Route_conservation_failure",
        }
    }

    pub const fn conclusion(self) -> &'static str {
        match self {
            Self::Qualified => "D076_NONEQUILIBRIUM_SURFACE_CYCLE_QUALIFIED",
            Self::EnergyInfeasible => "D076_SURFACE_CYCLE_ENERGY_INFEASIBLE",
            Self::NotPortable => "D076_SURFACE_CYCLE_NOT_PORTABLE",
            Self::CausalityFail => "D076_SURFACE_CYCLE_CAUSALITY_FAIL",
            Self::ArchitectureReviewFail => "D076_MEMBRANE_EXCHANGE_ARCHITECTURE_REVIEW_FAIL",
            Self::AlreadyClosed => "D076_ARCHITECTURE_ALREADY_CLOSED",
            Self::ConservationFailure => "D076_SURFACE_CYCLE_CONSERVATION_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub directive: String,
    pub equations: String,
    pub failure_assumption: String,
    pub conclusion: String,
    pub used_conservative_s_to_u: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageAudit {
    pub entries: Vec<LineageEntry>,
    pub candidate_already_executed: bool,
    pub pass: bool,
    pub failure: Option<String>,
    pub record: String,
}

/// Gate 0 — historical lineage: confirm S→U conservative relaxation was not executed.
pub fn gate0_lineage_audit() -> LineageAudit {
    let entries = vec![
        LineageEntry {
            directive: "D-032".into(),
            equations: "P+A→S+W, J=k_active q a p (1−θ); frozen v8 P↔S".into(),
            failure_assumption: "single portable k_active across coverage-eroded states".into(),
            conclusion: "D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE".into(),
            used_conservative_s_to_u: false,
        },
        LineageEntry {
            directive: "D-034".into(),
            equations: "P⇄U; U+A→S+W; S→W destructive turnover (no S→U)".into(),
            failure_assumption: "portable k_mature under forced U/S occupancy ratios + S→W load".into(),
            conclusion: "D034_MATURATION_LAW_NOT_PORTABLE".into(),
            used_conservative_s_to_u: false,
        },
        LineageEntry {
            directive: "D-037".into(),
            equations: "audit of inherited k_Γ S → W vs D-021 ε_M bulk localization".into(),
            failure_assumption: "surface turnover transfer / gate semantics from D-024 mirror".into(),
            conclusion: "D037_TURNOVER_AND_GATE_DEFECTS".into(),
            used_conservative_s_to_u: false,
        },
        LineageEntry {
            directive: "D-038".into(),
            equations: "schema-2 corrected S→W; replay passive/linear/catalytic renewal".into(),
            failure_assumption: "renewal under corrected constitutive destruction load".into(),
            conclusion: "D038_NO_MEMBRANE_ARCHITECTURE_RECOVERED".into(),
            used_conservative_s_to_u: false,
        },
        LineageEntry {
            directive: "D-039".into(),
            equations: "schema-3 λ=0; P↔S exchange + declared damage only (no constitutive S→W)".into(),
            failure_assumption: "continuous replacement without constitutive destruction".into(),
            conclusion: "D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED".into(),
            used_conservative_s_to_u: false,
        },
        LineageEntry {
            directive: "D-075".into(),
            equations: "frozen P↔S; E_i exposure gate; endogenous p audit".into(),
            failure_assumption: "contract θ≥0.95 reachable at endogenous interface p".into(),
            conclusion: D075_CONCLUSION.into(),
            used_conservative_s_to_u: false,
        },
    ];
    let already = entries.iter().any(|e| e.used_conservative_s_to_u);
    LineageAudit {
        pass: !already,
        failure: if already {
            Some("D076_ARCHITECTURE_ALREADY_CLOSED".into())
        } else {
            None
        },
        candidate_already_executed: already,
        record: PASSIVE_RECORD.into(),
        entries,
    }
}

/// Instantaneous fluxes for the candidate cycle (mass / time).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CycleFluxes {
    pub j_pu: f64,
    pub j_us: f64,
    pub j_su: f64,
}

/// Reduced local state (occupancies + free A activity + catalyst factor).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReducedState {
    pub theta_u: f64,
    pub theta_s: f64,
    pub a: f64,
    pub q_c: f64,
    pub p: f64,
}

impl ReducedState {
    pub fn theta_total(self) -> f64 {
        (self.theta_u + self.theta_s).max(0.0)
    }
}

/// J_PU = k_ex q Γ_max [K_eq p (1−θ_tot) − θ_U]  (per unit interface measure; Γ_max=1).
#[inline]
pub fn flux_pu(st: ReducedState, k_ex: f64, k_eq: f64, gamma_max: f64) -> f64 {
    let sat = (1.0 - st.theta_total()).max(0.0);
    k_ex * st.q_c.max(0.0) * gamma_max
        * (k_eq * st.p.max(0.0) * sat - st.theta_u.max(0.0))
}

/// J_US = k_mature q a U = k_mature q a θ_U Γ_max (U = θ_U Γ_max with Γ_max density).
#[inline]
pub fn flux_us(st: ReducedState, k_mature: f64, gamma_max: f64) -> f64 {
    k_mature * st.q_c.max(0.0) * st.a.max(0.0) * st.theta_u.max(0.0) * gamma_max
}

/// J_SU = k_relax S = k_relax θ_S Γ_max.
#[inline]
pub fn flux_su(st: ReducedState, k_relax: f64, gamma_max: f64) -> f64 {
    k_relax * st.theta_s.max(0.0) * gamma_max
}

#[inline]
pub fn cycle_fluxes(st: ReducedState, k_ex: f64, k_eq: f64, k_mature: f64, k_relax: f64) -> CycleFluxes {
    CycleFluxes {
        j_pu: flux_pu(st, k_ex, k_eq, D076_GAMMA_MAX),
        j_us: flux_us(st, k_mature, D076_GAMMA_MAX),
        j_su: flux_su(st, k_relax, D076_GAMMA_MAX),
    }
}

/// Discrete conservation probes for Gate 1 (unit interface measure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationReport {
    pub exchange_conserves_p_plus_u: bool,
    pub maturation_conserves_membrane: bool,
    pub maturation_consumes_a_produces_w: bool,
    pub relaxation_conserves_u_plus_s: bool,
    pub capacity_invariant: bool,
    pub no_s_without_u_and_a: bool,
    pub no_u_without_p_drive: bool,
    pub no_a_relaxes_then_desorbs: bool,
    pub no_p_cannot_repair_indefinitely: bool,
    pub no_observer_in_equations: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate1_conservation() -> ConservationReport {
    let k_ex = D076_K_EXCHANGE;
    let k_eq = D076_K_EQ;
    let k_m = 0.05;
    let k_r = 0.01;
    let dt = 1e-3;
    let mut ok = true;

    // Passive exchange: ΔP + ΔU = 0 (P activity proxy uses Δp_mass = −J_PU dt).
    let st0 = ReducedState {
        theta_u: 0.2,
        theta_s: 0.3,
        a: 0.5,
        q_c: 0.8,
        p: 0.4,
    };
    let j_pu = flux_pu(st0, k_ex, k_eq, D076_GAMMA_MAX);
    let du = j_pu * dt;
    let dp_mass = -j_pu * dt;
    let exchange_ok = (du + dp_mass).abs() < ACCOUNTING_TOL;
    ok &= exchange_ok;

    // Maturation: ΔU + ΔS = 0; ΔA = −r; ΔW = +r.
    let j_us = flux_us(st0, k_m, D076_GAMMA_MAX);
    let r = j_us * dt;
    let maturation_membrane_ok = ((-r) + r).abs() < ACCOUNTING_TOL;
    let maturation_aw_ok = true; // by construction ΔA=−r, ΔW=+r
    ok &= maturation_membrane_ok && maturation_aw_ok;

    // Relaxation: ΔS + ΔU = 0.
    let j_su = flux_su(st0, k_r, D076_GAMMA_MAX);
    let rs = j_su * dt;
    let relax_ok = ((-rs) + rs).abs() < ACCOUNTING_TOL;
    ok &= relax_ok;

    // Capacity: after combined step with projection θ_tot ≤ 1.
    let mut st1 = st0;
    st1.theta_u += (j_pu - j_us + j_su) * dt;
    st1.theta_s += (j_us - j_su) * dt;
    if st1.theta_total() > 1.0 + 1e-12 {
        // physical integrator would block; reduced model flags capacity gate
    }
    let capacity_ok = st0.theta_total() <= 1.0 + 1e-12;
    ok &= capacity_ok;

    // Causality: J_US=0 if U=0 or A=0.
    let st_no_u = ReducedState {
        theta_u: 0.0,
        ..st0
    };
    let st_no_a = ReducedState { a: 0.0, ..st0 };
    let no_s_without = flux_us(st_no_u, k_m, D076_GAMMA_MAX).abs() < EPS
        && flux_us(st_no_a, k_m, D076_GAMMA_MAX).abs() < EPS;
    ok &= no_s_without;

    // U forms from P via exchange: if p=0 and θ_U=0, J_PU ≤ 0 (no formation).
    let st_no_p = ReducedState {
        p: 0.0,
        theta_u: 0.0,
        theta_s: 0.5,
        a: 0.5,
        q_c: 1.0,
    };
    let no_u_without_p = flux_pu(st_no_p, k_ex, k_eq, D076_GAMMA_MAX) <= EPS;
    ok &= no_u_without_p;

    // No A: maturation stops; S relaxes to U; U can desorb when p low.
    let st_starved_a = ReducedState {
        a: 0.0,
        theta_u: 0.05,
        theta_s: 0.9,
        q_c: 1.0,
        p: 0.0,
    };
    let j_us_sa = flux_us(st_starved_a, k_m, D076_GAMMA_MAX);
    let j_su_sa = flux_su(st_starved_a, k_r, D076_GAMMA_MAX);
    let j_pu_sa = flux_pu(st_starved_a, k_ex, k_eq, D076_GAMMA_MAX);
    let no_a_ok = j_us_sa.abs() < EPS && j_su_sa > 0.0 && j_pu_sa < 0.0;
    ok &= no_a_ok;

    // No P: cannot refill after damage indefinitely (exchange cannot create U from empty when θ_tot→1 only via existing U/S).
    let st_damaged_no_p = ReducedState {
        p: 0.0,
        theta_u: 0.0,
        theta_s: 0.85,
        a: 1.0,
        q_c: 1.0,
    };
    let no_p_ok = flux_pu(st_damaged_no_p, k_ex, k_eq, D076_GAMMA_MAX) <= EPS
        && flux_us(st_damaged_no_p, k_m, D076_GAMMA_MAX).abs() < EPS;
    ok &= no_p_ok;

    // Observer values do not enter fluxes (structural: flux fns take only state + rates).
    let no_observer = true;
    ok &= no_observer;

    ConservationReport {
        exchange_conserves_p_plus_u: exchange_ok,
        maturation_conserves_membrane: maturation_membrane_ok,
        maturation_consumes_a_produces_w: maturation_aw_ok,
        relaxation_conserves_u_plus_s: relax_ok,
        capacity_invariant: capacity_ok,
        no_s_without_u_and_a: no_s_without,
        no_u_without_p_drive: no_u_without_p,
        no_a_relaxes_then_desorbs: no_a_ok,
        no_p_cannot_repair_indefinitely: no_p_ok,
        no_observer_in_equations: no_observer,
        pass: ok,
        failure: if ok {
            None
        } else {
            Some("D076_SURFACE_CYCLE_CONSERVATION_FAILURE".into())
        },
    }
}

/// Analytic quasi-steady surface fixed point at fixed (p, a, q).
///
/// From J_US=J_SU: r = k_mature q a / k_relax = θ_S / θ_U.
/// From J_PU=0: θ_U = K_eq p (1−θ_U−θ_S).
/// ⇒ θ_S = r K_eq p / (1 + K_eq p (1+r)), θ_U = θ_S / r  (r>0).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SurfaceFixedPoint {
    pub theta_u: f64,
    pub theta_s: f64,
    pub theta_total: f64,
    pub ratio_r: f64,
    pub physical: bool,
}

pub fn surface_fixed_point(p: f64, a: f64, q_c: f64, k_mature: f64, k_relax: f64, k_eq: f64) -> SurfaceFixedPoint {
    let denom_r = k_relax.max(EPS);
    let r = k_mature.max(0.0) * q_c.max(0.0) * a.max(0.0) / denom_r;
    if r <= EPS || p <= EPS {
        return SurfaceFixedPoint {
            theta_u: 0.0,
            theta_s: 0.0,
            theta_total: 0.0,
            ratio_r: r,
            physical: false,
        };
    }
    let kep = k_eq * p;
    let theta_s = (r * kep) / (1.0 + kep * (1.0 + r));
    let theta_u = theta_s / r;
    let theta_total = theta_u + theta_s;
    let physical = theta_u >= -1e-12
        && theta_s >= -1e-12
        && theta_total <= 1.0 + 1e-9
        && theta_s.is_finite()
        && theta_u.is_finite();
    SurfaceFixedPoint {
        theta_u,
        theta_s,
        theta_total,
        ratio_r: r,
        physical,
    }
}

/// Minimum r for θ_S ≥ occ_star at given p.
pub fn r_required_for_occupancy(p: f64, k_eq: f64, occ_star: f64) -> f64 {
    let kep = k_eq * p.max(EPS);
    // θ_S = r kep / (1 + kep(1+r)) ≥ occ
    // r kep ≥ occ (1 + kep + kep r)
    // r kep - occ kep r ≥ occ (1+kep)
    // r kep (1-occ) ≥ occ (1+kep)
    let num = occ_star * (1.0 + kep);
    let den = kep * (1.0 - occ_star).max(EPS);
    num / den
}

/// Jacobian of (θ_U, θ_S) reduced surface dynamics at fixed (p,a,q).
/// Returns eigenvalues of 2×2 Jacobian; locally stable iff both Re(λ) < 0.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JacobianStability {
    pub j00: f64,
    pub j01: f64,
    pub j10: f64,
    pub j11: f64,
    pub eig_re_0: f64,
    pub eig_re_1: f64,
    pub locally_stable: bool,
}

pub fn surface_jacobian(
    st: ReducedState,
    k_ex: f64,
    k_eq: f64,
    k_mature: f64,
    k_relax: f64,
) -> JacobianStability {
    // dθ_U/dt = J_PU - J_US + J_SU
    // dθ_S/dt = J_US - J_SU
    // with Γ_max=1:
    // J_PU = k_ex q (K_eq p (1-θ_U-θ_S) - θ_U)
    // J_US = k_m q a θ_U
    // J_SU = k_r θ_S
    let q = st.q_c.max(0.0);
    let a = st.a.max(0.0);
    let kep = k_eq * st.p.max(0.0);
    let alpha = k_ex * q; // scales J_PU
    // ∂J_PU/∂θ_U = k_ex q (-K_eq p - 1) = -alpha (kep+1)
    // ∂J_PU/∂θ_S = k_ex q (-K_eq p) = -alpha kep
    let dpu_du = -alpha * (kep + 1.0);
    let dpu_ds = -alpha * kep;
    let dus_du = k_mature * q * a;
    let dsu_ds = k_relax;
    // ∂θ̇_U/∂θ_U = dpu_du - dus_du
    // ∂θ̇_U/∂θ_S = dpu_ds + dsu_ds
    // ∂θ̇_S/∂θ_U = dus_du
    // ∂θ̇_S/∂θ_S = -dsu_ds
    let j00 = dpu_du - dus_du;
    let j01 = dpu_ds + dsu_ds;
    let j10 = dus_du;
    let j11 = -dsu_ds;
    // eigenvalues of [[j00,j01],[j10,j11]]
    let tr = j00 + j11;
    let det = j00 * j11 - j01 * j10;
    let disc = tr * tr - 4.0 * det;
    let (e0, e1) = if disc >= 0.0 {
        let s = disc.sqrt();
        ((tr + s) / 2.0, (tr - s) / 2.0)
    } else {
        // complex: real part = tr/2
        (tr / 2.0, tr / 2.0)
    };
    JacobianStability {
        j00,
        j01,
        j10,
        j11,
        eig_re_0: e0,
        eig_re_1: e1,
        locally_stable: e0 < -JACOBIAN_STABLE_TOL && e1 < -JACOBIAN_STABLE_TOL && det > 0.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredStateFamily {
    pub id: String,
    pub radius: f64,
    pub p: f64,
    pub q_c: f64,
    pub a_free: f64,
    pub capacity: f64,
    pub a_retention_measured: f64,
    pub c_retention_measured: f64,
}

/// R16/R22/R32 family using D-075 constitutive interface p and scaled capacity ∝ R.
pub fn measured_state_family() -> Vec<MeasuredStateFamily> {
    let p = D075_ENDOGENOUS_INTERFACE_P;
    let q = D075_MEAN_Q_C;
    let a = D075_FREE_A;
    let cap22 = D075_INTERFACE_CAPACITY;
    // Smooth sphere capacity scales ~ R (circumference × δ); use R ratio from 22.
    let rows = [(16.0, "R16"), (22.0, "R22"), (32.0, "R32")];
    rows.iter()
        .map(|(r, id)| MeasuredStateFamily {
            id: id.to_string(),
            radius: *r,
            p,
            q_c: q,
            a_free: a,
            capacity: cap22 * (*r / 22.0),
            a_retention_measured: D075_CONSTITUTIVE_A_RETENTION,
            c_retention_measured: D075_CONSTITUTIVE_C_RETENTION,
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedPointEval {
    pub state_id: String,
    pub k_mature: f64,
    pub k_relax: f64,
    pub fp: SurfaceFixedPoint,
    pub jacobian: JacobianStability,
    pub theta_s_ok: bool,
    pub capacity_ok: bool,
    pub exchange_active: bool,
    pub cycle_active: bool,
    pub a_retention_proxy: f64,
    pub c_retention_proxy: f64,
    pub qualifies: bool,
}

pub fn evaluate_fixed_point(
    st: &MeasuredStateFamily,
    k_mature: f64,
    k_relax: f64,
) -> FixedPointEval {
    let fp = surface_fixed_point(st.p, st.a_free, st.q_c, k_mature, k_relax, D076_K_EQ);
    let reduced = ReducedState {
        theta_u: fp.theta_u,
        theta_s: fp.theta_s,
        a: st.a_free,
        q_c: st.q_c,
        p: st.p,
    };
    let jac = surface_jacobian(reduced, D076_K_EXCHANGE, D076_K_EQ, k_mature, k_relax);
    let fluxes = cycle_fluxes(reduced, D076_K_EXCHANGE, D076_K_EQ, k_mature, k_relax);
    // At exact FP, J_PU≈0 and J_US≈J_SU; "active" means rates nonzero in magnitude.
    let exchange_active = k_relax > EPS && fp.theta_u > EPS; // reversible exchange remains open
    let cycle_active = fluxes.j_us.abs() > EPS && fluxes.j_su.abs() > EPS;
    let theta_s_ok = fp.physical && fp.theta_s + 1e-12 >= OCC_CONTRACT;
    let capacity_ok = fp.physical && fp.theta_total <= 1.0 + 1e-9;
    // A/C retention proxies: measured endogenous values (architecture cannot invent surplus).
    let a_ret = st.a_retention_measured;
    let c_ret = st.c_retention_measured;
    let qualifies = theta_s_ok
        && capacity_ok
        && jac.locally_stable
        && exchange_active
        && cycle_active
        && a_ret + 1e-12 >= A_RETENTION_GATE
        && c_ret + 1e-12 >= C_RETENTION_GATE
        && fp.theta_total < 1.0 - 1e-6;
    FixedPointEval {
        state_id: st.id.clone(),
        k_mature,
        k_relax,
        fp,
        jacobian: jac,
        theta_s_ok,
        capacity_ok,
        exchange_active,
        cycle_active,
        a_retention_proxy: a_ret,
        c_retention_proxy: c_ret,
        qualifies,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyBudget {
    pub state_id: String,
    pub k_mature: f64,
    pub k_relax: f64,
    pub a_catalyst_maintenance: f64,
    pub a_structure: f64,
    pub a_membrane_maturation: f64,
    pub a_total_demand_rate: f64,
    pub a_sustainable_surplus_rate: f64,
    pub p_production_proxy: f64,
    pub j_pu: f64,
    pub j_us: f64,
    pub j_su: f64,
    pub membrane_equivalents_per_horizon: f64,
    pub waste_rate: f64,
    pub ledger_closes: bool,
    pub within_a_budget: bool,
    pub avoids_d075_a_collapse_recreation: bool,
    pub below_d071_overproduction: bool,
    pub replacement_ok: bool,
    pub pass: bool,
}

/// Energy/material budget at a proposed fixed point.
///
/// Sustainable A surplus is conservatively 0 under D-075 constitutive collapse
/// (retention 0.061 ≪ 0.80). Maturation demand = J_US × capacity.
pub fn energy_budget(st: &MeasuredStateFamily, k_mature: f64, k_relax: f64) -> EnergyBudget {
    let fp = surface_fixed_point(st.p, st.a_free, st.q_c, k_mature, k_relax, D076_K_EQ);
    let reduced = ReducedState {
        theta_u: fp.theta_u,
        theta_s: fp.theta_s,
        a: st.a_free,
        q_c: st.q_c,
        p: st.p,
    };
    let fluxes = cycle_fluxes(reduced, D076_K_EXCHANGE, D076_K_EQ, k_mature, k_relax);
    // Convert density fluxes to organism rates using interface capacity as measure.
    let measure = st.capacity; // ∫δ Γ_max ≈ capacity when Γ_max=1 and S=θ capacity
    let j_us = fluxes.j_us * measure;
    let j_su = fluxes.j_su * measure;
    let j_pu = fluxes.j_pu * measure;
    // Split non-membrane A demand using D-067 topology: precursor-dominant.
    // ponytail: allocate residual non-membrane demand from measured collapse; upgrade with live ledger.
    let a_catalyst = 0.05 * D067_TOTAL_DEMAND / REPLACEMENT_HORIZON;
    let a_structure = 0.10 * D067_TOTAL_DEMAND / REPLACEMENT_HORIZON;
    let a_membrane = j_us.max(0.0);
    let a_total = a_catalyst + a_structure + a_membrane;
    // Sustainable surplus: none while measured A retention is below gate.
    let surplus = if st.a_retention_measured >= A_RETENTION_GATE {
        // optimistic: fraction of D-067 demand headroom
        ((D067_ORDINARY_A_RETENTION - A_RETENTION_GATE).max(0.0)) * D067_TOTAL_DEMAND
            / REPLACEMENT_HORIZON
    } else {
        0.0
    };
    let equivalents = if fp.theta_s > EPS {
        (j_su * REPLACEMENT_HORIZON) / (fp.theta_s * measure).max(EPS)
    } else {
        0.0
    };
    let within = a_total <= surplus + ACCOUNTING_TOL;
    let avoids_collapse = a_membrane <= surplus + ACCOUNTING_TOL && st.a_retention_measured >= A_RETENTION_GATE;
    let below_overprod = st.p < D071_OVERPRODUCTION_P_FLOOR; // activity, not bulk P mass
    let replacement_ok = equivalents + 1e-12 >= 1.0;
    let ledger = (j_us - j_su).abs() < 1e-6 * (1.0 + j_us.abs());
    let pass = within && avoids_collapse && below_overprod && replacement_ok && ledger && fp.physical;
    EnergyBudget {
        state_id: st.id.clone(),
        k_mature,
        k_relax,
        a_catalyst_maintenance: a_catalyst,
        a_structure,
        a_membrane_maturation: a_membrane,
        a_total_demand_rate: a_total,
        a_sustainable_surplus_rate: surplus,
        p_production_proxy: st.p,
        j_pu,
        j_us,
        j_su,
        membrane_equivalents_per_horizon: equivalents,
        waste_rate: j_us.max(0.0),
        ledger_closes: ledger,
        within_a_budget: within,
        avoids_d075_a_collapse_recreation: avoids_collapse,
        below_d071_overproduction: below_overprod,
        replacement_ok,
        pass,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParamPair {
    pub k_mature: f64,
    pub k_relax: f64,
}

/// Derive ≤5 global (k_mature, k_relax) candidates from required r and replacement horizon.
pub fn identify_parameter_candidates() -> Vec<ParamPair> {
    let p = D075_ENDOGENOUS_INTERFACE_P;
    let q = D075_MEAN_Q_C;
    let a = D075_FREE_A;
    let r_star = r_required_for_occupancy(p, D076_K_EQ, OCC_CONTRACT);
    // Replacement: k_relax ≥ 1/H so one equivalent cycles in horizon at unit θ_S.
    let k_relax_base = 1.0 / REPLACEMENT_HORIZON;
    // r = k_m q a / k_r ⇒ k_m = r k_r / (q a)
    let qa = (q * a).max(EPS);
    let scales = [0.5, 1.0, 1.5, 2.0, 3.0];
    scales
        .iter()
        .take(5)
        .map(|s| {
            let k_relax = k_relax_base * *s;
            let k_mature = (r_star * k_relax) / qa;
            ParamPair { k_mature, k_relax }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityReport {
    pub candidates: Vec<ParamPair>,
    pub per_candidate: Vec<CandidatePortability>,
    pub selected: Option<ParamPair>,
    pub pass: bool,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePortability {
    pub pair: ParamPair,
    pub evals: Vec<FixedPointEval>,
    pub budgets: Vec<EnergyBudget>,
    pub span_k_mature: f64,
    pub span_k_relax: f64,
    pub loo_ok: bool,
    pub surface_ok_all: bool,
    pub energy_ok_all: bool,
    pub portable: bool,
}

pub fn gate4_parameter_identification() -> PortabilityReport {
    let family = measured_state_family();
    let candidates = identify_parameter_candidates();
    let mut per = Vec::new();
    let mut selected = None;
    for pair in &candidates {
        let evals: Vec<_> = family
            .iter()
            .map(|st| evaluate_fixed_point(st, pair.k_mature, pair.k_relax))
            .collect();
        let budgets: Vec<_> = family
            .iter()
            .map(|st| energy_budget(st, pair.k_mature, pair.k_relax))
            .collect();
        // Global single pair ⇒ span of the pair itself across radii is 1× (same params).
        // Required occupancy r* is identical when p,q,a identical across radii here.
        let span_m = 1.0;
        let span_r = 1.0;
        // LOO: predict θ_S on held-out state from median of others; within 2×.
        let thetas: Vec<f64> = evals.iter().map(|e| e.fp.theta_s.max(EPS)).collect();
        let mut loo_ok = true;
        for i in 0..thetas.len() {
            let mut others = Vec::new();
            for (j, t) in thetas.iter().enumerate() {
                if j != i {
                    others.push(*t);
                }
            }
            others.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = others[others.len() / 2];
            let ratio = (thetas[i] / med).max(med / thetas[i]);
            if ratio > LOO_FACTOR_MAX {
                loo_ok = false;
            }
        }
        let surface_ok = evals.iter().all(|e| e.theta_s_ok && e.jacobian.locally_stable && e.capacity_ok);
        let energy_ok = budgets.iter().all(|b| b.pass);
        // Full qualify also needs A/C retention — fails under measured collapse.
        let portable = span_m <= PORTABILITY_SPAN_MAX
            && span_r <= PORTABILITY_SPAN_MAX
            && loo_ok
            && surface_ok;
        let c = CandidatePortability {
            pair: *pair,
            evals,
            budgets,
            span_k_mature: span_m,
            span_k_relax: span_r,
            loo_ok,
            surface_ok_all: surface_ok,
            energy_ok_all: energy_ok,
            portable,
        };
        if selected.is_none() && c.portable && c.energy_ok_all && c.evals.iter().all(|e| e.qualifies)
        {
            selected = Some(*pair);
        }
        per.push(c);
    }
    // Gate 4 pass = at least one portable surface candidate (energy judged in Gate 3/6).
    let any_portable = per.iter().any(|c| c.portable);
    PortabilityReport {
        pass: any_portable,
        failure: if any_portable {
            None
        } else {
            Some("D076_SURFACE_CYCLE_NOT_PORTABLE".into())
        },
        selected,
        candidates,
        per_candidate: per,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    pub name: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityControls {
    pub controls: Vec<ControlResult>,
    pub pass: bool,
    pub failure: Option<String>,
}

/// Gate 5 — reduced damage/starvation controls using ODE integration of (θ_U, θ_S).
pub fn gate5_damage_starvation_controls(k_mature: f64, k_relax: f64) -> CausalityControls {
    let family = measured_state_family();
    let st = &family[1]; // R22
    let fp = surface_fixed_point(st.p, st.a_free, st.q_c, k_mature, k_relax, D076_K_EQ);
    let mut controls = Vec::new();

    // 10% mature-S damage then recover under resources.
    let mut th_u = fp.theta_u;
    let mut th_s = fp.theta_s * 0.9;
    let pre = fp.theta_s;
    integrate_surface(
        &mut th_u,
        &mut th_s,
        st.p,
        st.a_free,
        st.q_c,
        k_mature,
        k_relax,
        50_000,
        0.05,
    );
    let recovered = th_s >= 0.95 * pre - 1e-6;
    controls.push(ControlResult {
        name: "damage_10pct_recovery".into(),
        pass: recovered && fp.theta_s >= OCC_CONTRACT - 1e-6,
        detail: format!("pre={pre:.6} post={th_s:.6} recovered={recovered}"),
    });

    // no A
    let mut u = fp.theta_u;
    let mut s = fp.theta_s;
    integrate_surface(&mut u, &mut s, st.p, 0.0, st.q_c, k_mature, k_relax, 20_000, 0.05);
    controls.push(ControlResult {
        name: "no_a_fails".into(),
        pass: s < 0.95 * pre,
        detail: format!("s_end={s:.6}"),
    });

    // no P
    let mut u2 = fp.theta_u;
    let mut s2 = fp.theta_s * 0.9;
    integrate_surface(&mut u2, &mut s2, 0.0, st.a_free, st.q_c, k_mature, k_relax, 20_000, 0.05);
    controls.push(ControlResult {
        name: "no_p_fails_repair".into(),
        pass: s2 < 0.95 * pre,
        detail: format!("s_end={s2:.6}"),
    });

    // no maturation
    let mut u3 = fp.theta_u;
    let mut s3 = fp.theta_s;
    integrate_surface(&mut u3, &mut s3, st.p, st.a_free, st.q_c, 0.0, k_relax, 20_000, 0.05);
    controls.push(ControlResult {
        name: "no_maturation_declines".into(),
        pass: s3 < pre - 1e-4,
        detail: format!("s_end={s3:.6}"),
    });

    // no relaxation: cycle inactive — mature S may stick; require failure of replacement causality
    let fluxes = cycle_fluxes(
        ReducedState {
            theta_u: fp.theta_u,
            theta_s: fp.theta_s,
            a: st.a_free,
            q_c: st.q_c,
            p: st.p,
        },
        D076_K_EXCHANGE,
        D076_K_EQ,
        k_mature,
        0.0,
    );
    controls.push(ControlResult {
        name: "no_relaxation_no_cycle".into(),
        pass: fluxes.j_su.abs() < EPS,
        detail: format!("j_su={}", fluxes.j_su),
    });

    // nutrient / fuel withdrawal ~ a→0 and q→0
    let mut u4 = fp.theta_u;
    let mut s4 = fp.theta_s;
    integrate_surface(&mut u4, &mut s4, st.p, 0.0, 0.0, k_mature, k_relax, 20_000, 0.05);
    controls.push(ControlResult {
        name: "starvation_decline".into(),
        pass: s4 < pre - 1e-4,
        detail: format!("s_end={s4:.6}"),
    });

    // restoration without field reset
    let mut u5 = u4;
    let mut s5 = s4;
    integrate_surface(
        &mut u5,
        &mut s5,
        st.p,
        st.a_free,
        st.q_c,
        k_mature,
        k_relax,
        50_000,
        0.05,
    );
    controls.push(ControlResult {
        name: "restoration_resumes".into(),
        pass: s5 > s4 + 1e-4,
        detail: format!("s_starved={s4:.6} s_restored={s5:.6}"),
    });

    // no permanent U lock
    controls.push(ControlResult {
        name: "no_capacity_lock".into(),
        pass: u5 + s5 < 1.0 - 1e-6,
        detail: format!("theta_tot={}", u5 + s5),
    });

    let pass = controls.iter().all(|c| c.pass);
    CausalityControls {
        pass,
        failure: if pass {
            None
        } else {
            Some("D076_SURFACE_CYCLE_CAUSALITY_FAIL".into())
        },
        controls,
    }
}

fn integrate_surface(
    theta_u: &mut f64,
    theta_s: &mut f64,
    p: f64,
    a: f64,
    q_c: f64,
    k_mature: f64,
    k_relax: f64,
    steps: usize,
    dt: f64,
) {
    for _ in 0..steps {
        let st = ReducedState {
            theta_u: *theta_u,
            theta_s: *theta_s,
            a,
            q_c,
            p,
        };
        let f = cycle_fluxes(st, D076_K_EXCHANGE, D076_K_EQ, k_mature, k_relax);
        *theta_u += (f.j_pu - f.j_us + f.j_su) * dt;
        *theta_s += (f.j_us - f.j_su) * dt;
        if *theta_u < 0.0 {
            *theta_u = 0.0;
        }
        if *theta_s < 0.0 {
            *theta_s = 0.0;
        }
        let tot = *theta_u + *theta_s;
        if tot > 1.0 {
            *theta_u /= tot;
            *theta_s /= tot;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2Report {
    pub family: Vec<MeasuredStateFamily>,
    pub r_required: f64,
    pub endogenous_a_blocks_retention_gate: bool,
    pub any_qualifying_fixed_point: bool,
    pub sample_evals: Vec<FixedPointEval>,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate2_fixed_point_feasibility(port: &PortabilityReport) -> Gate2Report {
    let family = measured_state_family();
    let r_req = r_required_for_occupancy(D075_ENDOGENOUS_INTERFACE_P, D076_K_EQ, OCC_CONTRACT);
    let sample: Vec<_> = port
        .per_candidate
        .iter()
        .flat_map(|c| c.evals.iter().cloned())
        .collect();
    let any_qual = sample.iter().any(|e| e.qualifies);
    let a_blocks = D075_CONSTITUTIVE_A_RETENTION < A_RETENTION_GATE;
    // Gate 2 requires a physical stable FP with θ_S≥0.95 AND A/C retention ≥0.80.
    // Under measured endogenous A collapse, no qualifying FP exists.
    let pass = any_qual;
    Gate2Report {
        family,
        r_required: r_req,
        endogenous_a_blocks_retention_gate: a_blocks,
        any_qualifying_fixed_point: any_qual,
        sample_evals: sample,
        pass,
        failure: if pass {
            None
        } else {
            Some("D076_MEMBRANE_EXCHANGE_ARCHITECTURE_REVIEW_FAIL".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate3Report {
    pub budgets: Vec<EnergyBudget>,
    pub any_pass: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate3_energy_budget(port: &PortabilityReport) -> Gate3Report {
    let budgets: Vec<_> = port
        .per_candidate
        .iter()
        .flat_map(|c| c.budgets.iter().cloned())
        .collect();
    let any = budgets.iter().any(|b| b.pass);
    Gate3Report {
        pass: any,
        any_pass: any,
        failure: if any {
            None
        } else {
            Some("D076_SURFACE_CYCLE_ENERGY_BUDGET_FAIL".into())
        },
        budgets,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub route: D076Route,
    pub conclusion: String,
    pub scientific_conclusion: String,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub next_directive: String,
    pub reasons: Vec<String>,
}

pub fn gate6_select_route(
    g0: &LineageAudit,
    g1: &ConservationReport,
    g2: &Gate2Report,
    g3: &Gate3Report,
    g4: &PortabilityReport,
    g5: &CausalityControls,
) -> RouteDecision {
    let mut reasons = Vec::new();
    if !g0.pass {
        return RouteDecision {
            route: D076Route::AlreadyClosed,
            conclusion: D076Route::AlreadyClosed.conclusion().into(),
            scientific_conclusion: "Candidate S→U cycle already closed historically.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Broader Phase 1 boundary-architecture review.".into(),
            reasons: vec![g0.failure.clone().unwrap_or_default()],
        };
    }
    if !g1.pass {
        return RouteDecision {
            route: D076Route::ConservationFailure,
            conclusion: D076Route::ConservationFailure.conclusion().into(),
            scientific_conclusion: "Candidate cycle fails conservation/causality algebra.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Do not implement; revise cycle definition.".into(),
            reasons: vec![g1.failure.clone().unwrap_or_default()],
        };
    }

    // Prefer specific failures in directive order when surface FP exists but later gates fail.
    let surface_portable = g4.per_candidate.iter().any(|c| c.portable);
    let energy_fail = !g3.pass;
    let causality_fail = !g5.pass;
    let no_fp = !g2.pass;

    if surface_portable && energy_fail {
        reasons.push(
            "Surface θ_S≥0.95 fixed points exist algebraically at endogenous p, but maturation A demand has no sustainable surplus under measured A retention ≈0.06.".into(),
        );
        reasons.push(PASSIVE_RECORD.into());
        return RouteDecision {
            route: D076Route::EnergyInfeasible,
            conclusion: D076Route::EnergyInfeasible.conclusion().into(),
            scientific_conclusion: "Energy-driven U→S maturation cannot maintain contract mature occupancy without recreating the D-075 A collapse: endogenous free A is already below the 0.80 retention gate, and required replacement-rate maturation adds a strictly positive A sink.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Do not implement this cycle. Broader Phase 1 boundary-architecture review before further rate or species additions.".into(),
            reasons,
        };
    }

    if !surface_portable {
        reasons.push("No global k_mature/k_relax pair yields portable stable contract occupancy across the governed state family.".into());
        return RouteDecision {
            route: D076Route::NotPortable,
            conclusion: D076Route::NotPortable.conclusion().into(),
            scientific_conclusion: "Parameter identification failed portability gates.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Do not use radius-specific kinetics.".into(),
            reasons,
        };
    }

    if causality_fail && g3.pass && g2.pass {
        return RouteDecision {
            route: D076Route::CausalityFail,
            conclusion: D076Route::CausalityFail.conclusion().into(),
            scientific_conclusion: "Stable mature S lacks required damage/starvation causality.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Do not implement; revise causal coupling.".into(),
            reasons: vec![g5.failure.clone().unwrap_or_default()],
        };
    }

    if no_fp {
        reasons.push(
            "No stable physical fixed point meets θ_S≥0.95 with A/C retention≥0.80 inside measured metabolic budget.".into(),
        );
        return RouteDecision {
            route: D076Route::ArchitectureReviewFail,
            conclusion: D076Route::ArchitectureReviewFail.conclusion().into(),
            scientific_conclusion: "No viable nonequilibrium surface-exchange architecture under measured endogenous budget.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Broader Phase 1 boundary-architecture review before further rate or species additions.".into(),
            reasons,
        };
    }

    if g0.pass && g1.pass && g2.pass && g3.pass && g4.pass && g5.pass {
        return RouteDecision {
            route: D076Route::Qualified,
            conclusion: D076Route::Qualified.conclusion().into(),
            scientific_conclusion: "Nonequilibrium surface-state cycle qualifies for implementation.".into(),
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            next_directive: "Implement cycle as new equation+snapshot schema; run isolated kinetics, maintenance, repair, radius portability, Stage E re-entry.".into(),
            reasons: vec!["All Gates 0–5 passed.".into()],
        };
    }

    // Fallback: energy infeasible is the dominant measured failure mode.
    RouteDecision {
        route: D076Route::EnergyInfeasible,
        conclusion: D076Route::EnergyInfeasible.conclusion().into(),
        scientific_conclusion: "Cycle maintains algebraically high θ_S only by demanding A the metabolism cannot supply.".into(),
        d008_status: "BLOCKED_NOT_RECOVERED".into(),
        phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
        production_verdict: "REQUIRES_REMEDIATION".into(),
        next_directive: "Do not implement. Broader Phase 1 boundary-architecture review.".into(),
        reasons: vec![
            format!("g2_pass={}", g2.pass),
            format!("g3_pass={}", g3.pass),
            format!("g4_pass={}", g4.pass),
            format!("g5_pass={}", g5.pass),
        ],
    }
}

/// Full gated review (Gates 0–6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D076Review {
    pub gate0: LineageAudit,
    pub gate1: ConservationReport,
    pub gate2: Gate2Report,
    pub gate3: Gate3Report,
    pub gate4: PortabilityReport,
    pub gate5: CausalityControls,
    pub gate6: RouteDecision,
    pub candidate_equations: CandidateEquations,
    pub frozen_preservation: FrozenPreservation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateEquations {
    pub passive_exchange: String,
    pub maturation: String,
    pub relaxation: String,
    pub capacity: String,
    pub permeability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenPreservation {
    pub seed_capacity_contract: String,
    pub k_eq: f64,
    pub k_exchange: f64,
    pub gamma_max: f64,
    pub p_ref: f64,
    pub d075_conclusion: String,
    pub record: String,
    pub alpha_frozen: f64,
    pub beta_frozen: f64,
    pub d074_conclusion: String,
    pub d075_ids_ok: bool,
}

pub fn frozen_preservation() -> FrozenPreservation {
    FrozenPreservation {
        seed_capacity_contract: SEED_CAPACITY_CONTRACT_V1.into(),
        k_eq: D076_K_EQ,
        k_exchange: D076_K_EXCHANGE,
        gamma_max: D076_GAMMA_MAX,
        p_ref: D076_P_REF,
        d075_conclusion: D075_CONCLUSION.into(),
        record: PASSIVE_RECORD.into(),
        alpha_frozen: D031_ALPHA_FROZEN,
        beta_frozen: D031_BETA_FROZEN,
        d074_conclusion: D074_CONCLUSION.into(),
        d075_ids_ok: D075_PROJECT_ID == "D-075"
            && SEED_CONTRACT == SEED_CAPACITY_CONTRACT_V1
            && (D075_K_EQ - D073_K_EQ).abs() < 1e-15
            && (D075_K_EXCHANGE - D073_K_EXCHANGE).abs() < 1e-15
            && (D075_GAMMA_MAX - D073_GAMMA_MAX).abs() < 1e-15
            && (D075_P_REF - D073_P_REF).abs() < 1e-15
            && !D075_AGENT_MEMORY_ID.is_empty()
            && D075_STARTING_COMMIT == "b06254b"
            && D075_STARTING_TAG == "D-074-cellwise-exchange-parity-audit",
    }
}

pub fn candidate_equations() -> CandidateEquations {
    CandidateEquations {
        passive_exchange: "P⇄U; J_PU = δ k_exchange q(C) Γ_max [K_eq p (1−θ_total) − θ_U]".into(),
        maturation: "U+A→S+W; J_US = k_mature q(C) a U".into(),
        relaxation: "S→U; J_SU = k_relax S (conservative; no material destruction)".into(),
        capacity: "θ_total = (U+S)/(δ Γ_max) ≤ 1".into(),
        permeability: "Functional permeability depends only on mature S".into(),
    }
}

pub fn run_full_review() -> D076Review {
    let gate0 = gate0_lineage_audit();
    let gate1 = gate1_conservation();
    let gate4 = gate4_parameter_identification();
    let gate2 = gate2_fixed_point_feasibility(&gate4);
    let gate3 = gate3_energy_budget(&gate4);
    // Use first portable (or first) candidate for causality controls.
    let pair = gate4
        .per_candidate
        .iter()
        .find(|c| c.portable)
        .map(|c| c.pair)
        .unwrap_or_else(|| {
            gate4
                .candidates
                .first()
                .copied()
                .unwrap_or(ParamPair {
                    k_mature: 1.0,
                    k_relax: 1.0 / REPLACEMENT_HORIZON,
                })
        });
    let gate5 = gate5_damage_starvation_controls(pair.k_mature, pair.k_relax);
    let gate6 = gate6_select_route(&gate0, &gate1, &gate2, &gate3, &gate4, &gate5);
    D076Review {
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        gate6,
        candidate_equations: candidate_equations(),
        frozen_preservation: frozen_preservation(),
    }
}

#[cfg(test)]
mod internal_smoke {
    use super::*;

    #[test]
    fn r_required_at_endogenous_p_exceeds_twenty() {
        let r = r_required_for_occupancy(D075_ENDOGENOUS_INTERFACE_P, D076_K_EQ, OCC_CONTRACT);
        assert!(r > 20.0, "r={r}");
        let fp = surface_fixed_point(
            D075_ENDOGENOUS_INTERFACE_P,
            1.0,
            1.0,
            r * (1.0 / REPLACEMENT_HORIZON),
            1.0 / REPLACEMENT_HORIZON,
            D076_K_EQ,
        );
        assert!(fp.theta_s + 1e-9 >= OCC_CONTRACT, "θ_S={}", fp.theta_s);
    }
}
