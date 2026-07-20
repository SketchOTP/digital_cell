# D-045 — Fuel-Charged Catalyst Activation Cycle

## Primary conclusion

`D045_CATALYST_LINEARITY_REJECTED`

`C_star` was **not** implemented. Phase A Gate 0 failed the catalyst-linearity representability test.

## D-044 seal (Gate −1)

| Item | Value |
|------|-------|
| Result commit | `1473f0775c395e942fae7d98576d9a4640ad7ae9` |
| Tag | `D-044-activation-law-fail` |
| Tag target | `1473f0775c395e942fae7d98576d9a4640ad7ae9` |
| Record | `SINGLE_STEP_ACTIVATION_LAW_BRANCH_CLOSED` |
| Pass | yes |

Frozen conclusions preserved: `D042_ACTIVATION_CAPACITY_DEFICIT`, `D043_ACTIVATION_RATE_NOT_PORTABLE`, `D044_ACTIVATION_LAW_ARCHITECTURE_REJECTED`. Historical `k_activation=0.020`. Membrane turnover remains schema 3.

## Catalyst-demand scaling (Gate 0)

Matched `N=F=0.8` diagnostic states (not organismal steady states):

| State | R | C | L_A | M_C | V | d_C=L_A/M_C |
|-------|---|---|-----|-----|---|-------------|
| R16 | 16 | 0.80 | 97.9 | 651 | 812 | 0.150 |
| R22 | 22 | 0.80 | 177.6 | 1225 | 1528 | 0.145 |
| R32 | 32 | 0.80 | 363.5 | 2586 | 3228 | 0.141 |
| low_c | 22 | 0.30 | 144.2 | 459 | 1528 | 0.314 |
| med_c | 22 | 0.60 | 167.8 | 919 | 1528 | 0.183 |
| high_c | 22 | 1.00 | 185.5 | 1531 | 1528 | 0.121 |

Metrics:

| Check | Value | Result |
|-------|-------|--------|
| `d_C` span (C series) | 2.59× ≤ 3× | pass |
| radius `d_C` span | 1.07× ≤ 1.5× | pass (no radius bias) |
| superlinear L_A vs M_C | no (L_A span 1.29× < M_C 3.33×) | pass |
| ledger completeness | yes | pass |
| catalyst-linear max rel err | **52.7%** > 25% | **fail** |
| volume-linear max rel err | 15.0% | better fit |

**Finding:** At fixed radius, authorized demand barely tracks catalyst loading. Across true radii at matched C/N/F, `d_C` is nearly constant — demand tracks compartment size. Any catalyst-linear cycle (including QSS charged-catalyst) would reproduce the same non-portability across C levels.

## Stopped before

- Gate 1 QSS architecture fit
- Gate 2 catalyst-state mapping
- Phase B `membrane_metabolism_v13_fuel_charged_catalyst`
- Gates 3–11

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Next directive

Fundamental review of **authorized A-demand topology** (what sets productive A consumption vs catalyst mass), not another activation-supply law.
