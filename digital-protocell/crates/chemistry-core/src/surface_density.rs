//! D-024 conserved interfacial membrane surface density (v7).
//!
//! Stored field: `S = δ Γ` (Cartesian-grid membrane mass density).
//! Reconstructed: `Γ = S / max(δ, δ_floor)` inside the diffuse interface band.
//! Geometry: `H(φ) = φ²(3−2φ)`, `δ = |∇H(φ)|`, `n = ∇φ / |∇φ|_η`, `T = I − n⊗n`.

use crate::config::{SimParams, SurfaceExchangeIntegrator, DX};
use crate::fields::interior_weight;
use crate::grid::Grid;
use crate::membrane::{membrane_catalyst_saturation, precursor_decay_rate, precursor_synthesis_rate};

/// Default regularization for interface normal.
pub const DEFAULT_ETA_N: f64 = 1e-6;
/// Default floor preventing division by zero when reconstructing Γ.
pub const DEFAULT_DELTA_FLOOR: f64 = 1e-12;
/// Faces with mean δ below this are treated as off-interface (no surface flux).
pub const DEFAULT_DELTA_FACE_EPS: f64 = 1e-14;

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SurfaceAccountingTotals {
    pub surface_diffusion_delta: f64,
    pub adsorption_delta: f64,
    pub gamma_decay_delta: f64,
    pub advection_delta: f64,
    pub precursor_synthesis_delta: f64,
    pub precursor_decay_delta: f64,
    /// Exact P→S transfer (same magnitude as adsorption_delta for irreversible; net for reversible).
    pub precursor_to_surface: f64,
    /// Exact S→W transfer (same magnitude as gamma_decay_delta).
    pub surface_to_waste: f64,
    pub absolute_face_flux: f64,
    /// D-029 gross forward exchange extent (P→S), ≥ 0.
    pub exchange_forward: f64,
    /// D-029 gross reverse exchange extent (S→P), ≥ 0.
    pub exchange_reverse: f64,
    /// D-029 net exchange into S (= forward − reverse = −ΔP from exchange).
    pub exchange_net: f64,
    /// D-029 integrated exchange dissipation proxy ∑ δ m (a_f − a_r) ln(a_f/a_r) Δt ≥ 0.
    pub exchange_dissipation: f64,
    /// D-032 active assembly extent R for P+A→S+W (≥ 0).
    pub active_assembly: f64,
    /// D-032 activation potential consumed by active assembly (= active_assembly for 1:1 stoichiometry).
    pub active_assembly_activation: f64,
    /// D-033 charge extent P+A→X+W (≥ 0).
    pub charge_delta: f64,
    /// D-033 insertion extent X→S (≥ 0).
    pub insert_delta: f64,
    /// D-033 relaxation extent X→P (≥ 0).
    pub relax_delta: f64,
    /// D-033 activation potential produced into X (= charge_delta).
    pub activation_production: f64,
    /// D-033 activation potential stored in X (net ΔE_X over the step).
    pub activation_storage_delta: f64,
    /// D-033 activation potential consumed by insertion (= insert_delta).
    pub activation_work: f64,
    /// D-033 activation potential dissipated by relaxation (= relax_delta).
    pub activation_dissipation: f64,
    /// D-034 maturation extent R for U+A→S+W (≥ 0); exact S gain and A/W transfer.
    pub maturation_delta: f64,
    /// D-034 net immature-surface (U) tangential diffusion transfer over the step.
    pub immature_diffusion_delta: f64,
    /// D-034 net immature-surface (U) advection transfer over the step.
    pub immature_advection_delta: f64,
}

impl SurfaceAccountingTotals {
    pub fn saturating_sub(self, baseline: Self) -> Self {
        Self {
            surface_diffusion_delta: self.surface_diffusion_delta - baseline.surface_diffusion_delta,
            adsorption_delta: self.adsorption_delta - baseline.adsorption_delta,
            gamma_decay_delta: self.gamma_decay_delta - baseline.gamma_decay_delta,
            advection_delta: self.advection_delta - baseline.advection_delta,
            precursor_synthesis_delta: self.precursor_synthesis_delta
                - baseline.precursor_synthesis_delta,
            precursor_decay_delta: self.precursor_decay_delta - baseline.precursor_decay_delta,
            precursor_to_surface: self.precursor_to_surface - baseline.precursor_to_surface,
            surface_to_waste: self.surface_to_waste - baseline.surface_to_waste,
            absolute_face_flux: self.absolute_face_flux - baseline.absolute_face_flux,
            exchange_forward: self.exchange_forward - baseline.exchange_forward,
            exchange_reverse: self.exchange_reverse - baseline.exchange_reverse,
            exchange_net: self.exchange_net - baseline.exchange_net,
            exchange_dissipation: self.exchange_dissipation - baseline.exchange_dissipation,
            active_assembly: self.active_assembly - baseline.active_assembly,
            active_assembly_activation: self.active_assembly_activation
                - baseline.active_assembly_activation,
            charge_delta: self.charge_delta - baseline.charge_delta,
            insert_delta: self.insert_delta - baseline.insert_delta,
            relax_delta: self.relax_delta - baseline.relax_delta,
            activation_production: self.activation_production - baseline.activation_production,
            activation_storage_delta: self.activation_storage_delta
                - baseline.activation_storage_delta,
            activation_work: self.activation_work - baseline.activation_work,
            activation_dissipation: self.activation_dissipation - baseline.activation_dissipation,
            maturation_delta: self.maturation_delta - baseline.maturation_delta,
            immature_diffusion_delta: self.immature_diffusion_delta
                - baseline.immature_diffusion_delta,
            immature_advection_delta: self.immature_advection_delta
                - baseline.immature_advection_delta,
        }
    }

    pub fn accumulate(&mut self, step: Self) {
        self.surface_diffusion_delta += step.surface_diffusion_delta;
        self.adsorption_delta += step.adsorption_delta;
        self.gamma_decay_delta += step.gamma_decay_delta;
        self.advection_delta += step.advection_delta;
        self.precursor_synthesis_delta += step.precursor_synthesis_delta;
        self.precursor_decay_delta += step.precursor_decay_delta;
        self.precursor_to_surface += step.precursor_to_surface;
        self.surface_to_waste += step.surface_to_waste;
        self.absolute_face_flux += step.absolute_face_flux;
        self.exchange_forward += step.exchange_forward;
        self.exchange_reverse += step.exchange_reverse;
        self.exchange_net += step.exchange_net;
        self.exchange_dissipation += step.exchange_dissipation;
        self.active_assembly += step.active_assembly;
        self.active_assembly_activation += step.active_assembly_activation;
        self.charge_delta += step.charge_delta;
        self.insert_delta += step.insert_delta;
        self.relax_delta += step.relax_delta;
        self.activation_production += step.activation_production;
        self.activation_storage_delta += step.activation_storage_delta;
        self.activation_work += step.activation_work;
        self.activation_dissipation += step.activation_dissipation;
        self.maturation_delta += step.maturation_delta;
        self.immature_diffusion_delta += step.immature_diffusion_delta;
        self.immature_advection_delta += step.immature_advection_delta;
    }
}

/// Cumulative + window-local surface ledgers for v7 (D-027 Gate 0).
///
/// Window-local extents are `cumulative − window_baseline`. On checkpoint restore,
/// call [`SurfaceAccountingState::begin_window_local`] so rates do not depend on
/// pre-checkpoint cumulative history and the restore boundary cannot double-count.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SurfaceAccountingState {
    pub last_step: SurfaceAccountingTotals,
    pub cumulative: SurfaceAccountingTotals,
    pub window_baseline: SurfaceAccountingTotals,
    pub window_baseline_substep: u64,
    pub window_baseline_time: f64,
    pub accepted_steps: u64,
}

impl SurfaceAccountingState {
    pub fn record_accepted(&mut self, step: SurfaceAccountingTotals) {
        self.cumulative.accumulate(step);
        self.last_step = step;
        self.accepted_steps += 1;
    }

    /// Anchor window-local ledgers at the current cumulative (restore / window start).
    pub fn begin_window_local(&mut self, substep: u64, sim_time: f64) {
        self.window_baseline = self.cumulative;
        self.window_baseline_substep = substep;
        self.window_baseline_time = sim_time;
    }

    pub fn window_local(&self) -> SurfaceAccountingTotals {
        self.cumulative.saturating_sub(self.window_baseline)
    }

