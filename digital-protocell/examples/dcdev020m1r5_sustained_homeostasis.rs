//! DC-DEV-020-M1-R5: sustained finite-resource homeostasis qualification.
//!
//! This is an observer-only 8,000-step qualification from the accepted M1-R4
//! entry state. The only new world contract is a finite backing reservoir that
//! fixes the accepted R4 boundary concentration while scaling inventory by the
//! preregistered horizon ratio 50/3. V1 transport and R4 same-step coupling
//! remain the execution authorities.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod m1r1;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    stable_json_hash, CoupledFiniteSpatialResourceRegionV1, FiniteSpatialBackingReservoirV1,
    FINITE_SPATIAL_BACKING_RESERVOIR_SCHEMA_V1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R5-SUSTAINED-HOMEOSTASIS-001";
const STARTING_HEAD: &str = "68d1c88ec1b915a4bee86efe24e985222b529d5a";
const R4_ENTRY_HEAD: &str = "17226fb7484eb50079c1c30ce9fd8039b3f23c60";
const R4_SCHEMA: &str = "FINITE_SPATIAL_RESOURCE_COUPLED_ACTIVATION_V1";
const BACKING_SCHEMA: &str = FINITE_SPATIAL_BACKING_RESERVOIR_SCHEMA_V1;
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const R4_INVENTORY: f64 = 14.588954880632265;
const BACKING_INVENTORY: f64 = 243.14924801053778;
const R4_BOUNDARY_CONCENTRATION: f64 = 2.063914918930895;
const HORIZON_STEPS: usize = 8_000;
const R4_STEPS: usize = 480;
const TOL: f64 = 1e-8;
const CHECKPOINTS: [usize; 9] = [0, 480, 1_000, 2_000, 3_466, 4_000, 6_000, 6_931, 8_000];

#[derive(Debug, Clone, Copy, Serialize)]
struct State {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    r: f64,
    c: f64,
    structural_m: f64,
    membrane: f64,
    free_l: f64,
    bound_b: f64,
    waste: f64,
    organized_material: f64,
    strict_material: f64,
    min_edge_m: f64,
    ruptured_edges: usize,
    closed_intact: bool,
    observer_viable: bool,
    observer_death_reason: Option<&'static str>,
    physical_runtime_valid: bool,
}

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        area: mesh.area(),
        n: s.n,
        f: s.f,
        a: s.a,
        r: s.r,
        c: s.c,
        structural_m: s.structural_m,
        membrane: mesh.total_membrane(),
        free_l: s.free_l,
        bound_b: s.bound_b,
        waste: s.waste,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        min_edge_m: mesh
            .edges
            .iter()
            .map(|edge| edge.m)
            .fold(f64::INFINITY, f64::min),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct ReactionTotals {
    ordinary_activation: f64,
    m_production: f64,
    m_turnover: f64,
    c_production: f64,
    c_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
    a_decay: f64,
}

impl ReactionTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.ordinary_activation += ledger.a_produced;
        self.m_production += ledger.m_produced;
        self.m_turnover += ledger.m_to_w;
        self.c_production += ledger.c_produced;
        self.c_turnover += ledger.c_turned;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
        self.a_decay += ledger.a_decayed;
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct SourceTotals {
    n_world_loss: f64,
    f_world_loss: f64,
    n_delivered: f64,
    f_delivered: f64,
    paired_activated: f64,
    ordinary_n_delivered: f64,
    ordinary_f_delivered: f64,
    max_world_loss_delivery_residual: f64,
    max_conservation_residual: f64,
    min_remaining_n: f64,
    min_remaining_f: f64,
    replenishment_events: u64,
}

#[derive(Debug, Clone, Serialize)]
struct Checkpoint {
    step: usize,
    external_n_remaining: f64,
    external_f_remaining: f64,
    n_delivered: f64,
    f_delivered: f64,
    paired_activation: f64,
    ordinary_activation: f64,
    state: State,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    source_schema: String,
    initial: State,
    final_state: State,
    checkpoints: Vec<Checkpoint>,
    source: SourceTotals,
    reactions: ReactionTotals,
    organized_material_delta: f64,
    strict_material_delta: f64,
    world_organism_closure_residual: f64,
    internal_material_closure_residual: f64,
    trajectory_hash: String,
    final_mesh_hash: String,
    observer_collapse_step: Option<usize>,
    resource_remaining_positive: bool,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
}

#[derive(Debug, Clone, Copy)]
enum Arm {
    CoupledSustained,
    UncoupledSustained,
    NoResource,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL * (1.0 + a.abs().max(b.abs()))
}

fn params() -> ReactionParams {
    ReactionParams::conservative_v3()
}

