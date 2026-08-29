#!/usr/bin/env python3
"""Create the compact, machine-readable post-M1 extraction inventory."""

from __future__ import annotations

import json
import subprocess
from collections import Counter
from pathlib import Path

BASE = "1e242f28152797b512e25cd56c7b718e45d6ca97"
SOURCE = "fb77f472b1519a9e0f713833efba5b1d403f4723"
OUT = Path("digital-protocell/experiments/generated/dcdev020postm1baseline001/capability_inventory.json")


def git(*args: str) -> list[str]:
    return subprocess.check_output(["git", *args], text=True).splitlines()


def classify(path: str, retained: bool) -> str:
    if path.startswith("digital-protocell/experiments/generated/") or path.startswith("experiments/generated/"):
        return "GENERATED_EVIDENCE_ONLY"
    if path.startswith(".agent/"):
        return "REQUIRED_CURRENT_GOVERNANCE" if retained else "HISTORICAL_PROVENANCE_ONLY"
    if path.startswith(".github/workflows/"):
        return "REQUIRED_BUILD_CONFIGURATION" if retained else "SUPERSEDED_EXPERIMENT"
    if path.endswith(("Cargo.toml", "Cargo.lock")) or path.startswith(".cargo/"):
        return "REQUIRED_BUILD_CONFIGURATION"
    if "/tests/" in path or "/examples/" in path or path.startswith("scripts/"):
        return "REQUIRED_TEST_OR_CERTIFIER" if retained else "SUPERSEDED_EXPERIMENT"
    if path.startswith("digital-protocell/crates/chemistry-core/") or path.startswith("digital-protocell/crates/phase1-certifier/"):
        return "REQUIRED_PRODUCTION_RUNTIME" if retained else "OBSERVER_DIAGNOSTIC_ONLY"
    if path.startswith("digital-protocell/crates/regulatory-core/") or path.startswith("digital-protocell/crates/evolution-harness/"):
        return "REQUIRED_FUTURE_CAPABILITY" if retained else "OBSERVER_DIAGNOSTIC_ONLY"
    if path.startswith("digital-protocell/docs/") or path.startswith("docs/"):
        return "REQUIRED_CURRENT_GOVERNANCE" if retained else "HISTORICAL_PROVENANCE_ONLY"
    if retained:
        return "REQUIRED_CURRENT_GOVERNANCE"
    return "HISTORICAL_PROVENANCE_ONLY"


def main() -> None:
    changed = git("diff", "--name-only", BASE, SOURCE)
    baseline_changed = git("diff", "--name-only", BASE, "HEAD")
    baseline_status = git("diff", "--name-status", BASE, "HEAD")
    baseline_paths = set(git("ls-tree", "-r", "--name-only", "HEAD"))
    records = []
    for path in changed:
        retained = path in baseline_paths
        records.append({"path": path, "classification": classify(path, retained), "retained": retained})
    categories = Counter(item["classification"] for item in records)
    payload = {
        "schema": "dcdev020postm1baseline001-capability-inventory-v1",
        "directive": "DC-DEV-020-POST-M1-BASELINE-001-CLEAN-CAPABILITY-BASELINE-001",
        "source_closure_head": SOURCE,
        "source_base": BASE,
        "baseline_branch": "baseline/m1-v4-closed",
        "baseline_tree_ref": "HEAD",
        "source_changed_file_count": len(changed),
        "source_commit_count_above_base": int(git("rev-list", "--count", f"{BASE}..{SOURCE}")[0]),
        "clean_baseline_changed_file_count": len(baseline_changed),
        "clean_baseline_deleted_file_count": sum(line.startswith("D") for line in baseline_status),
        "historical_experiment_workflow_files_omitted": sum(
            line.startswith("D") and ("experiments/generated/" in line or ".github/workflows/" in line or ".agent/legacy/" in line)
            for line in baseline_status
        ),
        "classification_counts": dict(sorted(categories.items())),
        "dependency_proof": {
            "cargo_workspace": "digital-protocell/Cargo.toml",
            "packaged_runtime": "digital-protocell/crates/phase1-certifier/src/bin/phase1_runtime.rs",
            "production_selector": "digital-protocell/crates/phase1-certifier/src/sim.rs::seed_production_mesh",
            "governance_validator": "scripts/validate_governance.py",
            "omission_rule": "Only generated evidence, superseded diagnostic workflows, and historical-only records absent from the retained capability tree are omitted.",
        },
        "changed_paths": records,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT} ({len(records)} source-changed paths classified)")


if __name__ == "__main__":
    main()
