//! D-025 autonomous surface transport tests (Gate 1 manufactured velocity first).

use chemistry_core::config::{SimParams, DX, GRID_HEIGHT, GRID_WIDTH};
use chemistry_core::grid::Grid;
use chemistry_core::operators::laplacian;
use chemistry_core::phase_field::chemical_potential_local;
use chemistry_core::surface_density::{
    circular_phi_profile, circular_phi_profile_at, circumferential_gamma_variance,
    compute_interface_geometry, estimate_interface_normal_velocity,
    evolve_surface_density, evolve_surface_density_with_vn, grad_phi_magnitude,
    in_interface_velocity_band, reconstruct_gamma_field, seed_surface_from_gamma,
    surface_localization, total_surface_mass, InterfaceGeometryCell,
};

fn v7_geom_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = chemistry_core::config::EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p
}

fn band_mean_vn_weighted(
    grid: &Grid,
    phi_old: &[f64],
    geometry: &[InterfaceGeometryCell],
    vn: &[f64],
    params: &SimParams,
) -> (f64, usize) {
    let w = grid.width;
    let mut w_sum = 0.0;
    let mut vn_w = 0.0;
    let mut count = 0usize;
    for j in 0..grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let grad = grad_phi_magnitude(grid, phi_old, i, j);
            if !in_interface_velocity_band(
                geometry[idx].delta,
                grad,
                params.delta_floor,
                params.interface_grad_min,
            ) {
                continue;
            }
            let wgt = geometry[idx].delta.max(0.0);
            vn_w += vn[idx] * wgt;
            w_sum += wgt;
            count += 1;
        }
    }
    (
        if w_sum > 0.0 { vn_w / w_sum } else { 0.0 },
        count,
    )
}

#[test]
fn test_surface_velocity_from_translation() {
    let params = v7_geom_params();
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi_old = vec![0.0; n];
    let mut phi_next = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut vn = vec![0.0; n];

    let radius = 22.0;
    let eps = 3.0;
    let dt = 0.05;
    let vx = 0.4; // rigid +x translation speed
    let dx = vx * dt;

    circular_phi_profile_at(&grid, grid.cx, grid.cy, radius, eps, &mut phi_old);
    circular_phi_profile_at(&grid, grid.cx + dx, grid.cy, radius, eps, &mut phi_next);
    compute_interface_geometry(&grid, &phi_old, params.eta_n, &mut geometry);

    let diag = estimate_interface_normal_velocity(
        &grid,
        &phi_old,
        &phi_next,
        &geometry,
        dt,
        params.eta_v,
        params.delta_floor,
        params.interface_grad_min,
        &mut vn,
    );
    assert!(diag.band_cell_count > 100, "expected populated interface band");

    let mut rel_err_sum = 0.0;
    let mut rel_count = 0usize;
    let w = grid.width;
    for j in 0..grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let grad = grad_phi_magnitude(&grid, &phi_old, i, j);
            if !in_interface_velocity_band(
                geometry[idx].delta,
                grad,
                params.delta_floor,
                params.interface_grad_min,
            ) {
                assert_eq!(vn[idx], 0.0, "weak-gradient / off-band must be zero");
                continue;
            }
            // True interface velocity is Cartesian (vx,0); normal projection is vx·n.
            let expected = vx * geometry[idx].nx;
            let denom = expected.abs().max(0.05);
            rel_err_sum += ((vn[idx] - expected) / denom).abs();
            rel_count += 1;
        }
    }
    let mean_rel = rel_err_sum / rel_count as f64;
    assert!(
        mean_rel <= 0.05,
        "translation vn mean relative error {mean_rel} exceeds 5%"
    );
    // Sign: mean nx-weighted vn should share sign with vx
    let mut nx_weighted = 0.0;
    let mut wsum = 0.0;
    for idx in 0..n {
        if !grid.in_dish(idx) || vn[idx] == 0.0 {
            continue;
        }
        nx_weighted += vn[idx] * geometry[idx].nx;
        wsum += geometry[idx].nx.abs();
    }
    assert!(wsum > 0.0);
    assert!(
        nx_weighted / wsum > 0.0,
        "translation sign incorrect: nx-weighted vn={}",
        nx_weighted / wsum
    );

    // Tangential loci for +x motion: poles (|nx| small) should have small vn.
    let mut pole_err = 0.0;
    let mut pole_n = 0usize;
    for idx in 0..n {
        if !grid.in_dish(idx) || vn[idx] == 0.0 {
            continue;
        }
        if geometry[idx].nx.abs() < 0.15 && geometry[idx].ny.abs() > 0.85 {
            pole_err += vn[idx].abs();
            pole_n += 1;
        }
    }
    assert!(pole_n > 0, "expected polar interface samples");
    let mean_pole = pole_err / pole_n as f64;
    assert!(
        mean_pole < 0.08 * vx,
        "x-translation appeared as normal motion at poles: {mean_pole}"
    );
}

