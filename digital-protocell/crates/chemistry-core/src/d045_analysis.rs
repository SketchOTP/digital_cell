//! D-045 fuel-charged catalyst activation — Phase A observer analysis.
//!
//! Gate −1 seals D-044. Gate 0 asks whether authorized A demand scales
//! approximately with total catalyst mass before any `C_star` field is added.
//! Gate 1 holds the QSS charged-catalyst production law for observer fitting.

use serde::{Deserialize, Serialize};

pub const D045_AGENT_MEMORY_ID: &str = "D-20260720-d045-fuel-charged-catalyst-activation-cycle";
pub const D045_STARTING_COMMIT_ENV: &str = "D045_STARTING_COMMIT";
pub const D045_D044_TAG: &str = "D-044-activation-law-fail";
pub const D045_RECORD_BRANCH_CLOSED: &str = "SINGLE_STEP_ACTIVATION_LAW_BRANCH_CLOSED";
pub const D045_HISTORICAL_K: f64 = 0.020;
pub const D045_N_REFERENCE: f64 = 1.0;
pub const D045_F_REFERENCE: f64 = 1.0;

/// Maximum allowed span of catalyst-normalized demand `L_A / M_C`.
pub const D045_DEMAND_DC_MAX_SPAN: f64 = 3.0;
/// Max relative residual of through-origin catalyst-linear fit `L_A ≈ α M_C`.
pub const D045_CATALYST_LINEAR_MAX_REL_ERR: f64 = 0.25;
/// Radius bias: max/min `d_C` across true-radius matched-N/F states.
pub const D045_RADIUS_DC_MAX_SPAN: f64 = 1.5;
/// Superlinear test: L_A must not grow faster than M_C (ratio of spans ≤ 1 + tol).
pub const D045_SUPERLINEAR_TOL: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D045Conclusion {
    FuelChargedActivationQualified,
    D044EvidenceNotSealed,
    CatalystLinearityRejected,
    ChargedCatalystArchitectureRejected,
    CatalystStateMappingUnresolved,
    ChargedCatalystConservationFailure,
    ChargedCatalystKineticsNotIdentifiable,
    CatalystRecyclingNotEstablished,
    ChargedCatalystLawNotPortable,
    ActivationCapacityNotRecovered,
    FoundationalActivationRegression,
    MembraneBasinNotRecovered,
    ContinuousReplacementNotRecovered,
    DamageRepairNotRecovered,
    ResourceDependenceNotEstablished,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D045Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FuelChargedActivationQualified => "D045_FUEL_CHARGED_ACTIVATION_QUALIFIED",
            Self::D044EvidenceNotSealed => "D045_D044_EVIDENCE_NOT_SEALED",
            Self::CatalystLinearityRejected => "D045_CATALYST_LINEARITY_REJECTED",
            Self::ChargedCatalystArchitectureRejected => {
                "D045_CHARGED_CATALYST_ARCHITECTURE_REJECTED"
            }
            Self::CatalystStateMappingUnresolved => "D045_CATALYST_STATE_MAPPING_UNRESOLVED",
            Self::ChargedCatalystConservationFailure => {
                "D045_CHARGED_CATALYST_CONSERVATION_FAILURE"
            }
            Self::ChargedCatalystKineticsNotIdentifiable => {
                "D045_CHARGED_CATALYST_KINETICS_NOT_IDENTIFIABLE"
            }
            Self::CatalystRecyclingNotEstablished => "D045_CATALYST_RECYCLING_NOT_ESTABLISHED",
            Self::ChargedCatalystLawNotPortable => "D045_CHARGED_CATALYST_LAW_NOT_PORTABLE",
            Self::ActivationCapacityNotRecovered => "D045_ACTIVATION_CAPACITY_NOT_RECOVERED",
            Self::FoundationalActivationRegression => "D045_FOUNDATIONAL_ACTIVATION_REGRESSION",
            Self::MembraneBasinNotRecovered => "D045_MEMBRANE_BASIN_NOT_RECOVERED",
            Self::ContinuousReplacementNotRecovered => "D045_CONTINUOUS_REPLACEMENT_NOT_RECOVERED",
            Self::DamageRepairNotRecovered => "D045_DAMAGE_REPAIR_NOT_RECOVERED",
            Self::ResourceDependenceNotEstablished => "D045_RESOURCE_DEPENDENCE_NOT_ESTABLISHED",
            Self::StageEMembraneContractFailure => "D045_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D045_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D045_NUMERICAL_FAILURE",
            Self::Fail => "D045_FAIL",
        }
    }
}

/// One controlled diagnostic demand measurement (not an organismal steady state).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemandScalingRow {
    pub label: String,
    pub radius: f64,
    pub c: f64,
    pub n: f64,
    pub f: f64,
    pub l_a: f64,
    pub m_c: f64,
    pub interior_volume: f64,
    pub structural_mass: f64,
    pub membrane_area: f64,
    pub resource_influx: f64,
    pub j_reproduction: f64,
    pub j_structural: f64,
    pub j_precursor: f64,
    pub j_decay: f64,
    pub j_out: f64,
    pub j_in: f64,
}

