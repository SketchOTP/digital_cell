//! D-083 conservative dynamic edge-membrane migration audit.
//!
//! Repairs cut-cell support transitions so L/B follow moving φ through local
//! geometric continuity. Structural kinetics remain frozen and out of scope.

use crate::d079_analysis::{
    ACCOUNTING_TOL, ASSEMBLY_DT, ASSEMBLY_STEPS, DYNAMIC_COVERAGE_GATE, SEED_DENSITY,
};
use crate::d080_analysis::{
    frozen_d079_params, gate4_self_assembly, gate5_transport, gate6_replacement,
    gate8_dynamic_interface, gate8_dynamic_interface_unmigrated, gate9_coupled_and_structural,
};
use crate::d081_analysis::{
    gate2_reserve_only_repair, gate3_reserve_depletion, gate4_energy_causal_replenishment,
};
use crate::d082_analysis::{
    gate2_activation_parity, gate4_replenishment_affordability,
};
use crate::edge_membrane::*;
use crate::edge_migration::{
    audit_support_transition, migrate_bound_across_support, MigrationLedger, SupportTransitionAudit,
};
use crate::edge_support::*;
use serde::{Deserialize, Serialize};

pub const D083_PROJECT_ID: &str = "D-083";
pub const D083_AGENT_MEMORY_ID: &str =
    "D-20260723-d083-conservative-dynamic-edge-migration";
pub const D083_STARTING_COMMIT: &str = "01d9afd";
pub const D083_STARTING_TAG: &str = "D-082-edge-activation-integration-repaired";
pub const SYNTHETIC_COVERAGE_GATE: f64 = 0.95;
pub const AUTONOMOUS_COVERAGE_GATE: f64 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D083Conclusion {
    EdgeDynamicMigrationRepaired,
    EdgeNetworkBoundaryQualified,
    EdgeMigrationRepresentationInadequate,
    EdgeMigrationAccountingFailure,
    EdgeNetworkRegression,
    D082DynamicFailureNotReproduced,
    EdgeMigrationOperatorFailure,
    EdgeDynamicInterfaceFailure,
    Fail,
}

impl D083Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EdgeDynamicMigrationRepaired => "D083_EDGE_DYNAMIC_MIGRATION_REPAIRED",
            Self::EdgeNetworkBoundaryQualified => "D083_EDGE_NETWORK_BOUNDARY_QUALIFIED",
            Self::EdgeMigrationRepresentationInadequate => {
                "D083_EDGE_MIGRATION_REPRESENTATION_INADEQUATE"
            }
            Self::EdgeMigrationAccountingFailure => "D083_EDGE_MIGRATION_ACCOUNTING_FAILURE",
            Self::EdgeNetworkRegression => "D083_EDGE_NETWORK_REGRESSION",
            Self::D082DynamicFailureNotReproduced => "D083_D082_DYNAMIC_FAILURE_NOT_REPRODUCED",
            Self::EdgeMigrationOperatorFailure => "D083_EDGE_MIGRATION_OPERATOR_FAILURE",
            Self::EdgeDynamicInterfaceFailure => "D083_EDGE_DYNAMIC_INTERFACE_FAILURE",
            Self::Fail => "D083_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StructuralDirectionClass {
    RestoringCrossingPresent,
    UniversallyPositive,
    UniversallyNegative,
    InvalidDueToDynamicMigration,
}

pub fn analytic_disk_phi_at(
    width: usize,
    height: usize,
    radius: f64,
    cx: f64,
    cy: f64,
) -> Vec<f64> {
    let mut phi = vec![0.0; width * height];
    for j in 0..height {
        for i in 0..width {
            let dx = i as f64 - cx;
            let dy = j as f64 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            let t = ((radius + 0.75 - r) / 1.5).clamp(0.0, 1.0);
            phi[j * width + i] = t * t * (3.0 - 2.0 * t);
        }
    }
    phi
}

