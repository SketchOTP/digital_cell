use chemistry_core::d094_analysis::{run_audit_only, run_pipeline};
use chemistry_core::d094_selection::run_gate6_completion_only;
use serde_json::{json, Value};
use std::path::Path;

pub fn run_pipeline_cli(out: &Path) -> Result<Value, String> {
    let report = run_pipeline(out)?;
    Ok(json!({
        "primary_conclusion": report.primary_conclusion,
        "phase2_status": report.phase2_status,
        "phase3_authorized": report.phase3_authorized,
        "production_verdict": report.production_verdict,
        "zero_gen_blocker": report.zero_gen_blocker,
        "schema_equation": report.schema_equation,
        "smoke": report.smoke,
        "manifest": report,
    }))
}

pub fn run_audit_cli(out: &Path) -> Result<Value, String> {
    let audit = run_audit_only(out)?;
    Ok(json!({ "audit": audit }))
}

/// D-094R: Gate 6 completion only (no Gates 7/8).
pub fn run_gate6_complete_cli(out: &Path) -> Result<Value, String> {
    run_gate6_completion_only(out)
}
