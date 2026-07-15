# D-013 Activation-Potential Accounting

Schema version: `activation_potential_schema_version = 1`

Weights: `e_F = 1`, `e_A = 1`.

Every accepted step, checkpoint, window sample, and final governed result must include the activation-potential ledger:

- initial activation potential
- fuel reservoir contribution
- activation transfer
- productive consumption
- turnover dissipation
- waste-associated potential
- numerical correction
- final activation potential
- residual / relative residual

Physical directionality:

- closed chemistry cannot increase total activation potential
- fuel reservoir input may increase available potential
- activation transfers potential from fuel substrate into A
- productive chemistry consumes A potential
- turnover and waste formation do not create potential
- waste cannot spontaneously become A or F

A result missing this ledger is automatically `INVALID_ARTIFACT`.
