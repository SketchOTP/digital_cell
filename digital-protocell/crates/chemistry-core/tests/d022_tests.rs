//! D-022 interface-affinity membrane localization tests.

use chemistry_core::config::{
    D008StageMode, EquationVersion, SimParams, MEMBRANE_TRANSPORT_SCHEMA_VERSION_V1,
    MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2, GRID_HEIGHT, GRID_WIDTH,
};
use chemistry_core::d011_analysis::{sensitivity_matrix, STAGE_E_FAILED_RATES};
use chemistry_core::d018_analysis::D018_FROZEN_K_STRUCTURE;
use chemistry_core::d020_analysis::{placeholder_metrics, restoring_sign_pattern_pass};
use chemistry_core::d022_analysis::*;
use chemistry_core::grid::Grid;
use chemistry_core::membrane::{membrane_face_flux, membrane_transport_rate};
use chemistry_core::operators::total_mass;
use chemistry_core::{build_candidate_identity, Simulation};

fn v4_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV4InterfaceProtected;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.eta_c = 1.0;
    p.eta_phi = 1.0;
    p.eta_m = 1.0;
    p.eps_m = D022_FROZEN_EPS_M;
    p.chi_m = 0.0;
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    p.k_membrane = 0.2;
    p.d_m = 0.001;
    p
}

fn v5_params(ratio: f64) -> SimParams {
    let mut p = v4_params();
    p.equation_version = EquationVersion::MembraneMetabolismV5InterfaceAffinity;
    p.chi_m = chi_m_from_ratio(p.d_m, ratio);
    D022_ANALYTICAL_V5_RATES.apply_to(&mut p);
    p
}

#[test]
fn test_antisymmetric_m_flux() {
    let j = membrane_face_flux(0.4, 0.6, 0.2, 0.8, 0.001, 0.002);
    let jr = membrane_face_flux(0.6, 0.4, 0.8, 0.2, 0.001, 0.002);
    assert!((j + jr).abs() < 1e-15, "J(i→j)={j} J(j→i)={jr}");
}

#[test]
fn test_m_transport_conservation() {
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut membrane = vec![0.0; n];
    let mut phi = vec![0.0; n];
    let mut out = vec![0.0; n];
    let mut scratch = vec![0.0; n];
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let x = (idx % GRID_WIDTH) as f64;
        let y = (idx / GRID_WIDTH) as f64;
        let r = ((x - 96.0).powi(2) + (y - 96.0).powi(2)).sqrt();
        phi[idx] = 0.5 * (1.0 - ((r - 22.0) / 2.0).tanh());
        membrane[idx] = 0.3 + 0.1 * ((x * 0.05).sin());
    }
    let p = v5_params(1.0);
    membrane_transport_rate(&grid, &membrane, &phi, &p, &mut scratch, &mut out);
    let sum: f64 = out
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, v)| *v)
        .sum();
    assert!(sum.abs() < 1e-9, "transport mass rate sum={sum}");
    assert!(out.iter().all(|v| v.is_finite()));
}

#[test]
fn test_local_only_interface_affinity() {
    assert!(affinity_is_local_only());
    assert!(!affinity_encodes_forbidden_target());
}

#[test]
fn test_chi_zero_v4_equivalence() {
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut membrane = vec![0.0; n];
    let mut phi = vec![0.0; n];
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let x = (idx % GRID_WIDTH) as f64;
        membrane[idx] = 0.25 + 0.05 * (x * 0.03).cos();
        phi[idx] = 0.5;
    }
    let mut p4 = v4_params();
    let mut p5 = v5_params(1.0);
    p5.chi_m = 0.0;
    let mut o4 = vec![0.0; n];
    let mut o5 = vec![0.0; n];
    let mut s4 = vec![0.0; n];
    let mut s5 = vec![0.0; n];
    membrane_transport_rate(&grid, &membrane, &phi, &p4, &mut s4, &mut o4);
    membrane_transport_rate(&grid, &membrane, &phi, &p5, &mut s5, &mut o5);
    let max_diff = o4
        .iter()
        .zip(o5.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    assert!(max_diff < 1e-12, "max_diff={max_diff}");
    let _ = p4;
}

#[test]
fn test_bounded_candidate_screen() {
    assert_eq!(D022_CHI_OVER_D_RATIOS, [0.5, 1.0, 2.0]);
    assert_eq!(D022_MAX_CANDIDATES, 5);
    assert_eq!(D022_MAX_SOLVER_ROUNDS, 4);
    let a = D022_ANALYTICAL_V5_RATES;
    let mut high = a;
    high.k_membrane = a.k_membrane * 10.0;
    let clamped = clamp_rates_to_global_bounds_d022(&high, &a);
    assert!(rates_within_global_bounds_d022(&clamped, &a));
    let sens = sensitivity_matrix(&[
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let sens_history = vec![sens.clone(), sens.clone(), sens.clone(), sens];
    let report = bounded_joint_solver_d022(&a, &a, &[[1.0; 4]; 4], &sens_history);
    assert!(report.candidates.len() <= D022_MAX_CANDIDATES);
    assert!(report.bounded);
}

#[test]
fn test_stage_b_localization_gate_constant() {
    assert_eq!(D022_LOCALIZATION_MIN, 0.90);
}

#[test]
fn test_stage_d_retention_gate_constant() {
    assert_eq!(D022_RETENTION_MIN, 0.80);
    let mut m = placeholder_metrics([-1.0; 4], [1.0; 4]);
    m.catalyst_retention = 0.85;
    m.activated_retention = 0.85;
    m.membrane_localization = 0.92;
    assert!(localization_promotion_gate(&m, 0.01));
}

#[test]
fn test_stage_e_restoring_radius_gate() {
    assert!(restoring_sign_pattern_pass(0.4, 0.01, -0.35));
    assert_eq!(
        select_d022_conclusion(true, true, true, true, true, false).as_str(),
        "D022_STAGE_E_RECOVERED"
    );
}

#[test]
fn test_candidate_schema_identity() {
    assert!(v5_identity_ok(
        EquationVersion::MembraneMetabolismV5InterfaceAffinity
    ));
    assert_eq!(
        EquationVersion::MembraneMetabolismV5InterfaceAffinity.membrane_transport_schema_version(),
        MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2
    );
    assert_eq!(
        EquationVersion::MembraneMetabolismV4InterfaceProtected.membrane_transport_schema_version(),
        MEMBRANE_TRANSPORT_SCHEMA_VERSION_V1
    );
    let v4 = build_candidate_identity(v4_params(), "t", Some("v4"), None, "v4", None, None);
    let v5a = build_candidate_identity(v5_params(0.5), "t", Some("v5"), None, "v5", None, None);
    let v5b = build_candidate_identity(v5_params(1.0), "t", Some("v5"), None, "v5", None, None);
    assert_ne!(v4.candidate_hash, v5a.candidate_hash);
    assert_ne!(v5a.candidate_hash, v5b.candidate_hash);
}

#[test]
fn test_v5_short_sim_no_negative_m() {
    let mut sim = Simulation::new(v5_params(1.0));
    for _ in 0..40 {
        if !sim.step() {
            break;
        }
    }
    assert!(sim.substep > 0);
    assert!(sim.fields.membrane.iter().all(|&m| m >= -1e-9));
    let mass = total_mass(&sim.grid, &sim.fields.membrane);
    assert!(mass.is_finite());
}
