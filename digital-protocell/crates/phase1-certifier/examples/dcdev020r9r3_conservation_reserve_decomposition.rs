//! DC-DEV-020-R9-R3: observer-only contract × reserve certification matrix.
//!
//! The four arms use the actual D-087 certifier. Contract and D-091 reserve
//! selection are independent environment-controlled diagnostic axes; no
//! production defaults or organism constants are changed.

use phase1_certifier::campaign::run_certification;
use phase1_certifier::sim::{run_coupled, seed_mesh};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const STARTING_HEAD: &str = "b7be0e7476ad153f8657d832c9df629e522f09b9";
const BRANCH: &str = "strategy/dc-dev-020r9-mesh-contract-requalification";
const PR: &str = "#44";
const R9R2_CLASSIFICATION: &str = "DCDEV020R9R2_CONSERVATIVE_CERTIFICATION_REGRESSION";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReserveExecution {
    a_to_r: f64,
    r_to_a: f64,
    r_to_w: f64,
    rejected_steps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArmSummary {
    id: String,
    contract: String,
    reserve: bool,
    equation_lineage: String,
    output: String,
    gates: Vec<bool>,
    gate_details: Vec<String>,
    passed_gates: usize,
    scientific_gates_pass: bool,
    runtime_gate_pass: bool,
    primary_conclusion: String,
    r_m: Option<f64>,
    r_b: Option<f64>,
    r_c: Option<f64>,
    reserve_execution: ReserveExecution,
}

#[derive(Debug, Serialize)]
struct MatrixReport {
    directive: String,
    starting_head: String,
    branch: String,
    pr: String,
    arms: Vec<ArmSummary>,
    h0_historical_reproduction: bool,
    h0_hard_stop: bool,
    r9r2_material_fate_preserved: bool,
    primary_classification: String,
    production_chemistry_changed: String,
    production_behavior_changed: String,
    recycling_authorized: bool,
    dc_dev_021_authorized: bool,
    next_execution_started: bool,
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn set_arm(contract: &str, reserve: bool) {
    env::remove_var("DCDEV020R9R1_V2");
    env::remove_var("DCDEV020R9R2_V2");
    env::set_var("DCDEV020R9R3_CONTRACT", contract);
    env::set_var("DCDEV020R9R3_RESERVE", if reserve { "1" } else { "0" });
}

fn number(value: &Value, path: &[&str]) -> Option<f64> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_f64)
}

fn reserve_execution() -> ReserveExecution {
    let mut mesh = seed_mesh(14.0, 2);
    let ledger = run_coupled(&mut mesh, 480, true, true);
    ReserveExecution {
        a_to_r: ledger.reserve_a_to_r,
        r_to_a: ledger.reserve_r_to_a,
        r_to_w: ledger.reserve_r_to_w,
        rejected_steps: ledger.reserve_rejected_steps,
    }
}

fn run_arm(root: &Path, id: &str, contract: &str, reserve: bool) -> Result<ArmSummary, String> {
    set_arm(contract, reserve);
    let reserve_execution = reserve_execution();
    let output = root.join("matrix").join(id);
    let report = run_certification(Path::new(".."), &output)?;
    if report.mesh_contract != contract || report.reserve_enabled != reserve {
        return Err(format!(
            "arm {id} selector mismatch: requested contract={contract} reserve={reserve}, report contract={} reserve={}",
            report.mesh_contract, report.reserve_enabled
        ));
    }
    let audit = read_json(&output.join("metric_semantics/audit.json"))?;
    let gates = vec![
        report.gate0.pass,
        report.gate1.pass,
        report.gate2.pass,
        report.gate3.pass,
        report.gate4.pass,
        report.gate5.pass,
        report.gate6.pass,
        report.gate7.pass,
    ];
    let gate_details = vec![
        report.gate0.detail,
        report.gate1.detail,
        report.gate2.detail,
        report.gate3.detail,
        report.gate4.detail,
        report.gate5.detail,
        report.gate6.detail,
        report.gate7.detail,
    ];
    let scientific_gates_pass = gates[..7].iter().all(|pass| *pass);
    Ok(ArmSummary {
        id: id.into(),
        contract: report.mesh_contract,
        reserve: report.reserve_enabled,
        equation_lineage: report.equation_lineage,
        output: output.display().to_string(),
        passed_gates: gates.iter().filter(|pass| **pass).count(),
        runtime_gate_pass: report.gate7.pass,
        primary_conclusion: report.primary_conclusion,
        r_m: number(&audit, &["audit", "structural", "r_x"]),
        r_b: number(&audit, &["audit", "membrane", "r_x"]),
        r_c: number(&audit, &["audit", "catalyst", "r_x"]),
        reserve_execution,
        gates,
        gate_details,
        scientific_gates_pass,
    })
}

