# CURRENT.md

## Active directive
- ID: D-20260721-d055-strict-resource-gate-passive-architecture-review
- Project directive: D-055
- Goal: Unify D-053 resource gates; strict replay; resume passive architecture review
- Status: done
- Acceptance: met — primary `D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT` Route_P
- Touched files: d053_analysis/evaluator; d053.rs; d055_*; experiments/generated/d055; docs/d055_*; .agent/*
- Next action: next directive = carrier-mediated import observer review; next_execution_started=false

## Repo facts needed now
- Strict Gate5/8 in d053_analysis::evaluate_gate5/8; short_horizon_relaxed removed
- Strict replay: D055_D053_STRICT_REPLAY_CONFIRMED_NOT_FOUND; retains D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND
- Control E χ≈0.90 → PASSIVE_RESOURCE_DELIVERY_HARD_BOUND_FAIL; frontier incompatible
- V14 remains EXPERIMENTAL_FAILED; not production

## Last validation
- Command: cargo test d055/d053/d054 32/32; D055_MAX_ACCEPTED=10000 D055_DIAG_HORIZON=2500 d055 pipeline release
- Result: primary=D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT route=Route_P

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Stage F not authorized
