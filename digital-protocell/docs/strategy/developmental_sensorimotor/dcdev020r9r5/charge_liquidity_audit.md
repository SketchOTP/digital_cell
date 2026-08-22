# DC-DEV-020-R9-R5 charge/liquidity audit

Status: observer-only diagnostic; architect acceptance pending.

This audit starts at `08e1c45b11892e0b5533b11c74f175ee84d243ed` on PR #44 and
uses ConservativeV2 with the unchanged D-091 reserve equations for exactly
5,000 accepted steps. It does not modify the production reserve kernel,
parameters, target state, controller, source, sinks, transport, or mechanics.

## Question

R9-R4 established that standing A→R storage is causally implicated but that
deferring storage after maintenance is insufficient. R9-R5 separates two
remaining explanations:

1. **Standing-stock overcharge:** storage consumes pre-existing A rather than
   positive same-step surplus.
2. **Reserve-liquidity deficit:** stored R is not available to existing
   A-dependent maintenance.

The per-step ledger records entering A/R, new A, A→C, decay, A→M, A→L,
R→A, R→W, A→R, the positive surplus cap, same-step/pre-existing attribution,
and activation-equivalent closure.

## Counterfactuals

`SURPLUS_ONLY_STORE` retains the frozen charging potential but applies

`A_to_R = min(frozen potential, positive same-step A surplus, available A, reserve room)`

after the existing productive and loss fluxes. `LIQUID_RESERVE_UB` retains
the frozen reserve ordering and permits only an instantaneous 1:1 use of
actual R for existing structural and membrane A demand, recording that use
separately. The combined arm is executed only as the bounded interaction
check.

## Result

The local observer run preserved exact 5,000-step completion and strict
activation-equivalent closure. FULL gives `R_m=0.8398695202805284` and
STORE_OFF gives `R_m=1.0180981834599838`. Surplus-only storage remains capped
and has `R_m=0.8399798978913839`; its storage cap is non-binding because the
measured positive new-A surplus exceeds the frozen store potential. The
liquid upper bound uses `0` direct diagnostic R in this trajectory, so it is
identical to FULL. The combined arm likewise does not restore the certifier.

Reserve execution remains live: replete A→R is positive, starvation R→A is
positive, rejected steps are zero, and closure passes. The actual D-087 shadow
arms retain the Gates 1–4 failure signature; Linux packaged-runtime Gate 7 is
the environment-dependent stage on the local Windows run and is rechecked by
scoped remote CI.

The fail-closed diagnostic classification is:

`DCDEV020R9R5_RESERVE_DEFECT_OUTSIDE_CHARGE_LIQUIDITY_FACTORIZATION`

This does not authorize a reserve repair, recycling/salvage, parameter or
kinetic tuning, production integration, DC-DEV-021, or any behavior work.
Dense JSONL ledgers are local/external audit output; compact protocol and
qualification records are under `experiments/generated/dcdev020r9r5/`.
