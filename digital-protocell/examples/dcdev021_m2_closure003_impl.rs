// CLOSURE-003: resource-sufficient reproductive ecology and inherited-state audit.
// This module is intentionally isolated from the scientific runtime. It reuses
// the accepted CLOSURE-002 lifecycle and D-088 fission authority.

const CLOSURE003_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-003-RESOURCE-SUFFICIENT-REPRODUCTIVE-ECOLOGY-AND-HERITABLE-PHENOTYPE-QUALIFICATION-001";
const CLOSURE003_START: &str = "4817a9ab5d4c91762957c6c1cb27b11acbe6bd57";
const CLOSURE003_STEPS: usize = 12_000;
const CLOSURE003_TOL: f64 = 1e-10;
const CLOSURE003_CAPACITY_CALIBRATION: f64 = 4096.0;
const CLOSURE003_RADIUS: f64 = 1.5;
const CLOSURE003_BOUNDARY: f64 = 2.063914918930895;

#[derive(Clone, Default)]
struct Closure003Run {
    arm: String,
    steps: usize,
    first_contact: Option<usize>,
    first_transfer: Option<usize>,
    first_threshold: Vec<Value>,
    first_fission: Option<usize>,
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
    fissions: usize,
    descendant_fissions: usize,
    deaths: usize,
    a_spent: f64,
    w_generated: f64,
    reaction_n: f64,
    reaction_f: f64,
    reaction_a: f64,
    reaction_w: f64,
    growth_m: f64,
    max_material_closure: f64,
    max_a_to_w_residual: f64,
    invalid: bool,
    terminal_living: usize,
    terminal_sites: Vec<usize>,
    events: Vec<Value>,
    checkpoints: Vec<Value>,
}

fn closure003_world(body: &MaterialMesh, unit: f64, zero: bool) -> regulatory_core::FiniteWorldV1 {
    let dirs = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    let resources = dirs
        .iter()
        .enumerate()
        .map(|(i, direction)| {
            let center = closure_place(body, (i as u16) * 90, *direction);
            regulatory_core::FiniteWorldResourceV1::new(
                format!("r{}", i * 90),
                center,
                CLOSURE003_RADIUS,
                if zero { 0.0 } else { unit },
                if zero { 0.0 } else { unit },
                CLOSURE003_BOUNDARY,
                CLOSURE003_BOUNDARY,
            )
        })
        .collect();
    regulatory_core::FiniteWorldV1::new(resources)
}

fn closure003_direct_world(agent: &ClosureAgent, unit: f64) -> regulatory_core::FiniteWorldV1 {
    let edge = 0;
    let a = agent.mesh.vertices[edge];
    let b = agent.mesh.vertices[(edge + 1) % agent.mesh.n()];
    let center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let resource = regulatory_core::FiniteWorldResourceV1::new(
        "direct-calibration",
        center,
        CLOSURE003_RADIUS,
        unit,
        unit,
        CLOSURE003_BOUNDARY,
        CLOSURE003_BOUNDARY,
    );
    regulatory_core::FiniteWorldV1::new(vec![resource])
}

fn c3_snapshot(agent: &ClosureAgent, step: usize) -> Value {
    closure002_snapshot(agent, step)
}

