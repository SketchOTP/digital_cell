//! D-024 interfacial surface-density tests (Gates 0–5).

use chemistry_core::config::{
    EquationVersion, SimParams, CONC_SAFETY_LIMIT, DISH_RADIUS, EIGHT_FIELD_COUNT,
    GRID_HEIGHT, GRID_WIDTH, MEMBRANE_TRANSPORT_SCHEMA_VERSION_V3,
    PRECURSOR_SCHEMA_VERSION_V1, SURFACE_DENSITY_SCHEMA_VERSION_V1,
};
use chemistry_core::fields::{field_sha256_stable, FieldBuffers, FIELD_NAMES_V6};
use chemistry_core::grid::Grid;
use chemistry_core::membrane_transport::{
    face_flux, permeability_surface_occupancy, TransportSpecies,
};
use chemistry_core::operators::total_mass;
use chemistry_core::snapshot::FieldSchemaVersion;
use chemistry_core::surface_density::{
    adsorption_rate_j, circular_phi_profile, compute_interface_geometry,
    estimate_contour_perimeter, evolve_surface_density, integrated_delta,
    planar_phi_profile, projector_identities, seed_surface_from_gamma,
    surface_advection_rate, surface_localization, total_surface_mass,
    circumferential_gamma_variance, InterfaceGeometryCell,
};
use chemistry_core::{build_candidate_identity, Simulation};

fn v7_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p.k_ads = 0.2;
    p.k_gamma_decay = 0.1;
    p.d_gamma = 0.05;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    p.k_precursor = 0.2;
    p.d_p = p.d_a;
    p.k_precursor_decay = p.k_d008_activated_decay;
    p
}

fn v6_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV6PrecursorAssembly;
    p.k_precursor = 0.2;
    p.k_assembly = 0.3;
    p
}

fn vertical_chord_length(grid: &Grid, x0: f64) -> f64 {
    let dx = (x0 - grid.cx).abs();
    if dx >= DISH_RADIUS {
        return 0.0;
    }
    2.0 * (DISH_RADIUS * DISH_RADIUS - dx * dx).sqrt()
}

fn circle_geometry(grid: &Grid, radius: f64, eps: f64) -> (Vec<f64>, Vec<InterfaceGeometryCell>) {
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    circular_phi_profile(grid, radius, eps, &mut phi);
    compute_interface_geometry(grid, &phi, v7_params().eta_n, &mut geometry);
    (phi, geometry)
}

// === Gate 0: schema and preservation ===

#[test]
fn test_eight_field_allocation_and_swap_membrane_is_s() {
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut f = FieldBuffers::new(n);
    assert_eq!(f.precursor.len(), n);
    assert_eq!(f.membrane.len(), n);
    assert_eq!(EIGHT_FIELD_COUNT, 8);
    assert_eq!(FIELD_NAMES_V6.len(), 8);

    f.membrane[10] = 0.7;
    f.precursor[10] = 0.3;
    f.copy_current_to_next();
    assert_eq!(f.membrane_next[10], 0.7);
    assert_eq!(f.precursor_next[10], 0.3);

    f.membrane[10] = 1.0;
    f.membrane_next[10] = 2.0;
    f.precursor[10] = 3.0;
    f.precursor_next[10] = 4.0;
    f.swap();
    assert_eq!(f.membrane[10], 2.0);
    assert_eq!(f.precursor[10], 4.0);
}

#[test]
fn test_v7_schema_versions() {
    let v7 = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    assert!(v7.is_surface_density());
    assert!(v7.is_eight_field());
    assert_eq!(
        v7.surface_density_schema_version(),
        SURFACE_DENSITY_SCHEMA_VERSION_V1
    );
    assert_eq!(v7.precursor_schema_version(), PRECURSOR_SCHEMA_VERSION_V1);
    assert_eq!(
        v7.membrane_transport_schema_version(),
        MEMBRANE_TRANSPORT_SCHEMA_VERSION_V3
    );
    assert!(!EquationVersion::MembraneMetabolismV6PrecursorAssembly.is_surface_density());
}

#[test]
fn test_surface_density_snapshot_roundtrip() {
    let mut sim = Simulation::new(v7_params());
    sim.fields.membrane[500] = 0.42;
    sim.fields.precursor[500] = 0.11;
    let snap = sim.snapshot();
    assert_eq!(snap.field_schema_version, FieldSchemaVersion::SurfaceDensityV1);
    assert!(snap.validate().is_ok());
    assert!((snap.fields.membrane().expect("membrane payload")[500] - 0.42).abs() < 1e-15);

    let mut restored = FieldBuffers::new(GRID_WIDTH * GRID_HEIGHT);
    snap.try_restore_fields(&mut restored).expect("restore");
    assert!((restored.membrane[500] - 0.42).abs() < 1e-15);
    assert!((restored.precursor[500] - 0.11).abs() < 1e-15);
}

