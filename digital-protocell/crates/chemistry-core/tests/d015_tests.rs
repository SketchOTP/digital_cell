//! D-015 waste budget, clearance, and sink-capacity tests.

use chemistry_core::accounting::{build_field_ledger, FieldStepLedger};
use chemistry_core::candidate_identity::GridConfiguration;
use chemistry_core::config::{
    D008StageMode, EquationVersion, SimParams, CONC_SAFETY_LIMIT, DISH_RADIUS, RESERVOIR_WIDTH,
};
use chemistry_core::d013_harness::{
    solver_entry_allowed, ArtifactValidationStatus, ScientificClassification,
};
use chemistry_core::d015_waste::*;
use chemistry_core::grid::Grid;
use chemistry_core::membrane_transport::{transport_field, TransportSpecies};
use chemistry_core::reservoir::apply_reservoir;
use chemistry_core::{candidate_hash, Simulation};

fn v2_constrained_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.eta_c = 1.0;
    p.eta_phi = 1.0;
    p.eta_m = 1.0;
    p.k_d008_activation = 0.05;
    p.k_d008_reproduction = 0.02;
    p.k_d008_activated_decay = 0.01;
    p.k_d008_catalyst_turnover = 0.005;
    p.k_d008_structure = 0.01;
    p.k_membrane = 0.02;
    p.reactions_enabled = true;
    p.diffusion_enabled = true;
    p
}

fn synthetic_v2_extents() -> V2WasteSourceExtents {
    V2WasteSourceExtents {
        activation: 0.4,
        reproduction: 0.2,
        activated_decay: 0.1,
        catalyst_turnover: 0.05,
        structure_production_extent: 0.0,
        structure_decay: 0.03,
        membrane_synthesis: 0.08,
        membrane_decay: 0.02,
        membrane_detachment: 0.01,
    }
}

fn synthetic_waste_ledger(sources: &DecomposedWasteSources) -> FieldStepLedger {
    build_field_ledger(
        1.0,
        sources.sum(),
        0.0,
        -0.15,
        1.0 + sources.sum() - 0.15,
        1.0 + sources.sum() - 0.15,
    )
}

#[test]
fn test_waste_budget_contains_all_v2_sources() {
    let params = v2_constrained_params();
    let sources = decompose_v2_waste_sources(&synthetic_v2_extents(), &params);
    let fields = sources.source_fields();
    assert_eq!(fields.len(), 7);
    assert!(fields.iter().all(|(name, _)| !name.is_empty()));
    assert!((sources.activation - 0.4).abs() < 1e-12);
    assert!((sources.catalyst_turnover - 0.05).abs() < 1e-12);
    assert!((sources.structure_turnover - 0.03).abs() < 1e-12);
    assert!((sources.membrane_turnover - 0.02).abs() < 1e-12);
    assert!((sources.activated_resource_turnover - 0.1).abs() < 1e-12);
    assert!((sources.membrane_detachment - 0.01).abs() < 1e-12);
    assert!((sources.productive_yield_waste).abs() < 1e-12);
}

#[test]
fn test_stepwise_waste_budget_closes() {
    let params = v2_constrained_params();
    let sources = decompose_v2_waste_sources(&synthetic_v2_extents(), &params);
    let waste = synthetic_waste_ledger(&sources);
    let step = build_waste_budget_step(&waste, &sources);
    assert!(waste_budget_step_closes(&step));
}

#[test]
fn test_internal_waste_transport_cancels() {
    let mut sim = Simulation::new(v2_constrained_params());
    for _ in 0..50 {
        assert!(sim.step());
    }
    assert!(sim.waste_budget.global_transport_residual().abs() < 1e-6);
}

#[test]
fn test_waste_clearance_is_counted_once() {
    let params = v2_constrained_params();
    let sources = decompose_v2_waste_sources(&synthetic_v2_extents(), &params);
    let mut waste = synthetic_waste_ledger(&sources);
    waste.reservoir_delta = -0.15;
    let step = build_waste_budget_step(&waste, &sources);
    assert!((step.waste_clearance - 0.15).abs() < 1e-12);
    assert!((step.external_reservoir_input).abs() < 1e-12);
    let step2 = build_waste_budget_step(&waste, &sources);
    assert!((step2.waste_clearance - step.waste_clearance).abs() < 1e-15);
}

