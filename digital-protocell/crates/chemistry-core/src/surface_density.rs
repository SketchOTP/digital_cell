//! D-024 conserved interfacial membrane surface density (v7).
//!
//! Stored field: `S = δ Γ` (Cartesian-grid membrane mass density).
//! Reconstructed: `Γ = S / max(δ, δ_floor)` inside the diffuse interface band.
//! Geometry: `H(φ) = φ²(3−2φ)`, `δ = |∇H(φ)|`, `n = ∇φ / |∇φ|_η`, `T = I − n⊗n`.

use crate::config::{SimParams, DX};
use crate::fields::interior_weight;
use crate::grid::Grid;
use crate::membrane::{membrane_catalyst_saturation, precursor_decay_rate, precursor_synthesis_rate};

/// Default regularization for interface normal.
pub const DEFAULT_ETA_N: f64 = 1e-6;
/// Default floor preventing division by zero when reconstructing Γ.
pub const DEFAULT_DELTA_FLOOR: f64 = 1e-12;
/// Faces with mean δ below this are treated as off-interface (no surface flux).
pub const DEFAULT_DELTA_FACE_EPS: f64 = 1e-14;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SurfaceAccountingTotals {
    pub surface_diffusion_delta: f64,
    pub adsorption_delta: f64,
    pub gamma_decay_delta: f64,
    pub advection_delta: f64,
    pub precursor_synthesis_delta: f64,
    pub precursor_decay_delta: f64,
    /// Exact P→S transfer (same magnitude as adsorption_delta).
    pub precursor_to_surface: f64,
    /// Exact S→W transfer (same magnitude as gamma_decay_delta).
    pub surface_to_waste: f64,
    pub absolute_face_flux: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InterfaceGeometryCell {
    pub h: f64,
    pub delta: f64,
    pub nx: f64,
    pub ny: f64,
}

#[inline]
pub fn smooth_interior_h(phi: f64) -> f64 {
    interior_weight(phi)
}

#[inline]
pub fn reconstruct_gamma(s: f64, delta: f64, delta_floor: f64) -> f64 {
    if delta <= delta_floor {
        0.0
    } else {
        (s / delta).max(0.0)
    }
}

#[inline]
pub fn theta_gamma(gamma: f64, gamma_reference: f64) -> f64 {
    if gamma_reference <= 0.0 {
        return 0.0;
    }
    (gamma / gamma_reference).clamp(0.0, 1.0)
}

/// Central differences for ∇φ and ∇H with no-flux dish mirroring.
pub fn compute_interface_geometry(
    grid: &Grid,
    phi: &[f64],
    eta_n: f64,
    out: &mut [InterfaceGeometryCell],
) {
    let w = grid.width;
    let h = grid.height;
    let inv_2dx = 0.5 / DX;
    let eta2 = eta_n * eta_n;
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                out[idx] = InterfaceGeometryCell::default();
                continue;
            }
            let p = phi[idx];
            let h_c = smooth_interior_h(p);
            let left = sample_phi(grid, phi, i.wrapping_sub(1), j, i, j);
            let right = sample_phi(grid, phi, i + 1, j, i, j);
            let down = sample_phi(grid, phi, i, j.wrapping_sub(1), i, j);
            let up = sample_phi(grid, phi, i, j + 1, i, j);
            let dphix = (right - left) * inv_2dx;
            let dphiy = (up - down) * inv_2dx;
            let hl = smooth_interior_h(left);
            let hr = smooth_interior_h(right);
            let hd = smooth_interior_h(down);
            let hu = smooth_interior_h(up);
            let dhx = (hr - hl) * inv_2dx;
            let dhy = (hu - hd) * inv_2dx;
            let delta = (dhx * dhx + dhy * dhy).sqrt();
            let n_norm = (dphix * dphix + dphiy * dphiy + eta2).sqrt();
            out[idx] = InterfaceGeometryCell {
                h: h_c,
                delta,
                nx: dphix / n_norm,
                ny: dphiy / n_norm,
            };
        }
    }
}

