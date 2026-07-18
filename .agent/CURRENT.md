# CURRENT.md

## Active directive
- ID: D-20260718-d032-activated-surface-assembly
- Project directive: D-032
- Goal: Activated nonequilibrium surface assembly (P+A→S+W) on v8 interfacial architecture
- Status: done — `D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE`
- Acceptance: One D032_* conclusion with Gate evidence — met (Gate2 stop)
- Touched files: chemistry-core v9 + d032_*; experiment-runner/d032; experiments/generated/d032; docs/d032_*
- Next action: Architect follow-on for activated/immature membrane species; do not Stage F

## Repo facts needed now
- Gate0 PASS; Gate1 9/9 PASS; Gate2 FAIL portability (span≈12×, k: 155→1854)
- Record: PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT_FOR_MEMBRANE_MAINTENANCE
- Escalation: explicit activated/immature membrane species authorized next (not in D-032)
- Mimir V2 MCP: BLOCKED

## Last validation
- Command: cargo test d032_tests 9/9; d032 pipeline Gate0–2
- Result: D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE

## Open blockers
- One-rate active assembly not portable across D-031 desorption trajectory
- D-008 remains BLOCKED_NOT_RECOVERED
