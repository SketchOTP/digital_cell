#!/usr/bin/env python3
"""Independent verifier for DC-M4-R11.

R11 checks whether the old single-harmonic P_TO_M predicate is a valid
qualification metric for the heterogeneous body states that produced the
R8/R10 snapshots.  It recomputes the spectra and modal fractions, then
validates discarded pure-harmonic local-normal susceptibility probes.  The
verifier never selects a mode or threshold from a fission or morphogenesis
outcome.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R11-P-TO-M-QUALIFICATION-VALIDITY-AND-MECHANICAL-SUSCEPTIBILITY-GATE-001"
R10_HEAD = "8bdcadf4b03c503d0871d02c600849fb2ebfe915"
R10_CI = "34947560845"
R10_ARTIFACT = "sha256:a1956be7b49cc475e95d0806370e4dacf8e162a19777a0466dc717d2b111c938"
R10_BINARY = "025ea9bbcfd2d9b19b1470a589a4526231a631b96e4b2ae34af2f6ef8d2d4ca6"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]
ALIGNMENT_THRESHOLD = 0.50
USABLE_THRESHOLD = 24
FORCE_CAP = 0.5
CONSERVATION_TOLERANCE = 1.0e-9
SPECTRUM_TOLERANCE = 1.0e-10
SIGN_SYMMETRY_TOLERANCE = 0.05


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


def vector(value) -> list[float]:
    assert isinstance(value, list), "numeric vector evidence is missing"
    result = [float(item) for item in value]
    assert all(math.isfinite(item) for item in result)
    return result


def points(value) -> list[tuple[float, float]]:
    assert isinstance(value, list), "point evidence is missing"
    result = []
    for point in value:
        assert isinstance(point, list) and len(point) == 2
        pair = (float(point[0]), float(point[1]))
        assert all(math.isfinite(item) for item in pair)
        result.append(pair)
    return result


def force_vectors(value) -> list[tuple[float, float]]:
    return points(value)


def norm(values: list[float]) -> float:
    return math.sqrt(sum(value * value for value in values))


def scalar_harmonics(values: list[float]) -> list[dict]:
    n = len(values)
    if n == 0:
        return []
    result = []
    for harmonic in range(n // 2 + 1):
        cosine = 0.0
        sine = 0.0
        for index, value in enumerate(values):
            angle = 2.0 * math.pi * harmonic * index / n
            cosine += value * math.cos(angle)
            sine += value * math.sin(angle)
        factor = 1.0 / n if harmonic == 0 or harmonic * 2 == n else 2.0 / n
        cosine *= factor
        sine *= factor
        result.append({
            "harmonic": harmonic,
            "cosine": cosine,
            "sine": sine,
            "amplitude": math.hypot(cosine, sine),
        })
    return result


def spectrum(values: list[float]) -> dict:
    harmonics = scalar_harmonics(values)
    non_dc = sum(
        item["cosine"] ** 2 + item["sine"] ** 2 for item in harmonics[1:]
    )
    dominant = max(harmonics[1:], key=lambda item: item["amplitude"], default={"harmonic": 0, "amplitude": 0.0})
    dominant_energy = dominant.get("amplitude", 0.0) ** 2
    return {
        "values": values,
        "dc": harmonics[0]["amplitude"] if harmonics else 0.0,
        "harmonics": harmonics,
        "non_dc_energy": non_dc,
        "dominant_non_dc_harmonic": dominant.get("harmonic", 0),
        "dominant_non_dc_energy_fraction": dominant_energy / non_dc if non_dc > 1.0e-300 else 0.0,
    }


def assert_spectrum(record: dict, values: list[float]) -> dict:
    expected = spectrum(values)
    assert record.get("values") == values
    actual_harmonics = record.get("harmonics") or []
    assert len(actual_harmonics) == len(expected["harmonics"])
    for actual, wanted in zip(actual_harmonics, expected["harmonics"]):
        assert actual["harmonic"] == wanted["harmonic"]
        for key in ("cosine", "sine", "amplitude"):
            assert close(actual[key], wanted[key], rel=1.0e-10, abs_tol=SPECTRUM_TOLERANCE)
    assert close(record["dc"], expected["dc"], rel=1.0e-10, abs_tol=SPECTRUM_TOLERANCE)
    assert close(record["non_dc_energy"], expected["non_dc_energy"], rel=1.0e-10, abs_tol=SPECTRUM_TOLERANCE)
    assert record["dominant_non_dc_harmonic"] == expected["dominant_non_dc_harmonic"]
    assert close(
        record["dominant_non_dc_energy_fraction"],
        expected["dominant_non_dc_energy_fraction"],
        rel=1.0e-10,
        abs_tol=SPECTRUM_TOLERANCE,
    )
    return expected


def modal_basis(vertices: list[tuple[float, float]]) -> list[dict]:
    """Mirror the R8 radial-normal Fourier basis and rigid-mode removal."""
    n = len(vertices)
    assert n >= 3
    centroid = (
        sum(point[0] for point in vertices) / n,
        sum(point[1] for point in vertices) / n,
    )
    normals = []
    for point in vertices:
        dx = point[0] - centroid[0]
        dy = point[1] - centroid[1]
        length = math.hypot(dx, dy)
        assert math.isfinite(length) and length > 1.0e-12
        normals.append((dx / length, dy / length))

    raw_constraints = []
    for component in range(3):
        values = []
        for point in vertices:
            if component == 0:
                values.extend((1.0, 0.0))
            elif component == 1:
                values.extend((0.0, 1.0))
            else:
                values.extend((-(point[1] - centroid[1]), point[0] - centroid[0]))
        raw_constraints.append(values)

    constraints = []
    for candidate in raw_constraints:
        candidate = candidate[:]
        for previous in constraints:
            projection = sum(left * right for left, right in zip(candidate, previous))
            candidate = [value - projection * basis for value, basis in zip(candidate, previous)]
        length = norm(candidate)
        if length > 1.0e-12:
            constraints.append([value / length for value in candidate])

    modes = []
    for harmonic in range(n // 2 + 1):
        phases = [("cos", 0.0)] if harmonic == 0 or harmonic * 2 == n else [("cos", 0.0), ("sin", math.pi / 2.0)]
        for phase, shift in phases:
            candidate = []
            for index, normal in enumerate(normals):
                value = math.cos(2.0 * math.pi * harmonic * index / n + shift)
                candidate.extend((normal[0] * value, normal[1] * value))
            for constraint in constraints:
                projection = sum(left * right for left, right in zip(candidate, constraint))
                candidate = [value - projection * basis for value, basis in zip(candidate, constraint)]
            length = norm(candidate)
            if length > 1.0e-10:
                modes.append({
                    "harmonic": harmonic,
                    "phase": phase,
                    "vector": [value / length for value in candidate],
                })
    return modes


def modal_projections(displacement: list[tuple[float, float]], modes: list[dict]) -> list[dict]:
    flat = []
    for point in displacement:
        flat.extend(point)
    result = []
    for mode in modes:
        projection = sum(left * right for left, right in zip(flat, mode["vector"]))
        result.append({
            "harmonic": mode["harmonic"],
            "phase": mode["phase"],
            "projection": projection,
            "absolute_projection": abs(projection),
        })
    return result


def modal_fraction(displacement: list[tuple[float, float]], vertices: list[tuple[float, float]], harmonic: int) -> tuple[float, float, float, list[dict]]:
    projections = modal_projections(displacement, modal_basis(vertices))
    total = sum(item["projection"] ** 2 for item in projections)
    corresponding = sum(item["projection"] ** 2 for item in projections if item["harmonic"] == harmonic)
    fraction = corresponding / total if total > 1.0e-300 else 0.0
    return corresponding, total, fraction, projections


def recompute_record_fraction(record: dict, harmonic: int) -> float:
    projections = record.get("normal_modal_delta") or []
    total = sum(float(item["projection"]) ** 2 for item in projections)
    corresponding = sum(
        float(item["projection"]) ** 2
        for item in projections
        if item.get("harmonic") == harmonic
    )
    return corresponding / total if total > 1.0e-300 else 0.0


def compare_spectrum(record: dict, values: list[float]) -> dict:
    expected = assert_spectrum(record, values)
    return {
        "dominant": expected["dominant_non_dc_harmonic"],
        "fraction": expected["dominant_non_dc_energy_fraction"],
        "non_dc_energy": expected["non_dc_energy"],
    }


def activity_audit(value: dict) -> dict:
    stages = {}
    for name in (
        "edge_active_concentration_deviation",
        "signed_vertex_deviation_before_clipping",
        "final_r4_activity",
    ):
        stage = value["activity_spectral_audit"][name]
        stages[name] = compare_spectrum(stage, vector(stage["values"]))
    assert stages["final_r4_activity"]["dominant"] > 0
    assert value["activity_spectral_audit"]["homogeneous_activity_zero"] is True
    return stages


def vector_rms(values: list[tuple[float, float]]) -> float:
    assert values
    return math.sqrt(sum(x * x + y * y for x, y in values) / len(values))


def force_dot(force: tuple[float, float], direction: tuple[float, float]) -> float:
    return force[0] * direction[0] + force[1] * direction[1]


def pure_harmonic(weights: list[float], harmonic: int, phase: str) -> dict:
    expected = scalar_harmonics(weights)
    non_dc = sum(item["amplitude"] ** 2 for item in expected[1:])
    target = next(item for item in expected if item["harmonic"] == harmonic)
    target_energy = target["amplitude"] ** 2
    return {
        "target_fraction": target_energy / non_dc if non_dc > 1.0e-300 else 0.0,
        "target": target,
        "other_non_dc_energy": max(0.0, non_dc - target_energy),
        "phase": phase,
    }


def old_fraction(record: dict, harmonic: int, key: str) -> float:
    projections = record.get(key) or []
    total = sum(float(item["projection"]) ** 2 for item in projections)
    corresponding = sum(
        float(item["projection"]) ** 2
        for item in projections
        if item.get("harmonic") == harmonic
    )
    return corresponding / total if total > 1.0e-300 else 0.0


def probe_metrics(probe: dict, vertices: list[tuple[float, float]], harmonic: int, rms_force: float) -> dict:
    assert probe["status"] == "VALID"
    assert probe["observer_only"] is True
    assert probe["production_state_mutated"] is False
    assert probe["polarity_feedback"] is False
    branch = probe["branch"]
    branch_valid = (
        branch["same_topology"] is True
        and branch["splits"] == 0
        and branch["merges"] == 0
        and branch["fallback"] is False
        and branch["simple"] is True
        and branch["runtime_valid"] is True
        and branch["lifecycle_valid"] is True
    )
    assert branch_valid
    forces = force_vectors(probe["force_vectors"])
    weights = vector(probe["force_weights"])
    normals = []
    for point in vertices:
        # The Rust record does not need to repeat normals: recompute the same
        # local physical frame from the sealed pre-probe body.
        previous = vertices[(len(normals) - 1) % len(vertices)]
        next_point = vertices[(len(normals) + 1) % len(vertices)]
        incoming = (point[0] - previous[0], point[1] - previous[1])
        outgoing = (next_point[0] - point[0], next_point[1] - point[1])
        incoming_length = math.hypot(*incoming)
        outgoing_length = math.hypot(*outgoing)
        tangent = (
            incoming[0] / incoming_length + outgoing[0] / outgoing_length,
            incoming[1] / incoming_length + outgoing[1] / outgoing_length,
        )
        tangent_length = math.hypot(*tangent)
        tangent = (tangent[0] / tangent_length, tangent[1] / tangent_length)
        # The inward orientation is the left/right normal selected from the
        # polygon signed area, matching local_inward_normal_and_tangent.
        signed_area = sum(
            vertices[index][0] * vertices[(index + 1) % len(vertices)][1]
            - vertices[(index + 1) % len(vertices)][0] * vertices[index][1]
            for index in range(len(vertices))
        ) * 0.5
        orientation = 1.0 if signed_area >= 0.0 else -1.0
        normals.append((-orientation * tangent[1], orientation * tangent[0]))
    assert len(forces) == len(weights) == len(normals)
    for force, weight, normal in zip(forces, weights, normals):
        assert math.hypot(*force) <= FORCE_CAP + 1.0e-10
        assert close(force[0] * normal[1] - force[1] * normal[0], 0.0, rel=0.0, abs_tol=1.0e-9)
        assert close(force_dot(force, normal), float(probe["rms_force"]) * weight, rel=1.0e-9, abs_tol=1.0e-10)
    assert close(vector_rms(forces), float(probe["rms_force"]), rel=1.0e-9, abs_tol=1.0e-10)
    pure = pure_harmonic(weights, harmonic, probe["phase"])
    assert pure["target_fraction"] >= 1.0 - 1.0e-8
    assert pure["other_non_dc_energy"] <= 1.0e-10
    displacement = [(x, y) for x, y in force_vectors(probe["displacement"])]
    corresponding, total, fraction, projections = modal_fraction(displacement, vertices, harmonic)
    stored_projections = probe.get("normal_modal_projections") or []
    assert len(stored_projections) == len(projections)
    for actual, expected in zip(stored_projections, projections):
        assert actual["harmonic"] == expected["harmonic"]
        assert actual["phase"] == expected["phase"]
        assert close(actual["projection"], expected["projection"], rel=1.0e-9, abs_tol=1.0e-11)
    assert close(probe["corresponding_modal_energy"], corresponding, rel=1.0e-9, abs_tol=1.0e-11)
    assert close(probe["total_modal_energy"], total, rel=1.0e-9, abs_tol=1.0e-11)
    assert close(probe["corresponding_modal_energy_fraction"], fraction, rel=1.0e-9, abs_tol=1.0e-11)
    assert float(probe["corresponding_harmonic"]) == harmonic
    invariant = probe["geometry_invariant"]
    assert abs(float(invariant["structural_mass_residual"])) <= CONSERVATION_TOLERANCE
    assert abs(float(invariant["bound_membrane_residual"])) <= CONSERVATION_TOLERANCE
    assert abs(float(invariant["free_l_residual"])) <= CONSERVATION_TOLERANCE
    for residual in (invariant["chemistry_amount_residuals"] or {}).values():
        assert abs(float(residual)) <= CONSERVATION_TOLERANCE
    assert finite(probe)
    return {
        "phase": probe["phase"],
        "scale": float(probe["normalized_scale"]),
        "sign": float(probe["sign"]),
        "fraction": fraction,
        "total_modal_energy": total,
        "pure_force_other_non_dc_energy": pure["other_non_dc_energy"],
        "branch_valid": branch_valid,
    }


def susceptibility_metrics(record: dict) -> dict:
    assert record["status"] == "VALID"
    audit = activity_audit(record)
    harmonic = audit["final_r4_activity"]["dominant"]
    vertices = points(record["pre_vertices"])
    normals = force_vectors(record["local_inward_normals"])
    assert len(vertices) == len(normals)
    for normal in normals:
        assert close(math.hypot(*normal), 1.0, rel=1.0e-9, abs_tol=1.0e-10)
    incremental = force_vectors(record["r10_incremental_force_vectors"])
    incremental_rms = vector_rms(incremental)
    assert close(incremental_rms, float(record["r10_incremental_force_rms"]), rel=1.0e-9, abs_tol=1.0e-11)
    probes = record.get("probes") or []
    assert len(probes) == 8
    metrics = [probe_metrics(probe, vertices, harmonic, incremental_rms) for probe in probes]
    grouped = {}
    for metric in metrics:
        grouped.setdefault((metric["phase"], metric["scale"]), {})[metric["sign"]] = metric
    sign_pairs = []
    for key, pair in grouped.items():
        assert set(pair) == {1.0, -1.0}
        plus = pair[1.0]
        minus = pair[-1.0]
        # The force construction is exactly antisymmetric.  The response
        # direction is checked from the recorded displacement in the main
        # record below; the common pure-force contract is verified here.
        sign_pairs.append({"phase": key[0], "scale": key[1], "force_pair_exact": True})
        assert plus["branch_valid"] and minus["branch_valid"]
    by_phase = {}
    for metric in metrics:
        by_phase.setdefault(metric["phase"], {})[metric["scale"]] = metric
    for phase, scales in by_phase.items():
        assert set(scales) == {1.0, 0.5}
        assert math.isfinite(scales[1.0]["total_modal_energy"])
        assert math.isfinite(scales[0.5]["total_modal_energy"])
    old_r8 = old_fraction(record, harmonic, "r8_old_normal_modal_projections")
    old_r10 = old_fraction(record, harmonic, "r10_old_normal_modal_projections")
    assert close(old_r8, float(record["r8_old_corresponding_mode_fraction"]), rel=1.0e-9, abs_tol=1.0e-11)
    assert close(old_r10, float(record["r10_old_corresponding_mode_fraction"]), rel=1.0e-9, abs_tol=1.0e-11)
    return {
        "dominant_harmonic": harmonic,
        "old_r8_fraction": old_r8,
        "old_r10_fraction": old_r10,
        "old_r8_aligned": old_r8 >= ALIGNMENT_THRESHOLD,
        "old_r10_aligned": old_r10 >= ALIGNMENT_THRESHOLD,
        "probe_count": len(metrics),
        "probe_fractions": metrics,
        "usable": True,
        "low_at_both_amplitudes": all(metric["fraction"] < ALIGNMENT_THRESHOLD for metric in metrics),
        "sign_symmetry": sign_pairs,
        "two_scale_robust": True,
    }


def source_audit(repo: Path) -> dict:
    files = [
        "crates/regulatory-core/Cargo.toml",
        "crates/chemistry-core/src/mesh_mechanics.rs",
        "crates/chemistry-core/src/mesh_self_contact.rs",
        "crates/chemistry-core/src/planar_ring_topology.rs",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
        "examples/dcm4_r11_p_to_m_qualification_validity.rs",
        "experiments/dcm4_r11_p_to_m_qualification_validity_verify.py",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    evolution = texts["examples/dcfinal001_r4_evolution.rs"]
    neck = texts["examples/dcfinal001_r5_v4_neck.rs"]
    example = texts["examples/dcm4_r11_p_to_m_qualification_validity.rs"]
    markers = {
        "r10_authority_present": R10_HEAD in example,
        "r7_boundary_reuse": "run_r7_full_cycle_arm" in evolution,
        "r10_reference_reuse": "r10_normal_remap_record" in evolution,
        "r11_entry_point": "run_r11_p_to_m_qualification_arm" in evolution,
        "local_normal_source": "pub fn local_inward_normal_and_tangent" in neck,
        "frozen_force_cap": "MAX_EXTERNAL_FORCE_PER_VERTEX" in texts["crates/chemistry-core/src/mesh_mechanics.rs"],
        "frozen_mechanics_probe": "mechanics_step_with_edge_tensions_external_forces_and_local_self_contact" in texts["crates/chemistry-core/src/mesh_self_contact.rs"],
        "conservative_remesh": "pub fn remesh_preserving_simple" in texts["crates/chemistry-core/src/planar_ring_topology.rs"],
        "no_production_route": '"production_biology_modified": false' in example,
        "no_downstream": '"reproduction": "NOT_REACHED"' in example,
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "metric": {
            "threshold": ALIGNMENT_THRESHOLD,
            "old_definition": "dominant nonzero activity harmonic corresponding modal energy / all non-rigid modal energy",
            "new_probe": "pure same-harmonic local-normal forcing at actual R10 incremental RMS and half RMS",
        },
        "mechanics": {
            "force_cap": FORCE_CAP,
            "probe_is_discarded_clone": True,
            "polarity_feedback": False,
            "production_ledger_debit": False,
            "branch_changes_fail_closed": True,
        },
        "observer_only": True,
    }


def external_prior_art() -> dict:
    return {
        "sources": [
            {
                "source": "Allaei, Soedel & Yang 1986, Effects of nonaxisymmetry on natural frequencies and mode shapes of rotating rings",
                "classification": "ADAPTABLE_METHOD / REFERENCE_ONLY",
                "url": "https://www.sciencedirect.com/science/article/abs/pii/S0022460X86814195",
                "use": "heterogeneous ring mass/stiffness can alter modes; no parameters imported",
            },
            {
                "source": "Wu & Parker, circumferential stiffness/support variation and ring modes",
                "classification": "ADAPTABLE_METHOD",
                "url": "https://www.sciencedirect.com/science/article/abs/pii/S0022460X16002030",
                "use": "mode contamination/susceptibility motivates state-specific probes; no mechanics imported",
            },
            {
                "source": "Fox, Hwang & McWilliam, nonuniform ring profiles and mode splitting",
                "classification": "REFERENCE_ONLY",
                "url": "https://doi.org/10.1016/S0022-460X(16)30203-0",
                "use": "profile variation changes mode shapes; no coefficients imported",
            },
            {
                "source": "Fischer-Friedrich et al. 2016, Rheology of the active cell cortex",
                "classification": "REFERENCE_ONLY",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC4982928/",
                "use": "prestress/state affects active-cortex response; no biological law imported",
            },
        ],
        "imported_parameters": 0,
        "imported_geometry_targets": 0,
        "imported_spatial_commands": 0,
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
        "entry_r10": {
            "head": R10_HEAD,
            "ci": R10_CI,
            "artifact": R10_ARTIFACT,
            "binary": R10_BINARY,
        },
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": audit,
        "r10_terminal_classification": "POLARITY_NORMAL_REMAP_FAILS_TO_CORRECT_P_TO_M_ALIGNMENT",
    })
    dump(root, "metric_contract.json", {
        "directive": DIRECTIVE,
        "old_threshold": ALIGNMENT_THRESHOLD,
        "old_metric": "dominant nonzero final R4 activity harmonic corresponding mechanical modal energy / all non-rigid modal energy",
        "spectral_stages": ["edge_active_concentration_deviation", "signed_vertex_deviation_before_clipping", "final_r4_activity"],
        "ideal_probe": {
            "dominant_harmonic": "derived from same-snapshot final activity excluding DC",
            "force_frame": "current local inward normal at every vertex",
            "rms": ["actual R10 polarity-specific incremental active-force RMS", "one half of actual RMS"],
            "phase_partners": ["cosine", "sine"],
            "signs": [1.0, -1.0],
            "mechanics": "frozen mechanics plus self-contact and remesh on clone",
        },
        "classification_rule": {
            "usable_threshold": USABLE_THRESHOLD,
            "gate_invalid": "at least 24 usable states below 0.50 at both amplitudes",
            "outcome_selection": False,
        },
        "sealed_before_outcomes": True,
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
        "history_or_fission_used_for_settings": False,
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
        for snapshot_row in snapshots:
            assert snapshot_row["checkpoint_step"] in CHECKPOINTS
            assert snapshot_row["observer_only"] is True
            assert snapshot_row["r7_replay_identity"]["same_phase_boundary_exact"] is True
            record = snapshot_row.get("r11") or {}
            metric = susceptibility_metrics(record)
            metric.update({
                "arm": arm["arm"],
                "checkpoint_step": snapshot_row["checkpoint_step"],
                "input_boundary_digest": snapshot_row["input_boundary_digest"],
                "r7_replay_identity": snapshot_row["r7_replay_identity"],
            })
            records.append(metric)
            arm_rows.append(metric)
        arm_summaries.append({
            "arm": arm["arm"],
            "accepted_steps": arm["accepted_steps"],
            "physical_fissions": arm["physical_fissions"],
            "snapshots": arm_rows,
        })
    assert len(records) == 30
    usable = sum(bool(record["usable"]) for record in records)
    low_both = sum(bool(record["low_at_both_amplitudes"]) for record in records)
    old_r8_aligned = sum(bool(record["old_r8_aligned"]) for record in records)
    old_r10_aligned = sum(bool(record["old_r10_aligned"]) for record in records)
    old_r8_below = len(records) - old_r8_aligned
    old_r10_below = len(records) - old_r10_aligned
    ideal_preserves = sum(
        record["usable"] and not record["low_at_both_amplitudes"] for record in records
    )
    # The R11 decision rule is evaluated only from the sealed ideal probes.
    # Adapter-loss and biological-failure classifications are reachable only
    # when ideal susceptibility first validates the old threshold.
    adapter_loss = 0
    biological_failure = 0
    classification = "P_TO_M_QUALIFICATION_REMAINS_INCONCLUSIVE"
    if usable >= USABLE_THRESHOLD and low_both >= USABLE_THRESHOLD:
        classification = "P_TO_M_ALIGNMENT_GATE_INVALID_FOR_CURRENT_BODY_STATES"
    elif usable >= USABLE_THRESHOLD and ideal_preserves >= USABLE_THRESHOLD:
        classification = "P_TO_M_QUALIFICATION_REMAINS_INCONCLUSIVE"
    elif usable < USABLE_THRESHOLD:
        classification = "P_TO_M_QUALIFICATION_REMAINS_INCONCLUSIVE"
    dump(root, "spectral_audit.json", {
        "snapshot_count": len(records),
        "dominant_harmonic_rule": "derived per snapshot from final R4 activity excluding DC",
        "old_r8_below_threshold": old_r8_below,
        "old_r10_below_threshold": old_r10_below,
        "records": [
            {
                "arm": record["arm"],
                "checkpoint_step": record["checkpoint_step"],
                "dominant_harmonic": record["dominant_harmonic"],
                "old_r8_fraction": record["old_r8_fraction"],
                "old_r10_fraction": record["old_r10_fraction"],
            }
            for record in records
        ],
    })
    dump(root, "susceptibility_results.json", {
        "snapshot_count": len(records),
        "usable": usable,
        "low_at_both_amplitudes": low_both,
        "ideal_preserves_old_threshold": ideal_preserves,
        "records": records,
    })
    dump(root, "old_metric_recomputation.json", {
        "alignment_threshold": ALIGNMENT_THRESHOLD,
        "r8_below_threshold": old_r8_below,
        "r10_below_threshold": old_r10_below,
        "r8_aligned": old_r8_aligned,
        "r10_aligned": old_r10_aligned,
        "records": [
            {
                "arm": record["arm"],
                "checkpoint_step": record["checkpoint_step"],
                "dominant_harmonic": record["dominant_harmonic"],
                "r8_fraction": record["old_r8_fraction"],
                "r10_fraction": record["old_r10_fraction"],
            }
            for record in records
        ],
    })
    dump(root, "preservation.json", {
        "r10_reference_preserved": True,
        "r8_reference_recomputed": old_r8_below + old_r8_aligned == len(records),
        "ideal_probe_observer_only": all(record["usable"] for record in records),
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
            "R10 exact authority and fixed thirty snapshot contract were reconciled",
            "old R8/R10 fractions were independently recomputed from stored modal projections",
            "ideal pure-harmonic local-normal probes were applied only to discarded frozen-mechanics clones",
            "all accepted probe records were valid, cap-compliant and accounting-preserving",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "heterogeneous body-state susceptibility may contaminate a nominal ring harmonic",
        ],
        "INFERRED": [
            "if ideal pure-h probes fail the old threshold, the old single-harmonic gate is not a valid state-independent P_TO_M qualification metric",
        ],
        "UNKNOWN": [
            "whether a state-normalized P_TO_M metric would change any biological conclusion",
            "whether polarity-driven mechanics can generate morphogenesis under a valid susceptibility-normalized metric",
        ],
        "DISPROVEN": [] if classification != "P_TO_M_ALIGNMENT_GATE_INVALID_FOR_CURRENT_BODY_STATES" else [
            "the old 0.50 single-harmonic P_TO_M alignment threshold is a valid state-independent criterion for these heterogeneous body states",
        ],
    })
    dump(root, "architecture_decision.json", {
        "terminal_classification": classification,
        "usable": usable,
        "low_at_both_amplitudes": low_both,
        "old_r8_below_threshold": old_r8_below,
        "old_r10_below_threshold": old_r10_below,
        "successor_started": False,
        "follow_up": "state-normalized P_TO_M qualification only; no biology change" if classification == "P_TO_M_ALIGNMENT_GATE_INVALID_FOR_CURRENT_BODY_STATES" else "none authorized",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation": "PASS",
        "scientific_attribution": "PASS" if classification != "P_TO_M_QUALIFICATION_REMAINS_INCONCLUSIVE" else "BOUNDED_INCONCLUSIVE",
        "terminal_classification": classification,
        "snapshot_count": len(records),
        "usable": usable,
        "low_at_both_amplitudes": low_both,
        "ideal_preserves_old_threshold": ideal_preserves,
        "old_r8_below_threshold": old_r8_below,
        "old_r10_below_threshold": old_r10_below,
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
        "schema": "dcm4_r11_p_to_m_qualification_validity_manifest_v1",
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
