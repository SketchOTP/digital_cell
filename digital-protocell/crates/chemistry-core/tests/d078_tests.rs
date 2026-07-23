//! D-078 Phase 1 boundary substrate redesign downselect tests.

use chemistry_core::d078_analysis::*;

#[test]
fn preservation_closes_ps_architecture() {
    let p = frozen_preservation();
    assert!(p.ids_ok);
    assert_eq!(p.ps_architecture_record, PS_ARCHITECTURE_RECORD);
    assert_eq!(p.d077_conclusion, D077_CONCLUSION);
    assert!(p.production_biology_unchanged);
    assert!(p.frozen_evidence.len() >= 10);
}

#[test]
fn gate0_both_candidates_novel() {
    let g0 = gate0_lineage_audit();
    assert!(g0.pass, "{g0:?}");
    assert!(g0.candidate_a.novel);
    assert!(g0.candidate_b.novel);
    assert!(!g0.candidate_a.is_rename_of_closed_architecture);
    assert!(!g0.candidate_b.is_rename_of_closed_architecture);
    assert!(g0
        .candidate_b
        .genuinely_new_mechanism
        .contains("free-energy"));
}

#[test]
fn gate1_conservation_and_local_causality() {
    for c in [
        D078CandidateId::StructureNative,
        D078CandidateId::SingleAmphiphile,
    ] {
        let g1 = gate1_conservation(c);
        assert!(g1.pass, "{g1:?}");
        assert!(g1.explicit_sources_sinks);
        assert!(g1.no_observer_driven_chemistry);
        assert!(g1.no_target_mass_radius_coverage_health);
        assert!(g1.starvation_removes_repair_flow);
    }
    assert!(local_causality_ok(false, false, false));
    assert!(!local_causality_ok(true, false, false));
}

#[test]
fn gate2_neither_meets_retention_gates() {
    let a = gate2_coupled_feasibility(D078CandidateId::StructureNative);
    let b = gate2_coupled_feasibility(D078CandidateId::SingleAmphiphile);
    assert!(!a.pass);
    assert!(!b.pass);
    assert!(a.phi_maintenance_a_cost_included);
    assert!(b.m_production_a_cost_included);
    assert!(a.rows.iter().all(|r| !r.a_ok));
    assert!(b.rows.iter().all(|r| !r.a_ok));
    assert!(a.optimistic_a_ceiling < A_RETENTION_GATE);
}

#[test]
fn gate3_no_restoring_crossing_under_current_structure() {
    for c in [
        D078CandidateId::StructureNative,
        D078CandidateId::SingleAmphiphile,
    ] {
        let g3 = gate3_structural_stability(c);
        assert!(!g3.pass, "{g3:?}");
        assert!(!g3.large_negative);
        assert!(!g3.no_universal_growth);
        assert!(g3.samples.len() == 3);
    }
}

#[test]
fn gate4_boundary_function_fails_retention() {
    let a = gate4_boundary_function(D078CandidateId::StructureNative);
    let b = gate4_boundary_function(D078CandidateId::SingleAmphiphile);
    assert!(a.one_global_param_set && b.one_global_param_set);
    assert!(!a.pass);
    assert!(!b.pass);
    assert!(a
        .rows
        .iter()
        .all(|r| r.a_ret + 1e-12 < A_RETENTION_GATE));
    assert!(b
        .rows
        .iter()
        .all(|r| r.a_ret + 1e-12 < A_RETENTION_GATE));
}

#[test]
fn gate5_a_lacks_molecular_replacement_b_starvation_ok() {
    let a = gate5_repair_controls(D078CandidateId::StructureNative);
    let b = gate5_repair_controls(D078CandidateId::SingleAmphiphile);
    assert!(!a.real_molecular_replacement);
    assert!(!a.pass);
    assert!(b.real_molecular_replacement);
    assert!(b.starvation_blocks_indefinite_repair);
    // B still fails overall science via other gates; repair algebra alone may pass.
    let recovery = reduced_damage_recovery(true, true);
    assert!(recovery >= DAMAGE_RECOVERY_GATE);
    assert_eq!(reduced_damage_recovery(false, true), 0.0);
}

#[test]
fn jacobian_and_complexity_scoring() {
    assert!(jacobian_stable(1.0));
    assert!(!jacobian_stable(0.0));
    let ca = gate6_complexity(D078CandidateId::StructureNative);
    let cb = gate6_complexity(D078CandidateId::SingleAmphiphile);
    assert!(ca.total < cb.total);
    assert_eq!(ca.new_fields, 0);
    assert_eq!(cb.new_fields, 1);
}

#[test]
fn radius_portability_uses_one_global_set() {
    let a = gate4_boundary_function(D078CandidateId::StructureNative);
    assert_eq!(a.rows.len(), 3);
    assert!(a.rows.iter().all(|r| [16.0, 22.0, 32.0].contains(&r.radius)));
    let i = interface_strength_proxy(0.5, 1.0);
    assert!(i > 0.0);
    let p = face_permeability(1.0, 4.0, i);
    assert!(p < 1.0);
    assert!((amphiphile_seal_proxy(0.95, 1.0) - 0.95).abs() < 1e-12);
}

#[test]
fn energy_budgets_unaffordable() {
    let review = run_full_review();
    assert!(review.energy_budgets.iter().all(|e| !e.affordable));
    assert!(review.energy_budgets.iter().all(|e| e.optimistic_a_retention < A_RETENTION_GATE));
}

#[test]
fn route_selects_continuum_rejected() {
    let review = run_full_review();
    assert!(review.evidence_complete);
    assert!(review.gate0.pass);
    assert!(!review.candidate_a.science_pass);
    assert!(!review.candidate_b.science_pass);
    assert_eq!(review.route.route, D078Route::ContinuumRejected);
    assert_eq!(
        review.route.conclusion,
        "D078_CONTINUUM_BOUNDARY_SUBSTRATE_REJECTED"
    );
    assert!(!review.route.next_execution_started);
    assert_eq!(review.route.d008_status, "BLOCKED_NOT_RECOVERED");
    assert_eq!(
        review.route.phase1_status,
        "PHASE1_SELF_MAINTENANCE_PARTIAL"
    );
    assert_eq!(review.route.production_verdict, "REQUIRES_REMEDIATION");
    assert!(review
        .route
        .reasons
        .iter()
        .any(|r| r == PS_ARCHITECTURE_RECORD));
}

#[test]
fn definitions_forbid_closed_mechanisms() {
    let a = candidate_a_definition();
    let b = candidate_b_definition();
    assert!(a.forbidden.iter().any(|f| f.contains("membrane-material")));
    assert!(b.forbidden.iter().any(|f| f.contains("precursor")));
    assert!(b.equations.iter().any(|e| e.contains("∇·(L_M ∇μ_M)")));
}