#[inline]
fn sample_phi(grid: &Grid, phi: &[f64], ni: usize, nj: usize, ci: usize, cj: usize) -> f64 {
    let w = grid.width;
    let h = grid.height;
    if ni >= w || nj >= h {
        return phi[Grid::index(w, ci, cj)];
    }
    let nidx = Grid::index(w, ni, nj);
    if !grid.in_dish(nidx) {
        phi[Grid::index(w, ci, cj)]
    } else {
        phi[nidx]
    }
}

/// Reconstruct Γ from S and δ; zero outside the active band.
pub fn reconstruct_gamma_field(
    grid: &Grid,
    s: &[f64],
    geometry: &[InterfaceGeometryCell],
    delta_floor: f64,
    gamma: &mut [f64],
) {
    for idx in 0..s.len() {
        if !grid.in_dish(idx) {
            gamma[idx] = 0.0;
            continue;
        }
        gamma[idx] = reconstruct_gamma(s[idx], geometry[idx].delta, delta_floor);
    }
}

/// Integrated diffuse surface measure ∑ δ Δx Δy.
pub fn integrated_delta(grid: &Grid, geometry: &[InterfaceGeometryCell]) -> f64 {
    let mut total = 0.0;
    let cell = DX * DX;
    for idx in 0..geometry.len() {
        if grid.in_dish(idx) {
            total += geometry[idx].delta * cell;
        }
    }
    total
}

/// Total conserved surface mass ∑ S Δx Δy.
pub fn total_surface_mass(grid: &Grid, s: &[f64]) -> f64 {
    let mut total = 0.0;
    let cell = DX * DX;
    for idx in 0..s.len() {
        if grid.in_dish(idx) {
            total += s[idx] * cell;
        }
    }
    total
}

/// Seed S = δ · Γ on the active interface from a Γ callback (i, j, idx) → Γ.
pub fn seed_surface_from_gamma<F>(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    delta_floor: f64,
    s: &mut [f64],
    mut gamma_at: F,
) where
    F: FnMut(usize, usize, usize) -> f64,
{
    let w = grid.width;
    for j in 0..grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                s[idx] = 0.0;
                continue;
            }
            let d = geometry[idx].delta;
            if d <= delta_floor {
                s[idx] = 0.0;
            } else {
                s[idx] = d * gamma_at(i, j, idx).max(0.0);
            }
        }
    }
}

/// Adsorption volumetric rate density J_ads (before multiplying by δ).
#[inline]
pub fn adsorption_rate_j(
    precursor: f64,
    catalyst: f64,
    gamma: f64,
    params: &SimParams,
) -> f64 {
    params.k_ads
        * precursor.max(0.0)
        * membrane_catalyst_saturation(catalyst, params)
        * (1.0 - gamma / params.gamma_max).max(0.0)
}

/// Surface loss volumetric rate density J_loss (before multiplying by δ).
#[inline]
pub fn gamma_decay_rate_j(gamma: f64, params: &SimParams) -> f64 {
    params.k_gamma_decay * gamma.max(0.0)
}

/// Conservative tangential surface diffusion: ∂S/∂t += ∇·(δ D_Γ T ∇Γ).
///
/// Face fluxes are antisymmetric. Faces with negligible mean δ contribute zero.
pub fn surface_diffusion_rate(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    gamma: &[f64],
    d_gamma: f64,
    delta_face_eps: f64,
    out_rate: &mut [f64],
) -> f64 {
    let w = grid.width;
    let h = grid.height;
    let inv_dx = 1.0 / DX;
    let inv_dx2 = inv_dx * inv_dx;
    out_rate.fill(0.0);
    let mut abs_flux = 0.0;

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            // +x face
            if i + 1 < w {
                let jdx = Grid::index(w, i + 1, j);
                if grid.in_dish(jdx) {
                    let flux = tangential_face_flux(
                        geometry[idx],
                        geometry[jdx],
                        gamma[idx],
                        gamma[jdx],
                        1.0,
                        0.0,
                        d_gamma,
                        delta_face_eps,
                        inv_dx2,
                    );
                    out_rate[idx] -= flux;
                    out_rate[jdx] += flux;
                    abs_flux += flux.abs();
                }
            }
            // +y face
            if j + 1 < h {
                let jdx = Grid::index(w, i, j + 1);
                if grid.in_dish(jdx) {
                    let flux = tangential_face_flux(
                        geometry[idx],
                        geometry[jdx],
                        gamma[idx],
                        gamma[jdx],
                        0.0,
                        1.0,
                        d_gamma,
                        delta_face_eps,
                        inv_dx2,
                    );
                    out_rate[idx] -= flux;
                    out_rate[jdx] += flux;
                    abs_flux += flux.abs();
                }
            }
        }
    }
    abs_flux
}

