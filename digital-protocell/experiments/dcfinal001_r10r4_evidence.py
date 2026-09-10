#!/usr/bin/env python3
"""Build governed evidence for the R10R4 intensive D096 gain audit."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R10R4-D096-INTENSIVE-CATALYST-GAIN-INTEGRATED-"
    "REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001"
)
START = "1c7bb0b7da3503663d756b1da14788e60458a8db"
R10R3_SCIENTIFIC = "895d89f35455ea109e9da288105e71736f616bf3"
R10R3_CI = 34469457434
R10R3_ARTIFACT = "8d249bbfd93471ac2187aad08bcd16f5e161c9297fa376d70e8083bd7977a478"
STOP = "NOT_REACHED_GATE10_D096_V3_REPRODUCTION_STOP"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def not_reached(reason: str) -> dict:
    return {
        "status": STOP,
        "reason": reason,
        "scientific_claim": "NOT_ESTABLISHED",
    }


def campaign_by_policy(audit: dict, policy: str) -> dict:
    return next(row for row in audit["campaigns"] if row["gain_policy"] == policy)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--audit", type=Path, required=True)
    parser.add_argument("--v1", type=Path, required=True)
    parser.add_argument("--v2", type=Path, required=True)
    parser.add_argument("--v3", type=Path, required=True)
    parser.add_argument("--r10-control", type=Path, required=True)
    parser.add_argument("--r10r3", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    audit = json.loads(args.audit.read_text())
    v1 = json.loads(args.v1.read_text())
    v2 = json.loads(args.v2.read_text())
    v3 = json.loads(args.v3.read_text())
    r10_control = json.loads(args.r10_control.read_text())
    sealed_v1 = json.loads((args.r10r3 / "d096v1_integrated_reproduction.json").read_text())
    sealed_v2 = json.loads((args.r10r3 / "d096v2_integrated_reproduction.json").read_text())

    assert v1 == sealed_v1
    assert v2 == sealed_v2
    expected_v3 = {
        "growth_qualified": 10,
        "geometry_valid_fissions": 5,
        "full_state_viable_daughter_pairs": 5,
    }
    assert v3["counts"] == expected_v3
    assert not v3["robust_reproduction"]
    assert r10_control["signed_stress_counts"]["geometry_valid_fissions"] == 7
    assert (
        r10_control["signed_stress_counts"]["full_state_viable_daughter_pairs"] == 6
    )
    assert r10_control["robust_v4_reproduction"] is True

    full = campaign_by_policy(audit, "D096_V2_ABSOLUTE_AMOUNT_GAIN")
    cost = campaign_by_policy(audit, "D096_V2_COST_ONLY_GAIN_DISABLED")
    functions_01 = campaign_by_policy(audit, "D096_V2_FUNCTIONS_0_1_ONLY")
    function_2 = campaign_by_policy(audit, "D096_V2_FUNCTION_2_ONLY")
    intensive = campaign_by_policy(
        audit, "D096_V2_COST_WITH_INTENSIVE_GAIN_COUNTERFACTUAL"
    )
    assert full["counts"] == {
        "growth_qualified": 10,
        "geometry_valid_fissions": 1,
        "full_state_viable_daughter_pairs": 1,
    }
    assert cost["counts"] == {
        "growth_qualified": 10,
        "geometry_valid_fissions": 7,
        "full_state_viable_daughter_pairs": 6,
    }
    assert functions_01["counts"]["geometry_valid_fissions"] == 3
    assert function_2["counts"]["geometry_valid_fissions"] == 7
    assert intensive["counts"] == expected_v3

    continuity = []
    for run in v3["runs"]:
        details = run.get("daughter_diagnostics") or {}
        row = details.get("allocation_fission_continuity")
        if row is not None:
            residual = max(abs(value) for value in row["catalyst_material_residuals"])
            concentration_delta = max(
                abs(child[index] - row["parent_concentrations"][index])
                for child in (
                    row["daughter_a_concentrations"],
                    row["daughter_b_concentrations"],
                )
                for index in range(4)
            )
            continuity.append(
                {
                    "name": run["name"],
                    "fission_step": run["fission_step"],
                    "maximum_catalyst_material_residual": residual,
                    "maximum_concentration_delta": concentration_delta,
                    **row,
                }
            )
    assert len(continuity) == 5
    assert max(row["maximum_catalyst_material_residual"] for row in continuity) < 1e-12
    assert max(row["maximum_concentration_delta"] for row in continuity) < 1e-12

    motor_residual = max(
        abs(
            run["expression_budget"]["active_motor_a"]
            - run["expression_budget"]["active_motor_w"]
        )
        for run in v3["runs"]
    )
    assert motor_residual < 1e-8
    assert all(run["expression_budget"]["expression_failures"] == 0 for run in v3["runs"])

    out = args.output
    out.mkdir(parents=True, exist_ok=True)
    for stale in out.glob("*.json"):
        stale.unlink()

    write(
        out,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": START,
            "result_scientific_head": args.head,
            "r10r3_scientific_head": R10R3_SCIENTIFIC,
            "r10r3_governed_head": START,
            "r10r3_exact_head_ci": {"run": R10R3_CI, "result": "PASS"},
            "r10r3_artifact_sha256": R10R3_ARTIFACT,
            "r10_robust_reproduction": "QUALIFIED_7_OF_10_6_OF_10",
            "independent_architect_acceptance": "PENDING",
        },
    )
    write(
        out,
        "architect_disposition.json",
        {
            "status": "R10R3_ACCEPTED_D096_V2_MATERIAL_REPAIR_INTENSIVE_GAIN_REPLAN",
            "sole_active_directive": DIRECTIVE,
            "r10r4_stop_boundary": "GATE_10_D096_V3_INTEGRATED_REPRODUCTION",
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
            "status": "PASS_BOUNDED",
            "classification": "ADAPTABLE_PRINCIPLE_REFERENCE_ONLY",
            "source": {
                "title": "Quantitative proteomic analysis reveals a simple strategy of global resource allocation in bacteria",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC4358657/",
                "relevant_principle": "Effective catalytic capacity is represented as enzyme/proteome fractional abundance, an intensive state.",
            },
            "external_numerical_parameters_imported": 0,
        },
    )
    write(
        out,
        "catalyst_dimensional_audit.json",
        {
            "classification": "D096_CATALYST_IS_EXTENSIVE_PHYSICAL_MATERIAL",
            "state": "AllocationState.catalysts",
            "production": "finite A precursor is converted into catalyst material",
            "turnover": "catalyst material transfers to W",
            "fission": "each amount is multiplied by daughter physical area fraction",
            "partition_residual_is_conserved": True,
            "current_gain_input": "absolute catalyst amount",
            "gain_saturation_constant_explicit_absolute_mass_definition_found": False,
            "scale_counterfactual": audit["scale_counterfactual"],
        },
    )
    write(
        out,
        "fission_gain_continuity.json",
        {
            "status": "PASS",
            "events": continuity,
            "maximum_material_residual": max(
                row["maximum_catalyst_material_residual"] for row in continuity
            ),
            "maximum_concentration_delta": max(
                row["maximum_concentration_delta"] for row in continuity
            ),
            "classification": "CATALYST_CONCENTRATION_FISSION_CONTINUITY_PASS",
        },
    )
    write(
        out,
        "gain_channel_causal_decomposition.json",
        {
            "classification": "D096_GAIN_CAUSALLY_DISTORTS_R10_APPOSITION",
            "all_material_expression_costs_preserved": True,
            "absolute_amount_gain": full,
            "cost_only": cost,
            "functions_0_1_only": functions_01,
            "function_2_only": function_2,
            "function_3": "DORMANT_RESERVE_OFF",
            "causal_result": "Gain removal restores sealed R10 threshold behavior; functions 0/1 are the dominant suppressive channel while function 2 alone retains robust fission.",
        },
    )
    write(
        out,
        "intensive_gain_counterfactual.json",
        {
            "classification": "D096_EXTENSIVE_TO_INTENSIVE_GAIN_MISMATCH_CONFIRMED",
            "observer_only": True,
            "new_parameters": 0,
            "saturation_numeric": 0.1,
            "fission_counterfactual": audit["fission_counterfactual"],
            "campaign": intensive,
            "counterfactual_robust_reproduction": False,
        },
    )
    write(
        out,
        "d096v3_gain_contract.json",
        {
            "status": "PASS",
            "versioned": True,
            "equation": "autopoietic_material_mesh_finite_catalytic_allocation_v3_intensive_gain",
            "schema_version": 4,
            "material_law": "D096_V2_PRESERVED",
            "gain_input": "CATALYST_AMOUNT_DIVIDED_BY_POSITIVE_PHYSICAL_AREA",
            "gain": "1 + c/(0.1+c)",
            "area_floor": None,
            "new_numerical_biological_parameters": 0,
        },
    )
    write(
        out,
        "d096v1_v2_historical_parity.json",
        {
            "status": "PASS",
            "comparison": "PARSED_JSON_SEMANTIC_EXACT",
            "d096v1": {
                "source_sha256": digest(args.r10r3 / "d096v1_integrated_reproduction.json"),
                "replay_sha256": digest(args.v1),
                "semantic_equal": True,
                "counts": v1["counts"],
            },
            "d096v2": {
                "source_sha256": digest(args.r10r3 / "d096v2_integrated_reproduction.json"),
                "replay_sha256": digest(args.v2),
                "semantic_equal": True,
                "counts": v2["counts"],
            },
            "byte_difference_reason": "sealed files use sorted JSON object keys; parsed values are exact",
        },
    )
    write(
        out,
        "d096v3_material_energy_closure.json",
        {
            "status": "PASS",
            "expression_material_law": "UNIT_TEST_EXACT_V2_LEDGER_PARITY",
            "expression_failures": 0,
            "structural_expression_draw": 0,
            "maximum_active_a_to_w_residual": motor_residual,
            "catalyst_material_conservation_at_fission": "PASS",
            "concentration_normalization_changes_material_stock": False,
        },
    )
    write(out, "d096v3_integrated_reproduction.json", v3)
    write(
        out,
        "daughter_continuation.json",
        {
            "status": "PASS_FOR_FIVE_PHYSICAL_FISSIONS",
            "full_state_viable_pairs": 5,
            "events": [
                {
                    "name": run["name"],
                    "step": run["fission_step"],
                    "full_state": run["daughter_diagnostics"]["full_state"],
                }
                for run in v3["runs"]
                if run["physical_fission"]
            ],
        },
    )

    prior_m1 = args.r10r3 / "m1_preservation.json"
    write(
        out,
        "m1_preservation.json",
        {
            "status": "PASS_PRESERVED",
            "source": str(prior_m1),
            "source_sha256": digest(prior_m1),
            "d087": json.loads(prior_m1.read_text())["d087"],
            "scope": "D096-v3 is opt-in; v1/v2 semantic exact replay and D096 contract tests pass",
            "r10_d096_off_control": r10_control["signed_stress_counts"],
        },
    )
    for name, source_name, status in (
        ("m2_preservation.json", "m2_preservation.json", "QUALIFIED_PRESERVED_NOT_REEXECUTED_AFTER_GATE10_STOP"),
        ("development_preservation.json", "development_preservation.json", "PASS_PRESERVED_NOT_REEXECUTED_AFTER_GATE10_STOP"),
    ):
        source = args.r10r3 / source_name
        write(out, name, {"status": status, "source_sha256": digest(source)})

    reason = "D096-v3 reached 5/10 fissions and 5/10 full-state viable pairs, below required 7/10 and 6/10."
    for name in (
        "generation2_lineage.json",
        "population_turnover.json",
        "genotype_phenotype_causality.json",
        "environment_a_selection.json",
        "environment_b_selection.json",
        "environment_dependence.json",
        "mutation_off_control.json",
        "reversal.json",
        "checkpoint_restart.json",
        "linux_runtime.json",
        "sensory_embodiment.json",
        "experiential_memory.json",
        "godot_independence.json",
    ):
        write(out, name, not_reached(reason))

    write(
        out,
        "forbidden_information_audit.json",
        {
            "status": "PASS",
            "new_parameters": 0,
            "biology_change": "VERSIONED_D096_V3_GAIN_INPUT_ONLY",
            "absent": [
                "body_size_controller",
                "target_concentration",
                "expression_shutoff",
                "division_state",
                "growth_limiter",
                "motor_change",
                "fission_change",
                "fitness",
                "breeder",
                "population_cap",
                "forced_birth_or_death",
                "observer_feedback",
            ],
        },
    )
    write(
        out,
        "global_material_energy_closure.json",
        {
            "status": "PASS_EXECUTED_R10R4_SCOPE",
            "final_integrated_m1_m5": STOP,
            "d096v3_expression": "PASS",
            "fission_catalyst_material": "PASS",
            "active_a_to_w": "PASS",
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "M1": "CLOSED_FROZEN_PRESERVED",
            "M2": "QUALIFIED_PRESERVED",
            "M3": "PASS_PRESERVED",
            "R10_reproduction_without_D096": "QUALIFIED",
            "D096_v3_integrated_reproduction": "FAIL_5_OF_10_5_OF_10",
            "M4_evolution": STOP,
            "M5_integrated": STOP,
            "digital_cell_end_goal": "NOT_ESTABLISHED",
        },
    )
    write(
        out,
        "qualification.json",
        {
            "directive": DIRECTIVE,
            "status": "GOAL_AGENT_PROVISIONAL_BOUNDED_NEGATIVE_GATE10_STOP",
            "classification": "D096_V3_INTEGRATED_ROBUST_REPRODUCTION_NOT_ESTABLISHED",
            "dimensional_mismatch": "CONFIRMED",
            "gain_channel_causality": "CONFIRMED",
            "d096v3_integrated_reproduction": "5_FISSIONS_5_FULL_STATE_VIABLE_PAIRS",
            "robust_d096_on_v4_reproduction": "FAIL",
            "downstream_gates": STOP,
            "digital_cell_end_goal": "NOT_ESTABLISHED",
            "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
            "next_execution_started": False,
            "independent_architect_acceptance": "PENDING",
        },
    )

    files = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        files.append(
            {"path": path.name, "sha256": digest(path), "bytes": path.stat().st_size}
        )
    write(
        out,
        "artifact_manifest.json",
        {
            "directive": DIRECTIVE,
            "scientific_head": args.head,
            "files": files,
        },
    )


if __name__ == "__main__":
    main()
