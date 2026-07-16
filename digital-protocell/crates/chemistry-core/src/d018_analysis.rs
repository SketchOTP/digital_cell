//! D-018 structural basis, radius scaling, nullcline feasibility, and conclusions.

use crate::d012_analysis::{D012_GLOBAL_RATE_MAX_FACTOR, D012_GLOBAL_RATE_MIN_FACTOR};
use crate::d018_provenance::HistoricalWasteOriginClass;
use serde::{Deserialize, Serialize};

/// Frozen D-012 conservative-v2 analytical structure-rate reference (ledger-calibrated).
pub const D018_ANALYTICAL_K_STRUCTURE: f64 = 1.0812170527078209;

/// Frozen candidate rates (identity hash 9a452d34…).
pub const D018_FROZEN_K_STRUCTURE: f64 = 1.0812170527078209;

pub const D018_RADII: [f64; 6] = [14.0, 18.0, 22.0, 26.0, 30.0, 34.0];
pub const D018_PREBALANCE_RADII: [f64; 3] = [18.0, 22.0, 26.0];
pub const D018_PREBALANCE_FACTORS: [f64; 3] = [0.75, 1.0, 1.25];
pub const D018_Q_STRUCTURE_PROMOTE_MIN: f64 = 0.50;
pub const D018_Q_STRUCTURE_PROMOTE_MAX: f64 = 2.00;
pub const D018_STATE_CHANGE_MAX: f64 = 0.05;

/// Authorized D-012 global domain for k_structure relative to analytical estimate.
pub fn authorized_k_structure_domain() -> (f64, f64) {
    (
        D018_ANALYTICAL_K_STRUCTURE * D012_GLOBAL_RATE_MIN_FACTOR,
        D018_ANALYTICAL_K_STRUCTURE * D012_GLOBAL_RATE_MAX_FACTOR,
    )
}

pub fn k_structure_inside_authorized(k: f64) -> bool {
    let (lo, hi) = authorized_k_structure_domain();
    k >= lo && k <= hi
}

/// Raw-basis required rate: L / B (not k/Q unless identity verified).
pub fn required_k_structure(b_structure: f64, l_structure: f64) -> f64 {
    l_structure / b_structure.max(1e-30)
}

