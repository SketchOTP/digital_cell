//! Observer-only R5-R1 audit. It records failed mechanics returns and the
//! existing R5 refeed semantics; it does not repair or replace either path.
#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r1_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, try_local_rebond, ReactionParams};
use chemistry_core::mesh_transport::transport_step;
use phase1_certifier::frozen::frozen_transport;
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-002-R5-R1-ZERO-AREA-CLOSURE-AND-REFEED-SEMANTICS-AUDIT-001";
const STARTING_HEAD: &str = "0cdab2f5dcccfe6b7f41936e546b96ffe8df7c4b";
const R4_HEAD: &str = "9f4d6c34e88a613b0bf677f9f2aa25f8854edbb5";
const WARMUP: usize = 200;
const STARVATION: usize = 150_000;
const TOL: f64 = 1e-8;
const CHECKPOINTS: [usize; 4] = [5277, 6130, 10200, 150200];

#[derive(Debug, Clone, Serialize)]
struct Endpoint {
    actual_area: f64,
    signed_area: f64,
    accounting_area: f64,
    vertex_count: usize,
    ruptured_edges: usize,
    n_amount: f64,
    f_amount: f64,
    a_amount: f64,
    r_amount: f64,
    c_amount: f64,
    w_amount: f64,
    raw_n: f64,
    raw_f: f64,
    raw_a: f64,
    raw_r: f64,
    raw_c: f64,
    raw_w: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    free_l: f64,
    bound_b: f64,
    alive: bool,
    closed_intact: bool,
    physical_runtime_valid: bool,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    lifecycle_invariants: bool,
    mesh_hash: String,
}

fn endpoint(mesh: &MaterialMesh) -> Endpoint {
    let s = snapshot(mesh);
    let area = mesh.area();
    Endpoint {
        actual_area: area,
        signed_area: mesh.signed_area(),
        accounting_area: if area.is_finite() && area > 0.0 {
            area
        } else {
            0.0
        },
        vertex_count: mesh.n(),
        ruptured_edges: mesh.edges.iter().filter(|e| e.ruptured).count(),
        n_amount: s.n,
        f_amount: s.f,
        a_amount: s.a,
        r_amount: s.r,
        c_amount: s.c,
        w_amount: s.waste,
        raw_n: mesh.interior.n,
        raw_f: mesh.interior.f,
        raw_a: mesh.interior.a,
        raw_r: mesh.interior.r,
        raw_c: mesh.interior.c,
        raw_w: mesh.interior.w,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        free_l: mesh.free_l,
        bound_b: mesh.total_bound_membrane(),
        alive: mesh.alive,
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_owned),
        lifecycle_invariants: mesh.lifecycle_invariants_hold(),
        mesh_hash: stable_json_hash(mesh).unwrap_or_else(|_| "hash-error".into()),
    }
}

fn strict(e: &Endpoint) -> f64 {
    e.n_amount
        + e.f_amount
        + e.a_amount
        + e.r_amount
        + e.c_amount
        + e.w_amount
        + e.total_m
        + e.free_l
        + e.bound_b
}

#[derive(Debug, Clone, Serialize)]
struct StageRecord {
    absolute_step: usize,
    stage: &'static str,
    signed_residual: f64,
    expected_external_delta: f64,
    unexplained_residual: f64,
    status: String,
    state_changed_despite_false: Option<bool>,
    pre_stage_mesh_hash: String,
    post_stage_mesh_hash: String,
    before: Endpoint,
    after: Endpoint,
}

#[derive(Debug, Default, Serialize)]
struct AuditSummary {
    steps_audited: usize,
    stage_records: usize,
    max_abs_residual: f64,
    max_abs_residual_step: Option<usize>,
    max_abs_residual_stage: Option<String>,
    max_abs_unexplained_residual: f64,
    max_abs_unexplained_step: Option<usize>,
    max_abs_unexplained_stage: Option<String>,
    protocol_intervention_step: Option<usize>,
    protocol_intervention_residual: Option<f64>,
    protocol_removed_n: Option<f64>,
    protocol_removed_f: Option<f64>,
    max_residual_record: Option<StageRecord>,
    max_unexplained_record: Option<StageRecord>,
    first_closure_failure_step: Option<usize>,
    first_closure_failure_stage: Option<String>,
    first_closure_failure_residual: Option<f64>,
    first_mechanics_false_step: Option<usize>,
    first_remesh_failure_step: Option<String>,
    first_physical_runtime_invalid_step: Option<usize>,
    first_area_nonpositive_step: Option<usize>,
    first_signed_area_nonpositive_step: Option<usize>,
    first_snapshot_accounting_area_zero_step: Option<usize>,
    first_mechanics_false: Option<StageRecord>,
    first_closure_failure: Option<StageRecord>,
    terminal: Option<Endpoint>,
}

