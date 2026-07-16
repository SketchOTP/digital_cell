# D-015 fresh reference

## Preflight (repaired env)

| Field | Value |
| --- | --- |
| Accepted substeps | 25,000 |
| Checkpoints | 10k, 25k present |
| Termination | `MAX_ACCEPTED_SUBSTEPS_REACHED` |
| Waste budget | PASS (max rel residual ≈ 2.6e-14) |
| Concentration-bound abort | none |
| Candidate / config hashes | frozen `9a452d…` / `87ff7e…` |
| `waste_sink_inner_radius` | 30.0 |

Artifact: `digital-protocell/experiments/generated/d015/preflight/`

## Fresh R22 (repaired env)

Governed run: `experiment-runner d015 fresh-r22 --repaired`

- Max 200,000 accepted substeps
- Checkpoints: 10k, 25k, 50k, 100k, 150k, 200k
- Organism frozen; environment repaired

See `digital-protocell/experiments/generated/d015/fresh_reference_r22/` and final `docs/d015_candidate_report.md` after completion.

## Fresh R22 result (repaired environment)

| Field | Value |
| --- | --- |
| Accepted substeps | 162,073 |
| Simulated time | ≈405.18 |
| Termination | `UNBOUNDED_ACCUMULATION` |
| Waste budget max rel residual | ≈ 3.2e-14 |
| Cumulative W removed at 150k | ≈ 3565 (vs ≈0.008 baseline) |
| Max W at 150k (center) | ≈ 9.46 (≈ same as unrepaired) |

Environmental repair restored exterior clearance but did **not** prevent interior ceiling hit.
