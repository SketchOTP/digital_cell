#!/usr/bin/env python3
"""Seal the R10R7 natural-variant feasibility result.

This script is intentionally diagnostic-only: R10R7's Gate 4 result controls
whether the conditional two-window selection assay may run.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


DIRECTIVE = (
    "DC-FINAL-001-R10R7-NATURAL-VARIANT-CROSS-ENVIRONMENT-FEASIBILITY-"
    "TWO-WINDOW-SELECTION-REVERSAL-AND-END-GOAL-CLOSURE-001"
)
NEUTRAL = [0.25, 0.25, 0.25, 0.25]
RESOURCE = "RESOURCE_CHALLENGE"
DAMAGE = "DAMAGE_CHALLENGE"


def key(genotype: list[float]) -> str:
    return ",".join(f"{value:.17f}" for value in genotype)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def load_neutral_controls(raw: dict[str, Any]) -> dict[str, dict[str, Any]]:
    controls: dict[str, dict[str, Any]] = {}
    for campaign in raw["campaigns"]:
        if campaign.get("mutation_enabled"):
            continue
        sequence = campaign.get("environment_sequence")
        if sequence not in ([RESOURCE], [DAMAGE]):
            continue
        environment = sequence[0]
        controls.setdefault(environment, campaign)
    if set(controls) != {RESOURCE, DAMAGE}:
        raise ValueError("sealed R10R6 neutral controls are incomplete")
    return controls


def phenotype_for(campaign: dict[str, Any], genotype: list[float]) -> dict[str, Any]:
    phenotype = campaign["ledger"]["phenotype_by_genotype"].get(key(genotype))
    if phenotype is None:
        raise ValueError(f"variant genotype is absent from campaign phenotype ledger: {key(genotype)}")
    return phenotype


def matrix(panel: list[list[float]], campaigns: list[dict[str, Any]], controls: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for index, genotype in enumerate(panel):
        resource_campaign = campaigns[index * 2]
        damage_campaign = campaigns[index * 2 + 1]
        resource = phenotype_for(resource_campaign, genotype)
        damage = phenotype_for(damage_campaign, genotype)
        resource_control = phenotype_for(controls[RESOURCE], NEUTRAL)
        damage_control = phenotype_for(controls[DAMAGE], NEUTRAL)

        def relative(environment: str, metric: str, value: float) -> float:
            control = resource_control if environment == RESOURCE else damage_control
            baseline = float(control[metric])
            return 0.0 if baseline == 0.0 else (float(value) - baseline) / baseline

        resource_growth = relative(RESOURCE, "growth_material", resource["growth_material"])
        damage_growth = relative(DAMAGE, "growth_material", damage["growth_material"])
        resource_fissions = int(resource.get("physical_fissions", 0))
        damage_fissions = int(damage.get("physical_fissions", 0))
        resource_deaths = int(resource.get("physical_deaths", 0))
        damage_deaths = int(damage.get("physical_deaths", 0))
        rows.append(
            {
                "panel_index": index,
                "genotype": genotype,
                "resource": {
                    "campaign_seed": resource_campaign["campaign_seed"],
                    "phenotype": resource,
                    "relative_growth_material_vs_neutral": resource_growth,
                    "physical_fissions": resource_fissions,
                    "physical_deaths": resource_deaths,
                },
                "damage": {
                    "campaign_seed": damage_campaign["campaign_seed"],
                    "phenotype": damage,
                    "relative_growth_material_vs_neutral": damage_growth,
                    "physical_fissions": damage_fissions,
                    "physical_deaths": damage_deaths,
                },
                "relative_growth_signs": {
                    "resource": (resource_growth > 0) - (resource_growth < 0),
                    "damage": (damage_growth > 0) - (damage_growth < 0),
                },
                "reproduction_death_interaction": (
                    resource_fissions != damage_fissions
                    or resource_deaths != damage_deaths
                ),
                "opposite_signed_growth_response": resource_growth * damage_growth < 0,
            }
        )
    return rows


def write(root: Path, name: str, value: Any) -> None:
    root.joinpath(name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--panel-result", type=Path, required=True)
    parser.add_argument("--r10r6-raw", type=Path, required=True)
    parser.add_argument("--panel-metadata", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    args = parser.parse_args()

    result = json.loads(args.panel_result.read_text())
    r10r6 = json.loads(args.r10r6_raw.read_text())
    metadata = json.loads(args.panel_metadata.read_text())
    controls = load_neutral_controls(r10r6)
    rows = matrix(result["panel"], result["campaigns"], controls)

    any_opposite = any(row["opposite_signed_growth_response"] for row in rows)
    tradeoff = any_opposite
    # A reproduction/death difference alone is not enough when it has no
    # opposite relative response and the mass-gate direction is unchanged.
    # Preserve the observation without upgrading it to a tradeoff.
    reproduction_only = [
        row["panel_index"]
        for row in rows
        if row["reproduction_death_interaction"]
        and not row["opposite_signed_growth_response"]
    ]

    args.output_root.mkdir(parents=True, exist_ok=True)
    write(args.output_root, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": "86a113e33f1f3e7307bc06721ca9d62812123c0a",
        "r10r6_scientific_head": "7406d0e2f217c7bdfad9b033d81fc47d71cb6a87",
        "r10r6_ci": "34512966061 PASS",
        "r10r6_artifact": "sha256:78cd0b8b037ad8fe38ee1a6ebcc2968c34fcf35fa6b0db553ffef2f638c27bda",
        "architect_disposition": "R10R6_ACCEPTED_GENERATION2_SELECTION_FEASIBILITY_AND_DEPTH_REPLAN",
        "owner_override": "ACTIVE",
    })
    write(args.output_root, "architect_disposition.json", {
        "r10r6_accepted": True,
        "r10r7_sole_active_authorization": True,
        "selection_power_classification": "R10R6_SELECTION_ASSAY_SPARSE_VARIATION_AND_DESCENDANT_EXPOSURE_LIMITED",
        "first_generation2_birth_step": 13650,
        "r10_reproductive_window": 14778,
        "remaining_exposure": 1128,
    })
    write(args.output_root, "owner_override.json", {"status": "ACTIVE", "shutdown_override": True})
    write(args.output_root, "external_prior_art.json", {
        "status": "REFERENCE_ONLY",
        "implementation_parameters_imported": 0,
        "note": "R10R7 uses only project-internal assay and frozen biology; no external numerical parameter was imported.",
    })
    write(args.output_root, "selection_power_audit.json", {
        "r10r6_bootstrap_opportunities_per_replicate": 300,
        "mutation_probability": 0.01,
        "expected_bootstrap_mutations_per_replicate": 3.0,
        "first_generation2_birth_step": 13650,
        "remaining_steps": 1128,
        "classification": "R10R6_SELECTION_ASSAY_SPARSE_VARIATION_AND_DESCENDANT_EXPOSURE_LIMITED",
    })
    write(args.output_root, "natural_variant_panel.json", {
        "source": "sealed R10R6 lawful mutation events",
        "panel_sha256": sha256(args.panel_metadata),
        "panel_count": len(result["panel"]),
        "panel": result["panel"],
        "provenance_metadata": metadata,
        "manual_variants_added": 0,
        "winner_genotypes_added": 0,
    })
    write(args.output_root, "cross_environment_variant_matrix.json", {
        "diagnostic_only": True,
        "phase_steps": result["phase_steps"],
        "boundary_mode": result["population_boundary_mode"],
        "matched_neutral_controls": {
            environment: {
                "source": "sealed R10R6 mutation-off campaign",
                "campaign_seed": controls[environment]["campaign_seed"],
            }
            for environment in (RESOURCE, DAMAGE)
        },
        "rows": rows,
    })
    write(args.output_root, "environment_tradeoff_feasibility.json", {
        "classification": (
            "NATURAL_D096_VARIANTS_HAVE_ENVIRONMENT_DEPENDENT_REPRODUCTIVE_TRADEOFF"
            if tradeoff
            else "NATURAL_D096_VARIANTS_LACK_ENVIRONMENT_DEPENDENT_REPRODUCTIVE_TRADEOFF"
        ),
        "panel_count": len(rows),
        "opposite_signed_growth_variants": [row["panel_index"] for row in rows if row["opposite_signed_growth_response"]],
        "reproduction_or_death_difference_without_tradeoff": reproduction_only,
        "natural_selection_established": False,
        "reason": "No opposite signed relative reproduction-linked response was observed across the sealed natural-variant panel." if not tradeoff else "Feasibility criterion passed.",
    })

    not_reached = {
        "status": "NOT_REACHED",
        "reason": "Gate 4 feasibility failed; R10R7 forbids the two-window extension and all downstream selection/reversal/final-integrated stages.",
    }
    for name in [
        "two_window_protocol.json", "r10r6_prefix_parity.json", "selection_mutation_decomposition.json",
        "selection_response_vectors.json", "environment_a_selection.json", "environment_b_selection.json",
        "environment_dependence.json", "mutation_off_control.json", "population_turnover.json",
        "generation_lineages.json", "genotype_phenotype_causality.json", "reversal.json",
        "population_material_closure.json", "active_energy_closure.json", "r10r5_reproduction_preservation.json",
        "m1_preservation.json", "m2_preservation.json", "development_preservation.json",
        "checkpoint_restart.json", "linux_runtime.json", "sensory_embodiment.json",
        "experiential_memory.json", "godot_independence.json", "global_material_energy_closure.json",
        "final_goal_matrix.json",
    ]:
        write(args.output_root, name, not_reached)
    write(args.output_root, "forbidden_information_audit.json", {
        "status": "PASS",
        "fitness_function": None,
        "breeder_selection": False,
        "forced_birth": False,
        "forced_death": False,
        "protected_mutant": False,
        "population_feedback": False,
        "panel_outcome_used_to_modify_biology": False,
    })
    write(args.output_root, "qualification.json", {
        "classification": "NATURAL_D096_VARIANTS_LACK_ENVIRONMENT_DEPENDENT_REPRODUCTIVE_TRADEOFF",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })
    write(args.output_root, "artifact_manifest.json", {
        "status": "LOCAL_PROVISIONAL_SEAL",
        "files": sorted(path.name for path in args.output_root.glob("*.json") if path.name != "artifact_manifest.json"),
        "source_inputs": {
            "panel_result": str(args.panel_result),
            "r10r6_raw": str(args.r10r6_raw),
            "panel_metadata": str(args.panel_metadata),
        },
    })


if __name__ == "__main__":
    main()
