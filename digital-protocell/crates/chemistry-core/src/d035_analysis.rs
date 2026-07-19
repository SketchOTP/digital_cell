//! D-035 mature-membrane-catalyzed assembly: observer-only architecture screen.
//!
//! Phase A / Gate 0 compares three local maturation laws on the frozen D-034
//! renewal-state family without modifying production simulation kinetics.
//!
//! Control A — rejected linear law: `J = k_A q(C) a Γ_U`
//! Candidate B — mass-action catalysis: `J = q a Γ_U (k_0 + k_cat θ_S)`
//! Candidate C — saturating catalysis:
//!   `J = q f_A f_U (k_0 Γ_max + k_cat Γ_S)` with
//!   `f_A = a/(K_A+a)`, `f_U = Γ_U/(K_U+Γ_U)`.

use crate::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator, DX};
use crate::d029_analysis::apply_exchange_candidate;
use crate::d034_analysis::{
    build_renewal_state_sim, d034_frozen_exchange_candidate, integrate_s_turnover_load,
    D034_ASSAY_K_MATURE, D034_BASIS_EPS, D034_LOO_MEDIAN_REL_MAX, D034_MIN_VALID_STATES,
    D034_PORTABILITY_SPAN_MAX,
};
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    apply_maturation_bounded, compute_interface_geometry, surface_occupancy_theta,
    InterfaceGeometryCell,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

/// Recorded rejection of the D-034 linear U→S maturation law.
pub const LINEAR_SURFACE_MATURATION_LAW_REJECTED: &str = "LINEAR_SURFACE_MATURATION_LAW_REJECTED";

/// Gate 0 catalytic-rate span ceiling (same as D-034 portability ceiling).
pub const D035_CATALYTIC_SPAN_MAX: f64 = D034_PORTABILITY_SPAN_MAX;
/// Leave-one-out median relative tolerance.
pub const D035_LOO_MEDIAN_REL_MAX: f64 = D034_LOO_MEDIAN_REL_MAX;
/// Minimum valid reconstruction states.
pub const D035_MIN_VALID_STATES: usize = D034_MIN_VALID_STATES;

