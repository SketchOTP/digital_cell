//! D-053 combined exterior + membrane N/F resource-delivery repair.
//!
//! Authorized parameters only: `m_ext` (exterior–exterior N/F conductance) and
//! `m_beta` (co-scaled N/F membrane attenuation). Activation and C/A/W frozen.

use crate::config::SimParams;
use crate::membrane_transport::{
    exterior_face_weight, exterior_resource_diffusivity_factor, face_diffusivity, species_beta,
    TransportSpecies,
};
use serde::{Deserialize, Serialize};

pub const D053_PROJECT_ID: &str = "D-053";
pub const D053_AGENT_MEMORY_ID: &str = "D-20260721-d053-combined-resource-delivery-repair";
pub const D053_STARTING_COMMIT: &str = "20c5d50";
pub const D053_STARTING_TAG: &str = "D-052-resource-delivery-resistance-audit";
pub const D053_FROZEN_D051: &str = "D051_RESOURCE_THROUGHPUT_LIMIT";
pub const D053_FROZEN_D052: &str = "D052_MIXED_RESOURCE_DELIVERY_LIMIT";
pub const D053_AUTHORIZATION: &str = "MIXED_NF_DELIVERY_REPAIR_AUTHORIZED";
pub const D053_ARCHITECTURE: &str = "V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED";
pub const D053_ARCHITECTURE_LEGACY_LABEL: &str = "V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_REPAIR";
pub const D053_EXHAUSTION: &str = "BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED";

pub const D053_FITTED_V_A: f64 = 0.12544510052968755;
pub const D053_FITTED_K_C: f64 = 0.10;
pub const D053_N_REF: f64 = 1.0;
pub const D053_F_REF: f64 = 1.0;
pub const D053_RADIUS: f64 = 22.0;
pub const D053_THETA: f64 = 0.6;
pub const D053_DEFAULT_HORIZON: u64 = 10_000;
pub const D053_WINDOW: u64 = 10_000;
pub const D053_CHI_MIN: f64 = 1.05;
pub const D053_RETENTION_MIN: f64 = 0.80;
pub const D053_LOCALIZATION_MIN: f64 = 0.95;
pub const D053_NET_S_TOL: f64 = 1.0e-4;
pub const D053_STAGE_A_NF_PERM_LO: f64 = 0.20;
pub const D053_STAGE_A_NF_PERM_HI: f64 = 0.50;
pub const D053_M_EXT_LO: f64 = 1.0;
pub const D053_M_EXT_HI: f64 = 4.0;
pub const D053_M_BETA_LO: f64 = 0.50;
pub const D053_M_BETA_HI: f64 = 1.0;
pub const D053_MAX_CANDIDATES: usize = 6;
pub const D053_RESISTANCE_TOL: f64 = 0.08;
pub const D053_EXT_RESISTANCE_REF: f64 = 0.43;
pub const D053_MEM_RESISTANCE_REF: f64 = 0.37;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D053PrimaryConclusion {
    StageERecovered,
    ResourceDeliveryRepairQualifiedStageEBlocked,
    D052MixedLimitNotReproduced,
    TransportIsolationFailure,
    CombinedDeliveryNotIdentifiable,
    CombinedDeliveryRepairNotSupported,
    BoundedDeliveryRepairNotFound,
    ResourceTransportNumericalFailure,
    StageASelectivityRegression,
    FixedCompartmentResourceRegression,
    NoHealthyResourceRepairedAttractor,
    ResourceRepairedBasinFailure,
    ContinuousReplacementFailure,
    DamageRepairFailure,
    RepairResourceDependenceFailure,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D053PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StageERecovered => "D053_STAGE_E_RECOVERED",
            Self::ResourceDeliveryRepairQualifiedStageEBlocked => {
                "D053_RESOURCE_DELIVERY_REPAIR_QUALIFIED_STAGE_E_BLOCKED"
            }
            Self::D052MixedLimitNotReproduced => "D053_D052_MIXED_LIMIT_NOT_REPRODUCED",
            Self::TransportIsolationFailure => "D053_TRANSPORT_ISOLATION_FAILURE",
            Self::CombinedDeliveryNotIdentifiable => "D053_COMBINED_DELIVERY_NOT_IDENTIFIABLE",
            Self::CombinedDeliveryRepairNotSupported => "D053_COMBINED_DELIVERY_REPAIR_NOT_SUPPORTED",
            Self::BoundedDeliveryRepairNotFound => "D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND",
            Self::ResourceTransportNumericalFailure => "D053_RESOURCE_TRANSPORT_NUMERICAL_FAILURE",
            Self::StageASelectivityRegression => "D053_STAGE_A_SELECTIVITY_REGRESSION",
            Self::FixedCompartmentResourceRegression => "D053_FIXED_COMPARTMENT_RESOURCE_REGRESSION",
            Self::NoHealthyResourceRepairedAttractor => "D053_NO_HEALTHY_RESOURCE_REPAIRED_ATTRACTOR",
            Self::ResourceRepairedBasinFailure => "D053_RESOURCE_REPAIRED_BASIN_FAILURE",
            Self::ContinuousReplacementFailure => "D053_CONTINUOUS_REPLACEMENT_FAILURE",
            Self::DamageRepairFailure => "D053_DAMAGE_REPAIR_FAILURE",
            Self::RepairResourceDependenceFailure => "D053_REPAIR_RESOURCE_DEPENDENCE_FAILURE",
            Self::StageEMembraneContractFailure => "D053_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D053_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D053_NUMERICAL_FAILURE",
            Self::Fail => "D053_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DeliveryRepairPair {
    pub m_ext: f64,
    pub m_beta: f64,
}

