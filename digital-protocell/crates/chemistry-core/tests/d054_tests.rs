//! Focused D-054 unit coverage (diagnostic helpers; no production chemistry change).

use chemistry_core::d054_analysis::*;

#[test]
fn d053_source_result_provenance_constants() {
    assert_eq!(D054_D053_SOURCE_COMMIT, "76c0898e297b0abf04362df3e848e32c9d228b15");
    assert!(D054_D053_SOURCE_SUBJECT.contains("combined exterior and membrane"));
    assert_eq!(D054_D053_RESULT_TAG, "D-053-combined-resource-delivery-fail");
    assert_eq!(
        D054_V14_EXPERIMENTAL_FAILED,
        "V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED"
    );
    assert_ne!(D054_INFORMAL_D053_PRIMARY, D054_SEALED_D053_PRIMARY);
}

#[test]
fn provenance_rerun_divergence_detection() {
    assert!(provenance_rerun_diverged(
        D054_INFORMAL_D053_PRIMARY,
        D054_SEALED_D053_PRIMARY
    ));
    assert!(!provenance_rerun_diverged(
        D054_SEALED_D053_PRIMARY,
        D054_SEALED_D053_PRIMARY
    ));
}

#[test]
fn fixed_dynamic_assay_and_gate8_threshold() {
    assert!(!fixed_compartment_chi_meets_contract(0.53, 0.53));
    assert!(fixed_compartment_chi_meets_contract(1.05, 1.05));
    // Prior informal Gate 8: short-horizon relax + χ≪1.05 → defect.
    let cases = [(0.5314, 0.5314), (0.3758, 0.3758), (0.2903, 0.2903)];
    assert!(fixed_compartment_gate_defect(&cases, true));
    assert!(fixed_compartment_gate_defect(&cases, false));
    let ok = [(1.1, 1.1), (1.05, 1.06), (1.2, 1.2)];
    assert!(!fixed_compartment_gate_defect(&ok, false));
}

#[test]
fn interface_to_area_and_flux_density() {
    let sigma = interface_to_area(2.0 * std::f64::consts::PI * 22.0, std::f64::consts::PI * 22.0 * 22.0);
    assert!((sigma - 2.0 / 22.0).abs() < 1e-12);
    assert!((demand_density(10.0, 5.0) - 2.0).abs() < 1e-12);
    assert!((flux_density(8.0, 4.0) - 2.0).abs() < 1e-12);
    assert!((chi_a(3.0, 2.0, 4.0) - 0.5).abs() < 1e-12);
}

#[test]
fn radius_scaling_and_critical_radius() {
    let p = scaling_exponent(16.0, 4.0, 32.0, 1.0).unwrap();
    assert!((p + 2.0).abs() < 1e-9);
    let r_c = critical_radius_from_chi_a(16.0, 2.0, 32.0, 0.5).unwrap();
    assert!((r_c - 22.627416997969522).abs() < 1e-6);
}

#[test]
fn stage_a_band_and_selectivity_helpers() {
    assert!((selectivity_n_over_c(0.50, 0.05) - 10.0).abs() < 1e-12);
    assert!((selectivity_f_over_a(0.50, 0.05) - 10.0).abs() < 1e-12);
}

#[test]
fn gate5_admission_legacy_informal_vs_strict() {
    // Legacy informal OR-path (audit): χ-rise + a_ret>=0.5 could admit without χ≥1.05.
    assert!(!gate5_candidate_admitted(false, false, true, 0.09011105905699698));
    assert!(gate5_candidate_admitted(false, false, true, 0.50));
    assert!(gate5_candidate_admitted(true, false, false, 0.09));
    assert!(gate5_candidate_admitted(false, true, false, 0.09));
    // Strict D-055 contract rejects the same χ≪1.05 metrics.
    use chemistry_core::d053_analysis::{
        evaluate_gate5, gate5_fixture_a_pass, Gate5Verdict,
    };
    let mut ev = gate5_fixture_a_pass();
    if let Some(ref mut a) = ev.analytic {
        a.chi_n = 0.53;
        a.chi_f = 0.53;
        a.final_a_retention = 0.50;
    }
    if let Some(ref mut r) = ev.restored {
        r.chi_n = 0.53;
        r.chi_f = 0.53;
        r.final_a_retention = 0.50;
    }
    assert_eq!(evaluate_gate5(&ev), Gate5Verdict::FailResourceSufficiency);
}

#[test]
fn route_selection_provenance_stop_first() {
    let (route, conc) = select_route(
        true,  // provenance diverged
        true,  // also gate defect present as secondary
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(route, D054Route::I);
    assert_eq!(conc, D054Conclusion::D053ProvenanceRerunDiverged);
    assert_eq!(conc.as_str(), "D054_D053_PROVENANCE_RERUN_DIVERGED");
}

#[test]
fn route_selection_fixed_compartment_defect() {
    let (route, conc) = select_route(false, true, false, false, false, false, false, false, false);
    assert_eq!(route, D054Route::F);
    assert_eq!(conc, D054Conclusion::D053FixedCompartmentGateDefect);
}

#[test]
fn no_diagnostic_feedback_into_production_defaults() {
    // Frozen max candidate identity only; D-054 must not alter production m_ext/m_beta defaults.
    assert_eq!(D054_FROZEN_M_EXT, 4.0);
    assert!((D054_FROZEN_M_BETA - 0.5776226504666211).abs() < 1e-15);
    assert_eq!(D054_BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED, "BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED");
}
