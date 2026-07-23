//! D-082 edge-membrane activation supply integration runner.

use crate::d013::atomic_write_json;
use chemistry_core::d082_analysis::*;
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
    if path.file_name().map(|n| n == "d082").unwrap_or(false) {
        let target = archive_root.join("d082");
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
        let local = resolve_path(Path::new("experiments/generated/d082"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;
    for sub in [
        "preservation",
        "d081_reproduction",
        "activation_lineage",
        "activation_parity",
        "energy_ledger",
        "replenishment",
        "controls",
        "demand_audit",
        "route_selection",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    write_gate(
        &out,
        "preservation",
        &json!({
            "d081_primary": D081_PRIMARY,
            "d081_status": D081_PROVISIONAL,
            "d081_tag": D082_STARTING_TAG,
            "d081_commit": D082_STARTING_COMMIT,
            "d080_gate7": "PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT",
            "seed_contract": SEED_CONTRACT_V1,
            "frozen": {
                "activation_kinetics": true,
                "a_to_l_yield": true,
                "binding_transport": true,
                "seed_reserve": true,
                "structural": true,
            },
        }),
    )?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(&out, "d081_reproduction", &serde_json::to_value(&review.gate0)?)?;
    write_gate(&out, "activation_lineage", &serde_json::to_value(&review.gate1)?)?;
    write_gate(&out, "activation_parity", &serde_json::to_value(&review.gate2)?)?;
    write_gate(&out, "energy_ledger", &serde_json::to_value(&review.gate3)?)?;
    write_gate(&out, "replenishment", &serde_json::to_value(&review.gate4)?)?;
    write_gate(
        &out,
        "controls",
        &json!({
            "arms": review.gate4.arms,
            "controls_flat": review.gate4.controls_flat,
        }),
    )?;
    write_gate(&out, "demand_audit", &serde_json::to_value(&review.gate5)?)?;
    write_gate(
        &out,
        "route_selection",
        &serde_json::to_value(&review.route)?,
    )?;

    let result = json!({
        "project_directive": D082_PROJECT_ID,
        "agent_memory_directive": D082_AGENT_MEMORY_ID,
        "starting_commit": D082_STARTING_COMMIT,
        "starting_tag": D082_STARTING_TAG,
        "ending_commit_at_run": commit,
        "d081_primary": D081_PRIMARY,
        "d081_status": review.route.d081_status,
        "d080_gate7_status": review.route.d080_gate7_status,
        "activation_lineage": review.gate1.classification,
        "integration_repaired": review.gate2.integration_repaired,
        "demand_classification": review.gate5.classification,
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
        "seed_contract": review.seed_contract,
        "gates": {
            "gate0": review.gate0.pass,
            "gate1": review.gate1.pass,
            "gate2": review.gate2.pass,
            "gate3": review.gate3.pass,
            "gate4": review.gate4.pass,
            "gate5": review.gate5.pass,
        },
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": "D-082",
            "conclusion": review.route.conclusion,
            "artifacts": [
                "preservation", "d081_reproduction", "activation_lineage", "activation_parity",
                "energy_ledger", "replenishment", "controls", "demand_audit", "route_selection",
                "result.json"
            ],
        }),
    )?;
    Ok(result)
}
