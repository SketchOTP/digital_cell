# D-069 — Mature-Membrane Exchange Equilibrium and Desorption Audit

## Primary conclusion

`D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT`

## Route

`Route_X_exchange_execution_defect`

## Preserved D-068 state

- Conclusion: `D068_MEMBRANE_DESORPTION_DOMINANT`
- Records: `PRECURSOR_SUPPLY_NOT_PRIMARY_MEMBRANE_LIMIT`, `REVERSE_MEMBRANE_EXCHANGE_CAUSE_UNRESOLVED`
- Gate 0 reproduction (R22, 1200 accepted): A→P≈425.3; accepted ads≈2.77; accepted des≈99.666; ΔS≈−96.9; S retention≈0.449; fixed healthy P does not arrest S

## Causal finding

Accepted desorption is **capacity dump of overseeded mature S**, not a portable equilibrium-kinetics failure.

| Quantity | Value |
|---|---|
| Seeded S total | 176.0 |
| Σ δ·Γ_max capacity | ≈76.333 |
| S / capacity | ≈2.306 |
| Over-capacity mass | ≈99.6666418 |
| Accepted desorption | ≈99.6663195 |
| \|des − over_capacity\| | ≈3×10⁻⁴ |
| Retention if dump to capacity | ≈0.434 |
| Observed S retention | ≈0.449 |

All 252 membrane-supported cells start above local capacity. The reversible integrator then removes the illegal excess. Fixed healthy P and FixedAllP(0.5) cannot “rescue” retention vs the seed because the seed itself is not a feasible occupancy state.

## Exchange lineage (frozen)

- Equation: `dS/dt = δ · k_exchange · q(C) · Γ_max · (K_eq · p · (1−θ) − θ)`
- `p = P / P_reference` (dimensionless activity; P_ref=1)
- `θ = S / (δ · Γ_max)` (dimensionless occupancy)
- `k_exchange = β ≈ 0.003339877461040047`
- `K_eq = α/β ≈ 50.00000000005883`
- `Γ_max = 1`
- δ applied once as interface measure; runtime accepted transfer is one signed `exchange_net` with ΔP=−ξ, ΔS=+ξ

Analytical dose response matches equilibrium identities (zero-P desorption; q(C) rate-only; zero crossing = p_eq(θ)).

## Equilibrium / timescale / feasibility (secondary)

- Initial manifold: systematically desorption-favored because θ is capped/saturated while S>capacity
- `K_eq★` not portable on the illegal oversaturated state (θ→1 ⇒ K_eq★→∞)
- Timescale: `EXCHANGE_TIMESCALE_NOT_PRIMARY`
- Material feasibility of “maintain seeded θ≈1” under current K_eq is rejected; the seed is the defect, not precursor supply

## Candidates

Stopped before kinetic qualification. No `K_eq` change and no `(k_on,k_off)` change authorized. Candidate evaluation skipped under `D069_SKIP_LATE_GATES` after Route X stop rule.

## Final disposition

| Item | Status |
|---|---|
| Selected route | Route X |
| Primary conclusion | `D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT` |
| Selected exchange law | none (frozen law retained; execution/seed contract broken) |
| Exchange-law authorization | unauthorized |
| Precursor-law authorization | unauthorized |
| Activation-law authorization | unauthorized (closed) |
| V15 | unauthorized |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Next directive

Repair the mature-S seed / capacity contract so initial interface S satisfies `S ≤ δ·Γ_max` under frozen exchange kinetics. Do not change `K_eq` or `k_exchange` to mask overseed desorption. Re-audit equilibrium only after lawful initial occupancy.

## Artifacts

`experiments/generated/d069/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d069`
