
const C9_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-009-DIRECT-POST-INGESTIVE-MOTOR-ALLOCATION-REQUALIFICATION-001";
const C9_START: &str = "a7d51bcc984df55f90c03231c8503e19a0c5603c";
const C9_TOL: f64 = 1e-10;

fn c9_direct_mechanics(
    agent: &mut ClosureAgent,
    material_signal: f64,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Result<(usize, usize, f64, f64, f64, f64, f64), String> {
    let raw = entry025_anti(&agent.polarity);
    let clutch = entry025_direct(&agent.polarity);
    if raw.len() != agent.mesh.n() || clutch.len() != raw.len() {
        return Err("direct material motor topology mismatch".into());
    }
    let motor: Vec<f64> = raw
        .iter()
        .map(|base| base * (1.0 - material_signal))
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
    .map_err(|e| format!("direct material motor: {e:?}"))?;
    if !agent.mesh.physical_runtime_valid() || !agent.mesh.lifecycle_invariants_hold() {
        return Err("invalid mesh after direct material mechanics".into());
    }
    let spent = ledger.contractility.as_ref().map(|x| x.resource_spent).unwrap_or(0.0);
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

fn c9_direct_run(
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
            let mechanics_result = if motor_off {
                c6_motor_off_mechanics(agent, &mechanics, &contractility, &traction)
            } else {
                c9_direct_mechanics(agent, signal, &mechanics, &contractility, &traction)
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


fn c9_direct_value(run: &C7Run) -> Value {
    c7_value(run)
}
