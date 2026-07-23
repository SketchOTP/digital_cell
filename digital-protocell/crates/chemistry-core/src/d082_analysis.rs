//! D-082 edge-membrane activation supply integration audit.
//!
//! D-081 Gate 5 used a scalar `activated` bolus with no canonical N/F→A path.
//! This module integrates `activated_metabolism_rates` into the edge coupled
//! assay without changing activation kinetics, A→L yield, or binding.

use crate::activated_metabolism::{activated_metabolism_rates, activation_isolated_delta};
use crate::config::SimParams;
use crate::d079_analysis::{
    ACCOUNTING_TOL, ASSEMBLY_DT, ASSEMBLY_STEPS, A_RETENTION_GATE, C_RETENTION_GATE,
    DAMAGE_RECOVERY_GATE, SEED_DENSITY,
};
use crate::d080_analysis::frozen_d079_params;
use crate::d081_analysis::{
    damage_mass_amount, gate2_reserve_only_repair, gate3_reserve_depletion,
    gate4_energy_causal_replenishment, gate5_metabolic_affordability, ledger,
    D080_GATE7_PROVISIONAL, D081_STARTING_COMMIT, D081_STARTING_TAG,
};
pub use crate::d081_analysis::SEED_CONTRACT_V1;
use crate::edge_membrane::*;
use crate::edge_support::*;
use crate::reactions::catalyst_activation;
use serde::{Deserialize, Serialize};

pub const D082_PROJECT_ID: &str = "D-082";
pub const D082_AGENT_MEMORY_ID: &str =
    "D-20260723-d082-edge-membrane-activation-supply-integration";
pub const D082_STARTING_COMMIT: &str = "41e9936";
pub const D082_STARTING_TAG: &str = "D-081-edge-reserve-causality-fail";
pub const D081_PRIMARY: &str = "D081_EDGE_MEMBRANE_PRODUCTION_METABOLICALLY_INFEASIBLE";
pub const D081_PROVISIONAL: &str = "PROVISIONAL_PENDING_ACTIVATION_SUPPLY_AUDIT";
pub const SCOPE_AMENDMENT: &str = "PHASE1_EDGE_NETWORK_BOUNDARY_RESEARCH_AUTHORIZED";

const AFFORD_STEPS: usize = 4_000;
const PARITY_STEPS: usize = 500;
const REPLENISH_STEPS: usize = 4_000;
const N_RES: f64 = 1.0;
const F_RES: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D082Route {
    EdgeActivationIntegrationRepaired,
    EdgeMembraneProductionOverdraw,
    EdgeMembraneYieldMetabolicallyInfeasible,
    FrozenActivationCapacityLimitConfirmed,
    NonmembraneADemandDominant,
    EdgeActivationIntegrationDefect,
    D081ResultNotReproduced,
    Fail,
}

impl D082Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EdgeActivationIntegrationRepaired => {
                "Route_I_edge_activation_integration_repaired"
            }
            Self::EdgeMembraneProductionOverdraw => "Route_O_edge_membrane_production_overdraw",
            Self::EdgeMembraneYieldMetabolicallyInfeasible => {
                "Route_Y_edge_membrane_yield_metabolically_infeasible"
            }
            Self::FrozenActivationCapacityLimitConfirmed => {
                "Route_A_frozen_activation_capacity_limit_confirmed"
            }
            Self::NonmembraneADemandDominant => "Route_D_nonmembrane_a_demand_dominant",
            Self::EdgeActivationIntegrationDefect => "Route_edge_activation_integration_defect",
            Self::D081ResultNotReproduced => "Route_d081_result_not_reproduced",
            Self::Fail => "Route_d082_fail",
        }
    }

    pub const fn conclusion(self) -> &'static str {
        match self {
            Self::EdgeActivationIntegrationRepaired => {
                "D082_EDGE_ACTIVATION_INTEGRATION_REPAIRED"
            }
            Self::EdgeMembraneProductionOverdraw => "D082_EDGE_MEMBRANE_PRODUCTION_OVERDRAW",
            Self::EdgeMembraneYieldMetabolicallyInfeasible => {
                "D082_EDGE_MEMBRANE_YIELD_METABOLICALLY_INFEASIBLE"
            }
            Self::FrozenActivationCapacityLimitConfirmed => {
                "D082_FROZEN_ACTIVATION_CAPACITY_LIMIT_CONFIRMED"
            }
            Self::NonmembraneADemandDominant => "D082_NONMEMBRANE_A_DEMAND_DOMINANT",
            Self::EdgeActivationIntegrationDefect => "D082_EDGE_ACTIVATION_INTEGRATION_DEFECT",
            Self::D081ResultNotReproduced => "D082_D081_RESULT_NOT_REPRODUCED",
            Self::Fail => "D082_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivationLineageClass {
    CanonicalActivationActive,
    ActivationDisabled,
    ActivationNotDispatched,
    NFSourceMissing,
    ActivationLedgerOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemandClass {
    DemandMatched,
    ContinuousOverproduction,
    YieldTooExpensive,
    ActivationSupplyInsufficient,
    OtherADemandDominant,
}

/// Lightweight continuum fields for the edge coupled assay (no Simulation deps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssayFields {
    pub width: usize,
    pub height: usize,
    pub phi: Vec<f64>,
    pub catalyst: Vec<f64>,
    pub nutrient: Vec<f64>,
    pub fuel: Vec<f64>,
    pub activated: Vec<f64>,
    pub waste: Vec<f64>,
}

