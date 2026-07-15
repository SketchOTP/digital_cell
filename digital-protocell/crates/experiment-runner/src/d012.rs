//! D-012 conservative v2 stage validation (reuses D-008 stage machinery).

use crate::d008::{self, D008StageOptions};
use serde_json::Value;
use std::path::Path;

const V2_OPTIONS: D008StageOptions = D008StageOptions {
    equation_version: chemistry_core::EquationVersion::MembraneMetabolismV2Conservative,
    classification_prefix: "D012",
};

pub fn run_v2_stage_b(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    d008::run_stage_b_with(output, &V2_OPTIONS)
}

pub fn run_v2_stage_c(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    d008::run_stage_c_with(output, &V2_OPTIONS)
}

pub fn run_v2_stage_d(root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    d008::run_stage_d_with(root, &V2_OPTIONS)
}

pub fn v2_stage_options() -> D008StageOptions {
    V2_OPTIONS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("d012_{label}_{stamp}"))
    }

    #[test]
    fn run_v2_stage_b_passes_localization_gate() {
        let output = temp_dir("stage_b");
        let _ = fs::remove_dir_all(&output);
        let result = run_v2_stage_b(&output).expect("stage b");
        let on_disk: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
        assert_eq!(
            on_disk["stage_classification"],
            serde_json::json!("D012_STAGE_B_LOCALIZATION_PASS")
        );
        assert!(
            on_disk["localization"]["minimum_after_transient"]
                .as_f64()
                .unwrap()
                >= 0.90
        );
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn run_v2_stage_c_passes_metabolism_gate() {
        let output = temp_dir("stage_c");
        let _ = fs::remove_dir_all(&output);
        let result = run_v2_stage_c(&output).expect("stage c");
        let on_disk: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
        assert_eq!(
            on_disk["stage_classification"],
            serde_json::json!("D012_STAGE_C_METABOLISM_PASS")
        );
        assert_eq!(on_disk["equation_version"], serde_json::json!("membrane_metabolism_v2_conservative"));
        assert_eq!(result["run_count"], on_disk["run_count"]);
        for control in on_disk["controls"].as_array().unwrap() {
            assert_eq!(control["result"], serde_json::json!("pass"));
        }
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn run_v2_stage_d_passes_retention_gate() {
        let output = temp_dir("stage_d");
        let _ = fs::remove_dir_all(&output);
        let result = run_v2_stage_d(&output).expect("stage d");
        let attempt = output.join(
            result["attempt_directory"]
                .as_str()
                .expect("attempt dir"),
        );
        let on_disk: serde_json::Value =
            serde_json::from_slice(&fs::read(attempt.join("result.json")).unwrap()).unwrap();
        assert_eq!(
            on_disk["stage_classification"],
            serde_json::json!("D012_STAGE_D_FIXED_COMPARTMENT_PASS")
        );
        for row in on_disk["radius_results"].as_array().unwrap() {
            assert!(row["catalyst_retention"].as_f64().unwrap() >= 0.80);
            assert!(row["activated_retention"].as_f64().unwrap() >= 0.80);
        }
        let _ = fs::remove_dir_all(&output);
    }
}
