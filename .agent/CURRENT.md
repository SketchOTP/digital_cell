# CURRENT.md

## Active directive
- ID: D-20260715-1717-d012-tasks15-18
- Project directive: D-012
- Goal: Tasks 15-18 — v2 Stage E through robustness
- Status: in_progress (full 200k horizon running)
- Acceptance: infrastructure committed; diagnostic reference recorded; full horizon in progress
- Touched files: d012_analysis.rs, d012_stage_e.rs, d012_tests.rs, experiments/generated/d012/v2_stage_e_*
- Next action: await full Stage E R22 center; then solver/robust on converged candidate if any

## Repo facts needed now
- Diagnostic: LongTransientUnresolved at 5k; no joint overlap; no restoring radius
- Full run log: /tmp/d012_stage_e_full.log

## Last validation
- Command: cargo test -p chemistry-core --release --test d012_tests; cargo test -p experiment-runner --release d012
- Result: 50/50 + 3/3 PASS; diagnostic Stage E completed

## Open blockers
- Full 200k×multi-radius assay wall-clock (hours); solver/robust pending converged center