/// Production basis from ledger: virtual_production / (k_structure * dt_window).
pub fn production_basis_from_extent(virtual_production: f64, k_structure: f64) -> f64 {
    virtual_production / k_structure.max(1e-30)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct StructureBasisPoint {
    pub radius: f64,
    pub b_structure: f64,
    pub l_structure: f64,
    pub k_required: f64,
    pub k_current: f64,
    pub required_over_current: f64,
    pub authorized_min: f64,
    pub authorized_max: f64,
    pub inside_authorized_domain: bool,
    pub sampling_window_steps: u64,
    pub constraint_fraction_of_total_w: f64,
    pub window_usable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RadiusScalingFit {
    pub production_exponent_p: f64,
    pub decay_exponent_q: f64,
    pub required_rate_exponent: f64,
    pub production_residual_rms: f64,
    pub decay_residual_rms: f64,
    pub radius_lo: f64,
    pub radius_hi: f64,
    pub production_scaling_class: ScalingClass,
    pub decay_scaling_class: ScalingClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalingClass {
    InterfaceScaled,
    BulkScaled,
    Mixed,
}

/// Log-log least squares: ln y = ln a + p ln R. Returns (p, rms residual in log space).
pub fn power_law_exponent(radii: &[f64], values: &[f64]) -> (f64, f64) {
    assert_eq!(radii.len(), values.len());
    let n = radii.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0);
    }
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xx = 0.0;
    let mut sum_xy = 0.0;
    let mut count = 0.0;
    for (&r, &v) in radii.iter().zip(values.iter()) {
        if r > 0.0 && v > 0.0 {
            let x = r.ln();
            let y = v.ln();
            sum_x += x;
            sum_y += y;
            sum_xx += x * x;
            sum_xy += x * y;
            count += 1.0;
        }
    }
    if count < 2.0 {
        return (0.0, 0.0);
    }
    let denom = count * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-30 {
        return (0.0, 0.0);
    }
    let p = (count * sum_xy - sum_x * sum_y) / denom;
    let ln_a = (sum_y - p * sum_x) / count;
    let mut rss = 0.0;
    for (&r, &v) in radii.iter().zip(values.iter()) {
        if r > 0.0 && v > 0.0 {
            let pred = ln_a + p * r.ln();
            let err = v.ln() - pred;
            rss += err * err;
        }
    }
    (p, (rss / count).sqrt())
}

pub fn classify_scaling_exponent(exponent: f64) -> ScalingClass {
    // Interface ~ R^1, bulk ~ R^2; mixed otherwise.
    if (exponent - 1.0).abs() <= 0.35 {
        ScalingClass::InterfaceScaled
    } else if (exponent - 2.0).abs() <= 0.35 {
        ScalingClass::BulkScaled
    } else {
        ScalingClass::Mixed
    }
}

pub fn fit_radius_scaling(points: &[StructureBasisPoint]) -> Option<RadiusScalingFit> {
    let usable: Vec<_> = points.iter().filter(|p| p.window_usable && p.b_structure > 0.0).collect();
    if usable.len() < 3 {
        return None;
    }
    let radii: Vec<f64> = usable.iter().map(|p| p.radius).collect();
    let b: Vec<f64> = usable.iter().map(|p| p.b_structure).collect();
    let l: Vec<f64> = usable.iter().map(|p| p.l_structure).collect();
    let (p, p_rms) = power_law_exponent(&radii, &b);
    let (q, q_rms) = power_law_exponent(&radii, &l);
    Some(RadiusScalingFit {
        production_exponent_p: p,
        decay_exponent_q: q,
        required_rate_exponent: q - p,
        production_residual_rms: p_rms,
        decay_residual_rms: q_rms,
        radius_lo: radii.iter().copied().fold(f64::INFINITY, f64::min),
        radius_hi: radii.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        production_scaling_class: classify_scaling_exponent(p),
        decay_scaling_class: classify_scaling_exponent(q),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StructuralNullclineClass {
    StructuralNullclineExistsInDomain,
    StructuralBalancePointWithoutRestoringCrossing,
    StructuralBalanceOutsideRateDomain,
    NoStructuralNullcline,
    InsufficientValidWindows,
}

/// g_structure(R) = k * B(R) - L(R) for constant k.
pub fn g_structure_at(k: f64, b: f64, l: f64) -> f64 {
    k * b - l
}

pub fn restoring_crossing_signs(g_below: f64, g_center: f64, g_above: f64) -> bool {
    g_below > 0.0 && g_above < 0.0 && g_center.abs() <= 0.25 * g_below.abs().max(g_above.abs()).max(1e-6)
}

pub fn classify_structural_nullcline(
    points: &[StructureBasisPoint],
    k: f64,
) -> StructuralNullclineClass {
    let usable: Vec<_> = points.iter().filter(|p| p.window_usable).collect();
    if usable.len() < 3 {
        return StructuralNullclineClass::InsufficientValidWindows;
    }
    let any_inside = usable.iter().any(|p| k_structure_inside_authorized(p.k_required));
    if !any_inside && usable.iter().all(|p| !k_structure_inside_authorized(p.k_required)) {
        return StructuralNullclineClass::StructuralBalanceOutsideRateDomain;
    }
    let mut pairs: Vec<(f64, f64)> = usable
        .iter()
        .map(|p| (p.radius, g_structure_at(k, p.b_structure, p.l_structure)))
        .collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut sign_change = false;
    for w in pairs.windows(2) {
        if w[0].1 == 0.0 || w[1].1 == 0.0 || w[0].1.signum() != w[1].1.signum() {
            sign_change = true;
            break;
        }
    }
    let center = pairs.iter().find(|p| (p.0 - 22.0).abs() < 1e-9).map(|p| p.1);
    if let Some(g_c) = center {
        if g_c.abs() < 1e-3 && !sign_change {
            return StructuralNullclineClass::StructuralBalancePointWithoutRestoringCrossing;
        }
    }
    // Check R18/R22/R26 restoring pattern if present.
    let g18 = pairs.iter().find(|p| (p.0 - 18.0).abs() < 1e-9).map(|p| p.1);
    let g22 = pairs.iter().find(|p| (p.0 - 22.0).abs() < 1e-9).map(|p| p.1);
    let g26 = pairs.iter().find(|p| (p.0 - 26.0).abs() < 1e-9).map(|p| p.1);
    if let (Some(a), Some(b), Some(c)) = (g18, g22, g26) {
        if restoring_crossing_signs(a, b, c) {
            return StructuralNullclineClass::StructuralNullclineExistsInDomain;
        }
    }
    if sign_change && k_structure_inside_authorized(k) {
        // Sign change exists but not the restoring pattern at R18/22/26.
        return StructuralNullclineClass::StructuralBalancePointWithoutRestoringCrossing;
    }
    if !sign_change {
        StructuralNullclineClass::NoStructuralNullcline
    } else {
        StructuralNullclineClass::NoStructuralNullcline
    }
}

/// Clip pre-balance candidates to authorized domain; at most three.
pub fn prebalance_k_candidates(k_required: f64) -> Vec<f64> {
    let (lo, hi) = authorized_k_structure_domain();
    let mut out = Vec::new();
    for f in D018_PREBALANCE_FACTORS {
        let k = (f * k_required).clamp(lo, hi);
        if !out.iter().any(|x: &f64| (*x - k).abs() < 1e-12) {
            out.push(k);
        }
        if out.len() == 3 {
            break;
        }
    }
    out
}

pub fn promote_structure_candidate(
    q_structure: f64,
    constraint_fraction_of_total_w: f64,
    extinct: bool,
    ceiling: bool,
    accounting_ok: bool,
) -> bool {
    promote_structure_candidate_with_g(
        q_structure,
        0.0,
        constraint_fraction_of_total_w,
        extinct,
        ceiling,
        accounting_ok,
    )
}

pub fn promote_structure_candidate_with_g(
    q_structure: f64,
    g_structure: f64,
    constraint_fraction_of_total_w: f64,
    extinct: bool,
    ceiling: bool,
    accounting_ok: bool,
) -> bool {
    !extinct
        && !ceiling
        && accounting_ok
        && q_structure >= D018_Q_STRUCTURE_PROMOTE_MIN
        && q_structure <= D018_Q_STRUCTURE_PROMOTE_MAX
        && constraint_fraction_of_total_w <= 0.05
        && g_structure.abs() <= (q_structure.max(1.0) * 1e-1).max(1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnconstrainedClass {
    StructureCollapseLimitsWSource,
    IntrinsicWasteUnboundedWithoutConstraint,
    UnconstrainedStructureStable,
    FragmentationBeforeDiagnosis,
    NumericalFailure,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D018PrimaryConclusion {
    D018ConstrainedAssayRecoverable,
    D018ConstraintWasteArtifactConfirmed,
    D018StructureBalanceOutsideRateDomain,
    D018SurfaceVolumeScalingIncompatible,
    D018IntrinsicStructureWasteFailure,
    D018StructuralNullclineRecovered,
    D018ProvenanceTracerInvalid,
    D018NumericalFailure,
    D018Inconclusive,
    D018Fail,
}

pub fn d018_primary_conclusion_tag(c: D018PrimaryConclusion) -> &'static str {
    match c {
        D018PrimaryConclusion::D018ConstrainedAssayRecoverable => {
            "D-018-constrained-assay-recoverable"
        }
        D018PrimaryConclusion::D018ConstraintWasteArtifactConfirmed => {
            "D-018-inconclusive" // artifact-only uses inconclusive tag family when sole primary
        }
        D018PrimaryConclusion::D018StructureBalanceOutsideRateDomain => {
            "D-018-structure-balance-outside-domain"
        }
        D018PrimaryConclusion::D018SurfaceVolumeScalingIncompatible => {
            "D-018-surface-volume-scaling-incompatible"
        }
        D018PrimaryConclusion::D018IntrinsicStructureWasteFailure => {
            "D-018-intrinsic-structure-waste-failure"
        }
        D018PrimaryConclusion::D018StructuralNullclineRecovered => {
            "D-018-structural-nullcline-recovered"
        }
        D018PrimaryConclusion::D018ProvenanceTracerInvalid
        | D018PrimaryConclusion::D018NumericalFailure
        | D018PrimaryConclusion::D018Inconclusive
        | D018PrimaryConclusion::D018Fail => "D-018-inconclusive",
    }
}

/// Select primary + optional subsidiary per §25.
pub fn select_d018_conclusion(
    tracer_valid: bool,
    historical: HistoricalWasteOriginClass,
    unconstrained: UnconstrainedClass,
    nullcline: StructuralNullclineClass,
    scaling: Option<&RadiusScalingFit>,
    assay_recoverable: bool,
    numerical_failure: bool,
) -> (D018PrimaryConclusion, Option<D018PrimaryConclusion>) {
    if !tracer_valid {
        return (D018PrimaryConclusion::D018ProvenanceTracerInvalid, None);
    }
    if numerical_failure {
        return (D018PrimaryConclusion::D018NumericalFailure, None);
    }
    if assay_recoverable {
        return (D018PrimaryConclusion::D018ConstrainedAssayRecoverable, None);
    }

    let artifact = matches!(
        historical,
        HistoricalWasteOriginClass::ConstraintWasteDominant
            | HistoricalWasteOriginClass::MixedStructuralWaste
    ) && matches!(
        unconstrained,
        UnconstrainedClass::StructureCollapseLimitsWSource
    );

    let subsidiary = if artifact {
        Some(D018PrimaryConclusion::D018ConstraintWasteArtifactConfirmed)
    } else {
        None
    };

    if matches!(
        unconstrained,
        UnconstrainedClass::IntrinsicWasteUnboundedWithoutConstraint
    ) && matches!(
        historical,
        HistoricalWasteOriginClass::EndogenousWasteDominant
    ) {
        return (
            D018PrimaryConclusion::D018IntrinsicStructureWasteFailure,
            subsidiary,
        );
    }

    match nullcline {
        StructuralNullclineClass::StructuralNullclineExistsInDomain => {
            return (
                D018PrimaryConclusion::D018StructuralNullclineRecovered,
                subsidiary,
            );
        }
        StructuralNullclineClass::StructuralBalanceOutsideRateDomain => {
            return (
                D018PrimaryConclusion::D018StructureBalanceOutsideRateDomain,
                subsidiary,
            );
        }
        StructuralNullclineClass::NoStructuralNullcline
        | StructuralNullclineClass::StructuralBalancePointWithoutRestoringCrossing => {
            if let Some(fit) = scaling {
                let incompatible = fit.production_scaling_class != fit.decay_scaling_class
                    || fit.required_rate_exponent.abs() > 0.5;
                if incompatible
                    || matches!(
                        nullcline,
                        StructuralNullclineClass::NoStructuralNullcline
                    )
                {
                    return (
                        D018PrimaryConclusion::D018SurfaceVolumeScalingIncompatible,
                        subsidiary.or(Some(
                            D018PrimaryConclusion::D018ConstraintWasteArtifactConfirmed,
                        )),
                    );
                }
            }
            return (
                D018PrimaryConclusion::D018SurfaceVolumeScalingIncompatible,
                subsidiary,
            );
        }
        StructuralNullclineClass::InsufficientValidWindows => {
            if artifact {
                return (
                    D018PrimaryConclusion::D018ConstraintWasteArtifactConfirmed,
                    None,
                );
            }
            return (D018PrimaryConclusion::D018Inconclusive, None);
        }
    }
}

pub fn classify_unconstrained(
    structure_frac_remaining: f64,
    w_declined_with_structure: bool,
    w_still_unbounded: bool,
    structure_stable: bool,
    fragmented: bool,
    numerical: bool,
) -> UnconstrainedClass {
    if numerical {
        return UnconstrainedClass::NumericalFailure;
    }
    if fragmented {
        return UnconstrainedClass::FragmentationBeforeDiagnosis;
    }
    if structure_stable {
        return UnconstrainedClass::UnconstrainedStructureStable;
    }
    if structure_frac_remaining <= 0.50 {
        if w_still_unbounded && !w_declined_with_structure {
            return UnconstrainedClass::IntrinsicWasteUnboundedWithoutConstraint;
        }
        if w_declined_with_structure {
            return UnconstrainedClass::StructureCollapseLimitsWSource;
        }
    }
    UnconstrainedClass::Inconclusive
}
