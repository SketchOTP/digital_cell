//! Gate 0 source / forbidden-pattern classification.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternHit {
    pub pattern: String,
    pub path: String,
    pub line: usize,
    pub snippet: String,
    pub classification: String,
}

const PATTERNS: &[&str] = &[
    "alive",
    "health",
    "target_radius",
    "target_area",
    "target_mass",
    "repair",
    "respawn",
    "reconstruct",
    "desired_shape",
    "force_close",
    "auto_heal",
];

fn classify(pattern: &str, path: &str, snippet: &str) -> String {
    let p = path.replace('\\', "/");
    let s = snippet.to_lowercase();
    match pattern {
        "alive" if p.contains("material_mesh") || p.contains("mesh_") || p.contains("d086") || p.contains("phase1") => {
            "OBSERVER_OR_TERMINAL_FLAG: organism death flag set by evaluate_death causal criteria; not a health controller".into()
        }
        "alive" => "REVIEW: alive flag outside mesh package".into(),
        "health" => "FORBIDDEN_IF_CONTROLLER: health variable".into(),
        "target_radius" | "target_area" | "target_mass" | "desired_shape" => {
            "FORBIDDEN_IF_CONTROLLER: target geometry".into()
        }
        "repair" if s.contains("fn repair") || s.contains("repair(") => {
            "FORBIDDEN_COMMAND: explicit repair()".into()
        }
        "repair" => {
            "COMMENT_OR_METRIC: repair wording in analysis/docs; verify not a command".into()
        }
        "respawn" | "reconstruct" | "force_close" | "auto_heal" => {
            if s.contains("fn ") && s.contains(pattern) {
                "FORBIDDEN_COMMAND".into()
            } else {
                "COMMENT_OR_ABSENCE_CHECK".into()
            }
        }
        _ => "UNCLASSIFIED".into(),
    }
}

pub fn scan_source_tree(root: &Path) -> Vec<PatternHit> {
    let mut hits = Vec::new();
    let roots = [
        root.join("digital-protocell/crates/chemistry-core/src"),
        root.join("digital-protocell/crates/phase1-certifier/src"),
        root.join("digital-protocell/crates/experiment-runner/src"),
    ];
    for dir in &roots {
        if !dir.exists() {
            continue;
        }
        walk(dir, &mut hits);
    }
    hits
}

fn walk(dir: &Path, hits: &mut Vec<PatternHit>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "target" || name.starts_with('.') {
                continue;
            }
            walk(&p, hits);
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        // Do not scan d086_analysis for certifier independence of conclusions,
        // but DO scan it for forbidden controllers.
        let Ok(text) = fs::read_to_string(&p) else {
            continue;
        };
        for (li, line) in text.lines().enumerate() {
            let low = line.to_lowercase();
            for pat in PATTERNS {
                if low.contains(pat) {
                    let path = p.display().to_string();
                    hits.push(PatternHit {
                        pattern: (*pat).into(),
                        path: path.clone(),
                        line: li + 1,
                        snippet: line.trim().chars().take(160).collect(),
                        classification: classify(pat, &path, line),
                    });
                }
            }
        }
    }
}

pub fn forbidden_controller_failures(hits: &[PatternHit]) -> Vec<String> {
    hits.iter()
        .filter(|h| {
            h.classification.starts_with("FORBIDDEN")
                || h.classification.starts_with("UNCLASSIFIED")
        })
        .map(|h| {
            format!(
                "{}:{} [{}] {}",
                h.path, h.line, h.classification, h.snippet
            )
        })
        .collect()
}

pub fn git_short(args: &[&str], cwd: &Path) -> String {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub head: String,
    pub tag_commit: String,
    pub branch: String,
    pub entry_commit_ok: bool,
    pub tag_ok: bool,
    pub dirty_candidate_source: bool,
    pub pattern_hits: usize,
    pub forbidden_failures: Vec<String>,
}

pub fn integrity_check(repo: &Path) -> IntegrityReport {
    let head = git_short(&["rev-parse", "--short", "HEAD"], repo);
    let tag = git_short(
        &["rev-parse", "--short", "D-086-mesh-protocell-phase1-pass^{}"],
        repo,
    );
    let branch = git_short(&["branch", "--show-current"], repo);
    let status = git_short(&["status", "--short"], repo);
    let dirty_candidate = status.lines().any(|l| {
        let f = l.trim_start_matches(|c: char| c == 'M' || c == 'A' || c == '?' || c == ' ');
        f.contains("material_mesh")
            || f.contains("mesh_mechanics")
            || f.contains("mesh_reactions")
            || f.contains("mesh_transport")
    });
    let hits = scan_source_tree(repo);
    let forbidden = forbidden_controller_failures(&hits);
    IntegrityReport {
        entry_commit_ok: head.starts_with("6f8a80a") || head == "6f8a80a",
        tag_ok: tag.starts_with("6f8a80a"),
        branch,
        head,
        tag_commit: tag,
        dirty_candidate_source: dirty_candidate,
        pattern_hits: hits.len(),
        forbidden_failures: forbidden,
    }
}
