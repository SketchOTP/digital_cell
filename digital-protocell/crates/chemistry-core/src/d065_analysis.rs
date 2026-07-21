//! D-065 canonical net accepted-flux resource sufficiency and topology-necessity audit.
//! Shadow/observer diagnostics only — no production carrier, V15, or morphogenesis.

use crate::d064_analysis::{
    chi_ratio, legacy_analytical_requested_capacity, legacy_d063_chi_proxy, productive_demand,
    D064_CHI_VIABLE, D064_FROZEN_KT, D064_PRODUCTIVE_DEMAND_DENSITY,
};
use serde::{Deserialize, Serialize};

pub const D065_PROJECT_ID: &str = "D-065";
pub const D065_AGENT_MEMORY_ID: &str =
    "D-20260721-d065-canonical-resource-sufficiency-topology-necessity";
pub const D065_STARTING_COMMIT: &str = "4260a64";
pub const D065_STARTING_TAG: &str = "D-064-connected-geometry-coupled-failure-audit";
pub const D065_D064_CONCLUSION: &str = "D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT";
pub const D065_D064_RECORD: &str =
    "CONNECTED_GEOMETRY_STATIC_CAPACITY_QUALIFIED_COUPLED_CAUSE_UNRESOLVED";
pub const D065_D063_RANKING_INVALIDATED: &str =
    "D063_TOPOLOGY_RANKING_INVALIDATED_BY_RESOURCE_METRIC_DEFECT";
pub const D065_FROZEN_KT: f64 = D064_FROZEN_KT;
pub const D065_CHI_VIABLE: f64 = D064_CHI_VIABLE;
pub const D065_A_RETENTION_TARGET: f64 = 0.80;
pub const D065_PRODUCTIVE_DEMAND_DENSITY: f64 = D064_PRODUCTIVE_DEMAND_DENSITY;
pub const D065_EPS: f64 = 1e-18;
pub const D065_LEDGER_TOL: f64 = 1e-4;
pub const D065_OVERSUPPLY_FACTOR: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D065PrimaryConclusion {
    SmoothMembraneResourceCapacitySufficient,
    ConnectedMembraneResourceCapacityRequired,
    ResourceDeliverySufficientActivationLimited,
    ResourceDeliverySufficientADemandLimited,
    WasteExportExecutionDefect,
    ConnectedMembraneNotPrimaryRepair,
    CanonicalResourceRequalificationInconclusive,
    D064MetricDefectNotReproduced,
    CanonicalResourceEvaluatorFailure,
    ResourceEvaluatorParityFailure,
    ResourceFateAccountingFailure,
    WasteRejectionProvenanceFailure,
    ALedgerFailure,
    WorkspaceScopeNotIsolated,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D065PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SmoothMembraneResourceCapacitySufficient => {
                "D065_SMOOTH_MEMBRANE_RESOURCE_CAPACITY_SUFFICIENT"
            }
            Self::ConnectedMembraneResourceCapacityRequired => {
                "D065_CONNECTED_MEMBRANE_RESOURCE_CAPACITY_REQUIRED"
            }
            Self::ResourceDeliverySufficientActivationLimited => {
                "D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED"
            }
            Self::ResourceDeliverySufficientADemandLimited => {
                "D065_RESOURCE_DELIVERY_SUFFICIENT_A_DEMAND_LIMITED"
            }
            Self::WasteExportExecutionDefect => "D065_WASTE_EXPORT_EXECUTION_DEFECT",
            Self::ConnectedMembraneNotPrimaryRepair => {
                "D065_CONNECTED_MEMBRANE_NOT_PRIMARY_REPAIR"
            }
            Self::CanonicalResourceRequalificationInconclusive => {
                "D065_CANONICAL_RESOURCE_REQUALIFICATION_INCONCLUSIVE"
            }
            Self::D064MetricDefectNotReproduced => "D065_D064_METRIC_DEFECT_NOT_REPRODUCED",
            Self::CanonicalResourceEvaluatorFailure => {
                "D065_CANONICAL_RESOURCE_EVALUATOR_FAILURE"
            }
            Self::ResourceEvaluatorParityFailure => "D065_RESOURCE_EVALUATOR_PARITY_FAILURE",
            Self::ResourceFateAccountingFailure => "D065_RESOURCE_FATE_ACCOUNTING_FAILURE",
            Self::WasteRejectionProvenanceFailure => "D065_WASTE_REJECTION_PROVENANCE_FAILURE",
            Self::ALedgerFailure => "D065_A_LEDGER_FAILURE",
            Self::WorkspaceScopeNotIsolated => "D065_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::AccountingFailure => "D065_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D065_NUMERICAL_FAILURE",
            Self::Fail => "D065_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D065Route {
    S,
    C,
    A,
    D,
    W,
    U,
    M,
}

