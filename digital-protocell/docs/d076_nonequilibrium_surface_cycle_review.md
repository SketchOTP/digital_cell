# D-076 — Nonequilibrium Surface-State Cycle Architecture Review

## Conclusion

`D076_SURFACE_CYCLE_ENERGY_INFEASIBLE` (Route E)

## Record

`PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE`

## Mission

Observer / reduced-model review of a conservative energy-driven surface-state cycle:

```text
bulk P
  ↕ passive exchange (frozen D-030)
surface U
  ↓ U+A→S+W  (consumes A)
mature S
  ↓ S→U      (conservative relaxation)
surface U
```

No production chemistry change. No Stage E claim. No Stage F.

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| D-075 seal | `983c01f` / tag `D-075-exposure-gated-membrane-audit` |
| D-075 conclusion | `D075_FROZEN_EXCHANGE_METABOLICALLY_UNREACHABLE` |
| Endogenous interface p | ≈0.1898 |
| Passive θ_eq(p) | ≈0.9047 (< 0.95 contract) |
| Constitutive A retention | ≈0.061 |

## Candidate equations

\[
J_{PU}=\delta k_{\mathrm{exchange}}q(C)\Gamma_{\max}\big[K_{\mathrm{eq}}p(1-\theta_{\mathrm{total}})-\theta_U\big]
\]

\[
J_{US}=k_{\mathrm{mature}}q(C)\,a\,U
\]

\[
J_{SU}=k_{\mathrm{relax}}S
\]

\[
\theta_{\mathrm{total}}=\frac{U+S}{\delta\Gamma_{\max}}\le 1
\]

Functional permeability depends only on mature \(S\).

## Gate results

| Gate | Result |
|------|--------|
| 0 Lineage | **PASS** — D-032/034/037/038/039/075 never executed conservative `S→U` (D-034 used `S→W`) |
| 1 Conservation | **PASS** — P+U, U+S, A↔W ledgers; capacity; zero-A/P causality; no observer in rates |
| 2 Fixed point | Algebraic θ_S≥0.95 fixed points exist at endogenous p when \(r=k_m q a/k_r\) is large enough (\(r^*\approx21\)), with locally stable Jacobian — **but** A/C retention gates fail under measured collapse |
| 3 Energy budget | **FAIL** — maturation A demand > sustainable surplus (surplus≡0 while A_ret≈0.06) |
| 4 Parameters | ≤5 global \((k_m,k_r)\) from \(r^*\) + replacement horizon; surface span/LOO OK; no fully qualifying set |
| 5 Damage/starvation | Surface ODE shows no-A / no-P / starvation decline; recovery limited by collapsed a |
| 6 Route | **Route E** |

## Fixed-point algebra (summary)

At fixed \((p,a,q)\):

\[
\theta_S=\frac{r K_{\mathrm{eq}}p}{1+K_{\mathrm{eq}}p(1+r)},\quad r=\frac{k_{\mathrm{mature}}q a}{k_{\mathrm{relax}}}
\]

Decoupling mature S from passive P↔S equilibrium is algebraically possible: immature U can sit at low occupancy while S is high. The metabolic price is \(J_{US}=k_{\mathrm{relax}}S\) continuous A consumption for replacement.

## Energy failure

Measured D-075 constitutive free-A retention ≈0.061 already violates the ≥0.80 retention gate **before** adding membrane maturation. Any positive replacement-rate maturation sink recreates / deepens the D-075 A collapse. Therefore the cycle can hold contract θ_S only by demanding A the organism cannot sustain.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Production biology: unchanged

## Next directive

Do **not** implement this cycle. Broader Phase 1 boundary-architecture review before further rate or species additions.

## Evidence

- `chemistry-core/src/d076_analysis.rs`
- `chemistry-core/tests/d076_tests.rs` (10/10)
- `experiment-runner/src/d076.rs`
- `experiments/generated/d076/`
