//! D-081 edge-membrane reserve provenance and replenishment runner.

use crate::d013::atomic_write_json;
use chemistry_core::d081_analysis::*;
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
    if path.file_name().map(|n| n == "d081").unwrap_or(false) {
        let target = archive_root.join("d081");
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
        let local = resolve_path(Path::new("experiments/generated/d081"));
        fs::create_dir_all(&local)?;
        local
    };
    fs::create_dir_all(&out)?;
    for sub in [
        "preservation",
        "d080_reproduction",
        "seed_provenance",
        "reserve_repair",
        "reserve_depletion",
        "replenishment",
        "energy_controls",
        "affordability",
        "dynamic_interface",
        "coupled_requalification",
        "structural_direction",
        "accounting",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    write_gate(
        &out,
        "preservation",
        &json!({
            "d080_primary": D080_PRIMARY,
            "d080_gate7_status": D080_GATE7_PROVISIONAL,
            "d080_tag": D081_STARTING_TAG,
            "d080_commit": D081_STARTING_COMMIT,
            "d080_artifacts": "experiments/generated/d080/",
            "seed_contract": SEED_CONTRACT_V1,
            "unchanged_mechanisms": true,
        }),
    )?;

    let review = run_full_review();
    let commit = git_commit_hash();

    write_gate(
        &out,
        "d080_reproduction",
        &serde_json::to_value(&review.gate0)?,
    )?;
    write_gate(
        &out,
        "seed_provenance",
        &serde_json::to_value(&review.gate1)?,
    )?;
    write_gate(&out, "reserve_repair", &serde_json::to_value(&review.gate2)?)?;
    write_gate(
        &out,
        "reserve_depletion",
        &serde_json::to_value(&review.gate3)?,
    )?;
    write_gate(
        &out,
        "replenishment",
        &serde_json::to_value(&review.gate4)?,
    )?;
    write_gate(
        &out,
        "energy_controls",
        &json!({
            "arms": review.gate4.arms,
            "only_normal_increases": review.gate4.only_normal_increases,
            "post_replenish_repair": review.gate4.post_replenish_repair,
        }),
    )?;
    write_gate(
        &out,
        "affordability",
        &serde_json::to_value(&review.gate5)?,
    )?;
    if let Some(ref dyn_r) = review.gate7_dynamic {
        write_gate(&out, "dynamic_interface", &serde_json::to_value(dyn_r)?)?;
    } else {
        write_gate(&out, "dynamic_interface", &json!({"skipped": true}))?;
    }
    if let Some(ref coupled) = review.gate7_coupled {
        write_gate(
            &out,
            "coupled_requalification",
            &serde_json::to_value(coupled)?,
        )?;
        write_gate(
            &out,
            "structural_direction",
            &serde_json::to_value(&coupled.structural)?,
        )?;
    } else {
        write_gate(&out, "coupled_requalification", &json!({"skipped": true}))?;
        write_gate(&out, "structural_direction", &json!({"skipped": true}))?;
    }
    write_gate(
        &out,
        "accounting",
        &json!({
            "k_lateral_scale": review.k_lateral_scale,
            "seed_contract": review.seed_contract,
            "gate6": review.gate6,
            "frozen_kinetics": true,
            "no_a_for_binding": true,
        }),
    )?;

    let initial_budgets: Value = serde_json::to_value(
        review
            .gate1
            .rows
            .iter()
            .map(|r| {
                json!({
                    "radius": r.radius,
                    "initial_m_l": r.initial_m_l,
                    "initial_m_b": r.initial_m_b,
                    "full_ring_capacity": r.full_ring_capacity,
                    "reserve_over_capacity_frac": r.reserve_over_capacity_frac,
                    "identity": r.identity,
                })
            })
            .collect::<Vec<_>>(),
    )?;

    let result = json!({
        "project_directive": D081_PROJECT_ID,
        "agent_memory_directive": D081_AGENT_MEMORY_ID,
        "starting_commit": D081_STARTING_COMMIT,
        "starting_tag": D081_STARTING_TAG,
        "ending_commit_at_run": commit,
        "d080_primary": D080_PRIMARY,
        "d080_gate7_status": review.route.d080_gate7_status,
        "seed_contract": review.seed_contract,
        "seed_classification": review.gate1.classification,
        "initial_budgets": initial_budgets,
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
            "gate2": review.gate2.pass,
            "gate3": review.gate3.pass,
            "gate4": review.gate4.pass,
            "gate5": review.gate5.pass,
            "gate6": review.gate6.pass,
            "gate7_dynamic": review.gate7_dynamic.as_ref().map(|g| g.pass),
            "gate7_coupled": review.gate7_coupled.as_ref().map(|g| g.pass),
        },
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": "D-081",
            "conclusion": review.route.conclusion,
            "artifacts": [
                "preservation", "d080_reproduction", "seed_provenance", "reserve_repair",
                "reserve_depletion", "replenishment", "energy_controls", "affordability",
                "dynamic_interface", "coupled_requalification", "structural_direction",
                "accounting", "result.json"
            ],
        }),
    )?;
    Ok(result)
}
