//! D-021 interface-protected membrane retention/localization tests.

use chemistry_core::config::{
    D008StageMode, EquationVersion, SimParams, MEMBRANE_SCHEMA_VERSION_V1,
    MEMBRANE_SCHEMA_VERSION_V2,
};
use chemistry_core::d011_analysis::{sensitivity_matrix, STAGE_E_FAILED_RATES};
use chemistry_core::d018_analysis::D018_FROZEN_K_STRUCTURE;
use chemistry_core::d020_analysis::{
    only_productive_rates_differ, placeholder_metrics, restoring_sign_pattern_pass,
};
use chemistry_core::d021_analysis::*;
use chemistry_core::membrane::{membrane_decay_factor, membrane_rates};
use chemistry_core::reactions::interface_weight;
use chemistry_core::{build_candidate_identity, Simulation};

fn v3_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV3StructuralScaling;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.eta_c = 1.0;
    p.eta_phi = 1.0;
    p.eta_m = 1.0;
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    p.k_membrane = 0.2;
    p.k_membrane_decay = 0.002;
    p.k_membrane_detach = 0.020;
    p
}

fn v4_params(eps_m: f64) -> SimParams {
    let mut p = v3_params();
    p.equation_version = EquationVersion::MembraneMetabolismV4InterfaceProtected;
    p.eps_m = eps_m;
    D021_ANALYTICAL_V4_RATES.apply_to(&mut p);
    p
}

#[test]
fn test_local_only_membrane_protection() {
    assert!(membrane_protection_is_local_only());
    assert!(!membrane_encodes_forbidden_target());
    for &eps in &D021_EPS_CANDIDATES {
        let p = v4_params(eps);
        let gate = evaluate_local_mechanism_gate(0.5, 0.0, 0.4, 0.3, 0.5, &p);
        assert!(gate.local_only);
        assert!(gate.no_forbidden_target);
        assert!(gate.all_pass(), "eps={eps} gate={gate:?}");
    }
}

#[test]
fn test_nonzero_interface_turnover() {
    for &eps in &D021_EPS_CANDIDATES {
        assert!(interface_turnover_nonzero(eps, 0.002, 0.5));
        let p = v4_params(eps);
        // Living interface: I(φ)=1 at φ=0.5
        let on = membrane_rates(0.5, 0.4, 0.3, 0.5, &p);
        assert!(
            on.decay > 1e-12,
            "interface must retain nonzero decay; eps={eps} decay={}",
            on.decay
        );
        let expected = p.k_membrane_decay * 0.5 * eps;
        assert!((on.decay - expected).abs() < 1e-12);
    }
}

#[test]
fn test_faster_off_interface_loss() {
    for &eps in &D021_EPS_CANDIDATES {
        assert!(faster_off_interface_loss(eps));
        let p = v4_params(eps);
        let on = membrane_rates(0.5, 0.4, 0.3, 0.5, &p);
        let off = membrane_rates(0.0, 0.4, 0.3, 0.5, &p);
        assert!(off.decay > on.decay);
        // Detachment retained off-interface.
        assert!(off.detachment > on.detachment);
        assert!((on.detachment).abs() < 1e-15);
    }
}

#[test]
fn test_no_target_radius_or_mass() {
    assert!(!membrane_encodes_forbidden_target());
    let p = v4_params(0.05);
    assert!(p.d019_mechanism_probe.is_none());
    // Decay factor uses only local I(φ) and ε_M.
    let f = decay_factor_at(0.5, 0.05);
    assert!((f - (0.05 + (1.0 - interface_weight(0.5)))).abs() < 1e-15);
}

#[test]
fn test_candidate_version_identity() {
    assert!(v4_equation_identity_ok(
        EquationVersion::MembraneMetabolismV4InterfaceProtected
    ));
    assert_eq!(
        EquationVersion::MembraneMetabolismV4InterfaceProtected.membrane_schema_version(),
        MEMBRANE_SCHEMA_VERSION_V2
    );
    assert_eq!(
        EquationVersion::MembraneMetabolismV3StructuralScaling.membrane_schema_version(),
        MEMBRANE_SCHEMA_VERSION_V1
    );
    let v3 = build_candidate_identity(v3_params(), "test", Some("d021-v3"), None, "v3", None, None);
    let v4a = build_candidate_identity(
        v4_params(0.05),
        "test",
        Some("d021-v4"),
        None,
        "v4",
        None,
        None,
    );
    let v4b = build_candidate_identity(
        v4_params(0.02),
        "test",
        Some("d021-v4"),
        None,
        "v4",
        None,
        None,
    );
    assert_ne!(v3.candidate_hash, v4a.candidate_hash);
    assert_ne!(v4a.candidate_hash, v4b.candidate_hash);
    assert_eq!(
        v4a.equation_version,
        EquationVersion::MembraneMetabolismV4InterfaceProtected
    );
}

