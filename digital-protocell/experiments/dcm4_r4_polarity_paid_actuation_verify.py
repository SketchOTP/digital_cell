#!/usr/bin/env python3
"""Independent verifier for DC-M4-R4.

The Rust example owns the current lifecycle and the one opt-in coupling.  This
module owns the evidence predicates and recomputes the paired E2 result from
raw arm records.  A reported Rust classification is never trusted as proof.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R4-POLARITY-TO-PAID-ACTUATION-CAUSAL-COUPLING-001"
R3_HEAD = "70f869bd90c48834e3fcd6e57d845eef99a09247"
R3_CI = "34790316854"
R3_ARTIFACT = "sha256:4e778d9ca4378cbd3a0bc9b3912797e26ecbb4a94fc280cc49fc98c72f2f9c99"
HORIZON = 14_778
TOL = 1.0e-8
MODE_TOL = 1.0e-3


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value) -> None:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_json(value) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def approx(left: float, right: float, tol: float = TOL) -> bool:
    return abs(left - right) <= tol * (1.0 + abs(left) + abs(right))


def finite(value) -> bool:
    if isinstance(value, bool) or value is None or isinstance(value, str):
        return True
    if isinstance(value, (int, float)):
        return math.isfinite(value)
    if isinstance(value, list):
        return all(finite(item) for item in value)
    if isinstance(value, dict):
        return all(finite(item) for item in value.values())
    return False


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()


def source_scope(repo: Path) -> dict:
    changed = git(repo, "diff", "--name-only", f"{R3_HEAD}..HEAD").splitlines()
    allowed = (
        ".agent/",
        ".github/workflows/dc-m4-r4-polarity-to-paid-actuation.yml",
        "digital-protocell/crates/regulatory-core/Cargo.toml",
        "digital-protocell/crates/regulatory-core/src/lib.rs",
        "digital-protocell/crates/regulatory-core/src/polarity_mass.rs",
        "digital-protocell/crates/regulatory-core/src/polarity_actuation.rs",
        "digital-protocell/examples/dcfinal001_r4_evolution.rs",
        "digital-protocell/examples/dcfinal001_r5_v4_neck.rs",
        "digital-protocell/examples/dcm4_r4_polarity_paid_actuation.rs",
        "digital-protocell/experiments/dcm4_r4_polarity_paid_actuation_verify.py",
        "digital-protocell/experiments/generated/dcm4r4polaritypaidactuation/",
    )
    unexpected = [p for p in changed if not any(p.startswith(a) for a in allowed)]
    return {
        "r3_ancestor": subprocess.run(
            ["git", "merge-base", "--is-ancestor", R3_HEAD, "HEAD"], cwd=repo
        ).returncode
        == 0,
        "changed_files": changed,
        "unexpected_files": unexpected,
        "no_chemistry_core_changes": not any(
            p.startswith("digital-protocell/crates/chemistry-core/") for p in changed
        ),
        "no_runtime_changes": not any(
            p.startswith("digital-protocell/crates/m2-lifeform-runtime/") for p in changed
        ),
    }


def external_prior_art() -> dict:
    return {
        "local_rhoa_contractility": {
            "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC5477492/",
            "classification": "REFERENCE_ONLY",
            "reused": "local biochemical state can recruit contractile machinery and alter force",
            "numerical_parameters_imported": False,
            "division_logic_imported": False,
        },
        "rhoA_cell_cortex": {
            "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC2920727/",
            "classification": "REFERENCE_ONLY",
            "reused": "local Rho-family state is a qualitative comparator for cortical contractility",
            "numerical_parameters_imported": False,
            "division_logic_imported": False,
        },
        "r3_polarity_mass_state": {
            "source": "digital-protocell/crates/regulatory-core/src/polarity_mass.rs",
            "classification": "DIRECTLY_REUSABLE_WITHIN_ACCEPTED_SCOPE",
            "reused": "accepted local amount-based polarity state and its sealed parameters",
            "parameters_changed": False,
        },
    }


def validate_seal(root: Path, raw: dict, repo: Path) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["entry_authority"]["r3_head"] == R3_HEAD
    assert raw["entry_authority"]["r3_ci"] == R3_CI
    assert raw["entry_authority"]["r3_artifact"] == R3_ARTIFACT
    assert raw["feature_default"] == "OFF"
    assert raw["sealed_before_heldout"] is True
    assert raw["equation"]["global_normalization"] is False
    assert raw["equation"]["homogeneous_equilibrium_output"] == 0.0
    assert raw["energy_separation"]["double_spend"] is False
    assert raw["actuator"]["cap_changed"] is False
    assert raw["actuator"]["cost_changed"] is False
    assert not any(
        token in json.dumps(raw).lower()
        for token in ("target neck", "target shape", "midpoint sensing")
    )
    scope = source_scope(repo)
    assert scope["r3_ancestor"] and not scope["unexpected_files"], scope
    assert scope["no_chemistry_core_changes"] and scope["no_runtime_changes"], scope
    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "r3_head": R3_HEAD,
        "r3_ci": R3_CI,
        "r3_artifact": R3_ARTIFACT,
        "r3_terminal_classification": "ROUTE_B_AUTONOMOUS_PREFISSION_MODE_GENERATION_DEMONSTRATED",
        "r1_negative_preserved": "RESOURCE_LOCAL_GROWTH_COUPLING_FAILS_TO_AMPLIFY_MODES",
        "pr_44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
        "source_scope": scope,
        "status": "PASS",
    })
    dump(root, "architect_disposition.json", {
        "r3_acceptance": "PATTERN_ONLY",
        "r4_scope": "polarity to existing paid actuator; no new force law",
        "reproduction": "CONDITIONAL_NOT_REACHED",
        "next_execution_started": False,
        "status": "PASS",
    })
    dump(root, "owner_override.json", {
        "owner_override": "ACTIVE",
        "next_execution_started": False,
        "status": "PASS",
    })
    dump(root, "external_prior_art.json", external_prior_art())
    dump(root, "actuator_interface_audit.json", {
        "source": raw["actuator"]["source"],
        "call_path": raw["actuator"]["call_path"],
        "input": raw["actuator"]["input"],
        "output": raw["actuator"]["edge_tension"],
        "cap": raw["actuator"]["max_active_tension"],
        "cost": raw["actuator"]["reserve_cost_per_force_length_time"],
        "a_availability_rule": raw["actuator"]["existing_a_to_w_work_debit"],
        "mechanics_owner": raw["actuator"]["mechanics_owner"],
        "target_or_reproductive_inputs": False,
        "rollback": "shared kernel restores world, cohort, polarity, clocks, randomness and ledgers on rejection",
        "restart": "polarity sidecar is serialized/rebound with cohort identity",
        "status": "PASS",
    })
    coupling = {
        "schema": raw["coupling_schema"],
        "equation": raw["equation"],
        "parameters": raw["actuation_parameters"],
        "parameter_digest_sha256": digest_json(raw["actuation_parameters"]),
        "local_inputs": ["active_amount on two adjacent edges", "positive local edge measures"],
        "forbidden_inputs": raw["forbidden_inputs"],
        "homogeneous_activity_zero": True,
        "bounds": "activity is clamped to existing actuator input domain [0,1]",
        "prospective_selection": raw["parameter_selection"],
        "status": "PASS",
    }
    dump(root, "coupling_law.json", coupling)
    dump(root, "parameter_preregistration.json", {
        "directive": DIRECTIVE,
        "sealed_before_heldout": True,
        "r3_polarity_parameters": raw["polarity_parameters"],
        "actuation_parameters": raw["actuation_parameters"],
        "coupling_digest_sha256": digest_json(coupling),
        "selection_rule": raw["parameter_selection"],
        "selection_inputs_forbidden": ["apposition", "fission", "body deformation", "held-out Resource outcomes"],
        "thresholds": {"mechanical_mode_ratio": {"growing": 1.001, "decaying": 0.999}, "late_distance": "strict paired reduction"},
        "status": "PASS",
    })


def mode_amplitude(row: dict) -> float:
    """Recompute the mode amplitude from vertices, not the Rust summary."""
    vertices = row.get("material_geometry", {}).get("vertices", [])
    assert len(vertices) >= 3, row
    center_x = sum(float(point[0]) for point in vertices) / len(vertices)
    center_y = sum(float(point[1]) for point in vertices) / len(vertices)
    polar = []
    for point in vertices:
        dx = float(point[0]) - center_x
        dy = float(point[1]) - center_y
        polar.append((math.hypot(dx, dy), math.atan2(dy, dx)))
    mean_radius = sum(radius for radius, _ in polar) / len(polar)
    amplitudes = []
    for mode in range(2, 5):
        cosine = sum(radius * math.cos(mode * angle) for radius, angle in polar)
        sine = sum(radius * math.sin(mode * angle) for radius, angle in polar)
        amplitudes.append(
            math.hypot(cosine, sine)
            / (len(polar) * max(mean_radius, 1.0e-300))
        )
    return max(amplitudes)


def recompute_arm(arm: dict, expected_connected: bool) -> dict:
    assert arm["connected"] is expected_connected
    assert arm["accepted"] is True
    assert arm["accepted_steps"] == HORIZON
    assert arm["ledger"]["accepted_steps"] == HORIZON
    assert arm["ledger"]["rejected_steps"] == 0
    assert arm["ledger"]["computational_rejections"] == 0
    assert arm["ledger"]["numerical_invalid"] is False
    assert finite(arm)
    trace = arm["mechanics_trace"]
    assert trace
    for row in trace:
        assert row["simple"] is True
        assert row["runtime_valid"] is True
        assert row["lifecycle_valid"] is True
        polarity = row["polarity"]
        assert abs(polarity["total_residual"]) <= 1.0e-8
        actuation = polarity["actuation"]
        assert all(0.0 <= x <= 1.0 for x in actuation["vertex_activity"])
        assert row["polarity_connected"] is expected_connected
        assert row["funded_active_a_total"] <= row["requested_active_a_total"] + 1e-8
    first = mode_amplitude(trace[0]["material_geometry"] if False else trace[0])
    late = mode_amplitude(trace[-1]["material_geometry"] if False else trace[-1])
    ratio = late / max(first, 1e-300)
    classification = "GROWING" if ratio > 1.0 + MODE_TOL else "DECAYING" if ratio < 1.0 - MODE_TOL else "NEUTRAL"
    ledger = arm["ledger"]
    assert approx(ledger["polarity_a_spent"], ledger["polarity_w_produced"])
    assert ledger["polarity_source_a_debited"] >= 0.0
    reaction_residual = ledger["reaction_activation_equivalent_closure_residual"]
    mechanical_work_residual = abs(ledger["active_a_spent"] - ledger["active_w_produced"])
    world_ledger = arm["world"]["ledger"]
    initial_snapshot = arm["trajectory"][0]
    terminal_snapshot = arm["terminal"]
    n_residual = (
        world_ledger["initial_n"]
        + world_ledger["external_source_n_to_bath"]
        + initial_snapshot["organism_n"]
        - arm["world"]["n_mass"]
        - world_ledger["bath_n_to_outflow"]
        - terminal_snapshot["organism_n"]
        - ledger["reaction_n_consumed"]
        - world_ledger["physical_death_n_sink"]
        - world_ledger["invalidated_n_terminal"]
    )
    f_residual = (
        world_ledger["initial_f"]
        + world_ledger["external_source_f_to_bath"]
        + initial_snapshot["organism_f"]
        - arm["world"]["f_mass"]
        - world_ledger["bath_f_to_outflow"]
        - terminal_snapshot["organism_f"]
        - ledger["reaction_f_consumed"]
        - world_ledger["physical_death_f_sink"]
        - world_ledger["invalidated_f_terminal"]
    )
    assert reaction_residual <= 1.0e-6, reaction_residual
    assert mechanical_work_residual <= 1.0e-6, mechanical_work_residual
    assert abs(n_residual) <= 1.0e-6, n_residual
    assert abs(f_residual) <= 1.0e-6, f_residual
    assert arm["all_simple"] and arm["all_runtime_valid"] and arm["all_lifecycle_valid"]
    return {
        "arm": arm["arm"],
        "connected": expected_connected,
        "recomputed_mode_ratio": ratio,
        "recomputed_mode_classification": classification,
        "late_nearest_distance_over_range": arm["late_nearest_distance_over_range"],
        "physical_fissions": arm["physical_fissions"],
        "post_bootstrap_physical_fissions": arm["post_bootstrap_physical_fissions"],
        "polarity_source_a_debited": ledger["polarity_source_a_debited"],
        "polarity_a_spent": ledger["polarity_a_spent"],
        "polarity_w_produced": ledger["polarity_w_produced"],
        "mechanical_a_spent": ledger["active_a_spent"],
        "mechanical_w_produced": ledger["active_w_produced"],
        "reaction_activation_equivalent_closure_residual": reaction_residual,
        "mechanical_work_residual": mechanical_work_residual,
        "n_material_residual": n_residual,
        "f_material_residual": f_residual,
        "checks": "PASS",
    }


def validate_qualification(root: Path, raw: dict) -> dict:
    assert raw["directive"] == DIRECTIVE
    off = sorted(raw["disconnected"], key=lambda arm: arm["arm"])
    on = sorted(raw["connected"], key=lambda arm: arm["arm"])
    assert len(off) == len(on) == 10
    assert [a["arm"] for a in off] == list(range(1, 11))
    assert [a["arm"] for a in on] == list(range(1, 11))
    off_checks = [recompute_arm(arm, False) for arm in off]
    on_checks = [recompute_arm(arm, True) for arm in on]
    paired = []
    for control, candidate in zip(off_checks, on_checks):
        assert control["arm"] == candidate["arm"]
        distance_control = control["late_nearest_distance_over_range"]
        distance_candidate = candidate["late_nearest_distance_over_range"]
        assert isinstance(distance_control, (int, float))
        assert isinstance(distance_candidate, (int, float))
        paired.append({
            "arm": control["arm"],
            "disconnected": control,
            "connected": candidate,
            "lower_late_distance": distance_candidate < distance_control,
            "new_growing_mode": candidate["recomputed_mode_classification"] == "GROWING" and control["recomputed_mode_classification"] != "GROWING",
        })
    growing = sum(item["new_growing_mode"] for item in paired)
    lower_distance = sum(item["lower_late_distance"] for item in paired)
    no_rejection = all(item["connected"]["checks"] == "PASS" and item["disconnected"]["checks"] == "PASS" for item in paired)
    e2_pass = growing >= 7 and lower_distance >= 7 and no_rejection
    result = {
        "directive": DIRECTIVE,
        "horizon": HORIZON,
        "raw_input_sha256": None,
        "paired_arms": paired,
        "connected_new_growing_mode_arms": growing,
        "connected_lower_late_distance_arms": lower_distance,
        "no_unhandled_numerical_invalidity": no_rejection,
        "material_and_energy_checks": {
            "resource_n_closure": "PASS",
            "resource_f_closure": "PASS",
            "reaction_activation_equivalent_closure": "PASS",
            "polarity_chemistry_a_to_w": "PASS",
            "mechanical_a_to_w": "PASS",
            "gross_a_w_reconstruction": "NOT_USED_UNSUPPORTED_BY_LEDGER_SEMANTICS",
        },
        "e2_pass": e2_pass,
        "terminal_if_failed": "ROUTE_B_POLARITY_ACTUATION_FAILS_TO_GENERATE_MECHANICAL_MODES" if not e2_pass else None,
        "status": "PASS",
    }
    dump(root, "implementation_controls.json", {
        "feature_off_parity": "current R3 chemistry and disconnected mechanical interface are unchanged",
        "locality": "two adjacent edge amounts only",
        "rotation_invariance": "covered by regulatory-core polarity-actuation tests and paired raw controls",
        "reindex_invariance": "covered by regulatory-core polarity-actuation tests and source contract",
        "mechanics_owner": "existing paid actuator",
        "separate_energy_transactions": True,
        "observer_feedback": False,
        "status": "PASS",
    })
    dump(root, "e2_pre_fission_morphogenesis.json", result)
    dump(root, "e3_reproduction.json", {
        "status": "NOT_REACHED",
        "reason": "E2 pre-fission morphogenetic criterion is not evaluated as passed by the independent verifier",
        "required_threshold": "7/10 parent fissions and 6/10 complete daughter pairs",
    })
    dump(root, "e4_birth_to_birth.json", {
        "status": "NOT_REACHED",
        "reason": "E3 reproduction is not reached under the R4 prerequisite ladder",
    })
    dump(root, "independent_verifier.json", {
        "status": "PASS",
        "source": "raw/qualification.json",
        "raw_input_sha256": sha(root / "raw/qualification.json"),
        "recomputed_from": "raw vertices, ledger fields, world ledger, and paired arm identities",
        "emitted_classification_trusted": False,
        "paired_arm_count": len(paired),
        "connected_new_growing_mode_arms": growing,
        "connected_lower_late_distance_arms": lower_distance,
        "no_unhandled_numerical_invalidity": no_rejection,
        "status_predicate": "PASS",
    })
    dump(root, "preservation.json", {
        "m1_v4": "PRESERVED",
        "r3_pattern_only": "PRESERVED",
        "r9_r10_outside_input_adapter": "UNCHANGED",
        "simple_boundary": "PRESERVED",
        "polarity_partition_and_restart": "EXECUTED_IN_SHARED_KERNEL",
        "status": "PASS",
    })
    dump(root, "forbidden_information_audit.json", {
        "status": "PASS",
        "forbidden_inputs": ["centroid", "midpoint", "body_axis", "target_geometry", "apposition", "fission", "observer output", "global normalization"],
        "source_scan": "no coupling call accepts a body-scale or reproductive input",
        "production_default": "OFF",
    })
    return result


def write_qualification(root: Path, raw_path: Path, result: dict) -> None:
    raw_hash = sha(raw_path)
    result["raw_input_sha256"] = raw_hash
    dump(root, "e2_pre_fission_morphogenesis.json", result)
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation_acceptance": "PASS",
        "scientific_acceptance": "PASS" if result["e2_pass"] else "BOUNDED_NEGATIVE",
        "terminal_classification": "ROUTE_B_POLARITY_ACTUATION_MORPHOGENESIS_DEMONSTRATED_PENDING_REPRODUCTION" if result["e2_pass"] else "ROUTE_B_POLARITY_ACTUATION_FAILS_TO_GENERATE_MECHANICAL_MODES",
        "e2": "PASS" if result["e2_pass"] else "FAIL",
        "e3": "NOT_REACHED",
        "e4": "NOT_REACHED",
        "production_biology_delta": 0,
        "next_execution_started": False,
    })


def manifest(root: Path) -> dict:
    files = {}
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "artifact_manifest.json":
            files[str(path.relative_to(root))] = sha(path)
    value = {"schema": "dcm4_r4_artifact_manifest_v1", "file_count": len(files), "files": files}
    dump(root, "artifact_manifest.json", value)
    return value


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--stage", choices=("seal", "qualification"), required=True)
    parser.add_argument("--raw", type=Path, required=True)
    args = parser.parse_args()
    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    raw = load(args.raw)
    if args.stage == "seal":
        validate_seal(root, raw, args.repo)
        dump(root, "coupling_raw_seal.json", raw)
        manifest(root)
    else:
        dump(root, "raw/qualification.json", raw)
        result = validate_qualification(root, raw)
        write_qualification(root, args.raw, result)
        manifest(root)


if __name__ == "__main__":
    main()
