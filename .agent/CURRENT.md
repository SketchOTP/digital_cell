# CURRENT.md

## Active directive
- ID: D-20260717-d025-autonomous-surface-stage-e
- Project directive: D-025
- Goal: Seal D-024 provenance; autonomous Γ transport; revalidate B–D; Stage E re-entry
- Status: done — Gate 8 failed; `D025_STAGE_E_LONG_TRANSIENT_UNRESOLVED`
- Acceptance: D025_STAGE_E_RECOVERED or honest D025_* failure with artifacts — met (honest failure)
- Touched files: surface_density/autonomous vn, d025.rs, d025_stage_e.rs, d025_analysis.rs, d025 artifacts/docs
- Next action: Do not start Stage F; remediate Stage E A-retention / quasi-steady under v7

## Repo facts needed now
- D-024 provenance sealed at `06477f6`; tag `D-024-surface-density-pass-provenance-sealed`
- Gates 0–7 PASS; Gate 8 formal 200k NOT_CONVERGED; A_ret≈0.512
- Architecture: INTERFACIAL_SURFACE_DENSITY_SELECTED
- Mimir: BLOCKED (Windows path mapping)

## Last validation
- Command: cargo test -p chemistry-core --release --test d025_tests; --test d024_tests
- Result: 15/15 PASS; 24/24 PASS; Stage E 200k NOT_CONVERGED

## Open blockers
- Mimir project register/resolve fails: `\\home\\sketch\\Projects\\digital_cell`
- Stage E not recovered; D-008 remains BLOCKED_NOT_RECOVERED