#[test]
fn test_surface_velocity_from_expansion() {
    let params = v7_geom_params();
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi_old = vec![0.0; n];
    let mut phi_next = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut vn = vec![0.0; n];

    let radius = 22.0;
    let dt = 0.05;
    let v_rad = 0.3;
    for &eps in &[2.0, 3.0, 4.0] {
        circular_phi_profile(&grid, radius, eps, &mut phi_old);
        circular_phi_profile(&grid, radius + v_rad * dt, eps, &mut phi_next);
        compute_interface_geometry(&grid, &phi_old, params.eta_n, &mut geometry);
        let diag = estimate_interface_normal_velocity(
            &grid,
            &phi_old,
            &phi_next,
            &geometry,
            dt,
            params.eta_v,
            params.delta_floor,
            params.interface_grad_min,
            &mut vn,
        );
        let (mean, count) =
            band_mean_vn_weighted(&grid, &phi_old, &geometry, &vn, &params);
        assert!(count > 50, "eps={eps}: empty band");
        // n = ∇φ/|∇φ| points inward; expansion (outward motion) ⇒ v_n < 0.
        let expected = -v_rad;
        let rel = ((mean - expected) / expected.abs()).abs();
        assert!(
            rel <= 0.05,
            "eps={eps}: mean vn {mean} vs {expected}, rel err {rel}"
        );
        // Spatial variation within documented tolerance (std / |mean|).
        let mut var = 0.0;
        let mut w_sum = 0.0;
        for idx in 0..n {
            if vn[idx] != 0.0 {
                let wgt = geometry[idx].delta.max(0.0);
                var += wgt * (vn[idx] - mean).powi(2);
                w_sum += wgt;
            }
        }
        let std = if w_sum > 0.0 {
            (var / w_sum).sqrt()
        } else {
            0.0
        };
        assert!(
            std / v_rad.abs() <= 0.25,
            "eps={eps}: spatial std {std} too large (diag={diag:?})"
        );
    }
}

#[test]
fn test_surface_velocity_from_contraction() {
    let params = v7_geom_params();
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi_old = vec![0.0; n];
    let mut phi_next = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut vn = vec![0.0; n];

    let radius = 22.0;
    let eps = 3.0;
    let dt = 0.05;
    let v_rad = -0.3;
    circular_phi_profile(&grid, radius, eps, &mut phi_old);
    circular_phi_profile(&grid, radius + v_rad * dt, eps, &mut phi_next);
    compute_interface_geometry(&grid, &phi_old, params.eta_n, &mut geometry);
    let _ = estimate_interface_normal_velocity(
        &grid,
        &phi_old,
        &phi_next,
        &geometry,
        dt,
        params.eta_v,
        params.delta_floor,
        params.interface_grad_min,
        &mut vn,
    );
    let (mean, count) = band_mean_vn_weighted(&grid, &phi_old, &geometry, &vn, &params);
    assert!(count > 50);
    // Contraction ⇒ positive v_n with inward-pointing n.
    let expected = -v_rad; // v_rad negative ⇒ expected positive
    assert!(mean > 0.0, "contraction must yield positive vn, got {mean}");
    let rel = ((mean - expected) / expected.abs()).abs();
    assert!(rel <= 0.05, "contraction rel err {rel}, mean={mean}");
    assert!(vn.iter().all(|v| v.is_finite()), "no numerical singularity");
}

#[test]
fn test_static_velocity_zero() {
    let params = v7_geom_params();
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut vn = vec![0.0; n];
    circular_phi_profile(&grid, 22.0, 3.0, &mut phi);
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let diag = estimate_interface_normal_velocity(
        &grid,
        &phi,
        &phi,
        &geometry,
        0.05,
        params.eta_v,
        params.delta_floor,
        params.interface_grad_min,
        &mut vn,
    );
    assert!(diag.max_abs_vn < 1e-12, "static max |vn|={}", diag.max_abs_vn);
    assert!(vn.iter().all(|&v| v == 0.0));
}