impl D065Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::S => "Route_S_smooth_membrane_resource_capacity_sufficient",
            Self::C => "Route_C_connected_membrane_resource_capacity_required",
            Self::A => "Route_A_resource_delivery_sufficient_activation_limited",
            Self::D => "Route_D_resource_delivery_sufficient_a_demand_limited",
            Self::W => "Route_W_waste_export_execution_defect",
            Self::U => "Route_U_connected_membrane_not_primary_repair",
            Self::M => "Route_M_canonical_resource_requalification_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D065PrimaryConclusion {
        match self {
            Self::S => D065PrimaryConclusion::SmoothMembraneResourceCapacitySufficient,
            Self::C => D065PrimaryConclusion::ConnectedMembraneResourceCapacityRequired,
            Self::A => D065PrimaryConclusion::ResourceDeliverySufficientActivationLimited,
            Self::D => D065PrimaryConclusion::ResourceDeliverySufficientADemandLimited,
            Self::W => D065PrimaryConclusion::WasteExportExecutionDefect,
            Self::U => D065PrimaryConclusion::ConnectedMembraneNotPrimaryRepair,
            Self::M => D065PrimaryConclusion::CanonicalResourceRequalificationInconclusive,
        }
    }
}

/// One accepted environmental transport event across reservoir-connected exterior → interior.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AcceptedEnvFluxEvent {
    pub resource_is_n: bool,
    pub amount_signed: f64,
    /// +1 = into interior from exterior-connected cell; −1 = out to exterior-connected cell.
    pub direction_into_interior: f64,
    pub is_carrier: bool,
    pub is_passive: bool,
    pub exterior_connected: bool,
    pub closed_vesicle: bool,
    pub step_accepted: bool,
}

/// Canonical signed net environmental flux window — sole authorized χ source.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CanonicalNetFluxWindow {
    pub j_n_passive_net: f64,
    pub j_n_carrier_net: f64,
    pub j_f_passive_net: f64,
    pub j_f_carrier_net: f64,
    pub j_n_in_accepted: f64,
    pub j_n_out_accepted: f64,
    pub j_f_in_accepted: f64,
    pub j_f_out_accepted: f64,
    pub j_n_rejected_excluded: f64,
    pub j_f_rejected_excluded: f64,
    pub j_n_closed_vesicle_excluded: f64,
    pub j_f_closed_vesicle_excluded: f64,
    pub j_n_recirculation_excluded: f64,
    pub j_f_recirculation_excluded: f64,
    pub l_n_required: f64,
    pub l_f_required: f64,
    pub accepted_steps: u64,
    pub window_time: f64,
    pub interior_area: f64,
}

impl CanonicalNetFluxWindow {
    pub fn empty(interior_area: f64, window_time: f64, accepted_steps: u64) -> Self {
        let demand = productive_demand(interior_area, window_time);
        Self {
            j_n_passive_net: 0.0,
            j_n_carrier_net: 0.0,
            j_f_passive_net: 0.0,
            j_f_carrier_net: 0.0,
            j_n_in_accepted: 0.0,
            j_n_out_accepted: 0.0,
            j_f_in_accepted: 0.0,
            j_f_out_accepted: 0.0,
            j_n_rejected_excluded: 0.0,
            j_f_rejected_excluded: 0.0,
            j_n_closed_vesicle_excluded: 0.0,
            j_f_closed_vesicle_excluded: 0.0,
            j_n_recirculation_excluded: 0.0,
            j_f_recirculation_excluded: 0.0,
            l_n_required: demand,
            l_f_required: demand,
            accepted_steps,
            window_time,
            interior_area,
        }
    }

    pub fn j_n_net(self) -> f64 {
        self.j_n_passive_net + self.j_n_carrier_net
    }

    pub fn j_f_net(self) -> f64 {
        self.j_f_passive_net + self.j_f_carrier_net
    }

    pub fn chi_n(self) -> f64 {
        chi_ratio(self.j_n_net(), self.l_n_required)
    }

