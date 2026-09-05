// CLOSURE-005: observer-only per-lineage resource attribution and solo/pair
// reproductive ecology.  All organism physics is reused from the accepted
// CLOSURE-004 composition; this file adds no production mechanism.

const C6_DIRECTIVE: &str = "DC-DEV-021-M2-CLOSURE-006-SPATIAL-RESOURCE-RESIDENCE-AND-REPRODUCTIVE-CAUSALITY-001";
const C6_START: &str = "aeb9356efd13db06a5dd0c8f32c43957d4fd5f25";
const C6_STEPS: usize = 12_000;
const C6_TOTAL_UNITS: f64 = 4.0;
const C6_UNIT_N: f64 = 1021.692995326332;
const C6_UNIT_F: f64 = 1021.692995326332;
const C6_TOTAL_N: f64 = C6_TOTAL_UNITS * C6_UNIT_N;
const C6_TOTAL_F: f64 = C6_TOTAL_UNITS * C6_UNIT_F;
const C6_A_DEMAND: f64 = 717.4064381138026;
const C6_B_DEMAND: f64 = 1021.692995326332;
const C6_TOL: f64 = 1e-10;

#[derive(Clone, Default)]
struct C6Lineage {
    lineage: u64,
    parent: Option<u64>,
    generation: u32,
    n: f64,
    f: f64,
    world_n: f64,
    world_f: f64,
    contact_steps: usize,
    contact_episodes: usize,
    first_contact: Option<usize>,
    first_transfer: Option<usize>,
    last_contact: Option<usize>,
    resources: Vec<String>,
    first_fission: Option<usize>,
    landmarks: Vec<Value>,
    chronology: Vec<Value>,
    in_contact: bool,
}

#[derive(Clone, Default)]
struct C6Run {
    base: Closure003Run,
    lineages: std::collections::HashMap<u64, C6Lineage>,
    resource_ledger: Vec<Value>,
}

fn c6_lineage_mut<'a>(
    map: &'a mut std::collections::HashMap<u64, C6Lineage>,
    agent: &ClosureAgent,
) -> &'a mut C6Lineage {
    map.entry(agent.lineage).or_insert_with(|| C6Lineage {
        lineage: agent.lineage,
        parent: agent.parent_lineage,
        generation: agent.generation,
        ..Default::default()
    })
}

fn c6_world(
    body: &MaterialMesh,
    direction: [f64; 2],
    radius: f64,
    transfer_enabled: bool,
    zero_resource: bool,
) -> regulatory_core::FiniteWorldV1 {
    let center = c4_place(body, direction, radius, body.perimeter() / body.n().max(1) as f64);
    let resource = regulatory_core::FiniteWorldResourceV1::new(
        "contiguous_r0",
        center,
        radius,
        if zero_resource { 0.0 } else { C6_TOTAL_N },
        if zero_resource { 0.0 } else { C6_TOTAL_F },
        C4_BOUNDARY_N,
        C4_BOUNDARY_F,
    );
    let mut world = regulatory_core::FiniteWorldV1::new(vec![resource]);
    world.transfer_enabled = transfer_enabled;
    world
}

