//! DC-FINAL-001 work package 1: geometry-valid reproduction qualification.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{
    find_local_segment_apposition, topology_step, try_local_fission, try_local_segment_fission,
    FissionParams,
};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::planar_ring_topology::{remesh_preserving_simple, PlanarRingTopology};
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

const STEPS: usize = 12_000;
const DAUGHTER_STEPS: usize = 3_000;

fn perturb(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    match kind {
        "rotate" => {
            let c = mesh.centroid();
            let (s, co) = mag.sin_cos();
            for p in &mut mesh.vertices {
                let (x, y) = (p[0] - c[0], p[1] - c[1]);
                p[0] = c[0] + co * x - s * y;
                p[1] = c[1] + s * x + co * y;
            }
        }
        "vertex" => {
            for (i, p) in mesh.vertices.iter_mut().enumerate() {
                let f = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                p[0] += mag * (f - 0.5);
                p[1] += mag * ((f * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + mag)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + mag)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + mag)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + mag)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + mag)).max(0.0);
        }
        _ => {}
    }
}

fn fixture(seed: u64, kind: &str, mag: f64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb(&mut mesh, kind, mag);
    perturb(&mut mesh, "vertex", 0.35);
    let c = mesh.centroid();
    for p in &mut mesh.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
    }
    mesh
}

fn geometry(mesh: &MaterialMesh) -> Value {
    json!({
        "simple": polygon_simple(&mesh.vertices),
        "vertices": mesh.n(),
        "area": mesh.area(),
        "perimeter": mesh.perimeter(),
        "mass": mesh.total_structural_mass()
    })
}

fn daughter_viability(
    mut mesh: MaterialMesh,
    mech: &MechParams,
    reaction: &ReactionParams,
    transport: &TransportParams,
    fission: &FissionParams,
) -> Value {
    let c0 = mesh.interior.c;
    let a0 = mesh.interior.a;
    let growth_off = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let mut all_simple = polygon_simple(&mesh.vertices);
    for _ in 0..DAUGHTER_STEPS {
        if !mesh.alive || !mesh.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut mesh, transport, mech.dt);
        let _ = reactions_step(&mut mesh, reaction, mech.dt, true, true);
        let _ = growth_step(&mut mesh, reaction, &growth_off, mech.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, mech).is_none() {
            all_simple = false;
            break;
        }
        let _ = remesh_preserving_simple(&mut mesh);
        let _ = topology_step(&mut mesh, fission);
        all_simple &= polygon_simple(&mesh.vertices);
    }
    let c_retention = if c0 > 1e-12 {
        mesh.interior.c / c0
    } else {
        1.0
    };
    let a_retention = if a0 > 1e-12 {
        mesh.interior.a / a0
    } else {
        1.0
    };
    let viable = mesh.alive
        && mesh.closed_intact()
        && all_simple
        && c_retention >= 0.80
        && a_retention >= 0.80;
    json!({
        "viable": viable,
        "alive": mesh.alive,
        "closed_intact": mesh.closed_intact(),
        "all_states_simple": all_simple,
        "c_retention": c_retention,
        "a_retention": a_retention,
        "terminal": geometry(&mesh)
    })
}

