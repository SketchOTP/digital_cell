//! D-050 catalyst-saturating volume activation (schema 2) analysis helpers.
//!
//! Schema 1 (historical, preserved): `r = k · C · N · F` with `k = 0.020`.
//! Schema 2 (proposed): `r = V_A · H(φ) · q_C(C) · (N/N_ref) · (F/F_ref)`
//! with `q_C = C/(K_C+C)`, `H(φ)=φ²(3−2φ)`, `N_ref=F_ref=1`.

use crate::d046_analysis::{through_origin_alpha, DemandStateRow, ModelFitReport};
use crate::d047_analysis::{candidate_a_rate, D047_HISTORICAL_K, D047_K_C_MEMBRANE};
use crate::fields::interior_weight;
use serde::{Deserialize, Serialize};

pub const D050_PROJECT_ID: &str = "D-050";
pub const D050_AGENT_MEMORY_ID: &str =
    "D-20260720-d050-coupled-catalyst-saturating-activation-repair";
pub const D050_STARTING_COMMIT: &str = "479ca35";
pub const D050_STARTING_TAG: &str = "D-049-coupled-aps-collapse-audit";
pub const D050_RECORD: &str = "COUPLED_HISTORICAL_ACTIVATION_CAPACITY_REJECTED";
pub const D050_HISTORICAL_K: f64 = D047_HISTORICAL_K;
pub const D050_N_REF: f64 = 1.0;
pub const D050_F_REF: f64 = 1.0;
pub const D050_RADIUS: f64 = 22.0;
pub const D050_THETA: f64 = 0.6;
pub const D050_DEFAULT_HORIZON: u64 = 200_000;
pub const D050_WINDOW: u64 = 10_000;
pub const D050_RETENTION_MIN: f64 = 0.80;
pub const D050_A_COLLAPSE_MAX: f64 = 0.10;
pub const D050_LOCALIZATION_MIN: f64 = 0.95;
pub const D050_NET_S_TOL: f64 = 1.0e-4;

pub const D050_FIT_TRAIN_MEDIAN_MAX: f64 = 0.15;
pub const D050_FIT_HOLD_MEDIAN_MAX: f64 = 0.20;
pub const D050_FIT_HOLD_MAX_ERR: f64 = 0.35;
pub const D050_BOOTSTRAP_SPREAD_MAX: f64 = 0.50;
pub const D050_LOO_FACTOR_MAX: f64 = 2.0;

pub const ACTIVATION_SCHEMA_HISTORICAL: u32 = 1;
pub const ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME: u32 = 2;
pub const ACTIVATION_SCHEMA_BOUNDED_NF: u32 = 3;
pub const ACTIVATION_SCHEMA_2_NAME: &str = "catalyst_saturating_volume_activation";
pub const EQUATION_VERSION_V13: &str = "membrane_metabolism_v13_catalyst_saturating_activation";

/// Schema 1 historical activation.
#[inline]
pub fn activation_rate_schema1(k: f64, c: f64, n: f64, f: f64) -> f64 {
    candidate_a_rate(k, c, n, f)
}

/// Schema 2 catalyst-saturating volume activation:
/// `V_A · H(φ) · q(C) · (N/N_ref) · (F/F_ref)`.
#[inline]
pub fn schema2_activation_rate(
    v_a: f64,
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    k_c: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    let h = interior_weight(phi);
    let c_pos = c.max(0.0);
    let q = c_pos / (k_c + c_pos).max(1e-18);
    let n_hat = n.max(0.0) / n_ref.max(1e-18);
    let f_hat = f.max(0.0) / f_ref.max(1e-18);
    v_a.max(0.0) * h * q * n_hat * f_hat
}

/// Production dispatcher used by runtime and observer parity checks.
#[inline]
pub fn production_activation_rate(
    activation_schema: u32,
    k_or_v_a: f64,
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    k_c: f64,
    n_ref: f64,
    f_ref: f64,
) -> f64 {
    match activation_schema {
        ACTIVATION_SCHEMA_BOUNDED_NF => {
            k_or_v_a.max(0.0)
                * interior_weight(phi)
                * q_c_saturation(c, k_c)
                * q_c_saturation(n, n_ref)
                * q_c_saturation(f, f_ref)
        }
        ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME => {
            schema2_activation_rate(k_or_v_a, phi, c, n, f, k_c, n_ref, f_ref)
        }
        _ => activation_rate_schema1(k_or_v_a, c, n, f),
    }
}

#[inline]
pub fn q_c_saturation(c: f64, k_c: f64) -> f64 {
    let c_pos = c.max(0.0);
    c_pos / (k_c + c_pos).max(1e-18)
}

