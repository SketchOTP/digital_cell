// DC-FINAL-001-R4: contract ownership, V4 topology coherence, and evolution.
//
// Identical organisms are represented by an exact state plus an integer
// multiplicity.  The compression is semantic: shared-medium requests,
// organism/world material transfers, deaths, physical fissions, and mutation
// draws are all weighted or expanded by that multiplicity.  No cohort value
// enters organism biology.

use chemistry_core::d096_allocation::{
    expression_step, expression_step_activated_material_v2, expression_step_activated_material_v4,
    expression_step_activated_material_v4_turnover_only, mutate_allocation_at_reproduction,
    AllocationGenotype, AllocationParams, ExpressionLedger, ExpressionReject,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
use chemistry_core::mesh_fission::{segment_apposition_stress_audit, try_local_segment_fission};
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{
    growth_step, growth_step_with_placement, GrowthParams, GrowthPlacementMode,
};
use chemistry_core::mesh_mechanics::{
    compute_forces, local_pressure, mechanics_step_with_external_forces, remesh, MechParams,
};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_transport::{
    mean_occupancy, permeability, transport_step, TransportParams,
};
use chemistry_core::metabolic_reserve::ReserveParams;
use chemistry_core::planar_ring_topology::{remesh_preserving_simple, PlanarRingTopology};
use regulatory_core::{
    derive_local_activity, PlasticityStateV1, PolarityActuationParamsV1, PolarityMassParamsV1,
    PolarityMassStateV1,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::{env, fs, path::PathBuf};

#[path = "dcfinal001_r5_v4_neck.rs"]
mod r10_closure;

const DIRECTIVE: &str =
    "DC-FINAL-001-R4-CONTRACT-OWNERSHIP-V4-TOPOLOGY-COHERENCE-EVOLUTION-AND-FINAL-GOAL-CLOSURE-001";
const TEMPLATE_STEPS: usize = 6_500;
const PHASE_STEPS: usize = 2_500;
const R10R2_PHASE_STEPS: usize = 14_778;
const R10R7_SELECTION_STEPS: usize = 29_556;
const R10R9R4_FOUNDER_MULTIPLICITY: u64 = 900;
const R10_PREFIX_STEPS: usize = 2_500;
const FOUNDER_MULTIPLICITY: u64 = 150;
const REPLICATES: u64 = 2;
const REPRODUCTION_STEPS: usize = 12_000;
const DAUGHTER_CONTINUATION_STEPS: usize = 3_000;
// Numerical probe settings for the observer-only stability assay. These are
// not organism parameters and never enter a production transition.
const STABILITY_PROBE_EPSILON: f64 = 1.0e-7;
const STABILITY_PROBE_STEPS: usize = 24;
const STABILITY_CLASSIFICATION_TOLERANCE: f64 = 1.0e-3;
// R6 observer-only accepted-time checkpoints. These are fixed by the
// directive and are never selected from the R4 endpoint labels.
const R6_ATOMIC_CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];
// R7 reuses the exact R6 checkpoint contract, but captures the earlier
// pre-polarity.advance boundary.
const R7_FULL_CYCLE_CHECKPOINTS: [usize; 3] = R6_ATOMIC_CHECKPOINTS;

/// R8 asks the shared R7 lifecycle only for its exact replay seeds.  Skipping
/// R7's already-sealed response probe in that mode changes no organism
/// transition; it only avoids recomputing a superseded global Jacobian while
/// the clone-local causal-edge cuts are collected.
fn r8_capture_only() -> bool {
    matches!(
        env::var("DCFINAL001_R8_CAPTURE_ONLY").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

/// R9 is opt-in and remains disabled for every historical/current entry point
/// unless the dedicated R9 workflow explicitly enables it.  The flag changes
/// only the polarity chemistry input; it never changes the actuator or
/// mechanics contract.
fn r9_mechanosensitive_polarity_enabled() -> bool {
    matches!(
        env::var("DCFINAL001_R9_MECHANOSENSITIVE").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10_normal_remap_enabled() -> bool {
    matches!(
        env::var("DCFINAL001_R10_NORMAL_REMAP").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

/// R9 held-out histories use a domain-separated campaign seed.  This is an
/// assay identity only and is never selected from a morphogenetic outcome.
fn r9_held_out_history_mode() -> bool {
    matches!(
        env::var("DCFINAL001_R9_HELD_OUT").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10r9r5_canonical_lifecycle() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R5_CANONICAL").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10r9r1_reserve_enabled() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R1_RESERVE").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10r9r3_buffered_reserve_enabled() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R3_RESERVE").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10r9r4_mutation_off_only() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R4_MUTATION_OFF_ONLY")
            .ok()
            .as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10r9r1_reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut reaction = ReactionParams::default();
    if r10r9r1_reserve_enabled() {
        reaction.reserve = if r10r9r3_buffered_reserve_enabled() {
            ReserveParams::derived_buffered(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area())
        } else {
            ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area())
        };
    }
    reaction
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum PopulationBoundaryMode {
    RateReinterpretation,
    FixedConcentrationBoundary,
    /// Diagnostic-only adapter for the historical reproduction assay.  It
    /// delegates directly to each fixture mesh's exterior field and does not
    /// participate in population ecology.
    FixtureExteriorReference,
}

impl PopulationBoundaryMode {
    fn label(self) -> &'static str {
        match self {
            Self::RateReinterpretation => "SEALED_R10R5_RATE_REINTERPRETATION",
            Self::FixedConcentrationBoundary => "R10R6_FIXED_CONCENTRATION_BOUNDARY",
            Self::FixtureExteriorReference => "HISTORICAL_FIXTURE_EXTERIOR_REFERENCE",
        }
    }
}

/// Per-species source used only by the bounded R5 causal comparison. The
/// scheduled Resource waveform and exact per-arm fixture values are both
/// explicit diagnostic inputs; neither changes the production ecology.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
enum DiagnosticBoundarySource {
    Scheduled,
    Fixture(f64),
}

impl DiagnosticBoundarySource {
    fn value(self, environment: Environment, phase_step: usize) -> f64 {
        match self {
            Self::Scheduled => environment.fixed_inflow_concentrations(phase_step).0,
            Self::Fixture(value) => value,
        }
    }

    fn fuel_value(self, environment: Environment, phase_step: usize) -> f64 {
        match self {
            Self::Scheduled => environment.fixed_inflow_concentrations(phase_step).1,
            Self::Fixture(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum FissionClockMode {
    CurrentStep,
    HistoricalStepMinusOne,
}

impl FissionClockMode {
    fn label(self) -> &'static str {
        match self {
            Self::CurrentStep => "STEP_MODULO_CADENCE",
            Self::HistoricalStepMinusOne => "ABSOLUTE_STEP_MINUS_ONE_MODULO_CADENCE",
        }
    }

    fn cadence_step(self, step: usize) -> usize {
        match self {
            Self::CurrentStep => step,
            Self::HistoricalStepMinusOne => step.saturating_sub(1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum ReserveResolutionMode {
    PerStep,
    OnceBeforeTrajectory,
}

impl ReserveResolutionMode {
    fn label(self) -> &'static str {
        match self {
            Self::PerStep => "RESOLVED_FROM_CURRENT_MESH_EACH_STEP",
            Self::OnceBeforeTrajectory => "RESOLVED_ON_INITIAL_MESH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Environment {
    Resource,
    Damage,
}

impl Environment {
    fn label(self) -> &'static str {
        match self {
            Self::Resource => "RESOURCE_CHALLENGE",
            Self::Damage => "DAMAGE_CHALLENGE",
        }
    }

    fn fixed_inflow_concentrations(self, phase_step: usize) -> (f64, f64) {
        match self {
            // Frozen D-096 H pulse/lean schedule.
            Self::Resource if phase_step % 400 < 100 => (2.75, 1.0),
            Self::Resource => (0.264, 1.0),
            // Frozen D-096 B resource boundary.
            Self::Damage => (1.98, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cohort {
    mesh: MaterialMesh,
    plasticity: Option<PlasticityStateV1>,
    count: u64,
    generation: u32,
    birth_mass: f64,
    id: u64,
}

/// R4-only sidecar for the accepted R3 polarity state.  It is deliberately
/// keyed by physical cohort identity and is not part of the existing cohort
/// biology schema.  Fission insertion/removal is handled at the same event as
/// the corresponding physical daughter transition.
type PolarityStateMap = BTreeMap<u64, PolarityMassStateV1>;

/// Diagnostic-only clone of the complete state presented at the R7
/// pre-polarity.advance boundary.  It is never part of the production
/// transition or used to steer an organism.
#[derive(Debug, Clone, Deserialize)]
struct R7BoundarySeed {
    cohort: Cohort,
    world: OpenMedium,
    ledger: CampaignLedger,
    polarity_states: PolarityStateMap,
    next_id: u64,
    step: usize,
    campaign_seed: u64,
    connected: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WorldLedger {
    initial_n: f64,
    initial_f: f64,
    inflow_n: f64,
    inflow_f: f64,
    external_source_n_to_bath: f64,
    external_source_f_to_bath: f64,
    bath_n_to_outflow: f64,
    bath_f_to_outflow: f64,
    delivered_n: f64,
    delivered_f: f64,
    returned_n: f64,
    returned_f: f64,
    c_outflow: f64,
    a_outflow: f64,
    w_outflow: f64,
    damage_structural_sink: f64,
    damage_membrane_sink: f64,
    physical_death_n_sink: f64,
    physical_death_f_sink: f64,
    physical_death_structural_sink: f64,
    invalidated_n_terminal: f64,
    invalidated_f_terminal: f64,
    invalidated_structural_terminal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenMedium {
    /// One fixed reference volume per preregistered founder. This scales the
    /// assay vessel, not any organism rule, and never follows population size.
    volume: f64,
    n_mass: f64,
    f_mass: f64,
    ledger: WorldLedger,
}

#[derive(Debug, Clone, Serialize)]
struct BoundaryExchangeReport {
    sampled_boundary_n: f64,
    sampled_boundary_f: f64,
    transport_boundaries: Vec<[f64; 2]>,
    published_boundary_n: f64,
    published_boundary_f: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CampaignLedger {
    mutation_opportunities: u64,
    mutations: u64,
    physical_fissions: u64,
    valid_simple_fissions: u64,
    physical_deaths: u64,
    runtime_invalidations: u64,
    expression_failures: u64,
    invalid_geometry_events: u64,
    partition_failures: u64,
    reaction_n_consumed: f64,
    reaction_f_consumed: f64,
    a_produced: f64,
    w_produced: f64,
    growth_material: f64,
    expression_material: f64,
    expression_catalyst_precursor_a: f64,
    expression_activation: f64,
    expression_turnover_waste: f64,
    m1_structural_build: f64,
    m1_structural_turnover: f64,
    growth_a_consumed: f64,
    growth_w_produced: f64,
    reserve_a_to_r: f64,
    reserve_r_to_a: f64,
    reserve_r_to_w: f64,
    reserve_r_to_m: f64,
    reserve_funded_growth: f64,
    reserve_active_steps: u64,
    mechanics_topology_structural_net: f64,
    bootstrap_fission_closure_structural_input: f64,
    post_bootstrap_fission_closure_structural_input: f64,
    active_a_spent: f64,
    active_w_produced: f64,
    reaction_activation_equivalent_closure_residual: f64,
    polarity_source_a_debited: f64,
    polarity_a_spent: f64,
    polarity_w_produced: f64,
    adaptation_remesh_mappings: u64,
    legal_depletion_steps: u64,
    observer_nonviable_observations: u64,
    computational_rejections: u64,
    physical_disintegrations: u64,
    rollback_count: u64,
    accepted_steps: u64,
    rejected_steps: u64,
    numerical_invalid: bool,
    numerical_invalid_reason: Option<Value>,
    lifecycle_events: Vec<Value>,
    bootstrap_physical_fissions: u64,
    post_bootstrap_physical_fissions: u64,
    mutation_events: Vec<Value>,
    physical_birth_events: Vec<Value>,
    fissions_by_parent_genotype: BTreeMap<String, u64>,
    deaths_by_genotype: BTreeMap<String, u64>,
    phenotype_by_genotype: BTreeMap<String, GenotypePhenotypeLedger>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum D096ExpressionPath {
    V1Structural,
    Off,
    ActivatedMaterialCandidate,
    V2ActivatedMaterial,
    V4FiniteBudgetCentered,
}

impl D096ExpressionPath {
    fn label(self) -> &'static str {
        match self {
            Self::V1Structural => "CURRENT_D096_V1",
            Self::Off => "D096_OFF",
            Self::ActivatedMaterialCandidate => "D096_ACTIVATED_MATERIAL_CANDIDATE",
            Self::V2ActivatedMaterial => "D096_V2_ACTIVATED_MATERIAL",
            Self::V4FiniteBudgetCentered => "D096_V4_FINITE_BUDGET_CENTERED_INTENSIVE_GAIN",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ExpressionAccounting {
    structural_consumed: f64,
    catalyst_precursor_a: f64,
    activation_consumed: f64,
    maintenance_consumed: f64,
    turnover_waste: f64,
}

impl From<ExpressionLedger> for ExpressionAccounting {
    fn from(value: ExpressionLedger) -> Self {
        Self {
            structural_consumed: value.material_consumed,
            catalyst_precursor_a: 0.0,
            activation_consumed: value.activation_consumed,
            maintenance_consumed: value.maintenance_consumed,
            turnover_waste: value.turnover_waste,
        }
    }
}

/// Observer-only R10R3 candidate. This deliberately retains the historical v1
/// schema stamp while changing no production implementation; it is used only
/// for the preregistered Gate 4/5 counterfactual.
fn expression_step_activated_material_candidate(
    mesh: &mut MaterialMesh,
    params: &AllocationParams,
    dt: f64,
) -> Result<ExpressionAccounting, &'static str> {
    if !dt.is_finite() || dt <= 0.0 {
        return Err("invalid step");
    }
    let mut next = mesh.clone();
    let area = next.area().max(1e-9);
    let structural = next.total_structural_mass().max(0.0);
    let activated = (next.interior.a.max(0.0) * area).max(0.0);
    if structural <= 0.0 || activated <= 0.0 {
        return Err("insufficient material or activated resource");
    }
    let state = next.finite_allocation.as_mut().ok_or("allocation absent")?;
    if !state.genotype.valid(params) {
        return Err("invalid allocation");
    }
    let total_c = state.catalysts.iter().sum::<f64>();
    let production_demand =
        params.synthesis_rate * structural.min(activated / params.activation_cost);
    let maintenance = (params.maintenance_rate * total_c * dt).min(activated);
    let available_after_maintenance = (activated - maintenance).max(0.0);
    let max_synthesis = available_after_maintenance / (1.0 + params.activation_cost) / dt;
    let actual_synthesis = production_demand.min(max_synthesis);
    let mut accounting = ExpressionAccounting {
        maintenance_consumed: maintenance,
        ..Default::default()
    };
    for index in 0..state.catalysts.len() {
        let synthesis_rate = state.genotype.0[index] * actual_synthesis;
        let turnover_rate = params.turnover_rate * state.catalysts[index];
        state.catalysts[index] =
            (state.catalysts[index] + (synthesis_rate - turnover_rate) * dt).max(0.0);
        accounting.catalyst_precursor_a += synthesis_rate * dt;
        accounting.turnover_waste += turnover_rate * dt;
    }
    accounting.activation_consumed = params.activation_cost * accounting.catalyst_precursor_a;
    let activated_spent = accounting.catalyst_precursor_a
        + accounting.activation_consumed
        + accounting.maintenance_consumed;
    if activated_spent > activated + 1e-10 {
        return Err("activated material cap violated");
    }
    next.interior.a -= activated_spent / area;
    next.interior.w += (accounting.activation_consumed
        + accounting.maintenance_consumed
        + accounting.turnover_waste)
        / area;
    *mesh = next;
    Ok(accounting)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum ExpressionBoundaryError {
    IncompatibleSchema,
    InvalidAllocation,
    InvalidStep,
    InsufficientMaterial,
    InsufficientActivatedResource,
    CandidateRejected,
}

impl ExpressionBoundaryError {
    fn label(self) -> &'static str {
        match self {
            Self::IncompatibleSchema => "INCOMPATIBLE_SCHEMA",
            Self::InvalidAllocation => "INVALID_ALLOCATION",
            Self::InvalidStep => "INVALID_STEP",
            Self::InsufficientMaterial => "INSUFFICIENT_MATERIAL",
            Self::InsufficientActivatedResource => "INSUFFICIENT_ACTIVATED_RESOURCE",
            Self::CandidateRejected => "CANDIDATE_REJECTED",
        }
    }
}

impl From<ExpressionReject> for ExpressionBoundaryError {
    fn from(value: ExpressionReject) -> Self {
        match value {
            ExpressionReject::IncompatibleSchema => Self::IncompatibleSchema,
            ExpressionReject::InvalidAllocation => Self::InvalidAllocation,
            ExpressionReject::InvalidStep => Self::InvalidStep,
            ExpressionReject::InsufficientMaterial => Self::InsufficientMaterial,
            ExpressionReject::InsufficientActivatedResource => Self::InsufficientActivatedResource,
        }
    }
}

fn apply_expression_path(
    mesh: &mut MaterialMesh,
    params: &AllocationParams,
    dt: f64,
    path: D096ExpressionPath,
) -> Result<ExpressionAccounting, ExpressionBoundaryError> {
    match path {
        D096ExpressionPath::V1Structural => expression_step(mesh, params, dt)
            .map(ExpressionAccounting::from)
            .map_err(ExpressionBoundaryError::from),
        D096ExpressionPath::Off => Ok(ExpressionAccounting::default()),
        D096ExpressionPath::ActivatedMaterialCandidate => {
            expression_step_activated_material_candidate(mesh, params, dt)
                .map_err(|_| ExpressionBoundaryError::CandidateRejected)
        }
        D096ExpressionPath::V2ActivatedMaterial => {
            expression_step_activated_material_v2(mesh, params, dt)
                .map(|ledger| ExpressionAccounting {
                    structural_consumed: ledger.material_consumed,
                    catalyst_precursor_a: ledger.catalyst_precursor_consumed,
                    activation_consumed: ledger.activation_consumed,
                    maintenance_consumed: ledger.maintenance_consumed,
                    turnover_waste: ledger.turnover_waste,
                })
                .map_err(ExpressionBoundaryError::from)
        }
        D096ExpressionPath::V4FiniteBudgetCentered => {
            expression_step_activated_material_v4(mesh, params, dt)
                .map(|ledger| ExpressionAccounting {
                    structural_consumed: ledger.material_consumed,
                    catalyst_precursor_a: ledger.catalyst_precursor_consumed,
                    activation_consumed: ledger.activation_consumed,
                    maintenance_consumed: ledger.maintenance_consumed,
                    turnover_waste: ledger.turnover_waste,
                })
                .map_err(ExpressionBoundaryError::from)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct GenotypePhenotypeLedger {
    organism_step_exposure: u64,
    expression_material: f64,
    expression_activation: f64,
    n_uptake: f64,
    f_uptake: f64,
    n_return: f64,
    f_return: f64,
    reaction_n_consumed: f64,
    reaction_f_consumed: f64,
    a_produced: f64,
    w_produced: f64,
    growth_material: f64,
    damage_structural: f64,
    damage_membrane: f64,
    active_a_spent: f64,
    active_w_produced: f64,
    fission_attempts: u64,
    physical_fissions: u64,
    physical_deaths: u64,
    terminal_multiplicity: u64,
}

fn phenotype_ledger<'a>(
    ledger: &'a mut CampaignLedger,
    genotype: AllocationGenotype,
) -> &'a mut GenotypePhenotypeLedger {
    ledger
        .phenotype_by_genotype
        .entry(genotype_key(genotype))
        .or_default()
}

fn genotype_key(genotype: AllocationGenotype) -> String {
    genotype
        .0
        .iter()
        .map(|value| format!("{value:.17}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn mutation_seed(campaign_seed: u64, step: usize, cohort: u64, ordinal: u64, child: u64) -> u64 {
    splitmix64(
        campaign_seed
            ^ (step as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ cohort.rotate_left(17)
            ^ ordinal.wrapping_mul(0xd1b5_4a32_d192_ed03)
            ^ child.wrapping_mul(0x94d0_49bb_1331_11eb),
    )
}

fn perturb_seed_1(mesh: &mut MaterialMesh) {
    let center = mesh.centroid();
    let (sine, cosine) = 0.3_f64.sin_cos();
    for point in &mut mesh.vertices {
        let x = point[0] - center[0];
        let y = point[1] - center[1];
        point[0] = center[0] + cosine * x - sine * y;
        point[1] = center[1] + sine * x + cosine * y;
    }
    for (index, point) in mesh.vertices.iter_mut().enumerate() {
        let fraction = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        point[0] += 0.35 * (fraction - 0.5);
        point[1] += 0.35 * ((fraction * 7.13).fract() - 0.5);
    }
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] = center[0] + (point[0] - center[0]) * 1.25;
    }
}

/// Reconstruct the sealed WP1 seed-1 parent whose two daughters independently
/// passed the complete 3,000-step simple-boundary viability assay. The returned
/// state is immediately before its unchanged fission.
fn lawful_parent_template() -> (MaterialMesh, f64, usize) {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb_seed_1(&mut mesh);
    let birth_mass = mesh.total_structural_mass();
    let mechanics = MechParams::default();
    let transport = TransportParams::default();
    let reaction = ReactionParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for step in 0..TEMPLATE_STEPS {
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        assert!(mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some());
        let _ = remesh(&mut mesh);
        if step % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        assert!(polygon_simple(&mesh.vertices));
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            if let Some((a, b, event)) = try_local_fission(&mesh, &fission) {
                if event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices)
                {
                    return (mesh, birth_mass, step + 1);
                }
            }
        }
    }
    panic!("sealed seed-10 geometry-valid parent did not replay");
}

fn signed_request(
    mesh: &MaterialMesh,
    transport: &TransportParams,
    species: &str,
    interior: f64,
    boundary: f64,
    dt: f64,
) -> f64 {
    let theta = mean_occupancy(mesh);
    let ruptured_fraction =
        mesh.edges.iter().filter(|edge| edge.ruptured).count() as f64 / mesh.n().max(1) as f64;
    transport.k_flux
        * permeability(theta, species)
        * (1.0 + 4.0 * ruptured_fraction)
        * (boundary - interior)
        * mesh.perimeter().max(1e-6)
        * dt
}

impl OpenMedium {
    fn accepted_boundary(&self) -> LumpedChem {
        LumpedChem {
            n: self.n_mass / self.volume,
            f: self.f_mass / self.volume,
            ..LumpedChem::default()
        }
    }

    fn publish_accepted_boundary(&self, cohorts: &mut [Cohort]) {
        let boundary = self.accepted_boundary();
        for cohort in cohorts {
            cohort.mesh.exterior = boundary;
        }
    }

    fn new(environment: Environment, volume: f64) -> Self {
        let (n, f) = environment.fixed_inflow_concentrations(0);
        let initial_n = n * volume;
        let initial_f = f * volume;
        Self {
            volume,
            n_mass: initial_n,
            f_mass: initial_f,
            ledger: WorldLedger {
                initial_n,
                initial_f,
                ..WorldLedger::default()
            },
        }
    }

    fn add_fixed_inflow(&mut self, environment: Environment, phase_step: usize, dt: f64) {
        let (n, f) = environment.fixed_inflow_concentrations(phase_step);
        let add_n = n * dt * self.volume;
        let add_f = f * dt * self.volume;
        self.n_mass += add_n;
        self.f_mass += add_f;
        self.ledger.inflow_n += add_n;
        self.ledger.inflow_f += add_f;
        self.ledger.external_source_n_to_bath += add_n;
        self.ledger.external_source_f_to_bath += add_f;
    }

    fn refresh_fixed_concentration_boundary(
        &mut self,
        environment: Environment,
        phase_step: usize,
    ) {
        let (target_n, target_f) = environment.fixed_inflow_concentrations(phase_step);
        self.refresh_fixed_concentration_boundary_values(target_n, target_f);
    }

    fn refresh_fixed_concentration_boundary_values(&mut self, target_n: f64, target_f: f64) {
        let target_n_mass = target_n * self.volume;
        let target_f_mass = target_f * self.volume;
        let delta_n = target_n_mass - self.n_mass;
        let delta_f = target_f_mass - self.f_mass;
        if delta_n >= 0.0 {
            self.n_mass += delta_n;
            self.ledger.external_source_n_to_bath += delta_n;
        } else {
            let outflow = (-delta_n).min(self.n_mass);
            self.n_mass -= outflow;
            self.ledger.bath_n_to_outflow += outflow;
        }
        if delta_f >= 0.0 {
            self.f_mass += delta_f;
            self.ledger.external_source_f_to_bath += delta_f;
        } else {
            let outflow = (-delta_f).min(self.f_mass);
            self.f_mass -= outflow;
            self.ledger.bath_f_to_outflow += outflow;
        }
    }

    /// Exact multiplicity-aware finite shared-boundary exchange. The effective
    /// boundary passed to the frozen transport law is reduced only by the one
    /// common finite-world allocation scale. The staged transport boundary is
    /// not a mechanical boundary: after all requests are settled, the actual
    /// accepted finite-bath state is published to every downstream consumer.
    fn exchange(&mut self, cohorts: &mut [Cohort], transport: &TransportParams, dt: f64) {
        let _ = self.exchange_with_report(cohorts, transport, dt);
    }

    fn exchange_with_report(
        &mut self,
        cohorts: &mut [Cohort],
        transport: &TransportParams,
        dt: f64,
    ) -> BoundaryExchangeReport {
        let boundary_n = self.n_mass / self.volume;
        let boundary_f = self.f_mass / self.volume;
        let mut raw = Vec::with_capacity(cohorts.len());
        let mut transport_boundaries = Vec::with_capacity(cohorts.len());
        let mut requested_n = 0.0;
        let mut requested_f = 0.0;
        let mut returned_n = 0.0;
        let mut returned_f = 0.0;
        for cohort in cohorts.iter() {
            let area = cohort.mesh.area().max(1e-15);
            let rn = signed_request(
                &cohort.mesh,
                transport,
                "N",
                cohort.mesh.interior.n,
                boundary_n,
                dt,
            );
            let rf = signed_request(
                &cohort.mesh,
                transport,
                "F",
                cohort.mesh.interior.f,
                boundary_f,
                dt,
            );
            let rn = rn.max(-cohort.mesh.interior.n.max(0.0) * area);
            let rf = rf.max(-cohort.mesh.interior.f.max(0.0) * area);
            if rn >= 0.0 {
                requested_n += rn * cohort.count as f64;
            } else {
                returned_n += -rn * cohort.count as f64;
            }
            if rf >= 0.0 {
                requested_f += rf * cohort.count as f64;
            } else {
                returned_f += -rf * cohort.count as f64;
            }
            raw.push((rn, rf));
        }
        let available_n = self.n_mass + returned_n;
        let available_f = self.f_mass + returned_f;
        let n_scale = if requested_n > 0.0 {
            (available_n / requested_n).min(1.0)
        } else {
            1.0
        };
        let f_scale = if requested_f > 0.0 {
            (available_f / requested_f).min(1.0)
        } else {
            1.0
        };
        let scale = n_scale.min(f_scale).clamp(0.0, 1.0);

        for (cohort, (rn, rf)) in cohorts.iter_mut().zip(raw) {
            cohort.mesh.exterior.c = 0.0;
            cohort.mesh.exterior.a = 0.0;
            cohort.mesh.exterior.w = 0.0;
            cohort.mesh.exterior.n = if rn >= 0.0 {
                cohort.mesh.interior.n + scale * (boundary_n - cohort.mesh.interior.n)
            } else {
                boundary_n
            };
            cohort.mesh.exterior.f = if rf >= 0.0 {
                cohort.mesh.interior.f + scale * (boundary_f - cohort.mesh.interior.f)
            } else {
                boundary_f
            };
            transport_boundaries.push([cohort.mesh.exterior.n, cohort.mesh.exterior.f]);
            let ledger = transport_step(&mut cohort.mesh, transport, dt);
            let count = cohort.count as f64;
            self.n_mass += count * (ledger.n_out - ledger.n_in);
            self.f_mass += count * (ledger.f_out - ledger.f_in);
            self.ledger.delivered_n += count * ledger.n_in;
            self.ledger.delivered_f += count * ledger.f_in;
            self.ledger.returned_n += count * ledger.n_out;
            self.ledger.returned_f += count * ledger.f_out;
            self.ledger.c_outflow += count * ledger.c_leak;
            self.ledger.a_outflow += count * ledger.a_leak;
            self.ledger.w_outflow += count * ledger.w_out;
        }
        self.n_mass = self.n_mass.max(0.0);
        self.f_mass = self.f_mass.max(0.0);
        self.publish_accepted_boundary(cohorts);
        BoundaryExchangeReport {
            sampled_boundary_n: boundary_n,
            sampled_boundary_f: boundary_f,
            transport_boundaries,
            published_boundary_n: self.n_mass / self.volume,
            published_boundary_f: self.f_mass / self.volume,
        }
    }
}

fn r10_exchange_with_observer(
    world: &mut OpenMedium,
    cohorts: &mut [Cohort],
    transport: &TransportParams,
    dt: f64,
    ledger: &mut CampaignLedger,
) {
    let before = cohorts
        .iter()
        .map(|cohort| {
            let area = cohort.mesh.area();
            (
                cohort
                    .mesh
                    .finite_allocation
                    .expect("R10 allocation")
                    .genotype,
                cohort.count as f64,
                cohort.mesh.interior.n * area,
                cohort.mesh.interior.f * area,
            )
        })
        .collect::<Vec<_>>();
    world.exchange(cohorts, transport, dt);
    for (cohort, (genotype, count, before_n, before_f)) in cohorts.iter().zip(before) {
        let area = cohort.mesh.area();
        let delta_n = cohort.mesh.interior.n * area - before_n;
        let delta_f = cohort.mesh.interior.f * area - before_f;
        let observer = phenotype_ledger(ledger, genotype);
        observer.n_uptake += delta_n.max(0.0) * count;
        observer.f_uptake += delta_f.max(0.0) * count;
        observer.n_return += (-delta_n).max(0.0) * count;
        observer.f_return += (-delta_f).max(0.0) * count;
    }
}

/// Execute the historical fixture-boundary transport contract without a
/// shared bath.  The fixture's own exterior concentrations are the complete
/// boundary input, matching the direct `transport_step` used by the original
/// reproduction assay.  This is intentionally a diagnostic adapter only.
fn r10_fixture_exterior_exchange_with_observer(
    cohorts: &mut [Cohort],
    transport: &TransportParams,
    dt: f64,
    ledger: &mut CampaignLedger,
) {
    for cohort in cohorts {
        let before_n = cohort.mesh.interior.n * cohort.mesh.area();
        let before_f = cohort.mesh.interior.f * cohort.mesh.area();
        let _ = transport_step(&mut cohort.mesh, transport, dt);
        let after_n = cohort.mesh.interior.n * cohort.mesh.area();
        let after_f = cohort.mesh.interior.f * cohort.mesh.area();
        let count = cohort.count as f64;
        let genotype = cohort
            .mesh
            .finite_allocation
            .expect("R10 allocation")
            .genotype;
        let observer = phenotype_ledger(ledger, genotype);
        observer.n_uptake += (after_n - before_n).max(0.0) * count;
        observer.f_uptake += (after_f - before_f).max(0.0) * count;
        observer.n_return += (before_n - after_n).max(0.0) * count;
        observer.f_return += (before_f - after_f).max(0.0) * count;
        // The direct fixture has no population bath.  Do not fabricate world
        // source/sink entries for this comparison adapter; the per-genotype
        // uptake/return accounting above is the diagnostic record.
    }
}

fn fixed_boundary_transport_reference_parity() -> Value {
    let (template, _, _, _) = r10_closure::r10_seed3_fission_state();
    let transport = TransportParams::default();
    let dt = MechParams::default().dt;
    let (target_n, target_f) = Environment::Resource.fixed_inflow_concentrations(0);

    let mut reference = template.clone();
    let original_exterior = reference.exterior;
    reference.exterior.n = target_n;
    reference.exterior.f = target_f;
    let reference_ledger = transport_step(&mut reference, &transport, dt);
    reference.exterior = original_exterior;

    let mut candidate = Cohort {
        mesh: template,
        plasticity: None,
        count: 1,
        generation: 0,
        birth_mass: 0.0,
        id: 1,
    };
    let mut bath = OpenMedium {
        volume: 1.0,
        n_mass: target_n,
        f_mass: target_f,
        ledger: WorldLedger::default(),
    };
    bath.exchange(std::slice::from_mut(&mut candidate), &transport, dt);
    let n_delta = (candidate.mesh.interior.n - reference.interior.n).abs();
    let f_delta = (candidate.mesh.interior.f - reference.interior.f).abs();
    let delivered_n_delta = (bath.ledger.delivered_n - reference_ledger.n_in).abs();
    let delivered_f_delta = (bath.ledger.delivered_f - reference_ledger.f_in).abs();
    let pass = n_delta <= 1e-12
        && f_delta <= 1e-12
        && delivered_n_delta <= 1e-12
        && delivered_f_delta <= 1e-12;
    json!({
        "mode": "one_reference_volume_fixed_boundary_vs_direct_boundary_transport",
        "target_n_concentration": target_n,
        "target_f_concentration": target_f,
        "dt": dt,
        "n_in_reference": reference_ledger.n_in,
        "f_in_reference": reference_ledger.f_in,
        "n_in_candidate": bath.ledger.delivered_n,
        "f_in_candidate": bath.ledger.delivered_f,
        "interior_n_abs_delta": n_delta,
        "interior_f_abs_delta": f_delta,
        "delivered_n_abs_delta": delivered_n_delta,
        "delivered_f_abs_delta": delivered_f_delta,
        "tolerance": 1e-12,
        "pass": pass,
    })
}

fn boundary_state_coherence_contract() -> Value {
    let (template, _, birth_mass, _) = r10_closure::r10_seed3_fission_state();
    let transport = TransportParams::default();
    let dt = MechParams::default().dt;
    let fixture_exterior = template.exterior;
    let pressure_cases = [
        ("resource_pulse", Environment::Resource, 0_usize),
        ("resource_lean", Environment::Resource, 100_usize),
        ("damage", Environment::Damage, 0_usize),
    ];
    let pressure_rows = pressure_cases
        .into_iter()
        .map(|(label, environment, phase_step)| {
            let (target_n, target_f) = environment.fixed_inflow_concentrations(phase_step);
            let stale_pressure = local_pressure(&template, 0);
            let mut coherent = template.clone();
            coherent.exterior = LumpedChem {
                n: target_n,
                f: target_f,
                ..LumpedChem::default()
            };
            let coherent_pressure = local_pressure(&coherent, 0);
            let expected_outside = 0.5 * (target_n + target_f);
            let expected_inside = template.interior.c
                + template.interior.a
                + 0.5 * (template.interior.n + template.interior.f);
            json!({
                "label": label,
                "environment": environment.label(),
                "phase_step": phase_step,
                "scheduled_reservoir_concentration": [target_n, target_f],
                "fixture_exterior": fixture_exterior,
                "fixture_pressure": stale_pressure,
                "coherent_boundary_outside_pressure_term": expected_outside,
                "coherent_pressure": coherent_pressure,
                "pressure_delta_from_stale_fixture": coherent_pressure - stale_pressure,
                "pressure_equation_pass": (coherent_pressure - (expected_inside - expected_outside)).abs() <= 1e-12,
                "pressure_equation": "inside - (C_out + A_out + 0.5*(N_out + F_out))",
            })
        })
        .collect::<Vec<_>>();

    let allocation = AllocationParams::default();
    let mut candidate_mesh = template.clone();
    candidate_mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
    let mut candidate = Cohort {
        mesh: candidate_mesh,
        plasticity: None,
        count: 1,
        generation: 0,
        birth_mass,
        id: 1,
    };
    let stale_before_exchange = candidate.mesh.exterior;
    let mut bath = OpenMedium::new(Environment::Resource, 150.0);
    bath.refresh_fixed_concentration_boundary(Environment::Resource, 0);
    let sampled_boundary = bath.accepted_boundary();
    let report = bath.exchange_with_report(std::slice::from_mut(&mut candidate), &transport, dt);
    let published = candidate.mesh.exterior;
    let actual = bath.accepted_boundary();
    let published_matches_world = published == actual;
    let stale_was_not_published = published != stale_before_exchange;
    let mechanics_reads_published_boundary = {
        let mut expected = candidate.mesh.clone();
        expected.exterior = actual;
        (local_pressure(&candidate.mesh, 0) - local_pressure(&expected, 0)).abs() <= 1e-12
    };

    let make_order_cohort = |index: usize, count: u64| {
        let mut mesh = r10_closure::r10_reproduction_fixture(index);
        mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
        let vertex_count = mesh.n();
        Cohort {
            birth_mass: mesh.total_structural_mass(),
            mesh,
            plasticity: Some(PlasticityStateV1::new(vertex_count)),
            count,
            generation: 0,
            id: index as u64 + 1,
        }
    };
    let run_order = |reverse: bool| {
        let mut cohorts = vec![make_order_cohort(0, 75), make_order_cohort(1, 75)];
        if reverse {
            cohorts.reverse();
        }
        let initial_n =
            OpenMedium::new(Environment::Resource, 150.0).n_mass + organism_amount(&cohorts, 'n');
        let initial_f =
            OpenMedium::new(Environment::Resource, 150.0).f_mass + organism_amount(&cohorts, 'f');
        let mut world = OpenMedium::new(Environment::Resource, 150.0);
        world.refresh_fixed_concentration_boundary(Environment::Resource, 0);
        let before_refresh_n = world.n_mass;
        let before_refresh_f = world.f_mass;
        let report = world.exchange_with_report(&mut cohorts, &transport, dt);
        cohorts.sort_by_key(|cohort| cohort.id);
        let mut internal_state = cohorts.clone();
        for cohort in &mut internal_state {
            cohort.mesh.exterior = LumpedChem::default();
        }
        let final_n = world.n_mass + organism_amount(&cohorts, 'n');
        let final_f = world.f_mass + organism_amount(&cohorts, 'f');
        (
            deterministic_state_digest(&internal_state),
            world,
            report,
            [
                initial_n,
                initial_f,
                before_refresh_n,
                before_refresh_f,
                final_n,
                final_f,
            ],
        )
    };
    let (ordered_digest, ordered_world, ordered_report, ordered_totals) = run_order(false);
    let (reversed_digest, reversed_world, reversed_report, reversed_totals) = run_order(true);
    let order_tolerance = 1e-12;
    let conservation_tolerance = 1e-10;
    let order_independent = ordered_digest == reversed_digest
        && (ordered_world.n_mass - reversed_world.n_mass).abs() <= order_tolerance
        && (ordered_world.f_mass - reversed_world.f_mass).abs() <= order_tolerance;
    let finite_donor_n_residual = (ordered_totals[0] - ordered_totals[4]).abs();
    let finite_donor_f_residual = (ordered_totals[1] - ordered_totals[5]).abs();
    let finite_donor_conservation = finite_donor_n_residual <= conservation_tolerance
        && finite_donor_f_residual <= conservation_tolerance;
    let pressure_cases_pass = pressure_rows
        .iter()
        .all(|row| row["pressure_equation_pass"] == true);
    let fixture_mismatch_observed = pressure_rows.iter().any(|row| {
        row["pressure_delta_from_stale_fixture"]
            .as_f64()
            .map(|delta| delta.abs() > order_tolerance)
            .unwrap_or(false)
    });
    let transport_cap_is_staged_separately = report
        .transport_boundaries
        .iter()
        .all(|boundary| boundary[0].is_finite() && boundary[1].is_finite())
        && report.published_boundary_n.is_finite()
        && report.published_boundary_f.is_finite();
    let pass = pressure_rows.len() == 3
        && pressure_cases_pass
        && fixture_mismatch_observed
        && published_matches_world
        && stale_was_not_published
        && mechanics_reads_published_boundary
        && order_independent
        && finite_donor_conservation
        && transport_cap_is_staged_separately;
    json!({
        "pass": pass,
        "pressure_cases_pass": pressure_cases_pass,
        "fixture_mismatch_observed": fixture_mismatch_observed,
        "pressure_cases": pressure_rows,
        "exchange_provenance": {
            "sampled_boundary": sampled_boundary,
            "sampled_stage": "accepted finite bath before transport",
            "transport_allocation_boundaries": report.transport_boundaries,
            "published_mechanical_boundary": published,
            "published_stage": "accepted finite bath after complete simultaneous exchange before mechanics",
            "actual_world_boundary": actual,
            "published_matches_world": published_matches_world,
            "stale_fixture_before_exchange": stale_before_exchange,
            "stale_fixture_was_not_published": stale_was_not_published,
            "mechanics_reads_published_boundary": mechanics_reads_published_boundary,
            "transport_cap_is_staged_separately": transport_cap_is_staged_separately,
        },
        "finite_donor_conservation": {
            "ordered_initial_total_n": ordered_totals[0],
            "ordered_final_total_n": ordered_totals[4],
            "ordered_initial_total_f": ordered_totals[1],
            "ordered_final_total_f": ordered_totals[5],
            "n_residual": finite_donor_n_residual,
            "f_residual": finite_donor_f_residual,
            "tolerance": conservation_tolerance,
            "pass": finite_donor_conservation,
        },
        "cohort_order_independence": {
            "ordered_digest": ordered_digest,
            "reversed_digest": reversed_digest,
            "ordered_world_n": ordered_world.n_mass,
            "reversed_world_n": reversed_world.n_mass,
            "ordered_world_f": ordered_world.f_mass,
            "reversed_world_f": reversed_world.f_mass,
            "ordered_transport_boundaries": ordered_report.transport_boundaries,
            "reversed_transport_boundaries": reversed_report.transport_boundaries,
            "tolerance": order_tolerance,
            "pass": order_independent,
        },
    })
}

fn apply_damage(cohort: &mut Cohort, step: usize, world: &mut OpenMedium) {
    if step % 350 != 0 || cohort.mesh.edges.is_empty() {
        return;
    }
    let structural = 0.08_f64.min(cohort.mesh.edges[0].m.max(0.0));
    let membrane = 0.048_f64.min(cohort.mesh.edges[0].b.max(0.0));
    cohort.mesh.edges[0].m -= structural;
    cohort.mesh.edges[0].b -= membrane;
    world.ledger.damage_structural_sink += structural * cohort.count as f64;
    world.ledger.damage_membrane_sink += membrane * cohort.count as f64;
}

fn organism_amount(cohorts: &[Cohort], species: char) -> f64 {
    cohorts
        .iter()
        .map(|cohort| {
            let concentration = match species {
                'n' => cohort.mesh.interior.n,
                'f' => cohort.mesh.interior.f,
                'a' => cohort.mesh.interior.a,
                'w' => cohort.mesh.interior.w,
                _ => 0.0,
            };
            concentration.max(0.0) * cohort.mesh.area() * cohort.count as f64
        })
        .sum()
}

fn population_count(cohorts: &[Cohort]) -> u64 {
    cohorts.iter().map(|cohort| cohort.count).sum()
}

fn mean_genotype(cohorts: &[Cohort]) -> [f64; 4] {
    let count = population_count(cohorts).max(1) as f64;
    let mut mean = [0.0; 4];
    for cohort in cohorts {
        let genotype = cohort.mesh.finite_allocation.unwrap().genotype;
        for (index, value) in genotype.0.iter().enumerate() {
            mean[index] += *value * cohort.count as f64 / count;
        }
    }
    mean
}

fn genotype_frequencies(cohorts: &[Cohort]) -> BTreeMap<String, u64> {
    let mut frequencies = BTreeMap::new();
    for cohort in cohorts {
        *frequencies
            .entry(genotype_key(
                cohort.mesh.finite_allocation.expect("allocation").genotype,
            ))
            .or_default() += cohort.count;
    }
    frequencies
}

fn snapshot(cohorts: &[Cohort], world: &OpenMedium, step: usize) -> Value {
    let mean = mean_genotype(cohorts);
    json!({
        "step": step,
        "population": population_count(cohorts),
        "cohorts": cohorts.len(),
        "maximum_generation": cohorts.iter().map(|c| c.generation).max().unwrap_or(0),
        "genotype_frequencies": genotype_frequencies(cohorts),
        "mean_genotype": mean,
        "processing_activation": mean[0] + mean[1],
        "repair": mean[2],
        "growth_reserve_only_dormant": mean[3],
        "world_n": world.n_mass,
        "world_f": world.f_mass,
        "organism_n": organism_amount(cohorts, 'n'),
        "organism_f": organism_amount(cohorts, 'f'),
        "organism_a": organism_amount(cohorts, 'a'),
        "organism_w": organism_amount(cohorts, 'w'),
        "organism_structural_mass": cohorts
            .iter()
            .map(|cohort| cohort.mesh.total_structural_mass() * cohort.count as f64)
            .sum::<f64>(),
        "all_simple": cohorts.iter().all(|c| polygon_simple(&c.mesh.vertices)),
    })
}

fn deterministic_state_digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("serializable observer state");
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn prefix_state(
    cohorts: &[Cohort],
    world: &OpenMedium,
    ledger: &CampaignLedger,
    step: usize,
) -> Value {
    json!({
        "step": step,
        "snapshot": snapshot(cohorts, world, step),
        "cohort_state_digest": deterministic_state_digest(&cohorts),
        "plasticity_state_digest": deterministic_state_digest(
            &cohorts.iter().map(|cohort| &cohort.plasticity).collect::<Vec<_>>()
        ),
        "world_state_digest": deterministic_state_digest(world),
        "ledger_state_digest": deterministic_state_digest(ledger),
        "world": world,
        "ledger": ledger,
    })
}

fn vector_norm_sum(vectors: &[[f64; 2]]) -> f64 {
    vectors.iter().map(|force| force[0].hypot(force[1])).sum()
}

/// Reuse the frozen mechanics operator with one coefficient disabled at a
/// time. This is an observer decomposition of the exact force calculation;
/// it is not a production parameter override.
fn passive_force_components(mesh: &MaterialMesh, params: &MechParams) -> Value {
    let mut stretch_params = *params;
    stretch_params.kappa_b = 0.0;
    stretch_params.k_pi = 0.0;
    let mut bending_params = *params;
    bending_params.k_s = 0.0;
    bending_params.k_pi = 0.0;
    let mut pressure_params = *params;
    pressure_params.k_s = 0.0;
    pressure_params.kappa_b = 0.0;
    let stretch = compute_forces(mesh, &stretch_params);
    let bending = compute_forces(mesh, &bending_params);
    let pressure = compute_forces(mesh, &pressure_params);
    let total = compute_forces(mesh, params);
    json!({
        "stretch_force_norm_sum": vector_norm_sum(&stretch),
        "bending_force_norm_sum": vector_norm_sum(&bending),
        "pressure_force_norm_sum": vector_norm_sum(&pressure),
        "passive_force_norm_sum": vector_norm_sum(&total),
        "pressure_values": (0..mesh.n()).map(|edge| local_pressure(mesh, edge)).collect::<Vec<_>>(),
    })
}

fn low_order_shape_modes(mesh: &MaterialMesh) -> Value {
    let center = mesh.centroid();
    let radii = mesh
        .vertices
        .iter()
        .map(|point| {
            let dx = point[0] - center[0];
            let dy = point[1] - center[1];
            (dx.hypot(dy), dy.atan2(dx))
        })
        .collect::<Vec<_>>();
    let mean_radius =
        radii.iter().map(|(radius, _)| *radius).sum::<f64>() / radii.len().max(1) as f64;
    let modes = (2..=4)
        .map(|mode| {
            let (cosine, sine) = radii.iter().fold((0.0, 0.0), |(c, s), (radius, angle)| {
                let phase = *angle * mode as f64;
                (c + radius * phase.cos(), s + radius * phase.sin())
            });
            let normalization = radii.len().max(1) as f64 * mean_radius.max(1e-300);
            json!({
                "mode": mode,
                "cosine_coefficient": cosine / radii.len().max(1) as f64,
                "sine_coefficient": sine / radii.len().max(1) as f64,
                "normalized_amplitude": cosine.hypot(sine) / normalization,
            })
        })
        .collect::<Vec<_>>();
    json!({"centroid": center, "mean_radius": mean_radius, "modes": modes})
}

fn deterministic_zero_mean_mode(mesh: &mut MaterialMesh, mode: usize) -> f64 {
    let center = mesh.centroid();
    let scale = mesh
        .vertices
        .iter()
        .map(|point| (point[0] - center[0]).hypot(point[1] - center[1]))
        .sum::<f64>()
        / mesh.n().max(1) as f64;
    let mut displacement = Vec::with_capacity(mesh.n());
    for point in &mesh.vertices {
        let dx = point[0] - center[0];
        let dy = point[1] - center[1];
        let radius = dx.hypot(dy).max(1e-300);
        let phase = (mode as f64) * dy.atan2(dx);
        displacement.push([
            STABILITY_PROBE_EPSILON * scale * phase.cos() * dx / radius,
            STABILITY_PROBE_EPSILON * scale * phase.cos() * dy / radius,
        ]);
    }
    let mean = displacement.iter().fold([0.0, 0.0], |sum, delta| {
        [sum[0] + delta[0], sum[1] + delta[1]]
    });
    let mean = [
        mean[0] / displacement.len().max(1) as f64,
        mean[1] / displacement.len().max(1) as f64,
    ];
    let mut initial_rms = 0.0;
    for (point, delta) in mesh.vertices.iter_mut().zip(displacement) {
        let centered = [delta[0] - mean[0], delta[1] - mean[1]];
        point[0] += centered[0];
        point[1] += centered[1];
        initial_rms += centered[0] * centered[0] + centered[1] * centered[1];
    }
    (initial_rms / mesh.n().max(1) as f64).sqrt()
}

fn vertex_rms_difference(left: &MaterialMesh, right: &MaterialMesh) -> Option<f64> {
    (left.n() == right.n()).then(|| {
        (left
            .vertices
            .iter()
            .zip(&right.vertices)
            .map(|(a, b)| {
                let dx = a[0] - b[0];
                let dy = a[1] - b[1];
                dx * dx + dy * dy
            })
            .sum::<f64>()
            / left.n().max(1) as f64)
            .sqrt()
    })
}

/// Run the frozen passive local mechanics on a real accepted trajectory
/// snapshot and on a one-shot, zero-mean Fourier perturbation. The perturbed
/// clone is never returned to the production trajectory.
fn frozen_stability_assay(mesh: &MaterialMesh, mechanics: &MechParams) -> Value {
    let modes = (2..=4)
        .map(|mode| {
            let mut baseline = mesh.clone();
            let mut perturbed = mesh.clone();
            let initial_rms = deterministic_zero_mean_mode(&mut perturbed, mode);
            let mut trace = Vec::new();
            let mut valid = polygon_simple(&perturbed.vertices);
            for step in 0..STABILITY_PROBE_STEPS {
                let baseline_ok =
                    mechanics_step_with_local_self_contact(&mut baseline, mechanics).is_some();
                let perturbed_ok =
                    mechanics_step_with_local_self_contact(&mut perturbed, mechanics).is_some();
                valid &= baseline_ok
                    && perturbed_ok
                    && polygon_simple(&baseline.vertices)
                    && polygon_simple(&perturbed.vertices);
                let rms = vertex_rms_difference(&baseline, &perturbed);
                trace.push(json!({
                    "step": step + 1,
                    "baseline_accepted": baseline_ok,
                    "perturbed_accepted": perturbed_ok,
                    "rms_difference": rms,
                }));
                if !valid {
                    break;
                }
            }
            let final_rms = trace.last().and_then(|row| row["rms_difference"].as_f64());
            let growth_ratio = final_rms
                .zip(Some(initial_rms))
                .map(|(final_value, initial)| final_value / initial.max(1e-300));
            let classification = growth_ratio.map(|ratio| {
                if ratio > 1.0 + STABILITY_CLASSIFICATION_TOLERANCE {
                    "GROWING"
                } else if ratio < 1.0 - STABILITY_CLASSIFICATION_TOLERANCE {
                    "DECAYING"
                } else {
                    "NEUTRAL"
                }
            });
            json!({
                "mode": mode,
                "initial_perturbation_rms": initial_rms,
                "final_difference_rms": final_rms,
                "growth_ratio": growth_ratio,
                "classification": classification,
                "valid": valid,
                "trace": trace,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "observer_only": true,
        "mechanics": "frozen passive local mechanics with current edge masses/rest lengths",
        "epsilon": STABILITY_PROBE_EPSILON,
        "steps": STABILITY_PROBE_STEPS,
        "classification_tolerance": STABILITY_CLASSIFICATION_TOLERANCE,
        "modes": modes,
    })
}

fn architecture_geometry_record(
    before: &MaterialMesh,
    after: &MaterialMesh,
    diagnostic: &Value,
    fission: &FissionParams,
    mechanics: &MechParams,
    transport: Option<[f64; 2]>,
    expression: Option<ExpressionAccounting>,
    reactions: &chemistry_core::mesh_reactions::ReactionLedger,
    growth: &chemistry_core::mesh_growth::GrowthLedger,
    active_a: f64,
    active_w: f64,
    step: usize,
) -> Value {
    let perimeter = after.perimeter();
    let area = after.area().max(1e-300);
    let mature_rest_perimeter = (0..after.n())
        .map(|edge| after.rest_length(edge))
        .sum::<f64>();
    let material_equivalent_perimeter = after
        .edges
        .iter()
        .map(|edge| edge.m.max(0.0) / after.rho_s.max(1e-15))
        .sum::<f64>();
    let rest_lengths = (0..after.n())
        .map(|edge| after.rest_length(edge))
        .collect::<Vec<_>>();
    let rest_length_delta = (before.n() == after.n()).then(|| {
        (0..after.n())
            .map(|edge| after.rest_length(edge) - before.rest_length(edge))
            .collect::<Vec<_>>()
    });
    let passive = passive_force_components(before, mechanics);
    let active_requested = diagnostic["requested_inward_normal_forces"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| Some([value[0].as_f64()?, value[1].as_f64()?]))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let active_funded = diagnostic["funded_inward_normal_forces"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| Some([value[0].as_f64()?, value[1].as_f64()?]))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let passive_forces = compute_forces(before, mechanics);
    let combined_forces = passive_forces
        .iter()
        .zip(&active_funded)
        .map(|(passive, active)| [passive[0] + active[0], passive[1] + active[1]])
        .collect::<Vec<_>>();
    let mut no_contact = before.clone();
    let no_contact_accepted = combined_forces.len() == before.n()
        && mechanics_step_with_external_forces(&mut no_contact, mechanics, &combined_forces);
    let no_contact_displacement = no_contact_accepted.then(|| {
        before
            .vertices
            .iter()
            .zip(&no_contact.vertices)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .sum::<f64>()
    });
    let active_requested_norm = vector_norm_sum(&active_requested);
    let active_funded_norm = vector_norm_sum(&active_funded);
    json!({
        "step": step,
        "accepted_snapshot": true,
        "vertices": after.vertices,
        "perimeter": perimeter,
        "area": area,
        "shape_factor_p_squared_over_4pi_area": perimeter * perimeter / (4.0 * std::f64::consts::PI * area),
        "reduced_area_inverse_shape_factor": 4.0 * std::f64::consts::PI * area / perimeter.max(1e-300).powi(2),
        "mature_rest_perimeter": mature_rest_perimeter,
        "material_equivalent_perimeter": material_equivalent_perimeter,
        "perimeter_minus_mature_rest_perimeter": perimeter - mature_rest_perimeter,
        "young_structural_mass": after.total_young_structural_mass(),
        "mature_structural_mass": after.total_structural_mass() - after.total_young_structural_mass(),
        "total_structural_mass": after.total_structural_mass(),
        "edge_material": (0..after.n()).map(|edge| json!({
            "edge": edge,
            "m": after.edges[edge].m,
            "m_young": after.young_structural_mass(edge),
            "m_mature": after.mature_structural_mass(edge),
            "geometric_length": after.edge_length(edge),
            "rest_length": after.rest_length(edge),
            "rest_length_delta": rest_length_delta.as_ref().and_then(|values| values.get(edge).copied()),
            "strain": after.strain(edge),
            "mature_fraction": after.mature_structural_fraction(edge),
        })).collect::<Vec<_>>(),
        "rest_lengths": rest_lengths,
        "rest_length_delta": rest_length_delta,
        "shape_modes": low_order_shape_modes(after),
        "curvature": r10_closure::r10_curvature_summary(after),
        "segment_geometry": r10_closure::r10_segment_geometry_summary(after, fission),
        "passive_force_components_at_mechanics_input": passive,
        "active_force": {
            "requested_vector_norm_sum": active_requested_norm,
            "funded_vector_norm_sum": active_funded_norm,
            "requested_active_a": diagnostic["requested_active_a"],
            "funded_active_a": diagnostic["funded_active_a"],
            "funding_ratio": diagnostic["funding_ratio"],
        },
        "contact_intervention": {
            "no_contact_composed_mechanics_accepted": no_contact_accepted,
            "no_contact_displacement_norm": no_contact_displacement,
            "contact_or_projection_observed": !no_contact_accepted,
        },
        "boundary": {
            "organism_n_f_before_mechanics": transport,
            "mechanics_exterior": [after.exterior.n, after.exterior.f, after.exterior.c, after.exterior.a, after.exterior.w],
        },
        "fluxes": {
            "transport_delta_n_f": transport,
            "expression": expression.map(|value| json!({
                "structural_consumed": value.structural_consumed,
                "catalyst_precursor_a": value.catalyst_precursor_a,
                "activation_consumed": value.activation_consumed,
                "maintenance_consumed": value.maintenance_consumed,
                "turnover_waste": value.turnover_waste,
            })),
            "reaction_n_consumed": reactions.n_consumed,
            "reaction_f_consumed": reactions.f_consumed,
            "a_produced": reactions.a_produced,
            "w_produced": reactions.w_produced,
            "activation_equivalent_closure_residual": reactions.activation_equivalent_closure_residual,
            "growth_material": growth.m_grown,
            "growth_a_consumed": growth.a_consumed_growth,
            "growth_w_produced": growth.w_from_growth,
            "active_w_produced": active_w,
            "active_a_spent": active_a,
        },
        "interior": {
            "n": after.interior.n,
            "f": after.interior.f,
            "a": after.interior.a,
            "r": after.interior.r,
            "w": after.interior.w,
        },
        "raw_diagnostic": diagnostic,
        "stability_assay": frozen_stability_assay(after, mechanics),
    })
}

fn terminal_cohort_diagnostics(cohorts: &[Cohort], fission: &FissionParams) -> Vec<Value> {
    cohorts
        .iter()
        .map(|cohort| {
            let audits = segment_apposition_stress_audit(&cohort.mesh, fission);
            let signed_stress_pairs = audits
                .iter()
                .filter(|audit| audit.signed_magnitude_predicate)
                .count();
            let terminal_fission_ready = PlanarRingTopology::from_mesh(&cohort.mesh)
                .and_then(|topology| topology.try_local_scission(&cohort.mesh, fission))
                .or_else(|| try_local_segment_fission(&cohort.mesh, fission))
                .is_some();
            let mass_eligible =
                cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass;
            let deepest_blocker = if !mass_eligible {
                "INSUFFICIENT_GROWTH"
            } else if audits.is_empty() {
                "APPOSITION_ABSENT"
            } else if signed_stress_pairs == 0 {
                "STRESS_ABSENT"
            } else if !terminal_fission_ready {
                "DOWNSTREAM_FISSION_PREREQUISITE_OTHER"
            } else {
                "FISSION_READY_AT_TERMINAL_OBSERVER_ONLY"
            };
            json!({
                "cohort_id": cohort.id,
                "count": cohort.count,
                "generation": cohort.generation,
                "genotype": cohort.mesh.finite_allocation.expect("R10 allocation").genotype.0,
                "birth_mass": cohort.birth_mass,
                "terminal_mass": cohort.mesh.total_structural_mass(),
                "mass_over_birth_mass": cohort.mesh.total_structural_mass() / cohort.birth_mass,
                "interior_a": cohort.mesh.interior.a,
                "interior_c": cohort.mesh.interior.c,
                "simple": polygon_simple(&cohort.mesh.vertices),
                "runtime_valid": cohort.mesh.physical_runtime_valid(),
                "lifecycle_valid": cohort.mesh.lifecycle_invariants_hold(),
                "in_range_segment_pairs": audits.len(),
                "signed_stress_qualified_pairs": signed_stress_pairs,
                "minimum_distance_over_range": audits.first().map(|audit| audit.distance / audit.range),
                "maximum_compressive_effective_strain": audits.iter().map(|audit| audit.maximum_compressive_effective_strain_magnitude).fold(0.0_f64, f64::max),
                "terminal_fission_ready_observer_only": terminal_fission_ready,
                "deepest_physical_blocker": deepest_blocker,
                "plasticity_state_digest": deterministic_state_digest(&cohort.plasticity),
            })
        })
        .collect()
}

/// Re-evaluate the real R10 fission dispatch without mutating the parent.
/// This replaces the opaque `NO_VALID_PHYSICAL_PROPOSAL` observer record with
/// the production predicates that can actually reject a mass-eligible attempt.
/// The nearest in-range candidate carries all four incident edge states; the
/// aggregate counts preserve the remaining candidate population without
/// duplicating large geometry payloads for every attempt.
fn r10_fission_attempt_detail(mesh: &MaterialMesh, fission: &FissionParams) -> Value {
    let audits = segment_apposition_stress_audit(mesh, fission);
    let signed_stress_count = audits
        .iter()
        .filter(|audit| audit.signed_magnitude_predicate)
        .count();
    let proposal = try_local_fission(mesh, fission).or_else(|| {
        PlanarRingTopology::from_mesh(mesh)
            .and_then(|topology| topology.try_local_scission(mesh, fission))
            .or_else(|| try_local_segment_fission(mesh, fission))
    });
    let proposal_validation = proposal.as_ref().map(|(daughter_a, daughter_b, event)| {
        json!({
            "partition_ok": event.partition.ok,
            "parent_simple": polygon_simple(&mesh.vertices),
            "daughter_a_simple": polygon_simple(&daughter_a.vertices),
            "daughter_b_simple": polygon_simple(&daughter_b.vertices),
            "daughter_a_runtime_valid": daughter_a.physical_runtime_valid(),
            "daughter_b_runtime_valid": daughter_b.physical_runtime_valid(),
            "daughter_a_lifecycle_valid": daughter_a.lifecycle_invariants_hold(),
            "daughter_b_lifecycle_valid": daughter_b.lifecycle_invariants_hold(),
            "partition": event.partition,
        })
    });
    let proposal_valid = proposal_validation.as_ref().is_some_and(|value| {
        value["partition_ok"] == true
            && value["parent_simple"] == true
            && value["daughter_a_simple"] == true
            && value["daughter_b_simple"] == true
            && value["daughter_a_runtime_valid"] == true
            && value["daughter_b_runtime_valid"] == true
            && value["daughter_a_lifecycle_valid"] == true
            && value["daughter_b_lifecycle_valid"] == true
    });
    let failure_class = if !mesh.can_advance_physics() || !polygon_simple(&mesh.vertices) {
        "FISSION_CANDIDATE_TOPOLOGY_INVALID"
    } else if audits.is_empty() {
        "APPOSITION_OUTSIDE_LOCAL_RANGE"
    } else if signed_stress_count == 0 {
        "APPOSITION_PRESENT_STRESS_FALSE"
    } else if proposal.is_some() && !proposal_valid {
        "FISSION_CANDIDATE_TOPOLOGY_INVALID"
    } else if proposal.is_none() {
        "OTHER_WITH_EVIDENCE"
    } else {
        "PROPOSAL_AVAILABLE"
    };
    let nearest = audits.first().map(|audit| {
        json!({
            "edge_i": audit.edge_i,
            "edge_j": audit.edge_j,
            "segment_i_fraction": audit.segment_i_fraction,
            "segment_j_fraction": audit.segment_j_fraction,
            "distance": audit.distance,
            "range": audit.range,
            "distance_over_range": audit.distance / audit.range.max(1e-300),
            "ring_separation": audit.ring_separation,
            "incident_edges": audit.incident_edges,
            "raw_strains": audit.raw_strains,
            "mature_fractions": audit.mature_fractions,
            "effective_signed_strains": audit.effective_signed_strains,
            "maximum_tensile_effective_strain": audit.maximum_tensile_effective_strain,
            "maximum_compressive_effective_strain_magnitude": audit.maximum_compressive_effective_strain_magnitude,
            "legacy_tensile_only_predicate": audit.legacy_tensile_only_predicate,
            "signed_magnitude_predicate": audit.signed_magnitude_predicate,
        })
    });
    let closure_a_required = nearest
        .as_ref()
        .and_then(|candidate| candidate["distance"].as_f64())
        .map(|distance| 2.0 * mesh.rho_s * distance / 0.9);
    json!({
        "mass_eligible": true,
        "failure_class": failure_class,
        "structural_mass": mesh.total_structural_mass(),
        "absolute_a": mesh.interior.a.max(0.0) * mesh.area().max(1e-300),
        "in_range_candidate_count": audits.len(),
        "signed_stress_qualified_candidate_count": signed_stress_count,
        "in_range_pairs": audits.len(),
        "signed_stress_qualified_pairs": signed_stress_count,
        "nearest_candidate": nearest,
        "closure_a_required_for_two_v4_edges": closure_a_required,
        "proposal_available": proposal.is_some(),
        "proposal_valid": proposal_valid,
        "proposal_validation": proposal_validation,
    })
}

fn split_cohort(
    cohort: Cohort,
    mutation_enabled: bool,
    campaign_seed: u64,
    step: usize,
    next_id: &mut u64,
    allocation: &AllocationParams,
    fission: &FissionParams,
    ledger: &mut CampaignLedger,
) -> Result<Vec<Cohort>, Cohort> {
    let Some((daughter_a, daughter_b, event)) = try_local_fission(&cohort.mesh, fission) else {
        return Err(cohort);
    };
    if !event.partition.ok {
        ledger.partition_failures += cohort.count;
        return Err(cohort);
    }
    if !polygon_simple(&cohort.mesh.vertices)
        || !polygon_simple(&daughter_a.vertices)
        || !polygon_simple(&daughter_b.vertices)
    {
        ledger.invalid_geometry_events += cohort.count;
        return Err(cohort);
    }
    ledger.physical_fissions += cohort.count;
    ledger.valid_simple_fissions += cohort.count;
    *ledger
        .fissions_by_parent_genotype
        .entry(genotype_key(
            cohort.mesh.finite_allocation.unwrap().genotype,
        ))
        .or_default() += cohort.count;

    let mutation_params = if mutation_enabled {
        *allocation
    } else {
        AllocationParams {
            mutation_probability: 0.0,
            ..*allocation
        }
    };
    let children = [daughter_a, daughter_b];
    let mut groups: BTreeMap<(usize, [u64; 4]), (MaterialMesh, u64)> = BTreeMap::new();
    for ordinal in 0..cohort.count {
        for (side, child_template) in children.iter().enumerate() {
            let parent = child_template.finite_allocation.unwrap().genotype;
            let seed = mutation_seed(campaign_seed, step, cohort.id, ordinal, side as u64);
            let mutation = mutate_allocation_at_reproduction(parent, &mutation_params, seed);
            ledger.mutation_opportunities += 1;
            if mutation.mutated {
                ledger.mutations += 1;
                ledger.mutation_events.push(json!({
                    "step": step,
                    "generation": cohort.generation + 1,
                    "seed": seed,
                    "parent": mutation.parent.0,
                    "offspring": mutation.offspring.0,
                    "source_index": mutation.source_index,
                    "target_index": mutation.target_index,
                    "transferred": mutation.transferred,
                }));
            }
            let key = (side, mutation.offspring.0.map(f64::to_bits));
            groups
                .entry(key)
                .and_modify(|(_, count)| *count += 1)
                .or_insert_with(|| {
                    let mut mesh = child_template.clone();
                    mesh.finite_allocation.as_mut().unwrap().genotype = mutation.offspring;
                    (mesh, 1)
                });
        }
    }
    let mut result = Vec::new();
    for (_, (mesh, count)) in groups {
        let id = *next_id;
        *next_id += 1;
        result.push(Cohort {
            birth_mass: mesh.total_structural_mass(),
            mesh,
            plasticity: None,
            count,
            generation: cohort.generation + 1,
            id,
        });
    }
    Ok(result)
}

fn invalidated_material_to_terminal(cohort: &Cohort, world: &mut OpenMedium) {
    world.ledger.invalidated_n_terminal +=
        cohort.mesh.interior.n.max(0.0) * cohort.mesh.area() * cohort.count as f64;
    world.ledger.invalidated_f_terminal +=
        cohort.mesh.interior.f.max(0.0) * cohort.mesh.area() * cohort.count as f64;
    world.ledger.invalidated_structural_terminal +=
        cohort.mesh.total_structural_mass() * cohort.count as f64;
}

fn dead_material_to_sink(cohort: &Cohort, world: &mut OpenMedium) {
    world.ledger.physical_death_n_sink +=
        cohort.mesh.interior.n.max(0.0) * cohort.mesh.area() * cohort.count as f64;
    world.ledger.physical_death_f_sink +=
        cohort.mesh.interior.f.max(0.0) * cohort.mesh.area() * cohort.count as f64;
    world.ledger.physical_death_structural_sink +=
        cohort.mesh.total_structural_mass() * cohort.count as f64;
}

fn lifecycle_event(
    ledger: &mut CampaignLedger,
    phase: &str,
    step: usize,
    cohort: &Cohort,
    outcome: &str,
    detail: Value,
) {
    ledger.lifecycle_events.push(json!({
        "phase": phase,
        "step": step,
        "cohort_id": cohort.id,
        "count": cohort.count,
        "generation": cohort.generation,
        "genotype": cohort.mesh.finite_allocation.map(|state| state.genotype.0),
        "outcome": outcome,
        "detail": detail,
    }));
}

fn advance_phase(
    cohorts: &mut Vec<Cohort>,
    world: &mut OpenMedium,
    environment: Environment,
    phase_index: usize,
    mutation_enabled: bool,
    campaign_seed: u64,
    next_id: &mut u64,
    ledger: &mut CampaignLedger,
    trajectory: &mut Vec<Value>,
) {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for phase_step in 0..PHASE_STEPS {
        if cohorts.is_empty() {
            break;
        }
        let step = phase_index * PHASE_STEPS + phase_step + 1;
        world.add_fixed_inflow(environment, phase_step, mechanics.dt);
        if environment == Environment::Damage {
            for cohort in cohorts.iter_mut() {
                apply_damage(cohort, phase_step, world);
            }
        }
        let mut retained = Vec::new();
        for mut cohort in std::mem::take(cohorts) {
            match expression_step(&mut cohort.mesh, &allocation, mechanics.dt) {
                Ok(expression) => {
                    let count = cohort.count as f64;
                    ledger.expression_material += expression.material_consumed * count;
                    ledger.expression_activation +=
                        (expression.activation_consumed + expression.maintenance_consumed) * count;
                    retained.push(cohort);
                }
                Err(_) => {
                    ledger.expression_failures += cohort.count;
                    ledger.runtime_invalidations += cohort.count;
                    invalidated_material_to_terminal(&cohort, world);
                }
            }
        }
        *cohorts = retained;
        world.exchange(cohorts, &transport, mechanics.dt);

        let mut survivors = Vec::new();
        for mut cohort in std::mem::take(cohorts) {
            let count = cohort.count as f64;
            let reactions = reactions_step(&mut cohort.mesh, &reaction, mechanics.dt, true, true);
            ledger.reaction_n_consumed += reactions.n_consumed * count;
            ledger.reaction_f_consumed += reactions.f_consumed * count;
            ledger.a_produced += reactions.a_produced * count;
            ledger.w_produced += reactions.w_produced * count;
            let grown = growth_step(&mut cohort.mesh, &reaction, &growth, mechanics.dt);
            ledger.growth_material += grown.m_grown * count;
            let valid =
                mechanics_step_with_local_self_contact(&mut cohort.mesh, &mechanics).is_some();
            if !valid {
                ledger.runtime_invalidations += cohort.count;
                ledger.invalid_geometry_events += cohort.count;
                invalidated_material_to_terminal(&cohort, world);
                continue;
            }
            let _ = remesh(&mut cohort.mesh);
            if step % 10 == 0 {
                let _ = topology_step(&mut cohort.mesh, &fission);
            }
            if !polygon_simple(&cohort.mesh.vertices) {
                ledger.runtime_invalidations += cohort.count;
                ledger.invalid_geometry_events += cohort.count;
                invalidated_material_to_terminal(&cohort, world);
                continue;
            }
            if !cohort.mesh.observer_viable() {
                ledger.physical_deaths += cohort.count;
                dead_material_to_sink(&cohort, world);
                continue;
            }
            let eligible =
                cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass && step % 25 == 0;
            if eligible {
                match split_cohort(
                    cohort,
                    mutation_enabled,
                    campaign_seed,
                    step,
                    next_id,
                    &allocation,
                    &fission,
                    ledger,
                ) {
                    Ok(children) => survivors.extend(children),
                    Err(parent) => survivors.push(parent),
                }
            } else {
                survivors.push(cohort);
            }
        }
        *cohorts = survivors;
        if phase_step == 0 || (phase_step + 1) % 250 == 0 {
            trajectory.push(snapshot(cohorts, world, step));
        }
    }
}

fn initial_population(
    template: &MaterialMesh,
    original_birth_mass: f64,
    mutation_enabled: bool,
    campaign_seed: u64,
    founder_count: u64,
    ledger: &mut CampaignLedger,
) -> Vec<Cohort> {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let cohort = Cohort {
        mesh: parent,
        plasticity: None,
        count: founder_count,
        generation: 0,
        birth_mass: original_birth_mass,
        id: 1,
    };
    let mut next_id = 2;
    split_cohort(
        cohort,
        mutation_enabled,
        campaign_seed,
        0,
        &mut next_id,
        &allocation,
        &fission,
        ledger,
    )
    .expect("sealed geometry-valid parent must fission")
}

fn campaign(
    template: &MaterialMesh,
    original_birth_mass: f64,
    template_step: usize,
    sequence: &[Environment],
    mutation_enabled: bool,
    replicate: u64,
) -> Value {
    let campaign_seed = splitmix64(
        replicate
            ^ if mutation_enabled {
                0x6a09_e667_f3bc_c909
            } else {
                0xbb67_ae85_84ca_a73b
            }
            ^ sequence.iter().fold(0_u64, |state, env| {
                state.rotate_left(9)
                    ^ match env {
                        Environment::Resource => 0x11,
                        Environment::Damage => 0x22,
                    }
            }),
    );
    let mut ledger = CampaignLedger::default();
    let mut cohorts = initial_population(
        template,
        original_birth_mass,
        mutation_enabled,
        campaign_seed,
        FOUNDER_MULTIPLICITY,
        &mut ledger,
    );
    let mut next_id = 10_000;
    let mut world = OpenMedium::new(sequence[0], FOUNDER_MULTIPLICITY as f64);
    let initial_organism_n = organism_amount(&cohorts, 'n');
    let initial_organism_f = organism_amount(&cohorts, 'f');
    let initial = snapshot(&cohorts, &world, 0);
    let mut trajectory = vec![initial.clone()];
    for (phase, environment) in sequence.iter().copied().enumerate() {
        advance_phase(
            &mut cohorts,
            &mut world,
            environment,
            phase,
            mutation_enabled,
            campaign_seed,
            &mut next_id,
            &mut ledger,
            &mut trajectory,
        );
    }
    let terminal_step = sequence.len() * PHASE_STEPS;
    let terminal = snapshot(&cohorts, &world, terminal_step);
    let terminal_organism_n = organism_amount(&cohorts, 'n');
    let terminal_organism_f = organism_amount(&cohorts, 'f');
    let n_closure = (world.ledger.initial_n + world.ledger.inflow_n + initial_organism_n
        - world.n_mass
        - terminal_organism_n
        - ledger.reaction_n_consumed
        - world.ledger.physical_death_n_sink
        - world.ledger.invalidated_n_terminal)
        .abs();
    let f_closure = (world.ledger.initial_f + world.ledger.inflow_f + initial_organism_f
        - world.f_mass
        - terminal_organism_f
        - ledger.reaction_f_consumed
        - world.ledger.physical_death_f_sink
        - world.ledger.invalidated_f_terminal)
        .abs();
    json!({
        "replicate": replicate,
        "mutation_enabled": mutation_enabled,
        "campaign_seed": campaign_seed,
        "environment_sequence": sequence.iter().map(|environment| environment.label()).collect::<Vec<_>>(),
        "phase_steps": PHASE_STEPS,
        "founder_multiplicity": FOUNDER_MULTIPLICITY,
        "template_fission_step": template_step,
        "fitness_function": null,
        "breeder_selection": false,
        "population_cap": null,
        "resource_feedback": false,
        "exchangeability_compression": true,
        "initial": initial,
        "trajectory": trajectory,
        "terminal": terminal,
        "world": world,
        "ledger": ledger,
        "n_closure_residual": n_closure,
        "f_closure_residual": f_closure,
    })
}

fn compression_parity(template: &MaterialMesh, original_birth_mass: f64) -> Value {
    let mut compressed_ledger = CampaignLedger::default();
    let mut compressed = initial_population(
        template,
        original_birth_mass,
        false,
        101,
        2,
        &mut compressed_ledger,
    );
    let mut explicit = Vec::new();
    let mut explicit_ledger = CampaignLedger::default();
    for ordinal in 0..2_u64 {
        let mut ledger = CampaignLedger::default();
        explicit.extend(initial_population(
            template,
            original_birth_mass,
            false,
            101 ^ ordinal.rotate_left(3),
            1,
            &mut ledger,
        ));
        explicit_ledger.mutation_opportunities += ledger.mutation_opportunities;
    }
    let transport = TransportParams::default();
    let dt = MechParams::default().dt;
    let mut compressed_world = OpenMedium::new(Environment::Resource, FOUNDER_MULTIPLICITY as f64);
    let mut explicit_world = compressed_world.clone();
    compressed_world.exchange(&mut compressed, &transport, dt);
    explicit_world.exchange(&mut explicit, &transport, dt);
    let transport_residual = (compressed_world.n_mass - explicit_world.n_mass)
        .abs()
        .max((compressed_world.f_mass - explicit_world.f_mass).abs())
        .max((organism_amount(&compressed, 'n') - organism_amount(&explicit, 'n')).abs())
        .max((organism_amount(&compressed, 'f') - organism_amount(&explicit, 'f')).abs());
    json!({
        "scope": "initial geometry-valid fission, mutation-off inheritance, and one finite shared-medium exchange",
        "compressed_population": population_count(&compressed),
        "explicit_population": population_count(&explicit),
        "compressed_opportunities": compressed_ledger.mutation_opportunities,
        "explicit_opportunities": explicit_ledger.mutation_opportunities,
        "transport_residual": transport_residual,
        "pass": population_count(&compressed) == population_count(&explicit)
            && compressed_ledger.mutation_opportunities == explicit_ledger.mutation_opportunities
            && transport_residual <= 1e-10,
    })
}

fn one_frozen_step(mut mesh: MaterialMesh, expression: bool) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let initial_physical_valid = mesh.physical_runtime_valid();
    let initial_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let initial_simple = polygon_simple(&mesh.vertices);
    let (expression_ok, expression_material) = if expression {
        match expression_step(&mut mesh, &allocation, mechanics.dt) {
            Ok(ledger) => (true, ledger.material_consumed),
            Err(_) => (false, 0.0),
        }
    } else {
        (true, 0.0)
    };
    let post_expression_physical_valid = mesh.physical_runtime_valid();
    let post_expression_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let minimum_m_minus_young_after_expression = mesh
        .edges
        .iter()
        .map(|edge| edge.m - edge.m_young)
        .fold(f64::INFINITY, f64::min);
    let _ = transport_step(&mut mesh, &transport, mechanics.dt);
    let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
    let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
    let pre_contact_physical_valid = mesh.physical_runtime_valid();
    let pre_contact_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let pre_contact_simple = polygon_simple(&mesh.vertices);
    let contact_accepted = mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some();
    json!({
        "expression_enabled": expression,
        "expression_ok": expression_ok,
        "expression_material_consumed": expression_material,
        "initial_physical_runtime_valid": initial_physical_valid,
        "initial_lifecycle_invariants_hold": initial_lifecycle_valid,
        "initial_polygon_simple": initial_simple,
        "post_expression_physical_runtime_valid": post_expression_physical_valid,
        "post_expression_lifecycle_invariants_hold": post_expression_lifecycle_valid,
        "minimum_m_minus_young_after_expression": minimum_m_minus_young_after_expression,
        "pre_contact_physical_runtime_valid": pre_contact_physical_valid,
        "pre_contact_lifecycle_invariants_hold": pre_contact_lifecycle_valid,
        "pre_contact_polygon_simple": pre_contact_simple,
        "contact_accepted": contact_accepted,
        "polygon_simple_after": polygon_simple(&mesh.vertices),
        "observer_viable_after": mesh.observer_viable(),
    })
}

fn exact_daughter_root_cause(mesh: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let mut repaired = mesh.clone();
    let material_before = repaired.total_structural_mass();
    let before = repaired
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            json!({
                "edge": index,
                "m": edge.m,
                "m_young": edge.m_young,
                "m_mature": edge.m - edge.m_young,
                "young_fraction": edge.m_young / edge.m.max(f64::MIN_POSITIVE),
                "rest_length": repaired.rest_length(index),
                "edge_length": repaired.edge_length(index),
                "strain": repaired.strain(index),
            })
        })
        .collect::<Vec<_>>();
    let ledger = expression_step(&mut repaired, &allocation, MechParams::default().dt)
        .expect("exact daughter expression");
    let fraction_left = 1.0 - ledger.material_consumed / material_before;
    let legacy_violation_edges = mesh
        .edges
        .iter()
        .enumerate()
        .filter_map(|(index, edge)| {
            let legacy_m = edge.m * fraction_left;
            (edge.m_young > legacy_m + 1e-12).then_some(json!({
                "edge": index,
                "legacy_m_after": legacy_m,
                "legacy_m_young_after": edge.m_young,
                "excess": edge.m_young - legacy_m,
            }))
        })
        .collect::<Vec<_>>();
    let after = repaired
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            json!({
                "edge": index,
                "m": edge.m,
                "m_young": edge.m_young,
                "m_mature": edge.m - edge.m_young,
                "young_fraction": edge.m_young / edge.m.max(f64::MIN_POSITIVE),
                "rest_length": repaired.rest_length(index),
                "edge_length": repaired.edge_length(index),
                "strain": repaired.strain(index),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "contract_version": format!("{:?}", mesh.contract_version),
        "maturation_coupled": mesh.is_maturation_coupled(),
        "before": before,
        "material_consumed": ledger.material_consumed,
        "fraction_left": fraction_left,
        "legacy_counterfactual_violation_edges": legacy_violation_edges,
        "legacy_failing_predicate": "M_YOUNG_EXCEEDS_M_AFTER_D096_STRUCTURAL_DRAW",
        "repaired_after": after,
        "repaired_lifecycle_invariants_hold": repaired.lifecycle_invariants_hold(),
        "repaired_physical_runtime_valid": repaired.physical_runtime_valid(),
    })
}

fn v4_daughter_lifecycle(mut mesh: MaterialMesh, birth_mass: f64) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut maximum_material_residual = 0.0_f64;
    let mut maximum_activation_residual = 0.0_f64;
    let mut maximum_waste_residual = 0.0_f64;
    let mut runtime_invalidation = None;
    let mut physical_fission = None;
    let mut rejected_non_simple_fission_candidates = Vec::new();
    let mut observer_nonviable_steps = 0_u64;
    let mut completed_steps = 0_usize;
    let mut first_invalid_edges = Vec::new();
    let mut first_topology_ledger = Value::Null;
    for step in 0..3_000_usize {
        let area = mesh.area();
        let material_before = mesh.total_structural_mass();
        let a_before = mesh.interior.a * area;
        let w_before = mesh.interior.w * area;
        let expression = match expression_step(&mut mesh, &allocation, mechanics.dt) {
            Ok(ledger) => ledger,
            Err(reason) => {
                runtime_invalidation = Some(
                    json!({"step":step + 1,"phase":"expression","reason":format!("{reason:?}")}),
                );
                break;
            }
        };
        maximum_material_residual = maximum_material_residual.max(
            (material_before - mesh.total_structural_mass() - expression.material_consumed).abs(),
        );
        let activated_spent = expression.activation_consumed + expression.maintenance_consumed;
        maximum_activation_residual = maximum_activation_residual
            .max((a_before - mesh.interior.a * area - activated_spent).abs());
        maximum_waste_residual = maximum_waste_residual.max(
            (mesh.interior.w * area - w_before - activated_spent - expression.turnover_waste).abs(),
        );
        if !mesh.lifecycle_invariants_hold() || !mesh.physical_runtime_valid() {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"post_expression"}));
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_none() {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"self_contact_mechanics"}));
            break;
        }
        let _ = remesh(&mut mesh);
        if !mesh.lifecycle_invariants_hold() || !mesh.physical_runtime_valid() {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"post_remesh"}));
            first_invalid_edges = mesh
                .edges
                .iter()
                .enumerate()
                .filter(|(_, edge)| edge.m_young > edge.m + 1e-12)
                .map(|(index, edge)| json!({"edge":index,"m":edge.m,"m_young":edge.m_young,"ruptured":edge.ruptured}))
                .collect();
            break;
        }
        if step % 10 == 0 {
            let topology = topology_step(&mut mesh, &fission);
            first_topology_ledger = json!({
                "step": step + 1,
                "tension_ruptures": topology.tension_ruptures,
                "local_rebonds": topology.local_rebonds,
                "cross_bonds": topology.cross_bonds,
            });
        }
        if !mesh.lifecycle_invariants_hold()
            || !mesh.physical_runtime_valid()
            || !polygon_simple(&mesh.vertices)
        {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"post_topology"}));
            first_invalid_edges = mesh
                .edges
                .iter()
                .enumerate()
                .filter(|(_, edge)| edge.m_young > edge.m + 1e-12)
                .map(|(index, edge)| json!({"edge":index,"m":edge.m,"m_young":edge.m_young,"ruptured":edge.ruptured}))
                .collect();
            break;
        }
        if !mesh.observer_viable() {
            observer_nonviable_steps += 1;
        }
        completed_steps = step + 1;
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            if let Some((a, b, event)) = try_local_fission(&mesh, &fission) {
                let candidate = json!({
                    "step": step + 1,
                    "partition_ok": event.partition.ok,
                    "parent_simple": polygon_simple(&mesh.vertices),
                    "daughter_a_simple": polygon_simple(&a.vertices),
                    "daughter_b_simple": polygon_simple(&b.vertices),
                });
                if event.partition.ok
                    && polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                {
                    physical_fission = Some(candidate);
                    break;
                }
                rejected_non_simple_fission_candidates.push(candidate);
            }
        }
    }
    json!({
        "contract_version": format!("{:?}", mesh.contract_version),
        "target_steps": 3000,
        "completed_steps": completed_steps,
        "runtime_invalidation": runtime_invalidation,
        "first_invalid_edges": first_invalid_edges,
        "first_topology_ledger": first_topology_ledger,
        "physical_fission": physical_fission,
        "rejected_non_simple_fission_candidates": rejected_non_simple_fission_candidates,
        "observer_nonviable_steps_reported_not_removed": observer_nonviable_steps,
        "terminal_lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
        "terminal_physical_runtime_valid": mesh.physical_runtime_valid(),
        "terminal_polygon_simple": polygon_simple(&mesh.vertices),
        "maximum_structural_material_closure_residual": maximum_material_residual,
        "maximum_activation_closure_residual": maximum_activation_residual,
        "maximum_waste_closure_residual": maximum_waste_residual,
    })
}

fn v4_counterpart(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.contract_version =
        chemistry_core::material_mesh::MeshContractVersion::MaturationCoupledV4;
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("V4 counterpart fission");
    assert!(event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices));
    let a_birth = a.total_structural_mass();
    let b_birth = b.total_structural_mass();
    json!({
        "scope": "same frozen parent state and fission, with the production V4 contract selected before fission; qualification of the authorized repair, not an exact R2 replay",
        "daughter_a_first_step": one_frozen_step(a.clone(), true),
        "daughter_b_first_step": one_frozen_step(b.clone(), true),
        "daughter_a_lifecycle": v4_daughter_lifecycle(a, a_birth),
        "daughter_b_lifecycle": v4_daughter_lifecycle(b, b_birth),
    })
}

fn newborn_first_step_attribution(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("template fission");
    assert!(event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices));
    json!({
        "daughter_a": {
            "root_cause": exact_daughter_root_cause(&a),
            "expression_off": one_frozen_step(a.clone(), false),
            "expression_on": one_frozen_step(a, true),
        },
        "daughter_b": {
            "root_cause": exact_daughter_root_cause(&b),
            "expression_off": one_frozen_step(b.clone(), false),
            "expression_on": one_frozen_step(b, true),
        },
        "interpretation_boundary": "one frozen post-birth step; observer counterfactual only",
    })
}

fn mutated_descendant_heredity_control(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let parent_genotype = AllocationGenotype::neutral();
    let (seed, mutation) = (1_u64..=100_000)
        .map(|seed| {
            (
                seed,
                mutate_allocation_at_reproduction(parent_genotype, &allocation, splitmix64(seed)),
            )
        })
        .find(|(_, event)| event.mutated)
        .expect("configured mutation must be observable in powered search");
    let mut parent = template.clone();
    parent.enable_finite_allocation(mutation.offspring, &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("template fission");
    let inherited = event.partition.ok
        && polygon_simple(&a.vertices)
        && polygon_simple(&b.vertices)
        && a.finite_allocation.unwrap().genotype == mutation.offspring
        && b.finite_allocation.unwrap().genotype == mutation.offspring;
    json!({
        "scope": "mechanistic fission-partition control; not an evolving founder population",
        "lawful_mutation_seed": splitmix64(seed),
        "mutant": mutation.offspring.0,
        "simple_parent": polygon_simple(&parent.vertices),
        "simple_daughter_a": polygon_simple(&a.vertices),
        "simple_daughter_b": polygon_simple(&b.vertices),
        "partition_ok": event.partition.ok,
        "mutated_genotype_inherited_by_both_daughters": inherited,
    })
}

fn r4_perturb(mesh: &mut MaterialMesh, kind: &str, magnitude: f64) {
    match kind {
        "rotate" => {
            let center = mesh.centroid();
            let (sine, cosine) = magnitude.sin_cos();
            for point in &mut mesh.vertices {
                let x = point[0] - center[0];
                let y = point[1] - center[1];
                point[0] = center[0] + cosine * x - sine * y;
                point[1] = center[1] + sine * x + cosine * y;
            }
        }
        "vertex" => {
            for (index, point) in mesh.vertices.iter_mut().enumerate() {
                let fraction = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                point[0] += magnitude * (fraction - 0.5);
                point[1] += magnitude * ((fraction * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + magnitude)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + magnitude)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + magnitude)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + magnitude)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + magnitude)).max(0.0);
        }
        _ => {}
    }
}

fn r4_reproduction_fixture(seed: u64, kind: &str, magnitude: f64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    r4_perturb(&mut mesh, kind, magnitude);
    r4_perturb(&mut mesh, "vertex", 0.35);
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] = center[0] + (point[0] - center[0]) * 1.25;
    }
    mesh.stamp_maturation_coupled_schema();
    mesh
}

fn r4_geometry(mesh: &MaterialMesh) -> Value {
    json!({
        "simple": polygon_simple(&mesh.vertices),
        "vertices": mesh.n(),
        "area": mesh.area(),
        "perimeter": mesh.perimeter(),
        "mass": mesh.total_structural_mass(),
        "young_mass": mesh.total_young_structural_mass(),
        "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
        "physical_runtime_valid": mesh.physical_runtime_valid(),
    })
}

fn r4_daughter_viability(
    mut mesh: MaterialMesh,
    mechanics: &MechParams,
    reaction: &ReactionParams,
    transport: &TransportParams,
    fission: &FissionParams,
) -> Value {
    let c_initial = mesh.interior.c;
    let a_initial = mesh.interior.a;
    let growth_off = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let mut all_runtime_valid = mesh.physical_runtime_valid();
    let mut completed_steps = 0_usize;
    for step in 0..DAUGHTER_CONTINUATION_STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut mesh, transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, reaction, &growth_off, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, mechanics).is_none() {
            all_simple = false;
            break;
        }
        let _ = remesh_preserving_simple(&mut mesh);
        let _ = topology_step(&mut mesh, fission);
        all_simple &= polygon_simple(&mesh.vertices);
        all_lifecycle_valid &= mesh.lifecycle_invariants_hold();
        all_runtime_valid &= mesh.physical_runtime_valid();
        if !all_simple || !all_lifecycle_valid || !all_runtime_valid {
            break;
        }
        completed_steps = step + 1;
    }
    let c_retention = if c_initial > 1e-12 {
        mesh.interior.c / c_initial
    } else {
        1.0
    };
    let a_retention = if a_initial > 1e-12 {
        mesh.interior.a / a_initial
    } else {
        1.0
    };
    let viable = completed_steps == DAUGHTER_CONTINUATION_STEPS
        && mesh.observer_viable()
        && mesh.closed_intact()
        && all_simple
        && all_lifecycle_valid
        && all_runtime_valid
        && c_retention >= 0.80
        && a_retention >= 0.80;
    json!({
        "viable": viable,
        "completed_steps": completed_steps,
        "target_steps": DAUGHTER_CONTINUATION_STEPS,
        "alive": mesh.alive,
        "observer_viable": mesh.observer_viable(),
        "closed_intact": mesh.closed_intact(),
        "all_states_simple": all_simple,
        "all_lifecycle_invariants_hold": all_lifecycle_valid,
        "all_physical_runtime_valid": all_runtime_valid,
        "c_retention": c_retention,
        "a_retention": a_retention,
        "terminal": r4_geometry(&mesh),
    })
}

