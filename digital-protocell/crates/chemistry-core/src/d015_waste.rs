//! D-015 exact accepted-step waste budget, spatial partition, and sink-capacity observers.

use crate::accounting::FieldStepLedger;
use crate::activated_metabolism::ActivatedMetabolismStepAccounting;
use crate::candidate_identity::{candidate_hash, sha256_hex, GridConfiguration};
use crate::config::{SimParams, CONC_SAFETY_LIMIT, DISH_RADIUS, RESERVOIR_WIDTH};
use crate::constraint_accounting::StructureConstraintStep;
use crate::grid::Grid;
use crate::membrane_accounting::{MembraneStepAccounting, SpeciesTransportAccounting};
use crate::reactions::interface_weight;
use crate::reservoir::waste_sink_cell;
use serde::{Deserialize, Serialize};

pub const WASTE_BUDGET_REL_TOL: f64 = 1e-8;
pub const WASTE_SPATIAL_INTERFACE_THRESHOLD: f64 = 0.25;
/// ponytail: fixed near-exterior band beyond prescribed R; upgrade path is configurable buffer.
pub const NEAR_EXTERIOR_BUFFER: f64 = 8.0;
pub const D015_ENVIRONMENT_SCHEMA_VERSION: u32 = 2;
pub const D015_PREFLIGHT_ACCEPTED_SUBSTEPS: u64 = 25_000;
pub const D015_PREFLIGHT_CHECKPOINTS: [u64; 2] = [10_000, 25_000];

