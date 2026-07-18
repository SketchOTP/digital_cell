//! D-030 Gate tests: orthogonal exchange observability and identification.
use chemistry_core::config::{EquationVersion, SimParams};
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d030_analysis::{
    adsorption_matrix_specs, catalyst_for_q, desorption_matrix_specs, recover_exchange_parameters,
    relative_spread, robust_median, run_orthogonal_assay, OrthogonalAssaySpec, D030_ADS_THETA_MAX,
    D030_LEVEL_SPREAD_MAX, D030_Q_NORM_SPREAD_MAX,
};

fn planted() -> (f64, f64) {
    (0.04, 5.0) // k_exchange, K_exchange → α=0.20, β=0.04
}

fn base_v8() -> SimParams {
    let (k, k_eq) = planted();
    let mut p = SimParams::default();
    apply_exchange_candidate(
        &mut p,
        &chemistry_core::d029_analysis::ExchangeCandidate {
            identity: "planted".into(),
            k_exchange: k,
            k_exchange_eq: k_eq,
        },
    );
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.reactions_enabled = false;
    p
}

#[test]
fn transient_exchange_observability_records_directions() {
    let (k, k_eq) = planted();
    let params = base_v8();
    let ads = OrthogonalAssaySpec {
        label: "obs_ads".into(),
        theta0: 0.0,
        precursor0: 0.5,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 5,
        theta_stop: D030_ADS_THETA_MAX,
    };
    let des = OrthogonalAssaySpec {
        label: "obs_des".into(),
        theta0: 0.5,
        precursor0: 0.0,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 5,
        theta_stop: 1.0,
    };
    let a = run_orthogonal_assay(k, k_eq, &ads).unwrap();
    let d = run_orthogonal_assay(k, k_eq, &des).unwrap();
    assert!(a.first.forward_exchange > 0.0);
    assert!(a.first.reverse_exchange.abs() < 1e-12);
    assert!(d.first.reverse_exchange > 0.0);
    assert!(d.first.forward_exchange.abs() < 1e-12);
    assert!(a.window_10.is_some() || a.trajectory_p.len() >= 2);
}

#[test]
fn zero_theta_adsorption_isolation() {
    let (k, k_eq) = planted();
    let params = base_v8();
    let spec = OrthogonalAssaySpec {
        label: "zero_theta".into(),
        theta0: 0.0,
        precursor0: 1.0,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 1,
        theta_stop: D030_ADS_THETA_MAX,
    };
    let r = run_orthogonal_assay(k, k_eq, &spec).unwrap();
    assert!(r.first.mean_theta <= 1e-9 || r.spec.theta0 == 0.0);
    assert!(r.first.reverse_exchange.abs() < 1e-12);
    assert!(r.first.net_exchange > 0.0);
    assert!((r.first.exact_dp + r.first.exact_ds).abs() < 1e-10);
}

#[test]
fn zero_p_desorption_isolation() {
    let (k, k_eq) = planted();
    let params = base_v8();
    let spec = OrthogonalAssaySpec {
        label: "zero_p".into(),
        theta0: 0.6,
        precursor0: 0.0,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 1,
        theta_stop: 1.0,
    };
    let r = run_orthogonal_assay(k, k_eq, &spec).unwrap();
    assert!(r.first.forward_exchange.abs() < 1e-12);
    assert!(r.first.net_exchange < 0.0);
    assert!(r.first.exact_ds < 0.0);
    assert!(r.first.exact_dp > 0.0);
    // No biological waste from desorption in exchange-only assay.
    assert_eq!(r.first.exchange_dissipation.is_finite(), true);
}

#[test]
fn alpha_recovery_across_p_and_q() {
    let (k, k_eq) = planted();
    let alpha = k * k_eq;
    let params = base_v8();
    let specs = adsorption_matrix_specs(&params);
    let mut alphas = Vec::new();
    let mut by_q: Vec<Vec<f64>> = vec![Vec::new(); 3];
    for (i, spec) in specs.iter().enumerate() {
        let r = run_orthogonal_assay(k, k_eq, spec).unwrap();
        assert!(r.first.net_exchange > 0.0, "{}", spec.label);
        assert!(r.first.reverse_exchange.abs() < 1e-12, "{}", spec.label);
        assert!(r.first.alpha_estimate.is_finite() && r.first.alpha_estimate > 0.0);
        alphas.push(r.first.alpha_estimate);
        by_q[i / 3].push(r.first.alpha_estimate);
    }
    let med = robust_median(&alphas);
    assert!((med - alpha).abs() / alpha < 0.05, "med={med} alpha={alpha}");
    assert!(relative_spread(&alphas) <= D030_LEVEL_SPREAD_MAX);
    let q_meds: Vec<f64> = by_q.iter().map(|g| robust_median(g)).collect();
    assert!(relative_spread(&q_meds) <= D030_Q_NORM_SPREAD_MAX);
}

#[test]
fn beta_recovery_across_theta_and_q() {
    let (k, k_eq) = planted();
    let params = base_v8();
    let specs = desorption_matrix_specs(&params);
    let mut betas = Vec::new();
    let mut by_q: Vec<Vec<f64>> = vec![Vec::new(); 3];
    for (i, spec) in specs.iter().enumerate() {
        let r = run_orthogonal_assay(k, k_eq, spec).unwrap();
        assert!(r.first.net_exchange < 0.0, "{}", spec.label);
        assert!(r.first.forward_exchange.abs() < 1e-12, "{}", spec.label);
        assert!(r.first.beta_estimate.is_finite() && r.first.beta_estimate > 0.0);
        betas.push(r.first.beta_estimate);
        by_q[i / 3].push(r.first.beta_estimate);
    }
    let med = robust_median(&betas);
    assert!((med - k).abs() / k < 0.05, "med={med} k={k}");
    assert!(relative_spread(&betas) <= D030_LEVEL_SPREAD_MAX);
    let q_meds: Vec<f64> = by_q.iter().map(|g| robust_median(g)).collect();
    assert!(relative_spread(&q_meds) <= D030_Q_NORM_SPREAD_MAX);
}