impl DemandScalingRow {
    pub fn d_c(&self) -> f64 {
        if self.m_c <= 1e-18 {
            return f64::INFINITY;
        }
        self.l_a / self.m_c
    }

    pub fn d_v(&self) -> f64 {
        if self.interior_volume <= 1e-18 {
            return f64::INFINITY;
        }
        self.l_a / self.interior_volume
    }

    pub fn d_s(&self) -> f64 {
        if self.structural_mass <= 1e-18 {
            return f64::INFINITY;
        }
        self.l_a / self.structural_mass
    }

    pub fn d_area(&self) -> f64 {
        if self.membrane_area <= 1e-18 {
            return f64::INFINITY;
        }
        self.l_a / self.membrane_area
    }

    pub fn ledger_terms_nonnegative(&self) -> bool {
        self.j_reproduction >= 0.0
            && self.j_structural >= 0.0
            && self.j_precursor >= 0.0
            && self.j_decay >= 0.0
            && self.j_out >= 0.0
            && self.j_in >= 0.0
            && self.l_a.is_finite()
            && self.l_a >= 0.0
    }

    /// Authorized loss reconstructed from ledger parts.
    pub fn reconstructed_l_a(&self) -> f64 {
        self.j_reproduction + self.j_structural + self.j_precursor + self.j_decay + self.j_out
            - self.j_in
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemandScalingReport {
    pub rows: Vec<DemandScalingRow>,
    pub d_c_span: f64,
    pub d_c_span_ok: bool,
    pub radius_d_c_span: f64,
    pub radius_bias_ok: bool,
    pub superlinear: bool,
    pub catalyst_linear_max_rel_err: f64,
    pub catalyst_linear_ok: bool,
    pub volume_linear_max_rel_err: f64,
    pub ledger_complete: bool,
    pub l_a_span: f64,
    pub m_c_span: f64,
    pub pass: bool,
    pub conclusion_if_fail: String,
}

fn positive_span(vals: &[f64]) -> f64 {
    let finite: Vec<f64> = vals
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    if finite.is_empty() {
        return f64::INFINITY;
    }
    let lo = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = finite.iter().copied().fold(0.0_f64, f64::max);
    if lo <= 1e-18 {
        return f64::INFINITY;
    }
    hi / lo
}

fn through_origin_max_rel_err(xs: &[f64], ys: &[f64]) -> f64 {
    let mut xx = 0.0;
    let mut xy = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        if x.is_finite() && y.is_finite() && x > 0.0 && y > 0.0 {
            xx += x * x;
            xy += x * y;
        }
    }
    if xx <= 1e-18 {
        return f64::INFINITY;
    }
    let alpha = xy / xx;
    let mut max_err: f64 = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        if y > 1e-18 {
            let pred = alpha * x;
            max_err = max_err.max(((y - pred) / y).abs());
        }
    }
    max_err
}

