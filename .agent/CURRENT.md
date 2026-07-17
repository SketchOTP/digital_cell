# CURRENT.md

## Active directive
- ID: D-20260717-d025-autonomous-surface-stage-e
- Project directive: D-025
- Goal: Seal D-024 provenance; couple surface density to autonomous φ motion; revalidate B–D; re-enter Stage E
- Status: Gates 0–6 PASS; starting Gate 7 dynamic R22
- Acceptance: Gates 0–8 in order or stop at first fail
- Touched files: surface_density, simulation, config, d025_tests, d025 runner, d025 artifacts
- Next action: Gate 7 fully dynamic R22 bootstrap; then Stage E

## Repo facts needed now
- D024_PROVENANCE_SEALED; Gate1–2 unit PASS; Gate3–6 runner PASS
- Stage C harness needed v2 eta_c closure; Stage D fixed_geometry is φ-only
- Autonomous advection when enforce_structure_constraint=false
- Mimir: BLOCKED (path mapping)

## Last validation
- Command: d025 stage-c PASS; d025 stage-d PASS; d025_tests + d024_tests prior PASS
- Result: D025_GATE6_PASS

## Open blockers
- Mimir V2 unavailable
- Gates 7–8 not yet run
