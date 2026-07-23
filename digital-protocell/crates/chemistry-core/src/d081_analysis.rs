//! D-081 edge-membrane reserve provenance and replenishment causality audit.
//!
//! Freezes D-080 cut-cell support and edge kinetics. Does not make binding
//! consume A. Audits whether D-080 Gate 7 is valid finite-reserve repair.

use crate::d079_analysis::{
    ACCOUNTING_TOL, ASSEMBLY_DT, ASSEMBLY_STEPS, A_RETENTION_GATE, C_RETENTION_GATE,
    DAMAGE_RECOVERY_GATE, DYNAMIC_COVERAGE_GATE, SEED_DENSITY,
};
use crate::d080_analysis::{
    frozen_d079_params, gate3_geometry_qualification, gate4_self_assembly, gate5_transport,
    gate6_replacement, gate7_damage_and_causality, gate8_dynamic_interface,
    gate9_coupled_and_structural, D080_STARTING_COMMIT, D080_STARTING_TAG,
};
use crate::edge_membrane::*;
use crate::edge_support::*;
use serde::{Deserialize, Serialize};

pub const D081_PROJECT_ID: &str = "D-081";
pub const D081_AGENT_MEMORY_ID: &str =
    "D-20260723-d081-edge-membrane-reserve-causality-audit";
pub const D081_STARTING_COMMIT: &str = "f5dc5a5";
pub const D081_STARTING_TAG: &str = "D-080-edge-network-requalification-fail";
pub const D080_PRIMARY: &str = "D080_EDGE_NETWORK_REPAIR_OR_CAUSALITY_FAILURE";
pub const D080_GATE7_PROVISIONAL: &str = "PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT";
pub const SEED_CONTRACT_V1: &str = "EDGE_MEMBRANE_SEED_CONTRACT_V1";
pub const SCOPE_AMENDMENT: &str = "PHASE1_EDGE_NETWORK_BOUNDARY_RESEARCH_AUTHORIZED";

const REPAIR_STEPS: usize = 10_000;
const DEPLETION_REPAIR_STEPS: usize = 3_000;
const REPLENISH_STEPS: usize = 4_000;
const AFFORD_STEPS: usize = 8_000;
const DENSITY_REL_TOL: f64 = 0.08;
const EXCESS_RESERVE_FRAC: f64 = 0.60; // >60% over full-ring capacity ⇒ EXCESS

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D081Route {
    EdgeReserveCausalityQualified,
    EdgeNetworkBoundaryQualified,
    EdgeMembraneSeedUnauthorized,
    ReserveNotFiniteOrNotConserved,
    MembraneReplenishmentNotEnergyCausal,
    EdgeMembraneProductionMetabolicallyInfeasible,
    EdgeNetworkDynamicInterfaceFailure,
    EdgeNetworkCoupledFailure,
    EdgeNetworkStructuralIncompatibility,
    D080ResultNotReproduced,
    Fail,
}

impl D081Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EdgeReserveCausalityQualified => "Route_edge_reserve_causality_qualified",
            Self::EdgeNetworkBoundaryQualified => "Route_edge_network_boundary_qualified",
            Self::EdgeMembraneSeedUnauthorized => "Route_edge_membrane_seed_unauthorized",
            Self::ReserveNotFiniteOrNotConserved => "Route_reserve_not_finite_or_not_conserved",
            Self::MembraneReplenishmentNotEnergyCausal => {
                "Route_membrane_replenishment_not_energy_causal"
            }
            Self::EdgeMembraneProductionMetabolicallyInfeasible => {
                "Route_edge_membrane_production_metabolically_infeasible"
            }
            Self::EdgeNetworkDynamicInterfaceFailure => {
                "Route_edge_network_dynamic_interface_failure"
            }
            Self::EdgeNetworkCoupledFailure => "Route_edge_network_coupled_failure",
            Self::EdgeNetworkStructuralIncompatibility => {
                "Route_edge_network_structural_incompatibility"
            }
            Self::D080ResultNotReproduced => "Route_d080_result_not_reproduced",
            Self::Fail => "Route_d081_fail",
        }
    }

    pub const fn conclusion(self) -> &'static str {
        match self {
            Self::EdgeReserveCausalityQualified => "D081_EDGE_RESERVE_CAUSALITY_QUALIFIED",
            Self::EdgeNetworkBoundaryQualified => "D081_EDGE_NETWORK_BOUNDARY_QUALIFIED",
            Self::EdgeMembraneSeedUnauthorized => "D081_EDGE_MEMBRANE_SEED_UNAUTHORIZED",
            Self::ReserveNotFiniteOrNotConserved => "D081_RESERVE_NOT_FINITE_OR_NOT_CONSERVED",
            Self::MembraneReplenishmentNotEnergyCausal => {
                "D081_MEMBRANE_REPLENISHMENT_NOT_ENERGY_CAUSAL"
            }
            Self::EdgeMembraneProductionMetabolicallyInfeasible => {
                "D081_EDGE_MEMBRANE_PRODUCTION_METABOLICALLY_INFEASIBLE"
            }
            Self::EdgeNetworkDynamicInterfaceFailure => {
                "D081_EDGE_NETWORK_DYNAMIC_INTERFACE_FAILURE"
            }
            Self::EdgeNetworkCoupledFailure => "D081_EDGE_NETWORK_COUPLED_FAILURE",
            Self::EdgeNetworkStructuralIncompatibility => {
                "D081_EDGE_NETWORK_STRUCTURAL_INCOMPATIBILITY"
            }
            Self::D080ResultNotReproduced => "D081_D080_RESULT_NOT_REPRODUCED",
            Self::Fail => "D081_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeedClassification {
    CapacityValidFiniteReserve,
    ExcessReserve,
    RadiusInconsistent,
    UnauthorizedMaterial,
    ProvenanceUnknown,
}

/// Membrane material ledger (unit cell volumes V_i=1; B stored as material mass).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MembraneLedger {
    pub m_l: f64,
    pub m_b: f64,
    pub m_mem: f64,
    pub m_b_ell: f64,
    pub interface_measure: f64,
    pub waste: f64,
}

