//! D-026 Stage E observability schema and dynamic/constrained chemistry parity (Gates 0–1).

use crate::accounting::STEP_RESIDUAL_TOL;
use crate::config::SimParams;
use crate::constraint_accounting::CONSTRAINT_RESIDUAL_TOL;
use crate::d011_analysis::StageEReferenceRates;
use crate::d025_analysis::{
    clamp_productive_to_global, D025_GLOBAL_RATE_MAX_FACTOR, D025_GLOBAL_RATE_MIN_FACTOR,
    D025_MAX_CANDIDATES, D025_PRODUCTIVE_NAMES, D025_ROUND_RATE_MAX_FACTOR,
    D025_ROUND_RATE_MIN_FACTOR, D025ProductiveRates,
};
use crate::field_mass;
use crate::fields::field_sha256_stable;
use crate::simulation::Simulation;
use crate::surface_density::{
    compute_interface_geometry, integrated_delta, reconstruct_gamma_field, theta_gamma,
    total_surface_mass, InterfaceGeometryCell,
};
use serde::{Deserialize, Serialize};

pub const D026_PARITY_ABS_TOL: f64 = 1e-10;
pub const D026_PARITY_REL_TOL: f64 = 1e-8;
pub const D026_ADVECTION_ABS_TOL: f64 = 1e-12;
pub const D026_SETTLE_STEPS: u64 = 20;
pub const D026_GLOBAL_RATE_MIN_FACTOR: f64 = D025_GLOBAL_RATE_MIN_FACTOR;
pub const D026_GLOBAL_RATE_MAX_FACTOR: f64 = D025_GLOBAL_RATE_MAX_FACTOR;
pub const D026_ROUND_RATE_MIN_FACTOR: f64 = D025_ROUND_RATE_MIN_FACTOR;
pub const D026_ROUND_RATE_MAX_FACTOR: f64 = D025_ROUND_RATE_MAX_FACTOR;
pub const D026_MAX_CANDIDATES: usize = D025_MAX_CANDIDATES;

pub const D026_CHRONOLOGY_REL_TOL: f64 = 0.05;
pub const D026_GATE5_DIAGNOSTIC_STEPS: u64 = 3_000;
pub const D026_GATE5_SLOPE_WINDOW: u64 = 500;
pub const D026_REFERENCE_CHECKPOINTS: [u64; 6] =
    [10_000, 25_000, 50_000, 100_000, 150_000, 200_000];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D026Conclusion {
    D026Gate0ParityPass,
    D026Gate0ParityFail,
    D026Gate1ObservabilityReady,
    D026Gate2HistoryReady,
    D026Gate5ControlsReady,
    D026MechanismIdentified,
    D026StageEHarnessDefect,
    D026AnalyticSeedTransient,
    D026CorrectionApplied,
    D026StageERecovered,
    D026Fail,
}

impl D026Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::D026Gate0ParityPass => "D026_GATE0_PARITY_PASS",
            Self::D026Gate0ParityFail => "D026_GATE0_PARITY_FAIL",
            Self::D026Gate1ObservabilityReady => "D026_GATE1_OBSERVABILITY_READY",
            Self::D026Gate2HistoryReady => "D026_GATE2_HISTORY_READY",
            Self::D026Gate5ControlsReady => "D026_GATE5_CONTROLS_READY",
            Self::D026MechanismIdentified => "D026_MECHANISM_IDENTIFIED",
            Self::D026StageEHarnessDefect => "D026_STAGE_E_HARNESS_DEFECT",
            Self::D026AnalyticSeedTransient => "D026_ANALYTIC_SEED_TRANSIENT",
            Self::D026CorrectionApplied => "D026_CORRECTION_APPLIED",
            Self::D026StageERecovered => "D026_STAGE_E_RECOVERED",
            Self::D026Fail => "D026_FAIL",
        }
    }
}

/// Gate 2 chronology: earliest upstream divergence label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D026ChronologyLabel {
    SurfaceCoverageDecline,
    ActivationCapacityDecline,
    StructuralDemandExcess,
    CatalystDemandExcess,
    PrecursorDemandExcess,
    ALeakageIncrease,
    InitialStateDivergence,
    OscillatoryOnset,
    MonotonicSlowDrift,
    Unknown,
}

impl D026ChronologyLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SurfaceCoverageDecline => "SURFACE_COVERAGE_DECLINE",
            Self::ActivationCapacityDecline => "ACTIVATION_CAPACITY_DECLINE",
            Self::StructuralDemandExcess => "STRUCTURAL_DEMAND_EXCESS",
            Self::CatalystDemandExcess => "CATALYST_DEMAND_EXCESS",
            Self::PrecursorDemandExcess => "PRECURSOR_DEMAND_EXCESS",
            Self::ALeakageIncrease => "A_LEAKAGE_INCREASE",
            Self::InitialStateDivergence => "INITIAL_STATE_DIVERGENCE",
            Self::OscillatoryOnset => "OSCILLATORY_ONSET",
            Self::MonotonicSlowDrift => "MONOTONIC_SLOW_DRIFT",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Gate 6 / §19 mechanism labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D026MechanismLabel {
    Unknown,
    StageEHarnessDefect,
    AnalyticSeedTransient,
    SurfaceCoverageMaintenanceDeficit,
    ActivationCapacityDeficit,
    StructuralADemandExcess,
    CatalystADemandExcess,
    PrecursorADemandExcess,
    ABoundaryLeakage,
    CoupledOscillation,
    NoSingleDominantMechanism,
    TrueLongTransient,
}

