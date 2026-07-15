# D-012 Conservative Network (`membrane_metabolism_v2_conservative`)

## Status

**Conservation gate: PASSED** (Task 10, 2026-07-15)

V2 unit-yield internal chemistry is strictly conservative under the all-ones material weight.
Stage B–E spatial experiments remain **blocked** until Task 11+.

## Scientific non-equivalence to v1

| Topic | v1 (`membrane_metabolism_v1`) | v2 (`membrane_metabolism_v2_conservative`) |
| --- | --- | --- |
| Stoichiometric schema | `1` — `NO_POSITIVE_CONSERVATION_VECTOR` | `2` — `STRICTLY_CONSERVATIVE` |
| Candidate hashes | Historical D-008/D-011 evidence only | Not comparable to v1 hashes |
| Snapshot resume | v1 snapshots inspectable | v1→v2 resume **rejected** |
| Productive catalyst | `A → C + W` (net material creation) | `A → η_C C + (1−η_C) W` |
| Structure (constrained radius) | `A → φ` (no W on productive step) | `A → η_φ φ + (1−η_φ) W` |
| Membrane synthesis | `∅ → M` (rate depends on A but no A consumption) | `A → η_M M + (1−η_M) W` |
| Membrane decay/detachment | `M → ∅` | `M → W` |
| D-011 expensive branch | Superseded (`D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY`) | N/A |

## Governed v2 reactions (unit yield η=1)

Column order matches `stoichiometry::ReactionId`:

1. **Activation:** `N + F → A + W`
2. **Catalyst production:** `A → C` (η_C=1 ⇒ no W branch)
3. **Structure production:** `A → φ`
4. **Membrane production:** `A → M`
5. **Structure decay:** `φ → W`
6. **Catalyst turnover:** `C → W`
7. **Activated decay:** `A → W`
8. **Membrane decay:** `M → W`
9. **Membrane detachment:** `M → W`

Lower permitted yields (`17/20`, `7/10`) route the unconverted fraction to `W` and remain conservative.

## Identity

- `equation_version`: `membrane_metabolism_v2_conservative`
- `stoichiometric_schema_version`: `2`
- `field_schema_version`: `seven_field_v1` (unchanged)
- Yield params: `eta_c`, `eta_phi`, `eta_m` with `0 < η ≤ 1`

## Artifacts

- Matrix audit: `digital-protocell/experiments/generated/d012/v2_stoichiometric_matrix/audit.json`
- Accounting spec: `digital-protocell/experiments/generated/d012/accounting/ledger_spec.json`
- Tests: `digital-protocell/crates/chemistry-core/tests/d012_tests.rs` (36 tests)

## Production verdict

`REQUIRES_REMEDIATION` — conservative Stage E not yet run.
