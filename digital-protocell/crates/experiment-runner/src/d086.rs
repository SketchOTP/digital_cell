//! D-086 autopoietic material-mesh Phase 1 qualification runner.

use crate::d013::atomic_write_json;
use chemistry_core::d086_analysis::*;
use chemistry_core::material_mesh::{
    EQUATION_VERSION_MATERIAL_MESH, FIELD_SCHEMA_MATERIAL_MESH, MATERIAL_MESH_SCHEMA_VERSION,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}

fn ensure_archive_symlink(path: &Path) {
    if path.exists() {
        return;
    }
    let archive_root = PathBuf::from(
        "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated",
    );
    if path.file_name().map(|n| n == "d086").unwrap_or(false) {
        let target = archive_root.join("d086");
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

fn git_out(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
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
        let local = resolve_path(Path::new("experiments/generated/d086"));
        fs::create_dir_all(&local)?;
        local
    };
    for sub in [
        "preservation",
        "mechanics",
        "remeshing",
        "transport",
        "metabolism",
        "turnover",
        "basin",
        "damage",
        "starvation",
        "death",
        "accounting",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    let branch = git_out(&["branch", "--show-current"]);
    let head = git_out(&["rev-parse", "--short", "HEAD"]);
    let tag_commit = git_out(&["rev-parse", "--short", &format!("{}^{{}}", D086_STARTING_TAG)]);

    let review = run_full_review(&branch, &head, &tag_commit);

    write_gate(
        &out,
        "preservation",
        &json!({
            "gate0": review.gate0,
            "starting_commit": D086_STARTING_COMMIT,
            "starting_tag": D086_STARTING_TAG,
            "branch": D086_BRANCH,
            "equation_version": EQUATION_VERSION_MATERIAL_MESH,
            "field_schema": FIELD_SCHEMA_MATERIAL_MESH,
            "schema_version": MATERIAL_MESH_SCHEMA_VERSION,
            "records": review.route.records,
            "scope_amendment": "Phase 1 requires one persistent metabolically active mesh protocell; phi body retired",
        }),
    )?;
    write_gate(&out, "mechanics", &serde_json::to_value(&review.gate1)?)?;
    write_gate(
        &out,
        "remeshing",
        &json!({
            "covered_in_gate1": true,
            "split_merge_conservative": review.gate1.pass,
        }),
    )?;
    write_gate(
        &out,
        "transport",
        &json!({
            "gate3_detail": review.gate3.detail,
            "permeability_targets": "C,A<=0.05; N,F in [0.20,0.50]; W>=0.70",
        }),
    )?;
    write_gate(&out, "metabolism", &serde_json::to_value(&review.gate3)?)?;
    write_gate(&out, "turnover", &serde_json::to_value(&review.gate4)?)?;
    write_gate(
        &out,
        "basin",
        &json!({
            "gate2": review.gate2,
            "gate5": review.gate5,
            "rows": review.gate5_rows,
            "selected_mech": review.route.selected_mech,
        }),
    )?;
    write_gate(&out, "damage", &serde_json::to_value(&review.gate6)?)?;
    write_gate(&out, "starvation", &serde_json::to_value(&review.gate7)?)?;
    write_gate(
        &out,
        "death",
        &json!({
            "gate7": review.gate7,
            "irreversible": review.gate7.pass,
        }),
    )?;
    write_gate(
        &out,
        "accounting",
        &json!({
            "accepted_step_model": "transport→reactions→mechanics→remesh",
            "no_target_geometry": true,
            "gates": {
                "gate0": review.gate0.pass,
                "gate1": review.gate1.pass,
                "gate2": review.gate2.pass,
                "gate3": review.gate3.pass,
                "gate4": review.gate4.pass,
                "gate5": review.gate5.pass,
                "gate6": review.gate6.pass,
                "gate7": review.gate7.pass,
            },
        }),
    )?;

    let result = json!({
        "project_directive": D086_PROJECT_ID,
        "agent_memory_directive": D086_AGENT_MEMORY_ID,
        "starting_commit": D086_STARTING_COMMIT,
        "starting_tag": D086_STARTING_TAG,
        "branch": branch,
        "ending_commit_at_run": git_out(&["rev-parse", "HEAD"]),
        "equation_version": EQUATION_VERSION_MATERIAL_MESH,
        "field_schema": FIELD_SCHEMA_MATERIAL_MESH,
        "primary_conclusion": review.route.conclusion,
        "stopped_at_gate": review.route.stopped_at_gate,
        "selected_mech": review.route.selected_mech,
        "d008_status": review.route.d008_status,
        "phase1_status": review.route.phase1_status,
        "production_verdict": review.route.production_verdict,
        "scientific_conclusion": review.route.scientific_conclusion,
        "next_directive": review.route.next_directive,
        "next_execution_started": false,
        "records": review.route.records,
        "smoke": smoke_mode(),
        "route": review.route,
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": D086_PROJECT_ID,
            "conclusion": review.route.conclusion,
            "artifacts": [
                "preservation","mechanics","remeshing","transport","metabolism",
                "turnover","basin","damage","starvation","death","accounting","result.json"
            ],
        }),
    )?;
    Ok(result)
}
