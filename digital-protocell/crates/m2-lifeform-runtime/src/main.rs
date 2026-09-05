use chemistry_core::material_mesh::MeshContractVersion;
use chemistry_core::environmental_assimilation;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_population::{MeshIndividual, MeshPopulation};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_transport::transport_step;
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    ContractilityParamsV1, FiniteWorldResourceV1, FiniteWorldV1, SpatialMaterialFieldV1,
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
    /// Opt-in D-091 reserve composition; absent preserves reserve-off runtime.
    #[serde(default)]
    reserve_parameters: Option<ReserveParams>,
    /// Opt-in finite environmental assimilation substrate. Absent/false
    /// preserves all historical runtime compositions.
    #[serde(default)]
    assimilation_enabled: bool,
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
    motor_steps: u64,
    motor_failures: u64,
    #[serde(default)]
    polarity_states: Vec<Option<PolarityState>>,
    #[serde(default)]
    previous_centroids: Vec<[f64; 2]>,
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
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
    cumulative_assimilation_n_processed: f64,
    cumulative_assimilation_f_processed: f64,
    cumulative_assimilation_a_produced: f64,
    cumulative_assimilation_m_grown: f64,
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
    current_max_polarity_amplitude: f64,
    terminal_observer_death_reasons: Vec<Option<&'static str>>,
    active_motility: String,
    autonomous_resource_acquisition: &'static str,
    resource_causal_reproduction: &'static str,
    checkpoint: String,
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
    routec_reserve_growth: bool,
    assimilation_material_flow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FissionObservation {
    step: u64,
    parent_lineage_id: u64,
    parent_generation: u32,
    parent_n_delivered: f64,
    parent_f_delivered: f64,
}

#[derive(Debug, Default)]
struct RuntimeDelivery {
    organism_index: usize,
    exposed_edges: usize,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
}

fn default_true() -> bool {
    true
}

