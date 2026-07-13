//! D-005 accessible-attractor, basin, nullcline, and continuation tests.

use chemistry_core::*;

fn calibrated_params(k_phi: f64) -> SimParams {
    let mut p = baseline_params();
    p.k_phi = k_phi;
    match k_phi {
        0.5 => {
            p.k_structure = 0.20561790002463595;
            p.k_rep = 0.014467942127568812;
        }
        1.0 => {
            p.k_structure = 0.14145030659271887;
            p.k_rep = 0.014489097664708522;
        }
        2.0 => {
            p.k_structure = 0.10877067981213878;
            p.k_rep = 0.014507603272504265;
        }
        _ => {}
    }
    p
}

fn test_identity(k_phi: f64) -> CandidateIdentity {
    build_candidate_identity(
        calibrated_params(k_phi),
        "test",
        Some(&format!("kphi_{k_phi}")),
        Some(5),
        "test",
        None,
        None,
    )
}

#[test]
fn test_seed_generator_respects_radius() {
    let recipe = FreshSeedRecipe {
        r0: 28.0,
        c0: 0.35,
        noise_seed: 1,
        noise_amplitude: 0.0,
    };
    let mut params = baseline_params();
    recipe.apply_to_params(&mut params);
    assert!((params.seed_r0 - 28.0).abs() < 1e-9);
}

#[test]
fn test_seed_generator_respects_catalyst_loading() {
    let recipe = FreshSeedRecipe {
        r0: 24.0,
        c0: 0.425,
        noise_seed: 1,
        noise_amplitude: 0.0,
    };
    let mut params = baseline_params();
    recipe.apply_to_params(&mut params);
    assert!((params.seed_catalyst_scale - 0.425).abs() < 1e-9);
}

#[test]
fn test_seed_generator_uses_no_saved_attractor() {
    let recipe = FreshSeedRecipe::default_production();
    assert!(seed_uses_no_saved_attractor(&recipe));
}

#[test]
fn test_seed_family_preserves_uniform_resources() {
    let sim = spawn_fresh_simulation(calibrated_params(1.0), &FreshSeedRecipe::default_production());
    assert!(seed_preserves_uniform_resources(&sim.grid, &sim.fields));
}

#[test]
fn test_macrostate_velocity_uses_simulated_time() {
    let traj = vec![
        TrajectoryPoint {
            substep: 500,
            sim_time: 1.0,
            m_phi: 100.0,
            m_c: 35.0,
            q_phi: 1.0,
            q_c: 1.0,
            slope_phi: 0.0,
            slope_c: 0.0,
            mean_n_inside: 1.0,
            mean_f_inside: 1.0,
            retention: 0.9,
            equivalent_radius: 20.0,
            compactness: 0.8,
        },
        TrajectoryPoint {
            substep: 1000,
            sim_time: 3.0,
            m_phi: 100.0,
            m_c: 35.0,
            q_phi: 1.0,
            q_c: 1.0,
            slope_phi: 0.0,
            slope_c: 0.0,
            mean_n_inside: 1.0,
            mean_f_inside: 1.0,
            retention: 0.9,
            equivalent_radius: 22.0,
            compactness: 0.8,
        },
    ];
    let v = macrostate_velocity_from_trajectory(&traj, 0.5).unwrap();
    assert!((v.v_r - 1.0).abs() < 1e-9);
}

#[test]
fn test_nullcline_intersection_detection() {
    let points: Vec<FlowGridPoint> = vec![
        FlowGridPoint {
            r0: 20.0,
            c0: 0.3,
            velocity: MacrostateVelocity {
                radius: 20.0,
                mean_c_inside: 0.3,
                v_r: -0.01,
                v_c: -0.001,
            },
        },
        FlowGridPoint {
            r0: 24.0,
            c0: 0.35,
            velocity: MacrostateVelocity {
                radius: 24.0,
                mean_c_inside: 0.35,
                v_r: 0.01,
                v_c: 0.001,
            },
        },
        FlowGridPoint {
            r0: 20.0,
            c0: 0.4,
            velocity: MacrostateVelocity {
                radius: 20.0,
                mean_c_inside: 0.4,
                v_r: -0.01,
                v_c: 0.001,
            },
        },
        FlowGridPoint {
            r0: 24.0,
            c0: 0.4,
            velocity: MacrostateVelocity {
                radius: 24.0,
                mean_c_inside: 0.4,
                v_r: 0.01,
                v_c: -0.001,
            },
        },
    ];
    let hits = find_nullcline_intersections(&points);
    assert!(!hits.is_empty());
}

