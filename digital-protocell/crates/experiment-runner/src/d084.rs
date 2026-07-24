//! D-084 edge-boundary structural homeostasis runner.

use crate::d013::atomic_write_json;
use chemistry_core::d084_analysis::*;
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
    if path.file_name().map(|n| n == "d084").unwrap_or(false) {
        let target = archive_root.join("d084");
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
        let local = resolve_path(Path::new("experiments/generated/d084"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;
    for sub in [
        "preservation",
        "d083_reproduction",
        "structural_ledger",
        "scaling",
        "candidate_identification",
        "prescribed_radius",
        "dynamic_basin",
        "energy_waste",
        "damage_starvation",
        "stage_e",
        "accounting",
        "route_selection",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    write_gate(
        &out,
        "preservation",
        &json!({
            "starting_commit": D084_STARTING_COMMIT,
            "starting_tag": D084_STARTING_TAG,
            "records": [
                "D083_EDGE_DYNAMIC_MIGRATION_REPAIRED",
                "STRUCTURAL_RESTORING_BLOCKER_REMAINS"
            ],
            "frozen": {
                "structural_production": true,
                "activation": true,
                "edge_membrane": true,
                "no_target_radius_or_mass": true,
                "no_global_feedback": true,
            },
            "closed_repairs": [
                "scalar_decay_multiplier",
                "a_deficit_loss",
                "activation_increase",
                "production_rate_sweep",
                "radius_specific_decay",
                "target_radius_mass"
            ],
        }),
    )?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(&out, "d083_reproduction", &serde_json::to_value(&review.gate0)?)?;
    write_gate(&out, "structural_ledger", &serde_json::to_value(&review.gate1)?)?;
    write_gate(
        &out,
        "scaling",
        &json!({
            "p_g": review.gate1.p_g,
            "p_l": review.gate1.p_l,
            "scaling_class": review.gate1.scaling_class,
            "approximately_matched": review.gate1.approximately_matched,
        }),
    )?;
    write_gate(
        &out,
        "candidate_identification",
        &serde_json::to_value(&review.gate2)?,
    )?;
    write_gate(&out, "prescribed_radius", &serde_json::to_value(&review.gate4)?)?;
    write_gate(&out, "dynamic_basin", &serde_json::to_value(&review.gate5)?)?;
    write_gate(&out, "energy_waste", &serde_json::to_value(&review.gate6)?)?;
    write_gate(
        &out,
        "damage_starvation",
        &serde_json::to_value(&review.gate7)?,
    )?;
    write_gate(&out, "stage_e", &serde_json::to_value(&review.gate8)?)?;
    write_gate(
        &out,
        "accounting",
        &serde_json::to_value(&review.gate3)?,
    )?;

    let result = json!({
        "project_directive": D084_PROJECT_ID,
        "agent_memory_directive": D084_AGENT_MEMORY_ID,
        "starting_commit": D084_STARTING_COMMIT,
        "starting_tag": D084_STARTING_TAG,
        "ending_commit_at_run": commit,
        "primary_conclusion": review.route.conclusion,
        "stopped_at_gate": review.route.stopped_at_gate,
        "selected_eta": review.route.selected_eta,
        "selected_k_phi_minus": review.route.selected_k_phi_minus,
        "selected_hash": review.route.selected_hash,
        "p_g": review.route.p_g,
        "p_l": review.route.p_l,
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
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "scientific_conclusion": review.route.scientific_conclusion,
        "next_directive": review.route.next_directive,
        "next_execution_started": review.route.next_execution_started,
        "skip_late_gates": skip_late_gates(),
        "full_gate0": full_gate0(),
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": D084_PROJECT_ID,
            "conclusion": review.route.conclusion,
            "artifacts": [
                "preservation",
                "d083_reproduction",
                "structural_ledger",
                "scaling",
                "candidate_identification",
                "prescribed_radius",
                "dynamic_basin",
                "energy_waste",
                "damage_starvation",
                "stage_e",
                "accounting",
                "result.json"
            ],
        }),
    )?;
    write_gate(&out, "route_selection", &serde_json::to_value(&review.route)?)?;
    Ok(result)
}
