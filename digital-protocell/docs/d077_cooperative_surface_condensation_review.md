# D-077 — Cooperative Surface Condensation Architecture Review

## Conclusion

`D077_COOPERATIVE_COHESION_NOT_PORTABLE` (Route P)

## Records

- `ENERGY_DRIVEN_SURFACE_STATE_CYCLE_REJECTED` (from D-076)
- `PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE` (from D-075)

## Mission

Observer / reduced-model review of a Frumkin/Fowler-type cooperative P↔S exchange with local lateral cohesion χ. No production chemistry change. No Stage E claim. No Stage F.

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Start | `d82628f` / tag `D-076-nonequilibrium-surface-cycle-review` |
| D-076 | `D076_SURFACE_CYCLE_ENERGY_INFEASIBLE` |
| D-075 | endogenous p≈0.1898 → θ_eq≈0.9047; A_ret≈0.061 |

## Candidate law

\[
g(\theta)=\theta\ln\theta+(1-\theta)\ln(1-\theta)-\frac{\chi}{2}\theta^2
\]

\[
\mu_S=\ln\frac{\theta}{1-\theta}-\chi\theta,\quad
\mu_P=\ln(K_{\mathrm{eq}}p)
\]

\[
J_\chi=\delta\,k_{\mathrm{exchange}}\,q(C)\,\Gamma_{\max}
\big[K_{\mathrm{eq}}p(1-\theta)-\theta e^{-\chi\theta}\big]
\]

\[
K_{\mathrm{eq}}p=\frac{\theta}{1-\theta}e^{-\chi\theta}
\]

`χ=0` recovers the frozen linear Langmuir exchange exactly. No A is consumed for mature-S persistence. P+S conserved under exchange.

## Gate results

| Gate | Result |
|------|--------|
| 0 Lineage | **PASS** — cooperative μ-driven χ exchange not previously executed (distinct from D-022 affinity χ, linear exchange, active insertion, U/S cycle) |
| 1 Thermodynamics | **PASS** — χ=0 equivalence; P+S conservation; J follows Δμ; σ=JΔμ≥0; θ∈[0,1] invariant-domain; reject χ>4 bistable region |
| 2 Cohesion | **FAIL** — global required-χ span ≈2.35× (≤3× OK) but leave-one-out median exceeds factor-of-two: constitutive R16/22/32 need χ≈0.69–0.78 while D-071 reduced-p needs χ≈1.62 |
| 3 Metabolic | **FAIL** (secondary) — even max global χ that covers occupancy leaves measured A_ret≪0.80 (constitutive≈0.06; regulated≈0.12) |
| 4 Replacement | **PASS** at candidate eq — gross ads/des >0, near-zero net, residence within Phase 1 horizon |
| 5 Damage/starvation | **PASS** reduced ODE — 10% recovery; no-P / starvation fail; restoration resumes; no A in rates |
| 6 Stability | **PASS** for selected χ≪4 — monostable healthy branch; damage in basin; no spontaneous fill; no permanent lock after P loss |
| 7 Radius | Occupancy OK under max χ; A/C retention fail at all radii |

## Required χ (θ\*=0.95)

| State | p | χ\* |
|-------|---|-----|
| Constitutive R16 | ≈0.182 | ≈0.775 |
| Constitutive R22 | ≈0.190 | ≈0.731 |
| Constitutive R32 | ≈0.197 | ≈0.691 |
| D-071 reduced | ≈0.082 | ≈1.615 |
| Stage E θ\*=0.50 | — | cohesion not required (χ\*≤0) |

Selected diagnostic χ = max χ\* ≈1.62 (covers occupancy everywhere, fails LOO portability).

## Scientific conclusion

Local mature-S cohesion can algebraically raise equilibrium occupancy at endogenous precursor activity without continuous A consumption and without new species. It is **not** a portable architecture under the D-075 governed state family: required χ is policy-dependent (constitutive vs reduced precursor), violating leave-one-out cohesion portability. Independently, measured A/C retention remains below the 0.80 gate under every non-control precursor policy, so the candidate is also metabolically unreachable in the full Phase 1 envelope.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Production biology: unchanged

## Next directive

Do **not** implement cooperative exchange with state- or radius-specific χ. Do **not** raise precursor/activation production. Proceed to a formal decision on redesigning the Phase 1 boundary substrate rather than adding rates or species to the current P/S architecture.

`next_execution_started`: false

## Evidence

- `chemistry-core/src/d077_analysis.rs`
- `chemistry-core/tests/d077_tests.rs` (12/12)
- `experiment-runner/src/d077.rs`
- `experiments/generated/d077/`