fn r4_reproduction_run(
    initial_mesh: MaterialMesh,
    name: &str,
    allow_segment: bool,
    half_edge: bool,
) -> Value {
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth_mass = initial_mesh.total_structural_mass();
    let mut mesh = initial_mesh;
    let mut result = None;
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let mut all_runtime_valid = mesh.physical_runtime_valid();
    let mut bookkeeping_runtime_invalidations = 0_usize;
    let mut maximum_mass_ratio = 1.0_f64;
    let mut vertex_attempts = 0_usize;
    let mut segment_attempts = 0_usize;
    let mut first_invalid = None;
    for step in 0..REPRODUCTION_STEPS {
        if !mesh.can_advance_physics() {
            bookkeeping_runtime_invalidations += 1;
            first_invalid = Some(json!({"step": step + 1, "phase": "pre_step"}));
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_none() {
            all_simple = false;
            first_invalid = Some(json!({"step": step + 1, "phase": "self_contact_mechanics"}));
            break;
        }
        if half_edge {
            let _ = remesh_preserving_simple(&mut mesh);
        } else {
            let _ = remesh(&mut mesh);
        }
        let planar = if half_edge {
            PlanarRingTopology::from_mesh(&mesh)
        } else {
            None
        };
        if half_edge && planar.is_none() {
            all_simple = false;
            first_invalid = Some(json!({"step": step + 1, "phase": "half_edge_rebuild"}));
            break;
        }
        if step % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        let simple = polygon_simple(&mesh.vertices);
        let lifecycle_valid = mesh.lifecycle_invariants_hold();
        let runtime_valid = mesh.physical_runtime_valid();
        all_simple &= simple;
        all_lifecycle_valid &= lifecycle_valid;
        all_runtime_valid &= runtime_valid;
        if !lifecycle_valid || !runtime_valid {
            bookkeeping_runtime_invalidations += 1;
        }
        if (!simple || !lifecycle_valid || !runtime_valid) && first_invalid.is_none() {
            first_invalid = Some(json!({
                "step": step + 1,
                "phase": "post_topology",
                "simple": simple,
                "lifecycle_valid": lifecycle_valid,
                "runtime_valid": runtime_valid,
            }));
        }
        if !simple || !lifecycle_valid || !runtime_valid {
            break;
        }
        maximum_mass_ratio =
            maximum_mass_ratio.max(mesh.total_structural_mass() / birth_mass.max(1e-300));
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            vertex_attempts += 1;
            let vertex_split = try_local_fission(&mesh, &fission).and_then(|(a, b, event)| {
                (polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok
                    && a.lifecycle_invariants_hold()
                    && b.lifecycle_invariants_hold()
                    && a.physical_runtime_valid()
                    && b.physical_runtime_valid())
                .then_some((a, b, event))
            });
            let split = vertex_split.or_else(|| {
                if !allow_segment {
                    return None;
                }
                segment_attempts += 1;
                if let Some(topology) = planar.as_ref() {
                    topology.try_local_scission(&mesh, &fission)
                } else {
                    try_local_segment_fission(&mesh, &fission)
                }
            });
            if let Some((a, b, event)) = split {
                let valid = polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok
                    && a.lifecycle_invariants_hold()
                    && b.lifecycle_invariants_hold()
                    && a.physical_runtime_valid()
                    && b.physical_runtime_valid();
                if valid {
                    let viability_a = r4_daughter_viability(
                        a.clone(),
                        &mechanics,
                        &reaction,
                        &transport,
                        &fission,
                    );
                    let viability_b = r4_daughter_viability(
                        b.clone(),
                        &mechanics,
                        &reaction,
                        &transport,
                        &fission,
                    );
                    result = Some(json!({
                        "step": step + 1,
                        "valid": true,
                        "parent": r4_geometry(&mesh),
                        "daughter_a": r4_geometry(&a),
                        "daughter_b": r4_geometry(&b),
                        "daughter_a_viability": viability_a,
                        "daughter_b_viability": viability_b,
                        "both_daughters_viable": viability_a["viable"] == true && viability_b["viable"] == true,
                        "partition": event.partition,
                        "pinch": event.pinch,
                    }));
                    break;
                }
            }
        }
    }
    json!({
        "name": name,
        "contract": "MaturationCoupledV4",
        "segment_apposition_enabled": allow_segment,
        "half_edge_topology": half_edge,
        "all_parent_states_simple": all_simple,
        "all_parent_lifecycle_invariants_hold": all_lifecycle_valid,
        "all_parent_physical_runtime_valid": all_runtime_valid,
        "bookkeeping_runtime_invalidations": bookkeeping_runtime_invalidations,
        "max_mass_over_birth": maximum_mass_ratio,
        "growth_qualified": maximum_mass_ratio >= 1.35,
        "vertex_attempts": vertex_attempts,
        "segment_attempts": segment_attempts,
        "physical_fission": result.is_some(),
        "first_invalid": first_invalid,
        "fission": result,
        "final": r4_geometry(&mesh),
    })
}

