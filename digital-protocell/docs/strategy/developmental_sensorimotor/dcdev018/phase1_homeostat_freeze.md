# DC-DEV-018 Phase 1 homeostat freeze

This is the single post-Phase-1 production wrapper authorized after the
passing source-feasibility audit. It is versioned as
`digital_cell_integral_metabolic_homeostat_v1`, default-off, and does not
import the failed DC-DEV-017 proportional adapter.

The exact parameters are derived from the committed Phase 0 audit:

```text
E_target       = 77.91027880846893
tau_integral   = 80.0
G_cap_required = 3.368462987851295
capacity_max   = G_cap_required - 1 = 2.368462987851295
k_integral    = capacity_max / tau_integral = 0.02960578734814119
```

The only adaptive state is `assimilation_capacity`. At each accepted step:

```text
E_stored = area * (A + R)
e = clamp((E_target - E_stored) / E_target, -1, +1)
capacity_next = clamp(capacity + k_integral * e * dt, 0, capacity_max)
gain = 1 + capacity_next
```

The wrapper supplies `gain` only to the existing N/F -> A activation extent.
It does not write A, R, N, F, structure, catalyst, membrane, reserve, waste,
or any behavioral state. If N or F is absent, the existing reaction clamps
produce no additional A. The legacy `reactions_step` function remains the
feature-off path and is unchanged in behavior.

Formal qualification is frozen to M0 through M4 from the directive, with
4,000-step metabolic horizons, the exact DC-DEV-016 settlement/deprivation,
the derived inventory `14.588954880632265`, and the matched precursor clamp
`0.1476710565778127`. No parameter, inventory, geometry, or horizon sweep is
authorized.
