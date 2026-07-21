# CURRENT.md

## Active directive
- ID: D-20260721-d057-carrier-geometry-normalization-driving-force-audit
- Project directive: D-057
- Goal: Observer-only audit of waste-coupled carrier k_T★ nonportability
- Status: done
- Acceptance: met — primary `D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT` Route G
- Touched files: d057_analysis/tests; experiment-runner/d057; docs/d057_*; experiments/generated/d057; .agent/*
- Next action: next directive = repair carrier surface/face/dt normalization + rerun D-056 Phase A; next_execution_started=false

## Repo facts needed now
- D-056 sealed ed6de2c / D-056-waste-coupled-resource-carrier-fail
- D-056 δ proxy = interface_weight; production δ = cell_delta_estimate
- No measure/drive model portable; S/V limit secondary pending norm repair
- No V15; Stage E BLOCKED

## Last validation
- Command: cargo test d057_tests 10/10; D057_MAX_ACCEPTED=2500 d057 pipeline release
- Result: primary=D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT Route_G

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Unrelated dirty: PROJECT_GOAL UMBRA + Cursor rule migration
