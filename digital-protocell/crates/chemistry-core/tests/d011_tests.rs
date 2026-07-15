//! D-011 transport-coupled constrained-radius assay tests.

use chemistry_core::*;

fn d011_params() -> SimParams {
    let mut params = SimParams::default();
    params.equation_version = EquationVersion::MembraneMetabolismV1;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    params.reactions_enabled = true;
    params.reservoir_rate = 1.0;
    STAGE_E_FAILED_RATES.apply_to(&mut params);
    params
}

fn constrained_simulation(radius: f64) -> Simulation {
    let mut sim = Simulation::new(d011_params());
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let x = (idx % sim.grid.width) as f64 - sim.grid.cx;
        let y = (idx / sim.grid.width) as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        sim.fields.membrane[idx] = interface_weight(phi);
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.2;
            sim.fields.nutrient[idx] = 0.2;
            sim.fields.fuel[idx] = 0.2;
            sim.fields.waste[idx] = 0.5;
        } else {
            sim.fields.nutrient[idx] = 0.8;
            sim.fields.fuel[idx] = 0.7;
        }
    }
    sim
}

fn interior_mean(sim: &Simulation, field: &[f64]) -> f64 {
    let mut total = 0.0;
    let mut area = 0.0_f64;
    for (idx, value) in field.iter().enumerate() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            total += value;
            area += 1.0;
        }
    }
    total / area.max(1.0)
}

fn retention(sim: &Simulation, field: &[f64]) -> f64 {
    let mut inside = 0.0;
    for (idx, value) in field.iter().enumerate() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            inside += value;
        }
    }
    inside / field_mass(&sim.grid, field).max(f64::EPSILON)
}

fn run_steps(sim: &mut Simulation, steps: u64) {
    for _ in 0..steps {
        assert!(sim.step());
    }
}

fn window_snapshot(sim: &Simulation, start_step: u64, start_time: f64) -> SteadyWindowSnapshot {
    SteadyWindowSnapshot {
        start_step,
        end_step: sim.substep,
        simulated_time_start: start_time,
        simulated_time_end: sim.sim_time,
        mass_c: field_mass(&sim.grid, &sim.fields.catalyst),
        mass_a: field_mass(&sim.grid, &sim.fields.activated),
        mass_m: field_mass(&sim.grid, &sim.fields.membrane),
        mean_n_interior: interior_mean(sim, &sim.fields.nutrient),
        mean_f_interior: interior_mean(sim, &sim.fields.fuel),
        mean_w_interior: interior_mean(sim, &sim.fields.waste),
        structure_production: sim.constraint_accounting.cumulative.virtual_production,
        structure_decay: sim.constraint_accounting.cumulative.virtual_decay,
        catalyst_reproduction: sim.metabolism_accounting.cumulative.reproduction,
        catalyst_turnover: sim.metabolism_accounting.cumulative.catalyst_turnover,
        membrane_synthesis: sim.membrane_accounting.cumulative.synthesis,
        membrane_loss: sim.membrane_accounting.cumulative.decay
            + sim.membrane_accounting.cumulative.detachment,
        activation: sim.metabolism_accounting.cumulative.activation,
        activated_loss: sim.metabolism_accounting.cumulative.activated_decay
            + sim.constraint_accounting.cumulative.virtual_production,
        nutrient_transport_interior: sim
            .transport_accounting
            .cumulative
            .nutrient
            .interior_net_flux_rate
            * sim.sim_time.max(1.0),
        fuel_transport_interior: sim
            .transport_accounting
            .cumulative
            .fuel
            .interior_net_flux_rate
            * sim.sim_time.max(1.0),
        waste_transport_interior: sim
            .transport_accounting
            .cumulative
            .waste
            .interior_net_flux_rate
            * sim.sim_time.max(1.0),
    }
}

#[test]
fn test_constrained_radius_keeps_phi_fixed() {
    let mut sim = constrained_simulation(24.0);
    let phi_hash = field_sha256_stable(&sim.fields.structure);
    run_steps(&mut sim, 20);
    assert_eq!(field_sha256_stable(&sim.fields.structure), phi_hash);
}

