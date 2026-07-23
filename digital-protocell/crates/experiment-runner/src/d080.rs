//! D-080 geometry-consistent edge-network repair runner.

use crate::d013::atomic_write_json;
use chemistry_core::d080_analysis::*;
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
        if name != "d080" {
            return;
        }
    }
    let target = archive_root.join("d080");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::create_dir_all(&target);
    #[cfg(unix)]
    {
        let _ = std::os::unix::fs::symlink(&target, path);
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
        let local = resolve_path(Path::new("experiments/generated/d080"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;
    for sub in [
        "preservation",
        "d079_reproduction",
        "gap_provenance",
        "cut_cell_support",
        "geometry_qualification",
        "self_assembly",
        "transport",
        "replacement",
        "damage_repair",
        "dynamic_interface",
        "coupled_requalification",
        "accounting",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    // Preserve D-079 failure record pointer.
    write_gate(
        &out,
        "preservation",
        &json!({
            "d079_conclusion": D079_CONCLUSION,
            "d079_pending_audit": D079_PENDING_AUDIT,
            "d079_tag": D080_STARTING_TAG,
            "d079_commit": D080_STARTING_COMMIT,
            "d079_artifacts": "experiments/generated/d079/",
            "unchanged": true,
        }),
    )?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(
        &out,
        "d079_reproduction",
        &serde_json::to_value(&review.gate0)?,
    )?;
    write_gate(
        &out,
        "gap_provenance",
        &serde_json::to_value(&review.gate1)?,
    )?;
    write_gate(
        &out,
        "cut_cell_support",
        &json!({
            "note": review.gate2_note,
            "module": "chemistry-core/src/edge_support.rs",
            "interior_rule": "phi > 0.5 strict",
            "saddle_rule": "sw+ne >= se+nw",
        }),
    )?;
    write_gate(
        &out,
        "geometry_qualification",
        &serde_json::to_value(&review.gate3)?,
    )?;
    write_gate(
        &out,
        "self_assembly",
        &serde_json::to_value(&review.gate4)?,
    )?;
    write_gate(&out, "transport", &serde_json::to_value(&review.gate5)?)?;
    write_gate(
        &out,
        "replacement",
        &serde_json::to_value(&review.gate6)?,
    )?;
    write_gate(
        &out,
        "damage_repair",
        &serde_json::to_value(&review.gate7)?,
    )?;
    write_gate(
        &out,
        "dynamic_interface",
        &serde_json::to_value(&review.gate8)?,
    )?;
    write_gate(
        &out,
        "coupled_requalification",
        &serde_json::to_value(&review.gate9)?,
    )?;
    write_gate(
        &out,
        "accounting",
        &json!({
            "k_lateral_scale": review.k_lateral_scale,
            "params": review.params,
            "frozen_d079_kinetics": true,
        }),
    )?;

    let result = json!({
        "project_directive": D080_PROJECT_ID,
        "agent_memory_directive": D080_AGENT_MEMORY_ID,
        "starting_commit": D080_STARTING_COMMIT,
        "starting_tag": D080_STARTING_TAG,
        "ending_commit_at_run": commit,
        "d079_conclusion": D079_CONCLUSION,
        "d079_pending_audit": D079_PENDING_AUDIT,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "stopped_at_gate": review.route.stopped_at_gate,
        "scientific_conclusion": review.route.scientific_conclusion,
        "next_directive": review.route.next_directive,
        "next_execution_started": false,
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "scope_amendment": review.scope_amendment,
        "k_lateral_scale": review.k_lateral_scale,
        "gates": {
            "gate0": review.gate0.pass,
            "gate1": review.gate1.pass,
            "gate3": review.gate3.pass,
            "gate4": review.gate4.pass,
            "gate5": review.gate5.pass,
            "gate6": review.gate6.pass,
            "gate7": review.gate7.pass,
            "gate8": review.gate8.pass,
            "gate9": review.gate9.pass,
        },
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": "D-080",
            "conclusion": review.route.conclusion,
            "artifacts": [
                "preservation", "d079_reproduction", "gap_provenance", "cut_cell_support",
                "geometry_qualification", "self_assembly", "transport", "replacement",
                "damage_repair", "dynamic_interface", "coupled_requalification", "accounting",
                "result.json"
            ],
        }),
    )?;
    Ok(result)
}
