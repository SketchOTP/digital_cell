//! D-052 nutrient/fuel delivery resistance decomposition (diagnostic only).
//!
//! No biological parameter or equation changes. Classifiers and observer math only.

use serde::{Deserialize, Serialize};

pub const D052_PROJECT_ID: &str = "D-052";
pub const D052_AGENT_MEMORY_ID: &str =
    "D-20260720-2335-d052-resource-delivery-resistance-decomposition";
pub const D052_STARTING_COMMIT: &str = "e08075a";
pub const D052_STARTING_TAG: &str = "D-051-coupled-activation-throughput-audit";
pub const D052_FROZEN_D049: &str = "D049_COUPLED_ACTIVATION_CAPACITY_FAILURE";
pub const D052_FROZEN_D050: &str = "D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED";
pub const D052_FROZEN_D051: &str = "D051_RESOURCE_THROUGHPUT_LIMIT";
pub const D052_FROZEN_TOPOLOGY: &str = "COUPLED_ACTIVATION_TOPOLOGY_CAPABLE";
pub const D052_ACTIVATION_SUPPLY_LAW_NOTE: &str = "ACTIVATION_SUPPLY_LAW_NOT_CURRENT_REPAIR_TARGET";

pub const D052_FITTED_V_A: f64 = 0.12544510052968755;
pub const D052_FITTED_K_C: f64 = 0.10;
pub const D052_N_REF: f64 = 1.0;
pub const D052_F_REF: f64 = 1.0;
pub const D052_RADIUS: f64 = 22.0;
pub const D052_THETA: f64 = 0.6;
pub const D052_DEFAULT_HORIZON: u64 = 10_000;
pub const D052_CONTROL_HORIZON: u64 = 5_000;
pub const D052_EPS: f64 = 1.0e-18;
pub const D052_LEDGER_REL_TOL: f64 = 0.08;
pub const D052_RETENTION_COLLAPSE: f64 = 0.10;
pub const D052_MATERIAL_RISE: f64 = 0.50; // ≥50% rise for diagnostic dominance
pub const D052_RESISTANCE_DOMINANCE: f64 = 0.50;
pub const D052_HEALTHY_N: f64 = 1.0;
pub const D052_HEALTHY_F: f64 = 1.0;
pub const D052_STAGE_A_NF_PERM_LO: f64 = 0.20;
pub const D052_STAGE_A_NF_PERM_HI: f64 = 0.50;

/// Resistance / delivery path segments (Gate 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeliverySegment {
    ReservoirRelaxation,
    ReservoirToExteriorDiffusion,
    ExteriorDiffusion,
    MembraneCrossing,
    PeripheralInteriorDiffusion,
    CentralInteriorDelivery,
}

impl DeliverySegment {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReservoirRelaxation => "reservoir_relaxation",
            Self::ReservoirToExteriorDiffusion => "reservoir_to_exterior_diffusion",
            Self::ExteriorDiffusion => "exterior_diffusion",
            Self::MembraneCrossing => "membrane_crossing",
            Self::PeripheralInteriorDiffusion => "peripheral_interior_diffusion",
            Self::CentralInteriorDelivery => "central_interior_delivery",
        }
    }

    pub fn all() -> &'static [DeliverySegment] {
        &[
            Self::ReservoirRelaxation,
            Self::ReservoirToExteriorDiffusion,
            Self::ExteriorDiffusion,
            Self::MembraneCrossing,
            Self::PeripheralInteriorDiffusion,
            Self::CentralInteriorDelivery,
        ]
    }
}

/// Effective segment resistance R = Δc / max(|J|, ε).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SegmentResistance {
    pub segment: DeliverySegment,
    pub delta_c: f64,
    pub flux: f64,
    pub resistance: f64,
    pub fraction: f64,
}

pub fn segment_resistance(delta_c: f64, flux: f64) -> f64 {
    delta_c.abs() / flux.abs().max(D052_EPS)
}

pub fn normalize_resistance_fractions(resistances: &mut [SegmentResistance]) {
    let sum: f64 = resistances.iter().map(|r| r.resistance).sum::<f64>().max(D052_EPS);
    for r in resistances.iter_mut() {
        r.fraction = r.resistance / sum;
    }
}

