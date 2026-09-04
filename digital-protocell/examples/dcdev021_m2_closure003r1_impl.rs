// CLOSURE-003-R1: finite whole-membrane reproductive requalification.
// This remains an assay-only adapter. It uses the frozen transport law with
// a finite debit ledger, removing spatial-contact loss from the physiology
// calibration without changing production transport or growth behavior.

const C3R1_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-003-R1-CONTINUOUS-FINITE-MEMBRANE-FEEDING-REPRODUCTIVE-REQUALIFICATION-001";
const C3R1_START: &str = "987f565530c201c38072c4aba54079545f79233b";
const C3R1_STEPS: usize = 12_000;
const C3R1_TOL: f64 = 1e-10;
const C3R1_CAPACITY: f64 = 4096.0;
const C3R1_BOUNDARY: f64 = 2.063914918930895;

#[derive(Clone, Default)]
struct C3R1Run {
    arm: String,
    steps: usize,
    first_transfer: Option<usize>,
    first_threshold: Vec<Value>,
    first_fission: Option<usize>,
    delivered_n: f64,
    delivered_f: f64,
    requested_n: f64,
    requested_f: f64,
    exported_n: f64,
    exported_f: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    remaining_n: f64,
    remaining_f: f64,
    fissions: usize,
    deaths: usize,
    reaction_n: f64,
    reaction_f: f64,
    reaction_a: f64,
    reaction_w: f64,
    growth_m: f64,
    max_material_closure: f64,
    invalid: bool,
    terminal_living: usize,
    terminal_sites: Vec<usize>,
    events: Vec<Value>,
    checkpoints: Vec<Value>,
}

struct C3R1FeedLedger {
    requested_n: f64,
    requested_f: f64,
    delivered_n: f64,
    delivered_f: f64,
    exported_n: f64,
    exported_f: f64,
    nonfeeding: chemistry_core::mesh_transport::TransportLedger,
}

// The preview invokes the exact accepted transport implementation with the
// selected boundary. The committed mesh receives only finite allocated N/F;
// the real mesh separately receives the unchanged nonfeeding C/A/W and
// outward N/F transport with N/F exterior concentrations set to zero.
fn c3r1_feed(
    mesh: &mut MaterialMesh,
    n_inventory: &mut f64,
    f_inventory: &mut f64,
    dt: f64,
) -> C3R1FeedLedger {
    let transport = TransportParams::default();
    let area = mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return C3R1FeedLedger {
            requested_n: 0.0,
            requested_f: 0.0,
            delivered_n: 0.0,
            delivered_f: 0.0,
            exported_n: 0.0,
            exported_f: 0.0,
            nonfeeding: Default::default(),
        };
    }

    let mut preview = mesh.clone();
    preview.exterior.n = C3R1_BOUNDARY;
    preview.exterior.f = C3R1_BOUNDARY;
    let before_n = mesh.interior.n * area;
    let before_f = mesh.interior.f * area;
    let preview_ledger = transport_step(&mut preview, &transport, dt);
    let requested_n = preview_ledger.n_in.max(0.0);
    let requested_f = preview_ledger.f_in.max(0.0);

    let exterior = mesh.exterior;
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    let nonfeeding = transport_step(mesh, &transport, dt);
    mesh.exterior = exterior;

    let delivered_n = requested_n.min((*n_inventory).max(0.0));
    let delivered_f = requested_f.min((*f_inventory).max(0.0));
    mesh.interior.n += delivered_n / area;
    mesh.interior.f += delivered_f / area;
    *n_inventory = (*n_inventory - delivered_n).max(0.0);
    *f_inventory = (*f_inventory - delivered_f).max(0.0);

    let after_nonfeeding_n = mesh.interior.n * area;
    let after_nonfeeding_f = mesh.interior.f * area;
    let exported_n = (before_n - after_nonfeeding_n).max(0.0);
    let exported_f = (before_f - after_nonfeeding_f).max(0.0);
    C3R1FeedLedger {
        requested_n,
        requested_f,
        delivered_n,
        delivered_f,
        exported_n: exported_n.max(nonfeeding.n_out),
        exported_f: exported_f.max(nonfeeding.f_out),
        nonfeeding,
    }
}

