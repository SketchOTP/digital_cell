//! D-083 conservative dynamic edge-membrane migration runner.

use crate::d013::atomic_write_json;
use chemistry_core::d083_analysis::*;
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
    if path.file_name().map(|n| n == "d083").unwrap_or(false) {
        let target = archive_root.join("d083");
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
        let local = resolve_path(Path::new("experiments/generated/d083"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;
    for sub in [
        "preservation",
        "d082_reproduction",
        "migration_provenance",
        "synthetic_motion",
        "autonomous_motion",
        "regressions",
        "structural_separation",
        "accounting",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    write_gate(
        &out,
        "preservation",
        &json!({
            "starting_commit": D083_STARTING_COMMIT,
            "starting_tag": D083_STARTING_TAG,
            "frozen": {
                "activation": true,
                "a_to_l": true,
                "bind_unbind_lateral": true,
                "capacity_perm_seed_damage": true,
                "cut_cell_support_reconstruction": true,
                "structural_rates": true,
            },
            "no_global_remapping": true,
        }),
    )?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(
        &out,
        "d082_reproduction",
        &serde_json::to_value(&review.gate0)?,
    )?;
    write_gate(
        &out,
        "migration_provenance",
        &serde_json::to_value(&review.gate1)?,
    )?;
    write_gate(
        &out,
        "synthetic_motion",
        &serde_json::to_value(&review.gate3)?,
    )?;
    write_gate(
        &out,
        "autonomous_motion",
        &serde_json::to_value(&review.gate4)?,
    )?;
    write_gate(&out, "regressions", &serde_json::to_value(&review.gate5)?)?;
    write_gate(
        &out,
        "structural_separation",
        &serde_json::to_value(&review.gate6)?,
    )?;
    write_gate(
        &out,
        "accounting",
        &json!({
            "synthetic_conservation": review.gate3.cases.iter().map(|c| {
                json!({
                    "name": c.name,
                    "delta_membrane": c.delta_membrane,
                    "accounting_residual": c.accounting_residual,
                    "ok": c.conservation_ok,
                })
            }).collect::<Vec<_>>(),
            "autonomous_conservation": review.gate4.rows.iter().map(|r| {
                json!({
                    "radius": r.radius,
                    "delta_membrane": r.metrics.delta_membrane,
                    "ok": r.metrics.conservation_ok,
                })
            }).collect::<Vec<_>>(),
        }),
    )?;
    write_gate(
        &out,
        "route_selection",
        &serde_json::to_value(&review.route)?,
    )?;

    let result = json!({
        "project_directive": D083_PROJECT_ID,
        "agent_memory_directive": D083_AGENT_MEMORY_ID,
        "starting_commit": D083_STARTING_COMMIT,
        "starting_tag": D083_STARTING_TAG,
        "ending_commit_at_run": commit,
        "primary_conclusion": review.route.conclusion,
        "structural_direction": review.route.structural_direction,
        "structural_blocker_remains": review.route.structural_blocker_remains,
        "stopped_at_gate": review.route.stopped_at_gate,
        "scientific_conclusion": review.route.scientific_conclusion,
        "next_directive": review.route.next_directive,
        "next_execution_started": false,
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "first_divergence": review.gate1.first_divergence,
        "gates": {
            "gate0": review.gate0.pass,
            "gate1": review.gate1.pass,
            "gate3": review.gate3.pass,
            "gate4": review.gate4.pass,
            "gate5": review.gate5.pass,
            "gate6": review.gate6.pass,
        },
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": "D-083",
            "conclusion": review.route.conclusion,
            "artifacts": [
                "preservation", "d082_reproduction", "migration_provenance",
                "synthetic_motion", "autonomous_motion", "regressions",
                "structural_separation", "accounting", "route_selection", "result.json"
            ],
        }),
    )?;
    Ok(result)
}