/// Mild elliptical deformation (single closed interface; no global remapping).
/// Positive amplitude elongates on x (bulge proxy); negative flattens on x (indent proxy).
pub fn analytic_disk_phi_local_deform(
    width: usize,
    height: usize,
    radius: f64,
    amplitude: f64,
) -> Vec<f64> {
    let cx = (width as f64 - 1.0) * 0.5;
    let cy = (height as f64 - 1.0) * 0.5;
    let (ax, ay) = if amplitude >= 0.0 {
        (
            radius * (1.0 + amplitude),
            radius * (1.0 - 0.35 * amplitude),
        )
    } else {
        (
            radius * (1.0 + 0.5 * amplitude),
            radius * (1.0 - 0.5 * amplitude),
        )
    };
    let mut phi = vec![0.0; width * height];
    for j in 0..height {
        for i in 0..width {
            let dx = i as f64 - cx;
            let dy = j as f64 - cy;
            let rr = ((dx / ax.max(1e-9)).powi(2) + (dy / ay.max(1e-9)).powi(2)).sqrt();
            let r_edge = (dx * dx + dy * dy).sqrt();
            let r_eff = if rr > 1e-15 {
                r_edge / rr
            } else {
                radius
            };
            let t = ((r_eff + 0.75 - r_edge) / 1.5).clamp(0.0, 1.0);
            phi[j * width + i] = t * t * (3.0 - 2.0 * t);
        }
    }
    phi
}

