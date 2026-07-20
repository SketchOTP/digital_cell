//! D-046 focused tests: provenance, lineage, parity rules, models, route.

use chemistry_core::d046_analysis::{
    a_demand_lineage_catalog, basis_saturating_volumetric, basis_zero_resource_controls,
    classify_d045_threshold_provenance, classify_elasticity, classify_yield, elasticity_loo_stable,
    fit_model_a, fit_model_b, fit_model_c, fit_model_d, log_elasticity, preregistered_split,
    select_route, ADemandDecomposition, DemandScaleClass, DemandStateRow, RouteDecisionInput,
    YieldClass, D045ThresholdProvenance, D046Route, D046_D045_IMPL_FIT_ERR,
};

fn row(label: &str, family: &str, train: bool, c: f64, v: f64, l_a: f64, m_c: f64) -> DemandStateRow {
    DemandStateRow {
        label: label.into(),
        family: family.into(),
        train,
        radius: 22.0,
        c,
        n: 0.8,
        f: 0.8,
        a: 0.5,
        p: 0.05,
        s_occupancy: 0.6,
        m_c,
        interior_volume: v,
        structural_mass: v,
        membrane_area: 140.0,
        l_a,
        j_reproduction: l_a * 0.11,
        j_structural: l_a * 0.10,
        j_precursor: l_a * 0.76,
        j_decay: l_a * 0.02,
        j_out: l_a * 0.01,
        j_in: 0.0,
        k_structure_scale: 1.0,
        k_precursor_scale: 1.0,
    }
}

fn synth_campaign() -> Vec<DemandStateRow> {
    vec![
        row("R16", "radius", true, 0.8, 800.0, 96.0, 640.0),
        row("R22", "radius", true, 0.8, 1500.0, 180.0, 1200.0),
        row("R32", "radius", false, 0.8, 3200.0, 384.0, 2560.0),
        row("low_c", "catalyst", true, 0.3, 1500.0, 144.0, 450.0),
        row("med_c", "catalyst", true, 0.6, 1500.0, 168.0, 900.0),
        row("high_c", "catalyst", false, 1.0, 1500.0, 186.0, 1500.0),
        row("struct_lo", "structural", true, 0.8, 1500.0, 170.0, 1200.0),
        row("struct_hi", "structural", false, 0.8, 1500.0, 190.0, 1200.0),
        row("prec_lo", "precursor", true, 0.8, 1500.0, 100.0, 1200.0),
        row("prec_hi", "precursor", false, 0.8, 1500.0, 260.0, 1200.0),
        row("s_healthy", "membrane", true, 0.8, 1500.0, 180.0, 1200.0),
        row("s_low", "membrane", false, 0.8, 1500.0, 175.0, 1200.0),
        row("s_damaged25", "membrane", false, 0.8, 1500.0, 182.0, 1200.0),
    ]
}

#[test]
fn d045_threshold_provenance_provisional() {
    let p = classify_d045_threshold_provenance(false, true, false, false);
    assert_eq!(p, D045ThresholdProvenance::ImplementationBeforeEvidence);
    assert_eq!(
        p.rejection_status(),
        "D045_CATALYST_LINEARITY_REJECTION_PROVISIONAL"
    );
    assert!((D046_D045_IMPL_FIT_ERR - 0.25).abs() < 1e-15);
}

#[test]
fn complete_a_sink_enumeration() {
    let cat = a_demand_lineage_catalog();
    for id in [
        "L_rep",
        "L_structure",
        "L_precursor",
        "L_decay",
        "L_transport",
        "L_membrane",
    ] {
        assert!(cat.iter().any(|s| s.id == id), "missing {id}");
    }
}

#[test]
fn no_double_counting_decomposition() {
    let d = ADemandDecomposition::from_rates(19.5, 18.3, 135.1, 0.0, 3.8, 0.77, 0.0, 0.0, 177.47);
    assert!(d.residual_ok(1e-3));
    assert!(d.l_membrane.abs() < 1e-15);
}

#[test]
fn constraint_flux_exclusion_rule() {
    assert_eq!(
        classify_yield(10.0, 0.0, 1.0, false, true),
        YieldClass::ConstraintArtifact
    );
}

#[test]
fn exact_product_a_yield() {
    assert_eq!(
        classify_yield(71.0, 71.0, 1.0, false, false),
        YieldClass::ValidProductiveCost
    );
}

#[test]
fn elasticity_and_loo() {
    let v = [800.0, 1500.0, 3200.0];
    let y = [96.0, 180.0, 384.0];
    let e = log_elasticity(&v, &y).unwrap();
    assert!((e - 1.0).abs() < 0.05);
    assert_eq!(
        classify_elasticity(Some(0.21), Some(e), Some(0.05)),
        DemandScaleClass::InteriorVolumeScaled
    );
    assert!(elasticity_loo_stable(&v, &y, 2.0));
}

#[test]
fn fixed_train_holdout_split() {
    assert!(preregistered_split("R16"));
    assert!(preregistered_split("R22"));
    assert!(!preregistered_split("R32"));
    assert!(!preregistered_split("high_c"));
    assert!(preregistered_split("s_healthy"));
    assert!(!preregistered_split("s_damaged25"));
}

#[test]
fn aggregate_model_errors_and_saturation_basis() {
    let rows = synth_campaign();
    let train: Vec<_> = rows.iter().filter(|r| r.train).cloned().collect();
    let hold: Vec<_> = rows.iter().filter(|r| !r.train).cloned().collect();
    let a = fit_model_a(&train, &hold);
    let b = fit_model_b(&train, &hold);
    let c = fit_model_c(&train, &hold, 0.10);
    let d = fit_model_d(&train, &hold);
    // Mechanistic sink sum must close; aggregate A should not uniquely win.
    assert!(d.adequate);
    assert!(a.median_hold_err.is_finite());
    assert!(b.median_hold_err.is_finite());
    assert!(c.median_hold_err.is_finite());
    let r = &rows[1];
    assert!(basis_saturating_volumetric(r, 0.10) > 0.0);
}

#[test]
fn zero_cnf_controls() {
    assert!(basis_zero_resource_controls(0.10, 1.0));
}

#[test]
fn route_selection_rules() {
    assert_eq!(
        select_route(&RouteDecisionInput {
            accounting_defect: true,
            constraint_contaminated: false,
            structural_defect: false,
            precursor_defect: false,
            reproduction_defect: false,
            all_sinks_valid: false,
            volume_dominant: false,
            catalyst_saturating: false,
            model_c_adequate: false,
            basis_b_adequate: false,
            mixed_no_single_basis: false,
        }),
        D046Route::RouteA
    );
    assert_eq!(
        select_route(&RouteDecisionInput {
            accounting_defect: false,
            constraint_contaminated: false,
            structural_defect: false,
            precursor_defect: false,
            reproduction_defect: false,
            all_sinks_valid: true,
            volume_dominant: true,
            catalyst_saturating: true,
            model_c_adequate: true,
            basis_b_adequate: true,
            mixed_no_single_basis: false,
        }),
        D046Route::RouteV
    );
}

#[test]
fn no_observer_feedback_in_supply_basis() {
    let mut r = row("x", "t", true, 0.8, 1500.0, 180.0, 1200.0);
    let b1 = basis_saturating_volumetric(&r, 0.10);
    r.s_occupancy = 0.01;
    let b2 = basis_saturating_volumetric(&r, 0.10);
    assert_eq!(b1, b2);
}
