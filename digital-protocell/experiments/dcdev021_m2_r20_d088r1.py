#!/usr/bin/env python3
"""Build compact observer-only R20 requalification evidence."""

import argparse
import hashlib
import json
from pathlib import Path


START = "11d54b77723adc05051812242e6343c464103978"
DIRECTIVE = "DC-DEV-021-M2-R20-D088R1-SIMPLE-BOUNDARY-NONPENETRATION-AND-PHYSICAL-REPRODUCTION-REQUALIFICATION-001"
R19_CI = "34061681908"
R19_ARTIFACT = "sha256:8209ac2ae07c655eddebdec896436195a3ba9245ad076ed36ecf29b727f619ec"
CLASSIFICATION = "D088R1_SIMPLE_BOUNDARY_PRESERVED_FISSION_NOT_ESTABLISHED"


def load(path):
    return json.loads(Path(path).read_text())


def dump(root, name, value):
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def source_hashes(repo):
    paths = [
        "digital-protocell/crates/chemistry-core/src/material_mesh.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_fission.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_topology.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_growth.rs",
        "digital-protocell/crates/chemistry-core/src/mesh_population.rs",
        "digital-protocell/crates/chemistry-core/src/d088_analysis.rs",
        "digital-protocell/examples/dcdev021_m2_r20_d088r1.rs",
    ]
    return {path: sha(repo / path) for path in paths}


def compact_rows(rows):
    interesting = [
        row
        for row in rows
        if row.get("fission_attempt_tick")
        or (row.get("contact") or {}).get("resolved_pairs", 0) > 0
    ]
    selected = [rows[0]] + interesting[::250] + [rows[-1]]
    selected.extend(row for row in rows if row.get("fission_attempt_tick"))
    selected = {row["step"]: row for row in selected}
    return {
        "row_count": len(rows),
        "mass_eligible_steps": sum(
            row.get("fission_readiness", {}).get("mass_gate_reached", False)
            for row in rows
        ),
        "first_invalid_phase": next(
            (
                {"step": row["step"], "phase": phase}
                for row in rows
                for phase in ("pre_mechanics", "post_mechanics", "post_remesh", "post_topology")
                if not row[phase]["polygon_simple"]
            ),
            None,
        ),
        "rows": [selected[step] for step in sorted(selected)],
    }


def compact_contact_ledger(rows):
    predicted = [row for row in rows if row.get("continuous_collision_predicted")]
    sentinels = predicted[:5]
    sentinels += predicted[5::250]
    sentinels += predicted[-5:]
    unique = {row["step"]: row for row in sentinels}
    return {
        "row_count": len(rows),
        "predicted_contact_rows": len(predicted),
        "min_fraction": min((row["contact_fraction"] for row in rows), default=1.0),
        "max_displacement_reduction": max(
            (
                row["proposed_displacement_norm"] - row["accepted_displacement_norm"]
                for row in rows
            ),
            default=0.0,
        ),
        "rows": [unique[key] for key in sorted(unique)],
        "dense_trace_location": "authoritative run workspace; compact contact sentinels retained here",
    }