/// Sentinel values excluded from organism frozen hash (environment varies independently).
const ORGANISM_HASH_ENV_SENTINEL: f64 = -999.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WasteSpatialRegion {
    Interior,
    Interface,
    NearExterior,
    BulkExterior,
    ReservoirRegion,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct DecomposedWasteSources {
    pub activation: f64,
    pub catalyst_turnover: f64,
    pub structure_turnover: f64,
    pub membrane_turnover: f64,
    pub activated_resource_turnover: f64,
    pub membrane_detachment: f64,
    pub productive_yield_waste: f64,
}

impl DecomposedWasteSources {
    pub fn sum(&self) -> f64 {
        self.activation
            + self.catalyst_turnover
            + self.structure_turnover
            + self.membrane_turnover
            + self.activated_resource_turnover
            + self.membrane_detachment
            + self.productive_yield_waste
    }

    pub fn source_fields(&self) -> [(&'static str, f64); 7] {
        [
            ("activation", self.activation),
            ("catalyst_turnover", self.catalyst_turnover),
            ("structure_turnover", self.structure_turnover),
            ("membrane_turnover", self.membrane_turnover),
            ("activated_resource_turnover", self.activated_resource_turnover),
            ("membrane_detachment", self.membrane_detachment),
            ("productive_yield_waste", self.productive_yield_waste),
        ]
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct V2WasteSourceExtents {
    pub activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
    pub structure_production_extent: f64,
    pub structure_decay: f64,
    pub membrane_synthesis: f64,
    pub membrane_decay: f64,
    pub membrane_detachment: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct WasteBudgetStep {
    pub mass_before: f64,
    pub mass_after: f64,
    pub observed_change: f64,
    pub activation: f64,
    pub catalyst_turnover: f64,
    pub structure_turnover: f64,
    pub membrane_turnover: f64,
    pub activated_resource_turnover: f64,
    pub membrane_detachment: f64,
    pub productive_yield_waste: f64,
    pub external_reservoir_input: f64,
    pub waste_clearance: f64,
    pub transport_delta: f64,
    pub numerical_correction: f64,
    pub residual: f64,
    pub relative_residual: f64,
    pub max_waste_value: f64,
    pub max_waste_index: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WasteSpatialMasks {
    pub interior: Vec<bool>,
    pub interface: Vec<bool>,
    pub near_exterior: Vec<bool>,
    pub bulk_exterior: Vec<bool>,
    pub reservoir_region: Vec<bool>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct WasteSpatialMetrics {
    pub mass_interior: f64,
    pub mass_interface: f64,
    pub mass_near_exterior: f64,
    pub mass_bulk_exterior: f64,
    pub mass_reservoir_region: f64,
    pub max_waste_value: f64,
    pub max_waste_index: usize,
    pub max_waste_region: Option<WasteSpatialRegion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClearanceImplementationClass {
    Correct,
    SignError,
    DtScalingError,
    MaskError,
    LedgerError,
    CapacityError,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SinkCapacityClass {
    ClearanceCapacityExceedsProduction,
    ClearanceCapacityBelowProduction,
    TransportToSinkLimited,
    NoFiniteEquilibrium,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct SinkCapacityAnalysis {
    pub production_rate: f64,
    pub delivery_rate_to_reservoir: f64,
    pub clearance_rate_at_current_w: f64,
    pub max_clearance_rate_below_ceiling: f64,
    pub predicted_equilibrium_w: f64,
    pub clearance_margin: f64,
    pub reservoir_cell_count: usize,
    pub classification: Option<SinkCapacityClass>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WasteBudgetState {
    pub last_step: WasteBudgetStep,
    pub cumulative_sources: DecomposedWasteSources,
    pub cumulative_clearance: f64,
    pub cumulative_external_input: f64,
    pub cumulative_transport: f64,
    pub cumulative_numerical_correction: f64,
    pub cumulative_observed_change: f64,
    pub cumulative_residual: f64,
    pub accepted_steps: u64,
    pub max_step_relative_residual: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct D015PreflightRequirements {
    pub waste_budget_required: bool,
    pub spatial_partition_required: bool,
    pub sink_capacity_required: bool,
    pub checkpoints: Vec<u64>,
    pub accepted_substeps: u64,
}

pub fn decompose_v2_waste_sources(extents: &V2WasteSourceExtents, params: &SimParams) -> DecomposedWasteSources {
    let productive_yield_waste = (1.0 - params.eta_c) * extents.reproduction
        + (1.0 - params.eta_phi) * extents.structure_production_extent
        + (1.0 - params.eta_m) * extents.membrane_synthesis;
    DecomposedWasteSources {
        activation: extents.activation,
        catalyst_turnover: extents.catalyst_turnover,
        structure_turnover: extents.structure_decay,
        membrane_turnover: extents.membrane_decay,
        activated_resource_turnover: extents.activated_decay,
        membrane_detachment: extents.membrane_detachment,
        productive_yield_waste,
    }
}

pub fn v2_waste_source_extents(
    metabolism: &ActivatedMetabolismStepAccounting,
    membrane: &MembraneStepAccounting,
    constraint: &StructureConstraintStep,
    params: &SimParams,
) -> V2WasteSourceExtents {
    let structure_production_extent = if params.eta_phi > 0.0 {
        constraint.virtual_production / params.eta_phi
    } else {
        0.0
    };
    V2WasteSourceExtents {
        activation: metabolism.activation,
        reproduction: metabolism.reproduction,
        activated_decay: metabolism.activated_decay,
        catalyst_turnover: metabolism.catalyst_turnover,
        structure_production_extent,
        structure_decay: constraint.virtual_decay,
        membrane_synthesis: membrane.synthesis,
        membrane_decay: membrane.decay,
        membrane_detachment: membrane.detachment,
    }
}

pub fn build_waste_budget_step(
    waste: &FieldStepLedger,
    sources: &DecomposedWasteSources,
) -> WasteBudgetStep {
    let observed_change = waste.mass_after - waste.mass_before;
    let external_reservoir_input = waste.reservoir_delta.max(0.0);
    let waste_clearance = (-waste.reservoir_delta).max(0.0);
    let transport_delta = waste.diffusion_delta;
    let numerical_correction = waste.numerical_correction_delta;
    let source_sum = sources.sum();
    let budgeted_change =
        source_sum + external_reservoir_input - waste_clearance + numerical_correction;
    let residual = observed_change - transport_delta - budgeted_change;
    let scale = waste.mass_before.abs().max(waste.mass_after.abs()).max(1.0);
    WasteBudgetStep {
        mass_before: waste.mass_before,
        mass_after: waste.mass_after,
        observed_change,
        activation: sources.activation,
        catalyst_turnover: sources.catalyst_turnover,
        structure_turnover: sources.structure_turnover,
        membrane_turnover: sources.membrane_turnover,
        activated_resource_turnover: sources.activated_resource_turnover,
        membrane_detachment: sources.membrane_detachment,
        productive_yield_waste: sources.productive_yield_waste,
        external_reservoir_input,
        waste_clearance,
        transport_delta,
        numerical_correction,
        residual,
        relative_residual: residual.abs() / scale,
        max_waste_value: 0.0,
        max_waste_index: 0,
    }
}

pub fn waste_budget_step_closes(step: &WasteBudgetStep) -> bool {
    step.relative_residual <= WASTE_BUDGET_REL_TOL
}

pub fn attach_waste_max_location(step: &mut WasteBudgetStep, grid: &Grid, waste: &[f64]) {
    let (idx, value) = max_waste_location(grid, waste);
    step.max_waste_index = idx;
    step.max_waste_value = value;
}

impl WasteBudgetState {
    pub fn record_accepted(&mut self, step: WasteBudgetStep, sources: DecomposedWasteSources) {
        self.cumulative_sources.activation += sources.activation;
        self.cumulative_sources.catalyst_turnover += sources.catalyst_turnover;
        self.cumulative_sources.structure_turnover += sources.structure_turnover;
        self.cumulative_sources.membrane_turnover += sources.membrane_turnover;
        self.cumulative_sources.activated_resource_turnover += sources.activated_resource_turnover;
        self.cumulative_sources.membrane_detachment += sources.membrane_detachment;
        self.cumulative_sources.productive_yield_waste += sources.productive_yield_waste;
        self.cumulative_clearance += step.waste_clearance;
        self.cumulative_external_input += step.external_reservoir_input;
        self.cumulative_transport += step.transport_delta;
        self.cumulative_numerical_correction += step.numerical_correction;
        self.cumulative_observed_change += step.observed_change;
        self.cumulative_residual += step.residual.abs();
        self.max_step_relative_residual = self.max_step_relative_residual.max(step.relative_residual);
        self.last_step = step;
        self.accepted_steps += 1;
    }

    pub fn global_transport_residual(&self) -> f64 {
        self.cumulative_transport
    }
}

pub fn build_waste_spatial_masks(grid: &Grid, phi: &[f64], prescribed_radius: f64) -> WasteSpatialMasks {
    let n = grid.width * grid.height;
    let mut masks = WasteSpatialMasks {
        interior: vec![false; n],
        interface: vec![false; n],
        near_exterior: vec![false; n],
        bulk_exterior: vec![false; n],
        reservoir_region: grid.reservoir_mask.clone(),
    };
    let near_outer = prescribed_radius + NEAR_EXTERIOR_BUFFER;
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        if grid.reservoir_mask[idx] {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        let r = grid.distance_from_center(i, j);
        let iface = interface_weight(phi[idx]) >= WASTE_SPATIAL_INTERFACE_THRESHOLD;
        if iface {
            masks.interface[idx] = true;
        } else if phi[idx] >= 0.5 {
            masks.interior[idx] = true;
        } else if r <= near_outer {
            masks.near_exterior[idx] = true;
        } else {
            masks.bulk_exterior[idx] = true;
        }
    }
    masks
}

pub fn waste_spatial_partition(
    grid: &Grid,
    waste: &[f64],
    masks: &WasteSpatialMasks,
) -> WasteSpatialMetrics {
    let mut metrics = WasteSpatialMetrics::default();
    for idx in 0..waste.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let w = waste[idx];
        if masks.interior[idx] {
            metrics.mass_interior += w;
        } else if masks.interface[idx] {
            metrics.mass_interface += w;
        } else if masks.near_exterior[idx] {
            metrics.mass_near_exterior += w;
        } else if masks.bulk_exterior[idx] {
            metrics.mass_bulk_exterior += w;
        } else if masks.reservoir_region[idx] {
            metrics.mass_reservoir_region += w;
        }
        if w > metrics.max_waste_value {
            metrics.max_waste_value = w;
            metrics.max_waste_index = idx;
            metrics.max_waste_region = region_of_index(masks, idx);
        }
    }
    metrics
}

fn region_of_index(masks: &WasteSpatialMasks, idx: usize) -> Option<WasteSpatialRegion> {
    if masks.interior[idx] {
        Some(WasteSpatialRegion::Interior)
    } else if masks.interface[idx] {
        Some(WasteSpatialRegion::Interface)
    } else if masks.near_exterior[idx] {
        Some(WasteSpatialRegion::NearExterior)
    } else if masks.bulk_exterior[idx] {
        Some(WasteSpatialRegion::BulkExterior)
    } else if masks.reservoir_region[idx] {
        Some(WasteSpatialRegion::ReservoirRegion)
    } else {
        None
    }
}

pub fn masks_cover_dish_once(grid: &Grid, masks: &WasteSpatialMasks) -> bool {
    for idx in 0..grid.width * grid.height {
        if !grid.in_dish(idx) {
            continue;
        }
        let count = masks.interior[idx] as u8
            + masks.interface[idx] as u8
            + masks.near_exterior[idx] as u8
            + masks.bulk_exterior[idx] as u8
            + masks.reservoir_region[idx] as u8;
        if count != 1 {
            return false;
        }
    }
    true
}

pub fn max_waste_location(grid: &Grid, waste: &[f64]) -> (usize, f64) {
    let mut best_idx = 0usize;
    let mut best_val = 0.0;
    for idx in 0..waste.len() {
        if grid.in_dish(idx) && waste[idx] > best_val {
            best_val = waste[idx];
            best_idx = idx;
        }
    }
    (best_idx, best_val)
}

/// Per-cell linear clearance increment matching `reservoir::apply_reservoir`.
pub fn predicted_cell_clearance_delta(w: f64, w_target: f64, reservoir_rate: f64, dt: f64) -> f64 {
    reservoir_rate * dt * (w_target - w)
}

pub fn total_reservoir_waste_delta(
    grid: &Grid,
    waste: &[f64],
    params: &SimParams,
    dt: f64,
) -> f64 {
    let rate = params.reservoir_rate * dt;
    let mut delta = 0.0;
    for idx in 0..waste.len() {
        if waste_sink_cell(grid, idx, params) {
            delta += rate * (params.w_reservoir - waste[idx]);
        }
    }
    delta
}

pub fn waste_sink_cell_count(grid: &Grid, params: &SimParams) -> usize {
    (0..grid.width * grid.height)
        .filter(|&idx| waste_sink_cell(grid, idx, params))
        .count()
}

pub fn apply_reservoir_waste_delta(
    grid: &Grid,
    waste: &mut [f64],
    params: &SimParams,
    dt: f64,
) -> f64 {
    let before = waste_sink_mass(grid, waste, params);
    let rate = params.reservoir_rate * dt;
    for idx in 0..waste.len() {
        if waste_sink_cell(grid, idx, params) {
            waste[idx] += rate * (params.w_reservoir - waste[idx]);
        }
    }
    waste_sink_mass(grid, waste, params) - before
}

fn waste_sink_mass(grid: &Grid, field: &[f64], params: &SimParams) -> f64 {
    field
        .iter()
        .enumerate()
        .filter(|(idx, _)| waste_sink_cell(grid, *idx, params))
        .map(|(_, &v)| v)
        .sum()
}

pub fn classify_clearance_implementation(
    predicted_delta: f64,
    ledger_delta: f64,
    dt: f64,
    used_old_state: bool,
    mask_matches_w_sink: bool,
) -> ClearanceImplementationClass {
    if !mask_matches_w_sink {
        return ClearanceImplementationClass::MaskError;
    }
    if !used_old_state {
        return ClearanceImplementationClass::LedgerError;
    }
    if dt <= 0.0 {
        return ClearanceImplementationClass::DtScalingError;
    }
    if predicted_delta.abs() > 1e-15 && ledger_delta.signum() != predicted_delta.signum() {
        return ClearanceImplementationClass::SignError;
    }
    if (predicted_delta - ledger_delta).abs() > 1e-10 * predicted_delta.abs().max(ledger_delta.abs()).max(1.0) {
        return ClearanceImplementationClass::LedgerError;
    }
    ClearanceImplementationClass::Correct
}

pub fn linear_sink_clearance_rate(
    grid: &Grid,
    waste: &[f64],
    params: &SimParams,
) -> f64 {
    let k = params.reservoir_rate;
    let target = params.w_reservoir;
    let mut rate = 0.0;
    for idx in 0..waste.len() {
        if waste_sink_cell(grid, idx, params) {
            rate += k * (target - waste[idx]);
        }
    }
    rate
}

pub fn max_linear_clearance_rate_below_ceiling(
    grid: &Grid,
    params: &SimParams,
    ceiling: f64,
) -> f64 {
    let k = params.reservoir_rate;
    let target = params.w_reservoir;
    let mut rate = 0.0;
    for idx in 0..grid.width * grid.height {
        if waste_sink_cell(grid, idx, params) {
            rate += k * (target - ceiling);
        }
    }
    rate
}

pub fn predicted_equilibrium_w(production_rate: f64, params: &SimParams, reservoir_cells: usize) -> f64 {
    if params.reservoir_rate <= 0.0 || reservoir_cells == 0 {
        return f64::INFINITY;
    }
    let target = params.w_reservoir;
    let clearance_at_target = params.reservoir_rate * reservoir_cells as f64 * (target - target);
    let _ = clearance_at_target;
    // dW/dt = k*(N*target - sum(W)) for uniform approximation; at equilibrium sum(W)=N*W_eq
    // production = k*N*(target - W_eq) => W_eq = target - P/(k*N)
    target - production_rate / (params.reservoir_rate * reservoir_cells as f64)
}

pub fn analyze_sink_capacity(
    grid: &Grid,
    waste: &[f64],
    params: &SimParams,
    production_rate: f64,
    delivery_rate_to_reservoir: f64,
) -> SinkCapacityAnalysis {
    let reservoir_cells = waste_sink_cell_count(grid, params);
    let clearance_rate = linear_sink_clearance_rate(grid, waste, params);
    let max_clearance = max_linear_clearance_rate_below_ceiling(grid, params, CONC_SAFETY_LIMIT);
    let predicted_eq = predicted_equilibrium_w(production_rate, params, reservoir_cells);
    let clearance_margin = if production_rate.abs() > 1e-15 {
        clearance_rate / production_rate
    } else {
        f64::INFINITY
    };
    let classification = classify_sink_capacity(
        production_rate,
        delivery_rate_to_reservoir,
        clearance_rate,
        predicted_eq,
    );
    SinkCapacityAnalysis {
        production_rate,
        delivery_rate_to_reservoir,
        clearance_rate_at_current_w: clearance_rate,
        max_clearance_rate_below_ceiling: max_clearance,
        predicted_equilibrium_w: predicted_eq,
        clearance_margin,
        reservoir_cell_count: reservoir_cells,
        classification: Some(classification),
    }
}

pub fn classify_sink_capacity(
    production_rate: f64,
    delivery_rate_to_reservoir: f64,
    clearance_rate: f64,
    predicted_equilibrium_w: f64,
) -> SinkCapacityClass {
    if !predicted_equilibrium_w.is_finite() {
        return SinkCapacityClass::NoFiniteEquilibrium;
    }
    if delivery_rate_to_reservoir + 1e-12 < production_rate {
        return SinkCapacityClass::TransportToSinkLimited;
    }
    let removal = clearance_rate.abs();
    if removal + 1e-12 >= production_rate {
        SinkCapacityClass::ClearanceCapacityExceedsProduction
    } else if predicted_equilibrium_w > CONC_SAFETY_LIMIT {
        SinkCapacityClass::NoFiniteEquilibrium
    } else {
        SinkCapacityClass::ClearanceCapacityBelowProduction
    }
}

pub fn finite_domain_capacity(grid: &Grid, ceiling: f64) -> f64 {
    let mut capacity = 0.0;
    for idx in 0..grid.width * grid.height {
        if grid.in_dish(idx) {
            capacity += ceiling;
        }
    }
    capacity
}

pub fn ceiling_raise_allowed(predicted_equilibrium_w: f64, ceiling: f64) -> bool {
    predicted_equilibrium_w.is_finite()
        && predicted_equilibrium_w > ceiling
        && predicted_equilibrium_w < f64::INFINITY
}

pub fn environment_configuration_hash(params: &SimParams) -> String {
    let s = format!(
        "env:v{D015_ENVIRONMENT_SCHEMA_VERSION};n_reservoir={};f_reservoir={};w_reservoir={};reservoir_rate={};waste_sink_inner_radius={}",
        params.n_reservoir,
        params.f_reservoir,
        params.w_reservoir,
        params.reservoir_rate,
        params.waste_sink_inner_radius
    );
    sha256_hex(s.as_bytes())
}

/// Smallest W-sink radius placing clearance in the near-exterior band around the cell.
pub fn d015_repaired_waste_sink_inner_radius(prescribed_r: f64) -> f64 {
    prescribed_r + NEAR_EXTERIOR_BUFFER
}

pub fn apply_d015_repaired_environment(params: &mut SimParams, prescribed_r: f64) {
    params.waste_sink_inner_radius = d015_repaired_waste_sink_inner_radius(prescribed_r);
}

pub fn organism_frozen_hash(params: &SimParams, grid: &GridConfiguration) -> String {
    let mut normalized = params.clone();
    normalized.n_reservoir = ORGANISM_HASH_ENV_SENTINEL;
    normalized.f_reservoir = ORGANISM_HASH_ENV_SENTINEL;
    normalized.w_reservoir = ORGANISM_HASH_ENV_SENTINEL;
    normalized.reservoir_rate = ORGANISM_HASH_ENV_SENTINEL;
    normalized.waste_sink_inner_radius = ORGANISM_HASH_ENV_SENTINEL;
    candidate_hash(&normalized, grid)
}

pub fn d015_preflight_requirements() -> D015PreflightRequirements {
    D015PreflightRequirements {
        waste_budget_required: true,
        spatial_partition_required: true,
        sink_capacity_required: true,
        checkpoints: D015_PREFLIGHT_CHECKPOINTS.to_vec(),
        accepted_substeps: D015_PREFLIGHT_ACCEPTED_SUBSTEPS,
    }
}

pub fn d015_preflight_requires_waste_budget(req: &D015PreflightRequirements) -> bool {
    req.waste_budget_required
        && req.spatial_partition_required
        && req.sink_capacity_required
        && req.checkpoints.contains(&10_000)
        && req.checkpoints.contains(&25_000)
        && req.accepted_substeps >= 25_000
}

pub fn solver_remains_closed_without_quasi_steady(
    artifact_valid: bool,
    quasi_steady: bool,
    waste_budget_closes: bool,
) -> bool {
    !(artifact_valid && quasi_steady && waste_budget_closes)
}

pub fn global_waste_transport_cancels(steps: &[WasteBudgetStep]) -> bool {
    steps.iter().map(|s| s.transport_delta).sum::<f64>().abs() <= WASTE_BUDGET_REL_TOL
}

pub fn internal_waste_transport_cancels(transport: &SpeciesTransportAccounting, dt: f64) -> bool {
    transport.net_change_rate.abs() * dt <= WASTE_BUDGET_REL_TOL
}

/// Diagnostic-only membrane bypass: zero waste permeability without mutating stored params.
pub fn diagnostic_membrane_bypass_waste(params: &SimParams) -> SimParams {
    let mut p = params.clone();
    p.beta_w = 0.0;
    p
}

pub fn is_diagnostic_membrane_bypass(params: &SimParams, baseline_beta_w: f64) -> bool {
    params.beta_w == 0.0 && baseline_beta_w != 0.0
}

pub fn reservoir_geometry_summary(grid: &Grid) -> (f64, f64, usize) {
    (
        DISH_RADIUS,
        RESERVOIR_WIDTH,
        grid.reservoir_mask.iter().filter(|&&m| m).count(),
    )
}
