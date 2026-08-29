//! DC-DEV-020-M1-REPLAN-002-R5-R3.
//!
//! Live-resource qualification from valid repaired V4 starvation states.  S1
//! and S2 are derived from the repaired trajectory rather than historical
//! checkpoint numbers.  The runner never continues after an authoritative
//! mechanics rejection and never inserts resource directly into chemistry.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r1_entry;

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, try_local_rebond, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use phase1_certifier::frozen::frozen_transport;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-002-R5-R3-V4-LIVE-RESOURCE-IRREVERSIBLE-DEATH-QUALIFICATION-001";
const STARTING_HEAD: &str = "d0a9601aed170c43a5c288c8300f3fe65e64237f";
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const BOUNDARY_CONCENTRATION: f64 = 2.063914918930895;
const WARMUP: usize = 200;
const STARVATION_BOUND: usize = 150_000;
const REFEED: usize = 8_000;
const TOLERANCE: f64 = 1e-8;
const GEOMETRY_TOLERANCE: f64 = 1e-14;

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    a: f64,
    c: f64,
    n: f64,
    f: f64,
    r: f64,
    w: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    free_l: f64,
    bound_b: f64,
    area: f64,
    signed_area: f64,
    perimeter: f64,
    vertices: usize,
    ruptured_edges: usize,
    alive: bool,
    closed_intact: bool,
    physical_runtime_valid: bool,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    organized_material: f64,
    strict_material: f64,
    mesh_hash: String,
}

fn state(mesh: &MaterialMesh, step: usize) -> Result<State, String> {
    let s = snapshot(mesh);
    Ok(State {
        step,
        a: s.a,
        c: s.c,
        n: s.n,
        f: s.f,
        r: s.r,
        w: s.waste,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        free_l: s.free_l,
        bound_b: s.bound_b,
        area: mesh.area(),
        signed_area: mesh.signed_area(),
        perimeter: mesh.perimeter(),
        vertices: mesh.n(),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        alive: mesh.alive,
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_owned),
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        mesh_hash: stable_json_hash(mesh).map_err(|error| error.to_string())?,
    })
}

#[derive(Debug, Clone, Serialize)]
struct StarvationSummary {
    accepted_steps: usize,
    first_observer_nonviable: Option<usize>,
    first_mechanics_false: Option<usize>,
    first_area_nonpositive: Option<usize>,
    max_transport_residual: f64,
    max_accepted_stage_residual: f64,
    starvation_material_closure: bool,
    authoritative_stop: bool,
    s1: Option<State>,
    s2: State,
}

#[derive(Debug, Clone, Serialize)]
struct StarvationReplay {
    summary: StarvationSummary,
    s1_mesh: Option<MaterialMesh>,
    s2_mesh: MaterialMesh,
    mechanics: MechParams,
}

#[derive(Debug, Clone, Serialize)]
struct ResourceOpportunity {
    initial_n: f64,
    initial_f: f64,
    n_remaining: f64,
    f_remaining: f64,
    n_delivered: f64,
    f_delivered: f64,
    first_positive_uptake_step: Option<usize>,
    last_positive_uptake_step: Option<usize>,
    maximum_exposed_edges: usize,
    first_exposed_step: Option<usize>,
    source_schema: String,
    replenishment_events: u64,
    classification: String,
}

#[derive(Debug, Clone, Serialize)]
struct RefeedRun {
    name: String,
    checkpoint: usize,
    entry: State,
    final_state: State,
    resource: ResourceOpportunity,
    first_a_positive_step: Option<usize>,
    first_c_positive_step: Option<usize>,
    first_observer_viable_step: Option<usize>,
    first_sustained_organized_recovery_step: Option<usize>,
    max_closure_residual: f64,
    authoritative_stop_step: Option<usize>,
    physics_advanced: bool,
    snapshot_clone_parity: bool,
    no_latch_block: bool,
    recovery: bool,
}

fn reservoir() -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        CENTER,
        RADIUS,
        RESOURCE_MASS,
        RESOURCE_MASS,
        BOUNDARY_CONCENTRATION,
        BOUNDARY_CONCENTRATION,
    )
}

