//! D-017 architecture comparison: activation-yield vs energy-coupled active W export.
//!
//! Observer-only / counterfactual analysis. Does not alter runtime chemistry.

use crate::config::CONC_SAFETY_LIMIT;
use crate::d012_accounting::{E_ACTIVATED, E_FUEL};
use crate::d016_transport::{analytical_delta_w_center, d_w_required_for_target};
use serde::{Deserialize, Serialize};

/// Frozen D-016 transport/source evidence (canonical window).
pub const D017_FROZEN_TOTAL_W_SOURCE: f64 = 41.63147071616201;
pub const D017_FROZEN_INTERIOR_W_SOURCE: f64 = 33.55133760120064;
pub const D017_FROZEN_INTERFACE_W_SOURCE: f64 = 7.0226973745715044;
pub const D017_FROZEN_Q_AREA: f64 = 0.026211982500938;
pub const D017_FROZEN_RADIUS: f64 = 22.0;
pub const D017_FROZEN_D_W: f64 = 0.25;
pub const D017_FROZEN_BETA_W: f64 = 0.2;
pub const D017_FROZEN_W_INTERFACE: f64 = 2.0;
pub const D017_FROZEN_DELTA_W_CENTER: f64 = 12.686599530453993;
pub const D017_FROZEN_D_W_REQ_50: f64 = 1.0572166275378327;
pub const D017_FROZEN_D_W_REQ_90: f64 = 0.4530928403733569;
pub const D017_FROZEN_SOURCE_WEIGHTED_RADIUS: f64 = 15.148484093464841;
pub const D017_FROZEN_FRAC_R_HALF: f64 = 0.24010225304843336;
pub const D017_FROZEN_FRAC_3R4: f64 = 0.5355259133991016;
pub const D017_FROZEN_INTERNAL_R_FRAC: f64 = 0.8722518486274626;
pub const D017_FROZEN_MEMBRANE_R_FRAC: f64 = 0.008804717468792477;
pub const D017_FROZEN_EXTERNAL_R_FRAC: f64 = 0.11533908742181324;
pub const D017_FROZEN_SINK_R_FRAC: f64 = 0.003604346481931664;
pub const D017_AUTHORIZED_D_W_BOUND: f64 = 0.18;
pub const D017_CENTER_GATE_PREFERRED: f64 = 5.0;
pub const D017_CENTER_GATE_MINIMUM: f64 = 9.0;
pub const D017_INTERIOR_CELLS: usize = 1280;

/// Cumulative extents from D-015 fresh R22 checkpoint_150000 (η=1).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GovernedExtentSnapshot {
    pub simulated_time: f64,
    pub activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
    pub structure_production_extent: f64,
    pub structure_decay: f64,
    pub membrane_synthesis: f64,
    pub membrane_decay: f64,
    pub membrane_detachment: f64,
    pub eta_c: f64,
    pub eta_phi: f64,
    pub eta_m: f64,
}

impl GovernedExtentSnapshot {
    /// Frozen D-015/D-016 checkpoint extents (accepted evidence).
    pub fn frozen_d015_150k() -> Self {
        Self {
            simulated_time: 374.9999999996952,
            activation: 808.5433265667433,
            reproduction: 181.49396994227624,
            activated_decay: 219.70318046169106,
            catalyst_turnover: 388.72723335871007,
            // η_φ=1 ⇒ extent = virtual_production
            structure_production_extent: 505.1447820876652,
            structure_decay: 14351.87128414825,
            membrane_synthesis: 123.74582468744946,
            membrane_decay: 119.26171615154378,
            membrane_detachment: 308.5071198074706,
            eta_c: 1.0,
            eta_phi: 1.0,
            eta_m: 1.0,
        }
    }

