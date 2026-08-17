# DC-DEV-018 Phase 0 source-feasibility freeze

Entry authority: `1e242f28152797b512e25cd56c7b718e45d6ca97`  
Phase: 0A and 0B  
Execution: observer-only

The clean DC-DEV-016 settled body was reproduced for 5,000 mechanics steps,
then deprived for 480 metabolic steps. Exact live-code values are:

- `E_target = 77.91027880846893`
- deprived `E_stored = 60.82781514212436`
- `dt = 0.02`
- storage horizon: 4,000 steps / 80.0 time units
- matched precursor trace: `N = F = 0.1476710565778127`

The observer accounts source material as the existing N/F -> A reaction
ledger's `a_produced`. Irreversible stored-material demand is inferred A
decay plus A consumed into structure, catalyst, free membrane, R -> W, and
R -> structural material. A <-> R transfers are reported but excluded from
net demand.

Across the exact 4,000-step matched-precursor trace:

- `G_required(t) = J_irreversible_demand / J_source_legacy` remained finite.
- `G_cap_required = 3.368462987851295`.
- The hard substrate ceiling, `min(N,F) * area`, was `10.459175627198706`
  per accepted step and exceeded every observed irreversible demand.
- Gate 0A passed: a finite source-side activation capacity exists without
  suppressing sinks, changing substrate laws, or inventing a source ceiling.

The only authorized production parameter derivation is:

```text
tau_integral = 80.0
capacity_max = G_cap_required - 1 = 2.368462987851295
k_integral = capacity_max / tau_integral = 0.02960578734814119
```

No sweep, alternate controller, proportional-gain tuning, sink change, or
parameter search was performed. The complete compact machine-readable audit
is `experiments/generated/dcdev018/source_feasibility.json`.
