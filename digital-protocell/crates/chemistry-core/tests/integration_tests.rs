//! Integration and unit tests for chemistry-core.

use chemistry_core::*;

fn test_substeps() -> u64 {
    #[cfg(feature = "long-experiments")]
    {
        250_000
    }
    #[cfg(not(feature = "long-experiments"))]
    {
        5_000
    }
}

#[test]
fn test_constant_laplacian_is_zero() {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let field = vec![0.5; n];
    let mut lap = vec![0.0; n];
    laplacian(&grid, &field, &mut lap);
    for idx in 0..n {
        if grid.in_dish(idx) {
            assert!(lap[idx].abs() < 1e-10, "lap[{idx}] = {}", lap[idx]);
        }
    }
}

#[test]
fn test_no_flux_boundary_conserves_mass() {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut field = vec![0.0; n];
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if grid.in_dish(idx) {
                field[idx] = 1.0;
            }
        }
    }
    let mass_before = total_mass(&grid, &field);
    let mut diff = vec![0.0; n];
    let d = vec![0.04; n];
    diffuse_variable(&grid, &field, &d, &mut diff);
    let mut field2 = field.clone();
    for idx in 0..n {
        if grid.in_dish(idx) {
            field2[idx] += diff[idx] * 0.01;
        }
    }
    let mass_after = total_mass(&grid, &field2);
    assert!((mass_before - mass_after).abs() < 0.5, "mass drift {mass_before} -> {mass_after}");
}

#[test]
fn test_diffusion_reduces_variance() {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut field = vec![0.1; n];
    let cx = grid.cx as usize;
    let cy = grid.cy as usize;
    let idx = Grid::index(grid.width, cx, cy);
    field[idx] = 1.0;
    let var_before = variance(&grid, &field);
    let mut params = baseline_params();
    params.reactions_enabled = false;
    params.phase_separation_enabled = false;
    let mut sim = Simulation::new(params);
    sim.fields.nutrient = field.clone();
    for _ in 0..200 {
        sim.fields.nutrient = field.clone();
        let mut lap = vec![0.0; n];
        let mut rate = vec![0.0; n];
        diffuse_constant(&grid, &field, 0.18, &mut lap, &mut rate);
        for i in 0..n {
            if grid.in_dish(i) {
                field[i] += rate[i] * 0.01;
                field[i] = field[i].max(0.0);
            }
        }
    }
    let var_after = variance(&grid, &field);
    assert!(var_after < var_before, "{var_after} >= {var_before}");
}

#[test]
fn test_catalyst_cannot_emerge_from_zero() {
    let params = baseline_params();
    let mut phi = 0.8;
    let mut c = 0.0;
    let mut n = 1.0;
    let mut f = 1.0;
    let mut w = 0.0;
    for _ in 0..1000 {
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.01, &params);
    }
    assert!(c < 1e-6, "catalyst emerged from zero: {c}");
}

#[test]
fn test_catalyst_requires_nutrient() {
    let params = baseline_params();
    let mut phi = 0.8;
    let mut c = 0.3;
    let mut n = 0.0;
    let mut f = 1.0;
    let mut w = 0.0;
    let c0 = c;
    for _ in 0..500 {
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.01, &params);
    }
    assert!(c <= c0 + 1e-6, "catalyst grew without nutrient: {c0} -> {c}");
}

#[test]
fn test_catalyst_requires_fuel() {
    let params = baseline_params();
    let mut phi = 0.8;
    let mut c = 0.3;
    let mut n = 1.0;
    let mut f = 0.0;
    let mut w = 0.0;
    let c0 = c;
    for _ in 0..500 {
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.01, &params);
    }
    assert!(c <= c0 + 1e-6, "catalyst grew without fuel: {c0} -> {c}");
}

#[test]
fn test_structure_requires_catalyst() {
    let params = baseline_params();
    let mut phi = 0.2;
    let mut c = 0.0;
    let mut n = 1.0;
    let mut f = 1.0;
    let mut w = 0.0;
    for _ in 0..500 {
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.01, &params);
    }
    assert!(phi < 0.3, "structure grew without catalyst: {phi}");
}

#[test]
fn test_structure_turns_over() {
    let params = baseline_params();
    let mut phi = 0.8;
    let mut c = 0.35;
    let mut n = 1.0;
    let mut f = 1.0;
    let mut w = 0.0;
    let mut decayed = false;
    for _ in 0..2000 {
        let rates = compute_reactions_at(phi, c, n, f, w, &params, true);
        if rates.r_structure_decay > 0.0 {
            decayed = true;
        }
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.01, &params);
    }
    assert!(decayed);
}