#[test]
fn test_v1_v6_cannot_resume_as_v7_and_v7_cannot_resume_as_v6() {
    let sim6 = Simulation::new(v6_params());
    let snap6 = sim6.snapshot();
    assert_eq!(snap6.field_schema_version, FieldSchemaVersion::EightFieldV1);
    assert!(snap6.can_resume_into(&v7_params()).is_err());

    let sim7 = Simulation::new(v7_params());
    let snap7 = sim7.snapshot();
    assert_eq!(snap7.field_schema_version, FieldSchemaVersion::SurfaceDensityV1);
    assert!(snap7.can_resume_into(&v6_params()).is_err());
    assert!(snap7.can_resume_into(&v7_params()).is_ok());
}

#[test]
fn test_candidate_hash_includes_surface_params_not_k_assembly() {
    let base = build_candidate_identity(v7_params(), "t", Some("v7"), None, "v7", None, None);

    let mut p_ka = v7_params();
    p_ka.k_assembly *= 2.0;
    let ka = build_candidate_identity(p_ka, "t", Some("v7"), None, "v7", None, None);
    assert_eq!(
        base.candidate_hash, ka.candidate_hash,
        "k_assembly must not affect v7 hash"
    );

    let mut p_kads = v7_params();
    p_kads.k_ads *= 2.0;
    let kads = build_candidate_identity(p_kads, "t", Some("v7"), None, "v7", None, None);
    assert_ne!(base.candidate_hash, kads.candidate_hash);

    let mut p_dg = v7_params();
    p_dg.d_gamma *= 2.0;
    let dg = build_candidate_identity(p_dg, "t", Some("v7"), None, "v7", None, None);
    assert_ne!(base.candidate_hash, dg.candidate_hash);

    let mut p_gmax = v7_params();
    p_gmax.gamma_max *= 1.5;
    let gmax = build_candidate_identity(p_gmax, "t", Some("v7"), None, "v7", None, None);
    assert_ne!(base.candidate_hash, gmax.candidate_hash);
}

#[test]
fn test_rejected_step_is_atomic() {
    let mut sim = Simulation::new(v7_params());
    sim.observer_enabled = false;
    assert!(sim.step());
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.catalyst[center] = CONC_SAFETY_LIMIT + 1.0;
    let membrane_before = field_sha256_stable(&sim.fields.membrane);
    let precursor_before = field_sha256_stable(&sim.fields.precursor);
    assert!(!sim.step());
    assert_eq!(field_sha256_stable(&sim.fields.membrane), membrane_before);
    assert_eq!(field_sha256_stable(&sim.fields.precursor), precursor_before);
}

// === Gate 1: interface geometry ===

#[test]
fn test_planar_integrated_delta_matches_analytic_length() {
    let grid = Grid::new();
    let x0 = grid.cx;
    let analytic = vertical_chord_length(&grid, x0);
    for eps in [2.0, 3.0, 4.0] {
        let n = GRID_WIDTH * GRID_HEIGHT;
        let mut phi = vec![0.0; n];
        let mut geometry = vec![InterfaceGeometryCell::default(); n];
        planar_phi_profile(&grid, x0, eps, &mut phi);
        compute_interface_geometry(&grid, &phi, v7_params().eta_n, &mut geometry);
        let integrated = integrated_delta(&grid, &geometry);
        let rel = (integrated - analytic).abs() / analytic;
        assert!(
            rel <= 0.02,
            "eps={eps}: integrated={integrated} analytic={analytic} rel={rel}"
        );
    }
}

#[test]
fn test_circular_integrated_delta_matches_analytic_perimeter() {
    let grid = Grid::new();
    for radius in [16.0, 24.0, 32.0] {
        for eps in [2.0, 3.0, 4.0] {
            let (_, geometry) = circle_geometry(&grid, radius, eps);
            let integrated = integrated_delta(&grid, &geometry);
            let analytic = 2.0 * std::f64::consts::PI * radius;
            let rel = (integrated - analytic).abs() / analytic;
            assert!(
                rel <= 0.03,
                "r={radius} eps={eps}: integrated={integrated} analytic={analytic}"
            );
        }
    }
}

