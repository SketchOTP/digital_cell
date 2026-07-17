# CURRENT.md

## Active directive
- ID: D-20260717-d025-autonomous-surface-stage-e
- Project directive: D-025
- Goal: Seal D-024 provenance; couple surface density to autonomous φ motion; revalidate B–D; re-enter Stage E
- Status: Gate 0–2 PASS (unit); Gate 3+ pending
- Acceptance: Gates 0–8 in order or stop at first fail with exact conclusion
- Touched files: surface_density.rs, config.rs, simulation.rs, d025_tests.rs, d025 artifacts
- Next action: Gate 3 chemistry-driven growth/shrinkage; Stage B–D regression; dynamic R22; Stage E

## Repo facts needed now
- D024_PROVENANCE_SEALED @ 06477f6; seal tag D-024-surface-density-pass-provenance-sealed
- Autonomous vn: v_n=−∂tφ/sqrt(|∇φ|²+η_v²); n inward ⇒ expansion mean vn < 0
- Advection on when enforce_structure_constraint=false (apply_phi); constrained Stage E unchanged
- Mimir: BLOCKED (Windows path mapping)

## Last validation
- Command: cargo test -p chemistry-core --release --test d024_tests --test d025_tests
- Result: d024 24/24 PASS; d025 9/9 PASS

## Open blockers
- Mimir V2 register/resolve path mapping unavailable
- Gates 3–8 not yet executed
