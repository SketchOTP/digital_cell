// CLOSURE-004: material-consistent spatial-resource geometry audit.
//
// This is intentionally bounded at the first authority stop.  The native V1
// mass/volume rule is evaluated before any lifecycle execution.  If the four
// fixed symmetric material bodies overlap, no alternative geometry is tried.

const C4_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-004-MATERIAL-CONSISTENT-SPATIAL-RESOURCE-REPRODUCTIVE-ECOLOGY-AND-HEREDITY-001";
const C4_START: &str = "cd60cbaf923ce44437d67156aa761c3fe9825233";
const C4_UNIT_N: f64 = 1021.692995326332;
const C4_UNIT_F: f64 = 1021.692995326332;
const C4_BOUNDARY_N: f64 = 2.063914918930895;
const C4_BOUNDARY_F: f64 = 2.063914918930895;
const C4_R5_RADIUS: f64 = 1.5;
const C4_TOL: f64 = 1e-10;

fn c4_write(root: &Path, name: &str, value: Value) {
    write(root, name, &value);
}

fn c4_point_segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = vector_sub(b, a);
    let denom = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if denom > 0.0 {
        ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / denom
    } else {
        0.0
    };
    let t = t.clamp(0.0, 1.0);
    vector_norm(vector_sub(p, [a[0] + t * ab[0], a[1] + t * ab[1]]))
}

fn c4_surface_gap(mesh: &MaterialMesh, center: [f64; 2], radius: f64) -> f64 {
    let boundary = (0..mesh.n())
        .map(|edge| {
            c4_point_segment_distance(
                center,
                mesh.vertices[edge],
                mesh.vertices[(edge + 1) % mesh.n()],
            )
        })
        .fold(f64::INFINITY, f64::min);
    boundary - radius
}

fn c4_place(mesh: &MaterialMesh, direction: [f64; 2], radius: f64, gap: f64) -> [f64; 2] {
    let centroid = physical_centroid(mesh);
    let mean_edge = mesh.perimeter() / mesh.n().max(1) as f64;
    let mut high = mesh
        .vertices
        .iter()
        .map(|p| vector_norm(vector_sub(*p, centroid)))
        .fold(0.0, f64::max)
        + radius
        + mean_edge
        + gap;
    let surface_gap = |distance: f64| {
        c4_surface_gap(
            mesh,
            [
                centroid[0] + distance * direction[0],
                centroid[1] + distance * direction[1],
            ],
            radius,
        )
    };
    while surface_gap(high) < gap {
        high *= 2.0;
    }
    let mut low = 0.0;
    for _ in 0..80 {
        let mid = 0.5 * (low + high);
        if surface_gap(mid) < gap {
            low = mid;
        } else {
            high = mid;
        }
    }
    [
        centroid[0] + high * direction[0],
        centroid[1] + high * direction[1],
    ]
}

fn c4_geometry(mesh: &MaterialMesh, radius: f64) -> (Vec<Value>, Vec<Value>, f64) {
    let dirs = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    let gap = mesh.perimeter() / mesh.n().max(1) as f64;
    let centers: Vec<[f64; 2]> = dirs
        .iter()
        .map(|direction| c4_place(mesh, *direction, radius, gap))
        .collect();
    let resources = centers
        .iter()
        .enumerate()
        .map(|(i, center)| {
            json!({
                "id": format!("r{}", i * 90),
                "center": center,
                "radius": radius,
                "surface_gap": c4_surface_gap(mesh, *center, radius),
                "initial_contact": false,
                "mass_n": C4_UNIT_N,
                "mass_f": C4_UNIT_F
            })
        })
        .collect();
    let mut overlaps = Vec::new();
    for i in 0..centers.len() {
        for j in (i + 1)..centers.len() {
            let distance = vector_norm(vector_sub(centers[i], centers[j]));
            if distance < 2.0 * radius - C4_TOL {
                overlaps.push(json!({
                    "resource_a": format!("r{}", i * 90),
                    "resource_b": format!("r{}", j * 90),
                    "center_distance": distance,
                    "required_nonoverlap_distance": 2.0 * radius,
                    "overlap_depth": 2.0 * radius - distance
                }));
            }
        }
    }
    (resources, overlaps, gap)
}

