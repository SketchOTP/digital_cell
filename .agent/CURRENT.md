# CURRENT.md

## Active directive
- ID: D-20260714-d007-joint-kinetic-nullclines
- Project directive: D-007
- Goal: Joint structural–catalyst fixed-point search within bounded rates
- Status: done — structural gate FAIL; scientific `D007_NO_STRUCTURAL_NULLCLINE`; catalyst/joint not run
- Acceptance: met (one conclusion; strict schema; D-006 preserved; §10 stop honored)
- Touched files: d007_analysis.rs, d007.rs, d007_tests, scripts/d007_*, configs/d007/, docs/d007_*, experiments/generated/d007/
- Next action: Next directive — transport boundary or metabolic intermediate (not another rate sweep)

## Repo facts needed now
- D007_NO_STRUCTURAL_NULLCLINE (0.50–0.80× all fail restoring gate; 0.80× ALL_GROW)
- Reference: config hash matches D-006 1.0×; 10k transient vs 50k Stage-D direction
- Phase1: PHASE1_SELF_MAINTENANCE_PARTIAL

## Last validation
- Command: cargo test -p chemistry-core --release --test d007_tests
- Result: 26 PASS; structural 63/63 complete

## Open blockers
- None for D-007; architecture ceiling reached for surface_turnover_v1 rate tuning