pub fn dominant_segment(fractions: &[SegmentResistance]) -> Option<DeliverySegment> {
    fractions
        .iter()
        .filter(|r| r.fraction >= D052_RESISTANCE_DOMINANCE)
        .max_by(|a, b| {
            a.fraction
                .partial_cmp(&b.fraction)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|r| r.segment)
}

/// N/F regional ledger closure (Gate 1).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourceRegionalLedger {
    pub j_reservoir: f64,
    pub j_exterior: f64,
    pub j_interface: f64,
    pub j_interior: f64,
    pub loss_activation: f64,
    pub loss_reproduction: f64,
    pub loss_structural: f64,
    pub loss_precursor: f64,
    pub loss_other: f64,
    pub delta_reservoir: f64,
    pub delta_exterior: f64,
    pub delta_interface: f64,
    pub delta_peripheral: f64,
    pub delta_central: f64,
}

impl ResourceRegionalLedger {
    pub fn total_loss(self) -> f64 {
        self.loss_activation
            + self.loss_reproduction
            + self.loss_structural
            + self.loss_precursor
            + self.loss_other
    }

    pub fn total_storage_delta(self) -> f64 {
        self.delta_reservoir
            + self.delta_exterior
            + self.delta_interface
            + self.delta_peripheral
            + self.delta_central
    }

    /// ΔM ≟ J_reservoir + J_transport − L  (transport = exterior+interface+interior proxy net).
    pub fn predicted_delta(self) -> f64 {
        self.j_reservoir + self.j_exterior + self.j_interface + self.j_interior - self.total_loss()
    }

    pub fn closes(self, observed_delta: f64, rel_tol: f64) -> bool {
        let pred = self.predicted_delta();
        let scale = observed_delta.abs().max(pred.abs()).max(1.0);
        (observed_delta - pred).abs() <= rel_tol * scale
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceIdentity {
    NutrientDominant,
    FuelDominant,
    JointResourceLimit,
    ResourceIdentityUnresolved,
}

impl ResourceIdentity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NutrientDominant => "NUTRIENT_DOMINANT",
            Self::FuelDominant => "FUEL_DOMINANT",
            Self::JointResourceLimit => "JOINT_RESOURCE_LIMIT",
            Self::ResourceIdentityUnresolved => "RESOURCE_IDENTITY_UNRESOLVED",
        }
    }
}

/// Classify N vs F limitation from matched controls.
pub fn classify_resource_identity(
    a_n_only: f64,
    a_f_only: f64,
    a_joint: f64,
    a_baseline: f64,
    n_limited_frac: f64,
    f_limited_frac: f64,
) -> ResourceIdentity {
    let n_rise = material_throughput_rise(a_baseline, a_n_only);
    let f_rise = material_throughput_rise(a_baseline, a_f_only);
    let joint_rise = material_throughput_rise(a_baseline, a_joint);
    if joint_rise && n_rise && f_rise {
        return ResourceIdentity::JointResourceLimit;
    }
    if joint_rise && n_rise && !f_rise {
        return ResourceIdentity::NutrientDominant;
    }
    if joint_rise && f_rise && !n_rise {
        return ResourceIdentity::FuelDominant;
    }
    if n_limited_frac > 0.55 && f_limited_frac < 0.25 {
        return ResourceIdentity::NutrientDominant;
    }
    if f_limited_frac > 0.55 && n_limited_frac < 0.25 {
        return ResourceIdentity::FuelDominant;
    }
    if n_limited_frac > 0.30 && f_limited_frac > 0.30 {
        return ResourceIdentity::JointResourceLimit;
    }
    if joint_rise {
        return ResourceIdentity::JointResourceLimit;
    }
    ResourceIdentity::ResourceIdentityUnresolved
}

pub fn material_throughput_rise(baseline: f64, treatment: f64) -> bool {
    if baseline <= D052_EPS {
        return treatment > baseline + D052_EPS;
    }
    (treatment - baseline) / baseline >= D052_MATERIAL_RISE
}