/// Flux of S from cell i → j across a face with unit Cartesian normal (ex, ey).
///
/// J = δ D_Γ (T ∇Γ); face flux density uses face-averaged δ, n, and ΔΓ/Δx.
#[inline]
fn tangential_face_flux(
    gi: InterfaceGeometryCell,
    gj: InterfaceGeometryCell,
    gamma_i: f64,
    gamma_j: f64,
    ex: f64,
    ey: f64,
    d_gamma: f64,
    delta_face_eps: f64,
    inv_dx2: f64,
) -> f64 {
    let delta_f = gi.delta.min(gj.delta);
    if delta_f <= delta_face_eps {
        return 0.0;
    }
    // Face-averaged normal (renormalized).
    let mut nx = 0.5 * (gi.nx + gj.nx);
    let mut ny = 0.5 * (gi.ny + gj.ny);
    let nlen = (nx * nx + ny * ny).sqrt();
    if nlen > 0.0 {
        nx /= nlen;
        ny /= nlen;
    }
    // Cartesian gradient component along the face normal: (Γ_j − Γ_i)/Δx * e.
    let dgamma_e = (gamma_j - gamma_i) * (DX.recip());
    let grad_x = dgamma_e * ex;
    let grad_y = dgamma_e * ey;
    // T ∇Γ = ∇Γ − (n·∇Γ) n
    let n_dot_g = nx * grad_x + ny * grad_y;
    let tgrad_x = grad_x - n_dot_g * nx;
    let tgrad_y = grad_y - n_dot_g * ny;
    let tgrad_dot_e = tgrad_x * ex + tgrad_y * ey;
    // Divergence uses flux/Δx; rate units match diffuse_constant (÷ Δx again via inv_dx2).
    delta_f * d_gamma * tgrad_dot_e * inv_dx2 * DX
}

/// Conservative surface advection ∂S/∂t += −∇·(S u_Γ) with u_Γ = v_n n.
///
/// `vn` is a per-cell prescribed normal speed (diagnostic Gate 5 only).
pub fn surface_advection_rate(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &[f64],
    vn: &[f64],
    out_rate: &mut [f64],
) -> f64 {
    let w = grid.width;
    let h = grid.height;
    let inv_dx = 1.0 / DX;
    out_rate.fill(0.0);
    let mut abs_flux = 0.0;

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            if i + 1 < w {
                let jdx = Grid::index(w, i + 1, j);
                if grid.in_dish(jdx) {
                    let ux_i = vn[idx] * geometry[idx].nx;
                    let ux_j = vn[jdx] * geometry[jdx].nx;
                    let s_up = if 0.5 * (ux_i + ux_j) >= 0.0 {
                        s[idx]
                    } else {
                        s[jdx]
                    };
                    let flux = s_up * 0.5 * (ux_i + ux_j) * inv_dx;
                    out_rate[idx] -= flux;
                    out_rate[jdx] += flux;
                    abs_flux += flux.abs();
                }
            }
            if j + 1 < h {
                let jdx = Grid::index(w, i, j + 1);
                if grid.in_dish(jdx) {
                    let uy_i = vn[idx] * geometry[idx].ny;
                    let uy_j = vn[jdx] * geometry[jdx].ny;
                    let s_up = if 0.5 * (uy_i + uy_j) >= 0.0 {
                        s[idx]
                    } else {
                        s[jdx]
                    };
                    let flux = s_up * 0.5 * (uy_i + uy_j) * inv_dx;
                    out_rate[idx] -= flux;
                    out_rate[jdx] += flux;
                    abs_flux += flux.abs();
                }
            }
        }
    }
    abs_flux
}

