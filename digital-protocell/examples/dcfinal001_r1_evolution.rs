//! DC-FINAL-001-R1-H/I: physical heredity, mutation, selection, and reversal.
//!
//! The finite population contains matched physical twins carrying the two
//! frozen D-096 trade-off allocations. Population composition changes only by
//! physical death and geometry-valid fission. Mutation is applied only to the
//! copied allocation at a successful fission event and reads no environment or
//! observer score.

use chemistry_core::d096_allocation::{
    apply_assay_environment, expression_step, mutate_allocation_at_reproduction,
    AllocationGenotype, AllocationParams, AssayEnvironment,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde::Serialize;
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

const SHARED_DEVELOPMENT_STEPS: usize = 5_000;
const PHASE_STEPS: usize = 2_500;
const PROCESSING: AllocationGenotype = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
const REPAIR: AllocationGenotype = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);

#[derive(Clone)]
struct Agent {
    mesh: MaterialMesh,
    lineage: u64,
    generation: u32,
    birth_mass: f64,
    clade: &'static str,
}

#[derive(Default, Serialize)]
struct CampaignLedger {
    physical_fissions: usize,
    simple_fissions: usize,
    deaths: usize,
    mutations: usize,
    partition_failures: usize,
    mutation_events: Vec<Value>,
    fission_events: Vec<Value>,
}

