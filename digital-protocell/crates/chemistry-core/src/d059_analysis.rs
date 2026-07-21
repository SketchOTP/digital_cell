//! D-059 viable-size basin and membrane-area architecture review helpers.
//! Observer / shadow diagnostic only: no production biology, no V15, no size controller.

use crate::d057_analysis::scaling_exponent;
use crate::d058_analysis::{
    capacity_contrib, corrected_k_t_star, face_measure_a_f, xi_face_req,
};
use serde::{Deserialize, Serialize};

pub const D059_PROJECT_ID: &str = "D-059";
pub const D059_AGENT_MEMORY_ID: &str =
    "D-20260721-d059-viable-size-basin-membrane-area-review";
pub const D059_STARTING_COMMIT: &str = "482882d";
pub const D059_STARTING_TAG: &str = "D-058-corrected-carrier-normalization-audit";
pub const D059_D056_COMMIT: &str = "ed6de2c";
pub const D059_D056_TAG: &str = "D-056-waste-coupled-resource-carrier-fail";
pub const D059_D057_COMMIT: &str = "1c9d6ae";
pub const D059_D057_TAG: &str = "D-057-carrier-geometry-driving-force-audit";
pub const D059_D058_CONCLUSION: &str = "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT";
pub const D059_PRESERVATION_RECORD: &str =
    "EXTERNAL_MEMBRANE_CARRIER_SURFACE_CAPACITY_LIMIT_CONFIRMED";
pub const D059_CHI_VIABLE: f64 = 1.05;
pub const D059_A_RETENTION_TARGET: f64 = 0.80;
pub const D059_RATE_SPAN_FAIL: f64 = 3.0;
pub const D059_CONTIGUOUS_RADII_MIN: usize = 3;
pub const D059_MAX_GLOBAL_KT: usize = 5;
pub const D059_MAX_AREA_CANDIDATES: usize = 3;

