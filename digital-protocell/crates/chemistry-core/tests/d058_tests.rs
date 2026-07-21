//! Focused D-058 coverage: production δ, face/dt once, volume, invariance, defective fixture, routes.

use chemistry_core::config::DX;
use chemistry_core::d056_analysis::relative_flux_error;
use chemistry_core::d057_analysis::{CarrierMeasureKind, DriveModelKind};
use chemistry_core::d058_analysis::*;

#[test]
fn seal_and_invalidation_constants() {
    assert_eq!(D058_STARTING_TAG, "D-057-carrier-geometry-driving-force-audit");
    assert_eq!(D058_D056_TAG, "D-056-waste-coupled-resource-carrier-fail");
    assert_eq!(
        D058_STARTING_COMMIT,
        "1c9d6ae73ac828622d1315e7a2137385a5ac1e71"
    );
    assert_eq!(
        D058_D056_COMMIT,
        "ed6de2cb0ce78202a665ddc4335ca198ac79b625"
    );
    assert_eq!(D058_D057_CONCLUSION, "D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT");
    assert_eq!(
        D058_INVALIDATION,
        "D056_CARRIER_IDENTIFICATION_INVALIDATED_BY_OBSERVER_NORMALIZATION"
    );
}

#[test]
fn production_delta_identity_not_interface_weight() {
    let df = 1e-6;
    let phi = 0.5;
    let prod = production_cell_delta_estimate(phi, df);
    // At φ=0.5: 6*0.5*0.5/DX = 1.5/DX
    assert!((prod - 1.5 / DX).abs() < 1e-12);
    // interface_weight(0.5) = 16*(0.25)*(0.25) = 1.0 — different construction
    let iw = 16.0 * phi * phi * (1.0 - phi) * (1.0 - phi);
    assert!((iw - 1.0).abs() < 1e-12);
    assert!((prod - iw).abs() > 0.1);
}

#[test]
fn face_measure_and_timestep_applied_once() {
    let t = dimensional_table();
    assert_eq!(t.face_measure_count, 1);
    assert_eq!(t.timestep_count, 1);
    assert_eq!(t.interface_reconstruction_count, 1);
    assert_eq!(t.cell_volume_conversion_count, 1);
    assert!((face_measure_a_f() - DX).abs() < 1e-15);
    let xi = xi_face_req(2.0, 1.5, 0.4, face_measure_a_f(), 0.01);
    assert!((xi - 2.0 * 1.5 * 0.4 * DX * 0.01).abs() < 1e-15);
    assert!(observer_kernel_parity(2.0, 1.5, 0.4, DX, 0.01));
}

#[test]
fn cell_volume_conversion() {
    let v = cell_volume();
    assert!((v - DX * DX).abs() < 1e-15);
    let xi = 0.08;
    let di = concentration_delta_from_xi(xi, v, true);
    let dj = concentration_delta_from_xi(xi, v, false);
    assert!((di - xi / v).abs() < 1e-15);
    assert!((dj + xi / v).abs() < 1e-15);
    assert!((di + dj).abs() < 1e-15);
}

#[test]
fn synthetic_dx_scaling_and_invariance_suite() {
    let (pass, checks) = synthetic_normalization_invariance(3.0);
    assert!(pass, "failed checks: {:?}", checks);
    for (name, ok) in &checks {
        assert!(ok, "invariance failed: {name}");
    }
}

#[test]
fn orientation_reversal_and_traversal_order() {
    let f = SyntheticFace {
        gamma: 1.0,
        drive: 0.5,
        a_f: 1.0,
        dt: 0.02,
        volume_i: 1.0,
        volume_j: 1.0,
        orientation: 1.0,
    };
    let mut r = f;
    r.orientation = -1.0;
    assert!((f.xi(1.0) + r.xi(1.0)).abs() < 1e-15);
    let g = SyntheticFace {
        gamma: 2.0,
        drive: 0.25,
        ..f
    };
    assert!(((f.xi(1.0) + g.xi(1.0)) - (g.xi(1.0) + f.xi(1.0))).abs() < 1e-15);
}

#[test]
fn corrected_k_t_star_and_accepted_step_only_semantics() {
    // Two accepted steps, one rejected (dt=0 contribution)
    let cap = capacity_contrib(1.0, 0.5, 1.0, 0.01)
        + capacity_contrib(1.0, 0.5, 1.0, 0.02)
        + capacity_contrib(1.0, 0.5, 1.0, 0.0); // rejected / zero
    let j_missing = 0.015;
    let k = corrected_k_t_star(j_missing, cap).unwrap();
    assert!((k - j_missing / 0.015).abs() < 1e-12); // 1*0.5*1*(0.01+0.02)=0.015
}