pub fn schema2_zero_resource_controls(v_a: f64, k_c: f64) -> bool {
    schema2_activation_rate(v_a, 1.0, 0.0, 1.0, 1.0, k_c, 1.0, 1.0).abs() < 1e-15
        && schema2_activation_rate(v_a, 1.0, 1.0, 0.0, 1.0, k_c, 1.0, 1.0).abs() < 1e-15
        && schema2_activation_rate(v_a, 1.0, 1.0, 1.0, 0.0, k_c, 1.0, 1.0).abs() < 1e-15
}

pub fn schema2_bounded_high_c(v_a: f64, k_c: f64) -> bool {
    let r1 = schema2_activation_rate(v_a, 1.0, 1.0, 1.0, 1.0, k_c, 1.0, 1.0);
    let r10 = schema2_activation_rate(v_a, 1.0, 10.0, 1.0, 1.0, k_c, 1.0, 1.0);
    let r100 = schema2_activation_rate(v_a, 1.0, 100.0, 1.0, 1.0, k_c, 1.0, 1.0);
    r10 > r1 && r100 > r10 && r100 < v_a * 1.01
}

pub fn schema2_monotonic_c_n_f(v_a: f64, k_c: f64) -> bool {
    let base = schema2_activation_rate(v_a, 1.0, 0.5, 0.5, 0.5, k_c, 1.0, 1.0);
    schema2_activation_rate(v_a, 1.0, 0.8, 0.5, 0.5, k_c, 1.0, 1.0) >= base
        && schema2_activation_rate(v_a, 1.0, 0.5, 0.8, 0.5, k_c, 1.0, 1.0) >= base
        && schema2_activation_rate(v_a, 1.0, 0.5, 0.5, 0.8, k_c, 1.0, 1.0) >= base
}

/// Preregistered Gate-1 training labels (fixed biochemistry only).
pub fn d050_training_labels() -> &'static [&'static str] {
    &[
        "R16",
        "R22",
        "R32",
        "low_c",
        "med_c",
        "high_c",
        "analytic_early",
        "restored_early",
    ]
}

/// Preregistered Gate-1 holdout labels.
pub fn d050_holdout_labels() -> &'static [&'static str] {
    &[
        "low_n",
        "low_f",
        "high_nf",
        "analytic_late",
        "restored_late",
        "s_low",
        "s_damaged25",
    ]
}

pub fn is_fixed_biochemistry_row(row: &DemandStateRow) -> bool {
    (row.k_precursor_scale - 1.0).abs() < 1e-12 && (row.k_structure_scale - 1.0).abs() < 1e-12
}

/// Reconstruct D-047 Model C for identification:
/// `L_A ≈ V_A · V · q(C)` under fixed biochemistry (sealed D-047 proxy).
/// Production schema-2 still evaluates `V_A · H(φ) · q(C) · n · f` with N_ref=F_ref=1;
/// at reference reservoirs this matches the volumetric Model C scale.
pub fn fit_schema2_v_a(
    train: &[DemandStateRow],
    hold: &[DemandStateRow],
    k_c: f64,
) -> ModelFitReport {
    let xs: Vec<f64> = train
        .iter()
        .map(|r| {
            let q = r.c / f64::max(k_c + r.c, 1e-18);
            r.interior_volume * q
        })
        .collect();
    let ys: Vec<f64> = train.iter().map(|r| r.l_a).collect();
    let lambda = through_origin_alpha(&xs, &ys);
    evaluate_schema2_fit("schema2_catalyst_saturating_volume", lambda, k_c, train, hold)
}

