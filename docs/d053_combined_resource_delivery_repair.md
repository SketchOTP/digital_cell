# D-053 — Combined Exterior and Membrane Resource-Delivery Repair

## Governed seal (D-054 Gate −1)

| Item | Value |
|------|-------|
| Source commit | `76c0898e297b0abf04362df3e848e32c9d228b15` |
| Source subject | `D-053: Add combined exterior and membrane resource delivery` |
| Result tag | `D-053-combined-resource-delivery-fail` |
| Architecture record | `V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED` |
| Exhaustion record | `BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED` |

**V14 is an experimental failed candidate.** It is not selected, not qualified, and not a production default.

## Corrected primary conclusion (sealed source rerun)

`D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND`

Failed gate: `gate5_short_screen` (no candidate admitted under the sealed Gate 5 contract).

### Provenance divergence

| Run | Primary | Gate 5 | Selected pair |
|-----|---------|--------|---------------|
| Informal pre-seal | `D053_NO_HEALTHY_RESOURCE_REPAIRED_ATTRACTOR` | PASS (labeled) | `m_ext=4.0`, `m_beta≈0.5776` |
| Sealed source `76c0898` | `D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND` | FAIL | none |

Upper-bracket campaign metrics are **identical** between informal and sealed runs (`a≈0.0901`, `χ≈0.403`, `chi_rise=true`, `a_rise=false`), but sealed Gate 5 requires:

`capacity || a_rise || (chi_rise && a_retention ≥ 0.5)`

so χ-rise alone does not admit. The informal artifact labeled the same metrics `pass=true`, implying a harness/binary mismatch with the committed source.

## Informal maximum candidate (not selected by sealed Gate 5)

| Parameter | Value |
|-----------|-------|
| `m_ext` | `4.0` |
| `m_beta` | `0.5776226504666211` |
| `Π_N = Π_F` | `0.50` (Stage A ceiling) |

Informal dynamic Gate 9 @10k (frozen pair, not a sealed selection): A retention ≈0.047, `χ≈0.47`.

## Informal Gate 8 threshold audit (secondary)

Gate 8 was reported PASS with `short_horizon_relaxed=true` and measured:

| Radius | `χ_N≈χ_F` | Meets stated `χ≥1.05`? |
|--------|-----------|-------------------------|
| R16 | ≈0.531 | no |
| R24 | ≈0.376 | no |
| R32 | ≈0.290 | no |

Stated contract `χ_N,χ_F≥1.05` was **not** met; short-horizon relax used `χ≥0.20` and `A≥0.15`.

## Authorization / freeze

- Frozen: `D051_RESOURCE_THROUGHPUT_LIMIT`, `D052_MIXED_RESOURCE_DELIVERY_LIMIT`
- Schema-2 activation frozen (`V_A≈0.1254`, `K_C=0.10`, `N_ref=F_ref=1`)
- Do not raise `m_ext` above 4, reduce `m_beta` below authorized bound, or promote N/F Π above 0.50 as production defaults

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Tests

`cargo test -p chemistry-core --test d053_tests` — 12/12 PASS (on source commit)

## Next

D-054 stopped at provenance divergence. Next directive must repair the D-053 validation harness (Gate 5 admission alignment; Gate 8 χ≥1.05 without silent short-horizon weaken) and rerun D-053 before any architecture selection.
