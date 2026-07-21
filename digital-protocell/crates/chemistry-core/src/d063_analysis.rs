//! D-063 environmentally connected membrane invagination architecture helpers.
//! Shadow/observer only — no production carrier, morphogenesis, or free area multipliers.

use crate::config::DX;
use crate::d058_analysis::{face_measure_a_f, xi_face_req};
use crate::d061_analysis::D061_FROZEN_KT;
use crate::d062_analysis::D062_D061_EXECUTION;
use crate::grid::Grid;
use crate::surface_density::circular_phi_profile;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const D063_PROJECT_ID: &str = "D-063";
pub const D063_AGENT_MEMORY_ID: &str =
    "D-20260721-d063-environmentally-connected-membrane-invagination-architecture";
pub const D063_STARTING_COMMIT: &str = "47f2abb";
pub const D063_STARTING_TAG: &str = "D-062-structural-maintenance-decay-review";
pub const D063_D062_CONCLUSION: &str = "D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW";
pub const D063_D061_EXECUTION: &str = D062_D061_EXECUTION;
pub const D063_D059_CONCLUSION: &str = "D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN";
pub const D063_D058_CONCLUSION: &str = "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT";
pub const D063_FROZEN_KT: f64 = D061_FROZEN_KT;
pub const D063_CHI_VIABLE: f64 = 1.05;
pub const D063_A_RETENTION_TARGET: f64 = 0.80;
pub const D063_C_RETENTION_TARGET: f64 = 0.80;
pub const D063_PHI_INTERIOR: f64 = 0.5;
pub const D063_IFACE_EPS: f64 = 2.0;
pub const D063_AREA_TOL: f64 = 1e-9;
pub const D063_PA_TOL: f64 = 0.25;
pub const D063_N_MIN: f64 = 1e-3;
pub const D063_F_MIN: f64 = 1e-3;
pub const D063_THROUGHPUT_RADII: &[f64] = &[16.0, 22.0, 32.0];
pub const D063_ALPHA_TARGETS: &[f64] = &[1.0, 1.25, 1.5, 2.0, 3.0, 5.0];
pub const D063_RECORD_SMALL_SIZE_CLOSED: &str = "EXTERNAL_CARRIER_SMALL_SIZE_ROUTE_CLOSED";
pub const D063_RECORD_AREA_REVIEW: &str = "EXPLICIT_CONNECTED_MEMBRANE_AREA_REVIEW_AUTHORIZED";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D063PrimaryConclusion {
    ExternalInvaginationArchitectureJustified,
    ExteriorConnectedChannelArchitectureJustified,
    ConnectedMembraneMorphogenesisRequired,
    ConnectedMembraneBootstrapFailure,
    ConnectedMembraneAreaInsufficient,
    ConnectedChannelDepletionLimit,
    ConnectedMembraneArchitectureInconclusive,
    PriorRouteNotReproduced,
    MembraneConnectivityUnresolved,
    ConnectedAreaAccountingFailure,
    MembraneMaterialAccountingFailure,
    ConnectedCarrierParityFailure,
    ConnectedAreaDoesNotIncreaseThroughput,
    ConnectedMembraneShadowRepairFailure,
    TopologyDamageOrConnectivityFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D063PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalInvaginationArchitectureJustified => {
                "D063_EXTERNAL_INVAGINATION_ARCHITECTURE_JUSTIFIED"
            }
            Self::ExteriorConnectedChannelArchitectureJustified => {
                "D063_EXTERIOR_CONNECTED_CHANNEL_ARCHITECTURE_JUSTIFIED"
            }
            Self::ConnectedMembraneMorphogenesisRequired => {
                "D063_CONNECTED_MEMBRANE_MORPHOGENESIS_REQUIRED"
            }
            Self::ConnectedMembraneBootstrapFailure => {
                "D063_CONNECTED_MEMBRANE_BOOTSTRAP_FAILURE"
            }
            Self::ConnectedMembraneAreaInsufficient => {
                "D063_CONNECTED_MEMBRANE_AREA_INSUFFICIENT"
            }
            Self::ConnectedChannelDepletionLimit => "D063_CONNECTED_CHANNEL_DEPLETION_LIMIT",
            Self::ConnectedMembraneArchitectureInconclusive => {
                "D063_CONNECTED_MEMBRANE_ARCHITECTURE_INCONCLUSIVE"
            }
            Self::PriorRouteNotReproduced => "D063_PRIOR_ROUTE_NOT_REPRODUCED",
            Self::MembraneConnectivityUnresolved => "D063_MEMBRANE_CONNECTIVITY_UNRESOLVED",
            Self::ConnectedAreaAccountingFailure => "D063_CONNECTED_AREA_ACCOUNTING_FAILURE",
            Self::MembraneMaterialAccountingFailure => {
                "D063_MEMBRANE_MATERIAL_ACCOUNTING_FAILURE"
            }
            Self::ConnectedCarrierParityFailure => "D063_CONNECTED_CARRIER_PARITY_FAILURE",
            Self::ConnectedAreaDoesNotIncreaseThroughput => {
                "D063_CONNECTED_AREA_DOES_NOT_INCREASE_THROUGHPUT"
            }
            Self::ConnectedMembraneShadowRepairFailure => {
                "D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE"
            }
            Self::TopologyDamageOrConnectivityFailure => {
                "D063_TOPOLOGY_DAMAGE_OR_CONNECTIVITY_FAILURE"
            }
            Self::WorkspaceScopeNotIsolated => "D063_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D063_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D063_NUMERICAL_FAILURE",
            Self::Fail => "D063_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D063Route {
    A,
    B,
    P,
    M,
    T,
    C,
    I,
}

