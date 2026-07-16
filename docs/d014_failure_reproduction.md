# D-014 Failure Reproduction

## Entry

| Item | Value |
| --- | --- |
| Source checkpoint | D-013 `checkpoint_150000.json` |
| Checkpoint field-hash match | yes |
| Candidate hash | `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626` |
| Configuration hash | `87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6` |
| Start accepted substeps | 150000 |
| Start simulated time | 374.9999999996952 |
| Start dt | 0.0025 |

## Reproduction result

| Item | D-013 original | D-014 replay |
| --- | --- | --- |
| Floor failure | yes | yes |
| Accepted substeps at failure | 161166 | 161165 |
| Simulated time | 402.8940165897883 | 402.8940165849259 |
| Final / min attempted dt | ~1.9×10⁻⁸ | 1.42×10⁻⁸ |
| Classification | NUMERICAL_FAILURE | reproduced |

`reproduced_near_original: true` (same candidate/config, hash match, failure in 160k–165k band, dt at floor).

## Dominant limiter

```text
FIELD_BOUND_VALIDATION
```

Terminal detail:

```text
waste_next:excessive concentration at 18335: 10.000000000099783
```

Overshoot above `CONC_SAFETY_LIMIT` (10.0): **≈ 9.98×10⁻¹¹** (machine-scale).

## Numerical-cause classification

```text
FIELD_BOUND_STIFFNESS
```

Not adaptive-controller ratcheting as the primary physical cause: the controller cascaded `dt` to the floor because every attempt produced `waste_next > 10` at a cell already sitting on the safety ceiling. Shrinking `dt` cannot clear a hard concentration validation fail of this form.

## Limiter transition

One terminal transition into `FIELD_BOUND_VALIDATION` at accepted substep 161165.

Artifact: `digital-protocell/experiments/generated/d014/failure_replay/result.json`
