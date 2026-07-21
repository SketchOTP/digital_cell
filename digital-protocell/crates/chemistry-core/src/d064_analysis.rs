//! D-064 connected-geometry coupled rejection and membrane-load decomposition.
//! Shadow/observer diagnostics only — no production carrier or morphogenesis.

use crate::d058_analysis::{cell_volume, face_measure_a_f, xi_face_req};
use crate::d063_analysis::{
    account_geometry, classify_membrane_face, exterior_connected_mask, generate_phi,
    seed_mature_s_on_interfaces, smooth_baseline_length, GeometryFamily, GeometrySpec,
    MembraneFaceClass, D063_FROZEN_KT, D063_PHI_INTERIOR,
};
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

pub const D064_PROJECT_ID: &str = "D-064";
pub const D064_AGENT_MEMORY_ID: &str =
    "D-20260721-d064-connected-geometry-coupled-rejection-decomposition";
pub const D064_STARTING_COMMIT: &str = "3ab07cb";
pub const D064_STARTING_TAG: &str = "D-063-connected-membrane-architecture-review";
pub const D064_D063_CONCLUSION: &str = "D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE";
pub const D064_FROZEN_KT: f64 = D063_FROZEN_KT;
pub const D064_CHI_VIABLE: f64 = 1.05;
pub const D064_A_RETENTION_TARGET: f64 = 0.80;
pub const D064_C_RETENTION_TARGET: f64 = 0.80;
pub const D064_PRODUCTIVE_DEMAND_DENSITY: f64 = 0.01;
pub const D064_GAMMA_DRIVE_STATIC_LEGACY: f64 = 0.35;
pub const D064_EPS: f64 = 1e-18;
pub const D064_LEDGER_TOL: f64 = 1e-4;
pub const D064_OVERCOMMIT_EPS: f64 = 1e-12;
pub const D064_RECORD_STATIC_CAPACITY: &str =
    "CONNECTED_AREA_STATIC_CAPACITY_QUALIFIED_COUPLED_CAUSE_UNRESOLVED";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D064PrimaryConclusion {
    StaticCoupledResourceMetricDefect,
    MultifaceCarrierBudgetingDefect,
    ConnectedGeometryDiscretizationDefect,
    PrebuiltConnectedSeedNonequilibrium,
    ConnectedAreaExchangeLoadFailure,
    ConnectedAreaPrecursorDemandFailure,
    ConnectedMembraneCoupledShadowQualified,
    ConnectedMembraneNotPrimaryCoupledRepair,
    ConnectedGeometryFailureDecompositionInconclusive,
    D063CoupledFailureNotReproduced,
    ResourceSufficiencyAccountingFailure,
    RejectionProvenanceUnresolved,
    PrebuiltSeedAccountingFailure,
    ConnectedGeometryApsLedgerFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D064PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaticCoupledResourceMetricDefect => {
                "D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT"
            }
            Self::MultifaceCarrierBudgetingDefect => "D064_MULTIFACE_CARRIER_BUDGETING_DEFECT",
            Self::ConnectedGeometryDiscretizationDefect => {
                "D064_CONNECTED_GEOMETRY_DISCRETIZATION_DEFECT"
            }
            Self::PrebuiltConnectedSeedNonequilibrium => {
                "D064_PREBUILT_CONNECTED_SEED_NONEQUILIBRIUM"
            }
            Self::ConnectedAreaExchangeLoadFailure => "D064_CONNECTED_AREA_EXCHANGE_LOAD_FAILURE",
            Self::ConnectedAreaPrecursorDemandFailure => {
                "D064_CONNECTED_AREA_PRECURSOR_DEMAND_FAILURE"
            }
            Self::ConnectedMembraneCoupledShadowQualified => {
                "D064_CONNECTED_MEMBRANE_COUPLED_SHADOW_QUALIFIED"
            }
            Self::ConnectedMembraneNotPrimaryCoupledRepair => {
                "D064_CONNECTED_MEMBRANE_NOT_PRIMARY_COUPLED_REPAIR"
            }
            Self::ConnectedGeometryFailureDecompositionInconclusive => {
                "D064_CONNECTED_GEOMETRY_FAILURE_DECOMPOSITION_INCONCLUSIVE"
            }
            Self::D063CoupledFailureNotReproduced => "D064_D063_COUPLED_FAILURE_NOT_REPRODUCED",
            Self::ResourceSufficiencyAccountingFailure => {
                "D064_RESOURCE_SUFFICIENCY_ACCOUNTING_FAILURE"
            }
            Self::RejectionProvenanceUnresolved => "D064_REJECTION_PROVENANCE_UNRESOLVED",
            Self::PrebuiltSeedAccountingFailure => "D064_PREBUILT_SEED_ACCOUNTING_FAILURE",
            Self::ConnectedGeometryApsLedgerFailure => "D064_CONNECTED_GEOMETRY_APS_LEDGER_FAILURE",
            Self::WorkspaceScopeNotIsolated => "D064_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D064_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D064_NUMERICAL_FAILURE",
            Self::Fail => "D064_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D064Route {
    A,
    B,
    G,
    S,
    E,
    P,
    Q,
    N,
    I,
}

