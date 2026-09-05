#!/usr/bin/env python3
"""Emit compact evidence for the opt-in assimilation preservation audit.

This script records preservation scope and provenance only.  It does not run a
new organism/world composition and it does not promote assimilation to project
architecture.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


START_HEAD = "721172711a31ac80c6fa798474ffeef519eb963c"
CLASSIFICATION = "GOAL_AGENT_PROVISIONALLY_ACCEPTED_PRESERVATION_AUDIT"


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output
    root.mkdir(parents=True, exist_ok=True)

    protocol = {
        "directive": "DC-DEV-021-GOAL-ASSIMILATION-PRESERVATION-AUDIT-R9",
        "starting_head": START_HEAD,
        "scope": "preservation audit for the opt-in assimilation state extension",
        "scientific_runtime_mechanism_added": False,
        "assimilation_architecture_status": "INVESTIGATE_NOT_ACCEPTED",
        "resource_causal_fission": "NOT_ESTABLISHED",
        "closed_family": "CLOSURE-006-through-CLOSURE-014",
        "stop_condition": (
            "Any preservation failure rejects or replans assimilation; a pass "
            "does not authorize another material-flow variant."
        ),
    }
    preservation = {
        "physical_runtime_validity": {
            "status": "TESTED",
            "coverage": "assimilation_n and assimilation_f are finite-checked",
        },
        "legacy_schema_defaults": {
            "status": "TESTED",
            "coverage": "missing assimilation fields deserialize as zero",
        },
        "geometry_and_remesh_conservation": {"status": "TESTED"},
        "fission_partitioning": {"status": "TESTED"},
        "observer_death_semantics": {
            "status": "TESTED",
            "coverage": "assimilated nutrient participates without a new latch",
        },
        "checkpoint_serialization": {"status": "TESTED"},
        "d087_m1_preservation": {
            "status": "REUSED_SEALED_EVIDENCE",
            "v2": "8/8",
            "v3": "8/8",
            "v4": "7/8",
            "v4_vector": [True, True, False, True, True, True, True, True],
        },
        "d088": {"status": "TESTED"},
        "d091": {"status": "TESTED"},
        "downstream_m2_runtime": {"status": "TESTED"},
    }
    commands = [
        "cargo +1.89.0 test --locked -p chemistry-core --test goal_assimilation_preservation -- --nocapture",
        "cargo +1.89.0 test --locked -p chemistry-core --test d094_tests -- --nocapture",
        "cargo +1.89.0 test --locked -p chemistry-core --test d098_geometry_material_conservation -- --nocapture",
        "cargo +1.89.0 test --locked -p chemistry-core --test d088_tests -- --nocapture",
        "cargo +1.89.0 test --locked -p chemistry-core --test d091_tests -- --nocapture",
        "cargo +1.89.0 test --locked -p m2-lifeform-runtime -- --nocapture",
    ]
    qualification = {
        "classification": CLASSIFICATION,
        "independent_architect_acceptance": "PENDING",
        "preservation_status": "PASS_IF_WORKFLOW_CHECKS_PASS",
        "assimilation_architecture": "INVESTIGATE_NOT_ACCEPTED",
        "resource_causal_reproduction": "NOT_ESTABLISHED",
        "scientific_runtime_source_changed": False,
        "test_commands": commands,
        "next_execution_started": False,
        "architecture_decision": (
            "Preservation is necessary but not sufficient. Do not add another "
            "pool, buffer, field-placement, allocation, or active-work variant "
            "without a new source-justified organism/world material contract."
        ),
    }
    write_json(root / "protocol.json", protocol)
    write_json(root / "preservation_matrix.json", preservation)
    write_json(root / "qualification.json", qualification)

    files = [root / name for name in ("protocol.json", "preservation_matrix.json", "qualification.json")]
    write_json(
        root / "artifact_manifest.json",
        {
            "files": [
                {"path": path.name, "sha256": digest(path)}
                for path in files
            ],
            "manifest_excludes": ["artifact_manifest.json"],
        },
    )


if __name__ == "__main__":
    main()