    pub fn chi_f(self) -> f64 {
        chi_ratio(self.j_f_net(), self.l_f_required)
    }

    pub fn chi_min(self) -> f64 {
        self.chi_n().min(self.chi_f())
    }
}

/// Accumulate signed net environmental flux from typed events.
/// Rules: accepted steps only; exterior-connected only; exclude closed vesicles;
/// signed net = in − out; recirculation (equal in/out pairs) nets to zero naturally.
pub fn evaluate_canonical_net_flux(
    events: &[AcceptedEnvFluxEvent],
    interior_area: f64,
    window_time: f64,
    accepted_steps: u64,
) -> CanonicalNetFluxWindow {
    let mut w = CanonicalNetFluxWindow::empty(interior_area, window_time, accepted_steps);
    for e in events {
        let amt = e.amount_signed.abs();
        if amt <= D065_EPS {
            continue;
        }
        if !e.step_accepted {
            if e.resource_is_n {
                w.j_n_rejected_excluded += amt;
            } else {
                w.j_f_rejected_excluded += amt;
            }
            continue;
        }
        if e.closed_vesicle || !e.exterior_connected {
            if e.resource_is_n {
                w.j_n_closed_vesicle_excluded += amt;
            } else {
                w.j_f_closed_vesicle_excluded += amt;
            }
            continue;
        }
        // Non-environmental internal interface recirculation marker:
        // exterior_connected=false already excluded; explicit recirculation flag via
        // direction 0 is treated as excluded recirculation.
        if e.direction_into_interior.abs() < D065_EPS {
            if e.resource_is_n {
                w.j_n_recirculation_excluded += amt;
            } else {
                w.j_f_recirculation_excluded += amt;
            }
            continue;
        }
        let signed = amt * e.direction_into_interior.signum();
        let is_in = signed > 0.0;
        if e.resource_is_n {
            if is_in {
                w.j_n_in_accepted += amt;
            } else {
                w.j_n_out_accepted += amt;
            }
            if e.is_carrier {
                w.j_n_carrier_net += signed;
            } else if e.is_passive {
                w.j_n_passive_net += signed;
            }
        } else {
            if is_in {
                w.j_f_in_accepted += amt;
            } else {
                w.j_f_out_accepted += amt;
            }
            if e.is_carrier {
                w.j_f_carrier_net += signed;
            } else if e.is_passive {
                w.j_f_passive_net += signed;
            }
        }
    }
    // Recompute demand from identical definition.
    let demand = productive_demand(interior_area, window_time);
    w.l_n_required = demand;
    w.l_f_required = demand;
    w
}

/// Build a window from already-aggregated signed nets (static/coupled parity helper).
pub fn window_from_signed_nets(
    j_n_passive_net: f64,
    j_n_carrier_net: f64,
    j_f_passive_net: f64,
    j_f_carrier_net: f64,
    interior_area: f64,
    window_time: f64,
    accepted_steps: u64,
) -> CanonicalNetFluxWindow {
    let mut w = CanonicalNetFluxWindow::empty(interior_area, window_time, accepted_steps);
    w.j_n_passive_net = j_n_passive_net;
    w.j_n_carrier_net = j_n_carrier_net;
    w.j_f_passive_net = j_f_passive_net;
    w.j_f_carrier_net = j_f_carrier_net;
    w.j_n_in_accepted = j_n_passive_net.max(0.0) + j_n_carrier_net.max(0.0);
    w.j_n_out_accepted = (-j_n_passive_net).max(0.0) + (-j_n_carrier_net).max(0.0);
    w.j_f_in_accepted = j_f_passive_net.max(0.0) + j_f_carrier_net.max(0.0);
    w.j_f_out_accepted = (-j_f_passive_net).max(0.0) + (-j_f_carrier_net).max(0.0);
    w
}