impl D063Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "Route_A_external_invagination_architecture",
            Self::B => "Route_B_exterior_connected_channel_architecture",
            Self::P => "Route_P_morphogenesis_required",
            Self::M => "Route_M_bootstrap_failure",
            Self::T => "Route_T_connected_area_insufficient",
            Self::C => "Route_C_channel_depletion_limit",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D063PrimaryConclusion {
        match self {
            Self::A => D063PrimaryConclusion::ExternalInvaginationArchitectureJustified,
            Self::B => D063PrimaryConclusion::ExteriorConnectedChannelArchitectureJustified,
            Self::P => D063PrimaryConclusion::ConnectedMembraneMorphogenesisRequired,
            Self::M => D063PrimaryConclusion::ConnectedMembraneBootstrapFailure,
            Self::T => D063PrimaryConclusion::ConnectedMembraneAreaInsufficient,
            Self::C => D063PrimaryConclusion::ConnectedChannelDepletionLimit,
            Self::I => D063PrimaryConclusion::ConnectedMembraneArchitectureInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MembraneFaceClass {
    ExternalBoundary,
    ExteriorConnectedInvagination,
    ClosedInternal,
    InvalidOrAmbiguous,
}

impl MembraneFaceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalBoundary => "EXTERNAL_BOUNDARY_MEMBRANE",
            Self::ExteriorConnectedInvagination => "EXTERIOR_CONNECTED_INVAGINATION_MEMBRANE",
            Self::ClosedInternal => "CLOSED_INTERNAL_MEMBRANE",
            Self::InvalidOrAmbiguous => "INVALID_OR_AMBIGUOUS_INTERFACE",
        }
    }

    pub const fn carrier_active(self) -> bool {
        matches!(
            self,
            Self::ExternalBoundary | Self::ExteriorConnectedInvagination
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeometryFamily {
    ASmoothExternal,
    BRadialInvaginations,
    CBranchedExteriorChannels,
    DCorrugatedOuterBoundary,
    EClosedInternalVesicles,
}

impl GeometryFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ASmoothExternal => "Family_A_smooth_external_boundary",
            Self::BRadialInvaginations => "Family_B_radial_invaginations",
            Self::CBranchedExteriorChannels => "Family_C_branched_exterior_channels",
            Self::DCorrugatedOuterBoundary => "Family_D_corrugated_outer_boundary",
            Self::EClosedInternalVesicles => "Family_E_closed_internal_vesicles",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaterialFeasibility {
    MaterialAvailableFromInitialSeed,
    MaterialBuildableFromEndogenousP,
    MaterialRequiresUnauthorizedSeed,
    MaterialBudgetInsufficient,
}

impl MaterialFeasibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MaterialAvailableFromInitialSeed => "MATERIAL_AVAILABLE_FROM_INITIAL_SEED",
            Self::MaterialBuildableFromEndogenousP => "MATERIAL_BUILDABLE_FROM_ENDOGENOUS_P",
            Self::MaterialRequiresUnauthorizedSeed => "MATERIAL_REQUIRES_UNAUTHORIZED_SEED",
            Self::MaterialBudgetInsufficient => "MATERIAL_BUDGET_INSUFFICIENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelAccessClass {
    ConnectedAreaResourceAccessible,
    ChannelDepletionLimit,
    ChannelGeometryOversealed,
    ConnectedAreaAccessInconclusive,
}

impl ChannelAccessClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConnectedAreaResourceAccessible => "CONNECTED_AREA_RESOURCE_ACCESSIBLE",
            Self::ChannelDepletionLimit => "CHANNEL_DEPLETION_LIMIT",
            Self::ChannelGeometryOversealed => "CHANNEL_GEOMETRY_OVERSEALED",
            Self::ConnectedAreaAccessInconclusive => "CONNECTED_AREA_ACCESS_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TopologyPersistenceClass {
    TopologyPersistsPassively,
    TopologyCollapses,
    TopologySealsFromExterior,
    TopologyExpandsUnbounded,
    TopologyRequiresMorphogeneticMaintenance,
}

impl TopologyPersistenceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopologyPersistsPassively => "TOPOLOGY_PERSISTS_PASSIVELY",
            Self::TopologyCollapses => "TOPOLOGY_COLLAPSES",
            Self::TopologySealsFromExterior => "TOPOLOGY_SEALS_FROM_EXTERIOR",
            Self::TopologyExpandsUnbounded => "TOPOLOGY_EXPANDS_UNBOUNDED",
            Self::TopologyRequiresMorphogeneticMaintenance => {
                "TOPOLOGY_REQUIRES_MORPHOGENETIC_MAINTENANCE"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BootstrapClass {
    ConnectedAreaBootstrapFeasible,
    ConnectedAreaBootstrapSubcritical,
    ConnectedAreaBootstrapMaterialBlocked,
    ConnectedAreaBootstrapInconclusive,
}

impl BootstrapClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConnectedAreaBootstrapFeasible => "CONNECTED_AREA_BOOTSTRAP_FEASIBLE",
            Self::ConnectedAreaBootstrapSubcritical => "CONNECTED_AREA_BOOTSTRAP_SUBCRITICAL",
            Self::ConnectedAreaBootstrapMaterialBlocked => {
                "CONNECTED_AREA_BOOTSTRAP_MATERIAL_BLOCKED"
            }
            Self::ConnectedAreaBootstrapInconclusive => "CONNECTED_AREA_BOOTSTRAP_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeometrySpec {
    pub family: GeometryFamily,
    pub radius: f64,
    pub invagination_count: usize,
    pub depth_frac: f64,
    pub width: f64,
    pub branch_count: usize,
    pub corrugation_amp: f64,
    pub corrugation_modes: usize,
    pub vesicle_count: usize,
    pub vesicle_radius: f64,
}