fn r4_reproduction_campaign() -> Value {
    let kinds = [
        ("rotate", 0.3),
        ("vertex", 0.12),
        ("c", 0.08),
        ("a", 0.08),
        ("env", 0.1),
        ("l", 0.1),
        ("rotate", -0.5),
        ("vertex", -0.1),
        ("c", -0.05),
        ("env", -0.08),
    ];
    let vertex_only = kinds
        .iter()
        .enumerate()
        .map(|(index, (kind, magnitude))| {
            r4_reproduction_run(
                r4_reproduction_fixture((index + 1) as u64, kind, *magnitude),
                &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                false,
                false,
            )
        })
        .collect::<Vec<_>>();
    let vertex_fissions = vertex_only
        .iter()
        .filter(|arm| arm["physical_fission"] == true)
        .count();
    let segment = if vertex_fissions >= 7 {
        Vec::new()
    } else {
        kinds
            .iter()
            .enumerate()
            .map(|(index, (kind, magnitude))| {
                r4_reproduction_run(
                    r4_reproduction_fixture((index + 1) as u64, kind, *magnitude),
                    &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                    true,
                    false,
                )
            })
            .collect::<Vec<_>>()
    };
    let segment_fissions = segment
        .iter()
        .filter(|arm| arm["physical_fission"] == true)
        .count();
    let half_edge = if vertex_fissions >= 7 || segment_fissions >= 7 {
        Vec::new()
    } else {
        kinds
            .iter()
            .enumerate()
            .map(|(index, (kind, magnitude))| {
                r4_reproduction_run(
                    r4_reproduction_fixture((index + 1) as u64, kind, *magnitude),
                    &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                    true,
                    true,
                )
            })
            .collect::<Vec<_>>()
    };
    let selected = if vertex_fissions >= 7 {
        &vertex_only
    } else if segment_fissions >= 7 {
        &segment
    } else {
        &half_edge
    };
    let growth_qualified = selected
        .iter()
        .filter(|arm| arm["growth_qualified"] == true)
        .count();
    let valid_fissions = selected
        .iter()
        .filter(|arm| arm["physical_fission"] == true)
        .count();
    let viable_pairs = selected
        .iter()
        .filter(|arm| arm["fission"]["both_daughters_viable"] == true)
        .count();
    let bookkeeping_invalidations = selected
        .iter()
        .map(|arm| {
            arm["bookkeeping_runtime_invalidations"]
                .as_u64()
                .unwrap_or(0)
        })
        .sum::<u64>();
    let pass = selected.len() == 10
        && growth_qualified >= 8
        && valid_fissions >= 7
        && viable_pairs >= 6
        && bookkeeping_invalidations == 0;
    json!({
        "frozen_protocol": {
            "arms": 10,
            "steps": REPRODUCTION_STEPS,
            "daughter_steps": DAUGHTER_CONTINUATION_STEPS,
            "growth_yield": 0.9,
            "mass_gate": 1.35,
            "fission_cadence": 25,
            "topology_cadence": 10,
            "contract": "MaturationCoupledV4",
        },
        "vertex_only": vertex_only,
        "segment_apposition": segment,
        "half_edge_fallback": half_edge,
        "selected_path": if vertex_fissions >= 7 { "VERTEX_ONLY" } else if segment_fissions >= 7 { "SEGMENT_APPOSITION" } else { "HALF_EDGE_FALLBACK" },
        "growth_qualified": growth_qualified,
        "geometry_valid_fissions": valid_fissions,
        "simple_viable_daughter_pairs": viable_pairs,
        "bookkeeping_runtime_invalidations": bookkeeping_invalidations,
        "pass": pass,
    })
}

fn contract_ownership_matrix() -> Value {
    use chemistry_core::material_mesh::MeshContractVersion;
    let contracts = [
        MeshContractVersion::HistoricalV1,
        MeshContractVersion::ConservativeV2,
        MeshContractVersion::GeometryConservativeV3,
        MeshContractVersion::MaturationCoupledV4,
    ];
    let states = [
        ("A_ZERO", 0.0),
        ("B_PARTIAL", 0.5),
        ("C_EQUAL", 1.0),
        ("D_EXCEEDS", 1.5),
    ];
    let mut rows = Vec::new();
    for contract in contracts {
        for (state, young) in states {
            let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, 1, 2.2)
                .individuals
                .remove(0)
                .mesh;
            mesh.contract_version = contract;
            mesh.edges[0].m = 1.0;
            mesh.edges[0].m_young = young;
            let serialized_before = serde_json::to_vec(&mesh).unwrap();
            let row = json!({
                "contract": format!("{contract:?}"),
                "state": state,
                "m": mesh.edges[0].m,
                "m_young_stored": mesh.edges[0].m_young,
                "young_structural_mass": mesh.young_structural_mass(0),
                "mature_structural_mass": mesh.mature_structural_mass(0),
                "rest_length": mesh.rest_length(0),
                "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
                "physical_runtime_valid": mesh.physical_runtime_valid(),
                "can_advance_physics": mesh.can_advance_physics(),
            });
            let serialized_after = serde_json::to_vec(&mesh).unwrap();
            rows.push(json!({
                "observation": row,
                "serialized_state_unchanged_by_validation": serialized_before == serialized_after,
            }));
        }
    }
    json!({
        "rows": rows,
        "classification": "M_YOUNG_OWNED_ONLY_BY_MATURATION_COUPLED_V4",
        "non_v4_state_rewritten": false,
    })
}

fn non_v4_d096_parity(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (mut daughter, _, event) =
        try_local_fission(&parent, &fission).expect("historical daughter");
    assert!(event.partition.ok);
    let expression = expression_step(&mut daughter, &allocation, MechParams::default().dt)
        .expect("historical D096 expression");
    let bytes_before_validation = serde_json::to_vec(&daughter).unwrap();
    let physical_runtime_valid = daughter.physical_runtime_valid();
    let can_advance_physics = daughter.can_advance_physics();
    let bytes_after_validation = serde_json::to_vec(&daughter).unwrap();
    let violating_inert_edges = daughter
        .edges
        .iter()
        .filter(|edge| edge.m_young > edge.m + 1e-12)
        .count();
    json!({
        "contract": format!("{:?}", daughter.contract_version),
        "expression_ledger": expression,
        "violating_inert_m_young_edges": violating_inert_edges,
        "physical_runtime_valid_after_repair": physical_runtime_valid,
        "can_advance_physics_after_repair": can_advance_physics,
        "serialized_state_unchanged_by_validation": bytes_before_validation == bytes_after_validation,
        "historical_d096_equations_changed": false,
        "classification": "PREVIOUS_RUNTIME_INVALIDATION_REMOVED",
    })
}

fn tagged_closing_edges(mesh: &MaterialMesh, mechanics: &MechParams) -> Vec<Value> {
    mesh.edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| edge.tracer_m > 1e-15)
        .map(|(index, edge)| {
            let length = mesh.edge_length(index);
            let rest = mesh.rest_length(index);
            let reference = rest.max(0.25 * length).max(1e-3);
            let stretch = (mechanics.k_s * (length - rest) / reference)
                .clamp(-mechanics.k_s * 8.0, mechanics.k_s * 8.0);
            json!({
                "edge": index,
                "observer_provenance_fraction": edge.tracer_m / edge.m.max(f64::MIN_POSITIVE),
                "m": edge.m,
                "m_young": edge.m_young,
                "m_mature": mesh.mature_structural_mass(index),
                "young_fraction": edge.m_young / edge.m.max(f64::MIN_POSITIVE),
                "rest_length": rest,
                "geometric_length": length,
                "strain": mesh.strain(index),
                "stretch_force_contribution": stretch,
                "ruptured": edge.ruptured,
            })
        })
        .collect()
}

fn trace_one_v4_closing_edge(mut mesh: MaterialMesh, daughter: &str) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for edge in &mut mesh.edges {
        edge.tracer_m = 0.0;
    }
    let closing_index = mesh.n() - 1;
    mesh.edges[closing_index].tracer_m = mesh.edges[closing_index].m;
    let initial_closing_edge = tagged_closing_edges(&mesh, &mechanics);
    let mut trace = Vec::new();
    let mut provenance_termination = None;
    for step in 0..100_usize {
        let before = tagged_closing_edges(&mesh, &mechanics);
        let area_before_topology = mesh.area().max(1e-9);
        let expression = expression_step(&mut mesh, &allocation, mechanics.dt)
            .expect("closing edge D096 expression");
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let reaction_ledger = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        let contact_accepted =
            mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some();
        let _ = remesh(&mut mesh);
        let a_before_topology = mesh.interior.a * mesh.area().max(1e-9);
        let w_before_topology = mesh.interior.w * mesh.area().max(1e-9);
        let topology = if step % 10 == 0 {
            topology_step(&mut mesh, &fission)
        } else {
            Default::default()
        };
        let a_after_topology = mesh.interior.a * mesh.area().max(1e-9);
        let w_after_topology = mesh.interior.w * mesh.area().max(1e-9);
        let after = tagged_closing_edges(&mesh, &mechanics);
        if !before.is_empty() && after.is_empty() && provenance_termination.is_none() {
            provenance_termination = Some(json!({
                "step": step + 1,
                "cause": if topology.tension_ruptures > 0 { "STRUCTURAL_MATERIAL_DESTROYED_BY_TENSION_RUPTURE" } else { "TAGGED_MATERIAL_TURNED_OVER_OR_REMESHED_BELOW_NUMERICAL_TRACE" },
            }));
        }
        trace.push(json!({
            "step": step + 1,
            "before": before,
            "after": after,
            "expression_material_consumed": expression.material_consumed,
            "maturation_amount": reaction_ledger.m_matured,
            "contact_accepted": contact_accepted,
            "tension_ruptures": topology.tension_ruptures,
            "rebonded": topology.local_rebonds > 0,
            "local_rebonds": topology.local_rebonds,
            "a_used_for_rebond_upper_bound": (a_before_topology - a_after_topology).max(0.0),
            "w_added_by_rupture": (w_after_topology - w_before_topology).max(0.0),
            "area_before_topology_observer": area_before_topology,
            "polygon_simple": polygon_simple(&mesh.vertices),
            "runtime_valid": mesh.physical_runtime_valid(),
            "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
        }));
        if !contact_accepted || !mesh.physical_runtime_valid() || !mesh.lifecycle_invariants_hold()
        {
            break;
        }
    }
    json!({
        "daughter": daughter,
        "observer_tag": "tracer_m set on diagnostic clone only; observer tracer never enters biology",
        "initial_closing_edge_index": closing_index,
        "initial_closing_edge": initial_closing_edge,
        "trace": trace,
        "provenance_termination": provenance_termination,
    })
}

fn v4_closing_edge_trace(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.stamp_maturation_coupled_schema();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("V4 trace fission");
    assert!(event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices));
    json!({
        "parent_contract": format!("{:?}", parent.contract_version),
        "pinch": event.pinch,
        "daughter_a": trace_one_v4_closing_edge(a, "A"),
        "daughter_b": trace_one_v4_closing_edge(b, "B"),
        "classification": "BOOKKEEPING_ONLY_RUPTURE_PATH",
        "conditional_topology_created_edge_repair": "NOT_REQUIRED",
    })
}

