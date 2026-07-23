# CURRENT.md

## Active directive
- ID: D-20260723-d081-edge-membrane-reserve-causality-audit
- Project directive: D-081
- Goal: Edge-membrane reserve provenance and replenishment causality audit
- Status: started
- Acceptance: One D081_* primary; D-080 preserved+tagged; Gate7 provisional pending audit; stop at first mandatory fail
- Touched files: (pending) d081_*, d080 provisional record, .agent/*
- Next action: Reproduce D-080; commit+tag preservation; then implement D-081 gates

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Sealed start: 99c0236 / D-079-edge-network-boundary-fail
- Uncommitted D-080: D080_EDGE_NETWORK_REPAIR_OR_CAUSALITY_FAILURE
- Leave unstaged: .cursor/rules/*, AGENTS.md
- Frozen: cut-cell support, bind/unbind/lateral/perm, damage, A→L law; no A-for-binding

## Last validation
- Command: (pending D-080 reproduction)
- Result: —

## Open blockers
- Mimir baseline path degraded on remote host (task still active)
- Stage E BLOCKED_NOT_RECOVERED pending D-081 outcome
