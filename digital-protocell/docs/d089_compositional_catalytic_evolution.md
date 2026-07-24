# D-089: Compositional Catalytic Heredity and Natural Selection

## Schema

- Equation: `autopoietic_material_mesh_catalytic_composition_v1`
- Fields: `mesh_vertices_edges_catalyst_composition_v1`
- Materials: `C_H` (harvest-biased) + `C_B` (build-biased), `C = C_H + C_B`
- Frozen scalar catalyst schema unchanged when `composition.enable = false`

## Copying (production only)

\[
p_H = C_H/(C_H+C_B+\epsilon),\quad p_B = C_B/(C_H+C_B+\epsilon)
\]

\[
J_{C_H} = J_C[(1-\mu)p_H + \mu p_B],\quad J_{C_B} = J_C[(1-\mu)p_B + \mu p_H]
\]

- No catalyst ⇒ no copying
- No mutation at division
- Turnover proportional on both types
- A cost of total `J_C` unchanged

## Mutation rate

\[
\mu = \mathrm{clamp}(2/B_C,\ 10^{-5},\ 10^{-2})
\]

Derived once from D-088 median catalyst-production equivalents per generation (`B_C`). Observed campaign value typically hits the upper clamp (`μ = 0.01`).

## Tradeoff (frozen σ = 0.15)

\[
z = (C_H-C_B)/(C_H+C_B+\epsilon)
\]

\[
g_{\mathrm{harvest}} = 1+\sigma z,\quad g_{\mathrm{build}} = 1-\sigma z \in [0.85,1.15]
\]

Applied only to N/F activation vs structural growth / membrane production / repair. Not to transport, mechanics, death, or topology.

## Gate results

| Gate | Result |
|------|--------|
| 0 D-088 preservation (σ=μ=0) | PASS (rel diff ~1e-16) |
| 1 Composition accounting / partition | PASS |
| 2 Mutation calibration | PASS (conversion ratios ≈ 1.0) |
| 3 Parent–offspring heritability | PASS (corr ≈ 1.0, slope ≈ 1.0) |
| 4 Phenotype causality | PASS (isolated matched organisms) |
| 5 Shared-dish harness exercised | PASS (campaigns ran) |
| 6 Environment-dependent selection | **PROVISIONAL** — not established under D-089 shared-dish ecology |
| 7–9 | Not reached |

## Records

- `D089_HEREDITY_AND_PHENOTYPE_QUALIFIED` — Gates 0–4
- `D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT` — Gate 6 primary conclusion
- Historical campaign label: `D089_ENVIRONMENT_DEPENDENT_SELECTION_NOT_ESTABLISHED` (invalid ecology suspected; **not** trait rejection)
- Hypothesis for D-090: `EARLY_FISSION_PRECEDED_SELECTION_PRESSURE`

Compositional catalyst architecture is **not** rejected by D-089.

## Shared-dish audit (Gate 6)

Environment-dependent natural selection was **not** established under the D-089 shared-dish assay.

Observed:

1. Isolated phenotype assays (Gate 4) show harvest bias improves activation under scarce N/F and build bias improves growth/repair under damage.
2. In a shared finite bath, **construction-biased founders often leave equal or more descendants even under resource-limited supply**, because early surplus growth / fission is construction-gated (`mass ≥ 1.35 × birth_mass`). Builders reach fission first while the bath is still partially stocked — founders enter with stored A and free material that funds early builder fission before scarcity becomes biologically relevant.
3. Catalyst-frequency shifts of magnitude ≥ 0.15 with matching clade reproductive advantage did not reach the required replicate threshold in either Environment H or B (smoke campaign also used softened thresholds).
4. No fitness controller, population cull, or σ sweep was introduced (σ remains 0.15).

## Primary conclusion

`D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT`

Physical heredity (compositional catalysts, imperfect copying, partition inheritance) and isolated phenotype causality are qualified. Differential reproduction under shared finite environments is not yet interpretable because the D-089 ecology allowed reserve-funded early fission.

## Next

`D-090: Ecological Timescale Repair and Natural Selection Requalification` — freeze the organism; repair only the ecology; requalify or permanently reject the `C_H`/`C_B` architecture under a valid timescale contract. No σ sweep. No growth/fission retune.

## Product status

`RESEARCH_PROGRAM_ACTIVE_FINAL_PRODUCT_NOT_READY`
