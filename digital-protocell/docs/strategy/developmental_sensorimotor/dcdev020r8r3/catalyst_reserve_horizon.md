# DC-DEV-020-R8-R3: Catalyst Reserve Horizon

## Disposition

This is an observer-only reserve-horizon audit from accepted R8-R2 head
`9fdd292bbd13f62ef9c88d08e8d887f15326d242`. It does not change production
chemistry, catalyst turnover, source dynamics, reserve behavior, or organism
behavior, and it does not authorize DC-DEV-021.

The result is:

`DCDEV020R8R3_CATALYST_RESERVE_SOURCE_CONTEXT_DEPENDENT`

## Frozen protocol

The accepted R8-R2 480-step normal and catalyst-production-deferred replays
were reproduced before the reserve audit. The frozen reaction parameter is
`k_c_turn = 0.01`, the frozen mechanics timestep is `dt = 0.02`, and the
diagnostic half-life is:

`H_C = ln(2) / (k_c_turn * dt) = 3465.7359027997263 accepted steps`.

The sustained horizon is `8000` accepted steps. The deterministic checkpoints,
deduplicated and sorted before execution, are:

`433, 480, 866, 1733, 3466, 5199, 6931`.

Each source context uses the established continuous precursor semantics: N and
F are set to the frozen sustained-feed value before every accepted reaction
step. NORMAL uses frozen catalyst production and turnover. DEFERRED sets only
`k_c_prod = 0` and retains frozen catalyst turnover.

## Results

Both D016 bilinear and sealed R6 power-law NORMAL and DEFERRED trajectories
remain alive, finite, and conservation-closed through all 8000 steps. Neither
whole-strategy comparison crosses, with NORMAL remaining below DEFERRED over
the governed horizon.

D016 has no marginal catalyst-investment payback at any checkpoint. R6 pays
back at `433`, `480`, `866`, `1733`, and `3466` steps, but not at `5199` or
`6931`. The R6 result therefore does not define a source-portable reserve
horizon, and conditional delayed-resume validation is not run.

The D016/R6 disagreement is the classification, not a production
recommendation. No catalyst target, deficit signal, feedback law, parameter
fit, or source modification was introduced.

## Evidence

Compact evidence is committed under
`digital-protocell/experiments/generated/dcdev020r8r3/`. The dense trajectory
and marginal ledger is stored in governed external evidence storage and is
referenced by `external_evidence_manifest.json`.

The R8-R2 input is preserved by SHA-256:

`e932f6ab96e34516de98c97c2cae102553db9764383af3d61abf015743c3a376`.

The external literature is contextual only. Wu et al. is recorded as
`ADAPTABLE_DYNAMIC_RESERVE_CONCEPT`, and Schmidt et al. as
`REFERENCE_RESERVE_TRADEOFF`. No external expression timing, concentration,
protein fraction, or environmental constant was imported.

## Boundary

Production chemistry changed: no.

Production behavior changed: no.

Implementation authorized: no.

DC-DEV-021 authorized: no.

Architect acceptance: pending.
