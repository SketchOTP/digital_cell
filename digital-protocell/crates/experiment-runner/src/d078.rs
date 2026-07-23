//! D-078 Phase 1 boundary substrate redesign downselect runner.
//!
//! Architecture review only. Writes gate artifacts; does not change production chemistry.

use crate::d013::atomic_write_json;
use chemistry_core::d078_analysis::*;
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
        if name != "d078" {
            return;
        }
        let target = archive_root.join("d078");
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
        let local = resolve_path(Path::new("experiments/generated/d078"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(
        &out,
        "preservation",
        &serde_json::to_value(&review.preservation)?,
    )?;
    write_gate(&out, "lineage_audit", &serde_json::to_value(&review.gate0)?)?;
    write_gate(
        &out,
        "structure_native",
        &serde_json::to_value(&review.candidate_a)?,
    )?;
    write_gate(
        &out,
        "single_amphiphile",
        &serde_json::to_value(&review.candidate_b)?,
    )?;
    write_gate(
        &out,
        "conservation",
        &json!({
            "candidate_a": review.candidate_a.gate1,
            "candidate_b": review.candidate_b.gate1,
        }),
    )?;
    write_gate(
        &out,
        "coupled_feasibility",
        &json!({
            "candidate_a": review.candidate_a.gate2,
            "candidate_b": review.candidate_b.gate2,
            "energy_budgets": review.energy_budgets,
        }),
    )?;
    write_gate(
        &out,
        "structural_stability",
        &json!({
            "candidate_a": review.candidate_a.gate3,
            "candidate_b": review.candidate_b.gate3,
        }),
    )?;
    write_gate(
        &out,
        "boundary_function",
        &json!({
            "candidate_a": review.candidate_a.gate4,
            "candidate_b": review.candidate_b.gate4,
        }),
    )?;
    write_gate(
        &out,
        "repair_controls",
        &json!({
            "candidate_a": review.candidate_a.gate5,
            "candidate_b": review.candidate_b.gate5,
        }),
    )?;
    write_gate(
        &out,
        "complexity",
        &json!({
            "candidate_a": review.candidate_a.gate6,
            "candidate_b": review.candidate_b.gate6,
        }),
    )?;
    write_gate(
        &out,
        "route_selection",
        &serde_json::to_value(&review.route)?,
    )?;

    let result = json!({
        "project_directive": D078_PROJECT_ID,
        "agent_memory_directive": D078_AGENT_MEMORY_ID,
        "starting_tag": D078_STARTING_TAG,
        "starting_commit": D078_STARTING_COMMIT,
        "ending_commit_at_run": commit,
        "ps_architecture_record": PS_ARCHITECTURE_RECORD,
        "d077_conclusion": D077_CONCLUSION,
        "d076_conclusion": D076_CONCLUSION,
        "energy_cycle_record": ENERGY_CYCLE_RECORD,
        "passive_record": PASSIVE_RECORD,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "scientific_conclusion": review.route.scientific_conclusion,
        "selected_candidate": review.route.selected_candidate,
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "next_directive": review.route.next_directive,
        "next_execution_started": review.route.next_execution_started,
        "production_biology_unchanged": true,
        "gates": {
            "gate0_pass": review.gate0.pass,
            "a_science_pass": review.candidate_a.science_pass,
            "b_science_pass": review.candidate_b.science_pass,
            "a_gate1": review.candidate_a.gate1.pass,
            "a_gate2": review.candidate_a.gate2.pass,
            "a_gate3": review.candidate_a.gate3.pass,
            "a_gate4": review.candidate_a.gate4.pass,
            "a_gate5": review.candidate_a.gate5.pass,
            "b_gate1": review.candidate_b.gate1.pass,
            "b_gate2": review.candidate_b.gate2.pass,
            "b_gate3": review.candidate_b.gate3.pass,
            "b_gate4": review.candidate_b.gate4.pass,
            "b_gate5": review.candidate_b.gate5.pass,
        },
        "complexity": {
            "a_total": review.candidate_a.gate6.total,
            "b_total": review.candidate_b.gate6.total,
        },
        "reasons": review.route.reasons,
    });
    atomic_write_json(&out.join("result.json"), &result)?;

    let manifest = json!({
        "project_directive": D078_PROJECT_ID,
        "agent_memory_directive": D078_AGENT_MEMORY_ID,
        "branch": "d008-membrane-metabolic-closure",
        "starting_tag": D078_STARTING_TAG,
        "starting_commit": D078_STARTING_COMMIT,
        "source_commit": commit,
        "primary_conclusion": review.route.conclusion,
        "route": review.route.route.as_str(),
        "ps_architecture_record": PS_ARCHITECTURE_RECORD,
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "artifacts": [
            "preservation",
            "lineage_audit",
            "structure_native",
            "single_amphiphile",
            "conservation",
            "coupled_feasibility",
            "structural_stability",
            "boundary_function",
            "repair_controls",
            "complexity",
            "route_selection",
            "result.json"
        ],
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    Ok(result)
}