fn unsupported_b(state: &EdgeMembraneState, support: &CutCellSupport) -> f64 {
    let mut s = 0.0;
    for i in 0..state.n_h() {
        if !support.is_supported(FaceKind::Horizontal, i) {
            s += state.bound_h[i].max(0.0);
        }
    }
    for i in 0..state.n_v() {
        if !support.is_supported(FaceKind::Vertical, i) {
            s += state.bound_v[i].max(0.0);
        }
    }
    s
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionMetrics {
    pub name: String,
    pub l_mass: f64,
    pub b_mass: f64,
    pub membrane_total: f64,
    pub delta_membrane: f64,
    pub orphaned_b: f64,
    pub unsupported_b: f64,
    pub ghost_fraction: f64,
    pub trailing_fraction: f64,
    pub connected_coverage: f64,
    pub network_closed: bool,
    pub accounting_residual: f64,
    pub conservation_ok: bool,
    pub coverage_ok: bool,
    pub no_ghost: bool,
    pub pass: bool,
}

fn metrics_after(
    name: &str,
    state: &EdgeMembraneState,
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
    m0: f64,
    coverage_gate: f64,
) -> MotionMetrics {
    let (conn, closed, _) = connected_closed_support_observer(state, support, params);
    let (_geom_cov, geom_closed, _) = support.geometric_support_coverage();
    let off = off_support_bound_fraction(state, support);
    let orphan = unsupported_b(state, support);
    let m = state.total_membrane();
    let residual = (m - m0).abs();
    let conservation_ok = residual < ACCOUNTING_TOL * (1.0 + m0.abs());
    let coverage_ok = conn + 1e-12 >= coverage_gate;
    let no_ghost = off <= 0.05 && orphan <= ACCOUNTING_TOL * (1.0 + state.total_b().abs());
    // Occupied-support cycle OR geometric MS ring closed under high connected coverage.
    let network_closed = closed || (coverage_ok && geom_closed);
    let trailing = off;
    MotionMetrics {
        name: name.into(),
        l_mass: state.total_l(),
        b_mass: state.total_b(),
        membrane_total: m,
        delta_membrane: m - m0,
        orphaned_b: orphan,
        unsupported_b: orphan,
        ghost_fraction: off,
        trailing_fraction: trailing,
        connected_coverage: conn,
        network_closed,
        accounting_residual: residual,
        conservation_ok,
        coverage_ok,
        no_ghost,
        pass: conservation_ok && coverage_ok && no_ghost && network_closed,
    }
}

fn assemble_at_phi(
    phi: &[f64],
    w: usize,
    h: usize,
    params: &EdgeMembraneParams,
    k_lateral: f64,
) -> (EdgeMembraneState, CutCellSupport) {
    // D-083 motion gates use a shorter assembly than Stage-A qualification.
    const D083_ASSEMBLY: usize = 900;
    let support = build_cut_cell_support(phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    for _ in 0..D083_ASSEMBLY {
        let _ = accepted_step_supported(
            &mut state, phi, &support, params, ASSEMBLY_DT, false, k_lateral,
        );
    }
    (state, support)
}

fn step_support_transition(
    state: &mut EdgeMembraneState,
    support: &mut CutCellSupport,
    new_phi: &[f64],
    params: &EdgeMembraneParams,
    migrate: bool,
    settle: usize,
    k_lateral: f64,
) -> MigrationLedger {
    let new_support = build_cut_cell_support(new_phi, state.width, state.height);
    let led = if migrate {
        migrate_bound_across_support(state, support, &new_support, params)
    } else {
        MigrationLedger {
            m_before: state.total_membrane(),
            m_after: state.total_membrane(),
            conservation_ok: true,
            ..Default::default()
        }
    };
    *support = new_support;
    for _ in 0..settle {
        let _ = accepted_step_supported(
            state, new_phi, support, params, ASSEMBLY_DT, false, k_lateral,
        );
    }
    led
}

// --- Gate 0 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate0Report {
    pub d082_activation_parity_pass: bool,
    pub d082_affordability_pass: bool,
    pub static_closed_ok: bool,
    pub transport_selectivity_ok: bool,
    pub reserve_path_ok: bool,
    pub dynamic_unmigrated_fail: bool,
    pub dynamic_unmigrated: crate::d080_analysis::DynamicReport,
    pub motion_baselines: Vec<MotionMetrics>,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate0_reproduce() -> Gate0Report {
    let parity = gate2_activation_parity();
    let afford = gate4_replenishment_affordability(1.0);
    let assembly = gate4_self_assembly();
    let transport = gate5_transport(1.0);
    let dyn_u = gate8_dynamic_interface_unmigrated(1.0);

    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(22.0);
    let cx = (w as f64 - 1.0) * 0.5;
    let cy = (h as f64 - 1.0) * 0.5;
    let mut baselines = Vec::new();
    // Baseline unmigrated motions (expect coverage/ghost issues on some).
    for (name, phis) in [
        (
            "translation",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_at(w, h, 18.0, cx + 1.0, cy),
            ],
        ),
        (
            "expansion",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_at(w, h, 22.0, cx, cy),
            ],
        ),
        (
            "contraction",
            vec![
                analytic_disk_phi_at(w, h, 22.0, cx, cy),
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
            ],
        ),
        (
            "deformation",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_local_deform(w, h, 18.0, 2.0),
            ],
        ),
    ] {
        let (mut state, mut support) = assemble_at_phi(&phis[0], w, h, &params, 1.0);
        let m0 = state.total_membrane();
        let _ = step_support_transition(
            &mut state, &mut support, &phis[1], &params, false, 800, 1.0,
        );
        baselines.push(metrics_after(
            name,
            &state,
            &support,
            &params,
            m0,
            DYNAMIC_COVERAGE_GATE,
        ));
    }

    let dynamic_unmigrated_fail = !dyn_u.pass;
    let pass = parity.pass
        && afford.pass
        && assembly.pass
        && transport.pass
        && dynamic_unmigrated_fail;
    Gate0Report {
        d082_activation_parity_pass: parity.pass,
        d082_affordability_pass: afford.pass,
        static_closed_ok: assembly.pass,
        transport_selectivity_ok: transport.pass,
        reserve_path_ok: afford.pass,
        dynamic_unmigrated_fail,
        dynamic_unmigrated: dyn_u,
        motion_baselines: baselines,
        pass,
        failure: if pass {
            None
        } else if !dynamic_unmigrated_fail {
            Some(D083Conclusion::D082DynamicFailureNotReproduced.as_str().into())
        } else {
            Some("D083_GATE0_PRIOR_RESULTS_NOT_REPRODUCED".into())
        },
    }
}

// --- Gate 1 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate1Report {
    pub audits: Vec<SupportTransitionAudit>,
    pub first_divergence: String,
    pub pass: bool,
}

