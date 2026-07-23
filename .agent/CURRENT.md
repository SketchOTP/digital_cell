# CURRENT.md

## Active directive
- ID: D-20260723-d082-edge-membrane-activation-supply-integration
- Project directive: D-082
- Goal: Integrate canonical N/F→A into edge assay; requalify affordability
- Status: done
- Acceptance: met — `D082_EDGE_ACTIVATION_INTEGRATION_REPAIRED` (Route I)
- Touched files: d082_*, docs/d082_*, d081 provisional, experiments/generated/d082, .agent/*
- Next action: D-080 dynamic-interface repair; next_execution_started=false

## Repo facts needed now
- Start: 41e9936 / D-081-edge-reserve-causality-fail
- D-081 Gate5 was ACTIVATION_NOT_DISPATCHED (bolus only)
- Integration repaired; Gate4 affordability PASS
- Resume: Gate8 dynamic FAIL; Gate9 coupled OK; structural incompatible
- Leave unstaged: .cursor/rules/*, AGENTS.md

## Last validation
- Command: cargo test -p chemistry-core --test d082_tests; cargo run --release d082 pipeline; resume gate8/9
- Result: d082 7/7; Route I; dynamic fail; structural incompatible

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- Dynamic interface under cut-cell support still fails
- Frozen structural drive universally positive
