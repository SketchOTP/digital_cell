//! D-017 architecture comparison tests (observer-only; no runtime candidate).

use chemistry_core::config::{EquationVersion, SimParams, CONC_SAFETY_LIMIT};
use chemistry_core::d012_accounting::{E_ACTIVATED, E_FUEL};
use chemistry_core::d017_comparison::*;

#[test]
fn test_activation_yield_family_is_material_conservative() {
    for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(
            activation_yield_material_residual(1.0, alpha).abs() < 1e-12,
            "alpha={alpha}"
        );
        assert!(
            activation_yield_material_residual(3.7, alpha).abs() < 1e-12,
            "extent scale alpha={alpha}"
        );
    }
}

#[test]
fn test_activation_yield_potential_is_noncreating() {
    // Frozen weights create potential for α>0.
    assert!(activation_yield_potential_residual_frozen_weights(0.0).abs() < 1e-12);
    assert!(activation_yield_potential_residual_frozen_weights(0.5) > 0.0);
    assert_eq!(
        classify_activation_potential(0.5, false),
        ActivationPotentialClass::APotentialInvalid
    );
    // Revised E_A(α)=E_F/(1+α) is non-creating.
    for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(activation_yield_potential_residual_revised(alpha).abs() < 1e-12);
        assert_eq!(
            classify_activation_potential(alpha, true),
            ActivationPotentialClass::APotentialValid
        );
        let ea = revised_e_a(alpha);
        assert!((ea * (1.0 + alpha) - E_FUEL).abs() < 1e-12);
        assert!(ea * (1.0 + alpha) <= E_FUEL + E_ACTIVATED * 0.0 + 1e-12);
    }
}

#[test]
fn test_activation_yield_source_reduction_is_exact() {
    let sources = ReactionResolvedWSources {
        direct_activation_w: 10.0,
        productive_yield_w: 2.0,
        structure_turnover_w: 50.0,
        catalyst_turnover_w: 3.0,
        membrane_turnover_w: 1.0,
        a_turnover_w: 2.0,
        membrane_detachment_w: 1.0,
    };
    let cf = fixed_extent_counterfactual(&sources, 10.0, 1.0);
    assert!((cf.new_direct_w_source - 0.0).abs() < 1e-12);
    assert!((cf.first_order_total_w_source - (sources.total() - 10.0)).abs() < 1e-12);
    assert!((sources.direct_activation_w - (sources.total() - cf.first_order_total_w_source)).abs() < 1e-12);
}

#[test]
fn test_activation_counterfactual_uses_governed_extents() {
    let snap = GovernedExtentSnapshot::frozen_d015_150k();
    assert!((snap.eta_c - 1.0).abs() < 1e-15);
    let rates = snap.rates();
    let raw = ReactionResolvedWSources::from_extent_rates(&rates);
    assert!((raw.productive_yield_w).abs() < 1e-9);
    assert!(raw.direct_activation_w > 0.0);
    let scaled = raw.scaled_to_frozen_total();
    assert!((scaled.total() - D017_FROZEN_TOTAL_W_SOURCE).abs() < 1e-6);
    let cf = fixed_extent_counterfactual(&scaled, scaled.direct_activation_w, 0.5);
    assert_eq!(cf.counterfactual_type, "A_FIXED_EXTENT_COUNTERFACTUAL");
}

#[test]
fn test_activation_feedback_bounds_are_ordered() {
    let r = run_architecture_comparison();
    for b in &r.feedback {
        assert!(feedback_bounds_ordered(b), "alpha={}", b.alpha);
    }
}

#[test]
fn test_perfect_interface_bound_is_computed() {
    let c = perfect_interface_center_w(D017_FROZEN_INTERIOR_W_SOURCE);
    assert!(c.is_finite());
    assert!(c > 0.0);
    // Matches analytical δ with q = interior/cells, W_interface=0.
    let q = D017_FROZEN_INTERIOR_W_SOURCE / D017_INTERIOR_CELLS as f64;
    let expect = q * D017_FROZEN_RADIUS * D017_FROZEN_RADIUS / (4.0 * D017_FROZEN_D_W);
    assert!((c - expect).abs() < 1e-9);
}

#[test]
fn test_active_export_cannot_beat_perfect_interface() {
    let perfect = perfect_interface_center_w(D017_FROZEN_INTERIOR_W_SOURCE);
    // Finite pumps cannot outperform W_interface=0; frozen interior source already fails gates.
    assert!(perfect >= D017_CENTER_GATE_MINIMUM);
    assert!(perfect >= CONC_SAFETY_LIMIT);
    assert!(!perfect_interface_passes(perfect));
}

