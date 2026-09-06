use chemistry_core::material_mesh::MeshContractVersion;
use chemistry_core::environmental_assimilation;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_topology::{find_local_pinch, local_rebond_range, TopologyLedger};
use chemistry_core::mesh_growth::{growth_step, GrowthLedger, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_population::{MeshIndividual, MeshPopulation};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionLedger, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_transport::transport_step;
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    ContractilityParamsV1, FiniteWorldResourceV1, FiniteWorldV1,
    MovingMembraneFiniteFluxV1, SharedFiniteExtracellularMediumV1, SpatialMaterialFieldV1,
    StickSlipTractionParamsV1,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod polarity;
use polarity::PolarityState;

const SCHEMA: &str = "digital_cell_m2_checkpointable_lifeform_runtime_v3_developmental_polarity";
const RESOURCE_RADIUS: f64 = 1.5;
// These are the already accepted CLOSURE-003-R1/CLOSURE-004 material units,
// not a runtime tuning sweep.  The earlier three-unit smoke fixture could not
// support even one accepted reproductive unit after separated contact.
const RESOURCE_MASS: f64 = 1021.692995326332;
const RESOURCE_BOUNDARY: f64 = 2.063914918930895;
const DEVELOPMENT_MAX_STEPS: usize = 12_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeSnapshot {
    schema: String,
    step: u64,
    seed: u64,
    population: MeshPopulation,
    world: FiniteWorldV1,
    /// Opt-in Route-B world. `None` preserves the historical FiniteWorldV1
    /// runtime path and its checkpoint schema semantics.
    #[serde(default)]
    spatial_field: Option<SpatialMaterialFieldV1>,
    /// Opt-in R11 finite shared extracellular medium. `None` preserves the
    /// historical FiniteWorldV1 and Route-B checkpoint semantics.
    #[serde(default)]
    shared_medium: Option<SharedFiniteExtracellularMediumV1>,
    /// Opt-in R15 moving-membrane finite interface. `None` preserves every
    /// historical runtime composition and checkpoint interpretation.
    #[serde(default)]
    moving_membrane: Option<MovingMembraneFiniteFluxV1>,
    /// R17 assay-only whole-membrane finite feed. This is world-side state;
    /// it is never read by organism biology and is absent from historical
    /// checkpoints.
    #[serde(default)]
    matched_whole_membrane: Option<MatchedWholeMembraneFiniteFeed>,
    /// Opt-in D-091 reserve composition; absent preserves reserve-off runtime.
    #[serde(default)]
    reserve_parameters: Option<ReserveParams>,
    /// Opt-in finite environmental assimilation substrate. Absent/false
    /// preserves all historical runtime compositions.
    #[serde(default)]
    assimilation_enabled: bool,
    /// Opt-in incorporation of newly processed environmental A through the
    /// existing structural-build law. False preserves R4 semantics.
    #[serde(default)]
    anabolic_incorporation_enabled: bool,
    #[serde(default = "default_true")]
    spatial_field_transfer_enabled: bool,
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
    #[serde(default)]
    cumulative_assimilation_n_processed: f64,
    #[serde(default)]
    cumulative_assimilation_f_processed: f64,
    #[serde(default)]
    cumulative_assimilation_a_produced: f64,
    #[serde(default)]
    cumulative_assimilation_m_grown: f64,
    #[serde(default)]
    cumulative_assimilation_m_incorporated: f64,
    cumulative_n_world_loss: f64,
    cumulative_f_world_loss: f64,
    cumulative_fissions: usize,
    cumulative_motor_a_spent: f64,
    cumulative_slipping_contacts: usize,
    #[serde(default)]
    cumulative_path: f64,
    #[serde(default)]
    cumulative_contacts: usize,
    #[serde(default)]
    first_contact_step: Option<u64>,
    #[serde(default)]
    first_transfer_step: Option<u64>,
    #[serde(default)]
    first_fission_step: Option<u64>,
    #[serde(default)]
    fission_observations: Vec<FissionObservation>,
    #[serde(default)]
    lineage_n_delivered: BTreeMap<u64, f64>,
    #[serde(default)]
    lineage_f_delivered: BTreeMap<u64, f64>,
    #[serde(default)]
    developmental_bootstrap_steps: usize,
    #[serde(default)]
    developmental_initial_polarity_amplitude: f64,
    #[serde(default)]
    developmental_initial_topology: usize,
    #[serde(default)]
    developmental_fission_boundary_reached: bool,
    /// Ecological clock starts after an already-available unforced fission.
    #[serde(default)]
    ecology_started_after_unforced_fission: bool,
    /// Developmental fissions are provenance, not ecological events.
    #[serde(default)]
    pre_ecology_fission_events: usize,
    motor_steps: u64,
    motor_failures: u64,
    #[serde(default)]
    polarity_states: Vec<Option<PolarityState>>,
    #[serde(default)]
    previous_centroids: Vec<[f64; 2]>,
    /// Opt-in observer-only post-transfer flux accounting. `None` preserves
    /// all historical checkpoint semantics and runtime behavior.
    #[serde(default)]
    flux_audit: Option<FluxAuditState>,
    /// Opt-in observer-only fission-readiness trace. It stores only cloned
    /// prerequisite observations and never participates in the authoritative
    /// fission decision.
    #[serde(default)]
    fission_readiness_audit: Option<FissionReadinessAudit>,
    scientific_boundary: ScientificBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScientificBoundary {
    finite_world_exchange: String,
    frozen_reactions: String,
    frozen_growth: String,
    physical_fission: String,
    active_motility: String,
    autonomous_resource_acquisition: String,
    resource_causal_reproduction: String,
}

impl Default for ScientificBoundary {
    fn default() -> Self {
        Self {
            finite_world_exchange: "FiniteWorldV1".to_string(),
            frozen_reactions: "ReactionParams::conservative_v3 / reserve OFF".to_string(),
            frozen_growth: "MeshPopulation::step / existing GrowthParams".to_string(),
            physical_fission: "mesh_fission::try_local_fission via MeshPopulation::step"
                .to_string(),
            active_motility:
                "ENTRY-019..027 native inherited-polarity motor with existing A-funded stick-slip"
                    .to_string(),
            autonomous_resource_acquisition: "NOT_ESTABLISHED".to_string(),
            resource_causal_reproduction: "NOT_ESTABLISHED".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct RuntimeReport {
    schema: &'static str,
    step: u64,
    seed: u64,
    living_count: usize,
    total_individuals: usize,
    maximum_generation: u32,
    fission_events: usize,
    world_n_mass_remaining: f64,
    world_f_mass_remaining: f64,
    spatial_field_n_mass_remaining: f64,
    spatial_field_f_mass_remaining: f64,
    shared_medium_n_mass_remaining: f64,
    shared_medium_f_mass_remaining: f64,
    moving_membrane_n_mass_remaining: f64,
    moving_membrane_f_mass_remaining: f64,
    matched_whole_membrane_n_mass_remaining: f64,
    matched_whole_membrane_f_mass_remaining: f64,
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
    cumulative_assimilation_n_processed: f64,
    cumulative_assimilation_f_processed: f64,
    cumulative_assimilation_a_produced: f64,
    cumulative_assimilation_m_grown: f64,
    cumulative_assimilation_m_incorporated: f64,
    world_n_conservation_error: f64,
    world_f_conservation_error: f64,
    motor_steps: u64,
    motor_failures: u64,
    cumulative_motor_a_spent: f64,
    cumulative_slipping_contacts: usize,
    cumulative_path: f64,
    cumulative_contacts: usize,
    first_contact_step: Option<u64>,
    first_transfer_step: Option<u64>,
    first_fission_step: Option<u64>,
    first_fission_before_first_transfer: Option<bool>,
    fission_observations: Vec<FissionObservation>,
    resource_transfer_enabled: bool,
    resource_mode: String,
    reserve_enabled: bool,
    developmental_bootstrap_steps: usize,
    developmental_initial_topology: usize,
    developmental_initial_polarity_amplitude: f64,
    developmental_fission_boundary_reached: bool,
    ecology_started_after_unforced_fission: bool,
    pre_ecology_fission_events: usize,
    current_max_polarity_amplitude: f64,
    terminal_observer_death_reasons: Vec<Option<&'static str>>,
    active_motility: String,
    autonomous_resource_acquisition: &'static str,
    resource_causal_reproduction: &'static str,
    checkpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    flux_audit: Option<FluxAuditState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fission_readiness_audit: Option<FissionReadinessAudit>,
}

#[derive(Debug)]
struct Config {
    steps: u64,
    seed: u64,
    checkpoint: PathBuf,
    report: PathBuf,
    resume: Option<PathBuf>,
    transfer_disabled: bool,
    routeb_spatial_field: bool,
    shared_extracellular_medium: bool,
    shared_medium_from_birth: bool,
    moving_membrane_flux: bool,
    r17_early_whole_membrane: bool,
    r17_delayed_whole_membrane: bool,
    r18_fission_audit: bool,
    routec_reserve_growth: bool,
    assimilation_material_flow: bool,
    anabolic_incorporation: bool,
    post_fission_ecology: bool,
    flux_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FissionObservation {
    step: u64,
    parent_lineage_id: u64,
    parent_generation: u32,
    parent_n_delivered: f64,
    parent_f_delivered: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FissionReadinessRow {
    step: u64,
    phase: String,
    total_structural_mass: f64,
    birth_mass: f64,
    mass_over_birth_mass: f64,
    mass_gate_reached: bool,
    vertex_count: usize,
    can_advance_physics: bool,
    area: f64,
    perimeter: f64,
    shape_factor: f64,
    max_edge_strain: f64,
    mean_edge_strain: f64,
    ruptured_edge_count: usize,
    concave_vertex_count: usize,
    local_rebond_range: f64,
    best_nonadjacent_distance: Option<f64>,
    best_distance_over_range: Option<f64>,
    pinch_candidate_exists: bool,
    pinch_i: Option<usize>,
    pinch_j: Option<usize>,
    pinch_distance: Option<f64>,
    pinch_stress_condition: bool,
    pinch_proximity_condition: bool,
    absolute_a_mass: f64,
    cross_bond_mass_needed: Option<f64>,
    a_over_cross_bond_need: Option<f64>,
    cross_bond_a_sufficient: bool,
    shadow_try_local_fission: String,
    mass_gate_attempt_tick: bool,
    reason_not_ready: String,
    topology_tension_ruptures: usize,
    topology_local_rebonds: usize,
    topology_cross_bonds: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FissionReadinessAudit {
    rows: Vec<FissionReadinessRow>,
    official_attempt_ticks: Vec<FissionReadinessRow>,
    passive_mechanics_shadow: Vec<FissionReadinessRow>,
}

fn best_nonadjacent_distance(mesh: &chemistry_core::material_mesh::MaterialMesh) -> Option<f64> {
    let n = mesh.n();
    if n < 8 {
        return None;
    }
    let min_sep = (n / 4).max(3);
    let mut best = None;
    for i in 0..n {
        for dj in min_sep..=(n - min_sep) {
            let j = (i + dj) % n;
            if j <= i {
                continue;
            }
            let ring_sep = (j - i).min(n - (j - i));
            if ring_sep < min_sep {
                continue;
            }
            let a = mesh.vertices[i];
            let b = mesh.vertices[j];
            let distance = (b[0] - a[0]).hypot(b[1] - a[1]);
            best = Some(best.map_or(distance, |current: f64| current.min(distance)));
        }
    }
    best
}

fn concave_vertex_count(mesh: &chemistry_core::material_mesh::MaterialMesh) -> usize {
    let n = mesh.n();
    if n < 3 {
        return 0;
    }
    let orientation = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    (0..n)
        .filter(|&i| {
            let previous = mesh.vertices[(i + n - 1) % n];
            let current = mesh.vertices[i];
            let next = mesh.vertices[(i + 1) % n];
            let ab = [current[0] - previous[0], current[1] - previous[1]];
            let bc = [next[0] - current[0], next[1] - current[1]];
            orientation * (ab[0] * bc[1] - ab[1] * bc[0]) < -1e-12
        })
        .count()
}

fn fission_readiness_row(
    mesh: &chemistry_core::material_mesh::MaterialMesh,
    birth_mass: f64,
    fission: &FissionParams,
    step: u64,
    phase: &str,
    attempt_tick: bool,
    topology: &TopologyLedger,
) -> FissionReadinessRow {
    let total = mesh.total_structural_mass();
    let area = mesh.area().abs();
    let perimeter = mesh.perimeter();
    let range = local_rebond_range(mesh, &fission.topo);
    let candidate = find_local_pinch(mesh, &fission.topo);
    let best_distance = best_nonadjacent_distance(mesh);
    let best_over_range = best_distance.map(|distance| distance / range.max(1e-12));
    let (pinch_i, pinch_j, pinch_distance, stress, proximity) = if let Some((i, j)) = candidate {
        let a = mesh.vertices[i];
        let b = mesh.vertices[j];
        let distance = (b[0] - a[0]).hypot(b[1] - a[1]);
        let strain_i = mesh.strain(i).max(mesh.strain((i + mesh.n() - 1) % mesh.n()));
        let strain_j = mesh.strain(j).max(mesh.strain((j + mesh.n() - 1) % mesh.n()));
        let stressed = strain_i > 0.15
            || strain_j > 0.15
            || mesh.edges[i].ruptured
            || mesh.edges[(j + mesh.n() - 1) % mesh.n()].ruptured
            || distance < range * 0.55;
        (Some(i), Some(j), Some(distance), stressed, distance <= range)
    } else {
        (None, None, None, false, false)
    };
    let need = pinch_distance.map(|distance| mesh.rho_s * distance);
    let absolute_a = mesh.interior.a.max(0.0) * area;
    let conservative = mesh.uses_observer_only_death();
    let required = need.map(|value| if conservative { value } else { value * 0.25 });
    let ratio = required.map(|value| absolute_a / value.max(1e-12));
    let sufficient = required.map(|value| absolute_a >= value).unwrap_or(false);
    let shadow = if try_local_fission(&mesh.clone(), fission).is_some() {
        "SUCCESS"
    } else {
        "FAIL"
    };
    let gate = total >= 1.35 * birth_mass.max(1e-9);
    let reason = if !gate {
        "MASS_NOT_ELIGIBLE"
    } else if !mesh.can_advance_physics() {
        "PHYSICS_INACTIVE"
    } else if mesh.n() < fission.min_vertices {
        "VERTEX_REQUIREMENT"
    } else if candidate.is_none() {
        match best_distance {
            Some(distance) if distance > range => "PINCH_OUT_OF_RANGE",
            Some(_) => "PINCH_NOT_STRESSED",
            None => "NO_PINCH",
        }
    } else if !stress {
        "PINCH_NOT_STRESSED"
    } else if !proximity {
        "PINCH_OUT_OF_RANGE"
    } else if !sufficient {
        "CROSS_BOND_A_INSUFFICIENT"
    } else if shadow == "SUCCESS" {
        "FISSION_READY"
    } else {
        "UNRESOLVED"
    };
    let strains: Vec<f64> = (0..mesh.n()).map(|i| mesh.strain(i)).collect();
    FissionReadinessRow {
        step,
        phase: phase.to_string(),
        total_structural_mass: total,
        birth_mass,
        mass_over_birth_mass: total / birth_mass.max(1e-9),
        mass_gate_reached: gate,
        vertex_count: mesh.n(),
        can_advance_physics: mesh.can_advance_physics(),
        area,
        perimeter,
        shape_factor: if perimeter > 0.0 {
            4.0 * std::f64::consts::PI * area / (perimeter * perimeter)
        } else {
            0.0
        },
        max_edge_strain: strains.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        mean_edge_strain: if strains.is_empty() {
            0.0
        } else {
            strains.iter().sum::<f64>() / strains.len() as f64
        },
        ruptured_edge_count: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        concave_vertex_count: concave_vertex_count(mesh),
        local_rebond_range: range,
        best_nonadjacent_distance: best_distance,
        best_distance_over_range: best_over_range,
        pinch_candidate_exists: candidate.is_some(),
        pinch_i,
        pinch_j,
        pinch_distance,
        pinch_stress_condition: stress,
        pinch_proximity_condition: proximity,
        absolute_a_mass: absolute_a,
        cross_bond_mass_needed: required,
        a_over_cross_bond_need: ratio,
        cross_bond_a_sufficient: sufficient,
        shadow_try_local_fission: shadow.to_string(),
        mass_gate_attempt_tick: attempt_tick,
        reason_not_ready: reason.to_string(),
        topology_tension_ruptures: topology.tension_ruptures,
        topology_local_rebonds: topology.local_rebonds,
        topology_cross_bonds: topology.cross_bonds,
    }
}

#[derive(Debug, Clone, Default)]
struct RuntimeDelivery {
    organism_index: usize,
    exposed_edges: usize,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
}

/// R17 uses the already-qualified whole-membrane transport law as an
/// assay-only counterfactual. The organism receives the unchanged
/// `transport_step` inward ledger, while finite N/F are debited from this
/// world-side inventory. No organism state or chemistry law is added.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchedWholeMembraneFiniteFeed {
    schema: String,
    initial_n_mass: f64,
    initial_f_mass: f64,
    n_mass: f64,
    f_mass: f64,
    boundary_n: f64,
    boundary_f: f64,
    step: u64,
    transfer_enabled: bool,
    ledger_n_taken: f64,
    ledger_f_taken: f64,
}

impl MatchedWholeMembraneFiniteFeed {
    fn new(n_mass: f64, f_mass: f64) -> Self {
        Self {
            schema: "digital_cell_r17_matched_whole_membrane_finite_feed_v1".to_string(),
            initial_n_mass: n_mass.max(0.0),
            initial_f_mass: f_mass.max(0.0),
            n_mass: n_mass.max(0.0),
            f_mass: f_mass.max(0.0),
            boundary_n: RESOURCE_BOUNDARY,
            boundary_f: RESOURCE_BOUNDARY,
            step: 0,
            transfer_enabled: true,
            ledger_n_taken: 0.0,
            ledger_f_taken: 0.0,
        }
    }

    fn total_n_mass(&self) -> f64 {
        self.n_mass
    }

    fn total_f_mass(&self) -> f64 {
        self.f_mass
    }

    fn exchange(
        &mut self,
        meshes: &mut [chemistry_core::material_mesh::MaterialMesh],
        transport: &TransportParams,
        dt: f64,
    ) -> Vec<RuntimeDelivery> {
        #[derive(Clone, Copy)]
        struct Request {
            organism_index: usize,
            requested_n: f64,
            requested_f: f64,
        }

        let mut deliveries = vec![RuntimeDelivery::default(); meshes.len()];
        let mut requests = Vec::with_capacity(meshes.len());
        for (organism_index, mesh) in meshes.iter_mut().enumerate() {
            let exterior = mesh.exterior;
            mesh.exterior.n = 0.0;
            mesh.exterior.f = 0.0;
            let _nonfeeding = transport_step(mesh, transport, dt);
            mesh.exterior = exterior;

            if !mesh.can_advance_physics() || dt <= 0.0 {
                continue;
            }
            let mut preview = mesh.clone();
            preview.exterior.n = self.boundary_n;
            preview.exterior.f = self.boundary_f;
            let requested = transport_step(&mut preview, transport, dt);
            requests.push(Request {
                organism_index,
                requested_n: requested.n_in.max(0.0),
                requested_f: requested.f_in.max(0.0),
            });
            deliveries[organism_index].exposed_edges = mesh.n();
        }

        let total_n: f64 = requests.iter().map(|request| request.requested_n).sum();
        let total_f: f64 = requests.iter().map(|request| request.requested_f).sum();
        let n_scale = if total_n > 0.0 {
            (self.n_mass / total_n).min(1.0)
        } else {
            1.0
        };
        let f_scale = if total_f > 0.0 {
            (self.f_mass / total_f).min(1.0)
        } else {
            1.0
        };
        let scale = if self.transfer_enabled {
            n_scale.min(f_scale).max(0.0)
        } else {
            0.0
        };

        for request in requests {
            let n = request.requested_n * scale;
            let f = request.requested_f * scale;
            if let Some(mesh) = meshes.get_mut(request.organism_index) {
                let area = mesh.area();
                if area.is_finite() && area > 0.0 {
                    mesh.interior.n += n / area;
                    mesh.interior.f += f / area;
                }
            }
            let delivery = &mut deliveries[request.organism_index];
            delivery.n_delivered += n;
            delivery.f_delivered += f;
            delivery.n_world_loss += n;
            delivery.f_world_loss += f;
        }

        let delivered_n: f64 = deliveries.iter().map(|delivery| delivery.n_delivered).sum();
        let delivered_f: f64 = deliveries.iter().map(|delivery| delivery.f_delivered).sum();
        self.n_mass = (self.n_mass - delivered_n).max(0.0);
        self.f_mass = (self.f_mass - delivered_f).max(0.0);
        self.ledger_n_taken += delivered_n;
        self.ledger_f_taken += delivered_f;
        self.step = self.step.saturating_add(1);
        deliveries
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FluxAuditCheckpoint {
    step: u64,
    steps_since_first_transfer: Option<u64>,
    checkpoint_reason: String,
    environmental_n_remaining: f64,
    environmental_f_remaining: f64,
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
    interior_n: f64,
    interior_f: f64,
    cumulative_reaction_n_consumed: f64,
    cumulative_reaction_f_consumed: f64,
    cumulative_a_produced: f64,
    cumulative_w_produced: f64,
    cumulative_reaction_w_produced: f64,
    cumulative_growth_w_produced: f64,
    a_pool: f64,
    cumulative_maintenance_a: f64,
    cumulative_active_work_a: f64,
    cumulative_growth_a: f64,
    young_structural_mass: f64,
    mature_structural_mass: f64,
    total_structural_mass: f64,
    birth_mass: f64,
    mass_over_birth_mass: f64,
    fission_gate_mass: f64,
    fission_gate_reached: bool,
    pinch_available: String,
    cross_bond_a_available: String,
    physical_fission: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FluxAuditState {
    #[serde(default)]
    initial_birth_mass: f64,
    #[serde(default)]
    initial_interior_n: f64,
    #[serde(default)]
    initial_interior_f: f64,
    #[serde(default)]
    initial_a_pool: f64,
    #[serde(default)]
    initial_young_structural_mass: f64,
    #[serde(default)]
    initial_mature_structural_mass: f64,
    #[serde(default)]
    initial_total_structural_mass: f64,
    first_transfer_step: Option<u64>,
    first_contact_step: Option<u64>,
    cumulative_reaction_n_consumed: f64,
    cumulative_reaction_f_consumed: f64,
    cumulative_a_produced: f64,
    cumulative_w_produced: f64,
    cumulative_reaction_w_produced: f64,
    cumulative_growth_w_produced: f64,
    cumulative_maintenance_a: f64,
    cumulative_active_work_a: f64,
    cumulative_growth_a: f64,
    cumulative_growth_material: f64,
    last_motor_a_spent: f64,
    checkpoints: Vec<FluxAuditCheckpoint>,
    recorded_delivery_thresholds: [bool; 5],
    unresolved_fields: Vec<String>,
}

impl FluxAuditState {
    fn new(snapshot: &RuntimeSnapshot) -> Self {
        let (initial_n, initial_f, initial_a, initial_young, initial_total) =
            audit_physical_totals(snapshot);
        let initial_birth_mass = snapshot
            .population
            .individuals
            .iter()
            .filter(|individual| individual.mesh.alive)
            .map(|individual| individual.birth_mass)
            .sum();
        Self {
            initial_birth_mass,
            initial_interior_n: initial_n,
            initial_interior_f: initial_f,
            initial_a_pool: initial_a,
            initial_young_structural_mass: initial_young,
            initial_mature_structural_mass: (initial_total - initial_young).max(0.0),
            initial_total_structural_mass: initial_total,
            first_transfer_step: snapshot.first_transfer_step,
            first_contact_step: snapshot.first_contact_step,
            cumulative_reaction_n_consumed: 0.0,
            cumulative_reaction_f_consumed: 0.0,
            cumulative_a_produced: 0.0,
            cumulative_w_produced: 0.0,
            cumulative_reaction_w_produced: 0.0,
            cumulative_growth_w_produced: 0.0,
            cumulative_maintenance_a: 0.0,
            cumulative_active_work_a: 0.0,
            cumulative_growth_a: 0.0,
            cumulative_growth_material: 0.0,
            last_motor_a_spent: snapshot.cumulative_motor_a_spent,
            checkpoints: Vec::new(),
            recorded_delivery_thresholds: [false; 5],
            unresolved_fields: vec![
                "pinch_available=UNRESOLVED_BY_CURRENT_LEDGER".to_string(),
                "cross_bond_a_available=UNRESOLVED_BY_CURRENT_LEDGER".to_string(),
            ],
        }
    }
}

fn audit_environmental_remaining(snapshot: &RuntimeSnapshot) -> (f64, f64) {
    if let Some(medium) = snapshot.matched_whole_membrane.as_ref() {
        (medium.total_n_mass(), medium.total_f_mass())
    } else if let Some(medium) = snapshot.moving_membrane.as_ref() {
        (medium.total_n_mass(), medium.total_f_mass())
    } else if let Some(medium) = snapshot.shared_medium.as_ref() {
        (medium.total_n_mass(), medium.total_f_mass())
    } else if let Some(field) = snapshot.spatial_field.as_ref() {
        (field.total_n_mass(), field.total_f_mass())
    } else {
        (snapshot.world.total_n_mass(), snapshot.world.total_f_mass())
    }
}

fn audit_physical_totals(snapshot: &RuntimeSnapshot) -> (f64, f64, f64, f64, f64) {
    snapshot
        .population
        .individuals
        .iter()
        .filter(|individual| individual.mesh.alive)
        .fold((0.0, 0.0, 0.0, 0.0, 0.0), |totals, individual| {
            (
                totals.0 + individual.mesh.interior.n.max(0.0) * individual.mesh.area(),
                totals.1 + individual.mesh.interior.f.max(0.0) * individual.mesh.area(),
                totals.2 + individual.mesh.interior.a.max(0.0) * individual.mesh.area(),
                totals.3 + individual.mesh.total_young_structural_mass(),
                totals.4 + individual.mesh.total_structural_mass(),
            )
        })
}

fn audit_should_checkpoint(audit: &FluxAuditState, snapshot: &RuntimeSnapshot) -> Option<String> {
    let Some(transfer) = audit.first_transfer_step else {
        if snapshot.step == 1 || snapshot.step % 250 == 0 {
            return Some("no_transfer_control_checkpoint".to_string());
        }
        return None;
    };
    let since = snapshot.step.saturating_sub(transfer);
    if matches!(since, 0 | 1 | 25 | 50 | 100 | 250 | 500) {
        return Some(if since == 0 {
            "first_transfer".to_string()
        } else {
            format!("post_transfer_{since}")
        });
    }
    if snapshot.step % 250 == 0 {
        return Some("periodic_250_step".to_string());
    }
    None
}

fn update_flux_audit(
    snapshot: &mut RuntimeSnapshot,
    reaction_n: f64,
    reaction_f: f64,
    reaction_a: f64,
    reaction_w: f64,
    maintenance_a: f64,
    growth_a: f64,
    growth_material: f64,
    growth_w: f64,
) {
    let Some(mut audit) = snapshot.flux_audit.take() else {
        return;
    };
    audit.first_contact_step = snapshot.first_contact_step;
    audit.first_transfer_step = snapshot.first_transfer_step;
    audit.cumulative_reaction_n_consumed += reaction_n;
    audit.cumulative_reaction_f_consumed += reaction_f;
    audit.cumulative_a_produced += reaction_a;
    audit.cumulative_reaction_w_produced += reaction_w;
    audit.cumulative_growth_w_produced += growth_w;
    audit.cumulative_w_produced += reaction_w + growth_w;
    audit.cumulative_maintenance_a += maintenance_a;
    audit.cumulative_growth_a += growth_a;
    audit.cumulative_growth_material += growth_material;
    audit.cumulative_active_work_a +=
        (snapshot.cumulative_motor_a_spent - audit.last_motor_a_spent).max(0.0);
    audit.last_motor_a_spent = snapshot.cumulative_motor_a_spent;

    if let Some(reason) = audit_should_checkpoint(&audit, snapshot) {
        let (interior_n, interior_f, a_pool, young_mass, total_mass) =
            audit_physical_totals(snapshot);
        let (environmental_n_remaining, environmental_f_remaining) =
            audit_environmental_remaining(snapshot);
        let birth_mass: f64 = snapshot
            .population
            .individuals
            .iter()
            .filter(|individual| individual.mesh.alive)
            .map(|individual| individual.birth_mass)
            .sum();
        let fission_gate_mass = 1.35 * birth_mass.max(1e-9);
        let steps_since_first_transfer = audit
            .first_transfer_step
            .map(|transfer| snapshot.step.saturating_sub(transfer));
        audit.checkpoints.push(FluxAuditCheckpoint {
            step: snapshot.step,
            steps_since_first_transfer,
            checkpoint_reason: reason,
            environmental_n_remaining,
            environmental_f_remaining,
            cumulative_n_delivered: snapshot.cumulative_n_delivered,
            cumulative_f_delivered: snapshot.cumulative_f_delivered,
            interior_n,
            interior_f,
            cumulative_reaction_n_consumed: audit.cumulative_reaction_n_consumed,
            cumulative_reaction_f_consumed: audit.cumulative_reaction_f_consumed,
            cumulative_a_produced: audit.cumulative_a_produced,
            cumulative_w_produced: audit.cumulative_w_produced,
            cumulative_reaction_w_produced: audit.cumulative_reaction_w_produced,
            cumulative_growth_w_produced: audit.cumulative_growth_w_produced,
            a_pool,
            cumulative_maintenance_a: audit.cumulative_maintenance_a,
            cumulative_active_work_a: audit.cumulative_active_work_a,
            cumulative_growth_a: audit.cumulative_growth_a,
            young_structural_mass: young_mass,
            mature_structural_mass: (total_mass - young_mass).max(0.0),
            total_structural_mass: total_mass,
            birth_mass,
            mass_over_birth_mass: total_mass / birth_mass.max(1e-9),
            fission_gate_mass,
            fission_gate_reached: total_mass + 1e-12 >= fission_gate_mass,
            pinch_available: "UNRESOLVED_BY_CURRENT_LEDGER".to_string(),
            cross_bond_a_available: "UNRESOLVED_BY_CURRENT_LEDGER".to_string(),
            physical_fission: snapshot.first_fission_step == Some(snapshot.step),
        });
    }
    snapshot.flux_audit = Some(audit);
}

fn default_true() -> bool {
    true
}

fn usage() -> ! {
    eprintln!(
        "usage: digital-protocell-m2-runtime [--steps N] [--seed N] \\\n          [--checkpoint PATH] [--report PATH] [--resume PATH] \\\n          [--transfer-disabled] [--routeb-spatial-field] [--moving-membrane-flux] [--routec-reserve-growth] [--assimilation-material-flow] [--assimilation-anabolic-incorporation]"
    );
    std::process::exit(2);
}

fn parse_config() -> Config {
    let mut steps = 100_u64;
    let mut seed = 1_u64;
    let mut checkpoint = PathBuf::from("m2-lifeform-runtime.snapshot.json");
    let mut report = PathBuf::from("m2-lifeform-runtime.report.json");
    let mut resume = None;
    let mut transfer_disabled = false;
    let mut routeb_spatial_field = false;
    let mut shared_extracellular_medium = false;
    let mut shared_medium_from_birth = false;
    let mut moving_membrane_flux = false;
    let mut r17_early_whole_membrane = false;
    let mut r17_delayed_whole_membrane = false;
    let mut routec_reserve_growth = false;
    let mut assimilation_material_flow = false;
    let mut anabolic_incorporation = false;
    let mut post_fission_ecology = false;
    let mut flux_audit = false;
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let value = |i: &mut usize| -> String {
            *i += 1;
            args.get(*i).cloned().unwrap_or_else(|| usage())
        };
        match args[i].as_str() {
            "--steps" => steps = value(&mut i).parse().unwrap_or_else(|_| usage()),
            "--seed" => seed = value(&mut i).parse().unwrap_or_else(|_| usage()),
            "--checkpoint" => checkpoint = PathBuf::from(value(&mut i)),
            "--report" => report = PathBuf::from(value(&mut i)),
            "--resume" => resume = Some(PathBuf::from(value(&mut i))),
            "--transfer-disabled" => transfer_disabled = true,
            "--routeb-spatial-field" => routeb_spatial_field = true,
            "--shared-extracellular-medium" => shared_extracellular_medium = true,
            "--shared-medium-from-birth" => shared_medium_from_birth = true,
            "--moving-membrane-flux" => moving_membrane_flux = true,
            "--r17-early-whole-membrane" => r17_early_whole_membrane = true,
            "--r17-delayed-whole-membrane" => r17_delayed_whole_membrane = true,
            "--r18-fission-audit" => r18_fission_audit = true,
            "--routec-reserve-growth" => routec_reserve_growth = true,
            "--assimilation-material-flow" => assimilation_material_flow = true,
            "--assimilation-anabolic-incorporation" => {
                assimilation_material_flow = true;
                anabolic_incorporation = true;
            }
            "--post-fission-ecology" => {
                assimilation_material_flow = true;
                anabolic_incorporation = true;
                post_fission_ecology = true;
            }
            "--flux-audit" => flux_audit = true,
            _ => usage(),
        }
        i += 1;
    }
    Config {
        steps,
        seed,
        checkpoint,
        report,
        resume,
        transfer_disabled,
        routeb_spatial_field,
        shared_extracellular_medium,
        shared_medium_from_birth,
        moving_membrane_flux,
        r17_early_whole_membrane,
        r17_delayed_whole_membrane,
        r18_fission_audit,
        routec_reserve_growth,
        assimilation_material_flow,
        anabolic_incorporation,
        post_fission_ecology,
        flux_audit,
    }
}

fn perturb_founder(mesh: &mut chemistry_core::material_mesh::MaterialMesh) {
    // This is the accepted D-088/development founder geometry used by
    // ENTRY-019..021 to demonstrate a physical, rotation-equivariant seed.
    // It is not a behavioral seed and is never read by the polarity state.
    let center = mesh.centroid();
    let (sine, cosine) = 0.3_f64.sin_cos();
    for point in &mut mesh.vertices {
        let x = point[0] - center[0];
        let y = point[1] - center[1];
        point[0] = center[0] + cosine * x - sine * y;
        point[1] = center[1] + sine * x + cosine * y;
    }
    for (index, point) in mesh.vertices.iter_mut().enumerate() {
        let z = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        point[0] += 0.35 * (z - 0.5);
        point[1] += 0.35 * ((z * 7.13).fract() - 0.5);
    }
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] = center[0] + (point[0] - center[0]) * 1.25;
    }
}

fn initial_population(seed: u64) -> MeshPopulation {
    // Match the accepted D-088 / ENTRY-019..027 founder geometry.  The
    // runtime does not synthesize a smaller convenience organism.
    let mut population = MeshPopulation::seed_one(14.0, seed, 2.2);
    for individual in &mut population.individuals {
        perturb_founder(&mut individual.mesh);
        individual.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    population
}

fn develop_founder(individual: &mut MeshIndividual) -> (PolarityState, usize, bool) {
    // Reuse the accepted ENTRY-019 physical-history order.  This is a
    // developmental bootstrap, not a behavioral seed: no actuator, resource,
    // observer, or polarity-to-motor decision is involved here.  The first
    // fission is only probed, never forced, and the runtime begins with the
    // same mother immediately before that accepted physical event.
    let dt = MechParams::default().dt;
    let transport = TransportParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth_mass = individual.birth_mass;
    let mut polarity = PolarityState::homogeneous(&individual.mesh);

    for step in 0..DEVELOPMENT_MAX_STEPS {
        if !individual.mesh.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut individual.mesh, &transport, dt);
        let _ = reactions_step_with_reserve_mode(
            &mut individual.mesh,
            &reaction,
            dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let _ = growth_step(&mut individual.mesh, &reaction, &growth, dt);
        let _ = mechanics_step(&mut individual.mesh, &mechanics);
        let old_vertices = individual.mesh.vertices.clone();
        let _ = remesh(&mut individual.mesh);
        let origin = individual
            .mesh
            .vertices
            .first()
            .and_then(|first| {
                old_vertices
                    .iter()
                    .position(|old| (old[0] - first[0]).hypot(old[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        if step % 10 == 0 {
            let _ = topology_step(&mut individual.mesh, &fission);
        }
        polarity.remap_and_advance(&individual.mesh, origin, dt);

        let eligible = individual.mesh.total_structural_mass() >= 1.35 * birth_mass.max(1e-9)
            && try_local_fission(&individual.mesh, &fission).is_some();
        if eligible {
            return (polarity, step + 1, true);
        }
    }

    // A bounded development horizon is itself valid evidence.  Some lawful
    // founders do not reach fission readiness within the established horizon;
    // preserve that trajectory for the runtime instead of converting a
    // biological negative into a process failure.
    (polarity, DEVELOPMENT_MAX_STEPS, false)
}

fn routeb_field(mesh: &chemistry_core::material_mesh::MaterialMesh) -> SpatialMaterialFieldV1 {
    let nx = 32;
    let ny = 32;
    let dx = 4.0;
    let origin = [mesh.centroid()[0] - 64.0, mesh.centroid()[1] - 64.0];
    let mut n = vec![0.0; nx * ny];
    let mut f = vec![0.0; nx * ny];
    let center = [nx / 2, ny / 2];
    let cell_mass = RESOURCE_MASS / 36.0;
    // Finite six-by-six source patch around the founder.  This is an
    // environmental initial condition, not a bath or a behavior signal.
    for j in center[1] - 3..=center[1] + 2 {
        for i in center[0] - 3..=center[0] + 2 {
            let index = j * nx + i;
            n[index] = cell_mass;
            f[index] = cell_mass;
        }
    }
    SpatialMaterialFieldV1::new(nx, ny, dx, origin, n, f, 6.0)
        .expect("valid route-B finite spatial material field")
}

/// Build the same finite Route-B material units around both daughters after
/// the already-observed developmental fission.  The ecology starts only at
/// this boundary; no environmental material participates in the parent
/// fission and the pre-ecology event is kept as provenance only.
fn post_fission_field(
    meshes: &[chemistry_core::material_mesh::MaterialMesh],
    transfer_enabled: bool,
) -> SpatialMaterialFieldV1 {
    let nx = 64;
    let ny = 64;
    let dx = 4.0;
    let min_x = meshes
        .iter()
        .map(|mesh| mesh.centroid()[0])
        .fold(f64::INFINITY, f64::min);
    let min_y = meshes
        .iter()
        .map(|mesh| mesh.centroid()[1])
        .fold(f64::INFINITY, f64::min);
    let origin = [min_x - 96.0, min_y - 96.0];
    let mut n = vec![0.0; nx * ny];
    let mut f = vec![0.0; nx * ny];
    let cell_mass = RESOURCE_MASS / 36.0;
    for mesh in meshes {
        let center = mesh.centroid();
        let cx = ((center[0] - origin[0]) / dx).floor() as isize;
        let cy = ((center[1] - origin[1]) / dx).floor() as isize;
        for j in cy - 3..=cy + 2 {
            for i in cx - 3..=cx + 2 {
                if i < 0 || j < 0 || i >= nx as isize || j >= ny as isize {
                    continue;
                }
                let index = j as usize * nx + i as usize;
                n[index] += cell_mass;
                f[index] += cell_mass;
            }
        }
    }
    let mut field = SpatialMaterialFieldV1::new(nx, ny, dx, origin, n, f, 6.0)
        .expect("valid post-fission finite spatial material field");
    if !transfer_enabled {
        field.n.fill(0.0);
        field.f.fill(0.0);
    }
    field
}

/// Start the finite-material ecology from the exact unforced first fission
/// already observed by the runtime causality audit.  This is a lifecycle
/// composition correction, not a new growth, motor, or fission law.
fn new_post_fission_snapshot(
    seed: u64,
    transfer_enabled: bool,
) -> RuntimeSnapshot {
    let mut founder = initial_population(seed).individuals.remove(0);
    let (parent_polarity, developmental_bootstrap_steps, boundary_reached) =
        develop_founder(&mut founder);
    assert!(
        boundary_reached,
        "post-fission ecology requires the preregistered unforced developmental boundary"
    );
    let fission = FissionParams::default();
    let (daughter_a, daughter_b, event) =
        try_local_fission(&founder.mesh, &fission).expect("accepted unforced fission boundary");
    let parent_amplitude = parent_polarity.nonconstant_amplitude();
    let (state_a, state_b) =
        parent_polarity.split_after_fission(&event, &daughter_a, &daughter_b, MechParams::default().dt);
    let birth_a = daughter_a.total_structural_mass();
    let birth_b = daughter_b.total_structural_mass();
    let individuals = vec![
        MeshIndividual {
            mesh: daughter_a,
            lineage_id: 2,
            generation: 1,
            birth_mass: birth_a,
            clade: 0,
        },
        MeshIndividual {
            mesh: daughter_b,
            lineage_id: 3,
            generation: 1,
            birth_mass: birth_b,
            clade: 0,
        },
    ];
    let meshes: Vec<_> = individuals.iter().map(|individual| individual.mesh.clone()).collect();
    let field = post_fission_field(&meshes, transfer_enabled);
    let developmental_initial_topology = state_a.topology();
    let previous_centroids = individuals
        .iter()
        .map(|individual| individual.mesh.centroid())
        .collect();
    RuntimeSnapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        seed,
        population: MeshPopulation {
            individuals,
            next_lineage: 4,
            fission_log: vec![event],
        },
        world: FiniteWorldV1::new(Vec::new()),
        spatial_field: Some(field),
        shared_medium: None,
        moving_membrane: None,
        matched_whole_membrane: None,
        reserve_parameters: None,
        assimilation_enabled: true,
        anabolic_incorporation_enabled: true,
        spatial_field_transfer_enabled: transfer_enabled,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_assimilation_n_processed: 0.0,
        cumulative_assimilation_f_processed: 0.0,
        cumulative_assimilation_a_produced: 0.0,
        cumulative_assimilation_m_grown: 0.0,
        cumulative_assimilation_m_incorporated: 0.0,
        cumulative_n_world_loss: 0.0,
        cumulative_f_world_loss: 0.0,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        cumulative_path: 0.0,
        cumulative_contacts: 0,
        first_contact_step: None,
        first_transfer_step: None,
        first_fission_step: None,
        fission_observations: Vec::new(),
        lineage_n_delivered: [(2, 0.0), (3, 0.0)].into_iter().collect(),
        lineage_f_delivered: [(2, 0.0), (3, 0.0)].into_iter().collect(),
        developmental_bootstrap_steps,
        developmental_initial_polarity_amplitude: parent_amplitude,
        developmental_initial_topology,
        developmental_fission_boundary_reached: true,
        ecology_started_after_unforced_fission: true,
        pre_ecology_fission_events: 1,
        motor_steps: 0,
        motor_failures: 0,
        polarity_states: vec![Some(state_a), Some(state_b)],
        previous_centroids,
        flux_audit: None,
        fission_readiness_audit: None,
        scientific_boundary: ScientificBoundary {
            finite_world_exchange: "SpatialMaterialFieldV1 / post-fission daughter-local field".to_string(),
            ..ScientificBoundary::default()
        },
    }
}

fn develop_founder_routeb(
    individual: &mut MeshIndividual,
    field: &mut SpatialMaterialFieldV1,
    transfer_enabled: bool,
    reserve_parameters: Option<&ReserveParams>,
    assimilation_enabled: bool,
    anabolic_incorporation_enabled: bool,
) -> (
    PolarityState,
    usize,
    bool,
    f64,
    f64,
    Option<usize>,
    f64,
    f64,
    f64,
    f64,
    f64,
) {
    let dt = MechParams::default().dt;
    let transport = TransportParams::default();
    let mechanics = MechParams::default();
    let mut reaction = ReactionParams::conservative_v3();
    if let Some(reserve) = reserve_parameters {
        reaction.reserve = *reserve;
        stamp_reserve_equation(&mut individual.mesh);
    }
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth_mass = individual.birth_mass;
    let mut polarity = PolarityState::homogeneous(&individual.mesh);
    let mut cumulative_n = 0.0;
    let mut cumulative_f = 0.0;
    let mut first_transfer_step = None;
    let mut cumulative_assimilation_n = 0.0;
    let mut cumulative_assimilation_f = 0.0;
    let mut cumulative_assimilation_a = 0.0;
    let mut cumulative_assimilation_m = 0.0;
    let mut cumulative_assimilation_m_incorporated = 0.0;

    for step in 0..DEVELOPMENT_MAX_STEPS {
        if !individual.mesh.can_advance_physics() {
            break;
        }
        field.diffuse(dt);
        let mut meshes = vec![individual.mesh.clone()];
        if !transfer_enabled {
            field.n.fill(0.0);
            field.f.fill(0.0);
        }
        let deliveries = field.exchange(&mut meshes, &transport, dt);
        individual.mesh = meshes.pop().expect("one route-B founder");
        if let Some(delivery) = deliveries.first() {
            cumulative_n += delivery.n_delivered;
            cumulative_f += delivery.f_delivered;
            if first_transfer_step.is_none() && delivery.n_delivered + delivery.f_delivered > 1e-12
            {
                first_transfer_step = Some(step + 1);
            }
            field.emit_w(&individual.mesh, delivery.nonfeeding_transport.w_out);
            if assimilation_enabled {
                let area = individual.mesh.area().max(1e-6);
                individual.mesh.interior.n =
                    (individual.mesh.interior.n - delivery.n_delivered / area).max(0.0);
                individual.mesh.interior.f =
                    (individual.mesh.interior.f - delivery.f_delivered / area).max(0.0);
                environmental_assimilation::receive(
                    &mut individual.mesh,
                    delivery.n_delivered,
                    delivery.f_delivered,
                );
            }
        }
        let _ = reactions_step_with_reserve_mode(
            &mut individual.mesh,
            &reaction,
            dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        if assimilation_enabled {
            let processed = environmental_assimilation::process(&mut individual.mesh, &reaction, dt);
            cumulative_assimilation_n += processed.n_processed;
            cumulative_assimilation_f += processed.f_processed;
            cumulative_assimilation_a += processed.assimilation_a_produced;
            if anabolic_incorporation_enabled {
                let incorporated = environmental_assimilation::incorporate_into_structure(
                    &mut individual.mesh,
                    &reaction,
                    dt,
                    processed.assimilation_a_produced,
                );
                cumulative_assimilation_m_incorporated += incorporated.m_produced;
            }
        }
        let mass_before_growth = individual.mesh.total_structural_mass();
        let _ = growth_step(&mut individual.mesh, &reaction, &growth, dt);
        if assimilation_enabled {
            cumulative_assimilation_m +=
                (individual.mesh.total_structural_mass() - mass_before_growth).max(0.0);
        }
        let _ = mechanics_step(&mut individual.mesh, &mechanics);
        let old_vertices = individual.mesh.vertices.clone();
        let _ = remesh(&mut individual.mesh);
        let origin = individual
            .mesh
            .vertices
            .first()
            .and_then(|first| {
                old_vertices
                    .iter()
                    .position(|old| (old[0] - first[0]).hypot(old[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        if step % 10 == 0 {
            let _ = topology_step(&mut individual.mesh, &fission);
        }
        polarity.remap_and_advance(&individual.mesh, origin, dt);
        let eligible = individual.mesh.total_structural_mass() >= 1.35 * birth_mass.max(1e-9)
            && try_local_fission(&individual.mesh, &fission).is_some();
        if eligible {
            return (
                polarity,
                step + 1,
                true,
                cumulative_n,
                cumulative_f,
                first_transfer_step,
                cumulative_assimilation_n,
                cumulative_assimilation_f,
                cumulative_assimilation_a,
                cumulative_assimilation_m,
                cumulative_assimilation_m_incorporated,
            );
        }
    }
    (
        polarity,
        DEVELOPMENT_MAX_STEPS,
        false,
        cumulative_n,
        cumulative_f,
        first_transfer_step,
        cumulative_assimilation_n,
        cumulative_assimilation_f,
        cumulative_assimilation_a,
        cumulative_assimilation_m,
        cumulative_assimilation_m_incorporated,
    )
}

fn separated_world(mesh: &chemistry_core::material_mesh::MaterialMesh) -> FiniteWorldV1 {
    let center = mesh.centroid();
    let mean_edge = mesh.perimeter() / mesh.n().max(1) as f64;
    let directions = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    let resources = directions
        .into_iter()
        .enumerate()
        .map(|(index, direction)| {
            let mut low = 0.0;
            let mut high = mesh
                .vertices
                .iter()
                .map(|point| (point[0] - center[0]).hypot(point[1] - center[1]))
                .fold(0.0, f64::max)
                + RESOURCE_RADIUS
                + mean_edge;
            let surface_gap = |distance: f64| {
                let resource_center = [
                    center[0] + distance * direction[0],
                    center[1] + distance * direction[1],
                ];
                (0..mesh.n())
                    .map(|edge| {
                        let a = mesh.vertices[edge];
                        let b = mesh.vertices[(edge + 1) % mesh.n()];
                        let ab = [b[0] - a[0], b[1] - a[1]];
                        let denom = ab[0] * ab[0] + ab[1] * ab[1];
                        let t = if denom > 0.0 {
                            ((resource_center[0] - a[0]) * ab[0]
                                + (resource_center[1] - a[1]) * ab[1])
                                / denom
                        } else {
                            0.0
                        }
                        .clamp(0.0, 1.0);
                        let nearest = [a[0] + t * ab[0], a[1] + t * ab[1]];
                        (resource_center[0] - nearest[0]).hypot(resource_center[1] - nearest[1])
                    })
                    .fold(f64::INFINITY, f64::min)
                    - RESOURCE_RADIUS
            };
            while surface_gap(high) < mean_edge {
                high *= 2.0;
            }
            for _ in 0..80 {
                let midpoint = 0.5 * (low + high);
                if surface_gap(midpoint) < mean_edge {
                    low = midpoint;
                } else {
                    high = midpoint;
                }
            }
            let distance = high;
            let center = [
                center[0] + distance * direction[0],
                center[1] + distance * direction[1],
            ];
            FiniteWorldResourceV1::new(
                format!("runtime-resource-{index}"),
                center,
                RESOURCE_RADIUS,
                RESOURCE_MASS,
                RESOURCE_MASS,
                RESOURCE_BOUNDARY,
                RESOURCE_BOUNDARY,
            )
        })
        .collect();
    FiniteWorldV1::new(resources)
}

fn new_snapshot(seed: u64) -> RuntimeSnapshot {
    let mut population = initial_population(seed);
    let (developed_polarity, developmental_bootstrap_steps, developmental_fission_boundary_reached) =
        develop_founder(&mut population.individuals[0]);
    // The seeded boundary is part of the accepted developmental founder
    // history.  The standalone ecology begins only after that history, with
    // zero external N/F and the separated finite world as its sole source.
    population.individuals[0].mesh.exterior.n = 0.0;
    population.individuals[0].mesh.exterior.f = 0.0;
    let developmental_initial_polarity_amplitude = developed_polarity.nonconstant_amplitude();
    let developmental_initial_topology = developed_polarity.topology();
    let polarity_states = vec![Some(developed_polarity)];
    let previous_centroids = population
        .individuals
        .iter()
        .map(|individual| individual.mesh.centroid())
        .collect();
    let world = separated_world(&population.individuals[0].mesh);
    RuntimeSnapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        seed,
        population,
        world,
        spatial_field: None,
        shared_medium: None,
        moving_membrane: None,
        matched_whole_membrane: None,
        reserve_parameters: None,
        assimilation_enabled: false,
        anabolic_incorporation_enabled: false,
        spatial_field_transfer_enabled: true,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_assimilation_n_processed: 0.0,
        cumulative_assimilation_f_processed: 0.0,
        cumulative_assimilation_a_produced: 0.0,
        cumulative_assimilation_m_grown: 0.0,
        cumulative_assimilation_m_incorporated: 0.0,
        cumulative_n_world_loss: 0.0,
        cumulative_f_world_loss: 0.0,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        cumulative_path: 0.0,
        cumulative_contacts: 0,
        first_contact_step: None,
        first_transfer_step: None,
        first_fission_step: None,
        fission_observations: Vec::new(),
        lineage_n_delivered: BTreeMap::new(),
        lineage_f_delivered: BTreeMap::new(),
        developmental_bootstrap_steps,
        developmental_initial_polarity_amplitude,
        developmental_initial_topology,
        developmental_fission_boundary_reached,
        ecology_started_after_unforced_fission: false,
        pre_ecology_fission_events: 0,
        motor_steps: 0,
        motor_failures: 0,
        polarity_states,
        previous_centroids,
        flux_audit: None,
        fission_readiness_audit: None,
        scientific_boundary: ScientificBoundary::default(),
    }
}

/// R11 uses the same preregistered separated geometry as the historical
/// finite-world runtime, but replaces four independently represented resource
/// objects with one finite world-owned shared extracellular compartment.
fn new_shared_medium_snapshot(seed: u64, transfer_enabled: bool) -> RuntimeSnapshot {
    let mut snapshot = new_snapshot(seed);
    let resource = separated_world(&snapshot.population.individuals[0].mesh)
        .resources
        .into_iter()
        .next()
        .expect("separated geometry has one or more resources");
    let region = resource.backing.region;
    let mut medium = SharedFiniteExtracellularMediumV1::new(
        region.center,
        region.radius,
        RESOURCE_MASS,
        RESOURCE_MASS,
    )
    .expect("valid shared extracellular medium");
    medium.transfer_enabled = transfer_enabled;
    snapshot.world = FiniteWorldV1::new(Vec::new());
    snapshot.shared_medium = Some(medium);
    snapshot.scientific_boundary.finite_world_exchange =
        "SharedFiniteExtracellularMediumV1 / local membrane exchange".to_string();
    snapshot
}

/// Start the unchanged shared-medium ecology at the existing founder birth
/// state.  Unlike `new_snapshot`, this does not run the developmental
/// bootstrap before environmental exchange, so transfer can be tested before
/// the existing growth/fission eligibility boundary.  It is an opt-in causal
/// composition assay; no growth, fission, transport, or polarity law changes.
fn new_shared_medium_from_birth_snapshot(
    seed: u64,
    transfer_enabled: bool,
) -> RuntimeSnapshot {
    let population = initial_population(seed);
    let resource = separated_world(&population.individuals[0].mesh)
        .resources
        .into_iter()
        .next()
        .expect("separated geometry has one or more resources");
    let region = resource.backing.region;
    let mut medium = SharedFiniteExtracellularMediumV1::new(
        region.center,
        region.radius,
        RESOURCE_MASS,
        RESOURCE_MASS,
    )
    .expect("valid shared extracellular medium");
    medium.transfer_enabled = transfer_enabled;
    let polarity_states: Vec<_> = population
        .individuals
        .iter()
        .map(|individual| Some(PolarityState::homogeneous(&individual.mesh)))
        .collect();
    let developmental_initial_topology = polarity_states
        .first()
        .and_then(Option::as_ref)
        .map(PolarityState::topology)
        .unwrap_or(0);
    let previous_centroids = population
        .individuals
        .iter()
        .map(|individual| individual.mesh.centroid())
        .collect();
    RuntimeSnapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        seed,
        population,
        world: FiniteWorldV1::new(Vec::new()),
        spatial_field: None,
        shared_medium: Some(medium),
        moving_membrane: None,
        matched_whole_membrane: None,
        reserve_parameters: None,
        assimilation_enabled: false,
        anabolic_incorporation_enabled: false,
        spatial_field_transfer_enabled: true,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_assimilation_n_processed: 0.0,
        cumulative_assimilation_f_processed: 0.0,
        cumulative_assimilation_a_produced: 0.0,
        cumulative_assimilation_m_grown: 0.0,
        cumulative_assimilation_m_incorporated: 0.0,
        cumulative_n_world_loss: 0.0,
        cumulative_f_world_loss: 0.0,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        cumulative_path: 0.0,
        cumulative_contacts: 0,
        first_contact_step: None,
        first_transfer_step: None,
        first_fission_step: None,
        fission_observations: Vec::new(),
        lineage_n_delivered: BTreeMap::new(),
        lineage_f_delivered: BTreeMap::new(),
        developmental_bootstrap_steps: 0,
        developmental_initial_polarity_amplitude: 0.0,
        developmental_initial_topology,
        developmental_fission_boundary_reached: false,
        ecology_started_after_unforced_fission: false,
        pre_ecology_fission_events: 0,
        motor_steps: 0,
        motor_failures: 0,
        polarity_states,
        previous_centroids,
        flux_audit: None,
        fission_readiness_audit: None,
        scientific_boundary: ScientificBoundary {
            finite_world_exchange:
                "SharedFiniteExtracellularMediumV1 / local membrane exchange from founder birth"
                    .to_string(),
            ..ScientificBoundary::default()
        },
    }
}

/// R15 starts from the same accepted founder-birth ecology and finite
/// circular control volume as R13, but replaces edge-midpoint requests with
/// the actual membrane/control-volume intersection flux substrate.
fn new_moving_membrane_snapshot(seed: u64, transfer_enabled: bool) -> RuntimeSnapshot {
    let mut snapshot = new_shared_medium_from_birth_snapshot(seed, transfer_enabled);
    let medium = snapshot
        .shared_medium
        .take()
        .expect("founder-birth shared medium exists");
    let mut moving = MovingMembraneFiniteFluxV1::new(
        medium.center,
        medium.radius,
        medium.initial_n_mass,
        medium.initial_f_mass,
    )
    .expect("valid moving-membrane finite medium");
    moving.transfer_enabled = transfer_enabled;
    snapshot.moving_membrane = Some(moving);
    snapshot.scientific_boundary.finite_world_exchange =
        "MovingMembraneFiniteFluxV1 / exact membrane-control-volume intersection".to_string();
    snapshot
}

fn convert_r15_moving_to_r17_whole_membrane(mut snapshot: RuntimeSnapshot) -> RuntimeSnapshot {
    let moving = snapshot
        .moving_membrane
        .take()
        .expect("R17 whole-membrane conversion requires R15 moving medium");
    let mut whole = MatchedWholeMembraneFiniteFeed::new(moving.n_mass, moving.f_mass);
    whole.transfer_enabled = moving.transfer_enabled;
    whole.step = moving.step;
    whole.ledger_n_taken = moving.ledger_n_taken;
    whole.ledger_f_taken = moving.ledger_f_taken;
    snapshot.matched_whole_membrane = Some(whole);
    snapshot.scientific_boundary.finite_world_exchange =
        "R17 assay-only matched whole-membrane finite feed using frozen transport_step"
            .to_string();
    snapshot
}

fn new_r17_early_whole_membrane_snapshot(
    seed: u64,
    transfer_enabled: bool,
) -> RuntimeSnapshot {
    convert_r15_moving_to_r17_whole_membrane(new_moving_membrane_snapshot(
        seed,
        transfer_enabled,
    ))
}

fn new_routeb_snapshot(
    seed: u64,
    transfer_enabled: bool,
    assimilation_enabled: bool,
    anabolic_incorporation_enabled: bool,
) -> RuntimeSnapshot {
    let mut population = initial_population(seed);
    let mut field = routeb_field(&population.individuals[0].mesh);
    let (
        developed_polarity,
        developmental_bootstrap_steps,
        developmental_fission_boundary_reached,
        cumulative_n_delivered,
        cumulative_f_delivered,
        first_transfer_step,
        cumulative_assimilation_n_processed,
        cumulative_assimilation_f_processed,
        cumulative_assimilation_a_produced,
        cumulative_assimilation_m_grown,
        cumulative_assimilation_m_incorporated,
    ) = develop_founder_routeb(
        &mut population.individuals[0],
        &mut field,
        transfer_enabled,
        None,
        assimilation_enabled,
        anabolic_incorporation_enabled,
    );
    population.individuals[0].mesh.exterior.n = 0.0;
    population.individuals[0].mesh.exterior.f = 0.0;
    let developmental_initial_polarity_amplitude = developed_polarity.nonconstant_amplitude();
    let developmental_initial_topology = developed_polarity.topology();
    let previous_centroids = population
        .individuals
        .iter()
        .map(|individual| individual.mesh.centroid())
        .collect();
    RuntimeSnapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        seed,
        population,
        // Keep the historical field present for backward-compatible report
        // shape; Route-B uses only the explicit spatial field below.
        world: FiniteWorldV1::new(Vec::new()),
        spatial_field: Some(field),
        shared_medium: None,
        moving_membrane: None,
        matched_whole_membrane: None,
        reserve_parameters: None,
        assimilation_enabled,
        anabolic_incorporation_enabled,
        spatial_field_transfer_enabled: transfer_enabled,
        cumulative_n_delivered,
        cumulative_f_delivered,
        cumulative_assimilation_n_processed,
        cumulative_assimilation_f_processed,
        cumulative_assimilation_a_produced,
        cumulative_assimilation_m_grown,
        cumulative_assimilation_m_incorporated,
        cumulative_n_world_loss: cumulative_n_delivered,
        cumulative_f_world_loss: cumulative_f_delivered,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        cumulative_path: 0.0,
        cumulative_contacts: 0,
        first_contact_step: None,
        first_transfer_step: first_transfer_step.map(|step| step as u64),
        first_fission_step: None,
        fission_observations: Vec::new(),
        lineage_n_delivered: [(1, cumulative_n_delivered)].into_iter().collect(),
        lineage_f_delivered: [(1, cumulative_f_delivered)].into_iter().collect(),
        developmental_bootstrap_steps,
        developmental_initial_polarity_amplitude,
        developmental_initial_topology,
        developmental_fission_boundary_reached,
        ecology_started_after_unforced_fission: false,
        pre_ecology_fission_events: 0,
        motor_steps: 0,
        motor_failures: 0,
        polarity_states: vec![Some(developed_polarity)],
        previous_centroids,
        flux_audit: None,
        fission_readiness_audit: None,
        scientific_boundary: ScientificBoundary {
            finite_world_exchange: "SpatialMaterialFieldV1 / local edge exchange".to_string(),
            ..ScientificBoundary::default()
        },
    }
}

fn new_routec_snapshot(seed: u64, transfer_enabled: bool) -> RuntimeSnapshot {
    // Route-C composes the already-derived D-091 material reserve with the
    // finite Route-B environmental field. No reserve parameter is selected by
    // this runtime or by the result; D-091 owns the derivation and H=2 choice.
    let reserve = chemistry_core::d091_analysis::selected_reserve_parameters();
    let mut population = initial_population(seed);
    let mut field = routeb_field(&population.individuals[0].mesh);
    let (
        developed_polarity,
        developmental_bootstrap_steps,
        developmental_fission_boundary_reached,
        cumulative_n_delivered,
        cumulative_f_delivered,
        first_transfer_step,
        _cumulative_assimilation_n_processed,
        _cumulative_assimilation_f_processed,
        _cumulative_assimilation_a_produced,
        _cumulative_assimilation_m_grown,
        _cumulative_assimilation_m_incorporated,
    ) = develop_founder_routeb(
        &mut population.individuals[0],
        &mut field,
        transfer_enabled,
        Some(&reserve),
        false,
        false,
    );
    population.individuals[0].mesh.exterior.n = 0.0;
    population.individuals[0].mesh.exterior.f = 0.0;
    let developmental_initial_polarity_amplitude = developed_polarity.nonconstant_amplitude();
    let developmental_initial_topology = developed_polarity.topology();
    let previous_centroids = population
        .individuals
        .iter()
        .map(|individual| individual.mesh.centroid())
        .collect();
    RuntimeSnapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        seed,
        population,
        world: FiniteWorldV1::new(Vec::new()),
        spatial_field: Some(field),
        shared_medium: None,
        moving_membrane: None,
        matched_whole_membrane: None,
        reserve_parameters: Some(reserve),
        assimilation_enabled: false,
        anabolic_incorporation_enabled: false,
        spatial_field_transfer_enabled: transfer_enabled,
        cumulative_n_delivered,
        cumulative_f_delivered,
        cumulative_assimilation_n_processed: 0.0,
        cumulative_assimilation_f_processed: 0.0,
        cumulative_assimilation_a_produced: 0.0,
        cumulative_assimilation_m_grown: 0.0,
        cumulative_assimilation_m_incorporated: 0.0,
        cumulative_n_world_loss: cumulative_n_delivered,
        cumulative_f_world_loss: cumulative_f_delivered,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        cumulative_path: 0.0,
        cumulative_contacts: 0,
        first_contact_step: None,
        first_transfer_step: first_transfer_step.map(|step| step as u64),
        first_fission_step: None,
        fission_observations: Vec::new(),
        lineage_n_delivered: [(1, cumulative_n_delivered)].into_iter().collect(),
        lineage_f_delivered: [(1, cumulative_f_delivered)].into_iter().collect(),
        developmental_bootstrap_steps,
        developmental_initial_polarity_amplitude,
        developmental_initial_topology,
        developmental_fission_boundary_reached,
        ecology_started_after_unforced_fission: false,
        pre_ecology_fission_events: 0,
        motor_steps: 0,
        motor_failures: 0,
        polarity_states: vec![Some(developed_polarity)],
        previous_centroids,
        flux_audit: None,
        fission_readiness_audit: None,
        scientific_boundary: ScientificBoundary {
            finite_world_exchange: "SpatialMaterialFieldV1 / local edge exchange".to_string(),
            frozen_reactions: "ReactionParams::conservative_v3 + sealed D-091 reserve".to_string(),
            frozen_growth: "D-091 reserve-funded GrowthParams".to_string(),
            ..ScientificBoundary::default()
        },
    }
}

fn load_snapshot(path: &Path) -> RuntimeSnapshot {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read snapshot {}: {error}", path.display()));
    let snapshot: RuntimeSnapshot = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot decode snapshot {}: {error}", path.display()));
    assert_eq!(
        snapshot.schema, SCHEMA,
        "unsupported runtime snapshot schema"
    );
    snapshot
}

fn save_snapshot(path: &Path, snapshot: &RuntimeSnapshot) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                panic!(
                    "cannot create checkpoint directory {}: {error}",
                    parent.display()
                )
            });
        }
    }
    let encoded = serde_json::to_vec_pretty(snapshot).expect("snapshot serialization");
    fs::write(path, encoded)
        .unwrap_or_else(|error| panic!("cannot write snapshot {}: {error}", path.display()));
}

fn run_step(snapshot: &mut RuntimeSnapshot) -> usize {
    let dt = MechParams::default().dt;
    let transport = TransportParams::default();
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    if snapshot.previous_centroids.len() < snapshot.population.individuals.len() {
        snapshot.previous_centroids = snapshot
            .population
            .individuals
            .iter()
            .map(|individual| individual.mesh.centroid())
            .collect();
    }
    let active_indices: Vec<usize> = snapshot
        .population
        .individuals
        .iter()
        .enumerate()
        .filter_map(|(index, individual)| {
            (individual.mesh.alive && individual.mesh.can_advance_physics()).then_some(index)
        })
        .collect();

    // Accepted native-ring polarity is the runtime motor source.  It is
    // advanced only after the physical/material step below; thus no future
    // uptake or observer quantity can enter the current motor decision.
    for &index in &active_indices {
        let individual = &mut snapshot.population.individuals[index];
        let Some(state) = snapshot.polarity_states.get(index).and_then(Option::as_ref) else {
            snapshot.motor_failures += 1;
            continue;
        };
        let motor = state.motor_fraction();
        match regulatory_core::apply_local_activated_energy_contractility_with_stick_slip(
            &mut individual.mesh,
            &motor,
            &mechanics,
            &contractility,
            &traction,
        ) {
            Ok(ledger) => {
                snapshot.motor_steps += 1;
                snapshot.cumulative_slipping_contacts += ledger.slipping_contacts;
                if let Some(contractility) = ledger.contractility {
                    snapshot.cumulative_motor_a_spent += contractility.resource_spent;
                }
            }
            Err(_) => {
                snapshot.motor_failures += 1;
                snapshot.polarity_states[index] = None;
            }
        }
    }

    let mut meshes: Vec<_> = active_indices
        .iter()
        .map(|&index| snapshot.population.individuals[index].mesh.clone())
        .collect();
    let deliveries: Vec<RuntimeDelivery> = if let Some(medium) = snapshot.matched_whole_membrane.as_mut() {
        medium.exchange(&mut meshes, &transport, dt)
    } else if let Some(medium) = snapshot.moving_membrane.as_mut() {
        medium
            .exchange(&mut meshes, &transport, dt)
            .into_iter()
            .map(|delivery| RuntimeDelivery {
                organism_index: delivery.organism_index,
                exposed_edges: delivery.interfaced_edges,
                n_delivered: delivery.n_delivered,
                f_delivered: delivery.f_delivered,
                n_world_loss: delivery.n_world_loss,
                f_world_loss: delivery.f_world_loss,
            })
            .collect()
    } else if let Some(medium) = snapshot.shared_medium.as_mut() {
        medium
            .exchange(&mut meshes, &transport, dt)
            .into_iter()
            .map(|delivery| RuntimeDelivery {
                organism_index: delivery.organism_index,
                exposed_edges: delivery.exposed_edges,
                n_delivered: delivery.n_delivered,
                f_delivered: delivery.f_delivered,
                n_world_loss: delivery.n_world_loss,
                f_world_loss: delivery.f_world_loss,
            })
            .collect()
    } else if let Some(field) = snapshot.spatial_field.as_mut() {
        field.diffuse(dt);
        let field_deliveries = field.exchange(&mut meshes, &transport, dt);
        for (mesh, delivery) in meshes.iter().zip(&field_deliveries) {
            field.emit_w(mesh, delivery.nonfeeding_transport.w_out);
        }
        field_deliveries
            .into_iter()
            .map(|delivery| RuntimeDelivery {
                organism_index: delivery.organism_index,
                exposed_edges: delivery.exposed_edges,
                n_delivered: delivery.n_delivered,
                f_delivered: delivery.f_delivered,
                n_world_loss: delivery.n_world_loss,
                f_world_loss: delivery.f_world_loss,
            })
            .collect()
    } else {
        snapshot
            .world
            .exchange(&mut meshes, &transport, dt)
            .into_iter()
            .map(|delivery| RuntimeDelivery {
                organism_index: delivery.organism_index,
                exposed_edges: delivery.exposed_edges,
                n_delivered: delivery.n_delivered,
                f_delivered: delivery.f_delivered,
                n_world_loss: delivery.n_world_loss,
                f_world_loss: delivery.f_world_loss,
            })
            .collect()
    };
    for (&index, mesh) in active_indices.iter().zip(meshes) {
        snapshot.population.individuals[index].mesh = mesh;
    }
    if snapshot.assimilation_enabled {
        for delivery in &deliveries {
            if let Some(&global_index) = active_indices.get(delivery.organism_index) {
                let mesh = &mut snapshot.population.individuals[global_index].mesh;
                let area = mesh.area().max(1e-6);
                mesh.interior.n = (mesh.interior.n - delivery.n_delivered / area).max(0.0);
                mesh.interior.f = (mesh.interior.f - delivery.f_delivered / area).max(0.0);
                environmental_assimilation::receive(
                    mesh,
                    delivery.n_delivered,
                    delivery.f_delivered,
                );
            }
        }
    }
    for delivery in &deliveries {
        if let Some(&global_index) = active_indices.get(delivery.organism_index) {
            let lineage_id = snapshot.population.individuals[global_index].lineage_id;
            *snapshot.lineage_n_delivered.entry(lineage_id).or_default() += delivery.n_delivered;
            *snapshot.lineage_f_delivered.entry(lineage_id).or_default() += delivery.f_delivered;
        }
        snapshot.cumulative_n_delivered += delivery.n_delivered;
        snapshot.cumulative_f_delivered += delivery.f_delivered;
        snapshot.cumulative_n_world_loss += delivery.n_world_loss;
        snapshot.cumulative_f_world_loss += delivery.f_world_loss;
        if delivery.exposed_edges > 0 {
            snapshot.cumulative_contacts += 1;
            if snapshot.first_contact_step.is_none() {
                snapshot.first_contact_step = Some(snapshot.step + 1);
            }
        }
        if delivery.n_delivered > 1e-12 || delivery.f_delivered > 1e-12 {
            if snapshot.first_transfer_step.is_none() {
                snapshot.first_transfer_step = Some(snapshot.step + 1);
            }
        }
    }

    let mut newborns: Vec<(MeshIndividual, PolarityState)> = Vec::new();
    let fission = FissionParams::default();
    let mut reaction = ReactionParams::conservative_v3();
    if let Some(reserve) = snapshot.reserve_parameters {
        reaction.reserve = reserve;
    }
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let assimilation_enabled = snapshot.assimilation_enabled;
    let anabolic_incorporation_enabled = snapshot.anabolic_incorporation_enabled;
    let mut assimilation_n_processed = 0.0;
    let mut assimilation_f_processed = 0.0;
    let mut assimilation_a_produced = 0.0;
    let mut assimilation_m_grown = 0.0;
    let mut assimilation_m_incorporated = 0.0;
    let mut reaction_n_consumed = 0.0;
    let mut reaction_f_consumed = 0.0;
    let mut reaction_a_produced = 0.0;
    let mut reaction_w_produced = 0.0;
    let mut maintenance_a = 0.0;
    let mut growth_a = 0.0;
    let mut growth_material = 0.0;
    let mut growth_w = 0.0;
    let mut fissions = 0;
    let mut fission_readiness_rows = Vec::new();
    let mut fission_readiness_attempt_rows = Vec::new();
    let mut passive_mechanics_shadow_rows = Vec::new();
    let fission_audit_enabled = snapshot.fission_readiness_audit.is_some();
    for &index in &active_indices {
        let individual = &mut snapshot.population.individuals[index];
        if !individual.mesh.alive
            || snapshot
                .polarity_states
                .get(index)
                .and_then(Option::as_ref)
                .is_none()
        {
            continue;
        }
        // FiniteWorldV1::exchange already owns the accepted zero-bath
        // nonfeeding transport pass before allocating finite N/F.  Do not
        // run a second transport step here: it would introduce an extra
        // chemistry boundary between uptake and the frozen reaction kernel.
        let reaction_ledger: ReactionLedger = reactions_step_with_reserve_mode(
            &mut individual.mesh,
            &reaction,
            dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        reaction_n_consumed += reaction_ledger.n_consumed;
        reaction_f_consumed += reaction_ledger.f_consumed;
        reaction_a_produced += reaction_ledger.a_produced;
        reaction_w_produced += reaction_ledger.w_produced;
        maintenance_a += reaction_ledger.a_to_m + reaction_ledger.a_to_l;
        if assimilation_enabled {
            let processed = environmental_assimilation::process(&mut individual.mesh, &reaction, dt);
            assimilation_n_processed += processed.n_processed;
            assimilation_f_processed += processed.f_processed;
            assimilation_a_produced += processed.assimilation_a_produced;
            if anabolic_incorporation_enabled {
                let incorporated = environmental_assimilation::incorporate_into_structure(
                    &mut individual.mesh,
                    &reaction,
                    dt,
                    processed.assimilation_a_produced,
                );
                assimilation_m_incorporated += incorporated.m_produced;
            }
        }
        let mass_before_growth = individual.mesh.total_structural_mass();
        let growth_ledger: GrowthLedger = growth_step(&mut individual.mesh, &reaction, &growth, dt);
        growth_a += growth_ledger.a_consumed_growth;
        growth_material += growth_ledger.m_grown;
        growth_w += growth_ledger.w_from_growth;
        if assimilation_enabled {
            assimilation_m_grown +=
                (individual.mesh.total_structural_mass() - mass_before_growth).max(0.0);
        }
        let old_vertices = individual.mesh.vertices.clone();
        remesh(&mut individual.mesh);
        let tick = snapshot.step + 1;
        if fission_audit_enabled {
            fission_readiness_rows.push(fission_readiness_row(
                &individual.mesh,
                individual.birth_mass,
                &fission,
                tick,
                "before_topology",
                false,
                &TopologyLedger::default(),
            ));
        }
        let topology_ledger = if tick % 10 == 0 {
            topology_step(&mut individual.mesh, &fission)
        } else {
            TopologyLedger::default()
        };
        if fission_audit_enabled {
            fission_readiness_rows.push(fission_readiness_row(
                &individual.mesh,
                individual.birth_mass,
                &fission,
                tick,
                "after_topology",
                false,
                &topology_ledger,
            ));
        }
        let origin = individual
            .mesh
            .vertices
            .first()
            .and_then(|first| {
                old_vertices
                    .iter()
                    .position(|old| (old[0] - first[0]).hypot(old[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        snapshot.polarity_states[index]
            .as_mut()
            .expect("polarity state checked above")
            .remap_and_advance(&individual.mesh, origin, dt);

        let current_centroid = individual.mesh.centroid();
        let previous = snapshot.previous_centroids[index];
        snapshot.cumulative_path +=
            (current_centroid[0] - previous[0]).hypot(current_centroid[1] - previous[1]);
        snapshot.previous_centroids[index] = current_centroid;

        let grown_enough =
            individual.mesh.total_structural_mass() >= 1.35 * individual.birth_mass.max(1e-9);
        if fission_audit_enabled {
            let row = fission_readiness_row(
                &individual.mesh,
                individual.birth_mass,
                &fission,
                tick,
                "fission_evaluation",
                tick % 25 == 0,
                &topology_ledger,
            );
            if tick % 25 == 0 {
                fission_readiness_rows.push(row.clone());
                fission_readiness_attempt_rows.push(row);
            } else {
                fission_readiness_rows.push(row);
            }
            if grown_enough {
                let mut shadow_mesh = individual.mesh.clone();
                let _ = mechanics_step(&mut shadow_mesh, &mechanics);
                let _ = remesh(&mut shadow_mesh);
                let shadow_topology = if tick % 10 == 0 {
                    topology_step(&mut shadow_mesh, &fission)
                } else {
                    TopologyLedger::default()
                };
                passive_mechanics_shadow_rows.push(fission_readiness_row(
                    &shadow_mesh,
                    individual.birth_mass,
                    &fission,
                    tick,
                    "passive_mechanics_shadow",
                    tick % 25 == 0,
                    &shadow_topology,
                ));
            }
        }
        if grown_enough && tick % 25 == 0 {
            if let Some((daughter_a, daughter_b, event)) =
                try_local_fission(&individual.mesh, &fission)
            {
                let parent_lineage_id = individual.lineage_id;
                let parent_generation = individual.generation;
                let parent_n_delivered = snapshot
                    .lineage_n_delivered
                    .get(&parent_lineage_id)
                    .copied()
                    .unwrap_or(0.0);
                let parent_f_delivered = snapshot
                    .lineage_f_delivered
                    .get(&parent_lineage_id)
                    .copied()
                    .unwrap_or(0.0);
                let parent_state = snapshot.polarity_states[index]
                    .as_ref()
                    .expect("parent polarity state")
                    .clone();
                let (state_a, state_b) =
                    parent_state.split_after_fission(&event, &daughter_a, &daughter_b, dt);
                let generation = individual.generation + 1;
                let id_a = snapshot.population.next_lineage;
                let id_b = id_a + 1;
                snapshot.population.next_lineage += 2;
                let clade = individual.clade;
                if snapshot.first_fission_step.is_none() {
                    snapshot.first_fission_step = Some(tick);
                }
                snapshot.fission_observations.push(FissionObservation {
                    step: tick,
                    parent_lineage_id,
                    parent_generation,
                    parent_n_delivered,
                    parent_f_delivered,
                });
                individual.mesh.alive = false;
                individual.mesh.death_reason = Some("fissioned".to_string());
                snapshot.population.fission_log.push(event);
                newborns.push((
                    MeshIndividual {
                        mesh: daughter_a,
                        lineage_id: id_a,
                        generation,
                        birth_mass: 0.0,
                        clade,
                    },
                    state_a,
                ));
                newborns.push((
                    MeshIndividual {
                        mesh: daughter_b,
                        lineage_id: id_b,
                        generation,
                        birth_mass: 0.0,
                        clade,
                    },
                    state_b,
                ));
                let child_len = newborns.len();
                let a = newborns[child_len - 2].0.mesh.total_structural_mass();
                let b = newborns[child_len - 1].0.mesh.total_structural_mass();
                newborns[child_len - 2].0.birth_mass = a;
                newborns[child_len - 1].0.birth_mass = b;
                fissions += 1;
            }
        }
    }
    if let Some(audit) = snapshot.fission_readiness_audit.as_mut() {
        audit.rows.extend(fission_readiness_rows);
        audit
            .official_attempt_ticks
            .extend(fission_readiness_attempt_rows);
        audit
            .passive_mechanics_shadow
            .extend(passive_mechanics_shadow_rows);
    }
    snapshot.cumulative_assimilation_n_processed += assimilation_n_processed;
    snapshot.cumulative_assimilation_f_processed += assimilation_f_processed;
    snapshot.cumulative_assimilation_a_produced += assimilation_a_produced;
    snapshot.cumulative_assimilation_m_grown += assimilation_m_grown;
    snapshot.cumulative_assimilation_m_incorporated += assimilation_m_incorporated;
    for (individual, state) in newborns {
        snapshot.population.individuals.push(individual);
        snapshot.polarity_states.push(Some(state));
        snapshot.previous_centroids.push(
            snapshot
                .population
                .individuals
                .last()
                .unwrap()
                .mesh
                .centroid(),
        );
    }
    snapshot.cumulative_fissions += fissions;
    snapshot.step += 1;
    update_flux_audit(
        snapshot,
        reaction_n_consumed,
        reaction_f_consumed,
        reaction_a_produced,
        reaction_w_produced,
        maintenance_a,
        growth_a,
        growth_material,
        growth_w,
    );
    fissions
}

fn report(snapshot: &RuntimeSnapshot, checkpoint: &Path) -> RuntimeReport {
    let current_max_polarity_amplitude = snapshot
        .polarity_states
        .iter()
        .filter_map(Option::as_ref)
        .map(PolarityState::nonconstant_amplitude)
        .fold(0.0, f64::max);
    RuntimeReport {
        schema: SCHEMA,
        step: snapshot.step,
        seed: snapshot.seed,
        living_count: snapshot.population.living_count(),
        total_individuals: snapshot.population.individuals.len(),
        maximum_generation: snapshot
            .population
            .individuals
            .iter()
            .map(|individual| individual.generation)
            .max()
            .unwrap_or(0),
        fission_events: snapshot.cumulative_fissions,
        world_n_mass_remaining: snapshot.world.total_n_mass(),
        world_f_mass_remaining: snapshot.world.total_f_mass(),
        spatial_field_n_mass_remaining: snapshot
            .spatial_field
            .as_ref()
            .map(SpatialMaterialFieldV1::total_n_mass)
            .unwrap_or(0.0),
        spatial_field_f_mass_remaining: snapshot
            .spatial_field
            .as_ref()
            .map(SpatialMaterialFieldV1::total_f_mass)
            .unwrap_or(0.0),
        shared_medium_n_mass_remaining: snapshot
            .shared_medium
            .as_ref()
            .map(SharedFiniteExtracellularMediumV1::total_n_mass)
            .unwrap_or(0.0),
        shared_medium_f_mass_remaining: snapshot
            .shared_medium
            .as_ref()
            .map(SharedFiniteExtracellularMediumV1::total_f_mass)
            .unwrap_or(0.0),
        moving_membrane_n_mass_remaining: snapshot
            .moving_membrane
            .as_ref()
            .map(MovingMembraneFiniteFluxV1::total_n_mass)
            .unwrap_or(0.0),
        moving_membrane_f_mass_remaining: snapshot
            .moving_membrane
            .as_ref()
            .map(MovingMembraneFiniteFluxV1::total_f_mass)
            .unwrap_or(0.0),
        matched_whole_membrane_n_mass_remaining: snapshot
            .matched_whole_membrane
            .as_ref()
            .map(MatchedWholeMembraneFiniteFeed::total_n_mass)
            .unwrap_or(0.0),
        matched_whole_membrane_f_mass_remaining: snapshot
            .matched_whole_membrane
            .as_ref()
            .map(MatchedWholeMembraneFiniteFeed::total_f_mass)
            .unwrap_or(0.0),
        cumulative_n_delivered: snapshot.cumulative_n_delivered,
        cumulative_f_delivered: snapshot.cumulative_f_delivered,
        cumulative_assimilation_n_processed: snapshot.cumulative_assimilation_n_processed,
        cumulative_assimilation_f_processed: snapshot.cumulative_assimilation_f_processed,
        cumulative_assimilation_a_produced: snapshot.cumulative_assimilation_a_produced,
        cumulative_assimilation_m_grown: snapshot.cumulative_assimilation_m_grown,
        cumulative_assimilation_m_incorporated: snapshot.cumulative_assimilation_m_incorporated,
        world_n_conservation_error: snapshot.cumulative_n_delivered
            - snapshot.cumulative_n_world_loss,
        world_f_conservation_error: snapshot.cumulative_f_delivered
            - snapshot.cumulative_f_world_loss,
        motor_steps: snapshot.motor_steps,
        motor_failures: snapshot.motor_failures,
        cumulative_motor_a_spent: snapshot.cumulative_motor_a_spent,
        cumulative_slipping_contacts: snapshot.cumulative_slipping_contacts,
        cumulative_path: snapshot.cumulative_path,
        cumulative_contacts: snapshot.cumulative_contacts,
        first_contact_step: snapshot.first_contact_step,
        first_transfer_step: snapshot.first_transfer_step,
        first_fission_step: snapshot.first_fission_step,
        first_fission_before_first_transfer: snapshot.first_fission_step.map(|fission| {
            snapshot
                .first_transfer_step
                .map(|transfer| fission < transfer)
                .unwrap_or(true)
        }),
        fission_observations: snapshot.fission_observations.clone(),
        resource_transfer_enabled: if let Some(medium) = snapshot.matched_whole_membrane.as_ref() {
            medium.transfer_enabled
        } else if let Some(medium) = snapshot.moving_membrane.as_ref() {
            medium.transfer_enabled
        } else if let Some(medium) = snapshot.shared_medium.as_ref() {
            medium.transfer_enabled
        } else {
            snapshot
                .spatial_field
                .as_ref()
                .map(|_| snapshot.spatial_field_transfer_enabled)
                .unwrap_or(snapshot.world.transfer_enabled)
        },
        resource_mode: if snapshot.matched_whole_membrane.is_some() {
            "R17MatchedWholeMembraneFiniteFeedV1".to_string()
        } else if snapshot.moving_membrane.is_some() {
            "MovingMembraneFiniteFluxV1".to_string()
        } else if snapshot.shared_medium.is_some() {
            "SharedFiniteExtracellularMediumV1".to_string()
        } else if snapshot.spatial_field.is_some() {
            "SpatialMaterialFieldV1".to_string()
        } else {
            "FiniteWorldV1".to_string()
        },
        reserve_enabled: snapshot.reserve_parameters.is_some(),
        developmental_bootstrap_steps: snapshot.developmental_bootstrap_steps,
        developmental_initial_topology: snapshot.developmental_initial_topology,
        developmental_initial_polarity_amplitude: snapshot.developmental_initial_polarity_amplitude,
        developmental_fission_boundary_reached: snapshot.developmental_fission_boundary_reached,
        ecology_started_after_unforced_fission: snapshot.ecology_started_after_unforced_fission,
        pre_ecology_fission_events: snapshot.pre_ecology_fission_events,
        current_max_polarity_amplitude,
        terminal_observer_death_reasons: snapshot
            .population
            .individuals
            .iter()
            .map(|individual| individual.mesh.observer_death_reason())
            .collect(),
        active_motility:
            "ENTRY-019..027 native inherited-polarity motor with existing A-funded stick-slip"
                .to_string(),
        autonomous_resource_acquisition: "NOT_ESTABLISHED",
        resource_causal_reproduction: "NOT_ESTABLISHED",
        checkpoint: checkpoint.display().to_string(),
        flux_audit: snapshot.flux_audit.clone(),
        fission_readiness_audit: snapshot.fission_readiness_audit.clone(),
    }
}

fn main() {
    let config = parse_config();
    let mut snapshot = config
        .resume
        .as_deref()
        .map(load_snapshot)
        .unwrap_or_else(|| {
            if config.r17_early_whole_membrane {
                new_r17_early_whole_membrane_snapshot(config.seed, !config.transfer_disabled)
            } else if config.moving_membrane_flux {
                new_moving_membrane_snapshot(config.seed, !config.transfer_disabled)
            } else if config.shared_medium_from_birth {
                new_shared_medium_from_birth_snapshot(config.seed, !config.transfer_disabled)
            } else if config.post_fission_ecology {
                new_post_fission_snapshot(config.seed, !config.transfer_disabled)
            } else if config.shared_extracellular_medium {
                new_shared_medium_snapshot(config.seed, !config.transfer_disabled)
            } else if config.assimilation_material_flow {
                new_routeb_snapshot(
                    config.seed,
                    !config.transfer_disabled,
                    true,
                    config.anabolic_incorporation,
                )
            } else if config.routec_reserve_growth {
                new_routec_snapshot(config.seed, !config.transfer_disabled)
            } else if config.routeb_spatial_field {
                new_routeb_snapshot(config.seed, !config.transfer_disabled, false, false)
            } else {
                new_snapshot(config.seed)
            }
        });
    if config.r17_delayed_whole_membrane {
        snapshot = convert_r15_moving_to_r17_whole_membrane(snapshot);
    }
    if config.transfer_disabled {
        if let Some(medium) = snapshot.matched_whole_membrane.as_mut() {
            medium.transfer_enabled = false;
        } else if let Some(medium) = snapshot.moving_membrane.as_mut() {
            medium.transfer_enabled = false;
        } else if let Some(medium) = snapshot.shared_medium.as_mut() {
            medium.transfer_enabled = false;
        } else if snapshot.spatial_field.is_none() {
            snapshot.world.transfer_enabled = false;
        }
    }
    if config.flux_audit && snapshot.flux_audit.is_none() {
        snapshot.flux_audit = Some(FluxAuditState::new(&snapshot));
    }
    if config.r18_fission_audit && snapshot.fission_readiness_audit.is_none() {
        snapshot.fission_readiness_audit = Some(FissionReadinessAudit {
            rows: Vec::new(),
            official_attempt_ticks: Vec::new(),
            passive_mechanics_shadow: Vec::new(),
        });
    }
    let target = snapshot.step.saturating_add(config.steps);
    while snapshot.step < target {
        let _ = run_step(&mut snapshot);
    }
    save_snapshot(&config.checkpoint, &snapshot);
    if let Some(parent) = config.report.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("report directory");
        }
    }
    let rendered = serde_json::to_vec_pretty(&report(&snapshot, &config.checkpoint))
        .expect("report serialization");
    fs::write(&config.report, rendered)
        .unwrap_or_else(|error| panic!("cannot write report {}: {error}", config.report.display()));
    println!(
        "{}",
        serde_json::to_string_pretty(&report(&snapshot, &config.checkpoint)).unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_round_trip_preserves_resume_step() {
        let path = std::env::temp_dir().join(format!(
            "digital-cell-m2-runtime-{}.json",
            std::process::id()
        ));
        let mut original = new_snapshot(1);
        run_step(&mut original);
        run_step(&mut original);
        save_snapshot(&path, &original);
        let mut resumed = load_snapshot(&path);
        assert_eq!(resumed.step, 2);
        run_step(&mut original);
        run_step(&mut resumed);
        assert_eq!(resumed.step, 3);
        assert_eq!(resumed.seed, original.seed);
        assert!(original.developmental_bootstrap_steps > 0);
        assert!(original.developmental_initial_polarity_amplitude > 0.0);
        assert_eq!(
            resumed.developmental_initial_topology,
            original.developmental_initial_topology
        );
        assert_eq!(
            resumed.developmental_fission_boundary_reached,
            original.developmental_fission_boundary_reached
        );
        assert_eq!(
            resumed.population.individuals.len(),
            original.population.individuals.len()
        );
        assert_eq!(resumed.cumulative_contacts, original.cumulative_contacts);
        assert_eq!(resumed.first_contact_step, original.first_contact_step);
        assert_eq!(resumed.first_transfer_step, original.first_transfer_step);
        assert!((resumed.cumulative_path - original.cumulative_path).abs() <= 1e-12);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn assimilation_checkpoint_round_trip_preserves_opt_in_material_state() {
        let path = std::env::temp_dir().join(format!(
            "digital-cell-m2-assimilation-runtime-{}.json",
            std::process::id()
        ));
        let mut original = new_post_fission_snapshot(2, true);
        original.population.individuals[0].mesh.interior.assimilation_n = 0.41;
        original.population.individuals[0].mesh.interior.assimilation_f = 0.37;
        original.population.individuals[1].mesh.interior.assimilation_n = 0.29;
        original.population.individuals[1].mesh.interior.assimilation_f = 0.23;
        save_snapshot(&path, &original);
        let resumed = load_snapshot(&path);
        for (left, right) in original
            .population
            .individuals
            .iter()
            .zip(resumed.population.individuals.iter())
        {
            assert_eq!(
                left.mesh.interior.assimilation_n,
                right.mesh.interior.assimilation_n
            );
            assert_eq!(
                left.mesh.interior.assimilation_f,
                right.mesh.interior.assimilation_f
            );
        }
        assert_eq!(
            original.spatial_field.as_ref().map(SpatialMaterialFieldV1::total_n_mass),
            resumed.spatial_field.as_ref().map(SpatialMaterialFieldV1::total_n_mass)
        );
        let _ = fs::remove_file(path);
    }
}
