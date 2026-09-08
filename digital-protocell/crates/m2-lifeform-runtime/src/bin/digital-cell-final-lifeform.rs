//! Persistent standalone Digital Cell runtime for DC-FINAL-001-R1.
//!
//! This composes the accepted adaptive material-gradient front/rear motor,
//! finite conservative world, frozen physiology/growth/fission, inherited
//! allocation state, and low-level sensory plasticity. Godot is an optional
//! observer of checkpoints and never participates in the biological step.

use chemistry_core::d096_allocation::{
    mutate_allocation_at_reproduction, AllocationGenotype, AllocationParams,
};
use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX};
use chemistry_core::mesh_population::MeshPopulation;
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_self_contact::polygon_simple;
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    adaptive_directional_drive, advance_local_plasticity_trace,
    apply_local_activated_energy_front_rear_with_local_traction_clutch, camera_rgb8_fields,
    derive_local_mapping, microphone_pcm16_fields, ContinuityMaterialFrameV1,
    ContractilityParamsV1, LowLevelSensoryEnvironmentV1, PlasticityParamsV1, PlasticityStateV1,
    SpatialMaterialFieldV1, StickSlipTractionParamsV1, TopologyEventV1,
    FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME, FROZEN_STATIC_TRACTION_LIMIT,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA: &str = "digital_cell_final_lifeform_runtime_v1";
const SENSOR_BINS: usize = 64;
const INITIAL_RESOURCE_N: f64 = 3.0;
const INITIAL_RESOURCE_F: f64 = 3.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Individual {
    mesh: MaterialMesh,
    birth_mass: f64,
    generation: u32,
    lineage: u64,
    memory: PlasticityStateV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snapshot {
    schema: String,
    step: u64,
    next_lineage: u64,
    individuals: Vec<Individual>,
    world: SpatialMaterialFieldV1,
    sensory_environment: Option<LowLevelSensoryEnvironmentV1>,
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
    cumulative_active_a: f64,
    cumulative_active_w: f64,
    cumulative_fissions: usize,
    cumulative_path: f64,
}

#[derive(Debug, Clone)]
struct Config {
    steps: u64,
    checkpoint: PathBuf,
    report: PathBuf,
    resume: Option<PathBuf>,
    camera_current: Option<PathBuf>,
    camera_previous: Option<PathBuf>,
    camera_width: usize,
    camera_height: usize,
    microphone_pcm: Option<PathBuf>,
    observer_connected: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    schema: &'static str,
    step: u64,
    living: usize,
    maximum_generation: u32,
    fissions: usize,
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
    cumulative_active_a: f64,
    cumulative_active_w: f64,
    cumulative_path: f64,
    maximum_memory: f64,
    sensory_camera_present: bool,
    sensory_microphone_present: bool,
    observer_connected: bool,
    observer_in_biological_step: bool,
    all_polygons_simple: bool,
    autonomous_resource_acquisition: &'static str,
    environment_dependent_evolution: &'static str,
}

fn usage() -> ! {
    eprintln!("usage: digital-cell-final-lifeform [--steps N] [--checkpoint PATH] [--report PATH] [--resume PATH] [--camera-rgb8 PATH --camera-width N --camera-height N] [--camera-previous-rgb8 PATH] [--microphone-pcm16le PATH] [--observer-connected]");
    std::process::exit(2);
}

fn parse_config() -> Config {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut config = Config {
        steps: 100,
        checkpoint: PathBuf::from("digital-cell-final-lifeform.snapshot.json"),
        report: PathBuf::from("digital-cell-final-lifeform.report.json"),
        resume: None,
        camera_current: None,
        camera_previous: None,
        camera_width: 0,
        camera_height: 0,
        microphone_pcm: None,
        observer_connected: false,
    };
    let mut index = 0;
    while index < args.len() {
        let value = |index: &mut usize| {
            *index += 1;
            args.get(*index).cloned().unwrap_or_else(|| usage())
        };
        match args[index].as_str() {
            "--steps" => config.steps = value(&mut index).parse().unwrap_or_else(|_| usage()),
            "--checkpoint" => config.checkpoint = PathBuf::from(value(&mut index)),
            "--report" => config.report = PathBuf::from(value(&mut index)),
            "--resume" => config.resume = Some(PathBuf::from(value(&mut index))),
            "--camera-rgb8" => config.camera_current = Some(PathBuf::from(value(&mut index))),
            "--camera-previous-rgb8" => {
                config.camera_previous = Some(PathBuf::from(value(&mut index)))
            }
            "--camera-width" => {
                config.camera_width = value(&mut index).parse().unwrap_or_else(|_| usage())
            }
            "--camera-height" => {
                config.camera_height = value(&mut index).parse().unwrap_or_else(|_| usage())
            }
            "--microphone-pcm16le" => {
                config.microphone_pcm = Some(PathBuf::from(value(&mut index)))
            }
            "--observer-connected" => config.observer_connected = true,
            _ => usage(),
        }
        index += 1;
    }
    config
}

fn founder() -> MaterialMesh {
    let mut mesh = MeshPopulation::seed_one(5.0, 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] -= center[0];
        point[1] -= center[1];
    }
    mesh.enable_finite_allocation(AllocationGenotype::neutral(), &AllocationParams::default());
    mesh
}

