# D-057 — Carrier Geometry, Normalization, and Driving-Force Audit

## Primary conclusion

`D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT`

**Route G** — discrete geometry / surface normalization defect.

No V15. No production carrier. Shadow trajectories not run (Gate 9 failed).

## D-056 seal

| Item | Value |
|------|-------|
| Commit | `ed6de2cb0ce78202a665ddc4335ca198ac79b625` |
| Tag | `D-056-waste-coupled-resource-carrier-fail` |
| Subject | `D-056: Reject nonidentifiable waste-coupled carrier kinetics` |
| Frozen | `D056_CARRIER_KINETICS_NOT_IDENTIFIABLE` |
| Unresolved | `WASTE_COUPLED_CARRIER_ARCHITECTURE_UNRESOLVED` |

Excluded from seal: unrelated `PROJECT_GOAL` UMBRA rewrite and Cursor rule migration.

## Gate 0 — D-056 reproduction

At horizon 2500:

- Sealed `k_T★` span ≈ **185×** reproduced exactly from enriched observer states
- Conservation / reversibility checklist PASS
- W-capacity / starvation directionality PASS
- Ordinary → R16 required-rate span matches sealed Phase A

## Dimensional audit (exact units)

| Quantity | Interpretation |
|----------|----------------|
| N, F, W | concentration (amount / cell; `DX² = 1`) |
| Γ_S | `S / max(δ, δ_floor)` surface density |
| Production δ | `cell_delta_estimate(φ) = max(6φ(1−φ)/DX, δ_floor)` |
| D-056 δ proxy | **`interface_weight(φ)`** (mismatch) |
| Face length | `DX = 1` — **not** multiplied into observer `J_T` |
| Timestep | adaptive accepted `dt` — **not** in observer rate vs integrated `J_missing` |
| Observer `J_T` | `k_T · M · D_net` with mixed units absorbed into `k_T` |
| `J_missing` | horizon-integrated `margin · max(L−J)` mass deficit |
| Face measure count in D-056 observer | **0** (omitted) |

Conversion chain defect: local rate → (missing face measure / dt) → integrated deficit / `(M·D_net)` → nonportable `k_T★`.

## Carrier measures

| Measure | Span (Model A) | Portable ≤3× |
|---------|----------------|--------------|
| M_A = Γ_S (iw-proxy) | ~185× | no |
| M_B = δ·Γ | ~225× | no |
| M_C = δ·θ_S | ~225× | no |
| M_D = face-assigned S | ~237× | no |

`CARRIER_SURFACE_NORMALIZATION_IDENTIFIED` = **false**. Alternate S-derived measures do not restore portability.

## Grid / interface

- `DX` frozen at 1.0 (no multi-resolution grid in codebase)
- Interface-width cut proxy shows sensitivity; primary G evidence is **δ-proxy mismatch + omitted face/dt**
- `CARRIER_GRID_NORMALIZATION_DEFECT` recorded

## Radius scaling

| Exponent | Value |
|----------|-------|
| `p_missing` (J_missing ∝ R^p) | ≈ **4.33** |
| `p_throughput` (∫ M D_net ∝ R^p) | ≈ **1.01** |

Secondary: `CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT` pending normalization repair (`p_M > p_T`). Not selected as primary because dimensions are not yet corrected.

## Drive decomposition

- Forward / reverse / net recorded per state
- Ordinary states retain positive net drive; **not** near-equilibrium cancellation of high `k_T★`
- `CARRIER_DRIVING_FORCE_NONPORTABLE` = false (as cancellation-driven)

## Activity models

All Model A–D × Measure A–D combos fail identifiability (span ≫ 3×, bootstrap ≫ 50%). Portable count = **0**.

## Families

`MULTIPLE_FAMILIES_NONPORTABLE` — radius and drive-normalized spans both large; membrane-only family small.

## Observer candidates / shadow

- Gate 9: **FAIL** (no portable candidate)
- Gate 10: **skipped**

## Selected route

**Route G** — repair carrier surface/face/`dt` normalization (use production δ); rerun D-056 Phase A observer identification.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`
- Selected architecture: none

## Artifacts

`digital-protocell/experiments/generated/d057/` — seal, preservation, reproduction, dimensional_audit, carrier_measures, grid_convergence, interface_width, radius_scaling, drive_decomposition, activity_models, state_families, observer_candidates, shadow_trajectories, route_decision, accounting, manifest.

## Tests

`cargo test -p chemistry-core --test d057_tests` — 10/10 PASS.

## Deviations

- Multi-resolution DX refinement not available (`DX=1` fixed); interface-width probed via cut proxy only.
- Shadow trajectories not executed (Gate 9 fail).
- Unrelated dirty tree (UMBRA `PROJECT_GOAL`, Cursor rules) left uncommitted.
