# CURRENT.md

## Active directive
- ID: D-20260718-d033-activated-membrane-intermediate
- Project directive: D-033
- Goal: Two-stage activated membrane intermediate pathway; recover Stage E if portable
- Status: done — `D033_ISOLATED_RENEWAL_FAILURE`
- Acceptance: One D033_* conclusion with Gate evidence — met (Gate5 stop)
- Touched files: chemistry-core v10 + d033_*; experiment-runner/d033; experiments/generated/d033; docs/d033_*
- Next action: Architect immature/mature surface membrane states; do not Stage F; do not retune rates/transport

## Repo facts needed now
- Gates 0–4 PASS; Gate5 FAIL (bulk X + desorption deficit across rate screen)
- Kinetics identifiable; buffering proven; renewal not portable under frozen transport
- Escalation: immature/mature surface states authorized
- D-008 remains BLOCKED_NOT_RECOVERED

## Last validation
- Command: cargo test d033_tests 9/9; d033 gates 0–4 PASS; Gate5 screen FAIL
- Result: D033_ISOLATED_RENEWAL_FAILURE

## Open blockers
- One soluble intermediate insufficient for portable isolated renewal under frozen D_X/no attraction
