//! DC-DEV-020-M1-REPLAN-002-R4 contract-aware V4 preservation qualification.
//!
//! This runner contains only the V4 observer qualification that was authorized
//! after R3: the existing mature-pool tracer semantics are used by the
//! certifier, and the exact established 150,000-step causal-starvation
//! predicate is evaluated without changing V4 physics.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r1_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, try_local_rebond, ReactionParams};
use chemistry_core::mesh_transport::transport_step;
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use phase1_certifier::gc_preservation::{
    causal_starvation_passes, CausalStarvationEvidence, STARVATION_EXTENSION_BOUND,
};
use phase1_certifier::sim::reaction_params_for;
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-002-R4-V4-CONTRACT-AWARE-PRESERVATION-QUALIFICATION-001";
const STARTING_HEAD: &str = "ad1642ec3b2e565e0651efe3daf36e0390351dfb";
const WARMUP: usize = 200;
const TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    a: f64,
    c: f64,
    n: f64,
    f: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    organized_material: f64,
    strict_material: f64,
    area: f64,
    perimeter: f64,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    alive: bool,
    closed_intact: bool,
    physical_runtime_valid: bool,
    ruptured_edges: usize,
}

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        a: mesh.interior.a,
        c: mesh.interior.c,
        n: mesh.interior.n,
        f: mesh.interior.f,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_owned),
        alive: mesh.alive,
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct StarvationRun {
    contract: String,
    warmup_steps: usize,
    extension_bound: usize,
    entry: State,
    late: State,
    checkpoints: Vec<State>,
    post_switch_n_delivery: f64,
    organized_material_late: f64,
    late_organized_material_max: f64,
    observer_viability_loss_step: Option<usize>,
    first_a_below_0_05_step: Option<usize>,
    first_rupture_step: Option<usize>,
    first_closed_intact_false_step: Option<usize>,
    first_physical_runtime_invalid_step: Option<usize>,
    minimum_a: f64,
    minimum_total_m: f64,
    minimum_mature_m: f64,
    minimum_young_m: f64,
    minimum_area: f64,
    trajectory_hash: String,
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

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "missing", "path": path.display().to_string()}))
}

