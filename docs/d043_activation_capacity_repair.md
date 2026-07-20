# D-043 — Activation-Reaction Capacity Repair

## Mission

Audit the conservative activation reaction and determine whether a bounded recalibration of `k_activation` can eliminate the persistent activated-resource production deficit identified by D-042.

## Frozen starting state

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Commit | `6c7328f` |
| Tag | `D-042-activation-buffer-feasibility` |
| Architecture | v8 reversible surface exchange |
| Turnover | schema 3, damage only |
| A transport | historical `ρ_A = 1` |
| Passive exchange | frozen α, β, K |
| Record | `ACTIVATION_BUFFER_BRANCH_CLOSED` |

## Exact activation equation (production)

Extracted from `activated_metabolism.rs` — not an audit approximation:

```text
B_activation(C, N, F) = max(0,C) · max(0,N) · max(0,F)
r_activation          = k_d008_activation · B_activation
```

- No interior weighting on activation.
- No saturation on activation.
- Stoichiometry: `N+F → A+W` (extent Δ: N−, F−, A+, W+).
- Historical rate: `k_d008_activation = 0.020` (`default_k_d008_activation` unchanged).

## Gate results

| Gate | Result |
|------|--------|
| 0 D-042 reproduction | **PASS** @ 25 000 accepted — ∫R_A ≈ −760; `A_PRODUCTION_DECLINE` earliest; ledger closes; exchange parity |
| 1 Activation parity | **PASS** — observer / runtime / stoichiometry / activation-potential / zero-C/N/F |
| 2 Capacity decomposition | **RATE_CAPACITY** @ diagnostic 3000 — healthy N/F/C and demand/decay disables do not close sustained A deficit |
| 3 Portable rate reconstruction | **FAIL** — span ≈ 3.38× → `D043_ACTIVATION_RATE_NOT_PORTABLE` |
| 4–9 | Not started (stop-on-fail) |

### Gate 3 evidence (sealed diagnostic, `D043_DIAGNOSTIC_HORIZON=3000`)

Domain-total basis `B = j_activation / k` (production flux). Authorized loss `L_A` from early windows under A clamp `0.5` (surface not frozen). Relative basis floor excludes `low_nf` cubic product collapse.

| State | Valid | B | L_A | k_required |
|-------|-------|------|------|------------|
| R16 | yes | 450 | 168 | 0.373 |
| R22 | yes | 784 | 178 | 0.226 |
| R32 | yes | 1240 | 186 | 0.150 |
| low_c | yes | 294 | 144 | 0.491 |
| med_c | yes | 588 | 168 | 0.285 |
| high_c | yes | 980 | 186 | 0.189 |
| low_nf | dominated | 111 | 178 | — |
| high_nf | yes | 1224 | 178 | 0.145 |

- valid estimates: 7 (≥6)
- span: **3.38×** (>3× limit)
- LOO median deviation: 0.13 (≤0.50)
- median k (if forced): 0.226

Failure mode is **span**, driven by catalyst/product dependence: low-C states require ~3–3.4× the k of high-N/F peers while A-driven demand does not scale down proportionally.

## Primary conclusion

`D043_ACTIVATION_RATE_NOT_PORTABLE`

## Selected architecture

None. Do **not** select `MEMBRANE_ARCHITECTURE_V8_SCHEMA3_RECALIBRATED_ACTIVATION`.

## Scientific conclusion

D-042 correctly identified a persistent activation-production shortfall. D-043 shows the shortfall is **not** repairable by a portable scalar recalibration of the existing mass-action law `r = k·C·N·F`. Increasing `k` alone would either under-serve low-C states or over-drive high-C / high-N/F states. Next work must review activation saturation, catalyst normalization, or reaction topology — **not** raise `k_activation` inside D-043.

## Status constraints

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`
- Historical `k_d008_activation = 0.020` unchanged

## Tests

`cargo test -p chemistry-core --test d043_tests --release` — 18/18 PASS.

## Artifacts

`digital-protocell/experiments/generated/d043/` — preservation through rate_reconstruction, decision, manifest.

## Tag

`D-043-activation-capacity-fail`

## Next directive

Activation-law review: saturating / catalyst-normalized activation, or revised reaction topology. Do not raise historical `k_activation` without a portable law. Do not begin Stage F. Do not declare Stage E recovery.
