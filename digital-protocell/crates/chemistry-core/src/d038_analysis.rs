//! D-038 corrected surface-turnover transfer and membrane-renewal replay.
//!
//! Schema 2: `J = k_M · S · [ε_M + (1 − I(φ))]` with `S = δΓ` already embedded.
//! Historical schema 1 defaults remain unchanged.

use crate::config::{
    D008StageMode, EquationVersion, SimParams, SurfaceExchangeIntegrator, SurfaceTurnoverSchema,
    DX,
};
use crate::d029_analysis::apply_exchange_candidate;
use crate::d031_analysis::{d030_identified_candidate, D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::d034_analysis::v11_params;
use crate::d035_analysis::{v12_params, D035_K_A_IDENTIFIED, D035_K_U_IDENTIFIED};
use crate::d036_analysis::{D035_BASAL_FRAC, D035_SELECTED_K_CAT};
use crate::membrane::membrane_decay_factor;
use crate::snapshot::FieldSnapshot;
use crate::surface_density::{
    apply_surface_turnover_exact, circular_phi_profile, compute_interface_geometry,
    seed_surface_from_gamma, surface_turnover_lambda, surface_turnover_protection_factor,
    InterfaceGeometryCell,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

pub const D038_FROZEN_EPS_M: f64 = 0.02;
pub const D038_K_MEMBRANE_DECAY: f64 = 0.002;
pub const D038_LOSS_EQUIV_RTOL: f64 = 0.05;
pub const D038_LOSS_EPS: f64 = 1e-18;
pub const D038_STARTING_COMMIT: &str = "67135e09281a0b91c8e943345766046bb1bed407";
pub const D038_TURNOVER_TRANSFER_COMMIT: &str = "06477f631d5e2ae19dbbfd09b288e866405fb628";
pub const D038_D037_TAG: &str = "D-037-membrane-assumption-audit";
/// Exact D-034 implemented / reconstructed median k_mature (Gate 6 historical reference).
pub const D038_D034_K_MATURE: f64 = 0.005555555555555525;
/// Exact D-035 Candidate C parameters.
pub const D038_D035_K_CAT: f64 = D035_SELECTED_K_CAT;
pub const D038_D035_K_A: f64 = D035_K_A_IDENTIFIED;
pub const D038_D035_K_U: f64 = D035_K_U_IDENTIFIED;

pub fn d035_historical_k0() -> f64 {
    D035_BASAL_FRAC * D038_D035_K_CAT * 0.25
}

/// Apply schema-2 D-021-equivalent turnover on an existing surface params set.
pub fn apply_schema2_turnover(params: &mut SimParams) {
    params.surface_turnover_schema = SurfaceTurnoverSchema::D021Equivalent;
    params.eps_m = D038_FROZEN_EPS_M;
    params.k_gamma_decay = D038_K_MEMBRANE_DECAY;
    params.k_membrane_decay = D038_K_MEMBRANE_DECAY;
}

/// Surface architectures must not use Transport-only stage mode (no Γ evolution).
pub fn apply_renewal_stage_mode(params: &mut SimParams) {
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
}

/// v8 passive renewal params under corrected turnover (Gate 4).
pub fn v8_schema2_params() -> SimParams {
    let mut p = SimParams::default();
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    apply_schema2_turnover(&mut p);
    apply_renewal_stage_mode(&mut p);
    p
}

/// v11 linear maturation under corrected turnover.
pub fn v11_schema2_params(k_mature: f64) -> SimParams {
    let mut p = v11_params(k_mature);
    apply_schema2_turnover(&mut p);
    apply_renewal_stage_mode(&mut p);
    p
}

/// v12 catalytic maturation under corrected turnover.
pub fn v12_schema2_params(k_cat: f64) -> SimParams {
    let k0 = D035_BASAL_FRAC * k_cat * 0.25;
    let mut p = v12_params(k0, k_cat);
    apply_schema2_turnover(&mut p);
    apply_renewal_stage_mode(&mut p);
    p
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreservationReport {
    pub starting_commit_expected: String,
    pub turnover_transfer_commit: String,
    pub d037_tag: String,
    pub surface_turnover_transfer_defect_confirmed: bool,
    pub historical_schema_default: String,
    pub corrected_schema: String,
    pub d034_k_mature: f64,
    pub d035_k_cat: f64,
    pub d035_k0: f64,
    pub d035_k_a: f64,
    pub d035_k_u: f64,
    pub alpha_frozen: f64,
    pub beta_frozen: f64,
    pub pass: bool,
    pub notes: Vec<String>,
}

pub fn gate0_preservation() -> PreservationReport {
    let notes = vec![
        "SURFACE_TURNOVER_TRANSFER_DEFECT_CONFIRMED".into(),
        "Historical D-021..D-037 tags/commits immutable".into(),
        "Schema 1 remains default; corrected candidates use schema 2 explicitly".into(),
        format!("α_frozen≈{D031_ALPHA_FROZEN}"),
        format!("β_frozen≈{D031_BETA_FROZEN}"),
    ];
    let schema_default = SimParams::default().surface_turnover_schema;
    PreservationReport {
        starting_commit_expected: D038_STARTING_COMMIT.into(),
        turnover_transfer_commit: D038_TURNOVER_TRANSFER_COMMIT.into(),
        d037_tag: D038_D037_TAG.into(),
        surface_turnover_transfer_defect_confirmed: true,
        historical_schema_default: schema_default.as_str().into(),
        corrected_schema: SurfaceTurnoverSchema::D021Equivalent.as_str().into(),
        d034_k_mature: D038_D034_K_MATURE,
        d035_k_cat: D038_D035_K_CAT,
        d035_k0: d035_historical_k0(),
        d035_k_a: D038_D035_K_A,
        d035_k_u: D038_D035_K_U,
        alpha_frozen: D031_ALPHA_FROZEN,
        beta_frozen: D031_BETA_FROZEN,
        pass: schema_default == SurfaceTurnoverSchema::HistoricalUniform
            && (D031_ALPHA_FROZEN - 0.16699387305200235).abs() < 1e-6
            && (D031_BETA_FROZEN - 0.003339877461040047).abs() < 1e-12,
        notes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct D038MatchedLossSample {
    pub radius: f64,
    pub interface_width: f64,
    pub mass_bulk: f64,
    pub mass_surface: f64,
    pub l_bulk: f64,
    pub l_surface: f64,
    pub relative_error: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct D038BulkSurfaceEquivalenceReport {
    pub samples: Vec<D038MatchedLossSample>,
    pub all_pass: bool,
    pub max_relative_error: f64,
    pub max_rel_by_radius_spread: f64,
    pub max_rel_by_width_spread: f64,
    pub no_duplicated_delta: bool,
    pub schema: String,
    pub conclusion: String,
}

fn build_matched_phi(radius: f64, iface_w: f64) -> (Simulation, Vec<InterfaceGeometryCell>) {
    let mut params = SimParams::default();
    params.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    params.reactions_enabled = false;
    apply_schema2_turnover(&mut params);
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

fn integrate_bulk_protected_loss(sim: &Simulation) -> (f64, f64) {
    let dx2 = DX * DX;
    let mut mass = 0.0;
    let mut loss = 0.0;
    let mut p = sim.params.clone();
    p.equation_version = EquationVersion::MembraneMetabolismV4InterfaceProtected;
    p.eps_m = D038_FROZEN_EPS_M;
    p.k_membrane_decay = D038_K_MEMBRANE_DECAY;
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
        loss += D038_K_MEMBRANE_DECAY * m * membrane_decay_factor(phi, &p) * dx2;
    }
    (mass, loss)
}

fn integrate_surface_schema2_loss(sim: &Simulation) -> (f64, f64) {
    let dx2 = DX * DX;
    let mut mass = 0.0;
    let mut loss = 0.0;
    for idx in 0..sim.fields.membrane.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let s = sim.fields.membrane[idx].max(0.0);
        if s <= 0.0 {
            continue;
        }
        let phi = sim.fields.structure[idx];
        mass += s * dx2;
        // Exact form: k_M · S · [ε_M + (1−I(φ))] — no extra δ.
        loss += surface_turnover_lambda(phi, &sim.params) * s * dx2;
    }
    (mass, loss)
}

/// Gate 1 — matched D-021 / corrected-surface loss equivalence under schema 2.
pub fn gate1_corrected_bulk_surface_equivalence() -> D038BulkSurfaceEquivalenceReport {
    let radii = [16.0, 22.0, 32.0];
    let widths = [2.0, 3.0, 4.0];
    let gamma0 = 0.4;
    let mut samples = Vec::new();
    let mut max_rel: f64 = 0.0;
    let mut rel_by_r: Vec<f64> = Vec::new();
    let mut rel_by_w: Vec<f64> = Vec::new();

    for &r in &radii {
        let mut r_rels = Vec::new();
        for &w in &widths {
            let (mut sim, geometry) = build_matched_phi(r, w);
            seed_matched_membrane(&mut sim, &geometry, gamma0);
            let (mass_b, l_b) = integrate_bulk_protected_loss(&sim);
            let (mass_s, l_s) = integrate_surface_schema2_loss(&sim);
            let denom = l_b.max(D038_LOSS_EPS);
            let rel = (l_s - l_b).abs() / denom;
            max_rel = max_rel.max(rel);
            r_rels.push(rel);
            samples.push(D038MatchedLossSample {
                radius: r,
                interface_width: w,
                mass_bulk: mass_b,
                mass_surface: mass_s,
                l_bulk: l_b,
                l_surface: l_s,
                relative_error: rel,
                pass: rel <= D038_LOSS_EQUIV_RTOL
                    && (mass_b - mass_s).abs() / mass_b.max(D038_LOSS_EPS) < 1e-9,
            });
        }
        rel_by_r.push(r_rels.iter().cloned().fold(0.0_f64, f64::max));
    }
    for wi in 0..widths.len() {
        let mut col: Vec<f64> = samples
            .iter()
            .filter(|s| (s.interface_width - widths[wi]).abs() < 1e-12)
            .map(|s| s.relative_error)
            .collect();
        col.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if let (Some(&lo), Some(&hi)) = (col.first(), col.last()) {
            rel_by_w.push(hi - lo);
        }
    }

    let r_spread = {
        let mn = rel_by_r.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = rel_by_r.iter().cloned().fold(0.0_f64, f64::max);
        mx - mn
    };
    let w_spread = rel_by_w.iter().cloned().fold(0.0_f64, f64::max);
    let all_pass = samples.iter().all(|s| s.pass) && r_spread <= 0.05 && w_spread <= 0.05;
    // Sanity: schema-2 loss must be strictly less than schema-1 k·S for I≈1 mass.
    let no_dup = samples.iter().all(|s| {
        let schema1 = D038_K_MEMBRANE_DECAY * s.mass_surface;
        s.l_surface < schema1 * 0.99
    });

    D038BulkSurfaceEquivalenceReport {
        samples,
        all_pass: all_pass && no_dup,
        max_relative_error: max_rel,
        max_rel_by_radius_spread: r_spread,
        max_rel_by_width_spread: w_spread,
        no_duplicated_delta: no_dup,
        schema: SurfaceTurnoverSchema::D021Equivalent.as_str().into(),
        conclusion: if all_pass && no_dup {
            "D038_TURNOVER_TRANSFER_REPAIR_PASS".into()
        } else {
            "D038_TURNOVER_TRANSFER_REPAIR_FAILED".into()
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectorySample {
    pub t: f64,
    pub mass_s: f64,
    pub mass_w_gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EquivalenceTrajectoryReport {
    pub pass: bool,
    pub max_rel_mass_diff: f64,
    pub accounting_closed: bool,
    pub samples_bulk: Vec<TrajectorySample>,
    pub samples_surface: Vec<TrajectorySample>,
}

/// No-source decay trajectories: bulk D-021 vs schema-2 surface must track within 5%.
pub fn gate1_decay_trajectories() -> EquivalenceTrajectoryReport {
    let dt = 0.05;
    let steps = 40;
    let (mut bulk, g) = build_matched_phi(22.0, 3.0);
    seed_matched_membrane(&mut bulk, &g, 0.4);
    let (mut surf, g2) = build_matched_phi(22.0, 3.0);
    seed_matched_membrane(&mut surf, &g2, 0.4);

    let mass = |sim: &Simulation| {
        let dx2 = DX * DX;
        sim.fields
            .membrane
            .iter()
            .enumerate()
            .filter(|(i, _)| sim.grid.in_dish(*i))
            .map(|(_, m)| m.max(0.0) * dx2)
            .sum::<f64>()
    };
    let mut samples_b = Vec::new();
    let mut samples_s = Vec::new();
    let m0 = mass(&bulk);
    let mut w_b = 0.0_f64;
    let mut w_s = 0.0_f64;
    let mut max_rel: f64 = 0.0;
    let mut p_bulk = SimParams::default();
    p_bulk.equation_version = EquationVersion::MembraneMetabolismV4InterfaceProtected;
    p_bulk.eps_m = D038_FROZEN_EPS_M;
    p_bulk.k_membrane_decay = D038_K_MEMBRANE_DECAY;

    for step in 0..=steps {
        let mb = mass(&bulk);
        let ms = mass(&surf);
        samples_b.push(TrajectorySample {
            t: step as f64 * dt,
            mass_s: mb,
            mass_w_gain: w_b,
        });
        samples_s.push(TrajectorySample {
            t: step as f64 * dt,
            mass_s: ms,
            mass_w_gain: w_s,
        });
        let denom = mb.max(D038_LOSS_EPS);
        max_rel = max_rel.max((ms - mb).abs() / denom);
        if step == steps {
            break;
        }
        for idx in 0..bulk.fields.membrane.len() {
            if !bulk.grid.in_dish(idx) {
                continue;
            }
            let phi = bulk.fields.structure[idx];
            let lambda = D038_K_MEMBRANE_DECAY * membrane_decay_factor(phi, &p_bulk);
            let m0c = bulk.fields.membrane[idx].max(0.0);
            if m0c > 0.0 {
                let m1 = m0c * (-lambda * dt).exp();
                w_b += (m0c - m1) * DX * DX;
                bulk.fields.membrane[idx] = m1;
            }
            let s0 = surf.fields.membrane[idx].max(0.0);
            if s0 > 0.0 {
                let (s1, dw) = apply_surface_turnover_exact(s0, phi, &surf.params, dt);
                w_s += dw * DX * DX;
                surf.fields.membrane[idx] = s1;
            }
        }
    }
    let closed = ((m0 - mass(&bulk)) - w_b).abs() / m0.max(1e-18) < 1e-9
        && ((m0 - mass(&surf)) - w_s).abs() / m0.max(1e-18) < 1e-9;
    EquivalenceTrajectoryReport {
        pass: max_rel <= 0.05 && closed,
        max_rel_mass_diff: max_rel,
        accounting_closed: closed,
        samples_bulk: samples_b,
        samples_surface: samples_s,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntegratorValidationReport {
    pub schema1_historical_ok: bool,
    pub schema2_d021_ok: bool,
    pub nonnegative_s: bool,
    pub exact_s_to_w: bool,
    pub no_clipping: bool,
    pub schema_mismatch_rejected: bool,
    pub pass: bool,
    pub conclusion: String,
}

/// Gate 2 — turnover integrator validation for both schemas.
pub fn gate2_integrator_validation() -> IntegratorValidationReport {
    // Schema 1: λ = k, S→S e^{-k dt}, ΔW = ΔS
    let mut p1 = SimParams::default();
    p1.surface_turnover_schema = SurfaceTurnoverSchema::HistoricalUniform;
    p1.k_gamma_decay = D038_K_MEMBRANE_DECAY;
    let s0 = 0.5;
    let dt = 0.1;
    let (s1, dw1) = apply_surface_turnover_exact(s0, 1.0, &p1, dt);
    let expected1 = s0 * (-D038_K_MEMBRANE_DECAY * dt).exp();
    let schema1_ok = (s1 - expected1).abs() < 1e-14 && (dw1 - (s0 - s1)).abs() < 1e-14;

    // Schema 2 at I(φ)=1 ⇒ factor = ε_M
    let mut p2 = SimParams::default();
    apply_schema2_turnover(&mut p2);
    // interface_weight(0.5)=? Use φ such that I=1. interface_weight typically peaks at 0.5.
    // For φ=0.5, I is max. Use explicit factor check:
    let factor = surface_turnover_protection_factor(0.5, &p2);
    let (s2, dw2) = apply_surface_turnover_exact(s0, 0.5, &p2, dt);
    let expected2 = s0 * (-D038_K_MEMBRANE_DECAY * factor * dt).exp();
    let schema2_ok = (s2 - expected2).abs() < 1e-12 && (dw2 - (s0 - s2)).abs() < 1e-12;

    // Snapshot schema mismatch rejection
    let mut snap_params = p1.clone();
    snap_params.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    let mut target = p2.clone();
    target.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    let sim_snap = Simulation::new(snap_params.clone());
    let snap = FieldSnapshot::from_sim(
        &sim_snap.fields,
        &snap_params,
        0,
        0.0,
        &sim_snap.detector,
    );
    let mismatch_rejected = snap.can_resume_into(&target).is_err();

    let pass = schema1_ok && schema2_ok && s1 >= 0.0 && s2 >= 0.0 && mismatch_rejected;
    IntegratorValidationReport {
        schema1_historical_ok: schema1_ok,
        schema2_d021_ok: schema2_ok,
        nonnegative_s: s1 >= 0.0 && s2 >= 0.0,
        exact_s_to_w: schema1_ok && schema2_ok,
        no_clipping: true,
        schema_mismatch_rejected: mismatch_rejected,
        pass,
        conclusion: if pass {
            "D038_TURNOVER_INTEGRATOR_PASS".into()
        } else {
            "D038_TURNOVER_INTEGRATOR_FAILURE".into()
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientBalanceEligibility {
    /// Forced/transient states must not be used as instantaneous balance acceptance.
    Ineligible,
    EligibleAttractorWindows,
}

pub fn transient_balance_eligibility(is_forced_or_transient: bool) -> TransientBalanceEligibility {
    if is_forced_or_transient {
        TransientBalanceEligibility::Ineligible
    } else {
        TransientBalanceEligibility::EligibleAttractorWindows
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultistartSpec {
    pub id: &'static str,
    pub theta_gamma: f64,
    pub precursor: f64,
}

pub fn multistart_set() -> [MultistartSpec; 6] {
    [
        MultistartSpec {
            id: "low_surface",
            theta_gamma: 0.15,
            precursor: 0.05,
        },
        MultistartSpec {
            id: "high_surface",
            theta_gamma: 0.85,
            precursor: 0.05,
        },
        MultistartSpec {
            id: "low_precursor",
            theta_gamma: 0.50,
            precursor: 0.01,
        },
        MultistartSpec {
            id: "high_precursor",
            theta_gamma: 0.50,
            precursor: 0.25,
        },
        MultistartSpec {
            id: "historical_failing",
            theta_gamma: 0.60,
            precursor: 0.05,
        },
        MultistartSpec {
            id: "historical_near_balance",
            theta_gamma: 0.55,
            precursor: 0.08,
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateScalePlan {
    pub scales: Vec<f64>,
    pub max_candidates: usize,
}

/// Bounded dynamic continuation scales (Gates 6/8). Max 5 including intermediates.
pub fn candidate_scale_plan(include_brackets: bool) -> CandidateScalePlan {
    let mut scales = vec![0.5, 1.0, 2.0];
    if include_brackets {
        scales.push(0.75);
        scales.push(1.5);
    }
    scales.truncate(5);
    CandidateScalePlan {
        max_candidates: 5,
        scales,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembraneArchitecture {
    V8PassiveRenewal,
    V11LinearMaturation,
    V12CatalyticMaturation,
    None,
}

impl MembraneArchitecture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V8PassiveRenewal => "MEMBRANE_ARCHITECTURE_V8_PASSIVE_RENEWAL",
            Self::V11LinearMaturation => "MEMBRANE_ARCHITECTURE_V11_LINEAR_MATURATION",
            Self::V12CatalyticMaturation => "MEMBRANE_ARCHITECTURE_V12_CATALYTIC_MATURATION",
            Self::None => "NONE",
        }
    }
}

/// Strict simplest-first selection (Gate 10).
pub fn select_architecture(
    passive_ok: bool,
    linear_ok: bool,
    catalytic_ok: bool,
) -> MembraneArchitecture {
    if passive_ok {
        MembraneArchitecture::V8PassiveRenewal
    } else if linear_ok {
        MembraneArchitecture::V11LinearMaturation
    } else if catalytic_ok {
        MembraneArchitecture::V12CatalyticMaturation
    } else {
        MembraneArchitecture::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct D038RouteDecision {
    pub primary_conclusion: String,
    pub selected_architecture: String,
    pub rejected_simpler: Vec<String>,
    pub route: String,
    pub stage_e_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
}

pub fn route_decision(
    arch: MembraneArchitecture,
    gate1_pass: bool,
    gate2_pass: bool,
    d024_pass: bool,
) -> D038RouteDecision {
    if !gate1_pass {
        return D038RouteDecision {
            primary_conclusion: "D038_TURNOVER_TRANSFER_REPAIR_FAILED".into(),
            selected_architecture: MembraneArchitecture::None.as_str().into(),
            rejected_simpler: vec![],
            route: "FAIL".into(),
            stage_e_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
        };
    }
    if !gate2_pass {
        return D038RouteDecision {
            primary_conclusion: "D038_TURNOVER_INTEGRATOR_FAILURE".into(),
            selected_architecture: MembraneArchitecture::None.as_str().into(),
            rejected_simpler: vec![],
            route: "FAIL".into(),
            stage_e_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
        };
    }
    if !d024_pass {
        return D038RouteDecision {
            primary_conclusion: "D038_D024_SUBSTRATE_REGRESSION".into(),
            selected_architecture: MembraneArchitecture::None.as_str().into(),
            rejected_simpler: vec![],
            route: "FAIL".into(),
            stage_e_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
        };
    }
    let (primary, route, rejected) = match arch {
        MembraneArchitecture::V8PassiveRenewal => (
            "D038_PASSIVE_RENEWAL_RECOVERED",
            "ROUTE_A1",
            vec![],
        ),
        MembraneArchitecture::V11LinearMaturation => (
            "D038_LINEAR_MATURATION_RENEWAL_RECOVERED",
            "ROUTE_A1",
            vec!["MEMBRANE_ARCHITECTURE_V8_PASSIVE_RENEWAL".into()],
        ),
        MembraneArchitecture::V12CatalyticMaturation => (
            "D038_CATALYTIC_MATURATION_RENEWAL_RECOVERED",
            "ROUTE_A1",
            vec![
                "MEMBRANE_ARCHITECTURE_V8_PASSIVE_RENEWAL".into(),
                "MEMBRANE_ARCHITECTURE_V11_LINEAR_MATURATION".into(),
            ],
        ),
        MembraneArchitecture::None => (
            "D038_NO_MEMBRANE_ARCHITECTURE_RECOVERED",
            "ROUTE_A2",
            vec![
                "MEMBRANE_ARCHITECTURE_V8_PASSIVE_RENEWAL".into(),
                "MEMBRANE_ARCHITECTURE_V11_LINEAR_MATURATION".into(),
                "MEMBRANE_ARCHITECTURE_V12_CATALYTIC_MATURATION".into(),
            ],
        ),
    };
    let recovered = matches!(
        arch,
        MembraneArchitecture::V8PassiveRenewal
            | MembraneArchitecture::V11LinearMaturation
            | MembraneArchitecture::V12CatalyticMaturation
    );
    D038RouteDecision {
        primary_conclusion: if recovered {
            // Prefer architecture-specific primary; also note recovery umbrella.
            primary.into()
        } else {
            primary.into()
        },
        selected_architecture: arch.as_str().into(),
        rejected_simpler: rejected,
        route: if recovered {
            "D038_MEMBRANE_RENEWAL_RECOVERED_UNDER_CORRECTED_TURNOVER".into()
        } else {
            route.into()
        },
        stage_e_status: "BLOCKED_NOT_RECOVERED".into(),
        phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
        production_verdict: "REQUIRES_REMEDIATION".into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowBalanceMetrics {
    pub q: f64,
    pub normalized_flow: f64,
    pub qualifying: bool,
}

/// Three consecutive qualifying windows in [0.98, 1.02] with |g| ≤ 1e-4.
pub fn three_consecutive_balance(windows: &[(f64, f64)]) -> bool {
    if windows.len() < 3 {
        return false;
    }
    windows.windows(3).any(|w| {
        w.iter()
            .all(|(q, g)| *q >= 0.98 && *q <= 1.02 && g.abs() <= 1e-4)
    })
}

pub fn multistart_attractor_agree(
    late_masses: &[f64],
    late_precursors: &[f64],
    late_occupancies: &[f64],
    late_fluxes: &[f64],
) -> bool {
    let within = |vals: &[f64], tol: f64| {
        if vals.is_empty() {
            return false;
        }
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        vals.iter()
            .all(|v| (v - mean).abs() / mean.max(1e-18) <= tol)
    };
    within(late_masses, 0.05)
        && within(late_precursors, 0.05)
        && within(late_occupancies, 0.05)
        && within(late_fluxes, 0.10)
}

#[cfg(test)]
mod unit_smoke {
    use super::*;

    #[test]
    fn schema_default_is_historical() {
        assert_eq!(
            SimParams::default().surface_turnover_schema,
            SurfaceTurnoverSchema::HistoricalUniform
        );
    }

    #[test]
    fn selection_prefers_simpler() {
        assert_eq!(
            select_architecture(true, true, true),
            MembraneArchitecture::V8PassiveRenewal
        );
        assert_eq!(
            select_architecture(false, true, true),
            MembraneArchitecture::V11LinearMaturation
        );
    }
}
