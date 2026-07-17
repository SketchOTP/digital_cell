# CURRENT.md

## Active directive
- ID: D-20260716-d022-interface-affinity-localization
- Project directive: D-022
- Goal: Conservative interface-affinity M transport to recover localization under coupled R22
- Status: done — D022_LOCALIZATION_NOT_RECOVERED
- Acceptance: met (honest failure conclusion with Gate1–2 evidence)
- Touched files: membrane.rs, config.rs, d022_*, main.rs, docs/d022_*, experiments/generated/d022
- Next action: Next directive must add membrane-precursor / membrane-bound component (no more seven-field loc tuning)

## Repo facts needed now
- D-021 preserved: 16213c7 / tag D-021-retention-localization-not-recovered
- D-022: v5 χ screen {0.5,1,2}×D_M; R22 M loc 0.8895–0.8907; A ret ≈0.9996
- Mimir slug: digital_cell

## Last validation
- Command: cargo test … d008–d022 release + `d022 pipeline`
- Result: tests PASS; pipeline conclusion D022_LOCALIZATION_NOT_RECOVERED

## Open blockers
- Coupled R22 M localization < 0.90 under all screened χ; Stage E blocked
