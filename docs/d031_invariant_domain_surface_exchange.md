# D-031 — Invariant-Domain Integration of Reversible Surface Exchange

## Status

In progress. Gate 0 classification recorded.

## Gate 0

| Item | Result |
|------|--------|
| D-021–D-030 preservation | D-029/D-030 tags present; D-030 result commit `921bd42` |
| Continuous boundary signs | `dP/dt≥0` at P=0; `dS/dt≥0` at S=0; `dS/dt≤0` at θ=1 |
| Discrete proposal | Explicit Euler can exceed θ=1 |
| Classification | `D031_EXPLICIT_INTEGRATION_OVERSHOOT_CONFIRMED` |

Operative D-030 qualification:

`D030_TURNOVER_EXCHANGE_INCOMPATIBILITY_NOT_ESTABLISHED_DUE_TO_ZERO_ACCEPTED_STEPS`

Historical D-030 conclusion/tag unchanged.

## Numerical schema

- Equation (unchanged): `membrane_metabolism_v8_reversible_surface_exchange`
- Exchange schema (unchanged): `2`
- Integrator: `surface_exchange_integrator_v2_invariant_domain`
- Local solve: backward Euler on `S∈[0,min(T,C_surface)]` with safeguarded bisection
- Turnover: exact `S ← S exp(−λ_Γ dt)` via Strang (½ turnover → exchange → ½ turnover)
- Frozen kinetics: α≈0.167, β≈0.00334, k≈0.00334, K≈50

## Substep order (v8 + v2)

1. Surface diffusion (+ optional advection)
2. Precursor synthesis / precursor decay
3. Half biological S→W turnover (exact)
4. Full reversible P↔S exchange (invariant-domain BE)
5. Half biological S→W turnover (exact)