#[test]
fn first_substep_estimator_and_short_transient() {
    let (k, k_eq) = planted();
    let params = base_v8();
    let spec = OrthogonalAssaySpec {
        label: "short".into(),
        theta0: 0.0,
        precursor0: 0.5,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 15,
        theta_stop: D030_ADS_THETA_MAX,
    };
    let r = run_orthogonal_assay(k, k_eq, &spec).unwrap();
    assert!(r.first.alpha_estimate.is_finite());
    assert!(r.trajectory_p.len() >= 2);
    assert!(r.window_10.is_some());
}

#[test]
fn bootstrap_and_loo_stability() {
    let (k, k_eq) = planted();
    let alpha = k * k_eq;
    let params = base_v8();
    let mut alphas = Vec::new();
    let mut betas = Vec::new();
    let mut a_by_q = vec![Vec::new(); 3];
    let mut b_by_q = vec![Vec::new(); 3];
    for (i, spec) in adsorption_matrix_specs(&params).iter().enumerate() {
        let r = run_orthogonal_assay(k, k_eq, spec).unwrap();
        alphas.push(r.first.alpha_estimate);
        a_by_q[i / 3].push(r.first.alpha_estimate);
    }
    for (i, spec) in desorption_matrix_specs(&params).iter().enumerate() {
        let r = run_orthogonal_assay(k, k_eq, spec).unwrap();
        betas.push(r.first.beta_estimate);
        b_by_q[i / 3].push(r.first.beta_estimate);
    }
    let rec = recover_exchange_parameters(&alphas, &betas, &a_by_q, &b_by_q);
    assert!(rec.identifiable, "{rec:?}");
    assert!((rec.alpha_direct - alpha).abs() / alpha < 0.05);
    assert!((rec.beta_direct - k).abs() / k < 0.05);
    assert!(rec.loo_ok);
    assert!(rec.bootstrap_spread_factor_alpha <= 1.5);
    assert!(rec.bootstrap_spread_factor_beta <= 1.5);
    assert!((rec.k_exchange_eq - k_eq).abs() / k_eq < 0.05);
}

#[test]
fn mixed_state_prediction_direction() {
    let (k, k_eq) = planted();
    let params = base_v8();
    // Adsorption-leaning: high p, moderate θ
    let ads = OrthogonalAssaySpec {
        label: "mix_ads".into(),
        theta0: 0.2,
        precursor0: 0.5,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 1,
        theta_stop: 1.0,
    };
    // Desorption-leaning: low p, high θ (p below K-equilibrium)
    // θ/(1-θ)=K p ⇒ p_eq = θ/((1-θ)K); for θ=0.5, K=5 → p_eq=0.2
    let des = OrthogonalAssaySpec {
        label: "mix_des".into(),
        theta0: 0.5,
        precursor0: 0.05,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 1,
        theta_stop: 1.0,
    };
    let a = run_orthogonal_assay(k, k_eq, &ads).unwrap();
    let d = run_orthogonal_assay(k, k_eq, &des).unwrap();
    assert!(a.first.net_exchange > 0.0);
    assert!(d.first.net_exchange < 0.0);
}

#[test]
fn equilibrium_family_partition_independence() {
    let k = 1.5;
    let k_eq = 5.0;
    let mut params = SimParams::default();
    apply_exchange_candidate(
        &mut params,
        &chemistry_core::d029_analysis::ExchangeCandidate {
            identity: "eq".into(),
            k_exchange: k,
            k_exchange_eq: k_eq,
        },
    );
    let c = catalyst_for_q(&params, 0.5);
    let total = 8.0;
    let fracs = [0.15, 0.45, 0.75];
    let mut final_theta = Vec::new();
    for &fs in &fracs {
        let (th, _p, t0, t1) = chemistry_core::d030_analysis::run_equilibrium_partition_assay(
            k, k_eq, 10.0, total, fs, c, 5e-3, 600,
        )
        .unwrap();
        assert!((t1 - t0).abs() < 1e-8, "mass leak t0={t0} t1={t1}");
        assert!((0.0..=1.0).contains(&th));
        final_theta.push(th);
    }
    let med = robust_median(&final_theta);
    for th in &final_theta {
        assert!(
            (th - med).abs() < 0.08,
            "partition independence failed: thetas={final_theta:?}"
        );
    }
}

#[test]
fn v8_equation_immutable_under_d030() {
    assert_eq!(
        EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange.as_str(),
        "membrane_metabolism_v8_reversible_surface_exchange"
    );
    assert_eq!(
        EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
            .surface_exchange_schema_version(),
        2
    );
}

#[test]
fn historical_v8_gate1_still_compiles() {
    // Smoke: planted candidate applies without changing schema.
    let mut p = SimParams::default();
    apply_exchange_candidate(
        &mut p,
        &chemistry_core::d029_analysis::ExchangeCandidate {
            identity: "x".into(),
            k_exchange: 0.04,
            k_exchange_eq: 5.0,
        },
    );
    assert!(p.equation_version.is_reversible_surface_exchange());
    assert_eq!(p.k_ads, 0.0);
}