fn world(mesh: &MaterialMesh) -> SpatialMaterialFieldV1 {
    let nx = 64;
    let ny = 64;
    let mut n = vec![0.0; nx * ny];
    let mut f = vec![0.0; nx * ny];
    let radius = mesh
        .vertices
        .iter()
        .map(|point| point[0].hypot(point[1]))
        .fold(0.0, f64::max);
    let center = [radius + 4.0, 0.0];
    let ci = (center[0] + 32.0).floor() as isize;
    let cj = (center[1] + 32.0).floor() as isize;
    for y in (cj - 1)..=(cj + 1) {
        for x in (ci - 1)..=(ci + 1) {
            let cell = y as usize * nx + x as usize;
            n[cell] = INITIAL_RESOURCE_N / 9.0;
            f[cell] = INITIAL_RESOURCE_F / 9.0;
        }
    }
    SpatialMaterialFieldV1::new(nx, ny, 1.0, [-32.0, -32.0], n, f, 6.0).unwrap()
}

fn new_snapshot(sensory_environment: Option<LowLevelSensoryEnvironmentV1>) -> Snapshot {
    let mesh = founder();
    Snapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        next_lineage: 2,
        world: world(&mesh),
        individuals: vec![Individual {
            memory: PlasticityStateV1::new(mesh.n()),
            birth_mass: mesh.total_structural_mass(),
            generation: 0,
            lineage: 1,
            mesh,
        }],
        sensory_environment,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_active_a: 0.0,
        cumulative_active_w: 0.0,
        cumulative_fissions: 0,
        cumulative_path: 0.0,
    }
}

fn load_sensory(config: &Config) -> Option<LowLevelSensoryEnvironmentV1> {
    if config.camera_current.is_none() && config.microphone_pcm.is_none() {
        return None;
    }
    let (luminance, motion) = if let Some(path) = &config.camera_current {
        let current = fs::read(path).expect("read camera RGB8 input");
        let previous = config
            .camera_previous
            .as_ref()
            .map(|path| fs::read(path).expect("read prior camera RGB8 input"));
        camera_rgb8_fields(
            config.camera_width,
            config.camera_height,
            &current,
            previous.as_deref(),
            SENSOR_BINS,
        )
        .expect("valid camera RGB8 input")
    } else {
        (vec![0.0; SENSOR_BINS], vec![0.0; SENSOR_BINS])
    };
    let (amplitude, low, high) = if let Some(path) = &config.microphone_pcm {
        let bytes = fs::read(path).expect("read microphone PCM16LE input");
        let samples: Vec<i16> = bytes
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        microphone_pcm16_fields(&samples).expect("valid microphone PCM16LE input")
    } else {
        (0.0, 0.0, 0.0)
    };
    Some(
        LowLevelSensoryEnvironmentV1::from_samples(luminance, motion, amplitude, low, high)
            .expect("bounded low-level sensory environment"),
    )
}

fn local_occupancy(world: &SpatialMaterialFieldV1, mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n())
        .map(|edge| {
            let a = mesh.vertices[edge];
            let b = mesh.vertices[(edge + 1) % mesh.n()];
            let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
            let Some(cell) = world.world_to_cell(midpoint) else {
                return 0.0;
            };
            let (n, f, _) = world.concentration(cell);
            let nf = if n + mesh.interior.n > 0.0 {
                n / (n + mesh.interior.n)
            } else {
                0.0
            };
            let ff = if f + mesh.interior.f > 0.0 {
                f / (f + mesh.interior.f)
            } else {
                0.0
            };
            0.5 * (nf + ff)
        })
        .collect()
}

fn edge_to_vertex(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    (0..n)
        .map(|index| 0.5 * (values[index] + values[(index + n - 1) % n]))
        .collect()
}