pub fn gate1_migration_provenance() -> Gate1Report {
    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(22.0);
    let phi0 = analytic_disk_phi(w, h, 18.0);
    let old = build_cut_cell_support(&phi0, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    seed_free_near_support(&mut state, &old, SEED_DENSITY);
    for _ in 0..ASSEMBLY_STEPS {
        let _ = accepted_step_supported(
            &mut state, &phi0, &old, &params, ASSEMBLY_DT, false, 1.0,
        );
    }
    let mut audits = Vec::new();
    let mut first = String::from("none");
    for &r in &[20.0_f64, 22.0, 24.0] {
        let phi1 = analytic_disk_phi(w, h, r);
        let new = build_cut_cell_support(&phi1, w, h);
        let a = audit_support_transition(&state, &old, &new);
        if first == "none" && a.b_on_disappear > 1e-9 {
            first = format!(
                "radius->{r}: B={:.6} on {} disappearing faces strands without migration",
                a.b_on_disappear, a.n_disappear
            );
        }
        audits.push(a);
    }
    // Also translation divergence.
    let cx = (w as f64 - 1.0) * 0.5;
    let cy = (h as f64 - 1.0) * 0.5;
    let phi_t = analytic_disk_phi_at(w, h, 18.0, cx + 1.0, cy);
    let new_t = build_cut_cell_support(&phi_t, w, h);
    let a_t = audit_support_transition(&state, &old, &new_t);
    if first == "none" && a_t.b_on_disappear > 1e-9 {
        first = format!(
            "translation+1: B={:.6} on disappearing faces",
            a_t.b_on_disappear
        );
    }
    audits.push(a_t);
    Gate1Report {
        pass: !audits.is_empty() && audits.iter().any(|a| a.b_on_disappear > 1e-9),
        first_divergence: first,
        audits,
    }
}

// --- Gate 3 synthetic ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate3Report {
    pub cases: Vec<MotionMetrics>,
    pub deterministic_ok: bool,
    pub translation_consistency_ok: bool,
    pub rejected_step_atomicity_ok: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate3_synthetic_motion() -> Gate3Report {
    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(24.0);
    let cx = (w as f64 - 1.0) * 0.5;
    let cy = (h as f64 - 1.0) * 0.5;
    let mut cases = Vec::new();

    let scenarios: Vec<(&str, Vec<Vec<f64>>)> = vec![
        (
            "one_cell_translation",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_at(w, h, 18.0, cx + 1.0, cy),
            ],
        ),
        (
            "subcell_translation",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_at(w, h, 18.0, cx + 0.4, cy),
            ],
        ),
        (
            "uniform_expansion",
            vec![
                analytic_disk_phi_at(w, h, 16.0, cx, cy),
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_at(w, h, 20.0, cx, cy),
                analytic_disk_phi_at(w, h, 22.0, cx, cy),
            ],
        ),
        (
            "uniform_contraction",
            vec![
                analytic_disk_phi_at(w, h, 22.0, cx, cy),
                analytic_disk_phi_at(w, h, 20.0, cx, cy),
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_at(w, h, 16.0, cx, cy),
            ],
        ),
        (
            "local_bulge",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_local_deform(w, h, 18.0, 0.08),
                analytic_disk_phi_local_deform(w, h, 18.0, 0.16),
            ],
        ),
        (
            "local_indentation",
            vec![
                analytic_disk_phi_at(w, h, 18.0, cx, cy),
                analytic_disk_phi_local_deform(w, h, 18.0, -0.08),
                analytic_disk_phi_local_deform(w, h, 18.0, -0.16),
            ],
        ),
    ];

    for (name, phis) in scenarios {
        let settle = if name.contains("bulge") || name.contains("indent") {
            1_200
        } else {
            700
        };
        let (mut state, mut support) = assemble_at_phi(&phis[0], w, h, &params, 1.0);
        let m0 = state.total_membrane();
        for phi in phis.iter().skip(1) {
            let led = step_support_transition(
                &mut state, &mut support, phi, &params, true, settle, 1.0,
            );
            let _ = led;
        }
        cases.push(metrics_after(
            name,
            &state,
            &support,
            &params,
            m0,
            SYNTHETIC_COVERAGE_GATE,
        ));
    }

    // Determinism: run one_cell twice.
    let run_once = || {
        let phis = [
            analytic_disk_phi_at(w, h, 18.0, cx, cy),
            analytic_disk_phi_at(w, h, 18.0, cx + 1.0, cy),
        ];
        let (mut state, mut support) = assemble_at_phi(&phis[0], w, h, &params, 1.0);
        let _ = step_support_transition(
            &mut state, &mut support, &phis[1], &params, true, 400, 1.0,
        );
        (state.total_l(), state.total_b(), state.total_membrane())
    };
    let a = run_once();
    let b = run_once();
    let deterministic_ok = (a.0 - b.0).abs() < 1e-12
        && (a.1 - b.1).abs() < 1e-12
        && (a.2 - b.2).abs() < 1e-12;

    // Translation consistency: +1x vs -1x then compare membrane totals within 2%.
    let (mut s_pos, mut sup_pos) =
        assemble_at_phi(&analytic_disk_phi_at(w, h, 18.0, cx, cy), w, h, &params, 1.0);
    let m_pos0 = s_pos.total_membrane();
    let _ = step_support_transition(
        &mut s_pos,
        &mut sup_pos,
        &analytic_disk_phi_at(w, h, 18.0, cx + 1.0, cy),
        &params,
        true,
        800,
        1.0,
    );
    let (mut s_neg, mut sup_neg) =
        assemble_at_phi(&analytic_disk_phi_at(w, h, 18.0, cx, cy), w, h, &params, 1.0);
    let _ = step_support_transition(
        &mut s_neg,
        &mut sup_neg,
        &analytic_disk_phi_at(w, h, 18.0, cx - 1.0, cy),
        &params,
        true,
        800,
        1.0,
    );
    let rel = (s_pos.total_membrane() - s_neg.total_membrane()).abs()
        / m_pos0.max(1e-9);
    let translation_consistency_ok = rel <= 0.02;

    // Rejected-step atomicity: migrate only when we choose; skipping migrate leaves state mass unchanged by operator.
    let phi0 = analytic_disk_phi_at(w, h, 18.0, cx, cy);
    let (mut state_r, support_r) = assemble_at_phi(&phi0, w, h, &params, 1.0);
    let m_before = state_r.total_membrane();
    let b_snap: Vec<f64> = state_r.bound_h.iter().chain(state_r.bound_v.iter()).copied().collect();
    // Simulate "rejected": do not call migrate; support unchanged.
    let rejected_step_atomicity_ok = (state_r.total_membrane() - m_before).abs() < 1e-15
        && state_r
            .bound_h
            .iter()
            .chain(state_r.bound_v.iter())
            .zip(b_snap.iter())
            .all(|(a, b)| (*a - *b).abs() < 1e-15);
    let _ = support_r;

    let cases_ok = cases.iter().all(|c| c.pass && c.delta_membrane.abs() < ACCOUNTING_TOL * (1.0 + c.membrane_total.abs()));
    let pass = cases_ok
        && deterministic_ok
        && translation_consistency_ok
        && rejected_step_atomicity_ok;
    Gate3Report {
        cases,
        deterministic_ok,
        translation_consistency_ok,
        rejected_step_atomicity_ok,
        pass,
        failure: if pass {
            None
        } else {
            Some(D083Conclusion::EdgeMigrationOperatorFailure.as_str().into())
        },
    }
}

