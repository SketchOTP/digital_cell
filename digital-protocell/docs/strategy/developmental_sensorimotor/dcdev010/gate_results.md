# DC-DEV-010 gate results

| gate | result | evidence |
| --- | --- | --- |
| Gate 0 authority/scope | pass | one substrate module, no controller/sensor/reward/evolution |
| Gate 1 passive substrate | failed | motor-off directional arm moved `0.013504913541228361`, above `2.220446049250313e-13` |
| Gate 2 legacy parity | pass diagnostically | no-substrate active hash and regulatory trace match DC-DEV-009 |
| Gate 3 directional coupling | pass diagnostically | positive and negative axial reactions differ and remain local |
| Gates 4-8 | not qualifying | execution stopped after the first failed scientific gate; later values are diagnostic only |

Passivity was not the failed subcondition: maximum positive substrate work was
`0.0`, and every recorded substrate work value was non-positive within the
same tolerance. The failure is the motor-off no-propulsion requirement. The
substrate therefore cannot be credited with a lawful contractility-to-
translation chain.

Required conclusion:

`DCDEV010_DIRECTIONAL_SUBSTRATE_TRANSLATION_NOT_ESTABLISHED`

`NEXT_EXECUTION_STARTED:false`

## DC-DEV-010-R1

R1 failed the first new causal-isolation gate because the seeded body did not
reach the preregistered mechanical-rest contract within 5,000 accepted steps.
No R1 matched arms were executed.

Conclusion: `DCDEV010R1_BASELINE_MECHANICAL_REST_NOT_ESTABLISHED`

## DC-DEV-010-R2

| gate | result | evidence |
| --- | --- | --- |
| Gate 0 authority/scope | pass | exact R1 entry head, fixed seed/equations/horizon, observer-only |
| Gate 1 observer parity | pass | exact legacy trajectory parity; force reconstruction max error `1.2412670766236366e-16` |
| Gate 2 legacy baseline | pass | late net force approaches the preserved R1 references without substrate |
| Gate 3 isotropic control | pass | isotropic passive control also converges to near-zero net force |
| Gate 4 directional substrate | diagnostic | directional arm retains the R1-exposed late residual |
| Gate 5 attribution | diagnostic | component terms cancel; unresolved interaction, bending largest standalone term |
| Gate 6 classification | confirmed | `DCDEV010R2_DIRECTIONAL_SUBSTRATE_SPECIFIC_RESIDUAL_CONFIRMED` |
| Gate 7 preservation | pass | original, R1, Phase-1, D-088, evolution-harness, and governance checks |

Required conclusion:

`DCDEV010R2_BASELINE_FORCE_BALANCE_AUDIT_COMPLETE`

`NEXT_EXECUTION_STARTED:false`
