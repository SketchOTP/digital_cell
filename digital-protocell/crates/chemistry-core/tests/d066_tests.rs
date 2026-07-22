//! D-066 activation utilization audit tests (shadow/diagnostic only).

use chemistry_core::activated_metabolism::activation_isolated_delta;
use chemistry_core::d050_analysis::schema2_activation_rate;
use chemistry_core::d066_analysis::*;

#[test]
fn d065_reproduction_predicate_matches_control_c_pattern() {
    assert!(d065_reproduction_predicate(2.5, 1.8, 1.3, 0.40, 1.81, 0.40));
    assert!(!d065_reproduction_predicate(0.9, 1.8, 1.3, 0.40, 1.81, 0.40));
}

#[test]
fn activation_stoichiometry_parity_n_f_a_w() {
    assert!(activation_stoichiometry_parity(1.25));
    assert!(activation_isolated_stoichiometric(2.0));
    let d = activation_isolated_delta(2.0);
    assert!((d[2] + 2.0).abs() < 1e-15);
    assert!((d[3] + 2.0).abs() < 1e-15);
    assert!((d[5] - 2.0).abs() < 1e-15);
    assert!((d[4] - 2.0).abs() < 1e-15);
}

#[test]
fn request_equals_accepted_no_execution_defect() {
    assert!(!acceptance_execution_defect(0.5, 0.5, true));
    assert!(acceptance_execution_defect(0.5, 0.4, true));
    assert!(!acceptance_execution_defect(0.5, 0.4, false));
}

#[test]
fn limiter_classification() {
    let none = classify_limiter(1.0, 1.0, 1.0, 1.0, 1.0, true, false, false);
    assert_eq!(none, ActivationLimiterClass::NoLimit);
    let nlim = classify_limiter(1.0, 1.0, 0.1, 1.0, 1.0, true, false, false);
    assert_eq!(nlim, ActivationLimiterClass::NLimited);
    let ts = classify_limiter(1.0, 0.0, 1.0, 1.0, 1.0, true, false, true);
    assert_eq!(ts, ActivationLimiterClass::TimestepLimited);
}

#[test]
fn rejected_step_contribution_zero() {
    assert_eq!(accepted_contribution(false, 3.0), 0.0);
    assert_eq!(accepted_contribution(true, 3.0), 3.0);
}

#[test]
fn spatial_overlap_integral() {
    let phi = vec![1.0, 1.0, 0.0];
    let n = vec![1.0, 0.5, 0.0];
    let f = vec![1.0, 0.5, 0.0];
    let c = vec![1.0, 1.0, 0.0];
    let indices = vec![0, 1];
    let o = overlap_integral_o_cnf(&phi, &n, &f, &c, &indices);
    assert!(o > 0.0);
    let fa = f_active(&phi, &n, &f, &c, &indices, 1e-6);
    assert!(fa > 0.0);
}

#[test]
fn mass_conservative_redistribution() {
    let mut n = vec![0.0, 1.0, 3.0];
    let mut f = vec![0.0, 2.0, 2.0];
    let indices = vec![1, 2];
    redistribute_nf_uniform(&mut n, &mut f, &indices);
    assert!((n[1] + n[2] - 4.0).abs() < 1e-12);
    assert!((f[1] + f[2] - 4.0).abs() < 1e-12);
    let mut n2 = vec![0.0, 4.0, 0.0];
    let mut f2 = vec![0.0, 2.0, 0.0];
    let c = vec![0.0, 1.0, 3.0];
    let phi = vec![0.0, 1.0, 1.0];
    redistribute_nf_catalyst_weighted(&mut n2, &mut f2, &c, &phi, &indices);
    assert!((n2[1] + n2[2] - 4.0).abs() < 1e-12);
}

#[test]
fn utilization_classes() {
    assert!((utilization(2.0, 4.0) - 0.5).abs() < 1e-12);
    assert_eq!(
        classify_utilization(0.6, 0.1, 1.0).as_str(),
        "HIGH_DELIVERY_HIGH_UTILIZATION"
    );
}

#[test]
fn catalyst_support_classification() {
    let total = classify_catalyst_support(10.0, 2.0, false, true, 0.4);
    assert_eq!(total.as_str(), "TOTAL_C_LIMIT");
    let spat = classify_catalyst_support(2.0, 2.0, true, false, 0.4);
    assert_eq!(spat.as_str(), "C_SPATIAL_SUPPORT_LIMIT");
}

