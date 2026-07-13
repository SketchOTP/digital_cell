//! D-003 validation tests.

use chemistry_core::*;

#[test]
fn test_simulated_time_accumulates_accepted_dt() {
    let mut sim = Simulation::new(baseline_params());
    sim.observer_enabled = false;
    sim.run_substeps(100, 0);
    let summary = sim.dt_telemetry.summary();
    assert!((summary.accepted_simulated_time - sim.sim_time).abs() < 1e-9);
    assert_eq!(summary.accepted_substeps, 100);
}

#[test]
fn test_reaction_ledgers_scale_with_simulated_time() {
    let params = baseline_params();
    let mut totals_half = 0.0;
    let mut phi = 0.8;
    let mut c = 0.3;
    let mut n = 1.0;
    let mut f = 1.0;
    let mut w = 0.0;
    for _ in 0..200 {
        let r = compute_reactions_at(phi, c, n, f, w, &params, true);
        totals_half += r.r_structure_decay * 0.005;
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.005, &params);
    }
    let mut totals_quarter = 0.0;
    phi = 0.8;
    c = 0.3;
    n = 1.0;
    f = 1.0;
    w = 0.0;
    for _ in 0..400 {
        let r = compute_reactions_at(phi, c, n, f, w, &params, true);
        totals_quarter += r.r_structure_decay * 0.0025;
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.0025, &params);
    }
    assert!(
        (totals_half - totals_quarter).abs() / totals_half.max(1e-12) < 0.05,
        "{totals_half} vs {totals_quarter}"
    );
}

#[test]
fn test_equal_simulated_time_converges_across_timesteps() {
    let params = baseline_params();
    let target_time = 1.0;
    let integrate = |dt: f64, steps: u64| -> f64 {
        let mut phi = 0.8;
        let mut c = 0.3;
        let mut n = 1.0;
        let mut f = 1.0;
        let mut w = 0.0;
        let mut decay = 0.0;
        for _ in 0..steps {
            let r = compute_reactions_at(phi, c, n, f, w, &params, true);
            decay += r.r_structure_decay * dt;
            reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, dt, &params);
        }
        decay
    };
    let d1 = integrate(0.01, (target_time / 0.01) as u64);
    let d2 = integrate(0.005, (target_time / 0.005) as u64);
    let d4 = integrate(0.0025, (target_time / 0.0025) as u64);
    assert!((d1 - d2).abs() / d1.max(1e-12) < 0.08);
    assert!((d1 - d4).abs() / d1.max(1e-12) < 0.12);
}

#[test]
fn test_seed_changes_noise_not_macrostate() {
    let grid = Grid::new();
    let mut r1 = None;
    let mut r2 = None;
    for seed in [1u64, 2] {
        let mut p = baseline_params();
        p.random_seed = seed;
        let mut fields = FieldBuffers::for_grid(&grid);
        initialize_seed(&grid, &p, &mut fields);
        let audit = audit_initial_seed(&grid, &fields, seed, p.seed_r0);
        if seed == 1 {
            r1 = Some(audit);
        } else {
            r2 = Some(audit);
        }
    }
    let a = r1.unwrap();
    let b = r2.unwrap();
    assert!((a.structural_mass - b.structural_mass).abs() / a.structural_mass < 0.02);
    assert_ne!(a.structure_hash, b.structure_hash);
}

#[test]
fn test_seed_one_has_no_special_execution_path() {
    let mut sim1 = Simulation::from_config(&ExperimentConfig {
        name: "s1".into(),
        seed: 1,
        substeps: 100,
        params: baseline_params(),
        interventions: vec![],
        record_every: 0,
    });
    let mut sim2 = Simulation::from_config(&ExperimentConfig {
        name: "s2".into(),
        seed: 2,
        substeps: 100,
        params: baseline_params(),
        interventions: vec![],
        record_every: 0,
    });
    sim1.observer_enabled = false;
    sim2.observer_enabled = false;
    sim1.run_substeps(100, 0);
    sim2.run_substeps(100, 0);
    assert_eq!(sim1.substep, sim2.substep);
    assert_ne!(sim1.field_hash(), sim2.field_hash());
}

#[test]
fn test_initial_noise_respects_configured_amplitude() {
    let grid = Grid::new();
    let mut p = baseline_params();
    p.noise_amplitude = 0.005;
    p.random_seed = 99;
    let mut fields = FieldBuffers::for_grid(&grid);
    initialize_seed(&grid, &p, &mut fields);
    let audit = audit_initial_seed(&grid, &fields, 99, p.seed_r0);
    assert!(audit.max_perturbation <= p.noise_amplitude + 1e-9);
    assert!(audit.min_perturbation >= -p.noise_amplitude - 1e-9);
}

#[test]
fn test_structure_crowding_function_is_finite() {
    for phi in [-0.1, 0.0, 0.5, 1.0, 1.25, 2.0] {
        let g = structure_crowding(phi, 1.0);
        assert!(g.is_finite() && g > 0.0);
    }
}

#[test]
fn test_structure_crowding_function_is_monotonic() {
    let mut prev = structure_crowding(0.0, 1.0);
    for i in 1..=100 {
        let phi = i as f64 / 100.0 * 2.0;
        let g = structure_crowding(phi, 1.0);
        assert!(g <= prev + 1e-12, "not monotonic at {phi}: {g} > {prev}");
        prev = g;
    }
}

#[test]
fn test_structure_synthesis_remains_nonzero_at_phi_one() {
    let params = baseline_params();
    let r = compute_reactions_at(1.0, 0.3, 1.0, 1.0, 0.0, &params, true);
    assert!(r.r_structure > 0.0, "synthesis zero at phi=1");
    assert!((structure_crowding(1.0, params.k_phi) - 0.5).abs() < 1e-12);
}

