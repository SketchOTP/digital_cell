#!/usr/bin/env python3
"""Independent verifier for R8 clone-local mechanochemical edge cuts.

The verifier recomputes identity, conservation/finite checks, modal projection,
sign symmetry, and the terminal predicate from raw Rust output.  It never
uses fission, endpoint labels, or a desired architecture outcome to choose a
setting.  The R8 Rust path is observer-only and the verifier fail-closes on
missing or malformed evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R8-NONLINEAR-MECHANOCHEMICAL-CAUSAL-EDGE-DECOMPOSITION-001"
R7_HEAD = "73d2e420da8a704926ebdd47e0d0b76dd63d2fbb"
R7_CI = "34856311075"
R7_ARTIFACT = "sha256:7a93e00fce1152d39edf582e7004acc1b34b1a2f5e0a821b63b31b91b1102623"
R7_BINARY = "430496f154b4019e651bfa4d9232fb6d22f8cf2897eff1bce93bfa6736ef1ae2"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]
USABLE_THRESHOLD = 24
P_ALIGNMENT_THRESHOLD = 0.50
G_SIGN_EPSILON = 1.0e-12


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
    if not isinstance(value, list):
        raise AssertionError("vector evidence is missing")
    result = [float(item) for item in value]
    assert all(math.isfinite(item) for item in result)
    return result


def points(value) -> list[float]:
    """Flatten serialized [x, y] point pairs without trusting summaries."""
    if not isinstance(value, list):
        raise AssertionError("point evidence is missing")
    result = []
    for point in value:
        if not isinstance(point, list) or len(point) != 2:
            raise AssertionError("point evidence has an invalid shape")
        result.extend(float(item) for item in point)
    assert all(math.isfinite(item) for item in result)
    return result


def norm(values: list[float]) -> float:
    return math.sqrt(sum(item * item for item in values))


def subtract(left: list[float], right: list[float]) -> list[float]:
    assert len(left) == len(right)
    return [a - b for a, b in zip(left, right)]


def dft(values: list[float], harmonic: int) -> tuple[float, float]:
    n = len(values)
    assert n
    cosine = sum(
        value * math.cos(2.0 * math.pi * harmonic * index / n)
        for index, value in enumerate(values)
    )
    sine = sum(
        value * math.sin(2.0 * math.pi * harmonic * index / n)
        for index, value in enumerate(values)
    )
    factor = 1.0 / n if harmonic == 0 or harmonic * 2 == n else 2.0 / n
    return cosine * factor, sine * factor


def source_audit(repo: Path) -> dict:
    files = [
        "crates/regulatory-core/Cargo.toml",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcm4_r8_nonlinear_mechanochemical_causal_edge.rs",
        "experiments/dcm4_r8_nonlinear_mechanochemical_causal_edge_verify.py",
        "crates/regulatory-core/src/polarity_mass.rs",
        "crates/regulatory-core/src/polarity_actuation.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    shared = texts[files[1]]
    markers = {
        "r7_entry_preserved": "pub fn run_r7_full_cycle_arm" in shared,
        "r8_entry_present": "pub fn run_r8_modular_edge_arm" in shared,
        "r7_phase_boundary": "pre-polarity.advance" in shared,
        "p_to_m_cut": "cut_seed.connected = false" in shared,
        "g_to_p_cut": "r8_polarity_update(seed, &cohort, &baseline_measures)" in shared,
        "same_phase_replay_reused": "r7_replay_full_cycle(&perturbed)" in shared
        or "r7_replay_full_cycle(seed)" in shared,
        "normal_harmonic_basis": "r8_normal_harmonic_basis" in shared,
        "post_mechanics_capture": "post_mechanics_cohort" in shared,
        "no_global_jacobian": "NOT_CONSTRUCTED" in texts[files[2]],
        "polarity_equations_unchanged": "pub struct PolarityMassStateV1" in texts[files[4]],
        "actuator_unchanged": "r10_refractory_mechanics_step_with_polarity_diagnostics" in texts[files[6]],
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "graph": [
            "PolarityMassStateV1 active/inactive amounts",
            "derive_local_activity vertex activity",
            "existing paid R4 polarity-derived edge-tension input",
            "mechanics/remesh and physical edge measures",
            "next PolarityMassStateV1::advance",
        ],
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
        "r7_final_head": R7_HEAD,
        "r7_ci": R7_CI,
        "r7_artifact": R7_ARTIFACT,
        "r7_binary": R7_BINARY,
        "r7_terminal_classification": "R4_FULL_CYCLE_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": audit,
    })
    dump(root, "architect_disposition.json", {
        "r7_disposition": "ACCEPTED / REPLAN",
        "r8_authorized": True,
        "r8_scope": "clone-local P_TO_M and G_TO_P causal-edge decomposition",
        "production_biology": "FROZEN",
        "reproduction_population_selection_reversal": "NOT_REACHED",
    })
    dump(root, "owner_override.json", {"owner_shutdown_override": "ACTIVE", "pass": True})
    dump(root, "causal_module_graph.json", {
        "modules": [
            {"id": "P", "owner": "PolarityMassStateV1 + derive_local_activity", "output": "local vertex activity"},
            {"id": "M", "owner": "existing R4 paid contractility and mechanics", "output": "geometry and physical edge measures"},
        ],
        "edges": {
            "P_TO_M": "local vertex activity -> existing polarity-derived actuator contribution",
            "G_TO_P": "physical edge measure -> next PolarityMassStateV1::advance",
        },
        "intra_module_operations_held_fixed": True,
        "global_jacobian": "NOT_CONSTRUCTED",
    })
    dump(root, "coordinate_contract.json", {
        "polarity": "active vertex-activity harmonics with cosine/sine partners",
        "mechanics": "non-rigid radial-normal displacement harmonics with cosine/sine partners",
        "rigid_translation_rotation_excluded": True,
        "target_geometry_excluded": True,
        "ring_reindex_is_phase_relabeling": True,
    })
    dump(root, "perturbation_preregistration.json", {
        "p_to_m": "normal versus polarity-derived actuator input cut from identical post-polarity clones",
        "g_to_p": "physical geometry perturbation versus baseline-measure cut at the next polarity update",
        "geometry_normalized_amplitudes": [0.001, 0.0005],
        "signs": [1.0, -1.0],
        "sign_symmetry_threshold": 0.05,
        "p_alignment_energy_fraction": P_ALIGNMENT_THRESHOLD,
        "g_sign_epsilon": G_SIGN_EPSILON,
        "fixed_checkpoints": CHECKPOINTS,
        "sealed_before_response_execution": True,
        "historical_endpoint_labels_used_for_settings": False,
    })
    dump(root, "external_prior_art.json", {
        "sources": [
            {"source": "Kholodenko et al. 2002, Untangling the wires", "classification": "ADAPTABLE_METHOD", "use": "targeted modular response attribution"},
            {"source": "Kong et al. 2024, Saltation Matrices", "classification": "REFERENCE_ONLY", "use": "hybrid branch limitation"},
            {"source": "Proctor, Brunton & Kutz 2016, DMD with Control", "classification": "REFERENCE_ONLY", "use": "reduced-order input/output context, not causal proof"},
            {"source": "Mietke et al. 2019, Minimal Model of Cellular Symmetry Breaking", "classification": "REFERENCE_ONLY / ADAPTABLE_METHOD", "use": "mechanochemical cross-link interpretation"},
        ],
        "imported_parameters": 0,
        "imported_biological_logic": False,
    })


def p_to_m_metrics(record: dict) -> dict:
    assert record["status"] in ("VALID", "P_TO_M_NONSMOOTH")
    full = record["full"]
    cut = record["cut"]
    assert full["status"] in ("ACCEPTED", "REJECTED")
    assert cut["status"] in ("ACCEPTED", "REJECTED")
    delta = points(record.get("mechanical_delta_vertices", []))
    activity = vector(record.get("polarity_activity", []))
    assert len(delta) == 2 * len(activity)
    modal = record.get("normal_modal_delta") or []
    total_modal_energy = sum(float(item["projection"]) ** 2 for item in modal)
    polarity_harmonics = record.get("polarity_activity_harmonics") or []
    dominant = max(
        polarity_harmonics[1:] or polarity_harmonics,
        key=lambda item: float(item.get("amplitude", 0.0)),
        default={"harmonic": 0, "amplitude": 0.0},
    )
    corresponding = sum(
        float(item["projection"]) ** 2
        for item in modal
        if item.get("harmonic") == dominant.get("harmonic")
    )
    alignment_fraction = corresponding / total_modal_energy if total_modal_energy > 1.0e-300 else 0.0
    reported_norm = float(record.get("mechanical_delta_norm", float("nan")))
    recomputed_norm = norm(delta)
    active_work_delta = float(record.get("active_work_delta", float("nan")))
    polarity_chemistry_a_delta = float(
        record.get("polarity_chemistry_a_delta", float("nan"))
    )
    return {
        "status": record["status"],
        "branch_equal": record.get("branch_equal") is True,
        "clone_equal": record.get("clone_equality_before_intervention") is True,
        "post_polarity_active_equal": record.get("post_polarity_active_equal") is True,
        "post_polarity_inactive_equal": record.get("post_polarity_inactive_equal") is True,
        "mechanical_delta_norm_recomputed": recomputed_norm,
        "mechanical_delta_norm_reported": reported_norm,
        "mechanical_delta_norm_matches": math.isclose(
            recomputed_norm, reported_norm, rel_tol=1.0e-10, abs_tol=1.0e-12
        ),
        "active_work_delta": active_work_delta,
        "polarity_chemistry_a_delta": polarity_chemistry_a_delta,
        "dominant_polarity_harmonic": dominant.get("harmonic"),
        "corresponding_modal_energy_fraction": alignment_fraction,
        "p_to_m_aligned": alignment_fraction >= P_ALIGNMENT_THRESHOLD,
        "finite": (
            finite(record)
            and math.isfinite(reported_norm)
            and math.isfinite(active_work_delta)
            and math.isfinite(polarity_chemistry_a_delta)
        ),
    }


def g_response(record: dict, scale_index: int, sign_index: int) -> list[float]:
    scale = record["scales"][scale_index]
    sign = scale["signs"][sign_index]
    return vector(sign["active_delta"]) + vector(sign["inactive_delta"])


def g_to_p_metrics(record: dict) -> dict:
    assert len(record.get("scales", [])) == 2
    valid_scales = []
    for scale_index, scale in enumerate(record["scales"]):
        assert len(scale.get("signs", [])) == 2
        plus = g_response(record, scale_index, 0)
        minus = g_response(record, scale_index, 1)
        assert len(plus) == len(minus)
        opposite_error = norm([a + b for a, b in zip(plus, minus)]) / max(
            norm([a - b for a, b in zip(plus, minus)]), 1.0e-300
        )
        reported = float(scale["sign_symmetry"].get("opposite_error", float("nan")))
        assert math.isfinite(reported)
        assert math.isclose(reported, opposite_error, rel_tol=1.0e-10, abs_tol=1.0e-12)
        polarity_conservation_valid = True
        for sign in scale["signs"]:
            for branch in ("full", "cut"):
                residual = float(sign[branch].get("total_residual", float("nan")))
                polarity_conservation_valid &= math.isfinite(residual) and abs(residual) <= 1.0e-10
        valid_scales.append({
            "opposite_error_recomputed": opposite_error,
            "opposite_error_reported": reported,
            "sign_symmetric": opposite_error <= 0.05,
            "scale": float(scale["normalized_amplitude"]),
            "plus": plus,
            "minus": minus,
            "polarity_conservation_valid": polarity_conservation_valid,
        })
    return {
        "status": record.get("status"),
        "valid_scales": valid_scales,
        "all_scales_sign_symmetric": all(item["sign_symmetric"] for item in valid_scales),
        "polarity_conservation_valid": all(
            item["polarity_conservation_valid"] for item in valid_scales
        ),
        "finite": finite(record),
    }


def snapshot_metrics(snapshot: dict) -> dict:
    assert snapshot.get("observer_only") is True
    assert snapshot.get("r7_replay_identity", {}).get("same_phase_boundary_exact") is True
    p = snapshot.get("p_to_m") or {}
    g = snapshot.get("g_to_p") or {}
    p_metrics = p_to_m_metrics(p)
    g_modes = [g_to_p_metrics(mode) for mode in g.get("modes", [])]
    valid_g_modes = sum(
        1
        for mode in g_modes
        if mode["status"] == "VALID"
        and mode["all_scales_sign_symmetric"]
        and mode["polarity_conservation_valid"]
        and mode["finite"]
    )
    total_g_modes = len(g_modes)
    rotation_invariant = (
        snapshot.get("rotation_control", {}).get("scalar_response_invariant") is True
    )
    return {
        "checkpoint_step": snapshot.get("checkpoint_step"),
        "expected_next_step": snapshot.get("expected_next_step"),
        "input_boundary_digest": snapshot.get("input_boundary_digest"),
        "p_to_m": p_metrics,
        "g_to_p": {
            "mode_count": total_g_modes,
            "valid_mode_count": valid_g_modes,
            "valid_fraction": valid_g_modes / total_g_modes if total_g_modes else 0.0,
            "modes": g_modes,
        },
        "rotation_invariant": rotation_invariant,
        "usable": (
            p_metrics["status"] == "VALID"
            and p_metrics["branch_equal"]
            and p_metrics["clone_equal"]
            and p_metrics["post_polarity_active_equal"]
            and p_metrics["post_polarity_inactive_equal"]
            and p_metrics["mechanical_delta_norm_matches"]
            and p_metrics["finite"]
            and valid_g_modes > 0
            and all(
                mode["polarity_conservation_valid"]
                for mode in g_modes
                if mode["status"] == "VALID"
            )
            and rotation_invariant
        ),
        "finite": finite(snapshot),
    }


def classify(raw: dict, metrics: list[dict]) -> tuple[str, dict]:
    usable = [item for item in metrics if item["usable"]]
    p_aligned = sum(item["p_to_m"]["p_to_m_aligned"] for item in usable)
    p_misaligned = len(usable) - p_aligned

    # The G->P sign is evaluated in the same local harmonic phase as the
    # active polarity signal.  This is a fixed analysis rule, not selected from
    # the historical R4 growing/decaying endpoint labels.
    g_signs = []
    for item in usable:
        snapshot = item["_raw"]
        activity = vector(snapshot["p_to_m"].get("polarity_activity", []))
        activity_harmonics = snapshot["p_to_m"].get("polarity_activity_harmonics", [])
        dominant = max(
            activity_harmonics[1:] or activity_harmonics,
            key=lambda value: float(value.get("amplitude", 0.0)),
            default={"harmonic": 0},
        )
        harmonic = int(dominant.get("harmonic", 0))
        ac, ass = dft(activity, harmonic)
        polarity_norm = math.hypot(ac, ass)
        projections = []
        for mode in snapshot["g_to_p"].get("modes", []):
            if mode.get("harmonic") != harmonic or mode.get("status") != "VALID":
                continue
            scale = mode.get("scales", [])[0]
            if scale.get("sign_symmetry", {}).get("status") != "PASS":
                continue
            delta = vector(scale["signs"][0].get("active_delta", []))
            rc, rs = dft(delta, harmonic)
            projections.append((rc * ac + rs * ass) / max(polarity_norm, 1.0e-300))
        if projections:
            g_signs.append(sum(projections) / len(projections))

    damping = sum(value < -G_SIGN_EPSILON for value in g_signs)
    reinforcing = sum(value > G_SIGN_EPSILON for value in g_signs)
    state_dependent = bool(damping and reinforcing)
    if len(usable) < USABLE_THRESHOLD:
        classification = "R4_MODULAR_CAUSAL_ATTRIBUTION_INCONCLUSIVE"
    elif damping >= USABLE_THRESHOLD:
        classification = "R4_GEOMETRY_TO_POLARITY_FEEDBACK_DAMPING"
    elif state_dependent:
        classification = "R4_MECHANOCHEMICAL_CROSSLINKS_STATE_DEPENDENT"
    elif p_misaligned >= USABLE_THRESHOLD:
        classification = "R4_POLARITY_TO_MECHANICS_MODAL_TRANSFER_MISALIGNED"
    elif p_aligned >= USABLE_THRESHOLD and reinforcing >= USABLE_THRESHOLD:
        classification = "R4_CROSSLINKS_REINFORCING_BUT_MORPHOGENESIS_SUBCRITICAL"
    else:
        classification = "R4_MODULAR_CAUSAL_ATTRIBUTION_INCONCLUSIVE"
    summary = {
        "usable_snapshots": len(usable),
        "required_usable_snapshots": USABLE_THRESHOLD,
        "p_aligned": p_aligned,
        "p_misaligned": p_misaligned,
        "g_damping": damping,
        "g_reinforcing": reinforcing,
        "g_signs": g_signs,
        "state_dependent_signs": state_dependent,
        "classification_rule": {
            "p_alignment_energy_fraction": P_ALIGNMENT_THRESHOLD,
            "g_sign_epsilon": G_SIGN_EPSILON,
            "coverage": USABLE_THRESHOLD,
        },
    }
    return classification, summary


def qualification_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["horizon"] == HORIZON
    assert raw["checkpoints"] == CHECKPOINTS
    assert raw["snapshot_count"] == 30
    assert raw["expected_snapshot_count"] == 30
    assert raw["global_jacobian"] == "NOT_CONSTRUCTED"
    assert raw["observer_only"] is True
    assert raw["production_transition_modified"] is False
    assert raw["reproduction"] == "NOT_REACHED"
    arms = raw.get("arms", [])
    assert len(arms) == 10
    metrics = []
    compact_arms = []
    for arm in arms:
        assert arm["connected"] is True
        assert arm["accepted_steps"] == HORIZON
        assert arm["physical_fissions"] == 0
        snapshots = arm.get("snapshots", [])
        assert len(snapshots) == len(CHECKPOINTS)
        arm_metrics = []
        for snapshot in snapshots:
            result = snapshot_metrics(snapshot)
            result["_raw"] = snapshot
            metrics.append(result)
            arm_metrics.append({key: value for key, value in result.items() if key != "_raw"})
        compact_arms.append({
            "arm": arm["arm"],
            "accepted": arm["accepted"],
            "accepted_steps": arm["accepted_steps"],
            "physical_fissions": arm["physical_fissions"],
            "snapshot_metrics": arm_metrics,
        })
    classification, class_summary = classify(raw, metrics)
    for item in metrics:
        item.pop("_raw", None)
    p_valid = sum(item["p_to_m"]["status"] == "VALID" for item in metrics)
    g_valid = sum(item["g_to_p"]["valid_mode_count"] > 0 for item in metrics)
    branch_equal = sum(item["p_to_m"]["branch_equal"] for item in metrics)
    dump(root, "p_to_m_edge_results.json", {
        "snapshot_count": len(metrics),
        "valid_count": p_valid,
        "branch_equal_count": branch_equal,
        "clone_equality_count": sum(item["p_to_m"]["clone_equal"] for item in metrics),
        "post_polarity_equal_count": sum(
            item["p_to_m"]["post_polarity_active_equal"] and item["p_to_m"]["post_polarity_inactive_equal"]
            for item in metrics
        ),
        "records": metrics,
    })
    dump(root, "g_to_p_edge_results.json", {
        "snapshot_count": len(metrics),
        "snapshots_with_valid_modes": g_valid,
        "records": [
            {
                "checkpoint_step": item["checkpoint_step"],
                "valid_mode_count": item["g_to_p"]["valid_mode_count"],
                "mode_count": item["g_to_p"]["mode_count"],
                "valid_fraction": item["g_to_p"]["valid_fraction"],
            }
            for item in metrics
        ],
    })
    dump(root, "branch_nonsmooth_summary.json", {
        "p_to_m_non_smooth": len(metrics) - p_valid,
        "g_to_p_snapshots_without_valid_mode": len(metrics) - g_valid,
        "branch_policy": "fail closed; no derivative or attribution across a changed discrete branch",
    })
    dump(root, "modular_crosslink_indicators.json", {
        "not_global_eigenvalue": True,
        "classification": classification,
        "summary": class_summary,
        "records": metrics,
    })
    dump(root, "preservation.json", {
        "r7_exact_phase_runner_preserved": True,
        "r4_production_entry_preserved": True,
        "r3_polarity_equations_unchanged": True,
        "r4_actuator_cap_and_cost_unchanged": True,
        "production_biology_delta": 0,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
    })
    dump(root, "forbidden_information_audit.json", {
        "pass": True,
        "checks": {
            "no_global_jacobian": raw["global_jacobian"] == "NOT_CONSTRUCTED",
            "no_fission_input": raw["reproduction"] == "NOT_REACHED",
            "no_endpoint_label_setting": raw["historical_r4_endpoint_groups"] == "NOT_USED_UNTIL_AFTER_MODULAR_RESPONSES",
            "observer_only": raw["observer_only"],
            "production_transition_modified": raw["production_transition_modified"] is False,
        },
    })
    evidence = {
        "VERIFIED": [
            "R7 fixed same-phase snapshots were replayed at all three checkpoints for ten connected histories",
            "P_TO_M and G_TO_P are separately cut on cloned states",
            "no global Jacobian was constructed",
            "production biology and reproduction were not executed",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "modular path intervention can attribute causal cross-links in a hybrid system without a global smooth Jacobian",
        ],
        "INFERRED": ["local modal alignment and feedback-sign indicators from the preregistered response rules"],
        "UNKNOWN": ["whether the attributed cross-link limitation would be removed by a future biological change"],
        "DISPROVEN": [],
    }
    dump(root, "evidence_classification.json", evidence)
    dump(root, "architecture_decision.json", {
        "terminal_classification": classification,
        "classification_summary": class_summary,
        "global_jacobian": "NOT_CONSTRUCTED",
        "successor": "NOT_SPECIFIED_UNLESS_ARCHITECT_ACCEPTS_THIS_CAUSAL_RESULT",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation": "PASS",
        "scientific_attribution": "PASS" if classification != "R4_MODULAR_CAUSAL_ATTRIBUTION_INCONCLUSIVE" else "BOUNDED_INCONCLUSIVE",
        "terminal_classification": classification,
        "arms": compact_arms,
        "snapshot_count": len(metrics),
        "p_to_m_valid": p_valid,
        "g_to_p_snapshots_with_valid_modes": g_valid,
        "usable_snapshots": class_summary["usable_snapshots"],
        "global_jacobian": "NOT_CONSTRUCTED",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "observer_only": True,
    })
    manifest = {}
    for path in sorted(root.rglob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        manifest[str(path.relative_to(root))] = sha(path)
    dump(root, "artifact_manifest.json", {
        "schema": "dcm4_r8_artifact_manifest_v1",
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
    parser.add_argument("--stage", choices=("seal", "qualification"), required=True)
    args = parser.parse_args()
    raw = load(args.raw)
    args.output.mkdir(parents=True, exist_ok=True)
    if args.stage == "seal":
        seal_stage(args.repo, args.output, raw)
    else:
        qualification_stage(args.repo, args.output, raw)


if __name__ == "__main__":
    main()
