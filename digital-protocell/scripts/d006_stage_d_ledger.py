#!/usr/bin/env python3
"""D-006C Stage D job ledger + radius/catalyst gate helpers (offline, post-artifact)."""
from __future__ import annotations

import argparse
import glob
import json
import re
import statistics
import subprocess
import time
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
D006 = ROOT / "experiments/generated/d006"
RADII = [16.0, 20.0, 24.0, 28.0, 32.0]
CATS = [0.275, 0.35, 0.425]
SEEDS = [1, 2, 3]
BASIC = [
    "candidate_id",
    "candidate_hash",
    "configuration_hash",
    "equation_version",
    "r0",
    "c0",
    "noise_seed",
    "accepted_substeps",
    "simulated_time",
    "radial_velocity",
    "q_phi",
    "q_c",
    "classification",
]
STRICT = [
    "initial_field_hashes",
    "final_field_hashes",
    "accounting",
    "clean_termination",
]


def load_candidates() -> tuple[list[dict], list[dict], float]:
    idx = json.loads((D006 / "candidates/index.json").read_text())
    k0 = json.loads((D006 / "planar_interface/calibration.json").read_text())[
        "k_structure_interface_initial"
    ]
    surviving, rejected = [], []
    for e in idx:
        cid = e["candidate_id"]
        pr = json.loads((D006 / "prescribed_radius" / cid / "result.json").read_text())
        fac = round(e["k_structure_interface"] / k0, 2)
        rec = {
            "candidate_id": cid,
            "candidate_hash": e["candidate_hash"],
            "configuration_hash": e["configuration_hash"],
            "k": e["k_structure_interface"],
            "factor": fac,
            "crossing": pr["has_stable_crossing"],
            "equation_version": e.get("equation_version", "surface_turnover_v1"),
        }
        (surviving if pr["has_stable_crossing"] else rejected).append(rec)
    return surviving, rejected, k0


def running_jobs() -> dict[tuple, dict]:
    ps = subprocess.check_output(["ps", "-eo", "pid=,args="], text=True)
    out: dict[tuple, dict] = {}
    for line in ps.splitlines():
        if "experiment-runner d006 run-one" not in line:
            continue
        m = re.search(
            r"^\s*(\d+)\s+.*--candidate-id\s+(\S+)\s+--r0\s+(\S+)\s+--c0\s+(\S+)\s+--seed\s+(\S+)",
            line,
        )
        if not m:
            continue
        try:
            pid, cid, r0, c0, seed = m.groups()
            key = (cid, float(r0), float(c0), int(float(seed)))
            out[key] = {"pid": int(pid)}
        except ValueError:
            continue
    return out


def result_map() -> dict[tuple, dict]:
    results: dict[tuple, dict] = {}
    for p in glob.glob(str(D006 / "candidate_screen/*/R*_C*_s*/result.json")):
        d = json.loads(Path(p).read_text())
        key = (d["candidate_id"], float(d["r0"]), float(d["c0"]), int(d["noise_seed"]))
        missing = [f for f in BASIC if f not in d]
        strict_missing = [f for f in STRICT if f not in d]
        steps = d.get("accepted_substeps")
        steps_ok = isinstance(steps, (int, float)) and steps >= 50000
        if missing or not steps_ok:
            status = "COMPLETE_INVALID"
        elif strict_missing:
            # flow-usable; §6 strict gap
            status = "COMPLETE_INVALID"
        else:
            status = "COMPLETE_VALID"
        results[key] = {
            "path": p,
            "status": status,
            "missing": missing,
            "strict_missing": strict_missing,
            "accepted_substeps": steps,
            "usable_for_flow_analysis": not missing and steps_ok,
            "record": d,
        }
    return results


