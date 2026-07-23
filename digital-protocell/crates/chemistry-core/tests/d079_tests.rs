//! D-079 conserved edge-network membrane feasibility tests.

use chemistry_core::d079_analysis::*;
use chemistry_core::edge_membrane::*;

#[test]
fn face_indexing_roundtrip() {
    let s = EdgeMembraneState::new(5, 4);
    assert_eq!(s.n_h(), 4 * 4);
    assert_eq!(s.n_v(), 5 * 3);
    assert_eq!(s.h_idx(1, 2), 2 * 4 + 1);
    assert_eq!(s.v_idx(2, 1), 1 * 5 + 2);
    let (i0, j0, i1, j1) = s.face_cells(FaceKind::Horizontal, s.h_idx(1, 2));
    assert_eq!((i0, j0, i1, j1), (1, 2, 2, 2));
}

#[test]
fn gate0_schema_and_scope() {
    let g0 = gate0_preservation();
    assert!(g0.pass, "{g0:?}");
    assert_eq!(g0.scope_amendment, SCOPE_AMENDMENT);
    assert_eq!(g0.equation_version, EQUATION_VERSION_EDGE_NETWORK);
}

#[test]
fn gate1_conservation_pass() {
    let g1 = gate1_conservation();
    assert!(g1.pass, "{g1:?}");
    assert!(g1.rejected_atomic);
    assert!(g1.damage_b_to_w);
}

#[test]
fn accepted_rejected_atomicity() {
    let mut s = EdgeMembraneState::new(8, 8);
    s.free_l[0] = 3.0;
    let before = s.free_l.clone();
    rejected_step(&mut s);
    assert_eq!(s.free_l, before);
    assert_eq!(s.rejected_steps, 1);
}

#[test]
fn bind_unbind_conserves_total() {
    let params = EdgeMembraneParams::default();
    let phi = analytic_disk_phi(24, 24, 8.0);
    let mut s = EdgeMembraneState::new(24, 24);
    s.catalyst = 1.0;
    seed_free_near_interface(&mut s, &phi, 0.9);
    let m0 = s.total_membrane();
    for _ in 0..200 {
        let _ = accepted_step(&mut s, &phi, &params, 0.05, false);
    }
    assert!((s.total_membrane() - m0).abs() < 1e-5, "m0={m0} m={}", s.total_membrane());
}

#[test]
fn capacity_not_exceeded() {
    let params = EdgeMembraneParams {
        k_bind: 50.0,
        ..EdgeMembraneParams::default()
    };
    let phi = analytic_disk_phi(24, 24, 8.0);
    let mut s = EdgeMembraneState::new(24, 24);
    s.catalyst = 1.0;
    seed_free_near_interface(&mut s, &phi, 5.0);
    for _ in 0..500 {
        let _ = accepted_step(&mut s, &phi, &params, 0.05, false);
    }
    assert!(s.bound_h.iter().chain(s.bound_v.iter()).all(|v| *v <= params.b_max + 1e-9));
}

#[test]
fn legacy_snapshot_cannot_resume() {
    let legacy = EdgeSnapshot {
        equation_version: "old".into(),
        field_schema: "old".into(),
        schema_version: 0,
        width: 8,
        height: 8,
        free_l: vec![0.0; 64],
        bound_h: vec![0.0; 56],
        bound_v: vec![0.0; 56],
        waste: 0.0,
        activated: 0.0,
        catalyst: 1.0,
        params: EdgeMembraneParams::default(),
    };
    let mut s = EdgeMembraneState::new(8, 8);
    assert!(legacy.resume_into(&mut s).is_err());
}

#[test]
fn stage_a_targets_constants() {
    assert_eq!(STAGE_A_C_PERM_MAX, 0.05);
    assert_eq!(STAGE_A_NF_PERM_LO, 0.20);
    assert_eq!(STAGE_A_W_PERM_MIN, 0.70);
}

#[test]
fn permeability_monotonic_in_theta() {
    let p0 = face_permeability(0.0, BETA_C);
    let p1 = face_permeability(1.0, BETA_C);
    assert!(p1 < p0);
    assert!(p1 <= STAGE_A_C_PERM_MAX + 1e-6);
}

#[test]
fn historical_preservation_ids() {
    assert_eq!(D079_STARTING_COMMIT, "039044f");
    assert_eq!(D078_CONCLUSION, "D078_CONTINUUM_BOUNDARY_SUBSTRATE_REJECTED");
}

#[test]
fn route_stops_honestly() {
    let review = run_full_review();
    assert!(review.gate0.pass);
    assert!(review.gate1.pass);
    let c = review.route.conclusion.as_str();
    eprintln!(
        "D079 route={} conclusion={} stopped={} gate2_pass={} assembly={:?}",
        review.route.route.as_str(),
        c,
        review.route.stopped_at_gate,
        review.gate2.pass,
        review.gate2.rows
    );
    assert!(c.starts_with("D079_"), "{c}");
    assert!(!review.route.next_execution_started);
    assert_eq!(review.scope_amendment, SCOPE_AMENDMENT);
}

#[test]
fn closed_network_detection_observer_only() {
    // Empty network is not closed.
    let s = EdgeMembraneState::new(16, 16);
    let phi = analytic_disk_phi(16, 16, 5.0);
    let params = EdgeMembraneParams::default();
    let (cov, closed, _) = connected_closed_observer(&s, &phi, &params);
    assert_eq!(cov, 0.0);
    assert!(!closed);
}
