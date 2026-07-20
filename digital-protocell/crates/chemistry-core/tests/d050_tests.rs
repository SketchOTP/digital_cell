//! D-050 focused tests: schema1 preservation, schema2 saturation, parity, identification.

use chemistry_core::activated_metabolism::activated_metabolism_rates;
use chemistry_core::config::EquationVersion;
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d046_analysis::{through_origin_alpha, DemandStateRow};
use chemistry_core::d047_analysis::{D047_HISTORICAL_K, D047_K_C_MEMBRANE};
use chemistry_core::d048_analysis::d048_frozen_organism_params;
use chemistry_core::d050_analysis::{
    activation_rate_schema1, activation_stoichiometry_ok, build_v_a_candidates,
    check_schema2_parity, d050_holdout_labels, d050_training_labels, fit_schema2_v_a,
    identify_schema2_parameters, is_fixed_biochemistry_row, production_activation_rate,
    q_c_saturation, schema2_activation_rate, schema2_bounded_high_c, schema2_monotonic_c_n_f,
    schema2_zero_resource_controls, select_smallest_passing_v_a, ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
    ACTIVATION_SCHEMA_HISTORICAL, D050_F_REF, D050_HISTORICAL_K, D050_N_REF,
};
use chemistry_core::Simulation;

fn row(label: &str, train: bool, c: f64, n: f64, f: f64, l_a: f64, k_p: f64, k_s: f64) -> DemandStateRow {
    DemandStateRow {
        label: label.into(),
        family: "test".into(),
        train,
        radius: 22.0,
        c,
        n,
        f,
        a: 0.5,
        p: 0.05,
        s_occupancy: 0.6,
        m_c: 1200.0,
        interior_volume: 1500.0,
        structural_mass: 1500.0,
        membrane_area: 140.0,
        l_a,
        j_reproduction: l_a * 0.11,
        j_structural: l_a * 0.10,
        j_precursor: l_a * 0.76,
        j_decay: l_a * 0.02,
        j_out: l_a * 0.01,
        j_in: 0.0,
        k_precursor_scale: k_p,
        k_structure_scale: k_s,
    }
}

#[test]
fn schema1_historical_rate_preserved() {
    assert!((D050_HISTORICAL_K - D047_HISTORICAL_K).abs() < 1e-15);
    assert!((D050_HISTORICAL_K - 0.020).abs() < 1e-15);
    let r = activation_rate_schema1(0.020, 0.4, 0.5, 0.5);
    assert!((r - 0.020 * 0.4 * 0.5 * 0.5).abs() < 1e-15);
}

#[test]
fn schema1_default_non_v13() {
    let frozen = d048_frozen_organism_params(&v8_schema3_params());
    assert_eq!(frozen.activation_schema, ACTIVATION_SCHEMA_HISTORICAL);
    assert!(!frozen.equation_version.is_catalyst_saturating_activation());
    assert!((frozen.k_d008_activation - 0.020).abs() < 1e-15);
}

#[test]
fn q_c_saturation_midpoint() {
    let k = 0.10;
    assert!((q_c_saturation(0.0, k) - 0.0).abs() < 1e-15);
    assert!((q_c_saturation(k, k) - 0.5).abs() < 1e-12);
    assert!(q_c_saturation(100.0, k) > 0.99);
}

#[test]
fn schema2_zero_c_n_f() {
    let v_a = 1.0;
    let k_c = 0.10;
    assert!(schema2_zero_resource_controls(v_a, k_c));
    assert!(schema2_activation_rate(v_a, 1.0, 0.0, 1.0, 1.0, k_c, 1.0, 1.0).abs() < 1e-15);
    assert!(schema2_activation_rate(v_a, 1.0, 1.0, 0.0, 1.0, k_c, 1.0, 1.0).abs() < 1e-15);
    assert!(schema2_activation_rate(v_a, 1.0, 1.0, 1.0, 0.0, k_c, 1.0, 1.0).abs() < 1e-15);
}

#[test]
fn schema2_bounded_at_high_c() {
    assert!(schema2_bounded_high_c(1.0, 0.10));
}

#[test]
fn schema2_monotonic_in_c_n_f() {
    assert!(schema2_monotonic_c_n_f(1.0, 0.10));
}

#[test]
fn schema1_vs_schema2_dispatcher_isolation() {
    let phi = 0.8;
    let c = 0.4;
    let n = 0.5;
    let f = 0.5;
    let k = 0.020;
    let v_a = 0.5;
    let k_c = 0.10;
    let s1 = production_activation_rate(1, k, phi, c, n, f, k_c, 1.0, 1.0);
    let s2 = production_activation_rate(2, v_a, phi, c, n, f, k_c, 1.0, 1.0);
    assert!((s1 - k * c * n * f).abs() < 1e-15);
    assert!(s2 > s1);
    assert!((s2 - schema2_activation_rate(v_a, phi, c, n, f, k_c, 1.0, 1.0)).abs() < 1e-15);
}