fn run(mesh: MaterialMesh, name: &str, allow_segment: bool, half_edge: bool) -> Value {
    let mech = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth_mass = mesh.total_structural_mass();
    let mut mesh = mesh;
    let mut contact_steps = 0usize;
    let mut projected_vertices = 0usize;
    let mut tangent_retained = 0.0;
    let mut vertex_attempts = 0usize;
    let mut segment_attempts = 0usize;
    let mut result = None;
    let mut first_apposition = None;
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut first_invalid = None;
    let mut max_ratio: f64 = 1.0;
    for s in 0..STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mech.dt);
        let _ = reactions_step(&mut mesh, &reaction, mech.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mech.dt);
        let Some(contact) = mechanics_step_with_local_self_contact(&mut mesh, &mech) else {
            all_simple = false;
            first_invalid = Some(
                json!({"step":s+1,"phase":"contact_step_rejected","geometry":geometry(&mesh)}),
            );
            break;
        };
        if contact.active_pairs > 0 {
            contact_steps += 1;
        }
        projected_vertices += contact.projected_vertices;
        tangent_retained += contact.tangential_displacement_retained.abs();
        if half_edge {
            let _ = remesh_preserving_simple(&mut mesh);
        } else {
            let _ = remesh(&mut mesh);
        }
        if !polygon_simple(&mesh.vertices) && first_invalid.is_none() {
            first_invalid =
                Some(json!({"step":s+1,"phase":"post_remesh","geometry":geometry(&mesh)}));
        }
        let planar = if half_edge {
            PlanarRingTopology::from_mesh(&mesh)
        } else {
            None
        };
        if half_edge && planar.is_none() {
            all_simple = false;
            first_invalid =
                Some(json!({"step":s+1,"phase":"half_edge_rebuild","geometry":geometry(&mesh)}));
            break;
        }
        if s % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        all_simple &= polygon_simple(&mesh.vertices);
        if first_apposition.is_none() {
            if let Some((i, j, si, sj, distance)) = find_local_segment_apposition(&mesh, &fission) {
                first_apposition = Some(
                    json!({"step":s+1,"i":i,"j":j,"si":si,"sj":sj,"distance":distance,"mass_eligible":mesh.total_structural_mass() >= 1.35*birth_mass}),
                );
            }
        }
        max_ratio = max_ratio.max(mesh.total_structural_mass() / birth_mass.max(1e-300));
        if mesh.total_structural_mass() >= 1.35 * birth_mass && s % 25 == 0 {
            vertex_attempts += 1;
            let vertex_split = try_local_fission(&mesh, &fission).and_then(|(a, b, event)| {
                (polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok)
                    .then_some((a, b, event))
            });
            let split = vertex_split.or_else(|| {
                if allow_segment {
                    segment_attempts += 1;
                    if let Some(topology) = planar.as_ref() {
                        topology.try_local_scission(&mesh, &fission)
                    } else {
                        try_local_segment_fission(&mesh, &fission)
                    }
                } else {
                    None
                }
            });
            if let Some((a, b, event)) = split {
                let valid = polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok;
                let viability_a =
                    daughter_viability(a.clone(), &mech, &reaction, &transport, &fission);
                let viability_b =
                    daughter_viability(b.clone(), &mech, &reaction, &transport, &fission);
                result = Some(json!({
                    "step":s+1,
                    "valid":valid,
                    "parent":geometry(&mesh),
                    "daughter_a":geometry(&a),
                    "daughter_b":geometry(&b),
                    "daughter_a_viability": viability_a,
                    "daughter_b_viability": viability_b,
                    "both_daughters_viable": viability_a["viable"] == true && viability_b["viable"] == true,
                    "partition":event.partition,
                    "pinch":event.pinch
                }));
                break;
            }
        }
    }
    json!({
        "name":name,
        "segment_apposition_enabled":allow_segment,
        "half_edge_topology":half_edge,
        "all_parent_states_simple":all_simple,
        "max_mass_over_birth":max_ratio,
        "growth_qualified":max_ratio >= 1.35,
        "contact_steps":contact_steps,
        "projected_vertices":projected_vertices,
        "tangential_motion_retained":tangent_retained,
        "vertex_attempts":vertex_attempts,
        "segment_attempts":segment_attempts,
        "physical_fission":result.is_some(),
        "first_segment_apposition":first_apposition,
        "first_invalid":first_invalid,
        "fission":result,
        "final":geometry(&mesh)
    })
}

fn main() {
    let mut out = PathBuf::from("/tmp/dcfinal001_reproduction.json");
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            out = PathBuf::from(&args[i + 1]);
        }
    }
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
    let take = if env::var("DC_FINAL_ONE").ok().as_deref() == Some("1") {
        1
    } else {
        kinds.len()
    };
    let only_half_edge = env::var("DC_FINAL_ONLY_HALF_EDGE").ok().as_deref() == Some("1");
    let vertex_only: Vec<_> = if only_half_edge {
        Vec::new()
    } else {
        kinds
            .iter()
            .take(take)
            .enumerate()
            .map(|(i, (k, m))| {
                run(
                    fixture((i + 1) as u64, k, *m),
                    &format!("seed_{}_{}_{}", i + 1, k, m),
                    false,
                    false,
                )
            })
            .collect()
    };
    let vertex_fissions = vertex_only
        .iter()
        .filter(|v| v["physical_fission"] == true)
        .count();
    let segment: Vec<_> = if only_half_edge || vertex_fissions >= 7 {
        Vec::new()
    } else {
        kinds
            .iter()
            .take(take)
            .enumerate()
            .map(|(i, (k, m))| {
                run(
                    fixture((i + 1) as u64, k, *m),
                    &format!("seed_{}_{}_{}", i + 1, k, m),
                    true,
                    false,
                )
            })
            .collect()
    };
    let segment_fissions = segment
        .iter()
        .filter(|v| v["physical_fission"] == true)
        .count();
    let half_edge: Vec<_> = if vertex_fissions >= 7 || segment_fissions >= 7 {
        Vec::new()
    } else {
        kinds
            .iter()
            .take(take)
            .enumerate()
            .map(|(i, (k, m))| {
                run(
                    fixture((i + 1) as u64, k, *m),
                    &format!("seed_{}_{}_{}", i + 1, k, m),
                    true,
                    true,
                )
            })
            .collect()
    };
    let value = json!({
        "directive":"DC-FINAL-001-END-TO-END-AUTONOMOUS-LIFEFORM-CLOSURE-OR-SHUTDOWN-001",
        "work_package":"WP1_GEOMETRY_VALID_REPRODUCTION",
        "vertex_only":vertex_only,
        "segment_apposition":segment,
        "half_edge_fallback":half_edge,
    });
    fs::write(out, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
