//! D-043 focused tests: activation rate law, parity, classification, portable reconstruction, candidates.

use chemistry_core::config::SimParams;
use chemistry_core::d042_analysis::{ALedgerIntegral, ALedgerTerms, D042_LEDGER_REL_TOL};
use chemistry_core::d043_analysis::{
    activation_basis, activation_rate, build_activation_candidates, build_rate_estimate,
    check_activation_parity, check_activation_potential_transfer, check_activation_stoichiometry,
    classify_capacity_deficit, d042_capacity_deficit_reproduced, evaluate_candidate_row,
    evaluate_portable_rate, parity_suite_passes, required_k_activation, screen_candidates,
    select_smallest_passing, zero_control_passes, CapacityClassification, D043_BASIS_FLOOR,
    D043_HISTORICAL_K_ACTIVATION, D043_LEDGER_REL_TOL, D043_MAX_CANDIDATES,
    D043_PORTABLE_MAX_SPAN,
};

#[test]
fn activation_rate_law_basis() {
    assert!((activation_basis(2.0, 3.0, 4.0) - 24.0).abs() < 1e-15);
    assert!((activation_rate(0.02, 2.0, 3.0, 4.0) - 0.48).abs() < 1e-15);
    assert_eq!(activation_basis(0.0, 1.0, 1.0), 0.0);
    assert_eq!(D043_HISTORICAL_K_ACTIVATION, 0.02);
}

#[test]
fn observer_runtime_parity() {
    let params = SimParams::default();
    let p = check_activation_parity(0.02, 1.0, 0.8, 0.6, 0.2, &params);
    assert!(p.basis_match);
    assert!(p.rate_match);
    assert!((p.rate_observer - 0.02 * 0.48).abs() < 1e-12);
}

#[test]
fn stoichiometry_n_f_a_w() {
    let s = check_activation_stoichiometry(0.07);
    assert!(s.pass);
    assert!((s.d_n + 0.07).abs() < 1e-15);
    assert!((s.d_f + 0.07).abs() < 1e-15);
    assert!((s.d_a - 0.07).abs() < 1e-15);
    assert!((s.d_w - 0.07).abs() < 1e-15);
}

#[test]
fn activation_potential_transfer_zero() {
    assert!(check_activation_potential_transfer(0.05).pass);
}

#[test]
fn zero_c_n_f_controls() {
    let params = SimParams::default();
    assert!(zero_control_passes(0.02, &params));
    assert!(parity_suite_passes(0.02, &params));
}

#[test]
fn a_ledger_closure_identity() {
    let w = ALedgerTerms {
        j_activation: 1.0,
        j_in: 0.1,
        a_initial: 2.0,
        j_reproduction: 0.2,
        j_structural: 0.1,
        j_precursor: 0.1,
        j_decay: 0.05,
        j_out: 0.1,
        j_reservoir: 0.0,
        numerical_correction: 0.0,
        a_final: 2.0 + 0.55,
        dt: 1.0,
        ..Default::default()
    };
    assert!(w.closes(D043_LEDGER_REL_TOL));
}

#[test]
fn d042_deficit_reproduction_check() {
    let mut integ = ALedgerIntegral::default();
    let w = ALedgerTerms {
        j_activation: 0.5,
        j_reproduction: 0.4,
        j_structural: 0.3,
        j_precursor: 0.2,
        j_decay: 0.1,
        j_out: 0.05,
        j_in: 0.0,
        dt: 1.0,
        a_initial: 1.0,
        a_final: 1.0 - 0.55, // matches R_A = 0.5 - 0.9 - 0.1 - 0.05
        ..Default::default()
    };
    integ.accumulate(&w);
    integ.accumulate(&w);
    integ.accumulate(&w);
    assert!(d042_capacity_deficit_reproduced(&integ, 3));
    assert!(integ.closes(D042_LEDGER_REL_TOL));
}

#[test]
fn capacity_classification_rate_capacity() {
    // Rate capacity only when healthy C/N/F still leave a persistent deficit.
    let (c, _) = classify_capacity_deficit(
        -1.0, -0.8, -0.7, -0.6, -0.5, -0.4, -0.9, -0.8, 1e-9,
    );
    assert_eq!(c, CapacityClassification::RateCapacity);
}

#[test]
fn capacity_classification_substrate_delivery() {
    let (c, _) = classify_capacity_deficit(
        -1.0, 0.1, -0.5, -0.6, 0.2, -0.1, -0.9, -0.8, 1e-9,
    );
    assert_eq!(c, CapacityClassification::SubstrateDelivery);
}

