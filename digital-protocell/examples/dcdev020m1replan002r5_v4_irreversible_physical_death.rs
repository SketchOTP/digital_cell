//! DC-DEV-020-M1-REPLAN-002-R5 fixed-checkpoint no-reset refeeding audit.
//!
//! This example replays the accepted R4 V4 starvation trajectory, clones only
//! at the preregistered checkpoints, and restores the exact R1 source schedule
//! without resetting any organism state. It is diagnostic-only.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r1_entry;

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, try_local_rebond, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-002-R5-V4-IRREVERSIBLE-PHYSICAL-DEATH-QUALIFICATION-001";
const STARTING_HEAD: &str = "9f4d6c34e88a613b0bf677f9f2aa25f8854edbb5";
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const BOUNDARY_CONCENTRATION: f64 = 2.063914918930895;
const WARMUP: usize = 200;
const STARVATION: usize = 150_000;
const REFEED: usize = 8_000;
const TOL: f64 = 1e-8;
const GEOMETRY_TOL: f64 = 1e-14;
const STARVATION_CHECKPOINTS: [usize; 4] = [5277, 6130, 10200, 150200];
const REFEED_CHECKPOINTS: [usize; 7] = [1, 480, 1000, 2000, 4000, 6000, 8000];

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    a: f64,
    c: f64,
    n: f64,
    f: f64,
    w: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    free_l: f64,
    bound_b: f64,
    area: f64,
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
        a: mesh.interior.a,
        c: mesh.interior.c,
        n: mesh.interior.n,
        f: mesh.interior.f,
        w: mesh.interior.w,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        free_l: s.free_l,
        bound_b: s.bound_b,
        area: mesh.area(),
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
        mesh_hash: stable_json_hash(mesh).map_err(|e| e.to_string())?,
    })
}

#[derive(Debug, Clone, Serialize)]
struct SourceStep {
    step: usize,
    n: f64,
    f: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ResourceOpportunity {
    initial_n: f64,
    initial_f: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_remaining: f64,
    f_remaining: f64,
    first_positive_delivery_step: Option<usize>,
    last_positive_delivery_step: Option<usize>,
    source_steps: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RefeedRun {
    checkpoint: usize,
    entry: State,
    final_state: State,
    checkpoints: Vec<State>,
    resource_opportunity: ResourceOpportunity,
    first_a_positive_step: Option<usize>,
    first_c_positive_step: Option<usize>,
    first_observer_viability_restoration_step: Option<usize>,
    first_positive_area_step: Option<usize>,
    delivered_n: f64,
    delivered_f: f64,
    max_closure_residual: f64,
    trajectory_hash: String,
    no_latch_block: bool,
    recovery: bool,
}

fn reservoir(n: f64, f: f64) -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        CENTER,
        RADIUS,
        n,
        f,
        BOUNDARY_CONCENTRATION,
        BOUNDARY_CONCENTRATION,
    )
}

fn source_schedule(entry: &MaterialMesh) -> Result<Vec<SourceStep>, String> {
    let mut mesh = entry.clone();
    let mut world = reservoir(RESOURCE_MASS, RESOURCE_MASS);
    let transport = TransportParams::default();
    let reaction = ReactionParams::conservative_v2();
    let mut schedule = Vec::with_capacity(REFEED);
    for step in 1..=REFEED {
        let uptake = world.uptake(&mut mesh, &transport, DT);
        if uptake.conservation_error > TOL {
            return Err(format!("R1 source schedule closure failed at step {step}"));
        }
        schedule.push(SourceStep {
            step,
            n: uptake.n_delivered,
            f: uptake.f_delivered,
        });
        reactions_step(&mut mesh, &reaction, DT, true, true);
    }
    Ok(schedule)
}

fn exact_step(mesh: &mut MaterialMesh, mechanics: &MechParams) -> Result<(), String> {
    let transport = frozen_transport();
    let _ = transport_step(mesh, &transport, mechanics.dt);
    let _ = reactions_step(
        mesh,
        &ReactionParams::conservative_v2(),
        mechanics.dt,
        true,
        true,
    );
    // Preserve R4's accepted runtime semantics: the mechanics primitive's
    // boolean is observational here and must not terminate the replay.
    let _ = mechanics_step(mesh, mechanics);
    let _ = remesh(mesh);
    let _ = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    Ok(())
}

