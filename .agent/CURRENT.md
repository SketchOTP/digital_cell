# CURRENT.md

## Active directive
- ID: D-20260719-0326-d034-surface-bound-membrane-maturation
- Project directive: D-034
- Goal: Immature U / mature S surface maturation; recover Stage E if portable
- Status: done — `D034_MATURATION_LAW_NOT_PORTABLE`
- Acceptance: One D034_* conclusion with Gate evidence — met (Gate6 stop)
- Touched files: chemistry-core v11 + d034_*; experiment-runner/d034; experiments/generated/d034; docs/d034_*
- Next action: Architecture review of membrane-bound catalytic assembly; do not Stage F; do not add species or retune rates

## Repo facts needed now
- Gates 0–5 PASS (unit/smoke); Gate6 FAIL (k_req span ≈33× > 3×)
- Local maturation ID works; portable renewal rate across U/S states does not
- D-008 remains BLOCKED_NOT_RECOVERED

## Last validation
- Command: cargo test d034_tests 9/9; d034 gates 0/2/4 PASS; Gate6 FAIL; pipeline stop
- Result: D034_MATURATION_LAW_NOT_PORTABLE

## Open blockers
- Single k_mature cannot balance forced U/S occupancy family under L_S∝Γ_S / B∝Γ_U a