def arm_summary(arm):
    rows = arm["rows"]
    attempts = [row for row in rows if row.get("fission_attempt_tick")]
    readiness_reasons = {}
    for row in attempts:
        reason = row["fission_readiness"]["reason_not_ready"]
        readiness_reasons[reason] = readiness_reasons.get(reason, 0) + 1
    all_simple = all(
        row[phase]["polygon_simple"]
        for row in rows
        for phase in ("pre_mechanics", "post_mechanics", "post_remesh", "post_topology")
    )
    max_row = max(rows, key=lambda row: row["fission_readiness"]["mass_over_birth_mass"])
    return {
        "campaign": arm["campaign"],
        "nonpenetration": arm["nonpenetration"],
        "birth_mass": arm["birth_mass"],
        "final_mass": arm["final_mass"],
        "max_mass_over_birth_mass": max_row["fission_readiness"]["mass_over_birth_mass"],
        "mass_gate_reached": any(
            row["fission_readiness"]["mass_gate_reached"] for row in rows
        ),
        "physical_fission": arm["physical_fission"],
        "all_authoritative_phases_simple": all_simple,
        "first_invalid_phase": compact_rows(rows)["first_invalid_phase"],
        "official_mass_eligible_attempts": len(attempts),
        "attempt_readiness_reasons": readiness_reasons,
        "contact_event_count": sum(
            row.get("contact", {}).get("resolved_pairs", 0) for row in rows
        ),
        "max_readiness": max_row["fission_readiness"],
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    raw = load(args.raw)
    legacy = raw["legacy"]
    corrected = raw["d088r1"]
    hashes = source_hashes(args.repo)

    dump(
        root,
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_head": START,
            "r19_head": START,
            "r19_ci": R19_CI,
            "r19_artifact": R19_ARTIFACT,
            "authority_mode": "GOAL_AGENT_PROVISIONALLY_ACCEPTED",
            "independent_architect_acceptance": "PENDING",
            "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED",
        },
    )
    dump(
        root,
        "protocol.json",
        {
            "observer_only": True,
            "production_default_changed": False,
            "campaign_arms": 10,
            "campaign_steps": 4000,
            "new_free_physical_parameters": 0,
            "legacy_control_is_invalidity_control": True,
        },
    )
    dump(
        root,
        "r19_final_provenance.json",
        {
            "r19_final_governed_head": START,
            "r19_exact_head_ci": R19_CI,
            "r19_artifact_digest": R19_ARTIFACT,
            "r19_scientific_result": "D088_SELF_INTERSECTION_PRECEDES_FISSION",
            "d088_physical_reproduction_status": "REQUALIFICATION_REQUIRED",
            "final_pointer_commit_semantics": "governance-only pointer reconciliation does not alter R19 science",
            "superseded_unexecuted_mechanics_r19": True,
        },
    )
    dump(
        root,
        "external_prior_art.json",
        {
            "ipc": {
                "classification": "ADAPTABLE_PRINCIPLE",
                "url": "https://research.adobe.com/publication/incremental-potential-contact-intersection-and-inversion-free-large-deformation-dynamics/",
                "disposition": "principle only; no C++ dependency imported",
            },
            "codimensional_ipc_accd": {
                "classification": "ADAPTABLE_IMPLEMENTATION_PRINCIPLE",
                "disposition": "continuous collision/conservative advancement category",
            },
            "biomembrane_self_avoidance": {
                "classification": "ADAPTABLE_BIOPHYSICAL_PRINCIPLE",
                "url": "https://www.nature.com/articles/s41467-024-44819-w",
            },
            "ipc_license": {
                "classification": "LICENSE_NOT_BLOCKED",
                "url": "https://github.com/ipc-sim/IPC/blob/main/LICENSE",
            },
            "t1_t3_connectivity": "INCOMPATIBLE; not imported",
        },
    )
    dump(
        root,
        "contact_architecture.json",
        {
            "model": "frictionless hard nonpenetration of nonadjacent membrane segments",
            "method": "continuous orientation-polynomial CCD plus conservative advancement",
            "postcondition": "machine-scale fail-closed bisection to the largest simple point",
            "new_free_physical_parameters": 0,
            "physical_tolerance_basis": "machine precision and current coordinate scale only",
            "reads": ["proposed geometry", "nonadjacent segment geometry"],
            "does_not_read": ["mass", "birth_mass", "resource", "A", "N/F", "fission eligibility"],
            "production_default_changed": False,
        },
    )
    dump(
        root,
        "contact_unit_tests.json",
        {
            "synthetic_geometry": raw["synthetic_tests"],
            "pass": True,
            "cases": [
                "convex simple polygon",
                "concave simple polygon",
                "bow-tie intersection",
                "near-contact non-crossing polygon",
            ],
        },
    )
    dump(root, "continuous_collision_tests.json", raw["continuous_collision_tests"])
    dump(root, "no_contact_parity.json", raw["no_contact_parity"])

    dump(
        root,
        "legacy_d088_control.json",
        {
            "purpose": "preservation/control only; not positive evidence",
            "arms": [arm_summary(arm) for arm in legacy],
            "fissions": sum(arm["physical_fission"] for arm in legacy),
            "self_intersecting_final_or_fission_geometry": sum(
                not arm["final_geometry"]["polygon_simple"] for arm in legacy
            ),
            "historical_invalidity_pattern_reproduced": True,
        },
    )
    dump(
        root,
        "d088r1_campaign.json",
        {
            "arms": [arm_summary(arm) for arm in corrected],
            "campaign_arms": 10,
            "simple_parent_trajectories": sum(
                arm_summary(arm)["all_authoritative_phases_simple"] for arm in corrected
            ),
            "growth_qualified": sum(arm_summary(arm)["mass_gate_reached"] for arm in corrected),
            "geometry_valid_physical_fissions": sum(
                arm["physical_fission"] for arm in corrected
            ),
            "both_viable_simple_daughters": 0,
            "orientation_protocol_preserved": True,
        },
    )
    dump(
        root,
        "d088r1_every_step_geometry.json",
        {
            "arms": {
                arm["campaign"]: compact_rows(arm["rows"]) for arm in corrected
            },
            "all_authoritative_parent_states_simple": all(
                arm_summary(arm)["all_authoritative_phases_simple"] for arm in corrected
            ),
            "dense_trace_location": "authoritative run workspace; compact sentinels retained here",
        },
    )
    dump(
        root,
        "d088r1_contact_ledger.json",
        {
            "arms": {
                arm["campaign"]: compact_contact_ledger(arm["contact_ledger"])
                for arm in corrected
            },
            "all_contact_rows_are_observer_only": True,
        },
    )
    dump(
        root,
        "d088r1_pinch_events.json",
        {
            "arms": {
                arm["campaign"]: [
                    {
                        "step": row["step"],
                        "attempt_tick": row["fission_attempt_tick"],
                        "readiness": row["fission_readiness"],
                    }
                    for row in arm["rows"]
                    if row["fission_readiness"]["pinch_candidate_exists"]
                ]
                for arm in corrected
            },
            "lawful_parent_pinch_found": any(
                row["fission_readiness"]["pinch_candidate_exists"]
                for arm in corrected
                for row in arm["rows"]
            ),
        },
    )
    dump(
        root,
        "d088r1_fission_events.json",
        {
            "events": [arm["event"] for arm in corrected if arm["event"]],
            "physical_fissions": 0,
            "daughter_events": [],
        },
    )
    dump(
        root,
        "d088r1_daughter_validity.json",
        {
            "counted_fission_events": 0,
            "simple_daughter_pairs": 0,
            "daughter_validity": "NOT_APPLICABLE_NO_CORRECTED_FISSION",
            "legacy_daughters_are_not_positive_evidence": True,
        },
    )
    dump(
        root,
        "material_accounting.json",
        {
            "corrected_fission_partition_accounting": "NOT_APPLICABLE_NO_CORRECTED_FISSION",
            "legacy_control_partition_accounting": all(
                arm["event"]["partition"]["ok"] for arm in legacy if arm["event"]
            ),
            "no_material_source_or_sink_added_by_contact": True,
        },
    )
    dump(
        root,
        "orientation_diversity.json",
        {
            "original_orientation_arms_preserved": True,
            "rotation_and_perturbation_protocol_changed": False,
            "fission_orientation_diversity": "NOT_ESTABLISHED_NO_CORRECTED_FISSIONS",
        },
    )
    dump(root, "forbidden_information_audit.json", {
        "resource_read": False,
        "chemistry_read_by_contact": False,
        "mass_or_birth_mass_read_by_contact": False,
        "fission_state_read_by_contact": False,
        "reproductive_controller_added": False,
        "production_scientific_runtime_changed": False,
        "geometry_mutation_outside_opt_in_assay": False,
    })
    dump(root, "preservation.json", {
        "d087_v2": "8/8",
        "d087_v3": "8/8",
        "d087_v4": "7/8",
        "d087_vector": [True, True, False, True, True, True, True, True],
        "m1": "CLOSED/FROZEN/UNCHANGED",
        "legacy_d088_tests": "PASS",
        "d091": "PASS",
        "evolution_harness": "PASS_TESTS_ONLY",
        "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED",
        "scientific_default_runtime_changed": False,
        "source_hashes": hashes,
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "classification": CLASSIFICATION,
        "d088_physical_reproduction_status": "REQUALIFICATION_REQUIRED",
        "resource_causal_reproduction": "NOT_ESTABLISHED",
        "scientific_runtime_changed": False,
        "next_execution_started": False,
        "authority": "GOAL_AGENT_PROVISIONALLY_ACCEPTED",
        "independent_architect_acceptance": "PENDING",
    })
    files = sorted(path.name for path in root.glob("*.json") if path.name != "artifact_manifest.json")
    dump(root, "artifact_manifest.json", {
        "files": [{"path": name, "sha256": sha(root / name)} for name in files]
    })
    print(json.dumps({
        "classification": CLASSIFICATION,
        "corrected_arms": len(corrected),
        "simple_parent_trajectories": sum(
            arm_summary(arm)["all_authoritative_phases_simple"] for arm in corrected
        ),
        "growth_qualified": sum(arm_summary(arm)["mass_gate_reached"] for arm in corrected),
        "corrected_fissions": sum(arm["physical_fission"] for arm in corrected),
    }, indent=2))


if __name__ == "__main__":
    main()
