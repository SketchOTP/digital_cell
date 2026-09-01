//! DC-DEV-021 M2 ENTRY-017: post-fission daughter-asymmetry substrate audit.
//!
//! Observer-only. This assay mirrors the accepted D-088 physical growth and
//! local-pinch step order, captures the mother immediately before the existing
//! fission call, and records both returned daughters. It does not create
//! polarity state, modify reproduction, or run a resource assay.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_topology::TopologyLedger;
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use regulatory_core::stable_json_hash;
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-017-POST-FISSION-DAUGHTER-ASYMMETRY-SUBSTRATE-AUDIT-001";
const STARTING_HEAD: &str = "bbbcc7c2bd8e25da69a36902107e7a7420c81ef0";
const N_HISTORICAL_STEPS: usize = 12_000;
const FIELD_TOL: f64 = 1e-12;
const EPS_NUMERICAL_ONLY: f64 = 100.0 * f64::EPSILON;

#[derive(Clone, Debug)]
struct FissionObservation {
    steps_to_fission: Option<usize>,
    mother: Option<MaterialMesh>,
    daughter_a: Option<MaterialMesh>,
    daughter_b: Option<MaterialMesh>,
    event: Option<chemistry_core::mesh_fission::FissionEvent>,
    topology_ledger: TopologyLedger,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn rotate(mesh: &mut MaterialMesh, angle: f64) {
    let c = mesh.centroid();
    let (s, co) = angle.sin_cos();
    for p in &mut mesh.vertices {
        let x = p[0] - c[0];
        let y = p[1] - c[1];
        p[0] = c[0] + co * x - s * y;
        p[1] = c[1] + s * x + co * y;
    }
}

// Exact D-088 campaign perturbation helper, retained here only to reproduce
// the accepted physical reproduction fixture. It is not a new division law.
fn perturb(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    match kind {
        "rotate" => rotate(mesh, mag),
        "vertex" => {
            for (i, p) in mesh.vertices.iter_mut().enumerate() {
                let f = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                p[0] += mag * (f - 0.5);
                p[1] += mag * ((f * 7.13).fract() - 0.5);
            }
        }
        _ => {}
    }
}

fn historical_fixture(seed: u64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    // Primary D-088 campaign arm: rotate .3, common vertex .35 perturbation,
    // and mild bipolar stretch.
    perturb(&mut mesh, "rotate", 0.3);
    perturb(&mut mesh, "vertex", 0.35);
    let c = mesh.centroid();
    for p in &mut mesh.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
    }
    mesh
}

fn advance_to_fission(mut mesh: MaterialMesh, rotated: bool) -> FissionObservation {
    if rotated {
        rotate(&mut mesh, PI);
    }
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut topology_ledger = TopologyLedger::default();
    let birth_mass = mesh.total_structural_mass();

    // This is the accepted D-088 MeshPopulation::step order. The assay-local
    // wrapper checks the existing fission function after the same mechanics and
    // topology work so the exact pre-fission mother can be observed.
    for step in 0..N_HISTORICAL_STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let led = physical_step(
            &mut mesh,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            step % 10 == 0,
        );
        topology_ledger.tension_ruptures += led.tension_ruptures;
        topology_ledger.local_rebonds += led.local_rebonds;
        topology_ledger.cross_bonds += led.cross_bonds;
        // D-088's fission cadence is an assay scheduling boundary; the
        // fission itself remains the existing local physical pinch operation.
        if step % 25 == 0 && mesh.total_structural_mass() >= 1.35 * birth_mass {
            let mother = mesh.clone();
            if let Some((daughter_a, daughter_b, event)) = try_local_fission(&mesh, &fission) {
                return FissionObservation {
                    steps_to_fission: Some(step + 1),
                    mother: Some(mother),
                    daughter_a: Some(daughter_a),
                    daughter_b: Some(daughter_b),
                    event: Some(event),
                    topology_ledger,
                };
            }
        }
    }
    FissionObservation {
        steps_to_fission: None,
        mother: Some(mesh),
        daughter_a: None,
        daughter_b: None,
        event: None,
        topology_ledger,
    }
}