fn r10_split_cohort(
    cohort: Cohort,
    mutation_enabled: bool,
    campaign_seed: u64,
    step: usize,
    next_id: &mut u64,
    allocation: &AllocationParams,
    fission: &FissionParams,
    ledger: &mut CampaignLedger,
    mut daughter_continuation: Option<&mut Vec<Value>>,
    capture_daughter_continuation: bool,
    mut polarity_states: Option<&mut PolarityStateMap>,
) -> Result<Vec<Cohort>, Cohort> {
    let parent_cohort_id = cohort.id;
    let parent_generation = cohort.generation;
    let parent_count = cohort.count;
    let parent_genotype = cohort
        .mesh
        .finite_allocation
        .expect("R10 allocation")
        .genotype;
    let parent_structural_mass = cohort.mesh.total_structural_mass();
    let Some(parent_plasticity) = cohort.plasticity.as_ref() else {
        return Err(cohort);
    };
    let proposed = try_local_fission(&cohort.mesh, fission).or_else(|| {
        PlanarRingTopology::from_mesh(&cohort.mesh)
            .and_then(|topology| topology.try_local_scission(&cohort.mesh, fission))
            .or_else(|| try_local_segment_fission(&cohort.mesh, fission))
    });
    let Some((daughter_a, daughter_b, event)) = proposed else {
        lifecycle_event(
            ledger,
            "fission_attempt",
            step,
            &cohort,
            "NO_VALID_PHYSICAL_PROPOSAL",
            r10_fission_attempt_detail(&cohort.mesh, fission),
        );
        return Err(cohort);
    };
    if !event.partition.ok {
        ledger.partition_failures += cohort.count;
        let mut detail = r10_fission_attempt_detail(&cohort.mesh, fission);
        detail["dispatch_outcome"] = json!("PARTITION_REJECTED");
        lifecycle_event(
            ledger,
            "fission_attempt",
            step,
            &cohort,
            "PARTITION_REJECTED",
            detail,
        );
        return Err(cohort);
    }
    if !polygon_simple(&cohort.mesh.vertices)
        || !polygon_simple(&daughter_a.vertices)
        || !polygon_simple(&daughter_b.vertices)
        || !daughter_a.physical_runtime_valid()
        || !daughter_b.physical_runtime_valid()
        || !daughter_a.lifecycle_invariants_hold()
        || !daughter_b.lifecycle_invariants_hold()
    {
        ledger.invalid_geometry_events += cohort.count;
        let mut detail = r10_fission_attempt_detail(&cohort.mesh, fission);
        detail["dispatch_outcome"] = json!("DAUGHTER_GEOMETRY_REJECTED");
        detail["daughter_geometry"] = json!({
            "parent_simple": polygon_simple(&cohort.mesh.vertices),
            "daughter_a_simple": polygon_simple(&daughter_a.vertices),
            "daughter_b_simple": polygon_simple(&daughter_b.vertices),
            "daughter_a_runtime_valid": daughter_a.physical_runtime_valid(),
            "daughter_b_runtime_valid": daughter_b.physical_runtime_valid(),
            "daughter_a_lifecycle_valid": daughter_a.lifecycle_invariants_hold(),
            "daughter_b_lifecycle_valid": daughter_b.lifecycle_invariants_hold(),
        });
        lifecycle_event(
            ledger,
            "fission_attempt",
            step,
            &cohort,
            "DAUGHTER_GEOMETRY_REJECTED",
            detail,
        );
        return Err(cohort);
    }
    let Some(state_a) = r10_closure::r10_partition_plasticity_state(
        parent_plasticity,
        &event.daughter_a_parent_vertex_sources,
    ) else {
        ledger.runtime_invalidations += cohort.count;
        let mut detail = r10_fission_attempt_detail(&cohort.mesh, fission);
        detail["dispatch_outcome"] = json!("REFRACTORY_PARTITION_REJECTED");
        detail["error"] = json!("LOCAL_STATE_CORRESPONDENCE_UNAVAILABLE");
        lifecycle_event(
            ledger,
            "fission_attempt",
            step,
            &cohort,
            "REFRACTORY_PARTITION_REJECTED",
            detail,
        );
        return Err(cohort);
    };
    let Some(state_b) = r10_closure::r10_partition_plasticity_state(
        parent_plasticity,
        &event.daughter_b_parent_vertex_sources,
    ) else {
        ledger.runtime_invalidations += cohort.count;
        let mut detail = r10_fission_attempt_detail(&cohort.mesh, fission);
        detail["dispatch_outcome"] = json!("REFRACTORY_PARTITION_REJECTED");
        detail["error"] = json!("LOCAL_STATE_CORRESPONDENCE_UNAVAILABLE");
        lifecycle_event(
            ledger,
            "fission_attempt",
            step,
            &cohort,
            "REFRACTORY_PARTITION_REJECTED",
            detail,
        );
        return Err(cohort);
    };
    if state_a.adaptation.len() != daughter_a.n() || state_b.adaptation.len() != daughter_b.n() {
        ledger.runtime_invalidations += cohort.count;
        let mut detail = r10_fission_attempt_detail(&cohort.mesh, fission);
        detail["dispatch_outcome"] = json!("REFRACTORY_PARTITION_SHAPE_MISMATCH");
        detail["state_shape"] = json!({
            "state_a": state_a.adaptation.len(),
            "daughter_a": daughter_a.n(),
            "state_b": state_b.adaptation.len(),
            "daughter_b": daughter_b.n(),
        });
        lifecycle_event(
            ledger,
            "fission_attempt",
            step,
            &cohort,
            "REFRACTORY_PARTITION_SHAPE_MISMATCH",
            detail,
        );
        return Err(cohort);
    }

    // Partition the accepted polarity material by the exact physical parent
    // correspondence before any child is published.  A missing or invalid
    // mapping is a lifecycle error, not a reason to discard the parent.
    let polarity_children = if let Some(states) = polarity_states.as_deref() {
        let Some(parent_polarity) = states.get(&parent_cohort_id) else {
            ledger.runtime_invalidations += cohort.count;
            lifecycle_event(
                ledger,
                "polarity_fission_partition",
                step,
                &cohort,
                "POLARITY_PARTITION_REJECTED",
                json!({"error": "PARENT_POLARITY_STATE_MISSING"}),
            );
            return Err(cohort);
        };
        let parent_measures = (0..cohort.mesh.n())
            .map(|index| cohort.mesh.edge_length(index))
            .collect::<Vec<_>>();
        let daughter_a_measures = (0..daughter_a.n())
            .map(|index| daughter_a.edge_length(index))
            .collect::<Vec<_>>();
        let daughter_b_measures = (0..daughter_b.n())
            .map(|index| daughter_b.edge_length(index))
            .collect::<Vec<_>>();
        let daughter_a_parent = event
            .daughter_a_parent_vertex_sources
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<_>>();
        let daughter_b_parent = event
            .daughter_b_parent_vertex_sources
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<_>>();
        match parent_polarity.split_by_correspondence(
            &parent_measures,
            &daughter_a_measures,
            &daughter_a_parent,
            &daughter_b_measures,
            &daughter_b_parent,
        ) {
            Ok((a, b)) => Some((a, b)),
            Err(error) => {
                ledger.runtime_invalidations += cohort.count;
                lifecycle_event(
                    ledger,
                    "polarity_fission_partition",
                    step,
                    &cohort,
                    "POLARITY_PARTITION_REJECTED",
                    json!({"error": error.to_string()}),
                );
                return Err(cohort);
            }
        }
    } else {
        None
    };

    let closure_input = (daughter_a.total_structural_mass() + daughter_b.total_structural_mass()
        - parent_structural_mass)
        * cohort.count as f64;
    if step == 0 {
        ledger.bootstrap_fission_closure_structural_input += closure_input;
    } else {
        ledger.post_bootstrap_fission_closure_structural_input += closure_input;
    }

    ledger.physical_fissions += cohort.count;
    ledger.valid_simple_fissions += cohort.count;
    if step == 0 {
        ledger.bootstrap_physical_fissions += cohort.count;
    } else {
        ledger.post_bootstrap_physical_fissions += cohort.count;
    }
    phenotype_ledger(ledger, parent_genotype).physical_fissions += cohort.count;
    *ledger
        .fissions_by_parent_genotype
        .entry(genotype_key(
            cohort
                .mesh
                .finite_allocation
                .expect("R10 allocation")
                .genotype,
        ))
        .or_default() += cohort.count;
    let mutation_params = if mutation_enabled {
        *allocation
    } else {
        AllocationParams {
            mutation_probability: 0.0,
            ..*allocation
        }
    };
    let children = [(daughter_a, state_a), (daughter_b, state_b)];
    let mut groups: BTreeMap<(usize, [u64; 4]), (MaterialMesh, PlasticityStateV1, u64, u64)> =
        BTreeMap::new();
    for ordinal in 0..cohort.count {
        for (side, (child_template, state_template)) in children.iter().enumerate() {
            let parent = child_template
                .finite_allocation
                .expect("R10 daughter allocation")
                .genotype;
            let seed = mutation_seed(campaign_seed, step, cohort.id, ordinal, side as u64);
            let mutation = mutate_allocation_at_reproduction(parent, &mutation_params, seed);
            ledger.mutation_opportunities += 1;
            if mutation.mutated {
                ledger.mutations += 1;
                ledger.mutation_events.push(json!({
                    "step": step,
                    "generation": cohort.generation + 1,
                    "seed": seed,
                    "parent": mutation.parent.0,
                    "offspring": mutation.offspring.0,
                    "source_index": mutation.source_index,
                    "target_index": mutation.target_index,
                    "transferred": mutation.transferred,
                }));
            }
            let key = (side, mutation.offspring.0.map(f64::to_bits));
            groups
                .entry(key)
                .and_modify(|(_, _, count, mutations)| {
                    *count += 1;
                    *mutations += u64::from(mutation.mutated);
                })
                .or_insert_with(|| {
                    let mut mesh = child_template.clone();
                    mesh.finite_allocation.as_mut().unwrap().genotype = mutation.offspring;
                    (mesh, state_template.clone(), 1, u64::from(mutation.mutated))
                });
        }
    }
    let mut result = Vec::new();
    for ((side, _), (mesh, plasticity, count, mutations)) in groups {
        let id = *next_id;
        *next_id += 1;
        let daughter_genotype = mesh
            .finite_allocation
            .expect("R10 daughter allocation")
            .genotype;
        ledger.physical_birth_events.push(json!({
            "step": step,
            "bootstrap": step == 0,
            "parent_cohort_id": parent_cohort_id,
            "parent_generation": parent_generation,
            "parent_count": parent_count,
            "parent_genotype": parent_genotype.0,
            "daughter_side": side,
            "daughter_cohort_id": id,
            "daughter_generation": parent_generation + 1,
            "daughter_count": count,
            "daughter_genotype": daughter_genotype.0,
            "mutated_daughters": mutations,
            "refractory_state_digest": deterministic_state_digest(&plasticity),
            "refractory_source_correspondence": if side == 0 {
                &event.daughter_a_parent_vertex_sources
            } else {
                &event.daughter_b_parent_vertex_sources
            },
        }));
        if capture_daughter_continuation {
            if let Some(records) = daughter_continuation.as_mut() {
                let records = &mut **records;
                records.push(json!({
                    "step": step,
                    "bootstrap": step == 0,
                    "parent_cohort_id": parent_cohort_id,
                    "parent_generation": parent_generation,
                    "daughter_side": side,
                    "daughter_cohort_id": id,
                    "daughter_generation": parent_generation + 1,
                    "daughter_count": count,
                    "daughter_genotype": daughter_genotype.0,
                    "refractory_state_digest": deterministic_state_digest(&plasticity),
                    "continuation": r10_closure::r10_current_daughter_continuation_json(
                        mesh.clone(),
                        plasticity.clone(),
                    ),
                }));
            }
        }
        result.push(Cohort {
            birth_mass: mesh.total_structural_mass(),
            mesh,
            plasticity: Some(plasticity),
            count,
            generation: parent_generation + 1,
            id,
        });
        if let (Some(states), Some((state_a, state_b))) =
            (polarity_states.as_deref_mut(), polarity_children.as_ref())
        {
            states.insert(
                id,
                if side == 0 {
                    state_a.clone()
                } else {
                    state_b.clone()
                },
            );
        }
    }
    if let Some(states) = polarity_states.as_deref_mut() {
        states.remove(&parent_cohort_id);
    }
    Ok(result)
}

fn r10_initial_population(
    template: &MaterialMesh,
    template_plasticity: &PlasticityStateV1,
    original_birth_mass: f64,
    mutation_enabled: bool,
    campaign_seed: u64,
    founder_count: u64,
    ledger: &mut CampaignLedger,
    expression_path: D096ExpressionPath,
) -> Vec<Cohort> {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    if expression_path == D096ExpressionPath::V2ActivatedMaterial {
        parent.enable_finite_allocation_v2(AllocationGenotype::neutral(), &allocation);
    } else if expression_path == D096ExpressionPath::V4FiniteBudgetCentered {
        parent.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
    } else {
        parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    }
    let cohort = Cohort {
        mesh: parent,
        plasticity: Some(template_plasticity.clone()),
        count: founder_count,
        generation: 0,
        birth_mass: original_birth_mass,
        id: 1,
    };
    let mut next_id = 2;
    r10_split_cohort(
        cohort,
        mutation_enabled,
        campaign_seed,
        0,
        &mut next_id,
        &allocation,
        &fission,
        ledger,
        None,
        false,
        None,
    )
    .expect("qualified R10 parent must physically fission")
}

fn r10_advance_phase(
    cohorts: &mut Vec<Cohort>,
    world: &mut OpenMedium,
    environment: Environment,
    phase_index: usize,
    mutation_enabled: bool,
    campaign_seed: u64,
    next_id: &mut u64,
    ledger: &mut CampaignLedger,
    trajectory: &mut Vec<Value>,
    prefix_2500: &mut Option<Value>,
    phase_steps: usize,
    expression_path: D096ExpressionPath,
    boundary_mode: PopulationBoundaryMode,
    clock_mode: FissionClockMode,
    reserve_resolution: ReserveResolutionMode,
    daughter_continuations: &mut Vec<Value>,
    stop_after_first_fission: bool,
    boundary_n_source: DiagnosticBoundarySource,
    boundary_f_source: DiagnosticBoundarySource,
    motor_enabled: bool,
    adaptation_enabled: bool,
    growth_placement: GrowthPlacementMode,
    mut mechanics_diagnostics: Option<&mut Vec<Value>>,
    mut polarity_states: Option<&mut PolarityStateMap>,
    polarity_actuator_connected: bool,
    mut atomic_replay_snapshots: Option<&mut Vec<Value>>,
    mut full_cycle_snapshots: Option<&mut Vec<Value>>,
) -> bool {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let initial_resolved_reaction = match reserve_resolution {
        ReserveResolutionMode::PerStep => None,
        ReserveResolutionMode::OnceBeforeTrajectory => cohorts
            .first()
            .map(|cohort| r10r9r1_reaction_params(&cohort.mesh)),
    };
    let mut pending_full_cycle = BTreeMap::<(u64, usize), Value>::new();
    for phase_step in 0..phase_steps {
        if cohorts.is_empty() {
            break;
        }
        let step = phase_index * phase_steps + phase_step + 1;
        // Snapshot the complete accepted-step boundary before any boundary
        // refresh, damage, cohort transition, random birth, or ledger write.
        // A rejected mechanics/remap proposal invalidates this whole step;
        // the campaign then stops instead of selectively advancing biology.
        let step_cohorts_before = cohorts.clone();
        let step_world_before = world.clone();
        let step_ledger_before = ledger.clone();
        let step_next_id = *next_id;
        let step_polarity_before = polarity_states.as_deref().cloned();
        let mut first_fission_this_step = false;
        let mut expression_by_cohort = BTreeMap::<u64, ExpressionAccounting>::new();
        let mut polarity_activity: Option<Vec<f64>> = None;
        let mut polarity_observation: Option<Value> = None;
        let boundary_target_n = boundary_n_source.value(environment, phase_step);
        let boundary_target_f = boundary_f_source.fuel_value(environment, phase_step);
        match boundary_mode {
            PopulationBoundaryMode::RateReinterpretation => {
                world.add_fixed_inflow(environment, phase_step, mechanics.dt)
            }
            PopulationBoundaryMode::FixedConcentrationBoundary => world
                .refresh_fixed_concentration_boundary_values(boundary_target_n, boundary_target_f),
            PopulationBoundaryMode::FixtureExteriorReference => {}
        }
        if environment == Environment::Damage {
            for cohort in cohorts.iter_mut() {
                let genotype = cohort
                    .mesh
                    .finite_allocation
                    .expect("R10 allocation")
                    .genotype;
                let structural_before = world.ledger.damage_structural_sink;
                let membrane_before = world.ledger.damage_membrane_sink;
                apply_damage(cohort, phase_step, world);
                let observer = phenotype_ledger(ledger, genotype);
                observer.damage_structural +=
                    world.ledger.damage_structural_sink - structural_before;
                observer.damage_membrane += world.ledger.damage_membrane_sink - membrane_before;
            }
        }
        let mut retained = Vec::new();
        for mut cohort in std::mem::take(cohorts) {
            match apply_expression_path(
                &mut cohort.mesh,
                &allocation,
                mechanics.dt,
                expression_path,
            ) {
                Ok(expression) => {
                    let count = cohort.count as f64;
                    let genotype = cohort
                        .mesh
                        .finite_allocation
                        .expect("R10 allocation")
                        .genotype;
                    ledger.expression_material += expression.structural_consumed * count;
                    ledger.expression_catalyst_precursor_a +=
                        expression.catalyst_precursor_a * count;
                    ledger.expression_activation +=
                        (expression.activation_consumed + expression.maintenance_consumed) * count;
                    ledger.expression_turnover_waste += expression.turnover_waste * count;
                    expression_by_cohort.insert(cohort.id, expression);
                    let observer = phenotype_ledger(ledger, genotype);
                    observer.organism_step_exposure += cohort.count;
                    observer.expression_material += expression.structural_consumed * count;
                    observer.expression_activation +=
                        (expression.activation_consumed + expression.maintenance_consumed) * count;
                    retained.push(cohort);
                }
                Err(error) if r10r9r5_canonical_lifecycle() => {
                    // Expression is already transactional in chemistry-core:
                    // it computes on a clone and commits only on success.  A
                    // depleted valid organism therefore takes a zero-flux
                    // expression step; no cohort is computationally deleted.
                    if error == ExpressionBoundaryError::InsufficientActivatedResource {
                        // The synthesis proposal is zero-funded, but the
                        // existing V4 catalyst turnover/maintenance dynamics
                        // still run.  Only a malformed or otherwise
                        // rejected state is restored unchanged.
                        let turnover_result = match expression_path {
                            D096ExpressionPath::V4FiniteBudgetCentered => {
                                expression_step_activated_material_v4_turnover_only(
                                    &mut cohort.mesh,
                                    &allocation,
                                    mechanics.dt,
                                )
                                .map(|value| ExpressionAccounting {
                                    catalyst_precursor_a: value.catalyst_precursor_consumed,
                                    activation_consumed: value.activation_consumed,
                                    maintenance_consumed: value.maintenance_consumed,
                                    turnover_waste: value.turnover_waste,
                                    ..Default::default()
                                })
                                .map_err(ExpressionBoundaryError::from)
                            }
                            _ => Err(ExpressionBoundaryError::InsufficientActivatedResource),
                        };
                        if let Ok(expression) = turnover_result {
                            let count = cohort.count as f64;
                            let genotype = cohort
                                .mesh
                                .finite_allocation
                                .expect("R10 allocation")
                                .genotype;
                            ledger.expression_activation += expression.maintenance_consumed * count;
                            ledger.expression_turnover_waste += expression.turnover_waste * count;
                            expression_by_cohort.insert(cohort.id, expression);
                            ledger.legal_depletion_steps += cohort.count;
                            let observer = phenotype_ledger(ledger, genotype);
                            observer.organism_step_exposure += cohort.count;
                            observer.expression_activation +=
                                expression.maintenance_consumed * count;
                            retained.push(cohort);
                            lifecycle_event(
                                ledger,
                                "d096_expression",
                                step,
                                retained.last().expect("retained cohort"),
                                "LEGAL_DEPLETION_TURNOVER_ONLY",
                                json!({"error": error.label(), "synthesis": 0.0}),
                            );
                            continue;
                        }
                        let rejected_count = cohort.count;
                        *cohorts = step_cohorts_before.clone();
                        *world = step_world_before.clone();
                        *ledger = step_ledger_before.clone();
                        *next_id = step_next_id;
                        ledger.computational_rejections += rejected_count;
                        ledger.rejected_steps += 1;
                        ledger.rollback_count += rejected_count;
                        ledger.numerical_invalid = true;
                        ledger.numerical_invalid_reason = Some(json!({
                            "phase": phase_index,
                            "step": step,
                            "cohort_id": cohort.id,
                            "error": "ZERO_FUNDED_TURNOVER_TRANSITION_REJECTED",
                            "detail": error.label(),
                        }));
                        lifecycle_event(
                            ledger,
                            "d096_expression",
                            step,
                            &cohort,
                            "ATOMIC_REJECTION_CAMPAIGN_INVALIDATED",
                            json!({"error": error.label()}),
                        );
                        return false;
                    } else {
                        let rejected_count = cohort.count;
                        *cohorts = step_cohorts_before.clone();
                        *world = step_world_before.clone();
                        *ledger = step_ledger_before.clone();
                        *next_id = step_next_id;
                        ledger.computational_rejections += rejected_count;
                        ledger.rejected_steps += 1;
                        ledger.rollback_count += rejected_count;
                        ledger.numerical_invalid = true;
                        ledger.numerical_invalid_reason = Some(json!({
                            "phase": phase_index,
                            "step": step,
                            "cohort_id": cohort.id,
                            "error": error.label(),
                        }));
                        lifecycle_event(
                            ledger,
                            "d096_expression",
                            step,
                            &cohort,
                            "ATOMIC_REJECTION_RETAINED",
                            json!({"error": error.label()}),
                        );
                        return false;
                    }
                    retained.push(cohort);
                }
                Err(error) => {
                    // Preserve the historical R4 behavior only on the
                    // explicitly legacy control. The R5 path never uses
                    // computational removal as a biological loss.
                    ledger.expression_failures += cohort.count;
                    ledger.runtime_invalidations += cohort.count;
                    lifecycle_event(
                        ledger,
                        "d096_expression",
                        step,
                        &cohort,
                        "LEGACY_COMPUTATIONAL_REMOVAL",
                        json!({"error": error.label()}),
                    );
                    invalidated_material_to_terminal(&cohort, world);
                }
            }
        }
        *cohorts = retained;
        let transport_before = cohorts
            .iter()
            .map(|cohort| {
                (
                    cohort.id,
                    [
                        cohort.mesh.interior.n * cohort.mesh.area(),
                        cohort.mesh.interior.f * cohort.mesh.area(),
                    ],
                )
            })
            .collect::<BTreeMap<_, _>>();
        if boundary_mode == PopulationBoundaryMode::FixtureExteriorReference {
            r10_fixture_exterior_exchange_with_observer(cohorts, &transport, mechanics.dt, ledger);
        } else {
            r10_exchange_with_observer(world, cohorts, &transport, mechanics.dt, ledger);
        }

        let mut survivors = Vec::new();
        for mut cohort in std::mem::take(cohorts) {
            let count = cohort.count as f64;
            let reaction = initial_resolved_reaction
                .as_ref()
                .cloned()
                .unwrap_or_else(|| r10r9r1_reaction_params(&cohort.mesh));
            let genotype = cohort
                .mesh
                .finite_allocation
                .expect("R10 allocation")
                .genotype;
            let reactions = reactions_step(&mut cohort.mesh, &reaction, mechanics.dt, true, true);
            ledger.reaction_n_consumed += reactions.n_consumed * count;
            ledger.reaction_f_consumed += reactions.f_consumed * count;
            ledger.a_produced += reactions.a_produced * count;
            ledger.w_produced += reactions.w_produced * count;
            ledger.reaction_activation_equivalent_closure_residual +=
                reactions.activation_equivalent_closure_residual * count;
            ledger.m1_structural_build += reactions.m_produced * count;
            ledger.m1_structural_turnover += reactions.m_to_w * count;
            let grown = growth_step_with_placement(
                &mut cohort.mesh,
                &reaction,
                &growth,
                mechanics.dt,
                growth_placement,
            );
            ledger.growth_material += grown.m_grown * count;
            ledger.growth_a_consumed += grown.a_consumed_growth * count;
            ledger.growth_w_produced += grown.w_from_growth * count;
            ledger.reserve_a_to_r += reactions.reserve.a_to_r * count;
            ledger.reserve_r_to_a += reactions.reserve.r_to_a * count;
            ledger.reserve_r_to_w += reactions.reserve.r_to_w * count;
            ledger.reserve_r_to_m += reactions.reserve.r_to_m * count;
            ledger.reserve_funded_growth += grown.r_consumed_growth * count;
            if reactions.reserve.a_to_r.abs()
                + reactions.reserve.r_to_a.abs()
                + reactions.reserve.r_to_w.abs()
                + grown.r_consumed_growth.abs()
                > 1e-12
            {
                ledger.reserve_active_steps += cohort.count;
            }
            {
                let observer = phenotype_ledger(ledger, genotype);
                observer.reaction_n_consumed += reactions.n_consumed * count;
                observer.reaction_f_consumed += reactions.f_consumed * count;
                observer.a_produced += reactions.a_produced * count;
                observer.w_produced += reactions.w_produced * count;
                observer.growth_material += grown.m_grown * count;
            }
            // R7 captures the earlier same-phase boundary, immediately
            // before polarity.advance.  The replay is observer-only: it
            // starts from a typed clone of this boundary and is compared to
            // the next canonical boundary before the live transition runs.
            if full_cycle_snapshots.is_some()
                && (R7_FULL_CYCLE_CHECKPOINTS.contains(&step)
                    || pending_full_cycle.contains_key(&(cohort.id, step)))
            {
                let Some(polarity_snapshot) = polarity_states.as_deref().cloned() else {
                    *cohorts = step_cohorts_before.clone();
                    *world = step_world_before.clone();
                    *ledger = step_ledger_before.clone();
                    *next_id = step_next_id;
                    ledger.numerical_invalid = true;
                    ledger.numerical_invalid_reason = Some(json!({
                        "phase": phase_index,
                        "step": step,
                        "error": "R7_POLARITY_STATE_NOT_CAPTURED",
                    }));
                    return false;
                };
                let boundary = r7_boundary_state_value(
                    &cohort,
                    world,
                    ledger,
                    &polarity_snapshot,
                    *next_id,
                    step,
                    campaign_seed,
                    polarity_actuator_connected,
                );
                if let Some(mut record) = pending_full_cycle.remove(&(cohort.id, step)) {
                    let expected = record["replay"]["next_boundary"].clone();
                    let identity = record["replay"]["status"] == "ACCEPTED" && expected == boundary;
                    record["actual_next_boundary"] = boundary.clone();
                    record["identity"] = json!({
                        "same_phase_boundary_exact": identity,
                        "expected_step": step,
                        "accepted_steps_at_boundary": ledger.accepted_steps,
                    });
                    if let Some(records) = full_cycle_snapshots.as_deref_mut() {
                        records.push(record);
                    }
                }
                if R7_FULL_CYCLE_CHECKPOINTS.contains(&step) {
                    let seed = R7BoundarySeed {
                        cohort: cohort.clone(),
                        world: world.clone(),
                        ledger: ledger.clone(),
                        polarity_states: polarity_snapshot,
                        next_id: *next_id,
                        step,
                        campaign_seed,
                        connected: polarity_actuator_connected,
                    };
                    let replay = r7_replay_full_cycle(&seed);
                    let response_operator = if r8_capture_only() {
                        json!({
                            "status": "SKIPPED_FOR_R8_CAPTURE_ONLY",
                            "reason": "R8 reuses the exact same-phase replay seeds and applies clone-local modular cuts; the superseded R7 global response probe is not part of the R8 measurement",
                            "observer_only": true,
                        })
                    } else {
                        r7_full_cycle_response_probe(&seed)
                    };
                    let same_state_disconnected_replay = if polarity_actuator_connected {
                        if r8_capture_only() {
                            Value::Null
                        } else {
                            let mut disconnected_seed = seed.clone();
                            disconnected_seed.connected = false;
                            r7_replay_full_cycle(&disconnected_seed)
                        }
                    } else {
                        Value::Null
                    };
                    pending_full_cycle.insert(
                        (cohort.id, step + 1),
                        json!({
                            "checkpoint_step": step,
                            "expected_next_step": step + 1,
                            "accepted_steps_before": ledger.accepted_steps,
                            "input_boundary": boundary,
                            "replay": replay,
                            "response_operator": response_operator,
                            "same_state_disconnected_replay": same_state_disconnected_replay,
                        }),
                    );
                }
            }
            if let Some(states) = polarity_states.as_deref_mut() {
                let measures = (0..cohort.mesh.n())
                    .map(|index| cohort.mesh.edge_length(index))
                    .collect::<Vec<_>>();
                let local_load_bearing_strain = if r9_mechanosensitive_polarity_enabled() {
                    Some(
                        (0..cohort.mesh.n())
                            .map(|index| cohort.mesh.load_bearing_strain(index))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                };
                let polarity_params = PolarityMassParamsV1::candidate();
                let actuation_params = PolarityActuationParamsV1::sealed_r3_candidate();
                let polarity_result = states
                    .get_mut(&cohort.id)
                    .ok_or_else(|| "POLARITY_STATE_MISSING".to_string())
                    .and_then(|state| {
                        let available_a = cohort.mesh.interior.a.max(0.0) * cohort.mesh.area();
                        let step_ledger = state
                            .advance_with_load_bearing_strain(
                                &measures,
                                &polarity_params,
                                available_a,
                                local_load_bearing_strain.as_deref(),
                            )
                            .map_err(|error| error.to_string())?;
                        let area = cohort.mesh.area();
                        if !area.is_finite() || area <= 0.0 {
                            return Err("POLARITY_INVALID_AREA".to_string());
                        }
                        let a_before = cohort.mesh.interior.a.max(0.0) * area;
                        let w_before = cohort.mesh.interior.w.max(0.0) * area;
                        if step_ledger.a_consumed > a_before + 1e-10 {
                            return Err("POLARITY_A_OVERDRAW".to_string());
                        }
                        cohort.mesh.interior.a =
                            (a_before - step_ledger.a_consumed).max(0.0) / area;
                        cohort.mesh.interior.w = (w_before + step_ledger.w_produced) / area;
                        let actuation_params =
                            actuation_params.map_err(|error| error.to_string())?;
                        let proposal = derive_local_activity(state, &measures, &actuation_params)
                            .map_err(|error| error.to_string())?;
                        polarity_activity = Some(proposal.vertex_activity.clone());
                        ledger.polarity_a_spent += step_ledger.a_consumed * count;
                        ledger.polarity_w_produced += step_ledger.w_produced * count;
                        Ok(json!({
                            "state_schema": state.schema,
                            "accepted": step_ledger.accepted,
                            "a_consumed": step_ledger.a_consumed,
                            "w_produced": step_ledger.w_produced,
                            "total_before": step_ledger.total_before,
                            "total_after": step_ledger.total_after,
                            "total_residual": step_ledger.total_after - step_ledger.total_before,
                            "active_before": step_ledger.active_before,
                            "active_after": step_ledger.active_after,
                            "inactive_before": step_ledger.inactive_before,
                            "inactive_after": step_ledger.inactive_after,
                            "mode_summaries": state.mode_summaries(&measures, 8)
                                .map_err(|error| error.to_string())?,
                            "actuation": proposal,
                            "mechanical_output": polarity_actuator_connected,
                        }))
                    });
                match polarity_result {
                    Ok(observation) => polarity_observation = Some(observation),
                    Err(error) => {
                        *cohorts = step_cohorts_before.clone();
                        *world = step_world_before.clone();
                        *ledger = step_ledger_before.clone();
                        *next_id = step_next_id;
                        if let (Some(states), Some(snapshot)) =
                            (polarity_states.as_deref_mut(), step_polarity_before.clone())
                        {
                            *states = snapshot;
                        }
                        ledger.computational_rejections += cohort.count;
                        ledger.rejected_steps += 1;
                        ledger.rollback_count += cohort.count;
                        ledger.numerical_invalid = true;
                        ledger.numerical_invalid_reason = Some(json!({
                            "phase": phase_index,
                            "step": step,
                            "cohort_id": cohort.id,
                            "error": "POLARITY_TRANSACTION_REJECTED",
                            "detail": error,
                        }));
                        lifecycle_event(
                            ledger,
                            "polarity",
                            step,
                            &cohort,
                            "ATOMIC_REJECTION_CAMPAIGN_INVALIDATED",
                            json!({"error": "POLARITY_TRANSACTION_REJECTED"}),
                        );
                        return false;
                    }
                }
            }
            let cadence_step = clock_mode.cadence_step(step);
            let topology_tick = cadence_step % 10 == 0;
            let structural_before_mechanics = cohort.mesh.total_structural_mass();
            // Snapshot only the state entering the mechanics transition.  The
            // reaction/growth ledgers above are already accepted for this
            // step; a rejected mechanics proposal must not roll those global
            // accounting entries back implicitly.
            let pre_mechanics = cohort.clone();
            let polarity_measures_before_mechanics = polarity_states
                .as_ref()
                .and_then(|states| states.get(&cohort.id))
                .map(|_| {
                    (0..cohort.mesh.n())
                        .map(|index| cohort.mesh.edge_length(index))
                        .collect::<Vec<_>>()
                });
            let mut pending_atomic_replay: Option<Value> = None;
            if atomic_replay_snapshots.is_some() && R6_ATOMIC_CHECKPOINTS.contains(&step) {
                let pre_polarity = polarity_states
                    .as_ref()
                    .and_then(|states| states.get(&cohort.id))
                    .cloned();
                let mut replay_cohort = pre_mechanics.clone();
                let replay_result =
                    r10_closure::r10_refractory_mechanics_step_with_polarity_diagnostics(
                        &mut replay_cohort.mesh,
                        replay_cohort
                            .plasticity
                            .as_mut()
                            .expect("R10 replay plasticity"),
                        topology_tick,
                        motor_enabled,
                        adaptation_enabled,
                        if polarity_actuator_connected {
                            polarity_activity.as_deref()
                        } else {
                            None
                        },
                    );
                let replay_polarity = match (
                    pre_polarity.clone(),
                    polarity_measures_before_mechanics.as_deref(),
                    replay_result.as_ref().map(|(_, _, mappings, _)| *mappings),
                ) {
                    (Some(mut state), Some(old_measures), Some(mappings)) if mappings > 0 => {
                        let new_measures = (0..replay_cohort.mesh.n())
                            .map(|index| replay_cohort.mesh.edge_length(index))
                            .collect::<Vec<_>>();
                        state.remap_conservative(old_measures, &new_measures).ok()
                    }
                    (state, _, _) => state,
                };
                let replay = match replay_result {
                    Some((active_a, active_w, remesh_mappings, diagnostic)) => json!({
                        "status": "ACCEPTED",
                        "active_a": active_a,
                        "active_w": active_w,
                        "remesh_mappings": remesh_mappings,
                        "diagnostic": diagnostic,
                        "post_cohort": serde_json::to_value(&replay_cohort).expect("serialize R6 replay cohort"),
                        "post_polarity": replay_polarity,
                    }),
                    None => json!({"status": "REJECTED"}),
                };

                let mut disconnected_cohort = pre_mechanics.clone();
                let disconnected_result =
                    r10_closure::r10_refractory_mechanics_step_with_polarity_diagnostics(
                        &mut disconnected_cohort.mesh,
                        disconnected_cohort
                            .plasticity
                            .as_mut()
                            .expect("R10 disconnected replay plasticity"),
                        topology_tick,
                        motor_enabled,
                        adaptation_enabled,
                        None,
                    );
                let same_state_disconnected = match disconnected_result {
                    Some((active_a, active_w, remesh_mappings, diagnostic)) => json!({
                        "status": "ACCEPTED",
                        "active_a": active_a,
                        "active_w": active_w,
                        "remesh_mappings": remesh_mappings,
                        "diagnostic": diagnostic,
                        "post_cohort": serde_json::to_value(&disconnected_cohort).expect("serialize R6 disconnected cohort"),
                    }),
                    None => json!({"status": "REJECTED"}),
                };
                pending_atomic_replay = Some(json!({
                    "checkpoint_step": step,
                    "accepted_steps_before": ledger.accepted_steps,
                    "accepted_time_before": ledger.accepted_steps as f64 * mechanics.dt,
                    "phase_index": phase_index,
                    "phase_step": phase_step + 1,
                    "cohort_id": cohort.id,
                    "campaign_seed": campaign_seed,
                    "next_id_before": *next_id,
                    "boundary": {
                        "target_n": boundary_target_n,
                        "target_f": boundary_target_f,
                        "accepted_bath_n": world.n_mass / world.volume,
                        "accepted_bath_f": world.f_mass / world.volume,
                    },
                    "configuration": {
                        "boundary_mode": boundary_mode.label(),
                        "clock_mode": clock_mode.label(),
                        "reserve_resolution": reserve_resolution.label(),
                        "polarity_actuator_connected": polarity_actuator_connected,
                        "motor_enabled": motor_enabled,
                        "adaptation_enabled": adaptation_enabled,
                        "topology_tick": topology_tick,
                    },
                    "pre_state": {
                        "cohort": serde_json::to_value(&pre_mechanics).expect("serialize R6 pre-mechanics cohort"),
                        "world": serde_json::to_value(&world).expect("serialize R6 pre-mechanics world"),
                        "ledger": serde_json::to_value(&ledger).expect("serialize R6 pre-mechanics ledger"),
                        "polarity": pre_polarity,
                        "polarity_activity": polarity_activity.clone(),
                        "polarity_measures": polarity_measures_before_mechanics.clone(),
                    },
                    "replay": replay,
                    "same_state_disconnected_replay": same_state_disconnected,
                    "response_operator": r6_local_response_probe(
                        &pre_mechanics.mesh,
                        pre_mechanics.plasticity.as_ref().expect("R6 plasticity"),
                        pre_polarity.as_ref(),
                        topology_tick,
                        motor_enabled,
                        adaptation_enabled,
                        polarity_actuator_connected,
                    ),
                }));
            }
            let Some((active_a, active_w, remesh_mappings, mechanics_diagnostic)) =
                r10_closure::r10_refractory_mechanics_step_with_polarity_route(
                    &mut cohort.mesh,
                    cohort.plasticity.as_mut().expect("R10 plasticity"),
                    topology_tick,
                    motor_enabled,
                    adaptation_enabled,
                    if polarity_actuator_connected {
                        polarity_activity.as_deref()
                    } else {
                        None
                    },
                    if r10_normal_remap_enabled() {
                        r10_closure::PolarityMechanicsRoute::R10InwardNormal
                    } else {
                        r10_closure::PolarityMechanicsRoute::R4EdgeTension
                    },
                )
            else {
                if r10r9r5_canonical_lifecycle() {
                    // A mechanics/remap rejection is a transaction failure,
                    // not a lawful organism outcome.  The whole accepted
                    // physical step is restored below and the campaign is
                    // invalidated; no selected cohort may advance past it.
                    cohort = pre_mechanics;
                    let rejected_count = cohort.count;
                    *cohorts = step_cohorts_before.clone();
                    *world = step_world_before.clone();
                    *ledger = step_ledger_before.clone();
                    *next_id = step_next_id;
                    if let (Some(states), Some(snapshot)) =
                        (polarity_states.as_deref_mut(), step_polarity_before.clone())
                    {
                        *states = snapshot;
                    }
                    ledger.rejected_steps += 1;
                    ledger.computational_rejections += rejected_count;
                    ledger.rollback_count += rejected_count;
                    ledger.numerical_invalid = true;
                    ledger.numerical_invalid_reason = Some(json!({
                        "phase": phase_index,
                        "step": step,
                        "cohort_id": cohort.id,
                        "error": "MECHANICS_OR_REMAP_REJECTED",
                    }));
                    lifecycle_event(
                        ledger,
                        "mechanics",
                        step,
                        &cohort,
                        "ATOMIC_REJECTION_CAMPAIGN_INVALIDATED",
                        json!({"error": "MECHANICS_OR_REMAP_REJECTED"}),
                    );
                    return false;
                } else {
                    ledger.runtime_invalidations += cohort.count;
                    ledger.invalid_geometry_events += cohort.count;
                    lifecycle_event(
                        ledger,
                        "mechanics",
                        step,
                        &cohort,
                        "LEGACY_COMPUTATIONAL_REMOVAL",
                        json!({"error": "MECHANICS_OR_REMAP_REJECTED"}),
                    );
                    invalidated_material_to_terminal(&cohort, world);
                }
                continue;
            };
            if remesh_mappings > 0 {
                let remap_result = polarity_measures_before_mechanics
                    .as_deref()
                    .ok_or_else(|| "POLARITY_REMAP_SOURCE_MISSING".to_string())
                    .and_then(|old_measures| {
                        let new_measures = (0..cohort.mesh.n())
                            .map(|index| cohort.mesh.edge_length(index))
                            .collect::<Vec<_>>();
                        let states = polarity_states
                            .as_deref_mut()
                            .ok_or_else(|| "POLARITY_REMAP_STATE_MISSING".to_string())?;
                        let state = states
                            .get(&cohort.id)
                            .ok_or_else(|| "POLARITY_STATE_MISSING".to_string())?;
                        let remapped = state
                            .remap_conservative(old_measures, &new_measures)
                            .map_err(|error| error.to_string())?;
                        states.insert(cohort.id, remapped);
                        Ok(())
                    });
                if let Err(error) = remap_result {
                    let rejected_count = cohort.count;
                    *cohorts = step_cohorts_before.clone();
                    *world = step_world_before.clone();
                    *ledger = step_ledger_before.clone();
                    *next_id = step_next_id;
                    if let (Some(states), Some(snapshot)) =
                        (polarity_states.as_deref_mut(), step_polarity_before.clone())
                    {
                        *states = snapshot;
                    }
                    ledger.rejected_steps += 1;
                    ledger.computational_rejections += rejected_count;
                    ledger.rollback_count += rejected_count;
                    ledger.numerical_invalid = true;
                    ledger.numerical_invalid_reason = Some(json!({
                        "phase": phase_index,
                        "step": step,
                        "cohort_id": cohort.id,
                        "error": "POLARITY_REMAP_REJECTED",
                        "detail": error,
                    }));
                    lifecycle_event(
                        ledger,
                        "polarity_remap",
                        step,
                        &cohort,
                        "ATOMIC_REJECTION_CAMPAIGN_INVALIDATED",
                        json!({"error": error}),
                    );
                    return false;
                }
            }
            ledger.mechanics_topology_structural_net +=
                (cohort.mesh.total_structural_mass() - structural_before_mechanics) * count;
            ledger.active_a_spent += active_a * count;
            ledger.active_w_produced += active_w * count;
            ledger.adaptation_remesh_mappings += remesh_mappings as u64 * cohort.count;
            {
                let observer = phenotype_ledger(ledger, genotype);
                observer.active_a_spent += active_a * count;
                observer.active_w_produced += active_w * count;
            }
            if let Some(mut capture) = pending_atomic_replay.take() {
                let actual_cohort =
                    serde_json::to_value(&cohort).expect("serialize R6 actual cohort");
                let actual_polarity = polarity_states
                    .as_ref()
                    .and_then(|states| states.get(&cohort.id))
                    .cloned();
                let actual_diagnostic = mechanics_diagnostic.clone();
                let actual = json!({
                    "active_a": active_a,
                    "active_w": active_w,
                    "remesh_mappings": remesh_mappings,
                    "diagnostic": actual_diagnostic,
                    "post_cohort": actual_cohort,
                    "post_polarity": actual_polarity,
                    "world": serde_json::to_value(&world).expect("serialize R6 actual world"),
                    "ledger": serde_json::to_value(&ledger).expect("serialize R6 actual ledger"),
                    "next_id_after": *next_id,
                    "accepted_steps_after": ledger.accepted_steps + 1,
                });
                let expected_key = if polarity_actuator_connected {
                    "replay"
                } else {
                    "same_state_disconnected_replay"
                };
                let expected = capture[expected_key].clone();
                let identity = expected["status"] == "ACCEPTED"
                    && expected["active_a"] == actual["active_a"]
                    && expected["active_w"] == actual["active_w"]
                    && expected["remesh_mappings"] == actual["remesh_mappings"]
                    && expected["diagnostic"] == actual["diagnostic"]
                    && expected["post_cohort"] == actual["post_cohort"]
                    && (expected["post_polarity"].is_null()
                        || expected["post_polarity"] == actual["post_polarity"]);
                capture["actual_state"] = actual;
                capture["identity"] = json!({
                    "expected_replay": expected_key,
                    "unperturbed_transition_exact": identity,
                    "world_unchanged_by_mechanics": capture["pre_state"]["world"]
                        == capture["actual_state"]["world"],
                    "next_id_unchanged": capture["next_id_before"] == capture["actual_state"]["next_id_after"],
                    "accepted_step_advances_once": capture["accepted_steps_before"].as_u64()
                        .map(|before| capture["actual_state"]["accepted_steps_after"] == before + 1)
                        .unwrap_or(false),
                });
                if let Some(records) = atomic_replay_snapshots.as_deref_mut() {
                    records.push(capture);
                }
            }
            if let Some(records) = mechanics_diagnostics.as_deref_mut() {
                if phase_step == 0 || (phase_step + 1) % 250 == 0 {
                    let mut row = mechanics_diagnostic;
                    row["step"] = json!(step);
                    row["phase_step"] = json!(phase_step + 1);
                    row["cohort_id"] = json!(cohort.id);
                    row["boundary_target_n"] = json!(boundary_target_n);
                    row["boundary_target_f"] = json!(boundary_target_f);
                    row["accepted_bath_n"] = json!(world.n_mass / world.volume);
                    row["accepted_bath_f"] = json!(world.f_mass / world.volume);
                    row["mass_eligible"] =
                        json!(cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass);
                    row["segment_geometry"] =
                        r10_closure::r10_segment_geometry_summary(&cohort.mesh, &fission);
                    let transport_delta = transport_before.get(&cohort.id).map(|before| {
                        [
                            cohort.mesh.interior.n * cohort.mesh.area() - before[0],
                            cohort.mesh.interior.f * cohort.mesh.area() - before[1],
                        ]
                    });
                    row["material_geometry"] = architecture_geometry_record(
                        &pre_mechanics.mesh,
                        &cohort.mesh,
                        &row,
                        &fission,
                        &mechanics,
                        transport_delta,
                        expression_by_cohort.get(&cohort.id).copied(),
                        &reactions,
                        &grown,
                        active_a,
                        active_w,
                        step,
                    );
                    row["polarity"] = polarity_observation.clone().unwrap_or(Value::Null);
                    records.push(row);
                }
            }
            if !polygon_simple(&cohort.mesh.vertices)
                || !cohort.mesh.physical_runtime_valid()
                || !cohort.mesh.lifecycle_invariants_hold()
            {
                if r10r9r5_canonical_lifecycle() {
                    let rejected_count = cohort.count;
                    let detail = json!({
                        "simple": polygon_simple(&cohort.mesh.vertices),
                        "runtime_valid": cohort.mesh.physical_runtime_valid(),
                        "lifecycle_valid": cohort.mesh.lifecycle_invariants_hold(),
                    });
                    *cohorts = step_cohorts_before.clone();
                    *world = step_world_before.clone();
                    *ledger = step_ledger_before.clone();
                    *next_id = step_next_id;
                    if let (Some(states), Some(snapshot)) =
                        (polarity_states.as_deref_mut(), step_polarity_before.clone())
                    {
                        *states = snapshot;
                    }
                    ledger.computational_rejections += rejected_count;
                    ledger.rejected_steps += 1;
                    ledger.rollback_count += rejected_count;
                    ledger.numerical_invalid = true;
                    ledger.numerical_invalid_reason = Some(json!({
                        "phase": phase_index,
                        "step": step,
                        "cohort_id": cohort.id,
                        "error": "INVALID_POST_MECHANICS_STATE",
                        "detail": detail.clone(),
                    }));
                    lifecycle_event(
                        ledger,
                        "post_mechanics_validation",
                        step,
                        &cohort,
                        "ATOMIC_REJECTION_CAMPAIGN_INVALIDATED",
                        detail,
                    );
                    return false;
                } else {
                    ledger.runtime_invalidations += cohort.count;
                    ledger.invalid_geometry_events += cohort.count;
                    lifecycle_event(
                        ledger,
                        "post_mechanics_validation",
                        step,
                        &cohort,
                        "LEGACY_COMPUTATIONAL_REMOVAL",
                        json!({"error": "INVALID_GEOMETRY_OR_STATE"}),
                    );
                    invalidated_material_to_terminal(&cohort, world);
                }
                continue;
            }
            if cohort.mesh.uses_observer_only_death() {
                if r10r9r5_canonical_lifecycle() {
                    // Record the label for audit only.  It must not alter
                    // physical death, fission eligibility, or continuation.
                    if !cohort.mesh.observer_viable() {
                        ledger.observer_nonviable_observations += cohort.count;
                        lifecycle_event(
                            ledger,
                            "observer",
                            step,
                            &cohort,
                            "OBSERVER_NONVIABLE_NOT_AUTHORITATIVE",
                            json!({"reason": cohort.mesh.observer_death_reason()}),
                        );
                    }
                } else if !cohort.mesh.observer_viable() {
                    ledger.physical_deaths += cohort.count;
                    *ledger
                        .deaths_by_genotype
                        .entry(genotype_key(genotype))
                        .or_default() += cohort.count;
                    phenotype_ledger(ledger, genotype).physical_deaths += cohort.count;
                    dead_material_to_sink(&cohort, world);
                    continue;
                }
            }
            if !cohort.mesh.uses_observer_only_death() && !cohort.mesh.alive {
                ledger.physical_deaths += cohort.count;
                *ledger
                    .deaths_by_genotype
                    .entry(genotype_key(genotype))
                    .or_default() += cohort.count;
                phenotype_ledger(ledger, genotype).physical_deaths += cohort.count;
                lifecycle_event(
                    ledger,
                    "physical_death",
                    step,
                    &cohort,
                    "PHYSICAL_DEATH",
                    json!({"reason": cohort.mesh.death_reason}),
                );
                dead_material_to_sink(&cohort, world);
                continue;
            }
            let eligible = cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass
                && cadence_step % 25 == 0;
            if eligible {
                phenotype_ledger(ledger, genotype).fission_attempts += cohort.count;
                match r10_split_cohort(
                    cohort,
                    mutation_enabled,
                    campaign_seed,
                    step,
                    next_id,
                    &allocation,
                    &fission,
                    ledger,
                    Some(daughter_continuations),
                    expression_path == D096ExpressionPath::V4FiniteBudgetCentered,
                    polarity_states.as_deref_mut(),
                ) {
                    Ok(children) => {
                        first_fission_this_step = true;
                        survivors.extend(children)
                    }
                    Err(parent) => survivors.push(parent),
                }
            } else {
                survivors.push(cohort);
            }
        }
        *cohorts = survivors;
        ledger.accepted_steps += 1;
        if step == R10_PREFIX_STEPS && prefix_2500.is_none() {
            *prefix_2500 = Some(prefix_state(cohorts, world, ledger, step));
        }
        if phase_step == 0 || (phase_step + 1) % 250 == 0 {
            trajectory.push(snapshot(cohorts, world, step));
        }
        if stop_after_first_fission && first_fission_this_step {
            return true;
        }
    }
    if let Some(records) = full_cycle_snapshots.as_deref_mut() {
        for (_, mut record) in pending_full_cycle {
            record["actual_next_boundary"] = Value::Null;
            record["identity"] = json!({
                "same_phase_boundary_exact": false,
                "status": "MISSING_NEXT_BOUNDARY",
            });
            records.push(record);
        }
    }
    true
}

fn r10_campaign(
    template: &MaterialMesh,
    template_plasticity: &PlasticityStateV1,
    original_birth_mass: f64,
    template_step: usize,
    sequence: &[Environment],
    mutation_enabled: bool,
    replicate: u64,
    phase_steps: usize,
    expression_path: D096ExpressionPath,
    founder_multiplicity: u64,
    bath_volume: f64,
    boundary_mode: PopulationBoundaryMode,
    forced_genotype: Option<AllocationGenotype>,
) -> Value {
    let campaign_seed = splitmix64(
        replicate
            ^ if mutation_enabled {
                0x6a09_e667_f3bc_c909
            } else {
                0xbb67_ae85_84ca_a73b
            }
            ^ sequence.iter().fold(0_u64, |state, env| {
                state.rotate_left(9)
                    ^ match env {
                        Environment::Resource => 0x11,
                        Environment::Damage => 0x22,
                    }
            }),
    );
    let mut ledger = CampaignLedger::default();
    let mut cohorts = r10_initial_population(
        template,
        template_plasticity,
        original_birth_mass,
        mutation_enabled,
        campaign_seed,
        founder_multiplicity,
        &mut ledger,
        expression_path,
    );
    if let Some(genotype) = forced_genotype {
        for cohort in &mut cohorts {
            cohort
                .mesh
                .finite_allocation
                .as_mut()
                .expect("R10 allocation")
                .genotype = genotype;
        }
    }
    let mut next_id = 10_000;
    let mut world = OpenMedium::new(sequence[0], bath_volume);
    let initial_organism_n = organism_amount(&cohorts, 'n');
    let initial_organism_f = organism_amount(&cohorts, 'f');
    let initial_structural_mass = cohorts
        .iter()
        .map(|cohort| cohort.mesh.total_structural_mass() * cohort.count as f64)
        .sum::<f64>();
    let initial_catalyst_mass = cohorts
        .iter()
        .map(|cohort| {
            cohort
                .mesh
                .finite_allocation
                .map(|state| state.catalysts.iter().sum::<f64>())
                .unwrap_or(0.0)
                * cohort.count as f64
        })
        .sum::<f64>();
    let initial = snapshot(&cohorts, &world, 0);
    let mut trajectory = vec![initial.clone()];
    let mut prefix_2500 = None;
    let mut daughter_continuations = Vec::new();
    for (phase, environment) in sequence.iter().copied().enumerate() {
        let phase_ok = r10_advance_phase(
            &mut cohorts,
            &mut world,
            environment,
            phase,
            mutation_enabled,
            campaign_seed,
            &mut next_id,
            &mut ledger,
            &mut trajectory,
            &mut prefix_2500,
            phase_steps,
            expression_path,
            boundary_mode,
            FissionClockMode::CurrentStep,
            ReserveResolutionMode::PerStep,
            &mut daughter_continuations,
            false,
            DiagnosticBoundarySource::Scheduled,
            DiagnosticBoundarySource::Scheduled,
            true,
            true,
            GrowthPlacementMode::FrozenD088,
            None,
            None,
            false,
            None,
            None,
        );
        if !phase_ok {
            break;
        }
    }
    let terminal_step = ledger.accepted_steps as usize;
    let terminal = snapshot(&cohorts, &world, terminal_step);
    for cohort in &cohorts {
        let genotype = cohort
            .mesh
            .finite_allocation
            .expect("R10 allocation")
            .genotype;
        phenotype_ledger(&mut ledger, genotype).terminal_multiplicity += cohort.count;
    }
    let terminal_cohorts = terminal_cohort_diagnostics(&cohorts, &FissionParams::default());
    let terminal_organism_n = organism_amount(&cohorts, 'n');
    let terminal_organism_f = organism_amount(&cohorts, 'f');
    let terminal_structural_mass = cohorts
        .iter()
        .map(|cohort| cohort.mesh.total_structural_mass() * cohort.count as f64)
        .sum::<f64>();
    let terminal_catalyst_mass = cohorts
        .iter()
        .map(|cohort| {
            cohort
                .mesh
                .finite_allocation
                .map(|state| state.catalysts.iter().sum::<f64>())
                .unwrap_or(0.0)
                * cohort.count as f64
        })
        .sum::<f64>();
    let n_closure =
        (world.ledger.initial_n + world.ledger.external_source_n_to_bath + initial_organism_n
            - world.n_mass
            - world.ledger.bath_n_to_outflow
            - terminal_organism_n
            - ledger.reaction_n_consumed
            - world.ledger.physical_death_n_sink
            - world.ledger.invalidated_n_terminal)
            .abs();
    let f_closure =
        (world.ledger.initial_f + world.ledger.external_source_f_to_bath + initial_organism_f
            - world.f_mass
            - world.ledger.bath_f_to_outflow
            - terminal_organism_f
            - ledger.reaction_f_consumed
            - world.ledger.physical_death_f_sink
            - world.ledger.invalidated_f_terminal)
            .abs();
    let active_energy_residual = (ledger.active_a_spent - ledger.active_w_produced).abs();
    let structural_expected_terminal = initial_structural_mass
        + ledger.m1_structural_build
        + ledger.growth_material
        + ledger.mechanics_topology_structural_net
        + ledger.post_bootstrap_fission_closure_structural_input
        - ledger.expression_material
        - ledger.m1_structural_turnover
        - world.ledger.damage_structural_sink
        - world.ledger.physical_death_structural_sink
        - world.ledger.invalidated_structural_terminal;
    let structural_closure_residual =
        (structural_expected_terminal - terminal_structural_mass).abs();
    let structural_budget = json!({
        "initial_structural_mass": initial_structural_mass,
        "d096_structural_material_converted_to_catalysts": ledger.expression_material,
        "ordinary_m1_structural_build": ledger.m1_structural_build,
        "ordinary_m1_structural_turnover": ledger.m1_structural_turnover,
        "surplus_growth_structural_production": ledger.growth_material,
        "damage_structural_loss": world.ledger.damage_structural_sink,
        "physical_death_structural_sink": world.ledger.physical_death_structural_sink,
        "invalidated_structural_terminal": world.ledger.invalidated_structural_terminal,
        "rupture_rebond_and_remesh_net": ledger.mechanics_topology_structural_net,
        "bootstrap_fission_closure_material_before_budget_start": ledger.bootstrap_fission_closure_structural_input,
        "post_bootstrap_fission_closure_material": ledger.post_bootstrap_fission_closure_structural_input,
        "terminal_structural_mass": terminal_structural_mass,
        "closure_residual": structural_closure_residual,
        "initial_allocation_catalyst_mass": initial_catalyst_mass,
        "terminal_allocation_catalyst_mass": terminal_catalyst_mass,
        "allocation_catalyst_turnover": ledger.expression_turnover_waste,
        "catalyst_precursor_a": ledger.expression_catalyst_precursor_a,
        "d096_activation_and_maintenance_a": ledger.expression_activation,
        "growth_a_consumed": ledger.growth_a_consumed,
        "active_motor_a": ledger.active_a_spent,
        "active_motor_w": ledger.active_w_produced,
    });
    json!({
        "replicate": replicate,
        "mutation_enabled": mutation_enabled,
        "campaign_seed": campaign_seed,
        "environment_sequence": sequence.iter().map(|environment| environment.label()).collect::<Vec<_>>(),
        "phase_steps": phase_steps,
        "population_boundary_mode": boundary_mode.label(),
        "d091_reserve_enabled": r10r9r1_reserve_enabled(),
        "d091_configuration": if r10r9r1_reserve_enabled() {
            serde_json::to_value(r10r9r1_reaction_params(template).reserve).unwrap()
        } else {
            serde_json::Value::Null
        },
        "expression_path": expression_path.label(),
        "founder_multiplicity": founder_multiplicity,
        "template_fission_step": template_step,
        "fitness_function": null,
        "breeder_selection": false,
        "population_cap": null,
        "resource_feedback": false,
        "exchangeability_compression": true,
        "initial": initial,
        "trajectory": trajectory,
        "prefix_2500": prefix_2500,
        "terminal": terminal,
        "terminal_cohorts": terminal_cohorts,
        "daughter_continuations": daughter_continuations,
        "world": world,
        "ledger": ledger,
        "structural_budget": structural_budget,
        "n_closure_residual": n_closure,
        "f_closure_residual": f_closure,
        "active_energy_residual": active_energy_residual,
        "accepted_steps": ledger.accepted_steps,
        "rejected_steps": ledger.rejected_steps,
        "numerical_invalid": ledger.numerical_invalid,
        "numerical_invalid_reason": ledger.numerical_invalid_reason,
    })
}

fn run_r10_evolution_with_horizon(
    default_output: &str,
    directive: &str,
    phase_steps: usize,
    expression_path: D096ExpressionPath,
    boundary_mode: PopulationBoundaryMode,
    founder_multiplicity: u64,
    bath_volume: f64,
) {
    let mut output = PathBuf::from(default_output);
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let (template, plasticity, original_birth_mass, template_step) =
        r10_closure::r10_seed3_fission_state();
    let mut handles = Vec::new();
    let mutation_modes: Vec<bool> = if r10r9r4_mutation_off_only() {
        vec![false]
    } else {
        vec![true, false]
    };
    for replicate in 1..=REPLICATES {
        for mutation_enabled in mutation_modes.iter().copied() {
            for sequence in [
                vec![Environment::Resource],
                vec![Environment::Damage],
                vec![Environment::Resource, Environment::Damage],
            ] {
                let template = template.clone();
                let plasticity = plasticity.clone();
                handles.push(std::thread::spawn(move || {
                    r10_campaign(
                        &template,
                        &plasticity,
                        original_birth_mass,
                        template_step,
                        &sequence,
                        mutation_enabled,
                        replicate,
                        phase_steps,
                        expression_path,
                        founder_multiplicity,
                        bath_volume,
                        boundary_mode,
                        None,
                    )
                }));
            }
        }
    }
    let campaigns = handles
        .into_iter()
        .map(|handle| handle.join().expect("R10 evolution campaign"))
        .collect::<Vec<_>>();
    let value = json!({
        "directive": directive,
        "execution_identity": {
            "kernel": "R10_CANONICAL_POPULATION_STEP_V1",
            "kernel_contract": "D096 expression -> finite exchange -> reactions/growth -> R9/R10 mechanics -> accepted-step refractory commit -> remesh/topology -> physical fate -> R10 fission",
            "lifecycle_mode": if r10r9r5_canonical_lifecycle() { "CANONICAL_RETAIN_AND_ROLLBACK" } else { "LEGACY_COMPUTATIONAL_REMOVAL_CONTROL" },
            "organism_biology_source": "digital-protocell/examples/dcfinal001_r4_evolution.rs::r10_advance_phase",
            "mechanics_source": "digital-protocell/examples/dcfinal001_r5_v4_neck.rs::r10_refractory_mechanics_step",
            "fission_source": "chemistry-core/src/mesh_fission.rs::R10 signed segment scission",
            "configuration_digest": deterministic_state_digest(&json!({
                "expression_path": expression_path.label(),
                "boundary_mode": boundary_mode.label(),
                "founder_multiplicity": founder_multiplicity,
                "bath_volume": bath_volume,
                "phase_steps": phase_steps,
                "replicates": REPLICATES,
                "mutation_probability": AllocationParams::default().mutation_probability,
                "mutation_sigma": AllocationParams::default().mutation_sigma,
                "canonical_lifecycle": r10r9r5_canonical_lifecycle(),
            })),
        },
        "protocol": {
            "body": "MaturationCoupledV4 + R8 closure + R8R1 sign-aware mechanics + R9 refractory curvature-normal + R10 signed scission stress",
            "mutation_probability": AllocationParams::default().mutation_probability,
            "mutation_sigma": AllocationParams::default().mutation_sigma,
            "mutation_semantics": "one deterministic blind draw per daughter at geometry-valid physical fission",
            "minimum_opportunities_for_95_percent_at_least_one": 299,
            "founder_multiplicity": founder_multiplicity,
            "opportunities_per_initial_campaign": 2 * founder_multiplicity,
            "replicates": REPLICATES,
            "mutation_off_only": r10r9r4_mutation_off_only(),
            "phase_steps": phase_steps,
            "population_boundary_mode": boundary_mode.label(),
            "switch_schedule": [phase_steps, 2 * phase_steps],
            "open_medium_volume": bath_volume,
            "inflow_schedule_source": "frozen D-096 Resource/Damage values",
            "component_3_boundary": "reserve-only endpoint dormant under frozen reserve-OFF physiology",
        },
        "template": {
            "r10_arm": "seed_3_c_0.08",
            "fission_step": template_step,
            "simple": polygon_simple(&template.vertices),
            "vertices": template.n(),
            "mass": template.total_structural_mass(),
            "original_birth_mass": original_birth_mass,
            "contract_version": format!("{:?}", template.contract_version),
            "plasticity_patches": plasticity.adaptation.len(),
        },
        "fixed_boundary_transport_reference_parity": fixed_boundary_transport_reference_parity(),
        "campaigns": campaigns,
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

pub fn run_r10r7_variant_feasibility(default_output: &str, panel_path: &str) {
    let panel: Vec<AllocationGenotype> =
        serde_json::from_slice(&fs::read(panel_path).expect("R10R7 natural-variant panel"))
            .expect("valid R10R7 natural-variant panel");
    let (template, plasticity, original_birth_mass, template_step) =
        r10_closure::r10_seed3_fission_state();
    let mut jobs = Vec::new();
    for (index, genotype) in panel.iter().copied().enumerate() {
        for environment in [Environment::Resource, Environment::Damage] {
            let template = template.clone();
            let plasticity = plasticity.clone();
            jobs.push(std::thread::spawn(move || {
                r10_campaign(
                    &template,
                    &plasticity,
                    original_birth_mass,
                    template_step,
                    &[environment],
                    false,
                    10_000 + index as u64,
                    R10R2_PHASE_STEPS,
                    D096ExpressionPath::V4FiniteBudgetCentered,
                    FOUNDER_MULTIPLICITY,
                    FOUNDER_MULTIPLICITY as f64,
                    PopulationBoundaryMode::FixedConcentrationBoundary,
                    Some(genotype),
                )
            }));
        }
    }
    let campaigns = jobs
        .into_iter()
        .map(|job| job.join().expect("R10R7 feasibility campaign"))
        .collect::<Vec<_>>();
    fs::write(
        default_output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R10R7-NATURAL-VARIANT-CROSS-ENVIRONMENT-FEASIBILITY-TWO-WINDOW-SELECTION-REVERSAL-AND-END-GOAL-CLOSURE-001",
            "panel": panel,
            "phase_steps": R10R2_PHASE_STEPS,
            "population_boundary_mode": PopulationBoundaryMode::FixedConcentrationBoundary.label(),
            "campaigns": campaigns,
        }))
        .unwrap(),
    )
    .unwrap();
}

pub fn run_r10_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10_evolution.json",
        "DC-FINAL-001-R10-SIGNED-LOAD-BEARING-NECK-STRESS-REPRODUCTION-AND-END-GOAL-CLOSURE-001",
        PHASE_STEPS,
        D096ExpressionPath::V1Structural,
        PopulationBoundaryMode::RateReinterpretation,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

pub fn run_r10r2_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r2_evolution.json",
        "DC-FINAL-001-R10R2-PRODUCTION-EVOLUTION-HORIZON-REQUALIFICATION-SELECTION-AND-END-GOAL-CLOSURE-001",
        R10R2_PHASE_STEPS,
        D096ExpressionPath::V1Structural,
        PopulationBoundaryMode::RateReinterpretation,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

pub fn run_r10r5_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r5_evolution.json",
        "DC-FINAL-001-R10R5-D096-FINITE-BUDGET-CENTERED-GAIN-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001",
        R10R2_PHASE_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::RateReinterpretation,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

/// Requalify the composed organism through the same repaired production step
/// used by the population harness.  This deliberately uses one organism per
/// frozen perturbation arm, so it is a bounded kernel/reproduction check and
/// not the ecological population campaign.
pub fn run_r10r9r5_shared_kernel_reproduction() {
    let output = PathBuf::from("/tmp/dcfinal001_r10r9r5_shared_kernel_reproduction.json");
    let allocation = AllocationParams::default();
    let mut arms = Vec::new();
    for index in 0..10 {
        let mut mesh = r10_closure::r10_reproduction_fixture(index);
        let birth_mass = mesh.total_structural_mass();
        mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
        let cohort = Cohort {
            plasticity: Some(PlasticityStateV1::new(mesh.n())),
            mesh,
            count: 1,
            generation: 0,
            birth_mass,
            id: 1,
        };
        let mut cohorts = vec![cohort];
        let mut world = OpenMedium::new(Environment::Resource, R10R9R4_FOUNDER_MULTIPLICITY as f64);
        let initial = snapshot(&cohorts, &world, 0);
        let mut ledger = CampaignLedger::default();
        let mut next_id = 2;
        let mut trajectory = vec![initial];
        let mut prefix = None;
        let mut daughter_continuations = Vec::new();
        let accepted = r10_advance_phase(
            &mut cohorts,
            &mut world,
            Environment::Resource,
            0,
            false,
            splitmix64((index + 1) as u64),
            &mut next_id,
            &mut ledger,
            &mut trajectory,
            &mut prefix,
            14_778,
            D096ExpressionPath::V4FiniteBudgetCentered,
            PopulationBoundaryMode::FixedConcentrationBoundary,
            FissionClockMode::CurrentStep,
            ReserveResolutionMode::PerStep,
            &mut daughter_continuations,
            true,
            DiagnosticBoundarySource::Scheduled,
            DiagnosticBoundarySource::Scheduled,
            true,
            true,
            GrowthPlacementMode::FrozenD088,
            None,
            None,
            false,
            None,
            None,
        );
        let terminal = snapshot(&cohorts, &world, ledger.accepted_steps as usize);
        let terminal_structural_mass = cohorts
            .iter()
            .map(|cohort| cohort.mesh.total_structural_mass() * cohort.count as f64)
            .sum::<f64>();
        arms.push(json!({
            "arm": index + 1,
            "accepted": accepted,
            "accepted_steps": ledger.accepted_steps,
            "birth_mass": birth_mass,
            "terminal_structural_mass": terminal_structural_mass,
            "rejected_steps": ledger.rejected_steps,
            "numerical_invalid": ledger.numerical_invalid,
            "physical_fissions": ledger.physical_fissions,
            "valid_simple_fissions": ledger.valid_simple_fissions,
            "full_state_daughter_continuation": if daughter_continuations.is_empty() {
                Value::Null
            } else {
                Value::Bool(daughter_continuations.iter().all(|record| {
                    record["continuation"]["viable"] == true
                }))
            },
            "daughter_continuations": daughter_continuations,
            "post_bootstrap_physical_fissions": ledger.post_bootstrap_physical_fissions,
            "mutation_opportunities": ledger.mutation_opportunities,
            "terminal": terminal,
            "world": world,
            "ledger": ledger,
        }));
    }
    let fissions = arms
        .iter()
        .map(|arm| arm["physical_fissions"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let viable = arms
        .iter()
        .filter(|arm| arm["full_state_daughter_continuation"] == true)
        .count();
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R10R9R5-CANONICAL-LIFECYCLE-AND-EVOLUTION-EVIDENCE-RECOVERY-001",
            "kernel": "R10_CANONICAL_POPULATION_STEP_V1",
            "configuration": {
                "expression_path": D096ExpressionPath::V4FiniteBudgetCentered.label(),
                "boundary_mode": PopulationBoundaryMode::FixedConcentrationBoundary.label(),
                "phase_steps": 14_778,
                "mutation_enabled": false,
                "founder_multiplicity": 1,
                "bath_volume": R10R9R4_FOUNDER_MULTIPLICITY,
            },
            "arms": arms,
            "counts": {
                "terminal_mass_qualified": arms.iter().filter(|arm| {
                    arm["terminal_structural_mass"].as_f64().unwrap_or(0.0)
                        >= 1.35 * arm["birth_mass"].as_f64().unwrap_or(f64::INFINITY)
                }).count(),
                "physical_fissions": fissions,
                "full_state_daughter_continuations": viable,
            },
            "biology_delta": 0,
            "population_campaign": "NOT_RUN",
        }))
        .unwrap(),
    )
    .unwrap();
}

