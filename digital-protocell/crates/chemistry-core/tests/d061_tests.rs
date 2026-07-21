//! Focused D-061 coverage: mode identity, parity, drive/route classifiers, resume.

use chemistry_core::config::StructureEvolutionMode;
use chemistry_core::d061_analysis::*;
use chemistry_core::Simulation;
use chemistry_core::SimParams;

#[test]
fn seal_and_frozen_constants() {
    assert_eq!(D061_STARTING_COMMIT, "5e3abdf");
    assert_eq!(D061_STARTING_TAG, "D-060-structural-size-feedback-audit");
    assert_eq!(D061_D060_CONCLUSION, "D060_STRUCTURAL_GEOMETRY_EXECUTION_DEFECT");
    assert!((D061_FROZEN_KT - 1.4346157818803311).abs() < 1e-15);
    assert_eq!(
        D061_AGENT_MEMORY_ID,
        "D-20260721-d061-structural-constraint-execution-repair"
    );
}

#[test]
fn d060_defect_reproduction_predicate() {
    assert!(d060_defect_reproduced(true, true, true));
    assert!(!d060_defect_reproduced(false, true, true));
    assert!(!d060_defect_reproduced(true, false, true));
    assert!(!d060_defect_reproduced(true, true, false));
}

#[test]
fn structure_mode_identity_and_dispatch() {
    assert!(StructureEvolutionMode::FixedGeometry.apply_phi() == false);
    assert!(StructureEvolutionMode::DynamicStructure.apply_phi());
    assert!(StructureEvolutionMode::FixedGeometry.enforce_constraint());
    assert!(!StructureEvolutionMode::DynamicStructure.enforce_constraint());
    assert!(structure_mode_identity_differs(
        StructureEvolutionMode::FixedGeometry,
        StructureEvolutionMode::DynamicStructure
    ));
    assert!(!structure_mode_identity_differs(
        StructureEvolutionMode::FixedGeometry,
        StructureEvolutionMode::FixedGeometry
    ));
}

#[test]
fn simulation_mode_setters_sync_legacy_bool() {
    let mut sim = Simulation::new(SimParams::default());
    assert_eq!(
        sim.structure_evolution_mode,
        StructureEvolutionMode::FixedGeometry
    );
    assert!(sim.enforce_structure_constraint);
    assert!(!sim.apply_phi_updates());

    sim.set_structure_evolution_mode(StructureEvolutionMode::DynamicStructure);
    assert!(!sim.enforce_structure_constraint);
    assert!(sim.apply_phi_updates());

    sim.set_enforce_structure_constraint(true);
    assert_eq!(
        sim.structure_evolution_mode,
        StructureEvolutionMode::FixedGeometry
    );
}

#[test]
fn configuration_identity_includes_structure_mode() {
    let params = SimParams::default();
    let grid = chemistry_core::candidate_identity::GridConfiguration::default();
    let h_fixed = chemistry_core::configuration_hash_with_structure_mode(
        &params,
        &grid,
        StructureEvolutionMode::FixedGeometry,
    );
    let h_dyn = chemistry_core::configuration_hash_with_structure_mode(
        &params,
        &grid,
        StructureEvolutionMode::DynamicStructure,
    );
    assert_ne!(h_fixed, h_dyn);
    let id = chemistry_core::build_candidate_identity_with_structure_mode(
        params,
        "test",
        None,
        None,
        "d061",
        None,
        None,
        StructureEvolutionMode::DynamicStructure,
    );
    assert_eq!(
        id.structure_evolution_mode,
        StructureEvolutionMode::DynamicStructure
    );
    assert_eq!(id.configuration_hash, h_dyn);
}

#[test]
fn incompatible_resume_rejected() {
    assert!(resume_rejects_structure_mode_change(
        StructureEvolutionMode::FixedGeometry,
        StructureEvolutionMode::DynamicStructure
    ));
    assert!(!resume_rejects_structure_mode_change(
        StructureEvolutionMode::DynamicStructure,
        StructureEvolutionMode::DynamicStructure
    ));
}

#[test]
fn structural_update_parity() {
    assert!(structural_update_parity_ok(
        0.4,
        1.0,
        1.2,
        0.8,
        0.0,
        0.0,
        D061_UPDATE_PARITY_TOL
    ));
    assert!(!structural_update_parity_ok(
        0.9,
        1.0,
        1.2,
        0.8,
        0.0,
        0.0,
        D061_UPDATE_PARITY_TOL
    ));
    let ledger = structural_update_ledger(0.4, 1.0, 1.2, 0.8, 0.0, 0.0);
    assert!(ledger.closes(D061_LEDGER_TOL));
}