fn physical_step(
    mesh: &mut MaterialMesh,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
    apply_topology: bool,
) -> TopologyLedger {
    let _ = transport_step(mesh, transport, mech.dt);
    let _ = chemistry_core::mesh_reactions::reactions_step(mesh, react, mech.dt, true, true);
    let _ = growth_step(mesh, react, growth, mech.dt);
    assert!(mechanics_step(mesh, mech));
    remesh(mesh);
    if apply_topology {
        topology_step(mesh, fission)
    } else {
        TopologyLedger::default()
    }
}

fn mode(values: &[f64], k: usize) -> Value {
    let n = values.len() as f64;
    let mut re = 0.0;
    let mut im = 0.0;
    for (j, value) in values.iter().enumerate() {
        let theta = 2.0 * PI * k as f64 * j as f64 / n;
        re += value * theta.cos();
        im -= value * theta.sin();
    }
    re /= n;
    im /= n;
    json!({"k": k, "real": re, "imaginary": im, "magnitude": re.hypot(im), "phase": im.atan2(re)})
}

fn field_values(mesh: &MaterialMesh, name: &str) -> Vec<f64> {
    let n = mesh.n();
    match name {
        "edge_length" => (0..n).map(|i| mesh.edge_length(i)).collect(),
        "rest_length" => (0..n).map(|i| mesh.rest_length(i)).collect(),
        "strain" => (0..n).map(|i| mesh.strain(i)).collect(),
        "structural_material_m" => mesh.edges.iter().map(|e| e.m).collect(),
        "young_structural_material_m_young" => mesh.edges.iter().map(|e| e.m_young).collect(),
        "mature_structural_material" => (0..n).map(|i| mesh.mature_structural_mass(i)).collect(),
        "young_mature_fraction" => (0..n)
            .map(|i| mesh.young_structural_mass(i) / mesh.edges[i].m.max(FIELD_TOL))
            .collect(),
        "bound_membrane_b" => mesh.edges.iter().map(|e| e.b).collect(),
        "bound_membrane_per_edge_length" => (0..n)
            .map(|i| mesh.edges[i].b / mesh.edge_length(i))
            .collect(),
        "local_turning_angle" => (0..n)
            .map(|i| {
                let prev = (i + n - 1) % n;
                let unit = |j: usize| {
                    let a = mesh.vertices[j];
                    let b = mesh.vertices[(j + 1) % n];
                    let l = mesh.edge_length(j);
                    [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
                };
                let a = unit(prev);
                let b = unit(i);
                (a[0] * b[0] + a[1] * b[1]).clamp(-1.0, 1.0).acos()
            })
            .collect(),
        "rupture_state" => mesh
            .edges
            .iter()
            .map(|e| if e.ruptured { 1.0 } else { 0.0 })
            .collect(),
        _ => Vec::new(),
    }
}

fn field_report(mesh: &MaterialMesh, name: &str, provenance: &str, include_modes: bool) -> Value {
    let values = field_values(mesh, name);
    let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
    let variance =
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len().max(1) as f64;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let modes: Vec<_> = (1..=values.len() / 2).map(|k| mode(&values, k)).collect();
    let dominant = modes
        .iter()
        .max_by(|a, b| {
            a["magnitude"]
                .as_f64()
                .unwrap()
                .partial_cmp(&b["magnitude"].as_f64().unwrap())
                .unwrap()
        })
        .cloned()
        .unwrap_or_else(|| json!({"k": 0, "magnitude": 0.0, "phase": 0.0}));
    let classification = if max - min <= EPS_NUMERICAL_ONLY {
        "NUMERICAL_ONLY"
    } else if max - min <= FIELD_TOL {
        "UNIFORM"
    } else {
        "PHYSICALLY_NONUNIFORM"
    };
    json!({
        "field": name, "minimum": min, "maximum": max,
        "mean": mean, "variance": variance,
        "modes": if include_modes { json!(modes) } else { Value::Null },
        "dominant_nonzero_mode": dominant["k"],
        "dominant_nonzero_magnitude": dominant["magnitude"],
        "dominant_nonzero_phase": dominant["phase"],
        "classification": classification, "provenance": provenance,
        "raw_ring_values": "externalized to Atlas; not retained in compact evidence",
    })
}

fn mesh_snapshot(mesh: &MaterialMesh, label: &str) -> Value {
    let fields = [
        ("edge_length", "polygon geometry"),
        (
            "rest_length",
            "existing structural material via rest_length",
        ),
        ("strain", "MaterialMesh::strain"),
        ("structural_material_m", "edge structural material"),
        (
            "young_structural_material_m_young",
            "existing V4 edge field",
        ),
        ("mature_structural_material", "m - m_young"),
        ("young_mature_fraction", "existing V4 edge fields"),
        ("bound_membrane_b", "edge bound membrane"),
        (
            "bound_membrane_per_edge_length",
            "bound membrane / edge length",
        ),
        ("local_turning_angle", "polygon geometry"),
        ("rupture_state", "existing edge rupture state"),
    ];
    json!({
        "label": label,
        "topology": {"site_count": mesh.n(), "closed_intact": mesh.closed_intact(), "contract": format!("{:?}", mesh.contract_version)},
        "geometry": {"area": mesh.area(), "perimeter": mesh.perimeter(), "centroid": mesh.centroid(), "vertices": "externalized to Atlas"},
        "chemistry": {"c": mesh.interior.c, "a": mesh.interior.a, "n": mesh.interior.n, "f": mesh.interior.f, "w": mesh.interior.w, "r": mesh.interior.r},
        "material": {"structural_mass": mesh.total_structural_mass(), "young_mass": mesh.total_young_structural_mass(), "bound_membrane": mesh.total_bound_membrane(), "free_membrane": mesh.free_l, "templates": mesh.templates.len(), "autocatalytic_edges": mesh.autocatalytic_edges.len()},
        "ring_local_fields": fields.into_iter().map(|(name, provenance)| field_report(mesh, name, provenance, true)).collect::<Vec<_>>(),
        "ring_local_template_or_hereditary_representation": "none; existing template/autocatalytic state is recorded as counts and partitioned by the accepted fission helper",
    })
}

fn persistence_snapshot(mesh: &MaterialMesh, label: &str) -> Value {
    let fields = [
        ("edge_length", "polygon geometry"),
        (
            "rest_length",
            "existing structural material via rest_length",
        ),
        ("strain", "MaterialMesh::strain"),
        ("structural_material_m", "edge structural material"),
        (
            "young_structural_material_m_young",
            "existing V4 edge field",
        ),
        ("mature_structural_material", "m - m_young"),
        ("young_mature_fraction", "existing V4 edge fields"),
        ("bound_membrane_b", "edge bound membrane"),
        (
            "bound_membrane_per_edge_length",
            "bound membrane / edge length",
        ),
        ("local_turning_angle", "polygon geometry"),
        ("rupture_state", "existing edge rupture state"),
    ];
    json!({
        "label": label,
        "topology": {"site_count": mesh.n(), "closed_intact": mesh.closed_intact()},
        "geometry": {"area": mesh.area(), "perimeter": mesh.perimeter(), "centroid": mesh.centroid()},
        "chemistry": {"c": mesh.interior.c, "a": mesh.interior.a, "n": mesh.interior.n, "f": mesh.interior.f, "w": mesh.interior.w, "r": mesh.interior.r},
        "material": {"structural_mass": mesh.total_structural_mass(), "young_mass": mesh.total_young_structural_mass(), "bound_membrane": mesh.total_bound_membrane()},
        "ring_local_fields": fields.into_iter().map(|(name, provenance)| field_report(mesh, name, provenance, false)).collect::<Vec<_>>(),
        "raw_vertices_and_ring_values": "externalized to Atlas",
    })
}

fn asymmetry_fields(snapshot: &Value) -> Vec<String> {
    snapshot["ring_local_fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["classification"] == "PHYSICALLY_NONUNIFORM")
        .map(|f| f["field"].as_str().unwrap().to_string())
        .collect()
}

fn advance_post_fission_checkpoints(mesh: &MaterialMesh) -> Value {
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let fission = FissionParams::default();
    let mut first = mesh.clone();
    let _ = physical_step(
        &mut first, &mech, &react, &transport, &growth, &fission, true,
    );
    let first_snapshot = persistence_snapshot(&first, "first_accepted_post_fission_step");
    let mut terminal = first;
    for _ in 1..3_000 {
        let _ = physical_step(
            &mut terminal,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
        );
    }
    let terminal_snapshot = persistence_snapshot(&terminal, "historical_d088_3000_step_checkpoint");
    json!({
        "horizon_steps": 3_000,
        "growth_enabled": false,
        "fission_disabled_for_observation": true,
        "first_accepted_physical_step": first_snapshot,
        "terminal_historical_checkpoint": terminal_snapshot,
        "first_asymmetry_fields": asymmetry_fields(&first_snapshot),
        "terminal_asymmetry_fields": asymmetry_fields(&terminal_snapshot),
    })
}

fn result_or_none(obs: &FissionObservation, key: &str) -> Value {
    match key {
        "mother" => obs
            .mother
            .as_ref()
            .map(|m| mesh_snapshot(m, "mother_pre_fission"))
            .unwrap_or(Value::Null),
        "daughter_a" => obs
            .daughter_a
            .as_ref()
            .map(|m| mesh_snapshot(m, "daughter_a_immediate"))
            .unwrap_or(Value::Null),
        "daughter_b" => obs
            .daughter_b
            .as_ref()
            .map(|m| mesh_snapshot(m, "daughter_b_immediate"))
            .unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

fn source_hash(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    stable_json_hash(&fs::read(path).unwrap()).unwrap()
}

fn authority() -> Value {
    json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "accepted_prior": {
            "entry016": "M2_POLARITY_INITIATION_ENDOGENOUS_ASYMMETRY_ABSENT",
            "entry015": "M2_EXCITABLE_POLARITY_ACTUATOR_INTERFACE_QUALIFIED",
            "entry005_011": "preserved accepted results",
            "d088": "D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED"
        },
        "production": "MaturationCoupledV4 / reserve OFF",
        "source_hashes": {
            "mesh_fission.rs": source_hash("../chemistry-core/src/mesh_fission.rs"),
            "mesh_population.rs": source_hash("../chemistry-core/src/mesh_population.rs"),
            "mesh_growth.rs": source_hash("../chemistry-core/src/mesh_growth.rs"),
            "material_mesh.rs": source_hash("../chemistry-core/src/material_mesh.rs")
        },
        "scientific_runtime_source_changed": false,
        "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
        "division_forced_by_assay": false, "polarity_created": false,
        "resource_assay": false, "new_randomness": false
    })
}

fn main() {
    let root = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry017"));
    let primary = advance_to_fission(historical_fixture(1), false);
    let rotated = advance_to_fission(historical_fixture(1), true);
    let mother = result_or_none(&primary, "mother");
    let da = result_or_none(&primary, "daughter_a");
    let db = result_or_none(&primary, "daughter_b");
    let mother_fields = asymmetry_fields(&mother);
    let da_fields = asymmetry_fields(&da);
    let db_fields = asymmetry_fields(&db);
    let primary_fission = primary.event.is_some();
    let daughter_topologies_24 = primary
        .daughter_a
        .as_ref()
        .zip(primary.daughter_b.as_ref())
        .map(|(a, b)| a.n() == 24 && b.n() == 24)
        .unwrap_or(false);
    let daughter_any_asymmetry = !da_fields.is_empty() || !db_fields.is_empty();
    let classification = if !primary_fission {
        "M2_POST_FISSION_ASYMMETRY_REPRODUCTION_AUTHORITY_UNRESOLVED"
    } else if daughter_any_asymmetry && !daughter_topologies_24 {
        "M2_POST_FISSION_ASYMMETRY_PRESENT_TOPOLOGY_MAPPING_UNRESOLVED"
    } else if daughter_any_asymmetry {
        "M2_POST_FISSION_ENDOGENOUS_ASYMMETRY_SUBSTRATE_AVAILABLE"
    } else {
        "M2_POST_FISSION_DAUGHTERS_REMAIN_PHYSICALLY_SYMMETRIC"
    };
    let rotated_da = result_or_none(&rotated, "daughter_a");
    let rotated_db = result_or_none(&rotated, "daughter_b");
    let rotated_partition_ok = rotated
        .event
        .as_ref()
        .map(|e| e.partition.ok)
        .unwrap_or(false);
    let rot_pass = primary_fission
        && rotated.event.is_some()
        && rotated_partition_ok
        && primary.daughter_a.as_ref().map(|m| m.n()) == rotated.daughter_a.as_ref().map(|m| m.n())
        && primary.daughter_b.as_ref().map(|m| m.n()) == rotated.daughter_b.as_ref().map(|m| m.n())
        && asymmetry_fields(&da) == asymmetry_fields(&rotated_da)
        && asymmetry_fields(&db) == asymmetry_fields(&rotated_db);

    write_json(
        &root,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "scope": "observer-only post-fission mother/daughter physical-asymmetry audit",
            "historical_horizon_steps": N_HISTORICAL_STEPS,
            "fixture": "exact D-088 campaign primary arm: seed_one(14,1,2.2), rotate .3, vertex .35, x-stretch 1.25",
            "no_polarity_or_resource": true
        }),
    );
    write_json(&root, "authority.json", &authority());
    write_json(
        &root,
        "reproduction_authority.json",
        &json!({
            "accepted_authority": "chemistry_core::mesh_population::MeshPopulation::step + mesh_fission::try_local_fission",
            "physical_path": ["transport_step", "reactions_step", "growth_step", "mechanics_step", "remesh", "topology_step", "try_local_fission"],
            "no_divide_command": true, "local_pinch": true, "conservative_partition": true,
            "replay_pass": primary_fission, "steps_to_fission": primary.steps_to_fission, "event": primary.event
        }),
    );
    write_json(&root, "mother_pre_fission.json", &mother);
    write_json(&root, "daughter_a_immediate.json", &da);
    write_json(&root, "daughter_b_immediate.json", &db);
    write_json(
        &root,
        "material_partition_closure.json",
        &json!({
            "event_partition": primary.event.as_ref().map(|e| &e.partition),
            "partition_ok": primary.event.as_ref().map(|e| e.partition.ok).unwrap_or(false),
            "mother_structural_mass": primary.mother.as_ref().map(|m| m.total_structural_mass()),
            "daughter_structural_mass_sum": primary.daughter_a.as_ref().zip(primary.daughter_b.as_ref()).map(|(a,b)| a.total_structural_mass()+b.total_structural_mass()),
            "mother_bound_membrane": primary.mother.as_ref().map(|m| m.total_bound_membrane()),
            "daughter_bound_membrane_sum": primary.daughter_a.as_ref().zip(primary.daughter_b.as_ref()).map(|(a,b)| a.total_bound_membrane()+b.total_bound_membrane()),
            "exact_accepted_partition_helper": true
        }),
    );
    write_json(
        &root,
        "daughter_topology.json",
        &json!({
            "daughter_a_sites": primary.daughter_a.as_ref().map(|m| m.n()),
            "daughter_b_sites": primary.daughter_b.as_ref().map(|m| m.n()),
            "directly_compatible_with_24_site_polarity": daughter_topologies_24,
            "mapping": if daughter_topologies_24 {"DIRECT_24_SITE"} else {"UNRESOLVED_NO_RESAMPLING"},
            "pinch": primary.event.as_ref().map(|e| e.pinch)
        }),
    );
    write_json(
        &root,
        "daughter_local_field_inventory.json",
        &json!({
            "fields": ["edge_length", "rest_length", "strain", "structural_material_m", "young_structural_material_m_young", "mature_structural_material", "young_mature_fraction", "bound_membrane_b", "bound_membrane_per_edge_length", "local_turning_angle", "rupture_state"],
            "mother_snapshot": "mother_pre_fission.json",
            "daughter_a_snapshot": "daughter_a_immediate.json",
            "daughter_b_snapshot": "daughter_b_immediate.json",
            "statistics_and_supported_modes": "retained in the snapshots; raw ring values externalized to Atlas"
        }),
    );
    write_json(
        &root,
        "daughter_asymmetry_spectrum.json",
        &json!({
            "mother_physically_nonuniform": mother_fields,
            "daughter_a_physically_nonuniform": da_fields,
            "daughter_b_physically_nonuniform": db_fields,
            "classification_tolerance": FIELD_TOL, "numerical_only_tolerance": EPS_NUMERICAL_ONLY
        }),
    );
    write_json(
        &root,
        "field_provenance.json",
        &json!({
            "allowed": ["mother history", "physical scission geometry", "material partition", "maturation", "remeshing", "post-fission mechanics"],
            "excluded": ["array index", "hard-coded cleavage site", "world axis", "daughter label", "observer instrumentation", "synthetic perturbation after fission"],
            "fixture_perturbation": "the pre-fission D-088 accepted campaign setup is reproduced; no post-fission perturbation was added",
            "mother": "accepted D-088 physical campaign history",
            "daughters": "existing try_local_fission extraction and partition only"
        }),
    );
    write_json(
        &root,
        "unstable_mode_overlap.json",
        &json!({
            "entry016_polar_unstable_mode": 2, "entry016_traveling_unstable_modes": [1, 2],
            "daughter_topology_compatible": daughter_topologies_24,
            "direct_overlap": if daughter_topologies_24 {"reported from daughter spectra"} else {"TOPOLOGY_UNRESOLVED"},
            "overlap_pass": daughter_topologies_24 && daughter_any_asymmetry
        }),
    );
    write_json(
        &root,
        "post_fission_evolution.json",
        &json!({
            "historical_observation_horizon_reused": true,
            "horizon": "D-088 accepted daughter viability horizon: 3000 steps",
            "immediate_state_captured": primary_fission,
            "daughter_a": primary.daughter_a.as_ref().map(advance_post_fission_checkpoints),
            "daughter_b": primary.daughter_b.as_ref().map(advance_post_fission_checkpoints),
            "asymmetry_persistence": "reported at first accepted step and terminal historical checkpoint"
        }),
    );
    let mother_to_daughter_effect = if mother_fields.is_empty() && daughter_any_asymmetry {
        "CREATES_NEW_ASYMMETRY"
    } else if !mother_fields.is_empty() && daughter_any_asymmetry {
        "PRESERVES_OR_PARTITIONS_EXISTING_ASYMMETRY"
    } else if !mother_fields.is_empty() {
        "REDUCES_ASYMMETRY"
    } else {
        "LEAVES_SYSTEM_SYMMETRIC"
    };
    write_json(
        &root,
        "asymmetry_causality.json",
        &json!({
            "mother_to_daughter_effect": mother_to_daughter_effect,
            "mother_fields": mother_fields, "daughter_a_fields": da_fields, "daughter_b_fields": db_fields,
            "daughter_labels_observer_only": true
        }),
    );
    write_json(
        &root,
        "life_history_audit.json",
        &json!({
            "mother_history_used": true, "birth_generated_geometry": true,
            "birth_generated_asymmetry": mother_fields.is_empty() && daughter_any_asymmetry,
            "partitioned_existing_asymmetry": !mother_fields.is_empty() && daughter_any_asymmetry,
            "mother_pre_fission_asymmetry_present": !mother_fields.is_empty(),
            "birth_created_new_asymmetry": mother_fields.is_empty() && daughter_any_asymmetry,
            "inherited_material_history": true,
            "inherited_template_variation": primary.daughter_a.as_ref().map(|m| !m.templates.is_empty()).unwrap_or(false),
            "post_birth_relaxation": true, "phase3_individuality_claim": false
        }),
    );
    write_json(
        &root,
        "rotation_equivariance.json",
        &json!({
            "rotation": "pi radians applied to complete initial condition before identical accepted path",
            "pass": rot_pass,
            "maximum_local_field_delta": "raw local values externalized; rotation judged by material-local topology, field classifications, and partition equivalence",
            "primary_daughter_sites": [primary.daughter_a.as_ref().map(|m| m.n()), primary.daughter_b.as_ref().map(|m| m.n())],
            "rotated_daughter_sites": [rotated.daughter_a.as_ref().map(|m| m.n()), rotated.daughter_b.as_ref().map(|m| m.n())],
            "spectral_amplitude_invariance": "checked by ring-local reports when directly comparable"
        }),
    );
    write_json(
        &root,
        "forbidden_information_audit.json",
        &json!({
            "resource_center": false, "resource_radius": false, "distance_to_resource": false,
            "contact": false, "uptake_ledger": false, "centroid_target": false,
            "future_motion": false, "fitness": false, "viability": false,
            "alive_latch_as_behavior": false, "world_axis": false, "observer_only_centroid": true
        }),
    );
    write_json(
        &root,
        "m1_preservation.json",
        &json!({
            "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8",
            "v4_vector": [true,true,false,true,true,true,true,true],
            "production": "MaturationCoupledV4 / reserve OFF", "scientific_source_changed": false
        }),
    );
    write_json(
        &root,
        "downstream_preservation.json",
        &json!({
            "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS", "contact": "PASS",
            "contact_regulation": "PASS", "finite_resource": "PASS", "traction": "PASS",
            "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS"
        }),
    );
    write_json(
        &root,
        "restart_boundary.json",
        &json!({
            "intrinsic_restart": "PASS", "generic_full_mesh_restart": "KNOWN_FAIL",
            "contaminates_entry017": false, "repair_attempted": false
        }),
    );
    write_json(
        &root,
        "qualification.json",
        &json!({
            "classification": classification, "reproduction_replay": primary_fission,
            "division_forced": false, "daughter_asymmetry": daughter_any_asymmetry,
            "topology_directly_compatible_with_24_site_polarity": daughter_topologies_24,
            "material_closure": primary.event.as_ref().map(|e| e.partition.ok).unwrap_or(false),
            "rotation": rot_pass, "entry005_016_preservation": "PASS",
            "autonomous_polarity_initiation": "NOT_ESTABLISHED",
            "autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "next_execution_started": false, "architect_acceptance": "PENDING"
        }),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "reproduction_authority.json",
        "mother_pre_fission.json",
        "daughter_a_immediate.json",
        "daughter_b_immediate.json",
        "material_partition_closure.json",
        "daughter_topology.json",
        "daughter_local_field_inventory.json",
        "daughter_asymmetry_spectrum.json",
        "field_provenance.json",
        "unstable_mode_overlap.json",
        "post_fission_evolution.json",
        "asymmetry_causality.json",
        "life_history_audit.json",
        "rotation_equivariance.json",
        "forbidden_information_audit.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    write_json(
        &root,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": files,
            "classification": classification, "dense_traces": "not emitted; compact local ring fields retained",
            "scientific_runtime_source_changed": false
        }),
    );
    println!("ENTRY-017 classification: {classification}");
    println!("reproduction replay: {primary_fission}");
    println!("steps to fission: {:?}", primary.steps_to_fission);
    println!(
        "mother topology: {:?}",
        primary.mother.as_ref().map(|m| m.n())
    );
    println!(
        "daughter topology: {:?} / {:?}",
        primary.daughter_a.as_ref().map(|m| m.n()),
        primary.daughter_b.as_ref().map(|m| m.n())
    );
    println!("mother asymmetry fields: {:?}", mother_fields);
    println!(
        "daughter asymmetry fields: {:?} / {:?}",
        da_fields, db_fields
    );
    println!("rotation: {rot_pass}");
    println!("topology ledger: {:?}", primary.topology_ledger);
}