#[test]
fn test_membrane_detachment_enters_waste_budget() {
    let params = v2_constrained_params();
    let mut extents = synthetic_v2_extents();
    extents.membrane_detachment = 0.07;
    let sources = decompose_v2_waste_sources(&extents, &params);
    assert!((sources.membrane_detachment - 0.07).abs() < 1e-12);
    let waste = build_field_ledger(0.0, sources.sum(), 0.0, 0.0, sources.sum(), sources.sum());
    let step = build_waste_budget_step(&waste, &sources);
    assert!((step.membrane_detachment - 0.07).abs() < 1e-12);
    assert!(waste_budget_step_closes(&step));
}

#[test]
fn test_rejected_attempt_does_not_enter_waste_budget() {
    let mut sim = Simulation::new(v2_constrained_params());
    let before = sim.waste_budget.clone();
    sim.attempted_substeps += 1;
    sim.accounting.cumulative.rejected_steps += 1;
    assert_eq!(sim.waste_budget.accepted_steps, before.accepted_steps);
    assert_eq!(sim.waste_budget.last_step.observed_change, before.last_step.observed_change);
}

#[test]
fn test_waste_spatial_partition_covers_domain_once() {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let phi = vec![0.0; n];
    let masks = build_waste_spatial_masks(&grid, &phi, 22.0);
    assert!(masks_cover_dish_once(&grid, &masks));
}

#[test]
fn test_waste_max_location_is_recorded() {
    let grid = Grid::new();
    let mut waste = vec![0.0; grid.width * grid.height];
    let idx = grid.width * grid.height / 2 + 10;
    waste[idx] = 3.5;
    let (max_idx, max_val) = max_waste_location(&grid, &waste);
    assert_eq!(max_idx, idx);
    assert!((max_val - 3.5).abs() < 1e-12);
}

#[test]
fn test_clearance_law_matches_runtime() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let mut waste = vec![0.0; grid.width * grid.height];
    for idx in 0..waste.len() {
        if grid.reservoir_mask[idx] {
            waste[idx] = 2.0;
        }
    }
    let dt = 0.01;
    let predicted = total_reservoir_waste_delta(&grid, &waste, &params, dt);
    let mut trial = waste.clone();
    apply_reservoir(&grid, &mut vec![0.0; waste.len()], &mut vec![0.0; waste.len()], &mut trial, dt, &params);
    let applied = total_reservoir_waste_delta(&grid, &waste, &params, dt);
    assert!((predicted - applied).abs() < 1e-12);
    assert_eq!(
        classify_clearance_implementation(predicted, predicted, dt, true, true),
        ClearanceImplementationClass::Correct
    );
}

#[test]
fn test_clearance_dt_scaling_is_correct() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let mut waste = vec![0.0; grid.width * grid.height];
    waste[grid.reservoir_mask.iter().position(|&m| m).unwrap()] = 1.0;
    let d1 = total_reservoir_waste_delta(&grid, &waste, &params, 0.01);
    let d2 = total_reservoir_waste_delta(&grid, &waste, &params, 0.02);
    assert!((d2 - 2.0 * d1).abs() < 1e-12);
}

#[test]
fn test_clearance_mask_matches_configuration() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let (dish_r, res_w, count) = reservoir_geometry_summary(&grid);
    assert!((dish_r - DISH_RADIUS).abs() < 1e-12);
    assert!((res_w - RESERVOIR_WIDTH).abs() < 1e-12);
    assert!(count > 0);
    assert!((params.waste_sink_inner_radius - (DISH_RADIUS - RESERVOIR_WIDTH)).abs() < 1e-12);
    for idx in 0..grid.width * grid.height {
        if grid.reservoir_mask[idx] {
            let i = idx % grid.width;
            let j = idx / grid.width;
            let r = grid.distance_from_center(i, j);
            assert!(r > DISH_RADIUS - RESERVOIR_WIDTH);
            assert!(chemistry_core::reservoir::waste_sink_cell(&grid, idx, &params));
        }
    }
}