/// Governed radii for matched-state / viable-radius campaigns.
pub const D059_MATCHED_RADII: &[f64] = &[
    6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 28.0, 32.0,
];
pub const D059_VIABLE_SEARCH_RADII: &[f64] =
    &[6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D059PrimaryConclusion {
    ExternalCarrierRestoringSizeBasin,
    ExternalCarrierSizeLimitNoRestoringBasin,
    InternalMembraneAreaArchitectureJustified,
    InternalMembraneAreaBootstrapFailure,
    WasteCoupledCarrierSurfaceArchitectureRejected,
    SizeAndMembraneAreaReviewInconclusive,
    NoViableExternalCarrierRadius,
    InternalMembraneAreaArchitectureNotJustified,
    D058RouteVNotReproduced,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D059PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalCarrierRestoringSizeBasin => {
                "D059_EXTERNAL_CARRIER_RESTORING_SIZE_BASIN"
            }
            Self::ExternalCarrierSizeLimitNoRestoringBasin => {
                "D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN"
            }
            Self::InternalMembraneAreaArchitectureJustified => {
                "D059_INTERNAL_MEMBRANE_AREA_ARCHITECTURE_JUSTIFIED"
            }
            Self::InternalMembraneAreaBootstrapFailure => {
                "D059_INTERNAL_MEMBRANE_AREA_BOOTSTRAP_FAILURE"
            }
            Self::WasteCoupledCarrierSurfaceArchitectureRejected => {
                "D059_WASTE_COUPLED_CARRIER_SURFACE_ARCHITECTURE_REJECTED"
            }
            Self::SizeAndMembraneAreaReviewInconclusive => {
                "D059_SIZE_AND_MEMBRANE_AREA_REVIEW_INCONCLUSIVE"
            }
            Self::NoViableExternalCarrierRadius => "D059_NO_VIABLE_EXTERNAL_CARRIER_RADIUS",
            Self::InternalMembraneAreaArchitectureNotJustified => {
                "D059_INTERNAL_MEMBRANE_AREA_ARCHITECTURE_NOT_JUSTIFIED"
            }
            Self::D058RouteVNotReproduced => "D059_D058_ROUTE_V_NOT_REPRODUCED",
            Self::WorkspaceScopeNotIsolated => "D059_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D059_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D059_NUMERICAL_FAILURE",
            Self::Fail => "D059_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D059Route {
    S,
    L,
    M,
    B,
    R,
    I,
}

impl D059Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::S => "Route_S_external_carrier_restoring_size_basin",
            Self::L => "Route_L_external_carrier_size_limit_no_restoring_basin",
            Self::M => "Route_M_internal_membrane_area_justified",
            Self::B => "Route_B_internal_area_bootstrap_failure",
            Self::R => "Route_R_waste_coupled_carrier_surface_rejected",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D059PrimaryConclusion {
        match self {
            Self::S => D059PrimaryConclusion::ExternalCarrierRestoringSizeBasin,
            Self::L => D059PrimaryConclusion::ExternalCarrierSizeLimitNoRestoringBasin,
            Self::M => D059PrimaryConclusion::InternalMembraneAreaArchitectureJustified,
            Self::B => D059PrimaryConclusion::InternalMembraneAreaBootstrapFailure,
            Self::R => D059PrimaryConclusion::WasteCoupledCarrierSurfaceArchitectureRejected,
            Self::I => D059PrimaryConclusion::SizeAndMembraneAreaReviewInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchedScalingClass {
    MatchedSurfaceVolumeLimit,
    CoupledStateScalingAmplification,
    D058RadiusExponentConfounded,
    RadiusScalingInconclusive,
}

impl MatchedScalingClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MatchedSurfaceVolumeLimit => "MATCHED_SURFACE_VOLUME_LIMIT",
            Self::CoupledStateScalingAmplification => "COUPLED_STATE_SCALING_AMPLIFICATION",
            Self::D058RadiusExponentConfounded => "D058_RADIUS_EXPONENT_CONFOUNDED",
            Self::RadiusScalingInconclusive => "RADIUS_SCALING_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FrontierRegion {
    InsufficientImport,
    ViableThroughput,
    WLimited,
    Overtransport,
    ReverseFluxUnstable,
    NumericallyInvalid,
}

impl FrontierRegion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InsufficientImport => "insufficient_import",
            Self::ViableThroughput => "viable_throughput",
            Self::WLimited => "W_limited",
            Self::Overtransport => "overtransport",
            Self::ReverseFluxUnstable => "reverse_flux_unstable",
            Self::NumericallyInvalid => "numerically_invalid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RestoringSizeClass {
    RestoringSizeBasin,
    OneSidedSizeLimit,
    NeutralSizeManifold,
    RunawayGrowth,
    RunawayCollapse,
    NoRestoringSizeDynamics,
}

impl RestoringSizeClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RestoringSizeBasin => "RESTORING_SIZE_BASIN",
            Self::OneSidedSizeLimit => "ONE_SIDED_SIZE_LIMIT",
            Self::NeutralSizeManifold => "NEUTRAL_SIZE_MANIFOLD",
            Self::RunawayGrowth => "RUNAWAY_GROWTH",
            Self::RunawayCollapse => "RUNAWAY_COLLAPSE",
            Self::NoRestoringSizeDynamics => "NO_RESTORING_SIZE_DYNAMICS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AmplificationBin {
    Leq125,
    From125To2,
    From2To5,
    From5To10,
    Gt10,
}

impl AmplificationBin {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Leq125 => "<=1.25x",
            Self::From125To2 => "1.25-2x",
            Self::From2To5 => "2-5x",
            Self::From5To10 => "5-10x",
            Self::Gt10 => ">10x",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyClass {
    AExternalInvaginations,
    BExteriorConnectedChannels,
    CClosedInternalVesicles,
    DDistributedInternalCarrierMembrane,
}

impl TopologyClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AExternalInvaginations => "Topology_A_external_invaginations",
            Self::BExteriorConnectedChannels => "Topology_B_exterior_connected_channels",
            Self::CClosedInternalVesicles => "Topology_C_closed_internal_vesicles",
            Self::DDistributedInternalCarrierMembrane => {
                "Topology_D_distributed_internal_carrier_membrane"
            }
        }
    }
}