impl D064Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "Route_A_static_coupled_resource_metric_defect",
            Self::B => "Route_B_multiface_carrier_budgeting_defect",
            Self::G => "Route_G_connected_geometry_discretization_defect",
            Self::S => "Route_S_prebuilt_connected_seed_nonequilibrium",
            Self::E => "Route_E_connected_area_exchange_load_failure",
            Self::P => "Route_P_connected_area_precursor_demand_failure",
            Self::Q => "Route_Q_connected_membrane_coupled_shadow_qualified",
            Self::N => "Route_N_connected_membrane_not_primary_coupled_repair",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D064PrimaryConclusion {
        match self {
            Self::A => D064PrimaryConclusion::StaticCoupledResourceMetricDefect,
            Self::B => D064PrimaryConclusion::MultifaceCarrierBudgetingDefect,
            Self::G => D064PrimaryConclusion::ConnectedGeometryDiscretizationDefect,
            Self::S => D064PrimaryConclusion::PrebuiltConnectedSeedNonequilibrium,
            Self::E => D064PrimaryConclusion::ConnectedAreaExchangeLoadFailure,
            Self::P => D064PrimaryConclusion::ConnectedAreaPrecursorDemandFailure,
            Self::Q => D064PrimaryConclusion::ConnectedMembraneCoupledShadowQualified,
            Self::N => D064PrimaryConclusion::ConnectedMembraneNotPrimaryCoupledRepair,
            Self::I => D064PrimaryConclusion::ConnectedGeometryFailureDecompositionInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RejectionClass {
    CarrierNOverdraw,
    CarrierFOverdraw,
    CarrierWOverdraw,
    PassiveTransportOverdraw,
    PSExchangeOverdraw,
    ReactionOverdraw,
    PhiUpdateFailure,
    CombinedOperatorOvercommit,
    TimestepStiffness,
    UnknownRejectionSource,
}

impl RejectionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CarrierNOverdraw => "CARRIER_N_OVERDRAW",
            Self::CarrierFOverdraw => "CARRIER_F_OVERDRAW",
            Self::CarrierWOverdraw => "CARRIER_W_OVERDRAW",
            Self::PassiveTransportOverdraw => "PASSIVE_TRANSPORT_OVERDRAW",
            Self::PSExchangeOverdraw => "P_S_EXCHANGE_OVERDRAW",
            Self::ReactionOverdraw => "REACTION_OVERDRAW",
            Self::PhiUpdateFailure => "PHI_UPDATE_FAILURE",
            Self::CombinedOperatorOvercommit => "COMBINED_OPERATOR_OVERCOMMIT",
            Self::TimestepStiffness => "TIMESTEP_STIFFNESS",
            Self::UnknownRejectionSource => "UNKNOWN_REJECTION_SOURCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeometryStiffnessClass {
    SubgridChannelStiffness,
    HighCurvatureFaceMultiplicity,
    GeometryDiscretizationAcceptable,
    GeometryStiffnessInconclusive,
}

impl GeometryStiffnessClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubgridChannelStiffness => "SUBGRID_CHANNEL_STIFFNESS",
            Self::HighCurvatureFaceMultiplicity => "HIGH_CURVATURE_FACE_MULTIPLICITY",
            Self::GeometryDiscretizationAcceptable => "GEOMETRY_DISCRETIZATION_ACCEPTABLE",
            Self::GeometryStiffnessInconclusive => "GEOMETRY_STIFFNESS_INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeedEquilibriumClass {
    PrebuiltSeedExchangeBalanced,
    PrebuiltSeedDesorptionLoaded,
    PrebuiltSeedAdsorptionLoaded,
    PrebuiltSeedMaterialInconsistent,
}

