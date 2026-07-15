# CURRENT.md

## Active directive
- ID: D-20260715-d013-stage-e-harness-integrity
- Project directive: D-013
- Goal: Repair Stage E harness; recover governed conservative-v2 reference
- Status: done — D013_REFERENCE_NUMERICAL_FAILURE
- Acceptance: preflight PASS; valid R22 governed artifact with accepted-step windows/checkpoints/activation; one D013_* conclusion
- Touched files: d013_harness.rs, d013.rs, d011.rs assay loop, simulation attempt counters, d013_tests, docs/d013_*, experiments/generated/d013/
- Next action: repair timestep-floor numerical failure before scientific Stage E / solver

## Repo facts needed now
- Mimir slug: digital_cell; Mimir MCP unavailable this session
- Frozen candidate/config hashes unchanged
- Invalid D-012 ref preserved + tag D-012-stage-e-reference-invalid
- R22 valid artifact terminates TIMESTEP_FLOOR_FAILURE at 161166 accepted steps
- Solver entry closed; R18/R26 not run

## Last validation
- Command: cargo test d013_tests+d012_tests; preflight PASS; R22 governed run
- Result: d013 32 PASS; preflight_pass=true; R22 VALID_GOVERNED_ARTIFACT + NUMERICAL_FAILURE

## Open blockers
- Numerical: timestep floor at ~161k accepted; need root-cause repair before scientific pass
- BLOCKED: Mimir MCP unavailable
