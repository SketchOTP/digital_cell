//! D-039 focused tests: schema-3 isolation, tracer, damage, revised Stage E contract.

use chemistry_core::accounting::field_mass;
use chemistry_core::candidate_identity::canonical_params_bytes;
use chemistry_core::config::{SimParams, SurfaceTurnoverSchema};
use chemistry_core::d039_analysis::{
    gate0_contract_audit, gate1_schema_safety, revised_stage_e_membrane_contract, select_conclusion,
    v8_schema3_params, D039Conclusion,
};
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::membrane_label_tracer::MembraneLabelTracer;
use chemistry_core::surface_density::{
    apply_surface_turnover_exact, surface_turnover_lambda, SurfaceAccountingTotals,
};
use chemistry_core::Simulation;

#[test]
fn gate0_contract_allows_exchange_plus_damage() {
    let a = gate0_contract_audit();
    assert!(a.pass);
    assert!(!a.constitutive_destruction_required);
    assert_eq!(
        a.conclusion,
        "MEMBRANE_MAINTENANCE_MAY_USE_EXCHANGE_PLUS_CAUSAL_DAMAGE"
    );
}

#[test]
fn schema3_isolation_and_zero_normal_turnover() {
    let s = gate1_schema_safety();
    assert!(s.pass, "{s:?}");
    assert!(!s.schema_3_is_default);
    assert!(s.schema_3_lambda_zero);
    assert!(s.schema_1_lambda_positive);
    assert_eq!(
        SimParams::default().surface_turnover_schema,
        SurfaceTurnoverSchema::HistoricalUniform
    );
}

#[test]
fn schema3_cannot_silently_resume_as_schema1_or_2() {
    let mut p3 = v8_schema3_params();
    assert!(p3.surface_turnover_schema.is_exchange_damage_only());
    let mut p1 = p3.clone();
    p1.surface_turnover_schema = SurfaceTurnoverSchema::HistoricalUniform;
    let mut p2 = p3.clone();
    p2.surface_turnover_schema = SurfaceTurnoverSchema::D021Equivalent;
    assert_ne!(
        String::from_utf8_lossy(&canonical_params_bytes(&p3)),
        String::from_utf8_lossy(&canonical_params_bytes(&p1))
    );
    assert_ne!(
        String::from_utf8_lossy(&canonical_params_bytes(&p3)),
        String::from_utf8_lossy(&canonical_params_bytes(&p2))
    );
}

#[test]
fn exact_declared_s_to_w_damage_and_accounting() {
    let params = v8_schema3_params();
    let mut sim = Simulation::new(params);
    // Seed some interface membrane.
    for idx in 0..sim.fields.membrane.len() {
        if sim.grid.in_dish(idx) {
            let phi = sim.fields.structure[idx];
            if (0.2..0.8).contains(&phi) {
                sim.fields.membrane[idx] = 0.05;
            }
        }
    }
    let s_before = field_mass(&sim.grid, &sim.fields.membrane);
    let w_before = field_mass(&sim.grid, &sim.fields.waste);
    let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, 0.25);
    let s_after = field_mass(&sim.grid, &sim.fields.membrane);
    let w_after = field_mass(&sim.grid, &sim.fields.waste);
    assert!(report.s_removed > 0.0);
    assert!((report.s_removed - (s_before - s_after)).abs() < 1e-9);
    assert!((report.w_gained - (w_after - w_before)).abs() < 1e-9);
    assert!((report.s_removed / s_before - 0.25).abs() < 0.05);
}