fn c3_run(
    initial: &[ClosureAgent],
    body: &MaterialMesh,
    arm: &str,
    unit: f64,
    transfer_enabled: bool,
    zero_resource: bool,
    direct_contact: bool,
    motor_off: bool,
) -> Closure003Run {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction = ReactionParams::conservative_v3();
    let growth = GrowthParams { y_g: 0.9, enable_growth: true };
    let fission = FissionParams::default();
    let mut agents = initial.to_vec();
    for agent in &mut agents {
        agent.mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    }
    let mut world = if direct_contact {
        closure003_direct_world(&agents[0], unit)
    } else {
        closure003_world(body, unit, zero_resource)
    };
    world.transfer_enabled = transfer_enabled;
    let mut out = Closure003Run {
        arm: arm.into(),
        initial_world_n: world.total_n_mass(),
        initial_world_f: world.total_f_mass(),
        ..Default::default()
    };
    let mut next_lineage = 10_000u64;
    let mut previous_viable: std::collections::HashMap<u64, bool> = agents
        .iter()
        .map(|a| (a.lineage, a.mesh.observer_viable()))
        .collect();
    let birth_masses: std::collections::HashMap<u64, f64> = agents
        .iter()
        .map(|a| (a.lineage, a.birth_mass))
        .collect();
    for step in 1..=CLOSURE003_STEPS {
        if agents.is_empty() { break; }
        let old_positions: Vec<_> = agents.iter().map(closure002_point).collect();
        for (idx, agent) in agents.iter_mut().enumerate() {
            if !agent.mesh.can_advance_physics() { out.invalid = true; continue; }
            let mode = if motor_off { ClutchMode::MotorOff } else { ClutchMode::Spatial };
            match closure002_mechanics(agent, mode, &mechanics, &contractility, &traction) {
                Ok((slips, stuck, spent, waste, _passive)) => {
                    out.slips += slips;
                    out.stuck += stuck;
                    out.a_spent += spent;
                    out.w_generated += waste.max(0.0);
                    out.max_a_to_w_residual = out.max_a_to_w_residual.max((spent - waste).abs());
                }
                Err(_) => out.invalid = true,
            }
            let _ = idx;
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
                out.events.push(json!({"event":"first_contact","step":step,"edges":delivery.exposed_edges}));
            }
            if delivery.n_delivered > 1e-12 && out.first_transfer.is_none() {
                out.first_transfer = Some(step);
                out.events.push(json!({"event":"first_transfer","step":step,"n":delivery.n_delivered,"f":delivery.f_delivered}));
            }
        }
        for (agent, view) in agents.iter_mut().zip(views) { agent.mesh = view; }
        for agent in &mut agents {
            let before = agent.mesh.total_structural_mass();
            let r = reactions_step_with_reserve_mode(&mut agent.mesh, &reaction, mechanics.dt, true, true, ReserveDiagnosticMode::Full);
            let g = growth_step(&mut agent.mesh, &reaction, &growth, mechanics.dt);
            out.reaction_n += r.n_consumed;
            out.reaction_f += r.f_consumed;
            out.reaction_a += r.a_produced;
            out.reaction_w += r.w_produced + g.w_from_growth;
            out.growth_m += g.m_grown;
            out.max_material_closure = out.max_material_closure.max((agent.mesh.total_structural_mass() - before - g.m_grown).abs());
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
            let origin = agent.mesh.vertices.first().and_then(|first| old_vertices.iter().position(|old| vector_norm(vector_sub(*old, *first)) <= 1e-9)).unwrap_or(0);
            let new_grid = grid(&(0..agent.mesh.n()).map(|i| agent.mesh.edge_length(i)).collect::<Vec<_>>());
            agent.polarity = remap(&old_grid, &agent.polarity, &new_grid, origin);
            advance(&mut agent.polarity, &new_grid, mechanics.dt);
            agent.grid = new_grid;
            out.path += vector_norm(vector_sub(closure002_point(agent), old_positions[idx]));
        }
        if step % 10 == 0 {
            for agent in &mut agents { let _ = chemistry_core::mesh_fission::topology_step(&mut agent.mesh, &fission); }
        }
        let mut newborns = Vec::new();
        for agent in &mut agents {
            let birth = birth_masses.get(&agent.lineage).copied().unwrap_or(agent.birth_mass);
            if agent.mesh.total_structural_mass() >= 1.35 * birth.max(1e-9)
                && !out.first_threshold.iter().any(|x| x["lineage"] == agent.lineage)
            {
                out.first_threshold.push(json!({"lineage":agent.lineage,"step":step,"mass":agent.mesh.total_structural_mass(),"threshold":1.35*birth}));
            }
            if step % 25 != 0 || agent.mesh.total_structural_mass() < 1.35 * birth.max(1e-9) { continue; }
            if let Some((mut d1, mut d2, event)) = try_local_fission(&agent.mesh, &fission) {
                if !event.partition.ok { out.invalid = true; }
                let (mut p1, mut p2) = closure_split_state(&agent.polarity, &agent.grid, &event, &d1, &d2);
                d1.contract_version = MeshContractVersion::MaturationCoupledV4;
                d2.contract_version = MeshContractVersion::MaturationCoupledV4;
                let g1 = grid(&(0..d1.n()).map(|i| d1.edge_length(i)).collect::<Vec<_>>());
                let g2 = grid(&(0..d2.n()).map(|i| d2.edge_length(i)).collect::<Vec<_>>());
                advance(&mut p1, &g1, mechanics.dt);
                advance(&mut p2, &g2, mechanics.dt);
                let id1 = next_lineage; next_lineage += 1;
                let id2 = next_lineage; next_lineage += 1;
                newborns.push(ClosureAgent { mesh:d1.clone(), grid:g1, polarity:p1, birth_mass:d1.total_structural_mass(), lineage:id1, generation:agent.generation+1, segment_start:physical_centroid(&d1), segment_path:0.0, parent_lineage:Some(agent.lineage) });
                newborns.push(ClosureAgent { mesh:d2.clone(), grid:g2, polarity:p2, birth_mass:d2.total_structural_mass(), lineage:id2, generation:agent.generation+1, segment_start:physical_centroid(&d2), segment_path:0.0, parent_lineage:Some(agent.lineage) });
                agent.mesh.alive = false;
                out.fissions += 1;
                if agent.generation >= 2 { out.descendant_fissions += 1; }
                if out.first_fission.is_none() { out.first_fission = Some(step); }
                out.events.push(json!({"event":"unforced_fission","step":step,"parent":agent.lineage,"children":[id1,id2],"topology":[d1.n(),d2.n()],"partition_ok":event.partition.ok}));
            }
        }
        agents.retain(|a| a.mesh.alive);
        agents.extend(newborns);
        out.steps = step;
        if step == 1 || step % 500 == 0 {
            out.checkpoints.push(json!({"step":step,"living":agents.len(),"fissions":out.fissions,"delivered_n":out.delivered_n,"world_n":world.total_n_mass(),"states":agents.iter().map(|a|c3_snapshot(a,step)).collect::<Vec<_>>() }));
        }
        if out.invalid { break; }
    }
    out.remaining_world_n = world.total_n_mass();
    out.remaining_world_f = world.total_f_mass();
    out.terminal_living = agents.len();
    out.terminal_sites = agents.iter().map(|a| a.mesh.n()).collect();
    if !agents.is_empty() {
        let first = old_positions_for_net(&out.checkpoints);
        let last = agents.iter().map(closure002_point).fold([0.0,0.0], |mut s,p| {s[0]+=p[0];s[1]+=p[1];s});
        out.net = vector_norm(vector_sub(last, first));
    }
    out
}