impl D026MechanismLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::StageEHarnessDefect => "STAGE_E_HARNESS_DEFECT",
            Self::AnalyticSeedTransient => "ANALYTIC_SEED_TRANSIENT",
            Self::SurfaceCoverageMaintenanceDeficit => "SURFACE_COVERAGE_MAINTENANCE_DEFICIT",
            Self::ActivationCapacityDeficit => "ACTIVATION_CAPACITY_DEFICIT",
            Self::StructuralADemandExcess => "STRUCTURAL_A_DEMAND_EXCESS",
            Self::CatalystADemandExcess => "CATALYST_A_DEMAND_EXCESS",
            Self::PrecursorADemandExcess => "PRECURSOR_A_DEMAND_EXCESS",
            Self::ABoundaryLeakage => "A_BOUNDARY_LEAKAGE",
            Self::CoupledOscillation => "COUPLED_OSCILLATION",
            Self::NoSingleDominantMechanism => "NO_SINGLE_DOMINANT_MECHANISM",
            Self::TrueLongTransient => "TRUE_LONG_TRANSIENT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceCoverageStats {
    pub mean_gamma: f64,
    pub median_gamma: f64,
    pub min_gamma: f64,
    pub p10_gamma: f64,
    pub p25_gamma: f64,
    pub p50_gamma: f64,
    pub p75_gamma: f64,
    pub p90_gamma: f64,
    pub fraction_below_0_25_gamma_ref: f64,
    pub fraction_below_0_50_gamma_ref: f64,
    pub fraction_below_0_75_gamma_ref: f64,
    pub mean_theta_gamma: f64,
    pub min_theta_gamma: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEObservabilitySample {
    pub step: u64,
    pub sim_time: f64,
    pub mass_c: f64,
    pub mass_a: f64,
    pub mass_p: f64,
    pub mass_s: f64,
    pub interior_c: f64,
    pub exterior_c: f64,
    pub interior_a: f64,
    pub exterior_a: f64,
    pub interior_p: f64,
    pub exterior_p: f64,
    pub mean_internal_n: f64,
    pub mean_internal_f: f64,
    pub mean_internal_w: f64,
    pub max_c: f64,
    pub min_c: f64,
    pub max_a: f64,
    pub min_a: f64,
    pub structural_mass: f64,
    pub interface_measure: f64,
    pub surface: SurfaceCoverageStats,
    pub adsorption_rate_step: f64,
    pub gamma_loss_rate_step: f64,
    pub tangential_flux_placeholder: f64,
    pub normal_leakage_diagnostic_placeholder: f64,
    pub a_production_activation: f64,
    pub a_consumption_catalyst_reproduction: f64,
    pub a_consumption_precursor_production: f64,
    pub a_consumption_virtual_structural: f64,
    pub a_consumption_decay: f64,
    pub a_transport_in_flux: f64,
    pub a_transport_out_flux: f64,
    pub a_transport_net_interface: f64,
    pub a_numerical_correction: f64,
    pub a_observed_mass_change: f64,
    pub a_predicted_mass_change: f64,
    pub a_transport_residual: f64,
    pub a_retention: f64,
    pub interior_a_concentration: f64,
    pub exterior_a_concentration: f64,
    pub outward_leakage_per_interface: f64,
    pub a_residence_time_placeholder: f64,
    pub activation_to_demand: f64,
    pub activation_to_leakage: f64,
}

fn structural_a_consumption_extent(sim: &Simulation) -> f64 {
    let constraint = &sim.constraint_accounting.last_step;
    let eta_phi = if sim.params.equation_version.is_conservative_membrane_metabolism() {
        sim.params.eta_phi.max(f64::EPSILON)
    } else {
        1.0
    };
    constraint.virtual_production / eta_phi
}

fn precursor_a_consumption_extent(sim: &Simulation) -> f64 {
    sim.last_surface_totals
        .map(|t| t.precursor_synthesis_delta)
        .unwrap_or(0.0)
}

