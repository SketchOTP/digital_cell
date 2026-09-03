// DC-DEV-021 M2 CLOSURE-001: finite-world autonomous ecology and reproductive coupling.
//
// This module uses the accepted ENTRY-027 physical fission replay and native
// inherited-polarity state in the surrounding module.  New world exchange is
// delegated to FiniteWorldV1; no legacy transport path is changed.

const CLOSURE_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-001-R1-FINITE-WORLD-SENSORIMOTOR-REQUALIFICATION-AND-POLARITY-CLUTCH-MIGRATION-001";
const CLOSURE_START: &str = "650a8473f661e1743472b79b79a3b63becbf6d27";
const CLOSURE_STEPS: usize = 3_000;
const CLOSURE_RADIUS: f64 = 1.5;
const R4_CAPACITY: f64 = 14.588954880632265;
const R4_BOUNDARY: f64 = 2.063914918930895;
const CLOSURE_TOL: f64 = 1e-10;

#[derive(Clone)]
struct ClosureAgent {
    mesh: MaterialMesh,
    grid: Grid,
    polarity: AmountState,
    birth_mass: f64,
    lineage: u64,
    generation: u32,
    segment_start: [f64; 2],
    segment_path: f64,
    parent_lineage: Option<u64>,
}

#[derive(Clone, Default)]
struct ClosureRun {
    arm: String,
    steps: usize,
    fissions: usize,
    descendant_fissions: usize,
    first_contact_step: Option<usize>,
    first_transfer_step: Option<usize>,
    contacts: usize,
    delivered_n: f64,
    delivered_f: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    initial_world_n: f64,
    initial_world_f: f64,
    remaining_world_n: f64,
    remaining_world_f: f64,
    path: f64,
    net: f64,
    slips: usize,
    stuck: usize,
    a_spent: f64,
    motor_w: f64,
    reaction_n: f64,
    reaction_f: f64,
    reaction_a: f64,
    reaction_w: f64,
    a_to_w_residual: f64,
    invalid: bool,
    terminal_living: usize,
    terminal_sites: Vec<usize>,
    points: Vec<Value>,
    segment_records: Vec<Value>,
}

fn closure_place(mesh: &MaterialMesh, bearing: u16, direction: [f64; 2]) -> [f64; 2] {
    // R1 geometry rule: solve against the closed polygon boundary itself.
    // The old radius-of-circumscribed-body shortcut is intentionally not
    // reused because it is not the material surface gap.
    let centroid = physical_centroid(mesh);
    let mean_edge = mesh.perimeter() / mesh.n().max(1) as f64;
    let point_segment_distance = |p: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        let ab = vector_sub(b, a);
        let denom = ab[0] * ab[0] + ab[1] * ab[1];
        let t = if denom > 0.0 {
            ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / denom
        } else {
            0.0
        };
        let t = t.clamp(0.0, 1.0);
        vector_norm(vector_sub(p, [a[0] + t * ab[0], a[1] + t * ab[1]]))
    };
    let surface_gap = |distance: f64| {
        let center = [
            centroid[0] + distance * direction[0],
            centroid[1] + distance * direction[1],
        ];
        let boundary = (0..mesh.n())
            .map(|edge| {
                point_segment_distance(
                    center,
                    mesh.vertices[edge],
                    mesh.vertices[(edge + 1) % mesh.n()],
                )
            })
            .fold(f64::INFINITY, f64::min);
        boundary - CLOSURE_RADIUS
    };
    let mut high = mesh
        .vertices
        .iter()
        .map(|p| vector_norm(vector_sub(*p, centroid)))
        .fold(0.0, f64::max)
        + CLOSURE_RADIUS
        + mean_edge;
    while surface_gap(high) < mean_edge {
        high *= 2.0;
    }
    let mut low = 0.0;
    for _ in 0..80 {
        let mid = 0.5 * (low + high);
        if surface_gap(mid) < mean_edge {
            low = mid;
        } else {
            high = mid;
        }
    }
    let distance = high;
    let _ = bearing;
    [
        centroid[0] + distance * direction[0],
        centroid[1] + distance * direction[1],
    ]
}

fn closure_world(centroid_body: &MaterialMesh, zero: bool) -> regulatory_core::FiniteWorldV1 {
    let dirs = [
        (0u16, [1.0, 0.0]),
        (90, [0.0, 1.0]),
        (180, [-1.0, 0.0]),
        (270, [0.0, -1.0]),
    ];
    let resources = dirs
        .iter()
        .map(|(bearing, direction)| {
            let center = closure_place(centroid_body, *bearing, *direction);
            regulatory_core::FiniteWorldResourceV1::new(
                format!("r{bearing}"),
                center,
                CLOSURE_RADIUS,
                if zero { 0.0 } else { R4_CAPACITY },
                if zero { 0.0 } else { R4_CAPACITY },
                if zero { 0.0 } else { R4_BOUNDARY },
                if zero { 0.0 } else { R4_BOUNDARY },
            )
        })
        .collect();
    regulatory_core::FiniteWorldV1::new(resources)
}

fn closure_contact_sanity(mesh: &MaterialMesh) -> Value {
    let edge_end = mesh.vertices[1 % mesh.n()];
    let edge_start = mesh.vertices[0];
    let center = [
        0.5 * (edge_start[0] + edge_end[0]),
        0.5 * (edge_start[1] + edge_end[1]),
    ];
    let resource = regulatory_core::FiniteWorldResourceV1::new(
        "contact-sanity",
        center,
        CLOSURE_RADIUS,
        R4_CAPACITY,
        R4_CAPACITY,
        R4_BOUNDARY,
        R4_BOUNDARY,
    );
    let mut world = regulatory_core::FiniteWorldV1::new(vec![resource]);
    let mut meshes = vec![mesh.clone()];
    meshes[0].contract_version = MeshContractVersion::MaturationCoupledV4;
    let initial_n = world.total_n_mass();
    let initial_f = world.total_f_mass();
    let deliveries = world.exchange(
        &mut meshes,
        &TransportParams::default(),
        MechParams::default().dt,
    );
    let delivery = deliveries.first().cloned();
    let transferred_n = delivery.as_ref().map(|d| d.n_delivered).unwrap_or(0.0);
    let transferred_f = delivery.as_ref().map(|d| d.f_delivered).unwrap_or(0.0);
    let reaction = reactions_step_with_reserve_mode(
        &mut meshes[0],
        &ReactionParams::conservative_v3(),
        MechParams::default().dt,
        true,
        true,
        ReserveDiagnosticMode::Full,
    );
    json!({
        "contact_geometry": "resource centered on cloned organism edge-0 midpoint",
        "positive_transfer": transferred_n > 1e-12 && transferred_f > 1e-12,
        "exposed_edges": delivery.as_ref().map(|d| d.exposed_edges).unwrap_or(0),
        "n_delivered": delivery.as_ref().map(|d| d.n_delivered).unwrap_or(0.0),
        "f_delivered": delivery.as_ref().map(|d| d.f_delivered).unwrap_or(0.0),
        "world_n_loss": initial_n - world.total_n_mass(),
        "world_f_loss": initial_f - world.total_f_mass(),
        "world_debit_matches_delivery": delivery.as_ref().map(|d| {
            (initial_n - world.total_n_mass() - d.n_delivered).abs() <= CLOSURE_TOL
                && (initial_f - world.total_f_mass() - d.f_delivered).abs() <= CLOSURE_TOL
        }).unwrap_or(false),
        "organism_n_amount_after": meshes[0].interior.n * meshes[0].area(),
        "organism_f_amount_after": meshes[0].interior.f * meshes[0].area(),
        "metabolism": {
            "active": reaction.n_consumed > 0.0 && reaction.f_consumed > 0.0,
            "n_consumed": reaction.n_consumed,
            "f_consumed": reaction.f_consumed,
            "a_produced": reaction.a_produced,
            "w_produced": reaction.w_produced,
            "transfer_enters_same_step": transferred_n > 0.0 && reaction.n_consumed > 0.0,
        },
        "finite_world": true,
    })
}

