# CURRENT.md

## Active directive
- ID: D-20260718-d030-orthogonal-reversible-exchange-identification
- Project directive: D-030
- Goal: Orthogonal transient ID of v8 reversible P↔S exchange (α,β); recover Stage E without replacing architecture
- Status: done — `D030_TURNOVER_EXCHANGE_INCOMPATIBILITY`
- Acceptance: One D030_* conclusion; stop at first failed gate — met (Gate 7)
- Touched files: chemistry-core d030_analysis/tests, experiment-runner/d030, experiments/generated/d030, docs/d030_*, docs/d029_*, .agent/*
- Next action: Architect follow-on; do not Stage F; do not reject v8 solely from Gate 7; do not productive-rate-only repair

## Repo facts needed now
- Gates 0–6 PASS (orthogonal α/β identifiable; mixed+equilibrium OK)
- Gate 7 FAIL: CapacityExceeded after long isolated burn; accepted_in_window=0
- D-029 operative reinterpretation recorded; historical conclusion/tag unchanged
- Mimir V2: BLOCKED (Windows path mapping on project_register)

## Last validation
- Command: cargo test -p chemistry-core --release --test d030_tests; experiment-runner d030 pipeline
- Result: 11/11 PASS; Gate7 D030_TURNOVER_EXCHANGE_INCOMPATIBILITY

## Open blockers
- Identified reversible kinetics not compatible with sustained isolated biological renewal
- D-008 remains BLOCKED_NOT_RECOVERED