#[test]
fn capacity_classification_catalyst_basis() {
    // Healthy C (alone or with N/F) closes; N/F alone do not.
    let (c, _) = classify_capacity_deficit(
        -1.0, -0.5, -0.4, 0.1, -0.3, 0.05, -0.9, -0.8, 1e-9,
    );
    assert_eq!(c, CapacityClassification::CatalystBasis);
}

#[test]
fn capacity_classification_decay_defect() {
    let (c, dem) = classify_capacity_deficit(
        -1.0, -0.8, -0.7, -0.6, -0.5, -0.4, 0.1, -0.8, 1e-9,
    );
    assert_eq!(c, CapacityClassification::DecayDefect);
    assert_eq!(dem.as_deref(), Some("a_decay"));
}

#[test]
fn capacity_classification_demand_defect() {
    let (c, dem) = classify_capacity_deficit(
        -1.0, -0.8, -0.7, -0.6, -0.5, -0.4, -0.9, 0.2, 1e-9,
    );
    assert_eq!(c, CapacityClassification::DemandDefect);
    assert_eq!(dem.as_deref(), Some("productive_demands"));
}

#[test]
fn portable_rate_reconstruction_pass() {
    let labels = [
        "R16", "R22", "R32", "low_c", "med_c", "high_c", "low_nf", "high_nf",
    ];
    let mut estimates = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        let c = 0.5 + 0.1 * (i as f64);
        let n = 0.8;
        let f = 0.7;
        // Domain-total basis ~ O(10²); L_A scales mildly with state.
        let total_basis = 100.0 + 10.0 * (i as f64);
        let terms = ALedgerTerms {
            j_reproduction: 1.0 + 0.05 * (i as f64),
            j_structural: 0.5,
            j_precursor: 0.3,
            j_decay: 0.1,
            j_out: 0.05,
            j_in: 0.0,
            ..Default::default()
        };
        estimates.push(build_rate_estimate(
            label,
            c,
            n,
            f,
            total_basis,
            &terms,
            D043_BASIS_FLOOR,
        ));
    }
    let report = evaluate_portable_rate(&estimates);
    assert!(report.valid_count >= 6);
    assert!(report.span <= D043_PORTABLE_MAX_SPAN + 1e-9);
    assert!(report.pass);
    // k_required must stay near historical scale, not O(interior volume).
    assert!(report.k_median > 0.01 && report.k_median < 1.0);
}

#[test]
fn required_k_from_loss_and_basis() {
    let terms = ALedgerTerms {
        j_reproduction: 0.4,
        j_structural: 0.0,
        j_precursor: 0.0,
        j_decay: 0.0,
        j_out: 0.0,
        j_in: 0.0,
        ..Default::default()
    };
    let k = required_k_activation(terms.j_demands(), 0.5);
    assert!((k - 0.8).abs() < 1e-12);
}

#[test]
fn candidate_count_limit() {
    let ks = build_activation_candidates(0.04);
    assert!(ks.len() <= D043_MAX_CANDIDATES);
    assert!(ks.contains(&D043_HISTORICAL_K_ACTIVATION));
}

#[test]
fn smallest_passing_candidate_selection() {
    let rows = vec![
        evaluate_candidate_row(
            0.03, 0.04, 0.01, 0.5, 0.025, 0.55, 0.5, 0.5, 0.5, true, false, false, false,
        ),
        evaluate_candidate_row(
            0.025, 0.04, 0.02, 0.5, 0.022, 0.52, 0.5, 0.5, 0.5, true, false, false, false,
        ),
        evaluate_candidate_row(
            0.05, 0.04, 0.05, 0.5, 0.03, 0.6, 0.5, 0.5, 0.5, true, false, false, false,
        ),
    ];
    assert_eq!(select_smallest_passing(&rows), Some(0.025));
    let report = screen_candidates(0.04, rows);
    assert!(report.pass);
    assert!((report.selected_k.unwrap() - 0.025).abs() < 1e-12);
}

#[test]
fn exhaustion_candidate_rejected() {
    let row = evaluate_candidate_row(
        0.04, 0.04, 0.01, 0.5, 0.025, 0.55, 0.5, 0.5, 0.5, true, true, false, false,
    );
    assert!(!row.pass);
    assert_eq!(row.reject_reason.as_deref(), Some("resource_exhaustion"));
}

#[test]
fn accumulation_candidate_rejected() {
    let row = evaluate_candidate_row(
        0.04, 0.04, 0.01, 0.5, 0.025, 0.55, 0.5, 0.5, 0.5, true, false, true, false,
    );
    assert!(!row.pass);
    assert_eq!(row.reject_reason.as_deref(), Some("a_accumulation"));
}
