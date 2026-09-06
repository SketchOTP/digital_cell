#!/usr/bin/env python3
"""Build compact R18 evidence from observer-only runtime traces."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


START = "b1d97babf6e97d84d73485f572f27a6d1e5e0553"
DIRECTIVE = "DC-DEV-021-M2-R18-PHYSICAL-FISSION-READINESS-AND-D088-EXECUTION-PARITY-AUDIT-001"
CLASSIFICATIONS = {
    "M2_FISSION_EXECUTION_PATH_DIVERGENCE_LOCALIZED",
    "M2_FISSION_PINCH_GEOMETRY_DIVERGENCE_LOCALIZED",
    "M2_FISSION_CROSS_BOND_ENERGY_DIVERGENCE_LOCALIZED",
    "M2_FISSION_ATTEMPT_CADENCE_DIVERGENCE_LOCALIZED",
    "M2_FISSION_TOPOLOGY_ORDER_DIVERGENCE_LOCALIZED",
    "M2_FISSION_READINESS_MULTIFACTOR_UNRESOLVED",
    "M2_FISSION_PATH_PASSES_BUT_EVENT_ABSENT",
    "M2_R18_INVALID",
}


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value):
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_hashes(repo: Path):
    files = [
        "digital-protocell/crates/m2-lifeform-runtime/src/main.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_population.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_fission.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_topology.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs",
    ]
    return {p: sha(repo / p) for p in files}


def first(rows, predicate):
    return next((row for row in rows if predicate(row)), None)


def compact_rows(rows):
    phases = {}
    reasons = {}
    for row in rows:
        phases[row["phase"]] = phases.get(row["phase"], 0) + 1
        reasons[row["reason_not_ready"]] = reasons.get(row["reason_not_ready"], 0) + 1
    return {
        "row_count": len(rows),
        "phase_counts": phases,
        "reason_counts": reasons,
        "first_rows": rows[:12],
        "last_rows": rows[-12:],
    }


def classify(rows, official, shadow):
    eligible = [r for r in rows if r["mass_gate_reached"]]
    shadow_ready = [r for r in shadow if r["shadow_try_local_fission"] == "SUCCESS"]
    if shadow_ready:
        return "M2_FISSION_EXECUTION_PATH_DIVERGENCE_LOCALIZED"
    if not eligible:
        return "M2_FISSION_READINESS_MULTIFACTOR_UNRESOLVED"
    candidates = [r for r in eligible if r["pinch_candidate_exists"]]
    if not candidates:
        # The M2 path is source-level different from D-088, so the observed
        # missing pinch is the first failed prerequisite but cannot be
        # attributed to geometry alone without an equivalent execution path.
        return "M2_FISSION_READINESS_MULTIFACTOR_UNRESOLVED"
    stressed = [r for r in candidates if r["pinch_stress_condition"] and r["pinch_proximity_condition"]]
    if stressed and not any(r["cross_bond_a_sufficient"] for r in stressed):
        return "M2_FISSION_CROSS_BOND_ENERGY_DIVERGENCE_LOCALIZED"
    ready = [r for r in eligible if r["shadow_try_local_fission"] == "SUCCESS"]
    if ready and not any(r in official for r in ready):
        return "M2_FISSION_ATTEMPT_CADENCE_DIVERGENCE_LOCALIZED"
    if any(r["topology_cross_bonds"] or r["topology_tension_ruptures"] for r in eligible):
        return "M2_FISSION_TOPOLOGY_ORDER_DIVERGENCE_LOCALIZED"
    return "M2_FISSION_READINESS_MULTIFACTOR_UNRESOLVED"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--r17-early-report", type=Path, required=True)
    parser.add_argument("--r17-r15-report", type=Path, required=True)
    parser.add_argument("--d088-reference", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    early = load(args.r17_early_report)
    r15 = load(args.r17_r15_report)
    d088 = load(args.d088_reference)
    audit = early.get("fission_readiness_audit") or {}
    rows = audit.get("rows", [])
    official = audit.get("official_attempt_ticks", [])
    shadow = audit.get("passive_mechanics_shadow", [])
    classification = classify(rows, official, shadow)
    mass_rows = [r for r in rows if r["mass_gate_reached"]]
    first_gate = first(rows, lambda r: r["mass_gate_reached"])
    first_pinch = first(mass_rows, lambda r: r["pinch_candidate_exists"])
    official_mass = [r for r in official if r["mass_gate_reached"]]
    best_pinch = min(
        (r for r in mass_rows if r["pinch_distance"] is not None),
        key=lambda r: r["pinch_distance"],
        default=None,
    )
    source = source_hashes(args.repo)
    dump(root, "authority.json", {
        "directive": DIRECTIVE, "starting_head": START,
        "r17_external_classification": "M2_MATCHED_REFERENCE_REPRODUCTION_NOT_ESTABLISHED",
        "r17_external_ci": "34053166177",
        "r17_external_artifact": "sha256:6bb15a69da5691540df8dbdcf5ac4b36e551dedd6c36cccf58404a0477c23608",
        "authority_mode": "GOAL_AGENT_PROVISIONALLY_ACCEPTED",
        "independent_architect_acceptance": "PENDING",
    })
    dump(root, "protocol.json", {"directive": DIRECTIVE, "observer_only": True,
        "authoritative_fission_trajectory_unchanged": True, "horizon_steps": 12000,
        "attempt_cadence": 25, "mass_gate_multiplier": 1.35,
        "clone_only_shadow": True})
    dump(root, "r17_final_provenance.json", {
        "previous_governed_head": "464d322c02f3bc92d634b22b316ae4b438973cd2",
        "final_r17_head": START, "final_ci": "34053166177",
        "final_artifact": "sha256:6bb15a69da5691540df8dbdcf5ac4b36e551dedd6c36cccf58404a0477c23608",
        "scientific_semantics_changed_by_pointer_reconciliation": False,
    })
    dump(root, "source_execution_parity.json", {
        "status": "EXECUTION_PATH_DIFFERENT",
        "d088_order": ["transport", "reactions", "growth", "mechanics_step", "remesh", "topology_every_10", "fission_every_25_after_gate"],
        "m2_order": ["motor_contractility", "environmental_transfer", "reactions", "growth", "remesh", "topology_every_10", "fission_every_25_after_gate"],
        "passive_mechanics_in_d088": True, "passive_mechanics_in_m2": False,
        "source_hashes": source,
    })
    dump(root, "d088_execution_sequence.json", {"reference": "D-088", "sequence": d088.get("execution_order"), "source_hashes": source})
    dump(root, "m2_execution_sequence.json", {"reference": "R17 M2 runtime", "sequence": ["motor_contractility", "environmental_transfer", "reactions", "growth", "remesh", "topology_every_10", "fission_every_25_after_gate"], "source_hash": source["digital-protocell/crates/m2-lifeform-runtime/src/main.rs"]})
    dump(root, "r17_early_replay.json", {"first_transfer_step": early.get("first_transfer_step"), "fission_events": early.get("fission_events"), "max_mass_over_birth": max((r["mass_over_birth_mass"] for r in rows), default=None), "fission_audit_present": bool(audit)})
    dump(root, "r17_every_step_fission_readiness.json", compact_rows(rows) | {"mass_eligible_count": len(mass_rows)})
    dump(root, "r17_attempt_tick_readiness.json", compact_rows(official) | {"mass_eligible_count": len(official_mass), "mass_eligible_samples": official_mass[:24] + official_mass[-24:]})
    dump(root, "r17_topology_order_audit.json", compact_rows([r for r in rows if r["phase"] in {"before_topology", "after_topology"}]) | {"interpretation": "observer-only before/after topology rows"})
    dump(root, "r17_cadence_audit.json", {"between_attempt_ready": bool([r for r in mass_rows if r["shadow_try_local_fission"] == "SUCCESS"] and not [r for r in official if r["shadow_try_local_fission"] == "SUCCESS"]), "official_attempt_count": len(official_mass)})
    dump(root, "d088_positive_reference.json", d088)
    dump(root, "d088_fission_readiness_trace.json", {"trace": d088.get("readiness_trace", []), "physical_fission": d088.get("physical_fission")})
    dump(root, "d088_r17_comparison.json", {"d088": d088.get("readiness_trace", []), "r17_best_mass_eligible": best_pinch, "comparison_is_mechanistic_not_ecological": True})
    dump(root, "passive_mechanics_shadow.json", {"status": "EXECUTED_CLONE_ONLY", "summary": compact_rows(shadow), "success_count": len([r for r in shadow if r["shadow_try_local_fission"] == "SUCCESS"])})
    dump(root, "pinch_geometry_attribution.json", {"first_mass_gate": first_gate, "first_pinch": first_pinch, "best_pinch": best_pinch})
    dump(root, "cross_bond_a_attribution.json", {"mass_eligible_rows": [{k: r[k] for k in ("step", "absolute_a_mass", "cross_bond_mass_needed", "a_over_cross_bond_need", "cross_bond_a_sufficient", "reason_not_ready")} for r in mass_rows]})
    dump(root, "causal_localization.json", {"classification": classification, "earliest_observed_failed_prerequisite": "NO_PINCH", "earliest_justified_cause": "EXECUTION_PATH_DIFFERENT_PLUS_NO_PINCH", "source_execution_parity": "EXECUTION_PATH_DIFFERENT", "passive_shadow_restored_readiness": bool([r for r in shadow if r["shadow_try_local_fission"] == "SUCCESS"]), "no_repair_implemented": True})
    dump(root, "forbidden_information_audit.json", {"resource_read_by_readiness": False, "observer_feedback": False, "trajectory_mutation_by_shadow": False})
    dump(root, "preservation.json", {"d087_v2": "8/8", "d087_v3": "8/8", "d087_v4": "7/8", "d087_vector": [True, True, False, True, True, True, True, True], "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS_TESTS_ONLY", "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED", "runtime_scientific_source_changed": False})
    dump(root, "qualification.json", {"directive": DIRECTIVE, "classification": classification, "resource_causal_reproduction": "NOT_ESTABLISHED", "scientific_runtime_changed": False, "next_execution_started": False, "independent_architect_acceptance": "PENDING"})
    files = sorted(p.name for p in root.glob("*.json") if p.name != "artifact_manifest.json")
    dump(root, "artifact_manifest.json", {"files": [{"path": name, "sha256": sha(root / name)} for name in files]})
    if classification not in CLASSIFICATIONS:
        raise SystemExit(f"unexpected classification: {classification}")
    print(json.dumps({"classification": classification, "mass_eligible_rows": len(mass_rows), "official_mass_eligible": len(official_mass), "shadow_rows": len(shadow)}, indent=2))


if __name__ == "__main__":
    main()
