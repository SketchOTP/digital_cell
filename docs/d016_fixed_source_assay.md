# D-016 fixed-source waste transport assay

## Protocol

1. Disable biological reactions.
2. Freeze φ, C, N, F, A, M from the D-015 150k checkpoint geometry.
3. Inject the reconstructed spatial source `q_W(x,y)`.
4. Evolve only W: diffusion, membrane permeability, D-015 W sink, clearance.
5. Terminate at finite steady state, 200,000 accepted substeps, or concentration bound.

## Expected baseline behavior

Because `ΔW_center ≈ 12.7` and `D_W_required ≫ max(D_N,D_F)`, the baseline
assay must reproduce interior accumulation / concentration-bound failure
consistent with the biological R22 waste ceiling.

## Classifications

Governed labels include `FINITE_TRANSPORT_STEADY_STATE`,
`INTERIOR_DIFFUSION_FAILURE`, `CONCENTRATION_BOUND_REACHED`, and related
membrane/external failure modes.

Artifacts:

- `experiments/generated/d016/fixed_source_baseline/`
- `experiments/generated/d016/diffusivity_candidates/`
- `experiments/generated/d016/permeability_candidates/`
