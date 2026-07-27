# D-096 reciprocal pre-fission effect

Gate 4 passed: the frozen H pulse, B damage, and neutral baseline produce
distinct local resource/damage states before fission, while organism snapshots
contain no treatment identity.

Gate 5 failed under eight paired seeds, mutation disabled, fission disabled, and
1,000 synchronized steps per run.

- H processing-heavy minus repair-heavy reserve change: `0.0` for 8/8 pairs.
- Neutral counterpart: `0.0` for 8/8 pairs.
- B repair-heavy minus processing-heavy retained material: mean `2.066658`.
- Neutral counterpart: mean `1.877086`.

The B material effect exceeded neutral, but the mandatory H processing advantage
was absent. Exact conclusion: `D096_PROCESSING_ADVANTAGE_NOT_ESTABLISHED`.
No later gate ran and no parameter was tuned after observing this result.

Evidence: `experiments/generated/d096/reciprocal_prefission/attempt_001/result.json`.