impl SeedEquilibriumClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrebuiltSeedExchangeBalanced => "PREBUILT_SEED_EXCHANGE_BALANCED",
            Self::PrebuiltSeedDesorptionLoaded => "PREBUILT_SEED_DESORPTION_LOADED",
            Self::PrebuiltSeedAdsorptionLoaded => "PREBUILT_SEED_ADSORPTION_LOADED",
            Self::PrebuiltSeedMaterialInconsistent => "PREBUILT_SEED_MATERIAL_INCONSISTENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoupledLoadClass {
    CarrierNumericalLoad,
    MembraneExchangeLoad,
    PrecursorProductionLoad,
    ActivationResourceLoad,
    PassiveLeakageLoad,
    MultipleCoupledLoads,
    CoupledLoadUnresolved,
}

impl CoupledLoadClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CarrierNumericalLoad => "CARRIER_NUMERICAL_LOAD",
            Self::MembraneExchangeLoad => "MEMBRANE_EXCHANGE_LOAD",
            Self::PrecursorProductionLoad => "PRECURSOR_PRODUCTION_LOAD",
            Self::ActivationResourceLoad => "ACTIVATION_RESOURCE_LOAD",
            Self::PassiveLeakageLoad => "PASSIVE_LEAKAGE_LOAD",
            Self::MultipleCoupledLoads => "MULTIPLE_COUPLED_LOADS",
            Self::CoupledLoadUnresolved => "COUPLED_LOAD_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpperBoundClass {
    ConnectedGeometryCapableRemainingDeliveryDefect,
    ConnectedGeometryNotPrimaryCoupledRepair,
}

impl UpperBoundClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConnectedGeometryCapableRemainingDeliveryDefect => {
                "CONNECTED_GEOMETRY_CAPABLE_REMAINING_DELIVERY_DEFECT"
            }
            Self::ConnectedGeometryNotPrimaryCoupledRepair => {
                "CONNECTED_GEOMETRY_NOT_PRIMARY_COUPLED_REPAIR"
            }
        }
    }
}

/// Canonical resource-sufficiency window (accepted flux only).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ResourceSufficiencyWindow {
    pub j_n_passive_accepted: f64,
    pub j_n_carrier_accepted: f64,
    pub j_f_passive_accepted: f64,
    pub j_f_carrier_accepted: f64,
    pub l_n_required: f64,
    pub l_f_required: f64,
    pub accepted_steps: u64,
    pub window_time: f64,
}

impl ResourceSufficiencyWindow {
    pub fn chi_n(self) -> f64 {
        chi_ratio(
            self.j_n_passive_accepted + self.j_n_carrier_accepted,
            self.l_n_required,
        )
    }

    pub fn chi_f(self) -> f64 {
        chi_ratio(
            self.j_f_passive_accepted + self.j_f_carrier_accepted,
            self.l_f_required,
        )
    }

