# D-092 Minimal Catalytic Template Heredity

## Primary conclusion

`D092_TEMPLATE_HEREDITY_QUALIFIED_MOTIF_SELECTION_REJECTED`

## Records

- `MINIMAL_CATALYTIC_TEMPLATE_HEREDITY_QUALIFIED`
- `TEMPLATE_MOTIF_EXPRESSION_ARCHITECTURE_REJECTED`
- `PHASE3_NOT_AUTHORIZED`
- `TEMPLATE_INFORMATION_LOSS_IRREVERSIBLE_WITHOUT_TEMPLATE`

## Schema

- Equation: `autopoietic_material_mesh_catalytic_template_v1`
- Fields: `mesh_vertices_edges_reserve_template_polymer_v1`
- D-091 reserve physiology preserved under template stamp
- `C_H/C_B` composition remains available for historical tests only (`enable: false` in D-092)

## Architecture

Physical template chains live on `MaterialMesh.templates` (not an organism genome).

- Free monomers `U_H` / `U_B` on `LumpedChem`
- Ordered bonded monomers store sequence information
- Local match/mismatch association (100× affinity ratio) + A-consuming ligation
- Motifs `HHB` / `BBH` bind catalyst into `K_H` / `K_B` complexes
- Complex efficiencies: free `1.0/1.0`, `K_H` `1.5/0.5`, `K_B` `0.5/1.5`
- Fission partitions chains by spatial position (no sequence copy)

### Founders (L=12, 6H+6B)

| Role | Sequence |
|------|----------|
| Harvest | `HHBHHBHHBBBB` |
| Build | `BBHBBHBBHHHH` |
| Neutral | `HBHBHBHBHBHB` |

## Gate summary (smoke evidence)

| Gate | Result |
|------|--------|
| 0 Preservation / schema | PASS |
| 1 Polymer accounting | PASS |
| 2 Local copying (~1% mismatch) | PASS |
| 3 Population maintenance | PASS |
| 4 Fission inheritance | PASS |
| 5 Sequence→phenotype causality | PASS |
| 6 Shared-dish selection | FAIL |
| 7 Mutation adaptation | FAIL |
| 8 Environmental reversal | FAIL |
| 9 Information necessity | PASS |
| 10 Stability | PASS |

Measured per-site mismatch ≈ 0.013 (target ~0.01).

## Interpretation

Template material organization, local copying, fission partition, and motif-complex expression are established. Environment-dependent selection on fixed motif specialization was not established under the D-091 reserve ecologies in this campaign.

Phase 3 is **not** authorized. Next architecture may retain template copying but should replace fixed motif specialization with a local catalytic reaction-network topology. Do not reopen `C_H/C_B` or raise `σ`.

## Entry / seals

- D-091 seal: `58817ac` / tag `D-091-metabolic-reserve-qualified-selection-rejected`
- D-092 start: `58817ac`
- D-092 end: `a2196ae` / tag `D-092-template-heredity-qualified-selection-rejected`
- Branch: `phase2-growth-division-inheritance` (no Phase 3 branch)

## Artifacts

`digital-protocell/experiments/generated/d092/`

## Note

Smoke mode shrinks selection replicates; Gate 6–8 failures are reported honestly without smoke auto-pass. Full non-smoke matrices remain available for follow-up if desired.