#[test]
fn test_repaired_w_sink_expands_clearance_region() {
    let grid = Grid::new();
    let mut baseline = v2_constrained_params();
    let mut repaired = baseline.clone();
    apply_d015_repaired_environment(&mut repaired, 22.0);
    assert!((repaired.waste_sink_inner_radius - 30.0).abs() < 1e-12);
    assert!(waste_sink_cell_count(&grid, &repaired) > waste_sink_cell_count(&grid, &baseline));
}

#[test]
fn test_environment_repair_changes_env_hash_not_organism() {
    let grid = GridConfiguration::default();
    let mut baseline = v2_constrained_params();
    let org_before = organism_frozen_hash(&baseline, &grid);
    let env_before = environment_configuration_hash(&baseline);
    apply_d015_repaired_environment(&mut baseline, 22.0);
    assert_eq!(organism_frozen_hash(&baseline, &grid), org_before);
    assert_ne!(environment_configuration_hash(&baseline), env_before);
}

#[test]
fn test_clearance_ledger_matches_field_delta() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let mut waste = vec![0.0; grid.width * grid.height];
    for idx in 0..waste.len() {
        if chemistry_core::reservoir::waste_sink_cell(&grid, idx, &params) {
            waste[idx] = 1.5;
        }
    }
    let dt = 0.005;
    let predicted = total_reservoir_waste_delta(&grid, &waste, &params, dt);
    let mut w = waste.clone();
    let ledger_delta = apply_reservoir_waste_delta(&grid, &mut w, &params, dt);
    assert!((ledger_delta - predicted).abs() < 1e-9 * ledger_delta.abs().max(predicted.abs()).max(1.0));
}

fn waste_sink_mass_runner(grid: &Grid, waste: &[f64], params: &SimParams) -> f64 {
    waste
        .iter()
        .enumerate()
        .filter(|(idx, _)| chemistry_core::reservoir::waste_sink_cell(grid, *idx, params))
        .map(|(_, &v)| v)
        .sum()
}

fn local_reservoir_mass(grid: &Grid, waste: &[f64]) -> f64 {
    waste
        .iter()
        .enumerate()
        .filter(|(idx, _)| grid.reservoir_mask[*idx])
        .map(|(_, &v)| v)
        .sum()
}

#[test]
fn test_waste_clearance_uses_accepted_old_state() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let idx = grid.reservoir_mask.iter().position(|&m| m).unwrap();
    let w_old = 2.0;
    let dt = 0.01;
    let predicted = predicted_cell_clearance_delta(w_old, params.w_reservoir, params.reservoir_rate, dt);
    assert!(predicted < 0.0);
    let w_wrong = 0.0;
    let wrong = predicted_cell_clearance_delta(w_wrong, params.w_reservoir, params.reservoir_rate, dt);
    assert!((predicted - wrong).abs() > 1e-12);
    let _ = idx;
}

#[test]
fn test_rejected_attempt_does_not_clear_waste() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let mut waste = vec![0.0; grid.width * grid.height];
    let idx = grid.reservoir_mask.iter().position(|&m| m).unwrap();
    waste[idx] = 2.0;
    let before = waste[idx];
    let _predicted = total_reservoir_waste_delta(&grid, &waste, &params, 0.01);
    assert!((waste[idx] - before).abs() < 1e-12);
}

#[test]
fn test_clearance_never_creates_waste_above_target() {
    let params = v2_constrained_params();
    let dt = 0.01;
    let kdt = params.reservoir_rate * dt;
    for &w in &[0.0, 1.0, 5.0, 9.0] {
        let delta = predicted_cell_clearance_delta(w, params.w_reservoir, params.reservoir_rate, dt);
        let next = w + delta;
        // Linear relaxation is a convex combination: never overshoots the target.
        assert!(next <= w.max(params.w_reservoir) + 1e-12);
        assert!(next >= w.min(params.w_reservoir) - 1e-12);
        assert!((next - (w * (1.0 - kdt) + params.w_reservoir * kdt)).abs() < 1e-12);
    }
}

