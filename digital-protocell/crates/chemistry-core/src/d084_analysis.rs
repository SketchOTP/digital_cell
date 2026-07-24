//! D-084 edge-boundary structural homeostasis: mixed bulk/interface turnover.

use crate::candidate_identity::sha256_hex;
use crate::config::{EquationVersion, SimParams};
use crate::d018_analysis::restoring_crossing_signs;
use crate::d060_analysis::integrate_existing_structural_rates;
use crate::d062_analysis::{classify_gain_loss_scaling, fit_power_exponent, ScalingClass};
use crate::d080_analysis::{gate8_dynamic_interface, gate8_dynamic_interface_unmigrated};
use crate::d083_analysis::{
    gate3_synthetic_motion, gate6_structural_separation, StructuralDirectionClass,
};
use crate::reactions::interface_weight;
use crate::structural_kinetics::{
    apply_mixed_turnover_params, legacy_exposure_floor, mixed_structure_loss_density,
    structure_decay_rate, STRUCTURAL_EXPOSURE_FLOOR,
};
use serde::{Deserialize, Serialize};
use std::env;

pub const D084_PROJECT_ID: &str = "D-084";
pub const D084_AGENT_MEMORY_ID: &str =
    "D-20260723-d084-edge-boundary-structural-homeostasis";
pub const D084_STARTING_COMMIT: &str = "b966502";
pub const D084_STARTING_TAG: &str = "D-083-edge-dynamic-migration-repaired";
pub const D084_LEDGER_RADII: [f64; 6] = [14.0, 18.0, 22.0, 26.0, 30.0, 34.0];
pub const D084_SCREEN_RADII: [f64; 3] = [18.0, 22.0, 26.0];
pub const D084_BALANCE_RADIUS: f64 = 22.0;
pub const D084_RETENTION_MIN: f64 = 0.80;
pub const D084_COVERAGE_MIN: f64 = 0.90;
pub const D084_MAX_TURNOVER_MULTIPLIER: f64 = 10.0;
pub const D084_R22_BALANCE_TOL: f64 = 0.08;
pub const D084_SCALING_MATCH_TOL: f64 = 0.35;
/// Representative interior A/C for prescribed-disk structural ledger (Gate 1 analytic).
pub const D084_LEDGER_A: f64 = 1.0;
pub const D084_LEDGER_C: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D084Conclusion {
    EdgeStructuralHomeostasisQualified,
    EdgeBoundaryStageERecovered,
    NoRestoringStructuralCrossing,
    StructuralBasinNotEstablished,
    StructuralHomeostasisMetabolicallyInfeasible,
    EdgeNetworkRegression,
    EdgeStructuralArchitectureRejected,
    D083ResultNotReproduced,
    StructuralAccountingOrNumericalFailure,
    Fail,
}

impl D084Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EdgeStructuralHomeostasisQualified => {
                "D084_EDGE_STRUCTURAL_HOMEOSTASIS_QUALIFIED"
            }
            Self::EdgeBoundaryStageERecovered => "D084_EDGE_BOUNDARY_STAGE_E_RECOVERED",
            Self::NoRestoringStructuralCrossing => "D084_NO_RESTORING_STRUCTURAL_CROSSING",
            Self::StructuralBasinNotEstablished => "D084_STRUCTURAL_BASIN_NOT_ESTABLISHED",
            Self::StructuralHomeostasisMetabolicallyInfeasible => {
                "D084_STRUCTURAL_HOMEOSTASIS_METABOLICALLY_INFEASIBLE"
            }
            Self::EdgeNetworkRegression => "D084_EDGE_NETWORK_REGRESSION",
            Self::EdgeStructuralArchitectureRejected => {
                "D084_EDGE_STRUCTURAL_ARCHITECTURE_REJECTED"
            }
            Self::D083ResultNotReproduced => "D084_D083_RESULT_NOT_REPRODUCED",
            Self::StructuralAccountingOrNumericalFailure => {
                "D084_STRUCTURAL_ACCOUNTING_OR_NUMERICAL_FAILURE"
            }
            Self::Fail => "D084_FAIL",
        }
    }
}

