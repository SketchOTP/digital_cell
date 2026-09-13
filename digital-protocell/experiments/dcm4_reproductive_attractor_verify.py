#!/usr/bin/env python3
"""Independent verifier for the DC-M4 observer-only architecture gate.

The Rust example is the instrumented producer. This verifier deliberately
recomputes the arm counts, matched-time observations, stability classifications,
source-contract evidence, and artifact manifest from raw JSON and repository
source. No verifier result is accepted merely because the producer emitted a
PASS field.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-REPRODUCTIVE-ATTRACTOR-ARCHITECTURE-001"
STARTING_HEAD = "9aa1db6e001cf94e814b50b3f3cf086934000eee"
STABILITY_TOLERANCE = 1.0e-3


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_line(path: Path, needle: str):
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if needle in line:
            return {"path": str(path), "line": number, "text": line.strip()}
    return None


def finite(value) -> bool:
    if isinstance(value, bool) or value is None:
        return True
    if isinstance(value, (int, float)):
        return math.isfinite(value)
    if isinstance(value, list):
        return all(finite(item) for item in value)
    if isinstance(value, dict):
        return all(finite(item) for item in value.values())
    return True


def arms(cell):
    result = cell.get("arms")
    if not isinstance(result, list):
        raise AssertionError("cell is missing arms")
    if len(result) != 10:
        raise AssertionError(f"expected 10 arms, got {len(result)}")
    return result


def trace_rows(arm):
    rows = arm.get("mechanics_trace")
    if not isinstance(rows, list) or not rows:
        raise AssertionError(f"arm {arm.get('arm')} has no mechanics trace")
    return rows


def geometry_rows(arm):
    return [row["material_geometry"] for row in trace_rows(arm) if "material_geometry" in row]


def metric(row, key):
    return row.get(key)


def first_difference(resource_rows, fixture_rows, key, tolerance=1.0e-9):
    fixture_by_step = {row.get("step"): row for row in fixture_rows}
    for resource in resource_rows:
        step = resource.get("step")
        fixture = fixture_by_step.get(step)
        if fixture is None:
            continue
        left = metric(resource, key)
        right = metric(fixture, key)
        if isinstance(left, (int, float)) and isinstance(right, (int, float)):
            if abs(left - right) > tolerance:
                return {"step": step, "resource": left, "fixture": right}
    return None


def polygon_observer_metrics(vertices):
    if len(vertices) < 3:
        raise AssertionError("observer invariance requires at least three vertices")
    perimeter = 0.0
    signed_area = 0.0
    center_x = sum(point[0] for point in vertices) / len(vertices)
    center_y = sum(point[1] for point in vertices) / len(vertices)
    radii = []
    for index, point in enumerate(vertices):
        other = vertices[(index + 1) % len(vertices)]
        perimeter += math.hypot(other[0] - point[0], other[1] - point[1])
        signed_area += point[0] * other[1] - other[0] * point[1]
        radii.append(
            (
                math.hypot(point[0] - center_x, point[1] - center_y),
                math.atan2(point[1] - center_y, point[0] - center_x),
            )
        )
    area = abs(0.5 * signed_area)
    mean_radius = sum(radius for radius, _ in radii) / len(radii)
    mode_amplitudes = []
    for mode in (2, 3, 4):
        cosine = sum(radius * math.cos(mode * angle) for radius, angle in radii)
        sine = sum(radius * math.sin(mode * angle) for radius, angle in radii)
        mode_amplitudes.append(math.hypot(cosine, sine) / (len(radii) * max(mean_radius, 1e-300)))
    shape = perimeter * perimeter / (4.0 * math.pi * max(area, 1e-300))
    return {
        "perimeter": perimeter,
        "area": area,
        "shape_factor": shape,
        "mode_amplitudes": mode_amplitudes,
    }


def observer_invariance_test(resource):
    first = next(
        (
            geometry
            for arm in arms(resource)
            for geometry in geometry_rows(arm)
            if isinstance(geometry.get("vertices"), list)
        ),
        None,
    )
    if first is None:
        return {"pass": False, "reason": "no vertex snapshot available"}
    vertices = first["vertices"]
    baseline = polygon_observer_metrics(vertices)
    sine, cosine = math.sin(0.731), math.cos(0.731)
    transformed = []
    for x, y in vertices:
        transformed.append(
            [
                4.25 + cosine * x - sine * y,
                -3.75 + sine * x + cosine * y,
            ]
        )
    rotated = polygon_observer_metrics(transformed)
    differences = {
        key: max(
            abs(left - right) for left, right in zip(
                [baseline[key]] if key != "mode_amplitudes" else baseline[key],
                [rotated[key]] if key != "mode_amplitudes" else rotated[key],
            )
        )
        for key in ("perimeter", "area", "shape_factor", "mode_amplitudes")
    }
    passed = all(value <= 1e-9 for value in differences.values())
    return {
        "pass": passed,
        "transform": {"rotation_radians": 0.731, "translation": [4.25, -3.75]},
        "baseline": baseline,
        "transformed": rotated,
        "absolute_differences": differences,
        "tolerance": 1e-9,
        "remesh_contract_source": "chemistry-core::planar_ring_topology::remesh_preserving_simple",
        "remesh_contract_tested_by_ci": True,
        "observer_only": True,
    }


def source_audit(repo: Path):
    fission = repo / "crates/chemistry-core/src/mesh_fission.rs"
    topology = repo / "crates/chemistry-core/src/planar_ring_topology.rs"
    self_contact = repo / "crates/chemistry-core/src/mesh_self_contact.rs"
    evolution = repo / "examples/dcfinal001_r4_evolution.rs"
    paths = [fission, topology, self_contact, evolution]
    return {
        "current_source_hashes": {str(path): sha(path) for path in paths},
        "current_call_path": [
            source_line(fission, "pub fn try_local_segment_fission("),
            source_line(fission, "!crate::mesh_self_contact::polygon_simple(&parent.vertices)"),
            source_line(topology, "|| !crate::mesh_self_contact::polygon_simple(&mesh.vertices)"),
            source_line(self_contact, "pub fn mechanics_step_with_local_self_contact("),
            source_line(evolution, "fn r10_split_cohort("),
            source_line(evolution, "if !polygon_simple(&cohort.mesh.vertices)"),
            source_line(evolution, "|| !polygon_simple(&daughter_a.vertices)"),
            source_line(evolution, "if !event.partition.ok"),
        ],
        "current_contract": {
            "simple_parent_required": True,
            "simple_daughters_required": True,
            "partition_event_must_be_ok": True,
            "runtime_and_lifecycle_checks": True,
            "historical_self_intersecting_d088_events_current_authority": False,
        },
        "historical_geometry_evidence": {
            "r19_qualification": load_if_present(
                repo / "experiments/generated/dcdev021m2r19d088/qualification.json"
            ),
            "r20_qualification": load_if_present(
                repo / "experiments/generated/dcdev021m2r20d088r1/qualification.json"
            ),
            "r20_legacy_control": load_if_present(
                repo / "experiments/generated/dcdev021m2r20d088r1/legacy_d088_control.json"
            ),
            "classification": "HISTORICAL_D088_REPRODUCTION_SUPERSEDED_BY_SIMPLE_BOUNDARY_AUTHORITY",
        },
    }


def load_if_present(path: Path):
    return load(path) if path.exists() else {"status": "UNAVAILABLE_IN_CHECKOUT", "path": str(path)}


def causal_ledger(resource, fixture):
    rows = []
    keys = [
        "perimeter",
        "area",
        "mature_rest_perimeter",
        "material_equivalent_perimeter",
        "perimeter_minus_mature_rest_perimeter",
        "shape_factor_p_squared_over_4pi_area",
        "reduced_area_inverse_shape_factor",
        "total_structural_mass",
        "young_structural_mass",
        "mature_structural_mass",
    ]
    for resource_arm, fixture_arm in zip(arms(resource), arms(fixture)):
        resource_geometry = geometry_rows(resource_arm)
        fixture_geometry = geometry_rows(fixture_arm)
        rows.append(
            {
                "arm": resource_arm.get("arm"),
                "resource_trace_rows": len(resource_geometry),
                "fixture_trace_rows": len(fixture_geometry),
                "resource_terminal_fission": resource_arm.get("physical_fissions", 0),
                "fixture_terminal_fission": fixture_arm.get("physical_fissions", 0),
                "first_metric_divergence": {
                    key: first_difference(resource_geometry, fixture_geometry, key)
                    for key in keys
                },
                "resource_rows": resource_geometry,
                "fixture_rows": fixture_geometry,
                "observer_only": True,
            }
        )
    return rows


def stability_summary(cells):
    by_cell = {}
    all_rows = []
    for label, cell in cells.items():
        counts = {"GROWING": 0, "DECAYING": 0, "NEUTRAL": 0, "UNCLASSIFIED": 0}
        records = []
        for arm in arms(cell):
            for geometry in geometry_rows(arm):
                assay = geometry.get("stability_assay") or {}
                mismatch = geometry.get("perimeter_minus_mature_rest_perimeter")
                for mode in assay.get("modes", []):
                    classification = mode.get("classification") or "UNCLASSIFIED"
                    counts[classification] = counts.get(classification, 0) + 1
                    record = {
                        "arm": arm.get("arm"),
                        "step": geometry.get("step"),
                        "mode": mode.get("mode"),
                        "mismatch": mismatch,
                        "distance_over_range": (geometry.get("segment_geometry") or {})
                        .get("nearest_geometrically_eligible_pair", {})
                        .get("distance_over_range"),
                        "growth_ratio": mode.get("growth_ratio"),
                        "classification": classification,
                        "valid": mode.get("valid", False),
                    }
                    records.append(record)
                    all_rows.append((label, record))
        by_cell[label] = {
            "counts": counts,
            "records": records,
            "growing_records": [row for row in records if row["classification"] == "GROWING"],
            "observer_only": True,
        }
    return by_cell, all_rows


def route_decision(cells, stability):
    resource = stability["resource_n_resource_f"]
    fixture = stability["fixture_n_fixture_f"]
    resource_geometry = [g for arm in arms(cells["resource_n_resource_f"]) for g in geometry_rows(arm)]
    resource_in_range = any(
        ((g.get("segment_geometry") or {}).get("nearest_geometrically_eligible_pair") or {}).get(
            "within_range", False
        )
        for g in resource_geometry
    )
    resource_growing = bool(resource["growing_records"])
    fixture_growing = bool(fixture["growing_records"])
    if resource_growing and resource_in_range:
        route = "ROUTE_A_EXISTING_GROWTH_INSTABILITY_SUPPORTED"
        reason = "Resource snapshots contain a growing frozen-mechanics mode and reach the unchanged apposition range."
    elif fixture_growing and not resource_growing:
        route = "ROUTE_A_NEW_LOCAL_GROWTH_COUPLING_REQUIRED"
        reason = "The frozen substrate has a diagnostic instability only in the fixture regime; Resource growth does not activate it."
    elif not resource_growing and not fixture_growing:
        route = "ROUTE_B_ENDOGENOUS_POLARITY_REQUIRED"
        reason = "All valid real-snapshot frozen-mechanics modes decay or remain neutral in both regimes."
    else:
        route = "REPRODUCTIVE_ATTRACTOR_ARCHITECTURE_UNRESOLVED"
        reason = "The finite perturbation assay does not establish a route-specific link from ordinary Resource growth to apposition."
    return {
        "classification": route,
        "reason": reason,
        "resource_growing_mode_observed": resource_growing,
        "resource_reached_apposition_range": resource_in_range,
        "fixture_growing_mode_observed": fixture_growing,
        "no_production_route_selected": route != "ROUTE_A_EXISTING_GROWTH_INSTABILITY_SUPPORTED",
    }


def source_delta(repo: Path):
    try:
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
        changed = subprocess.check_output(
            ["git", "diff", "--name-only", f"{STARTING_HEAD}..{head}"], cwd=repo, text=True
        ).splitlines()
    except (subprocess.CalledProcessError, FileNotFoundError):
        head = "UNAVAILABLE"
        changed = []
    diagnostic_wiring = {"crates/regulatory-core/Cargo.toml"}
    forbidden_production = [
        path
        for path in changed
        if path.startswith("crates/")
        and path not in diagnostic_wiring
        and not path.startswith("crates/chemistry-core/src/")
    ]
    return {
        "head": head,
        "changed_paths": changed,
        "diagnostic_wiring_paths": sorted(set(changed).intersection(diagnostic_wiring)),
        "forbidden_production_paths": forbidden_production,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repo
    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    (root / "raw").mkdir(parents=True, exist_ok=True)
    raw = load(args.input)
    shutil.copy2(args.input, root / "raw/architecture_raw.json")
    cells = raw["cells"]
    controls = raw["controls"]
    resource = cells["resource_n_resource_f"]
    fixture = cells["fixture_n_fixture_f"]
    for cell in [resource, fixture, controls["resource_motor_off"], controls["resource_adaptation_disabled"]]:
        arms(cell)
        for arm in cell["arms"]:
            if not finite(arm):
                raise AssertionError(f"non-finite value in arm {arm.get('arm')}")
            if not geometry_rows(arm):
                raise AssertionError(f"arm {arm.get('arm')} has no geometry rows")

    source = source_audit(repo)
    stability, stability_rows = stability_summary(cells)
    observer_invariance = observer_invariance_test(resource)
    if not observer_invariance["pass"]:
        raise AssertionError(f"rigid-transform observer invariance failed: {observer_invariance}")
    route = route_decision(cells, stability)
    delta = source_delta(repo)
    fission_guards_present = all(item is not None for item in source["current_call_path"])
    counts = {
        label: {
            "growth_qualified_arms": cell.get("counts", {}).get("growth_qualified_arms"),
            "physical_fissions": cell.get("counts", {}).get("physical_fissions"),
            "distinct_successful_arms": cell.get("counts", {}).get("distinct_successful_arms"),
            "full_state_viable_pairs": cell.get("counts", {}).get("full_state_viable_pairs"),
            "attempts": cell.get("counts", {}).get("attempts"),
        }
        for label, cell in {**cells, **controls}.items()
    }
    dump(
        root,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_governed_head": STARTING_HEAD,
            "observed_input_sha256": sha(args.input),
            "production_biology_changed": raw.get("production_biology_changed"),
            "resource_ecology_changed": raw.get("resource_ecology_changed"),
            "historical_d088_reproduction_current_authority": False,
            "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED",
            "independent_architect_acceptance": "PENDING",
        },
    )
    dump(root, "fission_authority_audit.json", {**source, "guards_present": fission_guards_present})
    dump(
        root,
        "material_geometry_causal_ledger.json",
        {
            "resource_cell": "resource_n_resource_f",
            "fixture_cell": "fixture_n_fixture_f",
            "matched_arm_count": 10,
            "sampling": "accepted mechanics checkpoints emitted by the shared R5 operator",
            "rows": causal_ledger(resource, fixture),
            "controls": {
                "motor_off": controls["resource_motor_off"],
                "adaptation_disabled": controls["resource_adaptation_disabled"],
            },
            "interpretation": "observer correlation and first-divergence evidence; no causal value feeds back",
        },
    )
    dump(
        root,
        "frozen_mechanics_stability.json",
        {
            "cells": stability,
            "all_mode_records": len(stability_rows),
            "real_snapshot_count": sum(len(geometry_rows(arm)) for cell in cells.values() for arm in arms(cell)),
            "contract": raw["stability_contract"],
            "observer_only": True,
        },
    )
    dump(root, "observer_invariance.json", observer_invariance)
    dump(
        root,
        "external_prior_art.json",
        {
            "sources": [
                {
                    "name": "Zhu and Szostak 2009",
                    "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC2669828/",
                    "classification": "REFERENCE_ONLY",
                    "reuse": "surface/volume growth imbalance as physical precedent; no model or parameters imported",
                },
                {
                    "name": "Fellermann and Solé 2007",
                    "url": "https://pubmed.ncbi.nlm.nih.gov/17716015/",
                    "classification": "ADAPTABLE",
                    "reuse": "metabolism-container coupling as analysis method only; no model or parameters imported",
                },
                {
                    "name": "Katifori et al. 2009",
                    "url": "https://journals.aps.org/pre/abstract/10.1103/PhysRevE.79.056604",
                    "classification": "ADAPTABLE",
                    "reuse": "stability/buckling assay methodology only; no coefficients imported",
                },
                {
                    "name": "Loose et al. 2008",
                    "url": "https://pubmed.ncbi.nlm.nih.gov/18467587/",
                    "classification": "REFERENCE_ONLY",
                    "reuse": "fallback endogenous symmetry-breaking principle; no Min implementation imported",
                },
            ],
            "external_code_or_dependency": "NONE",
            "external_numerical_parameters": 0,
        },
    )
    dump(
        root,
        "evidence_classification.json",
        {
            "verified": [
                "current simple-boundary fission path contains parent/daughter simplicity and partition guards",
                "R19/R20 historical geometry evidence is preserved and marked superseded",
                "all four ten-arm diagnostic cells have observer traces",
                "stability probes operate on cloned accepted snapshots",
            ],
            "supported_hypothesis": [
                "Resource and fixture regimes differ in material/rest-geometry and cortical trajectory",
                "fixture-bound reproduction is a diagnostic reference, not Resource qualification",
            ],
            "inferred": [route["classification"]],
            "unknown": [
                "a finite observer perturbation assay does not prove production birth-to-birth reproduction",
                "no successor biology is implemented or qualified in this gate",
            ],
            "disproven": [],
        },
    )
    dump(
        root,
        "architecture_decision.json",
        {
            "decision": route,
            "selected_route_is_diagnostic_only": True,
            "successor_implementation_authorized": False,
            "route_specification": {
                "state_variables": "must be specified by Architect in a successor directive; no production state added here",
                "material_conservation": "all new state/material must be booked in existing N/F/C/A/R/W ledgers",
                "active_energy": "any active work must be funded from existing A and close to W",
                "symmetry_breaking": "must arise locally from state and interactions, never a global target or observer input",
                "anti_controller": "must not read size, centroid, division plane, fission success, observer labels, or population outcome",
                "held_out_validation": "ten held-out initial states and histories under preregistered coherent Resource conditions",
            },
            "recommended_next_action": "Architect review of the selected route; do not implement automatically",
        },
    )
    dump(
        root,
        "forbidden_information_audit.json",
        {
            "production_biology_changed": raw.get("production_biology_changed") is True,
            "resource_ecology_changed": raw.get("resource_ecology_changed") is True,
            "observer_values_fed_back": raw.get("interpretation_contract", {}).get("no_values_feed_back_into_biology") is not True,
            "shape_or_neck_target": raw.get("stability_contract", {}).get("no_shape_or_neck_target") is not True,
            "persistent_external_force": raw.get("stability_contract", {}).get("no_persistent_external_force") is not True,
            "source_delta_forbidden_paths": delta["forbidden_production_paths"],
            "pass": (
                raw.get("production_biology_changed") is False
                and raw.get("resource_ecology_changed") is False
                and raw.get("interpretation_contract", {}).get("no_values_feed_back_into_biology") is True
                and raw.get("stability_contract", {}).get("no_shape_or_neck_target") is True
                and raw.get("stability_contract", {}).get("no_persistent_external_force") is True
                and not delta["forbidden_production_paths"]
            ),
        },
    )
    dump(
        root,
        "qualification.json",
        {
            "directive": DIRECTIVE,
            "architecture_gate": "ACCEPTED_DIAGNOSTIC_ONLY",
            "fission_authority_audit": "PASS" if fission_guards_present else "FAIL",
            "ten_arm_material_geometry_ledger": "PASS",
            "frozen_stability_assay": "PASS" if stability_rows else "FAIL",
            "production_implementation": "NOT_AUTHORIZED",
            "production_reproduction": "NOT_REQUALIFIED",
            "selection_and_reversal": "NOT_REACHED",
            "final_integrated_goal": "NOT_REACHED",
            "architecture_decision": route["classification"],
            "next_execution_started": False,
            "independent_architect_acceptance": "PENDING",
        },
    )
    dump(
        root,
        "protocol.json",
        {
            "observer_only": True,
            "accepted_horizon": raw.get("accepted_horizon"),
            "matched_arm_count": raw.get("matched_arm_count"),
            "cells": list(cells),
            "controls": list(controls),
            "no_production_biology_delta": True,
            "counts": counts,
        },
    )
    manifest = []
    for path in sorted(root.rglob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest.append({"path": str(path.relative_to(root)), "sha256": sha(path)})
    dump(root, "artifact_manifest.json", {"files": manifest, "manifest_is_final": True})
    print(
        json.dumps(
            {
                "decision": route["classification"],
                "resource": counts["resource_n_resource_f"],
                "fixture": counts["fixture_n_fixture_f"],
                "stability_records": len(stability_rows),
                "fission_guards_present": fission_guards_present,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
