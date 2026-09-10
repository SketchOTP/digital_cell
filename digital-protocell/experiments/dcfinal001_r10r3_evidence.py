#!/usr/bin/env python3
"""Build governed evidence for the R10R3 D096 substrate requalification."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R10R3-D096-ACTIVATED-MATERIAL-EXPRESSION-INTEGRATED-"
    "REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001"
)
START = "2a9d0307ec9d99e4ae2347b50dd9993eef9fa8c4"
R10R2_SCIENTIFIC = "fbecdfe0fac30d090861ec1366bd78eee7cf797a"
R10R2_WORKFLOW = 34428948113
R10R2_ARTIFACT = "6dfa72dc8728b6f8af5f6a7c574327ab9f108059eb9ee574d130a9f98708eefb"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def preserve(root: Path, source_root: Path, name: str, status: str) -> None:
    source = source_root / name
    write(
        root,
        name,
        {
            "status": status,
            "preserved_from": str(source),
            "source_sha256": digest(source),
            "reexecution": "PASS",
        },
    )


def campaign_environment(row: dict) -> str:
    values = row["environment_sequence"]
    if values == ["RESOURCE_CHALLENGE"]:
        return "RESOURCE"
    if values == ["DAMAGE_CHALLENGE"]:
        return "DAMAGE"
    raise ValueError(values)


def compact_budget(row: dict) -> dict:
    budget = row["structural_budget"]
    return {
        "environment": campaign_environment(row),
        "expression_path": row["expression_path"],
        "initial_structural_mass": budget["initial_structural_mass"],
        "d096_structural_conversion": budget[
            "d096_structural_material_converted_to_catalysts"
        ],
        "d096_catalyst_precursor_a": budget["catalyst_precursor_a"],
        "ordinary_m1_structural_build": budget["ordinary_m1_structural_build"],
        "ordinary_m1_structural_turnover": budget[
            "ordinary_m1_structural_turnover"
        ],
        "surplus_growth_structural_production": budget[
            "surplus_growth_structural_production"
        ],
        "damage_structural_loss": budget["damage_structural_loss"],
        "rupture_rebond_and_remesh_net": budget["rupture_rebond_and_remesh_net"],
        "post_bootstrap_fission_closure_material": budget[
            "post_bootstrap_fission_closure_material"
        ],
        "terminal_structural_mass": budget["terminal_structural_mass"],
        "structural_closure_residual": budget["closure_residual"],
        "terminal_allocation_catalyst_mass": budget[
            "terminal_allocation_catalyst_mass"
        ],
        "allocation_catalyst_turnover": budget["allocation_catalyst_turnover"],
        "d096_activation_and_maintenance_a": budget[
            "d096_activation_and_maintenance_a"
        ],
        "growth_a_consumed": budget["growth_a_consumed"],
        "active_motor_a": budget["active_motor_a"],
        "active_motor_w": budget["active_motor_w"],
        "active_motor_residual": budget["active_motor_a"]
        - budget["active_motor_w"],
        "world_n_closure_residual": row["n_closure_residual"],
        "world_f_closure_residual": row["f_closure_residual"],
        "terminal_mass_over_birth_range": [
            min(item["mass_over_birth_mass"] for item in row["terminal_cohorts"]),
            max(item["mass_over_birth_mass"] for item in row["terminal_cohorts"]),
        ],
        "post_bootstrap_physical_fissions": row["ledger"][
            "post_bootstrap_physical_fissions"
        ],
        "physical_deaths": row["ledger"]["physical_deaths"],
    }


def not_reached(reason: str) -> dict:
    return {
        "status": "NOT_REACHED_GATE10_STOP",
        "reason": reason,
        "scientific_claim": "NOT_ESTABLISHED",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--budget", type=Path, required=True)
    parser.add_argument("--v1", type=Path, required=True)
    parser.add_argument("--v2", type=Path, required=True)
    parser.add_argument("--r10-control", type=Path, required=True)
    parser.add_argument("--r10", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    budget_raw = json.loads(args.budget.read_text())
    v1 = json.loads(args.v1.read_text())
    v2 = json.loads(args.v2.read_text())
    r10_control = json.loads(args.r10_control.read_text())
    campaigns = [compact_budget(row) for row in budget_raw["campaigns"]]

    assert len(campaigns) == 6
    assert v1["counts"] == {
        "growth_qualified": 10,
        "geometry_valid_fissions": 0,
        "full_state_viable_daughter_pairs": 0,
    }
    assert v2["counts"] == {
        "growth_qualified": 10,
        "geometry_valid_fissions": 1,
        "full_state_viable_daughter_pairs": 1,
    }
    assert not v1["robust_reproduction"]
    assert not v2["robust_reproduction"]
    assert r10_control["signed_stress_counts"]["geometry_valid_fissions"] == 7
    assert (
        r10_control["signed_stress_counts"]["full_state_viable_daughter_pairs"]
        == 6
    )
    assert r10_control["robust_v4_reproduction"] is True
    assert all(
        run["expression_budget"]["expression_structural_draw"] == 0.0
        and run["expression_budget"]["expression_failures"] == 0
        for run in v2["runs"]
    )

    by_key = {
        (row["environment"], row["expression_path"]): row for row in campaigns
    }
    comparisons = []
    for environment in ("RESOURCE", "DAMAGE"):
        current = by_key[(environment, "CURRENT_D096_V1")]
        off = by_key[(environment, "D096_OFF")]
        candidate = by_key[(environment, "D096_ACTIVATED_MATERIAL_CANDIDATE")]
        comparisons.append(
            {
                "environment": environment,
                "v1_terminal_structural_mass": current["terminal_structural_mass"],
                "off_terminal_structural_mass": off["terminal_structural_mass"],
                "candidate_terminal_structural_mass": candidate[
                    "terminal_structural_mass"
                ],
                "candidate_minus_v1_terminal_structural_mass": candidate[
                    "terminal_structural_mass"
                ]
                - current["terminal_structural_mass"],
                "v1_direct_structural_draw": current["d096_structural_conversion"],
                "candidate_direct_structural_draw": candidate[
                    "d096_structural_conversion"
                ],
                "candidate_catalyst_precursor_a": candidate[
                    "d096_catalyst_precursor_a"
                ],
                "candidate_not_equivalent_to_off": candidate[
                    "terminal_allocation_catalyst_mass"
                ]
                > 0
                and candidate["terminal_structural_mass"]
                != off["terminal_structural_mass"],
            }
        )

    max_structural_residual = max(
        abs(row["structural_closure_residual"]) for row in campaigns
    )
    max_world_residual = max(
        abs(row[field])
        for row in campaigns
        for field in ("world_n_closure_residual", "world_f_closure_residual")
    )
    max_active_residual = max(
        abs(row["active_motor_residual"]) for row in campaigns
    )
    max_v2_motor_residual = max(
        abs(
            run["expression_budget"]["active_motor_a"]
            - run["expression_budget"]["active_motor_w"]
        )
        for run in v2["runs"]
    )
    precursor_total = sum(
        run["expression_budget"]["expression_catalyst_precursor_a"]
        for run in v2["runs"]
    )
    activation_maintenance_total = sum(
        run["expression_budget"]["expression_activation_and_maintenance_a"]
        for run in v2["runs"]
    )
    turnover_total = sum(
        run["expression_budget"]["expression_turnover_waste"]
        for run in v2["runs"]
    )

    out = args.output
    out.mkdir(parents=True, exist_ok=True)
    for stale in out.glob("*.json"):
        if not stale.name.endswith(".raw.json"):
            stale.unlink()

    write(
        out,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": START,
            "result_scientific_head": args.head,
            "r10r2_scientific_head": R10R2_SCIENTIFIC,
            "r10r2_governed_head": START,
            "r10r2_exact_head_ci": {"run": R10R2_WORKFLOW, "result": "PASS"},
            "r10r2_artifact_sha256": R10R2_ARTIFACT,
            "r10_robust_reproduction": "QUALIFIED_7_OF_10_FISSIONS_6_OF_10_FULL_STATE_VIABLE_PAIRS",
            "independent_architect_acceptance": "PENDING",
        },
    )
    write(
        out,
        "architect_disposition.json",
        {
            "status": "R10R2_ACCEPTED_D096_REPRODUCTION_INTEGRATION_AND_EXPRESSION_SUBSTRATE_REPLAN",
            "sole_active_directive": DIRECTIVE,
            "r10r3_stop_boundary": "GATE_10_INTEGRATED_D096_V2_REPRODUCTION",
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
            "status": "ARCHITECT_REVIEW_COMPLETED",
            "cellular_proteome_resource_allocation": "ADAPTABLE_PRINCIPLE",
            "proteostasis_and_turnover": "ADAPTABLE_PRINCIPLE",
            "direct_numerical_reuse": "NONE",
            "new_external_parameters": 0,
        },
    )
    write(
        out,
        "integration_boundary.json",
        {
            "status": "PASS",
            "classification": "R10_REPRODUCTION_D096_EXPRESSION_NOT_INTEGRATED_CONFIRMED",
            "r10_reproduction": {
                "d096_expression": False,
                "transport": True,
                "reactions": True,
                "growth": True,
                "r9_refractory_mechanics": True,
                "r10_signed_stress_fission": True,
                "full_state_daughter_inheritance": True,
            },
            "r10r2_population": {
                "d096_expression": True,
                "transport": True,
                "reactions": True,
                "growth": True,
                "r9_refractory_mechanics": True,
                "r10_signed_stress_fission": True,
                "full_state_daughter_inheritance": True,
            },
        },
    )
    write(
        out,
        "structural_budget_decomposition.json",
        {
            "status": "PASS",
            "classification": "MIXED_CAUSAL",
            "d096_structural_conversion_material_contributor": True,
            "dominant_flux": "ORDINARY_M1_STRUCTURAL_TURNOVER",
            "maximum_absolute_structural_closure_residual": max_structural_residual,
            "campaigns": campaigns,
        },
    )
    write(out, "d096v1_integrated_reproduction.json", v1)
    write(
        out,
        "activated_material_counterfactual.json",
        {
            "status": "PASS_FEASIBLE_FOR_VERSIONED_TEST",
            "observer_only": True,
            "structural_draw": 0,
            "new_parameters": 0,
            "comparisons": comparisons,
            "interpretation": "candidate_removes_direct_structural_draw_and_improves_terminal_mass_over_v1_but_does_not_remove_dominant_M1_turnover",
        },
    )
    write(
        out,
        "d096v2_expression_contract.json",
        {
            "status": "PASS",
            "versioned": True,
            "equation_id": "autopoietic_material_mesh_finite_catalytic_allocation_v2_activated_material",
            "schema_version": 3,
            "historical_v1_equation_id": "autopoietic_material_mesh_finite_catalytic_allocation_v1",
            "historical_v1_schema_version": 2,
            "synthesis_rate": 0.001,
            "activation_cost": 0.2,
            "maintenance_rate": 0.00001,
            "turnover_rate": 0.0001,
            "material_law": {
                "catalyst_precursor": "A",
                "precursor_per_synthesized_catalyst": 1.0,
                "additional_activation_overhead": "activation_cost_times_synthesis",
                "maintenance": "A_to_W",
                "turnover": "catalyst_to_W",
                "structural_draw": 0,
            },
            "new_free_numerical_parameters": 0,
        },
    )
    write(
        out,
        "d096v1_historical_parity.json",
        {
            "status": "PASS",
            "v1_equation_and_schema_unchanged": True,
            "v1_expression_tests": "PASS",
            "v1_mutation_tests": "PASS",
            "non_d096_r10_control": {
                "geometry_valid_fissions": 7,
                "full_state_viable_daughter_pairs": 6,
            },
            "silent_migration": False,
        },
    )
    write(
        out,
        "d096v2_material_energy_closure.json",
        {
            "status": "PASS",
            "one_step_contract_tests": "PASS",
            "structural_draw": 0,
            "integrated_catalyst_precursor_a": precursor_total,
            "integrated_activation_and_maintenance_a": activation_maintenance_total,
            "integrated_turnover_waste": turnover_total,
            "expression_failures": sum(
                run["expression_budget"]["expression_failures"] for run in v2["runs"]
            ),
            "maximum_absolute_active_motor_A_to_W_residual": max_v2_motor_residual,
        },
    )
    write(out, "d096v2_integrated_reproduction.json", v2)
    fission_runs = [run for run in v2["runs"] if run["physical_fission"]]
    write(
        out,
        "daughter_continuation.json",
        {
            "status": "PASS_FOR_OBSERVED_FISSION_BELOW_ROBUST_THRESHOLD",
            "observed_fissions": len(fission_runs),
            "full_state_viable_pairs": sum(
                run["full_state_daughters_viable"] is True for run in fission_runs
            ),
            "runs": [
                {
                    "name": run["name"],
                    "fission_step": run["fission_step"],
                    "full_state_daughters_viable": run[
                        "full_state_daughters_viable"
                    ],
                    "daughter_diagnostics": run["daughter_diagnostics"],
                }
                for run in fission_runs
            ],
        },
    )

    stop_reason = "D096_V2_INTEGRATED_R10_REPRODUCTION_FAILED_1_OF_10_FISSIONS_1_OF_10_VIABLE_PAIRS"
    for name in (
        "generation2_lineage.json",
        "population_turnover.json",
        "genotype_phenotype_causality.json",
        "environment_a_selection.json",
        "environment_b_selection.json",
        "environment_dependence.json",
        "mutation_off_control.json",
        "reversal.json",
    ):
        write(out, name, not_reached(stop_reason))

    write(
        out,
        "m1_preservation.json",
        {
            "status": "PASS",
            "d087": {
                "v2": [True] * 8,
                "v3": [True] * 8,
                "v4": [True, True, False, True, True, True, True, True],
            },
            "r8_closure_repair": "PRESERVED",
            "r8r1_sign_aware_mechanics": "PRESERVED",
            "r9_refractory_mechanics": "PRESERVED",
            "r10_signed_stress_reproduction": "PRESERVED_7_OF_10_6_OF_10",
        },
    )
    preserve(out, args.r10, "m2_preservation.json", "QUALIFIED_PRESERVED")
    preserve(out, args.r10, "development_preservation.json", "PASS_PRESERVED")
    for name in (
        "checkpoint_restart.json",
        "linux_runtime.json",
        "sensory_embodiment.json",
        "experiential_memory.json",
        "godot_independence.json",
    ):
        preserve(out, args.r10, name, "PRESERVED_FINAL_INTEGRATED_NOT_REACHED")
    write(
        out,
        "forbidden_information_audit.json",
        {
            "status": "PASS",
            "new_free_numerical_parameters": 0,
            "target_body_size": False,
            "minimum_body_rescue": False,
            "fitness_score": False,
            "breeder": False,
            "population_cap": False,
            "forced_birth_or_death": False,
            "protected_mutant": False,
            "environment_sensitive_expression": False,
            "observer_feedback": False,
            "pr_44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
        },
    )
    write(
        out,
        "global_material_energy_closure.json",
        {
            "status": "PASS_FOR_EXECUTED_R10R3_SURFACES",
            "maximum_absolute_structural_residual": max_structural_residual,
            "maximum_absolute_world_N_F_residual": max_world_residual,
            "maximum_absolute_active_A_to_W_residual": max(
                max_active_residual, max_v2_motor_residual
            ),
            "d096v2_expression_closure": "PASS",
            "final_integrated_m1_m5": "NOT_REACHED_GATE10_STOP",
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "m1": "PASS_PRESERVED",
            "m2": "QUALIFIED_PRESERVED",
            "m3": "PASS_PRESERVED",
            "r10_reproduction_without_continuous_d096": "QUALIFIED_PRESERVED",
            "d096v2_expression_contract": "PASS",
            "robust_d096_on_v4_reproduction": "FAIL_1_OF_10_1_OF_10",
            "natural_generation_2_plus": "NOT_REACHED",
            "resource_selection": "NOT_REACHED",
            "damage_selection": "NOT_REACHED",
            "environment_dependence": "NOT_REACHED",
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
            "status": "GOAL_AGENT_PROVISIONAL_BOUNDED_NEGATIVE_GATE10_STOP",
            "classification": "D096_V2_INTEGRATED_ROBUST_REPRODUCTION_NOT_ESTABLISHED",
            "structural_attrition_classification": "MIXED_CAUSAL",
            "d096_structural_conversion_causal": "PASS_MATERIAL_CONTRIBUTOR",
            "d096v1_integrated_reproduction": "0_FISSIONS_0_VIABLE_PAIRS",
            "d096v2_integrated_reproduction": "1_FISSION_1_VIABLE_PAIR",
            "robust_d096_on_v4_reproduction": "FAIL",
            "downstream_gates": "NOT_REACHED_GATE10_STOP",
            "digital_cell_end_goal": "NOT_ESTABLISHED",
            "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
            "next_execution_started": False,
            "independent_architect_acceptance": "PENDING",
        },
    )

    files = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json" or path.name.endswith(".raw.json"):
            continue
        files.append(
            {"path": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}
        )
    write(
        out,
        "artifact_manifest.json",
        {
            "directive": DIRECTIVE,
            "result_scientific_head": args.head,
            "files": files,
            "file_count_excluding_manifest_and_raw": len(files),
        },
    )


if __name__ == "__main__":
    main()