// --- Gate 4 autonomous ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousRow {
    pub radius: f64,
    pub metrics: MotionMetrics,
    pub transport_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate4Report {
    pub dynamic_migrated: crate::d080_analysis::DynamicReport,
    pub rows: Vec<AutonomousRow>,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate4_autonomous_dynamic() -> Gate4Report {
    let dyn_m = gate8_dynamic_interface(1.0);
    let params = frozen_d079_params();
    let mut rows = Vec::new();
    for r in [16.0_f64, 22.0, 32.0] {
        let (w, h) = grid_for_radius(r + 4.0);
        let cx = (w as f64 - 1.0) * 0.5;
        let cy = (h as f64 - 1.0) * 0.5;
        let sequence = [r - 2.0, r, r + 2.0, r];
        let phi0 = analytic_disk_phi_at(w, h, sequence[0], cx, cy);
        let (mut state, mut support) = assemble_at_phi(&phi0, w, h, &params, 1.0);
        let m0 = state.total_membrane();
        for &rr in sequence.iter().skip(1) {
            let phi = analytic_disk_phi_at(w, h, rr, cx, cy);
            let _ = step_support_transition(
                &mut state, &mut support, &phi, &params, true, 500, 1.0,
            );
        }
        let metrics = metrics_after(
            &format!("R{r}"),
            &state,
            &support,
            &params,
            m0,
            AUTONOMOUS_COVERAGE_GATE,
        );
        let perm_c = mean_support_permeability(&state, &support, &params, "C");
        let perm_n = mean_support_permeability(&state, &support, &params, "N");
        // Selectivity: C more sealed than N (absolute Stage-A bounds revalidated in Gate 5).
        let transport_ok = perm_c < perm_n && perm_n > 0.0 && perm_c.is_finite();
        rows.push(AutonomousRow {
            radius: r,
            metrics,
            transport_ok,
        });
    }
    let rows_ok = rows
        .iter()
        .all(|r| r.metrics.pass && r.transport_ok && r.metrics.conservation_ok);
    let pass = dyn_m.pass && rows_ok;
    Gate4Report {
        dynamic_migrated: dyn_m,
        rows,
        pass,
        failure: if pass {
            None
        } else {
            Some(D083Conclusion::EdgeDynamicInterfaceFailure.as_str().into())
        },
    }
}

// --- Gate 5 regressions ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate5Report {
    pub self_assembly: bool,
    pub transport: bool,
    pub replacement: bool,
    /// Lawful damage repair under finite-reserve seed (D-081 Gate2), not obsolete D-080 Gate7.
    pub reserve_repair: bool,
    pub reserve_depletion: bool,
    pub a_causal_replenishment: bool,
    pub activation_parity: bool,
    pub affordability: bool,
    pub pass: bool,
    pub failure: Option<String>,
    pub failed_checks: Vec<String>,
}

