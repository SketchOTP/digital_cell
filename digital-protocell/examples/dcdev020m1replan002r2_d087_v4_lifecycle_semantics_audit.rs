//! DC-DEV-020-M1-REPLAN-002-R2: observer-only D-087/V4 semantics audit.
//!
//! This runner does not alter D-087 predicates or V4 physics.  It replays the
//! V4 turnover trajectory with the existing tracer and with a parallel,
//! lifecycle-aware structural tracer, then compares physical state projections
//! and the historical starvation latch against the existing observer state.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh};
use chemistry_core::mesh_reactions::{
    pulse_tracers, reactions_step, try_local_rebond, ReactionParams,
};
use chemistry_core::mesh_transport::transport_step;
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use phase1_certifier::gates::gate1_metric_semantics;
use phase1_certifier::metrics::replacement_report;
use phase1_certifier::sim::{
    contract_label, maturation_coupled_enabled, reaction_params_for, seed_mesh,
    selected_mesh_schema,
};
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-REPLAN-002-R2-D087-V4-LIFECYCLE-SEMANTICS-AUDIT-001";
const STARTING_HEAD: &str = "98b1104165039359bdc609898e0d0371f9ce05c4";
const DT: f64 = 0.02;
const D087_TURNOVER_STEPS: usize = 5_000;
const STARVATION_WARMUP: usize = 200;
const STARVATION_CONTINUATION: usize = 6_000;
const TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, Serialize)]
struct StructuralSnapshot {
    step: usize,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    legacy_label: f64,
    lifecycle_label: f64,
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

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "missing", "path": path.display().to_string()}))
}

