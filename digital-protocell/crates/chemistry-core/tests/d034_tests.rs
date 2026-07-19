//! D-034 dual-surface maturation: schema, causality, conservation, capacity.
use chemistry_core::config::{
    EquationVersion, SimParams, SurfaceExchangeIntegrator, NINE_FIELD_COUNT, DX, GRID_HEIGHT,
    GRID_WIDTH,
};
use chemistry_core::d033_analysis::v10_params;
use chemistry_core::d034_analysis::{
    d034_frozen_exchange_kinetics_ok, maturation_material_residual, v11_params,
    D034_ALPHA_FROZEN, D034_BETA_FROZEN, SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED,
};
use chemistry_core::fields::FIELD_NAMES_V11;
use chemistry_core::grid::Grid;
use chemistry_core::snapshot::{FieldSchemaVersion, SnapshotFields};
use chemistry_core::surface_density::{
    apply_maturation_bounded, circular_phi_profile, compute_interface_geometry,
    evolve_surface_density, exchange_activities_dual, exchange_rate_j_dual, seed_surface_from_gamma,
    total_surface_mass, validate_dual_capacity, InterfaceGeometryCell, SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use chemistry_core::Simulation;

fn v11_test_params(k_mature: f64) -> SimParams {
    let mut p = v11_params(k_mature);
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.d_u = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.reactions_enabled = false;
    p
}

fn tiny_dual_interface(
    params: &SimParams,
    theta_u: f64,
    theta_s: f64,
    precursor: f64,
    activated: f64,
) -> (
    Grid,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<InterfaceGeometryCell>,
    Vec<f64>,
    Vec<f64>,
) {
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, 10.0, 2.0, &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let mut s = vec![0.0; n];
    let mut u = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| {
        theta_s * params.gamma_max
    });
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut u, |_, _, _| {
        theta_u * params.gamma_max
    });
    let mut catalyst = vec![0.0; n];
    let mut activated_f = vec![0.0; n];
    let mut precursor_f = vec![0.0; n];
    let waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            catalyst[idx] = 0.4;
            precursor_f[idx] = precursor;
            activated_f[idx] = activated;
        }
    }
    let gamma = vec![0.0; n];
    let diffusion = vec![0.0; n];
    (
        grid,
        phi,
        catalyst,
        activated_f,
        precursor_f,
        s,
        u,
        waste,
        geometry,
        gamma,
        diffusion,
    )
}

#[test]
fn v11_schema_and_dispatch() {
    let p = v11_params(1.0);
    assert_eq!(
        p.equation_version.as_str(),
        "membrane_metabolism_v11_surface_maturation"
    );
    assert!(p.equation_version.is_surface_maturation());
    assert!(!p.equation_version.is_activated_intermediate());
    assert!(p.equation_version.is_nine_field());
    assert!(p.equation_version.is_surface_density());
    assert!(p.equation_version.is_reversible_surface_exchange());
    assert_eq!(p.equation_version.surface_exchange_schema_version(), 5);
    assert_eq!(p.equation_version.surface_maturation_schema_version(), 1);
    assert_eq!(p.equation_version.dual_surface_schema_version(), 1);
    assert_eq!(p.equation_version.activated_intermediate_schema_version(), 0);
    assert_eq!(NINE_FIELD_COUNT, 9);
    assert_eq!(FIELD_NAMES_V11.len(), 9);
    assert!(FIELD_NAMES_V11.contains(&"immature_membrane"));
    assert!(!FIELD_NAMES_V11.contains(&"activated_intermediate"));
    assert_eq!(
        SurfaceExchangeIntegrator::InvariantDomainV2.as_str(),
        SURFACE_EXCHANGE_INTEGRATOR_V2
    );
    assert!(d034_frozen_exchange_kinetics_ok());
    assert!((D034_ALPHA_FROZEN - 0.167).abs() < 5e-3);
    assert!((D034_BETA_FROZEN - 0.00334).abs() < 5e-5);
    assert_eq!(
        SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED,
        "SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED_USE_SURFACE_BOUND_IMMATURE_MEMBRANE"
    );
    let id_a = chemistry_core::canonical_params_bytes(&p);
    let mut p2 = p.clone();
    p2.k_mature *= 1.01;
    let id_b = chemistry_core::canonical_params_bytes(&p2);
    assert_ne!(id_a, id_b, "k_mature must enter candidate hash");
}

