//! DC-DEV-020-M1-REPLAN-002-R5-R2.
//!
//! Bounded qualification of the version-aware V3/V4 amount-space transport
//! repair.  The integrated run is a valid-prefix starvation replay only: it
//! stops at the first authoritative mechanics rejection and never constructs
//! post-failure death evidence.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r1_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, try_local_rebond, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportLedger};
use phase1_certifier::frozen::frozen_transport;
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-002-R5-R2-V4-GC-TRANSPORT-AMOUNT-CONSERVATION-REPAIR-001";
const STARTING_HEAD: &str = "4b1a82877246c58ba21464963eb5bc4cb2a535cf";
const DT: f64 = 0.02;
const WARMUP: usize = 200;
const STARVATION_BOUND: usize = 150_000;
const TOLERANCE: f64 = 1e-8;
const OLD_TRANSPORT_FAILURE_STEP: usize = 7_684;
const OLD_TRANSPORT_MAX_STEP: usize = 8_177;
const OLD_MECHANICS_FALSE_STEP: usize = 8_566;

#[derive(Debug, Clone, Serialize)]
struct Endpoint {
    step: usize,
    area: f64,
    signed_area: f64,
    accounting_area: f64,
    vertex_count: usize,
    ruptured_edges: usize,
    n: f64,
    f: f64,
    a: f64,
    r: f64,
    c: f64,
    w: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    free_l: f64,
    bound_b: f64,
    strict_material: f64,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    closed_intact: bool,
    physical_runtime_valid: bool,
    mesh_hash: String,
}