const R4_POLARITY_INITIAL_TOTAL_CONCENTRATION: f64 =
    regulatory_core::R3_TOTAL_POLARITY_CONCENTRATION;
const R4_POLARITY_INITIAL_SIGNAL_SENSITIVITY: f64 = 0.02;
const R4_POLARITY_INITIAL_INACTIVE_MODE_RATIO: f64 = -0.15575423416695056;

/// Source-fund the accepted R3 polarity state from existing organism A and a
/// local young-structure history signal.  The source debit is committed to
/// the organism's A amount before the first accepted transition; polarity
/// material is the resulting physical state, not a hidden free reservoir.
fn initialize_r4_polarity(mesh: &mut MaterialMesh) -> Result<(PolarityMassStateV1, Value), String> {
    let actuation =
        PolarityActuationParamsV1::sealed_r3_candidate().map_err(|error| error.to_string())?;
    let n = mesh.n();
    let measures = (0..n)
        .map(|index| mesh.edge_length(index))
        .collect::<Vec<_>>();
    let active_equilibrium = actuation.reference_active_concentration;
    let mut active = Vec::with_capacity(n);
    let mut inactive = Vec::with_capacity(n);
    for (index, measure) in measures.iter().enumerate() {
        let young_density = mesh.young_structural_mass(index) / measure;
        let delta = R4_POLARITY_INITIAL_SIGNAL_SENSITIVITY * (young_density - 1.0);
        active.push(
            (active_equilibrium + delta)
                .max(0.0)
                .min(R4_POLARITY_INITIAL_TOTAL_CONCENTRATION),
        );
        inactive.push(
            (R4_POLARITY_INITIAL_TOTAL_CONCENTRATION - active_equilibrium
                + R4_POLARITY_INITIAL_INACTIVE_MODE_RATIO * delta)
                .max(0.0),
        );
    }
    let area = mesh.area();
    let source_a_available = mesh.interior.a.max(0.0) * area;
    let (state, source) = PolarityMassStateV1::from_source_funded_concentrations(
        &measures,
        &active,
        &inactive,
        source_a_available,
        &PolarityMassParamsV1::candidate(),
    )
    .map_err(|error| error.to_string())?;
    mesh.interior.a = (source_a_available - source.source_a_debited) / area;
    Ok((
        state,
        json!({
            "source_ledger": source,
            "initial_active_concentrations": active,
            "initial_inactive_concentrations": inactive,
            "initial_measures": measures,
            "a_after_source_conversion": mesh.interior.a,
            "polarity_initialization": "existing_A_to_R3_amounts_with_local_young_structure_signal",
        }),
    ))
}

// R6-only local response instrumentation.  This computes a finite-difference
// response of the already-authorized pre-mechanics transition.  It is kept in
// the observer path and returns summaries only; no probe result is consulted
// by the production transition.
fn r6_orthogonalized_basis(raw: Vec<Vec<f64>>, constraints: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let mut orthogonal_constraints = Vec::new();
    for mut vector in constraints.iter().cloned() {
        for previous in &orthogonal_constraints {
            let projection = vector
                .iter()
                .zip(previous)
                .map(|(left, right)| left * right)
                .sum::<f64>();
            for (value, basis_value) in vector.iter_mut().zip(previous) {
                *value -= projection * basis_value;
            }
        }
        let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
        if norm > 1.0e-12 {
            vector.iter_mut().for_each(|value| *value /= norm);
            orthogonal_constraints.push(vector);
        }
    }
    let mut basis = Vec::new();
    for mut vector in raw {
        for previous in orthogonal_constraints.iter().chain(&basis) {
            let projection = vector
                .iter()
                .zip(previous)
                .map(|(left, right)| left * right)
                .sum::<f64>();
            for (value, basis_value) in vector.iter_mut().zip(previous) {
                *value -= projection * basis_value;
            }
        }
        let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
        if norm > 1.0e-10 {
            vector.iter_mut().for_each(|value| *value /= norm);
            basis.push(vector);
        }
    }
    basis
}

fn r6_geometry_basis(mesh: &MaterialMesh) -> Vec<Vec<f64>> {
    let n = mesh.n();
    let centroid = mesh.centroid();
    let mut translation_x = vec![0.0; 2 * n];
    let mut translation_y = vec![0.0; 2 * n];
    let mut rotation = vec![0.0; 2 * n];
    for (index, point) in mesh.vertices.iter().enumerate() {
        translation_x[2 * index] = 1.0;
        translation_y[2 * index + 1] = 1.0;
        rotation[2 * index] = -(point[1] - centroid[1]);
        rotation[2 * index + 1] = point[0] - centroid[0];
    }
    let raw = (0..2 * n)
        .map(|index| {
            let mut vector = vec![0.0; 2 * n];
            vector[index] = 1.0;
            vector
        })
        .collect::<Vec<_>>();
    r6_orthogonalized_basis(raw, &[translation_x, translation_y, rotation])
}

fn r6_zero_sum_basis(n: usize) -> Vec<Vec<f64>> {
    let uniform = vec![1.0; n];
    let raw = (0..n)
        .map(|index| {
            let mut vector = vec![0.0; n];
            vector[index] = 1.0;
            vector
        })
        .collect::<Vec<_>>();
    r6_orthogonalized_basis(raw, &[uniform])
}

fn r6_centered_geometry_output(mesh: &MaterialMesh) -> Vec<f64> {
    let centroid = mesh.centroid();
    mesh.vertices
        .iter()
        .flat_map(|point| [point[0] - centroid[0], point[1] - centroid[1]])
        .collect()
}

fn r6_response_output(
    mesh: &MaterialMesh,
    polarity: Option<&PolarityMassStateV1>,
    plasticity: &PlasticityStateV1,
) -> Vec<f64> {
    let mut output = r6_centered_geometry_output(mesh);
    if let Some(state) = polarity {
        output.extend(state.active_amount.iter().copied());
        output.extend(state.inactive_amount.iter().copied());
    } else {
        output.extend(std::iter::repeat_n(0.0, 2 * mesh.n()));
    }
    output.extend(plasticity.adaptation.iter().copied());
    output
}

#[derive(Debug, Clone, Copy)]
enum R6ResponseChannel {
    Geometry,
    ActivePolarity,
    InactivePolarity,
    Adaptation,
}

fn r6_probe_transition(
    source_mesh: &MaterialMesh,
    source_plasticity: &PlasticityStateV1,
    source_polarity: Option<&PolarityMassStateV1>,
    geometry_delta: Option<&[f64]>,
    polarity_delta: Option<(&[f64], R6ResponseChannel)>,
    adaptation_delta: Option<&[f64]>,
    amplitude: f64,
    sign: f64,
    topology_tick: bool,
    motor_enabled: bool,
    adaptation_enabled: bool,
    connected: bool,
) -> Option<Vec<f64>> {
    let mut mesh = source_mesh.clone();
    let mut plasticity = source_plasticity.clone();
    let mut polarity = source_polarity.cloned();
    if let Some(delta) = geometry_delta {
        for (index, value) in delta.chunks_exact(2).enumerate() {
            mesh.vertices[index][0] += sign * amplitude * value[0];
            mesh.vertices[index][1] += sign * amplitude * value[1];
        }
    }
    if let Some((delta, channel)) = polarity_delta {
        let state = polarity.as_mut()?;
        let target = match channel {
            R6ResponseChannel::ActivePolarity => &mut state.active_amount,
            R6ResponseChannel::InactivePolarity => &mut state.inactive_amount,
            _ => return None,
        };
        for (value, increment) in target.iter_mut().zip(delta) {
            *value += sign * amplitude * increment;
        }
        if target
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return None;
        }
    }
    if let Some(delta) = adaptation_delta {
        for (value, increment) in plasticity.adaptation.iter_mut().zip(delta) {
            *value += sign * amplitude * increment;
        }
        if plasticity
            .adaptation
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return None;
        }
    }
    if !polygon_simple(&mesh.vertices) || !mesh.physical_runtime_valid() {
        return None;
    }
    let activity = if connected {
        let state = polarity.as_ref()?;
        let measures = (0..mesh.n())
            .map(|index| mesh.edge_length(index))
            .collect::<Vec<_>>();
        let params = PolarityActuationParamsV1::sealed_r3_candidate().ok()?;
        Some(
            derive_local_activity(state, &measures, &params)
                .ok()?
                .vertex_activity,
        )
    } else {
        None
    };
    let (_, _, remesh_mappings, _) =
        r10_closure::r10_refractory_mechanics_step_with_polarity_diagnostics(
            &mut mesh,
            &mut plasticity,
            topology_tick,
            motor_enabled,
            adaptation_enabled,
            activity.as_deref(),
        )?;
    if remesh_mappings > 0 || mesh.n() != source_mesh.n() {
        return None;
    }
    Some(r6_response_output(&mesh, polarity.as_ref(), &plasticity))
}