    /// Mean rates over the open window (`Δextent / Δt`).
    pub fn window_local_rates(&self, sim_time: f64) -> SurfaceAccountingTotals {
        let dt = (sim_time - self.window_baseline_time).max(f64::EPSILON);
        let w = self.window_local();
        SurfaceAccountingTotals {
            surface_diffusion_delta: w.surface_diffusion_delta / dt,
            adsorption_delta: w.adsorption_delta / dt,
            gamma_decay_delta: w.gamma_decay_delta / dt,
            advection_delta: w.advection_delta / dt,
            precursor_synthesis_delta: w.precursor_synthesis_delta / dt,
            precursor_decay_delta: w.precursor_decay_delta / dt,
            precursor_to_surface: w.precursor_to_surface / dt,
            surface_to_waste: w.surface_to_waste / dt,
            absolute_face_flux: w.absolute_face_flux / dt,
            exchange_forward: w.exchange_forward / dt,
            exchange_reverse: w.exchange_reverse / dt,
            exchange_net: w.exchange_net / dt,
            exchange_dissipation: w.exchange_dissipation / dt,
            active_assembly: w.active_assembly / dt,
            active_assembly_activation: w.active_assembly_activation / dt,
            charge_delta: w.charge_delta / dt,
            insert_delta: w.insert_delta / dt,
            relax_delta: w.relax_delta / dt,
            activation_production: w.activation_production / dt,
            activation_storage_delta: w.activation_storage_delta / dt,
            activation_work: w.activation_work / dt,
            activation_dissipation: w.activation_dissipation / dt,
            maturation_delta: w.maturation_delta / dt,
            immature_diffusion_delta: w.immature_diffusion_delta / dt,
            immature_advection_delta: w.immature_advection_delta / dt,
        }
    }
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

/// Surface occupancy θ = Γ / Γ_max (biological [0,1]).
#[inline]
pub fn surface_occupancy_theta(gamma: f64, gamma_max: f64) -> f64 {
    if gamma_max <= 0.0 {
        0.0
    } else {
        (gamma / gamma_max).max(0.0)
    }
}

/// Dimensionless precursor activity p = P / P_reference.
#[inline]
pub fn precursor_activity(precursor: f64, p_reference: f64) -> f64 {
    let pref = if p_reference > 0.0 { p_reference } else { 1.0 };
    precursor.max(0.0) / pref
}

/// Forward / reverse activities for reversible exchange.
#[inline]
pub fn exchange_activities(
    precursor: f64,
    gamma: f64,
    params: &SimParams,
) -> (f64, f64) {
    let p = precursor_activity(precursor, params.p_reference);
    let theta = surface_occupancy_theta(gamma, params.gamma_max);
    let a_forward = params.k_exchange_eq * p * (1.0 - theta).max(0.0);
    let a_reverse = theta;
    (a_forward, a_reverse)
}

/// Nonnegative C-dependent exchange mobility m_exchange = k_exchange × q(C) × Γ_max.
#[inline]
pub fn exchange_mobility(catalyst: f64, params: &SimParams) -> f64 {
    params.k_exchange
        * membrane_catalyst_saturation(catalyst, params)
        * params.gamma_max.max(0.0)
}

/// Dimensionless activated activity a = A / A_reference.
#[inline]
pub fn activated_activity(activated: f64, a_reference: f64) -> f64 {
    let aref = if a_reference > 0.0 { a_reference } else { 1.0 };
    activated.max(0.0) / aref
}

/// Continuous active-assembly flux density (before ×δ):
/// `J_active = k_active × q(C) × a × p × max(0,1−θ)`.
#[inline]
pub fn active_assembly_rate_j(
    precursor: f64,
    activated: f64,
    catalyst: f64,
    gamma: f64,
    params: &SimParams,
) -> f64 {
    if params.k_active <= 0.0 || !params.equation_version.is_activated_surface_assembly() {
        return 0.0;
    }
    let q_c = membrane_catalyst_saturation(catalyst, params);
    let a = activated_activity(activated, params.a_reference);
    let p = precursor_activity(precursor, params.p_reference);
    let theta = surface_occupancy_theta(gamma, params.gamma_max);
    params.k_active * q_c * a * p * (1.0 - theta).max(0.0)
}

/// Active-assembly basis density for rate reconstruction: `q(C) a p (1−θ)`.
#[inline]
pub fn active_assembly_basis_density(
    precursor: f64,
    activated: f64,
    catalyst: f64,
    gamma: f64,
    params: &SimParams,
) -> f64 {
    let q_c = membrane_catalyst_saturation(catalyst, params);
    let a = activated_activity(activated, params.a_reference);
    let p = precursor_activity(precursor, params.p_reference);
    let theta = surface_occupancy_theta(gamma, params.gamma_max);
    q_c * a * p * (1.0 - theta).max(0.0)
}

/// Analytically bounded active transfer for P+A→S+W.
///
/// Returns `(p_next, a_next, s_next, w_delta, r)` with
/// `r = min(δ J_active Δt, P, A, capacity)` and no post-clip.
#[inline]
pub fn apply_active_assembly_bounded(
    precursor: f64,
    activated: f64,
    surface: f64,
    delta: f64,
    catalyst: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64) {
    if delta <= params.delta_floor || dt <= 0.0 {
        return (precursor, activated, surface, 0.0, 0.0);
    }
    let gamma = surface / delta;
    let j = active_assembly_rate_j(precursor, activated, catalyst, gamma, params);
    let r_want = delta * j * dt;
    let capacity = (delta * params.gamma_max.max(0.0) - surface).max(0.0);
    let r = r_want
        .min(precursor.max(0.0))
        .min(activated.max(0.0))
        .min(capacity)
        .max(0.0);
    (
        precursor - r,
        activated - r,
        surface + r,
        r,
        r,
    )
}

/// D-033 charge rate: `r_charge = k_charge × H(φ) × q(C) × P × A`.
#[inline]
pub fn charge_rate(
    phi: f64,
    catalyst: f64,
    precursor: f64,
    activated: f64,
    params: &SimParams,
) -> f64 {
    if params.k_charge <= 0.0 || !params.equation_version.is_activated_intermediate() {
        return 0.0;
    }
    params.k_charge
        * interior_weight(phi)
        * membrane_catalyst_saturation(catalyst, params)
        * precursor.max(0.0)
        * activated.max(0.0)
}

/// D-033 insertion rate: `r_insert = k_insert × δ × X × max(0,1−θ)`.
#[inline]
pub fn insert_rate(
    intermediate: f64,
    surface: f64,
    delta: f64,
    params: &SimParams,
) -> f64 {
    if params.k_insert <= 0.0
        || !params.equation_version.is_activated_intermediate()
        || delta <= params.delta_floor
    {
        return 0.0;
    }
    let gamma = surface / delta;
    let theta = surface_occupancy_theta(gamma, params.gamma_max);
    params.k_insert * delta * intermediate.max(0.0) * (1.0 - theta).max(0.0)
}

/// D-033 relaxation rate: `r_relax = k_relax × X`.
#[inline]
pub fn relax_rate(intermediate: f64, params: &SimParams) -> f64 {
    if params.k_relax <= 0.0 || !params.equation_version.is_activated_intermediate() {
        return 0.0;
    }
    params.k_relax * intermediate.max(0.0)
}

/// D-034/D-035 maturation volumetric rate density `J_mature` (before ×δ) for U+A→S+W.
///
/// v11 linear: `J = k_mature · q(C) · a · Γ_U`
/// v12 catalytic: `J = q · f_A · f_U · (k0 Γ_max + k_cat Γ_S)` with
/// `f_A = a/(K_A+a)`, `f_U = Γ_U/(K_U+Γ_U)`.
#[inline]
pub fn maturation_rate_j(
    activated: f64,
    catalyst: f64,
    gamma_u: f64,
    gamma_s: f64,
    params: &SimParams,
) -> f64 {
    let q_c = membrane_catalyst_saturation(catalyst, params);
    let a = activated_activity(activated, params.a_reference);
    if params.equation_version.is_membrane_catalytic_assembly() {
        let k_a = params.k_a_half.max(0.0);
        let k_u = params.k_u_half.max(0.0);
        let f_a = if a <= 0.0 {
            0.0
        } else {
            a / (k_a + a)
        };
        let gu = gamma_u.max(0.0);
        let f_u = if gu <= 0.0 {
            0.0
        } else {
            gu / (k_u + gu)
        };
        return q_c
            * f_a
            * f_u
            * (params.k_mature_basal.max(0.0) * params.gamma_max.max(0.0)
                + params.k_mature_cat.max(0.0) * gamma_s.max(0.0));
    }
    params.k_mature * q_c * a * gamma_u.max(0.0)
}

/// Bounded U+A→S+W maturation transfer, converting immature U into mature S in place.
///
/// Returns `(u_next, a_next, s_next, w_delta, r)` with
/// `r = min(δ J_mature Δt, U, A)` (no capacity bound). Zero without U, A, or q(C).
#[inline]
pub fn apply_maturation_bounded(
    immature: f64,
    activated: f64,
    surface: f64,
    delta: f64,
    catalyst: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64) {
    let v12 = params.equation_version.is_membrane_catalytic_assembly();
    let has_rate = if v12 {
        params.k_mature_basal > 0.0 || params.k_mature_cat > 0.0
    } else {
        params.k_mature > 0.0
    };
    if !has_rate
        || !params.equation_version.is_surface_maturation()
        || delta <= params.delta_floor
        || dt <= 0.0
    {
        return (immature, activated, surface, 0.0, 0.0);
    }
    let gamma_u = immature.max(0.0) / delta;
    let gamma_s = surface.max(0.0) / delta;
    let j = maturation_rate_j(activated, catalyst, gamma_u, gamma_s, params);
    let r_want = delta * j * dt;
    let r = r_want
        .min(immature.max(0.0))
        .min(activated.max(0.0))
        .max(0.0);
    (immature - r, activated - r, surface + r, r, r)
}

/// Bounded charge transfer P+A→X+W.
///
/// Returns `(p, a, x, w_delta, r)` with `r = min(r_charge·dt, P, A)`.
#[inline]
pub fn apply_charge_bounded(
    phi: f64,
    catalyst: f64,
    precursor: f64,
    activated: f64,
    intermediate: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64) {
    if dt <= 0.0 {
        return (precursor, activated, intermediate, 0.0, 0.0);
    }
    let r_want = charge_rate(phi, catalyst, precursor, activated, params) * dt;
    let r = r_want
        .min(precursor.max(0.0))
        .min(activated.max(0.0))
        .max(0.0);
    (
        precursor - r,
        activated - r,
        intermediate + r,
        r,
        r,
    )
}

/// Bounded insertion X→S (consumes stored activation; does not consume A).
///
/// Returns `(x, s, r)` with `r = min(r_insert·dt, X, capacity)`.
#[inline]
pub fn apply_insert_bounded(
    intermediate: f64,
    surface: f64,
    delta: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64) {
    if dt <= 0.0 || delta <= params.delta_floor {
        return (intermediate, surface, 0.0);
    }
    let r_want = insert_rate(intermediate, surface, delta, params) * dt;
    let capacity = (delta * params.gamma_max.max(0.0) - surface).max(0.0);
    let r = r_want
        .min(intermediate.max(0.0))
        .min(capacity)
        .max(0.0);
    (intermediate - r, surface + r, r)
}

/// Bounded relaxation X→P (dissipates stored activation; no membrane, no waste).
///
/// Returns `(x, p, r)` with `r = min(r_relax·dt, X)`.
#[inline]
pub fn apply_relax_bounded(
    intermediate: f64,
    precursor: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64) {
    if dt <= 0.0 {
        return (intermediate, precursor, 0.0);
    }
    let r_want = relax_rate(intermediate, params) * dt;
    let r = r_want.min(intermediate.max(0.0)).max(0.0);
    (intermediate - r, precursor + r, r)
}

/// Sequential charge → insert → relax with local bounds.
///
/// Returns `(p, a, x, s, w_delta, charge, insert, relax)`.
#[inline]
pub fn apply_activated_intermediate_bounded(
    phi: f64,
    catalyst: f64,
    precursor: f64,
    activated: f64,
    intermediate: f64,
    surface: f64,
    delta: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
    let (p1, a1, x1, w_delta, r_charge) =
        apply_charge_bounded(phi, catalyst, precursor, activated, intermediate, dt, params);
    let (x2, s2, r_insert) = apply_insert_bounded(x1, surface, delta, dt, params);
    let (x3, p3, r_relax) = apply_relax_bounded(x2, p1, dt, params);
    (p3, a1, x3, s2, w_delta, r_charge, r_insert, r_relax)
}

/// Net volumetric exchange flux density J_exchange = J_forward − J_reverse (before ×δ).
/// Positive ⇒ P→S (adsorption); negative ⇒ S→P (desorption).
#[inline]
pub fn exchange_rate_j(
    precursor: f64,
    catalyst: f64,
    gamma: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64) {
    let (a_forward, a_reverse) = exchange_activities(precursor, gamma, params);
    let m = exchange_mobility(catalyst, params);
    let j_forward = m * a_forward;
    let j_reverse = m * a_reverse;
    let j_net = j_forward - j_reverse;
    (j_net, j_forward, j_reverse, a_forward, a_reverse)
}

/// Exchange affinity A_exchange = ln(a_forward / a_reverse) with safe limits.
#[inline]
pub fn exchange_affinity(a_forward: f64, a_reverse: f64) -> f64 {
    const EPS: f64 = 1e-30;
    if a_forward <= 0.0 && a_reverse <= 0.0 {
        return 0.0;
    }
    if a_forward <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if a_reverse <= 0.0 {
        return f64::INFINITY;
    }
    (a_forward.max(EPS)).ln() - (a_reverse.max(EPS)).ln()
}

/// Discrete dissipation inequality: (a_f − a_r)(ln a_f − ln a_r) ≥ 0 when both > 0.
#[inline]
pub fn exchange_dissipation_density(a_forward: f64, a_reverse: f64, mobility: f64) -> f64 {
    if mobility <= 0.0 {
        return 0.0;
    }
    if a_forward <= 0.0 || a_reverse <= 0.0 {
        // One-sided: flux goes toward the nonzero activity; treat as nonnegative limit.
        return mobility * (a_forward - a_reverse).abs() * 0.0;
    }
    let da = a_forward - a_reverse;
    let dln = a_forward.ln() - a_reverse.ln();
    mobility * da * dln
}

/// Rejection reason for a proposed exchange substep (no buffer swap on reject).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExchangeReject {
    NegPrecursor,
    NegSurface,
    CapacityExceeded,
    NonfiniteFlux,
    NonfiniteAffinity,
    DissipationViolation,
}