pub fn ledger(state: &EdgeMembraneState, support: &CutCellSupport) -> MembraneLedger {
    let m_l = state.total_l();
    let m_b = state.total_b();
    let mut m_b_ell = 0.0;
    let mut interface_measure = 0.0;
    for (kind, idx) in support.supported_faces() {
        let ell = support.measure(kind, idx);
        interface_measure += ell;
        m_b_ell += state.bound_ref(kind)[idx] * ell;
    }
    MembraneLedger {
        m_l,
        m_b,
        m_mem: m_l + m_b,
        m_b_ell,
        interface_measure,
        waste: state.waste,
    }
}

pub fn seed_contract_identity(radius: f64, density: f64) -> String {
    format!("{SEED_CONTRACT_V1}|R={radius:.0}|density={density}|no_B_ring")
}

fn assemble(
    radius: f64,
    params: &EdgeMembraneParams,
    k_lateral_scale: f64,
) -> (EdgeMembraneState, Vec<f64>, CutCellSupport) {
    let (w, h) = grid_for_radius(radius);
    let phi = analytic_disk_phi(w, h, radius);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    for _ in 0..ASSEMBLY_STEPS {
        let _ = accepted_step_supported(
            &mut state,
            &phi,
            &support,
            params,
            ASSEMBLY_DT,
            false,
            k_lateral_scale,
        );
    }
    (state, phi, support)
}

fn run_steps(
    state: &mut EdgeMembraneState,
    phi: &[f64],
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
    k_lateral_scale: f64,
    steps: usize,
    allow_produce: bool,
) -> StepLedger {
    let mut acc = StepLedger::default();
    for _ in 0..steps {
        let led = accepted_step_supported(
            state,
            phi,
            support,
            params,
            ASSEMBLY_DT,
            allow_produce,
            k_lateral_scale,
        );
        acc.bind += led.bind;
        acc.unbind += led.unbind;
        acc.lateral += led.lateral;
        acc.produce += led.produce;
        acc.damage += led.damage;
    }
    acc
}