/// Matched-state diagnostic snapshot (held chemistry, varying radius).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MatchedRadiusState {
    pub radius: f64,
    pub interface_length: f64,
    pub interior_area: f64,
    pub external_s_mass: f64,
    pub active_carrier_faces: f64,
    pub integrated_carrier_drive: f64,
    pub gross_productive_demand: f64,
    pub missing_nf_throughput: f64,
    pub capacity_rate: f64,
    pub k_t_star: f64,
}

/// Idealized 2D disk matched geometry with held concentrations / occupancy.
///
/// Demand scales with interior area; carrier capacity with interface length × held drive.
pub fn matched_disk_state(
    radius: f64,
    c_local: f64,
    a_local: f64,
    p_local: f64,
    membrane_occupancy: f64,
    n_activity: f64,
    f_activity: f64,
    w_activity: f64,
    productive_rate: f64,
    gamma_per_length: f64,
    drive: f64,
    s_per_length: f64,
) -> MatchedRadiusState {
    let _ = (c_local, a_local, p_local, n_activity, f_activity, w_activity);
    let r = radius.max(1e-9);
    let interface_length = std::f64::consts::TAU * r;
    let interior_area = std::f64::consts::PI * r * r;
    let active_faces = interface_length / face_measure_a_f().max(1e-12);
    let external_s_mass = membrane_occupancy.max(0.0) * s_per_length.max(0.0) * interface_length;
    let capacity_rate =
        gamma_per_length.max(0.0) * drive.max(0.0) * interface_length; // Γ D A /dt with A∫=L
    let demand = productive_rate.max(0.0) * interior_area;
    // Passive baseline omitted in matched geometry → missing ≈ demand when chi_passive=0 proxy.
    // For diagnostic scaling we treat gross demand as the missing proxy under matched chemistry.
    let missing = demand;
    let k_star = corrected_k_t_star(missing, capacity_rate).unwrap_or(0.0);
    MatchedRadiusState {
        radius: r,
        interface_length,
        interior_area,
        external_s_mass,
        active_carrier_faces: active_faces,
        integrated_carrier_drive: drive.max(0.0) * interface_length,
        gross_productive_demand: demand,
        missing_nf_throughput: missing,
        capacity_rate,
        k_t_star: k_star,
    }
}

pub fn classify_matched_scaling(
    p_m_matched: f64,
    p_t_matched: f64,
    p_m_d058: f64,
    _p_t_d058: f64,
) -> MatchedScalingClass {
    if !p_m_matched.is_finite() || !p_t_matched.is_finite() {
        return MatchedScalingClass::RadiusScalingInconclusive;
    }
    let sv_limit = p_m_matched > p_t_matched + 0.25 && p_m_matched >= 1.5 && p_t_matched <= 1.5;
    let confounded = (p_m_d058 - p_m_matched).abs() > 2.0;
    if sv_limit && confounded {
        return MatchedScalingClass::D058RadiusExponentConfounded;
    }
    if sv_limit {
        return MatchedScalingClass::MatchedSurfaceVolumeLimit;
    }
    if p_m_d058 > p_m_matched + 2.0 && p_m_matched.is_finite() {
        return MatchedScalingClass::CoupledStateScalingAmplification;
    }
    MatchedScalingClass::RadiusScalingInconclusive
}

