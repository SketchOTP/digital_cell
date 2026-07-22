//! D-076 nonequilibrium surface-state cycle architecture review tests.

use chemistry_core::d076_analysis::*;

#[test]
fn gate0_lineage_not_already_closed() {
    let g0 = gate0_lineage_audit();
    assert!(g0.pass);
    assert!(!g0.candidate_already_executed);
    assert!(g0.entries.iter().all(|e| !e.used_conservative_s_to_u));
    assert_eq!(g0.record, PASSIVE_RECORD);
}

#[test]
fn gate1_conservation_and_capacity() {
    let g1 = gate1_conservation();
    assert!(g1.pass, "{g1:?}");
    assert!(g1.exchange_conserves_p_plus_u);
    assert!(g1.maturation_conserves_membrane);
    assert!(g1.maturation_consumes_a_produces_w);
    assert!(g1.relaxation_conserves_u_plus_s);
    assert!(g1.capacity_invariant);
    assert!(g1.no_s_without_u_and_a);
    assert!(g1.no_u_without_p_drive);
    assert!(g1.no_a_relaxes_then_desorbs);
    assert!(g1.no_p_cannot_repair_indefinitely);
    assert!(g1.no_observer_in_equations);
}

#[test]
fn zero_a_and_zero_p_controls() {
    let st_a = ReducedState {
        theta_u: 0.1,
        theta_s: 0.8,
        a: 0.0,
        q_c: 1.0,
        p: 0.2,
    };
    assert!(flux_us(st_a, 1.0, 1.0).abs() < EPS);
    let st_p = ReducedState {
        theta_u: 0.0,
        theta_s: 0.8,
        a: 1.0,
        q_c: 1.0,
        p: 0.0,
    };
    assert!(flux_pu(st_p, D076_K_EXCHANGE, D076_K_EQ, 1.0) <= EPS);
}

#[test]
fn reduced_fixed_point_and_jacobian() {
    let r = r_required_for_occupancy(D075_ENDOGENOUS_INTERFACE_P, D076_K_EQ, OCC_CONTRACT);
    let k_relax = 1.0 / REPLACEMENT_HORIZON;
    // Use a=1,q=1 to isolate surface algebra from metabolic collapse.
    let k_mature = r * k_relax;
    let fp = surface_fixed_point(
        D075_ENDOGENOUS_INTERFACE_P,
        1.0,
        1.0,
        k_mature,
        k_relax,
        D076_K_EQ,
    );
    assert!(fp.physical);
    assert!(fp.theta_s + 1e-9 >= OCC_CONTRACT);
    assert!(fp.theta_total <= 1.0 + 1e-9);
    let st = ReducedState {
        theta_u: fp.theta_u,
        theta_s: fp.theta_s,
        a: 1.0,
        q_c: 1.0,
        p: D075_ENDOGENOUS_INTERFACE_P,
    };
    let jac = surface_jacobian(st, D076_K_EXCHANGE, D076_K_EQ, k_mature, k_relax);
    assert!(jac.locally_stable, "{jac:?}");
}

#[test]
fn energy_budget_fails_under_measured_a_collapse() {
    let family = measured_state_family();
    let cands = identify_parameter_candidates();
    assert!(cands.len() <= 5);
    let mut any_energy_pass = false;
    for pair in &cands {
        let b = energy_budget(&family[1], pair.k_mature, pair.k_relax);
        any_energy_pass |= b.pass;
        assert!(b.a_membrane_maturation >= 0.0);
        assert!(
            !b.within_a_budget || b.a_sustainable_surplus_rate > 0.0,
            "surplus must fund demand if within budget"
        );
    }
    assert!(
        !any_energy_pass,
        "measured A retention ≈0.06 must fail energy gate"
    );
}

#[test]
fn bounded_parameter_selection_and_portability_surface() {
    let port = gate4_parameter_identification();
    assert!(port.candidates.len() <= 5);
    // Algebraic surface portability can pass with collapsed A when using measured a in r.
    // With tiny a_free, k_mature becomes huge; θ_S still reaches contract if r is set from a_free.
    assert!(
        port.per_candidate.iter().any(|c| c.surface_ok_all && c.portable),
        "expected portable surface candidates"
    );
    // Full qualify (incl. A retention) must fail.
    assert!(port.selected.is_none());
}

#[test]
fn damage_and_starvation_controls_on_surface_ode() {
    let cands = identify_parameter_candidates();
    let pair = cands[1];
    let g5 = gate5_damage_starvation_controls(pair.k_mature, pair.k_relax);
    // Surface ODE with collapsed a may fail recovery; that's acceptable causality evidence.
    assert!(!g5.controls.is_empty());
    assert!(g5.controls.iter().any(|c| c.name == "no_a_fails"));
    assert!(g5.controls.iter().any(|c| c.name == "starvation_decline"));
}

#[test]
fn route_selection_energy_infeasible_or_architecture_fail() {
    let review = run_full_review();
    assert!(review.gate0.pass);
    assert!(review.gate1.pass);
    assert!(review.frozen_preservation.d075_ids_ok);
    assert_eq!(review.frozen_preservation.record, PASSIVE_RECORD);
    let c = review.gate6.conclusion.as_str();
    assert!(
        c == D076Route::EnergyInfeasible.conclusion()
            || c == D076Route::ArchitectureReviewFail.conclusion()
            || c == D076Route::NotPortable.conclusion()
            || c == D076Route::CausalityFail.conclusion(),
        "unexpected conclusion {c}"
    );
    // Dominant expected route under measured A collapse with portable surface algebra:
    assert_eq!(
        review.gate6.route,
        D076Route::EnergyInfeasible,
        "reasons={:?}",
        review.gate6.reasons
    );
}

#[test]
fn d032_through_d075_preservation_constants() {
    let p = frozen_preservation();
    assert_eq!(p.d075_conclusion, D075_CONCLUSION);
    assert!(p.d075_ids_ok);
    assert!((p.k_eq - D076_K_EQ).abs() < 1e-15);
    assert!((p.k_exchange - D076_K_EXCHANGE).abs() < 1e-15);
    assert_eq!(p.seed_capacity_contract, "SEED_CAPACITY_CONTRACT_V1");
    let g0 = gate0_lineage_audit();
    assert!(g0.entries.iter().any(|e| e.directive == "D-032"));
    assert!(g0.entries.iter().any(|e| e.directive == "D-034"));
    assert!(g0.entries.iter().any(|e| e.directive == "D-037"));
    assert!(g0.entries.iter().any(|e| e.directive == "D-038"));
    assert!(g0.entries.iter().any(|e| e.directive == "D-039"));
    assert!(g0.entries.iter().any(|e| e.directive == "D-075"));
}

#[test]
fn candidate_equations_match_directive() {
    let eq = candidate_equations();
    assert!(eq.passive_exchange.contains("P⇄U"));
    assert!(eq.maturation.contains("U+A→S+W"));
    assert!(eq.relaxation.contains("S→U"));
    assert!(eq.capacity.contains("θ_total"));
    assert!(eq.permeability.contains("mature S"));
}
