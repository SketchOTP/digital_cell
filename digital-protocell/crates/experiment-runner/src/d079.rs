//! D-079 conserved edge-network membrane feasibility runner.

use crate::d013::atomic_write_json;
use chemistry_core::d079_analysis::*;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn ensure_archive_symlink(path: &Path) {
    if path.exists() {
        return;
    }
    let archive_root = PathBuf::from(
        "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated",
    );
    if let Some(name) = path.file_name() {
        if name != "d079" {
            return;
        }
        let target = archive_root.join("d079");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::create_dir_all(&target);
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(&target, path);
        }
    }
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn write_gate(out: &Path, name: &str, body: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let dir = out.join(name);
    fs::create_dir_all(&dir)?;
    atomic_write_json(&dir.join("result.json"), body)?;
    Ok(())
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    ensure_archive_symlink(&out);
    let out = if out.exists() {
        out
    } else {
        let local = resolve_path(Path::new("experiments/generated/d079"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(
        &out,
        "preservation",
        &serde_json::to_value(&review.gate0)?,
    )?;
    write_gate(&out, "schema", &json!({
        "equation_version": review.gate0.equation_version,
        "field_schema": review.gate0.field_schema,
        "schema_version": review.gate0.schema_version,
        "scope_amendment": review.scope_amendment,
        "params": review.params,
    }))?;
    write_gate(
        &out,
        "conservation",
        &serde_json::to_value(&review.gate1)?,
    )?;
    write_gate(
        &out,
        "self_assembly",
        &serde_json::to_value(&review.gate2)?,
    )?;
    write_gate(&out, "transport", &serde_json::to_value(&review.gate3)?)?;
    write_gate(
        &out,
        "replacement",
        &serde_json::to_value(&review.gate4)?,
    )?;
    write_gate(
        &out,
        "damage_repair",
        &serde_json::to_value(&review.gate5)?,
    )?;
    write_gate(
        &out,
        "resource_controls",
        &serde_json::to_value(&review.gate6)?,
    )?;
    write_gate(
        &out,
        "dynamic_interface",
        &serde_json::to_value(&review.gate7)?,
    )?;
    write_gate(
        &out,
        "coupled_feasibility",
        &serde_json::to_value(&review.gate8)?,
    )?;
    write_gate(
        &out,
        "accounting",
        &json!({
            "gate1": review.gate1,
            "assembly_accounting_rows": review.gate2.rows,
        }),
    )?;
    write_gate(
        &out,
        "route_selection",
        &serde_json::to_value(&review.route)?,
    )?;

    let result = json!({
        "project_directive": D079_PROJECT_ID,
        "agent_memory_directive": D079_AGENT_MEMORY_ID,
        "starting_tag": D079_STARTING_TAG,
        "starting_commit": D079_STARTING_COMMIT,
        "ending_commit_at_run": commit,
        "scope_amendment": SCOPE_AMENDMENT,
        "d078_conclusion": D078_CONCLUSION,
        "equation_version": review.gate0.equation_version,
        "field_schema": review.gate0.field_schema,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "stopped_at_gate": review.route.stopped_at_gate,
        "scientific_conclusion": review.route.scientific_conclusion,
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "next_directive": review.route.next_directive,
        "next_execution_started": review.route.next_execution_started,
        "production_continuum_unchanged": true,
        "gates": {
            "gate0": review.gate0.pass,
            "gate1": review.gate1.pass,
            "gate2": review.gate2.pass,
            "gate3": review.gate3.pass,
            "gate4": review.gate4.pass,
            "gate5": review.gate5.pass,
            "gate6": review.gate6.pass,
            "gate7": review.gate7.pass,
            "gate8": review.gate8.pass,
        },
        "self_assembly_rows": review.gate2.rows,
        "reasons": review.route.reasons,
    });
    atomic_write_json(&out.join("result.json"), &result)?;

    let manifest = json!({
        "project_directive": D079_PROJECT_ID,
        "agent_memory_directive": D079_AGENT_MEMORY_ID,
        "branch": "d008-membrane-metabolic-closure",
        "starting_tag": D079_STARTING_TAG,
        "starting_commit": D079_STARTING_COMMIT,
        "source_commit": commit,
        "scope_amendment": SCOPE_AMENDMENT,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "artifacts": [
            "preservation",
            "schema",
            "conservation",
            "self_assembly",
            "transport",
            "replacement",
            "damage_repair",
            "resource_controls",
            "dynamic_interface",
            "coupled_feasibility",
            "accounting",
            "route_selection",
            "result.json"
        ],
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    Ok(result)
}