#[test]
fn test_stage_b_localization_gate_constant() {
    assert_eq!(D021_LOCALIZATION_MIN, 0.90);
}

#[test]
fn test_stage_d_retention_gate_constant() {
    assert_eq!(D021_RETENTION_MIN, 0.80);
    let ok = placeholder_metrics([-1.0, -1.0, -1.0, -0.5], [0.9, 0.9, 0.9, 1.1]);
    let mut m = ok;
    m.catalyst_retention = 0.85;
    m.activated_retention = 0.85;
    m.membrane_localization = 0.92;
    assert!(evaluate_retention_localization(&m, 0.01).all_pass());
    m.activated_retention = 0.50;
    assert!(!evaluate_retention_localization(&m, 0.01).all_pass());
}

#[test]
fn test_bounded_solver_limits() {
    assert_eq!(D021_MAX_SOLVER_ROUNDS, 4);
    assert_eq!(D021_MAX_CANDIDATES, 5);
    assert_eq!(D021_GLOBAL_RATE_MIN_FACTOR, 0.5);
    assert_eq!(D021_GLOBAL_RATE_MAX_FACTOR, 2.0);
    let a = D021_ANALYTICAL_V4_RATES;
    let mut high = a;
    high.k_d008_structure = a.k_d008_structure * 10.0;
    let clamped = clamp_rates_to_global_bounds_d021(&high, &a);
    assert!(
        (clamped.k_d008_structure - a.k_d008_structure * D021_GLOBAL_RATE_MAX_FACTOR).abs() < 1e-12
    );
    assert!(rates_within_global_bounds_d021(&clamped, &a));

    let sens = sensitivity_matrix(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let g_history = [[1.0, 1.0, 1.0, 1.0]; 8];
    let sens_history = vec![sens; 8];
    let report = bounded_joint_solver_d021(&a, &a, &g_history, &sens_history);
    assert!(report.candidates.len() <= D021_MAX_CANDIDATES);
    assert!(report.bounded);
    assert!(report.rounds_attempted <= D021_MAX_SOLVER_ROUNDS);
}

#[test]
fn test_stage_e_restoring_radius_gate() {
    assert!(restoring_sign_pattern_pass(0.4, 0.01, -0.35));
    assert!(!restoring_sign_pattern_pass(-0.1, 0.0, -0.2));
    let conclusion = select_d021_conclusion(true, true, true, true, true, false);
    assert_eq!(conclusion.as_str(), "D021_STAGE_E_RECOVERED");
}

#[test]
fn test_historical_v3_equivalence() {
    assert!(historical_v3_decay_is_uniform(0.0, 0.5, 0.002));
    assert!(historical_v3_decay_is_uniform(1.0, 0.5, 0.002));
    let p3 = v3_params();
    let p4 = v4_params(0.05);
    let r3 = membrane_rates(0.5, 0.4, 0.3, 0.5, &p3);
    let r4 = membrane_rates(0.5, 0.4, 0.3, 0.5, &p4);
    assert!((r3.decay - p3.k_membrane_decay * 0.5).abs() < 1e-15);
    assert!(r4.decay < r3.decay); // protected at interface relative to uniform
    assert!((membrane_decay_factor(0.5, &p3) - 1.0).abs() < 1e-15);
    let a = D021_ANALYTICAL_V4_RATES;
    let mut b = a;
    b.k_membrane *= 1.1;
    assert!(only_productive_rates_differ(&a, &b));
}

#[test]
fn test_v4_short_sim_steps() {
    let mut sim = Simulation::new(v4_params(0.05));
    for _ in 0..50 {
        if !sim.step() {
            break;
        }
    }
    assert!(sim.substep > 0);
}
