#!/usr/bin/env python3
"""Estimate required k_rep from D-006 Stage D aggregate_flow.json."""
from __future__ import annotations

import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AGG = ROOT / "experiments/generated/d006/stage_d/aggregate_flow.json"
OUT = ROOT / "experiments/generated/d007/diagnosis/catalyst_rate_estimate.json"
D006_K_REP = 0.014489097664708522
EPS = 1e-12
BOUND = 3.0 * D006_K_REP


def median(xs: list[float]) -> float:
    return statistics.median(xs) if xs else float("nan")


def iqr(xs: list[float]) -> float:
    if len(xs) < 4:
        return 0.0
    s = sorted(xs)
    n = len(s)
    return s[(3 * n) // 4] - s[n // 4]


def invalid(r: dict) -> bool:
    flags = r.get("invalid_stabilization_flags") or []
    if flags:
        return True
    if r.get("catalyst_retention", 1.0) < 0.05:
        return True
    if r.get("final_catalyst_mass", 1.0) < 1e-3:
        return True
    qc = r.get("Q_C")
    if qc is None or not math.isfinite(qc) or qc <= 0:
        return True
    return False


def main() -> None:
    data = json.loads(AGG.read_text())
    runs = data["runs"]
    valid = []
    rejected = 0
    for r in runs:
        if invalid(r):
            rejected += 1
            continue
        req = D006_K_REP / max(float(r["Q_C"]), EPS)
        valid.append({**r, "required_k_rep": req})

    vals = [v["required_k_rep"] for v in valid]
    by_r = defaultdict(list)
    by_c = defaultdict(list)
    by_cand = defaultdict(list)
    for v in valid:
        by_r[v["R0"]].append(v["required_k_rep"])
        by_c[v["C0"]].append(v["required_k_rep"])
        by_cand[v["candidate_id"]].append(v["required_k_rep"])

    med = median(vals)
    outside = bool(math.isfinite(med) and med > BOUND)
    center = BOUND if outside else med
    out = {
        "source": str(AGG),
        "current_k_rep": D006_K_REP,
        "n_input": len(runs),
        "n_valid": len(valid),
        "n_rejected": rejected,
        "median_required_k_rep": med,
        "iqr_required_k_rep": iqr(vals),
        "min_required_k_rep": min(vals) if vals else None,
        "max_required_k_rep": max(vals) if vals else None,
        "median_by_R0": {str(k): median(vs) for k, vs in sorted(by_r.items())},
        "median_by_C0": {str(k): median(vs) for k, vs in sorted(by_c.items())},
        "median_by_structural_candidate": {
            k: median(vs) for k, vs in sorted(by_cand.items())
        },
        "k_rep_center": med,
        "k_rep_center_clamped": center,
        "bounded_max": BOUND,
        "outside_bounded_range": outside,
        "classification": (
            "D007_CATALYST_RATE_OUTSIDE_BOUNDED_RANGE"
            if outside
            else "D007_CATALYST_RATE_WITHIN_BOUNDED_RANGE"
        ),
        "epsilon": EPS,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(out, indent=2) + "\n")
    print(json.dumps({k: out[k] for k in (
        "n_valid", "median_required_k_rep", "k_rep_center_clamped",
        "outside_bounded_range", "classification"
    )}, indent=2))


if __name__ == "__main__":
    main()