pub fn static_coupled_parity(static_w: CanonicalNetFluxWindow, coupled_w: CanonicalNetFluxWindow) -> bool {
    // Equivalent accepted events must yield identical χ under identical demand dens × area × time.
    let dens_ok = (static_w.l_n_required
        - D065_PRODUCTIVE_DEMAND_DENSITY * static_w.interior_area * static_w.window_time)
        .abs()
        <= 1e-9 * (1.0 + static_w.l_n_required.abs())
        && (coupled_w.l_n_required
            - D065_PRODUCTIVE_DEMAND_DENSITY * coupled_w.interior_area * coupled_w.window_time)
            .abs()
            <= 1e-9 * (1.0 + coupled_w.l_n_required.abs());
    dens_ok
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TopologyNecessityClass {
    SmoothSufficient,
    ConnectedAreaNecessary,
    ConnectedAreaInsufficient,
    ResourceOversupply,
}

impl TopologyNecessityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SmoothSufficient => "SMOOTH_SUFFICIENT",
            Self::ConnectedAreaNecessary => "CONNECTED_AREA_NECESSARY",
            Self::ConnectedAreaInsufficient => "CONNECTED_AREA_INSUFFICIENT",
            Self::ResourceOversupply => "RESOURCE_OVERSUPPLY",
        }
    }
}

pub fn delta_chi_topology(chi_connected: f64, chi_smooth: f64) -> f64 {
    chi_connected - chi_smooth
}

pub fn classify_topology_necessity(chi_smooth: f64, chi_connected: f64) -> TopologyNecessityClass {
    if chi_smooth >= D065_CHI_VIABLE * D065_OVERSUPPLY_FACTOR {
        return TopologyNecessityClass::ResourceOversupply;
    }
    if chi_smooth >= D065_CHI_VIABLE {
        return TopologyNecessityClass::SmoothSufficient;
    }
    if chi_connected >= D065_CHI_VIABLE {
        return TopologyNecessityClass::ConnectedAreaNecessary;
    }
    TopologyNecessityClass::ConnectedAreaInsufficient
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceFateClass {
    ActivationConsumed,
    InternalAccumulation,
    RapidReexport,
    ReverseCarrierRecirculation,
    ResourceLedgerUnresolved,
}

impl ResourceFateClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationConsumed => "ACTIVATION_CONSUMED",
            Self::InternalAccumulation => "INTERNAL_ACCUMULATION",
            Self::RapidReexport => "RAPID_REEXPORT",
            Self::ReverseCarrierRecirculation => "REVERSE_CARRIER_RECIRCULATION",
            Self::ResourceLedgerUnresolved => "RESOURCE_LEDGER_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ResourceFateLedger {
    pub j_net: f64,
    pub u_activation: f64,
    pub u_other: f64,
    pub delta_inventory: f64,
    pub reexport: f64,
    pub reverse_carrier: f64,
    pub numerical_correction: f64,
    pub rejected_excluded: f64,
}

impl ResourceFateLedger {
    pub fn residual(self) -> f64 {
        // ΔM = J_net − U_act − U_other  (inventory change should match)
        self.j_net - self.u_activation - self.u_other - self.delta_inventory
            - self.reexport
            - self.reverse_carrier
            - self.numerical_correction
    }

    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs() <= tol.max(D065_LEDGER_TOL) * (1.0 + self.j_net.abs())
    }
}