def build_ledger() -> dict[str, Any]:
    surviving, rejected, k0 = load_candidates()
    running = running_jobs()
    results = result_map()
    ledger = []
    for s in surviving:
        for r0 in RADII:
            for c0 in CATS:
                for seed in SEEDS:
                    key = (s["candidate_id"], r0, c0, seed)
                    if key in running:
                        st = "RUNNING"
                    elif key in results:
                        st = results[key]["status"]
                    else:
                        st = "PENDING"
                    ledger.append(
                        {
                            "equation_version": "surface_turnover_v1",
                            "candidate_id": s["candidate_id"],
                            "candidate_hash": s["candidate_hash"],
                            "configuration_hash": s["configuration_hash"],
                            "interface_rate_factor": s["factor"],
                            "k_structure_interface": s["k"],
                            "R0": r0,
                            "C0": c0,
                            "noise_seed": seed,
                            "noise_amplitude": 0.005,
                            "accepted_substep_target": 50000,
                            "status": st,
                            "pid": running.get(key, {}).get("pid"),
                            "result_path": results.get(key, {}).get("path"),
                            "usable_for_flow_analysis": results.get(key, {}).get(
                                "usable_for_flow_analysis"
                            ),
                        }
                    )
    counts = Counter(j["status"] for j in ledger)
    out = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "execution_status": "D006_RESULT_UNRESOLVED_STAGE_D_IN_PROGRESS",
        "theoretical_5x5x3x3": 225,
        "scheduled_jobs": 180,
        "scheduling_explanation": (
            "0.60× prescribed-radius candidate has_stable_crossing=false; "
            "Stage D scheduled 4 survivors × 5 radii × 3 C0 × 3 seeds = 180"
        ),
        "k_structure_interface_initial": k0,
        "rejected_prescribed": rejected,
        "surviving_candidates": surviving,
        "status_counts": dict(counts),
        "running_count": len(running),
        "strict_field_gap": True,
        "jobs": ledger,
    }
    dest = D006 / "stage_d/job_ledger.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(out, indent=2))
    slim = {k: v for k, v in out.items() if k != "jobs"}
    (D006 / "stage_d/job_ledger_summary.json").write_text(json.dumps(slim, indent=2))
    return out


def invalid_stabilization_flags(rec: dict) -> list[str]:
    flags = []
    ret = float(rec.get("retention", 1.0))
    q_phi = float(rec.get("q_phi", 1.0))
    final_r = float(rec.get("final_radius", 1.0))
    final_mc = float(rec.get("final_m_c", 1.0))
    classif = str(rec.get("classification", ""))
    if ret < 0.05 or final_mc < 1e-3:
        flags.append("CATALYST_EXTINCTION_STALL")
    if q_phi < 0.15 and float(rec.get("radial_velocity", 0.0)) > -1e-6:
        flags.append("COLLAPSE_STALL")
    if final_r <= 1.0:
        flags.append("COLLAPSE_STALL")
    if "Fragment" in classif:
        flags.append("FRAGMENTATION_STALL")
    if "Clip" in classif or "Numerical" in classif:
        flags.append("NUMERICAL_STALL")
    if final_r >= 63.0:  # dish half-width heuristic for 128 grid
        flags.append("DISH_BOUNDARY_STALL")
    return sorted(set(flags))


def ordered_restoring_crossing(median_by_r: list[tuple[float, float]]) -> bool:
    """Single ordered +→− transition by radius; rejects random sign flips."""
    pts = sorted(median_by_r, key=lambda x: x[0])
    if len(pts) < 2:
        return False
    saw_pos = saw_neg = crossed = False
    for _r, v in pts:
        if v > 0:
            if crossed or saw_neg:
                return False
            saw_pos = True
        elif v < 0:
            if not saw_pos:
                return False
            saw_neg = True
            crossed = True
    return saw_pos and saw_neg and crossed


def seed_agreement(vs: list[float], require: int = 2) -> bool:
    if len(vs) < require:
        return False
    pos = sum(1 for v in vs if v > 0)
    neg = sum(1 for v in vs if v < 0)
    return max(pos, neg) >= require


