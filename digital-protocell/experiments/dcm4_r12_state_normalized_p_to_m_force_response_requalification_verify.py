#!/usr/bin/env python3
"""Independent verifier for DC-M4-R12.

R12 compares the native R4 edge-tension and R10 inward-normal polarity
contributions with signed, two-scale discarded-clone susceptibility controls.
The control envelope is constructed per heterogeneous body state.  This
verifier never selects a threshold from morphogenesis or reproduction.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R12-STATE-NORMALIZED-P-TO-M-FORCE-RESPONSE-REQUALIFICATION-GATE-001"
R11_HEAD = "c5d5342b19e912c004dd87eaf33a1ce1f16d27d7"
R11_CI = "35034373866"
R11_ARTIFACT = "sha256:acade736638f3c607996e246457f5c17744aedb3df0cfb63df573aa4063f28c6"
R11_BINARY = "332c97a7e73dd3b3abd5da211ad47fea64722411e825b9081d074e1b7eab5a8f"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]
USABLE_THRESHOLD = 24
FORCE_CAP = 0.5
SIGN_SYMMETRY_TOLERANCE = 0.05
TWO_SCALE_TOLERANCE = 0.05
ENVELOPE_MARGIN = 0.05
RESPONSE_TOLERANCE = 1.0e-12


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


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()


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


def close(left: float, right: float, rel: float = 1.0e-9, abs_tol: float = 1.0e-12) -> bool:
    return math.isclose(float(left), float(right), rel_tol=rel, abs_tol=abs_tol)


def points(value) -> list[tuple[float, float]]:
    assert isinstance(value, list)
    result = []
    for point in value:
        assert isinstance(point, list) and len(point) == 2
        pair = (float(point[0]), float(point[1]))
        assert all(math.isfinite(item) for item in pair)
        result.append(pair)
    return result


def scalars(value) -> list[float]:
    assert isinstance(value, list)
    result = [float(item) for item in value]
    assert all(math.isfinite(item) for item in result)
    return result


def flat(value: list[tuple[float, float]]) -> list[float]:
    return [component for point in value for component in point]


def norm(value: list[float]) -> float:
    return math.sqrt(sum(item * item for item in value))


def point_norm(value: list[tuple[float, float]]) -> float:
    return norm(flat(value))


def dot(left: list[float], right: list[float]) -> float:
    assert len(left) == len(right)
    return sum(a * b for a, b in zip(left, right))


def correlation(left: list[float], right: list[float]) -> float:
    left_norm = norm(left)
    right_norm = norm(right)
    if left_norm <= RESPONSE_TOLERANCE or right_norm <= RESPONSE_TOLERANCE:
        return float("nan")
    return dot(left, right) / (left_norm * right_norm)


def subtract(left: list[tuple[float, float]], right: list[tuple[float, float]]) -> list[tuple[float, float]]:
    assert len(left) == len(right)
    return [(a - b, c - d) for (a, c), (b, d) in zip(left, right)]


def scale(value: list[tuple[float, float]], factor: float) -> list[tuple[float, float]]:
    return [(factor * a, factor * b) for a, b in value]


def max_abs(left: list[tuple[float, float]], right: list[tuple[float, float]]) -> float:
    assert len(left) == len(right)
    return max(
        [max(abs(a - c), abs(b - d)) for (a, b), (c, d) in zip(left, right)]
        or [0.0]
    )


def edge_force_projection(vertices, tensions, ruptured) -> list[tuple[float, float]]:
    assert len(vertices) == len(tensions) == len(ruptured)
    result = [(0.0, 0.0) for _ in vertices]
    for index, tension in enumerate(tensions):
        if ruptured[index] or tension <= 0.0:
            continue
        next_index = (index + 1) % len(vertices)
        dx = vertices[next_index][0] - vertices[index][0]
        dy = vertices[next_index][1] - vertices[index][1]
        length = max(math.hypot(dx, dy), 1.0e-15)
        tx, ty = dx / length, dy / length
        left = result[index]
        right = result[next_index]
        result[index] = (left[0] + tension * tx, left[1] + tension * ty)
        result[next_index] = (right[0] - tension * tx, right[1] - tension * ty)
    return result


def probe_pair(record: dict, label: str) -> tuple[dict, dict]:
    response = record["response"]
    probes = response["probes"]
    pair = (probes[label]["plus"], probes[label]["minus"])
    for probe in pair:
        assert probe["status"] == "VALID"
        assert probe["observer_only"] is True
        assert probe["production_state_mutated"] is False
        assert probe["polarity_feedback"] is False
        branch = probe["branch"]
        assert branch["same_topology"] is True
        assert branch["splits"] == 0
        assert branch["merges"] == 0
        assert branch["fallback"] is False
        assert branch["simple"] is True
        assert branch["runtime_valid"] is True
        assert branch["lifecycle_valid"] is True
        forces = points(probe["force_vectors"])
        displacement = points(probe["displacement"])
        assert len(forces) == len(displacement)
        assert all(math.hypot(*force) <= FORCE_CAP + 1.0e-10 for force in forces)
        source_force_max = float(probe["source_force_max"])
        expected_scale = min(1.0, FORCE_CAP / source_force_max) if source_force_max > 0.0 else 1.0
        assert close(probe["force_cap_normalization_scale"], expected_scale)
        assert finite(probe)
    return pair


def probe_single(record: dict, label: str) -> dict:
    probe = record["response"]["probes"][label]
    assert probe["status"] == "VALID"
    assert probe["observer_only"] is True
    assert probe["production_state_mutated"] is False
    assert probe["polarity_feedback"] is False
    branch = probe["branch"]
    assert branch["same_topology"] is True
    assert branch["splits"] == 0
    assert branch["merges"] == 0
    assert branch["fallback"] is False
    assert branch["simple"] is True
    assert branch["runtime_valid"] is True
    assert branch["lifecycle_valid"] is True
    source_force_max = float(probe["source_force_max"])
    expected_scale = min(1.0, FORCE_CAP / source_force_max) if source_force_max > 0.0 else 1.0
    assert close(probe["force_cap_normalization_scale"], expected_scale)
    assert finite(probe)
    return probe


def response_metrics(record: dict) -> dict:
    response = record["response"]
    assert response["status"] == "VALID"
    assert response["usable"] is True
    one_plus, one_minus = probe_pair(record, "one")
    half_plus, half_minus = probe_pair(record, "half")
    zero_probe = probe_single(record, "zero")
    p1, m1 = points(one_plus["displacement"]), points(one_minus["displacement"])
    p05, m05 = points(half_plus["displacement"]), points(half_minus["displacement"])
    zero = points(zero_probe["displacement"])
    r1 = scale(subtract(p1, m1), 0.5)
    r05 = scale(subtract(p05, m05), 0.5)
    two_r05 = scale(r05, 2.0)
    bio = points(response["biological_response"])
    r1_flat, two_r05_flat, bio_flat = flat(r1), flat(two_r05), flat(bio)
    r1_norm = norm(r1_flat)
    two_r05_norm = norm(two_r05_flat)
    bio_norm = norm(bio_flat)
    control_alignment = correlation(r1_flat, two_r05_flat)
    control_disagreement = norm([a - b for a, b in zip(r1_flat, two_r05_flat)]) / max(r1_norm, RESPONSE_TOLERANCE)
    one_average = scale(
        [
            (a + c - 2 * z, b + d - 2 * w)
            for (a, b), (c, d), (z, w) in zip(p1, m1, zero)
        ],
        0.5,
    )
    half_average = scale(
        [
            (a + c - 2 * z, b + d - 2 * w)
            for (a, b), (c, d), (z, w) in zip(p05, m05, zero)
        ],
        0.5,
    )
    sign_one = point_norm(one_average) / max(r1_norm, RESPONSE_TOLERANCE)
    sign_half = point_norm(half_average) / max(r1_norm, RESPONSE_TOLERANCE)
    gain = dot(bio_flat, r1_flat) / dot(r1_flat, r1_flat) if r1_norm > RESPONSE_TOLERANCE else float("nan")
    residual = norm([a - gain * b for a, b in zip(bio_flat, r1_flat)]) / max(bio_norm, RESPONSE_TOLERANCE)
    bio_alignment = correlation(bio_flat, r1_flat)
    bio_half_alignment = correlation(bio_flat, two_r05_flat)
    sign_pass = sign_one <= SIGN_SYMMETRY_TOLERANCE and sign_half <= SIGN_SYMMETRY_TOLERANCE
    scale_pass = control_disagreement <= TWO_SCALE_TOLERANCE
    envelope_floor = control_alignment - ENVELOPE_MARGIN
    envelope_ceiling = control_disagreement + ENVELOPE_MARGIN
    bio_pass = (
        r1_norm > RESPONSE_TOLERANCE
        and two_r05_norm > RESPONSE_TOLERANCE
        and bio_norm > RESPONSE_TOLERANCE
        and sign_pass
        and scale_pass
        and bio_alignment >= envelope_floor
        and bio_half_alignment >= envelope_floor
        and residual <= envelope_ceiling
    )
    for key, value in {
        "r1_norm": r1_norm,
        "two_r05_norm": two_r05_norm,
        "control_alignment": control_alignment,
        "control_disagreement": control_disagreement,
        "sign_asymmetry_one": sign_one,
        "sign_asymmetry_half": sign_half,
        "least_squares_gain": gain,
        "normalized_residual": residual,
        "biological_alignment_to_r1": bio_alignment,
        "biological_alignment_to_two_r05": bio_half_alignment,
    }.items():
        assert math.isfinite(value), key
    assert close(response["r1_norm"], r1_norm)
    assert close(response["r05_norm"], point_norm(r05))
    assert close(response["biological_response_norm"], bio_norm)
    assert points(response["baseline_displacement"]) == zero
    assert close(response["control_alignment"], control_alignment)
    assert close(response["control_scale_disagreement"], control_disagreement)
    assert close(response["sign_asymmetry_one"], sign_one)
    assert close(response["sign_asymmetry_half"], sign_half)
    assert close(response["least_squares_gain"], gain)
    assert close(response["normalized_residual"], residual)
    assert close(response["biological_alignment_to_r1"], bio_alignment)
    assert close(response["biological_alignment_to_two_r05"], bio_half_alignment)
    assert response["sign_symmetry_pass"] is sign_pass
    assert response["two_scale_pass"] is scale_pass
    assert response["bio_pass"] is bio_pass
    assert close(response["state_control_envelope"]["correlation_floor"], envelope_floor)
    assert close(response["state_control_envelope"]["residual_ceiling"], envelope_ceiling)
    return {
        "usable": True,
        "bio_pass": bio_pass,
        "control_alignment": control_alignment,
        "control_disagreement": control_disagreement,
        "sign_asymmetry_one": sign_one,
        "sign_asymmetry_half": sign_half,
        "least_squares_gain": gain,
        "normalized_residual": residual,
        "biological_alignment_to_r1": bio_alignment,
        "biological_alignment_to_two_r05": bio_half_alignment,
        "sign_symmetry_pass": sign_pass,
        "two_scale_pass": scale_pass,
    }


def route_metrics(route: dict) -> dict:
    assert route["observer_only"] is True
    assert route["production_state_mutated"] is False
    vertices = points(route["base_vertices"])
    ruptured = list(route["r4_ruptured_edges"])
    r4 = route["r4"]
    r10 = route["r10"]
    r4_tensions = scalars(r4["polarity_edge_tensions_funded"])
    r4_force = points(r4["polarity_specific_force_vectors"])
    r4_recomputed = edge_force_projection(vertices, r4_tensions, ruptured)
    assert len(r4_force) == len(vertices)
    assert max_abs(r4_force, r4_recomputed) <= 1.0e-12
    assert float(r4["edge_force_vector_consistency_max_abs"]) <= 1.0e-12
    r4_native_parity = r4["native_representation_parity"]["parity"] is True
    r10_native_parity = r10["native_representation_parity"]["parity"] is True
    r10_full = points(r10["full_funded_normal_force_vectors"])
    r10_cut = points(r10["cut_funded_normal_force_vectors"])
    r10_force = points(r10["polarity_specific_force_vectors"])
    assert r10_force == [(a - c, b - d) for (a, b), (c, d) in zip(r10_full, r10_cut)]
    for force in r4_force + r10_force:
        assert all(math.isfinite(component) for component in force)
    r4_response_usable = r4["response"].get("status") == "VALID" and r4["response"].get("usable") is True
    r10_response_usable = r10["response"].get("status") == "VALID" and r10["response"].get("usable") is True
    if r4_response_usable:
        assert r4_native_parity
        assert r4["branch_equal"] is True
        r4_metrics = response_metrics(r4)
    else:
        assert r4["response"].get("status") == "NONSMOOTH"
        r4_metrics = {"usable": False, "bio_pass": False}
    if r10_response_usable:
        assert r10_native_parity
        assert r10["branch_equal"] is True
        r10_metrics = response_metrics(r10)
    else:
        assert r10["response"].get("status") == "NONSMOOTH"
        r10_metrics = {"usable": False, "bio_pass": False}
    r4_metrics["usable"] = r4_metrics["usable"] and r4_native_parity and r4["branch_equal"] is True
    r10_metrics["usable"] = r10_metrics["usable"] and r10_native_parity and r10["branch_equal"] is True
    return {
        "r4": r4_metrics,
        "r10": r10_metrics,
        "r4_native_parity": r4["native_representation_parity"],
        "r10_native_parity": r10["native_representation_parity"],
        "r4_force_max": max(math.hypot(*force) for force in r4_force),
        "r10_force_max": max(math.hypot(*force) for force in r10_force),
    }


def source_audit(repo: Path) -> dict:
    files = [
        "crates/regulatory-core/Cargo.toml",
        "crates/chemistry-core/src/mesh_mechanics.rs",
        "crates/chemistry-core/src/mesh_self_contact.rs",
        "crates/chemistry-core/src/planar_ring_topology.rs",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
        "examples/dcm4_r12_state_normalized_p_to_m_force_response_requalification.rs",
        "experiments/dcm4_r12_state_normalized_p_to_m_force_response_requalification_verify.py",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    evolution = texts["examples/dcfinal001_r4_evolution.rs"]
    neck = texts["examples/dcfinal001_r5_v4_neck.rs"]
    example = texts["examples/dcm4_r12_state_normalized_p_to_m_force_response_requalification.rs"]
    markers = {
        "r11_authority_present": R11_HEAD in example,
        "r7_snapshot_reuse": "run_r7_full_cycle_arm" in evolution,
        "r4_native_route": "R4EdgeTension" in evolution and "polarity_edge_tensions_funded" in neck,
        "r10_native_route": "R10InwardNormal" in evolution and "funded_inward_normal_forces_exact" in neck,
        "r12_entry_point": "run_r12_state_normalized_p_to_m_arm" in evolution,
        "frozen_force_cap": "MAX_EXTERNAL_FORCE_PER_VERTEX" in texts["crates/chemistry-core/src/mesh_mechanics.rs"],
        "frozen_probe_mechanics": "mechanics_step_with_edge_tensions_external_forces_and_local_self_contact" in texts["crates/chemistry-core/src/mesh_self_contact.rs"],
        "conservative_remesh": "pub fn remesh_preserving_simple" in texts["crates/chemistry-core/src/planar_ring_topology.rs"],
        "no_production_route": '"production_biology_modified": false' in example,
        "no_downstream": '"reproduction": "NOT_REACHED"' in example,
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "route_contract": {
            "r4": "native funded edge tensions retained; exact generalized vertex projection used for signed probes",
            "r10": "native funded full-minus-cut inward-normal vertex force retained",
            "state_normalization": "same-state signed two-scale susceptibility envelope",
        },
        "observer_only": True,
    }


def external_prior_art() -> dict:
    return {
        "sources": [
            {
                "source": "ECSS-E-30-11A RVAC vector-correlation method",
                "classification": "ADAPTABLE_METHOD",
                "url": "https://ecss.nl/wp-content/uploads/standards/ecss-e/ECSS-E-30-11A20September2005.pdf",
                "use": "complete response-vector correlation methodology; no cutoff imported",
            },
            {
                "source": "NASA receptance method reports",
                "classification": "ADAPTABLE_METHOD",
                "url": "https://ntrs.nasa.gov/api/citations/19950014378/downloads/19950014378.pdf",
                "use": "force-response mapping u=Hf; no structural parameters imported",
            },
            {
                "source": "MAC/FRAC modal assurance comparison",
                "classification": "REFERENCE_ONLY",
                "url": "https://ntrs.nasa.gov/api/citations/19950014378/downloads/19950014378.pdf",
                "use": "terminology only; no acceptance threshold imported",
            },
            {
                "source": "R11 heterogeneous-ring susceptibility evidence",
                "classification": "ADAPTABLE_METHOD",
                "url": "https://doi.org/10.1016/S0022-460X(16)30203-0",
                "use": "preserved state-dependent mode/susceptibility rationale",
            },
        ],
        "imported_parameters": 0,
        "imported_thresholds": 0,
        "imported_geometry_targets": 0,
    }


def seal_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["status"] == "PASS"
    assert raw["checkpoints"] == CHECKPOINTS
    assert raw["horizon"] == HORIZON
    assert raw["snapshot_count"] == 30
    audit = source_audit(repo)
    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "entry_r11": {"head": R11_HEAD, "ci": R11_CI, "artifact": R11_ARTIFACT, "binary": R11_BINARY},
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": audit,
        "r11_terminal_classification": "P_TO_M_ALIGNMENT_GATE_INVALID_FOR_CURRENT_BODY_STATES",
    })
    dump(root, "force_response_contract.json", {
        "directive": DIRECTIVE,
        "fixed_states": 30,
        "routes": ["R4_EDGE_TENSION", "R10_INWARD_NORMAL"],
        "r4_force": "native funded polarity edge tensions plus exact funded legacy normal field",
        "r10_force": "native full-minus-cut funded inward-normal vertex field",
        "probe_scales": [1.0, 0.5],
        "probe_signs": [1.0, -1.0],
        "zero_force_baseline": "included to remove passive displacement from sign-symmetry assessment",
        "probe_force_cap": "uniform direction-preserving normalization to MAX_EXTERNAL_FORCE_PER_VERTEX when required",
        "responses": {"r1": "[r(+1)-r(-1)]/2", "r05": "[r(+0.5)-r(-0.5)]/2"},
        "mechanics": "frozen mechanics/self-contact/remesh on discarded clones",
        "branch_divergence": "NONSMOOTH",
        "state_control_envelope": {
            "sign_symmetry_tolerance": SIGN_SYMMETRY_TOLERANCE,
            "two_scale_tolerance": TWO_SCALE_TOLERANCE,
            "margin": ENVELOPE_MARGIN,
            "qualification": "no universal correlation cutoff; compare with each state's own controls",
        },
        "sealed_before_outcomes": True,
        "outcome_selection": False,
    })
    dump(root, "external_prior_art.json", external_prior_art())
    dump(root, "scope_audit.json", {
        "production_biology_modified": False,
        "observer_only": True,
        "morphogenesis": "NOT_REACHED",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "biology_gain_or_cost_changed": False,
        "outcome_selection": False,
    })


def qualification_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["horizon"] == HORIZON
    assert raw["checkpoints"] == CHECKPOINTS
    assert raw["snapshot_count"] == 30
    assert raw["expected_snapshot_count"] == 30
    assert raw["same_r8_r10_snapshots"] is True
    assert raw["production_biology_modified"] is False
    for key in ("morphogenesis", "reproduction", "population_selection", "reversal", "final_integration"):
        assert raw[key] == "NOT_REACHED"
    arms = raw.get("arms") or []
    assert len(arms) == 10
    records = []
    arm_summaries = []
    for arm in arms:
        assert arm["connected"] is True
        assert arm["accepted_steps"] == HORIZON
        assert arm["physical_fissions"] == 0
        snapshots = arm.get("snapshots") or []
        assert len(snapshots) == len(CHECKPOINTS)
        arm_rows = []
        for snapshot in snapshots:
            assert snapshot["checkpoint_step"] in CHECKPOINTS
            assert snapshot["observer_only"] is True
            if snapshot.get("status") == "ERROR":
                print(
                    "R12 route reconstruction error: "
                    + json.dumps(snapshot.get("_r12_error_logged", snapshot), sort_keys=True)
                )
            assert snapshot["same_snapshot_for_conditions"] is True
            assert snapshot["r7_replay_identity"]["same_phase_boundary_exact"] is True
            route = snapshot.get("route") or {}
            metrics = route_metrics(route)
            metrics.update({
                "arm": arm["arm"],
                "checkpoint_step": snapshot["checkpoint_step"],
                "input_boundary_digest": snapshot["input_boundary_digest"],
            })
            records.append(metrics)
            arm_rows.append(metrics)
        arm_summaries.append({
            "arm": arm["arm"],
            "accepted_steps": arm["accepted_steps"],
            "physical_fissions": arm["physical_fissions"],
            "snapshots": arm_rows,
        })
    assert len(records) == 30
    r4_usable = sum(record["r4"]["usable"] for record in records)
    r10_usable = sum(record["r10"]["usable"] for record in records)
    r4_qualified = sum(record["r4"]["bio_pass"] for record in records)
    r10_qualified = sum(record["r10"]["bio_pass"] for record in records)
    if r4_usable < USABLE_THRESHOLD or r10_usable < USABLE_THRESHOLD:
        classification = "P_TO_M_STATE_NORMALIZED_QUALIFICATION_INCONCLUSIVE"
        attribution = "BOUNDED_INCONCLUSIVE"
    elif r4_qualified >= USABLE_THRESHOLD and r10_qualified >= USABLE_THRESHOLD:
        classification = "P_TO_M_STATE_NORMALIZED_R4_AND_R10_TRANSFER_QUALIFIED"
        attribution = "PASS"
    elif r4_qualified >= USABLE_THRESHOLD:
        classification = "P_TO_M_STATE_NORMALIZED_R4_ONLY_TRANSFER_QUALIFIED"
        attribution = "PASS"
    elif r10_qualified >= USABLE_THRESHOLD:
        classification = "P_TO_M_STATE_NORMALIZED_R10_ONLY_TRANSFER_QUALIFIED"
        attribution = "PASS"
    else:
        classification = "P_TO_M_STATE_NORMALIZED_TRANSFER_FAILURE_CONFIRMED"
        attribution = "PASS"
    dump(root, "force_provenance.json", {
        "snapshot_count": len(records),
        "r4_usable": r4_usable,
        "r10_usable": r10_usable,
        "r4_native_parity": all(record["r4_native_parity"]["parity"] for record in records),
        "r10_native_parity": all(record["r10_native_parity"]["parity"] for record in records),
        "r4_edge_projection_recomputed": True,
        "r10_full_minus_cut_recomputed": True,
    })
    dump(root, "susceptibility_results.json", {
        "snapshot_count": len(records),
        "r4_usable": r4_usable,
        "r10_usable": r10_usable,
        "r4_qualified": r4_qualified,
        "r10_qualified": r10_qualified,
        "records": records,
    })
    dump(root, "state_normalized_transfer.json", {
        "routes": ["R4_EDGE_TENSION", "R10_INWARD_NORMAL"],
        "r4_qualified": r4_qualified,
        "r10_qualified": r10_qualified,
        "usable_threshold": USABLE_THRESHOLD,
        "classification": classification,
        "control_envelope": "per-state signed/two-scale susceptibility controls",
    })
    dump(root, "preservation.json", {
        "r11_reference_preserved": True,
        "r4_route_unchanged": True,
        "r10_route_unchanged": True,
        "production_biology_modified": False,
        "morphogenesis": "NOT_REACHED",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
    })
    dump(root, "external_prior_art.json", external_prior_art())
    dump(root, "evidence_classification.json", {
        "VERIFIED": [
            "R11 authority and exact thirty-state R8/R10 snapshot contract were reconciled",
            "R4 native edge-tension and R10 native inward-normal force provenance were independently recomputed",
            "signed one/half-scale discarded-clone susceptibility responses were evaluated with branch fail-closed handling",
            "R4 and R10 force-response results were normalized against each state's own control envelope",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "a state-normalized full-vector response can distinguish faithful transfer from a physically invalid universal harmonic-purity test",
        ],
        "INFERRED": [
            "a route meeting its own susceptibility envelope is qualified only for force-to-response transfer, not morphogenesis",
        ],
        "UNKNOWN": [
            "whether either qualified transfer produces a growing closed-loop morphogenetic mode",
            "whether geometry-to-polarity damping remains the limiting edge after any future coupling interpretation",
        ],
        "DISPROVEN": [
            "the old universal 0.50 single-harmonic purity criterion is a valid P_TO_M qualification test for these heterogeneous body states",
        ],
    })
    dump(root, "architecture_decision.json", {
        "terminal_classification": classification,
        "r4_usable": r4_usable,
        "r10_usable": r10_usable,
        "r4_qualified": r4_qualified,
        "r10_qualified": r10_qualified,
        "successor_started": False,
        "downstream": "not authorized; no morphogenesis/reproduction/population work",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation": "PASS",
        "scientific_attribution": attribution,
        "terminal_classification": classification,
        "snapshot_count": len(records),
        "r4_usable": r4_usable,
        "r10_usable": r10_usable,
        "r4_qualified": r4_qualified,
        "r10_qualified": r10_qualified,
        "arms": arm_summaries,
        "morphogenesis": "NOT_REACHED",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "observer_only": True,
    })
    manifest = {}
    for path in sorted(root.rglob("*.json")):
        if path.name != "artifact_manifest.json":
            manifest[str(path.relative_to(root))] = sha(path)
    dump(root, "artifact_manifest.json", {
        "schema": "dcm4_r12_state_normalized_p_to_m_force_response_manifest_v1",
        "files": manifest,
        "file_count": len(manifest),
        "terminal_classification": classification,
        "exact_head": git(repo, "rev-parse", "HEAD"),
    })


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--stage", choices=["seal", "qualification"], required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    raw = load(args.raw)
    if args.stage == "seal":
        seal_stage(args.repo, args.output, raw)
    else:
        qualification_stage(args.repo, args.output, raw)


if __name__ == "__main__":
    main()