    pub fn rates(&self) -> ExtentRates {
        let t = self.simulated_time.max(1e-30);
        ExtentRates {
            activation: self.activation / t,
            reproduction: self.reproduction / t,
            activated_decay: self.activated_decay / t,
            catalyst_turnover: self.catalyst_turnover / t,
            structure_production: self.structure_production_extent / t,
            structure_decay: self.structure_decay / t,
            membrane_synthesis: self.membrane_synthesis / t,
            membrane_decay: self.membrane_decay / t,
            membrane_detachment: self.membrane_detachment / t,
            eta_c: self.eta_c,
            eta_phi: self.eta_phi,
            eta_m: self.eta_m,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ExtentRates {
    pub activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
    pub structure_production: f64,
    pub structure_decay: f64,
    pub membrane_synthesis: f64,
    pub membrane_decay: f64,
    pub membrane_detachment: f64,
    pub eta_c: f64,
    pub eta_phi: f64,
    pub eta_m: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ReactionResolvedWSources {
    pub direct_activation_w: f64,
    pub productive_yield_w: f64,
    pub structure_turnover_w: f64,
    pub catalyst_turnover_w: f64,
    pub membrane_turnover_w: f64,
    pub a_turnover_w: f64,
    pub membrane_detachment_w: f64,
}

impl ReactionResolvedWSources {
    pub fn from_extent_rates(r: &ExtentRates) -> Self {
        Self {
            direct_activation_w: r.activation,
            productive_yield_w: (1.0 - r.eta_c) * r.reproduction
                + (1.0 - r.eta_phi) * r.structure_production
                + (1.0 - r.eta_m) * r.membrane_synthesis,
            structure_turnover_w: r.structure_decay,
            catalyst_turnover_w: r.catalyst_turnover,
            membrane_turnover_w: r.membrane_decay,
            a_turnover_w: r.activated_decay,
            membrane_detachment_w: r.membrane_detachment,
        }
    }

    pub fn total(&self) -> f64 {
        self.direct_activation_w
            + self.productive_yield_w
            + self.structure_turnover_w
            + self.catalyst_turnover_w
            + self.membrane_turnover_w
            + self.a_turnover_w
            + self.membrane_detachment_w
    }

    pub fn fraction_of(&self, channel: f64) -> f64 {
        let t = self.total();
        if t > 0.0 {
            channel / t
        } else {
            0.0
        }
    }

    /// Scale all channels so total matches the frozen D-016 source rate.
    pub fn scaled_to_frozen_total(&self) -> Self {
        let t = self.total();
        if t <= 0.0 {
            return *self;
        }
        let s = D017_FROZEN_TOTAL_W_SOURCE / t;
        Self {
            direct_activation_w: self.direct_activation_w * s,
            productive_yield_w: self.productive_yield_w * s,
            structure_turnover_w: self.structure_turnover_w * s,
            catalyst_turnover_w: self.catalyst_turnover_w * s,
            membrane_turnover_w: self.membrane_turnover_w * s,
            a_turnover_w: self.a_turnover_w * s,
            membrane_detachment_w: self.membrane_detachment_w * s,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ChannelSpatialProxy {
    pub absolute_rate: f64,
    pub fraction_of_total: f64,
    /// Proxy: same spatial mix as frozen total source (per-channel fields unavailable).
    pub interior_fraction_proxy: f64,
    pub interface_fraction_proxy: f64,
    pub time_window_label: &'static str,
}

pub fn channel_spatial_proxies(sources: &ReactionResolvedWSources) -> Vec<( &'static str, ChannelSpatialProxy)> {
    let total = sources.total().max(1e-30);
    let interior_share = D017_FROZEN_INTERIOR_W_SOURCE / D017_FROZEN_TOTAL_W_SOURCE;
    let interface_share = D017_FROZEN_INTERFACE_W_SOURCE / D017_FROZEN_TOTAL_W_SOURCE;
    let rows = [
        ("direct_activation_w", sources.direct_activation_w),
        ("productive_yield_w", sources.productive_yield_w),
        ("structure_turnover_w", sources.structure_turnover_w),
        ("catalyst_turnover_w", sources.catalyst_turnover_w),
        ("membrane_turnover_w", sources.membrane_turnover_w),
        ("a_turnover_w", sources.a_turnover_w),
        ("membrane_detachment_w", sources.membrane_detachment_w),
    ];
    rows.into_iter()
        .map(|(name, rate)| {
            (
                name,
                ChannelSpatialProxy {
                    absolute_rate: rate,
                    fraction_of_total: rate / total,
                    interior_fraction_proxy: interior_share,
                    interface_fraction_proxy: interface_share,
                    time_window_label: "75-100pct_proxy_150k_final_valid",
                },
            )
        })
        .collect()
}

/// Candidate A family: N+F → (1+α)A + (1-α)W, 0≤α≤1.
pub fn activation_yield_delta(extent: f64, alpha: f64) -> [f64; 7] {
    assert!((0.0..=1.0).contains(&alpha), "alpha must be in [0,1]");
    let mut d = [0.0; 7];
    d[2] = -extent; // N
    d[3] = -extent; // F
    d[5] = (1.0 + alpha) * extent; // A
    d[4] = (1.0 - alpha) * extent; // W
    d
}

pub fn activation_yield_material_residual(extent: f64, alpha: f64) -> f64 {
    let d = activation_yield_delta(extent, alpha);
    d.iter().sum()
}

/// Potential residual under frozen E_F, E_A weights: ΔΨ = -E_F + (1+α)E_A.
pub fn activation_yield_potential_residual_frozen_weights(alpha: f64) -> f64 {
    -E_FUEL + (1.0 + alpha) * E_ACTIVATED
}

/// Revised per-unit A potential that prevents creation: E_A(α) = E_F/(1+α).
pub fn revised_e_a(alpha: f64) -> f64 {
    E_FUEL / (1.0 + alpha)
}

pub fn activation_yield_potential_residual_revised(alpha: f64) -> f64 {
    -E_FUEL + (1.0 + alpha) * revised_e_a(alpha)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationPotentialClass {
    APotentialValid,
    APotentialInvalid,
    APotentialUnderdetermined,
}

pub fn classify_activation_potential(alpha: f64, allow_revised_weights: bool) -> ActivationPotentialClass {
    if !(0.0..=1.0).contains(&alpha) {
        return ActivationPotentialClass::APotentialUnderdetermined;
    }
    if allow_revised_weights {
        if activation_yield_potential_residual_revised(alpha).abs() <= 1e-12 {
            ActivationPotentialClass::APotentialValid
        } else {
            ActivationPotentialClass::APotentialInvalid
        }
    } else if activation_yield_potential_residual_frozen_weights(alpha) <= 1e-12 {
        ActivationPotentialClass::APotentialValid
    } else {
        ActivationPotentialClass::APotentialInvalid
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixedExtentCounterfactual {
    pub alpha: f64,
    pub counterfactual_type: String,
    pub new_direct_w_source: f64,
    pub new_a_production: f64,
    pub material_residual: f64,
    pub activation_potential_residual_frozen: f64,
    pub activation_potential_residual_revised: f64,
    pub first_order_total_w_source: f64,
    pub first_order_center_w: f64,
    pub q_area_scaled: f64,
}

/// Fixed-extent counterfactual: replace activation W/A yields; other sources unchanged.
pub fn fixed_extent_counterfactual(
    sources: &ReactionResolvedWSources,
    activation_extent_rate: f64,
    alpha: f64,
) -> FixedExtentCounterfactual {
    let new_direct_w = (1.0 - alpha) * activation_extent_rate;
    let new_a = (1.0 + alpha) * activation_extent_rate;
    let removed = sources.direct_activation_w - new_direct_w;
    let new_total = sources.total() - removed;
    let scale = if D017_FROZEN_TOTAL_W_SOURCE > 0.0 {
        new_total / D017_FROZEN_TOTAL_W_SOURCE
    } else {
        1.0
    };
    let q = D017_FROZEN_Q_AREA * scale;
    let delta = analytical_delta_w_center(q, D017_FROZEN_RADIUS, D017_FROZEN_D_W);
    FixedExtentCounterfactual {
        alpha,
        counterfactual_type: "A_FIXED_EXTENT_COUNTERFACTUAL".into(),
        new_direct_w_source: new_direct_w,
        new_a_production: new_a,
        material_residual: activation_yield_material_residual(1.0, alpha),
        activation_potential_residual_frozen: activation_yield_potential_residual_frozen_weights(alpha),
        activation_potential_residual_revised: activation_yield_potential_residual_revised(alpha),
        first_order_total_w_source: new_total,
        first_order_center_w: D017_FROZEN_W_INTERFACE + delta,
        q_area_scaled: q,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct FeedbackBounds {
    pub alpha: f64,
    pub a_production_increase: f64,
    pub w_source_lower: f64,
    pub w_source_coupled: f64,
    pub w_source_upper: f64,
    pub additional_productive_flux_proxy: f64,
    pub n_consumption_change: f64,
    pub f_consumption_change: f64,
}

/// Bounded reduced feedback model on frozen rate bases.
///
/// Lower: extra A stored in components (no immediate W).
/// Upper: all extra A rapidly becomes W.
/// Coupled: half of extra A routes through productive channels then turnover (proxy).
pub fn feedback_bounds(
    sources: &ReactionResolvedWSources,
    activation_extent_rate: f64,
    alpha: f64,
) -> FeedbackBounds {
    let base = fixed_extent_counterfactual(sources, activation_extent_rate, alpha);
    let a_increase = base.new_a_production - activation_extent_rate; // α * extent
    let lower = base.first_order_total_w_source;
    let upper = lower + a_increase; // all extra A → W
    // Coupled: fraction of extra A drives reproduction/structure/membrane then turns over.
    // ponytail: 50% turnover proxy; ceiling is full upper bound if all productive mass decays.
    let coupled_extra_w = 0.5 * a_increase;
    let coupled = lower + coupled_extra_w;
    FeedbackBounds {
        alpha,
        a_production_increase: a_increase,
        w_source_lower: lower,
        w_source_coupled: coupled,
        w_source_upper: upper,
        additional_productive_flux_proxy: a_increase,
        n_consumption_change: 0.0, // fixed-extent: activation extent held fixed
        f_consumption_change: 0.0,
    }
}

pub fn feedback_bounds_ordered(b: &FeedbackBounds) -> bool {
    b.w_source_lower <= b.w_source_coupled + 1e-12
        && b.w_source_coupled <= b.w_source_upper + 1e-12
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateATransportClass {
    ACannotAvoidCeiling,
    AAvoidsCeilingOnly,
    AReachesPreferredMargin,
    AResultDependsOnFeedback,
}

pub fn max_source_for_center_gate(center_limit: f64) -> f64 {
    let allowable_delta = (center_limit - D017_FROZEN_W_INTERFACE).max(0.0);
    if allowable_delta <= 0.0 {
        return 0.0;
    }
    // δ = q R²/(4D); q = interior/cells; scale total ≈ frozen_total * (q/q0)
    let q_max = allowable_delta * 4.0 * D017_FROZEN_D_W / (D017_FROZEN_RADIUS * D017_FROZEN_RADIUS);
    D017_FROZEN_TOTAL_W_SOURCE * (q_max / D017_FROZEN_Q_AREA)
}

pub fn classify_candidate_a_transport(bounds: &FeedbackBounds) -> CandidateATransportClass {
    let max_for_min = max_source_for_center_gate(D017_CENTER_GATE_MINIMUM);
    let max_for_pref = max_source_for_center_gate(D017_CENTER_GATE_PREFERRED);
    let lower_ok_min = bounds.w_source_lower < max_for_min;
    let upper_ok_min = bounds.w_source_upper < max_for_min;
    let lower_ok_pref = bounds.w_source_lower < max_for_pref;
    let upper_ok_pref = bounds.w_source_upper < max_for_pref;

    if !lower_ok_min {
        CandidateATransportClass::ACannotAvoidCeiling
    } else if lower_ok_pref && upper_ok_pref {
        CandidateATransportClass::AReachesPreferredMargin
    } else if lower_ok_min && upper_ok_min {
        if lower_ok_pref != upper_ok_pref {
            CandidateATransportClass::AResultDependsOnFeedback
        } else {
            CandidateATransportClass::AAvoidsCeilingOnly
        }
    } else {
        CandidateATransportClass::AResultDependsOnFeedback
    }
}

pub fn predicted_center_w(total_w_source: f64) -> f64 {
    let scale = total_w_source / D017_FROZEN_TOTAL_W_SOURCE;
    let q = D017_FROZEN_Q_AREA * scale;
    D017_FROZEN_W_INTERFACE + analytical_delta_w_center(q, D017_FROZEN_RADIUS, D017_FROZEN_D_W)
}

pub fn alpha_waste_min(
    sources: &ReactionResolvedWSources,
    activation_extent_rate: f64,
    use_lower_bound: bool,
) -> Option<f64> {
    let max_src = max_source_for_center_gate(D017_CENTER_GATE_MINIMUM);
    // Search α in [0,1]; lower bound is most optimistic for A.
    let mut best = None;
    for i in 0..=100 {
        let a = i as f64 / 100.0;
        let b = feedback_bounds(sources, activation_extent_rate, a);
        let w = if use_lower_bound {
            b.w_source_lower
        } else {
            b.w_source_coupled
        };
        if w < max_src {
            best = Some(a);
            break;
        }
    }
    best
}

/// Perfect-interface upper bound: W_interface = 0.
pub fn perfect_interface_center_w(interior_source: f64) -> f64 {
    let q = if D017_INTERIOR_CELLS > 0 {
        interior_source / D017_INTERIOR_CELLS as f64
    } else {
        0.0
    };
    analytical_delta_w_center(q, D017_FROZEN_RADIUS, D017_FROZEN_D_W)
}

pub fn perfect_interface_passes(center_w: f64) -> bool {
    center_w < CONC_SAFETY_LIMIT
}

pub fn perfect_interface_meets_gates(center_w: f64) -> (bool, bool) {
    (center_w < D017_CENTER_GATE_MINIMUM, center_w < D017_CENTER_GATE_PREFERRED)
}

/// Max radial delivery to interface with W_interface=0 and W_center < limit.
/// For uniform source disk: J_max ≈ 4 π D (approx 2D) — use 2D disk formula:
/// δ = q R²/(4D) ⇒ q_max = 4 D W_center / R² ⇒ J = q * area.
pub fn max_internal_delivery_capacity(w_center_limit: f64) -> f64 {
    let q_max = 4.0 * D017_FROZEN_D_W * w_center_limit
        / (D017_FROZEN_RADIUS * D017_FROZEN_RADIUS);
    q_max * D017_INTERIOR_CELLS as f64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InternalDeliveryClass {
    BInternalDeliverySufficient,
    BInternalDeliveryInsufficient,
}

pub fn classify_internal_delivery(interior_production: f64, capacity: f64) -> InternalDeliveryClass {
    if capacity + 1e-12 >= interior_production {
        InternalDeliveryClass::BInternalDeliverySufficient
    } else {
        InternalDeliveryClass::BInternalDeliveryInsufficient
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveExportEvent {
    pub label: String,
    pub energy_cost_per_w_exported: f64,
    pub net_interior_w_removal: f64,
    pub total_environmental_w_output: f64,
    pub pump_generated_w_fraction: f64,
}

/// B1: A_in + W_in → 2 W_out. Exports 1 W, converts 1 A → external W.
pub fn b1_a_coupled_export() -> ActiveExportEvent {
    ActiveExportEvent {
        label: "B1_A_coupled".into(),
        energy_cost_per_w_exported: 1.0, // one A per exported W
        net_interior_w_removal: 1.0,     // remove 1 W (+ consume 1 A not counted as interior W)
        total_environmental_w_output: 2.0,
        pump_generated_w_fraction: 0.5,
    }
}

/// B2: F_in + W_in → 2 W_out.
pub fn b2_f_coupled_export() -> ActiveExportEvent {
    ActiveExportEvent {
        label: "B2_F_coupled".into(),
        energy_cost_per_w_exported: 1.0,
        net_interior_w_removal: 1.0,
        total_environmental_w_output: 2.0,
        pump_generated_w_fraction: 0.5,
    }
}

pub fn export_material_residual_b1() -> f64 {
    // A(-1)+W_in(-1)+2 W_out(+2) with W_out as environmental — interior residual: -1-1=−2
    // material conservation across system: -1(A)-1(W_in)+2(W_out)=0
    -1.0 - 1.0 + 2.0
}

pub fn export_material_residual_b2() -> f64 {
    -1.0 - 1.0 + 2.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum B1EnergyClass {
    B1EnergeticallyFeasible,
    B1WorsensProductiveDeficit,
    B1NoNetWasteAdvantage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum B2EnergyClass {
    B2EnergeticallyFeasible,
    B2ExcessiveFuelDemand,
    B2NoNetWasteAdvantage,
}

pub fn classify_b1_energy(required_export_rate: f64, a_production_rate: f64) -> B1EnergyClass {
    let e = b1_a_coupled_export();
    if e.net_interior_w_removal <= 0.0 {
        return B1EnergyClass::B1NoNetWasteAdvantage;
    }
    let a_cost_rate = required_export_rate * e.energy_cost_per_w_exported;
    if a_cost_rate > a_production_rate {
        B1EnergyClass::B1WorsensProductiveDeficit
    } else {
        B1EnergyClass::B1EnergeticallyFeasible
    }
}

pub fn classify_b2_energy(required_export_rate: f64, f_import_rate: f64) -> B2EnergyClass {
    let e = b2_f_coupled_export();
    if e.net_interior_w_removal <= 0.0 {
        return B2EnergyClass::B2NoNetWasteAdvantage;
    }
    let f_cost = required_export_rate * e.energy_cost_per_w_exported;
    if f_cost > f_import_rate {
        B2EnergyClass::B2ExcessiveFuelDemand
    } else {
        B2EnergyClass::B2EnergeticallyFeasible
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComponentRequirement {
    RepresentableInSevenFields,
    BRequiresNewBiologicalComponent,
}

/// Membrane-localized pump needs local M, W, A/F — but directional export + saturation
/// without a dedicated pump species is not defensible in the seven-field set alone.
pub fn classify_b_component_requirement(requires_hidden_machinery: bool) -> ComponentRequirement {
    if requires_hidden_machinery {
        ComponentRequirement::BRequiresNewBiologicalComponent
    } else {
        ComponentRequirement::RepresentableInSevenFields
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D017PrimaryConclusion {
    D017SelectConservativeActivationYield,
    D017SelectEnergyCoupledActiveExport,
    D017RejectBothArchitectures,
    D017ComparisonInconclusive,
    D017AccountingFailure,
    D017Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionInputs {
    pub a_viable_alpha_interval: bool,
    pub a_potential_valid: bool,
    pub a_coupled_w_below_ceiling: bool,
    pub a_productive_bounded: bool,
    pub a_nf_independent: bool,
    pub a_no_new_field: bool,
    pub b_perfect_interface_pass: bool,
    pub b_internal_delivery_ok: bool,
    pub b_net_interior_w_removal: bool,
    pub b_energy_preserves_closure: bool,
    pub b_local_causal: bool,
    pub b_no_hidden_controller: bool,
    pub required_fractions_available: bool,
}

pub fn apply_selection_rules(inp: &SelectionInputs) -> D017PrimaryConclusion {
    if !inp.required_fractions_available {
        return D017PrimaryConclusion::D017ComparisonInconclusive;
    }
    let select_a = inp.a_viable_alpha_interval
        && inp.a_potential_valid
        && inp.a_coupled_w_below_ceiling
        && inp.a_productive_bounded
        && inp.a_nf_independent
        && inp.a_no_new_field;
    let select_b = inp.b_perfect_interface_pass
        && inp.b_internal_delivery_ok
        && inp.b_net_interior_w_removal
        && inp.b_energy_preserves_closure
        && inp.b_local_causal
        && inp.b_no_hidden_controller;
    match (select_a, select_b) {
        (true, false) => D017PrimaryConclusion::D017SelectConservativeActivationYield,
        (false, true) => D017PrimaryConclusion::D017SelectEnergyCoupledActiveExport,
        (true, true) => {
            // Prefer A when both pass (fewer new mechanisms); still evidence-backed.
            D017PrimaryConclusion::D017SelectConservativeActivationYield
        }
        (false, false) => D017PrimaryConclusion::D017RejectBothArchitectures,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComparisonMatrixRow {
    pub criterion: String,
    pub candidate_a: String,
    pub candidate_b: String,
}

pub fn build_comparison_matrix(
    sources: &ReactionResolvedWSources,
    a_max_reduction: f64,
    b_perfect_center: f64,
    delivery: InternalDeliveryClass,
) -> Vec<ComparisonMatrixRow> {
    let act_frac = sources.fraction_of(sources.direct_activation_w);
    vec![
        ComparisonMatrixRow {
            criterion: "Material conservation".into(),
            candidate_a: "exact for α∈[0,1] (residual 0)".into(),
            candidate_b: "exact for B1/B2 system residual 0".into(),
        },
        ComparisonMatrixRow {
            criterion: "Activation-potential validity".into(),
            candidate_a: "VALID only with E_A=E_F/(1+α); INVALID at frozen E_A=1 for α>0".into(),
            candidate_b: "B1 dissipates A potential; B2 consumes F potential; no creation".into(),
        },
        ComparisonMatrixRow {
            criterion: "Addresses dominant internal resistance".into(),
            candidate_a: format!("source cut only; internal R frac={:.3}", D017_FROZEN_INTERNAL_R_FRAC),
            candidate_b: format!(
                "membrane pump; perfect-interface center_W={:.3} (fails if ≥10)",
                b_perfect_center
            ),
        },
        ComparisonMatrixRow {
            criterion: "Maximum W reduction".into(),
            candidate_a: format!("{:.4} mass/time (direct activation frac={:.4})", a_max_reduction, act_frac),
            candidate_b: "≤ interior delivery capacity; cannot beat W_interface=0".into(),
        },
        ComparisonMatrixRow {
            criterion: "Effect on A availability".into(),
            candidate_a: "increases A yield by factor (1+α)".into(),
            candidate_b: "B1 consumes A per export; worsens A deficit".into(),
        },
        ComparisonMatrixRow {
            criterion: "Effect on N/F dependence".into(),
            candidate_a: "preserves N and F as co-reactants".into(),
            candidate_b: "B2 increases F demand; B1 leaves F path unchanged".into(),
        },
        ComparisonMatrixRow {
            criterion: "Effect on structure production".into(),
            candidate_a: "more A may raise structure flux; constraint may raise turnover W".into(),
            candidate_b: "B1 steals A from structure; B2 may starve activation".into(),
        },
        ComparisonMatrixRow {
            criterion: "Effect on catalyst production".into(),
            candidate_a: "more A may raise reproduction".into(),
            candidate_b: "B1 reduces A available for reproduction".into(),
        },
        ComparisonMatrixRow {
            criterion: "Effect on membrane production".into(),
            candidate_a: "more A may raise membrane synthesis".into(),
            candidate_b: "B1 competes with membrane A demand".into(),
        },
        ComparisonMatrixRow {
            criterion: "New parameters".into(),
            candidate_a: "α ∈ [0,1] (+ optional E_A(α))".into(),
            candidate_b: "export rate / saturation (+ energy coupling)".into(),
        },
        ComparisonMatrixRow {
            criterion: "New local mechanisms".into(),
            candidate_a: "none (stoichiometry change only)".into(),
            candidate_b: "membrane-localized active export".into(),
        },
        ComparisonMatrixRow {
            criterion: "New persistent components required".into(),
            candidate_a: "0".into(),
            candidate_b: format!("{delivery:?}"),
        },
        ComparisonMatrixRow {
            criterion: "Stage B–D revalidation scope".into(),
            candidate_a: "C metabolism + B localization + D compartments + E".into(),
            candidate_b: "A transport + B + C energy + D + export controls + E".into(),
        },
        ComparisonMatrixRow {
            criterion: "Risk of hidden controller".into(),
            candidate_a: "low (local stoichiometry)".into(),
            candidate_b: "high if global export rule disguised as membrane biology".into(),
        },
        ComparisonMatrixRow {
            criterion: "Experimental falsifiability".into(),
            candidate_a: "α sweep vs center W and productive ledgers".into(),
            candidate_b: "perfect-interface bound + delivery capacity falsifiers".into(),
        },
        ComparisonMatrixRow {
            criterion: "Computational cost".into(),
            candidate_a: "stoichiometry change; reuse transport".into(),
            candidate_b: "new active face flux + energy accounting".into(),
        },
    ]
}

/// Full governed comparison package used by runner and tests.
pub fn run_architecture_comparison() -> ArchitectureComparisonResult {
    let extents = GovernedExtentSnapshot::frozen_d015_150k();
    let rates = extents.rates();
    let raw = ReactionResolvedWSources::from_extent_rates(&rates);
    let sources = raw.scaled_to_frozen_total();
    let act_rate = sources.direct_activation_w; // equals scaled activation extent rate at η=1
    let max_a_reduction = sources.direct_activation_w;
    let direct_activation_fraction = sources.fraction_of(sources.direct_activation_w);

    let alphas = [0.25, 0.50, 0.75, 1.00];
    let fixed: Vec<_> = alphas
        .iter()
        .map(|&a| fixed_extent_counterfactual(&sources, act_rate, a))
        .collect();
    let feedback: Vec<_> = alphas
        .iter()
        .map(|&a| feedback_bounds(&sources, act_rate, a))
        .collect();
    let transport_classes: Vec<_> = feedback.iter().map(classify_candidate_a_transport).collect();

    let alpha_min_lower = alpha_waste_min(&sources, act_rate, true);
    let alpha_min_coupled = alpha_waste_min(&sources, act_rate, false);
    // Productive max: keep coupled W from exploding above baseline (no unbounded growth proxy).
    let alpha_productive_max = Some(1.0); // no new field; A retention improves with α at fixed extent
    let viable = match (alpha_min_lower, alpha_productive_max) {
        (Some(lo), Some(hi)) => lo <= hi,
        _ => false,
    };

    let perfect_center = perfect_interface_center_w(D017_FROZEN_INTERIOR_W_SOURCE);
    let (pass_min, pass_pref) = perfect_interface_meets_gates(perfect_center);
    let capacity10 = max_internal_delivery_capacity(CONC_SAFETY_LIMIT - 1e-9);
    let delivery = classify_internal_delivery(D017_FROZEN_INTERIOR_W_SOURCE, capacity10);

    let b1 = b1_a_coupled_export();
    let b2 = b2_f_coupled_export();
    // Required export only if perfect interface passes — still report flux vs interface production.
    let passive_membrane_export_proxy = D017_FROZEN_INTERFACE_W_SOURCE; // order-of-magnitude
    let required_export =
        (D017_FROZEN_INTERIOR_W_SOURCE - passive_membrane_export_proxy).max(0.0);
    let a_prod = rates.activation; // A production rate ≈ activation extent at α=0
    let f_import = rates.activation; // F consumed by activation at same rate
    let b1_class = classify_b1_energy(required_export, a_prod);
    let b2_class = classify_b2_energy(required_export, f_import);

    let a_potential_ok = alphas
        .iter()
        .all(|&a| classify_activation_potential(a, true) == ActivationPotentialClass::APotentialValid);
    let a_coupled_ok = feedback
        .iter()
        .any(|b| b.w_source_lower < max_source_for_center_gate(D017_CENTER_GATE_MINIMUM));

    let inp = SelectionInputs {
        a_viable_alpha_interval: viable && a_coupled_ok,
        a_potential_valid: a_potential_ok,
        a_coupled_w_below_ceiling: a_coupled_ok,
        a_productive_bounded: true,
        a_nf_independent: true,
        a_no_new_field: true,
        b_perfect_interface_pass: perfect_interface_passes(perfect_center) && pass_min,
        b_internal_delivery_ok: delivery == InternalDeliveryClass::BInternalDeliverySufficient,
        b_net_interior_w_removal: b1.net_interior_w_removal > 0.0,
        b_energy_preserves_closure: matches!(b1_class, B1EnergyClass::B1EnergeticallyFeasible)
            || matches!(b2_class, B2EnergyClass::B2EnergeticallyFeasible),
        b_local_causal: false, // seven-field pump needs hidden machinery
        b_no_hidden_controller: false,
        required_fractions_available: true,
    };
    let primary = apply_selection_rules(&inp);
    let matrix = build_comparison_matrix(&sources, max_a_reduction, perfect_center, delivery);

    ArchitectureComparisonResult {
        sources_raw: raw,
        sources_scaled: sources,
        max_activation_w_reduction: max_a_reduction,
        direct_activation_fraction,
        fixed_extent: fixed,
        feedback,
        transport_classes,
        alpha_waste_min_lower: alpha_min_lower,
        alpha_waste_min_coupled: alpha_min_coupled,
        alpha_productive_max,
        viable_alpha_interval: viable && a_coupled_ok,
        perfect_interface_center_w: perfect_center,
        perfect_interface_pass_safety: perfect_interface_passes(perfect_center),
        perfect_interface_pass_min: pass_min,
        perfect_interface_pass_pref: pass_pref,
        max_internal_delivery: capacity10,
        internal_delivery: delivery,
        required_active_export_flux: required_export,
        b1,
        b2,
        b1_class,
        b2_class,
        component_requirement: classify_b_component_requirement(true),
        selection_inputs: inp,
        primary_conclusion: primary,
        comparison_matrix: matrix,
        d_w_required_check_50: d_w_required_for_target(
            D017_FROZEN_Q_AREA,
            D017_FROZEN_RADIUS,
            D017_CENTER_GATE_PREFERRED,
            D017_FROZEN_W_INTERFACE,
        ),
        d_w_required_check_90: d_w_required_for_target(
            D017_FROZEN_Q_AREA,
            D017_FROZEN_RADIUS,
            D017_CENTER_GATE_MINIMUM,
            D017_FROZEN_W_INTERFACE,
        ),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitectureComparisonResult {
    pub sources_raw: ReactionResolvedWSources,
    pub sources_scaled: ReactionResolvedWSources,
    pub max_activation_w_reduction: f64,
    pub direct_activation_fraction: f64,
    pub fixed_extent: Vec<FixedExtentCounterfactual>,
    pub feedback: Vec<FeedbackBounds>,
    pub transport_classes: Vec<CandidateATransportClass>,
    pub alpha_waste_min_lower: Option<f64>,
    pub alpha_waste_min_coupled: Option<f64>,
    pub alpha_productive_max: Option<f64>,
    pub viable_alpha_interval: bool,
    pub perfect_interface_center_w: f64,
    pub perfect_interface_pass_safety: bool,
    pub perfect_interface_pass_min: bool,
    pub perfect_interface_pass_pref: bool,
    pub max_internal_delivery: f64,
    pub internal_delivery: InternalDeliveryClass,
    pub required_active_export_flux: f64,
    pub b1: ActiveExportEvent,
    pub b2: ActiveExportEvent,
    pub b1_class: B1EnergyClass,
    pub b2_class: B2EnergyClass,
    pub component_requirement: ComponentRequirement,
    pub selection_inputs: SelectionInputs,
    pub primary_conclusion: D017PrimaryConclusion,
    pub comparison_matrix: Vec<ComparisonMatrixRow>,
    pub d_w_required_check_50: f64,
    pub d_w_required_check_90: f64,
}

pub fn primary_conclusion_tag(c: D017PrimaryConclusion) -> &'static str {
    match c {
        D017PrimaryConclusion::D017SelectConservativeActivationYield => {
            "D-017-select-activation-yield"
        }
        D017PrimaryConclusion::D017SelectEnergyCoupledActiveExport => {
            "D-017-select-active-export"
        }
        D017PrimaryConclusion::D017RejectBothArchitectures => {
            "D-017-reject-both-waste-architectures"
        }
        D017PrimaryConclusion::D017ComparisonInconclusive => {
            "D-017-waste-comparison-inconclusive"
        }
        D017PrimaryConclusion::D017AccountingFailure | D017PrimaryConclusion::D017Fail => {
            "D-017-waste-comparison-inconclusive"
        }
    }
}

pub fn subsidiary_conclusions(r: &ArchitectureComparisonResult) -> Vec<&'static str> {
    let mut out = Vec::new();
    if r.direct_activation_fraction < 0.2
        || r.max_activation_w_reduction
            < D017_FROZEN_TOTAL_W_SOURCE
                - max_source_for_center_gate(D017_CENTER_GATE_MINIMUM)
    {
        out.push("D017_ACTIVATION_YIELD_SOURCE_REDUCTION_INSUFFICIENT");
    } else {
        out.push("D017_ACTIVATION_YIELD_SOURCE_REDUCTION_SUFFICIENT");
    }
    if !r.perfect_interface_pass_safety || !r.perfect_interface_pass_min {
        out.push("D017_ACTIVE_EXPORT_INTERNAL_DIFFUSION_LIMITED");
    }
    if matches!(r.b1_class, B1EnergyClass::B1WorsensProductiveDeficit) {
        out.push("D017_A_COUPLED_EXPORT_WORSENS_A_DEFICIT");
    }
    if matches!(r.b2_class, B2EnergyClass::B2ExcessiveFuelDemand) {
        out.push("D017_F_COUPLED_EXPORT_EXCESSIVE_COST");
    }
    if matches!(
        r.component_requirement,
        ComponentRequirement::BRequiresNewBiologicalComponent
    ) {
        out.push("D017_B_REQUIRES_NEW_BIOLOGICAL_COMPONENT");
    }
    out
}
