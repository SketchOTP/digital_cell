# D-015 waste source decomposition

## Evidence basis

D-014 fresh R22 checkpoint at **150,000** accepted steps (`t ≈ 375`).
Executed extents from metabolism/membrane ledgers (not rate constants alone).

## Cumulative biological W production proxies

| Source channel | Amount (mass) | Notes |
| --- | --- | --- |
| Metabolism `waste_reaction_delta` | ≈ 16196.6 | Authoritative total biological ΔW |
| `waste_from_r1` (activation extent proxy) | ≈ 808.5 | Under-counts productive-yield W |
| `waste_from_r2` (reproduction extent proxy) | ≈ 181.5 | Under-counts productive-yield W |
| `waste_from_decay` bundle | ≈ 608.4 | Activated + catalyst turnover bundle |
| Membrane decay | ≈ 119.3 | Routed to W in v2 |
| Membrane detachment | ≈ 308.5 | Routed to W in v2 |
| Reservoir removed | ≈ 0.0084 | Negligible vs production |

**Dominant channel:** productive chemistry + turnover captured in `metabolism_cumulative.waste_reaction_delta` (not the incomplete legacy `waste_from_*` split).

Production rate ≈ 16196 / 375 ≈ **43.2** mass/time.

## Spatial distribution at 150k

| Region | Mean W | Max W |
| --- | --- | --- |
| INTERIOR | ≈ 7.65 | ≈ 9.49 at idx 18335 |
| INTERFACE | ≈ 4.68 | ≈ 5.39 |
| NEAR_EXTERIOR | ≈ 2.26 | ≈ 3.87 |
| BULK_EXTERIOR | ≈ 0.089 | ≈ 1.18 |
| RESERVOIR_REGION | ≈ 0 | ≈ 1e-6 |

First region to approach ceiling: **INTERIOR** (center).

## Artifact

`digital-protocell/experiments/generated/d015/source_decomposition/d014_150k_sources.json`
