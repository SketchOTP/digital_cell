# D-018 Structure Provenance Tracer

Observer-only inventories `E` (endogenous) and `K` (constraint-supplied) with `E+K = φ` under the constrained assay.

- Synthesis credits `E`
- Decay attributes W proportionally to `E/(E+K)` and `K/(E+K)`
- Positive constraint flux credits `K`
- Negative constraint flux removes proportionally from `E` and `K`

Non-causality: tracer-enabled vs disabled runs match seven field hashes, accepted substeps, and simulated time (`d018_tests`).

## Historical R22 resume (150k → 162073)

| Metric | Value |
| --- | --- |
| Endogenous fraction at 150k | 1.0000 |
| Constraint fraction at 150k | 0.0000 |
| Endogenous fraction at termination | 0.4815 |
| Constraint fraction at termination | 0.5185 |
| W from endogenous structure | 818.4635 |
| W from constraint structure | 336.6708 |
| Constraint fraction of total W (proxy) | 0.4594 |
| Constraint turnovers | 0.7384 |
| Origin class | MIXED_STRUCTURAL_WASTE |

Note: tracer initialized at resume treats pre-150k mass as endogenous; terminal K inventory share is used to avoid understating full-run constraint contamination.