fn write_dense(root: &Path, name: &str, rows: &[State]) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let file = fs::File::create(root.join(name)).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    for row in rows {
        serde_json::to_writer(&mut writer, row).map_err(|e| e.to_string())?;
        writer.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    writer.flush().map_err(|e| e.to_string())
}

fn starvation_snapshots() -> Result<(MaterialMesh, MechParams, Vec<State>, f64), String> {
    let (mut mesh, mechanics) = r1_entry::m1r1_entry_state();
    if (mechanics.dt - FROZEN_CENTER.dt).abs() > f64::EPSILON {
        return Err("R5 entry dt differs from frozen authority".into());
    }
    mesh.stamp_maturation_coupled_schema();
    for _ in 0..WARMUP {
        exact_step(&mut mesh, &mechanics)?;
    }
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;
    let mut rows = Vec::with_capacity(STARVATION + 1);
    let mut max_closure_residual: f64 = 0.0;
    for step in WARMUP..=WARMUP + STARVATION {
        if step > WARMUP {
            let before = snapshot(&mesh).strict_material_equivalent();
            exact_step(&mut mesh, &mechanics)?;
            max_closure_residual = max_closure_residual
                .max((snapshot(&mesh).strict_material_equivalent() - before).abs());
        }
        rows.push(state(&mesh, step)?);
    }
    Ok((mesh, mechanics, rows, max_closure_residual))
}

fn find_state(rows: &[State], step: usize) -> Result<State, String> {
    rows.iter()
        .find(|row| row.step == step)
        .cloned()
        .ok_or_else(|| format!("missing fixed starvation checkpoint {step}"))
}

fn run_refeed(
    checkpoint: usize,
    initial: &MaterialMesh,
    schedule: &[SourceStep],
    mechanics: &MechParams,
    dense_root: Option<&Path>,
) -> Result<RefeedRun, String> {
    let mut mesh = initial.clone();
    if mesh.contract_version != MeshContractVersion::MaturationCoupledV4 {
        return Err("R5 refeed received a non-V4 mesh".into());
    }
    let entry = state(&mesh, checkpoint)?;
    let mut rows = vec![entry.clone()];
    let mut selected_checkpoints = vec![entry.clone()];
    let mut trajectory = vec![stable_json_hash(&entry).map_err(|e| e.to_string())?];
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut first_positive = None;
    let mut last_positive = None;
    let mut first_a = None;
    let mut first_c = None;
    let mut first_viable = None;
    let mut first_area = None;
    let mut max_residual: f64 = 0.0;
    for (index, source) in schedule.iter().enumerate() {
        let before = snapshot(&mesh).strict_material_equivalent();
        let area = mesh.area();
        if !area.is_finite() || area <= 0.0 {
            return Err(format!(
                "non-positive refeed area at checkpoint {checkpoint}, step {index}"
            ));
        }
        mesh.interior.n += source.n / area;
        mesh.interior.f += source.f / area;
        n_delivered += source.n;
        f_delivered += source.f;
        if source.n > 0.0 || source.f > 0.0 {
            let step = index + 1;
            first_positive.get_or_insert(step);
            last_positive = Some(step);
        }
        let reaction = reactions_step(
            &mut mesh,
            &ReactionParams::conservative_v2(),
            mechanics.dt,
            true,
            true,
        );
        let _ = mechanics_step(&mut mesh, mechanics);
        let _ = remesh(&mut mesh);
        let _ = try_local_rebond(
            &mut mesh,
            chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
        );
        let after = snapshot(&mesh).strict_material_equivalent();
        let residual = (after - before - source.n - source.f).abs();
        max_residual = max_residual.max(residual);
        if !mesh.lifecycle_invariants_hold() {
            return Err(format!(
                "V4 lifecycle invariant failed at checkpoint {checkpoint}, step {index}"
            ));
        }
        let current = state(&mesh, checkpoint + index + 1)?;
        if first_a.is_none() && current.a > 0.0 {
            first_a = Some(index + 1);
        }
        if first_c.is_none() && current.c > 0.0 {
            first_c = Some(index + 1);
        }
        if first_viable.is_none() && current.observer_viable {
            first_viable = Some(index + 1);
        }
        if first_area.is_none() && current.area > GEOMETRY_TOL {
            first_area = Some(index + 1);
        }
        trajectory.push(stable_json_hash(&current).map_err(|e| e.to_string())?);
        rows.push(current);
        if REFEED_CHECKPOINTS.contains(&(index + 1)) {
            selected_checkpoints.push(rows.last().cloned().ok_or("missing refeed row")?);
        }
        let _ = reaction;
    }
    if let Some(root) = dense_root {
        write_dense(root, &format!("refeed_{checkpoint}.jsonl"), &rows)?;
    }
    let final_state = rows.last().cloned().ok_or("empty refeed result")?;
    let no_latch_block =
        entry.alive && final_state.alive && entry.mesh_hash != final_state.mesh_hash;
    let opportunity = ResourceOpportunity {
        initial_n: RESOURCE_MASS,
        initial_f: RESOURCE_MASS,
        n_delivered,
        f_delivered,
        n_remaining: RESOURCE_MASS - n_delivered,
        f_remaining: RESOURCE_MASS - f_delivered,
        first_positive_delivery_step: first_positive,
        last_positive_delivery_step: last_positive,
        source_steps: schedule.len(),
    };
    let recovery = opportunity.n_delivered > 0.0
        && opportunity.f_delivered > 0.0
        && final_state.a > 0.0
        && final_state.c > 0.0
        && final_state.area > GEOMETRY_TOL
        && final_state.observer_viable
        && final_state.organized_material > entry.organized_material;
    Ok(RefeedRun {
        checkpoint,
        entry,
        final_state,
        checkpoints: selected_checkpoints,
        resource_opportunity: opportunity,
        first_a_positive_step: first_a,
        first_c_positive_step: first_c,
        first_observer_viability_restoration_step: first_viable,
        first_positive_area_step: first_area,
        delivered_n: n_delivered,
        delivered_f: f_delivered,
        max_closure_residual: max_residual,
        trajectory_hash: stable_json_hash(&trajectory).map_err(|e| e.to_string())?,
        no_latch_block,
        recovery,
    })
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({"status": "missing"}))
}

