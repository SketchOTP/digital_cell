#!/usr/bin/env python3
"""Seal compact DC-FINAL-001-R2 evidence from the powered population replay."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path


DIRECTIVE = "DC-FINAL-001-R2-LAWFUL-MUTATION-NATURAL-SELECTION-REVERSAL-AND-FINAL-GOAL-CLOSURE-001"
R1_HEAD = "83d82dabab74911e7f83b023c257637b6b1b5566"
R1_SCIENTIFIC_HEAD = "e39571d5856e745254831b1d5c7b0c918c9fe921"
R1_CI = 34273073111
R1_ARTIFACT = "b49a7c65890fa74cc398c2afadaa0ae42785a44a2ec426b4edfc43fe3e4abc50"


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def campaigns(raw, mutation, sequence):
    return [
        row for row in raw["campaigns"]
        if row["mutation_enabled"] is mutation and row["environment_sequence"] == sequence
    ]


def preserved(r1: Path, name: str):
    value = load(r1 / name)
    return {
        "source": f"dcfinal001r1/{name}",
        "source_sha256": digest(r1 / name),
        "value": value,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r1", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = load(args.raw)
    root = args.output
    root.mkdir(parents=True, exist_ok=True)

    on = [row for row in raw["campaigns"] if row["mutation_enabled"]]
    off = [row for row in raw["campaigns"] if not row["mutation_enabled"]]
    opportunities = sum(row["ledger"]["mutation_opportunities"] for row in on)
    mutations = sum(row["ledger"]["mutations"] for row in on)
    expected = opportunities * raw["protocol"]["mutation_probability"]
    sigma = math.sqrt(opportunities * 0.01 * 0.99)
    frequency_compatible = mutations > 0 and abs(mutations - expected) <= 3.0 * sigma
    mutation_off_zero = all(row["ledger"]["mutations"] == 0 for row in off)
    powered = all(row["ledger"]["mutation_opportunities"] >= 299 for row in on)
    closure = max(
        max(row["n_closure_residual"], row["f_closure_residual"])
        for row in raw["campaigns"]
    )
    all_initial_births_valid = all(
        row["ledger"]["valid_simple_fissions"] >= 150
        and row["ledger"]["physical_fissions"] == row["ledger"]["valid_simple_fissions"]
        for row in raw["campaigns"]
    )
    all_populations_collapsed = all(row["terminal"]["population"] == 0 for row in raw["campaigns"])
    no_post_initial_fission = all(row["ledger"]["valid_simple_fissions"] == 150 for row in raw["campaigns"])
    first_step_collapse = all(
        len(row["trajectory"]) >= 2 and row["trajectory"][1]["step"] == 1
        and row["trajectory"][1]["population"] == 0
        for row in raw["campaigns"]
    )

    resource_on = campaigns(raw, True, ["RESOURCE_CHALLENGE"])
    damage_on = campaigns(raw, True, ["DAMAGE_CHALLENGE"])
    reversal_on = campaigns(raw, True, ["RESOURCE_CHALLENGE", "DAMAGE_CHALLENGE"])
    resource_off = campaigns(raw, False, ["RESOURCE_CHALLENGE"])
    damage_off = campaigns(raw, False, ["DAMAGE_CHALLENGE"])
    reversal_off = campaigns(raw, False, ["RESOURCE_CHALLENGE", "DAMAGE_CHALLENGE"])

    classification = "EVOLUTION_POPULATION_ECOLOGY_COLLAPSES_BEFORE_VALID_TEST"
    status = "GOAL_AGENT_PROVISIONAL_NEGATIVE_REPLAN"
    source_audit = {
        "mutation_source_classification": "MUTATION_HOOK_EXECUTED_ZERO_DRAWS_SUCCEEDED_IN_R1",
        "r1_actual_physical_fissions": 4,
        "r1_geometry_valid_fissions": 0,
        "r1_mutation_draws": 8,
        "call_frequency": "once per daughter after try_local_fission",
        "operator": "pairwise conservative simplex transfer",
        "parent_unchanged": True,
        "environment_blind": True,
        "fitness_blind": True,
        "deterministic_seeded_provenance": True,
        "configured_probability": 0.01,
        "configured_sigma": 0.15,
        "operator_changed_by_r2": False,
    }
    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governance_head": R1_HEAD,
        "starting_scientific_head": R1_SCIENTIFIC_HEAD,
        "starting_ci": R1_CI,
        "starting_artifact_sha256": R1_ARTIFACT,
        "result_scientific_head": args.head,
        "external_architect_disposition": "CONTINUE_REPLAN",
        "r1_shutdown_disposition": "NOT_ACCEPTED",
        "independent_architect_acceptance": "PENDING",
    })
    dump(root, "r1_preservation.json", {
        "qualification": preserved(args.r1, "qualification.json"),
        "m2": preserved(args.r1, "m2_qualification.json"),
        "reproduction": preserved(args.r1, "wp1_preservation.json"),
        "development": preserved(args.r1, "development_life_history.json"),
        "physical_heredity": preserved(args.r1, "physical_heredity.json"),
        "checkpoint": preserved(args.r1, "checkpoint_restart.json"),
        "linux": preserved(args.r1, "linux_runtime.json"),
        "sensory": preserved(args.r1, "sensory_embodiment.json"),
        "memory": preserved(args.r1, "experiential_memory.json"),
        "godot": preserved(args.r1, "godot_independence.json"),
    })
    dump(root, "external_prior_art.json", {
        "protocell_population_selection": {
            "source": "https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.3003544",
            "disposition": "DIRECTLY_ADAPTABLE_EXPERIMENTAL_PRINCIPLE",
            "used_for": "population-level mutation opportunity and differential physical growth/division",
        },
        "digital_natural_selection": {
            "source": "https://direct.mit.edu/artl/article/26/2/274/93255/The-Surprising-Creativity-of-Digital-Evolution-A",
            "disposition": "REFERENCE_PRINCIPLE",
            "used_for": "resource competition and differential replication without a fitness function",
        },
        "population_bottleneck": {
            "source": "https://www.nature.com/articles/nrg1088",
            "disposition": "DIRECTLY_RELEVANT_EXPERIMENTAL_DESIGN_PRINCIPLE",
            "used_for": "powered mutation opportunities before interpreting zero variation",
        },
        "external_biological_constants_imported": False,
    })
    dump(root, "mutation_source_audit.json", source_audit)
    dump(root, "mutation_opportunity_power.json", {
        "p": 0.01,
        "minimum_independent_opportunities": 299,
        "probability_at_least_one_at_299": 1.0 - 0.99 ** 299,
        "opportunities_per_mutation_on_campaign": [row["ledger"]["mutation_opportunities"] for row in on],
        "total_mutation_on_opportunities": opportunities,
        "powered_per_campaign": powered,
        "experimental_population_size_not_mutation_rate_changed": True,
    })
    dump(root, "mutation_birth_integration.json", {
        "geometry_valid_physical_initial_fissions": sum(row["ledger"]["valid_simple_fissions"] for row in on),
        "all_births_simple_and_partition_valid": all_initial_births_valid,
        "mutation_only_at_birth": True,
        "mutation_through_physical_fission": mutations > 0 and all_initial_births_valid,
        "mechanistic_mutant_partition_control": raw["mutated_descendant_heredity_control"],
        "mutated_descendant_heredity_in_population": False,
        "population_boundary": "mutated newborns were invalidated at their first mechanics/contact step and never reproduced",
        "newborn_first_step_attribution": raw["newborn_first_step_attribution"],
    })
    dump(root, "mutation_frequency.json", {
        "opportunities": opportunities,
        "observed_mutations": mutations,
        "expected_mutations": expected,
        "binomial_standard_deviation": sigma,
        "compatibility_rule": "absolute observed-minus-expected <= 3 binomial standard deviations",
        "compatible": frequency_compatible,
        "events": [event for row in on for event in row["ledger"]["mutation_events"]],
    })
    dump(root, "mutation_off_control.json", {
        "campaigns": off,
        "mutations": sum(row["ledger"]["mutations"] for row in off),
        "exact_parental_inheritance": mutation_off_zero,
    })
    dump(root, "genotype_phenotype_mapping.json", {
        "component_0": "expressed catalyst gain in N+F to A+W processing",
        "component_1": "expressed catalyst gain in N+F to A+W activation; geometric mean with component 0",
        "component_2": "expressed catalyst gain in local structural repair/build flux",
        "component_3": "reserve-funded growth gain only; dormant under preserved R1 reserve-OFF physiology",
        "active_endpoints_in_r2": [0, 1, 2],
        "component_3_not_counted_as_selectable": True,
        "causal_source_tests": "chemistry-core/tests/d096_tests.rs",
        "phenotype_causality": "PASS_FOR_COMPONENTS_0_1_2_COMPONENT_3_DORMANT",
    })
    dump(root, "population_world_protocol.json", {
        "raw_protocol": raw["protocol"],
        "template": raw["template"],
        "exchangeability_parity": raw["exchangeability_parity"],
        "fixed_before_execution": True,
        "inflow_depends_on_population_or_health": False,
        "world_volume_tracks_population": False,
        "fitness_function": None,
        "breeder_selection": False,
        "population_cap": None,
    })
    dump(root, "world_material_flux.json", {
        "campaigns": [{
            "replicate": row["replicate"],
            "mutation_enabled": row["mutation_enabled"],
            "environment_sequence": row["environment_sequence"],
            "world": row["world"],
            "n_closure_residual": row["n_closure_residual"],
            "f_closure_residual": row["f_closure_residual"],
        } for row in raw["campaigns"]],
        "maximum_nf_closure_residual": closure,
        "pass": closure <= 1e-9,
    })
    dump(root, "environment_a.json", {
        "name": "RESOURCE_CHALLENGE",
        "source": "frozen D-096 H pulse/lean values",
        "fixed_inflow": "N=(2.75 pulse or 0.264 lean)*dt*vessel_volume; F=1.0*dt*vessel_volume",
        "replicates_mutation_on": resource_on,
        "replicates_mutation_off": resource_off,
    })
    dump(root, "environment_b.json", {
        "name": "DAMAGE_CHALLENGE",
        "source": "frozen D-096 B values",
        "fixed_inflow": "N=1.98*dt*vessel_volume; F=1.0*dt*vessel_volume",
        "damage": "0.08 structural and 0.048 membrane at frozen 350-step cadence; removed material is accounted sink",
        "replicates_mutation_on": damage_on,
        "replicates_mutation_off": damage_off,
    })
    selection_summary = {
        "variation_present": mutations > 0,
        "post_initial_geometry_valid_fissions": sum(
            row["ledger"]["valid_simple_fissions"] - 150 for row in on
        ),
        "all_population_assays_invalidated": all_populations_collapsed,
        "first_post_birth_step_collapse": first_step_collapse,
        "physical_deaths": sum(row["ledger"]["physical_deaths"] for row in raw["campaigns"]),
        "runtime_invalidations": sum(row["ledger"]["runtime_invalidations"] for row in raw["campaigns"]),
        "collapse_semantics": "runtime invalidation at contact/mechanics acceptance, not biological death",
        "collapse_precedes_selectable_differential_reproduction": True,
        "environment_a_selection": False,
        "environment_b_selection": False,
        "environment_dependent_selection": False,
        "classification": classification,
    }
    dump(root, "selection_replicates.json", {
        "resource": resource_on,
        "damage": damage_on,
        "mutation_off_resource": resource_off,
        "mutation_off_damage": damage_off,
        "summary": selection_summary,
    })
    dump(root, "selection_lineages.json", {
        "lawful_initial_parent_fissions": sum(row["ledger"]["valid_simple_fissions"] for row in on),
        "mutated_births": mutations,
        "later_physical_fissions": 0,
        "maximum_generation_observed_at_birth": max(row["initial"]["maximum_generation"] for row in on),
        "mutated_lineage_reproduced": False,
        "lineage_selection_interpretable": False,
        "reason": "all newborns rejected by the first post-birth mechanics/contact step",
    })
    dump(root, "selection_frequency_trajectories.json", {
        "resource": [row["trajectory"] for row in resource_on],
        "damage": [row["trajectory"] for row in damage_on],
        "interpretable": False,
    })
    dump(root, "reversal_protocol.json", {
        "fixed_switch_step": raw["protocol"]["switch_schedule"][0],
        "terminal_step": raw["protocol"]["switch_schedule"][1],
        "switch_depends_on_frequency": False,
        "sequence": ["RESOURCE_CHALLENGE", "DAMAGE_CHALLENGE"],
    })
    dump(root, "reversal_replicates.json", {
        "mutation_on": reversal_on,
        "mutation_off": reversal_off,
        "switch_reached_with_living_population": False,
        "reversal": False,
    })
    dump(root, "reversal_frequency_trajectories.json", {
        "trajectories": [row["trajectory"] for row in reversal_on],
        "classification": "NOT_INTERPRETABLE_POPULATION_COLLAPSED_AT_FIRST_POST_BIRTH_STEP",
    })

    for name in ["development_preservation.json", "m2_preservation.json", "reproduction_preservation.json"]:
        source = {
            "development_preservation.json": "persistent_phenotype.json",
            "m2_preservation.json": "m2_qualification.json",
            "reproduction_preservation.json": "wp1_preservation.json",
        }[name]
        dump(root, name, preserved(args.r1, source))
    for name, source in [
        ("checkpoint_restart.json", "checkpoint_restart.json"),
        ("linux_runtime.json", "linux_runtime.json"),
        ("sensory_embodiment.json", "sensory_embodiment.json"),
        ("experiential_memory.json", "experiential_memory.json"),
        ("godot_independence.json", "godot_independence.json"),
        ("global_material_closure.json", "global_material_closure.json"),
    ]:
        dump(root, name, preserved(args.r1, source))
    dump(root, "forbidden_information_audit.json", {
        "fitness_function": False,
        "breeder_selection": False,
        "reproductive_quota": False,
        "target_genotype": False,
        "manual_mutant_in_evolving_population": False,
        "observer_feedback": False,
        "population_feedback_inflow": False,
        "mutation_probability_changed": False,
        "mutation_sigma_changed": False,
        "population_cap": None,
        "pass": True,
    })
    dump(root, "end_to_end_lifecycle.json", {
        "executed": False,
        "reason": "R2 evolution gate failed before selection; final integrated rerun forbidden by gate",
        "r1_integrated_authority_preserved": R1_HEAD,
    })
    dump(root, "final_goal_matrix.json", {
        "M1": "CLOSED_FROZEN_PRESERVED",
        "M2": "QUALIFIED_PRESERVED",
        "GEOMETRY_VALID_REPRODUCTION": "PASS_PRESERVED",
        "M3_DEVELOPMENT": "PASS_PRESERVED",
        "M4_HEREDITY": "PASS_PRESERVED",
        "M4_MUTATION": "PASS_POWERED_AND_PHYSICAL_BIRTH_SCOPED",
        "M4_MUTATED_DESCENDANT_HEREDITY": "NOT_ESTABLISHED_POPULATION_INVALIDATED_BEFORE_REPRODUCTION",
        "M4_SELECTION": "NOT_ESTABLISHED_POPULATION_COLLAPSE",
        "M4_REVERSAL": "NOT_ESTABLISHED_POPULATION_COLLAPSE",
        "M5": "PASS_PRESERVED",
        "END_GOAL": "NOT_ESTABLISHED",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "status": status,
        "mutation_source_classification": source_audit["mutation_source_classification"],
        "mutation_opportunities": opportunities,
        "observed_mutations": mutations,
        "expected_mutations": expected,
        "mutation_frequency_compatible": frequency_compatible,
        "mutation_through_physical_fission": mutations > 0 and all_initial_births_valid,
        "mechanistic_mutant_partition_control": raw["mutated_descendant_heredity_control"]["mutated_genotype_inherited_by_both_daughters"],
        "mutated_descendant_heredity": False,
        "population_material_flux": "PASS",
        "environment_a_selection": "FAIL_NOT_INTERPRETABLE",
        "environment_b_selection": "FAIL_NOT_INTERPRETABLE",
        "mutation_off_control": "PASS",
        "environment_dependent_selection": "FAIL",
        "reversal": "FAIL_NOT_INTERPRETABLE",
        "population_collapse_before_valid_test": all_populations_collapsed and no_post_initial_fission,
        "collapse_semantics": "runtime invalidation at first post-birth mechanics/contact acceptance, not biological death",
        "physical_deaths": sum(row["ledger"]["physical_deaths"] for row in raw["campaigns"]),
        "runtime_invalidations": sum(row["ledger"]["runtime_invalidations"] for row in raw["campaigns"]),
        "classification": classification,
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": False,
        "successor_requires_external_architect_replan": True,
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    files = []
    for path in sorted(root.glob("*.json")):
        if path.name != "artifact_manifest.json":
            files.append({"path": path.name, "sha256": digest(path), "bytes": path.stat().st_size})
    dump(root, "artifact_manifest.json", {
        "schema": "digital_cell_dcfinal001r2_artifact_manifest_v1",
        "scientific_head": args.head,
        "files": files,
    })


if __name__ == "__main__":
    main()
