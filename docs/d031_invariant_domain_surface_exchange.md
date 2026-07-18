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


## Gate 3 — identification regression

PASS. Recovered α≈0.16698 (rel err 6e-5), β≈0.003340 (rel err 1e-5) under V2 integrator.

## Gate 4 — isolated renewal (partial)

Short diagnostic (`short_diagnostic.json`):

| Metric | Value |
|--------|-------|
| accepted steps | 6020 |
| capacity reject | false |
| Q_renewal | ≈1.45 (outside 0.98–1.02) |
| g_surface | ≈9e-4 (outside 1e-4) |
| forward / reverse / turnover | all active |
| localization | 1.0 |

Earlier full-horizon probe reached ≥10_000 accepted steps with no CapacityExceeded (contrasts D-030 accepted_in_window=0).

Full progressive horizons are running; biology judgment deferred until three qualifying windows or 200k accepted steps.

## Interim conclusion

`D031_EXPLICIT_INTEGRATION_OVERSHOOT_CONFIRMED` (Gate 0).

D-030 operative: numerical capacity integration failure; turnover–exchange incompatibility **not established**.