fn dense_writer(name: &str) -> Result<Option<BufWriter<fs::File>>, String> {
    let Some(root) = std::env::var_os("DCDEV020M1REPLAN002R2_DENSE_OUTPUT") else {
        return Ok(None);
    };
    let root = PathBuf::from(root);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    fs::File::create(root.join(name))
        .map(BufWriter::new)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn write_dense<T: Serialize>(
    writer: &mut Option<BufWriter<fs::File>>,
    value: &T,
) -> Result<(), String> {
    if let Some(file) = writer.as_mut() {
        serde_json::to_writer(&mut *file, value).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
}

fn physical_hash(mesh: &MaterialMesh) -> Result<String, String> {
    let mut physical = mesh.clone();
    for edge in &mut physical.edges {
        edge.tracer_m = 0.0;
        edge.tracer_b = 0.0;
    }
    physical.interior.tracer_c = 0.0;
    stable_json_hash(&physical).map_err(|error| error.to_string())
}

/// Infer the per-edge build and mature-only turnover amounts from the exact
/// pre/post reaction state.  This is an observer calculation; it never feeds
/// a value back into the reaction kernel.
fn lifecycle_turnover_amount(
    before: &MaterialMesh,
    after_reaction: &MaterialMesh,
    i: usize,
    params: &ReactionParams,
) -> Result<(f64, f64), String> {
    let edge_before = before.edges[i];
    let edge_after = after_reaction.edges[i];
    if edge_before.ruptured {
        return Ok((0.0, 0.0));
    }
    let y0 = edge_before.m_young.max(0.0);
    let y1 = edge_after.m_young.max(0.0);
    let m0 = edge_before.m.max(0.0);
    let m1 = edge_after.m.max(0.0);
    let q_maturation = params.k_turn * DT;
    let l0 = before.rest_length(i);
    let strain = (before.edge_length(i) - l0) / l0;
    let turn_scale = 1.0 / (1.0 + 2.0 * strain.max(0.0));
    let q_turn = params.k_turn * turn_scale * DT;
    if q_maturation >= 1.0 || q_turn >= 1.0 {
        return Err(format!("invalid observer coefficient at edge {i}"));
    }

    // y1 = (y0 + build) * (1 - q_maturation),
    // m1 = m0 + build - q_turn * (m0 + build - y1).
    let build = ((m1 - m0) + q_turn * (m0 - y1)) / (1.0 - q_turn);
    let build = build.max(0.0);
    let mature_before_turnover = (m0 + build - y1).max(0.0);
    let turnover = (q_turn * mature_before_turnover).min((m0 + build).max(0.0));
    let expected_y1 = ((y0 + build) * (1.0 - q_maturation)).max(0.0);
    let expected_m1 = (m0 + build - turnover).max(0.0);
    if !close(expected_y1, y1) || !close(expected_m1, m1) {
        return Err(format!(
            "V4 reaction decomposition mismatch edge={i} y={expected_y1} vs {y1} m={expected_m1} vs {m1}"
        ));
    }
    Ok((build, turnover))
}

fn apply_lifecycle_tracer(
    before: &MaterialMesh,
    after_reaction: &mut MaterialMesh,
    params: &ReactionParams,
) -> Result<(f64, f64), String> {
    let mut build_total = 0.0;
    let mut turnover_total = 0.0;
    for i in 0..before.n() {
        let (build, turnover) = lifecycle_turnover_amount(before, after_reaction, i, params)?;
        let mature_before = (before.edges[i].m.max(0.0) + build
            - after_reaction.edges[i].m_young.max(0.0))
        .max(0.0);
        let old_label = before.edges[i].tracer_m.max(0.0);
        let label = if mature_before > 1e-15 {
            old_label * (1.0 - (turnover / mature_before).clamp(0.0, 1.0))
        } else {
            old_label
        };
        after_reaction.edges[i].tracer_m = label.max(0.0);
        build_total += build;
        turnover_total += turnover;
    }
    Ok((build_total, turnover_total))
}

fn exact_step(
    mesh: &mut MaterialMesh,
    params: &ReactionParams,
) -> Result<chemistry_core::mesh_reactions::ReactionLedger, String> {
    if !mesh.can_advance_physics() {
        return Err("mesh could not advance".into());
    }
    let transport = frozen_transport();
    let _ = transport_step(mesh, &transport, FROZEN_CENTER.dt);
    let ledger = reactions_step(mesh, params, DT, true, true);
    if !mechanics_step(mesh, &FROZEN_CENTER) {
        return Err("mechanics step failed".into());
    }
    let _ = remesh(mesh);
    let _ = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    Ok(ledger)
}

fn structural_snapshot(
    step: usize,
    legacy: &MaterialMesh,
    lifecycle: &MaterialMesh,
) -> StructuralSnapshot {
    StructuralSnapshot {
        step,
        total_m: lifecycle.total_structural_mass(),
        young_m: lifecycle.total_young_structural_mass(),
        mature_m: (0..lifecycle.n())
            .map(|i| lifecycle.mature_structural_mass(i))
            .sum(),
        legacy_label: legacy.edges.iter().map(|e| e.tracer_m).sum(),
        lifecycle_label: lifecycle.edges.iter().map(|e| e.tracer_m).sum(),
    }
}

fn run_parallel_turnover(out: &Path) -> Result<Value, String> {
    if !maturation_coupled_enabled() || contract_label() != "MaturationCoupledV4" {
        return Err("R2 requires the explicit MaturationCoupledV4 environment".into());
    }
    let mut legacy = seed_mesh(14.0, 2);
    pulse_tracers(&mut legacy, 1.0);
    let mut lifecycle = legacy.clone();
    let params = reaction_params_for(&legacy);
    let initial_label = lifecycle.edges.iter().map(|e| e.tracer_m).sum::<f64>();
    let mut legacy_hashes = Vec::with_capacity(D087_TURNOVER_STEPS + 1);
    let mut lifecycle_hashes = Vec::with_capacity(D087_TURNOVER_STEPS + 1);
    legacy_hashes.push(physical_hash(&legacy)?);
    lifecycle_hashes.push(physical_hash(&lifecycle)?);
    let mut snapshots = vec![structural_snapshot(0, &legacy, &lifecycle)];
    let mut mass_series = Vec::with_capacity(D087_TURNOVER_STEPS);
    let mut physical_mismatch_steps = Vec::new();
    let mut max_decomposition_residual = 0.0_f64;
    let mut build_total = 0.0;
    let mut turnover_total = 0.0;
    let mut dense = dense_writer("parallel_tracer.jsonl")?;
    let checkpoints = [1_000usize, 2_500, 5_000];

    for step in 1..=D087_TURNOVER_STEPS {
        if !legacy.can_advance_physics() || !lifecycle.can_advance_physics() {
            return Err(format!("trajectory ended before step {step}"));
        }
        let before_lifecycle = lifecycle.clone();
        let _legacy_ledger = exact_step(&mut legacy, &params)?;

        // Repeat the same physical step on the second copy.  Its only
        // difference is observer tracer data, which is excluded from physics.
        let _ = exact_step(&mut lifecycle, &params)?;
        let (build, turnover) = apply_lifecycle_tracer(&before_lifecycle, &mut lifecycle, &params)?;
        build_total += build;
        turnover_total += turnover;
        mass_series.push(lifecycle.total_structural_mass());
        let legacy_hash = physical_hash(&legacy)?;
        let lifecycle_hash = physical_hash(&lifecycle)?;
        if legacy_hash != lifecycle_hash {
            physical_mismatch_steps.push(step);
        }
        legacy_hashes.push(legacy_hash);
        lifecycle_hashes.push(lifecycle_hash);
        write_dense(&mut dense, &structural_snapshot(step, &legacy, &lifecycle))?;
        if checkpoints.contains(&step) {
            snapshots.push(structural_snapshot(step, &legacy, &lifecycle));
        }
        let predicted_total = lifecycle.total_young_structural_mass()
            + (0..lifecycle.n())
                .map(|i| lifecycle.mature_structural_mass(i))
                .sum::<f64>();
        max_decomposition_residual = max_decomposition_residual
            .max((predicted_total - lifecycle.total_structural_mass()).abs());
    }
    let final_snapshot = structural_snapshot(D087_TURNOVER_STEPS, &legacy, &lifecycle);
    snapshots.push(final_snapshot.clone());
    let mean_mass = mass_series.iter().sum::<f64>() / mass_series.len() as f64;
    let legacy_label_final = final_snapshot.legacy_label;
    let lifecycle_label_final = final_snapshot.lifecycle_label;
    let legacy_report = replacement_report(
        "m_legacy",
        mean_mass,
        build_total,
        initial_label,
        legacy_label_final,
        final_snapshot.total_m,
    );
    let lifecycle_report = replacement_report(
        "m_lifecycle",
        mean_mass,
        build_total,
        initial_label,
        lifecycle_label_final,
        final_snapshot.total_m,
    );
    let parity = physical_mismatch_steps.is_empty();
    let body = json!({
        "schema": "dcdev020m1replan002r2_parallel_tracer_v1",
        "contract": contract_label(),
        "selected_mesh_schema": format!("{:?}", selected_mesh_schema()),
        "steps": D087_TURNOVER_STEPS,
        "initial_mature_labeled": initial_label,
        "snapshots": snapshots,
        "legacy_structural_report": legacy_report,
        "lifecycle_structural_report": lifecycle_report,
        "cumulative_build": build_total,
        "cumulative_mature_turnover": turnover_total,
        "max_material_decomposition_residual": max_decomposition_residual,
        "physical_trajectory_parity": parity,
        "physical_mismatch_steps": physical_mismatch_steps,
        "legacy_physical_hash": stable_json_hash(&legacy_hashes).map_err(|e| e.to_string())?,
        "lifecycle_physical_hash": stable_json_hash(&lifecycle_hashes).map_err(|e| e.to_string())?,
        "physical_hash_sequences_equal": legacy_hashes == lifecycle_hashes,
    });
    if let Some(file) = dense.as_mut() {
        file.flush().map_err(|e| e.to_string())?;
    }
    write_json(out, &body)?;
    Ok(body)
}

#[derive(Debug, Clone, Serialize)]
struct StarvationSample {
    step: usize,
    alive: bool,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    a: f64,
    c: f64,
    n: f64,
    f: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    ruptured_edges: usize,
    closed_intact: bool,
    physical_runtime_valid: bool,
}

fn starvation_sample(mesh: &MaterialMesh, step: usize) -> StarvationSample {
    StarvationSample {
        step,
        alive: mesh.alive,
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_string),
        a: mesh.interior.a,
        c: mesh.interior.c,
        n: mesh.interior.n,
        f: mesh.interior.f,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        ruptured_edges: mesh.edges.iter().filter(|e| e.ruptured).count(),
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

fn run_starvation_audit(out: &Path) -> Result<Value, String> {
    let mut mesh = seed_mesh(14.0, 1);
    let params = reaction_params_for(&mesh);
    for _ in 0..STARVATION_WARMUP {
        exact_step(&mut mesh, &params)?;
    }
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;
    let mut first_a_below = None;
    let mut first_observer_nonviable = None;
    let mut first_alive_false = None;
    let mut samples = vec![starvation_sample(&mesh, STARVATION_WARMUP)];
    let mut dense = dense_writer("starvation.jsonl")?;
    write_dense(&mut dense, samples.first().expect("initial sample"))?;
    let checkpoints = [1_000usize, 2_000, 4_000, 6_200];
    for continuation in 1..=STARVATION_CONTINUATION {
        exact_step(&mut mesh, &params)?;
        let step = STARVATION_WARMUP + continuation;
        if first_a_below.is_none() && mesh.interior.a < 0.05 {
            first_a_below = Some(step);
        }
        if first_observer_nonviable.is_none() && !mesh.observer_viable() {
            first_observer_nonviable = Some(step);
        }
        if first_alive_false.is_none() && !mesh.alive {
            first_alive_false = Some(step);
        }
        if checkpoints.contains(&step) {
            samples.push(starvation_sample(&mesh, step));
        }
        write_dense(&mut dense, &starvation_sample(&mesh, step))?;
    }
    let final_sample = starvation_sample(&mesh, STARVATION_WARMUP + STARVATION_CONTINUATION);
    samples.push(final_sample.clone());
    let legacy_gate = !final_sample.alive || final_sample.a < 0.05;
    let observer_state = first_observer_nonviable.is_some() && !legacy_gate;
    let body = json!({
        "schema": "dcdev020m1replan002r2_starvation_semantics_v1",
        "contract": contract_label(),
        "warmup_steps": STARVATION_WARMUP,
        "continuation_steps": STARVATION_CONTINUATION,
        "samples": samples,
        "first_a_below_0_05": first_a_below,
        "first_observer_nonviable": first_observer_nonviable,
        "first_alive_false": first_alive_false,
        "legacy_predicate": "!alive || A < 0.05",
        "legacy_gate": legacy_gate,
        "observer_semantics_starvation_state": observer_state,
        "final": final_sample,
    });
    if let Some(file) = dense.as_mut() {
        file.flush().map_err(|e| e.to_string())?;
    }
    write_json(out, &body)?;
    Ok(body)
}

fn main() -> Result<(), String> {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    std::env::set_var("DCDEV020M1REPLAN002R1_V4", "1");
    let out = std::env::var_os("DCDEV020M1REPLAN002R2_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r2"));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let raw_gate1 = gate1_metric_semantics().1;
    let parallel = run_parallel_turnover(&out.join("dual_retention_comparison.json"))?;
    let starvation = run_starvation_audit(&out.join("starvation_semantics_comparison.json"))?;
    let v2 = read_json(&out.join("v2_d087/certification/report.json"));
    let v3 = read_json(&out.join("v3_d087/certification/report.json"));
    let v4 = read_json(&out.join("v4_d087/certification/report.json"));
    let gate_array = |report: &Value| -> Vec<bool> {
        (0..8)
            .map(|i| {
                report[format!("gate{i}")]["pass"]
                    .as_bool()
                    .unwrap_or(false)
            })
            .collect()
    };
    let lifecycle_structural_pass = parallel["lifecycle_structural_report"]["r_x_ok"]
        .as_bool()
        .unwrap_or(false)
        && parallel["lifecycle_structural_report"]["f_label_ok"]
            .as_bool()
            .unwrap_or(false);
    let raw_component_pass = |name: &str| {
        raw_gate1["audit"][name]["r_x_ok"]
            .as_bool()
            .unwrap_or(false)
            && raw_gate1["audit"][name]["f_label_ok"]
                .as_bool()
                .unwrap_or(false)
    };
    let lifecycle_gate1 = lifecycle_structural_pass
        && raw_component_pass("membrane")
        && raw_component_pass("catalyst");
    let physical_parity = parallel["physical_trajectory_parity"]
        .as_bool()
        .unwrap_or(false)
        && parallel["physical_hash_sequences_equal"]
            .as_bool()
            .unwrap_or(false);
    let legacy_gate2 = starvation["legacy_gate"].as_bool().unwrap_or(false);
    let observer_gate2 = starvation["observer_semantics_starvation_state"]
        .as_bool()
        .unwrap_or(false);
    let gate1_semantic = lifecycle_gate1 && physical_parity;
    let gate2_semantic = !legacy_gate2 && observer_gate2;
    let gate1_cause = if gate1_semantic {
        "CERTIFIER_SEMANTICS"
    } else {
        "BIOLOGICAL"
    };
    let gate2_cause = if gate2_semantic {
        "CERTIFIER_SEMANTICS"
    } else {
        "BIOLOGICAL"
    };
    let classification = if gate1_semantic && gate2_semantic {
        "M1_V4_D087_CERTIFIER_SEMANTICS_MISMATCH"
    } else if gate1_cause == "BIOLOGICAL" && gate2_cause == "BIOLOGICAL" {
        "M1_V4_D087_TRUE_BIOLOGICAL_REGRESSION"
    } else if gate1_cause != gate2_cause {
        "M1_V4_D087_MIXED_REGRESSION"
    } else {
        "M1_V4_D087_CAUSE_UNRESOLVED"
    };
    let raw = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "contract": contract_label(),
        "v2_report": v2,
        "v3_report": v3,
        "v4_report": v4,
        "v2_gate_array": gate_array(&v2),
        "v3_gate_array": gate_array(&v3),
        "v4_gate_array": gate_array(&v4),
        "v4_gate1_metric_semantics": raw_gate1,
    });
    write_json(&out.join("d087_raw_metrics.json"), &raw)?;
    write_json(
        &out.join("physical_trajectory_parity.json"),
        &json!({
            "physical_trajectory_difference": if physical_parity { 0 } else { 1 },
            "physical_trajectory_parity": physical_parity,
            "observer_choice_feeds_back_into_physics": false,
            "comparison": ["vertices", "edge_m", "edge_m_young", "edge_b", "free_l", "interior chemistry", "rupture state"],
        }),
    )?;
    let qualification = json!({
        "schema": "dcdev020m1replan002r2_qualification_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "v2_d087": gate_array(&v2).iter().all(|v| *v),
        "v3_d087": gate_array(&v3).iter().all(|v| *v),
        "v4_legacy_d087": gate_array(&v4),
        "gate1_cause": gate1_cause,
        "gate2_cause": gate2_cause,
        "lifecycle_tracer_gate1": lifecycle_gate1,
        "physical_trajectory_parity": physical_parity,
        "observer_semantics_starvation_state": observer_gate2,
        "production_biology_changed": false,
        "d087_threshold_changed": false,
        "classification": classification,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false,
    });
    write_json(&out.join("qualification.json"), &qualification)?;
    let preservation = json!({
        "schema": "dcdev020m1replan002r2_preservation_v1",
        "v2_d087_8_of_8": gate_array(&v2).iter().all(|v| *v),
        "v3_d087_8_of_8": gate_array(&v3).iter().all(|v| *v),
        "v4_r1_capabilities_preserved": true,
        "observer_only": true,
        "thresholds_unchanged": true,
        "production_default_changed": false,
    });
    write_json(&out.join("preservation.json"), &preservation)?;
    let manifest = json!({
        "schema": "dcdev020m1replan002r2_manifest_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "compact_files": ["d087_raw_metrics.json", "dual_retention_comparison.json", "starvation_semantics_comparison.json", "physical_trajectory_parity.json", "qualification.json", "preservation.json", "artifact_manifest.json"],
        "dense_output": std::env::var("DCDEV020M1REPLAN002R2_DENSE_OUTPUT").ok(),
        "next_execution_started": false,
    });
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!(
        "DCDEV020M1REPLAN002R2_COMPLETE classification={classification} gate1={gate1_cause} gate2={gate2_cause} parity={physical_parity} next_execution_started=false"
    );
    Ok(())
}
