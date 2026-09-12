"""Fail-closed verifier for the bounded R5 causal comparison.

The Rust runner is the authority for physical transitions. This verifier only
recomputes the declared arm/control counts and checks that the sampled
operator evidence is complete, finite, and internally consistent.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


HORIZON = 14_778
ACTIVE_ENERGY_TOLERANCE = 1e-8
EXPECTED_CELLS = {
    "A_RESOURCE_N_RESOURCE_F_CURRENT_CONTROL",
    "B_FIXTURE_N_RESOURCE_F",
    "C_RESOURCE_N_FIXTURE_F",
    "D_FIXTURE_N_FIXTURE_F",
}
EXPECTED_TRACE_FIELDS = {
    "step",
    "raw_curvature_drive",
    "effective_drive",
    "adaptation_before",
    "requested_active_a",
    "funded_active_a",
    "active_w_produced",
    "passive_force_norm",
    "actual_displacement_norm",
    "segment_geometry",
}


def fail(message: str):
    raise SystemExit(message)


def finite(value, path="evidence"):
    if isinstance(value, bool):
        return
    if isinstance(value, (int, float)):
        if not math.isfinite(float(value)):
            fail(f"non-finite numeric value at {path}")
        return
    if isinstance(value, list):
        for index, item in enumerate(value):
            finite(item, f"{path}[{index}]")
        return
    if isinstance(value, dict):
        for key, item in value.items():
            finite(item, f"{path}.{key}")


def required(mapping, key, path):
    if not isinstance(mapping, dict) or key not in mapping:
        fail(f"missing {path}.{key}")
    return mapping[key]


def within_active_energy_tolerance(value):
    return abs(float(value)) <= ACTIVE_ENERGY_TOLERANCE


def run_contract_fixtures():
    """Exercise the declared zero-work and closure tolerance convention."""
    cases = {
        "exact_zero": 0.0,
        "conservation_roundoff": ACTIVE_ENERGY_TOLERANCE / 2.0,
    }
    if not all(within_active_energy_tolerance(value) for value in cases.values()):
        fail(f"active-energy tolerance fixture rejected: {cases}")
    if within_active_energy_tolerance(ACTIVE_ENERGY_TOLERANCE * 2.0):
        fail("active-energy tolerance fixture accepted an over-limit residual")
    return cases


def arm_verdict(arm, expected_cell, expected_motor=True, expected_adaptation=True):
    path = f"{expected_cell}/arm-{required(arm, 'arm', expected_cell)}"
    if required(arm, "cell", path) != expected_cell:
        fail(f"cell identity mismatch at {path}")
    if required(arm, "motor_enabled", path) is not expected_motor:
        fail(f"motor control mismatch at {path}")
    if required(arm, "adaptation_enabled", path) is not expected_adaptation:
        fail(f"adaptation control mismatch at {path}")
    if required(arm, "accepted", path) is not True:
        fail(f"accepted-step runner failed at {path}")
    accepted_steps = required(arm, "accepted_steps", path)
    if not isinstance(accepted_steps, int) or not 0 < accepted_steps <= HORIZON:
        fail(f"invalid accepted-step count at {path}: {accepted_steps!r}")
    if required(arm, "numerical_invalid", path) is not False:
        fail(f"numerical invalidity at {path}")
    if required(arm, "rejected_steps", path) != 0:
        fail(f"rejected step at {path}")
    sources = required(arm, "boundary_sources", path)
    finite(required(sources, "fixture_n", f"{path}.boundary_sources"), f"{path}.fixture_n")
    finite(required(sources, "fixture_f", f"{path}.boundary_sources"), f"{path}.fixture_f")
    if expected_cell.startswith("A_"):
        expected_n = expected_f = "RESOURCE_WAVEFORM"
    elif expected_cell.startswith("B_"):
        expected_n, expected_f = "EXACT_PER_ARM_FIXTURE_EXTERIOR", "RESOURCE_WAVEFORM"
    elif expected_cell.startswith("C_"):
        expected_n, expected_f = "RESOURCE_WAVEFORM", "EXACT_PER_ARM_FIXTURE_EXTERIOR"
    elif expected_cell.startswith("D_"):
        expected_n = expected_f = "EXACT_PER_ARM_FIXTURE_EXTERIOR"
    else:
        fail(f"unknown boundary cell at {path}")
    if required(sources, "n", f"{path}.boundary_sources") != expected_n:
        fail(f"N boundary source mismatch at {path}")
    if required(sources, "f", f"{path}.boundary_sources") != expected_f:
        fail(f"F boundary source mismatch at {path}")
    trace = required(arm, "mechanics_trace", path)
    if not isinstance(trace, list) or not trace:
        fail(f"missing mechanics trace at {path}")
    for row_index, row in enumerate(trace):
        row_path = f"{path}.mechanics_trace[{row_index}]"
        if not EXPECTED_TRACE_FIELDS.issubset(row):
            fail(f"incomplete mechanics trace at {row_path}")
        finite(row, row_path)
        geometry = required(row, "segment_geometry", row_path)
        if required(geometry, "nearest_geometrically_eligible_pair", f"{row_path}.segment_geometry") is None:
            fail(f"no nearest eligible pair recorded at {row_path}")
        finite(geometry, f"{row_path}.segment_geometry")
    attempts = required(arm, "fission_attempts", path)
    if not isinstance(attempts, list):
        fail(f"fission attempts are not a list at {path}")
    for attempt_index, attempt in enumerate(attempts):
        detail = required(attempt, "detail", f"{path}.fission_attempts[{attempt_index}]")
        if not isinstance(detail, dict):
            fail(f"attempt detail is not an object at {path}")
        for field in ("failure_class", "in_range_pairs", "signed_stress_qualified_pairs"):
            if field not in detail:
                fail(f"missing attempt predicate {path}.fission_attempts[{attempt_index}].{field}")
    ledger = required(arm, "ledger", path)
    active_spent = required(ledger, "active_a_spent", f"{path}.ledger")
    active_w = required(ledger, "active_w_produced", f"{path}.ledger")
    residual = required(arm, "active_energy_residual", path)
    if (
        not within_active_energy_tolerance(float(active_spent) - float(active_w))
        or float(residual) > ACTIVE_ENERGY_TOLERANCE
    ):
        fail(f"active A->W closure failed at {path}")
    physical_fissions = required(arm, "physical_fissions", path)
    continuations = required(arm, "daughter_continuations", path)
    viable_pairs = required(arm, "full_state_viable_pairs", path)
    if physical_fissions == 0 and continuations:
        fail(f"daughter continuation without fission at {path}")
    if viable_pairs not in (0, 1):
        fail(f"invalid viable-pair count at {path}")
    if not expected_motor and (
        not within_active_energy_tolerance(active_spent)
        or not within_active_energy_tolerance(active_w)
    ):
        fail(f"motor-off control performed active work at {path}")
    return {
        "arm": arm["arm"],
        "accepted_steps": accepted_steps,
        "physical_fissions": physical_fissions,
        "full_state_viable_pairs": viable_pairs,
        "attempts": len(attempts),
        "trace_rows": len(trace),
        "active_a_spent": active_spent,
        "active_w_produced": active_w,
        "raw_drive_maximum": max(
            max((float(value) for value in row["raw_curvature_drive"]), default=0.0)
            for row in trace
        ),
        "effective_drive_maximum": max(
            max((float(value) for value in row["effective_drive"]), default=0.0)
            for row in trace
        ),
        "adaptation_maximum": max(
            max((float(value) for value in row["adaptation_before"]), default=0.0)
            for row in trace
        ),
        "requested_active_a_maximum": max(float(row["requested_active_a"]) for row in trace),
        "funded_active_a_maximum": max(float(row["funded_active_a"]) for row in trace),
        "nearest_distance_over_range": min(
            float(row["segment_geometry"]["nearest_geometrically_eligible_pair"]["distance_over_range"])
            for row in trace
        ),
    }


def verify(value):
    contract_fixtures = run_contract_fixtures()
    finite(value)
    if required(value, "phase_steps", "root") != HORIZON:
        fail("causal comparison horizon changed")
    cells = required(value, "boundary_cells", "root")
    if (
        not isinstance(cells, list)
        or len(cells) != len(EXPECTED_CELLS)
        or {required(cell, "label", "cell") for cell in cells} != EXPECTED_CELLS
    ):
        fail("causal boundary matrix is incomplete or duplicated")
    cell_results = {}
    for cell in cells:
        label = cell["label"]
        arms = required(cell, "arms", label)
        if len(arms) != 10:
            fail(f"{label} does not contain ten arms")
        if required(cell, "phase_steps", label) != HORIZON:
            fail(f"{label} horizon changed")
        rows = [arm_verdict(arm, label) for arm in arms]
        counts = {
            "growth_qualified_arms": sum(
                bool(arm["growth_qualified"]) for arm in arms
            ),
            "physical_fissions": sum(row["physical_fissions"] for row in rows),
            "distinct_successful_arms": sum(row["physical_fissions"] > 0 for row in rows),
            "full_state_viable_pairs": sum(row["full_state_viable_pairs"] for row in rows),
        }
        emitted = required(cell, "counts", label)
        for key, expected in counts.items():
            if required(emitted, key, f"{label}.counts") != expected:
                fail(f"emitted count {key} disagrees with raw arms in {label}")
        cell_results[label] = {"counts": counts, "arms": rows}

    controls = required(value, "controls", "root")
    control_results = {}
    for key, expected_motor, expected_adaptation in (
        ("motor_off", False, True),
        ("adaptation_disabled", True, False),
    ):
        control = required(controls, key, "root.controls")
        label = required(control, "label", f"root.controls.{key}")
        if label != "A_RESOURCE_N_RESOURCE_F_CURRENT_CONTROL":
            fail(f"{key} does not use the Resource control boundary")
        arms = required(control, "arms", f"root.controls.{key}")
        if len(arms) != 10:
            fail(f"{key} does not contain ten arms")
        rows = [
            arm_verdict(arm, label, expected_motor, expected_adaptation)
            for arm in arms
        ]
        control_results[key] = {
            "physical_fissions": sum(row["physical_fissions"] for row in rows),
            "distinct_successful_arms": sum(row["physical_fissions"] > 0 for row in rows),
            "full_state_viable_pairs": sum(row["full_state_viable_pairs"] for row in rows),
            "arms": rows,
        }

    resource = cell_results["A_RESOURCE_N_RESOURCE_F_CURRENT_CONTROL"]
    adaptation = control_results["adaptation_disabled"]
    mechanism_comparison = {
        "resource_fission_arms": resource["counts"]["distinct_successful_arms"],
        "adaptation_disabled_fission_arms": adaptation["distinct_successful_arms"],
        "adaptation_disabled_changes_fission_count": adaptation["distinct_successful_arms"]
        != resource["counts"]["distinct_successful_arms"],
        "motor_off_fission_arms": control_results["motor_off"]["distinct_successful_arms"],
        "raw_excitation_present_in_resource": any(
            arm["raw_drive_maximum"] > 1e-12 for arm in resource["arms"]
        ),
        "refractory_loading_present_in_resource": any(
            arm["adaptation_maximum"] > 1e-12 for arm in resource["arms"]
        ),
        "funding_limitation_observed": any(
            arm["requested_active_a_maximum"]
            > arm["funded_active_a_maximum"] + 1e-12
            for arm in resource["arms"]
        ),
    }
    return {
        "pass": True,
        "matrix": cell_results,
        "controls": control_results,
        "mechanism_comparison": mechanism_comparison,
        "qualification": {
            "diagnostic_only": True,
            "production_ecology_requalified": False,
            "new_biological_parameters": 0,
            "thresholds_unchanged": True,
        },
        "contract_fixtures": {
            "active_energy_tolerance": ACTIVE_ENERGY_TOLERANCE,
            "cases": contract_fixtures,
            "pass": True,
        },
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not args.input.exists():
        fail(f"missing input {args.input}")
    result = verify(json.loads(args.input.read_text()))
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
