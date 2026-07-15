# CURRENT.md

## Active directive
- ID: D-20260715-1717-d012-tasks15-18
- Project directive: D-012
- Goal: Tasks 15-18 — v2 Stage E through robustness
- Status: in_progress
- Acceptance: d012_tests PASS; Stage E infrastructure committed; governed assays run with honest classification
- Touched files: d012_analysis.rs, d012_stage_e.rs, d012.rs, main.rs, d012_tests.rs, docs/d012_v2_joint_balance.md
- Next action: commit infrastructure; run diagnostic then full Stage E horizons

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- v2 base rates from stage_c_selected.toml + ledger estimate at R=22
- D011 v1 search superseded (nonconservative v1)

## Last validation
- Command: cargo test -p chemistry-core --release --test d012_tests; cargo test -p experiment-runner --release d012
- Result: d012_tests 50/50 PASS; experiment-runner d012 3/3 PASS

## Open blockers
- Full 200k×6 radii runs may take hours; use background + job ledger