    pub fn chi_min(self) -> f64 {
        self.chi_n().min(self.chi_f())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CarrierFaceRequest {
    pub inside: usize,
    pub outside: usize,
    pub face_id: usize,
    pub xi_req: f64,
    pub topology: MembraneFaceClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellBudgetAudit {
    pub max_omega_n: f64,
    pub max_omega_f: f64,
    pub max_omega_w: f64,
    pub max_omega_p: f64,
    pub max_omega_s: f64,
    pub p95_omega_n: f64,
    pub p95_omega_f: f64,
    pub p95_omega_w: f64,
    pub cells_overcommitted_n: usize,
    pub cells_overcommitted_f: usize,
    pub cells_overcommitted_w: usize,
    pub multiface_defect: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence064 {
    pub workspace_isolated: bool,
    pub d063_reproduced: bool,
    pub accounting_reconciled: bool,
    pub static_used_requested_flux: bool,
    pub rejection_provenance_resolved: bool,
    pub multiface_budget_defect: bool,
    pub joint_allocator_rescues: bool,
    pub geometry_discretization_defect: bool,
    pub seed_nonequilibrium: bool,
    pub seed_material_inconsistent: bool,
    pub exchange_load_dominant: bool,
    pub precursor_demand_dominant: bool,
    pub aps_ledger_ok: bool,
    pub short_screen_pass: bool,
    pub authoritative_pass: bool,
    pub upper_bound_restores_aps: bool,
    pub upper_bound_still_collapses: bool,
}

/// χ = accepted_supply / required_load (accepted flux only).
pub fn chi_ratio(accepted_supply: f64, required_load: f64) -> f64 {
    if required_load <= D064_EPS {
        return if accepted_supply >= 0.0 {
            f64::INFINITY
        } else {
            0.0
        };
    }
    accepted_supply / required_load
}

pub fn productive_demand(interior_area: f64, window_time: f64) -> f64 {
    D064_PRODUCTIVE_DEMAND_DENSITY * interior_area.max(0.0) * window_time.max(0.0)
}

/// Legacy D-063 analytical (requested) capacity — must NOT enter canonical χ numerator.
pub fn legacy_analytical_requested_capacity(connected_length: f64, dt: f64) -> f64 {
    D064_FROZEN_KT * D064_GAMMA_DRIVE_STATIC_LEGACY * connected_length.max(0.0) * dt.max(0.0)
}

pub fn evaluate_resource_sufficiency(w: ResourceSufficiencyWindow) -> ResourceSufficiencyWindow {
    w
}

pub fn static_coupled_accounting_mismatch(
    static_used_requested_or_unbounded: bool,
    static_demand_differs: bool,
    static_time_norm_differs: bool,
) -> bool {
    static_used_requested_or_unbounded || static_demand_differs || static_time_norm_differs
}

pub fn classify_rejection_from_detail(
    limiter: &str,
    detail: &str,
    carrier_applied_prev: bool,
    omega_n: f64,
    omega_f: f64,
    omega_w: f64,
) -> RejectionClass {
    let d = detail.to_ascii_lowercase();
    let lim = limiter.to_ascii_uppercase();
    if omega_n > 1.0 + 1e-6 && carrier_applied_prev {
        return RejectionClass::CarrierNOverdraw;
    }
    if omega_f > 1.0 + 1e-6 && carrier_applied_prev {
        return RejectionClass::CarrierFOverdraw;
    }
    if omega_w > 1.0 + 1e-6 && carrier_applied_prev {
        return RejectionClass::CarrierWOverdraw;
    }
    if d.contains("excessive concentration") && d.contains("waste") && carrier_applied_prev {
        return RejectionClass::CarrierWOverdraw;
    }
    if lim.contains("INCOMINGSTATEINVALID") && d.contains("waste") && carrier_applied_prev {
        return RejectionClass::CarrierWOverdraw;
    }
    if d.contains("surface_exchange") {
        return RejectionClass::PSExchangeOverdraw;
    }
    if lim.contains("POSITIVITY") || d.contains("neg_clamp") {
        if carrier_applied_prev && (omega_n.max(omega_f).max(omega_w) > 0.95) {
            return RejectionClass::CombinedOperatorOvercommit;
        }
        if d.contains("nutrient") || d.contains("fuel") || d.contains("waste") {
            return RejectionClass::PassiveTransportOverdraw;
        }
        return RejectionClass::TimestepStiffness;
    }
    if lim.contains("STRUCTURE") || d.contains("structure") || d.contains("phi") {
        return RejectionClass::PhiUpdateFailure;
    }
    if lim.contains("REACTION") || lim.contains("ACTIVATION") || lim.contains("TURNOVER") {
        return RejectionClass::ReactionOverdraw;
    }
    if lim.contains("ADAPTIVE") || lim.contains("FIELD_BOUND") {
        return RejectionClass::TimestepStiffness;
    }
    RejectionClass::UnknownRejectionSource
}

pub fn omega_overcommit(xi_out: f64, budget: f64) -> f64 {
    xi_out / budget.max(D064_OVERCOMMIT_EPS)
}

pub fn aggregate_outgoing_xi(requests: &[CarrierFaceRequest], field_is_waste: bool) -> Vec<(usize, f64)> {
    use std::collections::HashMap;
    let mut map: HashMap<usize, f64> = HashMap::new();
    for r in requests {
        let amt = r.xi_req.max(0.0);
        if amt <= 0.0 {
            continue;
        }
        if field_is_waste {
            *map.entry(r.inside).or_default() += amt;
        } else {
            // N/F: each takes 0.5·ξ from the exterior cell (d063 stoichiometry).
            *map.entry(r.outside).or_default() += 0.5 * amt;
        }
    }
    let mut out: Vec<_> = map.into_iter().collect();
    out.sort_by_key(|(i, _)| *i);
    out
}

pub fn cell_budget_audit(
    requests: &[CarrierFaceRequest],
    n: &[f64],
    f: &[f64],
    w: &[f64],
    p: &[f64],
    s: &[f64],
) -> CellBudgetAudit {
    let vol = cell_volume();
    let n_out = aggregate_outgoing_xi(requests, false);
    let f_out = aggregate_outgoing_xi(requests, false);
    let w_out = aggregate_outgoing_xi(requests, true);

    let mut omegas_n = Vec::new();
    let mut omegas_f = Vec::new();
    let mut omegas_w = Vec::new();
    let mut over_n = 0usize;
    let mut over_f = 0usize;
    let mut over_w = 0usize;

    for (idx, xi) in &n_out {
        let b = n.get(*idx).copied().unwrap_or(0.0).max(0.0) * vol;
        let o = omega_overcommit(*xi, b);
        omegas_n.push(o);
        if o > 1.0 + 1e-9 {
            over_n += 1;
        }
    }
    for (idx, xi) in &f_out {
        let b = f.get(*idx).copied().unwrap_or(0.0).max(0.0) * vol;
        let o = omega_overcommit(*xi, b);
        omegas_f.push(o);
        if o > 1.0 + 1e-9 {
            over_f += 1;
        }
    }
    for (idx, xi) in &w_out {
        let b = w.get(*idx).copied().unwrap_or(0.0).max(0.0) * vol;
        let o = omega_overcommit(*xi, b);
        omegas_w.push(o);
        if o > 1.0 + 1e-9 {
            over_w += 1;
        }
    }

    // P/S: no carrier outbound; report occupancy ratios as diagnostic placeholders.
    let max_omega_p = p.iter().copied().fold(0.0_f64, f64::max);
    let max_omega_s = s.iter().copied().fold(0.0_f64, f64::max);

    let multiface = over_n + over_f + over_w > 0;
    CellBudgetAudit {
        max_omega_n: percentile_max(&omegas_n, 1.0),
        max_omega_f: percentile_max(&omegas_f, 1.0),
        max_omega_w: percentile_max(&omegas_w, 1.0),
        max_omega_p,
        max_omega_s,
        p95_omega_n: percentile_max(&omegas_n, 0.95),
        p95_omega_f: percentile_max(&omegas_f, 0.95),
        p95_omega_w: percentile_max(&omegas_w, 0.95),
        cells_overcommitted_n: over_n,
        cells_overcommitted_f: over_f,
        cells_overcommitted_w: over_w,
        multiface_defect: multiface,
    }
}

fn percentile_max(vals: &[f64], q: f64) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if q >= 1.0 {
        return *v.last().unwrap_or(&0.0);
    }
    let idx = ((v.len() as f64 - 1.0) * q.clamp(0.0, 1.0)).round() as usize;
    v[idx.min(v.len() - 1)]
}

/// Conservative joint face allocator: cell-wise λ then symmetric face scale; preserves N/F/W stoichiometry.
pub fn joint_allocate_faces(
    requests: &[CarrierFaceRequest],
    n: &[f64],
    f: &[f64],
    w: &[f64],
) -> Vec<f64> {
    let vol = cell_volume();
    let n_len = n.len().max(f.len()).max(w.len());
    let mut xi_n = vec![0.0; n_len];
    let mut xi_f = vec![0.0; n_len];
    let mut xi_w = vec![0.0; n_len];
    for r in requests {
        let amt = r.xi_req.max(0.0);
        if amt <= 0.0 {
            continue;
        }
        if r.outside < n_len {
            xi_n[r.outside] += 0.5 * amt;
            xi_f[r.outside] += 0.5 * amt;
        }
        if r.inside < n_len {
            xi_w[r.inside] += amt;
        }
    }
    let mut lambda_n = vec![1.0; n_len];
    let mut lambda_f = vec![1.0; n_len];
    let mut lambda_w = vec![1.0; n_len];
    for i in 0..n_len {
        let bn = n.get(i).copied().unwrap_or(0.0).max(0.0) * vol;
        let bf = f.get(i).copied().unwrap_or(0.0).max(0.0) * vol;
        let bw = w.get(i).copied().unwrap_or(0.0).max(0.0) * vol;
        if xi_n[i] > D064_EPS {
            lambda_n[i] = (bn / xi_n[i]).min(1.0);
        }
        if xi_f[i] > D064_EPS {
            lambda_f[i] = (bf / xi_f[i]).min(1.0);
        }
        if xi_w[i] > D064_EPS {
            lambda_w[i] = (bw / xi_w[i]).min(1.0);
        }
    }
    let mut scaled = Vec::with_capacity(requests.len());
    for r in requests {
        let ln = lambda_n.get(r.outside).copied().unwrap_or(1.0);
        let lf = lambda_f.get(r.outside).copied().unwrap_or(1.0);
        let lw = lambda_w.get(r.inside).copied().unwrap_or(1.0);
        let face_lambda = ln.min(lf).min(lw).clamp(0.0, 1.0);
        scaled.push(r.xi_req * face_lambda);
    }
    scaled
}

pub fn joint_allocator_order_invariant(a: &[f64], b: &[f64], tol: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| (x - y).abs() <= tol)
}

pub fn classify_geometry_stiffness(
    min_channel_width: f64,
    median_channel_width: f64,
    max_active_faces_per_cell: usize,
    diagonal_iface_frac: f64,
) -> GeometryStiffnessClass {
    if min_channel_width < 1.5 && median_channel_width < 2.0 {
        return GeometryStiffnessClass::SubgridChannelStiffness;
    }
    if max_active_faces_per_cell >= 4 || diagonal_iface_frac > 0.35 {
        return GeometryStiffnessClass::HighCurvatureFaceMultiplicity;
    }
    if min_channel_width >= 2.0 {
        return GeometryStiffnessClass::GeometryDiscretizationAcceptable;
    }
    GeometryStiffnessClass::GeometryStiffnessInconclusive
}

/// Local Langmuir equilibrium θ* = K p / (1 + K p).
pub fn theta_eq(k_eq: f64, p_activity: f64) -> f64 {
    let kp = k_eq.max(0.0) * p_activity.max(0.0);
    kp / (1.0 + kp)
}

pub fn exchange_imbalance(
    alpha: f64,
    beta: f64,
    q_c: f64,
    p_activity: f64,
    theta: f64,
) -> f64 {
    // E_PS = J_ads - J_des = q (α p (1-θ) - β θ)
    q_c.max(0.0) * (alpha * p_activity.max(0.0) * (1.0 - theta).max(0.0) - beta * theta.max(0.0))
}

pub fn classify_seed_equilibrium(
    integrated_e_ps: f64,
    frac_overoccupied: f64,
    material_inconsistent: bool,
) -> SeedEquilibriumClass {
    if material_inconsistent {
        return SeedEquilibriumClass::PrebuiltSeedMaterialInconsistent;
    }
    if integrated_e_ps < -1e-6 || frac_overoccupied > 0.05 {
        return SeedEquilibriumClass::PrebuiltSeedDesorptionLoaded;
    }
    if integrated_e_ps > 1e-6 {
        return SeedEquilibriumClass::PrebuiltSeedAdsorptionLoaded;
    }
    SeedEquilibriumClass::PrebuiltSeedExchangeBalanced
}

/// Material-conservative redistribute: scale interface S so total S equals baseline_s_total.
pub fn redistribute_s_conserve_total(
    grid: &Grid,
    phi: &[f64],
    baseline_s_total: f64,
    s_per_length: f64,
) -> Vec<f64> {
    let raw = seed_mature_s_on_interfaces(grid, phi, s_per_length);
    let mut total = 0.0;
    for (idx, &v) in raw.iter().enumerate() {
        if grid.in_dish(idx) {
            total += v * face_measure_a_f(); // S stored as surface density share; mass ~ sum*DX^2
        }
    }
    // Match d063 total_surface_mass convention: Σ s_i * DX² with DX=1 → Σ s_i.
    let total_mass: f64 = raw
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, &v)| v)
        .sum();
    let scale = if total_mass > D064_EPS {
        baseline_s_total / total_mass
    } else {
        0.0
    };
    let _ = total;
    raw.into_iter().map(|v| v * scale).collect()
}

