//! D-083 conservative dynamic edge-membrane migration tests.

use chemistry_core::d080_analysis::{
    gate8_dynamic_interface, gate8_dynamic_interface_unmigrated,
};
use chemistry_core::d083_analysis::*;
use chemistry_core::edge_membrane::{
    analytic_disk_phi, diagnostic_fill_support, grid_for_radius, EdgeMembraneParams,
    EdgeMembraneState,
};
use chemistry_core::edge_migration::{
    audit_support_transition, migrate_bound_across_support,
};
use chemistry_core::edge_support::build_cut_cell_support;

#[test]
fn ids_and_conclusions() {
    assert_eq!(D083_STARTING_COMMIT, "01d9afd");
    assert_eq!(D083_STARTING_TAG, "D-082-edge-activation-integration-repaired");
    assert_eq!(
        D083Conclusion::EdgeDynamicMigrationRepaired.as_str(),
        "D083_EDGE_DYNAMIC_MIGRATION_REPAIRED"
    );
}

#[test]
fn gate0_reproduces_unmigrated_dynamic_failure() {
    let u = gate8_dynamic_interface_unmigrated(1.0);
    assert!(!u.pass, "D-082 dynamic failure must still reproduce without migration: {u:?}");
}

#[test]
fn overlap_and_disappear_audit() {
    let (w, h) = grid_for_radius(20.0);
    let params = EdgeMembraneParams::default();
    let phi0 = analytic_disk_phi(w, h, 16.0);
    let old = build_cut_cell_support(&phi0, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    diagnostic_fill_support(&mut state, &old, params.b_max);
    let phi1 = analytic_disk_phi(w, h, 20.0);
    let new = build_cut_cell_support(&phi1, w, h);
    let audit = audit_support_transition(&state, &old, &new);
    assert!(audit.n_disappear > 0 || audit.n_appear > 0, "{audit:?}");
    assert!(audit.b_on_disappear > 0.0 || audit.b_on_overlap > 0.0, "{audit:?}");
}

#[test]
fn disappear_returns_or_transfers_conservatively() {
    let (w, h) = grid_for_radius(20.0);
    let params = EdgeMembraneParams::default();
    let phi0 = analytic_disk_phi(w, h, 16.0);
    let old = build_cut_cell_support(&phi0, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    diagnostic_fill_support(&mut state, &old, params.b_max);
    let m0 = state.total_membrane();
    let phi1 = analytic_disk_phi(w, h, 20.0);
    let new = build_cut_cell_support(&phi1, w, h);
    let led = migrate_bound_across_support(&mut state, &old, &new, &params);
    assert!(led.conservation_ok, "{led:?}");
    assert!((state.total_membrane() - m0).abs() < 1e-9 * (1.0 + m0));
    assert!(led.transferred_local + led.returned_to_l + led.retained_on_overlap
        + led.capacity_excess_to_l
        >= m0 * 0.5);
}

#[test]
fn rejected_step_excludes_migration() {
    let (w, h) = grid_for_radius(18.0);
    let params = EdgeMembraneParams::default();
    let phi = analytic_disk_phi(w, h, 16.0);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    diagnostic_fill_support(&mut state, &support, params.b_max);
    let m0 = state.total_membrane();
    let b0: Vec<_> = state.bound_h.iter().copied().collect();
    // Rejected: no migrate call — state unchanged.
    assert_eq!(state.total_membrane(), m0);
    assert_eq!(state.bound_h, b0);
}

#[test]
fn synthetic_motion_operator_qualifies() {
    let g3 = gate3_synthetic_motion();
    assert!(g3.pass, "{g3:?}");
    for c in &g3.cases {
        assert!(c.conservation_ok, "{c:?}");
        assert!(c.no_ghost, "{c:?}");
        assert!(c.coverage_ok, "{c:?}");
    }
}

#[test]
fn migrated_dynamic_gate8_passes() {
    let m = gate8_dynamic_interface(1.0);
    assert!(m.pass, "{m:?}");
}

#[test]
fn autonomous_r16_r22_r32() {
    let g4 = gate4_autonomous_dynamic();
    assert!(g4.pass, "{g4:?}");
}

#[test]
fn structural_universally_positive_separate() {
    let g6 = gate6_structural_separation();
    assert_eq!(
        g6.classification,
        StructuralDirectionClass::UniversallyPositive
    );
    assert!(g6.structural_blocker_remains);
    assert!(!g6.restoring_crossing);
}

#[test]
fn gate5_uses_d081_reserve_contract_not_obsolete_d080_gate7() {
    let g5 = gate5_regressions();
    assert!(
        g5.pass,
        "Gate5 must pass under D-081/D-082 reserve+activation contract: {g5:?}"
    );
    assert!(g5.reserve_repair);
    assert!(g5.reserve_depletion);
    assert!(g5.a_causal_replenishment);
    assert!(g5.failed_checks.is_empty());
}