pub fn classify_mechanism(sample: &StageEObservabilitySample) -> D026MechanismLabel {
    if sample.a_retention < 0.45 && sample.surface.mean_theta_gamma < 0.35 {
        return D026MechanismLabel::SurfaceCoverageMaintenanceDeficit;
    }
    if sample.activation_to_demand < 0.5 {
        return D026MechanismLabel::ActivationCapacityDeficit;
    }
    if sample.a_consumption_virtual_structural
        > sample
            .a_consumption_catalyst_reproduction
            .max(sample.a_consumption_precursor_production)
    {
        return D026MechanismLabel::StructuralADemandExcess;
    }
    if sample.a_consumption_catalyst_reproduction > sample.a_consumption_precursor_production {
        return D026MechanismLabel::CatalystADemandExcess;
    }
    if sample.a_consumption_precursor_production > sample.a_production_activation {
        return D026MechanismLabel::PrecursorADemandExcess;
    }
    if sample.outward_leakage_per_interface > sample.a_production_activation * 0.01 {
        return D026MechanismLabel::ABoundaryLeakage;
    }
    D026MechanismLabel::Unknown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedConservationReport {
    pub total_cnpfwas: f64,
    pub within_seed_tol: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceHistoryPoint {
    pub checkpoint_step: u64,
    pub source: String,
    pub sample: StageEObservabilitySample,
    pub q_activated: Option<f64>,
    pub q_membrane: Option<f64>,
    pub total_a_demand: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceHistoryReport {
    pub checkpoints_available: bool,
    pub fallback_diagnostic: bool,
    pub earliest_divergence: D026ChronologyLabel,
    pub points: Vec<ReferenceHistoryPoint>,
    pub rolling_window_slopes: Option<serde_json::Value>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalControlMetrics {
    pub label: String,
    pub accepted_steps: u64,
    pub a_slope: f64,
    pub a_retention_end: f64,
    pub a_leakage_end: f64,
    pub activation_mean: f64,
    pub total_a_demand_mean: f64,
    pub theta_gamma_end: f64,
    pub mass_s_slope: f64,
    pub mass_c_slope: f64,
    pub mass_p_slope: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalControlsReport {
    pub diagnostic_only: bool,
    pub horizon_steps: u64,
    pub baseline: CausalControlMetrics,
    pub controls: Vec<CausalControlMetrics>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismClassificationReport {
    pub gate6_mechanism: D026MechanismLabel,
    pub chronology: D026ChronologyLabel,
    pub evidence: Vec<String>,
    pub gate7_continuation_warranted: bool,
    pub gate8_rate_correction_warranted: bool,
    pub suggested_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityMetricDiff {
    pub name: String,
    pub path_a: f64,
    pub path_b: f64,
    pub abs_diff: f64,
    pub within_tolerance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerParityReport {
    pub path_a_accepted: bool,
    pub path_b_accepted: bool,
    pub path_a_reject_detail: String,
    pub path_b_reject_detail: String,
    pub path_a_dt: f64,
    pub path_b_dt: f64,
    pub gate0_pass: bool,
    pub chemistry_parity_pass: bool,
    pub allowed_differences_ok: bool,
    pub max_abs_diff: f64,
    pub max_abs_diff_metric: String,
    pub diffs: Vec<ParityMetricDiff>,
    pub path_a_phi_changed: bool,
    pub path_b_phi_unchanged: bool,
    pub path_a_constraint_flux_zero: bool,
    pub path_b_constraint_isolated: bool,
    pub path_b_advection_disabled: bool,
    pub surface_mass_parity: bool,
    pub surface_mass_abs_diff: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct StepParitySnapshot {
    delta_c: f64,
    delta_n: f64,
    delta_f: f64,
    delta_w: f64,
    delta_a: f64,
    delta_p: f64,
    delta_s: f64,
    activation: f64,
    reproduction: f64,
    activated_decay: f64,
    catalyst_turnover: f64,
    virtual_production: f64,
    virtual_decay: f64,
    adsorption: f64,
    gamma_turnover: f64,
    membrane_diffusion_net: f64,
    transport_c: f64,
    transport_a: f64,
    transport_n: f64,
    transport_f: f64,
    transport_w: f64,
    n_reservoir_delta: f64,
    f_reservoir_delta: f64,
    w_reservoir_delta: f64,
    constraint_flux: f64,
    constraint_residual: f64,
    virtual_net: f64,
    phi_hash: String,
    phi_mass_delta: f64,
}

pub fn parity_within(a: f64, b: f64) -> bool {
    let diff = (a - b).abs();
    let scale = a.abs().max(b.abs()).max(1.0);
    diff <= D026_PARITY_ABS_TOL + D026_PARITY_REL_TOL * scale
}

pub fn record_metric_diff(
    diffs: &mut Vec<ParityMetricDiff>,
    name: &str,
    path_a: f64,
    path_b: f64,
) -> (f64, bool) {
    let abs_diff = (path_a - path_b).abs();
    let within = parity_within(path_a, path_b);
    diffs.push(ParityMetricDiff {
        name: name.to_string(),
        path_a,
        path_b,
        abs_diff,
        within_tolerance: within,
    });
    (abs_diff, within)
}

fn interior_exterior_mean(sim: &Simulation, field: &[f64]) -> (f64, f64) {
    let mut in_sum = 0.0;
    let mut in_area = 0.0_f64;
    let mut out_sum = 0.0;
    let mut out_area = 0.0_f64;
    for (idx, &value) in field.iter().enumerate() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        if sim.fields.structure[idx] >= 0.5 {
            in_sum += value;
            in_area += 1.0;
        } else {
            out_sum += value;
            out_area += 1.0;
        }
    }
    (
        in_sum / in_area.max(1.0),
        out_sum / out_area.max(1.0),
    )
}

fn interior_mean(sim: &Simulation, field: &[f64]) -> f64 {
    interior_exterior_mean(sim, field).0
}

fn retention(sim: &Simulation, field: &[f64]) -> f64 {
    let mut inside = 0.0;
    for (idx, &value) in field.iter().enumerate() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            inside += value;
        }
    }
    inside / field_mass(&sim.grid, field).max(f64::EPSILON)
}

fn field_min_max(sim: &Simulation, field: &[f64]) -> (f64, f64) {
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    for (idx, &value) in field.iter().enumerate() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        min_v = min_v.min(value);
        max_v = max_v.max(value);
    }
    if min_v.is_infinite() {
        (0.0, 0.0)
    } else {
        (min_v, max_v)
    }
}

fn structural_mass(sim: &Simulation) -> f64 {
    sim.fields
        .structure
        .iter()
        .enumerate()
        .filter(|(idx, _)| sim.grid.in_dish(*idx))
        .map(|(_, &phi)| if phi >= 0.5 { 1.0 } else { 0.0 })
        .sum()
}

fn interface_measure(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    integrated_delta(&sim.grid, &geometry)
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = (p * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn surface_coverage_stats(sim: &Simulation) -> SurfaceCoverageStats {
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
    let gamma_ref = sim.params.gamma_reference.max(f64::EPSILON);
    let mut gammas = Vec::new();
    let mut thetas = Vec::new();
    for idx in 0..n {
        if geometry[idx].delta > sim.params.delta_floor {
            gammas.push(gamma[idx]);
            thetas.push(theta_gamma(gamma[idx], gamma_ref));
        }
    }
    gammas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let count = gammas.len().max(1) as f64;
    let mean_gamma = gammas.iter().sum::<f64>() / count;
    let frac_below = |threshold: f64| {
        gammas.iter().filter(|&&g| g < threshold * gamma_ref).count() as f64 / count
    };
    let mean_theta = if thetas.is_empty() {
        0.0
    } else {
        thetas.iter().sum::<f64>() / thetas.len() as f64
    };
    let min_theta = thetas.iter().copied().fold(f64::INFINITY, f64::min);
    SurfaceCoverageStats {
        mean_gamma,
        median_gamma: percentile(&gammas, 0.5),
        min_gamma: gammas.first().copied().unwrap_or(0.0),
        p10_gamma: percentile(&gammas, 0.10),
        p25_gamma: percentile(&gammas, 0.25),
        p50_gamma: percentile(&gammas, 0.50),
        p75_gamma: percentile(&gammas, 0.75),
        p90_gamma: percentile(&gammas, 0.90),
        fraction_below_0_25_gamma_ref: frac_below(0.25),
        fraction_below_0_50_gamma_ref: frac_below(0.50),
        fraction_below_0_75_gamma_ref: frac_below(0.75),
        mean_theta_gamma: if mean_theta.is_finite() { mean_theta } else { 0.0 },
        min_theta_gamma: if min_theta.is_finite() { min_theta } else { 0.0 },
    }
}

pub fn mean_step_a_budget_from_cumulative(sim: &Simulation) -> (f64, f64, f64, f64, f64) {
    let n = sim.substep.max(1) as f64;
    let metab = &sim.metabolism_accounting.cumulative;
    let constraint = &sim.constraint_accounting.cumulative;
    let eta_phi = if sim.params.equation_version.is_conservative_membrane_metabolism() {
        sim.params.eta_phi.max(f64::EPSILON)
    } else {
        1.0
    };
    let virtual_a = constraint.virtual_production / eta_phi / n;
    let precursor_a = sim
        .last_surface_totals
        .map(|t| t.precursor_synthesis_delta)
        .unwrap_or(0.0);
    (
        metab.activation / n,
        metab.reproduction / n,
        precursor_a,
        virtual_a,
        metab.activated_decay / n,
    )
}

pub fn sample_stage_e_observability(sim: &Simulation) -> StageEObservabilitySample {
    let (interior_c, exterior_c) = interior_exterior_mean(sim, &sim.fields.catalyst);
    let (interior_a, exterior_a) = interior_exterior_mean(sim, &sim.fields.activated);
    let (interior_p, exterior_p) = interior_exterior_mean(sim, &sim.fields.precursor);
    let (min_c, max_c) = field_min_max(sim, &sim.fields.catalyst);
    let (min_a, max_a) = field_min_max(sim, &sim.fields.activated);
    let mass_a = field_mass(&sim.grid, &sim.fields.activated);
    let a_ret = retention(sim, &sim.fields.activated);
    let iface = interface_measure(sim).max(f64::EPSILON);
    let metab = &sim.metabolism_accounting.last_step;
    let mem = &sim.membrane_accounting.last_step;
    let transport = &sim.transport_accounting.last_step;
    let activated_ledger = &sim.accounting.last_step.activated;
    let _constraint = &sim.constraint_accounting.last_step;
    let dt = sim.dt.max(f64::EPSILON);
    let a_in = transport.activated.interior_net_flux_rate.max(0.0) * dt;
    let a_out = (-transport.activated.interior_net_flux_rate).max(0.0) * dt;
    let a_net_iface = transport.activated.interior_net_flux_rate * dt;
    let a_observed = activated_ledger.mass_after - activated_ledger.mass_before;
    let a_predicted = activated_ledger.reaction_delta
        + activated_ledger.diffusion_delta
        + activated_ledger.reservoir_delta;
    let virtual_a = structural_a_consumption_extent(sim);
    let precursor_a = precursor_a_consumption_extent(sim);
    let activation = metab.activation;
    let demand = virtual_a + metab.reproduction + precursor_a + metab.activated_decay;
    let mut sample = StageEObservabilitySample {
        step: sim.substep,
        sim_time: sim.sim_time,
        mass_c: field_mass(&sim.grid, &sim.fields.catalyst),
        mass_a,
        mass_p: field_mass(&sim.grid, &sim.fields.precursor),
        mass_s: total_surface_mass(&sim.grid, &sim.fields.membrane),
        interior_c,
        exterior_c,
        interior_a,
        exterior_a,
        interior_p,
        exterior_p,
        mean_internal_n: interior_mean(sim, &sim.fields.nutrient),
        mean_internal_f: interior_mean(sim, &sim.fields.fuel),
        mean_internal_w: interior_mean(sim, &sim.fields.waste),
        max_c,
        min_c,
        max_a,
        min_a,
        structural_mass: structural_mass(sim),
        interface_measure: iface,
        surface: surface_coverage_stats(sim),
        adsorption_rate_step: mem.synthesis / dt,
        gamma_loss_rate_step: mem.decay / dt,
        tangential_flux_placeholder: 0.0,
        normal_leakage_diagnostic_placeholder: 0.0,
        a_production_activation: metab.activation,
        a_consumption_catalyst_reproduction: metab.reproduction,
        a_consumption_precursor_production: precursor_a,
        a_consumption_virtual_structural: virtual_a,
        a_consumption_decay: metab.activated_decay,
        a_transport_in_flux: a_in,
        a_transport_out_flux: a_out,
        a_transport_net_interface: a_net_iface,
        a_numerical_correction: activated_ledger.numerical_correction_delta,
        a_observed_mass_change: a_observed,
        a_predicted_mass_change: a_predicted,
        a_transport_residual: activated_ledger.accounting_residual,
        a_retention: a_ret,
        interior_a_concentration: interior_a,
        exterior_a_concentration: exterior_a,
        outward_leakage_per_interface: a_out / iface,
        a_residence_time_placeholder: mass_a / a_out.max(f64::EPSILON),
        activation_to_demand: activation / demand.max(f64::EPSILON),
        activation_to_leakage: activation / a_out.max(f64::EPSILON),
    };
    if sample.a_production_activation.abs() < 1e-12 && sim.substep > 0 {
        let (act, repro, prec, virt, dec) = mean_step_a_budget_from_cumulative(sim);
        sample.a_production_activation = act;
        sample.a_consumption_catalyst_reproduction = repro;
        sample.a_consumption_precursor_production = prec;
        sample.a_consumption_virtual_structural = virt;
        sample.a_consumption_decay = dec;
        let demand = virt + repro + prec + dec;
        sample.activation_to_demand = act / demand.max(f64::EPSILON);
    }
    sample
}

/// ponytail: linear sum of C/N/F/W/A/P/S masses only; no cross-term stoichiometry check yet.
pub fn analytic_seed_conservation_check(sim: &Simulation) -> SeedConservationReport {
    let total = field_mass(&sim.grid, &sim.fields.catalyst)
        + field_mass(&sim.grid, &sim.fields.nutrient)
        + field_mass(&sim.grid, &sim.fields.fuel)
        + field_mass(&sim.grid, &sim.fields.waste)
        + field_mass(&sim.grid, &sim.fields.activated)
        + field_mass(&sim.grid, &sim.fields.precursor)
        + total_surface_mass(&sim.grid, &sim.fields.membrane);
    SeedConservationReport {
        total_cnpfwas: total,
        within_seed_tol: total.is_finite() && total > 0.0,
        note: "stub: finite positive total mass only".into(),
    }
}

pub fn productive_rates_within_global_bounds(
    candidate: &D025ProductiveRates,
    reference: &D025ProductiveRates,
) -> bool {
    let clamped = clamp_productive_to_global(candidate, reference);
    let c = candidate.to_vector();
    let r = clamped.to_vector();
    c.iter().zip(r.iter()).all(|(a, b)| (*a - *b).abs() <= STEP_RESIDUAL_TOL)
}

pub fn productive_rates_within_round_bounds(factor: f64) -> bool {
    (D026_ROUND_RATE_MIN_FACTOR..=D026_ROUND_RATE_MAX_FACTOR).contains(&factor)
}

pub fn global_rate_bounds_ok(factor: f64) -> bool {
    (D026_GLOBAL_RATE_MIN_FACTOR..=D026_GLOBAL_RATE_MAX_FACTOR).contains(&factor)
}

pub fn apply_stage_e_reference_rates(params: &mut SimParams, rates: &StageEReferenceRates) {
    rates.apply_to(params);
}

fn capture_parity_snapshot(sim: &Simulation, before: &PreStepMasses) -> StepParitySnapshot {
    let dt = sim.dt;
    let transport = &sim.transport_accounting.last_step;
    let metab = &sim.metabolism_accounting.last_step;
    let mem = &sim.membrane_accounting.last_step;
    let constraint = &sim.constraint_accounting.last_step;
    let ledgers = &sim.accounting.last_step;
    StepParitySnapshot {
        delta_c: field_mass(&sim.grid, &sim.fields.catalyst) - before.c,
        delta_n: field_mass(&sim.grid, &sim.fields.nutrient) - before.n,
        delta_f: field_mass(&sim.grid, &sim.fields.fuel) - before.f,
        delta_w: field_mass(&sim.grid, &sim.fields.waste) - before.w,
        delta_a: field_mass(&sim.grid, &sim.fields.activated) - before.a,
        delta_p: field_mass(&sim.grid, &sim.fields.precursor) - before.p,
        delta_s: total_surface_mass(&sim.grid, &sim.fields.membrane) - before.s,
        activation: metab.activation,
        reproduction: metab.reproduction,
        activated_decay: metab.activated_decay,
        catalyst_turnover: metab.catalyst_turnover,
        virtual_production: constraint.virtual_production,
        virtual_decay: constraint.virtual_decay,
        adsorption: mem.synthesis,
        gamma_turnover: mem.decay,
        membrane_diffusion_net: mem.diffusion_net,
        transport_c: transport.catalyst.net_change_rate * dt,
        transport_a: transport.activated.net_change_rate * dt,
        transport_n: transport.nutrient.net_change_rate * dt,
        transport_f: transport.fuel.net_change_rate * dt,
        transport_w: transport.waste.net_change_rate * dt,
        n_reservoir_delta: ledgers.nutrient.reservoir_delta,
        f_reservoir_delta: ledgers.fuel.reservoir_delta,
        w_reservoir_delta: ledgers.waste.reservoir_delta,
        constraint_flux: constraint.constraint_flux,
        constraint_residual: constraint.residual,
        virtual_net: constraint.virtual_net,
        phi_hash: field_sha256_stable(&sim.fields.structure),
        phi_mass_delta: field_mass(&sim.grid, &sim.fields.structure) - before.phi,
    }
}

#[derive(Debug, Clone)]
struct PreStepMasses {
    c: f64,
    n: f64,
    f: f64,
    w: f64,
    a: f64,
    p: f64,
    s: f64,
    phi: f64,
    phi_hash: String,
}

fn pre_step_masses(sim: &Simulation) -> PreStepMasses {
    PreStepMasses {
        c: field_mass(&sim.grid, &sim.fields.catalyst),
        n: field_mass(&sim.grid, &sim.fields.nutrient),
        f: field_mass(&sim.grid, &sim.fields.fuel),
        w: field_mass(&sim.grid, &sim.fields.waste),
        a: field_mass(&sim.grid, &sim.fields.activated),
        p: field_mass(&sim.grid, &sim.fields.precursor),
        s: total_surface_mass(&sim.grid, &sim.fields.membrane),
        phi: field_mass(&sim.grid, &sim.fields.structure),
        phi_hash: field_sha256_stable(&sim.fields.structure),
    }
}

pub fn settle_constrained(sim: &mut Simulation, steps: u64) -> u64 {
    sim.enforce_structure_constraint = true;
    let mut accepted = 0u64;
    for _ in 0..steps {
        if sim.step() {
            accepted += 1;
        }
    }
    accepted
}

pub fn run_runner_parity(base: &Simulation) -> RunnerParityReport {
    let mut path_a = base.clone();
    let mut path_b = base.clone();
    path_a.enforce_structure_constraint = false;
    path_b.enforce_structure_constraint = true;
    path_a.observer_enabled = false;
    path_b.observer_enabled = false;

    let before_a = pre_step_masses(&path_a);
    let before_b = pre_step_masses(&path_b);
    let phi_hash_b_before = before_b.phi_hash.clone();

    let path_a_accepted = path_a.step();
    let path_b_accepted = path_b.step();

    let snap_a = capture_parity_snapshot(&path_a, &before_a);
    let snap_b = capture_parity_snapshot(&path_b, &before_b);

    let mut diffs = Vec::new();
    let mut max_abs_diff = 0.0;
    let mut max_abs_diff_metric = String::new();
    let mut chemistry_pass = path_a_accepted && path_b_accepted;

    let metrics: [(&str, f64, f64); 23] = [
        ("delta_c", snap_a.delta_c, snap_b.delta_c),
        ("delta_n", snap_a.delta_n, snap_b.delta_n),
        ("delta_f", snap_a.delta_f, snap_b.delta_f),
        ("delta_w", snap_a.delta_w, snap_b.delta_w),
        ("delta_a", snap_a.delta_a, snap_b.delta_a),
        ("delta_p", snap_a.delta_p, snap_b.delta_p),
        ("delta_s", snap_a.delta_s, snap_b.delta_s),
        ("activation", snap_a.activation, snap_b.activation),
        ("reproduction", snap_a.reproduction, snap_b.reproduction),
        ("activated_decay", snap_a.activated_decay, snap_b.activated_decay),
        ("catalyst_turnover", snap_a.catalyst_turnover, snap_b.catalyst_turnover),
        (
            "virtual_production",
            snap_a.virtual_production,
            snap_b.virtual_production,
        ),
        ("virtual_decay", snap_a.virtual_decay, snap_b.virtual_decay),
        ("adsorption", snap_a.adsorption, snap_b.adsorption),
        ("gamma_turnover", snap_a.gamma_turnover, snap_b.gamma_turnover),
        ("transport_c", snap_a.transport_c, snap_b.transport_c),
        ("transport_a", snap_a.transport_a, snap_b.transport_a),
        ("transport_n", snap_a.transport_n, snap_b.transport_n),
        ("transport_f", snap_a.transport_f, snap_b.transport_f),
        ("transport_w", snap_a.transport_w, snap_b.transport_w),
        ("n_reservoir_delta", snap_a.n_reservoir_delta, snap_b.n_reservoir_delta),
        ("f_reservoir_delta", snap_a.f_reservoir_delta, snap_b.f_reservoir_delta),
        ("w_reservoir_delta", snap_a.w_reservoir_delta, snap_b.w_reservoir_delta),
    ];

    for (name, a, b) in metrics {
        let (abs_diff, within) = record_metric_diff(&mut diffs, name, a, b);
        if !within {
            chemistry_pass = false;
        }
        if abs_diff > max_abs_diff {
            max_abs_diff = abs_diff;
            max_abs_diff_metric = name.to_string();
        }
    }

    let path_a_phi_changed = snap_a.phi_hash != before_a.phi_hash
        || snap_a.phi_mass_delta.abs() > D026_PARITY_ABS_TOL;
    let path_b_phi_unchanged =
        snap_b.phi_hash == phi_hash_b_before && snap_b.phi_mass_delta.abs() <= D026_PARITY_ABS_TOL;
    let path_a_constraint_flux_zero = snap_a.constraint_flux.abs() <= CONSTRAINT_RESIDUAL_TOL;
    let path_b_constraint_isolated = snap_b.constraint_residual.abs()
        <= CONSTRAINT_RESIDUAL_TOL * snap_b.virtual_net.abs().max(1.0)
        && parity_within(snap_b.constraint_flux, -snap_b.virtual_net);
    let advection_gap = (snap_a.membrane_diffusion_net - snap_b.membrane_diffusion_net).abs();
    let path_b_advection_disabled = path_b_phi_unchanged;
    let surface_mass_abs_diff = (snap_a.delta_s - snap_b.delta_s).abs();
    let surface_mass_parity = parity_within(snap_a.delta_s, snap_b.delta_s);

    let mut notes = Vec::new();
    if !path_a_accepted {
        notes.push(format!(
            "path_a rejected: {}",
            path_a.last_reject_detail
        ));
    }
    if !path_b_accepted {
        notes.push(format!(
            "path_b rejected: {}",
            path_b.last_reject_detail
        ));
    }
    if path_a_accepted && path_b_accepted && !parity_within(path_a.dt, path_b.dt) {
        notes.push(format!(
            "dt mismatch: path_a={} path_b={}",
            path_a.dt, path_b.dt
        ));
        chemistry_pass = false;
    }
    if !surface_mass_parity {
        notes.push(format!(
            "surface mass delta mismatch: a={} b={}",
            snap_a.delta_s, snap_b.delta_s
        ));
    }
    if advection_gap > D026_ADVECTION_ABS_TOL && surface_mass_parity {
        notes.push(format!(
            "membrane diffusion_net differs by {advection_gap} (expected: advection on path A only)"
        ));
    }

    let allowed_ok = path_a_constraint_flux_zero
        && path_b_constraint_isolated
        && path_b_phi_unchanged
        && (path_a_phi_changed || snap_a.virtual_production.abs() <= D026_PARITY_ABS_TOL);

    let gate0_pass = chemistry_pass && allowed_ok && surface_mass_parity && path_a_accepted && path_b_accepted;

    RunnerParityReport {
        path_a_accepted,
        path_b_accepted,
        path_a_reject_detail: path_a.last_reject_detail.clone(),
        path_b_reject_detail: path_b.last_reject_detail.clone(),
        path_a_dt: path_a.dt,
        path_b_dt: path_b.dt,
        gate0_pass,
        chemistry_parity_pass: chemistry_pass,
        allowed_differences_ok: allowed_ok,
        max_abs_diff,
        max_abs_diff_metric,
        diffs,
        path_a_phi_changed,
        path_b_phi_unchanged,
        path_a_constraint_flux_zero,
        path_b_constraint_isolated,
        path_b_advection_disabled,
        surface_mass_parity,
        surface_mass_abs_diff,
        notes,
    }
}

pub fn total_a_demand_from_sample(sample: &StageEObservabilitySample) -> f64 {
    sample.a_consumption_virtual_structural
        + sample.a_consumption_catalyst_reproduction
        + sample.a_consumption_precursor_production
        + sample.a_consumption_decay
}

pub fn linear_slope(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    (values.last().copied().unwrap_or(0.0) - values.first().copied().unwrap_or(0.0))
        / (values.len() - 1) as f64
}

fn rel_drop(baseline: f64, current: f64) -> f64 {
    if baseline.abs() <= f64::EPSILON {
        return 0.0;
    }
    (baseline - current) / baseline.abs()
}

fn rel_rise(baseline: f64, current: f64) -> f64 {
    if baseline.abs() <= f64::EPSILON {
        return 0.0;
    }
    (current - baseline) / baseline.abs()
}

/// Earliest checkpoint (chronological) whose metrics cross relative thresholds vs baseline.
pub fn classify_chronology_earliest(points: &[ReferenceHistoryPoint]) -> D026ChronologyLabel {
    if points.len() < 2 {
        return D026ChronologyLabel::Unknown;
    }
    let base = &points[0].sample;
    let base_theta = base.surface.mean_theta_gamma;
    let base_activation = base.a_production_activation.max(f64::EPSILON);
    let base_ret = base.a_retention;
    let base_leak = base.outward_leakage_per_interface;
    let base_struct = base.a_consumption_virtual_structural;
    let base_catalyst = base.a_consumption_catalyst_reproduction;
    let base_precursor = base.a_consumption_precursor_production;

    for point in points.iter().skip(1) {
        let s = &point.sample;
        if rel_drop(base_theta, s.surface.mean_theta_gamma) >= D026_CHRONOLOGY_REL_TOL {
            return D026ChronologyLabel::SurfaceCoverageDecline;
        }
        if rel_drop(base_activation, s.a_production_activation) >= D026_CHRONOLOGY_REL_TOL {
            return D026ChronologyLabel::ActivationCapacityDecline;
        }
        if rel_rise(base_struct, s.a_consumption_virtual_structural) >= D026_CHRONOLOGY_REL_TOL {
            return D026ChronologyLabel::StructuralDemandExcess;
        }
        if rel_rise(base_catalyst, s.a_consumption_catalyst_reproduction) >= D026_CHRONOLOGY_REL_TOL {
            return D026ChronologyLabel::CatalystDemandExcess;
        }
        if rel_rise(base_precursor, s.a_consumption_precursor_production) >= D026_CHRONOLOGY_REL_TOL {
            return D026ChronologyLabel::PrecursorDemandExcess;
        }
        if rel_rise(base_leak, s.outward_leakage_per_interface) >= D026_CHRONOLOGY_REL_TOL {
            return D026ChronologyLabel::ALeakageIncrease;
        }
        if (s.a_retention - base_ret).abs() >= D026_CHRONOLOGY_REL_TOL && point.checkpoint_step <= 25_000 {
            return D026ChronologyLabel::InitialStateDivergence;
        }
    }

    let thetas: Vec<f64> = points.iter().map(|p| p.sample.surface.mean_theta_gamma).collect();
    let mut sign_changes = 0usize;
    for w in thetas.windows(2) {
        if w[0].signum() != w[1].signum() && w[0].abs() > 1e-6 && w[1].abs() > 1e-6 {
            sign_changes += 1;
        }
    }
    if sign_changes >= 2 {
        return D026ChronologyLabel::OscillatoryOnset;
    }
    if points.len() >= 3 {
        return D026ChronologyLabel::MonotonicSlowDrift;
    }
    D026ChronologyLabel::Unknown
}

pub fn map_chronology_to_mechanism(label: D026ChronologyLabel) -> D026MechanismLabel {
    match label {
        D026ChronologyLabel::SurfaceCoverageDecline => {
            D026MechanismLabel::SurfaceCoverageMaintenanceDeficit
        }
        D026ChronologyLabel::ActivationCapacityDecline => {
            D026MechanismLabel::ActivationCapacityDeficit
        }
        D026ChronologyLabel::StructuralDemandExcess => D026MechanismLabel::StructuralADemandExcess,
        D026ChronologyLabel::CatalystDemandExcess => D026MechanismLabel::CatalystADemandExcess,
        D026ChronologyLabel::PrecursorDemandExcess => D026MechanismLabel::PrecursorADemandExcess,
        D026ChronologyLabel::ALeakageIncrease => D026MechanismLabel::ABoundaryLeakage,
        D026ChronologyLabel::OscillatoryOnset => D026MechanismLabel::CoupledOscillation,
        D026ChronologyLabel::MonotonicSlowDrift => D026MechanismLabel::TrueLongTransient,
        D026ChronologyLabel::InitialStateDivergence => D026MechanismLabel::AnalyticSeedTransient,
        D026ChronologyLabel::Unknown => D026MechanismLabel::Unknown,
    }
}

pub fn classify_mechanism_from_evidence(
    history: &ReferenceHistoryReport,
    controls: &CausalControlsReport,
) -> MechanismClassificationReport {
    let mut evidence = Vec::new();
    let chronology = history.earliest_divergence;
    let mut mechanism = map_chronology_to_mechanism(chronology);

    if !history.checkpoints_available && history.fallback_diagnostic {
        mechanism = D026MechanismLabel::AnalyticSeedTransient;
        evidence.push("D-025 checkpoints unavailable; diagnostic seed transient".into());
    }

    let base = &controls.baseline;
    let mut rescue_scores: Vec<(D026MechanismLabel, f64)> = Vec::new();
    for ctrl in &controls.controls {
        let d_ret = ctrl.a_retention_end - base.a_retention_end;
        let d_theta = ctrl.theta_gamma_end - base.theta_gamma_end;
        let d_leak = base.a_leakage_end - ctrl.a_leakage_end;
        match ctrl.label.as_str() {
            "control_a_no_a_transport" => {
                rescue_scores.push((D026MechanismLabel::ABoundaryLeakage, d_ret + d_leak));
            }
            "control_b_freeze_surface" => {
                rescue_scores.push((
                    D026MechanismLabel::SurfaceCoverageMaintenanceDeficit,
                    d_theta + d_ret,
                ));
            }
            "control_c_no_virtual_structure" => {
                rescue_scores.push((
                    D026MechanismLabel::StructuralADemandExcess,
                    d_ret + (base.total_a_demand_mean - ctrl.total_a_demand_mean),
                ));
            }
            "control_d_no_catalyst_reproduction" => {
                rescue_scores.push((
                    D026MechanismLabel::CatalystADemandExcess,
                    d_ret + (base.total_a_demand_mean - ctrl.total_a_demand_mean),
                ));
            }
            "control_e_no_precursor_synthesis" => {
                rescue_scores.push((
                    D026MechanismLabel::PrecursorADemandExcess,
                    d_ret + (base.total_a_demand_mean - ctrl.total_a_demand_mean),
                ));
            }
            _ => {}
        }
    }

    if let Some((best, score)) = rescue_scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        if score > 0.02 {
            if mechanism != best && mechanism != D026MechanismLabel::Unknown {
                mechanism = D026MechanismLabel::NoSingleDominantMechanism;
                evidence.push(format!(
                    "chronology={} disagrees with strongest control rescue={}",
                    chronology.as_str(),
                    best.as_str()
                ));
            } else {
                mechanism = best;
                evidence.push(format!("causal control rescue score={score:.4}"));
            }
        }
    }

    if mechanism == D026MechanismLabel::Unknown {
        if let Some(last) = history.points.last() {
            mechanism = classify_mechanism(&last.sample);
        }
    }

    let gate7 = matches!(
        mechanism,
        D026MechanismLabel::TrueLongTransient | D026MechanismLabel::CoupledOscillation
    );
    let gate8 = matches!(
        mechanism,
        D026MechanismLabel::StructuralADemandExcess
            | D026MechanismLabel::CatalystADemandExcess
            | D026MechanismLabel::PrecursorADemandExcess
            | D026MechanismLabel::ActivationCapacityDeficit
    );
    let suggested_rate = match mechanism {
        D026MechanismLabel::StructuralADemandExcess => Some("k_d008_structure".into()),
        D026MechanismLabel::CatalystADemandExcess => Some("k_d008_reproduction".into()),
        D026MechanismLabel::PrecursorADemandExcess => Some("k_precursor".into()),
        D026MechanismLabel::ActivationCapacityDeficit => Some("k_d008_activation".into()),
        D026MechanismLabel::ABoundaryLeakage | D026MechanismLabel::SurfaceCoverageMaintenanceDeficit => {
            None
        }
        _ => None,
    };

    evidence.push(format!("chronology={}", chronology.as_str()));
    evidence.push(format!("gate6={}", mechanism.as_str()));

    MechanismClassificationReport {
        gate6_mechanism: mechanism,
        chronology,
        evidence,
        gate7_continuation_warranted: gate7,
        gate8_rate_correction_warranted: gate8 && suggested_rate.is_some(),
        suggested_rate,
    }
}

pub fn d026_productive_param_names() -> &'static [&'static str] {
    &D025_PRODUCTIVE_NAMES
}