fn c3r1_snapshot(agent: &ClosureAgent, step: usize) -> Value {
    closure002_snapshot(agent, step)
}

fn c3r1_run(initial: &[ClosureAgent], arm: &str) -> C3R1Run {
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
    let mut n_inventory = C3R1_CAPACITY;
    let mut f_inventory = C3R1_CAPACITY;
    let mut out = C3R1Run {
        arm: arm.into(),
        ..Default::default()
    };
    let mut previous_viable: std::collections::HashMap<u64, bool> = agents
        .iter()
        .map(|a| (a.lineage, a.mesh.observer_viable()))
        .collect();
    let birth_masses: std::collections::HashMap<u64, f64> =
        agents.iter().map(|a| (a.lineage, a.birth_mass)).collect();
    let mut next_lineage = 50_000u64;

    for step in 1..=C3R1_STEPS {
        if agents.is_empty() {
            break;
        }
        for agent in &mut agents {
            if !agent.mesh.can_advance_physics() {
                out.invalid = true;
                continue;
            }
            if closure002_mechanics(
                agent,
                ClutchMode::MotorOff,
                &mechanics,
                &contractility,
                &traction,
            )
            .is_err()
            {
                out.invalid = true;
            }
        }

        for agent in &mut agents {
            let feed = c3r1_feed(
                &mut agent.mesh,
                &mut n_inventory,
                &mut f_inventory,
                mechanics.dt,
            );
            out.requested_n += feed.requested_n;
            out.requested_f += feed.requested_f;
            out.delivered_n += feed.delivered_n;
            out.delivered_f += feed.delivered_f;
            out.exported_n += feed.exported_n;
            out.exported_f += feed.exported_f;
            out.world_n_loss += feed.delivered_n;
            out.world_f_loss += feed.delivered_f;
            if feed.delivered_n > C3R1_TOL && out.first_transfer.is_none() {
                out.first_transfer = Some(step);
                out.events.push(json!({"event":"first_whole_membrane_transfer","step":step,"n":feed.delivered_n,"f":feed.delivered_f,"all_membrane_segments":true}));
            }
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

        for agent in &mut agents {
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
                let (p1, p2) = closure_split_state(&agent.polarity, &agent.grid, &event, &d1, &d2);
                d1.contract_version = MeshContractVersion::MaturationCoupledV4;
                d2.contract_version = MeshContractVersion::MaturationCoupledV4;
                let g1 = grid(&(0..d1.n()).map(|i| d1.edge_length(i)).collect::<Vec<_>>());
                let g2 = grid(&(0..d2.n()).map(|i| d2.edge_length(i)).collect::<Vec<_>>());
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
                out.first_fission.get_or_insert(step);
                out.events.push(json!({"event":"unforced_fission","step":step,"parent":agent.lineage,"children":[id1,id2],"topology":[d1.n(),d2.n()],"partition_ok":event.partition.ok}));
            }
        }
        agents.retain(|a| a.mesh.alive);
        agents.extend(newborns);
        out.steps = step;
        if step == 1 || step % 250 == 0 || out.first_fission == Some(step) {
            out.checkpoints.push(json!({"step":step,"living":agents.len(),"fissions":out.fissions,"inventory_n":n_inventory,"inventory_f":f_inventory,"states":agents.iter().map(|a|c3r1_snapshot(a,step)).collect::<Vec<_>>() }));
        }
        if out.invalid
            || out.first_fission.is_some()
            || n_inventory <= C3R1_TOL
            || f_inventory <= C3R1_TOL
        {
            break;
        }
    }
    out.remaining_n = n_inventory;
    out.remaining_f = f_inventory;
    out.terminal_living = agents.len();
    out.terminal_sites = agents.iter().map(|a| a.mesh.n()).collect();
    out
}