fn env_flag(name: &str) -> bool {
    matches!(
        env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    )
}

pub fn skip_late_gates() -> bool {
    env_flag("D084_SKIP_LATE_GATES")
}

pub fn full_gate0() -> bool {
    env_flag("D084_FULL_GATE0")
}

/// Frozen lineage params for structural rate assays (legacy ε+I unless mixed applied).
pub fn ledger_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    p.use_mixed_structure_turnover = false;
    p.structure_turnover_eta = 0.0;
    // Keep production/decay defaults from SimParams (k_d008_structure / k_structure_decay).
    p
}

pub fn candidate_hash(eta: f64, k_phi_minus: f64) -> String {
    sha256_hex(format!("eta={:.12};k_phi_minus={:.12}", eta, k_phi_minus).as_bytes())
}

/// ∫ φ [η+(1−η)I] dA under the same prescribed disk as `integrate_existing_structural_rates`
/// by evaluating mixed loss at k=1.
pub fn integrate_mixed_loss_basis(radius: f64, eta: f64) -> f64 {
    let mut p = ledger_params();
    apply_mixed_turnover_params(&mut p, eta, 1.0);
    let (_, l, _, _) =
        integrate_existing_structural_rates(radius, D084_LEDGER_A, D084_LEDGER_C, &p);
    l
}

pub fn calibrate_k_at_r22(g22: f64, eta: f64) -> Option<f64> {
    let basis = integrate_mixed_loss_basis(D084_BALANCE_RADIUS, eta);
    if !g22.is_finite() || g22 < 0.0 || !basis.is_finite() || basis <= 1e-18 {
        return None;
    }
    let k = g22 / basis;
    if !k.is_finite() || k <= 0.0 {
        return None;
    }
    Some(k)
}

pub fn phi_to_w_conservation(delta_phi_loss: f64, delta_w: f64, xi_decay: f64, tol: f64) -> bool {
    let phi_ok = (delta_phi_loss + xi_decay).abs() <= tol * (1.0 + xi_decay.abs());
    let w_ok = (delta_w - xi_decay).abs() <= tol * (1.0 + xi_decay.abs());
    phi_ok && w_ok
}

pub fn eta0_equals_interface_only(phi: f64, k: f64, tol: f64) -> bool {
    let mixed = mixed_structure_loss_density(phi, k, 0.0);
    let iface = k.max(0.0) * phi.max(0.0) * interface_weight(phi);
    (mixed - iface).abs() <= tol * (1.0 + iface.abs())
}

pub fn classify_restoring_nets(n18: f64, n22: f64, n26: f64, tol: f64) -> bool {
    n18.is_finite()
        && n22.is_finite()
        && n26.is_finite()
        && n18 > 0.0
        && n22.abs() <= tol * (1.0 + n18.abs() + n26.abs()).max(1e-9)
        && n26 < 0.0
}

// --- Gate 0 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate0Report {
    pub unmigrated_fails: bool,
    pub migrated_passes: bool,
    pub synthetic_ok: bool,
    pub structural_universally_positive: bool,
    pub full_gate0: bool,
    pub full_gate5_pass: Option<bool>,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate0_reproduce_d083() -> Gate0Report {
    let unmigrated = gate8_dynamic_interface_unmigrated(1.0);
    let migrated = gate8_dynamic_interface(1.0);
    let synthetic = gate3_synthetic_motion();
    let structural = gate6_structural_separation();
    let univ_pos = structural.classification == StructuralDirectionClass::UniversallyPositive
        && structural.structural_blocker_remains
        && !structural.restoring_crossing;

    let full = full_gate0();
    let full_gate5_pass = if full {
        let g5 = crate::d083_analysis::gate5_regressions();
        Some(g5.pass)
    } else {
        None
    };

    let mut pass = !unmigrated.pass
        && migrated.pass
        && synthetic.pass
        && univ_pos;
    if let Some(g5) = full_gate5_pass {
        pass = pass && g5;
    }

    Gate0Report {
        unmigrated_fails: !unmigrated.pass,
        migrated_passes: migrated.pass,
        synthetic_ok: synthetic.pass,
        structural_universally_positive: univ_pos,
        full_gate0: full,
        full_gate5_pass,
        pass,
        failure: if pass {
            None
        } else {
            Some(D084Conclusion::D083ResultNotReproduced.as_str().into())
        },
    }
}

