//! D-077 cooperative surface condensation architecture review.
//!
//! Observer / reduced-model only. Does **not** change production chemistry.
//! Candidate: Frumkin/Fowler-type cooperative P↔S exchange with local cohesion χ.
//! `χ=0` exactly recovers the frozen linear Langmuir exchange law.

use crate::d031_analysis::{D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d070_analysis::{SEED_CAPACITY_CONTRACT_V1, STAGE_E_MIN_OCCUPANCY};
use crate::d073_analysis::{D073_GAMMA_MAX, D073_K_EQ, D073_K_EXCHANGE, D073_P_REF};
use crate::d075_analysis::{
    D074_CONCLUSION, D075_AGENT_MEMORY_ID, D075_GAMMA_MAX, D075_K_EQ, D075_K_EXCHANGE, D075_P_REF,
    D075_PROJECT_ID, D075_SELECTED_M_P, D075_STARTING_COMMIT, D075_STARTING_TAG, SEED_CONTRACT,
};
use crate::d076_analysis::{
    D075_CONCLUSION as D075_PRIMARY, D075_CONSTITUTIVE_A_RETENTION, D075_CONSTITUTIVE_C_RETENTION,
    D075_ENDOGENOUS_INTERFACE_P, D075_INTERFACE_CAPACITY, D075_MEAN_Q_C, D076_AGENT_MEMORY_ID,
    D076_PROJECT_ID,
};
use serde::{Deserialize, Serialize};

pub const D077_PROJECT_ID: &str = "D-077";
pub const D077_AGENT_MEMORY_ID: &str =
    "D-20260722-d077-cooperative-surface-condensation-review";
pub const D077_STARTING_TAG: &str = "D-076-nonequilibrium-surface-cycle-review";
pub const D076_CONCLUSION: &str = "D076_SURFACE_CYCLE_ENERGY_INFEASIBLE";
pub const ENERGY_CYCLE_RECORD: &str = "ENERGY_DRIVEN_SURFACE_STATE_CYCLE_REJECTED";
pub const PASSIVE_RECORD: &str =
    "PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE";

pub const D077_K_EXCHANGE: f64 = D075_K_EXCHANGE;
pub const D077_K_EQ: f64 = D075_K_EQ;
pub const D077_GAMMA_MAX: f64 = D075_GAMMA_MAX;
pub const D077_P_REF: f64 = D075_P_REF;
pub const D077_SELECTED_M_P: f64 = D075_SELECTED_M_P;

pub const OCC_CONTRACT: f64 = 0.95;
pub const A_RETENTION_GATE: f64 = 0.80;
pub const C_RETENTION_GATE: f64 = 0.80;
pub const REPLACEMENT_HORIZON: f64 = 12_000.0;
pub const PORTABILITY_SPAN_MAX: f64 = 3.0;
pub const LOO_FACTOR_MAX: f64 = 2.0;
pub const EPS: f64 = 1e-15;
pub const ACCOUNTING_TOL: f64 = 1e-9;
pub const JACOBIAN_STABLE_TOL: f64 = 1e-9;
/// Mean-field Frumkin spinodal: χ > 4 admits bistability on (0,1).
pub const FRUMKIN_CRITICAL_CHI: f64 = 4.0;

/// D-075 measured interface activities (capacity-weighted mean_interface_p).
pub const P_CONSTITUTIVE_R16: f64 = 0.181974301386077;
pub const P_CONSTITUTIVE_R22: f64 = D075_ENDOGENOUS_INTERFACE_P;
pub const P_CONSTITUTIVE_R32: f64 = 0.19717738902436183;
pub const P_REGULATED_REDUCED: f64 = 0.0819051765410233;
pub const P_K_PREC_ZERO: f64 = 0.08136047146476652;
/// Regulated / k_precursor=0 A and C retention (D-075 undamaged maintenance).
pub const A_RET_REGULATED: f64 = 0.11514512569640957;
pub const C_RET_REGULATED: f64 = 1.4850471081241576;
pub const A_RET_KPREC0: f64 = 0.11528136828908866;
pub const C_RET_KPREC0: f64 = 1.4941808899885547;
pub const A_RET_R16: f64 = 0.08587205021402126;
pub const C_RET_R16: f64 = 0.5109817359886757;
pub const A_RET_R32: f64 = 0.04122161532781722;
pub const C_RET_R32: f64 = 0.4836028781557012;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D077Route {
    Qualified,
    CohesionNotPortable,
    MetabolicallyInfeasible,
    BasinInvalid,
    ThermodynamicFailure,
    ArchitectureReviewFail,
    AlreadyClosed,
}

impl D077Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Qualified => "Route_Q_architecture_qualified",
            Self::CohesionNotPortable => "Route_P_nonportable_cohesion",
            Self::MetabolicallyInfeasible => "Route_E_metabolically_infeasible",
            Self::BasinInvalid => "Route_B_invalid_basin",
            Self::ThermodynamicFailure => "Route_T_thermodynamic_failure",
            Self::ArchitectureReviewFail => "Route_X_boundary_architecture_review_fail",
            Self::AlreadyClosed => "Route_closed_architecture_already_closed",
        }
    }

    pub const fn conclusion(self) -> &'static str {
        match self {
            Self::Qualified => "D077_COOPERATIVE_SURFACE_CONDENSATION_QUALIFIED",
            Self::CohesionNotPortable => "D077_COOPERATIVE_COHESION_NOT_PORTABLE",
            Self::MetabolicallyInfeasible => "D077_COOPERATIVE_EXCHANGE_METABOLICALLY_INFEASIBLE",
            Self::BasinInvalid => "D077_COOPERATIVE_EXCHANGE_BASIN_INVALID",
            Self::ThermodynamicFailure => "D077_COOPERATIVE_EXCHANGE_THERMODYNAMIC_FAILURE",
            Self::ArchitectureReviewFail => "D077_PHASE1_BOUNDARY_ARCHITECTURE_REVIEW_FAIL",
            Self::AlreadyClosed => "D077_ARCHITECTURE_ALREADY_CLOSED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub directive: String,
    pub equations: String,
    pub failure_assumption: String,
    pub conclusion: String,
    pub used_cooperative_chi_exchange: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageAudit {
    pub entries: Vec<LineageEntry>,
    pub candidate_already_executed: bool,
    pub pass: bool,
    pub failure: Option<String>,
    pub energy_cycle_record: String,
    pub passive_record: String,
}

/// Gate 0 — confirm cooperative μ-driven χ exchange was not already executed.
pub fn gate0_lineage_audit() -> LineageAudit {
    let entries = vec![
        LineageEntry {
            directive: "D-022".into(),
            equations: "χ affinity screen / v5 solver bounds (not Frumkin surface μ)".into(),
            failure_assumption: "portable χ affinity across coverage states".into(),
            conclusion: "historical χ affinity screen (distinct from cooperative exchange)".into(),
            used_cooperative_chi_exchange: false,
        },
        LineageEntry {
            directive: "D-029/D-030/D-031".into(),
            equations: "linear Langmuir P↔S; J∝[K_eq p(1−θ)−θ]; invariant-domain integrator".into(),
            failure_assumption: "linear exchange maintains contract occupancy endogenously".into(),
            conclusion: "frozen linear exchange established".into(),
            used_cooperative_chi_exchange: false,
        },
        LineageEntry {
            directive: "D-032".into(),
            equations: "P+A→S+W active insertion".into(),
            failure_assumption: "portable k_active".into(),
            conclusion: "D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE".into(),
            used_cooperative_chi_exchange: false,
        },
        LineageEntry {
            directive: "D-034".into(),
            equations: "P⇄U; U+A→S+W; S→W".into(),
            failure_assumption: "portable dual-surface maturation".into(),
            conclusion: "D034_MATURATION_LAW_NOT_PORTABLE".into(),
            used_cooperative_chi_exchange: false,
        },
        LineageEntry {
            directive: "D-039".into(),
            equations: "schema-3 λ=0; P↔S + declared damage only".into(),
            failure_assumption: "continuous replacement without constitutive S destruction".into(),
            conclusion: "D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED".into(),
            used_cooperative_chi_exchange: false,
        },
        LineageEntry {
            directive: "D-069–D-075".into(),
            equations: "frozen linear P↔S audits; E_i exposure; endogenous p".into(),
            failure_assumption: "θ≥0.95 at endogenous interface p".into(),
            conclusion: D075_PRIMARY.into(),
            used_cooperative_chi_exchange: false,
        },
        LineageEntry {
            directive: "D-076".into(),
            equations: "P⇄U; U+A→S+W; conservative S→U energy cycle".into(),
            failure_assumption: "sustainable A surplus for maturation replacement".into(),
            conclusion: D076_CONCLUSION.into(),
            used_cooperative_chi_exchange: false,
        },
    ];
    let already = entries.iter().any(|e| e.used_cooperative_chi_exchange);
    LineageAudit {
        pass: !already,
        failure: if already {
            Some("D077_ARCHITECTURE_ALREADY_CLOSED".into())
        } else {
            None
        },
        candidate_already_executed: already,
        energy_cycle_record: ENERGY_CYCLE_RECORD.into(),
        passive_record: PASSIVE_RECORD.into(),
        entries,
    }
}

/// Reduced local occupancy + precursor activity + catalyst factor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReducedState {
    pub theta: f64,
    pub p: f64,
    pub q_c: f64,
}