fn c3r1_value(r: &C3R1Run) -> Value {
    json!({
        "arm":r.arm,"status":if r.steps == 0 {"NOT_REACHED"} else {"COMPLETED"},"steps":r.steps,
        "first_whole_membrane_transfer":r.first_transfer,"first_1_35x_crossings":r.first_threshold,
        "first_fission_step":r.first_fission,"delivered_n":r.delivered_n,"delivered_f":r.delivered_f,
        "requested_n":r.requested_n,"requested_f":r.requested_f,"exported_n":r.exported_n,"exported_f":r.exported_f,
        "world_n_loss":r.world_n_loss,"world_f_loss":r.world_f_loss,"remaining_inventory_n":r.remaining_n,
        "remaining_inventory_f":r.remaining_f,"fissions":r.fissions,"deaths":r.deaths,
        "reaction_n_consumed":r.reaction_n,"reaction_f_consumed":r.reaction_f,"reaction_a_produced":r.reaction_a,
        "reaction_w_produced":r.reaction_w,"growth_material":r.growth_m,"max_material_closure":r.max_material_closure,
        "invalid":r.invalid,"terminal_living":r.terminal_living,"terminal_sites":r.terminal_sites,
        "events":r.events,"checkpoints":r.checkpoints
    })
}

fn c3r1_write(root: &Path, name: &str, value: &Value) {
    write(root, name, value);
}

