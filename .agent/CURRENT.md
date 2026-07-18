# CURRENT.md

## Active directive
- ID: D-20260717-d028-bracketed-surface-renewal-root
- Project directive: D-028
- Goal: Deterministic bracketed k_ads root solve in [0.0338, 0.0676] then revalidate through Stage E
- Status: done — `D028_ROOT_NOT_PORTABLE`
- Acceptance: One D028_* conclusion; Gate0 reproduces 1×/2× bracket; stop at first failed gate — met
- Touched files: d028_analysis, d028 runner/tests, main.rs, docs/d028_*, experiments/generated/d028, .agent/*
- Next action: Architect exchange-law revision; do not Stage F; do not productive-rate-only repair

## Repo facts needed now
- Selected isolated root k_ads=0.04867196940427757 Q≈1.0168 g≈3.36e-5; Gate2 ±2% ordered PASS
- Gate3 portability 2/6 (fixed R22 + Stage E 10k only); dynamic/late Stage E overshoot
- Additional record D027_SURFACE_BALANCE_ROOT_BRACKETED; D-027 historical conclusion unchanged
- Mimir: BLOCKED (Windows path mapping on project_register)

## Last validation
- Command: cargo test -p chemistry-core --release --test d028_tests --test d027_tests; d028 pipeline Gates0-3
- Result: d028 8/8 PASS; d027 7/7 PASS; Gate0/1/2 PASS; Gate3 FAIL (D028_ROOT_NOT_PORTABLE)

## Open blockers
- Isolated root not portable → exchange law escalation for next directive
- D-008 remains BLOCKED_NOT_RECOVERED
