# CURRENT.md

## Active directive
- ID: D-20260718-d032-activated-surface-assembly
- Project directive: D-032
- Goal: Activated nonequilibrium surface assembly (P+A→S+W) on v8 interfacial architecture
- Status: in progress — Gate0 PASS; Gate2–5 pipeline running
- Acceptance: One D032_* conclusion; Stage E recovered only if all applicable gates pass
- Touched files: chemistry-core config/surface_density/d032_analysis/candidate_identity/simulation; tests/d032_tests; experiment-runner/d032; main.rs
- Next action: Await Gate2 portability + candidate screen + Gate5 isolated renewal

## Repo facts needed now
- V9: membrane_metabolism_v9_activated_surface_assembly; schema exchange=3, active_assembly=1
- Frozen α≈0.167 β≈0.00334; integrator v2 unchanged
- Gate1 unit tests: 9/9 PASS
- Disk ~5.1–5.4G available (98%)
- Mimir V2 MCP: BLOCKED (legacy HTTP only)

## Last validation
- Command: cargo test -p chemistry-core --release --test d032_tests
- Result: 9/9 PASS; Gate0 D032_PRESERVATION_PASS

## Open blockers
- Mimir V2 status: BLOCKED (MCP server unavailable)
- Long Gate2–15 horizons pending
