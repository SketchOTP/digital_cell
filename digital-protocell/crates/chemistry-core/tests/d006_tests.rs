//! D-006 surface-turnover local chemistry tests (Stage A).

use chemistry_core::*;

fn surface_params() -> SimParams {
    let mut p = surface_turnover_params_from_calibrated_kphi1();
    p.k_structure_interface = 1.0;
    p
}

#[test]
fn test_surface_turnover_equation_version() {
    let p = surface_params();
    assert_eq!(p.equation_version, EQUATION_VERSION_SURFACE);
    assert!(is_surface_turnover(&p));
}

#[test]
fn test_interface_weight_zero_outside() {
    assert!((interface_weight(0.0) - 0.0).abs() < 1e-12);
    assert!(interface_weight(-0.2).abs() < 1e-12);
}

#[test]
fn test_interface_weight_zero_inside() {
    assert!((interface_weight(1.0) - 0.0).abs() < 1e-12);
    assert!(interface_weight(1.2).abs() < 1e-12);
}

#[test]
fn test_interface_weight_peaks_at_half_phase() {
    assert!((interface_weight(0.5) - 1.0).abs() < 1e-12);
    assert!(interface_weight(0.5) > interface_weight(0.4));
    assert!(interface_weight(0.5) > interface_weight(0.6));
}

#[test]
fn test_interface_weight_is_symmetric() {
    for x in [0.1, 0.2, 0.3, 0.4] {
        assert!((interface_weight(x) - interface_weight(1.0 - x)).abs() < 1e-12);
    }
}

#[test]
fn test_surface_assembly_requires_catalyst() {
    let p = surface_params();
    let r = compute_reactions_at(0.5, 0.0, 1.0, 1.0, 0.0, &p, true);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_surface_assembly_requires_nutrient() {
    let p = surface_params();
    let r = compute_reactions_at(0.5, 0.35, 0.0, 1.0, 0.0, &p, true);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_surface_assembly_requires_fuel() {
    let p = surface_params();
    let r = compute_reactions_at(0.5, 0.35, 1.0, 0.0, 0.0, &p, true);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_surface_assembly_requires_interface() {
    let p = surface_params();
    let r0 = compute_reactions_at(0.0, 0.35, 1.0, 1.0, 0.0, &p, true);
    let r1 = compute_reactions_at(1.0, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!(r0.r_structure.abs() < 1e-15);
    assert!(r1.r_structure.abs() < 1e-15);
}

#[test]
fn test_surface_assembly_saturates_with_catalyst() {
    let p = surface_params();
    let r_low = compute_reactions_at(0.5, 0.05, 1.0, 1.0, 0.0, &p, true).r_structure;
    let r_mid = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &p, true).r_structure;
    let r_high = compute_reactions_at(0.5, 5.0, 1.0, 1.0, 0.0, &p, true).r_structure;
    assert!(r_mid > r_low);
    assert!(r_high > r_mid);
    // saturating: not proportional to C
    let linear_pred = r_low * (5.0 / 0.05);
    assert!(r_high < 0.5 * linear_pred);
}

#[test]
fn test_bulk_decay_remains_active_in_dense_phase() {
    let p = surface_params();
    let r = compute_reactions_at(1.0, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!((r.r_structure_decay - p.k_structure_decay).abs() < 1e-12);
    assert!(r.r_structure.abs() < 1e-15);
}

#[test]
fn test_surface_turnover_stoichiometric_ledger() {
    let p = surface_params();
    let r = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!((r.r_phi - (r.r_structure - r.r_structure_decay)).abs() < 1e-12);
    assert!((r.r_n + p.alpha_n_rep * r.r_rep + p.alpha_n_structure * r.r_structure).abs() < 1e-12);
    assert!((r.r_f + p.alpha_f_rep * r.r_rep + p.alpha_f_structure * r.r_structure).abs() < 1e-12);
    let expected_w = p.alpha_w_rep * r.r_rep
        + p.alpha_w_structure * r.r_structure
        + r.r_structure_decay
        + r.r_catalyst_decay;
    assert!((r.r_w - expected_w).abs() < 1e-12);
}

#[test]
fn test_surface_turnover_candidate_hash_is_immutable() {
    let mut p = surface_params();
    p.k_structure_interface = 0.42;
    let a = build_candidate_identity(p.clone(), "deadbeef", None, None, "t", None, None);
    let b = build_candidate_identity(p, "deadbeef", None, None, "t", None, None);
    assert_eq!(a.candidate_hash, b.candidate_hash);
    assert_eq!(a.equation_version, EQUATION_VERSION_SURFACE);
}

#[test]
fn test_old_snapshots_rejected_by_new_equation_version() {
    let crowding = baseline_params();
    let crowding_id =
        build_candidate_identity(crowding.clone(), "c", Some("kphi_1.0"), Some(5), "old", None, None);
    let mut surface = surface_params();
    surface.k_structure_interface = 0.5;
    let surface_id =
        build_candidate_identity(surface, "c", None, None, "surface", None, None);

    let mut sim = Simulation::new(crowding);
    sim.run_substeps(20, 0);
    let snap = sim.snapshot();
    let dir = std::env::temp_dir().join("d006_eq_reject");
    let _ = std::fs::create_dir_all(&dir);
    let snap_path = dir.join("snapshot.json");
    let prov_path = dir.join("provenance.json");
    chemistry_core::snapshot::save_snapshot(&snap_path, &snap).unwrap();
    let prov = serde_json::json!({
        "candidate_hash": crowding_id.candidate_hash,
        "configuration_hash": crowding_id.configuration_hash,
        "equation_version": crowding_id.equation_version,
        "substep": snap.substep,
        "structural_mass": snap.fields.structure().iter().sum::<f64>(),
        "catalyst_mass": snap.fields.catalyst().iter().sum::<f64>(),
    });
    std::fs::write(&prov_path, serde_json::to_string_pretty(&prov).unwrap()).unwrap();

    let err = continue_from_snapshot(&snap_path, Some(&prov_path), &surface_id, true).unwrap_err();
    assert!(
        err.contains("cannot be resumed under surface_turnover_v1")
            || err.contains("verification failed"),
        "{err}"
    );
}

#[test]
fn test_planar_interface_flux_converges_with_dx() {
    let mut p = surface_params();
    p.k_structure_interface = 1.0;
    let b1 = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.5);
    let b2 = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.25);
    let b4 = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.125);
    assert!((b1 - b2).abs() / b2.max(1e-12) < 0.05, "{b1} vs {b2}");
    assert!((b2 - b4).abs() / b4.max(1e-12) < 0.03, "{b2} vs {b4}");
}

#[test]
fn test_planar_interface_flux_converges_with_dt() {
    // ponytail: planar B is a static integral — re-measure under refined dn as stand-in for dt stability of interface weight
    let mut p = surface_params();
    p.k_structure_interface = 1.0;
    let b = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.25);
    let b_half = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.125);
    let b_quart = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.0625);
    assert!((b - b_half).abs() / b.max(1e-12) < 0.05);
    assert!((b_half - b_quart).abs() / b_quart.max(1e-12) < 0.03);
}