#[test]
fn corrected_drive_and_runaway_classifiers() {
    let positive: Vec<(f64, f64)> = [4.0, 8.0, 12.0, 16.0]
        .iter()
        .map(|&r| (r, 0.02))
        .collect();
    assert_eq!(
        classify_corrected_drive(&positive, D061_DRIVE_EPS),
        CorrectedDriveClass::PositiveAllRadii
    );

    let negative: Vec<(f64, f64)> = [4.0, 8.0, 12.0]
        .iter()
        .map(|&r| (r, -0.02))
        .collect();
    assert_eq!(
        classify_corrected_drive(&negative, D061_DRIVE_EPS),
        CorrectedDriveClass::NegativeAllRadii
    );

    let restoring = vec![(4.0, 0.05), (8.0, 0.02), (12.0, -0.01), (16.0, -0.04)];
    assert_eq!(
        classify_corrected_drive(&restoring, D061_DRIVE_EPS),
        CorrectedDriveClass::RestoringZeroCrossing
    );

    let neutral: Vec<(f64, f64)> = [4.0, 8.0, 12.0].iter().map(|&r| (r, 0.0)).collect();
    assert_eq!(
        classify_corrected_drive(&neutral, D061_DRIVE_EPS),
        CorrectedDriveClass::NeutralAfterRepair
    );

    assert!(classify_runaway_growth(&[0.1, 0.2, 0.15, 0.05, 0.08], 1e-3));
    assert!(classify_runaway_collapse(&[-0.1, -0.2, -0.15, -0.05, -0.08], 1e-3));
    assert!(!classify_runaway_growth(&[0.1, -0.2, 0.0, -0.05, 0.08], 1e-3));
}

#[test]
fn route_selection_rules() {
    let base = RouteEvidence061 {
        workspace_isolated: true,
        d060_defect_reproduced: true,
        mode_semantics_ok: true,
        mode_implementation_ok: true,
        update_parity_ok: true,
        synthetic_geometry_ok: true,
        fixed_geometry_regression_ok: true,
        causality_ok: true,
        accounting_ok: true,
        numerical_ok: true,
        restoring_basin_qualified: false,
        runaway_growth: true,
        runaway_collapse: false,
        size_restored_metabolism_fail: false,
        no_existing_restoring_basin: false,
    };
    let (route, conclusion) = select_route(base);
    assert_eq!(route, D061Route::G);
    assert_eq!(
        conclusion.as_str(),
        "D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH"
    );

    let mut fail_parity = base;
    fail_parity.runaway_growth = false;
    fail_parity.update_parity_ok = false;
    let (r, c) = select_route(fail_parity);
    assert_eq!(r, D061Route::X);
    assert_eq!(c, D061PrimaryConclusion::StructuralUpdateParityFailure);

    let mut basin = base;
    basin.runaway_growth = false;
    basin.restoring_basin_qualified = true;
    let (r, c) = select_route(basin);
    assert_eq!(r, D061Route::E);
    assert_eq!(
        c,
        D061PrimaryConclusion::ExistingStructuralRestoringBasinQualified
    );
}

#[test]
fn constraint_path_semantics() {
    assert!(constraint_path_semantics_match(true, true));
    assert!(constraint_path_semantics_match(false, false));
    assert!(!constraint_path_semantics_match(true, false));
    assert_eq!(
        classify_constraint_path(false, false),
        PathGeometryClass::FixedGeometry
    );
    assert_eq!(
        classify_constraint_path(true, true),
        PathGeometryClass::DynamicOrganism
    );
}

#[test]
fn fixed_geometry_immobility_vs_dynamic_apply() {
    let mut fixed = Simulation::new(SimParams::default());
    fixed.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    assert!(!fixed.apply_phi_updates());

    let mut dynamic = Simulation::new(SimParams::default());
    dynamic.set_structure_evolution_mode(StructureEvolutionMode::DynamicStructure);
    assert!(dynamic.apply_phi_updates());
}

#[test]
fn execution_repair_disposition_labels() {
    assert_eq!(
        ExecutionRepairDisposition::Qualified.as_str(),
        "D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED"
    );
    assert_eq!(
        ExecutionRepairDisposition::Rejected.as_str(),
        "D061_STRUCTURE_EXECUTION_REPAIR_REJECTED"
    );
}
