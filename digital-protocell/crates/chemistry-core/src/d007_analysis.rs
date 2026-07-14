//! D-007 joint kinetic nullcline search — pure analysis (no chemistry changes).

use crate::nullcline::FixedPointClass;
use serde::{Deserialize, Serialize};

/// Machine-precision D-006 derived interface rate (1.0× planar calibration).
pub const D006_K_STRUCTURE_INTERFACE: f64 = 0.09642857142857159;
/// Frozen D-006 catalyst reproduction rate.
pub const D006_K_REP: f64 = 0.014489097664708522;
/// Configuration hash of D-006 1.0× survivor (cand-a65c9c86e5ad…).
pub const D006_REFERENCE_CONFIGURATION_HASH: &str =
    "53c5fd482d171d8a5d20dfbc16e7fdc1f1fc782d06d98c659c1a82fd23a172bb";
pub const D006_K_REP_MAX_FACTOR: f64 = 3.0;
pub const D006_K_REP_MIN_FACTOR: f64 = 0.75;
pub const D007_EPS: f64 = 1e-12;

/// Required result.json fields for strict D-007 provenance schema.
pub const D007_REQUIRED_RESULT_FIELDS: &[&str] = &[
    "equation_version",
    "candidate_id",
    "candidate_hash",
    "configuration_hash",
    "k_structure_interface",
    "k_rep",
    "R0",
    "C0",
    "noise_seed",
    "noise_amplitude",
    "source_commit",
    "binary_hash",
    "accepted_substeps",
    "simulated_time",
    "initial_field_hashes",
    "final_field_hashes",
    "initial_field_masses",
    "final_field_masses",
    "reaction_accounting",
    "diffusion_accounting",
    "reservoir_accounting",
    "numerical_corrections",
    "accounting_residual",
    "termination_status",
    "clean_termination",
    "Q_phi",
    "Q_C",
    "slope_phi",
    "slope_C",
    "equivalent_radius",
    "v_R",
    "mean_C_inside",
    "v_C_inside",
    "retention",
    "connected_component_fraction",
    "turnover_ratios",
    "resource_statistics",
    "waste_statistics",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StructuralBracketClass {
    AllDecline,
    RestoringCrossing,
    AllGrow,
    Disordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CatalystBracketClass {
    AllDecline,
    RestoringCrossing,
    AllGrow,
    Disordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JointCandidateClass {
    NoRadiusNullcline,
    NoCatalystNullcline,
    NullclinesDisjoint,
    UnstableIntersection,
    SaddleIntersection,
    StableIntersectionNarrow,
    StableIntersectionRobust,
    NumericallyInvalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredKRepEstimate {
    pub current_k_rep: f64,
    pub n_input: usize,
    pub n_valid: usize,
    pub n_rejected: usize,
    pub median_required_k_rep: f64,
    pub iqr_required_k_rep: f64,
    pub min_required_k_rep: f64,
    pub max_required_k_rep: f64,
    pub median_by_r0: Vec<(f64, f64)>,
    pub median_by_c0: Vec<(f64, f64)>,
    pub median_by_candidate: Vec<(String, f64)>,
    pub k_rep_center: f64,
    pub k_rep_center_clamped: f64,
    pub outside_bounded_range: bool,
    pub classification: String,
}

/// Reject Stage D rows that must not inform the k_rep estimator.
pub fn reject_required_k_rep_row(
    q_c: Option<f64>,
    retention: f64,
    final_catalyst_mass: f64,
    connected_component_fraction: Option<f64>,
    dish_contact: bool,
    numerical_failure: bool,
    resource_exhausted: bool,
    fragmentation: bool,
) -> bool {
    if q_c.is_none() || q_c.unwrap().is_nan() || q_c.unwrap() <= 0.0 {
        return true;
    }
    if retention < 0.05 || final_catalyst_mass < 1e-3 {
        return true;
    }
    if resource_exhausted || fragmentation || dish_contact || numerical_failure {
        return true;
    }
    if let Some(cc) = connected_component_fraction {
        if cc < 0.50 {
            return true;
        }
    }
    false
}

pub fn required_k_rep(current_k_rep: f64, q_c: f64) -> f64 {
    current_k_rep / q_c.max(D007_EPS)
}

pub fn clamp_k_rep_to_d006_bounds(k_rep: f64) -> f64 {
    let lo = D006_K_REP * D006_K_REP_MIN_FACTOR;
    let hi = D006_K_REP * D006_K_REP_MAX_FACTOR;
    k_rep.clamp(lo, hi)
}

pub fn k_rep_search_remains_bounded(k_rep: f64) -> bool {
    k_rep <= D006_K_REP * D006_K_REP_MAX_FACTOR + 1e-15
        && k_rep >= D006_K_REP * D006_K_REP_MIN_FACTOR - 1e-15
}

fn median_sorted(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if xs.is_empty() {
        return f64::NAN;
    }
    xs[xs.len() / 2]
}

fn iqr_sorted(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = xs.len();
    if n < 4 {
        return 0.0;
    }
    xs[(3 * n) / 4] - xs[n / 4]
}

/// Build catalyst-rate estimate from (q_c, metadata) rows.
pub fn estimate_required_k_rep(
    current_k_rep: f64,
    rows: &[(f64, f64, f64, f64, String, f64, Option<f64>, bool, bool, bool, bool)],
) -> RequiredKRepEstimate {
    // tuple: (q_c, retention, final_c_mass, r0, cand_id, c0, cc, dish, num, res, frag)
    let n_input = rows.len();
    let mut valid_reqs = Vec::new();
    let mut buckets_r: std::collections::BTreeMap<i64, Vec<f64>> = std::collections::BTreeMap::new();
    let mut buckets_c: std::collections::BTreeMap<i64, Vec<f64>> = std::collections::BTreeMap::new();
    let mut buckets_cand: std::collections::BTreeMap<String, Vec<f64>> =
        std::collections::BTreeMap::new();

    for (q_c, ret, cm, r0, cid, c0, cc, dish, num, res, frag) in rows {
        if reject_required_k_rep_row(
            Some(*q_c),
            *ret,
            *cm,
            *cc,
            *dish,
            *num,
            *res,
            *frag,
        ) {
            continue;
        }
        let req = required_k_rep(current_k_rep, *q_c);
        valid_reqs.push(req);
        buckets_r
            .entry((*r0 * 1000.0).round() as i64)
            .or_default()
            .push(req);
        buckets_c
            .entry((*c0 * 1000.0).round() as i64)
            .or_default()
            .push(req);
        buckets_cand.entry(cid.clone()).or_default().push(req);
    }

    let by_r0: Vec<(f64, f64)> = buckets_r
        .into_iter()
        .map(|(k, vs)| (k as f64 / 1000.0, median_sorted(vs)))
        .collect();
    let by_c0: Vec<(f64, f64)> = buckets_c
        .into_iter()
        .map(|(k, vs)| (k as f64 / 1000.0, median_sorted(vs)))
        .collect();
    let by_cand: Vec<(String, f64)> = buckets_cand
        .into_iter()
        .map(|(k, vs)| (k, median_sorted(vs)))
        .collect();

    let n_valid = valid_reqs.len();
    let med = median_sorted(valid_reqs.clone());
    let iq = iqr_sorted(valid_reqs.clone());
    let (min_v, max_v) = if valid_reqs.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        (
            valid_reqs.iter().cloned().fold(f64::INFINITY, f64::min),
            valid_reqs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        )
    };
    let bound = current_k_rep * D006_K_REP_MAX_FACTOR;
    let outside = med.is_finite() && med > bound;
    let center_clamped = if outside { bound } else { med };
    RequiredKRepEstimate {
        current_k_rep,
        n_input,
        n_valid,
        n_rejected: n_input - n_valid,
        median_required_k_rep: med,
        iqr_required_k_rep: iq,
        min_required_k_rep: min_v,
        max_required_k_rep: max_v,
        median_by_r0: by_r0,
        median_by_c0: by_c0,
        median_by_candidate: by_cand,
        k_rep_center: med,
        k_rep_center_clamped: center_clamped,
        outside_bounded_range: outside,
        classification: if outside {
            "D007_CATALYST_RATE_OUTSIDE_BOUNDED_RANGE".into()
        } else {
            "D007_CATALYST_RATE_WITHIN_BOUNDED_RANGE".into()
        },
    }
}

/// Classify structural-bracket median velocities at R0=16,24,32.
pub fn classify_structural_bracket(v16: f64, v24: f64, v32: f64) -> StructuralBracketClass {
    let all_neg = v16 < 0.0 && v24 < 0.0 && v32 < 0.0;
    let all_pos = v16 > 0.0 && v24 > 0.0 && v32 > 0.0;
    if all_neg {
        return StructuralBracketClass::AllDecline;
    }
    if all_pos {
        return StructuralBracketClass::AllGrow;
    }
    let restoring = v16 > 0.0 && v32 < 0.0 && v16 >= v24 && v24 >= v32;
    // near-monotonic: allow tiny float noise between equal steps
    let near_mono = v16 > 0.0 && v32 < 0.0 && (v16 - v24) >= -1e-9 && (v24 - v32) >= -1e-9;
    if restoring || near_mono {
        return StructuralBracketClass::RestoringCrossing;
    }
    StructuralBracketClass::Disordered
}

pub fn provisional_structural_factor_passes(v16: f64, v24: f64, v32: f64) -> bool {
    matches!(
        classify_structural_bracket(v16, v24, v32),
        StructuralBracketClass::RestoringCrossing
    ) && v16 > 0.0
        && v32 < 0.0
}

pub fn structural_failure_gate(classes: &[StructuralBracketClass]) -> Option<&'static str> {
    if classes.is_empty() {
        return Some("D007_NO_STRUCTURAL_NULLCLINE");
    }
    if classes
        .iter()
        .any(|c| matches!(c, StructuralBracketClass::RestoringCrossing))
    {
        return None;
    }
    // No ordered restoring crossing inside the tested factor domain.
    Some("D007_NO_STRUCTURAL_NULLCLINE")
}

pub fn classify_catalyst_bracket(v_low: f64, v_mid: f64, v_high: f64) -> CatalystBracketClass {
    let all_neg = v_low < 0.0 && v_mid < 0.0 && v_high < 0.0;
    let all_pos = v_low > 0.0 && v_mid > 0.0 && v_high > 0.0;
    if all_neg {
        return CatalystBracketClass::AllDecline;
    }
    if all_pos {
        return CatalystBracketClass::AllGrow;
    }
    // low C tends to increase, high C tends to decrease
    if v_low > 0.0 && v_high < 0.0 {
        return CatalystBracketClass::RestoringCrossing;
    }
    CatalystBracketClass::Disordered
}

pub fn provisional_catalyst_rate_passes(
    v_low: f64,
    v_high: f64,
    retention: f64,
    catalyst_explosion: bool,
    resource_exhausted: bool,
) -> bool {
    v_low > 0.0
        && v_high < 0.0
        && retention >= 0.80
        && !catalyst_explosion
        && !resource_exhausted
}

pub fn catalyst_failure_gate(classes: &[CatalystBracketClass]) -> Option<&'static str> {
    if classes
        .iter()
        .all(|c| matches!(c, CatalystBracketClass::AllDecline))
    {
        return Some("D007_NO_CATALYST_NULLCLINE");
    }
    if classes
        .iter()
        .all(|c| matches!(c, CatalystBracketClass::AllGrow))
    {
        return Some("D007_UNBOUNDED_CATALYST");
    }
    None
}

pub fn joint_candidate_count_bounded(n: usize) -> bool {
    n <= 9
}

pub fn result_schema_is_complete(keys: &[&str]) -> bool {
    D007_REQUIRED_RESULT_FIELDS
        .iter()
        .all(|req| keys.iter().any(|k| k == req))
}

pub fn clean_termination_is_required(clean: bool, usable: bool) -> bool {
    // Results used for candidate selection must be cleanly terminated.
    !usable || clean
}

pub fn accounting_is_required(has_accounting: bool, usable: bool) -> bool {
    !usable || has_accounting
}

pub fn candidate_parameters_match_hash(
    declared_k_iface: f64,
    declared_k_rep: f64,
    hashed_k_iface: f64,
    hashed_k_rep: f64,
) -> bool {
    (declared_k_iface - hashed_k_iface).abs() < 1e-15
        && (declared_k_rep - hashed_k_rep).abs() < 1e-15
}

pub fn nullclines_must_intersect(radius_hit: bool, catalyst_hit: bool, same_region: bool) -> bool {
    radius_hit && catalyst_hit && same_region
}

pub fn joint_jacobian_must_be_stable(max_real_eigenvalue: f64) -> bool {
    max_real_eigenvalue < 0.0
}

pub fn classify_joint_intersection(
    radius_nullcline: bool,
    catalyst_nullcline: bool,
    same_region: bool,
    class: FixedPointClass,
    numerically_invalid: bool,
    robust_basin: bool,
) -> JointCandidateClass {
    if numerically_invalid {
        return JointCandidateClass::NumericallyInvalid;
    }
    if !radius_nullcline {
        return JointCandidateClass::NoRadiusNullcline;
    }
    if !catalyst_nullcline {
        return JointCandidateClass::NoCatalystNullcline;
    }
    if !same_region {
        return JointCandidateClass::NullclinesDisjoint;
    }
    match class {
        FixedPointClass::Stable => {
            if robust_basin {
                JointCandidateClass::StableIntersectionRobust
            } else {
                JointCandidateClass::StableIntersectionNarrow
            }
        }
        FixedPointClass::SaddleLike => JointCandidateClass::SaddleIntersection,
        FixedPointClass::Unstable => JointCandidateClass::UnstableIntersection,
        FixedPointClass::Indeterminate => JointCandidateClass::NumericallyInvalid,
    }
}

pub fn saddle_intersection_fails(class: JointCandidateClass) -> bool {
    matches!(class, JointCandidateClass::SaddleIntersection)
}

pub fn disjoint_nullclines_fail(class: JointCandidateClass) -> bool {
    matches!(class, JointCandidateClass::NullclinesDisjoint)
}

pub fn refined_center_requires_four_of_five(successes: usize) -> bool {
    successes >= 4
}

pub fn refined_neighbors_require_contiguous_patch(
    neighbor_recipes_ok: usize,
    contiguous: bool,
) -> bool {
    neighbor_recipes_ok >= 3 && contiguous
}

pub fn parameter_domain_rejection(
    no_structural_nullcline: bool,
    no_catalyst_nullcline: bool,
    nullclines_disjoint: bool,
    only_unstable: bool,
    isolated_noise_sensitive: bool,
    k_rep_above_3x: bool,
    k_iface_below_0_5x: bool,
    depends_on_aged_snapshot: bool,
    turnover_cannot_pass: bool,
) -> Option<&'static str> {
    if no_structural_nullcline
        || no_catalyst_nullcline
        || nullclines_disjoint
        || only_unstable
        || isolated_noise_sensitive
        || k_rep_above_3x
        || k_iface_below_0_5x
        || depends_on_aged_snapshot
        || turnover_cannot_pass
    {
        Some("D007_FIVE_FIELD_MODEL_REJECTED")
    } else {
        None
    }
}