#[test]
fn no_u_without_adsorption_creates_u_from_p() {
    let params = v11_test_params(0.0);
    // Keep frozen exchange rates from v11_params; only maturation is off.
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_dual_interface(&params, 0.0, 0.0, 1.0, 0.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut u_next = u.clone();
    let u_before = total_surface_mass(&grid, &u);
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.05,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        Some(&u),
        Some(&mut u_next),
    )
    .expect("evolve");
    let u_after = total_surface_mass(&grid, &u_next);
    assert!(u_after > u_before, "P>0 + exchange must create U");
    assert!(totals.adsorption_delta > 0.0);
    assert_eq!(totals.maturation_delta, 0.0);
    // No mature S without maturation.
    let s_after = total_surface_mass(&grid, &s_next);
    assert!(
        (s_after - total_surface_mass(&grid, &s)).abs() < 1e-12,
        "S must stay zero without maturation"
    );
}

#[test]
fn no_s_without_maturation() {
    let params = v11_test_params(0.0);
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_dual_interface(&params, 0.3, 0.0, 0.5, 1.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut u_next = u.clone();
    let s_before = total_surface_mass(&grid, &s);
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.05,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        Some(&u),
        Some(&mut u_next),
    )
    .expect("evolve");
    assert_eq!(totals.maturation_delta, 0.0);
    assert!((total_surface_mass(&grid, &s_next) - s_before).abs() < 1e-12);
}

#[test]
fn no_maturation_without_a_or_q_c() {
    let params = v11_test_params(2.0);
    // No A.
    let (u1, a1, s1, w1, r1) = apply_maturation_bounded(0.4, 0.0, 0.1, 0.5, 0.4, 0.01, &params);
    assert_eq!(r1, 0.0);
    assert!((u1 - 0.4).abs() < 1e-15);
    assert_eq!(a1, 0.0);
    assert!((s1 - 0.1).abs() < 1e-15);
    assert_eq!(w1, 0.0);
    // q(C)=0.
    let (u2, _, s2, _, r2) = apply_maturation_bounded(0.4, 1.0, 0.1, 0.5, 0.0, 0.01, &params);
    assert_eq!(r2, 0.0);
    assert!((u2 - 0.4).abs() < 1e-15);
    assert!((s2 - 0.1).abs() < 1e-15);
    // No U.
    let (_, _, s3, _, r3) = apply_maturation_bounded(0.0, 1.0, 0.1, 0.5, 0.4, 0.01, &params);
    assert_eq!(r3, 0.0);
    assert!((s3 - 0.1).abs() < 1e-15);
}

#[test]
fn desorption_returns_u_to_p() {
    let mut params = v11_test_params(0.0);
    // Low P, high U ⇒ reverse exchange U→P.
    params.k_exchange_eq = 0.1;
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_dual_interface(&params, 0.8, 0.0, 0.01, 0.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut u_next = u.clone();
    let p_before: f64 = precursor.iter().sum();
    let u_before = total_surface_mass(&grid, &u);
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.05,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        Some(&u),
        Some(&mut u_next),
    )
    .expect("evolve");
    let cell = DX * DX;
    let dp = (p_next.iter().sum::<f64>() - p_before) * cell;
    let du = total_surface_mass(&grid, &u_next) - u_before;
    assert!(totals.adsorption_delta < 0.0, "expected desorption");
    assert!(du < 0.0 && dp > 0.0, "U→P");
    assert!((dp + du).abs() < 1e-9, "dp={dp} du={du}");
}

#[test]
fn maturation_stoichiometry_u_a_to_s_w() {
    let params = v11_test_params(2.0);
    let (u1, a1, s1, w1, r) = apply_maturation_bounded(0.5, 0.4, 0.1, 0.5, 0.4, 0.05, &params);
    assert!(r > 0.0);
    assert!((0.5 - u1 - r).abs() < 1e-12);
    assert!((0.4 - a1 - r).abs() < 1e-12);
    assert!((s1 - 0.1 - r).abs() < 1e-12);
    assert!((w1 - r).abs() < 1e-12);
    let (residual, r2) =
        maturation_material_residual(0.5, 0.4, 0.1, 0.0, 0.5, 0.4, 0.05, &params);
    assert!(residual.abs() < 1e-12, "material residual={residual}");
    assert!((r2 - r).abs() < 1e-12);
}