#[test]
fn test_constrained_radius_evolves_all_other_fields() {
    let mut sim = constrained_simulation(24.0);
    let before = (
        field_sha256_stable(&sim.fields.catalyst),
        field_sha256_stable(&sim.fields.nutrient),
        field_sha256_stable(&sim.fields.membrane),
    );
    run_steps(&mut sim, 20);
    let after = (
        field_sha256_stable(&sim.fields.catalyst),
        field_sha256_stable(&sim.fields.nutrient),
        field_sha256_stable(&sim.fields.membrane),
    );
    assert_ne!(before.0, after.0);
    assert_ne!(before.1, after.1);
    assert_ne!(before.2, after.2);
}

#[test]
fn test_virtual_structure_reaction_consumes_a() {
    let mut sim = constrained_simulation(24.0);
    let a_before = field_mass(&sim.grid, &sim.fields.activated);
    run_steps(&mut sim, 50);
    assert!(sim.constraint_accounting.cumulative.virtual_production > 0.0);
    assert!(field_mass(&sim.grid, &sim.fields.activated) < a_before);
}

#[test]
fn test_virtual_structure_reaction_produces_w() {
    let mut sim = constrained_simulation(24.0);
    run_steps(&mut sim, 50);
    assert!(sim.constraint_accounting.cumulative.virtual_decay > 0.0);
    assert!(sim.metabolism_accounting.cumulative.waste_reaction_delta >= 0.0);
}

#[test]
fn test_constraint_flux_has_no_chemical_effect() {
    let mut sim = constrained_simulation(24.0);
    let phi_before = field_sha256_stable(&sim.fields.structure);
    run_steps(&mut sim, 50);
    assert!(sim.constraint_accounting.cumulative.structure_constraint_flux.abs() > 0.0);
    assert_eq!(field_sha256_stable(&sim.fields.structure), phi_before);
}

#[test]
fn test_constrained_radius_uses_old_state_transport() {
    let mut sim = constrained_simulation(24.0);
    let m_before = field_sha256_stable(&sim.fields.membrane);
    assert!(sim.step());
    assert_ne!(field_sha256_stable(&sim.fields.membrane), m_before);
    assert!(sim.transport_accounting.last_step.nutrient.interior_net_flux_rate.is_finite());
}

#[test]
fn test_constrained_radius_swaps_all_dynamic_fields() {
    let mut sim = constrained_simulation(24.0);
    let (current, next) = (
        sim.fields.catalyst.as_ptr(),
        sim.fields.catalyst_next.as_ptr(),
    );
    assert_ne!(current, next);
    assert!(sim.step());
    assert_ne!(sim.fields.catalyst.as_ptr(), current);
}

#[test]
fn test_rejected_constrained_step_swaps_none() {
    let mut sim = constrained_simulation(24.0);
    sim.fields.catalyst[sim.grid.width * sim.grid.height / 2] = 99.0;
    let before = field_sha256_stable(&sim.fields.catalyst);
    let accepted = sim.step();
    assert!(!accepted || field_sha256_stable(&sim.fields.catalyst) != before);
}

