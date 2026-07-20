//! D-044 focused tests: reconstruction, eligibility, scaling, candidate laws, holdout split.

use chemistry_core::d042_analysis::ALedgerTerms;
use chemistry_core::d043_analysis::{build_rate_estimate, evaluate_portable_rate};
use chemistry_core::d044_analysis::{
    activation_rate_a, activation_rate_b, activation_rate_c, build_holdout_states,
    build_training_states, classify_state_eligibility, classify_viable_domain,
    d043_reconstruction_within_tolerance, dimensionless_activities, evaluate_heldout_steady,
    evaluate_heldout_transient, evaluate_scaling_audit, fit_candidate_a, fit_candidate_b,
    fit_candidate_c, monotonicity_passes_a, monotonicity_passes_b, monotonicity_passes_c,
    predict_steady_demand_a, predict_steady_demand_b, predict_steady_demand_c,
    saturation_factor_b, saturation_factor_c, scaling_audit_row, select_candidate,
    zero_control_passes_a, zero_control_passes_b, zero_control_passes_c,
    ActivationLawId, ActivationTrainingRow, CandidateBFitReport, CandidateCFitReport,
    D043_RECONSTRUCTION_STATES, D044_F_REFERENCE, D044_HISTORICAL_K, D044_N_REFERENCE,
    D044_PORTABLE_MAX_SPAN, EligibilityControls, EligibilityWindow, StateEligibility,
    ViableDomainClass,
};

#[test]
fn candidate_a_mass_action_law() {
    assert!((activation_rate_a(0.02, 2.0, 3.0, 4.0) - 0.48).abs() < 1e-12);
    assert_eq!(activation_rate_a(0.02, 0.0, 1.0, 1.0), 0.0);
}

#[test]
fn candidate_b_joint_saturation() {
    let sat = saturation_factor_b(0.5, 0.5, 0.25, D044_N_REFERENCE, D044_F_REFERENCE);
    assert!(sat > 0.0 && sat < 1.0);
    let r = activation_rate_b(1.0, 0.8, 0.5, 0.5, 0.25, D044_N_REFERENCE, D044_F_REFERENCE);
    assert!(r > 0.0);
    assert!(zero_control_passes_b(1.0, 0.25));
    assert!(monotonicity_passes_b(1.0, 0.25));
}

#[test]
fn candidate_c_dual_saturation() {
    let sat = saturation_factor_c(0.5, 0.5, 0.3, 0.3, D044_N_REFERENCE, D044_F_REFERENCE);
    assert!(sat > 0.0 && sat < 1.0);
    let r = activation_rate_c(1.0, 0.8, 0.5, 0.5, 0.3, 0.3, D044_N_REFERENCE, D044_F_REFERENCE);
    assert!(r > 0.0);
    assert!(zero_control_passes_c(1.0, 0.3, 0.3));
    assert!(monotonicity_passes_c(1.0, 0.3, 0.3));
}

#[test]
fn d043_reconstruction_tolerance_sealed() {
    let mk = |label: &str, k: f64| {
        let mut terms = ALedgerTerms::default();
        terms.j_activation = k * 100.0;
        build_rate_estimate(label, 0.8, 0.8, 0.8, 100.0, &terms, 0.05)
    };
    let estimates: Vec<_> = [
        ("R16", 0.373),
        ("R22", 0.226),
        ("R32", 0.150),
        ("low_c", 0.491),
        ("med_c", 0.285),
        ("high_c", 0.189),
        ("high_nf", 0.145),
    ]
    .iter()
    .map(|(l, k)| {
        let mut e = mk(l, *k);
        e.k_required = *k;
        e.valid = true;
        e
    })
    .collect();
    assert!(d043_reconstruction_within_tolerance(3.38, &estimates).pass);
}

#[test]
fn portable_mass_action_span_fails_d043() {
    let mk = |label: &str, b: f64, l: f64| {
        let mut terms = ALedgerTerms::default();
        terms.j_activation = l;
        build_rate_estimate(label, 0.8, 0.8, 0.8, b, &terms, 0.05)
    };
    let estimates = vec![
        mk("R16", 450.0, 168.0),
        mk("R22", 784.0, 178.0),
        mk("R32", 1240.0, 186.0),
        mk("low_c", 294.0, 144.0),
        mk("med_c", 588.0, 168.0),
        mk("high_c", 980.0, 186.0),
        mk("high_nf", 1224.0, 178.0),
    ];
    let report = evaluate_portable_rate(&estimates);
    assert!(!report.pass);
    assert!(report.span > D044_PORTABLE_MAX_SPAN);
}

#[test]
fn state_eligibility_forced_diagnostic() {
    let windows = vec![EligibilityWindow {
        c_flow: 1.0,
        n_flow: 1.0,
        f_flow: 1.0,
        a_flow: 1.0,
        c_mean: 0.8,
        n_mean: 0.8,
        f_mean: 0.8,
        a_mean: 0.5,
        l_a: 0.1,
        timestep_ok: true,
        concentration_ok: true,
    }; 3];
    let ctrl = EligibilityControls {
        clamp_a: true,
        ..Default::default()
    };
    assert_eq!(
        classify_state_eligibility(&windows, &ctrl),
        StateEligibility::ForcedDiagnostic
    );
}

