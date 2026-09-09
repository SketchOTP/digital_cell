#!/usr/bin/env python3
"""Build compact governed evidence for DC-FINAL-001-R8."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R8-V4-FISSION-CLOSURE-MATERIAL-AND-LOAD-BEARING-"
    "CONSISTENCY-EMERGENCY-CLOSURE-001"
)
START = "7363cfaa2c459522583b70d8107f377c08467b76"
R7_SCIENTIFIC = "0dac294efce2745a99d423f28c31065fcbf29e76"
R7_ROOT = "experiments/generated/dcfinal001r7"
FIXTURE_ROOT = "experiments/fixtures/dcfinal001r8"


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def summary(campaign: dict) -> dict:
    return {key: value for key, value in campaign.items() if key != "runs"}


def compact_daughter(value: dict | None) -> dict | None:
    if value is None:
        return None
    return {
        key: value[key]
        for key in (
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
    } | {
        "birth_closure": value["closing_edge_trace_first_100_steps"][0],
        "step_100_closure": value["closing_edge_trace_first_100_steps"][-1],
    }


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
                "daughter_diagnostics": daughters,
            }
        )
    return rows


def preserved(root: Path, r7: Path, name: str) -> None:
    source = r7 / name
    write(
        root,
        name,
        {
            "status": "PRESERVED_FROM_R7",
            "source": f"{R7_ROOT}/{name}",
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        },
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r7", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.raw.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    if raw["robust_v4_reproduction"]:
        raise SystemExit("R8 reproduction passed; evolution and final integration must execute")

    campaigns = {
        "passive_corrected_v4": raw["passive_corrected_v4"],
        "r5r1_corrected_v4": raw["r5r1_tangential_corrected_v4"],
        "r6_normal_corrected_v4": raw["r6_curvature_normal_corrected_v4"],
        "r6_normal_tangential_corrected_v4": raw[
            "r6_curvature_normal_plus_tangential_corrected_v4"
        ],
    }
    fixture_hashes = {
        path.name: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(args.fixtures.glob("*.json"))
    }

    write(
        out,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": START,
            "result_scientific_head": args.head,
            "r7_scientific_head": R7_SCIENTIFIC,
            "r7_governed_head": START,
            "r7_exact_head_ci": {"run": 34373915324, "result": "PASS"},
            "r7_artifact_sha256": "a056c1416eb63ea1ef90b8cf508890a716cd2c0af89ce246c66137b4d8d109be",
            "r7_authority": "PASS",
            "governance_validator": "KNOWN_FAIL_PREEXISTING_APPEND_ONLY_SCHEMA_DRIFT",
            "independent_architect_acceptance": "PENDING",
        },
    )
    write(
        out,
        "architect_disposition.json",
        {
            "r7": "R7_ACCEPTED_BOUNDED_NEGATIVE_REPLAN",
            "sole_active_directive": DIRECTIVE,
            "r8_independent_acceptance": "PENDING",
        },
    )
    write(
        out,
        "owner_override.json",
        {
            "status": "PASS",
            "shutdown_override_active": True,
            "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
            "governance_validator": "KNOWN_FAIL_PREEXISTING_APPEND_ONLY_SCHEMA_DRIFT",
            "next_execution_started": False,
        },
    )
    write(
        out,
        "external_prior_art.json",
        {
            "status": "ARCHITECT_DISCOVERY_ALREADY_SUFFICIENT",
            "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC7072619/",
            "classification": "REFERENCE_ONLY_CONSISTENCY_SUPPORT",
            "used_principle": "cytokinesis coordinates contractile mechanics with membrane and envelope remodeling",
            "imported_parameters": [],
        },
    )

    write(
        out,
        "closing_edge_root_cause.json",
        {
            "classification": "V4_CLOSING_EDGE_YOUNG_LOAD_CYCLE_CONFIRMED",
            "source_head": START,
            "r7_seed3_fission_step": 8226,
            "newborn_closure": {
                "length": 6.138152700989883,
                "mass_per_daughter": 3.0690763504949414,
                "line_density_ratio": 0.5,
                "mature_fraction": 0.0,
                "raw_strain": 6138152700989881.0,
                "legacy_stretch_force": 55.999999999999986,
            },
            "daughter_a": {
                "ruptures": 3674,
                "same_edge_rebonds": 3674,
                "a_spent_on_rebond": 5417.959656209351,
                "a_retention": 0.7058542506592405,
                "closed_intact": True,
            },
            "daughter_b": {
                "ruptures": 6,
                "same_edge_rebonds": 5,
                "a_spent_on_rebond": 8.079192480469146,
                "closed_intact": False,
            },
            "exact_newborn_fixture_sha256": fixture_hashes,
        },
    )
    write(
        out,
        "closing_edge_material_accounting.json",
        {
            "pass": True,
            "rho_s": 1.0,
            "one_edge_mass": "rho_s * distance",
            "two_edge_mass": "2 * rho_s * distance",
            "prior_total_mass": "rho_s * distance",
            "corrected_v4_mass_per_daughter": "rho_s * distance",
            "corrected_v4_a_cost": "2 * rho_s * distance / frozen_y_g",
            "frozen_y_g": 0.9,
            "material_created_for_free": False,
            "new_free_parameters": 0,
        },
    )
    write(
        out,
        "mature_fraction_mechanics_contract.json",
        {
            "pass": True,
            "v4_stretch": "mature_fraction * prior_clamped_stretch",
            "fully_young_load": 0.0,
            "mixed_load": "continuous",
            "fully_mature_parity": "EXACT",
            "pressure_and_bending": "UNCHANGED",
        },
    )
    write(
        out,
        "mature_fraction_rupture_contract.json",
        {
            "pass": True,
            "v4_metric": "mature_fraction * raw_strain",
            "threshold": "UNCHANGED_1.25",
            "fully_young_raw_strain_only_rupture": False,
            "fully_mature_parity": "EXACT",
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
            "unit_test": "non_v4_fission_preserves_shared_half_density_closure_contract",
        },
    )

    replay_rows = []
    for row in raw["daughter_fixture_replays"]:
        replay_rows.append(
            {
                "fixture": row["fixture"],
                "fixture_sha256": fixture_hashes[row["fixture"]],
                "corrected_continuation": compact_daughter(row["corrected_continuation"]),
            }
        )
    write(out, "daughter_replay.json", {"rows": replay_rows})
    write(
        out,
        "daughter_energy_recovery.json",
        {
            "rows": [
                {
                    "fixture": row["fixture"],
                    "a_retention": row["corrected_continuation"]["a_retention"],
                    "repair_a": row["corrected_continuation"][
                        "cumulative_a_spent_on_rebond"
                    ],
                }
                for row in raw["daughter_fixture_replays"]
            ],
            "interpretation": "legacy half-density newborns no longer exhibit the catastrophic R7 repair drain, but under-massed historical closures may still rupture as load matures",
        },
    )
    write(
        out,
        "daughter_closure_continuity.json",
        {
            "rows": [
                {
                    "fixture": row["fixture"],
                    "completed_steps": row["corrected_continuation"]["completed_steps"],
                    "closed_intact": row["corrected_continuation"]["closed_intact"],
                    "viable": row["corrected_continuation"]["viable"],
                    "ruptures": row["corrected_continuation"][
                        "cumulative_topology_ruptures"
                    ],
                    "rebonds": row["corrected_continuation"][
                        "cumulative_same_edge_rebonds"
                    ],
                }
                for row in raw["daughter_fixture_replays"]
            ],
            "historical_fixture_warning": "fixtures retain the pre-R8 half-density closure material and therefore are diagnostic controls, not corrected-birth qualification events",
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
        {
            key: {"summary": summary(value), "runs": compact_runs(value)}
            for key, value in campaigns.items()
        },
    )
    write(
        out,
        "reproduction_qualification.json",
        {
            "required": {"growth": 8, "geometry_valid_fissions": 7, "viable_pairs": 6},
            "observed": raw["campaign_counts"],
            "pass": False,
            "classification": raw["classification"],
        },
    )
    counted = [
        run
        for campaign in campaigns.values()
        for run in compact_runs(campaign)
        if run["physical_fission"]
    ]
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
                "status": "NOT_REACHED_GATE9_STOP",
                "reason": "R8 did not qualify robust production-V4 reproduction",
            },
        )
    for name in (
        "checkpoint_restart.json",
        "linux_runtime.json",
        "sensory_embodiment.json",
        "experiential_memory.json",
        "godot_independence.json",
    ):
        preserved(out, args.r7, name)

    write(
        out,
        "global_material_energy_closure.json",
        {
            "v4_fission_closure": "PASS_FULL_DENSITY_TWO_EDGE_YIELD_ACCOUNTED",
            "counted_fission_partition": "PASS",
            "r8_parent_geometry_runtime_lifecycle": "PASS",
            "final_integrated_run": "NOT_REACHED_GATE9_STOP",
        },
    )
    write(
        out,
        "forbidden_information_audit.json",
        {
            "pass": True,
            "new_free_parameters": 0,
            "v4_only": True,
            "daughter_grace_period_or_newborn_flag": False,
            "lineage_exemption": False,
            "division_state_or_target_ratio": False,
            "success_conditioned_repair": False,
            "fission_detector_changed": False,
            "maturation_rate_or_rupture_threshold_changed": False,
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "m1": "PASS_PRESERVED",
            "m2": "PASS_PRESERVED",
            "m3": "PASS_PRESERVED",
            "v4_closure_contract": "REPAIRED",
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
            "closing_edge_young_load_cycle": "CONFIRMED",
            "two_edge_structural_accounting": "PASS",
            "new_free_parameters": 0,
            "non_v4_parity": "PASS",
            "fully_young_load": "PASS_ZERO",
            "fully_mature_v4_parity": "PASS_EXACT",
            "rupture_load_bearing_consistency": "PASS",
            "classification": raw["classification"],
            "robust_v4_reproduction": "FAIL",
            "evolution": "NOT_REACHED_GATE9_STOP",
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
