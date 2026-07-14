# CURRENT.md

## Active directive
- ID: D-20260713-d006c-surface-turnover-completion
- Project directive: D-006C
- Goal: Finish Stage D without chemistry changes; decide restoring gate; progressive E→F if pass
- Status: done — Stage D 180/180; gate FAIL; scientific `D006_NO_RESTORING_RADIUS`; E–F not run
- Acceptance: met (Stage D complete + conclusion; E/F correctly skipped)
- Touched files: stage_d_gate.rs, d006.rs, d006_tests, scripts/d006_*, docs/d006_*
- Next action: Phase 1 remaining closure experiments under later directive (not chemistry redesign)

## Repo facts needed now
- Stage D: all survivors grow (median v_R>0); v_C_inside<0; 0 nullcline intersections
- Artifacts external: digital-protocell/experiments/generated/d006/ (gitignored)
- Prescribed restoring did not survive coupling

## Last validation
- Command: cargo test -p chemistry-core --release --test integration_tests --test validation_tests --test d003_tests --test d004_tests --test d005_tests --test d006_tests
- Result: all PASS (d006 44; validation 17; etc.)

## Open blockers
- None for D-006C