impl GeometrySpec {
    pub fn smooth(radius: f64) -> Self {
        Self {
            family: GeometryFamily::ASmoothExternal,
            radius,
            invagination_count: 0,
            depth_frac: 0.0,
            width: 0.0,
            branch_count: 0,
            corrugation_amp: 0.0,
            corrugation_modes: 0,
            vesicle_count: 0,
            vesicle_radius: 0.0,
        }
    }

    pub fn radial(radius: f64, count: usize, depth_frac: f64, width: f64) -> Self {
        Self {
            family: GeometryFamily::BRadialInvaginations,
            radius,
            invagination_count: count,
            depth_frac,
            width,
            branch_count: 0,
            corrugation_amp: 0.0,
            corrugation_modes: 0,
            vesicle_count: 0,
            vesicle_radius: 0.0,
        }
    }

    pub fn branched(radius: f64, count: usize, depth_frac: f64, width: f64, branches: usize) -> Self {
        Self {
            family: GeometryFamily::CBranchedExteriorChannels,
            radius,
            invagination_count: count,
            depth_frac,
            width,
            branch_count: branches,
            corrugation_amp: 0.0,
            corrugation_modes: 0,
            vesicle_count: 0,
            vesicle_radius: 0.0,
        }
    }

    pub fn corrugated(radius: f64, amp: f64, modes: usize) -> Self {
        Self {
            family: GeometryFamily::DCorrugatedOuterBoundary,
            radius,
            invagination_count: 0,
            depth_frac: 0.0,
            width: 0.0,
            branch_count: 0,
            corrugation_amp: amp,
            corrugation_modes: modes,
            vesicle_count: 0,
            vesicle_radius: 0.0,
        }
    }