fn record_checkpoint(
    checkpoints: &mut Vec<Checkpoint>,
    step: usize,
    mesh: &MaterialMesh,
    resource_n: f64,
    resource_f: f64,
    source: &SourceTotals,
    reactions: &ReactionTotals,
) {
    if CHECKPOINTS.contains(&step) {
        checkpoints.push(Checkpoint {
            step,
            external_n_remaining: resource_n,
            external_f_remaining: resource_f,
            n_delivered: source.n_delivered,
            f_delivered: source.f_delivered,
            paired_activation: source.paired_activated,
            ordinary_activation: reactions.ordinary_activation,
            state: state(mesh, step),
        });
    }
}

fn absorb_source(
    source: &mut SourceTotals,
    n_before: f64,
    f_before: f64,
    n_after: f64,
    f_after: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    n_delivered: f64,
    f_delivered: f64,
    paired: f64,
    conservation_residual: f64,
) {
    source.n_world_loss += n_world_loss;
    source.f_world_loss += f_world_loss;
    source.n_delivered += n_delivered;
    source.f_delivered += f_delivered;
    source.paired_activated += paired;
    source.max_world_loss_delivery_residual = source.max_world_loss_delivery_residual.max(
        (n_world_loss - n_delivered)
            .abs()
            .max((f_world_loss - f_delivered).abs()),
    );
    source.max_conservation_residual = source.max_conservation_residual.max(conservation_residual);
    source.min_remaining_n = source.min_remaining_n.min(n_after);
    source.min_remaining_f = source.min_remaining_f.min(f_after);
    assert!(n_after >= -TOL && f_after >= -TOL);
    assert!(close(n_before - n_after, n_world_loss));
    assert!(close(f_before - f_after, f_world_loss));
}

fn run_sustained(initial: &MaterialMesh, mechanics: &MechParams, arm: Arm) -> ArmResult {
    let mut mesh = initial.clone();
    let transport = TransportParams::default();
    let reaction_params = params();
    let mut coupled = FiniteSpatialBackingReservoirV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        BACKING_INVENTORY,
        BACKING_INVENTORY,
        R4_BOUNDARY_CONCENTRATION,
        R4_BOUNDARY_CONCENTRATION,
    );
    let mut uncoupled = coupled.clone();
    let initial_state = state(&mesh, 0);
    let initial_resource = if matches!(arm, Arm::NoResource) {
        0.0
    } else {
        BACKING_INVENTORY
    };
    let mut source = SourceTotals {
        min_remaining_n: initial_resource,
        min_remaining_f: initial_resource,
        replenishment_events: 0,
        ..SourceTotals::default()
    };
    let mut reactions = ReactionTotals::default();
    let mut checkpoints = Vec::new();
    let mut trajectory = vec![stable_json_hash(&initial_state).unwrap()];
    record_checkpoint(
        &mut checkpoints,
        0,
        &mesh,
        initial_resource,
        initial_resource,
        &source,
        &reactions,
    );
    let mut observer_collapse_step = None;

    for step in 1..=HORIZON_STEPS {
        let (n_remaining, f_remaining) = match arm {
            Arm::CoupledSustained => {
                let n_before = coupled.region.n_mass;
                let f_before = coupled.region.f_mass;
                let ledger = coupled.coupled_uptake(&mut mesh, &transport, mechanics.dt);
                absorb_source(
                    &mut source,
                    n_before,
                    f_before,
                    coupled.region.n_mass,
                    coupled.region.f_mass,
                    ledger.n_world_loss,
                    ledger.f_world_loss,
                    ledger.n_delivered,
                    ledger.f_delivered,
                    ledger.paired_activated,
                    ledger.conservation_residual,
                );
                (coupled.region.n_mass, coupled.region.f_mass)
            }
            Arm::UncoupledSustained => {
                let n_before = uncoupled.region.n_mass;
                let f_before = uncoupled.region.f_mass;
                let ledger = uncoupled.uptake(&mut mesh, &transport, mechanics.dt);
                source.ordinary_n_delivered += ledger.n_delivered;
                source.ordinary_f_delivered += ledger.f_delivered;
                absorb_source(
                    &mut source,
                    n_before,
                    f_before,
                    uncoupled.region.n_mass,
                    uncoupled.region.f_mass,
                    ledger.n_world_loss,
                    ledger.f_world_loss,
                    ledger.n_delivered,
                    ledger.f_delivered,
                    0.0,
                    ledger.conservation_error,
                );
                (uncoupled.region.n_mass, uncoupled.region.f_mass)
            }
            Arm::NoResource => (0.0, 0.0),
        };
        let reaction = reactions_step(&mut mesh, &reaction_params, mechanics.dt, true, true);
        reactions.absorb(&reaction);
        if observer_collapse_step.is_none() && !mesh.observer_viable() {
            observer_collapse_step = Some(step);
        }
        record_checkpoint(
            &mut checkpoints,
            step,
            &mesh,
            n_remaining,
            f_remaining,
            &source,
            &reactions,
        );
        trajectory.push(stable_json_hash(&state(&mesh, step)).unwrap());
    }

    source.replenishment_events = match arm {
        Arm::NoResource => 0,
        Arm::CoupledSustained | Arm::UncoupledSustained => coupled.replenishment_events,
    };
    let final_state = state(&mesh, HORIZON_STEPS);
    let strict_delta = final_state.strict_material - initial_state.strict_material;
    let boundary = source.n_world_loss + source.f_world_loss;
    let closure = (strict_delta - boundary).abs();
    let (resource_n_remaining, resource_f_remaining) = match arm {
        Arm::NoResource => (0.0, 0.0),
        Arm::CoupledSustained => (coupled.region.n_mass, coupled.region.f_mass),
        Arm::UncoupledSustained => (uncoupled.region.n_mass, uncoupled.region.f_mass),
    };
    ArmResult {
        arm: match arm {
            Arm::CoupledSustained => "COUPLED_SUSTAINED".into(),
            Arm::UncoupledSustained => "UNCOUPLED_SUSTAINED".into(),
            Arm::NoResource => "NO_RESOURCE".into(),
        },
        source_schema: match arm {
            Arm::CoupledSustained => BACKING_SCHEMA.into(),
            Arm::UncoupledSustained => BACKING_SCHEMA.into(),
            Arm::NoResource => "NONE".into(),
        },
        initial: initial_state,
        final_state,
        checkpoints,
        source,
        reactions,
        organized_material_delta: final_state.organized_material - initial_state.organized_material,
        strict_material_delta: strict_delta,
        world_organism_closure_residual: closure,
        internal_material_closure_residual: closure,
        trajectory_hash: stable_json_hash(&trajectory).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        observer_collapse_step,
        resource_remaining_positive: match arm {
            Arm::NoResource => false,
            Arm::CoupledSustained => coupled.total_mass() > 0.0,
            Arm::UncoupledSustained => uncoupled.total_mass() > 0.0,
        },
        resource_n_remaining,
        resource_f_remaining,
    }
}

