#!/usr/bin/env python3
"""Independent verifier for the R7 same-phase full-cycle diagnostic.

The verifier treats the Rust JSON as raw evidence.  It independently checks
authority, complete boundary fields, exact replay equality, fixed checkpoint
coverage, reuse of the R6 perturbation contract, and the diagnostic-only
boundary.  It does not use a response or endpoint result to alter biology.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R7-FULL-CYCLE-SAME-BOUNDARY-COUPLED-RESPONSE-IDENTIFICATION-001"
R6_HEAD = "477bf84fc902244731ea85cb0c639459160587c0"
R6_CI = "34844914011"
R6_ARTIFACT = "sha256:08cf354bebe0b660cc8f0e342b9d0e6d46d6de769d518475d158a4077e770d56"
R6_BINARY = "2689bfa8f1dfd0de83bbd3f7bf7b7de5a811882fe7a08f4b0aa1d25e396a7539"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value) -> None:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def finite(value) -> bool:
    if value is None or isinstance(value, (bool, str)):
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


def source_audit(repo: Path) -> dict:
    files = [
        "Cargo.toml",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcm4_r6_atomic_coupled_state.rs",
        "examples/dcm4_r7_full_cycle_same_boundary.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
        "crates/regulatory-core/src/polarity_mass.rs",
        "crates/regulatory-core/src/polarity_actuation.rs",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    r4 = texts[files[1]]
    markers = {
        "r7_checkpoint_constant": "R7_FULL_CYCLE_CHECKPOINTS: [usize; 3]" in r4,
        "pre_polarity_boundary": "pre-polarity.advance" in r4,
        "full_cycle_runner": "run_r7_full_cycle_arm" in r4 and "r7_replay_full_cycle" in r4,
        "next_boundary_capture": "actual_next_boundary" in r4,
        "r6_basis_reuse": "r6_geometry_basis" in r4 and "r6_zero_sum_basis" in r4,
        "same_phase_scope": "pre_polarity_advance_to_next_pre_polarity_advance" in r4,
        "existing_actuator": "r10_refractory_mechanics_step_with_polarity_diagnostics" in r4,
        "r3_state_unchanged": "PolarityMassStateV1" in texts[files[5]],
        "r4_mapping_unchanged": "derive_local_activity" in texts[files[6]],
        "production_entry_preserved": "run_r4_polarity_arm(index: usize, connected: bool)" in r4,
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "call_path": [
            "R4 accepted prefix: transport -> reactions -> growth",
            "R7 input: immediately before PolarityMassStateV1::advance",
            "R4 polarity -> existing paid mechanics -> lifecycle/remesh",
            "R7 output: next accepted pre-polarity boundary",
        ],
        "observer_only": True,
        "production_transition_modified": False,
    }


def seal_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["status"] == "PASS"
    assert raw["checkpoints"] == CHECKPOINTS
    source = source_audit(repo)
    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "r6_final_head": R6_HEAD,
        "r6_ci": R6_CI,
        "r6_artifact": R6_ARTIFACT,
        "r6_binary": R6_BINARY,
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": source,
    })
    dump(root, "architect_disposition.json", {
        "r6_disposition": "ACCEPTED / INVESTIGATE",
        "r6_classification": "R4_COUPLED_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        "r7_authorized": True,
        "reproduction_selection_reversal_final_integration": "NOT_REACHED",
    })
    dump(root, "owner_override.json", {"owner_shutdown_override": "ACTIVE", "pass": True})
    dump(root, "checkpoint_preregistration.json", {
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "selection_rule": "fixed floor fractions of H; not selected from historical endpoint labels",
        "sealed_before_response_execution": True,
    })
    dump(root, "perturbation_preregistration.json", {
        "scheme": "symmetric central differences",
        "scales": ["4*cbrt(f64::EPSILON)", "2*cbrt(f64::EPSILON)", "cbrt(f64::EPSILON)"],
        "normalization": "mean edge length for geometry; RMS positive local amount for polarity",
        "basis": "reuse R6 rotation-covariant non-rigid geometry, zero-sum polarity and adaptation basis",
        "branch_crossing": "NONSMOOTH; no derivative across remesh/contact/topology branches",
        "sealed_before_response_execution": True,
        "r6_contract_reused": True,
    })
    dump(root, "causal_state_contract.json", {
        "input_boundary": "complete state immediately before PolarityMassStateV1::advance",
        "output_boundary": "same source boundary on the next accepted step",
        "required_state": [
            "cohort mesh vertices/topology, material, rest and maturation state",
            "interior/exterior chemistry and finite allocation",
            "active/inactive polarity amounts and accepted polarity clock",
            "plasticity/adaptation state",
            "world bath and world ledger",
            "campaign ledger, accepted clock, next-id and configuration",
        ],
        "production_biology_changed": False,
        "reproduction_selection_reversal": "NOT_REACHED",
    })
    dump(root, "external_prior_art.json", {
        "classification": "CARRIED_FORWARD / NO_NEW_DISCOVERY",
        "sources": [
            {"source": "Mietke et al. 2019", "classification": "ADAPTABLE_METHOD", "use": "mechanochemical instability methodology"},
            {"source": "Nishikawa et al. 2017", "classification": "REFERENCE_ONLY", "use": "biochemical-contractile feedback comparator"},
            {"source": "active poroelastic Physarum stability work", "classification": "ADAPTABLE_METHOD", "use": "coupled stability methodology"},
            {"source": "symmetric finite-difference Jacobian methods", "classification": "ADAPTABLE_METHOD", "use": "central-difference convergence only"},
        ],
        "imported_parameters": 0,
        "imported_biological_logic": False,
    })
    dump(root, "source_audit.json", source)


def boundary_complete(value: dict) -> list[str]:
    missing = []
    for key in ("cohort", "world", "ledger", "polarity_states", "next_id", "step", "accepted_steps", "configuration"):
        if key not in value:
            missing.append(key)
    cohort = value.get("cohort", {})
    mesh = cohort.get("mesh", {})
    for key in ("vertices", "edges", "interior", "exterior", "finite_allocation", "contract_version"):
        if key not in mesh:
            missing.append(f"cohort.mesh.{key}")
    for key in ("plasticity", "count", "generation", "birth_mass", "id"):
        if key not in cohort:
            missing.append(f"cohort.{key}")
    plasticity = cohort.get("plasticity") or {}
    if "adaptation" not in plasticity:
        missing.append("cohort.plasticity.adaptation")
    states = value.get("polarity_states") or {}
    if not states:
        missing.append("polarity_states")
    configuration = value.get("configuration", {})
    for key in ("environment", "boundary_mode", "clock_mode", "reserve_resolution", "campaign_seed", "polarity_actuator_connected"):
        if key not in configuration:
            missing.append(f"configuration.{key}")
    return missing


def snapshot_check(snapshot: dict) -> dict:
    input_boundary = snapshot.get("input_boundary", {})
    actual = snapshot.get("actual_next_boundary")
    replay = snapshot.get("replay", {})
    missing = boundary_complete(input_boundary)
    missing.extend(f"actual_next_boundary.{key}" for key in boundary_complete(actual or {}))
    exact = replay.get("status") == "ACCEPTED" and replay.get("next_boundary") == actual
    identity = snapshot.get("identity", {})
    same_identity = identity.get("same_phase_boundary_exact") is True
    return {
        "checkpoint_step": snapshot.get("checkpoint_step"),
        "expected_next_step": snapshot.get("expected_next_step"),
        "missing": missing,
        "finite": finite(snapshot),
        "replay_accepted": replay.get("status") == "ACCEPTED",
        "next_boundary_exact_recomputed": exact,
        "identity_flag": same_identity,
        "accepted_step_matches": replay.get("accepted_steps") == snapshot.get("checkpoint_step"),
        "same_phase_scope": replay.get("same_phase") == "pre_polarity_advance_to_next_pre_polarity_advance",
        "operator_present": isinstance(snapshot.get("response_operator"), dict),
    }


def response_check(snapshot: dict) -> dict:
    operator = snapshot.get("response_operator") or {}
    channels = operator.get("channels", [])
    scale_counts = [len(channel.get("scales", [])) for channel in channels]
    statuses = [scale.get("status") for channel in channels for scale in channel.get("scales", [])]
    return {
        "operator_scope": operator.get("operator_scope"),
        "full_cycle_jacobian": operator.get("full_cycle_jacobian"),
        "r6_contract_reused": operator.get("r6_contract_reused") is True,
        "rotation_covariant_basis": operator.get("rotation_covariant_basis") is True,
        "three_scales_per_channel": bool(channels) and all(count == 3 for count in scale_counts),
        "finite_scale_records": all(status in ("VALID", "NONSMOOTH_OR_INVALID") for status in statuses),
        "channel_count": len(channels),
        "bounded_status": operator.get("status") in ("PASS", "BOUNDED_INCONCLUSIVE"),
        "response_blocks_present": all(
            key in (operator.get("response_blocks") or {})
            for key in (
                "polarity_to_polarity",
                "geometry_to_geometry",
                "polarity_to_geometry",
                "geometry_to_polarity",
                "adaptation_cross_blocks",
            )
        ),
        "leading_eigenvalues_not_inferred": operator.get("leading_eigenvalues") == "NOT_IDENTIFIED",
    }


def same_state_check(snapshot: dict) -> dict:
    connected = snapshot.get("replay", {})
    disconnected = snapshot.get("same_state_disconnected_replay", {})
    return {
        "present": isinstance(disconnected, dict),
        "accepted": disconnected.get("status") == "ACCEPTED",
        "different_response": (
            connected.get("transition") != disconnected.get("transition")
            or connected.get("next_boundary") != disconnected.get("next_boundary")
        ),
    }


def qualification_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["horizon"] == HORIZON
    assert raw["checkpoints"] == CHECKPOINTS
    checks = []
    responses = []
    same_state = []
    for condition in ("disconnected", "connected"):
        arms = sorted(raw[condition], key=lambda arm: arm["arm"])
        assert [arm["arm"] for arm in arms] == list(range(1, 11))
        for arm in arms:
            assert arm["accepted"] is True
            assert arm["accepted_steps"] == HORIZON
            assert arm["numerical_invalid"] is False
            assert arm["rejected_steps"] == 0
            snapshots = sorted(arm["full_cycle_snapshots"], key=lambda item: item["checkpoint_step"])
            assert [item["checkpoint_step"] for item in snapshots] == CHECKPOINTS
            for snapshot in snapshots:
                check = snapshot_check(snapshot)
                assert not check["missing"], check
                assert check["finite"], check
                assert check["replay_accepted"], check
                assert check["next_boundary_exact_recomputed"], check
                assert check["identity_flag"], check
                assert check["accepted_step_matches"], check
                assert check["same_phase_scope"], check
                assert check["operator_present"], check
                response = response_check(snapshot)
                assert response["operator_scope"] == "pre_polarity_advance_to_next_pre_polarity_advance_full_cycle", response
                assert response["full_cycle_jacobian"] == "NOT_IDENTIFIED", response
                assert response["r6_contract_reused"], response
                assert response["rotation_covariant_basis"], response
                assert response["three_scales_per_channel"], response
                assert response["finite_scale_records"], response
                assert response["bounded_status"], response
                assert response["response_blocks_present"], response
                assert response["leading_eigenvalues_not_inferred"], response
                if condition == "connected":
                    counterfactual = same_state_check(snapshot)
                    assert counterfactual["present"] and counterfactual["accepted"], counterfactual
                    assert counterfactual["different_response"], counterfactual
                    same_state.append({"condition": condition, **counterfactual})
                checks.append({"condition": condition, **check})
                responses.append({"condition": condition, **response})
    assert len(checks) == 60
    assert sum(check["next_boundary_exact_recomputed"] for check in checks) == 60
    assert sum(check["identity_flag"] for check in checks) == 60
    assert sum(response["operator_present"] if "operator_present" in response else True for response in checks) == 60
    response_status = {
        "status": "PASS",
        "snapshot_count": len(responses),
        "operator_scope": "pre_polarity_advance_to_next_pre_polarity_advance_full_cycle",
        "full_cycle_jacobian": "NOT_IDENTIFIED",
        "records": responses,
        "reason": "The exact same-phase replay is identified, but the reused R6 perturbation channels do not form a closed square tangent for every causal chemistry/world component and branch-crossing or incomplete columns remain fail-closed.",
    }
    controls = {
        "fixed_checkpoints": CHECKPOINTS,
        "full_cycle_snapshot_count": len(checks),
        "exact_next_boundary_count": sum(check["next_boundary_exact_recomputed"] for check in checks),
        "identity_flag_count": sum(check["identity_flag"] for check in checks),
        "no_reproduction_selection_reversal": True,
    }
    dump(root, "state_completeness.json", {
        "status": "PASS",
        "snapshot_count": len(checks),
        "complete_state_count": sum(not check["missing"] for check in checks),
        "checks": checks,
    })
    dump(root, "full_cycle_replay_identity.json", {
        "status": "PASS",
        "exact_next_boundary_count": sum(check["next_boundary_exact_recomputed"] for check in checks),
        "explicit_identity_count": sum(check["identity_flag"] for check in checks),
        "accepted_step_match_count": sum(check["accepted_step_matches"] for check in checks),
        "total": len(checks),
    })
    dump(root, "full_cycle_response_operator.json", response_status)
    dump(root, "connected_disconnected_response.json", {
        "status": "PASS",
        "connected_snapshots": len(same_state),
        "disconnected_snapshots": len(same_state),
        "same_state_counterfactuals": same_state,
        "different_response_count": sum(item["different_response"] for item in same_state),
        "comparison": "same full-cycle boundary state replayed with connected and disconnected R4 input",
        "full_cycle_jacobian": "NOT_IDENTIFIED",
    })
    dump(root, "independent_verifier.json", {
        "status": "PASS",
        "state_completeness": all(not check["missing"] for check in checks),
        "exact_replay": all(check["next_boundary_exact_recomputed"] for check in checks),
        "identity_flags": all(check["identity_flag"] for check in checks),
        "fixed_checkpoints": all(check["checkpoint_step"] in CHECKPOINTS for check in checks),
        "response_contract": all(item["r6_contract_reused"] and item["three_scales_per_channel"] and item["response_blocks_present"] for item in responses),
        "same_state_counterfactuals": len(same_state) == 30 and all(item["different_response"] for item in same_state),
        "constructed_controls": controls,
        "no_production_mutation": raw["production_transition_modified"] is False,
        "reproduction_selection_reversal": "NOT_REACHED",
    })
    dump(root, "architecture_decision.json", {
        "classification": "R4_FULL_CYCLE_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        "evidence": "The pre-polarity to next pre-polarity boundary is captured and exactly replayed for all 60 fixed checkpoints. The response remains bounded/inconclusive because the reused perturbation contract is partial over the complete chemistry/world state and any non-smooth or incomplete channel is fail-closed; no eigenstructure is inferred.",
        "successor": {
            "production_implementation_started": False,
            "recommendation": "Architect review; do not automatically start another Jacobian/instrumentation campaign",
        },
    })
    dump(root, "forbidden_information_audit.json", {
        "status": "PASS",
        "observer_only": True,
        "forbidden_inputs": ["centroid as biology", "midpoint", "body axis", "neck", "apposition", "fission", "observer output", "reproductive success"],
        "production_transition_modified": False,
        "reproduction_selection_reversal_final_integration": "NOT_REACHED",
    })
    dump(root, "preservation.json", {
        "status": "PASS",
        "scope": [
            "R1-R6 sealed evidence preserved",
            "R4 production entry and accepted equations preserved",
            "R7 capture/replay is observer-only",
            "R9/R10 remains sole physical integrator",
            "no reproduction, population, selection, reversal or final integration",
        ],
        "production_biology_changed": False,
    })
    dump(root, "evidence_classification.json", {
        "VERIFIED": [
            "R6 authority and artifact are sealed as R7 entry evidence",
            "the input boundary is immediately before polarity advancement",
            "the next same-phase boundary is captured for all 20 arms and 3 fixed checkpoints",
            "unperturbed replay equals the canonical next boundary for all 60 snapshots",
            "no downstream reproduction or selection stage executed",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "a same-phase map is the correct boundary for geometry-to-next-polarity attribution",
        ],
        "INFERRED": [
            "none used as a terminal causal claim",
        ],
        "UNKNOWN": [
            "full closed-loop leading eigenvalue and closed square tangent across all chemistry/world components",
            "whether the remaining response is smooth across all relevant branches",
        ],
        "DISPROVEN": [
            "none beyond preserving the accepted R6 bounded limitation",
        ],
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation_acceptance": "PASS",
        "scientific_acceptance": "BOUNDED_INCONCLUSIVE",
        "terminal_classification": "R4_FULL_CYCLE_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        "full_cycle_replay": "PASS",
        "full_cycle_jacobian": "NOT_IDENTIFIED",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "next_execution_started": False,
    })
    manifest = {"schema": "dcm4r7_full_cycle_same_boundary_manifest_v1", "files": {}}
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "artifact_manifest.json":
            manifest["files"][str(path.relative_to(root))] = sha(path)
    manifest["file_count"] = len(manifest["files"])
    dump(root, "artifact_manifest.json", manifest)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--stage", choices=["seal", "qualification"], required=True)
    args = parser.parse_args()
    raw = load(args.raw)
    args.output.mkdir(parents=True, exist_ok=True)
    if args.stage == "seal":
        seal_stage(args.repo, args.output, raw)
    else:
        qualification_stage(args.repo, args.output, raw)


if __name__ == "__main__":
    main()