/// Evolve S with surface diffusion + optional adsorption/decay/synthesis (fixed φ).
///
/// Chemical coupling (when enabled):
/// `A → P` (synthesis), `P → W` (precursor decay),
/// `P -= δ J_ads`, `S += δ J_ads`, `S -= δ J_loss`, `W += δ J_loss`.
#[allow(clippy::too_many_arguments)]
pub fn evolve_surface_density(
    grid: &Grid,
    phi: &[f64],
    catalyst: &[f64],
    activated: &[f64],
    precursor: &[f64],
    s: &[f64],
    params: &SimParams,
    dt: f64,
    enable_synthesis: bool,
    enable_adsorption: bool,
    enable_precursor_decay: bool,
    enable_gamma_decay: bool,
    enable_diffusion: bool,
    geometry: &mut [InterfaceGeometryCell],
    gamma: &mut [f64],
    diffusion_rate: &mut [f64],
    s_next: &mut [f64],
    activated_next: &mut [f64],
    precursor_next: &mut [f64],
    waste_next: &mut [f64],
) -> SurfaceAccountingTotals {
    let eta_n = params.eta_n;
    let delta_floor = params.delta_floor;
    compute_interface_geometry(grid, phi, eta_n, geometry);
    reconstruct_gamma_field(grid, s, geometry, delta_floor, gamma);

    let mut totals = SurfaceAccountingTotals::default();
    if enable_diffusion {
        totals.absolute_face_flux = surface_diffusion_rate(
            grid,
            geometry,
            gamma,
            params.d_gamma,
            params.delta_face_eps,
            diffusion_rate,
        );
    } else {
        diffusion_rate.fill(0.0);
    }

    for idx in 0..s.len() {
        if !grid.in_dish(idx) {
            s_next[idx] = 0.0;
            continue;
        }
        let d = geometry[idx].delta;
        let g = gamma[idx];
        let mut ds = if enable_diffusion {
            diffusion_rate[idx] * dt
        } else {
            0.0
        };
        totals.surface_diffusion_delta += if enable_diffusion {
            diffusion_rate[idx] * dt
        } else {
            0.0
        };

        if enable_synthesis {
            let syn = precursor_synthesis_rate(phi[idx], catalyst[idx], activated[idx], params) * dt;
            activated_next[idx] -= syn;
            precursor_next[idx] += syn;
            totals.precursor_synthesis_delta += syn;
        }
        if enable_precursor_decay {
            let dec = precursor_decay_rate(precursor[idx], params) * dt;
            precursor_next[idx] -= dec;
            waste_next[idx] += dec;
            totals.precursor_decay_delta += dec;
        }
        if enable_adsorption {
            let j_ads = adsorption_rate_j(precursor[idx], catalyst[idx], g, params);
            let ads = d * j_ads * dt;
            ds += ads;
            precursor_next[idx] -= ads;
            totals.adsorption_delta += ads;
            totals.precursor_to_surface += ads;
        }
        if enable_gamma_decay {
            let j_loss = gamma_decay_rate_j(g, params);
            let loss = d * j_loss * dt;
            ds -= loss;
            waste_next[idx] += loss;
            totals.gamma_decay_delta += loss;
            totals.surface_to_waste += loss;
        }
        s_next[idx] = s[idx] + ds;
    }
    totals
}

/// Analytic planar tanh profile φ(x) transitioning over width `eps` at `x0`.
pub fn planar_phi_profile(
    grid: &Grid,
    x0: f64,
    eps: f64,
    phi: &mut [f64],
) {
    let w = grid.width;
    for j in 0..grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                phi[idx] = 0.0;
                continue;
            }
            let x = i as f64;
            // φ → 1 for x << x0 (left interior), φ → 0 for x >> x0.
            phi[idx] = (0.5 * (1.0 - ((x - x0) / eps).tanh())).clamp(0.0, 1.0);
        }
    }
}

