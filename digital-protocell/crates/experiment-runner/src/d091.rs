//! D-091 metabolic reserve / ecological timescale pipeline.

use chemistry_core::d091_analysis::run_pipeline;
use serde_json::json;
use std::path::Path;

pub fn run_pipeline_cli(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let report = run_pipeline(output)?;
    Ok(json!({
        "primary_conclusion": report.primary_conclusion,
        "phase2_status": report.phase2_status,
        "phase3_authorized": report.phase3_authorized,
        "production_verdict": report.production_verdict,
        "schema_equation": report.schema_equation,
        "schema_fields": report.schema_fields,
        "selected_reserve": report.selected_reserve,
        "selected_ecology_h": report.selected_ecology_h,
        "selected_ecology_b": report.selected_ecology_b,
        "sigma": report.sigma,
        "mu": report.mu,
        "y_g": report.y_g,
        "smoke": report.smoke,
        "starting_commit": report.starting_commit,
        "gates": report.gates,
        "next_directive": report.next_directive,
        "next_execution_started": report.next_execution_started,
    }))
}
