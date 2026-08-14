# Sealed D-094R2 contract

The authoritative executed contract is the source/config/binary combination
identified by `d094r2_gate6_execution.md` and the sealed attempt manifest. The
current `d094_selection.rs` on another revision is not used to fill gaps.

## Founders and seeds

- Eight independent single-founder lineages per replicate: four H and four B.
- H seed rule: `100 + replicate * 20 + i`, `i = 0..3`.
- B seed rule: `200 + replicate * 20 + i`, `i = 0..3`.
- H edges, duplicated twice: `E_AA, E_AR, E_RA, E_RB, E_BA`.
- B edges, duplicated twice: `E_BB, E_BR, E_RB, E_RA, E_AB`.
- Neutral uses the same mixed founder construction under baseline catalytic
  efficiencies (`rho_node = 0`); it is not an all-neutral genotype.
- Mutation was off: `AutocatalyticParams::with_mutation_off()`, so Gate 6
  mutation supply was zero. The frozen historical mutation constant remains
  `mu_E = 0.0089` for mutation-enabled work only.

## Organism and ecology

The source uses the existing mesh organism with `MechParams::default()`
(`dt = 0.02`), `GrowthParams { y_g = 0.9, enable_growth = true }`,
`FissionParams::default()`, `TransportParams::default()`, and reserve
parameters derived as:

```text
ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area)
AutocatalyticParams::derived(1 / k_release)
```

The seed mesh area is runtime-derived by the historical runner; it was not
emitted as a standalone contract field.

- H ecology: absolute exterior N/F values are `rich * 1.25` during the pulse
  and `rich * 0.18` during lean; `rich = 2.2`, pulse fraction `0.40`, and
  cycle period `PULSE_PERIOD_MULTS[0] * t_maint * 4 = 0.5*t_maint*4`.
- B ecology: absolute exterior N/F is `rich * 1.20`; identity-blind abrasion
  fires every `1.5*t_maint` using `ABRASION_STRENGTHS[0]` and membrane factor
  `0.6`.
- Neutral ecology: absolute exterior N/F is `rich * 0.70` and autocatalytic
  node efficiency is baseline (`rho_node = 0`).
- The source applies these fields independently to each lineage. It does not
  decrement one finite pool when another organism consumes resources.

## Horizon and endpoint

- Replicates: `8` per H, B and neutral campaign.
- Accepted steps: `22,000`, with `dt = 0.02`, hence horizon `440.0`.
- Target and completed generation: `8`.
- Frequency requirement: delta at least frozen `0.15`.
- Descendant requirement: at least `1.20x` in the historical Gate 6 win test.
- Neutral control: same mixed founder construction and seed campaign under
  baseline ecology; the historical analysis uses it as a zero-change control.
- A numeric minimum viable population threshold was not emitted by the sealed
  runner and remains `NOT_RECORDED`, not guessed.

The harness translation is exact about these source semantics where emitted,
and remains `execution_authorized = false` because the D-094 adapter,
historical material hashes, placement coordinates and causal differential
endpoints are not available in the accepted harness.
