# CURRENT.md

## Active directive
- ID: D-20260720-d048-frozen-biology-membrane-basin-repair
- Project directive: D-048
- Goal: Frozen-biology membrane basin and repair validation under historical activation
- Status: done
- Acceptance: met — `D048_NO_HEALTHY_MEMBRANE_ATTRACTOR`; Gates 0–1 pass; Gate 2 decisive fail; Gates 3–10 not run
- Touched files: d048_analysis.rs, d048_tests.rs, d048.rs, main.rs, lib.rs, docs/d048_*, experiments/generated/d048, .agent/*
- Next action: membrane–metabolism coupling review via full A/P/S histories; do not redesign activation; next_execution_started=false

## Repo facts needed now
- Gate2: A retention ~0.01 by first 10k window; net S flow strongly negative; localization OK
- Freeze: r_A=0.020*C*N*F; schema-3; HISTORICAL_ACTIVATION_FROZEN_FOR_MEMBRANE_VALIDATION
- D-047 authorization stands; membrane basin not established

## Last validation
- Command: cargo test d048_tests 12/12; pipeline D048_MAX_ACCEPTED=50000 Gate2 fail
- Result: primary=D048_NO_HEALTHY_MEMBRANE_ATTRACTOR
- Tag: D-048-frozen-biology-membrane-fail (pending)

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