#[test]
fn test_quasi_steady_requires_three_windows() {
    let windows = vec![
        SteadyWindowSnapshot {
            start_step: 0,
            end_step: 1000,
            simulated_time_start: 0.0,
            simulated_time_end: 1.0,
            mass_c: 100.0,
            mass_a: 100.0,
            mass_m: 100.0,
            mean_n_interior: 0.2,
            mean_f_interior: 0.2,
            mean_w_interior: 0.5,
            structure_production: 100.0,
            structure_decay: 100.0,
            catalyst_reproduction: 100.0,
            catalyst_turnover: 100.0,
            membrane_synthesis: 100.0,
            membrane_loss: 100.0,
            activation: 100.0,
            activated_loss: 100.0,
            nutrient_transport_interior: 100.0,
            fuel_transport_interior: 100.0,
            waste_transport_interior: -100.0,
        },
        SteadyWindowSnapshot {
            start_step: 1000,
            end_step: 2000,
            simulated_time_start: 1.0,
            simulated_time_end: 2.0,
            mass_c: 100.0,
            mass_a: 100.0,
            mass_m: 100.0,
            mean_n_interior: 0.2,
            mean_f_interior: 0.2,
            mean_w_interior: 0.5,
            structure_production: 101.0,
            structure_decay: 101.0,
            catalyst_reproduction: 101.0,
            catalyst_turnover: 101.0,
            membrane_synthesis: 101.0,
            membrane_loss: 101.0,
            activation: 101.0,
            activated_loss: 101.0,
            nutrient_transport_interior: 101.0,
            fuel_transport_interior: 101.0,
            waste_transport_interior: -101.0,
        },
        SteadyWindowSnapshot {
            start_step: 2000,
            end_step: 3000,
            simulated_time_start: 2.0,
            simulated_time_end: 3.0,
            mass_c: 100.0,
            mass_a: 100.0,
            mass_m: 100.0,
            mean_n_interior: 0.2,
            mean_f_interior: 0.2,
            mean_w_interior: 0.5,
            structure_production: 102.0,
            structure_decay: 102.0,
            catalyst_reproduction: 102.0,
            catalyst_turnover: 102.0,
            membrane_synthesis: 102.0,
            membrane_loss: 102.0,
            activation: 102.0,
            activated_loss: 102.0,
            nutrient_transport_interior: 102.0,
            fuel_transport_interior: 102.0,
            waste_transport_interior: -102.0,
        },
        SteadyWindowSnapshot {
            start_step: 3000,
            end_step: 4000,
            simulated_time_start: 3.0,
            simulated_time_end: 4.0,
            mass_c: 100.0,
            mass_a: 100.0,
            mass_m: 100.0,
            mean_n_interior: 0.2,
            mean_f_interior: 0.2,
            mean_w_interior: 0.5,
            structure_production: 103.0,
            structure_decay: 103.0,
            catalyst_reproduction: 103.0,
            catalyst_turnover: 103.0,
            membrane_synthesis: 103.0,
            membrane_loss: 103.0,
            activation: 103.0,
            activated_loss: 103.0,
            nutrient_transport_interior: 103.0,
            fuel_transport_interior: 103.0,
            waste_transport_interior: -103.0,
        },
    ];
    let report = quasi_steady_report(&windows, D011_TEST_WINDOW, 3);
    assert!(report.converged);
    let short = quasi_steady_report(&windows[..2], D011_TEST_WINDOW, 3);
    assert!(!short.converged);
}

#[test]
fn test_nonconverged_state_cannot_pass() {
    let metrics = JointBalanceMetrics {
        structure: ComponentBalance { q: 1.0, g: 1.0, production: 1.0, loss: 1.0 },
        catalyst: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        membrane: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        activated: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        catalyst_retention: 0.9,
        activated_retention: 0.9,
        membrane_localization: 0.95,
        nutrient_influx: 1.0,
        fuel_influx: 1.0,
        waste_efflux: 1.0,
    };
    assert!(!joint_overlap_pass(&metrics));
    let quasi = QuasiSteadyReport {
        window_size: 1000,
        converged_windows: 0,
        required_windows: 3,
        converged: false,
        window_slopes: Vec::new(),
    };
    let class = classify_convergence(
        &quasi,
        &metrics,
        1.0,
        1.0,
        1.0,
        0.2,
        0.2,
        1.0,
        true,
        0.0,
    );
    assert_eq!(class, ConvergenceClassification::NotConverged);
}

#[test]
fn test_joint_overlap_requires_all_four_components() {
    let mut metrics = JointBalanceMetrics {
        structure: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        catalyst: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        membrane: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        activated: ComponentBalance { q: 1.0, g: 0.0, production: 1.0, loss: 1.0 },
        catalyst_retention: 0.9,
        activated_retention: 0.9,
        membrane_localization: 0.95,
        nutrient_influx: 1.0,
        fuel_influx: 1.0,
        waste_efflux: 1.0,
    };
    assert!(joint_overlap_pass(&metrics));
    metrics.membrane.q = 1.5;
    assert!(!joint_overlap_pass(&metrics));
}

#[test]
fn test_joint_sensitivity_uses_log_rates() {
    let up = log_central_difference(1.05, 0.95);
    assert!(up.is_finite());
    assert!(up.abs() > 0.0);
}

#[test]
fn test_joint_sensitivity_uses_central_difference() {
    let center = log_central_difference(1.05, 0.95);
    let forward = (1.05_f64.ln() - 1.0_f64.ln()) / D011_SENSITIVITY_PERTURB;
    assert!((center - forward).abs() > 0.01);
}

