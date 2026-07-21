# CURRENT.md

## Active directive
- ID: D-20260721-d056-waste-coupled-resource-carrier
- Project directive: D-056
- Goal: Observer-only waste-coupled N/F↔W antiporter review
- Status: done
- Acceptance: met — primary `D056_CARRIER_KINETICS_NOT_IDENTIFIABLE`
- Touched files: d056_analysis/tests; experiment-runner/d056; docs/d056_*; experiments/generated/d056; .agent/*
- Next action: next directive = carrier-rate portability / alternate conservative import review; next_execution_started=false

## Repo facts needed now
- Gates 0–2 PASS (preservation, conservation, W capacity)
- Gate 3 FAIL: k_T★ ~0.005–0.95 across training; no portable (K_NF,K_W,k_T)
- Phase B not authorized; no V15; no production chemistry
- Ordinary passive import remains closed; Stage E BLOCKED

## Last validation
- Command: cargo test d056_tests 9/9; D056_MAX_ACCEPTED=2500 d056 pipeline release
- Result: primary=D056_CARRIER_KINETICS_NOT_IDENTIFIABLE

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Stage F not authorized