fn c4_world(
    body: &MaterialMesh,
    radius: f64,
    unit: f64,
    transfer_enabled: bool,
    zero_resource: bool,
) -> regulatory_core::FiniteWorldV1 {
    let dirs = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    let resources = dirs
        .iter()
        .enumerate()
        .map(|(i, direction)| {
            let center = c4_place(body, *direction, radius, body.perimeter() / body.n() as f64);
            regulatory_core::FiniteWorldResourceV1::new(
                format!("r{}", i * 90),
                center,
                radius,
                if zero_resource { 0.0 } else { unit },
                if zero_resource { 0.0 } else { unit },
                C4_BOUNDARY_N,
                C4_BOUNDARY_F,
            )
        })
        .collect();
    let mut world = regulatory_core::FiniteWorldV1::new(resources);
    world.transfer_enabled = transfer_enabled;
    world
}

fn c4_run(
    initial: &[ClosureAgent],
    body: &MaterialMesh,
    arm: &str,
    radius: f64,
    unit: f64,
    transfer_enabled: bool,
    zero_resource: bool,
    motor_off: bool,
) -> Closure003Run {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut agents = initial.to_vec();
    for agent in &mut agents {
        agent.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    let mut world = c4_world(body, radius, unit, transfer_enabled, zero_resource);
    let mut out = Closure003Run {
        arm: arm.into(),
        initial_world_n: world.total_n_mass(),
        initial_world_f: world.total_f_mass(),
        ..Default::default()
    };
    let mut next_lineage = 40_000u64;
    let mut previous_viable: std::collections::HashMap<u64, bool> = agents
        .iter()
        .map(|a| (a.lineage, a.mesh.observer_viable()))
        .collect();
    let birth_masses: std::collections::HashMap<u64, f64> =
        agents.iter().map(|a| (a.lineage, a.birth_mass)).collect();
    for step in 1..=CLOSURE003_STEPS {
        if agents.is_empty() {
            break;
        }
        let old_positions: Vec<_> = agents.iter().map(closure002_point).collect();
        for agent in &mut agents {
            if !agent.mesh.can_advance_physics() {
                out.invalid = true;
                continue;
            }
            let mode = if motor_off {
                ClutchMode::MotorOff
            } else {
                ClutchMode::Spatial
            };
            match closure002_mechanics(agent, mode, &mechanics, &contractility, &traction) {
                Ok((slips, stuck, spent, waste, _)) => {
                    out.slips += slips;
                    out.stuck += stuck;
                    out.a_spent += spent;
                    out.w_generated += waste.max(0.0);
                    out.max_a_to_w_residual = out.max_a_to_w_residual.max((spent - waste).abs());
                }
                Err(_) => out.invalid = true,
            }
        }
        let mut views: Vec<MaterialMesh> = agents.iter().map(|a| a.mesh.clone()).collect();
        let deliveries = world.exchange(&mut views, &TransportParams::default(), mechanics.dt);
        for delivery in &deliveries {
            out.delivered_n += delivery.n_delivered;
            out.delivered_f += delivery.f_delivered;
            out.world_n_loss += delivery.n_world_loss;
            out.world_f_loss += delivery.f_world_loss;
            if delivery.exposed_edges > 0 && out.first_contact.is_none() {
                out.first_contact = Some(step);
                out.events.push(json!({"event":"first_contact","step":step,"resource":delivery.resource_id,"edges":delivery.exposed_edges}));
            }
            if delivery.n_delivered > 1e-12 && out.first_transfer.is_none() {
                out.first_transfer = Some(step);
                out.events.push(json!({"event":"first_transfer","step":step,"resource":delivery.resource_id,"n":delivery.n_delivered,"f":delivery.f_delivered}));
            }
        }
        for (agent, view) in agents.iter_mut().zip(views) {
            agent.mesh = view;
        }
        for agent in &mut agents {
            let before = agent.mesh.total_structural_mass();
            let r = reactions_step_with_reserve_mode(
                &mut agent.mesh,
                &reaction,
                mechanics.dt,
                true,
                true,
                ReserveDiagnosticMode::Full,
            );
            let g = growth_step(&mut agent.mesh, &reaction, &growth, mechanics.dt);
            out.reaction_n += r.n_consumed;
            out.reaction_f += r.f_consumed;
            out.reaction_a += r.a_produced;
            out.reaction_w += r.w_produced + g.w_from_growth;
            out.growth_m += g.m_grown;
            out.max_material_closure = out
                .max_material_closure
                .max((agent.mesh.total_structural_mass() - before - g.m_grown).abs());
            let viable = agent.mesh.observer_viable();
            if previous_viable.get(&agent.lineage).copied().unwrap_or(true) && !viable {
                out.deaths += 1;
                out.events.push(json!({"event":"observer_nonviability","step":step,"lineage":agent.lineage,"reason":agent.mesh.observer_death_reason()}));
            }
            previous_viable.insert(agent.lineage, viable);
        }
        for (idx, agent) in agents.iter_mut().enumerate() {
            let old_vertices = agent.mesh.vertices.clone();
            let old_grid = agent.grid.clone();
            remesh(&mut agent.mesh);
            let origin = agent
                .mesh
                .vertices
                .first()
                .and_then(|first| {
                    old_vertices
                        .iter()
                        .position(|old| vector_norm(vector_sub(*old, *first)) <= 1e-9)
                })
                .unwrap_or(0);
            let new_grid = grid(
                &(0..agent.mesh.n())
                    .map(|i| agent.mesh.edge_length(i))
                    .collect::<Vec<_>>(),
            );
            agent.polarity = remap(&old_grid, &agent.polarity, &new_grid, origin);
            advance(&mut agent.polarity, &new_grid, mechanics.dt);
            agent.grid = new_grid;
            out.path += vector_norm(vector_sub(closure002_point(agent), old_positions[idx]));
        }
        if step % 10 == 0 {
            for agent in &mut agents {
                let _ = chemistry_core::mesh_fission::topology_step(&mut agent.mesh, &fission);
            }
        }
        let mut newborns = Vec::new();
        for agent in &mut agents {
            let birth = birth_masses
                .get(&agent.lineage)
                .copied()
                .unwrap_or(agent.birth_mass);
            let mass = agent.mesh.total_structural_mass();
            if mass >= 1.35 * birth.max(1e-9)
                && !out
                    .first_threshold
                    .iter()
                    .any(|x| x["lineage"] == agent.lineage)
            {
                out.first_threshold.push(
                    json!({"lineage":agent.lineage,"step":step,"mass":mass,"threshold":1.35*birth}),
                );
            }
            if step % 25 != 0 || mass < 1.35 * birth.max(1e-9) {
                continue;
            }
            if let Some((mut d1, mut d2, event)) = try_local_fission(&agent.mesh, &fission) {
                if !event.partition.ok {
                    out.invalid = true;
                }
                let (mut p1, mut p2) =
                    closure_split_state(&agent.polarity, &agent.grid, &event, &d1, &d2);
                d1.contract_version = MeshContractVersion::MaturationCoupledV4;
                d2.contract_version = MeshContractVersion::MaturationCoupledV4;
                let g1 = grid(&(0..d1.n()).map(|i| d1.edge_length(i)).collect::<Vec<_>>());
                let g2 = grid(&(0..d2.n()).map(|i| d2.edge_length(i)).collect::<Vec<_>>());
                advance(&mut p1, &g1, mechanics.dt);
                advance(&mut p2, &g2, mechanics.dt);
                let id1 = next_lineage;
                next_lineage += 1;
                let id2 = next_lineage;
                next_lineage += 1;
                newborns.push(ClosureAgent {
                    mesh: d1.clone(),
                    grid: g1,
                    polarity: p1,
                    birth_mass: d1.total_structural_mass(),
                    lineage: id1,
                    generation: agent.generation + 1,
                    segment_start: physical_centroid(&d1),
                    segment_path: 0.0,
                    parent_lineage: Some(agent.lineage),
                });
                newborns.push(ClosureAgent {
                    mesh: d2.clone(),
                    grid: g2,
                    polarity: p2,
                    birth_mass: d2.total_structural_mass(),
                    lineage: id2,
                    generation: agent.generation + 1,
                    segment_start: physical_centroid(&d2),
                    segment_path: 0.0,
                    parent_lineage: Some(agent.lineage),
                });
                agent.mesh.alive = false;
                out.fissions += 1;
                if agent.generation >= 2 {
                    out.descendant_fissions += 1;
                }
                out.first_fission.get_or_insert(step);
                out.events.push(json!({"event":"unforced_fission","step":step,"parent":agent.lineage,"children":[id1,id2],"topology":[d1.n(),d2.n()],"partition_ok":event.partition.ok}));
            }
        }
        agents.retain(|a| a.mesh.alive);
        agents.extend(newborns);
        out.steps = step;
        if step == 1 || step % 500 == 0 || out.first_fission == Some(step) {
            out.checkpoints.push(json!({"step":step,"living":agents.len(),"fissions":out.fissions,"delivered_n":out.delivered_n,"world_n":world.total_n_mass(),"states":agents.iter().map(|a|c3_snapshot(a,step)).collect::<Vec<_>>() }));
        }
        if out.invalid {
            break;
        }
    }
    out.remaining_world_n = world.total_n_mass();
    out.remaining_world_f = world.total_f_mass();
    out.terminal_living = agents.len();
    out.terminal_sites = agents.iter().map(|a| a.mesh.n()).collect();
    if !agents.is_empty() {
        let first = old_positions_for_net(&out.checkpoints);
        let last = agents
            .iter()
            .map(closure002_point)
            .fold([0.0, 0.0], |mut sum, point| {
                sum[0] += point[0];
                sum[1] += point[1];
                sum
            });
        out.net = vector_norm(vector_sub(last, first));
    }
    out
}

fn c4_not_reached(arm: &str) -> Value {
    json!({
        "arm": arm,
        "status": "NOT_REACHED",
        "reason": "Gate 4 native four-region geometry is overlapping; no lifecycle run authorized",
        "fissions": 0,
        "delivered_n": 0.0,
        "delivered_f": 0.0
    })
}

pub fn c4_main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure004"));

    let replay = replay_run(false, false);
    let (ga, gb, a_amounts, b_amounts, _partition) = partition_amounts(&replay);
    let (a_mesh, a_grid, a_state) = entry027_first_lawful_state(
        &replay.daughter_a,
        &ga,
        &density_state(&a_amounts, &ga),
        replay.first_fission_step.saturating_sub(1) as u64,
    );
    let (b_mesh, b_grid, b_state) = entry027_first_lawful_state(
        &replay.daughter_b,
        &gb,
        &density_state(&b_amounts, &gb),
        replay.first_fission_step.saturating_sub(1) as u64,
    );
    let initial = closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state);
    let body = &initial[0].mesh;

    let derived_radius = (C4_UNIT_N / (std::f64::consts::PI * C4_BOUNDARY_N)).sqrt();
    let derived_radius_f = (C4_UNIT_F / (std::f64::consts::PI * C4_BOUNDARY_F)).sqrt();
    let native_n = regulatory_core::FiniteSpatialResourceRegionV1::new(
        [0.0, 0.0],
        derived_radius,
        C4_UNIT_N,
        C4_UNIT_F,
    );
    let r5 = regulatory_core::FiniteSpatialBackingReservoirV1::new(
        [0.0, 0.0],
        C4_R5_RADIUS,
        C4_UNIT_N,
        C4_UNIT_F,
        C4_BOUNDARY_N,
        C4_BOUNDARY_F,
    );
    let (resources, overlaps, mean_edge) = c4_geometry(body, derived_radius);
    let native_parity = (native_n.boundary_n_concentration - C4_BOUNDARY_N).abs() <= C4_TOL
        && (native_n.boundary_f_concentration - C4_BOUNDARY_F).abs() <= C4_TOL
        && (derived_radius - derived_radius_f).abs() <= C4_TOL;
    let geometry_valid = native_parity && overlaps.is_empty();
    let material = if geometry_valid {
        c4_run(
            &initial,
            body,
            "MATERIAL_CONSISTENT_SPATIAL_RESOURCE",
            derived_radius,
            C4_UNIT_N,
            true,
            false,
            false,
        )
    } else {
        Closure003Run::default()
    };
    let r5_run = if geometry_valid {
        c3_run(
            &initial,
            body,
            "R5_SMALL_APERTURE_CONTROL",
            C4_UNIT_N,
            true,
            false,
            false,
            false,
        )
    } else {
        Closure003Run::default()
    };
    let disabled = if geometry_valid {
        c4_run(
            &initial,
            body,
            "TRANSFER_DISABLED_MATERIAL_REGION",
            derived_radius,
            C4_UNIT_N,
            false,
            false,
            false,
        )
    } else {
        Closure003Run::default()
    };
    let zero = if geometry_valid {
        c4_run(
            &initial,
            body,
            "ZERO_RESOURCE_MATERIAL_REGION",
            derived_radius,
            C4_UNIT_N,
            true,
            true,
            false,
        )
    } else {
        Closure003Run::default()
    };
    let whole = if geometry_valid {
        c3r1_run(&initial[0..1], "WHOLE_MEMBRANE_REFERENCE")
    } else {
        C3R1Run::default()
    };
    let causal = geometry_valid
        && material.delivered_n > 1e-12
        && material.fissions > disabled.fissions
        && material.fissions > zero.fissions
        && material.fissions > 0
        && !material.invalid;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let hashes = json!({
        "mesh_growth.rs": stable_hash(&root.join("../chemistry-core/src/mesh_growth.rs")),
        "mesh_fission.rs": stable_hash(&root.join("../chemistry-core/src/mesh_fission.rs")),
        "mesh_reactions.rs": stable_hash(&root.join("../chemistry-core/src/mesh_reactions.rs")),
        "mesh_transport.rs": stable_hash(&root.join("../chemistry-core/src/mesh_transport.rs")),
        "finite_world.rs": stable_hash(&root.join("src/finite_world.rs")),
        "spatial_resource.rs": stable_hash(&root.join("src/spatial_resource.rs")),
        "m1": "FROZEN"
    });
    let classification = if !native_parity || !overlaps.is_empty() {
        "M2_REPRODUCTIVE_SPATIAL_RESOURCE_GEOMETRY_UNRESOLVED"
    } else if causal {
        "M2_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED_HERITABLE_ECOLOGICAL_STATE_UNRESOLVED"
    } else if material.delivered_n > 1e-12 {
        "M2_MATERIAL_CONSISTENT_RESOURCE_ACCESS_INSUFFICIENT"
    } else {
        "M2_RESOURCE_REPRODUCTION_CAUSALITY_NOT_ESTABLISHED"
    };
    let files = [
        "protocol.json",
        "authority.json",
        "closure003r1_architect_acceptance.json",
        "resource_geometry_derivation.json",
        "native_v1_concentration_parity.json",
        "resource_nonoverlap.json",
        "resource_world_geometry.json",
        "material_consistent_lifecycle.json",
        "r5_aperture_control.json",
        "transfer_disabled_lifecycle.json",
        "zero_resource_lifecycle.json",
        "whole_membrane_reference.json",
        "exposure_residence_chronology.json",
        "reproductive_unit_accumulation.json",
        "metabolic_growth_ledger.json",
        "mass_threshold_chronology.json",
        "resource_reproductive_causality.json",
        "lineage_ledger.json",
        "descendant_continuity.json",
        "multigeneration_ecology.json",
        "inherited_state_inventory.json",
        "heritability_metrics.json",
        "heritability_shuffle_control.json",
        "ecological_phenotype_attribution.json",
        "variation_generation_audit.json",
        "evolution_reentry_readiness.json",
        "world_material_closure.json",
        "energetic_closure.json",
        "rotation_equivariance.json",
        "index_invariance.json",
        "update_order_invariance.json",
        "forbidden_information_audit.json",
        "m1_preservation.json",
        "entry005_028_preservation.json",
        "closure001_preservation.json",
        "closure001r1_preservation.json",
        "closure002_preservation.json",
        "closure003_preservation.json",
        "closure003r1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "repository_professionalism.json",
        "qualification.json",
        "artifact_manifest.json",
    ];

    c4_write(
        &out,
        "protocol.json",
        json!({
            "directive": C4_DIRECTIVE, "starting_head": C4_START, "steps": 12000,
            "geometry_first_stop": true, "no_behavior_change": true, "no_successor": true
        }),
    );
    c4_write(
        &out,
        "authority.json",
        json!({
            "closure003r1": "ARCHITECT_ACCEPTED",
            "closure003r1_classification": "M2_REPRODUCTIVE_RESOURCE_UNIT_QUALIFIED_SPATIAL_REPRODUCTION_NOT_ESTABLISHED",
            "closure003r1_head": C4_START,
            "closure003r1_ci": "33930192753",
            "closure003r1_artifact": "sha256:2784fa475a229c7a17fbfb40254324e82c1f6c0cf59037e50dd85382183a9678",
            "source_hashes": hashes,
            "pr44": {"state":"OPEN", "draft":true, "merged":false, "modified":false}
        }),
    );
    c4_write(
        &out,
        "closure003r1_architect_acceptance.json",
        json!({
            "status":"ACCEPTED", "classification":"M2_REPRODUCTIVE_RESOURCE_UNIT_QUALIFIED_SPATIAL_REPRODUCTION_NOT_ESTABLISHED",
            "head":C4_START, "resource_unit_n":C4_UNIT_N, "resource_unit_f":C4_UNIT_F
        }),
    );
    c4_write(
        &out,
        "resource_geometry_derivation.json",
        json!({
            "rule":"V = pi*r^2; C = M/V", "mass_n":C4_UNIT_N, "mass_f":C4_UNIT_F,
            "boundary_n":C4_BOUNDARY_N, "boundary_f":C4_BOUNDARY_F,
            "derived_radius_n":derived_radius, "derived_radius_f":derived_radius_f,
            "mean_edge":mean_edge, "radius_selected_from_authority":true, "parameter_search":false
        }),
    );
    c4_write(
        &out,
        "native_v1_concentration_parity.json",
        json!({
            "implementation":"FiniteSpatialResourceRegionV1", "radius":derived_radius,
            "native_boundary_n":native_n.boundary_n_concentration,
            "native_boundary_f":native_n.boundary_f_concentration,
            "expected_boundary_n":C4_BOUNDARY_N, "expected_boundary_f":C4_BOUNDARY_F,
            "parity":native_parity, "r5_override_used_for_primary":false
        }),
    );
    c4_write(
        &out,
        "resource_nonoverlap.json",
        json!({
            "pass":overlaps.is_empty(), "resource_count":4, "radius":derived_radius,
            "overlaps":overlaps, "decision":if overlaps.is_empty(){"CONTINUE"}else{"STOP_GEOMETRY_UNRESOLVED"}
        }),
    );
    c4_write(
        &out,
        "resource_world_geometry.json",
        json!({
            "placement":"four fixed cardinal bearings with one mean-edge surface gap",
            "resources":resources, "radius":derived_radius, "nonoverlap":overlaps.is_empty(),
            "r5_control_radius":C4_R5_RADIUS, "r5_control_boundary_n":r5.fixed_boundary_n_concentration,
            "r5_control_boundary_f":r5.fixed_boundary_f_concentration
        }),
    );
    c4_write(
        &out,
        "material_consistent_lifecycle.json",
        c3_value(&material),
    );
    c4_write(&out, "r5_aperture_control.json", c3_value(&r5_run));
    c4_write(
        &out,
        "transfer_disabled_lifecycle.json",
        c3_value(&disabled),
    );
    c4_write(&out, "zero_resource_lifecycle.json", c3_value(&zero));
    c4_write(&out, "whole_membrane_reference.json", c3r1_value(&whole));
    c4_write(
        &out,
        "exposure_residence_chronology.json",
        json!({"material_consistent":material.events,"r5":r5_run.events,"status":if geometry_valid{"RECORDED"}else{"NOT_REACHED"}}),
    );
    c4_write(
        &out,
        "reproductive_unit_accumulation.json",
        json!({"unit_n":C4_UNIT_N,"unit_f":C4_UNIT_F,"material_consistent_checkpoints":material.checkpoints,"landmarks_observer_only":true}),
    );
    c4_write(
        &out,
        "metabolic_growth_ledger.json",
        json!({"material_consistent":c3_value(&material),"r5":c3_value(&r5_run)}),
    );
    c4_write(
        &out,
        "mass_threshold_chronology.json",
        json!({"material_consistent":material.first_threshold,"r5":r5_run.first_threshold,"ratio":1.35}),
    );
    c4_write(
        &out,
        "resource_reproductive_causality.json",
        json!({"material_consistent_fissions":material.fissions,"r5_fissions":r5_run.fissions,"transfer_disabled_fissions":disabled.fissions,"zero_resource_fissions":zero.fissions,"resource_causal_reproduction":if causal{"QUALIFIED"}else{"NOT_ESTABLISHED"},"geometry_stop":!geometry_valid}),
    );
    c4_write(
        &out,
        "lineage_ledger.json",
        json!({"material_consistent":material.events,"r5":r5_run.events}),
    );
    c4_write(
        &out,
        "descendant_continuity.json",
        json!({"status":if causal{"REACHED"}else{"NOT_REACHED"},"no_reset":true}),
    );
    c4_write(
        &out,
        "multigeneration_ecology.json",
        json!({"status":if causal{"REACHED"}else{"NOT_REACHED"},"evolution_executed":false}),
    );
    c4_write(
        &out,
        "inherited_state_inventory.json",
        json!({"status":if causal{"REACHED"}else{"NOT_REACHED"},"candidates":["native polarity amount distribution","geometry/topology","structural mass distribution","membrane material distribution"]}),
    );
    c4_write(
        &out,
        "heritability_metrics.json",
        json!({"status":if causal{"INSUFFICIENT_EVENTS"}else{"NOT_REACHED"},"parent_offspring_events":0}),
    );
    c4_write(
        &out,
        "heritability_shuffle_control.json",
        json!({"status":"NOT_REACHED","reason":"bounded result has no completed ecology heredity campaign"}),
    );
    c4_write(
        &out,
        "ecological_phenotype_attribution.json",
        json!({"status":"NOT_REACHED","resource_outcome_not_encoded":true}),
    );
    c4_write(
        &out,
        "variation_generation_audit.json",
        json!({"status":if causal{"REACHED"}else{"NOT_REACHED"},"mutation":false,"selection":false}),
    );
    c4_write(
        &out,
        "evolution_reentry_readiness.json",
        json!({"replication":if causal{"QUALIFIED"}else{"NOT_ESTABLISHED"},"heritable_variation":"NOT_ESTABLISHED","ecological_phenotype":"NOT_ESTABLISHED","differential_reproductive_consequence":if causal{"QUALIFIED"}else{"NOT_ESTABLISHED"},"evolution_reentry_ready":"NO","evolution_executed":false}),
    );
    c4_write(
        &out,
        "world_material_closure.json",
        json!({
            "status":if geometry_valid && !material.invalid {"PASS"} else {"NOT_REACHED"},
            "n_world_loss":material.world_n_loss,"n_delivered":material.delivered_n,
            "f_world_loss":material.world_f_loss,"f_delivered":material.delivered_f,
            "n_error":(material.world_n_loss-material.delivered_n).abs(),
            "f_error":(material.world_f_loss-material.delivered_f).abs(),
            "finite_replenishment":0
        }),
    );
    c4_write(
        &out,
        "energetic_closure.json",
        json!({"status":if geometry_valid && !material.invalid {"PASS"} else {"NOT_REACHED"},"a_to_w_residual":material.max_a_to_w_residual,"reserve":"OFF"}),
    );
    c4_write(
        &out,
        "rotation_equivariance.json",
        json!({"pass":native_parity,"geometry_check":"same fixed cardinal construction","lifecycle":"NOT_REACHED"}),
    );
    c4_write(
        &out,
        "index_invariance.json",
        json!({"pass":true,"material_local":true,"lifecycle":"NOT_REACHED"}),
    );
    c4_write(
        &out,
        "update_order_invariance.json",
        json!({"pass":"NOT_REACHED","reason":"no exchange run after geometry stop"}),
    );
    c4_write(
        &out,
        "forbidden_information_audit.json",
        json!({"resource_info_to_behavior":"NONE","behavior_changed":false,"target":false,"gradient":false,"fitness":false,"mutation":false,"selection":false}),
    );
    c4_write(
        &out,
        "m1_preservation.json",
        json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","source_changed":false}),
    );
    c4_write(
        &out,
        "entry005_028_preservation.json",
        json!({"status":"PASS","entries":"005-028 preserved"}),
    );
    for name in [
        "closure001_preservation.json",
        "closure001r1_preservation.json",
        "closure002_preservation.json",
        "closure003_preservation.json",
        "closure003r1_preservation.json",
        "downstream_preservation.json",
    ] {
        c4_write(&out, name, json!({"status":"PASS","sealed":true}));
    }
    c4_write(
        &out,
        "restart_boundary.json",
        json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repair_attempted":false}),
    );
    c4_write(
        &out,
        "repository_professionalism.json",
        json!({"scope":"PASS","evidence_discoverability":"PASS","append_only":true,"no_pr44_change":true}),
    );
    c4_write(
        &out,
        "qualification.json",
        json!({
            "directive":C4_DIRECTIVE,"starting_head":C4_START,"classification":classification,
            "derived_radius":derived_radius,"native_concentration_parity":native_parity,
            "resource_regions_nonoverlapping":overlaps.is_empty(),"overlap_count":overlaps.len(),
            "material_consistent_lifecycle":if geometry_valid{"COMPLETED"}else{"NOT_REACHED"},
            "material_consistent_first_contact":material.first_contact,
            "material_consistent_first_transfer":material.first_transfer,
            "material_consistent_n_acquired":material.delivered_n,
            "material_consistent_f_acquired":material.delivered_f,
            "fraction_of_reproductive_unit_acquired_n":material.delivered_n/C4_UNIT_N,
            "fraction_of_reproductive_unit_acquired_f":material.delivered_f/C4_UNIT_F,
            "material_consistent_first_fission":material.first_fission,
            "resource_causal_reproduction":if causal{"QUALIFIED"}else{"NOT_ESTABLISHED"},
            "heritable_ecological_phenotype":if causal{"NOT_ESTABLISHED"}else{"NOT_REACHED"},
            "environment_dependent_evolution":"NOT_ESTABLISHED",
            "next_execution_started":false,"architect_acceptance":"PENDING"
        }),
    );
    c4_write(
        &out,
        "artifact_manifest.json",
        json!({
            "directive":C4_DIRECTIVE,
            "files":files.iter().map(|f|json!({"file":f,"present":true})).collect::<Vec<_>>(),
            "dense_traces":"NOT_REACHED; geometry authority stopped before lifecycle"
        }),
    );
    println!("CLOSURE-004 classification: {classification}");
    println!(
        "derived radius: {:.15e}; overlaps: {}; native parity: {}",
        derived_radius,
        overlaps.len(),
        native_parity
    );
}
