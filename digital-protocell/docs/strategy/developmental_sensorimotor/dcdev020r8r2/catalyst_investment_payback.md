# DC-DEV-020-R8-R2: Catalyst Investment Payback

## Disposition

This is an observer-only audit from accepted DC-DEV-020-R8-R1 head
`d2c4f76a46f6baf7eab544847dd58c034adea156`. It does not change production
chemistry, resource delivery, reserve behavior, turnover, or organism
behavior, and it does not authorize DC-DEV-021.

The authorized classification is:

`DCDEV020R8R2_CATALYST_INVESTMENT_ACUTE_RECOVERY_BOTTLENECK`

## Frozen reconstruction

The audit reconstructs all 480 accepted R7 on-policy pre-reaction states. The
stored-material quantity is `E_AR = area * (A + R)`. Each physical root is
paired with a cloned shadow in which only `k_c_prod` is set to zero for one
counterfactual step. The normal R6 N/F power-law replay is then compared with
a whole-window cprod-deferred shadow.

Checkpoints are `1, 40, 80, 120, 160, 200, 240, 280, 320, 360, 400, 440` in
both the D016 bilinear and sealed R6 power-law source contexts. The material
ledger separates A/R store, catalyst C, structural mass, membrane mass, N/F
retained material, irreversible W, and source-delivery/loss.

## Results

All 480 normal and no-catalyst-production physical roots were valid, with zero
capacity failures and zero pre-crossing non-monotonicity failures. The median
catalyst-production burden was `0.008328836524032168` E_AR units and the
median burden/root ratio was `0.4248821456687416`. The burden was at least the
R6 source shortfall on all 480 states.

The exact normal R6 replay ended at `E_AR = 60.06203101178377`, matching the
sealed endpoint `60.0620310117838`. The deprived start was
`60.82781514212436`. Deferring catalyst production for the whole 480-step
window ended at `63.645566711951915`, above the deprived start, while normal
catalyst production ended below it. This is the acute recovery bottleneck
classification, not a production recommendation.

No checkpoint paid back within the remaining finite-feed window in either
source context. Both arms remained alive and finite. The deferred shadow
recorded no catalyst-production A consumption; all other frozen reaction and
turnover paths remained active.

## Evidence

Compact evidence is committed under
`digital-protocell/experiments/generated/dcdev020r8r2/`. The dense root,
payback, and full-shadow ledger is stored in governed external evidence
storage and referenced by `external_evidence_manifest.json`.

Prior sealed inputs are preserved by hash: R5
`4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585`, R7
`abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8`, R8
`12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff`, and
R8-R1 `f44e8f9fa441451ee40bcbfccac5f556131e4d26868868607e9507c29e7bcf90`.

Literature is methodology/context only; no external constants or production
parameters were imported.

## Boundary

Production chemistry changed: no. Production behavior changed: no.
Implementation authorized: no. DC-DEV-021 authorized: no.
Architect acceptance: pending.
