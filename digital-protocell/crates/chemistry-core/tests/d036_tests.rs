//! D-036 Gate 0 — D-035 observer/runtime/ledger maturation-rate parity.

use chemistry_core::d036_analysis::{gate0_parity_audit, D035_SELECTED_K_CAT, D036_PARITY_RTOL};

#[test]
fn local_observer_runtime_j_parity() {
    let audit = gate0_parity_audit(0);
    assert!(
        audit.local_parity_ok,
        "local J mismatch count={}",
        audit.local_samples.iter().filter(|s| !s.ok).count()
    );
    assert!((audit.k_cat - D035_SELECTED_K_CAT).abs() < 1e-15);
}

#[test]
fn frozen_state_observer_runtime_unbounded_parity() {
    // advance=0 still builds frozen family; skip Gate5 restore cost in unit test path
    // by using gate0 with advance 0 — Gate5 at t=0 is still a valid pre-capacity state.
    let audit = gate0_parity_audit(0);
    assert!(
        audit.frozen_parity_ok,
        "frozen parity failed: {:?}",
        audit
            .frozen_state_reports
            .iter()
            .filter(|r| !r.parity_ok)
            .collect::<Vec<_>>()
    );
    for r in &audit.frozen_state_reports {
        assert!(
            r.observer_vs_unbounded_rel <= D036_PARITY_RTOL * 10.0
                || r.observer_vs_unbounded_rel < 1e-12,
            "state={} rel={}",
            r.state_id,
            r.observer_vs_unbounded_rel
        );
        assert!(r.stoichiometry_ok, "stoichiometry {}", r.state_id);
    }
}

#[test]
fn gate5_seed_maturation_only_parity() {
    let audit = gate0_parity_audit(0);
    assert!(
        audit.gate5_parity_ok,
        "gate5 integrated parity failed: {:?}",
        audit.gate5_integrated
    );
    let g5 = audit.gate5_integrated.expect("gate5 report");
    assert!(
        g5.observer_vs_unbounded_rel < 1e-9,
        "observer/runtime unbounded rel={}",
        g5.observer_vs_unbounded_rel
    );
}

#[test]
fn gate0_records_explicit_conclusion() {
    let audit = gate0_parity_audit(0);
    assert!(
        audit.conclusion == "D035_RUNTIME_DEFICIT_CONFIRMED"
            || audit.conclusion == "D035_RUNTIME_PARITY_PASS_NO_LARGE_DEFICIT"
            || audit.conclusion == "D036_D035_RATE_PARITY_DEFECT",
        "unexpected conclusion {}",
        audit.conclusion
    );
    assert_eq!(
        audit.mature_membrane_autocatalysis_rejected,
        "MATURE_MEMBRANE_AUTOCATALYSIS_REJECTED"
    );
}

#[test]
fn gate1_architecture_screen_runs() {
    use chemistry_core::d036_analysis::gate1_architecture_review;
    let g = gate1_architecture_review();
    assert!(
        g.valid_count >= 6,
        "expected ≥6 valid states, got {}",
        g.valid_count
    );
    assert!(
        g.conclusion == "D036_CATALYTIC_COMPLEX_ARCHITECTURE_FEASIBLE"
            || g.conclusion == "D036_CATALYTIC_COMPLEX_ARCHITECTURE_REJECTED",
        "unexpected {}",
        g.conclusion
    );
    assert!(g.zero_controls_ok);
    assert!(g.fixed_point_bounded);
    assert!(!g.estimates.is_empty());
}