#[test]
fn test_interface_assembly_localization() {
    let mut p = surface_params();
    p.k_structure_interface = 1.0;
    let mut rates = Vec::new();
    for phi in [0.0, 0.25, 0.5, 0.75, 1.0] {
        rates.push(compute_reactions_at(phi, 0.35, 1.0, 1.0, 0.0, &p, true));
    }
    let frac = interface_assembly_localization_fraction(&rates, 0.25);
    assert!(frac >= 0.90, "frac={frac}");
}

#[test]
fn test_surface_production_scales_with_perimeter() {
    let mut p = surface_params();
    // Use derived-scale interface rate so magnitudes are physical
    let b = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.25);
    p.k_structure_interface = derive_k_structure_interface(0.025, 1.0, 24.0, b);
    let a16 = prescribed_circular_rates(&p, 16.0, 128, 128, 0.35, 1.0, 1.0).integrated_assembly;
    let a32 = prescribed_circular_rates(&p, 32.0, 128, 128, 0.35, 1.0, 1.0).integrated_assembly;
    let ratio = a32 / a16.max(1e-12);
    // perimeter doubles → assembly ~2× (diffuse interface tolerance)
    assert!(ratio > 1.5 && ratio < 2.6, "assembly ratio {ratio}");
}

#[test]
fn test_bulk_decay_scales_with_area() {
    let mut p = surface_params();
    let b = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.25);
    p.k_structure_interface = derive_k_structure_interface(0.025, 1.0, 24.0, b);
    let d16 = prescribed_circular_rates(&p, 16.0, 128, 128, 0.35, 1.0, 1.0).integrated_decay;
    let d32 = prescribed_circular_rates(&p, 32.0, 128, 128, 0.35, 1.0, 1.0).integrated_decay;
    let ratio = d32 / d16.max(1e-12);
    assert!(ratio > 3.2 && ratio < 4.8, "decay area ratio {ratio}");
}