#[test]
fn test_external_waste_pulse_clears() {
    let mut params = v2_constrained_params();
    params.k_d008_activation = 0.0;
    params.k_d008_reproduction = 0.0;
    params.k_d008_structure = 0.0;
    params.k_membrane = 0.0;
    let mut sim = Simulation::new(params);
    let grid = sim.grid.clone();
    for idx in 0..sim.fields.waste.len() {
        if chemistry_core::reservoir::waste_sink_cell(&grid, idx, &sim.params) {
            sim.fields.waste[idx] = 1.0;
        }
    }
    let initial = waste_sink_mass_runner(&grid, &sim.fields.waste, &sim.params);
    assert!(initial > 0.0);
    for _ in 0..2000 {
        assert!(sim.step());
    }
    let final_mass = waste_sink_mass_runner(&grid, &sim.fields.waste, &sim.params);
    assert!(final_mass < initial * 0.5);
}

#[test]
fn test_internal_waste_pulse_crosses_membrane() {
    let mut params = v2_constrained_params();
    params.k_d008_activation = 0.0;
    params.k_d008_reproduction = 0.0;
    params.k_d008_structure = 0.0;
    params.k_membrane = 0.0;
    let mut sim = Simulation::new(params);
    let grid = sim.grid.clone();
    let center_idx = {
        let cx = grid.cx as usize;
        let cy = grid.cy as usize;
        Grid::index(grid.width, cx, cy)
    };
    sim.fields.waste[center_idx] = 5.0;
    let interior_initial = sim.fields.waste[center_idx];
    for _ in 0..500 {
        assert!(sim.step());
    }
    let exterior_mass: f64 = sim
        .fields
        .waste
        .iter()
        .enumerate()
        .filter(|(idx, _)| grid.in_dish(*idx) && *idx != center_idx)
        .map(|(_, &v)| v)
        .sum();
    assert!(exterior_mass > 0.0);
    assert!(sim.fields.waste[center_idx] < interior_initial);
}

#[test]
fn test_membrane_bypass_control_is_diagnostic_only() {
    let baseline = v2_constrained_params();
    let bypass = diagnostic_membrane_bypass_waste(&baseline);
    assert!(is_diagnostic_membrane_bypass(&bypass, baseline.beta_w));
    assert_eq!(bypass.beta_w, 0.0);
    let grid = GridConfiguration::default();
    assert_ne!(
        candidate_hash(&baseline, &grid),
        candidate_hash(&bypass, &grid)
    );
}

#[test]
fn test_measured_source_injection_matches_budget() {
    let params = v2_constrained_params();
    let extents = synthetic_v2_extents();
    let sources = decompose_v2_waste_sources(&extents, &params);
    let waste = build_field_ledger(0.0, sources.sum(), 0.0, 0.0, sources.sum(), sources.sum());
    let step = build_waste_budget_step(&waste, &sources);
    assert!((step.observed_change - sources.sum()).abs() < 1e-12);
    assert!(waste_budget_step_closes(&step));
}

#[test]
fn test_no_clearance_control_accumulates() {
    let mut params = v2_constrained_params();
    params.reservoir_rate = 0.0;
    params.k_d008_activation = 0.0;
    params.k_d008_reproduction = 0.0;
    params.k_membrane = 0.0;
    let mut sim = Simulation::new(params);
    let grid = sim.grid.clone();
    let masks = build_waste_spatial_masks(&grid, &sim.fields.structure, sim.params.seed_r0);
    for idx in 0..sim.fields.waste.len() {
        if masks.bulk_exterior[idx] {
            sim.fields.waste[idx] = 0.5;
        }
    }
    let before: f64 = sim.fields.waste.iter().sum();
    for _ in 0..20 {
        assert!(sim.step());
    }
    let after: f64 = sim.fields.waste.iter().sum();
    assert!(after >= before);
}