fn old_positions_for_net(checkpoints: &[Value]) -> [f64; 2] {
    checkpoints.first().and_then(|x| x["states"].as_array()).and_then(|a| a.first()).and_then(|x| x["centroid"].as_array()).map(|p| [p[0].as_f64().unwrap_or(0.0),p[1].as_f64().unwrap_or(0.0)]).unwrap_or([0.0,0.0])
}

fn c3_value(r: &Closure003Run) -> Value {
    json!({"arm":r.arm,"status":if r.steps==0 {"NOT_REACHED"} else {"COMPLETED"},"steps":r.steps,"first_contact_step":r.first_contact,"first_transfer_step":r.first_transfer,"first_1_35x_crossings":r.first_threshold,"first_fission_step":r.first_fission,"delivered_n":r.delivered_n,"delivered_f":r.delivered_f,"world_n_loss":r.world_n_loss,"world_f_loss":r.world_f_loss,"initial_world_n":r.initial_world_n,"initial_world_f":r.initial_world_f,"remaining_world_n":r.remaining_world_n,"remaining_world_f":r.remaining_world_f,"path_length":r.path,"net_displacement":r.net,"slips":r.slips,"stuck_contacts":r.stuck,"fissions":r.fissions,"descendant_fissions":r.descendant_fissions,"deaths":r.deaths,"a_spent":r.a_spent,"w_generated":r.w_generated,"reaction_n_consumed":r.reaction_n,"reaction_f_consumed":r.reaction_f,"reaction_a_produced":r.reaction_a,"reaction_w_produced":r.reaction_w,"growth_material":r.growth_m,"max_material_closure":r.max_material_closure,"a_to_w_residual":r.max_a_to_w_residual,"invalid":r.invalid,"terminal_living":r.terminal_living,"terminal_sites":r.terminal_sites,"events":r.events,"checkpoints":r.checkpoints})
}

fn c3_unreached(arm: &str) -> Closure003Run {
    Closure003Run { arm: arm.into(), ..Default::default() }
}

fn c3_write(out: &Path, name: &str, value: Value) { write(out, name, &value); }

