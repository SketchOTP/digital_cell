# D-015 sink capacity

## Linear law capacity

For `dW/dt = k (W_target − W)` with `W_target = 0`:

```text
clearance_rate = k * W * V_sink
W_eq = P / (k * V_sink)   # if production P is delivered into the sink
```

## D-014 baseline (peripheral annulus only)

| Quantity | Value |
| --- | --- |
| Observed biological production rate P | ≈ 43.2 mass/time |
| Reservoir cells V | 2692 |
| k | 0.5 |
| Predicted eq W if delivered | ≈ **0.032** |
| Max clearance at W=10 | ≈ 13460 |
| Clearance margin (max / P) | ≈ **312** |
| Observed delivery to reservoir | ≈ 0 |
| Observed clearance rate | ≈ 0 |

**Classification:** `TRANSPORT_TO_SINK_LIMITED` (not capacity-limited once delivered).

## After W-only repair (`waste_sink_inner_radius = 30`)

Sink region overlaps near-exterior where W already accumulates under baseline export,
so clearance engages without changing N/F supply geometry.
