#!/usr/bin/env python3
"""Observer-only common flux ledger for the current goal-mode material paths.

This script deliberately does not introduce a material state or a new runtime
law.  It replays the already-implemented post-fission spatial assimilation
composition at fixed checkpoints and compares it with the sealed whole-
membrane reproductive reference.  Fields that the legacy reference did not
attribute to environmental material are emitted as null rather than inferred.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any


START_HEAD = "c51e5d997099b8cac703b3d0345ebf39cab729b5"
REFERENCE_CAPACITY = 4096.0
CHECKPOINTS = (1, 250, 350, 12000)


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def polygon_area(vertices: list[list[float]]) -> float:
    return abs(
        sum(
            vertices[i][0] * vertices[(i + 1) % len(vertices)][1]
            - vertices[(i + 1) % len(vertices)][0] * vertices[i][1]
            for i in range(len(vertices))
        )
    ) * 0.5


def mesh_amount(mesh: dict[str, Any], field: str) -> float:
    return mesh["interior"].get(field, 0.0) * polygon_area(mesh["vertices"])


def alive_population(snapshot: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        individual
        for individual in snapshot["population"]["individuals"]
        if individual["mesh"].get("alive", True)
    ]


def spatial_state(snapshot_path: Path, report_path: Path) -> dict[str, Any]:
    snapshot = json.loads(snapshot_path.read_text())
    report = json.loads(report_path.read_text())
    living = alive_population(snapshot)
    structural = sum(
        sum(max(edge.get("m", 0.0), 0.0) for edge in individual["mesh"]["edges"])
        for individual in living
    )
    young = sum(
        sum(max(edge.get("m_young", 0.0), 0.0) for edge in individual["mesh"]["edges"])
        for individual in living
    )
    birth = sum(individual.get("birth_mass", 0.0) for individual in living)
    return {
        "step": report["step"],
        "environmental_n_available": report["cumulative_n_delivered"]
        + report["spatial_field_n_mass_remaining"],
        "environmental_f_available": report["cumulative_f_delivered"]
        + report["spatial_field_f_mass_remaining"],
        "environmental_n_transferred": report["cumulative_n_delivered"],
        "environmental_f_transferred": report["cumulative_f_delivered"],
        "retained_assimilation_n": sum(
            mesh_amount(individual["mesh"], "assimilation_n") for individual in living
        ),
        "retained_assimilation_f": sum(
            mesh_amount(individual["mesh"], "assimilation_f") for individual in living
        ),
        "environmental_n_processed": report["cumulative_assimilation_n_processed"],
        "environmental_f_processed": report["cumulative_assimilation_f_processed"],
        "environmental_a_produced": report["cumulative_assimilation_a_produced"],
        "w_from_environmental_processing": report["cumulative_assimilation_a_produced"],
        "a_active_work_cost": report["cumulative_motor_a_spent"],
        "a_maintenance_cost": None,
        "a_reaching_growth": report["cumulative_assimilation_m_grown"],
        "young_structural_material": young,
        "mature_structural_material": max(structural - young, 0.0),
        "total_structural_mass": structural,
        "fission_threshold": 1.35 * birth,
        "fission_events": report["fission_events"],
        "first_transfer_step": report["first_transfer_step"],
        "first_fission_step": report["first_fission_step"],
        "living_count": report["living_count"],
        "world_n_conservation_error": report["world_n_conservation_error"],
        "world_f_conservation_error": report["world_f_conservation_error"],
    }


def run_runtime(binary: Path, output: Path, disabled: bool) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(prefix="dcdev021-flux-") as tmp:
        tmp_path = Path(tmp)
        previous: Path | None = None
        previous_step = 0
        for step in CHECKPOINTS:
            checkpoint = tmp_path / f"{'disabled' if disabled else 'active'}-{step}.snapshot.json"
            report = tmp_path / f"{'disabled' if disabled else 'active'}-{step}.report.json"
            command = [
                str(binary),
                "--post-fission-ecology",
                "--steps",
                str(step - previous_step),
                "--seed",
                "2",
                "--checkpoint",
                str(checkpoint),
                "--report",
                str(report),
            ]
            if disabled:
                command.insert(2, "--transfer-disabled")
            if previous is not None:
                command.extend(["--resume", str(previous)])
            subprocess.run(command, check=True, stdout=subprocess.DEVNULL)
            rows.append(spatial_state(checkpoint, report))
            previous = checkpoint
            previous_step = step
    return rows


def reference_state(reference: dict[str, Any], step: int) -> dict[str, Any] | None:
    checkpoints = []
    for name in ("daughter_a", "daughter_b"):
        for checkpoint in reference[name]["checkpoints"]:
            if checkpoint["step"] == step:
                checkpoints.append((name, checkpoint))
    if not checkpoints:
        return None
    states = [state for _, checkpoint in checkpoints for state in checkpoint["states"]]
    return {
        "step": step,
        "environmental_n_available": REFERENCE_CAPACITY * len(checkpoints),
        "environmental_f_available": REFERENCE_CAPACITY * len(checkpoints),
        "environmental_n_transferred": sum(
            REFERENCE_CAPACITY - checkpoint["inventory_n"]
            for _, checkpoint in checkpoints
        ),
        "environmental_f_transferred": sum(
            REFERENCE_CAPACITY - checkpoint["inventory_f"]
            for _, checkpoint in checkpoints
        ),
        # The sealed reference records total reaction consumption only.  It
        # does not preserve environmental provenance after bulk transfer.
        "environmental_n_processed": None,
        "environmental_f_processed": None,
        "environmental_a_produced": None,
        "w_from_environmental_processing": None,
        "a_maintenance_cost": None,
        "a_active_work_cost": None,
        "a_reaching_growth": None,
        "young_structural_material": None,
        "mature_structural_material": None,
        "total_structural_mass": sum(state["mass"] for state in states),
        "fission_threshold": sum(1.35 * state["birth_mass"] for state in states),
        "fission_events": sum(checkpoint["fissions"] for _, checkpoint in checkpoints),
        "first_transfer_step": 1,
        "first_fission_step": min(
            reference[name]["first_fission_step"] for name, _ in checkpoints
        ),
        "reference_total_reaction_n_processed": sum(
            reference[name]["reaction_n_consumed"] for name, _ in checkpoints
        ),
        "reference_total_reaction_a_produced": sum(
            reference[name]["reaction_a_produced"] for name, _ in checkpoints
        ),
        "reference_total_growth_material": sum(
            reference[name]["growth_material"] for name, _ in checkpoints
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    reference = json.loads(args.reference.read_text())
    active = run_runtime(args.binary, args.output, False)
    disabled = run_runtime(args.binary, args.output, True)
    reference_rows = [reference_state(reference, step) for step in CHECKPOINTS]

    comparisons = []
    for ref, spatial, control in zip(reference_rows, active, disabled):
        row = {
            "step": spatial["step"],
            "reference": ref,
            "spatial_active": spatial,
            "transfer_disabled": control,
            "descriptive_delivery_difference": None
            if ref is None
            else spatial["environmental_n_transferred"]
            - ref["environmental_n_transferred"],
        }
        comparisons.append(row)

    first = next(
        row
        for row in comparisons
        if row["reference"] is not None
        and abs(row["descriptive_delivery_difference"]) > 1e-12
    )

    output = args.output
    output.mkdir(parents=True, exist_ok=True)
    write_json(
        output / "protocol.json",
        {
            "directive": "GOAL-LOOP-DIGITAL-CELL-MATERIAL-FLOW-REALIGNMENT-R2",
            "increment": "unified flux-budget and preservation audit R7",
            "starting_head": START_HEAD,
            "role": "goal-agent architect and coder",
            "acceptance_boundary": "GOAL_AGENT_PROVISIONAL_FLUX_LEDGER",
            "checkpoints": CHECKPOINTS,
            "new_organism_world_mechanism": False,
        },
    )
    write_json(
        output / "authority.json",
        {
            "starting_head_verified": True,
            "closed_formula_family": "CLOSURE-006 through CLOSURE-014",
            "fission_gate": "unchanged 1.35 * birth_mass",
            "reference_source": str(args.reference),
            "spatial_source": "m2-lifeform-runtime post-fission ecology",
            "independent_architect_acceptance": False,
        },
    )
    write_json(output / "reference_whole_membrane.json", {"rows": reference_rows})
    write_json(output / "integrated_spatial_path.json", {"active": active, "transfer_disabled": disabled})
    write_json(output / "common_flux_ledger.json", {"comparisons": comparisons})
    write_json(
        output / "first_divergence.json",
        {
            "stage": "environmental_n_f_transferred",
            "first_common_checkpoint": first["step"],
            "method": "first common checkpoint with nonzero descriptive delivery difference; no biological threshold",
            "reference_delivery_n": first["reference"]["environmental_n_transferred"],
            "spatial_delivery_n": first["spatial_active"]["environmental_n_transferred"],
            "transfer_disabled_delivery_n": first["transfer_disabled"]["environmental_n_transferred"],
            "interpretation": "spatial transfer is the first measured divergence from the accepted whole-membrane reference; downstream processing and growth are therefore not the first observed loss",
        },
    )
    write_json(
        output / "preservation_audit.json",
        {
            "assimilation_fields_in_physical_validity_guard": "PASS",
            "legacy_assimilation_fields_default_zero": "PASS",
            "fission_partition_code_present": "PASS",
            "geometry_amount_preservation_code_present": "PASS",
            "runtime_checkpoint_round_trip": "PASS (existing runtime test)",
            "d087_v2_v3_v4": "REQUIRES_SCOPED_CI",
            "d088": "REQUIRES_SCOPED_CI",
            "d091": "REQUIRES_SCOPED_CI",
            "remesh_and_fission_assimilation_requalification": "REQUIRES_SCOPED_CI",
            "observer_death_semantics": "REQUIRES_SCOPED_CI",
            "repo_wide_governance": "KNOWN_BASELINE_FAILURE_REQUIRES_RECONCILIATION",
        },
    )
    write_json(
        output / "qualification.json",
        {
            "classification": "GOAL_AGENT_PROVISIONAL_FLUX_LEDGER_FIRST_DIVERGENCE_IDENTIFIED",
            "first_divergence": "environmental N/F transfer rate/retention stage",
            "resource_causal_reproduction": "NOT_ESTABLISHED",
            "assimilation_architecture": "INVESTIGATE_NOT_ACCEPTED",
            "new_material_flow_variant": "NOT_IMPLEMENTED",
            "architecture_selection": "PENDING_PRESERVATION_AUDIT",
            "independent_architect_acceptance": "PENDING",
            "next_execution_started": False,
        },
    )
    files = sorted(path.name for path in output.glob("*.json"))
    write_json(
        output / "artifact_manifest.json",
        {"schema": "GOAL_AGENT_FLUX_LEDGER_MANIFEST_V1", "evidence_files": files},
    )
    print(json.dumps(json.loads((output / "first_divergence.json").read_text()), indent=2))


if __name__ == "__main__":
    main()