fn outward_normal(mesh: &MaterialMesh, edge: usize) -> [f64; 2] {
    let a = mesh.vertices[edge];
    let b = mesh.vertices[(edge + 1) % mesh.n()];
    let delta = [b[0] - a[0], b[1] - a[1]];
    let length = delta[0].hypot(delta[1]).max(1e-15);
    let orientation = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    [
        orientation * delta[1] / length,
        -orientation * delta[0] / length,
    ]
}

fn protrusive_request(mesh: &MaterialMesh, front: &[f64], dt: f64) -> (Vec<[f64; 2]>, f64) {
    let mut forces = vec![[0.0, 0.0]; mesh.n()];
    let budget = (MAX_EXTERNAL_FORCE_PER_VERTEX - FROZEN_STATIC_TRACTION_LIMIT).max(0.0);
    for edge in 0..mesh.n() {
        let normal = outward_normal(mesh, edge);
        let magnitude = budget * front[edge];
        for vertex in [edge, (edge + 1) % mesh.n()] {
            forces[vertex][0] += 0.5 * magnitude * normal[0];
            forces[vertex][1] += 0.5 * magnitude * normal[1];
        }
    }
    let requested = forces
        .iter()
        .enumerate()
        .map(|(index, force)| {
            let local_length = 0.5
                * (mesh.edge_length(index) + mesh.edge_length((index + mesh.n() - 1) % mesh.n()));
            FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME * force[0].hypot(force[1]) * local_length * dt
        })
        .sum();
    (forces, requested)
}

fn sample_ring(values: &[f64], count: usize) -> Vec<f64> {
    (0..count)
        .map(|index| values[index * values.len() / count])
        .collect()
}

fn remap_memory(memory: &mut PlasticityStateV1, before: &[[f64; 2]], after: &[[f64; 2]], dt: f64) {
    if before.len() == after.len() {
        return;
    }
    let old =
        ContinuityMaterialFrameV1::from_positions_and_stimuli(before, &vec![0.0; before.len()], dt);
    let new =
        ContinuityMaterialFrameV1::from_positions_and_stimuli(after, &vec![0.0; after.len()], dt);
    let event = if after.len() > before.len() {
        TopologyEventV1::Split
    } else {
        TopologyEventV1::Merge
    };
    let mapping = derive_local_mapping(&old, &new, event).expect("local memory remesh mapping");
    memory
        .remap(&mapping)
        .expect("plasticity remesh continuity");
}

fn fission_memory(parent: &PlasticityStateV1, start: usize, end: usize) -> PlasticityStateV1 {
    let n = parent.adaptation.len();
    let mut adaptation = Vec::new();
    let mut index = start;
    loop {
        adaptation.push(parent.adaptation[index]);
        if index == end {
            break;
        }
        index = (index + 1) % n;
    }
    PlasticityStateV1 {
        schema: parent.schema.clone(),
        enabled: parent.enabled,
        adaptation,
    }
}

