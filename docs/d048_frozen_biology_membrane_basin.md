# D-048 — Frozen-Biology Membrane Basin and Repair Validation

## Primary conclusion

`D048_NO_HEALTHY_MEMBRANE_ATTRACTOR`

Selected route: `RETURN_TO_MEMBRANE_METABOLISM_COUPLING_FULL_APS_HISTORIES`

## Preservation

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Starting commit | `3b211f9` |
| Starting tag | `D-047-shared-activated-resource-audit` |
| Record | `HISTORICAL_ACTIVATION_FROZEN_FOR_MEMBRANE_VALIDATION` |
| Historical activation | `r_A = 0.020 · C · N · F` |
| Schema | 3 (exchange+damage; constitutive S→W = 0) |
| Chemistry changed | **no** |
| Productive rates changed | **no** |
| Stage E certified | **no** |
| Stage F | not authorized |

Frozen conclusions retained: `D047_HISTORICAL_ACTIVATION_FIXED_BIOLOGY_QUALIFIED`, `D047_CROSS_PARAMETER_PORTABILITY_DEFECT`.

## Gate 0 — Preservation and candidate identity

**PASS.** Tag present; `k_d008_activation = 0.020` (not STAGE_E_FAILED_RATES 0.024); schema-3; `ρ_A = 1`; immutable candidate identity recorded under `candidate_identity/`.

## Gate 1 — Seed contract

**PASS.** Governed v7 compartment seed classified as permitted organism seed + environmental reservoirs. Zero-S is diagnostic only. No forbidden target/repair reserve.

## Gate 2 — Long-horizon healthy attractor

**FAIL (decisive at 50k accepted; full 200k not required).**

| Window | C retention | A retention | \|g\| net S | Qualifying |
|--------|-------------|-------------|------------|------------|
| 0 (10k) | 0.988 | **0.010** | 0.062 | no |
| 1 | 0.898 | **0.006** | 0.599 | no |
| 2 | 0.814 | **0.012** | 1.262 | no |
| 3 | 0.740 | **0.018** | 0.448 | no |
| 4 | 0.672 | **0.023** | 0.252 | no |

- Localization remains ≥0.95 throughout.
- Analytic seed: **fail** (A retention collapse + net S loss).
- Restored healthy state: **unavailable** (no healthy checkpoint formed; no prior provenance snapshot).
- Basin accessibility secondary: not applicable (no attractor observed).

## Gates 3–10

Not executed (stop-on-fail at Gate 2). Secondary fields recorded as `not_run_stopped_gate2`.

## Scientific conclusion

Under the exact frozen biology authorized by D-047 (historical activation, schema-3, passive exchange, productive rates frozen), the governed seeded organism does **not** reach a healthy membrane–metabolism attractor on the measured horizon. Activated resource collapses within the first 10k-step window (~1% A retention) while membrane mass undergoes large negative net exchange. This is consistent with earlier D-039 dynamic baseline A-retention failure, now re-tested with activation explicitly frozen at 0.020 rather than the Stage-E reference 0.024.

D-047 remains valid as a fixed-biochemistry supply-demand authorization for one shared A pool; it does **not** by itself establish a reachable membrane maintenance basin.

## Architecture / status

| Item | Status |
|------|--------|
| Selected architecture | none (failure route) |
| D-008 Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Next directive

Return to membrane–metabolism coupling using full A/P/S histories. Do **not** redesign activation without new contradictory evidence. Do **not** restore constitutive mature-membrane turnover automatically.

## Tests

- `cargo test -p chemistry-core --lib d048_analysis` — 8/8 PASS
- `cargo test -p chemistry-core --release --test d048_tests` — 12/12 PASS
- Pipeline Gate0–1 PASS; Gate2 FAIL @ 50k accepted

## Artifacts

`digital-protocell/experiments/generated/d048/` — preservation, candidate_identity, seed_contract, healthy_attractor, result.json, manifest.json.