#[test]
fn test_weak_gradient_exclusion() {
    let params = v7_geom_params();
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi_old = vec![0.5; n]; // flat → weak gradient
    let mut phi_next = vec![0.51; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut vn = vec![1.0; n]; // prefill to prove overwrite
    compute_interface_geometry(&grid, &phi_old, params.eta_n, &mut geometry);
    let diag = estimate_interface_normal_velocity(
        &grid,
        &phi_old,
        &phi_next,
        &geometry,
        0.05,
        params.eta_v,
        params.delta_floor,
        params.interface_grad_min,
        &mut vn,
    );
    assert_eq!(diag.band_cell_count, 0);
    assert!(vn.iter().all(|&v| v == 0.0));
}

#[test]
fn test_old_state_velocity_discipline() {
    // Estimator must use only φ_old geometry/gradients, not φ_next normals.
    let params = v7_geom_params();
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi_old = vec![0.0; n];
    let mut phi_next = vec![0.0; n];
    let mut geometry_old = vec![InterfaceGeometryCell::default(); n];
    let mut geometry_next = vec![InterfaceGeometryCell::default(); n];
    let mut vn_a = vec![0.0; n];
    let mut vn_b = vec![0.0; n];
    circular_phi_profile(&grid, 20.0, 3.0, &mut phi_old);
    circular_phi_profile(&grid, 22.0, 3.0, &mut phi_next);
    compute_interface_geometry(&grid, &phi_old, params.eta_n, &mut geometry_old);
    compute_interface_geometry(&grid, &phi_next, params.eta_n, &mut geometry_next);
    let _ = estimate_interface_normal_velocity(
        &grid,
        &phi_old,
        &phi_next,
        &geometry_old,
        0.1,
        params.eta_v,
        params.delta_floor,
        params.interface_grad_min,
        &mut vn_a,
    );
    // Passing next geometry must not be possible via API; re-run with old proves determinism.
    let _ = estimate_interface_normal_velocity(
        &grid,
        &phi_old,
        &phi_next,
        &geometry_old,
        0.1,
        params.eta_v,
        params.delta_floor,
        params.interface_grad_min,
        &mut vn_b,
    );
    assert_eq!(vn_a, vn_b);
    // Band membership decided from old geometry/grad only.
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % GRID_WIDTH;
        let j = idx / GRID_WIDTH;
        let grad = grad_phi_magnitude(&grid, &phi_old, i, j);
        let in_band = in_interface_velocity_band(
            geometry_old[idx].delta,
            grad,
            params.delta_floor,
            params.interface_grad_min,
        );
        if !in_band {
            assert_eq!(vn_a[idx], 0.0);
        }
    }
}

// === Gate 2: autonomous passive surface conservation ===

fn allen_cahn_relax_step(
    grid: &Grid,
    phi: &[f64],
    params: &SimParams,
    dt: f64,
    lap: &mut [f64],
    phi_next: &mut [f64],
) {
    laplacian(grid, phi, lap);
    for idx in 0..phi.len() {
        if !grid.in_dish(idx) {
            phi_next[idx] = 0.0;
            continue;
        }
        let mu = chemical_potential_local(phi[idx], lap[idx], params);
        // Gradient flow toward lower free energy (interface relaxation).
        phi_next[idx] = (phi[idx] - dt * mu).clamp(0.0, 1.0);
    }
}

#[test]
fn test_autonomous_s_mass_conservation_shape_relaxation() {
    let mut params = v7_geom_params();
    params.d_gamma = 0.05;
    params.a = 1.0;
    params.kappa = 1.0;
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    let mut phi_next = vec![0.0; n];
    let mut lap = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut s = vec![0.0; n];
    let mut s_next = vec![0.0; n];
    let mut gamma = vec![0.0; n];
    let mut diffusion_rate = vec![0.0; n];
    let mut advection_rate = vec![0.0; n];
    let mut vn = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let mut waste = vec![0.0; n];
    let mut activated_n = activated.clone();
    let mut precursor_n = precursor.clone();
    let mut waste_n = waste.clone();
    let catalyst = vec![0.0; n];

    // Non-circular coherent blob: ellipse-like via stretched distance.
    let w = grid.width;
    for j in 0..grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let x = (i as f64 - grid.cx) / 28.0;
            let y = (j as f64 - grid.cy) / 16.0;
            let r = (x * x + y * y).sqrt();
            phi[idx] = (0.5 * (1.0 - ((r - 1.0) / 0.12).tanh())).clamp(0.0, 1.0);
        }
    }
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| 1.0);
    let s0 = total_surface_mass(&grid, &s);
    assert!(s0 > 0.0);

    let dt = 0.002;
    let steps = 80;
    let mut min_loc = 1.0_f64;
    for _ in 0..steps {
        allen_cahn_relax_step(&grid, &phi, &params, dt, &mut lap, &mut phi_next);
        compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
        let _ = estimate_interface_normal_velocity(
            &grid,
            &phi,
            &phi_next,
            &geometry,
            dt,
            params.eta_v,
            params.delta_floor,
            params.interface_grad_min,
            &mut vn,
        );
        activated_n.copy_from_slice(&activated);
        precursor_n.copy_from_slice(&precursor);
        waste_n.copy_from_slice(&waste);
        let _ = evolve_surface_density_with_vn(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &params,
            dt,
            false,
            false,
            false,
            false,
            true,
            Some(&vn),
            &mut geometry,
            &mut gamma,
            &mut diffusion_rate,
            &mut advection_rate,
            &mut s_next,
            &mut activated_n,
            &mut precursor_n,
            &mut waste_n,
        );
        for v in s_next.iter_mut() {
            *v = v.max(0.0);
        }
        let loc = surface_localization(&grid, &geometry, &s_next, params.delta_floor);
        min_loc = min_loc.min(loc);
        phi.copy_from_slice(&phi_next);
        s.copy_from_slice(&s_next);
    }
    let s1 = total_surface_mass(&grid, &s);
    let drift = ((s1 - s0) / s0).abs();
    assert!(
        drift < 1e-6,
        "S mass not conserved under autonomous motion: drift={drift}, s0={s0}, s1={s1}"
    );
    assert!(min_loc >= 0.95, "localization dropped to {min_loc}");
}