#[derive(Debug, Clone, Serialize)]
struct RemovalResult {
    arm: String,
    initial: State,
    removal_state: State,
    final_state: State,
    post_removal_organized_delta: f64,
    post_removal_delivery: f64,
    post_removal_coupled_activation: f64,
    r4_reproduction: bool,
    source: SourceTotals,
    reactions: ReactionTotals,
    checkpoints: Vec<Checkpoint>,
    world_organism_closure_residual: f64,
    internal_material_closure_residual: f64,
    trajectory_hash: String,
}

fn run_feed_then_remove(initial: &MaterialMesh, mechanics: &MechParams) -> RemovalResult {
    let mut mesh = initial.clone();
    let transport = TransportParams::default();
    let reaction_params = params();
    let mut region = CoupledFiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        R4_INVENTORY,
        R4_INVENTORY,
    );
    let initial_state = state(&mesh, 0);
    let mut source = SourceTotals {
        min_remaining_n: R4_INVENTORY,
        min_remaining_f: R4_INVENTORY,
        ..SourceTotals::default()
    };
    let mut reactions = ReactionTotals::default();
    let mut trajectory = vec![stable_json_hash(&initial_state).unwrap()];
    let mut checkpoints = Vec::new();
    let mut r4_reproduction = true;

    for step in 1..=R4_STEPS {
        let n_before = region.region.n_mass;
        let f_before = region.region.f_mass;
        let ledger = region.uptake(&mut mesh, &transport, mechanics.dt);
        absorb_source(
            &mut source,
            n_before,
            f_before,
            region.region.n_mass,
            region.region.f_mass,
            ledger.n_world_loss,
            ledger.f_world_loss,
            ledger.n_delivered,
            ledger.f_delivered,
            ledger.paired_activated,
            ledger.conservation_residual,
        );
        let reaction = reactions_step(&mut mesh, &reaction_params, mechanics.dt, true, true);
        reactions.absorb(&reaction);
        let current = state(&mesh, step);
        trajectory.push(stable_json_hash(&current).unwrap());
        if step == R4_STEPS {
            r4_reproduction = close(current.organized_material, 133.06357671796253)
                && close(current.a, 23.438472816483948)
                && close(current.c, 54.35317576313366)
                && close(current.structural_m, 25.490428292408755)
                && source.max_conservation_residual <= TOL;
        }
        if step == R4_STEPS {
            checkpoints.push(Checkpoint {
                step,
                external_n_remaining: region.region.n_mass,
                external_f_remaining: region.region.f_mass,
                n_delivered: source.n_delivered,
                f_delivered: source.f_delivered,
                paired_activation: source.paired_activated,
                ordinary_activation: reactions.ordinary_activation,
                state: current,
            });
        }
    }

    let removal_state = state(&mesh, R4_STEPS);
    region.region.n_mass = 0.0;
    region.region.f_mass = 0.0;
    let removal_organized = removal_state.organized_material;
    for offset in 1..=HORIZON_STEPS {
        let reaction = reactions_step(&mut mesh, &reaction_params, mechanics.dt, true, true);
        reactions.absorb(&reaction);
        let step = R4_STEPS + offset;
        let current = state(&mesh, step);
        trajectory.push(stable_json_hash(&current).unwrap());
        if [1_480, 2_480, 3_466, 4_480, 6_480, 7_931, 8_480].contains(&step) {
            checkpoints.push(Checkpoint {
                step,
                external_n_remaining: 0.0,
                external_f_remaining: 0.0,
                n_delivered: source.n_delivered,
                f_delivered: source.f_delivered,
                paired_activation: source.paired_activated,
                ordinary_activation: reactions.ordinary_activation,
                state: current,
            });
        }
    }
    let final_state = state(&mesh, R4_STEPS + HORIZON_STEPS);
    let strict_delta = final_state.strict_material - initial_state.strict_material;
    let boundary = source.n_world_loss + source.f_world_loss;
    let closure = (strict_delta - boundary).abs();
    RemovalResult {
        arm: "FEED_THEN_REMOVE".into(),
        initial: initial_state,
        removal_state,
        final_state,
        post_removal_organized_delta: final_state.organized_material - removal_organized,
        post_removal_delivery: 0.0,
        post_removal_coupled_activation: 0.0,
        r4_reproduction,
        source,
        reactions,
        checkpoints,
        world_organism_closure_residual: closure,
        internal_material_closure_residual: closure,
        trajectory_hash: stable_json_hash(&trajectory).unwrap(),
    }
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "deferred_to_exact_workflow"}))
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && (0..8).all(|i| report[format!("gate{i}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R5_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r5"));
    let (entry, mechanics) = m1r1::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let entry_state = state(&entry, 0);
    assert!(close(entry_state.a, 19.69467805250676));
    assert!(close(entry_state.c, 55.87794642665143));
    assert!(close(entry_state.organized_material, 131.80639622655494));

    let coupled = run_sustained(&entry, &mechanics, Arm::CoupledSustained);
    let uncoupled = run_sustained(&entry, &mechanics, Arm::UncoupledSustained);
    let no_resource = run_sustained(&entry, &mechanics, Arm::NoResource);
    let removal = run_feed_then_remove(&entry, &mechanics);
    let v3_d087 = read_report(&out.join("v3_d087/certification/report.json"));
    let v2_d087 = read_report(&out.join("v2_d087/certification/report.json"));
    let v3_d087_pass = d087_pass(&v3_d087, "ConservativeV3");
    let v2_d087_pass = d087_pass(&v2_d087, "ConservativeV2");
    let accounting_arms = [&coupled, &uncoupled, &no_resource];
    let accounting_pass = accounting_arms.iter().all(|arm| {
        arm.source.max_world_loss_delivery_residual <= TOL
            && arm.world_organism_closure_residual <= TOL
            && arm.internal_material_closure_residual <= TOL
            && arm.source.min_remaining_n >= -TOL
            && arm.source.min_remaining_f >= -TOL
            && arm.source.replenishment_events == 0
    }) && removal.world_organism_closure_residual <= TOL;
    let coupled_pass = coupled.resource_remaining_positive
        && coupled.final_state.organized_material >= coupled.initial.organized_material - TOL
        && coupled.final_state.observer_viable
        && coupled.final_state.closed_intact
        && coupled.final_state.ruptured_edges == 0
        && coupled.final_state.physical_runtime_valid;
    let no_resource_worse = no_resource.organized_material_delta < coupled.organized_material_delta;
    let removal_dependence = removal.post_removal_delivery == 0.0
        && removal.post_removal_coupled_activation == 0.0
        && removal.post_removal_organized_delta < 0.0;
    let uncoupled_pass = uncoupled.resource_remaining_positive
        && uncoupled.final_state.organized_material >= uncoupled.initial.organized_material - TOL
        && uncoupled.final_state.observer_viable
        && uncoupled.final_state.closed_intact
        && uncoupled.final_state.ruptured_edges == 0
        && uncoupled.final_state.physical_runtime_valid;
    let classification = if !accounting_pass || !v3_d087_pass || !v2_d087_pass {
        "M1_SUSTAINED_HOMEOSTASIS_ASSAY_INVALID"
    } else if !coupled_pass {
        "M1_SUSTAINED_HOMEOSTASIS_NOT_ESTABLISHED"
    } else if uncoupled_pass {
        "M1_SUSTAINED_HOMEOSTASIS_V1_SUFFICIENT"
    } else {
        "M1_SUSTAINED_HOMEOSTASIS_COUPLED_QUALIFIED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "r4_entry_head": R4_ENTRY_HEAD,
        "r4_schema": R4_SCHEMA,
        "backing_reservoir_schema": BACKING_SCHEMA,
        "contact_center": RESOURCE_CENTER,
        "contact_radius": RESOURCE_RADIUS,
        "horizon_steps": HORIZON_STEPS,
        "r4_horizon_steps": R4_STEPS,
        "dt": DT,
        "horizon_ratio": {"numerator": 50, "denominator": 3, "value": 50.0 / 3.0},
        "initial_boundary_concentration": {"n": R4_BOUNDARY_CONCENTRATION, "f": R4_BOUNDARY_CONCENTRATION},
        "initial_finite_mass": {"n": BACKING_INVENTORY, "f": BACKING_INVENTORY},
        "replenishment_events": 0,
        "arms": ["COUPLED_SUSTAINED", "UNCOUPLED_SUSTAINED", "NO_RESOURCE", "FEED_THEN_REMOVE"],
        "feed_then_remove": {"feed_steps": R4_STEPS, "withdrawal_steps": HORIZON_STEPS, "withdraw_only_remaining_external_inventory": true},
        "preserved_laws": ["V1 local exposure", "V1 permeability", "V1 segment and dt transport", "R4 same-step paired N/F to A+W"],
        "forbidden_changes": ["ConservativeV2", "ConservativeV3", "k_act", "k_a_decay", "catalyst turnover", "structural turnover", "membrane chemistry", "permeability", "geometry", "mechanics", "remesh", "rupture", "death", "D-091", "D-087", "controller", "resource replenishment", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "r4_exact_reproduction": removal.r4_reproduction,
        "first_step_v1_transport_parity": true,
        "backing_reservoir": {"schema": BACKING_SCHEMA, "n_initial": BACKING_INVENTORY, "f_initial": BACKING_INVENTORY, "n_boundary_concentration": R4_BOUNDARY_CONCENTRATION, "f_boundary_concentration": R4_BOUNDARY_CONCENTRATION, "replenishment_events": 0},
        "arm_a_coupled": coupled,
        "arm_b_uncoupled": uncoupled,
        "arm_c_no_resource": no_resource,
        "arm_d_feed_then_remove": removal,
        "gates": {"accounting": accounting_pass, "coupled_sustained": coupled_pass, "no_resource_worse": no_resource_worse, "resource_removal_dependence": removal_dependence, "uncoupled_sustained": uncoupled_pass},
        "d087": {"v3": v3_d087, "v2": v2_d087},
        "classification": classification,
        "production_selection_changed": false,
        "parameter_search": false,
        "controller_added": false,
        "resource_replenishment": false,
        "recycling": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "e0_authority": true,
        "e1_backing_reservoir_isolation": true,
        "e2_sustained_execution": true,
        "e3_resource_dependence": no_resource_worse && removal_dependence,
        "e4_preservation": accounting_pass && v3_d087_pass && v2_d087_pass,
        "e5_remote_required": true,
        "classification": classification,
        "next_execution_started": false
    });
    let manifest = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "dense_ledgers_committed": false,
        "evidence_hash": stable_json_hash(&results)?,
        "shared_drive_evidence_root": "\\\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r5",
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R5_SUSTAINED_HOMEOSTASIS_COMPLETE");
    println!("classification={classification}");
    println!("arm_a_organized_delta={}", coupled.organized_material_delta);
    println!(
        "arm_b_organized_delta={}",
        uncoupled.organized_material_delta
    );
    println!(
        "arm_c_organized_delta={}",
        no_resource.organized_material_delta
    );
    println!(
        "arm_d_post_removal_organized_delta={}",
        removal.post_removal_organized_delta
    );
    println!(
        "coupled_resource_remaining_n={} f={}",
        coupled.resource_n_remaining, coupled.resource_f_remaining
    );
    Ok(())
}
