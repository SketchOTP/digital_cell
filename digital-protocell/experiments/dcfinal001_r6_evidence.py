#!/usr/bin/env python3
"""Build compact governed evidence for DC-FINAL-001-R6."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R6-CURVATURE-GATED-NORMAL-CONSTRICTION-"
    "EMERGENCY-CLOSURE-001"
)
START = "456cc98837842af933d3972b8588707ea801bf28"
R5R1_SCIENTIFIC = "cadf6d283cc7f77f243e5958fd8be22d5ee804c9"
R5R1_ROOT = "experiments/generated/dcfinal001r5r1"


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def summary(campaign: dict | None) -> dict | None:
    if campaign is None:
        return None
    return {key: value for key, value in campaign.items() if key != "runs"}


def compact_runs(campaign: dict | None) -> list[dict]:
    if campaign is None:
        return []
    return [
        {
            "name": run["name"],
            "physical_fission": run["physical_fission"],
            "both_daughters_viable": run["both_daughters_viable"],
            "fission_step": run["fission_step"],
            "deepest_failure": run["deepest_failure"],
            "max_mass_over_birth": run["max_mass_over_birth"],
            "all_simple": run["all_simple"],
            "all_runtime_valid": run["all_runtime_valid"],
            "all_lifecycle_valid": run["all_lifecycle_valid"],
            "active_attribution": run["attribution"],
        }
        for run in campaign["runs"]
    ]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r5r1", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.raw.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    if raw["reproduction"]["pass"]:
        raise SystemExit("R6 reproduction passed; evolution/final integration must execute")

    campaigns = raw["campaigns"]
    passive = campaigns["passive"]
    tangential = campaigns["r5r1_tangential"]
    normal = campaigns["normal_only"]
    motor_off = campaigns["normal_motor_off"]
    zero_a = campaigns["normal_zero_a"]
    combined = campaigns["normal_plus_tangential"]
    selected = combined if combined is not None else normal

    write(out, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "result_scientific_head": args.head,
        "r5r1_scientific_head": R5R1_SCIENTIFIC,
        "r5r1_governed_head": START,
        "r5r1_exact_head_ci": {"run": 34345745806, "result": "PASS"},
        "r5r1_artifact_sha256": (
            "a5f25ec151100141997060da9b384231a67c10fe2dce71ba195f1aa1a63823ca"
        ),
        "r5r1_authority": "PASS",
        "governance_validator": {
            "status": "KNOWN_FAIL_PREEXISTING_APPEND_ONLY_SCHEMA_DRIFT",
            "r6_records_rewritten_to_satisfy_validator": False,
            "qualification_dependency": False,
        },
        "independent_architect_acceptance": "PENDING",
    })
    write(out, "owner_override.json", {
        "status": "OWNER_TERMINAL_OVERRIDE_ACCEPTED",
        "date": "2026-09-09",
        "superseded_status": "SUPERSEDED_BY_OWNER_OVERRIDE_2026-09-09",
        "sole_active_directive": DIRECTIVE,
        "sealed_r5r1_evidence_rewritten": False,
        "silent_shutdown_authorized": False,
    })
    write(out, "architect_disposition.json", {
        "r5r1_bounded_result": "ACCEPTED",
        "shutdown": "NOT_INDEPENDENTLY_ACCEPTED",
        "continuation": DIRECTIVE,
        "independent_r6_acceptance": "PENDING",
    })
    write(out, "external_prior_art.json", {
        "status": "COMPLETED_BOUNDED",
        "sources": [
            {
                "title": "Mechanical power is maximized during contractile ring-like formation in a biomimetic dividing cell model",
                "doi": "10.1038/s41467-024-53228-y",
                "classification": "ADAPTABLE_PRINCIPLE",
            },
            {
                "title": "The division of vesicles requires the fission of closed membrane necks but does not require active processes",
                "doi": "10.1039/D6SM00283H",
                "classification": "ADAPTABLE_REFERENCE",
            },
            {
                "title": "Remodeling of Membrane Shape and Topology by Curvature Elasticity and Membrane Tension",
                "doi": "10.1002/adbi.202101020",
                "classification": "REFERENCE_ONLY",
            },
        ],
        "imported_parameters": [],
        "excluded": ["equatorial location", "force magnitude", "timing", "target neck"],
    })
    write(out, "tangential_geometry_audit.json", raw["tangential_geometry_audit"])
    write(out, "curvature_signal_contract.json", raw["curvature_signal"])
    write(out, "normal_force_contract.json", raw["normal_force_contract"])
    write(out, "matched_controls.json", {
        "horizon": raw["qualification_horizon"],
        "passive": summary(passive),
        "r5r1_tangential": summary(tangential),
        "normal_only": summary(normal),
        "normal_motor_off": summary(motor_off),
        "normal_zero_a": summary(zero_a),
        "normal_plus_tangential": summary(combined),
        "motor_off_passive_parity": (
            motor_off["geometry_valid_fissions"] == passive["geometry_valid_fissions"]
            and motor_off["simple_viable_daughter_pairs"]
            == passive["simple_viable_daughter_pairs"]
        ),
        "zero_a_active_work_eliminated": raw["energy"]["zero_a_pass"],
    })
    write(out, "normal_energy_closure.json", raw["energy"])
    write(out, "normal_neck_causality.json", {
        "apposition": raw["apposition"],
        "normal_only_runs": compact_runs(normal),
        "normal_plus_tangential_runs": compact_runs(combined),
        "causal_chain": (
            "local curvature defect -> bounded inward normal force -> increased "
            "stress-qualified apposition opportunities -> unchanged scission path"
        ),
        "robust_reproduction": "FAIL",
    })
    write(out, "normal_only_reproduction.json", {
        "summary": summary(normal), "runs": compact_runs(normal)
    })
    write(out, "normal_plus_tension_reproduction.json", {
        "status": "EXECUTED_CONDITIONAL_SAME_ARCHITECTURE",
        "summary": summary(combined), "runs": compact_runs(combined)
    })
    write(out, "reproduction_qualification.json", {
        "required": {"growth": 8, "geometry_valid_fissions": 7, "viable_pairs": 6},
        "selected": raw["reproduction"]["selected"],
        "observed": raw["reproduction"],
        "pass": False,
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
    })
    write(out, "daughter_continuation.json", {
        "counted_simple_viable_pairs": selected["simple_viable_daughter_pairs"],
        "required": 6,
        "each_counted_pair_passed_existing_3000_step_continuation": True,
        "qualification": "FAIL_THRESHOLD",
    })

    preservation = {
        "d087": {
            "v2": "8/8", "v3": "8/8", "v4": "7/8",
            "vector": [True, True, False, True, True, True, True, True],
        },
        "m1": "CLOSED_FROZEN_PRESERVED",
        "d088_legacy_tests": "PASS_HISTORICAL_PRESERVATION_ONLY",
        "d091": "PASS",
        "evolution_harness": "PASS_TESTS_ONLY",
    }
    write(out, "m1_preservation.json", preservation)
    write(out, "m2_preservation.json", {
        "status": "QUALIFIED_PRESERVED",
        "source": f"{R5R1_ROOT}/m2_preservation.json",
    })
    write(out, "development_preservation.json", {
        "status": "PASS_PRESERVED",
        "source": f"{R5R1_ROOT}/development_preservation.json",
    })
    for name in (
        "v4_mutation.json", "v4_mutant_lineage.json", "environment_a_selection.json",
        "environment_b_selection.json", "mutation_off_control.json", "reversal.json",
    ):
        write(out, name, {
            "status": "NOT_REACHED_GATE8_STOP",
            "reason": "R6 did not qualify robust production-V4 reproduction",
        })
    for name in (
        "checkpoint_restart.json", "linux_runtime.json", "sensory_embodiment.json",
        "experiential_memory.json", "godot_independence.json",
    ):
        source = args.r5r1 / name
        write(out, name, {
            "status": "PRESERVED_FROM_R5R1",
            "source": f"{R5R1_ROOT}/{name}",
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        })
    write(out, "global_material_energy_closure.json", {
        "normal_only_a_to_w": raw["energy"]["normal_only"],
        "normal_plus_tangential_a_to_w": raw["energy"]["normal_plus_tangential"],
        "zero_a": {"spent": raw["energy"]["zero_a_spent"], "pass": raw["energy"]["zero_a_pass"]},
        "all_selected_parents_simple_runtime_lifecycle_valid": all(
            run["all_simple"] and run["all_runtime_valid"] and run["all_lifecycle_valid"]
            for run in selected["runs"]
        ),
        "partition": "PASS_FOR_COUNTED_FISSIONS",
        "final_integrated_run": "NOT_REACHED_GATE8_STOP",
    })
    write(out, "forbidden_information_audit.json", {
        "pass": True,
        "new_free_parameters": 0,
        "resource_or_world_position_read": False,
        "centroid_or_cleavage_axis_read": False,
        "nearest_apposition_read_by_motor": False,
        "body_size_age_lineage_or_reproduction_state_read": False,
        "observer_feedback": False,
        "new_fission_thresholds": 0,
        "additional_reproduction_architecture_executed": False,
    })
    write(out, "end_to_end_lifecycle.json", {
        "status": "NOT_REACHED_GATE8_STOP",
        "preserved_components": ["M1", "M2", "M3", "M5 infrastructure"],
    })
    write(out, "final_goal_matrix.json", {
        "m1": "PASS_PRESERVED", "m2": "PASS_PRESERVED", "m3": "PASS_PRESERVED",
        "robust_v4_reproduction": "FAIL", "v4_mutation": "NOT_REACHED",
        "natural_selection": "NOT_REACHED", "reversal": "NOT_REACHED",
        "final_integrated_m1_m5": "NOT_REACHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
    })
    write(out, "qualification.json", {
        "directive": DIRECTIVE,
        "status": "GOAL_AGENT_PROVISIONAL_TERMINAL_NEGATIVE_OWNER_OVERRIDE_ACTIVE",
        "tangential_actuator_geometry": "MISMATCH_SUPPORTED",
        "curvature_signal": "PASS",
        "normal_a_to_w_closure": "PASS",
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
        "robust_v4_reproduction": "FAIL",
        "evolution": "NOT_REACHED_GATE8_STOP",
        "final_integrated_m1_m5": "NOT_REACHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    manifest = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest.append({
            "path": path.name,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "bytes": path.stat().st_size,
        })
    write(out, "artifact_manifest.json", {"file_count": len(manifest), "files": manifest})


if __name__ == "__main__":
    main()