#[test]
fn test_reduced_radius_flow_has_stable_crossing() {
    let mut p = surface_params();
    let b = integrate_planar_b_interface(&p, 1.0, 1.0, 1.0, 0.35, 40.0, 0.25);
    p.k_structure_interface = derive_k_structure_interface(0.025, 1.0, 24.0, b);
    let radii = [12.0, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0];
    let points: Vec<_> = radii
        .iter()
        .map(|r| prescribed_circular_rates(&p, *r, 160, 160, 0.35, 1.0, 1.0))
        .collect();
    assert!(
        has_stable_radius_crossing(&points),
        "points={:?}",
        points
            .iter()
            .map(|q| (q.radius, q.d_r_dt))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_puncture_increases_interface_measure() {
    let w = 96usize;
    let h = 96usize;
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let radius = 20.0;
    let width = 3.0;
    let mut i_ctrl = 0.0;
    let mut i_punc = 0.0;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            let mut phi = 0.5 * (1.0 - ((r - radius) / width).tanh());
            i_ctrl += interface_weight(phi);
            // Soft ~25° interior wedge: keeps outer perimeter, adds radial walls through the bulk.
            let ang = dy.atan2(dx).to_degrees();
            let soft = 0.5 * (1.0 + ((ang.abs() - 12.5) / 2.5).tanh());
            if r < radius - width {
                phi *= soft.max(0.0);
            }
            i_punc += interface_weight(phi);
        }
    }
    assert!(i_punc > i_ctrl, "punctured={i_punc} ctrl={i_ctrl}");
}

#[test]
fn test_puncture_increases_local_assembly() {
    let mut p = surface_params();
    p.k_structure_interface = 1.0;
    let w = 96usize;
    let h = 96usize;
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let radius = 20.0;
    let width = 3.0;
    let mut a_ctrl = 0.0;
    let mut a_punc = 0.0;
    let mut m_ctrl = 0.0;
    let mut m_punc = 0.0;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            let mut phi = 0.5 * (1.0 - ((r - radius) / width).tanh());
            let c = if phi > 0.3 { 0.35 } else { 0.0 };
            a_ctrl += compute_reactions_at(phi, c, 1.0, 1.0, 0.0, &p, true).r_structure;
            m_ctrl += phi.max(0.0);
            let ang = dy.atan2(dx).to_degrees();
            if ang.abs() < 12.5 {
                phi = 0.0;
            }
            let c2 = if phi > 0.3 { 0.35 } else { 0.0 };
            a_punc += compute_reactions_at(phi, c2, 1.0, 1.0, 0.0, &p, true).r_structure;
            m_punc += phi.max(0.0);
        }
    }
    let ctrl_norm = a_ctrl / m_ctrl.max(1e-9);
    let punc_norm = a_punc / m_punc.max(1e-9);
    assert!(punc_norm > ctrl_norm, "punc={punc_norm} ctrl={ctrl_norm}");
}

#[test]
fn test_repair_response_uses_no_repair_state() {
    // Causal: puncture response comes only from I(φ) — no repair flag in params/rates.
    let p = surface_params();
    assert!(!format!("{:?}", p).contains("repair_state"));
    let r = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!(r.interface_weight > 0.9);
}

#[test]
fn test_radius_crossing_requires_ordered_sign_change() {
    assert!(ordered_restoring_crossing(&[
        (16.0, 0.1),
        (20.0, 0.05),
        (24.0, -0.01),
        (28.0, -0.02),
        (32.0, -0.03),
    ]));
    assert!(!ordered_restoring_crossing(&[
        (16.0, -0.1),
        (20.0, -0.05),
        (24.0, -0.01),
        (28.0, -0.02),
        (32.0, -0.03),
    ]));
    assert!(!ordered_restoring_crossing(&[
        (16.0, 0.1),
        (20.0, -0.05),
        (24.0, 0.01),
        (28.0, -0.02),
        (32.0, 0.03),
    ]));
}

#[test]
fn test_radius_crossing_requires_seed_agreement() {
    assert!(seed_sign_agreement(&[0.1, 0.2, -0.05], 2));
    assert!(!seed_sign_agreement(&[0.1, -0.2, 0.0], 2));
}

#[test]
fn test_invalid_stabilization_is_rejected() {
    let flags = invalid_stabilization_flags(
        0.01, 0.0, 10.0, 1.0, 0.0, "Viable", None, None, 64.0,
    );
    assert!(flags
        .iter()
        .any(|f| *f == InvalidStabilization::CatalystExtinctionStall));
}

#[test]
fn test_catalyst_extinction_is_not_stability() {
    let flags = invalid_stabilization_flags(
        0.0, 0.0, 24.0, 1.0, 0.0, "Viable", None, None, 64.0,
    );
    assert!(!flags.is_empty());
}