impl DeliveryRepairPair {
    pub const BASELINE: Self = Self {
        m_ext: 1.0,
        m_beta: 1.0,
    };

    pub fn delta_norm(self) -> f64 {
        let dx = (self.m_ext.ln()).abs();
        let dy = ((1.0 / self.m_beta.max(1e-18)).ln()).abs();
        (dx * dx + dy * dy).sqrt()
    }

    pub fn is_authorized(self) -> bool {
        self.m_ext >= D053_M_EXT_LO - 1e-12
            && self.m_ext <= D053_M_EXT_HI + 1e-12
            && self.m_beta >= D053_M_BETA_LO - 1e-12
            && self.m_beta <= D053_M_BETA_HI + 1e-12
    }
}

/// Apply authorized delivery repair to params (does not touch activation / C/A/W betas).
pub fn apply_delivery_repair(params: &mut SimParams, pair: DeliveryRepairPair) {
    params.m_ext = pair.m_ext;
    params.m_beta = pair.m_beta;
}

/// Stage A normalized N/F permeability at unit occupancy: Π = exp(−m_β β).
pub fn nf_permeability_normalized(beta: f64, m_beta: f64) -> f64 {
    (-(beta.max(0.0) * m_beta.max(0.0))).exp()
}

pub fn stage_a_nf_band_ok(perm: f64) -> bool {
    (D053_STAGE_A_NF_PERM_LO..=D053_STAGE_A_NF_PERM_HI).contains(&perm)
}

/// Minimum m_β keeping Π ≤ 0.50 at given β and θ=1.
pub fn m_beta_min_for_upper_band(beta: f64) -> f64 {
    let b = beta.max(1e-18);
    ((0.50_f64.ln().abs()) / b).clamp(D053_M_BETA_LO, D053_M_BETA_HI)
}

pub fn pair_stage_a_authorized(pair: DeliveryRepairPair, beta_n: f64, beta_f: f64) -> bool {
    if !pair.is_authorized() {
        return false;
    }
    stage_a_nf_band_ok(nf_permeability_normalized(beta_n, pair.m_beta))
        && stage_a_nf_band_ok(nf_permeability_normalized(beta_f, pair.m_beta))
}

/// Exterior-only face identification: g_ext=1 iff both endpoints extracellular.
pub fn is_exterior_exterior_face(phi_i: f64, phi_j: f64) -> bool {
    exterior_face_weight(phi_i, phi_j) > 0.5
}