    pub fn closed_vesicles(radius: f64, count: usize, vesicle_radius: f64) -> Self {
        Self {
            family: GeometryFamily::EClosedInternalVesicles,
            radius,
            invagination_count: 0,
            depth_frac: 0.0,
            width: 0.0,
            branch_count: 0,
            corrugation_amp: 0.0,
            corrugation_modes: 0,
            vesicle_count: count,
            vesicle_radius,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryAccount {
    pub family: GeometryFamily,
    pub outer_equivalent_radius: f64,
    pub total_structural_mass: f64,
    pub occupied_interior_area: f64,
    pub total_physical_interface_length: f64,
    pub external_boundary_length: f64,
    pub connected_invagination_length: f64,
    pub closed_internal_interface_length: f64,
    pub ambiguous_interface_length: f64,
    pub mature_s_mass: f64,
    pub mean_s_occupancy: f64,
    pub channel_volume: f64,
    pub active_carrier_face_count: usize,
    pub min_channel_width: f64,
    pub alpha_gamma: f64,
    pub connectivity_resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MaterialBudget063 {
    pub baseline_external_s_mass: f64,
    pub added_connected_length: f64,
    pub s_per_unit_length: f64,
    pub delta_m_s: f64,
    pub p_requirement: f64,
    pub a_cost: f64,
    pub w_generated: f64,
    pub construction_time: f64,
    pub replacement_cost: f64,
    pub candidate_s_mass: f64,
    pub feasibility: MaterialFeasibility,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteEvidence063 {
    pub workspace_isolated: bool,
    pub prior_route_reproduced: bool,
    pub connectivity_resolved: bool,
    pub area_accounting_ok: bool,
    pub material_accounting_ok: bool,
    pub carrier_parity_ok: bool,
    pub throughput_scales_with_area: bool,
    pub usable_throughput_ok: bool,
    pub channel_depletion_limit: bool,
    pub shadow_repair_ok: bool,
    pub topology_persists: bool,
    pub topology_requires_morphogenesis: bool,
    pub bootstrap_feasible: bool,
    pub bootstrap_material_blocked: bool,
    pub damage_connectivity_ok: bool,
    pub invagination_sufficient: bool,
    pub channel_required: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
}

pub fn d062_route_n_reproduced(
    primary: &str,
    p_g: f64,
    p_l: f64,
    gain_loss_ratio: f64,
    scalar_restoring: bool,
) -> bool {
    primary == D063_D062_CONCLUSION
        && (p_g - 1.18).abs() < 0.15
        && (p_l - 1.22).abs() < 0.15
        && gain_loss_ratio >= 12.0
        && gain_loss_ratio <= 17.0
        && !scalar_restoring
}

pub fn rejected_architectures_disabled(
    production_carrier: bool,
    free_area_multiplier: bool,
    v15: bool,
    closed_vesicle_import: bool,
    radius_specific_kt: bool,
) -> bool {
    !production_carrier
        && !free_area_multiplier
        && !v15
        && !closed_vesicle_import
        && !radius_specific_kt
}

/// Flood-fill extracellular (φ < threshold) cells reachable from the reservoir boundary.
pub fn exterior_connected_mask(grid: &Grid, phi: &[f64], phi_thresh: f64) -> Vec<bool> {
    let n = phi.len();
    let mut connected = vec![false; n];
    let mut q = VecDeque::new();
    for idx in 0..n {
        if grid.reservoir_mask[idx] && grid.in_dish(idx) && phi[idx] < phi_thresh {
            connected[idx] = true;
            q.push_back(idx);
        }
    }
    // Also seed any exterior dish cell adjacent to a reservoir cell that is extracellular.
    for idx in 0..n {
        if !grid.in_dish(idx) || phi[idx] >= phi_thresh || connected[idx] {
            continue;
        }
        let (i, j) = (idx % grid.width, idx / grid.width);
        for (di, dj) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
            let ni = i as isize + di;
            let nj = j as isize + dj;
            if ni < 0 || nj < 0 || ni as usize >= grid.width || nj as usize >= grid.height {
                continue;
            }
            let nidx = Grid::index(grid.width, ni as usize, nj as usize);
            if grid.reservoir_mask[nidx] {
                connected[idx] = true;
                q.push_back(idx);
                break;
            }
        }
    }
    while let Some(idx) = q.pop_front() {
        let (i, j) = (idx % grid.width, idx / grid.width);
        for (di, dj) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
            let ni = i as isize + di;
            let nj = j as isize + dj;
            if ni < 0 || nj < 0 || ni as usize >= grid.width || nj as usize >= grid.height {
                continue;
            }
            let nidx = Grid::index(grid.width, ni as usize, nj as usize);
            if !grid.in_dish(nidx) || connected[nidx] || phi[nidx] >= phi_thresh {
                continue;
            }
            connected[nidx] = true;
            q.push_back(nidx);
        }
    }
    connected
}

fn carve_radial_channel(
    grid: &Grid,
    phi: &mut [f64],
    angle: f64,
    outer_r: f64,
    depth: f64,
    width: f64,
) {
    let inner_r = (outer_r - depth).max(0.0);
    let half_w = width.max(DX) * 0.5;
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt();
            if r < inner_r || r > outer_r + D063_IFACE_EPS {
                continue;
            }
            let theta = dy.atan2(dx);
            let mut dtheta = theta - angle;
            while dtheta > std::f64::consts::PI {
                dtheta -= std::f64::consts::TAU;
            }
            while dtheta < -std::f64::consts::PI {
                dtheta += std::f64::consts::TAU;
            }
            let arc = dtheta.abs() * r.max(1e-9);
            if arc <= half_w {
                // Extracellular channel: force low φ while remaining continuous to exterior.
                phi[idx] = phi[idx].min(0.05);
            }
        }
    }
}

fn carve_branch(
    grid: &Grid,
    phi: &mut [f64],
    base_angle: f64,
    branch_angle: f64,
    outer_r: f64,
    depth: f64,
    width: f64,
) {
    let start_r = outer_r - depth * 0.55;
    let end_r = (outer_r - depth * 0.15).max(start_r);
    let half_w = width.max(DX) * 0.5;
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt();
            if r < start_r || r > end_r + width {
                continue;
            }
            let theta = dy.atan2(dx);
            let mut dtheta = theta - (base_angle + branch_angle);
            while dtheta > std::f64::consts::PI {
                dtheta -= std::f64::consts::TAU;
            }
            while dtheta < -std::f64::consts::PI {
                dtheta += std::f64::consts::TAU;
            }
            let arc = dtheta.abs() * r.max(1e-9);
            if arc <= half_w {
                phi[idx] = phi[idx].min(0.05);
            }
        }
    }
}

fn carve_closed_vesicle(grid: &Grid, phi: &mut [f64], cx: f64, cy: f64, radius: f64) {
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - cx;
            let dy = j as f64 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            // Only carve if currently interior (prevent accidental exterior connection).
            if phi[idx] >= D063_PHI_INTERIOR && r <= radius {
                phi[idx] = 0.05;
            }
        }
    }
}

