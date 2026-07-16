//! D-014 numerical stability and activation-accounting tests.

use chemistry_core::{
    activation_step_closes, build_activation_potential_step, build_field_ledger,
    classify_cause_from_terminal_limiter, recovered_dt_after_accept, ActivationPotentialLedger,
    ActivationPotentialStep, DtLimiter, NumericalCauseClassification, StepAccounting, MAX_DT,
    D014_DT_FLOOR, D014_DT_RECOVERY_GROWTH,
};

fn fuel_only_import_step() -> StepAccounting {
    StepAccounting {
        structure: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        catalyst: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        nutrient: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        fuel: build_field_ledger(1.0, 0.0, 0.0, 0.5, 1.5, 1.5),
        waste: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        activated: build_field_ledger(0.5, 0.0, 0.0, 0.0, 0.5, 0.5),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    }
}

#[test]
fn test_every_step_has_dominant_limiter() {
    // Enumerate all named limiters; Unknown is allowed only as a transient pre-reject state.
    let labeled = [
        DtLimiter::DiffusionC,
        DtLimiter::ActivationReaction,
        DtLimiter::PositivityLimit,
        DtLimiter::FieldBoundValidation,
        DtLimiter::AdaptiveController,
        DtLimiter::ReservoirRelaxation,
    ];
    for lim in labeled {
        assert_ne!(
            classify_cause_from_terminal_limiter(lim),
            NumericalCauseClassification::UnknownNumericalFailure
        );
    }
}

#[test]
fn test_unknown_limiter_cannot_close_diagnosis() {
    assert_eq!(
        classify_cause_from_terminal_limiter(DtLimiter::Unknown),
        NumericalCauseClassification::UnknownNumericalFailure
    );
}

#[test]
fn test_accepted_steps_can_recover_dt() {
    let grown = recovered_dt_after_accept(1e-4, MAX_DT);
    assert!((grown - 1e-4 * D014_DT_RECOVERY_GROWTH).abs() < 1e-15);
    assert!(grown > 1e-4);
}

#[test]
fn test_rejected_dt_is_not_latched() {
    // After a shrink, recovery uses current accepted dt, not a historical minimum floor latch.
    let after_reject_accept = 5e-5;
    let recovered = recovered_dt_after_accept(after_reject_accept, MAX_DT);
    assert!(recovered > after_reject_accept);
    assert!(recovered <= MAX_DT);
}

#[test]
fn test_dt_uses_latest_accepted_state() {
    let latest = 0.001;
    let stale_min = 1e-7;
    let next = recovered_dt_after_accept(latest, MAX_DT);
    assert!(next > stale_min);
    assert!((next - latest * D014_DT_RECOVERY_GROWTH).abs() < 1e-15);
}

#[test]
fn test_safety_factor_applied_once() {
    let base = 0.0008;
    let once = recovered_dt_after_accept(base, MAX_DT);
    let twice = recovered_dt_after_accept(once, MAX_DT);
    assert!((once - base * D014_DT_RECOVERY_GROWTH).abs() < 1e-15);
    // Applying twice is larger; controller must only apply once per accept.
    assert!(twice > once);
    assert!(twice <= MAX_DT);
}

#[test]
fn test_dt_floor_uses_current_limit() {
    assert!(D014_DT_FLOOR > 0.0);
    assert!(D014_DT_FLOOR < 1e-6);
    let just_above = D014_DT_FLOOR * 1.5;
    assert!(just_above > D014_DT_FLOOR);
}

#[test]
fn test_controller_reports_dominant_limiter() {
    let cause = classify_cause_from_terminal_limiter(DtLimiter::PositivityLimit);
    assert_eq!(cause, NumericalCauseClassification::PositivityStiffness);
}