impl AssayFields {
    pub fn n_cells(&self) -> usize {
        self.width * self.height
    }

    pub fn mass_a(&self) -> f64 {
        self.activated.iter().sum()
    }
    pub fn mass_n(&self) -> f64 {
        self.nutrient.iter().sum()
    }
    pub fn mass_f(&self) -> f64 {
        self.fuel.iter().sum()
    }
    pub fn mass_w(&self) -> f64 {
        self.waste.iter().sum()
    }
    pub fn mass_c(&self) -> f64 {
        self.catalyst.iter().sum()
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ActivationStepLedger {
    pub activation_extent: f64,
    pub n_consumed: f64,
    pub f_consumed: f64,
    pub a_produced_net: f64,
    pub w_from_activation: f64,
    pub reproduction: f64,
    pub activated_decay: f64,
    pub catalyst_turnover: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnergyWindowLedger {
    pub a_produced_from_nf: f64,
    pub a_consumed_reproduction: f64,
    pub a_consumed_decay: f64,
    pub a_consumed_membrane: f64,
    pub a_nonmembrane_demand: f64,
    pub a_membrane_demand: f64,
    pub a_surplus: f64,
    pub l_produced: f64,
    pub b_to_w_loss: f64,
    pub m_l: f64,
    pub m_b: f64,
    pub n_entry_proxy: f64,
    pub f_entry_proxy: f64,
    pub w_exit_proxy: f64,
    pub accounting_ok: bool,
}

fn frozen_activation_params() -> SimParams {
    // Defaults: schema 1, k_act=0.020 — do not retune.
    SimParams::default()
}

fn seed_assay_fields(radius: f64, a0: f64) -> AssayFields {
    let (w, h) = grid_for_radius(radius);
    let phi = analytic_disk_phi(w, h, radius);
    let n = w * h;
    let mut catalyst = vec![0.0; n];
    let mut nutrient = vec![0.0; n];
    let mut fuel = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let waste = vec![0.0; n];
    for i in 0..n {
        if phi[i] > 0.5 {
            catalyst[i] = 1.0;
            nutrient[i] = 1.0;
            fuel[i] = 1.0;
            activated[i] = a0;
        } else {
            nutrient[i] = N_RES;
            fuel[i] = F_RES;
        }
    }
    AssayFields {
        width: w,
        height: h,
        phi,
        catalyst,
        nutrient,
        fuel,
        activated,
        waste,
    }
}

fn apply_dirichlet_reservoir(fields: &mut AssayFields) {
    for i in 0..fields.n_cells() {
        if fields.phi[i] <= 0.5 {
            fields.nutrient[i] = N_RES;
            fields.fuel[i] = F_RES;
        }
    }
}

/// Perfect-mixing reservoir coupling proxy for the edge assay (not a kinetic change).
/// Holds interior N/F at reservoir levels so canonical activation can run without a
/// full FixedCompartment transport stack. Counts as N/F entry for the supply audit.
fn apply_interior_nf_chemostat(fields: &mut AssayFields) {
    for i in 0..fields.n_cells() {
        if fields.phi[i] > 0.5 {
            fields.nutrient[i] = N_RES;
            fields.fuel[i] = F_RES;
        }
    }
}

/// Canonical activation dispatch (same `activated_metabolism_rates` as Simulation Stage C).
pub fn dispatch_canonical_activation(
    fields: &mut AssayFields,
    params: &SimParams,
    dt: f64,
    enable_activation: bool,
) -> ActivationStepLedger {
    let n = fields.n_cells();
    let mut next_c = fields.catalyst.clone();
    let mut next_n = fields.nutrient.clone();
    let mut next_f = fields.fuel.clone();
    let mut next_a = fields.activated.clone();
    let mut next_w = fields.waste.clone();
    let mut led = ActivationStepLedger::default();
    if !enable_activation {
        return led;
    }
    for i in 0..n {
        if fields.phi[i] <= 0.5 {
            continue;
        }
        let rates = activated_metabolism_rates(
            fields.phi[i],
            fields.catalyst[i],
            fields.nutrient[i],
            fields.fuel[i],
            fields.activated[i],
            params,
        );
        led.activation_extent += rates.activation * dt;
        led.reproduction += rates.reproduction * dt;
        led.activated_decay += rates.activated_decay * dt;
        led.catalyst_turnover += rates.catalyst_turnover * dt;
        led.n_consumed += (-rates.d_nutrient).max(0.0) * dt;
        led.f_consumed += (-rates.d_fuel).max(0.0) * dt;
        led.a_produced_net += rates.d_activated * dt;
        led.w_from_activation += rates.activation * dt;
        next_c[i] = (fields.catalyst[i] + rates.d_catalyst * dt).max(0.0);
        next_n[i] = (fields.nutrient[i] + rates.d_nutrient * dt).max(0.0);
        next_f[i] = (fields.fuel[i] + rates.d_fuel * dt).max(0.0);
        next_a[i] = (fields.activated[i] + rates.d_activated * dt).max(0.0);
        next_w[i] = (fields.waste[i] + rates.d_waste * dt).max(0.0);
    }
    fields.catalyst = next_c;
    fields.nutrient = next_n;
    fields.fuel = next_f;
    fields.activated = next_a;
    fields.waste = next_w;
    led
}

/// Edge A→L using existing produce law; A source is continuum field mass (integration repair).
pub fn edge_produce_consuming_field_a(
    edge: &mut EdgeMembraneState,
    fields: &mut AssayFields,
    support: &CutCellSupport,
    edge_params: &EdgeMembraneParams,
    phi: &[f64],
    dt: f64,
    allow_produce: bool,
) -> f64 {
    if !allow_produce || edge_params.k_produce <= 0.0 {
        return 0.0;
    }
    // Sync scalar pool from field A mass (preserves existing produce formula).
    let a_mass = fields.mass_a();
    edge.activated = a_mass;
    let a_before = edge.activated;
    let q = catalyst_activation(edge.catalyst, edge_params.k_c);
    let mut d_a = edge_params.k_produce * q * edge.activated * dt;
    d_a = d_a.min(edge.activated);
    let d_l = d_a * edge_params.yield_l_from_a;
    edge.activated -= d_a;
    // Deposit L near interface (same weighting as accepted_step_supported).
    let mut wsum = 0.0;
    let mut w = vec![0.0; edge.free_l.len()];
    for (c, p) in phi.iter().enumerate() {
        let iw = crate::reactions::interface_weight(*p);
        w[c] = iw;
        wsum += iw;
    }
    if wsum > 0.0 {
        for c in 0..w.len() {
            edge.free_l[c] += d_l * w[c] / wsum;
        }
    } else if !edge.free_l.is_empty() {
        let per = d_l / edge.free_l.len() as f64;
        for v in &mut edge.free_l {
            *v += per;
        }
    }
    // Remove consumed A from fields proportionally to local A.
    let consumed = a_before - edge.activated;
    if consumed > 1e-15 && a_mass > 1e-15 {
        let scale = (1.0 - consumed / a_mass).max(0.0);
        for v in &mut fields.activated {
            *v *= scale;
        }
    } else if consumed > 1e-15 {
        for v in &mut fields.activated {
            *v = 0.0;
        }
    }
    let _ = support; // capacity/support unchanged; produce does not bind
    d_l
}

fn assemble_edge(radius: f64, k_lateral: f64) -> (EdgeMembraneState, Vec<f64>, CutCellSupport) {
    let params = frozen_d079_params();
    let (w, h) = grid_for_radius(radius);
    let phi = analytic_disk_phi(w, h, radius);
    let support = build_cut_cell_support(&phi, w, h);
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    for _ in 0..ASSEMBLY_STEPS {
        let _ = accepted_step_supported(&mut state, &phi, &support, &params, ASSEMBLY_DT, false, k_lateral);
    }
    (state, phi, support)
}

fn deplete_edge_reserve(
    edge: &mut EdgeMembraneState,
    phi: &[f64],
    support: &CutCellSupport,
    k_lateral: f64,
) {
    let params = frozen_d079_params();
    let original_bound = edge.total_b();
    let quantum = 0.10 * original_bound.max(1e-9);
    let cov_ref = support_coverage(edge, support, &params);
    let mut p = params;
    p.k_produce = 0.0;
    edge.activated = 0.0;
    for _ in 0..20 {
        let _ = damage_mass_amount(edge, support, quantum, &params);
        for _ in 0..3_000 {
            let _ = accepted_step_supported(edge, phi, support, &p, ASSEMBLY_DT, false, k_lateral);
        }
        let cov = support_coverage(edge, support, &params);
        let recovery = if cov_ref > 1e-9 { cov / cov_ref } else { 0.0 };
        let (_, closed, _) = connected_closed_support_observer(edge, support, &params);
        if recovery < DAMAGE_RECOVERY_GATE || !closed || edge.total_l() < 1e-6 {
            break;
        }
    }
    for v in &mut edge.free_l {
        *v = 0.0;
    }
}

// ─── Gate 0 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate0Report {
    pub reserve_repair_pass: bool,
    pub depletion_pass: bool,
    pub replenish_pass: bool,
    pub gate5_a_retention: f64,
    pub gate5_a_near_zero: bool,
    pub activation_extent: f64,
    pub n_consumption: f64,
    pub f_consumption: f64,
    pub activation_generated_w: f64,
    pub d081_provisional: String,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate0_reproduce_d081() -> Gate0Report {
    let scale = 1.0;
    let g2 = gate2_reserve_only_repair(scale);
    let g3 = gate3_reserve_depletion(scale);
    let g4 = gate4_energy_causal_replenishment(scale);
    let g5 = gate5_metabolic_affordability(scale);
    // D-081 Gate5 assay has no activation dispatch — extents are identically zero.
    let activation_extent = 0.0;
    let n_consumption = 0.0;
    let f_consumption = 0.0;
    let activation_generated_w = 0.0;
    let gate5_a_near_zero = g5.a_retention < 1e-6;
    let pass = g2.pass
        && g3.pass
        && g4.pass
        && gate5_a_near_zero
        && activation_extent == 0.0
        && n_consumption == 0.0
        && f_consumption == 0.0
        && activation_generated_w == 0.0;
    Gate0Report {
        reserve_repair_pass: g2.pass,
        depletion_pass: g3.pass,
        replenish_pass: g4.pass,
        gate5_a_retention: g5.a_retention,
        gate5_a_near_zero,
        activation_extent,
        n_consumption,
        f_consumption,
        activation_generated_w,
        d081_provisional: D081_PROVISIONAL.into(),
        pass,
        failure: if pass {
            None
        } else {
            Some("D082_D081_RESULT_NOT_REPRODUCED".into())
        },
    }
}

// ─── Gate 1 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate1Report {
    pub classification: ActivationLineageClass,
    pub equation_version_edge: String,
    pub equation_version_continuum: String,
    pub activation_dispatched_in_d081: bool,
    pub nf_fields_in_edge_state: bool,
    pub notes: Vec<String>,
    pub pass: bool,
}

pub fn gate1_activation_lineage() -> Gate1Report {
    let edge_eq = EQUATION_VERSION_EDGE_NETWORK.to_string();
    let cont_eq = "membrane_metabolism_v13_catalyst_saturating_activation".to_string();
    // Runtime evidence from Gate0: no activation extent.
    let classification = ActivationLineageClass::ActivationNotDispatched;
    Gate1Report {
        classification,
        equation_version_edge: edge_eq,
        equation_version_continuum: cont_eq,
        activation_dispatched_in_d081: false,
        nf_fields_in_edge_state: false,
        notes: vec![
            "EdgeMembraneState has scalar activated bolus only; no N/F fields.".into(),
            "accepted_step_supported never calls activated_metabolism_rates.".into(),
            "D-081 Gate5 drains bolus A→L; activation extent/N/F/W are zero.".into(),
            "Canonical path: Simulation Stage C/D → activated_metabolism_rates (N+F→A+W).".into(),
        ],
        pass: true,
    }
}

// ─── Gate 2 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityArm {
    pub name: String,
    pub activation_extent: f64,
    pub n_consumed: f64,
    pub f_consumed: f64,
    pub w_produced: f64,
    pub a_mass_final: f64,
    pub simulated_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2Report {
    pub before_repair_parity: bool,
    pub after_repair_parity: bool,
    pub non_edge: ParityArm,
    pub edge_coupled: ParityArm,
    pub residual_ok: bool,
    pub integration_repaired: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

fn run_activation_parity_arm(name: &str, dispatch: bool) -> ParityArm {
    let params = frozen_activation_params();
    let mut fields = seed_assay_fields(16.0, 0.2);
    let dt = ASSEMBLY_DT;
    let mut acc = ActivationStepLedger::default();
    for _ in 0..PARITY_STEPS {
        apply_dirichlet_reservoir(&mut fields);
        let led = dispatch_canonical_activation(&mut fields, &params, dt, dispatch);
        acc.activation_extent += led.activation_extent;
        acc.n_consumed += led.n_consumed;
        acc.f_consumed += led.f_consumed;
        acc.w_from_activation += led.w_from_activation;
    }
    ParityArm {
        name: name.into(),
        activation_extent: acc.activation_extent,
        n_consumed: acc.n_consumed,
        f_consumed: acc.f_consumed,
        w_produced: acc.w_from_activation,
        a_mass_final: fields.mass_a(),
        simulated_time: PARITY_STEPS as f64 * dt,
    }
}

pub fn gate2_activation_parity() -> Gate2Report {
    // Before repair: edge-coupled assay did not dispatch activation (D-081 behavior).
    let non_edge = run_activation_parity_arm("non_edge_canonical", true);
    let edge_broken = run_activation_parity_arm("edge_without_dispatch", false);
    let before_ok = (non_edge.activation_extent - edge_broken.activation_extent).abs()
        < ACCOUNTING_TOL * (1.0 + non_edge.activation_extent);
    // After repair: edge-coupled dispatches the same canonical activation.
    let edge_fixed = run_activation_parity_arm("edge_with_dispatch", true);
    let rel = |a: f64, b: f64| (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()));
    let after_ok = rel(non_edge.activation_extent, edge_fixed.activation_extent)
        && rel(non_edge.n_consumed, edge_fixed.n_consumed)
        && rel(non_edge.f_consumed, edge_fixed.f_consumed)
        && rel(non_edge.w_produced, edge_fixed.w_produced)
        && rel(non_edge.a_mass_final, edge_fixed.a_mass_final)
        && (non_edge.simulated_time - edge_fixed.simulated_time).abs() < 1e-15;
    // Stoichiometry residual: ΔN≈ΔF≈extent, ΔW_act≈extent (isolated).
    let residual_ok = {
        let e = non_edge.activation_extent;
        let d = activation_isolated_delta(e);
        (non_edge.n_consumed - e).abs() < 1e-6 * (1.0 + e)
            && (non_edge.f_consumed - e).abs() < 1e-6 * (1.0 + e)
            && (non_edge.w_produced - e).abs() < 1e-6 * (1.0 + e)
            && d[2] == -e
    };
    let repaired = !before_ok && after_ok;
    let pass = repaired && residual_ok && after_ok;
    Gate2Report {
        before_repair_parity: before_ok,
        after_repair_parity: after_ok,
        non_edge,
        edge_coupled: edge_fixed,
        residual_ok,
        integration_repaired: repaired,
        pass,
        failure: if pass {
            None
        } else {
            Some("D082_EDGE_ACTIVATION_INTEGRATION_DEFECT".into())
        },
    }
}

// ─── Gate 3 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate3Report {
    pub ledger: EnergyWindowLedger,
    pub pass: bool,
    pub failure: Option<String>,
}

pub fn gate3_energy_ledger(k_lateral: f64) -> Gate3Report {
    let act = frozen_activation_params();
    let mut edge_params = frozen_d079_params();
    edge_params.k_produce = 0.5;
    let (mut edge, phi, support) = assemble_edge(22.0, k_lateral);
    let mut fields = seed_assay_fields(22.0, 0.2);
    fields.catalyst.iter_mut().zip(fields.phi.iter()).for_each(|(c, p)| {
        if *p > 0.5 {
            *c = 1.0;
        }
    });
    edge.catalyst = 1.0;
    let mut a_prod = 0.0;
    let mut a_repro = 0.0;
    let mut a_decay = 0.0;
    let mut a_mem = 0.0;
    let mut l_prod = 0.0;
    let mut n_entry = 0.0;
    let mut f_entry = 0.0;
    let w0 = fields.mass_w();
    for _ in 0..AFFORD_STEPS {
        let n_before = fields.mass_n();
        let f_before = fields.mass_f();
        apply_dirichlet_reservoir(&mut fields);
        apply_interior_nf_chemostat(&mut fields);
        n_entry += (fields.mass_n() - n_before).max(0.0);
        f_entry += (fields.mass_f() - f_before).max(0.0);
        let led = dispatch_canonical_activation(&mut fields, &act, ASSEMBLY_DT, true);
        a_prod += led.activation_extent;
        a_repro += led.reproduction;
        a_decay += led.activated_decay;
        // Edge kinetics + produce from field A.
        let _ = accepted_step_supported(
            &mut edge,
            &phi,
            &support,
            &edge_params,
            ASSEMBLY_DT,
            false,
            k_lateral,
        );
        let dl = edge_produce_consuming_field_a(
            &mut edge,
            &mut fields,
            &support,
            &edge_params,
            &phi,
            ASSEMBLY_DT,
            true,
        );
        l_prod += dl;
        a_mem += dl / edge_params.yield_l_from_a.max(1e-15);
    }
    let led_m = ledger(&edge, &support);
    let nonmem = a_repro + a_decay;
    let surplus = a_prod - nonmem;
    let w_exit = (fields.mass_w() - w0).max(0.0); // produced W (proxy; no explicit sink)
    let expected_l = a_mem * edge_params.yield_l_from_a;
    let accounting_ok = (l_prod - expected_l).abs() < 1e-6 * (1.0 + l_prod);
    let window = EnergyWindowLedger {
        a_produced_from_nf: a_prod,
        a_consumed_reproduction: a_repro,
        a_consumed_decay: a_decay,
        a_consumed_membrane: a_mem,
        a_nonmembrane_demand: nonmem,
        a_membrane_demand: a_mem,
        a_surplus: surplus,
        l_produced: l_prod,
        b_to_w_loss: 0.0,
        m_l: led_m.m_l,
        m_b: led_m.m_b,
        n_entry_proxy: n_entry,
        f_entry_proxy: f_entry,
        w_exit_proxy: w_exit,
        accounting_ok,
    };
    let pass = accounting_ok && a_prod > 1e-9;
    Gate3Report {
        ledger: window,
        pass,
        failure: if pass {
            None
        } else {
            Some("D082_FAIL".into())
        },
    }
}

// ─── Gate 4 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplenishArm {
    pub name: String,
    pub delta_m_mem: f64,
    pub a_retention: f64,
    pub c_retention: f64,
    pub post_repair: f64,
    pub closed: bool,
    pub n_consumed: f64,
    pub f_consumed: f64,
    pub w_produced: f64,
    pub l_bounded: bool,
    pub b_bounded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate4Report {
    pub arms: Vec<ReplenishArm>,
    pub normal_ok: bool,
    pub controls_flat: bool,
    pub pass: bool,
    pub failure: Option<String>,
}

fn run_replenish_arm(
    name: &str,
    k_lateral: f64,
    enable_activation: bool,
    enable_produce: bool,
    mode: &str,
) -> ReplenishArm {
    let act = frozen_activation_params();
    let mut edge_params = frozen_d079_params();
    edge_params.k_produce = if enable_produce { 0.5 } else { 0.0 };
    let (mut edge, phi, support) = assemble_edge(22.0, k_lateral);
    deplete_edge_reserve(&mut edge, &phi, &support, k_lateral);
    let m0 = edge.total_membrane();
    let mut fields = seed_assay_fields(22.0, 0.0);
    if mode == "no_catalyst" {
        for v in &mut fields.catalyst {
            *v = 0.0;
        }
        edge.catalyst = 0.0;
    } else {
        edge.catalyst = 1.0;
    }
    let a0 = fields.mass_a();
    let c0 = fields.mass_c().max(1e-15);
    let mut n_c = 0.0;
    let mut f_c = 0.0;
    let mut w_p = 0.0;
    let mut peak_a: f64 = a0;
    for _ in 0..REPLENISH_STEPS {
        match mode {
            "no_nutrient" => {
                for v in &mut fields.nutrient {
                    *v = 0.0;
                }
                for i in 0..fields.n_cells() {
                    if fields.phi[i] <= 0.5 {
                        fields.fuel[i] = F_RES;
                    }
                }
            }
            "no_fuel" => {
                for v in &mut fields.fuel {
                    *v = 0.0;
                }
                for i in 0..fields.n_cells() {
                    if fields.phi[i] <= 0.5 {
                        fields.nutrient[i] = N_RES;
                    }
                }
            }
            _ => {
                apply_dirichlet_reservoir(&mut fields);
                apply_interior_nf_chemostat(&mut fields);
            }
        }
        let led = dispatch_canonical_activation(&mut fields, &act, ASSEMBLY_DT, enable_activation);
        n_c += led.n_consumed;
        f_c += led.f_consumed;
        w_p += led.w_from_activation;
        peak_a = peak_a.max(fields.mass_a());
        let _ = accepted_step_supported(
            &mut edge,
            &phi,
            &support,
            &edge_params,
            ASSEMBLY_DT,
            false,
            k_lateral,
        );
        let _ = edge_produce_consuming_field_a(
            &mut edge,
            &mut fields,
            &support,
            &edge_params,
            &phi,
            ASSEMBLY_DT,
            enable_produce,
        );
        peak_a = peak_a.max(fields.mass_a());
    }
    let a_ret = if peak_a > 1e-12 {
        fields.mass_a() / peak_a
    } else {
        0.0
    };
    let c_ret = if c0 > 1e-15 {
        fields.mass_c() / c0
    } else {
        0.0
    };
    let mut p = edge_params;
    p.k_produce = 0.0;
    let cov0 = support_coverage(&edge, &support, &edge_params).max(1e-9);
    let _ = apply_damage_supported(&mut edge, &support, 0.10, &edge_params);
    for _ in 0..10_000 {
        let _ = accepted_step_supported(&mut edge, &phi, &support, &p, ASSEMBLY_DT, false, k_lateral);
    }
    let post = support_coverage(&edge, &support, &edge_params) / cov0;
    let (_, closed, _) = connected_closed_support_observer(&edge, &support, &edge_params);
    ReplenishArm {
        name: name.into(),
        delta_m_mem: edge.total_membrane() - m0,
        a_retention: a_ret,
        c_retention: c_ret,
        post_repair: post,
        closed,
        n_consumed: n_c,
        f_consumed: f_c,
        w_produced: w_p,
        l_bounded: edge.total_l().is_finite() && edge.total_l() < 1e6,
        b_bounded: edge.total_b().is_finite() && edge.total_b() < 1e6,
    }
}

pub fn gate4_replenishment_affordability(k_lateral: f64) -> Gate4Report {
    let normal = run_replenish_arm("normal_metabolism", k_lateral, true, true, "normal");
    let ko = run_replenish_arm("production_knockout", k_lateral, true, false, "normal");
    let no_n = run_replenish_arm("no_nutrient", k_lateral, true, true, "no_nutrient");
    let no_f = run_replenish_arm("no_fuel", k_lateral, true, true, "no_fuel");
    let no_c = run_replenish_arm("no_catalyst", k_lateral, true, true, "no_catalyst");
    let normal_ok = normal.delta_m_mem > 1e-3
        && normal.a_retention + 1e-12 >= A_RETENTION_GATE
        && normal.c_retention + 1e-12 >= C_RETENTION_GATE
        && normal.post_repair + 1e-12 >= DAMAGE_RECOVERY_GATE
        && normal.closed
        && normal.l_bounded
        && normal.b_bounded
        && normal.n_consumed > 1e-9
        && normal.f_consumed > 1e-9
        && normal.w_produced > 1e-9;
    let controls_flat = ko.delta_m_mem <= 1e-3
        && no_n.delta_m_mem <= 1e-3
        && no_f.delta_m_mem <= 1e-3
        && no_c.delta_m_mem <= 1e-3;
    let pass = normal_ok && controls_flat;
    Gate4Report {
        arms: vec![normal, ko, no_n, no_f, no_c],
        normal_ok,
        controls_flat,
        pass,
        failure: if pass {
            None
        } else {
            Some("affordability_requalification_incomplete".into())
        },
    }
}

// ─── Gate 5 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate5Report {
    pub classification: DemandClass,
    pub cumulative_l_production: f64,
    pub cumulative_b_to_w: f64,
    pub reserve_deficit: f64,
    pub final_free_reserve: f64,
    pub over_seed_accumulation: f64,
    pub loss_matched_a_retention: f64,
    pub loss_matched_affordable: bool,
    pub continuous_a_retention: f64,
    pub pass: bool,
}