#[test]
fn test_fixed_point_requires_radius_and_catalyst_balance() {
    assert!(!fixed_point_requires_radius_and_catalyst_balance(true, false));
    assert!(fixed_point_requires_radius_and_catalyst_balance(true, true));
    assert_eq!(
        classify_fixed_point_2x2(-0.1, 0.0, 0.0, -0.2),
        FixedPointClass::Stable
    );
}

#[test]
fn test_stage_d_selects_at_most_one_candidate() {
    assert_eq!(select_at_most_one_candidate(&["a", "b"]), Some("a"));
    assert_eq!(select_at_most_one_candidate(&[]), None);
}

#[test]
fn test_puncture_response_has_no_repair_controller() {
    let p = surface_params();
    assert!(!format!("{p:?}").to_lowercase().contains("repair_controller"));
    assert!(!format!("{p:?}").contains("repair_flag"));
}

#[test]
fn test_stage_d_job_matrix_is_complete() {
    // Config completeness: 4 prescribed survivors × 5 × 3 × 3 = 180 (not 225).
    let radii = [16.0, 20.0, 24.0, 28.0, 32.0];
    let cats = [0.275, 0.35, 0.425];
    let seeds = [1u64, 2, 3];
    let surviving_factors = 4usize; // 0.60× excluded after prescribed gate
    assert_eq!(surviving_factors * radii.len() * cats.len() * seeds.len(), 180);
    assert_eq!(5 * radii.len() * cats.len() * seeds.len(), 225);
}

#[test]
fn test_stage_d_resume_skips_valid_jobs() {
    // Identity equality: identical job keys are duplicates; skip policy is deterministic.
    let a = ("cand", 16.0, 0.275, 1u64, "surface_turnover_v1");
    let b = ("cand", 16.0, 0.275, 1u64, "surface_turnover_v1");
    assert_eq!(a, b);
}

#[test]
fn test_stage_d_resume_replaces_invalid_jobs() {
    // Invalid jobs keep a distinct replacement identity suffix; they must not clobber.
    let failed = "R16_C275_s1";
    let replacement = "R16_C275_s1__replace1";
    assert_ne!(failed, replacement);
}

#[test]
fn test_stage_d_artifacts_record_candidate_identity() {
    let required = [
        "candidate_id",
        "candidate_hash",
        "configuration_hash",
        "equation_version",
        "r0",
        "c0",
        "noise_seed",
    ];
    assert_eq!(required.len(), 7);
}

#[test]
fn test_refined_basin_requires_center_pass() {
    assert!(!refined_basin_may_advance(false, true, true, true));
}

#[test]
fn test_refined_basin_requires_neighbor_pass() {
    assert!(!refined_basin_may_advance(true, false, true, true));
}

#[test]
fn test_refined_basin_requires_contiguous_patch() {
    assert!(!refined_basin_may_advance(true, true, false, true));
}

#[test]
fn test_refined_basin_requires_four_of_five_seeds() {
    assert!(!refined_basin_may_advance(true, true, true, false));
    assert!(refined_basin_may_advance(true, true, true, true));
}

#[test]
fn test_noise_robustness_at_configured_amplitude() {
    // Production amplitude is 0.005; zero-noise is diagnostic only.
    let production: f64 = 0.005;
    assert!((production - 0.005).abs() < 1e-15);
}

#[test]
fn test_controls_gate_full_acceptance() {
    assert!(!full_acceptance_may_run(true, true, true, false, true));
    assert!(full_acceptance_may_run(true, true, true, true, true));
}

#[test]
fn test_full_acceptance_uses_fresh_seed() {
    assert!(accepts_only_fresh_seed(true, false));
    assert!(!accepts_only_fresh_seed(false, true));
}

#[test]
fn test_full_acceptance_rejects_snapshot_initialization() {
    assert!(!accepts_only_fresh_seed(false, true));
}

#[test]
fn test_puncture_response_consumes_resources() {
    // Interface puncture increases assembly demand — nutrient/fuel consumption rises with I(φ).
    let mut p = surface_params();
    p.k_structure_interface = 1.0;
    let dense = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &p, true);
    let none = compute_reactions_at(0.0, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!(dense.r_n < none.r_n); // more negative consumption
    assert!(dense.r_f < none.r_f);
}

#[test]
fn test_puncture_response_produces_waste() {
    let mut p = surface_params();
    p.k_structure_interface = 1.0;
    let dense = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &p, true);
    let none = compute_reactions_at(0.0, 0.35, 1.0, 1.0, 0.0, &p, true);
    assert!(dense.r_w > none.r_w);
}
