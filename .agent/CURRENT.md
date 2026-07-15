# CURRENT.md

## Active directive
- ID: D-20260715-d012-conservative-stoichiometric-closure
- Project directive: D-012
- Goal: Conservative stoichiometric repair through Stage E / D-012 conclusion
- Status: Stage E Tasks 15–18 running (background); Tasks 1–14 complete
- Acceptance: conservation gate PASS; Stages A–D PASS; Stage E restoring overlap with material/activation accounting before Stage F
- Touched files: stoichiometry.rs, d012_accounting.rs, activated_metabolism.rs, membrane.rs, simulation.rs, d012.rs, d008.rs, docs/d012_*, experiments/generated/d012/
- Next action: await/finish transport-coupled Stage E calibration/solver/robustness; then Task 19 final reports/tags

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- V1: D012_NONCONSERVATIVE_V1_CONFIRMED (rank 6, left-null dim 1, not nonnegative)
- D-011: D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY
- Tags: D-011-long-horizon-incomplete, D-012-stoichiometric-audit
- V2 conservation gate: PASSED
- V2 Stage B/C/D: PASS (Stage B robustness used M∈{0.50,0.75}, not 0.25)
- Stage E fail tag preserved: D-008-stage-e-balance-fail
- Stages F–G still blocked until conservative Stage E passes
- Production verdict: REQUIRES_REMEDIATION
- Phase 1: PHASE1_SELF_MAINTENANCE_PARTIAL

## Last validation
- Command: d012_tests 41 PASS; d008_tests 50 PASS; governed v2 stages B/C/D PASS
- Result: Task 15 authorized and launched

## Open blockers
- Stage E long-horizon runs in progress
- Mimir MCP unavailable (fetch failed / server not in session)
