//! DC-FINAL-001-R1: adaptive material-gradient front/rear acquisition.
//!
//! The organism reads only membrane-local N/F occupancy.  A perimeter-weighted
//! comparator creates an outward, A-funded front and an A-funded contractile
//! rear.  No resource coordinate, bearing, observer gradient, reward, or
//! desired velocity enters the organism path; bearing is used only after the
//! run for observer scoring.

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX};
use chemistry_core::mesh_population::MeshPopulation;
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_self_contact::polygon_simple;
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    adaptive_directional_drive, apply_local_activated_energy_front_rear_with_local_traction_clutch,
    apply_stick_slip_to_legacy_mechanics, ContractilityParamsV1, SpatialMaterialFieldV1,
    StickSlipTractionParamsV1, FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
    FROZEN_STATIC_TRACTION_LIMIT,
};
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

const STEPS: usize = 3_000;
const INITIAL_RESOURCE_N: f64 = 3.0;
const INITIAL_RESOURCE_F: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Adaptive,
    SensorOff,
    MotorOff,
    FieldOff,
    UniformField,
    DirectionalStateShuffled,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive_sensor_motor",
            Self::SensorOff => "sensor_off",
            Self::MotorOff => "motor_off",
            Self::FieldOff => "field_off",
            Self::UniformField => "uniform_field",
            Self::DirectionalStateShuffled => "directional_state_shuffled",
        }
    }
}

fn founder(seed: u64) -> MaterialMesh {
    let mut mesh = MeshPopulation::seed_one(5.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] -= center[0];
        point[1] -= center[1];
    }
    mesh
}

fn field(mesh: &MaterialMesh, bearing: f64, arm: Arm) -> SpatialMaterialFieldV1 {
    let nx = 64usize;
    let ny = 64usize;
    let dx = 1.0;
    let origin = [-32.0, -32.0];
    let mut n = vec![0.0; nx * ny];
    let mut f = vec![0.0; nx * ny];
    if arm == Arm::UniformField {
        let n_value = INITIAL_RESOURCE_N / (nx * ny) as f64;
        let f_value = INITIAL_RESOURCE_F / (nx * ny) as f64;
        n.fill(n_value);
        f.fill(f_value);
    } else if arm != Arm::FieldOff {
        let radius = mesh
            .vertices
            .iter()
            .map(|point| point[0].hypot(point[1]))
            .fold(0.0, f64::max);
        let center = [
            (radius + 4.0) * bearing.cos(),
            (radius + 4.0) * bearing.sin(),
        ];
        let ci = ((center[0] - origin[0]) / dx).floor() as isize;
        let cj = ((center[1] - origin[1]) / dx).floor() as isize;
        for y in (cj - 1)..=(cj + 1) {
            for x in (ci - 1)..=(ci + 1) {
                let index = y as usize * nx + x as usize;
                n[index] = INITIAL_RESOURCE_N / 9.0;
                f[index] = INITIAL_RESOURCE_F / 9.0;
            }
        }
    }
    SpatialMaterialFieldV1::new(nx, ny, dx, origin, n, f, 6.0).unwrap()
}

fn local_occupancy(field: &SpatialMaterialFieldV1, mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n())
        .map(|edge| {
            let a = mesh.vertices[edge];
            let b = mesh.vertices[(edge + 1) % mesh.n()];
            let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
            let Some(cell) = field.world_to_cell(midpoint) else {
                return 0.0;
            };
            let (n, f, _) = field.concentration(cell);
            let n_fraction = if n + mesh.interior.n > 0.0 {
                n / (n + mesh.interior.n)
            } else {
                0.0
            };
            let f_fraction = if f + mesh.interior.f > 0.0 {
                f / (f + mesh.interior.f)
            } else {
                0.0
            };
            0.5 * (n_fraction + f_fraction)
        })
        .collect()
}