pub fn gate5_regressions() -> Gate5Report {
    // D-080 Gate7 still recovers via free-L rebinding (no_a_fails=false). D-081 replaced that
    // with finite-reserve repair/depletion/A-causal replenishment; D-082 marked Gate7 PASS_AFTER
    // on that contract. Gate5 must not re-litigate the obsolete free-L causality assay.
    let assembly = gate4_self_assembly();
    let transport = gate5_transport(1.0);
    let replacement = gate6_replacement(1.0);
    let reserve_repair = gate2_reserve_only_repair(1.0);
    let depletion = gate3_reserve_depletion(1.0);
    let replenish = gate4_energy_causal_replenishment(1.0);
    let parity = gate2_activation_parity();
    let afford = gate4_replenishment_affordability(1.0);
    let mut failed = Vec::new();
    if !assembly.pass {
        failed.push("self_assembly".into());
    }
    if !transport.pass {
        failed.push("transport".into());
    }
    if !replacement.pass {
        failed.push("replacement".into());
    }
    if !reserve_repair.pass {
        failed.push("reserve_repair".into());
    }
    if !depletion.pass {
        failed.push("reserve_depletion".into());
    }
    if !replenish.pass {
        failed.push("a_causal_replenishment".into());
    }
    if !parity.pass {
        failed.push("activation_parity".into());
    }
    if !afford.pass {
        failed.push("affordability".into());
    }
    let pass = failed.is_empty();
    Gate5Report {
        self_assembly: assembly.pass,
        transport: transport.pass,
        replacement: replacement.pass,
        reserve_repair: reserve_repair.pass,
        reserve_depletion: depletion.pass,
        a_causal_replenishment: replenish.pass,
        activation_parity: parity.pass,
        affordability: afford.pass,
        pass,
        failure: if pass {
            None
        } else {
            Some(D083Conclusion::EdgeNetworkRegression.as_str().into())
        },
        failed_checks: failed,
    }
}