fn r6_local_response_probe(
    mesh: &MaterialMesh,
    plasticity: &PlasticityStateV1,
    polarity: Option<&PolarityMassStateV1>,
    topology_tick: bool,
    motor_enabled: bool,
    adaptation_enabled: bool,
    connected: bool,
) -> Value {
    let n = mesh.n();
    let geometry_basis = r6_geometry_basis(mesh);
    let polarity_basis = r6_zero_sum_basis(n);
    let adaptation_basis = (0..n)
        .map(|index| {
            let mut vector = vec![0.0; n];
            vector[index] = 1.0;
            vector
        })
        .collect::<Vec<_>>();
    let mean_edge = (0..n).map(|index| mesh.edge_length(index)).sum::<f64>() / n.max(1) as f64;
    let active_rms = polarity
        .map(|state| {
            (state
                .active_amount
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                / n.max(1) as f64)
                .sqrt()
        })
        .unwrap_or(0.0);
    let inactive_rms = polarity
        .map(|state| {
            (state
                .inactive_amount
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                / n.max(1) as f64)
                .sqrt()
        })
        .unwrap_or(0.0);
    let h = f64::EPSILON.cbrt();
    let scale_factors = [4.0, 2.0, 1.0];
    let geometry_dimension = geometry_basis.len();
    let polarity_dimension = polarity_basis.len();
    let mut channels = Vec::new();
    for (channel, basis, normalization) in [
        (R6ResponseChannel::Geometry, geometry_basis, mean_edge),
        (
            R6ResponseChannel::ActivePolarity,
            polarity_basis.clone(),
            active_rms,
        ),
        (
            R6ResponseChannel::InactivePolarity,
            polarity_basis,
            inactive_rms,
        ),
        (R6ResponseChannel::Adaptation, adaptation_basis, 1.0),
    ] {
        for (basis_index, vector) in basis.iter().enumerate() {
            let mut scales = Vec::new();
            for scale_factor in scale_factors {
                let amplitude = h * scale_factor * normalization.max(1.0e-300);
                let (geometry_delta, polarity_delta, adaptation_delta) = match channel {
                    R6ResponseChannel::Geometry => (Some(vector.as_slice()), None, None),
                    R6ResponseChannel::ActivePolarity => (
                        None,
                        Some((vector.as_slice(), R6ResponseChannel::ActivePolarity)),
                        None,
                    ),
                    R6ResponseChannel::InactivePolarity => (
                        None,
                        Some((vector.as_slice(), R6ResponseChannel::InactivePolarity)),
                        None,
                    ),
                    R6ResponseChannel::Adaptation => (None, None, Some(vector.as_slice())),
                };
                let plus = r6_probe_transition(
                    mesh,
                    plasticity,
                    polarity,
                    geometry_delta,
                    polarity_delta,
                    adaptation_delta,
                    amplitude,
                    1.0,
                    topology_tick,
                    motor_enabled,
                    adaptation_enabled,
                    connected,
                );
                let minus = r6_probe_transition(
                    mesh,
                    plasticity,
                    polarity,
                    geometry_delta,
                    polarity_delta,
                    adaptation_delta,
                    amplitude,
                    -1.0,
                    topology_tick,
                    motor_enabled,
                    adaptation_enabled,
                    connected,
                );
                let response = match (plus, minus) {
                    (Some(plus), Some(minus)) if plus.len() == minus.len() => {
                        let norm = plus
                            .iter()
                            .zip(minus)
                            .map(|(left, right)| ((left - right) / (2.0 * amplitude)).powi(2))
                            .sum::<f64>()
                            .sqrt();
                        json!({
                            "status": "VALID",
                            "response_norm": norm,
                            "input_amplitude": amplitude,
                        })
                    }
                    _ => json!({"status": "NONSMOOTH_OR_INVALID", "input_amplitude": amplitude}),
                };
                scales.push(response);
            }
            channels.push(json!({
                "channel": match channel {
                    R6ResponseChannel::Geometry => "geometry",
                    R6ResponseChannel::ActivePolarity => "active_polarity",
                    R6ResponseChannel::InactivePolarity => "inactive_polarity",
                    R6ResponseChannel::Adaptation => "adaptation",
                },
                "basis_index": basis_index,
                "scales": scales,
            }));
        }
    }
    let mut counts = BTreeMap::new();
    for channel in &channels {
        let label = channel["channel"].as_str().unwrap_or("unknown");
        let entry = counts.entry(label.to_string()).or_insert((0usize, 0usize));
        let valid = channel["scales"]
            .as_array()
            .map(|scales| scales.iter().all(|scale| scale["status"] == "VALID"))
            .unwrap_or(false);
        if valid {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
    json!({
        "status": "PASS",
        "operator_scope": "pre_mechanics_to_post_mechanics_local_boundary_response",
        "full_closed_loop_jacobian": "NOT_IDENTIFIED",
        "reason_full_closed_loop": "capture begins after polarity.advance and ends after mechanics; a next-equivalent-boundary polarity chemistry replay is not inferred from post-mechanics data",
        "base_step": h,
        "scale_factors": scale_factors,
        "basis_dimension": {
            "geometry_non_rigid": geometry_dimension,
            "active_polarity_zero_sum": if polarity.is_some() { polarity_dimension } else { 0 },
            "inactive_polarity_zero_sum": if polarity.is_some() { polarity_dimension } else { 0 },
            "adaptation": n,
        },
        "counts": counts,
        "channels": channels,
        "rotation_covariant_basis": true,
        "observer_only": true,
    })
}

/// Serialize the complete diagnostic state presented immediately before the
/// polarity advance.  This is intentionally a diagnostic representation; it
/// is not a second lifecycle state and is never read by production biology.
fn r7_boundary_state_value(
    cohort: &Cohort,
    world: &OpenMedium,
    ledger: &CampaignLedger,
    polarity_states: &PolarityStateMap,
    next_id: u64,
    step: usize,
    campaign_seed: u64,
    connected: bool,
) -> Value {
    json!({
        "cohort": serde_json::to_value(cohort).expect("serialize R7 cohort boundary"),
        "world": serde_json::to_value(world).expect("serialize R7 world boundary"),
        "ledger": serde_json::to_value(ledger).expect("serialize R7 ledger boundary"),
        "polarity_states": serde_json::to_value(polarity_states).expect("serialize R7 polarity boundary"),
        "next_id": next_id,
        "step": step,
        "accepted_steps": ledger.accepted_steps,
        "configuration": {
            "environment": "RESOURCE_CHALLENGE",
            "boundary_mode": "R10R6_FIXED_CONCENTRATION_BOUNDARY",
            "clock_mode": "STEP_MODULO_CADENCE",
            "reserve_resolution": "RESOLVED_FROM_CURRENT_MESH_EACH_STEP",
            "campaign_seed": campaign_seed,
            "polarity_actuator_connected": connected,
        },
    })
}

/// Project the selected physical state at the next pre-polarity boundary into
/// a compact response vector.  Geometry is translation-centred, while the
/// polarity and plasticity entries retain their physical local ordering.  The
/// vector is used only for diagnostic finite differences.
fn r7_response_output(cohort: &Cohort, polarity_states: &PolarityStateMap) -> Vec<f64> {
    let mut output = r6_centered_geometry_output(&cohort.mesh);
    if let Some(state) = polarity_states.get(&cohort.id) {
        output.extend(state.active_amount.iter().copied());
        output.extend(state.inactive_amount.iter().copied());
    } else {
        output.extend(std::iter::repeat_n(0.0, 2 * cohort.mesh.n()));
    }
    if let Some(plasticity) = cohort.plasticity.as_ref() {
        output.extend(plasticity.adaptation.iter().copied());
    } else {
        output.extend(std::iter::repeat_n(0.0, cohort.mesh.n()));
    }
    output
}

fn r7_replay_polarity_and_mechanics(
    cohort: &mut Cohort,
    world: &mut OpenMedium,
    ledger: &mut CampaignLedger,
    polarity_states: &mut PolarityStateMap,
    connected: bool,
    step: usize,
) -> Result<Value, String> {
    r7_replay_polarity_and_mechanics_with_route(
        cohort,
        world,
        ledger,
        polarity_states,
        connected,
        step,
        r10_closure::PolarityMechanicsRoute::R4EdgeTension,
    )
}

fn r7_replay_polarity_and_mechanics_with_route(
    cohort: &mut Cohort,
    world: &mut OpenMedium,
    ledger: &mut CampaignLedger,
    polarity_states: &mut PolarityStateMap,
    connected: bool,
    step: usize,
    polarity_mechanics_route: r10_closure::PolarityMechanicsRoute,
) -> Result<Value, String> {
    let count = cohort.count as f64;
    let measures = (0..cohort.mesh.n())
        .map(|index| cohort.mesh.edge_length(index))
        .collect::<Vec<_>>();
    let polarity_params = PolarityMassParamsV1::candidate();
    let actuation_params =
        PolarityActuationParamsV1::sealed_r3_candidate().map_err(|error| error.to_string())?;
    let state = polarity_states
        .get_mut(&cohort.id)
        .ok_or_else(|| "POLARITY_STATE_MISSING".to_string())?;
    let area = cohort.mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return Err("POLARITY_INVALID_AREA".to_string());
    }
    let available_a = cohort.mesh.interior.a.max(0.0) * area;
    let step_ledger = state
        .advance(&measures, &polarity_params, available_a)
        .map_err(|error| error.to_string())?;
    if step_ledger.a_consumed > available_a + 1.0e-10 {
        return Err("POLARITY_A_OVERDRAW".to_string());
    }
    let a_before = cohort.mesh.interior.a.max(0.0) * area;
    let w_before = cohort.mesh.interior.w.max(0.0) * area;
    cohort.mesh.interior.a = (a_before - step_ledger.a_consumed).max(0.0) / area;
    cohort.mesh.interior.w = (w_before + step_ledger.w_produced) / area;
    let proposal = derive_local_activity(state, &measures, &actuation_params)
        .map_err(|error| error.to_string())?;
    ledger.polarity_a_spent += step_ledger.a_consumed * count;
    ledger.polarity_w_produced += step_ledger.w_produced * count;

    let old_measures = measures;
    let topology_tick = step % 10 == 0;
    let structural_before = cohort.mesh.total_structural_mass();
    let (active_a, active_w, remesh_mappings, mechanics_diagnostic) =
        r10_closure::r10_refractory_mechanics_step_with_polarity_route(
            &mut cohort.mesh,
            cohort
                .plasticity
                .as_mut()
                .ok_or_else(|| "R7_PLASTICITY_MISSING".to_string())?,
            topology_tick,
            true,
            true,
            if connected {
                Some(proposal.vertex_activity.as_slice())
            } else {
                None
            },
            polarity_mechanics_route,
        )
        .ok_or_else(|| "MECHANICS_OR_REMAP_REJECTED".to_string())?;
    if remesh_mappings > 0 {
        let new_measures = (0..cohort.mesh.n())
            .map(|index| cohort.mesh.edge_length(index))
            .collect::<Vec<_>>();
        let state = polarity_states
            .get(&cohort.id)
            .ok_or_else(|| "POLARITY_STATE_MISSING".to_string())?;
        let remapped = state
            .remap_conservative(&old_measures, &new_measures)
            .map_err(|error| error.to_string())?;
        polarity_states.insert(cohort.id, remapped);
    }
    ledger.mechanics_topology_structural_net +=
        (cohort.mesh.total_structural_mass() - structural_before) * count;
    ledger.active_a_spent += active_a * count;
    ledger.active_w_produced += active_w * count;
    ledger.adaptation_remesh_mappings += remesh_mappings as u64 * cohort.count;
    let genotype = cohort
        .mesh
        .finite_allocation
        .ok_or_else(|| "R7_ALLOCATION_MISSING".to_string())?
        .genotype;
    let observer = phenotype_ledger(ledger, genotype);
    observer.active_a_spent += active_a * count;
    observer.active_w_produced += active_w * count;
    if !polygon_simple(&cohort.mesh.vertices)
        || !cohort.mesh.physical_runtime_valid()
        || !cohort.mesh.lifecycle_invariants_hold()
    {
        return Err("INVALID_POST_MECHANICS_STATE".to_string());
    }
    Ok(json!({
        "polarity": {
            "accepted": step_ledger.accepted,
            "a_consumed": step_ledger.a_consumed,
            "w_produced": step_ledger.w_produced,
            "total_before": step_ledger.total_before,
            "total_after": step_ledger.total_after,
            "active_amount_after": polarity_states
                .get(&cohort.id)
                .map(|state| state.active_amount.clone()),
            "inactive_amount_after": polarity_states
                .get(&cohort.id)
                .map(|state| state.inactive_amount.clone()),
            "actuation": proposal,
        },
        "mechanics": {
            "active_a": active_a,
            "active_w": active_w,
            "remesh_mappings": remesh_mappings,
            "diagnostic": mechanics_diagnostic,
            "post_mechanics_cohort": cohort,
        },
        "connected": connected,
        "accepted_step": step,
        "world": serde_json::to_value(world).expect("serialize R7 replay world"),
    }))
}

fn r7_replay_next_prefix(
    cohort: &mut Cohort,
    world: &mut OpenMedium,
    ledger: &mut CampaignLedger,
    step: usize,
) -> Result<(), String> {
    let allocation = AllocationParams::default();
    let transport = TransportParams::default();
    let mechanics = MechParams::default();
    world.refresh_fixed_concentration_boundary(Environment::Resource, step - 1);
    let expression = apply_expression_path(
        &mut cohort.mesh,
        &allocation,
        mechanics.dt,
        D096ExpressionPath::V4FiniteBudgetCentered,
    )
    .map_err(|error| error.label().to_string())?;
    let count = cohort.count as f64;
    let genotype = cohort
        .mesh
        .finite_allocation
        .ok_or_else(|| "R7_ALLOCATION_MISSING".to_string())?
        .genotype;
    ledger.expression_material += expression.structural_consumed * count;
    ledger.expression_catalyst_precursor_a += expression.catalyst_precursor_a * count;
    ledger.expression_activation +=
        (expression.activation_consumed + expression.maintenance_consumed) * count;
    ledger.expression_turnover_waste += expression.turnover_waste * count;
    let observer = phenotype_ledger(ledger, genotype);
    observer.organism_step_exposure += cohort.count;
    observer.expression_material += expression.structural_consumed * count;
    observer.expression_activation +=
        (expression.activation_consumed + expression.maintenance_consumed) * count;

    r10_exchange_with_observer(
        world,
        std::slice::from_mut(cohort),
        &transport,
        mechanics.dt,
        ledger,
    );
    let reaction = r10r9r1_reaction_params(&cohort.mesh);
    let reactions = reactions_step(&mut cohort.mesh, &reaction, mechanics.dt, true, true);
    ledger.reaction_n_consumed += reactions.n_consumed * count;
    ledger.reaction_f_consumed += reactions.f_consumed * count;
    ledger.a_produced += reactions.a_produced * count;
    ledger.w_produced += reactions.w_produced * count;
    ledger.reaction_activation_equivalent_closure_residual +=
        reactions.activation_equivalent_closure_residual * count;
    ledger.m1_structural_build += reactions.m_produced * count;
    ledger.m1_structural_turnover += reactions.m_to_w * count;
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let grown = growth_step_with_placement(
        &mut cohort.mesh,
        &reaction,
        &growth,
        mechanics.dt,
        GrowthPlacementMode::FrozenD088,
    );
    ledger.growth_material += grown.m_grown * count;
    ledger.growth_a_consumed += grown.a_consumed_growth * count;
    ledger.growth_w_produced += grown.w_from_growth * count;
    ledger.reserve_a_to_r += reactions.reserve.a_to_r * count;
    ledger.reserve_r_to_a += reactions.reserve.r_to_a * count;
    ledger.reserve_r_to_w += reactions.reserve.r_to_w * count;
    ledger.reserve_r_to_m += reactions.reserve.r_to_m * count;
    ledger.reserve_funded_growth += grown.r_consumed_growth * count;
    if reactions.reserve.a_to_r.abs()
        + reactions.reserve.r_to_a.abs()
        + reactions.reserve.r_to_w.abs()
        + grown.r_consumed_growth.abs()
        > 1.0e-12
    {
        ledger.reserve_active_steps += cohort.count;
    }
    let observer = phenotype_ledger(ledger, genotype);
    observer.reaction_n_consumed += reactions.n_consumed * count;
    observer.reaction_f_consumed += reactions.f_consumed * count;
    observer.a_produced += reactions.a_produced * count;
    observer.w_produced += reactions.w_produced * count;
    observer.growth_material += grown.m_grown * count;
    if !polygon_simple(&cohort.mesh.vertices)
        || !cohort.mesh.physical_runtime_valid()
        || !cohort.mesh.lifecycle_invariants_hold()
    {
        return Err("INVALID_NEXT_PRE_POLARITY_STATE".to_string());
    }
    Ok(())
}

fn r7_replay_full_cycle(seed: &R7BoundarySeed) -> Value {
    let mut cohort = seed.cohort.clone();
    let mut world = seed.world.clone();
    let mut ledger = seed.ledger.clone();
    let mut polarity_states = seed.polarity_states.clone();
    let current_step = seed.step;
    let mechanics = match r7_replay_polarity_and_mechanics(
        &mut cohort,
        &mut world,
        &mut ledger,
        &mut polarity_states,
        seed.connected,
        current_step,
    ) {
        Ok(value) => value,
        Err(reason) => return json!({"status": "REJECTED", "reason": reason}),
    };
    ledger.accepted_steps += 1;
    if let Err(reason) =
        r7_replay_next_prefix(&mut cohort, &mut world, &mut ledger, current_step + 1)
    {
        return json!({"status": "REJECTED", "reason": reason});
    }
    let next_boundary = r7_boundary_state_value(
        &cohort,
        &world,
        &ledger,
        &polarity_states,
        seed.next_id,
        current_step + 1,
        seed.campaign_seed,
        seed.connected,
    );
    json!({
        "status": "ACCEPTED",
        "next_boundary": next_boundary.clone(),
        "response_output": r7_response_output(&cohort, &polarity_states),
        "accepted_steps": ledger.accepted_steps,
        "transition": mechanics,
        "same_phase": "pre_polarity_advance_to_next_pre_polarity_advance",
    })
}

/// Execute only the same accepted pre-polarity -> post-mechanics transition
/// used by R7, with an explicit diagnostic actuator route.  The route runs on
/// a clone and is never part of the production lifecycle.
fn r10_replay_mechanics_route(
    seed: &R7BoundarySeed,
    route: r10_closure::PolarityMechanicsRoute,
    connected: bool,
) -> Value {
    let mut cohort = seed.cohort.clone();
    let mut world = seed.world.clone();
    let mut ledger = seed.ledger.clone();
    let mut polarity_states = seed.polarity_states.clone();
    match r7_replay_polarity_and_mechanics_with_route(
        &mut cohort,
        &mut world,
        &mut ledger,
        &mut polarity_states,
        connected,
        seed.step,
        route,
    ) {
        Ok(transition) => json!({
            "status": "ACCEPTED",
            "transition": transition,
            "route": match route {
                r10_closure::PolarityMechanicsRoute::R4EdgeTension => "R4_EDGE_TENSION",
                r10_closure::PolarityMechanicsRoute::R10InwardNormal => "R10_INWARD_NORMAL",
            },
            "connected": connected,
            "observer_only": true,
        }),
        Err(reason) => json!({
            "status": "REJECTED",
            "reason": reason,
            "route": match route {
                r10_closure::PolarityMechanicsRoute::R4EdgeTension => "R4_EDGE_TENSION",
                r10_closure::PolarityMechanicsRoute::R10InwardNormal => "R10_INWARD_NORMAL",
            },
            "connected": connected,
            "observer_only": true,
        }),
    }
}

fn r10_normal_remap_record(seed: &R7BoundarySeed) -> Result<Value, String> {
    let r4 = r7_replay_full_cycle(seed);
    let r8_reference = r8_p_to_m_record(seed)?;
    let normal = r10_replay_mechanics_route(
        seed,
        r10_closure::PolarityMechanicsRoute::R10InwardNormal,
        true,
    );
    let cut = r10_replay_mechanics_route(
        seed,
        r10_closure::PolarityMechanicsRoute::R10InwardNormal,
        false,
    );
    if r4["status"] != "ACCEPTED"
        || normal["status"] != "ACCEPTED"
        || cut["status"] != "ACCEPTED"
    {
        return Ok(json!({
            "status": "P_TO_M_NONSMOOTH",
            "reason": "one or more matched route transitions were not accepted",
            "r4_edge_tension": r4,
            "normal_remap": normal,
            "polarity_cut": cut,
        }));
    }
    let normal_cohort = r8_mesh_from_transition(&normal)?;
    let cut_cohort = r8_mesh_from_transition(&cut)?;
    let pre_vertices = &seed.cohort.mesh.vertices;
    let normal_vertices = &normal_cohort.mesh.vertices;
    let cut_vertices = &cut_cohort.mesh.vertices;
    if normal_vertices.len() != cut_vertices.len() || normal_vertices.len() != pre_vertices.len() {
        return Ok(json!({
            "status": "P_TO_M_NONSMOOTH",
            "reason": "matched normal-remap clones changed topology",
            "r4_edge_tension": r4,
            "normal_remap": normal,
            "polarity_cut": cut,
        }));
    }
    let delta = normal_vertices
        .iter()
        .zip(cut_vertices)
        .map(|(normal, cut)| [normal[0] - cut[0], normal[1] - cut[1]])
        .collect::<Vec<_>>();
    let normal_displacement = normal_vertices
        .iter()
        .zip(pre_vertices)
        .map(|(after, before)| [after[0] - before[0], after[1] - before[1]])
        .collect::<Vec<_>>();
    let modes = r8_normal_harmonic_basis(&seed.cohort.mesh);
    let activity = normal["transition"]["polarity"]["actuation"]["vertex_activity"]
        .as_array()
        .ok_or_else(|| "R10_POLARITY_ACTIVITY_MISSING".to_string())?
        .iter()
        .map(|value| value.as_f64().unwrap_or(f64::NAN))
        .collect::<Vec<_>>();
    let normal_polarity = &normal["transition"]["polarity"];
    let cut_polarity = &cut["transition"]["polarity"];
    let normal_mechanics = &normal["transition"]["mechanics"];
    let cut_mechanics = &cut["transition"]["mechanics"];
    let branch_equal = normal_mechanics["remesh_mappings"] == cut_mechanics["remesh_mappings"]
        && normal_mechanics["diagnostic"]["topology_ruptures"]
            == cut_mechanics["diagnostic"]["topology_ruptures"]
        && normal_mechanics["diagnostic"]["topology_rebonds"]
            == cut_mechanics["diagnostic"]["topology_rebonds"];
    let mechanical_a_delta = normal_mechanics["active_a"].as_f64().unwrap_or(0.0)
        - cut_mechanics["active_a"].as_f64().unwrap_or(0.0);
    let mechanical_w_delta = normal_mechanics["active_w"].as_f64().unwrap_or(0.0)
        - cut_mechanics["active_w"].as_f64().unwrap_or(0.0);
    let polarity_a_delta = normal_polarity["a_consumed"].as_f64().unwrap_or(0.0)
        - cut_polarity["a_consumed"].as_f64().unwrap_or(0.0);
    Ok(json!({
        "status": if branch_equal { "VALID" } else { "P_TO_M_NONSMOOTH" },
        "input_digest": deterministic_state_digest(&r7_boundary_state_value(
            &seed.cohort,
            &seed.world,
            &seed.ledger,
            &seed.polarity_states,
            seed.next_id,
            seed.step,
            seed.campaign_seed,
            true,
        )),
        "clone_equality_before_intervention": true,
        "r4_edge_tension": r4["transition"].clone(),
        "r8_route_off_reference": r8_reference,
        "normal_remap": normal["transition"].clone(),
        "polarity_cut": cut["transition"].clone(),
        "post_polarity_active_equal": normal_polarity["active_amount_after"]
            == cut_polarity["active_amount_after"],
        "post_polarity_inactive_equal": normal_polarity["inactive_amount_after"]
            == cut_polarity["inactive_amount_after"],
        "branch_equal": branch_equal,
        "polarity_activity": activity.clone(),
        "polarity_activity_harmonics": r8_scalar_harmonics(&activity),
        "mechanical_delta_vertices": delta.clone(),
        "mechanical_delta_norm": r8_point_vector_norm(&delta),
        "normal_displacement_norm": r8_point_vector_norm(&normal_displacement),
        "normal_modal_delta": r8_normal_modal_projections(&delta, &modes),
        "normal_full_modal": r8_normal_modal_projections(&normal_displacement, &modes),
        "mechanical_a_delta": mechanical_a_delta,
        "mechanical_w_delta": mechanical_w_delta,
        "polarity_chemistry_a_delta": polarity_a_delta,
        "normal_route": normal_mechanics["diagnostic"].clone(),
        "cut_route": cut_mechanics["diagnostic"].clone(),
        "candidate_contract": {
            "equation": "d_i_R10 = min(1, d_i_legacy + p_i)",
            "polarity_edge_tension": "DISABLED",
            "actuator": "inward_normal_request(mesh, drive, dt)",
            "new_force_law": false,
            "new_energy_price": false,
        },
        "observer_only": true,
        "production_transition_modified": false,
    }))
}

fn r7_probe_transition(
    seed: &R7BoundarySeed,
    channel: R6ResponseChannel,
    vector: &[f64],
    amplitude: f64,
    sign: f64,
) -> Option<Vec<f64>> {
    let mut perturbed = seed.clone();
    match channel {
        R6ResponseChannel::Geometry => {
            for (index, point) in perturbed.cohort.mesh.vertices.iter_mut().enumerate() {
                point[0] += sign * amplitude * vector[2 * index];
                point[1] += sign * amplitude * vector[2 * index + 1];
            }
        }
        R6ResponseChannel::ActivePolarity | R6ResponseChannel::InactivePolarity => {
            let state = perturbed.polarity_states.get_mut(&perturbed.cohort.id)?;
            let target = if matches!(channel, R6ResponseChannel::ActivePolarity) {
                &mut state.active_amount
            } else {
                &mut state.inactive_amount
            };
            for (value, increment) in target.iter_mut().zip(vector) {
                *value += sign * amplitude * increment;
            }
            if target
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0)
            {
                return None;
            }
        }
        R6ResponseChannel::Adaptation => {
            let adaptation = perturbed
                .cohort
                .plasticity
                .as_mut()?
                .adaptation
                .as_mut_slice();
            for (value, increment) in adaptation.iter_mut().zip(vector) {
                *value += sign * amplitude * increment;
            }
            if adaptation
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            {
                return None;
            }
        }
    }
    if !polygon_simple(&perturbed.cohort.mesh.vertices)
        || !perturbed.cohort.mesh.physical_runtime_valid()
    {
        return None;
    }
    let replay = r7_replay_full_cycle(&perturbed);
    if replay["status"] != "ACCEPTED" {
        return None;
    }
    replay["response_output"].as_array().map(|values| {
        values
            .iter()
            .map(|value| value.as_f64().unwrap_or(f64::NAN))
            .collect()
    })
}

fn r7_full_cycle_response_probe(seed: &R7BoundarySeed) -> Value {
    let n = seed.cohort.mesh.n();
    let geometry_basis = r6_geometry_basis(&seed.cohort.mesh);
    let polarity_basis = r6_zero_sum_basis(n);
    let adaptation_basis = (0..n)
        .map(|index| {
            let mut vector = vec![0.0; n];
            vector[index] = 1.0;
            vector
        })
        .collect::<Vec<_>>();
    let mean_edge = (0..n)
        .map(|index| seed.cohort.mesh.edge_length(index))
        .sum::<f64>()
        / n.max(1) as f64;
    let state = seed.polarity_states.get(&seed.cohort.id);
    let active_rms = state
        .map(|value| {
            (value
                .active_amount
                .iter()
                .map(|entry| entry * entry)
                .sum::<f64>()
                / n.max(1) as f64)
                .sqrt()
        })
        .unwrap_or(0.0);
    let inactive_rms = state
        .map(|value| {
            (value
                .inactive_amount
                .iter()
                .map(|entry| entry * entry)
                .sum::<f64>()
                / n.max(1) as f64)
                .sqrt()
        })
        .unwrap_or(0.0);
    let h = f64::EPSILON.cbrt();
    let scale_factors = [4.0, 2.0, 1.0];
    let mut channels = Vec::new();
    let mut valid_columns = 0usize;
    let mut non_smooth_columns = 0usize;
    for (channel, basis, normalization) in [
        (R6ResponseChannel::Geometry, geometry_basis, mean_edge),
        (
            R6ResponseChannel::ActivePolarity,
            polarity_basis.clone(),
            active_rms,
        ),
        (
            R6ResponseChannel::InactivePolarity,
            polarity_basis,
            inactive_rms,
        ),
        (R6ResponseChannel::Adaptation, adaptation_basis, 1.0),
    ] {
        for (basis_index, vector) in basis.iter().enumerate() {
            let mut samples: Vec<(f64, Option<Vec<f64>>)> = Vec::new();
            let mut scale_records = Vec::new();
            for scale_factor in scale_factors {
                let amplitude = h * scale_factor * normalization.max(1.0e-300);
                let plus = r7_probe_transition(seed, channel, vector, amplitude, 1.0);
                let minus = r7_probe_transition(seed, channel, vector, amplitude, -1.0);
                let derivative = match (plus, minus) {
                    (Some(plus), Some(minus)) if plus.len() == minus.len() => {
                        let response = plus
                            .iter()
                            .zip(minus.iter())
                            .map(|(left, right)| (left - right) / (2.0 * amplitude))
                            .collect::<Vec<_>>();
                        let norm = response
                            .iter()
                            .map(|value| value * value)
                            .sum::<f64>()
                            .sqrt();
                        scale_records.push(json!({
                            "status": "VALID",
                            "input_amplitude": amplitude,
                            "response_norm": norm,
                        }));
                        Some(response)
                    }
                    _ => {
                        scale_records.push(json!({
                            "status": "NONSMOOTH_OR_INVALID",
                            "input_amplitude": amplitude,
                        }));
                        None
                    }
                };
                samples.push((scale_factor, derivative));
            }
            let finest = samples[2].1.as_ref();
            let middle = samples[1].1.as_ref();
            let coarse = samples[0].1.as_ref();
            let norm = |value: &[f64]| value.iter().map(|entry| entry * entry).sum::<f64>().sqrt();
            let difference = |left: &[f64], right: &[f64]| {
                left.iter()
                    .zip(right)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f64>()
                    .sqrt()
            };
            let convergence = match (finest, middle, coarse) {
                (Some(finest), Some(middle), Some(coarse)) => {
                    let fine_norm = norm(finest);
                    let fine_middle = difference(finest, middle);
                    let middle_coarse = difference(middle, coarse);
                    let fine_tolerance = (0.05 * fine_norm).max(1.0e-8);
                    let converged = fine_middle <= fine_tolerance && fine_middle <= middle_coarse;
                    if converged {
                        valid_columns += 1;
                    }
                    json!({
                        "status": if converged { "CONVERGED" } else { "NOT_CONVERGED" },
                        "fine_middle_disagreement": fine_middle,
                        "middle_coarse_disagreement": middle_coarse,
                        "relative_tolerance": fine_tolerance,
                    })
                }
                _ => {
                    non_smooth_columns += 1;
                    json!({"status": "NONSMOOTH_OR_INCOMPLETE"})
                }
            };
            if finest.is_none() || middle.is_none() || coarse.is_none() {
                non_smooth_columns += 1;
            }
            channels.push(json!({
                "channel": match channel {
                    R6ResponseChannel::Geometry => "geometry",
                    R6ResponseChannel::ActivePolarity => "active_polarity",
                    R6ResponseChannel::InactivePolarity => "inactive_polarity",
                    R6ResponseChannel::Adaptation => "adaptation",
                },
                "basis_index": basis_index,
                "scales": scale_records,
                "convergence": convergence,
            }));
        }
    }
    let all_columns = channels.len();
    json!({
        "status": if valid_columns == all_columns { "PASS" } else { "BOUNDED_INCONCLUSIVE" },
        "operator_scope": "pre_polarity_advance_to_next_pre_polarity_advance_full_cycle",
        "full_cycle_jacobian": "NOT_IDENTIFIED",
        "reason_full_cycle": "the captured same-phase map is replayable, but the sealed R6 perturbation channels do not form a closed square tangent on every causal chemistry/world component and any incomplete or branch-crossing column is fail-closed",
        "base_step": h,
        "scale_factors": scale_factors,
        "basis_dimension": {
            "geometry_non_rigid": channels.iter().filter(|row| row["channel"] == "geometry").count(),
            "active_polarity_zero_sum": if state.is_some() { n.saturating_sub(1) } else { 0 },
            "inactive_polarity_zero_sum": if state.is_some() { n.saturating_sub(1) } else { 0 },
            "adaptation": n,
        },
        "columns": all_columns,
        "converged_columns": valid_columns,
        "non_smooth_or_incomplete_columns": non_smooth_columns,
        "channels": channels,
        "rotation_covariant_basis": true,
        "observer_only": true,
        "r6_contract_reused": true,
        "response_blocks": {
            "polarity_to_polarity": "RECORDED_CHANNEL_RESPONSE_FULL_OPERATOR_NOT_IDENTIFIED",
            "geometry_to_geometry": "RECORDED_CHANNEL_RESPONSE_FULL_OPERATOR_NOT_IDENTIFIED",
            "polarity_to_geometry": "RECORDED_CHANNEL_RESPONSE_FULL_OPERATOR_NOT_IDENTIFIED",
            "geometry_to_polarity": "RECORDED_CHANNEL_RESPONSE_FULL_OPERATOR_NOT_IDENTIFIED",
            "adaptation_cross_blocks": "RECORDED_CHANNEL_RESPONSE_FULL_OPERATOR_NOT_IDENTIFIED",
        },
        "leading_eigenvalues": "NOT_IDENTIFIED",
        "connected_disconnected_operator_difference":
            "SAME_STATE_REPLAY_RECORDED; FULL_OPERATOR_NOT_IDENTIFIED",
    })
}

/// R8 uses the exact R7 replay seed, but it does not reuse R7's incomplete
/// global response probe.  These helpers cut only one named module edge on a
/// cloned state and return the local response in the physical coordinates
/// consumed by the adjacent module.
fn r8_seed_from_snapshot(snapshot: &Value) -> Result<R7BoundarySeed, String> {
    let boundary = snapshot
        .get("input_boundary")
        .ok_or_else(|| "R8_INPUT_BOUNDARY_MISSING".to_string())?;
    Ok(R7BoundarySeed {
        cohort: serde_json::from_value(
            boundary
                .get("cohort")
                .cloned()
                .ok_or_else(|| "R8_COHORT_BOUNDARY_MISSING".to_string())?,
        )
        .map_err(|error| format!("R8_COHORT_DESERIALIZE:{error}"))?,
        world: serde_json::from_value(
            boundary
                .get("world")
                .cloned()
                .ok_or_else(|| "R8_WORLD_BOUNDARY_MISSING".to_string())?,
        )
        .map_err(|error| format!("R8_WORLD_DESERIALIZE:{error}"))?,
        ledger: serde_json::from_value(
            boundary
                .get("ledger")
                .cloned()
                .ok_or_else(|| "R8_LEDGER_BOUNDARY_MISSING".to_string())?,
        )
        .map_err(|error| format!("R8_LEDGER_DESERIALIZE:{error}"))?,
        polarity_states: serde_json::from_value(
            boundary
                .get("polarity_states")
                .cloned()
                .ok_or_else(|| "R8_POLARITY_BOUNDARY_MISSING".to_string())?,
        )
        .map_err(|error| format!("R8_POLARITY_DESERIALIZE:{error}"))?,
        next_id: boundary
            .get("next_id")
            .and_then(Value::as_u64)
            .ok_or_else(|| "R8_NEXT_ID_MISSING".to_string())?,
        step: boundary
            .get("step")
            .and_then(Value::as_u64)
            .ok_or_else(|| "R8_STEP_MISSING".to_string())? as usize,
        campaign_seed: boundary
            .get("configuration")
            .and_then(|configuration| configuration.get("campaign_seed"))
            .and_then(Value::as_u64)
            .ok_or_else(|| "R8_CAMPAIGN_SEED_MISSING".to_string())?,
        connected: boundary
            .get("configuration")
            .and_then(|configuration| configuration.get("polarity_actuator_connected"))
            .and_then(Value::as_bool)
            .ok_or_else(|| "R8_CONNECTION_FLAG_MISSING".to_string())?,
    })
}

fn r8_constraint_basis(mesh: &MaterialMesh) -> Vec<Vec<f64>> {
    let n = mesh.n();
    let centroid = mesh.centroid();
    let mut translation_x = vec![0.0; 2 * n];
    let mut translation_y = vec![0.0; 2 * n];
    let mut rotation = vec![0.0; 2 * n];
    for (index, point) in mesh.vertices.iter().enumerate() {
        translation_x[2 * index] = 1.0;
        translation_y[2 * index + 1] = 1.0;
        rotation[2 * index] = -(point[1] - centroid[1]);
        rotation[2 * index + 1] = point[0] - centroid[0];
    }
    r6_orthogonalized_basis(vec![translation_x, translation_y, rotation], &[])
}

/// Build radial normal Fourier modes.  The radial frame is derived from the
/// current mesh and the sine/cosine partners transform together under cyclic
/// reindexing; no absolute axis or preferred edge is introduced.
fn r8_normal_harmonic_basis(mesh: &MaterialMesh) -> Vec<(usize, String, Vec<f64>)> {
    let n = mesh.n();
    if n < 3 {
        return Vec::new();
    }
    let centroid = mesh.centroid();
    let mut normals = Vec::with_capacity(n);
    for point in &mesh.vertices {
        let dx = point[0] - centroid[0];
        let dy = point[1] - centroid[1];
        let norm = dx.hypot(dy);
        if !norm.is_finite() || norm <= 1.0e-12 {
            return Vec::new();
        }
        normals.push([dx / norm, dy / norm]);
    }
    let constraints = r8_constraint_basis(mesh);
    let mut modes = Vec::new();
    for harmonic in 0..=(n / 2) {
        let phases = if harmonic == 0 || harmonic * 2 == n {
            vec![("cos", 0.0_f64)]
        } else {
            vec![("cos", 0.0_f64), ("sin", std::f64::consts::FRAC_PI_2)]
        };
        for (phase, phase_shift) in phases {
            let mut vector = vec![0.0; 2 * n];
            for (index, normal) in normals.iter().enumerate() {
                let angle = std::f64::consts::TAU * harmonic as f64 * index as f64 / n as f64;
                let value = (angle + phase_shift).cos();
                vector[2 * index] = normal[0] * value;
                vector[2 * index + 1] = normal[1] * value;
            }
            for constraint in &constraints {
                let projection = vector
                    .iter()
                    .zip(constraint)
                    .map(|(left, right)| left * right)
                    .sum::<f64>();
                for (value, constraint_value) in vector.iter_mut().zip(constraint) {
                    *value -= projection * constraint_value;
                }
            }
            let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
            if norm > 1.0e-10 {
                vector.iter_mut().for_each(|value| *value /= norm);
                modes.push((harmonic, phase.to_string(), vector));
            }
        }
    }
    modes
}

fn r8_scalar_harmonics(values: &[f64]) -> Vec<Value> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    (0..=(n / 2))
        .map(|harmonic| {
            let mut cosine = 0.0;
            let mut sine = 0.0;
            for (index, value) in values.iter().enumerate() {
                let angle = std::f64::consts::TAU * harmonic as f64 * index as f64 / n as f64;
                cosine += value * angle.cos();
                sine += value * angle.sin();
            }
            let factor = if harmonic == 0 || harmonic * 2 == n {
                1.0 / n as f64
            } else {
                2.0 / n as f64
            };
            let cosine = cosine * factor;
            let sine = sine * factor;
            json!({
                "harmonic": harmonic,
                "cosine": cosine,
                "sine": sine,
                "amplitude": cosine.hypot(sine),
            })
        })
        .collect()
}

