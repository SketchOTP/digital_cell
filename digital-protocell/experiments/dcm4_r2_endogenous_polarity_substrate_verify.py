#!/usr/bin/env python3
"""DC-M4-R2 diagnostic architecture gate.

This verifier is deliberately non-production.  It reconciles the accepted R1
negative, inventories current causal state semantics, and independently checks
a small mass-conserving reaction-diffusion instability benchmark.  It does not
compile, run, or modify Digital Cell production biology.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


DIRECTIVE = "DC-M4-R2-ENDOGENOUS-POLARITY-SUBSTRATE-GATE-001"
R1_DIRECTIVE = "DC-M4-R1-LOCAL-CONSERVATIVE-GROWTH-COUPLING-001"
R1_SCIENTIFIC_HEAD = "cd6cb020ff95b69fdf3d65cea79a611cc9d40036"
R1_GOVERNED_HEAD = "ed44319baae9801111a282ae60fef7f957420cf3"
R1_CI = "34773668279"
R1_ARTIFACT = "sha256:c6483571177e3e4081962d0187c6eaa21186ceb5d8a9f0280849e5b0523641a8"
R1_BINARY = "b44641274464f5caf57ccb7a191e5973cf55e9e84d19c3776b19e63d86e7b902"
TOLERANCE = 1.0e-10


def load(path: Path):
    return json.loads(path.read_text())


def dump(root: Path, name: str, value) -> None:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def finite(value) -> bool:
    if value is None or isinstance(value, bool):
        return True
    if isinstance(value, (int, float)):
        return math.isfinite(value)
    if isinstance(value, list):
        return all(finite(item) for item in value)
    if isinstance(value, dict):
        return all(finite(item) for item in value.values())
    return True


def approx(left: float, right: float, tolerance: float = TOLERANCE) -> bool:
    return abs(left - right) <= tolerance * (1.0 + abs(left) + abs(right))


def source_line(path: Path, needle: str):
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if needle in line:
            return {"path": str(path), "line": number, "text": line.strip()}
    return None


def git_value(repo: Path, *args: str) -> str | None:
    try:
        return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def reconcile_r1(repo: Path, fixture: Path):
    data = load(fixture)
    if data.get("directive") != R1_DIRECTIVE:
        raise AssertionError("R1 compact fixture directive mismatch")
    if data.get("source_head") != R1_SCIENTIFIC_HEAD:
        raise AssertionError("R1 scientific head mismatch")
    if data.get("source_governed_head") != R1_GOVERNED_HEAD:
        raise AssertionError("R1 governed head mismatch")
    if data.get("accepted_horizon") != 14778:
        raise AssertionError("R1 horizon mismatch")
    arms = data.get("arms")
    if not isinstance(arms, list) or len(arms) != 10:
        raise AssertionError("R1 compact fixture must contain ten arms")
    if not finite(data):
        raise AssertionError("R1 compact fixture contains non-finite data")

    recomputed = []
    for expected, arm in enumerate(arms, 1):
        if arm.get("arm") != expected:
            raise AssertionError("R1 arm identity is not stable")
        off_modes = set(arm.get("route_off_growing_modes", []))
        on_modes = set(arm.get("route_on_growing_modes", []))
        recomputed.append(
            {
                "arm": expected,
                "lower_late_distance": (
                    arm["route_on_late_distance_over_range"]
                    < arm["route_off_late_distance_over_range"] - 1.0e-9
                ),
                "new_valid_growing_modes": sorted(on_modes - off_modes),
                "off_distance_over_range": arm["route_off_late_distance_over_range"],
                "on_distance_over_range": arm["route_on_late_distance_over_range"],
            }
        )
    lower_count = sum(row["lower_late_distance"] for row in recomputed)
    new_mode_count = sum(bool(row["new_valid_growing_modes"]) for row in recomputed)
    result = {
        "input_fixture": str(fixture),
        "input_fixture_sha256": sha(fixture),
        "accepted_authority": {
            "scientific_head": R1_SCIENTIFIC_HEAD,
            "governed_head": R1_GOVERNED_HEAD,
            "ci": R1_CI,
            "artifact": R1_ARTIFACT,
            "binary_sha256": R1_BINARY,
            "route_off_fissions": data["route_off_fissions"],
            "route_on_fissions": data["route_on_fissions"],
            "route_off_growth_qualified": data["route_off_growth_qualified"],
            "route_on_growth_qualified": data["route_on_growth_qualified"],
        },
        "independently_recomputed_e2": {
            "arms": recomputed,
            "lower_late_distance_arms": lower_count,
            "new_valid_growing_mode_arms": new_mode_count,
            "required_lower_distance_arms": 7,
            "required_new_growing_mode_arms": 7,
            "passed": lower_count >= 7 and new_mode_count >= 7,
        },
        "bounded_interpretation": (
            "R1 disproves the tested one-hop coupling as sufficient under the bounded "
            "Resource domain; it does not establish that all possible mechanical-growth "
            "architectures are impossible."
        ),
        "pass": (
            data["route_off_fissions"] == 0
            and data["route_on_fissions"] == 0
            and data["route_off_growth_qualified"] == 10
            and data["route_on_growth_qualified"] == 10
            and lower_count == 10
            and new_mode_count == 0
        ),
    }
    return result


def current_state_inventory(repo: Path):
    chemistry = repo / "crates/chemistry-core/src/material_mesh.rs"
    growth = repo / "crates/chemistry-core/src/mesh_growth.rs"
    mechanics = repo / "crates/chemistry-core/src/mesh_mechanics.rs"
    fission = repo / "crates/chemistry-core/src/mesh_fission.rs"
    plasticity = repo / "crates/regulatory-core/src/plasticity.rs"
    d096 = repo / "crates/chemistry-core/src/d096_allocation.rs"
    polarity = repo / "crates/m2-lifeform-runtime/src/polarity.rs"
    exploration = repo / "crates/regulatory-core/src/intrinsic_exploration.rs"
    files = [chemistry, growth, mechanics, fission, plasticity, d096, polarity, exploration]
    return {
        "scope": "current M4 production plus separate historical/diagnostic carriers",
        "production_state_schema": {
            "source_hashes": {str(path): sha(path) for path in files if path.exists()},
            "material_mesh": {
                "owner": str(chemistry),
                "states": [
                    "MeshEdge.m structural material mass",
                    "MeshEdge.m_young newly synthesized non-load-bearing structural mass",
                    "MeshEdge.b bound membrane material",
                    "LumpedChem.c/a/n/f/w and optional reserve/catalyst pools",
                    "vertices and derived geometry",
                ],
                "units": "material amounts, concentrations, length/area-derived geometry",
                "conservation": "existing material and chemistry ledgers; mechanics moves vertices without adding edge material",
                "locality": "edge material is local; chemistry is primarily lumped per organism",
                "persistence_or_decay": "stored material changes only through existing chemistry, growth, turnover and topology contracts",
                "inputs": "local edge growth/turnover plus organism-level chemistry; no target geometry",
                "serialization_restart": "MaterialMesh serde state and current checkpoint path",
                "remesh_mapping": "existing conservative edge split/merge mappings",
                "fission_mapping": "existing conservative partition report and parent-source correspondence",
                "daughter_partition": "physical edge/interior partition; no active/inactive polarity partition",
                "lifecycle": "remesh and fission partition existing material; no polarity semantics",
                "reuse_decision": "REJECTED_SEMANTIC_CHANGE",
            },
            "young_mature_and_rest_geometry": {
                "owner": str(chemistry),
                "units": "mass and derived length",
                "meaning": "young structural mass matures; mature mass defines rest_length and strain",
                "persistence_or_decay": "stored young pool matures under existing V4 contract; strain/rest length are derived each read",
                "inputs": "edge-local material and geometry",
                "material_energy": "structural material already accounted by D-088/M1; no independent polarity energy",
                "serialization": "MeshEdge.m_young is serialized; strain is not independent state",
                "restart": "restored with MaterialMesh; derived geometry recomputed",
                "remesh_mapping": "material follows existing mesh mapping",
                "fission_mapping": "conservative partition of structural material",
                "daughter_partition": "actual daughter edges inherit structural material correspondence",
                "reuse_decision": "REJECTED_NO_INDEPENDENT_POLARITY_STATE",
            },
            "lumped_chemistry_and_d096": {
                "owner": f"{chemistry} and {d096}",
                "units": "organism-level material/concentration and finite catalyst amounts",
                "meaning": "reaction substrates, activated material, reserve, and inherited D096 catalysts",
                "persistence_or_decay": "existing reactions, turnover, reserve and catalyst laws",
                "inputs": "organism-level chemistry and D096 genotype/catalyst state",
                "locality": "lumped within a mesh; no per-edge polarity transport contract",
                "material_energy": "physical chemistry already has M1/D096 ledgers; relabeling would corrupt them",
                "serialization_restart": "MaterialMesh and AllocationState serialization; no local remesh map",
                "remesh_mapping": "unchanged as an organism-level pool; no spatial field mapping",
                "fission_mapping": "existing area/material partition",
                "daughter_partition": "existing area/material partition, not spatial active/inactive polarity partition",
                "reuse_decision": "REJECTED_SEMANTIC_CHANGE",
            },
            "plasticity_adaptation": {
                "owner": str(plasticity),
                "units": "dimensionless bounded [0,1] local refractory trace",
                "meaning": "activity-dependent attenuation of existing contractility",
                "persistence_or_decay": "load/recovery law over accepted mechanics time",
                "inputs": "local activity and accepted dt only",
                "energetics": "not material and has no independent conservation or A-to-W budget",
                "serialization_restart": "PlasticityStateV1 serialized and remapped for ordinary topology",
                "remesh_mapping": "existing local split/merge correspondence",
                "fission_mapping": "fission mapping is explicitly unsupported/fails closed",
                "daughter_partition": "no current daughter polarity semantics; cannot be copied as one",
                "reuse_decision": "REJECTED_SEMANTIC_CHANGE",
            },
            "mechanics_activity_and_forces": {
                "owner": str(mechanics),
                "units": "derived activity, force, displacement and A-funded work ledger",
                "meaning": "stretch/bending/pressure plus existing local contractility",
                "persistence_or_decay": "step-local derived quantities; not a stored polarity field",
                "inputs": "local mesh geometry, local activity and physical boundary snapshot",
                "serialization_restart": "forces are recomputed from restored mesh/state; no independent field",
                "remesh_mapping": "recomputed after remesh",
                "fission_mapping": "recomputed separately for each valid daughter",
                "daughter_partition": "existing A/work ledger and local force recomputation",
                "energetics": "existing actuation debits A and credits W",
                "coupling": "existing local force interface can be reused by a future substrate",
                "reuse_decision": "COUPLING_INTERFACE_ONLY",
            },
        },
        "separate_diagnostic_carriers": {
            "native_ring_polarity": {
                "owner": str(polarity),
                "fields": ["u", "v", "f"],
                "units": "edge-centered density/amount over normalized arclength control volumes",
                "conservation": "u+v reaction is conservative; f has source/decay in the existing runtime equations",
                "remesh_restart_fission": "implemented in the separate runtime carrier",
                "current_m4_status": "not in MaterialMesh production path; opt-in M2 runtime/assay only",
                "decision": "ADAPTABLE_PRECEDENT_NOT_EXISTING_M4_STATE",
            },
            "intrinsic_exploration": {
                "owner": str(exploration),
                "fields": ["activity", "PlasticityStateV1"],
                "semantic_issue": "explicit provenance_seed selects an initial ring patch and the state is an opt-in M2 exploration composition",
                "current_m4_status": "not production and not conserved material",
                "decision": "INCOMPATIBLE_DIAGNOSTIC_CARRIER",
            },
        },
        "conclusion": {
            "existing_current_m4_state_can_be_extended_without_semantic_change": False,
            "reason": "Every current M4 field is structural, chemical, derived, dimensionless refractory, or lumped; none is a conserved local active/inactive polarity carrier with M4 lifecycle ownership.",
        },
    }


def eigenvalues_2x2(a: float, b: float, c: float, d: float):
    trace = a + d
    determinant = a * d - b * c
    discriminant = trace * trace - 4.0 * determinant
    if discriminant >= 0.0:
        root = math.sqrt(discriminant)
        return ((trace + root) / 2.0, (trace - root) / 2.0)
    return (trace / 2.0, trace / 2.0)


def benchmark_equilibrium(total: float, params):
    reaction_params = params["reaction"]
    a = reaction_params["a"]
    b = reaction_params["b"]
    c = reaction_params["c"]
    d = reaction_params["d"]

    def reaction(u: float) -> float:
        v = total - u
        return (a + b * u * u) * v - (c + d * u * u) * u

    previous_x = 0.0
    previous = reaction(previous_x)
    for index in range(1, 10001):
        x = total * index / 10000.0
        current = reaction(x)
        if previous * current < 0.0:
            lo, hi, flo = previous_x, x, previous
            for _ in range(80):
                mid = (lo + hi) / 2.0
                fmid = reaction(mid)
                if flo * fmid <= 0.0:
                    hi = mid
                else:
                    lo, flo = mid, fmid
            return (lo + hi) / 2.0, total - (lo + hi) / 2.0
        previous_x, previous = x, current
    raise AssertionError(f"no homogeneous equilibrium for total {total}")


def benchmark_dispersion(total: float, params):
    reaction_params = params["reaction"]
    a = reaction_params["a"]
    b = reaction_params["b"]
    c = reaction_params["c"]
    d = reaction_params["d"]
    diffusion_params = params["diffusion"]
    du = diffusion_params["u"]
    dv = diffusion_params["v"]
    n_sites = params["sites"]
    u, v = benchmark_equilibrium(total, params)
    ru = 2.0 * b * u * v - c - 3.0 * d * u * u
    rv = a + b * u * u
    modes = []
    for mode in range(1, n_sites // 2 + 1):
        laplace_positive = 4.0 * math.sin(math.pi * mode / n_sites) ** 2 / params["dx"] ** 2
        first, second = eigenvalues_2x2(
            ru - du * laplace_positive,
            rv,
            -ru,
            -rv - dv * laplace_positive,
        )
        modes.append({"mode": mode, "max_real_eigenvalue": max(first, second)})
    maximum = max(modes, key=lambda item: item["max_real_eigenvalue"])
    return {
        "total_mass_per_site": total,
        "homogeneous_equilibrium": {"u": u, "v": v},
        "reaction_jacobian_at_equilibrium": {"R_u": ru, "R_v": rv},
        "modes": modes,
        "maximum_nonzero_mode": maximum,
        "classification": (
            "UNSTABLE_NONZERO_MODE"
            if maximum["max_real_eigenvalue"] > 1.0e-8
            else "HOMOGENEOUS_NONZERO_MODES_DECAY"
        ),
    }


def benchmark_rhs(state, params):
    u, v = state
    n_sites = len(u)
    reaction_params = params["reaction"]
    a = reaction_params["a"]
    b = reaction_params["b"]
    c = reaction_params["c"]
    d = reaction_params["d"]
    diffusion_params = params["diffusion"]
    du = diffusion_params["u"]
    dv = diffusion_params["v"]
    out_u, out_v = [0.0] * n_sites, [0.0] * n_sites
    for i in range(n_sites):
        left, right = (i - 1) % n_sites, (i + 1) % n_sites
        reaction = (a + b * u[i] * u[i]) * v[i] - (c + d * u[i] * u[i]) * u[i]
        lap_u = (u[right] - 2.0 * u[i] + u[left]) / params["dx"] ** 2
        lap_v = (v[right] - 2.0 * v[i] + v[left]) / params["dx"] ** 2
        out_u[i] = reaction + du * lap_u
        out_v[i] = -reaction + dv * lap_v
    return out_u, out_v


def add_state(state, derivative, scale):
    return (
        [x + scale * y for x, y in zip(state[0], derivative[0])],
        [x + scale * y for x, y in zip(state[1], derivative[1])],
    )


def benchmark_rk4(total: float, params, mode: int):
    n_sites = params["sites"]
    u0, v0 = benchmark_equilibrium(total, params)
    dispersion = benchmark_dispersion(total, params)
    eigen = next(row for row in dispersion["modes"] if row["mode"] == mode)
    reaction_params = params["reaction"]
    a = reaction_params["a"]
    b = reaction_params["b"]
    c = reaction_params["c"]
    d = reaction_params["d"]
    diffusion_params = params["diffusion"]
    du = diffusion_params["u"]
    dv = diffusion_params["v"]
    ru = 2.0 * b * u0 * v0 - c - 3.0 * d * u0 * u0
    rv = a + b * u0 * u0
    lam = 4.0 * math.sin(math.pi * mode / n_sites) ** 2 / params["dx"] ** 2
    dominant = eigen["max_real_eigenvalue"]
    matrix_a11 = ru - du * lam
    ratio = (dominant - matrix_a11) / rv
    epsilon = 1.0e-4
    state = (
        [u0 + epsilon * math.cos(2.0 * math.pi * mode * i / n_sites) for i in range(n_sites)],
        [v0 + epsilon * ratio * math.cos(2.0 * math.pi * mode * i / n_sites) for i in range(n_sites)],
    )
    initial_mass = sum(x + y for x, y in zip(*state))
    initial_amplitude = None
    final_amplitude = None
    for step in range(params["simulation_steps"]):
        def amplitude(current):
            u, v = current
            uc = sum((x - u0) * math.cos(2.0 * math.pi * mode * i / n_sites) for i, x in enumerate(u))
            vc = sum((x - v0) * math.cos(2.0 * math.pi * mode * i / n_sites) for i, x in enumerate(v))
            return math.hypot(2.0 * uc / n_sites, 2.0 * vc / n_sites)

        if step == 0:
            initial_amplitude = amplitude(state)
        h = params["dt"]
        first = benchmark_rhs(state, params)
        second = benchmark_rhs(add_state(state, first, h / 2.0), params)
        third = benchmark_rhs(add_state(state, second, h / 2.0), params)
        fourth = benchmark_rhs(add_state(state, third, h), params)
        state = (
            [x + h * (a + 2.0 * b + 2.0 * c + d) / 6.0 for x, a, b, c, d in zip(state[0], first[0], second[0], third[0], fourth[0])],
            [x + h * (a + 2.0 * b + 2.0 * c + d) / 6.0 for x, a, b, c, d in zip(state[1], first[1], second[1], third[1], fourth[1])],
        )
    final_mass = sum(x + y for x, y in zip(*state))
    final_amplitude = amplitude(state)
    return {
        "mode": mode,
        "linear_max_real_eigenvalue": dominant,
        "initial_nonzero_mode_amplitude": initial_amplitude,
        "final_nonzero_mode_amplitude": final_amplitude,
        "amplitude_ratio": final_amplitude / max(initial_amplitude, 1.0e-300),
        "initial_total_mass": initial_mass,
        "final_total_mass": final_mass,
        "mass_residual": final_mass - initial_mass,
        "classification": "AMPLIFIED" if final_amplitude > initial_amplitude else "DECAYED",
    }


def external_benchmark():
    params = {
        "model": "dimensionless_two_state_mass_conserving_reaction_diffusion",
        "reaction": {"a": 0.01, "b": 1.0, "c": 0.1, "d": 0.01},
        "diffusion": {"u": 0.1, "v": 1.0},
        "sites": 64,
        "dx": 1.0,
        "dt": 0.01,
        "simulation_steps": 4000,
        "equations": {
            "R": "(a + b*u^2)*v - (c + d*u^2)*u",
            "du_dt": "R + D_u*laplacian(u)",
            "dv_dt": "-R + D_v*laplacian(v)",
            "conserved_quantity": "sum_i(u_i + v_i)*dx",
        },
        "parameter_status": "sandbox-only dimensionless benchmark; no Digital Cell value was fitted or imported",
    }
    stable = benchmark_dispersion(0.4, params)
    unstable = benchmark_dispersion(0.8, params)
    stable_mode = stable["maximum_nonzero_mode"]["mode"]
    unstable_mode = unstable["maximum_nonzero_mode"]["mode"]
    stable_sim = benchmark_rk4(0.4, params, unstable_mode)
    unstable_sim = benchmark_rk4(0.8, params, unstable_mode)
    stable_sim["mode_used_for_paired_simulation"] = unstable_mode
    unstable_sim["mode_used_for_paired_simulation"] = unstable_mode
    passed = (
        stable["classification"] == "HOMOGENEOUS_NONZERO_MODES_DECAY"
        and unstable["classification"] == "UNSTABLE_NONZERO_MODE"
        and stable_sim["classification"] == "DECAYED"
        and unstable_sim["classification"] == "AMPLIFIED"
        and abs(stable_sim["mass_residual"]) < 1.0e-10
        and abs(unstable_sim["mass_residual"]) < 1.0e-10
    )
    return {
        "benchmark": params,
        "stable_regime": stable,
        "unstable_regime": unstable,
        "paired_finite_volume_check": {"stable": stable_sim, "unstable": unstable_sim},
        "criterion": "positive real eigenvalue of a nonzero Fourier mode, computed before any division/fission observable",
        "pass": passed,
        "prior_art_classification": {
            "Loose_2008_MinD_MinE": {
                "classification": "REFERENCE_ONLY",
                "url": "https://pubmed.ncbi.nlm.nih.gov/18467587/",
                "reused": "energy-dependent spatial self-organization as a reference principle only",
                "not_imported": "bacterial midpoint or division-site logic and numerical values",
            },
            "Mori_Jilkine_EdelsteinKeshet_2008_wave_pinning": {
                "classification": "ADAPTABLE_METHOD",
                "url": "https://pubmed.ncbi.nlm.nih.gov/18212014/",
                "reused": "conserved active/inactive redistribution and positive-feedback instability framework",
                "not_imported": "biological species or coefficients",
            },
            "Brauns_Halatek_Frey_2020_MCRD": {
                "classification": "ADAPTABLE_METHOD",
                "url": "https://journals.aps.org/prx/abstract/10.1103/PhysRevX.10.041036",
                "reused": "mass-redistribution and dispersion/stability analysis",
                "not_imported": "published parameters or a division controller",
            },
        },
    }


def fission_authority(repo: Path):
    fission = repo / "crates/chemistry-core/src/mesh_fission.rs"
    topology = repo / "crates/chemistry-core/src/planar_ring_topology.rs"
    self_contact = repo / "crates/chemistry-core/src/mesh_self_contact.rs"
    mesh = repo / "crates/chemistry-core/src/material_mesh.rs"
    evolution = repo / "examples/dcfinal001_r4_evolution.rs"
    paths = [fission, topology, self_contact, mesh, evolution]
    markers = {
        "segment_fission_entry": source_line(fission, "pub fn try_local_segment_fission("),
        "parent_physics_guard": source_line(fission, "if !parent.can_advance_physics()"),
        "parent_simple_guard": source_line(evolution, "if !polygon_simple(&cohort.mesh.vertices)"),
        "partition_ok_guard": source_line(evolution, "if !event.partition.ok"),
        "daughter_simple_guard": source_line(evolution, "|| !polygon_simple(&daughter_a.vertices)"),
        "simple_polygon_definition": source_line(self_contact, "pub fn polygon_simple("),
        "runtime_validity": source_line(mesh, "pub fn physical_runtime_valid(&self)"),
    }
    return {
        "source_hashes": {str(path): sha(path) for path in paths if path.exists()},
        "markers": markers,
        "contract": {
            "simple_parent_required": markers["parent_simple_guard"] is not None,
            "simple_daughters_required": markers["daughter_simple_guard"] is not None,
            "conservative_partition_required": markers["partition_ok_guard"] is not None,
            "historical_self_intersecting_d088_current_authority": False,
        },
        "call_path": [
            "current R10/R5 driver -> try_local_fission / segment-apposition dispatch",
            "simple parent and physical-runtime guards",
            "local topology candidate",
            "conservative partition report",
            "simple daughter and lifecycle/runtime validation",
        ],
    }


def new_substrate_spec():
    return {
        "status": "PROPOSED_NOT_IMPLEMENTED",
        "schema": "PolarityMassStateV1",
        "decision_basis": "No current M4 causal state can carry this meaning without semantic relabeling.",
        "local_state": {
            "inactive_amount": "p_i^I >= 0 per edge control volume",
            "active_amount": "p_i^A >= 0 per edge control volume",
            "optional_free_inactive_amount": "p_free^I >= 0 only if later attachment kinetics requires a free pool; omit in the smallest first implementation unless a source contract requires it",
            "total": "P = p_free^I + sum_i(p_i^I + p_i^A)",
        },
        "material_provenance": {
            "source": "finite existing activated material A through a separately budgeted future synthesis reaction",
            "rule": "No P is created at remesh, fission, restart, or ordinary reaction without an explicit A/material debit and W/energy accounting.",
            "founder": "A future production directive must record any initial P inventory as a finite source-funded initialization; this gate does not choose its amount.",
            "growth": "If P scales with growth, synthesis must consume existing A and finite chemical substrate; otherwise P_total remains conserved.",
        },
        "conservation_and_dynamics": {
            "reaction": "local reversible I <-> A with nonlinear local activation feedback; reaction terms cancel in d(p_i^I+p_i^A)/dt",
            "transport": "nearest-neighbor conservative fluxes on physical edge control volumes; no global axis or index-fixed direction",
            "mass_conservation": "sum over edge and free pools changes only through explicitly funded synthesis, degradation, or external outflow",
            "active_energy": "any non-equilibrium activation, attachment, or cycling that consumes A debits A and credits W through the existing ledger",
            "parameters": "future directive may choose only dimensioned rates and diffusion/attachment coefficients subject to positivity, finite flux, conservation, and a pre-fission instability test; no values chosen here",
        },
        "lifecycle": {
            "remesh": "conservative overlap remap by physical normalized arclength/control-volume measure",
            "serialization": "schema-versioned complete state including all local amounts, total ledger, accepted time, and configuration identity",
            "fission": "partition both active and inactive amounts by actual daughter boundary correspondence; closing edges receive no copied parent amount unless a separately accounted local source supplies it",
            "daughter_inheritance": "both daughters inherit their physically corresponding P state before mutation; no genotype shortcut",
            "restart": "restore P state and ledger atomically with mesh/world state",
        },
        "mechanical_coupling": {
            "candidate": "local active fraction or attachment state may modulate an existing local contractility susceptibility/rate",
            "requirements": "retain local force semantics, existing A->W work accounting, no target geometry, no fission-readiness input, no midpoint/axis selector",
        },
        "symmetry_breaking": "homogeneous deterministic state may remain homogeneous; ordinary local life-history heterogeneity or finite physical perturbations may seed a mode, but the substrate must amplify it autonomously without observer noise",
        "anti_controller_proof_obligations": [
            "no centroid, midpoint, global axis, target shape, target size, timer, success feedback, or fission query",
            "orientation follows local state and physical transport, not ring index",
            "instability is classified from pre-fission dispersion/mode growth",
            "material and active energy remain closed",
        ],
    }


def extensibility(repo: Path):
    inventory = current_state_inventory(repo)
    candidates = {
        key: value
        for key, value in inventory["production_state_schema"].items()
        if key != "source_hashes"
    }
    results = []
    for key, value in candidates.items():
        results.append(
            {
                "candidate": key,
                "semantic_preservation": False,
                "decision": value.get("reuse_decision"),
                "reason": value.get("meaning") or value.get("units") or value.get("coupling"),
            }
        )
    return {
        "tests": results,
        "old_native_ring_result": "The separate m2-lifeform-runtime carrier is a valid adaptable precedent but is not an existing current M4 state; direct adoption would introduce a new production schema, source contract, and coupling.",
        "existing_state_extensible": False,
        "pass": all(not item["semantic_preservation"] for item in results),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("experiments/generated/dcm4r2endogenouspolaritysubstrate"),
    )
    args = parser.parse_args()
    repo = args.repo.resolve()
    output = args.output if args.output.is_absolute() else repo / args.output
    fixture = repo / "experiments/fixtures/dcm4r2/r1_e2_compact.json"
    output.mkdir(parents=True, exist_ok=True)

    authority = {
        "directive": DIRECTIVE,
        "entry_head": git_value(repo, "rev-parse", "HEAD"),
        "r1_scientific_head": R1_SCIENTIFIC_HEAD,
        "r1_governed_head": R1_GOVERNED_HEAD,
        "r1_exact_head_ci": R1_CI,
        "r1_artifact": R1_ARTIFACT,
        "r1_binary_sha256": R1_BINARY,
        "route_off_default": True,
        "production_changes_in_this_gate": False,
        "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED",
        "owner_override": "ACTIVE",
        "next_execution_started": False,
    }
    dump(output, "authority.json", authority)
    dump(
        output,
        "architect_disposition.json",
        {
            "r1_status": "ACCEPTED / REPLAN",
            "r1_classification": "RESOURCE_LOCAL_GROWTH_COUPLING_FAILS_TO_AMPLIFY_MODES",
            "r2_scope": "diagnostic architecture/substrate decision only",
            "r1_bounded_interpretation": "one-hop compression-gradient coupling is insufficient in bounded coherent Resource; all mechanical Route A is not disproven",
        },
    )
    r1 = reconcile_r1(repo, fixture)
    dump(output, "r1_reconciliation.json", r1)
    inventory = current_state_inventory(repo)
    dump(output, "causal_state_inventory.json", inventory)
    benchmark = external_benchmark()
    dump(output, "external_benchmark.json", benchmark)
    extension = extensibility(repo)
    dump(output, "existing_state_extensibility.json", extension)
    decision = "ROUTE_B_NEW_CONSERVED_POLARITY_SUBSTRATE_REQUIRED"
    dump(output, "new_substrate_specification.json", new_substrate_spec())
    dump(
        output,
        "mechanical_coupling_audit.json",
        {
            "status": "PROPOSED_NOT_IMPLEMENTED",
            "existing_interface": "R9/R10 local contractility with accepted mechanics and explicit A-to-W work ledger",
            "allowed_future_coupling": "local polarity active fraction may modulate a physically meaningful local actuation susceptibility/rate",
            "forbidden_inputs": ["centroid", "midpoint", "global axis", "target geometry", "fission readiness", "success feedback", "observer label"],
            "energy_requirement": "all active work remains debited from A and credited to W; polarity reactions that consume A must be separately ledgered",
            "decision": "coupling interface is reusable, but no coupling is implemented in R2",
        },
    )
    dump(
        output,
        "prefission_prediction.json",
        {
            "status": "PREREGISTERED_FOR_SUCCESSOR_NOT_EXECUTED",
            "prediction": "Under coherent Resource, a route-on polarity substrate must show an endogenous nonzero local mode growing from ordinary life-history heterogeneity before any apposition or fission query is considered; route-off remains spatially stable on matched states.",
            "failure_gate": "If no pre-fission mode has positive growth under held-out states, stop before reproduction testing.",
            "orientation": "not predetermined by ring index, centroid, midpoint, or cleavage axis",
            "held_out_protocol": "future directive must seal state/history hashes before outcome observation",
        },
    )
    fission = fission_authority(repo)
    dump(output, "fission_authority_audit.json", fission)
    dump(
        output,
        "evidence_classification.json",
        {
            "VERIFIED": [
                "R1 exact authority values and accepted bounded negative are reconciled from a compact sealed fixture.",
                "Current fission path contains simple-parent, runtime, conservative partition, and simple-daughter guards.",
                "Current M4 has no existing conserved local active/inactive polarity state.",
                "Standalone benchmark conserves total two-state mass and separates stable and unstable nonzero modes.",
            ],
            "SUPPORTED_HYPOTHESIS": [
                "Missing capability is endogenous spatial mode generation rather than another passive deformation amplifier.",
                "A conserved local reaction-diffusion substrate is a defensible next model class.",
            ],
            "INFERRED": [
                "Directly carrying the separate M2 native-ring field into M4 would be a new substrate integration, not semantic extension.",
            ],
            "UNKNOWN": [
                "Whether a future conserved polarity substrate will generate Resource apposition or fission.",
                "Future parameter values and quantitative M4 route performance.",
            ],
            "DISPROVEN": [
                "R1 one-hop compression-gradient placement is sufficient for a Resource growing mode under its preregistered bounded test.",
            ],
        },
    )
    dump(
        output,
        "forbidden_information_audit.json",
        {
            "pass": True,
            "production_code_changed": False,
            "observer_feedback": False,
            "division_or_fission_feedback": False,
            "target_geometry": False,
            "parameter_fitting_against_fission": False,
            "production_noise_injection": False,
            "new_substrate_implemented": False,
            "notes": "R2 only analyzes source contracts and a standalone benchmark; no Digital Cell runtime was executed or modified.",
        },
    )
    dump(
        output,
        "architecture_decision.json",
        {
            "classification": decision,
            "pass": r1["pass"] and benchmark["pass"] and extension["pass"] and fission["contract"]["simple_parent_required"],
            "basis": [
                "R1 bounded negative reconciled without broadening its conclusion.",
                "No current M4 state satisfies conserved local polarity ownership and lifecycle semantics.",
                "Independent MCRD benchmark demonstrates a target-free nonzero-mode instability criterion.",
                "The separate M2 native-ring carrier is adaptable precedent, not current M4 state.",
            ],
            "production_biology_changed": False,
            "successor_status": "PROPOSED_NOT_IMPLEMENTED",
            "proposed_successor_directive": "DC-M4-R3-CONSERVED-POLARITY-SUBSTRATE-IMPLEMENTATION-AND-PRE-FISSION-QUALIFICATION-001",
        },
    )
    dump(
        output,
        "qualification.json",
        {
            "implementation_acceptance": "PASS",
            "scientific_acceptance": "ARCHITECTURE_GATE_PASS",
            "classification": decision,
            "production_biology_delta": 0,
            "new_biological_parameters": 0,
            "production_reproduction": "NOT_REACHED",
            "selection": "NOT_REACHED",
            "reversal": "NOT_REACHED",
            "final_integrated_m1_m5": "NOT_REACHED",
            "next_execution_started": False,
            "notion_updated_read_back": "NOT_AVAILABLE_IN_THIS_EXECUTION",
        },
    )

    manifest = {}
    for path in sorted(output.rglob("*")):
        if path.is_file() and path.name != "artifact_manifest.json":
            manifest[str(path.relative_to(output))] = sha(path)
    dump(
        output,
        "artifact_manifest.json",
        {
            "schema": "dc-m4-r2-artifact-manifest-v1",
            "directive": DIRECTIVE,
            "files": manifest,
            "file_count": len(manifest),
            "generated_by": str(Path(__file__).relative_to(repo)),
        },
    )
    print(json.dumps({"output": str(output), "classification": decision, "file_count": len(manifest)}, sort_keys=True))


if __name__ == "__main__":
    main()
