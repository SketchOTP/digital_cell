# CURRENT.md

## Active directive
- ID: D-20260721-d054-dynamic-resource-geometry-passive-upper-bound
- Project directive: D-054
- Goal: Seal D-053; audit fixed-vs-dynamic resource geometry; select architecture route (no repair)
- Status: done
- Acceptance: met — primary `D054_D053_PROVENANCE_RERUN_DIVERGED`
- Touched files: d053 seal docs/label; d054_analysis/tests; docs/d054_*; experiments/generated/d054; .agent/*
- Next action: repair D-053 Gate5/Gate8 harness and rerun; next_execution_started=false

## Repo facts needed now
- D-053 source: 76c0898; corrected primary BOUNDED_DELIVERY_REPAIR_NOT_FOUND at Gate5
- Informal Gate9 metrics reproducible when pair forced (A≈0.047, χ≈0.47)
- Informal Gate8 never met χ≥1.05 (short_horizon_relaxed)
- V14 recorded EXPERIMENTAL_FAILED; not qualified

## Last validation
- Command: cargo test d053_tests 12/12; d054_tests 10/10; sealed D053 early→Gate5 fail; forced Gate9@10k
- Result: primary=D054_D053_PROVENANCE_RERUN_DIVERGED

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Stage F not authorized
- D-053 validation harness must be repaired before architecture selection
