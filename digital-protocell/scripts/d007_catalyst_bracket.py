#!/usr/bin/env python3
"""Catalyst-rate bracket around k_rep_center for retained structural factors."""
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
D006_K_REP = 0.014489097664708522
R0 = 24.0
C0S = [0.275, 0.35, 0.425]
SEEDS = [1, 2, 3]
STEPS = 30_000
CAT_FACTORS = [0.75, 0.875, 1.0, 1.125, 1.25]


def run_cmd(args: list[str]) -> None:
    env = os.environ.copy()
    env["PATH"] = str(Path.home() / ".cargo/bin") + ":" + env.get("PATH", "")
    subprocess.check_call(args, cwd=ROOT, env=env)


def load_center() -> float:
    est = json.loads((D007 / "diagnosis/catalyst_rate_estimate.json").read_text())
    return float(est["k_rep_center_clamped"])


def load_structural_factors() -> list[float]:
    agg = json.loads((D007 / "structural_bracket/aggregate.json").read_text())
    fac = agg.get("retained_structural_factors") or []
    if not fac:
        # approach: nearest three to any disordered/restore boundary
        facs = [r["factor"] for r in agg["factors"]]
        fac = facs[:3]
    return [float(x) for x in fac]


def clamp_k(k: float) -> float:
    return max(0.75 * D006_K_REP, min(3.0 * D006_K_REP, k))


def prepare() -> dict:
    center = load_center()
    structs = load_structural_factors()
    rates = [clamp_k(center * f) for f in CAT_FACTORS]
    # unique rates
    uniq = []
    for k in rates:
        if not any(abs(k - u) < 1e-15 for u in uniq):
            uniq.append(k)
    candidates = []
    for sf in structs:
        for kf, k_rep in zip(CAT_FACTORS, rates):
            run_cmd(
                [
                    str(BIN),
                    "d007",
                    "write-joint-candidate",
                    "--structural-factor",
                    str(sf),
                    "--k-rep",
                    str(k_rep),
                    "--catalyst-factor",
                    str(kf),
                    "--parent",
                    "d006-1.0x-reference",
                ]
            )
    # index from configs
    joint_root = ROOT / "configs/d007/joint_candidates"
    for d in joint_root.iterdir():
        meta = json.loads((d / "meta.json").read_text())
        if meta["structural_factor"] in structs and meta["k_rep"] in uniq:
            candidates.append(
                {
                    **meta,
                    "identity": str(d / "identity.json"),
                }
            )
    idx = {
        "k_rep_center": center,
        "structural_factors": structs,
        "k_rep_values": uniq,
        "candidates": candidates,
    }
    dest = D007 / "catalyst_bracket/index.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(idx, indent=2) + "\n")
    return idx


def jobs(idx: dict) -> list[dict]:
    out = []
    for c in idx["candidates"]:
        for c0 in C0S:
            for seed in SEEDS:
                out_dir = (
                    D007
                    / "catalyst_bracket"
                    / "runs"
                    / f"{c['candidate_id']}_R{int(R0)}_C{int(c0*1000)}_s{seed}"
                )
                out.append(
                    {
                        "identity": c["identity"],
                        "candidate_id": c["candidate_id"],
                        "structural_factor": c["structural_factor"],
                        "k_rep": c["k_rep"],
                        "r0": R0,
                        "c0": c0,
                        "seed": seed,
                        "output": str(out_dir),
                        "result": str(out_dir / "result.json"),
                    }
                )
    return out


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
    t0 = time.time()
    try:
        run_cmd(cmd)
        return {"status": "ok", "wall": time.time() - t0, **job}
    except subprocess.CalledProcessError as e:
        return {"status": "fail", "error": str(e), **job}


def classify(v_low: float, v_mid: float, v_high: float) -> str:
    if v_low < 0 and v_mid < 0 and v_high < 0:
        return "ALL_DECLINE"
    if v_low > 0 and v_mid > 0 and v_high > 0:
        return "ALL_GROW"
    if v_low > 0 and v_high < 0:
        return "RESTORING_CROSSING"
    return "DISORDERED"


