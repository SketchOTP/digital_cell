# CURRENT.md

## Active directive
- ID: D-20260717-d025-autonomous-surface-stage-e
- Project directive: D-025
- Goal: Seal D-024 provenance; couple surface density to autonomous φ motion; revalidate B–D; re-enter Stage E
- Status: Gate 0 PASS — D024_PROVENANCE_SEALED; implementing Gate 1 velocity
- Acceptance: Gates 0–8 in order or stop at first fail with exact conclusion
- Touched files: experiments/generated/d025/d024_provenance_seal/, docs/d024_provenance_seal_addendum.md
- Next action: Implement autonomous v_n estimator + Gate 1 manufactured velocity tests

## Repo facts needed now
- D024_PROVENANCE_SEALED at source 06477f6; binary 5894fcec…; k_ads=0.001111…
- Gate6 sealed: Γ≈1.0, C≈0.991, A≈0.924, residual≈2.5e-7
- Tag to create: D-024-surface-density-pass-provenance-sealed (preserve D-024-surface-density-pass)
- Mimir: BLOCKED (Windows path mapping on register/resolve)

## Last validation
- Command: worktree@06477f6 release experiment-runner d024 pipeline
- Result: PASS primary_conclusion=D024_INTERFACIAL_SURFACE_DENSITY_PASS; gate0_pass=true

## Open blockers
- Mimir V2 register/resolve path mapping unavailable
