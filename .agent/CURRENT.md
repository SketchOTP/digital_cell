# CURRENT.md

## Active directive
- ID: D-20260721-d058-corrected-carrier-normalization-reidentification
- Project directive: D-058
- Goal: Correct carrier observer normalization; re-identify portable reversible waste-coupled carrier (shadow-only)
- Status: done
- Acceptance: met — primary `D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT` Route V
- Touched files: d058_analysis/tests; experiment-runner/d058; docs/d058_*; experiments/generated/d058; .agent/*
- Next action: next directive = review viable organism size or additional membrane area; next_execution_started=false

## Repo facts needed now
- Start 1c9d6ae / D-057-carrier-geometry-driving-force-audit
- Defective span~185× reproduced; corrected span~194×
- Corrected p_missing≈7.81 > p_throughput≈1.07
- Invalidation: D056_CARRIER_IDENTIFICATION_INVALIDATED_BY_OBSERVER_NORMALIZATION
- No V15; Stage E BLOCKED

## Last validation
- Command: cargo test d058_tests 12/12; D058_MAX_ACCEPTED=2500 d058 pipeline release
- Result: primary=D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT Route_V

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Unrelated dirty: untracked .cursor/rules governance files (excluded)