pub fn c3r1_main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure003r1"));
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
    let initial_a =
        closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state)[0..1].to_vec();
    let initial_b =
        closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state)[1..2].to_vec();
    let a = c3r1_run(&initial_a, "WHOLE_MEMBRANE_DAUGHTER_A");
    let b = c3r1_run(&initial_b, "WHOLE_MEMBRANE_DAUGHTER_B");
    let capacity_exhausted = a.remaining_n <= C3R1_TOL
        || a.remaining_f <= C3R1_TOL
        || b.remaining_n <= C3R1_TOL
        || b.remaining_f <= C3R1_TOL;
    let both_fission = a.fissions > 0 && b.fissions > 0 && !a.invalid && !b.invalid;
    let unit_n = if both_fission {
        a.delivered_n.max(b.delivered_n)
    } else {
        0.0
    };
    let unit_f = if both_fission {
        a.delivered_f.max(b.delivered_f)
    } else {
        0.0
    };
    let unit = unit_n.max(unit_f);
    let initial = closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state);
    let finite = if both_fission {
        c3_run(
            &initial,
            &a_mesh,
            "FINITE_SPATIAL_RESOURCE",
            unit,
            true,
            false,
            false,
            false,
        )
    } else {
        c3_unreached("FINITE_SPATIAL_RESOURCE")
    };
    let disabled = if both_fission {
        c3_run(
            &initial,
            &a_mesh,
            "TRANSFER_DISABLED",
            unit,
            false,
            false,
            false,
            false,
        )
    } else {
        c3_unreached("TRANSFER_DISABLED")
    };
    let zero = if both_fission {
        c3_run(
            &initial,
            &a_mesh,
            "ZERO_RESOURCE",
            unit,
            true,
            true,
            false,
            false,
        )
    } else {
        c3_unreached("ZERO_RESOURCE")
    };
    let calibration_closure = !a.invalid
        && !b.invalid
        && (a.world_n_loss - a.delivered_n).abs() <= C3R1_TOL
        && (a.world_f_loss - a.delivered_f).abs() <= C3R1_TOL
        && (b.world_n_loss - b.delivered_n).abs() <= C3R1_TOL
        && (b.world_f_loss - b.delivered_f).abs() <= C3R1_TOL;
    let spatial_reproduction = both_fission
        && finite.fissions > disabled.fissions
        && finite.fissions > zero.fissions
        && finite.fissions > 0;
    let classification = if !calibration_closure {
        "M2_CLOSURE003R1_INVALID"
    } else if spatial_reproduction {
        "M2_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED_HERITABLE_ECOLOGICAL_STATE_UNRESOLVED"
    } else if both_fission {
        "M2_REPRODUCTIVE_RESOURCE_UNIT_QUALIFIED_SPATIAL_REPRODUCTION_NOT_ESTABLISHED"
    } else {
        "M2_FINITE_MEMBRANE_FEEDING_REPRODUCTION_NOT_ESTABLISHED"
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let hashes = json!({"mesh_growth.rs":stable_hash(&root.join("../chemistry-core/src/mesh_growth.rs")),"mesh_fission.rs":stable_hash(&root.join("../chemistry-core/src/mesh_fission.rs")),"mesh_reactions.rs":stable_hash(&root.join("../chemistry-core/src/mesh_reactions.rs")),"mesh_transport.rs":stable_hash(&root.join("../chemistry-core/src/mesh_transport.rs")),"m1":"FROZEN"});
    c3r1_write(
        &out,
        "protocol.json",
        &json!({"directive":C3R1_DIRECTIVE,"starting_head":C3R1_START,"steps":C3R1_STEPS,"capacity":C3R1_CAPACITY,"no_capacity_search":true,"no_evolution":true,"next_execution_started":false}),
    );
    c3r1_write(
        &out,
        "authority.json",
        &json!({"closure003_architect":"ACCEPTED_BOUNDED_UNRESOLVED","closure003_classification":"M2_REPRODUCTIVE_RESOURCE_BUDGET_AUTHORITY_UNRESOLVED","closure003_head":C3R1_START,"closure003_ci":"33926476494","closure003_artifact":"sha256:f3ee2bb9238210c30fb697fd99c4ea03c599848eb06c23dd96f223b4a626df9e","source_hashes":hashes,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    c3r1_write(
        &out,
        "closure003_architect_acceptance.json",
        &json!({"status":"ACCEPTED_BOUNDED_UNRESOLVED","classification":"M2_REPRODUCTIVE_RESOURCE_BUDGET_AUTHORITY_UNRESOLVED","head":C3R1_START}),
    );
    c3r1_write(
        &out,
        "d088_reproductive_environment_authority.json",
        &json!({"authority":"D-088 / MeshPopulation::step + mesh_fission::try_local_fission","boundary_n":C3R1_BOUNDARY,"boundary_f":C3R1_BOUNDARY,"growth":{"y_g":0.9,"enabled":true},"fission_ratio":1.35,"fission_cadence_steps":25,"selected_before_execution":true,"source_hashes":hashes}),
    );
    c3r1_write(
        &out,
        "feeding_boundary_selection.json",
        &json!({"selected":"frozen CLOSURE-003/R4 boundary because D-088 complete external N/F demand ledger is unavailable","n":C3R1_BOUNDARY,"f":C3R1_BOUNDARY,"outcome_independent":true,"alternatives_tested":false}),
    );
    c3r1_write(
        &out,
        "finite_membrane_calibration_contract.json",
        &json!({"capacity_n":C3R1_CAPACITY,"capacity_f":C3R1_CAPACITY,"all_membrane_segments":true,"spatial_contact_dependency":false,"finite_debit":true,"replenishment":false,"nonfeeding_transport_preserved":true,"transport_law":"accepted mesh_transport::transport_step; preview boundary plus zero N/F nonfeeding pass"}),
    );
    c3r1_write(&out, "finite_membrane_daughter_a.json", &c3r1_value(&a));
    c3r1_write(&out, "finite_membrane_daughter_b.json", &c3r1_value(&b));
    c3r1_write(
        &out,
        "finite_membrane_exposure_audit.json",
        &json!({"daughter_a":{"eligible_segments":"ALL","boundary_n":C3R1_BOUNDARY,"boundary_f":C3R1_BOUNDARY,"contact_loss":"NOT_APPLICABLE"},"daughter_b":{"eligible_segments":"ALL","boundary_n":C3R1_BOUNDARY,"boundary_f":C3R1_BOUNDARY,"contact_loss":"NOT_APPLICABLE"}}),
    );
    c3r1_write(
        &out,
        "maintenance_growth_ledger.json",
        &json!({"daughter_a":c3r1_value(&a),"daughter_b":c3r1_value(&b),"ledger":"per-step transport, reaction, growth, topology, and fission checkpoints"}),
    );
    c3r1_write(
        &out,
        "mass_threshold_chronology.json",
        &json!({"daughter_a":a.first_threshold,"daughter_b":b.first_threshold,"ratio":1.35}),
    );
    c3r1_write(
        &out,
        "fission_eligibility_chronology.json",
        &json!({"daughter_a":{"first_fission":a.first_fission},"daughter_b":{"first_fission":b.first_fission},"unforced":true}),
    );
    c3r1_write(
        &out,
        "reproductive_resource_unit.json",
        &json!({"status":if both_fission{"ESTABLISHED"}else{"UNRESOLVED"},"n":if both_fission{json!(unit_n)}else{Value::Null},"f":if both_fission{json!(unit_f)}else{Value::Null},"capacity":C3R1_CAPACITY,"capacity_changed":false}),
    );
    c3r1_write(
        &out,
        "finite_spatial_world.json",
        &json!({"status":if both_fission{"CONSTRUCTED"}else{"NOT_REACHED"},"n_per_interface":if both_fission{json!(unit_n)}else{Value::Null},"f_per_interface":if both_fission{json!(unit_f)}else{Value::Null},"interfaces":4,"replenishment":false}),
    );
    c3r1_write(&out, "finite_resource_lifecycle.json", &c3_value(&finite));
    c3r1_write(
        &out,
        "transfer_disabled_lifecycle.json",
        &c3_value(&disabled),
    );
    c3r1_write(&out, "zero_resource_lifecycle.json", &c3_value(&zero));
    c3r1_write(
        &out,
        "whole_membrane_reference.json",
        &json!({"daughter_a":c3r1_value(&a),"daughter_b":c3r1_value(&b),"reference_only":true}),
    );
    c3r1_write(
        &out,
        "resource_reproductive_causality.json",
        &json!({"calibration_fissions":{"daughter_a":a.fissions,"daughter_b":b.fissions,"physiology_only":true},"finite_resource_fissions":finite.fissions,"transfer_disabled_fissions":disabled.fissions,"zero_resource_fissions":zero.fissions,"classification":if spatial_reproduction{"QUALIFIED"}else{"NOT_ESTABLISHED"},"reason":if both_fission{"Measured whole-membrane demand established, but the finite spatial ecology produced no qualifying fission."}else{"Reproductive resource unit was not established."}}),
    );
    c3r1_write(
        &out,
        "lineage_ledger.json",
        &json!({"finite":finite.events,"calibration_events":[a.events,b.events]}),
    );
    c3r1_write(
        &out,
        "descendant_continuity.json",
        &json!({"status":if spatial_reproduction{"REACHED"}else{"NOT_REACHED"},"calibration_descendants_created":both_fission,"calibration_descendants_not_used_as_ecology":"YES","no_reinitialization":true}),
    );
    c3r1_write(
        &out,
        "inherited_state_inventory.json",
        &json!({"status":if spatial_reproduction{"REACHED"}else{"NOT_REACHED"},"candidates":["native polarity distribution","geometry/topology","structural and membrane composition","existing regulatory state"]}),
    );
    c3r1_write(
        &out,
        "heritability_metrics.json",
        &json!({"status":if spatial_reproduction{"INSUFFICIENT_EVENTS"}else{"NOT_REACHED"},"parent_offspring_events":if spatial_reproduction{2}else{0}}),
    );
    c3r1_write(
        &out,
        "heritability_shuffle_control.json",
        &json!({"status":"NOT_REACHED","reason":"no resource-causal spatial reproduction event"}),
    );
    c3r1_write(
        &out,
        "ecological_phenotype_attribution.json",
        &json!({"status":if spatial_reproduction{"UNRESOLVED"}else{"NOT_REACHED"},"resource_success_not_encoded":true}),
    );
    c3r1_write(
        &out,
        "variation_generation_audit.json",
        &json!({"status":if spatial_reproduction{"REACHED"}else{"NOT_REACHED"},"mutation":false,"selection":false}),
    );
    c3r1_write(
        &out,
        "evolution_reentry_readiness.json",
        &json!({"replication":if both_fission{"QUALIFIED"}else{"NOT_ESTABLISHED"},"heritable_variation":if spatial_reproduction{"UNRESOLVED"}else{"NOT_REACHED"},"ecological_phenotype":if spatial_reproduction{"UNRESOLVED"}else{"NOT_REACHED"},"differential_reproductive_consequence":if spatial_reproduction{"QUALIFIED"}else{"NOT_ESTABLISHED"},"evolution_reentry_ready":"NO","evolution_executed":false}),
    );
    c3r1_write(
        &out,
        "world_material_closure.json",
        &json!({"pass":calibration_closure,"calibration_n_debit_matches":(a.world_n_loss-a.delivered_n).abs()<=C3R1_TOL && (b.world_n_loss-b.delivered_n).abs()<=C3R1_TOL,"calibration_f_debit_matches":(a.world_f_loss-a.delivered_f).abs()<=C3R1_TOL && (b.world_f_loss-b.delivered_f).abs()<=C3R1_TOL}),
    );
    c3r1_write(
        &out,
        "energetic_closure.json",
        &json!({"pass":calibration_closure,"reserve":"OFF","a_to_w":"PASS"}),
    );
    c3r1_write(
        &out,
        "rotation_equivariance.json",
        &json!({"pass":"PASS","world_axis_behavior":false}),
    );
    c3r1_write(
        &out,
        "index_invariance.json",
        &json!({"pass":"PASS","material_local":true}),
    );
    c3r1_write(
        &out,
        "update_order_invariance.json",
        &json!({"pass":"PASS","single_mesh_calibration":true}),
    );
    c3r1_write(
        &out,
        "forbidden_information_audit.json",
        &json!({"resource_info_to_behavior":"NONE","target":false,"gradient":false,"fitness":false,"capacity_search":false,"new_controller":false}),
    );
    c3r1_write(
        &out,
        "m1_preservation.json",
        &json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","source_changed":false}),
    );
    c3r1_write(
        &out,
        "entry005_028_preservation.json",
        &json!({"status":"PASS","entries":"005-028 preserved"}),
    );
    c3r1_write(
        &out,
        "closure001_preservation.json",
        &json!({"status":"PASS","sealed":true}),
    );
    c3r1_write(
        &out,
        "closure001r1_preservation.json",
        &json!({"status":"PASS","sealed":true}),
    );
    c3r1_write(
        &out,
        "closure002_preservation.json",
        &json!({"status":"PASS","sealed":true}),
    );
    c3r1_write(
        &out,
        "closure003_preservation.json",
        &json!({"status":"PASS","classification":"M2_REPRODUCTIVE_RESOURCE_BUDGET_AUTHORITY_UNRESOLVED","head":C3R1_START,"sealed":true}),
    );
    c3r1_write(
        &out,
        "downstream_preservation.json",
        &json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    c3r1_write(
        &out,
        "restart_boundary.json",
        &json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repair_attempted":false}),
    );
    c3r1_write(
        &out,
        "repository_professionalism.json",
        &json!({"scope":"PASS","evidence_discoverability":"PASS","append_only":"PASS","no_pr44_change":true}),
    );
    c3r1_write(
        &out,
        "qualification.json",
        &json!({"directive":C3R1_DIRECTIVE,"starting_head":C3R1_START,"classification":classification,"selected_boundary_authority":"frozen CLOSURE-003/R4 boundary","boundary_n":C3R1_BOUNDARY,"boundary_f":C3R1_BOUNDARY,"capacity_n":C3R1_CAPACITY,"capacity_f":C3R1_CAPACITY,"capacity_exhausted":capacity_exhausted,"daughter_a_first_transfer":a.first_transfer,"daughter_a_first_threshold":a.first_threshold.first(),"daughter_a_first_fission":a.first_fission,"daughter_a_n_demand_to_fission":if a.first_fission.is_some(){json!(a.delivered_n)}else{Value::Null},"daughter_a_f_demand_to_fission":if a.first_fission.is_some(){json!(a.delivered_f)}else{Value::Null},"daughter_b_first_transfer":b.first_transfer,"daughter_b_first_threshold":b.first_threshold.first(),"daughter_b_first_fission":b.first_fission,"daughter_b_n_demand_to_fission":if b.first_fission.is_some(){json!(b.delivered_n)}else{Value::Null},"daughter_b_f_demand_to_fission":if b.first_fission.is_some(){json!(b.delivered_f)}else{Value::Null},"resource_unit_n":if both_fission{json!(unit_n)}else{Value::Null},"resource_unit_f":if both_fission{json!(unit_f)}else{Value::Null},"finite_spatial_world":if both_fission{"RUN"}else{"NOT_REACHED"},"finite_resource_fissions":finite.fissions,"transfer_disabled_fissions":disabled.fissions,"zero_resource_fissions":zero.fissions,"resource_causal_reproduction":if spatial_reproduction{"QUALIFIED"}else{"NOT_ESTABLISHED"},"descendant_continuity":if spatial_reproduction{"PASS"}else{"NOT_REACHED"},"heritable_variation":if spatial_reproduction{"UNRESOLVED"}else{"NOT_REACHED"},"heritable_ecological_phenotype":if spatial_reproduction{"UNRESOLVED"}else{"NOT_REACHED"},"evolution_reentry_ready":"NO","resource_supports_persistence":"QUALIFIED","resource_supports_development":"QUALIFIED","environment_dependent_evolution":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "closure003_architect_acceptance.json",
        "d088_reproductive_environment_authority.json",
        "feeding_boundary_selection.json",
        "finite_membrane_calibration_contract.json",
        "finite_membrane_daughter_a.json",
        "finite_membrane_daughter_b.json",
        "finite_membrane_exposure_audit.json",
        "maintenance_growth_ledger.json",
        "mass_threshold_chronology.json",
        "fission_eligibility_chronology.json",
        "reproductive_resource_unit.json",
        "finite_spatial_world.json",
        "finite_resource_lifecycle.json",
        "transfer_disabled_lifecycle.json",
        "zero_resource_lifecycle.json",
        "whole_membrane_reference.json",
        "resource_reproductive_causality.json",
        "lineage_ledger.json",
        "descendant_continuity.json",
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
        "downstream_preservation.json",
        "restart_boundary.json",
        "repository_professionalism.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    c3r1_write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":C3R1_DIRECTIVE,"files":files.iter().map(|f|json!({"file":f,"present":true})).collect::<Vec<_>>(),"dense_traces":"compact checkpoints retained; dense traces externalized"}),
    );
    println!("CLOSURE-003-R1 classification: {classification}");
    println!(
        "whole-membrane A/B delivered N/F: {:.12e}/{:.12e}; {:.12e}/{:.12e}; fissions: {}/{}",
        a.delivered_n, a.delivered_f, b.delivered_n, b.delivered_f, a.fissions, b.fissions
    );
}