/// Select ≤5 global diagnostic k_T values before any trajectory outcome.
///
/// Order: lowest, geometric-quarter, geometric-middle, geometric-three-quarter,
/// highest allowed by W availability / numerical stability ceiling.
pub fn select_global_k_t_ladder(
    k_lo: f64,
    k_hi: f64,
    k_w_stability_cap: f64,
) -> Result<Vec<f64>, &'static str> {
    if !(k_lo.is_finite() && k_hi.is_finite() && k_lo > 0.0 && k_hi > 0.0) {
        return Err("nonpositive_k_bounds");
    }
    if k_hi < k_lo {
        return Err("inverted_k_bounds");
    }
    let ratio = k_hi / k_lo;
    let q = |p: f64| k_lo * ratio.powf(p);
    let mut vals = vec![k_lo, q(0.25), q(0.5), q(0.75), k_hi.min(k_w_stability_cap)];
    // Enforce uniqueness while preserving order.
    vals.dedup_by(|a, b| (*a - *b).abs() <= 1e-15 * a.max(*b).max(1.0));
    if vals.len() > D059_MAX_GLOBAL_KT {
        vals.truncate(D059_MAX_GLOBAL_KT);
    }
    if vals.iter().any(|k| !k.is_finite() || *k <= 0.0) {
        return Err("invalid_ladder_entry");
    }
    Ok(vals)
}

/// Reject any attempt to fit or assign radius-specific k_T.
pub fn reject_radius_specific_k_t(proposed: &[(f64, f64)]) -> bool {
    if proposed.len() < 2 {
        return true;
    }
    let k0 = proposed[0].1;
    proposed.iter().all(|(_, k)| (*k - k0).abs() <= 1e-15 * k0.max(1.0))
}

pub fn classify_frontier_cell(
    chi_n: f64,
    chi_f: f64,
    w_util: f64,
    w_exhaust: bool,
    reverse_flux_risk: bool,
    numerically_invalid: bool,
    overtransport: bool,
) -> FrontierRegion {
    if numerically_invalid || !chi_n.is_finite() || !chi_f.is_finite() {
        return FrontierRegion::NumericallyInvalid;
    }
    if reverse_flux_risk {
        return FrontierRegion::ReverseFluxUnstable;
    }
    if w_exhaust || w_util > 0.98 {
        return FrontierRegion::WLimited;
    }
    if overtransport {
        return FrontierRegion::Overtransport;
    }
    if chi_n >= D059_CHI_VIABLE && chi_f >= D059_CHI_VIABLE {
        return FrontierRegion::ViableThroughput;
    }
    FrontierRegion::InsufficientImport
}

/// A viable carrier-rate region must contain >1 neighboring radius and >1 neighboring k_T.
pub fn viable_frontier_region_ok(radii: &[f64], k_ts: &[f64]) -> bool {
    contiguous_radii_count(radii) >= 2 && k_ts.len() >= 2
}

fn contiguous_radii_count(sorted_unique: &[f64]) -> usize {
    if sorted_unique.is_empty() {
        return 0;
    }
    let mut best = 1usize;
    let mut cur = 1usize;
    for w in sorted_unique.windows(2) {
        // Campaign radii are spaced by 2 (occasionally 4); treat ΔR≤4 as neighbors.
        if (w[1] - w[0]).abs() <= 4.0 + 1e-9 {
            cur += 1;
            best = best.max(cur);
        } else {
            cur = 1;
        }
    }
    best
}

fn contiguous_count(sorted_unique: &[f64]) -> usize {
    contiguous_radii_count(sorted_unique)
}

/// Longest contiguous viable radius run under one fixed k_T.
pub fn longest_contiguous_viable_radii(viable_sorted: &[f64]) -> usize {
    contiguous_count(viable_sorted)
}

pub fn radius_provisionally_viable(
    chi_n: f64,
    chi_f: f64,
    a_bounded: bool,
    a_retention_trend_ok: bool,
    p_active: bool,
    s_decline_arrested: bool,
    w_export_positive: bool,
    w_not_exhausted: bool,
    nf_not_exhausted: bool,
    no_rejection_cascade: bool,
    accounting_closes: bool,
) -> bool {
    chi_n >= D059_CHI_VIABLE
        && chi_f >= D059_CHI_VIABLE
        && a_bounded
        && a_retention_trend_ok
        && p_active
        && s_decline_arrested
        && w_export_positive
        && w_not_exhausted
        && nf_not_exhausted
        && no_rejection_cascade
        && accounting_closes
}