pub fn classify_resource_fate(ledger: ResourceFateLedger, closes: bool) -> ResourceFateClass {
    if !closes {
        return ResourceFateClass::ResourceLedgerUnresolved;
    }
    let scale = ledger.j_net.abs().max(1e-9);
    if ledger.reverse_carrier.abs() / scale > 0.5 {
        return ResourceFateClass::ReverseCarrierRecirculation;
    }
    if ledger.reexport / scale > 0.5 {
        return ResourceFateClass::RapidReexport;
    }
    if ledger.u_activation / scale > 0.4 {
        return ResourceFateClass::ActivationConsumed;
    }
    if ledger.delta_inventory.abs() / scale > 0.4 {
        return ResourceFateClass::InternalAccumulation;
    }
    if ledger.u_activation > D065_EPS {
        ResourceFateClass::ActivationConsumed
    } else {
        ResourceFateClass::InternalAccumulation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WasteRejectionClass {
    WDestinationOvercommit,
    WExternalDispersalLimit,
    WCeilingPolicyDefect,
    WExportSignDefect,
    LegitimateWAccumulation,
    WRejectionUnresolved,
}

impl WasteRejectionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WDestinationOvercommit => "W_DESTINATION_OVERCOMMIT",
            Self::WExternalDispersalLimit => "W_EXTERNAL_DISPERSAL_LIMIT",
            Self::WCeilingPolicyDefect => "W_CEILING_POLICY_DEFECT",
            Self::WExportSignDefect => "W_EXPORT_SIGN_DEFECT",
            Self::LegitimateWAccumulation => "LEGITIMATE_W_ACCUMULATION",
            Self::WRejectionUnresolved => "W_REJECTION_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WasteAuditEvidence {
    pub multiface_overcommit: bool,
    pub perfect_sink_removes_rejection: bool,
    pub carrier_disabled_removes_rejection: bool,
    pub reduced_dt_removes_rejection: bool,
    pub export_sign_inverted: bool,
    pub exterior_w_rises_faster_than_dispersal: bool,
    pub smooth_also_hits_ceiling: bool,
    pub rejection_observed: bool,
}

pub fn classify_waste_rejection(ev: WasteAuditEvidence) -> WasteRejectionClass {
    if !ev.rejection_observed {
        return WasteRejectionClass::WRejectionUnresolved;
    }
    if ev.export_sign_inverted {
        return WasteRejectionClass::WExportSignDefect;
    }
    if ev.multiface_overcommit && !ev.perfect_sink_removes_rejection {
        // Overcommit present; sink may still help — prefer destination overcommit when ω>1.
        return WasteRejectionClass::WDestinationOvercommit;
    }
    if ev.perfect_sink_removes_rejection && ev.exterior_w_rises_faster_than_dispersal {
        return WasteRejectionClass::WExternalDispersalLimit;
    }
    if ev.perfect_sink_removes_rejection && !ev.carrier_disabled_removes_rejection {
        return WasteRejectionClass::WCeilingPolicyDefect;
    }
    if ev.carrier_disabled_removes_rejection && ev.multiface_overcommit {
        return WasteRejectionClass::WDestinationOvercommit;
    }
    if !ev.perfect_sink_removes_rejection && !ev.reduced_dt_removes_rejection {
        return WasteRejectionClass::LegitimateWAccumulation;
    }
    if ev.multiface_overcommit {
        return WasteRejectionClass::WDestinationOvercommit;
    }
    WasteRejectionClass::WRejectionUnresolved
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ABalanceClass {
    ResourceDeliveryNotUsedByActivation,
    ActivationCapacityLimit,
    ActivationYieldLimit,
    AProductiveDemandExceedsProduction,
    APassiveLossLimit,
    MultipleABalanceLimits,
    ABalanceUnresolved,
}

impl ABalanceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceDeliveryNotUsedByActivation => {
                "RESOURCE_DELIVERY_NOT_USED_BY_ACTIVATION"
            }
            Self::ActivationCapacityLimit => "ACTIVATION_CAPACITY_LIMIT",
            Self::ActivationYieldLimit => "ACTIVATION_YIELD_LIMIT",
            Self::AProductiveDemandExceedsProduction => {
                "A_PRODUCTIVE_DEMAND_EXCEEDS_PRODUCTION"
            }
            Self::APassiveLossLimit => "A_PASSIVE_LOSS_LIMIT",
            Self::MultipleABalanceLimits => "MULTIPLE_A_BALANCE_LIMITS",
            Self::ABalanceUnresolved => "A_BALANCE_UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ALedger {
    pub g_activation: f64,
    pub l_catalyst: f64,
    pub l_structure: f64,
    pub l_precursor: f64,
    pub l_decay: f64,
    pub j_out: f64,
    pub j_in: f64,
    pub delta_a: f64,
    pub activation_requested: f64,
    pub activation_accepted: f64,
    pub j_n_net: f64,
    pub j_f_net: f64,
}

impl ALedger {
    pub fn residual(self) -> f64 {
        // ΔA = G − L_cat − L_str − L_pre − L_decay − J_out + J_in
        self.g_activation
            - self.l_catalyst
            - self.l_structure
            - self.l_precursor
            - self.l_decay
            - self.j_out
            + self.j_in
            - self.delta_a
    }

    pub fn closes(self, tol: f64) -> bool {
        self.residual().abs() <= tol.max(D065_LEDGER_TOL) * (1.0 + self.g_activation.abs().max(self.delta_a.abs()))
    }

    pub fn eta_delivery_to_a(self) -> f64 {
        let denom = self.j_n_net.min(self.j_f_net);
        if denom <= D065_EPS {
            return 0.0;
        }
        self.g_activation / denom
    }

    pub fn total_demand(self) -> f64 {
        self.l_catalyst + self.l_structure + self.l_precursor + self.l_decay + self.j_out
    }

    pub fn dominant_sink(self) -> &'static str {
        let terms = [
            ("catalyst", self.l_catalyst),
            ("structure", self.l_structure),
            ("precursor", self.l_precursor),
            ("decay", self.l_decay),
            ("transport_out", self.j_out),
        ];
        terms
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _)| *n)
            .unwrap_or("none")
    }
}

