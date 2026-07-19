//! D-037 membrane-turnover provenance and renewal-gate audit pipeline.

use chemistry_core::d037_analysis::run_d037_audit;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260719-1040-d037-turnover-provenance-renewal-gate-audit";

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn compact_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    crate::d013::atomic_write_bytes(path, &serde_json::to_vec(value)?)
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn git_tag_present(tag: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn preserve(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-024-surface-density-pass",
        "D-024-surface-density-pass-provenance-sealed",
        "D-031-invariant-exchange-fail",
        "D-034-surface-maturation-fail",
        "D-035-catalytic-assembly-fail",
        "D-036-catalytic-complex-fail",
    ];
    let mut present = serde_json::Map::new();
    for t in tags {
        present.insert(t.into(), json!(git_tag_present(t)));
    }
    let body = json!({
        "project_directive": "D-037",
        "agent_memory_id": AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "d036_qualification": "D036_ARCHITECTURE_REJECTION_PENDING_ASSUMPTION_AUDIT",
        "preserved_tags": present,
        "chemistry_unchanged": true,
        "note": "Audit only; historical conclusions and tags unchanged.",
    });
    compact_write_json(&output.join("preservation.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;

    let preservation = preserve(&output.join("preservation"))?;
    let bundle = run_d037_audit(AGENT_MEMORY_ID);

    let write_gate = |name: &str, value: Value| -> Result<(), Box<dyn std::error::Error>> {
        let dir = output.join(name);
        fs::create_dir_all(&dir)?;
        compact_write_json(&dir.join("result.json"), &value)?;
        Ok(())
    };

    write_gate(
        "turnover_lineage",
        serde_json::to_value(&bundle.gate0)?,
    )?;
    write_gate(
        "bulk_surface_equivalence",
        serde_json::to_value(&bundle.gate1)?,
    )?;
    write_gate(
        "turnover_provenance",
        serde_json::to_value(&bundle.gate2)?,
    )?;
    write_gate(
        "state_classification",
        serde_json::to_value(&bundle.gate3)?,
    )?;
    write_gate("gate_semantics", serde_json::to_value(&bundle.gate4)?)?;
    write_gate("reduced_dynamics", serde_json::to_value(&bundle.gate5)?)?;
    write_gate("multistart", serde_json::to_value(&bundle.gate6)?)?;
    write_gate("route_decision", serde_json::to_value(&bundle.gate7)?)?;

    let result = json!({
        "project_directive": "D-037",
        "agent_memory_id": AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "d036_qualification": bundle.d036_qualification,
        "primary_conclusion": bundle.gate7.primary_conclusion,
        "secondary_findings": bundle.gate7.secondary_findings,
        "selected_route": bundle.gate7.route_label,
        "d008_status": bundle.gate7.d008_status,
        "phase1_status": bundle.gate7.phase1_status,
        "stage_f": bundle.gate7.stage_f,
        "production_verdict": bundle.gate7.production_verdict,
        "next_directive": bundle.gate7.next_directive,
        "next_execution_started": bundle.gate7.next_execution_started,
        "gate1_conclusion": bundle.gate1.conclusion,
        "gate1_max_relative_error": bundle.gate1.max_relative_error,
        "gate2_classification": bundle.gate2.classification_label,
        "gate4_defect": bundle.gate4.defect,
    });
    compact_write_json(&output.join("result.json"), &result)?;

    let manifest = json!({
        "project_directive": "D-037",
        "agent_memory_id": AGENT_MEMORY_ID,
        "source_commit": git_commit_hash(),
        "artifacts": [
            "preservation/",
            "turnover_lineage/",
            "bulk_surface_equivalence/",
            "turnover_provenance/",
            "state_classification/",
            "gate_semantics/",
            "reduced_dynamics/",
            "multistart/",
            "route_decision/",
            "result.json",
            "manifest.json"
        ],
        "primary_conclusion": bundle.gate7.primary_conclusion,
        "selected_route": bundle.gate7.route_label,
        "preservation": preservation,
    });
    compact_write_json(&output.join("manifest.json"), &manifest)?;
    Ok(result)
}
