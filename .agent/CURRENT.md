# CURRENT.md

## Active directive
- ID: D-20260714-2027-stage-c-evidence-harden
- Project directive: D-010 (Stage C quality follow-up)
- Goal: Harden Stage C evidence/provenance (boundedness, missing-A, φ/M invariance, Stage A/B hashes)
- Status: done — follow-up commit pending
- Acceptance: Four findings fixed; d008_tests 45 PASS; runner d008::tests 8 PASS; no governed Stage C; no reports/manifest edits
- Touched files: activated_metabolism.rs, candidate_identity.rs, d008_tests.rs, experiment-runner/d008.rs
- Next action: commit Stage C evidence/provenance hardening

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Stage C source tip before harden: 95e53f96fd745d99b2040542837df9940db13aa0
- Hash scheme: Transport (no Stage B) = Stage A short betas; Stage B enabled = membrane fields; ActivatedMetabolism = membrane + Stage C rates
- Boundedness: stage_c_clamp_negligible uses CUMULATIVE_RESIDUAL_TOL on catalyst/activated clamp corrections

## Last validation
- Command: cargo test -p chemistry-core --release --test d008_tests; cargo test -p experiment-runner --release d008::tests
- Result: 45 PASS; 8 PASS

## Open blockers
- None
