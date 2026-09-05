# Goal-mode material-flow contract audit R2

Status: `GOAL_AGENT_PROVISIONALLY_ACCEPTED_REPLAN`

This is a provisional Goal-agent architecture record. It is not independent
Architect acceptance and does not authorize implementation of a new material
species or chemistry coupling.

## Largest unresolved end-goal gap

The project still lacks the causal chain:

```text
finite environmental transfer
→ later resource-dependent developmental growth
→ resource-causal physical fission
→ continuous daughter/lineage history
```

The resource-to-fission audit is sealed as a negative: the active-transfer and
transfer-disabled arms fissioned at the same pre-transfer step, while finite
environmental transfer began later. The existing hard-contact route therefore
cannot count that fission toward the completion gate.

## Source-level invariant reconstruction

### Environmental ownership

`FiniteWorldV1` and `SpatialMaterialFieldV1` own finite N/F inventories. They
construct local edge requests from the existing permeability and flux law,
debit the same world inventory, and add delivered material to the organism's
bulk `MaterialMesh::interior.n/f` pools. Their conservation boundary is sound:

```text
world N/F loss = organism N/F delivery
```

The transport boundary does not retain environmental provenance after delivery.

### Bulk reaction ordering

The current runtime and `MeshPopulation::step` order are:

```text
finite-world exchange / transport
→ reactions_step_with_reserve_mode
→ growth_step
→ mechanics and remesh
→ topology/fission eligibility
```

Inside the frozen reaction step, N and F are consumed together by the existing
`N+F → A+W` reaction before the rest of the activated-material, catalyst,
maintenance, reserve, and structural pathways execute. With reserve disabled,
D-088 growth consumes post-reaction A surplus. With the opt-in D-091 reserve,
growth consumes R.

### Reserve semantics

`MaterialMesh::interior.r` is an activated-material reserve. It is charged from
A and used for existing reserve-funded work/growth. It is not an environmental
N/F quota, does not retain resource provenance, and cannot by itself prove that
environmental transfer preceded reproductive divergence. Enabling D-091 does
not repair that semantic gap.

### Reproduction boundary

The unchanged physical gate remains:

```text
total_structural_mass >= 1.35 * birth_mass
```

followed by the existing topology/pinch/fission checks. This gate is not being
weakened or reinterpreted.

## Candidate architecture comparison

| Candidate | Existing implementation | Goal status | Decision |
|---|---|---|---|
| Direct environmental N/F → bulk N/F | `FiniteWorldV1` / `SpatialMaterialFieldV1` | finite and conserved, but not sufficient for resource-causal growth/fission | closed as sufficient composition |
| Reuse bulk N/F as quota | same `MaterialMesh::interior.n/f` pools | immediately exposed to frozen metabolism and geometry/concentration feedback | not a distinct assimilation boundary |
| Reuse D-091 R | `MaterialMesh::interior.r` | activated-material reserve with no environmental provenance | semantically incompatible |
| Symbolic food/ledger/flag | not an organism material state | would violate physical ownership and observer-independence | prohibited |
| New finite assimilation/provenance compartment | not implemented | could preserve environmental ownership until lawful processing, but requires a new physical state and coupling contract | architecture candidate; external review required |

## Required contract for any future implementation

Before code is authorized, the candidate must specify and test all of these
invariants without changing the `1.35×` gate:

1. The world remains the sole owner of finite untransferred N/F.
2. A local interfacial transfer has one explicit owner and one conservation
   ledger; no hidden bath or replenishment exists.
3. Assimilated material remains an organism-internal physical quantity long
   enough to participate in development; it is not an observer ledger or a
   behavior flag.
4. The exact conversion from assimilated material into existing N/F, A/W, or
   structural synthesis is specified before execution. It must not silently
   duplicate or bypass the frozen reaction ledger.
5. Every source and sink is reconciled across environment, interface, bulk
   chemistry, reserve, growth, work, waste, remesh, checkpoint, fission, and
   daughter partition.
6. A transfer-disabled control must remove the causal input and reproduce the
   corresponding non-resource developmental trajectory.
7. The quantity must survive the required runtime boundaries: checkpoint,
   remesh, fission, and daughter continuation.
8. The new state must have a declared physical meaning and schema/version
   authority. It may not be smuggled into `R`, `free_l`, template fields, or an
   observer-only field under a different name.

## Prior-art boundary

Nutrient-limited growth models generally keep environmental influx, internal
processing, biomass accumulation, and division-related state in a coupled
material-flow model; they do not treat contact delivery alone as proof of
resource-dependent reproduction. This is an adaptable systems principle only;
no biological rate, quota, or threshold is imported. See [nutrient-limited
colony modelling](https://pmc.ncbi.nlm.nih.gov/articles/PMC5832723/), [cell
size and nutrient availability](https://pmc.ncbi.nlm.nih.gov/articles/PMC3350639/),
and [dynamic nutrient/proteome growth coupling](https://pmc.ncbi.nlm.nih.gov/articles/PMC10163005/).

## Decision and stop boundary

The largest coherent advancement is an architectural replan, not another
runtime assay. The local CLOSURE-006→014 active-work/contact/material family is
collapsed and remains closed. Route A is provisionally negative for the
current composition. Route B is required.

Implementation stops here because a lawful assimilation compartment would alter
organism material state and its path into frozen chemistry/growth. That is a
new physical coupling and therefore needs an explicit independent Architect
review before coding or runtime execution.

```text
ROUTE_A_EXISTING_COMPOSITION: GOAL_AGENT_PROVISIONALLY_NEGATIVE
ROUTE_B_MATERIAL_FLOW_ARCHITECTURE: REQUIRED
NEW_PHYSICAL_ASSIMILATION_STATE: REVIEW_REQUIRED
RESOURCE_CAUSAL_REPRODUCTION: NOT_ESTABLISHED
AUTONOMOUS_RESOURCE_ACQUISITION: NOT_ESTABLISHED
NEXT_RUNTIME_EXECUTION: NOT_AUTHORIZED
SELF_ACCEPTANCE: GOAL_AGENT_PROVISIONAL_ONLY
```