/// Evaluate Gate 0 catalyst-demand scaling on controlled diagnostic rows.
///
/// Uses matched-N/F C-series for linearity and true-radius rows for radius bias.
pub fn evaluate_demand_scaling(rows: &[DemandScalingRow]) -> DemandScalingReport {
    let matched: Vec<&DemandScalingRow> = rows
        .iter()
        .filter(|r| {
            r.label == "low_c" || r.label == "med_c" || r.label == "high_c" || {
                let matched_nf = (r.n - 0.8).abs() < 1e-6 && (r.f - 0.8).abs() < 1e-6;
                matched_nf
                    && (r.label.starts_with('R') || r.label.starts_with("low") || r.label.starts_with("med")
                        || r.label.starts_with("high"))
            }
        })
        .collect();
    // Prefer explicit C-level labels when present.
    let c_series: Vec<&DemandScalingRow> = rows
        .iter()
        .filter(|r| matches!(r.label.as_str(), "low_c" | "med_c" | "high_c"))
        .collect();
    let c_basis: Vec<&DemandScalingRow> = if c_series.len() >= 2 {
        c_series
    } else {
        matched.clone()
    };

    let radius_series: Vec<&DemandScalingRow> = rows
        .iter()
        .filter(|r| {
            (r.label == "R16" || r.label == "R22" || r.label == "R32")
                && (r.n - 0.8).abs() < 1e-6
                && (r.f - 0.8).abs() < 1e-6
        })
        .collect();

    let d_c_vals: Vec<f64> = c_basis.iter().map(|r| r.d_c()).collect();
    let l_a_vals: Vec<f64> = c_basis.iter().map(|r| r.l_a).collect();
    let m_c_vals: Vec<f64> = c_basis.iter().map(|r| r.m_c).collect();
    let v_vals: Vec<f64> = c_basis.iter().map(|r| r.interior_volume).collect();

    let d_c_span = positive_span(&d_c_vals);
    let l_a_span = positive_span(&l_a_vals);
    let m_c_span = positive_span(&m_c_vals);
    let d_c_span_ok = d_c_span.is_finite() && d_c_span <= D045_DEMAND_DC_MAX_SPAN;

    let radius_d_c: Vec<f64> = radius_series.iter().map(|r| r.d_c()).collect();
    let radius_d_c_span = if radius_d_c.len() >= 2 {
        positive_span(&radius_d_c)
    } else {
        1.0
    };
    let radius_bias_ok = radius_d_c_span.is_finite() && radius_d_c_span <= D045_RADIUS_DC_MAX_SPAN;

    // Superlinear: L_A grows faster than M_C across the C series.
    let superlinear = m_c_span.is_finite()
        && m_c_span > 1.0
        && l_a_span.is_finite()
        && l_a_span > m_c_span * (1.0 + D045_SUPERLINEAR_TOL);

    let catalyst_linear_max_rel_err = through_origin_max_rel_err(&m_c_vals, &l_a_vals);
    let catalyst_linear_ok = catalyst_linear_max_rel_err <= D045_CATALYST_LINEAR_MAX_REL_ERR;
    let volume_linear_max_rel_err = through_origin_max_rel_err(&v_vals, &l_a_vals);

    let ledger_complete = rows.iter().all(|r| {
        r.ledger_terms_nonnegative()
            && (r.reconstructed_l_a() - r.l_a).abs()
                <= 1e-6 * r.l_a.abs().max(1.0)
    });

    let pass = d_c_span_ok
        && radius_bias_ok
        && !superlinear
        && catalyst_linear_ok
        && ledger_complete
        && !c_basis.is_empty();

    DemandScalingReport {
        rows: rows.to_vec(),
        d_c_span,
        d_c_span_ok,
        radius_d_c_span,
        radius_bias_ok,
        superlinear,
        catalyst_linear_max_rel_err,
        catalyst_linear_ok,
        volume_linear_max_rel_err,
        ledger_complete,
        l_a_span,
        m_c_span,
        pass,
        conclusion_if_fail: D045Conclusion::CatalystLinearityRejected.as_str().to_string(),
    }
}

/// Dimensionless activities for the proposed charged-catalyst cycle.
pub fn dimensionless_activities(n: f64, f: f64) -> (f64, f64) {
    (
        n.max(0.0) / D045_N_REFERENCE,
        f.max(0.0) / D045_F_REFERENCE,
    )
}

/// QSS production rate under fuel-charged catalyst cycling.
///
/// `r_QSS = C_total * (k_charge f * k_transfer n) / (k_charge f + k_transfer n)`
pub fn qss_production_rate(
    c_total: f64,
    n: f64,
    f: f64,
    k_charge: f64,
    k_transfer: f64,
) -> f64 {
    let (n_act, f_act) = dimensionless_activities(n, f);
    let a = k_charge * f_act;
    let b = k_transfer * n_act;
    let denom = a + b;
    if c_total <= 0.0 || denom <= 1e-18 {
        return 0.0;
    }
    c_total * a * b / denom
}

/// Charged fraction at QSS: `C_star / C_total = (k_charge f) / (k_charge f + k_transfer n)`.
pub fn qss_charged_fraction(n: f64, f: f64, k_charge: f64, k_transfer: f64) -> f64 {
    let (n_act, f_act) = dimensionless_activities(n, f);
    let a = k_charge * f_act;
    let b = k_transfer * n_act;
    let denom = a + b;
    if denom <= 1e-18 {
        return 0.0;
    }
    (a / denom).clamp(0.0, 1.0)
}

/// Required effective catalyst-linear coefficient from demand: `L_A / M_C` at matched activities.
pub fn required_effective_rate(l_a: f64, m_c: f64) -> f64 {
    if m_c <= 1e-18 || l_a < 0.0 {
        return f64::INFINITY;
    }
    l_a / m_c
}

/// Verify D-044 seal identity (commit tip equals annotated tag target).
pub fn d044_seal_consistent(commit: &str, tag_target: &str, tag_name: &str) -> bool {
    !commit.is_empty()
        && !tag_target.is_empty()
        && tag_name == D045_D044_TAG
        && commit == tag_target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qss_reduces_to_min_law() {
        // When k_charge f << k_transfer n, r ≈ C * k_charge f
        let r = qss_production_rate(2.0, 10.0, 0.1, 1.0, 100.0);
        let expected = 2.0 * 1.0 * 0.1;
        assert!((r - expected).abs() < 1e-3 * expected);
    }

    #[test]
    fn charged_fraction_bounded() {
        let frac = qss_charged_fraction(0.5, 0.5, 1.0, 1.0);
        assert!((0.0..=1.0).contains(&frac));
        assert!((frac - 0.5).abs() < 1e-12);
    }
}