/// Build an explicit diagnostic φ field. Never mutates production defaults.
pub fn generate_phi(grid: &Grid, spec: &GeometrySpec) -> Vec<f64> {
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    match spec.family {
        GeometryFamily::ASmoothExternal | GeometryFamily::EClosedInternalVesicles => {
            circular_phi_profile(grid, spec.radius, D063_IFACE_EPS, &mut phi);
        }
        GeometryFamily::DCorrugatedOuterBoundary => {
            let amp = spec.corrugation_amp.max(0.0);
            let modes = spec.corrugation_modes.max(1) as f64;
            for j in 0..grid.height {
                for i in 0..grid.width {
                    let idx = Grid::index(grid.width, i, j);
                    if !grid.in_dish(idx) {
                        phi[idx] = 0.0;
                        continue;
                    }
                    let dx = i as f64 - grid.cx;
                    let dy = j as f64 - grid.cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    let theta = dy.atan2(dx);
                    let local_r = spec.radius + amp * (modes * theta).sin();
                    phi[idx] =
                        (0.5 * (1.0 - ((r - local_r) / D063_IFACE_EPS).tanh())).clamp(0.0, 1.0);
                }
            }
        }
        GeometryFamily::BRadialInvaginations | GeometryFamily::CBranchedExteriorChannels => {
            circular_phi_profile(grid, spec.radius, D063_IFACE_EPS, &mut phi);
            let depth = (spec.depth_frac.clamp(0.0, 0.95) * spec.radius).max(0.0);
            let width = spec.width.max(DX);
            let count = spec.invagination_count.max(1);
            for k in 0..count {
                let angle = std::f64::consts::TAU * (k as f64) / (count as f64);
                carve_radial_channel(grid, &mut phi, angle, spec.radius, depth, width);
                if spec.family == GeometryFamily::CBranchedExteriorChannels {
                    for b in 0..spec.branch_count.max(1) {
                        let sign = if b % 2 == 0 { 1.0 } else { -1.0 };
                        let branch_ang = sign * (0.35 + 0.15 * (b as f64));
                        carve_branch(
                            grid,
                            &mut phi,
                            angle,
                            branch_ang,
                            spec.radius,
                            depth,
                            width * 0.85,
                        );
                    }
                }
            }
        }
    }
    if spec.family == GeometryFamily::EClosedInternalVesicles {
        let count = spec.vesicle_count.max(1);
        let vr = spec.vesicle_radius.max(1.5);
        let orbit = (spec.radius * 0.45).max(vr + 2.0);
        for k in 0..count {
            let angle = std::f64::consts::TAU * (k as f64) / (count as f64);
            let cx = grid.cx + orbit * angle.cos();
            let cy = grid.cy + orbit * angle.sin();
            carve_closed_vesicle(grid, &mut phi, cx, cy, vr);
        }
    }
    phi
}

fn neighbors4(grid: &Grid, idx: usize) -> [(usize, bool); 4] {
    let i = idx % grid.width;
    let j = idx / grid.width;
    let mut out = [(0usize, false); 4];
    let dirs = [(-1isize, 0), (1, 0), (0, -1), (0, 1)];
    for (k, (di, dj)) in dirs.iter().enumerate() {
        let ni = i as isize + di;
        let nj = j as isize + dj;
        if ni < 0 || nj < 0 || ni as usize >= grid.width || nj as usize >= grid.height {
            out[k] = (0, false);
            continue;
        }
        out[k] = (Grid::index(grid.width, ni as usize, nj as usize), true);
    }
    out
}

/// Classify a membrane face from interior/exterior indices and extracellular connectivity.
pub fn classify_membrane_face(
    exterior_idx: usize,
    connected: &[bool],
    exterior_is_extracellular: bool,
    near_smooth_radius: bool,
) -> MembraneFaceClass {
    if !exterior_is_extracellular {
        return MembraneFaceClass::InvalidOrAmbiguous;
    }
    if !connected[exterior_idx] {
        return MembraneFaceClass::ClosedInternal;
    }
    if near_smooth_radius {
        MembraneFaceClass::ExternalBoundary
    } else {
        MembraneFaceClass::ExteriorConnectedInvagination
    }
}

pub fn account_geometry(
    grid: &Grid,
    phi: &[f64],
    s: &[f64],
    baseline_smooth_length: f64,
    outer_radius: f64,
) -> GeometryAccount {
    let connected = exterior_connected_mask(grid, phi, D063_PHI_INTERIOR);
    let mut total_iface = 0.0;
    let mut external = 0.0;
    let mut invag = 0.0;
    let mut closed = 0.0;
    let mut ambiguous = 0.0;
    let mut active_faces = 0usize;
    let mut interior_area = 0.0;
    let mut structural_mass = 0.0;
    let mut s_mass = 0.0;
    let mut channel_vol = 0.0;
    let mut min_width = f64::INFINITY;
    let mut connectivity_resolved = true;
    let a_f = face_measure_a_f();

    for idx in 0..phi.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        structural_mass += phi[idx] * DX * DX;
        s_mass += s[idx].max(0.0);
        if phi[idx] >= D063_PHI_INTERIOR {
            interior_area += DX * DX;
        } else if connected[idx] {
            let r = {
                let i = idx % grid.width;
                let j = idx / grid.width;
                grid.distance_from_center(i, j)
            };
            if r < outer_radius - 0.5 {
                channel_vol += DX * DX;
            }
        }
    }

    // Face traversal: each undirected edge once (right + up).
    for idx in 0..phi.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        for &(ni, nj) in &[(i + 1, j), (i, j + 1)] {
            if ni >= grid.width || nj >= grid.height {
                continue;
            }
            let jdx = Grid::index(grid.width, ni, nj);
            if !grid.in_dish(jdx) {
                continue;
            }
            let a = phi[idx] >= D063_PHI_INTERIOR;
            let b = phi[jdx] >= D063_PHI_INTERIOR;
            if a == b {
                continue;
            }
            let (interior, exterior) = if a { (idx, jdx) } else { (jdx, idx) };
            let _ = interior;
            let exterior_extra = phi[exterior] < D063_PHI_INTERIOR;
            let r_ext = {
                let ei = exterior % grid.width;
                let ej = exterior / grid.width;
                grid.distance_from_center(ei, ej)
            };
            let near_outer = r_ext >= outer_radius - 2.5 * D063_IFACE_EPS;
            let class = classify_membrane_face(exterior, &connected, exterior_extra, near_outer);
            if class == MembraneFaceClass::InvalidOrAmbiguous {
                connectivity_resolved = false;
            }
            total_iface += a_f;
            match class {
                MembraneFaceClass::ExternalBoundary => {
                    external += a_f;
                    if s[idx].max(s[jdx]) > 1e-12 {
                        active_faces += 1;
                    }
                }
                MembraneFaceClass::ExteriorConnectedInvagination => {
                    invag += a_f;
                    if s[idx].max(s[jdx]) > 1e-12 {
                        active_faces += 1;
                    }
                    // Local channel width proxy: count consecutive extracellular cells normal to face.
                    let mut width = a_f;
                    let mut cur = exterior;
                    for _ in 0..32 {
                        let mut advanced = false;
                        for (nidx, ok) in neighbors4(grid, cur) {
                            if !ok || !grid.in_dish(nidx) {
                                continue;
                            }
                            if phi[nidx] < D063_PHI_INTERIOR && connected[nidx] && nidx != interior
                            {
                                let ri = (nidx % grid.width) as f64;
                                let rj = (nidx / grid.width) as f64;
                                let ci = (cur % grid.width) as f64;
                                let cj = (cur / grid.width) as f64;
                                let toward_center = (ri - grid.cx).hypot(rj - grid.cy)
                                    < (ci - grid.cx).hypot(cj - grid.cy);
                                if toward_center {
                                    width += a_f;
                                    cur = nidx;
                                    advanced = true;
                                    break;
                                }
                            }
                        }
                        if !advanced {
                            break;
                        }
                    }
                    min_width = min_width.min(width);
                }
                MembraneFaceClass::ClosedInternal => closed += a_f,
                MembraneFaceClass::InvalidOrAmbiguous => ambiguous += a_f,
            }
        }
    }

    if !min_width.is_finite() {
        min_width = 0.0;
    }
    let connected_len = external + invag;
    let baseline = baseline_smooth_length.max(1e-18);
    let alpha = connected_len / baseline;
    let mean_s = if total_iface > 1e-18 {
        s_mass / total_iface
    } else {
        0.0
    };
    GeometryAccount {
        family: GeometryFamily::ASmoothExternal, // caller may overwrite
        outer_equivalent_radius: (interior_area / std::f64::consts::PI).max(0.0).sqrt(),
        total_structural_mass: structural_mass,
        occupied_interior_area: interior_area,
        total_physical_interface_length: total_iface,
        external_boundary_length: external,
        connected_invagination_length: invag,
        closed_internal_interface_length: closed,
        ambiguous_interface_length: ambiguous,
        mature_s_mass: s_mass,
        mean_s_occupancy: mean_s,
        channel_volume: channel_vol,
        active_carrier_face_count: active_faces,
        min_channel_width: min_width,
        alpha_gamma: alpha,
        connectivity_resolved,
    }
}

