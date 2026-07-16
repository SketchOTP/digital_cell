//! D-016 intracellular waste transport timescale audit and fixed-source assay.

use crate::activated_metabolism::activated_metabolism_rates;
use crate::config::{
    SimParams, CONC_SAFETY_LIMIT, DX, MAX_DT, TRANSPORT_SCHEMA_VERSION_V1,
};
use crate::d015_waste::{build_waste_spatial_masks, linear_sink_clearance_rate};
use crate::grid::Grid;
use crate::membrane::membrane_rates;
use crate::membrane_transport::{
    face_diffusivity, face_geometry, permeability, transport_field, TransportSpecies,
};
use crate::reactions::interface_weight;
use crate::reservoir::{apply_reservoir, waste_sink_cell};
use serde::{Deserialize, Serialize};

pub const D016_W_TARGET_FRAC: f64 = 0.50;
pub const D016_W_CEILING_DIAG_FRAC: f64 = 0.90;
pub const D016_SINK_MATCH_TOL: f64 = 0.02;
pub const D016_FIXED_SOURCE_MAX_STEPS: u64 = 200_000;
pub const D016_AUTHORIZED_D_W_BOUND_SPECIES: &str = "max(D_N, D_F)";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasteTransportAudit {
    pub base_d_w: f64,
    pub inside_d_w: f64,
    pub interface_d_w_at_m1: f64,
    pub outside_d_w: f64,
    pub face_averaging_rule: String,
    pub membrane_permeability_rule: String,
    pub beta_w: f64,
    pub p_w_at_m: Vec<(f64, f64)>,
    pub grid_spacing: f64,
    pub transport_timestep_limit: f64,
    pub boundary_condition: String,
    pub reservoir_sink_geometry: String,
    pub d_w_uniform_across_dish: bool,
    pub d_w_phase_dependent: bool,
    pub membrane_dependent_only_through_p_w: bool,
    pub shared_with_another_soluble_field: bool,
    pub modified_elsewhere: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeciesDiffusivityRow {
    pub species: String,
    pub base_diffusivity: f64,
    pub permeability_at_m1: f64,
    pub effective_interface_diffusivity: f64,
    pub characteristic_r22_crossing_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceFieldSummary {
    pub total_source_rate: f64,
    pub interior_source_rate: f64,
    pub interface_source_rate: f64,
    pub maximum_local_source_rate: f64,
    pub source_weighted_radius: f64,
    pub fraction_inside_r_over_2: f64,
    pub fraction_inside_3r_over_4: f64,
    pub q_area: f64,
    pub interior_cells: usize,
    pub window_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimescaleAnalysis {
    pub tau_fill: f64,
    pub tau_center_to_interface: f64,
    pub tau_interface_crossing: f64,
    pub tau_interface_to_sink: f64,
    pub tau_sink_clearance: f64,
    pub da_internal: f64,
    pub da_membrane: f64,
    pub da_external: f64,
    pub da_clearance: f64,
    pub analytical_delta_w_center: f64,
    pub d_w_required_50pct: f64,
    pub d_w_required_90pct: f64,
    pub authorized_d_w_bound: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MembraneConductanceAnalysis {
    pub j_required: f64,
    pub g_w: f64,
    pub delta_w_required: f64,
    pub allowable_delta_w: f64,
    pub classification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResistanceDecomposition {
    pub r_internal: f64,
    pub r_membrane: f64,
    pub r_external: f64,
    pub r_sink: f64,
    pub internal_fraction: f64,
    pub membrane_fraction: f64,
    pub external_fraction: f64,
    pub sink_fraction: f64,
    pub dominant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixedSourceAssayResult {
    pub classification: String,
    pub accepted_substeps: u64,
    pub simulated_time: f64,
    pub center_w: f64,
    pub max_w: f64,
    pub mean_source_rate: f64,
    pub mean_sink_removal_rate: f64,
    pub sink_matches_source: bool,
    pub d_w: f64,
    pub beta_w: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveTransportFeasibility {
    Feasible,
    Insufficient,
}

pub fn waste_permeability_at_m(m: f64, params: &SimParams) -> f64 {
    let geometry = face_geometry(0.5, 0.5, m, m);
    permeability(TransportSpecies::Waste, geometry, params)
}

pub fn audit_waste_transport(params: &SimParams) -> WasteTransportAudit {
    let p_w_at_m = [0.0, 0.25, 0.50, 0.75, 1.00]
        .iter()
        .map(|&m| (m, waste_permeability_at_m(m, params)))
        .collect();
    let iface_d = face_diffusivity(TransportSpecies::Waste, 0.5, 0.5, 1.0, 1.0, params);
    WasteTransportAudit {
        base_d_w: params.d_w,
        inside_d_w: params.d_w,
        interface_d_w_at_m1: iface_d,
        outside_d_w: params.d_w,
        face_averaging_rule: "0.5*(D_i+D_j)*P(species, face geometry)".into(),
        membrane_permeability_rule: "P=exp(-beta*M_face*I_face); I=0.5*(I(phi_i)+I(phi_j))".into(),
        beta_w: params.beta_w,
        p_w_at_m,
        grid_spacing: DX,
        transport_timestep_limit: MAX_DT,
        boundary_condition: "no-flux dish; W clearance on waste_sink_cell only".into(),
        reservoir_sink_geometry: format!(
            "waste_sink_inner_radius={}; N/F reservoir mask unchanged",
            params.waste_sink_inner_radius
        ),
        d_w_uniform_across_dish: true,
        d_w_phase_dependent: false,
        membrane_dependent_only_through_p_w: true,
        shared_with_another_soluble_field: false,
        modified_elsewhere: false,
    }
}

pub fn species_diffusivity_comparison(params: &SimParams, radius: f64) -> Vec<SpeciesDiffusivityRow> {
    let rows = [
        ("C", params.d_c_inside, params.beta_c, TransportSpecies::Catalyst),
        ("A", params.d_a, params.beta_a, TransportSpecies::Activated),
        ("N", params.d_n, params.beta_n, TransportSpecies::Nutrient),
        ("F", params.d_f, params.beta_f, TransportSpecies::Fuel),
        ("W", params.d_w, params.beta_w, TransportSpecies::Waste),
    ];
    rows.iter()
        .map(|(name, base, _beta, species)| {
            let p = permeability(
                *species,
                face_geometry(0.5, 0.5, 1.0, 1.0),
                params,
            );
            let eff = face_diffusivity(*species, 0.5, 0.5, 1.0, 1.0, params);
            let tau = if *base > 0.0 {
                (radius * radius) / (4.0 * *base)
            } else {
                f64::INFINITY
            };
            SpeciesDiffusivityRow {
                species: (*name).into(),
                base_diffusivity: *base,
                permeability_at_m1: p,
                effective_interface_diffusivity: eff,
                characteristic_r22_crossing_time: tau,
            }
        })
        .collect()
}

pub fn w_ordering_vs_nutrient_fuel(params: &SimParams) -> &'static str {
    let nf = params.d_n.max(params.d_f);
    if params.d_w < nf * (1.0 - 1e-12) {
        "slower than nutrient and fuel"
    } else if (params.d_w - nf).abs() <= 1e-12 {
        "similar to nutrient and fuel"
    } else {
        "faster than nutrient and fuel"
    }
}

/// Local v2 waste production rate (mass/time per cell) from frozen geometry fields.
pub fn local_waste_source_rate(
    phi: f64,
    catalyst: f64,
    nutrient: f64,
    fuel: f64,
    activated: f64,
    membrane: f64,
    params: &SimParams,
) -> f64 {
    let rates = activated_metabolism_rates(catalyst, nutrient, fuel, activated, params);
    let i_face = interface_weight(phi);
    let r_structure = params.k_d008_structure * activated.max(0.0) * i_face;
    let r_structure_decay = params.k_structure_decay * phi.max(0.0);
    let d_w_structure = (1.0 - params.eta_phi) * r_structure + r_structure_decay;
    let m_rates = membrane_rates(phi, catalyst, activated, membrane, params);
    let d_w_membrane =
        (1.0 - params.eta_m) * m_rates.synthesis + m_rates.decay + m_rates.detachment;
    rates.d_waste + d_w_structure + d_w_membrane
}

pub fn summarize_source_field(
    grid: &Grid,
    phi: &[f64],
    catalyst: &[f64],
    nutrient: &[f64],
    fuel: &[f64],
    activated: &[f64],
    membrane: &[f64],
    params: &SimParams,
    prescribed_radius: f64,
    window_label: &str,
) -> (Vec<f64>, SourceFieldSummary) {
    let n = grid.width * grid.height;
    let mut q = vec![0.0; n];
    let masks = build_waste_spatial_masks(grid, phi, prescribed_radius);
    let mut total = 0.0;
    let mut interior = 0.0;
    let mut interface = 0.0;
    let mut max_local = 0.0;
    let mut weighted_r = 0.0;
    let mut inside_half = 0.0;
    let mut inside_three_quarter = 0.0;
    let mut interior_cells = 0usize;
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let rate = local_waste_source_rate(
            phi[idx],
            catalyst[idx],
            nutrient[idx],
            fuel[idx],
            activated[idx],
            membrane[idx],
            params,
        );
        q[idx] = rate;
        if rate <= 0.0 {
            continue;
        }
        total += rate;
        if rate > max_local {
            max_local = rate;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        let r = grid.distance_from_center(i, j);
        weighted_r += rate * r;
        if r <= prescribed_radius * 0.5 {
            inside_half += rate;
        }
        if r <= prescribed_radius * 0.75 {
            inside_three_quarter += rate;
        }
        if masks.interior[idx] {
            interior += rate;
            interior_cells += 1;
        }
        if masks.interface[idx] {
            interface += rate;
        }
    }
    let q_area = if interior_cells > 0 {
        interior / interior_cells as f64
    } else {
        0.0
    };
    let summary = SourceFieldSummary {
        total_source_rate: total,
        interior_source_rate: interior,
        interface_source_rate: interface,
        maximum_local_source_rate: max_local,
        source_weighted_radius: if total > 0.0 { weighted_r / total } else { 0.0 },
        fraction_inside_r_over_2: if total > 0.0 { inside_half / total } else { 0.0 },
        fraction_inside_3r_over_4: if total > 0.0 {
            inside_three_quarter / total
        } else {
            0.0
        },
        q_area,
        interior_cells,
        window_label: window_label.into(),
    };
    (q, summary)
}

pub fn tau_fill(
    interior_cells: usize,
    interior_source_rate: f64,
    initial_mean_w: f64,
) -> f64 {
    let capacity = interior_cells as f64 * (CONC_SAFETY_LIMIT - initial_mean_w).max(0.0);
    if interior_source_rate > 0.0 {
        capacity / interior_source_rate
    } else {
        f64::INFINITY
    }
}

pub fn analytical_delta_w_center(q_area: f64, radius: f64, d_w: f64) -> f64 {
    if d_w <= 0.0 {
        return f64::INFINITY;
    }
    q_area * radius * radius / (4.0 * d_w)
}

pub fn d_w_required_for_target(
    q_area: f64,
    radius: f64,
    w_target: f64,
    w_interface: f64,
) -> f64 {
    let denom = 4.0 * (w_target - w_interface);
    if denom <= 0.0 {
        return f64::INFINITY;
    }
    q_area * radius * radius / denom
}

pub fn authorized_d_w_bound(params: &SimParams) -> f64 {
    params.d_n.max(params.d_f)
}

pub fn derive_d_w_candidates(baseline: f64, d_w_required: f64, bound: f64) -> Vec<f64> {
    let mut vals = vec![
        baseline,
        0.75 * d_w_required,
        1.00 * d_w_required,
        1.25 * d_w_required,
    ];
    for v in &mut vals {
        *v = (*v).min(bound);
    }
    vals.retain(|v| v.is_finite() && *v > 0.0);
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    vals
}

pub fn d_w_candidates_within_bound(candidates: &[f64], bound: f64) -> bool {
    candidates.iter().all(|d| *d <= bound + 1e-15)
}

pub fn membrane_conductance(d_w: f64, p_w: f64, dx: f64) -> f64 {
    d_w * p_w / dx
}

pub fn analyze_membrane_conductance(
    interior_source_rate: f64,
    interface_length: f64,
    d_w: f64,
    beta_w: f64,
    params: &SimParams,
) -> MembraneConductanceAnalysis {
    let mut p_params = params.clone();
    p_params.beta_w = beta_w;
    let p_w = waste_permeability_at_m(1.0, &p_params);
    let g_w = membrane_conductance(d_w, p_w, DX);
    let j_required = if interface_length > 0.0 {
        interior_source_rate / interface_length
    } else {
        f64::INFINITY
    };
    let delta_w_required = if g_w > 0.0 {
        j_required / g_w
    } else {
        f64::INFINITY
    };
    let allowable = CONC_SAFETY_LIMIT;
    let classification = if !delta_w_required.is_finite() {
        "INSUFFICIENT"
    } else if delta_w_required < 0.25 * allowable {
        "ADEQUATE"
    } else if delta_w_required < 0.75 * allowable {
        "MARGINALLY_ADEQUATE"
    } else if delta_w_required < allowable {
        "INSUFFICIENT"
    } else {
        "INSUFFICIENT"
    };
    MembraneConductanceAnalysis {
        j_required,
        g_w,
        delta_w_required,
        allowable_delta_w: allowable,
        classification: classification.into(),
    }
}

pub fn analyze_timescales(
    source: &SourceFieldSummary,
    params: &SimParams,
    radius: f64,
    initial_mean_w: f64,
    w_interface: f64,
    pulse_tau_center_to_interface: Option<f64>,
    pulse_tau_interface_to_sink: Option<f64>,
) -> TimescaleAnalysis {
    let tf = tau_fill(source.interior_cells, source.interior_source_rate, initial_mean_w);
    let d_w = params.d_w;
    let tau_c2i = pulse_tau_center_to_interface
        .unwrap_or_else(|| if d_w > 0.0 { (radius * radius) / (4.0 * d_w) } else { f64::INFINITY });
    let p_w = waste_permeability_at_m(1.0, params);
    let tau_cross = if d_w * p_w > 0.0 {
        DX * DX / (d_w * p_w)
    } else {
        f64::INFINITY
    };
    let sink_gap = (params.waste_sink_inner_radius - radius).max(0.0);
    let tau_ext = pulse_tau_interface_to_sink.unwrap_or_else(|| {
        if d_w > 0.0 {
            (sink_gap * sink_gap) / (4.0 * d_w)
        } else {
            f64::INFINITY
        }
    });
    // Clearance timescale ~ 1/k for linear sink.
    let tau_clear = if params.reservoir_rate > 0.0 {
        1.0 / params.reservoir_rate
    } else {
        f64::INFINITY
    };
    let delta = analytical_delta_w_center(source.q_area, radius, d_w);
    let d50 = d_w_required_for_target(
        source.q_area,
        radius,
        D016_W_TARGET_FRAC * CONC_SAFETY_LIMIT,
        w_interface,
    );
    let d90 = d_w_required_for_target(
        source.q_area,
        radius,
        D016_W_CEILING_DIAG_FRAC * CONC_SAFETY_LIMIT,
        w_interface,
    );
    TimescaleAnalysis {
        tau_fill: tf,
        tau_center_to_interface: tau_c2i,
        tau_interface_crossing: tau_cross,
        tau_interface_to_sink: tau_ext,
        tau_sink_clearance: tau_clear,
        da_internal: if tf > 0.0 && tf.is_finite() {
            tau_c2i / tf
        } else {
            f64::INFINITY
        },
        da_membrane: if tf > 0.0 && tf.is_finite() {
            tau_cross / tf
        } else {
            f64::INFINITY
        },
        da_external: if tf > 0.0 && tf.is_finite() {
            tau_ext / tf
        } else {
            f64::INFINITY
        },
        da_clearance: if tf > 0.0 && tf.is_finite() {
            tau_clear / tf
        } else {
            f64::INFINITY
        },
        analytical_delta_w_center: delta,
        d_w_required_50pct: d50,
        d_w_required_90pct: d90,
        authorized_d_w_bound: authorized_d_w_bound(params),
    }
}

pub fn resistance_decomposition(times: &TimescaleAnalysis) -> ResistanceDecomposition {
    // Map diagnostic timescales to serial resistances (relative).
    let r_internal = times.tau_center_to_interface.max(0.0);
    let r_membrane = times.tau_interface_crossing.max(0.0);
    let r_external = times.tau_interface_to_sink.max(0.0);
    let r_sink = times.tau_sink_clearance.max(0.0);
    let total = r_internal + r_membrane + r_external + r_sink;
    let frac = |r: f64| if total > 0.0 { r / total } else { 0.0 };
    let fracs = [
        ("internal", frac(r_internal)),
        ("membrane", frac(r_membrane)),
        ("external", frac(r_external)),
        ("sink", frac(r_sink)),
    ];
    let dominant = fracs
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(n, _)| (*n).to_string())
        .unwrap_or_else(|| "unknown".into());
    ResistanceDecomposition {
        r_internal,
        r_membrane,
        r_external,
        r_sink,
        internal_fraction: frac(r_internal),
        membrane_fraction: frac(r_membrane),
        external_fraction: frac(r_external),
        sink_fraction: frac(r_sink),
        dominant,
    }
}

pub fn resistance_fractions_sum_to_one(r: &ResistanceDecomposition) -> bool {
    let s = r.internal_fraction + r.membrane_fraction + r.external_fraction + r.sink_fraction;
    (s - 1.0).abs() < 1e-9
}

pub fn membrane_branch_authorized(dominant: &str, membrane_fraction: f64) -> bool {
    dominant == "membrane" || membrane_fraction >= 0.25
}

/// Fixed-source W transport assay: freeze geometry, inject q_W, evolve W only.
pub fn run_fixed_source_assay(
    grid: &Grid,
    phi: &[f64],
    membrane: &[f64],
    q_w: &[f64],
    params: &SimParams,
    initial_waste: Option<&[f64]>,
    max_steps: u64,
) -> FixedSourceAssayResult {
    let n = grid.width * grid.height;
    let mut waste = vec![0.0; n];
    if let Some(w0) = initial_waste {
        waste.copy_from_slice(w0);
    }
    let mut next = waste.clone();
    let mut transport_rate = vec![0.0; n];
    let mut nutrient = vec![0.0; n];
    let mut fuel = vec![0.0; n];
    let dt = MAX_DT;
    let mut accepted = 0u64;
    let mut t = 0.0;
    let mut sink_removed = 0.0;
    let mut source_injected = 0.0;
    let cx = grid.width / 2;
    let cy = grid.height / 2;
    let center = Grid::index(grid.width, cx, cy);

    let mean_q: f64 = q_w.iter().sum();

    loop {
        if accepted >= max_steps {
            let sink_rate = if t > 0.0 { sink_removed / t } else { 0.0 };
            let src_rate = if t > 0.0 { source_injected / t } else { mean_q };
            let match_ok = src_rate > 0.0
                && ((sink_rate - src_rate).abs() / src_rate) <= D016_SINK_MATCH_TOL;
            let class = if match_ok && waste[center] < D016_W_TARGET_FRAC * CONC_SAFETY_LIMIT {
                "FINITE_TRANSPORT_STEADY_STATE"
            } else if match_ok {
                "SLOW_TRANSPORT_CONVERGENCE"
            } else {
                "CONCENTRATION_BOUND_REACHED"
            };
            // Refine classification when still accumulating at horizon.
            let class = if waste.iter().copied().fold(0.0_f64, f64::max) >= CONC_SAFETY_LIMIT - 1e-9
            {
                "CONCENTRATION_BOUND_REACHED"
            } else if !match_ok {
                "INTERIOR_DIFFUSION_FAILURE"
            } else {
                class
            };
            return FixedSourceAssayResult {
                classification: class.into(),
                accepted_substeps: accepted,
                simulated_time: t,
                center_w: waste[center],
                max_w: waste.iter().copied().fold(0.0, f64::max),
                mean_source_rate: src_rate,
                mean_sink_removal_rate: sink_rate,
                sink_matches_source: match_ok,
                d_w: params.d_w,
                beta_w: params.beta_w,
            };
        }

        let max_w = waste.iter().copied().fold(0.0, f64::max);
        if max_w >= CONC_SAFETY_LIMIT - 1e-12 {
            return FixedSourceAssayResult {
                classification: "CONCENTRATION_BOUND_REACHED".into(),
                accepted_substeps: accepted,
                simulated_time: t,
                center_w: waste[center],
                max_w,
                mean_source_rate: if t > 0.0 {
                    source_injected / t
                } else {
                    mean_q
                },
                mean_sink_removal_rate: if t > 0.0 { sink_removed / t } else { 0.0 },
                sink_matches_source: false,
                d_w: params.d_w,
                beta_w: params.beta_w,
            };
        }

        let _acct = transport_field(
            grid,
            TransportSpecies::Waste,
            &waste,
            phi,
            membrane,
            params,
            &mut transport_rate,
        );
        next.copy_from_slice(&waste);
        for idx in 0..n {
            if !grid.in_dish(idx) {
                next[idx] = 0.0;
                continue;
            }
            let inj = q_w[idx] * dt;
            source_injected += inj;
            next[idx] = waste[idx] + dt * transport_rate[idx] + inj;
            if next[idx] < 0.0 {
                next[idx] = 0.0;
            }
        }
        let mut before_sink = next.clone();
        apply_reservoir(grid, &mut nutrient, &mut fuel, &mut next, dt, params);
        let mut cleared_exact = 0.0;
        for idx in 0..n {
            if waste_sink_cell(grid, idx, params) {
                cleared_exact += (before_sink[idx] - next[idx]).max(0.0);
            }
        }
        let _ = before_sink;
        sink_removed += cleared_exact;
        waste.copy_from_slice(&next);
        accepted += 1;
        t += dt;
    }
}

pub fn classify_passive_feasibility(
    assay: &FixedSourceAssayResult,
    at_authorized_bound: bool,
    beta_at_zero: bool,
) -> PassiveTransportFeasibility {
    let center_ok = assay.center_w < D016_W_TARGET_FRAC * CONC_SAFETY_LIMIT;
    let steady = assay.classification == "FINITE_TRANSPORT_STEADY_STATE"
        || (assay.sink_matches_source && center_ok);
    if steady && center_ok && assay.sink_matches_source {
        return PassiveTransportFeasibility::Feasible;
    }
    if at_authorized_bound && beta_at_zero {
        PassiveTransportFeasibility::Insufficient
    } else {
        PassiveTransportFeasibility::Insufficient
    }
}

pub fn d016_preflight_requires_closed_waste_budget(waste_budget_ok: bool) -> bool {
    waste_budget_ok
}

pub fn solver_requires_quasi_steady_biological_reference(
    artifact_valid: bool,
    quasi_steady: bool,
) -> bool {
    artifact_valid && quasi_steady
}

pub fn transport_schema_for_repair(pass: bool) -> u32 {
    if pass {
        crate::config::TRANSPORT_SCHEMA_VERSION_V2
    } else {
        TRANSPORT_SCHEMA_VERSION_V1
    }
}

/// Interface circumference proxy for prescribed circle (cell-length units).
pub fn interface_length_proxy(radius: f64) -> f64 {
    std::f64::consts::TAU * radius
}

pub fn probe_linear_sink_idle(grid: &Grid, waste: &[f64], params: &SimParams) -> f64 {
    linear_sink_clearance_rate(grid, waste, params)
}

#[cfg(test)]
mod inline_smoke {
    use super::*;

    #[test]
    fn fractions_sum() {
        let t = TimescaleAnalysis {
            tau_fill: 1.0,
            tau_center_to_interface: 4.0,
            tau_interface_crossing: 1.0,
            tau_interface_to_sink: 3.0,
            tau_sink_clearance: 2.0,
            da_internal: 4.0,
            da_membrane: 1.0,
            da_external: 3.0,
            da_clearance: 2.0,
            analytical_delta_w_center: 1.0,
            d_w_required_50pct: 1.0,
            d_w_required_90pct: 1.0,
            authorized_d_w_bound: 0.18,
        };
        let r = resistance_decomposition(&t);
        assert!(resistance_fractions_sum_to_one(&r));
    }
}
