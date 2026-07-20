//! D-046 activated-resource demand topology audit (observer / diagnostic only).
//!
//! No chemistry, activation-law, C_star, or rate changes. Establishes what causally
//! sets productive A demand and selects one topology route.

use serde::{Deserialize, Serialize};

pub const D046_AGENT_MEMORY_ID: &str =
    "D-20260720-d046-activated-resource-demand-topology-audit";
pub const D046_D045_TAG: &str = "D-045-fuel-charged-activation-fail";
pub const D046_D044_TAG: &str = "D-044-activation-law-fail";
pub const D046_D045_RESULT_COMMIT: &str = "41f9b75";
pub const D046_D044_RESULT_COMMIT: &str = "1473f0775c395e942fae7d98576d9a4640ad7ae9";
pub const D046_RECORD_FUEL_CHARGED: &str = "FUEL_CHARGED_CATALYST_NOT_AUTHORIZED";
pub const D046_HISTORICAL_K: f64 = 0.020;
pub const D046_LEDGER_REL_TOL: f64 = 1e-3;
pub const D046_RESIDUAL_TOL: f64 = 1e-3;
/// Gate 8 prospective held-out error limits (preregistered).
pub const D046_MODEL_MEDIAN_HOLD_ERR: f64 = 0.20;
pub const D046_MODEL_MAX_HOLD_ERR: f64 = 0.35;
pub const D046_MODEL_BOOTSTRAP_SPREAD: f64 = 0.50;
pub const D046_MODEL_LOO_FACTOR: f64 = 2.0;
/// Issued D-045 Gate 0 checks (directive text).
pub const D046_D045_ISSUED_DC_SPAN: f64 = 3.0;
/// Implementation-only D-045 fit threshold (not in issued directive).
pub const D046_D045_IMPL_FIT_ERR: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D046Conclusion {
    ADemandAccountingDefect,
    ConstraintContaminatedDemand,
    StructuralADemandDefect,
    PrecursorADemandDefect,
    ReproductionADemandDefect,
    MixedADemandTopology,
    CatalystSaturatingVolumeActivationJustified,
    ADemandTopologyInconclusive,
    D045ThresholdProvenanceUnresolved,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D046Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ADemandAccountingDefect => "D046_A_DEMAND_ACCOUNTING_DEFECT",
            Self::ConstraintContaminatedDemand => "D046_CONSTRAINT_CONTAMINATED_DEMAND",
            Self::StructuralADemandDefect => "D046_STRUCTURAL_A_DEMAND_DEFECT",
            Self::PrecursorADemandDefect => "D046_PRECURSOR_A_DEMAND_DEFECT",
            Self::ReproductionADemandDefect => "D046_REPRODUCTION_A_DEMAND_DEFECT",
            Self::MixedADemandTopology => "D046_MIXED_A_DEMAND_TOPOLOGY",
            Self::CatalystSaturatingVolumeActivationJustified => {
                "D046_CATALYST_SATURATING_VOLUME_ACTIVATION_JUSTIFIED"
            }
            Self::ADemandTopologyInconclusive => "D046_A_DEMAND_TOPOLOGY_INCONCLUSIVE",
            Self::D045ThresholdProvenanceUnresolved => "D046_D045_THRESHOLD_PROVENANCE_UNRESOLVED",
            Self::AccountingFailure => "D046_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D046_NUMERICAL_FAILURE",
            Self::Fail => "D046_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D046Route {
    RouteA,
    RouteC,
    RouteS,
    RouteP,
    RouteR,
    RouteM,
    RouteV,
    RouteI,
}

