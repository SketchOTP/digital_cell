# D-046 — Activated-Resource Demand Topology Audit

## Primary conclusion

`D046_MIXED_A_DEMAND_TOPOLOGY`

Selected route: `ROUTE_M_MIXED_A_DEMAND_TOPOLOGY`

## Preservation

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| D-044 commit | `1473f0775c395e942fae7d98576d9a4640ad7ae9` |
| D-044 tag | `D-044-activation-law-fail` |
| D-045 commit | `41f9b75` |
| D-045 tag | `D-045-fuel-charged-activation-fail` (not erased) |
| Record | `FUEL_CHARGED_CATALYST_NOT_AUTHORIZED` |
| Historical activation | `r = 0.020 · C · N · F` |
| Schema | 3 (no constitutive S→W) |
| Chemistry changed | **no** |
| `C_star` added | **no** |

## Gate 0 — D-045 threshold provenance

| Fact | Result |
|------|--------|
| Issued Gate 0 checks | `d_C` span ≤3×, no radius bias, no superlinear, ledger complete |
| `25%` fit threshold in issued directive | **no** |
| `D045_CATALYST_LINEAR_MAX_REL_ERR=0.25` in source before campaign | **yes** (same implementation commit) |
| Provenance | `IMPLEMENTATION_BEFORE_EVIDENCE` |
| Status | `D045_CATALYST_LINEARITY_REJECTION_PROVISIONAL` |

Not implementing `C_star` remains correct. Provisional status only means D-046 must establish topology under explicit prospective criteria (done below).

## Gate 1 — A-demand lineage

Complete catalog under v8 / schema-3:

| Sink | Equation | Spatial weight | Stoichiometry |
|------|----------|----------------|---------------|
| `L_rep` | `k_rep · C · A` | bulk | A → η_C C + (1−η_C) W |
| `L_structure` | `k_φ · A · I(φ)` (default) | interface | A cost from φ production |
| `L_precursor` | `k_P · A · q(C) · H(φ)` | interior | A → P |
| `L_membrane` | constitutive | n/a | **0** under schema 3 |
| `L_decay` | `k_dec · A` | bulk | A → W |
| `L_transport` | selective A faces | interface | conservative |
| `L_other` | reservoir / numerical | — | correction only |

Unclassified residual ≤ tolerance.

## Gate 2 — Runtime / ledger parity

| Check | Result |
|-------|--------|
| Clamped sink decomposition residual | **0** (PASS) |
| Unclamped accepted-step ledger closure | residual ~1e−14 (PASS) |
| Double counting | none found |
| Clamp-injection ΔA artifact | excluded (not a chemistry defect) |

## Gate 3 — Constraint contamination

D-045 / D-046 demand assays use `enforce_structure_constraint=false` and activity clamps as **observer measurement**, not artificial productive replacement. Campaign classified **valid** for topology inference.

## Gate 4 — Prospective scaling campaign

Preregistered train/hold before run. Families: radius (R16/22/32), catalyst (0.3/0.6/1.0), structural load (0.5×/2× `k_φ`), precursor load (0.5×/2× `k_P`), membrane (low / healthy / 25% damaged). **13/13** states measured.

## Gate 5 — Elasticities (total and dominant sinks)

| Sink | ε_C | ε_V | Class |
|------|-----|-----|-------|
| total | 0.21 | 0.95 | interior-volume-scaled |
| precursor | 0.16 | 1.00 | interior-volume-scaled |
| reproduction | 1.00 | 1.00 | catalyst-scaled |
| structure | ~0 | 0.52 | mixed / weak interface |
| decay | ~0 | 1.00 | interior-volume-scaled |

Leave-one-out stable for total volume elasticity.

## Gate 6 — Productive yield

Reproduction, structure, precursor: `VALID_PRODUCTIVE_COST` (unit A extents). Decay: `VALID_MAINTENANCE_COST`. No duplication / unsupported stoichiometry.

## Gate 7 — Sink isolation

Largest total and persistent sink: **precursor** (~76% of L_A at R22). Disabling precursor drops demand sharply; reproduction disable removes catalyst-linear component. Largest ≠ defective.

## Gate 8 — Demand models (held-out)

| Model | median err | max err | adequate |
|-------|------------|---------|----------|
| A catalyst-linear | 2.4% | 44% | no |
| B volume | 10.9% | 49% | no |
| C saturating-volume | 8.2% | 47% | no |
| D mechanistic sink sum | 0% | 0% | **yes** |

Max errors driven by precursor-load holdouts (`prec_hi`): changing `k_P` changes demand without changing C/N/F/V.

## Gate 9 — Supply-basis feasibility

Zero-C/N/F controls PASS. Best aggregate basis among A/B/C still fails max held-out ≤35% when precursor load varies. No observer feedback / target occupancy in bases.

## Route decision

**Route M** — all sinks causal and correctly accounted; total demand predominantly volume-scaled with saturating catalyst response; yet **no single local activation basis** predicts combined demand across independent precursor/structural load variation.

## Secondary findings

- D-045 linearity rejection: **provisional** (threshold not directive-preregistered)
- Dominant sink: **precursor** (`A · q(C) · H(φ)`)
- Radius scaling: ~size-linear (ε_V ≈ 0.95)
- Catalyst scaling: weak / saturating (ε_C ≈ 0.21; M_C 3.33× → L_A 1.29×)
- Structural load: weak effect on total L_A
- Precursor load: strong, near-linear in `k_P`
- Best valid supply basis: none aggregate-adequate; mechanistic sink sum explains topology

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Next directive

Review whether one shared A pool is structurally sufficient given mixed legitimate demands (volume-distributed precursor + catalyst-linear reproduction + interface structure). Do **not** automatically add another energy species. Do **not** implement `C_star` from D-045.

## Artifacts

`digital-protocell/experiments/generated/d046/`
