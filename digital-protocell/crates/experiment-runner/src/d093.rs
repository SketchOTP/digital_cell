//! D-093 template-encoded catalytic network evolution pipeline.

use chemistry_core::d093_analysis::{repair_info_and_finalize, run_pipeline};
use serde_json::json;
use std::path::Path;

fn report_json(report: chemistry_core::d093_analysis::D093Report) -> serde_json::Value {
    json!({
        "primary_conclusion": report.primary_conclusion,
        "phase2_status": report.phase2_status,
        "phase3_authorized": report.phase3_authorized,
        "production_verdict": report.production_verdict,
        "schema_equation": report.schema_equation,
        "schema_fields": report.schema_fields,
        "founder_sequences": report.founder_sequences,
        "measured_fidelity": report.measured_fidelity,
        "foundation": report.foundation,
        "smoke": report.smoke,
        "starting_commit": report.starting_commit,
        "gates": report.gates,
        "records": report.records,
        "deviations": report.deviations,
        "next_directive": report.next_directive,
        "next_execution_started": report.next_execution_started,
    })
}

pub fn run_pipeline_cli(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(report_json(run_pipeline(output)?))
}

pub fn repair_info_cli(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(report_json(repair_info_and_finalize(output)?))
}