#[test]
fn tracer_conservation_and_proportional_transfer() {
    let mut tracer = MembraneLabelTracer::init_from_totals(10.0, 5.0);
    assert!((tracer.conserved_inventory() - 15.0).abs() < 1e-15);
    let totals = SurfaceAccountingTotals {
        exchange_forward: 2.0,
        exchange_reverse: 1.0,
        ..Default::default()
    };
    tracer.record_accepted_exchange(&totals, 10.0, 5.0);
    assert!(tracer.inventory_residual() < 1e-12);
    // ads moves 2/10 of label_p=10 → 2; des moves 1/5 of label_s(=5+2=7) wait:
    // after ads: label_p=8, label_s=7; then des uses total_s_before=5 → moves 1/5 of label_s?
    // Implementation uses total_s_before for desorption fraction on current label_s after ads.
    // Using before-state S for des: moved = label_s * (des/s_before) = 7 * (1/5) = 1.4
    assert!((tracer.label_p + tracer.label_s - 15.0).abs() < 1e-12);

    tracer.record_declared_damage(1.0, 5.0);
    assert!(tracer.inventory_residual() < 1e-12);
    assert!(tracer.label_removed_to_w > 0.0);
}

#[test]
fn pulse_chase_old_fraction_falls_with_unlabeled_adsorption() {
    let mut tracer = MembraneLabelTracer::init_from_totals(0.0, 10.0);
    tracer.pulse_label_all_s_as_old(10.0);
    assert!((tracer.old_fraction_in_s(10.0) - 1.0).abs() < 1e-15);
    // Unlabeled adsorption grows physical S without moving old label.
    let totals = SurfaceAccountingTotals {
        exchange_forward: 2.0,
        exchange_reverse: 0.0,
        ..Default::default()
    };
    tracer.record_accepted_exchange(&totals, 5.0, 10.0);
    assert!((tracer.label_s - 10.0).abs() < 1e-15);
    let frac = tracer.old_fraction_in_s(12.0);
    assert!(frac < 1.0 && (frac - 10.0 / 12.0).abs() < 1e-12);
    assert!(tracer.replacement_fraction(12.0) >= 0.10 - 1e-12);
}

#[test]
fn no_observer_feedback_tracer_does_not_change_fields() {
    let params = v8_schema3_params();
    let mut a = Simulation::new(params.clone());
    let mut b = Simulation::new(params);
    let s = field_mass(&a.grid, &a.fields.membrane);
    let p = field_mass(&a.grid, &a.fields.precursor);
    b.membrane_label_tracer = Some(MembraneLabelTracer::init_from_totals(p, s));
    for _ in 0..20 {
        assert!(a.step());
        assert!(b.step());
    }
    for idx in 0..a.fields.membrane.len() {
        assert_eq!(a.fields.membrane[idx], b.fields.membrane[idx]);
        assert_eq!(a.fields.precursor[idx], b.fields.precursor[idx]);
        assert_eq!(a.fields.waste[idx], b.fields.waste[idx]);
    }
}

#[test]
fn schema3_zero_constitutive_loss_even_with_nonzero_k() {
    let mut p = v8_schema3_params();
    p.k_gamma_decay = 0.002;
    assert_eq!(surface_turnover_lambda(0.5, &p), 0.0);
    let (s, dw) = apply_surface_turnover_exact(1.0, 0.5, &p, 10.0);
    assert_eq!(s, 1.0);
    assert_eq!(dw, 0.0);
}

#[test]
fn revised_stage_e_drops_constitutive_ratio() {
    let c = revised_stage_e_membrane_contract();
    assert!(!c.constitutive_production_destruction_ratio_required);
    assert!(c.require_molecular_replacement);
    assert!(c.require_metabolism_dependent_damage_repair);
}

#[test]
fn conclusion_selection_ordering() {
    assert_eq!(
        select_conclusion(
            false, true, true, true, true, true, true, true, true, true, true
        ),
        D039Conclusion::ConstitutiveTurnoverContractRequired
    );
    assert_eq!(
        select_conclusion(
            true, true, true, true, true, true, true, true, true, true, true
        ),
        D039Conclusion::ExchangeDamageMaintenanceQualified
    );
    assert_eq!(
        select_conclusion(
            true, true, true, true, false, true, true, true, true, true, true
        ),
        D039Conclusion::ContinuousReplacementNotEstablished
    );
    assert_eq!(
        select_conclusion(
            true, true, true, false, true, false, true, true, true, true, true
        ),
        D039Conclusion::DamageRepairFailure
    );
}