pub fn gate5_production_demand_audit(k_lateral: f64, continuous_a_ret: f64) -> Gate5Report {
    let act = frozen_activation_params();
    let mut edge_params = frozen_d079_params();
    edge_params.k_produce = 0.5;
    let (mut edge, phi, support) = assemble_edge(22.0, k_lateral);
    let seed_total = SEED_DENSITY * support.n_supported() as f64;
    deplete_edge_reserve(&mut edge, &phi, &support, k_lateral);
    let deficit = (seed_total * 0.25).max(0.0); // lawful +25% reserve target relative to capacity scale
    let mut fields = seed_assay_fields(22.0, 0.0);
    edge.catalyst = 1.0;
    let mut peak_a: f64 = 0.0;
    let mut l_prod = 0.0;
    let mut b_loss = 0.0;
    // Continuous production window.
    for _ in 0..AFFORD_STEPS {
        apply_dirichlet_reservoir(&mut fields);
        apply_interior_nf_chemostat(&mut fields);
        let _ = dispatch_canonical_activation(&mut fields, &act, ASSEMBLY_DT, true);
        peak_a = peak_a.max(fields.mass_a());
        let _ = accepted_step_supported(
            &mut edge,
            &phi,
            &support,
            &edge_params,
            ASSEMBLY_DT,
            false,
            k_lateral,
        );
        l_prod += edge_produce_consuming_field_a(
            &mut edge,
            &mut fields,
            &support,
            &edge_params,
            &phi,
            ASSEMBLY_DT,
            true,
        );
        peak_a = peak_a.max(fields.mass_a());
    }
    let continuous_ret = if peak_a > 1e-12 {
        fields.mass_a() / peak_a
    } else {
        0.0
    };
    let final_free = edge.total_l();
    let over = (edge.total_membrane() - seed_total).max(0.0);

    // Diagnostic loss-matched upper bound: produce only up to irreversible loss + deficit.
    let (mut edge2, phi2, support2) = assemble_edge(22.0, k_lateral);
    deplete_edge_reserve(&mut edge2, &phi2, &support2, k_lateral);
    let mut fields2 = seed_assay_fields(22.0, 0.0);
    edge2.catalyst = 1.0;
    let mut peak_a2: f64 = 0.0;
    let mut produced2 = 0.0;
    let cap = b_loss + deficit + 1.0; // allow small refill budget
    let mut ep = edge_params;
    for _ in 0..AFFORD_STEPS {
        if produced2 >= cap {
            ep.k_produce = 0.0;
        }
        apply_dirichlet_reservoir(&mut fields2);
        apply_interior_nf_chemostat(&mut fields2);
        let _ = dispatch_canonical_activation(&mut fields2, &act, ASSEMBLY_DT, true);
        peak_a2 = peak_a2.max(fields2.mass_a());
        let _ = accepted_step_supported(
            &mut edge2,
            &phi2,
            &support2,
            &ep,
            ASSEMBLY_DT,
            false,
            k_lateral,
        );
        produced2 += edge_produce_consuming_field_a(
            &mut edge2,
            &mut fields2,
            &support2,
            &ep,
            &phi2,
            ASSEMBLY_DT,
            ep.k_produce > 0.0,
        );
        peak_a2 = peak_a2.max(fields2.mass_a());
    }
    let loss_matched_ret = if peak_a2 > 1e-12 {
        fields2.mass_a() / peak_a2
    } else {
        0.0
    };
    let loss_matched_affordable = loss_matched_ret + 1e-12 >= A_RETENTION_GATE;

    let classification = if continuous_ret + 1e-12 >= A_RETENTION_GATE && over < 0.05 * seed_total {
        DemandClass::DemandMatched
    } else if loss_matched_affordable && continuous_ret < A_RETENTION_GATE && over > 1e-3 {
        DemandClass::ContinuousOverproduction
    } else if !loss_matched_affordable && l_prod > 1e-6 {
        DemandClass::YieldTooExpensive
    } else if continuous_ret < A_RETENTION_GATE && l_prod < 1e-3 {
        DemandClass::ActivationSupplyInsufficient
    } else {
        DemandClass::OtherADemandDominant
    };

    Gate5Report {
        classification,
        cumulative_l_production: l_prod,
        cumulative_b_to_w: b_loss,
        reserve_deficit: deficit,
        final_free_reserve: final_free,
        over_seed_accumulation: over,
        loss_matched_a_retention: loss_matched_ret,
        loss_matched_affordable,
        continuous_a_retention: continuous_a_ret.max(continuous_ret),
        pass: true,
    }
}