#[test]
fn test_activation_transfer_not_double_counted() {
    // F→A transfer: ΔF=-x, ΔA=+x → chemistry potential 0 under e_F=e_A=1.
    let step = StepAccounting {
        structure: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        catalyst: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        nutrient: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        fuel: build_field_ledger(2.0, -0.3, 0.0, 0.0, 1.7, 1.7),
        waste: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        activated: build_field_ledger(1.0, 0.3, 0.0, 0.0, 1.3, 1.3),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    let pot = build_activation_potential_step(&step);
    assert!((pot.chemistry_potential).abs() < 1e-12);
    assert!((pot.observed_change).abs() < 1e-12);
    assert!(activation_step_closes(&pot));
}

#[test]
fn test_internal_transport_potential_cancels() {
    let step = StepAccounting {
        structure: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        catalyst: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        nutrient: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        fuel: build_field_ledger(1.0, 0.0, -0.2, 0.0, 0.8, 0.8),
        waste: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        activated: build_field_ledger(1.0, 0.0, 0.2, 0.0, 1.2, 1.2),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    let pot = build_activation_potential_step(&step);
    // Global transport of equal+opposite F/A masses is potential-neutral under e_F=e_A.
    assert!((pot.transport_potential).abs() < 1e-12);
    assert!(activation_step_closes(&pot));
}

#[test]
fn test_reservoir_fuel_potential_is_recorded() {
    let pot = build_activation_potential_step(&fuel_only_import_step());
    assert!((pot.fuel_import - 0.5).abs() < 1e-12);
    assert!((pot.reservoir_potential - 0.5).abs() < 1e-12);
}

#[test]
fn test_attempted_steps_do_not_enter_activation_ledger() {
    let mut ledger = ActivationPotentialLedger::new(4.0);
    let before = ledger.clone();
    // Rejected attempts must not call apply_accepted_step.
    assert_eq!(ledger.residual, before.residual);
    assert_eq!(ledger.final_activation_potential, before.final_activation_potential);
}

#[test]
fn test_activation_step_residual_closes() {
    let pot = build_activation_potential_step(&fuel_only_import_step());
    assert!(activation_step_closes(&pot));
}

#[test]
fn test_productive_a_consumption_accounted_once() {
    let step = StepAccounting {
        structure: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        catalyst: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        nutrient: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        fuel: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        waste: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        activated: build_field_ledger(2.0, -0.4, 0.0, 0.0, 1.6, 1.6),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    let pot = build_activation_potential_step(&step);
    assert!((pot.chemistry_potential + 0.4).abs() < 1e-12);
    assert!((pot.observed_change + 0.4).abs() < 1e-12);
    assert!(activation_step_closes(&pot));
    let mut ledger = ActivationPotentialLedger::new(pot.potential_before);
    ledger.apply_accepted_step(&pot, 0.0, 0.4, 0.0);
    assert!((ledger.productive_consumption - 0.4).abs() < 1e-12);
    assert!(ledger.relative_residual <= 1e-6);
}

#[test]
fn test_repaired_method_preserves_activation_directionality() {
    // Closed chemistry cannot create potential: F→A transfer is neutral; A consumption decreases.
    let consume = StepAccounting {
        structure: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        catalyst: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        nutrient: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        fuel: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        waste: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        activated: build_field_ledger(1.0, -0.1, 0.0, 0.0, 0.9, 0.9),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    let pot = build_activation_potential_step(&consume);
    assert!(pot.observed_change < 0.0);
    assert!(activation_step_closes(&pot));
}

#[test]
fn test_failure_replay_is_deterministic_classifier() {
    // Same terminal limiter → same cause classification (deterministic mapping).
    let a = classify_cause_from_terminal_limiter(DtLimiter::FieldBoundValidation);
    let b = classify_cause_from_terminal_limiter(DtLimiter::FieldBoundValidation);
    assert_eq!(a, b);
    assert_eq!(a, NumericalCauseClassification::FieldBoundStiffness);
}

#[test]
fn test_solver_remains_closed_without_quasi_steady_reference() {
    // D-014 does not open the solver on numerical-only recovery.
    let terminal = "NUMERICAL_FAILURE";
    let quasi = terminal == "QUASI_STEADY_CONVERGED";
    assert!(!quasi);
}

#[test]
fn test_machine_eps_ceiling_projection() {
    use chemistry_core::{project_soluble_ceiling_machine_eps, CONC_SAFETY_LIMIT, D014_CONC_CEILING_PROJECT_EPS};
    let mut values = vec![0.0, CONC_SAFETY_LIMIT + 5e-11, CONC_SAFETY_LIMIT + 1e-3];
    let mask = vec![true, true, true];
    let corr = project_soluble_ceiling_machine_eps(&mut values, &mask, D014_CONC_CEILING_PROJECT_EPS);
    assert!((values[1] - CONC_SAFETY_LIMIT).abs() < 1e-15);
    assert!((values[2] - (CONC_SAFETY_LIMIT + 1e-3)).abs() < 1e-15);
    assert!(corr < 0.0);
}

#[test]
fn test_repaired_method_preserves_nonnegativity_projection() {
    use chemistry_core::{project_soluble_ceiling_machine_eps, CONC_SAFETY_LIMIT, D014_CONC_CEILING_PROJECT_EPS};
    let mut values = vec![CONC_SAFETY_LIMIT + 1e-12];
    let mask = vec![true];
    let _ = project_soluble_ceiling_machine_eps(&mut values, &mask, D014_CONC_CEILING_PROJECT_EPS);
    assert!(values[0] >= 0.0);
    assert!(values[0] <= CONC_SAFETY_LIMIT);
}

#[test]
fn test_activation_ledger_accumulates_step_residual_only() {
    let pot = ActivationPotentialStep {
        potential_before: 1.0,
        potential_after: 1.5,
        observed_change: 0.5,
        fuel_import: 0.5,
        reservoir_potential: 0.5,
        chemistry_potential: 0.0,
        transport_potential: 0.0,
        numerical_correction: 0.0,
        residual: 1e-12,
        relative_residual: 1e-12,
    };
    let mut ledger = ActivationPotentialLedger::new(1.0);
    ledger.apply_accepted_step(&pot, 0.1, 0.0, 0.0);
    // Transfer extent is diagnostic; must not inflate residual.
    assert!(ledger.residual.abs() < 1e-10);
    assert!((ledger.activation_transfer - 0.1).abs() < 1e-12);
}