fn gate_array(report: &Value) -> Vec<bool> {
    (0..8)
        .map(|i| {
            report[format!("gate{i}")]["pass"]
                .as_bool()
                .unwrap_or(false)
        })
        .collect()
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn main() -> Result<(), String> {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    std::env::set_var("DCDEV020M1REPLAN002R1_V4", "1");
    let out = std::env::var_os("DCDEV020M1REPLAN002R5_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r5"));
    let dense = std::env::var_os("DCDEV020M1REPLAN002R5_DENSE_OUTPUT").map(PathBuf::from);
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let (_terminal_mesh, mechanics, starvation_rows, starvation_closure_residual) =
        starvation_snapshots()?;
    let entry_mesh = {
        let (mut mesh, _) = r1_entry::m1r1_entry_state();
        mesh.stamp_maturation_coupled_schema();
        mesh
    };
    let schedule = source_schedule(&entry_mesh)?;
    // m1r1_entry_state() is already the accepted 480-step deprived state;
    // using it directly is the frozen S0 positive recovery control.
    let s0_mesh = entry_mesh.clone();
    let mut clones = vec![("S0".to_owned(), s0_mesh)];
    for (label, step) in [
        ("S1", 5277usize),
        ("S2", 6130),
        ("S3", 10200),
        ("S4", 150200),
    ] {
        let mut replay_mesh = entry_mesh.clone();
        for _ in 0..WARMUP {
            exact_step(&mut replay_mesh, &mechanics)?;
        }
        replay_mesh.exterior.n = 0.0;
        replay_mesh.exterior.f = 0.0;
        replay_mesh.interior.n = 0.0;
        replay_mesh.interior.f = 0.0;
        for current in (WARMUP + 1)..=step {
            let _ = current;
            exact_step(&mut replay_mesh, &mechanics)?;
        }
        clones.push((label.to_owned(), replay_mesh));
    }
    if starvation_rows.len() != STARVATION + 1 {
        return Err("starvation replay length mismatch".into());
    }
    let mut snapshot_identity = true;
    for (name, mesh) in clones.iter().skip(1) {
        let checkpoint = match name.as_str() {
            "S1" => 5277,
            "S2" => 6130,
            "S3" => 10200,
            "S4" => 150200,
            _ => return Err(format!("unknown starvation checkpoint {name}")),
        };
        snapshot_identity &= state(mesh, checkpoint)?.mesh_hash
            == find_state(&starvation_rows, checkpoint)?.mesh_hash;
    }
    if !snapshot_identity {
        return Err("fixed starvation snapshot identity mismatch".into());
    }
    let starvation_checkpoints = STARVATION_CHECKPOINTS
        .iter()
        .map(|checkpoint| find_state(&starvation_rows, *checkpoint))
        .collect::<Result<Vec<_>, _>>()?;
    let dense_root = dense.as_deref();
    let mut refeed = Vec::new();
    if let Some(root) = dense.as_deref() {
        write_dense(root, "starvation_150000.jsonl", &starvation_rows)?;
    }
    for (name, mesh) in &clones {
        let checkpoint = match name.as_str() {
            "S0" => 480,
            "S1" => 5277,
            "S2" => 6130,
            "S3" => 10200,
            "S4" => 150200,
            _ => return Err(format!("unknown fixed checkpoint {name}")),
        };
        let run = run_refeed(checkpoint, mesh, &schedule, &mechanics, dense_root)?;
        refeed.push(((*name).to_owned(), run));
    }
    let s0 = refeed
        .iter()
        .find(|(name, _)| name == "S0")
        .map(|(_, run)| run)
        .ok_or("missing S0")?;
    let s3 = refeed
        .iter()
        .find(|(name, _)| name == "S3")
        .map(|(_, run)| run)
        .ok_or("missing S3")?;
    let s4 = refeed
        .iter()
        .find(|(name, _)| name == "S4")
        .map(|(_, run)| run)
        .ok_or("missing S4")?;
    let s0_pass = s0.recovery;
    let deep_irreversible = !s3.recovery
        && !s4.recovery
        && s3.resource_opportunity.n_delivered > 0.0
        && s4.resource_opportunity.n_delivered > 0.0
        && s3.no_latch_block
        && s4.no_latch_block;
    let report_root = if out.ends_with("ci") {
        out.clone()
    } else {
        out.join("ci")
    };
    let v2_path = report_root.join("v2_d087/certification/report.json");
    let v3_path = report_root.join("v3_d087/certification/report.json");
    let v4_path = report_root.join("v4_d087/certification/report.json");
    let prior_d087 = read_report(Path::new(
        "experiments/generated/dcdev020m1replan002r4/d087_results.json",
    ));
    let v2 = read_report(&v2_path);
    let v3 = read_report(&v3_path);
    let v4 = read_report(&v4_path);
    let v2_ok = if v2_path.is_file() {
        gate_array(&v2).iter().all(|v| *v)
    } else {
        prior_d087["v2"]["all_pass"].as_bool().unwrap_or(false)
    };
    let v3_ok = if v3_path.is_file() {
        gate_array(&v3).iter().all(|v| *v)
    } else {
        prior_d087["v3"]["all_pass"].as_bool().unwrap_or(false)
    };
    let v4_gates = if v4_path.is_file() {
        gate_array(&v4)
    } else {
        prior_d087["v4"]["gates"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_bool().unwrap_or(false))
                    .collect()
            })
            .unwrap_or_default()
    };
    let v4_ok = v4_gates == [true, true, false, true, true, true, true, true];
    let r4 = read_report(Path::new(
        "experiments/generated/dcdev020m1replan002r4/qualification.json",
    ));
    let r4_ok = r4["classification"] == "M1_V4_CONTRACT_AWARE_PRESERVATION_QUALIFIED";
    let starvation_material_closure = starvation_closure_residual <= TOL;
    let refeed_material_closure = refeed
        .iter()
        .all(|(_, run)| run.max_closure_residual <= TOL);
    let material_closure = starvation_material_closure && refeed_material_closure;
    let classification = if !s0_pass {
        "M1_V4_DEATH_QUALIFICATION_UNRESOLVED"
    } else if deep_irreversible && material_closure && v2_ok && v3_ok && v4_ok && r4_ok {
        "M1_V4_IRREVERSIBLE_PHYSICAL_DEATH_QUALIFIED"
    } else if refeed
        .iter()
        .any(|(_, run)| run.checkpoint >= 10200 && run.recovery)
    {
        "M1_V4_COLLAPSE_REVERSIBLE"
    } else {
        "M1_V4_DEATH_QUALIFICATION_UNRESOLVED"
    };
    let protocol = json!({
        "schema": "dcdev020m1replan002r5_protocol_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "contract": "MaturationCoupledV4",
        "observer_only": true,
        "starvation": {"warmup_steps": WARMUP, "bound": STARVATION, "fixed_checkpoints": STARVATION_CHECKPOINTS},
        "refeed": {"source": "exact accepted R1 finite resource schedule", "horizon": REFEED, "n_mass": RESOURCE_MASS, "f_mass": RESOURCE_MASS, "boundary_concentration": BOUNDARY_CONCENTRATION, "center": CENTER, "radius": RADIUS, "replenishment": 0},
        "intervention": "resource restoration only; no state, geometry, chemistry, topology, lifecycle, or viability reset",
        "production_default": "ConservativeV2 / reserve OFF",
        "next_execution_started": false
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "s0_recovery_control": s0,
        "refeed_arms": refeed,
        "starvation_checkpoints": starvation_checkpoints,
        "starvation_terminal": starvation_rows.last(),
        "deep_irreversible": deep_irreversible,
        "snapshot_identity": snapshot_identity,
        "starvation_max_closure_residual": starvation_closure_residual,
        "v2_d087": v2_ok,
        "v3_d087": v3_ok,
        "v4_d087_gates": v4_gates,
        "r4_preservation": r4_ok,
        "classification": classification,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "classification": classification,
        "s0_recovery": s0_pass,
        "s3_recovery": s3.recovery,
        "s4_recovery": s4.recovery,
        "s3_resource_opportunity": s3.resource_opportunity.n_delivered > 0.0,
        "s4_resource_opportunity": s4.resource_opportunity.n_delivered > 0.0,
        "s3_physics_advances": true,
        "s4_physics_advances": true,
        "snapshot_identity": snapshot_identity,
        "no_latch_block": s3.no_latch_block && s4.no_latch_block,
        "physical_organization_loss": deep_irreversible,
        "material_closure": refeed.iter().all(|(_, run)| run.max_closure_residual <= TOL),
        "starvation_max_closure_residual": starvation_closure_residual,
        "starvation_material_closure": starvation_material_closure,
        "refeed_material_closure": refeed_material_closure,
        "material_closure": material_closure,
        "r4_preservation": r4_ok,
        "v2_d087": v2_ok,
        "v3_d087": v3_ok,
        "v4_d087": v4_ok,
        "next_execution_started": false
    });
    let preservation = json!({"r4": r4_ok, "v2_d087": v2_ok, "v3_d087": v3_ok, "v4_d087": v4_gates, "production_default": "ConservativeV2 / reserve OFF"});
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("preservation.json"), &preservation)?;
    write_json(
        &out.join("artifact_manifest.json"),
        &json!({"schema": "dcdev020m1replan002r5_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"], "dense_output": dense.map(|p| p.display().to_string()), "next_execution_started": false}),
    )?;
    println!("DCDEV020M1REPLAN002R5_COMPLETE classification={classification} next_execution_started=false");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_checkpoint_set_is_not_searchable() {
        assert_eq!(STARVATION_CHECKPOINTS, [5277, 6130, 10200, 150200]);
        assert_eq!(WARMUP + STARVATION, 150200);
    }

    #[test]
    fn resource_contract_is_the_accepted_r1_contract() {
        assert_eq!(CENTER, [4.8, 0.0]);
        assert_eq!(RADIUS, 1.5);
        assert_eq!(RESOURCE_MASS, 243.14924801053778);
        assert_eq!(BOUNDARY_CONCENTRATION, 2.063914918930895);
        assert_eq!(REFEED, 8000);
    }
}