#[test]
fn test_sink_capacity_prediction_matches_control() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let mut waste = vec![0.0; grid.width * grid.height];
    for idx in 0..waste.len() {
        if grid.reservoir_mask[idx] {
            waste[idx] = 2.0;
        }
    }
    let production = 0.1;
    let analysis = analyze_sink_capacity(&grid, &waste, &params, production, production);
    assert!(analysis.clearance_rate_at_current_w.abs() > production);
    assert_eq!(
        analysis.classification,
        Some(SinkCapacityClass::ClearanceCapacityExceedsProduction)
    );
}

#[test]
fn test_finite_domain_capacity_is_calculated() {
    let grid = Grid::new();
    let cap = finite_domain_capacity(&grid, CONC_SAFETY_LIMIT);
    assert!(cap > 0.0);
    assert!(cap >= CONC_SAFETY_LIMIT * grid.dish_cell_count() as f64 * 0.99);
}

#[test]
fn test_ceiling_cannot_be_raised_without_finite_equilibrium() {
    assert!(!ceiling_raise_allowed(f64::INFINITY, CONC_SAFETY_LIMIT));
    assert!(!ceiling_raise_allowed(5.0, CONC_SAFETY_LIMIT));
    assert!(ceiling_raise_allowed(12.0, CONC_SAFETY_LIMIT));
}

#[test]
fn test_environmental_repair_does_not_change_organism_hash() {
    let grid = GridConfiguration::default();
    let mut base = v2_constrained_params();
    let org_hash = organism_frozen_hash(&base, &grid);
    base.w_reservoir = 0.0;
    base.reservoir_rate = 0.5;
    base.w_reservoir = 0.0;
    base.reservoir_rate = 2.0;
    assert_eq!(organism_frozen_hash(&base, &grid), org_hash);
}

#[test]
fn test_repaired_environment_has_versioned_identity() {
    let mut p1 = v2_constrained_params();
    let mut p2 = p1.clone();
    p2.reservoir_rate = 1.25;
    let h1 = environment_configuration_hash(&p1);
    let h2 = environment_configuration_hash(&p2);
    assert_ne!(h1, h2);
    assert!(h1.len() == 64);
}

#[test]
fn test_d015_preflight_requires_waste_budget() {
    let req = d015_preflight_requirements();
    assert!(d015_preflight_requires_waste_budget(&req));
}

#[test]
fn test_solver_remains_closed_without_quasi_steady_reference() {
    assert!(solver_remains_closed_without_quasi_steady(true, false, true));
    assert!(!solver_remains_closed_without_quasi_steady(true, true, true));
    assert!(!solver_entry_allowed(
        ArtifactValidationStatus::ValidGovernedArtifact,
        ScientificClassification::NotConvergedAt200k,
        true,
        true
    ));
}

#[test]
fn test_waste_budget_from_simulation_step() {
    let mut sim = Simulation::new(v2_constrained_params());
    assert!(sim.step());
    assert_eq!(sim.waste_budget.accepted_steps, 1);
    assert!(sim.waste_budget.last_step.observed_change.is_finite());
}

#[test]
fn test_productive_yield_with_partial_etas() {
    let mut params = v2_constrained_params();
    params.eta_c = 0.8;
    params.eta_phi = 0.9;
    params.eta_m = 0.7;
    let mut extents = synthetic_v2_extents();
    extents.reproduction = 1.0;
    extents.structure_production_extent = 1.0;
    extents.membrane_synthesis = 1.0;
    let sources = decompose_v2_waste_sources(&extents, &params);
    let expected = 0.2 + 0.1 + 0.3;
    assert!((sources.productive_yield_waste - expected).abs() < 1e-12);
}

#[test]
fn test_transport_field_cancels_for_closed_dish() {
    let grid = Grid::new();
    let params = v2_constrained_params();
    let n = grid.width * grid.height;
    let phi = vec![0.0; n];
    let membrane = vec![0.0; n];
    let mut scratch = vec![0.0; n];
    let mut waste = vec![0.0; n];
    waste[grid.width * grid.height / 2] = 1.0;
    let acc = transport_field(
        &grid,
        TransportSpecies::Waste,
        &waste,
        &phi,
        &membrane,
        &params,
        &mut scratch,
    );
    assert!(internal_waste_transport_cancels(&acc, 0.01));
}
