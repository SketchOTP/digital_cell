//! D-027 coupled surface-renewal calibration: adsorption basis, candidates, gates.

use crate::config::SimParams;
use crate::d025_analysis::D025_FROZEN_K_ADS;
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    compute_interface_geometry, reconstruct_gamma_field, theta_gamma, InterfaceGeometryCell,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

/// Numerical floor for adsorption basis integrals.
pub const D027_ADS_BASIS_EPS: f64 = 1e-18;
/// Portability: valid k_ads_required estimates must span ≤ this factor.
pub const D027_PORTABILITY_SPAN_MAX: f64 = 3.0;
/// Analytical candidate scale factors (exactly three).
pub const D027_CANDIDATE_SCALES: [f64; 3] = [0.5, 1.0, 2.0];
pub const D027_MAX_ADSORPTION_CANDIDATES: usize = 3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WindowLocalSurfaceRates {
    pub precursor_synthesis: f64,
    pub adsorption: f64,
    pub precursor_decay: f64,
    pub gamma_turnover: f64,
    pub surface_transport: f64,
    pub accepted_steps_in_window: u64,
    pub window_dt: f64,
}

impl WindowLocalSurfaceRates {
    pub fn from_sim(sim: &Simulation) -> Self {
        let rates = sim.surface_accounting.window_local_rates(sim.sim_time);
        Self {
            precursor_synthesis: rates.precursor_synthesis_delta,
            adsorption: rates.adsorption_delta,
            precursor_decay: rates.precursor_decay_delta,
            gamma_turnover: rates.gamma_decay_delta,
            surface_transport: rates.surface_diffusion_delta + rates.advection_delta,
            accepted_steps_in_window: sim
                .substep
                .saturating_sub(sim.surface_accounting.window_baseline_substep),
            window_dt: (sim.sim_time - sim.surface_accounting.window_baseline_time).max(0.0),
        }
    }
}

/// Capture window-local surface + A + material exchange rates after restore/baseline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowLocalLedgerSnapshot {
    pub surface: WindowLocalSurfaceRates,
    pub a_production_activation: f64,
    pub a_consumption_reproduction: f64,
    pub a_consumption_activated_decay: f64,
    pub nutrient_exchange: f64,
    pub fuel_exchange: f64,
    pub waste_exchange: f64,
    pub material_relative_hint: f64,
}

pub fn window_local_ledger_snapshot(sim: &Simulation) -> WindowLocalLedgerSnapshot {
    let dt = (sim.sim_time - sim.surface_accounting.window_baseline_time).max(f64::EPSILON);
    let metab = &sim.metabolism_accounting.cumulative;
    // ponytail: A window-local uses metabolism cumulative / full sim_time until dedicated
    // A window baseline is added; surface path is the Gate 0 authority.
    let n = sim.substep.max(1) as f64;
    let surface = WindowLocalSurfaceRates {
        precursor_synthesis: sim
            .surface_accounting
            .window_local_rates(sim.sim_time)
            .precursor_synthesis_delta,
        adsorption: sim
            .surface_accounting
            .window_local_rates(sim.sim_time)
            .adsorption_delta,
        precursor_decay: sim
            .surface_accounting
            .window_local_rates(sim.sim_time)
            .precursor_decay_delta,
        gamma_turnover: sim
            .surface_accounting
            .window_local_rates(sim.sim_time)
            .gamma_decay_delta,
        surface_transport: {
            let r = sim.surface_accounting.window_local_rates(sim.sim_time);
            r.surface_diffusion_delta + r.advection_delta
        },
        accepted_steps_in_window: sim.substep.saturating_sub(sim.surface_accounting.window_baseline_substep),
        window_dt: dt,
    };
    let transport = &sim.transport_accounting.last_step;
    WindowLocalLedgerSnapshot {
        surface,
        a_production_activation: metab.activation / n,
        a_consumption_reproduction: metab.reproduction / n,
        a_consumption_activated_decay: metab.activated_decay / n,
        nutrient_exchange: transport.nutrient.interior_net_flux_rate,
        fuel_exchange: transport.fuel.interior_net_flux_rate,
        waste_exchange: transport.waste.interior_net_flux_rate,
        material_relative_hint: {
            let processed = sim
                .accounting
                .cumulative
                .cumulative_processed_mass
                .max(1.0);
            sim.accounting.cumulative.cumulative_unexplained_residual / processed
        },
    }
}