pub fn classify_coupled_load(
    carrier_only_rejects: bool,
    exchange_off_stable: bool,
    precursor_off_helps: bool,
    activation_off_helps: bool,
    passive_only_ok: bool,
) -> CoupledLoadClass {
    let mut hits = 0;
    let mut primary = CoupledLoadClass::CoupledLoadUnresolved;
    if carrier_only_rejects {
        hits += 1;
        primary = CoupledLoadClass::CarrierNumericalLoad;
    }
    if exchange_off_stable {
        hits += 1;
        primary = CoupledLoadClass::MembraneExchangeLoad;
    }
    if precursor_off_helps {
        hits += 1;
        primary = CoupledLoadClass::PrecursorProductionLoad;
    }
    if activation_off_helps {
        hits += 1;
        primary = CoupledLoadClass::ActivationResourceLoad;
    }
    if !passive_only_ok {
        hits += 1;
        primary = CoupledLoadClass::PassiveLeakageLoad;
    }
    if hits >= 2 {
        CoupledLoadClass::MultipleCoupledLoads
    } else if hits == 1 {
        primary
    } else {
        CoupledLoadClass::CoupledLoadUnresolved
    }
}

pub fn ledger_closes(observed: f64, ledger: f64, tol: f64) -> bool {
    (observed - ledger).abs() <= tol.max(D064_LEDGER_TOL) * (1.0 + observed.abs().max(ledger.abs()))
}

