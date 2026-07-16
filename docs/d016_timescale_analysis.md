# D-016 timescale analysis

Using the D-015 repaired sink (`r_sink = 30`), not the historical peripheral-reservoir τ.

## Characteristic times (canonical 150k source)

| Timescale | Value |
| --- | --- |
| `τ_fill` | ≈ 381.5 |
| `τ_center_to_interface` | 484.0 |
| `τ_interface_crossing` | ≈ 4.89 |
| `τ_interface_to_sink` | 64.0 |
| `τ_sink_clearance` | 2.0 |

## Diagnostic ratios

| Ratio | Value |
| --- | --- |
| `Da_internal` | ≈ 1.27 |
| `Da_membrane` | ≈ 0.013 |
| `Da_external` | ≈ 0.168 |
| `Da_clearance` | ≈ 0.005 |

## Analytical center rise

```text
ΔW_center ≈ q_area × R² / (4 D_W) ≈ 12.69
```

exceeds available headroom to the safety ceiling.

## Required diffusivity

| Target | `D_W_required` | Authorized bound `max(D_N,D_F)` |
| --- | --- | --- |
| 0.50 × ceiling (W_interface≈2) | ≈ 1.057 | 0.18 |
| 0.90 × ceiling | ≈ 0.453 | 0.18 |

Both required values exceed the authorized small-solute bound.

Historical D-015 clue `τ_transport/t_fail ≈ 16129/405 ≈ 39.8` referred to the
**unrepaired** peripheral reservoir distance and remains preserved as motivation,
not as the repaired-geometry resistance model.

Artifact: `digital-protocell/experiments/generated/d016/timescale_analysis/timescales.json`