/// Face diffusivity ratio vs baseline for isolation proofs.
pub fn face_d_ratio(
    species: TransportSpecies,
    phi_i: f64,
    phi_j: f64,
    membrane_i: f64,
    membrane_j: f64,
    baseline: &SimParams,
    probe: &SimParams,
) -> f64 {
    let d0 = face_diffusivity(species, phi_i, phi_j, membrane_i, membrane_j, baseline);
    let d1 = face_diffusivity(species, phi_i, phi_j, membrane_i, membrane_j, probe);
    d1 / d0.max(1e-18)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TransportIsolationReport {
    pub exterior_only_changes_exterior_nf: bool,
    pub exterior_only_preserves_membrane_nf: bool,
    pub exterior_only_preserves_interior_nf: bool,
    pub exterior_only_preserves_caw: bool,
    pub membrane_only_changes_crossing_nf: bool,
    pub membrane_only_preserves_exterior_base: bool,
    pub membrane_only_preserves_caw: bool,
    pub combined_composes: bool,
    pub face_symmetric: bool,
}

pub fn prove_transport_isolation(base: &SimParams) -> TransportIsolationReport {
    let mut exterior = base.clone();
    exterior.m_ext = 2.0;
    exterior.m_beta = 1.0;
    let mut membrane = base.clone();
    membrane.m_ext = 1.0;
    membrane.m_beta = 0.75;
    let mut combined = base.clone();
    combined.m_ext = 2.0;
    combined.m_beta = 0.75;

    // Exterior–exterior face (both outside).
    let (pe, qe) = (0.1, 0.2);
    // Interior–interior.
    let (pi, qi) = (0.8, 0.9);
    // Membrane-crossing.
    let (pc, qc) = (0.2, 0.8);
    let m0 = 0.0;
    let m1 = 1.0;

    let r_ext_n = face_d_ratio(TransportSpecies::Nutrient, pe, qe, m0, m0, base, &exterior);
    let r_cross_n = face_d_ratio(TransportSpecies::Nutrient, pc, qc, m1, m1, base, &exterior);
    let r_int_n = face_d_ratio(TransportSpecies::Nutrient, pi, qi, m0, m0, base, &exterior);
    let r_c = face_d_ratio(TransportSpecies::Catalyst, pe, qe, m0, m0, base, &exterior);
    let r_a = face_d_ratio(TransportSpecies::Activated, pc, qc, m1, m1, base, &exterior);
    let r_w = face_d_ratio(TransportSpecies::Waste, pc, qc, m1, m1, base, &exterior);

    let m_cross = face_d_ratio(TransportSpecies::Nutrient, pc, qc, m1, m1, base, &membrane);
    let m_ext = face_d_ratio(TransportSpecies::Nutrient, pe, qe, m0, m0, base, &membrane);
    let m_c = face_d_ratio(TransportSpecies::Catalyst, pc, qc, m1, m1, base, &membrane);

    let c_cross = face_d_ratio(TransportSpecies::Nutrient, pc, qc, m1, m1, base, &combined);
    let c_ext = face_d_ratio(TransportSpecies::Nutrient, pe, qe, m0, m0, base, &combined);
    let expected_cross = face_d_ratio(TransportSpecies::Nutrient, pc, qc, m1, m1, base, &membrane);
    let expected_ext = face_d_ratio(TransportSpecies::Nutrient, pe, qe, m0, m0, base, &exterior);

    let d_fwd = face_diffusivity(TransportSpecies::Nutrient, pe, qe, m0, m0, &combined);
    let d_rev = face_diffusivity(TransportSpecies::Nutrient, qe, pe, m0, m0, &combined);

    TransportIsolationReport {
        exterior_only_changes_exterior_nf: (r_ext_n - 2.0).abs() < 1e-12,
        exterior_only_preserves_membrane_nf: (r_cross_n - 1.0).abs() < 1e-9,
        exterior_only_preserves_interior_nf: (r_int_n - 1.0).abs() < 1e-12,
        exterior_only_preserves_caw: (r_c - 1.0).abs() < 1e-12
            && (r_a - 1.0).abs() < 1e-12
            && (r_w - 1.0).abs() < 1e-12,
        membrane_only_changes_crossing_nf: m_cross > 1.0 + 1e-6,
        membrane_only_preserves_exterior_base: (m_ext - 1.0).abs() < 1e-12,
        membrane_only_preserves_caw: (m_c - 1.0).abs() < 1e-12,
        combined_composes: (c_cross - expected_cross).abs() < 1e-9
            && (c_ext - expected_ext).abs() < 1e-9,
        face_symmetric: (d_fwd - d_rev).abs() < 1e-15,
    }
}

impl TransportIsolationReport {
    pub fn pass(self) -> bool {
        self.exterior_only_changes_exterior_nf
            && self.exterior_only_preserves_membrane_nf
            && self.exterior_only_preserves_interior_nf
            && self.exterior_only_preserves_caw
            && self.membrane_only_changes_crossing_nf
            && self.membrane_only_preserves_exterior_base
            && self.membrane_only_preserves_caw
            && self.combined_composes
            && self.face_symmetric
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensitivityMatrix {
    /// Columns: ∂y/∂ln m_ext , ∂y/∂ln(1/m_β). Rows: ln J_N, ln J_F, ln R_A, ln M_A.
    pub matrix: [[f64; 2]; 4],
    pub singular_values: [f64; 2],
    pub condition_number: f64,
    pub rank: usize,
    pub both_columns_measurable: bool,
}

/// Finite-difference sensitivity of observables y vs log-parameters.
pub fn sensitivity_from_observations(
    y0: [f64; 4],
    y_ext_plus: [f64; 4],
    y_ext_minus: [f64; 4],
    y_beta_plus: [f64; 4],
    y_beta_minus: [f64; 4],
    dln_ext: f64,
    dln_inv_beta: f64,
) -> SensitivityMatrix {
    let mut matrix = [[0.0; 2]; 4];
    for i in 0..4 {
        matrix[i][0] = (y_ext_plus[i].ln() - y_ext_minus[i].ln()) / (2.0 * dln_ext.max(1e-12));
        matrix[i][1] =
            (y_beta_plus[i].ln() - y_beta_minus[i].ln()) / (2.0 * dln_inv_beta.max(1e-12));
        let _ = y0[i];
    }
    // 4×2 SVD via Gram matrix of columns (2×2).
    let mut g = [[0.0; 2]; 2];
    for j in 0..2 {
        for k in 0..2 {
            let mut s = 0.0;
            for i in 0..4 {
                s += matrix[i][j] * matrix[i][k];
            }
            g[j][k] = s;
        }
    }
    let tr = g[0][0] + g[1][1];
    let det = g[0][0] * g[1][1] - g[0][1] * g[1][0];
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();
    let l1 = 0.5 * (tr + disc);
    let l2 = 0.5 * (tr - disc);
    let s1 = l1.max(0.0).sqrt();
    let s2 = l2.max(0.0).sqrt();
    let cond = if s2 > 1e-12 { s1 / s2 } else { f64::INFINITY };
    let col0: f64 = (0..4).map(|i| matrix[i][0].abs()).sum();
    let col1: f64 = (0..4).map(|i| matrix[i][1].abs()).sum();
    let measurable = col0 > 1e-6 && col1 > 1e-6;
    let rank = if s1 > 1e-8 {
        if s2 > 1e-8 {
            2
        } else {
            1
        }
    } else {
        0
    };
    SensitivityMatrix {
        matrix,
        singular_values: [s1, s2],
        condition_number: cond,
        rank,
        both_columns_measurable: measurable,
    }
}

/// Interaction I = Δ_combined − (Δ_ext + Δ_mem) on a chosen scalar (e.g. A retention).
pub fn interaction_excess(delta_combined: f64, delta_ext: f64, delta_mem: f64) -> f64 {
    delta_combined - (delta_ext + delta_mem)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepairCandidate {
    pub name: String,
    pub pair: DeliveryRepairPair,
    pub pi_n: f64,
    pub pi_f: f64,
    pub parent: String,
    pub justification: String,
}

/// Preregistered ≤6 candidates from sensitivity-guided brackets.
pub fn build_candidate_set(
    predicted: DeliveryRepairPair,
    beta_n: f64,
    beta_f: f64,
) -> Vec<RepairCandidate> {
    let m_beta_floor = m_beta_min_for_upper_band(beta_n.max(beta_f));
    let pred_ext = predicted.m_ext.clamp(D053_M_EXT_LO, D053_M_EXT_HI).max(2.0);
    let pred_beta = predicted
        .m_beta
        .clamp(m_beta_floor, D053_M_BETA_HI)
        .min(0.85)
        .max(m_beta_floor);

    let mut pairs = vec![
        (
            "baseline",
            DeliveryRepairPair::BASELINE,
            "frozen D-052 baseline",
        ),
        (
            "exterior_only",
            DeliveryRepairPair {
                m_ext: pred_ext.min(3.0).max(2.0),
                m_beta: 1.0,
            },
            "exterior-only control",
        ),
        (
            "membrane_only",
            DeliveryRepairPair {
                m_ext: 1.0,
                m_beta: pred_beta,
            },
            "membrane-only control",
        ),
        (
            "min_predicted",
            DeliveryRepairPair {
                m_ext: pred_ext,
                m_beta: pred_beta,
            },
            "minimum predicted combined pair",
        ),
        (
            "lower_bracket",
            DeliveryRepairPair {
                m_ext: (0.5 * (1.0 + pred_ext)).clamp(D053_M_EXT_LO, D053_M_EXT_HI).max(1.5),
                m_beta: (0.5 * (1.0 + pred_beta)).clamp(m_beta_floor, D053_M_BETA_HI),
            },
            "lower combined bracket",
        ),
        (
            "upper_bracket",
            DeliveryRepairPair {
                m_ext: D053_M_EXT_HI,
                m_beta: m_beta_floor,
            },
            "upper combined bracket at authorized bounds",
        ),
    ];

    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (name, pair, just) in pairs.drain(..) {
        if !pair_stage_a_authorized(pair, beta_n, beta_f) {
            continue;
        }
        let key = format!("{:.6}:{:.6}", pair.m_ext, pair.m_beta);
        if !seen.insert(key) {
            continue;
        }
        out.push(RepairCandidate {
            name: name.to_string(),
            pair,
            pi_n: nf_permeability_normalized(beta_n, pair.m_beta),
            pi_f: nf_permeability_normalized(beta_f, pair.m_beta),
            parent: "schema2_d050_center".to_string(),
            justification: just.to_string(),
        });
        if out.len() >= D053_MAX_CANDIDATES {
            break;
        }
    }
    out
}

pub fn select_minimum_change(passing: &[RepairCandidate]) -> Option<RepairCandidate> {
    passing
        .iter()
        .filter(|c| c.pair.m_ext > 1.0 + 1e-12 || c.pair.m_beta < 1.0 - 1e-12)
        .min_by(|a, b| {
            a.pair
                .delta_norm()
                .partial_cmp(&b.pair.delta_norm())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

/// Prefer higher A-retention / χ when recorded; else minimum |Δp|.
pub fn select_best_screened(
    passing: &[(RepairCandidate, f64, f64)],
) -> Option<RepairCandidate> {
    passing
        .iter()
        .filter(|(c, _, _)| c.pair.m_ext > 1.0 + 1e-12 || c.pair.m_beta < 1.0 - 1e-12)
        .max_by(|a, b| {
            // Primary: a_retention; secondary: chi; tertiary: smaller parameter change.
            a.1
                .partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| {
                    b.0.pair
                        .delta_norm()
                        .partial_cmp(&a.0.pair.delta_norm())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .map(|(c, _, _)| c.clone())
}

/// Predicted minimum correction from sensitivity (log-space step toward target rise).
pub fn predict_min_pair(
    sens: &SensitivityMatrix,
    target_ln_rise: f64,
) -> DeliveryRepairPair {
    // Use first two observables (J_N, J_F mean) and solve 2×2 least-squares for Δp.
    let a00 = sens.matrix[0][0] + sens.matrix[1][0];
    let a01 = sens.matrix[0][1] + sens.matrix[1][1];
    let a10 = sens.matrix[2][0] + sens.matrix[3][0];
    let a11 = sens.matrix[2][1] + sens.matrix[3][1];
    let det = a00 * a11 - a01 * a10;
    let (dln_ext, dln_inv_beta) = if det.abs() > 1e-12 {
        let b0 = 2.0 * target_ln_rise;
        let b1 = 2.0 * target_ln_rise;
        (
            (b0 * a11 - a01 * b1) / det,
            (a00 * b1 - b0 * a10) / det,
        )
    } else {
        (target_ln_rise.max(0.0), target_ln_rise.max(0.0))
    };
    DeliveryRepairPair {
        m_ext: dln_ext.exp().clamp(D053_M_EXT_LO, D053_M_EXT_HI),
        m_beta: (1.0 / dln_inv_beta.exp().max(1e-12))
            .clamp(D053_M_BETA_LO, D053_M_BETA_HI),
    }
}

pub fn chi_supply(j_in: f64, l_required: f64) -> f64 {
    j_in / l_required.max(1e-18)
}

pub fn resistance_fractions_within_tol(
    exterior_frac: f64,
    membrane_frac: f64,
    tol: f64,
) -> bool {
    (exterior_frac - D053_EXT_RESISTANCE_REF).abs() <= tol
        && (membrane_frac - D053_MEM_RESISTANCE_REF).abs() <= tol
}

/// Effective N beta after m_beta (for diagnostics).
pub fn effective_beta_n(params: &SimParams) -> f64 {
    species_beta(TransportSpecies::Nutrient, params)
}

pub fn effective_beta_f(params: &SimParams) -> f64 {
    species_beta(TransportSpecies::Fuel, params)
}

pub fn exterior_factor_at(phi_i: f64, phi_j: f64, m_ext: f64) -> f64 {
    exterior_resource_diffusivity_factor(phi_i, phi_j, m_ext)
}

// ---------------------------------------------------------------------------
// Canonical D-053 Gate 5 / Gate 8 evaluator (D-055: one shared contract)
// ---------------------------------------------------------------------------

/// Minimum final-window A retention for provisional Gate 5 admission.
pub const D053_GATE5_A_RETENTION_MIN: f64 = 0.50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HorizonClass {
    /// Full qualification horizon — may produce PASS.
    Full,
    /// Quick / smoke horizon — never qualifies a candidate.
    QuickDiagnostic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gate5Verdict {
    Pass,
    FailResourceSufficiency,
    FailACapacity,
    FailIncompleteEvidence,
    FailChecklist,
    DiagnosticOnly,
}

impl Gate5Verdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::FailResourceSufficiency => "FAIL_RESOURCE_SUFFICIENCY",
            Self::FailACapacity => "FAIL_A_CAPACITY",
            Self::FailIncompleteEvidence => "FAIL_INCOMPLETE_EVIDENCE",
            Self::FailChecklist => "FAIL_CHECKLIST",
            Self::DiagnosticOnly => "DIAGNOSTIC_ONLY",
        }
    }

    pub const fn admits_candidate(self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gate8Verdict {
    Pass,
    FailResourceSufficiency,
    FailRetention,
    FailTransportDirection,
    FailRadiusScaling,
    FailIncompleteEvidence,
    FailChecklist,
    DiagnosticOnly,
}

impl Gate8Verdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::FailResourceSufficiency => "FAIL_RESOURCE_SUFFICIENCY",
            Self::FailRetention => "FAIL_RETENTION",
            Self::FailTransportDirection => "FAIL_TRANSPORT_DIRECTION",
            Self::FailRadiusScaling => "FAIL_RADIUS_SCALING",
            Self::FailIncompleteEvidence => "FAIL_INCOMPLETE_EVIDENCE",
            Self::FailChecklist => "FAIL_CHECKLIST",
            Self::DiagnosticOnly => "DIAGNOSTIC_ONLY",
        }
    }

    pub const fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// Per-branch evidence for Gate 5 (analytic or restored).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Gate5BranchEvidence {
    pub chi_n: f64,
    pub chi_f: f64,
    pub activation_meets_a_demand: bool,
    pub a_retention_not_monotone_declining: bool,
    pub final_a_retention: f64,
    pub final_a_retention_slope: f64,
    pub p_production_active: bool,
    pub net_s_decline_arrested: bool,
    pub n_not_exhausted: bool,
    pub f_not_exhausted: bool,
    pub no_numerical_invalidity: bool,
    pub accounting_closes: bool,
}

impl Gate5BranchEvidence {
    pub fn chi_ok(self) -> bool {
        self.chi_n >= D053_CHI_MIN && self.chi_f >= D053_CHI_MIN
    }

    pub fn a_capacity_ok(self) -> bool {
        self.a_retention_not_monotone_declining
            && self.final_a_retention >= D053_GATE5_A_RETENTION_MIN
            && self.final_a_retention_slope >= 0.0
    }

    pub fn checklist_ok(self) -> bool {
        self.activation_meets_a_demand
            && self.a_capacity_ok()
            && self.p_production_active
            && self.net_s_decline_arrested
            && self.n_not_exhausted
            && self.f_not_exhausted
            && self.no_numerical_invalidity
            && self.accounting_closes
            && self.chi_ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Gate5Evidence {
    pub horizon_class: HorizonClass,
    pub analytic: Option<Gate5BranchEvidence>,
    pub restored: Option<Gate5BranchEvidence>,
}

/// Single shared Gate 5 classifier — no χ-rise / A-rise / short-horizon bypass.
pub fn evaluate_gate5(ev: &Gate5Evidence) -> Gate5Verdict {
    if ev.horizon_class == HorizonClass::QuickDiagnostic {
        return Gate5Verdict::DiagnosticOnly;
    }
    let (Some(a), Some(r)) = (ev.analytic, ev.restored) else {
        return Gate5Verdict::FailIncompleteEvidence;
    };
    if !a.chi_ok() || !r.chi_ok() {
        return Gate5Verdict::FailResourceSufficiency;
    }
    if !a.a_capacity_ok() || !r.a_capacity_ok() {
        return Gate5Verdict::FailACapacity;
    }
    if !a.checklist_ok() || !r.checklist_ok() {
        return Gate5Verdict::FailChecklist;
    }
    Gate5Verdict::Pass
}

/// Legacy informal Gate 5 path (audit only — never use for admission).
pub fn gate5_legacy_informal_admitted(
    capacity: bool,
    a_rise: bool,
    chi_rise: bool,
    a_retention: f64,
) -> bool {
    capacity || a_rise || (chi_rise && a_retention >= 0.5)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Gate8RadiusEvidence {
    pub radius: f64,
    pub chi_n: f64,
    pub chi_f: f64,
    pub c_retention: f64,
    pub a_retention: f64,
    pub n_enters: bool,
    pub f_enters: bool,
    pub w_exits: bool,
    pub bounded_fields: bool,
    pub accounting_closes: bool,
    pub influx_per_area: f64,
}

impl Gate8RadiusEvidence {
    pub fn chi_ok(self) -> bool {
        self.chi_n >= D053_CHI_MIN && self.chi_f >= D053_CHI_MIN
    }

    pub fn retention_ok(self) -> bool {
        self.c_retention >= D053_RETENTION_MIN && self.a_retention >= D053_RETENTION_MIN
    }

    pub fn transport_ok(self) -> bool {
        self.n_enters && self.f_enters && self.w_exits
    }

    pub fn radius_pass(self) -> bool {
        self.chi_ok()
            && self.retention_ok()
            && self.transport_ok()
            && self.bounded_fields
            && self.accounting_closes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Gate8Evidence {
    pub horizon_class: HorizonClass,
    /// Must include R16, R24, R32 (order unconstrained).
    pub radii: Vec<Gate8RadiusEvidence>,
}

fn gate8_find_radius(ev: &Gate8Evidence, r: f64) -> Option<&Gate8RadiusEvidence> {
    ev.radii.iter().find(|c| (c.radius - r).abs() < 1e-9)
}

/// Single shared Gate 8 classifier — short_horizon_relaxed is prohibited.
pub fn evaluate_gate8(ev: &Gate8Evidence) -> Gate8Verdict {
    if ev.horizon_class == HorizonClass::QuickDiagnostic {
        return Gate8Verdict::DiagnosticOnly;
    }
    let Some(r16) = gate8_find_radius(ev, 16.0) else {
        return Gate8Verdict::FailIncompleteEvidence;
    };
    let Some(r24) = gate8_find_radius(ev, 24.0) else {
        return Gate8Verdict::FailIncompleteEvidence;
    };
    let Some(r32) = gate8_find_radius(ev, 32.0) else {
        return Gate8Verdict::FailIncompleteEvidence;
    };
    for case in [r16, r24, r32] {
        if !case.chi_ok() {
            return Gate8Verdict::FailResourceSufficiency;
        }
        if !case.retention_ok() {
            return Gate8Verdict::FailRetention;
        }
        if !case.transport_ok() {
            return Gate8Verdict::FailTransportDirection;
        }
        if !case.bounded_fields || !case.accounting_closes {
            return Gate8Verdict::FailChecklist;
        }
    }
    if !(r16.influx_per_area > r24.influx_per_area && r24.influx_per_area > r32.influx_per_area) {
        return Gate8Verdict::FailRadiusScaling;
    }
    Gate8Verdict::Pass
}

/// Fixture helpers for invariance tests (Gate 2).
pub fn gate5_fixture_a_pass() -> Gate5Evidence {
    let branch = Gate5BranchEvidence {
        chi_n: 1.06,
        chi_f: 1.06,
        activation_meets_a_demand: true,
        a_retention_not_monotone_declining: true,
        final_a_retention: 0.55,
        final_a_retention_slope: 0.0,
        p_production_active: true,
        net_s_decline_arrested: true,
        n_not_exhausted: true,
        f_not_exhausted: true,
        no_numerical_invalidity: true,
        accounting_closes: true,
    };
    Gate5Evidence {
        horizon_class: HorizonClass::Full,
        analytic: Some(branch),
        restored: Some(branch),
    }
}

pub fn gate5_fixture_b_resource_fail() -> Gate5Evidence {
    let mut b = gate5_fixture_a_pass();
    if let Some(ref mut a) = b.analytic {
        a.chi_n = 0.53;
        a.chi_f = 0.53;
    }
    if let Some(ref mut r) = b.restored {
        r.chi_n = 0.53;
        r.chi_f = 0.53;
    }
    b
}

pub fn gate5_fixture_c_a_capacity_fail() -> Gate5Evidence {
    let mut b = gate5_fixture_a_pass();
    if let Some(ref mut a) = b.analytic {
        a.chi_n = 1.10;
        a.chi_f = 1.10;
        a.a_retention_not_monotone_declining = false;
        a.final_a_retention_slope = -0.01;
    }
    if let Some(ref mut r) = b.restored {
        r.chi_n = 1.10;
        r.chi_f = 1.10;
        r.a_retention_not_monotone_declining = false;
        r.final_a_retention_slope = -0.01;
    }
    b
}

pub fn gate5_fixture_d_incomplete() -> Gate5Evidence {
    let mut b = gate5_fixture_a_pass();
    b.restored = None;
    b
}

pub fn gate5_fixture_e_quick() -> Gate5Evidence {
    let mut b = gate5_fixture_a_pass();
    b.horizon_class = HorizonClass::QuickDiagnostic;
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_identity() {
        assert!((DeliveryRepairPair::BASELINE.delta_norm()).abs() < 1e-15);
        assert!(DeliveryRepairPair::BASELINE.is_authorized());
    }

    #[test]
    fn stage_a_band_with_m_beta() {
        let ordinary = nf_permeability_normalized(1.2, 1.0);
        assert!(stage_a_nf_band_ok(ordinary));
        let lo = nf_permeability_normalized(1.2, 0.5);
        // exp(-0.6)≈0.549 > 0.50 → unauthorized at θ=1
        assert!(!stage_a_nf_band_ok(lo));
        let ok = nf_permeability_normalized(1.2, m_beta_min_for_upper_band(1.2));
        assert!(stage_a_nf_band_ok(ok) || (ok - 0.50).abs() < 1e-9);
    }

    #[test]
    fn gate5_fixtures_match_contract() {
        assert_eq!(evaluate_gate5(&gate5_fixture_a_pass()), Gate5Verdict::Pass);
        assert_eq!(
            evaluate_gate5(&gate5_fixture_b_resource_fail()),
            Gate5Verdict::FailResourceSufficiency
        );
        assert_eq!(
            evaluate_gate5(&gate5_fixture_c_a_capacity_fail()),
            Gate5Verdict::FailACapacity
        );
        assert_eq!(
            evaluate_gate5(&gate5_fixture_d_incomplete()),
            Gate5Verdict::FailIncompleteEvidence
        );
        assert_eq!(
            evaluate_gate5(&gate5_fixture_e_quick()),
            Gate5Verdict::DiagnosticOnly
        );
    }

    #[test]
    fn legacy_informal_gate5_is_not_admission() {
        // Document the defect path; strict evaluator must reject the same metrics.
        assert!(gate5_legacy_informal_admitted(false, false, true, 0.50));
        let mut ev = gate5_fixture_a_pass();
        if let Some(ref mut a) = ev.analytic {
            a.chi_n = 0.53;
            a.chi_f = 0.53;
            a.final_a_retention = 0.50;
        }
        if let Some(ref mut r) = ev.restored {
            r.chi_n = 0.53;
            r.chi_f = 0.53;
            r.final_a_retention = 0.50;
        }
        assert_eq!(evaluate_gate5(&ev), Gate5Verdict::FailResourceSufficiency);
    }
}