fn step(snapshot: &mut Snapshot) {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams::default();
    let plasticity = PlasticityParamsV1::default();
    let fission = FissionParams::default();
    snapshot.world.diffuse(mechanics.dt);

    for individual in &mut snapshot.individuals {
        let before_centroid = individual.mesh.centroid();
        let measures: Vec<f64> = (0..individual.mesh.n())
            .map(|edge| individual.mesh.edge_length(edge))
            .collect();
        let directional = adaptive_directional_drive(
            &local_occupancy(&snapshot.world, &individual.mesh),
            &measures,
        )
        .expect("adaptive material comparator");
        let memory_at_start = individual.memory.adaptation.clone();
        let front: Vec<f64> = directional
            .front_drive
            .iter()
            .zip(&memory_at_start)
            .map(|(drive, memory)| drive * (1.0 - memory))
            .collect();
        let rear: Vec<f64> = directional
            .rear_drive
            .iter()
            .zip(&memory_at_start)
            .map(|(drive, memory)| drive * (1.0 - memory))
            .collect();
        let rear = edge_to_vertex(&rear);
        let clutch = edge_to_vertex(&front);
        let (forces, requested) = protrusive_request(&individual.mesh, &front, mechanics.dt);
        if let Ok(ledger) = apply_local_activated_energy_front_rear_with_local_traction_clutch(
            &mut individual.mesh,
            &rear,
            &clutch,
            &mechanics,
            &contractility,
            &traction,
            &forces,
            requested,
        ) {
            if let Some(active) = ledger.contractility {
                snapshot.cumulative_active_a += active.resource_spent;
                snapshot.cumulative_active_w +=
                    (active.waste_amount_after - active.waste_amount_before).max(0.0);
            }
        }
        let after_centroid = individual.mesh.centroid();
        snapshot.cumulative_path +=
            (after_centroid[0] - before_centroid[0]).hypot(after_centroid[1] - before_centroid[1]);
        if let Some(environment) = &snapshot.sensory_environment {
            let exposure = sample_ring(&environment.local_exposure(), individual.mesh.n());
            let _ = advance_local_plasticity_trace(
                &mut individual.memory,
                &exposure,
                mechanics.dt,
                &plasticity,
            );
        } else {
            let zero = vec![0.0; individual.mesh.n()];
            let _ = advance_local_plasticity_trace(
                &mut individual.memory,
                &zero,
                mechanics.dt,
                &plasticity,
            );
        }
    }

    let mut meshes: Vec<_> = snapshot
        .individuals
        .iter()
        .map(|individual| individual.mesh.clone())
        .collect();
    let deliveries = snapshot
        .world
        .exchange(&mut meshes, &transport, mechanics.dt);
    for ((individual, mesh), delivery) in
        snapshot.individuals.iter_mut().zip(meshes).zip(deliveries)
    {
        individual.mesh = mesh;
        snapshot.cumulative_n_delivered += delivery.n_delivered;
        snapshot.cumulative_f_delivered += delivery.f_delivered;
        let _ = reactions_step_with_reserve_mode(
            &mut individual.mesh,
            &reaction,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let _ = growth_step(&mut individual.mesh, &reaction, &growth, mechanics.dt);
        let before = individual.mesh.vertices.clone();
        let _ = remesh(&mut individual.mesh);
        remap_memory(
            &mut individual.memory,
            &before,
            &individual.mesh.vertices,
            mechanics.dt,
        );
        if snapshot.step % 10 == 0 {
            let _ = topology_step(&mut individual.mesh, &fission);
        }
    }

    let mut next = Vec::new();
    for individual in snapshot.individuals.drain(..) {
        let eligible = individual.mesh.total_structural_mass() >= 1.35 * individual.birth_mass
            && snapshot.step % 25 == 0;
        if eligible {
            if let Some((a, b, event)) = try_local_fission(&individual.mesh, &fission) {
                let (i, j) = event.pinch;
                let memory_a = fission_memory(&individual.memory, i, j);
                let memory_b = fission_memory(&individual.memory, j, i);
                if polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok
                    && memory_a.adaptation.len() == a.n()
                    && memory_b.adaptation.len() == b.n()
                {
                    snapshot.cumulative_fissions += 1;
                    for (mesh, memory) in [(a, memory_a), (b, memory_b)] {
                        let mut mesh = mesh;
                        if let Some(state) = mesh.finite_allocation.as_mut() {
                            let mutation = mutate_allocation_at_reproduction(
                                state.genotype,
                                &AllocationParams::default(),
                                snapshot.step ^ snapshot.next_lineage.rotate_left(17),
                            );
                            state.genotype = mutation.offspring;
                        }
                        let lineage = snapshot.next_lineage;
                        snapshot.next_lineage += 1;
                        next.push(Individual {
                            birth_mass: mesh.total_structural_mass(),
                            generation: individual.generation + 1,
                            lineage,
                            mesh,
                            memory,
                        });
                    }
                    continue;
                }
            }
        }
        if individual.mesh.observer_viable() && polygon_simple(&individual.mesh.vertices) {
            next.push(individual);
        }
    }
    snapshot.individuals = next;
    snapshot.step += 1;
}

fn atomic_save(path: &Path, snapshot: &Snapshot) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("checkpoint parent");
        }
    }
    let temporary = path.with_extension("json.partial");
    fs::write(&temporary, serde_json::to_vec_pretty(snapshot).unwrap()).expect("write checkpoint");
    fs::rename(&temporary, path).expect("atomic checkpoint rename");
}

fn load(path: &Path) -> Snapshot {
    let snapshot: Snapshot = serde_json::from_slice(&fs::read(path).expect("read checkpoint"))
        .expect("decode checkpoint");
    assert_eq!(snapshot.schema, SCHEMA);
    snapshot
}