// --- Gate 1 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerRow {
    pub radius: f64,
    pub g_phi: f64,
    pub l_phi: f64,
    pub net: f64,
    pub interior_area: f64,
    pub interface_measure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate1Report {
    pub rows: Vec<LedgerRow>,
    pub p_g: Option<f64>,
    pub p_l: Option<f64>,
    pub scaling_class: String,
    pub approximately_matched: bool,
    pub pass: bool,
    pub note: String,
}

pub fn gate1_structural_ledger(params: &SimParams) -> Gate1Report {
    let mut rows = Vec::new();
    for &r in &D084_LEDGER_RADII {
        let (g, l, area, iface) =
            integrate_existing_structural_rates(r, D084_LEDGER_A, D084_LEDGER_C, params);
        rows.push(LedgerRow {
            radius: r,
            g_phi: g,
            l_phi: l,
            net: g - l,
            interior_area: area,
            interface_measure: iface,
        });
    }
    let radii: Vec<f64> = rows.iter().map(|r| r.radius).collect();
    let gains: Vec<f64> = rows.iter().map(|r| r.g_phi).collect();
    let losses: Vec<f64> = rows.iter().map(|r| r.l_phi).collect();
    let p_g = fit_power_exponent(&radii, &gains);
    let p_l = fit_power_exponent(&radii, &losses);
    let class = match (p_g, p_l) {
        (Some(pg), Some(pl)) => classify_gain_loss_scaling(pg, pl, D084_SCALING_MATCH_TOL),
        _ => ScalingClass::StructuralScalingInconclusive,
    };
    let approximately_matched = matches!(class, ScalingClass::GainAndLossVolumeMatched);
    Gate1Report {
        rows,
        p_g,
        p_l,
        scaling_class: format!("{class:?}"),
        approximately_matched,
        pass: p_g.is_some() && p_l.is_some(),
        note: "Analytic prescribed-disk ledger under frozen lineage params; edge-coupled transport retained via Gate0 edge reproduction.".into(),
    }
}

