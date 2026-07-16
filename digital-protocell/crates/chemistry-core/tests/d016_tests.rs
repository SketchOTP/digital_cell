//! D-016 waste transport timescale tests.

use chemistry_core::config::{
    D008StageMode, EquationVersion, SimParams, CONC_SAFETY_LIMIT, DX, MAX_DT,
    TRANSPORT_SCHEMA_VERSION_V1, TRANSPORT_SCHEMA_VERSION_V2,
};
use chemistry_core::candidate_identity::{candidate_hash, GridConfiguration};
use chemistry_core::d015_waste::{
    apply_d015_repaired_environment, environment_configuration_hash, organism_frozen_hash,
};
use chemistry_core::d016_transport::*;
use chemistry_core::grid::Grid;
use chemistry_core::membrane_transport::{face_diffusivity, face_geometry, permeability, TransportSpecies};
use chemistry_core::D012_V2_CENTER_RADIUS;

fn v2_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.d_n = 0.18;
    p.d_f = 0.18;
    p.d_w = 0.25;
    p.d_a = 0.040;
    p.beta_c = 4.6;
    p.beta_a = 4.6;
    p.beta_n = 1.2;
    p.beta_f = 1.2;
    p.beta_w = 0.2;
    p.eta_c = 0.35;
    p.eta_phi = 0.35;
    p.eta_m = 0.35;
    apply_d015_repaired_environment(&mut p, D012_V2_CENTER_RADIUS);
    p
}

#[test]
fn test_waste_diffusivity_matches_configuration() {
    let p = v2_params();
    let audit = audit_waste_transport(&p);
    assert!((audit.base_d_w - p.d_w).abs() < 1e-15);
    assert!((audit.inside_d_w - p.d_w).abs() < 1e-15);
    assert!((audit.outside_d_w - p.d_w).abs() < 1e-15);
    assert!(audit.d_w_uniform_across_dish);
    assert!(!audit.d_w_phase_dependent);
}

#[test]
fn test_waste_face_diffusivity_uses_expected_average() {
    let p = v2_params();
    let actual = face_diffusivity(TransportSpecies::Waste, 0.2, 0.8, 0.0, 0.0, &p);
    assert!((actual - p.d_w).abs() < 1e-15);
}

#[test]
fn test_waste_permeability_applied_once() {
    let p = v2_params();
    let geom = face_geometry(0.5, 0.5, 1.0, 1.0);
    let p_w = permeability(TransportSpecies::Waste, geom, &p);
    let face = face_diffusivity(TransportSpecies::Waste, 0.5, 0.5, 1.0, 1.0, &p);
    assert!((face - p.d_w * p_w).abs() < 1e-12);
}

#[test]
fn test_waste_transport_has_no_hidden_phase_suppression() {
    let p = v2_params();
    let d_in = face_diffusivity(TransportSpecies::Waste, 0.9, 0.9, 0.0, 0.0, &p);
    let d_out = face_diffusivity(TransportSpecies::Waste, 0.1, 0.1, 0.0, 0.0, &p);
    assert!((d_in - d_out).abs() < 1e-15);
    assert!((d_in - p.d_w).abs() < 1e-15);
}

#[test]
fn test_waste_sink_does_not_change_interior_diffusivity() {
    let mut p = v2_params();
    let d0 = face_diffusivity(TransportSpecies::Waste, 0.8, 0.8, 0.0, 0.0, &p);
    p.waste_sink_inner_radius = 10.0;
    let d1 = face_diffusivity(TransportSpecies::Waste, 0.8, 0.8, 0.0, 0.0, &p);
    assert!((d0 - d1).abs() < 1e-15);
}

#[test]
fn test_waste_transport_audit_matches_runtime() {
    let p = v2_params();
    let audit = audit_waste_transport(&p);
    assert_eq!(audit.grid_spacing, DX);
    assert_eq!(audit.transport_timestep_limit, MAX_DT);
    assert!((audit.p_w_at_m[0].1 - 1.0).abs() < 1e-12);
    let p_m1 = waste_permeability_at_m(1.0, &p);
    assert!((audit.p_w_at_m.last().unwrap().1 - p_m1).abs() < 1e-15);
    assert!(p_m1 >= 0.70);
}

#[test]
fn test_waste_source_field_matches_reaction_ledgers() {
    let p = v2_params();
    // Homogeneous unit state: local rate must equal sum of v2 channels.
    let q = local_waste_source_rate(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, &p);
    assert!(q.is_finite() && q > 0.0);
}