#[test]
fn historical_defective_estimator_fixture() {
    let fix = defective_estimator_fixture();
    assert!(fix.used_interface_weight_as_delta);
    assert!(fix.omitted_face_measure);
    assert!(fix.omitted_timestep);
    let k_def = defective_k_t_star(10.0, 2.0, 0.5).unwrap();
    assert!((k_def - 10.0).abs() < 1e-12); // 10/(2*0.5)
    // Corrected with A=1, one step dt=0.01 would differ:
    let k_corr = corrected_k_t_star(10.0, capacity_contrib(2.0, 0.5, 1.0, 0.01)).unwrap();
    assert!((k_corr - 10.0 / 0.01).abs() < 1e-9);
    assert!((k_corr / k_def - 100.0).abs() < 1e-6); // 1/dt factor
}

#[test]
fn corrected_measures_and_drives_reversible() {
    let g = gamma_face_production(0.4, 0.5, 0.4, 0.5, 1e-6);
    assert!(g > 0.0);
    assert!(
        corrected_measure_value(CarrierMeasureKind::AGammaS, g, 1.0, 0.5, 0.2) > 0.0
    );
    let d = drive_original_a(2.0, 2.0, 3.0, 0.2, 0.2, 0.1, 1.0, 1.0);
    let d_rev = drive_original_a(0.2, 0.2, 0.1, 2.0, 2.0, 3.0, 1.0, 1.0);
    assert!(d > 0.0 && d_rev < 0.0);
    assert!((d + d_rev).abs() < 1e-12);
    let db = drive_net_for_model(
        DriveModelKind::BSeparateNf,
        2.0,
        2.0,
        3.0,
        0.2,
        0.2,
        0.1,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
    );
    assert!(db > 0.0);
}

#[test]
fn starvation_and_parameter_portability_helpers() {
    let d_starved = drive_original_a(0.0, 1.0, 2.0, 0.0, 0.0, 0.0, 1.0, 1.0);
    assert!(d_starved <= 1e-15);
    let rep = build_identifiability_report(
        "M_A",
        "A",
        &[1.0, 1.5, 2.0],
        &[0.05, 0.1, 0.15],
        true,
        true,
    );
    assert!(identifiability_passes_corrected(&rep));
    let bad = build_identifiability_report(
        "M_A",
        "A",
        &[1.0, 10.0],
        &[0.05, 0.4],
        true,
        true,
    );
    assert!(!identifiability_passes_corrected(&bad));
}

#[test]
fn residual_scaling_and_route_selection() {
    let exp = chemistry_core::d057_analysis::scaling_exponent(&[8.0, 16.0, 32.0], &[1.0, 4.0, 16.0])
        .unwrap();
    assert!((exp - 2.0).abs() < 1e-6);
    assert!(corrected_surface_volume_limit(true, false, 4.0, 1.0));
    assert!(!corrected_surface_volume_limit(true, true, 4.0, 1.0));

    let (route, conc) = select_route(RouteEvidence058 {
        workspace_isolated: true,
        d057_defect_reproduced: true,
        canonical_operator_valid: true,
        observer_parity_ok: true,
        invariance_ok: true,
        original_model_portable: true,
        alt_drive_portable: false,
        surface_volume_limit: false,
        shadow_ok: true,
        architecture_rejected: false,
        kinetics_not_identifiable: false,
    });
    assert_eq!(route, D058Route::Q);
    assert_eq!(
        conc.as_str(),
        "D058_WASTE_COUPLED_CARRIER_NORMALIZATION_QUALIFIED"
    );

    let (_, fail) = select_route(RouteEvidence058 {
        workspace_isolated: false,
        d057_defect_reproduced: true,
        canonical_operator_valid: true,
        observer_parity_ok: true,
        invariance_ok: true,
        original_model_portable: false,
        alt_drive_portable: false,
        surface_volume_limit: false,
        shadow_ok: false,
        architecture_rejected: false,
        kinetics_not_identifiable: false,
    });
    assert_eq!(fail.as_str(), "D058_WORKSPACE_SCOPE_NOT_ISOLATED");
}

#[test]
fn relative_error_helper_unchanged() {
    assert!((relative_flux_error(1.0, 2.0) - 0.5).abs() < 1e-12);
    assert!(!D058_EQUATION.contains("production_biology"));
}