pub fn short_screen_admits(
    chi_n: f64,
    chi_f: f64,
    a_ret: f64,
    c_ret: f64,
    s_declining: bool,
    rejection_cascade: bool,
) -> bool {
    chi_n >= D064_CHI_VIABLE
        && chi_f >= D064_CHI_VIABLE
        && a_ret >= D064_A_RETENTION_TARGET
        && c_ret >= D064_C_RETENTION_TARGET
        && !s_declining
        && !rejection_cascade
}

pub fn d063_failure_reproduced(
    static_chi_pass: bool,
    a_ret: f64,
    s_initial: f64,
    s_final: f64,
    accepted_before_cascade: u64,
    steps_ok: bool,
) -> bool {
    // Physical D-063 failure signature (χ proxy is not required — D-063 coupled χ
    // used accepted-step count as time, which is an accounting defect under audit).
    static_chi_pass
        && a_ret < 0.6
        && s_final < s_initial * 0.85
        && (!steps_ok || accepted_before_cascade < 2000)
}

/// D-063-style coupled χ proxy (demand ∝ accepted steps, treating Δt≡1). Diagnostic only.
pub fn legacy_d063_chi_proxy(import: f64, interior_area: f64, accepted: u64) -> f64 {
    let demand = D064_PRODUCTIVE_DEMAND_DENSITY * interior_area.max(0.0) * accepted as f64;
    chi_ratio(import, demand)
}

