//! D-045 focused tests: D-044 seal, demand scaling, QSS law.

use chemistry_core::d045_analysis::{
    d044_seal_consistent, dimensionless_activities, evaluate_demand_scaling, qss_charged_fraction,
    qss_production_rate, required_effective_rate, DemandScalingRow, D045_CATALYST_LINEAR_MAX_REL_ERR,
    D045_D044_TAG, D045_DEMAND_DC_MAX_SPAN, D045_F_REFERENCE, D045_N_REFERENCE,
};

fn row(
    label: &str,
    radius: f64,
    c: f64,
    l_a: f64,
    m_c: f64,
    volume: f64,
) -> DemandScalingRow {
    DemandScalingRow {
        label: label.to_string(),
        radius,
        c,
        n: 0.8,
        f: 0.8,
        l_a,
        m_c,
        interior_volume: volume,
        structural_mass: volume,
        membrane_area: 2.0 * std::f64::consts::PI * radius,
        resource_influx: 10.0,
        j_reproduction: l_a * 0.2,
        j_structural: l_a * 0.1,
        j_precursor: l_a * 0.55,
        j_decay: l_a * 0.1,
        j_out: l_a * 0.15,
        j_in: l_a * 0.1,
    }
}

#[test]
fn d044_seal_identity() {
    assert!(d044_seal_consistent(
        "1473f0775c395e942fae7d98576d9a4640ad7ae9",
        "1473f0775c395e942fae7d98576d9a4640ad7ae9",
        D045_D044_TAG
    ));
    assert!(!d044_seal_consistent("aaa", "bbb", D045_D044_TAG));
    assert!(!d044_seal_consistent("aaa", "aaa", "wrong-tag"));
}

#[test]
fn qss_equation_and_bounds() {
    let (n, f) = dimensionless_activities(0.5, 0.5);
    assert!((n - 0.5 / D045_N_REFERENCE).abs() < 1e-12);
    assert!((f - 0.5 / D045_F_REFERENCE).abs() < 1e-12);
    let r = qss_production_rate(2.0, 0.5, 0.5, 1.0, 1.0);
    // C * (0.5*0.5)/(0.5+0.5) = 2 * 0.25 / 1 = 0.5
    assert!((r - 0.5).abs() < 1e-12);
    let frac = qss_charged_fraction(0.5, 0.5, 1.0, 1.0);
    assert!((frac - 0.5).abs() < 1e-12);
    assert_eq!(qss_production_rate(0.0, 1.0, 1.0, 1.0, 1.0), 0.0);
}

#[test]
fn catalyst_linear_demand_passes() {
    // L_A proportional to M_C → portable catalyst-normalized demand.
    let rows = vec![
        row("low_c", 22.0, 0.3, 50.0, 500.0, 1500.0),
        row("med_c", 22.0, 0.6, 100.0, 1000.0, 1500.0),
        row("high_c", 22.0, 1.0, 150.0, 1500.0, 1500.0),
        row("R16", 16.0, 0.8, 80.0, 800.0, 1000.0),
        row("R22", 22.0, 0.8, 120.0, 1200.0, 1500.0),
        row("R32", 32.0, 0.8, 200.0, 2000.0, 2500.0),
    ];
    let report = evaluate_demand_scaling(&rows);
    assert!(report.d_c_span_ok);
    assert!(report.d_c_span <= D045_DEMAND_DC_MAX_SPAN);
    assert!(report.catalyst_linear_ok);
    assert!(report.catalyst_linear_max_rel_err <= D045_CATALYST_LINEAR_MAX_REL_ERR);
    assert!(report.pass);
}

#[test]
fn catalyst_nonlinear_demand_rejects() {
    // Sealed-style: L_A nearly flat while M_C spans ~3×.
    let rows = vec![
        row("low_c", 22.0, 0.3, 144.0, 459.0, 1531.0),
        row("med_c", 22.0, 0.6, 168.0, 919.0, 1531.0),
        row("high_c", 22.0, 1.0, 186.0, 1531.0, 1531.0),
        row("R16", 16.0, 0.8, 100.0, 650.0, 800.0),
        row("R22", 22.0, 0.8, 168.0, 1225.0, 1531.0),
        row("R32", 32.0, 0.8, 280.0, 2590.0, 3200.0),
    ];
    let report = evaluate_demand_scaling(&rows);
    assert!(report.d_c_span <= D045_DEMAND_DC_MAX_SPAN || !report.d_c_span_ok);
    assert!(!report.catalyst_linear_ok);
    assert!(report.catalyst_linear_max_rel_err > D045_CATALYST_LINEAR_MAX_REL_ERR);
    assert!(!report.pass);
}

#[test]
fn required_effective_rate_is_d_c() {
    assert!((required_effective_rate(100.0, 500.0) - 0.2).abs() < 1e-12);
}