fn edge_to_vertex(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    (0..n)
        .map(|vertex| 0.5 * (values[vertex] + values[(vertex + n - 1) % n]))
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

fn protrusive_request(mesh: &MaterialMesh, front_edge: &[f64], dt: f64) -> (Vec<[f64; 2]>, f64) {
    let mut forces = vec![[0.0, 0.0]; mesh.n()];
    let force_budget = (MAX_EXTERNAL_FORCE_PER_VERTEX - FROZEN_STATIC_TRACTION_LIMIT).max(0.0);
    for edge in 0..mesh.n() {
        if mesh.edges[edge].ruptured {
            continue;
        }
        let normal = outward_normal(mesh, edge);
        let magnitude = force_budget * front_edge[edge];
        let next = (edge + 1) % mesh.n();
        for vertex in [edge, next] {
            forces[vertex][0] += 0.5 * magnitude * normal[0];
            forces[vertex][1] += 0.5 * magnitude * normal[1];
        }
    }
    let requested = forces
        .iter()
        .enumerate()
        .map(|(vertex, force)| {
            let local_length = 0.5
                * (mesh.edge_length(vertex) + mesh.edge_length((vertex + mesh.n() - 1) % mesh.n()));
            FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME * force[0].hypot(force[1]) * local_length * dt
        })
        .sum();
    (forces, requested)
}

fn rotate_mapping(values: &mut Vec<f64>, seed: u64) {
    let n = values.len();
    if n > 1 {
        values.rotate_right(1 + (seed as usize * 7) % (n - 1));
    }
}

fn run(seed: u64, bearing: f64, arm: Arm) -> Value {
    let mut mesh = founder(seed);
    let initial = mesh.clone();
    let birth_mass = mesh.total_structural_mass();
    let mut world = field(&mesh, bearing, arm);
    let initial_world_n = world.total_n_mass();
    let initial_world_f = world.total_f_mass();
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams::default();
    let start = mesh.centroid();
    let mut previous = start;
    let mut path = 0.0;
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut a_produced = 0.0;
    let mut active_a_spent = 0.0;
    let mut active_w = 0.0;
    let mut growth_a = 0.0;
    let mut growth_material = 0.0;
    let mut first_contact = None;
    let mut first_transfer = None;
    let mut invalid = false;
    let mut max_mass = birth_mass;
    let mut max_front = 0.0_f64;
    let mut max_rear = 0.0_f64;
    let mut max_uniform_response = 0.0_f64;
    let mut substrate_work = 0.0;
    let mut stuck = 0usize;
    let mut slips = 0usize;

    for step in 1..=STEPS {
        world.diffuse(mechanics.dt);
        let mut occupancy = local_occupancy(&world, &mesh);
        if arm == Arm::SensorOff {
            occupancy.fill(0.0);
        }
        let measures: Vec<f64> = (0..mesh.n()).map(|edge| mesh.edge_length(edge)).collect();
        if arm == Arm::UniformField {
            let total_measure: f64 = measures.iter().sum();
            let weighted_mean = occupancy
                .iter()
                .zip(&measures)
                .map(|(value, measure)| value * measure)
                .sum::<f64>()
                / total_measure;
            occupancy.fill(weighted_mean);
        }
        let directional = adaptive_directional_drive(&occupancy, &measures).unwrap();
        let mut front_edge = directional.front_drive;
        let mut rear_edge = directional.rear_drive;
        if arm == Arm::DirectionalStateShuffled {
            rotate_mapping(&mut front_edge, seed);
            rotate_mapping(&mut rear_edge, seed);
        }
        max_front = max_front.max(front_edge.iter().copied().fold(0.0, f64::max));
        max_rear = max_rear.max(rear_edge.iter().copied().fold(0.0, f64::max));
        if arm == Arm::UniformField {
            max_uniform_response = max_uniform_response.max(
                front_edge
                    .iter()
                    .chain(&rear_edge)
                    .copied()
                    .fold(0.0, f64::max),
            );
        }
        let rear_vertex = edge_to_vertex(&rear_edge);
        let clutch_vertex = edge_to_vertex(&front_edge);
        let (forces, requested) = protrusive_request(&mesh, &front_edge, mechanics.dt);
        let movement = if arm == Arm::MotorOff {
            apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).map(|ledger| {
                (
                    ledger.substrate_work,
                    ledger.stuck_contacts,
                    ledger.slipping_contacts,
                    None,
                )
            })
        } else {
            apply_local_activated_energy_front_rear_with_local_traction_clutch(
                &mut mesh,
                &rear_vertex,
                &clutch_vertex,
                &mechanics,
                &contractility,
                &traction,
                &forces,
                requested,
            )
            .map(|ledger| {
                (
                    ledger.substrate_work,
                    ledger.stuck_contacts,
                    ledger.slipping_contacts,
                    ledger.contractility,
                )
            })
        };
        let Ok((passive_work, step_stuck, step_slips, active)) = movement else {
            invalid = true;
            break;
        };
        substrate_work += passive_work;
        stuck += step_stuck;
        slips += step_slips;
        if let Some(ledger) = active {
            active_a_spent += ledger.resource_spent;
            active_w += (ledger.waste_amount_after - ledger.waste_amount_before).max(0.0);
        }
        if !polygon_simple(&mesh.vertices) {
            invalid = true;
            break;
        }

        let delivery = world.exchange(std::slice::from_mut(&mut mesh), &transport, mechanics.dt);
        let delivery = &delivery[0];
        if delivery.exposed_edges > 0 && first_contact.is_none() {
            first_contact = Some(step);
        }
        if delivery.n_delivered > 0.0 && first_transfer.is_none() {
            first_transfer = Some(step);
        }
        delivered_n += delivery.n_delivered;
        delivered_f += delivery.f_delivered;
        let reaction_ledger = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        a_produced += reaction_ledger.a_produced;
        let growth_ledger = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        growth_a += growth_ledger.a_consumed_growth;
        growth_material += growth_ledger.m_grown;
        max_mass = max_mass.max(mesh.total_structural_mass());
        let centroid = mesh.centroid();
        path += (centroid[0] - previous[0]).hypot(centroid[1] - previous[1]);
        previous = centroid;
    }

    let end = mesh.centroid();
    let displacement = [end[0] - start[0], end[1] - start[1]];
    json!({
        "seed": seed,
        "bearing_radians": bearing,
        "arm": arm.name(),
        "initial_state_equal_hash_basis": {
            "vertices": initial.vertices,
            "interior": initial.interior,
            "structural_mass": birth_mass
        },
        "zero_initial_contact": local_occupancy(&field(&initial, bearing, arm), &initial)
            .iter().all(|value| *value == 0.0),
        "first_contact": first_contact,
        "first_transfer": first_transfer,
        "delivered_n": delivered_n,
        "delivered_f": delivered_f,
        "a_produced": a_produced,
        "active_a_spent": active_a_spent,
        "active_w_generated": active_w,
        "growth_a": growth_a,
        "growth_material": growth_material,
        "birth_mass": birth_mass,
        "terminal_mass": mesh.total_structural_mass(),
        "max_mass": max_mass,
        "max_mass_over_birth": max_mass / birth_mass,
        "young_structural_mass": mesh.total_young_structural_mass(),
        "mature_structural_mass": mesh.total_structural_mass() - mesh.total_young_structural_mass(),
        "observer_death_reason": mesh.observer_death_reason(),
        "path": path,
        "net": displacement[0].hypot(displacement[1]),
        "displacement_vector": displacement,
        "displacement_toward_resource": displacement[0] * bearing.cos() + displacement[1] * bearing.sin(),
        "max_front_drive": max_front,
        "max_rear_drive": max_rear,
        "max_uniform_directional_response": max_uniform_response,
        "substrate_work": substrate_work,
        "stuck_contacts": stuck,
        "slipping_contacts": slips,
        "world_n_remaining": world.total_n_mass(),
        "world_f_remaining": world.total_f_mass(),
        "world_n_closure": (initial_world_n - world.total_n_mass() - delivered_n).abs(),
        "world_f_closure": (initial_world_f - world.total_f_mass() - delivered_f).abs(),
        "simple": polygon_simple(&mesh.vertices),
        "invalid": invalid
    })
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        0.5 * (values[middle - 1] + values[middle])
    } else {
        values[middle]
    }
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r1_acquisition.json");
    let args: Vec<String> = env::args().collect();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let bearings = [
        0.0,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    ];
    let arms = [
        Arm::Adaptive,
        Arm::SensorOff,
        Arm::MotorOff,
        Arm::FieldOff,
        Arm::UniformField,
        Arm::DirectionalStateShuffled,
    ];
    let mut rows = Vec::new();
    for seed in 1..=3 {
        for bearing in bearings {
            for arm in arms {
                rows.push(run(seed, bearing, arm));
            }
        }
    }
    let active: Vec<_> = rows
        .iter()
        .filter(|row| row["arm"] == Arm::Adaptive.name())
        .collect();
    let summary = json!({
        "directive": "DC-FINAL-001-R1-ADAPTIVE-CHEMOSENSING-FRONT-REAR-MIGRATION-AND-END-TO-END-CONTINUATION-001",
        "architecture": "perimeter-weighted material comparator -> A-funded outward front + A-funded contractile rear + passive local clutch",
        "new_free_parameters": 0,
        "seeds": 3,
        "bearings": 4,
        "arms": arms.iter().map(|arm| arm.name()).collect::<Vec<_>>(),
        "active_median_directed_displacement": median(active.iter().map(|row| row["displacement_toward_resource"].as_f64().unwrap()).collect()),
        "rows": rows
    });
    fs::write(output, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
}