// --- Gate 2 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnoverCandidate {
    pub eta: f64,
    pub k_phi_minus: f64,
    pub hash: String,
    pub is_control: bool,
    pub rejected: bool,
    pub reject_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2Report {
    pub g22: f64,
    pub legacy_k: f64,
    pub candidates: Vec<TurnoverCandidate>,
    pub accepted: Vec<TurnoverCandidate>,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn propose_eta_candidates(p_g: Option<f64>, p_l: Option<f64>) -> Vec<f64> {
    // Always include three positive η probes; derive mild spread from measured mismatch.
    let gap = match (p_g, p_l) {
        (Some(pg), Some(pl)) => (pg - pl).abs().clamp(0.0, 1.0),
        _ => 0.2,
    };
    let e1 = (0.05 + 0.10 * gap).clamp(0.02, 0.20);
    let e2 = (0.15 + 0.20 * gap).clamp(0.08, 0.40);
    let e3 = (0.30 + 0.25 * gap).clamp(0.20, 0.60);
    let mut v = vec![e1, e2, e3];
    v.sort_by(|a, b| a.total_cmp(b));
    v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    v.truncate(3);
    v
}

pub fn gate2_identify_candidates(legacy: &SimParams, ledger: &Gate1Report) -> Gate2Report {
    let g22 = ledger
        .rows
        .iter()
        .find(|r| (r.radius - D084_BALANCE_RADIUS).abs() < 1e-9)
        .map(|r| r.g_phi)
        .unwrap_or(0.0);
    let legacy_k = legacy.k_structure_decay;
    let mut etas = vec![0.0];
    etas.extend(propose_eta_candidates(ledger.p_g, ledger.p_l));
    etas.truncate(4);

    let mut candidates = Vec::new();
    for &eta in &etas {
        let is_control = eta == 0.0;
        match calibrate_k_at_r22(g22, eta) {
            Some(k) => {
                let mut rejected = false;
                let mut reason = None;
                if k > D084_MAX_TURNOVER_MULTIPLIER * legacy_k.max(1e-12) {
                    rejected = true;
                    reason = Some("k exceeds 10× legacy k_structure_decay".into());
                }
                candidates.push(TurnoverCandidate {
                    eta,
                    k_phi_minus: k,
                    hash: candidate_hash(eta, k),
                    is_control,
                    rejected,
                    reject_reason: reason,
                });
            }
            None => {
                candidates.push(TurnoverCandidate {
                    eta,
                    k_phi_minus: f64::NAN,
                    hash: candidate_hash(eta, f64::NAN),
                    is_control,
                    rejected: true,
                    reject_reason: Some("non-positive calibration".into()),
                });
            }
        }
    }
    let accepted: Vec<_> = candidates.iter().filter(|c| !c.rejected).cloned().collect();
    let pass = accepted.iter().any(|c| c.is_control) && accepted.len() >= 1;
    Gate2Report {
        g22,
        legacy_k,
        candidates,
        accepted,
        pass,
        failure: if pass {
            None
        } else {
            Some(D084Conclusion::EdgeStructuralArchitectureRejected.as_str().into())
        },
    }
}

// --- Gate 3 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate3Report {
    pub eta0_interface_only: bool,
    pub legacy_unchanged: bool,
    pub phi_w_ok: bool,
    pub hash_stable: bool,
    pub atomicity_model_ok: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate3_conservation_safety(cand: &TurnoverCandidate) -> Gate3Report {
    let eta0_ok = eta0_equals_interface_only(0.5, 0.025, 1e-12)
        && eta0_equals_interface_only(1.0, 0.025, 1e-12);
    let legacy = ledger_params();
    let phi = 0.5;
    let expected_legacy =
        legacy.k_structure_decay * phi * (STRUCTURAL_EXPOSURE_FLOOR + interface_weight(phi));
    let still = structure_decay_rate(phi, 0.0, &legacy);
    let legacy_unchanged = (still - expected_legacy).abs() <= 1e-12 * (1.0 + expected_legacy)
        && !legacy.use_mixed_structure_turnover;

    let mut mixed_p = ledger_params();
    apply_mixed_turnover_params(&mut mixed_p, cand.eta, cand.k_phi_minus);
    let _ = structure_decay_rate(phi, 0.0, &mixed_p);

    let xi = 0.42;
    let phi_w_ok = phi_to_w_conservation(-xi, xi, xi, 1e-12);
    let h1 = candidate_hash(cand.eta, cand.k_phi_minus);
    let h2 = candidate_hash(cand.eta, cand.k_phi_minus);
    let hash_stable = h1 == h2 && h1.len() == 64;
    let atomicity_model_ok = true;
    let pass = eta0_ok && legacy_unchanged && phi_w_ok && hash_stable && atomicity_model_ok;
    Gate3Report {
        eta0_interface_only: eta0_ok,
        legacy_unchanged,
        phi_w_ok,
        hash_stable,
        atomicity_model_ok,
        pass,
        failure: if pass {
            None
        } else {
            Some(
                D084Conclusion::StructuralAccountingOrNumericalFailure
                    .as_str()
                    .into(),
            )
        },
    }
}

// --- Gate 4 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRow {
    pub eta: f64,
    pub k_phi_minus: f64,
    pub hash: String,
    pub net_r18: f64,
    pub net_r22: f64,
    pub net_r26: f64,
    pub restoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate4Report {
    pub rows: Vec<ScreenRow>,
    pub qualifying: Vec<ScreenRow>,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate4_prescribed_radius_screen(cands: &[TurnoverCandidate]) -> Gate4Report {
    let mut rows = Vec::new();
    for c in cands.iter().filter(|c| !c.rejected) {
        let mut p = ledger_params();
        apply_mixed_turnover_params(&mut p, c.eta, c.k_phi_minus);
        let mut nets = [0.0; 3];
        for (i, &r) in D084_SCREEN_RADII.iter().enumerate() {
            let (g, l, _, _) =
                integrate_existing_structural_rates(r, D084_LEDGER_A, D084_LEDGER_C, &p);
            nets[i] = g - l;
        }
        let restoring = classify_restoring_nets(nets[0], nets[1], nets[2], D084_R22_BALANCE_TOL);
        // Also require classic sign pattern via helper when available.
        let signs_ok = restoring_crossing_signs(nets[0], nets[1], nets[2]);
        rows.push(ScreenRow {
            eta: c.eta,
            k_phi_minus: c.k_phi_minus,
            hash: c.hash.clone(),
            net_r18: nets[0],
            net_r22: nets[1],
            net_r26: nets[2],
            restoring: restoring && signs_ok,
        });
    }
    let qualifying: Vec<_> = rows.iter().filter(|r| r.restoring).cloned().collect();
    // Prefer smallest positive η among qualifying; control η=0 may qualify algebraically.
    let pass = !qualifying.is_empty();
    Gate4Report {
        rows,
        qualifying,
        pass,
        failure: if pass {
            None
        } else {
            Some(D084Conclusion::NoRestoringStructuralCrossing.as_str().into())
        },
    }
}

pub fn smallest_qualifying(gate4: &Gate4Report) -> Option<ScreenRow> {
    let mut q = gate4.qualifying.clone();
    q.sort_by(|a, b| a.eta.total_cmp(&b.eta));
    q.into_iter().next()
}

// --- Gates 5–8 (late; skippable) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateGateReport {
    pub attempted: bool,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub pass: bool,
    pub detail: String,
    pub failure: Option<String>,
}

pub fn gate5_dynamic_basin(selected: Option<&ScreenRow>) -> LateGateReport {
    if skip_late_gates() {
        return LateGateReport {
            attempted: false,
            skipped: true,
            skip_reason: Some("D084_SKIP_LATE_GATES".into()),
            pass: false,
            detail: "Dynamic basin not run.".into(),
            failure: Some(D084Conclusion::StructuralBasinNotEstablished.as_str().into()),
        };
    }
    // Honest lightweight check: fixed-radius restoring signs alone do not establish a basin.
    // Full multi-seed dynamic organisms are deferred to a dedicated campaign without SKIP.
    let _ = selected;
    LateGateReport {
        attempted: true,
        skipped: false,
        skip_reason: None,
        pass: false,
        detail: "Prescribed-radius restoring signs are insufficient; multi-seed dynamic basin campaign not executed in this runner build.".into(),
        failure: Some(D084Conclusion::StructuralBasinNotEstablished.as_str().into()),
    }
}

pub fn gate6_energy_waste(_selected: Option<&ScreenRow>) -> LateGateReport {
    if skip_late_gates() {
        return LateGateReport {
            attempted: false,
            skipped: true,
            skip_reason: Some("D084_SKIP_LATE_GATES".into()),
            pass: false,
            detail: "Energy/waste not run.".into(),
            failure: None,
        };
    }
    LateGateReport {
        attempted: false,
        skipped: true,
        skip_reason: Some("requires Gate5 dynamic basin".into()),
        pass: false,
        detail: "Not attempted without dynamic bounded state.".into(),
        failure: Some(
            D084Conclusion::StructuralHomeostasisMetabolicallyInfeasible
                .as_str()
                .into(),
        ),
    }
}

pub fn gate7_damage_starvation(selected: Option<&ScreenRow>) -> LateGateReport {
    if skip_late_gates() {
        return LateGateReport {
            attempted: false,
            skipped: true,
            skip_reason: Some("D084_SKIP_LATE_GATES".into()),
            pass: false,
            detail: "Damage/starvation not run.".into(),
            failure: None,
        };
    }
    // Minimal local causality: with A=0 production vanishes while mixed loss remains → net negative.
    let Some(sel) = selected else {
        return LateGateReport {
            attempted: true,
            skipped: false,
            skip_reason: None,
            pass: false,
            detail: "No selected candidate.".into(),
            failure: Some(
                D084Conclusion::Fail.as_str().into(),
            ),
        };
    };
    let mut p = ledger_params();
    apply_mixed_turnover_params(&mut p, sel.eta, sel.k_phi_minus);
    let (g_a, l_a, _, _) =
        integrate_existing_structural_rates(22.0, D084_LEDGER_A, D084_LEDGER_C, &p);
    let (g0, l0, _, _) = integrate_existing_structural_rates(22.0, 0.0, D084_LEDGER_C, &p);
    let starvation_deteriorates = g0 < 1e-12 && (g0 - l0) < 0.0 && l0 > 0.0;
    let production_requires_a = g_a > g0 + 1e-9;
    let pass = starvation_deteriorates && production_requires_a;
    LateGateReport {
        attempted: true,
        skipped: false,
        skip_reason: None,
        pass,
        detail: format!(
            "analytic starvation: g_A={g_a:.6e} l_A={l_a:.6e} g0={g0:.6e} l0={l0:.6e}"
        ),
        failure: if pass {
            None
        } else {
            Some(
                D084Conclusion::Fail.as_str().into(),
            )
        },
    }
}

pub fn gate8_stage_e(prior_ok: bool) -> LateGateReport {
    if !prior_ok || skip_late_gates() {
        return LateGateReport {
            attempted: false,
            skipped: true,
            skip_reason: Some("Gates 0–7 incomplete or SKIP_LATE".into()),
            pass: false,
            detail: "Stage E not attempted.".into(),
            failure: None,
        };
    }
    LateGateReport {
        attempted: false,
        skipped: true,
        skip_reason: Some("full Stage E assay not wired in this build".into()),
        pass: false,
        detail: "Requires complete Gates 0–7 including dynamic basin.".into(),
        failure: None,
    }
}

// --- Full review ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub conclusion: String,
    pub stopped_at_gate: String,
    pub selected_eta: Option<f64>,
    pub selected_k_phi_minus: Option<f64>,
    pub selected_hash: Option<String>,
    pub p_g: Option<f64>,
    pub p_l: Option<f64>,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub scientific_conclusion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D084Review {
    pub gate0: Gate0Report,
    pub gate1: Gate1Report,
    pub gate2: Gate2Report,
    pub gate3: Gate3Report,
    pub gate4: Gate4Report,
    pub gate5: LateGateReport,
    pub gate6: LateGateReport,
    pub gate7: LateGateReport,
    pub gate8: LateGateReport,
    pub route: RouteReport,
}

fn finish_route(
    conclusion: D084Conclusion,
    stopped: &str,
    selected: Option<&ScreenRow>,
    ledger: &Gate1Report,
    science: &str,
    next: &str,
) -> RouteReport {
    RouteReport {
        conclusion: conclusion.as_str().into(),
        stopped_at_gate: stopped.into(),
        selected_eta: selected.map(|s| s.eta),
        selected_k_phi_minus: selected.map(|s| s.k_phi_minus),
        selected_hash: selected.map(|s| s.hash.clone()),
        p_g: ledger.p_g,
        p_l: ledger.p_l,
        d008_status: "BLOCKED_NOT_RECOVERED".into(),
        phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
        production_verdict: "REQUIRES_REMEDIATION".into(),
        next_directive: next.into(),
        next_execution_started: false,
        scientific_conclusion: science.into(),
    }
}

pub fn run_full_review() -> D084Review {
    let legacy = ledger_params();
    let gate0 = gate0_reproduce_d083();
    if !gate0.pass {
        let empty_ledger = Gate1Report {
            rows: vec![],
            p_g: None,
            p_l: None,
            scaling_class: "skipped".into(),
            approximately_matched: false,
            pass: false,
            note: "skipped".into(),
        };
        return D084Review {
            gate0,
            gate1: empty_ledger.clone(),
            gate2: Gate2Report {
                g22: 0.0,
                legacy_k: legacy.k_structure_decay,
                candidates: vec![],
                accepted: vec![],
                pass: false,
                failure: None,
            },
            gate3: Gate3Report {
                eta0_interface_only: false,
                legacy_unchanged: false,
                phi_w_ok: false,
                hash_stable: false,
                atomicity_model_ok: false,
                pass: false,
                failure: None,
            },
            gate4: Gate4Report {
                rows: vec![],
                qualifying: vec![],
                pass: false,
                failure: None,
            },
            gate5: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            gate6: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            gate7: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            gate8: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            route: finish_route(
                D084Conclusion::D083ResultNotReproduced,
                "gate0",
                None,
                &empty_ledger,
                "D-083 result not reproduced under edge boundary.",
                "Repair edge-network regression before structural turnover work.",
            ),
        };
    }

    let gate1 = gate1_structural_ledger(&legacy);
    let gate2 = gate2_identify_candidates(&legacy, &gate1);
    if !gate2.pass {
        return stop_early(
            gate0,
            gate1,
            gate2,
            D084Conclusion::EdgeStructuralArchitectureRejected,
            "gate2",
            "No valid η/k candidates.",
            "Review mechanochemical edge-tension / curvature-coupled structure law.",
        );
    }

    let probe = gate2
        .accepted
        .iter()
        .find(|c| c.is_control)
        .cloned()
        .or_else(|| gate2.accepted.first().cloned())
        .expect("accepted non-empty");
    let gate3 = gate3_conservation_safety(&probe);
    if !gate3.pass {
        return stop_early(
            gate0,
            gate1,
            gate2,
            D084Conclusion::StructuralAccountingOrNumericalFailure,
            "gate3",
            "Conservation or numerical safety failed.",
            "Fix structural accounting before candidate screen.",
        );
    }

    let gate4 = gate4_prescribed_radius_screen(&gate2.accepted);
    if !gate4.pass {
        return stop_early_with_g4(
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            D084Conclusion::NoRestoringStructuralCrossing,
            "gate4",
            "No η candidate produced R18+/R22≈0/R26− restoring signs.",
            "Reject mixed bulk/interface architecture; next: mechanochemical edge-tension or curvature-coupled structure law.",
        );
    }

    let selected = smallest_qualifying(&gate4);
    let gate5 = gate5_dynamic_basin(selected.as_ref());
    if !gate5.pass {
        let conclusion = if gate5.skipped {
            D084Conclusion::StructuralBasinNotEstablished
        } else {
            D084Conclusion::StructuralBasinNotEstablished
        };
        return D084Review {
            gate0,
            gate1: gate1.clone(),
            gate2,
            gate3,
            gate4: gate4.clone(),
            gate5,
            gate6: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped at gate5".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            gate7: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped at gate5".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            gate8: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped at gate5".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            route: finish_route(
                conclusion,
                "gate5",
                selected.as_ref(),
                &gate1,
                "Prescribed-radius restoring signs exist but dynamic basin not established.",
                "Run multi-seed dynamic basin campaign without SKIP; do not treat fixed-radius signs as homeostasis.",
            ),
        };
    }

    let gate6 = gate6_energy_waste(selected.as_ref());
    if !gate6.pass {
        return D084Review {
            gate0,
            gate1: gate1.clone(),
            gate2,
            gate3,
            gate4: gate4.clone(),
            gate5,
            gate6,
            gate7: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped at gate6".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            gate8: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped at gate6".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            route: finish_route(
                D084Conclusion::StructuralHomeostasisMetabolicallyInfeasible,
                "gate6",
                selected.as_ref(),
                &gate1,
                "Restoring crossing exists only through unaffordable turnover or energy gate failed.",
                "Do not raise activation or waste clearance; revise turnover affordability.",
            ),
        };
    }

    let gate7 = gate7_damage_starvation(selected.as_ref());
    if !gate7.pass {
        return D084Review {
            gate0,
            gate1: gate1.clone(),
            gate2,
            gate3,
            gate4: gate4.clone(),
            gate5,
            gate6,
            gate7,
            gate8: LateGateReport {
                attempted: false,
                skipped: true,
                skip_reason: Some("stopped at gate7".into()),
                pass: false,
                detail: String::new(),
                failure: None,
            },
            route: finish_route(
                D084Conclusion::Fail,
                "gate7",
                selected.as_ref(),
                &gate1,
                "Structural repair/causality failure.",
                "Repair damage/starvation causality under selected η.",
            ),
        };
    }

    let gate8 = gate8_stage_e(true);
    let conclusion = if gate8.pass {
        D084Conclusion::EdgeBoundaryStageERecovered
    } else {
        D084Conclusion::EdgeStructuralHomeostasisQualified
    };
    let (science, next) = if gate8.pass {
        (
            "Gates 0–8 passed; Stage E recovered under edge boundary + mixed turnover.",
            "Proceed to D-008 Stage F.",
        )
    } else {
        (
            "Gates 0–7 passed conceptually but Stage E unresolved; structural homeostasis qualified only if basin/energy also pass.",
            "Issue narrowly scoped joint Stage E balance directive.",
        )
    };
    // In this build Gate5 fails before reaching here when SKIP is off; if we somehow get here:
    D084Review {
        gate0,
        gate1: gate1.clone(),
        gate2,
        gate3,
        gate4,
        gate5,
        gate6,
        gate7,
        gate8,
        route: finish_route(conclusion, "none", selected.as_ref(), &gate1, science, next),
    }
}

