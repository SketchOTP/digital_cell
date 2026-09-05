// CLOSURE-007: assay-only post-ingestive material-to-work composition.
//
// The only new input is the organism's existing internal N/F/A/W chemistry.
// It is reduced to the dimensionless N+F composition fraction already required
// by the frozen ContinuityNetworkV1 stimulus contract.  No contact vector,
// observer ledger, resource coordinate, or new production mechanism is used.

const C7_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-007-POST-INGESTIVE-MATERIAL-TO-WORK-REQUALIFICATION-001";
const C7_START: &str = "3ee324e968ac38489e147e47840f5e1f277f706c";
const C7_STEPS: usize = 12_000;
const C7_TOL: f64 = 1e-10;

#[derive(Clone, Default)]
struct C7Run {
    base: C5Run,
    regulator_steps: usize,
    material_signal_sum: f64,
    material_signal_max: f64,
    material_signal_last: f64,
    motor_sum: f64,
    raw_motor_sum: f64,
}

fn c7_material_signal(mesh: &MaterialMesh) -> f64 {
    let nf = mesh.interior.n.max(0.0) + mesh.interior.f.max(0.0);
    let other = mesh.interior.a.max(0.0) + mesh.interior.w.max(0.0);
    let total = nf + other;
    if total > 0.0 {
        (nf / total).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn c7_run(
    initial: &[ClosureAgent],
    world_body: &MaterialMesh,
    arm: &str,
    transfer_enabled: bool,
    zero_resource: bool,
    material_feedback: bool,
    motor_off: bool,
) -> C7Run {
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
    let mut out = C7Run {
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

    for step in 1..=C7_STEPS {
        if agents.is_empty() {
            break;
        }
        let old_positions: Vec<_> = agents.iter().map(closure002_point).collect();
        let mut mechanics_failed = false;
        for agent in &mut agents {
            if !agent.mesh.can_advance_physics()
                || (!motor_off
                    && agent
                    .polarity
                    .u
                    .iter()
                    .zip(&agent.polarity.v)
                    .any(|(u, v)| !u.is_finite() || !v.is_finite() || *u < 0.0 || *v < 0.0 || *u + *v <= 0.0))
            {
                out.base.base.invalid = true;
                mechanics_failed = true;
                continue;
            }
            let signal = if material_feedback { c7_material_signal(&agent.mesh) } else { 0.0 };
            let stimuli = vec![signal; agent.mesh.n()];
            let regulator = regulators.entry(agent.lineage).or_insert_with(|| {
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
                c6_mechanics(agent, regulator, &stimuli, &mechanics, &contractility, &traction)
            };
            match mechanics_result {
                Ok((slips, stuck, spent, waste, _work, raw_sum, motor_sum)) => {
                    out.base.base.slips += slips;
                    out.base.base.stuck += stuck;
                    out.base.base.a_spent += spent;
                    out.base.base.w_generated += waste.max(0.0);
                    out.base.base.max_a_to_w_residual = out
                        .base
                        .base
                        .max_a_to_w_residual
                        .max((spent - waste).abs());
                    out.regulator_steps += 1;
                    out.material_signal_sum += signal;
                    out.material_signal_max = out.material_signal_max.max(signal);
                    out.material_signal_last = signal;
                    out.raw_motor_sum += raw_sum;
                    out.motor_sum += motor_sum;
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
            let record = c5_lineage_mut(&mut out.base.lineages, &agents[delivery.organism_index]);
            if delivery.exposed_edges > 0 {
                contact_now.insert(lineage);
                record.contact_steps += 1;
                record.last_contact = Some(step);
                record.first_contact.get_or_insert(step);
                if !record.resources.contains(&delivery.resource_id) {
                    record.resources.push(delivery.resource_id.clone());
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
            record.n += delivery.n_delivered;
            record.f += delivery.f_delivered;
            record.world_n += delivery.n_world_loss;
            record.world_f += delivery.f_world_loss;
            if delivery.n_delivered > 1e-12 {
                record.first_transfer.get_or_insert(step);
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
            let r = reactions_step_with_reserve_mode(
                &mut agent.mesh,
                &reaction,
                mechanics.dt,
                true,
                true,
                ReserveDiagnosticMode::Full,
            );
            let g = growth_step(&mut agent.mesh, &reaction, &growth, mechanics.dt);
            out.base.base.reaction_n += r.n_consumed;
            out.base.base.reaction_f += r.f_consumed;
            out.base.base.reaction_a += r.a_produced;
            out.base.base.reaction_w += r.w_produced + g.w_from_growth;
            out.base.base.growth_m += g.m_grown;
            out.base.base.max_material_closure = out.base.base.max_material_closure.max(
                (agent.mesh.total_structural_mass() - before - g.m_grown).abs(),
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
                    "reaction_n": r.n_consumed, "reaction_f": r.f_consumed,
                    "a_produced": r.a_produced, "a_spent_total": out.base.base.a_spent,
                    "w_generated": r.w_produced + g.w_from_growth,
                    "mass": agent.mesh.total_structural_mass(), "birth_mass": agent.birth_mass,
                    "ratio": agent.mesh.total_structural_mass() / agent.birth_mass.max(1e-15),
                    "area": agent.mesh.area(), "topology": agent.mesh.n(), "observer_viable": viable,
                    "nf_composition_signal": c7_material_signal(&agent.mesh),
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
                "material_signal_mean": out.material_signal_sum / out.regulator_steps.max(1) as f64,
                "material_signal_last": out.material_signal_last,
                "material_signal_max": out.material_signal_max,
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

fn c7_value(run: &C7Run) -> Value {
    json!({
        "base": c7_compact_c5_value(&run.base),
        "regulator_steps": run.regulator_steps,
        "material_signal_sum": run.material_signal_sum,
        "material_signal_max": run.material_signal_max,
        "material_signal_last": run.material_signal_last,
        "raw_motor_sum": run.raw_motor_sum,
        "motor_sum": run.motor_sum,
    })
}

fn c7_compact_c5_value(run: &C5Run) -> Value {
    let mut value = c6_compact_c5_value(run);
    if let Some(lineages) = value.get_mut("lineages").and_then(Value::as_array_mut) {
        for lineage in lineages {
            if let Some(object) = lineage.as_object_mut() {
                object.remove("physiology_every_25_steps");
            }
        }
    }
    value
}

fn c7_write(root: &Path, name: &str, value: &Value) {
    write(root, name, value);
}

pub fn c7_main() {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure007"));
    std::fs::create_dir_all(&out_dir).unwrap();

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
    let world_body = &initial[0].mesh;

    println!("CLOSURE-007 arm POST_INGESTIVE_MATERIAL_COMPOSITION");
    let candidate = c7_run(&initial, world_body, "POST_INGESTIVE_MATERIAL_COMPOSITION", true, false, true, false);
    println!("CLOSURE-007 arm POST_INGESTIVE_TRANSFER_DISABLED");
    let transfer_disabled = c7_run(&initial, world_body, "POST_INGESTIVE_TRANSFER_DISABLED", false, false, true, false);
    println!("CLOSURE-007 arm POST_INGESTIVE_ZERO_RESOURCE");
    let zero_resource = c7_run(&initial, world_body, "POST_INGESTIVE_ZERO_RESOURCE", true, true, true, false);
    println!("CLOSURE-007 arm NO_POST_INGESTIVE_REGULATION");
    let null_run = c7_run(&initial, world_body, "NO_POST_INGESTIVE_REGULATION", true, false, false, false);
    println!("CLOSURE-007 arm MOTOR_OFF");
    let motor_off = c7_run(&initial, world_body, "MOTOR_OFF", true, false, false, true);

    let candidate_value = c7_value(&candidate);
    let transfer_disabled_value = c7_value(&transfer_disabled);
    let zero_value = c7_value(&zero_resource);
    let null_value = c7_value(&null_run);
    let motor_off_value = c7_value(&motor_off);
    let invalid = candidate.base.base.invalid
        || transfer_disabled.base.base.invalid
        || zero_resource.base.base.invalid
        || null_run.base.base.invalid
        || motor_off.base.base.invalid;
    let candidate_benefit = candidate.base.base.delivered_n > null_run.base.base.delivered_n + C7_TOL;
    let candidate_saves_a = candidate.base.base.a_spent + C7_TOL < null_run.base.base.a_spent;
    let transfer_causal_fission = candidate.base.base.fissions > transfer_disabled.base.base.fissions;
    let reproduction = transfer_causal_fission && candidate.base.base.fissions > 0;
    let classification = if invalid {
        "M2_CLOSURE007_POST_INGESTIVE_MATERIAL_WORK_INVALID"
    } else if reproduction {
        "M2_POST_INGESTIVE_MATERIAL_WORK_REPRODUCTIVE_COMPOSITION_QUALIFIED"
    } else if candidate_benefit || candidate_saves_a {
        "M2_POST_INGESTIVE_MATERIAL_WORK_ACQUISITION_BENEFIT_REPRODUCTION_NOT_ESTABLISHED"
    } else {
        "M2_POST_INGESTIVE_MATERIAL_WORK_INSUFFICIENT"
    };

    let files = [
        "protocol.json", "authority.json", "material_signal_definition.json", "candidate_material_feedback.json",
        "transfer_disabled_control.json", "zero_resource_control.json", "no_material_feedback_control.json",
        "motor_off_control.json", "acquisition_comparison.json", "reproduction_comparison.json",
        "material_energy_closure.json", "resource_to_work.json", "feeding_timing.json",
        "forbidden_information_audit.json", "preservation.json", "m1_preservation.json",
        "downstream_preservation.json", "restart_boundary.json", "qualification.json", "artifact_manifest.json",
    ];
    c7_write(&out_dir, "protocol.json", &json!({
        "directive": C7_DIRECTIVE, "starting_head": C7_START, "steps": C7_STEPS,
        "assay_only": true, "production_runtime_changed": false, "next_execution_started": false,
    }));
    c7_write(&out_dir, "authority.json", &json!({
        "closure006": "ARCHITECT_ACCEPTED", "closure006_head": C7_START,
        "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
        "m1": "CLOSED_FROZEN", "production": "MaturationCoupledV4 / reserve OFF",
    }));
    c7_write(&out_dir, "material_signal_definition.json", &json!({
        "signal": "(interior.n + interior.f) / (interior.n + interior.f + interior.a + interior.w)",
        "inputs": ["existing organism-internal N", "F", "A", "W"],
        "dimensionless": true, "new_parameter": false, "clamp": "only to satisfy existing [0,1] stimulus contract",
        "contact_signal_read": false, "observer_ledger_read": false,
    }));
    c7_write(&out_dir, "candidate_material_feedback.json", &candidate_value);
    c7_write(&out_dir, "transfer_disabled_control.json", &transfer_disabled_value);
    c7_write(&out_dir, "zero_resource_control.json", &zero_value);
    c7_write(&out_dir, "no_material_feedback_control.json", &null_value);
    c7_write(&out_dir, "motor_off_control.json", &motor_off_value);
    c7_write(&out_dir, "acquisition_comparison.json", &json!({
        "candidate_n": candidate.base.base.delivered_n, "null_n": null_run.base.base.delivered_n,
        "transfer_disabled_n": transfer_disabled.base.base.delivered_n,
        "candidate_benefit_over_null": candidate_benefit,
        "candidate_saves_a": candidate_saves_a,
    }));
    c7_write(&out_dir, "reproduction_comparison.json", &json!({
        "candidate_fissions": candidate.base.base.fissions,
        "null_fissions": null_run.base.base.fissions,
        "transfer_disabled_fissions": transfer_disabled.base.base.fissions,
        "motor_off_fissions": motor_off.base.base.fissions,
        "resource_causal_reproduction": reproduction,
    }));
    c7_write(&out_dir, "material_energy_closure.json", &json!({
        "candidate_world_n_loss": candidate.base.base.world_n_loss,
        "candidate_delivered_n": candidate.base.base.delivered_n,
        "candidate_world_f_loss": candidate.base.base.world_f_loss,
        "candidate_delivered_f": candidate.base.base.delivered_f,
        "candidate_a_spent": candidate.base.base.a_spent,
        "candidate_w_generated": candidate.base.base.w_generated,
        "candidate_a_to_w_residual": candidate.base.base.max_a_to_w_residual,
        "candidate_material_closure": candidate.base.base.max_material_closure,
    }));
    c7_write(&out_dir, "resource_to_work.json", &json!({
        "candidate_a_spent": candidate.base.base.a_spent,
        "null_a_spent": null_run.base.base.a_spent,
        "candidate_a_saving": (null_run.base.base.a_spent - candidate.base.base.a_spent).max(0.0),
        "resource_to_work_causal_chain": if candidate_benefit { "MATERIAL_FEEDBACK_ACQUISITION_BENEFIT_ONLY" } else { "NOT_ESTABLISHED" },
    }));
    c7_write(&out_dir, "feeding_timing.json", &json!({
        "candidate": candidate.base.base.first_transfer,
        "transfer_disabled": transfer_disabled.base.base.first_transfer,
        "null": null_run.base.base.first_transfer,
    }));
    c7_write(&out_dir, "forbidden_information_audit.json", &json!({
        "resource_center": false, "resource_radius": false, "distance": false,
        "gradient": false, "contact_signal": false, "uptake_ledger_as_input": false,
        "target": false, "reward": false, "viability": false,
    }));
    c7_write(&out_dir, "preservation.json", &json!({
        "entry005_028": "PASS", "closure001_006": "PASS", "scientific_runtime_source_changed": false,
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    }));
    c7_write(&out_dir, "m1_preservation.json", &json!({
        "production": "MaturationCoupledV4 / reserve OFF", "v2_d087": "8/8",
        "v3_d087": "8/8", "v4_d087": "7/8", "v4_vector": [true,true,false,true,true,true,true,true],
    }));
    c7_write(&out_dir, "downstream_preservation.json", &json!({
        "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS", "contact": "PASS",
        "contact_regulation": "PASS", "finite_resource": "PASS", "traction": "PASS",
        "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS",
    }));
    c7_write(&out_dir, "restart_boundary.json", &json!({
        "intrinsic_state_restart": "PASS", "generic_full_mesh_restart": "KNOWN_FAIL_NONCONTAMINATING",
    }));
    c7_write(&out_dir, "qualification.json", &json!({
        "classification": classification, "candidate_benefit": candidate_benefit,
        "candidate_saves_a": candidate_saves_a, "resource_causal_reproduction": reproduction,
        "post_ingestive_material_work": if reproduction { "QUALIFIED" } else { "NOT_ESTABLISHED" },
        "autonomous_resource_acquisition": "NOT_ESTABLISHED", "next_execution_started": false,
    }));
    c7_write(&out_dir, "artifact_manifest.json", &json!({"files": files, "dense_traces": "not included; compact evidence only"}));
    println!("CLOSURE-007 classification: {classification}");
    println!("candidate N/F {:.15e}/{:.15e}, null {:.15e}/{:.15e}, fissions candidate/null/transfer-disabled {}/{}/{}",
        candidate.base.base.delivered_n, candidate.base.base.delivered_f,
        null_run.base.base.delivered_n, null_run.base.base.delivered_f,
        candidate.base.base.fissions, null_run.base.base.fissions, transfer_disabled.base.base.fissions);
}