pub fn sealed_radial_r22_spec() -> GeometrySpec {
    GeometrySpec::radial(22.0, 8, 0.45, 2.5)
}

pub fn measure_geometry(spec: &GeometrySpec) -> crate::d063_analysis::GeometryAccount {
    let grid = Grid::new();
    let phi = generate_phi(&grid, spec);
    let s = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let base = smooth_baseline_length(spec.radius);
    let mut acc = account_geometry(&grid, &phi, &s, base, spec.radius);
    acc.family = spec.family;
    acc
}

pub fn collect_carrier_requests(
    grid: &Grid,
    phi: &[f64],
    membrane: &[f64],
    nutrient: &[f64],
    fuel: &[f64],
    waste: &[f64],
    connected: &[bool],
    k_t: f64,
    k_nf0: f64,
    k_w0: f64,
    delta_floor: f64,
    dt: f64,
    gamma_face: impl Fn(f64, f64, f64, f64, f64) -> f64,
    drive_face: impl Fn(f64, f64, f64, f64, f64, f64, f64, f64) -> f64,
) -> Vec<CarrierFaceRequest> {
    let face_area = face_measure_a_f();
    let mut out = Vec::new();
    let mut face_id = 0usize;
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
            let (inside, outside) = if a { (idx, jdx) } else { (jdx, idx) };
            let on_envelope = !connected_is_invagination_proxy(grid, outside, phi);
            let topology = classify_membrane_face(outside, connected, true, on_envelope);
            if !topology.carrier_active() {
                face_id += 1;
                continue;
            }
            let gamma = gamma_face(
                membrane[idx],
                phi[idx],
                membrane[jdx],
                phi[jdx],
                delta_floor,
            );
            if gamma <= D064_EPS {
                face_id += 1;
                continue;
            }
            let drive = drive_face(
                nutrient[outside],
                fuel[outside],
                waste[inside],
                nutrient[inside],
                fuel[inside],
                waste[outside],
                k_nf0,
                k_w0,
            );
            out.push(CarrierFaceRequest {
                inside,
                outside,
                face_id,
                xi_req: xi_face_req(k_t, gamma, drive, face_area, dt),
                topology,
            });
            face_id += 1;
        }
    }
    out
}

