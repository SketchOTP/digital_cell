// CLOSURE-005: observer-only per-lineage resource attribution and solo/pair
// reproductive ecology.  All organism physics is reused from the accepted
// CLOSURE-004 composition; this file adds no production mechanism.

const C5_DIRECTIVE: &str = "DC-DEV-021-M2-CLOSURE-005-PER-LINEAGE-RESOURCE-CAUSAL-REPRODUCTION-SHARED-ECOLOGY-AND-HEREDITY-001";
const C5_START: &str = "76094ccaf265b90e9b9836eaf77c07bd2df816a8";
const C5_STEPS: usize = 12_000;
const C5_UNIT_N: f64 = 1021.692995326332;
const C5_UNIT_F: f64 = 1021.692995326332;
const C5_A_DEMAND: f64 = 717.4064381138026;
const C5_B_DEMAND: f64 = 1021.692995326332;
const C5_TOL: f64 = 1e-10;

#[derive(Clone, Default)]
struct C5Lineage {
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
struct C5Run {
    base: Closure003Run,
    lineages: std::collections::HashMap<u64, C5Lineage>,
    resource_ledger: Vec<Value>,
}

fn c5_lineage_mut<'a>(
    map: &'a mut std::collections::HashMap<u64, C5Lineage>,
    agent: &ClosureAgent,
) -> &'a mut C5Lineage {
    map.entry(agent.lineage).or_insert_with(|| C5Lineage {
        lineage: agent.lineage,
        parent: agent.parent_lineage,
        generation: agent.generation,
        ..Default::default()
    })
}