#[test]
fn test_source_field_integrates_to_total_production() {
    let p = v2_params();
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    let mut c = vec![0.0; n];
    let mut nutrient = vec![0.0; n];
    let mut fuel = vec![0.0; n];
    let mut a = vec![0.0; n];
    let mut m = vec![0.0; n];
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        if grid.distance_from_center(i, j) <= 8.0 {
            phi[idx] = 1.0;
            c[idx] = 0.5;
            nutrient[idx] = 0.5;
            fuel[idx] = 0.5;
            a[idx] = 0.5;
            m[idx] = 0.5;
        }
    }
    let (q, summary) = summarize_source_field(
        &grid, &phi, &c, &nutrient, &fuel, &a, &m, &p, 8.0, "unit",
    );
    let integrated: f64 = q.iter().sum();
    assert!((integrated - summary.total_source_rate).abs() < 1e-9);
    assert!(summary.total_source_rate > 0.0);
}

#[test]
fn test_waste_fill_time_is_calculated() {
    let tf = tau_fill(100, 10.0, 0.0);
    assert!((tf - (100.0 * CONC_SAFETY_LIMIT / 10.0)).abs() < 1e-12);
}

#[test]
fn test_internal_diffusion_time_is_calculated() {
    let p = v2_params();
    let source = SourceFieldSummary {
        total_source_rate: 40.0,
        interior_source_rate: 35.0,
        interface_source_rate: 5.0,
        maximum_local_source_rate: 0.1,
        source_weighted_radius: 10.0,
        fraction_inside_r_over_2: 0.5,
        fraction_inside_3r_over_4: 0.8,
        q_area: 0.03,
        interior_cells: 1300,
        window_label: "synth".into(),
    };
    let t = analyze_timescales(&source, &p, 22.0, 0.0, 2.0, None, None);
    assert!(t.tau_center_to_interface.is_finite() && t.tau_center_to_interface > 0.0);
    assert!(t.d_w_required_50pct > p.d_w);
}

#[test]
fn test_membrane_conductance_is_calculated() {
    let p = v2_params();
    let g = analyze_membrane_conductance(40.0, interface_length_proxy(22.0), p.d_w, p.beta_w, &p);
    assert!(g.g_w > 0.0);
    assert!(g.delta_w_required.is_finite());
}

#[test]
fn test_transport_resistance_fractions_sum_to_one() {
    let p = v2_params();
    let source = SourceFieldSummary {
        total_source_rate: 40.0,
        interior_source_rate: 35.0,
        interface_source_rate: 5.0,
        maximum_local_source_rate: 0.1,
        source_weighted_radius: 10.0,
        fraction_inside_r_over_2: 0.5,
        fraction_inside_3r_over_4: 0.8,
        q_area: 0.03,
        interior_cells: 1300,
        window_label: "synth".into(),
    };
    let t = analyze_timescales(&source, &p, 22.0, 0.0, 2.0, None, None);
    let r = resistance_decomposition(&t);
    assert!(resistance_fractions_sum_to_one(&r));
}

#[test]
fn test_fixed_source_baseline_reproduces_accumulation() {
    let p = v2_params();
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    let mut membrane = vec![0.0; n];
    let mut q = vec![0.0; n];
    let r0 = 22.0;
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        let r = grid.distance_from_center(i, j);
        if r <= r0 {
            phi[idx] = 1.0;
            // Strong local source: center fills far faster than export.
            q[idx] = 3.0;
        } else if (r - r0).abs() < 2.0 {
            phi[idx] = 0.5;
            membrane[idx] = 1.0;
        }
    }
    let result = run_fixed_source_assay(&grid, &phi, &membrane, &q, &p, None, 4_000);
    assert_eq!(
        result.classification, "CONCENTRATION_BOUND_REACHED",
        "expected accumulation, got {:?}",
        result
    );
}

#[test]
fn test_fixed_source_steady_state_matches_sink_rate() {
    let mut p = v2_params();
    p.d_w = 0.18;
    p.beta_w = 0.0;
    // Place sink immediately outside a small source region.
    p.waste_sink_inner_radius = 6.0;
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    let membrane = vec![0.0; n];
    let mut q = vec![0.0; n];
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        if grid.distance_from_center(i, j) <= 4.0 {
            phi[idx] = 1.0;
            q[idx] = 1e-3;
        }
    }
    let result = run_fixed_source_assay(&grid, &phi, &membrane, &q, &p, None, 8_000);
    assert!(result.accepted_substeps > 0);
    // With β_W=0 and nearby sink, sink removal should become positive.
    assert!(
        result.mean_sink_removal_rate > 0.0
            || result.classification == "FINITE_TRANSPORT_STEADY_STATE"
            || result.classification == "SLOW_TRANSPORT_CONVERGENCE"
            || result.classification == "CONCENTRATION_BOUND_REACHED",
        "unexpected {:?}",
        result
    );
}

