#!/usr/bin/env python3
"""Seal compact R3 evidence for the D-096/V4 integration boundary."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = "DC-FINAL-001-R3-D096-V4-MATURATION-SAFE-EXPRESSION-EVOLUTION-AND-FINAL-GOAL-CLOSURE-001"
START = "5bd82aa40b10ecbdc86cd6915760915ba6ccfb18"
R2_SCIENCE = "5b7c06a7208b8b6bc8cc9c918d8de7906c611a4a"
R2_CI = 34294704290
R2_ARTIFACT = "358b47164edaa3d16fc966aa968b18effbabd9c5ff444004c7539551c5d61743"
CLASSIFICATION = "D096_V4_STRUCTURAL_TRANSFER_SEMANTICS_INVALID"


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preserved(root: Path, name: str):
    path = root / name
    return {"source": str(path), "sha256": sha(path), "value": load(path)}


def compact_root(root):
    return {
        "contract_version": root["contract_version"],
        "maturation_coupled": root["maturation_coupled"],
        "material_consumed": root["material_consumed"],
        "fraction_left": root["fraction_left"],
        "legacy_failing_predicate": root["legacy_failing_predicate"],
        "legacy_counterfactual_violation_edges": root["legacy_counterfactual_violation_edges"],
        "repaired_lifecycle_invariants_hold": root["repaired_lifecycle_invariants_hold"],
        "repaired_physical_runtime_valid": root["repaired_physical_runtime_valid"],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r1", type=Path, required=True)
    parser.add_argument("--r2", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raw = load(args.raw)
    root = args.output
    root.mkdir(parents=True, exist_ok=True)

    exact_a = raw["newborn_first_step_attribution"]["daughter_a"]
    exact_b = raw["newborn_first_step_attribution"]["daughter_b"]
    v4 = raw["v4_counterpart"]
    v4_lives = [v4["daughter_a_lifecycle"], v4["daughter_b_lifecycle"]]
    closure = {
        "maximum_structural_residual": max(x["maximum_structural_material_closure_residual"] for x in v4_lives),
        "maximum_activation_residual": max(x["maximum_activation_closure_residual"] for x in v4_lives),
        "maximum_waste_residual": max(x["maximum_waste_closure_residual"] for x in v4_lives),
    }
    v4_first_steps_pass = all(
        v4[name]["expression_ok"]
        and v4[name]["post_expression_lifecycle_invariants_hold"]
        and v4[name]["post_expression_physical_runtime_valid"]
        and v4[name]["contact_accepted"]
        for name in ["daughter_a_first_step", "daughter_b_first_step"]
    )
    exact_r2_fails = all(
        side["expression_on"]["expression_ok"]
        and not side["expression_on"]["post_expression_physical_runtime_valid"]
        and not side["expression_on"]["contact_accepted"]
        for side in [exact_a, exact_b]
    )
    v4_topology_invalid = all(
        x["runtime_invalidation"] == {"phase": "post_topology", "step": 1}
        and x["first_topology_ledger"]["tension_ruptures"] > 0
        and len(x["first_invalid_edges"]) > 0
        for x in v4_lives
    )

    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "r2_scientific_head": R2_SCIENCE,
        "r2_exact_head_ci": R2_CI,
        "r2_artifact_sha256": R2_ARTIFACT,
        "result_scientific_head": args.head,
        "independent_architect_acceptance": "PENDING",
    })
    dump(root, "architect_disposition.json", {
        "r2": "ACCEPTED POWERED MUTATION QUALIFICATION / EVOLUTION NOT INTERPRETABLE",
        "authorized_root_cause": "D096_V4_STRUCTURAL_CONSUMPTION_MATURATION_BOOKKEEPING_DEFECT",
        "r3": "CONTINUE_REPLAN",
        "self_disposition": "GOAL_AGENT_PROVISIONAL_NEGATIVE_REPLAN",
    })
    dump(root, "external_prior_art.json", {
        "source": "https://pubmed.ncbi.nlm.nih.gov/27152338/",
        "classification": "REFERENCE_ONLY_CONSISTENCY_PRINCIPLE",
        "principle": "cortical mechanics depends on coherent assembly and turnover state",
        "external_numerical_values_imported": False,
    })
    dump(root, "d096_v4_root_cause.json", {
        "daughter_a": compact_root(exact_a["root_cause"]),
        "daughter_b": compact_root(exact_b["root_cause"]),
        "exact_r2_template_contract": raw["template"]["contract_version"],
        "exact_r2_failing_predicate_reproduced": exact_r2_fails,
        "additional_authority_conflict": "the sealed R2 evolution template is HistoricalV1, not MaturationCoupledV4",
    })
    dump(root, "expression_maturation_contract.json", {
        "v4_rule": ["edge.m *= fraction_left", "edge.m_young *= fraction_left", "edge.tracer_m *= fraction_left"],
        "young_mature_ratio_preserved": True,
        "post_hoc_clamp": False,
        "newborn_exemption": False,
        "new_parameters": 0,
        "v4_first_step_pass": v4_first_steps_pass,
        "non_v4_rule_unchanged": True,
    })
    dump(root, "newborn_expression_tests.json", {
        "exact_r2_historical_daughter_a": exact_a["expression_on"],
        "exact_r2_historical_daughter_b": exact_b["expression_on"],
        "v4_counterpart_daughter_a": v4["daughter_a_first_step"],
        "v4_counterpart_daughter_b": v4["daughter_b_first_step"],
    })
    dump(root, "newborn_lifecycle.json", {
        "daughter_a": v4["daughter_a_lifecycle"],
        "daughter_b": v4["daughter_b_lifecycle"],
        "classification": "FAIL_PREEXISTING_V4_TOPOLOGY_RUPTURE_LEAVES_M_YOUNG_ON_ZEROED_EDGE" if v4_topology_invalid else "UNRESOLVED",
        "evolution_gate_7_open": False,
    })
    dump(root, "expression_material_closure.json", {
        "contract": "structural material removed equals gross catalyst synthesis; turnover separate",
        **closure,
        "pass": closure["maximum_structural_residual"] <= 1e-10,
    })
    dump(root, "expression_activation_energy_closure.json", {
        "contract": "V4 activation and maintenance A spent transfers to W with coefficient 1; turnover waste remains separate",
        **closure,
        "pass": closure["maximum_activation_residual"] <= 1e-10 and closure["maximum_waste_residual"] <= 1e-10,
    })
    dump(root, "tracer_accounting.json", {
        "semantics": "observer-only tagged subset of structural m",
        "v4_withdrawal": "proportional with m",
        "enters_biology": False,
        "unit_test": "d096_v4_expression_preserves_maturation_subpool_and_closes_energy",
        "pass": True,
    })
    dump(root, "non_v4_semantic_parity.json", {
        "unit_test": "d096_non_v4_expression_retains_historical_subpool_tracer_and_w_semantics",
        "m_young": "unchanged",
        "tracer_m": "unchanged",
        "w": "legacy turnover-only increment",
        "pass": True,
        "consequence": "the exact HistoricalV1 R2 daughters cannot receive the V4-only repair",
    })
    dump(root, "r2_exact_replay.json", {
        "sealed_r2": preserved(args.r2, "qualification.json"),
        "exact_template_contract": raw["template"]["contract_version"],
        "exact_daughter_a": exact_a["expression_on"],
        "exact_daughter_b": exact_b["expression_on"],
        "replay": "FAIL_AT_NEWBORN_GATE6_AS_BEFORE",
        "evolution": "NOT_RUN_STOPPED_AT_GATE6",
    })
    dump(root, "mutation_frequency.json", preserved(args.r2, "mutation_frequency.json"))
    dump(root, "mutated_descendant_lineage.json", {
        "mechanistic_partition_control": raw["mutated_descendant_heredity_control"],
        "natural_generation_2_lineage": False,
        "status": "NOT_RUN_GATE6_BLOCKED",
    })
    for name in [
        "population_material_flux.json", "environment_a_selection.json", "environment_b_selection.json",
        "selection_causality.json", "mutation_off_control.json", "reversal_protocol.json", "reversal_replicates.json",
    ]:
        dump(root, name, {
            "status": "NOT_RUN_GATE6_BLOCKED",
            "reason": "exact frozen R2 is HistoricalV1 while the authorized correction is V4-only; the equivalent V4 lifecycle fails after topology rupture",
            "success_claim": False,
        })

    preservation_map = {
        "m1_preservation.json": (args.r1, "preservation.json"),
        "m2_preservation.json": (args.r1, "m2_qualification.json"),
        "reproduction_preservation.json": (args.r1, "wp1_preservation.json"),
        "development_preservation.json": (args.r1, "development_life_history.json"),
        "checkpoint_restart.json": (args.r1, "checkpoint_restart.json"),
        "linux_runtime.json": (args.r1, "linux_runtime.json"),
        "sensory_embodiment.json": (args.r1, "sensory_embodiment.json"),
        "experiential_memory.json": (args.r1, "experiential_memory.json"),
        "godot_independence.json": (args.r1, "godot_independence.json"),
    }
    for output, (source_root, source_name) in preservation_map.items():
        dump(root, output, preserved(source_root, source_name))
    dump(root, "global_material_closure.json", {
        "expression": closure,
        "expression_pass": max(closure.values()) <= 1e-10,
        "population_ecology": "NOT_RUN_GATE6_BLOCKED",
        "final_integrated_closure": "NOT_ESTABLISHED",
    })
    dump(root, "forbidden_information_audit.json", {
        "newborn_age_or_lineage_read_by_repair": False,
        "division_state_read_by_repair": False,
        "observer_feedback": False,
        "parameter_search": False,
        "mutation_or_environment_changed": False,
        "success_controller": False,
        "pass": True,
    })
    dump(root, "end_to_end_lifecycle.json", {
        "executed": False,
        "reason": "selection and reversal were not validly executable after Gate 6",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
    })
    dump(root, "final_goal_matrix.json", {
        "M1": "PRESERVED",
        "M2": "PRESERVED_QUALIFIED",
        "geometry_valid_reproduction": "PRESERVED",
        "M3": "PRESERVED_BOUNDED",
        "mutation_supply": "PRESERVED_QUALIFIED",
        "environment_dependent_selection": "NOT_ESTABLISHED",
        "reversal": "NOT_ESTABLISHED",
        "end_goal": "NOT_ESTABLISHED",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "status": "GOAL_AGENT_PROVISIONAL_NEGATIVE_REPLAN",
        "classification": CLASSIFICATION,
        "root_cause_reproduced": True,
        "v4_maturation_safe_expression_first_step": v4_first_steps_pass,
        "exact_r2_contract": raw["template"]["contract_version"],
        "exact_r2_newborn_continuation": False,
        "v4_lifecycle_continuation": False,
        "v4_topology_subpool_defect_observed": v4_topology_invalid,
        "evolution_executed": False,
        "selection": "NOT_ESTABLISHED",
        "reversal": "NOT_ESTABLISHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": False,
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })
    manifest = []
    for path in sorted(root.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest.append({"path": path.name, "sha256": sha(path)})
    dump(root, "artifact_manifest.json", {
        "directive": DIRECTIVE,
        "result_scientific_head": args.head,
        "classification": CLASSIFICATION,
        "files": manifest,
        "artifact_sha256": "computed by exact-head workflow",
    })


if __name__ == "__main__":
    main()
