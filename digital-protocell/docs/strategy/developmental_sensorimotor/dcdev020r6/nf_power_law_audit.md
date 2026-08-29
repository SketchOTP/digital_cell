# DC-DEV-020-R6 symmetric N/F power-law observer audit

## Authority and boundary

R6 starts from architect-accepted R5 head `d215cfc00ce70517e25fa7c3b51b13d85d9ce521`. Production biology remains rooted in clean DC-DEV-016 head `1e242f28152797b512e25cd56c7b718e45d6ca97`. The candidate is counterfactual only. No production chemistry, parameters, resource law, transport, sink, mechanics, behavior, or DC-DEV-021 work changed.

The accepted R5 dense ledger was verified at SHA-256 `4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585`. It contains all 4,800 P0-P4 baseline and endpoint-break-even statewise zero-drift roots. R5's conclusions remain `DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT` and `ENDPOINT_SURROGATE_NOT_LOCAL_REQUIREMENT`; these establish one-step information sufficiency, not a qualified production law.

## Historical architecture guard

- D-043 tested scalar recalibration of the existing `C*N*F` mass-action law and failed portability. It did not change kinetic order or fit statewise R5 roots.
- D-067 tested baseline, global capacity scaling, and a bounded `q_N*q_F` low-substrate response. Its static/bounded alternatives failed durable qualification. It did not test a generalized power law.
- DC-DEV-020-R4 tested a symmetric Michaelis-style saturation denominator against endpoint-derived surrogate trajectories. R6 instead fits kinetic order against R5's true local zero-drift roots, with no saturation denominator or global multiplier.

No repository evidence showed that the exact `K_PL*N^p*F^p` family had already been tested against equivalent current-mesh statewise evidence.

## Deterministic identification

The frozen candidate is:

```text
J_PL = q_C * g_h * K_PL * N^p * F^p
```

with `g_h=1`. Positive P0-P2 states from both R5 trajectory classes were fitted by closed-form ordinary least squares:

```text
ln(S_zero / (q_C * g_h * area * dt))
  = ln(K_PL) + p * (ln(N) + ln(F))
```

No search, iterative fitting, endpoint optimization, or holdout information was used.

| Quantity | Result |
|---|---:|
| Training states | 2,880 |
| `K_PL` | `0.017556661171593057` |
| `p` | `0.0003277429681759396` |
| Training relative RMSE | `0.05968037547912959` |
| Training p95 absolute relative error | `0.13340470069304416` |

The fitted order is finite and inside the preregistered `[0,1]` interval. It is very close to the fixed-capacity limit while remaining strictly positive and exactly substrate-dependent: the implementation returns zero explicitly when either N or F is absent.

## Held-out local-root result

| Holdout | Relative RMSE | p95 absolute relative error |
|---|---:|---:|
| P3 | `0.05106673550084852` | `0.1286856886209483` |
| P4 | `0.05106673550084852` | `0.12868568862094845` |
| Combined | `0.05106673550084852` | `0.12868568862094845` |

The combined result passes the fixed `0.15` RMSE and `0.30` p95 limits. Across all 4,800 states there were zero predicted-capacity violations and zero clipping-dependent predictions. Explicit zero-substrate, N/F symmetry, finiteness, and nonnegative-request controls pass.

## Selected finite-feed result

The exact deprived state received one balanced patch with `N=F=19.878372106390554` for 480 accepted steps.

| Arm | Final `E_stored` | Paired delivered | Paired consumed | Accelerated steps | Clipping steps |
|---|---:|---:|---:|---:|---:|
| Baseline bilinear | `54.3584702923158` | `15.5438732643124` | `1.69110554535767` | 0 | 0 |
| R6 power law | `60.0620310117838` | `15.8204748677412` | `8.60613188296125` | 0 | 0 |
| Source-saturated bound | `61.6843481847883` | `16.1099691587623` | `16.1099691587623` | 480 | 0 |

R6 remained alive, finite, nonnegative, conservative, and physically bounded. Its maximum resource conservation error was zero and its maximum stored-accounting residual was `2.61683036351101e-14`. It moved A toward the settled value and reduced the A/R state distance from `0.201418619001007` to `0.183720187418416`.

However, final `E_stored=60.0620310117838` remained below the deprived start `60.82781514212436`. Gate 5 therefore failed. The law improves conversion substantially and approximates the one-step zero-drift boundary, but it does not restore stored material over the frozen finite-feed window.

## Fail-closed disposition

Classification:

`DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE`

Balanced dose robustness, sustained-fed stability, and three-cycle reversibility were not run as authoritative gates after the decisive Gate 5 failure. No restoration multiplier, additional coordinate, second exponent, second kinetic family, controller, or production integration was attempted.

## Prior-art disposition

Savageau 1969 parts I and II and Muller and Regensburger 2012 are `ADAPTABLE_ARCHITECTURE_ONLY`. They support power-law rate representation and generalized kinetic-order formalisms. No kinetic orders, rate constants, concentrations, molecular identities, or organism-specific mechanisms were imported. `K_PL` and `p` came only from Digital Cell R5 evidence.
