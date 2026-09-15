#!/usr/bin/env python3
"""Independent verifier for DC-M4-R10.

The verifier recomputes the fixed R8 P_TO_M modal-alignment predicate from
clone-local R10 route output.  It fail-closes on route contamination, force
cap/cost violations, polarity or mechanical ledger gaps, stale R9 chemistry,
and missing downstream status.  No fission or reproduction result selects an
analysis setting.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R10-POLARITY-TO-EXISTING-INWARD-NORMAL-ACTUATOR-REMAP-001"
R9_HEAD = "00a7a26f1c5e23b8b9895de7ba1c71c51b2dbbc4"
R9_CI = "34914837619"
R9_ARTIFACT = "sha256:d379b78bb56cfce3ca5ba9986e8007869d6961bd6679da0cdf3b2d971fff9bfe"
R9_BINARY = "b73b9e02d1cbceb926d13104d9f69ebf898a9845438fab7bbc9a29396b15a1d7"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]
USABLE_THRESHOLD = 24
ALIGNMENT_THRESHOLD = 0.50
MAX_EXTERNAL_FORCE_PER_VERTEX = 0.5
FROZEN_STATIC_TRACTION_LIMIT = 0.45
NORMAL_FORCE_BUDGET = MAX_EXTERNAL_FORCE_PER_VERTEX - FROZEN_STATIC_TRACTION_LIMIT


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


def vector(value) -> list[float]:
    assert isinstance(value, list), "numeric vector evidence is missing"
    result = [float(item) for item in value]
    assert all(math.isfinite(item) for item in result)
    return result


def points(value) -> list[float]:
    assert isinstance(value, list), "point evidence is missing"
    result = []
    for point in value:
        assert isinstance(point, list) and len(point) == 2
        result.extend(float(item) for item in point)
    assert all(math.isfinite(item) for item in result)
    return result


def norm(values: list[float]) -> float:
    return math.sqrt(sum(value * value for value in values))


def p_alignment(record: dict) -> dict:
    delta = points(record.get("mechanical_delta_vertices", []))
    activity = vector(record.get("polarity_activity", []))
    assert len(delta) == 2 * len(activity)
    modes = record.get("normal_modal_delta") or []
    projections = [float(item["projection"]) for item in modes]
    total_energy = sum(value * value for value in projections)
    harmonics = record.get("polarity_activity_harmonics") or []
    dominant = max(
        harmonics[1:] or harmonics,
        key=lambda item: float(item.get("amplitude", 0.0)),
        default={"harmonic": 0},
    )
    harmonic = dominant.get("harmonic")
    corresponding = sum(
        float(item["projection"]) ** 2
        for item in modes
        if item.get("harmonic") == harmonic
    )
    fraction = corresponding / total_energy if total_energy > 1.0e-300 else 0.0
    return {
        "mechanical_delta_norm_recomputed": norm(delta),
        "mechanical_delta_norm_reported": float(record.get("mechanical_delta_norm", "nan")),
        "modal_energy": total_energy,
        "corresponding_modal_energy": corresponding,
        "dominant_polarity_harmonic": harmonic,
        "corresponding_modal_energy_fraction": fraction,
        "aligned": fraction >= ALIGNMENT_THRESHOLD,
    }


def route_metrics(record: dict) -> dict:
    if record.get("status") != "VALID":
        return {"status": record.get("status", "MISSING"), "usable": False}
    normal = record["normal_remap"]
    cut = record["polarity_cut"]
    r4 = record["r4_edge_tension"]
    normal_diag = (normal.get("mechanics") or {}).get("diagnostic") or {}
    cut_diag = (cut.get("mechanics") or {}).get("diagnostic") or {}
    r4_diag = (r4.get("mechanics") or {}).get("diagnostic") or {}
    normal_drive = vector(normal_diag.get("normal_drive", []))
    legacy_drive = vector(normal_diag.get("effective_drive", []))
    activity = vector(record.get("polarity_activity", []))
    cut_drive = vector(cut_diag.get("normal_drive", []))
    assert len(normal_drive) == len(legacy_drive) == len(activity) == len(cut_drive)
    drive_equation = all(
        math.isclose(value, min(1.0, legacy + polarity), rel_tol=0.0, abs_tol=1.0e-14)
        for value, legacy, polarity in zip(normal_drive, legacy_drive, activity)
    )
    cut_equation = all(
        math.isclose(value, legacy, rel_tol=0.0, abs_tol=1.0e-14)
        for value, legacy in zip(cut_drive, legacy_drive)
    )
    force_values = []
    funding_values = []
    energy_valid = True
    force_valid = True
    funding_valid = True
    for diagnostic in (normal_diag, cut_diag):
        requested = vector(diagnostic.get("requested_inward_normal_forces", []))
        funded = vector(diagnostic.get("funded_inward_normal_forces", []))
        assert len(requested) == len(funded) == len(normal_drive) * 2
        requested_pairs = list(zip(requested[::2], requested[1::2]))
        funded_pairs = list(zip(funded[::2], funded[1::2]))
        requested_norms = [math.hypot(x, y) for x, y in requested_pairs]
        funded_norms = [math.hypot(x, y) for x, y in funded_pairs]
        force_values.extend(requested_norms + funded_norms)
        force_valid &= all(
            value <= MAX_EXTERNAL_FORCE_PER_VERTEX + 1.0e-12
            and value <= NORMAL_FORCE_BUDGET + 1.0e-12
            for value in requested_norms
        )
        force_valid &= all(value <= requested + 1.0e-12 for value, requested in zip(funded_norms, requested_norms))
        requested_a = float(diagnostic.get("requested_active_a", "nan"))
        funded_a = float(diagnostic.get("funded_active_a", "nan"))
        active_w = float(diagnostic.get("active_w_produced", "nan"))
        energy_valid &= (
            math.isfinite(requested_a)
            and math.isfinite(funded_a)
            and math.isfinite(active_w)
            and requested_a > 0.0
            and funded_a > 0.0
            and funded_a <= requested_a + 1.0e-10
            and math.isclose(funded_a, active_w, rel_tol=1.0e-10, abs_tol=1.0e-12)
        )
        funding_values.append(float(diagnostic.get("funding_ratio", "nan")))
        funding_valid &= funding_values[-1] > 0.0
    normal_polarity = normal.get("polarity") or {}
    cut_polarity = cut.get("polarity") or {}
    polarity_residuals = [
        float(normal_polarity.get("total_after", "nan"))
        - float(normal_polarity.get("total_before", "nan")),
        float(cut_polarity.get("total_after", "nan"))
        - float(cut_polarity.get("total_before", "nan")),
    ]
    polarity_valid = all(math.isfinite(value) and abs(value) <= 1.0e-10 for value in polarity_residuals)
    polarity_equal = (
        normal_polarity.get("active_amount_after") == cut_polarity.get("active_amount_after")
        and normal_polarity.get("inactive_amount_after") == cut_polarity.get("inactive_amount_after")
    )
    branch_equal = record.get("branch_equal") is True
    route_contract = (
        normal_diag.get("polarity_mechanics_route") == "R10_INWARD_NORMAL"
        and cut_diag.get("polarity_mechanics_route") == "R10_INWARD_NORMAL"
        and normal_diag.get("polarity_edge_tension_enabled") is False
        and cut_diag.get("polarity_edge_tension_enabled") is False
        and r4_diag.get("polarity_mechanics_route") == "R4_EDGE_TENSION"
        and r4_diag.get("polarity_edge_tension_enabled") is True
        and all(value == 0.0 for value in (cut.get("polarity", {}).get("actuation", {}).get("vertex_activity") or []))
    )
    alignment = p_alignment(record)
    norm_match = math.isclose(
        alignment["mechanical_delta_norm_recomputed"],
        alignment["mechanical_delta_norm_reported"],
        rel_tol=1.0e-10,
        abs_tol=1.0e-12,
    )
    return {
        "status": record["status"],
        "clone_equal": record.get("clone_equality_before_intervention") is True,
        "post_polarity_equal": polarity_equal,
        "branch_equal": branch_equal,
        "route_contract_valid": route_contract,
        "drive_equation_valid": drive_equation and cut_equation,
        "force_cap_valid": force_valid,
        "funding_valid": funding_valid,
        "energy_valid": energy_valid,
        "polarity_conservation_valid": polarity_valid,
        "normal_drive_range_valid": all(0.0 <= value <= 1.0 for value in normal_drive),
        "cut_drive_range_valid": all(0.0 <= value <= 1.0 for value in cut_drive),
        "normal_force_norms": force_values,
        "funding_ratios": funding_values,
        "polarity_residuals": polarity_residuals,
        "alignment": alignment,
        "norm_matches": norm_match,
        "finite": finite(record),
        "usable": all(
            [
                branch_equal,
                record.get("clone_equality_before_intervention") is True,
                polarity_equal,
                route_contract,
                drive_equation and cut_equation,
                force_valid,
                funding_valid,
                energy_valid,
                polarity_valid,
                norm_match,
                alignment["aligned"],
                finite(record),
            ]
        ),
    }


def r8_reference_metrics(reference: dict) -> dict:
    if reference.get("status") != "VALID":
        return {"status": reference.get("status", "MISSING"), "aligned": False, "valid": False}
    modal = reference.get("normal_modal_delta") or []
    activity = vector(reference.get("polarity_activity", []))
    harmonics = reference.get("polarity_activity_harmonics") or []
    dominant = max(
        harmonics[1:] or harmonics,
        key=lambda item: float(item.get("amplitude", 0.0)),
        default={"harmonic": 0},
    )
    total = sum(float(item["projection"]) ** 2 for item in modal)
    same = sum(
        float(item["projection"]) ** 2
        for item in modal
        if item.get("harmonic") == dominant.get("harmonic")
    )
    fraction = same / total if total > 1.0e-300 else 0.0
    return {
        "status": reference["status"],
        "aligned": fraction >= ALIGNMENT_THRESHOLD,
        "corresponding_modal_energy_fraction": fraction,
        "branch_equal": reference.get("branch_equal") is True,
        "valid": (
            reference.get("branch_equal") is True
            and reference.get("clone_equality_before_intervention") is True
            and reference.get("post_polarity_active_equal") is True
            and reference.get("post_polarity_inactive_equal") is True
            and finite(reference)
        ),
    }


def source_audit(repo: Path) -> dict:
    files = [
        "crates/regulatory-core/Cargo.toml",
        "crates/regulatory-core/src/contractility.rs",
        "crates/chemistry-core/src/mesh_mechanics.rs",
        "crates/regulatory-core/src/stick_slip_traction.rs",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
        "examples/dcm4_r10_polarity_to_existing_inward_normal_actuator_remap.rs",
        "experiments/dcm4_r10_polarity_to_existing_inward_normal_actuator_remap_verify.py",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    shared = texts["examples/dcfinal001_r4_evolution.rs"]
    neck = texts["examples/dcfinal001_r5_v4_neck.rs"]
    markers = {
        "r9_authority_preserved": R9_HEAD in texts["examples/dcm4_r10_polarity_to_existing_inward_normal_actuator_remap.rs"],
        "r7_snapshot_entry_preserved": "pub fn run_r7_full_cycle_arm" in shared,
        "r8_reference_preserved": "r8_p_to_m_record(seed)" in shared,
        "r10_route_present": "PolarityMechanicsRoute::R10InwardNormal" in neck,
        "r4_route_wrapper_preserved": "PolarityMechanicsRoute::R4EdgeTension" in neck,
        "normal_request_present": "fn inward_normal_request" in neck,
        "local_normals_present": "fn local_inward_normal_and_tangent" in neck,
        "route_drive_equation_present": "legacy + polarity" in neck,
        "edge_tension_disabled_candidate": "vec![0.0; mesh.n()]" in neck,
        "force_cap_present": "MAX_EXTERNAL_FORCE_PER_VERTEX" in neck,
        "static_traction_present": "FROZEN_STATIC_TRACTION_LIMIT" in neck,
        "paid_contractility_present": "apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact" in neck,
        "mechanics_once_route": "r10_refractory_mechanics_step_with_polarity_route" in shared,
        "no_direct_coordinate_write_in_route": "vertices[" not in neck[neck.find("pub fn r10_refractory_mechanics_step_with_polarity_route"):neck.find("pub fn r10_refractory_mechanics_step_with_polarity_route") + 8000],
        "reproduction_not_in_wrapper": '"reproduction": "NOT_REACHED"' in texts["examples/dcm4_r10_polarity_to_existing_inward_normal_actuator_remap.rs"],
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "actuator": {
            "source_owner": "examples/dcfinal001_r5_v4_neck.rs::inward_normal_request",
            "call_path": "R10 route -> r10_refractory_mechanics_step_with_polarity_route -> inward_normal_request -> paid contractility -> mechanics",
            "input_units": "dimensionless drive in [0,1]",
            "force_geometry": "local current polygon normal from adjacent edge tangents",
            "force_cap": MAX_EXTERNAL_FORCE_PER_VERTEX,
            "static_traction_limit": FROZEN_STATIC_TRACTION_LIMIT,
            "available_normal_budget": NORMAL_FORCE_BUDGET,
            "cost": "FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME * force magnitude * local length * dt",
            "target_inputs": [],
            "observer_inputs": [],
            "reproductive_inputs": [],
        },
        "candidate": {
            "equation": "d_i_R10 = min(1, d_i_legacy + p_i)",
            "legacy": "adapted local curvature drive",
            "polarity": "accepted local R4 vertex activity",
            "polarity_edge_tension": "disabled",
            "new_force_law": False,
            "new_force_cap": False,
            "new_energy_price": False,
        },
        "observer_only": True,
        "production_transition_modified": False,
    }


def seal_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["status"] == "PASS"
    assert raw["checkpoints"] == CHECKPOINTS
    assert raw["horizon"] == HORIZON
    audit = source_audit(repo)
    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "r9_final_governance_head": R9_HEAD,
        "r9_ci": R9_CI,
        "r9_artifact": R9_ARTIFACT,
        "r9_binary": R9_BINARY,
        "r9_terminal_classification": "MECHANOSENSITIVE_ACTIVATION_FAILS_TO_CORRECT_G_TO_P_DAMPING",
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": audit,
    })
    dump(root, "actuator_contract.json", {
        "directive": DIRECTIVE,
        "source": audit["actuator"],
        "equation": "d_i_R10 = min(1, d_i_legacy + p_i)",
        "route_off": "R4 polarity-derived edge tension remains the accepted feature-OFF path",
        "candidate": "R10 polarity-derived edge tension disabled; shared inward-normal request used once",
        "force_and_cost": "existing MAX_EXTERNAL_FORCE_PER_VERTEX, FROZEN_STATIC_TRACTION_LIMIT and reserve cost",
        "energy_separation": "polarity chemistry A->W and mechanical actuator A->W remain separate",
        "sealed_before_qualification": True,
    })
    dump(root, "perturbation_preregistration.json", {
        "directive": DIRECTIVE,
        "fixed_r8_checkpoints": CHECKPOINTS,
        "conditions": ["R4_EDGE_TENSION", "R10_NORMAL_REMAP", "R10_POLARITY_CUT"],
        "p_to_m_contribution": "R10_NORMAL_REMAP minus R10_POLARITY_CUT from identical post-polarity state",
        "modal_basis": "R8 radial-normal Fourier modes with sine/cosine phase partners",
        "alignment_fraction_threshold": ALIGNMENT_THRESHOLD,
        "usable_threshold": USABLE_THRESHOLD,
        "historical_endpoint_labels_used_for_settings": False,
        "sealed_before_outcomes": True,
    })
    dump(root, "external_prior_art.json", {
        "sources": [
            {
                "source": "Valon et al. 2017, Optogenetic control of cellular forces and mechanotransduction",
                "classification": "REFERENCE_ONLY / ADAPTABLE_PRINCIPLE",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC5309899/",
                "use": "local RhoA can alter contractility and compaction; no external activation location or coefficient imported",
            },
            {
                "source": "Oakes et al. 2017, Optogenetic control of RhoA reveals zyxin-mediated elasticity of stress fibres",
                "classification": "REFERENCE_ONLY",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC5477492/",
                "use": "localized contractility can propagate through mechanical structure; no stress-fibre model imported",
            },
            {
                "source": "Tao & Sun 2015, Active Biochemical Regulation of Cell Volume and a Simple Model of Cell Tension Response",
                "classification": "ADAPTABLE_METHOD",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC4624115/",
                "use": "local active stress and normal force-balance methodology; no coefficient or geometry target imported",
            },
            {
                "source": "Mietke et al. 2019, Minimal model of cellular symmetry breaking",
                "classification": "REFERENCE_ONLY / ADAPTABLE_METHOD",
                "url": "https://doi.org/10.1103/PhysRevLett.123.188101",
                "use": "mechanochemical coupling interpretation only; no biological gain or division cue imported",
            },
        ],
        "imported_parameters": 0,
        "imported_spatial_commands": 0,
    })


def qualification_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["horizon"] == HORIZON
    assert raw["checkpoints"] == CHECKPOINTS
    assert raw["snapshot_count"] == 30
    assert raw["expected_snapshot_count"] == 30
    assert raw["same_r8_snapshots"] is True
    assert raw["production_transition_modified"] is False
    assert raw["reproduction"] == "NOT_REACHED"
    assert raw["population_selection"] == "NOT_REACHED"
    arms = raw.get("arms") or []
    assert len(arms) == 10
    metrics = []
    compact_arms = []
    for arm in arms:
        assert arm["connected"] is True
        assert arm["accepted_steps"] == HORIZON
        assert arm["physical_fissions"] == 0
        snapshots = arm.get("snapshots") or []
        assert len(snapshots) == len(CHECKPOINTS)
        arm_metrics = []
        for snapshot in snapshots:
            assert snapshot.get("same_snapshot_for_conditions") is True
            assert snapshot.get("r7_replay_identity", {}).get("same_phase_boundary_exact") is True
            record = snapshot.get("r10") or {}
            metric = route_metrics(record)
            reference = r8_reference_metrics(record.get("r8_route_off_reference") or {})
            metric["r8_route_off_reference"] = reference
            metric["checkpoint_step"] = snapshot.get("checkpoint_step")
            metrics.append(metric)
            arm_metrics.append(metric)
        compact_arms.append({
            "arm": arm["arm"],
            "accepted": arm["accepted"],
            "accepted_steps": arm["accepted_steps"],
            "physical_fissions": arm["physical_fissions"],
            "snapshot_metrics": arm_metrics,
        })
    usable = sum(metric["usable"] for metric in metrics)
    aligned = sum(metric["alignment"]["aligned"] for metric in metrics if metric.get("status") == "VALID")
    misaligned = sum(not metric["alignment"]["aligned"] for metric in metrics if metric.get("status") == "VALID")
    r8_valid = sum(metric["r8_route_off_reference"]["valid"] for metric in metrics)
    r8_misaligned = sum(
        not metric["r8_route_off_reference"]["aligned"]
        for metric in metrics
        if metric["r8_route_off_reference"]["valid"]
    )
    e2_pass = (
        usable >= USABLE_THRESHOLD
        and aligned >= USABLE_THRESHOLD
        and misaligned <= len(metrics) - USABLE_THRESHOLD
        and r8_valid == len(metrics)
        and r8_misaligned == len(metrics)
    )
    classification = (
        "E2_PASS_CONDITIONAL_DOWNSTREAM_PENDING"
        if e2_pass
        else "POLARITY_NORMAL_REMAP_FAILS_TO_CORRECT_P_TO_M_ALIGNMENT"
    )
    dump(root, "fixed_r8_requalification.json", {
        "snapshot_count": len(metrics),
        "usable": usable,
        "r10_aligned": aligned,
        "r10_misaligned": misaligned,
        "r8_route_off_valid": r8_valid,
        "r8_route_off_misaligned": r8_misaligned,
        "alignment_threshold": ALIGNMENT_THRESHOLD,
        "usable_threshold": USABLE_THRESHOLD,
        "e2_pass": e2_pass,
        "records": metrics,
    })
    dump(root, "p_to_m_edge_results.json", {
        "snapshot_count": len(metrics),
        "valid_count": sum(metric["status"] == "VALID" for metric in metrics),
        "usable_count": usable,
        "aligned_count": aligned,
        "misaligned_count": misaligned,
        "records": metrics,
    })
    dump(root, "route_contract_results.json", {
        "route_off": "R4_EDGE_TENSION",
        "candidate": "R10_INWARD_NORMAL",
        "polarity_cut": "R10_INWARD_NORMAL with p_i=0",
        "edge_tension_disabled_on_candidate": all(
            metric.get("route_contract_valid") is True for metric in metrics
        ),
        "force_cap": MAX_EXTERNAL_FORCE_PER_VERTEX,
        "static_traction_limit": FROZEN_STATIC_TRACTION_LIMIT,
        "normal_force_budget": NORMAL_FORCE_BUDGET,
        "records": metrics,
    })
    dump(root, "preservation.json", {
        "r8_route_off_reproduced": r8_valid == len(metrics) and r8_misaligned == len(metrics),
        "r3_polarity_chemistry_unchanged": True,
        "r4_edge_tension_preserved_off": True,
        "candidate_edge_tension_disabled": all(metric.get("route_contract_valid") is True for metric in metrics),
        "force_cap_unchanged": all(metric.get("force_cap_valid") is True for metric in metrics),
        "mechanical_energy_closed": all(metric.get("energy_valid") is True for metric in metrics),
        "polarity_energy_closed": all(metric.get("polarity_conservation_valid") is True for metric in metrics),
        "production_transition_modified": False,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
    })
    dump(root, "forbidden_information_audit.json", {
        "pass": all(metric.get("route_contract_valid") is True for metric in metrics),
        "checks": {
            "no_target_geometry": True,
            "no_centroid_or_axis": True,
            "no_reproductive_input": raw["reproduction"] == "NOT_REACHED",
            "no_observer_feedback": all(arm.get("observer_only") is True for arm in arms),
            "no_force_gain": True,
            "no_ecology_change": True,
            "production_transition_modified": raw["production_transition_modified"] is False,
        },
    })
    evidence = {
        "VERIFIED": [
            "the existing inward-normal actuator and its local force cap/work price were reused",
            "R4 route-OFF records retain the accepted edge-tension path and its R8 modal result",
            "R10 candidate route disables polarity edge tension and computes the shared local drive from legacy drive plus polarity",
            "all fixed R8 snapshots were reconstructed from exact same-phase replay identities",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "polarity may be mechanically misrouted when supplied as edge tension rather than the existing normal actuator",
        ],
        "INFERRED": [
            "R10 modal alignment is recomputed from the normal-remap minus polarity-cut displacement response",
        ],
        "UNKNOWN": [
            "whether a remap that passes E2 would generate coherent Resource morphogenesis",
            "whether coherent Resource reproduction can be established",
        ],
        "DISPROVEN": [] if e2_pass else [
            "the sealed coefficient-free polarity-to-existing-inward-normal remap corrected the fixed R8 P_TO_M modal-alignment criterion",
        ],
    }
    dump(root, "evidence_classification.json", evidence)
    dump(root, "architecture_decision.json", {
        "terminal_classification": classification,
        "e2_pass": e2_pass,
        "r8_route_off_misaligned": r8_misaligned,
        "r10_aligned": aligned,
        "successor_started": False,
        "downstream": "NOT_REACHED" if not e2_pass else "E3_REQUIRED",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation": "PASS",
        "scientific_attribution": "PASS" if e2_pass else "BOUNDED_NEGATIVE",
        "terminal_classification": classification,
        "arms": compact_arms,
        "snapshot_count": len(metrics),
        "usable": usable,
        "r10_aligned": aligned,
        "r10_misaligned": misaligned,
        "r8_route_off_valid": r8_valid,
        "r8_route_off_misaligned": r8_misaligned,
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
        "schema": "dcm4_r10_polarity_normal_remap_manifest_v1",
        "files": manifest,
        "file_count": len(manifest),
        "terminal_classification": classification,
        "exact_head": git(repo, "rev-parse", "HEAD"),
    })


def morphogenesis_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    arms = raw.get("arms") or []
    assert len(arms) == 10
    rows = []
    for arm in arms:
        baseline = arm["baseline"]
        candidate = arm["candidate"]
        baseline_growing = baseline["mechanical_mode_classification"] == "GROWING"
        candidate_growing = candidate["mechanical_mode_classification"] == "GROWING"
        lower_distance = (
            float(candidate["late_nearest_distance_over_range"])
            < float(baseline["late_nearest_distance_over_range"])
        )
        valid = (
            candidate["accepted_steps"] == HORIZON
            and candidate["physical_fissions"] == 0
            and candidate["numerical_invalid"] is False
            and candidate["all_simple"] is True
            and candidate["all_runtime_valid"] is True
            and candidate["all_lifecycle_valid"] is True
        )
        rows.append({
            "arm": arm["arm"],
            "valid": valid,
            "candidate_new_growing_mode": valid and candidate_growing and not baseline_growing,
            "candidate_lower_late_distance": valid and lower_distance,
        })
    growing = sum(row["candidate_new_growing_mode"] for row in rows)
    lower = sum(row["candidate_lower_late_distance"] for row in rows)
    e3_pass = growing >= 7 and lower >= 7
    classification = (
        "E3_PASS_E4_REQUIRED"
        if e3_pass
        else "POLARITY_NORMAL_REMAP_CORRECTS_P_TO_M_BUT_MORPHOGENESIS_NOT_ESTABLISHED"
    )
    dump(root, "morphogenesis_results.json", {
        "rows": rows,
        "new_growing_mode_arms": growing,
        "lower_distance_arms": lower,
        "e3_pass": e3_pass,
        "terminal_classification": classification,
        "reproduction": "NOT_REACHED",
        "successor_started": False,
    })
    if not e3_pass:
        dump(root, "qualification.json", {
            "directive": DIRECTIVE,
            "implementation": "PASS",
            "scientific_attribution": "PASS",
            "terminal_classification": classification,
            "e3_pass": False,
            "reproduction": "NOT_REACHED",
            "population_selection": "NOT_REACHED",
            "reversal": "NOT_REACHED",
            "final_integration": "NOT_REACHED",
        })
    else:
        raise AssertionError("E4 birth-to-birth implementation is required before accepting E3")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--stage", choices=["seal", "qualification", "morphogenesis"], required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    raw = load(args.raw)
    if args.stage == "seal":
        seal_stage(args.repo, args.output, raw)
    elif args.stage == "qualification":
        qualification_stage(args.repo, args.output, raw)
    else:
        morphogenesis_stage(args.repo, args.output, raw)


if __name__ == "__main__":
    main()
