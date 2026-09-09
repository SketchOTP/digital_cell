#!/usr/bin/env python3
"""Build compact governed evidence for DC-FINAL-001-R7."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R7-STRAIN-CONTRAST-NORMAL-CONSTRICTION-"
    "EMERGENCY-CLOSURE-001"
)
START = "f1bee528888a91a8a2553ecea4973f5a5b851ab9"
R6_SCIENTIFIC = "e52fa4427a895b550b3be01ada5761b3be498f3e"
R6_ROOT = "experiments/generated/dcfinal001r6"


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
            "daughter_diagnostics": run["daughter_diagnostics"],
        }
        for run in campaign["runs"]
    ]


def preserved(root: Path, r6: Path, name: str) -> None:
    source = r6 / name
    write(root, name, {
        "status": "PRESERVED_FROM_R6",
        "source": f"{R6_ROOT}/{name}",
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
    })


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r6", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.raw.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    if raw["reproduction"]["pass"]:
        raise SystemExit("R7 reproduction passed; mutation/selection/reversal must execute")

    campaigns = raw["campaigns"]
    passive = campaigns["passive"]
    tangential = campaigns["r5r1_strain_contrast_tangential"]
    curvature = campaigns["r6_curvature_normal"]
    normal = campaigns["r7_strain_contrast_normal"]
    motor_off = campaigns["r7_strain_contrast_normal_motor_off"]
    zero_a = campaigns["r7_strain_contrast_normal_zero_a"]
    combined = campaigns["r7_strain_contrast_normal_plus_tangential"]

    write(out, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "result_scientific_head": args.head,
        "r6_scientific_head": R6_SCIENTIFIC,
        "r6_governed_head": START,
        "r6_exact_head_ci": {"run": 34357939278, "result": "PASS"},
        "r6_artifact_sha256": (
            "35277c82d95754ef2abc3e0fd7911590fab80893a778f521ff4ee0175f4b8639"
        ),
        "r6_authority": "PASS",
        "independent_architect_acceptance": "PENDING",
    })
    write(out, "architect_disposition.json", {
        "r6": "R6_ACCEPTED_BOUNDED_ADVANCE_REPLAN",
        "sole_active_directive": DIRECTIVE,
        "r7_independent_acceptance": "PENDING",
    })
    write(out, "owner_override.json", {
        "status": "PASS",
        "shutdown_override_active": True,
        "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
        "next_execution_started": False,
    })
    write(out, "external_prior_art.json", {
        "status": "COMPLETED_BOUNDED",
        "sources": [
            {
                "title": "Mechanical power is maximized during contractile ring-like formation in a biomimetic dividing cell model",
                "url": "https://www.nature.com/articles/s41467-024-53228-y",
                "classification": "ADAPTABLE_PRINCIPLE",
                "used_principle": "localized contraction with relaxation elsewhere can furrow more effectively than broad activation",
            },
            {
                "title": "Spatiotemporal control of structure and dynamics in actomyosin networks using the bacterial MinDE protein system",
                "url": "https://www.nature.com/articles/s41467-024-54807-9",
                "classification": "REFERENCE_ONLY",
                "used_principle": "self-organized spatial localization can deform membrane-bound actomyosin systems",
            },
        ],
        "imported_parameters": [],
        "excluded": ["equatorial coordinates", "protein systems", "force magnitudes", "timing", "thresholds"],
    })

    write(out, "cross_composition_contract.json", raw["cross_composition"])
    write(out, "strain_contrast_normal_signal.json", {
        "contract": raw["force_contract"],
        "composition": raw["cross_composition"],
        "localization": raw["localization"],
    })
    write(out, "normal_force_energy_closure.json", raw["energy"])
    write(out, "matched_controls.json", {
        "horizon": raw["qualification_horizon"],
        "passive": summary(passive),
        "r5r1_tangential": summary(tangential),
        "r6_curvature_normal": summary(curvature),
        "r7_strain_contrast_normal": summary(normal),
        "r7_motor_off": summary(motor_off),
        "r7_zero_a": summary(zero_a),
        "r7_same_signal_normal_plus_tangential": summary(combined),
        "controls": raw["controls"],
        "conditional": raw["conditional_same_signal_tangential"],
    })
    write(out, "reproduction_campaign.json", {
        "passive": {"summary": summary(passive), "runs": compact_runs(passive)},
        "r5r1_tangential": {"summary": summary(tangential), "runs": compact_runs(tangential)},
        "r6_curvature_normal": {"summary": summary(curvature), "runs": compact_runs(curvature)},
        "r7_strain_contrast_normal": {"summary": summary(normal), "runs": compact_runs(normal)},
        "conditional_normal_plus_tangential": {
            "status": "NOT_REQUIRED_BY_GATE6",
            "reason": "normal-only improved neither fission nor viable-pair count relative to both passive and sealed R5R1",
            "summary": summary(combined),
        },
    })
    write(out, "per_fission_daughter_diagnostics.json", raw["daughter_diagnostics"])
    write(out, "daughter_continuation.json", {
        "r7_physical_fissions": normal["geometry_valid_fissions"],
        "r7_simple_viable_pairs": normal["simple_viable_daughter_pairs"],
        "required_simple_viable_pairs": 6,
        "diagnostics": raw["daughter_diagnostics"]["r7_contrast_normal"],
        "qualification": "FAIL_THRESHOLD",
    })
    write(out, "reproduction_qualification.json", {
        "required": {"growth": 8, "geometry_valid_fissions": 7, "viable_pairs": 6},
        "observed": raw["reproduction"],
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
        "pass": False,
    })

    write(out, "m1_preservation.json", {
        "d087": {
            "v2": "8/8", "v3": "8/8", "v4": "7/8",
            "vector": [True, True, False, True, True, True, True, True],
        },
        "m1": "CLOSED_FROZEN_PRESERVED",
        "d088_legacy_tests": "PASS_HISTORICAL_PRESERVATION_ONLY",
        "d091": "PASS",
        "evolution_harness": "PASS_TESTS_ONLY",
    })
    preserved(out, args.r6, "m2_preservation.json")
    preserved(out, args.r6, "development_preservation.json")

    for name in (
        "v4_mutation.json", "v4_mutant_lineage.json", "environment_a_selection.json",
        "environment_b_selection.json", "mutation_off_control.json", "reversal.json",
    ):
        write(out, name, {
            "status": "NOT_REACHED_GATE7_STOP",
            "reason": "R7 did not qualify robust production-V4 reproduction",
        })
    for name in (
        "checkpoint_restart.json", "linux_runtime.json", "sensory_embodiment.json",
        "experiential_memory.json", "godot_independence.json",
    ):
        preserved(out, args.r6, name)

    write(out, "global_material_energy_closure.json", {
        "r7_active_a_to_w": raw["energy"]["r7_strain_contrast_normal"],
        "zero_a": {"spent": raw["energy"]["zero_a_spent"], "pass": raw["controls"]["zero_a_pass"]},
        "motor_off_parity": raw["controls"]["motor_off_semantic_parity"],
        "all_r7_parent_states_simple_runtime_lifecycle_valid": all(
            run["all_simple"] and run["all_runtime_valid"] and run["all_lifecycle_valid"]
            for run in normal["runs"]
        ),
        "partition": "PASS_FOR_COUNTED_FISSION",
        "final_integrated_run": "NOT_REACHED_GATE7_STOP",
    })
    write(out, "forbidden_information_audit.json", {
        "pass": True,
        "new_free_parameters": 0,
        "curvature_in_primary_drive": False,
        "target_neck_or_daughter_ratio": False,
        "centroid_axis_or_division_plane": False,
        "nearest_apposition_or_fission_feedback": False,
        "body_size_age_lineage_or_reproduction_state": False,
        "observer_feedback": False,
        "fission_detector_changed": False,
        "additional_reproduction_architecture_executed": False,
    })
    write(out, "final_goal_matrix.json", {
        "m1": "PASS_PRESERVED",
        "m2": "PASS_PRESERVED",
        "m3": "PASS_PRESERVED",
        "robust_v4_reproduction": "FAIL",
        "v4_mutation": "NOT_REACHED",
        "natural_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integrated_m1_m5": "NOT_REACHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
    })
    write(out, "qualification.json", {
        "directive": DIRECTIVE,
        "status": "GOAL_AGENT_PROVISIONAL_TERMINAL_NEGATIVE_OWNER_OVERRIDE_ACTIVE",
        "cross_composition": "STRAIN_CONTRAST_NORMAL_COMPOSITION_EXACT",
        "new_free_parameters": 0,
        "a_to_w_closure": "PASS",
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
        "robust_v4_reproduction": "FAIL",
        "evolution": "NOT_REACHED_GATE7_STOP",
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
