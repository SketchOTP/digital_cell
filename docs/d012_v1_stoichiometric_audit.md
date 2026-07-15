# D-012 v1 Stoichiometric Conservation Audit

**Date:** 2026-07-15  
**Branch:** `d008-membrane-metabolic-closure`  
**Equation:** `membrane_metabolism_v1`  
**Primary finding:** `D012_NONCONSERVATIVE_V1_CONFIRMED`  
**Conservation class:** `NO_POSITIVE_CONSERVATION_VECTOR`

## Scope

Exact 7×9 internal-reaction matrix for v1. Species rows: φ, C, N, F, W, A, M.
Nine governed reaction columns (reservoir, clearance, diffusion, membrane transport excluded).

## Matrix rank and nullspace

| Property | Value |
| --- | --- |
| Matrix rank | 6 |
| Left nullspace dimension | 1 |
| Strictly positive conservation vectors | none |
| Nonnegative conservation vectors | none |

Exact analysis yields a one-dimensional left nullspace, but the basis vector is not nonnegative. No strictly positive material-equivalent vector `m` satisfies `mᵀS = 0` for the actual v1 runtime stoichiometry. Classification remains `NO_POSITIVE_CONSERVATION_VECTOR`; Stage F advancement with v1 is blocked under `D012_NONCONSERVATIVE_V1_CONFIRMED`.

## Governed reaction columns (v1 runtime encoding)

| # | Reaction | Stoichiometry (actual runtime) |
| --- | --- | --- |
| 1 | Activation | N + F → A + W (C rate-modifier only) |
| 2 | Catalyst production | A → C + W (**creates net material**) |
| 3 | Structure production | A → φ (no W on productive step) |
| 4 | Membrane production | ∅ → M (synthesis adds M without A/W) |
| 5 | Structure decay | φ → W |
| 6 | Catalyst decay | C → W |
| 7 | Activated decay | A → W |
| 8 | Membrane decay | M → ∅ |
| 9 | Membrane detachment | M → ∅ |

## Nonconservative under all-ones weight

Reactions with nonzero `1ᵀS` column:

- `catalyst_production`
- `membrane_production`
- `membrane_decay`
- `membrane_detachment`

## Documented vs runtime mismatches

1. `activated_metabolism.rs` comment claims `C+N+F→C+A+W`; runtime activation consumes N+F only (C modulates rate).
2. Comment claims `C+A→2C+W`; runtime reproduction is `A→C+W` (C not consumed).
3. Membrane synthesis rate depends on A,C,φ but stoichiometric delta is ∅→M.
4. Membrane decay/detachment remove M without W product in v1.
5. Structure production consumes A without W on the productive step (constrained-radius path).

## Field ledger vs total stoichiometry

Stage-C per-field ledgers can close for activation (N,F,A,W balance individually) while catalyst production creates +1 net mass per extent under sum-of-fields check. Field accounting closure does **not** imply total stoichiometric conservation.

## D-011 branch recommendation

**Skip expensive D-011 completion.** Classify operative D-011 work as:

`D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY`

Historical quick/50k attempts remain evidence; exhaustive v1 rate-domain search is unauthorized.

## Artifacts

- JSON: `digital-protocell/experiments/generated/d012/v1_stoichiometric_audit/audit.json`
- Source: `digital-protocell/crates/chemistry-core/src/stoichiometry.rs`
- Tests: `digital-protocell/crates/chemistry-core/tests/d012_tests.rs`
