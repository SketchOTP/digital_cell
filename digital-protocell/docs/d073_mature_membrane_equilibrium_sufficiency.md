# D-073 — Mature-Membrane Equilibrium Sufficiency Audit

## Primary conclusion

`D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT` (Route E)

## Mission result

Synthetic frozen exchange remains parity-valid. D-072’s reported fixed-P control was **not actually fixed**. Target-consistent fixed-P holds at `0.9×`/`1.0×`/`1.1× p_required(0.95)` keep interface `p` within 2% of target, yet **do not** restore ≥95% of pre-damage mature mass within five local exchange timescales. Therefore D-072 Route X is **not** upheld as a pure architecture-failure proof from that control, and the organism-level blocker under analytically sufficient `p` is an **integration defect**.

## Preservation

| Item | Value |
|---|---|
| Branch | `d008-membrane-metabolic-closure` |
| D-072 commit / tag | `28dcdc4` / `D-072-membrane-damage-refill-audit` |
| D-072 original conclusion | `D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE` (retained) |
| D-072 route status | `PROVISIONAL_PENDING_EQUILIBRIUM_SUFFICIENCY_AUDIT` |
| Frozen | `K_eq≈50`, `k_exchange`, `Γ_max`, Seed B / Policy D, activation, carrier, damage, repair threshold |

## Gate 0 — Equilibrium contract

\[
\theta_{eq}=\frac{K_{eq}p}{1+K_{eq}p},\qquad
p_{required}(\theta^*)=\frac{\theta^*}{K_{eq}(1-\theta^*)}
\]

| θ* | p_required |
|---|---|
| 0.75 | ≈0.06 |
| 0.90 | ≈0.18 |
| 0.95 | ≈0.38 |
| 0.992 (D-070 lawful maintenance) | ≈2.48 |
| 0.50 (Stage E threshold) | ≈0.02 |

Independent check for `K_eq=50`: `p_required(0.90)=0.18`, `p_required(0.95)=0.38` — PASS.

## Gate 1 — D-072 fixed-P control

- Imposed concentration at t0: `1.0` (`p=1.0`)
- Spatially covered all dish cells at t0
- **Not reheld** while reactions remained enabled
- Classification: `NOT_ACTUALLY_FIXED`
- Analytically capable of θ_eq≈0.98 at t0, but not a valid sufficiency control

## Gates 2–3 — Target-consistent fixed P / damage recovery

| Control | intended p | mean interface p | recovery ratio | recovers (≥0.95) |
|---|---|---|---|---|
| 0.9× | 0.342 | ≈0.342 | ≈0.931 | no |
| 1.0× | 0.380 | ≈0.380 | ≈0.941 | no |
| 1.1× | 0.418 | ≈0.418 | ≈0.948 | no |
| D-070 maintenance | 2.48 | ≈2.48 | ≈0.979 | yes |

Hint: `D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT` (1.0×/1.1× fail despite valid holds).

## Gate 4 — Long-horizon undamaged Seed B

Constitutive occupancy `0.998 → 0.668` over ≈544 simulated time (`≈5τ` median local). Classification: `SLOW_TRANSIENT_DECAY`. A retention ≈0.056. D-070/D-071 1200-step (~0.032τ) maintenance is not equilibrium qualification.

## Gate 5 — Endogenous precursor sufficiency

- Total P mass ≈786; interface-supported ≈744; bulk ≈42
- Mean interface `p≈0.180` — sufficient for Stage E (0.50) and borderline for θ=0.90; **not** for θ=0.95
- Do not infer local sufficiency from total mass alone (here interface inventory is large, but activity still below 0.38)

## Gate 6 — Spatial delivery

Conservative redistribution of existing P did not raise interface `p` above the 0.95 requirement and did not repair damage (`recovery≈0.85`). Not classified as primary Route L.

## Gate 7 — Selected route

**Route E** — `D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT`

Secondary facts retained: D-072 control `NOT_ACTUALLY_FIXED`; long-horizon `SLOW_TRANSIENT_DECAY`; endogenous interface `p` below 0.38.

## Disposition

| Item | Status |
|---|---|
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |
| Biology / kinetics | unchanged |
| Stage F / D-009 / V15 | not authorized |

## Next directive

D-074: repair organism exchange integration defect before architecture changes.

`next_execution_started=false`

## Artifacts

`experiments/generated/d073/` (archive symlink) — preservation, equilibrium_contract, d072_control_audit, sufficient_fixed_p, damage_recovery, long_horizon_baseline, endogenous_sufficiency, spatial_delivery, radius_audit, route_selection, accounting, manifest.json