fn classify(arms: &[ArmSummary]) -> String {
    let h0 = &arms[0].scientific_gates_pass;
    let v20 = &arms[1].scientific_gates_pass;
    let h1 = &arms[2].scientific_gates_pass;
    let v21 = &arms[3].scientific_gates_pass;
    if !*h0 {
        "DCDEV020R9R3_HISTORICAL_CONTROL_NOT_REPRODUCED".into()
    } else if *v20 && !*h1 && !*v21 {
        "DCDEV020R9R3_RESERVE_PHYSIOLOGY_CERTIFICATION_GAP_CONFIRMED".into()
    } else if !*v20 && *h1 && !*v21 {
        "DCDEV020R9R3_CONSERVATIVE_CONTRACT_REGRESSION_CONFIRMED".into()
    } else if *v20 && *h1 && !*v21 {
        "DCDEV020R9R3_CONSERVATIVE_RESERVE_INTERACTION_REGRESSION".into()
    } else if !*v20 && !*h1 {
        "DCDEV020R9R3_MIXED_CERTIFICATION_REGRESSION".into()
    } else if *v20 && *h1 && *v21 {
        "DCDEV020R9R3_CERTIFIER_HARNESS_DEFECT".into()
    } else {
        "DCDEV020R9R3_MIXED_CERTIFICATION_REGRESSION".into()
    }
}

fn r9r2_material_fate_preserved(repo_root: &Path) -> bool {
    let qualification = repo_root.join("experiments/generated/dcdev020r9r2/qualification.json");
    let fate = repo_root.join("experiments/generated/dcdev020r9r2/material_fate.json");
    let Ok(q) = read_json(&qualification) else {
        return false;
    };
    let Ok(f) = read_json(&fate) else {
        return false;
    };
    q["primary_classification"] == R9R2_CLASSIFICATION
        && f["finite_d016"]["organized_retained_delta"]
            .as_f64()
            .is_some_and(|v| (v + 10.277547850163074).abs() < 1e-9)
        && f["finite_d016"]["fate"]["c_to_w"]
            .as_f64()
            .is_some_and(|v| (v - 5.290173380171319).abs() < 1e-9)
        && [
            "d016_normal",
            "d016_cprod_deferred",
            "r6_normal",
            "r6_cprod_deferred",
        ]
        .into_iter()
        .all(|arm| {
            f["sustained"][arm]["final_quarter_organized_slope"]
                .as_f64()
                .is_some_and(|v| v < 0.0)
        })
}

fn main() -> Result<(), String> {
    let root = PathBuf::from(
        env::var("DCDEV020R9R3_OUTPUT")
            .unwrap_or_else(|_| "experiments/generated/dcdev020r9r3".into()),
    );
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;

    let mut arms = Vec::new();
    let h0 = run_arm(&root, "h0", "HistoricalV1", false)?;
    let h0_reproduced = h0.scientific_gates_pass;
    arms.push(h0);
    if !h0_reproduced {
        let report = MatrixReport {
            directive: "DC-DEV-020-R9-R3".into(),
            starting_head: STARTING_HEAD.into(),
            branch: BRANCH.into(),
            pr: PR.into(),
            arms,
            h0_historical_reproduction: false,
            h0_hard_stop: true,
            r9r2_material_fate_preserved: r9r2_material_fate_preserved(Path::new(".")),
            primary_classification: "DCDEV020R9R3_HISTORICAL_CONTROL_NOT_REPRODUCED".into(),
            production_chemistry_changed: "NO".into(),
            production_behavior_changed: "NO".into(),
            recycling_authorized: false,
            dc_dev_021_authorized: false,
            next_execution_started: false,
        };
        write_json(&root.join("qualification.json"), &report)?;
        println!("DCDEV020R9R3_HISTORICAL_CONTROL_NOT_REPRODUCED");
        return Ok(());
    }

    arms.push(run_arm(&root, "v20", "ConservativeV2", false)?);
    arms.push(run_arm(&root, "h1", "HistoricalV1", true)?);
    arms.push(run_arm(&root, "v21", "ConservativeV2", true)?);
    let classification = classify(&arms);
    let report = MatrixReport {
        directive: "DC-DEV-020-R9-R3".into(),
        starting_head: STARTING_HEAD.into(),
        branch: BRANCH.into(),
        pr: PR.into(),
        arms,
        h0_historical_reproduction: true,
        h0_hard_stop: false,
        r9r2_material_fate_preserved: r9r2_material_fate_preserved(Path::new(".")),
        primary_classification: classification.clone(),
        production_chemistry_changed: "NO".into(),
        production_behavior_changed: "NO".into(),
        recycling_authorized: false,
        dc_dev_021_authorized: false,
        next_execution_started: false,
    };
    write_json(
        &root.join("protocol.json"),
        &json!({
            "directive": "DC-DEV-020-R9-R3",
            "starting_head": STARTING_HEAD,
            "branch": BRANCH,
            "pr": PR,
            "matrix": [
                {"id": "H0", "contract": "HistoricalV1", "reserve": false},
                {"id": "V20", "contract": "ConservativeV2", "reserve": false},
                {"id": "H1", "contract": "HistoricalV1", "reserve": true},
                {"id": "V21", "contract": "ConservativeV2", "reserve": true}
            ],
            "h0_hard_stop": "scientific gates 0-6 must pass before V20/H1/V21 execute",
            "r9r2_material_fate_not_rerun": true,
            "recycling_authorized": false,
            "dc_dev_021_authorized": false
        }),
    )?;
    write_json(&root.join("qualification.json"), &report)?;
    println!("DCDEV020R9R3_MATRIX_COMPLETE classification={classification}");
    Ok(())
}