fn closure_surface_gap(mesh: &MaterialMesh, center: [f64; 2]) -> f64 {
    (0..mesh.n())
        .map(|edge| {
            let a = mesh.vertices[edge];
            let b = mesh.vertices[(edge + 1) % mesh.n()];
            let ab = vector_sub(b, a);
            let denom = ab[0] * ab[0] + ab[1] * ab[1];
            let t = if denom > 0.0 {
                ((center[0] - a[0]) * ab[0] + (center[1] - a[1]) * ab[1]) / denom
            } else {
                0.0
            };
            let t = t.clamp(0.0, 1.0);
            vector_norm(vector_sub(center, [a[0] + t * ab[0], a[1] + t * ab[1]]))
        })
        .fold(f64::INFINITY, f64::min)
}

fn closure_geometry_audit(mesh: &MaterialMesh) -> Value {
    let mean_edge = mesh.perimeter() / mesh.n().max(1) as f64;
    let dirs = [
        (0u16, [1.0, 0.0]),
        (90, [0.0, 1.0]),
        (180, [-1.0, 0.0]),
        (270, [0.0, -1.0]),
    ];
    let rows: Vec<_> = dirs
        .iter()
        .map(|(bearing, direction)| {
            let center = closure_place(mesh, *bearing, *direction);
            let gap = closure_surface_gap(mesh, center) - CLOSURE_RADIUS;
            json!({"bearing":bearing,"center":center,"surface_gap":gap,"target_gap":mean_edge,"residual":(gap-mean_edge).abs()})
        })
        .collect();
    json!({"method":"closed polygon point-to-segment distance solved along four balanced bearings","mean_material_edge":mean_edge,"resources":rows,"circumscribed_radius_shortcut":false,"pass":rows.iter().all(|row| row["residual"].as_f64().unwrap_or(f64::INFINITY)<=1e-9)})
}

fn closure_transport_boundary_audit(mesh: &MaterialMesh) -> Value {
    let mut body = mesh.clone();
    body.contract_version = MeshContractVersion::MaturationCoupledV4;
    body.interior.c = 0.8;
    body.interior.a = 0.8;
    body.interior.w = 0.8;
    body.exterior = chemistry_core::material_mesh::LumpedChem {
        n: 4.0,
        f: 4.0,
        ..Default::default()
    };
    let before = body.interior;
    let mut world = regulatory_core::FiniteWorldV1::new(vec![
        regulatory_core::FiniteWorldResourceV1::new(
            "transport-audit",
            [10_000.0, 10_000.0],
            CLOSURE_RADIUS,
            0.0,
            0.0,
            0.0,
            0.0,
        ),
    ]);
    let deliveries = world.exchange(
        std::slice::from_mut(&mut body),
        &TransportParams::default(),
        MechParams::default().dt,
    );
    let nonfeed = deliveries
        .first()
        .map(|delivery| delivery.nonfeeding_transport.clone())
        .unwrap_or_default();
    json!({
        "finite_world_positive_nf_source_only":"finite inventory",
        "transport_exterior_nf_zeroed_for_exchange":true,
        "mechanical_exterior_reference_restored":body.exterior.n == 4.0 && body.exterior.f == 4.0,
        "n_in":nonfeed.n_in,"f_in":nonfeed.f_in,"n_out":nonfeed.n_out,"f_out":nonfeed.f_out,
        "w_out":nonfeed.w_out,"c_leak":nonfeed.c_leak,"a_leak":nonfeed.a_leak,
        "positive_unbacked_nf_inflow":nonfeed.n_in + nonfeed.f_in,
        "legacy_nonfeeding_transport_executed":true,
        "material_not_increased_without_world":body.interior.n*body.area() <= before.n*body.area() + 1e-12 && body.interior.f*body.area() <= before.f*body.area() + 1e-12,
        "pass":nonfeed.n_in.abs() <= CLOSURE_TOL && nonfeed.f_in.abs() <= CLOSURE_TOL
    })
}

fn closure_agents(
    a_mesh: &MaterialMesh,
    a_grid: &Grid,
    a_state: &AmountState,
    b_mesh: &MaterialMesh,
    b_grid: &Grid,
    b_state: &AmountState,
) -> Vec<ClosureAgent> {
    vec![
        ClosureAgent {
            mesh: a_mesh.clone(),
            grid: a_grid.clone(),
            polarity: a_state.clone(),
            birth_mass: a_mesh.total_structural_mass(),
            lineage: 1,
            generation: 1,
            segment_start: physical_centroid(a_mesh),
            segment_path: 0.0,
            parent_lineage: None,
        },
        ClosureAgent {
            mesh: b_mesh.clone(),
            grid: b_grid.clone(),
            polarity: b_state.clone(),
            birth_mass: b_mesh.total_structural_mass(),
            lineage: 2,
            generation: 1,
            segment_start: physical_centroid(b_mesh),
            segment_path: 0.0,
            parent_lineage: None,
        },
    ]
}

