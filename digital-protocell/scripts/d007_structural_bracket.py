#!/usr/bin/env python3
"""Launch / aggregate D-007 structural-rate bracket (7×3×3 = 63 runs)."""
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
FACTORS = [0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80]
RADII = [16.0, 24.0, 32.0]
SEEDS = [1, 2, 3]
C0 = 0.35
STEPS = 30_000


def run_cmd(args: list[str]) -> None:
    env = os.environ.copy()
    env["PATH"] = str(Path.home() / ".cargo/bin") + ":" + env.get("PATH", "")
    subprocess.check_call(args, cwd=ROOT, env=env)


def ensure_binary() -> None:
    if not BIN.exists():
        run_cmd(["cargo", "build", "-p", "experiment-runner", "--release"])


def write_candidates() -> dict[float, dict]:
    out = {}
    for fac in FACTORS:
        run_cmd([str(BIN), "d007", "write-structural-candidate", "--factor", str(fac)])
        cand_root = D007 / "structural_bracket" / "candidates"
        # find newest matching factor meta
        hit = None
        for d in cand_root.iterdir():
            meta = json.loads((d / "meta.json").read_text())
            if abs(meta["structural_factor"] - fac) < 1e-12:
                hit = meta
                hit["identity_path"] = str(d / "identity.json")
                hit["dir"] = str(d)
                break
        if hit is None:
            raise RuntimeError(f"missing candidate for factor {fac}")
        out[fac] = hit
    (D007 / "structural_bracket" / "candidates_index.json").write_text(
        json.dumps(out, indent=2) + "\n"
    )
    return out


def job_spec(cands: dict[float, dict]) -> list[dict]:
    jobs = []
    for fac, meta in cands.items():
        for r0 in RADII:
            for seed in SEEDS:
                out = (
                    D007
                    / "structural_bracket"
                    / "runs"
                    / f"fac{fac:.2f}_R{int(r0)}_C{int(C0*1000)}_s{seed}"
                )
                jobs.append(
                    {
                        "factor": fac,
                        "r0": r0,
                        "c0": C0,
                        "seed": seed,
                        "identity": meta["identity_path"],
                        "output": str(out),
                        "result": str(out / "result.json"),
                    }
                )
    return jobs


def launch_one(job: dict) -> dict:
    result = Path(job["result"])
    if result.exists():
        rec = json.loads(result.read_text())
        if rec.get("clean_termination") and "v_R" in rec:
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
        return {"status": "fail", "error": str(e), "wall": time.time() - t0, **job}


def classify(v16: float, v24: float, v32: float) -> str:
    if v16 < 0 and v24 < 0 and v32 < 0:
        return "ALL_DECLINE"
    if v16 > 0 and v24 > 0 and v32 > 0:
        return "ALL_GROW"
    if v16 > 0 and v32 < 0 and v16 >= v24 - 1e-9 and v24 >= v32 - 1e-9:
        return "RESTORING_CROSSING"
    return "DISORDERED"


