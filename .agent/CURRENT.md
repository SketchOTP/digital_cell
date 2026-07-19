# CURRENT.md

## Active directive
- ID: D-20260719-1040-d037-turnover-provenance-renewal-gate-audit
- Project directive: D-037
- Goal: Audit membrane-turnover provenance and renewal-gate semantics
- Status: done
- Acceptance: met — `D037_TURNOVER_AND_GATE_DEFECTS`; Route A authorized; next execution not started
- Touched files: chemistry-core d037_analysis/tests, experiment-runner/d037, experiments/generated/d037, docs/d037_*
- Next action: D-038 Route A (representation mapping only) when authorized to start

## Repo facts needed now
- Primary: `D037_TURNOVER_AND_GATE_DEFECTS`
- Route: `ROUTE_A_TURNOVER_TRANSFER_REPAIR`
- D036 rejection criterion not upheld; historical tags unchanged
- Stage E remains `BLOCKED_NOT_RECOVERED`

## Last validation
- Command: `cargo test -p chemistry-core --release --test d037_tests`; `d037 pipeline`
- Result: 11/11 PASS; primary D037_TURNOVER_AND_GATE_DEFECTS

## Open blockers
- Mimir V2 MCP unavailable this session (`user-mimir` not in active MCP catalog)
