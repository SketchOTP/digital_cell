//! D-092 minimal catalytic template heredity pipeline.

use chemistry_core::d092_analysis::run_pipeline;
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
        "founder_sequences": report.founder_sequences,
        "measured_fidelity": report.measured_fidelity,
        "smoke": report.smoke,
        "starting_commit": report.starting_commit,
        "gates": report.gates,
        "records": report.records,
        "next_directive": report.next_directive,
        "next_execution_started": report.next_execution_started,
    }))
}
