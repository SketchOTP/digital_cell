# Gate 5 authority profile

The complete authoritative construction is the existing
`chemistry_core::d096_allocation::pre_fission_assay` path. R2 extracted the
narrow shared constructors:

- `seed_d096_prefission_founder`
- `d096_prefission_reaction_params`
- `d096_prefission_transport_params`
- `d096_prefission_growth_params`

The profile preserves the seed-dependent vertex count `12 + seed % 3`, radius
`8.0`, center `[0, 0]`, `rho_s=1.0`, `theta_b=0.8`, `free_l=1.0`, Gate 5 lumped
chemistry, finite-allocation defaults, derived reserve parameters with
`reserve.enable=true`, transport defaults, `y_g=0.9`, growth enabled, and
`dt=0.02`. The serialized full profile is in `authority_profile.json`.

The sealed effects remain `H=0.5988859008884848` and
`B=3.811469763347633`.
