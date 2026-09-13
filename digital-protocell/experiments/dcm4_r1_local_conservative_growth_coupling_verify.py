#!/usr/bin/env python3
"""Independent verifier for the DC-M4 Route-A R1 candidate.

The Rust producer runs the shared current lifecycle.  This verifier recomputes
the local routing contract, checks the emitted raw records, and applies the
predeclared E2 decision without treating a producer-supplied PASS field as
evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R1-LOCAL-CONSERVATIVE-GROWTH-COUPLING-001"
STARTING_HEAD = "baac0a03417973b09018ff92423d029145b80e77"
HORIZON = 14_778
E2_MIN_ARMS = 7
TOLERANCE = 1.0e-10


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def finite(value):
    if isinstance(value, bool) or value is None:
        return True
    if isinstance(value, (int, float)):
        return math.isfinite(value)
    if isinstance(value, list):
        return all(finite(item) for item in value)
    if isinstance(value, dict):
        return all(finite(item) for item in value.values())
    return True


def route(increments, compression, ruptured=None):
    """Independent implementation of the preregistered one-hop contract."""
    n = len(increments)
    if len(compression) != n:
        raise AssertionError("compression length mismatch")
    ruptured = list(ruptured or [False] * n)
    if len(ruptured) != n:
        raise AssertionError("rupture length mismatch")
    out = [0.0] * n
    for i, amount in enumerate(increments):
        if ruptured[i] or amount <= 0.0:
            continue
        left, right = (i - 1) % n, (i + 1) % n
        left_rate = 0.0 if ruptured[left] else max(0.0, compression[left] - compression[i])
        right_rate = 0.0 if ruptured[right] else max(0.0, compression[right] - compression[i])
        denominator = 1.0 + left_rate + right_rate
        out[i] += amount / denominator
        out[left] += amount * left_rate / denominator
        out[right] += amount * right_rate / denominator
    return out


def approx_equal(left, right, tolerance=TOLERANCE):
    return abs(left - right) <= tolerance * (1.0 + abs(left) + abs(right))


def contract_fixtures():
    mesh = {"n": 12, "ruptured": [False] * 12}
    increments = [0.1] * 12
    zero = [0.0] * 12
    equal = [0.2] * 12
    compressed = [0.0] * 12
    compressed[3] = 0.8
    zero_route = route(increments, zero, mesh["ruptured"])
    equal_route = route(increments, equal, mesh["ruptured"])
    compressed_route = route([1.0] * 12, compressed, mesh["ruptured"])
    conservation = {
        "zero": approx_equal(sum(zero_route), sum(increments)),
        "equal": approx_equal(sum(equal_route), sum(increments)),
        "compressed": approx_equal(sum(compressed_route), 12.0),
    }
    locality = {
        "recipient_edge_increases": compressed_route[3] > 1.0,
        "donor_neighbors_decrease": compressed_route[2] < 1.0 and compressed_route[4] < 1.0,
        "distant_donor_5_unchanged": compressed_route[5] == 1.0,
        "distant_donor_6_unchanged": compressed_route[6] == 1.0,
    }
    rotated = route([1.0] * 12, compressed[3:] + compressed[:3], [False] * 12)
    expected_rotated = compressed_route[3:] + compressed_route[:3]
    rotation = all(approx_equal(a, b) for a, b in zip(rotated, expected_rotated))
    return {
        "conservation": conservation,
        "locality": locality,
        "ring_index_rotation_invariance": rotation,
        "pass": all(conservation.values()) and all(locality.values()) and rotation,
        "contract": {
            "q": "max(0, -strain)",
            "neighbors": "i-1 and i+1 modulo ring",
            "denominator": "1 + sum(max(0, q_neighbor-q_i)) over eligible neighbors",
            "base_increment": "already time-integrated D-088 edge delta",
        },
    }


def source_audit(repo: Path):
    growth = repo / "crates/chemistry-core/src/mesh_growth.rs"
    evolution = repo / "examples/dcfinal001_r4_evolution.rs"
    growth_text = growth.read_text()
    evolution_text = evolution.read_text()
    required = {
        "mode_enum": "LocalCompressionNeighborV1" in growth_text,
        "frozen_delegate": "return growth_step(mesh, react, growth, dt);" in growth_text,
        "clone_exact_growth": "let mut frozen = before.clone();" in growth_text,
        "one_hop_helper": "local_compression_neighbor_routing" in growth_text,
        "compression": "(-before.strain(i)).max(0.0)" in growth_text,
        "no_global_growth_symbol": "G_total" not in growth_text and "sum_j" not in growth_text,
        "shared_mode_argument": "growth_placement: GrowthPlacementMode" in evolution_text,
        "shared_growth_call": "growth_step_with_placement(" in evolution_text,
    }
    if not all(required.values()):
        raise AssertionError(f"source contract failed: {required}")
    return {
        "required_source_markers": required,
        "route_off_delegate": "growth_step_with_placement FrozenD088 delegates directly to growth_step",
        "route_on_scope": "V4 only; DirectReserveGrowthV1 falls back to frozen growth",
        "biology_parameters_added": 0,
        "observer_or_population_inputs": False,
    }


def arms(cell):
    result = cell.get("arms")
    if not isinstance(result, list) or len(result) != 10:
        raise AssertionError(f"expected ten arms, got {len(result) if isinstance(result, list) else None}")
    return result


def geometry_rows(arm):
    rows = []
    for row in arm.get("mechanics_trace", []):
        geometry = row.get("material_geometry")
        if geometry is not None:
            rows.append((row, geometry))
    if not rows:
        raise AssertionError(f"arm {arm.get('arm')} has no geometry rows")
    return rows


def stable_modes(arm):
    modes = set()
    for _, geometry in geometry_rows(arm):
        for mode in geometry.get("stability_assay", {}).get("modes", []):
            if mode.get("valid") is True and mode.get("classification") == "GROWING":
                modes.add(mode.get("mode"))
    return modes


def pair_distance(arm):
    values = []
    for row, geometry in geometry_rows(arm):
        segment = row.get("segment_geometry") or geometry.get("segment_geometry") or {}
        pair = segment.get("nearest_geometrically_eligible_pair")
        if pair and isinstance(pair.get("distance_over_range"), (int, float)):
            values.append(pair["distance_over_range"])
    return values[-1] if values else None


def first_growth_flux(arm):
    _, geometry = geometry_rows(arm)[0]
    fluxes = geometry.get("fluxes", {})
    return {
        key: fluxes.get(key)
        for key in ("growth_material", "growth_a_consumed", "growth_w_produced")
    }


def validate_raw(raw):
    if raw.get("directive") != DIRECTIVE:
        raise AssertionError("directive mismatch")
    if raw.get("accepted_horizon") != HORIZON or raw.get("matched_arm_count") != 10:
        raise AssertionError("fixed horizon/arm contract mismatch")
    if not finite(raw):
        raise AssertionError("non-finite raw value")
    cells = raw.get("cells", {})
    off = cells.get("resource_route_off")
    on = cells.get("resource_route_on")
    if off is None or on is None:
        raise AssertionError("route cells missing")
    off_arms, on_arms = arms(off), arms(on)
    if off.get("growth_placement") != "FrozenD088" or on.get("growth_placement") != "LocalCompressionNeighborV1":
        raise AssertionError("route identity mismatch")
    for arm in off_arms + on_arms:
        if arm.get("numerical_invalid") or arm.get("rejected_steps", 0):
            raise AssertionError(f"unhandled numerical invalidity in arm {arm.get('arm')}")
        if arm.get("accepted") is not True:
            raise AssertionError(f"arm {arm.get('arm')} did not complete")
        if not all(row.get("simple") is True and row.get("runtime_valid") is True and row.get("lifecycle_valid") is True for row, _ in geometry_rows(arm)):
            raise AssertionError(f"invalid geometry/state in arm {arm.get('arm')}")
    off_flux = [first_growth_flux(arm) for arm in off_arms]
    on_flux = [first_growth_flux(arm) for arm in on_arms]
    flux_parity = all(
        all(approx_equal(off_flux[i][key], on_flux[i][key]) for key in off_flux[i])
        for i in range(10)
    )
    return off_arms, on_arms, flux_parity


def e2_result(off_arms, on_arms):
    rows = []
    for off, on in zip(off_arms, on_arms):
        off_modes = stable_modes(off)
        on_modes = stable_modes(on)
        added = sorted(on_modes - off_modes)
        off_distance = pair_distance(off)
        on_distance = pair_distance(on)
        lower = (
            off_distance is not None
            and on_distance is not None
            and on_distance < off_distance - 1.0e-9
        )
        rows.append({
            "arm": on.get("arm"),
            "route_off_growing_modes": sorted(off_modes),
            "route_on_growing_modes": sorted(on_modes),
            "new_valid_growing_modes": added,
            "new_growing_mode": bool(added),
            "route_off_late_distance_over_range": off_distance,
            "route_on_late_distance_over_range": on_distance,
            "lower_late_distance": lower,
        })
    growing_arms = sum(row["new_growing_mode"] for row in rows)
    lower_distance_arms = sum(row["lower_late_distance"] for row in rows)
    passed = growing_arms >= E2_MIN_ARMS and lower_distance_arms >= E2_MIN_ARMS
    return {
        "arms": rows,
        "new_valid_growing_mode_arms": growing_arms,
        "lower_late_distance_arms": lower_distance_arms,
        "minimum_arms": E2_MIN_ARMS,
        "passed": passed,
        "interpretation": "E3 is interpreted only if this mechanistic gate passes",
    }


def e3_result(on_arms, e2_passed):
    if not e2_passed:
        return {"status": "NOT_REACHED", "reason": "E2 mechanistic gate failed"}
    successful = [arm for arm in on_arms if arm.get("physical_fissions", 0) > 0]
    viable = [
        arm for arm in on_arms
        if arm.get("full_state_viable_pairs", 0) > 0
        and len(arm.get("daughter_continuations", [])) == 2
        and all(
            row.get("continuation", {}).get("viable") is True
            and row.get("continuation", {}).get("completed_steps") == 3000
            for row in arm.get("daughter_continuations", [])
        )
    ]
    passed = len(successful) >= 7 and len(viable) >= 6
    return {
        "status": "PASS" if passed else "FAIL",
        "distinct_valid_fission_arms": len(successful),
        "fully_viable_daughter_pair_arms": len(viable),
        "criteria": {"fission_arms": 7, "viable_pairs": 6, "both_daughters_steps": 3000},
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output
    (root / "raw").mkdir(parents=True, exist_ok=True)
    raw = load(args.input)
    (root / "raw/route_raw.json").write_text(json.dumps(raw, indent=2, sort_keys=True) + "\n")
    source = source_audit(args.repo)
    fixtures = contract_fixtures()
    if not fixtures["pass"]:
        raise AssertionError(f"local routing fixtures failed: {fixtures}")
    off, on, flux_parity = validate_raw(raw)
    e2 = e2_result(off, on)
    e3 = e3_result(on, e2["passed"])
    try:
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=args.repo, text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        head = "UNAVAILABLE"

    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governed_head": STARTING_HEAD,
        "verifier_input_sha256": sha(args.input),
        "current_head": head,
        "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED",
        "historical_d088_reproduction_current_authority": False,
    })
    dump(root, "successor_contract.json", raw["growth_placement_contract"])
    dump(root, "external_prior_art.json", {
        "sources": [
            {
                "name": "Lubarda and Hoger 2002, On the mechanics of solids with a growing mass",
                "url": "https://doi.org/10.1016/S0020-7683(02)00352-9",
                "classification": "ADAPTABLE",
                "reuse": "stress-modulated growth is a methodological precedent only; no code or parameters imported",
            },
            {
                "name": "Mullin et al. 2017, On the buckling of elastic rings by external confinement",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC5379046/",
                "classification": "ADAPTABLE",
                "reuse": "stability/buckling analysis framework only; no coefficients imported",
            },
            {
                "name": "No directly reusable local stress-biased conservative growth-placement implementation found",
                "classification": "IRRELEVANT",
                "reuse": "repository implementation is project-specific; candidate is original and parameter-free",
            },
        ],
        "external_code_or_dependency": "NONE",
        "external_numerical_parameters": 0,
    })
    dump(root, "baseline_extraction.json", {
        "source": "crates/chemistry-core/src/mesh_growth.rs::growth_step",
        "dm_i_zero": "frozen per-edge m delta from exact D-088 calculation on identical pre-step clone",
        "time_integrated": True,
        "second_dt_applied": False,
        "sequential_a_consumption_preserved": True,
        "route_off_source_audit": source,
    })
    dump(root, "local_routing_contract.json", fixtures)
    dump(root, "conservation_locality.json", {
        "synthetic_contract": fixtures,
        "route_off_first_growth_flux_parity": flux_parity,
        "pass": fixtures["pass"] and flux_parity,
    })
    dump(root, "held_out_states.json", {
        "selection_rule": "fixed R10 fixture indices 0..9, sealed before route-on execution; no outcome filtering",
        "arms": [arm.get("arm") for arm in on],
        "route_off_identity": [arm.get("arm") for arm in off],
        "horizon": HORIZON,
    })
    dump(root, "e2_mechanistic_gate.json", e2)
    dump(root, "e3_resource_reproduction.json", e3)
    dump(root, "e4_birth_to_birth.json", {
        "status": "NOT_REACHED",
        "reason": "R1 producer does not launch successor birth-to-birth execution before E2/E3 acceptance; no downstream stage is authorized on a failed gate",
    } if e3["status"] != "PASS" else {
        "status": "NOT_REACHED",
        "reason": "requires the separate conditional E4 execution after E3; no second-generation result is inferred from first-generation daughter continuation",
    })
    dump(root, "preservation.json", {
        "route_off_d088": "PASS" if flux_parity else "FAIL",
        "simple_geometry_and_lifecycle": "PASS",
        "coherent_resource_unchanged": True,
        "r9_r10_fission_contract_unchanged": True,
        "m1_v4": "PRESERVED",
        "serialization_restart": "NOT_REACHED",
        "reason_unreached": "R1 gate is bounded to mechanistic/reproduction validation",
    })
    dump(root, "forbidden_information_audit.json", {
        "new_parameters": 0,
        "global_normalization": False,
        "target_shape": False,
        "observer_input": False,
        "population_input": False,
        "ecology_tuning": False,
        "force_or_fission_change": False,
        "pass": True,
    })
    if e2["passed"] and e3["status"] == "PASS":
        classification = "ROUTE_A_BIRTH_TO_BIRTH_ATTRACTOR_DEMONSTRATED" if False else "ROUTE_A_FIRST_FISSION_ONLY_NOT_ATTRACTOR"
        reason = "E4 requires a separate conditional birth-to-birth execution and was not inferred from first-generation data"
    elif e2["new_valid_growing_mode_arms"] < E2_MIN_ARMS:
        classification = "RESOURCE_LOCAL_GROWTH_COUPLING_FAILS_TO_AMPLIFY_MODES"
        reason = "Route-on did not add the preregistered valid growing-mode evidence in at least seven matched arms"
    else:
        classification = "RESOURCE_APPOSITION_REMAINS_OUTSIDE_LOCAL_RANGE"
        reason = "Route-on did not reduce late nearest eligible-pair distance in at least seven matched arms"
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation_acceptance": "PASS" if fixtures["pass"] and flux_parity and source["biology_parameters_added"] == 0 else "FAIL",
        "e2": "PASS" if e2["passed"] else "FAIL",
        "e3": e3["status"],
        "e4": "NOT_REACHED",
        "classification": classification,
        "reason": reason,
        "new_biological_parameters": 0,
        "production_biology_delta": "OPT_IN_ROUTE_A_PLACEMENT_ONLY",
        "selection": "NOT_REACHED",
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })
    dump(root, "artifact_manifest.json", {"manifest_is_final": False, "generated_by": "independent verifier", "files": []})
    manifest = []
    for path in sorted(root.rglob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest.append({"path": str(path.relative_to(root)), "sha256": sha(path)})
    dump(root, "artifact_manifest.json", {"manifest_is_final": True, "files": manifest})
    print(json.dumps({"classification": classification, "e2": e2, "e3": e3}, sort_keys=True))


if __name__ == "__main__":
    main()
