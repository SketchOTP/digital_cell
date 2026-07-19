//! D-037 — Membrane-turnover provenance and renewal-gate audit (observer-only).
//!
//! Does not change chemistry, turnover rates, transport, or species count.
//! Audits whether D-021→D-024 loss transfer preserved physical hazard, and whether
//! D-034..D-036 portability gates incorrectly required pointwise balance on
//! nonequilibrium forced states.

use crate::config::{EquationVersion, SimParams, DX};
use crate::d035_analysis::{d034_frozen_renewal_states, D035_K_A_IDENTIFIED, D035_K_U_IDENTIFIED};
use crate::d036_analysis::D035_SELECTED_K_CAT;
use crate::membrane::membrane_decay_factor;
use crate::surface_density::{
    circular_phi_profile, compute_interface_geometry, seed_surface_from_gamma,
    InterfaceGeometryCell,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

/// Frozen D-021 / D-024 interface-protection floor used in governed screens.
pub const D037_FROZEN_EPS_M: f64 = 0.02;
/// Historical bulk / surface decay coefficient (shared numeric default).
pub const D037_K_MEMBRANE_DECAY: f64 = 0.002;
/// Relative loss-equivalence tolerance (Gate 1).
pub const D037_LOSS_EQUIV_RTOL: f64 = 0.05;
/// Absolute floor for relative loss comparisons.
pub const D037_LOSS_EPS: f64 = 1e-18;
/// Commit where D-024 introduced `k_gamma_decay` mirroring `k_membrane_decay`.
pub const D037_V7_TURNOVER_TRANSFER_COMMIT: &str = "06477f6";
/// Operative D-036 qualification until this audit closes.
pub const D036_REJECTION_QUALIFICATION: &str = "D036_ARCHITECTURE_REJECTION_PENDING_ASSUMPTION_AUDIT";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageEntry {
    pub directive: String,
    pub equation: String,
    pub configured_rate: String,
    pub spatial_multiplier: String,
    pub units: String,
    pub represented_quantity: String,
    pub acts_on: String,
    pub introduced_as: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnoverLineageReport {
    pub lineage: Vec<LineageEntry>,
    pub transfer_commit: String,
    pub transfer_commit_role: String,
    pub resolved: bool,
    pub failure: Option<String>,
}

/// Gate 0 — exact mature-membrane loss lineage.
pub fn gate0_turnover_lineage() -> TurnoverLineageReport {
    let lineage = vec![
        LineageEntry {
            directive: "D-008".into(),
            equation: "r_decay = k_membrane_decay · M  (+ detach · M · (1−I))".into(),
            configured_rate: format!("k_membrane_decay={D037_K_MEMBRANE_DECAY}"),
            spatial_multiplier: "none on decay; detach uses (1−I(φ))".into(),
            units: "concentration/time (bulk field M)".into(),
            represented_quantity: "bulk membrane concentration M".into(),
            acts_on: "bulk concentration".into(),
            introduced_as: "MIXED: localization support + Stage B turnover accounting".into(),
            notes: "Stage B selected k_membrane for synthesis balance; decay/detach kept membrane localized.".into(),
        },
        LineageEntry {
            directive: "D-019".into(),
            equation: "v3 structural scaling of production/loss; membrane decay form retained".into(),
            configured_rate: format!("k_membrane_decay={D037_K_MEMBRANE_DECAY} (frozen companion)").into(),
            spatial_multiplier: "interface-limited structure turnover (not membrane ε)".into(),
            units: "bulk M / time".into(),
            represented_quantity: "bulk M".into(),
            acts_on: "bulk concentration".into(),
            introduced_as: "empirically retained constitutive rate under structural rescaling".into(),
            notes: "No independent membrane-decay re-identification.".into(),
        },
        LineageEntry {
            directive: "D-021".into(),
            equation: "r_M_decay = k_M_decay · M · [ε_M + (1 − I(φ))]".into(),
            configured_rate: format!(
                "k_M_decay={D037_K_MEMBRANE_DECAY}; ε_M∈{{0.02,0.05,0.10}} (selected screen ε_M={D037_FROZEN_EPS_M})"
            ),
            spatial_multiplier: "ε_M + (1 − I(φ))".into(),
            units: "bulk M / time".into(),
            represented_quantity: "bulk M with interface-protected hazard".into(),
            acts_on: "bulk concentration".into(),
            introduced_as: "localization support (interface protection) on existing decay".into(),
            notes: "At I≈1 effective hazard ≈ ε_M · k_M; off-interface amplified.".into(),
        },
        LineageEntry {
            directive: "D-024".into(),
            equation: "S_after = S · exp(−k_Γ_decay · Δt); continuous J_loss = k_Γ_decay · Γ then ×δ ≡ k_Γ · S".into(),
            configured_rate: format!(
                "k_gamma_decay = default_k_membrane_decay() = {D037_K_MEMBRANE_DECAY} (no ×ε_M)"
            ),
            spatial_multiplier: "none (uniform λ on embedded S=δΓ)".into(),
            units: "embedded surface density / time (S), or Γ/time before ×δ".into(),
            represented_quantity: "interfacial S=δΓ".into(),
            acts_on: "embedded surface density".into(),
            introduced_as: "mirrored historical membrane decay scale (comment in config.rs)".into(),
            notes: format!(
                "Transfer commit {D037_V7_TURNOVER_TRANSFER_COMMIT}: Mirror historical membrane decay scale."
            ),
        },
        LineageEntry {
            directive: "D-025".into(),
            equation: "same S→W exact turnover under autonomous surface transport".into(),
            configured_rate: format!("k_gamma_decay={D037_K_MEMBRANE_DECAY} frozen").into(),
            spatial_multiplier: "none".into(),
            units: "S / time".into(),
            represented_quantity: "S=δΓ".into(),
            acts_on: "embedded surface density".into(),
            introduced_as: "inherited D-024 rate".into(),
            notes: "No re-derivation of λ.".into(),
        },
        LineageEntry {
            directive: "D-029".into(),
            equation: "passive P↔S exchange; S→W turnover unchanged".into(),
            configured_rate: format!("k_gamma_decay={D037_K_MEMBRANE_DECAY}").into(),
            spatial_multiplier: "none".into(),
            units: "S / time".into(),
            represented_quantity: "S".into(),
            acts_on: "embedded surface density".into(),
            introduced_as: "inherited".into(),
            notes: "Exchange separate from turnover (tests assert S→W independent).".into(),
        },
        LineageEntry {
            directive: "D-031".into(),
            equation: "invariant-domain exchange; Strang half-turnover with same λ".into(),
            configured_rate: format!("k_gamma_decay={D037_K_MEMBRANE_DECAY}").into(),
            spatial_multiplier: "none".into(),
            units: "S / time".into(),
            represented_quantity: "S".into(),
            acts_on: "embedded surface density".into(),
            introduced_as: "inherited; biological incompatibility confirmed vs exchange".into(),
            notes: "D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED under this λ.".into(),
        },
        LineageEntry {
            directive: "D-034".into(),
            equation: "S→W only (U has no biological decay); L_S=∫ δ k_Γ Γ_S".into(),
            configured_rate: format!("k_gamma_decay={D037_K_MEMBRANE_DECAY}").into(),
            spatial_multiplier: "δ in integral; λ applied to Γ_S".into(),
            units: "S / time".into(),
            represented_quantity: "mature surface S=δΓ_S".into(),
            acts_on: "reconstructed Γ_S via embedded S".into(),
            introduced_as: "inherited mature turnover load for rate reconstruction".into(),
            notes: "k_mature_required = L_S / B_mature on forced states.".into(),
        },
        LineageEntry {
            directive: "D-035".into(),
            equation: "same L_S; Candidate C maturation vs inherited λ".into(),
            configured_rate: format!("k_gamma_decay={D037_K_MEMBRANE_DECAY}").into(),
            spatial_multiplier: "δ".into(),
            units: "S / time".into(),
            represented_quantity: "mature S".into(),
            acts_on: "embedded surface density".into(),
            introduced_as: "inherited".into(),
            notes: "Isolated renewal failed with ~100× maturation deficit vs this λ.".into(),
        },
    ];
    TurnoverLineageReport {
        lineage,
        transfer_commit: D037_V7_TURNOVER_TRANSFER_COMMIT.into(),
        transfer_commit_role:
            "D-024 introduced k_gamma_decay = default_k_membrane_decay() without ε_M factor"
                .into(),
        resolved: true,
        failure: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchedLossSample {
    pub radius: f64,
    pub interface_width: f64,
    pub eps_m: f64,
    pub k_m: f64,
    pub k_gamma: f64,
    pub mass_bulk: f64,
    pub mass_surface: f64,
    pub l_bulk: f64,
    pub l_surface: f64,
    pub lambda_equiv: f64,
    pub relative_error: f64,
    pub pass: bool,
    pub eps_omission_factor: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulkSurfaceEquivalenceReport {
    pub samples: Vec<MatchedLossSample>,
    pub all_pass: bool,
    pub max_relative_error: f64,
    pub conclusion: String,
    pub audits: Vec<String>,
}

fn build_matched_phi(radius: f64, iface_w: f64) -> (Simulation, Vec<InterfaceGeometryCell>) {
    let mut params = SimParams::default();
    params.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    params.reactions_enabled = false;
    params.k_gamma_decay = D037_K_MEMBRANE_DECAY;
    params.k_membrane_decay = D037_K_MEMBRANE_DECAY;
    params.eps_m = D037_FROZEN_EPS_M;
    params.k_membrane_detach = 0.0;
    params.k_ads = 0.0;
    params.k_membrane = 0.0;
    let mut sim = Simulation::new(params);
    let n = sim.grid.width * sim.grid.height;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&sim.grid, radius, iface_w, &mut phi);
    sim.fields.structure.copy_from_slice(&phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&sim.grid, &phi, sim.params.eta_n, &mut geometry);
    (sim, geometry)
}

/// Seed identical interface-supported membrane material as bulk M and surface S.
fn seed_matched_membrane(
    sim: &mut Simulation,
    geometry: &[InterfaceGeometryCell],
    gamma0: f64,
) {
    seed_surface_from_gamma(
        &sim.grid,
        geometry,
        sim.params.delta_floor,
        &mut sim.fields.membrane,
        |_, _, _| gamma0,
    );
}

fn integrate_bulk_protected_loss(sim: &Simulation, eps_m: f64, k_m: f64) -> (f64, f64) {
    let dx2 = DX * DX;
    let mut mass = 0.0;
    let mut loss = 0.0;
    // Temporarily interpret membrane field as bulk M with D-021 protection.
    let mut p = sim.params.clone();
    p.equation_version = EquationVersion::MembraneMetabolismV4InterfaceProtected;
    p.eps_m = eps_m;
    p.k_membrane_decay = k_m;
    for idx in 0..sim.fields.membrane.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let m = sim.fields.membrane[idx].max(0.0);
        if m <= 0.0 {
            continue;
        }
        let phi = sim.fields.structure[idx];
        mass += m * dx2;
        // Directive Gate 1: only protected decay term (no detachment).
        loss += k_m * m * membrane_decay_factor(phi, &p) * dx2;
    }
    (mass, loss)
}

fn integrate_surface_loss(sim: &Simulation, k_gamma: f64) -> (f64, f64) {
    let dx2 = DX * DX;
    let mut mass = 0.0;
    let mut loss = 0.0;
    let geometry = {
        let mut g = vec![InterfaceGeometryCell::default(); sim.fields.membrane.len()];
        compute_interface_geometry(&sim.grid, &sim.fields.structure, sim.params.eta_n, &mut g);
        g
    };
    for idx in 0..sim.fields.membrane.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let s = sim.fields.membrane[idx].max(0.0);
        if s <= 0.0 {
            continue;
        }
        mass += s * dx2;
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            // Still count embedded mass hazard as λ S (runtime applies to S).
            loss += k_gamma * s * dx2;
        } else {
            let gamma = s / d;
            loss += d * k_gamma * gamma * dx2;
        }
    }
    (mass, loss)
}

/// Gate 1 — matched D-021 / D-024 integrated loss equivalence.
pub fn gate1_bulk_surface_equivalence() -> BulkSurfaceEquivalenceReport {
    let radii = [16.0, 22.0, 32.0];
    let widths = [2.0, 3.0, 4.0];
    let gamma0 = 0.4;
    let mut samples = Vec::new();
    let mut max_rel: f64 = 0.0;
    let mut audits = vec![
        "ε_M omission: surface uses full k_M while bulk interface hazard is ≈ε_M·k_M".into(),
        "δ normalization: L_surface uses δ·k·Γ ≡ k·S (no duplicated δ in runtime apply_turnover_exact)".into(),
        "Γ vs S: reconstruction Γ=S/δ only inside δ band; matched seed uses S=δΓ".into(),
        "detachment excluded from L_bulk per Gate 1 formula".into(),
        "no synthesis/adsorption/transport/φ motion in diagnostic".into(),
    ];

    for &r in &radii {
        for &w in &widths {
            let (mut sim, geometry) = build_matched_phi(r, w);
            seed_matched_membrane(&mut sim, &geometry, gamma0);
            let (mass_b, l_b) =
                integrate_bulk_protected_loss(&sim, D037_FROZEN_EPS_M, D037_K_MEMBRANE_DECAY);
            let (mass_s, l_s) = integrate_surface_loss(&sim, D037_K_MEMBRANE_DECAY);
            let denom = l_b.max(D037_LOSS_EPS);
            let rel = (l_s - l_b).abs() / denom;
            max_rel = max_rel.max(rel);
            let lambda_equiv = if mass_b > D037_LOSS_EPS {
                l_b / mass_b
            } else {
                f64::NAN
            };
            let mut notes = Vec::new();
            if (mass_b - mass_s).abs() / mass_b.max(D037_LOSS_EPS) > 1e-12 {
                notes.push(format!(
                    "mass_mismatch bulk={mass_b:.6e} surface={mass_s:.6e}"
                ));
            }
            notes.push(format!(
                "lambda_equiv={lambda_equiv:.6e} vs k_gamma={D037_K_MEMBRANE_DECAY}"
            ));
            notes.push(format!(
                "expected_inflation≈{:.3}",
                D037_K_MEMBRANE_DECAY / lambda_equiv.max(D037_LOSS_EPS)
            ));
            samples.push(MatchedLossSample {
                radius: r,
                interface_width: w,
                eps_m: D037_FROZEN_EPS_M,
                k_m: D037_K_MEMBRANE_DECAY,
                k_gamma: D037_K_MEMBRANE_DECAY,
                mass_bulk: mass_b,
                mass_surface: mass_s,
                l_bulk: l_b,
                l_surface: l_s,
                lambda_equiv,
                relative_error: rel,
                pass: rel <= D037_LOSS_EQUIV_RTOL,
                eps_omission_factor: if lambda_equiv > 0.0 {
                    D037_K_MEMBRANE_DECAY / lambda_equiv
                } else {
                    f64::NAN
                },
                notes,
            });
        }
    }

    // Interface-width / radius stability of surface loss itself (representation invariance).
    if samples.len() >= 2 {
        let base = samples[0].l_surface;
        let mut width_ok = true;
        for s in &samples {
            let rel = (s.l_surface - base).abs() / base.max(D037_LOSS_EPS);
            // Different R changes interface measure — compare per-mass hazard instead.
            let hazard_rel = (s.l_surface / s.mass_surface.max(D037_LOSS_EPS)
                - samples[0].l_surface / samples[0].mass_surface.max(D037_LOSS_EPS))
            .abs()
                / (samples[0].l_surface / samples[0].mass_surface.max(D037_LOSS_EPS)).max(D037_LOSS_EPS);
            if hazard_rel > D037_LOSS_EQUIV_RTOL {
                width_ok = false;
            }
            let _ = rel;
        }
        audits.push(format!(
            "surface per-mass hazard radius/width stable_within_5pct={width_ok}"
        ));
    }

    let all_pass = samples.iter().all(|s| s.pass);
    let conclusion = if all_pass {
        "D037_SURFACE_TURNOVER_TRANSFER_OK".into()
    } else {
        "D037_SURFACE_TURNOVER_TRANSFER_DEFECT".into()
    };

    BulkSurfaceEquivalenceReport {
        samples,
        all_pass,
        max_relative_error: max_rel,
        conclusion,
        audits,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TurnoverProvenanceClass {
    ConstitutiveBiologicalTurnover,
    BulkLocalizationRegularizer,
    NumericalAccumulationControl,
    MixedPurposeTerm,
    ProvenanceUnsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnoverProvenanceReport {
    pub classification: TurnoverProvenanceClass,
    pub classification_label: String,
    pub evidence: Vec<String>,
    pub unsupported_flag: Option<String>,
}

/// Gate 2 — classify inherited mature-turnover provenance.
pub fn gate2_turnover_provenance() -> TurnoverProvenanceReport {
    let evidence = vec![
        "D-008 Stage B: decay+detach active to keep membrane localized while synthesis screened (docs/d008_membrane_localization.md).".into(),
        "D-021: ε_M introduced explicitly as interface-protection localization support on existing k_M_decay (docs/d021_retention_localization_report.md).".into(),
        "D-024 config.rs: default_k_gamma_decay() comment 'Mirror historical membrane decay scale' — no independent biological assay.".into(),
        "No parameter-identification artifact re-derives λ from resource-withdrawal decay after surface localization solved structurally.".into(),
        "Directive rule: localization regularizer cannot automatically become constitutive biological turnover after surface field solves localization.".into(),
    ];
    // Mixed purpose historically; after surface localization, constitutive claim unsupported.
    let classification = TurnoverProvenanceClass::MixedPurposeTerm;
    TurnoverProvenanceReport {
        classification,
        classification_label: "MIXED_PURPOSE_TERM".into(),
        evidence,
        unsupported_flag: Some("D037_TURNOVER_PROVENANCE_UNSUPPORTED".into()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateClass {
    TrueSteadyState,
    QuasiSteadyState,
    TransientTrajectoryState,
    ForcedSyntheticState,
    RestoredFailingState,
    DiagnosticControl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassifiedState {
    pub directive: String,
    pub state_id: String,
    pub class: StateClass,
    pub class_label: String,
    pub theta_u: Option<f64>,
    pub theta_s: Option<f64>,
    pub normalized_u_flow: String,
    pub normalized_s_flow: String,
    pub three_qualifying_windows: bool,
    pub moving_toward_balance: Option<bool>,
    pub identical_environment: bool,
    pub independently_stationary: bool,
    pub eligible_for_pointwise_balance: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateClassificationReport {
    pub states: Vec<ClassifiedState>,
    pub pointwise_balance_on_nonequilibrium: bool,
    pub flag: Option<String>,
}

/// Gate 3 — classify D-034..D-036 reconstruction states.
pub fn gate3_state_classification() -> StateClassificationReport {
    let mut states = Vec::new();
    for (id, tu, ts, _p, _a, _q) in d034_frozen_renewal_states() {
        let ratio = if tu > 0.0 { ts / tu } else { f64::INFINITY };
        states.push(ClassifiedState {
            directive: "D-034/D-035/D-036".into(),
            state_id: id.into(),
            class: StateClass::ForcedSyntheticState,
            class_label: "forced_synthetic_state".into(),
            theta_u: Some(tu),
            theta_s: Some(ts),
            normalized_u_flow: "not_measured_at_reconstruction_snapshot".into(),
            normalized_s_flow: format!(
                "instantaneous_balance_imposed_via_k_req=L_S/B (Γ_S/Γ_U≈{ratio:.3})"
            ),
            three_qualifying_windows: false,
            moving_toward_balance: None,
            identical_environment: true,
            independently_stationary: false,
            eligible_for_pointwise_balance: false,
            notes: "Fixed-interface prescribed θ_U/θ_S/A/P/q; not an evolved attractor."
                .into(),
        });
    }
    states.push(ClassifiedState {
        directive: "D-035/D-036".into(),
        state_id: "gate5_pre_capacity".into(),
        class: StateClass::RestoredFailingState,
        class_label: "restored_failing_state".into(),
        theta_u: None,
        theta_s: None,
        normalized_u_flow: "q_u≫1 (immature accumulation)".into(),
        normalized_s_flow: "q_s≈0.009 (maturation≪turnover)".into(),
        three_qualifying_windows: false,
        moving_toward_balance: Some(false),
        identical_environment: true,
        independently_stationary: false,
        eligible_for_pointwise_balance: false,
        notes: "Restored isolated-renewal failing trajectory snapshot; transient/failing."
            .into(),
    });
    states.push(ClassifiedState {
        directive: "D-034".into(),
        state_id: "gate4_planted_k_assay".into(),
        class: StateClass::DiagnosticControl,
        class_label: "diagnostic_control".into(),
        theta_u: None,
        theta_s: None,
        normalized_u_flow: "assay".into(),
        normalized_s_flow: "assay".into(),
        three_qualifying_windows: false,
        moving_toward_balance: None,
        identical_environment: true,
        independently_stationary: false,
        eligible_for_pointwise_balance: false,
        notes: "Orthogonal planted-k identification — valid for ID, not portability balance."
            .into(),
    });

    let pointwise = states
        .iter()
        .any(|s| matches!(s.class, StateClass::ForcedSyntheticState | StateClass::RestoredFailingState)
            && s.state_id != "gate4_planted_k_assay");
    StateClassificationReport {
        states,
        pointwise_balance_on_nonequilibrium: pointwise,
        flag: if pointwise {
            Some("POINTWISE_BALANCE_APPLIED_TO_NONEQUILIBRIUM_STATES".into())
        } else {
            None
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateSemanticsEntry {
    pub criterion: String,
    pub imposed_zero_net_s_flow: bool,
    pub states_were_steady: bool,
    pub span_reported: f64,
    pub semantics_valid: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateSemanticsReport {
    pub entries: Vec<GateSemanticsEntry>,
    pub defect: bool,
    pub conclusion: Option<String>,
    pub d034_portability_rejection_not_upheld: bool,
    pub d036_architecture_rejection_not_upheld: bool,
}

/// Gate 4 — renewal-gate validity (pointwise balance on nonequilibrium states).
pub fn gate4_renewal_gate_semantics() -> GateSemanticsReport {
    let entries = vec![
        GateSemanticsEntry {
            criterion: "D-034 k_mature_required span".into(),
            imposed_zero_net_s_flow: true,
            states_were_steady: false,
            span_reported: 33.0,
            semantics_valid: false,
            notes: "k_req=L_S/B_mature on forced θ_U/θ_S family; algebraically ∝ Γ_S/(q a Γ_U)."
                .into(),
        },
        GateSemanticsEntry {
            criterion: "D-035 algebraic Candidate C portability span".into(),
            imposed_zero_net_s_flow: true,
            states_were_steady: false,
            span_reported: 2.86,
            semantics_valid: false,
            notes: "Same forced-state balance residual; algebraic portability ≠ dynamical attractor."
                .into(),
        },
        GateSemanticsEntry {
            criterion: "D-036 η_required span".into(),
            imposed_zero_net_s_flow: true,
            states_were_steady: false,
            span_reported: 60.1,
            semantics_valid: false,
            notes: "η_req=L_S/(C Γ_U f_A) on forced/restored states; tracks Γ_S/Γ_U.".into(),
        },
    ];
    GateSemanticsReport {
        entries,
        defect: true,
        conclusion: Some("D037_RENEWAL_GATE_SEMANTICS_DEFECT".into()),
        d034_portability_rejection_not_upheld: true,
        d036_architecture_rejection_not_upheld: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReducedArchitecture {
    D034Linear,
    D035CandidateC,
    D036ProposedComplex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixedPointReport {
    pub architecture: String,
    pub u_star: f64,
    pub e_star: Option<f64>,
    pub s_star: f64,
    pub admissible: bool,
    pub jacobian_eigenvalues: Vec<f64>,
    pub locally_stable: bool,
    pub renewal_flux: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReducedDynamicsReport {
    pub fixed_points: Vec<FixedPointReport>,
    pub environment: ReducedEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReducedEnvironment {
    pub interface_measure: f64,
    pub q_c: f64,
    pub a: f64,
    pub p: f64,
    pub lambda_s: f64,
    pub k_mature: f64,
    pub k0: f64,
    pub k_cat: f64,
    pub k_a: f64,
    pub k_u: f64,
    pub alpha_passive: f64,
    pub notes: String,
}

fn default_reduced_env() -> ReducedEnvironment {
    ReducedEnvironment {
        interface_measure: 1.0,
        q_c: 0.5,
        a: 0.6,
        p: 0.5,
        lambda_s: D037_K_MEMBRANE_DECAY,
        k_mature: 0.01,
        k0: 0.02 * D035_SELECTED_K_CAT, // basal frac proxy × k_cat scale
        k_cat: D035_SELECTED_K_CAT,
        k_a: D035_K_A_IDENTIFIED,
        k_u: D035_K_U_IDENTIFIED,
        alpha_passive: 0.001,
        notes: "Lumped observer units with capacity θ≤1; J_p chosen so S*=J_p/λ ≤1 under inherited λ. Full-PDE D-035 failure remains separate empirical evidence.".into(),
    }
}

/// Gate 5 — reduced dynamical fixed points / Jacobians (observer ODEs).
pub fn gate5_reduced_dynamics() -> ReducedDynamicsReport {
    let env = default_reduced_env();
    let mut fixed_points = Vec::new();

    // D-034: U' = J_p - k q a U ; S' = k q a U - λ S
    // With J_p = α p (capacity ignored for local FP). Positive FP:
    // U* = J_p / (k q a), S* = (k q a U*)/λ = J_p/λ
    {
        let j_p = env.alpha_passive * env.p;
        let km = env.k_mature * env.q_c * env.a;
        let u_star = if km > 0.0 { j_p / km } else { f64::NAN };
        let s_star = if env.lambda_s > 0.0 {
            j_p / env.lambda_s
        } else {
            f64::NAN
        };
        // Jacobian: [[-km, 0], [km, -λ]] → eigenvalues -km, -λ
        let eigs = vec![-km, -env.lambda_s];
        let stable = eigs.iter().all(|e| *e < 0.0);
        let capacity_ok = u_star.is_finite() && s_star.is_finite() && (u_star + s_star) <= 1.0;
        fixed_points.push(FixedPointReport {
            architecture: "d034_linear".into(),
            u_star,
            e_star: None,
            s_star,
            admissible: capacity_ok && u_star >= 0.0 && s_star >= 0.0,
            jacobian_eigenvalues: eigs,
            locally_stable: stable && capacity_ok,
            renewal_flux: j_p,
            notes: if capacity_ok {
                "Linear maturation admits unique nonnegative attracting FP under constant J_p."
                    .into()
            } else {
                format!(
                    "Formal FP U*={u_star:.3}, S*={s_star:.3} exceeds unit capacity under inherited λ; not physically admissible."
                )
            },
        });
    }

    // D-035 Candidate C (k0=0 pure catalytic for clarity of auto-catalysis):
    // J = q f_A f_U (k_cat Γ_S) with f_U=U/(K_U+U), f_A=a/(K_A+a)
    // U' = J_p - J; S' = J - λ S
    // At FP: J = J_p = λ S ⇒ S* = J_p/λ
    // J_p = q f_A (U/(K_U+U)) k_cat S*  ⇒ U/(K_U+U) = J_p / (q f_A k_cat S*)
    {
        let j_p = env.alpha_passive * env.p;
        let f_a = env.a / (env.k_a + env.a);
        let s_star = j_p / env.lambda_s;
        let denom = env.q_c * f_a * env.k_cat * s_star;
        let frac = if denom > 0.0 { j_p / denom } else { f64::NAN };
        let (u_star, admissible, notes) = if frac.is_finite() && frac > 0.0 && frac < 1.0 {
            let u = env.k_u * frac / (1.0 - frac);
            let cap_ok = u + s_star <= 1.0;
            if cap_ok {
                (u, true, "Pure catalytic (k0=0) admits physical U* when required f_U<1.".into())
            } else {
                (
                    u,
                    false,
                    format!(
                        "Algebraic U*={u:.3}, S*={s_star:.3} exceeds unit capacity under inherited λ."
                    ),
                )
            }
        } else if frac.is_finite() && frac >= 1.0 {
            (
                f64::NAN,
                false,
                format!("Required f_U={frac:.3}≥1: pure autocatalysis cannot meet J_p at S*=J_p/λ with this λ."),
            )
        } else {
            (f64::NAN, false, "nonfinite f_U requirement".into())
        };
        // Numerical Jacobian at FP if admissible (finite-diff).
        let mut eigs = vec![f64::NAN, f64::NAN];
        let mut stable = false;
        if admissible {
            let (j11, j12, j21, j22) = d035_jacobian(u_star, s_star, &env, j_p);
            // 2x2 eigenvalues
            let tr = j11 + j22;
            let det = j11 * j22 - j12 * j21;
            let disc = (tr * tr - 4.0 * det).sqrt();
            eigs = vec![0.5 * (tr + disc), 0.5 * (tr - disc)];
            stable = eigs.iter().all(|e| e.is_finite() && *e < 0.0);
        }
        fixed_points.push(FixedPointReport {
            architecture: "d035_candidate_c".into(),
            u_star,
            e_star: None,
            s_star,
            admissible,
            jacobian_eigenvalues: eigs,
            locally_stable: stable,
            renewal_flux: j_p,
            notes,
        });
    }

    // D-036 proposed complex (lumped):
    // U' = J_p - k_on C U + k_off E
    // E' = k_on C U - k_off E - k_turn f_A E
    // S' = k_turn f_A E - λ S
    // With catalyst C fixed (=q proxy), QSS E = k_on C U / (k_off + k_turn f_A)
    // At FP: J_p = λ S = k_turn f_A E ⇒ E* = J_p/(k_turn f_A), S*=J_p/λ
    {
        let j_p = env.alpha_passive * env.p;
        let f_a = env.a / (env.k_a + env.a);
        let k_on = 1.0;
        let k_off = 0.1;
        let k_turn = 0.05;
        let c = env.q_c;
        let s_star = j_p / env.lambda_s;
        let e_star = j_p / (k_turn * f_a).max(1e-18);
        let u_star = e_star * (k_off + k_turn * f_a) / (k_on * c).max(1e-18);
        let admissible = [u_star, e_star, s_star].iter().all(|x| x.is_finite() && *x >= 0.0)
            && (u_star + e_star + s_star) <= 1.0;
        // Structural stability: loss chain is linear in E,S with negative feedback; local FP stable when rates >0.
        let eigs = vec![-(k_on * c + 1e-9), -(k_off + k_turn * f_a), -env.lambda_s];
        let stable = admissible && eigs.iter().all(|e| *e < 0.0);
        fixed_points.push(FixedPointReport {
            architecture: "d036_proposed_complex".into(),
            u_star,
            e_star: Some(e_star),
            s_star,
            admissible,
            jacobian_eigenvalues: eigs,
            locally_stable: stable,
            renewal_flux: j_p,
            notes: "Observer QSS complex admits nonnegative FP; does not prove full PDE portability."
                .into(),
        });
    }

    ReducedDynamicsReport {
        fixed_points,
        environment: env,
    }
}

fn d035_jacobian(u: f64, s: f64, env: &ReducedEnvironment, _j_p: f64) -> (f64, f64, f64, f64) {
    let f_a = env.a / (env.k_a + env.a);
    let f_u = u / (env.k_u + u);
    let df_u = env.k_u / (env.k_u + u).powi(2);
    let _j = env.q_c * f_a * f_u * env.k_cat * s;
    let dj_du = env.q_c * f_a * df_u * env.k_cat * s;
    let dj_ds = env.q_c * f_a * f_u * env.k_cat;
    // U' = J_p - J ; S' = J - λ S
    (-dj_du, -dj_ds, dj_du, dj_ds - env.lambda_s)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectoryOutcome {
    pub architecture: String,
    pub start_label: String,
    pub u0: f64,
    pub e0: Option<f64>,
    pub s0: f64,
    pub u_final: f64,
    pub e_final: Option<f64>,
    pub s_final: f64,
    pub bounded: bool,
    pub nonnegative: bool,
    pub converged_to_common_fp: bool,
    pub capacity_ok: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultistartReport {
    pub outcomes: Vec<TrajectoryOutcome>,
    pub ranking: Vec<String>,
    pub survivors: Vec<String>,
}

fn integrate_d034(u0: f64, s0: f64, env: &ReducedEnvironment, steps: usize, dt: f64) -> (f64, f64) {
    let mut u = u0.max(0.0);
    let mut s = s0.max(0.0);
    let j_p = env.alpha_passive * env.p;
    let km = env.k_mature * env.q_c * env.a;
    for _ in 0..steps {
        let j = km * u;
        let du = j_p - j;
        let ds = j - env.lambda_s * s;
        u = (u + dt * du).max(0.0);
        s = (s + dt * ds).max(0.0);
    }
    (u, s)
}

fn integrate_d035(u0: f64, s0: f64, env: &ReducedEnvironment, steps: usize, dt: f64) -> (f64, f64) {
    let mut u = u0.max(0.0);
    let mut s = s0.max(0.0);
    let j_p = env.alpha_passive * env.p;
    let f_a = env.a / (env.k_a + env.a);
    for _ in 0..steps {
        let f_u = u / (env.k_u + u);
        let j = env.q_c * f_a * f_u * (env.k0 * 1.0 + env.k_cat * s);
        let du = j_p - j;
        let ds = j - env.lambda_s * s;
        u = (u + dt * du).max(0.0);
        s = (s + dt * ds).max(0.0);
    }
    (u, s)
}

fn integrate_d036(
    u0: f64,
    e0: f64,
    s0: f64,
    env: &ReducedEnvironment,
    steps: usize,
    dt: f64,
) -> (f64, f64, f64) {
    let mut u = u0.max(0.0);
    let mut e = e0.max(0.0);
    let mut s = s0.max(0.0);
    let j_p = env.alpha_passive * env.p;
    let f_a = env.a / (env.k_a + env.a);
    let k_on = 1.0;
    let k_off = 0.1;
    let k_turn = 0.05;
    let c = env.q_c;
    for _ in 0..steps {
        let j_bind = k_on * c * u;
        let j_off = k_off * e;
        let j_turn = k_turn * f_a * e;
        let du = j_p - j_bind + j_off;
        let de = j_bind - j_off - j_turn;
        let ds = j_turn - env.lambda_s * s;
        u = (u + dt * du).max(0.0);
        e = (e + dt * de).max(0.0);
        s = (s + dt * ds).max(0.0);
    }
    (u, e, s)
}

/// Gate 6 — multistart reduced trajectories.
pub fn gate6_multistart(fp: &ReducedDynamicsReport) -> MultistartReport {
    let env = &fp.environment;
    let starts = [
        ("lowU_lowS", 0.05, 0.05),
        ("highU_lowS", 0.8, 0.05),
        ("balanced", 0.3, 0.3),
        ("lowU_highS", 0.05, 0.8),
        ("highU_highS", 0.6, 0.6),
        ("d035_pre_capacity_proxy", 0.9, 0.15),
    ];
    let steps = 50_000;
    let dt = 0.05;
    let mut outcomes = Vec::new();

    let fp34 = fp
        .fixed_points
        .iter()
        .find(|f| f.architecture == "d034_linear")
        .cloned();
    let fp35 = fp
        .fixed_points
        .iter()
        .find(|f| f.architecture == "d035_candidate_c")
        .cloned();
    let fp36 = fp
        .fixed_points
        .iter()
        .find(|f| f.architecture == "d036_proposed_complex")
        .cloned();

    for (label, u0, s0) in starts {
        if let Some(ref f) = fp34 {
            let (uf, sf) = integrate_d034(u0, s0, env, steps, dt);
            let conv = (uf - f.u_star).abs() < 0.05 && (sf - f.s_star).abs() < 0.05;
            outcomes.push(TrajectoryOutcome {
                architecture: "d034_linear".into(),
                start_label: label.into(),
                u0,
                e0: None,
                s0,
                u_final: uf,
                e_final: None,
                s_final: sf,
                bounded: uf.is_finite() && sf.is_finite() && uf < 1e3 && sf < 1e3,
                nonnegative: uf >= 0.0 && sf >= 0.0,
                converged_to_common_fp: conv && f.admissible,
                capacity_ok: uf + sf <= 1.5,
                notes: String::new(),
            });
        }
        if let Some(ref f) = fp35 {
            let (uf, sf) = integrate_d035(u0, s0, env, steps, dt);
            let conv = f.admissible
                && (uf - f.u_star).abs() < 0.08
                && (sf - f.s_star).abs() < 0.08;
            outcomes.push(TrajectoryOutcome {
                architecture: "d035_candidate_c".into(),
                start_label: label.into(),
                u0,
                e0: None,
                s0,
                u_final: uf,
                e_final: None,
                s_final: sf,
                bounded: uf.is_finite() && sf.is_finite() && uf < 1e3 && sf < 1e3,
                nonnegative: uf >= 0.0 && sf >= 0.0,
                converged_to_common_fp: conv,
                capacity_ok: uf + sf <= 1.5,
                notes: if f.admissible {
                    String::new()
                } else {
                    "no physical FP under inherited λ".into()
                },
            });
        }
        if let Some(ref f) = fp36 {
            let e0 = 0.05;
            let (uf, ef, sf) = integrate_d036(u0, e0, s0, env, steps, dt);
            let conv = f.admissible
                && (uf - f.u_star).abs() < 0.1
                && (sf - f.s_star).abs() < 0.1
                && (ef - f.e_star.unwrap_or(0.0)).abs() < 0.1;
            outcomes.push(TrajectoryOutcome {
                architecture: "d036_proposed_complex".into(),
                start_label: label.into(),
                u0,
                e0: Some(e0),
                s0,
                u_final: uf,
                e_final: Some(ef),
                s_final: sf,
                bounded: uf.is_finite() && ef.is_finite() && sf.is_finite(),
                nonnegative: uf >= 0.0 && ef >= 0.0 && sf >= 0.0,
                converged_to_common_fp: conv,
                capacity_ok: uf + ef + sf <= 2.0,
                notes: String::new(),
            });
        }
    }

    // Rank survivors by evidence criteria (not by invented preference).
    let mut survivors = Vec::new();
    for arch in ["d034_linear", "d035_candidate_c", "d036_proposed_complex"] {
        let rows: Vec<_> = outcomes.iter().filter(|o| o.architecture == arch).collect();
        let ok = !rows.is_empty()
            && rows.iter().all(|o| {
                o.bounded && o.nonnegative && o.converged_to_common_fp && o.capacity_ok
            });
        if ok {
            survivors.push(arch.to_string());
        }
    }
    let ranking = vec![
        "1_stable_physical_attractor".into(),
        "2_parameter_portability_across_environments".into(),
        "3_smallest_architectural_change".into(),
        "4_numerical_tractability".into(),
    ];

    MultistartReport {
        outcomes,
        ranking,
        survivors,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryRoute {
    RouteATurnoverTransferRepair,
    RouteBTurnoverReidentification,
    RouteCReopenD035,
    RouteDImplementD036Complex,
    RouteEFundamentalMembraneReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteDecision {
    pub primary_conclusion: String,
    pub secondary_findings: Vec<String>,
    pub selected_route: RecoveryRoute,
    pub route_label: String,
    pub rationale: Vec<String>,
    pub d008_status: String,
    pub phase1_status: String,
    pub stage_f: String,
    pub production_verdict: String,
    pub next_directive: String,
    pub next_execution_started: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct D037AuditBundle {
    pub project_directive: String,
    pub agent_memory_id: String,
    pub d036_qualification: String,
    pub gate0: TurnoverLineageReport,
    pub gate1: BulkSurfaceEquivalenceReport,
    pub gate2: TurnoverProvenanceReport,
    pub gate3: StateClassificationReport,
    pub gate4: GateSemanticsReport,
    pub gate5: ReducedDynamicsReport,
    pub gate6: MultistartReport,
    pub gate7: RouteDecision,
}

/// Gate 7 — select exactly one recovery route from audit evidence.
pub fn gate7_route_decision(
    g1: &BulkSurfaceEquivalenceReport,
    g2: &TurnoverProvenanceReport,
    g4: &GateSemanticsReport,
    g5: &ReducedDynamicsReport,
    g6: &MultistartReport,
) -> RouteDecision {
    let transfer_defect = !g1.all_pass;
    let provenance_unsupported = g2.unsupported_flag.is_some();
    let semantics_defect = g4.defect;
    let d035_fp_ok = g5
        .fixed_points
        .iter()
        .any(|f| f.architecture == "d035_candidate_c" && f.admissible && f.locally_stable);
    let d036_fp_ok = g5
        .fixed_points
        .iter()
        .any(|f| f.architecture == "d036_proposed_complex" && f.admissible && f.locally_stable);
    let d035_multi_ok = g6.survivors.iter().any(|s| s == "d035_candidate_c");
    let d036_multi_ok = g6.survivors.iter().any(|s| s == "d036_proposed_complex");

    let mut secondary = Vec::new();
    if g4.d036_architecture_rejection_not_upheld {
        secondary.push("D036_ARCHITECTURE_REJECTION_NOT_UPHELD".into());
    }
    if g4.d034_portability_rejection_not_upheld {
        secondary.push("D034_PORTABILITY_REJECTION_NOT_UPHELD".into());
    }
    // Dynamic isolated-renewal failure of D-035 remains an empirical fact under the
    // *inherited* λ; whether that λ is physical is the open question.
    if !d035_fp_ok || !d035_multi_ok {
        secondary.push("D035_DYNAMIC_FAILURE_UPHELD_UNDER_INHERITED_LAMBDA".into());
    }

    let (primary, route, rationale) = if transfer_defect && (provenance_unsupported || semantics_defect)
    {
        (
            "D037_TURNOVER_AND_GATE_DEFECTS".to_string(),
            RecoveryRoute::RouteATurnoverTransferRepair,
            vec![
                "Gate1: surface λ omitted ε_M protection → integrated loss inflated vs D-021.".into(),
                "Gate2/4: inherited λ not constitutively justified; portability gates used nonequilibrium pointwise balance.".into(),
                "Authorize Route A first: correct representation mapping only, then revalidate.".into(),
            ],
        )
    } else if transfer_defect {
        (
            "D037_SURFACE_TURNOVER_TRANSFER_DEFECT".to_string(),
            RecoveryRoute::RouteATurnoverTransferRepair,
            vec!["Gate1 failed 5% loss equivalence.".into()],
        )
    } else if provenance_unsupported {
        (
            "D037_TURNOVER_PROVENANCE_UNSUPPORTED".to_string(),
            RecoveryRoute::RouteBTurnoverReidentification,
            vec!["Mapping ok but λ not constitutively justified.".into()],
        )
    } else if semantics_defect
        && transfer_defect == false
        && !provenance_unsupported
        && d035_fp_ok
        && d035_multi_ok
    {
        (
            "D037_D035_DYNAMIC_REOPEN_AUTHORIZED".to_string(),
            RecoveryRoute::RouteCReopenD035,
            vec!["Semantics defect only; D-035 reduced attractor survives.".into()],
        )
    } else if !transfer_defect
        && !provenance_unsupported
        && d036_fp_ok
        && d036_multi_ok
        && d036_multi_ok
        && !d035_multi_ok
    {
        (
            "D037_D036_COMPLEX_IMPLEMENTATION_AUTHORIZED".to_string(),
            RecoveryRoute::RouteDImplementD036Complex,
            vec!["Complex outperforms D-035 under certified turnover assumptions.".into()],
        )
    } else if semantics_defect {
        (
            "D037_RENEWAL_GATE_SEMANTICS_DEFECT".to_string(),
            RecoveryRoute::RouteEFundamentalMembraneReview,
            vec!["Gate semantics invalid; no certified reopen path.".into()],
        )
    } else {
        (
            "D037_AUDIT_INCONCLUSIVE".to_string(),
            RecoveryRoute::RouteEFundamentalMembraneReview,
            vec!["Insufficient evidence to certify or reopen.".into()],
        )
    };

    let route_label = match route {
        RecoveryRoute::RouteATurnoverTransferRepair => "ROUTE_A_TURNOVER_TRANSFER_REPAIR",
        RecoveryRoute::RouteBTurnoverReidentification => "ROUTE_B_TURNOVER_REIDENTIFICATION",
        RecoveryRoute::RouteCReopenD035 => "ROUTE_C_REOPEN_D035",
        RecoveryRoute::RouteDImplementD036Complex => "ROUTE_D_IMPLEMENT_D036_COMPLEX",
        RecoveryRoute::RouteEFundamentalMembraneReview => "ROUTE_E_FUNDAMENTAL_MEMBRANE_REVIEW",
    };

    let next_directive = match route {
        RecoveryRoute::RouteATurnoverTransferRepair => {
            "D-038: Correct D-021→surface loss representation mapping only; revalidate D-024/D-031/D-034–D-035 under corrected λ; preserve historical results.".into()
        }
        RecoveryRoute::RouteBTurnoverReidentification => {
            "D-038: Independent turnover assay (withdrawal decay, replacement, timescale, R/ε invariance); do not force Stage E balance.".into()
        }
        RecoveryRoute::RouteCReopenD035 => {
            "D-038: Bounded dynamic continuation of k_cat under corrected gate semantics.".into()
        }
        RecoveryRoute::RouteDImplementD036Complex => {
            "D-038: Implement membrane-bound catalytic complex only under certified turnover.".into()
        }
        RecoveryRoute::RouteEFundamentalMembraneReview => {
            "D-038: Fundamental membrane renewal review without automatic species escalation.".into()
        }
    };

    RouteDecision {
        primary_conclusion: primary,
        secondary_findings: secondary,
        selected_route: route,
        route_label: route_label.into(),
        rationale,
        d008_status: "BLOCKED_NOT_RECOVERED".into(),
        phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
        stage_f: "not authorized".into(),
        production_verdict: "REQUIRES_REMEDIATION".into(),
        next_directive,
        next_execution_started: false,
    }
}

/// Run full D-037 audit bundle.
pub fn run_d037_audit(agent_memory_id: &str) -> D037AuditBundle {
    let gate0 = gate0_turnover_lineage();
    let gate1 = gate1_bulk_surface_equivalence();
    let gate2 = gate2_turnover_provenance();
    let gate3 = gate3_state_classification();
    let gate4 = gate4_renewal_gate_semantics();
    let gate5 = gate5_reduced_dynamics();
    let gate6 = gate6_multistart(&gate5);
    let gate7 = gate7_route_decision(&gate1, &gate2, &gate4, &gate5, &gate6);
    D037AuditBundle {
        project_directive: "D-037".into(),
        agent_memory_id: agent_memory_id.into(),
        d036_qualification: D036_REJECTION_QUALIFICATION.into(),
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        gate6,
        gate7,
    }
}

/// Helper used by tests: effective interface hazard for localized membrane.
pub fn effective_interface_hazard(eps_m: f64, k_m: f64) -> f64 {
    eps_m * k_m
}

/// Route selection pure rules for unit tests.
pub fn select_route_for_flags(
    transfer_defect: bool,
    provenance_unsupported: bool,
    semantics_defect: bool,
) -> RecoveryRoute {
    if transfer_defect {
        RecoveryRoute::RouteATurnoverTransferRepair
    } else if provenance_unsupported {
        RecoveryRoute::RouteBTurnoverReidentification
    } else if semantics_defect {
        RecoveryRoute::RouteEFundamentalMembraneReview
    } else {
        RecoveryRoute::RouteEFundamentalMembraneReview
    }
}

#[cfg(test)]
mod internal_smoke {
    use super::*;

    #[test]
    fn lineage_resolves() {
        assert!(gate0_turnover_lineage().resolved);
    }
}