def aggregate_flow(usable_only: bool = True) -> dict[str, Any]:
    results = result_map()
    rows = []
    for key, meta in results.items():
        if usable_only and not meta.get("usable_for_flow_analysis"):
            continue
        d = meta["record"]
        flags = invalid_stabilization_flags(d)
        dt = float(d["simulated_time"])
        # mean internal C proxy: mass / (pi R^2)
        r_i = float(d["initial_radius"])
        r_f = float(d["final_radius"])
        c_i = float(d["initial_m_c"]) / max(3.141592653589793 * r_i * r_i, 1e-12)
        c_f = float(d["final_m_c"]) / max(3.141592653589793 * r_f * r_f, 1e-12)
        rows.append(
            {
                "candidate_id": d["candidate_id"],
                "candidate_hash": d["candidate_hash"],
                "configuration_hash": d["configuration_hash"],
                "equation_version": d["equation_version"],
                "R0": float(d["r0"]),
                "C0": float(d["c0"]),
                "noise_seed": int(d["noise_seed"]),
                "initial_equivalent_radius": r_i,
                "final_equivalent_radius": r_f,
                "initial_structural_mass": float(d["initial_m_phi"]),
                "final_structural_mass": float(d["final_m_phi"]),
                "initial_catalyst_mass": float(d["initial_m_c"]),
                "final_catalyst_mass": float(d["final_m_c"]),
                "radial_velocity": float(d["radial_velocity"]),
                "catalyst_mass_velocity": float(d["catalyst_velocity"]),
                "v_C_inside": (c_f - c_i) / max(dt, 1e-12),
                "C_inside_initial": c_i,
                "C_inside_final": c_f,
                "Q_phi": float(d["q_phi"]),
                "Q_C": float(d["q_c"]),
                "slope_phi": float(d["slope_phi"]),
                "slope_C": float(d["slope_c"]),
                "catalyst_retention": float(d["retention"]),
                "simulated_time": dt,
                "accepted_substeps": d["accepted_substeps"],
                "classification": d["classification"],
                "invalid_stabilization_flags": flags,
                "valid_for_restoring": len(flags) == 0,
            }
        )
    # by (candidate, C0, R0)
    grouped: dict[tuple, list] = defaultdict(list)
    for r in rows:
        grouped[(r["candidate_id"], r["C0"], r["R0"])].append(r)

    radius_table = []
    for (cid, c0, r0), grp in sorted(grouped.items()):
        valid = [g for g in grp if g["valid_for_restoring"]]
        vs = [g["radial_velocity"] for g in valid]
        fails = Counter(
            f for g in grp for f in (g["invalid_stabilization_flags"] or ["none"])
        )
        radius_table.append(
            {
                "candidate": cid,
                "C0": c0,
                "R0": r0,
                "median_radial_velocity": statistics.median(vs) if vs else None,
                "minimum_radial_velocity": min(vs) if vs else None,
                "maximum_radial_velocity": max(vs) if vs else None,
                "successful_replicates": len(vs),
                "seed_count": len(grp),
                "seed_agreement": seed_agreement(vs) if vs else False,
                "failure_classifications": dict(fails),
            }
        )

    # candidate gate provisional
    surviving, _, _ = load_candidates()
    cand_eval = []
    for s in surviving:
        cid = s["candidate_id"]
        by_c0 = defaultdict(list)
        for row in radius_table:
            if row["candidate"] != cid or row["median_radial_velocity"] is None:
                continue
            by_c0[row["C0"]].append((row["R0"], row["median_radial_velocity"], row))
        crossings = []
        for c0, pts in by_c0.items():
            meds = [(r, v) for r, v, _ in pts]
            ok = ordered_restoring_crossing(meds)
            agree = all(meta["seed_agreement"] for _r, _v, meta in pts if meta["successful_replicates"] >= 2)
            crossings.append(
                {
                    "C0": c0,
                    "ordered_crossing": ok,
                    "seed_agreement_all_radii": agree,
                    "median_v_R_by_R0": sorted(
                        [{"R0": r, "median_v_R": v} for r, v, _ in pts],
                        key=lambda x: x["R0"],
                    ),
                }
            )
        # catalyst: retention medians
        cret = [
            r["catalyst_retention"]
            for r in rows
            if r["candidate_id"] == cid and r["valid_for_restoring"]
        ]
        vc = [
            r["v_C_inside"]
            for r in rows
            if r["candidate_id"] == cid and r["valid_for_restoring"]
        ]
        cand_eval.append(
            {
                "candidate_id": cid,
                "factor": s["factor"],
                "crossings_by_C0": crossings,
                "any_ordered_crossing": any(c["ordered_crossing"] for c in crossings),
                "median_retention": statistics.median(cret) if cret else None,
                "median_v_C_inside": statistics.median(vc) if vc else None,
                "n_usable_runs": sum(1 for r in rows if r["candidate_id"] == cid),
            }
        )

    out = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "n_runs": len(rows),
        "runs": rows,
        "radius_flow_table": radius_table,
        "candidate_evaluations": cand_eval,
    }
    dest = D006 / "stage_d/aggregate_flow.json"
    dest.write_text(json.dumps(out, indent=2))
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["ledger", "aggregate", "status"])
    args = ap.parse_args()
    if args.cmd == "ledger":
        out = build_ledger()
        print(json.dumps(out["status_counts"]))
    elif args.cmd == "aggregate":
        out = aggregate_flow()
        print(json.dumps({"n_runs": out["n_runs"], "candidates": len(out["candidate_evaluations"])}))
    else:
        out = build_ledger()
        print(json.dumps({k: out[k] for k in ("status_counts", "running_count", "scheduled_jobs")}))


if __name__ == "__main__":
    main()
