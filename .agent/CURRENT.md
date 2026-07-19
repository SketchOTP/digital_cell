# CURRENT.md

## Active directive
- ID: D-20260719-1536-d040-exchange-precursor-coupling-decomposition
- Project directive: D-040
- Goal: Diagnose exchange–precursor coupling failure under schema-3 v8 without changing chemistry
- Status: done
- Acceptance: met — `D040_MEMBRANE_METABOLISM_BISTABILITY` (Route F)
- Touched files: chemistry-core/d040_analysis+tests, experiment-runner/d040, docs/d040_*, experiments/generated/d040, .agent/*
- Next action: basin-accessibility / local-bootstrap directive; do not alter validated exchange law

## Repo facts needed now
- Record: SCHEMA3_V8_MAINTENANCE_COUPLING_FAILED
- Gate1: EXCHANGE_LAW_PARITY_PASS_PRECURSOR_BELOW_EQUILIBRIUM
- Gate3: PASSIVE_EXCHANGE_CAN_REPAIR_WITH_SUFFICIENT_PRECURSOR (p≈0.02)
- Gate4: synthesis_capacity_sufficient (offline exchange)
- Stage E: BLOCKED_NOT_RECOVERED

## Last validation
- Command: cargo test -p chemistry-core --test d040_tests --release; D040_MAX_ACCEPTED=2000 d040 pipeline
- Result: 15/15 PASS; pipeline Route_F / D040_MEMBRANE_METABOLISM_BISTABILITY

## Open blockers
- None for D-040 closeout
