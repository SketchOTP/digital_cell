# Repair contract

Starting from `3cd3649dc6dbb4d6a1e484f5f1578cd1124156f3`, R4 changes exactly one production law:

```text
J_repaired = J_base + J_strain * g_repair
J_base = k_build * q(C) * A * g0 * edge_length
J_strain = k_build * q(C) * A * max(g_strain(eps) - g0, 0) * edge_length
```

The condition is restricted to the D-096 finite-allocation structural-build path. No parameter, schema, candidate, heredity, mutation, mechanics, transport, reserve, growth, or fission change is included. The separate free-membrane coordinate-2 callsite remains unchanged.