pub fn chi_supply(j_in: f64, l_required: f64) -> f64 {
    j_in / l_required.max(D052_EPS)
}

pub fn chi_activation(j_n: f64, j_f: f64, l_a: f64) -> f64 {
    j_n.min(j_f) / l_a.max(D052_EPS)
}

/// Stage A N/F permeability band check (normalized Π at full occupancy proxy).
pub fn stage_a_nf_permeability_in_range(perm: f64) -> bool {
    (D052_STAGE_A_NF_PERM_LO..=D052_STAGE_A_NF_PERM_HI).contains(&perm)
}

pub fn nf_permeability_from_beta(beta: f64, theta: f64) -> f64 {
    (-beta * theta.max(0.0)).exp()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepletionLocus {
    BeforeMembrane,
    AcrossMembrane,
    ImmediatelyAfterEntry,
    WithinInterior,
    AtActivationSites,
    Unresolved,
}

impl DepletionLocus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeMembrane => "before_membrane",
            Self::AcrossMembrane => "across_membrane",
            Self::ImmediatelyAfterEntry => "immediately_after_entry",
            Self::WithinInterior => "within_interior",
            Self::AtActivationSites => "at_activation_sites",
            Self::Unresolved => "unresolved",
        }
    }
}

/// Identify primary depletion locus from regional mean concentrations.
pub fn classify_depletion_locus(
    c_reservoir: f64,
    c_exterior: f64,
    c_outside: f64,
    c_inside: f64,
    c_peripheral: f64,
    c_central: f64,
    c_activation: f64,
) -> DepletionLocus {
    let drops = [
        (
            DepletionLocus::BeforeMembrane,
            (c_reservoir - c_outside).max(0.0),
        ),
        (
            DepletionLocus::AcrossMembrane,
            (c_outside - c_inside).max(0.0),
        ),
        (
            DepletionLocus::ImmediatelyAfterEntry,
            (c_inside - c_peripheral).max(0.0),
        ),
        (
            DepletionLocus::WithinInterior,
            (c_peripheral - c_central).max(0.0),
        ),
        (
            DepletionLocus::AtActivationSites,
            (c_central - c_activation).max(0.0),
        ),
    ];
    // Also credit exterior drop before membrane vicinity.
    let exterior_drop = (c_exterior - c_outside).max(0.0);
    let mut best = drops
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(l, _)| *l)
        .unwrap_or(DepletionLocus::Unresolved);
    let best_drop = drops
        .iter()
        .find(|(l, _)| *l == best)
        .map(|(_, d)| *d)
        .unwrap_or(0.0);
    if exterior_drop > best_drop && exterior_drop > (c_reservoir - c_exterior).max(0.0) {
        best = DepletionLocus::BeforeMembrane;
    }
    if best_drop <= D052_EPS && exterior_drop <= D052_EPS {
        return DepletionLocus::Unresolved;
    }
    best
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D052PrimaryConclusion {
    ReservoirResourceDeliveryLimit,
    ExteriorResourceDiffusionLimit,
    MembraneResourcePermeabilityLimit,
    InteriorResourceDiffusionLimit,
    ReactionShellDepletion,
    ResourceSurfaceVolumeScalingLimit,
    SelectivityThroughputIncompatibility,
    ActivationStoichiometricYieldLimit,
    MixedResourceDeliveryLimit,
    ResourceDeliveryDecompositionInconclusive,
    D051ResourceLimitNotReproduced,
    ResourceLedgerFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D052PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReservoirResourceDeliveryLimit => "D052_RESERVOIR_RESOURCE_DELIVERY_LIMIT",
            Self::ExteriorResourceDiffusionLimit => "D052_EXTERIOR_RESOURCE_DIFFUSION_LIMIT",
            Self::MembraneResourcePermeabilityLimit => "D052_MEMBRANE_RESOURCE_PERMEABILITY_LIMIT",
            Self::InteriorResourceDiffusionLimit => "D052_INTERIOR_RESOURCE_DIFFUSION_LIMIT",
            Self::ReactionShellDepletion => "D052_REACTION_SHELL_DEPLETION",
            Self::ResourceSurfaceVolumeScalingLimit => "D052_RESOURCE_SURFACE_VOLUME_SCALING_LIMIT",
            Self::SelectivityThroughputIncompatibility => {
                "D052_SELECTIVITY_THROUGHPUT_INCOMPATIBILITY"
            }
            Self::ActivationStoichiometricYieldLimit => {
                "D052_ACTIVATION_STOICHIOMETRIC_YIELD_LIMIT"
            }
            Self::MixedResourceDeliveryLimit => "D052_MIXED_RESOURCE_DELIVERY_LIMIT",
            Self::ResourceDeliveryDecompositionInconclusive => {
                "D052_RESOURCE_DELIVERY_DECOMPOSITION_INCONCLUSIVE"
            }
            Self::D051ResourceLimitNotReproduced => "D052_D051_RESOURCE_LIMIT_NOT_REPRODUCED",
            Self::ResourceLedgerFailure => "D052_RESOURCE_LEDGER_FAILURE",
            Self::AccountingFailure => "D052_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D052_NUMERICAL_FAILURE",
            Self::Fail => "D052_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RouteDecisionInput {
    pub d051_reproduced: bool,
    pub ledger_ok: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
    pub reservoir_dominant: bool,
    pub exterior_diffusion_dominant: bool,
    pub membrane_permeability_dominant: bool,
    pub interior_diffusion_dominant: bool,
    pub reaction_shell: bool,
    pub surface_volume_scaling: bool,
    pub selectivity_incompatibility: bool,
    pub yield_limit: bool,
    pub mixed_delivery: bool,
}