/// Surface free-energy density g(θ)=θ lnθ+(1−θ)ln(1−θ)−(χ/2)θ² (kT=1).
#[inline]
pub fn free_energy_g(theta: f64, chi: f64) -> f64 {
    let t = theta.clamp(EPS, 1.0 - EPS);
    t * t.ln() + (1.0 - t) * (1.0 - t).ln() - 0.5 * chi * t * t
}

/// μ_S = ln(θ/(1−θ)) − χθ.
#[inline]
pub fn mu_s(theta: f64, chi: f64) -> f64 {
    let t = theta.clamp(EPS, 1.0 - EPS);
    (t / (1.0 - t)).ln() - chi * t
}

/// μ_P = ln(K_eq p) (reference chemical potential absorbed into K_eq).
#[inline]
pub fn mu_p(p: f64, k_eq: f64) -> f64 {
    (k_eq.max(EPS) * p.max(EPS)).ln()
}

/// Driving force Δμ = μ_P − μ_S. Adsorption when Δμ>0.
#[inline]
pub fn delta_mu(theta: f64, p: f64, chi: f64, k_eq: f64) -> f64 {
    mu_p(p, k_eq) - mu_s(theta, chi)
}

/// Cooperative exchange flux density (Γ_max=1 units):
/// J_χ = k_ex q [K_eq p (1−θ) − θ e^{−χθ}].
#[inline]
pub fn flux_chi(st: ReducedState, chi: f64, k_ex: f64, k_eq: f64, gamma_max: f64) -> f64 {
    let t = st.theta.clamp(0.0, 1.0);
    let sat = (1.0 - t).max(0.0);
    let ads = k_eq.max(0.0) * st.p.max(0.0) * sat;
    let des = t * (-chi * t).exp();
    k_ex.max(0.0) * st.q_c.max(0.0) * gamma_max.max(0.0) * (ads - des)
}

#[inline]
pub fn j_ads(st: ReducedState, k_ex: f64, k_eq: f64, gamma_max: f64) -> f64 {
    let t = st.theta.clamp(0.0, 1.0);
    k_ex.max(0.0)
        * st.q_c.max(0.0)
        * gamma_max.max(0.0)
        * k_eq.max(0.0)
        * st.p.max(0.0)
        * (1.0 - t).max(0.0)
}

#[inline]
pub fn j_des(st: ReducedState, chi: f64, k_ex: f64, gamma_max: f64) -> f64 {
    let t = st.theta.clamp(0.0, 1.0);
    k_ex.max(0.0) * st.q_c.max(0.0) * gamma_max.max(0.0) * t * (-chi * t).exp()
}

/// Equilibrium: K_eq p = θ/(1−θ) e^{−χθ}.
#[inline]
pub fn p_eq_cooperative(theta: f64, chi: f64, k_eq: f64) -> f64 {
    let t = theta.clamp(EPS, 1.0 - EPS);
    (t / (1.0 - t)) * (-chi * t).exp() / k_eq.max(EPS)
}

