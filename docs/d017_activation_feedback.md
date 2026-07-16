# D-017 Candidate A — coupled feedback bounds

For each α, with fixed activation extent:

- **Lower:** extra A stored in components (no immediate W)
- **Coupled:** 50% of extra A turns over to W (proxy)
- **Upper:** all extra A becomes W

At α=1:

| Bound | W source |
| --- | ---: |
| lower | 39.553 |
| coupled | 40.592 |
| upper | 41.631 |

Bounds are ordered: lower ≤ coupled ≤ upper.

`α_waste_min` does not exist on [0,1] for either lower or coupled trajectories.
`α_productive_max` formally 1.0, but **no viable waste interval**.

More A would tend to raise productive fluxes; under constrained radius this risks **more** structure-constraint W (upper bound).

Artifact: `digital-protocell/experiments/generated/d017/activation_feedback/bounds.json`
