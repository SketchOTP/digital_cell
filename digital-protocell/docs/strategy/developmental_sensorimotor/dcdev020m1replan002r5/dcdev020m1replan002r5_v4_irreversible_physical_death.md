# DC-DEV-020-M1-REPLAN-002-R5

## Fixed-checkpoint no-reset physical-death qualification

This is an observer-only qualification from accepted R4 head
`9f4d6c34e88a613b0bf677f9f2aa25f8854edbb5`. It replays the frozen
`MaturationCoupledV4` starvation path and restores the exact accepted R1
finite N/F source schedule at S0 through S4:

| Checkpoint | Meaning |
| ---: | --- |
| S0 / 480 | accepted R1 deprived entry state |
| S1 / 5277 | first `A < 0.05` |
| S2 / 6130 | first observer-nonviable state |
| S3 / 10200 | deep collapse |
| S4 / 150200 | late collapse |

The S0 state is loaded directly from the accepted R1 entry fixture. S1--S4
are cloned from one 150,000-step starvation ledger. Refeeding runs for 8,000
steps with the fixed R1-compatible finite reservoir (`243.14924801053778` N
and F each, boundary concentration `2.063914918930895`, center `[4.8, 0.0]`,
radius `1.5`, and no replenishment). No geometry, chemistry, lifecycle,
viability, or alive state is reset. A death latch is not allowed to prevent
physics from advancing.

## Bounded result

The diagnostic records positive source opportunity for all checkpoints. S0,
S1, and S2 recover under the fixed refeed. S3 and S4 do not recover despite
positive delivery and continued physics, and their refeed runs record the
no-latch proof. This is useful evidence about deep-collapse non-recovery.

The starvation continuation currently has a maximum strict-material closure
residual of `0.45051928554230614`, above the unchanged `1e-8` accounting
tolerance. Refeed closure is within tolerance. Because the qualification
requires closure over the complete starvation/refeed package, the result is
fail-closed and remains:

`M1_V4_DEATH_QUALIFICATION_UNRESOLVED`

This report does not establish irreversible physical death, select V4, close
M1, or authorize a successor experiment. The dense starvation and refeed
ledgers belong in the governed Atlas evidence root; only compact JSON is kept
in the repository for CI and audit.

## Preservation boundary

The R5 package does not change V4 biology, historical contracts, production
selection, D-087 thresholds, mechanics, transport, reserve, recycling,
salvage, or M2 behavior. Architect acceptance remains pending.