def aggregate(cands: dict[float, dict]) -> dict:
    by_fac_r: dict[tuple, list[float]] = defaultdict(list)
    rows = []
    for job in job_spec(cands):
        p = Path(job["result"])
        if not p.exists():
            continue
        rec = json.loads(p.read_text())
        if not rec.get("clean_termination"):
            continue
        rows.append(rec)
        by_fac_r[(job["factor"], job["r0"])].append(float(rec["v_R"]))

    factor_rows = []
    classes = []
    for fac in FACTORS:
        meds = {}
        for r0 in RADII:
            vs = by_fac_r.get((fac, r0), [])
            meds[r0] = statistics.median(vs) if vs else None
        if None in meds.values():
            cls = "INCOMPLETE"
        else:
            cls = classify(meds[16.0], meds[24.0], meds[32.0])
        classes.append(cls)
        provisional = (
            cls == "RESTORING_CROSSING"
            and meds[16.0] is not None
            and meds[16.0] > 0
            and meds[32.0] is not None
            and meds[32.0] < 0
        )
        factor_rows.append(
            {
                "factor": fac,
                "median_v_R_R16": meds[16.0],
                "median_v_R_R24": meds[24.0],
                "median_v_R_R32": meds[32.0],
                "class": cls,
                "provisional_pass": provisional,
                "candidate_id": cands[fac]["candidate_id"],
            }
        )

    if all(c == "ALL_DECLINE" for c in classes if c != "INCOMPLETE") and classes:
        gate = "D007_NO_STRUCTURAL_NULLCLINE"
    elif all(c == "ALL_GROW" for c in classes if c != "INCOMPLETE") and "INCOMPLETE" not in classes:
        gate = "D007_NO_STRUCTURAL_NULLCLINE"
    elif any(r["provisional_pass"] for r in factor_rows):
        gate = "CONTINUE"
    elif "INCOMPLETE" in classes:
        gate = "INCOMPLETE"
    else:
        # No ordered restoring crossing within bounded factors (includes DISORDERED / reverse flips).
        gate = "D007_NO_STRUCTURAL_NULLCLINE"

    # retain ≤3 neighboring provisional (or nearest approaching)
    passes = [i for i, r in enumerate(factor_rows) if r["provisional_pass"]]
    retained = []
    if passes:
        idxs = set()
        for i in passes:
            idxs.update({i, max(0, i - 1), min(len(FACTORS) - 1, i + 1)})
        retained = [FACTORS[i] for i in sorted(idxs)[:3]]
    out = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "n_results": len(rows),
        "n_jobs": len(job_spec(cands)),
        "factors": factor_rows,
        "structural_gate": gate,
        "retained_structural_factors": retained,
        "steps": STEPS,
        "c0": C0,
    }
    dest = D007 / "structural_bracket" / "aggregate.json"
    dest.write_text(json.dumps(out, indent=2) + "\n")
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["prepare", "run", "aggregate", "status"])
    ap.add_argument("--jobs", type=int, default=8)
    args = ap.parse_args()
    ensure_binary()
    D007.mkdir(parents=True, exist_ok=True)
    (D007 / "structural_bracket").mkdir(parents=True, exist_ok=True)

    if args.cmd == "prepare":
        cands = write_candidates()
        jobs = job_spec(cands)
        print(json.dumps({"candidates": len(cands), "jobs": len(jobs)}))
        return

    cands_path = D007 / "structural_bracket" / "candidates_index.json"
    if not cands_path.exists():
        cands = write_candidates()
    else:
        raw = json.loads(cands_path.read_text())
        cands = {float(k): v for k, v in raw.items()}

    if args.cmd == "status":
        jobs = job_spec(cands)
        done = sum(1 for j in jobs if Path(j["result"]).exists())
        print(json.dumps({"done": done, "total": len(jobs)}))
        return

    if args.cmd == "aggregate":
        out = aggregate(cands)
        print(json.dumps({
            "structural_gate": out["structural_gate"],
            "n_results": out["n_results"],
            "retained": out["retained_structural_factors"],
            "classes": [(f["factor"], f["class"]) for f in out["factors"]],
        }, indent=2))
        return

    # run
    jobs = job_spec(cands)
    pending = [j for j in jobs if not Path(j["result"]).exists()]
    print(f"launching {len(pending)}/{len(jobs)} structural jobs with {args.jobs} workers")
    oks = fails = 0
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = [ex.submit(launch_one, j) for j in pending]
        for fut in as_completed(futs):
            r = fut.result()
            if r["status"] in ("ok", "cached"):
                oks += 1
            else:
                fails += 1
            print(r["status"], Path(r["output"]).name, r.get("wall"))
    agg = aggregate(cands)
    print(json.dumps({
        "oks": oks,
        "fails": fails,
        "gate": agg["structural_gate"],
        "retained": agg["retained_structural_factors"],
        "n_results": agg["n_results"],
    }, indent=2))


if __name__ == "__main__":
    main()