pub fn surface_rates_parity(a: &WindowLocalSurfaceRates, b: &WindowLocalSurfaceRates) -> (f64, bool) {
    let pairs = [
        a.precursor_synthesis - b.precursor_synthesis,
        a.adsorption - b.adsorption,
        a.precursor_decay - b.precursor_decay,
        a.gamma_turnover - b.gamma_turnover,
        a.surface_transport - b.surface_transport,
    ];
    let max_abs = pairs.iter().map(|d| d.abs()).fold(0.0_f64, f64::max);
    let tol = 1e-9 + 1e-6 * pairs
        .iter()
        .zip([
            a.precursor_synthesis.abs().max(b.precursor_synthesis.abs()),
            a.adsorption.abs().max(b.adsorption.abs()),
            a.precursor_decay.abs().max(b.precursor_decay.abs()),
            a.gamma_turnover.abs().max(b.gamma_turnover.abs()),
            a.surface_transport.abs().max(b.surface_transport.abs()),
        ])
        .map(|(d, s)| d.abs() / s.max(1.0))
        .fold(0.0_f64, f64::max)
        .max(1.0);
    // Absolute primary: restored vs uninterrupted should be near machine/window noise.
    let ok = max_abs <= 1e-9 || max_abs <= 1e-6 * [
        a.adsorption.abs(),
        b.adsorption.abs(),
        a.gamma_turnover.abs(),
        b.gamma_turnover.abs(),
        1.0,
    ]
    .into_iter()
    .fold(0.0_f64, f64::max);
    let _ = tol;
    (max_abs, ok)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdsorptionBasisReport {
    pub label: String,
    pub b_ads: f64,
    pub l_gamma: f64,
    pub k_ads_required: f64,
    pub mean_theta_gamma: f64,
    pub mean_p_near_interface: f64,
    pub mean_saturation_factor: f64,
    pub mean_q_c: f64,
    pub interface_measure: f64,
    pub finite: bool,
    pub underflow_dominated: bool,
}

/// B_ads = ∫ δ P q(C) max(0, 1−Γ/Γ_max) dV ; L_gamma = ∫ δ k_gamma_decay Γ dV
pub fn compute_adsorption_basis(sim: &Simulation) -> AdsorptionBasisReport {
    compute_adsorption_basis_labeled(sim, "unnamed")
}

pub fn compute_adsorption_basis_labeled(sim: &Simulation, label: &str) -> AdsorptionBasisReport {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut gamma = vec![0.0; n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    reconstruct_gamma_field(
        &sim.grid,
        &sim.fields.membrane,
        &geometry,
        sim.params.delta_floor,
        &mut gamma,
    );
    let gamma_max = sim.params.gamma_max.max(f64::EPSILON);
    let gamma_ref = sim.params.gamma_reference.max(f64::EPSILON);
    let k_decay = sim.params.k_gamma_decay;
    let mut b_ads = 0.0;
    let mut l_gamma = 0.0;
    let mut iface = 0.0;
    let mut p_sum = 0.0;
    let mut sat_sum = 0.0;
    let mut q_sum = 0.0;
    let mut theta_sum = 0.0;
    let mut cells = 0.0;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let delta = geometry[idx].delta;
        if delta <= sim.params.delta_floor {
            continue;
        }
        let g = gamma[idx].max(0.0);
        let p = sim.fields.precursor[idx].max(0.0);
        let c = sim.fields.catalyst[idx].max(0.0);
        let q_c = membrane_q_c(c, &sim.params);
        let sat = (1.0 - g / gamma_max).max(0.0);
        b_ads += delta * p * q_c * sat;
        l_gamma += delta * k_decay * g;
        iface += delta;
        p_sum += p;
        sat_sum += sat;
        q_sum += q_c;
        theta_sum += theta_gamma(g, gamma_ref);
        cells += 1.0;
    }
    let k_req = l_gamma / b_ads.max(D027_ADS_BASIS_EPS);
    let finite = k_req.is_finite() && b_ads.is_finite() && l_gamma.is_finite();
    let underflow = b_ads < D027_ADS_BASIS_EPS * 10.0 && l_gamma > 0.0;
    AdsorptionBasisReport {
        label: label.to_string(),
        b_ads,
        l_gamma,
        k_ads_required: k_req,
        mean_theta_gamma: if cells > 0.0 { theta_sum / cells } else { 0.0 },
        mean_p_near_interface: if cells > 0.0 { p_sum / cells } else { 0.0 },
        mean_saturation_factor: if cells > 0.0 { sat_sum / cells } else { 0.0 },
        mean_q_c: if cells > 0.0 { q_sum / cells } else { 0.0 },
        interface_measure: iface,
        finite,
        underflow_dominated: underflow,
    }
}

fn membrane_q_c(c: f64, params: &SimParams) -> f64 {
    membrane_catalyst_saturation(c, params)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortabilityResult {
    pub portable: bool,
    pub span: f64,
    pub median_k_ads_required: f64,
    pub valid_count: usize,
    pub conclusion: String,
}

pub fn classify_adsorption_portability(reports: &[AdsorptionBasisReport]) -> PortabilityResult {
    let valid: Vec<f64> = reports
        .iter()
        .filter(|r| r.finite && !r.underflow_dominated && r.b_ads > D027_ADS_BASIS_EPS)
        .filter(|r| r.mean_saturation_factor > 1e-6) // not permanently saturated
        .filter(|r| r.mean_p_near_interface > 1e-12) // P available
        .map(|r| r.k_ads_required)
        .filter(|k| k.is_finite() && *k > 0.0)
        .collect();
    if valid.is_empty() {
        return PortabilityResult {
            portable: false,
            span: f64::INFINITY,
            median_k_ads_required: f64::NAN,
            valid_count: 0,
            conclusion: "D027_ADSORPTION_LAW_NOT_PORTABLE".into(),
        };
    }
    let mut sorted = valid.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = sorted[sorted.len() / 2];
    let min_k = sorted[0];
    let max_k = *sorted.last().unwrap();
    let span = max_k / min_k.max(f64::EPSILON);
    let portable = span <= D027_PORTABILITY_SPAN_MAX;
    PortabilityResult {
        portable,
        span,
        median_k_ads_required: median,
        valid_count: valid.len(),
        conclusion: if portable {
            "PORTABLE_CANDIDATE_SET".into()
        } else {
            "D027_ADSORPTION_LAW_NOT_PORTABLE".into()
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdsorptionCandidate {
    pub candidate_id: String,
    pub parent_candidate: String,
    pub k_ads: f64,
    pub scale: f64,
    pub analytical_center: f64,
    pub source_windows: Vec<String>,
}

pub fn generate_analytical_candidates(
    portability: &PortabilityResult,
    parent: &str,
    source_windows: &[String],
) -> Result<Vec<AdsorptionCandidate>, String> {
    if !portability.portable || !portability.median_k_ads_required.is_finite() {
        return Err(portability.conclusion.clone());
    }
    let center = portability.median_k_ads_required;
    let mut out = Vec::with_capacity(D027_MAX_ADSORPTION_CANDIDATES);
    for (i, &scale) in D027_CANDIDATE_SCALES.iter().enumerate() {
        out.push(AdsorptionCandidate {
            candidate_id: format!("d027-ads-{}x", scale),
            parent_candidate: parent.to_string(),
            k_ads: scale * center,
            scale,
            analytical_center: center,
            source_windows: source_windows.to_vec(),
        });
        let _ = i;
    }
    assert_eq!(out.len(), D027_MAX_ADSORPTION_CANDIDATES);
    Ok(out)
}

pub fn frozen_k_ads_d024() -> f64 {
    D025_FROZEN_K_ADS
}

pub fn surface_balance_q(adsorption_rate: f64, gamma_turnover_rate: f64) -> f64 {
    adsorption_rate / gamma_turnover_rate.max(f64::EPSILON)
}

#[cfg(test)]
mod unit_smoke {
    use super::*;

    #[test]
    fn candidate_count_is_three() {
        let p = PortabilityResult {
            portable: true,
            span: 1.5,
            median_k_ads_required: 0.03,
            valid_count: 3,
            conclusion: "PORTABLE_CANDIDATE_SET".into(),
        };
        let c = generate_analytical_candidates(&p, "d024", &["w1".into()]).unwrap();
        assert_eq!(c.len(), 3);
        assert!((c[1].k_ads - 0.03).abs() < 1e-15);
    }
}
