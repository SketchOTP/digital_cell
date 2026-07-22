//! D-076 nonequilibrium surface-state cycle architecture review runner.
//!
//! Observer / reduced-model only. Writes gate artifacts; does not change production chemistry.

use crate::d013::atomic_write_json;
use chemistry_core::d076_analysis::*;
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
        if name != "d076" {
            return;
        }
        let target = archive_root.join("d076");
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
        // Fallback to local path if symlink failed.
        let local = resolve_path(Path::new("experiments/generated/d076"));
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
        "conservation",
        &serde_json::to_value(&review.gate1)?,
    )?;
    write_gate(
        &out,
        "reduced_model",
        &json!({
            "candidate_equations": review.candidate_equations,
            "measured_family": review.gate2.family,
            "r_required": review.gate2.r_required,
            "endogenous_interface_p": D075_ENDOGENOUS_INTERFACE_P,
            "endogenous_theta_eq_passive": D075_ENDOGENOUS_THETA_EQ,
            "constitutive_a_retention": D075_CONSTITUTIVE_A_RETENTION,
            "record": PASSIVE_RECORD,
        }),
    )?;
    write_gate(
        &out,
        "fixed_points",
        &serde_json::to_value(&review.gate2)?,
    )?;
    write_gate(
        &out,
        "energy_budget",
        &serde_json::to_value(&review.gate3)?,
    )?;
    write_gate(
        &out,
        "parameter_identification",
        &serde_json::to_value(&review.gate4)?,
    )?;
    write_gate(
        &out,
        "damage_controls",
        &json!({
            "controls": review.gate5.controls.iter().filter(|c| c.name.contains("damage") || c.name.contains("no_a") || c.name.contains("no_p") || c.name.contains("maturation") || c.name.contains("relaxation")).cloned().collect::<Vec<_>>(),
            "pass": review.gate5.pass,
            "failure": review.gate5.failure,
        }),
    )?;
    write_gate(
        &out,
        "starvation_controls",
        &json!({
            "controls": review.gate5.controls.iter().filter(|c| c.name.contains("starvation") || c.name.contains("restoration") || c.name.contains("capacity")).cloned().collect::<Vec<_>>(),
            "pass": review.gate5.pass,
            "failure": review.gate5.failure,
        }),
    )?;
    write_gate(
        &out,
        "route_selection",
        &serde_json::to_value(&review.gate6)?,
    )?;

    let result = json!({
        "project_directive": D076_PROJECT_ID,
        "agent_memory_directive": D076_AGENT_MEMORY_ID,
        "starting_tag": D076_STARTING_TAG,
        "starting_commit": commit,
        "d075_conclusion": D075_CONCLUSION,
        "record": PASSIVE_RECORD,
        "primary_conclusion": review.gate6.conclusion,
        "route": review.gate6.route.as_str(),
        "scientific_conclusion": review.gate6.scientific_conclusion,
        "d008_status": review.gate6.d008_status,
        "phase1_status": review.gate6.phase1_status,
        "production_verdict": review.gate6.production_verdict,
        "next_directive": review.gate6.next_directive,
        "production_biology_unchanged": true,
        "gates": {
            "gate0_pass": review.gate0.pass,
            "gate1_pass": review.gate1.pass,
            "gate2_pass": review.gate2.pass,
            "gate3_pass": review.gate3.pass,
            "gate4_pass": review.gate4.pass,
            "gate5_pass": review.gate5.pass,
        },
        "candidate_equations": review.candidate_equations,
        "reasons": review.gate6.reasons,
    });
    atomic_write_json(&out.join("result.json"), &result)?;

    let manifest = json!({
        "project_directive": D076_PROJECT_ID,
        "agent_memory_directive": D076_AGENT_MEMORY_ID,
        "branch": "d008-membrane-metabolic-closure",
        "starting_tag": D076_STARTING_TAG,
        "source_commit": commit,
        "primary_conclusion": review.gate6.conclusion,
        "route": review.gate6.route.as_str(),
        "d008_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "record": PASSIVE_RECORD,
        "artifacts": [
            "preservation",
            "lineage_audit",
            "conservation",
            "reduced_model",
            "fixed_points",
            "energy_budget",
            "parameter_identification",
            "damage_controls",
            "starvation_controls",
            "route_selection",
            "result.json"
        ],
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    Ok(result)
}