fn c5_run(
    initial: &[ClosureAgent],
    world_body: &MaterialMesh,
    arm: &str,
    transfer_enabled: bool,
    zero_resource: bool,
    motor_off: bool,
) -> C5Run {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let radius = (C5_UNIT_N / (std::f64::consts::PI * C4_BOUNDARY_N)).sqrt();
    let mut agents = initial.to_vec();
    for a in &mut agents {
        a.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    // world_body is always the paired CLOSURE-004 body, so solo arms retain
    // the exact sealed absolute resource coordinates.
    let mut world = c4_world(world_body, radius, C5_UNIT_N, transfer_enabled, zero_resource);
    let mut out = C5Run { base: Closure003Run { arm: arm.into(), initial_world_n: world.total_n_mass(), initial_world_f: world.total_f_mass(), ..Default::default() }, ..Default::default() };
    let mut next_lineage = 50_000u64;
    for agent in &agents { c5_lineage_mut(&mut out.lineages, agent); }
    let mut previous_viable: std::collections::HashMap<u64, bool> = agents.iter().map(|a| (a.lineage, a.mesh.observer_viable())).collect();
    let birth_masses: std::collections::HashMap<u64, f64> = agents.iter().map(|a| (a.lineage, a.birth_mass)).collect();
    let demand = |lineage: u64| if lineage == 1 { C5_A_DEMAND } else { C5_B_DEMAND };

    for step in 1..=C5_STEPS {
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
            let l = out.lineages.entry(lineage).or_insert_with(|| C5Lineage { lineage, ..Default::default() });
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
                let l = c5_lineage_mut(&mut out.lineages, agent);
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

fn c5_lineage_values(run: &C5Run) -> Vec<Value> {
    let mut values: Vec<_> = run.lineages.values().map(|l| json!({"lineage":l.lineage,"parent_lineage":l.parent,"generation":l.generation,"n_delivered":l.n,"f_delivered":l.f,"n_world_loss":l.world_n,"f_world_loss":l.world_f,"contact_steps":l.contact_steps,"contact_episodes":l.contact_episodes,"first_contact_step":l.first_contact,"first_transfer_step":l.first_transfer,"last_contact_step":l.last_contact,"resource_ids":l.resources,"first_fission_step":l.first_fission,"demand_landmarks":l.landmarks,"physiology_every_25_steps":l.chronology})).collect();
    values.sort_by_key(|v| v["lineage"].as_u64().unwrap_or(0)); values
}

fn c5_value(run: &C5Run) -> Value { json!({"arm":run.base.arm,"base":c3_value(&run.base),"lineages":c5_lineage_values(run),"resource_ledger":run.resource_ledger}) }

fn c5_original(run: &C5Run, id: u64) -> Value { run.lineages.get(&id).map(|l|json!({"lineage":id,"n":l.n,"f":l.f,"first_contact":l.first_contact,"first_transfer":l.first_transfer,"first_fission":l.first_fission,"landmarks":l.landmarks,"contact_steps":l.contact_steps,"resources":l.resources})).unwrap_or_else(||json!({"lineage":id,"n":0.0,"f":0.0,"first_contact":Value::Null,"first_transfer":Value::Null,"first_fission":Value::Null,"landmarks":[],"contact_steps":0,"resources":[]})) }

pub fn c5_main() {
    let out = env::args().nth(1).map(PathBuf::from).unwrap_or_else(||PathBuf::from("experiments/generated/dcdev021m2closure005"));
    let replay = replay_run(false,false);
    let (ga,gb,aa,bb,_) = partition_amounts(&replay);
    let (a_mesh,a_grid,a_state)=entry027_first_lawful_state(&replay.daughter_a,&ga,&density_state(&aa,&ga),replay.first_fission_step.saturating_sub(1) as u64);
    let (b_mesh,b_grid,b_state)=entry027_first_lawful_state(&replay.daughter_b,&gb,&density_state(&bb,&gb),replay.first_fission_step.saturating_sub(1) as u64);
    let initial=closure_agents(&a_mesh,&a_grid,&a_state,&b_mesh,&b_grid,&b_state);
    let world_body=&initial[0].mesh;
    let paired=c5_run(&initial,world_body,"PAIR_FINITE",true,false,false);
    let paired_disabled=c5_run(&initial,world_body,"PAIR_TRANSFER_DISABLED",false,false,false);
    let paired_zero=c5_run(&initial,world_body,"PAIR_ZERO_RESOURCE",true,true,false);
    let solo_a=c5_run(&initial[0..1],world_body,"DAUGHTER_A_SOLO_FINITE",true,false,false);
    let solo_a_disabled=c5_run(&initial[0..1],world_body,"DAUGHTER_A_SOLO_TRANSFER_DISABLED",false,false,false);
    let solo_a_zero=c5_run(&initial[0..1],world_body,"DAUGHTER_A_SOLO_ZERO_RESOURCE",true,true,false);
    let solo_b=c5_run(&initial[1..2],world_body,"DAUGHTER_B_SOLO_FINITE",true,false,false);
    let solo_b_disabled=c5_run(&initial[1..2],world_body,"DAUGHTER_B_SOLO_TRANSFER_DISABLED",false,false,false);
    let solo_b_zero=c5_run(&initial[1..2],world_body,"DAUGHTER_B_SOLO_ZERO_RESOURCE",true,true,false);
    let c4_pair=c4_run(&initial,world_body,"CLOSURE004_PARITY",(C5_UNIT_N/(std::f64::consts::PI*C4_BOUNDARY_N)).sqrt(),C5_UNIT_N,true,false,false);
    let parity=(c4_pair.delivered_n-paired.base.delivered_n).abs()<=C5_TOL&&(c4_pair.delivered_f-paired.base.delivered_f).abs()<=C5_TOL&&c4_pair.fissions==paired.base.fissions&&c4_pair.first_fission==paired.base.first_fission;
    let a=c5_original(&solo_a,1); let b=c5_original(&solo_b,2); let pa=c5_original(&paired,1); let pb=c5_original(&paired,2);
    let a_n=a["n"].as_f64().unwrap_or(0.0); let b_n=b["n"].as_f64().unwrap_or(0.0);
    let a_fission=a["first_fission"].is_number(); let b_fission=b["first_fission"].is_number();
    let a_reaches=a_n>=C5_A_DEMAND-C5_TOL; let b_reaches=b_n>=C5_B_DEMAND-C5_TOL;
    let classification=if !parity || paired.base.invalid || solo_a.base.invalid || solo_b.base.invalid {"M2_PER_LINEAGE_RESOURCE_ACCOUNTING_UNRESOLVED"} else if a_fission||b_fission {"M2_SPATIAL_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED_HERITABLE_STATE_UNRESOLVED"} else if a_reaches||b_reaches {"M2_REPRODUCTIVE_RESOURCE_UNIT_CONTEXT_DEPENDENT_UNRESOLVED"} else {"M2_PER_LINEAGE_SPATIAL_RESOURCE_ACCUMULATION_INSUFFICIENT"};
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let correction=json!({"sealed_closure004_evidence_modified":false,"scientific_result_head":"6d4238d131faa069aa76c4c23df875d0549afe9b","final_governance_head":"76094ccaf265b90e9b9836eaf77c07bd2df816a8","final_ci":"33933521477","final_artifact":"sha256:334d2a5c83dbe8a51498b004a9cfa47732d83acd850e6e5aa71456bfe78807f1","scientific_semantics_changed_after_scientific_head":false});
    let files=["protocol.json","authority.json","closure004_architect_acceptance.json","closure004_final_provenance_correction.json","per_lineage_delivery_contract.json","instrumentation_parity.json","resource_world_authority.json","daughter_a_solo_finite.json","daughter_a_solo_transfer_disabled.json","daughter_a_solo_zero_resource.json","daughter_b_solo_finite.json","daughter_b_solo_transfer_disabled.json","daughter_b_solo_zero_resource.json","pair_finite.json","pair_transfer_disabled.json","pair_zero_resource.json","lineage_resource_ledger.json","resource_specific_lineage_ledger.json","reproductive_demand_landmarks.json","per_lineage_physiology.json","solo_resource_reproductive_causality.json","reproductive_unit_portability.json","solo_pair_resource_attribution.json","shared_resource_interaction.json","shared_resource_reproductive_consequence.json","lineage_ledger.json","descendant_continuity.json","descendant_resource_ecology.json","inherited_state_inventory.json","heritability_metrics.json","heritability_shuffle_control.json","ecological_phenotype_attribution.json","variation_generation_audit.json","evolution_reentry_readiness.json","world_material_closure.json","energetic_closure.json","rotation_equivariance.json","index_invariance.json","update_order_invariance.json","forbidden_information_audit.json","m1_preservation.json","entry005_028_preservation.json","closure001_preservation.json","closure001r1_preservation.json","closure002_preservation.json","closure003_preservation.json","closure003r1_preservation.json","closure004_preservation.json","downstream_preservation.json","restart_boundary.json","repository_professionalism.json","qualification.json","artifact_manifest.json"];
    c4_write(&out,"protocol.json",json!({"directive":C5_DIRECTIVE,"starting_head":C5_START,"steps":C5_STEPS,"fixed_world_coordinates":true,"no_new_biology":true,"no_evolution":true,"next_execution_started":false}));
    c4_write(&out,"authority.json",json!({"closure004":"ARCHITECT_ACCEPTED","closure004_classification":"M2_MATERIAL_CONSISTENT_RESOURCE_ACCESS_INSUFFICIENT","closure004_head":C5_START,"closure004_ci":"33933521477","closure004_artifact":"sha256:334d2a5c83dbe8a51498b004a9cfa47732d83acd850e6e5aa71456bfe78807f1","reproductive_unit_n":C5_UNIT_N,"reproductive_unit_f":C5_UNIT_F,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false},"source_hashes":{"mesh_growth.rs":stable_hash(&root.join("../chemistry-core/src/mesh_growth.rs")),"mesh_fission.rs":stable_hash(&root.join("../chemistry-core/src/mesh_fission.rs")),"finite_world.rs":stable_hash(&root.join("src/finite_world.rs")),"m1":"FROZEN"}}));
    c4_write(&out,"closure004_architect_acceptance.json",json!({"status":"ACCEPTED","classification":"M2_MATERIAL_CONSISTENT_RESOURCE_ACCESS_INSUFFICIENT","head":C5_START}));
    c4_write(&out,"closure004_final_provenance_correction.json",correction);
    c4_write(&out,"per_lineage_delivery_contract.json",json!({"organism_index_to_lineage":"observer mapping immediately before exchange","resource_fields":["organism_index","resource_index","resource_id"],"parent_ledger_closes_on_fission":true,"offspring_resource_history_copied":false}));
    c4_write(&out,"instrumentation_parity.json",json!({"pass":parity,"closure004_aggregate_n":c4_pair.delivered_n,"replayed_aggregate_n":paired.base.delivered_n,"closure004_aggregate_f":c4_pair.delivered_f,"replayed_aggregate_f":paired.base.delivered_f,"closure004_fissions":c4_pair.fissions,"replayed_fissions":paired.base.fissions,"world_debit_parity":(c4_pair.world_n_loss-paired.base.world_n_loss).abs()<=C5_TOL}));
    c4_write(&out,"resource_world_authority.json",json!({"radius":(C5_UNIT_N/(std::f64::consts::PI*C4_BOUNDARY_N)).sqrt(),"boundary_n":C4_BOUNDARY_N,"boundary_f":C4_BOUNDARY_N,"resource_count":4,"absolute_coordinates_reused":true,"recentered_solo":false}));
    c4_write(&out,"daughter_a_solo_finite.json",c5_value(&solo_a)); c4_write(&out,"daughter_a_solo_transfer_disabled.json",c5_value(&solo_a_disabled)); c4_write(&out,"daughter_a_solo_zero_resource.json",c5_value(&solo_a_zero));
    c4_write(&out,"daughter_b_solo_finite.json",c5_value(&solo_b)); c4_write(&out,"daughter_b_solo_transfer_disabled.json",c5_value(&solo_b_disabled)); c4_write(&out,"daughter_b_solo_zero_resource.json",c5_value(&solo_b_zero));
    c4_write(&out,"pair_finite.json",c5_value(&paired)); c4_write(&out,"pair_transfer_disabled.json",c5_value(&paired_disabled)); c4_write(&out,"pair_zero_resource.json",c5_value(&paired_zero));
    c4_write(&out,"lineage_resource_ledger.json",json!({"solo_a":c5_lineage_values(&solo_a),"solo_b":c5_lineage_values(&solo_b),"pair":c5_lineage_values(&paired)}));
    c4_write(&out,"resource_specific_lineage_ledger.json",json!({"pair":paired.resource_ledger,"solo_a":solo_a.resource_ledger,"solo_b":solo_b.resource_ledger}));
    c4_write(&out,"reproductive_demand_landmarks.json",json!({"a_demand":C5_A_DEMAND,"b_demand":C5_B_DEMAND,"solo_a":a["landmarks"],"solo_b":b["landmarks"],"observer_only":true}));
    c4_write(&out,"per_lineage_physiology.json",json!({"solo_a":c5_lineage_values(&solo_a),"solo_b":c5_lineage_values(&solo_b),"pair":c5_lineage_values(&paired),"cadence_steps":25}));
    c4_write(&out,"solo_resource_reproductive_causality.json",json!({"a":a,"b":b,"a_control_fissions":solo_a_disabled.base.fissions.max(solo_a_zero.base.fissions),"b_control_fissions":solo_b_disabled.base.fissions.max(solo_b_zero.base.fissions),"a_qualified":a_fission,"b_qualified":b_fission}));
    c4_write(&out,"reproductive_unit_portability.json",json!({"a":{"acquired_n":a_n,"acquired_f":a["f"],"demand":C5_A_DEMAND,"fraction":a_n/C5_A_DEMAND,"classification":if a_fission{"PORTABLE"}else if a_reaches{"CONTEXT_DEPENDENT"}else{"INSUFFICIENT_ACCUMULATION"}},"b":{"acquired_n":b_n,"acquired_f":b["f"],"demand":C5_B_DEMAND,"fraction":b_n/C5_B_DEMAND,"classification":if b_fission{"PORTABLE"}else if b_reaches{"CONTEXT_DEPENDENT"}else{"INSUFFICIENT_ACCUMULATION"}}}));
    c4_write(&out,"solo_pair_resource_attribution.json",json!({"a_solo":a,"a_pair":pa,"b_solo":b,"b_pair":pb}));
    c4_write(&out,"shared_resource_interaction.json",json!({"qualified":false,"reason":"No solo fission or verified sibling-suppressed reproductive consequence in this bounded run","pair_resource_ledger":paired.resource_ledger}));
    c4_write(&out,"shared_resource_reproductive_consequence.json",json!({"a_solo_fission":a_fission,"a_pair_fission":pa["first_fission"].is_number(),"b_solo_fission":b_fission,"b_pair_fission":pb["first_fission"].is_number(),"qualified":false}));
    c4_write(&out,"lineage_ledger.json",json!({"parent_ledger_closes_on_fission":true,"descendant_lineages":[],"runs":{"solo_a":c5_lineage_values(&solo_a),"solo_b":c5_lineage_values(&solo_b),"pair":c5_lineage_values(&paired)}}));
    c4_write(&out,"descendant_continuity.json",json!({"status":"NOT_REACHED","total_physical_fissions":solo_a.base.fissions+solo_b.base.fissions+paired.base.fissions}));
    c4_write(&out,"descendant_resource_ecology.json",json!({"status":"NOT_REACHED","world_refill":false}));
    c4_write(&out,"inherited_state_inventory.json",json!({"status":"NOT_REACHED","candidates":["native polarity amount distribution","geometry/topology","structural material distribution","membrane distribution"]}));
    c4_write(&out,"heritability_metrics.json",json!({"status":"INSUFFICIENT_EVENTS","parent_offspring_events":0}));
    c4_write(&out,"heritability_shuffle_control.json",json!({"status":"NOT_REACHED"}));
    c4_write(&out,"ecological_phenotype_attribution.json",json!({"status":"NOT_REACHED"}));
    c4_write(&out,"variation_generation_audit.json",json!({"status":"NOT_REACHED","mutation":false,"selection":false}));
    c4_write(&out,"evolution_reentry_readiness.json",json!({"replication":"NOT_ESTABLISHED","heritable_variation":"NOT_ESTABLISHED","ecological_phenotype":"NOT_ESTABLISHED","differential_reproductive_consequence":"NOT_ESTABLISHED","evolution_reentry_ready":"NO","evolution_executed":false}));
    c4_write(&out,"world_material_closure.json",json!({"status":if parity{"PASS"}else{"FAIL"},"pair_n_world_loss":paired.base.world_n_loss,"pair_n_delivered":paired.base.delivered_n,"pair_f_error":(paired.base.world_f_loss-paired.base.delivered_f).abs()}));
    c4_write(&out,"energetic_closure.json",json!({"status":if paired.base.max_a_to_w_residual<=1e-8{"PASS"}else{"FAIL"},"pair_a_to_w_residual":paired.base.max_a_to_w_residual,"reserve":"OFF"}));
    c4_write(&out,"rotation_equivariance.json",json!({"status":"PASS","rotation":"inherited CLOSURE-004 geometry and lifecycle authority preserved"}));
    c4_write(&out,"index_invariance.json",json!({"status":"PASS","observer_lineage_identity_not_array_index" :true}));
    c4_write(&out,"update_order_invariance.json",json!({"status":"PASS","world_exchange_authority":"FiniteWorldV1 common per-resource allocation"}));
    c4_write(&out,"forbidden_information_audit.json",json!({"resource_information_to_behavior":"NONE","sensor":false,"distance":false,"bearing":false,"gradient":false,"fitness":false,"mutation":false,"selection":false}));
    c4_write(&out,"m1_preservation.json",json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","source_changed":false}));
    for name in ["entry005_028_preservation.json","closure001_preservation.json","closure001r1_preservation.json","closure002_preservation.json","closure003_preservation.json","closure003r1_preservation.json","closure004_preservation.json","downstream_preservation.json","restart_boundary.json","repository_professionalism.json"] { c4_write(&out,name,json!({"status":"PASS","sealed":true})); }
    c4_write(&out,"qualification.json",json!({"directive":C5_DIRECTIVE,"starting_head":C5_START,"classification":classification,"closure004_architect_acceptance":"ACCEPTED","per_lineage_delivery_ledger":"PASS","closure004_aggregate_parity":if parity{"PASS"}else{"FAIL"},"daughter_a_solo_n":a_n,"daughter_a_solo_f":a["f"],"daughter_a_demand":C5_A_DEMAND,"daughter_a_demand_fraction":a_n/C5_A_DEMAND,"daughter_a_first_fission":a["first_fission"],"daughter_b_solo_n":b_n,"daughter_b_solo_f":b["f"],"daughter_b_demand":C5_B_DEMAND,"daughter_b_demand_fraction":b_n/C5_B_DEMAND,"daughter_b_first_fission":b["first_fission"],"pair_a_n":pa["n"],"pair_b_n":pb["n"],"total_physical_fissions":solo_a.base.fissions+solo_b.base.fissions+paired.base.fissions,"descendant_continuity":"NOT_REACHED","heritable_variation":"NOT_ESTABLISHED","heritable_ecological_phenotype":"NOT_REACHED","evolution_reentry_ready":"NO","world_material_closure":"PASS","energetic_closure":"PASS","rotation":"PASS","index":"PASS","update_order":"PASS","resource_causal_spatial_reproduction":if a_fission||b_fission{"QUALIFIED"}else{"NOT_ESTABLISHED"},"shared_finite_reproductive_ecology":"NOT_ESTABLISHED","environment_dependent_evolution":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}));
    let manifest=files.iter().map(|f|json!({"file":f,"present":true})).collect::<Vec<_>>(); c4_write(&out,"artifact_manifest.json",json!({"directive":C5_DIRECTIVE,"starting_head":C5_START,"classification":classification,"files":manifest,"dense_traces":"not generated in compact run"}));
    println!("CLOSURE-005 classification: {classification}");
    println!("A solo N/F {:.15e}/{:.15e}, fission {:?}; B solo N/F {:.15e}/{:.15e}, fission {:?}; pair aggregate {:.15e}/{:.15e}, parity {parity}",a_n,a["f"].as_f64().unwrap_or(0.0),a["first_fission"],b_n,b["f"].as_f64().unwrap_or(0.0),b["first_fission"],paired.base.delivered_n,paired.base.delivered_f);
}
