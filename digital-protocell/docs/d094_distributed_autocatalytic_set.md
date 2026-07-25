# D-094 Distributed Autocatalytic-Set Heredity

## D-093 seal (corrected)

- Commit: `973222e`
- Tag: `D-093-template-network-heredity-qualified-selection-untestable`
- Primary: `D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_UNTESTABLE_ZERO_GENERATION`
- Record: `DIRECT_TEMPLATE_METABOLIC_EXPRESSION_CLOSED`

## Schema

| Item | Value |
|------|-------|
| Equation | `autopoietic_material_mesh_autocatalytic_set_v1` |
| Fields | `mesh_vertices_edges_reserve_autocatalytic_network_v1` |
| μ_E | `0.0089` (frozen) |

## Equations

Node production: `E_ij + Q_K + A → E_ij + K_j + W`  
Node turnover: `K_j → Q_K + W`  
Edge copy: `K_i + E_ij + Q_E + A → K_i + 2 E_ij + W` (target mutates with μ_E)

## Founders

| Set | Edges |
|-----|-------|
| H | E_AA, E_AR, E_RA, E_RB, E_BA |
| B | E_BB, E_BR, E_RB, E_RA, E_AB |
| N | E_AR, E_RB, E_BA, E_RA, E_BR |

## Zero-generation audit (mandatory)

Matched arms (full network / binding off / expression off / templates absent / D-091
reserve control) under the shared-dish pulse-lean H ecology. All arms grow (mass ratio
~2.1–2.5) but complete **0 generations**, including the D-091 reserve control. First
causal blocker: **`ECOLOGY_HORIZON_TOO_SHORT`** — the differentiated selection ecology
(pulse-lean / abrasion) suppresses reproduction to zero generations regardless of the
hereditary substrate. This is the same reproduction–ecology coupling that made D-093
selection untestable. Artifact: `d093_zero_generation_audit/audit.json`.

## Reproduction–ecology coupling repair

The shared-dish selection harness produced zero completed generations even for the
reserve-only control (`ECOLOGY_HORIZON_TOO_SHORT`). Gates 6–8 therefore run on the
reproduction-qualified `MeshPopulation` step (the same harness that qualifies D-088
reproduction) with ecology applied as:

- **H**: intermittent supply with a lean floor (pulse `1.15×`, lean `0.35×`) — harvesting
  networks favored; reproduction completes (validated: gen 7, 32 fissions single lineage).
- **B**: steady moderate supply (`0.85×`) plus periodic identity-blind abrasion — building
  networks favored.
- **N**: steady supply, node catalytic efficiencies set to baseline.

No parent selection, no culling, no lineage identity in chemistry. Fission remains local
pinch topology; edges partition by physical position (`autocatalytic_partition.rs`).

## Physical heritability (Gate 4)

Edges are dispersed in 2D toward distinct boundary vertices (`redistribute_edges_along_axis`)
so local-pinch fission partitions hereditary material into both daughters instead of emptying
one lobe. Metrics: closed/recoverable fraction, parent–offspring edge-frequency correlation,
and parent–offspring network-response correlation (target-node allocation).

## Artifacts

`digital-protocell/experiments/generated/d094/`

## Branch

`phase2-growth-division-inheritance` starting at D-093 seal `973222e`.