/// Tolerance used for positivity / capacity gates (not a physics clip).
pub const EXCHANGE_BOUND_TOLERANCE: f64 = 1e-12;
/// Soft tolerance for J·A ≥ −tol dissipation gate.
pub const EXCHANGE_DISSIPATION_TOLERANCE: f64 = 1e-9;

/// Validate a proposed cell update under reversible exchange (no clipping).
#[inline]
pub fn validate_exchange_cell(
    p_next: f64,
    s_next: f64,
    delta: f64,
    gamma_max: f64,
    delta_floor: f64,
    a_forward: f64,
    a_reverse: f64,
    j_net: f64,
) -> Result<(), ExchangeReject> {
    if !p_next.is_finite() || !s_next.is_finite() || !j_net.is_finite() {
        return Err(ExchangeReject::NonfiniteFlux);
    }
    if p_next < -EXCHANGE_BOUND_TOLERANCE {
        return Err(ExchangeReject::NegPrecursor);
    }
    if s_next < -EXCHANGE_BOUND_TOLERANCE {
        return Err(ExchangeReject::NegSurface);
    }
    if delta > delta_floor {
        let g_next = reconstruct_gamma(s_next, delta, delta_floor);
        let theta_next = surface_occupancy_theta(g_next, gamma_max);
        if theta_next > 1.0 + EXCHANGE_BOUND_TOLERANCE {
            return Err(ExchangeReject::CapacityExceeded);
        }
    }
    let aff = exchange_affinity(a_forward, a_reverse);
    if !aff.is_finite() && a_forward > 0.0 && a_reverse > 0.0 {
        return Err(ExchangeReject::NonfiniteAffinity);
    }
    // J × A ≥ −tol when both activities are strictly positive.
    if a_forward > 0.0 && a_reverse > 0.0 {
        let product = j_net * aff;
        if product < -EXCHANGE_DISSIPATION_TOLERANCE {
            return Err(ExchangeReject::DissipationViolation);
        }
    }
    Ok(())
}

/// D-031 integrator schema identity (v2 invariant domain).
pub const SURFACE_EXCHANGE_INTEGRATOR_V2: &str = "surface_exchange_integrator_v2_invariant_domain";

/// Floor below which local surface capacity disables exchange for the substep.
pub const SURFACE_CAPACITY_FLOOR: f64 = 1e-18;

/// Root tolerance for invariant-domain exchange solve (tighter than material accounting).
pub const INVARIANT_EXCHANGE_SOLVER_TOL: f64 = 1e-14;

/// Max bisection iterations before declaring a local solve failure.
pub const INVARIANT_EXCHANGE_MAX_ITERS: u32 = 48;

/// Diagnostics from one local invariant-domain exchange solve.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InvariantExchangeSolveInfo {
    pub iterations: u32,
    pub bracket_lo: f64,
    pub bracket_hi: f64,
    pub s_next: f64,
    pub p_next: f64,
    pub residual: f64,
    pub used_newton: bool,
}

