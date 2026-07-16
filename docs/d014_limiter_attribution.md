# D-014 Limiter Attribution

Frozen R22 failure replay (150k → floor):

| Field | Value |
| --- | --- |
| Dominant limiter | `FIELD_BOUND_VALIDATION` |
| Failing field | `waste_next` |
| Grid index | 18335 |
| Value | 10.000000000099783 |
| Safety limit | 10.0 |
| Overshoot | ≈ 9.98×10⁻¹¹ |

Limiter counts on the reproduction run: one `FieldBoundValidation` terminal reject cascade.

Transitions: single transition into `FIELD_BOUND_VALIDATION` at accepted substep 161165.

Artifact: `experiments/generated/d014/failure_replay/result.json`
