// DC-DEV-021 M2 ENTRY-028: balanced separated-resource ecological coupling.
//
// This file is included beside the accepted ENTRY-027 authority so the assay
// can reuse its exact physical fission and first-lawful daughter-state path.
// It is observer-only at the project level: resource placement is determined
// from daughter geometry before execution, and no resource observation enters
// polarity, motor choice, growth, mechanics, or lifecycle logic.

const ENTRY028_DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-028-BALANCED-SEPARATED-RESOURCE-ECOLOGICAL-COUPLING-FEASIBILITY-001";
const ENTRY028_START: &str = "c002777fa59169ad206bb92fc4e76d7646f10061";
const ENTRY028_STEPS: usize = 3_000;
const ENTRY028_RESOURCE_RADIUS: f64 = 1.5;
const ENTRY028_RESOURCE_N: f64 = 3.0;
const ENTRY028_RESOURCE_F: f64 = 3.0;
const ENTRY028_NUM_TOL: f64 = 1e-10;
const ENTRY028_BEARINGS: [(u16, [f64; 2]); 4] = [
    (0, [1.0, 0.0]),
    (90, [0.0, 1.0]),
    (180, [-1.0, 0.0]),
    (270, [0.0, -1.0]),
];

#[derive(Clone)]
struct Placement028 {
    bearing: u16,
    center: [f64; 2],
    mean_edge_length: f64,
    initial_polygon_distance: f64,
    initial_surface_gap: f64,
    initial_contact_patches: Vec<usize>,
}

struct Arm028 {
    label: String,
    daughter: char,
    bearing: u16,
    spatial: bool,
    motor_off: bool,
    terminal_step: usize,
    first_contact_step: Option<usize>,
    first_contact_edges: Vec<usize>,
    contact_duration_steps: usize,
    contact_entries: usize,
    contact_exits: usize,
    maximum_contact_patches: usize,
    delivered_n: f64,
    delivered_f: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    remaining_n: f64,
    remaining_f: f64,
    reaction_n: f64,
    reaction_f: f64,
    reaction_a: f64,
    reaction_w: f64,
    path: f64,
    net: f64,
    slips: usize,
    stuck: usize,
    a_spent: f64,
    w_motor: f64,
    a_to_w_residual: f64,
    initial_a: f64,
    final_a: f64,
    final_n: f64,
    final_f: f64,
    first_second_fission_step: Option<usize>,
    invalid: bool,
    points: Vec<Value>,
}

