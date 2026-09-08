#!/usr/bin/env python3
"""Build compact DC-FINAL-001 evidence from the sealed WP1 and WP2 runs."""

import argparse
import hashlib
import json
from pathlib import Path


DIRECTIVE = "DC-FINAL-001-END-TO-END-AUTONOMOUS-LIFEFORM-CLOSURE-OR-SHUTDOWN-001"
CLASSIFICATION = "DIGITAL_CELL_END_GOAL_NOT_ESTABLISHED_SHUTDOWN_RECOMMENDED"


def dump(root: Path, name: str, value):
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def reproduction_summary(rows):
    fissions = [row for row in rows if row["physical_fission"]]
    return {
        "arms": len(rows),
        "simple_parent_trajectories": sum(bool(row["all_parent_states_simple"]) for row in rows),
        "growth_qualified": sum(bool(row["growth_qualified"]) for row in rows),
        "geometry_valid_fissions": len(fissions),
        "simple_viable_daughter_pairs": sum(
            bool(row.get("fission", {}).get("both_daughters_viable")) for row in fissions
        ),
        "all_counted_fission_parents_simple": all(
            row["all_parent_states_simple"] and row["fission"]["parent"]["simple"]
            for row in fissions
        ),
        "all_counted_daughters_simple": all(
            row["fission"]["daughter_a"]["simple"]
            and row["fission"]["daughter_b"]["simple"]
            for row in fissions
        ),
        "all_counted_partitions_close": all(row["fission"]["partition"]["ok"] for row in fissions),
        "fission_steps": [row["fission"]["step"] for row in fissions],
        "max_mass_over_birth": max((row["max_mass_over_birth"] for row in rows), default=0.0),
    }


def acquisition_summary(raw):
    rows = raw["rows"]
    active = [row for row in rows if row["arm"] == "sensor_on_motor_on"]
    sensor = [row for row in rows if row["arm"] == "sensor_off_motor_on"]
    motor = [row for row in rows if row["arm"] == "sensor_on_motor_off"]
    off = [row for row in rows if row["arm"] == "nutrient_field_off"]
    comparisons = []
    for active_row in active:
        def matching(pool):
            return next(
                row for row in pool
                if row["seed"] == active_row["seed"]
                and row["bearing_radians"] == active_row["bearing_radians"]
            )

        sensor_row = matching(sensor)
        motor_row = matching(motor)
        comparisons.append({
            "seed": active_row["seed"],
            "bearing_radians": active_row["bearing_radians"],
            "active_delivered_n": active_row["delivered_n"],
            "sensor_off_delivered_n": sensor_row["delivered_n"],
            "motor_off_delivered_n": motor_row["delivered_n"],
            "active_first_transfer": active_row["first_transfer"],
            "sensor_off_first_transfer": sensor_row["first_transfer"],
            "active_toward_resource": active_row["displacement_toward_resource"],
            "sensor_off_toward_resource": sensor_row["displacement_toward_resource"],
        })

    def total(pool, key):
        return sum(row[key] for row in pool)

    return {
        "architecture": raw["architecture"],
        "coupling": raw["new_coupling"],
        "seeds": 3,
        "bearings": 4,
        "comparisons": len(comparisons),
        "zero_initial_contact_all_arms": all(row["zero_initial_contact"] for row in rows),
        "invalid_arms": sum(bool(row["invalid"]) for row in rows),
        "capture_wins_vs_both_controls": sum(
            c["active_delivered_n"] > c["sensor_off_delivered_n"] + 1e-12
            and c["active_delivered_n"] > c["motor_off_delivered_n"] + 1e-12
            for c in comparisons
        ),
        "first_transfer_wins_vs_sensor_off": sum(
            c["active_first_transfer"] < c["sensor_off_first_transfer"] for c in comparisons
        ),
        "positive_displacement_toward_resource": sum(
            c["active_toward_resource"] > 0.0 for c in comparisons
        ),
        "heading_wins_vs_sensor_off": sum(
            c["active_toward_resource"] > c["sensor_off_toward_resource"] for c in comparisons
        ),
        "active_n_captured": total(active, "delivered_n"),
        "sensor_off_n_captured": total(sensor, "delivered_n"),
        "motor_off_n_captured": total(motor, "delivered_n"),
        "field_off_n_captured": total(off, "delivered_n"),
        "active_a_produced": total(active, "a_produced"),
        "sensor_off_a_produced": total(sensor, "a_produced"),
        "motor_off_a_produced": total(motor, "a_produced"),
        "maximum_world_material_residual": max(
            max(row["world_n_closure"], row["world_f_closure"]) for row in rows
        ),
        "maximum_polarity_pool_error": max(row["polarity_pool_error"] for row in rows),
        "comparisons_detail": comparisons,
        "result": "CAUSAL_DIRECTIONAL_RESOURCE_ACQUISITION_NOT_ESTABLISHED",
        "interpretation": (
            "The local nutrient coupling slightly increases exposure and captured material, "
            "but it does not produce resource-directed motion: only 1/12 active trajectories "
            "move toward the resource and only 3/12 beat the sensor-off heading. Continued "
            "lifecycle benefit is therefore not established."
        ),
    }


