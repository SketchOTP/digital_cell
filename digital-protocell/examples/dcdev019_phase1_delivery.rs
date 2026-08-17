//! DC-DEV-019 Phase 0 and Phase 1 finite nutrient delivery audit.
//!
//! This example is observer/counterfactual only. It reconstructs the clean
//! DC-DEV-016 body, tests the accepted source-saturated finite-resource arm,
//! and separates finite inventory from membrane delivery throughput. No
//! persistent controller or production behavior is introduced.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, reactions_step_counterfactual, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{
    stable_json_hash, FiniteSpatialResourceRegionV1, SpatialResourceStepLedgerV1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ENTRY: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const SETTLE_STEPS: usize = 5_000;
const FEED_STEPS: usize = 480;
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const INVENTORY: f64 = 14.588954880632265;
const DEPRIVED_E_STORED: f64 = 60.82781514212436;
const ACCEPTED_D1_FINAL_E_STORED: f64 = 59.1464166923814;
const EPS: f64 = 1e-10;
const MAX_EXPANSIONS: usize = 8;
const MAX_BISECTION: usize = 64;

#[derive(Debug, Clone, Copy, Serialize)]
struct Snapshot {
    step: usize,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    e_stored: f64,
    alive: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmRun {
    arm: String,
    resource_center: [f64; 2],
    resource_radius: f64,
    inventory: f64,
    delivery_multiplier: f64,
    initial: Snapshot,
    final_state: Snapshot,
    delivered_n: f64,
    delivered_f: f64,
    consumed_n: f64,
    consumed_f: f64,
    final_resource_n: f64,
    final_resource_f: f64,
    max_resource_error: f64,
    resource_conservation: bool,
    alive: bool,
    finite_nonnegative: bool,
    trajectory_hash: String,
    final_mesh_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct RootPoint {
    mass: f64,
    final_e_stored: f64,
    passed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeliveryDiagnostic {
    r1_reference_replay: ArmRun,
    d1_existing_delivery: ArmRun,
    d2_ideal_transport: ArmRun,
    current_inventory_sufficient: bool,
    selected_inventory: f64,
    selected_inventory_derivation: String,
    phase_1b_bracket: Vec<RootPoint>,
    phase_1b_iterations: Vec<RootPoint>,
    phase_1c_passive_selected: Option<ArmRun>,
    phase_1d_transport_iterations: Vec<RootPoint>,
    g_transport_max: f64,
    finite_delivery_finding: String,
    gate_1_pass: bool,
    gate_2_pass: bool,
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn snapshot(mesh: &MaterialMesh, step: usize) -> Snapshot {
    Snapshot {
        step,
        area: mesh.area(),
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        e_stored: mesh.area() * (mesh.interior.a + mesh.interior.r),
        alive: mesh.alive,
    }
}

fn seed() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = ReactionParams::default();
    params.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    params
}

fn settle(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed();
    for _ in 0..SETTLE_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.alive);
    mesh
}

fn deprive(settled: &MaterialMesh, mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = settled.clone();
    let params = reaction_params(&mesh);
    for _ in 0..FEED_STEPS {
        reactions_step(&mut mesh, &params, mechanics.dt, true, true);
    }
    mesh
}

fn midpoint_exposed(mesh: &MaterialMesh, region: &FiniteSpatialResourceRegionV1) -> bool {
    (0..mesh.n()).any(|edge| {
        if mesh.edges[edge].ruptured {
            return false;
        }
        let a = mesh.vertices[edge];
        let b = mesh.vertices[(edge + 1) % mesh.n()];
        let midpoint = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        (midpoint[0] - region.center[0]).hypot(midpoint[1] - region.center[1]) <= region.radius
    })
}

fn ideal_transfer(
    mesh: &mut MaterialMesh,
    region: &mut FiniteSpatialResourceRegionV1,
) -> SpatialResourceStepLedgerV1 {
    let mut ledger = SpatialResourceStepLedgerV1::default();
    if !mesh.alive || !midpoint_exposed(mesh, region) {
        return ledger;
    }
    ledger.exposed_edges = 1;
    let area = mesh.area().max(1e-6);
    let n_need = (region.boundary_n_concentration - mesh.interior.n.max(0.0)).max(0.0) * area;
    let f_need = (region.boundary_f_concentration - mesh.interior.f.max(0.0)).max(0.0) * area;
    let n_delta = n_need.min(region.n_mass).max(0.0);
    let f_delta = f_need.min(region.f_mass).max(0.0);
    region.n_mass -= n_delta;
    region.f_mass -= f_delta;
    mesh.interior.n += n_delta / area;
    mesh.interior.f += f_delta / area;
    ledger.n_world_loss = n_delta;
    ledger.f_world_loss = f_delta;
    ledger.n_delivered = n_delta;
    ledger.f_delivered = f_delta;
    ledger
}

fn source_saturated_step(
    mesh: &MaterialMesh,
    params: &ReactionParams,
    dt: f64,
) -> (MaterialMesh, f64, f64, f64) {
    let area = mesh.area().max(1e-15);
    let mut one_mesh = mesh.clone();
    let one = reactions_step_counterfactual(&mut one_mesh, params, dt, true, true, 1.0);
    let capacity = (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area);
    let g_sat = if one.n_consumed > EPS {
        capacity / one.n_consumed
    } else {
        1.0
    };
    let gain = g_sat.max(1.0);
    let mut after = mesh.clone();
    let led = reactions_step_counterfactual(&mut after, params, dt, true, true, gain);
    (after, led.n_consumed, led.f_consumed, gain)
}

#[derive(Clone, Copy)]
enum DeliveryMode {
    Existing(f64),
    Ideal,
}

fn run_arm_with_geometry(
    initial: &MaterialMesh,
    inventory: f64,
    mode: DeliveryMode,
    mechanics: &MechParams,
    name: &str,
    center: [f64; 2],
    radius: f64,
) -> ArmRun {
    let mut mesh = initial.clone();
    let params = reaction_params(&mesh);
    let mut region = FiniteSpatialResourceRegionV1::new(center, radius, inventory, inventory);
    let initial_snapshot = snapshot(&mesh, 0);
    let mut hashes = vec![stable_json_hash(&initial_snapshot).unwrap()];
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut consumed_n = 0.0;
    let mut consumed_f = 0.0;
    let mut max_resource_error: f64 = 0.0;
    let mut finite_nonnegative = true;
    for step in 0..FEED_STEPS {
        let uptake = match mode {
            DeliveryMode::Existing(multiplier) => region.uptake_with_capacity_multiplier(
                &mut mesh,
                &TransportParams::default(),
                mechanics.dt,
                multiplier,
            ),
            DeliveryMode::Ideal => ideal_transfer(&mut mesh, &mut region),
        };
        delivered_n += uptake.n_delivered;
        delivered_f += uptake.f_delivered;
        max_resource_error = max_resource_error.max(uptake.conservation_error.abs());
        let (after, n_used, f_used, _) = source_saturated_step(&mesh, &params, mechanics.dt);
        mesh = after;
        consumed_n += n_used;
        consumed_f += f_used;
        finite_nonnegative &= [
            mesh.interior.a,
            mesh.interior.r,
            mesh.interior.n,
            mesh.interior.f,
            mesh.interior.c,
            mesh.interior.w,
            region.n_mass,
            region.f_mass,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= -EPS);
        hashes.push(stable_json_hash(&snapshot(&mesh, step + 1)).unwrap());
    }
    let final_snapshot = snapshot(&mesh, FEED_STEPS);
    let conservation = max_resource_error <= EPS
        && (inventory - region.n_mass - delivered_n).abs() <= EPS
        && (inventory - region.f_mass - delivered_f).abs() <= EPS
        && (initial_snapshot.n * initial_snapshot.area + delivered_n
            - consumed_n
            - final_snapshot.n * final_snapshot.area)
            .abs()
            <= EPS
        && (initial_snapshot.f * initial_snapshot.area + delivered_f
            - consumed_f
            - final_snapshot.f * final_snapshot.area)
            .abs()
            <= EPS;
    ArmRun {
        arm: name.to_string(),
        resource_center: center,
        resource_radius: radius,
        inventory,
        delivery_multiplier: match mode {
            DeliveryMode::Existing(g) => g,
            DeliveryMode::Ideal => f64::INFINITY,
        },
        initial: initial_snapshot,
        final_state: final_snapshot,
        delivered_n,
        delivered_f,
        consumed_n,
        consumed_f,
        final_resource_n: region.n_mass,
        final_resource_f: region.f_mass,
        max_resource_error,
        resource_conservation: conservation,
        alive: mesh.alive,
        finite_nonnegative,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
    }
}

fn run_arm(
    initial: &MaterialMesh,
    inventory: f64,
    mode: DeliveryMode,
    mechanics: &MechParams,
    name: &str,
) -> ArmRun {
    run_arm_with_geometry(initial, inventory, mode, mechanics, name, CENTER, RADIUS)
}

fn passed(run: &ArmRun) -> bool {
    run.final_state.e_stored > DEPRIVED_E_STORED + EPS
        && run.resource_conservation
        && run.alive
        && run.finite_nonnegative
}

fn resource_uptake_gain_one_parity(initial: &MaterialMesh, mechanics: &MechParams) -> bool {
    let mut direct_mesh = initial.clone();
    let mut diagnostic_mesh = initial.clone();
    let mut direct_region =
        FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, INVENTORY, INVENTORY);
    let mut diagnostic_region = direct_region.clone();
    let direct = direct_region.uptake(&mut direct_mesh, &TransportParams::default(), mechanics.dt);
    let diagnostic = diagnostic_region.uptake_with_capacity_multiplier(
        &mut diagnostic_mesh,
        &TransportParams::default(),
        mechanics.dt,
        1.0,
    );
    direct == diagnostic
        && direct_region == diagnostic_region
        && stable_json_hash(&direct_mesh).unwrap() == stable_json_hash(&diagnostic_mesh).unwrap()
}

fn chemistry_gain_one_parity(initial: &MaterialMesh, mechanics: &MechParams) -> bool {
    let params = reaction_params(initial);
    let mut direct = initial.clone();
    let mut diagnostic = initial.clone();
    let direct_ledger = reactions_step(&mut direct, &params, mechanics.dt, true, true);
    let diagnostic_ledger =
        reactions_step_counterfactual(&mut diagnostic, &params, mechanics.dt, true, true, 1.0);
    stable_json_hash(&direct).unwrap() == stable_json_hash(&diagnostic).unwrap()
        && stable_json_hash(&direct_ledger).unwrap()
            == stable_json_hash(&diagnostic_ledger).unwrap()
}

fn ideal_run(initial: &MaterialMesh, mass: f64, mechanics: &MechParams, name: &str) -> ArmRun {
    run_arm(initial, mass, DeliveryMode::Ideal, mechanics, name)
}

fn passive_run(
    initial: &MaterialMesh,
    mass: f64,
    gain: f64,
    mechanics: &MechParams,
    name: &str,
) -> ArmRun {
    run_arm(initial, mass, DeliveryMode::Existing(gain), mechanics, name)
}

fn passive_run_at(
    initial: &MaterialMesh,
    mass: f64,
    gain: f64,
    mechanics: &MechParams,
    name: &str,
    center: [f64; 2],
    radius: f64,
) -> ArmRun {
    run_arm_with_geometry(
        initial,
        mass,
        DeliveryMode::Existing(gain),
        mechanics,
        name,
        center,
        radius,
    )
}

fn main() {
    let out = std::env::var_os("DCDEV019_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev019/phase1"));
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= EPS);
    let settled = settle(&mechanics);
    let deprived = deprive(&settled, &mechanics);
    let settled_hash = stable_json_hash(&settled).unwrap();
    let deprived_hash = stable_json_hash(&deprived).unwrap();
    let target_e_stored = snapshot(&settled, SETTLE_STEPS).e_stored;
    let deprived_e_stored = snapshot(&deprived, FEED_STEPS).e_stored;
    assert!((deprived_e_stored - DEPRIVED_E_STORED).abs() <= EPS);
    let chemistry_parity = chemistry_gain_one_parity(&deprived, &mechanics);
    let resource_parity = resource_uptake_gain_one_parity(&deprived, &mechanics);
    assert!(chemistry_parity, "gain=1 chemistry parity failed");
    assert!(resource_parity, "gain=1 uptake parity failed");

    let r1_reference = passive_run_at(
        &deprived,
        INVENTORY,
        1.0,
        &mechanics,
        "R1_reference_replay",
        [0.0, 0.0],
        5.0,
    );
    let d1 = passive_run(
        &deprived,
        INVENTORY,
        1.0,
        &mechanics,
        "D1_existing_delivery",
    );
    assert!(
        (r1_reference.final_state.e_stored - ACCEPTED_D1_FINAL_E_STORED).abs() <= EPS,
        "R1 reference replay did not reproduce accepted result: {}",
        r1_reference.final_state.e_stored
    );
    let d2 = ideal_run(
        &deprived,
        INVENTORY,
        &mechanics,
        "D2_ideal_transport_upper_bound",
    );
    let current_inventory_sufficient = passed(&d2);
    let mut bracket = Vec::new();
    let mut iterations = Vec::new();
    let mut selected_inventory = INVENTORY;
    let mut selected_derivation = "current inventory retained because D2 passed".to_string();
    if !current_inventory_sufficient {
        let mut low = INVENTORY;
        let mut high = INVENTORY;
        let mut high_run = d2.clone();
        for _ in 0..MAX_EXPANSIONS {
            high *= 2.0;
            high_run = ideal_run(&deprived, high, &mechanics, "D2_ideal_bracket");
            bracket.push(RootPoint {
                mass: high,
                final_e_stored: high_run.final_state.e_stored,
                passed: passed(&high_run),
            });
            if passed(&high_run) {
                break;
            }
            low = high;
        }
        if !passed(&high_run) {
            selected_derivation =
                "no finite ideal-transport bracket reached break-even".to_string();
        } else {
            for _ in 0..MAX_BISECTION {
                let mid = (low + high) * 0.5;
                let mid_run = ideal_run(&deprived, mid, &mechanics, "D2_ideal_bisection");
                let ok = passed(&mid_run);
                iterations.push(RootPoint {
                    mass: mid,
                    final_e_stored: mid_run.final_state.e_stored,
                    passed: ok,
                });
                if ok {
                    high = mid;
                } else {
                    low = mid;
                }
                if (high - low).abs() <= EPS {
                    break;
                }
            }
            selected_inventory = high;
            selected_derivation =
                "minimum equal N/F inventory by deterministic ideal-transport bisection"
                    .to_string();
        }
    }
    let passive_selected = if selected_derivation.starts_with("no finite") {
        None
    } else {
        Some(passive_run(
            &deprived,
            selected_inventory,
            1.0,
            &mechanics,
            "D1_passive_selected_inventory",
        ))
    };
    let mut transport_iterations = Vec::new();
    let mut g_transport_max = 1.0;
    if let Some(passive) = &passive_selected {
        if !passed(passive) {
            let mut low = 1.0;
            let mut high = 1.0;
            let mut high_run = passive.clone();
            for _ in 0..MAX_EXPANSIONS {
                high *= 2.0;
                high_run = passive_run(
                    &deprived,
                    selected_inventory,
                    high,
                    &mechanics,
                    "D1_transport_bracket",
                );
                transport_iterations.push(RootPoint {
                    mass: high,
                    final_e_stored: high_run.final_state.e_stored,
                    passed: passed(&high_run),
                });
                if passed(&high_run) {
                    break;
                }
                low = high;
            }
            if passed(&high_run) {
                for _ in 0..MAX_BISECTION {
                    let mid = (low + high) * 0.5;
                    let mid_run = passive_run(
                        &deprived,
                        selected_inventory,
                        mid,
                        &mechanics,
                        "D1_transport_bisection",
                    );
                    let ok = passed(&mid_run);
                    transport_iterations.push(RootPoint {
                        mass: mid,
                        final_e_stored: mid_run.final_state.e_stored,
                        passed: ok,
                    });
                    if ok {
                        high = mid;
                    } else {
                        low = mid;
                    }
                    if (high - low).abs() <= EPS {
                        break;
                    }
                }
                g_transport_max = high;
            }
        }
    }
    let gate_1 = (r1_reference.final_state.e_stored - ACCEPTED_D1_FINAL_E_STORED).abs() <= EPS
        && d1.resource_center == CENTER
        && (d1.resource_radius - RADIUS).abs() <= EPS
        && d1.resource_conservation
        && d1.alive
        && d1.finite_nonnegative
        && chemistry_parity
        && resource_parity;
    let gate_2 = passive_selected.as_ref().map(passed).unwrap_or(false)
        || (g_transport_max > 1.0
            && passive_selected.is_some()
            && passed(&passive_run(
                &deprived,
                selected_inventory,
                g_transport_max,
                &mechanics,
                "D1_transport_selected",
            )));
    let finding = if !gate_1 {
        "DCDEV019_ENTRY_OR_RESOURCE_CONTRACT_INVALID"
    } else if selected_derivation.starts_with("no finite") {
        "DCDEV019_FINITE_RESOURCE_ECOLOGY_NOT_ESTABLISHED"
    } else if !gate_2 {
        "DCDEV019_FINITE_NUTRIENT_DELIVERY_NOT_ESTABLISHED"
    } else if current_inventory_sufficient && g_transport_max <= 1.0 + EPS {
        "DCDEV019_CURRENT_INVENTORY_AND_PASSIVE_DELIVERY_SUFFICIENT"
    } else if current_inventory_sufficient {
        "DCDEV019_CURRENT_INVENTORY_SUFFICIENT_DELIVERY_LIMIT_CONFIRMED"
    } else {
        "DCDEV019_FINITE_NUTRIENT_DELIVERY_CAPACITY_ESTABLISHED"
    };
    let diagnostic = DeliveryDiagnostic {
        r1_reference_replay: r1_reference.clone(),
        d1_existing_delivery: d1.clone(),
        d2_ideal_transport: d2.clone(),
        current_inventory_sufficient,
        selected_inventory,
        selected_inventory_derivation: selected_derivation,
        phase_1b_bracket: bracket,
        phase_1b_iterations: iterations,
        phase_1c_passive_selected: passive_selected,
        phase_1d_transport_iterations: transport_iterations,
        g_transport_max,
        finite_delivery_finding: finding.to_string(),
        gate_1_pass: gate_1,
        gate_2_pass: gate_2,
    };
    write(
        &out,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-019", "phase":"0-1", "entry_commit":ENTRY,
            "base_branch":"strategy/dc-dev-016-metabolic-break-even",
            "settlement_steps":SETTLE_STEPS, "feeding_steps":FEED_STEPS, "dt":mechanics.dt,
            "resource":{"center":CENTER,"radius":RADIUS,"material_volume":std::f64::consts::PI*RADIUS*RADIUS,
                "initial_n_mass":INVENTORY,"initial_f_mass":INVENTORY,
                "boundary_concentration":"initial mass / disk material volume, fixed while inventory remains",
                "edge_exposure":"intact mesh-edge midpoint inside static disk",
                "uptake":"existing permeability(theta,species) * k_flux * positive concentration gradient * segment length * dt, inventory capped",
                "ideal_transport":"well-mixed equilibration upper bound, exposure/gradient/world-mass constrained"},
            "source_saturation":"observer-only gain on existing N/F -> A extent; no direct A/R writes",
            "observer_only":true,"production_behavior_changed":false,"homeostat_started":false,"dcdev020_started":false
        }),
    );
    write(
        &out,
        "entry_parity.json",
        &json!({
            "settled_body_hash":settled_hash,"deprived_body_hash":deprived_hash,
            "target_e_stored":target_e_stored,"deprived_e_stored":deprived_e_stored,
            "expected_deprived_e_stored":DEPRIVED_E_STORED,
            "chemistry_gain_1_parity":chemistry_parity,"resource_gain_1_parity":resource_parity,
            "r1_reference_replay_e_stored":r1_reference.final_state.e_stored,
            "r1_reference_replay_expected":ACCEPTED_D1_FINAL_E_STORED,
            "pass":chemistry_parity && resource_parity && (deprived_e_stored-DEPRIVED_E_STORED).abs()<=EPS
                && (r1_reference.final_state.e_stored-ACCEPTED_D1_FINAL_E_STORED).abs()<=EPS
        }),
    );
    write(
        &out,
        "resource_semantics.json",
        &json!({
            "schema":"dcdev019_live_resource_semantics_v1","shape":"static_disk",
            "center":CENTER,"radius":RADIUS,"material_volume":std::f64::consts::PI*RADIUS*RADIUS,
            "boundary_concentration":"fixed from initial mass; not recomputed after depletion",
            "world_mass":"mutable finite n_mass/f_mass; delivery is exact world loss",
            "exposure":"edge midpoint containment and non-ruptured edge",
            "interior_concentration":"mesh interior species concentration; only positive external-minus-internal gradient transfers",
            "classification":"finite static reservoir with fixed boundary concentration and inventory clamp"
        }),
    );
    write(
        &out,
        "delivery_diagnostic.json",
        &serde_json::to_value(&diagnostic).unwrap(),
    );
    write(
        &out,
        "results.json",
        &json!({
            "directive":"DC-DEV-019","phase":"0-1","entry_commit":ENTRY,
            "settled_body_hash":settled_hash,"deprived_body_hash":deprived_hash,
            "diagnostic":diagnostic,"gate_1":gate_1,"gate_2":gate_2,
            "conclusion":"DCDEV019_FINITE_NUTRIENT_DELIVERY_PHASE_1_COMPLETE",
            "finite_delivery_finding":finding,"production_behavior_changed":false,
            "homeostat_started":false,"phase_4_started":false,"phase_5_started":false,
            "phase_6_started":false,"next_execution_started":false
        }),
    );
    write(
        &out,
        "artifact_manifest.json",
        &json!({
            "directive":"DC-DEV-019","phase":"0-1","entry_commit":ENTRY,
            "evidence_files":["protocol.json","entry_parity.json","resource_semantics.json","delivery_diagnostic.json","results.json","artifact_manifest.json"],
            "dense_raw_ledgers_external":true,"conclusion":"DCDEV019_FINITE_NUTRIENT_DELIVERY_PHASE_1_COMPLETE",
            "finding":finding,"next_execution_started":false
        }),
    );
    println!("DCDEV019_FINITE_NUTRIENT_DELIVERY_PHASE_1_COMPLETE\n{finding}\nD1_E_stored={}\nD2_E_stored={}\nM_selected={}\nG_transport_max={}\nsettled_hash={}\ndeprived_hash={}",
        d1.final_state.e_stored,d2.final_state.e_stored,selected_inventory,g_transport_max,settled_hash,deprived_hash);
}