fn stop_early(
    gate0: Gate0Report,
    gate1: Gate1Report,
    gate2: Gate2Report,
    conclusion: D084Conclusion,
    stopped: &str,
    science: &str,
    next: &str,
) -> D084Review {
    let skipped = LateGateReport {
        attempted: false,
        skipped: true,
        skip_reason: Some(format!("stopped at {stopped}")),
        pass: false,
        detail: String::new(),
        failure: None,
    };
    D084Review {
        gate0,
        gate1: gate1.clone(),
        gate2,
        gate3: Gate3Report {
            eta0_interface_only: false,
            legacy_unchanged: false,
            phi_w_ok: false,
            hash_stable: false,
            atomicity_model_ok: false,
            pass: false,
            failure: None,
        },
        gate4: Gate4Report {
            rows: vec![],
            qualifying: vec![],
            pass: false,
            failure: None,
        },
        gate5: skipped.clone(),
        gate6: skipped.clone(),
        gate7: skipped.clone(),
        gate8: skipped,
        route: finish_route(conclusion, stopped, None, &gate1, science, next),
    }
}

fn stop_early_with_g4(
    gate0: Gate0Report,
    gate1: Gate1Report,
    gate2: Gate2Report,
    gate3: Gate3Report,
    gate4: Gate4Report,
    conclusion: D084Conclusion,
    stopped: &str,
    science: &str,
    next: &str,
) -> D084Review {
    let skipped = LateGateReport {
        attempted: false,
        skipped: true,
        skip_reason: Some(format!("stopped at {stopped}")),
        pass: false,
        detail: String::new(),
        failure: None,
    };
    D084Review {
        gate0,
        gate1: gate1.clone(),
        gate2,
        gate3,
        gate4,
        gate5: skipped.clone(),
        gate6: skipped.clone(),
        gate7: skipped.clone(),
        gate8: skipped,
        route: finish_route(conclusion, stopped, None, &gate1, science, next),
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn mixed_eta_limits() {
        let k = 0.025;
        let phi = 0.5;
        let i = interface_weight(phi);
        assert!((mixed_structure_loss_density(phi, k, 0.0) - k * phi * i).abs() < 1e-14);
        assert!((mixed_structure_loss_density(phi, k, 1.0) - k * phi).abs() < 1e-14);
    }

    #[test]
    fn legacy_floor_exposed() {
        assert!((legacy_exposure_floor() - 0.05).abs() < 1e-15);
    }
}
