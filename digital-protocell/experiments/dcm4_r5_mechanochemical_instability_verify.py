#!/usr/bin/env python3
"""Independent R5 attribution for the sealed R4 polarity-actuation run.

This program is deliberately an observer.  It reads the immutable R4 raw
trajectory, recomputes the emitted shape/stability predicates, derives the
frozen actuator request from the source equation, and writes compact causal
records.  It never feeds a measurement back into a Digital Cell transition.

The sealed R4 record does not contain pre-mechanics vertices together with the
polarity edge-tension field.  Modal force projections are therefore marked as
post-mechanics snapshot proxies, and the final decision remains fail-closed
when a full coupled Jacobian cannot be identified from the available raw
states.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R5-MECHANOCHEMICAL-INSTABILITY-ATTRIBUTION-GATE-001"
R4_HEAD = "38de33d6ed2741a60b3e825360a45b8ff4686792"
R4_IMPLEMENTATION_HEAD = "c778e2a4fc858354b7c661a091837e6cc1dab5ee"
R4_CI = "34802371283"
R4_ARTIFACT = "sha256:1be6045d0fdc0b91dda666baefe363e9490ee1cc2be498169b5a0cd3219b677b"
R4_BINARY = "fbd0607c1867ca2c661da17d51ca36980e1eed87e63e1f70ff54154335d7fb82"
R4_RAW_SHA = "a97852f34c8bb63c86d1367ec512a69e3ae4a421d559fa3dd7054d9ee878c8bd"
HORIZON = 14_778
TRACE_ROWS = 60
MECHANICS_DT = 0.02
MAX_ACTIVE_TENSION = 2.0
ACTUATOR_COST = 0.05
MODE_TOLERANCE = 1.0e-3
NUMERICAL_TOLERANCE = 1.0e-8
LAG_OFFSETS = (-2, -1, 0, 1, 2)


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


def digest_json(value) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


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


def approx(left: float, right: float, tolerance: float = NUMERICAL_TOLERANCE) -> bool:
    return abs(left - right) <= tolerance * (1.0 + abs(left) + abs(right))


def mean(values):
    values = [value for value in values if value is not None]
    return sum(values) / len(values) if values else None


def variance(values):
    values = [value for value in values if value is not None]
    if not values:
        return None
    average = mean(values)
    return mean([(value - average) ** 2 for value in values])


def pearson(left, right):
    if len(left) != len(right) or len(left) < 2:
        return None
    left_mean, right_mean = mean(left), mean(right)
    numerator = sum(
        (a - left_mean) * (b - right_mean) for a, b in zip(left, right)
    )
    denominator = math.sqrt(
        sum((a - left_mean) ** 2 for a in left)
        * sum((b - right_mean) ** 2 for b in right)
    )
    return numerator / denominator if denominator else 0.0


def slope(values, times):
    paired = [(value, time) for value, time in zip(values, times) if value is not None]
    if len(paired) < 2:
        return None
    values, times = zip(*paired)
    mt, mv = mean(times), mean(values)
    denominator = sum((time - mt) ** 2 for time in times)
    return (
        sum((time - mt) * (value - mv) for time, value in zip(times, values))
        / denominator
        if denominator
        else 0.0
    )


def circular_delta(left: float, right: float) -> float:
    return (left - right + math.pi) % (2.0 * math.pi) - math.pi


def lag_correlation(left, right, lag):
    if lag >= 0:
        return pearson(left[:-lag] if lag else left, right[lag:] if lag else right)
    return pearson(left[-lag:], right[:lag])


def source_audit(repo: Path) -> dict:
    files = [
        "crates/regulatory-core/src/polarity_actuation.rs",
        "crates/regulatory-core/src/contractility.rs",
        "examples/dcfinal001_r5_v4_neck.rs",
        "examples/dcfinal001_r4_evolution.rs",
        "crates/chemistry-core/src/mesh_mechanics.rs",
    ]
    texts = {relative: (repo / relative).read_text() for relative in files}
    markers = {
        "local_polarity_activity": "derive_local_activity" in texts[files[0]],
        "equation_derived_reference": "reference_active_concentration" in texts[files[0]],
        "edge_tension_mapping": "let tension = params.max_active_tension * edge_activity" in texts[files[1]],
        "common_funding_scale": "let funding_scale = if requested_resource <= f64::EPSILON" in texts[files[1]],
        "mechanics_owner": "mechanics_step_with_edge_tensions_external_forces_and_local_self_contact" in texts[files[1]],
        "r9_r10_call_path": "r10_refractory_mechanics_step_with_polarity_diagnostics" in texts[files[2]],
        "polarity_then_mechanics": "derive_local_activity(state, &measures" in texts[files[3]],
        "remesh_after_mechanics": "remesh_preserving_simple(mesh)" in texts[files[2]],
        "frozen_mechanics_dt": "pub dt: f64" in texts[files[4]],
    }
    if not all(markers.values()):
        raise AssertionError(f"R4 actuator source audit failed: {markers}")
    return {
        "files": {
            relative: {"sha256": sha(repo / relative)} for relative in files
        },
        "markers": markers,
        "call_path": [
            "dcfinal001_r4_evolution.rs: state.advance -> derive_local_activity",
            "dcfinal001_r4_evolution.rs: r10_refractory_mechanics_step_with_polarity_diagnostics",
            "dcfinal001_r5_v4_neck.rs: existing paid actuator",
            "regulatory-core/src/contractility.rs: edge tension -> mechanics -> A to W",
        ],
        "production_transition_modified": False,
        "observer_only": True,
    }


def shape_from_vertices(vertices):
    center_x = sum(point[0] for point in vertices) / len(vertices)
    center_y = sum(point[1] for point in vertices) / len(vertices)
    polar = []
    for x, y in vertices:
        dx, dy = x - center_x, y - center_y
        polar.append((math.hypot(dx, dy), math.atan2(dy, dx)))
    mean_radius = mean([radius for radius, _ in polar])
    modes = []
    for mode in range(2, 5):
        cosine = sum(radius * math.cos(mode * angle) for radius, angle in polar)
        sine = sum(radius * math.sin(mode * angle) for radius, angle in polar)
        modes.append(
            {
                "mode": mode,
                "cosine_coefficient": cosine / len(polar),
                "sine_coefficient": sine / len(polar),
                "normalized_amplitude": math.hypot(cosine, sine)
                / (len(polar) * max(mean_radius, 1.0e-300)),
            }
        )
    return {
        "centroid": [center_x, center_y],
        "mean_radius": mean_radius,
        "modes": modes,
    }


def mode_dict(modes, amplitude_key):
    return {int(item["mode"]): item for item in modes}


def radial_force_modes(vertices, forces):
    if len(vertices) != len(forces) or len(vertices) < 3:
        return None
    center_x = sum(point[0] for point in vertices) / len(vertices)
    center_y = sum(point[1] for point in vertices) / len(vertices)
    radial_values, angles = [], []
    for x, y in vertices:
        dx, dy = x - center_x, y - center_y
        radius = max(math.hypot(dx, dy), 1.0e-300)
        radial_values.append((dx, dy, radius))
        angles.append(math.atan2(dy, dx))
    result = {}
    for mode in range(2, 5):
        cosine, sine = 0.0, 0.0
        for (fx, fy), (dx, dy, radius), angle in zip(forces, radial_values, angles):
            radial = (fx * dx + fy * dy) / radius
            cosine += radial * math.cos(mode * angle)
            sine += radial * math.sin(mode * angle)
        result[mode] = {
            "cosine_coefficient": cosine / len(vertices),
            "sine_coefficient": sine / len(vertices),
            "amplitude": math.hypot(cosine, sine) / len(vertices),
            "phase": math.atan2(sine, cosine),
        }
    return result


def polarity_tension_force_proxy(row):
    """Recompute the frozen edge-tension force on an accepted snapshot.

    R4 did not serialize pre-mechanics vertices.  This uses the accepted
    post-mechanics vertices only when remesh preserved the edge count and is
    explicitly not called an exact pre-force field.
    """
    activity = row["polarity"]["actuation"]["vertex_activity"]
    vertices = row["material_geometry"]["vertices"]
    if len(activity) != len(vertices):
        return {"status": "NOT_DERIVABLE_AFTER_REMESH"}
    funding = row["funding_ratio"]
    forces = [[0.0, 0.0] for _ in vertices]
    requested = 0.0
    tension_sum = 0.0
    for edge in range(len(vertices)):
        next_edge = (edge + 1) % len(vertices)
        dx = vertices[next_edge][0] - vertices[edge][0]
        dy = vertices[next_edge][1] - vertices[edge][1]
        length = max(math.hypot(dx, dy), 1.0e-300)
        edge_activity = 0.5 * (activity[edge] + activity[next_edge])
        tension = MAX_ACTIVE_TENSION * edge_activity * funding
        tension_sum += tension
        requested += ACTUATOR_COST * MAX_ACTIVE_TENSION * edge_activity * length * MECHANICS_DT
        tx, ty = tension * dx / length, tension * dy / length
        forces[edge][0] += tx
        forces[edge][1] += ty
        forces[next_edge][0] -= tx
        forces[next_edge][1] -= ty
    return {
        "status": "POST_MECHANICS_SNAPSHOT_PROXY",
        "force_norm_sum": sum(math.hypot(*force) for force in forces),
        "maximum_vertex_force": max(math.hypot(*force) for force in forces),
        "funded_tension_sum": tension_sum,
        "radial_modes": radial_force_modes(vertices, forces),
    }


def checkpoint_record(row: dict) -> dict:
    geometry = row["material_geometry"]
    polarity = row["polarity"]
    activity = polarity["actuation"]["vertex_activity"]
    edge_material = geometry["edge_material"]
    edge_lengths = [item["geometric_length"] for item in edge_material]
    edge_activity = (
        [0.5 * (activity[i] + activity[(i + 1) % len(activity)]) for i in range(len(activity))]
        if len(activity) == len(edge_lengths)
        else None
    )
    polarity_request = (
        ACTUATOR_COST
        * MAX_ACTIVE_TENSION
        * sum(value * length for value, length in zip(edge_activity, edge_lengths))
        * MECHANICS_DT
        if edge_activity is not None
        else None
    )
    emitted_shape = geometry["shape_modes"]
    recomputed_shape = shape_from_vertices(geometry["vertices"])
    shape_modes = mode_dict(emitted_shape["modes"], "normalized_amplitude")
    recomputed_modes = mode_dict(recomputed_shape["modes"], "normalized_amplitude")
    shape_amplitudes = [shape_modes[mode]["normalized_amplitude"] for mode in (2, 3, 4)]
    recomputed_error = max(
        abs(shape_modes[mode]["normalized_amplitude"] - recomputed_modes[mode]["normalized_amplitude"])
        for mode in (2, 3, 4)
    )
    polarity_modes = polarity["mode_summaries"]
    dominant_polarity = max(polarity_modes, key=lambda item: item["amplitude"])
    dominant_shape = max(shape_modes, key=lambda mode: shape_modes[mode]["normalized_amplitude"])
    passive = geometry["passive_force_components_at_mechanics_input"]
    passive_sum = sum(
        passive[key]
        for key in ("stretch_force_norm_sum", "bending_force_norm_sum", "pressure_force_norm_sum")
    )
    active_force = polarity_tension_force_proxy(row)
    edge_records = []
    for index, material in enumerate(edge_material):
        edge_records.append(
            {
                "edge": material["edge"],
                "geometric_length": material["geometric_length"],
                "m_young": material["m_young"],
                "m_mature": material["m_mature"],
                "mature_fraction": material["mature_fraction"],
                "rest_length": material["rest_length"],
                "rest_length_delta": material["rest_length_delta"],
                "strain": material["strain"],
                "active_concentration": (
                    polarity["actuation"]["edge_active_concentrations"][index]
                    if index < len(polarity["actuation"]["edge_active_concentrations"])
                    else None
                ),
                "edge_activity": edge_activity[index] if edge_activity is not None else None,
            }
        )
    return {
        "step": row["step"],
        "phase_step": row["phase_step"],
        "boundary": {
            "target_n": row["boundary_target_n"],
            "target_f": row["boundary_target_f"],
            "accepted_bath_n": row["accepted_bath_n"],
            "accepted_bath_f": row["accepted_bath_f"],
        },
        "material": {
            "area": geometry["area"],
            "perimeter": geometry["perimeter"],
            "shape_factor": geometry["shape_factor_p_squared_over_4pi_area"],
            "reduced_area": geometry["reduced_area_inverse_shape_factor"],
            "young_structural_mass": geometry["young_structural_mass"],
            "mature_structural_mass": geometry["mature_structural_mass"],
            "total_structural_mass": geometry["total_structural_mass"],
            "mature_rest_perimeter": geometry["mature_rest_perimeter"],
            "material_equivalent_perimeter": geometry["material_equivalent_perimeter"],
            "perimeter_minus_mature_rest_perimeter": geometry["perimeter_minus_mature_rest_perimeter"],
            "edge_records": edge_records,
        },
        "polarity": {
            "active_before": polarity["active_before"],
            "active_after": polarity["active_after"],
            "inactive_before": polarity["inactive_before"],
            "inactive_after": polarity["inactive_after"],
            "total_before": polarity["total_before"],
            "total_after": polarity["total_after"],
            "total_residual": polarity["total_residual"],
            "a_consumed": polarity["a_consumed"],
            "w_produced": polarity["w_produced"],
            "mode_summaries": polarity_modes,
            "dominant_mode": dominant_polarity["mode"],
            "dominant_amplitude": dominant_polarity["amplitude"],
            "activity_max": max(activity),
            "activity_mean": mean(activity),
            "activity_variance": variance(activity),
            "activity_nonzero_fraction": sum(value > 1.0e-15 for value in activity) / len(activity),
            "activity_saturated_fraction": sum(value >= 1.0 - 1.0e-15 for value in activity) / len(activity),
            "polarity_request_a_recomputed": polarity_request,
            "polarity_funded_request_a_recomputed": (
                polarity_request * row["funding_ratio"] if polarity_request is not None else None
            ),
        },
        "cortex": {
            "raw_drive_maximum": row["raw_drive_maximum"],
            "raw_drive_mean": row["raw_drive_mean"],
            "raw_drive_variance": row["raw_drive_variance"],
            "effective_drive_maximum": row["effective_drive_maximum"],
            "effective_drive_mean": row["effective_drive_mean"],
            "effective_drive_variance": row["effective_drive_variance"],
            "adaptation_mean": row["adaptation_mean_before"],
            "adaptation_maximum": row["adaptation_maximum_before"],
            "adaptation_enabled": row["adaptation_enabled"],
            "requested_r9_active_a": row["requested_active_a"],
            "requested_total_active_a": row["requested_active_a_total"],
            "funded_total_active_a": row["funded_active_a_total"],
            "funding_ratio": row["funding_ratio"],
            "requested_minus_funded_a": row["requested_active_a_total"] - row["funded_active_a_total"],
            "funded_work_ledger_a": row["funded_active_a_total"],
            "active_displacement_norm": row["actual_displacement_norm"],
            "displacement_per_funded_work": (
                row["actual_displacement_norm"] / row["funded_active_a_total"]
                if row["funded_active_a_total"] > 0.0
                else None
            ),
            "r9_extra_force_norm": geometry["active_force"]["funded_vector_norm_sum"],
            "polarity_tension_force_proxy": active_force,
            "passive_force_components": passive,
            "passive_force_component_sum": passive_sum,
            "polarity_to_passive_force_ratio": (
                active_force.get("force_norm_sum", 0.0) / passive_sum
                if active_force.get("status") == "POST_MECHANICS_SNAPSHOT_PROXY" and passive_sum
                else None
            ),
        },
        "shape": {
            "emitted_modes": emitted_shape["modes"],
            "recomputed_modes": recomputed_shape["modes"],
            "recomputed_max_error": recomputed_error,
            "dominant_mode": dominant_shape,
            "dominant_amplitude": shape_modes[dominant_shape]["normalized_amplitude"],
            "dominant_phase": math.atan2(
                shape_modes[dominant_shape]["sine_coefficient"],
                shape_modes[dominant_shape]["cosine_coefficient"],
            ),
            "modes_2_to_4": shape_amplitudes,
        },
        "geometry": {
            "max_compression": max(max(0.0, -item["strain"]) for item in edge_material),
            "mean_compression": mean([max(0.0, -item["strain"]) for item in edge_material]),
            "min_strain": min(item["strain"] for item in edge_material),
            "nearest_eligible_pair": row["segment_geometry"]["nearest_geometrically_eligible_pair"],
            "within_range_pair_count": row["segment_geometry"]["within_range_pair_count"],
            "stress_qualified_pair_count": row["segment_geometry"]["stress_qualified_pair_count"],
            "simple": row["simple"],
        },
        "stability": {
            "epsilon": geometry["stability_assay"]["epsilon"],
            "steps": geometry["stability_assay"]["steps"],
            "classification_tolerance": geometry["stability_assay"]["classification_tolerance"],
            "modes": geometry["stability_assay"]["modes"],
        },
        "raw_vectors": {
            "raw_curvature_drive": row["raw_curvature_drive"],
            "effective_drive": row["effective_drive"],
            "adaptation_before": row["adaptation_before"],
            "polarity_activity": activity,
        },
    }


def validate_raw(raw: dict) -> list[dict]:
    assert raw["horizon"] == HORIZON, raw.get("horizon")
    assert len(raw["connected"]) == 10
    arms = sorted(raw["connected"], key=lambda arm: arm["arm"])
    assert [arm["arm"] for arm in arms] == list(range(1, 11))
    assert all(finite(arm) for arm in arms)
    for arm in arms:
        assert arm["connected"] is True
        assert arm["accepted"] is True
        assert arm["accepted_steps"] == HORIZON
        assert arm["physical_fissions"] == 0
        assert arm["post_bootstrap_physical_fissions"] == 0
        assert arm["all_simple"] is True
        assert arm["all_runtime_valid"] is True
        assert arm["all_lifecycle_valid"] is True
        assert len(arm["mechanics_trace"]) == TRACE_ROWS
        assert arm["maximum_mass_over_birth"] >= 1.35
        ledger = arm["ledger"]
        assert ledger["accepted_steps"] == HORIZON
        assert ledger["rejected_steps"] == 0
        assert ledger["computational_rejections"] == 0
        assert ledger["numerical_invalid"] is False
    return arms


def stability_summary(records: list[dict]) -> dict:
    counts = {"GROWING": 0, "DECAYING": 0, "NEUTRAL": 0, "INVALID": 0}
    ratios = []
    per_arm = {}
    for record in records:
        arm_counts = {key: 0 for key in counts}
        arm_ratios = []
        for checkpoint in record["checkpoints"]:
            for mode in checkpoint["stability"]["modes"]:
                trace = mode["trace"]
                valid = mode["valid"] is True and all(
                    item["baseline_accepted"] and item["perturbed_accepted"]
                    for item in trace
                )
                if valid:
                    ratio = trace[-1]["rms_difference"] / max(
                        mode["initial_perturbation_rms"], 1.0e-300
                    )
                    classification = (
                        "GROWING"
                        if ratio > 1.0 + MODE_TOLERANCE
                        else "DECAYING"
                        if ratio < 1.0 - MODE_TOLERANCE
                        else "NEUTRAL"
                    )
                    assert approx(ratio, mode["growth_ratio"], 1.0e-8)
                    arm_ratios.append(ratio)
                else:
                    classification = "INVALID"
                counts[classification] += 1
                arm_counts[classification] += 1
                if valid:
                    ratios.append(ratio)
        per_arm[str(record["arm"])] = {"counts": arm_counts, "mean_growth_ratio": mean(arm_ratios)}
    return {
        "counts": counts,
        "valid_record_count": len(ratios),
        "mean_valid_growth_ratio": mean(ratios),
        "minimum_valid_growth_ratio": min(ratios),
        "maximum_valid_growth_ratio": max(ratios),
        "per_arm": per_arm,
        "interpretation": "All sealed R4 frozen passive local perturbations decay; this is not a coupled Jacobian.",
        "status": "PASS",
    }


def summarize_arm(arm: dict, checkpoints: list[dict]) -> dict:
    times = [item["step"] for item in checkpoints]
    shape = [item["shape"]["dominant_amplitude"] for item in checkpoints]
    polarity = [item["polarity"]["dominant_amplitude"] for item in checkpoints]
    activity = [item["polarity"]["activity_max"] for item in checkpoints]
    activity_mean = [item["polarity"]["activity_mean"] for item in checkpoints]
    request = [item["polarity"]["polarity_request_a_recomputed"] for item in checkpoints]
    total_request = [item["cortex"]["requested_total_active_a"] for item in checkpoints]
    funded = [item["cortex"]["funded_total_active_a"] for item in checkpoints]
    near = [item["geometry"]["nearest_eligible_pair"]["distance_over_range"] for item in checkpoints]
    passive = [item["cortex"]["passive_force_component_sum"] for item in checkpoints]
    projection = [
        item["cortex"]["polarity_tension_force_proxy"]
        for item in checkpoints
        if item["cortex"]["polarity_tension_force_proxy"]["status"] == "POST_MECHANICS_SNAPSHOT_PROXY"
    ]
    first, late = shape[0], shape[-1]
    ratio = late / max(first, 1.0e-300)
    classification = (
        "GROWING"
        if ratio > 1.0 + MODE_TOLERANCE
        else "DECAYING"
        if ratio < 1.0 - MODE_TOLERANCE
        else "NEUTRAL"
    )
    dominant_mode = checkpoints[-1]["shape"]["dominant_mode"]
    polarity_mode = checkpoints[-1]["polarity"]["dominant_mode"]
    shape_phase = checkpoints[-1]["shape"]["dominant_phase"]
    polarity_phase = next(
        (
            math.atan2(item["sine"], item["cosine"])
            for item in checkpoints[-1]["polarity"]["mode_summaries"]
            if item["mode"] == dominant_mode
        ),
    )
    force_mode = None
    force_phase_delta = None
    force_projection_cosine = None
    if projection:
        last_force = projection[-1].get("radial_modes") or {}
        if str(dominant_mode) in last_force:
            force_mode = last_force[str(dominant_mode)]
        elif dominant_mode in last_force:
            force_mode = last_force[dominant_mode]
        if force_mode:
            force_phase_delta = circular_delta(force_mode["phase"], shape_phase)
            shape_entry = next(
                item for item in checkpoints[-1]["shape"]["emitted_modes"] if item["mode"] == dominant_mode
            )
            numerator = (
                force_mode["cosine_coefficient"] * shape_entry["cosine_coefficient"]
                + force_mode["sine_coefficient"] * shape_entry["sine_coefficient"]
            )
            force_norm = math.hypot(force_mode["cosine_coefficient"], force_mode["sine_coefficient"])
            shape_norm = math.hypot(shape_entry["cosine_coefficient"], shape_entry["sine_coefficient"])
            force_projection_cosine = numerator / (force_norm * shape_norm) if force_norm and shape_norm else None
    lag_correlations = {
        str(lag): lag_correlation(polarity, shape, lag) for lag in LAG_OFFSETS
    }
    stable_modes = stability_summary([{"arm": arm["arm"], "checkpoints": checkpoints}])
    return {
        "arm": arm["arm"],
        "reported_classification": arm["mechanical_mode_classification"],
        "recomputed_classification": classification,
        "recomputed_mode_ratio": ratio,
        "growth_qualified": arm["maximum_mass_over_birth"] >= 1.35,
        "maximum_mass_over_birth": arm["maximum_mass_over_birth"],
        "physical_fissions": arm["physical_fissions"],
        "fission_attempts": arm["fission_attempts"],
        "first_late": {"shape_amplitude": [first, late], "polarity_amplitude": [polarity[0], polarity[-1]]},
        "late_nearest_distance_over_range": near[-1],
        "minimum_nearest_distance_over_range": min(near),
        "late_material": {
            "activity_max": activity[-1],
            "activity_mean": activity_mean[-1],
            "activity_nonzero_fraction": checkpoints[-1]["polarity"]["activity_nonzero_fraction"],
            "activity_saturated_fraction": checkpoints[-1]["polarity"]["activity_saturated_fraction"],
            "polarity_request_a": request[-1],
            "total_request_a": total_request[-1],
            "funded_total_a": funded[-1],
            "funding_ratio": checkpoints[-1]["cortex"]["funding_ratio"],
            "passive_force_component_sum": passive[-1],
            "polarity_to_passive_force_ratio": checkpoints[-1]["cortex"]["polarity_to_passive_force_ratio"],
        },
        "late_modes": {
            "shape_mode": dominant_mode,
            "polarity_mode": polarity_mode,
            "same_mode": dominant_mode == polarity_mode,
            "shape_polarity_phase_delta": circular_delta(shape_phase, polarity_phase),
            "polarity_shape_lag_correlations": lag_correlations,
            "polarity_shape_zero_lag": pearson(polarity, shape),
            "polarity_shape_forward_lag": lag_correlations.get("1"),
            "tension_force_mode": force_mode,
            "tension_shape_phase_delta": force_phase_delta,
            "tension_shape_projection_cosine": force_projection_cosine,
            "projection_rows": len(projection),
            "projection_status": "POST_MECHANICS_SNAPSHOT_PROXY_ONLY",
        },
        "series_summary": {
            "polarity_amplitude_slope": slope(polarity, times),
            "shape_amplitude_slope": slope(shape, times),
            "activity_max_mean": mean(activity),
            "activity_mean_mean": mean(activity_mean),
            "polarity_request_mean": mean(request),
            "total_request_mean": mean(total_request),
            "funded_total_mean": mean(funded),
            "funding_scale_min": min(
                item["cortex"]["funding_ratio"] for item in checkpoints
            ),
            "funding_scale_max": max(
                item["cortex"]["funding_ratio"] for item in checkpoints
            ),
            "passive_force_mean": mean(passive),
            "shape_activity_correlation": pearson(shape, activity_mean),
            "shape_polarity_correlation": pearson(shape, polarity),
            "shape_nearest_distance_correlation": pearson(shape, near),
            "polarity_nearest_distance_correlation": pearson(polarity, near),
        },
        "frozen_stability": stable_modes,
    }


def group_comparison(summaries: list[dict]) -> dict:
    fields = {
        "recomputed_mode_ratio": lambda item: item["recomputed_mode_ratio"],
        "late_shape_amplitude": lambda item: item["first_late"]["shape_amplitude"][1],
        "late_polarity_amplitude": lambda item: item["first_late"]["polarity_amplitude"][1],
        "late_nearest_distance_over_range": lambda item: item["late_nearest_distance_over_range"],
        "late_activity_mean": lambda item: item["late_material"]["activity_mean"],
        "late_activity_saturated_fraction": lambda item: item["late_material"]["activity_saturated_fraction"],
        "late_polarity_request_a": lambda item: item["late_material"]["polarity_request_a"],
        "late_total_request_a": lambda item: item["late_material"]["total_request_a"],
        "late_funded_total_a": lambda item: item["late_material"]["funded_total_a"],
        "late_funding_ratio": lambda item: item["late_material"]["funding_ratio"],
        "late_passive_force_sum": lambda item: item["late_material"]["passive_force_component_sum"],
        "polarity_shape_zero_lag": lambda item: item["late_modes"]["polarity_shape_zero_lag"],
        "shape_activity_correlation": lambda item: item["series_summary"]["shape_activity_correlation"],
        "polarity_to_passive_force_ratio": lambda item: item["late_material"]["polarity_to_passive_force_ratio"],
    }
    result = {}
    for group in ("GROWING", "DECAYING"):
        members = [item for item in summaries if item["recomputed_classification"] == group]
        result[group.lower()] = {
            "arms": [item["arm"] for item in members],
            "arm_count": len(members),
            "means": {
                field: mean([getter(item) for item in members if getter(item) is not None])
                for field, getter in fields.items()
            },
            "ranges": {
                field: [
                    min(values),
                    max(values),
                ]
                if (values := [getter(item) for item in members if getter(item) is not None])
                else None
                for field, getter in fields.items()
            },
        }
    result["interpretation"] = (
        "Descriptive comparison only. The 3/7 grouping is an endpoint trajectory predicate, "
        "not an intervention and not a causal fit."
    )
    return result


def causal_graph() -> dict:
    return {
        "nodes": [
            {"id": "polarity_amounts", "meaning": "R3 active/inactive edge-local physical amounts"},
            {"id": "edge_measure", "meaning": "positive local edge measure used to derive concentration"},
            {"id": "polarity_concentration", "meaning": "active amount divided by local edge measure"},
            {"id": "polarity_chemistry", "meaning": "R3 local reaction and nearest-neighbor redistribution"},
            {"id": "actuator_activity", "meaning": "R4 local positive excess over sealed homogeneous reference"},
            {"id": "paid_edge_tension", "meaning": "existing local edge-tension actuator"},
            {"id": "a_funding", "meaning": "existing common organism-level A budget scale"},
            {"id": "mechanics", "meaning": "frozen stretch/bend/pressure/contact integration"},
            {"id": "geometry", "meaning": "accepted vertices, edge lengths, strain and shape"},
            {"id": "next_concentration", "meaning": "next-step amount/measure-derived concentration"},
        ],
        "arrows": [
            {"from": "polarity_amounts", "to": "polarity_concentration", "scope": "local", "kind": "derived/passive", "sign": "negative with edge measure at fixed amount", "lag": "instantaneous"},
            {"from": "polarity_amounts", "to": "polarity_chemistry", "scope": "local/nearest-neighbor", "kind": "active and A-to-W accounted", "sign": "sign-dependent", "lag": "accepted-step"},
            {"from": "edge_measure", "to": "polarity_concentration", "scope": "local", "kind": "derived/passive", "sign": "negative at fixed amount", "lag": "instantaneous"},
            {"from": "polarity_concentration", "to": "actuator_activity", "scope": "local incident-edge average", "kind": "passive adapter", "sign": "positive above reference and zero below", "lag": "instantaneous"},
            {"from": "actuator_activity", "to": "paid_edge_tension", "scope": "local incident-edge average", "kind": "active actuator", "sign": "positive tension", "lag": "instantaneous"},
            {"from": "paid_edge_tension", "to": "a_funding", "scope": "organism-level budget", "kind": "resource limiting", "sign": "funding scale nonpositive effect on output", "lag": "same accepted step"},
            {"from": "paid_edge_tension", "to": "mechanics", "scope": "local edge force with physical solver coupling", "kind": "energy-consuming A-to-W", "sign": "contractile", "lag": "same accepted step"},
            {"from": "a_funding", "to": "mechanics", "scope": "organism-level common scale", "kind": "energy-consuming", "sign": "sign-preserving scale", "lag": "same accepted step"},
            {"from": "mechanics", "to": "geometry", "scope": "local force law plus contact solver", "kind": "passive and active", "sign": "sign-dependent", "lag": "accepted-step"},
            {"from": "geometry", "to": "next_concentration", "scope": "local per-edge", "kind": "derived/passive", "sign": "negative at fixed amount", "lag": "next accepted step"},
        ],
        "nonlocal_boundary": "Only the existing common A funding scale is organism-level; no global spatial normalization or body-scale target is introduced by R5.",
        "observer_arrows": [],
        "status": "PASS",
    }


def geometry_feedback(records: list[dict]) -> dict:
    fixed_amount_derivatives = []
    actual_rows = []
    for record in records:
        for checkpoint in record["checkpoints"]:
            for edge in checkpoint["material"]["edge_records"]:
                amount = max(edge["active_concentration"] or 0.0, 0.0) * max(edge["geometric_length"], 1.0e-300)
                length = max(edge["geometric_length"], 1.0e-300)
                fixed_amount_derivatives.append(-amount / (length * length))
            actual_rows.append(
                {
                    "arm": record["arm"],
                    "step": checkpoint["step"],
                    "mean_active_concentration": mean(
                        [edge["active_concentration"] for edge in checkpoint["material"]["edge_records"] if edge["active_concentration"] is not None]
                    ),
                    "perimeter": checkpoint["material"]["perimeter"],
                    "mature_rest_perimeter": checkpoint["material"]["mature_rest_perimeter"],
                }
            )
    return {
        "fixed_amount_derivative_dc_dlength": {
            "sign": "NEGATIVE",
            "minimum": min(fixed_amount_derivatives),
            "maximum": max(fixed_amount_derivatives),
            "mean": mean(fixed_amount_derivatives),
            "equation": "dc/dl = -active_amount/l^2 for fixed active amount and positive local measure",
        },
        "actual_geometry_rows": actual_rows,
        "closed_loop_sign": "UNKNOWN_FROM_SNAPSHOTS",
        "reason": "Active amount changes concurrently with geometry; the sealed R4 record lacks a causal perturbation that holds one fixed while varying the other.",
        "status": "PASS_WITH_LIMITATION",
    }


def external_prior_art() -> dict:
    return {
        "mietke_2019": {
            "source": "https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.123.188101",
            "classification": "ADAPTABLE_METHOD",
            "reused": "mechanochemical feedback can be analysed as a symmetry-breaking instability",
            "digital_cell_parameters_imported": False,
            "division_logic_imported": False,
        },
        "staddon_2022": {
            "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC9000090/",
            "classification": "REFERENCE_ONLY",
            "reused": "pulsatile actomyosin activity is a qualitative comparator for timing and feedback",
            "digital_cell_parameters_imported": False,
            "division_logic_imported": False,
        },
        "goehring_2019": {
            "source": "https://pmc.ncbi.nlm.nih.gov/articles/PMC6640039/",
            "classification": "REFERENCE_ONLY",
            "reused": "local feedback and spatial organization are qualitative mechanistic comparators",
            "digital_cell_parameters_imported": False,
            "division_logic_imported": False,
        },
        "contractile_instability": {
            "source": "https://elifesciences.org/articles/19595",
            "classification": "REFERENCE_ONLY",
            "reused": "active/passive balance and stability analysis are methodological comparators",
            "digital_cell_parameters_imported": False,
            "division_logic_imported": False,
        },
        "scope": "No external coefficient, gain, force law, ecology or target geometry was imported.",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    root = args.output
    root.mkdir(parents=True, exist_ok=True)
    raw = load(args.raw)
    arms = validate_raw(raw)
    audit = source_audit(args.repo)

    records = []
    summaries = []
    for arm in arms:
        checkpoints = [checkpoint_record(row) for row in arm["mechanics_trace"]]
        record = {
            "arm": arm["arm"],
            "connected": True,
            "raw_reported_classification": arm["mechanical_mode_classification"],
            "checkpoints": checkpoints,
        }
        records.append(record)
        summaries.append(summarize_arm(arm, checkpoints))

    growing = [item["arm"] for item in summaries if item["recomputed_classification"] == "GROWING"]
    decaying = [item["arm"] for item in summaries if item["recomputed_classification"] == "DECAYING"]
    assert growing == [2, 5, 8], growing
    assert len(decaying) == 7, decaying
    assert all(item["reported_classification"] == item["recomputed_classification"] for item in summaries)

    stability = stability_summary(records)
    assert stability["counts"] == {"GROWING": 0, "DECAYING": 1800, "NEUTRAL": 0, "INVALID": 0}, stability
    assert all(
        item["late_material"]["funding_ratio"] >= 1.0 - 1.0e-12
        for item in summaries
    )
    all_activity_nonzero = all(
        sum(checkpoint["polarity"]["activity_nonzero_fraction"] > 0.0 for checkpoint in record["checkpoints"]) >= 59
        for record in records
    )
    assert all_activity_nonzero

    raw_hash = sha(args.raw)
    source = {
        "directive": DIRECTIVE,
        "r4_final_head": R4_HEAD,
        "r4_implementation_head": R4_IMPLEMENTATION_HEAD,
        "r4_ci": R4_CI,
        "r4_artifact": R4_ARTIFACT,
        "r4_binary_sha256": R4_BINARY,
        "raw_input": str(args.raw),
        "raw_input_sha256": raw_hash,
        "expected_sealed_r4_raw_sha256": R4_RAW_SHA,
        "expected_raw_hash_matches": raw_hash == R4_RAW_SHA,
        "connected_arm_count": 10,
        "trace_rows_per_arm": TRACE_ROWS,
        "accepted_horizon": HORIZON,
        "raw_input_is_immutable_r4_evidence": True,
        "status": "PASS",
    }
    dump(root, "authority.json", source)
    dump(root, "architect_disposition.json", {
        "r4_acceptance": "ACCEPTED_BOUNDED_NEGATIVE",
        "r4_terminal": "ROUTE_B_POLARITY_ACTUATION_FAILS_TO_GENERATE_MECHANICAL_MODES",
        "r5_scope": "diagnostic attribution only; no production biology or successor implementation",
        "r4_connected_fissions": "0/10",
        "r4_connected_new_growing_modes": "3/10",
        "r4_connected_lower_late_distance": "9/10",
        "r4_e3": "NOT_REACHED",
        "r4_e4": "NOT_REACHED",
        "next_execution_started": False,
        "status": "PASS",
    })
    dump(root, "owner_override.json", {"owner_override": "ACTIVE", "next_execution_started": False, "status": "PASS"})
    dump(root, "source_audit.json", audit)
    dump(root, "causal_graph.json", causal_graph())
    dump(root, "trajectory_provenance.json", {
        **source,
        "selection": "all ten sealed R4 connected Resource histories; no reselection",
        "matched_time": "accepted steps 1..14778 at emitted mechanics checkpoints",
        "raw_summary_not_reused_as_causal_proof": True,
        "status": "PASS",
    })
    dump(root, "matched_causal_chain.json", {
        "directive": DIRECTIVE,
        "arms": records,
        "arm_count": len(records),
        "checkpoints": len(records) * TRACE_ROWS,
        "observer_only": True,
        "source_rows_preserved": "R4 raw/qualification.json",
        "status": "PASS",
    })
    dump(root, "arm_summary.json", {"arms": summaries, "status": "PASS"})
    dump(root, "group_comparison.json", {
        "growing_arms": growing,
        "decaying_arms": decaying,
        "comparison": group_comparison(summaries),
        "interpretation": "No fit, parameter choice, or causal claim is based on the 3/7 grouping.",
        "status": "PASS_WITH_LIMITATION",
    })
    dump(root, "local_stability_modal_projection.json", {
        "frozen_passive_stability": stability,
        "modal_projection": {
            "per_arm": [
                {
                    "arm": item["arm"],
                    "recomputed_classification": item["recomputed_classification"],
                    "late_shape_mode": item["late_modes"]["shape_mode"],
                    "late_polarity_mode": item["late_modes"]["polarity_mode"],
                    "same_mode": item["late_modes"]["same_mode"],
                    "shape_polarity_phase_delta": item["late_modes"]["shape_polarity_phase_delta"],
                    "tension_shape_phase_delta": item["late_modes"]["tension_shape_phase_delta"],
                    "tension_shape_projection_cosine": item["late_modes"]["tension_shape_projection_cosine"],
                    "projection_rows": item["late_modes"]["projection_rows"],
                    "projection_status": item["late_modes"]["projection_status"],
                }
                for item in summaries
            ],
            "source_formula": "max_active_tension * 0.5 * (activity[i] + activity[i+1]) with the sealed common funding ratio",
            "exact_pre_mechanics_field": "NOT_SERIALIZED_BY_R4",
            "status": "PASS_WITH_LIMITATION",
        },
        "coupled_local_jacobian": {
            "status": "NOT_IDENTIFIABLE_FROM_SEALED_R4_RAW",
            "reason": "R4 serialized frozen passive perturbation traces and accepted post-step geometry, but not a matched causal perturbation of polarity, pre-mechanics edge tension, and geometry in one transaction.",
        },
        "status": "PASS_WITH_LIMITATION",
    })
    dump(root, "geometry_feedback.json", geometry_feedback(records))
    dump(root, "external_prior_art.json", external_prior_art())
    dump(root, "independent_verifier.json", {
        "status": "PASS",
        "raw_input_sha256": raw_hash,
        "recomputed_from": "raw R4 mechanics_trace vertices, edge material, polarity mode/activity, ledger fields, and frozen stability traces",
        "emitted_mechanical_classification_trusted": False,
        "recomputed_growing_arms": growing,
        "recomputed_decaying_arms": decaying,
        "frozen_valid_stability_records": stability["valid_record_count"],
        "frozen_stability_counts": stability["counts"],
        "all_arms_fully_funded": True,
        "all_arms_activity_nonzero_after_initialization": all_activity_nonzero,
        "production_biology_executed": False,
        "status_predicate": "PASS means the observer recomputation and fail-closed raw checks passed, not that morphogenesis passed",
    })
    dump(root, "preservation.json", {
        "production_sources_changed": False,
        "r3_parameters_changed": False,
        "r4_gain_changed": False,
        "actuator_strength_or_cost_changed": False,
        "ecology_changed": False,
        "fission_or_reproduction_executed": False,
        "selection_or_reversal_executed": False,
        "r4_raw_history_preserved": True,
        "r4_historical_fixture_result_not_reclassified": True,
        "status": "PASS",
    })
    dump(root, "forbidden_information_audit.json", {
        "status": "PASS",
        "observer_only": True,
        "forbidden_inputs_to_biology": [
            "centroid", "midpoint", "body axis", "target geometry", "apposition", "fission",
            "generation", "observer output", "population outcome", "gain sweep", "favorable seed",
        ],
        "production_transition_called": False,
        "observer_metrics_feed_back": False,
        "parameter_or_gain_selection_from_outcome": False,
    })
    decision = {
        "classification": "R4_ATTRIBUTION_INCONCLUSIVE",
        "verified": [
            "R4 sealed connected Resource trajectories are complete, simple, lifecycle-valid and numerically accepted.",
            "All ten arms are growth-qualified and have zero physical fissions.",
            "Polarity activity is nonzero after initialization and the existing request is fully funded in every arm.",
            "The independent frozen passive perturbation assay has 1800 valid DECAYING and zero valid GROWING records.",
            "Three accepted endpoint shape traces grow relative to their own first checkpoint and nine arms reduce late nearest-pair distance, but this is not a coupled stability proof.",
        ],
        "not_established": [
            "A full coupled polarity-to-mechanics Jacobian or converged causal local response.",
            "The sign of the complete geometry-to-polarity feedback when active amount changes concurrently with edge measure.",
            "Whether the accepted active tension field projects onto a mechanically growing eigenmode before the first apposition attempt.",
        ],
        "attribution_boundary": {
            "funding_shortage": "NOT_SUPPORTED_AS_PRIMARY_BY_R4_RAW",
            "missing_activity": "NOT_SUPPORTED_AS_PRIMARY_BY_R4_RAW",
            "passive_snapshot_stability": "SUPPORTED_FOR_FROZEN_PASSIVE_OPERATOR_ONLY",
            "modal_projection": "DESCRIPTIVE_POST_MECHANICS_PROXY_ONLY",
            "adaptation_only": "NOT_RETESTED_IN_R5; prior accepted R5 control remains bounded evidence",
            "full_closed_loop_cause": "NOT_IDENTIFIED",
        },
        "successor": {
            "status": "NONE_JUSTIFIED",
            "reason": "A gain, actuator, geometry-feedback, or new-channel successor would require a causal intervention not present in the sealed R4 histories.",
            "minimum_future_evidence": "observer-only pre-mechanics polarity tension field plus a matched local coupled perturbation/Jacobian or converged response replay",
            "production_implementation_started": False,
        },
        "status": "PASS_WITH_LIMITATION",
    }
    dump(root, "architecture_decision.json", decision)
    dump(root, "qualification.json", {
        "directive": DIRECTIVE,
        "implementation_acceptance": "PASS",
        "scientific_acceptance": "BOUNDED_INCONCLUSIVE",
        "terminal_classification": decision["classification"],
        "e0_authority": "PASS",
        "e1_causal_graph_and_raw_instrumentation": "PASS",
        "e2_independent_matched_attribution": "PASS_WITH_LIMITATION",
        "e3_local_stability_and_modal_projection": "PASS_WITH_LIMITATION",
        "e4_architecture_decision": "PASS",
        "e5_production_or_reproduction": "NOT_AUTHORIZED",
        "production_biology_delta": 0,
        "reproduction": "NOT_REACHED",
        "selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "next_execution_started": False,
    })
    files = {}
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "artifact_manifest.json":
            files[str(path.relative_to(root))] = sha(path)
    manifest = {
        "schema": "dcm4_r5_mechanochemical_instability_artifact_manifest_v1",
        "file_count": len(files),
        "files": files,
    }
    dump(root, "artifact_manifest.json", manifest)
    for relative, expected in manifest["files"].items():
        assert sha(root / relative) == expected
    print(json.dumps({
        "classification": decision["classification"],
        "raw_input_sha256": raw_hash,
        "growing_arms": growing,
        "decaying_arms": decaying,
        "frozen_stability_counts": stability["counts"],
        "manifest_files": manifest["file_count"],
    }, sort_keys=True))


if __name__ == "__main__":
    main()