#[test]
fn test_no_trailing_or_leading_s_on_expansion() {
    let mut params = v7_geom_params();
    params.d_gamma = 0.02;
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    let mut phi_next = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut s = vec![0.0; n];
    let mut s_next = vec![0.0; n];
    let mut gamma = vec![0.0; n];
    let mut diffusion_rate = vec![0.0; n];
    let mut advection_rate = vec![0.0; n];
    let mut vn = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let mut waste = vec![0.0; n];
    let mut activated_n = activated.clone();
    let mut precursor_n = precursor.clone();
    let mut waste_n = waste.clone();
    let catalyst = vec![0.0; n];

    let mut radius = 18.0;
    let eps = 3.0;
    let dt = 0.05;
    let v_rad = 0.25;
    circular_phi_profile(&grid, radius, eps, &mut phi);
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| 1.0);
    let s0 = total_surface_mass(&grid, &s);

    for _ in 0..40 {
        circular_phi_profile(&grid, radius + v_rad * dt, eps, &mut phi_next);
        compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
        let _ = estimate_interface_normal_velocity(
            &grid,
            &phi,
            &phi_next,
            &geometry,
            dt,
            params.eta_v,
            params.delta_floor,
            params.interface_grad_min,
            &mut vn,
        );
        activated_n.copy_from_slice(&activated);
        precursor_n.copy_from_slice(&precursor);
        waste_n.copy_from_slice(&waste);
        let _ = evolve_surface_density_with_vn(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &params,
            dt,
            false,
            false,
            false,
            false,
            true,
            Some(&vn),
            &mut geometry,
            &mut gamma,
            &mut diffusion_rate,
            &mut advection_rate,
            &mut s_next,
            &mut activated_n,
            &mut precursor_n,
            &mut waste_n,
        );
        for v in s_next.iter_mut() {
            *v = v.max(0.0);
        }
        // No leading S: outside new interface band, S must stay ~0.
        compute_interface_geometry(&grid, &phi_next, params.eta_n, &mut geometry);
        for idx in 0..n {
            if !grid.in_dish(idx) {
                continue;
            }
            if geometry[idx].delta <= params.delta_floor * 10.0 {
                // Far from interface: trailing residue check on old interior shell is softer;
                // require no brand-new S far outside the advanced front.
                let i = idx % GRID_WIDTH;
                let j = idx / GRID_WIDTH;
                let r = grid.distance_from_center(i, j);
                if r > radius + v_rad * dt + 4.0 * eps {
                    assert!(
                        s_next[idx] < 1e-6,
                        "leading S created at r={r}, s={}",
                        s_next[idx]
                    );
                }
                if r + 4.0 * eps < radius {
                    assert!(
                        s_next[idx] < 1e-4,
                        "trailing S residue at r={r}, s={}",
                        s_next[idx]
                    );
                }
            }
        }
        phi.copy_from_slice(&phi_next);
        s.copy_from_slice(&s_next);
        radius += v_rad * dt;
    }
    let s1 = total_surface_mass(&grid, &s);
    assert!(((s1 - s0) / s0).abs() < 1e-5, "mass drift on expansion");
    // Γ dilutes: mean Γ decreases as interface length grows with fixed S.
    reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
    let mut gsum = 0.0;
    let mut gcnt = 0usize;
    for idx in 0..n {
        if geometry[idx].delta > params.delta_floor {
            gsum += gamma[idx];
            gcnt += 1;
        }
    }
    let mean_g = gsum / gcnt as f64;
    assert!(mean_g < 0.98, "expected expansion dilution, mean_g={mean_g}");
}

