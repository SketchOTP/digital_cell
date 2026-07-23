//! D-080 geometry-consistent edge-network tests.

use chemistry_core::d080_analysis::*;
use chemistry_core::edge_membrane::*;
use chemistry_core::edge_support::*;

#[test]
fn marching_squares_case_handling_closed_ring() {
    let row = geometry_qualify_row(16.0, 0.0, 0.0);
    assert!(row.closed, "{row:?}");
    assert!(row.geometric_coverage >= 0.99, "{row:?}");
    assert!(row.length_error_frac <= 0.05, "{row:?}");
}

#[test]
fn ambiguous_isolevel_determinism() {
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi_offset(w, h, 16.0, 0.0, 0.0);
    let a = build_cut_cell_support(&phi, w, h);
    let b = build_cut_cell_support(&phi, w, h);
    assert_eq!(a.adjacency, b.adjacency);
    assert!((a.interface_length - b.interface_length).abs() < 1e-12);
    let (cov, closed, _) = a.geometric_support_coverage();
    assert!(closed && cov >= 0.99);
}

#[test]
fn fractional_interface_measure_positive_on_support() {
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let s = build_cut_cell_support(&phi, w, h);
    assert!(s.n_supported() > 0);
    assert!(s.mean_positive_measure() > 0.0);
    for (k, i) in s.supported_faces() {
        assert!(s.measure(k, i) > 0.0);
        assert!(s.face_capacity(k, i, 1.0) > 0.0);
    }
}

#[test]
fn corner_connectivity_local_only() {
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let s = build_cut_cell_support(&phi, w, h);
    assert!(s.no_diagonal_leak_ok());
}

#[test]
fn translation_and_offset_invariance() {
    let a = geometry_qualify_row(22.0, 0.0, 0.0);
    let b = geometry_qualify_row(22.0, 0.5, 0.5);
    let d = (a.interface_length - b.interface_length).abs() / a.interface_length;
    assert!(d <= 0.02, "d={d} a={} b={}", a.interface_length, b.interface_length);
    assert!(a.row_ok && b.row_ok);
}

#[test]
fn closed_support_detection_and_diagnostic_fill() {
    let (w, h) = grid_for_radius(16.0);
    let phi = analytic_disk_phi(w, h, 16.0);
    let s = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    diagnostic_fill_support(&mut state, &s, 1.0);
    let params = EdgeMembraneParams {
        occupied_theta: 0.01,
        ..Default::default()
    };
    let (cov, closed, _) = connected_closed_support_observer(&state, &s, &params);
    assert!(closed);
    assert!(cov >= 0.99, "cov={cov}");
}

#[test]
fn accepted_rejected_atomicity_supported() {
    let (w, h) = (24, 24);
    let phi = analytic_disk_phi(w, h, 8.0);
    let s = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.free_l[0] = 3.0;
    let before = state.free_l.clone();
    rejected_step(&mut state);
    assert_eq!(state.free_l, before);
    let params = frozen_d079_params();
    seed_free_near_support(&mut state, &s, 0.5);
    let m0 = state.total_membrane();
    let _ = accepted_step_supported(&mut state, &phi, &s, &params, 0.05, false, 1.0);
    assert!((state.total_membrane() - m0).abs() < 1e-9);
}

#[test]
fn gate0_reproduces_d079_fingerprint() {
    let g0 = gate0_reproduce_d079();
    assert!(g0.pass, "{g0:?}");
    assert!((g0.rows[0].coverage - 0.848).abs() < 0.01);
    assert!((g0.rows[1].coverage - 0.889).abs() < 0.01);
    assert!((g0.rows[2].coverage - 0.923).abs() < 0.01);
    assert!(g0.rows.iter().all(|r| !r.closed));
}

#[test]
fn gate1_classifies_legacy_aliasing() {
    let g1 = gate1_gap_provenance();
    assert_eq!(g1.primary_cause, GapCause::LegacyCellEndpointAliasing);
    assert!(g1.rows.iter().any(|r| r.geometric_closed && !r.legacy_closed));
}

#[test]
fn gate3_geometry_qualification_pass() {
    let g3 = gate3_geometry_qualification();
    assert!(g3.pass, "{g3:?}");
    assert!(g3.translation_invariance_ok);
    assert!(g3.resolution_converging);
}

#[test]
fn ids_and_scope() {
    assert_eq!(D080_STARTING_COMMIT, "99c0236");
    assert_eq!(D080_STARTING_TAG, "D-079-edge-network-boundary-fail");
    assert_eq!(D079_CONCLUSION, "D079_EDGE_NETWORK_SELF_ASSEMBLY_FAILURE");
    assert_eq!(
        D079_PENDING_AUDIT,
        "D079_SELF_ASSEMBLY_FAILURE_PENDING_GEOMETRIC_SUPPORT_AUDIT"
    );
    assert_eq!(SCOPE_AMENDMENT, "PHASE1_EDGE_NETWORK_BOUNDARY_RESEARCH_AUTHORIZED");
}

#[test]
fn route_prefix_and_early_gates() {
    let g0 = gate0_reproduce_d079();
    assert!(g0.pass, "{g0:?}");
    let g3 = gate3_geometry_qualification();
    assert!(g3.pass, "{g3:?}");
    // Full Gates 4–9 are exercised by the experiment-runner pipeline (release).
    assert!(D080Route::Qualified.conclusion().starts_with("D080_"));
    assert!(!D080_AGENT_MEMORY_ID.is_empty());
}