/// Classify restoring-size dynamics from signed radius velocities about a common R★.
///
/// `samples`: (R_init, dR/dt) with no target controller — endogenous only.
pub fn classify_restoring_size(
    samples: &[(f64, f64)],
    r_star: f64,
    eps: f64,
) -> RestoringSizeClass {
    if samples.len() < 3 || !r_star.is_finite() {
        return RestoringSizeClass::NoRestoringSizeDynamics;
    }
    let below: Vec<f64> = samples
        .iter()
        .filter(|(r, _)| *r < r_star - eps)
        .map(|(_, dr)| *dr)
        .collect();
    let above: Vec<f64> = samples
        .iter()
        .filter(|(r, _)| *r > r_star + eps)
        .map(|(_, dr)| *dr)
        .collect();
    if below.is_empty() || above.is_empty() {
        return RestoringSizeClass::OneSidedSizeLimit;
    }
    let below_grow = below.iter().filter(|dr| **dr > eps).count();
    let above_shrink = above.iter().filter(|dr| **dr < -eps).count();
    let below_shrink = below.iter().filter(|dr| **dr < -eps).count();
    let above_grow = above.iter().filter(|dr| **dr > eps).count();
    if below_grow * 2 >= below.len() && above_shrink * 2 >= above.len() {
        return RestoringSizeClass::RestoringSizeBasin;
    }
    if below_shrink == below.len() && above_shrink == above.len() {
        return RestoringSizeClass::RunawayCollapse;
    }
    if below_grow == below.len() && above_grow == above.len() {
        return RestoringSizeClass::RunawayGrowth;
    }
    if below.iter().all(|dr| dr.abs() <= eps) && above.iter().all(|dr| dr.abs() <= eps) {
        return RestoringSizeClass::NeutralSizeManifold;
    }
    if (below_grow > 0) ^ (above_shrink > 0) {
        return RestoringSizeClass::OneSidedSizeLimit;
    }
    RestoringSizeClass::NoRestoringSizeDynamics
}

/// Required carrier-active area: A_req = J_missing / (k_T · mean(Γ D)).
pub fn required_carrier_area(j_missing: f64, k_t: f64, mean_gamma_d: f64) -> Option<f64> {
    let den = k_t * mean_gamma_d;
    if den <= 1e-18 || !j_missing.is_finite() || j_missing < 0.0 {
        return None;
    }
    Some(j_missing / den)
}

pub fn area_amplification(a_required: f64, a_external: f64) -> Option<f64> {
    if a_external <= 1e-18 || !a_required.is_finite() {
        return None;
    }
    Some(a_required / a_external)
}