/// Analytic circular tanh profile of radius `radius` and interface width `eps`.
pub fn circular_phi_profile(
    grid: &Grid,
    radius: f64,
    eps: f64,
    phi: &mut [f64],
) {
    let w = grid.width;
    for j in 0..grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                phi[idx] = 0.0;
                continue;
            }
            let r = grid.distance_from_center(i, j);
            phi[idx] = (0.5 * (1.0 - ((r - radius) / eps).tanh())).clamp(0.0, 1.0);
        }
    }
}

/// Projector identity checks at one cell: T symmetric, Tn≈0, T(tangential)=tangential.
pub fn projector_identities(nx: f64, ny: f64) -> (bool, f64, f64) {
    // T = I − n⊗n
    let txx = 1.0 - nx * nx;
    let txy = -nx * ny;
    let tyy = 1.0 - ny * ny;
    let symmetric = (txy - (-nx * ny)).abs() < 1e-15;
    let tnx = txx * nx + txy * ny;
    let tny = txy * nx + tyy * ny;
    let tn_norm = (tnx * tnx + tny * tny).sqrt();
    // Tangential vector (−ny, nx)
    let tx = -ny;
    let ty = nx;
    let ttx = txx * tx + txy * ty;
    let tty = txy * tx + tyy * ty;
    let tang_err = ((ttx - tx).powi(2) + (tty - ty).powi(2)).sqrt();
    (symmetric, tn_norm, tang_err)
}

/// Contour perimeter estimate: count φ=0.5 crossings along grid edges × DX.
pub fn estimate_contour_perimeter(grid: &Grid, phi: &[f64]) -> f64 {
    let w = grid.width;
    let h = grid.height;
    let mut length = 0.0;
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            if i + 1 < w {
                let jdx = Grid::index(w, i + 1, j);
                if grid.in_dish(jdx) && crosses_half(phi[idx], phi[jdx]) {
                    length += DX;
                }
            }
            if j + 1 < h {
                let jdx = Grid::index(w, i, j + 1);
                if grid.in_dish(jdx) && crosses_half(phi[idx], phi[jdx]) {
                    length += DX;
                }
            }
        }
    }
    length
}

#[inline]
fn crosses_half(a: f64, b: f64) -> bool {
    (a - 0.5) * (b - 0.5) < 0.0
}

/// Localization of S into the δ-supported interface band.
pub fn surface_localization(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    s: &[f64],
    delta_band: f64,
) -> f64 {
    let mut total = 0.0;
    let mut band = 0.0;
    for idx in 0..s.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let mass = s[idx].max(0.0);
        total += mass;
        if geometry[idx].delta >= delta_band {
            band += mass;
        }
    }
    if total <= f64::EPSILON {
        1.0
    } else {
        band / total
    }
}

/// Circumferential variance of Γ around a circular interface (angle-binned).
pub fn circumferential_gamma_variance(
    grid: &Grid,
    geometry: &[InterfaceGeometryCell],
    gamma: &[f64],
    delta_band: f64,
    n_bins: usize,
) -> f64 {
    let mut sums = vec![0.0; n_bins];
    let mut weights = vec![0.0; n_bins];
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) || geometry[idx].delta < delta_band {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let ang = dy.atan2(dx);
            let mut bin = ((ang + std::f64::consts::PI) / (2.0 * std::f64::consts::PI)
                * n_bins as f64)
                .floor() as isize;
            if bin < 0 {
                bin = 0;
            }
            if bin >= n_bins as isize {
                bin = n_bins as isize - 1;
            }
            let b = bin as usize;
            let w = geometry[idx].delta;
            sums[b] += gamma[idx] * w;
            weights[b] += w;
        }
    }
    let mut mean = 0.0;
    let mut wtot = 0.0;
    let mut vals = Vec::new();
    for b in 0..n_bins {
        if weights[b] > 0.0 {
            let v = sums[b] / weights[b];
            vals.push((v, weights[b]));
            mean += v * weights[b];
            wtot += weights[b];
        }
    }
    if wtot <= 0.0 || vals.is_empty() {
        return 0.0;
    }
    mean /= wtot;
    vals.iter()
        .map(|(v, w)| w * (v - mean).powi(2))
        .sum::<f64>()
        / wtot
}