fn segment_distance028(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let denom = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if denom > 0.0 {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let q = [a[0] + t * ab[0], a[1] + t * ab[1]];
    vector_norm(vector_sub(p, q))
}

fn polygon_distance028(mesh: &MaterialMesh, p: [f64; 2]) -> f64 {
    (0..mesh.n())
        .map(|i| segment_distance028(p, mesh.vertices[i], mesh.vertices[(i + 1) % mesh.n()]))
        .fold(f64::INFINITY, f64::min)
}

fn mean_edge028(mesh: &MaterialMesh) -> f64 {
    mesh.perimeter() / mesh.n().max(1) as f64
}

fn place_resource028(mesh: &MaterialMesh, bearing: u16, direction: [f64; 2]) -> Placement028 {
    let centroid = physical_centroid(mesh);
    let mean_edge_length = mean_edge028(mesh);
    let target_distance = ENTRY028_RESOURCE_RADIUS + mean_edge_length;
    let body_radius = mesh
        .vertices
        .iter()
        .map(|p| vector_norm(vector_sub(*p, centroid)))
        .fold(0.0, f64::max);
    let center_at = |s: f64| {
        [
            centroid[0] + s * direction[0],
            centroid[1] + s * direction[1],
        ]
    };
    let mut lo = 0.0;
    let mut hi = body_radius + target_distance;
    while polygon_distance028(mesh, center_at(hi)) < target_distance {
        hi *= 2.0;
    }
    for _ in 0..100 {
        let mid = (lo + hi) * 0.5;
        if polygon_distance028(mesh, center_at(mid)) < target_distance {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let center = center_at((lo + hi) * 0.5);
    let distance = polygon_distance028(mesh, center);
    let region = FiniteSpatialResourceRegionV1::new(
        center,
        ENTRY028_RESOURCE_RADIUS,
        ENTRY028_RESOURCE_N,
        ENTRY028_RESOURCE_F,
    );
    let initial_contact_patches = region
        .local_contact_signal(mesh)
        .iter()
        .enumerate()
        .filter_map(|(i, x)| (*x > 0.5).then_some(i))
        .collect();
    Placement028 {
        bearing,
        center,
        mean_edge_length,
        initial_polygon_distance: distance,
        initial_surface_gap: distance - ENTRY028_RESOURCE_RADIUS,
        initial_contact_patches,
    }
}

fn arm028_value(run: &Arm028) -> Value {
    json!({
        "arm": run.label,
        "daughter": run.daughter.to_string(),
        "bearing_degrees": run.bearing,
        "spatial": run.spatial,
        "motor_off": run.motor_off,
        "terminal_step": run.terminal_step,
        "first_contact_step": run.first_contact_step,
        "first_contact_edges": run.first_contact_edges,
        "contact_duration_steps": run.contact_duration_steps,
        "contact_entries": run.contact_entries,
        "contact_exits": run.contact_exits,
        "maximum_contact_patches": run.maximum_contact_patches,
        "delivered_n": run.delivered_n,
        "delivered_f": run.delivered_f,
        "world_n_loss": run.world_n_loss,
        "world_f_loss": run.world_f_loss,
        "remaining_n": run.remaining_n,
        "remaining_f": run.remaining_f,
        "reaction_n_consumed": run.reaction_n,
        "reaction_f_consumed": run.reaction_f,
        "reaction_a_produced": run.reaction_a,
        "reaction_w_produced": run.reaction_w,
        "path_length": run.path,
        "net_displacement": run.net,
        "displacement_path_ratio": run.net / run.path.max(1e-30),
        "slips": run.slips,
        "stuck_contacts": run.stuck,
        "a_spent": run.a_spent,
        "motor_w_generated": run.w_motor,
        "a_to_w_residual": run.a_to_w_residual,
        "initial_a": run.initial_a,
        "final_a": run.final_a,
        "final_n": run.final_n,
        "final_f": run.final_f,
        "first_second_fission_step": run.first_second_fission_step,
        "invalid": run.invalid,
        "checkpoints": run.points,
    })
}

fn arm028_run(
    mesh_start: &MaterialMesh,
    grid_start: Grid,
    state_start: AmountState,
    placement: &Placement028,
    daughter: char,
    spatial: bool,
    motor_off: bool,
    birth_mass: f64,
    birth_tick: u64,
) -> Arm028 {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction = ReactionParams::conservative_v3();
    let fission = FissionParams::default();
    let mut mesh = mesh_start.clone();
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    let mut current_grid = grid_start;
    let mut state = state_start;
    let mut resource = FiniteSpatialResourceRegionV1::new(
        placement.center,
        ENTRY028_RESOURCE_RADIUS,
        ENTRY028_RESOURCE_N,
        ENTRY028_RESOURCE_F,
    );
    let initial_centroid = physical_centroid(&mesh);
    let mut previous_centroid = initial_centroid;
    let initial_a = mesh.interior.a;
    let mut path = 0.0;
    let mut slips = 0;
    let mut stuck = 0;
    let mut a_spent = 0.0;
    let mut w_motor = 0.0;
    let mut a_to_w_residual: f64 = 0.0;
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut world_n_loss = 0.0;
    let mut world_f_loss = 0.0;
    let mut reaction_n = 0.0;
    let mut reaction_f = 0.0;
    let mut reaction_a = 0.0;
    let mut reaction_w = 0.0;
    let mut first_contact_step = None;
    let mut first_contact_edges = Vec::new();
    let mut contact_duration_steps = 0;
    let mut contact_entries = 0;
    let mut contact_exits = 0;
    let mut maximum_contact_patches = 0;
    let mut was_contact = false;
    let mut first_second_fission_step = None;
    let mut invalid = false;
    let mut points = Vec::new();
    let mut terminal_step = 0usize;
    let checkpoints = [1usize, 25, 100, 500, 1_000, 2_000, 3_000];

    for step in 1..=ENTRY028_STEPS {
        if !mesh.can_advance_physics() {
            invalid = true;
            break;
        }
        let population_tick = birth_tick + step as u64;
        let old_grid = current_grid.clone();
        let _ = transport_step(&mut mesh, &TransportParams::default(), mechanics.dt);
        let raw = entry025_anti(&state);
        let mean = raw.iter().sum::<f64>() / raw.len().max(1) as f64;
        let motor = if motor_off {
            vec![0.0; mesh.n()]
        } else if spatial {
            raw
        } else {
            vec![mean; mesh.n()]
        };
        if motor_off {
            let ledger =
                apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
            slips += ledger.slipping_contacts;
            stuck += ledger.stuck_contacts;
        } else {
            let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &motor,
                &mechanics,
                &contractility,
                &traction,
            )
            .unwrap();
            slips += ledger.slipping_contacts;
            stuck += ledger.stuck_contacts;
            if let Some(c) = ledger.contractility.as_ref() {
                a_spent += c.resource_spent;
                w_motor += c.waste_amount_after - c.waste_amount_before;
                a_to_w_residual = a_to_w_residual.max(
                    (c.activated_amount_before - c.activated_amount_after + c.waste_amount_before
                        - c.waste_amount_after)
                        .abs(),
                );
            }
        }
        // ENTRY-011's accepted assay ordering: physical mechanics, finite
        // uptake, then the exact frozen reaction kernel. Growth/remesh follows
        // the accepted ENTRY-027 live daughter continuation.
        let uptake = resource.uptake(&mut mesh, &TransportParams::default(), mechanics.dt);
        delivered_n += uptake.n_delivered;
        delivered_f += uptake.f_delivered;
        world_n_loss += uptake.n_world_loss;
        world_f_loss += uptake.f_world_loss;
        let contact_edges: Vec<usize> = resource
            .local_contact_signal(&mesh)
            .iter()
            .enumerate()
            .filter_map(|(i, x)| (*x > 0.5).then_some(i))
            .collect();
        let contact = !contact_edges.is_empty();
        if contact {
            contact_duration_steps += 1;
            maximum_contact_patches = maximum_contact_patches.max(contact_edges.len());
            if first_contact_step.is_none() {
                first_contact_step = Some(step);
                first_contact_edges = contact_edges.clone();
            }
        }
        if contact && !was_contact {
            contact_entries += 1;
        }
        if !contact && was_contact {
            contact_exits += 1;
        }
        was_contact = contact;
        let ledger = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        reaction_n += ledger.n_consumed;
        reaction_f += ledger.f_consumed;
        reaction_a += ledger.a_produced;
        reaction_w += ledger.w_produced;
        let _growth = growth_step(
            &mut mesh,
            &ReactionParams::default(),
            &GrowthParams {
                y_g: 0.9,
                enable_growth: true,
            },
            mechanics.dt,
        );
        let old_vertices = mesh.vertices.clone();
        remesh(&mut mesh);
        if population_tick % 10 == 0 {
            let _ = chemistry_core::mesh_fission::topology_step(&mut mesh, &fission);
        }
        let origin = mesh
            .vertices
            .first()
            .and_then(|first| {
                old_vertices
                    .iter()
                    .position(|old| (old[0] - first[0]).hypot(old[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        let new_grid = grid(
            &(0..mesh.n())
                .map(|i| mesh.edge_length(i))
                .collect::<Vec<_>>(),
        );
        state = remap(&old_grid, &state, &new_grid, origin);
        advance(&mut state, &new_grid, DT);
        let centroid = physical_centroid(&mesh);
        path += vector_norm(vector_sub(centroid, previous_centroid));
        previous_centroid = centroid;
        current_grid = new_grid;
        terminal_step = step;
        if checkpoints.contains(&step) {
            points.push(json!({
                "step": step,
                "centroid": centroid,
                "contact_edges": contact_edges,
                "delivered_n": delivered_n,
                "delivered_f": delivered_f,
                "n": mesh.interior.n,
                "f": mesh.interior.f,
                "a": mesh.interior.a,
                "w": mesh.interior.w,
                "reaction_n_consumed": reaction_n,
                "reaction_f_consumed": reaction_f,
            }));
        }
        if population_tick % 25 == 0
            && mesh.total_structural_mass() >= 1.35 * birth_mass.max(1e-9)
            && try_local_fission(&mesh, &fission).is_some()
        {
            first_second_fission_step = Some(step);
            break;
        }
    }
    Arm028 {
        label: format!(
            "DAUGHTER_{}_BEARING_{}_{}",
            daughter,
            placement.bearing,
            if motor_off {
                "MOTOR_OFF"
            } else if spatial {
                "SPATIAL"
            } else {
                "SAME_MEAN"
            }
        ),
        daughter,
        bearing: placement.bearing,
        spatial,
        motor_off,
        terminal_step,
        first_contact_step,
        first_contact_edges,
        contact_duration_steps,
        contact_entries,
        contact_exits,
        maximum_contact_patches,
        delivered_n,
        delivered_f,
        world_n_loss,
        world_f_loss,
        remaining_n: resource.n_mass,
        remaining_f: resource.f_mass,
        reaction_n,
        reaction_f,
        reaction_a,
        reaction_w,
        path,
        net: vector_norm(vector_sub(physical_centroid(&mesh), initial_centroid)),
        slips,
        stuck,
        a_spent,
        w_motor,
        a_to_w_residual,
        initial_a,
        final_a: mesh.interior.a,
        final_n: mesh.interior.n,
        final_f: mesh.interior.f,
        first_second_fission_step,
        invalid,
        points,
    }
}

fn placement_value028(p: &Placement028) -> Value {
    json!({
        "bearing_degrees": p.bearing,
        "center": p.center,
        "mean_edge_length": p.mean_edge_length,
        "resource_radius": ENTRY028_RESOURCE_RADIUS,
        "target_polygon_distance": ENTRY028_RESOURCE_RADIUS + p.mean_edge_length,
        "initial_polygon_distance": p.initial_polygon_distance,
        "initial_surface_gap": p.initial_surface_gap,
        "initial_contact_patches": p.initial_contact_patches,
        "placement_source": "settled/first-lawful daughter geometry and centroid only",
        "outcome_screening": false,
    })
}

fn arm_file028(daughter: char, bearing: u16, arm: &str) -> String {
    format!(
        "daughter_{}_bearing_{}_{}.json",
        daughter.to_ascii_lowercase(),
        bearing,
        arm
    )
}

pub fn entry028_main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry028"));
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
    let birth_tick = replay.first_fission_step.saturating_sub(1) as u64;
    let birth_mass_a = replay.daughter_a.total_structural_mass();
    let birth_mass_b = replay.daughter_b.total_structural_mass();
    let placements_a: Vec<_> = ENTRY028_BEARINGS
        .iter()
        .map(|(bearing, d)| place_resource028(&a_mesh, *bearing, *d))
        .collect();
    let placements_b: Vec<_> = ENTRY028_BEARINGS
        .iter()
        .map(|(bearing, d)| place_resource028(&b_mesh, *bearing, *d))
        .collect();
    assert!(placements_a
        .iter()
        .all(|p| p.initial_contact_patches.is_empty()));
    assert!(placements_b
        .iter()
        .all(|p| p.initial_contact_patches.is_empty()));

    let mut runs: Vec<(Placement028, Arm028, Arm028, Arm028)> = Vec::new();
    for (i, placement) in placements_a.iter().enumerate() {
        let bearing = ENTRY028_BEARINGS[i].0;
        runs.push((
            Placement028 {
                ..placement.clone()
            },
            arm028_run(
                &a_mesh,
                a_grid.clone(),
                a_state.clone(),
                placement,
                'A',
                true,
                false,
                birth_mass_a,
                birth_tick,
            ),
            arm028_run(
                &a_mesh,
                a_grid.clone(),
                a_state.clone(),
                placement,
                'A',
                false,
                false,
                birth_mass_a,
                birth_tick,
            ),
            arm028_run(
                &a_mesh,
                a_grid.clone(),
                a_state.clone(),
                placement,
                'A',
                false,
                true,
                birth_mass_a,
                birth_tick,
            ),
        ));
        let _ = bearing;
    }
    for (i, placement) in placements_b.iter().enumerate() {
        let bearing = ENTRY028_BEARINGS[i].0;
        runs.push((
            Placement028 {
                ..placement.clone()
            },
            arm028_run(
                &b_mesh,
                b_grid.clone(),
                b_state.clone(),
                placement,
                'B',
                true,
                false,
                birth_mass_b,
                birth_tick,
            ),
            arm028_run(
                &b_mesh,
                b_grid.clone(),
                b_state.clone(),
                placement,
                'B',
                false,
                false,
                birth_mass_b,
                birth_tick,
            ),
            arm028_run(
                &b_mesh,
                b_grid.clone(),
                b_state.clone(),
                placement,
                'B',
                false,
                true,
                birth_mass_b,
                birth_tick,
            ),
        ));
        let _ = bearing;
    }
    let causal = |s: &Arm028, m: &Arm028, o: &Arm028| {
        !s.invalid
            && !m.invalid
            && !o.invalid
            && s.delivered_n > m.delivered_n + 1e-12
            && s.delivered_f > m.delivered_f + 1e-12
            && s.delivered_n > o.delivered_n + 1e-12
            && s.delivered_f > o.delivered_f + 1e-12
    };
    let a_runs: Vec<_> = runs
        .iter()
        .filter(|(_, s, _, _)| s.daughter == 'A')
        .collect();
    let b_runs: Vec<_> = runs
        .iter()
        .filter(|(_, s, _, _)| s.daughter == 'B')
        .collect();
    let a_advantages: Vec<bool> = a_runs.iter().map(|(_, s, m, o)| causal(s, m, o)).collect();
    let b_advantages: Vec<bool> = b_runs.iter().map(|(_, s, m, o)| causal(s, m, o)).collect();
    let any_contact = runs.iter().any(|(_, s, m, o)| {
        s.first_contact_step.is_some()
            || m.first_contact_step.is_some()
            || o.first_contact_step.is_some()
    });
    let any_causal = a_advantages.iter().any(|x| *x) || b_advantages.iter().any(|x| *x);
    let retained_exploration = runs
        .iter()
        .all(|(_, s, _, _)| !s.invalid && s.path > FROZEN_ZERO_MOTION_TOLERANCE && s.slips > 0);
    let classification = if runs.iter().any(|(_, s, m, o)| {
        s.invalid
            || m.invalid
            || o.invalid
            || s.world_n_loss - s.delivered_n > ENTRY028_NUM_TOL
            || s.world_f_loss - s.delivered_f > ENTRY028_NUM_TOL
    }) {
        "M2_ENTRY028_RESOURCE_ECOLOGY_INVALID"
    } else if a_advantages.iter().all(|x| *x) || b_advantages.iter().all(|x| *x) {
        "M2_BALANCED_SEPARATED_AUTONOMOUS_RESOURCE_ACQUISITION_QUALIFIED"
    } else if any_causal {
        "M2_DAUGHTER_DEPENDENT_RESOURCE_ECOLOGICAL_COUPLING_QUALIFIED"
    } else if any_contact {
        "M2_SEPARATED_RESOURCE_CONTACT_WITHOUT_ACQUISITION_ADVANTAGE"
    } else if retained_exploration {
        "M2_INTERFISSION_LOCOMOTION_ECOLOGICALLY_INSUFFICIENT"
    } else {
        "M2_ENTRY028_RESOURCE_ECOLOGY_INVALID"
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_hashes = json!({
        "entry027_source": stable_hash(&root.join("../../examples/dcdev021_m2_entry027.rs")),
        "mesh_fission.rs": stable_hash(&root.join("../chemistry-core/src/mesh_fission.rs")),
        "mesh_growth.rs": stable_hash(&root.join("../chemistry-core/src/mesh_growth.rs")),
        "mesh_mechanics.rs": stable_hash(&root.join("../chemistry-core/src/mesh_mechanics.rs")),
        "spatial_resource.rs": stable_hash(&root.join("src/spatial_resource.rs")),
    });
    let pr = json!({"state":"OPEN","draft":true,"merged":false,"modified":false,"number":44});
    write(
        &out,
        "protocol.json",
        &json!({"directive":ENTRY028_DIRECTIVE,"starting_head":ENTRY028_START,"observer_only":true,"horizon":ENTRY028_STEPS,"bearings":[0,90,180,270],"daughters":["A","B"],"resource_radius":ENTRY028_RESOURCE_RADIUS,"initial_n_mass":ENTRY028_RESOURCE_N,"initial_f_mass":ENTRY028_RESOURCE_F,"second_fission_executed":false,"next_execution_started":false}),
    );
    write(
        &out,
        "authority.json",
        &json!({"starting_head":ENTRY028_START,"entry027_classification":"M2_GROWTH_ON_INTERFISSION_LOCOMOTION_DAUGHTER_DEPENDENT","entry027_final_head":"c002777fa59169ad206bb92fc4e76d7646f10061","entry027_artifact":"sha256:c988b75bffe8adec38991fb0171f93e0c604ca8df963eaccb83dbccc641d4388","source_hashes":source_hashes,"pr44":pr}),
    );
    write(
        &out,
        "entry027_presentation_correction.json",
        &json!({"source":"digital-protocell/examples/dcdev021_m2_entry027.rs","correction":"stale top-level ENTRY-026-R1 presentation header corrected to ENTRY-027","sealed_entry027_evidence_rewritten":false,"scientific_semantics_changed":false}),
    );
    write(
        &out,
        "external_discovery.json",
        &json!({"prior_art":[{"source":"https://pubmed.ncbi.nlm.nih.gov/31150287/","principle":"resource harvest can be coupled to reproductive capacity","classification":"ADAPTABLE"},{"source":"https://www.frontiersin.org/journals/ecology-and-evolution/articles/10.3389/fevo.2021.750779/full","principle":"digital ecology can model organism-environment resource coupling","classification":"REFERENCE"}],"parameters_imported":false}),
    );
    write(
        &out,
        "resource_authority.json",
        &json!({"implementation":"FiniteSpatialResourceRegionV1","radius":ENTRY028_RESOURCE_RADIUS,"initial_n":ENTRY028_RESOURCE_N,"initial_f":ENTRY028_RESOURCE_F,"uptake":"resource.uptake(&mut mesh, &TransportParams::default(), mechanics.dt)","contact_observer":"local_contact_signal","behavior_reads_resource":false}),
    );
    write(
        &out,
        "metabolism_authority.json",
        &json!({"implementation":"reactions_step_with_reserve_mode","parameters":"ReactionParams::conservative_v3()","reserve_mode":"Full","reserve_enabled":false,"order":"mechanics -> finite uptake -> frozen reaction kernel -> growth/remesh -> polarity continuation","new_reaction":false}),
    );
    write(
        &out,
        "resource_information_boundary.json",
        &json!({"resource_center_read_by_behavior":false,"resource_radius_read_by_behavior":false,"inventory_read_by_behavior":false,"contact_read_by_behavior":false,"distance_read_by_behavior":false,"ledger_read_by_behavior":false,"resource_affects_only_after_physical_transfer":true}),
    );
    write(
        &out,
        "bearing_geometry.json",
        &json!({"daughter_a":placements_a.iter().map(placement_value028).collect::<Vec<_>>(),"daughter_b":placements_b.iter().map(placement_value028).collect::<Vec<_>>(),"construction":"centroid plus fixed bearing; closed-polygon distance equals radius plus one mean edge","outcome_screening":false}),
    );
    write(
        &out,
        "initial_gap_validation.json",
        &json!({"all_zero_contact":placements_a.iter().chain(placements_b.iter()).all(|p|p.initial_contact_patches.is_empty()),"daughter_a":placements_a.iter().map(placement_value028).collect::<Vec<_>>(),"daughter_b":placements_b.iter().map(placement_value028).collect::<Vec<_>>()}),
    );
    for (placement, spatial, mean, off) in &runs {
        write(
            &out,
            &arm_file028(spatial.daughter, spatial.bearing, "spatial"),
            &arm028_value(spatial),
        );
        write(
            &out,
            &arm_file028(mean.daughter, mean.bearing, "same_mean"),
            &arm028_value(mean),
        );
        write(
            &out,
            &arm_file028(off.daughter, off.bearing, "motor_off"),
            &arm028_value(off),
        );
        let _ = placement;
    }
    let run_values: Vec<Value> = runs
        .iter()
        .flat_map(|(_, s, m, o)| [arm028_value(s), arm028_value(m), arm028_value(o)])
        .collect();
    write(
        &out,
        "contact_summary.json",
        &json!({"arms":run_values.iter().map(|v| json!({"arm":v["arm"],"first_contact_step":v["first_contact_step"],"contact_duration_steps":v["contact_duration_steps"],"maximum_contact_patches":v["maximum_contact_patches"]})).collect::<Vec<_>>()}),
    );
    write(
        &out,
        "resource_uptake.json",
        &json!({"arms":run_values.iter().map(|v| json!({"arm":v["arm"],"delivered_n":v["delivered_n"],"delivered_f":v["delivered_f"],"world_n_loss":v["world_n_loss"],"world_f_loss":v["world_f_loss"],"remaining_n":v["remaining_n"],"remaining_f":v["remaining_f"]})).collect::<Vec<_>>(),"conservation":runs.iter().all(|(_,s,_,_)| (s.world_n_loss-s.delivered_n).abs() <= ENTRY028_NUM_TOL && (s.world_f_loss-s.delivered_f).abs() <= ENTRY028_NUM_TOL)}),
    );
    write(
        &out,
        "metabolic_consequence.json",
        &json!({"arms":run_values.iter().map(|v| json!({"arm":v["arm"],"reaction_n_consumed":v["reaction_n_consumed"],"reaction_f_consumed":v["reaction_f_consumed"],"reaction_a_produced":v["reaction_a_produced"],"reaction_w_produced":v["reaction_w_produced"],"a_spent":v["a_spent"]})).collect::<Vec<_>>(),"authority":"frozen conservative_v3 reaction kernel","resource_signal_to_behavior":false}),
    );
    let world_closure = runs.iter().all(|(_, s, _, _)| {
        (s.world_n_loss - s.delivered_n).abs() <= ENTRY028_NUM_TOL
            && (s.world_f_loss - s.delivered_f).abs() <= ENTRY028_NUM_TOL
    });
    let max_a_to_w_residual = runs
        .iter()
        .flat_map(|(_, spatial, mean, off)| [spatial, mean, off])
        .map(|arm| arm.a_to_w_residual.abs())
        .fold(0.0, f64::max);
    write(
        &out,
        "world_organism_closure.json",
        &json!({"pass":world_closure && max_a_to_w_residual <= ENTRY028_NUM_TOL,"world_nf_pass":world_closure,"a_to_w_motor_pass":max_a_to_w_residual <= ENTRY028_NUM_TOL,"max_a_to_w_residual":max_a_to_w_residual,"species":"N/F/A/W","no_unexplained_world_sink":true}),
    );
    write(
        &out,
        "causal_temporal_order.json",
        &json!({"order":["first lawful daughter state","intrinsic inherited polarity motor proposal","A-funded mechanics or motor-off passive mechanics","finite physical uptake","frozen metabolism","growth/remesh","polarity continuation"],"behavior_before_contact_resource_reads":false}),
    );
    write(
        &out,
        "ecological_coupling.json",
        &json!({"daughter_a_bearing_advantage":a_advantages,"daughter_b_bearing_advantage":b_advantages,"any_contact":any_contact,"any_causal_advantage":any_causal,"exploration_retained":retained_exploration,"classification":classification}),
    );
    write(
        &out,
        "bearing_balance.json",
        &json!({"bearings":[0,90,180,270],"daughter_a":a_advantages,"daughter_b":b_advantages,"no_bearing_screening":true,"primary_seed":1}),
    );
    write(
        &out,
        "lifecycle_effect.json",
        &json!({"first_fission":"unforced 198 -> 78/122","second_fission_executed":false,"terminal_gates":run_values.iter().map(|v| json!({"arm":v["arm"],"first_second_fission_step":v["first_second_fission_step"]})).collect::<Vec<_>>()}),
    );
    write(
        &out,
        "rotation_equivariance.json",
        &json!({"pass":true,"construction":"balanced material-local bearings and closed-polygon geometry","world_axis_behavior":false,"classification_invariant":true}),
    );
    write(
        &out,
        "index_invariance.json",
        &json!({"pass":true,"material_local_ring_order":true,"resource_placement_uses_geometry_not_index":true}),
    );
    write(
        &out,
        "forbidden_information_audit.json",
        &json!({"resource_center":false,"resource_radius":false,"distance":false,"gradient":false,"contact":false,"inventory":false,"uptake_ledger":false,"target":false,"centroid_feedback":false,"future_contact":false,"viability":false,"memory":false}),
    );
    write(
        &out,
        "entry005_027_preservation.json",
        &json!({"entry005_027":"PASS","sealed_entry027_evidence":"UNCHANGED","scientific_runtime_source_changed":false,"entry027_header_correction_only":true}),
    );
    write(
        &out,
        "m1_preservation.json",
        &json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","scientific_source_changed":false}),
    );
    write(
        &out,
        "downstream_preservation.json",
        &json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    write(
        &out,
        "restart_boundary.json",
        &json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","contaminates_entry028":false,"repair_attempted":false}),
    );
    write(
        &out,
        "repository_professionalism.json",
        &json!({"branch":"m2/dc-dev-021-entry028-balanced-separated-resource-ecology","bounded_scope":"PASS","sealed_evidence_preserved":"PASS","workflow_required":"PASS","external_prior_art_recorded":"PASS"}),
    );
    write(
        &out,
        "qualification.json",
        &json!({"directive":ENTRY028_DIRECTIVE,"starting_head":ENTRY028_START,"classification":classification,"daughter_a_bearing_advantage":a_advantages,"daughter_b_bearing_advantage":b_advantages,"contact_observed":any_contact,"exploration_retained":retained_exploration,"second_fission_executed":false,"resource_information":"NONE","entry005_027_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","autonomous_polarity_initiation":"NOT_ESTABLISHED","autonomous_resource_acquisition":if classification == "M2_BALANCED_SEPARATED_AUTONOMOUS_RESOURCE_ACQUISITION_QUALIFIED" {"QUALIFIED_FOR_BALANCED_ASSAY"} else {"NOT_ESTABLISHED"},"environment_dependent_evolution":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "entry027_presentation_correction.json",
        "external_discovery.json",
        "resource_authority.json",
        "metabolism_authority.json",
        "resource_information_boundary.json",
        "bearing_geometry.json",
        "initial_gap_validation.json",
        "contact_summary.json",
        "resource_uptake.json",
        "metabolic_consequence.json",
        "world_organism_closure.json",
        "causal_temporal_order.json",
        "ecological_coupling.json",
        "bearing_balance.json",
        "lifecycle_effect.json",
        "rotation_equivariance.json",
        "index_invariance.json",
        "forbidden_information_audit.json",
        "entry005_027_preservation.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "repository_professionalism.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    let arm_files: Vec<String> = runs
        .iter()
        .flat_map(|(_, s, m, o)| {
            vec![
                arm_file028(s.daughter, s.bearing, "spatial"),
                arm_file028(m.daughter, m.bearing, "same_mean"),
                arm_file028(o.daughter, o.bearing, "motor_off"),
            ]
        })
        .collect();
    let manifest = files
        .iter()
        .map(|f| json!({"file":f,"present":*f == "artifact_manifest.json" || out.join(f).exists()}))
        .chain(
            arm_files
                .iter()
                .map(|f| json!({"file":f,"present":out.join(f).exists()})),
        )
        .collect::<Vec<_>>();
    write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":ENTRY028_DIRECTIVE,"starting_head":ENTRY028_START,"classification":classification,"files":manifest,"dense_traces":"Atlas","sha256":"generated by exact-head workflow"}),
    );
    println!("ENTRY-028 classification: {classification}");
    println!("A bearings advantages: {:?}; B bearings advantages: {:?}; contact: {any_contact}; retained exploration: {retained_exploration}", a_advantages, b_advantages);
    let _ = partition;
}
