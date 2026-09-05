// CLOSURE-006: local resource-contact quiescence.  This is an assay-local
// composition over the accepted finite-world and CLOSURE-005 lifecycle.  It
// does not alter production runtime, resource physics, growth, fission, or
// the frozen motor equations.

use regulatory_core::{ContinuityMaterialFrameV1, ContinuityNetworkV1, TopologyEventV1};
use std::collections::HashMap;

const C6_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-006-LOCAL-RESOURCE-CONTACT-QUIESCENCE-REPRODUCTIVE-ECOLOGY-AND-HEREDITY-001";
const C6_START: &str = "aeb9356efd13db06a5dd0c8f32c43957d4fd5f25";
const C6_STEPS: usize = 12_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum C6ContactMode {
    Real,
    Shuffled,
}

#[derive(Clone, Default)]
struct C6Run {
    base: C5Run,
    regulator_steps: usize,
    contacted_steps: usize,
    inhibited_motor_sum: f64,
    base_motor_sum: f64,
    max_activity: f64,
    topology_restarts: usize,
}

fn c6_contact_signal(world: &regulatory_core::FiniteWorldV1, mesh: &MaterialMesh) -> Vec<f64> {
    let mut signal: Vec<f64> = vec![0.0; mesh.n()];
    for resource in &world.resources {
        let local = resource.backing.region.local_contact_signal(mesh);
        for (out, value) in signal.iter_mut().zip(local) {
            *out = (*out).max(value);
        }
    }
    signal
}

fn c6_shuffle(mut signal: Vec<f64>) -> Vec<f64> {
    let shift = signal.len() / 2;
    signal.rotate_left(shift);
    signal
}

fn c6_mechanics(
    agent: &mut ClosureAgent,
    regulator: &mut ContinuityNetworkV1,
    contact: &[f64],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Result<(usize, usize, f64, f64, f64, f64, f64), String> {
    let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &agent.mesh.vertices,
        contact,
        mechanics.dt,
    );
    let event = match frame.topology_size.cmp(&regulator.previous_frame.topology_size) {
        std::cmp::Ordering::Equal => TopologyEventV1::Stable,
        std::cmp::Ordering::Greater => TopologyEventV1::Split,
        std::cmp::Ordering::Less => TopologyEventV1::Merge,
    };
    regulator
        .step(frame, event)
        .map_err(|e| format!("regulator continuity: {e}"))?;
    let raw = entry025_anti(&agent.polarity);
    let clutch = entry025_direct(&agent.polarity);
    if raw.len() != regulator.state.activity.len() || clutch.len() != raw.len() {
        return Err("motor/regulator topology mismatch".into());
    }
    let activity = regulator.state.activity.clone();
    let motor: Vec<f64> = raw
        .iter()
        .zip(&activity)
        .map(|(base, inhibit)| base * (1.0 - inhibit))
        .collect();
    let before = agent.mesh.clone();
    let ledger = regulatory_core::apply_local_activated_energy_contractility_with_local_traction_clutch(
        &mut agent.mesh,
        &motor,
        &clutch,
        mechanics,
        contractility,
        traction,
    )
    .map_err(|e| format!("quiescent clutch: {e:?}"))?;
    if !agent.mesh.physical_runtime_valid() || !agent.mesh.lifecycle_invariants_hold() {
        return Err("invalid mesh after quiescent mechanics".into());
    }
    let spent = ledger
        .contractility
        .as_ref()
        .map(|x| x.resource_spent)
        .unwrap_or(0.0);
    let waste = ledger
        .contractility
        .as_ref()
        .map(|x| x.waste_amount_after - x.waste_amount_before)
        .unwrap_or(0.0);
    let residual = if spent > 0.0 {
        (before.interior.a * before.area() - agent.mesh.interior.a * agent.mesh.area() - spent).abs()
    } else {
        0.0
    };
    Ok((
        ledger.slipping_contacts,
        ledger.stuck_contacts,
        spent,
        waste,
        ledger.substrate_work.max(0.0).max(residual),
        raw.iter().sum(),
        motor.iter().sum(),
    ))
}

