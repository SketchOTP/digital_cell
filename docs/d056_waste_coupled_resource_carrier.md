# D-056 — Waste-Coupled Resource Carrier Architecture

## Primary conclusion

`D056_CARRIER_KINETICS_NOT_IDENTIFIABLE`

Phase B production implementation **not authorized**.

## Phase A summary

| Gate | Result |
|------|--------|
| 0 Preservation + passive-bound reproduction | PASS |
| 1 Conservation / reversibility | PASS |
| 2 Waste-gradient capacity | PASS |
| 3 Parameter identification | **FAIL** |
| 4–5 Feasibility / shadow | not run |
| 6–14 Phase B | not started |

## Gate 0 — Preservation

- Starting commit/tag: `f9dd924` / `D-055-strict-resource-architecture-review`
- Sealed Control E χ = `0.9039035176168589` reproduced as hard bound (<1)
- Ordinary passive χ ≪ 1.05
- Recorded: `ORDINARY_PASSIVE_RESOURCE_IMPORT_BRANCH_CLOSED`
- Frozen D-051…D-055 conclusions retained; V14 remains `EXPERIMENTAL_FAILED`

## Gate 1 — Conservation and reversibility

Carrier law:

```text
J_T = k_T Γ_S [ a_z(N_o F_o) a_W(W_i) − a_z(N_i F_i) a_W(W_o) ]
```

Analytic checklist passed:

- global N/F/W conservation under ±ξ
- zero flux without S
- no inward flux without exterior N, exterior F, or interior W
- detailed-balance zero at equal activities
- exact antisymmetry under gradient reversal
- **no** `max(0,·)` rectification

## Gate 2 — Waste-gradient capacity

At diagnostic horizon 2500 (and smoke 800), for analytic / restored / R16 / R22 / R32 Control-E-class states:

- required additional paired influx ≪ measured W production + interior W inventory
- forward activity drive > 0 (internal W > exterior W at interface)
- capacity margin `1.10×` satisfied

Waste stoichiometry does **not** block the architecture.

## Gate 3 — Kinetics not identifiable

Half-saturation constants from training concentration ranges were finite and positive, but **required** `k_T★ = J_req / (Γ_S · drive)` is not portable:

| Training state | k_T★ (approx) |
|----------------|---------------|
| R16 | 0.005 |
| Control E R22 | 0.223 |
| frozen-S R22 | 0.267 |
| R32 | 0.610 |
| ordinary / low-ext | 0.946 |

- Bootstrap spread ≫ 50%
- Leave-one-out factor > 2
- Holdout median relative flux error ≫ 20%
- Starvation controls correctly predict **non-import** (reverse/zero), but healthy-state rate portability fails

Therefore: one `(K_NF, K_W, k_T)` triple cannot represent the required paired-carrier extent across the governed training/holdout domain.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`
- Selected architecture: none (V15 not promoted)
- V14 remains experimental-failed

## Deviations

- Phase A diagnostic horizon: `D056_MAX_ACCEPTED=2500` (labeled diagnostic; Gate 0 accepts χ<1 when h<10000 and sealed artifact match)
- Phase B not started (Gate 3 stop rule)
- No production chemistry / schema change

## Tests

`cargo test -p chemistry-core --test d056_tests` — 9/9 PASS

## Artifacts

`digital-protocell/experiments/generated/d056/`

## Next directive

Architecture review of **why required carrier rate is non-portable** (radius / Γ_S / drive scaling), or an alternate conservative import law that remains identifiable — **without** free pumps, A-powered import, or C★. Do not begin Stage F.