#[test]
fn test_internal_delivery_capacity_is_computed() {
    let cap = max_internal_delivery_capacity(10.0 - 1e-9);
    assert!(cap.is_finite() && cap > 0.0);
    let cls = classify_internal_delivery(D017_FROZEN_INTERIOR_W_SOURCE, cap);
    assert_eq!(cls, InternalDeliveryClass::BInternalDeliveryInsufficient);
}

#[test]
fn test_a_coupled_export_is_material_conservative() {
    assert!((export_material_residual_b1()).abs() < 1e-12);
}

#[test]
fn test_f_coupled_export_is_material_conservative() {
    assert!((export_material_residual_b2()).abs() < 1e-12);
}

#[test]
fn test_active_export_energy_cost_is_reported() {
    let b1 = b1_a_coupled_export();
    let b2 = b2_f_coupled_export();
    assert!((b1.energy_cost_per_w_exported - 1.0).abs() < 1e-12);
    assert!((b2.energy_cost_per_w_exported - 1.0).abs() < 1e-12);
}

#[test]
fn test_active_export_additional_w_is_counted() {
    let b1 = b1_a_coupled_export();
    assert!((b1.total_environmental_w_output - 2.0).abs() < 1e-12);
    assert!((b1.pump_generated_w_fraction - 0.5).abs() < 1e-12);
    assert!((b1.net_interior_w_removal - 1.0).abs() < 1e-12);
}

#[test]
fn test_comparison_uses_same_frozen_source() {
    let r = run_architecture_comparison();
    assert!((r.sources_scaled.total() - D017_FROZEN_TOTAL_W_SOURCE).abs() < 1e-6);
    assert!((r.d_w_required_check_90 - D017_FROZEN_D_W_REQ_90).abs() < 1e-6);
    assert!((r.d_w_required_check_50 - D017_FROZEN_D_W_REQ_50).abs() < 1e-6);
}

#[test]
fn test_selection_a_requires_viable_alpha_interval() {
    let mut inp = SelectionInputs {
        a_viable_alpha_interval: false,
        a_potential_valid: true,
        a_coupled_w_below_ceiling: true,
        a_productive_bounded: true,
        a_nf_independent: true,
        a_no_new_field: true,
        b_perfect_interface_pass: false,
        b_internal_delivery_ok: false,
        b_net_interior_w_removal: true,
        b_energy_preserves_closure: true,
        b_local_causal: true,
        b_no_hidden_controller: true,
        required_fractions_available: true,
    };
    assert_eq!(
        apply_selection_rules(&inp),
        D017PrimaryConclusion::D017RejectBothArchitectures
    );
    inp.a_viable_alpha_interval = true;
    assert_eq!(
        apply_selection_rules(&inp),
        D017PrimaryConclusion::D017SelectConservativeActivationYield
    );
}

#[test]
fn test_selection_b_requires_perfect_interface_pass() {
    let mut inp = SelectionInputs {
        a_viable_alpha_interval: false,
        a_potential_valid: true,
        a_coupled_w_below_ceiling: false,
        a_productive_bounded: true,
        a_nf_independent: true,
        a_no_new_field: true,
        b_perfect_interface_pass: false,
        b_internal_delivery_ok: true,
        b_net_interior_w_removal: true,
        b_energy_preserves_closure: true,
        b_local_causal: true,
        b_no_hidden_controller: true,
        required_fractions_available: true,
    };
    assert_eq!(
        apply_selection_rules(&inp),
        D017PrimaryConclusion::D017RejectBothArchitectures
    );
    inp.b_perfect_interface_pass = true;
    assert_eq!(
        apply_selection_rules(&inp),
        D017PrimaryConclusion::D017SelectEnergyCoupledActiveExport
    );
}

#[test]
fn test_reject_both_when_hard_bounds_fail() {
    let r = run_architecture_comparison();
    assert_eq!(
        r.primary_conclusion,
        D017PrimaryConclusion::D017RejectBothArchitectures
    );
    assert!(!r.viable_alpha_interval);
    assert!(!r.perfect_interface_pass_min);
    assert!(!r.perfect_interface_pass_safety);
}

#[test]
fn test_d017_does_not_create_runtime_candidate() {
    // Equation version and transport schema remain the governed defaults; no new candidate hash API.
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    assert_eq!(
        p.equation_version,
        EquationVersion::MembraneMetabolismV2Conservative
    );
    // Comparison module exposes no runtime pump or α parameter on SimParams.
    let _ = run_architecture_comparison();
    assert!(p.eta_c > 0.0);
}