pub fn alpha_gamma(connected_area: f64, smooth_baseline: f64) -> f64 {
    if smooth_baseline <= 1e-18 {
        return 0.0;
    }
    connected_area / smooth_baseline
}

pub fn subdivision_area_invariant(total_a: f64, part_a: f64, part_b: f64, tol: f64) -> bool {
    (total_a - (part_a + part_b)).abs() <= tol * (1.0 + total_a.abs())
}

pub fn orientation_area_invariant(a: f64, a_rotated: f64, tol: f64) -> bool {
    (a - a_rotated).abs() <= tol * (1.0 + a.abs())
}

pub fn material_budget_063(
    baseline_s: f64,
    added_connected_length: f64,
    s_per_unit_length: f64,
    free_p: f64,
    sustainable_p_rate: f64,
    a_per_p: f64,
    w_per_p: f64,
    replacement_rate: f64,
    unauthorized_seed_s: f64,
) -> MaterialBudget063 {
    let delta = added_connected_length.max(0.0) * s_per_unit_length.max(0.0);
    let candidate = baseline_s + delta;
    let need_from_p = (delta - free_p.max(0.0)).max(0.0);
    let construction_time = if need_from_p <= 1e-18 {
        0.0
    } else if sustainable_p_rate > 1e-18 {
        need_from_p / sustainable_p_rate
    } else {
        f64::INFINITY
    };
    let feasibility = if unauthorized_seed_s > 1e-12 {
        MaterialFeasibility::MaterialRequiresUnauthorizedSeed
    } else if delta <= free_p.max(0.0) + 1e-12 {
        MaterialFeasibility::MaterialAvailableFromInitialSeed
    } else if construction_time.is_finite() {
        MaterialFeasibility::MaterialBuildableFromEndogenousP
    } else {
        MaterialFeasibility::MaterialBudgetInsufficient
    };
    MaterialBudget063 {
        baseline_external_s_mass: baseline_s,
        added_connected_length,
        s_per_unit_length,
        delta_m_s: delta,
        p_requirement: delta,
        a_cost: a_per_p.max(0.0) * delta,
        w_generated: w_per_p.max(0.0) * delta,
        construction_time,
        replacement_cost: replacement_rate.max(0.0) * delta,
        candidate_s_mass: candidate,
        feasibility,
    }
}

pub fn carrier_face_selected(class: MembraneFaceClass, mature_s: f64) -> bool {
    class.carrier_active() && mature_s > 1e-12
}

pub fn shadow_xi_connected(k_t: f64, gamma: f64, drive: f64, dt: f64) -> f64 {
    xi_face_req(k_t, gamma, drive, face_measure_a_f(), dt)
}

pub fn nfw_conservation_ok(
    dn_global: f64,
    df_global: f64,
    dw_global: f64,
    tol: f64,
) -> bool {
    dn_global.abs() <= tol && df_global.abs() <= tol && dw_global.abs() <= tol
}