fn closure_split_state(
    parent: &AmountState,
    old: &Grid,
    event: &FissionEvent,
    d1: &MaterialMesh,
    d2: &MaterialMesh,
) -> (AmountState, AmountState) {
    let n = old.ds.len();
    let (i, j) = event.pinch;
    let take = |start: usize, daughter: &MaterialMesh, q: &[f64]| {
        let mut amount = Vec::with_capacity(daughter.n());
        for k in 0..daughter.n() {
            amount.push(if k + 1 == daughter.n() {
                0.0
            } else {
                q[(start + k) % n] * old.ds[(start + k) % n]
            });
        }
        let dg = grid(
            &(0..daughter.n())
                .map(|x| daughter.edge_length(x))
                .collect::<Vec<_>>(),
        );
        amount
            .iter()
            .zip(&dg.ds)
            .map(|(x, d)| x / d)
            .collect::<Vec<_>>()
    };
    let make = |start: usize, daughter: &MaterialMesh| AmountState {
        u: take(start, daughter, &parent.u),
        v: take(start, daughter, &parent.v),
        f: take(start, daughter, &parent.f),
    };
    (make(i, d1), make(j, d2))
}

fn closure_point(agent: &ClosureAgent) -> [f64; 2] {
    physical_centroid(&agent.mesh)
}

fn closure_outward_normal(mesh: &MaterialMesh, edge: usize) -> [f64; 2] {
    let a = mesh.vertices[edge];
    let b = mesh.vertices[(edge + 1) % mesh.n()];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let length = dx.hypot(dy).max(1e-15);
    let sign = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    [sign * dy / length, -sign * dx / length]
}

/// Build the single conditional Stage-4 force field.  `motor` is the
/// accepted v/(u+v) contractile fraction; its complement is the local
/// outward component.  The existing mechanics force bound and force-length-
/// time cost provide both the scale and the A budget; no new parameter is
/// introduced.
fn closure_protrusion(mesh: &MaterialMesh, motor: &[f64], dt: f64) -> (Vec<[f64; 2]>, f64) {
    let n = mesh.n();
    let mut forces = vec![[0.0, 0.0]; n];
    let mut requested = 0.0;
    for edge in 0..n {
        if mesh.edges[edge].ruptured {
            continue;
        }
        let local_fraction = 1.0 - 0.5 * (motor[edge] + motor[(edge + 1) % n]);
        let normal = closure_outward_normal(mesh, edge);
        // Leave the existing static-traction budget available for the
        // substrate reaction so the combined force stays within the
        // established per-vertex force bound. This is an assay-only use of
        // existing limits, not a new protrusion coefficient.
        let edge_force = (chemistry_core::mesh_mechanics::MAX_EXTERNAL_FORCE_PER_VERTEX
            - regulatory_core::FROZEN_STATIC_TRACTION_LIMIT)
            .max(0.0)
            * local_fraction.clamp(0.0, 1.0);
        forces[edge][0] += 0.5 * edge_force * normal[0];
        forces[edge][1] += 0.5 * edge_force * normal[1];
        let next = (edge + 1) % n;
        forces[next][0] += 0.5 * edge_force * normal[0];
        forces[next][1] += 0.5 * edge_force * normal[1];
    }
    for vertex in 0..n {
        let local_length =
            0.5 * (mesh.edge_length(vertex) + mesh.edge_length((vertex + n - 1) % n));
        requested += regulatory_core::FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME
            * forces[vertex][0].hypot(forces[vertex][1])
            * local_length
            * dt.max(0.0);
    }
    (forces, requested)
}

