//! D-036 membrane-bound catalytic complex — Gate 0 parity first.

use chemistry_core::d036_analysis::{
    gate0_parity_audit, gate1_architecture_review, Gate0ParityAudit,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260719-1312-d036-membrane-bound-catalytic-complex";

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn compact_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    crate::d013::atomic_write_bytes(path, &serde_json::to_vec(value)?)
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn audit_to_json(a: &Gate0ParityAudit) -> Value {
    serde_json::to_value(a).unwrap_or_else(|_| json!({"error": "serialize_failed"}))
}

pub fn run_gate0_parity(
    output: &Path,
    gate5_advance: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let audit = gate0_parity_audit(gate5_advance);
    let body = json!({
        "project_directive": "D-036",
        "agent_memory_id": AGENT_MEMORY_ID,
        "gate": 0,
        "gate5_advance_accepted": gate5_advance,
        "source_commit": git_commit_hash(),
        "audit": audit_to_json(&audit),
        "pass": audit.pass,
        "conclusion": audit.conclusion,
        "mature_membrane_autocatalysis_rejected": audit.mature_membrane_autocatalysis_rejected,
    });
    compact_write_json(&output.join("parity_audit.json"), &body)?;

    let mut summary_audit = audit_to_json(&audit);
    if let Some(obj) = summary_audit.as_object_mut() {
        obj.insert(
            "local_sample_count".into(),
            json!(audit.local_samples.len()),
        );
        obj.insert(
            "local_fail_count".into(),
            json!(audit.local_samples.iter().filter(|s| !s.ok).count()),
        );
        obj.remove("local_samples");
    }
    compact_write_json(
        &output.join("parity_summary.json"),
        &json!({
            "project_directive": "D-036",
            "gate": 0,
            "pass": audit.pass,
            "conclusion": audit.conclusion,
            "audit": summary_audit,
        }),
    )?;
    Ok(body)
}

pub fn run_gate1_architecture(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let review = gate1_architecture_review();
    let body = json!({
        "project_directive": "D-036",
        "agent_memory_id": AGENT_MEMORY_ID,
        "gate": 1,
        "source_commit": git_commit_hash(),
        "review": serde_json::to_value(&review)?,
        "pass": review.pass,
        "conclusion": review.conclusion,
    });
    compact_write_json(&output.join("architecture_review.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let preservation = json!({
        "project_directive": "D-036",
        "agent_memory_id": AGENT_MEMORY_ID,
        "starting_branch": "d008-membrane-metabolic-closure",
        "d035_tag": "D-035-catalytic-assembly-fail",
        "d035_conclusion": "D035_ISOLATED_CATALYTIC_RENEWAL_FAILURE",
        "record": "MATURE_MEMBRANE_AUTOCATALYSIS_REJECTED",
        "source_commit": git_commit_hash(),
    });
    compact_write_json(&out.join("preservation/preservation.json"), &preservation)?;

    let g0 = run_gate0_parity(&out.join("d035_parity"), 2500)?;
    let pass0 = g0["pass"].as_bool().unwrap_or(false);
    let conclusion0 = g0["conclusion"].as_str().unwrap_or("D036_FAIL").to_string();
    if !pass0 || conclusion0 == "D036_D035_RATE_PARITY_DEFECT" {
        let manifest = json!({
            "project_directive": "D-036",
            "agent_memory_id": AGENT_MEMORY_ID,
            "stopped_at_gate": 0,
            "pass": false,
            "conclusion": "D036_D035_RATE_PARITY_DEFECT",
            "phase": "parity_defect",
            "source_commit": git_commit_hash(),
            "gates": {"gate0": g0},
        });
        compact_write_json(&out.join("manifest.json"), &manifest)?;
        compact_write_json(&out.join("result.json"), &manifest)?;
        return Ok(manifest);
    }

    let g1 = run_gate1_architecture(&out.join("architecture_review"))?;
    let pass1 = g1["pass"].as_bool().unwrap_or(false);
    let conclusion1 = g1["conclusion"].as_str().unwrap_or("D036_FAIL").to_string();
    let (pass, conclusion, phase, stopped) = if pass1 {
        (
            true,
            conclusion1,
            "v13_implementation_authorized",
            Value::Null,
        )
    } else {
        (
            false,
            "D036_CATALYTIC_COMPLEX_ARCHITECTURE_REJECTED".into(),
            "architecture_rejected",
            json!(1),
        )
    };

    let manifest = json!({
        "project_directive": "D-036",
        "agent_memory_id": AGENT_MEMORY_ID,
        "stopped_at_gate": stopped,
        "pass": pass,
        "conclusion": conclusion,
        "phase": phase,
        "d035_parity_conclusion": conclusion0,
        "source_commit": git_commit_hash(),
        "gates": {"gate0": g0, "gate1": g1},
    });
    compact_write_json(&out.join("manifest.json"), &manifest)?;
    compact_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
