//! D-077 cooperative surface condensation architecture review tests.

use chemistry_core::d077_analysis::*;

#[test]
fn gate0_lineage_not_already_closed() {
    let g0 = gate0_lineage_audit();
    assert!(g0.pass);
    assert!(!g0.candidate_already_executed);
    assert!(g0.entries.iter().all(|e| !e.used_cooperative_chi_exchange));
    assert_eq!(g0.energy_cycle_record, ENERGY_CYCLE_RECORD);
    assert_eq!(g0.passive_record, PASSIVE_RECORD);
}

#[test]
fn chi0_recovers_frozen_linear_exchange() {
    let g1 = gate1_thermodynamic_review();
    assert!(g1.chi0_recovers_linear);
    assert!(g1.conserves_p_plus_s);
    assert!(g1.flux_follows_delta_mu);
    assert!(g1.entropy_production_nonneg);
    assert!(g1.theta_invariant);
    assert!(g1.pass, "{g1:?}");
}

#[test]
fn chemical_potential_and_flux_direction() {
    let st = ReducedState {
        theta: 0.3,
        p: 0.4,
        q_c: 1.0,
    };
    let dmu = delta_mu(st.theta, st.p, 1.0, D077_K_EQ);
    let j = flux_chi(st, 1.0, D077_K_EXCHANGE, D077_K_EQ, 1.0);
    assert!(j * dmu > 0.0);
}

#[test]
fn invariant_domain_and_nonnegative() {
    let mut th = 0.5;
    for _ in 0..5_000 {
        let st = ReducedState {
            theta: th,
            p: 0.12,
            q_c: 0.8,
        };
        let j = flux_chi(st, 1.4, D077_K_EXCHANGE, D077_K_EQ, 1.0);
        let max_up = (1.0 - th).max(0.0);
        let max_dn = th.max(0.0);
        let mut d = j * 1e-3;
        if d > max_up {
            d = max_up;
        } else if d < -max_dn {
            d = -max_dn;
        }
        th += d;
        assert!((0.0..=1.0).contains(&th));
    }
}

#[test]
fn required_chi_span_ok_but_loo_fails_portability() {
    let g2 = gate2_cohesion_reconstruction();
    assert!(
        g2.chi_span_095 <= PORTABILITY_SPAN_MAX,
        "span={}",
        g2.chi_span_095
    );
    assert!(
        !g2.loo_median_factor_ok,
        "expected LOO fail between constitutive and reduced-p"
    );
    assert!(g2.selected_chi < FRUMKIN_CRITICAL_CHI);
    assert!(!g2.pass);
    assert_eq!(
        g2.failure.as_deref(),
        Some("D077_COOPERATIVE_COHESION_NOT_PORTABLE")
    );
}

#[test]
fn fixed_point_jacobian_stable_below_critical() {
    let chi = gate2_cohesion_reconstruction().selected_chi;
    let g6 = gate6_stability(chi);
    assert!(!g6.bistable_risk);
    assert!(g6.healthy_stable);
    assert!(g6.damage_in_basin);
    assert!(g6.no_spontaneous_fill_from_negligible_p);
    assert!(g6.no_permanent_after_total_loss);
    assert!(g6.pass, "{g6:?}");
}

#[test]
fn gross_replacement_active_at_equilibrium() {
    let chi = gate2_cohesion_reconstruction().selected_chi;
    let g4 = gate4_replacement(chi, P_CONSTITUTIVE_R22, 1.0);
    assert!(g4.near_zero_net);
    assert!(g4.positive_gross);
    assert!(g4.replacement_in_horizon);
    assert!(g4.pass, "{g4:?}");
}

#[test]
fn damage_and_starvation_controls() {
    let chi = gate2_cohesion_reconstruction().selected_chi;
    let g5 = gate5_damage_starvation(chi);
    assert!(g5.pass, "{g5:?}");
    assert!(g5
        .controls
        .iter()
        .any(|c| c.name == "single_10pct_damage_recovery" && c.pass));
    assert!(g5
        .controls
        .iter()
        .any(|c| c.name == "no_precursor_fails_repair" && c.pass));
}

#[test]
fn metabolic_also_infeasible_under_measured_a_collapse() {
    let chi = gate2_cohesion_reconstruction().selected_chi;
    let g3 = gate3_metabolic_feasibility(chi);
    assert!(!g3.any_non_control_qualifies);
    assert!(g3.constitutive_hits_membrane_a_collapses);
    assert!(!g3.pass);
    assert_eq!(
        g3.failure.as_deref(),
        Some("D077_COOPERATIVE_EXCHANGE_METABOLICALLY_INFEASIBLE")
    );
}

#[test]
fn radius_occupancy_ok_but_retention_fails() {
    let chi = gate2_cohesion_reconstruction().selected_chi;
    let g7 = gate7_radius_portability(chi);
    assert!(g7.rows.iter().all(|r| r.occ_ok));
    assert!(g7.rows.iter().all(|r| !r.a_ok));
    assert!(!g7.pass);
}

#[test]
fn route_selects_cohesion_not_portable() {
    let review = run_full_review();
    assert_eq!(review.route.route, D077Route::CohesionNotPortable);
    assert_eq!(
        review.route.conclusion,
        "D077_COOPERATIVE_COHESION_NOT_PORTABLE"
    );
    assert!(!review.route.next_execution_started);
    assert_eq!(
        review.frozen_preservation.energy_cycle_record,
        ENERGY_CYCLE_RECORD
    );
}

#[test]
fn historical_preservation_ids() {
    let fp = frozen_preservation();
    assert!(fp.ids_ok);
    assert_eq!(fp.d076_conclusion, D076_CONCLUSION);
    assert!((fp.k_eq - D077_K_EQ).abs() < 1e-15);
}
