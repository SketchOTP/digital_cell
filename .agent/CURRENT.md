# CURRENT.md

## Active directive
- ID: D-20260716-d021-retention-localization-repair
- Project directive: D-021
- Goal: Interface-protected membrane retention/localization repair then Stage E recovery
- Status: done — scientific fail
- Acceptance: met; conclusion D021_RETENTION_LOCALIZATION_NOT_RECOVERED
- Touched files: membrane.rs, config.rs, candidate_identity, d021_*, docs/d021_*
- Next action: Issue architecture directive for stronger local membrane localization; do not continue rate calibration

## Repo facts needed now
- D-020 preserved at 243e540 / tag D-020-v3-joint-rate-recovery-fail
- v4: r_M_decay = k_M_decay * M * [ε_M + (1 - I(φ))]; membrane_schema_version=2
- Gate1: all ε∈{0.02,0.05,0.10} Stage B localization PASS (~0.907)
- Gate2: all ε Stage D FIXED_COMPARTMENT_PASS
- Gate3: A/C retention ~1.0; M localization ~0.889 < 0.90; no ε promoted; Gate4/5 skipped
- D-008 Stage E remains BLOCKED_NOT_RECOVERED; Stage F not started
- Mimir slug: digital_cell

## Last validation
- Command: cargo test -p chemistry-core --release --test d021_tests --test d020_tests --test d019_tests --test d012_tests --test d011_tests --test d008_tests
- Result: PASS

## Open blockers
- Constrained R22 membrane localization below 0.90 under v4 with frozen rates; seven-field membrane bootstrap rejected for further rate search