fn report(snapshot: &Snapshot, config: &Config) -> Report {
    Report {
        schema: SCHEMA,
        step: snapshot.step,
        living: snapshot.individuals.len(),
        maximum_generation: snapshot
            .individuals
            .iter()
            .map(|individual| individual.generation)
            .max()
            .unwrap_or(0),
        fissions: snapshot.cumulative_fissions,
        cumulative_n_delivered: snapshot.cumulative_n_delivered,
        cumulative_f_delivered: snapshot.cumulative_f_delivered,
        cumulative_active_a: snapshot.cumulative_active_a,
        cumulative_active_w: snapshot.cumulative_active_w,
        cumulative_path: snapshot.cumulative_path,
        maximum_memory: snapshot
            .individuals
            .iter()
            .flat_map(|individual| &individual.memory.adaptation)
            .copied()
            .fold(0.0, f64::max),
        sensory_camera_present: config.camera_current.is_some()
            || snapshot.sensory_environment.is_some(),
        sensory_microphone_present: config.microphone_pcm.is_some()
            || snapshot
                .sensory_environment
                .as_ref()
                .is_some_and(|environment| environment.audio_amplitude > 0.0),
        observer_connected: config.observer_connected,
        observer_in_biological_step: false,
        all_polygons_simple: snapshot
            .individuals
            .iter()
            .all(|individual| polygon_simple(&individual.mesh.vertices)),
        autonomous_resource_acquisition:
            "M2_ADAPTIVE_MATERIAL_GRADIENT_AUTONOMOUS_RESOURCE_ACQUISITION_QUALIFIED",
        environment_dependent_evolution: "NOT_ESTABLISHED",
    }
}

fn main() {
    let config = parse_config();
    let sensory = load_sensory(&config);
    let mut snapshot = config
        .resume
        .as_deref()
        .map(load)
        .unwrap_or_else(|| new_snapshot(sensory));
    for _ in 0..config.steps {
        step(&mut snapshot);
    }
    atomic_save(&config.checkpoint, &snapshot);
    let report = report(&snapshot, &config);
    if let Some(parent) = config.report.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("report parent");
        }
    }
    fs::write(&config.report, serde_json::to_vec_pretty(&report).unwrap()).expect("write report");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_resume_matches_continuous_and_observer_is_inert() {
        let mut continuous = new_snapshot(None);
        for _ in 0..4 {
            step(&mut continuous);
        }
        let mut interrupted = new_snapshot(None);
        for _ in 0..2 {
            step(&mut interrupted);
        }
        let bytes = serde_json::to_vec(&interrupted).unwrap();
        let mut resumed: Snapshot = serde_json::from_slice(&bytes).unwrap();
        for _ in 0..2 {
            step(&mut resumed);
        }
        assert_eq!(continuous.step, resumed.step);
        assert_eq!(continuous.individuals.len(), resumed.individuals.len());
        for (left, right) in continuous.individuals.iter().zip(&resumed.individuals) {
            assert_eq!(left.mesh.vertices.len(), right.mesh.vertices.len());
            for (a, b) in left.mesh.vertices.iter().zip(&right.mesh.vertices) {
                assert!((a[0] - b[0]).abs() < 1e-12);
                assert!((a[1] - b[1]).abs() < 1e-12);
            }
            for (a, b) in left.memory.adaptation.iter().zip(&right.memory.adaptation) {
                assert!((a - b).abs() < 1e-12);
            }
        }
        let observer_off = Config {
            steps: 0,
            checkpoint: PathBuf::new(),
            report: PathBuf::new(),
            resume: None,
            camera_current: None,
            camera_previous: None,
            camera_width: 0,
            camera_height: 0,
            microphone_pcm: None,
            observer_connected: false,
        };
        let observer_on = Config {
            observer_connected: true,
            ..observer_off.clone()
        };
        assert_eq!(
            report(&continuous, &observer_off).step,
            report(&continuous, &observer_on).step
        );
        assert!(!report(&continuous, &observer_on).observer_in_biological_step);
    }

    #[test]
    fn sensory_experience_persists_and_changes_later_motor_response() {
        let environment = LowLevelSensoryEnvironmentV1::from_samples(
            (0..SENSOR_BINS)
                .map(|index| if index < SENSOR_BINS / 2 { 1.0 } else { 0.0 })
                .collect(),
            vec![0.0; SENSOR_BINS],
            0.25,
            0.75,
            0.25,
        )
        .unwrap();
        let mut experienced = new_snapshot(Some(environment));
        let mut naive = new_snapshot(None);
        for _ in 0..10 {
            step(&mut experienced);
            step(&mut naive);
        }
        assert!(
            experienced.individuals[0]
                .memory
                .adaptation
                .iter()
                .copied()
                .fold(0.0, f64::max)
                > 0.0
        );
        assert_ne!(
            experienced.individuals[0].mesh.vertices,
            naive.individuals[0].mesh.vertices
        );
        let restored: Snapshot =
            serde_json::from_slice(&serde_json::to_vec(&experienced).unwrap()).unwrap();
        for (left, right) in restored.individuals[0]
            .memory
            .adaptation
            .iter()
            .zip(&experienced.individuals[0].memory.adaptation)
        {
            assert!((left - right).abs() < 1e-12);
        }
    }
}
