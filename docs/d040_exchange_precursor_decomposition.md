# D-040 — Exchange–Precursor Coupling Decomposition

## Mission

Determine why schema-3 v8 membrane loses occupancy and fails repair despite conserved interfacial substrate, validated reversible exchange, and no unsupported constitutive S→W. Diagnostic only: no rate, field, or reaction changes.

## Frozen evidence

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Commit | `0df0195` |
| Tag | `D-039-membrane-maintenance-qualification` |
| Architecture | `membrane_metabolism_v8_reversible_surface_exchange` |
| Turnover | `surface_turnover_schema_3_exchange_damage_only` |
| α / β / K | ≈0.167 / ≈0.00334 / ≈50 |
| Record | `SCHEMA3_V8_MAINTENANCE_COUPLING_FAILED` |
| Prior | `D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED` |

## Equations

\[
p = P / P_{\mathrm{reference}},\qquad
\theta_{\mathrm{eq}} = \frac{Kp}{1+Kp},\qquad
J_{\mathrm{predicted}} = \alpha q(C)\,p(1-\theta) - \beta q(C)\,\theta
\]

Required activity for target occupancy: \(p = \theta / (K(1-\theta))\).

At \(K\approx 50\): \(p(0.25)\approx0.0067\), \(p(0.50)\approx0.020\), \(p(0.75)\approx0.060\), \(p(0.90)\approx0.180\).

## Pipeline results (horizon `D040_MAX_ACCEPTED=2000`)

| Gate | Result |
|------|--------|
| 0 Preservation / observability | PASS — tag/kinetics/D-039 artifacts; window budgets recorded |
| 1 Exchange-equilibrium audit | `EXCHANGE_LAW_PARITY_PASS_PRECURSOR_BELOW_EQUILIBRIUM` |
| 2 Chronology | `A_PRODUCTION_DECLINE` (earliest predictive event) |
| 3 Precursor sufficiency | `PASSIVE_EXCHANGE_CAN_REPAIR_WITH_SUFFICIENT_PRECURSOR` (min \(p\approx0.020\)) |
| 4 Endogenous capacity | `synthesis_capacity_sufficient` (max endogenous \(p\approx0.063\) with exchange off) |
| 5 Causal controls | P clamp, A clamp, perm freeze, no-decay, no-leak all retain mid occupancy on short horizon |
| 6 Damage controls | Strongest single control: **fixed healthy A** (repair fraction ≈0.92) |
| 7 Reduced APS model | Healthy fixed point exists; bistable basins |
| 8 Multistart | `split_healthy_failed_attractors` |
| 9 Route | **Route F** |

### Primary conclusion

`D040_MEMBRANE_METABOLISM_BISTABILITY`

### Interpretation

The frozen reversible exchange law agrees with equilibrium theory; organism trajectories sit below \(\theta_{\mathrm{eq}}\) when exchange-local precursor is insufficient. Fixed external precursor at the \(\theta=0.5\) isotherm activity restores passive exchange toward equilibrium, so the exchange law itself is not invalidated.

Endogenous precursor chemistry can exceed the Gate-3 repair threshold when exchange is disabled, so a pure synthesis-capacity deficit is not indicated. Full-system multistart splits into healthy and failed attractors; reduced feedback admits a healthy fixed point with bistable basins. Membrane–metabolism feedback (A decline → P decline → desorption → permeability leak) therefore dominates the D-039 collapse mode.

Damage repair is most strongly restored by clamping healthy A (upstream of P synthesis), consistent with chronology `A_PRODUCTION_DECLINE`, but route selection prefers bistability because a healthy attractor exists and basin access fails under autonomous coupling.

## Artifacts

`digital-protocell/experiments/generated/d040/`

## Status constraints

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Next directive (authorized)

Address local bootstrap and basin accessibility without target feedback or state resets. Do not alter the validated passive exchange law solely for supply failure.

## Governing principle

Do not replace the exchange law merely because the organism fails to supply enough precursor for that law. First test whether sufficient precursor would maintain and repair the membrane; then locate why endogenous production or retention fails — here, coupling/basin access, not law invalidity.