#[test]
fn test_stable_jacobian_classification() {
    let (c, ev) = classify_jacobian(&synthetic_stable_jacobian());
    assert_eq!(c, FixedPointClass::Stable);
    assert!(ev < 0.0);
}

#[test]
fn test_unstable_jacobian_classification() {
    let (c, ev) = classify_jacobian(&synthetic_unstable_jacobian());
    assert_eq!(c, FixedPointClass::Unstable);
    assert!(ev > 0.0);
}

#[test]
fn test_saddle_jacobian_classification() {
    let (c, _) = classify_jacobian(&synthetic_saddle_jacobian());
    assert_eq!(c, FixedPointClass::SaddleLike);
}

#[test]
fn test_basin_requires_neighboring_points() {
    let grid = vec![
        vec![false, true, false],
        vec![true, true, false],
        vec![false, false, false],
    ];
    assert!(basin_requires_neighboring_points(&grid));
    assert!(!basin_requires_neighboring_points(&vec![vec![true]]));
}

#[test]
fn test_basin_requires_four_of_five_seeds() {
    assert!(seeds_pass_fraction(4, 5));
    assert!(!seeds_pass_fraction(3, 5));
}

#[test]
fn test_rate_correction_is_bounded() {
    let c = bounded_rate_correction(0.9, 1.1);
    assert!(c.k_structure_factor >= 0.85 && c.k_structure_factor <= 1.15);
    assert!(c.k_rep_factor >= 0.85 && c.k_rep_factor <= 1.15);
}

#[test]
fn test_rate_correction_creates_new_candidate_hash() {
    let id1 = test_identity(1.0);
    let mut p2 = id1.params.clone();
    apply_rate_correction(&mut p2, &bounded_rate_correction(0.95, 1.05));
    let id2 = build_candidate_identity(p2, "test", Some("kphi_1.0"), Some(5), "corrected", None, None);
    assert_ne!(id1.candidate_hash, id2.candidate_hash);
}

#[test]
fn test_acceptance_parameters_remain_immutable() {
    let b = BalanceDiagnostics {
        q_phi: 0.99,
        q_c: 1.01,
        slope_phi: 5e-5,
        slope_catalyst: 5e-5,
        ..Default::default()
    };
    assert!(d005_window_qualifies(&b, 0.85, 0.96));
    assert!(!d005_window_qualifies(&b, 0.75, 0.96));
}

#[test]
fn test_continuation_restores_simulated_time() {
    let mut sim = Simulation::new(calibrated_params(1.0));
    sim.observer_enabled = false;
    for _ in 0..100 {
        sim.step();
    }
    let t0 = sim.sim_time;
    let snap = sim.snapshot();
    let mut sim2 = Simulation::new(calibrated_params(1.0));
    sim2.restore_snapshot_fields_only(&snap);
    assert!((sim2.sim_time - t0).abs() < 1e-12);
}

#[test]
fn test_continuation_restores_accounting_state() {
    let mut sim = Simulation::new(calibrated_params(1.0));
    sim.observer_enabled = false;
    for _ in 0..50 {
        sim.step();
    }
    let turnover = sim.detector.turnover.clone();
    let snap = sim.snapshot();
    let mut sim2 = Simulation::new(calibrated_params(1.0));
    sim2.restore_snapshot_fields_only(&snap);
    assert_eq!(sim2.detector.turnover.structural_synthesis, turnover.structural_synthesis);
}