#[test]
fn v8_schema1_snapshot_cannot_resume_v13() {
    let params = d048_frozen_organism_params(&v8_schema3_params());
    let sim = Simulation::new(params.clone());
    let snap = sim.snapshot();
    let mut target = d048_frozen_organism_params(&v8_schema3_params());
    target.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    target.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    let err = snap.can_resume_into(&target).unwrap_err();
    assert!(
        err.contains("equation_version"),
        "expected equation_version mismatch, got {err}"
    );
}

#[test]
fn observer_runtime_schema2_parity() {
    let check = check_schema2_parity(0.42, 0.75, 0.4, 0.6, 0.5, 0.10);
    assert!(check.pass);
    assert!((check.observer_rate - check.runtime_rate).abs() < 1e-12);
}

#[test]
fn activated_metabolism_schema2_parity() {
    let mut params = d048_frozen_organism_params(&v8_schema3_params());
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = 0.35;
    params.k_c_activation = D047_K_C_MEMBRANE;
    params.n_ref_activation = D050_N_REF;
    params.f_ref_activation = D050_F_REF;
    let rates = activated_metabolism_rates(0.8, 0.4, 0.5, 0.5, 0.2, &params);
    let expected = production_activation_rate(
        ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
        params.k_d008_activation,
        0.8,
        0.4,
        0.5,
        0.5,
        params.k_c_activation,
        D050_N_REF,
        D050_F_REF,
    );
    assert!((rates.activation - expected).abs() < 1e-15);
}

#[test]
fn activation_stoichiometry_n_f_a_w() {
    assert!(activation_stoichiometry_ok(1.0));
    assert!(activation_stoichiometry_ok(0.37));
}

#[test]
fn fixed_biochemistry_filter() {
    let ok = row("R22", true, 0.8, 0.8, 0.8, 180.0, 1.0, 1.0);
    let bad = row("prec_hi", false, 0.8, 0.8, 0.8, 260.0, 2.0, 1.0);
    assert!(is_fixed_biochemistry_row(&ok));
    assert!(!is_fixed_biochemistry_row(&bad));
}

#[test]
fn candidate_count_limit_five() {
    let c = build_v_a_candidates(1.0);
    assert!(c.len() <= 5);
    assert_eq!(c.len(), 3);
}

#[test]
fn smallest_passing_v_a_selection() {
    let c = build_v_a_candidates(2.0);
    let sel = select_smallest_passing_v_a(&c, |v| v >= 2.0);
    assert_eq!(sel, Some(2.0));
    let none = select_smallest_passing_v_a(&c, |_| false);
    assert_eq!(none, None);
}

#[test]
fn through_origin_alpha_used_for_fit() {
    let xs = vec![100.0, 200.0, 300.0];
    let ys = vec![10.0, 20.0, 30.0];
    assert!((through_origin_alpha(&xs, &ys) - 0.1).abs() < 1e-15);
    let train = vec![
        row("R16", true, 0.8, 0.8, 0.8, 96.0, 1.0, 1.0),
        row("R22", true, 0.8, 0.8, 0.8, 180.0, 1.0, 1.0),
        row("low_c", true, 0.3, 0.8, 0.8, 144.0, 1.0, 1.0),
        row("med_c", true, 0.8, 0.8, 0.8, 168.0, 1.0, 1.0),
    ];
    let hold = vec![
        row("R32", false, 0.8, 0.8, 0.8, 384.0, 1.0, 1.0),
        row("high_c", false, 1.2, 0.8, 0.8, 186.0, 1.0, 1.0),
    ];
    let report = fit_schema2_v_a(&train, &hold, D047_K_C_MEMBRANE);
    assert!(report.lambda > 0.0);
    assert!(report.lambda.is_finite());
}

#[test]
fn schema2_identification_on_synthetic_family() {
    let train = vec![
        row("R16", true, 0.8, 0.8, 0.8, 96.0, 1.0, 1.0),
        row("R22", true, 0.8, 0.8, 0.8, 180.0, 1.0, 1.0),
        row("R32", true, 0.8, 0.8, 0.8, 384.0, 1.0, 1.0),
        row("low_c", true, 0.3, 0.8, 0.8, 144.0, 1.0, 1.0),
        row("med_c", true, 0.8, 0.8, 0.8, 168.0, 1.0, 1.0),
        row("high_c", true, 1.2, 0.8, 0.8, 186.0, 1.0, 1.0),
        row("analytic_early", true, 0.8, 0.8, 0.8, 170.0, 1.0, 1.0),
        row("restored_early", true, 0.8, 0.8, 0.8, 175.0, 1.0, 1.0),
    ];
    let hold = vec![
        row("low_n", false, 0.8, 0.05, 0.8, 120.0, 1.0, 1.0),
        row("low_f", false, 0.8, 0.8, 0.05, 120.0, 1.0, 1.0),
        row("high_nf", false, 0.8, 1.2, 1.2, 200.0, 1.0, 1.0),
        row("analytic_late", false, 0.8, 0.8, 0.8, 160.0, 1.0, 1.0),
        row("restored_late", false, 0.8, 0.8, 0.8, 165.0, 1.0, 1.0),
        row("s_low", false, 0.8, 0.8, 0.8, 170.0, 1.0, 1.0),
        row("s_damaged25", false, 0.8, 0.8, 0.8, 182.0, 1.0, 1.0),
    ];
    let id = identify_schema2_parameters(&train, &hold, 0.05, 2.0);
    assert!(id.v_a > 0.0);
    assert!((id.k_c - D047_K_C_MEMBRANE).abs() < 1e-15);
}

