use chemistry_core::d095_analysis::write_observational_artifacts;
use serde_json::Value;
use std::path::Path;

pub fn run_observational_cli(attempt: &Path, out: &Path) -> Result<Value, String> {
    write_observational_artifacts(attempt, out)
}