#[test]
fn viable_domain_low_nf_classification() {
    let audit = classify_viable_domain(
        "low_nf",
        1.0,
        1.0,
        0.3,
        0.3,
        0.01,
        0.01,
        0.05,
        0.05,
        -0.001,
        true,
    );
    assert_eq!(
        audit.classification,
        ViableDomainClass::IrreversibleStarvation
    );

    let viable = classify_viable_domain(
        "med_c",
        1.0,
        1.0,
        0.8,
        0.8,
        0.05,
        0.05,
        0.02,
        0.02,
        0.0,
        false,
    );
    assert_eq!(
        viable.classification,
        ViableDomainClass::ViableResourceLimited
    );
}

#[test]
fn training_holdout_disjoint() {
    let train = build_training_states();
    let hold = build_holdout_states();
    let train_labels: std::collections::HashSet<_> = train.iter().map(|s| s.label.as_str()).collect();
    for h in &hold {
        assert!(
            !train_labels.contains(h.label.as_str()),
            "overlap: {}",
            h.label
        );
    }
}

#[test]
fn scaling_audit_radius_independence() {
    let rows = vec![
        scaling_audit_row("R16", 16.0, 10.0, 20.0),
        scaling_audit_row("R22", 22.0, 10.0, 20.0),
        scaling_audit_row("R32", 32.0, 10.0, 20.0),
    ];
    assert!(evaluate_scaling_audit(&rows).pass);
}

#[test]
fn heldout_steady_error_gates() {
    let pass = evaluate_heldout_steady(&[0.18, 0.20, 0.22], &[0.20, 0.20, 0.20]);
    assert!(pass.pass);
    let fail = evaluate_heldout_steady(&[0.10, 0.50, 0.20], &[0.20, 0.20, 0.20]);
    assert!(!fail.pass);
}

#[test]
fn heldout_transient_sign() {
    let signs = vec![true, true, true, true, true, false];
    assert!(evaluate_heldout_transient(&signs).pass);
    let bad = vec![true, false, false, true, true, false];
    assert!(!evaluate_heldout_transient(&bad).pass);
}

#[test]
fn candidate_selection_prefers_simpler() {
    let fit_a = fit_candidate_a(&[]);
    let fit_b = CandidateBFitReport {
        law: ActivationLawId::CandidateB,
        v_b: 0.02,
        k_nf: 0.25,
        span: 2.5,
        loo_ok: true,
        loo_max_factor: 1.5,
        bootstrap_spread_rel: 0.2,
        pass: true,
        estimates: vec![],
        notes: vec![],
    };
    let fit_c = CandidateCFitReport {
        law: ActivationLawId::CandidateC,
        v_c: 0.02,
        k_n: 0.3,
        k_f: 0.3,
        span: 2.0,
        loo_ok: true,
        loo_max_factor: 1.5,
        bootstrap_spread_rel: 0.2,
        pass: true,
        estimates: vec![],
        notes: vec![],
    };
    let sel = select_candidate(fit_a.pass, &fit_b, &fit_c);
    assert_eq!(sel.selected, Some(ActivationLawId::CandidateB));
}

#[test]
fn saturation_fit_on_synthetic_training() {
    let rows = vec![
        ActivationTrainingRow {
            label: "s1".into(),
            c: 0.6,
            n: 0.8,
            f: 0.8,
            l_a: 0.10,
            valid: true,
        },
        ActivationTrainingRow {
            label: "s2".into(),
            c: 1.0,
            n: 0.9,
            f: 0.9,
            l_a: 0.12,
            valid: true,
        },
        ActivationTrainingRow {
            label: "s3".into(),
            c: 0.3,
            n: 0.8,
            f: 0.8,
            l_a: 0.08,
            valid: true,
        },
    ];
    let fit_b = fit_candidate_b(&rows);
    assert!(fit_b.v_b.is_finite());
    let fit_c = fit_candidate_c(&rows);
    assert!(fit_c.v_c.is_finite());
}

#[test]
fn predict_steady_demands_positive() {
    assert!(predict_steady_demand_a(D044_HISTORICAL_K, 0.8, 0.8, 0.8) > 0.0);
    assert!(predict_steady_demand_b(0.02, 0.25, 0.8, 0.8, 0.8) > 0.0);
    assert!(predict_steady_demand_c(0.02, 0.3, 0.3, 0.8, 0.8, 0.8) > 0.0);
}

#[test]
fn d043_state_count() {
    assert_eq!(D043_RECONSTRUCTION_STATES.len(), 8);
}

#[test]
fn dimensionless_activities_use_frozen_refs() {
    let (n, f) = dimensionless_activities(0.8, 0.6, D044_N_REFERENCE, D044_F_REFERENCE);
    assert!((n - 0.8).abs() < 1e-12);
    assert!((f - 0.6).abs() < 1e-12);
}
