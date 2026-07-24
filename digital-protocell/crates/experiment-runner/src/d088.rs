//! D-088 emergent growth / fission / inheritance pipeline.

use chemistry_core::d088_analysis::run_pipeline;
use serde_json::json;
use std::path::Path;

pub fn run_pipeline_cli(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let report = run_pipeline(output)?;
    Ok(json!({
        "primary_conclusion": report.primary_conclusion,
        "phase2_status": report.phase2_status,
        "production_verdict": report.production_verdict,
        "selected_y_g": report.selected_y_g,
        "next_directive": report.next_directive,
        "next_execution_started": report.next_execution_started,
        "smoke": report.smoke,
        "runtime_closure": report.runtime_closure,
        "gates": report.gates,
    }))
}
