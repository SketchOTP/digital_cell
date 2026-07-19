//! D-037 focused audit tests (no chemistry changes).

use chemistry_core::d037_analysis::{
    effective_interface_hazard, gate0_turnover_lineage, gate1_bulk_surface_equivalence,
    gate2_turnover_provenance, gate3_state_classification, gate4_renewal_gate_semantics,
    gate5_reduced_dynamics, gate6_multistart, gate7_route_decision, run_d037_audit,
    select_route_for_flags, RecoveryRoute, StateClass, TurnoverProvenanceClass,
    D037_FROZEN_EPS_M, D037_K_MEMBRANE_DECAY, D037_LOSS_EQUIV_RTOL, D037_V7_TURNOVER_TRANSFER_COMMIT,
};

#[test]
fn turnover_lineage_constants_and_transfer_commit() {
    let g0 = gate0_turnover_lineage();
    assert!(g0.resolved);
    assert!(g0.failure.is_none());
    assert_eq!(g0.transfer_commit, D037_V7_TURNOVER_TRANSFER_COMMIT);
    assert!(g0.lineage.iter().any(|e| e.directive == "D-021"));
    assert!(g0.lineage.iter().any(|e| e.directive == "D-024"));
    assert!(g0.lineage.iter().any(|e| e.equation.contains("ε_M")));
}

#[test]
fn matched_state_loss_detects_eps_omission() {
    let g1 = gate1_bulk_surface_equivalence();
    assert!(!g1.all_pass, "expected transfer defect");
    assert_eq!(g1.conclusion, "D037_SURFACE_TURNOVER_TRANSFER_DEFECT");
    assert!(g1.max_relative_error > D037_LOSS_EQUIV_RTOL);
    for s in &g1.samples {
        // Diffuse-interface matched seeds put mass across I(φ)<1 wings, so mean bulk
        // hazard > ε_M·k; surface still exceeds protected bulk (typically ~2–3× here,
        // approaching 1/ε_M only for idealized I≡1 localization).
        assert!(
            s.eps_omission_factor > 1.0 + D037_LOSS_EQUIV_RTOL,
            "expected surface loss > protected bulk, got factor {}",
            s.eps_omission_factor
        );
        assert!(!s.pass);
        assert!((s.mass_bulk - s.mass_surface).abs() / s.mass_bulk.max(1e-18) < 1e-9);
    }
}

#[test]
fn eps_protection_preservation_constant() {
    let haz = effective_interface_hazard(D037_FROZEN_EPS_M, D037_K_MEMBRANE_DECAY);
    assert!((haz - D037_FROZEN_EPS_M * D037_K_MEMBRANE_DECAY).abs() < 1e-15);
    assert!((D037_K_MEMBRANE_DECAY / haz - 1.0 / D037_FROZEN_EPS_M).abs() < 1e-9);
}

#[test]
fn delta_normalization_surface_equals_k_times_s() {
    let g1 = gate1_bulk_surface_equivalence();
    for s in &g1.samples {
        let expected = D037_K_MEMBRANE_DECAY * s.mass_surface;
        let rel = (s.l_surface - expected).abs() / expected.max(1e-18);
        assert!(
            rel < 1e-9,
            "δ·k·Γ should equal k·S; rel={rel} at R={} w={}",
            s.radius,
            s.interface_width
        );
    }
}

#[test]
fn radius_and_interface_width_surface_hazard_stable() {
    let g1 = gate1_bulk_surface_equivalence();
    let hazards: Vec<f64> = g1
        .samples
        .iter()
        .map(|s| s.l_surface / s.mass_surface.max(1e-18))
        .collect();
    let base = hazards[0];
    for h in &hazards {
        let rel = (h - base).abs() / base.max(1e-18);
        assert!(rel <= 0.05, "hazard drift {rel} exceeds 5%");
    }
}

#[test]
fn turnover_provenance_mixed_and_unsupported() {
    let g2 = gate2_turnover_provenance();
    assert_eq!(g2.classification, TurnoverProvenanceClass::MixedPurposeTerm);
    assert_eq!(
        g2.unsupported_flag.as_deref(),
        Some("D037_TURNOVER_PROVENANCE_UNSUPPORTED")
    );
    assert!(!g2.evidence.is_empty());
}

#[test]
fn state_classification_forced_ineligible() {
    let g3 = gate3_state_classification();
    assert!(g3.pointwise_balance_on_nonequilibrium);
    assert_eq!(
        g3.flag.as_deref(),
        Some("POINTWISE_BALANCE_APPLIED_TO_NONEQUILIBRIUM_STATES")
    );
    for s in &g3.states {
        if s.state_id.contains("highU") || s.state_id == "balanced" || s.state_id.contains("lowU") {
            assert_eq!(s.class, StateClass::ForcedSyntheticState);
            assert!(!s.eligible_for_pointwise_balance);
        }
    }
}

#[test]
fn steady_versus_transient_balance_eligibility() {
    let g3 = gate3_state_classification();
    let steady_eligible = g3
        .states
        .iter()
        .filter(|s| s.eligible_for_pointwise_balance)
        .count();
    assert_eq!(steady_eligible, 0);
    let g4 = gate4_renewal_gate_semantics();
    assert!(g4.defect);
    assert!(g4.d034_portability_rejection_not_upheld);
    assert!(g4.d036_architecture_rejection_not_upheld);
}

#[test]
fn reduced_fixed_points_and_jacobian() {
    let g5 = gate5_reduced_dynamics();
    let d34 = g5
        .fixed_points
        .iter()
        .find(|f| f.architecture == "d034_linear")
        .expect("d034");
    // Under inherited λ and the audit's lumped J_p, S*=J_p/λ exceeds unit capacity —
    // formal eigenvalues remain negative, but the FP is not physically admissible.
    assert!(d34.jacobian_eigenvalues.iter().all(|e| *e < 0.0));
    assert!(
        !d34.admissible || d34.s_star <= 1.0,
        "admissible FP must respect capacity"
    );
}

#[test]
fn multistart_convergence_screen_runs() {
    let g5 = gate5_reduced_dynamics();
    let g6 = gate6_multistart(&g5);
    assert!(!g6.outcomes.is_empty());
    assert!(g6.outcomes.iter().all(|o| o.bounded && o.nonnegative));
}

#[test]
fn route_selection_rules() {
    assert_eq!(
        select_route_for_flags(true, true, true),
        RecoveryRoute::RouteATurnoverTransferRepair
    );
    assert_eq!(
        select_route_for_flags(false, true, false),
        RecoveryRoute::RouteBTurnoverReidentification
    );
    let bundle = run_d037_audit("D-20260719-1040-d037-turnover-provenance-renewal-gate-audit");
    let g7 = gate7_route_decision(
        &bundle.gate1,
        &bundle.gate2,
        &bundle.gate4,
        &bundle.gate5,
        &bundle.gate6,
    );
    assert_eq!(g7.primary_conclusion, "D037_TURNOVER_AND_GATE_DEFECTS");
    assert_eq!(
        g7.selected_route,
        RecoveryRoute::RouteATurnoverTransferRepair
    );
    assert!(!g7.next_execution_started);
    assert_eq!(g7.d008_status, "BLOCKED_NOT_RECOVERED");
}