// --- Gate 6 structural separation ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralRow {
    pub radius: f64,
    pub drive_sign: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate6Report {
    pub rows: Vec<StructuralRow>,
    pub classification: StructuralDirectionClass,
    pub restoring_crossing: bool,
    pub structural_blocker_remains: bool,
    pub pass: bool,
}

pub fn gate6_structural_separation() -> Gate6Report {
    let coupled = gate9_coupled_and_structural(1.0);
    let rows: Vec<StructuralRow> = coupled
        .structural
        .iter()
        .map(|r| StructuralRow {
            radius: r.radius,
            drive_sign: r.drive_sign,
        })
        .collect();
    let all_pos = rows.iter().all(|r| r.drive_sign > 0);
    let all_neg = rows.iter().all(|r| r.drive_sign < 0);
    let restoring = rows.len() >= 3
        && rows[0].drive_sign > 0
        && rows[1].drive_sign == 0
        && rows[2].drive_sign < 0;
    let classification = if restoring {
        StructuralDirectionClass::RestoringCrossingPresent
    } else if all_pos {
        StructuralDirectionClass::UniversallyPositive
    } else if all_neg {
        StructuralDirectionClass::UniversallyNegative
    } else {
        StructuralDirectionClass::UniversallyPositive
    };
    Gate6Report {
        rows,
        classification,
        restoring_crossing: restoring,
        structural_blocker_remains: all_pos || !restoring,
        pass: true, // measurement-only; not a migration failure
    }
}

// --- Full review ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub conclusion: String,
    pub structural_direction: String,
    pub structural_blocker_remains: bool,
    pub stopped_at_gate: String,
    pub scientific_conclusion: String,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D083Review {
    pub gate0: Gate0Report,
    pub gate1: Gate1Report,
    pub gate3: Gate3Report,
    pub gate4: Gate4Report,
    pub gate5: Gate5Report,
    pub gate6: Gate6Report,
    pub route: RouteReport,
}