/// Scalar exchange RHS F(S) with frozen δ, q(C), T, C_surface, kinetics.
///
/// Matches: `dS/dt = δ · k · q · Γ_max · (K p (1−θ) − θ)` with
/// `p = (T−S)/P_ref`, `θ = S/C_surface`, `C_surface = δ Γ_max`.
#[inline]
pub fn exchange_scalar_f(
    s: f64,
    t_inventory: f64,
    c_surface: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
) -> f64 {
    if c_surface <= SURFACE_CAPACITY_FLOOR || delta <= 0.0 {
        return 0.0;
    }
    let pref = if p_reference > 0.0 { p_reference } else { 1.0 };
    let p = ((t_inventory - s) / pref).max(0.0);
    let theta = (s / c_surface).clamp(0.0, 1.0);
    let j = k_exchange * q_c * gamma_max.max(0.0) * (k_eq * p * (1.0 - theta) - theta);
    delta * j
}

/// Continuous exchange time-derivative of S at (P, S) with frozen δ and q(C).
#[inline]
pub fn exchange_ds_dt(
    precursor: f64,
    surface: f64,
    delta: f64,
    q_c: f64,
    params: &SimParams,
) -> f64 {
    let c_surface = delta * params.gamma_max.max(0.0);
    let t_inv = precursor.max(0.0) + surface.max(0.0);
    exchange_scalar_f(
        surface,
        t_inv,
        c_surface,
        delta,
        q_c,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    )
}

/// Exact positive biological turnover: S_after = S_before × exp(−λ_Γ dt).
#[inline]
pub fn apply_turnover_exact(s_before: f64, lambda_gamma: f64, dt: f64) -> (f64, f64) {
    if !(s_before > 0.0) || !(dt > 0.0) || !(lambda_gamma > 0.0) {
        return (s_before.max(0.0), 0.0);
    }
    let s_after = s_before * (-lambda_gamma * dt).exp();
    let dw = s_before - s_after;
    (s_after.max(0.0), dw.max(0.0))
}

/// Backward-Euler exchange on `[0, min(T, C_surface)]` via safeguarded bisection.
pub fn solve_exchange_backward_euler(
    s_old: f64,
    t_inventory: f64,
    c_surface: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
    dt: f64,
) -> Result<InvariantExchangeSolveInfo, ExchangeReject> {
    if !t_inventory.is_finite()
        || !s_old.is_finite()
        || !dt.is_finite()
        || dt < 0.0
        || !c_surface.is_finite()
    {
        return Err(ExchangeReject::NonfiniteFlux);
    }
    if c_surface <= SURFACE_CAPACITY_FLOOR {
        let s_next = s_old.clamp(0.0, t_inventory.max(0.0));
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: 0.0,
            bracket_hi: 0.0,
            s_next,
            p_next: (t_inventory - s_next).max(0.0),
            residual: 0.0,
            used_newton: false,
        });
    }
    let hi = t_inventory.min(c_surface).max(0.0);
    let lo = 0.0;
    if hi <= lo {
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: lo,
            bracket_hi: hi,
            s_next: 0.0,
            p_next: t_inventory.max(0.0),
            residual: 0.0,
            used_newton: false,
        });
    }
    let g = |s: f64| {
        s - s_old
            - dt
                * exchange_scalar_f(
                    s,
                    t_inventory,
                    c_surface,
                    delta,
                    q_c,
                    k_exchange,
                    k_eq,
                    p_reference,
                    gamma_max,
                )
    };
    let mut a = lo;
    let mut b = hi;
    let mut ga = g(a);
    let gb0 = g(b);
    if ga > 0.0 && gb0 > 0.0 {
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: a,
            bracket_hi: b,
            s_next: lo,
            p_next: (t_inventory - lo).max(0.0),
            residual: ga.abs(),
            used_newton: false,
        });
    }
    if ga < 0.0 && gb0 < 0.0 {
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: a,
            bracket_hi: b,
            s_next: hi,
            p_next: (t_inventory - hi).max(0.0),
            residual: gb0.abs(),
            used_newton: false,
        });
    }
    let mut gb = gb0;
    let mut used_newton = false;
    let mut x = 0.5 * (a + b);
    let mut iters = 0u32;
    for _ in 0..INVARIANT_EXCHANGE_MAX_ITERS {
        iters += 1;
        let gx = g(x);
        if gx.abs() <= INVARIANT_EXCHANGE_SOLVER_TOL
            || (b - a) <= INVARIANT_EXCHANGE_SOLVER_TOL * (1.0 + hi)
        {
            let s_next = x.clamp(lo, hi);
            let p_next = t_inventory - s_next;
            if p_next < -EXCHANGE_BOUND_TOLERANCE {
                return Err(ExchangeReject::NegPrecursor);
            }
            if s_next < -EXCHANGE_BOUND_TOLERANCE {
                return Err(ExchangeReject::NegSurface);
            }
            return Ok(InvariantExchangeSolveInfo {
                iterations: iters,
                bracket_lo: a,
                bracket_hi: b,
                s_next: s_next.max(0.0),
                p_next: p_next.max(0.0),
                residual: gx.abs(),
                used_newton,
            });
        }
        let eps = (1e-10_f64).max(1e-8 * hi);
        let s_hi = (x + eps).min(hi);
        let s_lo = (x - eps).max(lo);
        let denom = (s_hi - s_lo).max(eps);
        let df = (exchange_scalar_f(
            s_hi,
            t_inventory,
            c_surface,
            delta,
            q_c,
            k_exchange,
            k_eq,
            p_reference,
            gamma_max,
        ) - exchange_scalar_f(
            s_lo,
            t_inventory,
            c_surface,
            delta,
            q_c,
            k_exchange,
            k_eq,
            p_reference,
            gamma_max,
        )) / denom;
        let dg = 1.0 - dt * df;
        if dg.abs() > 1e-30 {
            let x_n = x - gx / dg;
            if x_n >= a && x_n <= b && x_n.is_finite() {
                x = x_n;
                used_newton = true;
                let gx_n = g(x);
                if ga.signum() != gx_n.signum() && !(ga == 0.0 && gx_n == 0.0) {
                    b = x;
                    gb = gx_n;
                } else {
                    a = x;
                    ga = gx_n;
                }
                continue;
            }
        }
        if ga.signum() != gx.signum() && !(ga == 0.0 && gx == 0.0) {
            b = x;
            gb = gx;
        } else {
            a = x;
            ga = gx;
        }
        let _ = gb;
        x = 0.5 * (a + b);
    }
    Err(ExchangeReject::NonfiniteFlux)
}

/// D-034 dual-surface forward/reverse activities for passive P↔U exchange.
///
/// `a_forward = K_eq · p · max(0, 1 − θ_total)`, `a_reverse = θ_U`,
/// with `θ_total = θ_U + θ_S` (mature S occupies shared capacity but does not desorb).
/// When `Γ_S = 0` this reduces exactly to the single-surface P↔S law (Gate2 regression).
#[inline]
pub fn exchange_activities_dual(
    precursor: f64,
    gamma_u: f64,
    gamma_s: f64,
    params: &SimParams,
) -> (f64, f64) {
    let p = precursor_activity(precursor, params.p_reference);
    let theta_u = surface_occupancy_theta(gamma_u, params.gamma_max);
    let theta_s = surface_occupancy_theta(gamma_s, params.gamma_max);
    let a_forward = params.k_exchange_eq * p * (1.0 - (theta_u + theta_s)).max(0.0);
    let a_reverse = theta_u;
    (a_forward, a_reverse)
}

/// D-034 net volumetric P↔U exchange flux density (before ×δ), `S` fixed.
/// Positive ⇒ P→U (adsorption); negative ⇒ U→P (desorption).
#[inline]
pub fn exchange_rate_j_dual(
    precursor: f64,
    catalyst: f64,
    gamma_u: f64,
    gamma_s: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64) {
    let (a_forward, a_reverse) = exchange_activities_dual(precursor, gamma_u, gamma_s, params);
    let m = exchange_mobility(catalyst, params);
    let j_forward = m * a_forward;
    let j_reverse = m * a_reverse;
    (j_forward - j_reverse, j_forward, j_reverse, a_forward, a_reverse)
}