fn connected_is_invagination_proxy(grid: &Grid, outside: usize, phi: &[f64]) -> bool {
    let i = outside % grid.width;
    let j = outside / grid.width;
    let mut interior_n = 0;
    for &(di, dj) in &[(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
        let ni = i as isize + di;
        let nj = j as isize + dj;
        if ni < 0 || nj < 0 || ni >= grid.width as isize || nj >= grid.height as isize {
            continue;
        }
        let nidx = Grid::index(grid.width, ni as usize, nj as usize);
        if grid.in_dish(nidx) && phi[nidx] >= D063_PHI_INTERIOR {
            interior_n += 1;
        }
    }
    interior_n >= 2
}

pub fn select_route(ev: RouteEvidence064) -> (D064Route, D064PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D064Route::I,
            D064PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d063_reproduced {
        return (
            D064Route::I,
            D064PrimaryConclusion::D063CoupledFailureNotReproduced,
        );
    }
    if !ev.accounting_reconciled {
        return (
            D064Route::I,
            D064PrimaryConclusion::ResourceSufficiencyAccountingFailure,
        );
    }
    if !ev.rejection_provenance_resolved {
        return (
            D064Route::I,
            D064PrimaryConclusion::RejectionProvenanceUnresolved,
        );
    }
    if !ev.aps_ledger_ok {
        return (
            D064Route::I,
            D064PrimaryConclusion::ConnectedGeometryApsLedgerFailure,
        );
    }
    if ev.seed_material_inconsistent {
        return (
            D064Route::I,
            D064PrimaryConclusion::PrebuiltSeedAccountingFailure,
        );
    }

    // Primary scientific routes (exactly one).
    if ev.static_used_requested_flux {
        return (D064Route::A, D064Route::A.conclusion());
    }
    if ev.multiface_budget_defect && ev.joint_allocator_rescues {
        return (D064Route::B, D064Route::B.conclusion());
    }
    if ev.geometry_discretization_defect {
        return (D064Route::G, D064Route::G.conclusion());
    }
    if ev.authoritative_pass || ev.short_screen_pass {
        return (D064Route::Q, D064Route::Q.conclusion());
    }
    if ev.seed_nonequilibrium {
        return (D064Route::S, D064Route::S.conclusion());
    }
    if ev.exchange_load_dominant {
        return (D064Route::E, D064Route::E.conclusion());
    }
    if ev.precursor_demand_dominant {
        return (D064Route::P, D064Route::P.conclusion());
    }
    if ev.upper_bound_still_collapses {
        return (D064Route::N, D064Route::N.conclusion());
    }
    if ev.upper_bound_restores_aps {
        // Delivery residual after geometry/seed corrections — still not primary repair of A/P/S
        // without further delivery work; map to inconclusive rather than inventing a route.
        return (D064Route::I, D064Route::I.conclusion());
    }
    (D064Route::I, D064Route::I.conclusion())
}

pub fn shadow_isolation_ok(production_carrier: bool, v15: bool, morphogenesis: bool) -> bool {
    !production_carrier && !v15 && !morphogenesis
}

pub fn family_label(f: GeometryFamily) -> &'static str {
    f.as_str()
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn chi_accepted_only() {
        let w = ResourceSufficiencyWindow {
            j_n_passive_accepted: 1.0,
            j_n_carrier_accepted: 2.0,
            j_f_passive_accepted: 1.0,
            j_f_carrier_accepted: 2.0,
            l_n_required: 2.0,
            l_f_required: 2.0,
            accepted_steps: 10,
            window_time: 0.05,
        };
        assert!((w.chi_n() - 1.5).abs() < 1e-12);
        assert!((legacy_analytical_requested_capacity(100.0, 0.005)
            - D064_FROZEN_KT * 0.35 * 100.0 * 0.005)
            .abs()
            < 1e-12);
    }

    #[test]
    fn joint_allocator_caps_overcommit() {
        let reqs = vec![
            CarrierFaceRequest {
                inside: 0,
                outside: 1,
                face_id: 0,
                xi_req: 2.0,
                topology: MembraneFaceClass::ExternalBoundary,
            },
            CarrierFaceRequest {
                inside: 0,
                outside: 1,
                face_id: 1,
                xi_req: 2.0,
                topology: MembraneFaceClass::ExteriorConnectedInvagination,
            },
        ];
        let n = vec![0.0, 1.0];
        let f = vec![0.0, 1.0];
        let w = vec![10.0, 0.0];
        let scaled = joint_allocate_faces(&reqs, &n, &f, &w);
        // Exterior cell 1 has budget 1 for N and F; two faces request 0.5*2 + 0.5*2 = 2 each → λ=0.5
        assert!((scaled[0] - 1.0).abs() < 1e-9);
        assert!((scaled[1] - 1.0).abs() < 1e-9);
        let audit = cell_budget_audit(&reqs, &n, &f, &w, &[0.0; 2], &[0.0; 2]);
        assert!(audit.multiface_defect);
        assert!(audit.max_omega_n > 1.0);
    }
}