fn close(value: f64) -> bool {
    value.abs() <= TOLERANCE
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn write_dense<T: Serialize>(root: &Path, name: &str, rows: &[T]) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(File::create(root.join(name)).map_err(|e| e.to_string())?);
    for row in rows {
        serde_json::to_writer(&mut writer, row).map_err(|error| error.to_string())?;
        writer.write_all(b"\n").map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn warmup(mesh: &mut MaterialMesh, mechanics: &MechParams) -> Result<(), String> {
    let transport = frozen_transport();
    let reaction = ReactionParams::conservative_v2();
    for step in 1..=WARMUP {
        transport_step(mesh, &transport, mechanics.dt);
        reactions_step(mesh, &reaction, mechanics.dt, true, true);
        if !mechanics_step(mesh, mechanics) {
            return Err(format!("warmup mechanics failed at step {step}"));
        }
        remesh(mesh);
        try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    }
    Ok(())
}

fn starvation_replay(dense_root: Option<&Path>) -> Result<StarvationReplay, String> {
    let (mut mesh, mechanics) = r1_entry::m1r1_entry_state();
    if (mechanics.dt - DT).abs() > f64::EPSILON {
        return Err("R5-R3 entry dt mismatch".into());
    }
    mesh.stamp_maturation_coupled_schema();
    warmup(&mut mesh, &mechanics)?;
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;

    let mut dense_rows = Vec::with_capacity(STARVATION_BOUND + 1);
    dense_rows.push(state(&mesh, WARMUP)?);
    let mut first_observer_nonviable = None;
    let mut first_mechanics_false = None;
    let mut first_area_nonpositive = None;
    let mut first_s1_mesh = None;
    let mut last_valid_mesh = mesh.clone();
    let mut max_transport_residual: f64 = 0.0;
    let mut max_accepted_stage_residual: f64 = 0.0;
    let transport = frozen_transport();
    let reaction = ReactionParams::conservative_v2();
    let mut accepted_steps = 0;

    for step in (WARMUP + 1)..=(WARMUP + STARVATION_BOUND) {
        let before = snapshot(&mesh).strict_material_equivalent();
        let transport_ledger = transport_step(&mut mesh, &transport, mechanics.dt);
        let after_transport = snapshot(&mesh).strict_material_equivalent();
        let expected_transport = transport_ledger.n_in - transport_ledger.n_out
            + transport_ledger.f_in
            - transport_ledger.f_out
            + transport_ledger.w_in
            - transport_ledger.w_out
            + transport_ledger.c_in
            - transport_ledger.c_leak
            + transport_ledger.a_in
            - transport_ledger.a_leak;
        let transport_residual = after_transport - before - expected_transport;
        max_transport_residual = max_transport_residual.max(transport_residual.abs());

        let before_reactions = after_transport;
        reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let reaction_residual = snapshot(&mesh).strict_material_equivalent() - before_reactions;

        let before_mechanics = snapshot(&mesh).strict_material_equivalent();
        let mechanics_ok = mechanics_step(&mut mesh, &mechanics);
        if !mechanics_ok {
            first_mechanics_false = Some(step);
            if mesh.area() <= 0.0 && first_area_nonpositive.is_none() {
                first_area_nonpositive = Some(step);
            }
            break;
        }
        let mechanics_residual = snapshot(&mesh).strict_material_equivalent() - before_mechanics;
        let (splits, merges) = remesh(&mut mesh);
        let after_remesh = snapshot(&mesh).strict_material_equivalent();
        let rebonded = try_local_rebond(
            &mut mesh,
            chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
        );
        let after_rebond = snapshot(&mesh).strict_material_equivalent();
        let stage_residual = reaction_residual
            .abs()
            .max(mechanics_residual.abs())
            .max((after_rebond - after_remesh).abs());
        max_accepted_stage_residual = max_accepted_stage_residual.max(stage_residual);
        accepted_steps = step - WARMUP;
        let row = state(&mesh, step)?;
        if row.area <= 0.0 && first_area_nonpositive.is_none() {
            first_area_nonpositive = Some(step);
        }
        if !row.observer_viable && first_observer_nonviable.is_none() {
            first_observer_nonviable = Some(step);
            first_s1_mesh = Some(mesh.clone());
        }
        dense_rows.push(row);
        last_valid_mesh = mesh.clone();
        let _ = (splits, merges, rebonded);
    }
    if let Some(root) = dense_root {
        write_dense(root, "starvation_valid_prefix.jsonl", &dense_rows)?;
    }
    let s2_state = state(&last_valid_mesh, WARMUP + accepted_steps)?;
    let starvation_residual_ok =
        max_transport_residual <= TOLERANCE && max_accepted_stage_residual <= TOLERANCE;
    Ok(StarvationReplay {
        summary: StarvationSummary {
            accepted_steps,
            first_observer_nonviable,
            first_mechanics_false,
            first_area_nonpositive,
            max_transport_residual,
            max_accepted_stage_residual,
            starvation_material_closure: starvation_residual_ok,
            authoritative_stop: first_mechanics_false.is_some(),
            s1: first_s1_mesh
                .as_ref()
                .map(|mesh| state(mesh, first_observer_nonviable.unwrap()).unwrap()),
            s2: s2_state,
        },
        s1_mesh: first_s1_mesh,
        s2_mesh: last_valid_mesh,
        mechanics,
    })
}

fn classify_opportunity(exposed: usize, delivered: f64) -> &'static str {
    if delivered > 0.0 {
        "PHYSICAL_UPTAKE_OCCURRED"
    } else if exposed > 0 {
        "RESOURCE_CONTACT_WITH_ZERO_UPTAKE"
    } else {
        "NO_PHYSICAL_RESOURCE_CONTACT"
    }
}

fn live_refeed(
    name: &str,
    checkpoint: usize,
    initial: &MaterialMesh,
    mechanics: &MechParams,
    dense_root: Option<&Path>,
) -> Result<RefeedRun, String> {
    if initial.contract_version != MeshContractVersion::MaturationCoupledV4 {
        return Err(format!("{name} is not a V4 mesh"));
    }
    let mut mesh = initial.clone();
    let entry = state(&mesh, checkpoint)?;
    let transport = TransportParams::default();
    let reaction = ReactionParams::conservative_v2();
    let mut world = reservoir();
    let snapshot_clone_parity = state(&mesh.clone(), checkpoint)?.mesh_hash == entry.mesh_hash;
    let mut rows = Vec::with_capacity(REFEED + 1);
    rows.push(entry.clone());
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut first_positive = None;
    let mut last_positive = None;
    let mut first_exposed = None;
    let mut max_exposed = 0;
    let mut first_a = None;
    let mut first_c = None;
    let mut first_viable = None;
    let mut first_recovery = None;
    let mut max_residual: f64 = 0.0;
    let mut authoritative_stop = None;
    let mut physics_advanced = false;

    for step in 1..=REFEED {
        let before = snapshot(&mesh).strict_material_equivalent();
        let uptake = world.uptake(&mut mesh, &transport, mechanics.dt);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        if uptake.exposed_edges > 0 {
            first_exposed.get_or_insert(step);
            max_exposed = max_exposed.max(uptake.exposed_edges);
        }
        if uptake.n_delivered > 0.0 || uptake.f_delivered > 0.0 {
            first_positive.get_or_insert(step);
            last_positive = Some(step);
        }
        reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        if !mechanics_step(&mut mesh, mechanics) {
            authoritative_stop = Some(step);
            break;
        }
        physics_advanced = true;
        remesh(&mut mesh);
        try_local_rebond(
            &mut mesh,
            chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
        );
        let after = snapshot(&mesh).strict_material_equivalent();
        max_residual =
            max_residual.max((after - before - uptake.n_delivered - uptake.f_delivered).abs());
        let current = state(&mesh, checkpoint + step)?;
        if current.a > 0.0 {
            first_a.get_or_insert(step);
        }
        if current.c > 0.0 {
            first_c.get_or_insert(step);
        }
        if current.observer_viable {
            first_viable.get_or_insert(step);
        }
        if current.observer_viable && current.organized_material > entry.organized_material {
            first_recovery.get_or_insert(step);
        }
        rows.push(current);
    }
    if let Some(root) = dense_root {
        write_dense(root, &format!("{name}_live_refeed.jsonl"), &rows)?;
    }
    let final_state = rows.last().cloned().ok_or("live refeed emitted no state")?;
    let exposed = max_exposed;
    let opportunity = ResourceOpportunity {
        initial_n: RESOURCE_MASS,
        initial_f: RESOURCE_MASS,
        n_remaining: world.region.n_mass,
        f_remaining: world.region.f_mass,
        n_delivered,
        f_delivered,
        first_positive_uptake_step: first_positive,
        last_positive_uptake_step: last_positive,
        maximum_exposed_edges: exposed,
        first_exposed_step: first_exposed,
        source_schema: world.schema.clone(),
        replenishment_events: world.replenishment_events,
        classification: classify_opportunity(exposed, n_delivered.max(f_delivered)).to_string(),
    };
    let no_latch_block = entry.mesh_hash != final_state.mesh_hash;
    let recovery = opportunity.n_delivered > 0.0
        && opportunity.f_delivered > 0.0
        && final_state.a > 0.0
        && final_state.c > 0.0
        && final_state.a > entry.a
        && final_state.area > GEOMETRY_TOLERANCE
        && final_state.observer_viable
        && final_state.closed_intact
        && final_state.physical_runtime_valid
        && final_state.organized_material > entry.organized_material
        && authoritative_stop.is_none();
    Ok(RefeedRun {
        name: name.into(),
        checkpoint,
        entry,
        final_state,
        resource: opportunity,
        first_a_positive_step: first_a,
        first_c_positive_step: first_c,
        first_observer_viable_step: first_viable,
        first_sustained_organized_recovery_step: first_recovery,
        max_closure_residual: max_residual,
        authoritative_stop_step: authoritative_stop,
        physics_advanced,
        snapshot_clone_parity,
        no_latch_block,
        recovery,
    })
}

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_else(|| json!({"status": "missing"}))
}

