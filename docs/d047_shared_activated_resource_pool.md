# D-047 — Shared Activated-Resource Pool Sufficiency

## Primary conclusion

`D047_HISTORICAL_ACTIVATION_FIXED_BIOLOGY_QUALIFIED`

Selected route: `ROUTE_H_HISTORICAL_ACTIVATION_FIXED_BIOLOGY`

## Preservation

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Starting commit | `bafc830` |
| Starting tag | `D-046-activated-resource-demand-audit` |
| Record | `MIXED_LEGITIMATE_A_DEMAND_CONFIRMED` |
| Historical activation | `r = 0.020 · C · N · F` |
| Schema | 3 (exchange+damage; no constitutive S→W) |
| Chemistry changed | **no** |
| `C_star` added | **no** |
| Activation law implemented | **no** |
| A pool split | **no** |

## Gate 0 — Cross-parameter portability

| Subset | Model A max | Model B max | Model C max | Aggregate adequate? |
|--------|-------------|-------------|-------------|---------------------|
| Complete (incl. altered `k_P`/`k_φ`) | 44.0% | 48.8% | 47.3% | **no** |
| Fixed biochemistry only | 32.5% | 9.5% | 3.0% | **yes** (A/B/C) |

Tag: `D047_CROSS_PARAMETER_PORTABILITY_DEFECT`

This qualifies the D-046 aggregate-model failure: Models A/B/C fail because independently altered productive-load states were included. It does not erase D-046 Route M topology facts (mixed legitimate sinks; precursor-dominant volume demand).

Altered-biochemistry states: `prec_lo`, `prec_hi`, `struct_lo`, `struct_hi`.

## Gate 1 — A-equivalent role

| Route | Role | Product dependence |
|-------|------|--------------------|
| A→C | material + activation | yes (∝ C) |
| A→φ | activation potential | yes (via I(φ)) |
| A→P | material equivalent | **no** |
| A→W decay | abstract combined | no |
| A transport | conservative relocation | no |

Shared-pool checklist: **PASS** (one local scalar currency; no incompatible activation history).

## Gate 2 — Fixed-biochemistry operating family

Constitutive rates frozen. Clamped observer demand protocol used for matched assays (forced states separated from organismal claims). Radius R16/R22/R32, environment (low/normal/high/starve N/F), init (low/high C, zero/low/healthy S), perturbation (10%/25% damage).

## Gate 3 — A lineage tracer

Noncausal proportional cohort of produced A (free A after seed; no candidate feedback).

| Destination | Fraction |
|-------------|----------|
| precursor | ~74.7% |
| structure | ~12.0% |
| reproduction | ~10.8% |
| decay | ~2.1% |
| transport out | ~0.3% |
| remaining free | ~0% |

Accounting: **PASS**.

## Gate 4 — Essential-service competition

Diagnostic activation multipliers `{1.0, 0.8, 0.6, 0.4, 0.2}`.

Class: `PROPORTIONAL_SHARED_DECLINE`

Failure-order proxy (mid-multiplier decline): structure → precursor → reproduction.

Shared decline under starvation is **not** a shared-pool structural failure.

## Gate 5 — Product self-limitation

| Sink | Class |
|------|-------|
| reproduction | self-limiting (∝ C) |
| structure | self-limiting (via I(φ)) |
| precursor | **`CONSTITUTIVE_WHILE_A_REMAINS`** |

`∂r_P/∂P = 0` at matched A,C,N,F → `PRECURSOR_SYNTHESIS_NOT_PRODUCT_REGULATED`.

Not automatically a defect: Gate 7 found constitutive precursor demand does **not** destroy the healthy reduced fixed point under frozen parameters.

## Gate 6 — Ideal shared-pool upper bound

| Control | Result |
|---------|--------|
| A healthy A | services sustained |
| B demand-replacement proxy | services sustained |
| C global mix | not distinct under uniform clamps |
| D local sufficient A | services sustained |

Result: `D047_SHARED_A_POOL_CAPABLE`

## Gate 7 — Reduced fixed-biochemistry dynamics

Observer lumped system with frozen productive parameters. Multistarts from low/healthy/high-P/low-S/pre-collapse/damaged.

- Precursor destroys healthy fixed point: **false**
- Reducing precursor restores stability: **false** (not required)

## Gates 8–10 — Candidates / shadow

Skipped: historical activation already adequate on fixed biochemistry (Route H). No production schema change. No shadow law authorized for implementation.

## Route decision

**Route H** — after excluding altered-parameter portability requirements, historical mass-action activation predicts fixed-biochemistry A demand within Gate 9 limits (Model C: median ~2.3%, max ~3.0%).

Next directive must implement **no** activation change and return to membrane-basin validation.

## Secondary conclusions

- Fixed vs altered model errors: complete fail / fixed pass (`D047_CROSS_PARAMETER_PORTABILITY_DEFECT`)
- A destinations: precursor-dominant (~75%)
- Service failure order: proportional shared decline
- Precursor self-limitation: not product-regulated; does not remove healthy fixed point
- Upper bound: shared A pool capable
- Spatial allocation: no defect established
- Reduced fixed points: healthy branch retained
- Shadow activation law: none selected

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Deviations

- Family/diagnostic horizons defaulted below full 25k for wall-clock; Gate 0 uses sealed D-046 campaign rows; re-run with `D047_FAMILY_HORIZON=25000` if stricter dynamic family evidence is required.
- Control B uses diagnostic activation multiplier as demand-replacement proxy (not exact per-step sink replacement clamp).
- Global mixing control not distinct under uniform interior clamps.

## Artifacts

`digital-protocell/experiments/generated/d047/`

## Next directive

Return to membrane-basin validation under frozen historical activation (no activation-law change). Do not implement Candidates B/C/D. Do not add `C_star` or split A.
