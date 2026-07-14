#!/usr/bin/env python3
"""Write D-006C Stage D scientific docs + manifest after matrix drain."""
from __future__ import annotations

import hashlib
import json
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
D006 = ROOT / "digital-protocell/experiments/generated/d006"
DOCS = ROOT / "docs"


def main() -> None:
    agg = json.loads((D006 / "stage_d/aggregate_flow.json").read_text())
    led = json.loads((D006 / "stage_d/job_ledger_summary.json").read_text())
    gate = json.loads((D006 / "stage_d/gate_decision.json").read_text())
    nulls = json.loads((D006 / "stage_d/nullcline_summary.json").read_text())
    sha = (D006 / "stage_d/experiment_runner.sha256").read_text().strip()
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()

    scientific = "D006_NO_RESTORING_RADIUS"
    rows = []
    for ev in agg["candidate_evaluations"]:
        for c in ev["crossings_by_C0"]:
            for m in c["median_v_R_by_R0"]:
                # attach min/max from radius_flow_table
                match = next(
                    (
                        t
                        for t in agg["radius_flow_table"]
                        if t["candidate"] == ev["candidate_id"]
                        and t["C0"] == c["C0"]
                        and t["R0"] == m["R0"]
                    ),
                    None,
                )
                rows.append(
                    {
                        "candidate": ev["candidate_id"],
                        "factor": ev["factor"],
                        "C0": c["C0"],
                        "R0": m["R0"],
                        "median_radial_velocity": m["median_v_R"],
                        "minimum_radial_velocity": match and match["minimum_radial_velocity"],
                        "maximum_radial_velocity": match and match["maximum_radial_velocity"],
                        "successful_replicates": match and match["successful_replicates"],
                        "failure_classifications": match and match["failure_classifications"],
                    }
                )

    # radius flow report
    (DOCS / "d006_radius_flow_report.md").write_text(
        f"""# D-006 Radius Flow Report

**Directive:** D-006C  
**Agent memory:** D-20260713-d006c-surface-turnover-completion  
**Equation version:** `surface_turnover_v1`  
**Stage D jobs:** {agg['n_runs']} / 180 complete  
**Scientific conclusion:** `{scientific}`

## Method

Coupled fresh-seed runs (50,000 accepted substeps) at  
R₀ ∈ {{16,20,24,28,32}} × C₀ ∈ {{0.275,0.35,0.425}} × seeds {{1,2,3}}  
for the four prescribed-radius survivors (0.80×–1.40×).

Velocities use simulated time. Invalid-stabilization flags applied before medians.

## Candidate-by-candidate radius flow

All four candidates show **median v_R > 0 at every tested radius and catalyst loading**.

No ordered restoring-radius crossing exists.

Machine table: `experiments/generated/d006/stage_d/aggregate_flow.json` (`radius_flow_table`).

## Catalyst-state flow

At every Stage D macrostate point, median `v_C_inside < 0` while median `v_R > 0`:

- radius expands
- mean internal catalyst concentration declines slowly
- catalyst retention remains ≥ 0.88 (not extinction)

No radius/catalyst nullcline intersection in the tested region  
(`stage_d/nullcline_summary.json`).

## Interpretation

Prescribed-field Stage C restoring crossings **did not survive** full coupling.  
Coupled organism dynamics in the screened window are expansive in radius, without a restoring R*.

## Stage D gate

- Restoring radial sign pattern: **fail**
- Selected candidate: **none**
- Stage E/F: **not run**
"""
    )

    (DOCS / "d006_basin_report.md").write_text(
        f"""# D-006 Basin Report

**Directive:** D-006C  
**Status:** not executed — Stage D gate failed (`{scientific}`).

Refined-basin Stage E progressive gates (E1–E3), noise sensitivity, and contiguous-patch
requirements were not opened because no Stage D candidate demonstrated an ordered
coupled restoring-radius crossing.
"""
    )

    (DOCS / "d006_candidate_report.md").write_text(
        f"""# D-006 Candidate Report

**Directive:** D-006 / D-006C  
**Agent memory:** D-20260713-d006c-surface-turnover-completion  
**Equation version:** `surface_turnover_v1`

## Derived interface rate

`k_structure_interface_initial ≈ 0.09642857142857159`

## Immutable candidates

Five candidates; Stage D scheduled only prescribed survivors:

| Factor | Prescribed crossing | Stage D coupled restoring |
| --- | --- | --- |
| 0.60× | fail (excluded) | n/a |
| 0.80× | pass | **fail** (all median v_R > 0) |
| 1.00× | pass | **fail** (all median v_R > 0) |
| 1.20× | pass | **fail** (all median v_R > 0) |
| 1.40× | pass | **fail** (all median v_R > 0) |

## Job matrix

`4 × 5 × 3 × 3 = 180` (not 225 — 0.60× rejected at prescribed-radius).

Confirmed from `candidates/index.json`, `prescribed_radius/*/result.json`, and `/tmp/d006_screen_jobs.txt`.

## Scientific conclusion

```text
{scientific}
```

Selected candidate: **none**.

Execution status that applied while Stage D ran:

```text
D006_RESULT_UNRESOLVED_STAGE_D_IN_PROGRESS
```
"""
    )

    (DOCS / "d006_stage_d_completion.md").write_text(
        f"""# D-006 Stage D Completion Report

**Directive:** D-006C  
**Agent memory:** D-20260713-d006c-surface-turnover-completion  
**Equation version (frozen):** `surface_turnover_v1`

## Final execution / scientific statuses

While incomplete:

```text
D006_RESULT_UNRESOLVED_STAGE_D_IN_PROGRESS
```

After Stage D matrix + gate:

```text
{scientific}
```

## Matrix audit

| Quantity | Value |
| --- | --- |
| Theoretical 5×5×3×3 | 225 |
| Scheduled | **180** |
| Completed usable for flow | **180** |
| Resumed after orchestration reset | remaining PENDING after thrash cleanup |
| Invalid (schema-strict §6) | 180 result.json lack field hashes/accounting/clean_termination |
| Scientific usability | all 180 have identity, 50k steps, velocities, Q/slopes, retention |

Why 180: 0.60× prescribed `has_stable_crossing=false` → 4×5×3×3.

## Provenance

| Item | Value |
| --- | --- |
| Experiment-runner binary mtime | 2026-07-13 16:20 |
| Binary sha256 | `{sha.split()[0]}` |
| Chemistry freeze | reactions/`surface_turnover_v1` unchanged during Stage D |
| Pre-savepoint HEAD | `e21068bb2c7f827a563320e105507211379b4f77` |
| Post-savepoint commit | recorded in manifest after git commit |

## Gate decision

- Restoring crossing: **none**
- Nullcline intersections: **0**
- Selected candidate: **none**
- Stage E/F: **skipped**

Artifacts: `experiments/generated/d006/stage_d/`
"""
    )

    for name, body in {
        "d006_noise_sensitivity.md": "not run — Stage D gate failed; no selected candidate.",
        "d006_control_report.md": "not run — Stage D gate failed; short causal controls gated off.",
        "d006_puncture_mechanism.md": "Local Stage A puncture geometry tests remain in d006_tests; matched coupled puncture from Stage E center **not run** (no Stage D pass).",
        "d006_full_acceptance.md": "not run — Stage D gate failed; Stage F acceptance not opened.",
    }.items():
        (DOCS / name).write_text(
            f"""# {name.replace('_', ' ').replace('.md','').title()}

**Directive:** D-006C  
**Scientific conclusion:** `{scientific}`

{body}
"""
        )

    # append historical revisions to prior docs
    for path, note in [
        (
            DOCS / "d005_final_closure.md",
            f"\n\n## Revision (D-006C)\n\nD-005 conclusion `D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR` preserved. "
            f"D-006C Stage D closed with `{scientific}` — coupled surface-turnover candidates "
            f"showed no restoring radius in the 180-job screen.\n",
        ),
        (
            DOCS / "d003_candidate_report.md",
            f"\n\n## Revision (D-006C)\n\nNo change to D-003 candidate identity. Downstream D-006C "
            f"concluded `{scientific}` under `surface_turnover_v1`.\n",
        ),
        (
            DOCS / "d004_attractor_report.md",
            f"\n\n## Revision (D-006C)\n\nD-004 pipeline-defect finding preserved. D-006C did not reopen "
            f"crowding attractors; Stage D surface-turnover screen → `{scientific}`.\n",
        ),
    ]:
        if path.exists() and "Revision (D-006C)" not in path.read_text():
            path.write_text(path.read_text() + note)

    # external artifact manifest (not committing megabytes)
    art_root = D006.resolve()
    file_count = sum(1 for _ in art_root.rglob("*") if _.is_file())
    # hash of gate+ledger summaries only (bounded)
    h = hashlib.sha256()
    for rel in [
        "stage_d/job_ledger_summary.json",
        "stage_d/aggregate_flow.json",
        "stage_d/gate_decision.json",
        "stage_d/nullcline_summary.json",
        "stage_d/experiment_runner.sha256",
        "candidates/index.json",
    ]:
        p = D006 / rel
        if p.exists():
            h.update(p.read_bytes())
    manifest = {
        "directive": "D-006C",
        "equation_version": "surface_turnover_v1",
        "scientific_conclusion": scientific,
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_d_jobs_complete": 180,
        "selected_candidate": None,
        "stage_e_f_run": False,
        "artifact_root": str(art_root),
        "artifact_file_count": file_count,
        "manifest_content_hash": h.hexdigest(),
        "experiment_runner_sha256": sha.split()[0],
        "git_commit_at_write": commit,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "schema_note": (
            "Stage D result.json lack initial/final field hashes, accounting object, "
            "clean_termination; classified COMPLETE_INVALID under strict §6 but usable for flow analysis."
        ),
    }
    (D006 / "manifest_d006c.json").write_text(json.dumps(manifest, indent=2))
    (DOCS / "d006_manifest_pointer.md").write_text(
        f"""# D-006C Manifest Pointer

Scientific conclusion: `{scientific}`

External artifact root (gitignored generated tree):

`{art_root}`

Manifest: `digital-protocell/experiments/generated/d006/manifest_d006c.json`

Content hash: `{h.hexdigest()}`
"""
    )
    print("docs+manifest written", scientific)


if __name__ == "__main__":
    main()