impl D046Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RouteA => "ROUTE_A_DEMAND_ACCOUNTING_DEFECT",
            Self::RouteC => "ROUTE_C_CONSTRAINT_CONTAMINATED_DEMAND",
            Self::RouteS => "ROUTE_S_STRUCTURAL_A_DEMAND_DEFECT",
            Self::RouteP => "ROUTE_P_PRECURSOR_A_DEMAND_DEFECT",
            Self::RouteR => "ROUTE_R_REPRODUCTION_A_DEMAND_DEFECT",
            Self::RouteM => "ROUTE_M_MIXED_A_DEMAND_TOPOLOGY",
            Self::RouteV => "ROUTE_V_CATALYST_SATURATING_VOLUME_ACTIVATION",
            Self::RouteI => "ROUTE_I_INCONCLUSIVE",
        }
    }

    pub const fn conclusion(self) -> D046Conclusion {
        match self {
            Self::RouteA => D046Conclusion::ADemandAccountingDefect,
            Self::RouteC => D046Conclusion::ConstraintContaminatedDemand,
            Self::RouteS => D046Conclusion::StructuralADemandDefect,
            Self::RouteP => D046Conclusion::PrecursorADemandDefect,
            Self::RouteR => D046Conclusion::ReproductionADemandDefect,
            Self::RouteM => D046Conclusion::MixedADemandTopology,
            Self::RouteV => D046Conclusion::CatalystSaturatingVolumeActivationJustified,
            Self::RouteI => D046Conclusion::ADemandTopologyInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D045ThresholdProvenance {
    /// Present in issued directive before campaign.
    PreregisteredGoverned,
    /// Added in implementation source before campaign results.
    ImplementationBeforeEvidence,
    /// Added after campaign evidence existed.
    AfterEvidence,
    /// Only in completion report.
    ReportOnly,
    Unresolved,
}

impl D045ThresholdProvenance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreregisteredGoverned => "PREREGISTERED_GOVERNED",
            Self::ImplementationBeforeEvidence => "IMPLEMENTATION_BEFORE_EVIDENCE",
            Self::AfterEvidence => "AFTER_EVIDENCE",
            Self::ReportOnly => "REPORT_ONLY",
            Self::Unresolved => "UNRESOLVED",
        }
    }

    pub const fn rejection_status(self) -> &'static str {
        match self {
            Self::PreregisteredGoverned => "D045_CATALYST_LINEARITY_REJECTION_GOVERNED",
            Self::ImplementationBeforeEvidence
            | Self::AfterEvidence
            | Self::ReportOnly => "D045_CATALYST_LINEARITY_REJECTION_PROVISIONAL",
            Self::Unresolved => "D046_D045_THRESHOLD_PROVENANCE_UNRESOLVED",
        }
    }
}

