use chemistry_core::material_mesh::MeshContractVersion;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_population::{MeshIndividual, MeshPopulation};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_transport::transport_step;
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    ContractilityParamsV1, FiniteWorldResourceV1, FiniteWorldV1, StickSlipTractionParamsV1,
};
use serde::{Deserialize, Serialize};
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
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
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
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
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
}

fn usage() -> ! {
    eprintln!(
        "usage: digital-protocell-m2-runtime [--steps N] [--seed N] \\\n+         [--checkpoint PATH] [--report PATH] [--resume PATH]"
    );
    std::process::exit(2);
}

fn parse_config() -> Config {
    let mut steps = 100_u64;
    let mut seed = 1_u64;
    let mut checkpoint = PathBuf::from("m2-lifeform-runtime.snapshot.json");
    let mut report = PathBuf::from("m2-lifeform-runtime.report.json");
    let mut resume = None;
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
    // Match the accepted D-088 / ENTRY-019..027 founder geometry.  The
    // runtime does not synthesize a smaller convenience organism.
    let mut population = MeshPopulation::seed_one(14.0, seed, 2.2);
    for individual in &mut population.individuals {
        perturb_founder(&mut individual.mesh);
        individual.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
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
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_n_world_loss: 0.0,
        cumulative_f_world_loss: 0.0,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        cumulative_path: 0.0,
        cumulative_contacts: 0,
        first_contact_step: None,
        first_transfer_step: None,
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
    let deliveries = snapshot.world.exchange(&mut meshes, &transport, dt);
    for (&index, mesh) in active_indices.iter().zip(meshes) {
        snapshot.population.individuals[index].mesh = mesh;
    }
    for delivery in &deliveries {
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
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
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
        let _ = growth_step(&mut individual.mesh, &reaction, &growth, dt);
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
        cumulative_n_delivered: snapshot.cumulative_n_delivered,
        cumulative_f_delivered: snapshot.cumulative_f_delivered,
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
        .unwrap_or_else(|| new_snapshot(config.seed));
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