/// D-034 scalar exchange RHS `F(U)` with frozen δ, q(C), inventory T=P+U, and fixed S.
///
/// `dU/dt = δ k q Γ_max (K p (1−θ_total) − θ_U)` with `p=(T−U)/P_ref`,
/// `θ_U = U/C_surface`, `θ_total = (U + S_fixed)/C_surface`, `C_surface = δ Γ_max`.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn exchange_scalar_f_dual(
    u: f64,
    t_inventory: f64,
    s_fixed: f64,
    c_surface: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
) -> f64 {
    if c_surface <= SURFACE_CAPACITY_FLOOR || delta <= 0.0 {
        return 0.0;
    }
    let pref = if p_reference > 0.0 { p_reference } else { 1.0 };
    let p = ((t_inventory - u) / pref).max(0.0);
    let theta_u = (u / c_surface).clamp(0.0, 1.0);
    let theta_total = ((u + s_fixed.max(0.0)) / c_surface).max(0.0);
    let free = (1.0 - theta_total).max(0.0);
    let j = k_exchange * q_c * gamma_max.max(0.0) * (k_eq * p * free - theta_u);
    delta * j
}

/// D-034 invariant-domain backward-Euler solve for U on `[0, min(T, C_surface − S_fixed)]`.
///
/// `S` is held fixed during the exchange substep so `θ_total` cannot exceed 1.
/// Returns `s_next = U_next` and `p_next = T − U_next` (inventory conserved).
#[allow(clippy::too_many_arguments)]
pub fn solve_exchange_backward_euler_dual(
    u_old: f64,
    t_inventory: f64,
    s_fixed: f64,
    c_surface: f64,
    delta: f64,
    q_c: f64,
    k_exchange: f64,
    k_eq: f64,
    p_reference: f64,
    gamma_max: f64,
    dt: f64,
) -> Result<InvariantExchangeSolveInfo, ExchangeReject> {
    if !t_inventory.is_finite()
        || !u_old.is_finite()
        || !dt.is_finite()
        || dt < 0.0
        || !c_surface.is_finite()
        || !s_fixed.is_finite()
    {
        return Err(ExchangeReject::NonfiniteFlux);
    }
    let cap = (c_surface - s_fixed.max(0.0)).max(0.0);
    if cap <= SURFACE_CAPACITY_FLOOR {
        let u_next = u_old.clamp(0.0, t_inventory.max(0.0));
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: 0.0,
            bracket_hi: 0.0,
            s_next: u_next,
            p_next: (t_inventory - u_next).max(0.0),
            residual: 0.0,
            used_newton: false,
        });
    }
    let hi = t_inventory.min(cap).max(0.0);
    let lo = 0.0;
    if hi <= lo {
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: lo,
            bracket_hi: hi,
            s_next: 0.0,
            p_next: t_inventory.max(0.0),
            residual: 0.0,
            used_newton: false,
        });
    }
    let f = |u: f64| {
        exchange_scalar_f_dual(
            u,
            t_inventory,
            s_fixed,
            c_surface,
            delta,
            q_c,
            k_exchange,
            k_eq,
            p_reference,
            gamma_max,
        )
    };
    let g = |u: f64| u - u_old - dt * f(u);
    let mut a = lo;
    let mut b = hi;
    let mut ga = g(a);
    let gb0 = g(b);
    if ga > 0.0 && gb0 > 0.0 {
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: a,
            bracket_hi: b,
            s_next: lo,
            p_next: (t_inventory - lo).max(0.0),
            residual: ga.abs(),
            used_newton: false,
        });
    }
    if ga < 0.0 && gb0 < 0.0 {
        return Ok(InvariantExchangeSolveInfo {
            iterations: 0,
            bracket_lo: a,
            bracket_hi: b,
            s_next: hi,
            p_next: (t_inventory - hi).max(0.0),
            residual: gb0.abs(),
            used_newton: false,
        });
    }
    let mut x = 0.5 * (a + b);
    let mut iters = 0u32;
    for _ in 0..INVARIANT_EXCHANGE_MAX_ITERS {
        iters += 1;
        let gx = g(x);
        if gx.abs() <= INVARIANT_EXCHANGE_SOLVER_TOL
            || (b - a) <= INVARIANT_EXCHANGE_SOLVER_TOL * (1.0 + hi)
        {
            let u_next = x.clamp(lo, hi);
            let p_next = t_inventory - u_next;
            if p_next < -EXCHANGE_BOUND_TOLERANCE {
                return Err(ExchangeReject::NegPrecursor);
            }
            if u_next < -EXCHANGE_BOUND_TOLERANCE {
                return Err(ExchangeReject::NegSurface);
            }
            return Ok(InvariantExchangeSolveInfo {
                iterations: iters,
                bracket_lo: a,
                bracket_hi: b,
                s_next: u_next.max(0.0),
                p_next: p_next.max(0.0),
                residual: gx.abs(),
                used_newton: false,
            });
        }
        if ga.signum() != gx.signum() && !(ga == 0.0 && gx == 0.0) {
            b = x;
        } else {
            a = x;
            ga = gx;
        }
        x = 0.5 * (a + b);
    }
    Err(ExchangeReject::NonfiniteFlux)
}

/// D-034 dual-surface positivity + shared-capacity validation (θ_U + θ_S ≤ 1).
#[inline]
pub fn validate_dual_capacity(
    precursor: f64,
    immature: f64,
    mature: f64,
    delta: f64,
    gamma_max: f64,
    delta_floor: f64,
) -> Result<(), ExchangeReject> {
    if !precursor.is_finite() || !immature.is_finite() || !mature.is_finite() {
        return Err(ExchangeReject::NonfiniteFlux);
    }
    if precursor < -EXCHANGE_BOUND_TOLERANCE {
        return Err(ExchangeReject::NegPrecursor);
    }
    if immature < -EXCHANGE_BOUND_TOLERANCE || mature < -EXCHANGE_BOUND_TOLERANCE {
        return Err(ExchangeReject::NegSurface);
    }
    if delta > delta_floor {
        let c_surface = delta * gamma_max.max(0.0);
        if c_surface > SURFACE_CAPACITY_FLOOR {
            let theta_total = (immature.max(0.0) + mature.max(0.0)) / c_surface;
            if theta_total > 1.0 + EXCHANGE_BOUND_TOLERANCE {
                return Err(ExchangeReject::CapacityExceeded);
            }
        }
    }
    Ok(())
}

/// Propose an explicit Euler exchange update (Gate 0 / V1). No clipping.
#[inline]
pub fn propose_explicit_exchange(
    precursor: f64,
    surface: f64,
    delta: f64,
    catalyst: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64, f64, f64, f64, f64) {
    let g = if delta > params.delta_floor {
        surface / delta
    } else {
        0.0
    };
    let (j_net, j_fwd, j_rev, a_fwd, a_rev) = exchange_rate_j(precursor, catalyst, g, params);
    let xfer = delta * j_net * dt;
    (
        precursor - xfer,
        surface + xfer,
        xfer,
        j_fwd,
        j_rev,
        exchange_affinity(a_fwd, a_rev),
    )
}

/// Gate 0 continuous invariant signs at physical boundaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvariantBoundarySigns {
    pub dp_at_p0: f64,
    pub ds_at_s0: f64,
    pub ds_at_theta1: f64,
    pub dp_prefailure: f64,
    pub ds_prefailure: f64,
    pub continuous_inward: bool,
}

/// Evaluate continuous exchange vector-field signs for invariant classification.
pub fn classify_exchange_invariant_field(
    p: f64,
    s: f64,
    delta: f64,
    q_c: f64,
    params: &SimParams,
) -> InvariantBoundarySigns {
    let c_surface = delta * params.gamma_max.max(0.0);
    let t = p.max(0.0) + s.max(0.0);
    let ds_p0 = exchange_scalar_f(
        s.min(c_surface),
        s.min(c_surface),
        c_surface,
        delta,
        q_c,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    );
    let dp_at_p0 = -ds_p0;
    let ds_at_s0 = exchange_scalar_f(
        0.0,
        t,
        c_surface,
        delta,
        q_c,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    );
    let s_cap = c_surface.min(t.max(c_surface));
    let ds_at_theta1 = exchange_scalar_f(
        c_surface.min(t.max(0.0)).max(0.0),
        t.max(c_surface),
        c_surface,
        delta,
        q_c,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    );
    let _ = s_cap;
    let ds_pre = exchange_ds_dt(p, s, delta, q_c, params);
    let dp_pre = -ds_pre;
    let theta = if c_surface > 0.0 { s / c_surface } else { 0.0 };
    let continuous_inward = dp_at_p0 >= -1e-14
        && ds_at_s0 >= -1e-14
        && ds_at_theta1 <= 1e-14
        && if theta >= 1.0 - 1e-9 {
            ds_pre <= 1e-14
        } else if p <= 1e-14 {
            dp_pre >= -1e-14
        } else if s <= 1e-14 {
            ds_pre >= -1e-14
        } else {
            true
        };
    InvariantBoundarySigns {
        dp_at_p0,
        ds_at_s0,
        ds_at_theta1,
        dp_prefailure: dp_pre,
        ds_prefailure: ds_pre,
        continuous_inward,
    }
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

/// Diagnostics for autonomous interface-normal velocity estimation (D-025).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InterfaceVelocityDiagnostics {
    pub min_interface_grad: f64,
    pub max_abs_vn: f64,
    pub interface_band_area: f64,
    pub mean_vn: f64,
    pub band_cell_count: usize,
}