#[test]
fn test_structure_synthesis_requires_catalyst() {
    let params = baseline_params();
    let r = compute_reactions_at(0.5, 0.0, 1.0, 1.0, 0.0, &params, true);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_structure_synthesis_requires_nutrient() {
    let params = baseline_params();
    let r = compute_reactions_at(0.5, 0.3, 0.0, 1.0, 0.0, &params, true);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_structure_synthesis_requires_fuel() {
    let params = baseline_params();
    let r = compute_reactions_at(0.5, 0.3, 1.0, 0.0, 0.0, &params, true);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_balance_ratio_detects_decline() {
    let samples = vec![
        BalanceWindowSample {
            sim_time: 0.0,
            m_phi: 100.0,
            m_c: 10.0,
            s_phi: 1.0,
            d_phi: 2.0,
            r_c: 0.1,
            d_c: 0.2,
        },
        BalanceWindowSample {
            sim_time: 1.0,
            m_phi: 90.0,
            m_c: 9.0,
            s_phi: 1.0,
            d_phi: 2.0,
            r_c: 0.1,
            d_c: 0.2,
        },
    ];
    let b = compute_balance(&samples);
    assert!(b.q_phi < 1.0);
    assert!(b.slope_phi < 0.0);
}

#[test]
fn test_balance_ratio_detects_growth() {
    let samples = vec![
        BalanceWindowSample {
            sim_time: 0.0,
            m_phi: 100.0,
            m_c: 10.0,
            s_phi: 2.0,
            d_phi: 1.0,
            r_c: 0.2,
            d_c: 0.1,
        },
        BalanceWindowSample {
            sim_time: 1.0,
            m_phi: 110.0,
            m_c: 11.0,
            s_phi: 2.0,
            d_phi: 1.0,
            r_c: 0.2,
            d_c: 0.1,
        },
    ];
    let b = compute_balance(&samples);
    assert!(b.q_phi > 1.0);
    assert!(b.slope_phi > 0.0);
}

#[test]
fn test_balance_ratio_detects_dynamic_equilibrium() {
    let samples = vec![
        BalanceWindowSample {
            sim_time: 0.0,
            m_phi: 100.0,
            m_c: 10.0,
            s_phi: 1.0,
            d_phi: 1.0,
            r_c: 0.1,
            d_c: 0.1,
        },
        BalanceWindowSample {
            sim_time: 1.0,
            m_phi: 100.5,
            m_c: 10.01,
            s_phi: 1.0,
            d_phi: 1.0,
            r_c: 0.1,
            d_c: 0.1,
        },
    ];
    let b = compute_balance(&samples);
    assert!((b.q_phi - 1.0).abs() < 0.01);
    assert!((b.q_c - 1.0).abs() < 0.01);
}

#[test]
fn test_bottleneck_diagnostics_partition_reactions() {
    let mut sim = Simulation::new(baseline_params());
    sim.step();
    let bn = compute_bottleneck(
        &sim.grid,
        &sim.fields.structure,
        &sim.fields.catalyst,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.reaction_scratch,
        sim.params.c_max,
        sim.params.n_reservoir,
        sim.params.f_reservoir,
    );
    let total_s = bn.synth_dense + bn.synth_interface + bn.synth_exterior;
    assert!(total_s > 0.0);
    let total_d = bn.decay_dense + bn.decay_interface + bn.decay_exterior;
    assert!(total_d > 0.0);
}

#[test]
fn test_transport_limitation_classification() {
    assert!(is_transport_limited(0.05, 0.05, 1.0, 1.0));
    assert!(!is_transport_limited(0.5, 0.5, 1.0, 1.0));
}

#[test]
fn test_retention_limitation_classification() {
    assert!(is_retention_limited(0.30, 0.0));
    assert!(!is_retention_limited(0.10, 0.20));
}

#[test]
fn test_catalyst_flux_accounting() {
    let mut sim = Simulation::new(baseline_params());
    sim.observer_enabled = false;
    sim.run_substeps(500, 0);
    compute_all_reactions(
        &sim.fields.structure,
        &sim.fields.catalyst,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.fields.waste,
        &sim.params,
        true,
        &mut sim.reaction_scratch,
    );
    let bn = compute_bottleneck(
        &sim.grid,
        &sim.fields.structure,
        &sim.fields.catalyst,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.reaction_scratch,
        sim.params.c_max,
        sim.params.n_reservoir,
        sim.params.f_reservoir,
    );
    assert!(bn.fraction_catalyst_outside >= 0.0 && bn.fraction_catalyst_outside <= 1.0);
    let total_decay = bn.catalyst_decay_inside + bn.catalyst_decay_interface + bn.catalyst_decay_outside;
    assert!(total_decay > 0.0);
}

#[test]
fn test_original_kinetics_reproduce_d002_decline() {
    let mut legacy = Simulation::new(legacy_d002_params());
    let mut crowding = Simulation::new(baseline_params());
    legacy.observer_enabled = false;
    crowding.observer_enabled = false;
    legacy.run_substeps(5000, 0);
    crowding.run_substeps(5000, 0);
    assert_eq!(legacy.substep, 5000);
    assert_eq!(crowding.substep, 5000);
    let s_legacy = legacy.accounting.cumulative.structural_synthesis;
    let s_crowd = crowding.accounting.cumulative.structural_synthesis;
    assert!(s_crowd > s_legacy, "crowding should increase synthesis: {s_crowd} vs {s_legacy}");
}