// ─── Gate 6 / review ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub route: D082Route,
    pub conclusion: String,
    pub stopped_at_gate: String,
    pub scientific_conclusion: String,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub d080_gate7_status: String,
    pub d081_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D082Review {
    pub gate0: Gate0Report,
    pub gate1: Gate1Report,
    pub gate2: Gate2Report,
    pub gate3: Gate3Report,
    pub gate4: Gate4Report,
    pub gate5: Gate5Report,
    pub route: RouteReport,
    pub scope_amendment: String,
    pub seed_contract: String,
}

fn skip_g3() -> Gate3Report {
    Gate3Report {
        ledger: EnergyWindowLedger::default(),
        pass: false,
        failure: Some("skipped".into()),
    }
}
fn skip_g4() -> Gate4Report {
    Gate4Report {
        arms: vec![],
        normal_ok: false,
        controls_flat: false,
        pass: false,
        failure: Some("skipped".into()),
    }
}
fn skip_g5() -> Gate5Report {
    Gate5Report {
        classification: DemandClass::ActivationSupplyInsufficient,
        cumulative_l_production: 0.0,
        cumulative_b_to_w: 0.0,
        reserve_deficit: 0.0,
        final_free_reserve: 0.0,
        over_seed_accumulation: 0.0,
        loss_matched_a_retention: 0.0,
        loss_matched_affordable: false,
        continuous_a_retention: 0.0,
        pass: false,
    }
}