pub fn reference_flow_direction_ok(v_r: f64, v_c_inside: f64) -> bool {
    v_r > 0.0 && v_c_inside < 0.0
}

pub fn structural_factors() -> [f64; 7] {
    [0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80]
}

pub fn catalyst_factors_around_center() -> [f64; 5] {
    [0.75, 0.875, 1.0, 1.125, 1.25]
}

/// Retain at most three neighboring factors that pass (or approach) the gate.
pub fn retain_neighboring_factors(factors: &[f64], pass: &[bool], max_n: usize) -> Vec<f64> {
    assert_eq!(factors.len(), pass.len());
    let mut idxs: Vec<usize> = pass
        .iter()
        .enumerate()
        .filter_map(|(i, p)| if *p { Some(i) } else { None })
        .collect();
    if idxs.is_empty() {
        // approach: keep three closest to a sign-change if any mixed pattern provided via pass=false all
        return factors.iter().copied().take(max_n.min(factors.len())).collect();
    }
    idxs.sort_unstable();
    // expand to neighboring indices
    let mut set = std::collections::BTreeSet::new();
    for &i in &idxs {
        set.insert(i);
        if i > 0 {
            set.insert(i - 1);
        }
        if i + 1 < factors.len() {
            set.insert(i + 1);
        }
    }
    set.into_iter()
        .take(max_n)
        .map(|i| factors[i])
        .collect()
}