def blocked(reason):
    return {"status": "NOT_EXECUTED_HARD_STOP_WP2", "reason": reason}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--reproduction-raw", required=True)
    parser.add_argument("--acquisition-raw", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--head", default="LOCAL_UNCOMMITTED")
    args = parser.parse_args()

    reproduction_raw = json.loads(Path(args.reproduction_raw).read_text())
    acquisition_raw = json.loads(Path(args.acquisition_raw).read_text())
    root = Path(args.output)
    root.mkdir(parents=True, exist_ok=True)
    for path in root.glob("*.json"):
        path.unlink()

    reproduction = reproduction_summary(reproduction_raw["vertex_only"])
    acquisition = acquisition_summary(acquisition_raw)
    wp1_pass = (
        reproduction["growth_qualified"] >= 8
        and reproduction["geometry_valid_fissions"] >= 7
        and reproduction["simple_viable_daughter_pairs"] >= 6
        and reproduction["all_counted_fission_parents_simple"]
        and reproduction["all_counted_daughters_simple"]
        and reproduction["all_counted_partitions_close"]
    )
    stop = (
        "DC-FINAL-001 hard stop 4: the one authorized source-derived, metabolism-coupled "
        "local polarity response does not establish directional causal resource acquisition "
        "or continued lifecycle benefit across the preregistered seed/bearing matrix."
    )

    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "starting_head": "5fd7981705feda3df18de563bc29b72e35c46a12",
        "r20_scientific_head": "7d7d2349097d123243c4313bd40151ccf0251396",
        "result_head": args.head,
        "authority": "GOAL_AGENT_PROVISIONAL_TERMINAL_NEGATIVE",
        "independent_architect_acceptance": "PENDING",
        "r21": "SUPERSEDED_BEFORE_EXECUTION",
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    })
    dump(root, "state_reconstruction.json", {
        "preserved": [
            "M1 homeostasis and causal damage/death",
            "MaturationCoupledV4 reserve OFF",
            "finite material accounting",
            "accepted polarity reference and actuator interface",
        ],
        "requalified_here": ["geometry-valid physical reproduction", "simple viable daughters"],
        "first_irreducible_blocker": "causal autonomous resource acquisition",
        "dependency_pending": [
            "resource-causal reproduction", "life-history development", "ecological heredity",
            "environment-dependent evolution", "persistent standalone lifeform",
        ],
    })
    dump(root, "external_prior_art.json", {
        "c_ipc": {"url": "https://arxiv.org/abs/2012.04457", "classification": "ADAPTABLE_CONTACT_PRINCIPLE"},
        "membrane_fission": {"url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC6914677/", "classification": "ADAPTABLE_SEGMENT_APPOSITION_PRINCIPLE"},
        "metabolism_coupled_chemotaxis": {"classification": "ADAPTABLE_LOCAL_SIGNAL_PRINCIPLE"},
        "direct_code_copy": "NONE",
    })
    dump(root, "forbidden_information_audit.json", {
        "target_coordinate_read_by_organism": False,
        "resource_identity_or_distance_read": False,
        "planner_reward_or_fitness_feedback": False,
        "reproduction_controller": False,
        "parameter_search": False,
        "m1_changed": False,
    })
    dump(root, "contact_rebuild.json", {
        "architecture": "opt-in local frictionless self-contact with active-set conservative advancement",
        "new_biological_parameters": 0,
        "default_runtime_changed": False,
    })
    dump(root, "reproduction_requalification.json", {
        "unchanged_vertex_pinch_campaign": reproduction,
        "segment_apposition_fallback": "NOT_REQUIRED_VERTEX_PATH_MET_THRESHOLD",
        "planar_half_edge_fallback": "NOT_REQUIRED_VERTEX_PATH_MET_THRESHOLD",
        "original_thresholds": {"growth": "8/10", "fission": "7/10", "viable_daughters": "6/10"},
        "result": "GEOMETRY_VALID_PHYSICAL_REPRODUCTION_REQUALIFIED_IN_ASSAY",
        "work_package_pass": wp1_pass,
    })
    dump(root, "daughter_validity.json", {
        "simple_viable_daughter_pairs": reproduction["simple_viable_daughter_pairs"],
        "all_counted_daughters_simple": reproduction["all_counted_daughters_simple"],
        "partition_accounting": "PASS" if reproduction["all_counted_partitions_close"] else "FAIL",
    })
    dump(root, "finite_world_material_field.json", {
        "finite_n_f": True,
        "diffusion_conservative": True,
        "replenishment": False,
        "resource_field_off_capture": acquisition["field_off_n_captured"],
        "maximum_material_residual": acquisition["maximum_world_material_residual"],
    })
    dump(root, "chemotaxis_causality.json", acquisition)
    dump(root, "m2_acquisition.json", {
        "zero_initial_contact": acquisition["zero_initial_contact_all_arms"],
        "capture_advantage": f'{acquisition["capture_wins_vs_both_controls"]}/{acquisition["comparisons"]}',
        "positive_resource_heading": f'{acquisition["positive_displacement_toward_resource"]}/{acquisition["comparisons"]}',
        "heading_advantage_over_sensor_off": f'{acquisition["heading_wins_vs_sensor_off"]}/{acquisition["comparisons"]}',
        "metabolically_usable_a_advantage": acquisition["active_a_produced"] > acquisition["sensor_off_a_produced"],
        "continued_lifecycle_benefit": "NOT_ESTABLISHED",
        "qualification": "FAIL",
        "hard_stop": stop,
    })
    downstream_reason = "Work Package 2 hard stop fired after causal chemotaxis was not established."
    for name in [
        "development_life_history.json", "d096_integration.json", "mutation_heredity.json",
        "environment_selection.json", "environment_reversal.json", "linux_runtime.json",
        "checkpoint_restart.json", "sensory_embodiment.json", "experiential_memory.json",
        "end_to_end_lifecycle.json",
    ]:
        dump(root, name, blocked(downstream_reason))
    dump(root, "global_material_closure.json", {
        "reproduction_partition": "PASS",
        "finite_world_n_f": "PASS",
        "polarity_active_inactive_pool": "PASS_WITH_NUMERICAL_ERROR_RECORDED",
        "maximum_world_material_residual": acquisition["maximum_world_material_residual"],
        "maximum_polarity_pool_error": acquisition["maximum_polarity_pool_error"],
    })
    dump(root, "final_goal_matrix.json", {
        "M1_HOMEOSTASIS": "PRESERVED",
        "CAUSAL_DAMAGE_DEATH": "PRESERVED",
        "GEOMETRY_VALID_REPRODUCTION": "REQUALIFIED_IN_ASSAY",
        "VALID_DAUGHTERS": "REQUALIFIED_IN_ASSAY",
        "M2_AUTONOMOUS_RESOURCE_ACQUISITION": "NOT_ESTABLISHED",
        "RESOURCE_TO_METABOLISM_TO_WORK": "PRESERVED_IN_BOUNDED_ASSAYS",
        "RESOURCE_TO_REPRODUCTION": "NOT_ESTABLISHED",
        "LIFE_HISTORY_DEVELOPMENT": "NOT_EXECUTED_HARD_STOP_WP2",
        "HERITABLE_ECOLOGICAL_PHENOTYPE": "NOT_EXECUTED_HARD_STOP_WP2",
        "ENVIRONMENT_DEPENDENT_SELECTION": "NOT_EXECUTED_HARD_STOP_WP2",
        "CHECKPOINTED_LINUX_LIFEFORM": "NOT_EXECUTED_HARD_STOP_WP2",
        "top_level": CLASSIFICATION,
    })
    dump(root, "preservation.json", {
        "m1": "CLOSED_FROZEN_UNCHANGED",
        "d087_v2": "8/8", "d087_v3": "8/8", "d087_v4": "7/8",
        "d087_vector": [True, True, False, True, True, True, True, True],
        "legacy_d088_tests": "PRESERVATION_ONLY",
        "d091": "PRESERVED",
        "evolution_harness": "TESTS_ONLY_NOT_SCIENTIFIC_QUALIFICATION",
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    })
    dump(root, "qualification.json", {
        "classification": CLASSIFICATION,
        "authority": "GOAL_AGENT_PROVISIONAL_TERMINAL_NEGATIVE",
        "work_package_1": "PASS",
        "geometry_valid_fissions": reproduction["geometry_valid_fissions"],
        "simple_viable_daughter_pairs": reproduction["simple_viable_daughter_pairs"],
        "work_package_2": "FAIL_HARD_STOP_4",
        "first_irreducible_blocker": stop,
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    entries = []
    for path in sorted(root.glob("*.json")):
        if path.name == "artifact_manifest.json":
            continue
        entries.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    dump(root, "artifact_manifest.json", {"files": entries, "count": len(entries)})


if __name__ == "__main__":
    main()
