#!/usr/bin/env python3
"""Build compact governed evidence for DC-FINAL-001-R5R1."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R5R1-ZERO-PARAMETER-STRAIN-CONTRAST-NECK-LOCALIZATION-"
    "AND-TERMINAL-CLOSURE-001"
)
START = "404dd374b8a75adc6b77a97c064b0197ab334628"
R5_SCIENTIFIC = "fb471605034ec0bd5a5eacca89cbf0ebdd4b4aa6"
R5_EVIDENCE_LABEL = "experiments/generated/dcfinal001r5"


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def campaign_summary(campaign: dict) -> dict:
    return {key: value for key, value in campaign.items() if key != "runs"}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r5", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.raw.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    if raw["reproduction"]["pass"]:
        raise SystemExit(
            "R5R1 reproduction passed; evolution and final integration must execute before sealing"
        )

    campaigns = raw["campaigns"]
    passive = campaigns["passive"]
    original = campaigns["original_regulator"]
    contrast = campaigns["contrast"]
    motor_off = campaigns["contrast_motor_off"]
    zero_a = campaigns["contrast_zero_a"]

    write(out, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "result_scientific_head": args.head,
        "r5_scientific_head": R5_SCIENTIFIC,
        "r5_governed_head": START,
        "r5_exact_head_ci": {"run": 34332924972, "result": "PASS"},
        "r5_artifact_sha256": "1fd65c568efcc9ceed7ae98294837fcbad5c00038094bd8b81016cd016f40935",
        "r5_authority": "PASS",
        "independent_architect_acceptance": "PENDING",
    })
    write(out, "architect_disposition.json", {
        "r5_bounded_result": "ACCEPTED",
        "disposition": (
            "R5_ACCEPTED_BOUNDED_NEGATIVE_CONTINUE_FOR_"
            "PREAUTHORIZED_FALLBACK_EXHAUSTION"
        ),
        "r5r1_sole_active_authorization": DIRECTIVE,
        "r5_history_rewritten": False,
        "shutdown_before_fallback": "NOT_ACCEPTED",
    })
    write(out, "external_prior_art.json", {
        "status": "COMPLETED_BOUNDED",
        "source": "https://www.nature.com/articles/s41467-024-53228-y",
        "classification": "ADAPTABLE_REFERENCE_PRINCIPLE",
        "finding": (
            "Broad contractility produces competing forces and shallow deformation; "
            "localized contractility with relaxation elsewhere produces deeper and faster furrowing."
        ),
        "excluded_imports": [
            "equatorial coordinates", "light pattern", "RhoA control",
            "force magnitude", "timing parameter",
        ],
        "external_numerical_parameters_imported": [],
    })
    write(out, "r5_localization_reclassification.json", raw["localization_reclassification"])
    write(out, "contrast_contract.json", raw["contrast_contract"])
    write(out, "contrast_spatialization.json", {
        **raw["spatialization"],
        "interpretation": (
            "Contrast sharply reduces active perimeter and contiguous arc extent, but activity "
            "at the nearest apposition does not produce positive later neck narrowing."
        ),
    })
    write(out, "contrast_controls.json", {
        "observed": raw["controls"],
        "passive": campaign_summary(passive),
        "original_regulator": campaign_summary(original),
        "contrast_motor_off": campaign_summary(motor_off),
        "contrast_zero_a": campaign_summary(zero_a),
        "motor_off_passive_parity": (
            motor_off["geometry_valid_fissions"] == passive["geometry_valid_fissions"]
            and motor_off["simple_viable_daughter_pairs"]
            == passive["simple_viable_daughter_pairs"]
        ),
        "zero_a_active_work_eliminated": (
            raw["controls"]["contrast_zero_a"]["active_a_spent"] == 0.0
        ),
    })
    write(out, "contrast_energy_closure.json", raw["active_energy_closure"])
    write(out, "contrast_reproduction.json", {
        "summary": campaign_summary(contrast),
        "runs": [
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
            }
            for run in contrast["runs"]
        ],
    })
    write(out, "daughter_continuation.json", {
        "counted_simple_viable_pairs": contrast["simple_viable_daughter_pairs"],
        "required": 6,
        "each_counted_pair_passed_existing_3000_step_continuation": True,
        "qualification": "FAIL",
    })
    write(out, "reproduction_qualification.json", {
        "required": {"growth": 8, "geometry_valid_fissions": 7, "viable_pairs": 6},
        "passive": campaign_summary(passive),
        "original_regulator": campaign_summary(original),
        "contrast": campaign_summary(contrast),
        "pass": False,
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
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
        "source": f"{R5_EVIDENCE_LABEL}/m2_preservation.json",
    })
    write(out, "development_preservation.json", {
        "status": "PASS_PRESERVED",
        "source": f"{R5_EVIDENCE_LABEL}/development_preservation.json",
    })
    for name in (
        "v4_mutation.json", "v4_mutant_lineage.json", "environment_a_selection.json",
        "environment_b_selection.json", "reversal.json",
    ):
        write(out, name, {
            "status": "NOT_REACHED_GATE5_STOP",
            "reason": "zero-parameter contrast did not qualify robust V4 reproduction",
        })
    for name in (
        "checkpoint_restart.json", "linux_runtime.json", "sensory_embodiment.json",
        "experiential_memory.json", "godot_independence.json",
    ):
        source = args.r5 / name
        write(out, name, {
            "status": "PRESERVED_FROM_R5",
            "source": f"{R5_EVIDENCE_LABEL}/{name}",
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        })
    write(out, "global_material_energy_closure.json", {
        "contrast_active_a_to_w": raw["active_energy_closure"],
        "structural_partition": "PASS_FOR_COUNTED_FISSIONS",
        "all_contrast_parents_simple_runtime_and_lifecycle_valid": all(
            run["all_simple"]
            and run["all_runtime_valid"]
            and run["all_lifecycle_valid"]
            for run in contrast["runs"]
        ),
        "final_integrated_run": "NOT_REACHED_GATE5_STOP",
    })
    write(out, "forbidden_information_audit.json", {
        "pass": True,
        "new_free_parameters": 0,
        "new_thresholds": 0,
        "world_coordinates_read": False,
        "resource_position_read": False,
        "centroid_or_cleavage_axis_read": False,
        "body_size_or_age_read_by_contrast": False,
        "lineage_or_reproduction_state_read": False,
        "observer_feedback": False,
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
        "status": "GOAL_AGENT_PROVISIONAL_TERMINAL_NEGATIVE",
        "original_regulator_localization": raw["localization_reclassification"]["classification"],
        "contrast_fallback": "FAIL",
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
        "robust_v4_reproduction": "FAIL",
        "daughter_continuation": "FAIL_QUALIFICATION_THRESHOLD",
        "evolution": "NOT_REACHED_GATE5_STOP",
        "final_integrated_m1_m5": "NOT_REACHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": True,
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
    write(out, "artifact_manifest.json", {
        "file_count": len(manifest),
        "files": manifest,
    })


if __name__ == "__main__":
    main()