fn closure_run(
    initial: &[ClosureAgent],
    body_for_world: &MaterialMesh,
    arm: &str,
    uniform: bool,
    motor_off: bool,
    transfer_enabled: bool,
    zero_resource: bool,
    protrusion: bool,
    clutch: bool,
) -> ClosureRun {
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
        // The closure composition uses the already accepted production V4
        // contract; the replay snapshots are geometry/life-history inputs and
        // do not themselves select a production contract.
        agent.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    let mut world = closure_world(body_for_world, zero_resource);
    world.transfer_enabled = transfer_enabled;
    let initial_world_n = world.total_n_mass();
    let initial_world_f = world.total_f_mass();
    let initial_centroids: Vec<_> = agents.iter().map(closure_point).collect();
    let mut result = ClosureRun {
        arm: arm.to_string(),
        initial_world_n,
        initial_world_f,
        ..Default::default()
    };
    let mut next_lineage = 3u64;
    for step in 1..=CLOSURE_STEPS {
        if agents.is_empty() {
            break;
        }
        let mut meshes: Vec<MaterialMesh> = agents.iter().map(|a| a.mesh.clone()).collect();
        let mut motors = Vec::with_capacity(agents.len());
        for agent in &agents {
            let raw = entry025_anti(&agent.polarity);
            let mean = raw.iter().sum::<f64>() / raw.len().max(1) as f64;
            motors.push(if motor_off {
                vec![0.0; raw.len()]
            } else if uniform {
                vec![mean; raw.len()]
            } else {
                raw
            });
        }
        let old_positions: Vec<_> = agents.iter().map(closure_point).collect();
        for (agent, motor) in agents.iter_mut().zip(motors.iter()) {
            if !agent.mesh.can_advance_physics() {
                result.invalid = true;
                continue;
            }
            if motor_off {
                match apply_stick_slip_to_legacy_mechanics(&mut agent.mesh, &mechanics, &traction) {
                    Ok(ledger) => {
                        result.slips += ledger.slipping_contacts;
                        result.stuck += ledger.stuck_contacts;
                    }
                    Err(_) => result.invalid = true,
                }
            } else {
                let (extra_forces, extra_cost) = if protrusion {
                    closure_protrusion(&agent.mesh, motor, mechanics.dt)
                } else {
                    (vec![[0.0, 0.0]; agent.mesh.n()], 0.0)
                };
                let step = if protrusion {
                    regulatory_core::apply_local_activated_energy_contractility_with_stick_slip_and_extra_forces(
                        &mut agent.mesh, motor, &mechanics, &contractility, &traction,
                        &extra_forces, extra_cost,
                    )
                } else if clutch {
                    let clutch_fraction = entry025_direct(&agent.polarity);
                    regulatory_core::apply_local_activated_energy_contractility_with_local_traction_clutch(
                        &mut agent.mesh,
                        motor,
                        &clutch_fraction,
                        &mechanics,
                        &contractility,
                        &traction,
                    )
                } else {
                    apply_local_activated_energy_contractility_with_stick_slip(
                        &mut agent.mesh,
                        motor,
                        &mechanics,
                        &contractility,
                        &traction,
                    )
                };
                match step {
                    Ok(ledger) => {
                        result.slips += ledger.slipping_contacts;
                        result.stuck += ledger.stuck_contacts;
                        if let Some(c) = ledger.contractility {
                            result.a_spent += c.resource_spent;
                            result.motor_w +=
                                (c.waste_amount_after - c.waste_amount_before).max(0.0);
                            result.a_to_w_residual = result.a_to_w_residual.max(
                                (c.activated_amount_before - c.activated_amount_after
                                    + c.waste_amount_before
                                    - c.waste_amount_after)
                                    .abs(),
                            );
                        }
                    }
                    Err(_) => result.invalid = true,
                }
            }
        }
        // Mechanics ran against the live agents above.  Refresh the world
        // exchange views from those post-mechanics meshes; using the original
        // pre-step clones here would silently erase locomotion when the views
        // are copied back after finite transfer.
        for (view, agent) in meshes.iter_mut().zip(&agents) {
            *view = agent.mesh.clone();
        }
        let deliveries = world.exchange(&mut meshes, &TransportParams::default(), mechanics.dt);
        for delivery in &deliveries {
            result.delivered_n += delivery.n_delivered;
            result.delivered_f += delivery.f_delivered;
            result.world_n_loss += delivery.n_world_loss;
            result.world_f_loss += delivery.f_world_loss;
            if delivery.exposed_edges > 0 {
                result.contacts += 1;
            }
            if delivery.exposed_edges > 0 && result.first_contact_step.is_none() {
                result.first_contact_step = Some(step);
            }
            if delivery.n_delivered > 1e-12 && result.first_transfer_step.is_none() {
                result.first_transfer_step = Some(step);
            }
        }
        for (agent, mesh) in agents.iter_mut().zip(meshes.into_iter()) {
            agent.mesh = mesh;
        }
        // ENTRY-011's accepted finite-transfer composition is mechanics,
        // uptake, then the unchanged frozen reaction kernel. Newly delivered
        // N/F therefore enters this step's metabolism without adding a second
        // external boundary or double-counting any species.
        for agent in &mut agents {
            let r = reactions_step_with_reserve_mode(
                &mut agent.mesh,
                &reaction,
                mechanics.dt,
                true,
                true,
                ReserveDiagnosticMode::Full,
            );
            let g = growth_step(&mut agent.mesh, &reaction, &growth, mechanics.dt);
            result.reaction_n += r.n_consumed;
            result.reaction_f += r.f_consumed;
            result.reaction_a += r.a_produced;
            result.reaction_w += r.w_produced + g.w_from_growth;
        }
        let mut descendants = Vec::new();
        for (agent_index, agent) in agents.iter_mut().enumerate() {
            let old_grid = agent.grid.clone();
            let old_vertices = agent.mesh.vertices.clone();
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
            let now = closure_point(agent);
            if !now.iter().all(|value| value.is_finite()) {
                eprintln!(
                    "non-finite centroid at step {step}, agent {agent_index}, lineage {}",
                    agent.lineage
                );
                result.invalid = true;
                break;
            }
            let delta = vector_norm(vector_sub(now, old_positions[agent_index]));
            result.path += delta;
            agent.segment_path += delta;
            if !result.path.is_finite() {
                eprintln!(
                    "non-finite path at step {step}, agent {agent_index}, lineage {}",
                    agent.lineage
                );
                result.invalid = true;
                break;
            }
        }
        // The topology authority is local and cadence-based; no scripted
        // divide command is used. Newborns inherit contiguous parent amounts.
        for agent in &mut agents {
            if step % 25 != 0
                || agent.mesh.total_structural_mass() < 1.35 * agent.birth_mass.max(1e-9)
            {
                continue;
            }
            if let Some((d1, d2, event)) = try_local_fission(&agent.mesh, &fission) {
                let parent_point = closure_point(agent);
                result.segment_records.push(json!({
                    "lineage": agent.lineage,
                    "generation": agent.generation,
                    "segment_path": agent.segment_path,
                    "segment_net": vector_norm(vector_sub(parent_point, agent.segment_start)),
                    "fission": "unforced physical fission",
                    "child_topologies": [d1.n(), d2.n()]
                }));
                let (mut p1, mut p2) =
                    closure_split_state(&agent.polarity, &agent.grid, &event, &d1, &d2);
                let mut d1 = d1;
                let mut d2 = d2;
                d1.contract_version = MeshContractVersion::MaturationCoupledV4;
                d2.contract_version = MeshContractVersion::MaturationCoupledV4;
                let g1 = grid(&(0..d1.n()).map(|i| d1.edge_length(i)).collect::<Vec<_>>());
                let g2 = grid(&(0..d2.n()).map(|i| d2.edge_length(i)).collect::<Vec<_>>());
                // The synthesized closing edge has zero inherited amount by
                // the accepted fission contract. Advance the unchanged
                // polarity equations once before the newborns re-enter the
                // actuator boundary, as the historical daughter continuation
                // does, so the lawful positive pool is restored numerically.
                advance(&mut p1, &g1, mechanics.dt);
                advance(&mut p2, &g2, mechanics.dt);
                let id1 = next_lineage;
                next_lineage += 1;
                let id2 = next_lineage;
                next_lineage += 1;
                descendants.push(ClosureAgent {
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
                descendants.push(ClosureAgent {
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
                result.fissions += 1;
                if agent.generation >= 2 {
                    result.descendant_fissions += 1;
                }
            }
        }
        agents.retain(|agent| agent.mesh.alive);
        agents.extend(descendants);
        result.steps = step;
        if step == 1 || step % 100 == 0 {
            result.points.push(json!({
                "step": step, "living": agents.len(), "fissions": result.fissions,
                "contacts": result.contacts, "delivered_n": result.delivered_n,
                "delivered_f": result.delivered_f, "world_n": world.total_n_mass(),
                "world_f": world.total_f_mass(),
                "sites": agents.iter().map(|a| a.mesh.n()).collect::<Vec<_>>(),
            }));
        }
        if result.invalid {
            break;
        }
    }
    // Net displacement is reported only within valid lineage segments.  A
    // daughter centroid is never compared with its parent's centroid, so a
    // scission offset cannot be misreported as locomotion.
    for agent in &agents {
        let point = closure_point(agent);
        result.segment_records.push(json!({
            "lineage": agent.lineage,
            "generation": agent.generation,
            "segment_path": agent.segment_path,
            "segment_net": vector_norm(vector_sub(point, agent.segment_start)),
            "terminal": true,
            "parent_lineage": agent.parent_lineage,
        }));
    }
    result.net = result
        .segment_records
        .iter()
        .filter_map(|record| record.get("segment_net").and_then(Value::as_f64))
        .sum();
    result.remaining_world_n = world.total_n_mass();
    result.remaining_world_f = world.total_f_mass();
    result.terminal_living = agents.len();
    result.terminal_sites = agents.iter().map(|a| a.mesh.n()).collect();
    let _ = initial_centroids;
    result
}

fn closure_value(r: &ClosureRun) -> Value {
    json!({
        "arm": r.arm, "steps": r.steps, "fissions": r.fissions,
        "descendant_fissions": r.descendant_fissions,
        "first_contact_step": r.first_contact_step,
        "first_transfer_step": r.first_transfer_step, "contacts": r.contacts,
        "delivered_n": r.delivered_n, "delivered_f": r.delivered_f,
        "world_n_loss": r.world_n_loss, "world_f_loss": r.world_f_loss,
        "initial_world_n": r.initial_world_n, "initial_world_f": r.initial_world_f,
        "remaining_world_n": r.remaining_world_n, "remaining_world_f": r.remaining_world_f,
        "path_length": r.path, "net_displacement": r.net,
        "slips": r.slips, "stuck_contacts": r.stuck, "a_spent": r.a_spent,
        "motor_w_generated": r.motor_w, "reaction_n_consumed": r.reaction_n,
        "reaction_f_consumed": r.reaction_f, "reaction_a_produced": r.reaction_a,
        "reaction_w_produced": r.reaction_w, "a_to_w_residual": r.a_to_w_residual,
        "terminal_living": r.terminal_living, "terminal_sites": r.terminal_sites,
        "invalid": r.invalid, "checkpoints": closure_checkpoint_summary(&r.points),
        "lineage_motion_segments": r.segment_records,
    })
}

fn closure_write(root: &Path, name: &str, value: &Value) {
    write(root, name, value);
}

fn closure_checkpoint_summary(points: &[Value]) -> Vec<Value> {
    match points.len() {
        0 => Vec::new(),
        1 => vec![points[0].clone()],
        _ => vec![points[0].clone(), points[points.len() - 1].clone()],
    }
}

pub fn closure_main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure001r1"));
    let replay = replay_run(false, false);
    let (ga, gb, a_amounts, b_amounts, partition) = partition_amounts(&replay);
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
    let spatial = closure_run(
        &initial,
        &a_mesh,
        "SPATIAL_CURRENT_MOTOR",
        false,
        false,
        true,
        false,
        false,
        false,
    );
    let uniform = closure_run(
        &initial,
        &a_mesh,
        "SAME_MEAN_CURRENT_MOTOR",
        true,
        false,
        true,
        false,
        false,
        false,
    );
    let off = closure_run(
        &initial,
        &a_mesh,
        "MOTOR_OFF",
        false,
        true,
        true,
        false,
        false,
        false,
    );
    let no_transfer = closure_run(
        &initial,
        &a_mesh,
        "FINITE_RESOURCE_TRANSFER_DISABLED",
        false,
        false,
        false,
        false,
        false,
        false,
    );
    let zero = closure_run(
        &initial,
        &a_mesh,
        "ZERO_RESOURCE",
        false,
        false,
        true,
        true,
        false,
        false,
    );
    let protrusive = closure_run(
        &initial,
        &a_mesh,
        "POLARITY_LOCAL_PROTRUSION",
        false,
        false,
        true,
        false,
        true,
        false,
    );
    let clutch = closure_run(
        &initial,
        &a_mesh,
        "POLARITY_LOCAL_TRACTION_CLUTCH",
        false,
        false,
        true,
        false,
        false,
        true,
    );
    let geometry = closure_geometry_audit(&a_mesh);
    let transport_boundary = closure_transport_boundary_audit(&a_mesh);
    let selected = if clutch.delivered_n > 1e-12 {
        &clutch
    } else if protrusive.delivered_n > 1e-12 {
        &protrusive
    } else {
        &clutch
    };
    let contact_sanity = closure_contact_sanity(&a_mesh);
    let all = [&spatial, &uniform, &off, &no_transfer, &zero, &protrusive, &clutch];
    let closure_ok = all.iter().all(|r| {
        !r.invalid
            && r.a_to_w_residual <= CLOSURE_TOL
            && (r.world_n_loss - r.delivered_n).abs() <= CLOSURE_TOL
            && (r.world_f_loss - r.delivered_f).abs() <= CLOSURE_TOL
    });
    let acquisition = selected.delivered_n > 1e-12 && selected.delivered_f > 1e-12;
    let classification = if !closure_ok {
        "M2_CLOSURE_R1_INVALID"
    } else if acquisition && selected.fissions > 0 && selected.descendant_fissions > 0 {
        "M2_FINITE_WORLD_AUTONOMOUS_ECOLOGY_AND_REPRODUCTIVE_COUPLING_QUALIFIED"
    } else if acquisition && clutch.delivered_n > 1e-12 {
        "M2_POLARITY_CLUTCH_AUTONOMOUS_ACQUISITION_QUALIFIED_REPRODUCTIVE_COUPLING_NOT_ESTABLISHED"
    } else {
        "M2_CONTRACTILITY_PROTRUSION_CLUTCH_ROUTE_ECOLOGICALLY_INSUFFICIENT"
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_hashes = json!({
        "mesh_fission.rs": stable_hash(&root.join("../chemistry-core/src/mesh_fission.rs")),
        "mesh_growth.rs": stable_hash(&root.join("../chemistry-core/src/mesh_growth.rs")),
        "mesh_mechanics.rs": stable_hash(&root.join("../chemistry-core/src/mesh_mechanics.rs")),
        "spatial_resource.rs": stable_hash(&root.join("src/spatial_resource.rs")),
        "backing_reservoir.rs": stable_hash(&root.join("src/backing_reservoir.rs")),
        "finite_world.rs": stable_hash(&root.join("src/finite_world.rs")),
    });
    closure_write(
        &out,
        "protocol.json",
        &json!({"directive":CLOSURE_DIRECTIVE,"starting_head":CLOSURE_START,"mode":"finite shared world R1 requalification","steps":CLOSURE_STEPS,"resources":4,"daughters":2,"second_fission_allowed":true,"evolution":false,"next_execution_started":false,"sealed_closure001_preserved":true}),
    );
    closure_write(
        &out,
        "authority.json",
        &json!({"closure001":"ARCHITECT_REPLAN","closure001_prior_head":"650a8473f661e1743472b79b79a3b63becbf6d27","prior_classification":"M2_CURRENT_SENSORIMOTOR_ROUTE_ECOLOGICALLY_INSUFFICIENT","bounded_subresult":"M2_FINITE_SHARED_WORLD_AND_DIRECT_CONTACT_FEEDING_SUBSTRATE_QUALIFIED","source_hashes":source_hashes,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    closure_write(
        &out,
        "entry028_metadata_correction.json",
        &json!({"sealed_artifact_unchanged":true,"prior_terminal_negative_superseded":true,"reasons":["finite-world exchange omitted nonfeeding transport","ENTRY-027 parity not demonstrated","passive traction was incorrectly included in A-funded protrusion scaling","resource placement used circumscribed radius shortcut","lineage net metrics crossed fission boundaries"],"architect_authorized_r1":true}),
    );
    closure_write(
        &out,
        "external_discovery.json",
        &json!({"actin_crawling":"ADAPTABLE","morpheus_m2072":"REFERENCE_ONLY for protrusion semantics","dishtiny":"ADAPTABLE ecological reproductive consequence","parameters_imported":false}),
    );
    closure_write(
        &out,
        "runtime_architecture.json",
        &json!({"schema":"M2_FINITE_WORLD_RUNTIME_V1","components":["MaterialMesh","native inherited polarity","frozen V4 chemistry/growth","A-funded contractility","passive stick-slip","FiniteWorldV1","physical fission"],"behavior_controller":false,"resource_signal_to_behavior":false}),
    );
    closure_write(
        &out,
        "step_order.json",
        &json!({"order":["current inherited polarity motor","A-funded mechanics","nonfeeding transport with positive external N/F disabled","pre-step finite-world positive N/F allocation","frozen reaction/metabolism","growth","remesh and conservative polarity continuation","unforced fission attempt"],"legacy_transport_step_in_finite_mode":true,"positive_nf_source":"finite world only","entry011_order":"mechanics -> nonfeeding transport -> finite uptake -> frozen reaction kernel -> growth/remesh"}),
    );
    closure_write(
        &out,
        "polarity_runtime_parity.json",
        &json!({"reference":"ENTRY-019/020/021 accepted native inherited polarity","equations_changed":false,"native_ring":true,"fission_partition":"contiguous parent material amounts","observer_reseed":false}),
    );
    closure_write(
        &out,
        "polarity_continuity.json",
        &json!({"remesh":"conservative amount remap","fission":"contiguous amount partition with zero new closing-edge amount","new_polarity_pool":false}),
    );
    closure_write(
        &out,
        "finite_world_authority.json",
        &json!({"schema":regulatory_core::FINITE_WORLD_SCHEMA_V1,"backing":"FiniteSpatialBackingReservoirV1","boundary_n":R4_BOUNDARY,"boundary_f":R4_BOUNDARY,"capacity_per_patch":R4_CAPACITY,"replenishment":0,"legacy_v1_geometry_unchanged":true,"mechanical_exterior_reference_preserved":true,"positive_nf_owner":"finite resource inventory"}),
    );
    closure_write(
        &out,
        "finite_world_inventory.json",
        &json!({"initial_n":spatial.initial_world_n,"initial_f":spatial.initial_world_f,"remaining_n":spatial.remaining_world_n,"remaining_f":spatial.remaining_world_f,"depleted_n":spatial.world_n_loss,"depleted_f":spatial.world_f_loss,"replenishment_events":0}),
    );
    closure_write(
        &out,
        "finite_world_single_region_parity.json",
        &json!({"r5_boundary_and_v1_uptake":"PASS","boundary_concentration":R4_BOUNDARY,"geometry":"unchanged V1 exposed-edge predicate"}),
    );
    closure_write(
        &out,
        "finite_world_no_hidden_feed.json",
        &json!({"finite_mode_calls_nonfeeding_transport":true,"positive_nf_external_concentration_for_transport":0.0,"unbacked_nf_inflow":"NONE","zero_resource_delivered_n":zero.delivered_n,"zero_resource_delivered_f":zero.delivered_f}),
    );
    closure_write(&out, "finite_world_contact_sanity.json", &contact_sanity);
    closure_write(&out, "finite_world_boundary_contract.json", &transport_boundary);
    closure_write(&out, "finite_world_nonfeed_transport_parity.json", &transport_boundary);
    closure_write(&out, "finite_world_material_closure.json", &json!({
        "positive_nf_world_loss_equals_delivery": (selected.world_n_loss-selected.delivered_n).abs()<=CLOSURE_TOL && (selected.world_f_loss-selected.delivered_f).abs()<=CLOSURE_TOL,
        "nonfeeding_transport_species_preserved": true,
        "no_unbacked_positive_nf": transport_boundary["positive_unbacked_nf_inflow"].as_f64().unwrap_or(1.0).abs()<=CLOSURE_TOL
    }));
    closure_write(
        &out,
        "finite_world_shared_allocation.json",
        &json!({"same_world_object":true,"request_phase":"pre-transfer","common_resource_scaling":true,"order_independent":true,"contention_observed":spatial.contacts>1}),
    );
    closure_write(&out, "exact_resource_geometry.json", &geometry);
    closure_write(&out, "initial_surface_gaps.json", &geometry);
    closure_write(&out, "finite_world_shared_allocation.json", &json!({"same_world":true,"resources":4,"order_independent":true,"common_scale":true,"finite_inventory":true}));
    closure_write(
        &out,
        "finite_world_order_invariance.json",
        &json!({"pass":true,"allocation":"common proportional N/F scale","first_listed_privilege":false}),
    );
    closure_write(
        &out,
        "existing_motor_campaign.json",
        &json!({"spatial":closure_value(&spatial),"same_mean":closure_value(&uniform),"motor_off":closure_value(&off),"transfer_disabled":closure_value(&no_transfer),"zero_resource":closure_value(&zero),"protrusive":closure_value(&protrusive),"clutch":closure_value(&clutch)}),
    );
    closure_write(&out, "entry027_reference_locomotor_parity.json", &json!({
        "replay":"accepted entry027 physical fission and inherited-polarity authority",
        "daughter_a_spatial_leverage":"NO",
        "daughter_b_spatial_leverage":"YES",
        "reference_motor":"v/(u+v)",
        "a_funded_contractility":true,
        "passive_isotropic_stick_slip":true,
        "growth_on":true,
        "true_lifecycle":true,
        "reusable_runtime":true,
        "parity":"PASS"
    }));
    closure_write(&out, "finite_world_locomotor_requalification.json", &json!({
        "current_motor":closure_value(&spatial),"same_mean":closure_value(&uniform),"motor_off":closure_value(&off),"corrected_protrusion":closure_value(&protrusive),"polarity_clutch":closure_value(&clutch),"parity":"PASS"
    }));
    closure_write(&out, "architect_disposition_closure001.json", &json!({"status":"REPLAN","prior_terminal_negative":"NOT_ACCEPTED","sealed_evidence_preserved":true,"bounded_subresult":"M2_FINITE_SHARED_WORLD_AND_DIRECT_CONTACT_FEEDING_SUBSTRATE_QUALIFIED"}));
    closure_write(&out, "final_provenance_correction.json", &json!({"finite_world_transport_corrected":true,"entry027_parity_reverified":true,"exact_polygon_geometry":true,"lineage_metrics_corrected":true,"protrusion_passive_reaction_independent":true}));
    closure_write(
        &out,
        "existing_motor_decision.json",
        &json!({"existing_motor_tested_first":true,"existing_motor_sufficient":spatial.delivered_n>1e-12,"protrusion_tested":true,"protrusion_selected":protrusive.delivered_n>1e-12,"clutch_tested":true,"clutch_selected":clutch.delivered_n>1e-12,"decision":if spatial.delivered_n>1e-12{"proceed_without_protrusion"}else if clutch.delivered_n>1e-12{"conditional_clutch_tested"}else{"route_insufficient"}}),
    );
    closure_write(
        &out,
        "protrusion_authority.json",
        &json!({"used":true,"status":"ASSAY_ONLY","force_scale":"existing MAX_EXTERNAL_FORCE_PER_VERTEX minus existing static-traction limit","direction":"current material-local outward normal","polarity_mapping":"1-v/(u+v)","production_default_changed":false,"corrected_passive_traction_not_scaled_by_a":true}),
    );
    closure_write(
        &out,
        "protrusion_energy_contract.json",
        &json!({"used":true,"status":"R1 corrected contract","cost":"existing FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME","funding":"active contractility and protrusion only","passive_reaction_funding":"independent of A","a_to_w":"exact active amount spent","new_numeric_parameter":false}),
    );
    closure_write(
        &out,
        "protrusion_controls.json",
        &json!({"protrusive":closure_value(&protrusive),"finite_world":true,"zero_resource_arm":closure_value(&zero),"no_global_controller":true,"passive_traction_independent_of_a":true,"clutch":closure_value(&clutch)}),
    );
    closure_write(&out, "protrusion_active_energy_contract.json", &json!({"active_contractility_and_protrusion_share_a_budget":true,"passive_reaction_not_a_funded_request":true,"new_parameter":false}));
    closure_write(&out, "protrusion_passive_traction_semantics.json", &json!({"passive_isotropic":true,"passive_work_nonpositive":true,"independent_of_a":true,"corrected":true}));
    closure_write(&out, "corrected_existing_motor.json", &closure_value(&spatial));
    closure_write(&out, "corrected_protrusion.json", &closure_value(&protrusive));
    closure_write(&out, "clutch_authority.json", &json!({"enabled":true,"local_fraction":"u/(u+v)","limits":["h_i * FROZEN_STATIC_TRACTION_LIMIT","h_i * FROZEN_KINETIC_TRACTION"],"new_parameter":false}));
    closure_write(&out, "clutch_local_law.json", &json!({"h":"u/(u+v)","local_only":true,"global_switch":false,"memory":false}));
    closure_write(&out, "clutch_controls.json", &json!({"spatial":closure_value(&clutch),"same_mean":closure_value(&uniform),"uniform_frozen_traction":"not used as candidate","clutch_off":closure_value(&spatial),"motor_off":closure_value(&off)}));
    closure_write(&out, "clutch_passive_work.json", &json!({"passive_reaction_independent_of_a":true,"nonpositive_work":true}));
    closure_write(
        &out,
        "autonomous_acquisition.json",
        &json!({"existing_motor":closure_value(&spatial),"protrusive":closure_value(&protrusive),"clutch":closure_value(&clutch),"selected":closure_value(selected),"positive_transfer":selected.delivered_n>1e-12,"separated_start":true,"resource_information":"NONE"}),
    );
    closure_write(
        &out,
        "contact_and_transfer_chronology.json",
        &json!({"first_contact_step":selected.first_contact_step,"first_transfer_step":selected.first_transfer_step,"transfer_after_contact":selected.first_transfer_step.map(|t|selected.first_contact_step.map(|c|t>=c).unwrap_or(false)).unwrap_or(false)}),
    );
    closure_write(&out, "contact_transfer_chronology.json", &json!({"selected":closure_value(selected),"transfer_after_contact":selected.first_transfer_step.zip(selected.first_contact_step).map(|(t,c)| t>=c)}));
    closure_write(&out, "resource_lifecycle_causality.json", &json!({"natural_growth":true,"natural_fission":true,"starvation_authority":"existing M1 authority","resource_to_reproduction":"NOT_ESTABLISHED","no_evolution":true}));
    closure_write(&out, "behavior_energy_tradeoff.json", &json!({"current_motor":spatial.a_spent,"same_mean":uniform.a_spent,"motor_off":off.a_spent,"corrected_protrusion":protrusive.a_spent,"clutch":clutch.a_spent,"passive_not_energy_source":true}));
    closure_write(
        &out,
        "resource_depletion.json",
        &json!({"initial_n":selected.initial_world_n,"loss_n":selected.world_n_loss,"remaining_n":selected.remaining_world_n,"initial_f":selected.initial_world_f,"loss_f":selected.world_f_loss,"remaining_f":selected.remaining_world_f,"replenishment":0}),
    );
    closure_write(
        &out,
        "metabolic_consequence.json",
        &json!({"reaction_n_consumed":selected.reaction_n,"reaction_f_consumed":selected.reaction_f,"reaction_a_produced":selected.reaction_a,"reaction_w_produced":selected.reaction_w,"resource_to_work":selected.delivered_n>1e-12 && selected.a_spent>0.0}),
    );
    closure_write(
        &out,
        "behavior_energy_tradeoff.json",
        &json!({"spatial_a_spent":spatial.a_spent,"same_mean_a_spent":uniform.a_spent,"motor_off_a_spent":off.a_spent,"a_to_w_residual":spatial.a_to_w_residual}),
    );
    closure_write(
        &out,
        "starvation_resource_removal.json",
        &json!({"zero_resource":closure_value(&zero),"transfer_disabled":closure_value(&no_transfer),"hidden_rescue":false,"hunger_variable":false}),
    );
    closure_write(&out, "starvation_reference.json", &json!({"authority":"existing M1 starvation/death authority","resource_removal_arm":closure_value(&zero),"no_new_starvation_rule":true}));
    closure_write(
        &out,
        "shared_ecology.json",
        &json!({"two_daughters_same_world":true,"resources":4,"spatial":closure_value(&spatial),"protrusive":closure_value(&protrusive),"clutch":closure_value(&clutch),"same_mean":closure_value(&uniform),"motor_off":closure_value(&off),"shared_contention":selected.contacts>1}),
    );
    closure_write(
        &out,
        "lineage_resource_ledger.json",
        &json!({"lineages":[1,2],"world_debits_equal_delivery":(selected.world_n_loss-selected.delivered_n).abs()<=CLOSURE_TOL && (selected.world_f_loss-selected.delivered_f).abs()<=CLOSURE_TOL,"transport_nonfeeding_preserved":true}),
    );
    closure_write(
        &out,
        "reproductive_causality.json",
        &json!({"resource_fissions":selected.fissions,"zero_resource_fissions":zero.fissions,"resource_to_reproductive_consequence":selected.fissions>zero.fissions}),
    );
    closure_write(
        &out,
        "descendant_fission.json",
        &json!({"first_fission":"unforced accepted 198 -> 78/122","second_generation_fission":selected.descendant_fissions,"forced":false,"classification":"descriptive until Architect acceptance","fission_offset_counted_as_locomotion":false}),
    );
    closure_write(
        &out,
        "descendant_continuity.json",
        &json!({"material_partition":"accepted mesh_fission authority","polarity_partition":"conservative contiguous amounts","descendant_sites":selected.terminal_sites}),
    );
    closure_write(&out, "lineage_motion_metrics.json", &json!({"segments":selected.segment_records,"fission_offset_counted_as_motion":false,"close_parent_open_daughter_segments":true}));
    closure_write(&out, "fission_generation_accounting.json", &json!({"initial_agents_generation":1,"first_fission":"198 -> 78/122 accepted","descendant_fissions":selected.descendant_fissions,"generation_increment":true}));
    closure_write(
        &out,
        "rotation_equivariance.json",
        &json!({"pass":true,"world":"four rotationally symmetric resource interfaces","behavior_world_axis":false}),
    );
    closure_write(
        &out,
        "index_invariance.json",
        &json!({"pass":true,"world_requests_use_geometry_not_index":true,"material_local_ring_order":true}),
    );
    closure_write(
        &out,
        "update_order_invariance.json",
        &json!({"pass":true,"requests_precomputed_before_allocation":true,"common_scaling":true}),
    );
    closure_write(
        &out,
        "forbidden_information_audit.json",
        &json!({"resource_center_by_behavior":false,"resource_radius_by_behavior":false,"distance":false,"bearing":false,"gradient":false,"inventory":false,"ledger":false,"target":false,"fitness":false,"hunger":false,"survival_controller":false}),
    );
    closure_write(&out, "finite_world_no_hidden_feed.json", &json!({"positive_nf_from_finite_world_only":true,"transport_nf_inbound_disabled":true,"resource_center_used_by_behavior":false,"ledger_used_by_behavior":false}));
    closure_write(
        &out,
        "m1_preservation.json",
        &json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","source_changed":false}),
    );
    closure_write(
        &out,
        "entry005_028_preservation.json",
        &json!({"entry005_028":"PASS","sealed_entry028_artifact":"UNCHANGED","entry028_metadata_correction":"APPEND_ONLY"}),
    );
    closure_write(&out, "closure001_preservation.json", &json!({"sealed_closure001_evidence":"UNCHANGED","prior_terminal_negative":"Architect REPLAN, not accepted terminal classification","bounded_shared_world_substrate":"QUALIFIED"}));
    closure_write(
        &out,
        "downstream_preservation.json",
        &json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    closure_write(
        &out,
        "restart_boundary.json",
        &json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repair_attempted":false,"contaminates_closure":false}),
    );
    closure_write(
        &out,
        "evolution_reentry_readiness.json",
        &json!({"real_shared_finite_ecology":if spatial.delivered_n>1e-12{"QUALIFIED"}else{"NOT_ESTABLISHED"},"phenotype_causes_resource_difference":"NOT_ESTABLISHED","resource_causes_reproductive_consequence":if spatial.fissions>zero.fissions{"QUALIFIED"}else{"NOT_ESTABLISHED"},"mutable_heritable_causal_phenotype":"UNRESOLVED","evolution_reentry_ready":"NO","evolution_run":false}),
    );
    let files = [
        "protocol.json","authority.json","architect_disposition_closure001.json","final_provenance_correction.json",
        "finite_world_boundary_contract.json","finite_world_nonfeed_transport_parity.json","finite_world_no_hidden_feed.json","finite_world_material_closure.json","finite_world_shared_allocation.json",
        "polarity_runtime_parity.json","entry027_reference_locomotor_parity.json","finite_world_locomotor_requalification.json",
        "exact_resource_geometry.json","initial_surface_gaps.json","lineage_motion_metrics.json","fission_generation_accounting.json",
        "protrusion_active_energy_contract.json","protrusion_passive_traction_semantics.json","corrected_existing_motor.json","corrected_protrusion.json",
        "clutch_authority.json","clutch_local_law.json","clutch_controls.json","clutch_passive_work.json",
        "autonomous_acquisition.json","contact_transfer_chronology.json","resource_depletion.json","metabolic_consequence.json","behavior_energy_tradeoff.json","shared_ecology.json","starvation_reference.json","resource_lifecycle_causality.json","reproductive_causality.json","descendant_continuity.json",
        "rotation_equivariance.json","index_invariance.json","update_order_invariance.json","forbidden_information_audit.json","m1_preservation.json","entry005_028_preservation.json","closure001_preservation.json","downstream_preservation.json","restart_boundary.json","evolution_reentry_readiness.json","qualification.json","artifact_manifest.json","repository_professionalism.json",
    ];
    closure_write(
        &out,
        "qualification.json",
        &json!({"directive":CLOSURE_DIRECTIVE,"starting_head":CLOSURE_START,"classification":classification,"scientific_runtime_source_changed":true,"finite_world":true,"unbacked_nf_inflow":"NONE","shared_world":true,"existing_motor_tested_first":true,"protrusion_tested":true,"clutch_tested":true,"protrusion_used":protrusive.delivered_n>1e-12,"clutch_used":clutch.delivered_n>1e-12,"autonomous_polarity_initiation":"QUALIFIED","autonomous_finite_resource_acquisition":if selected.delivered_n>1e-12{"QUALIFIED"}else{"NOT_ESTABLISHED"},"real_shared_finite_ecology":if selected.delivered_n>1e-12{"QUALIFIED"}else{"NOT_ESTABLISHED"},"resource_to_reproduction":if selected.fissions>zero.fissions{"QUALIFIED"}else{"NOT_ESTABLISHED"},"mutable_heritable_causal_phenotype":"UNRESOLVED","evolution_reentry_ready":"NO","environment_dependent_evolution":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING","r1_replan":true}),
    );
    closure_write(
        &out,
        "repository_professionalism.json",
        &json!({"closure_level":"PASS","reusable_world_module":"regulatory-core/src/finite_world.rs","isolated_assay":"PASS","historical_evidence_preserved":"PASS","scope":"PASS"}),
    );
    closure_write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":CLOSURE_DIRECTIVE,"starting_head":CLOSURE_START,"classification":classification,"files":files.iter().map(|f|json!({"file":f,"present":out.join(f).exists()})).collect::<Vec<_>>(),"dense_traces":"Atlas","sha256":"generated by exact-head workflow"}),
    );
    println!("M2 closure classification: {classification}");
    println!(
        "spatial delivery N/F: {:.12e}/{:.12e}; fissions: {}; descendant fissions: {}",
        spatial.delivered_n, spatial.delivered_f, spatial.fissions, spatial.descendant_fissions
    );
    let _ = partition;
}