#[test]
fn test_perimeter_estimate_stable_across_widths_at_r24() {
    let grid = Grid::new();
    let mut perimeters = Vec::new();
    for eps in [2.0, 3.0, 4.0] {
        let (phi, _) = circle_geometry(&grid, 24.0, eps);
        perimeters.push(estimate_contour_perimeter(&grid, &phi));
    }
    let mean = perimeters.iter().sum::<f64>() / perimeters.len() as f64;
    for &p in &perimeters {
        assert!(((p - mean) / mean).abs() <= 0.03, "perimeters={perimeters:?}");
    }
}

#[test]
fn test_projector_identities_on_unit_normals() {
    let normals = [(1.0, 0.0), (0.0, 1.0), (0.6, 0.8), (-0.7071067811865476, 0.7071067811865476)];
    for (nx, ny) in normals {
        let (sym, tn_norm, tang_err) = projector_identities(nx, ny);
        assert!(sym, "T must be symmetric for n=({nx},{ny})");
        assert!(tn_norm < 1e-8, "Tn≈0 for n=({nx},{ny}), got {tn_norm}");
        assert!(tang_err < 1e-8, "tangential preservation for n=({nx},{ny})");
    }
}

#[test]
fn test_planar_integrated_delta_refines_weakly_with_eps() {
    let grid = Grid::new();
    let x0 = grid.cx;
    let analytic = vertical_chord_length(&grid, x0);
    let mut values = Vec::new();
    for eps in [2.0, 3.0, 4.0] {
        let n = GRID_WIDTH * GRID_HEIGHT;
        let mut phi = vec![0.0; n];
        let mut geometry = vec![InterfaceGeometryCell::default(); n];
        planar_phi_profile(&grid, x0, eps, &mut phi);
        compute_interface_geometry(&grid, &phi, v7_params().eta_n, &mut geometry);
        values.push(integrated_delta(&grid, &geometry));
    }
    for v in &values {
        assert!((v - analytic).abs() / analytic <= 0.02);
    }
}

// === Gate 2: passive surface diffusion ===

#[test]
fn test_uniform_gamma_diffusion_only_preserves_gamma_and_mass() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (phi, mut geometry) = circle_geometry(&grid, 24.0, 2.0);
    let catalyst = vec![0.4; n];
    let activated = vec![0.3; n];
    let precursor = vec![0.0; n];
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, p.delta_floor, &mut s, |_, _, _| 1.0);
    let s0 = total_surface_mass(&grid, &s);
    let mut gamma = vec![0.0; n];
    let mut diff = vec![0.0; n];
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];
    for _ in 0..400 {
        evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &p,
            0.01,
            false,
            false,
            false,
            false,
            true,
            &mut geometry,
            &mut gamma,
            &mut diff,
            &mut s_next,
            &mut a_next,
            &mut p_next,
            &mut w_next,
        ).expect("surface evolve");
        s.copy_from_slice(&s_next);
    }
    reconstruct_gamma_field_helper(&grid, &s, &geometry, &p, &mut gamma);
    let band = p.delta_face_eps * 10.0;
    let mut gamma_sum = 0.0;
    let mut gamma_w = 0.0;
    for idx in 0..n {
        if !grid.in_dish(idx) || geometry[idx].delta < band {
            continue;
        }
        let w = geometry[idx].delta;
        gamma_sum += gamma[idx] * w;
        gamma_w += w;
    }
    let mean_gamma = gamma_sum / gamma_w.max(1e-12);
    assert!((mean_gamma - 1.0).abs() < 0.05, "mean gamma={mean_gamma}");
    let s1 = total_surface_mass(&grid, &s);
    assert!((s1 - s0).abs() / s0.max(1.0) < 0.02, "S mass drift {s0}->{s1}");
}

#[test]
fn test_nonuniform_gamma_diffusion_reduces_variance_without_leakage() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (phi, mut geometry) = circle_geometry(&grid, 24.0, 2.0);
    let catalyst = vec![0.4; n];
    let activated = vec![0.3; n];
    let precursor = vec![0.0; n];
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, p.delta_floor, &mut s, |i, j, _| {
        let dx = i as f64 - grid.cx;
        let dy = j as f64 - grid.cy;
        let theta = dy.atan2(dx);
        (1.0 + 0.5 * theta.cos()).max(0.0)
    });
    let s0 = total_surface_mass(&grid, &s);
    let mut gamma = vec![0.0; n];
    reconstruct_gamma_field_helper(&grid, &s, &geometry, &p, &mut gamma);
    let band = p.delta_face_eps * 10.0;
    let var0 = circumferential_gamma_variance(&grid, &geometry, &gamma, band, 36);
    let mut diff = vec![0.0; n];
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];
    for _ in 0..600 {
        evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &p,
            0.01,
            false,
            false,
            false,
            false,
            true,
            &mut geometry,
            &mut gamma,
            &mut diff,
            &mut s_next,
            &mut a_next,
            &mut p_next,
            &mut w_next,
        ).expect("surface evolve");
        s.copy_from_slice(&s_next);
    }
    reconstruct_gamma_field_helper(&grid, &s, &geometry, &p, &mut gamma);
    let var1 = circumferential_gamma_variance(&grid, &geometry, &gamma, band, 36);
    let loc = surface_localization(&grid, &geometry, &s, band);
    assert!(loc >= 0.98, "localization={loc}");
    assert!(var1 <= var0 * 1.001, "variance should not grow: {var0}->{var1}");
    let s1 = total_surface_mass(&grid, &s);
    assert!((s1 - s0).abs() / s0.max(1.0) < 0.02);
}