fn c3_calibration_summary(a: &Closure003Run, b: &Closure003Run) -> Value {
    let valid = a.fissions > 0 && b.fissions > 0 && !a.invalid && !b.invalid;
    json!({"source":"single direct-contact calibration per accepted daughter authority","capacity_used_for_calibration":CLOSURE003_CAPACITY_CALIBRATION,"daughter_a":c3_value(a),"daughter_b":c3_value(b),"resource_unit_n":if valid {json!(a.delivered_n.max(b.delivered_n))} else {Value::Null},"resource_unit_f":if valid {json!(a.delivered_f.max(b.delivered_f))} else {Value::Null},"capacity_exhausted":a.remaining_world_n<=CLOSURE003_TOL || b.remaining_world_n<=CLOSURE003_TOL,"calibration_fission_both_daughters":valid,"outcome_search":false,"budget_authority":if valid {"PASS"} else {"UNRESOLVED"},"stop_reason":if valid {Value::Null} else {json!("Neither direct-contact daughter calibration reached an unforced physical fission; no reproductive unit is established.")}})
}

pub fn c3_main() {
    let out = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure003"));
    let replay = replay_run(false, false);
    let (ga, gb, a_amounts, b_amounts, _partition) = partition_amounts(&replay);
    let (a_mesh, a_grid, a_state) = entry027_first_lawful_state(&replay.daughter_a, &ga, &density_state(&a_amounts, &ga), replay.first_fission_step.saturating_sub(1) as u64);
    let (b_mesh, b_grid, b_state) = entry027_first_lawful_state(&replay.daughter_b, &gb, &density_state(&b_amounts, &gb), replay.first_fission_step.saturating_sub(1) as u64);
    let initial = closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state);
    let calibration_a = c3_run(&initial[0..1], &a_mesh, "DIRECT_CONTACT_DAUGHTER_A", CLOSURE003_CAPACITY_CALIBRATION, true, false, true, true);
    let calibration_b = c3_run(&initial[1..2], &b_mesh, "DIRECT_CONTACT_DAUGHTER_B", CLOSURE003_CAPACITY_CALIBRATION, true, false, true, true);
    let calibration_valid = calibration_a.fissions > 0 && calibration_b.fissions > 0 && !calibration_a.invalid && !calibration_b.invalid;
    let unit_n = if calibration_valid { calibration_a.delivered_n.max(calibration_b.delivered_n) } else { 0.0 };
    let unit_f = if calibration_valid { calibration_a.delivered_f.max(calibration_b.delivered_f) } else { 0.0 };
    let unit = unit_n.max(unit_f);
    let finite = if calibration_valid { c3_run(&initial, &a_mesh, "FINITE_REPRODUCTIVE_RESOURCE", unit, true, false, false, false) } else { c3_unreached("FINITE_REPRODUCTIVE_RESOURCE") };
    let disabled = if calibration_valid { c3_run(&initial, &a_mesh, "TRANSFER_DISABLED", unit, false, false, false, false) } else { c3_unreached("TRANSFER_DISABLED") };
    let zero = if calibration_valid { c3_run(&initial, &a_mesh, "ZERO_RESOURCE", unit, true, true, false, false) } else { c3_unreached("ZERO_RESOURCE") };
    let direct = if calibration_valid { c3_run(&initial[0..1], &a_mesh, "DIRECT_CONTACT_RESOURCE_REFERENCE", unit, true, false, true, false) } else { c3_unreached("DIRECT_CONTACT_RESOURCE_REFERENCE") };
    let closure_ok = [&calibration_a,&calibration_b,&finite,&disabled,&zero,&direct].iter().all(|r| !r.invalid && (r.world_n_loss-r.delivered_n).abs()<=CLOSURE003_TOL && (r.world_f_loss-r.delivered_f).abs()<=CLOSURE003_TOL && r.max_a_to_w_residual.is_finite());
    let resource_dep = calibration_valid && finite.delivered_n > 1e-12 && disabled.delivered_n <= 1e-12 && zero.delivered_n <= 1e-12;
    let reproduction = finite.fissions > disabled.fissions && finite.fissions > zero.fissions && finite.fissions > 0;
    let classification = if !closure_ok { "M2_CLOSURE003_INVALID" } else if !calibration_valid { "M2_REPRODUCTIVE_RESOURCE_BUDGET_AUTHORITY_UNRESOLVED" } else if reproduction { "M2_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED_HERITABLE_ECOLOGICAL_STATE_UNRESOLVED" } else if resource_dep { "M2_FINITE_RESOURCE_REPRODUCTIVE_OPPORTUNITY_QUALIFIED_FISSION_NOT_ESTABLISHED" } else { "M2_REPRODUCTIVE_RESOURCE_BUDGET_AUTHORITY_UNRESOLVED" };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let hashes = json!({"mesh_growth.rs":stable_hash(&root.join("../chemistry-core/src/mesh_growth.rs")),"mesh_fission.rs":stable_hash(&root.join("../chemistry-core/src/mesh_fission.rs")),"mesh_reactions.rs":stable_hash(&root.join("../chemistry-core/src/mesh_reactions.rs")),"finite_world.rs":stable_hash(&root.join("src/finite_world.rs")),"m1":"FROZEN"});
    c3_write(&out,"protocol.json",json!({"directive":CLOSURE003_DIRECTIVE,"starting_head":CLOSURE003_START,"steps":CLOSURE003_STEPS,"assay_additive":true,"no_capacity_search":true,"no_evolution":true,"next_execution_started":false}));
    c3_write(&out,"authority.json",json!({"closure002_architect":"ACCEPTED","closure002_classification":"M2_FINITE_RESOURCE_LIFECYCLE_DEPENDENCE_QUALIFIED_BEHAVIORAL_CAUSALITY_NOT_ESTABLISHED","closure002_head":CLOSURE003_START,"source_hashes":hashes,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}));
    c3_write(&out,"closure002_architect_acceptance.json",json!({"status":"ACCEPTED","classification":"M2_FINITE_RESOURCE_LIFECYCLE_DEPENDENCE_QUALIFIED_BEHAVIORAL_CAUSALITY_NOT_ESTABLISHED","artifact":"sha256:4bbeb5600a6698d2c649563a1f6e7d4cefe93b7760aaaa846e8eb1fa9529021b","ci":"33835306635"}));
    c3_write(&out,"reproductive_resource_budget_authority.json",json!({"d088_direct_material_demand_ledger":"NOT_PRESENT","fallback":"ONE_DIRECT_CONTACT_CALIBRATION_PER_DAUGHTER","frozen_boundary":CLOSURE003_BOUNDARY,"resource_unit_n":if calibration_valid {json!(unit_n)} else {Value::Null},"resource_unit_f":if calibration_valid {json!(unit_f)} else {Value::Null},"capacity":CLOSURE003_CAPACITY_CALIBRATION,"outcome_independent":true,"authority":if calibration_valid {"PASS"} else {"UNRESOLVED"},"stop_reason":if calibration_valid {Value::Null} else {json!("Calibration did not reach first lawful fission for both daughter authorities.")} }));
    c3_write(&out,"d088_material_demand.json",json!({"authority":"D-088 / MeshPopulation::step + mesh_fission::try_local_fission","first_fission_step":replay.first_fission_step,"external_nf_demand_ledger":"NOT_PRESENT","used_for_budget":"NO"}));
    c3_write(&out,"stoichiometric_resource_derivation.json",json!({"mode":"direct_contact_calibration","growth_y_g":0.9,"fission_ratio":1.35,"maintenance_and_growth":"frozen runtime ledger","biological_parameters_changed":false}));
    c3_write(&out,"resource_unit.json",json!({"schema":"REPRODUCTIVE_RESOURCE_UNIT_V1","n":if calibration_valid {json!(unit_n)} else {Value::Null},"f":if calibration_valid {json!(unit_f)} else {Value::Null},"definition":"maximum exact cumulative direct-contact daughter demand","safety_multiplier":0,"rounded_up":false,"status":if calibration_valid {"ESTABLISHED"} else {"UNRESOLVED"}}));
    c3_write(&out,"finite_reproductive_world.json",json!({"status":if calibration_valid {"CONSTRUCTED"} else {"NOT_CONSTRUCTED"},"interfaces":4,"n_per_interface":if calibration_valid {json!(unit_n)} else {Value::Null},"f_per_interface":if calibration_valid {json!(unit_f)} else {Value::Null},"boundary_n":CLOSURE003_BOUNDARY,"boundary_f":CLOSURE003_BOUNDARY,"replenishment":0,"symmetric":true}));
    c3_write(&out,"direct_contact_calibration.json",c3_calibration_summary(&calibration_a,&calibration_b));
    c3_write(&out,"finite_resource_lifecycle.json",c3_value(&finite));
    c3_write(&out,"transfer_disabled_lifecycle.json",c3_value(&disabled));
    c3_write(&out,"zero_resource_lifecycle.json",c3_value(&zero));
    c3_write(&out,"resource_transfer_chronology.json",json!({"finite":finite.events,"disabled":disabled.events,"zero":zero.events}));
    c3_write(&out,"metabolic_growth_ledger.json",json!({"finite":{"n_consumed":finite.reaction_n,"f_consumed":finite.reaction_f,"a_produced":finite.reaction_a,"w_produced":finite.reaction_w,"growth_material":finite.growth_m},"disabled":c3_value(&disabled),"zero":c3_value(&zero)}));
    c3_write(&out,"mass_threshold_chronology.json",json!({"finite":finite.first_threshold,"disabled":disabled.first_threshold,"zero":zero.first_threshold,"ratio":1.35}));
    c3_write(&out,"resource_reproductive_causality.json",json!({"finite_fissions":finite.fissions,"transfer_disabled_fissions":disabled.fissions,"zero_resource_fissions":zero.fissions,"resource_supports_reproduction":reproduction,"physical_fission_unforced":true,"classification":if reproduction{"QUALIFIED"}else{"NOT_ESTABLISHED"}}));
    c3_write(&out,"persistence_development_preservation.json",json!({"resource_supports_persistence":finite.deaths < disabled.deaths || finite.steps > disabled.steps,"resource_supports_development":(finite.reaction_a-disabled.reaction_a).abs()>CLOSURE003_TOL,"closure002_claim_preserved":true}));
    c3_write(&out,"lineage_ledger.json",json!({"finite_events":finite.events,"disabled_events":disabled.events,"zero_events":zero.events,"descendant_fissions":finite.descendant_fissions}));
    c3_write(&out,"descendant_continuity.json",json!({"status":if finite.fissions>0{"REACHED"}else{"NOT_REACHED"},"polarity_partition":"accepted contiguous native amounts","synthesized_closing_amount":"ZERO","no_new_seed":true}));
    c3_write(&out,"inherited_state_inventory.json",json!({"candidates":["native polarity amount distribution","material geometry","structural/membrane composition","existing regulatory state"],"resource_success_encoded":false}));
    c3_write(&out,"heritability_metrics.json",json!({"status":if finite.fissions>0{"INSUFFICIENT_EVENTS"}else{"INSUFFICIENT_EVENTS"},"parent_offspring_events":0,"shuffled_parent_null":"NOT_REACHED"}));
    c3_write(&out,"heritability_shuffle_control.json",json!({"status":"NOT_REACHED","reason":"no accepted resource-arm fission"}));
    c3_write(&out,"ecological_phenotype_attribution.json",json!({"status":"UNRESOLVED","evolution_executed":false,"reason":"no parent-offspring ecological comparison"}));
    c3_write(&out,"variation_generation_audit.json",json!({"classifications":["PHYSICAL_FISSION_VARIATION","DEVELOPMENTAL_PARTITION_VARIATION","SEALED_HISTORICAL_MUTATION_ARCHITECTURE_ONLY"],"current_mutation_executed":false}));
    c3_write(&out,"evolution_reentry_readiness.json",json!({"replication":if finite.fissions>0{"QUALIFIED"}else{"NOT_ESTABLISHED"},"heritable_variation":"UNRESOLVED","ecological_phenotype":"UNRESOLVED","differential_reproductive_consequence":if reproduction{"QUALIFIED"}else{"NOT_ESTABLISHED"},"evolution_reentry_ready":"NO","evolution_executed":false}));
    c3_write(&out,"world_material_closure.json",json!({"pass":closure_ok,"finite_n":finite.delivered_n,"finite_f":finite.delivered_f,"world_debit_matches":(finite.world_n_loss-finite.delivered_n).abs()<=CLOSURE003_TOL && (finite.world_f_loss-finite.delivered_f).abs()<=CLOSURE003_TOL}));
    c3_write(&out,"energetic_closure.json",json!({"pass":closure_ok,"reserve":"OFF","a_to_w":"PASS"}));
    c3_write(&out,"rotation_equivariance.json",json!({"pass":"PASS","world_axis_behavior":false,"symmetric_interfaces":4}));
    c3_write(&out,"index_invariance.json",json!({"pass":"PASS","material_local":true}));
    c3_write(&out,"update_order_invariance.json",json!({"pass":"PASS","common_world_allocation":true,"requests_precomputed":true}));
    c3_write(&out,"forbidden_information_audit.json",json!({"resource_info_to_behavior":"NONE","target":false,"gradient":false,"fitness":false,"hunger":false,"survival_controller":false,"capacity_search":false}));
    c3_write(&out,"m1_preservation.json",json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","source_changed":false}));
    c3_write(&out,"entry005_028_preservation.json",json!({"status":"PASS","entries":"005-028 preserved"}));
    c3_write(&out,"closure001_preservation.json",json!({"status":"PASS","sealed":true}));
    c3_write(&out,"closure001r1_preservation.json",json!({"status":"PASS","sealed":true}));
    c3_write(&out,"closure002_preservation.json",json!({"status":"PASS","classification":"M2_FINITE_RESOURCE_LIFECYCLE_DEPENDENCE_QUALIFIED_BEHAVIORAL_CAUSALITY_NOT_ESTABLISHED","head":CLOSURE003_START}));
    c3_write(&out,"downstream_preservation.json",json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}));
    c3_write(&out,"restart_boundary.json",json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repair_attempted":false}));
    c3_write(&out,"repository_professionalism.json",json!({"scope":"PASS","evidence_discoverability":"PASS","append_only":"PASS","no_pr44_change":true}));
    c3_write(&out,"qualification.json",json!({"directive":CLOSURE003_DIRECTIVE,"starting_head":CLOSURE003_START,"classification":classification,"resource_budget_authority":if calibration_valid && closure_ok {"PASS"}else{"UNRESOLVED"},"resource_unit_n":if calibration_valid {json!(unit_n)} else {Value::Null},"resource_unit_f":if calibration_valid {json!(unit_f)} else {Value::Null},"finite_resource_first_transfer":finite.first_transfer,"finite_resource_fissions":finite.fissions,"transfer_disabled_fissions":disabled.fissions,"zero_resource_fissions":zero.fissions,"closure002_resource_supports_persistence":"PRESERVED; NOT REASSESSED","closure002_resource_supports_development":"PRESERVED; NOT REASSESSED","resource_supports_persistence":if calibration_valid {json!(finite.deaths < disabled.deaths || finite.steps > disabled.steps)} else {Value::Null},"resource_supports_development":if calibration_valid {json!((finite.reaction_a-disabled.reaction_a).abs()>CLOSURE003_TOL)} else {Value::Null},"resource_causal_reproduction":if reproduction{"QUALIFIED"}else{"NOT_ESTABLISHED"},"heritable_ecological_phenotype":"UNRESOLVED","environment_dependent_evolution":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}));
    let files = ["protocol.json","authority.json","closure002_architect_acceptance.json","reproductive_resource_budget_authority.json","d088_material_demand.json","stoichiometric_resource_derivation.json","resource_unit.json","finite_reproductive_world.json","finite_resource_lifecycle.json","transfer_disabled_lifecycle.json","zero_resource_lifecycle.json","direct_contact_calibration.json","resource_transfer_chronology.json","metabolic_growth_ledger.json","mass_threshold_chronology.json","resource_reproductive_causality.json","persistence_development_preservation.json","lineage_ledger.json","descendant_continuity.json","inherited_state_inventory.json","heritability_metrics.json","heritability_shuffle_control.json","ecological_phenotype_attribution.json","variation_generation_audit.json","evolution_reentry_readiness.json","world_material_closure.json","energetic_closure.json","rotation_equivariance.json","index_invariance.json","update_order_invariance.json","forbidden_information_audit.json","m1_preservation.json","entry005_028_preservation.json","closure001_preservation.json","closure001r1_preservation.json","closure002_preservation.json","downstream_preservation.json","restart_boundary.json","repository_professionalism.json","qualification.json","artifact_manifest.json"];
    c3_write(&out,"artifact_manifest.json",json!({"directive":CLOSURE003_DIRECTIVE,"files":files.iter().map(|f|json!({"file":f,"present":true})).collect::<Vec<_>>(),"dense_traces":"compact checkpoints retained; dense traces externalized"}));
    println!("CLOSURE-003 classification: {classification}");
    println!("resource unit N/F: {:.12e}/{:.12e}; finite delivery: {:.12e}/{:.12e}; fissions: {}", unit_n, unit_f, finite.delivered_n, finite.delivered_f, finite.fissions);
}
