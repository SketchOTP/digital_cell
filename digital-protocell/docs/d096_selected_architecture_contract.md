# D-096 finite-budget catalytic allocation contract

Status: `FROZEN`; implementation: `NOT_IMPLEMENTED`.

## Hypothesis and identity

Inherited finite catalytic allocations favor pulse processing under temporally
concentrated nutrient/fuel and favor repair under recurrent local damage,
creating reciprocal environment-specific descendant advantage.

Equation identity:
`autopoietic_material_mesh_finite_catalytic_allocation_v1`. Frozen D-087 through
D-095 equations are not modified.

## Heredity and expression

The ordered inherited simplex encodes resource processing, activation, repair,
and growth synthesis. Each value is in `[0,1]` and their sum is exactly one.
Initial pulse, damage, and neutral allocations are `[0.45,0.25,0.10,0.20]`,
`[0.20,0.20,0.45,0.15]`, and `[0.25,0.25,0.25,0.25]`.

At each qualified copy, mutation occurs with probability `0.01`. An ordered
source/target pair is chosen uniformly and
`δ=min(|Normal(0,0.15)|, source, 1-target)` is transferred between coordinates.
The vector is copied exactly before mutation; fission uses the qualified
conservative network partition. Identity hashes canonical ordered IEEE-754
allocation bytes with the equation identity.

For accepted step `dt`, shared synthesis is
`J_syn=1e-3 min(M_local,A_local/0.2)` and `J_i=α_i J_syn`.
`ΔM=-ΣJ_i dt`, synthesis debits `ΔA=-0.2ΣJ_i dt`, maintenance debits
`ΔA=-1e-5ΣC_i dt`, catalyst state changes by
`ΔC_i=(J_i-1e-4C_i)dt`, and turnover adds `Σ1e-4C_i dt` to local waste.
The fixed catalyst-production budget proves the mandatory tradeoff: increasing
one function decreases at least one other.

Physiology may read only local nutrient, fuel, activated resource, reserve,
structural damage, and membrane damage. Environment labels are prohibited. Each
allocation catalyst multiplies only its corresponding existing local flux by
`g_i=1+C_i/(0.1+C_i)`: nutrient/fuel processing, activation, repair, or
reserve-funded growth synthesis.

## Gates and authority

0. Preservation and schema
1. Conservation and invariant domain
2. Local expression identification
3. Mandatory tradeoff
4. Environmental input observability
5. Reciprocal pre-fission physiological effect
6. Heredity and mutation continuity
7. Single-generation fitness consequence
8. Multi-generation selection
9. Adaptation
10. Environmental reversal

No later gate may run after an earlier failure. Phase 3 remains unauthorized
until Gates 8–10 all pass.