/// Exactly one primary route (Gate 12). Failure modes first, then causal segments.
pub fn select_primary_route(input: &RouteDecisionInput) -> D052PrimaryConclusion {
    if !input.numerical_ok {
        return D052PrimaryConclusion::NumericalFailure;
    }
    if !input.accounting_ok {
        return D052PrimaryConclusion::AccountingFailure;
    }
    if !input.d051_reproduced {
        return D052PrimaryConclusion::D051ResourceLimitNotReproduced;
    }
    if !input.ledger_ok {
        return D052PrimaryConclusion::ResourceLedgerFailure;
    }
    // Architecture / special findings before ordinary segment dominance.
    if input.selectivity_incompatibility {
        return D052PrimaryConclusion::SelectivityThroughputIncompatibility;
    }
    if input.surface_volume_scaling {
        return D052PrimaryConclusion::ResourceSurfaceVolumeScalingLimit;
    }
    if input.yield_limit {
        return D052PrimaryConclusion::ActivationStoichiometricYieldLimit;
    }
    if input.reaction_shell {
        return D052PrimaryConclusion::ReactionShellDepletion;
    }
    let flags = [
        input.reservoir_dominant,
        input.exterior_diffusion_dominant,
        input.membrane_permeability_dominant,
        input.interior_diffusion_dominant,
    ];
    let n_true = flags.iter().filter(|&&b| b).count();
    if n_true >= 2 || input.mixed_delivery {
        return D052PrimaryConclusion::MixedResourceDeliveryLimit;
    }
    if input.membrane_permeability_dominant {
        return D052PrimaryConclusion::MembraneResourcePermeabilityLimit;
    }
    if input.interior_diffusion_dominant {
        return D052PrimaryConclusion::InteriorResourceDiffusionLimit;
    }
    if input.exterior_diffusion_dominant {
        return D052PrimaryConclusion::ExteriorResourceDiffusionLimit;
    }
    if input.reservoir_dominant {
        return D052PrimaryConclusion::ReservoirResourceDeliveryLimit;
    }
    D052PrimaryConclusion::ResourceDeliveryDecompositionInconclusive
}

/// Observer-only activation yield model N+F → y_A A + W (Gate 10).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ObserverYieldProbe {
    pub y_a: f64,
    pub predicted_a_from_min_nf: f64,
    pub chi_activation_at_yield: f64,
}

pub fn observer_yield_probe(j_n: f64, j_f: f64, l_a_required: f64, y_a: f64) -> ObserverYieldProbe {
    let available = j_n.min(j_f) * y_a;
    ObserverYieldProbe {
        y_a,
        predicted_a_from_min_nf: available,
        chi_activation_at_yield: available / l_a_required.max(D052_EPS),
    }
}