pub fn classify_a_balance(ledger: ALedger, closes: bool, a_ret: f64) -> ABalanceClass {
    if !closes {
        return ABalanceClass::ABalanceUnresolved;
    }
    let delivery = ledger.j_n_net.min(ledger.j_f_net);
    let mut hits = 0usize;
    let mut primary = ABalanceClass::ABalanceUnresolved;

    if delivery > D065_EPS && ledger.g_activation / delivery.max(D065_EPS) < 0.05 {
        hits += 1;
        primary = ABalanceClass::ResourceDeliveryNotUsedByActivation;
    }
    if ledger.activation_requested > D065_EPS
        && ledger.activation_accepted < 0.5 * ledger.activation_requested
    {
        hits += 1;
        primary = ABalanceClass::ActivationCapacityLimit;
    }
    if ledger.activation_accepted > D065_EPS
        && ledger.g_activation < 0.5 * ledger.activation_accepted
    {
        hits += 1;
        primary = ABalanceClass::ActivationYieldLimit;
    }
    if ledger.g_activation > D065_EPS && ledger.total_demand() > ledger.g_activation + ledger.j_in {
        hits += 1;
        primary = ABalanceClass::AProductiveDemandExceedsProduction;
    }
    if ledger.j_out > 0.4 * ledger.g_activation.max(D065_EPS) && a_ret < D065_A_RETENTION_TARGET {
        hits += 1;
        primary = ABalanceClass::APassiveLossLimit;
    }

    if hits >= 2 {
        ABalanceClass::MultipleABalanceLimits
    } else if hits == 1 {
        primary
    } else if a_ret < D065_A_RETENTION_TARGET && ledger.total_demand() > ledger.g_activation {
        ABalanceClass::AProductiveDemandExceedsProduction
    } else {
        ABalanceClass::ABalanceUnresolved
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidence065 {
    pub workspace_isolated: bool,
    pub d064_reproduced: bool,
    pub evaluator_ok: bool,
    pub parity_ok: bool,
    pub fate_ledger_ok: bool,
    pub waste_provenance_ok: bool,
    pub a_ledger_ok: bool,
    pub chi_smooth_min: f64,
    pub chi_connected_best: f64,
    pub connected_improves_a: bool,
    pub a_retention: f64,
    pub activation_limited: bool,
    pub a_demand_limited: bool,
    pub waste_execution_defect: bool,
    pub closed_vesicle_chi_near_zero: bool,
}

pub fn select_route(ev: RouteEvidence065) -> (D065Route, D065PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D065Route::M,
            D065PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d064_reproduced {
        return (
            D065Route::M,
            D065PrimaryConclusion::D064MetricDefectNotReproduced,
        );
    }
    if !ev.evaluator_ok {
        return (
            D065Route::M,
            D065PrimaryConclusion::CanonicalResourceEvaluatorFailure,
        );
    }
    if !ev.parity_ok {
        return (
            D065Route::M,
            D065PrimaryConclusion::ResourceEvaluatorParityFailure,
        );
    }
    if !ev.fate_ledger_ok {
        return (
            D065Route::M,
            D065PrimaryConclusion::ResourceFateAccountingFailure,
        );
    }
    if !ev.waste_provenance_ok {
        return (
            D065Route::M,
            D065PrimaryConclusion::WasteRejectionProvenanceFailure,
        );
    }
    if !ev.a_ledger_ok {
        return (D065Route::M, D065PrimaryConclusion::ALedgerFailure);
    }

    // Primary scientific routes — exactly one.
    // Prefer metabolic bottleneck when smooth already supplies N/F.
    if ev.chi_smooth_min >= D065_CHI_VIABLE {
        if ev.waste_execution_defect
            && ev.a_retention < D065_A_RETENTION_TARGET
            && !ev.activation_limited
            && !ev.a_demand_limited
        {
            // W defect can invalidate trajectories even with delivery; only pick W when
            // it is the dominant unresolved coupled failure without clear A classification.
            return (D065Route::W, D065Route::W.conclusion());
        }
        if ev.activation_limited && ev.a_retention < D065_A_RETENTION_TARGET {
            return (D065Route::A, D065Route::A.conclusion());
        }
        if ev.a_demand_limited && ev.a_retention < D065_A_RETENTION_TARGET {
            return (D065Route::D, D065Route::D.conclusion());
        }
        // Smooth sufficient for resource capacity — close connected-area capacity branch.
        return (D065Route::S, D065Route::S.conclusion());
    }

    // Smooth insufficient.
    if ev.chi_connected_best >= D065_CHI_VIABLE
        && ev.connected_improves_a
        && ev.a_retention >= D065_A_RETENTION_TARGET
        && ev.closed_vesicle_chi_near_zero
    {
        return (D065Route::C, D065Route::C.conclusion());
    }
    if ev.chi_connected_best > ev.chi_smooth_min + 0.05 && !ev.connected_improves_a {
        return (D065Route::U, D065Route::U.conclusion());
    }
    if ev.waste_execution_defect {
        return (D065Route::W, D065Route::W.conclusion());
    }
    (D065Route::M, D065Route::M.conclusion())
}

/// D-064 metric-defect reproduction predicate (frozen numbers + identities).
pub fn d064_metric_defect_reproduced(
    legacy_static_chi: f64,
    legacy_coupled_proxy: f64,
    canonical_chi: f64,
    a_ret: f64,
    s_declined: bool,
    w_ceiling_reject: bool,
    multiface_overcommit: bool,
) -> bool {
    legacy_static_chi > 1.05
        && legacy_coupled_proxy < 0.5
        && canonical_chi >= D065_CHI_VIABLE
        && a_ret < 0.6
        && s_declined
        && w_ceiling_reject
        && multiface_overcommit
}

pub fn connected_membrane_not_required(chi_smooth: f64) -> bool {
    chi_smooth >= D065_CHI_VIABLE
}

pub fn connected_area_delivery_not_causally_useful(
    chi_connected: f64,
    chi_smooth: f64,
    a_improved: bool,
) -> bool {
    chi_connected > chi_smooth + 0.05 && !a_improved
}

pub fn legacy_metrics_unauthorized_for_ranking() -> bool {
    true
}

pub fn shadow_isolation_ok(production_carrier: bool, v15: bool, morphogenesis: bool) -> bool {
    !production_carrier && !v15 && !morphogenesis
}

/// Reconstruct legacy static χ (requested analytical) for reproduction only.
pub fn legacy_static_chi(connected_length: f64, interior_area: f64, dt: f64) -> f64 {
    let supply = legacy_analytical_requested_capacity(connected_length, dt);
    chi_ratio(supply, productive_demand(interior_area, dt))
}

pub fn legacy_coupled_proxy_chi(import: f64, interior_area: f64, accepted: u64) -> f64 {
    legacy_d063_chi_proxy(import, interior_area, accepted)
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn signed_net_inward_positive() {
        let events = vec![
            AcceptedEnvFluxEvent {
                resource_is_n: true,
                amount_signed: 2.0,
                direction_into_interior: 1.0,
                is_carrier: true,
                is_passive: false,
                exterior_connected: true,
                closed_vesicle: false,
                step_accepted: true,
            },
            AcceptedEnvFluxEvent {
                resource_is_n: false,
                amount_signed: 2.0,
                direction_into_interior: 1.0,
                is_carrier: true,
                is_passive: false,
                exterior_connected: true,
                closed_vesicle: false,
                step_accepted: true,
            },
        ];
        let w = evaluate_canonical_net_flux(&events, 100.0, 1.0, 1);
        assert!(w.chi_n() > 0.0);
        assert!((w.j_n_net() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn recirculation_nets_zero() {
        let events = vec![
            AcceptedEnvFluxEvent {
                resource_is_n: true,
                amount_signed: 3.0,
                direction_into_interior: 1.0,
                is_carrier: true,
                is_passive: false,
                exterior_connected: true,
                closed_vesicle: false,
                step_accepted: true,
            },
            AcceptedEnvFluxEvent {
                resource_is_n: true,
                amount_signed: 3.0,
                direction_into_interior: -1.0,
                is_carrier: true,
                is_passive: false,
                exterior_connected: true,
                closed_vesicle: false,
                step_accepted: true,
            },
        ];
        let w = evaluate_canonical_net_flux(&events, 100.0, 1.0, 2);
        assert!(w.j_n_net().abs() < 1e-12);
        assert_eq!(w.chi_n(), 0.0);
    }
}