#[test]
fn test_contraction_concentration_and_static_d024_equivalence() {
    let mut params = v7_geom_params();
    params.d_gamma = 0.05;
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut s = vec![0.0; n];
    let mut s_next = vec![0.0; n];
    let mut gamma = vec![0.0; n];
    let mut diffusion_rate = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let mut waste = vec![0.0; n];
    let mut activated_n = activated.clone();
    let mut precursor_n = precursor.clone();
    let mut waste_n = waste.clone();
    let catalyst = vec![0.0; n];

    circular_phi_profile(&grid, 24.0, 3.0, &mut phi);
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    // Non-uniform Γ to exercise tangential diffusion variance decrease.
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |i, j, _| {
        let ang = (j as f64 - grid.cy).atan2(i as f64 - grid.cx);
        0.7 + 0.3 * ang.cos()
    });
    reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
    let var0 = circumferential_gamma_variance(&grid, &geometry, &gamma, params.delta_floor, 32);
    let s0 = total_surface_mass(&grid, &s);

    // Static control: vn = 0 path via evolve_surface_density (D-024 equivalence).
    activated_n.copy_from_slice(&activated);
    precursor_n.copy_from_slice(&precursor);
    waste_n.copy_from_slice(&waste);
    let _ = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.05,
        false,
        false,
        false,
        false,
        true,
        &mut geometry,
        &mut gamma,
        &mut diffusion_rate,
        &mut s_next,
        &mut activated_n,
        &mut precursor_n,
        &mut waste_n,
    );
    let s1 = total_surface_mass(&grid, &s_next);
    assert!(((s1 - s0) / s0).abs() < 1e-12);
    reconstruct_gamma_field(&grid, &s_next, &geometry, params.delta_floor, &mut gamma);
    let var1 = circumferential_gamma_variance(&grid, &geometry, &gamma, params.delta_floor, 32);
    assert!(var1 <= var0 * 1.001 + 1e-15, "variance should not increase");

    // Contraction: Γ concentrates with conserved S.
    let mut phi_next = vec![0.0; n];
    let mut vn = vec![0.0; n];
    let mut advection_rate = vec![0.0; n];
    s.copy_from_slice(&s_next);
    let mean_g0 = {
        reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
        let mut gsum = 0.0;
        let mut c = 0usize;
        for idx in 0..n {
            if geometry[idx].delta > params.delta_floor {
                gsum += gamma[idx];
                c += 1;
            }
        }
        gsum / c as f64
    };
    circular_phi_profile(&grid, 20.0, 3.0, &mut phi_next);
    let dt = 0.2;
    // Multi-step contraction via autonomous estimator.
    let mut radius = 24.0;
    for _ in 0..20 {
        circular_phi_profile(&grid, radius - 0.2, 3.0, &mut phi_next);
        compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
        let _ = estimate_interface_normal_velocity(
            &grid,
            &phi,
            &phi_next,
            &geometry,
            dt,
            params.eta_v,
            params.delta_floor,
            params.interface_grad_min,
            &mut vn,
        );
        activated_n.copy_from_slice(&activated);
        precursor_n.copy_from_slice(&precursor);
        waste_n.copy_from_slice(&waste);
        let _ = evolve_surface_density_with_vn(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &params,
            dt,
            false,
            false,
            false,
            false,
            true,
            Some(&vn),
            &mut geometry,
            &mut gamma,
            &mut diffusion_rate,
            &mut advection_rate,
            &mut s_next,
            &mut activated_n,
            &mut precursor_n,
            &mut waste_n,
        );
        for v in s_next.iter_mut() {
            *v = (*v).max(0.0);
        }
        phi.copy_from_slice(&phi_next);
        s.copy_from_slice(&s_next);
        radius -= 0.2;
    }
    let s_final = total_surface_mass(&grid, &s);
    assert!(((s_final - s0) / s0).abs() < 1e-4, "S deleted on contraction");
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
    let mut gsum = 0.0;
    let mut c = 0usize;
    for idx in 0..n {
        if geometry[idx].delta > params.delta_floor {
            gsum += gamma[idx];
            c += 1;
        }
    }
    let mean_g1 = gsum / c as f64;
    assert!(
        mean_g1 > mean_g0,
        "expected contraction concentration {mean_g1} > {mean_g0}"
    );
    let _ = DX; // silence if unused in some builds
}
