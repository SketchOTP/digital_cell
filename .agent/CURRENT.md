# CURRENT.md

## Active directive
- ID: D-20260724-d089-compositional-catalytic-heredity-selection
- Project directive: D-089
- Goal: Compositional catalytic heredity + environment-dependent selection
- Status: sealed — selection provisional pending ecology audit
- Acceptance: partial (gates 0–4 met; gate 6 provisional)
- Touched files: catalyst_composition*, population_selection*, d089_*, mesh_*, .agent/*
- Next action: D-090 ecological timescale repair (organism frozen)

## Repo facts needed now
- Entry: e4e049d / D-088-causal-growth-fission-inheritance-qualified
- Seal records: D089_HEREDITY_AND_PHENOTYPE_QUALIFIED; D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT
- Hypothesis: EARLY_FISSION_PRECEDED_SELECTION_PRESSURE
- μ=0.01; σ=0.15 frozen; schema catalytic_composition_v1
- Tag: D-089-natural-selection-not-established

## Last validation
- Command: cargo test -p chemistry-core --test d089_tests; D089_SMOKE=1 d089 pipeline
- Result: unit PASS; pipeline → provisional Gate6 conclusion

## Open blockers
- Shared-dish ecology timescale invalid for selection judgment

## Mimir V2
- D-090 task active: 75a7ae47e6204966a59d95ef7c0b48fc