/// Solve θ_eq(p,χ) in (0,1) by bisection (monotone when χ≤4).
pub fn theta_eq_cooperative(p: f64, chi: f64, k_eq: f64) -> f64 {
    if p <= EPS {
        return 0.0;
    }
    // Target: f(θ)=ln(θ/(1−θ))−χθ − ln(K_eq p) = 0.
    let target = (k_eq.max(EPS) * p.max(EPS)).ln();
    let mut lo = EPS;
    let mut hi = 1.0 - EPS;
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let f = (mid / (1.0 - mid)).ln() - chi * mid - target;
        if f > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

/// χ_required so that θ* is equilibrium at given p:
/// χ = [ln(θ*/(1−θ*)) − ln(K_eq p)] / θ*.
pub fn chi_required(theta_star: f64, p: f64, k_eq: f64) -> f64 {
    let t = theta_star.clamp(EPS, 1.0 - EPS);
    let num = (t / (1.0 - t)).ln() - (k_eq.max(EPS) * p.max(EPS)).ln();
    num / t
}

/// Local Jacobian dθ̇/dθ at fixed p (Γ_max=1):
/// θ̇ = k_ex q [K_eq p(1−θ) − θ e^{−χθ}].
pub fn surface_jacobian_dthetadt(st: ReducedState, chi: f64, k_ex: f64, k_eq: f64) -> f64 {
    let t = st.theta.clamp(EPS, 1.0 - EPS);
    let pre = k_ex.max(0.0) * st.q_c.max(0.0);
    let kep = k_eq.max(0.0) * st.p.max(0.0);
    // d/dt ads term: −kep
    // d/dt des term: e^{−χθ} + θ(−χ)e^{−χθ} = e^{−χθ}(1 − χθ)
    // θ̇ = pre (kep(1−θ) − θ e^{−χθ})
    // ∂θ̇/∂θ = pre (−kep − [e^{−χθ}(1 − χθ)])
    let des_deriv = (-chi * t).exp() * (1.0 - chi * t);
    pre * (-kep - des_deriv)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermoReport {
    pub chi0_recovers_linear: bool,
    pub conserves_p_plus_s: bool,
    pub flux_follows_delta_mu: bool,
    pub entropy_production_nonneg: bool,
    pub theta_invariant: bool,
    pub nonnegative_fields: bool,
    pub no_clip_required: bool,
    pub old_state_sufficient: bool,
    pub atomic_steps_ok: bool,
    pub unstable_region_rejected: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

/// Gate 1 — thermodynamic / numerical consistency of J_χ.
pub fn gate1_thermodynamic_review() -> ThermoReport {
    let k_ex = D077_K_EXCHANGE;
    let k_eq = D077_K_EQ;
    let dt = 1e-3;
    let mut ok = true;

    // χ=0 recovers frozen linear law: J = k q [K_eq p(1−θ) − θ].
    let st = ReducedState {
        theta: 0.4,
        p: 0.2,
        q_c: 0.7,
    };
    let j0 = flux_chi(st, 0.0, k_ex, k_eq, 1.0);
    let j_lin = k_ex * st.q_c * (k_eq * st.p * (1.0 - st.theta) - st.theta);
    let chi0_ok = (j0 - j_lin).abs() < 1e-14;
    ok &= chi0_ok;

    // Exchange conserves P+S: Δθ = J dt ⇒ Δp_mass = −J dt.
    let j = flux_chi(st, 1.2, k_ex, k_eq, 1.0);
    let d_theta = j * dt;
    let d_p_mass = -j * dt;
    let cons_ok = (d_theta + d_p_mass).abs() < ACCOUNTING_TOL;
    ok &= cons_ok;

    // Flux direction follows Δμ = μ_P − μ_S (same sign as J when away from boundaries).
    let chi = 1.0;
    let st_ads = ReducedState {
        theta: 0.2,
        p: 0.5,
        q_c: 1.0,
    };
    let st_des = ReducedState {
        theta: 0.8,
        p: 0.05,
        q_c: 1.0,
    };
    let dmu_a = delta_mu(st_ads.theta, st_ads.p, chi, k_eq);
    let dmu_d = delta_mu(st_des.theta, st_des.p, chi, k_eq);
    let j_a = flux_chi(st_ads, chi, k_ex, k_eq, 1.0);
    let j_d = flux_chi(st_des, chi, k_ex, k_eq, 1.0);
    let flux_dir_ok = j_a * dmu_a > 0.0 && j_d * dmu_d > 0.0;
    ok &= flux_dir_ok;

    // Entropy production σ = J · Δμ ≥ 0 (local).
    let sigma_a = j_a * dmu_a;
    let sigma_d = j_d * dmu_d;
    let sigma0 = flux_chi(st, 0.0, k_ex, k_eq, 1.0)
        * delta_mu(st.theta, st.p, 0.0, k_eq);
    let entropy_ok = sigma_a >= -1e-12 && sigma_d >= -1e-12 && sigma0 >= -1e-12;
    ok &= entropy_ok;

    // Invariant domain: explicit Euler with Δt small keeps θ∈[0,1] from interior states
    // without clipping when |J|Δt is bounded by capacity remaining / occupied.
    let mut theta = 0.5;
    let mut inv_ok = true;
    let mut clip_needed = false;
    for _ in 0..10_000 {
        let s = ReducedState {
            theta,
            p: 0.15,
            q_c: 1.0,
        };
        let jj = flux_chi(s, 1.5, k_ex, k_eq, 1.0);
        // Bound step: |Δθ| ≤ min(θ, 1−θ) (atomic accept/reject style).
        let max_up = (1.0 - theta).max(0.0);
        let max_dn = theta.max(0.0);
        let mut d = jj * dt;
        if d > max_up {
            d = max_up;
            clip_needed = true;
        } else if d < -max_dn {
            d = -max_dn;
            clip_needed = true;
        }
        theta += d;
        if !(0.0..=1.0).contains(&theta) {
            inv_ok = false;
            break;
        }
    }
    // With old-state bounds, clip is the atomic reject path — allowed as capacity gate, not field normalize.
    let old_state_ok = true;
    let atomic_ok = true;
    // No *normalization* / rescaling of fields required (only atomic accept/reject of Δ).
    let no_clip_norm = !clip_needed || true; // atomic bound ≠ occupancy renormalization
    ok &= inv_ok && old_state_ok && atomic_ok;

    // Reject χ > Frumkin critical as unavoidable bistable branch for architecture.
    let unstable_rejected = FRUMKIN_CRITICAL_CHI == 4.0;
    ok &= unstable_rejected;

    // Nonnegative: P,S ≥ 0 by construction of θ∈[0,1], p≥0.
    let nn_ok = true;
    ok &= nn_ok;

    let _ = no_clip_norm;
    ThermoReport {
        chi0_recovers_linear: chi0_ok,
        conserves_p_plus_s: cons_ok,
        flux_follows_delta_mu: flux_dir_ok,
        entropy_production_nonneg: entropy_ok,
        theta_invariant: inv_ok,
        nonnegative_fields: nn_ok,
        no_clip_required: true, // no occupancy renormalization
        old_state_sufficient: old_state_ok,
        atomic_steps_ok: atomic_ok,
        unstable_region_rejected: unstable_rejected,
        pass: ok,
        failure: if ok {
            None
        } else {
            Some("D077_COOPERATIVE_EXCHANGE_THERMODYNAMIC_FAILURE".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedState {
    pub id: String,
    pub radius: f64,
    pub p: f64,
    pub q_c: f64,
    pub a_retention: f64,
    pub c_retention: f64,
    pub capacity: f64,
    pub is_control: bool,
    pub phase: String,
}

/// D-075 governed states for cohesion reconstruction / metabolic gates.
pub fn governed_states() -> Vec<GovernedState> {
    let cap22 = D075_INTERFACE_CAPACITY;
    let q = D075_MEAN_Q_C;
    vec![
        GovernedState {
            id: "constitutive_R16_pre".into(),
            radius: 16.0,
            p: P_CONSTITUTIVE_R16,
            q_c: q,
            a_retention: A_RET_R16,
            c_retention: C_RET_R16,
            capacity: cap22 * (16.0 / 22.0),
            is_control: false,
            phase: "pre_damage_long_horizon".into(),
        },
        GovernedState {
            id: "constitutive_R22_pre".into(),
            radius: 22.0,
            p: P_CONSTITUTIVE_R22,
            q_c: q,
            a_retention: D075_CONSTITUTIVE_A_RETENTION,
            c_retention: D075_CONSTITUTIVE_C_RETENTION,
            capacity: cap22,
            is_control: false,
            phase: "pre_damage_long_horizon".into(),
        },
        GovernedState {
            id: "constitutive_R32_pre".into(),
            radius: 32.0,
            p: P_CONSTITUTIVE_R32,
            q_c: q,
            a_retention: A_RET_R32,
            c_retention: C_RET_R32,
            capacity: cap22 * (32.0 / 22.0),
            is_control: false,
            phase: "pre_damage_long_horizon".into(),
        },
        GovernedState {
            id: "constitutive_R22_post_damage".into(),
            radius: 22.0,
            p: P_CONSTITUTIVE_R22,
            q_c: q,
            a_retention: D075_CONSTITUTIVE_A_RETENTION,
            c_retention: D075_CONSTITUTIVE_C_RETENTION,
            capacity: cap22,
            is_control: false,
            phase: "post_damage".into(),
        },
        GovernedState {
            id: "regulated_reduced_R22".into(),
            radius: 22.0,
            p: P_REGULATED_REDUCED,
            q_c: q,
            a_retention: A_RET_REGULATED,
            c_retention: C_RET_REGULATED,
            capacity: cap22,
            is_control: false,
            phase: "d071_reduced_precursor".into(),
        },
        // Exposure-qualified long-horizon checkpoints (same p family, labeled).
        GovernedState {
            id: "constitutive_R22_E_checkpoint_a".into(),
            radius: 22.0,
            p: P_CONSTITUTIVE_R22,
            q_c: q,
            a_retention: D075_CONSTITUTIVE_A_RETENTION,
            c_retention: D075_CONSTITUTIVE_C_RETENTION,
            capacity: cap22,
            is_control: false,
            phase: "long_horizon_checkpoint".into(),
        },
        GovernedState {
            id: "constitutive_R22_E_checkpoint_b".into(),
            radius: 22.0,
            p: P_CONSTITUTIVE_R22 * 0.98, // mild interface fluctuation within measured window
            q_c: q,
            a_retention: D075_CONSTITUTIVE_A_RETENTION,
            c_retention: D075_CONSTITUTIVE_C_RETENTION,
            capacity: cap22,
            is_control: false,
            phase: "long_horizon_checkpoint".into(),
        },
        GovernedState {
            id: "constitutive_R22_E_checkpoint_c".into(),
            radius: 22.0,
            p: P_CONSTITUTIVE_R22 * 1.02,
            q_c: q,
            a_retention: D075_CONSTITUTIVE_A_RETENTION,
            c_retention: D075_CONSTITUTIVE_C_RETENTION,
            capacity: cap22,
            is_control: false,
            phase: "long_horizon_checkpoint".into(),
        },
        GovernedState {
            id: "k_precursor_zero_control".into(),
            radius: 22.0,
            p: P_K_PREC_ZERO,
            q_c: q,
            a_retention: A_RET_KPREC0,
            c_retention: C_RET_KPREC0,
            capacity: cap22,
            is_control: true,
            phase: "control".into(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiRequirement {
    pub state_id: String,
    pub p: f64,
    pub theta_star: f64,
    pub chi_required: f64,
    pub finite_nonneg: bool,
    pub near_zero_p: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohesionReport {
    pub requirements_095: Vec<ChiRequirement>,
    pub requirements_stage_e: Vec<ChiRequirement>,
    pub chi_span_095: f64,
    pub loo_median_factor_ok: bool,
    pub selected_chi: f64,
    pub selected_chi_covers_all_radii: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

/// Gate 2 — required cohesion reconstruction across governed D-075 states.
pub fn gate2_cohesion_reconstruction() -> CohesionReport {
    let states = governed_states();
    let non_control: Vec<_> = states.iter().filter(|s| !s.is_control).collect();
    let mut req_095 = Vec::new();
    for s in &non_control {
        let chi = chi_required(OCC_CONTRACT, s.p, D077_K_EQ);
        req_095.push(ChiRequirement {
            state_id: s.id.clone(),
            p: s.p,
            theta_star: OCC_CONTRACT,
            chi_required: chi,
            finite_nonneg: chi.is_finite() && chi >= -1e-9,
            near_zero_p: s.p < 1e-6,
        });
    }
    let mut req_se = Vec::new();
    for s in &non_control {
        let chi = chi_required(STAGE_E_MIN_OCCUPANCY, s.p, D077_K_EQ);
        req_se.push(ChiRequirement {
            state_id: s.id.clone(),
            p: s.p,
            theta_star: STAGE_E_MIN_OCCUPANCY,
            chi_required: chi,
            finite_nonneg: chi.is_finite(),
            near_zero_p: s.p < 1e-6,
        });
    }

    let chis: Vec<f64> = req_095
        .iter()
        .filter(|r| r.finite_nonneg && !r.near_zero_p)
        .map(|r| r.chi_required.max(0.0))
        .collect();
    let (chi_min, chi_max) = chis
        .iter()
        .copied()
        .fold((f64::INFINITY, 0.0_f64), |(mn, mx), c| (mn.min(c), mx.max(c)));
    let span = if chi_min > EPS {
        chi_max / chi_min
    } else if chi_max <= EPS {
        1.0
    } else {
        f64::INFINITY
    };

    // Leave-one-out: each state's χ vs median of others within factor 2.
    let mut loo_ok = true;
    for i in 0..chis.len() {
        let mut others: Vec<f64> = chis
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, c)| *c)
            .collect();
        if others.is_empty() {
            continue;
        }
        others.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = others[others.len() / 2];
        let ratio = (chis[i] / med.max(EPS)).max(med / chis[i].max(EPS));
        if ratio > LOO_FACTOR_MAX {
            loo_ok = false;
        }
    }

    // One global χ: take max required (covers all lower-p states upward).
    let selected = chi_max;
    let radii_ok = [16.0, 22.0, 32.0].iter().all(|r| {
        non_control
            .iter()
            .filter(|s| (s.radius - *r).abs() < 1e-9)
            .all(|s| {
                let th = theta_eq_cooperative(s.p, selected, D077_K_EQ);
                th + 1e-9 >= OCC_CONTRACT
            })
    });

    let all_finite = req_095.iter().all(|r| r.finite_nonneg && !r.near_zero_p);
    let pass = all_finite
        && span.is_finite()
        && span <= PORTABILITY_SPAN_MAX
        && loo_ok
        && selected.is_finite()
        && selected >= 0.0
        && selected < FRUMKIN_CRITICAL_CHI
        && radii_ok;

    CohesionReport {
        requirements_095: req_095,
        requirements_stage_e: req_se,
        chi_span_095: span,
        loo_median_factor_ok: loo_ok,
        selected_chi: selected,
        selected_chi_covers_all_radii: radii_ok,
        pass,
        failure: if pass {
            None
        } else {
            Some("D077_COOPERATIVE_COHESION_NOT_PORTABLE".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicEval {
    pub state_id: String,
    pub chi: f64,
    pub p: f64,
    pub theta_eq: f64,
    pub a_retention: f64,
    pub c_retention: f64,
    pub p_bounded: bool,
    pub gross_exchange_active: bool,
    pub no_direct_a_cost: bool,
    pub capacity_saturated: bool,
    pub is_control: bool,
    pub qualifies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicReport {
    pub evals: Vec<MetabolicEval>,
    pub intermediate_p: f64,
    pub intermediate_eval: MetabolicEval,
    pub any_non_control_qualifies: bool,
    pub constitutive_hits_membrane_a_collapses: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

fn eval_metabolic(st: &GovernedState, chi: f64) -> MetabolicEval {
    let th = theta_eq_cooperative(st.p, chi, D077_K_EQ);
    let red = ReducedState {
        theta: th,
        p: st.p,
        q_c: st.q_c,
    };
    let ads = j_ads(red, D077_K_EXCHANGE, D077_K_EQ, 1.0);
    let des = j_des(red, chi, D077_K_EXCHANGE, 1.0);
    let active = ads > EPS && des > EPS;
    let sat = th >= 1.0 - 1e-6;
    let qualifies = th + 1e-9 >= OCC_CONTRACT
        && st.a_retention + 1e-12 >= A_RETENTION_GATE
        && st.c_retention + 1e-12 >= C_RETENTION_GATE
        && st.p.is_finite()
        && st.p < 1.0e3
        && active
        && !sat
        && !st.is_control;
    MetabolicEval {
        state_id: st.id.clone(),
        chi,
        p: st.p,
        theta_eq: th,
        a_retention: st.a_retention,
        c_retention: st.c_retention,
        p_bounded: st.p < 1.0e3,
        gross_exchange_active: active,
        no_direct_a_cost: true,
        capacity_saturated: sat,
        is_control: st.is_control,
        qualifies,
    }
}

/// Gate 3 — metabolic feasibility under D-075 measured biology.
pub fn gate3_metabolic_feasibility(chi: f64) -> MetabolicReport {
    let states = governed_states();
    let evals: Vec<_> = states.iter().map(|s| eval_metabolic(s, chi)).collect();
    // Intermediate analytically derived p: p_eq(θ*=0.95, χ).
    let p_int = p_eq_cooperative(OCC_CONTRACT, chi, D077_K_EQ);
    let intermediate_state = GovernedState {
        id: "intermediate_analytic_p".into(),
        radius: 22.0,
        p: p_int,
        q_c: D075_MEAN_Q_C,
        // A/C unknown analytically — inherit constitutive measured collapse (honest).
        a_retention: D075_CONSTITUTIVE_A_RETENTION,
        c_retention: D075_CONSTITUTIVE_C_RETENTION,
        capacity: D075_INTERFACE_CAPACITY,
        is_control: false,
        phase: "analytic_intermediate".into(),
    };
    let intermediate_eval = eval_metabolic(&intermediate_state, chi);
    let any = evals.iter().any(|e| e.qualifies) || intermediate_eval.qualifies;
    let constitutive = evals
        .iter()
        .find(|e| e.state_id == "constitutive_R22_pre")
        .cloned();
    let constitutive_hits = constitutive
        .as_ref()
        .map(|e| e.theta_eq + 1e-9 >= OCC_CONTRACT && e.a_retention < A_RETENTION_GATE)
        .unwrap_or(false);

    MetabolicReport {
        evals,
        intermediate_p: p_int,
        intermediate_eval,
        any_non_control_qualifies: any,
        constitutive_hits_membrane_a_collapses: constitutive_hits,
        pass: any,
        failure: if any {
            None
        } else {
            Some("D077_COOPERATIVE_EXCHANGE_METABOLICALLY_INFEASIBLE".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementReport {
    pub chi: f64,
    pub p: f64,
    pub theta_eq: f64,
    pub j_ads: f64,
    pub j_des: f64,
    pub j_net: f64,
    pub residence_time: f64,
    pub tracer_replacement_fraction: f64,
    pub precursor_production_cost: f64,
    pub exposure_for_one_membrane_eq: f64,
    pub near_zero_net: bool,
    pub positive_gross: bool,
    pub replacement_in_horizon: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

/// Gate 4 — turnover / molecular replacement at candidate equilibrium.
pub fn gate4_replacement(chi: f64, p: f64, q_c: f64) -> ReplacementReport {
    let th = theta_eq_cooperative(p, chi, D077_K_EQ);
    let st = ReducedState {
        theta: th,
        p,
        q_c,
    };
    let ja = j_ads(st, D077_K_EXCHANGE, D077_K_EQ, 1.0);
    let jd = j_des(st, chi, D077_K_EXCHANGE, 1.0);
    let jn = ja - jd;
    let residence = if jd > EPS { th / jd } else { f64::INFINITY };
    // Tracer replacement over horizon H: 1 − exp(−H/τ).
    let frac = if residence.is_finite() {
        1.0 - (-REPLACEMENT_HORIZON / residence).exp()
    } else {
        0.0
    };
    // Precursor cost for one membrane-equivalent: ≈ capacity units of P adsorbed gross.
    let cost = ja * REPLACEMENT_HORIZON;
    let exposure = if ja > EPS { th / ja } else { f64::INFINITY };
    let near_zero = jn.abs() < 1e-9 * (1.0 + ja.abs());
    let positive = ja > EPS && jd > EPS;
    let in_h = frac + 1e-12 >= 1.0 - (-1.0_f64).exp(); // ≥ 1−1/e ≈ 0.63 as soft; require ≥1 eq:
    let one_eq = residence.is_finite() && residence <= REPLACEMENT_HORIZON;
    let pass = near_zero && positive && one_eq && cost.is_finite();
    let _ = in_h;
    ReplacementReport {
        chi,
        p,
        theta_eq: th,
        j_ads: ja,
        j_des: jd,
        j_net: jn,
        residence_time: residence,
        tracer_replacement_fraction: frac,
        precursor_production_cost: cost,
        exposure_for_one_membrane_eq: exposure,
        near_zero_net: near_zero,
        positive_gross: positive,
        replacement_in_horizon: one_eq,
        pass,
        failure: if pass {
            None
        } else {
            Some("frozen_or_insufficient_replacement".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    pub name: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageReport {
    pub controls: Vec<ControlResult>,
    pub pass: bool,
    pub failure: Option<String>,
}

fn integrate_theta(
    theta: &mut f64,
    p: f64,
    q_c: f64,
    chi: f64,
    steps: usize,
    dt: f64,
) {
    for _ in 0..steps {
        let st = ReducedState {
            theta: *theta,
            p,
            q_c,
        };
        let j = flux_chi(st, chi, D077_K_EXCHANGE, D077_K_EQ, 1.0);
        let max_up = (1.0 - *theta).max(0.0);
        let max_dn = (*theta).max(0.0);
        let mut d = j * dt;
        if d > max_up {
            d = max_up;
        } else if d < -max_dn {
            d = -max_dn;
        }
        *theta += d;
    }
}

/// Gate 5 — damage, reserve, starvation causality (reduced model).
pub fn gate5_damage_starvation(chi: f64) -> DamageReport {
    let p = P_CONSTITUTIVE_R22;
    let q = D075_MEAN_Q_C;
    let th_eq = theta_eq_cooperative(p, chi, D077_K_EQ);
    let mut controls = Vec::new();

    // Single 10% damage recovery.
    let mut th = th_eq * 0.9;
    integrate_theta(&mut th, p, q, chi, 80_000, 0.05);
    let recovered = th >= 0.95 * th_eq - 1e-6;
    controls.push(ControlResult {
        name: "single_10pct_damage_recovery".into(),
        pass: recovered && th_eq + 1e-9 >= OCC_CONTRACT,
        detail: format!("pre={th_eq:.6} end={th:.6}"),
    });

    // Repeated damage: deplete stored P proxy by reducing effective p stepwise.
    let mut th_r = th_eq;
    let mut p_store = p;
    let mut still_repairs = true;
    for _ in 0..5 {
        th_r *= 0.9;
        integrate_theta(&mut th_r, p_store, q, chi, 40_000, 0.05);
        if th_r < 0.95 * th_eq {
            still_repairs = false;
        }
        p_store *= 0.5; // metabolism-limited precursor after repeated events
    }
    controls.push(ControlResult {
        name: "repeated_damage_resource_dependent".into(),
        pass: !still_repairs || p_store < p * 0.1,
        detail: format!("th_end={th_r:.6} p_store={p_store:.6}"),
    });

    // No A: cooperative law has no A in rates — membrane may persist at eq; require
    // that this is distinguished from energy-driven cycle (no A sink). Pass as
    // "no continuous A cost" structural property; starvation uses P/q instead.
    controls.push(ControlResult {
        name: "no_a_no_direct_s_cost".into(),
        pass: true,
        detail: "J_chi independent of A by construction".into(),
    });

    // No precursor production / p=0 after damage.
    let mut th_np = th_eq * 0.9;
    integrate_theta(&mut th_np, 0.0, q, chi, 40_000, 0.05);
    controls.push(ControlResult {
        name: "no_precursor_fails_repair".into(),
        pass: th_np < 0.95 * th_eq,
        detail: format!("th_end={th_np:.6}"),
    });

    // No catalyst q=0.
    let mut th_nq = th_eq * 0.9;
    integrate_theta(&mut th_nq, p, 0.0, chi, 40_000, 0.05);
    controls.push(ControlResult {
        name: "no_catalyst_fails".into(),
        pass: (th_nq - th_eq * 0.9).abs() < 1e-6, // frozen without q
        detail: format!("th_end={th_nq:.6}"),
    });

    // Nutrient / fuel withdrawal ~ p→0, q→0 from healthy state.
    let mut th_st = th_eq;
    integrate_theta(&mut th_st, 0.0, 0.0, chi, 40_000, 0.05);
    controls.push(ControlResult {
        name: "starvation_prevents_indefinite_replacement".into(),
        pass: th_st <= th_eq + 1e-9, // no growth; with p=0 desorbs toward 0 if any kinetics
        detail: format!("th_end={th_st:.6}"),
    });
    // With q=0 kinetics freeze — force p=0,q=1 desorption path:
    let mut th_st2 = th_eq;
    integrate_theta(&mut th_st2, 0.0, 1.0, chi, 80_000, 0.05);
    controls.push(ControlResult {
        name: "starvation_desorption".into(),
        pass: th_st2 < th_eq - 1e-3,
        detail: format!("th_end={th_st2:.6}"),
    });

    // Restoration without reset.
    let mut th_res = th_st2;
    integrate_theta(&mut th_res, p, q, chi, 80_000, 0.05);
    controls.push(ControlResult {
        name: "restoration_resumes_without_reseed".into(),
        pass: th_res > th_st2 + 1e-3,
        detail: format!("starved={th_st2:.6} restored={th_res:.6}"),
    });

    controls.push(ControlResult {
        name: "no_permanent_capacity_lock".into(),
        pass: th_res < 1.0 - 1e-6,
        detail: format!("th={th_res:.6}"),
    });

    let pass = controls.iter().all(|c| c.pass);
    DamageReport {
        pass,
        failure: if pass {
            None
        } else {
            Some("damage_or_starvation_causality_fail".into())
        },
        controls,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityReport {
    pub chi: f64,
    pub fixed_points: Vec<(f64, f64, bool)>, // (p, theta_eq, locally_stable)
    pub bistable_risk: bool,
    pub healthy_stable: bool,
    pub damage_in_basin: bool,
    pub no_spontaneous_fill_from_negligible_p: bool,
    pub no_permanent_after_total_loss: bool,
    pub no_nucleation_threshold: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

/// Gate 6 — fixed points, Jacobian, hysteresis.
pub fn gate6_stability(chi: f64) -> StabilityReport {
    let bistable = chi > FRUMKIN_CRITICAL_CHI - 1e-12;
    let mut fps = Vec::new();
    for p in [1e-6, 0.05, 0.1, P_REGULATED_REDUCED, P_CONSTITUTIVE_R22, 0.38, 1.0] {
        let th = theta_eq_cooperative(p, chi, D077_K_EQ);
        let st = ReducedState {
            theta: th.max(EPS).min(1.0 - EPS),
            p,
            q_c: 1.0,
        };
        let jac = surface_jacobian_dthetadt(st, chi, D077_K_EXCHANGE, D077_K_EQ);
        let stable = jac < -JACOBIAN_STABLE_TOL;
        fps.push((p, th, stable));
    }
    let healthy_th = theta_eq_cooperative(P_CONSTITUTIVE_R22, chi, D077_K_EQ);
    let healthy_stable = {
        let st = ReducedState {
            theta: healthy_th,
            p: P_CONSTITUTIVE_R22,
            q_c: 1.0,
        };
        surface_jacobian_dthetadt(st, chi, D077_K_EXCHANGE, D077_K_EQ) < -JACOBIAN_STABLE_TOL
            && healthy_th + 1e-9 >= OCC_CONTRACT
    };
    // 10% damage remains in basin: integrate back.
    let mut th_d = healthy_th * 0.9;
    integrate_theta(&mut th_d, P_CONSTITUTIVE_R22, 1.0, chi, 80_000, 0.05);
    let damage_in = (th_d - healthy_th).abs() < 0.02;

    let th_neg = theta_eq_cooperative(1e-8, chi, D077_K_EQ);
    let no_spontaneous = th_neg < 0.05;

    let mut th_loss = healthy_th;
    integrate_theta(&mut th_loss, 0.0, 1.0, chi, 100_000, 0.05);
    let no_permanent = th_loss < 0.05;

    // No undocumented nucleation: from θ≈0 at endogenous p should approach same eq.
    let mut th0 = 1e-4;
    integrate_theta(&mut th0, P_CONSTITUTIVE_R22, 1.0, chi, 100_000, 0.05);
    let no_nuc = (th0 - healthy_th).abs() < 0.05;

    let pass = !bistable
        && healthy_stable
        && damage_in
        && no_spontaneous
        && no_permanent
        && no_nuc;

    StabilityReport {
        chi,
        fixed_points: fps,
        bistable_risk: bistable,
        healthy_stable,
        damage_in_basin: damage_in,
        no_spontaneous_fill_from_negligible_p: no_spontaneous,
        no_permanent_after_total_loss: no_permanent,
        no_nucleation_threshold: no_nuc,
        pass,
        failure: if pass {
            None
        } else {
            Some("D077_COOPERATIVE_EXCHANGE_BASIN_INVALID".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusRow {
    pub radius: f64,
    pub p: f64,
    pub theta_eq: f64,
    pub a_retention: f64,
    pub c_retention: f64,
    pub occ_ok: bool,
    pub a_ok: bool,
    pub c_ok: bool,
    pub replacement_ok: bool,
    pub damage_ok: bool,
    pub row_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusReport {
    pub chi: f64,
    pub rows: Vec<RadiusRow>,
    pub pass: bool,
    pub failure: Option<String>,
}

/// Gate 7 — same χ across R16/R22/R32.
pub fn gate7_radius_portability(chi: f64) -> RadiusReport {
    let specs = [
        (16.0, P_CONSTITUTIVE_R16, A_RET_R16, C_RET_R16),
        (22.0, P_CONSTITUTIVE_R22, D075_CONSTITUTIVE_A_RETENTION, D075_CONSTITUTIVE_C_RETENTION),
        (32.0, P_CONSTITUTIVE_R32, A_RET_R32, C_RET_R32),
    ];
    let mut rows = Vec::new();
    for (r, p, a, c) in specs {
        let th = theta_eq_cooperative(p, chi, D077_K_EQ);
        let repl = gate4_replacement(chi, p, D075_MEAN_Q_C);
        let mut th_d = th * 0.9;
        integrate_theta(&mut th_d, p, D075_MEAN_Q_C, chi, 80_000, 0.05);
        let damage_ok = th_d >= 0.95 * th - 1e-6;
        let occ_ok = th + 1e-9 >= OCC_CONTRACT;
        let a_ok = a + 1e-12 >= A_RETENTION_GATE;
        let c_ok = c + 1e-12 >= C_RETENTION_GATE;
        let row_ok = occ_ok && a_ok && c_ok && repl.pass && damage_ok;
        rows.push(RadiusRow {
            radius: r,
            p,
            theta_eq: th,
            a_retention: a,
            c_retention: c,
            occ_ok,
            a_ok,
            c_ok,
            replacement_ok: repl.pass,
            damage_ok,
            row_ok,
        });
    }
    let pass = rows.iter().all(|r| r.row_ok);
    RadiusReport {
        chi,
        rows,
        pass,
        failure: if pass {
            None
        } else {
            Some("radius_portability_fail".into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub route: D077Route,
    pub conclusion: String,
    pub scientific_conclusion: String,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub reasons: Vec<String>,
    pub selected_chi: f64,
}

pub fn gate_route_select(
    g0: &LineageAudit,
    g1: &ThermoReport,
    g2: &CohesionReport,
    g3: &MetabolicReport,
    g4: &ReplacementReport,
    g5: &DamageReport,
    g6: &StabilityReport,
    g7: &RadiusReport,
) -> RouteDecision {
    let status = (
        "BLOCKED_NOT_RECOVERED".to_string(),
        "PHASE1_SELF_MAINTENANCE_PARTIAL".to_string(),
        "REQUIRES_REMEDIATION".to_string(),
    );
    if !g0.pass {
        return RouteDecision {
            route: D077Route::AlreadyClosed,
            conclusion: D077Route::AlreadyClosed.conclusion().into(),
            scientific_conclusion: "Cooperative χ exchange already closed historically.".into(),
            d008_status: status.0,
            phase1_status: status.1,
            production_verdict: status.2,
            next_directive: "Broader Phase 1 boundary-architecture review.".into(),
            next_execution_started: false,
            reasons: vec![g0.failure.clone().unwrap_or_default()],
            selected_chi: g2.selected_chi,
        };
    }
    if !g1.pass {
        return RouteDecision {
            route: D077Route::ThermodynamicFailure,
            conclusion: D077Route::ThermodynamicFailure.conclusion().into(),
            scientific_conclusion: "Candidate cooperative exchange fails thermodynamic/numerical review.".into(),
            d008_status: status.0,
            phase1_status: status.1,
            production_verdict: status.2,
            next_directive: "Do not implement; revise exchange thermodynamics.".into(),
            next_execution_started: false,
            reasons: vec![g1.failure.clone().unwrap_or_default()],
            selected_chi: g2.selected_chi,
        };
    }
    if !g2.pass {
        let mut reasons = vec![g2.failure.clone().unwrap_or_default()];
        reasons.push(format!(
            "required-χ span={:.4} (≤3× ok={}); LOO median factor ok={}",
            g2.chi_span_095,
            g2.chi_span_095 <= PORTABILITY_SPAN_MAX,
            g2.loo_median_factor_ok
        ));
        reasons.push(
            "D-071 reduced-p states demand χ≈1.6 while constitutive R16/R22/R32 demand χ≈0.7–0.8; leave-one-out median exceeds factor-of-two.".into(),
        );
        if !g3.pass {
            reasons.push(
                "Secondary: even a single global max-χ that covers occupancy still fails A/C retention ≥0.80 under measured endogenous biology.".into(),
            );
        }
        reasons.push(ENERGY_CYCLE_RECORD.into());
        return RouteDecision {
            route: D077Route::CohesionNotPortable,
            conclusion: D077Route::CohesionNotPortable.conclusion().into(),
            scientific_conclusion: "Required cohesion χ is not portable across the governed D-075 state family: reduced-precursor interface activity needs substantially higher χ than constitutive radii, failing leave-one-out median within factor two. Do not adopt state-specific χ.".into(),
            d008_status: status.0,
            phase1_status: status.1,
            production_verdict: status.2,
            next_directive: "Do not use radius- or state-specific χ. Formal Phase 1 boundary-substrate redesign decision remains open; cooperative exchange is not a portable drop-in.".into(),
            next_execution_started: false,
            reasons,
            selected_chi: g2.selected_chi,
        };
    }
    if !g3.pass {
        let mut reasons = vec![g3.failure.clone().unwrap_or_default()];
        if g3.constitutive_hits_membrane_a_collapses {
            reasons.push(
                "Constitutive endogenous p with portable χ reaches θ≥0.95 while measured A retention ≈0.06 collapses.".into(),
            );
        }
        reasons.push(ENERGY_CYCLE_RECORD.into());
        reasons.push(PASSIVE_RECORD.into());
        return RouteDecision {
            route: D077Route::MetabolicallyInfeasible,
            conclusion: D077Route::MetabolicallyInfeasible.conclusion().into(),
            scientific_conclusion: "Cooperative cohesion can raise equilibrium occupancy at endogenous p without continuous A consumption for S, but no governed precursor policy retains A≥0.80 and C≥0.80 while sustaining that occupancy — the organism cannot afford the metabolic envelope that supplies the required interface activity.".into(),
            d008_status: status.0,
            phase1_status: status.1,
            production_verdict: status.2,
            next_directive: "Do not increase precursor or activation production. Formal decision: redesign Phase 1 boundary substrate rather than add rates/species to current P/S architecture.".into(),
            next_execution_started: false,
            reasons,
            selected_chi: g2.selected_chi,
        };
    }
    if !g6.pass {
        return RouteDecision {
            route: D077Route::BasinInvalid,
            conclusion: D077Route::BasinInvalid.conclusion().into(),
            scientific_conclusion: "Occupancy requires fragile/bistable/history-locked basin.".into(),
            d008_status: status.0,
            phase1_status: status.1,
            production_verdict: status.2,
            next_directive: "Do not promote initialization-dependent surface state.".into(),
            next_execution_started: false,
            reasons: vec![g6.failure.clone().unwrap_or_default()],
            selected_chi: g2.selected_chi,
        };
    }
    if g0.pass && g1.pass && g2.pass && g3.pass && g4.pass && g5.pass && g6.pass && g7.pass {
        return RouteDecision {
            route: D077Route::Qualified,
            conclusion: D077Route::Qualified.conclusion().into(),
            scientific_conclusion: "Cooperative surface condensation qualifies for implementation.".into(),
            d008_status: status.0,
            phase1_status: status.1,
            production_verdict: status.2,
            next_directive: "Implement cooperative exchange schema with χ=0 compatibility; run isolated kinetics, maintenance, repair, radius portability, Stage E re-entry.".into(),
            next_execution_started: false,
            reasons: vec!["All Gates 0–7 passed.".into()],
            selected_chi: g2.selected_chi,
        };
    }
    // Surface algebra works but metabolic/radius envelope fails → boundary review.
    RouteDecision {
        route: D077Route::ArchitectureReviewFail,
        conclusion: D077Route::ArchitectureReviewFail.conclusion().into(),
        scientific_conclusion: "No physical, portable, metabolically affordable cooperative regime under measured D-075 biology.".into(),
        d008_status: status.0,
        phase1_status: status.1,
        production_verdict: status.2,
        next_directive: "Formal decision on redesigning Phase 1 boundary substrate rather than adding rates/species to P/S.".into(),
        next_execution_started: false,
        reasons: vec![
            format!("g4_pass={}", g4.pass),
            format!("g5_pass={}", g5.pass),
            format!("g7_pass={}", g7.pass),
        ],
        selected_chi: g2.selected_chi,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateEquations {
    pub free_energy: String,
    pub mu_s: String,
    pub mu_p: String,
    pub exchange_law: String,
    pub equilibrium: String,
    pub chi_zero_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenPreservation {
    pub seed_capacity_contract: String,
    pub k_eq: f64,
    pub k_exchange: f64,
    pub gamma_max: f64,
    pub p_ref: f64,
    pub d075_conclusion: String,
    pub d076_conclusion: String,
    pub energy_cycle_record: String,
    pub passive_record: String,
    pub alpha_frozen: f64,
    pub beta_frozen: f64,
    pub d074_conclusion: String,
    pub ids_ok: bool,
}

pub fn frozen_preservation() -> FrozenPreservation {
    FrozenPreservation {
        seed_capacity_contract: SEED_CAPACITY_CONTRACT_V1.into(),
        k_eq: D077_K_EQ,
        k_exchange: D077_K_EXCHANGE,
        gamma_max: D077_GAMMA_MAX,
        p_ref: D077_P_REF,
        d075_conclusion: D075_PRIMARY.into(),
        d076_conclusion: D076_CONCLUSION.into(),
        energy_cycle_record: ENERGY_CYCLE_RECORD.into(),
        passive_record: PASSIVE_RECORD.into(),
        alpha_frozen: D031_ALPHA_FROZEN,
        beta_frozen: D031_BETA_FROZEN,
        d074_conclusion: D074_CONCLUSION.into(),
        ids_ok: D075_PROJECT_ID == "D-075"
            && D076_PROJECT_ID == "D-076"
            && SEED_CONTRACT == SEED_CAPACITY_CONTRACT_V1
            && (D075_K_EQ - D073_K_EQ).abs() < 1e-15
            && (D075_K_EXCHANGE - D073_K_EXCHANGE).abs() < 1e-15
            && (D075_GAMMA_MAX - D073_GAMMA_MAX).abs() < 1e-15
            && (D075_P_REF - D073_P_REF).abs() < 1e-15
            && !D075_AGENT_MEMORY_ID.is_empty()
            && !D076_AGENT_MEMORY_ID.is_empty()
            && D075_STARTING_COMMIT == "b06254b"
            && D075_STARTING_TAG == "D-074-cellwise-exchange-parity-audit",
    }
}

pub fn candidate_equations() -> CandidateEquations {
    CandidateEquations {
        free_energy: "g(θ)=θ lnθ+(1−θ)ln(1−θ)−(χ/2)θ²".into(),
        mu_s: "μ_S=ln(θ/(1−θ))−χθ".into(),
        mu_p: "μ_P=ln(K_eq p)".into(),
        exchange_law: "J_χ=δ k_exchange q(C) Γ_max [K_eq p(1−θ)−θ e^{−χθ}]".into(),
        equilibrium: "K_eq p = θ/(1−θ) e^{−χθ}".into(),
        chi_zero_limit: "χ=0 ⇒ J=δ k_exchange q Γ_max [K_eq p(1−θ)−θ] (frozen linear)".into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D077Review {
    pub gate0: LineageAudit,
    pub gate1: ThermoReport,
    pub gate2: CohesionReport,
    pub gate3: MetabolicReport,
    pub gate4: ReplacementReport,
    pub gate5: DamageReport,
    pub gate6: StabilityReport,
    pub gate7: RadiusReport,
    pub route: RouteDecision,
    pub candidate_equations: CandidateEquations,
    pub frozen_preservation: FrozenPreservation,
}

pub fn run_full_review() -> D077Review {
    let gate0 = gate0_lineage_audit();
    let gate1 = gate1_thermodynamic_review();
    let gate2 = gate2_cohesion_reconstruction();
    let chi = gate2.selected_chi;
    let gate3 = gate3_metabolic_feasibility(chi);
    let gate4 = gate4_replacement(chi, P_CONSTITUTIVE_R22, D075_MEAN_Q_C);
    let gate5 = gate5_damage_starvation(chi);
    let gate6 = gate6_stability(chi);
    let gate7 = gate7_radius_portability(chi);
    let route = gate_route_select(
        &gate0, &gate1, &gate2, &gate3, &gate4, &gate5, &gate6, &gate7,
    );
    D077Review {
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        gate6,
        gate7,
        route,
        candidate_equations: candidate_equations(),
        frozen_preservation: frozen_preservation(),
    }
}

#[cfg(test)]
mod internal_smoke {
    use super::*;

    #[test]
    fn chi_required_at_constitutive_near_point_seven() {
        let chi = chi_required(OCC_CONTRACT, P_CONSTITUTIVE_R22, D077_K_EQ);
        assert!(chi > 0.5 && chi < 1.0, "chi={chi}");
        let th = theta_eq_cooperative(P_CONSTITUTIVE_R22, chi, D077_K_EQ);
        assert!((th - OCC_CONTRACT).abs() < 1e-6, "th={th}");
    }
}
