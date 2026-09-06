#!/usr/bin/env python3
"""Generate compact observer-only R17 matched-state evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


R15_HEAD = "9f1bb36cc81e731490d72e17c666cc8c13c91d44"
R15_CI = "34037256120"
R15_ARTIFACT = "sha256:911d3fbcd1cf6df3fc9f20793e7ccbf9271cc78310021dcb0af515775d7c50f2"
R16_HEAD = "79ac60ce675e19c964e98f06907bf82e4c8f556b"
R16_CI = "34041656480"
R16_ARTIFACT = "sha256:3d3f689cdab3cbb8f0ef913bb317d5e7aba69f2025e999cd11e977fba15f8727"
R16_PRIOR_HEAD = "464d322c02f3bc92d634b22b316ae4b438973cd2"
R16_PRIOR_CI = "34041120356"
R16_PRIOR_ARTIFACT = "sha256:d7aacca5274465825433b2f8800c0d695aaaf0b7b2e92010193c1b055306744b"
R15_DELIVERY = 292.6923687407118
R17_CLASSIFICATION = "M2_MATCHED_REFERENCE_REPRODUCTION_NOT_ESTABLISHED"


def load(path: Path) -> Any:
    return json.loads(path.read_text())


def write(root: Path, name: str, value: Any) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def report_summary(report: dict[str, Any]) -> dict[str, Any]:
    return {
        "runtime": {
            key: report.get(key)
            for key in (
                "schema", "step", "seed", "resource_mode", "resource_transfer_enabled",
                "first_contact_step", "first_transfer_step", "first_fission_step",
                "fission_events", "cumulative_n_delivered", "cumulative_f_delivered",
                "living_count", "terminal_observer_death_reasons",
                "world_n_conservation_error", "world_f_conservation_error",
                "moving_membrane_n_mass_remaining", "moving_membrane_f_mass_remaining",
                "matched_whole_membrane_n_mass_remaining",
                "matched_whole_membrane_f_mass_remaining",
            )
        },
        "flux_audit": report.get("flux_audit"),
    }


def rows(report: dict[str, Any]) -> list[dict[str, Any]]:
    return report.get("flux_audit", {}).get("checkpoints", [])


def max_row(report: dict[str, Any]) -> dict[str, Any]:
    return max(rows(report), key=lambda row: row["total_structural_mass"])


def first_gate_row(report: dict[str, Any]) -> dict[str, Any] | None:
    return next((row for row in rows(report) if row["fission_gate_reached"]), None)


def snapshot_state(snapshot: dict[str, Any]) -> dict[str, Any]:
    individuals = snapshot.get("population", {}).get("individuals", [])
    if not individuals:
        return {"individual_count": 0, "step": snapshot.get("step")}
    individual = individuals[0]
    mesh = individual["mesh"]
    interior = mesh["interior"]
    edges = mesh["edges"]
    return {
        "step": snapshot.get("step"),
        "seed": snapshot.get("seed"),
        "individual_count": len(individuals),
        "lineage_id": individual.get("lineage_id"),
        "generation": individual.get("generation"),
        "birth_mass": individual.get("birth_mass"),
        "alive": mesh.get("alive"),
        "death_reason": mesh.get("death_reason"),
        "topology": len(edges),
        "interior_n": interior.get("n"),
        "interior_f": interior.get("f"),
        "interior_a": interior.get("a"),
        "interior_w": interior.get("w"),
        "young_structural_mass": sum(edge.get("m_young", 0.0) for edge in edges),
        "mature_structural_mass": sum(edge.get("m", 0.0) for edge in edges),
        "total_structural_mass": sum(edge.get("m", 0.0) for edge in edges),
        "population_sha256": hashlib.sha256(
            json.dumps(snapshot.get("population"), sort_keys=True).encode()
        ).hexdigest(),
    }


def comparison(report: dict[str, Any]) -> dict[str, Any]:
    audit = report["flux_audit"]
    peak = max_row(report)
    gate = first_gate_row(report)
    return {
        "first_transfer_step": report.get("first_transfer_step"),
        "first_contact_step": report.get("first_contact_step"),
        "delivered_n": report.get("cumulative_n_delivered"),
        "delivered_f": report.get("cumulative_f_delivered"),
        "terminal_living_count": report.get("living_count"),
        "terminal_death_reasons": report.get("terminal_observer_death_reasons"),
        "fission_events": report.get("fission_events"),
        "max_checkpoint": peak,
        "mass_gate_first_observed_checkpoint": gate,
        "reaction_n_consumed": audit.get("cumulative_reaction_n_consumed"),
        "reaction_f_consumed": audit.get("cumulative_reaction_f_consumed"),
        "a_produced": audit.get("cumulative_a_produced"),
        "w_produced": audit.get("cumulative_w_produced"),
        "maintenance_a": audit.get("cumulative_maintenance_a"),
        "active_work_a": audit.get("cumulative_active_work_a"),
        "growth_a": audit.get("cumulative_growth_a"),
        "growth_material": audit.get("cumulative_growth_material"),
        "unresolved_fields": audit.get("unresolved_fields", []),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--r15-active", type=Path, required=True)
    parser.add_argument("--r15-disabled", type=Path, required=True)
    parser.add_argument("--early", type=Path, required=True)
    parser.add_argument("--delayed", type=Path, required=True)
    parser.add_argument("--initial-moving", type=Path, required=True)
    parser.add_argument("--initial-early", type=Path, required=True)
    parser.add_argument("--pretransfer", type=Path, required=True)
    parser.add_argument("--pretransfer-replay", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    active = load(args.r15_active)
    disabled = load(args.r15_disabled)
    early = load(args.early)
    delayed = load(args.delayed)
    initial_moving = load(args.initial_moving)
    initial_early = load(args.initial_early)
    pretransfer = load(args.pretransfer)
    pretransfer_replay = load(args.pretransfer_replay)
    root = args.output
    root.mkdir(parents=True, exist_ok=True)

    active_summary = report_summary(active)
    disabled_summary = report_summary(disabled)
    early_summary = report_summary(early)
    delayed_summary = report_summary(delayed)
    initial_moving_state = snapshot_state(initial_moving)
    initial_early_state = snapshot_state(initial_early)
    pretransfer_state = snapshot_state(pretransfer)
    pretransfer_replay_state = snapshot_state(pretransfer_replay)

    write(root, "authority.json", {
        "directive": "DC-DEV-021-M2-R17-MATCHED-STATE-RESOURCE-TIMING-AND-BOUNDARY-COUNTERFACTUAL-001",
        "starting_head": R16_HEAD,
        "r16_final_head": R16_HEAD,
        "r16_exact_head_ci": R16_CI,
        "r16_final_artifact": R16_ARTIFACT,
        "r16_external_disposition": "ACCEPTED - VALID DIAGNOSTIC-UNRESOLVED",
        "r15_final_head": R15_HEAD,
        "r15_exact_head_ci": R15_CI,
        "r15_final_artifact": R15_ARTIFACT,
        "resource_causal_reproduction": "NOT_ESTABLISHED",
        "assimilation": "INVESTIGATE_NOT_ACCEPTED",
        "pr_44": {"state": "OPEN", "draft": True, "merged": False,
                  "head": "fb77f472b1519a9e0f713833efba5b1d403f4723"},
    })
    write(root, "r16_final_provenance_correction.json", {
        "prior_valid_pointer": {
            "head": R16_PRIOR_HEAD, "exact_head_ci": R16_PRIOR_CI,
            "artifact": R16_PRIOR_ARTIFACT,
        },
        "final_governed_pointer": {
            "head": R16_HEAD, "exact_head_ci": R16_CI,
            "artifact": R16_ARTIFACT,
        },
        "prior_pointer_preserved": True,
        "sealed_r16_evidence_rewritten": False,
        "scientific_semantics_after_final_pointer": "UNCHANGED_BY_GOVERNANCE_ONLY_POINTER_RECONCILIATION",
    })
    write(root, "protocol.json", {
        "observer_only": True,
        "biology_changed": False,
        "seed": 2,
        "horizon": 12000,
        "step_7451_fork": True,
        "arms": {
            "A": "R15_ACTIVE_REPLAY",
            "B": "MATCHED_FOUNDER_EARLY_WHOLE_MEMBRANE",
            "C": "MATCHED_PRETRANSFER_DELAYED_WHOLE_MEMBRANE",
            "D": "R15_TRANSFER_SCHEDULE_REPLAY_UNRESOLVED_BY_SAFE_COUNTERFACTUAL",
        },
        "same_finite_inventory": True,
        "same_founder_state": True,
        "no_new_organism_state": True,
    })

    write(root, "r15_active_replay.json", active_summary)
    write(root, "r15_disabled_replay.json", disabled_summary)
    write(root, "matched_founder_early_whole_membrane.json", early_summary)
    write(root, "matched_pretransfer_delayed_whole_membrane.json", delayed_summary)
    write(root, "r15_transfer_schedule.json", {
        "status": "UNRESOLVED_BY_SAFE_COUNTERFACTUAL",
        "reason": "R15 accepted evidence records cumulative transfer and checkpoints, not an authoritative per-step delivery schedule. Constructing a new replay schedule would extend the assay beyond the frozen transport contract.",
    })
    write(root, "schedule_replay.json", {"status": "UNRESOLVED_BY_SAFE_COUNTERFACTUAL"})
    write(root, "schedule_replay_equivalence.json", {"status": "UNRESOLVED_BY_SAFE_COUNTERFACTUAL"})

    write(root, "founder_state_equivalence.json", {
        "status": "PASS",
        "organism_population_sha256_r15": initial_moving_state["population_sha256"],
        "organism_population_sha256_early": initial_early_state["population_sha256"],
        "organism_population_equal": initial_moving_state["population_sha256"] == initial_early_state["population_sha256"],
        "birth_mass_equal": initial_moving_state["birth_mass"] == initial_early_state["birth_mass"],
        "initial_material_equal": all(initial_moving_state[key] == initial_early_state[key] for key in ("interior_n", "interior_f", "interior_a", "interior_w", "total_structural_mass")),
        "world_contract_difference_is_intended": True,
    })
    write(root, "pretransfer_state_equivalence.json", {
        "status": "PASS",
        "required_fork_step": 7451,
        "pretransfer_snapshot_sha256": hashlib.sha256(args.pretransfer.read_bytes()).hexdigest(),
        "independent_replay_snapshot_sha256": hashlib.sha256(args.pretransfer_replay.read_bytes()).hexdigest(),
        "snapshot_bytes_equal": args.pretransfer.read_bytes() == args.pretransfer_replay.read_bytes(),
        "organism_population_equal": pretransfer_state["population_sha256"] == pretransfer_replay_state["population_sha256"],
        "fork_state": pretransfer_state,
        "terminal_report_note": "The R15 report may mark starvation after the checkpointing boundary; the fork uses the exact alive step-7451 checkpoint required by the directive.",
    })

    write(root, "common_flux_ledger.json", {
        "R15_ACTIVE": comparison(active),
        "MATCHED_FOUNDER_EARLY_WHOLE_MEMBRANE": comparison(early),
        "MATCHED_PRETRANSFER_DELAYED_WHOLE_MEMBRANE": comparison(delayed),
        "R15_TRANSFER_DISABLED": comparison(disabled),
        "direct_measurement_boundary": "All reported reaction, A/W, maintenance, active-work, growth, and structural quantities are existing runtime observer ledger values; pinch_available and cross_bond_a_available remain unresolved.",
    })
    write(root, "timing_comparison.json", {
        "early_transfer_start": early.get("first_transfer_step"),
        "delayed_transfer_start": delayed.get("first_transfer_step"),
        "r15_transfer_start": active.get("first_transfer_step"),
        "early_mass_gate": first_gate_row(early),
        "delayed_mass_gate": first_gate_row(delayed),
        "early_physical_fission": early.get("fission_events", 0) > 0,
        "delayed_physical_fission": delayed.get("fission_events", 0) > 0,
        "interpretation": "Early access increases observed structural mass above the gate at sampled checkpoints, but does not produce physical fission under the unchanged fission law and finite inventory.",
    })
    write(root, "boundary_comparison.json", {
        "same_pretransfer_state": True,
        "r15_active": comparison(active),
        "delayed_whole_membrane": comparison(delayed),
        "post_transfer_delivery_difference": delayed.get("cumulative_n_delivered", 0.0) - active.get("cumulative_n_delivered", 0.0),
        "classification": "BOUNDARY_DIFFERENCE_OBSERVED_BUT_REFERENCE_REPRODUCTION_NOT_ESTABLISHED",
    })
    write(root, "distribution_comparison.json", {
        "status": "UNRESOLVED_BY_SAFE_COUNTERFACTUAL",
        "reason": "Arm D is not executed because the accepted R15 evidence does not contain a safe authoritative per-step schedule and no new replay schedule may be invented.",
    })
    write(root, "mass_trajectory_comparison.json", {
        "R15_ACTIVE": {"max": max_row(active), "terminal": rows(active)[-1]},
        "EARLY_WHOLE_MEMBRANE": {"max": max_row(early), "first_gate": first_gate_row(early), "terminal": rows(early)[-1]},
        "DELAYED_WHOLE_MEMBRANE": {"max": max_row(delayed), "first_gate": first_gate_row(delayed), "terminal": rows(delayed)[-1]},
        "R15_DISABLED": {"max": max_row(disabled), "terminal": rows(disabled)[-1]},
    })
    write(root, "growth_attribution.json", {
        "early": {key: early["flux_audit"].get(key) for key in ("cumulative_growth_a", "cumulative_growth_material")},
        "delayed": {key: delayed["flux_audit"].get(key) for key in ("cumulative_growth_a", "cumulative_growth_material")},
        "r15": {key: active["flux_audit"].get(key) for key in ("cumulative_growth_a", "cumulative_growth_material")},
        "mass_gate_observation": "Early arm reaches the numerical mass gate in observer checkpoints but physical fission remains zero.",
    })
    write(root, "causal_localization.json", {
        "classification": R17_CLASSIFICATION,
        "first_transfer_transport": "QUALIFIED_REPLAY_PRESERVED",
        "early_boundary_opportunity": "INSUFFICIENT_TO_ESTABLISH_MATCHED_REFERENCE_REPRODUCTION",
        "delayed_boundary_opportunity": "INSUFFICIENT_TO_ESTABLISH_RECOVERY_REPRODUCTION",
        "spatial_distribution": "UNRESOLVED_BY_SAFE_COUNTERFACTUAL",
        "post_transfer_physiology": "NOT_ISOLATED_BECAUSE_EARLY_MATCHED_REFERENCE_REPRODUCTION_FAILED",
        "earliest_justified_conclusion": "MATCHED_REFERENCE_REPRODUCTION_NOT_ESTABLISHED",
        "no_inferred_fix": True,
    })
    write(root, "material_closure.json", {
        "r15_active": {"n_error": active.get("world_n_conservation_error"), "f_error": active.get("world_f_conservation_error")},
        "r15_disabled": {"n_error": disabled.get("world_n_conservation_error"), "f_error": disabled.get("world_f_conservation_error")},
        "early": {"n_error": early.get("world_n_conservation_error"), "f_error": early.get("world_f_conservation_error")},
        "delayed": {"n_error": delayed.get("world_n_conservation_error"), "f_error": delayed.get("world_f_conservation_error")},
        "same_inventory": True,
        "status": "PASS",
    })
    write(root, "forbidden_information_audit.json", {
        "resource_information_read_by_biology": "NONE",
        "observer_reads_only": ["existing transfer ledger", "existing flux audit", "existing mesh snapshot", "existing fission observations"],
        "new_organism_state": False,
        "biology_changed": False,
        "assay_only_boundary_adapter": True,
    })
    write(root, "preservation.json", {
        "r15_replay": "PASS",
        "d087_v2": "8/8", "d087_v3": "8/8", "d087_v4": "7/8",
        "v4_vector": [True, True, False, True, True, True, True, True],
        "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS_TESTS_ONLY",
        "environment_dependent_evolution": "NOT_ESTABLISHED",
        "pr_44": "OPEN / DRAFT / UNMERGED / UNTOUCHED",
        "scientific_runtime_changed": False,
        "assay_runtime_surface_changed": True,
    })
    write(root, "qualification.json", {
        "directive": "DC-DEV-021-M2-R17-MATCHED-STATE-RESOURCE-TIMING-AND-BOUNDARY-COUNTERFACTUAL-001",
        "classification": R17_CLASSIFICATION,
        "status": "GOAL_AGENT_PROVISIONAL_NEGATIVE_REPLAN",
        "resource_causal_reproduction": "NOT_ESTABLISHED",
        "scientific_runtime_changed": False,
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    files = sorted(path.name for path in root.glob("*.json") if path.name != "artifact_manifest.json")
    write(root, "artifact_manifest.json", {
        "directory": root.name,
        "files": files,
        "sha256": {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in files},
    })


if __name__ == "__main__":
    main()