fn evaluate_schema2_fit(
    name: &str,
    v_a: f64,
    k_c: f64,
    train: &[DemandStateRow],
    hold: &[DemandStateRow],
) -> ModelFitReport {
    let pred = |r: &DemandStateRow, lam: f64| {
        let q = r.c / f64::max(k_c + r.c, 1e-18);
        lam * r.interior_volume * q
    };
    let mut errs = Vec::new();
    for r in hold {
        let p = pred(r, v_a);
        if r.l_a > 1e-18 {
            errs.push(((r.l_a - p) / r.l_a).abs());
        }
    }
    let med = median_f64(errs.clone());
    let maxe = errs.iter().copied().fold(0.0_f64, f64::max);

    let err_at = |lab: &str| {
        hold.iter()
            .chain(train.iter())
            .find(|r| r.label == lab)
            .map(|r| {
                let p = pred(r, v_a);
                if r.l_a > 1e-18 {
                    (r.l_a - p) / r.l_a
                } else {
                    0.0
                }
            })
    };
    let radius_bias = match (err_at("R16"), err_at("R32")) {
        (Some(a), Some(b)) => a * b < 0.0 && (a - b).abs() > 0.15,
        _ => false,
    };
    let catalyst_bias = match (err_at("low_c"), err_at("high_c")) {
        (Some(a), Some(b)) => a * b < 0.0 && (a - b).abs() > 0.15,
        _ => false,
    };

    let mut lambdas = Vec::new();
    for drop in 0..train.len() {
        let xs: Vec<f64> = train
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != drop)
            .map(|(_, r)| {
                let q = r.c / f64::max(k_c + r.c, 1e-18);
                r.interior_volume * q
            })
            .collect();
        let ys: Vec<f64> = train
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != drop)
            .map(|(_, r)| r.l_a)
            .collect();
        if !xs.is_empty() {
            lambdas.push(through_origin_alpha(&xs, &ys));
        }
    }
    let (spread, loo_ok) = loo_stats(&lambdas, v_a);

    let train_errs: Vec<f64> = train
        .iter()
        .filter(|r| r.l_a > 1e-18)
        .map(|r| ((r.l_a - pred(r, v_a)) / r.l_a).abs())
        .collect();
    let train_med = median_f64(train_errs);

    ModelFitReport {
        name: name.into(),
        lambda: v_a,
        median_hold_err: med,
        max_hold_err: maxe,
        radius_bias,
        catalyst_bias,
        starvation_false_positive: false,
        bootstrap_spread: spread,
        loo_factor_ok: loo_ok,
        adequate: train_med <= D050_FIT_TRAIN_MEDIAN_MAX
            && med <= D050_FIT_HOLD_MEDIAN_MAX
            && maxe <= D050_FIT_HOLD_MAX_ERR
            && spread <= D050_BOOTSTRAP_SPREAD_MAX
            && loo_ok
            && v_a.is_finite()
            && v_a > 0.0
            && k_c.is_finite()
            && k_c > 0.0
            && !radius_bias
            && !catalyst_bias,
    }
}

fn loo_stats(lambdas: &[f64], center: f64) -> (f64, bool) {
    if lambdas.is_empty() || center.abs() < 1e-18 {
        return (0.0, true);
    }
    let mean = lambdas.iter().sum::<f64>() / lambdas.len() as f64;
    let var = lambdas
        .iter()
        .map(|l| (l - mean).powi(2))
        .sum::<f64>()
        / lambdas.len().max(1) as f64;
    let spread = var.sqrt() / center.abs();
    let loo_ok = lambdas
        .iter()
        .all(|l| *l > 0.0 && (*l / center).max(center / l) <= D050_LOO_FACTOR_MAX);
    (spread, loo_ok)
}

fn median_f64(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len() % 2 == 0 {
        0.5 * (v[mid - 1] + v[mid])
    } else {
        v[mid]
    }
}

/// Joint identification: freeze K_C from membrane default, fit V_A; optionally scan nearby K_C.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema2Identification {
    pub v_a: f64,
    pub k_c: f64,
    pub train_median_err: f64,
    pub hold_median_err: f64,
    pub hold_max_err: f64,
    pub bootstrap_spread: f64,
    pub loo_factor_ok: bool,
    pub radius_bias: bool,
    pub catalyst_bias: bool,
    pub k_c_in_tested_range: bool,
    pub identifiable: bool,
    pub report: ModelFitReport,
}

pub fn identify_schema2_parameters(
    train: &[DemandStateRow],
    hold: &[DemandStateRow],
    c_min: f64,
    c_max: f64,
) -> Schema2Identification {
    let k_c = D047_K_C_MEMBRANE;
    let report = fit_schema2_v_a(train, hold, k_c);
    let train_med = {
        let pred = |r: &DemandStateRow| {
            let q = r.c / f64::max(k_c + r.c, 1e-18);
            report.lambda * r.interior_volume * q
        };
        median_f64(
            train
                .iter()
                .filter(|r| r.l_a > 1e-18)
                .map(|r| ((r.l_a - pred(r)) / r.l_a).abs())
                .collect(),
        )
    };
    let k_c_in_range = k_c >= c_min && k_c <= c_max;
    let identifiable = report.adequate && k_c_in_range;
    Schema2Identification {
        v_a: report.lambda,
        k_c,
        train_median_err: train_med,
        hold_median_err: report.median_hold_err,
        hold_max_err: report.max_hold_err,
        bootstrap_spread: report.bootstrap_spread,
        loo_factor_ok: report.loo_factor_ok,
        radius_bias: report.radius_bias,
        catalyst_bias: report.catalyst_bias,
        k_c_in_tested_range: k_c_in_range,
        identifiable,
        report,
    }
}

/// Gate-5 V_A candidate multipliers around fitted center.
pub fn v_a_candidate_multipliers() -> &'static [f64] {
    &[0.75, 1.0, 1.25]
}

pub fn build_v_a_candidates(fitted: f64) -> Vec<f64> {
    v_a_candidate_multipliers()
        .iter()
        .map(|m| fitted * m)
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect()
}

