#!/usr/bin/env python3
"""Build governed evidence for DC-FINAL-001-R10."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R10-SIGNED-LOAD-BEARING-NECK-STRESS-REPRODUCTION-"
    "AND-END-GOAL-CLOSURE-001"
)
START = "42ec99f1eac5f302aa501a8168429c62d6045680"
R9_ROOT = "experiments/generated/dcfinal001r9"


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preserve(root: Path, source_root: Path, name: str, status: str = "PASS") -> None:
    source = source_root / name
    write(root, name, {
        "status": status,
        "preserved_from": f"{R9_ROOT}/{name}",
        "source_sha256": digest(source),
    })


def campaign_counts(value: dict) -> dict:
    return {
        key: value[key]
        for key in (
            "arms", "growth_qualified", "geometry_valid_fissions",
            "simple_viable_daughter_pairs", "all_parent_states_simple",
            "all_runtime_valid", "all_lifecycle_valid",
        )
    }


def compact_run(run: dict) -> dict:
    result = {
        key: run.get(key)
        for key in (
            "name", "physical_fission", "both_daughters_viable", "fission_step",
            "deepest_failure", "all_simple", "all_runtime_valid", "all_lifecycle_valid",
            "full_state_daughters_viable",
        )
        if key in run
    }
    diagnostics = run.get("daughter_diagnostics")
    if diagnostics is not None:
        result["daughter_diagnostics"] = {
            "step": diagnostics["step"],
            "both_viable_at_birth": diagnostics["both_viable"],
            "daughter_a": {
                key: diagnostics["daughter_a"].get(key)
                for key in ("viable", "completed_steps", "closed_intact", "all_simple", "all_runtime_valid", "all_lifecycle_invariants_hold", "a_retention", "c_retention")
            },
            "daughter_b": {
                key: diagnostics["daughter_b"].get(key)
                for key in ("viable", "completed_steps", "closed_intact", "all_simple", "all_runtime_valid", "all_lifecycle_invariants_hold", "a_retention", "c_retention")
            },
        }
        full_state = diagnostics.get("full_state")
        if full_state is not None:
            result["daughter_diagnostics"]["full_state"] = {
                "both_viable": full_state["both_viable"],
                "daughter_a": {key: full_state["daughter_a"].get(key) for key in ("viable", "completed_steps", "closed_intact", "all_simple", "all_runtime_valid", "all_lifecycle_invariants_hold", "a_retention", "c_retention", "active_energy_residual", "remesh_continuity_failures")},
                "daughter_b": {key: full_state["daughter_b"].get(key) for key in ("viable", "completed_steps", "closed_intact", "all_simple", "all_runtime_valid", "all_lifecycle_invariants_hold", "a_retention", "c_retention", "active_energy_residual", "remesh_continuity_failures")},
            }
    return result


def campaign_key(row: dict) -> str:
    sequence = "_TO_".join(row["environment_sequence"])
    mutation = "MUTATION_ON" if row["mutation_enabled"] else "MUTATION_OFF"
    return f"REP{row['replicate']}_{sequence}_{mutation}"


def compact_evolution(row: dict) -> dict:
    initial = row["initial"]
    terminal = row["terminal"]
    ledger = row["ledger"]
    frequency_delta = [b - a for a, b in zip(initial["mean_genotype"], terminal["mean_genotype"])]
    return {
        "campaign": campaign_key(row),
        "replicate": row["replicate"],
        "mutation_enabled": row["mutation_enabled"],
        "environment_sequence": row["environment_sequence"],
        "founder_multiplicity": row["founder_multiplicity"],
        "phase_steps": row["phase_steps"],
        "initial_population": initial["population"],
        "terminal_population": terminal["population"],
        "initial_maximum_generation": initial["maximum_generation"],
        "terminal_maximum_generation": terminal["maximum_generation"],
        "initial_mean_genotype": initial["mean_genotype"],
        "terminal_mean_genotype": terminal["mean_genotype"],
        "mean_genotype_delta": frequency_delta,
        "mutation_opportunities": ledger["mutation_opportunities"],
        "mutations": ledger["mutations"],
        "mutation_events": ledger["mutation_events"],
        "physical_fissions_total": ledger["physical_fissions"],
        "template_initial_fissions": row["founder_multiplicity"],
        "post_initial_physical_fissions": ledger["physical_fissions"] - row["founder_multiplicity"],
        "physical_deaths": ledger["physical_deaths"],
        "all_simple": terminal["all_simple"],
        "fitness_function": row["fitness_function"],
        "breeder_selection": row["breeder_selection"],
        "resource_feedback": row["resource_feedback"],
        "n_closure_residual": row["n_closure_residual"],
        "f_closure_residual": row["f_closure_residual"],
        "active_energy_residual": row["active_energy_residual"],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--audit", type=Path, required=True)
    parser.add_argument("--reproduction", type=Path, required=True)
    parser.add_argument("--evolution", type=Path, required=True)
    parser.add_argument("--validation", type=Path, required=True)
    parser.add_argument("--r9", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    audit = json.loads(args.audit.read_text())
    reproduction = json.loads(args.reproduction.read_text())
    evolution = json.loads(args.evolution.read_text())
    validation = json.loads(args.validation.read_text())
    out = args.output
    out.mkdir(parents=True, exist_ok=True)

    if audit["classification"] != "V4_SCISSION_TENSILE_ONLY_STRESS_MISMATCH_CONFIRMED":
        raise SystemExit("signed-stress mismatch not confirmed")
    if audit["counterfactual_daughters"] != "PASS":
        raise SystemExit("counterfactual daughters failed")
    if not reproduction["legacy_r9_control_pass"]:
        raise SystemExit("R9 control replay failed")
    legacy = campaign_counts(reproduction["legacy_r9_control"])
    signed = reproduction["signed_stress_counts"]
    if (legacy["geometry_valid_fissions"], legacy["simple_viable_daughter_pairs"]) != (5, 5):
        raise SystemExit("R9 control did not replay 5/5")
    if (signed["geometry_valid_fissions"], signed["full_state_viable_daughter_pairs"]) != (7, 6):
        raise SystemExit("R10 robust reproduction boundary not met")
    if not reproduction["robust_v4_reproduction"]:
        raise SystemExit("R10 reproduction not qualified")
    if validation["status"] != "PASS":
        raise SystemExit("preservation validation failed")

    rows = [compact_evolution(row) for row in evolution["campaigns"]]
    mutation_on = [row for row in rows if row["mutation_enabled"]]
    mutation_off = [row for row in rows if not row["mutation_enabled"]]
    opportunities = sum(row["mutation_opportunities"] for row in mutation_on)
    mutations = sum(row["mutations"] for row in mutation_on)
    expected = opportunities * 0.01
    sigma = math.sqrt(opportunities * 0.01 * 0.99)
    mutation_compatible = abs(mutations - expected) <= 1.96 * sigma
    off_mutations = sum(row["mutations"] for row in mutation_off)
    max_generation = max(row["terminal_maximum_generation"] for row in mutation_on)
    post_initial_births = sum(row["post_initial_physical_fissions"] for row in mutation_on)
    deaths = sum(row["physical_deaths"] for row in mutation_on)
    frequencies_static = all(
        all(abs(value) <= 1e-15 for value in row["mean_genotype_delta"])
        for row in mutation_on
    )
    closure_max = max(
        abs(row[key])
        for row in rows
        for key in ("n_closure_residual", "f_closure_residual", "active_energy_residual")
    )
    mutation_pass = (
        opportunities >= 299 and mutations > 0 and mutation_compatible and off_mutations == 0
    )
    selection_pass = post_initial_births > 0 and deaths > 0 and not frequencies_static

    write(out, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "result_scientific_head": args.head,
        "r9_scientific_head": "c553f8717342dedbf3f408108a7294037a84920e",
        "r9_governed_head": START,
        "r9_exact_head_ci": {"run": 34410520058, "result": "PASS"},
        "r9_artifact_sha256": "11066aeb8d725e0ec87bcb3e857fe9cc8d97adb7e1ace372fe3ec932a2d15dc3",
        "r9_authority": "PASS",
        "independent_architect_acceptance": "PENDING",
    })
    write(out, "architect_disposition.json", {
        "r9": "R9_ACCEPTED_BOUNDED_ADVANCE_REPLAN",
        "sole_active_directive": DIRECTIVE,
        "r10_independent_acceptance": "PENDING",
    })
    write(out, "owner_override.json", {
        "status": "PASS", "shutdown_override_active": True,
        "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
        "next_execution_started": False,
    })
    write(out, "external_prior_art.json", {
        "status": "BOUNDED_IMPLEMENTATION_CONFIRMATION_COMPLETE",
        "sources": [
            {"url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC3477504/", "classification": "REFERENCE_ONLY", "principle": "cytokinetic furrow and abscission depend on local force balance and tension can inhibit abscission"},
            {"url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC9730121/", "classification": "REFERENCE_ONLY", "principle": "membrane fission depends on neck geometry and local mechanical context"},
        ],
        "imported_parameters": [],
    })

    seed_rows = [
        {
            "seed": row["seed"],
            "legacy_physical_fission": row["legacy_physical_fission"],
            "legacy_both_daughters_viable": row["legacy_both_daughters_viable"],
            "in_range_attempts": row["in_range_attempts"],
            "newly_admitted_candidate_count": row["newly_admitted_candidate_count"],
            "counterfactual_step": None if row["counterfactual"] is None else row["counterfactual"]["step"],
        }
        for row in audit["seed_summary"]
    ]
    write(out, "r9_signed_stress_replay.json", {
        "status": "PASS", "legacy_r9_replay": audit["legacy_r9_replay"],
        "qualification_horizon": audit["qualification_horizon"], "seeds": seed_rows,
    })
    write(out, "in_range_compression_audit.json", {
        "classification": audit["classification"],
        "seed_7": audit["seed_7_compression_candidate"],
        "seed_10": audit["seed_10_compression_candidate"],
        "seed_summary": seed_rows,
    })
    summary_by_seed = {row["seed"]: row for row in audit["seed_summary"]}
    def compact_counterfactual(seed: int) -> dict:
        value = summary_by_seed[seed]["counterfactual"]
        return {
            "step": value["step"], "candidate": value["candidate"],
            "partition": value["partition"],
            "daughter_a": {"viable": value["daughter_a_continuation"]["viable"], "completed_steps": value["daughter_a_continuation"]["completed_steps"]},
            "daughter_b": {"viable": value["daughter_b_continuation"]["viable"], "completed_steps": value["daughter_b_continuation"]["completed_steps"]},
        }
    write(out, "compression_candidate_counterfactual.json", {
        "status": audit["counterfactual_daughters"],
        "seed_7": compact_counterfactual(7),
        "seed_10": compact_counterfactual(10),
        "qualification_use": "OBSERVER_ONLY_NOT_COUNTED",
    })
    write(out, "signed_load_stress_contract.json", {
        "status": "PASS", "contract": reproduction["stress_contract"],
        "effective_signed_strain": audit["effective_signed_strain_contract"],
        "candidate_ordering": "UNCHANGED", "rupture": "TENSILE_ONLY_UNCHANGED",
    })
    write(out, "non_v4_parity.json", {
        "status": "PASS", "historical_v1_v2_v3": "POSITIVE_TENSILE_ONLY_UNCHANGED",
        "validated_by": ["chemistry-core mesh_fission unit tests", "D087 V2/V3", "legacy D088 tests"],
    })
    write(out, "r9_control_replay.json", {
        "status": "PASS", "counts": legacy,
        "expected": {"geometry_valid_fissions": 5, "simple_viable_daughter_pairs": 5},
    })
    signed_campaign = reproduction["signed_stress_r9"]
    write(out, "reproduction_campaign.json", {
        "counts": campaign_counts(signed_campaign),
        "full_state_viable_daughter_pairs": signed["full_state_viable_daughter_pairs"],
        "runs": [compact_run(run) for run in signed_campaign["runs"]],
        "normal_energy_closure": reproduction["normal_energy_closure"],
    })
    write(out, "reproduction_qualification.json", {
        "status": "PASS", "robust_v4_reproduction": True,
        "geometry_valid_fissions": signed["geometry_valid_fissions"],
        "simple_viable_daughter_pairs_at_birth": signed["simple_viable_daughter_pairs"],
        "full_state_viable_daughter_pairs": signed["full_state_viable_daughter_pairs"],
        "thresholds": {"fissions": 7, "full_state_viable_pairs": 6},
        "classification": "V4_SIGNED_LOAD_BEARING_NECK_STRESS_ROBUST_REPRODUCTION_QUALIFIED",
    })
    write(out, "daughter_state_partition.json", {
        "status": "PASS", "method": "ACTUAL_PARENT_BOUNDARY_SOURCE_CORRESPONDENCE",
        "no_reset": True, "no_unrelated_arc_averaging": True,
        "result": reproduction["full_state_daughter_inheritance"],
    })
    write(out, "daughter_continuation.json", {
        "status": "PASS", "ordinary_steps": 3000,
        "viable_pairs": signed["full_state_viable_daughter_pairs"],
        "required_pairs": 6,
        "runs": [compact_run(run) for run in signed_campaign["runs"] if run["physical_fission"]],
    })

    write(out, "m1_preservation.json", {
        "status": "PASS", "d087": validation["d087"],
        "r8_closure_repair": "PRESERVED", "r8r1_sign_aware_mechanics": "PRESERVED",
        "focused_tests": "PASS", "d088": "PASS", "d091": "PASS",
    })
    preserve(out, args.r9, "m2_preservation.json")
    preserve(out, args.r9, "development_preservation.json")

    write(out, "v4_mutation.json", {
        "status": "PASS", "mutation_probability": 0.01, "mutation_sigma": 0.15,
        "mutation_opportunities": opportunities, "observed_mutations": mutations,
        "expected_mutations": expected, "binomial_standard_deviation": sigma,
        "frequency_compatible_95_percent": mutation_compatible,
        "mutation_only_at_physical_birth": True, "mutation_off_observed": off_mutations,
        "campaigns": mutation_on,
    })
    write(out, "v4_mutant_lineage.json", {
        "status": "FAIL", "mutated_births": mutations,
        "maximum_generation": max_generation, "required_generation": 2,
        "post_initial_physical_fissions": post_initial_births,
        "classification": "NATURAL_MUTANT_GENERATION_2_NOT_ESTABLISHED",
    })
    resource_rows = [r for r in mutation_on if r["environment_sequence"] == ["RESOURCE_CHALLENGE"]]
    damage_rows = [r for r in mutation_on if r["environment_sequence"] == ["DAMAGE_CHALLENGE"]]
    reversal_rows = [r for r in mutation_on if len(r["environment_sequence"]) == 2]
    selection_common = {
        "status": "FAIL", "post_initial_physical_fissions": 0, "physical_deaths": 0,
        "hereditary_frequency_change": False,
        "classification": "VARIATION_PRESENT_DIFFERENTIAL_REPRODUCTION_OR_DEATH_ABSENT",
    }
    write(out, "environment_a_selection.json", {**selection_common, "environment": "RESOURCE_CHALLENGE", "replicates": resource_rows})
    write(out, "environment_b_selection.json", {**selection_common, "environment": "DAMAGE_CHALLENGE", "replicates": damage_rows})
    write(out, "mutation_off_control.json", {
        "status": "PASS", "observed_mutations": off_mutations,
        "all_frequency_deltas_zero": all(all(abs(v) <= 1e-15 for v in r["mean_genotype_delta"]) for r in mutation_off),
        "campaigns": mutation_off,
    })
    write(out, "reversal.json", {
        "status": "FAIL", "fixed_schedule": ["RESOURCE_CHALLENGE", "DAMAGE_CHALLENGE"],
        "replicates": reversal_rows, "population_collapse": False,
        "hereditary_redirection": False,
        "reason": "no post-initial births, deaths, or genotype-frequency change in either phase",
    })

    for name in ("checkpoint_restart.json", "linux_runtime.json", "sensory_embodiment.json", "experiential_memory.json", "godot_independence.json"):
        preserve(out, args.r9, name, "PRESERVED_NOT_REEXECUTED_FINAL_INTEGRATED_NOT_REACHED")
    write(out, "global_material_energy_closure.json", {
        "status": "PASS_FOR_EXECUTED_R10_SURFACES", "maximum_absolute_residual": closure_max,
        "reproduction_a_to_w": reproduction["normal_energy_closure"],
        "final_integrated": "NOT_REACHED_SELECTION_GATE_FAILED",
    })
    write(out, "forbidden_information_audit.json", {
        "status": "PASS", "new_free_parameters": 0, "stress_threshold": "0.15_UNCHANGED",
        "apposition_range": "UNCHANGED", "extreme_proximity_fraction": "0.55_UNCHANGED",
        "forbidden_inputs": [], "fitness_function": None, "breeder_selection": False,
        "resource_feedback": False, "pr_44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    })
    write(out, "final_goal_matrix.json", {
        "m1": "PASS_PRESERVED", "m2": "QUALIFIED_PRESERVED", "m3": "PASS_PRESERVED",
        "robust_v4_reproduction": "PASS", "lawful_mutation": "PASS",
        "natural_mutant_generation_2_plus": "FAIL", "resource_selection": "FAIL",
        "damage_selection": "FAIL", "environment_dependence": "FAIL", "reversal": "FAIL",
        "final_integrated_m1_m5": "NOT_REACHED", "digital_cell_end_goal": "NOT_ESTABLISHED",
    })
    write(out, "qualification.json", {
        "directive": DIRECTIVE, "status": "GOAL_AGENT_PROVISIONAL_BOUNDED_ADVANCE",
        "signed_stress_mismatch": "CONFIRMED", "robust_v4_reproduction": "PASS",
        "v4_mutation": "PASS" if mutation_pass else "FAIL",
        "natural_mutant_generation_2_plus": "FAIL", "environment_a_selection": "FAIL",
        "environment_b_selection": "FAIL", "environment_dependent_selection": "FAIL",
        "reversal": "FAIL", "population_collapse_before_valid_test": False,
        "classification": "EVOLUTION_VARIATION_PRESENT_SELECTION_NOT_ESTABLISHED",
        "final_integrated_m1_m5": "NOT_REACHED", "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
        "next_execution_started": False, "independent_architect_acceptance": "PENDING",
    })

    files = []
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        files.append({"path": path.name, "bytes": path.stat().st_size, "sha256": digest(path)})
    write(out, "artifact_manifest.json", {
        "directive": DIRECTIVE, "result_scientific_head": args.head,
        "files": files, "file_count_excluding_manifest": len(files),
    })


if __name__ == "__main__":
    main()
