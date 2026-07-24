//! Independent metric definitions for D-087 (raw-ledger based).
//!
//! D-086 reported `tracer_m/b/c` are **remaining label fractions**
//! `f_label = M_labeled(T) / M_total(T)` after a pulse, **not** replacement
//! equivalents `R_X`. Retention in D-086 was `final/initial` concentration.

use serde::{Deserialize, Serialize};

pub const E_INV: f64 = 0.3678794411714423; // e^{-1}
pub const RETENTION_MIN: f64 = 0.80;
pub const R_X_MIN: f64 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionReport {
    pub species: String,
    pub initial: f64,
    pub minimum: f64,
    pub final_value: f64,
    pub time_averaged: f64,
    pub denominator: String,
    pub formula: String,
    pub retention_final_over_initial: f64,
    pub increase_accounted_by_production: bool,
    pub qualifies_above_one: bool,
}

/// Independent retention: tracks concentration (or mass) series.
pub fn retention_report(species: &str, series: &[f64], produced: f64) -> RetentionReport {
    let initial = series.first().copied().unwrap_or(0.0);
    let final_value = series.last().copied().unwrap_or(0.0);
    let minimum = series.iter().copied().fold(f64::INFINITY, f64::min);
    let time_averaged = if series.is_empty() {
        0.0
    } else {
        series.iter().sum::<f64>() / series.len() as f64
    };
    let retention = if initial <= 1e-15 {
        1.0
    } else {
        final_value / initial
    };
    let increase = final_value > initial + 1e-12;
    let accounted = !increase || produced > 1e-12;
    RetentionReport {
        species: species.into(),
        initial,
        minimum: if minimum.is_finite() { minimum } else { 0.0 },
        final_value,
        time_averaged,
        denominator: "initial value at pulse/window start".into(),
        formula: "retention = final / initial (concentration or mass units as recorded)".into(),
        retention_final_over_initial: retention,
        increase_accounted_by_production: accounted,
        qualifies_above_one: !increase || accounted,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementReport {
    pub component: String,
    pub mean_mass: f64,
    pub gross_replacement_integral: f64,
    pub r_x: f64,
    /// Remaining original pulse label: labeled(T)/labeled(0).
    pub f_label: f64,
    /// D-086 reported metric: labeled(T)/pool_total(T).
    pub f_pool: f64,
    pub label_initial: f64,
    pub label_final: f64,
    pub r_x_ok: bool,
    pub f_label_ok: bool,
    pub interpretation: String,
}

/// R_X = ∫ J_gross_replacement dt / mean(M_X);
/// f_label = labeled(T)/labeled(0); f_pool = labeled(T)/pool(T) (D-086 style).
pub fn replacement_report(
    component: &str,
    mean_mass: f64,
    gross_integral: f64,
    label0: f64,
    label_t: f64,
    pool_t: f64,
) -> ReplacementReport {
    let r_x = if mean_mass <= 1e-15 {
        0.0
    } else {
        gross_integral / mean_mass
    };
    let f_label = if label0 <= 1e-15 {
        0.0
    } else {
        (label_t / label0).clamp(0.0, 1.0)
    };
    let f_pool = if pool_t <= 1e-15 {
        0.0
    } else {
        (label_t / pool_t).clamp(0.0, 1.0)
    };
    ReplacementReport {
        component: component.into(),
        mean_mass,
        gross_replacement_integral: gross_integral,
        r_x,
        f_label,
        f_pool,
        label_initial: label0,
        label_final: label_t,
        r_x_ok: r_x + 1e-12 >= R_X_MIN,
        f_label_ok: f_label <= E_INV + 1e-9,
        interpretation: format!(
            "R_X=gross_replacement/mean_mass; f_label=labeled(T)/labeled(0); f_pool=labeled(T)/pool(T)=D-086 tracer_* metric"
        ),
    }
}

/// Map a D-086 reported tracer value to its semantic class.
pub fn interpret_d086_tracer(name: &str, value: f64) -> String {
    format!(
        "{name}={value:.3}: remaining pulse-chase label fraction f_label ≈ M_labeled(T)/M_total(T) after unit pulse; NOT R_X replacement equivalents. Pass threshold in D-086 was f_label < 0.55 (m) / 0.70 (b,c), not the D-087 dual requirement R_X≥1 ∧ f_label≤e⁻¹."
    )
}
