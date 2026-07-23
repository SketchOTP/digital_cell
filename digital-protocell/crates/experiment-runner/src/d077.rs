//! D-077 cooperative surface condensation architecture review runner.
//!
//! Observer / reduced-model only. Writes gate artifacts; does not change production chemistry.

use crate::d013::atomic_write_json;
use chemistry_core::d077_analysis::*;
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
        if name != "d077" {
            return;
        }
        let target = archive_root.join("d077");
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
        let local = resolve_path(Path::new("experiments/generated/d077"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(
        &out,
        "preservation",
        &serde_json::to_value(&review.frozen_preservation)?,
    )?;
    write_gate(
        &out,
        "lineage_audit",
        &serde_json::to_value(&review.gate0)?,
    )?;
    write_gate(
        &out,
        "thermodynamics",
        &serde_json::to_value(&review.gate1)?,
    )?;
    write_gate(
        &out,
        "cohesion_reconstruction",
        &serde_json::to_value(&review.gate2)?,
    )?;
    write_gate(
        &out,
        "metabolic_feasibility",
        &serde_json::to_value(&review.gate3)?,
    )?;
    write_gate(
        &out,
        "replacement",
        &serde_json::to_value(&review.gate4)?,
    )?;
    write_gate(
        &out,
        "damage_controls",
        &serde_json::to_value(&review.gate5)?,
    )?;
    write_gate(
        &out,
        "stability",
        &serde_json::to_value(&review.gate6)?,
    )?;
    write_gate(
        &out,
        "radius_portability",
        &serde_json::to_value(&review.gate7)?,
    )?;
    write_gate(
        &out,
        "route_selection",
        &serde_json::to_value(&review.route)?,
    )?;

    let result = json!({
        "project_directive": D077_PROJECT_ID,
        "agent_memory_directive": D077_AGENT_MEMORY_ID,
        "starting_tag": D077_STARTING_TAG,
        "starting_commit": "d82628f",
        "ending_commit_at_run": commit,
        "d075_conclusion": "D075_FROZEN_EXCHANGE_METABOLICALLY_UNREACHABLE",
        "d076_conclusion": D076_CONCLUSION,
        "energy_cycle_record": ENERGY_CYCLE_RECORD,
        "passive_record": PASSIVE_RECORD,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "scientific_conclusion": review.route.scientific_conclusion,
        "selected_chi": review.route.selected_chi,
        "chi_span": review.gate2.chi_span_095,
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "next_directive": review.route.next_directive,
        "next_execution_started": review.route.next_execution_started,
        "production_biology_unchanged": true,
        "gates": {
            "gate0_pass": review.gate0.pass,
            "gate1_pass": review.gate1.pass,
            "gate2_pass": review.gate2.pass,
            "gate3_pass": review.gate3.pass,
            "gate4_pass": review.gate4.pass,
            "gate5_pass": review.gate5.pass,
            "gate6_pass": review.gate6.pass,
            "gate7_pass": review.gate7.pass,
        },
        "candidate_equations": review.candidate_equations,
        "reasons": review.route.reasons,
    });
    atomic_write_json(&out.join("result.json"), &result)?;

    let manifest = json!({
        "project_directive": D077_PROJECT_ID,
        "agent_memory_directive": D077_AGENT_MEMORY_ID,
        "branch": "d008-membrane-metabolic-closure",
        "starting_tag": D077_STARTING_TAG,
        "source_commit": commit,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "selected_chi": review.route.selected_chi,
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "energy_cycle_record": ENERGY_CYCLE_RECORD,
        "artifacts": [
            "preservation",
            "lineage_audit",
            "thermodynamics",
            "cohesion_reconstruction",
            "metabolic_feasibility",
            "replacement",
            "damage_controls",
            "stability",
            "radius_portability",
            "route_selection",
            "result.json"
        ],
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    Ok(result)
}
