# D-016 waste transport implementation audit

## Frozen identity

- Equation: `membrane_metabolism_v2_conservative`
- Stoichiometric schema: 2
- Transport schema: 1 (baseline; no calibrated repair selected)
- Organism candidate hash: `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626`
- Environment: D-015 repaired `waste_sink_inner_radius = 30`

## W transport path

```text
base D_W → face average 0.5*(D_i+D_j) → × P_W(M,I) → face flux / Δx²
        → no-flux dish boundary
        → linear clearance on waste_sink_cell (r ≥ 30)
```

| Quantity | Value |
| --- | --- |
| base `D_W` | 0.25 |
| inside / outside `D_W` | 0.25 (uniform; not phase-dependent) |
| interface `D_W` at M=1 | ≈ 0.2047 |
| `β_W` | 0.20 |
| `P_W(M=0)` | 1.00 |
| `P_W(M=0.25)` | ≈ 0.951 |
| `P_W(M=0.50)` | ≈ 0.905 |
| `P_W(M=0.75)` | ≈ 0.861 |
| `P_W(M=1)` | ≈ 0.819 (≥ 0.70 gate) |
| grid spacing `Δx` | 1.0 |
| transport timestep limit | 0.0025 |
| boundary | no-flux dish |
| sink geometry | W-only sink at r≥30; N/F reservoir mask unchanged |

## Diffusivity character

`D_W` is:

- uniform across the dish
- not phase-dependent
- membrane-dependent only through `P_W`
- not shared with another soluble field
- not modified by the sink implementation

Artifact: `digital-protocell/experiments/generated/d016/transport_audit/audit.json`