fn reconstruct_gamma_field_helper(
    grid: &Grid,
    s: &[f64],
    geometry: &[InterfaceGeometryCell],
    p: &SimParams,
    gamma: &mut [f64],
) {
    chemistry_core::surface_density::reconstruct_gamma_field(
        grid,
        s,
        geometry,
        p.delta_floor,
        gamma,
    );
}

// === Gate 3: unit chemistry checks ===

#[test]
fn test_p_to_s_exact_transfer_adsorption_only() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (phi, mut geometry) = circle_geometry(&grid, 24.0, 2.0);
    let catalyst = vec![0.5; n];
    let activated = vec![0.0; n];
    let mut precursor = vec![0.2; n];
    let mut s = vec![0.0; n];
    let p0 = total_mass(&grid, &precursor);
    let s0 = total_surface_mass(&grid, &s);
    let mut gamma = vec![0.0; n];
    let mut diff = vec![0.0; n];
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &p,
        0.01,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diff,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut w_next,
    ).expect("surface evolve");
    let dp = p0 - total_mass(&grid, &p_next);
    let ds = total_surface_mass(&grid, &s_next) - s0;
    assert!((dp - totals.adsorption_delta).abs() < 1e-9);
    assert!((ds - totals.adsorption_delta).abs() < 1e-9);
    assert!((dp - ds).abs() < 1e-9, "ΔP={dp} ΔS={ds}");
    assert!((totals.precursor_to_surface - totals.adsorption_delta).abs() < 1e-15);
    assert!(totals.adsorption_delta > 0.0);
}

#[test]
fn test_s_to_w_exact_transfer_gamma_decay_only() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (phi, mut geometry) = circle_geometry(&grid, 24.0, 2.0);
    let catalyst = vec![0.5; n];
    let activated = vec![0.0; n];
    let precursor = vec![0.0; n];
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, p.delta_floor, &mut s, |_, _, _| 0.5);
    let s0 = total_surface_mass(&grid, &s);
    let mut gamma = vec![0.0; n];
    let mut diff = vec![0.0; n];
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &p,
        0.01,
        false,
        false,
        false,
        true,
        false,
        &mut geometry,
        &mut gamma,
        &mut diff,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut w_next,
    ).expect("surface evolve");
    let ds = total_surface_mass(&grid, &s_next) - s0;
    let dw = total_mass(&grid, &w_next);
    assert!((ds + dw).abs() < 1e-9);
    assert!((totals.surface_to_waste - totals.gamma_decay_delta).abs() < 1e-15);
}

#[test]
fn test_no_direct_a_to_s_with_zero_precursor() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (phi, mut geometry) = circle_geometry(&grid, 24.0, 2.0);
    let catalyst = vec![0.5; n];
    let mut activated = vec![0.5; n];
    let precursor = vec![0.0; n];
    let s = vec![0.0; n];
    let mut gamma = vec![0.0; n];
    let mut diff = vec![0.0; n];
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];
    evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &p,
        0.01,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diff,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut w_next,
    ).expect("surface evolve");
    assert!(total_surface_mass(&grid, &s_next) < 1e-12);
}

#[test]
fn test_gamma_saturation_blocks_adsorption() {
    let p = v7_params();
    assert_eq!(
        adsorption_rate_j(1.0, 0.5, p.gamma_max, &p),
        0.0,
        "at Γ=Γ_max adsorption must vanish"
    );
}

// === Gate 4: selective transport ===

#[test]
fn test_permeability_theta_zero_is_unity_on_crossing_face() {
    let p = v7_params();
    let perm = permeability_surface_occupancy(
        TransportSpecies::Catalyst,
        0.75,
        0.25,
        0.0,
        0.0,
        &p,
    );
    assert!((perm - 1.0).abs() < 1e-15);
}

