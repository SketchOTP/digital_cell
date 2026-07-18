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

## Gate 4 — isolated renewal (sealed)

**Conclusion:** `D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED`

| Item | Value |
|------|-------|
| Source commit | `f7a3dca` |
| Binary hash | `6398bc2cd7aa0be386e6ac330864dee3df76ff07a07061a960268b488d272d39` |
| Total accepted | 206000 |
| Capacity rejects | 0 |
| Consecutive qualifying windows | 0 |
| Process exit | clean (conclusion printed) |

### Window Q / g summary

| Horizon | Q (3 windows) | g (3 windows) | notes |
|---------|---------------|---------------|-------|
| 2k | 1.65 → 1.31 | +1.3e-3 → +6.1e-4 | early adsorption excess |
| 10k | 1.13 → **1.0026** | +2.5e-4 → **+5.2e-6** | one window briefly qualified |
| 25k | −3.87 → −5.38 | −9.7e-3 → −1.3e-2 | net desorption |
| 50k | −6.59 → −6.78 | −1.5e-2 | same direction |
| 100k | −7.97 → −8.08 | −1.8e-2 | same direction |
| 200k | −10.29 → −10.37 | −2.26e-2 → −2.27e-2 | three late windows, same failure |

Late failure: `Q_renewal` converges well below 0.98; `g_surface` retains a statistically nonzero negative asymptote. Not a long-transient unresolved case (not trending toward balance). Not a numerical failure (0 capacity rejects; steps_ok throughout).

Gate 5 not started.

## Terminal D-031 conclusion

`D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED`

D-030 historical conclusion confirmed under invariant-domain integration after numerical overshoot was repaired.
