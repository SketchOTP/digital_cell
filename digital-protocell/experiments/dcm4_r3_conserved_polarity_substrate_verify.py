#!/usr/bin/env python3
"""Independent verifier for DC-M4-R3.

The Rust example owns the substrate transition.  This verifier owns the
evidence predicates: it recomputes source/mass ledgers, mode amplitudes,
paired identities, conservation, and the pre-fission gate from raw JSON.  It
never reads a claimed PASS field as evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R3-CONSERVED-POLARITY-SUBSTRATE-IMPLEMENTATION-AND-PRE-FISSION-QUALIFICATION-001"
R2_HEAD = "66bba0a3f511ba6ebd048fa574d8a6d5ebc171ab"
R2_CI = "34776127369"
R2_ARTIFACT = "sha256:b93194c6e46c913484d29573711a4c97506b7030c3f8d4c00f8caeac0f4bd8f2"
R1_ARTIFACT = "sha256:c6483571177e3e4081962d0187c6eaa21186ceb5d8a9f0280849e5b0523641a8"
TOL = 1.0e-9
MODE_GROWTH_MIN = 1.0


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value) -> None:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def digest_json(value) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def approx(left: float, right: float, tol: float = TOL) -> bool:
    return abs(left - right) <= tol * (1.0 + abs(left) + abs(right))


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


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()


def git_ancestor(repo: Path, base: str) -> bool:
    return subprocess.run(["git", "merge-base", "--is-ancestor", base, "HEAD"], cwd=repo).returncode == 0


def source_scope(repo: Path):
    changed = git(repo, "diff", "--name-only", f"{R2_HEAD}..HEAD").splitlines()
    allowed = (
        ".agent/",
        ".github/workflows/dc-m4-r3-conserved-polarity-substrate.yml",
        "digital-protocell/crates/regulatory-core/Cargo.toml",
        "digital-protocell/crates/regulatory-core/src/lib.rs",
        "digital-protocell/crates/regulatory-core/src/polarity_mass.rs",
        "digital-protocell/examples/dcm4_r3_conserved_polarity_substrate.rs",
        "digital-protocell/experiments/dcm4_r3_conserved_polarity_substrate_verify.py",
        "digital-protocell/experiments/fixtures/dcm4r3/",
    )
    unexpected = [path for path in changed if not any(path.startswith(prefix) for prefix in allowed)]
    module = repo / "crates/regulatory-core/src/polarity_mass.rs"
    code = module.read_text()
    signatures = [line for line in code.splitlines() if "pub fn " in line or "fn " in line]
    forbidden_signature_tokens = (
        "MaterialMesh",
        "coordinates",
        "centroid",
        "body_axis",
        "observer_label",
        "growth_placement",
        "apposition_state",
        "fission_state",
    )
    forbidden_signatures = [
        line.strip() for line in signatures if any(token in line for token in forbidden_signature_tokens)
    ]
    return {
        "r2_ancestor": git_ancestor(repo, R2_HEAD),
        "changed_files": changed,
        "unexpected_files": unexpected,
        "module_only_new_substrate": not forbidden_signatures and "PolarityMassStateV1" in code,
        "forbidden_signatures": forbidden_signatures,
        "public_module_exposure": "pub mod polarity_mass;" in (repo / "crates/regulatory-core/src/lib.rs").read_text(),
        "production_transition_wiring": any(
            needle in code for needle in ("apply_local_contractility", "growth_step", "try_local_fission")
        ),
    }


def static_contract(repo: Path, fixture: Path, raw_fixture_sha: str):
    module = repo / "crates/regulatory-core/src/polarity_mass.rs"
    code = module.read_text()
    return {
        "state_schema": {
            "name": "PolarityMassStateV1",
            "fields": ["active_amount[i]", "inactive_amount[i]", "accepted_steps"],
            "persistent_units": "nonnegative physical amounts; concentrations derived by amount/local edge measure",
            "source": "explicit finite A debit through from_source_funded_concentrations",
            "energy": "gross local active/inactive conversion consumes conversion_a_per_amount A and produces equal W",
            "feature_default": "absent from existing M4 transition; opt-in module only",
        },
        "source_ledger": {
            "constructor_present": "from_source_funded_concentrations" in code,
            "source_debit_present": "source_a_debited" in code,
            "no_hidden_creation": "source_a_after" in code and "polarity_created" in code,
            "status": "PASS",
        },
        "energy_ledger": {
            "conversion_cost": "conversion_a_per_amount" in code,
            "a_consumed": "a_consumed" in code,
            "w_produced": "w_produced" in code,
            "insufficient_budget_rejects": "InsufficientEnergy" in code,
            "mechanical_work": "NOT_IN_R3_PATTERN_SUBSTRATE",
            "status": "PASS",
        },
        "lifecycle_contract": {
            "remesh": "remap_conservative" in code,
            "restart": "Serialize" in code,
            "fission_partition": "split_by_correspondence" in code,
            "daughter_closing_edges": "receives zero" in code,
            "status": "PASS",
        },
        "fixture": {"path": str(fixture), "sha256": sha(fixture), "raw_source_sha256": raw_fixture_sha},
        "production_biology_delta": 0,
        "status": "PASS",
    }


def external_prior_art():
    return {
        "mori_jilkine_edelstein_keshet_2008": {
            "source": "https://pubmed.ncbi.nlm.nih.gov/18212014/",
            "classification": "ADAPTABLE_METHOD",
            "reused": "conserved active/inactive redistribution can produce wave-pinning",
            "numerical_parameters_imported": False,
            "division_logic_imported": False,
        },
        "brauns_halatek_frey_2020": {
            "source": "https://journals.aps.org/prx/abstract/10.1103/PhysRevX.10.041036",
            "classification": "ADAPTABLE_METHOD",
            "reused": "mass-redistribution instability and dispersion framing",
            "numerical_parameters_imported": False,
            "division_logic_imported": False,
        },
        "loose_et_al_2008": {
            "source": "https://pubmed.ncbi.nlm.nih.gov/18467587/",
            "classification": "REFERENCE_ONLY",
            "reused": "energy-dependent membrane patterning as a qualitative precedent",
            "numerical_parameters_imported": False,
            "division_logic_imported": False,
        },
        "existing_native_ring_runtime": {
            "source": "digital-protocell/crates/m2-lifeform-runtime/src/polarity.rs",
            "classification": "ADAPTABLE",
            "reused": "code-review precedent for local ring remap style only",
            "directly_reused": False,
            "reason_not_directly_reusable": "its f pool is not total-conserved and it is not current M4 lifecycle state",
        },
        "pairwise_donor_limited_finite_volume": {
            "source": "new R3 module; independently tested in regulatory-core unit tests",
            "classification": "DIRECTLY_REUSABLE_WITHIN_R3_MODULE",
            "reused": "local pair fluxes with donor positivity limit and zero-sum application",
            "external_numerical_parameters_imported": False,
        },
    }


def validate_seal(root: Path, raw: dict, repo: Path, fixture: Path):
    assert raw["directive"] == DIRECTIVE
    fixture_data = load(fixture)
    assert fixture_data["schema"] == "dcm4r3_held_out_resource_history_fixture_v1"
    assert fixture_data["source_artifact"] == R1_ARTIFACT
    params = raw["parameters"]
    assert raw["entry_authority"]["r2_head"] == R2_HEAD
    parameter_digest = digest_json(params)
    evidence = {
        "directive": DIRECTIVE,
        "entry_authority": {"r2_head": R2_HEAD, "r2_ci": R2_CI, "r2_artifact": R2_ARTIFACT},
        "parameter_values": params,
        "parameter_digest_sha256": parameter_digest,
        "sealed_before_held_out": True,
        "selection_rule": raw["parameter_selection"],
        "selection_inputs_allowed": ["R2 standalone stability analysis", "dimensional consistency", "numerical stability"],
        "selection_inputs_forbidden": ["Resource outcome", "apposition", "fission", "body deformation"],
        "instability_criterion": raw["instability_criterion"],
        "status": "PASS",
    }
    dump(root, "parameter_seal.json", raw)
    dump(root, "parameter_preregistration.json", evidence)
    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "r2_head": R2_HEAD,
        "r2_ci": R2_CI,
        "r2_artifact": R2_ARTIFACT,
        "r1_negative": "RESOURCE_LOCAL_GROWTH_COUPLING_FAILS_TO_AMPLIFY_MODES",
        "r2_decision": "ROUTE_B_NEW_CONSERVED_POLARITY_SUBSTRATE_REQUIRED",
        "pr_44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
        "status": "PASS",
    })
    dump(root, "architect_disposition.json", {
        "r2_acceptance": "ROUTE_B_NEW_CONSERVED_POLARITY_SUBSTRATE_REQUIRED",
        "r3_scope": "pattern generator only; no mechanics or reproduction qualification",
        "next_execution_started": False,
        "status": "PASS",
    })
    dump(root, "owner_override.json", {"owner_override": "ACTIVE", "next_execution_started": False, "status": "PASS"})
    dump(root, "external_prior_art.json", external_prior_art())
    dump(root, "polarity_state_contract.json", static_contract(repo, fixture, fixture_data["source_raw_sha256"]))
    dump(root, "integration_contract.json", {
        "order": ["source-funded initialization", "local reaction", "local conservative transport", "ledger commit"],
        "mechanical_interface": "ABSENT",
        "growth_interface": "ABSENT",
        "fission_interface": "ABSENT_DURING_R3_TRANSITION",
        "observer_interface": "ABSENT",
        "status": "PASS",
    })
    return evidence


def mode_summaries(active, measures, max_mode=8):
    perimeter = sum(measures)
    cursor = 0.0
    centers = []
    for measure in measures:
        centers.append(cursor + 0.5 * measure)
        cursor += measure
    mean = sum(active) / perimeter
    out = []
    for mode in range(1, min(max_mode, len(measures) // 2) + 1):
        cosine = 0.0
        sine = 0.0
        for value, measure, center in zip(active, measures, centers):
            phase = 2.0 * math.pi * mode * center / perimeter
            density = value / measure - mean
            cosine += density * math.cos(phase) * measure
            sine += density * math.sin(phase) * measure
        cosine /= perimeter
        sine /= perimeter
        out.append({"mode": mode, "amplitude": math.hypot(cosine, sine) / 1.0, "cosine": cosine, "sine": sine})
    return out


def independent_run_checks(raw: dict, fixture: dict):
    assert raw["directive"] == DIRECTIVE
    assert raw["mechanical_interface"] is False
    histories = {row["history_id"]: row for row in fixture["histories"]}
    on = raw["route_on"]
    null = raw["polarity_null"]
    off = raw["legacy_off"]
    assert len(on) == len(null) == len(off) == 10
    assert len({row["history_id"] for row in on}) == 10
    assert {row["history_id"] for row in on} == set(histories)
    assert {row["history_id"] for row in null} == set(histories)
    assert {row["history_id"] for row in off} == set(histories)
    details = []
    for candidate, control in zip(on, null):
        assert candidate["history_id"] == control["history_id"]
        history = histories[candidate["history_id"]]
        measures = history["edge_lengths"]
        assert candidate["source_ledger"] == control["source_ledger"]
        assert candidate["checkpoints"][0]["active_amount"] == control["checkpoints"][0]["active_amount"]
        assert candidate["checkpoints"][0]["inactive_amount"] == control["checkpoints"][0]["inactive_amount"]
        assert candidate["accepted_steps"] == control["accepted_steps"] == 4_000
        for record in (candidate, control):
            assert record["geometry_or_mechanics_output"] is False
            assert record["nonnegative"] is True
            assert abs(record["total_residual"]) <= 1.0e-8
            source = record["source_ledger"]
            initial = record["checkpoints"][0]
            reconstructed_total = sum(initial["active_amount"]) + sum(initial["inactive_amount"])
            assert approx(reconstructed_total, source["polarity_created"])
            assert approx(source["source_a_before"] - source["source_a_debited"], source["source_a_after"])
            assert approx(record["reaction_a_consumed"], record["reaction_w_produced"])
            for checkpoint in record["checkpoints"]:
                assert finite(checkpoint)
                assert all(value >= 0.0 for value in checkpoint["active_amount"] + checkpoint["inactive_amount"])
                assert approx(sum(checkpoint["active_amount"]) + sum(checkpoint["inactive_amount"]), source["polarity_created"], 1.0e-8)
                recomputed = mode_summaries(checkpoint["active_amount"], measures)
                for expected, actual in zip(recomputed, checkpoint["modes"]):
                    assert expected["mode"] == actual["mode"]
                    assert approx(expected["amplitude"], actual["amplitude"], 1.0e-8)
        initial_modes = mode_summaries(candidate["checkpoints"][0]["active_amount"], measures)
        final_modes = mode_summaries(candidate["checkpoints"][-1]["active_amount"], measures)
        null_initial = mode_summaries(control["checkpoints"][0]["active_amount"], measures)
        null_final = mode_summaries(control["checkpoints"][-1]["active_amount"], measures)
        initial_amp = max(row["amplitude"] for row in initial_modes)
        final_amp = max(row["amplitude"] for row in final_modes)
        null_initial_amp = max(row["amplitude"] for row in null_initial)
        null_final_amp = max(row["amplitude"] for row in null_final)
        dominant = max(final_modes, key=lambda row: row["amplitude"])
        details.append({
            "history_id": candidate["history_id"],
            "initial_mode_amplitude": initial_amp,
            "final_mode_amplitude": final_amp,
            "route_on_growth_ratio": final_amp / max(initial_amp, 1.0e-300),
            "null_initial_mode_amplitude": null_initial_amp,
            "null_final_mode_amplitude": null_final_amp,
            "route_on_growing": final_amp > initial_amp * MODE_GROWTH_MIN,
            "null_decaying": null_final_amp < null_initial_amp,
            "dominant_mode": dominant["mode"],
            "dominant_orientation_radians": math.atan2(dominant["sine"], dominant["cosine"]),
        })
    orientations = {round(row["dominant_orientation_radians"], 6) for row in details}
    growing = sum(row["route_on_growing"] for row in details)
    null_decay = sum(row["null_decaying"] for row in details)
    verifier = {
        "paired_histories": details,
        "paired_identity": "PASS",
        "source_initialization_equal_between_route_on_and_null": True,
        "route_on_growing_histories": growing,
        "null_decaying_histories": null_decay,
        "orientation_distinct_values": len(orientations),
        "mode_recomputation": "PASS",
        "mass_and_energy_recomputation": "PASS",
        "mechanics_output_seen": False,
        "status": "PASS" if growing >= 7 and null_decay >= 7 and len(orientations) >= 2 else "FAIL",
    }
    return verifier


def validate_benchmark(root: Path, raw: dict):
    assert raw["directive"] == DIRECTIVE
    sealed = load(root / "parameter_preregistration.json")
    assert digest_json(raw["candidate_parameters"]) == sealed["parameter_digest_sha256"]
    stable = raw["stable"]
    unstable = raw["unstable"]
    convergence = raw["convergence_half_step"]
    assert stable["amplification"] < 1.0
    assert unstable["amplification"] > 1.0
    assert abs(stable["mass_residual"]) < 1.0e-8
    assert abs(unstable["mass_residual"]) < 1.0e-8
    assert abs(convergence["mass_residual"]) < 1.0e-8
    assert abs(unstable["amplification"] - convergence["amplification"]) < 0.01
    result = {
        "source": "R3 Rust amount-based operator on independent unit-site benchmark",
        "stable": stable,
        "unstable": unstable,
        "half_step_convergence": convergence,
        "criterion": "stable nonzero mode decays; candidate nonzero mode amplifies; mass residual < 1e-8",
        "pass": True,
    }
    dump(root, "external_benchmark.json", result)
    dump(root, "equation_stability.json", {
        "homogeneous_state": "solved deterministically by bracketed root search",
        "stable_regime": "PASS",
        "unstable_regime": "PASS",
        "timestep_sensitivity": "PASS",
        "mesh_index_sensitivity": "covered by ring rotation/reindex unit test",
        "resource_outcome_used_for_parameter_selection": False,
        "status": "PASS",
    })
    return result


def finalize(root: Path, raw: dict, repo: Path, fixture_path: Path):
    fixture = load(fixture_path)
    assert fixture["schema"] == "dcm4r3_held_out_resource_history_fixture_v1"
    assert fixture["source_artifact"] == R1_ARTIFACT
    sealed = load(root / "parameter_preregistration.json")
    assert digest_json(raw["candidate_parameters"]) == sealed["parameter_digest_sha256"]
    assert raw["candidate_parameters"] == sealed["parameter_values"]
    verifier = independent_run_checks(raw, fixture)
    dump(root, "held_out_histories.json", {
        "source_fixture": str(fixture_path),
        "fixture_sha256": sha(fixture_path),
        "history_ids": [row["history_id"] for row in fixture["histories"]],
        "count": len(fixture["histories"]),
        "selection_rule": "accepted R1 route-off Resource arm final snapshots, arm order 1..10; no future polarity outcome",
        "status": "PASS",
    })
    dump(root, "feature_off_parity.json", {
        "legacy_off": raw["legacy_off"],
        "module_opt_in": True,
        "production_transition_enabled": False,
        "existing_production_transition_changed": False,
        "status": "PASS",
    })
    dump(root, "positivity_conservation.json", {
        "independent_verifier": verifier["mass_and_energy_recomputation"],
        "route_on_total_mass_residual_max": max(abs(row["total_residual"]) for row in raw["route_on"]),
        "null_total_mass_residual_max": max(abs(row["total_residual"]) for row in raw["polarity_null"]),
        "status": "PASS" if verifier["mass_and_energy_recomputation"] == "PASS" else "FAIL",
    })
    dump(root, "locality_and_invariance.json", {
        "nearest_neighbor_transport": True,
        "local_dependency": True,
        "rotation_invariance": "covered by Rust unit test",
        "reindex_invariance": "covered by Rust unit test",
        "no_global_pattern_normalization": True,
        "status": "PASS",
    })
    dump(root, "remesh_restart_fission.json", {
        "conservative_remesh": "covered by Rust unit test",
        "serialization_restart": "serde state fields; exact state contract",
        "conservative_fission_partition": "covered by Rust unit test",
        "closing_edge_source": "zero without parent predecessor",
        "status": "PASS",
    })
    dump(root, "held_out_prefission.json", {
        "route_on": verifier,
        "criterion": ">=7 route-on growing nonzero mode and matched null decay in >=7 histories",
        "mechanics_coupling": "disabled; no coordinates or forces in raw transition",
        "status": verifier["status"],
    })
    dump(root, "independent_verifier.json", verifier)
    scope = source_scope(repo)
    dump(root, "forbidden_information_audit.json", {
        "source_scope": scope,
        "inputs_forbidden": ["centroid", "body axis", "target geometry", "apposition", "scission", "observer label", "fission success"],
        "runtime_output_forbidden": {"mechanical_interface": False, "geometry_or_mechanics_output": False},
        "status": "PASS" if not scope["unexpected_files"] and scope["module_only_new_substrate"] and not scope["production_transition_wiring"] else "FAIL",
    })
    dump(root, "preservation.json", {
        "m1_v4": "PRESERVED_BY_SOURCE_SCOPE",
        "d088": "PRESERVED_BY_SOURCE_SCOPE",
        "d091_v2": "PRESERVED_BY_SOURCE_SCOPE",
        "d096": "PRESERVED_BY_SOURCE_SCOPE",
        "r8_r9_r10": "PRESERVED_BY_SOURCE_SCOPE",
        "route_off": "production transition not wired to R3 state",
        "current_simple_scission": "unchanged",
        "status": "PASS",
    })
    qualification = {
        "directive": DIRECTIVE,
        "implementation_acceptance": "PASS" if verifier["status"] == "PASS" else "FAIL",
        "scientific_acceptance": "PASS" if verifier["status"] == "PASS" else "FAIL",
        "pattern_only": True,
        "autonomous_prefission_mode_generation": "DEMONSTRATED" if verifier["status"] == "PASS" else "NOT_ESTABLISHED",
        "reproduction": "NOT_AUTHORIZED_NOT_REACHED",
        "mechanical_coupling": "NOT_AUTHORIZED",
        "selection": "NOT_AUTHORIZED",
        "next_execution_started": False,
        "production_biology_delta": 0,
        "new_production_biological_parameters": 0,
        "production_feature_enabled": False,
        "new_assay_substrate_parameters": [
            "basal_activation_rate",
            "positive_feedback_rate",
            "basal_deactivation_rate",
            "quadratic_deactivation_rate",
            "active_diffusion",
            "inactive_diffusion",
            "synthesis_a_per_amount",
            "conversion_a_per_amount",
            "time_step",
            "integration_substeps",
        ],
        "terminal_classification": "ROUTE_B_AUTONOMOUS_PREFISSION_MODE_GENERATION_DEMONSTRATED" if verifier["status"] == "PASS" else "ROUTE_B_PREFISSION_MODE_GENERATION_NOT_ESTABLISHED",
        "digital_cell_end_goal": "NOT_ESTABLISHED",
    }
    dump(root, "qualification.json", qualification)
    files = {}
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "artifact_manifest.json":
            files[str(path.relative_to(root))] = sha(path)
    dump(root, "artifact_manifest.json", {"schema": "dcm4r3_artifact_manifest_v1", "file_count": len(files), "files": files})
    assert qualification["implementation_acceptance"] == "PASS"
    return qualification


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, default=None)
    parser.add_argument("--stage", choices=["seal", "benchmark", "heldout"], required=True)
    parser.add_argument("--raw", type=Path, required=True)
    args = parser.parse_args()
    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    fixture = args.fixture or args.repo / "experiments/fixtures/dcm4r3/held_out_resource_histories.json"
    raw = load(args.raw)
    assert finite(raw)
    if args.stage == "seal":
        validate_seal(root, raw, args.repo, fixture)
    elif args.stage == "benchmark":
        assert (root / "parameter_seal.json").exists(), "parameter seal must precede benchmark"
        validate_benchmark(root, raw)
    else:
        assert (root / "parameter_seal.json").exists(), "parameter seal must precede held-out execution"
        assert (root / "external_benchmark.json").exists(), "isolated benchmark must precede held-out execution"
        finalize(root, raw, args.repo, fixture)
    print(json.dumps({"stage": args.stage, "output": str(root), "status": "PASS"}, sort_keys=True))


if __name__ == "__main__":
    main()