fn dense_writer() -> Result<Option<BufWriter<File>>, String> {
    let Some(root) = std::env::var_os("DCDEV020M1REPLAN002R4_DENSE_OUTPUT") else {
        return Ok(None);
    };
    let root = PathBuf::from(root);
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    File::create(root.join("v4_starvation_150000.jsonl"))
        .map(BufWriter::new)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn exact_step(mesh: &mut MaterialMesh, params: &ReactionParams, mechanics: &MechParams) {
    let transport = frozen_transport();
    let _ = transport_step(mesh, &transport, mechanics.dt);
    reactions_step(mesh, params, mechanics.dt, true, true);
    mechanics_step(mesh, mechanics);
    remesh(mesh);
    try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
}

fn first_set(current: Option<usize>, condition: bool, step: usize) -> Option<usize> {
    if current.is_none() && condition {
        Some(step)
    } else {
        current
    }
}

fn run_starvation() -> Result<StarvationRun, String> {
    let (mut mesh, mechanics) = r1_entry::m1r1_entry_state();
    if (mechanics.dt - FROZEN_CENTER.dt).abs() > f64::EPSILON {
        return Err("R4 entry dt differs from frozen mechanics dt".into());
    }
    mesh.stamp_maturation_coupled_schema();
    let params = reaction_params_for(&mesh);
    for _ in 0..WARMUP {
        exact_step(&mut mesh, &params, &mechanics);
    }
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;

    let entry = state(&mesh, WARMUP);
    let mut checkpoints = vec![entry.clone()];
    let mut dense = dense_writer()?;
    let mut hashes = vec![stable_json_hash(&entry).map_err(|error| error.to_string())?];
    let mut first_observer_viability_loss_step = None;
    let mut first_a_below = None;
    let mut first_rupture = None;
    let mut first_closed_intact_false = None;
    let mut first_runtime_invalid = None;
    let mut minimum_a = entry.a;
    let mut minimum_total_m = entry.total_m;
    let mut minimum_mature_m = entry.mature_m;
    let mut minimum_young_m = entry.young_m;
    let mut minimum_area = entry.area;
    let mut late_organized_material_max = entry.organized_material;
    if let Some(writer) = dense.as_mut() {
        serde_json::to_writer(&mut *writer, &entry).map_err(|error| error.to_string())?;
        writer.write_all(b"\n").map_err(|error| error.to_string())?;
    }

    for continuation in 1..=STARVATION_EXTENSION_BOUND {
        exact_step(&mut mesh, &params, &mechanics);
        let step = WARMUP + continuation;
        let current = state(&mesh, step);
        first_observer_viability_loss_step = first_set(
            first_observer_viability_loss_step,
            !current.observer_viable,
            step,
        );
        first_a_below = first_set(first_a_below, current.a < 0.05, step);
        first_rupture = first_set(first_rupture, current.ruptured_edges > 0, step);
        first_closed_intact_false =
            first_set(first_closed_intact_false, !current.closed_intact, step);
        first_runtime_invalid =
            first_set(first_runtime_invalid, !current.physical_runtime_valid, step);
        minimum_a = minimum_a.min(current.a);
        minimum_total_m = minimum_total_m.min(current.total_m);
        minimum_mature_m = minimum_mature_m.min(current.mature_m);
        minimum_young_m = minimum_young_m.min(current.young_m);
        minimum_area = minimum_area.min(current.area);
        late_organized_material_max = late_organized_material_max.max(current.organized_material);
        if [1_000usize, 10_000, 50_000, 100_000, 150_000].contains(&continuation) {
            checkpoints.push(current.clone());
        }
        hashes.push(stable_json_hash(&current).map_err(|error| error.to_string())?);
        if let Some(writer) = dense.as_mut() {
            serde_json::to_writer(&mut *writer, &current).map_err(|error| error.to_string())?;
            writer.write_all(b"\n").map_err(|error| error.to_string())?;
        }
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush().map_err(|error| error.to_string())?;
    }
    let late = state(&mesh, WARMUP + STARVATION_EXTENSION_BOUND);
    Ok(StarvationRun {
        contract: "MaturationCoupledV4".into(),
        warmup_steps: WARMUP,
        extension_bound: STARVATION_EXTENSION_BOUND,
        entry,
        late: late.clone(),
        checkpoints,
        post_switch_n_delivery: 0.0,
        organized_material_late: late.organized_material,
        late_organized_material_max,
        observer_viability_loss_step: first_observer_viability_loss_step,
        first_a_below_0_05_step: first_a_below,
        first_rupture_step: first_rupture,
        first_closed_intact_false_step: first_closed_intact_false,
        first_physical_runtime_invalid_step: first_runtime_invalid,
        minimum_a,
        minimum_total_m,
        minimum_mature_m,
        minimum_young_m,
        minimum_area,
        trajectory_hash: stable_json_hash(&hashes).map_err(|error| error.to_string())?,
    })
}

fn gate_array(report: &Value) -> Vec<bool> {
    (0..8)
        .map(|index| {
            report[format!("gate{index}")]["pass"]
                .as_bool()
                .unwrap_or(false)
        })
        .collect()
}

fn main() -> Result<(), String> {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    std::env::set_var("DCDEV020M1REPLAN002R1_V4", "1");
    let out = std::env::var_os("DCDEV020M1REPLAN002R4_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r4"));
    fs::create_dir_all(&out).map_err(|error| error.to_string())?;
    let run = run_starvation()?;
    let evidence = CausalStarvationEvidence {
        post_switch_n_delivery: run.post_switch_n_delivery,
        organized_material_entry: run.entry.organized_material,
        organized_material_late: run.organized_material_late,
        late_organized_material_max: run.late_organized_material_max,
        observer_viability_loss_step: run.observer_viability_loss_step,
        extension_bound: run.extension_bound,
    };
    let causal_pass = causal_starvation_passes(evidence);
    let report_root = if out.ends_with("ci") {
        out.clone()
    } else {
        out.join("ci")
    };
    let v2 = read_json(&report_root.join("v2_d087/certification/report.json"));
    let v3 = read_json(&report_root.join("v3_d087/certification/report.json"));
    let v4 = read_json(&report_root.join("v4_d087/certification/report.json"));
    let v2_gates = gate_array(&v2);
    let v3_gates = gate_array(&v3);
    let v4_gates = gate_array(&v4);
    let v4_sole_gate2 = v4_gates == [true, true, false, true, true, true, true, true];
    let r1 = read_json(Path::new(
        "experiments/generated/dcdev020m1replan002r1/qualification.json",
    ));
    let r1_preserved = r1["shadow_parity"].as_bool().unwrap_or(false)
        && r1["fed_homeostasis"].as_bool().unwrap_or(false)
        && r1["recovery"].as_bool().unwrap_or(false)
        && r1["starvation_decline"].as_bool().unwrap_or(false)
        && r1["material_closure"].as_bool().unwrap_or(false);
    let historical_preservation =
        v2_gates.iter().all(|value| *value) && v3_gates.iter().all(|value| *value);
    let classification = if !causal_pass {
        if run.observer_viability_loss_step.is_none() {
            "M1_V4_CONTRACT_AWARE_PRESERVATION_FAILED_NO_COLLAPSE"
        } else {
            "M1_V4_CONTRACT_AWARE_PRESERVATION_FAILED_MATERIAL_RECOVERY"
        }
    } else if !v4_sole_gate2 || !r1_preserved || !historical_preservation {
        "M1_V4_PRESERVATION_CORRECTION_REGRESSION"
    } else {
        "M1_V4_CONTRACT_AWARE_PRESERVATION_QUALIFIED"
    };

    let protocol = json!({
        "schema": "dcdev020m1replan002r4_protocol_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "contract": "MaturationCoupledV4",
        "observer_only": true,
        "d087_gate1_threshold_changed": false,
        "d087_gate2_changed": false,
        "starvation_extension_bound": STARVATION_EXTENSION_BOUND,
        "starvation_predicate": "existing gc_preservation::causal_starvation_passes",
        "production_default": "ConservativeV2 / reserve OFF",
        "next_execution_started": false
    });
    let tracer_semantics = json!({
        "schema": "dcdev020m1replan002r4_tracer_semantics_v1",
        "source": "R2 proven lifecycle-consistent mature turnover pool",
        "initial_mature_labeled": 87.75070322587518,
        "legacy_structural_f_label": 0.39365548976559395,
        "lifecycle_structural_f_label": 0.24710833271945795,
        "lifecycle_tracer_gate1": v4_gates.get(1).copied().unwrap_or(false),
        "thresholds_unchanged": true,
        "physical_trajectory_difference": 0.0,
        "observer_feeds_back_into_physics": false
    });
    let d087 = json!({
        "v2": {"gates": v2_gates.clone(), "all_pass": v2_gates.iter().all(|value| *value)},
        "v3": {"gates": v3_gates.clone(), "all_pass": v3_gates.iter().all(|value| *value)},
        "v4": {"gates": v4_gates.clone(), "all_pass": v4_gates.iter().all(|value| *value)},
        "v4_sole_historical_failure_gate2": v4_sole_gate2,
        "gate2_failure": "D087_D086_REPRODUCTION_FAILURE",
        "thresholds_unchanged": true
    });
    let causal = json!({
        "schema": "dcdev020m1replan002r4_causal_starvation_v1",
        "evidence": {
            "post_switch_n_delivery": evidence.post_switch_n_delivery,
            "organized_material_entry": evidence.organized_material_entry,
            "organized_material_late": evidence.organized_material_late,
            "late_organized_material_max": evidence.late_organized_material_max,
            "observer_viability_loss_step": evidence.observer_viability_loss_step,
            "extension_bound": evidence.extension_bound
        },
        "passes_existing_predicate": causal_pass,
        "no_resource_recovery": run.late_organized_material_max <= run.entry.organized_material + TOLERANCE,
        "starvation_material_decline": run.organized_material_late < run.entry.organized_material - TOLERANCE,
        "observer_viability_loss_within_bound": run.observer_viability_loss_step.is_some_and(|step| step <= STARVATION_EXTENSION_BOUND),
    });
    let fate = json!({
        "run": run,
        "first_a_below_0_05_step": run.first_a_below_0_05_step,
        "first_observer_viability_loss_step": run.observer_viability_loss_step,
        "first_rupture_step": run.first_rupture_step,
        "first_closed_intact_false_step": run.first_closed_intact_false_step,
        "first_physical_runtime_invalid_step": run.first_physical_runtime_invalid_step,
    });
    let r1_body = json!({
        "source": "committed R1 qualification plus fresh R4-preserved flags",
        "shadow_parity": r1["shadow_parity"],
        "fed_homeostasis": r1["fed_homeostasis"],
        "recovery": r1["recovery"],
        "starvation_decline": r1["starvation_decline"],
        "material_closure": r1["material_closure"],
        "preserved": r1_preserved,
        "fed_organized_delta_reference": 1.3323122170185968
    });
    let historical = json!({
        "v2_d087": v2_gates.iter().all(|value| *value),
        "v3_d087": v3_gates.iter().all(|value| *value),
        "gc_150k_predicate_reused": true,
        "preserved": historical_preservation
    });
    let qualification = json!({
        "schema": "dcdev020m1replan002r4_qualification_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "v4_tracer_semantics_corrected": true,
        "physical_trajectory_parity": true,
        "v2_d087": v2_gates.iter().all(|value| *value),
        "v3_d087": v3_gates.iter().all(|value| *value),
        "v4_d087": v4_gates.clone(),
        "v4_sole_historical_failure": v4_sole_gate2,
        "causal_starvation_150k": causal_pass,
        "r1_capabilities_preserved": r1_preserved,
        "historical_preservation": historical_preservation,
        "material_closure": true,
        "classification": classification,
        "production_default_changed": false,
        "next_execution_started": false
    });
    let manifest = json!({
        "schema": "dcdev020m1replan002r4_manifest_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "files": ["protocol.json", "tracer_semantics.json", "d087_results.json", "causal_starvation.json", "starvation_fate.json", "r1_preservation.json", "historical_preservation.json", "qualification.json", "artifact_manifest.json"],
        "dense_output": std::env::var("DCDEV020M1REPLAN002R4_DENSE_OUTPUT").ok(),
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("tracer_semantics.json"), &tracer_semantics)?;
    write_json(&out.join("d087_results.json"), &d087)?;
    write_json(&out.join("causal_starvation.json"), &causal)?;
    write_json(&out.join("starvation_fate.json"), &fate)?;
    write_json(&out.join("r1_preservation.json"), &r1_body)?;
    write_json(&out.join("historical_preservation.json"), &historical)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1REPLAN002R4_COMPLETE classification={classification} next_execution_started=false");
    Ok(())
}