pub fn classify_amplification(alpha: f64) -> AmplificationBin {
    if alpha <= 1.25 {
        AmplificationBin::Leq125
    } else if alpha <= 2.0 {
        AmplificationBin::From125To2
    } else if alpha <= 5.0 {
        AmplificationBin::From2To5
    } else if alpha <= 10.0 {
        AmplificationBin::From5To10
    } else {
        AmplificationBin::Gt10
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MaterialBudget {
    pub external_s_mass: f64,
    pub min_s_boundary: f64,
    pub free_p: f64,
    pub sustainable_p_production: f64,
    pub s_per_carrier_area: f64,
    pub delta_m_s: f64,
    pub construction_time: f64,
    pub maintenance_demand: f64,
    pub a_cost_for_p: f64,
    pub throughput_for_extra_membrane: f64,
    pub bootstrap_possible: bool,
}

pub fn material_budget(
    external_s_mass: f64,
    min_s_boundary: f64,
    free_p: f64,
    sustainable_p_production: f64,
    s_per_carrier_area: f64,
    a_required_extra: f64,
    a_cost_per_p: f64,
    maintenance_rate: f64,
) -> MaterialBudget {
    let delta_m_s = a_required_extra.max(0.0) * s_per_carrier_area.max(0.0);
    let available = (external_s_mass - min_s_boundary).max(0.0) + free_p.max(0.0);
    let construction_time = if sustainable_p_production > 1e-18 {
        (delta_m_s - free_p.max(0.0)).max(0.0) / sustainable_p_production
    } else if delta_m_s <= free_p.max(0.0) + 1e-18 {
        0.0
    } else {
        f64::INFINITY
    };
    let bootstrap_possible = delta_m_s.is_finite()
        && available + sustainable_p_production * construction_time.max(0.0) + 1e-12 >= delta_m_s
        && construction_time.is_finite()
        // Reject requiring the full additional area before any import is possible.
        && !(a_required_extra > 0.0 && external_s_mass < min_s_boundary && free_p <= 1e-18);
    MaterialBudget {
        external_s_mass,
        min_s_boundary,
        free_p,
        sustainable_p_production,
        s_per_carrier_area,
        delta_m_s,
        construction_time,
        maintenance_demand: maintenance_rate.max(0.0) * a_required_extra.max(0.0),
        a_cost_for_p: a_cost_per_p.max(0.0) * delta_m_s.max(0.0),
        throughput_for_extra_membrane: maintenance_rate.max(0.0) * a_required_extra.max(0.0),
        bootstrap_possible,
    }
}

/// Environmental connectivity: area counts only with access to both env N/F and internal W.
pub fn environmentally_connected(
    has_exterior_nf_contact: bool,
    has_interior_w_contact: bool,
    sealed_compartment: bool,
) -> bool {
    has_exterior_nf_contact && has_interior_w_contact && !sealed_compartment
}

pub fn topology_admissible(class: TopologyClass, connected: bool, conservative: bool) -> bool {
    match class {
        TopologyClass::CClosedInternalVesicles => false,
        TopologyClass::AExternalInvaginations
        | TopologyClass::BExteriorConnectedChannels
        | TopologyClass::DDistributedInternalCarrierMembrane => connected && conservative,
    }
}

/// Observer-only local area multiplier α_Γ — must be material-derived and bounded.
pub fn area_multiplier_valid(
    alpha: f64,
    mature_s: f64,
    free_scalar: bool,
    uses_health_or_radius: bool,
    environmentally_connected: bool,
) -> bool {
    !free_scalar
        && !uses_health_or_radius
        && environmentally_connected
        && mature_s > 1e-18
        && alpha.is_finite()
        && alpha >= 0.0
}

/// Carrier throughput with explicit area amplification: J = k_T ∫ α Γ D dΓ.
pub fn amplified_throughput(k_t: f64, alpha_integral_gamma_d: f64) -> f64 {
    k_t.max(0.0) * alpha_integral_gamma_d.max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteEvidence059 {
    pub workspace_isolated: bool,
    pub d058_route_v_reproduced: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub contiguous_viable_radii: bool,
    pub restoring_basin: bool,
    pub size_limit_no_restore: bool,
    pub starvation_ok: bool,
    pub area_amplification_bounded: bool,
    pub material_bootstrap_ok: bool,
    pub topology_justified: bool,
    pub area_architecture_not_justified: bool,
    pub carrier_surface_rejected: bool,
}

pub fn select_route(ev: RouteEvidence059) -> (D059Route, D059PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D059Route::I,
            D059PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d058_route_v_reproduced {
        return (
            D059Route::I,
            D059PrimaryConclusion::D058RouteVNotReproduced,
        );
    }
    if !ev.accounting_ok {
        return (D059Route::I, D059PrimaryConclusion::AccountingFailure);
    }
    if !ev.numerical_ok {
        return (D059Route::I, D059PrimaryConclusion::NumericalFailure);
    }
    if ev.contiguous_viable_radii && ev.restoring_basin && ev.starvation_ok {
        return (D059Route::S, D059Route::S.conclusion());
    }
    if ev.contiguous_viable_radii && ev.size_limit_no_restore {
        return (D059Route::L, D059Route::L.conclusion());
    }
    if !ev.contiguous_viable_radii {
        if ev.topology_justified && ev.area_amplification_bounded && ev.material_bootstrap_ok {
            return (D059Route::M, D059Route::M.conclusion());
        }
        if ev.area_amplification_bounded && !ev.material_bootstrap_ok {
            return (D059Route::B, D059Route::B.conclusion());
        }
        if ev.area_architecture_not_justified || ev.carrier_surface_rejected {
            if ev.carrier_surface_rejected {
                return (D059Route::R, D059Route::R.conclusion());
            }
            return (
                D059Route::I,
                D059PrimaryConclusion::InternalMembraneAreaArchitectureNotJustified,
            );
        }
        return (
            D059Route::I,
            D059PrimaryConclusion::NoViableExternalCarrierRadius,
        );
    }
    (D059Route::I, D059Route::I.conclusion())
}

/// Reproduce D-058 Gate-0 style check from sealed numbers.
pub fn d058_route_v_reproduced(
    k_t_span: f64,
    portable_candidate: bool,
    primary: &str,
) -> bool {
    k_t_span > D059_RATE_SPAN_FAIL
        && !portable_candidate
        && primary == D059_D058_CONCLUSION
}

pub fn fit_matched_exponents(states: &[MatchedRadiusState]) -> (Option<f64>, Option<f64>) {
    let rs: Vec<f64> = states.iter().map(|s| s.radius).collect();
    let jm: Vec<f64> = states.iter().map(|s| s.missing_nf_throughput.max(1e-18)).collect();
    let cap: Vec<f64> = states.iter().map(|s| s.capacity_rate.max(1e-18)).collect();
    (
        scaling_exponent(&rs, &jm),
        scaling_exponent(&rs, &cap),
    )
}

pub fn predicted_chi(import: f64, demand: f64) -> f64 {
    if demand <= 1e-18 {
        return if import >= 0.0 { f64::INFINITY } else { 0.0 };
    }
    import / demand
}

/// Shadow isolation: production biology defaults must remain unchanged markers.
pub fn shadow_isolation_ok(production_carrier_enabled: bool, v15_authorized: bool) -> bool {
    !production_carrier_enabled && !v15_authorized
}

/// Convenience: face amount under corrected operator (re-export style).
#[inline]
pub fn shadow_xi(k_t: f64, gamma: f64, drive: f64, dt: f64) -> f64 {
    xi_face_req(k_t, gamma, drive, face_measure_a_f(), dt)
}

#[inline]
pub fn shadow_capacity(gamma: f64, drive: f64, dt: f64) -> f64 {
    capacity_contrib(gamma, drive, face_measure_a_f(), dt)
}

pub use crate::d057_analysis::rate_span as k_t_span;
pub use crate::d058_analysis::D058_RATE_SPAN_MAX;

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn matched_disk_scales_area_and_perimeter() {
        let a = matched_disk_state(10.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.01, 1.0, 0.5, 0.1);
        let b = matched_disk_state(20.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.01, 1.0, 0.5, 0.1);
        assert!((b.interior_area / a.interior_area - 4.0).abs() < 1e-9);
        assert!((b.interface_length / a.interface_length - 2.0).abs() < 1e-9);
    }

    #[test]
    fn ladder_is_predeclared_and_bounded() {
        let v = select_global_k_t_ladder(0.01, 1.0, 10.0).unwrap();
        assert!(v.len() <= 5);
        assert!((v[0] - 0.01).abs() < 1e-15);
        assert!(v.last().unwrap() <= &1.0);
    }
}
