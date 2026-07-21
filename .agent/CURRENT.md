# CURRENT.md

## Active directive
- ID: D-20260720-2335-d052-resource-delivery-resistance-decomposition
- Project directive: D-052
- Goal: Decompose N/F delivery resistance preventing environmental resources from sustaining coupled activation (diagnostic only)
- Status: done
- Acceptance: met — primary `D052_MIXED_RESOURCE_DELIVERY_LIMIT`
- Touched files: d052_analysis, d052_tests, d052.rs, main.rs, docs/d052_*, experiments/generated/d052, .agent/*
- Next action: bounded combined exterior-near-interface + membrane-crossing delivery repair; next_execution_started=false

## Repo facts needed now
- Start: e08075a / D-051-coupled-activation-throughput-audit
- Resistance: exterior ≈43% + membrane ≈37%; no single-segment dominance
- Reservoir / NF attenuation bypass / diffusion 5× do not restore A ≥50%
- Healthy interior N+F still rescues A≈1.09; joint resource limit
- Record: ACTIVATION_SUPPLY_LAW_NOT_CURRENT_REPAIR_TARGET

## Last validation
- Command: cargo test d052_tests 13/13; D052_MAX_ACCEPTED=10000 pipeline
- Result: primary=D052_MIXED_RESOURCE_DELIVERY_LIMIT

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Do not alter activation until combined delivery resistance addressed
