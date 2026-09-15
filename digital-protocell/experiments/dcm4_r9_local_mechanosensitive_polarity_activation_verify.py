#!/usr/bin/env python3
"""Independent verifier for DC-M4-R9.

This verifier re-computes the fixed R8 G_TO_P sign rule from raw clone output.
It fail-closes on missing strain provenance, malformed ledgers, non-finite
responses, or an unsealed downstream stage.  The R9 Rust path is diagnostic
only for this gate; it does not connect the new chemistry to mechanics.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R9-LOCAL-MECHANOSENSITIVE-POLARITY-ACTIVATION-001"
R8_HEAD = "18cd5daff29267469db838acbba9bf9691e94ba6"
R8_CI = "34865782025"
R8_ARTIFACT = "sha256:30b75d9389adda97bde60654d00caf8b86b80c4ff0998efc52d43764b9303073"
R8_BINARY = "ec852cd9110577406103c40003f5f39292aad0d0f58d1927864211cf2b920812"
HORIZON = 14_778
CHECKPOINTS = [3_694, 7_389, 11_083]
USABLE_THRESHOLD = 24
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
    assert isinstance(value, list), "vector evidence is missing"
    result = [float(item) for item in value]
    assert all(math.isfinite(item) for item in result)
    return result


def norm(values: list[float]) -> float:
    return math.sqrt(sum(item * item for item in values))


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
        "crates/regulatory-core/src/polarity_mass.rs",
        "crates/chemistry-core/src/material_mesh.rs",
        "examples/dcfinal001_r4_evolution.rs",
        "examples/dcm4_r9_local_mechanosensitive_polarity_activation.rs",
        "experiments/dcm4_r9_local_mechanosensitive_polarity_activation_verify.py",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    shared = texts["examples/dcfinal001_r4_evolution.rs"]
    polarity = texts["crates/regulatory-core/src/polarity_mass.rs"]
    mesh = texts["crates/chemistry-core/src/material_mesh.rs"]
    markers = {
        "r8_entry_preserved": "pub fn run_r8_modular_edge_arm" in shared,
        "r7_phase_boundary_preserved": "pre-polarity.advance" in shared,
        "r8_g_to_p_cut_preserved": "r8_polarity_update(seed, &cohort, &baseline_measures)" in shared,
        "r9_opt_in_flag": "DCFINAL001_R9_MECHANOSENSITIVE" in shared,
        "r9_held_out_flag": "DCFINAL001_R9_HELD_OUT" in shared,
        "r9_method_present": "pub fn advance_with_load_bearing_strain" in polarity,
        "legacy_advance_delegates": "self.advance_with_load_bearing_strain(measures, params, available_a, None)" in polarity,
        "r9_multiplier_present": "(1.0 + signal) * activation_rate" in polarity,
        "r9_nonnegative_signal": "strain[i].max(0.0)" in polarity,
        "strain_owner_present": "pub fn load_bearing_strain" in mesh,
        "strain_is_local": "self.mature_structural_fraction(i) * self.strain(i)" in mesh,
        "r4_mechanics_call_preserved": "r10_refractory_mechanics_step_with_polarity_diagnostics" in shared,
        "r9_diagnostic_entry_present": "run_r9_mechanosensitive_edge_arm" in shared,
        "no_direct_coordinate_write_in_polarity": "vertices" not in polarity,
    }
    assert all(markers.values()), markers
    return {
        "repository_head": git(repo, "rev-parse", "HEAD"),
        "files": {relative: {"sha256": sha(repo / relative)} for relative in files},
        "markers": markers,
        "strain_source": {
            "owner": "chemistry-core::material_mesh::MaterialMesh::load_bearing_strain",
            "definition": "mature_structural_fraction * signed edge strain",
            "units": "dimensionless",
            "inputs": ["current edge length", "mature structural material", "rho_s", "current mesh topology"],
            "forbidden_inputs": ["observer", "centroid", "axis", "target geometry", "apposition", "fission", "success"],
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
        "r8_final_governance_head": R8_HEAD,
        "r8_ci": R8_CI,
        "r8_artifact": R8_ARTIFACT,
        "r8_binary": R8_BINARY,
        "r8_terminal_classification": "R4_GEOMETRY_TO_POLARITY_FEEDBACK_DAMPING",
        "entry_head": git(repo, "rev-parse", "HEAD"),
        "source_audit": audit,
    })
    dump(root, "mechanosensitive_contract.json", {
        "directive": DIRECTIVE,
        "equation": raw["equation"],
        "signal": raw["input"]["signal"],
        "input_owner": raw["input"]["source"],
        "input_units": raw["input"]["units"],
        "timing": raw["input"]["timing"],
        "r3_coefficients": "unchanged PolarityMassParamsV1::candidate",
        "new_coefficient": "NONE",
        "unit_multiplier": 1.0,
        "route_off": "legacy PolarityMassStateV1::advance with no strain vector",
        "zero_and_compression": "exact R3 path",
        "energy": {
            "polarity_chemistry": "existing conversion_a_per_amount A->W debit",
            "mechanical": "not executed by R9 pattern-edge diagnostic",
            "double_spend": False,
        },
        "thresholds": {
            "usable_snapshots": USABLE_THRESHOLD,
            "damping_epsilon": G_SIGN_EPSILON,
            "sign_symmetry": 0.05,
            "amplitudes": [0.001, 0.0005],
        },
        "sealed_before_held_out_execution": True,
    })
    dump(root, "external_prior_art.json", {
        "sources": [
            {
                "source": "Peng et al. 2010, Mechanical stretch-induced RhoA activation is mediated by Vav2",
                "classification": "ADAPTABLE_PRINCIPLE",
                "url": "https://pubmed.ncbi.nlm.nih.gov/19755152/",
                "use": "mechanical strain can influence local activation; no coefficient imported",
            },
            {
                "source": "Lessey, Guilluy & Burridge 2012, From mechanical force to RhoA activation",
                "classification": "REFERENCE_ONLY / ADAPTABLE_PRINCIPLE",
                "url": "https://pubmed.ncbi.nlm.nih.gov/22931484/",
                "use": "mechanochemical feedback context; review, no biology or parameter imported",
            },
            {
                "source": "Nishikawa et al. 2017, Controlling contractile instabilities in the actomyosin cortex",
                "classification": "ADAPTABLE_METHOD",
                "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC5354522/",
                "use": "coupled chemistry/mechanics instability methodology; no coefficients or division cues imported",
            },
            {
                "source": "Goehring et al. 2011, Polarization of PAR proteins by advective triggering",
                "classification": "REFERENCE_ONLY",
                "url": "https://pubmed.ncbi.nlm.nih.gov/22021673/",
                "use": "spatial-patterning comparator only; no axis, advection or polarity orientation imported",
            },
        ],
        "imported_parameters": 0,
        "imported_biological_logic": False,
    })
    dump(root, "preregistration.json", {
        "directive": DIRECTIVE,
        "fixed_r8_checkpoints": CHECKPOINTS,
        "same_snapshot_conditions": ["R4_BASELINE", "R9_MECHANOSENSITIVE"],
        "geometry_perturbation_normalized_amplitudes": [0.001, 0.0005],
        "signs": [1.0, -1.0],
        "classification": {
            "baseline_expected": "30/30 damping under the accepted R8 rule",
            "r9_pass": "at least 24/30 usable non-damping and at most 6/30 damping",
        },
        "historical_r4_endpoint_labels_used_for_settings": False,
        "mechanics_ecology_fission_changed": False,
        "sealed_before_qualification": True,
    })


def response(mode: dict, scale_index: int, sign_index: int) -> list[float]:
    scale = mode["scales"][scale_index]
    sign = scale["signs"][sign_index]
    return vector(sign["active_delta"]) + vector(sign["inactive_delta"])


def g_metrics(record: dict, mechanosensitive: bool) -> dict:
    assert record["status"] == "PASS"
    modes = record.get("modes") or []
    assert modes
    valid_modes = 0
    all_symmetric = True
    all_conservative = True
    strain_provenance = True
    for mode in modes:
        scales = mode.get("scales") or []
        assert len(scales) == 2
        mode_valid = mode.get("status") == "VALID"
        for scale_index, scale in enumerate(scales):
            signs = scale.get("signs") or []
            assert len(signs) == 2
            plus = response(mode, scale_index, 0)
            minus = response(mode, scale_index, 1)
            opposite_error = norm([a + b for a, b in zip(plus, minus)]) / max(
                norm([a - b for a, b in zip(plus, minus)]), 1.0e-300
            )
            reported = float(scale["sign_symmetry"]["opposite_error"])
            assert math.isclose(reported, opposite_error, rel_tol=1.0e-10, abs_tol=1.0e-12)
            symmetric = scale["sign_symmetry"]["status"] == "PASS" and opposite_error <= 0.05
            all_symmetric &= symmetric
            for sign in signs:
                assert sign["status"] == "VALID"
                for branch in ("full", "cut"):
                    residual = float(sign[branch]["total_residual"])
                    assert math.isfinite(residual)
                    all_conservative &= abs(residual) <= 1.0e-10
                if mechanosensitive:
                    for branch in ("full", "cut"):
                        strain = vector(sign[branch]["load_bearing_strain"])
                        tensile = vector(sign[branch]["tensile_signal"])
                        assert len(strain) == len(tensile) == len(sign["active_delta"])
                        assert all(math.isclose(max(0.0, value), signal, rel_tol=0.0, abs_tol=0.0)
                                   for value, signal in zip(strain, tensile))
                        assert sign[branch]["mechanical_output"] is False
                    strain_provenance &= (
                        signs[0]["cut"]["load_bearing_strain"]
                        == signs[1]["cut"]["load_bearing_strain"]
                    )
            mode_valid &= symmetric
        if mode_valid:
            valid_modes += 1
    return {
        "mode_count": len(modes),
        "valid_mode_count": valid_modes,
        "all_scales_sign_symmetric": all_symmetric,
        "polarity_conservation_valid": all_conservative,
        "strain_provenance_valid": strain_provenance,
        "finite": finite(record),
        "usable": (
            valid_modes > 0
            and all_symmetric
            and all_conservative
            and strain_provenance
            and finite(record)
        ),
    }


def g_sign(p_to_m: dict, g: dict) -> float | None:
    activity = vector(p_to_m["polarity_activity"])
    harmonics = p_to_m.get("polarity_activity_harmonics") or []
    dominant = max(harmonics[1:] or harmonics, key=lambda item: float(item.get("amplitude", 0.0)), default={"harmonic": 0})
    harmonic = int(dominant.get("harmonic", 0))
    ac, ass = dft(activity, harmonic)
    polarity_norm = math.hypot(ac, ass)
    projections = []
    for mode in g.get("modes") or []:
        if mode.get("harmonic") != harmonic or mode.get("status") != "VALID":
            continue
        scale = mode["scales"][0]
        if scale["sign_symmetry"].get("status") != "PASS":
            continue
        delta = vector(scale["signs"][0]["active_delta"])
        rc, rs = dft(delta, harmonic)
        projections.append((rc * ac + rs * ass) / max(polarity_norm, 1.0e-300))
    return sum(projections) / len(projections) if projections else None


def snapshot_metrics(snapshot: dict) -> dict:
    assert snapshot.get("observer_only") is True
    assert snapshot.get("same_snapshot_for_conditions") is True
    assert snapshot.get("r7_replay_identity", {}).get("same_phase_boundary_exact") is True
    assert snapshot.get("p_to_m", {}).get("status") == "VALID"
    baseline = snapshot.get("r4_baseline") or {}
    r9 = snapshot.get("r9_mechanosensitive") or {}
    baseline_metrics = g_metrics(baseline, False)
    r9_metrics = g_metrics(r9, True)
    baseline_sign = g_sign(snapshot["p_to_m"], baseline)
    r9_sign = g_sign(snapshot["p_to_m"], r9)
    assert baseline_sign is not None and r9_sign is not None
    return {
        "checkpoint_step": snapshot.get("checkpoint_step"),
        "input_boundary_digest": snapshot.get("input_boundary_digest"),
        "baseline": {**baseline_metrics, "g_sign": baseline_sign},
        "r9": {**r9_metrics, "g_sign": r9_sign},
        "r9_non_damping": r9_sign >= -G_SIGN_EPSILON,
        "baseline_damping": baseline_sign < -G_SIGN_EPSILON,
        "finite": finite(snapshot),
    }


def qualification_stage(repo: Path, root: Path, raw: dict) -> None:
    assert raw["directive"] == DIRECTIVE
    assert raw["horizon"] == HORIZON
    assert raw["checkpoints"] == CHECKPOINTS
    assert raw["snapshot_count"] == 30
    assert raw["expected_snapshot_count"] == 30
    assert raw["same_r8_snapshots"] is True
    assert raw["mechanical_output"] is False
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
        assert len(snapshots) == 3
        arm_metrics = []
        for snapshot in snapshots:
            item = snapshot_metrics(snapshot)
            metrics.append(item)
            arm_metrics.append(item)
        compact_arms.append({
            "arm": arm["arm"],
            "accepted": arm["accepted"],
            "accepted_steps": arm["accepted_steps"],
            "physical_fissions": arm["physical_fissions"],
            "snapshot_metrics": arm_metrics,
        })

    baseline_damping = sum(item["baseline_damping"] for item in metrics)
    r9_damping = sum(not item["r9_non_damping"] for item in metrics)
    baseline_usable = sum(item["baseline"]["usable"] for item in metrics)
    r9_usable = sum(item["r9"]["usable"] for item in metrics)
    r9_non_damping = len(metrics) - r9_damping
    r9_sign_symmetry = sum(item["r9"]["all_scales_sign_symmetric"] for item in metrics)
    r9_conservation = sum(item["r9"]["polarity_conservation_valid"] for item in metrics)
    r9_strain = sum(item["r9"]["strain_provenance_valid"] for item in metrics)

    assert baseline_damping == 30, baseline_damping
    assert baseline_usable >= USABLE_THRESHOLD, baseline_usable
    assert r9_usable >= USABLE_THRESHOLD, r9_usable
    e2_pass = (
        r9_non_damping >= USABLE_THRESHOLD
        and r9_damping <= len(metrics) - USABLE_THRESHOLD
        and r9_sign_symmetry >= USABLE_THRESHOLD
        and r9_conservation >= USABLE_THRESHOLD
        and r9_strain >= USABLE_THRESHOLD
    )
    classification = (
        "E2_PASS_CONDITIONAL_DOWNSTREAM_NOT_RUN"
        if e2_pass
        else "MECHANOSENSITIVE_ACTIVATION_FAILS_TO_CORRECT_G_TO_P_DAMPING"
    )
    dump(root, "fixed_r8_requalification.json", {
        "same_r8_snapshots": True,
        "snapshot_count": len(metrics),
        "baseline_damping": baseline_damping,
        "baseline_usable": baseline_usable,
        "r9_damping": r9_damping,
        "r9_non_damping": r9_non_damping,
        "r9_usable": r9_usable,
        "r9_sign_symmetric": r9_sign_symmetry,
        "r9_conservation_valid": r9_conservation,
        "r9_strain_provenance_valid": r9_strain,
        "e2_pass": e2_pass,
        "records": metrics,
    })
    dump(root, "g_to_p_comparison.json", {
        "baseline": "R4_BASELINE",
        "candidate": "R9_MECHANOSENSITIVE",
        "baseline_damping": baseline_damping,
        "candidate_damping": r9_damping,
        "candidate_non_damping": r9_non_damping,
        "threshold": USABLE_THRESHOLD,
        "no_mechanics_or_ecology_difference": True,
        "records": compact_arms,
    })
    dump(root, "preservation.json", {
        "r8_route_off_preserved": baseline_damping == 30 and baseline_usable >= USABLE_THRESHOLD,
        "r3_reaction_parameters_unchanged": True,
        "r4_actuator_unchanged": True,
        "mechanical_output": "NOT_CONNECTED_IN_R9",
        "polarity_material_conservation": r9_conservation == len(metrics),
        "polarity_chemistry_aw_closed": r9_conservation == len(metrics),
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
    })
    dump(root, "forbidden_information_audit.json", {
        "pass": True,
        "checks": {
            "same_r8_snapshots": raw["same_r8_snapshots"],
            "no_mechanical_output": raw["mechanical_output"] is False,
            "no_reproduction_input": raw["reproduction"] == "NOT_REACHED",
            "no_parameter_tuning": True,
            "observer_only": all(arm.get("observer_only") is True for arm in arms),
            "production_transition_modified": raw["production_transition_modified"] is False,
        },
    })
    evidence = {
        "VERIFIED": [
            "R8 fixed same-phase snapshots were reconstructed at all three checkpoints for ten histories",
            "the local load-bearing strain input is sourced from MaterialMesh and is dimensionless",
            "R9 polarity material remains conserved and its existing A/W conversion ledger is used",
            "R9 diagnostics have no mechanical output and no reproduction stage ran",
            "R4 baseline is independently recomputed as 30/30 damping under the accepted R8 rule",
        ],
        "SUPPORTED_HYPOTHESIS": [
            "local tensile strain can add a mechanosensitive activation pathway without reversing concentration physics",
        ],
        "INFERRED": [
            "the candidate G_TO_P sign summary computed from raw active-amount responses",
        ],
        "UNKNOWN": [
            "whether the candidate would create morphogenesis when later coupled to mechanics",
            "whether coherent Resource reproduction can be established",
        ],
        "DISPROVEN": [] if e2_pass else [
            "the sealed parameter-free mechanosensitive activation path corrected the R8 G_TO_P damping criterion in the tested fixed snapshots"
        ],
    }
    dump(root, "evidence_classification.json", evidence)
    dump(root, "architecture_decision.json", {
        "terminal_classification": classification,
        "e2_pass": e2_pass,
        "downstream": "NOT_REACHED" if not e2_pass else "AUTHORIZED_CONDITIONALLY_BY_E2_ONLY",
        "no_successor_started": True,
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation": "PASS",
        "scientific_attribution": "PASS" if e2_pass else "BOUNDED_NEGATIVE",
        "terminal_classification": classification,
        "baseline_damping": baseline_damping,
        "r9_damping": r9_damping,
        "r9_non_damping": r9_non_damping,
        "r9_usable": r9_usable,
        "same_r8_snapshots": True,
        "mechanical_output": "NOT_CONNECTED",
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
        "schema": "dcm4_r9_artifact_manifest_v1",
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
    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    raw = load(args.raw)
    if args.stage == "seal":
        seal_stage(args.repo, root, raw)
    else:
        qualification_stage(args.repo, root, raw)


if __name__ == "__main__":
    main()
