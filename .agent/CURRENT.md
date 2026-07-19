# CURRENT.md

## Active directive
- ID: D-20260719-1500-d038-correct-turnover-transfer-replay-renewal
- Project directive: D-038
- Goal: Correct D-021 surface-turnover transfer; replay renewal architectures
- Status: done
- Acceptance: met — `D038_NO_MEMBRANE_ARCHITECTURE_RECOVERED` (Route A2); Stage E still blocked
- Touched files: chemistry-core config/surface_density/snapshot/d038_*, experiment-runner/d038, docs/d038_*
- Next action: fundamental turnover and membrane-renewal review (not D-036 auto-impl)

## Repo facts needed now
- Schema 2: `J=k_M·S·[ε_M+(1−I(φ))]`; Gate1 max_rel ~ machine eps
- Passive/linear/catalytic all still incompatible under corrected turnover
- Stage E: `BLOCKED_NOT_RECOVERED` (MIXED_PURPOSE_TERM)

## Last validation
- Command: `cargo test -p chemistry-core --release --test d038_tests`; `d038 pipeline`
- Result: 15/15 PASS; primary D038_NO_MEMBRANE_ARCHITECTURE_RECOVERED

## Open blockers
- None for D-038 closeout