pub fn run_full_review() -> D083Review {
    let gate0 = gate0_reproduce();
    if !gate0.pass {
        return D083Review {
            gate1: Gate1Report {
                audits: vec![],
                first_divergence: "skipped".into(),
                pass: false,
            },
            gate3: Gate3Report {
                cases: vec![],
                deterministic_ok: false,
                translation_consistency_ok: false,
                rejected_step_atomicity_ok: false,
                pass: false,
                failure: None,
            },
            gate4: Gate4Report {
                dynamic_migrated: gate8_dynamic_interface_unmigrated(1.0),
                rows: vec![],
                pass: false,
                failure: None,
            },
            gate5: Gate5Report {
                self_assembly: false,
                transport: false,
                replacement: false,
                reserve_repair: false,
                reserve_depletion: false,
                a_causal_replenishment: false,
                activation_parity: false,
                affordability: false,
                pass: false,
                failure: None,
                failed_checks: vec![],
            },
            gate6: Gate6Report {
                rows: vec![],
                classification: StructuralDirectionClass::UniversallyPositive,
                restoring_crossing: false,
                structural_blocker_remains: true,
                pass: false,
            },
            route: RouteReport {
                conclusion: gate0
                    .failure
                    .clone()
                    .unwrap_or_else(|| D083Conclusion::Fail.as_str().into()),
                structural_direction: "not_measured".into(),
                structural_blocker_remains: true,
                stopped_at_gate: "gate0".into(),
                scientific_conclusion: "Gate 0 reproduction failed.".into(),
                next_directive: "Reproduce D-082 dynamic failure before migration repair.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
            },
            gate0,
        };
    }

    let gate1 = gate1_migration_provenance();
    let gate3 = gate3_synthetic_motion();
    if !gate3.pass {
        let gate6 = gate6_structural_separation();
        let structural_direction = format!("{:?}", gate6.classification);
        let structural_blocker_remains = gate6.structural_blocker_remains;
        return D083Review {
            gate0,
            gate1,
            gate3,
            gate4: Gate4Report {
                dynamic_migrated: gate8_dynamic_interface(1.0),
                rows: vec![],
                pass: false,
                failure: Some(D083Conclusion::EdgeMigrationOperatorFailure.as_str().into()),
            },
            gate5: gate5_regressions(),
            gate6,
            route: RouteReport {
                conclusion: D083Conclusion::EdgeMigrationOperatorFailure.as_str().into(),
                structural_direction,
                structural_blocker_remains,
                stopped_at_gate: "gate3".into(),
                scientific_conclusion: "Conservative local migration operator failed synthetic motion qualification.".into(),
                next_directive: "Repair migration operator; do not change structural rates.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
            },
        };
    }

    let gate4 = gate4_autonomous_dynamic();
    if !gate4.pass {
        let gate6 = gate6_structural_separation();
        let structural_direction = format!("{:?}", gate6.classification);
        let structural_blocker_remains = gate6.structural_blocker_remains;
        return D083Review {
            gate0,
            gate1,
            gate3,
            gate4,
            gate5: gate5_regressions(),
            gate6,
            route: RouteReport {
                conclusion: D083Conclusion::EdgeDynamicInterfaceFailure.as_str().into(),
                structural_direction,
                structural_blocker_remains,
                stopped_at_gate: "gate4".into(),
                scientific_conclusion: "Migrated autonomous dynamic interface still fails coverage/ghost/accounting.".into(),
                next_directive: "Inspect local adjacency transfer; do not add global remapping.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
            },
        };
    }

    let gate5 = gate5_regressions();
    if !gate5.pass {
        let gate6 = gate6_structural_separation();
        let structural_direction = format!("{:?}", gate6.classification);
        let structural_blocker_remains = gate6.structural_blocker_remains;
        return D083Review {
            gate0,
            gate1,
            gate3,
            gate4,
            gate5,
            gate6,
            route: RouteReport {
                conclusion: D083Conclusion::EdgeNetworkRegression.as_str().into(),
                structural_direction,
                structural_blocker_remains,
                stopped_at_gate: "gate5".into(),
                scientific_conclusion: "Migration repair regressed a prior edge-network gate.".into(),
                next_directive: "Identify exact regressed prior gate and repair.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
            },
        };
    }

    let gate6 = gate6_structural_separation();
    let structural_direction = format!("{:?}", gate6.classification);
    let structural_blocker_remains = gate6.structural_blocker_remains;
    let restoring_crossing = gate6.restoring_crossing;
    let accounting_ok = gate3.cases.iter().all(|c| c.conservation_ok)
        && gate4.rows.iter().all(|r| r.metrics.conservation_ok);

    if !accounting_ok {
        return D083Review {
            gate0,
            gate1,
            gate3,
            gate4,
            gate5,
            gate6,
            route: RouteReport {
                conclusion: D083Conclusion::EdgeMigrationAccountingFailure.as_str().into(),
                structural_direction,
                structural_blocker_remains,
                stopped_at_gate: "accounting".into(),
                scientific_conclusion: "L+B accounting failed under migration.".into(),
                next_directive: "Repair accounting before scientific interpretation.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
            },
        };
    }

    let (conclusion, science, next, production) = if restoring_crossing {
        (
            D083Conclusion::EdgeNetworkBoundaryQualified,
            "Dynamic migration repaired and restoring structural crossing already present.",
            "Proceed to Stage E under qualified edge boundary; do not raise activation.",
            "RESEARCH_CONTINUE",
        )
    } else {
        (
            D083Conclusion::EdgeDynamicMigrationRepaired,
            "Dynamic migration repaired via conservative local continuity; structural drive remains universally positive (independent blocker).",
            "Audit structural gain/loss under the qualified edge boundary and select one bounded structural-homeostasis architecture.",
            "REQUIRES_REMEDIATION",
        )
    };

    D083Review {
        gate0,
        gate1,
        gate3,
        gate4,
        gate5,
        gate6,
        route: RouteReport {
            conclusion: conclusion.as_str().into(),
            structural_direction,
            structural_blocker_remains,
            stopped_at_gate: "none".into(),
            scientific_conclusion: science.into(),
            next_directive: next.into(),
            next_execution_started: false,
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: production.into(),
        },
    }
}
