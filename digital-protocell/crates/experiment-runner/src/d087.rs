//! D-087 independent Phase 1 certification pipeline.

use phase1_certifier::campaign::{default_out_root, run_certification};
use serde_json::json;
use std::path::{Path, PathBuf};

fn resolve_repo_root() -> PathBuf {
    // experiment-runner cwd is typically digital-protocell/
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("crates/phase1-certifier").exists() {
        cwd.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| cwd.clone())
    } else if cwd.join("digital-protocell/crates/phase1-certifier").exists() {
        cwd
    } else {
        cwd
    }
}

pub fn run_pipeline(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let repo = resolve_repo_root();
    let out = if output.as_os_str().is_empty() {
        default_out_root(&repo)
    } else {
        output.to_path_buf()
    };
    std::fs::create_dir_all(&out)?;
    let report = run_certification(&repo, &out)?;
    Ok(json!({
        "primary_conclusion": report.primary_conclusion,
        "phase1_status": report.phase1_status,
        "phase2_authorized": report.phase2_authorized,
        "next_execution_started": report.next_execution_started,
        "production_verdict": report.production_verdict,
        "smoke": report.smoke,
        "elapsed_secs": report.elapsed_secs,
        "artifact_root": report.artifact_root,
        "gate0": report.gate0,
        "gate1": report.gate1,
        "gate2": report.gate2,
        "gate3": report.gate3,
        "gate4": report.gate4,
        "gate5": report.gate5,
        "gate6": report.gate6,
        "gate7": report.gate7,
    }))
}