fn r8_normal_modal_projections(
    displacement: &[[f64; 2]],
    modes: &[(usize, String, Vec<f64>)],
) -> Vec<Value> {
    modes
        .iter()
        .map(|(harmonic, phase, vector)| {
            let projection = displacement
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    point[0] * vector[2 * index] + point[1] * vector[2 * index + 1]
                })
                .sum::<f64>();
            json!({
                "harmonic": harmonic,
                "phase": phase,
                "projection": projection,
                "absolute_projection": projection.abs(),
            })
        })
        .collect()
}

fn r8_vertices(value: &Value, path: &str) -> Result<Vec<[f64; 2]>, String> {
    serde_json::from_value(value[path].clone())
        .map_err(|error| format!("R8_{path}_VERTICES_DESERIALIZE:{error}"))
}

fn r8_mesh_from_transition(value: &Value) -> Result<Cohort, String> {
    serde_json::from_value(
        value["transition"]["mechanics"]["post_mechanics_cohort"].clone(),
    )
    .map_err(|error| format!("R8_POST_MECHANICS_COHORT_DESERIALIZE:{error}"))
}

fn r8_transition_summary(value: &Value) -> Value {
    let diagnostic = value["transition"]["mechanics"]["diagnostic"].clone();
    json!({
        "status": value["status"].clone(),
        "accepted_steps": value["accepted_steps"].clone(),
        "active_a": diagnostic["funded_active_a"].clone(),
        "active_w": diagnostic["active_w_produced"].clone(),
        "requested_active_a": diagnostic["requested_active_a"].clone(),
        "funding_ratio": diagnostic["funding_ratio"].clone(),
        "remesh_mappings": diagnostic["remesh_mappings"].clone(),
        "topology_ruptures": diagnostic["topology_ruptures"].clone(),
        "topology_rebonds": diagnostic["topology_rebonds"].clone(),
        "simple": diagnostic["simple"].clone(),
        "runtime_valid": diagnostic["runtime_valid"].clone(),
        "lifecycle_valid": diagnostic["lifecycle_valid"].clone(),
        "mechanics_diagnostic": diagnostic,
    })
}

fn r8_p_to_m_record(seed: &R7BoundarySeed) -> Result<Value, String> {
    let mut cut_seed = seed.clone();
    cut_seed.connected = false;
    let full = r7_replay_full_cycle(seed);
    let cut = r7_replay_full_cycle(&cut_seed);
    if full["status"] != "ACCEPTED" || cut["status"] != "ACCEPTED" {
        return Ok(json!({
            "status": "P_TO_M_NONSMOOTH",
            "reason": "one or both matched clone transitions were not accepted",
            "full": r8_transition_summary(&full),
            "cut": r8_transition_summary(&cut),
        }));
    }
    let full_cohort = r8_mesh_from_transition(&full)?;
    let cut_cohort = r8_mesh_from_transition(&cut)?;
    let pre_vertices = &seed.cohort.mesh.vertices;
    let full_vertices = &full_cohort.mesh.vertices;
    let cut_vertices = &cut_cohort.mesh.vertices;
    if full_vertices.len() != cut_vertices.len() || full_vertices.len() != pre_vertices.len() {
        return Ok(json!({
            "status": "P_TO_M_NONSMOOTH",
            "reason": "matched actuator clones changed topology",
            "full": r8_transition_summary(&full),
            "cut": r8_transition_summary(&cut),
        }));
    }
    let full_cut_delta = full_vertices
        .iter()
        .zip(cut_vertices)
        .map(|(full, cut)| [full[0] - cut[0], full[1] - cut[1]])
        .collect::<Vec<_>>();
    let full_displacement = full_vertices
        .iter()
        .zip(pre_vertices)
        .map(|(after, before)| [after[0] - before[0], after[1] - before[1]])
        .collect::<Vec<_>>();
    let modes = r8_normal_harmonic_basis(&seed.cohort.mesh);
    let activity = full["transition"]["polarity"]["actuation"]["vertex_activity"]
        .as_array()
        .ok_or_else(|| "R8_ACTIVITY_MISSING".to_string())?
        .iter()
        .map(|value| value.as_f64().unwrap_or(f64::NAN))
        .collect::<Vec<_>>();
    let full_polarity = full["transition"]["polarity"]["active_amount_after"].clone();
    let cut_polarity = cut["transition"]["polarity"]["active_amount_after"].clone();
    let branch_equal = full["transition"]["mechanics"]["remesh_mappings"]
        == cut["transition"]["mechanics"]["remesh_mappings"]
        && full["transition"]["mechanics"]["diagnostic"]["topology_ruptures"]
            == cut["transition"]["mechanics"]["diagnostic"]["topology_ruptures"]
        && full["transition"]["mechanics"]["diagnostic"]["topology_rebonds"]
            == cut["transition"]["mechanics"]["diagnostic"]["topology_rebonds"];
    let full_polarity_state = &full["transition"]["polarity"];
    let cut_polarity_state = &cut["transition"]["polarity"];
    let mechanical_delta_norm = r8_point_vector_norm(&full_cut_delta);
    let polarity_a_delta = full_polarity_state["a_consumed"].as_f64().unwrap_or(0.0)
        - cut_polarity_state["a_consumed"].as_f64().unwrap_or(0.0);
    let active_work_delta = full["transition"]["mechanics"]["active_a"]
        .as_f64()
        .unwrap_or(0.0)
        - cut["transition"]["mechanics"]["active_a"]
            .as_f64()
            .unwrap_or(0.0);
    let normal_modal_delta = r8_normal_modal_projections(&full_cut_delta, &modes);
    let full_normal_modal = r8_normal_modal_projections(&full_displacement, &modes);
    Ok(json!({
        "status": if branch_equal { "VALID" } else { "P_TO_M_NONSMOOTH" },
        "input_digest": deterministic_state_digest(&r7_boundary_state_value(
            &seed.cohort,
            &seed.world,
            &seed.ledger,
            &seed.polarity_states,
            seed.next_id,
            seed.step,
            seed.campaign_seed,
            seed.connected,
        )),
        "clone_equality_before_intervention": true,
        "full": r8_transition_summary(&full),
        "cut": r8_transition_summary(&cut),
        "post_polarity_active_equal": full_polarity == cut_polarity,
        "post_polarity_inactive_equal": full_polarity_state["inactive_amount_after"]
            == cut_polarity_state["inactive_amount_after"],
        "branch_equal": branch_equal,
        "polarity_activity": activity,
        "polarity_activity_harmonics": r8_scalar_harmonics(&activity),
        "mechanical_delta_vertices": full_cut_delta,
        "mechanical_delta_norm": mechanical_delta_norm,
        "full_displacement_norm": r8_point_vector_norm(&full_displacement),
        "normal_modal_delta": normal_modal_delta,
        "full_normal_modal": full_normal_modal,
        "active_work_delta": active_work_delta,
        "polarity_chemistry_a_delta": polarity_a_delta,
        "polarity_a_consumed_full": full_polarity_state["a_consumed"].clone(),
        "polarity_a_consumed_cut": cut_polarity_state["a_consumed"].clone(),
        "mechanical_cut_scope": "only R4 polarity-derived actuator input; legacy/passive mechanics and actuator cost remain enabled",
        "observer_only": true,
    }))
}

fn r8_polarity_update(
    seed: &R7BoundarySeed,
    cohort: &Cohort,
    measures: &[f64],
) -> Result<Value, String> {
    let mut states = seed.polarity_states.clone();
    let state = states
        .get_mut(&cohort.id)
        .ok_or_else(|| "R8_POLARITY_STATE_MISSING".to_string())?;
    let area = cohort.mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return Err("R8_POLARITY_INVALID_AREA".to_string());
    }
    let a_before = cohort.mesh.interior.a.max(0.0) * area;
    let w_before = cohort.mesh.interior.w.max(0.0) * area;
    let step = state
        .advance(measures, &PolarityMassParamsV1::candidate(), a_before)
        .map_err(|error| format!("R8_POLARITY_UPDATE:{error}"))?;
    let a_after = a_before - step.a_consumed;
    let w_after = w_before + step.w_produced;
    if a_after < -1.0e-10 || !a_after.is_finite() || !w_after.is_finite() {
        return Err("R8_POLARITY_ENERGY_INVALID".to_string());
    }
    Ok(json!({
        "active_after": state.active_amount.clone(),
        "inactive_after": state.inactive_amount.clone(),
        "total_before": step.total_before,
        "total_after": step.total_after,
        "total_residual": step.total_after - step.total_before,
        "a_consumed": step.a_consumed,
        "w_produced": step.w_produced,
        "a_before": a_before,
        "a_after": a_after.max(0.0),
        "w_before": w_before,
        "w_after": w_after,
        "accepted": step.accepted,
        "measure_count": measures.len(),
    }))
}

/// R9 diagnostic polarity update.  The caller supplies the physical mesh
/// whose local strain is authoritative for this update.  `measures` can be a
/// diagnostic cut vector, while `strain_source` remains the explicitly named
/// geometry input; this keeps the G_TO_P intervention limited to the edge
/// measure vector and makes the extra R9 signal auditable.
fn r9_polarity_update(
    seed: &R7BoundarySeed,
    cohort: &Cohort,
    measures: &[f64],
    strain_source: &Cohort,
) -> Result<Value, String> {
    let mut states = seed.polarity_states.clone();
    let state = states
        .get_mut(&cohort.id)
        .ok_or_else(|| "R9_POLARITY_STATE_MISSING".to_string())?;
    let area = cohort.mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return Err("R9_POLARITY_INVALID_AREA".to_string());
    }
    if strain_source.mesh.n() != measures.len() {
        return Err("R9_STRAIN_MEASURE_LENGTH_MISMATCH".to_string());
    }
    let load_bearing_strain = (0..strain_source.mesh.n())
        .map(|index| strain_source.mesh.load_bearing_strain(index))
        .collect::<Vec<_>>();
    let tensile_signal = load_bearing_strain
        .iter()
        .map(|value| value.max(0.0))
        .collect::<Vec<_>>();
    let a_before = cohort.mesh.interior.a.max(0.0) * area;
    let w_before = cohort.mesh.interior.w.max(0.0) * area;
    let step = state
        .advance_with_load_bearing_strain(
            measures,
            &PolarityMassParamsV1::candidate(),
            a_before,
            Some(&load_bearing_strain),
        )
        .map_err(|error| format!("R9_POLARITY_UPDATE:{error}"))?;
    let a_after = a_before - step.a_consumed;
    let w_after = w_before + step.w_produced;
    if a_after < -1.0e-10 || !a_after.is_finite() || !w_after.is_finite() {
        return Err("R9_POLARITY_ENERGY_INVALID".to_string());
    }
    Ok(json!({
        "active_after": state.active_amount.clone(),
        "inactive_after": state.inactive_amount.clone(),
        "total_before": step.total_before,
        "total_after": step.total_after,
        "total_residual": step.total_after - step.total_before,
        "a_consumed": step.a_consumed,
        "w_produced": step.w_produced,
        "a_before": a_before,
        "a_after": a_after.max(0.0),
        "w_before": w_before,
        "w_after": w_after,
        "accepted": step.accepted,
        "measure_count": measures.len(),
        "load_bearing_strain": load_bearing_strain,
        "tensile_signal": tensile_signal,
        "mechanosensitive_equation": "(1 + max(0, load_bearing_strain_i)) * existing_R3_activation_term - existing_R3_deactivation_term",
        "mechanical_output": false,
        "observer_only": true,
    }))
}

fn r8_perturbed_cohort(
    seed: &R7BoundarySeed,
    mode: &[f64],
    amplitude: f64,
    sign: f64,
) -> Result<Cohort, String> {
    let mut cohort = seed.cohort.clone();
    for (index, point) in cohort.mesh.vertices.iter_mut().enumerate() {
        point[0] += sign * amplitude * mode[2 * index];
        point[1] += sign * amplitude * mode[2 * index + 1];
    }
    if !polygon_simple(&cohort.mesh.vertices) || !cohort.mesh.physical_runtime_valid() {
        return Err("R8_GEOMETRY_PERTURBATION_INVALID".to_string());
    }
    Ok(cohort)
}

/// R9 repeats the sealed R8 G_TO_P perturbation contract with the only
/// difference being the opt-in local tensile activation input.  The baseline
/// measure cut also uses the baseline strain source, so no other geometry
/// input is left active in the cut clone.
fn r9_g_to_p_record(seed: &R7BoundarySeed) -> Result<Value, String> {
    let n = seed.cohort.mesh.n();
    let baseline_measures = (0..n)
        .map(|index| seed.cohort.mesh.edge_length(index))
        .collect::<Vec<_>>();
    let mean_edge = baseline_measures.iter().sum::<f64>() / n.max(1) as f64;
    let modes = r8_normal_harmonic_basis(&seed.cohort.mesh);
    let mut mode_records = Vec::new();
    for (harmonic, phase, mode) in modes {
        let mut scales = Vec::new();
        for normalized in [1.0e-3_f64, 5.0e-4_f64] {
            let amplitude = normalized * mean_edge;
            let mut signs = Vec::new();
            for sign in [1.0_f64, -1.0_f64] {
                let cohort = match r8_perturbed_cohort(seed, &mode, amplitude, sign) {
                    Ok(cohort) => cohort,
                    Err(reason) => {
                        signs.push(json!({"status": "NONSMOOTH_OR_INVALID", "reason": reason}));
                        continue;
                    }
                };
                let actual_measures = (0..cohort.mesh.n())
                    .map(|index| cohort.mesh.edge_length(index))
                    .collect::<Vec<_>>();
                let full = r9_polarity_update(seed, &cohort, &actual_measures, &cohort);
                let cut = r9_polarity_update(seed, &cohort, &baseline_measures, &seed.cohort);
                match (full, cut) {
                    (Ok(full), Ok(cut)) => {
                        let active_delta = r8_amount_difference(&full, &cut, "active_after");
                        let inactive_delta = r8_amount_difference(&full, &cut, "inactive_after");
                        signs.push(json!({
                            "status": "VALID",
                            "full": full,
                            "cut": cut,
                            "active_delta": active_delta,
                            "inactive_delta": inactive_delta,
                        }));
                    }
                    (full, cut) => signs.push(json!({
                        "status": "NONSMOOTH_OR_INVALID",
                        "full_error": full.err(),
                        "cut_error": cut.err(),
                    })),
                }
            }
            let response = |record: &Value| {
                let mut values = record["active_delta"].as_array().cloned().unwrap_or_default()
                    .iter()
                    .map(|value| value.as_f64().unwrap_or(f64::NAN))
                    .collect::<Vec<_>>();
                values.extend(
                    record["inactive_delta"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|value| value.as_f64().unwrap_or(f64::NAN)),
                );
                values
            };
            let symmetry = if signs.len() == 2 && signs.iter().all(|record| record["status"] == "VALID") {
                let plus = response(&signs[0]);
                let minus = response(&signs[1]);
                let sum = plus
                    .iter()
                    .zip(&minus)
                    .map(|(left, right)| left + right)
                    .collect::<Vec<_>>();
                let difference = plus
                    .iter()
                    .zip(&minus)
                    .map(|(left, right)| left - right)
                    .collect::<Vec<_>>();
                let opposite_error = r8_vector_norm(&sum)
                    / r8_vector_norm(&difference).max(1.0e-300);
                json!({
                    "status": if opposite_error <= 0.05 { "PASS" } else { "FAIL" },
                    "opposite_error": opposite_error,
                    "plus_norm": r8_vector_norm(&plus),
                    "minus_norm": r8_vector_norm(&minus),
                })
            } else {
                json!({"status": "NONSMOOTH_OR_INVALID"})
            };
            scales.push(json!({
                "normalized_amplitude": normalized,
                "amplitude": amplitude,
                "signs": signs,
                "sign_symmetry": symmetry,
            }));
        }
        let valid = scales.iter().all(|scale| {
            scale["sign_symmetry"]["status"] == "PASS"
                && scale["signs"].as_array().is_some_and(|signs| {
                    signs.len() == 2 && signs.iter().all(|sign| sign["status"] == "VALID")
                })
        });
        mode_records.push(json!({
            "harmonic": harmonic,
            "phase": phase,
            "scales": scales,
            "status": if valid { "VALID" } else { "G_NONLINEAR_OR_NONSMOOTH" },
        }));
    }
    Ok(json!({
        "status": "PASS",
        "mechanosensitive": true,
        "baseline_measures": baseline_measures,
        "mean_edge_length": mean_edge,
        "modes": mode_records,
        "perturbation_contract": {
            "normalized_amplitudes": [1.0e-3, 5.0e-4],
            "definition": "mean edge length times radial normal Fourier mode",
            "measurement": "FULL minus G_TO_P_CUT immediately after mechanosensitive polarity.advance and before mechanics",
        },
        "observer_only": true,
    }))
}

fn r8_amount_difference(left: &Value, right: &Value, field: &str) -> Vec<f64> {
    let left = left[field].as_array().cloned().unwrap_or_default();
    let right = right[field].as_array().cloned().unwrap_or_default();
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left.as_f64().unwrap_or(f64::NAN) - right.as_f64().unwrap_or(f64::NAN))
        .collect()
}

fn r8_vector_norm(vector: &[f64]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn r8_point_vector_norm(points: &[[f64; 2]]) -> f64 {
    points
        .iter()
        .flat_map(|point| point.iter())
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt()
}

fn r8_g_to_p_record(seed: &R7BoundarySeed) -> Result<Value, String> {
    let n = seed.cohort.mesh.n();
    let baseline_measures = (0..n)
        .map(|index| seed.cohort.mesh.edge_length(index))
        .collect::<Vec<_>>();
    let mean_edge = baseline_measures.iter().sum::<f64>() / n.max(1) as f64;
    let modes = r8_normal_harmonic_basis(&seed.cohort.mesh);
    let mut mode_records = Vec::new();
    for (harmonic, phase, mode) in modes {
        let mut scales = Vec::new();
        for normalized in [1.0e-3_f64, 5.0e-4_f64] {
            let amplitude = normalized * mean_edge;
            let mut signs = Vec::new();
            for sign in [1.0_f64, -1.0_f64] {
                let cohort = match r8_perturbed_cohort(seed, &mode, amplitude, sign) {
                    Ok(cohort) => cohort,
                    Err(reason) => {
                        signs.push(json!({"status": "NONSMOOTH_OR_INVALID", "reason": reason}));
                        continue;
                    }
                };
                let actual_measures = (0..cohort.mesh.n())
                    .map(|index| cohort.mesh.edge_length(index))
                    .collect::<Vec<_>>();
                let full = r8_polarity_update(seed, &cohort, &actual_measures);
                let cut = r8_polarity_update(seed, &cohort, &baseline_measures);
                match (full, cut) {
                    (Ok(full), Ok(cut)) => {
                        let active_delta = r8_amount_difference(&full, &cut, "active_after");
                        let inactive_delta = r8_amount_difference(&full, &cut, "inactive_after");
                        signs.push(json!({
                            "status": "VALID",
                            "full": full,
                            "cut": cut,
                            "active_delta": active_delta,
                            "inactive_delta": inactive_delta,
                        }));
                    }
                    (full, cut) => signs.push(json!({
                        "status": "NONSMOOTH_OR_INVALID",
                        "full_error": full.err(),
                        "cut_error": cut.err(),
                    })),
                }
            }
            let response = |record: &Value| {
                let mut values = record["active_delta"].as_array().cloned().unwrap_or_default()
                    .iter()
                    .map(|value| value.as_f64().unwrap_or(f64::NAN))
                    .collect::<Vec<_>>();
                values.extend(
                    record["inactive_delta"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|value| value.as_f64().unwrap_or(f64::NAN)),
                );
                values
            };
            let symmetry = if signs.len() == 2 && signs.iter().all(|record| record["status"] == "VALID") {
                let plus = response(&signs[0]);
                let minus = response(&signs[1]);
                let sum = plus
                    .iter()
                    .zip(&minus)
                    .map(|(left, right)| left + right)
                    .collect::<Vec<_>>();
                let difference = plus
                    .iter()
                    .zip(&minus)
                    .map(|(left, right)| left - right)
                    .collect::<Vec<_>>();
                let opposite_error = r8_vector_norm(&sum) / r8_vector_norm(&difference).max(1.0e-300);
                json!({
                    "status": if opposite_error <= 0.05 { "PASS" } else { "FAIL" },
                    "opposite_error": opposite_error,
                    "plus_norm": r8_vector_norm(&plus),
                    "minus_norm": r8_vector_norm(&minus),
                })
            } else {
                json!({"status": "NONSMOOTH_OR_INVALID"})
            };
            scales.push(json!({
                "normalized_amplitude": normalized,
                "amplitude": amplitude,
                "signs": signs,
                "sign_symmetry": symmetry,
            }));
        }
        let valid = scales.iter().all(|scale| {
            scale["sign_symmetry"]["status"] == "PASS"
                && scale["signs"].as_array().is_some_and(|signs| {
                    signs.len() == 2 && signs.iter().all(|sign| sign["status"] == "VALID")
                })
        });
        mode_records.push(json!({
            "harmonic": harmonic,
            "phase": phase,
            "scales": scales,
            "status": if valid { "VALID" } else { "G_NONLINEAR_OR_NONSMOOTH" },
        }));
    }
    Ok(json!({
        "status": "PASS",
        "baseline_measures": baseline_measures,
        "mean_edge_length": mean_edge,
        "modes": mode_records,
        "perturbation_contract": {
            "normalized_amplitudes": [1.0e-3, 5.0e-4],
            "definition": "mean edge length times radial normal Fourier mode",
            "measurement": "FULL minus G_TO_P_CUT immediately after polarity.advance and before mechanics",
        },
        "observer_only": true,
    }))
}

fn r8_rotate_seed(seed: &R7BoundarySeed, angle: f64) -> R7BoundarySeed {
    let mut rotated = seed.clone();
    let (sin, cos) = angle.sin_cos();
    for point in &mut rotated.cohort.mesh.vertices {
        *point = [cos * point[0] - sin * point[1], sin * point[0] + cos * point[1]];
    }
    rotated
}

fn r8_p_to_m_scalar_summary(value: &Value) -> Value {
    json!({
        "status": value["status"].clone(),
        "mechanical_delta_norm": value["mechanical_delta_norm"].clone(),
        "full_displacement_norm": value["full_displacement_norm"].clone(),
        "active_work_delta": value["active_work_delta"].clone(),
        "polarity_activity_harmonics": value["polarity_activity_harmonics"].clone(),
        "normal_modal_delta": value["normal_modal_delta"].clone(),
        "polarity_chemistry_a_delta": value["polarity_chemistry_a_delta"].clone(),
    })
}

fn r8_rotation_control(seed: &R7BoundarySeed) -> Result<Value, String> {
    let rotated = r8_rotate_seed(seed, 0.371_f64);
    let original = r8_p_to_m_record(seed)?;
    let rotated_result = r8_p_to_m_record(&rotated)?;
    let scalar_equal = original["status"] == rotated_result["status"]
        && (original["mechanical_delta_norm"].as_f64().unwrap_or(f64::NAN)
            - rotated_result["mechanical_delta_norm"].as_f64().unwrap_or(f64::NAN))
            .abs()
            <= 1.0e-8
        && (original["full_displacement_norm"].as_f64().unwrap_or(f64::NAN)
            - rotated_result["full_displacement_norm"].as_f64().unwrap_or(f64::NAN))
            .abs()
            <= 1.0e-8;
    let modal_invariant = original["normal_modal_delta"]
        .as_array()
        .zip(rotated_result["normal_modal_delta"].as_array())
        .map(|(left, right)| {
            left.len() == right.len()
                && left.iter().zip(right).all(|(left, right)| {
                    (left["projection"].as_f64().unwrap_or(f64::NAN)
                        - right["projection"].as_f64().unwrap_or(f64::NAN))
                        .abs()
                        <= 1.0e-8
                })
        })
        .unwrap_or(false);
    let activity_invariant = original["polarity_activity_harmonics"]
        .as_array()
        .zip(rotated_result["polarity_activity_harmonics"].as_array())
        .map(|(left, right)| {
            left.len() == right.len()
                && left.iter().zip(right).all(|(left, right)| {
                    (left["cosine"].as_f64().unwrap_or(f64::NAN)
                        - right["cosine"].as_f64().unwrap_or(f64::NAN))
                        .abs()
                        <= 1.0e-8
                        && (left["sine"].as_f64().unwrap_or(f64::NAN)
                            - right["sine"].as_f64().unwrap_or(f64::NAN))
                            .abs()
                            <= 1.0e-8
                })
        })
        .unwrap_or(false);
    Ok(json!({
        "rigid_rotation_angle": 0.371_f64,
        "scalar_response_invariant": scalar_equal && modal_invariant && activity_invariant,
        "modal_response_invariant": modal_invariant,
        "activity_spectrum_invariant": activity_invariant,
        "original": r8_p_to_m_scalar_summary(&original),
        "rotated": r8_p_to_m_scalar_summary(&rotated_result),
        "observer_only": true,
    }))
}

/// R8 entry point.  The canonical R7 arm is executed only to recreate its
/// sealed same-phase replay seeds.  All causal cuts and geometry perturbations
/// thereafter run on clones and are discarded before the arm returns.
pub fn run_r8_modular_edge_arm(index: usize) -> Value {
    let mut snapshots = Vec::new();
    let result = run_r4_polarity_arm_with_atomic_capture(
        index,
        true,
        None,
        Some(&mut snapshots),
    );
    let mut records = Vec::new();
    for snapshot in snapshots.iter() {
        let record = match r8_seed_from_snapshot(snapshot).and_then(|seed| {
            let p_to_m = r8_p_to_m_record(&seed)?;
            let g_to_p = r8_g_to_p_record(&seed)?;
            let rotation = r8_rotation_control(&seed)?;
            Ok(json!({
                "checkpoint_step": snapshot["checkpoint_step"],
                "expected_next_step": snapshot["expected_next_step"],
                "input_boundary_digest": deterministic_state_digest(&snapshot["input_boundary"]),
                "r7_replay_identity": snapshot["identity"].clone(),
                "p_to_m": p_to_m,
                "g_to_p": g_to_p,
                "rotation_control": rotation,
                "observer_only": true,
            }))
        }) {
            Ok(record) => record,
            Err(reason) => json!({
                "checkpoint_step": snapshot["checkpoint_step"],
                "status": "ERROR",
                "error": reason,
                "observer_only": true,
            }),
        };
        records.push(record);
    }
    json!({
        "arm": result["arm"].clone(),
        "connected": true,
        "accepted": result["accepted"].clone(),
        "accepted_steps": result["accepted_steps"].clone(),
        "physical_fissions": result["physical_fissions"].clone(),
        "mechanical_mode_classification": result["mechanical_mode_classification"].clone(),
        "r7_snapshot_count": records.len(),
        "snapshots": records,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "observer_only": true,
        "production_transition_modified": false,
    })
}

/// R9 reuses the exact R8 fixed histories/checkpoints and compares the
/// accepted R4 G_TO_P response with the one opt-in mechanosensitive chemistry
/// path.  The two records are computed from the same serialized boundary
/// seed; neither result is fed into the production transition.
pub fn run_r9_mechanosensitive_edge_arm(index: usize) -> Value {
    let r7 = run_r7_full_cycle_arm(index, true);
    let snapshots = r7["full_cycle_snapshots"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let records = snapshots
        .iter()
        .map(|snapshot| {
            let record = r8_seed_from_snapshot(snapshot).and_then(|seed| {
                let p_to_m = r8_p_to_m_record(&seed)?;
                let baseline = r8_g_to_p_record(&seed)?;
                let mechanosensitive = r9_g_to_p_record(&seed)?;
                Ok(json!({
                    "checkpoint_step": snapshot["checkpoint_step"],
                    "expected_next_step": snapshot["expected_next_step"],
                    "input_boundary_digest": deterministic_state_digest(&snapshot["input_boundary"]),
                    "r7_replay_identity": snapshot["identity"].clone(),
                    "p_to_m": p_to_m,
                    "r4_baseline": baseline,
                    "r9_mechanosensitive": mechanosensitive,
                    "same_snapshot_for_conditions": true,
                    "observer_only": true,
                }))
            });
            match record {
                Ok(value) => value,
                Err(reason) => json!({
                    "checkpoint_step": snapshot["checkpoint_step"],
                    "status": "ERROR",
                    "error": reason,
                    "observer_only": true,
                }),
            }
        })
        .collect::<Vec<_>>();
    json!({
        "arm": r7["arm"].clone(),
        "connected": true,
        "accepted": r7["accepted"].clone(),
        "accepted_steps": r7["accepted_steps"].clone(),
        "physical_fissions": r7["physical_fissions"].clone(),
        "r7_snapshot_count": records.len(),
        "snapshots": records,
        "mechanosensitive_equation": "(1 + max(0, load_bearing_strain_i)) * existing_R3_activation_term - existing_R3_deactivation_term",
        "mechanosensitive_coefficient": "NONE; unit coefficient is fixed by the parameter-free contract",
        "production_transition_modified": false,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "observer_only": true,
    })
}

/// R10 reuses the exact thirty R8 same-phase snapshots and applies the
/// polarity-to-inward-normal remap only to discarded clone transitions.  The
/// accepted R4 edge-tension transition is retained as the route-OFF reference;
/// no R10 route is connected to the production lifecycle.
pub fn run_r10_polarity_normal_remap_arm(index: usize) -> Value {
    env::remove_var("DCFINAL001_R9_MECHANOSENSITIVE");
    env::set_var("DCFINAL001_R8_CAPTURE_ONLY", "1");
    let r7 = run_r7_full_cycle_arm(index, true);
    let snapshots = r7["full_cycle_snapshots"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let records = snapshots
        .iter()
        .map(|snapshot| {
            let record = r8_seed_from_snapshot(snapshot).and_then(|seed| {
                let r10 = r10_normal_remap_record(&seed)?;
                Ok(json!({
                    "checkpoint_step": snapshot["checkpoint_step"],
                    "expected_next_step": snapshot["expected_next_step"],
                    "input_boundary_digest": deterministic_state_digest(&snapshot["input_boundary"]),
                    "r7_replay_identity": snapshot["identity"].clone(),
                    "r10": r10,
                    "same_snapshot_for_conditions": true,
                    "observer_only": true,
                }))
            });
            match record {
                Ok(value) => value,
                Err(reason) => json!({
                    "checkpoint_step": snapshot["checkpoint_step"],
                    "status": "ERROR",
                    "error": reason,
                    "observer_only": true,
                }),
            }
        })
        .collect::<Vec<_>>();
    json!({
        "arm": r7["arm"].clone(),
        "connected": true,
        "accepted": r7["accepted"].clone(),
        "accepted_steps": r7["accepted_steps"].clone(),
        "physical_fissions": r7["physical_fissions"].clone(),
        "r7_snapshot_count": records.len(),
        "snapshots": records,
        "candidate_equation": "d_i_R10 = min(1, d_i_legacy + p_i)",
        "candidate_route": "R10_INWARD_NORMAL",
        "polarity_edge_tension": "DISABLED_ON_CANDIDATE",
        "production_transition_modified": false,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "observer_only": true,
    })
}

/// Run one current-kernel R4 parent arm under coherent Resource conditions.
/// This is the bounded ten-arm pre-fission comparison; it does not launch
/// population selection or alter the production default.
pub fn run_r4_polarity_arm(index: usize, connected: bool) -> Value {
    run_r4_polarity_arm_with_atomic_capture(index, connected, None, None)
}

/// R6-only diagnostic entry point. It executes the same R4 arm with atomic
/// pre-mechanics snapshots and in-process unperturbed replays. The capture is
/// observer-only and the default R4 entry point above remains unchanged.
pub fn run_r6_atomic_replay_arm(index: usize, connected: bool) -> Value {
    let mut snapshots = Vec::new();
    let mut result =
        run_r4_polarity_arm_with_atomic_capture(index, connected, Some(&mut snapshots), None);
    result["atomic_replay_snapshots"] = json!(snapshots);
    result
}

/// R7-only diagnostic entry point.  It captures the complete pre-polarity
/// boundary and runs the same-phase response probe.  The R4 production entry
/// point and R6 atomic replay entry point remain unchanged.
pub fn run_r7_full_cycle_arm(index: usize, connected: bool) -> Value {
    let mut snapshots = Vec::new();
    let mut result =
        run_r4_polarity_arm_with_atomic_capture(index, connected, None, Some(&mut snapshots));
    result["full_cycle_snapshots"] = json!(snapshots);
    result
}

fn run_r4_polarity_arm_with_atomic_capture(
    index: usize,
    connected: bool,
    mut atomic_replay_snapshots: Option<&mut Vec<Value>>,
    mut full_cycle_snapshots: Option<&mut Vec<Value>>,
) -> Value {
    assert!(index < 10);
    let allocation = AllocationParams::default();
    let mut mesh = r10_closure::r10_reproduction_fixture(index);
    let birth_mass = mesh.total_structural_mass();
    mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
    let initialization = initialize_r4_polarity(&mut mesh).expect("lawful R4 polarity source");
    let mut polarity_states = PolarityStateMap::new();
    polarity_states.insert(1, initialization.0);
    let cohort = Cohort {
        plasticity: Some(PlasticityStateV1::new(mesh.n())),
        mesh,
        count: 1,
        generation: 0,
        birth_mass,
        id: 1,
    };
    let mut cohorts = vec![cohort];
    let mut world = OpenMedium::new(Environment::Resource, R10R9R4_FOUNDER_MULTIPLICITY as f64);
    let initial = snapshot(&cohorts, &world, 0);
    let mut ledger = CampaignLedger::default();
    ledger.polarity_source_a_debited = initialization.1["source_ledger"]["source_a_debited"]
        .as_f64()
        .unwrap_or(0.0);
    let mut next_id = 2;
    let mut trajectory = vec![initial];
    let mut prefix = None;
    let mut daughter_continuations = Vec::new();
    let mut mechanics_trace = Vec::new();
    let accepted = r10_advance_phase(
        &mut cohorts,
        &mut world,
        Environment::Resource,
        0,
        false,
        splitmix64((index + 1) as u64),
        &mut next_id,
        &mut ledger,
        &mut trajectory,
        &mut prefix,
        R10R2_PHASE_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        FissionClockMode::CurrentStep,
        ReserveResolutionMode::PerStep,
        &mut daughter_continuations,
        false,
        DiagnosticBoundarySource::Scheduled,
        DiagnosticBoundarySource::Scheduled,
        true,
        true,
        GrowthPlacementMode::FrozenD088,
        Some(&mut mechanics_trace),
        Some(&mut polarity_states),
        connected,
        atomic_replay_snapshots.as_deref_mut(),
        full_cycle_snapshots.as_deref_mut(),
    );
    let terminal_step = ledger.accepted_steps as usize;
    let terminal = snapshot(&cohorts, &world, terminal_step);
    let polarity_total = polarity_states
        .values()
        .map(PolarityMassStateV1::total_amount)
        .sum::<f64>();
    let polarity_active = polarity_states
        .values()
        .map(PolarityMassStateV1::active_total)
        .sum::<f64>();
    let first_mode = mechanics_trace
        .first()
        .and_then(|row| row["material_geometry"]["shape_modes"]["modes"].as_array())
        .and_then(|modes| {
            modes
                .iter()
                .map(|mode| mode["normalized_amplitude"].as_f64().unwrap_or(0.0))
                .reduce(f64::max)
        })
        .unwrap_or(0.0);
    let late_mode = mechanics_trace
        .last()
        .and_then(|row| row["material_geometry"]["shape_modes"]["modes"].as_array())
        .and_then(|modes| {
            modes
                .iter()
                .map(|mode| mode["normalized_amplitude"].as_f64().unwrap_or(0.0))
                .reduce(f64::max)
        })
        .unwrap_or(0.0);
    let mode_ratio = late_mode / first_mode.max(1e-300);
    let classification = if mode_ratio > 1.001 {
        "GROWING"
    } else if mode_ratio < 0.999 {
        "DECAYING"
    } else {
        "NEUTRAL"
    };
    let nearest_distance = |row: &Value| {
        row["material_geometry"]["segment_geometry"]["nearest_geometrically_eligible_pair"]
            ["distance_over_range"]
            .as_f64()
    };
    let first_distance = mechanics_trace.first().and_then(nearest_distance);
    let late_distance = mechanics_trace.last().and_then(nearest_distance);
    let mut result = json!({
        "arm": index + 1,
        "connected": connected,
        "driver": "SHARED_CURRENT_KERNEL_R4_POLARITY_TO_EXISTING_PAID_ACTUATOR",
        "accepted": accepted,
        "accepted_steps": ledger.accepted_steps,
        "birth_mass": birth_mass,
        "maximum_mass_over_birth": cohorts.iter().map(|cohort| cohort.mesh.total_structural_mass() / cohort.birth_mass.max(1e-300)).fold(1.0_f64, f64::max),
        "physical_fissions": ledger.physical_fissions,
        "post_bootstrap_physical_fissions": ledger.post_bootstrap_physical_fissions,
        "fission_attempts": ledger.phenotype_by_genotype.values().map(|value| value.fission_attempts).sum::<u64>(),
        "first_mode_amplitude": first_mode,
        "late_mode_amplitude": late_mode,
        "mechanical_mode_ratio": mode_ratio,
        "mechanical_mode_classification": classification,
        "first_nearest_distance_over_range": first_distance,
        "late_nearest_distance_over_range": late_distance,
        "polarity_total_amount": polarity_total,
        "polarity_active_amount": polarity_active,
        "polarity_states": polarity_states,
        "initialization": initialization.1,
        "terminal": terminal,
        "trajectory": trajectory,
        "prefix_2500": prefix,
        "mechanics_trace": mechanics_trace,
        "daughter_continuations": daughter_continuations,
        "world": world,
        "ledger": ledger,
        "all_simple": cohorts.iter().all(|cohort| polygon_simple(&cohort.mesh.vertices)),
        "all_runtime_valid": cohorts.iter().all(|cohort| cohort.mesh.physical_runtime_valid()),
        "all_lifecycle_valid": cohorts.iter().all(|cohort| cohort.mesh.lifecycle_invariants_hold()),
        "source_material_and_audit": "explicit_source_debit; no polarity material or mechanical work is hidden",
    });
    if let Some(records) = atomic_replay_snapshots {
        result["atomic_replay_snapshots"] = json!(records);
    }
    result
}

fn r10_current_reproduction_comparison_arm(
    index: usize,
    boundary_mode: PopulationBoundaryMode,
    clock_mode: FissionClockMode,
    reserve_resolution: ReserveResolutionMode,
) -> Value {
    let allocation = AllocationParams::default();
    let mut mesh = r10_closure::r10_reproduction_fixture(index);
    let birth_mass = mesh.total_structural_mass();
    mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
    let cohort = Cohort {
        plasticity: Some(PlasticityStateV1::new(mesh.n())),
        mesh,
        count: 1,
        generation: 0,
        birth_mass,
        id: 1,
    };
    let mut cohorts = vec![cohort];
    let mut world = OpenMedium::new(Environment::Resource, R10R9R4_FOUNDER_MULTIPLICITY as f64);
    let initial = snapshot(&cohorts, &world, 0);
    let mut ledger = CampaignLedger::default();
    let mut next_id = 2;
    let mut trajectory = vec![initial.clone()];
    let mut prefix = None;
    let mut daughter_continuations = Vec::new();
    let accepted = r10_advance_phase(
        &mut cohorts,
        &mut world,
        Environment::Resource,
        0,
        false,
        splitmix64((index + 1) as u64),
        &mut next_id,
        &mut ledger,
        &mut trajectory,
        &mut prefix,
        R10R2_PHASE_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        boundary_mode,
        clock_mode,
        reserve_resolution,
        &mut daughter_continuations,
        true,
        DiagnosticBoundarySource::Scheduled,
        DiagnosticBoundarySource::Scheduled,
        true,
        true,
        GrowthPlacementMode::FrozenD088,
        None,
        None,
        false,
        None,
        None,
    );
    let terminal = snapshot(&cohorts, &world, ledger.accepted_steps as usize);
    let attempts = ledger
        .lifecycle_events
        .iter()
        .filter(|event| event["phase"] == "fission_attempt")
        .cloned()
        .collect::<Vec<_>>();
    let fission_step = ledger
        .physical_birth_events
        .first()
        .and_then(|event| event["step"].as_u64());
    let final_mesh_digest = deterministic_state_digest(
        &cohorts
            .iter()
            .map(|cohort| &cohort.mesh)
            .collect::<Vec<_>>(),
    );
    let final_plasticity_digest = deterministic_state_digest(
        &cohorts
            .iter()
            .map(|cohort| &cohort.plasticity)
            .collect::<Vec<_>>(),
    );
    let accepted_boundary = world.accepted_boundary();
    let boundary_consistent = boundary_mode == PopulationBoundaryMode::FixtureExteriorReference
        || cohorts
            .iter()
            .all(|cohort| cohort.mesh.exterior == accepted_boundary);
    json!({
        "arm": index + 1,
        "driver": "SHARED_CURRENT_KERNEL",
        "boundary_mode": boundary_mode.label(),
        "fission_clock": clock_mode.label(),
        "reserve_resolution": reserve_resolution.label(),
        "accepted": accepted,
        "accepted_steps": ledger.accepted_steps,
        "birth_mass": birth_mass,
        "maximum_mass_over_birth": cohorts.iter().map(|cohort| {
            cohort.mesh.total_structural_mass() / cohort.birth_mass.max(1e-300)
        }).fold(1.0_f64, f64::max),
        "physical_fissions": ledger.physical_fissions,
        "post_bootstrap_physical_fissions": ledger.post_bootstrap_physical_fissions,
        "fission_step": fission_step,
        "fission_attempt_count": attempts.len(),
        "fission_attempts": attempts,
        "physical_birth_events": ledger.physical_birth_events,
        "daughter_continuations": daughter_continuations,
        "terminal": terminal,
        "trajectory": trajectory,
        "prefix_2500": prefix,
        "final_mesh_digest": final_mesh_digest,
        "final_plasticity_digest": final_plasticity_digest,
        "boundary_state": {
            "mode": boundary_mode.label(),
            "accepted_bath_boundary": accepted_boundary,
            "all_surviving_cohort_exteriors_match_bath": boundary_consistent,
            "mechanics_boundary_contract": if boundary_mode == PopulationBoundaryMode::FixtureExteriorReference {
                "explicit historical fixture exterior"
            } else {
                "accepted finite bath after complete exchange"
            },
        },
        "world": world,
        "ledger_summary": {
            "rejected_steps": ledger.rejected_steps,
            "computational_rejections": ledger.computational_rejections,
            "runtime_invalidations": ledger.runtime_invalidations,
            "n_closure_relevant": [world.ledger.initial_n, world.n_mass],
            "f_closure_relevant": [world.ledger.initial_f, world.f_mass],
            "active_a_spent": ledger.active_a_spent,
            "active_w_produced": ledger.active_w_produced,
        },
    })
}

#[derive(Debug, Clone, Copy)]
struct CausalBoundaryCell {
    label: &'static str,
    fixture_n: bool,
    fixture_f: bool,
}

const R5_CAUSAL_BOUNDARY_CELLS: [CausalBoundaryCell; 4] = [
    CausalBoundaryCell {
        label: "A_RESOURCE_N_RESOURCE_F_CURRENT_CONTROL",
        fixture_n: false,
        fixture_f: false,
    },
    CausalBoundaryCell {
        label: "B_FIXTURE_N_RESOURCE_F",
        fixture_n: true,
        fixture_f: false,
    },
    CausalBoundaryCell {
        label: "C_RESOURCE_N_FIXTURE_F",
        fixture_n: false,
        fixture_f: true,
    },
    CausalBoundaryCell {
        label: "D_FIXTURE_N_FIXTURE_F",
        fixture_n: true,
        fixture_f: true,
    },
];

fn r10_causal_arm(
    index: usize,
    cell: CausalBoundaryCell,
    motor_enabled: bool,
    adaptation_enabled: bool,
    growth_placement: GrowthPlacementMode,
) -> Value {
    let allocation = AllocationParams::default();
    let mut mesh = r10_closure::r10_reproduction_fixture(index);
    let fixture_n = mesh.exterior.n;
    let fixture_f = mesh.exterior.f;
    let birth_mass = mesh.total_structural_mass();
    mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
    let initial_exterior = mesh.exterior;
    let cohort = Cohort {
        plasticity: Some(PlasticityStateV1::new(mesh.n())),
        mesh,
        count: 1,
        generation: 0,
        birth_mass,
        id: 1,
    };
    let mut cohorts = vec![cohort];
    let mut world = OpenMedium::new(Environment::Resource, R10R9R4_FOUNDER_MULTIPLICITY as f64);
    let initial = snapshot(&cohorts, &world, 0);
    let mut ledger = CampaignLedger::default();
    let mut next_id = 2;
    let mut trajectory = vec![initial];
    let mut prefix = None;
    let mut daughter_continuations = Vec::new();
    let mut mechanics_trace = Vec::new();
    let n_source = if cell.fixture_n {
        DiagnosticBoundarySource::Fixture(fixture_n)
    } else {
        DiagnosticBoundarySource::Scheduled
    };
    let f_source = if cell.fixture_f {
        DiagnosticBoundarySource::Fixture(fixture_f)
    } else {
        DiagnosticBoundarySource::Scheduled
    };
    let accepted = r10_advance_phase(
        &mut cohorts,
        &mut world,
        Environment::Resource,
        0,
        false,
        splitmix64((index + 1) as u64),
        &mut next_id,
        &mut ledger,
        &mut trajectory,
        &mut prefix,
        R10R2_PHASE_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        FissionClockMode::CurrentStep,
        ReserveResolutionMode::PerStep,
        &mut daughter_continuations,
        true,
        n_source,
        f_source,
        motor_enabled,
        adaptation_enabled,
        growth_placement,
        Some(&mut mechanics_trace),
        None,
        false,
        None,
        None,
    );
    let terminal_step = ledger.accepted_steps as usize;
    let terminal = snapshot(&cohorts, &world, terminal_step);
    let fission_attempts = ledger
        .lifecycle_events
        .iter()
        .filter(|event| event["phase"] == "fission_attempt")
        .cloned()
        .collect::<Vec<_>>();
    let fission_steps = ledger
        .physical_birth_events
        .iter()
        .filter_map(|event| event["step"].as_u64())
        .collect::<Vec<_>>();
    let terminal_cohorts = terminal_cohort_diagnostics(&cohorts, &FissionParams::default());
    let active_energy_residual = (ledger.active_a_spent - ledger.active_w_produced).abs();
    let daughter_branches_viable = daughter_continuations
        .iter()
        .filter(|row| row["continuation"]["viable"] == true)
        .count();
    let full_state_viable_pairs = usize::from(
        ledger.physical_fissions > 0
            && daughter_continuations.len() == 2
            && daughter_branches_viable == 2,
    );
    json!({
        "cell": cell.label,
        "arm": index + 1,
        "environment": Environment::Resource.label(),
        "boundary_sources": {
            "n": if cell.fixture_n { "EXACT_PER_ARM_FIXTURE_EXTERIOR" } else { "RESOURCE_WAVEFORM" },
            "f": if cell.fixture_f { "EXACT_PER_ARM_FIXTURE_EXTERIOR" } else { "RESOURCE_WAVEFORM" },
            "fixture_n": fixture_n,
            "fixture_f": fixture_f,
            "initial_mesh_exterior": initial_exterior,
        },
        "motor_enabled": motor_enabled,
        "adaptation_enabled": adaptation_enabled,
        "accepted": accepted,
        "accepted_steps": ledger.accepted_steps,
        "birth_mass": birth_mass,
        "maximum_mass_over_birth": cohorts.iter().map(|cohort| {
            cohort.mesh.total_structural_mass() / cohort.birth_mass.max(1e-300)
        }).fold(1.0_f64, f64::max),
        "growth_qualified": cohorts.iter().any(|cohort| {
            cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass
        }) || ledger.physical_fissions > 0,
        "physical_fissions": ledger.physical_fissions,
        "full_state_viable_pairs": full_state_viable_pairs,
        "full_state_viable_daughter_branches": daughter_branches_viable,
        "fission_steps": fission_steps,
        "fission_attempts": fission_attempts,
        "daughter_continuations": daughter_continuations,
        "trajectory": trajectory,
        "mechanics_trace": mechanics_trace,
        "terminal": terminal,
        "terminal_cohorts": terminal_cohorts,
        "world": world,
        "ledger": ledger,
        "active_energy_residual": active_energy_residual,
        "material_energy": {
            "active_a_spent": ledger.active_a_spent,
            "active_w_produced": ledger.active_w_produced,
            "reaction_n_consumed": ledger.reaction_n_consumed,
            "reaction_f_consumed": ledger.reaction_f_consumed,
            "growth_material": ledger.growth_material,
            "m1_structural_build": ledger.m1_structural_build,
            "m1_structural_turnover": ledger.m1_structural_turnover,
        },
        "numerical_invalid": ledger.numerical_invalid,
        "rejected_steps": ledger.rejected_steps,
    })
}

fn r10_causal_cell(
    cell: CausalBoundaryCell,
    motor_enabled: bool,
    adaptation_enabled: bool,
) -> Value {
    r10_causal_cell_with_growth_placement(
        cell,
        motor_enabled,
        adaptation_enabled,
        GrowthPlacementMode::FrozenD088,
    )
}

fn r10_causal_cell_with_growth_placement(
    cell: CausalBoundaryCell,
    motor_enabled: bool,
    adaptation_enabled: bool,
    growth_placement: GrowthPlacementMode,
) -> Value {
    let arms = (0..10)
        .map(|index| {
            r10_causal_arm(
                index,
                cell,
                motor_enabled,
                adaptation_enabled,
                growth_placement,
            )
        })
        .collect::<Vec<_>>();
    let physical_fissions = arms
        .iter()
        .map(|arm| arm["physical_fissions"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let successful_arms = arms
        .iter()
        .filter(|arm| arm["physical_fissions"].as_u64().unwrap_or(0) > 0)
        .count();
    let viable_pairs = arms
        .iter()
        .map(|arm| arm["full_state_viable_pairs"].as_u64().unwrap_or(0))
        .sum::<u64>();
    json!({
        "label": cell.label,
        "fixture_n": cell.fixture_n,
        "fixture_f": cell.fixture_f,
        "motor_enabled": motor_enabled,
        "adaptation_enabled": adaptation_enabled,
        "growth_placement": format!("{growth_placement:?}"),
        "phase_steps": R10R2_PHASE_STEPS,
        "arms": arms,
        "counts": {
            "growth_qualified_arms": arms.iter().filter(|arm| arm["growth_qualified"] == true).count(),
            "physical_fissions": physical_fissions,
            "distinct_successful_arms": successful_arms,
            "full_state_viable_pairs": viable_pairs,
            "attempts": arms.iter().map(|arm| arm["fission_attempts"].as_array().map(Vec::len).unwrap_or(0)).sum::<usize>(),
        },
    })
}

/// Execute the bounded R5 causal comparison. Four N/F boundary cells and two
/// named cortex controls use the shared production lifecycle; none is a
/// production ecology replacement or a selection run.
pub fn run_r10r9r5_causal_comparison() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r10r9r5_causal_comparison.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let mut cells = R5_CAUSAL_BOUNDARY_CELLS
        .into_iter()
        .map(|cell| r10_causal_cell(cell, true, true))
        .collect::<Vec<_>>();
    let motor_off = r10_causal_cell(R5_CAUSAL_BOUNDARY_CELLS[0], false, true);
    let adaptation_disabled = r10_causal_cell(R5_CAUSAL_BOUNDARY_CELLS[0], true, false);
    let resource_current = cells.remove(0);
    let mut boundary_cells = Vec::with_capacity(4);
    boundary_cells.push(resource_current);
    boundary_cells.extend(cells);
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R10R9R5-CANONICAL-LIFECYCLE-AND-EVOLUTION-EVIDENCE-RECOVERY-001",
            "gate": "COHERENT_RESOURCE_MORPHOGENETIC_DRIVE_ATTRIBUTION",
            "scientific_runtime_changed": false,
            "population_campaign": "NOT_RUN",
            "phase_steps": R10R2_PHASE_STEPS,
            "bath_volume": R10R9R4_FOUNDER_MULTIPLICITY,
            "founder_multiplicity": 1,
            "thresholds": {
                "fission_mass_gate": 1.35,
                "apposition_range": "FissionParams::default().topo local_rebond_range",
                "stress": 0.15,
                "daughter_continuation_steps": DAUGHTER_CONTINUATION_STEPS,
            },
            "boundary_cells": boundary_cells,
            "controls": {
                "motor_off": motor_off,
                "adaptation_disabled": adaptation_disabled,
            },
            "interpretation_contract": {
                "fixture_boundary": "diagnostic reference only",
                "resource_boundary": "coherent finite bath published to mechanics",
                "diagnostic_controls": "named mechanism intervention only; no production candidate",
                "no_new_parameters": true,
                "nearest_geometric_pair_includes_out_of_range": true,
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

/// Execute the DC-M4 reproductive-attractor architecture gate. This is an
/// observer-only extension of the already-authoritative R5 causal runner:
/// Resource and fully fixture-bound arms use the same accepted transition,
/// while each sampled accepted state carries material/rest-geometry,
/// component-force, cortex, and frozen-mechanics perturbation records.
pub fn run_dc_m4_architecture_gate() {
    let mut output = PathBuf::from("/tmp/dcm4_reproductive_attractor.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let resource = r10_causal_cell(R5_CAUSAL_BOUNDARY_CELLS[0], true, true);
    let fixture = r10_causal_cell(R5_CAUSAL_BOUNDARY_CELLS[3], true, true);
    let motor_off = r10_causal_cell(R5_CAUSAL_BOUNDARY_CELLS[0], false, true);
    let adaptation_disabled = r10_causal_cell(R5_CAUSAL_BOUNDARY_CELLS[0], true, false);
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-M4-REPRODUCTIVE-ATTRACTOR-ARCHITECTURE-001",
            "observer_only": true,
            "production_biology_changed": false,
            "resource_ecology_changed": false,
            "accepted_horizon": R10R2_PHASE_STEPS,
            "matched_arm_count": 10,
            "cells": {
                "resource_n_resource_f": resource,
                "fixture_n_fixture_f": fixture,
            },
            "controls": {
                "resource_motor_off": motor_off,
                "resource_adaptation_disabled": adaptation_disabled,
            },
            "stability_contract": {
                "mechanics": "frozen local mechanics on cloned accepted snapshots",
                "modes": [2, 3, 4],
                "perturbation": "one deterministic zero-mean radial Fourier mode",
                "epsilon": STABILITY_PROBE_EPSILON,
                "steps": STABILITY_PROBE_STEPS,
                "classification_tolerance": STABILITY_CLASSIFICATION_TOLERANCE,
                "no_shape_or_neck_target": true,
                "no_persistent_external_force": true,
            },
            "interpretation_contract": {
                "fixture_boundary": "diagnostic reproductive reference only",
                "resource_boundary": "current coherent finite bath",
                "all_metrics_observer_only": true,
                "no_values_feed_back_into_biology": true,
                "no_success_conditioned_execution": true,
            },
        }))
        .unwrap(),
    )
    .unwrap();
}

/// Execute the bounded Route-A R1 comparison.  The route-on candidate is an
/// opt-in destination-only redistribution of frozen D-088 growth; all other
/// operators, ecology, cadence, and fission contracts remain shared with the
/// accepted R5 path.  This function intentionally stops at the mechanistic
/// gate: population selection is outside this directive.
pub fn run_dc_m4_r1_local_growth_coupling() {
    let mut output = PathBuf::from("/tmp/dcm4r1_local_conservative_growth_coupling.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let resource = R5_CAUSAL_BOUNDARY_CELLS[0];
    let route_off = r10_causal_cell_with_growth_placement(
        resource,
        true,
        true,
        GrowthPlacementMode::FrozenD088,
    );
    let route_on = r10_causal_cell_with_growth_placement(
        resource,
        true,
        true,
        GrowthPlacementMode::LocalCompressionNeighborV1,
    );
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-M4-R1-LOCAL-CONSERVATIVE-GROWTH-COUPLING-001",
            "observer_only": false,
            "candidate_is_opt_in": true,
            "production_ecology": "R10R9R5_COHERENT_RESOURCE_UNCHANGED",
            "accepted_horizon": R10R2_PHASE_STEPS,
            "matched_arm_count": 10,
            "growth_placement_contract": {
                "route_off": "FROZEN_D088",
                "route_on": "LOCAL_COMPRESSION_NEIGHBOR_V1",
                "base_increment": "exact edge delta produced by frozen growth_step on the same pre-step clone",
                "compression": "q_i = max(0, -strain_i)",
                "neighborhood": "immediate cyclic neighbors only",
                "routing": "self plus compression-gradient neighbor shares; ruptured neighbors are ineligible",
                "maturation": "unchanged; routed increment enters existing m and m_young",
                "dt_application": "base increment is already time-integrated; no second dt",
                "new_state_variables": 0,
                "new_parameters": 0,
                "target_shape_or_division_signal": false,
                "observer_or_population_input": false,
            },
            "cells": {
                "resource_route_off": route_off,
                "resource_route_on": route_on,
            },
            "e2_gate": {
                "valid_growing_modes_per_arm": "route-on must add at least one valid GROWING mode absent from matched route-off",
                "late_distance": "route-on must reduce nearest geometrically eligible pair distance/range in at least 7/10 matched arms",
                "minimum_arms": 7,
                "mechanistic_only_before_e3": true,
            },
            "e3_status": "INTERPRETATION_BLOCKED_UNTIL_E2_VERIFIED",
            "e4_status": "NOT_REACHED_UNTIL_E3_VERIFIED",
        }))
        .unwrap(),
    )
    .unwrap();
}

