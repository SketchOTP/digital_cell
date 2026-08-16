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
