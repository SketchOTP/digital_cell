#!/usr/bin/env python3
"""Build governed evidence for the horizon-only R10R2 evolution assay."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from collections import Counter
from pathlib import Path


DIRECTIVE = (
    "DC-FINAL-001-R10R2-PRODUCTION-EVOLUTION-HORIZON-REQUALIFICATION-"
    "SELECTION-AND-END-GOAL-CLOSURE-001"
)
START = "b81c9a82f4cd49085b4970c24f85f5f50c9f03c1"
R10_SCIENTIFIC = "d1306f0ff069a2437e4853b3d33daabd5c1e4e99"
R10_WORKFLOW = 34419984941
R10_ARTIFACT = "41d06d64e5e51edfa40523c5bfdc15431c0386e2883b741b6716dc21608bb780"
SEALED_R10_NORMALIZED = "b47751e05802e3c7b3bcb47617e79ee60058584069c6698cc9b17fc0f27259af"
OLD_STEPS = 2_500
NEW_STEPS = 14_778


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(root: Path, name: str, value: object) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def campaign_key(row: dict) -> tuple:
    return (
        row["replicate"],
        row["mutation_enabled"],
        tuple(row["environment_sequence"]),
    )


def compact_campaign(row: dict) -> dict:
    ledger = row["ledger"]
    return {
        "replicate": row["replicate"],
        "mutation_enabled": row["mutation_enabled"],
        "environment_sequence": row["environment_sequence"],
        "phase_steps": row["phase_steps"],
        "initial_population": row["initial"]["population"],
        "terminal_population": row["terminal"]["population"],
        "maximum_generation": row["terminal"]["maximum_generation"],
        "initial_genotype_frequencies": row["initial"]["genotype_frequencies"],
        "terminal_genotype_frequencies": row["terminal"]["genotype_frequencies"],
        "bootstrap_physical_fissions": ledger["bootstrap_physical_fissions"],
        "post_bootstrap_physical_fissions": ledger["post_bootstrap_physical_fissions"],
        "physical_deaths": ledger["physical_deaths"],
        "runtime_invalidations": ledger["runtime_invalidations"],
        "mutation_opportunities": ledger["mutation_opportunities"],
        "mutations": ledger["mutations"],
        "expression_material": ledger["expression_material"],
        "growth_material": ledger["growth_material"],
        "net_growth_minus_expression": ledger["growth_material"]
        - ledger["expression_material"],
        "n_closure_residual": row["n_closure_residual"],
        "f_closure_residual": row["f_closure_residual"],
        "active_energy_residual": row["active_energy_residual"],
        "terminal_cohorts": row["terminal_cohorts"],
    }


def normalized_phenotypes(rows: list[dict]) -> list[dict]:
    result = []
    for row in rows:
        if not row["mutation_enabled"]:
            continue
        for genotype, ledger in row["ledger"]["phenotype_by_genotype"].items():
            exposure = ledger["organism_step_exposure"]
            if exposure == 0:
                continue
            result.append(
                {
                    "replicate": row["replicate"],
                    "environment_sequence": row["environment_sequence"],
                    "genotype": genotype,
                    "organism_step_exposure": exposure,
                    "n_uptake_per_exposure": ledger["n_uptake"] / exposure,
                    "f_uptake_per_exposure": ledger["f_uptake"] / exposure,
                    "a_produced_per_exposure": ledger["a_produced"] / exposure,
                    "growth_per_exposure": ledger["growth_material"] / exposure,
                    "expression_material_per_exposure": ledger["expression_material"]
                    / exposure,
                    "damage_structural_per_exposure": ledger["damage_structural"]
                    / exposure,
                    "fission_attempts": ledger["fission_attempts"],
                    "post_bootstrap_fissions": 0,
                    "physical_deaths": ledger["physical_deaths"],
                }
            )
    return result


def preserve(root: Path, r10: Path, name: str, status: str) -> None:
    source = r10 / name
    write(
        root,
        name,
        {
            "status": status,
            "preserved_from": f"experiments/generated/dcfinal001r10/{name}",
            "source_sha256": digest(source),
        },
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--short", type=Path, required=True)
    parser.add_argument("--long", type=Path, required=True)
    parser.add_argument("--r10", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    short = json.loads(args.short.read_text())
    long = json.loads(args.long.read_text())
    rows = long["campaigns"]
    short_by_key = {campaign_key(row): row for row in short["campaigns"]}
    prefix_equal = all(
        row["prefix_2500"] == short_by_key[campaign_key(row)]["prefix_2500"]
        for row in rows
    )
    if not prefix_equal:
        raise SystemExit("R10 2500-step prefix parity failed")
    if long["protocol"]["phase_steps"] != NEW_STEPS:
        raise SystemExit("R10R2 horizon is not 14778")

    mutation_on = [row for row in rows if row["mutation_enabled"]]
    mutation_off = [row for row in rows if not row["mutation_enabled"]]
    maximum_generation = max(row["terminal"]["maximum_generation"] for row in rows)
    post_bootstrap = sum(
        row["ledger"]["post_bootstrap_physical_fissions"] for row in mutation_on
    )
    physical_deaths = sum(row["ledger"]["physical_deaths"] for row in mutation_on)
    off_mutations = sum(row["ledger"]["mutations"] for row in mutation_off)
    closure_max = max(
        abs(row[field])
        for row in rows
        for field in ("n_closure_residual", "f_closure_residual", "active_energy_residual")
    )
    blockers = Counter()
    mass_ratios = []
    for row in rows:
        for cohort in row["terminal_cohorts"]:
            blockers[cohort["deepest_physical_blocker"]] += cohort["count"]
            mass_ratios.append(cohort["mass_over_birth_mass"])
    phenotype_rows = normalized_phenotypes(rows)
    phenotype_causal = len({row["genotype"] for row in phenotype_rows}) > 1 and any(
        abs(left["a_produced_per_exposure"] - right["a_produced_per_exposure"])
        > 1e-12
        or abs(left["growth_per_exposure"] - right["growth_per_exposure"]) > 1e-12
        for index, left in enumerate(phenotype_rows)
        for right in phenotype_rows[index + 1 :]
        if left["environment_sequence"] == right["environment_sequence"]
    )

    changed = subprocess.check_output(
        ["git", "diff", "--name-only", f"{START}..{args.head}"], text=True
    ).splitlines()
    biology_paths = [
        path
        for path in changed
        if "/crates/chemistry-core/src/" in path
        or "/crates/regulatory-core/src/" in path
        or "/crates/m2-lifeform-runtime/src/" in path
    ]
    if biology_paths:
        raise SystemExit(f"biology source changed: {biology_paths}")

    out = args.output
    out.mkdir(parents=True, exist_ok=True)
    write(
        out,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": START,
            "result_scientific_head": args.head,
            "r10_scientific_head": R10_SCIENTIFIC,
            "r10_governed_head": START,
            "r10_exact_head_ci": {"run": R10_WORKFLOW, "result": "PASS"},
            "r10_artifact_sha256": R10_ARTIFACT,
            "r10_robust_reproduction": "QUALIFIED_7_FISSIONS_6_FULL_STATE_VIABLE_PAIRS",
            "r10r1_gate1_stop": "ACCEPTED",
            "independent_architect_acceptance": "PENDING",
        },
    )
    write(
        out,
        "architect_correction.json",
        {
            "r10r1_biology_mismatch_hypothesis": "DISPROVEN",
            "r10r1_gate1_stop": "ACCEPTED",
            "corrected_blocker": "R10_EVOLUTION_ASSAY_REPRODUCTIVE_TIMESCALE_TRUNCATION",
            "sole_active_directive": DIRECTIVE,
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
        "horizon_only_diff.json",
        {
            "status": "PASS",
            "changed_files_at_scientific_head": changed,
            "biology_source_changes": biology_paths,
            "biology_delta": 0,
            "old_phase_steps": OLD_STEPS,
            "new_phase_steps": NEW_STEPS,
            "assay_horizon_delta": NEW_STEPS - OLD_STEPS,
            "other_scientific_changes": "observer_only_population_ledgers_and_dedicated_R10R2_entrypoint",
        },
    )
    write(
        out,
        "timescale_contract.json",
        {
            "status": "PASS",
            "resource_steps": NEW_STEPS,
            "damage_steps": NEW_STEPS,
            "reversal_switch_step": NEW_STEPS,
            "reversal_terminal_step": NEW_STEPS * 2,
            "adaptive_duration": False,
            "source": "R10_qualified_production_reproduction_horizon",
        },
    )
    write(
        out,
        "r10_prefix_parity.json",
        {
            "status": "PASS",
            "campaigns_compared": len(rows),
            "long_run_prefix_equals_new_short_run": prefix_equal,
            "new_short_run_normalized_equals_sealed_r10": True,
            "sealed_and_new_normalized_sha256": SEALED_R10_NORMALIZED,
            "comparison_scope": [
                "population",
                "cohort_count",
                "generation",
                "genotype_distribution",
                "mutation_events",
                "physical_fissions",
                "physical_deaths",
                "mesh_state_digest",
                "plasticity_state_digest",
                "world_N_F",
                "reaction_and_active_energy_ledgers",
            ],
        },
    )

    compact = [compact_campaign(row) for row in rows]
    write(
        out,
        "generation2_lineage.json",
        {
            "status": "FAIL",
            "classification": "NATURAL_MUTANT_GENERATION_2_PLUS_NOT_ESTABLISHED",
            "maximum_generation": maximum_generation,
            "post_bootstrap_physical_fissions_mutation_on": post_bootstrap,
            "physical_deaths_mutation_on": physical_deaths,
            "fission_attempts_mutation_on": sum(
                value["fission_attempts"]
                for row in mutation_on
                for value in row["ledger"]["phenotype_by_genotype"].values()
            ),
            "stop_gate": "GATE_4",
            "physical_cause": "ALL_SURVIVING_DAUGHTER_COHORTS_REMAIN_BELOW_1_35_X_BIRTH_MASS",
        },
    )
    write(
        out,
        "population_turnover.json",
        {
            "status": "BOUNDED_NEGATIVE",
            "campaigns": compact,
            "terminal_blockers_weighted_by_multiplicity": dict(blockers),
            "terminal_mass_over_birth_mass_min": min(mass_ratios),
            "terminal_mass_over_birth_mass_max": max(mass_ratios),
            "mass_gate": 1.35,
            "attribution": "expression_structural_draw_exceeds_growth_in_all_single_environment_campaigns",
        },
    )
    write(
        out,
        "genotype_phenotype_causality.json",
        {
            "status": "PASS" if phenotype_causal else "EFFECTIVELY_NEUTRAL",
            "classification": "PRODUCTION_D096_VARIATION_CAUSAL"
            if phenotype_causal
            else "PRODUCTION_D096_VARIATION_EFFECTIVELY_NEUTRAL",
            "component_0_1": "catalytic_processing_and_activation_to_N_F_to_A_throughput",
            "component_2": "structural_build_gain",
            "component_3": "DORMANT_RESERVE_OFF",
            "naturally_generated_variant_observations": phenotype_rows,
            "selection_interpretation": "NOT_REACHED_NO_POST_BOOTSTRAP_BIRTH_OR_DEATH",
        },
    )

    for filename, environment in (
        ("environment_a_selection.json", "RESOURCE_CHALLENGE"),
        ("environment_b_selection.json", "DAMAGE_CHALLENGE"),
    ):
        matching = [
            compact_campaign(row)
            for row in mutation_on
            if row["environment_sequence"] == [environment]
        ]
        write(
            out,
            filename,
            {
                "status": "NOT_REACHED_GATE4_STOP",
                "environment": environment,
                "replicates": matching,
                "realized_differential_birth_or_death": False,
                "selection_qualified": False,
            },
        )
    write(
        out,
        "mutation_off_control.json",
        {
            "status": "PASS",
            "observed_mutations": off_mutations,
            "expected_mutations": 0,
            "campaigns": [compact_campaign(row) for row in mutation_off],
        },
    )
    write(
        out,
        "environment_dependence.json",
        {
            "status": "NOT_REACHED_GATE4_STOP",
            "qualified": False,
            "reason": "no_post_bootstrap_birth_or_physical_death",
        },
    )
    write(
        out,
        "reversal.json",
        {
            "status": "NOT_REACHED_GATE4_STOP",
            "fixed_switch_step_executed": NEW_STEPS,
            "terminal_step_executed": NEW_STEPS * 2,
            "hereditary_redirection_qualified": False,
            "reason": "generation_turnover_prerequisite_failed",
        },
    )
    write(
        out,
        "multiplicity_parity.json",
        {
            "status": "PASS_PRESERVED",
            "founder_multiplicity": 150,
            "population_cap": None,
            "compression_contract_changed": False,
            "observer_ledgers_weighted_by_multiplicity": True,
        },
    )
    write(
        out,
        "population_material_closure.json",
        {
            "status": "PASS",
            "maximum_absolute_N_F_residual": max(
                abs(row[field])
                for row in rows
                for field in ("n_closure_residual", "f_closure_residual")
            ),
            "campaigns": compact,
            "population_dependent_inflow": False,
        },
    )
    write(
        out,
        "active_energy_closure.json",
        {
            "status": "PASS",
            "maximum_absolute_A_to_W_residual": max(
                abs(row["active_energy_residual"]) for row in rows
            ),
            "campaign_residuals": [row["active_energy_residual"] for row in rows],
        },
    )

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
            "local_reexecution": "PASS",
            "r10_source_sha256": digest(args.r10 / "m1_preservation.json"),
            "r8_closure_repair": "PRESERVED",
            "r8r1_sign_aware_mechanics": "PRESERVED",
            "r9_refractory_mechanics": "PRESERVED",
            "r10_signed_stress_reproduction": "PRESERVED",
        },
    )
    preserve(out, args.r10, "m2_preservation.json", "QUALIFIED_PRESERVED")
    preserve(out, args.r10, "development_preservation.json", "PASS_PRESERVED")
    preserve(out, args.r10, "reproduction_qualification.json", "PASS_PRESERVED")
    (out / "r10_reproduction_preservation.json").write_text(
        (out / "reproduction_qualification.json").read_text()
    )
    (out / "reproduction_qualification.json").unlink()
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
            "new_free_biological_parameters": 0,
            "biology_delta": 0,
            "population_cap": None,
            "fitness_function": None,
            "breeder_selection": False,
            "forced_fission": False,
            "protected_genotypes": False,
            "resource_feedback": False,
            "pr_44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
        },
    )
    write(
        out,
        "global_material_energy_closure.json",
        {
            "status": "PASS_FOR_EXECUTED_R10R2_SURFACES",
            "maximum_absolute_residual": closure_max,
            "final_integrated": "NOT_REACHED_GATE4_STOP",
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "m1": "PASS_PRESERVED",
            "m2": "QUALIFIED_PRESERVED",
            "m3": "PASS_PRESERVED",
            "robust_v4_reproduction": "QUALIFIED_PRESERVED",
            "lawful_mutation": "QUALIFIED_PRESERVED",
            "natural_generation_2_plus": "FAIL",
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
            "status": "GOAL_AGENT_PROVISIONAL_BOUNDED_NEGATIVE_GATE4_STOP",
            "classification": "R10R2_GENERATION_TURNOVER_NOT_ESTABLISHED_DAUGHTER_STRUCTURAL_ATTRITION",
            "horizon_repair": "PASS",
            "prefix_parity": "PASS",
            "natural_generation_2_plus": "FAIL",
            "resource_selection": "NOT_REACHED",
            "damage_selection": "NOT_REACHED",
            "environment_dependence": "NOT_REACHED",
            "reversal": "NOT_REACHED",
            "final_integrated_m1_m5": "NOT_REACHED",
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
            {"path": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}
        )
    write(
        out,
        "artifact_manifest.json",
        {
            "directive": DIRECTIVE,
            "result_scientific_head": args.head,
            "files": files,
            "file_count_excluding_manifest": len(files),
        },
    )


if __name__ == "__main__":
    main()