pub fn fit_area_throughput_exponent(areas: &[f64], fluxes: &[f64]) -> Option<f64> {
    if areas.len() != fluxes.len() || areas.len() < 2 {
        return None;
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (&a, &j) in areas.iter().zip(fluxes.iter()) {
        if a > 1e-12 && j > 1e-18 {
            xs.push(a.ln());
            ys.push(j.ln());
        }
    }
    if xs.len() < 2 {
        return None;
    }
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..xs.len() {
        num += (xs[i] - mx) * (ys[i] - my);
        den += (xs[i] - mx) * (xs[i] - mx);
    }
    if den <= 1e-18 {
        return None;
    }
    Some(num / den)
}

pub fn throughput_scales_with_area(p_a: f64) -> bool {
    p_a.is_finite() && (p_a - 1.0).abs() <= D063_PA_TOL
}

/// 1D channel depletion model: concentration decays along depth with diffusion length λ.
pub fn channel_concentration_profile(
    c_exterior: f64,
    depth: f64,
    depletion_length: f64,
    samples: usize,
) -> Vec<(f64, f64)> {
    let n = samples.max(2);
    let lam = depletion_length.max(1e-9);
    (0..n)
        .map(|i| {
            let x = depth * (i as f64) / ((n - 1) as f64);
            let c = c_exterior * (-x / lam).exp();
            (x, c)
        })
        .collect()
}

pub fn usable_connected_fraction(
    connected_length: f64,
    profile_n: &[(f64, f64)],
    profile_f: &[(f64, f64)],
    n_min: f64,
    f_min: f64,
) -> f64 {
    if connected_length <= 1e-18 || profile_n.is_empty() || profile_f.len() != profile_n.len() {
        return 0.0;
    }
    let mut usable = 0.0;
    let seg = connected_length / profile_n.len() as f64;
    for i in 0..profile_n.len() {
        if profile_n[i].1 >= n_min && profile_f[i].1 >= f_min {
            usable += seg;
        }
    }
    usable / connected_length
}

pub fn classify_channel_access(f_usable: f64, min_width: f64, oversealed_width: f64) -> ChannelAccessClass {
    if min_width > 0.0 && min_width < oversealed_width {
        return ChannelAccessClass::ChannelGeometryOversealed;
    }
    if !f_usable.is_finite() {
        return ChannelAccessClass::ConnectedAreaAccessInconclusive;
    }
    if f_usable >= 0.7 {
        ChannelAccessClass::ConnectedAreaResourceAccessible
    } else if f_usable < 0.4 {
        ChannelAccessClass::ChannelDepletionLimit
    } else {
        ChannelAccessClass::ConnectedAreaAccessInconclusive
    }
}

pub fn incremental_metabolic_return(delta_a_produced: f64, a_construction: f64, a_maintenance: f64) -> f64 {
    let den = a_construction.max(0.0) + a_maintenance.max(0.0);
    if den <= 1e-18 {
        return if delta_a_produced > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
    }
    delta_a_produced / den
}

pub fn classify_bootstrap(
    first_step_affordable: bool,
    early_return_gt_one: bool,
    cumulative_throughput_increases: bool,
    unauthorized_future_area: bool,
    material_blocked: bool,
) -> BootstrapClass {
    if unauthorized_future_area {
        return BootstrapClass::ConnectedAreaBootstrapInconclusive;
    }
    if material_blocked || !first_step_affordable {
        return BootstrapClass::ConnectedAreaBootstrapMaterialBlocked;
    }
    if early_return_gt_one && cumulative_throughput_increases {
        BootstrapClass::ConnectedAreaBootstrapFeasible
    } else if first_step_affordable && !early_return_gt_one {
        BootstrapClass::ConnectedAreaBootstrapSubcritical
    } else {
        BootstrapClass::ConnectedAreaBootstrapInconclusive
    }
}

pub fn classify_topology_persistence(
    connected_length_ratio: f64,
    sealed_from_exterior: bool,
    expanded_unbounded: bool,
    collapsed: bool,
) -> TopologyPersistenceClass {
    if expanded_unbounded {
        return TopologyPersistenceClass::TopologyExpandsUnbounded;
    }
    if sealed_from_exterior {
        return TopologyPersistenceClass::TopologySealsFromExterior;
    }
    if collapsed || connected_length_ratio < 0.5 {
        return TopologyPersistenceClass::TopologyCollapses;
    }
    if connected_length_ratio >= 0.85 {
        TopologyPersistenceClass::TopologyPersistsPassively
    } else {
        TopologyPersistenceClass::TopologyRequiresMorphogeneticMaintenance
    }
}

pub fn damage_seals_stop_import(
    was_connected: bool,
    still_connected_after_closure: bool,
    import_after: f64,
    tol: f64,
) -> bool {
    if was_connected && !still_connected_after_closure {
        return import_after.abs() <= tol;
    }
    true
}

pub fn select_route(ev: RouteEvidence063) -> (D063Route, D063PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D063Route::I,
            D063PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.prior_route_reproduced {
        return (D063Route::I, D063PrimaryConclusion::PriorRouteNotReproduced);
    }
    if !ev.connectivity_resolved {
        return (
            D063Route::I,
            D063PrimaryConclusion::MembraneConnectivityUnresolved,
        );
    }
    if !ev.area_accounting_ok {
        return (
            D063Route::I,
            D063PrimaryConclusion::ConnectedAreaAccountingFailure,
        );
    }
    if !ev.material_accounting_ok {
        return (
            D063Route::I,
            D063PrimaryConclusion::MembraneMaterialAccountingFailure,
        );
    }
    if !ev.carrier_parity_ok {
        return (
            D063Route::I,
            D063PrimaryConclusion::ConnectedCarrierParityFailure,
        );
    }
    if !ev.accounting_ok {
        return (D063Route::I, D063PrimaryConclusion::AccountingFailure);
    }
    if !ev.numerical_ok {
        return (D063Route::I, D063PrimaryConclusion::NumericalFailure);
    }
    if !ev.throughput_scales_with_area {
        return (
            D063Route::I,
            D063PrimaryConclusion::ConnectedAreaDoesNotIncreaseThroughput,
        );
    }
    if !ev.damage_connectivity_ok {
        return (
            D063Route::I,
            D063PrimaryConclusion::TopologyDamageOrConnectivityFailure,
        );
    }
    if ev.channel_depletion_limit && !ev.usable_throughput_ok {
        return (D063Route::C, D063Route::C.conclusion());
    }
    if !ev.usable_throughput_ok {
        return (D063Route::T, D063Route::T.conclusion());
    }
    if ev.bootstrap_material_blocked || !ev.bootstrap_feasible {
        return (D063Route::M, D063Route::M.conclusion());
    }
    if ev.usable_throughput_ok && !ev.shadow_repair_ok {
        return (
            D063Route::I,
            D063PrimaryConclusion::ConnectedMembraneShadowRepairFailure,
        );
    }
    if ev.usable_throughput_ok && ev.topology_requires_morphogenesis {
        return (D063Route::P, D063Route::P.conclusion());
    }
    if ev.usable_throughput_ok && ev.shadow_repair_ok {
        if ev.channel_required {
            return (D063Route::B, D063Route::B.conclusion());
        }
        if ev.invagination_sufficient {
            return (D063Route::A, D063Route::A.conclusion());
        }
        if ev.topology_persists {
            return (D063Route::A, D063Route::A.conclusion());
        }
        return (D063Route::P, D063Route::P.conclusion());
    }
    (D063Route::I, D063Route::I.conclusion())
}

/// Smooth-disk baseline interface length used for α_Γ.
pub fn smooth_baseline_length(radius: f64) -> f64 {
    std::f64::consts::TAU * radius.max(1e-9)
}

/// Seed unit mature-S occupancy along all membrane faces (diagnostic seed; material accounted separately).
pub fn seed_mature_s_on_interfaces(grid: &Grid, phi: &[f64], s_per_length: f64) -> Vec<f64> {
    let mut s = vec![0.0; phi.len()];
    let a_f = face_measure_a_f();
    for idx in 0..phi.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let i = idx % grid.width;
        let j = idx / grid.width;
        for &(ni, nj) in &[(i + 1, j), (i, j + 1)] {
            if ni >= grid.width || nj >= grid.height {
                continue;
            }
            let jdx = Grid::index(grid.width, ni, nj);
            if !grid.in_dish(jdx) {
                continue;
            }
            let a = phi[idx] >= D063_PHI_INTERIOR;
            let b = phi[jdx] >= D063_PHI_INTERIOR;
            if a == b {
                continue;
            }
            let share = 0.5 * s_per_length.max(0.0) * a_f;
            s[idx] += share;
            s[jdx] += share;
        }
    }
    s
}

