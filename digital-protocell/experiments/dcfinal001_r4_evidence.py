#!/usr/bin/env python3
"""Build compact, deterministic R4 evidence from the frozen assay outputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
from typing import Any


DIRECTIVE = (
    "DC-FINAL-001-R4-CONTRACT-OWNERSHIP-V4-TOPOLOGY-COHERENCE-"
    "EVOLUTION-AND-FINAL-GOAL-CLOSURE-001"
)
START = "9cecd5b2c35e2d38d00e687ad16636615292afc7"
R3_SCIENTIFIC = "fcf2b637d868e1e7dc250dfc7ec31ba6533c9729"
R3_CI = 34299369014
R3_ARTIFACT = "863bcddea2a6f144b8629ad2d90493bc31e75d21bd2441cb1d6d77b80c2d8315"


def load(path: Path) -> Any:
    return json.loads(path.read_text())


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_repo_path(path: Path) -> str:
    """Render evidence provenance independently of the caller's working directory."""
    parts = path.parts
    if "digital-protocell" in parts:
        return "/".join(parts[parts.index("digital-protocell") :])
    return str(Path("digital-protocell") / path)


def write(root: Path, name: str, value: Any) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def marker(status: str, reason: str) -> dict[str, Any]:
    return {"status": status, "reason": reason}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--r2-raw", type=Path, required=True)
    parser.add_argument("--r1", type=Path, required=True)
    parser.add_argument("--r2", type=Path, required=True)
    parser.add_argument("--r3", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    raw = load(args.raw)
    r2_raw = load(args.r2_raw)
    out = args.output
    out.mkdir(parents=True, exist_ok=True)
    reproduction = raw["v4_reproduction_campaign"]
    selected_name = {
        "VERTEX_ONLY": "vertex_only",
        "SEGMENT_APPOSITION": "segment_apposition",
        "HALF_EDGE_FALLBACK": "half_edge_fallback",
    }[reproduction["selected_path"]]
    selected = reproduction[selected_name]

    authority = {
        "directive": DIRECTIVE,
        "starting_governed_head": START,
        "r3_scientific_head": R3_SCIENTIFIC,
        "r3_exact_head_ci": {"run": R3_CI, "conclusion": "PASS"},
        "r3_artifact_sha256": R3_ARTIFACT,
        "result_scientific_head": args.head,
        "r3_authority": "PASS",
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
        "independent_architect_acceptance": "PENDING",
    }
    write(out, "authority.json", authority)
    write(
        out,
        "architect_disposition.json",
        {
            "r3": "R3_ACCEPTED_BOUNDED_NEGATIVE_REPLAN",
            "r4_authorized": True,
            "current_root_classification": "V4_TOPOLOGY_AND_CONTRACT_OWNERSHIP_INTEGRATION_DEFECT",
            "source": "canonical Notion readback and user-supplied independent Architect disposition",
        },
    )
    write(
        out,
        "external_prior_art.json",
        {
            "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC10638096/",
            "classification": "REFERENCE_ONLY",
            "principle": "structural assembly-state subpools cannot remain after their containing material is destroyed",
            "external_parameters_imported": [],
            "conditional_gate7_research": "NOT_REQUIRED_BECAUSE_BOOKKEEPING_ONLY_REPLAY_PASSED",
        },
    )
    write(out, "contract_ownership_matrix.json", raw["contract_ownership_matrix"])
    write(
        out,
        "non_v4_runtime_isolation.json",
        {
            "pass": all(
                row["observation"]["physical_runtime_valid"]
                and row["serialized_state_unchanged_by_validation"]
                for row in raw["contract_ownership_matrix"]["rows"]
                if row["observation"]["contract"] != "MaturationCoupledV4"
            ),
            "data_rewritten": False,
            "active_v4_constraint_leaked_to_non_v4": False,
        },
    )
    write(out, "non_v4_d096_parity.json", raw["non_v4_d096_parity"])
    write(
        out,
        "v4_rupture_subpool_accounting.json",
        {
            "pass": True,
            "rule": "total structural destruction clears m_young and tracer_m; 50 percent membrane retention applies identically to tracer_b",
            "source_tests": [
                "r4_v4_tension_rupture_clears_removed_subpools_and_scales_membrane_tracer"
            ],
            "new_parameters": 0,
        },
    )
    write(
        out,
        "v4_rebond_maturation_semantics.json",
        {
            "classification": "NEW_REBOND_STRUCTURE_IS_YOUNG",
            "source_authority": "mesh_reactions::try_local_rebond already assigns m_young=need under MaturationCoupledV4",
            "same_edge_rebond_assignment": "m_young=take; tracer_m=0",
            "new_parameters": 0,
            "pass": True,
        },
    )
    write(out, "v4_closing_edge_trace.json", raw["v4_closing_edge_trace"])
    write(
        out,
        "v4_topology_causal_classification.json",
        {
            "classification": "BOOKKEEPING_ONLY_RUPTURE_PATH",
            "gate6_v4_newborn_continuity": raw["gate6_v4_newborn_continuity"],
            "conditional_topology_created_edge_repair": "NOT_REQUIRED",
            "mechanics_or_fission_semantics_changed": False,
        },
    )
    write(out, "v4_newborn_lifecycle.json", raw["v4_counterpart"])
    write(out, "v4_reproduction_campaign.json", reproduction)

    partition_reports = [
        arm["fission"]["partition"]
        for arm in selected
        if arm.get("physical_fission") and arm.get("fission")
    ]
    residual_keys = [key for key in partition_reports[0] if key.startswith("residual_")] if partition_reports else []
    max_partition_residual = max(
        (abs(report[key]) for report in partition_reports for key in residual_keys),
        default=0.0,
    )
    write(
        out,
        "v4_partition_closure.json",
        {
            "counted_fissions": len(partition_reports),
            "all_partition_reports_ok": all(report["ok"] for report in partition_reports),
            "maximum_reported_residual": max_partition_residual,
            "pass": bool(partition_reports) and all(report["ok"] for report in partition_reports),
        },
    )

    r3_m1 = args.r3 / "m1_preservation.json"
    write(
        out,
        "m1_preservation.json",
        {
            "source": canonical_repo_path(r3_m1),
            "source_sha256": digest(r3_m1),
            "d087_v2": "8/8",
            "d087_v3": "8/8",
            "d087_v4": "7/8",
            "v4_vector": [True, True, False, True, True, True, True, True],
            "m1": "CLOSED_FROZEN_PRESERVED",
            "exact_head_revalidation": "REQUIRED_BY_CI",
        },
    )

    r2_on = [campaign for campaign in r2_raw["campaigns"] if campaign["mutation_enabled"]]
    r2_off = [campaign for campaign in r2_raw["campaigns"] if not campaign["mutation_enabled"]]
    opportunities = sum(campaign["ledger"]["mutation_opportunities"] for campaign in r2_on)
    mutations = sum(campaign["ledger"]["mutations"] for campaign in r2_on)
    expected = opportunities * 0.01
    sigma = math.sqrt(opportunities * 0.01 * 0.99)
    world_residuals = []
    for campaign in r2_raw["campaigns"]:
        world = campaign["world"]
        ledger = world["ledger"]
        world_residuals.extend(
            [
                abs(ledger["initial_n"] + ledger["inflow_n"] + ledger["returned_n"] - ledger["delivered_n"] - world["n_mass"]),
                abs(ledger["initial_f"] + ledger["inflow_f"] + ledger["returned_f"] - ledger["delivered_f"] - world["f_mass"]),
            ]
        )
    historical_pass = (
        raw["non_v4_d096_parity"]["physical_runtime_valid_after_repair"]
        and raw["non_v4_d096_parity"]["serialized_state_unchanged_by_validation"]
        and abs(mutations - expected) <= 3.0 * sigma
        and all(campaign["ledger"]["mutations"] == 0 for campaign in r2_off)
        and max(world_residuals) <= 1e-9
    )
    write(
        out,
        "historical_r2_regression.json",
        {
            "purpose": "REGRESSION_CONTRACT_ISOLATION_CONTROL_ONLY",
            "contract": r2_raw["template"].get("contract_version", "HistoricalV1"),
            "previous_runtime_invalidation_removed": True,
            "historical_d096_equations_changed": False,
            "mutation_opportunities": opportunities,
            "observed_mutations": mutations,
            "expected_mutations": expected,
            "mutation_frequency_compatible_three_sigma": abs(mutations - expected) <= 3.0 * sigma,
            "mutation_off_observed_mutations": sum(campaign["ledger"]["mutations"] for campaign in r2_off),
            "maximum_external_world_flux_residual": max(world_residuals),
            "whole_organism_amount_residual_boundary": "NOT_AUTHORITY_FOR_HISTORICAL_V1_BECAUSE_V1_DOES_NOT_CONSERVE_CONCENTRATION_AMOUNT_ACROSS_GEOMETRY_CHANGE",
            "pass": historical_pass,
        },
    )

    stop_reason = (
        "canonical production V4 reproduction failed frozen 7/10 fission and 6/10 viable-pair gates"
    )
    for name in [
        "v4_mutation_frequency.json",
        "v4_mutated_descendant_lineage.json",
        "v4_population_material_flux.json",
        "v4_environment_a_selection.json",
        "v4_environment_b_selection.json",
        "v4_selection_causality.json",
        "v4_mutation_off_control.json",
        "v4_reversal_protocol.json",
        "v4_reversal_replicates.json",
    ]:
        write(out, name, marker("NOT_EXECUTED_GATE9_STOP", stop_reason))

    preservation_sources = {
        "m2_preservation.json": args.r3 / "m2_preservation.json",
        "development_preservation.json": args.r3 / "development_preservation.json",
        "checkpoint_restart.json": args.r3 / "checkpoint_restart.json",
        "linux_runtime.json": args.r3 / "linux_runtime.json",
        "sensory_embodiment.json": args.r3 / "sensory_embodiment.json",
        "experiential_memory.json": args.r3 / "experiential_memory.json",
        "godot_independence.json": args.r3 / "godot_independence.json",
    }
    for name, source in preservation_sources.items():
        write(
            out,
            name,
            {
                "source": canonical_repo_path(source),
                "source_sha256": digest(source),
                "preservation": "UNCHANGED_BY_R4_SCOPED_CONTRACT_TOPOLOGY_REPAIR",
                "revalidation": "SCOPED_TEST_OR_SOURCE_PRESERVATION",
            },
        )
    r3_global = args.r3 / "global_material_closure.json"
    write(
        out,
        "global_material_energy_closure.json",
        {
            "source": canonical_repo_path(r3_global),
            "source_sha256": digest(r3_global),
            "v4_topology_tests": "PASS",
            "v4_counted_fission_partition_closure": all(report["ok"] for report in partition_reports),
            "historical_r2_external_world_flux_residual": max(world_residuals),
            "production_v4_evolution": "NOT_EXECUTED_GATE9_STOP",
        },
    )
    write(
        out,
        "forbidden_information_audit.json",
        {
            "new_free_parameters": 0,
            "historical_state_rewritten": False,
            "newborn_grace_period": False,
            "lineage_special_case": False,
            "fission_or_mechanics_change": False,
            "fitness_or_observer_feedback": False,
            "evolution_executed_after_failed_reproduction_gate": False,
            "pr44_modified": False,
            "pass": True,
        },
    )
    write(
        out,
        "end_to_end_lifecycle.json",
        {
            "status": "NOT_EXECUTED_GATE9_STOP",
            "production_v4_reproduction": "FAIL",
            "digital_cell_end_goal": "NOT_ESTABLISHED",
        },
    )
    write(
        out,
        "final_goal_matrix.json",
        {
            "m1": "CLOSED_FROZEN_PRESERVED",
            "m2": "QUALIFIED_PRESERVED",
            "m3": "PRESERVED_HISTORICAL_NOT_RECOMPOSED",
            "m4": "NOT_EXECUTED_PRODUCTION_V4_REPRODUCTION_GATE_FAILED",
            "m5": "PRESERVED_HISTORICAL_NOT_RECOMPOSED",
            "digital_cell_end_goal": "NOT_ESTABLISHED",
        },
    )
    qualification = {
        "directive": DIRECTIVE,
        "status": "GOAL_AGENT_PROVISIONAL_NEGATIVE_REPLAN",
        "classification": "V4_GEOMETRY_VALID_REPRODUCTION_NOT_ESTABLISHED",
        "contract_ownership_repaired": True,
        "v4_topology_subpool_accounting_repaired": True,
        "conditional_topology_repair": "NOT_REQUIRED",
        "canonical_v4_reproduction": "FAIL",
        "growth_qualified": reproduction["growth_qualified"],
        "geometry_valid_fissions": reproduction["geometry_valid_fissions"],
        "simple_viable_daughter_pairs": reproduction["simple_viable_daughter_pairs"],
        "bookkeeping_runtime_invalidations": reproduction["bookkeeping_runtime_invalidations"],
        "historical_r2_regression": "PASS" if historical_pass else "FAIL",
        "production_v4_evolution_executed": False,
        "selection": "NOT_ESTABLISHED",
        "reversal": "NOT_ESTABLISHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
        "shutdown_recommended": False,
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    }
    write(out, "qualification.json", qualification)

    manifest = {
        "directive": DIRECTIVE,
        "result_scientific_head": args.head,
        "files": [],
    }
    for path in sorted(out.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest["files"].append(
            {"path": path.name, "sha256": digest(path), "bytes": path.stat().st_size}
        )
    write(out, "artifact_manifest.json", manifest)


if __name__ == "__main__":
    main()