#[test]
fn preregistered_label_sets_nonempty() {
    assert!(!d050_training_labels().is_empty());
    assert!(!d050_holdout_labels().is_empty());
}

#[test]
fn schema2_high_c_saturates_below_v_a() {
    let v_a = 1.0;
    let k_c = 0.10;
    let r = schema2_activation_rate(v_a, 1.0, 1000.0, 1.0, 1.0, k_c, 1.0, 1.0);
    assert!(r <= v_a * 1.001);
    assert!(r > 0.9 * v_a);
}

#[test]
fn production_schema1_ignores_phi() {
    let r_in = production_activation_rate(1, 0.020, 0.0, 0.4, 0.5, 0.5, 0.10, 1.0, 1.0);
    let r_out = production_activation_rate(1, 0.020, 1.0, 0.4, 0.5, 0.5, 0.10, 1.0, 1.0);
    assert!((r_in - r_out).abs() < 1e-15);
}


#[test]
fn v13_runtime_activation_exceeds_historical() {
    use chemistry_core::config::EquationVersion;
    use chemistry_core::activated_metabolism::activated_metabolism_rates;
    use chemistry_core::d039_analysis::v8_schema3_params;
    use chemistry_core::d049_analysis::d049_frozen_params;
    use chemistry_core::d050_analysis::{ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME, D050_HISTORICAL_K};

    let base = d049_frozen_params(&v8_schema3_params());
    let mut hist = base.clone();
    hist.activation_schema = 1;
    hist.k_d008_activation = D050_HISTORICAL_K;
    let rh = activated_metabolism_rates(1.0, 0.8, 0.8, 0.8, 0.5, &hist).activation;

    let mut v13 = base;
    v13.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    v13.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    v13.k_d008_activation = 0.12544510052968755;
    v13.k_c_activation = 0.10;
    v13.n_ref_activation = 1.0;
    v13.f_ref_activation = 1.0;
    let r2 = activated_metabolism_rates(1.0, 0.8, 0.8, 0.8, 0.5, &v13).activation;
    assert!(r2 > 5.0 * rh, "schema2 should dominate historical: r2={r2} rh={rh}");
}


#[test]
fn v13_higher_v_a_produces_more_activation_extent() {
    use chemistry_core::config::EquationVersion;
    use chemistry_core::d039_analysis::v8_schema3_params;
    use chemistry_core::d048_analysis::d048_frozen_organism_params;
    use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    use chemistry_core::Simulation;

    fn run(v_a: f64) -> f64 {
        let mut p = d048_frozen_organism_params(&v8_schema3_params());
        p.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
        p.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
        p.k_d008_activation = v_a;
        p.k_c_activation = 0.10;
        p.n_ref_activation = 1.0;
        p.f_ref_activation = 1.0;
        let mut sim = Simulation::new(p);
        sim.enforce_structure_constraint = false;
        sim.dt_cap = 0.005;
        let w = sim.grid.width as i32;
        let h = sim.grid.height as i32;
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        for idx in 0..sim.grid.width * sim.grid.height {
            if !sim.grid.in_dish(idx) {
                continue;
            }
            let x = (idx % sim.grid.width) as f64;
            let y = (idx / sim.grid.width) as f64;
            let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            if r <= 22.0 {
                sim.fields.structure[idx] = 1.0;
                sim.fields.catalyst[idx] = 0.8;
                sim.fields.nutrient[idx] = 0.8;
                sim.fields.fuel[idx] = 0.8;
                sim.fields.activated[idx] = 0.5;
                sim.fields.precursor[idx] = 0.05;
            }
        }
        let mut act = 0.0;
        for _ in 0..200 {
            assert!(sim.step());
            act += sim.metabolism_accounting.last_step.activation;
        }
        act
    }
    let low = run(0.05);
    let high = run(0.20);
    assert!(
        high > low * 1.5,
        "high={high} low={low} schema2 V_A should scale activation extent"
    );
}