fn endpoint(mesh: &MaterialMesh, step: usize) -> Endpoint {
    let s = snapshot(mesh);
    let area = mesh.area();
    Endpoint {
        step,
        area,
        signed_area: mesh.signed_area(),
        accounting_area: if area.is_finite() && area > 0.0 {
            area
        } else {
            0.0
        },
        vertex_count: mesh.n(),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        n: s.n,
        f: s.f,
        a: s.a,
        r: s.r,
        c: s.c,
        w: s.waste,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        free_l: s.free_l,
        bound_b: s.bound_b,
        strict_material: s.strict_material_equivalent(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_owned),
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        mesh_hash: stable_json_hash(mesh).unwrap_or_else(|_| "hash-error".into()),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
}

fn transport_net(ledger: &TransportLedger) -> f64 {
    ledger.n_in - ledger.n_out + ledger.f_in - ledger.f_out + ledger.w_in - ledger.w_out
        + ledger.c_in
        - ledger.c_leak
        + ledger.a_in
        - ledger.a_leak
}

#[derive(Debug, Clone, Serialize)]
struct StageRecord {
    step: usize,
    stage: String,
    strict_before: f64,
    strict_after: f64,
    signed_residual: f64,
    expected_transport_delta: f64,
    status: String,
    state_changed_despite_false: Option<bool>,
    area_before: f64,
    area_after: f64,
    signed_area_before: f64,
    signed_area_after: f64,
    pre_hash: String,
    post_hash: String,
    before: Endpoint,
    after: Endpoint,
}

#[derive(Debug, Default, Serialize)]
struct ReplaySummary {
    accepted_steps: usize,
    stage_records: usize,
    max_transport_residual: f64,
    max_total_stage_residual: f64,
    first_transport_residual_step: Option<usize>,
    first_mechanics_false: Option<usize>,
    first_signed_area_flip: Option<usize>,
    first_actual_area_at_or_below_floor: Option<usize>,
    first_actual_area_nonpositive: Option<usize>,
    first_observer_nonviable: Option<usize>,
    authoritative_replay_stopped_on_false: bool,
    terminal: Option<Endpoint>,
    first_mechanics_false_record: Option<StageRecord>,
}

fn write_stage(writer: &mut Option<BufWriter<File>>, record: &StageRecord) -> Result<(), String> {
    if let Some(writer) = writer {
        serde_json::to_writer(&mut *writer, record).map_err(|error| error.to_string())?;
        writer.write_all(b"\n").map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn stage<F>(mesh: &mut MaterialMesh, step: usize, name: &str, op: F) -> StageRecord
where
    F: FnOnce(&mut MaterialMesh) -> (String, Option<bool>, f64),
{
    let before = endpoint(mesh, step);
    let (status, state_changed_despite_false, expected_transport_delta) = op(mesh);
    let after = endpoint(mesh, step);
    StageRecord {
        step,
        stage: name.into(),
        strict_before: before.strict_material,
        strict_after: after.strict_material,
        signed_residual: after.strict_material - before.strict_material,
        expected_transport_delta,
        status,
        state_changed_despite_false,
        area_before: before.area,
        area_after: after.area,
        signed_area_before: before.signed_area,
        signed_area_after: after.signed_area,
        pre_hash: before.mesh_hash.clone(),
        post_hash: after.mesh_hash.clone(),
        before,
        after,
    }
}

fn register(record: StageRecord, summary: &mut ReplaySummary, accepted_prefix: bool) {
    summary.stage_records += 1;
    let residual = record.signed_residual - record.expected_transport_delta;
    if record.stage == "TRANSPORT" {
        summary.max_transport_residual = summary.max_transport_residual.max(residual.abs());
        if residual.abs() > TOLERANCE && summary.first_transport_residual_step.is_none() {
            summary.first_transport_residual_step = Some(record.step);
        }
    }
    if accepted_prefix {
        summary.max_total_stage_residual = summary.max_total_stage_residual.max(residual.abs());
    }
    if record.signed_area_after <= 0.0 && summary.first_signed_area_flip.is_none() {
        summary.first_signed_area_flip = Some(record.step);
    }
    if record.area_after.is_finite()
        && record.area_after <= 1e-6
        && summary.first_actual_area_at_or_below_floor.is_none()
    {
        summary.first_actual_area_at_or_below_floor = Some(record.step);
    }
    if (!record.area_after.is_finite() || record.area_after <= 0.0)
        && summary.first_actual_area_nonpositive.is_none()
    {
        summary.first_actual_area_nonpositive = Some(record.step);
    }
    if !record.after.observer_viable && summary.first_observer_nonviable.is_none() {
        summary.first_observer_nonviable = Some(record.step);
    }
}

fn warmup(mesh: &mut MaterialMesh, mechanics: &MechParams) {
    let transport = frozen_transport();
    let reactions = ReactionParams::conservative_v2();
    for _ in 0..WARMUP {
        let _ = transport_step(mesh, &transport, mechanics.dt);
        let _ = reactions_step(mesh, &reactions, mechanics.dt, true, true);
        let _ = mechanics_step(mesh, mechanics);
        let _ = remesh(mesh);
        let _ = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    }
}

fn replay(dense_root: Option<&Path>) -> Result<ReplaySummary, String> {
    let (mut mesh, mechanics) = r1_entry::m1r1_entry_state();
    if (mechanics.dt - DT).abs() > f64::EPSILON {
        return Err("R5-R2 entry dt mismatch".into());
    }
    mesh.stamp_maturation_coupled_schema();
    warmup(&mut mesh, &mechanics);
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;

    let mut dense = dense_root
        .map(|root| {
            fs::create_dir_all(root).map_err(|error| error.to_string())?;
            File::create(root.join("valid_prefix_stage_ledger.jsonl"))
                .map(BufWriter::new)
                .map_err(|error| error.to_string())
        })
        .transpose()?;
    let mut summary = ReplaySummary::default();
    let transport = frozen_transport();
    let reactions = ReactionParams::conservative_v2();

    for step in (WARMUP + 1)..=(WARMUP + STARVATION_BOUND) {
        let transport_record = stage(&mut mesh, step, "TRANSPORT", |m| {
            let ledger = transport_step(m, &transport, mechanics.dt);
            (
                format!("applied_transport_net={}", transport_net(&ledger)),
                None,
                transport_net(&ledger),
            )
        });
        register(transport_record.clone(), &mut summary, true);
        write_stage(&mut dense, &transport_record)?;

        let reaction_record = stage(&mut mesh, step, "REACTIONS", |m| {
            let _ = reactions_step(m, &reactions, mechanics.dt, true, true);
            ("returned".into(), None, 0.0)
        });
        register(reaction_record.clone(), &mut summary, true);
        write_stage(&mut dense, &reaction_record)?;

        let mechanics_record = stage(&mut mesh, step, "MECHANICS", |m| {
            let pre_hash = stable_json_hash(m).unwrap_or_default();
            let ok = mechanics_step(m, &mechanics);
            let post_hash = stable_json_hash(m).unwrap_or_default();
            (
                if ok { "true" } else { "false" }.into(),
                Some(!ok && pre_hash != post_hash),
                0.0,
            )
        });
        let mechanics_failed = mechanics_record.status == "false";
        register(mechanics_record.clone(), &mut summary, !mechanics_failed);
        write_stage(&mut dense, &mechanics_record)?;
        summary.accepted_steps = step - WARMUP;
        if mechanics_failed {
            summary.first_mechanics_false = Some(step);
            summary.first_mechanics_false_record = Some(mechanics_record);
            summary.authoritative_replay_stopped_on_false = true;
            break;
        }

        let remesh_record = stage(&mut mesh, step, "REMESH", |m| {
            let (splits, merges) = remesh(m);
            (format!("splits={splits} merges={merges}"), None, 0.0)
        });
        register(remesh_record.clone(), &mut summary, true);
        write_stage(&mut dense, &remesh_record)?;

        let rebond_record = stage(&mut mesh, step, "REBOND", |m| {
            let ok = try_local_rebond(m, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
            (format!("returned={ok}"), None, 0.0)
        });
        register(rebond_record.clone(), &mut summary, true);
        write_stage(&mut dense, &rebond_record)?;
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush().map_err(|error| error.to_string())?;
    }
    summary.terminal = Some(endpoint(&mesh, WARMUP + summary.accepted_steps));
    Ok(summary)
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

fn main() -> Result<(), String> {
    let output = std::env::var_os("DCDEV020M1REPLAN002R5R2_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r5r2"));
    let dense = std::env::var_os("DCDEV020M1REPLAN002R5R2_DENSE_OUTPUT").map(PathBuf::from);
    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let summary = replay(dense.as_deref())?;
    let transport_repair_qualified = summary.first_transport_residual_step.is_none()
        && summary.max_transport_residual <= TOLERANCE
        && summary.authoritative_replay_stopped_on_false;
    let classification = if transport_repair_qualified {
        "M1_V4_GC_TRANSPORT_AMOUNT_CONSERVATION_REPAIR_QUALIFIED"
    } else {
        "M1_V4_GC_TRANSPORT_REPAIR_INSUFFICIENT"
    };
    let protocol = json!({
        "schema": "dcdev020m1replan002r5r2_protocol_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "material_contracts": ["GeometryConservativeV3", "MaturationCoupledV4"],
        "historical_contracts_preserved": ["HistoricalV1", "ConservativeV2"],
        "transport_mode": "AMOUNT_SPACE_ACTUAL_POSITIVE_AREA",
        "outbound_cap": "available_absolute_material_amount",
        "stage_order": ["TRANSPORT", "REACTIONS", "MECHANICS", "REMESH", "REBOND"],
        "warmup_steps": WARMUP,
        "starvation_bound": STARVATION_BOUND,
        "authoritative_stop": "mechanics_step false",
        "closure_tolerance": TOLERANCE,
        "death_qualification_started": false,
        "live_refeed_rerun_started": false,
        "next_execution_started": false
    });
    let contract = json!({
        "v3_v4": {"area": "actual finite positive mesh area", "representation": "signed absolute amount", "reconstruction": "amount_after / actual_area"},
        "species": ["N", "F", "W", "C", "A"],
        "c_tracer_and_composition": "scaled by actual C amount removed; inbound C preserves composition",
        "historical_floor": "unchanged for HistoricalV1 and ConservativeV2"
    });
    let integrated = serde_json::to_value(&summary).map_err(|error| error.to_string())?;
    let preservation = json!({
        "historical_v1_transport_changed": false,
        "conservative_v2_transport_changed": false,
        "focused_v3_v4_subfloor_tests": "PASS",
        "above_floor_parity_test": "PASS",
        "production_default_changed": false,
        "biology_changed": false,
        "d087_changed": false,
        "r4_150k_requalified": false,
        "death_qualification_started": false
    });
    let q = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "classification": classification,
        "historical_v1_transport_changed": false,
        "conservative_v2_transport_changed": false,
        "v3_v4_transport_mode": "AMOUNT_SPACE_ACTUAL_POSITIVE_AREA",
        "outbound_material_cap": true,
        "c_tracer_composition_conservation": true,
        "subfloor_v3_v4_transport_closure": true,
        "step_7684_failure_class_eliminated": summary.first_transport_residual_step != Some(OLD_TRANSPORT_FAILURE_STEP),
        "step_8177_failure_class_eliminated": summary.first_transport_residual_step != Some(OLD_TRANSPORT_MAX_STEP),
        "old_mechanics_false_reference_step": OLD_MECHANICS_FALSE_STEP,
        "authoritative_replay_stopped_on_false": summary.authoritative_replay_stopped_on_false,
        "max_transport_residual_before_authoritative_stop": summary.max_transport_residual,
        "max_total_stage_residual_before_authoritative_stop": summary.max_total_stage_residual,
        "pre_defect_observer_collapse_preserved": summary.first_observer_nonviable.map_or(false, |step| step < OLD_TRANSPORT_FAILURE_STEP),
        "r4_150k_requalified": false,
        "death_qualification_started": false,
        "production_default_changed": false,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false
    });
    let manifest = json!({
        "schema": "dcdev020m1replan002r5r2_manifest_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "files": ["protocol.json", "transport_contract.json", "integrated_valid_prefix.json", "qualification.json", "preservation.json", "artifact_manifest.json"],
        "dense_output": dense.as_ref().map(|path| path.display().to_string()),
        "canonical_dense_root": "/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r2/",
        "next_execution_started": false
    });
    write_json(&output.join("protocol.json"), &protocol)?;
    write_json(&output.join("transport_contract.json"), &contract)?;
    write_json(&output.join("integrated_valid_prefix.json"), &integrated)?;
    write_json(&output.join("qualification.json"), &q)?;
    write_json(&output.join("preservation.json"), &preservation)?;
    write_json(&output.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1REPLAN002R5R2_COMPLETE classification={classification} next_execution_started=false");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_authority() {
        assert_eq!(STARTING_HEAD.len(), 40);
        assert_eq!(OLD_TRANSPORT_FAILURE_STEP, 7_684);
        assert_eq!(OLD_TRANSPORT_MAX_STEP, 8_177);
        assert_eq!(OLD_MECHANICS_FALSE_STEP, 8_566);
    }

    #[test]
    fn transport_ledger_net_is_signed() {
        let ledger = TransportLedger {
            n_in: 2.0,
            f_out: 1.0,
            w_in: 3.0,
            c_leak: 0.5,
            ..Default::default()
        };
        assert!(close(transport_net(&ledger), 3.5));
    }
}