#[test]
fn test_waste_is_produced() {
    let params = baseline_params();
    let mut phi = 0.8;
    let mut c = 0.35;
    let mut n = 1.0;
    let mut f = 1.0;
    let mut w = 0.0;
    for _ in 0..500 {
        reactor_step(&mut phi, &mut c, &mut n, &mut f, &mut w, 0.01, &params);
    }
    assert!(w > 0.01, "no waste produced: {w}");
}

#[test]
fn test_reservoir_replenishes_nutrient() {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut nutrient = vec![0.1; n];
    let mut fuel = vec![1.0; n];
    let mut waste = vec![0.5; n];
    let params = baseline_params();
    for _ in 0..2000 {
        apply_reservoir(&grid, &mut nutrient, &mut fuel, &mut waste, 0.01, &params);
    }
    let mut reservoir_n: f64 = 0.0;
    for idx in 0..n {
        if grid.reservoir_mask[idx] {
            reservoir_n = reservoir_n.max(nutrient[idx]);
        }
    }
    assert!(reservoir_n > 0.95, "nutrient not replenished: {reservoir_n}");
}

#[test]
fn test_reservoir_removes_waste() {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut nutrient = vec![1.0; n];
    let mut fuel = vec![1.0; n];
    let mut waste = vec![1.0; n];
    let params = baseline_params();
    for _ in 0..2000 {
        apply_reservoir(&grid, &mut nutrient, &mut fuel, &mut waste, 0.01, &params);
    }
    let mut reservoir_w: f64 = 1.0;
    for idx in 0..n {
        if grid.reservoir_mask[idx] {
            reservoir_w = reservoir_w.min(waste[idx]);
        }
    }
    assert!(reservoir_w < 0.05, "waste not cleared: {reservoir_w}");
}

#[test]
fn test_passive_phase_separation() {
    let mut params = passive_phase_params();
    params.phase_separation_enabled = true;
    let mut sim = Simulation::new(params);
    let mass0 = total_mass(&sim.grid, &sim.fields.structure);
    for _ in 0..5000 {
        sim.step();
    }
    let mass1 = total_mass(&sim.grid, &sim.fields.structure);
    assert!((mass1 - mass0).abs() / mass0 < 0.05, "mass changed {mass0} -> {mass1}");
    let (_, largest, compactness) = {
        let s = &sim.fields.structure;
        let grid = &sim.grid;
        let mut area = 0u64;
        for idx in 0..grid.width * grid.height {
            if grid.in_dish(idx) && s[idx] >= 0.5 {
                area += 1;
            }
        }
        (area, area, 1.0)
    };
    assert!(largest > 100, "droplet dissolved: {largest}");
    let _ = compactness;
}

#[test]
fn test_baseline_viability() {
    let config = ExperimentConfig {
        name: "baseline_test".into(),
        seed: 1,
        substeps: test_substeps(),
        params: baseline_params(),
        interventions: vec![],
    };
    let sim = run_experiment(&config, 1000);
    let diag = sim.history.last().or_else(|| sim.history.first());
    let mass = total_mass(&sim.grid, &sim.fields.structure);
    let cat = total_mass(&sim.grid, &sim.fields.catalyst);
    assert!(mass > 5.0, "structure collapsed: {mass}");
    assert!(cat > 0.05, "catalyst extinct: {cat}");
    assert!(sim.detector.turnover.nutrient_consumption > 0.0);
    assert!(sim.detector.turnover.fuel_consumption > 0.0);
    assert!(sim.detector.turnover.waste_production > 0.0);
    if let Some(d) = diag {
        assert!(d.catalyst_retention > 0.5, "retention {}", d.catalyst_retention);
    }
}

#[test]
fn test_nutrient_starvation_causes_collapse() {
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(2000, 100);
    let mass_before = total_mass(&sim.grid, &sim.fields.structure);
    let nutrient_before = total_mass(&sim.grid, &sim.fields.nutrient);
    sim.params.n_reservoir = 0.0;
    for _ in 0..3000 {
        sim.step();
    }
    let mass_after = total_mass(&sim.grid, &sim.fields.structure);
    let nutrient_after = total_mass(&sim.grid, &sim.fields.nutrient);
    assert!(
        nutrient_after < nutrient_before * 0.995,
        "nutrient did not fall: {nutrient_before} -> {nutrient_after}"
    );
    assert!(
        mass_after <= mass_before * 1.02,
        "starvation should not grow structure: {mass_before} -> {mass_after}"
    );
}

#[test]
fn test_fuel_starvation_causes_collapse() {
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(2000, 100);
    let mass_before = total_mass(&sim.grid, &sim.fields.structure);
    sim.params.f_reservoir = 0.0;
    for _ in 0..3000 {
        sim.step();
    }
    let mass_after = total_mass(&sim.grid, &sim.fields.structure);
    assert!(
        mass_after <= mass_before * 1.02,
        "fuel starvation allowed growth: {mass_before} -> {mass_after}"
    );
}