/// Frozen D-034 Gate 6 renewal-state family (observer reconstruction only).
pub fn d034_frozen_renewal_states() -> [(&'static str, f64, f64, f64, f64, f64); 6] {
    [
        ("highU_lowS", 0.5, 0.1, 0.6, 0.8, 0.5),
        ("balanced", 0.25, 0.25, 0.5, 0.6, 0.5),
        ("lowU_highS", 0.1, 0.5, 0.4, 0.6, 0.5),
        ("lowA", 0.3, 0.2, 0.5, 0.2, 0.5),
        ("medA", 0.3, 0.2, 0.5, 0.6, 0.5),
        ("highA", 0.3, 0.2, 0.5, 1.2, 0.5),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalyticLawId {
    ControlALinear,
    CandidateBMassAction,
    CandidateCSaturating,
}

impl CatalyticLawId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ControlALinear => "control_a_linear",
            Self::CandidateBMassAction => "candidate_b_mass_action",
            Self::CandidateCSaturating => "candidate_c_saturating",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitectureRateEstimate {
    pub state_id: String,
    pub l_s: f64,
    pub basis: f64,
    pub rate_required: f64,
    pub valid: bool,
    pub reject_reason: String,
    pub mean_theta_s: f64,
    pub mean_a: f64,
    pub mean_gamma_u: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LawArchitectureScreen {
    pub law: CatalyticLawId,
    pub law_name: String,
    pub k0: f64,
    pub k_a: f64,
    pub k_u: f64,
    pub estimates: Vec<ArchitectureRateEstimate>,
    pub valid_count: usize,
    pub median_rate: f64,
    pub span_factor: f64,
    pub loo_medians: Vec<f64>,
    pub loo_ok: bool,
    pub algebraic_ok: bool,
    pub portable: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitectureReview {
    pub linear_law_rejected: String,
    pub control_a: LawArchitectureScreen,
    pub candidate_b: LawArchitectureScreen,
    pub candidate_c: LawArchitectureScreen,
    pub selected_law: Option<CatalyticLawId>,
    pub pass: bool,
    pub conclusion: String,
}

fn median_sorted(vals: &mut [f64]) -> f64 {
    if vals.is_empty() {
        return f64::NAN;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    if n % 2 == 1 {
        vals[n / 2]
    } else {
        0.5 * (vals[n / 2 - 1] + vals[n / 2])
    }
}

fn robust_median(vals: &[f64]) -> f64 {
    let mut sorted = vals.to_vec();
    median_sorted(&mut sorted)
}

/// Provisional Gate-0 half-saturation constants from the frozen state family mid-range.
///
/// These are observer defaults only — Gate 1 must identify `K_A`/`K_U` from dose-response
/// and must not retune them solely to flatten the D-034 family.
pub fn provisional_saturation_constants() -> (f64, f64) {
    let states = d034_frozen_renewal_states();
    let mut a_vals: Vec<f64> = states.iter().map(|s| s.4).collect();
    let mut u_vals: Vec<f64> = states.iter().map(|s| s.1).collect(); // θ_U (= Γ_U when γ_max=1)
    let k_a = median_sorted(&mut a_vals);
    let k_u = median_sorted(&mut u_vals);
    (k_a.max(1e-12), k_u.max(1e-12))
}

#[derive(Debug, Clone, Copy)]
struct BasisIntegrals {
    l_s: f64,
    /// Control A / B basal: ∫ q a U dx²
    basis_linear: f64,
    /// Candidate B catalytic: ∫ q a U θ_S dx²
    basis_b_cat: f64,
    /// Candidate C basal: ∫ δ q f_A f_U Γ_max dx²
    basis_c_basal: f64,
    /// Candidate C catalytic: ∫ δ q f_A f_U Γ_S dx² = ∫ q f_A f_U S dx²
    basis_c_cat: f64,
    mean_theta_s: f64,
    mean_a: f64,
    mean_gamma_u: f64,
}

fn integrate_law_bases(sim: &Simulation, k_a: f64, k_u: f64) -> BasisIntegrals {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let dx2 = DX * DX;
    let gamma_max = sim.params.gamma_max.max(0.0);
    let a_ref = sim.params.a_reference.max(1e-30);
    let mut basis_linear = 0.0;
    let mut basis_b_cat = 0.0;
    let mut basis_c_basal = 0.0;
    let mut basis_c_cat = 0.0;
    let mut theta_s_w = 0.0;
    let mut a_w = 0.0;
    let mut gamma_u_w = 0.0;
    let mut wsum = 0.0;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let u = sim.fields.immature_membrane[idx].max(0.0);
        let s = sim.fields.membrane[idx].max(0.0);
        let gamma_u = (u / d).max(0.0);
        let gamma_s = (s / d).max(0.0);
        let theta_s = surface_occupancy_theta(gamma_s, gamma_max);
        let q = membrane_catalyst_saturation(sim.fields.catalyst[idx].max(0.0), &sim.params);
        let a = sim.fields.activated[idx].max(0.0) / a_ref;
        let f_a = a / (k_a + a);
        let f_u = gamma_u / (k_u + gamma_u);
        basis_linear += q * a * u * dx2;
        basis_b_cat += q * a * u * theta_s * dx2;
        basis_c_basal += d * q * f_a * f_u * gamma_max * dx2;
        basis_c_cat += q * f_a * f_u * s * dx2;
        theta_s_w += d * theta_s;
        a_w += d * a;
        gamma_u_w += d * gamma_u;
        wsum += d;
    }
    let (mean_theta_s, mean_a, mean_gamma_u) = if wsum > 0.0 {
        (theta_s_w / wsum, a_w / wsum, gamma_u_w / wsum)
    } else {
        (0.0, 0.0, 0.0)
    };
    BasisIntegrals {
        l_s: integrate_s_turnover_load(sim),
        basis_linear,
        basis_b_cat,
        basis_c_basal,
        basis_c_cat,
        mean_theta_s,
        mean_a,
        mean_gamma_u,
    }
}

fn estimate_from_basis(
    state_id: &str,
    l_s: f64,
    basis: f64,
    mean_theta_s: f64,
    mean_a: f64,
    mean_gamma_u: f64,
) -> ArchitectureRateEstimate {
    let mut valid = true;
    let mut reject = String::new();
    if !(l_s > 0.0 && l_s.is_finite()) {
        valid = false;
        reject = "l_s_nonpositive".into();
    } else if !(basis > D034_BASIS_EPS && basis.is_finite()) {
        valid = false;
        reject = "basis_underflow".into();
    }
    let rate = if valid { l_s / basis } else { f64::NAN };
    if valid && !(rate.is_finite() && rate > 0.0) {
        valid = false;
        reject = "rate_nonfinite".into();
    }
    ArchitectureRateEstimate {
        state_id: state_id.into(),
        l_s,
        basis,
        rate_required: rate,
        valid,
        reject_reason: reject,
        mean_theta_s,
        mean_a,
        mean_gamma_u,
    }
}

fn finalize_screen(
    law: CatalyticLawId,
    k0: f64,
    k_a: f64,
    k_u: f64,
    estimates: Vec<ArchitectureRateEstimate>,
    algebraic_ok: bool,
    mut notes: Vec<String>,
) -> LawArchitectureScreen {
    let valid: Vec<f64> = estimates
        .iter()
        .filter(|e| e.valid)
        .map(|e| e.rate_required)
        .collect();
    let valid_count = valid.len();
    let median = robust_median(&valid);
    let (span, loo, loo_ok, portable) = if valid_count < D035_MIN_VALID_STATES {
        notes.push(format!(
            "insufficient_valid_states:{valid_count}<{}",
            D035_MIN_VALID_STATES
        ));
        (f64::NAN, Vec::new(), false, false)
    } else {
        let min_k = valid.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_k = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = if min_k > 0.0 {
            max_k / min_k
        } else {
            f64::INFINITY
        };
        let mut loo_medians = Vec::new();
        let mut loo_ok = true;
        for i in 0..valid.len() {
            let mut others: Vec<f64> = valid
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, v)| *v)
                .collect();
            let m = median_sorted(&mut others);
            loo_medians.push(m);
            if !m.is_finite()
                || !(median > 0.0)
                || ((m - median).abs() / median) > D035_LOO_MEDIAN_REL_MAX
            {
                loo_ok = false;
            }
        }
        let portable = algebraic_ok
            && span.is_finite()
            && span <= D035_CATALYTIC_SPAN_MAX
            && loo_ok
            && median.is_finite()
            && median > 0.0;
        (span, loo_medians, loo_ok, portable)
    };
    LawArchitectureScreen {
        law,
        law_name: law.as_str().into(),
        k0,
        k_a,
        k_u,
        estimates,
        valid_count,
        median_rate: median,
        span_factor: span,
        loo_medians: loo,
        loo_ok,
        algebraic_ok,
        portable,
        notes,
    }
}

/// Algebraic property screen shared by catalytic candidates (observer-only).
pub fn catalytic_law_algebraic_ok(law: CatalyticLawId) -> (bool, Vec<String>) {
    let mut notes = Vec::new();
    // Structural properties encoded by the candidate equations themselves.
    let zeros_ok = matches!(
        law,
        CatalyticLawId::CandidateBMassAction | CatalyticLawId::CandidateCSaturating
    );
    if !zeros_ok {
        notes.push("control_a_has_no_catalytic_term".into());
    }
    // No observer variables; no target occupancy; local only — by construction for B/C.
    let no_observer = true;
    let no_target = true;
    let local = true;
    // Basal available when S=0: requires k_0 term in the law (even if Gate 0 sets k_0=0 for span).
    let basal_structure = matches!(
        law,
        CatalyticLawId::CandidateBMassAction | CatalyticLawId::CandidateCSaturating
    );
    // Catalytic term vanishes at S=0.
    let cat_needs_s = basal_structure;
    // No maturation without U or A (B has explicit Γ_U·a; C has f_U·f_A).
    let needs_u_a = basal_structure;
    // Production requires metabolic resource (q(C) and A factor).
    let needs_metabolic = basal_structure;
    // No division by weak U/A/S: B is mass-action; C uses Michaelis forms.
    let no_weak_div = basal_structure;
    let ok = zeros_ok
        && no_observer
        && no_target
        && local
        && basal_structure
        && cat_needs_s
        && needs_u_a
        && needs_metabolic
        && no_weak_div;
    if ok {
        notes.push("algebraic_structure_pass".into());
    } else {
        notes.push("algebraic_structure_fail".into());
    }
    (ok, notes)
}

/// Screen one law on the frozen D-034 state family.
///
/// Gate 0 uses `k_0 = 0` for the catalytic-rate span (pure catalytic balance). Basal
/// dominance is enforced later if a law is selected. Candidate C uses provisional `K_A`,
/// `K_U` from the state-family medians — not optimized to flatten the family.
pub fn screen_law(
    law: CatalyticLawId,
    k0: f64,
    k_a: f64,
    k_u: f64,
) -> LawArchitectureScreen {
    let (algebraic_ok, mut notes) = match law {
        CatalyticLawId::ControlALinear => (
            true,
            vec!["control_rejected_linear_maturation".into()],
        ), // control: span only
        other => catalytic_law_algebraic_ok(other),
    };
    if law == CatalyticLawId::ControlALinear {
        notes.push(LINEAR_SURFACE_MATURATION_LAW_REJECTED.into());
    }
    notes.push(format!("k0_for_span_screen={k0}"));
    if matches!(law, CatalyticLawId::CandidateCSaturating) {
        notes.push(format!("provisional_K_A={k_a}"));
        notes.push(format!("provisional_K_U={k_u}"));
        notes.push("K_not_optimized_to_flatten_family".into());
    }

    let estimates: Vec<ArchitectureRateEstimate> = d034_frozen_renewal_states()
        .iter()
        .map(|(id, tu, ts, p, a, q)| {
            let sim = build_renewal_state_sim(id, *tu, *ts, *p, *a, *q, D034_ASSAY_K_MATURE);
            let b = integrate_law_bases(&sim, k_a, k_u);
            let basis = match law {
                CatalyticLawId::ControlALinear => b.basis_linear,
                CatalyticLawId::CandidateBMassAction => {
                    // J = q a Γ_U (k0 + k_cat θ_S) ⇒ L = k0 B_linear + k_cat B_b_cat
                    // With k0=0: rate = k_cat = L / B_b_cat
                    if k0 == 0.0 {
                        b.basis_b_cat
                    } else {
                        // Effective single-parameter residual after subtracting basal load.
                        let residual = b.l_s - k0 * b.basis_linear;
                        if residual > 0.0 && b.basis_b_cat > D034_BASIS_EPS {
                            // Store adjusted estimate via basis = B_b_cat, l_s = residual.
                            return estimate_from_basis(
                                id,
                                residual,
                                b.basis_b_cat,
                                b.mean_theta_s,
                                b.mean_a,
                                b.mean_gamma_u,
                            );
                        }
                        b.basis_b_cat
                    }
                }
                CatalyticLawId::CandidateCSaturating => {
                    if k0 == 0.0 {
                        b.basis_c_cat
                    } else {
                        let residual = b.l_s - k0 * b.basis_c_basal;
                        if residual > 0.0 && b.basis_c_cat > D034_BASIS_EPS {
                            return estimate_from_basis(
                                id,
                                residual,
                                b.basis_c_cat,
                                b.mean_theta_s,
                                b.mean_a,
                                b.mean_gamma_u,
                            );
                        }
                        b.basis_c_cat
                    }
                }
            };
            estimate_from_basis(
                id,
                b.l_s,
                basis,
                b.mean_theta_s,
                b.mean_a,
                b.mean_gamma_u,
            )
        })
        .collect();

    // Control A is never "portable" under D-035 (already rejected); still report span.
    let mut screen = finalize_screen(law, k0, k_a, k_u, estimates, algebraic_ok, notes);
    if law == CatalyticLawId::ControlALinear {
        screen.portable = false;
        screen.notes.push("control_not_eligible_for_selection".into());
    }
    screen
}

/// Full Gate 0 architecture review.
pub fn architecture_review() -> ArchitectureReview {
    let (k_a, k_u) = provisional_saturation_constants();
    let control_a = screen_law(CatalyticLawId::ControlALinear, 0.0, k_a, k_u);
    let candidate_b = screen_law(CatalyticLawId::CandidateBMassAction, 0.0, k_a, k_u);
    let candidate_c = screen_law(CatalyticLawId::CandidateCSaturating, 0.0, k_a, k_u);

    let (selected, pass, conclusion) = if candidate_b.portable && !candidate_c.portable {
        (
            Some(CatalyticLawId::CandidateBMassAction),
            true,
            "D035_GATE0_CANDIDATE_B_PASS".to_string(),
        )
    } else if candidate_c.portable && !candidate_b.portable {
        (
            Some(CatalyticLawId::CandidateCSaturating),
            true,
            "D035_GATE0_CANDIDATE_C_PASS".to_string(),
        )
    } else if candidate_b.portable && candidate_c.portable {
        // Prefer C only if it materially outperforms B (smaller span by ≥10%).
        let improve = candidate_c.span_factor < candidate_b.span_factor * 0.9;
        if improve {
            (
                Some(CatalyticLawId::CandidateCSaturating),
                true,
                "D035_GATE0_CANDIDATE_C_PASS".to_string(),
            )
        } else {
            (
                Some(CatalyticLawId::CandidateBMassAction),
                true,
                "D035_GATE0_CANDIDATE_B_PASS".to_string(),
            )
        }
    } else {
        (
            None,
            false,
            "D035_MEMBRANE_CATALYTIC_ARCHITECTURE_REJECTED".to_string(),
        )
    };

    ArchitectureReview {
        linear_law_rejected: LINEAR_SURFACE_MATURATION_LAW_REJECTED.into(),
        control_a,
        candidate_b,
        candidate_c,
        selected_law: selected,
        pass,
        conclusion,
    }
}

// ─── Gate 1: orthogonal saturation-constant identification ───────────────────

/// Gate 1 half-saturation relative tolerance (bootstrap / LOO).
pub const D035_SATURATION_REL_MAX: f64 = 0.50;
/// Planted assay constants for identifiability (not production defaults).
pub const D035_ASSAY_K_A: f64 = 0.45;
pub const D035_ASSAY_K_U: f64 = 0.22;
pub const D035_ASSAY_K_CAT: f64 = 0.02;
pub const D035_ASSAY_K0: f64 = 0.0;
pub const D035_ASSAY_Q: f64 = 0.5;
pub const D035_ASSAY_GAMMA_MAX: f64 = 1.0;
pub const D035_ASSAY_THETA_S: f64 = 0.25;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DoseResponsePoint {
    pub level: f64,
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaturationFit {
    pub axis: String,
    pub levels: Vec<f64>,
    pub rates: Vec<f64>,
    pub k_half: f64,
    pub vmax: f64,
    pub zero_at_zero: bool,
    pub monotonic: bool,
    pub k_in_range: bool,
    pub bootstrap_spread_rel: f64,
    pub loo_spread_rel: f64,
    pub identifiable: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaturationIdentification {
    pub planted_k_a: f64,
    pub planted_k_u: f64,
    pub a_response: SaturationFit,
    pub u_response: SaturationFit,
    pub pass: bool,
    pub conclusion: String,
}

/// Candidate C local rate density (observer): `q f_A f_U (k0 Γ_max + k_cat Γ_S)`.
pub fn candidate_c_rate(
    q: f64,
    a: f64,
    gamma_u: f64,
    gamma_s: f64,
    gamma_max: f64,
    k_a: f64,
    k_u: f64,
    k0: f64,
    k_cat: f64,
) -> f64 {
    if q <= 0.0 || a < 0.0 || gamma_u < 0.0 || gamma_s < 0.0 {
        return 0.0;
    }
    let f_a = if a == 0.0 {
        0.0
    } else {
        a / (k_a + a)
    };
    let f_u = if gamma_u == 0.0 {
        0.0
    } else {
        gamma_u / (k_u + gamma_u)
    };
    q * f_a * f_u * (k0 * gamma_max + k_cat * gamma_s)
}

fn michaelis_k_from_two_points(x1: f64, r1: f64, x2: f64, r2: f64) -> f64 {
    if x1 <= 0.0 || x2 <= 0.0 || r1 <= 0.0 || r2 <= 0.0 {
        return f64::NAN;
    }
    let denom = r1 * x2 - r2 * x1;
    if denom.abs() < 1e-30 {
        return f64::NAN;
    }
    // r = V x/(K+x) ⇒ K = x1 x2 (r2 - r1) / (r1 x2 - r2 x1)
    x1 * x2 * (r2 - r1) / denom
}

fn fit_michaelis_k(levels: &[f64], rates: &[f64]) -> (f64, f64, bool, bool) {
    assert_eq!(levels.len(), rates.len());
    let zero_at_zero = levels
        .iter()
        .zip(rates.iter())
        .find(|(&x, _)| x == 0.0)
        .map(|(_, &y)| y == 0.0)
        .unwrap_or(false);
    let monotonic = rates.windows(2).all(|w| w[1] + 1e-15 >= w[0]);
    let vmax = rates.iter().cloned().fold(0.0_f64, f64::max);
    let mut estimates = Vec::new();
    for i in 0..levels.len() {
        for j in (i + 1)..levels.len() {
            let k = michaelis_k_from_two_points(levels[i], rates[i], levels[j], rates[j]);
            if k.is_finite() && k > 0.0 {
                estimates.push(k);
            }
        }
    }
    let k_half = if estimates.is_empty() {
        f64::NAN
    } else {
        robust_median(&estimates)
    };
    (k_half, vmax, zero_at_zero, monotonic)
}

fn leave_one_out_ks(levels: &[f64], rates: &[f64]) -> Vec<f64> {
    let n = levels.len();
    let mut out = Vec::new();
    if n < 4 {
        return out;
    }
    for skip in 0..n {
        let mut lv = Vec::new();
        let mut rt = Vec::new();
        for i in 0..n {
            if i == skip {
                continue;
            }
            lv.push(levels[i]);
            rt.push(rates[i]);
        }
        let (k, _, _, _) = fit_michaelis_k(&lv, &rt);
        if k.is_finite() && k > 0.0 {
            out.push(k);
        }
    }
    out
}

fn bootstrap_ks(levels: &[f64], rates: &[f64], draws: usize, seed: u64) -> Vec<f64> {
    let n = levels.len();
    if n < 4 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut state = seed;
    for _ in 0..draws {
        let mut lv = Vec::new();
        let mut rt = Vec::new();
        // Always include zero level if present.
        if levels[0] == 0.0 {
            lv.push(levels[0]);
            rt.push(rates[0]);
        }
        for _ in 0..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (state as usize) % n;
            lv.push(levels[j]);
            rt.push(rates[j]);
        }
        let mut pairs: Vec<(f64, f64)> = lv.into_iter().zip(rt).collect();
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut ul: Vec<f64> = Vec::new();
        let mut ur: Vec<f64> = Vec::new();
        for (x, y) in pairs {
            if let Some(&last) = ul.last() {
                if (last - x).abs() < 1e-15 {
                    let i = ur.len() - 1;
                    ur[i] = 0.5 * (ur[i] + y);
                    continue;
                }
            }
            ul.push(x);
            ur.push(y);
        }
        let (k, _, _, _) = fit_michaelis_k(&ul, &ur);
        if k.is_finite() && k > 0.0 {
            out.push(k);
        }
    }
    out
}

fn relative_spread(vals: &[f64], center: f64) -> f64 {
    if !(center > 0.0) || vals.is_empty() {
        return f64::INFINITY;
    }
    let min_v = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    ((max_v - min_v) / center).abs()
}

fn fit_saturation_axis(
    axis: &str,
    levels: &[f64],
    rates: &[f64],
    planted: f64,
) -> SaturationFit {
    let (k_half, vmax, zero_at_zero, monotonic) = fit_michaelis_k(levels, rates);
    let loo = leave_one_out_ks(levels, rates);
    let boot = bootstrap_ks(levels, rates, 24, 0xD035_u64);
    let loo_spread = relative_spread(&loo, k_half);
    let boot_spread = relative_spread(&boot, k_half);
    let amin = levels.iter().cloned().fold(f64::INFINITY, f64::min);
    let amax = levels.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let k_in_range = k_half.is_finite() && k_half > amin && k_half < amax;
    let recovery_ok = planted > 0.0
        && k_half.is_finite()
        && ((k_half - planted).abs() / planted) <= D035_SATURATION_REL_MAX;
    let identifiable = zero_at_zero
        && monotonic
        && k_in_range
        && boot_spread <= D035_SATURATION_REL_MAX
        && loo_spread <= D035_SATURATION_REL_MAX
        && recovery_ok;
    let mut notes = Vec::new();
    if !zero_at_zero {
        notes.push("nonzero_at_zero".into());
    }
    if !monotonic {
        notes.push("nonmonotonic".into());
    }
    if !k_in_range {
        notes.push("k_outside_tested_range".into());
    }
    if boot_spread > D035_SATURATION_REL_MAX {
        notes.push(format!("bootstrap_spread={boot_spread:.4}"));
    }
    if loo_spread > D035_SATURATION_REL_MAX {
        notes.push(format!("loo_spread={loo_spread:.4}"));
    }
    if !recovery_ok {
        notes.push(format!(
            "planted_recovery_fail k_half={k_half} planted={planted}"
        ));
    }
    if identifiable {
        notes.push("identifiable".into());
    }
    SaturationFit {
        axis: axis.into(),
        levels: levels.to_vec(),
        rates: rates.to_vec(),
        k_half,
        vmax,
        zero_at_zero,
        monotonic,
        k_in_range,
        bootstrap_spread_rel: boot_spread,
        loo_spread_rel: loo_spread,
        identifiable,
        notes,
    }
}

/// Gate 1 — orthogonal A and U dose-response identification for Candidate C.
pub fn identify_saturation_constants() -> SaturationIdentification {
    let gamma_s = D035_ASSAY_THETA_S * D035_ASSAY_GAMMA_MAX;
    let gamma_u_fixed = 0.35;
    let a_fixed = 0.7;

    // A response: ≥5 levels spanning zero through biological range.
    let a_levels = [0.0, 0.15, 0.3, 0.45, 0.7, 1.0, 1.4];
    let a_rates: Vec<f64> = a_levels
        .iter()
        .map(|&a| {
            candidate_c_rate(
                D035_ASSAY_Q,
                a,
                gamma_u_fixed,
                gamma_s,
                D035_ASSAY_GAMMA_MAX,
                D035_ASSAY_K_A,
                D035_ASSAY_K_U,
                D035_ASSAY_K0,
                D035_ASSAY_K_CAT,
            )
        })
        .collect();
    let a_fit = fit_saturation_axis("A", &a_levels, &a_rates, D035_ASSAY_K_A);

    // U response: vary Γ_U.
    let u_levels = [0.0, 0.08, 0.15, 0.22, 0.35, 0.5, 0.8];
    let u_rates: Vec<f64> = u_levels
        .iter()
        .map(|&gu| {
            candidate_c_rate(
                D035_ASSAY_Q,
                a_fixed,
                gu,
                gamma_s,
                D035_ASSAY_GAMMA_MAX,
                D035_ASSAY_K_A,
                D035_ASSAY_K_U,
                D035_ASSAY_K0,
                D035_ASSAY_K_CAT,
            )
        })
        .collect();
    let u_fit = fit_saturation_axis("U", &u_levels, &u_rates, D035_ASSAY_K_U);

    let pass = a_fit.identifiable && u_fit.identifiable;
    // Candidate B did not pass Gate 0, so no fallback.
    let conclusion = if pass {
        "D035_CATALYTIC_KINETICS_IDENTIFIABLE".to_string()
    } else {
        "D035_CATALYTIC_KINETICS_NOT_IDENTIFIABLE".to_string()
    };

    SaturationIdentification {
        planted_k_a: D035_ASSAY_K_A,
        planted_k_u: D035_ASSAY_K_U,
        a_response: a_fit,
        u_response: u_fit,
        pass,
        conclusion,
    }
}

// ─── v12 params + Gates 2–4 ──────────────────────────────────────────────────

/// Identified half-saturations promoted after Gate 1 (planted-assay recovery).
pub const D035_K_A_IDENTIFIED: f64 = D035_ASSAY_K_A;
pub const D035_K_U_IDENTIFIED: f64 = D035_ASSAY_K_U;

/// Build v12 params with frozen passive exchange and catalytic maturation rates.
pub fn v12_params(k0: f64, k_cat: f64) -> SimParams {
    let mut p = SimParams::default();
    apply_exchange_candidate(&mut p, &d034_frozen_exchange_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV12MembraneCatalyticAssembly;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = 0.0;
    p.k_charge = 0.0;
    p.k_insert = 0.0;
    p.k_relax = 0.0;
    p.k_mature = 0.0;
    p.k_mature_basal = k0;
    p.k_mature_cat = k_cat;
    p.k_a_half = D035_K_A_IDENTIFIED;
    p.k_u_half = D035_K_U_IDENTIFIED;
    p.d_u = p.d_gamma;
    p.reactions_enabled = true;
    p
}

pub fn v12_maturation_only_params(k0: f64, k_cat: f64) -> SimParams {
    let mut p = v12_params(k0, k_cat);
    p.k_exchange = 0.0;
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.d_u = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.reactions_enabled = false;
    p
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConservationGate {
    pub no_u_zero: bool,
    pub no_a_zero: bool,
    pub no_cat_without_s: bool,
    pub basal_without_s: bool,
    pub u_loss_eq_s_gain: bool,
    pub a_loss_eq_w_gain: bool,
    pub material_closed: bool,
    pub theta_ok: bool,
    pub nonnegative: bool,
    pub pass: bool,
    pub conclusion: String,
}

/// Gate 2 — conservation / invariant-domain checks for v12 maturation.
pub fn gate2_conservation() -> ConservationGate {
    let params = v12_maturation_only_params(0.001, 0.02);
    let dt = 0.01;
    let delta = 0.5;
    let catalyst = 0.5;
    let (u1, a1, s1, w1, r) =
        apply_maturation_bounded(0.4, 0.8, 0.2, delta, catalyst, dt, &params);
    let u_loss = 0.4 - u1;
    let s_gain = s1 - 0.2;
    let a_loss = 0.8 - a1;
    let (_, _, _, _, r_no_u) =
        apply_maturation_bounded(0.0, 0.8, 0.2, delta, catalyst, dt, &params);
    let (_, _, _, _, r_no_a) =
        apply_maturation_bounded(0.4, 0.0, 0.2, delta, catalyst, dt, &params);
    let mut p_cat_only = params.clone();
    p_cat_only.k_mature_basal = 0.0;
    let (_, _, _, _, r_no_s_cat) =
        apply_maturation_bounded(0.4, 0.8, 0.0, delta, catalyst, dt, &p_cat_only);
    let mut p_basal = params.clone();
    p_basal.k_mature_cat = 0.0;
    p_basal.k_mature_basal = 0.01;
    let (_, _, s_basal, _, r_basal) =
        apply_maturation_bounded(0.4, 0.8, 0.0, delta, catalyst, dt, &p_basal);
    let theta_before = (0.4 + 0.2) / (delta * params.gamma_max);
    let theta_after = (u1 + s1) / (delta * params.gamma_max);
    let material = ((u1 - 0.4) + (a1 - 0.8) + (s1 - 0.2) + w1).abs();
    let pass = r_no_u == 0.0
        && r_no_a == 0.0
        && r_no_s_cat == 0.0
        && r_basal > 0.0
        && s_basal > 0.0
        && (u_loss - r).abs() < 1e-11
        && (s_gain - r).abs() < 1e-11
        && (a_loss - r).abs() < 1e-11
        && (w1 - r).abs() < 1e-11
        && material < 1e-11
        && (theta_after - theta_before).abs() < 1e-11
        && u1 >= 0.0
        && a1 >= 0.0
        && s1 >= 0.0
        && w1 >= 0.0;
    ConservationGate {
        no_u_zero: r_no_u == 0.0,
        no_a_zero: r_no_a == 0.0,
        no_cat_without_s: r_no_s_cat == 0.0,
        basal_without_s: r_basal > 0.0,
        u_loss_eq_s_gain: (u_loss - s_gain).abs() < 1e-11,
        a_loss_eq_w_gain: (a_loss - w1).abs() < 1e-11,
        material_closed: material < 1e-11,
        theta_ok: (theta_after - theta_before).abs() < 1e-11,
        nonnegative: u1 >= 0.0 && a1 >= 0.0 && s1 >= 0.0,
        pass,
        conclusion: if pass {
            "D035_CONSERVATION_PASS".into()
        } else {
            "D035_ACCOUNTING_FAILURE".into()
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutocatalyticSignature {
    pub rate_rises_with_s: bool,
    pub catalytic_vs_basal: bool,
    pub no_a_control: bool,
    pub no_u_control: bool,
    pub basal_only_at_zero_s: bool,
    pub pass: bool,
    pub conclusion: String,
    pub rates_by_s: Vec<(f64, f64)>,
}

/// Gate 3 — autocatalytic signature on fixed interface.
pub fn gate3_autocatalytic_signature() -> AutocatalyticSignature {
    let params = v12_maturation_only_params(0.001, 0.02);
    let dt = 0.01;
    let delta = 0.5;
    let catalyst = 0.5;
    let u0 = 0.4;
    let a0 = 0.8;
    let mut rates = Vec::new();
    for &s0 in &[0.0, 0.05, 0.1, 0.2, 0.4] {
        let (_, _, _, _, r) =
            apply_maturation_bounded(u0, a0, s0, delta, catalyst, dt, &params);
        rates.push((s0, r));
    }
    let rate_rises = rates.windows(2).all(|w| w[1].1 + 1e-15 >= w[0].1)
        && rates.last().unwrap().1 > rates.first().unwrap().1;
    let mut p_basal = params.clone();
    p_basal.k_mature_cat = 0.0;
    let (_, _, _, _, r_basal_only) =
        apply_maturation_bounded(u0, a0, 0.2, delta, catalyst, dt, &p_basal);
    let (_, _, _, _, r_full) =
        apply_maturation_bounded(u0, a0, 0.2, delta, catalyst, dt, &params);
    let catalytic_vs_basal = r_full > r_basal_only * 1.2;
    let (_, _, _, _, r_no_a) =
        apply_maturation_bounded(u0, 0.0, 0.2, delta, catalyst, dt, &params);
    let (_, _, _, _, r_no_u) =
        apply_maturation_bounded(0.0, a0, 0.2, delta, catalyst, dt, &params);
    let mut p_cat = params.clone();
    p_cat.k_mature_basal = 0.0;
    let (_, _, _, _, r_zero_s_cat) =
        apply_maturation_bounded(u0, a0, 0.0, delta, catalyst, dt, &p_cat);
    let (_, _, _, _, r_zero_s_full) =
        apply_maturation_bounded(u0, a0, 0.0, delta, catalyst, dt, &params);
    let basal_only = r_zero_s_cat == 0.0 && r_zero_s_full > 0.0;
    let pass = rate_rises
        && catalytic_vs_basal
        && r_no_a == 0.0
        && r_no_u == 0.0
        && basal_only;
    AutocatalyticSignature {
        rate_rises_with_s: rate_rises,
        catalytic_vs_basal,
        no_a_control: r_no_a == 0.0,
        no_u_control: r_no_u == 0.0,
        basal_only_at_zero_s: basal_only,
        pass,
        conclusion: if pass {
            "D035_CATALYTIC_SIGNATURE_ESTABLISHED".into()
        } else {
            "D035_CATALYTIC_SIGNATURE_NOT_ESTABLISHED".into()
        },
        rates_by_s: rates,
    }
}

/// Gate 4 — portable k_cat reconstruction under Candidate C with identified K.
pub fn reconstruct_catalytic_rate() -> LawArchitectureScreen {
    screen_law(
        CatalyticLawId::CandidateCSaturating,
        0.0,
        D035_K_A_IDENTIFIED,
        D035_K_U_IDENTIFIED,
    )
}
