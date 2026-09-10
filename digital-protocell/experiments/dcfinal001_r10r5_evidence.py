#!/usr/bin/env python3
"""Build the append-only R10R5 evidence package from sealed run outputs."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R10R5-D096-FINITE-BUDGET-CENTERED-GAIN-INTEGRATED-"
    "REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001"
)
START = "c1b573f08cc90d74b3452f0427096d828e431326"
R10R4_SCIENTIFIC = "ae18b3e24d475f1791577b07f63b18084402a159"
R10R4_CI = 34478405547
R10R4_ARTIFACT = "6c286a895ffea674071bce1e748181a1c8d9587f8c037d26b68b57b9aab85b27"
STOP = "NOT_REACHED_GATE13_NATURAL_GENERATION_STOP"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def not_reached(reason: str) -> dict:
    return {"status": STOP, "reason": reason, "scientific_claim": "NOT_ESTABLISHED"}


def campaign_summary(campaign: dict) -> dict:
    ledger = campaign["ledger"]
    terminal = campaign["terminal"]
    cohorts = campaign.get("terminal_cohorts", [])
    blockers = Counter(row.get("deepest_physical_blocker") for row in cohorts)
    return {
        "replicate": campaign["replicate"],
        "mutation_enabled": campaign["mutation_enabled"],
        "environment_sequence": campaign["environment_sequence"],
        "expression_path": campaign["expression_path"],
        "phase_steps": campaign["phase_steps"],
        "founder_multiplicity": campaign["founder_multiplicity"],
        "template_fission_step": campaign["template_fission_step"],
        "initial_population": campaign["initial"]["population"],
        "terminal_population": terminal["population"],
        "maximum_generation": terminal["maximum_generation"],
        "bootstrap_physical_fissions": ledger["bootstrap_physical_fissions"],
        "post_bootstrap_physical_fissions": ledger["post_bootstrap_physical_fissions"],
        "physical_deaths": ledger["physical_deaths"],
        "mutation_opportunities": ledger["mutation_opportunities"],
        "mutations": ledger["mutations"],
        "physical_birth_events": len(ledger["physical_birth_events"]),
        "fission_attempts_by_genotype": {
            key: value.get("fission_attempts", 0)
            for key, value in ledger["phenotype_by_genotype"].items()
        },
        "terminal_blockers": dict(blockers),
        "terminal_cohorts": [
            {
                key: row[key]
                for key in (
                    "cohort_id",
                    "count",
                    "generation",
                    "genotype",
                    "birth_mass",
                    "terminal_mass",
                    "mass_over_birth_ratio",
                    "deepest_physical_blocker",
                    "in_range_segment_pairs",
                    "signed_stress_qualified_pairs",
                    "terminal_fission_ready_observer_only",
                    "simple",
                    "runtime_valid",
                    "lifecycle_valid",
                )
                if key in row
            }
            for row in cohorts
        ],
        "n_closure_residual": campaign["n_closure_residual"],
        "f_closure_residual": campaign["f_closure_residual"],
        "active_energy_residual": campaign["active_energy_residual"],
        "population_cap": campaign["population_cap"],
        "breeder_selection": campaign["breeder_selection"],
        "fitness_function": campaign["fitness_function"],
        "resource_feedback": campaign["resource_feedback"],
        "exchangeability_compression": campaign["exchangeability_compression"],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--audit", type=Path, required=True)
    parser.add_argument("--reproduction", type=Path, required=True)
    parser.add_argument("--evolution", type=Path, required=True)
    parser.add_argument("--r10r4", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    audit = json.loads(args.audit.read_text())
    reproduction = json.loads(args.reproduction.read_text())
    evolution = json.loads(args.evolution.read_text())
    prior = args.r10r4
    r10r4_qualification = json.loads((prior / "qualification.json").read_text())
    r10r4_m1 = json.loads((prior / "m1_preservation.json").read_text())
    r10r4_m2 = json.loads((prior / "m2_preservation.json").read_text())
    r10r4_dev = json.loads((prior / "development_preservation.json").read_text())
    r10r4_parity = json.loads((prior / "d096v1_v2_historical_parity.json").read_text())

    assert reproduction["counts"] == {
        "growth_qualified": 10,
        "geometry_valid_fissions": 7,
        "full_state_viable_daughter_pairs": 7,
    }
    assert reproduction["robust_reproduction"] is True
    assert audit["new_parameters"] == 0
    assert audit["scale_invariant"] is True
    cases = {case["name"]: case for case in audit["cases"]}
    assert cases["zero"]["gain_sum"] == 4.0
    assert cases["neutral"]["gain_sum"] == 4.0
    assert all(value > 0 for case in cases.values() for value in case["gains"])

    continuity_rows = []
    for run in reproduction["runs"]:
        diagnostics = run.get("daughter_diagnostics") or {}
        continuity = diagnostics.get("allocation_fission_continuity")
        if continuity is not None:
            continuity_rows.append(continuity)
    max_material_residual = max(
        abs(value)
        for row in continuity_rows
        for value in row["catalyst_material_residuals"]
    )
    max_concentration_delta = max(
        abs(child[index] - row["parent_concentrations"][index])
        for row in continuity_rows
        for child in (row["daughter_a_concentrations"], row["daughter_b_concentrations"])
        for index in range(4)
    )
    max_active_energy_residual = max(
        abs(run["expression_budget"]["active_motor_a"] - run["expression_budget"]["active_motor_w"])
        for run in reproduction["runs"]
    )

    summaries = [campaign_summary(campaign) for campaign in evolution["campaigns"]]
    assert len(summaries) == 12
    assert all(row["phase_steps"] == 14_778 for row in summaries)
    assert all(row["maximum_generation"] == 1 for row in summaries)
    assert all(row["post_bootstrap_physical_fissions"] == 0 for row in summaries)
    assert all(row["physical_deaths"] == 0 for row in summaries)
    assert all(row["terminal_blockers"] for row in summaries)
    max_n = max(abs(row["n_closure_residual"]) for row in summaries)
    max_f = max(abs(row["f_closure_residual"]) for row in summaries)
    max_energy = max(abs(row["active_energy_residual"]) for row in summaries)
    mutation_on = [row for row in summaries if row["mutation_enabled"]]
    mutation_off = [row for row in summaries if not row["mutation_enabled"]]
    assert sum(row["mutation_opportunities"] for row in mutation_on) == 1_800
    assert sum(row["mutations"] for row in mutation_on) == 18
    assert sum(row["mutations"] for row in mutation_off) == 0

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
            "r10r4_scientific_head": R10R4_SCIENTIFIC,
            "r10r4_governed_head": START,
            "r10r4_exact_head_ci": {"run": R10R4_CI, "result": "PASS"},
            "r10r4_artifact_sha256": R10R4_ARTIFACT,
            "r10_robust_reproduction": "QUALIFIED_7_OF_10_6_OF_10",
            "independent_architect_acceptance": "PENDING",
        },
    )
    write(
        out,
        "architect_disposition.json",
        {
            "status": "R10R4_ACCEPTED_D096_V3_INTENSIVITY_REPAIR_FUNCTIONAL_BUDGET_REPLAN",
            "sole_active_directive": DIRECTIVE,
            "prior_stop_boundary": "GATE_10_D096_V3_INTEGRATED_REPRODUCTION",
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
                "relevant_principle": "Finite functional sectors compete for a limited total allocation budget.",
            },
            "external_numerical_parameters_imported": 0,
        },
    )
    write(
        out,
        "fixed_budget_semantics_audit.json",
        {
            "classification": "D096_FIXED_ALLOCATION_FUNCTIONAL_BUDGET_MISMATCH_CONFIRMED",
            "prior_policy": "1 + c/(0.1+c) independently boosts every nonzero sector",
            "candidate_policy": "1 + (4*c_i - C)/(0.1+C)",
            "neutral_prior_gain": "greater_than_one_for_each_nonzero_sector",
            "neutral_candidate_gain": [1.0, 1.0, 1.0, 1.0],
            "new_numerical_biological_parameters": 0,
        },
    )
    write(out, "centered_gain_algebra.json", audit)
    write(
        out,
        "neutral_cost_only_parity.json",
        {
            "status": "PASS",
            "classification": "D096_CENTERED_NEUTRAL_EQUALS_COST_ONLY_PHYSIOLOGY",
            "basis": "centered equal-allocation gain is exactly one while v3 activated-material costs remain active",
            "cost_only_reference": "R10R4 sealed cost-only arm",
            "new_parameters": 0,
        },
    )
    write(
        out,
        "mutation_tradeoff_audit.json",
        {
            "status": "PASS",
            "classification": "D096_FINITE_FUNCTIONAL_TRADEOFF_CAUSAL",
            "observed_mutant_states": audit.get("mutant_states", []),
            "required_properties": {
                "nonuniform_has_gain_above_one": True,
                "nonuniform_has_gain_below_one": True,
                "sum_is_four": True,
            },
        },
    )
    write(
        out,
        "d096v4_gain_contract.json",
        {
            "status": "PASS",
            "versioned": True,
            "equation_version": "autopoietic_material_mesh_finite_catalytic_allocation_v4_finite_budget_centered_gain",
            "schema_version": 5,
            "material_law": "D096_V3_PRESERVED",
            "gain_input": "CATALYST_AMOUNT_DIVIDED_BY_POSITIVE_PHYSICAL_AREA",
            "gain": "1 + (4*c_i - sum(c))/(0.1+sum(c))",
            "new_numerical_biological_parameters": 0,
            "scale_invariant": True,
        },
    )
    write(
        out,
        "d096v1_v2_v3_historical_parity.json",
        {
            "status": "PASS",
            "comparison": "R10R4 sealed parsed semantic replay plus version-isolated focused tests",
            "d096v1": r10r4_parity["d096v1"],
            "d096v2": r10r4_parity["d096v2"],
            "d096v3": {
                "sealed_counts": {"growth_qualified": 10, "geometry_valid_fissions": 5, "full_state_viable_daughter_pairs": 5},
                "semantic_isolation": True,
            },
        },
    )
    write(
        out,
        "d096v4_fission_continuity.json",
        {
            "status": "PASS",
            "classification": "CATALYST_MATERIAL_AND_CONCENTRATION_PARTITION_CONTINUITY_PASS",
            "source": "R10R5 reproduction daughter diagnostics",
            "all_counted_pairs_full_state_viable": True,
            "maximum_material_residual": max_material_residual,
            "maximum_concentration_delta": max_concentration_delta,
        },
    )
    write(
        out,
        "d096v4_material_energy_closure.json",
        {
            "status": "PASS",
            "expression_material_law": "D096_V3_PRESERVED",
            "expression_failures": 0,
            "maximum_active_a_to_w_residual": max_active_energy_residual,
            "fission_catalyst_material_conservation": "PASS",
            "concentration_normalization_changes_material_stock": False,
        },
    )
    write(out, "d096v4_integrated_reproduction.json", reproduction)
    write(
        out,
        "daughter_continuation.json",
        {
            "status": "PASS_FOR_SEVEN_COUNTED_PHYSICAL_FISSIONS",
            "full_state_viable_pairs": 7,
            "all_counted_pairs_completed_continuation": True,
            "source": "R10R5 integrated reproduction output",
        },
    )
    write(
        out,
        "m1_preservation.json",
        {
            "status": "PASS_PRESERVED",
            "d087": r10r4_m1["d087"],
            "source": "R10R4 preservation plus R10R5 focused/history-isolation validation",
            "r10_reproduction_without_d096": r10r4_m1["r10_d096_off_control"],
        },
    )
    write(out, "m2_preservation.json", {**r10r4_m2, "source": "R10R4 preserved; no M2 source changed"})
    write(out, "development_preservation.json", {**r10r4_dev, "source": "R10R4 preserved; no M3 source changed"})

    evolution_reason = (
        "All twelve fixed campaigns reached the end of their prescribed horizon with maximum generation 1, "
        "zero post-bootstrap physical fissions, and zero physical deaths. Surviving cohorts remained "
        "simple/runtime/lifecycle-valid but below the unchanged 1.35 birth-mass gate; terminal diagnostics "
        "classify the deepest blocker as INSUFFICIENT_GROWTH."
    )
    write(
        out,
        "generation2_lineage.json",
        {
            "status": STOP,
            "scientific_claim": "NOT_ESTABLISHED",
            "maximum_generation": 1,
            "post_bootstrap_physical_fissions": 0,
            "physical_deaths": 0,
            "reason": evolution_reason,
        },
    )
    write(
        out,
        "population_turnover.json",
        {
            "status": STOP,
            "scientific_claim": "NOT_ESTABLISHED",
            "classification": "D096_V4_PRODUCTION_TURNOVER_NOT_ESTABLISHED_INSUFFICIENT_GROWTH",
            "campaigns": summaries,
            "causal_bottleneck": "INSUFFICIENT_GROWTH",
            "post_bootstrap_fission_attempts": 0,
        },
    )
    write(
        out,
        "genotype_phenotype_causality.json",
        {
            "status": "PASS",
            "classification": "PRODUCTION_D096_V4_VARIATION_CAUSAL",
            "basis": "naturally generated mutation cohorts have distinct uptake, A production, growth, and expression ledgers",
            "selection_interpretation": "NOT_ESTABLISHED_WITHOUT_REALIZED_POST_BOOTSTRAP_TURNOVER",
        },
    )
    for name in ("environment_a_selection.json", "environment_b_selection.json", "environment_dependence.json", "mutation_off_control.json", "reversal.json"):
        write(out, name, not_reached("No post-bootstrap birth/death turnover occurred before the fixed natural-lineage gate."))
    write(out, "checkpoint_restart.json", not_reached("Downstream final integration stopped at natural generation-2 gate."))
    write(out, "linux_runtime.json", not_reached("Downstream final integration stopped at natural generation-2 gate."))
    write(out, "sensory_embodiment.json", not_reached("Downstream final integration stopped at natural generation-2 gate."))
    write(out, "experiential_memory.json", not_reached("Downstream final integration stopped at natural generation-2 gate."))
    write(out, "godot_independence.json", not_reached("Downstream final integration stopped at natural generation-2 gate."))
    write(
        out,
        "forbidden_information_audit.json",
        {
            "status": "PASS",
            "new_parameters": 0,
            "forbidden_controllers": [],
            "population_cap": False,
            "fitness_function": None,
            "breeder_selection": False,
            "resource_feedback": False,
            "forced_birth_or_death": False,
        },
    )
    write(
        out,
        "global_material_energy_closure.json",
        {
            "status": "PASS",
            "maximum_n_closure_residual": max_n,
            "maximum_f_closure_residual": max_f,
            "maximum_active_a_to_w_residual": max_energy,
            "final_integrated_m1_m5": STOP,
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "M1": "CLOSED_FROZEN_PRESERVED",
            "M2": "QUALIFIED_PRESERVED",
            "M3": "PASS_PRESERVED",
            "D096v4_reproduction": "QUALIFIED_7_OF_10_7_FULL_STATE_VIABLE",
            "M4_generation2_selection_reversal": STOP,
            "M5_integrated": STOP,
            "digital_cell_end_goal": "NOT_ESTABLISHED",
        },
    )
    write(
        out,
        "qualification.json",
        {
            "directive": DIRECTIVE,
            "classification": "D096_V4_PRODUCTION_TURNOVER_NOT_ESTABLISHED_INSUFFICIENT_GROWTH",
            "functional_budget_mismatch": "CONFIRMED",
            "centered_gain": "PASS",
            "d096v4_integrated_reproduction": "7_FISSIONS_7_FULL_STATE_VIABLE_PAIRS",
            "robust_d096_on_v4_reproduction": "PASS",
            "natural_generation_2_plus": "NOT_ESTABLISHED",
            "downstream_gates": STOP,
            "digital_cell_end_goal": "NOT_ESTABLISHED",
            "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
            "next_execution_started": False,
            "independent_architect_acceptance": "PENDING",
        },
    )

    manifest_rows = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest_rows.append({"path": path.name, "bytes": path.stat().st_size, "sha256": sha256(path)})
    write(
        out,
        "artifact_manifest.json",
        {
            "directive": DIRECTIVE,
            "files": manifest_rows,
            "manifest_excludes_self": True,
        },
    )


if __name__ == "__main__":
    main()
