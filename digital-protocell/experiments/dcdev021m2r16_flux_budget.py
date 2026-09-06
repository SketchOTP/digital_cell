#!/usr/bin/env python3
"""Build the compact R16 observer-only flux-budget evidence package."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


R15_HEAD = "9f1bb36cc81e731490d72e17c666cc8c13c91d44"
R15_CI = "34037256120"
R15_ARTIFACT = "sha256:911d3fbcd1cf6df3fc9f20793e7ccbf9271cc78310021dcb0af515775d7c50f2"
R15_DELIVERY = 292.6923687407118


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def write(root: Path, name: str, value: Any) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def summary(report: dict[str, Any]) -> dict[str, Any]:
    audit = report.get("flux_audit")
    return {
        "runtime": {k: report.get(k) for k in (
            "schema", "step", "seed", "resource_mode", "resource_transfer_enabled",
            "first_contact_step", "first_transfer_step", "first_fission_step",
            "fission_events", "cumulative_n_delivered", "cumulative_f_delivered",
            "living_count", "terminal_observer_death_reasons",
            "world_n_conservation_error", "world_f_conservation_error",
        )},
        "flux_audit": audit,
    }


def checkpoints(report: dict[str, Any]) -> list[dict[str, Any]]:
    return report.get("flux_audit", {}).get("checkpoints", [])


def closest_checkpoint(rows: list[dict[str, Any]], step: int) -> dict[str, Any] | None:
    return min(rows, key=lambda row: abs(row["step"] - step), default=None)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--active", type=Path, required=True)
    parser.add_argument("--disabled", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--sealed-reference", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    active = load(args.active)
    disabled = load(args.disabled)
    reference = load(args.reference)
    sealed_reference = load(args.sealed_reference)
    root = args.output
    root.mkdir(parents=True, exist_ok=True)

    write(root, "authority.json", {
        "directive": "DC-DEV-021-M2-R16-POST-TRANSFER-UNIFIED-FLUX-BUDGET-AND-PRESERVATION-AUDIT-001",
        "starting_head": R15_HEAD,
        "r15_final_head": R15_HEAD,
        "r15_exact_head_ci": R15_CI,
        "r15_final_artifact": R15_ARTIFACT,
        "r15_external_disposition": "ACCEPTED - BOUNDED NEGATIVE",
        "r15_classification": "M2_MOVING_MEMBRANE_FLUX_PARITY_QUALIFIED_REPRODUCTION_NOT_ESTABLISHED",
        "resource_causal_reproduction": "NOT_ESTABLISHED",
        "assimilation": "INVESTIGATE_NOT_ACCEPTED",
        "pr_44": {
            "state": "OPEN",
            "draft": True,
            "merged": False,
            "head": "fb77f472b1519a9e0f713833efba5b1d403f4723",
        },
    })
    write(root, "r15_final_provenance_correction.json", {
        "historical_r15_result_ancestry": [
            "2c4653ee422abcf410b3cc920f53bb8f16540b2d",
            "6664e98",
            "1e31994",
            R15_HEAD,
        ],
        "stale_internal_pointer": "1e319940...",
        "final_governed_head": R15_HEAD,
        "exact_head_ci": R15_CI,
        "final_artifact": R15_ARTIFACT,
        "scientific_semantics_after_pointer_commits": "UNCHANGED_BY_GOVERNANCE_ONLY_COMMITS",
        "sealed_r15_evidence_rewritten": False,
    })
    write(root, "protocol.json", {
        "observer_only": True,
        "biology_changed": False,
        "arms": ["R15_ACTIVE", "R15_TRANSFER_DISABLED", "WHOLE_MEMBRANE_REFERENCE"],
        "steps": 12000,
        "seed": 2,
        "required_first_contact": 7452,
        "required_first_transfer": 7452,
        "required_active_delivery_each_species": R15_DELIVERY,
        "unresolved_fields": [
            "pinch_available=UNRESOLVED_BY_CURRENT_LEDGER",
            "cross_bond_a_available=UNRESOLVED_BY_CURRENT_LEDGER",
        ],
    })
    write(root, "r15_active_replay.json", summary(active))
    write(root, "r15_disabled_replay.json", summary(disabled))
    write(root, "whole_membrane_reference.json", {
        "runtime_reference_replay": summary(reference),
        "sealed_qualified_reference": sealed_reference,
        "comparability_boundary": "physiological throughput reference only; not ecologically equivalent to R15 moving-membrane founder-birth ecology",
    })

    active_audit = active["flux_audit"]
    disabled_audit = disabled["flux_audit"]
    reference_audit = reference["flux_audit"]
    write(root, "unified_flux_ledger.json", {
        "active": active_audit,
        "transfer_disabled": disabled_audit,
        "runtime_whole_membrane_reference": reference_audit,
        "sealed_whole_membrane_reference_totals": {
            "delivered_n": sealed_reference.get("delivered_n"),
            "delivered_f": sealed_reference.get("delivered_f"),
            "reaction_n_consumed": sealed_reference.get("reaction_n_consumed"),
            "reaction_f_consumed": sealed_reference.get("reaction_f_consumed"),
            "reaction_a_produced": sealed_reference.get("reaction_a_produced"),
            "reaction_w_produced": sealed_reference.get("reaction_w_produced"),
            "growth_material": sealed_reference.get("growth_material"),
            "first_fission_step": sealed_reference.get("first_fission_step"),
        },
    })
    write(root, "post_transfer_checkpoints.json", {
        "active": checkpoints(active),
        "transfer_disabled": checkpoints(disabled),
        "runtime_whole_membrane_reference": checkpoints(reference),
        "required_dense_offsets": [0, 1, 25, 50, 100, 250, 500],
    })

    active_rows = checkpoints(active)
    disabled_rows = checkpoints(disabled)
    reference_rows = checkpoints(reference)
    normalized = []
    for fraction in (0.10, 0.25, 0.50, 0.75, 1.00):
        target = fraction * R15_DELIVERY
        row = min(active_rows, key=lambda item: abs(item["cumulative_n_delivered"] - target))
        normalized.append({
            "fraction_of_r15_delivered_n": fraction,
            "target_n": target,
            "active_checkpoint": row,
            "note": "R15 transfer arrives in a finite moving-interface event; repeated fractions may map to the same observed checkpoint.",
        })
    write(root, "material_normalized_comparison.json", {
        "r15_delivery_normalizer_each_species": R15_DELIVERY,
        "active_per_delivered_n": active_audit["cumulative_reaction_n_consumed"] / R15_DELIVERY,
        "active_growth_material_per_delivered_n": active_audit["cumulative_growth_material"] / R15_DELIVERY,
        "transfer_disabled_per_delivered_n": None,
        "sealed_reference_reaction_per_delivered_n": sealed_reference["reaction_n_consumed"] / sealed_reference["delivered_n"],
        "sealed_reference_growth_per_delivered_n": sealed_reference["growth_material"] / sealed_reference["delivered_n"],
        "normalized_checkpoints": normalized,
        "boundary": "delivery-normalized values are descriptive; they do not erase the different R15/reference boundary conditions or elapsed post-transfer time.",
    })

    write(root, "reaction_processing_attribution.json", {
        "active_reaction_n_consumed": active_audit["cumulative_reaction_n_consumed"],
        "active_reaction_f_consumed": active_audit["cumulative_reaction_f_consumed"],
        "disabled_reaction_n_consumed": disabled_audit["cumulative_reaction_n_consumed"],
        "disabled_reaction_f_consumed": disabled_audit["cumulative_reaction_f_consumed"],
        "directly_measured": True,
        "classification": "QUALIFIED",
    })
    write(root, "a_w_attribution.json", {
        "active_a_produced": active_audit["cumulative_a_produced"],
        "active_w_produced": active_audit["cumulative_w_produced"],
        "active_reaction_w_produced": active_audit["cumulative_reaction_w_produced"],
        "active_growth_w_produced": active_audit["cumulative_growth_w_produced"],
        "directly_measured": True,
        "classification": "QUALIFIED",
    })
    write(root, "maintenance_work_attribution.json", {
        "active_maintenance_a": active_audit["cumulative_maintenance_a"],
        "active_work_a": active_audit["cumulative_active_work_a"],
        "disabled_maintenance_a": disabled_audit["cumulative_maintenance_a"],
        "disabled_work_a": disabled_audit["cumulative_active_work_a"],
        "directly_measured": True,
        "classification": "QUALIFIED",
    })
    write(root, "growth_attribution.json", {
        "active_growth_a": active_audit["cumulative_growth_a"],
        "active_growth_material": active_audit["cumulative_growth_material"],
        "disabled_growth_a": disabled_audit["cumulative_growth_a"],
        "disabled_growth_material": disabled_audit["cumulative_growth_material"],
        "directly_measured": True,
        "classification": "QUALIFIED",
    })
    max_active = max(active_rows, key=lambda item: item["total_structural_mass"])
    write(root, "structural_mass_attribution.json", {
        "active_max_checkpoint": max_active,
        "disabled_terminal_checkpoint": disabled_rows[-1],
        "runtime_reference_terminal_checkpoint": reference_rows[-1],
        "active_mass_gate_reached": any(item["fission_gate_reached"] for item in active_rows),
        "active_physical_fission": active["fission_events"] > 0,
        "pinch_available": "UNRESOLVED_BY_CURRENT_LEDGER",
        "cross_bond_a_available": "UNRESOLVED_BY_CURRENT_LEDGER",
    })

    write(root, "causal_divergence.json", {
        "classification": "M2_POST_TRANSFER_DIVERGENCE_UNRESOLVED",
        "first_transfer": active["first_transfer_step"],
        "transport": "QUALIFIED_BY_R15_AND_REPLAY_PRESERVED",
        "intracellular_retention": "QUALIFIED_BY_DIRECT_STATE_CAPTURE",
        "reaction_processing": "QUALIFIED_AS_MEASURED_BUT_NOT_REFERENCE_EQUIVALENT",
        "a_generation": "QUALIFIED_AS_MEASURED_BUT_NOT_REFERENCE_EQUIVALENT",
        "maintenance_burden": "QUALIFIED_AS_MEASURED_BUT_NOT_REFERENCE_EQUIVALENT",
        "active_work_burden": "QUALIFIED_AS_MEASURED_BUT_NOT_REFERENCE_EQUIVALENT",
        "growth_and_structural_mass": "DIVERGENT",
        "mass_gate_attainment": "DIVERGENT",
        "physical_fission": "DIVERGENT",
        "earliest_justified_divergence": "POST_TRANSFER_THROUGHPUT_COMPARISON_REMAINS_UNRESOLVED",
        "reason": "R15 transfer is temporally late and the accepted whole-membrane reproductive reference has different boundary conditions; the current ledger measures every downstream stage but cannot isolate a unique first post-transfer deficit without changing biology or creating a matched ecological reference.",
        "no_inferred_fix": True,
    })
    write(root, "forbidden_information_audit.json", {
        "resource_information_read_by_biology": "NONE",
        "observer_reads_only": ["existing delivery ledger", "ReactionLedger", "GrowthLedger", "existing mesh state", "existing fission observations"],
        "new_state_variable_used_by_biology": False,
        "reaction_or_growth_semantics_changed": False,
    })
    write(root, "preservation.json", {
        "r15_replay": "PASS",
        "d087_v2": "8/8",
        "d087_v3": "8/8",
        "d087_v4": "7/8",
        "v4_vector": [True, True, False, True, True, True, True, True],
        "d088": "PASS",
        "d091": "PASS",
        "evolution_harness": "PASS_TESTS_ONLY",
        "environment_dependent_evolution": "NOT_ESTABLISHED",
        "pr_44": "OPEN / DRAFT / UNMERGED / UNTOUCHED",
        "scientific_runtime_changed": False,
    })
    write(root, "qualification.json", {
        "directive": "DC-DEV-021-M2-R16-POST-TRANSFER-UNIFIED-FLUX-BUDGET-AND-PRESERVATION-AUDIT-001",
        "classification": "M2_POST_TRANSFER_DIVERGENCE_UNRESOLVED",
        "status": "GOAL_AGENT_PROVISIONAL_NEGATIVE_REPLAN",
        "resource_causal_reproduction": "NOT_ESTABLISHED",
        "assimilation": "INVESTIGATE_NOT_ACCEPTED",
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    files = sorted(p.name for p in root.glob("*.json") if p.name != "artifact_manifest.json")
    manifest = {
        "directory": root.name,
        "files": files,
        "sha256": {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in files},
    }
    write(root, "artifact_manifest.json", manifest)


if __name__ == "__main__":
    main()