fn perturb(mesh: &mut MaterialMesh, seed: u64) {
    let kind: (&str, f64) = match seed {
        1 => ("rotate", 0.3),
        3 => ("c", 0.08),
        _ => ("a", 0.08),
    };
    match kind.0 {
        "rotate" => {
            let center = mesh.centroid();
            let (sine, cosine) = kind.1.sin_cos();
            for point in &mut mesh.vertices {
                let x = point[0] - center[0];
                let y = point[1] - center[1];
                point[0] = center[0] + cosine * x - sine * y;
                point[1] = center[1] + sine * x + cosine * y;
            }
        }
        "c" => mesh.interior.c *= 1.0 + kind.1,
        "a" => mesh.interior.a *= 1.0 + kind.1,
        _ => unreachable!(),
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

fn founder(seed: u64) -> (MaterialMesh, f64) {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb(&mut mesh, seed);
    let birth_mass = mesh.total_structural_mass();
    (mesh, birth_mass)
}

#[allow(clippy::too_many_arguments)]
fn physical_step(
    mesh: &mut MaterialMesh,
    step: usize,
    environment: Option<AssayEnvironment>,
    allocation: &AllocationParams,
    mechanics: &MechParams,
    reaction: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
) -> bool {
    if let Some(environment) = environment {
        let _ = apply_assay_environment(mesh, environment, step as u64);
        if expression_step(mesh, allocation, mechanics.dt).is_err() {
            return false;
        }
    }
    let _ = transport_step(mesh, transport, mechanics.dt);
    let _ = reactions_step(mesh, reaction, mechanics.dt, true, true);
    let _ = growth_step(mesh, reaction, growth, mechanics.dt);
    if mechanics_step_with_local_self_contact(mesh, mechanics).is_none() {
        return false;
    }
    let _ = remesh(mesh);
    if step % 10 == 0 {
        let _ = topology_step(mesh, fission);
    }
    polygon_simple(&mesh.vertices) && mesh.observer_viable()
}

fn shared_founders() -> Vec<(MaterialMesh, f64)> {
    let handles: Vec<_> = [1_u64, 3, 4]
        .into_iter()
        .map(|seed| {
            std::thread::spawn(move || {
            let allocation = AllocationParams::default();
            let mechanics = MechParams::default();
            let reaction = ReactionParams::default();
            let transport = TransportParams::default();
            let growth = GrowthParams { y_g: 0.9, enable_growth: true };
            let fission = FissionParams::default();
            let (mut mesh, birth_mass) = founder(seed);
            for step in 0..SHARED_DEVELOPMENT_STEPS {
                if !physical_step(
                    &mut mesh,
                    step,
                    None,
                    &allocation,
                    &mechanics,
                    &reaction,
                    &transport,
                    &growth,
                    &fission,
                ) {
                    break;
                }
            }
            (mesh, birth_mass)
            })
        })
        .collect()
    ;
    handles.into_iter().map(|handle| handle.join().unwrap()).collect()
}

fn summarize(agents: &[Agent], step: usize) -> Value {
    let processing = agents.iter().filter(|agent| agent.clade == "processing").count();
    let repair = agents.iter().filter(|agent| agent.clade == "repair").count();
    let count = agents.len().max(1) as f64;
    let mean_genotype = (0..4)
        .map(|index| {
            agents
                .iter()
                .filter_map(|agent| agent.mesh.finite_allocation)
                .map(|state| state.genotype.0[index])
                .sum::<f64>()
                / count
        })
        .collect::<Vec<_>>();
    json!({
        "step": step,
        "living": agents.len(),
        "processing_clade": processing,
        "repair_clade": repair,
        "processing_frequency": processing as f64 / count,
        "repair_frequency": repair as f64 / count,
        "maximum_generation": agents.iter().map(|agent| agent.generation).max().unwrap_or(0),
        "mean_genotype": mean_genotype,
        "all_simple": agents.iter().all(|agent| polygon_simple(&agent.mesh.vertices))
    })
}

fn run_phase(
    agents: &mut Vec<Agent>,
    environment: AssayEnvironment,
    start_step: usize,
    mutation: bool,
    next_lineage: &mut u64,
    ledger: &mut CampaignLedger,
) {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    for offset in 0..PHASE_STEPS {
        let step = start_step + offset;
        let mut survivors = Vec::new();
        for mut agent in agents.drain(..) {
            if !physical_step(
                &mut agent.mesh,
                step,
                Some(environment),
                &allocation,
                &mechanics,
                &reaction,
                &transport,
                &growth,
                &fission,
            ) {
                ledger.deaths += 1;
                continue;
            }
            let eligible = agent.mesh.total_structural_mass() >= 1.35 * agent.birth_mass
                && step % 25 == 0;
            if eligible {
                if let Some((mut a, mut b, event)) = try_local_fission(&agent.mesh, &fission) {
                    ledger.physical_fissions += 1;
                    let simple = polygon_simple(&agent.mesh.vertices)
                        && polygon_simple(&a.vertices)
                        && polygon_simple(&b.vertices);
                    ledger.simple_fissions += usize::from(simple);
                    ledger.partition_failures += usize::from(!event.partition.ok);
                    let mut child_rows = Vec::new();
                    for (child_index, child) in [&mut a, &mut b].into_iter().enumerate() {
                        let parent = child.finite_allocation.unwrap().genotype;
                        let seed = (step as u64)
                            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                            ^ agent.lineage.rotate_left(17)
                            ^ child_index as u64;
                        let mutation_event = if mutation {
                            mutate_allocation_at_reproduction(parent, &allocation, seed)
                        } else {
                            mutate_allocation_at_reproduction(parent, &AllocationParams { mutation_probability: 0.0, ..allocation }, seed)
                        };
                        child.finite_allocation.as_mut().unwrap().genotype = mutation_event.offspring;
                        ledger.mutations += usize::from(mutation_event.mutated);
                        child_rows.push(json!({"child":child_index,"mutation":mutation_event}));
                    }
                    let generation = agent.generation + 1;
                    ledger.fission_events.push(json!({
                        "step":step + 1,"parent":agent.lineage,"clade":agent.clade,
                        "generation":generation,"simple":simple,"partition_ok":event.partition.ok,
                        "children":child_rows
                    }));
                    for child in [a, b] {
                        let lineage = *next_lineage;
                        *next_lineage += 1;
                        survivors.push(Agent {
                            birth_mass: child.total_structural_mass(),
                            mesh: child,
                            lineage,
                            generation,
                            clade: agent.clade,
                        });
                    }
                    continue;
                }
            }
            survivors.push(agent);
        }
        *agents = survivors;
        if agents.is_empty() {
            break;
        }
    }
}

fn campaign(shared: &[(MaterialMesh, f64)], mutation: bool) -> Value {
    let allocation = AllocationParams::default();
    let mut agents = Vec::new();
    let mut next_lineage = 1_u64;
    for (mesh, birth_mass) in shared {
        for (genotype, clade) in [(PROCESSING, "processing"), (REPAIR, "repair")] {
            let mut twin = mesh.clone();
            twin.enable_finite_allocation(genotype, &allocation);
            agents.push(Agent {
                mesh: twin,
                lineage: next_lineage,
                generation: 0,
                birth_mass: *birth_mass,
                clade,
            });
            next_lineage += 1;
        }
    }
    let initial = summarize(&agents, SHARED_DEVELOPMENT_STEPS);
    let mut ledger = CampaignLedger::default();
    run_phase(
        &mut agents,
        AssayEnvironment::H,
        SHARED_DEVELOPMENT_STEPS,
        mutation,
        &mut next_lineage,
        &mut ledger,
    );
    let after_h = summarize(&agents, SHARED_DEVELOPMENT_STEPS + PHASE_STEPS);
    let h_fissions = ledger.physical_fissions;
    run_phase(
        &mut agents,
        AssayEnvironment::B,
        SHARED_DEVELOPMENT_STEPS + PHASE_STEPS,
        mutation,
        &mut next_lineage,
        &mut ledger,
    );
    let after_b = summarize(&agents, SHARED_DEVELOPMENT_STEPS + 2 * PHASE_STEPS);
    json!({
        "mutation_enabled": mutation,
        "population_is_finite": true,
        "population_cap": null,
        "fitness_function": null,
        "reproductive_quota": null,
        "initial": initial,
        "after_h": after_h,
        "after_b_switch": after_b,
        "h_phase_fissions": h_fissions,
        "b_phase_fissions": ledger.physical_fissions - h_fissions,
        "ledger": ledger
    })
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r1_evolution.json");
    let args: Vec<String> = env::args().collect();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let shared = shared_founders();
    let on_shared = shared.clone();
    let off_shared = shared.clone();
    let mutation_on = std::thread::spawn(move || campaign(&on_shared, true));
    let mutation_off = std::thread::spawn(move || campaign(&off_shared, false));
    let mutation_on = mutation_on.join().unwrap();
    let mutation_off = mutation_off.join().unwrap();
    let value = json!({
        "directive": "DC-FINAL-001-R1-ADAPTIVE-CHEMOSENSING-FRONT-REAR-MIGRATION-AND-END-TO-END-CONTINUATION-001",
        "work_packages": ["R1-H_PHYSICAL_HEREDITY_AND_MUTATION", "R1-I_ENVIRONMENT_DEPENDENT_NATURAL_SELECTION"],
        "founder_twins": 3,
        "initial_population": 6,
        "phase_steps": PHASE_STEPS,
        "environment_sequence": ["H", "B"],
        "mutation_on": mutation_on,
        "mutation_off": mutation_off
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