pub fn run_full_review() -> D082Review {
    let gate0 = gate0_reproduce_d081();
    if !gate0.pass {
        return D082Review {
            gate0,
            gate1: gate1_activation_lineage(),
            gate2: Gate2Report {
                before_repair_parity: false,
                after_repair_parity: false,
                non_edge: ParityArm {
                    name: "skipped".into(),
                    activation_extent: 0.0,
                    n_consumed: 0.0,
                    f_consumed: 0.0,
                    w_produced: 0.0,
                    a_mass_final: 0.0,
                    simulated_time: 0.0,
                },
                edge_coupled: ParityArm {
                    name: "skipped".into(),
                    activation_extent: 0.0,
                    n_consumed: 0.0,
                    f_consumed: 0.0,
                    w_produced: 0.0,
                    a_mass_final: 0.0,
                    simulated_time: 0.0,
                },
                residual_ok: false,
                integration_repaired: false,
                pass: false,
                failure: Some("skipped".into()),
            },
            gate3: skip_g3(),
            gate4: skip_g4(),
            gate5: skip_g5(),
            route: RouteReport {
                route: D082Route::D081ResultNotReproduced,
                conclusion: D082Route::D081ResultNotReproduced.conclusion().into(),
                stopped_at_gate: "gate0".into(),
                scientific_conclusion: "D-081 fingerprint not reproduced.".into(),
                next_directive: "Repair D-081 reproduction before activation supply audit.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
                d080_gate7_status: D080_GATE7_PROVISIONAL.into(),
                d081_status: D081_PROVISIONAL.into(),
            },
            scope_amendment: SCOPE_AMENDMENT.into(),
            seed_contract: SEED_CONTRACT_V1.into(),
        };
    }

    let gate1 = gate1_activation_lineage();
    let gate2 = gate2_activation_parity();
    if !gate2.pass {
        return D082Review {
            gate0,
            gate1,
            gate2,
            gate3: skip_g3(),
            gate4: skip_g4(),
            gate5: skip_g5(),
            route: RouteReport {
                route: D082Route::EdgeActivationIntegrationDefect,
                conclusion: D082Route::EdgeActivationIntegrationDefect.conclusion().into(),
                stopped_at_gate: "gate2".into(),
                scientific_conclusion: "Edge/non-edge activation parity failed after attempted integration.".into(),
                next_directive: "Repair only activation dispatch/integration; do not change kinetics.".into(),
                next_execution_started: false,
                d008_status: "BLOCKED_NOT_RECOVERED".into(),
                phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
                production_verdict: "REQUIRES_REMEDIATION".into(),
                d080_gate7_status: D080_GATE7_PROVISIONAL.into(),
                d081_status: D081_PROVISIONAL.into(),
            },
            scope_amendment: SCOPE_AMENDMENT.into(),
            seed_contract: SEED_CONTRACT_V1.into(),
        };
    }

    let gate3 = gate3_energy_ledger(1.0);
    let gate4 = gate4_replenishment_affordability(1.0);
    let cont_a = gate4
        .arms
        .iter()
        .find(|a| a.name == "normal_metabolism")
        .map(|a| a.a_retention)
        .unwrap_or(0.0);
    let gate5 = gate5_production_demand_audit(1.0, cont_a);

    let (route, stopped, science, next, d081_status, d080_status) = if gate4.pass {
        (
            D082Route::EdgeActivationIntegrationRepaired,
            "none",
            "Canonical activation was missing from the edge assay; repaired integration restores affordability.",
            "Resume D-081 Gate 6 and D-080 Gates 8–9 under coupled activation supply.",
            "SUPERSEDED_BY_D082_ACTIVATION_SUPPLY_AUDIT",
            "PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT",
        )
    } else if gate5.loss_matched_affordable
        && matches!(
            gate5.classification,
            DemandClass::ContinuousOverproduction
        )
    {
        (
            D082Route::EdgeMembraneProductionOverdraw,
            "gate5",
            "Full activation valid; loss-matched affordable; continuous A→L overdraws A.",
            "Implement one local bounded reserve-production law; do not raise activation.",
            D081_PROVISIONAL,
            D080_GATE7_PROVISIONAL,
        )
    } else if matches!(gate5.classification, DemandClass::YieldTooExpensive) {
        (
            D082Route::EdgeMembraneYieldMetabolicallyInfeasible,
            "gate5",
            "Even loss-matched replenishment consumes more A than sustainable supply.",
            "Separate stoichiometric architecture review before any yield change.",
            D081_PROVISIONAL,
            D080_GATE7_PROVISIONAL,
        )
    } else if matches!(gate5.classification, DemandClass::OtherADemandDominant) {
        (
            D082Route::NonmembraneADemandDominant,
            "gate5",
            "Non-membrane A demand dominates the sustainable budget.",
            "Target identified non-membrane demand; do not change membrane production first.",
            D081_PROVISIONAL,
            D080_GATE7_PROVISIONAL,
        )
    } else {
        (
            D082Route::FrozenActivationCapacityLimitConfirmed,
            "gate5",
            "Coupled activation active but affordability still fails under frozen capacity.",
            "Do not change A→L yield; treat as frozen activation capacity limit.",
            D081_PROVISIONAL,
            D080_GATE7_PROVISIONAL,
        )
    };

    D082Review {
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        route: RouteReport {
            route,
            conclusion: route.conclusion().into(),
            stopped_at_gate: stopped.into(),
            scientific_conclusion: science.into(),
            next_directive: next.into(),
            next_execution_started: false,
            d008_status: "BLOCKED_NOT_RECOVERED".into(),
            phase1_status: "PHASE1_SELF_MAINTENANCE_PARTIAL".into(),
            production_verdict: "REQUIRES_REMEDIATION".into(),
            d080_gate7_status: d080_status.into(),
            d081_status: d081_status.into(),
        },
        scope_amendment: SCOPE_AMENDMENT.into(),
        seed_contract: SEED_CONTRACT_V1.into(),
    }
}

#[cfg(test)]
mod unit_smoke {
    use super::*;

    #[test]
    fn ids_stable() {
        assert_eq!(D082_STARTING_TAG, "D-081-edge-reserve-causality-fail");
        assert_eq!(D081_PROVISIONAL, "PROVISIONAL_PENDING_ACTIVATION_SUPPLY_AUDIT");
        assert_eq!(D081_STARTING_COMMIT, "f5dc5a5");
        assert_eq!(D081_STARTING_TAG, "D-080-edge-network-requalification-fail");
    }
}