/// Select smallest V_A among ordered candidates that pass a boolean predicate.
pub fn select_smallest_passing_v_a(candidates: &[f64], passes: impl Fn(f64) -> bool) -> Option<f64> {
    let mut ordered = candidates.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ordered.into_iter().find(|v| passes(*v))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D050PrimaryConclusion {
    StageERecovered,
    CoupledActivationRepairQualifiedStageEBlocked,
    D049CoupledFailureNotReproduced,
    CatalystSaturationNotIdentifiable,
    ShadowActivationRepairFailure,
    ActivationImplementationFailure,
    CoupledActivationCapacityNotRecovered,
    FoundationalActivationRegression,
    NoHealthyCoupledAttractor,
    CoupledBasinNotRecovered,
    ContinuousMembraneReplacementFailure,
    LocalDamageRepairFailure,
    ResourceDependenceFailure,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D050PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StageERecovered => "D050_STAGE_E_RECOVERED",
            Self::CoupledActivationRepairQualifiedStageEBlocked => {
                "D050_COUPLED_ACTIVATION_REPAIR_QUALIFIED_STAGE_E_BLOCKED"
            }
            Self::D049CoupledFailureNotReproduced => "D050_D049_COUPLED_FAILURE_NOT_REPRODUCED",
            Self::CatalystSaturationNotIdentifiable => "D050_CATALYST_SATURATION_NOT_IDENTIFIABLE",
            Self::ShadowActivationRepairFailure => "D050_SHADOW_ACTIVATION_REPAIR_FAILURE",
            Self::ActivationImplementationFailure => "D050_ACTIVATION_IMPLEMENTATION_FAILURE",
            Self::CoupledActivationCapacityNotRecovered => {
                "D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED"
            }
            Self::FoundationalActivationRegression => "D050_FOUNDATIONAL_ACTIVATION_REGRESSION",
            Self::NoHealthyCoupledAttractor => "D050_NO_HEALTHY_COUPLED_ATTRACTOR",
            Self::CoupledBasinNotRecovered => "D050_COUPLED_BASIN_NOT_RECOVERED",
            Self::ContinuousMembraneReplacementFailure => {
                "D050_CONTINUOUS_MEMBRANE_REPLACEMENT_FAILURE"
            }
            Self::LocalDamageRepairFailure => "D050_LOCAL_DAMAGE_REPAIR_FAILURE",
            Self::ResourceDependenceFailure => "D050_RESOURCE_DEPENDENCE_FAILURE",
            Self::StageEMembraneContractFailure => "D050_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D050_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D050_NUMERICAL_FAILURE",
            Self::Fail => "D050_FAIL",
        }
    }
}

/// Observer/runtime parity for a single local state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationParityCheck {
    pub observer_rate: f64,
    pub runtime_rate: f64,
    pub pass: bool,
}

pub fn check_schema2_parity(
    v_a: f64,
    phi: f64,
    c: f64,
    n: f64,
    f: f64,
    k_c: f64,
) -> ActivationParityCheck {
    let observer = schema2_activation_rate(v_a, phi, c, n, f, k_c, D050_N_REF, D050_F_REF);
    let runtime = production_activation_rate(
        ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
        v_a,
        phi,
        c,
        n,
        f,
        k_c,
        D050_N_REF,
        D050_F_REF,
    );
    ActivationParityCheck {
        observer_rate: observer,
        runtime_rate: runtime,
        pass: (observer - runtime).abs() <= 1e-15 * (1.0 + observer.abs()),
    }
}

/// Exact N/F/A/W stoichiometry for accepted activation extent ξ.
pub fn activation_stoichiometry_ok(xi: f64) -> bool {
    let d = crate::activated_metabolism::activation_isolated_delta(xi);
    // indices: 0φ 1C 2N 3F 4W 5A 6M
    (d[2] + xi).abs() < 1e-15
        && (d[3] + xi).abs() < 1e-15
        && (d[5] - xi).abs() < 1e-15
        && (d[4] - xi).abs() < 1e-15
        && d[1].abs() < 1e-15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema2_zero_and_bound() {
        assert!(schema2_zero_resource_controls(1.0, 0.1));
        assert!(schema2_bounded_high_c(1.0, 0.1));
        assert!(schema2_monotonic_c_n_f(1.0, 0.1));
    }

    #[test]
    fn schema1_preserved() {
        let r = activation_rate_schema1(0.020, 0.4, 0.5, 0.5);
        assert!((r - 0.020 * 0.4 * 0.5 * 0.5).abs() < 1e-15);
    }

    #[test]
    fn smallest_passing_selection() {
        let c = build_v_a_candidates(1.0);
        assert_eq!(c.len(), 3);
        let sel = select_smallest_passing_v_a(&c, |v| v >= 1.0);
        assert_eq!(sel, Some(1.0));
    }
}