#[test]
fn test_rank_deficient_sensitivity_is_reported() {
    let matrix = [
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 4.0, 6.0, 8.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ];
    let report = sensitivity_matrix(&matrix);
    assert!(report.rank_deficient);
    assert!(report.rank < 4);
}

#[test]
fn test_joint_solver_respects_global_bounds() {
    let reference = STAGE_E_FAILED_RATES;
    let current = reference;
    let g = [1.0, 1.0, 1.0, 1.0];
    let sensitivity = sensitivity_matrix(&[[1.0; 4]; 4]);
    let candidate = solve_bounded_joint_step(&reference, &current, g, &sensitivity, 0).unwrap();
    for (idx, value) in rate_vector(&candidate.rates).iter().enumerate() {
        let ref_v = rate_vector(&reference)[idx];
        assert!(*value >= ref_v * D011_GLOBAL_RATE_MIN_FACTOR);
        assert!(*value <= ref_v * D011_GLOBAL_RATE_MAX_FACTOR);
    }
}

#[test]
fn test_joint_solver_respects_per_round_bounds() {
    let reference = STAGE_E_FAILED_RATES;
    let current = reference;
    let g = [10.0, 10.0, 10.0, 10.0];
    let sensitivity = sensitivity_matrix(&[[10.0; 4]; 4]);
    let candidate = solve_bounded_joint_step(&reference, &current, g, &sensitivity, 0).unwrap();
    for delta in candidate.rate_deltas_log {
        assert!(delta >= D011_ROUND_RATE_MIN_FACTOR.ln());
        assert!(delta <= D011_ROUND_RATE_MAX_FACTOR.ln());
    }
}

#[test]
fn test_joint_solver_candidate_count_is_bounded() {
    let report = bounded_joint_solver(
        &STAGE_E_FAILED_RATES,
        &STAGE_E_FAILED_RATES,
        &[[1.0; 4]; 4],
        &[],
    );
    assert!(report.candidates.len() <= D011_MAX_CANDIDATES);
}

#[test]
fn test_dynamic_field_accounting_closes() {
    let mut sim = constrained_simulation(24.0);
    run_steps(&mut sim, 100);
    assert!(sim.accounting.cumulative_within_tolerance());
}

#[test]
fn test_structure_constraint_accounting_closes() {
    let mut sim = constrained_simulation(24.0);
    run_steps(&mut sim, 100);
    assert!(sim.constraint_accounting.closes());
}

#[test]
fn test_failed_stage_e_evidence_is_preserved() {
    let artifact = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../experiments/generated/d008/stage_e_balance/attempt_003/result.json"
    ));
    let parsed: serde_json::Value = serde_json::from_str(artifact).expect("artifact json");
    let selected = &parsed["selected_rates"];
    assert!(
        (selected["k_membrane"].as_f64().unwrap() - STAGE_E_FAILED_RATES.k_membrane).abs()
            < 1e-10
    );
    assert_eq!(
        selected["k_d008_structure"].as_f64().unwrap(),
        STAGE_E_FAILED_RATES.k_d008_structure
    );
    assert_eq!(
        parsed["scientific_conclusion"].as_str().unwrap(),
        "D008_NO_JOINT_FIXED_POINT"
    );
}

#[test]
fn constrained_radius_collects_window_snapshots() {
    let mut sim = constrained_simulation(22.0);
    let mut windows = Vec::new();
    let mut start_step = 0;
    let mut start_time = 0.0;
    for _ in 0..3 {
        run_steps(&mut sim, D011_TEST_WINDOW);
        windows.push(window_snapshot(&sim, start_step, start_time));
        start_step = sim.substep;
        start_time = sim.sim_time;
    }
    let report = quasi_steady_report(&windows, D011_TEST_WINDOW, 3);
    assert!(report.window_slopes.len() >= 2);
    let metrics = build_balance_metrics(
        sim.sim_time,
        &sim.constraint_accounting.cumulative,
        &sim.metabolism_accounting.cumulative,
        &sim.membrane_accounting.cumulative,
        &sim.transport_accounting.cumulative,
        retention(&sim, &sim.fields.catalyst),
        retention(&sim, &sim.fields.activated),
        membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane)
            .localization_fraction,
    );
    assert!(metrics.structure.production.is_finite());
}