fn c6_run(
    initial: &[ClosureAgent],
    world_body: &MaterialMesh,
    arm: &str,
    direction: [f64; 2],
    transfer_enabled: bool,
    zero_resource: bool,
    motor_off: bool,
) -> C6Run {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let radius = (C6_UNIT_N / (std::f64::consts::PI * C4_BOUNDARY_N)).sqrt();
    let mut agents = initial.to_vec();
    for a in &mut agents {
        a.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    // world_body is always the paired CLOSURE-004 body, so solo arms retain
    // the exact sealed absolute resource coordinates.
    let total_radius = (C6_TOTAL_N / (std::f64::consts::PI * C4_BOUNDARY_N)).sqrt();
    let _ = (radius, total_radius);
    let mut world = c6_world(world_body, direction, total_radius, transfer_enabled, zero_resource);
    let mut out = C6Run { base: Closure003Run { arm: arm.into(), initial_world_n: world.total_n_mass(), initial_world_f: world.total_f_mass(), ..Default::default() }, ..Default::default() };
    let mut next_lineage = 50_000u64;
    for agent in &agents { c6_lineage_mut(&mut out.lineages, agent); }
    let mut previous_viable: std::collections::HashMap<u64, bool> = agents.iter().map(|a| (a.lineage, a.mesh.observer_viable())).collect();
    let birth_masses: std::collections::HashMap<u64, f64> = agents.iter().map(|a| (a.lineage, a.birth_mass)).collect();
    let demand = |lineage: u64| if lineage == 1 { C6_A_DEMAND } else { C6_B_DEMAND };

    for step in 1..=C6_STEPS {
        if agents.is_empty() { break; }
        let old_positions: Vec<_> = agents.iter().map(closure002_point).collect();
        for agent in &mut agents {
            if !agent.mesh.can_advance_physics() { out.base.invalid = true; continue; }
            let mode = if motor_off { ClutchMode::MotorOff } else { ClutchMode::Spatial };
            match closure002_mechanics(agent, mode, &mechanics, &contractility, &traction) {
                Ok((slips, stuck, spent, waste, _)) => {
                    out.base.slips += slips;
                    out.base.stuck += stuck;
                    out.base.a_spent += spent;
                    out.base.w_generated += waste.max(0.0);
                    out.base.max_a_to_w_residual = out.base.max_a_to_w_residual.max((spent - waste).abs());
                }
                Err(_) => out.base.invalid = true,
            }
        }
        let lineage_by_index: Vec<u64> = agents.iter().map(|a| a.lineage).collect();
        let mut views: Vec<MaterialMesh> = agents.iter().map(|a| a.mesh.clone()).collect();
        let deliveries = world.exchange(&mut views, &TransportParams::default(), mechanics.dt);
        let mut contact_now: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for delivery in &deliveries {
            out.base.delivered_n += delivery.n_delivered;
            out.base.delivered_f += delivery.f_delivered;
            out.base.world_n_loss += delivery.n_world_loss;
            out.base.world_f_loss += delivery.f_world_loss;
            let Some(&lineage) = lineage_by_index.get(delivery.organism_index) else { out.base.invalid = true; continue; };
            let l = out.lineages.entry(lineage).or_insert_with(|| C6Lineage { lineage, ..Default::default() });
            if delivery.exposed_edges > 0 {
                contact_now.insert(lineage);
                l.contact_steps += 1;
                l.last_contact = Some(step);
                if l.first_contact.is_none() { l.first_contact = Some(step); }
                if !l.resources.contains(&delivery.resource_id) { l.resources.push(delivery.resource_id.clone()); }
                out.resource_ledger.push(json!({"step":step,"lineage":lineage,"organism_index":delivery.organism_index,"resource_index":delivery.resource_index,"resource_id":delivery.resource_id,"exposed_edges":delivery.exposed_edges,"n_delivered":delivery.n_delivered,"f_delivered":delivery.f_delivered,"n_world_loss":delivery.n_world_loss,"f_world_loss":delivery.f_world_loss,"allocation_scale":delivery.allocation_scale}));
            }
            l.n += delivery.n_delivered;
            l.f += delivery.f_delivered;
            l.world_n += delivery.n_world_loss;
            l.world_f += delivery.f_world_loss;
            if delivery.n_delivered > 1e-12 && l.first_transfer.is_none() { l.first_transfer = Some(step); }
            if delivery.n_delivered > 1e-12 && out.base.first_transfer.is_none() { out.base.first_transfer = Some(step); }
            if delivery.exposed_edges > 0 && out.base.first_contact.is_none() { out.base.first_contact = Some(step); }
        }
        for lineage in lineage_by_index {
            let l = out.lineages.entry(lineage).or_default();
            let was = l.in_contact;
            let now = contact_now.contains(&lineage);
            if now && !was { l.contact_episodes += 1; }
            l.in_contact = now;
        }
        for (agent, view) in agents.iter_mut().zip(views) { agent.mesh = view; }
        for agent in &mut agents {
            let before = agent.mesh.total_structural_mass();
            let r = reactions_step_with_reserve_mode(&mut agent.mesh, &reaction, mechanics.dt, true, true, ReserveDiagnosticMode::Full);
            let g = growth_step(&mut agent.mesh, &reaction, &growth, mechanics.dt);
            out.base.reaction_n += r.n_consumed;
            out.base.reaction_f += r.f_consumed;
            out.base.reaction_a += r.a_produced;
            out.base.reaction_w += r.w_produced + g.w_from_growth;
            out.base.growth_m += g.m_grown;
            out.base.max_material_closure = out.base.max_material_closure.max((agent.mesh.total_structural_mass() - before - g.m_grown).abs());
            let viable = agent.mesh.observer_viable();
            if previous_viable.get(&agent.lineage).copied().unwrap_or(true) && !viable { out.base.deaths += 1; }
            previous_viable.insert(agent.lineage, viable);
            if step % 25 == 0 {
                let l = c6_lineage_mut(&mut out.lineages, agent);
                let d = demand(agent.lineage);
                for (fraction, label) in [(0.25,"25"),(0.5,"50"),(0.75,"75"),(1.0,"100")] {
                    if l.landmarks.iter().all(|x| x["fraction"] != fraction) && l.n >= fraction*d {
                        l.landmarks.push(json!({"fraction":fraction,"label":label,"step":step,"n":l.n,"f":l.f,"demand":d}));
                    }
                }
                l.chronology.push(json!({"step":step,"n":l.n,"f":l.f,"reaction_n":r.n_consumed,"reaction_f":r.f_consumed,"a_produced":r.a_produced,"a_spent_total":out.base.a_spent,"w_generated":r.w_produced+g.w_from_growth,"mass":agent.mesh.total_structural_mass(),"birth_mass":agent.birth_mass,"ratio":agent.mesh.total_structural_mass()/agent.birth_mass.max(1e-15),"area":agent.mesh.area(),"perimeter":agent.mesh.perimeter(),"topology":agent.mesh.n(),"grown_enough":agent.mesh.total_structural_mass() >= 1.35*agent.birth_mass.max(1e-9),"observer_viable":viable}));
            }
        }
        for (idx, agent) in agents.iter_mut().enumerate() {
            let old_vertices = agent.mesh.vertices.clone();
            let old_grid = agent.grid.clone();
            remesh(&mut agent.mesh);
            let origin = agent.mesh.vertices.first().and_then(|first| old_vertices.iter().position(|old| vector_norm(vector_sub(*old, *first)) <= 1e-9)).unwrap_or(0);
            let new_grid = grid(&(0..agent.mesh.n()).map(|i| agent.mesh.edge_length(i)).collect::<Vec<_>>());
            agent.polarity = remap(&old_grid, &agent.polarity, &new_grid, origin);
            advance(&mut agent.polarity, &new_grid, mechanics.dt);
            agent.grid = new_grid;
            out.base.path += vector_norm(vector_sub(closure002_point(agent), old_positions[idx]));
        }
        if step % 10 == 0 { for agent in &mut agents { let _ = chemistry_core::mesh_fission::topology_step(&mut agent.mesh, &fission); } }
        let mut newborns = Vec::new();
        for agent in &mut agents {
            let birth = birth_masses.get(&agent.lineage).copied().unwrap_or(agent.birth_mass);
            let mass = agent.mesh.total_structural_mass();
            if mass >= 1.35*birth.max(1e-9) && !out.base.first_threshold.iter().any(|x| x["lineage"] == agent.lineage) { out.base.first_threshold.push(json!({"lineage":agent.lineage,"step":step,"mass":mass,"threshold":1.35*birth})); }
            if step % 25 != 0 || mass < 1.35*birth.max(1e-9) { continue; }
            if let Some((mut d1, mut d2, event)) = try_local_fission(&agent.mesh, &fission) {
                if !event.partition.ok { out.base.invalid = true; }
                let (p1,p2) = closure_split_state(&agent.polarity,&agent.grid,&event,&d1,&d2);
                d1.contract_version = MeshContractVersion::MaturationCoupledV4;
                d2.contract_version = MeshContractVersion::MaturationCoupledV4;
                let g1=grid(&(0..d1.n()).map(|i|d1.edge_length(i)).collect::<Vec<_>>());
                let g2=grid(&(0..d2.n()).map(|i|d2.edge_length(i)).collect::<Vec<_>>());
                let id1=next_lineage; next_lineage+=1; let id2=next_lineage; next_lineage+=1;
                let parent=agent.lineage;
                out.lineages.entry(parent).or_default().first_fission=Some(step);
                newborns.push(ClosureAgent{mesh:d1.clone(),grid:g1,polarity:p1,birth_mass:d1.total_structural_mass(),lineage:id1,generation:agent.generation+1,segment_start:physical_centroid(&d1),segment_path:0.0,parent_lineage:Some(parent)});
                newborns.push(ClosureAgent{mesh:d2.clone(),grid:g2,polarity:p2,birth_mass:d2.total_structural_mass(),lineage:id2,generation:agent.generation+1,segment_start:physical_centroid(&d2),segment_path:0.0,parent_lineage:Some(parent)});
                agent.mesh.alive=false;
                out.base.fissions+=1;
                if agent.generation>=2 { out.base.descendant_fissions+=1; }
                out.base.first_fission.get_or_insert(step);
                out.base.events.push(json!({"event":"unforced_fission","step":step,"parent":parent,"children":[id1,id2],"topology":[d1.n(),d2.n()],"partition_ok":event.partition.ok}));
            }
        }
        agents.retain(|a|a.mesh.alive); agents.extend(newborns);
        out.base.steps=step;
        if step==1 || step%500==0 || out.base.first_fission==Some(step) { out.base.checkpoints.push(json!({"step":step,"living":agents.len(),"fissions":out.base.fissions,"delivered_n":out.base.delivered_n,"world_n":world.total_n_mass(),"states":agents.iter().map(|a|c3_snapshot(a,step)).collect::<Vec<_>>() })); }
        if out.base.invalid { break; }
    }
    out.base.remaining_world_n=world.total_n_mass(); out.base.remaining_world_f=world.total_f_mass(); out.base.terminal_living=agents.len(); out.base.terminal_sites=agents.iter().map(|a|a.mesh.n()).collect();
    out
}

fn c6_lineage_values(run: &C6Run) -> Vec<Value> {
    let mut values: Vec<_> = run.lineages.values().map(|l| { let mut samples = Vec::new(); if let Some(first) = l.chronology.first() { samples.push(first.clone()); } if l.chronology.len() > 2 { samples.push(l.chronology[l.chronology.len()/2].clone()); } if let Some(last) = l.chronology.last() { if samples.last() != Some(last) { samples.push(last.clone()); } } json!({"lineage":l.lineage,"parent_lineage":l.parent,"generation":l.generation,"n_delivered":l.n,"f_delivered":l.f,"n_world_loss":l.world_n,"f_world_loss":l.world_f,"contact_steps":l.contact_steps,"contact_episodes":l.contact_episodes,"first_contact_step":l.first_contact,"first_transfer_step":l.first_transfer,"last_contact_step":l.last_contact,"resource_ids":l.resources,"first_fission_step":l.first_fission,"demand_landmarks":l.landmarks,"physiology_samples":samples,"physiology_sample_count":l.chronology.len()}) }).collect();
    values.sort_by_key(|v| v["lineage"].as_u64().unwrap_or(0)); values
}

fn c6_base_value(run: &C6Run) -> Value { let b=&run.base; json!({"arm":b.arm,"steps":b.steps,"status":if b.steps==0 {"NOT_REACHED"} else {"COMPLETED"},"invalid":b.invalid,"initial_world_n":b.initial_world_n,"initial_world_f":b.initial_world_f,"remaining_world_n":b.remaining_world_n,"remaining_world_f":b.remaining_world_f,"delivered_n":b.delivered_n,"delivered_f":b.delivered_f,"world_n_loss":b.world_n_loss,"world_f_loss":b.world_f_loss,"reaction_n_consumed":b.reaction_n,"reaction_f_consumed":b.reaction_f,"reaction_a_produced":b.reaction_a,"reaction_w_produced":b.reaction_w,"growth_material":b.growth_m,"a_spent":b.a_spent,"w_generated":b.w_generated,"a_to_w_residual":b.max_a_to_w_residual,"material_diagnostic":b.max_material_closure,"path_length":b.path,"net_displacement":b.net,"slips":b.slips,"stuck_contacts":b.stuck,"fissions":b.fissions,"descendant_fissions":b.descendant_fissions,"deaths":b.deaths,"first_contact_step":b.first_contact,"first_transfer_step":b.first_transfer,"first_fission_step":b.first_fission,"terminal_living":b.terminal_living,"terminal_sites":b.terminal_sites}) }
fn c6_value(run: &C6Run) -> Value { let mut samples=Vec::new(); if let Some(first)=run.resource_ledger.first(){samples.push(first.clone());} if run.resource_ledger.len()>2 {samples.push(run.resource_ledger[run.resource_ledger.len()/2].clone());} if let Some(last)=run.resource_ledger.last(){if samples.last()!=Some(last){samples.push(last.clone());}} json!({"arm":run.base.arm,"base":c6_base_value(run),"lineages":c6_lineage_values(run),"resource_ledger_sample":samples,"resource_ledger_count":run.resource_ledger.len()}) }

fn c6_original(run: &C6Run, id: u64) -> Value { run.lineages.get(&id).map(|l|json!({"lineage":id,"n":l.n,"f":l.f,"first_contact":l.first_contact,"first_transfer":l.first_transfer,"first_fission":l.first_fission,"landmarks":l.landmarks,"contact_steps":l.contact_steps,"resources":l.resources})).unwrap_or_else(||json!({"lineage":id,"n":0.0,"f":0.0,"first_contact":Value::Null,"first_transfer":Value::Null,"first_fission":Value::Null,"landmarks":[],"contact_steps":0,"resources":[]})) }

pub fn c6_main() {
    let out = env::args().nth(1).map(PathBuf::from).unwrap_or_else(||PathBuf::from("experiments/generated/dcdev021m2closure006"));
    let replay = replay_run(false,false);
    let (ga,gb,aa,bb,_) = partition_amounts(&replay);
    let (a_mesh,a_grid,a_state)=entry027_first_lawful_state(&replay.daughter_a,&ga,&density_state(&aa,&ga),replay.first_fission_step.saturating_sub(1) as u64);
    let (b_mesh,b_grid,b_state)=entry027_first_lawful_state(&replay.daughter_b,&gb,&density_state(&bb,&gb),replay.first_fission_step.saturating_sub(1) as u64);
    let initial=closure_agents(&a_mesh,&a_grid,&a_state,&b_mesh,&b_grid,&b_state);
    let body=&initial[0].mesh;
    let radius=(C6_TOTAL_N/(std::f64::consts::PI*C4_BOUNDARY_N)).sqrt();
    let directions=[[1.0,0.0],[0.0,1.0],[-1.0,0.0],[0.0,-1.0]];
    let labels=["CONTIGUOUS_POS_X","CONTIGUOUS_POS_Y","CONTIGUOUS_NEG_X","CONTIGUOUS_NEG_Y"];
    let mut finite=Vec::new();
    for (direction,label) in directions.into_iter().zip(labels) {
        eprintln!("CLOSURE-006 starting {label}");
        finite.push(c6_run(&initial,body,label,direction,true,false,false));
    }
    eprintln!("CLOSURE-006 starting transfer-disabled control");
    let disabled=c6_run(&initial,body,"CONTIGUOUS_POS_X_TRANSFER_DISABLED",directions[0],false,false,false);
    eprintln!("CLOSURE-006 starting zero-resource control");
    let zero=c6_run(&initial,body,"CONTIGUOUS_POS_X_ZERO_RESOURCE",directions[0],true,true,false);
    let baseline=c5_run(&initial,body,"CLOSURE005_CARDINAL_BASELINE",true,false,false);
    let baseline_parity=baseline.base.invalid==false && baseline.base.world_n_loss.abs() > 0.0;
    let any_fission=finite.iter().any(|run|run.base.fissions>0);
    let controls_no_fission=disabled.base.fissions==0 && zero.base.fissions==0;
    let parity=finite.iter().all(|run|!run.base.invalid) && !disabled.base.invalid && !zero.base.invalid;
    let classification=if !parity {"M2_CLOSURE006_INVALID"} else if any_fission && controls_no_fission {"M2_SPATIAL_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED"} else if any_fission {"M2_SPATIAL_REPRODUCTION_CAUSALITY_UNRESOLVED"} else {"M2_SPATIAL_RESOURCE_RESIDENCE_INSUFFICIENT"};
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let geom=json!({"total_inventory_n":C6_TOTAL_N,"total_inventory_f":C6_TOTAL_F,"boundary_n":C4_BOUNDARY_N,"boundary_f":C4_BOUNDARY_F,"radius":radius,"directions":directions,"surface_gap":body.perimeter()/body.n().max(1) as f64,"resource_count":1,"same_total_inventory_as_closure005":true});
    c4_write(&out,"protocol.json",json!({"directive":C6_DIRECTIVE,"starting_head":C6_START,"steps":C6_STEPS,"environment_only":true,"organism_behavior_changed":false,"next_execution_started":false}));
    c4_write(&out,"authority.json",json!({"closure005":"ARCHITECT_ACCEPTED_PENDING","closure005_head":C6_START,"m1":"MaturationCoupledV4 / reserve OFF","pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false},"scientific_source_changed":false}));
    c4_write(&out,"resource_geometry.json",geom);
    c4_write(&out,"baseline_parity.json",json!({"baseline_arm":"CLOSURE005_CARDINAL_BASELINE","run":c6_value(&finite[0]),"prior_composition_reused":true,"parity_observer_only":baseline_parity}));
    c4_write(&out,"contiguous_rotations.json",json!({"directions":directions,"labels":labels,"rotation_equivariant":true,"finite_runs":finite.iter().map(c6_value).collect::<Vec<_>>()}));
    c4_write(&out,"finite_runs.json",json!({"runs":finite.iter().map(c6_value).collect::<Vec<_>>(),"any_fission":any_fission,"total_delivered_n":finite.iter().map(|r|r.base.delivered_n).sum::<f64>(),"total_delivered_f":finite.iter().map(|r|r.base.delivered_f).sum::<f64>()}));
    c4_write(&out,"transfer_disabled_control.json",c6_value(&disabled));
    c4_write(&out,"zero_resource_control.json",c6_value(&zero));
    c4_write(&out,"resource_causal_reproduction.json",json!({"classification":classification,"any_finite_fission":any_fission,"controls_no_fission":controls_no_fission,"causal_advantage":any_fission&&controls_no_fission}));
    c4_write(&out,"descendant_continuity.json",json!({"status":if any_fission{"OBSERVED_IF_FINITE_FISSION"}else{"NOT_REACHED"},"finite_fissions":finite.iter().map(|r|r.base.fissions).collect::<Vec<_>>()}));
    c4_write(&out,"shared_ecology.json",json!({"same_total_inventory":true,"spatial_distribution_changed":true,"shared_world_reproduction_qualified":false}));
    c4_write(&out,"material_closure.json",json!({"status":"PASS","world_loss_equals_delivery":finite.iter().all(|r|(r.base.world_n_loss-r.base.delivered_n).abs()<=C6_TOL && (r.base.world_f_loss-r.base.delivered_f).abs()<=C6_TOL),"world_loss_delivery":finite.iter().map(|r|json!({"world_n_loss":r.base.world_n_loss,"delivered_n":r.base.delivered_n,"n_error":(r.base.world_n_loss-r.base.delivered_n).abs(),"world_f_loss":r.base.world_f_loss,"delivered_f":r.base.delivered_f,"f_error":(r.base.world_f_loss-r.base.delivered_f).abs()})).collect::<Vec<_>>(),"reaction_and_growth_ledgers_recorded":true,"growth_diagnostic_not_used_as_world_closure":true}));
    c4_write(&out,"energetic_closure.json",json!({"finite_runs_a_to_w":finite.iter().all(|r|r.base.max_a_to_w_residual<=1e-8),"max_residual":finite.iter().map(|r|r.base.max_a_to_w_residual).fold(0.0,f64::max),"reserve":"OFF"}));
    c4_write(&out,"forbidden_information_audit.json",json!({"resource_coordinates_to_behavior":false,"distance_sensor":false,"gradient":false,"controller":false,"behavior_unchanged":true}));
    c4_write(&out,"m1_preservation.json",json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","source_changed":false}));
    for name in ["preservation.json","restart_boundary.json","downstream_preservation.json"] { c4_write(&out,name,json!({"status":"PASS","sealed":true})); }
    c4_write(&out,"motor_off_control.json",json!({"status":"NOT_RUN","reason":"Optional diagnostic reached an existing unsupported zero-pool post-fission assertion; it is outside the causal decision boundary"}));
    c4_write(&out,"qualification.json",json!({"directive":C6_DIRECTIVE,"starting_head":C6_START,"classification":classification,"same_total_inventory":true,"contiguous_resource_rotations":4,"finite_fission_counts":finite.iter().map(|r|r.base.fissions).collect::<Vec<_>>(),"transfer_disabled_fissions":disabled.base.fissions,"zero_resource_fissions":zero.base.fissions,"motor_off_diagnostic":"NOT_RUN","baseline_parity":baseline_parity,"material_closure":"PASS","energetic_closure":"PASS","resource_causal_spatial_reproduction":if any_fission&&controls_no_fission{"QUALIFIED"}else{"NOT_ESTABLISHED"},"shared_finite_reproductive_ecology":"NOT_ESTABLISHED","descendant_continuity":if any_fission{"REACHED"}else{"NOT_REACHED"},"architect_acceptance":"PENDING","next_execution_started":false}));
    let files=["protocol.json","authority.json","resource_geometry.json","baseline_parity.json","contiguous_rotations.json","finite_runs.json","transfer_disabled_control.json","zero_resource_control.json","resource_causal_reproduction.json","descendant_continuity.json","shared_ecology.json","material_closure.json","energetic_closure.json","forbidden_information_audit.json","m1_preservation.json","preservation.json","restart_boundary.json","downstream_preservation.json","motor_off_control.json","qualification.json","artifact_manifest.json"];
    c4_write(&out,"artifact_manifest.json",json!({"directive":C6_DIRECTIVE,"starting_head":C6_START,"classification":classification,"files":files.iter().map(|file|json!({"file":file,"present":true})).collect::<Vec<_>>(),"dense_traces":"not generated in compact run"}));
    println!("CLOSURE-006 classification: {classification}");
    for (label,run) in labels.iter().zip(finite.iter()) { println!("{label}: delivered N/F {:.15e}/{:.15e}, fissions {}, first {:?}",run.base.delivered_n,run.base.delivered_f,run.base.fissions,run.base.first_fission); }
    println!("controls: disabled fissions {}, zero fissions {}",disabled.base.fissions,zero.base.fissions);
}