pub fn predicted_chi(import: f64, demand: f64) -> f64 {
    if demand <= 1e-18 {
        return if import >= 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
    }
    import / demand
}

pub fn shadow_isolation_ok(production_carrier: bool, v15: bool) -> bool {
    !production_carrier && !v15
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn closed_vesicle_not_connected() {
        let grid = Grid::new();
        let spec = GeometrySpec::closed_vesicles(20.0, 2, 3.0);
        let phi = generate_phi(&grid, &spec);
        let connected = exterior_connected_mask(&grid, &phi, D063_PHI_INTERIOR);
        // Vesicle centers should be extracellular but not exterior-connected.
        let orbit = 20.0 * 0.45;
        let cx = (grid.cx + orbit) as usize;
        let cy = grid.cy as usize;
        let idx = Grid::index(grid.width, cx.min(grid.width - 1), cy.min(grid.height - 1));
        assert!(phi[idx] < D063_PHI_INTERIOR);
        assert!(!connected[idx]);
    }

    #[test]
    fn invagination_increases_connected_area() {
        let grid = Grid::new();
        let smooth = GeometrySpec::smooth(20.0);
        let radial = GeometrySpec::radial(20.0, 8, 0.45, 2.5);
        let phi_s = generate_phi(&grid, &smooth);
        let phi_r = generate_phi(&grid, &radial);
        let s_s = seed_mature_s_on_interfaces(&grid, &phi_s, 1.0);
        let s_r = seed_mature_s_on_interfaces(&grid, &phi_r, 1.0);
        let base = smooth_baseline_length(20.0);
        let a_s = account_geometry(&grid, &phi_s, &s_s, base, 20.0);
        let a_r = account_geometry(&grid, &phi_r, &s_r, base, 20.0);
        assert!(a_r.alpha_gamma > a_s.alpha_gamma + 0.05);
        assert!(a_r.connected_invagination_length > 0.0);
        assert!(a_s.closed_internal_interface_length < 1e-9);
    }
}