def aggregate(idx: dict) -> dict:
    by = defaultdict(list)
    for j in jobs(idx):
        p = Path(j["result"])
        if not p.exists():
            continue
        rec = json.loads(p.read_text())
        if not rec.get("clean_termination"):
            continue
        by[(j["structural_factor"], j["k_rep"], j["c0"])].append(rec)

    rows = []
    classes = []
    for (sf, k_rep, c0), recs in sorted(by.items()):
        vc = [float(r["v_C_inside"]) for r in recs]
        ret = [float(r["retention"]) for r in recs]
        qc = [float(r["Q_C"]) for r in recs]
        rows.append(
            {
                "structural_factor": sf,
                "k_rep": k_rep,
                "C0": c0,
                "median_v_C_inside": statistics.median(vc),
                "median_Q_C": statistics.median(qc),
                "median_retention": statistics.median(ret),
                "n": len(recs),
            }
        )

    # per (sf,k_rep) across C0
    rate_eval = []
    for sf in idx["structural_factors"]:
        for k_rep in idx["k_rep_values"]:
            pts = [r for r in rows if r["structural_factor"] == sf and abs(r["k_rep"] - k_rep) < 1e-15]
            if len(pts) < 3:
                cls = "INCOMPLETE"
                provisional = False
                med_by_c = {}
            else:
                pts = sorted(pts, key=lambda x: x["C0"])
                v_low, v_mid, v_high = [p["median_v_C_inside"] for p in pts]
                cls = classify(v_low, v_mid, v_high)
                classes.append(cls)
                med_ret = statistics.median([p["median_retention"] for p in pts])
                provisional = (
                    cls == "RESTORING_CROSSING" and med_ret >= 0.80
                )
                med_by_c = {p["C0"]: p["median_v_C_inside"] for p in pts}
            rate_eval.append(
                {
                    "structural_factor": sf,
                    "k_rep": k_rep,
                    "class": cls,
                    "provisional_pass": provisional,
                    "median_v_C_by_C0": med_by_c,
                }
            )

    if classes and all(c == "ALL_DECLINE" for c in classes):
        gate = "D007_NO_CATALYST_NULLCLINE"
    elif classes and all(c == "ALL_GROW" for c in classes):
        gate = "D007_UNBOUNDED_CATALYST"
    elif any(r["provisional_pass"] for r in rate_eval):
        gate = "CONTINUE"
    elif any(c == "INCOMPLETE" for c in [r["class"] for r in rate_eval]):
        gate = "INCOMPLETE"
    else:
        gate = "D007_NO_CATALYST_NULLCLINE"

    # retain ≤3 neighboring catalyst rates across any struct pass
    pass_rates = sorted(
        {r["k_rep"] for r in rate_eval if r["provisional_pass"]}
    )
    retained_rates = pass_rates[:3]
    retained_structs = sorted(
        {r["structural_factor"] for r in rate_eval if r["provisional_pass"]}
    )[:3]

    out = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "rows": rows,
        "rate_evaluations": rate_eval,
        "catalyst_gate": gate,
        "retained_k_rep": retained_rates,
        "retained_structural_factors": retained_structs,
    }
    (D007 / "catalyst_bracket/aggregate.json").write_text(json.dumps(out, indent=2) + "\n")
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["prepare", "run", "aggregate", "status"])
    ap.add_argument("--jobs", type=int, default=8)
    args = ap.parse_args()
    if args.cmd == "prepare":
        idx = prepare()
        print(json.dumps({"candidates": len(idx["candidates"]), "jobs": len(jobs(idx))}))
        return
    idx = json.loads((D007 / "catalyst_bracket/index.json").read_text())
    if args.cmd == "status":
        js = jobs(idx)
        done = sum(1 for j in js if Path(j["result"]).exists())
        print(json.dumps({"done": done, "total": len(js)}))
        return
    if args.cmd == "aggregate":
        print(json.dumps(aggregate(idx), indent=2)[:2500])
        return
    pending = [j for j in jobs(idx) if not Path(j["result"]).exists()]
    print(f"launching {len(pending)} catalyst jobs")
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = [ex.submit(launch_one, j) for j in pending]
        for fut in as_completed(futs):
            r = fut.result()
            print(r["status"], Path(r["output"]).name)
    print(json.dumps({"gate": aggregate(idx)["catalyst_gate"]}))


if __name__ == "__main__":
    main()
