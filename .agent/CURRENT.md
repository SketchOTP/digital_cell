# CURRENT.md

## Active directive
- ID: D-20260715-1658-d012-tasks11-14
- Project directive: D-012
- Goal: Tasks 11-14 — v2 Stage A-D validation
- Status: done
- Acceptance: d012_tests 41/41 PASS; governed v2 Stage B/C/D PASS; Task 15 authorized
- Touched files: d008.rs, d012.rs, main.rs, d012_tests.rs, docs/d012_v2_stage_validation.md, experiments/generated/d012/v2_stage_*
- Next action: Task 15 transport-coupled Stage E reference assay

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- v2 Stage B robustness: initial M levels {0.50, 0.75} (not 0.25) due to A-coupled synthesis
- v2 Stage D metrics identical to v1 at unit yield

## Last validation
- Command: cargo test -p chemistry-core --release --test d012_tests; d008_tests; experiment-runner run_v2_stage
- Result: d012_tests 41/41 PASS; d008_tests 50/50 PASS; run_v2_stage 3/3 PASS
- Stage B/C/D: D012_STAGE_*_PASS

## Open blockers
- Mimir MCP unavailable (server not in session)
