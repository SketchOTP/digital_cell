use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::{MeshPopulation, PopStepLedger};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::mesh_growth::GrowthParams;
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::material_mesh::MeshContractVersion;
use regulatory_core::{
    apply_intrinsic_exploration_refractory_motor_with_stick_slip, ContractilityParamsV1,
    FiniteWorldResourceV1, FiniteWorldV1, IntrinsicExplorationStateV1,
    StickSlipTractionParamsV1,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA: &str = "digital_cell_m2_checkpointable_lifeform_runtime_v1";
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 3.0;
const RESOURCE_BOUNDARY: f64 = 2.0639;

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
    motor_steps: u64,
    motor_failures: u64,
    #[serde(default)]
    motor_states: Vec<Option<IntrinsicExplorationStateV1>>,
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
            physical_fission: "mesh_fission::try_local_fission via MeshPopulation::step".to_string(),
            active_motility: "ENTRY-005 refractory motor integrated while topology is unchanged".to_string(),
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
    Config { steps, seed, checkpoint, report, resume }
}

fn separated_world() -> FiniteWorldV1 {
    let radius = 5.0 + RESOURCE_RADIUS + 1.0;
    let centers = [[radius, 0.0], [0.0, radius], [-radius, 0.0], [0.0, -radius]];
    let resources = centers
        .into_iter()
        .enumerate()
        .map(|(index, center)| {
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
    let mut population = MeshPopulation::seed_one(5.0, seed, 0.0);
    for individual in &mut population.individuals {
        individual.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
        individual.mesh.exterior.n = 0.0;
        individual.mesh.exterior.f = 0.0;
    }
    let motor_states = population
        .individuals
        .iter()
        .map(|individual| {
            Some(
                IntrinsicExplorationStateV1::new(individual.mesh.n(), Some(seed))
                    .expect("seeded intrinsic exploration state"),
            )
        })
        .collect();
    RuntimeSnapshot {
        schema: SCHEMA.to_string(),
        step: 0,
        seed,
        population,
        world: separated_world(),
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
        cumulative_n_world_loss: 0.0,
        cumulative_f_world_loss: 0.0,
        cumulative_fissions: 0,
        cumulative_motor_a_spent: 0.0,
        cumulative_slipping_contacts: 0,
        motor_steps: 0,
        motor_failures: 0,
        motor_states,
        scientific_boundary: ScientificBoundary::default(),
    }
}

fn load_snapshot(path: &Path) -> RuntimeSnapshot {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("cannot read snapshot {}: {error}", path.display())
    });
    let snapshot: RuntimeSnapshot = serde_json::from_str(&text).unwrap_or_else(|error| {
        panic!("cannot decode snapshot {}: {error}", path.display())
    });
    assert_eq!(snapshot.schema, SCHEMA, "unsupported runtime snapshot schema");
    snapshot
}

fn save_snapshot(path: &Path, snapshot: &RuntimeSnapshot) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                panic!("cannot create checkpoint directory {}: {error}", parent.display())
            });
        }
    }
    let encoded = serde_json::to_vec_pretty(snapshot).expect("snapshot serialization");
    fs::write(path, encoded).unwrap_or_else(|error| {
        panic!("cannot write snapshot {}: {error}", path.display())
    });
}

fn run_step(snapshot: &mut RuntimeSnapshot) -> PopStepLedger {
    let dt = MechParams::default().dt;
    let transport = TransportParams::default();

    // The accepted ENTRY-005 refractory motor is the only behavior included
    // here.  It is applied before finite-world exchange, matching the
    // existing assay causal boundary: intrinsic state -> A-funded mechanics
    // -> uptake -> frozen chemistry.  State is deliberately not reseeded
    // after fission; the continuation boundary is reported below instead of
    // being hidden by a new seed.
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    for (index, individual) in snapshot.population.individuals.iter_mut().enumerate() {
        if !individual.mesh.can_advance_physics() {
            continue;
        }
        let Some(state) = snapshot.motor_states.get_mut(index).and_then(Option::as_mut) else {
            continue;
        };
        match apply_intrinsic_exploration_refractory_motor_with_stick_slip(
            &mut individual.mesh,
            state,
            &mechanics,
            &contractility,
            &traction,
        ) {
            Ok(ledger) => {
                snapshot.motor_steps += 1;
                snapshot.cumulative_slipping_contacts += ledger.actuator.slipping_contacts;
                if let Some(contractility) = ledger.actuator.contractility {
                    snapshot.cumulative_motor_a_spent += contractility.resource_spent;
                }
            }
            Err(_) => {
                snapshot.motor_failures += 1;
                snapshot.motor_states[index] = None;
            }
        }
    }

    let mut meshes: Vec<_> = snapshot
        .population
        .individuals
        .iter()
        .map(|individual| individual.mesh.clone())
        .collect();
    let deliveries = snapshot.world.exchange(&mut meshes, &transport, dt);
    for (individual, mesh) in snapshot.population.individuals.iter_mut().zip(meshes) {
        individual.mesh = mesh;
    }
    for delivery in &deliveries {
        snapshot.cumulative_n_delivered += delivery.n_delivered;
        snapshot.cumulative_f_delivered += delivery.f_delivered;
        snapshot.cumulative_n_world_loss += delivery.n_world_loss;
        snapshot.cumulative_f_world_loss += delivery.f_world_loss;
    }

    // MeshPopulation owns the accepted reaction → growth → fission order. Its
    // transport pass sees a zero N/F exterior, so the finite world above
    // remains the sole positive N/F source. Mechanics is disabled here because
    // the accepted motor already executed the one physical mechanics step.
    let ledger = snapshot.population.step(
        &MechParams::default(),
        &ReactionParams::conservative_v3(),
        &transport,
        &GrowthParams::default(),
        &FissionParams::default(),
        false,
    );
    snapshot.cumulative_fissions += ledger.fissions;
    while snapshot.motor_states.len() < snapshot.population.individuals.len() {
        snapshot.motor_states.push(None);
    }
    if ledger.fissions > 0 {
        snapshot.scientific_boundary.active_motility =
            "ENTRY-005 integrated before fission; post-fission state continuation deferred"
                .to_string();
    }
    snapshot.step += 1;
    ledger
}

fn report(snapshot: &RuntimeSnapshot, checkpoint: &Path) -> RuntimeReport {
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
        world_n_conservation_error:
            snapshot.cumulative_n_delivered - snapshot.cumulative_n_world_loss,
        world_f_conservation_error:
            snapshot.cumulative_f_delivered - snapshot.cumulative_f_world_loss,
        motor_steps: snapshot.motor_steps,
        motor_failures: snapshot.motor_failures,
        cumulative_motor_a_spent: snapshot.cumulative_motor_a_spent,
        cumulative_slipping_contacts: snapshot.cumulative_slipping_contacts,
        active_motility: snapshot.scientific_boundary.active_motility.clone(),
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
    fs::write(&config.report, rendered).unwrap_or_else(|error| {
        panic!("cannot write report {}: {error}", config.report.display())
    });
    println!("{}", serde_json::to_string_pretty(&report(&snapshot, &config.checkpoint)).unwrap());
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
        run_step(&mut resumed);
        assert_eq!(resumed.step, 3);
        assert_eq!(resumed.seed, original.seed);
        assert_eq!(resumed.population.individuals.len(), original.population.individuals.len());
        let _ = fs::remove_file(path);
    }
}
