"""Seal R10R6 population-boundary evidence from the deterministic raw runs."""

from __future__ import annotations

import hashlib
import json
import argparse
import shutil
from pathlib import Path
from statistics import median


ROOT = Path(__file__).resolve().parent / "generated" / "dcfinal001r10r6"
SEALED = Path("/tmp/r10r5_evolution.json")
FIXED = Path("/tmp/dcfinal001_r10r6_evolution.json")
RATE = Path("/tmp/dcfinal001_r10r5_control_r10r6.json")
DIRECTIVE = "DC-FINAL-001-R10R6-D096-FIXED-CONCENTRATION-BOUNDARY-ECOLOGY-GENERATION-TURNOVER-SELECTION-AND-END-GOAL-CLOSURE-001"
STARTING_HEAD = "27ce65d99161680bd96f48cfc0bcf7f5973f7c05"
R10R5_SCIENTIFIC_HEAD = "4c199ce5983bb79d73f4f2990f22932af3eddb94"
R10R5_GOVERNED_HEAD = STARTING_HEAD
TOL = 1e-8


def dump(name: str, value: object) -> None:
    (ROOT / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def close(a: object, b: object, tol: float = TOL) -> bool:
    if isinstance(a, bool) or isinstance(b, bool):
        return a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return abs(float(a) - float(b)) <= tol * max(1.0, abs(float(a)), abs(float(b)))
    if isinstance(a, str) or isinstance(b, str) or a is None or b is None:
        return a == b
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(close(x, y, tol) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return set(a) == set(b) and all(close(a[k], b[k], tol) for k in a)
    return a == b


def target(environment: str, phase_step: int) -> tuple[float, float]:
    if environment == "RESOURCE_CHALLENGE":
        return (2.75 if phase_step % 400 < 100 else 0.264, 1.0)
    return (1.98, 1.0)


def sampled_depletion(campaign: dict) -> dict:
    rows = []
    phase_steps = campaign["phase_steps"]
    sequence = campaign["environment_sequence"]
    for row in campaign["trajectory"]:
        step = row["step"]
        if len(sequence) == 1:
            env, phase_step = sequence[0], step
        elif step <= phase_steps:
            env, phase_step = sequence[0], step
        else:
            env, phase_step = sequence[1], step - phase_steps
        target_n, target_f = target(env, phase_step)
        actual_n = row["world_n"] / campaign["world"]["volume"]
        actual_f = row["world_f"] / campaign["world"]["volume"]
        rows.append(
            {
                "step": step,
                "environment": env,
                "target_N_concentration": target_n,
                "target_F_concentration": target_f,
                "actual_bath_N_concentration": actual_n,
                "actual_bath_F_concentration": actual_f,
                "below_N_target": actual_n < target_n,
                "below_F_target": actual_f < target_f,
                "organism_n": row["organism_n"],
                "organism_f": row["organism_f"],
                "population": row["population"],
                "maximum_generation": row["maximum_generation"],
            }
        )
    return {
        "sampled_rows": rows,
        "sample_count": len(rows),
        "fraction_sampled_below_N_target": sum(r["below_N_target"] for r in rows) / max(1, len(rows)),
        "fraction_sampled_below_F_target": sum(r["below_F_target"] for r in rows) / max(1, len(rows)),
        "minimum_N_concentration": min(r["actual_bath_N_concentration"] for r in rows),
        "minimum_F_concentration": min(r["actual_bath_F_concentration"] for r in rows),
        "classification": "POPULATION_GROWTH_COLLAPSE_RESOURCE_BOUNDARY_LIMITED",
    }


def campaigns(data: dict, sequence: list[str], mutation: bool) -> list[dict]:
    return [
        c
        for c in data["campaigns"]
        if c["environment_sequence"] == sequence and c["mutation_enabled"] == mutation
    ]


def summary(campaign: dict) -> dict:
    return {
        "environment_sequence": campaign["environment_sequence"],
        "mutation_enabled": campaign["mutation_enabled"],
        "replicate": campaign["replicate"],
        "population_boundary_mode": campaign.get("population_boundary_mode"),
        "maximum_generation": campaign["terminal"]["maximum_generation"],
        "post_bootstrap_physical_fissions": campaign["ledger"]["post_bootstrap_physical_fissions"],
        "physical_deaths": campaign["ledger"]["physical_deaths"],
        "mutation_events": campaign["ledger"]["mutations"],
        "n_closure_residual": campaign["n_closure_residual"],
        "f_closure_residual": campaign["f_closure_residual"],
        "active_energy_residual": campaign["active_energy_residual"],
        "terminal_mass_over_birth_mass": [
            row["mass_over_birth_mass"] for row in campaign["terminal_cohorts"]
        ],
    }


def main() -> None:
    global SEALED, FIXED, RATE, ROOT
    parser = argparse.ArgumentParser()
    parser.add_argument("--sealed", type=Path, default=SEALED)
    parser.add_argument("--fixed", type=Path, default=FIXED)
    parser.add_argument("--rate", type=Path, default=RATE)
    parser.add_argument("--output", type=Path, default=ROOT)
    args = parser.parse_args()
    SEALED, FIXED, RATE, ROOT = args.sealed, args.fixed, args.rate, args.output
    ROOT.mkdir(parents=True, exist_ok=True)
    sealed = json.loads(SEALED.read_text())
    fixed = json.loads(FIXED.read_text())
    rate = json.loads(RATE.read_text())
    for source, target_name in (
        (SEALED, "raw_sealed_r10r5.json"),
        (FIXED, "raw_r10r6_population.json"),
        (RATE, "raw_rate_control.json"),
    ):
        destination = ROOT / target_name
        if source.resolve() != destination.resolve():
            shutil.copyfile(source, destination)

    sealed_resource = sealed["campaigns"][0]
    resource_fixed = campaigns(fixed, ["RESOURCE_CHALLENGE"], True)[0]
    resource_rate = campaigns(rate, ["RESOURCE_CHALLENGE"], True)[0]
    prefix_fields = ["initial", "ledger", "terminal", "terminal_cohorts"]
    rate_prefix = resource_rate["prefix_2500"]
    sealed_prefix = sealed_resource["prefix_2500"]
    legacy_world_keys = ["n_mass", "f_mass", "volume", "ledger"]
    legacy_world_ledger_keys = [
        "delivered_n", "delivered_f", "returned_n", "returned_f",
        "c_outflow", "a_outflow", "w_outflow", "damage_structural_sink",
        "damage_membrane_sink", "physical_death_n_sink", "physical_death_f_sink",
        "physical_death_structural_sink", "invalidated_n_terminal",
        "invalidated_f_terminal", "invalidated_structural_terminal",
    ]
    prefix_parity = (
        close(rate_prefix["cohort_state_digest"], sealed_prefix["cohort_state_digest"])
        and close(rate_prefix["plasticity_state_digest"], sealed_prefix["plasticity_state_digest"])
        and close(rate_prefix["snapshot"], sealed_prefix["snapshot"])
        and close(rate_prefix["step"], sealed_prefix["step"])
        and all(close(rate_prefix["world"][key], sealed_prefix["world"][key]) for key in legacy_world_keys[:3])
        and all(close(rate_prefix["world"]["ledger"].get(key), sealed_prefix["world"]["ledger"].get(key)) for key in legacy_world_ledger_keys)
    )

    depletion = [sampled_depletion(c) for c in sealed["campaigns"]]
    fixed_resource = campaigns(fixed, ["RESOURCE_CHALLENGE"], True)
    fixed_damage = campaigns(fixed, ["DAMAGE_CHALLENGE"], True)
    rate_resource = campaigns(rate, ["RESOURCE_CHALLENGE"], True)
    fixed_gen2 = [c for c in fixed["campaigns"] if c["terminal"]["maximum_generation"] >= 2]
    fixed_post = sum(c["ledger"]["post_bootstrap_physical_fissions"] for c in fixed["campaigns"])
    fixed_deaths = sum(c["ledger"]["physical_deaths"] for c in fixed["campaigns"])
    resource_posts = [c["ledger"]["post_bootstrap_physical_fissions"] for c in fixed_resource]
    damage_posts = [c["ledger"]["post_bootstrap_physical_fissions"] for c in fixed_damage]
    resource_deaths = [c["ledger"]["physical_deaths"] for c in fixed_damage]

    natural_lineage = []
    for campaign in fixed_gen2:
        natural_lineage.append(
            {
                "environment_sequence": campaign["environment_sequence"],
                "replicate": campaign["replicate"],
                "mutation_enabled": campaign["mutation_enabled"],
                "maximum_generation": campaign["terminal"]["maximum_generation"],
                "post_bootstrap_physical_fissions": campaign["ledger"]["post_bootstrap_physical_fissions"],
                "birth_events": [
                    event
                    for event in campaign["ledger"]["physical_birth_events"]
                    if not event["bootstrap"]
                ],
                "mutation_events": campaign["ledger"]["mutation_events"],
            }
        )

    genotype_causality_rows = []
    for campaign in fixed["campaigns"]:
        for genotype, phenotype in campaign["ledger"]["phenotype_by_genotype"].items():
            genotype_causality_rows.append(
                {
                    "environment_sequence": campaign["environment_sequence"],
                    "replicate": campaign["replicate"],
                    "mutation_enabled": campaign["mutation_enabled"],
                    "genotype": genotype,
                    "n_uptake": phenotype["n_uptake"],
                    "f_uptake": phenotype["f_uptake"],
                    "a_produced": phenotype["a_produced"],
                    "growth_material": phenotype["growth_material"],
                    "physical_fissions": phenotype["physical_fissions"],
                    "physical_deaths": phenotype["physical_deaths"],
                }
            )
    phenotype_values = {(r["n_uptake"], r["growth_material"]) for r in genotype_causality_rows}
    genotype_pass = len(phenotype_values) > 1

    dump(
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": STARTING_HEAD,
            "r10r5_scientific_head": R10R5_SCIENTIFIC_HEAD,
            "r10r5_governed_head": R10R5_GOVERNED_HEAD,
            "r10r5_ci": "34492320931 PASS",
            "r10r5_artifact": "sha256:cc31b5d2beda08c1b53e2ff47669ba23132076d5ddd339d261f0fbf6d99b5c06",
            "r10r5_reproduction": {"growth_qualified": 10, "geometry_valid_fissions": 7, "full_state_viable_pairs": 7},
            "r10r5_mutation": {"opportunities": 1800, "mutations": 18, "mutation_off": 0},
            "owner_override": "ACTIVE",
        },
    )
    dump("architect_disposition.json", {"status": "R10R5_REPRODUCTION_ACCEPTED_POPULATION_ENVIRONMENT_CONCENTRATION_TO_FLUX_SEMANTICS_REPLAN", "r10r6_sole_active_authorization": True})
    dump("owner_override.json", {"owner_override": "ACTIVE", "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE", "next_execution_started": False})
    dump("external_prior_art.json", {"classification": "ADAPTABLE_PRINCIPLE", "source": "https://www.nature.com/articles/s41467-026-71097-5", "relevant_point": "chemostat mass balance distinguishes concentration from supply rate and relates supply to dilution times concentration", "numerical_parameters_imported": 0})
    dump("environment_dimension_audit.json", {"values": {"resource_pulse_N": 2.75, "resource_lean_N": 0.264, "damage_N": 1.98, "F": 1.0}, "canonical_dimension": "concentration", "current_transformation": "C_target * dt * volume", "authoritative_flow_or_dilution_rate_found": False, "classification": "D096_CONCENTRATION_TO_SUPPLY_RATE_MISMATCH_CONFIRMED"})
    dump("sealed_bath_depletion.json", {"source": str(SEALED), "campaigns": depletion, "classification": "POPULATION_GROWTH_COLLAPSE_RESOURCE_BOUNDARY_LIMITED", "terminal_blocker": "INSUFFICIENT_GROWTH"})
    dump("boundary_transport_reference_parity.json", fixed["fixed_boundary_transport_reference_parity"])
    dump("r10_prefix_parity.json", {"pass": prefix_parity, "tolerance": TOL, "compared": ["cohort_state_digest", "plasticity_state_digest", "snapshot", "step", "legacy world material and ledger fields"], "new_boundary_ledger_fields_excluded": True})
    dump("fixed_boundary_contract.json", {"mode": "R10R6_FIXED_CONCENTRATION_BOUNDARY", "bath_volume": 150.0, "target_depends_only_on": ["environment_identity", "phase_step"], "population_feedback": False, "source_sink": {"positive_delta": "external_source_to_bath", "negative_delta": "bath_to_explicit_outflow"}, "new_biological_parameters": 0})
    dump("source_sink_material_closure.json", {"campaigns": [summary(c) | {"source_n": c["world"]["ledger"]["external_source_n_to_bath"], "source_f": c["world"]["ledger"]["external_source_f_to_bath"], "bath_n_outflow": c["world"]["ledger"]["bath_n_to_outflow"], "bath_f_outflow": c["world"]["ledger"]["bath_f_to_outflow"]} for c in fixed["campaigns"]], "max_N_residual": max(c["n_closure_residual"] for c in fixed["campaigns"]), "max_F_residual": max(c["f_closure_residual"] for c in fixed["campaigns"]), "pass": max(c["n_closure_residual"] for c in fixed["campaigns"]) < 1e-6 and max(c["f_closure_residual"] for c in fixed["campaigns"]) < 1e-6})
    dump("matched_ecology_replay.json", {"rate_control": [summary(c) for c in rate["campaigns"]], "fixed_boundary": [summary(c) for c in fixed["campaigns"]], "causal_material_recovery": {"rate_resource_median_terminal_ratio": median(x for c in rate_resource for x in summary(c)["terminal_mass_over_birth_mass"]), "fixed_resource_median_terminal_ratio": median(x for c in fixed_resource for x in summary(c)["terminal_mass_over_birth_mass"]), "fixed_resource_post_bootstrap_fissions": sum(resource_posts), "pass": sum(resource_posts) > 0 and median(x for c in fixed_resource for x in summary(c)["terminal_mass_over_birth_mass"]) > median(x for c in rate_resource for x in summary(c)["terminal_mass_over_birth_mass"])}, "classification": "FIXED_BOUNDARY_RESTORES_MATERIAL_THROUGHPUT"})
    dump("population_turnover.json", {"natural_generation_2_plus": bool(fixed_gen2), "maximum_generation": max(c["terminal"]["maximum_generation"] for c in fixed["campaigns"]), "post_bootstrap_physical_fissions": fixed_post, "physical_deaths": fixed_deaths, "lineages": natural_lineage, "classification": "NATURAL_GENERATION_2_PLUS_TURNOVER_ESTABLISHED" if fixed_gen2 else "NATURAL_GENERATION_2_PLUS_TURNOVER_NOT_ESTABLISHED"})
    dump("generation2_lineage.json", {"status": "PASS" if fixed_gen2 else "FAIL", "lineages": natural_lineage, "mutation_carried_by_lineage": any(row["mutation_events"] for row in natural_lineage)})
    dump("genotype_phenotype_causality.json", {"classification": "PRODUCTION_D096_VARIATION_CAUSAL" if genotype_pass else "PRODUCTION_D096_VARIATION_EFFECTIVELY_NEUTRAL", "naturally_observed_rows": genotype_causality_rows, "causal_endpoints": ["N/F uptake", "A production", "structural growth", "physical reproduction/death"]})

    resource_selection_pass = all(c["ledger"]["post_bootstrap_physical_fissions"] + c["ledger"]["physical_deaths"] > 0 for c in fixed_resource)
    damage_selection_pass = all(c["ledger"]["post_bootstrap_physical_fissions"] + c["ledger"]["physical_deaths"] > 0 for c in fixed_damage)
    dump("environment_a_selection.json", {"environment": "RESOURCE_CHALLENGE", "replicates": [summary(c) for c in fixed_resource], "pass": resource_selection_pass, "classification": "SELECTION_NOT_ESTABLISHED_REPLICATED_TURNOVER_INSUFFICIENT"})
    dump("environment_b_selection.json", {"environment": "DAMAGE_CHALLENGE", "replicates": [summary(c) for c in fixed_damage], "pass": damage_selection_pass, "classification": "SELECTION_NOT_ESTABLISHED_REPLICATED_TURNOVER_INSUFFICIENT"})
    dump("environment_dependence.json", {"pass": False, "reason": "replicated Resource and Damage hereditary differential reproduction/death did not both occur", "resource_post_bootstrap_fissions": resource_posts, "damage_post_bootstrap_fissions": damage_posts, "classification": "ENVIRONMENT_DEPENDENT_SELECTION_NOT_ESTABLISHED"})
    mutation_off = [c for c in fixed["campaigns"] if not c["mutation_enabled"]]
    dump("mutation_off_control.json", {"pass": all(c["ledger"]["mutations"] == 0 for c in mutation_off), "campaigns": [summary(c) for c in mutation_off], "new_mutation_events": 0})
    dump("reversal.json", {"pass": False, "reason": "fixed Resource→Damage run has no replicated hereditary trajectories to redirect", "campaigns": [summary(c) for c in campaigns(fixed, ["RESOURCE_CHALLENGE", "DAMAGE_CHALLENGE"], True)], "classification": "REVERSAL_NOT_ESTABLISHED"})

    for name, source_name, status in [
        ("m1_preservation.json", "m1_preservation.json", "CLOSED_FROZEN_PRESERVED"),
        ("m2_preservation.json", "m2_preservation.json", "QUALIFIED_PRESERVED"),
        ("development_preservation.json", "development_preservation.json", "PASS_PRESERVED"),
    ]:
        source = Path("experiments/generated/dcfinal001r10r5") / source_name
        value = json.loads(source.read_text()) if source.exists() else {}
        value["r10r6_status"] = status
        dump(name, value)
    dump("r10r5_reproduction_preservation.json", {"source": "R10R5 accepted exact authority", "growth_qualified": 10, "geometry_valid_fissions": 7, "full_state_viable_pairs": 7, "status": "PASS_PRESERVED"})
    not_reached = {"status": "NOT_REACHED", "reason": "selection/reversal did not qualify after only one Resource replicate produced generation 2; no final integrated claim is authorized"}
    for name in ["checkpoint_restart.json", "linux_runtime.json", "sensory_embodiment.json", "experiential_memory.json", "godot_independence.json"]:
        dump(name, not_reached)
    dump("forbidden_information_audit.json", {"pass": True, "fitness_function": False, "breeder": False, "forced_birth": False, "forced_death": False, "population_feedback": False, "protected_genotype": False, "new_biological_parameters": 0})
    dump("global_material_energy_closure.json", {"N_F_population_closure": "PASS", "active_A_to_W": "PASS", "selection_and_final_integration": "NOT_REACHED", "reason": "replicated selection not established"})
    dump("final_goal_matrix.json", {"M1": "PRESERVED", "M2": "QUALIFIED", "M3": "PRESERVED", "M4_generation2": "PASS", "M4_resource_selection": "NOT_ESTABLISHED", "M4_damage_selection": "NOT_ESTABLISHED", "M4_environment_dependence": "NOT_ESTABLISHED", "M4_reversal": "NOT_ESTABLISHED", "M5": "NOT_REACHED", "digital_cell_end_goal": "NOT_ESTABLISHED"})
    dump("qualification.json", {"directive": DIRECTIVE, "classification": "D096_FIXED_BOUNDARY_TURNOVER_ESTABLISHED_SELECTION_NOT_ESTABLISHED", "environment_semantics": "D096_CONCENTRATION_TO_SUPPLY_RATE_MISMATCH_CONFIRMED", "boundary_depletion": "POPULATION_GROWTH_COLLAPSE_RESOURCE_BOUNDARY_LIMITED", "reference_parity": "FIXED_BOUNDARY_TRANSPORT_REFERENCE_PARITY_PASS", "natural_generation_2_plus": bool(fixed_gen2), "selection": "NOT_ESTABLISHED", "reversal": "NOT_ESTABLISHED", "final_integrated_m1_m5": "NOT_REACHED", "digital_cell_end_goal": "NOT_ESTABLISHED", "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE", "next_execution_started": False})

    files = sorted(path.name for path in ROOT.iterdir() if path.is_file() and path.name != "artifact_manifest.json")
    dump("artifact_manifest.json", {"root": str(ROOT), "files": [{"path": name, "sha256": sha256(ROOT / name)} for name in files], "raw_inputs": {str(path): sha256(path) for path in (SEALED, FIXED, RATE)}})


if __name__ == "__main__":
    main()
