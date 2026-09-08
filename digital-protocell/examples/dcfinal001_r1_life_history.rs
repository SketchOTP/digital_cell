//! DC-FINAL-001-R1-G: organism-owned life-history state through physical fission.
//!
//! Both arms share the accepted WP1 seed-1 trajectory for 5,000 steps, then
//! experience one complete frozen D-096 environment cycle before returning to
//! neutral conditions. No history label is stored in either organism.

use chemistry_core::d096_allocation::{
    apply_assay_environment, expression_step, AllocationGenotype, AllocationParams,
    AssayEnvironment,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

const SHARED_DEVELOPMENT_STEPS: usize = 5_000;
const HISTORY_STEPS: usize = 400;
const COMMON_CHECKPOINT: usize = 200;
const TOTAL_STEPS: usize = 12_000;
const DAUGHTER_COMMON_STEPS: usize = 500;

fn perturb(mesh: &mut MaterialMesh) {
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

fn founder() -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb(&mut mesh);
    mesh
}

fn snapshot(mesh: &MaterialMesh, step: usize) -> Value {
    let allocation = mesh.finite_allocation;
    json!({
        "step": step,
        "mass": mesh.total_structural_mass(),
        "young_mass": mesh.total_young_structural_mass(),
        "area": mesh.area(),
        "a_amount": mesh.interior.a * mesh.area(),
        "reserve_amount": mesh.interior.r * mesh.area(),
        "catalysts": allocation.map(|state| state.catalysts),
        "genotype": allocation.map(|state| state.genotype.0),
        "simple": polygon_simple(&mesh.vertices),
        "alive": mesh.alive,
        "vertices": mesh.n()
    })
}

fn state_distance(a: &Value, b: &Value) -> f64 {
    let mut total = ["mass", "young_mass", "area", "a_amount", "reserve_amount"]
        .iter()
        .map(|key| (a[*key].as_f64().unwrap() - b[*key].as_f64().unwrap()).abs())
        .sum::<f64>();
    if let (Some(left), Some(right)) = (a["catalysts"].as_array(), b["catalysts"].as_array()) {
        total += left
            .iter()
            .zip(right)
            .map(|(x, y)| (x.as_f64().unwrap() - y.as_f64().unwrap()).abs())
            .sum::<f64>();
    }
    total
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
    polygon_simple(&mesh.vertices)
}

#[allow(clippy::too_many_arguments)]
fn daughter_recovery(
    mut mesh: MaterialMesh,
    start_step: usize,
    allocation: &AllocationParams,
    mechanics: &MechParams,
    reaction: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
) -> Value {
    let initial = snapshot(&mesh, start_step);
    let mut all_simple = polygon_simple(&mesh.vertices);
    for offset in 0..DAUGHTER_COMMON_STEPS {
        all_simple &= physical_step(
            &mut mesh,
            start_step + offset,
            Some(AssayEnvironment::Neutral),
            allocation,
            mechanics,
            reaction,
            transport,
            growth,
            fission,
        );
        if !all_simple {
            break;
        }
    }
    json!({
        "initial": initial,
        "terminal": snapshot(&mesh, start_step + DAUGHTER_COMMON_STEPS),
        "all_states_simple": all_simple
    })
}

fn run(shared: &MaterialMesh, history: AssayEnvironment, label: &str) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let mut mesh = shared.clone();
    mesh.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let birth_mass = mesh.total_structural_mass();
    let initial = snapshot(&mesh, SHARED_DEVELOPMENT_STEPS);
    let mut common_start = Value::Null;
    let mut common_checkpoint = Value::Null;
    let mut fission_result = Value::Null;
    let mut all_simple = polygon_simple(&mesh.vertices);
    for step in SHARED_DEVELOPMENT_STEPS..TOTAL_STEPS {
        let environment = if step < SHARED_DEVELOPMENT_STEPS + HISTORY_STEPS {
            history
        } else {
            AssayEnvironment::Neutral
        };
        all_simple &= physical_step(
            &mut mesh,
            step,
            Some(environment),
            &allocation,
            &mechanics,
            &reaction,
            &transport,
            &growth,
            &fission,
        );
        if !all_simple {
            break;
        }
        if step + 1 == SHARED_DEVELOPMENT_STEPS + HISTORY_STEPS {
            common_start = snapshot(&mesh, step + 1);
        }
        if step + 1 == SHARED_DEVELOPMENT_STEPS + HISTORY_STEPS + COMMON_CHECKPOINT {
            common_checkpoint = snapshot(&mesh, step + 1);
        }
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            if let Some((daughter_a, daughter_b, event)) = try_local_fission(&mesh, &fission) {
                let parent_state = mesh.finite_allocation.unwrap();
                let a_state = daughter_a.finite_allocation.unwrap();
                let b_state = daughter_b.finite_allocation.unwrap();
                fission_result = json!({
                    "step": step + 1,
                    "parent": snapshot(&mesh, step + 1),
                    "daughter_a": daughter_recovery(daughter_a.clone(), step + 1, &allocation, &mechanics, &reaction, &transport, &growth, &fission),
                    "daughter_b": daughter_recovery(daughter_b.clone(), step + 1, &allocation, &mechanics, &reaction, &transport, &growth, &fission),
                    "parent_simple": polygon_simple(&mesh.vertices),
                    "daughter_a_simple": polygon_simple(&daughter_a.vertices),
                    "daughter_b_simple": polygon_simple(&daughter_b.vertices),
                    "partition_ok": event.partition.ok,
                    "genotype_inherited": a_state.genotype == parent_state.genotype && b_state.genotype == parent_state.genotype,
                    "catalyst_partition_residual": (parent_state.catalysts.iter().sum::<f64>() - a_state.catalysts.iter().sum::<f64>() - b_state.catalysts.iter().sum::<f64>()).abs()
                });
                break;
            }
        }
    }
    json!({
        "history": label,
        "initial": initial,
        "common_start": common_start,
        "common_checkpoint": common_checkpoint,
        "fission": fission_result,
        "all_parent_states_simple": all_simple,
        "terminal": snapshot(&mesh, TOTAL_STEPS)
    })
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r1_life_history.json");
    let args: Vec<String> = env::args().collect();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let mut shared = founder();
    let mut shared_simple = polygon_simple(&shared.vertices);
    for step in 0..SHARED_DEVELOPMENT_STEPS {
        shared_simple &= physical_step(
            &mut shared,
            step,
            None,
            &allocation,
            &mechanics,
            &reaction,
            &transport,
            &growth,
            &fission,
        );
    }
    let pulse_history = run(&shared, AssayEnvironment::H, "finite_nutrient_pulse_history");
    let damage_history = run(&shared, AssayEnvironment::B, "recurrent_local_damage_history");
    let value = json!({
        "directive": "DC-FINAL-001-R1-ADAPTIVE-CHEMOSENSING-FRONT-REAR-MIGRATION-AND-END-TO-END-CONTINUATION-001",
        "work_package": "R1-G_LIFE_HISTORY_DEVELOPMENT",
        "shared_development_steps": SHARED_DEVELOPMENT_STEPS,
        "history_steps": HISTORY_STEPS,
        "history_horizon_authority": "one complete frozen D-096 H-pulse cycle",
        "shared_development_simple": shared_simple,
        "initial_state_equivalent": pulse_history["initial"] == damage_history["initial"],
        "common_environment": "AssayEnvironment::Neutral; organism reads only resulting N/F and physical damage",
        "common_start_state_distance": state_distance(&pulse_history["common_start"], &damage_history["common_start"]),
        "common_checkpoint_state_distance": state_distance(&pulse_history["common_checkpoint"], &damage_history["common_checkpoint"]),
        "pulse_history": pulse_history,
        "damage_history": damage_history
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
