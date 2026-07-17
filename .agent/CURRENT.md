# CURRENT.md

## Active directive
- ID: D-20260717-d027-coupled-surface-renewal
- Project directive: D-027
- Goal: Recalibrate k_ads for coupled surface renewal under sealed v7
- Status: done — `D027_ISOLATED_SURFACE_RENEWAL_FAILURE`
- Acceptance: One D027_* conclusion with Gates 0–2 evidence; stop at first failed gate — met
- Touched files: surface_density, simulation, d013, d027_analysis, d027_tests, d027.rs, main.rs, docs/d027_*, experiments/generated/d027, .agent/*
- Next action: Architect bulk–surface exchange improvement; do not Stage F; do not productive-rate-only repair

## Repo facts needed now
- HEAD was 77f7ab2 at start; Gate0 ledger repair + Gate1 portable ~30.4× k_ads
- Gate4: 1× Q≈0.91, 2× Q≈1.06 — bracket straddles balance; intermediates forbidden
- Mimir: BLOCKED (Windows path mapping on project_register)
- Disk: ~8GB free; avoid large checkpoint dumps in d027 artifacts

## Last validation
- Command: cargo test -p chemistry-core --release --test d027_tests; d026_tests; d027 pipeline Gates0-4
- Result: d027 7/7 PASS; d026 21/21 PASS; Gate0/1/2 PASS; Gate4 FAIL

## Open blockers
- Isolated surface renewal not achieved on mandated 0.5/1/2 grid
- D-008 remains BLOCKED_NOT_RECOVERED