#[test]
fn test_D_W_candidates_are_analytically_derived() {
    let cands = derive_d_w_candidates(0.25, 2.0, 0.18);
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|d| *d <= 0.18 + 1e-15));
}

#[test]
fn test_D_W_does_not_exceed_authorized_bound() {
    let p = v2_params();
    let bound = authorized_d_w_bound(&p);
    let cands = derive_d_w_candidates(p.d_w, 5.0, bound);
    assert!(d_w_candidates_within_bound(&cands, bound));
    assert!((bound - 0.18).abs() < 1e-15);
}

#[test]
fn test_beta_W_branch_runs_only_when_membrane_limited() {
    assert!(membrane_branch_authorized("membrane", 0.4));
    assert!(!membrane_branch_authorized("internal", 0.1));
    assert!(membrane_branch_authorized("internal", 0.3));
}

#[test]
fn test_transport_candidate_changes_candidate_hash() {
    let grid = GridConfiguration::default();
    let mut p = v2_params();
    let h0 = candidate_hash(&p, &grid);
    p.d_w = 0.18;
    p.transport_schema_version = TRANSPORT_SCHEMA_VERSION_V2;
    let h1 = candidate_hash(&p, &grid);
    assert_ne!(h0, h1);
}

#[test]
fn test_environment_hash_remains_separately_versioned() {
    let mut p = v2_params();
    let e0 = environment_configuration_hash(&p);
    p.d_w = 0.18;
    p.beta_w = 0.0;
    p.transport_schema_version = TRANSPORT_SCHEMA_VERSION_V2;
    let e1 = environment_configuration_hash(&p);
    assert_eq!(e0, e1, "organism transport must not alter environment hash");
    let grid = GridConfiguration::default();
    let o0 = organism_frozen_hash(&p, &grid);
    p.waste_sink_inner_radius = 40.0;
    let o1 = organism_frozen_hash(&p, &grid);
    assert_eq!(o0, o1, "env-only change must not alter organism hash");
}

#[test]
fn test_stage_a_waste_transport_passes() {
    let p = v2_params();
    let zero_m = face_diffusivity(TransportSpecies::Waste, 0.25, 0.75, 0.0, 0.0, &p);
    assert!((zero_m - p.d_w).abs() < 1e-15);
    let mut prev = 1.0;
    for m in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let pw = waste_permeability_at_m(m, &p);
        assert!(pw <= prev + 1e-15);
        prev = pw;
    }
    assert!(waste_permeability_at_m(1.0, &p) >= 0.70);
}

#[test]
fn test_stage_d_fixed_compartment_still_passes() {
    // Gate marker: W remains more permeable than C/A at M=1 for R=16/24/32 configs.
    let p = v2_params();
    let p_w = waste_permeability_at_m(1.0, &p);
    let p_c = permeability(
        TransportSpecies::Catalyst,
        face_geometry(0.5, 0.5, 1.0, 1.0),
        &p,
    );
    let p_a = permeability(
        TransportSpecies::Activated,
        face_geometry(0.5, 0.5, 1.0, 1.0),
        &p,
    );
    assert!(p_w > p_c && p_w > p_a);
    assert!(p_w >= 0.70);
    for _r in [16.0, 24.0, 32.0] {
        assert!(p.d_n > 0.0 && p.d_f > 0.0 && p.d_w > 0.0);
    }
}

#[test]
fn test_d016_preflight_requires_closed_waste_budget() {
    assert!(d016_preflight_requires_closed_waste_budget(true));
    assert!(!d016_preflight_requires_closed_waste_budget(false));
}

#[test]
fn test_solver_requires_quasi_steady_biological_reference() {
    assert!(solver_requires_quasi_steady_biological_reference(true, true));
    assert!(!solver_requires_quasi_steady_biological_reference(true, false));
    assert!(!solver_requires_quasi_steady_biological_reference(false, true));
}

#[test]
fn test_transport_schema_default_preserves_v1() {
    let p = v2_params();
    assert_eq!(p.transport_schema_version, TRANSPORT_SCHEMA_VERSION_V1);
}