/// True when the cell is inside the valid diffuse-interface band for surface velocity.
#[inline]
pub fn in_interface_velocity_band(delta: f64, grad_phi: f64, delta_floor: f64, grad_min: f64) -> bool {
    delta > delta_floor && grad_phi >= grad_min
}

/// Central-difference |∇φ| matching `compute_interface_geometry` sampling.
pub fn grad_phi_magnitude(grid: &Grid, phi: &[f64], i: usize, j: usize) -> f64 {
    let w = grid.width;
    let inv_2dx = 0.5 / DX;
    let left = sample_phi(grid, phi, i.wrapping_sub(1), j, i, j);
    let right = sample_phi(grid, phi, i + 1, j, i, j);
    let down = sample_phi(grid, phi, i, j.wrapping_sub(1), i, j);
    let up = sample_phi(grid, phi, i, j + 1, i, j);
    let dphix = (right - left) * inv_2dx;
    let dphiy = (up - down) * inv_2dx;
    (dphix * dphix + dphiy * dphiy).sqrt()
}

/// Derive interface-normal speed from old and tentative φ:
/// `v_n = −(φ_next − φ_old)/dt / sqrt(|∇φ_old|² + η_v²)` inside the valid band.
///
/// Outside the band, `vn_out = 0` (no weak-gradient division). Depends only on
/// old structure, tentative update, dt, and local derivatives.
pub fn estimate_interface_normal_velocity(
    grid: &Grid,
    phi_old: &[f64],
    phi_next: &[f64],
    geometry_old: &[InterfaceGeometryCell],
    dt: f64,
    eta_v: f64,
    delta_floor: f64,
    grad_min: f64,
    vn_out: &mut [f64],
) -> InterfaceVelocityDiagnostics {
    let w = grid.width;
    let h = grid.height;
    let inv_dt = if dt.abs() > 0.0 { 1.0 / dt } else { 0.0 };
    let eta2 = eta_v * eta_v;
    vn_out.fill(0.0);

    let mut min_grad = f64::INFINITY;
    let mut max_abs_vn = 0.0_f64;
    let mut band_area = 0.0_f64;
    let mut vn_sum = 0.0_f64;
    let mut band_count = 0usize;
    let cell = DX * DX;

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let grad = grad_phi_magnitude(grid, phi_old, i, j);
            let delta = geometry_old[idx].delta;
            if !in_interface_velocity_band(delta, grad, delta_floor, grad_min) {
                continue;
            }
            let dphi_dt = (phi_next[idx] - phi_old[idx]) * inv_dt;
            let denom = (grad * grad + eta2).sqrt();
            let vn = -dphi_dt / denom;
            vn_out[idx] = vn;
            min_grad = min_grad.min(grad);
            max_abs_vn = max_abs_vn.max(vn.abs());
            band_area += cell;
            vn_sum += vn;
            band_count += 1;
        }
    }
    if !min_grad.is_finite() {
        min_grad = 0.0;
    }
    InterfaceVelocityDiagnostics {
        min_interface_grad: min_grad,
        max_abs_vn,
        interface_band_area: band_area,
        mean_vn: if band_count > 0 {
            vn_sum / band_count as f64
        } else {
            0.0
        },
        band_cell_count: band_count,
    }
}

/// Analytic circular tanh profile centered at `(cx, cy)`.
pub fn circular_phi_profile_at(
    grid: &Grid,
    cx: f64,
    cy: f64,
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
            let x = i as f64;
            let y = j as f64;
            let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            phi[idx] = (0.5 * (1.0 - ((r - radius) / eps).tanh())).clamp(0.0, 1.0);
        }
    }
}

/// Conservative surface advection ∂S/∂t += −∇·(S u_Γ) with u_Γ = v_n n.
///
/// `vn` is a per-cell normal speed (prescribed diagnostic or autonomous estimate).
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
/// irreversible: `P -= δ J_ads`, `S += δ J_ads` (v7),
/// reversible: `P -= δ J_exchange`, `S += δ J_exchange` (v8),
/// `S -= δ J_loss`, `W += δ J_loss`.
///
/// When `vn` is `Some`, also applies conservative autonomous advection
/// `∂S/∂t += −∇·(S u_Γ)` using old-state geometry and the provided normal speed.
/// D-034 dedicated dual-surface maturation evolution (v11).
///
/// Fields: soluble precursor `P`, immature surface `U = δΓ_U`, mature surface `S = δΓ_S`.
/// `U` and `S` are transported independently (surface diffusion `D_U`/`D_S`, shared advection).
/// Per-cell Strang order: half S→W turnover → passive P↔U exchange (S fixed) →
/// maturation U+A→S+W → half S→W turnover. Skips all v10 charge/insert/relax and P↔S exchange.
///
/// `intermediate`/`intermediate_next` carry `U` (caller pre-copies current U into
/// `intermediate_next`). `geometry` is precomputed by the caller.
#[allow(clippy::too_many_arguments)]
fn evolve_surface_maturation_v11(
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
    vn: Option<&[f64]>,
    geometry: &[InterfaceGeometryCell],
    gamma_s: &mut [f64],
    diffusion_rate_s: &mut [f64],
    advection_rate: &mut Vec<f64>,
    s_next: &mut [f64],
    activated_next: &mut [f64],
    precursor_next: &mut [f64],
    waste_next: &mut [f64],
    immature: Option<&[f64]>,
    immature_next: Option<&mut [f64]>,
) -> Result<SurfaceAccountingTotals, ExchangeReject> {
    let delta_floor = params.delta_floor;
    let n = s.len();
    // v11 requires the U buffers; a missing buffer is a wiring bug, not a physical state.
    let (u_old, u_next) = match (immature, immature_next) {
        (Some(a), Some(b)) => (a, b),
        _ => return Err(ExchangeReject::NonfiniteFlux),
    };

    reconstruct_gamma_field(grid, s, geometry, delta_floor, gamma_s);
    let mut gamma_u = vec![0.0; n];
    reconstruct_gamma_field(grid, u_old, geometry, delta_floor, &mut gamma_u);

    let mut totals = SurfaceAccountingTotals::default();
    let mut diffusion_rate_u = vec![0.0; n];
    if enable_diffusion {
        totals.absolute_face_flux += surface_diffusion_rate(
            grid,
            geometry,
            gamma_s,
            params.d_gamma,
            params.delta_face_eps,
            diffusion_rate_s,
        );
        totals.absolute_face_flux += surface_diffusion_rate(
            grid,
            geometry,
            &gamma_u,
            params.d_u,
            params.delta_face_eps,
            &mut diffusion_rate_u,
        );
    } else {
        diffusion_rate_s.fill(0.0);
    }

    let mut advection_rate_u = vec![0.0; n];
    if let Some(vn_field) = vn {
        if advection_rate.len() != n {
            advection_rate.resize(n, 0.0);
        }
        totals.absolute_face_flux += surface_advection_rate(grid, geometry, s, vn_field, advection_rate);
        totals.absolute_face_flux +=
            surface_advection_rate(grid, geometry, u_old, vn_field, &mut advection_rate_u);
    } else if !advection_rate.is_empty() {
        advection_rate.fill(0.0);
    }

    for idx in 0..n {
        if !grid.in_dish(idx) {
            s_next[idx] = 0.0;
            u_next[idx] = 0.0;
            continue;
        }
        let d = geometry[idx].delta;
        let mut ds_s = 0.0;
        let mut ds_u = 0.0;
        if enable_diffusion {
            ds_s += diffusion_rate_s[idx] * dt;
            ds_u += diffusion_rate_u[idx] * dt;
            totals.surface_diffusion_delta += diffusion_rate_s[idx] * dt;
            totals.immature_diffusion_delta += diffusion_rate_u[idx] * dt;
        }
        if vn.is_some() && !advection_rate.is_empty() {
            ds_s += advection_rate[idx] * dt;
            ds_u += advection_rate_u[idx] * dt;
            totals.advection_delta += advection_rate[idx] * dt;
            totals.immature_advection_delta += advection_rate_u[idx] * dt;
        }

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

        let mut s_work = s[idx] + ds_s;
        let mut u_work = u_old[idx] + ds_u;
        let mut p_work = precursor_next[idx];
        let mut a_work = activated_next[idx];

        // Off-interface: transport already applied; no surface reactions.
        if d <= delta_floor {
            precursor_next[idx] = p_work;
            activated_next[idx] = a_work;
            s_next[idx] = s_work;
            u_next[idx] = u_work.max(0.0);
            continue;
        }

        let q_c = membrane_catalyst_saturation(catalyst[idx], params);
        let c_surface = d * params.gamma_max.max(0.0);

        // 1) half biological S→W turnover (U has no biological decay).
        if enable_gamma_decay {
            let (s1, dw1) = apply_turnover_exact(s_work, params.k_gamma_decay, 0.5 * dt);
            waste_next[idx] += dw1;
            totals.gamma_decay_delta += dw1;
            totals.surface_to_waste += dw1;
            s_work = s1;
        }

        // 2) passive P↔U exchange with S held fixed (invariant-domain backward Euler).
        if enable_adsorption && params.k_exchange > 0.0 && params.gamma_max > 0.0 {
            let gamma_u_pre = u_work / d;
            let gamma_s_fixed = s_work / d;
            let (j_net0, j_fwd0, j_rev0, a_fwd, a_rev) =
                exchange_rate_j_dual(p_work, catalyst[idx], gamma_u_pre, gamma_s_fixed, params);
            let m = exchange_mobility(catalyst[idx], params);
            let diss = d * exchange_dissipation_density(a_fwd, a_rev, m) * dt;
            let t_inv = p_work.max(0.0) + u_work.max(0.0);
            let solved = solve_exchange_backward_euler_dual(
                u_work,
                t_inv,
                s_work,
                c_surface,
                d,
                q_c,
                params.k_exchange,
                params.k_exchange_eq,
                params.p_reference,
                params.gamma_max,
                dt,
            )?;
            let u_ex = solved.s_next;
            let p_ex = solved.p_next;
            if (p_ex + u_ex - t_inv).abs() > 1e-12 {
                return Err(ExchangeReject::NonfiniteFlux);
            }
            validate_dual_capacity(p_ex, u_ex, s_work, d, params.gamma_max, delta_floor)?;
            let xfer = u_ex - u_work;
            p_work = p_ex;
            u_work = u_ex;
            totals.adsorption_delta += xfer;
            totals.precursor_to_surface += xfer;
            totals.exchange_forward += (d * j_fwd0 * dt).max(0.0);
            totals.exchange_reverse += (d * j_rev0 * dt).max(0.0);
            totals.exchange_net += xfer;
            totals.exchange_dissipation += diss.max(0.0);
            let _ = j_net0;
        }

        // 3) maturation U+A→S+W (bounded by U and A; converts in place).
        let (u_m, a_m, s_m, w_m, r) =
            apply_maturation_bounded(u_work, a_work, s_work, d, catalyst[idx], dt, params);
        u_work = u_m;
        a_work = a_m;
        s_work = s_m;
        waste_next[idx] += w_m;
        totals.maturation_delta += r;

        // 4) half biological S→W turnover.
        if enable_gamma_decay {
            let (s2, dw2) = apply_turnover_exact(s_work, params.k_gamma_decay, 0.5 * dt);
            waste_next[idx] += dw2;
            totals.gamma_decay_delta += dw2;
            totals.surface_to_waste += dw2;
            s_work = s2;
        }

        precursor_next[idx] = p_work;
        activated_next[idx] = a_work;
        s_next[idx] = s_work;
        u_next[idx] = u_work.max(0.0);
        validate_dual_capacity(
            precursor_next[idx],
            u_next[idx],
            s_next[idx],
            d,
            params.gamma_max,
            delta_floor,
        )?;
    }
    Ok(totals)
}

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
    intermediate: Option<&[f64]>,
    intermediate_next: Option<&mut [f64]>,
) -> Result<SurfaceAccountingTotals, ExchangeReject> {
    let mut advection_rate = Vec::new();
    evolve_surface_density_with_vn(
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        params,
        dt,
        enable_synthesis,
        enable_adsorption,
        enable_precursor_decay,
        enable_gamma_decay,
        enable_diffusion,
        None,
        geometry,
        gamma,
        diffusion_rate,
        &mut advection_rate,
        s_next,
        activated_next,
        precursor_next,
        waste_next,
        intermediate,
        intermediate_next,
    )
}

