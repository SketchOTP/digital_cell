#!/usr/bin/env python3
"""D-007 Stage J1/J2 joint screens + nullcline classification."""
from __future__ import annotations

import argparse
import json
import os
import statistics
import subprocess
import time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target/release/experiment-runner"
D007 = ROOT / "experiments/generated/d007"
RADII = [18.0, 24.0, 30.0]
CATS = [0.275, 0.35, 0.425]
J1_SEED = 2
J2_SEEDS = [1, 3]
STEPS = 50_000


def run_cmd(args: list[str]) -> None:
    env = os.environ.copy()
    env["PATH"] = str(Path.home() / ".cargo/bin") + ":" + env.get("PATH", "")
    subprocess.check_call(args, cwd=ROOT, env=env)


def load_joint_candidates() -> list[dict]:
    cat = json.loads((D007 / "catalyst_bracket/aggregate.json").read_text())
    structs = cat.get("retained_structural_factors") or []
    rates = cat.get("retained_k_rep") or []
    # from configs — at most 9
    out = []
    root = ROOT / "configs/d007/joint_candidates"
    for d in sorted(root.iterdir()):
        meta = json.loads((d / "meta.json").read_text())
        if meta["structural_factor"] in structs and any(
            abs(meta["k_rep"] - k) < 1e-12 for k in rates
        ):
            out.append({**meta, "identity": str(d / "identity.json")})
    return out[:9]


def job_list(stage: str, candidates: list[dict]) -> list[dict]:
    seeds = [J1_SEED] if stage == "j1" else J2_SEEDS
    jobs = []
    for c in candidates:
        for r0 in RADII:
            for c0 in CATS:
                for seed in seeds:
                    out = (
                        D007
                        / f"joint_screen_{stage}"
                        / "runs"
                        / f"{c['candidate_id']}_R{int(r0)}_C{int(c0*1000)}_s{seed}"
                    )
                    jobs.append(
                        {
                            "identity": c["identity"],
                            "candidate_id": c["candidate_id"],
                            "r0": r0,
                            "c0": c0,
                            "seed": seed,
                            "output": str(out),
                            "result": str(out / "result.json"),
                        }
                    )
    return jobs


def launch_one(job: dict) -> dict:
    if Path(job["result"]).exists():
        return {"status": "cached", **job}
    cmd = [
        str(BIN),
        "d007",
        "run-one",
        "--identity",
        job["identity"],
        "--r0",
        str(job["r0"]),
        "--c0",
        str(job["c0"]),
        "--seed",
        str(job["seed"]),
        "--steps",
        str(STEPS),
        "--output",
        job["output"],
    ]
    try:
        run_cmd(cmd)
        return {"status": "ok", **job}
    except subprocess.CalledProcessError as e:
        return {"status": "fail", "error": str(e), **job}


def sign(x: float) -> int:
    if x > 1e-12:
        return 1
    if x < -1e-12:
        return -1
    return 0


def analyze_candidate(cid: str, stage: str) -> dict:
    runs = []
    root = D007 / f"joint_screen_{stage}" / "runs"
    for p in root.glob(f"{cid}_*/result.json"):
        rec = json.loads(p.read_text())
        if not rec.get("clean_termination"):
            continue
        runs.append(rec)
    # grid medians by (R0,C0)
    g = defaultdict(list)
    for r in runs:
        g[(float(r["R0"]), float(r["C0"]))].append(r)
    points = []
    for (r0, c0), rs in sorted(g.items()):
        vr = statistics.median([float(x["v_R"]) for x in rs])
        vc = statistics.median([float(x["v_C_inside"]) for x in rs])
        points.append({"r0": r0, "c0": c0, "v_R": vr, "v_C_inside": vc, "n": len(rs)})

    # radius nullcline: sign change across R at fixed C
    radius_nc = False
    for c0 in CATS:
        pts = sorted([p for p in points if abs(p["c0"] - c0) < 1e-9], key=lambda p: p["r0"])
        signs = [sign(p["v_R"]) for p in pts if sign(p["v_R"]) != 0]
        if len(signs) >= 2 and any(signs[i] != signs[i + 1] for i in range(len(signs) - 1)):
            radius_nc = True
    catalyst_nc = False
    for r0 in RADII:
        pts = sorted([p for p in points if abs(p["r0"] - r0) < 1e-9], key=lambda p: p["c0"])
        signs = [sign(p["v_C_inside"]) for p in pts if sign(p["v_C_inside"]) != 0]
        if len(signs) >= 2 and any(signs[i] != signs[i + 1] for i in range(len(signs) - 1)):
            catalyst_nc = True

    # crude intersection: cell with opposing corner signs for both fields
    intersect = False
    ru = sorted({p["r0"] for p in points})
    cu = sorted({p["c0"] for p in points})
    lookup = {(p["r0"], p["c0"]): p for p in points}
    for i in range(len(ru) - 1):
        for j in range(len(cu) - 1):
            corners = [
                lookup.get((ru[i], cu[j])),
                lookup.get((ru[i + 1], cu[j])),
                lookup.get((ru[i], cu[j + 1])),
                lookup.get((ru[i + 1], cu[j + 1])),
            ]
            if any(c is None for c in corners):
                continue
            vr = [sign(c["v_R"]) for c in corners]
            vc = [sign(c["v_C_inside"]) for c in corners]
            if len(set(vr) - {0}) >= 2 and len(set(vc) - {0}) >= 2:
                intersect = True

    if not radius_nc:
        cls = "NO_RADIUS_NULLCLINE"
    elif not catalyst_nc:
        cls = "NO_CATALYST_NULLCLINE"
    elif not intersect:
        cls = "NULLCLINES_DISJOINT"
    else:
        # without dense Jacobian samples mark narrow if only one cell
        cls = "STABLE_INTERSECTION_NARROW"

    return {
        "candidate_id": cid,
        "points": points,
        "radius_nullcline": radius_nc,
        "catalyst_nullcline": catalyst_nc,
        "intersection": intersect,
        "class": cls,
        "n_runs": len(runs),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["run-j1", "run-j2", "analyze", "status"])
    ap.add_argument("--jobs", type=int, default=8)
    args = ap.parse_args()
    cands = load_joint_candidates()
    if args.cmd == "status":
        for stage in ("j1", "j2"):
            js = job_list(stage, cands)
            done = sum(1 for j in js if Path(j["result"]).exists())
            print(stage, done, "/", len(js))
        return
    if args.cmd == "analyze":
        outs = [analyze_candidate(c["candidate_id"], "j1") for c in cands]
        adv = [o for o in outs if o["class"] in (
            "STABLE_INTERSECTION_NARROW", "STABLE_INTERSECTION_ROBUST"
        )][:3]
        dest = D007 / "nullclines" / "j1_summary.json"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(json.dumps({"candidates": outs, "advance": adv}, indent=2) + "\n")
        print(json.dumps({"n": len(outs), "advance": [a["candidate_id"] for a in adv]}))
        return
    stage = "j1" if args.cmd == "run-j1" else "j2"
    if stage == "j2":
        # only advanced from j1
        j1 = json.loads((D007 / "nullclines/j1_summary.json").read_text())
        ids = {a["candidate_id"] for a in j1.get("advance", [])}
        cands = [c for c in cands if c["candidate_id"] in ids]
    pending = [j for j in job_list(stage, cands) if not Path(j["result"]).exists()]
    print(f"launching {len(pending)} {stage} jobs")
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = [ex.submit(launch_one, j) for j in pending]
        for fut in as_completed(futs):
            r = fut.result()
            print(r["status"], Path(r["output"]).name)


if __name__ == "__main__":
    main()