#[test]
fn a_ledger_closes_and_demand() {
    let led = ALedger066 {
        g_activation: 10.0,
        l_catalyst: 1.0,
        l_structure: 1.0,
        l_precursor: 6.0,
        l_decay: 1.0,
        j_out: 1.0,
        j_in: 0.0,
        delta_a: 0.0,
        activation_requested: 10.0,
        activation_accepted: 10.0,
        j_n_net: 5.0,
        j_f_net: 5.0,
    };
    assert!(led.closes(1e-9));
    assert_eq!(led.dominant_sink(), "precursor");
    // g=10 < td=10 is false equality; bump demand so precursor dominates below production
    let led2 = ALedger066 {
        g_activation: 5.0,
        l_catalyst: 1.0,
        l_structure: 1.0,
        l_precursor: 8.0,
        l_decay: 1.0,
        j_out: 1.0,
        j_in: 0.0,
        delta_a: -6.0, // residual: 5-1-1-8-1-1+0-(-6)= -1+6=5? wait close ledger
        activation_requested: 5.0,
        activation_accepted: 5.0,
        j_n_net: 5.0,
        j_f_net: 5.0,
    };
    // residual = 5 -1 -1 -8 -1 -1 +0 - (-6) = 5-12+6 = -1; adjust
    let led2 = ALedger066 {
        g_activation: 5.0,
        l_catalyst: 0.5,
        l_structure: 0.5,
        l_precursor: 8.0,
        l_decay: 0.5,
        j_out: 0.5,
        j_in: 0.0,
        delta_a: -5.0, // 5-0.5-0.5-8-0.5-0.5+0-(-5)=5-10+5=0
        activation_requested: 5.0,
        activation_accepted: 5.0,
        j_n_net: 5.0,
        j_f_net: 5.0,
    };
    assert!(led2.closes(1e-9));
    assert_eq!(led2.dominant_sink(), "precursor");
    let cls = led2.classify_demand().as_str();
    assert!(
        cls == "PRECURSOR_DEMAND_DOMINANT" || cls == "GROSS_ACTIVATION_BELOW_TOTAL_DEMAND",
        "got {cls}"
    );
}

#[test]
fn route_w_when_waste_masks() {
    let ev = RouteEvidence066 {
        workspace_isolated: true,
        d065_reproduced: true,
        lineage_ok: true,
        runtime_parity_ok: true,
        fate_ledger_ok: true,
        a_ledger_ok: true,
        acceptance_execution_defect: false,
        waste_masks_activation: true,
        usable_windows_available: false,
        redistribution_restores_a: false,
        ordinary_delivery_fails_a: true,
        healthy_c_restores_a_under_ordinary_nf: false,
        local_nf_and_c_sufficient_still_insufficient: false,
        activation_sufficient_demand_net_loss: false,
        multiple_limits_flagged: false,
        a_retention: 0.4,
        chi_smooth_min: 1.8,
        chi_a: 0.3,
    };
    assert_eq!(select_route(ev).0, D066Route::W);
}

#[test]
fn route_x_before_overlap() {
    let ev = RouteEvidence066 {
        workspace_isolated: true,
        d065_reproduced: true,
        lineage_ok: true,
        runtime_parity_ok: true,
        fate_ledger_ok: true,
        a_ledger_ok: true,
        acceptance_execution_defect: true,
        waste_masks_activation: false,
        usable_windows_available: true,
        redistribution_restores_a: true,
        ordinary_delivery_fails_a: true,
        healthy_c_restores_a_under_ordinary_nf: false,
        local_nf_and_c_sufficient_still_insufficient: false,
        activation_sufficient_demand_net_loss: false,
        multiple_limits_flagged: false,
        a_retention: 0.4,
        chi_smooth_min: 1.8,
        chi_a: 0.3,
    };
    assert_eq!(select_route(ev).0, D066Route::X);
}

#[test]
fn route_o_when_redistribute_restores() {
    let ev = RouteEvidence066 {
        workspace_isolated: true,
        d065_reproduced: true,
        lineage_ok: true,
        runtime_parity_ok: true,
        fate_ledger_ok: true,
        a_ledger_ok: true,
        acceptance_execution_defect: false,
        waste_masks_activation: false,
        usable_windows_available: true,
        redistribution_restores_a: true,
        ordinary_delivery_fails_a: true,
        healthy_c_restores_a_under_ordinary_nf: false,
        local_nf_and_c_sufficient_still_insufficient: false,
        activation_sufficient_demand_net_loss: false,
        multiple_limits_flagged: false,
        a_retention: 0.4,
        chi_smooth_min: 1.8,
        chi_a: 0.3,
    };
    assert_eq!(select_route(ev).0, D066Route::O);
}

#[test]
fn route_k_frozen_capacity() {
    let ev = RouteEvidence066 {
        workspace_isolated: true,
        d065_reproduced: true,
        lineage_ok: true,
        runtime_parity_ok: true,
        fate_ledger_ok: true,
        a_ledger_ok: true,
        acceptance_execution_defect: false,
        waste_masks_activation: false,
        usable_windows_available: true,
        redistribution_restores_a: false,
        ordinary_delivery_fails_a: true,
        healthy_c_restores_a_under_ordinary_nf: false,
        local_nf_and_c_sufficient_still_insufficient: true,
        activation_sufficient_demand_net_loss: false,
        multiple_limits_flagged: false,
        a_retention: 0.4,
        chi_smooth_min: 1.8,
        chi_a: 0.3,
    };
    let (route, conc) = select_route(ev);
    assert_eq!(route, D066Route::K);
    assert_eq!(conc.as_str(), "D066_FROZEN_ACTIVATION_CAPACITY_LIMIT");
}

#[test]
fn frozen_capacity_rate_matches_schema2() {
    let r = capacity_rate_at(0.4, 0.4, 0.4, 1.0);
    let expect = schema2_activation_rate(
        D066_V_A, 1.0, 0.4, 0.4, 0.4, D066_K_C, D066_N_REF, D066_F_REF,
    );
    assert!((r - expect).abs() < 1e-15);
}

#[test]
fn lineage_documents_schema2() {
    let lin = activation_lineage();
    assert_eq!(lin.equation_version, D066_EQUATION_VERSION);
    assert!(lin.zero_resource_controls_pass);
    assert!((lin.v_a - D066_V_A).abs() < 1e-18);
}