/// Execute the preregistered R5 reproduction-equivalence matrix.  This is a
/// bounded ten-arm comparison only; it does not run the population ecology.
/// The historical fixture driver, direct-boundary adapter, cadence origin,
/// and reserve-parameter lifetime are varied one at a time so an outcome is
/// not attributed to the shared-kernel refactor without evidence.
pub fn run_r10r9r5_reproduction_equivalence() {
    let output = PathBuf::from("/tmp/dcfinal001_r10r9r5_reproduction_equivalence.json");
    let variants = [
        (
            "shared_fixture_historical_contract",
            PopulationBoundaryMode::FixtureExteriorReference,
            FissionClockMode::HistoricalStepMinusOne,
            ReserveResolutionMode::OnceBeforeTrajectory,
        ),
        (
            "shared_fixture_current_clock",
            PopulationBoundaryMode::FixtureExteriorReference,
            FissionClockMode::CurrentStep,
            ReserveResolutionMode::OnceBeforeTrajectory,
        ),
        (
            "shared_fixture_per_step_reserve",
            PopulationBoundaryMode::FixtureExteriorReference,
            FissionClockMode::HistoricalStepMinusOne,
            ReserveResolutionMode::PerStep,
        ),
        (
            "shared_resource_historical_contract",
            PopulationBoundaryMode::FixedConcentrationBoundary,
            FissionClockMode::HistoricalStepMinusOne,
            ReserveResolutionMode::OnceBeforeTrajectory,
        ),
        (
            "shared_resource_current_contract",
            PopulationBoundaryMode::FixedConcentrationBoundary,
            FissionClockMode::CurrentStep,
            ReserveResolutionMode::PerStep,
        ),
    ];
    let mut handles = Vec::new();
    handles.push(std::thread::spawn(|| {
        (0..10)
            .map(r10_closure::r10_historical_fixture_reproduction_json)
            .collect::<Vec<_>>()
    }));
    for (label, boundary_mode, clock_mode, reserve_resolution) in variants {
        handles.push(std::thread::spawn(move || {
            (0..10)
                .map(|index| {
                    let mut result = r10_current_reproduction_comparison_arm(
                        index,
                        boundary_mode,
                        clock_mode,
                        reserve_resolution,
                    );
                    result
                        .as_object_mut()
                        .expect("comparison arm object")
                        .insert("variant".into(), Value::String(label.into()));
                    result
                })
                .collect::<Vec<_>>()
        }));
    }
    let historical = handles
        .remove(0)
        .join()
        .expect("historical comparison arms");
    let current = handles
        .into_iter()
        .map(|handle| handle.join().expect("current comparison arms"))
        .collect::<Vec<_>>();
    let historical_counts = json!({
        "physical_fissions": historical.iter().filter(|arm| arm["result"]["physical_fission"] == true).count(),
        "full_state_viable_pairs": historical.iter().filter(|arm| arm["result"]["full_state_daughters_viable"] == true).count(),
    });
    let current_counts = current
        .iter()
        .map(|arms| {
            let label = arms
                .first()
                .and_then(|arm| arm["variant"].as_str())
                .unwrap_or("UNKNOWN");
            json!({
                "variant": label,
                "physical_fissions": arms.iter().map(|arm| arm["physical_fissions"].as_u64().unwrap_or(0)).sum::<u64>(),
                "distinct_successful_arms": arms.iter().filter(|arm| arm["physical_fissions"].as_u64().unwrap_or(0) > 0).count(),
                "attempts": arms.iter().map(|arm| arm["fission_attempt_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R10R9R5-CANONICAL-LIFECYCLE-AND-EVOLUTION-EVIDENCE-RECOVERY-001",
            "comparison": "HISTORICAL_DIRECT_FIXTURE_VS_SHARED_CURRENT_KERNEL",
            "population_campaign": "NOT_RUN",
            "phase_steps": R10R2_PHASE_STEPS,
            "expression_path": D096ExpressionPath::V4FiniteBudgetCentered.label(),
            "mutation_enabled": false,
            "reserve_enabled": r10r9r1_reserve_enabled(),
            "historical": historical,
            "historical_counts": historical_counts,
            "current_variants": current,
            "current_counts": current_counts,
            "interpretation_contract": {
                "historical_boundary": "direct fixture mesh.exterior transport",
                "current_boundary": "fixed concentration Resource bath published to mechanics after complete exchange; diagnostic fixture adapter remains explicit",
                "boundary_state_contract": "one accepted finite-bath N/F snapshot is used by transport and subsequent mechanics",
                "transport_allocation_boundary_is_not_mechanical_boundary": true,
                "clock_difference": "historical (absolute_step - 1) modulo cadence vs current step modulo cadence",
                "reserve_difference": "initial mesh resolution vs current mesh resolution each step",
                "no_biological_parameter_change": true,
            },
        }))
        .unwrap(),
    )
    .unwrap();
}

pub fn run_r10r6_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r6_evolution.json",
        "DC-FINAL-001-R10R6-D096-FIXED-CONCENTRATION-BOUNDARY-ECOLOGY-GENERATION-TURNOVER-SELECTION-AND-END-GOAL-CLOSURE-001",
        R10R2_PHASE_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

pub fn run_r10r7_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r7_evolution.json",
        "DC-FINAL-001-R10R7-NATURAL-VARIANT-CROSS-ENVIRONMENT-FEASIBILITY-TWO-WINDOW-SELECTION-REVERSAL-AND-END-GOAL-CLOSURE-001",
        R10R7_SELECTION_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

pub fn run_r10r9r1_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r9r1_evolution.json",
        "DC-FINAL-001-R10R9R1-EXACT-D091-D096V4-COMPOSITION-SPECIALIZATION-SELECTION-AND-END-GOAL-CLOSURE-001",
        R10R7_SELECTION_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

pub fn run_r10r9r3_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r9r3_evolution.json",
        "DC-FINAL-001-R10R9R3-D091V2-BUFFERED-RESERVE-CANONICAL-GROWTH-REPRODUCTION-SPECIALIZATION-AND-M4-CLOSURE-001",
        R10R7_SELECTION_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        FOUNDER_MULTIPLICITY,
        FOUNDER_MULTIPLICITY as f64,
    );
}

/// R10R9R4 scales only the finite assay population and its fixed-concentration
/// vessel together.  Set DCFINAL001_R10R9R4_BASELINE=1 for the matched 150/150
/// mutation-off density-parity control; mutation and all organism biology stay
/// under the caller's existing R10 contract.
pub fn run_r10r9r4_evolution() {
    let baseline = matches!(
        env::var("DCFINAL001_R10R9R4_BASELINE").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    );
    let founder_multiplicity = if baseline {
        FOUNDER_MULTIPLICITY
    } else {
        R10R9R4_FOUNDER_MULTIPLICITY
    };
    let bath_volume = founder_multiplicity as f64;
    let output = if baseline {
        "/tmp/dcfinal001_r10r9r4_density_baseline.json"
    } else {
        "/tmp/dcfinal001_r10r9r4_evolution.json"
    };
    run_r10_evolution_with_horizon(
        output,
        "DC-FINAL-001-R10R9R4-DENSITY-PRESERVING-MUTATION-SUPPLY-SELECTION-REVERSAL-AND-FINAL-CLOSURE-001",
        R10R7_SELECTION_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        founder_multiplicity,
        bath_volume,
    );
}

/// R10R9R5 canonical lifecycle/evidence recovery.  The legacy R10R9R4
/// population path remains available as the historical comparison control;
/// this entry point opts into typed lifecycle retention and atomic rejection
/// handling without changing organism equations or constants.
pub fn run_r10r9r5_evolution() {
    run_r10_evolution_with_horizon(
        "/tmp/dcfinal001_r10r9r5_evolution.json",
        "DC-FINAL-001-R10R9R5-CANONICAL-LIFECYCLE-AND-EVOLUTION-EVIDENCE-RECOVERY-001",
        R10R7_SELECTION_STEPS,
        D096ExpressionPath::V4FiniteBudgetCentered,
        PopulationBoundaryMode::FixedConcentrationBoundary,
        R10R9R4_FOUNDER_MULTIPLICITY,
        R10R9R4_FOUNDER_MULTIPLICITY as f64,
    );
}

/// Bounded executable controls for the R5 lifecycle boundary.  These do not
/// run the population campaign.  They exercise the same R10 population step
/// used by the production entry point and prove that observer labels are not
/// inputs to physical transitions, while also checking the zero-funded
/// expression boundary.
pub fn run_r10r9r5_contract_tests() {
    let output = PathBuf::from("/tmp/dcfinal001_r10r9r5_contract_tests.json");
    let (template, plasticity, original_birth_mass, template_step) =
        r10_closure::r10_seed3_fission_state();
    let labels = ["unchanged", "disabled", "deliberately_altered"];
    let mut observer_runs = Vec::new();
    for label in labels {
        env::set_var("DCFINAL001_R10R9R5_OBSERVER_LABEL_PROBE", label);
        let allocation = AllocationParams::default();
        let mut probe_mesh = template.clone();
        probe_mesh.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
        let probe = Cohort {
            mesh: probe_mesh,
            plasticity: Some(plasticity.clone()),
            count: 1,
            generation: 0,
            birth_mass: original_birth_mass,
            id: 1,
        };
        let mut cohorts = vec![probe];
        let mut world = OpenMedium::new(Environment::Resource, 150.0);
        let mut ledger = CampaignLedger::default();
        let mut next_id = 2;
        let mut trajectory = vec![snapshot(&cohorts, &world, 0)];
        let mut prefix = None;
        let mut daughter_continuations = Vec::new();
        let accepted = r10_advance_phase(
            &mut cohorts,
            &mut world,
            Environment::Resource,
            0,
            false,
            9_001,
            &mut next_id,
            &mut ledger,
            &mut trajectory,
            &mut prefix,
            1,
            D096ExpressionPath::V4FiniteBudgetCentered,
            PopulationBoundaryMode::FixtureExteriorReference,
            FissionClockMode::HistoricalStepMinusOne,
            ReserveResolutionMode::PerStep,
            &mut daughter_continuations,
            true,
            DiagnosticBoundarySource::Scheduled,
            DiagnosticBoundarySource::Scheduled,
            true,
            true,
            GrowthPlacementMode::FrozenD088,
            None,
            None,
            false,
            None,
            None,
        );
        let fission_attempts = ledger
            .lifecycle_events
            .iter()
            .filter(|event| event["phase"] == "fission_attempt")
            .count();
        observer_runs.push(json!({
            "label_probe": label,
            "template_fission_step": template_step,
            "accepted": accepted,
            "physical_transition_digest": deterministic_state_digest(&json!({
                "cohorts": &cohorts,
                "ledger": &ledger,
                "world": &world,
            })),
            "fission_attempts": fission_attempts,
            "physical_fissions": ledger.physical_fissions,
            "daughter_continuations": daughter_continuations,
            "accepted_steps": ledger.accepted_steps,
        }));
    }
    env::remove_var("DCFINAL001_R10R9R5_OBSERVER_LABEL_PROBE");
    let observer_digest = observer_runs[0]["physical_transition_digest"].clone();
    let observer_pass = observer_runs
        .iter()
        .all(|row| row["physical_transition_digest"] == observer_digest);

    let allocation = AllocationParams::default();
    let mut depleted = template.clone();
    depleted.enable_finite_allocation_v4(AllocationGenotype::neutral(), &allocation);
    depleted
        .finite_allocation
        .as_mut()
        .expect("allocation")
        .catalysts = [0.4; 4];
    depleted.interior.a = 0.0;
    let catalyst_before = depleted.finite_allocation.expect("allocation").catalysts;
    let turnover = expression_step_activated_material_v4_turnover_only(
        &mut depleted,
        &allocation,
        MechParams::default().dt,
    )
    .expect("zero-funded turnover remains a valid transition");
    let catalyst_after = depleted.finite_allocation.expect("allocation").catalysts;
    let turnover_pass = turnover.synthesis.iter().all(|value| *value == 0.0)
        && turnover.turnover_waste > 0.0
        && catalyst_after
            .iter()
            .zip(catalyst_before)
            .all(|(after, before)| *after < before);

    let value = json!({
        "observer_label_independence": {
            "runs": observer_runs,
            "pass": observer_pass,
            "contract": "observer labels are audit-only and cannot control fission eligibility",
        },
        "zero_funded_expression_turnover": {
            "synthesis": turnover.synthesis,
            "turnover_waste": turnover.turnover_waste,
            "catalyst_before": catalyst_before,
            "catalyst_after": catalyst_after,
            "pass": turnover_pass,
        },
        "boundary_state_coherence": boundary_state_coherence_contract(),
    });
    eprintln!(
        "R5 boundary-state control: {}",
        serde_json::to_string(&value["boundary_state_coherence"]).unwrap()
    );
    assert!(observer_pass && turnover_pass && value["boundary_state_coherence"]["pass"] == true);
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

/// R10R3 Gates 2, 4, and 5: matched generation-1 production daughters under
/// current D096-v1, D096-off, and the observer-only activated-material
/// candidate. The candidate is not a production schema and mutation is off.
pub fn run_r10r3_budget_diagnostics() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r10r3_budget_diagnostics.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let (template, plasticity, original_birth_mass, template_step) =
        r10_closure::r10_seed3_fission_state();
    let mut handles = Vec::new();
    for environment in [Environment::Resource, Environment::Damage] {
        for expression_path in [
            D096ExpressionPath::V1Structural,
            D096ExpressionPath::Off,
            D096ExpressionPath::ActivatedMaterialCandidate,
        ] {
            let template = template.clone();
            let plasticity = plasticity.clone();
            handles.push(std::thread::spawn(move || {
                r10_campaign(
                    &template,
                    &plasticity,
                    original_birth_mass,
                    template_step,
                    &[environment],
                    false,
                    1,
                    R10R2_PHASE_STEPS,
                    expression_path,
                    FOUNDER_MULTIPLICITY,
                    FOUNDER_MULTIPLICITY as f64,
                    PopulationBoundaryMode::RateReinterpretation,
                    None,
                )
            }));
        }
    }
    let campaigns = handles
        .into_iter()
        .map(|handle| handle.join().expect("R10R3 budget diagnostic"))
        .collect::<Vec<_>>();
    let value = json!({
        "directive": "DC-FINAL-001-R10R3-D096-ACTIVATED-MATERIAL-EXPRESSION-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001",
        "gates": [2, 4, 5],
        "observer_only_candidate": true,
        "mutation_enabled": false,
        "phase_steps": R10R2_PHASE_STEPS,
        "founder_multiplicity": FOUNDER_MULTIPLICITY,
        "campaigns": campaigns,
        "candidate_contract": {
            "structural_draw": 0,
            "a_precursor_per_catalyst": 1.0,
            "additional_activation_overhead": AllocationParams::default().activation_cost,
            "maintenance_rate": AllocationParams::default().maintenance_rate,
            "turnover_rate": AllocationParams::default().turnover_rate,
            "new_parameters": 0,
            "production_schema_created": false,
        },
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r4_evolution.json");
    let args: Vec<String> = env::args().collect();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let (template, original_birth_mass, template_step) = lawful_parent_template();
    let parity = compression_parity(&template, original_birth_mass);
    assert_eq!(parity["pass"], true);
    let newborn_attribution = newborn_first_step_attribution(&template);
    let mutated_heredity = mutated_descendant_heredity_control(&template);
    let v4_counterpart = v4_counterpart(&template);
    let ownership_matrix = contract_ownership_matrix();
    let historical_d096_parity = non_v4_d096_parity(&template);
    let closing_edge_trace = v4_closing_edge_trace(&template);
    let exact_r2_contract = format!("{:?}", template.contract_version);
    let exact_r2_gate6_pass =
        newborn_attribution["daughter_a"]["expression_on"]["contact_accepted"] == true
            && newborn_attribution["daughter_b"]["expression_on"]["contact_accepted"] == true;
    let v4_gate6_pass = ["daughter_a_lifecycle", "daughter_b_lifecycle"]
        .into_iter()
        .all(|key| {
            v4_counterpart[key]["runtime_invalidation"].is_null()
                && v4_counterpart[key]["terminal_lifecycle_invariants_hold"] == true
                && v4_counterpart[key]["terminal_physical_runtime_valid"] == true
                && v4_counterpart[key]["terminal_polygon_simple"] == true
                && (v4_counterpart[key]["completed_steps"] == 3_000
                    || (v4_counterpart[key]["physical_fission"]["partition_ok"] == true
                        && v4_counterpart[key]["physical_fission"]["parent_simple"] == true
                        && v4_counterpart[key]["physical_fission"]["daughter_a_simple"] == true
                        && v4_counterpart[key]["physical_fission"]["daughter_b_simple"] == true))
        });
    let v4_reproduction = r4_reproduction_campaign();
    let v4_reproduction_pass = v4_reproduction["pass"] == true;
    let value = json!({
        "directive": DIRECTIVE,
        "protocol": {
            "mutation_probability": AllocationParams::default().mutation_probability,
            "mutation_sigma": AllocationParams::default().mutation_sigma,
            "mutation_semantics": "one deterministic blind draw per daughter at geometry-valid physical fission",
            "minimum_opportunities_for_95_percent_at_least_one": 299,
            "founder_multiplicity": FOUNDER_MULTIPLICITY,
            "opportunities_per_initial_campaign": 2 * FOUNDER_MULTIPLICITY,
            "replicates": REPLICATES,
            "phase_steps": PHASE_STEPS,
            "switch_schedule": [PHASE_STEPS, 2 * PHASE_STEPS],
            "open_medium_volume": FOUNDER_MULTIPLICITY,
            "inflow_schedule_source": "frozen D-096 H/B concentration values multiplied by dt and preregistered vessel volume",
            "component_3_boundary": "reserve-only growth endpoint dormant under preserved reserve-OFF R1 physiology",
        },
        "template": {
            "sealed_wp1_arm": "seed_1_rotate_0.3",
            "fission_step": template_step,
            "simple": polygon_simple(&template.vertices),
            "vertices": template.n(),
            "mass": template.total_structural_mass(),
            "original_birth_mass": original_birth_mass,
            "contract_version": exact_r2_contract,
        },
        "exchangeability_parity": parity,
        "contract_ownership_matrix": ownership_matrix,
        "non_v4_d096_parity": historical_d096_parity,
        "newborn_first_step_attribution": newborn_attribution,
        "v4_counterpart": v4_counterpart,
        "v4_closing_edge_trace": closing_edge_trace,
        "mutated_descendant_heredity_control": mutated_heredity,
        "gate6_exact_r2_newborn_continuity": if exact_r2_gate6_pass { "PASS" } else { "FAIL" },
        "gate6_v4_newborn_continuity": if v4_gate6_pass { "PASS" } else { "FAIL" },
        "v4_reproduction_campaign": v4_reproduction,
        "evolution_execution": if v4_reproduction_pass { "PENDING_AFTER_GATE9" } else { "NOT_RUN_GATE9_STOP" },
        "classification": if !v4_gate6_pass { "V4_TOPOLOGY_CREATED_EDGE_MATURATION_ARCHITECTURE_INVALID" } else if !v4_reproduction_pass { "V4_GEOMETRY_VALID_REPRODUCTION_NOT_ESTABLISHED" } else { "V4_CONTRACT_TOPOLOGY_COHERENCE_REPAIRED" },
        "causal_boundary": "Evolution executes only after canonical production-V4 reproduction qualifies.",
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