pub fn required_analytical_yield(j_n: f64, j_f: f64, l_a_required: f64) -> f64 {
    l_a_required / j_n.min(j_f).max(D052_EPS)
}

/// Cap-site fractions from local N/F vs refs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct CapSiteFractions {
    pub n_limited: f64,
    pub f_limited: f64,
    pub jointly_limited: f64,
    pub unconstrained: f64,
}

pub fn classify_cap_sites(n_vals: &[f64], f_vals: &[f64], n_ref: f64, f_ref: f64) -> CapSiteFractions {
    let mut n_lim = 0u64;
    let mut f_lim = 0u64;
    let mut joint = 0u64;
    let mut unc = 0u64;
    let n = n_vals.len().min(f_vals.len());
    for i in 0..n {
        let n_ok = n_vals[i] >= 0.5 * n_ref;
        let f_ok = f_vals[i] >= 0.5 * f_ref;
        match (n_ok, f_ok) {
            (true, true) => unc += 1,
            (false, true) => n_lim += 1,
            (true, false) => f_lim += 1,
            (false, false) => joint += 1,
        }
    }
    let t = n.max(1) as f64;
    CapSiteFractions {
        n_limited: n_lim as f64 / t,
        f_limited: f_lim as f64 / t,
        jointly_limited: joint as f64 / t,
        unconstrained: unc as f64 / t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resistance_normalization_and_dominance() {
        let mut segs = vec![
            SegmentResistance {
                segment: DeliverySegment::MembraneCrossing,
                delta_c: 0.8,
                flux: 0.1,
                resistance: segment_resistance(0.8, 0.1),
                fraction: 0.0,
            },
            SegmentResistance {
                segment: DeliverySegment::ExteriorDiffusion,
                delta_c: 0.05,
                flux: 0.1,
                resistance: segment_resistance(0.05, 0.1),
                fraction: 0.0,
            },
        ];
        normalize_resistance_fractions(&mut segs);
        assert!((segs.iter().map(|s| s.fraction).sum::<f64>() - 1.0).abs() < 1e-12);
        assert_eq!(dominant_segment(&segs), Some(DeliverySegment::MembraneCrossing));
    }

    #[test]
    fn ledger_closure() {
        let led = ResourceRegionalLedger {
            j_reservoir: 1.0,
            j_exterior: 0.0,
            j_interface: -0.2,
            j_interior: 0.0,
            loss_activation: 0.5,
            loss_reproduction: 0.1,
            loss_structural: 0.0,
            loss_precursor: 0.0,
            loss_other: 0.0,
            ..Default::default()
        };
        // pred = 1.0 + 0 - 0.2 + 0 - 0.6 = 0.2
        assert!(led.closes(0.2, 0.05));
        assert!(!led.closes(1.0, 0.05));
    }

    #[test]
    fn resource_identity_joint() {
        let id = classify_resource_identity(0.15, 0.15, 1.1, 0.1, 0.4, 0.4);
        assert_eq!(id, ResourceIdentity::JointResourceLimit);
    }

    #[test]
    fn route_membrane_over_reservoir() {
        let c = select_primary_route(&RouteDecisionInput {
            d051_reproduced: true,
            ledger_ok: true,
            accounting_ok: true,
            numerical_ok: true,
            membrane_permeability_dominant: true,
            ..Default::default()
        });
        assert_eq!(
            c.as_str(),
            "D052_MEMBRANE_RESOURCE_PERMEABILITY_LIMIT"
        );
    }

    #[test]
    fn no_diagnostic_feedback_constants() {
        assert_eq!(D052_ACTIVATION_SUPPLY_LAW_NOTE, "ACTIVATION_SUPPLY_LAW_NOT_CURRENT_REPAIR_TARGET");
        assert!((D052_FITTED_V_A - 0.12544510052968755).abs() < 1e-15);
        assert!((nf_permeability_from_beta(1.2, 1.0) - (-1.2_f64).exp()).abs() < 1e-15);
        assert!(stage_a_nf_permeability_in_range((-1.2_f64).exp()));
    }
}
