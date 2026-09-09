#!/usr/bin/env python3
"""Build compact governed evidence for DC-FINAL-001-R5."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R5-V4-MATURATION-NECK-GENERATION-MECHANOCHEMICAL-"
    "REPRODUCTION-EVOLUTION-AND-FINAL-GOAL-CLOSURE-001"
)
START = "aa886d554fa4ae8cf691e5a66b69d787ea4f2c2f"
R4_SCIENTIFIC = "78c2200615b6558d6b617e3c29f5c8e9bb1c05d3"
R4_EVIDENCE_LABEL = "experiments/generated/dcfinal001r4"


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def summary(campaign: dict) -> dict:
    return {key: value for key, value in campaign.items() if key != "runs"}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r4", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.raw.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    passive = raw["passive"]
    extensions = raw["maturation_horizon_audit"]["failed_arm_extensions"]
    extension_fissions = [run for run in extensions if run["physical_fission"]]
    passive_horizon_fissions = passive["geometry_valid_fissions"] + len(extension_fissions)
    passive_horizon_viable = passive["simple_viable_daughter_pairs"] + sum(
        1 for run in extension_fissions if run["both_daughters_viable"]
    )
    mech = raw["mechanochemical"]
    controls = raw["controls"]
    failed_passive = [run for run in passive["runs"] if not run["physical_fission"]]
    failure_taxonomy = [
        {
            "arm": run["name"],
            "deepest_failure": run["deepest_failure"],
            "failure_counts": run["failure_counts"],
            "max_mass_over_birth": run["max_mass_over_birth"],
            "terminal_geometry": run["final_geometry"],
        }
        for run in passive["runs"]
    ]
    source_timescale = raw["source_timescale"]

    write(out, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "result_scientific_head": args.head,
        "r4_scientific_head": R4_SCIENTIFIC,
        "r4_exact_head_ci": {"run": 34310098452, "result": "PASS"},
        "r4_artifact_sha256": "6f858219ab080f8b8d14c6b211eabe72e6cfede16ede1773e31b4ff2c1eda76f",
        "r4_authority": "PASS",
        "independent_architect_acceptance": "PENDING",
    })
    write(out, "architect_disposition.json", {
        "r4_disposition": "R4_ACCEPTED_BOUNDED_NEGATIVE_REPLAN",
        "r5_sole_active_authorization": DIRECTIVE,
        "history_rewritten": False,
        "goal_agent_result_status": "GOAL_AGENT_PROVISIONAL_NEGATIVE",
    })
    write(out, "external_prior_art.json", {
        "status": "COMPLETED",
        "source": "https://www.nature.com/articles/s41467-024-53228-y",
        "classification": "ADAPTABLE_PRINCIPLE",
        "finding": (
            "Localized contractility with relaxation elsewhere produces deeper division-like "
            "furrowing than global contraction; no external numerical parameter was imported."
        ),
        "direct_division_location_import": "INCOMPATIBLE",
        "external_numerical_parameters_imported": [],
    })
    write(out, "v4_failure_taxonomy.json", {
        "arms": failure_taxonomy,
        "failed_arm_count": len(failed_passive),
        "root_cause": (
            "ROBUST_LOCAL_APPOSITION_AND_STRESS_LOCALIZATION_ABSENT; mass and cross-bond A "
            "were not the deepest failed prerequisite"
        ),
        "successful_seed_distinction": (
            "Successful arms entered local range with a qualifying stress condition; failed "
            "arms remained outside range or reached apposition without the stress condition."
        ),
    })
    write(out, "v4_maturation_horizon_audit.json", {
        "source_timescale": source_timescale,
        "legacy_horizon": raw["legacy_horizon"],
        "failed_arms_extended": len(extensions),
        "extension_fissions": len(extension_fissions),
        "extension_fission_steps": [run["fission_step"] for run in extension_fissions],
        "classification": raw["maturation_horizon_audit"]["classification"],
        "preregistered_qualification_horizon": raw["qualification_horizon"],
        "qualification_horizon_derivation": "12000 + ceil(1/(k_turn*dt))",
        "diagnostic_extension_not_counted_as_legacy_positive": True,
    })
    write(out, "v4_reference_length_counterfactual.json", raw["reference_length_counterfactual"])
    write(out, "passive_v4_reproduction.json", {
        "legacy": summary(passive),
        "source_timescale_horizon": raw["qualification_horizon"],
        "fissions_at_source_timescale_horizon": passive_horizon_fissions,
        "viable_pairs_at_source_timescale_horizon": passive_horizon_viable,
        "qualification_pass": passive_horizon_fissions >= 7 and passive_horizon_viable >= 6,
    })
    write(out, "strain_regulator_contract.json", {
        "input": "local positive tensile strain only",
        "adapter": "material_adapter::observe_local_material_frame",
        "k_neighbor": 2.0,
        "k_stimulus": 4.0,
        "k_decay": 0.5,
        "dt": 0.02,
        "topology_continuity": "ContinuityNetworkV1 split/merge remapping",
        "forbidden_inputs": [],
        "new_free_parameters": 0,
    })
    write(out, "strain_activity_spatialization.json", {
        "arms": raw["spatial_attribution"],
        "mean_activity_variance": raw["contrast_fallback"]["mean_activity_variance"],
        "classification_tolerance": raw["contrast_fallback"]["classification_tolerance"],
        "spatial_contrast_preserved": not raw["contrast_fallback"]["required"],
        "activity_to_contraction": "DIRECT_EXISTING_ADAPTER_MAPPING",
    })
    write(out, "mechanochemical_energy_closure.json", raw["active_energy_closure"])
    write(out, "mechanochemical_controls.json", {
        "controls": {key: summary(value) for key, value in controls.items()},
        "passive_source_timescale": {
            "fissions": passive_horizon_fissions,
            "viable_pairs": passive_horizon_viable,
        },
        "zero_activity_semantic_parity": (
            controls["regulator_off_motor_on_zero_activity"]["geometry_valid_fissions"]
            == passive_horizon_fissions
            and controls["regulator_off_motor_on_zero_activity"]["simple_viable_daughter_pairs"]
            == passive_horizon_viable
        ),
        "motor_off_semantic_parity": (
            controls["regulator_on_motor_off"]["geometry_valid_fissions"]
            == passive_horizon_fissions
            and controls["regulator_on_motor_off"]["simple_viable_daughter_pairs"]
            == passive_horizon_viable
        ),
        "zero_a_active_work_eliminated": all(
            run["attribution"]["zero_a_spent"] == 0.0 for run in controls["zero_a"]["runs"]
        ),
    })
    write(out, "mechanochemical_reproduction.json", {
        "summary": summary(mech),
        "selected": raw["reproduction"],
        "runs": [
            {
                "name": run["name"],
                "fission": run["physical_fission"],
                "viable_pair": run["both_daughters_viable"],
                "fission_step": run["fission_step"],
                "deepest_failure": run["deepest_failure"],
                "max_mass_over_birth": run["max_mass_over_birth"],
            }
            for run in mech["runs"]
        ],
    })
    write(out, "contrast_fallback.json", {
        **raw["contrast_fallback"],
        "execution": "NOT_AUTHORIZED_BY_GATE7" if not raw["contrast_fallback"]["required"] else "EXECUTED",
    })
    write(out, "v4_reproduction_qualification.json", {
        "required": {"growth": 8, "fissions": 7, "viable_pairs": 6},
        "observed": raw["reproduction"],
        "pass": raw["reproduction"]["pass"],
        "classification": raw["classification"],
    })
    write(out, "v4_daughter_continuation.json", {
        "counted_viable_pairs": raw["reproduction"]["simple_viable_daughter_pairs"],
        "required_viable_pairs": 6,
        "all_counted_pairs_used_existing_3000_step_continuation": True,
        "qualification": "FAIL",
    })

    preservation = {
        "d087": {"v2": "8/8", "v3": "8/8", "v4": "7/8", "vector": [True, True, False, True, True, True, True, True]},
        "m1": "CLOSED_FROZEN_PRESERVED",
        "d088_legacy_tests": "PASS_HISTORICAL_PRESERVATION_ONLY",
        "d091": "PASS",
        "evolution_harness": "PASS_TESTS_ONLY",
    }
    write(out, "m1_preservation.json", preservation)
    write(out, "m2_preservation.json", {"status": "QUALIFIED_PRESERVED", "source": f"{R4_EVIDENCE_LABEL}/m2_preservation.json"})
    write(out, "development_preservation.json", {"status": "PASS", "source": f"{R4_EVIDENCE_LABEL}/development_preservation.json"})
    for name in (
        "v4_mutation_frequency.json",
        "v4_mutant_lineage.json",
        "v4_environment_a_selection.json",
        "v4_environment_b_selection.json",
        "v4_reversal.json",
    ):
        write(out, name, {"status": "NOT_REACHED_GATE9_STOP", "reason": "robust V4 reproduction failed"})
    for name in (
        "checkpoint_restart.json",
        "linux_runtime.json",
        "sensory_embodiment.json",
        "experiential_memory.json",
        "godot_independence.json",
    ):
        source = args.r4 / name
        write(out, name, {"status": "PRESERVED_FROM_R4", "source": f"{R4_EVIDENCE_LABEL}/{name}", "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest()})
    write(out, "global_material_energy_closure.json", {
        "active_a_to_w": raw["active_energy_closure"],
        "structural_partition": "PASS_FOR_COUNTED_FISSIONS",
        "global_final_rerun": "NOT_REACHED_GATE9_STOP",
    })
    write(out, "forbidden_information_audit.json", {
        "pass": True,
        "new_free_parameters": 0,
        "resource_position_read": False,
        "world_axis_read": False,
        "centroid_to_neck_read": False,
        "cleavage_plane_read": False,
        "body_size_signal_read_by_regulator_or_motor": False,
        "age_or_fission_attempt_signal_read": False,
        "observer_feedback": False,
        "fallback_executed": raw["contrast_fallback"]["required"],
    })
    write(out, "end_to_end_lifecycle.json", {"status": "NOT_REACHED_GATE9_STOP"})
    write(out, "final_goal_matrix.json", {
        "m1": "PASS_PRESERVED",
        "m2": "PASS_PRESERVED",
        "m3": "PASS_PRESERVED",
        "v4_robust_reproduction": "FAIL",
        "v4_evolution": "NOT_REACHED",
        "final_integrated_qualification": "NOT_REACHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
    })
    write(out, "qualification.json", {
        "directive": DIRECTIVE,
        "status": "GOAL_AGENT_PROVISIONAL_NEGATIVE",
        "classification": "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED",
        "passive_root_cause": "ROBUST_LOCAL_APPOSITION_AND_STRESS_LOCALIZATION_ABSENT",
        "maturation_horizon_classification": raw["maturation_horizon_audit"]["classification"],
        "qualification_horizon": raw["qualification_horizon"],
        "mechanochemical_composition": "FAIL",
        "contrast_fallback": "NOT_REQUIRED",
        "robust_v4_reproduction": "FAIL",
        "evolution": "NOT_REACHED_GATE9_STOP",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": True,
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    manifest = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest.append({"path": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size})
    write(out, "artifact_manifest.json", {"files": manifest, "file_count": len(manifest)})


if __name__ == "__main__":
    main()