/// Remove membrane mass equal to `amount` from occupied supported faces → W.
pub fn damage_mass_amount(
    state: &mut EdgeMembraneState,
    support: &CutCellSupport,
    amount: f64,
    params: &EdgeMembraneParams,
) -> f64 {
    let mut targets: Vec<(FaceKind, usize)> = support
        .supported_faces()
        .into_iter()
        .filter(|(k, i)| state.bound_ref(*k)[*i] > params.occupied_theta * params.b_max * 0.05)
        .collect();
    targets.sort_by(|a, b| {
        state.bound_ref(b.0)[b.1]
            .partial_cmp(&state.bound_ref(a.0)[a.1])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut left = amount.max(0.0);
    let mut removed = 0.0;
    for &(kind, idx) in &targets {
        if left <= 1e-15 {
            break;
        }
        let b = state.bound_ref(kind)[idx];
        let take = b.min(left);
        state.bound_mut(kind)[idx] = b - take;
        state.waste += take;
        removed += take;
        left -= take;
    }
    removed
}

// ─── Gate 0 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate0Report {
    pub geometric_cov: f64,
    pub connected_cov: f64,
    pub transport_pass: bool,
    pub replacement_pass: bool,
    pub recovery: f64,
    pub no_a_recovers: bool,
    pub no_production_recovers: bool,
    pub k_lateral_scale: f64,
    pub d080_gate7_provisional: String,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate0_reproduce_d080() -> Gate0Report {
    let g3 = gate3_geometry_qualification();
    let g4 = gate4_self_assembly();
    let scale = g4.k_lateral_scale;
    let g5 = gate5_transport(scale);
    let g6 = gate6_replacement(scale);
    let g7 = gate7_damage_and_causality(scale);
    let geometric_cov = g3
        .rows
        .iter()
        .map(|r| r.geometric_coverage)
        .fold(1.0_f64, f64::min);
    let connected_cov = g4
        .rows
        .iter()
        .map(|r| r.connected_coverage.min(r.occupied_coverage))
        .fold(1.0_f64, f64::min);
    let no_a_recovers = !g7.no_a_fails;
    let no_production_recovers = !g7.no_production_fails;
    let pass = g3.pass
        && g4.pass
        && g5.pass
        && g6.pass
        && geometric_cov + 1e-12 >= 1.0
        && connected_cov + 1e-12 >= 1.0
        && g7.recovery + 1e-12 >= DAMAGE_RECOVERY_GATE
        && no_a_recovers
        && no_production_recovers;
    Gate0Report {
        geometric_cov,
        connected_cov,
        transport_pass: g5.pass,
        replacement_pass: g6.pass,
        recovery: g7.recovery,
        no_a_recovers,
        no_production_recovers,
        k_lateral_scale: scale,
        d080_gate7_provisional: D080_GATE7_PROVISIONAL.into(),
        pass,
        failure: if pass {
            None
        } else {
            Some("D081_D080_RESULT_NOT_REPRODUCED".into())
        },
    }
}

// ─── Gate 1 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedRow {
    pub radius: f64,
    pub n_supported: usize,
    pub interface_measure: f64,
    pub initial_m_l: f64,
    pub initial_m_b: f64,
    pub declared_total: f64,
    pub density_per_face: f64,
    pub density_per_measure: f64,
    pub full_ring_capacity: f64,
    pub reserve_over_capacity_frac: f64,
    pub no_completed_b_ring: bool,
    pub identity: String,
    pub hidden_material: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate1Report {
    pub contract: String,
    pub rows: Vec<SeedRow>,
    pub classification: SeedClassification,
    pub density_consistent: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate1_seed_provenance() -> Gate1Report {
    let params = frozen_d079_params();
    let mut rows = Vec::new();
    for &radius in &[16.0, 22.0, 32.0] {
        let (w, h) = grid_for_radius(radius);
        let phi = analytic_disk_phi(w, h, radius);
        let support = build_cut_cell_support(&phi, w, h);
        let mut state = EdgeMembraneState::new(w, h);
        let before = ledger(&state, &support);
        seed_free_near_support(&mut state, &support, SEED_DENSITY);
        let after = ledger(&state, &support);
        let n = support.n_supported();
        let declared = SEED_DENSITY * n as f64;
        let mut full_cap = 0.0;
        for (kind, idx) in support.supported_faces() {
            full_cap += support.face_capacity(kind, idx, params.b_max);
        }
        let dens_m = if after.interface_measure > 1e-15 {
            after.m_l / after.interface_measure
        } else {
            0.0
        };
        let over = if full_cap > 1e-15 {
            (after.m_l - full_cap) / full_cap
        } else {
            0.0
        };
        let (_, closed, _) = connected_closed_support_observer(&state, &support, &params);
        let hidden = (after.m_mem - before.m_mem - declared).abs() > ACCOUNTING_TOL * (1.0 + declared)
            || after.m_b > ACCOUNTING_TOL;
        rows.push(SeedRow {
            radius,
            n_supported: n,
            interface_measure: after.interface_measure,
            initial_m_l: after.m_l,
            initial_m_b: after.m_b,
            declared_total: declared,
            density_per_face: SEED_DENSITY,
            density_per_measure: dens_m,
            full_ring_capacity: full_cap,
            reserve_over_capacity_frac: over,
            no_completed_b_ring: !closed && after.m_b <= ACCOUNTING_TOL,
            identity: seed_contract_identity(radius, SEED_DENSITY),
            hidden_material: hidden,
        });
    }
    let dens: Vec<f64> = rows.iter().map(|r| r.density_per_measure).collect();
    let dens_mean = dens.iter().sum::<f64>() / dens.len().max(1) as f64;
    let density_consistent = dens
        .iter()
        .all(|d| (d - dens_mean).abs() <= DENSITY_REL_TOL * dens_mean.max(1e-9));
    let any_hidden = rows.iter().any(|r| r.hidden_material);
    let any_b = rows.iter().any(|r| !r.no_completed_b_ring || r.initial_m_b > ACCOUNTING_TOL);
    let max_over = rows
        .iter()
        .map(|r| r.reserve_over_capacity_frac)
        .fold(0.0_f64, f64::max);

    let classification = if any_hidden || any_b {
        if any_hidden {
            SeedClassification::UnauthorizedMaterial
        } else {
            SeedClassification::UnauthorizedMaterial
        }
    } else if !density_consistent {
        SeedClassification::RadiusInconsistent
    } else if max_over > EXCESS_RESERVE_FRAC {
        SeedClassification::ExcessReserve
    } else if rows.iter().all(|r| r.initial_m_l > 0.0 && r.density_per_face == SEED_DENSITY) {
        SeedClassification::CapacityValidFiniteReserve
    } else {
        SeedClassification::ProvenanceUnknown
    };

    let unauthorized = matches!(
        classification,
        SeedClassification::UnauthorizedMaterial
            | SeedClassification::ExcessReserve
            | SeedClassification::RadiusInconsistent
            | SeedClassification::ProvenanceUnknown
    );
    let pass = !unauthorized && matches!(classification, SeedClassification::CapacityValidFiniteReserve);
    Gate1Report {
        contract: SEED_CONTRACT_V1.into(),
        rows,
        classification,
        density_consistent,
        pass,
        failure: if pass {
            None
        } else {
            Some("D081_EDGE_MEMBRANE_SEED_UNAUTHORIZED".into())
        },
    }
}

// ─── Gate 2 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2Report {
    pub m_l_before: f64,
    pub m_b_before: f64,
    pub m_l_after_damage: f64,
    pub m_b_after_damage: f64,
    pub damaged: f64,
    pub m_l_after_repair: f64,
    pub m_b_after_repair: f64,
    pub rebound: f64,
    pub m_mem_conserved: bool,
    pub recovery: f64,
    pub network_closed: bool,
    pub transport_recovered: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate2_reserve_only_repair(k_lateral_scale: f64) -> Gate2Report {
    let params = frozen_d079_params();
    let (mut state, phi, support) = assemble(22.0, &params, k_lateral_scale);
    let led0 = ledger(&state, &support);
    let cov0 = support_coverage(&state, &support, &params);
    let perm0 = mean_support_permeability(&state, &support, &params, "C");
    let damaged = apply_damage_supported(&mut state, &support, 0.10, &params);
    let led_d = ledger(&state, &support);
    let perm_hole = mean_support_permeability(&state, &support, &params, "C");
    let m_mem_post_damage = led_d.m_mem;
    let m_l_d = led_d.m_l;
    let m_b_d = led_d.m_b;
    // Production disabled; existing L available.
    let mut p = params;
    p.k_produce = 0.0;
    state.activated = 0.0;
    let _ = run_steps(
        &mut state,
        &phi,
        &support,
        &p,
        k_lateral_scale,
        REPAIR_STEPS,
        false,
    );
    let led1 = ledger(&state, &support);
    let rebound = (led1.m_b - m_b_d).max(0.0);
    let cov1 = support_coverage(&state, &support, &params);
    let recovery = if cov0 > 1e-9 { cov1 / cov0 } else { 0.0 };
    let (_, closed, _) = connected_closed_support_observer(&state, &support, &params);
    let perm1 = mean_support_permeability(&state, &support, &params, "C");
    // Transport recovers relative to the open hole and stays in Stage A C envelope.
    let transport_recovered = perm_hole > perm0 + 1e-9
        && perm1 + 1e-9 < perm_hole
        && perm1 <= STAGE_A_C_PERM_MAX + 1e-6;
    let m_mem_conserved = (led1.m_mem - m_mem_post_damage).abs()
        < ACCOUNTING_TOL * (1.0 + m_mem_post_damage)
        && (led_d.m_mem + damaged - led0.m_mem).abs() < ACCOUNTING_TOL * (1.0 + led0.m_mem);
    let ml_decreased = led1.m_l <= m_l_d + ACCOUNTING_TOL * (1.0 + m_l_d);
    let mb_increased = led1.m_b + ACCOUNTING_TOL >= m_b_d;
    let no_new = led1.m_mem <= m_mem_post_damage + ACCOUNTING_TOL * (1.0 + m_mem_post_damage);
    let pass = recovery + 1e-12 >= DAMAGE_RECOVERY_GATE
        && m_mem_conserved
        && ml_decreased
        && mb_increased
        && no_new
        && closed
        && transport_recovered
        && rebound + ACCOUNTING_TOL >= (m_l_d - led1.m_l) - ACCOUNTING_TOL * 10.0;
    Gate2Report {
        m_l_before: led0.m_l,
        m_b_before: led0.m_b,
        m_l_after_damage: m_l_d,
        m_b_after_damage: m_b_d,
        damaged,
        m_l_after_repair: led1.m_l,
        m_b_after_repair: led1.m_b,
        rebound,
        m_mem_conserved,
        recovery,
        network_closed: closed,
        transport_recovered,
        pass,
        failure: if pass {
            None
        } else {
            Some("D081_RESERVE_NOT_FINITE_OR_NOT_CONSERVED".into())
        },
    }
}

// ─── Gate 3 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepletionEvent {
    pub event: usize,
    pub removed: f64,
    pub free_l: f64,
    pub bound_b: f64,
    pub m_mem: f64,
    pub recovery: f64,
    pub closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate3Report {
    pub starting_free_reserve: f64,
    pub original_bound: f64,
    pub damage_quantum: f64,
    pub events: Vec<DepletionEvent>,
    pub cumulative_rebound: f64,
    pub eventual_failure: bool,
    pub no_hidden_regen: bool,
    pub l_monotone_aside_unbind: bool,
    pub perm_rises_when_open: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate3_reserve_depletion(k_lateral_scale: f64) -> Gate3Report {
    let params = frozen_d079_params();
    let (mut state, phi, support) = assemble(22.0, &params, k_lateral_scale);
    let led0 = ledger(&state, &support);
    let starting_free = led0.m_l;
    let original_bound = led0.m_b;
    let quantum = 0.10 * original_bound.max(1e-9);
    let cov_ref = support_coverage(&state, &support, &params);
    let perm_intact = mean_support_permeability(&state, &support, &params, "C");
    let mut p = params;
    p.k_produce = 0.0;
    state.activated = 0.0;

    let mut events = Vec::new();
    let mut cumulative_rebound = 0.0;
    let mut m_mem_track = led0.m_mem;
    let mut no_hidden = true;
    let mut l_ok = true;
    let mut prev_l = led0.m_l;
    let mut eventual_failure = false;
    let mut perm_rises = true;

    for event in 1..=20 {
        let b_before = state.total_b();
        let removed = damage_mass_amount(&mut state, &support, quantum, &params);
        if removed < 1e-9 {
            eventual_failure = true;
            break;
        }
        let led_d = ledger(&state, &support);
        let perm_hole = mean_support_permeability(&state, &support, &params, "C");
        let m_b_d = led_d.m_b;
        let m_l_d = led_d.m_l;
        // After damage, membrane mass must drop by removed (to W).
        if (led_d.m_mem + removed - m_mem_track).abs() > ACCOUNTING_TOL * (1.0 + m_mem_track) {
            no_hidden = false;
        }
        if perm_hole + 1e-9 <= perm_intact {
            perm_rises = false;
        }
        let _ = run_steps(
            &mut state,
            &phi,
            &support,
            &p,
            k_lateral_scale,
            DEPLETION_REPAIR_STEPS,
            false,
        );
        let led1 = ledger(&state, &support);
        if led1.m_mem > led_d.m_mem + ACCOUNTING_TOL * (1.0 + led_d.m_mem) {
            no_hidden = false;
        }
        // Free L should not increase except via unbind (tracked loosely: L+B conserved).
        if led1.m_l > m_l_d + (b_before - m_b_d).abs() + ACCOUNTING_TOL * 10.0 {
            // Allow unbind-sourced L; forbid net creation beyond conservation.
            if led1.m_mem > led_d.m_mem + ACCOUNTING_TOL {
                l_ok = false;
            }
        }
        let rebound = (led1.m_b - m_b_d).max(0.0);
        cumulative_rebound += rebound;
        let cov1 = support_coverage(&state, &support, &params);
        let recovery = if cov_ref > 1e-9 { cov1 / cov_ref } else { 0.0 };
        let (_, closed, _) = connected_closed_support_observer(&state, &support, &params);
        if !closed {
            let perm_open = mean_support_permeability(&state, &support, &params, "C");
            // Loss of closure must raise permeability above the intact baseline.
            if perm_open + 1e-9 <= perm_intact {
                perm_rises = false;
            }
        }
        events.push(DepletionEvent {
            event,
            removed,
            free_l: led1.m_l,
            bound_b: led1.m_b,
            m_mem: led1.m_mem,
            recovery,
            closed,
        });
        m_mem_track = led1.m_mem;
        prev_l = led1.m_l;
        if starting_free < ACCOUNTING_TOL && rebound > ACCOUNTING_TOL * 10.0 {
            // rebound without free reserve implies unbind redistribution only — still conserved
        }
        if recovery < DAMAGE_RECOVERY_GATE || !closed {
            eventual_failure = true;
            break;
        }
        if led1.m_l < ACCOUNTING_TOL * 10.0 && event >= 2 {
            // continue until recovery fails
        }
    }
    if !eventual_failure {
        // If still recovering after many events, check cumulative vs starting free + unbind pool.
        // Conserved finite material: m_mem must have dropped; if still closed with full recovery
        // after removing > original_bound, that is non-finite.
        let total_removed: f64 = events.iter().map(|e| e.removed).sum();
        if total_removed > original_bound + starting_free + ACCOUNTING_TOL
            && events.last().map(|e| e.recovery >= DAMAGE_RECOVERY_GATE).unwrap_or(false)
        {
            no_hidden = false;
            eventual_failure = false;
        } else if events.len() >= 20
            && events.last().map(|e| e.recovery >= DAMAGE_RECOVERY_GATE).unwrap_or(false)
        {
            // Still fully recovering after 20 quanta — treat as non-depleting unless m_mem collapsed.
            let last = events.last().unwrap();
            if last.m_mem > 0.5 * led0.m_mem {
                no_hidden = false;
            } else {
                eventual_failure = true;
            }
        }
    }

    // Cumulative rebound from free pool cannot exceed starting free + unbind of remaining B.
    // Soft: cumulative_rebound should not exceed starting_free + original_bound (material ceiling).
    let within_material =
        cumulative_rebound <= starting_free + original_bound + ACCOUNTING_TOL * 100.0;

    let pass = eventual_failure && no_hidden && within_material && l_ok && perm_rises;
    Gate3Report {
        starting_free_reserve: starting_free,
        original_bound,
        damage_quantum: quantum,
        events,
        cumulative_rebound,
        eventual_failure,
        no_hidden_regen: no_hidden,
        l_monotone_aside_unbind: l_ok,
        perm_rises_when_open: perm_rises,
        pass,
        failure: if pass {
            None
        } else {
            Some("D081_RESERVE_NOT_FINITE_OR_NOT_CONSERVED".into())
        },
    }
}

// ─── Gate 4 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplenishArm {
    pub name: String,
    pub delta_m_mem: f64,
    pub delta_a: f64,
    pub produced_l: f64,
    pub stoichiometry_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate4Report {
    pub depleted_m_l: f64,
    pub depleted_m_b: f64,
    pub depleted_m_mem: f64,
    pub arms: Vec<ReplenishArm>,
    pub only_normal_increases: bool,
    pub post_replenish_repair: f64,
    pub post_repair_closed: bool,
    pub no_field_reset: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

fn deplete_to_reserve_exhausted(
    k_lateral_scale: f64,
) -> (EdgeMembraneState, Vec<f64>, CutCellSupport, f64) {
    let params = frozen_d079_params();
    let (mut state, phi, support) = assemble(22.0, &params, k_lateral_scale);
    let original_bound = state.total_b();
    let quantum = 0.10 * original_bound.max(1e-9);
    let mut p = params;
    p.k_produce = 0.0;
    state.activated = 0.0;
    let cov_ref = support_coverage(&state, &support, &params);
    for _ in 0..20 {
        let _ = damage_mass_amount(&mut state, &support, quantum, &params);
        let _ = run_steps(
            &mut state,
            &phi,
            &support,
            &p,
            k_lateral_scale,
            DEPLETION_REPAIR_STEPS,
            false,
        );
        let cov = support_coverage(&state, &support, &params);
        let recovery = if cov_ref > 1e-9 { cov / cov_ref } else { 0.0 };
        let (_, closed, _) = connected_closed_support_observer(&state, &support, &params);
        if recovery < DAMAGE_RECOVERY_GATE || !closed || state.total_l() < 1e-6 {
            break;
        }
    }
    // Ensure free reserve is low.
    for v in &mut state.free_l {
        *v = 0.0;
    }
    // Keep bound remnant as depleted network state (do not wipe B).
    (state, phi, support, original_bound)
}

pub fn gate4_energy_causal_replenishment(k_lateral_scale: f64) -> Gate4Report {
    let params = frozen_d079_params();
    let (base, phi, support, _orig_b) = deplete_to_reserve_exhausted(k_lateral_scale);
    let led0 = ledger(&base, &support);
    let snap = EdgeSnapshot::from_state(&base, &params);

    let mut arms = Vec::new();

    // Normal metabolism
    {
        let mut state = base.clone();
        snap.resume_into(&mut state).expect("snapshot resume");
        let mut p = params;
        p.k_produce = 0.5;
        state.activated = 5.0;
        let a0 = state.activated;
        let m0 = state.total_membrane();
        let led = run_steps(
            &mut state,
            &phi,
            &support,
            &p,
            k_lateral_scale,
            REPLENISH_STEPS,
            true,
        );
        let d_a = a0 - state.activated;
        let stoich = (led.produce - d_a * p.yield_l_from_a).abs()
            < ACCOUNTING_TOL * 10.0 * (1.0 + led.produce);
        arms.push(ReplenishArm {
            name: "normal_metabolism".into(),
            delta_m_mem: state.total_membrane() - m0,
            delta_a: -d_a,
            produced_l: led.produce,
            stoichiometry_ok: stoich,
        });
    }

    // No A
    {
        let mut state = base.clone();
        snap.resume_into(&mut state).expect("snapshot resume");
        let mut p = params;
        p.k_produce = 0.5;
        state.activated = 0.0;
        let m0 = state.total_membrane();
        let led = run_steps(
            &mut state,
            &phi,
            &support,
            &p,
            k_lateral_scale,
            REPLENISH_STEPS,
            true,
        );
        arms.push(ReplenishArm {
            name: "no_a".into(),
            delta_m_mem: state.total_membrane() - m0,
            delta_a: 0.0,
            produced_l: led.produce,
            stoichiometry_ok: led.produce.abs() < ACCOUNTING_TOL * 10.0,
        });
    }

    // Production knockout
    {
        let mut state = base.clone();
        snap.resume_into(&mut state).expect("snapshot resume");
        let mut p = params;
        p.k_produce = 0.0;
        state.activated = 5.0;
        let m0 = state.total_membrane();
        let led = run_steps(
            &mut state,
            &phi,
            &support,
            &p,
            k_lateral_scale,
            REPLENISH_STEPS,
            true,
        );
        arms.push(ReplenishArm {
            name: "production_knockout".into(),
            delta_m_mem: state.total_membrane() - m0,
            delta_a: 0.0,
            produced_l: led.produce,
            stoichiometry_ok: led.produce.abs() < ACCOUNTING_TOL * 10.0,
        });
    }

    let normal = arms.iter().find(|a| a.name == "normal_metabolism").unwrap();
    let no_a = arms.iter().find(|a| a.name == "no_a").unwrap();
    let ko = arms.iter().find(|a| a.name == "production_knockout").unwrap();
    let only_normal = normal.delta_m_mem > 1e-6
        && no_a.delta_m_mem <= ACCOUNTING_TOL * 10.0
        && ko.delta_m_mem <= ACCOUNTING_TOL * 10.0
        && normal.stoichiometry_ok
        && no_a.stoichiometry_ok
        && ko.stoichiometry_ok;

    // Post-replenishment repair from normal arm state.
    let mut state = base.clone();
    snap.resume_into(&mut state).expect("snapshot resume");
    let mut p = params;
    p.k_produce = 0.5;
    state.activated = 5.0;
    let _ = run_steps(
        &mut state,
        &phi,
        &support,
        &p,
        k_lateral_scale,
        REPLENISH_STEPS,
        true,
    );
    // Freeze production; attempt one lawful 10% repair from replenished L.
    p.k_produce = 0.0;
    let a_before_repair = state.activated;
    let cov0 = support_coverage(&state, &support, &params).max(1e-9);
    let _ = apply_damage_supported(&mut state, &support, 0.10, &params);
    let _ = run_steps(
        &mut state,
        &phi,
        &support,
        &p,
        k_lateral_scale,
        REPAIR_STEPS,
        false,
    );
    let recovery = support_coverage(&state, &support, &params) / cov0;
    let (_, closed, _) = connected_closed_support_observer(&state, &support, &params);
    let no_field_reset = state.width == base.width
        && state.height == base.height
        && (state.activated - a_before_repair).abs() < 1e-9
        && phi.len() == state.width * state.height;

    let pass = only_normal && recovery + 1e-12 >= DAMAGE_RECOVERY_GATE && closed && no_field_reset;
    Gate4Report {
        depleted_m_l: led0.m_l,
        depleted_m_b: led0.m_b,
        depleted_m_mem: led0.m_mem,
        arms,
        only_normal_increases: only_normal,
        post_replenish_repair: recovery,
        post_repair_closed: closed,
        no_field_reset,
        pass,
        failure: if pass {
            None
        } else {
            Some("D081_MEMBRANE_REPLENISHMENT_NOT_ENERGY_CAUSAL".into())
        },
    }
}

// ─── Gate 5 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate5Report {
    pub a_retention: f64,
    pub c_retention: f64,
    pub l_bounded: bool,
    pub b_bounded: bool,
    pub b_closed: bool,
    pub a_fraction_consumed: f64,
    pub perm_n: f64,
    pub perm_f: f64,
    pub perm_w: f64,
    pub accounting_ok: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate5_metabolic_affordability(k_lateral_scale: f64) -> Gate5Report {
    let params = frozen_d079_params();
    let (mut state, phi, support) = assemble(22.0, &params, k_lateral_scale);
    let mut p = params;
    p.k_produce = 0.5;
    state.activated = 5.0;
    let a0 = state.activated;
    let c0 = state.catalyst;
    let m0 = state.total_membrane();
    let led = run_steps(
        &mut state,
        &phi,
        &support,
        &p,
        k_lateral_scale,
        AFFORD_STEPS,
        true,
    );
    let a1 = state.activated;
    let a_ret = if a0 > 1e-15 { a1 / a0 } else { 0.0 };
    let c_ret = if c0 > 1e-15 { state.catalyst / c0 } else { 0.0 };
    let consumed = (a0 - a1).max(0.0);
    let a_frac = if a0 > 1e-15 { consumed / a0 } else { 1.0 };
    let l_bounded = state.total_l().is_finite() && state.total_l() < 1e6;
    let b_bounded = state.total_b().is_finite() && state.total_b() < 1e6;
    let (_, closed, _) = connected_closed_support_observer(&state, &support, &params);
    let perm_n = mean_support_permeability(&state, &support, &params, "N");
    let perm_f = mean_support_permeability(&state, &support, &params, "F");
    let perm_w = mean_support_permeability(&state, &support, &params, "W");
    let expected_m = m0 + led.produce;
    let accounting_ok =
        (state.total_membrane() - expected_m).abs() < ACCOUNTING_TOL * 10.0 * (1.0 + expected_m);
    let nf_enter = perm_n >= STAGE_A_NF_PERM_LO - 1e-12 && perm_f >= STAGE_A_NF_PERM_LO - 1e-12;
    let w_exit = perm_w + 1e-12 >= STAGE_A_W_PERM_MIN;
    let pass = a_ret + 1e-12 >= A_RETENTION_GATE
        && c_ret + 1e-12 >= C_RETENTION_GATE
        && l_bounded
        && b_bounded
        && closed
        && a_frac <= 0.50 + 1e-12
        && nf_enter
        && w_exit
        && accounting_ok;
    Gate5Report {
        a_retention: a_ret,
        c_retention: c_ret,
        l_bounded,
        b_bounded,
        b_closed: closed,
        a_fraction_consumed: a_frac,
        perm_n,
        perm_f,
        perm_w,
        accounting_ok,
        pass,
        failure: if pass {
            None
        } else if a_ret < A_RETENTION_GATE && led.produce > 1e-6 {
            Some("D081_EDGE_MEMBRANE_PRODUCTION_METABOLICALLY_INFEASIBLE".into())
        } else {
            Some("D081_EDGE_MEMBRANE_PRODUCTION_METABOLICALLY_INFEASIBLE".into())
        },
    }
}

// ─── Gate 6 / 7 / review ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate6Report {
    pub seed_lawful: bool,
    pub reserve_repair_ok: bool,
    pub depletion_ok: bool,
    pub replenishment_ok: bool,
    pub affordability_ok: bool,
    pub d080_gate7_update: String,
    pub pass: bool,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub route: D081Route,
    pub conclusion: String,
    pub stopped_at_gate: String,
    pub scientific_conclusion: String,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub d080_gate7_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D081Review {
    pub gate0: Gate0Report,
    pub gate1: Gate1Report,
    pub gate2: Gate2Report,
    pub gate3: Gate3Report,
    pub gate4: Gate4Report,
    pub gate5: Gate5Report,
    pub gate6: Gate6Report,
    pub gate7_dynamic: Option<crate::d080_analysis::DynamicReport>,
    pub gate7_coupled: Option<crate::d080_analysis::CoupledReport>,
    pub route: RouteReport,
    pub k_lateral_scale: f64,
    pub scope_amendment: String,
    pub seed_contract: String,
}

fn skip_gate2() -> Gate2Report {
    Gate2Report {
        m_l_before: 0.0,
        m_b_before: 0.0,
        m_l_after_damage: 0.0,
        m_b_after_damage: 0.0,
        damaged: 0.0,
        m_l_after_repair: 0.0,
        m_b_after_repair: 0.0,
        rebound: 0.0,
        m_mem_conserved: false,
        recovery: 0.0,
        network_closed: false,
        transport_recovered: false,
        pass: false,
        failure: Some("skipped".into()),
    }
}
fn skip_gate3() -> Gate3Report {
    Gate3Report {
        starting_free_reserve: 0.0,
        original_bound: 0.0,
        damage_quantum: 0.0,
        events: vec![],
        cumulative_rebound: 0.0,
        eventual_failure: false,
        no_hidden_regen: false,
        l_monotone_aside_unbind: false,
        perm_rises_when_open: false,
        pass: false,
        failure: Some("skipped".into()),
    }
}
fn skip_gate4() -> Gate4Report {
    Gate4Report {
        depleted_m_l: 0.0,
        depleted_m_b: 0.0,
        depleted_m_mem: 0.0,
        arms: vec![],
        only_normal_increases: false,
        post_replenish_repair: 0.0,
        post_repair_closed: false,
        no_field_reset: false,
        pass: false,
        failure: Some("skipped".into()),
    }
}
fn skip_gate5() -> Gate5Report {
    Gate5Report {
        a_retention: 0.0,
        c_retention: 0.0,
        l_bounded: false,
        b_bounded: false,
        b_closed: false,
        a_fraction_consumed: 1.0,
        perm_n: 0.0,
        perm_f: 0.0,
        perm_w: 0.0,
        accounting_ok: false,
        pass: false,
        failure: Some("skipped".into()),
    }
}

fn finish(
    gate0: Gate0Report,
    gate1: Gate1Report,
    gate2: Gate2Report,
    gate3: Gate3Report,
    gate4: Gate4Report,
    gate5: Gate5Report,
    gate6: Gate6Report,
    gate7_dynamic: Option<crate::d080_analysis::DynamicReport>,
    gate7_coupled: Option<crate::d080_analysis::CoupledReport>,
    route: D081Route,
    stopped: &str,
    science: &str,
    next: &str,
    scale: f64,
    d080_gate7: &str,
) -> D081Review {
    let (d008, phase1, prod) = match route {
        D081Route::EdgeNetworkBoundaryQualified => (
            "STAGE_E_CANDIDATE_PENDING_CONTRACT",
            "PHASE1_EDGE_NETWORK_BOUNDARY_QUALIFIED",
            "EDGE_NETWORK_RESEARCH_PASS",
        ),
        D081Route::EdgeReserveCausalityQualified => (
            "BLOCKED_NOT_RECOVERED",
            "PHASE1_SELF_MAINTENANCE_PARTIAL",
            "REQUIRES_REMEDIATION",
        ),
        _ => (
            "BLOCKED_NOT_RECOVERED",
            "PHASE1_SELF_MAINTENANCE_PARTIAL",
            "REQUIRES_REMEDIATION",
        ),
    };
    D081Review {
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        gate6,
        gate7_dynamic,
        gate7_coupled,
        route: RouteReport {
            route,
            conclusion: route.conclusion().into(),
            stopped_at_gate: stopped.into(),
            scientific_conclusion: science.into(),
            next_directive: next.into(),
            next_execution_started: false,
            d008_status: d008.into(),
            phase1_status: phase1.into(),
            production_verdict: prod.into(),
            d080_gate7_status: d080_gate7.into(),
        },
        k_lateral_scale: scale,
        scope_amendment: SCOPE_AMENDMENT.into(),
        seed_contract: SEED_CONTRACT_V1.into(),
    }
}

pub fn run_full_review() -> D081Review {
    let gate0 = gate0_reproduce_d080();
    let scale = gate0.k_lateral_scale;
    if !gate0.pass {
        return finish(
            gate0,
            Gate1Report {
                contract: SEED_CONTRACT_V1.into(),
                rows: vec![],
                classification: SeedClassification::ProvenanceUnknown,
                density_consistent: false,
                pass: false,
                failure: Some("skipped".into()),
            },
            skip_gate2(),
            skip_gate3(),
            skip_gate4(),
            skip_gate5(),
            Gate6Report {
                seed_lawful: false,
                reserve_repair_ok: false,
                depletion_ok: false,
                replenishment_ok: false,
                affordability_ok: false,
                d080_gate7_update: D080_GATE7_PROVISIONAL.into(),
                pass: false,
                conclusion: D081Route::D080ResultNotReproduced.conclusion().into(),
            },
            None,
            None,
            D081Route::D080ResultNotReproduced,
            "gate0",
            "D-080 Gate0–7 fingerprint not reproduced.",
            "Repair D-080 reproduction before reserve audit.",
            scale,
            D080_GATE7_PROVISIONAL,
        );
    }

    let gate1 = gate1_seed_provenance();
    if !gate1.pass {
        return finish(
            gate0,
            gate1,
            skip_gate2(),
            skip_gate3(),
            skip_gate4(),
            skip_gate5(),
            Gate6Report {
                seed_lawful: false,
                reserve_repair_ok: false,
                depletion_ok: false,
                replenishment_ok: false,
                affordability_ok: false,
                d080_gate7_update: D080_GATE7_PROVISIONAL.into(),
                pass: false,
                conclusion: D081Route::EdgeMembraneSeedUnauthorized.conclusion().into(),
            },
            None,
            None,
            D081Route::EdgeMembraneSeedUnauthorized,
            "gate1",
            "Seed membrane material is unauthorized or inconsistent.",
            "Correct seed contract before causal interpretation.",
            scale,
            D080_GATE7_PROVISIONAL,
        );
    }

    let gate2 = gate2_reserve_only_repair(scale);
    if !gate2.pass {
        return finish(
            gate0,
            gate1,
            gate2,
            skip_gate3(),
            skip_gate4(),
            skip_gate5(),
            Gate6Report {
                seed_lawful: true,
                reserve_repair_ok: false,
                depletion_ok: false,
                replenishment_ok: false,
                affordability_ok: false,
                d080_gate7_update: D080_GATE7_PROVISIONAL.into(),
                pass: false,
                conclusion: D081Route::ReserveNotFiniteOrNotConserved.conclusion().into(),
            },
            None,
            None,
            D081Route::ReserveNotFiniteOrNotConserved,
            "gate2",
            "Reserve-only single repair failed conservation or recovery.",
            "Diagnose L/B ledger under frozen kinetics.",
            scale,
            D080_GATE7_PROVISIONAL,
        );
    }

    let gate3 = gate3_reserve_depletion(scale);
    if !gate3.pass {
        return finish(
            gate0,
            gate1,
            gate2,
            gate3,
            skip_gate4(),
            skip_gate5(),
            Gate6Report {
                seed_lawful: true,
                reserve_repair_ok: true,
                depletion_ok: false,
                replenishment_ok: false,
                affordability_ok: false,
                d080_gate7_update: D080_GATE7_PROVISIONAL.into(),
                pass: false,
                conclusion: D081Route::ReserveNotFiniteOrNotConserved.conclusion().into(),
            },
            None,
            None,
            D081Route::ReserveNotFiniteOrNotConserved,
            "gate3",
            "Reserve is not finite or not conserved under repeated damage.",
            "Find hidden regeneration or non-conserving assembly.",
            scale,
            D080_GATE7_PROVISIONAL,
        );
    }

    let gate4 = gate4_energy_causal_replenishment(scale);
    if !gate4.pass {
        return finish(
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            skip_gate5(),
            Gate6Report {
                seed_lawful: true,
                reserve_repair_ok: true,
                depletion_ok: true,
                replenishment_ok: false,
                affordability_ok: false,
                d080_gate7_update: D080_GATE7_PROVISIONAL.into(),
                pass: false,
                conclusion: D081Route::MembraneReplenishmentNotEnergyCausal
                    .conclusion()
                    .into(),
            },
            None,
            None,
            D081Route::MembraneReplenishmentNotEnergyCausal,
            "gate4",
            "Membrane replenishment is not energy-causal under A/production controls.",
            "Do not add A-for-binding; diagnose A→L production causality.",
            scale,
            D080_GATE7_PROVISIONAL,
        );
    }

    let gate5 = gate5_metabolic_affordability(scale);
    if !gate5.pass {
        return finish(
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            gate5,
            Gate6Report {
                seed_lawful: true,
                reserve_repair_ok: true,
                depletion_ok: true,
                replenishment_ok: true,
                affordability_ok: false,
                d080_gate7_update: D080_GATE7_PROVISIONAL.into(),
                pass: false,
                conclusion: D081Route::EdgeMembraneProductionMetabolicallyInfeasible
                    .conclusion()
                    .into(),
            },
            None,
            None,
            D081Route::EdgeMembraneProductionMetabolicallyInfeasible,
            "gate5",
            "A→L replenishment works but is metabolically unaffordable.",
            "Do not raise activation production; revise membrane yield/demand.",
            scale,
            D080_GATE7_PROVISIONAL,
        );
    }

    let gate6 = Gate6Report {
        seed_lawful: true,
        reserve_repair_ok: true,
        depletion_ok: true,
        replenishment_ok: true,
        affordability_ok: true,
        d080_gate7_update: "PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT".into(),
        pass: true,
        conclusion: D081Route::EdgeReserveCausalityQualified.conclusion().into(),
    };

    // Gate 7 — resume D-080 dynamic + coupled/structural.
    let dyn_r = gate8_dynamic_interface(scale);
    if !dyn_r.pass {
        return finish(
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            gate5,
            gate6,
            Some(dyn_r),
            None,
            D081Route::EdgeNetworkDynamicInterfaceFailure,
            "gate7_dynamic",
            "Resumed D-080 dynamic interface failed after reserve causality qualified.",
            "Conservative support migration repair under fixed kinetics.",
            scale,
            "PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT",
        );
    }

    let coupled = gate9_coupled_and_structural(scale);
    if !coupled.pass {
        let route = if coupled.structural_incompatible {
            D081Route::EdgeNetworkStructuralIncompatibility
        } else {
            D081Route::EdgeNetworkCoupledFailure
        };
        let science = if coupled.structural_incompatible {
            "Reserve causality qualified; frozen structural drive remains incompatible."
        } else {
            "Reserve causality qualified; coupled retention/coverage screen failed."
        };
        return finish(
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            gate5,
            gate6,
            Some(dyn_r),
            Some(coupled),
            route,
            "gate7_coupled",
            science,
            "Structural or coupled metabolic review under fixed support.",
            scale,
            "PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT",
        );
    }

    finish(
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        gate6,
        Some(dyn_r),
        Some(coupled),
        D081Route::EdgeNetworkBoundaryQualified,
        "none",
        "Reserve causality and resumed D-080 boundary gates passed.",
        "Formal D-008 edge-network boundary contract; Stages A–E revalidation.",
        scale,
        "PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT",
    )
}

#[cfg(test)]
mod unit_smoke {
    use super::*;

    #[test]
    fn seed_contract_id_stable() {
        assert!(seed_contract_identity(22.0, SEED_DENSITY).contains(SEED_CONTRACT_V1));
        assert_eq!(D081_STARTING_TAG, "D-080-edge-network-requalification-fail");
        assert_eq!(D080_STARTING_COMMIT, "99c0236");
        assert_eq!(D080_STARTING_TAG, "D-079-edge-network-boundary-fail");
        assert_eq!(D080_PRIMARY, "D080_EDGE_NETWORK_REPAIR_OR_CAUSALITY_FAILURE");
    }
}