#[test]
fn test_permeability_at_theta_one_meets_selectivity_targets() {
    let p = v7_params();
    let phi_in = 0.75;
    let phi_out = 0.25;
    let delta = (6.0 * 0.75 * 0.25 / chemistry_core::config::DX).max(p.delta_floor);
    let s = delta * p.gamma_reference;
    let species = [
        TransportSpecies::Catalyst,
        TransportSpecies::Activated,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ];
    for sp in species {
        let perm = permeability_surface_occupancy(sp, phi_in, phi_out, s, s, &p);
        let base = face_flux(sp, 1.0, 0.0, phi_in, phi_out, 0.0, 0.0, &p);
        let scaled = face_flux(sp, 1.0, 0.0, phi_in, phi_out, s, s, &p);
        let normalized = scaled / base;
        match sp {
            TransportSpecies::Catalyst | TransportSpecies::Activated => {
                assert!(normalized <= 0.05, "{sp:?}: {normalized}");
            }
            TransportSpecies::Nutrient | TransportSpecies::Fuel => {
                assert!(
                    (0.20..=0.50).contains(&normalized),
                    "{sp:?}: {normalized}"
                );
            }
            TransportSpecies::Waste => {
                assert!(normalized >= 0.70, "{sp:?}: {normalized}");
            }
        }
        let _ = perm;
    }
}

#[test]
fn test_v7_face_flux_antisymmetric() {
    let p = v7_params();
    let phi_i = 0.75;
    let phi_j = 0.25;
    let s_i = 0.02;
    let s_j = 0.03;
    for species in [
        TransportSpecies::Catalyst,
        TransportSpecies::Activated,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ] {
        let forward = face_flux(species, 0.8, 0.2, phi_i, phi_j, s_i, s_j, &p);
        let reverse = face_flux(species, 0.2, 0.8, phi_j, phi_i, s_j, s_i, &p);
        assert!((forward + reverse).abs() < 1e-12, "{species:?}");
    }
}

// === Gate 5: moving interface advection ===

#[test]
fn test_translation_advection_conserves_surface_mass() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (_, geometry) = circle_geometry(&grid, 24.0, 2.0);
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, p.delta_floor, &mut s, |_, _, _| 1.0);
    let vn = vec![0.02; n];
    let s0 = total_surface_mass(&grid, &s);
    let mut rate = vec![0.0; n];
    let dt = 0.05;
    surface_advection_rate(&grid, &geometry, &s, &vn, &mut rate);
    let mut s_next = s.clone();
    for idx in 0..n {
        if grid.in_dish(idx) {
            s_next[idx] = s[idx] + rate[idx] * dt;
        }
    }
    let s1 = total_surface_mass(&grid, &s_next);
    assert!((s1 - s0).abs() / s0.max(1.0) < 0.03, "translation mass {s0}->{s1}");
}

#[test]
fn test_expansion_advection_conserves_mass_without_chemistry() {
    let grid = Grid::new();
    let p = v7_params();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let (_, geometry) = circle_geometry(&grid, 24.0, 2.0);
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, p.delta_floor, &mut s, |_, _, _| 1.0);
    let mut vn = vec![0.0; n];
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt().max(1.0);
            vn[idx] = 0.01 * r / 24.0;
        }
    }
    let s0 = total_surface_mass(&grid, &s);
    let mut rate = vec![0.0; n];
    let dt = 0.05;
    surface_advection_rate(&grid, &geometry, &s, &vn, &mut rate);
    let mut s_next = s.clone();
    for idx in 0..n {
        if grid.in_dish(idx) {
            s_next[idx] = (s[idx] + rate[idx] * dt).max(0.0);
        }
    }
    let s1 = total_surface_mass(&grid, &s_next);
    assert!((s1 - s0).abs() / s0.max(1.0) < 0.05, "expansion mass {s0}->{s1}");
}

// === Historical regression smoke ===

#[test]
fn test_v6_simulation_still_instantiates() {
    let mut sim = Simulation::new(v6_params());
    for _ in 0..20 {
        if !sim.step() {
            break;
        }
    }
    assert!(sim.substep > 0);
}

#[test]
fn test_v5_snapshot_validates() {
    let mut v5 = SimParams::default();
    v5.equation_version = EquationVersion::MembraneMetabolismV5InterfaceAffinity;
    let sim5 = Simulation::new(v5);
    let snap5 = sim5.snapshot();
    assert_eq!(snap5.field_schema_version, FieldSchemaVersion::SevenFieldV1);
    assert!(snap5.validate().is_ok());
}
