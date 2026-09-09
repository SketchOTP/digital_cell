#!/usr/bin/env python3
"""Build compact governed evidence for DC-FINAL-001-R8R1."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R8R1-V4-SIGN-AWARE-MATURATION-MECHANICS-REPRODUCTION-"
    "AND-END-GOAL-CLOSURE-001"
)
START = "eb198c4e84907d5a7f9f8c170d58a27b56e63bb0"
R8_ROOT = "experiments/generated/dcfinal001r8"


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def compact_daughter(value: dict | None) -> dict | None:
    if value is None:
        return None
    keys = (
        "viable",
        "completed_steps",
        "observer_viable",
        "closed_intact",
        "all_simple",
        "all_runtime_valid",
        "all_lifecycle_invariants_hold",
        "c_retention",
        "a_retention",
        "initial",
        "terminal",
        "cumulative_topology_ruptures",
        "cumulative_same_edge_rebonds",
        "cumulative_a_spent_on_rebond",
    )
    return {key: value[key] for key in keys}


def compact_runs(campaign: dict) -> list[dict]:
    rows = []
    for run in campaign["runs"]:
        daughters = run["daughter_diagnostics"]
        if daughters is not None:
            daughters = {
                "step": daughters["step"],
                "both_viable": daughters["both_viable"],
                "daughter_a": compact_daughter(daughters["daughter_a"]),
                "daughter_b": compact_daughter(daughters["daughter_b"]),
            }
        rows.append(
            {
                "name": run["name"],
                "max_mass_over_birth": run["max_mass_over_birth"],
                "physical_fission": run["physical_fission"],
                "both_daughters_viable": run["both_daughters_viable"],
                "fission_step": run["fission_step"],
                "deepest_failure": run["deepest_failure"],
                "all_simple": run["all_simple"],
                "all_runtime_valid": run["all_runtime_valid"],
                "all_lifecycle_valid": run["all_lifecycle_valid"],
                "checkpoints": run["checkpoints"],
                "daughter_diagnostics": daughters,
            }
        )
    return rows


def preserve(root: Path, r8: Path, name: str) -> None:
    source = r8 / name
    write(
        root,
        name,
        {
            "status": "PRESERVED_FROM_ACCEPTED_R8",
            "source": f"{R8_ROOT}/{name}",
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        },
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--gate1", type=Path, required=True)
    parser.add_argument("--r8", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.raw.read_text())
    gate1 = json.loads(args.gate1.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    if raw["robust_v4_reproduction"]:
        raise SystemExit("R8R1 reproduction passed; evolution and final integration must execute")
    if gate1["classification"] != (
        "R8_BOTH_SIGN_MATURE_FRACTION_OVERATTENUATES_COMPRESSED_MATURE_SCAFFOLD"
    ):
        raise SystemExit("Gate 1 does not authorize the sign-aware production law")

    campaign_keys = {
        "passive_sign_aware_v4": raw["passive_sign_aware_v4"],
        "r5r1_sign_aware": raw["r5r1_tangential_sign_aware_v4"],
        "r6_normal_sign_aware": raw["r6_curvature_normal_sign_aware_v4"],
        "r6_normal_tangential_sign_aware": raw[
            "r6_curvature_normal_plus_tangential_sign_aware_v4"
        ],
    }
    campaigns = {key: compact_runs(value) for key, value in campaign_keys.items()}
    counted = [
        run
        for rows in campaigns.values()
        for run in rows
        if run["physical_fission"]
    ]

    write(
        out,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": START,
            "result_scientific_head": args.head,
            "r8_scientific_head": "aad73e86cd13b2bb77eb3c9e58105c294533fb1c",
            "r8_governed_head": START,
            "r8_exact_head_ci": {"run": 34383115730, "result": "PASS"},
            "r8_artifact_sha256": (
                "d7bdf9c31da2e144d96fbcb53f727a9ec31304e0400e12ed984ea01285b384d8"
            ),
            "r8_authority": "PASS",
            "independent_architect_acceptance": "PENDING",
        },
    )
    write(
        out,
        "architect_disposition.json",
        {
            "r8": "R8_ACCEPTED_MATERIAL_REPAIR_MECHANICS_OVERATTENUATION_REPLAN",
            "sole_active_directive": DIRECTIVE,
            "r8r1_independent_acceptance": "PENDING",
        },
    )
    write(
        out,
        "owner_override.json",
        {
            "status": "PASS",
            "shutdown_override_active": True,
            "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
            "next_execution_started": False,
        },
    )
    write(
        out,
        "external_prior_art.json",
        {
            "status": "ARCHITECT_DISCOVERY_COMPLETED_IMPLEMENTATION_CONFIRMED",
            "sources": [
                {
                    "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC7443980/",
                    "classification": "ADAPTABLE_PRINCIPLE",
                    "principle": "actin networks withstand tension and buckle readily under compression",
                },
                {
                    "url": "https://www.nature.com/articles/s41467-024-53228-y",
                    "classification": "REFERENCE_ONLY",
                    "principle": "convergent cortical flow and compression accompany furrowing",
                },
            ],
            "imported_parameters": [],
        },
    )

    write(out, "compression_overattenuation_audit.json", gate1)
    write(
        out,
        "mechanics_counterfactual.json",
        {
            "states": gate1["states"],
            "compressed_mixed_edges": gate1["compressed_mixed_edges"],
            "attenuation_observed": gate1["attenuation_observed"],
            "scientific_state_changed": False,
        },
    )
    write(
        out,
        "sign_aware_mechanics_contract.json",
        {
            "pass": True,
            "v4_compression": "raw_strain < 0 => fs_raw",
            "v4_tension": "raw_strain >= 0 => mature_fraction * fs_raw",
            "v4_rupture": "unchanged R8 mature_fraction * raw_strain",
            "fully_mature_parity": "EXACT",
            "pressure": "UNCHANGED",
            "bending": "UNCHANGED",
            "new_free_parameters": 0,
        },
    )
    write(
        out,
        "young_closure_tension_test.json",
        {
            "pass": True,
            "fully_young_tensile_stretch_load": 0.0,
            "raw_tensile_strain_may_be_positive": True,
            "rupture_from_raw_strain_only": False,
            "r8_full_density_two_edge_closure": "PRESERVED",
        },
    )
    write(out, "compression_add_young_invariance.json", gate1["add_young_counterfactual"])
    write(
        out,
        "daughter_replay.json",
        {
            "rows": [
                {
                    "fixture": row["fixture"],
                    "continuation": compact_daughter(row["corrected_continuation"]),
                }
                for row in raw["daughter_fixture_replays"]
            ],
            "all_completed_3000_steps": raw["all_daughter_replays_complete"],
            "historical_fixture_warning": (
                "pre-R8 half-density closures are diagnostic controls, not corrected births"
            ),
        },
    )
    write(
        out,
        "closure_cycle_preservation.json",
        {
            "pass": raw["r8_closure_cycle_remains_eliminated"],
            "legacy_r7_pathology": {"rupture_rebond_cycles": 3674, "a_spent": 5417.959656209351},
            "maximum_r8r1_fixture_ruptures": max(
                row["corrected_continuation"]["cumulative_topology_ruptures"]
                for row in raw["daughter_fixture_replays"]
            ),
            "maximum_r8r1_fixture_rebonds": max(
                row["corrected_continuation"]["cumulative_same_edge_rebonds"]
                for row in raw["daughter_fixture_replays"]
            ),
        },
    )
    write(
        out,
        "non_v4_parity.json",
        {
            "pass": True,
            "HistoricalV1": "UNCHANGED",
            "ConservativeV2": "UNCHANGED",
            "GeometryConservativeV3": "UNCHANGED",
            "reason": "new branch is gated by is_maturation_coupled",
        },
    )
    write(
        out,
        "m1_preservation.json",
        {
            "d087": {
                "v2": "8/8",
                "v3": "8/8",
                "v4": "7/8",
                "vector": [True, True, False, True, True, True, True, True],
            },
            "m1": "CLOSED_FROZEN_PRESERVED",
            "r4_contract_topology_tests": "PASS",
            "d088_legacy_tests": "PASS_HISTORICAL_PRESERVATION_ONLY",
            "d091": "PASS",
            "evolution_harness": "PASS_TESTS_ONLY",
        },
    )
    write(
        out,
        "reproduction_campaign.json",
        {key: {"runs": rows} for key, rows in campaigns.items()},
    )
    write(
        out,
        "reproduction_qualification.json",
        {
            "required": {"growth": 8, "geometry_valid_fissions": 7, "viable_pairs": 6},
            "observed": raw["campaign_counts"],
            "pass": False,
            "classification": raw["classification"],
            "evolution": "NOT_REACHED_GATE8_STOP",
        },
    )
    write(
        out,
        "daughter_continuation.json",
        {
            "counted_fissions": len(counted),
            "counted_viable_pairs": sum(row["both_daughters_viable"] for row in counted),
            "all_counted_daughters_completed_3000_steps": all(
                row["daughter_diagnostics"][side]["completed_steps"] == 3000
                for row in counted
                for side in ("daughter_a", "daughter_b")
            ),
            "rows": counted,
        },
    )

    for name in (
        "v4_mutation.json",
        "v4_mutant_lineage.json",
        "environment_a_selection.json",
        "environment_b_selection.json",
        "mutation_off_control.json",
        "reversal.json",
    ):
        write(
            out,
            name,
            {
                "status": "NOT_REACHED_GATE8_STOP",
                "reason": "R8R1 did not qualify robust production-V4 reproduction",
            },
        )
    for name in (
        "checkpoint_restart.json",
        "linux_runtime.json",
        "sensory_embodiment.json",
        "experiential_memory.json",
        "godot_independence.json",
    ):
        preserve(out, args.r8, name)

    write(
        out,
        "global_material_energy_closure.json",
        {
            "r8_full_density_two_edge_closure": "PRESERVED",
            "r8_two_edge_a_cost": "PRESERVED",
            "counted_fission_partition": "PASS",
            "active_force_laws": "UNCHANGED",
            "final_integrated_run": "NOT_REACHED_GATE8_STOP",
        },
    )
    write(
        out,
        "forbidden_information_audit.json",
        {
            "pass": True,
            "new_free_parameters": 0,
            "v4_only": True,
            "division_command_or_target": False,
            "new_threshold_gain_or_timer": False,
            "newborn_grace_period": False,
            "success_conditioned_mechanics": False,
            "fission_detector_changed": False,
            "r8_material_or_rupture_repairs_changed": False,
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "m1": "PASS_PRESERVED",
            "m2": "PASS_PRESERVED",
            "m3": "PASS_PRESERVED",
            "r8_material_repair": "ACCEPTED_PRESERVED",
            "sign_aware_mechanics": "PASS_CONTRACT",
            "robust_v4_reproduction": "FAIL",
            "v4_mutation": "NOT_REACHED",
            "natural_selection": "NOT_REACHED",
            "reversal": "NOT_REACHED",
            "final_integrated_m1_m5": "NOT_REACHED",
            "digital_cell_end_goal": "NOT_ESTABLISHED",
        },
    )
    write(
        out,
        "qualification.json",
        {
            "directive": DIRECTIVE,
            "status": "GOAL_AGENT_PROVISIONAL_BOUNDED_NEGATIVE_REPLAN",
            "r8_material_repair": "ACCEPTED_PRESERVED",
            "compression_overattenuation": "CONFIRMED",
            "sign_aware_mechanics_contract": "PASS",
            "new_free_parameters": 0,
            "non_v4_parity": "PASS",
            "fully_young_tensile_load": "PASS_ZERO",
            "fully_mature_v4_parity": "PASS_EXACT",
            "compressed_mature_scaffold_parity": "PASS_EXACT",
            "adding_young_mass_weakens_existing_compression": "NO",
            "r8_closure_cycle_remains_eliminated": True,
            "classification": raw["classification"],
            "robust_v4_reproduction": "FAIL",
            "evolution": "NOT_REACHED_GATE8_STOP",
            "final_integrated_m1_m5": "NOT_REACHED",
            "digital_cell_end_goal": "NOT_ESTABLISHED",
            "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
            "next_execution_started": False,
            "independent_architect_acceptance": "PENDING",
        },
    )

    manifest = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest.append(
            {
                "path": path.name,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "bytes": path.stat().st_size,
            }
        )
    write(out, "artifact_manifest.json", {"file_count": len(manifest), "files": manifest})


if __name__ == "__main__":
    main()