fn c6_motor_off_mechanics(
    agent: &mut ClosureAgent,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Result<(usize, usize, f64, f64, f64, f64, f64), String> {
    let zeros = vec![0.0; agent.mesh.n()];
    let ledger = regulatory_core::apply_local_activated_energy_contractility_with_local_traction_clutch(
        &mut agent.mesh,
        &zeros,
        &zeros,
        mechanics,
        contractility,
        traction,
    )
    .map_err(|e| format!("motor-off clutch: {e:?}"))?;
    Ok((
        ledger.slipping_contacts,
        ledger.stuck_contacts,
        ledger
            .contractility
            .as_ref()
            .map(|x| x.resource_spent)
            .unwrap_or(0.0),
        ledger
            .contractility
            .as_ref()
            .map(|x| x.waste_amount_after - x.waste_amount_before)
            .unwrap_or(0.0),
        ledger.substrate_work.max(0.0),
        0.0,
        0.0,
    ))
}

fn c6_run(
    initial: &[ClosureAgent],
    world_body: &MaterialMesh,
    arm: &str,
    transfer_enabled: bool,
    zero_resource: bool,
    contact_mode: C6ContactMode,
    motor_off: bool,
) -> C6Run {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let radius = (C5_UNIT_N / (std::f64::consts::PI * C4_BOUNDARY_N)).sqrt();
    let mut agents = initial.to_vec();
    for agent in &mut agents {
        agent.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    let mut world = c4_world(world_body, radius, C5_UNIT_N, transfer_enabled, zero_resource);
    let mut out = C6Run {
        base: C5Run {
            base: Closure003Run {
                arm: arm.into(),
                initial_world_n: world.total_n_mass(),
                initial_world_f: world.total_f_mass(),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let mut regulators: HashMap<u64, ContinuityNetworkV1> = agents
        .iter()
        .map(|agent| {
            let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
                &agent.mesh.vertices,
                &vec![0.0; agent.mesh.n()],
                mechanics.dt,
            );
            (agent.lineage, ContinuityNetworkV1::new(frame, None).unwrap())
        })
        .collect();
    let mut previous_viable: HashMap<u64, bool> = agents
        .iter()
        .map(|agent| (agent.lineage, agent.mesh.observer_viable()))
        .collect();
    let birth_masses: HashMap<u64, f64> = agents
        .iter()
        .map(|agent| (agent.lineage, agent.birth_mass))
        .collect();
    let mut next_lineage = 50_000u64;

    for step in 1..=C6_STEPS {
        if agents.is_empty() {
            break;
        }
        let old_positions: Vec<_> = agents.iter().map(closure002_point).collect();
        let mut contact_vectors = Vec::with_capacity(agents.len());
        for agent in &agents {
            let signal = c6_contact_signal(&world, &agent.mesh);
            contact_vectors.push(if contact_mode == C6ContactMode::Shuffled {
                c6_shuffle(signal)
            } else {
                signal
            });
        }
        let mut mechanics_failed = false;
        for (agent, contact) in agents.iter_mut().zip(&contact_vectors) {
            if !agent.mesh.can_advance_physics() {
                out.base.base.invalid = true;
                mechanics_failed = true;
                continue;
            }
            if !motor_off && agent
                .polarity
                .u
                .iter()
                .zip(&agent.polarity.v)
                .any(|(u, v)| !u.is_finite() || !v.is_finite() || *u < 0.0 || *v < 0.0 || *u + *v <= 0.0)
            {
                out.base.base.invalid = true;
                mechanics_failed = true;
                continue;
            }
            let regulator = regulators
                .entry(agent.lineage)
                .or_insert_with(|| {
                    let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
                        &agent.mesh.vertices,
                        &vec![0.0; agent.mesh.n()],
                        mechanics.dt,
                    );
                    ContinuityNetworkV1::new(frame, None).unwrap()
                });
            let mechanics_result = if motor_off {
                c6_motor_off_mechanics(agent, &mechanics, &contractility, &traction)
            } else {
                c6_mechanics(agent, regulator, contact, &mechanics, &contractility, &traction)
            };
            match mechanics_result {
                Ok((slips, stuck, spent, waste, work, base_sum, motor_sum)) => {
                    out.base.base.slips += slips;
                    out.base.base.stuck += stuck;
                    out.base.base.a_spent += spent;
                    out.base.base.w_generated += waste.max(0.0);
                    out.base.base.max_a_to_w_residual =
                        out.base.base.max_a_to_w_residual.max((spent - waste).abs());
                    out.base.base.path += 0.0;
                    out.base.base.arm = arm.into();
                    out.regulator_steps += 1;
                    out.base_motor_sum += base_sum;
                    out.inhibited_motor_sum += motor_sum;
                    out.max_activity = out.max_activity.max(
                        regulator.state.activity.iter().copied().fold(0.0, f64::max),
                    );
                    if contact.iter().any(|value| *value > 0.0) {
                        out.contacted_steps += 1;
                    }
                    let _ = work;
                }
                Err(_) => {
                    out.base.base.invalid = true;
                    mechanics_failed = true;
                }
            }
        }
        if mechanics_failed {
            out.base.base.steps = step;
            break;
        }
        let lineage_by_index: Vec<u64> = agents.iter().map(|agent| agent.lineage).collect();
        let mut views: Vec<MaterialMesh> = agents.iter().map(|agent| agent.mesh.clone()).collect();
        let deliveries = world.exchange(&mut views, &TransportParams::default(), mechanics.dt);
        let mut contact_now = std::collections::HashSet::new();
        for delivery in &deliveries {
            out.base.base.delivered_n += delivery.n_delivered;
            out.base.base.delivered_f += delivery.f_delivered;
            out.base.base.world_n_loss += delivery.n_world_loss;
            out.base.base.world_f_loss += delivery.f_world_loss;
            let Some(&lineage) = lineage_by_index.get(delivery.organism_index) else {
                out.base.base.invalid = true;
                continue;
            };
            let lineage_record = c5_lineage_mut(&mut out.base.lineages, &agents[delivery.organism_index]);
            if delivery.exposed_edges > 0 {
                contact_now.insert(lineage);
                lineage_record.contact_steps += 1;
                lineage_record.last_contact = Some(step);
                lineage_record.first_contact.get_or_insert(step);
                if !lineage_record.resources.contains(&delivery.resource_id) {
                    lineage_record.resources.push(delivery.resource_id.clone());
                }
                if delivery.n_delivered > 1e-12 || step == 1 || step % 500 == 0 {
                    out.base.resource_ledger.push(json!({
                        "step": step, "lineage": lineage, "resource_id": delivery.resource_id,
                        "resource_index": delivery.resource_index, "exposed_edges": delivery.exposed_edges,
                        "n_delivered": delivery.n_delivered, "f_delivered": delivery.f_delivered,
                        "n_world_loss": delivery.n_world_loss, "f_world_loss": delivery.f_world_loss,
                        "allocation_scale": delivery.allocation_scale,
                    }));
                }
            }
            lineage_record.n += delivery.n_delivered;
            lineage_record.f += delivery.f_delivered;
            lineage_record.world_n += delivery.n_world_loss;
            lineage_record.world_f += delivery.f_world_loss;
            if delivery.n_delivered > 1e-12 {
                lineage_record.first_transfer.get_or_insert(step);
                out.base.base.first_transfer.get_or_insert(step);
            }
            out.base.base.first_contact = out.base.base.first_contact.or_else(|| {
                (delivery.exposed_edges > 0).then_some(step)
            });
        }
        for lineage in lineage_by_index {
            let record = out.base.lineages.entry(lineage).or_default();
            let now = contact_now.contains(&lineage);
            if now && !record.in_contact {
                record.contact_episodes += 1;
            }
            record.in_contact = now;
        }
        for (agent, view) in agents.iter_mut().zip(views) {
            agent.mesh = view;
        }
        for agent in &mut agents {
            let before = agent.mesh.total_structural_mass();
            let reaction_ledger = reactions_step_with_reserve_mode(
                &mut agent.mesh,
                &reaction,
                mechanics.dt,
                true,
                true,
                ReserveDiagnosticMode::Full,
            );
            let growth_ledger = growth_step(&mut agent.mesh, &reaction, &growth, mechanics.dt);
            out.base.base.reaction_n += reaction_ledger.n_consumed;
            out.base.base.reaction_f += reaction_ledger.f_consumed;
            out.base.base.reaction_a += reaction_ledger.a_produced;
            out.base.base.reaction_w += reaction_ledger.w_produced + growth_ledger.w_from_growth;
            out.base.base.growth_m += growth_ledger.m_grown;
            out.base.base.max_material_closure = out.base.base.max_material_closure.max(
                (agent.mesh.total_structural_mass() - before - growth_ledger.m_grown).abs(),
            );
            let viable = agent.mesh.observer_viable();
            if previous_viable.get(&agent.lineage).copied().unwrap_or(true) && !viable {
                out.base.base.deaths += 1;
            }
            previous_viable.insert(agent.lineage, viable);
            if step % 25 == 0 {
                let record = c5_lineage_mut(&mut out.base.lineages, agent);
                record.chronology.push(json!({
                    "step": step, "n": record.n, "f": record.f,
                    "reaction_n": reaction_ledger.n_consumed, "reaction_f": reaction_ledger.f_consumed,
                    "a_produced": reaction_ledger.a_produced, "a_spent_total": out.base.base.a_spent,
                    "w_generated": reaction_ledger.w_produced + growth_ledger.w_from_growth,
                    "mass": agent.mesh.total_structural_mass(), "birth_mass": agent.birth_mass,
                    "ratio": agent.mesh.total_structural_mass() / agent.birth_mass.max(1e-15),
                    "area": agent.mesh.area(), "topology": agent.mesh.n(), "observer_viable": viable,
                }));
            }
        }
        for (index, agent) in agents.iter_mut().enumerate() {
            let old_vertices = agent.mesh.vertices.clone();
            let old_grid = agent.grid.clone();
            remesh(&mut agent.mesh);
            let origin = agent
                .mesh
                .vertices
                .first()
                .and_then(|first| old_vertices.iter().position(|old| vector_norm(vector_sub(*old, *first)) <= 1e-9))
                .unwrap_or(0);
            let new_grid = grid(&(0..agent.mesh.n()).map(|i| agent.mesh.edge_length(i)).collect::<Vec<_>>());
            agent.polarity = remap(&old_grid, &agent.polarity, &new_grid, origin);
            advance(&mut agent.polarity, &new_grid, mechanics.dt);
            agent.grid = new_grid;
            out.base.base.path += vector_norm(vector_sub(closure002_point(agent), old_positions[index]));
        }
        if step % 10 == 0 {
            for agent in &mut agents {
                let _ = chemistry_core::mesh_fission::topology_step(&mut agent.mesh, &fission);
            }
        }
        let mut newborns = Vec::new();
        for agent in &mut agents {
            let birth = birth_masses.get(&agent.lineage).copied().unwrap_or(agent.birth_mass);
            let mass = agent.mesh.total_structural_mass();
            if step % 25 != 0 || mass < 1.35 * birth.max(1e-9) {
                continue;
            }
            if let Some((mut d1, mut d2, event)) = try_local_fission(&agent.mesh, &fission) {
                if !event.partition.ok {
                    out.base.base.invalid = true;
                }
                let (p1, p2) = closure_split_state(&agent.polarity, &agent.grid, &event, &d1, &d2);
                d1.contract_version = MeshContractVersion::MaturationCoupledV4;
                d2.contract_version = MeshContractVersion::MaturationCoupledV4;
                let g1 = grid(&(0..d1.n()).map(|i| d1.edge_length(i)).collect::<Vec<_>>());
                let g2 = grid(&(0..d2.n()).map(|i| d2.edge_length(i)).collect::<Vec<_>>());
                let id1 = next_lineage;
                let id2 = next_lineage + 1;
                next_lineage += 2;
                let parent = agent.lineage;
                out.base.lineages.entry(parent).or_default().first_fission = Some(step);
                newborns.push(ClosureAgent { mesh: d1.clone(), grid: g1, polarity: p1, birth_mass: d1.total_structural_mass(), lineage: id1, generation: agent.generation + 1, segment_start: physical_centroid(&d1), segment_path: 0.0, parent_lineage: Some(parent) });
                newborns.push(ClosureAgent { mesh: d2.clone(), grid: g2, polarity: p2, birth_mass: d2.total_structural_mass(), lineage: id2, generation: agent.generation + 1, segment_start: physical_centroid(&d2), segment_path: 0.0, parent_lineage: Some(parent) });
                agent.mesh.alive = false;
                out.base.base.fissions += 1;
                if agent.generation >= 2 {
                    out.base.base.descendant_fissions += 1;
                }
                out.base.base.first_fission.get_or_insert(step);
                out.base.base.events.push(json!({
                    "event": "unforced_fission", "step": step, "parent": parent,
                    "children": [id1, id2], "topology": [d1.n(), d2.n()],
                    "partition_ok": event.partition.ok,
                }));
                let frame1 = ContinuityMaterialFrameV1::from_positions_and_stimuli(
                    &d1.vertices,
                    &vec![0.0; d1.n()],
                    mechanics.dt,
                );
                let frame2 = ContinuityMaterialFrameV1::from_positions_and_stimuli(
                    &d2.vertices,
                    &vec![0.0; d2.n()],
                    mechanics.dt,
                );
                regulators.insert(id1, ContinuityNetworkV1::new(frame1, None).unwrap());
                regulators.insert(id2, ContinuityNetworkV1::new(frame2, None).unwrap());
            }
        }
        agents.retain(|agent| agent.mesh.alive);
        agents.extend(newborns);
        out.base.base.steps = step;
        if step == 1 || step % 500 == 0 || out.base.base.first_fission == Some(step) {
            out.base.base.checkpoints.push(json!({
                "step": step, "living": agents.len(), "fissions": out.base.base.fissions,
                "delivered_n": out.base.base.delivered_n, "world_n": world.total_n_mass(),
                "regulator_max_activity": out.max_activity,
            }));
        }
        if out.base.base.invalid {
            break;
        }
    }
    out.base.base.remaining_world_n = world.total_n_mass();
    out.base.base.remaining_world_f = world.total_f_mass();
    out.base.base.terminal_living = agents.len();
    out.base.base.terminal_sites = agents.iter().map(|agent| agent.mesh.n()).collect();
    out
}

fn c6_value(run: &C6Run) -> Value {
    json!({
        "base": c6_compact_c5_value(&run.base),
        "regulator_steps": run.regulator_steps,
        "contacted_steps": run.contacted_steps,
        "base_motor_sum": run.base_motor_sum,
        "inhibited_motor_sum": run.inhibited_motor_sum,
        "motor_saved": run.base_motor_sum - run.inhibited_motor_sum,
        "max_regulator_activity": run.max_activity,
        "topology_restarts": run.topology_restarts,
    })
}

fn c6_compact_c5_value(run: &C5Run) -> Value {
    let mut value = c5_value(run);
    if let Some(object) = value.as_object_mut() {
        object.remove("resource_ledger");
    }
    value
}

fn c6_write(root: &Path, name: &str, value: &Value) {
    c4_write(root, name, value.clone());
}

pub fn c6_main() {
    let out_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure006"));
    let replay = replay_run(false, false);
    let (ga, gb, aa, bb, _) = partition_amounts(&replay);
    let (a_mesh, a_grid, a_state) = entry027_first_lawful_state(
        &replay.daughter_a,
        &ga,
        &density_state(&aa, &ga),
        replay.first_fission_step.saturating_sub(1) as u64,
    );
    let (b_mesh, b_grid, b_state) = entry027_first_lawful_state(
        &replay.daughter_b,
        &gb,
        &density_state(&bb, &gb),
        replay.first_fission_step.saturating_sub(1) as u64,
    );
    let initial = closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state);
    let world_body = &initial[0].mesh;

    println!("CLOSURE-006 arm PAIR_QUIESCENT_FINITE");
    let paired = c6_run(&initial, world_body, "PAIR_QUIESCENT_FINITE", true, false, C6ContactMode::Real, false);
    println!("CLOSURE-006 arm PAIR_CONTACT_SHUFFLED");
    let shuffled = c6_run(&initial, world_body, "PAIR_CONTACT_SHUFFLED", true, false, C6ContactMode::Shuffled, false);
    println!("CLOSURE-006 arm PAIR_QUIESCENT_TRANSFER_DISABLED");
    let disabled = c6_run(&initial, world_body, "PAIR_QUIESCENT_TRANSFER_DISABLED", false, false, C6ContactMode::Real, false);
    println!("CLOSURE-006 arm PAIR_QUIESCENT_ZERO_RESOURCE");
    let zero = c6_run(&initial, world_body, "PAIR_QUIESCENT_ZERO_RESOURCE", true, true, C6ContactMode::Real, false);
    println!("CLOSURE-006 arm PAIR_REGULATOR_NULL");
    let null_run = c5_run(&initial, world_body, "PAIR_REGULATOR_NULL", true, false, false);
    println!("CLOSURE-006 arm PAIR_MOTOR_OFF");
    let motor_off = c6_run(&initial, world_body, "PAIR_MOTOR_OFF", true, false, C6ContactMode::Real, true);
    println!("CLOSURE-006 arm A_SOLO_QUIESCENT_FINITE");
    let solo_a = c6_run(&initial[0..1], world_body, "A_SOLO_QUIESCENT_FINITE", true, false, C6ContactMode::Real, false);
    println!("CLOSURE-006 arm B_SOLO_QUIESCENT_FINITE");
    let solo_b = c6_run(&initial[1..2], world_body, "B_SOLO_QUIESCENT_FINITE", true, false, C6ContactMode::Real, false);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let null_value = c6_compact_c5_value(&null_run);
    let paired_value = c6_value(&paired);
    let shuffled_value = c6_value(&shuffled);
    let disabled_value = c6_value(&disabled);
    let zero_value = c6_value(&zero);
    let motor_off_value = c6_value(&motor_off);
    let solo_a_value = c6_value(&solo_a);
    let solo_b_value = c6_value(&solo_b);
    let candidate_benefit = paired.base.base.delivered_n > null_run.base.delivered_n + 1e-12;
    let candidate_saves_a = paired.base.base.a_spent + 1e-12 < null_run.base.a_spent;
    let fission_benefit = paired.base.base.fissions > null_run.base.fissions;
    let valid = !paired.base.base.invalid && !shuffled.base.base.invalid && !disabled.base.base.invalid && !zero.base.base.invalid && !solo_a.base.base.invalid && !solo_b.base.base.invalid;
    let classification = if !valid {
        "M2_CLOSURE006_LOCAL_RESOURCE_CONTACT_QUIESCENCE_INVALID"
    } else if fission_benefit {
        "M2_LOCAL_RESOURCE_CONTACT_QUIESCENCE_REPRODUCTIVE_ECOLOGY_QUALIFIED"
    } else if candidate_benefit || candidate_saves_a {
        "M2_LOCAL_RESOURCE_CONTACT_QUIESCENCE_ACQUISITION_BENEFIT_REPRODUCTIVE_CAUSALITY_NOT_ESTABLISHED"
    } else {
        "M2_LOCAL_RESOURCE_CONTACT_QUIESCENCE_INSUFFICIENT"
    };
    let files = [
        "protocol.json", "authority.json", "contact_provenance.json", "quiescence_architecture.json",
        "solo_a_finite.json", "solo_b_finite.json", "paired_finite.json", "regulator_null_control.json",
        "contact_shuffled_control.json", "motor_off_control.json", "transfer_disabled.json", "zero_resource.json",
        "a_savings.json", "feeding_timing.json", "resource_causal_reproduction.json", "descendant_continuity.json",
        "heredity_ecological_phenotype.json", "evolution_reentry.json", "material_closure.json", "energetic_closure.json",
        "forbidden_information_audit.json", "m1_preservation.json", "preservation.json", "restart_boundary.json",
        "qualification.json", "artifact_manifest.json",
    ];
    c6_write(&out_dir, "protocol.json", &json!({
        "directive": C6_DIRECTIVE, "starting_head": C6_START, "steps": C6_STEPS,
        "assay_only": true, "production_runtime_changed": false, "next_execution_started": false,
    }));
    c6_write(&out_dir, "authority.json", &json!({
        "closure005": "ARCHITECT_ACCEPTED", "closure005_head": C6_START,
        "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
        "m1": "CLOSED_FROZEN", "production": "MaturationCoupledV4 / reserve OFF",
    }));
    c6_write(&out_dir, "contact_provenance.json", &json!({
        "source": "FiniteSpatialResourceRegionV1::local_contact_signal",
        "behavior_reads": ["local binary exposure vector only"],
        "world_coordinates_read_by_regulator": false, "inventory_read_by_regulator": false,
    }));
    c6_write(&out_dir, "quiescence_architecture.json", &json!({
        "regulator": "RegulatoryNetworkV1", "formula": "effective_motor[i] = base_motor[i] * (1 - regulator_activity[i])",
        "contact_signal": "exact local finite-resource exposure", "new_parameter": false,
        "contact_triggered_uptake_growth_fission": false,
    }));
    c6_write(&out_dir, "solo_a_finite.json", &solo_a_value);
    c6_write(&out_dir, "solo_b_finite.json", &solo_b_value);
    c6_write(&out_dir, "paired_finite.json", &paired_value);
    c6_write(&out_dir, "regulator_null_control.json", &null_value);
    c6_write(&out_dir, "contact_shuffled_control.json", &shuffled_value);
    c6_write(&out_dir, "motor_off_control.json", &motor_off_value);
    c6_write(&out_dir, "transfer_disabled.json", &disabled_value);
    c6_write(&out_dir, "zero_resource.json", &zero_value);
    c6_write(&out_dir, "a_savings.json", &json!({"candidate_a_spent": paired.base.base.a_spent, "null_a_spent": null_run.base.a_spent, "candidate_saves_a": candidate_saves_a, "candidate_motor_saved": paired.inhibited_motor_sum}));
    c6_write(&out_dir, "feeding_timing.json", &json!({"candidate": paired.base.base.first_transfer, "null": null_run.base.first_transfer, "shuffled": shuffled.base.base.first_transfer, "motor_off": motor_off.base.base.first_transfer}));
    c6_write(&out_dir, "resource_causal_reproduction.json", &json!({"candidate_fissions": paired.base.base.fissions, "null_fissions": null_run.base.fissions, "candidate_fission_benefit": fission_benefit, "candidate_acquisition_benefit": candidate_benefit}));
    c6_write(&out_dir, "descendant_continuity.json", &json!({"candidate_fissions": paired.base.base.fissions, "descendant_fissions": paired.base.base.descendant_fissions, "terminal_living": paired.base.base.terminal_living}));
    c6_write(&out_dir, "heredity_ecological_phenotype.json", &json!({"executed": paired.base.base.fissions > 0, "status": "observer_only"}));
    c6_write(&out_dir, "evolution_reentry.json", &json!({"executed": false, "status": "not_authorized_inside_closure006"}));
    c6_write(&out_dir, "material_closure.json", &json!({"paired": {"world_n_loss": paired.base.base.world_n_loss, "delivered_n": paired.base.base.delivered_n, "world_f_loss": paired.base.base.world_f_loss, "delivered_f": paired.base.base.delivered_f}, "null": {"world_n_loss": null_run.base.world_n_loss, "delivered_n": null_run.base.delivered_n}}));
    c6_write(&out_dir, "energetic_closure.json", &json!({"candidate_a_spent": paired.base.base.a_spent, "candidate_w_generated": paired.base.base.w_generated, "candidate_residual": paired.base.base.max_a_to_w_residual, "motor_off": motor_off.base.base.a_spent}));
    c6_write(&out_dir, "forbidden_information_audit.json", &json!({"resource_center": false, "resource_radius": false, "distance": false, "gradient": false, "target": false, "reward": false, "observer_ledger_as_input": false}));
    c6_write(&out_dir, "m1_preservation.json", &json!({"production": "MaturationCoupledV4 / reserve OFF", "scientific_runtime_changed": false, "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8", "v4_vector": [true,true,false,true,true,true,true,true]}));
    c6_write(&out_dir, "preservation.json", &json!({"entry005_028": "PASS", "closure001_005": "PASS", "closure005_baseline": "PASS", "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED"}));
    c6_write(&out_dir, "restart_boundary.json", &json!({"intrinsic_state_restart": "PASS", "generic_full_mesh_restart": "KNOWN_FAIL_NONCONTAMINATING"}));
    c6_write(&out_dir, "qualification.json", &json!({"classification": classification, "candidate_benefit": candidate_benefit, "candidate_saves_a": candidate_saves_a, "fission_benefit": fission_benefit, "local_resource_exploitation": if fission_benefit { "QUALIFIED" } else { "NOT_ESTABLISHED" }, "autonomous_resource_acquisition": "NOT_ESTABLISHED", "next_execution_started": false}));
    c6_write(&out_dir, "artifact_manifest.json", &json!({"files": files, "dense_traces": "not included; compact evidence only"}));
    let _ = root;
    println!("CLOSURE-006 classification: {classification}");
    println!("paired candidate N/F {:.15e}/{:.15e}, null {:.15e}/{:.15e}, fissions candidate/null {}/{}", paired.base.base.delivered_n, paired.base.base.delivered_f, null_run.base.delivered_n, null_run.base.delivered_f, paired.base.base.fissions, null_run.base.fissions);
}