fn usage() -> ! {
    eprintln!(
        "usage: digital-protocell-m2-runtime [--steps N] [--seed N] \\\n          [--checkpoint PATH] [--report PATH] [--resume PATH] \\\n          [--transfer-disabled] [--routeb-spatial-field] [--routec-reserve-growth] [--assimilation-material-flow]"
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
    let mut routec_reserve_growth = false;
    let mut assimilation_material_flow = false;
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
            "--routec-reserve-growth" => routec_reserve_growth = true,
            "--assimilation-material-flow" => assimilation_material_flow = true,
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
        routec_reserve_growth,
        assimilation_material_flow,
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

fn develop_founder_routeb(
    individual: &mut MeshIndividual,
    field: &mut SpatialMaterialFieldV1,
    transfer_enabled: bool,
    reserve_parameters: Option<&ReserveParams>,
    assimilation_enabled: bool,
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
        reserve_parameters: None,
        assimilation_enabled: false,
        spatial_field_transfer_enabled: true,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_assimilation_n_processed: 0.0,
        cumulative_assimilation_f_processed: 0.0,
        cumulative_assimilation_a_produced: 0.0,
        cumulative_assimilation_m_grown: 0.0,
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
        motor_steps: 0,
        motor_failures: 0,
        polarity_states,
        previous_centroids,
        scientific_boundary: ScientificBoundary::default(),
    }
}

fn new_routeb_snapshot(
    seed: u64,
    transfer_enabled: bool,
    assimilation_enabled: bool,
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
    ) = develop_founder_routeb(
        &mut population.individuals[0],
        &mut field,
        transfer_enabled,
        None,
        assimilation_enabled,
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
        reserve_parameters: None,
        assimilation_enabled,
        spatial_field_transfer_enabled: transfer_enabled,
        cumulative_n_delivered,
        cumulative_f_delivered,
        cumulative_assimilation_n_processed,
        cumulative_assimilation_f_processed,
        cumulative_assimilation_a_produced,
        cumulative_assimilation_m_grown,
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
        motor_steps: 0,
        motor_failures: 0,
        polarity_states: vec![Some(developed_polarity)],
        previous_centroids,
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
    ) = develop_founder_routeb(
        &mut population.individuals[0],
        &mut field,
        transfer_enabled,
        Some(&reserve),
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
        reserve_parameters: Some(reserve),
        assimilation_enabled: false,
        spatial_field_transfer_enabled: transfer_enabled,
        cumulative_n_delivered,
        cumulative_f_delivered,
        cumulative_assimilation_n_processed: 0.0,
        cumulative_assimilation_f_processed: 0.0,
        cumulative_assimilation_a_produced: 0.0,
        cumulative_assimilation_m_grown: 0.0,
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
        motor_steps: 0,
        motor_failures: 0,
        polarity_states: vec![Some(developed_polarity)],
        previous_centroids,
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
    let deliveries: Vec<RuntimeDelivery> = if let Some(field) = snapshot.spatial_field.as_mut() {
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
    let mut assimilation_n_processed = 0.0;
    let mut assimilation_f_processed = 0.0;
    let mut assimilation_a_produced = 0.0;
    let mut assimilation_m_grown = 0.0;
    let mut fissions = 0;
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
            assimilation_n_processed += processed.n_processed;
            assimilation_f_processed += processed.f_processed;
            assimilation_a_produced += processed.assimilation_a_produced;
        }
        let mass_before_growth = individual.mesh.total_structural_mass();
        let _ = growth_step(&mut individual.mesh, &reaction, &growth, dt);
        if assimilation_enabled {
            assimilation_m_grown +=
                (individual.mesh.total_structural_mass() - mass_before_growth).max(0.0);
        }
        let old_vertices = individual.mesh.vertices.clone();
        remesh(&mut individual.mesh);
        let tick = snapshot.step + 1;
        if tick % 10 == 0 {
            let _ = topology_step(&mut individual.mesh, &fission);
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
    snapshot.cumulative_assimilation_n_processed += assimilation_n_processed;
    snapshot.cumulative_assimilation_f_processed += assimilation_f_processed;
    snapshot.cumulative_assimilation_a_produced += assimilation_a_produced;
    snapshot.cumulative_assimilation_m_grown += assimilation_m_grown;
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
        cumulative_n_delivered: snapshot.cumulative_n_delivered,
        cumulative_f_delivered: snapshot.cumulative_f_delivered,
        cumulative_assimilation_n_processed: snapshot.cumulative_assimilation_n_processed,
        cumulative_assimilation_f_processed: snapshot.cumulative_assimilation_f_processed,
        cumulative_assimilation_a_produced: snapshot.cumulative_assimilation_a_produced,
        cumulative_assimilation_m_grown: snapshot.cumulative_assimilation_m_grown,
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
        resource_transfer_enabled: snapshot
            .spatial_field
            .as_ref()
            .map(|_| snapshot.spatial_field_transfer_enabled)
            .unwrap_or(snapshot.world.transfer_enabled),
        resource_mode: if snapshot.spatial_field.is_some() {
            "SpatialMaterialFieldV1".to_string()
        } else {
            "FiniteWorldV1".to_string()
        },
        reserve_enabled: snapshot.reserve_parameters.is_some(),
        developmental_bootstrap_steps: snapshot.developmental_bootstrap_steps,
        developmental_initial_topology: snapshot.developmental_initial_topology,
        developmental_initial_polarity_amplitude: snapshot.developmental_initial_polarity_amplitude,
        developmental_fission_boundary_reached: snapshot.developmental_fission_boundary_reached,
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
    }
}

fn main() {
    let config = parse_config();
    let mut snapshot = config
        .resume
        .as_deref()
        .map(load_snapshot)
        .unwrap_or_else(|| {
            if config.assimilation_material_flow {
                new_routeb_snapshot(config.seed, !config.transfer_disabled, true)
            } else if config.routec_reserve_growth {
                new_routec_snapshot(config.seed, !config.transfer_disabled)
            } else if config.routeb_spatial_field {
                new_routeb_snapshot(config.seed, !config.transfer_disabled, false)
            } else {
                new_snapshot(config.seed)
            }
        });
    if config.transfer_disabled && snapshot.spatial_field.is_none() {
        snapshot.world.transfer_enabled = false;
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
}