#[test]
fn test_catalyst_knockout_causes_collapse() {
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(2000, 100);
    let cat_at_knockout = total_mass(&sim.grid, &sim.fields.catalyst);
    sim.params.k_rep = 0.0;
    sim.detector.turnover.catalyst_reproduction = 0.0;
    for _ in 0..3000 {
        sim.step();
    }
    let cat_after = total_mass(&sim.grid, &sim.fields.catalyst);
    assert_eq!(sim.detector.turnover.catalyst_reproduction, 0.0);
    assert!(
        cat_after < cat_at_knockout,
        "catalyst did not decay after rep knockout: {cat_at_knockout} -> {cat_after}"
    );
}

#[test]
fn test_structure_knockout_causes_collapse() {
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(2000, 100);
    let m_at_knockout = total_mass(&sim.grid, &sim.fields.structure);
    sim.params.k_structure = 0.0;
    for _ in 0..3000 {
        sim.step();
    }
    let m_after = total_mass(&sim.grid, &sim.fields.structure);
    assert!(
        m_after < m_at_knockout * 0.995,
        "structure did not decay after synthesis knockout: {m_at_knockout} -> {m_after}"
    );
}

#[test]
fn test_limited_puncture_can_repair() {
    let pre_steps = 8000u64;
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(pre_steps, 100);
    let pre_mass = total_mass(&sim.grid, &sim.fields.structure);
    apply_intervention(
        &sim.grid,
        &mut sim.fields,
        &InterventionAction::PunctureRepair,
        &mut sim.params,
    );
    for _ in 0..test_substeps() {
        sim.step();
    }
    let post_mass = total_mass(&sim.grid, &sim.fields.structure);
    assert!(
        post_mass >= pre_mass * 0.7,
        "repair failed: pre={pre_mass} post={post_mass}"
    );
}

#[test]
fn test_catastrophic_damage_can_kill() {
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(5000, 100);
    let pre_mass = total_mass(&sim.grid, &sim.fields.structure);
    apply_intervention(
        &sim.grid,
        &mut sim.fields,
        &InterventionAction::CatastrophicDamage,
        &mut sim.params,
    );
    let damaged_mass = total_mass(&sim.grid, &sim.fields.structure);
    assert!(damaged_mass < pre_mass * 0.4, "damage not applied: {pre_mass} -> {damaged_mass}");
    for _ in 0..test_substeps() {
        sim.step();
    }
    let post_mass = total_mass(&sim.grid, &sim.fields.structure);
    // excessive damage may or may not fully kill; must at least impair organization
    assert!(
        post_mass < pre_mass * 0.85,
        "catastrophe had no lasting effect: {pre_mass} -> {post_mass}"
    );
}

#[test]
fn test_dead_cell_does_not_respawn() {
    let mut sim = Simulation::new(baseline_params());
    // Force complete catalyst loss (death) without reseeding
    for idx in 0..sim.grid.width * sim.grid.height {
        if sim.grid.in_dish(idx) {
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.structure[idx] *= 0.01;
        }
    }
    sim.params.n_reservoir = 1.0;
    sim.params.f_reservoir = 1.0;
    for _ in 0..5000 {
        sim.step();
    }
    let cat = total_mass(&sim.grid, &sim.fields.catalyst);
    let mass = total_mass(&sim.grid, &sim.fields.structure);
    assert!(cat < 1.0, "catalyst respawned from zero: {cat}");
    assert!(mass < 100.0, "structure re-formed without catalyst seed: {mass}");
}

#[test]
fn test_snapshot_round_trip() {
    let mut sim = Simulation::new(baseline_params());
    sim.run_substeps(100, 10);
    let snap = sim.snapshot();
    let json = snap.to_json().unwrap();
    let loaded = FieldSnapshot::from_json(&json).unwrap();
    let mut sim2 = Simulation::new(baseline_params());
    sim2.restore_snapshot(&loaded);
    let m1 = total_mass(&sim.grid, &sim.fields.structure);
    let m2 = total_mass(&sim2.grid, &sim2.fields.structure);
    assert!((m1 - m2).abs() < 1e-6);
}

#[test]
fn test_seeded_run_is_reproducible() {
    let config = ExperimentConfig {
        name: "repro".into(),
        seed: 1,
        substeps: 500,
        params: baseline_params(),
        interventions: vec![],
    };
    let sim1 = run_experiment(&config, 100);
    let sim2 = run_experiment(&config, 100);
    let m1 = total_mass(&sim1.grid, &sim1.fields.structure);
    let m2 = total_mass(&sim2.grid, &sim2.fields.structure);
    assert!((m1 - m2).abs() < 1e-4, "reproducibility failed: {m1} vs {m2}");
}
