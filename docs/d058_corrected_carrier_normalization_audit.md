# D-058 — Corrected Carrier Normalization and Re-identification

## Primary conclusion

`D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT`

**Route V** — physical surface-to-volume membrane-area capacity limit.

Normalization repair succeeded. No portable global `k_T` exists after correction. Corrected radius exponents satisfy `p_missing > p_throughput`. No V15. No production carrier.

## Invalidation

`D056_CARRIER_IDENTIFICATION_INVALIDATED_BY_OBSERVER_NORMALIZATION`

## Preservation

| Item | Value |
|------|-------|
| Start commit | `1c9d6ae73ac828622d1315e7a2137385a5ac1e71` |
| Start tag | `D-057-carrier-geometry-driving-force-audit` |
| D-056 fail commit | `ed6de2cb0ce78202a665ddc4335ca198ac79b625` |
| D-056 tag | `D-056-waste-coupled-resource-carrier-fail` |
| D-057 conclusion | `D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT` |

## Gate −1 — Workspace safety

Unrelated untracked Cursor rules identified and excluded from D-058 commits. Explicit path selection only. No destructive git ops.

## Gate 0 — D-057 reproduction

At horizon 2500:

- Defective-estimator `k_T★` span ≈ **185.03×** (matches sealed D-056)
- Wrong δ proxy = `interface_weight`
- Face measure omitted; timestep omitted
- Defective estimator preserved as regression fixture
- Conservation / W-capacity / starvation directionality PASS

## Gate 1 — Canonical face operator

[
\xi_f^{\mathrm{req}} = k_T\,\Gamma_f\,D_f\,A_f\,\Delta t
]

| Quantity | Convention |
|----------|------------|
| `Γ_f` | `reconstruct_gamma(S, cell_delta_estimate(φ), δ_floor)` face-averaged |
| `D_f` | reversible Model A drive (dimensionless) |
| `A_f` | `DX` (Cartesian face length), applied once |
| `Δt` | accepted `sim.dt` only, applied once |
| `V` | `DX²`; `Δc = ±ξ/V` |

Production δ: `max(6φ(1−φ)/DX, δ_floor)` — not `interface_weight`.

## Gates 2–3 — Corrected observer + synthetic invariance

- Observer/kernel parity PASS (`ξ = k_T · capacity_contrib`)
- Synthetic face/dt/volume/orientation/traversal/DX scaling PASS

## Gate 4 — Corrected original-model ID

| Metric | Value | Threshold |
|--------|-------|-----------|
| Corrected `k_T★` span | ≈ **194.3×** | ≤3× |
| Bootstrap spread | ≈ 0.75 | ≤0.50 |
| Hold median / max err | ≈ 0.59 / 0.79 | ≤0.20 / 0.35 |
| Direction / starvation | PASS | — |

**FAIL** — original Model A not portable after normalization repair.

## Gates 5–6 — Measures / drives

All S-derived measures and Models A–D fail ≤3× portability under corrected normalization.

## Gate 7 — Residual scaling (corrected)

| Exponent | Value |
|----------|-------|
| `p_missing` (J_missing ∝ R^p) | ≈ **7.81** |
| `p_throughput` (capacity ∝ R^p) | ≈ **1.07** |

`CORRECTED_CARRIER_SURFACE_VOLUME_LIMIT` = **true** (`p_M > p_T`; no portable global rate; normalization correct).

Prior D-057 exponents (≈4.33 / ≈1.01) are superseded by these corrected values.

## Gates 8–9 — Candidate / shadow

- Gate 8: no qualified candidate
- Gate 9: skipped

## Selected route

**Route V** — next directive reviews viable organism size or additional internally generated membrane area. Do **not** use radius-dependent `k_T`. Do **not** implement V15 from this result.

## Status

- Selected architecture: **none**
- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Artifacts

`digital-protocell/experiments/generated/d058/`

## Tests

`cargo test -p chemistry-core --test d058_tests` — 12/12 PASS.

## Deviations

- Multi-resolution production `DX` still fixed at 1.0; synthetic DX scaling covers invariance.
- Shadow not run (no Gate 8 candidate).
- Unrelated untracked `.cursor/rules/*` left uncommitted.