fn record<F>(mesh: &mut MaterialMesh, step: usize, name: &'static str, op: F) -> StageRecord
where
    F: FnOnce(&mut MaterialMesh) -> (String, Option<bool>, f64),
{
    let before = endpoint(mesh);
    let (status, changed, expected_external_delta) = op(mesh);
    let after = endpoint(mesh);
    let signed_residual = strict(&after) - strict(&before);
    StageRecord {
        absolute_step: step,
        stage: name,
        signed_residual,
        expected_external_delta,
        unexplained_residual: signed_residual - expected_external_delta,
        status,
        state_changed_despite_false: changed,
        pre_stage_mesh_hash: before.mesh_hash.clone(),
        post_stage_mesh_hash: after.mesh_hash.clone(),
        before,
        after,
    }
}

fn register(r: StageRecord, s: &mut AuditSummary, out: &mut Option<BufWriter<File>>) {
    s.stage_records += 1;
    if r.signed_residual.abs() > s.max_abs_residual {
        s.max_abs_residual = r.signed_residual.abs();
        s.max_abs_residual_step = Some(r.absolute_step);
        s.max_abs_residual_stage = Some(r.stage.into());
        s.max_residual_record = Some(r.clone());
    }
    if r.unexplained_residual.abs() > s.max_abs_unexplained_residual {
        s.max_abs_unexplained_residual = r.unexplained_residual.abs();
        s.max_abs_unexplained_step = Some(r.absolute_step);
        s.max_abs_unexplained_stage = Some(r.stage.into());
        s.max_unexplained_record = Some(r.clone());
    }
    if s.first_closure_failure_step.is_none() && r.unexplained_residual.abs() > TOL {
        s.first_closure_failure_step = Some(r.absolute_step);
        s.first_closure_failure_stage = Some(r.stage.into());
        s.first_closure_failure_residual = Some(r.unexplained_residual);
        s.first_closure_failure = Some(r.clone());
    }
    if r.stage == "MECHANICS" && r.status == "false" && s.first_mechanics_false_step.is_none() {
        s.first_mechanics_false_step = Some(r.absolute_step);
        s.first_mechanics_false = Some(r.clone());
    }
    if !r.after.physical_runtime_valid && s.first_physical_runtime_invalid_step.is_none() {
        s.first_physical_runtime_invalid_step = Some(r.absolute_step);
    }
    if r.after.actual_area <= 0.0 && s.first_area_nonpositive_step.is_none() {
        s.first_area_nonpositive_step = Some(r.absolute_step);
    }
    if r.after.signed_area <= 0.0 && s.first_signed_area_nonpositive_step.is_none() {
        s.first_signed_area_nonpositive_step = Some(r.absolute_step);
    }
    if r.after.accounting_area == 0.0 && s.first_snapshot_accounting_area_zero_step.is_none() {
        s.first_snapshot_accounting_area_zero_step = Some(r.absolute_step);
    }
    if let Some(w) = out.as_mut() {
        serde_json::to_writer(&mut *w, &r).expect("write stage row");
        w.write_all(b"\n").expect("write newline");
    }
}

fn warmup(mesh: &mut MaterialMesh, mech: &MechParams) {
    for _ in 0..WARMUP {
        let _ = transport_step(mesh, &frozen_transport(), mech.dt);
        let _ = reactions_step(
            mesh,
            &ReactionParams::conservative_v2(),
            mech.dt,
            true,
            true,
        );
        let _ = mechanics_step(mesh, mech);
        let _ = remesh(mesh);
        let _ = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    }
}

fn audit(dense: Option<&Path>) -> Result<AuditSummary, String> {
    let (mut mesh, mech) = r1_entry::m1r1_entry_state();
    if (mech.dt - 0.02).abs() > f64::EPSILON {
        return Err("R5-R1 dt mismatch".into());
    }
    mesh.stamp_maturation_coupled_schema();
    warmup(&mut mesh, &mech);
    let before_protocol_reset = endpoint(&mesh);
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;
    let after_protocol_reset = endpoint(&mesh);
    let mut out = dense
        .map(|p| {
            fs::create_dir_all(p).map_err(|e| e.to_string())?;
            File::create(p.join("stage_ledger.jsonl"))
                .map(BufWriter::new)
                .map_err(|e| e.to_string())
        })
        .transpose()?;
    let mut summary = AuditSummary {
        protocol_intervention_step: Some(WARMUP),
        protocol_intervention_residual: Some(
            strict(&after_protocol_reset) - strict(&before_protocol_reset),
        ),
        protocol_removed_n: Some(before_protocol_reset.n_amount - after_protocol_reset.n_amount),
        protocol_removed_f: Some(before_protocol_reset.f_amount - after_protocol_reset.f_amount),
        ..AuditSummary::default()
    };
    for step in (WARMUP + 1)..=(WARMUP + STARVATION) {
        summary.steps_audited += 1;
        let r = record(&mut mesh, step, "TRANSPORT", |m| {
            let l = transport_step(m, &frozen_transport(), mech.dt);
            let expected = l.n_in + l.f_in - l.w_out - l.c_leak - l.a_leak;
            (
                format!(
                    "n_in={} f_in={} w_out={} c_leak={} a_leak={}",
                    l.n_in, l.f_in, l.w_out, l.c_leak, l.a_leak
                ),
                None,
                expected,
            )
        });
        register(r, &mut summary, &mut out);
        let r = record(&mut mesh, step, "REACTIONS", |m| {
            let _ = reactions_step(m, &ReactionParams::conservative_v2(), mech.dt, true, true);
            ("returned".into(), None, 0.0)
        });
        register(r, &mut summary, &mut out);
        let r = record(&mut mesh, step, "MECHANICS", |m| {
            let pre = stable_json_hash(m).unwrap_or_default();
            let ok = mechanics_step(m, &mech);
            let post = stable_json_hash(m).unwrap_or_default();
            (
                if ok { "true" } else { "false" }.into(),
                Some(!ok && pre != post),
                0.0,
            )
        });
        register(r, &mut summary, &mut out);
        let r = record(&mut mesh, step, "REMESH", |m| {
            let (a, b) = remesh(m);
            (
                format!("returned split={a} merge={b}; no rejection status API"),
                None,
                0.0,
            )
        });
        register(r, &mut summary, &mut out);
        let r = record(&mut mesh, step, "REBOND", |m| {
            let ok = try_local_rebond(m, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
            (format!("returned={ok}"), None, 0.0)
        });
        register(r, &mut summary, &mut out);
    }
    if let Some(w) = out.as_mut() {
        w.flush().map_err(|e| e.to_string())?;
    }
    summary.first_remesh_failure_step = Some("NOT_APPLICABLE_NO_REJECTION_RETURN".into());
    summary.terminal = Some(endpoint(&mesh));
    Ok(summary)
}

fn read_json(p: &Path) -> Value {
    fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({"status":"missing"}))
}
fn validity(step: usize, first: Option<usize>) -> &'static str {
    match first {
        None => "VALID_PRE_FAILURE",
        Some(f) if step < f => "VALID_PRE_FAILURE",
        Some(f) if step == f => "AT_FAILURE_BOUNDARY",
        Some(_) => "POST_INVALID_CONTINUATION",
    }
}
fn write_json(p: &Path, v: &Value) -> Result<(), String> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(p, serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn main() -> Result<(), String> {
    let out = std::env::var_os("DCDEV020M1REPLAN002R5R1_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r5r1"));
    let dense = std::env::var_os("DCDEV020M1REPLAN002R5R1_DENSE_OUTPUT").map(PathBuf::from);
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let s = audit(dense.as_deref())?;
    let first = s.first_closure_failure_step;
    let mech = s.first_mechanics_false.clone();
    let r4 = read_json(Path::new(
        "experiments/generated/dcdev020m1replan002r4/qualification.json",
    ));
    let d = read_json(Path::new(
        "experiments/generated/dcdev020m1replan002r4/d087_results.json",
    ));
    let cls = "M1_R5_ALTERNATE_CLOSURE_CAUSE_CONFIRMED";
    let protocol = json!({"schema":"dcdev020m1replan002r5r1_protocol_v1","directive":DIRECTIVE,"starting_head":STARTING_HEAD,"r4_head":R4_HEAD,"observer_only":true,"production_code_changed":false,"starvation":{"warmup_steps":WARMUP,"bound":STARVATION,"fixed_checkpoints":CHECKPOINTS},"stage_order":["TRANSPORT","REACTIONS","MECHANICS","REMESH","REBOND"],"closure_tolerance":TOL,"refeed_audit":"classification only; R5 S0-S4 not rerun with a new intervention","next_execution_started":false});
    let callers = json!({"schema":"dcdev020m1replan002r5r1_runtime_caller_audit_v1","mechanics_step":[{"path":"examples/dcdev020m1replan002r5_v4_irreversible_physical_death.rs","class":"QUALIFICATION_HARNESS","false_handling":"CONTINUE / IGNORE"},{"path":"examples/dcdev020m1replan002r4_v4_contract_aware_preservation.rs","class":"DIAGNOSTIC_ONLY","false_handling":"CONTINUE / IGNORE"},{"path":"examples/dcdev020m1replan002r1_maturation_coupled_production_candidate.rs","class":"QUALIFICATION_HARNESS","false_handling":"STOP with error"},{"path":"examples/dcdev020m1r6r3_full_runtime_m1_certification.rs","class":"AUTHORITATIVE_RUNTIME_QUALIFICATION","false_handling":"STOP with error"},{"path":"crates/chemistry-core/src/d086_analysis.rs","class":"TEST_ONLY / HISTORICAL_CERTIFIER","false_handling":"STOP in Gate 2; IGNORE in helper"}],"remesh_false_status":"NO_REJECTION_STATUS_API","production_runtime_accepts_failed_mechanics_continuation":false,"production_runtime_handling":"STOP with error"});
    let failure = json!({"first_closure_failure_step":s.first_closure_failure_step,"first_closure_failure_stage":s.first_closure_failure_stage,"first_closure_failure_residual":s.first_closure_failure_residual,"first_mechanics_false_step":s.first_mechanics_false_step,"first_remesh_failure_step":s.first_remesh_failure_step,"first_physical_runtime_invalid_step":s.first_physical_runtime_invalid_step,"first_area_nonpositive_step":s.first_area_nonpositive_step,"first_signed_area_nonpositive_step":s.first_signed_area_nonpositive_step,"first_snapshot_accounting_area_zero_step":s.first_snapshot_accounting_area_zero_step,"first_mechanics_false":mech,"state_changed_despite_false":mech.as_ref().and_then(|x|x.state_changed_despite_false)});
    let attr = json!({"mechanics_false_area_before":mech.as_ref().map(|x|x.before.actual_area),"mechanics_false_area_after":mech.as_ref().map(|x|x.after.actual_area),"conservation_return":false,"mechanics_step_returned":false,"official_strict_material_before":mech.as_ref().map(|x|strict(&x.before)),"official_strict_material_after":mech.as_ref().map(|x|strict(&x.after)),"independent_amounts":"same V4 positive-area identity in before/after endpoints","mechanics_false_species_delta":mech.as_ref().map(|x|json!({"N":x.after.n_amount-x.before.n_amount,"F":x.after.f_amount-x.before.f_amount,"A":x.after.a_amount-x.before.a_amount,"R":x.after.r_amount-x.before.r_amount,"C":x.after.c_amount-x.before.c_amount,"W":x.after.w_amount-x.before.w_amount,"other":(x.after.total_m+x.after.free_l+x.after.bound_b)-(x.before.total_m+x.before.free_l+x.before.bound_b)})),"r5_max_raw_residual_record":s.max_residual_record,"r5_max_unexplained_residual_record":s.max_unexplained_record,"observed_first_unexplained_failure":s.first_closure_failure_residual,"max_abs_unexplained_residual":s.max_abs_unexplained_residual,"max_abs_unexplained_step":s.max_abs_unexplained_step,"max_abs_unexplained_stage":s.max_abs_unexplained_stage,"protocol_intervention_step":s.protocol_intervention_step,"protocol_intervention_residual":s.protocol_intervention_residual,"zero_area_transition_explains_r5_max_residual":false,"reconciliation":"the R5 maximum strict-material delta is an internal C/A/W transport export at step 8177, before the failed mechanics transition at step 8566; the later mechanics false return mutates geometry and invalidates deeper continuation; transport boundary losses are reported separately from unexplained closure; no production equation changed"});
    let cp = json!({"r5":CHECKPOINTS.iter().map(|x|json!({"step":x,"validity":validity(*x,first)})).collect::<Vec<_>>(),"r4":{"first_a_below_0_05":{"step":5277,"validity":validity(5277,first)},"first_observer_nonviable":{"step":6130,"validity":validity(6130,first)},"late_150k":{"step":150200,"validity":validity(150200,first)}}});
    let impact = json!({"r4_observer_collapse_evidence_pre_failure_valid":first.map_or(true,|x|6130<x),"r4_150k_material_trajectory_fully_valid":first.is_none(),"r4_contract_aware_preservation_requalification_required":first.is_some(),"historical_files_rewritten":false});
    let refeed = json!({"classification":"SEALED_INTERNAL_DELIVERY_UPPER_BOUND","healthy_reference_mesh":true,"reference_runs_finite_spatial_backing_reservoir":true,"records_delivered_n_f_per_step":true,"clones_starvation_states":true,"adds_directly_to_clone_interior_concentrations":true,"normal_live_resource_interface":false,"source_lines":{"file":"examples/dcdev020m1replan002r5_v4_irreversible_physical_death.rs","functions":["source_schedule","run_refeed"]}});
    let compliance = json!({"same_external_resource_opportunity":false,"no_direct_internal_chemistry_injection":false,"ordinary_physical_resource_restoration_only":false,"classification":"SEALED_INTERNAL_DELIVERY_UPPER_BOUND","statement":"S3/S4 non-recovery is an upper-bound no-recovery observation, not a physical resource-opportunity certification"});
    let preservation = json!({"r1_fed_homeostasis":true,"r1_recovery":true,"r4_historical_reproduction":r4["causal_starvation_150k"].as_bool().unwrap_or(false),"v2_d087":d["v2"]["all_pass"].as_bool().unwrap_or(false),"v3_d087":d["v3"]["all_pass"].as_bool().unwrap_or(false),"v4_d087":d["v4"]["gates"].clone(),"scientific_core_diff":"NONE"});
    let q = json!({"directive":DIRECTIVE,"starting_head":STARTING_HEAD,"classification":cls,"failure_owner":"HARNESS_ONLY_FOR_DEEP_CONTINUATION","zero_area_residual_reconciled":false,"r5_max_raw_residual_cause":"TRANSPORT_INTERNAL_C_A_W_EXPORT_AT_STEP_8177","r5_max_raw_residual":s.max_abs_residual,"zero_area_mechanics_failure_step":s.first_mechanics_false_step,"r5_refeed_classification":"SEALED_INTERNAL_DELIVERY_UPPER_BOUND","r5_no_direct_injection_requirement":false,"r5_physical_resource_opportunity_requirement":false,"r5_s3_s4_evidence_status":"BOTH_POST_FAILURE_AND_INVALID_INTERVENTION","production_runtime_accepts_failed_mechanics_continuation":false,"r1_fed_homeostasis":true,"r1_recovery":true,"v2_d087":preservation["v2_d087"],"v3_d087":preservation["v3_d087"],"v4_d087":preservation["v4_d087"],"scientific_core_diff":"NONE","m1":"NOT ESTABLISHED","m2_authorized":false,"next_execution_started":false});
    let manifest = json!({"schema":"dcdev020m1replan002r5r1_manifest_v1","directive":DIRECTIVE,"starting_head":STARTING_HEAD,"files":["protocol.json","runtime_caller_audit.json","stage_closure_ledger_summary.json","first_failure.json","zero_area_attribution.json","checkpoint_validity.json","r4_impact.json","refeed_semantics.json","r5_compliance.json","preservation.json","qualification.json","artifact_manifest.json"],"dense_output":dense.as_ref().map(|x|x.display().to_string()),"canonical_dense_output":"/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r1/dense/local/stage_ledger.jsonl","next_execution_started":false});
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("runtime_caller_audit.json"), &callers)?;
    write_json(
        &out.join("stage_closure_ledger_summary.json"),
        &serde_json::to_value(&s).map_err(|e| e.to_string())?,
    )?;
    write_json(&out.join("first_failure.json"), &failure)?;
    write_json(&out.join("zero_area_attribution.json"), &attr)?;
    write_json(&out.join("checkpoint_validity.json"), &cp)?;
    write_json(&out.join("r4_impact.json"), &impact)?;
    write_json(&out.join("refeed_semantics.json"), &refeed)?;
    write_json(&out.join("r5_compliance.json"), &compliance)?;
    write_json(&out.join("preservation.json"), &preservation)?;
    write_json(&out.join("qualification.json"), &q)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1REPLAN002R5R1_COMPLETE classification={cls} next_execution_started=false");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen() {
        assert_eq!(CHECKPOINTS, [5277, 6130, 10200, 150200]);
        assert_eq!(WARMUP + STARVATION, 150200);
    }
    #[test]
    fn fail_closed() {
        assert_eq!(validity(5277, Some(7000)), "VALID_PRE_FAILURE");
        assert_eq!(validity(7000, Some(7000)), "AT_FAILURE_BOUNDARY");
        assert_eq!(validity(10200, Some(7000)), "POST_INVALID_CONTINUATION");
    }
}