/// Gate 0 provenance decision from audited facts (not runtime git).
pub fn classify_d045_threshold_provenance(
    in_issued_directive: bool,
    in_source_before_campaign: bool,
    in_source_after_evidence: bool,
    in_report_only: bool,
) -> D045ThresholdProvenance {
    if in_issued_directive && in_source_before_campaign {
        return D045ThresholdProvenance::PreregisteredGoverned;
    }
    if in_source_before_campaign && !in_issued_directive {
        return D045ThresholdProvenance::ImplementationBeforeEvidence;
    }
    if in_source_after_evidence {
        return D045ThresholdProvenance::AfterEvidence;
    }
    if in_report_only {
        return D045ThresholdProvenance::ReportOnly;
    }
    D045ThresholdProvenance::Unresolved
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ASinkKind {
    Reproduction,
    Structure,
    Precursor,
    Membrane,
    Decay,
    Transport,
    Reservoir,
    Numerical,
    ConstraintVirtual,
    Intervention,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ASinkLineage {
    pub id: String,
    pub kind: ASinkKind,
    pub equation: String,
    pub equation_version: String,
    pub source_file: String,
    pub function: String,
    pub local_basis: String,
    pub spatial_weighting: String,
    pub unit_stoichiometry: String,
    pub produced_material: String,
    pub w_production: String,
    pub enabled_fixed_compartment: bool,
    pub enabled_true_radius: bool,
    pub enabled_constrained_radius: bool,
    pub enabled_damage: bool,
    pub in_d042_d045_ledgers: bool,
}

/// Authoritative A-sink catalog for active v8 / schema-3.
pub fn a_demand_lineage_catalog() -> Vec<ASinkLineage> {
    vec![
        ASinkLineage {
            id: "L_rep".into(),
            kind: ASinkKind::Reproduction,
            equation: "r_rep = k_d008_reproduction * C * A; A → η_C C + (1-η_C) W (v2)".into(),
            equation_version: "membrane_metabolism_v8 / conservative".into(),
            source_file: "activated_metabolism.rs".into(),
            function: "activated_metabolism_rates".into(),
            local_basis: "C * A".into(),
            spatial_weighting: "per-cell bulk (dish cells)".into(),
            unit_stoichiometry: "1 A → η_C C + (1-η_C) W".into(),
            produced_material: "C".into(),
            w_production: "(1-η_C) per A (v2)".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
        ASinkLineage {
            id: "L_structure".into(),
            kind: ASinkKind::Structure,
            equation: "r_φ = k_d008_structure * A * I(φ) [default] or A*q_c(C)*H(φ) [phase-volume]".into(),
            equation_version: "structural_kinetics / schema-3".into(),
            source_file: "structural_kinetics.rs".into(),
            function: "structure_production_rate".into(),
            local_basis: "A * interface_weight(φ) (default)".into(),
            spatial_weighting: "interface-limited (default); interior for PhaseVolume".into(),
            unit_stoichiometry: "A cost = produced/η_φ (virtual ledger)".into(),
            produced_material: "φ (when enforce_structure_constraint=false)".into(),
            w_production: "structure decay → W separately".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
        ASinkLineage {
            id: "L_precursor".into(),
            kind: ASinkKind::Precursor,
            equation: "r_P = k_precursor * A * q(C) * H(φ); q(C)=C/(k_c_membrane+C); A→P".into(),
            equation_version: "v6+ membrane precursor path".into(),
            source_file: "membrane.rs".into(),
            function: "precursor_synthesis_rate".into(),
            local_basis: "A * q(C) * H(φ)".into(),
            spatial_weighting: "interior_weight H(φ)".into(),
            unit_stoichiometry: "1 A → 1 P".into(),
            produced_material: "P".into(),
            w_production: "none on synthesis; P→W via precursor_decay".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
        ASinkLineage {
            id: "L_membrane".into(),
            kind: ASinkKind::Membrane,
            equation: "schema-3: no constitutive S→W; mature maturation A paths disabled under v8".into(),
            equation_version: "surface_turnover_schema_3_exchange_damage_only".into(),
            source_file: "surface_density.rs / d039_analysis.rs".into(),
            function: "apply_schema3_exchange_damage_only".into(),
            local_basis: "n/a (zero constitutive A→S)".into(),
            spatial_weighting: "n/a".into(),
            unit_stoichiometry: "0 under schema 3 constitutive".into(),
            produced_material: "none constitutive".into(),
            w_production: "damage/loss paths only".into(),
            enabled_fixed_compartment: false,
            enabled_true_radius: false,
            enabled_constrained_radius: false,
            enabled_damage: false,
            in_d042_d045_ledgers: false,
        },
        ASinkLineage {
            id: "L_decay".into(),
            kind: ASinkKind::Decay,
            equation: "r_decay = k_d008_activated_decay * A; A→W".into(),
            equation_version: "activated_metabolism".into(),
            source_file: "activated_metabolism.rs".into(),
            function: "activated_metabolism_rates".into(),
            local_basis: "A".into(),
            spatial_weighting: "per-cell bulk".into(),
            unit_stoichiometry: "1 A → 1 W".into(),
            produced_material: "W".into(),
            w_production: "1 per A".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
        ASinkLineage {
            id: "L_transport".into(),
            kind: ASinkKind::Transport,
            equation: "selective face transport of A (historical ρ_A)".into(),
            equation_version: "membrane_transport".into(),
            source_file: "membrane_transport.rs".into(),
            function: "evolve_membrane_transport".into(),
            local_basis: "interface fluxes".into(),
            spatial_weighting: "membrane faces".into(),
            unit_stoichiometry: "conservative relocation".into(),
            produced_material: "none".into(),
            w_production: "none".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
        ASinkLineage {
            id: "L_other_reservoir".into(),
            kind: ASinkKind::Reservoir,
            equation: "reservoir mask exchange (N/F; A typically not reservoir-sourced)".into(),
            equation_version: "reservoir.rs".into(),
            source_file: "reservoir.rs".into(),
            function: "apply_reservoir".into(),
            local_basis: "mask".into(),
            spatial_weighting: "exterior mask".into(),
            unit_stoichiometry: "ledger delta".into(),
            produced_material: "none".into(),
            w_production: "none".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
        ASinkLineage {
            id: "L_other_numerical".into(),
            kind: ASinkKind::Numerical,
            equation: "nonnegativity / clamp corrections on A".into(),
            equation_version: "accounting".into(),
            source_file: "accounting.rs / simulation.rs".into(),
            function: "FieldStepLedger.numerical_correction_delta".into(),
            local_basis: "clamp residual".into(),
            spatial_weighting: "per-cell".into(),
            unit_stoichiometry: "correction only".into(),
            produced_material: "none".into(),
            w_production: "none".into(),
            enabled_fixed_compartment: true,
            enabled_true_radius: true,
            enabled_constrained_radius: true,
            enabled_damage: true,
            in_d042_d045_ledgers: true,
        },
    ]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ADemandDecomposition {
    pub l_rep: f64,
    pub l_structure: f64,
    pub l_precursor: f64,
    pub l_membrane: f64,
    pub l_decay: f64,
    pub l_transport: f64,
    pub l_other: f64,
    pub l_total: f64,
    pub residual: f64,
}

impl ADemandDecomposition {
    pub fn from_rates(
        j_rep: f64,
        j_struct: f64,
        j_prec: f64,
        j_membrane: f64,
        j_decay: f64,
        j_out: f64,
        j_in: f64,
        other: f64,
        observed_l_a: f64,
    ) -> Self {
        let l_transport = (j_out - j_in).max(0.0);
        let l_total =
            j_rep + j_struct + j_prec + j_membrane + j_decay + l_transport + other;
        Self {
            l_rep: j_rep,
            l_structure: j_struct,
            l_precursor: j_prec,
            l_membrane: j_membrane,
            l_decay: j_decay,
            l_transport,
            l_other: other,
            l_total,
            residual: observed_l_a - l_total,
        }
    }

    pub fn residual_ok(self, tol: f64) -> bool {
        let scale = self.l_total.abs().max(1.0);
        self.residual.abs() / scale <= tol
    }

    pub fn dominant_sink(self) -> ASinkKind {
        let parts = [
            (ASinkKind::Reproduction, self.l_rep),
            (ASinkKind::Structure, self.l_structure),
            (ASinkKind::Precursor, self.l_precursor),
            (ASinkKind::Membrane, self.l_membrane),
            (ASinkKind::Decay, self.l_decay),
            (ASinkKind::Transport, self.l_transport),
            (ASinkKind::Other, self.l_other),
        ];
        parts
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k)
            .unwrap_or(ASinkKind::Other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstraintDemandClass {
    FullyBiological,
    BiologicalObserverMeasurement,
    ConstraintContaminatedSeparable,
    InvalidForDemandTopology,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintAuditItem {
    pub campaign: String,
    pub feature: String,
    pub class: ConstraintDemandClass,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemandScaleClass {
    CatalystScaled,
    InteriorVolumeScaled,
    StructuralMassScaled,
    InterfaceScaled,
    PrecursorStateScaled,
    ConstantBackground,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElasticityReport {
    pub sink: String,
    pub eps_c: Option<f64>,
    pub eps_v: Option<f64>,
    pub eps_phi: Option<f64>,
    pub eps_s: Option<f64>,
    pub eps_p: Option<f64>,
    pub class: DemandScaleClass,
    pub bootstrap_lo: Option<f64>,
    pub bootstrap_hi: Option<f64>,
    pub loo_stable: bool,
}

/// Log elasticity ∂ln y / ∂ln x from paired finite samples (needs ≥3 points for class).
pub fn log_elasticity(xs: &[f64], ys: &[f64]) -> Option<f64> {
    if xs.len() < 2 || ys.len() != xs.len() {
        return None;
    }
    let mut num = 0.0;
    let mut den = 0.0;
    let mut n = 0u32;
    for i in 0..xs.len() {
        for j in (i + 1)..xs.len() {
            if xs[i] > 1e-18 && xs[j] > 1e-18 && ys[i] > 1e-18 && ys[j] > 1e-18 {
                let dx = (xs[j] / xs[i]).ln();
                let dy = (ys[j] / ys[i]).ln();
                if dx.abs() > 1e-12 {
                    num += dy / dx;
                    den += 1.0;
                    n += 1;
                }
            }
        }
    }
    if n == 0 || den <= 0.0 {
        None
    } else {
        Some(num / den)
    }
}

pub fn classify_elasticity(eps_c: Option<f64>, eps_v: Option<f64>, eps_phi: Option<f64>) -> DemandScaleClass {
    let abs = |o: Option<f64>| o.unwrap_or(0.0).abs();
    let c = abs(eps_c);
    let v = abs(eps_v);
    let p = abs(eps_phi);
    let max = c.max(v).max(p);
    if max < 0.15 {
        return DemandScaleClass::ConstantBackground;
    }
    let near = |a: f64, t: f64| (a - t).abs() < 0.35;
    if v >= c && v >= p && near(eps_v.unwrap_or(0.0), 1.0) {
        return DemandScaleClass::InteriorVolumeScaled;
    }
    if c >= v && c >= p && near(eps_c.unwrap_or(0.0), 1.0) {
        return DemandScaleClass::CatalystScaled;
    }
    if p >= v && p >= c && near(eps_phi.unwrap_or(0.0), 1.0) {
        return DemandScaleClass::StructuralMassScaled;
    }
    if c > 0.15 && v > 0.15 {
        return DemandScaleClass::Mixed;
    }
    if v >= c {
        DemandScaleClass::InteriorVolumeScaled
    } else if c >= p {
        DemandScaleClass::CatalystScaled
    } else {
        DemandScaleClass::StructuralMassScaled
    }
}

/// Leave-one-out stability: all leave-one elasticities within factor of reference.
pub fn elasticity_loo_stable(xs: &[f64], ys: &[f64], factor: f64) -> bool {
    if xs.len() < 3 {
        return false;
    }
    let Some(base) = log_elasticity(xs, ys) else {
        return false;
    };
    if base.abs() < 1e-9 {
        return true;
    }
    for drop in 0..xs.len() {
        let mut xx = Vec::with_capacity(xs.len() - 1);
        let mut yy = Vec::with_capacity(ys.len() - 1);
        for i in 0..xs.len() {
            if i != drop {
                xx.push(xs[i]);
                yy.push(ys[i]);
            }
        }
        let Some(e) = log_elasticity(&xx, &yy) else {
            return false;
        };
        if (e / base).abs() > factor || (base / e).abs() > factor {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YieldClass {
    ValidProductiveCost,
    ValidMaintenanceCost,
    GrowthOnlyCost,
    RepairOnlyCost,
    ConstraintArtifact,
    DuplicatedCost,
    StoichiometryUnsupported,
    SaturationWaste,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YieldAuditRow {
    pub sink: String,
    pub a_consumed: f64,
    pub product_formed: f64,
    pub w_formed: f64,
    pub product_per_a: f64,
    pub w_per_a: f64,
    pub class: YieldClass,
    pub note: String,
}

pub fn classify_yield(
    a_consumed: f64,
    product_formed: f64,
    expected_product_per_a: f64,
    duplicated: bool,
    constraint_only: bool,
) -> YieldClass {
    if constraint_only {
        return YieldClass::ConstraintArtifact;
    }
    if duplicated {
        return YieldClass::DuplicatedCost;
    }
    if a_consumed <= 1e-18 {
        return if product_formed > 1e-12 {
            YieldClass::StoichiometryUnsupported
        } else {
            YieldClass::ValidMaintenanceCost
        };
    }
    let ratio = product_formed / a_consumed;
    if expected_product_per_a > 0.0 {
        let rel = ((ratio - expected_product_per_a) / expected_product_per_a).abs();
        if rel <= 0.15 {
            return YieldClass::ValidProductiveCost;
        }
        return YieldClass::StoichiometryUnsupported;
    }
    if product_formed <= 1e-12 {
        YieldClass::SaturationWaste
    } else {
        YieldClass::ValidProductiveCost
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DemandStateRow {
    pub label: String,
    pub family: String,
    pub train: bool,
    pub radius: f64,
    pub c: f64,
    pub n: f64,
    pub f: f64,
    pub a: f64,
    pub p: f64,
    pub s_occupancy: f64,
    pub m_c: f64,
    pub interior_volume: f64,
    pub structural_mass: f64,
    pub membrane_area: f64,
    pub l_a: f64,
    pub j_reproduction: f64,
    pub j_structural: f64,
    pub j_precursor: f64,
    pub j_decay: f64,
    pub j_out: f64,
    pub j_in: f64,
    pub k_structure_scale: f64,
    pub k_precursor_scale: f64,
}

fn through_origin_alpha(xs: &[f64], ys: &[f64]) -> f64 {
    let mut xx = 0.0;
    let mut xy = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        xx += x * x;
        xy += x * y;
    }
    if xx <= 1e-18 {
        0.0
    } else {
        xy / xx
    }
}

fn max_rel_err(xs: &[f64], ys: &[f64], alpha: f64) -> f64 {
    let mut max_err = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        if y > 1e-18 {
            max_err = f64::max(max_err, ((y - alpha * x) / y).abs());
        }
    }
    max_err
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::INFINITY;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelFitReport {
    pub name: String,
    pub lambda: f64,
    pub median_hold_err: f64,
    pub max_hold_err: f64,
    pub radius_bias: bool,
    pub catalyst_bias: bool,
    pub starvation_false_positive: bool,
    pub bootstrap_spread: f64,
    pub loo_factor_ok: bool,
    pub adequate: bool,
}

/// Model A: L_A ≈ λ_C M_C
pub fn fit_model_a(train: &[DemandStateRow], hold: &[DemandStateRow]) -> ModelFitReport {
    let xs: Vec<f64> = train.iter().map(|r| r.m_c).collect();
    let ys: Vec<f64> = train.iter().map(|r| r.l_a).collect();
    let lambda = through_origin_alpha(&xs, &ys);
    evaluate_model("A_catalyst_linear", lambda, hold, |r, lam| lam * r.m_c, train)
}

/// Model B: L_A ≈ λ_V V
pub fn fit_model_b(train: &[DemandStateRow], hold: &[DemandStateRow]) -> ModelFitReport {
    let xs: Vec<f64> = train.iter().map(|r| r.interior_volume).collect();
    let ys: Vec<f64> = train.iter().map(|r| r.l_a).collect();
    let lambda = through_origin_alpha(&xs, &ys);
    evaluate_model(
        "B_interior_volume",
        lambda,
        hold,
        |r, lam| lam * r.interior_volume,
        train,
    )
}

/// Model C: L_A ≈ V_A ∫ H(φ) q(C) dV ≈ λ * V * q(C_mean) under uniform clamp.
pub fn fit_model_c(train: &[DemandStateRow], hold: &[DemandStateRow], k_c: f64) -> ModelFitReport {
    let xs: Vec<f64> = train
        .iter()
        .map(|r| r.interior_volume * r.c / f64::max(k_c + r.c, 1e-18))
        .collect();
    let ys: Vec<f64> = train.iter().map(|r| r.l_a).collect();
    let lambda = through_origin_alpha(&xs, &ys);
    evaluate_model(
        "C_catalyst_saturating_volume",
        lambda,
        hold,
        move |r, lam| lam * r.interior_volume * r.c / f64::max(k_c + r.c, 1e-18),
        train,
    )
}

/// Model D: mechanistic sink sum using measured reaction bases proxies.
pub fn fit_model_d(_train: &[DemandStateRow], hold: &[DemandStateRow]) -> ModelFitReport {
    // Identity check: predicted = sum of measured sinks on hold.
    let mut errs = Vec::new();
    for r in hold {
        let pred = r.j_reproduction + r.j_structural + r.j_precursor + r.j_decay
            + (r.j_out - r.j_in).max(0.0);
        if r.l_a > 1e-18 {
            errs.push(((r.l_a - pred) / r.l_a).abs());
        }
    }
    let med = median(errs.clone());
    let maxe = errs.into_iter().fold(0.0_f64, f64::max);
    ModelFitReport {
        name: "D_mechanistic_sink_sum".into(),
        lambda: 1.0,
        median_hold_err: med,
        max_hold_err: maxe,
        radius_bias: false,
        catalyst_bias: false,
        starvation_false_positive: false,
        bootstrap_spread: 0.0,
        loo_factor_ok: true,
        adequate: med <= D046_MODEL_MEDIAN_HOLD_ERR && maxe <= D046_MODEL_MAX_HOLD_ERR,
    }
}

fn evaluate_model<F>(
    name: &str,
    lambda: f64,
    hold: &[DemandStateRow],
    pred: F,
    train: &[DemandStateRow],
) -> ModelFitReport
where
    F: Fn(&DemandStateRow, f64) -> f64,
{
    let mut errs = Vec::new();
    for r in hold {
        let p = pred(r, lambda);
        if r.l_a > 1e-18 {
            errs.push(((r.l_a - p) / r.l_a).abs());
        }
    }
    let med = median(errs.clone());
    let maxe = errs.iter().copied().fold(0.0_f64, f64::max);

    // Systematic radius bias: mean signed error R16 vs R32.
    let err_at = |lab: &str| {
        hold.iter()
            .find(|r| r.label == lab)
            .map(|r| {
                let p = pred(r, lambda);
                if r.l_a > 1e-18 {
                    (r.l_a - p) / r.l_a
                } else {
                    0.0
                }
            })
    };
    let radius_bias = match (err_at("R16"), err_at("R32")) {
        (Some(a), Some(b)) => a * b < 0.0 && (a - b).abs() > 0.15,
        _ => false,
    };
    let catalyst_bias = match (err_at("low_c"), err_at("high_c")) {
        (Some(a), Some(b)) => a * b < 0.0 && (a - b).abs() > 0.15,
        _ => false,
    };

    // Bootstrap spread via leave-one-out λ.
    let mut lambdas = Vec::new();
    for drop in 0..train.len() {
        let xs: Vec<f64> = train
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != drop)
            .map(|(_, r)| {
                if name.starts_with('A') {
                    r.m_c
                } else if name.starts_with('B') {
                    r.interior_volume
                } else if name.starts_with('C') {
                    r.interior_volume * r.c / f64::max(0.10 + r.c, 1e-18)
                } else {
                    1.0
                }
            })
            .collect();
        let ys: Vec<f64> = train
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != drop)
            .map(|(_, r)| r.l_a)
            .collect();
        lambdas.push(through_origin_alpha(&xs, &ys));
    }
    let (lo, hi) = lambdas
        .iter()
        .copied()
        .fold((f64::INFINITY, 0.0_f64), |(lo, hi), v| (lo.min(v), hi.max(v)));
    let spread = if lambda.abs() > 1e-18 {
        (hi - lo) / lambda.abs()
    } else {
        f64::INFINITY
    };
    let loo_factor_ok = lo > 1e-18 && hi / lo <= D046_MODEL_LOO_FACTOR;

    let starvation_false_positive = hold.iter().any(|r| {
        (r.c < 1e-6 || r.n < 1e-6 || r.f < 1e-6) && pred(r, lambda) > 1e-6 && r.l_a < 1e-6
    });

    let adequate = med <= D046_MODEL_MEDIAN_HOLD_ERR
        && maxe <= D046_MODEL_MAX_HOLD_ERR
        && !radius_bias
        && !catalyst_bias
        && !starvation_false_positive
        && spread <= D046_MODEL_BOOTSTRAP_SPREAD
        && loo_factor_ok;

    let _ = max_rel_err;
    ModelFitReport {
        name: name.into(),
        lambda,
        median_hold_err: med,
        max_hold_err: maxe,
        radius_bias,
        catalyst_bias,
        starvation_false_positive,
        bootstrap_spread: spread,
        loo_factor_ok,
        adequate,
    }
}

/// Historical CNF basis integral proxy under uniform clamps: V * C * N * F.
pub fn basis_historical(r: &DemandStateRow) -> f64 {
    r.interior_volume * r.c.max(0.0) * r.n.max(0.0) * r.f.max(0.0)
}

/// Catalyst-saturating volumetric: V * q(C) * N * F.
pub fn basis_saturating_volumetric(r: &DemandStateRow, k_c: f64) -> f64 {
    let q = r.c.max(0.0) / f64::max(k_c + r.c.max(0.0), 1e-18);
    r.interior_volume * q * r.n.max(0.0) * r.f.max(0.0)
}

/// Joint-substrate saturating: V * q(C) * z/(K_NF+z), z=n*f.
pub fn basis_saturating_joint(r: &DemandStateRow, k_c: f64, k_nf: f64) -> f64 {
    let q = r.c.max(0.0) / f64::max(k_c + r.c.max(0.0), 1e-18);
    let z = r.n.max(0.0) * r.f.max(0.0);
    let sat = z / f64::max(k_nf + z, 1e-18);
    r.interior_volume * q * sat
}

pub fn basis_zero_resource_controls(k_c: f64, k_nf: f64) -> bool {
    let zc = DemandStateRow {
        label: "z".into(),
        family: "ctrl".into(),
        train: false,
        radius: 22.0,
        c: 0.0,
        n: 0.8,
        f: 0.8,
        a: 0.5,
        p: 0.05,
        s_occupancy: 0.6,
        m_c: 0.0,
        interior_volume: 1500.0,
        structural_mass: 1600.0,
        membrane_area: 140.0,
        l_a: 0.0,
        j_reproduction: 0.0,
        j_structural: 0.0,
        j_precursor: 0.0,
        j_decay: 0.0,
        j_out: 0.0,
        j_in: 0.0,
        k_structure_scale: 1.0,
        k_precursor_scale: 1.0,
    };
    let mut zn = zc.clone();
    zn.c = 0.8;
    zn.n = 0.0;
    let mut zf = zc.clone();
    zf.c = 0.8;
    zf.f = 0.0;
    basis_historical(&zc) == 0.0
        && basis_historical(&zn) == 0.0
        && basis_historical(&zf) == 0.0
        && basis_saturating_volumetric(&zc, k_c) == 0.0
        && basis_saturating_volumetric(&zn, k_c) == 0.0
        && basis_saturating_volumetric(&zf, k_c) == 0.0
        && basis_saturating_joint(&zc, k_c, k_nf) == 0.0
        && basis_saturating_joint(&zn, k_c, k_nf) == 0.0
        && basis_saturating_joint(&zf, k_c, k_nf) == 0.0
}

pub fn fit_basis_to_demand(
    name: &str,
    train: &[DemandStateRow],
    hold: &[DemandStateRow],
    basis_fn: impl Fn(&DemandStateRow) -> f64,
) -> ModelFitReport {
    let xs: Vec<f64> = train.iter().map(&basis_fn).collect();
    let ys: Vec<f64> = train.iter().map(|r| r.l_a).collect();
    let lambda = through_origin_alpha(&xs, &ys);
    evaluate_model(name, lambda, hold, |r, lam| lam * basis_fn(r), train)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteDecisionInput {
    pub accounting_defect: bool,
    pub constraint_contaminated: bool,
    pub structural_defect: bool,
    pub precursor_defect: bool,
    pub reproduction_defect: bool,
    pub all_sinks_valid: bool,
    pub volume_dominant: bool,
    pub catalyst_saturating: bool,
    pub model_c_adequate: bool,
    pub basis_b_adequate: bool,
    pub mixed_no_single_basis: bool,
}

pub fn select_route(input: &RouteDecisionInput) -> D046Route {
    if input.accounting_defect {
        return D046Route::RouteA;
    }
    if input.constraint_contaminated {
        return D046Route::RouteC;
    }
    if input.structural_defect {
        return D046Route::RouteS;
    }
    if input.precursor_defect {
        return D046Route::RouteP;
    }
    if input.reproduction_defect {
        return D046Route::RouteR;
    }
    if input.all_sinks_valid
        && input.volume_dominant
        && input.catalyst_saturating
        && input.model_c_adequate
        && input.basis_b_adequate
    {
        return D046Route::RouteV;
    }
    if input.all_sinks_valid && input.mixed_no_single_basis {
        return D046Route::RouteM;
    }
    D046Route::RouteI
}

/// Preregistered training / holdout split for Gate 4–8 (frozen before campaign).
pub fn preregistered_split(label: &str) -> bool {
    // Train: R16, R22, low_c, med_c, struct_lo, prec_lo, s_healthy
    // Hold: R32, high_c, struct_hi, prec_hi, s_low, s_damaged25
    matches!(
        label,
        "R16" | "R22" | "low_c" | "med_c" | "struct_lo" | "prec_lo" | "s_healthy"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_provisional_when_impl_only() {
        let p = classify_d045_threshold_provenance(false, true, false, false);
        assert_eq!(p, D045ThresholdProvenance::ImplementationBeforeEvidence);
        assert_eq!(
            p.rejection_status(),
            "D045_CATALYST_LINEARITY_REJECTION_PROVISIONAL"
        );
    }

    #[test]
    fn provenance_governed_when_directive() {
        let p = classify_d045_threshold_provenance(true, true, false, false);
        assert_eq!(p, D045ThresholdProvenance::PreregisteredGoverned);
    }

    #[test]
    fn lineage_catalog_covers_decomposition() {
        let cat = a_demand_lineage_catalog();
        assert!(cat.iter().any(|s| s.id == "L_rep"));
        assert!(cat.iter().any(|s| s.id == "L_structure"));
        assert!(cat.iter().any(|s| s.id == "L_precursor"));
        assert!(cat.iter().any(|s| s.id == "L_decay"));
        assert!(cat.iter().any(|s| s.id == "L_transport"));
    }

    #[test]
    fn decomposition_residual_and_dominant() {
        let d = ADemandDecomposition::from_rates(10.0, 18.0, 135.0, 0.0, 4.0, 1.0, 0.0, 0.0, 168.0);
        assert!(d.residual_ok(0.01));
        assert_eq!(d.dominant_sink(), ASinkKind::Precursor);
    }

    #[test]
    fn elasticity_volume_class() {
        let v = [800.0, 1500.0, 3200.0];
        let y = [80.0, 150.0, 320.0];
        let e = log_elasticity(&v, &y).unwrap();
        assert!((e - 1.0).abs() < 0.05);
        assert_eq!(
            classify_elasticity(Some(0.2), Some(e), Some(0.1)),
            DemandScaleClass::InteriorVolumeScaled
        );
        assert!(elasticity_loo_stable(&v, &y, 2.0));
    }

    #[test]
    fn yield_stoichiometry() {
        assert_eq!(
            classify_yield(10.0, 10.0, 1.0, false, false),
            YieldClass::ValidProductiveCost
        );
        assert_eq!(
            classify_yield(10.0, 0.0, 1.0, false, false),
            YieldClass::StoichiometryUnsupported
        );
        assert_eq!(
            classify_yield(10.0, 10.0, 1.0, true, false),
            YieldClass::DuplicatedCost
        );
    }

    #[test]
    fn route_v_selection() {
        let r = select_route(&RouteDecisionInput {
            accounting_defect: false,
            constraint_contaminated: false,
            structural_defect: false,
            precursor_defect: false,
            reproduction_defect: false,
            all_sinks_valid: true,
            volume_dominant: true,
            catalyst_saturating: true,
            model_c_adequate: true,
            basis_b_adequate: true,
            mixed_no_single_basis: false,
        });
        assert_eq!(r, D046Route::RouteV);
        assert_eq!(
            r.conclusion().as_str(),
            "D046_CATALYST_SATURATING_VOLUME_ACTIVATION_JUSTIFIED"
        );
    }

    #[test]
    fn route_a_priority() {
        let r = select_route(&RouteDecisionInput {
            accounting_defect: true,
            constraint_contaminated: true,
            structural_defect: true,
            precursor_defect: true,
            reproduction_defect: true,
            all_sinks_valid: false,
            volume_dominant: false,
            catalyst_saturating: false,
            model_c_adequate: false,
            basis_b_adequate: false,
            mixed_no_single_basis: false,
        });
        assert_eq!(r, D046Route::RouteA);
    }

    #[test]
    fn zero_resource_bases() {
        assert!(basis_zero_resource_controls(0.10, 1.0));
    }

    #[test]
    fn no_observer_feedback_in_basis() {
        // Bases use only local C,N,F,V — no θ target or observer term.
        let r = DemandStateRow {
            label: "x".into(),
            family: "t".into(),
            train: true,
            radius: 22.0,
            c: 0.8,
            n: 0.8,
            f: 0.8,
            a: 0.5,
            p: 0.05,
            s_occupancy: 0.99,
            m_c: 1000.0,
            interior_volume: 1500.0,
            structural_mass: 1600.0,
            membrane_area: 140.0,
            l_a: 100.0,
            j_reproduction: 10.0,
            j_structural: 10.0,
            j_precursor: 70.0,
            j_decay: 5.0,
            j_out: 5.0,
            j_in: 0.0,
            k_structure_scale: 1.0,
            k_precursor_scale: 1.0,
        };
        let mut r2 = r.clone();
        r2.s_occupancy = 0.01;
        assert_eq!(
            basis_saturating_volumetric(&r, 0.1),
            basis_saturating_volumetric(&r2, 0.1)
        );
    }
}
