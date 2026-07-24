//! Full D-087 certification campaign.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::frozen::{frozen_identity, D087_AGENT_ID, D087_PROJECT_ID, FROZEN_COMMIT, FROZEN_TAG};
use crate::gates::{
    gate0_integrity, gate1_metric_semantics, gate2_exact_replay, gate3_held_out, gate4_robustness,
    gate5_ablations, gate6_damage_generalization, smoke, D087Conclusion, GateResult,
};
use crate::runtime::{gate7_linux_runtime, RuntimeReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationReport {
    pub project_directive: String,
    pub agent_directive: String,
    pub entry_commit: String,
    pub entry_tag: String,
    pub frozen: serde_json::Value,
    pub smoke: bool,
    pub gate0: GateResult,
    pub gate1: GateResult,
    pub gate2: GateResult,
    pub gate3: GateResult,
    pub gate4: GateResult,
    pub gate5: GateResult,
    pub gate6: GateResult,
    pub gate7: GateResult,
    pub primary_conclusion: String,
    pub phase1_status: String,
    pub phase2_authorized: bool,
    pub next_execution_started: bool,
    pub production_verdict: String,
    pub elapsed_secs: f64,
    pub artifact_root: String,
}

fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    fs::write(path, s).map_err(|e| e.to_string())
}

fn first_failure(gates: &[(&str, &GateResult)]) -> Option<String> {
    for (_, g) in gates {
        if !g.pass {
            return g.failure.clone();
        }
    }
    None
}

pub fn decide(
    g0: &GateResult,
    g1: &GateResult,
    g2: &GateResult,
    g3: &GateResult,
    g4: &GateResult,
    g5: &GateResult,
    g6: &GateResult,
    g7: &GateResult,
) -> (String, String, bool, String) {
    let ordered = [
        ("g0", g0),
        ("g1", g1),
        ("g2", g2),
        ("g3", g3),
        ("g4", g4),
        ("g5", g5),
        ("g6", g6),
        ("g7", g7),
    ];
    if let Some(f) = first_failure(&ordered) {
        let (status, p2, verdict) = if f == D087Conclusion::D086AcceptanceInvalid.as_str() {
            (
                "PHASE1_ACCEPTANCE_INVALID",
                false,
                "PHASE1_NOT_CERTIFIED",
            )
        } else if f == D087Conclusion::LinuxRuntimeQualificationFailure.as_str()
            && g0.pass
            && g1.pass
            && g2.pass
            && g3.pass
            && g4.pass
            && g5.pass
            && g6.pass
        {
            (
                "PHASE1_SCIENCE_CERTIFIED_RUNTIME_NOT_QUALIFIED",
                false,
                "PHASE1_SCIENCE_OK_RUNTIME_FAIL",
            )
        } else if f.contains("REPRODUC") || f.contains("HELD_OUT") {
            (
                "PHASE1_NOT_REPRODUCIBLE",
                false,
                "PHASE1_NOT_CERTIFIED",
            )
        } else if f.contains("CAUSAL") {
            (
                "PHASE1_CAUSAL_CLOSURE_NOT_CERTIFIED",
                false,
                "PHASE1_NOT_CERTIFIED",
            )
        } else {
            ("PHASE1_NOT_CERTIFIED", false, "PHASE1_NOT_CERTIFIED")
        };
        // Map runtime-only fail to the specific conclusion string if science passed.
        let conclusion = if status == "PHASE1_SCIENCE_CERTIFIED_RUNTIME_NOT_QUALIFIED" {
            D087Conclusion::Phase1ScienceCertifiedRuntimeNotQualified
                .as_str()
                .to_string()
        } else {
            f
        };
        return (conclusion, status.into(), p2, verdict.into());
    }
    (
        D087Conclusion::Phase1AutopoieticProtocellCertified
            .as_str()
            .into(),
        "PHASE1_COMPLETE".into(),
        true,
        "PHASE1_RESEARCH_RUNTIME_QUALIFIED".into(),
    )
}

pub fn run_certification(repo_root: &Path, out_root: &Path) -> Result<CertificationReport, String> {
    let t0 = Instant::now();
    let dirs = [
        "preservation",
        "source_audit",
        "artifact_integrity",
        "metric_semantics",
        "exact_replay",
        "held_out_reproducibility",
        "robustness",
        "causal_ablations",
        "damage_generalization",
        "linux_runtime",
        "certification",
    ];
    for d in dirs {
        fs::create_dir_all(out_root.join(d)).map_err(|e| e.to_string())?;
    }

    let (g0, hits) = gate0_integrity(repo_root);
    write_json(&out_root.join("source_audit/pattern_hits.json"), &hits)?;
    write_json(&out_root.join("artifact_integrity/gate0.json"), &g0)?;

    let (g1, metrics_body) = gate1_metric_semantics();
    write_json(&out_root.join("metric_semantics/audit.json"), &metrics_body)?;
    write_json(&out_root.join("metric_semantics/gate1.json"), &g1)?;

    let g2 = gate2_exact_replay();
    write_json(&out_root.join("exact_replay/gate2.json"), &g2)?;

    let (g3, held) = gate3_held_out();
    write_json(
        &out_root.join("held_out_reproducibility/matrix.json"),
        &held,
    )?;
    write_json(&out_root.join("held_out_reproducibility/gate3.json"), &g3)?;

    let (g4, rob) = gate4_robustness();
    write_json(&out_root.join("robustness/matrix.json"), &rob)?;
    write_json(&out_root.join("robustness/gate4.json"), &g4)?;

    let (g5, abl) = gate5_ablations();
    write_json(&out_root.join("causal_ablations/rows.json"), &abl)?;
    write_json(&out_root.join("causal_ablations/gate5.json"), &g5)?;

    let g6 = gate6_damage_generalization();
    write_json(&out_root.join("damage_generalization/gate6.json"), &g6)?;

    let (g7, rt): (GateResult, RuntimeReport) = gate7_linux_runtime(repo_root, out_root);
    write_json(&out_root.join("linux_runtime/report.json"), &rt)?;
    write_json(&out_root.join("linux_runtime/gate7.json"), &g7)?;

    let (conclusion, phase1_status, phase2, production) =
        decide(&g0, &g1, &g2, &g3, &g4, &g5, &g6, &g7);

    let report = CertificationReport {
        project_directive: D087_PROJECT_ID.into(),
        agent_directive: D087_AGENT_ID.into(),
        entry_commit: FROZEN_COMMIT.into(),
        entry_tag: FROZEN_TAG.into(),
        frozen: serde_json::to_value(frozen_identity()).unwrap_or_default(),
        smoke: smoke(),
        gate0: g0,
        gate1: g1,
        gate2: g2,
        gate3: g3,
        gate4: g4,
        gate5: g5,
        gate6: g6,
        gate7: g7,
        primary_conclusion: conclusion,
        phase1_status,
        phase2_authorized: phase2,
        next_execution_started: false, // filled by launcher after branch
        production_verdict: production,
        elapsed_secs: t0.elapsed().as_secs_f64(),
        artifact_root: out_root.display().to_string(),
    };
    write_json(&out_root.join("certification/report.json"), &report)?;
    write_json(&out_root.join("manifest.json"), &report)?;
    Ok(report)
}

pub fn default_out_root(repo_root: &Path) -> PathBuf {
    let p = repo_root.join("digital-protocell/experiments/generated/d087");
    if p.exists() {
        p
    } else {
        repo_root.join("experiments/generated/d087")
    }
}