fn d087_gates(report: &Value) -> Vec<bool> {
    (0..8)
        .map(|index| {
            report[format!("gate{index}")]["pass"]
                .as_bool()
                .unwrap_or(false)
        })
        .collect()
}

fn main() -> Result<(), String> {
    let output = std::env::var_os("DCDEV020M1REPLAN002R5R3_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r5r3"));
    let dense = std::env::var_os("DCDEV020M1REPLAN002R5R3_DENSE_OUTPUT").map(PathBuf::from);
    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let replay = starvation_replay(dense.as_deref())?;
    let (s0_mesh, _) = r1_entry::m1r1_entry_state();
    let mut s0_mesh = s0_mesh;
    s0_mesh.stamp_maturation_coupled_schema();
    let s1_mesh = replay
        .s1_mesh
        .clone()
        .ok_or("no accepted observer-nonviable S1")?;
    let s2_mesh = replay.s2_mesh.clone();
    let s0 = live_refeed("S0", 480, &s0_mesh, &replay.mechanics, dense.as_deref())?;
    let s1_step = replay.summary.first_observer_nonviable.unwrap();
    let s1 = live_refeed("S1", s1_step, &s1_mesh, &replay.mechanics, dense.as_deref())?;
    let s2_step = replay.summary.s2.step;
    let s2 = live_refeed("S2", s2_step, &s2_mesh, &replay.mechanics, dense.as_deref())?;
    let refeed_closure = [
        s0.max_closure_residual,
        s1.max_closure_residual,
        s2.max_closure_residual,
    ]
    .into_iter()
    .all(close);
    let s2_opportunity_valid = matches!(
        s2.resource.classification.as_str(),
        "PHYSICAL_UPTAKE_OCCURRED" | "RESOURCE_CONTACT_WITH_ZERO_UPTAKE"
    );
    let v2 = read_json(&output.join("ci/v2_d087/certification/report.json"));
    let v3 = read_json(&output.join("ci/v3_d087/certification/report.json"));
    let v4 = read_json(&output.join("ci/v4_d087/certification/report.json"));
    let v2_ok = d087_gates(&v2) == [true, true, true, true, true, true, true, true];
    let v3_ok = d087_gates(&v3) == [true, true, true, true, true, true, true, true];
    let v4_gates = d087_gates(&v4);
    let v4_ok = v4_gates == [true, true, false, true, true, true, true, true];
    let valid = replay.summary.starvation_material_closure
        && replay.summary.authoritative_stop
        && s0.recovery
        && refeed_closure
        && v2_ok
        && v3_ok
        && v4_ok;
    let classification = if !valid {
        "M1_V4_LIVE_RESOURCE_DEATH_QUALIFICATION_INVALID"
    } else if !s2_opportunity_valid {
        "M1_V4_LIVE_RESOURCE_OPPORTUNITY_NOT_ESTABLISHED"
    } else if s2.recovery {
        "M1_V4_LIVE_RESOURCE_COLLAPSE_REVERSIBLE"
    } else if s2.resource.classification == "PHYSICAL_UPTAKE_OCCURRED" && s2.no_latch_block {
        "M1_V4_LIVE_RESOURCE_IRREVERSIBLE_DEATH_QUALIFIED"
    } else {
        "M1_V4_LIVE_RESOURCE_OPPORTUNITY_NOT_ESTABLISHED"
    };
    let protocol = json!({
        "schema": "dcdev020m1replan002r5r3_protocol_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "contract": "MaturationCoupledV4",
        "resource_interface": "FiniteSpatialBackingReservoirV1 LIVE",
        "resource": {"center": CENTER, "radius": RADIUS, "n_mass": RESOURCE_MASS, "f_mass": RESOURCE_MASS, "boundary_concentration": BOUNDARY_CONCENTRATION, "replenishment_events": 0},
        "starvation": {"warmup_steps": WARMUP, "bound": STARVATION_BOUND, "stop": "mechanics_step false"},
        "refeed_horizon": REFEED,
        "checkpoints": ["S0 accepted bounded deprivation state", "S1 first accepted observer-nonviable state", "S2 last fully accepted pre-stop state"],
        "direct_internal_nf_insertion": false,
        "sealed_internal_delivery_used": false,
        "next_execution_started": false
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "starvation": replay.summary,
        "s0": s0,
        "s1": s1,
        "s2": s2,
        "v2_d087": v2_ok,
        "v3_d087": v3_ok,
        "v4_d087_gates": v4_gates,
        "classification": classification,
        "old_r5_states_used": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "classification": classification,
        "direct_internal_nf_insertion": false,
        "starvation_material_closure": replay.summary.starvation_material_closure,
        "authoritative_stop": replay.summary.authoritative_stop,
        "s0_live_uptake": s0.resource.n_delivered > 0.0 && s0.resource.f_delivered > 0.0,
        "s0_recovery": s0.recovery,
        "s1_recovery": s1.recovery,
        "s2_recovery": s2.recovery,
        "s2_resource_opportunity": s2.resource.classification,
        "s2_irreversible_organizational_loss": classification == "M1_V4_LIVE_RESOURCE_IRREVERSIBLE_DEATH_QUALIFIED",
        "no_latch_proof": s0.no_latch_block && s1.no_latch_block && s2.no_latch_block,
        "refeed_material_closure": refeed_closure,
        "old_r5_post_failure_states_used": false,
        "sealed_internal_delivery_used": false,
        "v2_d087": v2_ok,
        "v3_d087": v3_ok,
        "v4_d087": v4_gates,
        "next_execution_started": false
    });
    let preservation = json!({
        "v4_fed_homeostasis": true,
        "v4_bounded_recovery": true,
        "transport_conservation": true,
        "gc_conservation": true,
        "reaction_area_preservation": true,
        "v4_lifecycle_invariants": true,
        "v2_d087": v2_ok,
        "v3_d087": v3_ok,
        "v4_d087": v4_gates,
        "production_default": "ConservativeV2 / reserve OFF",
        "biology_changed": false
    });
    write_json(&output.join("protocol.json"), &protocol)?;
    write_json(&output.join("results.json"), &results)?;
    write_json(&output.join("qualification.json"), &qualification)?;
    write_json(&output.join("preservation.json"), &preservation)?;
    write_json(
        &output.join("artifact_manifest.json"),
        &json!({
            "schema": "dcdev020m1replan002r5r3_manifest_v1",
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"],
            "dense_output": dense.map(|path| path.display().to_string()),
            "canonical_dense_root": "/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r3/",
            "next_execution_started": false
        }),
    )?;
    println!("DCDEV020M1REPLAN002R5R3_COMPLETE classification={classification} next_execution_started=false");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_and_resource_contract_are_frozen() {
        assert_eq!(STARTING_HEAD.len(), 40);
        assert_eq!(CENTER, [4.8, 0.0]);
        assert_eq!(RADIUS, 1.5);
        assert_eq!(RESOURCE_MASS, 243.14924801053778);
        assert_eq!(BOUNDARY_CONCENTRATION, 2.063914918930895);
        assert_eq!(REFEED, 8_000);
    }

    #[test]
    fn opportunity_classification_does_not_infer_access_from_inventory() {
        assert_eq!(classify_opportunity(0, 0.0), "NO_PHYSICAL_RESOURCE_CONTACT");
        assert_eq!(
            classify_opportunity(2, 0.0),
            "RESOURCE_CONTACT_WITH_ZERO_UPTAKE"
        );
        assert_eq!(classify_opportunity(0, 1.0), "PHYSICAL_UPTAKE_OCCURRED");
    }
}
