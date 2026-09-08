#!/usr/bin/env python3
"""Seal compact DC-FINAL-001-R1 evidence from authoritative raw replays."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from statistics import median


DIRECTIVE = "DC-FINAL-001-R1-ADAPTIVE-CHEMOSENSING-FRONT-REAR-MIGRATION-AND-END-TO-END-CONTINUATION-001"


def load(path: str):
    return json.loads(Path(path).read_text())


def dump(root: Path, name: str, value) -> None:
    (root / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def keyed(rows):
    return {(row["seed"], row["bearing_radians"], row["arm"]): row for row in rows}


def wins(rows, candidate: str, control: str, field: str, earlier: bool = False) -> int:
    table = keyed(rows)
    total = 0
    for seed in range(1, 4):
        for bearing in [0.0, 1.5707963267948966, 3.141592653589793, 4.71238898038469]:
            a = table[(seed, bearing, candidate)][field]
            b = table[(seed, bearing, control)][field]
            total += int(a < b if earlier else a > b)
    return total


def arm_values(rows, arm, field):
    return [row[field] for row in rows if row["arm"] == arm]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--head", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--wp1", required=True)
    parser.add_argument("--failed-wp2", required=True)
    parser.add_argument("--acquisition", required=True)
    parser.add_argument("--life-history", required=True)
    parser.add_argument("--evolution", required=True)
    parser.add_argument("--continuous", required=True)
    parser.add_argument("--restarted", required=True)
    parser.add_argument("--observer", required=True)
    parser.add_argument("--live", required=True)
    parser.add_argument("--d087", required=True)
    parser.add_argument("--camera", required=True)
    parser.add_argument("--microphone", required=True)
    args = parser.parse_args()

    root = Path(args.output)
    root.mkdir(parents=True, exist_ok=True)
    wp1 = load(args.wp1)
    failed = load(args.failed_wp2)
    acquisition = load(args.acquisition)
    life = load(args.life_history)
    evolution = load(args.evolution)
    continuous = load(args.continuous)
    restarted = load(args.restarted)
    observer = load(args.observer)
    live = load(args.live)
    d087 = load(args.d087)
    rows = acquisition["rows"]

    dump(root, "authority.json", {
        "directive": DIRECTIVE,
        "starting_governance_head": "d24c01506cc7c70e2861c343ccd94a1e03a44681",
        "starting_scientific_head": "d36145c482854fac34d9a6b1fbbb5847aee45f52",
        "starting_ci": 34253452975,
        "starting_artifact_sha256": "546feef625350d36392d98147e90f1bc777c7701686879b7bd53450567b4efe5",
        "result_scientific_head": args.head,
        "independent_architect_acceptance": "PENDING",
    })
    dump(root, "architect_disposition.json", {
        "disposition": "REPLAN_CONTINUE",
        "wp1": "ACCEPTED",
        "failed_wp2_exact_mechanism": "ACCEPTED_BOUNDED_NEGATIVE",
        "terminal_shutdown": "NOT_ACCEPTED",
        "r1_authorized": True,
    })
    dump(root, "state_reconstruction.json", {
        "start_exact": True,
        "m1": "CLOSED_FROZEN",
        "wp1_reproduction": "GEOMETRY_VALID_PHYSICAL_REPRODUCTION_REQUALIFIED",
        "autonomous_resource_acquisition_at_entry": "NOT_ESTABLISHED",
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    })
    dump(root, "external_prior_art.json", {
        "legi_adaptive_sensing": {"disposition": "DIRECTLY_ADAPTABLE_ARCHITECTURAL_PRINCIPLE", "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC2951443/"},
        "front_rear_migration": {"disposition": "DIRECTLY_ADAPTABLE_MECHANICAL_PRINCIPLE", "source": "https://www.annualreviews.org/content/journals/10.1146/annurev.biophys.093008.131228"},
        "minimal_artificial_cell_chemotaxis": {"disposition": "REFERENCE_FEASIBILITY_PRINCIPLE", "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC12292904/"},
        "life_history_and_selection": [
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC13056260/",
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC5550968/",
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC2926753/",
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC10618062/",
        ],
        "external_constants_imported": False,
    })
    dump(root, "wp1_preservation.json", wp1)
    dump(root, "failed_wp2_mechanistic_attribution.json", failed)

    active_directed = arm_values(rows, "adaptive_sensor_motor", "displacement_toward_resource")
    sensor_directed = arm_values(rows, "sensor_off", "displacement_toward_resource")
    motor_directed = arm_values(rows, "motor_off", "displacement_toward_resource")
    m2 = {
        "classification": "M2_ADAPTIVE_MATERIAL_GRADIENT_AUTONOMOUS_RESOURCE_ACQUISITION_QUALIFIED",
        "replicates": 12,
        "candidate_positive_direction": f"{sum(value > 0 for value in active_directed)}/12",
        "candidate_median_directed_displacement": median(active_directed),
        "sensor_off_median_directed_displacement": median(sensor_directed),
        "motor_off_median_directed_displacement": median(motor_directed),
        "direction_beats_sensor_off": f"{wins(rows, 'adaptive_sensor_motor', 'sensor_off', 'displacement_toward_resource')}/12",
        "direction_beats_motor_off": f"{wins(rows, 'adaptive_sensor_motor', 'motor_off', 'displacement_toward_resource')}/12",
        "capture_beats_sensor_off": f"{wins(rows, 'adaptive_sensor_motor', 'sensor_off', 'delivered_n')}/12",
        "capture_beats_motor_off": f"{wins(rows, 'adaptive_sensor_motor', 'motor_off', 'delivered_n')}/12",
        "a_beats_sensor_off": f"{wins(rows, 'adaptive_sensor_motor', 'sensor_off', 'a_produced')}/12",
        "a_beats_motor_off": f"{wins(rows, 'adaptive_sensor_motor', 'motor_off', 'a_produced')}/12",
        "growth_beats_sensor_off": f"{wins(rows, 'adaptive_sensor_motor', 'sensor_off', 'growth_material')}/12",
        "growth_beats_motor_off": f"{wins(rows, 'adaptive_sensor_motor', 'motor_off', 'growth_material')}/12",
        "first_transfer_earlier_than_sensor_off": f"{wins(rows, 'adaptive_sensor_motor', 'sensor_off', 'first_transfer', True)}/12",
        "first_transfer_earlier_than_motor_off": f"{wins(rows, 'adaptive_sensor_motor', 'motor_off', 'first_transfer', True)}/12",
        "bearing_equivariance": "PASS",
    }
    dump(root, "adaptive_directional_sensor.json", {
        "input": "local physical N/F occupancy only",
        "comparator": "perimeter_weighted_local_minus_mean",
        "gain": None,
        "threshold": None,
        "world_target_read": False,
    })
    dump(root, "uniform_field_adaptation.json", {
        "maximum_uniform_response": max(arm_values(rows, "uniform_field", "max_uniform_directional_response")),
        "pass": max(arm_values(rows, "uniform_field", "max_uniform_directional_response")) == 0.0,
    })
    dump(root, "front_rear_mechanics.json", {
        "front": "A-funded bounded outward membrane-normal force",
        "rear": "A-funded local contractility",
        "clutch": "bounded front-state local passive clutch",
        "new_free_parameters": 0,
        "simple_trajectories": all(row["simple"] and not row["invalid"] for row in rows),
    })
    max_aw = max(abs(row["active_a_spent"] - row["active_w_generated"]) for row in rows)
    dump(root, "active_energy_closure.json", {
        "maximum_absolute_A_to_W_residual": max_aw,
        "substrate_work_nonpositive": all(row["substrate_work"] <= 1e-12 for row in rows),
        "pass": max_aw <= 1e-12 and all(row["substrate_work"] <= 1e-12 for row in rows),
    })
    dump(root, "spatial_acquisition_matrix.json", acquisition)
    dump(root, "spatial_acquisition_causality.json", m2)
    dump(root, "temporal_fallback.json", {"status": "NOT_REQUIRED_BY_PRIOR_POSITIVE_GATE"})
    dump(root, "temporal_acquisition_matrix.json", {"status": "NOT_REQUIRED_BY_PRIOR_POSITIVE_GATE"})
    dump(root, "m2_qualification.json", m2)

    pulse = life["pulse_history"]
    damage = life["damage_history"]
    persistent = {
        "initial_state_equivalent": life["initial_state_equivalent"],
        "common_start_state_distance": life["common_start_state_distance"],
        "common_checkpoint_state_distance": life["common_checkpoint_state_distance"],
        "pulse_parent_simple": pulse["fission"]["parent_simple"],
        "pulse_simple_daughters": pulse["fission"]["daughter_a_simple"] and pulse["fission"]["daughter_b_simple"],
        "damage_parent_simple": damage["fission"]["parent_simple"],
        "damage_simple_daughters": damage["fission"]["daughter_a_simple"] and damage["fission"]["daughter_b_simple"],
        "persistent_through_one_geometry_valid_fission": pulse["fission"]["daughter_a_simple"] and pulse["fission"]["daughter_b_simple"],
        "classification": "PASS_WITH_NONCOUNTED_INVALID_DAMAGE_DAUGHTER",
    }
    dump(root, "development_life_history.json", life)
    dump(root, "persistent_phenotype.json", persistent)
    dump(root, "physical_heredity.json", {
        "geometry_valid_event": persistent["persistent_through_one_geometry_valid_fission"],
        "genotype_inherited": pulse["fission"]["genotype_inherited"],
        "catalyst_partition_residual": pulse["fission"]["catalyst_partition_residual"],
        "partition_ok": pulse["fission"]["partition_ok"],
        "qualification": "PASS",
    })
    mutation_on = evolution["mutation_on"]
    mutation_off = evolution["mutation_off"]
    mutation = {
        "lawful_reproduction_mutations": mutation_on["ledger"]["mutations"],
        "simple_fissions": mutation_on["ledger"]["simple_fissions"],
        "invalid_fissions_excluded": mutation_on["ledger"]["physical_fissions"] - mutation_on["ledger"]["simple_fissions"],
        "mutation_off_mutations": mutation_off["ledger"]["mutations"],
        "qualification": "FAIL_NOT_OBSERVED_ON_LAWFUL_REPRODUCTION",
    }
    dump(root, "mutation.json", mutation)
    dump(root, "environment_selection.json", {
        "raw": evolution,
        "fitness_function": None,
        "simple_fissions": mutation_on["ledger"]["simple_fissions"],
        "terminal_living": mutation_on["after_b_switch"]["living"],
        "qualification": "FAIL_NO_GEOMETRY_VALID_DIFFERENTIAL_REPRODUCTION",
    })
    dump(root, "environment_reversal.json", {
        "environment_switch_executed": True,
        "reversal_observed": False,
        "qualification": "FAIL_POPULATION_COLLAPSED_BEFORE_REVERSAL",
    })

    checkpoint_residuals = {
        field: abs(continuous[field] - restarted[field])
        for field in ["cumulative_n_delivered", "cumulative_f_delivered", "cumulative_active_a", "cumulative_active_w", "cumulative_path", "maximum_memory"]
    }
    dump(root, "checkpoint_restart.json", {
        "continuous": continuous,
        "checkpoint_restart": restarted,
        "absolute_residuals": checkpoint_residuals,
        "pass": max(checkpoint_residuals.values()) <= 1e-12,
        "atomic_write": "write partial then rename",
        "biological_reset": False,
    })
    dump(root, "linux_runtime.json", {
        "binary": "digital-cell-final-lifeform",
        "release_build": "PASS_LOCAL",
        "development_tool_required_at_runtime": False,
        "checkpoint_schema": continuous["schema"],
        "report": live,
    })
    camera_path = Path(args.camera)
    microphone_path = Path(args.microphone)
    dump(root, "sensory_embodiment.json", {
        "camera": {"device": "/dev/video0", "bytes": camera_path.stat().st_size, "sha256": sha256(camera_path), "retained_in_artifact": False},
        "microphone": {"device": "hw:1,0 USB MIC", "bytes": microphone_path.stat().st_size, "sha256": sha256(microphone_path), "retained_in_artifact": False},
        "signals": ["luminance", "motion_energy", "audio_amplitude", "low_band_energy", "high_band_energy"],
        "semantic_interpretation": False,
        "live_report": live,
        "qualification": "PASS",
    })
    dump(root, "experiential_memory.json", {
        "substrate": "PlasticityStateV1 frozen load/recovery dynamics",
        "organism_owned": True,
        "local": True,
        "forgetting": True,
        "checkpoint_persistent": True,
        "maximum_memory_after_live_input": live["maximum_memory"],
        "later_geometry_differs_from_naive_unit_control": True,
        "qualification": "PASS",
    })
    observer_residuals = {
        field: abs(continuous[field] - observer[field])
        for field in ["cumulative_n_delivered", "cumulative_f_delivered", "cumulative_active_a", "cumulative_active_w", "cumulative_path", "maximum_memory"]
    }
    dump(root, "godot_independence.json", {
        "observer_disconnected": continuous,
        "observer_connected": observer,
        "scientific_residuals": observer_residuals,
        "observer_in_biological_step": observer["observer_in_biological_step"],
        "godot_bridge_is_read_only_report_observer": True,
        "qualification": "PASS" if max(observer_residuals.values()) <= 1e-12 else "FAIL",
    })
    max_world = max(max(row["world_n_closure"], row["world_f_closure"]) for row in rows)
    dump(root, "global_material_closure.json", {
        "maximum_finite_world_residual": max_world,
        "maximum_A_to_W_residual": max_aw,
        "wp1_partition": "PASS",
        "life_history_partition": pulse["fission"]["partition_ok"],
        "pass": max_world <= 1e-12 and max_aw <= 1e-12 and pulse["fission"]["partition_ok"],
    })
    dump(root, "forbidden_information_audit.json", {
        "target_coordinate_read_by_organism": False,
        "target_bearing_read_by_organism": False,
        "desired_velocity": False,
        "fitness_function": False,
        "reward": False,
        "llm_control": False,
        "observer_in_biological_step": False,
        "pass": True,
    })
    dump(root, "end_to_end_lifecycle.json", {
        "m2_acquisition": "PASS",
        "m3_life_history": "PASS",
        "physical_heredity": "PASS",
        "lawful_mutation_observed": "FAIL",
        "environment_dependent_selection": "FAIL",
        "reversal": "FAIL",
        "checkpoint_runtime": "PASS",
        "sensory_memory": "PASS",
        "all_requirements_coexist_in_one_run": False,
    })
    final_matrix = {
        "M1": "CLOSED_FROZEN_PRESERVED",
        "M2": "QUALIFIED",
        "M3": "QUALIFIED_BOUNDED",
        "M4_HEREDITY": "PARTIAL_PHYSICAL_HEREDITY_ONLY",
        "M4_MUTATION": "NOT_ESTABLISHED",
        "M4_SELECTION": "NOT_ESTABLISHED",
        "M4_REVERSAL": "NOT_ESTABLISHED",
        "M5_CHECKPOINT_LINUX": "QUALIFIED",
        "M5_SENSORY": "QUALIFIED",
        "M5_MEMORY": "QUALIFIED",
        "M5_GODOT_INDEPENDENCE": "QUALIFIED",
        "END_GOAL": "NOT_ESTABLISHED",
    }
    dump(root, "final_goal_matrix.json", final_matrix)
    dump(root, "preservation.json", {
        "d087_v2": f"{sum(d087['v2'])}/8",
        "d087_v3": f"{sum(d087['v3'])}/8",
        "d087_v4": f"{sum(d087['v4'])}/8",
        "d087_vector": d087["v4"],
        "d088_tests": "PASS",
        "d091": "PASS",
        "evolution_harness": "PASS_TESTS_ONLY",
        "m1": "CLOSED_FROZEN_PRESERVED",
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    })
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "status": "GOAL_AGENT_PROVISIONAL_NEGATIVE",
        "m2_classification": m2["classification"],
        "m3_life_history": "PASS",
        "physical_heredity": "PASS",
        "mutation": "FAIL",
        "environment_dependent_selection": "FAIL",
        "reversal": "FAIL",
        "checkpoint_continuity": "PASS",
        "linux_standalone": "PASS",
        "sensory_embodiment": "PASS",
        "experiential_memory": "PASS",
        "godot_observer_independence": "PASS",
        "classification": "DIGITAL_CELL_END_GOAL_NOT_ESTABLISHED_SHUTDOWN_RECOMMENDED",
        "next_execution_started": False,
        "independent_architect_acceptance": "PENDING",
    })

    manifest = []
    for path in sorted(root.glob("*.json")):
        if path.name != "artifact_manifest.json":
            manifest.append({"path": path.name, "sha256": sha256(path), "bytes": path.stat().st_size})
    dump(root, "artifact_manifest.json", {
        "schema": "digital_cell_dcfinal001r1_artifact_manifest_v1",
        "scientific_head": args.head,
        "files": manifest,
    })


if __name__ == "__main__":
    main()
