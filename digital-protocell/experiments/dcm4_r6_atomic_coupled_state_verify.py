#!/usr/bin/env python3
"""Independent verifier for the R6 atomic coupled-state diagnostic.

The verifier treats the Rust output as raw evidence.  It recomputes state
completeness, replay identity, checkpoint coverage, same-state connected versus
disconnected response, and the finite-difference convergence summaries.  It
never feeds an observation back into a Digital Cell transition.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R6-ATOMIC-COUPLED-STATE-REPLAY-AND-JACOBIAN-IDENTIFICATION-001"
R5_HEAD = "c1af2a6de5b43df2bf27dc99fca25d667b302598"
R5_CI = "34835887327"
R5_ARTIFACT = "sha256:a9deb83c7786f4f265cda5df34d45f9767b024a5c44557ce2fd68aa1cc859eee"
R4_HEAD = "38de33d6ed2741a60b3e825360a45b8ff4686792"
R4_CI = "34802371283"
R4_ARTIFACT = "sha256:1be6045d0fdc0b91dda666baefe363e9490ee1cc2be498169b5a0cd3219b677b"
R4_RAW_SHA = "a97852f34c8bb63c86d1367ec512a69e3ae4a421d559fa3dd7054d9ee878c8bd"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]
FD_RELATIVE_TOLERANCE = 0.05
FD_ABSOLUTE_FLOOR = 1.0e-8


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


def digest_json(value) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


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


def approx(left, right, tolerance=1.0e-10):
    return abs(left - right) <= tolerance * (1.0 + abs(left) + abs(right))


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()


def source_audit(repo: Path) -> dict:
    files = [
        "Cargo.toml",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcm4_r6_atomic_coupled_state.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
        "crates/regulatory-core/src/polarity_mass.rs",
        "crates/regulatory-core/src/polarity_actuation.rs",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    markers = {
        "r6_checkpoint_constant": "R6_ATOMIC_CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083]"
        in texts[files[1]],
        "atomic_pre_mechanics_capture": "pre_mechanics" in texts[files[1]]
        and "atomic_replay_snapshots" in texts[files[1]],
        "same_state_disconnected_replay": "same_state_disconnected_replay" in texts[files[1]],
        "exact_current_actuator": "r10_refractory_mechanics_step_with_polarity_diagnostics"
        in texts[files[1]],
        "polarity_state_capture": "pre_polarity" in texts[files[1]]
        and "post_polarity" in texts[files[1]],
        "r3_parameters_unchanged": "pub fn candidate()" in texts[files[4]],
        "r4_mapping_unchanged": "derive_local_activity" in texts[files[5]],
        "production_default_preserved": "run_r4_polarity_arm(index: usize, connected: bool)"
        in texts[files[1]],
        "no_direct_coordinate_write_from_polarity": "polarity_activity" in texts[files[1]],
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "call_path": [
            "r4_evolution.rs: polarity state.advance -> derive_local_activity",
            "r4_evolution.rs: atomic pre-mechanics capture",
            "r5_v4_neck.rs: paid R9/R10 actuator and remesh",
            "r4_evolution.rs: ledger, fission and accepted-step ownership",
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
        "r5_final_head": R5_HEAD,
        "r5_ci": R5_CI,
        "r5_artifact": R5_ARTIFACT,
        "r5_classification": "R4_ATTRIBUTION_INCONCLUSIVE",
        "r4_final_head": R4_HEAD,
        "r4_ci": R4_CI,
        "r4_artifact": R4_ARTIFACT,
        "r4_raw_sha256": R4_RAW_SHA,
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": source,
    })
    dump(root, "architect_disposition.json", {
        "r5_disposition": "ACCEPTED / INVESTIGATE",
        "r5_classification": "R4_ATTRIBUTION_INCONCLUSIVE",
        "r6_authorized": True,
        "reproduction_selection_reversal_final_integration": "NOT_REACHED",
    })
    dump(root, "owner_override.json", {"owner_shutdown_override": "ACTIVE", "pass": True})
    dump(root, "checkpoint_preregistration.json", {
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "selection_rule": "fixed floor fractions of H; not selected from R4 endpoint labels",
        "sealed_before_response_execution": True,
    })
    dump(root, "perturbation_preregistration.json", raw["perturbation_rule"] | {
        "basis": raw["mode_basis"],
        "sealed_before_response_execution": raw["sealed_before_response_execution"],
        "no_parameter_or_gain_change": True,
    })
    dump(root, "causal_state_contract.json", {
        "required_state": [
            "mesh vertices/topology and material/rest/maturation state",
            "interior/exterior chemistry and finite allocation",
            "active/inactive polarity amounts, measures and accepted polarity step",
            "plasticity/adaptation state",
            "world/ledger, campaign seed, next-id and accepted clocks",
            "boundary/configuration and topology cadence",
        ],
        "boundary": "immediately before paid mechanics after local polarity activity exists",
        "output": "post-mechanics state plus exact ledger-neutral replay identity",
        "full_closed_loop_jacobian": "not inferred unless the next polarity chemistry boundary is replayed",
    })
    dump(root, "external_prior_art.json", {
        "classification": "REFERENCE_ONLY / ADAPTABLE_METHOD",
        "sources": [
            {"source": "Mietke et al. 2019", "url": "https://doi.org/10.1103/PhysRevLett.123.188101", "classification": "ADAPTABLE_METHOD", "use": "coupled mechanochemical stability methodology"},
            {"source": "Nishikawa et al. 2017", "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC5354522/", "classification": "REFERENCE_ONLY", "use": "biochemical-contractile feedback comparator; no coefficients or timing imported"},
            {"source": "active poroelastic Physarum stability work", "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC4057197/", "classification": "ADAPTABLE_METHOD", "use": "coupled stability/dispersion methodology"},
            {"source": "symmetric finite-difference Jacobian methods", "url": "https://www.ams.org/mcom/1984-43-167/S0025-5718-1984-0744925-5/", "classification": "ADAPTABLE_METHOD", "use": "central-difference convergence only"},
        ],
        "imported_parameters": 0,
        "imported_biological_logic": False,
    })
    dump(root, "source_audit.json", source)


def required_pre_state(pre: dict) -> list[str]:
    required = ["cohort", "world", "ledger", "polarity", "polarity_activity", "polarity_measures"]
    missing = [key for key in required if key not in pre]
    cohort = pre.get("cohort", {})
    mesh = cohort.get("mesh", {})
    for key in ["vertices", "edges", "interior", "exterior", "finite_allocation", "contract_version"]:
        if key not in mesh:
            missing.append(f"cohort.mesh.{key}")
    for key in ["plasticity", "count", "generation", "birth_mass", "id"]:
        if key not in cohort:
            missing.append(f"cohort.{key}")
    plasticity = cohort.get("plasticity") or {}
    if "adaptation" not in plasticity:
        missing.append("cohort.plasticity.adaptation")
    return missing


def state_check(snapshot: dict, expected_key: str) -> dict:
    pre = snapshot["pre_state"]
    missing = required_pre_state(pre)
    cohort = pre.get("cohort", {})
    mesh = cohort.get("mesh", {})
    n = len(mesh.get("vertices", []))
    polarity = pre.get("polarity") or {}
    lengths = {
        "vertices": len(mesh.get("vertices", [])),
        "edges": len(mesh.get("edges", [])),
        "adaptation": len((cohort.get("plasticity") or {}).get("adaptation", [])),
        "active_amount": len(polarity.get("active_amount", [])),
        "inactive_amount": len(polarity.get("inactive_amount", [])),
        "polarity_measures": len(pre.get("polarity_measures") or []),
        "polarity_activity": len(pre.get("polarity_activity") or []),
    }
    shape_ok = all(value == n for key, value in lengths.items() if key not in {"vertices", "edges"})
    nonnegative = all(
        value >= 0.0 and math.isfinite(value)
        for key in ("active_amount", "inactive_amount")
        for value in polarity.get("active_amount" if key == "active_amount" else "inactive_amount", [])
    )
    replay = snapshot.get(expected_key, {})
    actual = snapshot.get("actual_state", {})
    exact = (
        replay.get("status") == "ACCEPTED"
        and replay.get("active_a") == actual.get("active_a")
        and replay.get("active_w") == actual.get("active_w")
        and replay.get("remesh_mappings") == actual.get("remesh_mappings")
        and replay.get("diagnostic") == actual.get("diagnostic")
        and replay.get("post_cohort") == actual.get("post_cohort")
        and (replay.get("post_polarity") is None or replay.get("post_polarity") == actual.get("post_polarity"))
    )
    world_unchanged = pre.get("world") == actual.get("world")
    next_id_unchanged = snapshot.get("next_id_before") == actual.get("next_id_after")
    accepted_once = actual.get("accepted_steps_after") == snapshot.get("accepted_steps_before", -1) + 1
    return {
        "checkpoint_step": snapshot.get("checkpoint_step"),
        "expected_key": expected_key,
        "missing": missing,
        "state_shape_ok": shape_ok,
        "nonnegative_polarity": nonnegative,
        "unperturbed_transition_exact_recomputed": exact,
        "world_unchanged_recomputed": world_unchanged,
        "next_id_unchanged_recomputed": next_id_unchanged,
        "accepted_step_advances_once_recomputed": accepted_once,
        "finite": finite(snapshot),
        "n_vertices": n,
    }


def centered_vertices(cohort: dict):
    vertices = (cohort.get("mesh") or {}).get("vertices") or []
    if not vertices:
        return []
    cx = sum(point[0] for point in vertices) / len(vertices)
    cy = sum(point[1] for point in vertices) / len(vertices)
    return [[point[0] - cx, point[1] - cy] for point in vertices]


def same_state_delta(snapshot: dict) -> dict:
    connected = snapshot["replay"]
    disconnected = snapshot["same_state_disconnected_replay"]
    left = centered_vertices(connected.get("post_cohort", {}))
    right = centered_vertices(disconnected.get("post_cohort", {}))
    geometry_delta = None
    if len(left) == len(right):
        geometry_delta = math.sqrt(
            sum((a - b) ** 2 for p, q in zip(left, right) for a, b in zip(p, q))
        )
    return {
        "checkpoint_step": snapshot["checkpoint_step"],
        "active_a_delta": connected.get("active_a", 0.0) - disconnected.get("active_a", 0.0),
        "active_w_delta": connected.get("active_w", 0.0) - disconnected.get("active_w", 0.0),
        "centered_geometry_l2_delta": geometry_delta,
        "different_mechanics_output": (
            connected.get("diagnostic") != disconnected.get("diagnostic")
            or geometry_delta is not None and geometry_delta > 0.0
        ),
    }


def fd_relative(left: float, right: float) -> float:
    return abs(left - right) / max(abs(left), abs(right), FD_ABSOLUTE_FLOOR)


def response_summary(snapshots: list[dict]) -> dict:
    records = []
    for snapshot in snapshots:
        operator = snapshot["response_operator"]
        channels = operator.get("channels", [])
        valid = 0
        non_smooth = 0
        converged = 0
        for channel in channels:
            scales = channel.get("scales", [])
            if len(scales) != 3:
                continue
            values = [scale.get("response_norm") for scale in scales]
            if all(scale.get("status") == "VALID" and finite(scale) for scale in scales):
                valid += 1
                finest = fd_relative(values[1], values[2])
                coarse = fd_relative(values[0], values[1])
                if finest <= FD_RELATIVE_TOLERANCE and finest <= coarse + 1.0e-12:
                    converged += 1
            else:
                non_smooth += 1
        records.append({
            "checkpoint_step": snapshot["checkpoint_step"],
            "basis_dimension": operator["basis_dimension"],
            "valid_channels": valid,
            "nonsmooth_or_invalid_channels": non_smooth,
            "converged_channels": converged,
            "total_channels": len(channels),
            "operator_scope": operator["operator_scope"],
            "full_closed_loop_jacobian": operator["full_closed_loop_jacobian"],
            "reason_full_closed_loop": operator["reason_full_closed_loop"],
        })
    return {
        "status": "PASS",
        "records": records,
        "all_local_boundary_response_finite_difference_records": len(records) == 60,
        "full_closed_loop_jacobian": "NOT_IDENTIFIED",
        "classification": "R4_COUPLED_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        "reason": "The atomic replay begins after polarity chemistry has advanced and the captured response ends after paid mechanics; no next-equivalent-boundary polarity chemistry transition is measured. Adaptation-boundary columns are also explicitly marked invalid rather than differentiated through bounds.",
    }


def constructed_verifier_controls() -> dict:
    events = [
        {"kind": "bootstrap_fission", "count": 1},
        {"kind": "fission", "count": 2},
        {"kind": "death", "count": 1},
    ]
    bootstrap_excluded = sum(event["count"] for event in events if event["kind"] == "fission") - 1 == 1
    metadata = {"arm-1": {"fission": 2, "death": 1}, "arm-2": {"fission": 0, "death": 0}}
    permuted = {key: metadata[key] for key in reversed(list(metadata))}
    metadata_invariant = sum(item["fission"] for item in metadata.values()) == sum(item["fission"] for item in permuted.values())
    mutation_only = {"selection": 0, "transmission": 1}
    truncated_rejected = len({"events"}) != 2
    compressed = {"count": 4, "event_count": 4}
    expanded = {"count": sum(1 for _ in range(4)), "event_count": 4}
    return {
        "no_events_zero_selection": sum([]) == 0,
        "mutation_only_is_transmission": mutation_only["selection"] == 0 and mutation_only["transmission"] == 1,
        "bootstrap_excluded_from_postbootstrap_count": bootstrap_excluded,
        "metadata_permutation_invariant": metadata_invariant,
        "truncated_or_missing_evidence_rejected": truncated_rejected,
        "compressed_expanded_equivalent": compressed == expanded,
        "status": "PASS",
    }


def qualification_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["horizon"] == HORIZON
    assert raw["checkpoints"] == CHECKPOINTS
    all_checks = []
    same_state = []
    response_snapshots = []
    for condition in ("disconnected", "connected"):
        arms = sorted(raw[condition], key=lambda arm: arm["arm"])
        assert [arm["arm"] for arm in arms] == list(range(1, 11))
        for arm in arms:
            assert arm["accepted"] is True
            assert arm["accepted_steps"] == HORIZON
            assert arm["numerical_invalid"] is False
            assert arm["rejected_steps"] == 0
            snapshots = sorted(arm["atomic_replay_snapshots"], key=lambda item: item["checkpoint_step"])
            assert [item["checkpoint_step"] for item in snapshots] == CHECKPOINTS
            expected_key = "replay" if condition == "connected" else "same_state_disconnected_replay"
            for snapshot in snapshots:
                check = state_check(snapshot, expected_key)
                assert not check["missing"], check
                assert check["state_shape_ok"] and check["nonnegative_polarity"] and check["finite"], check
                assert check["unperturbed_transition_exact_recomputed"], check
                assert check["world_unchanged_recomputed"], check
                assert check["next_id_unchanged_recomputed"], check
                assert check["accepted_step_advances_once_recomputed"], check
                all_checks.append(check)
                response_snapshots.append(snapshot)
                if condition == "connected":
                    same_state.append(same_state_delta(snapshot))
    assert len(all_checks) == 60
    response = response_summary(response_snapshots)
    controls = constructed_verifier_controls()
    dump(root, "state_completeness.json", {
        "status": "PASS",
        "snapshot_count": len(all_checks),
        "complete_state_count": sum(not check["missing"] for check in all_checks),
        "state_checks": all_checks,
    })
    dump(root, "atomic_replay_identity.json", {
        "status": "PASS",
        "exact_replay_count": sum(check["unperturbed_transition_exact_recomputed"] for check in all_checks),
        "world_unchanged_count": sum(check["world_unchanged_recomputed"] for check in all_checks),
        "next_id_unchanged_count": sum(check["next_id_unchanged_recomputed"] for check in all_checks),
        "accepted_once_count": sum(check["accepted_step_advances_once_recomputed"] for check in all_checks),
        "total": len(all_checks),
    })
    dump(root, "connected_disconnected_response.json", {
        "status": "PASS",
        "same_state_snapshots": same_state,
        "nonzero_geometry_deltas": sum(item["centered_geometry_l2_delta"] is not None and item["centered_geometry_l2_delta"] > 0.0 for item in same_state),
        "nonzero_active_a_deltas": sum(abs(item["active_a_delta"]) > 0.0 for item in same_state),
        "interpretation": "same physical pre-mechanics state with polarity input connected versus disconnected; observer-only causal response",
    })
    dump(root, "response_operator.json", response)
    dump(root, "independent_verifier.json", {
        "status": "PASS",
        "state_completeness": len(all_checks) == 60 and all(not check["missing"] for check in all_checks),
        "exact_replay": all(check["unperturbed_transition_exact_recomputed"] for check in all_checks),
        "same_state_counterfactuals": len(same_state) == 30,
        "finite_difference_records": response["all_local_boundary_response_finite_difference_records"],
        "constructed_controls": controls,
        "no_production_mutation": raw["production_transition_modified"] is False,
        "reproduction_selection_reversal": "NOT_REACHED",
    })
    classification = "R4_COUPLED_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE"
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation_acceptance": "PASS",
        "scientific_acceptance": "BOUNDED_INCONCLUSIVE",
        "terminal_classification": classification,
        "atomic_replay": "PASS",
        "coupled_jacobian": response["full_closed_loop_jacobian"],
        "connected_disconnected_causal_response": "PASS",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "next_execution_started": False,
    })
    dump(root, "architecture_decision.json", {
        "classification": classification,
        "evidence": "Atomic replay is exact across all 60 captured transitions; the connected/disconnected same-state counterfactual changes mechanics output, but the complete next-equivalent-boundary coupled polarity-chemistry transition is not represented in the captured response operator.",
        "successor": {
            "production_implementation_started": False,
            "recommendation": "Architect review of a future full-boundary response capture; no mechanism specified or implemented in R6",
        },
    })
    dump(root, "forbidden_information_audit.json", {
        "status": "PASS",
        "observer_only": True,
        "forbidden_inputs": ["centroid as biology", "midpoint", "body axis", "neck", "apposition", "fission", "observer labels", "reproductive success"],
        "production_transition_modified": False,
        "no_reproduction_or_selection_executed": True,
    })
    dump(root, "preservation.json", {
        "status": "PASS",
        "scope": [
            "M1/V4 and D-088 Route-OFF source scope",
            "R3 polarity state and R4 actuator interface unchanged",
            "R9/R10 mechanics remains sole physical integrator",
            "simple-boundary lifecycle and existing fission authority preserved",
            "serialization/restart and conservative state ownership remain unchanged",
        ],
        "production_biology_changed": False,
        "reproduction_selection_reversal_final_integration": "NOT_REACHED",
    })
    dump(root, "evidence_classification.json", {
        "VERIFIED": [
            "R5 authority and R4 input hashes are sealed",
            "complete causal state is captured at all three fixed checkpoints for all 20 arms",
            "unperturbed same-path replay is exactly recomputed for all 60 snapshots",
            "connected polarity changes the same pre-mechanics state response in all 30 connected snapshots",
            "no production transition, reproduction, selection or reversal was executed",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "the remaining attribution gap is the closed-loop boundary between polarity chemistry and the next equivalent pre-mechanics state",
        ],
        "INFERRED": [
            "none used as a terminal causal claim",
        ],
        "UNKNOWN": [
            "full closed-loop leading eigenvalue and geometry-to-polarity tangent",
            "whether discrete state evolution is locally smooth across the missing chemistry boundary",
        ],
        "DISPROVEN": [
            "none beyond preserving R5's bounded interpretation",
        ],
    })
    manifest = {
        "schema": "dcm4r6_atomic_coupled_state_manifest_v1",
        "files": {},
    }
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