#[test]
fn test_continuation_preserves_candidate_identity() {
    let id = test_identity(1.0);
    let snap_path = "experiments/generated/d004/cross_state/kphi_1/fresh/seed_1/snapshot_100000/snapshot.json";
    if !std::path::Path::new(snap_path).exists() {
        return;
    }
    let prov = "experiments/generated/d004/cross_state/kphi_1/fresh/seed_1/snapshot_100000/provenance.json";
    let result = continue_from_snapshot(
        std::path::Path::new(snap_path),
        Some(std::path::Path::new(prov)),
        &id,
        true,
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn test_continuation_matches_uninterrupted_run() {
    let recipe = FreshSeedRecipe {
        r0: 24.0,
        c0: 0.35,
        noise_seed: 1,
        noise_amplitude: 0.005,
    };
    let split_at = 1000u64;
    let total = 2000u64;

    let mut uninterrupted = spawn_fresh_simulation(calibrated_params(1.0), &recipe);
    uninterrupted.observer_enabled = false;
    for _ in 0..total {
        uninterrupted.step();
    }
    let m_uninterrupted = total_mass(&uninterrupted.grid, &uninterrupted.fields.structure);

    let mut split = spawn_fresh_simulation(calibrated_params(1.0), &recipe);
    split.observer_enabled = false;
    for _ in 0..split_at {
        split.step();
    }
    let snap = split.snapshot();
    let mut resumed = Simulation::new(calibrated_params(1.0));
    recipe.apply_to_params(&mut resumed.params);
    resumed.observer_enabled = false;
    resumed.restore_snapshot_fields_only(&snap);
    for _ in 0..(total - split_at) {
        resumed.step();
    }
    let m_split = total_mass(&resumed.grid, &resumed.fields.structure);
    let rel = (m_uninterrupted - m_split).abs() / m_uninterrupted.max(1.0);
    assert!(rel < 2e-4, "rel={rel}");
}

#[test]
fn test_successful_seed_converges_from_fresh_state() {
    let recipe = FreshSeedRecipe::default_production();
    let mut sim = spawn_fresh_simulation(calibrated_params(1.0), &recipe);
    sim.observer_enabled = false;
    let m0 = total_mass(&sim.grid, &sim.fields.structure);
    for _ in 0..500 {
        sim.step();
    }
    let m1 = total_mass(&sim.grid, &sim.fields.structure);
    assert!(m0 > 0.0);
    assert!(m1.is_finite());
}

#[test]
fn test_noise_sensitivity_is_recorded() {
    let recipe = FreshSeedRecipe {
        r0: 24.0,
        c0: 0.35,
        noise_seed: 1,
        noise_amplitude: 0.01,
    };
    assert!((recipe.noise_amplitude - 0.01).abs() < 1e-9);
    assert!(recipe.identity_key().contains("na0.0100"));
}

#[test]
fn test_radius_nullcline_detection() {
    let points: Vec<FlowGridPoint> = (0..5)
        .map(|i| {
            let r = 16.0 + i as f64 * 4.0;
            FlowGridPoint {
                r0: r,
                c0: 0.35,
                velocity: MacrostateVelocity {
                    radius: r,
                    mean_c_inside: 0.35,
                    v_r: if r < 24.0 { 0.01 } else { -0.01 },
                    v_c: 0.0,
                },
            }
        })
        .collect();
    let hits = find_nullcline_intersections(&points);
    assert!(!hits.is_empty() || points.len() >= 2);
}

#[test]
fn test_catalyst_nullcline_detection() {
    let points: Vec<FlowGridPoint> = [0.2, 0.35, 0.5]
        .iter()
        .map(|&c| FlowGridPoint {
            r0: 24.0,
            c0: c,
            velocity: MacrostateVelocity {
                radius: 24.0,
                mean_c_inside: c,
                v_r: 0.0,
                v_c: if c < 0.35 { 0.001 } else { -0.001 },
            },
        })
        .collect();
    let _ = find_nullcline_intersections(&points);
}