#[test]
fn theta_total_never_exceeds_one() {
    let mut params = v11_test_params(0.0);
    params.k_exchange = 10.0;
    params.k_exchange_eq = 10.0;
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_dual_interface(&params, 0.4, 0.55, 5.0, 0.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut u_next = u.clone();
    evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.1,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        Some(&u),
        Some(&mut u_next),
    )
    .expect("evolve must respect capacity");
    for idx in 0..s_next.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= params.delta_floor {
            continue;
        }
        validate_dual_capacity(
            p_next[idx],
            u_next[idx],
            s_next[idx],
            d,
            params.gamma_max,
            params.delta_floor,
        )
        .expect("θ_total ≤ 1");
    }
}

#[test]
fn p_u_conservation_and_s_fixed_during_exchange() {
    let params = v11_test_params(0.0);
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_dual_interface(&params, 0.2, 0.3, 1.0, 0.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut u_next = u.clone();
    let p_before: f64 = precursor.iter().sum();
    let u_before = total_surface_mass(&grid, &u);
    let s_before = total_surface_mass(&grid, &s);
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.05,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        Some(&u),
        Some(&mut u_next),
    )
    .expect("evolve");
    let cell = DX * DX;
    let dp = (p_next.iter().sum::<f64>() - p_before) * cell;
    let du = total_surface_mass(&grid, &u_next) - u_before;
    let ds = total_surface_mass(&grid, &s_next) - s_before;
    assert!((dp + du).abs() < 1e-9, "P↔U conservation dp={dp} du={du}");
    assert!(
        ds.abs() < 1e-12,
        "S must stay fixed during P↔U exchange (no maturation); ds={ds}"
    );
    assert!((totals.exchange_net - totals.adsorption_delta).abs() < 1e-12);
    // Gate2 regression: when S=0 the dual activities match single-surface form.
    let (af, ar) = exchange_activities_dual(1.0, 0.2, 0.0, &params);
    let theta_u = 0.2 / params.gamma_max;
    let a_fwd_ref = params.k_exchange_eq * 1.0 * (1.0 - theta_u).max(0.0);
    assert!((af - a_fwd_ref).abs() < 1e-12);
    assert!((ar - theta_u).abs() < 1e-12);
    let _ = exchange_rate_j_dual(1.0, 0.4, 0.2, 0.3, &params);
}

#[test]
fn snapshot_v10_cannot_resume_as_v11() {
    let mut p10 = v10_params(1.0, 1.0, 0.1);
    p10.k_gamma_decay = 0.0;
    p10.d_gamma = 0.0;
    let sim10 = Simulation::new(p10);
    let mut snap = sim10.snapshot();
    assert_eq!(
        snap.field_schema_version,
        FieldSchemaVersion::NineFieldSurfaceDensityV1
    );
    // Attempt to resume a v10 schema as v11 equation.
    snap.equation_version = EquationVersion::MembraneMetabolismV11SurfaceMaturation;
    snap.params.equation_version = EquationVersion::MembraneMetabolismV11SurfaceMaturation;
    let err = snap.validate().unwrap_err();
    assert!(
        err.contains("nine_field_surface_density_v1")
            || err.contains("cannot resume")
            || err.contains("incompatible")
            || err.contains("X→U")
            || err.contains("membrane_metabolism_v11"),
        "{err}"
    );
    // Payload mismatch: v10 payload under v11 schema also rejects.
    let mut p11 = v11_params(1.0);
    p11.k_gamma_decay = 0.0;
    let sim11 = Simulation::new(p11);
    let mut snap11 = sim11.snapshot();
    assert_eq!(
        snap11.field_schema_version,
        FieldSchemaVersion::NineFieldSurfaceMaturationV1
    );
    if let SnapshotFields::NineFieldSurfaceDensity(payload) = sim10.snapshot().fields {
        snap11.fields = SnapshotFields::NineFieldSurfaceDensity(payload);
        let err2 = snap11.validate().unwrap_err();
        assert!(
            err2.contains("incompatible")
                || err2.contains("NineField")
                || err2.contains("nine_field")
                || err2.contains("maturation"),
            "{err2}"
        );
    } else {
        panic!("expected nine-field surface-density payload from v10");
    }
}