/// Like [`evolve_surface_density`], with optional autonomous/prescribed `vn`.
#[allow(clippy::too_many_arguments)]
pub fn evolve_surface_density_with_vn(
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
    vn: Option<&[f64]>,
    geometry: &mut [InterfaceGeometryCell],
    gamma: &mut [f64],
    diffusion_rate: &mut [f64],
    advection_rate: &mut Vec<f64>,
    s_next: &mut [f64],
    activated_next: &mut [f64],
    precursor_next: &mut [f64],
    waste_next: &mut [f64],
    intermediate: Option<&[f64]>,
    mut intermediate_next: Option<&mut [f64]>,
) -> Result<SurfaceAccountingTotals, ExchangeReject> {
    // ponytail: callers pre-copy current buffer into `intermediate_next`.
    // For v10, `intermediate_next` holds X (activation intermediate). For v11 the dual-surface
    // path below owns U = δΓ_U in `intermediate`/`intermediate_next`.
    let eta_n = params.eta_n;
    let delta_floor = params.delta_floor;
    let reversible = params.equation_version.is_reversible_surface_exchange();
    let v10 = params.equation_version.is_activated_intermediate();
    let v11 = params.equation_version.is_surface_maturation();
    compute_interface_geometry(grid, phi, eta_n, geometry);
    if v11 {
        return evolve_surface_maturation_v11(
            grid,
            phi,
            catalyst,
            activated,
            precursor,
            s,
            params,
            dt,
            enable_synthesis,
            enable_adsorption,
            enable_precursor_decay,
            enable_gamma_decay,
            enable_diffusion,
            vn,
            geometry,
            gamma,
            diffusion_rate,
            advection_rate,
            s_next,
            activated_next,
            precursor_next,
            waste_next,
            intermediate,
            intermediate_next.take(),
        );
    }
    let _ = intermediate;
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

    if let Some(vn_field) = vn {
        if advection_rate.len() != s.len() {
            advection_rate.resize(s.len(), 0.0);
        }
        let abs_flux = surface_advection_rate(grid, geometry, s, vn_field, advection_rate);
        totals.absolute_face_flux += abs_flux;
    } else if !advection_rate.is_empty() {
        advection_rate.fill(0.0);
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

        if vn.is_some() && !advection_rate.is_empty() {
            let adv = advection_rate[idx] * dt;
            ds += adv;
            totals.advection_delta += adv;
        }

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
        // Local reaction composition (exchange + turnover) after transport / A↔P coupling.
        // Substep order (v9 + invariant v2):
        //   1) surface diffusion + optional advection
        //   2) precursor synthesis / precursor decay
        //   3) half biological S→W turnover (exact)
        //   4) full reversible P↔S exchange (backward Euler on invariant domain)
        //   5) active P+A→S+W assembly (v9) OR charge/insert/relax (v10)
        //   6) half biological S→W turnover (exact)
        // v8 omits step 5 when k_active=0 / non-v9.
        // v10 replaces step 5 with activated-intermediate pathway.
        let use_invariant = reversible
            && params.surface_exchange_integrator
                == SurfaceExchangeIntegrator::InvariantDomainV2;

        if use_invariant {
            let mut s_work = s[idx] + ds;
            let mut p_work = precursor_next[idx];
            // Off-interface: transport already applied; still allow bulk charge/relax.
            if d <= delta_floor {
                if v10 {
                    if let Some(x_buf) = intermediate_next.as_deref_mut() {
                        let x0 = x_buf[idx];
                        let (p_c, a_c, x_c, w_c, r_c) = apply_charge_bounded(
                            phi[idx],
                            catalyst[idx],
                            p_work,
                            activated_next[idx],
                            x0,
                            dt,
                            params,
                        );
                        let (x_r, p_r, r_r) = apply_relax_bounded(x_c, p_c, dt, params);
                        p_work = p_r;
                        activated_next[idx] = a_c;
                        x_buf[idx] = x_r;
                        waste_next[idx] += w_c;
                        totals.charge_delta += r_c;
                        totals.relax_delta += r_r;
                        totals.activation_production += r_c;
                        totals.activation_dissipation += r_r;
                        totals.activation_storage_delta += r_c - r_r;
                    }
                }
                precursor_next[idx] = p_work;
                s_next[idx] = s_work;
                continue;
            }
            let q_c = membrane_catalyst_saturation(catalyst[idx], params);
            let c_surface = d * params.gamma_max.max(0.0);

            if enable_gamma_decay {
                let (s1, dw1) = apply_turnover_exact(s_work, params.k_gamma_decay, 0.5 * dt);
                waste_next[idx] += dw1;
                totals.gamma_decay_delta += dw1;
                totals.surface_to_waste += dw1;
                s_work = s1;
            }

            if enable_adsorption && params.k_exchange > 0.0 && params.gamma_max > 0.0 {
                // Fast path: if explicit Euler proposal already lies in the invariant
                // domain, accept it (matches V1 for mild steps; avoids BE).
                let g_pre = s_work / d;
                let (j_net0, j_fwd0, j_rev0, a_fwd, a_rev) =
                    exchange_rate_j(p_work, catalyst[idx], g_pre, params);
                let xfer_e = d * j_net0 * dt;
                let p_e = p_work - xfer_e;
                let s_e = s_work + xfer_e;
                let mild_ok = validate_exchange_cell(
                    p_e,
                    s_e,
                    d,
                    params.gamma_max,
                    delta_floor,
                    1.0,
                    1.0,
                    0.0,
                )
                .is_ok();
                let m = exchange_mobility(catalyst[idx], params);
                let diss = d * exchange_dissipation_density(a_fwd, a_rev, m) * dt;
                if mild_ok {
                    p_work = p_e.max(0.0);
                    s_work = s_e.max(0.0);
                    totals.adsorption_delta += xfer_e;
                    totals.precursor_to_surface += xfer_e;
                    totals.exchange_forward += (d * j_fwd0 * dt).max(0.0);
                    totals.exchange_reverse += (d * j_rev0 * dt).max(0.0);
                    totals.exchange_net += xfer_e;
                    totals.exchange_dissipation += diss.max(0.0);
                } else {
                    let t_inv = p_work.max(0.0) + s_work.max(0.0);
                    let solved = solve_exchange_backward_euler(
                        s_work,
                        t_inv,
                        c_surface,
                        d,
                        q_c,
                        params.k_exchange,
                        params.k_exchange_eq,
                        params.p_reference,
                        params.gamma_max,
                        dt,
                    )?;
                    let s_ex = solved.s_next;
                    let p_ex = solved.p_next;
                    let xfer = s_ex - s_work;
                    if (p_ex + s_ex - t_inv).abs() > 1e-12 {
                        return Err(ExchangeReject::NonfiniteFlux);
                    }
                    validate_exchange_cell(
                        p_ex,
                        s_ex,
                        d,
                        params.gamma_max,
                        delta_floor,
                        1.0,
                        1.0,
                        0.0,
                    )?;
                    p_work = p_ex;
                    s_work = s_ex;
                    totals.adsorption_delta += xfer;
                    totals.precursor_to_surface += xfer;
                    totals.exchange_forward += (d * j_fwd0 * dt).max(0.0);
                    totals.exchange_reverse += (d * j_rev0 * dt).max(0.0);
                    totals.exchange_net += xfer;
                    totals.exchange_dissipation += diss.max(0.0);
                    let _ = j_net0;
                }
            }

            // D-032: powered P+A→S+W after passive exchange, before final Strang turnover half.
            if params.equation_version.is_activated_surface_assembly() && params.k_active > 0.0 {
                let a_work = activated_next[idx];
                let (p_a, a_a, s_a, w_a, r) = apply_active_assembly_bounded(
                    p_work,
                    a_work,
                    s_work,
                    d,
                    catalyst[idx],
                    dt,
                    params,
                );
                p_work = p_a;
                activated_next[idx] = a_a;
                s_work = s_a;
                waste_next[idx] += w_a;
                totals.active_assembly += r;
                totals.active_assembly_activation += r;
            }

            // D-033: charge → insert → relax (X stores activation potential).
            if v10 {
                if let Some(x_buf) = intermediate_next.as_deref_mut() {
                    let x0 = x_buf[idx];
                    let (p_i, a_i, x_i, s_i, w_i, r_c, r_i, r_r) =
                        apply_activated_intermediate_bounded(
                            phi[idx],
                            catalyst[idx],
                            p_work,
                            activated_next[idx],
                            x0,
                            s_work,
                            d,
                            dt,
                            params,
                        );
                    p_work = p_i;
                    activated_next[idx] = a_i;
                    x_buf[idx] = x_i;
                    s_work = s_i;
                    waste_next[idx] += w_i;
                    totals.charge_delta += r_c;
                    totals.insert_delta += r_i;
                    totals.relax_delta += r_r;
                    totals.activation_production += r_c;
                    totals.activation_work += r_i;
                    totals.activation_dissipation += r_r;
                    totals.activation_storage_delta += r_c - r_i - r_r;
                }
            }

            if enable_gamma_decay {
                let (s2, dw2) = apply_turnover_exact(s_work, params.k_gamma_decay, 0.5 * dt);
                waste_next[idx] += dw2;
                totals.gamma_decay_delta += dw2;
                totals.surface_to_waste += dw2;
                s_work = s2;
            }

            precursor_next[idx] = p_work;
            s_next[idx] = s_work;
            if enable_adsorption {
                validate_exchange_cell(
                    precursor_next[idx],
                    s_next[idx],
                    d,
                    params.gamma_max,
                    delta_floor,
                    1.0,
                    1.0,
                    0.0,
                )?;
            }
        } else {
            if enable_adsorption {
                if reversible {
                    let (j_net, j_fwd, j_rev, a_fwd, a_rev) =
                        exchange_rate_j(precursor[idx], catalyst[idx], g, params);
                    let xfer = d * j_net * dt;
                    let fwd_ext = d * j_fwd * dt;
                    let rev_ext = d * j_rev * dt;
                    let m = exchange_mobility(catalyst[idx], params);
                    let diss = d * exchange_dissipation_density(a_fwd, a_rev, m) * dt;
                    let p_trial = precursor_next[idx] - xfer;
                    let s_trial = s[idx] + ds + xfer;
                    validate_exchange_cell(
                        p_trial,
                        s_trial,
                        d,
                        params.gamma_max,
                        delta_floor,
                        a_fwd,
                        a_rev,
                        j_net,
                    )?;
                    ds += xfer;
                    precursor_next[idx] = p_trial;
                    totals.adsorption_delta += xfer;
                    totals.precursor_to_surface += xfer;
                    totals.exchange_forward += fwd_ext;
                    totals.exchange_reverse += rev_ext;
                    totals.exchange_net += xfer;
                    totals.exchange_dissipation += diss;
                } else {
                    let j_ads = adsorption_rate_j(precursor[idx], catalyst[idx], g, params);
                    let ads = d * j_ads * dt;
                    ds += ads;
                    precursor_next[idx] -= ads;
                    totals.adsorption_delta += ads;
                    totals.precursor_to_surface += ads;
                }
            }
            if params.equation_version.is_activated_surface_assembly() && params.k_active > 0.0 {
                let s_pre = s[idx] + ds;
                let (p_a, a_a, s_a, w_a, r) = apply_active_assembly_bounded(
                    precursor_next[idx],
                    activated_next[idx],
                    s_pre,
                    d,
                    catalyst[idx],
                    dt,
                    params,
                );
                precursor_next[idx] = p_a;
                activated_next[idx] = a_a;
                ds += s_a - s_pre;
                waste_next[idx] += w_a;
                totals.active_assembly += r;
                totals.active_assembly_activation += r;
            }
            if v10 {
                if let Some(x_buf) = intermediate_next.as_deref_mut() {
                    let s_pre = s[idx] + ds;
                    let x0 = x_buf[idx];
                    let (p_i, a_i, x_i, s_i, w_i, r_c, r_i, r_r) =
                        apply_activated_intermediate_bounded(
                            phi[idx],
                            catalyst[idx],
                            precursor_next[idx],
                            activated_next[idx],
                            x0,
                            s_pre,
                            d,
                            dt,
                            params,
                        );
                    precursor_next[idx] = p_i;
                    activated_next[idx] = a_i;
                    x_buf[idx] = x_i;
                    ds += s_i - s_pre;
                    waste_next[idx] += w_i;
                    totals.charge_delta += r_c;
                    totals.insert_delta += r_i;
                    totals.relax_delta += r_r;
                    totals.activation_production += r_c;
                    totals.activation_work += r_i;
                    totals.activation_dissipation += r_r;
                    totals.activation_storage_delta += r_c - r_i - r_r;
                }
            }
            if enable_gamma_decay {
                let g_now = if d > delta_floor {
                    (s[idx] + ds) / d
                } else {
                    0.0
                };
                let j_loss = gamma_decay_rate_j(g_now, params);
                let loss = d * j_loss * dt;
                ds -= loss;
                waste_next[idx] += loss;
                totals.gamma_decay_delta += loss;
                totals.surface_to_waste += loss;
            }
            s_next[idx] = s[idx] + ds;
            if reversible && enable_adsorption {
                validate_exchange_cell(
                    precursor_next[idx],
                    s_next[idx],
                    d,
                    params.gamma_max,
                    delta_floor,
                    1.0,
                    1.0,
                    0.0,
                )?;
            }
        }
    }
    Ok(totals)
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
