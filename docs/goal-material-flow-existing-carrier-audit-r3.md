# Goal-mode existing material-carrier audit R3

Status: `GOAL_AGENT_PROVISIONALLY_ACCEPTED_REPLAN`

This is a source-level architecture audit performed by the Goal-mode
architect/coder. It is not an independent Architect acceptance and does not
authorize a successor runtime execution.

## Largest unresolved goal gap

The project still lacks a verified causal chain in which finite environmental
N/F transfer occurs before, and materially causes, developmental growth and
physical fission. The sealed resource-to-fission audit remains negative: the
active and transfer-disabled arms fissioned at the same step before the active
arm's first environmental transfer. Route-B finite spatial transfer and the
opt-in Route-C D-091 reserve composition conserved transfer and preserved
viability, but neither produced fission.

The local active-work/contact/material-allocation family is closed by
CLOSURE-006 through CLOSURE-014 and is not reopened here.

## Question audited

Can an already-existing physical field carry finite environmental provenance
from interfacial delivery through the maintenance timescale into the existing
growth/fission path, without adding a new semantic state?

## Source-level inventory

| Existing representation | Physical meaning in source | Can carry environmental provenance into growth? | Disposition |
|---|---|---:|---|
| `interior.n`, `interior.f` | Bulk nutrient/fuel concentrations | No | Direct transfer is exposed immediately to the frozen reaction kernel |
| `interior.a` | Activated material | No | Shared by activation, catalyst production, decay, reserve storage, and work |
| `interior.r` | D-091 stored activated-resource equivalents | No | Explicitly A-derived; not nutrient quota, readiness, age, fitness, or division progress |
| `free_l`, edge `b` / `m` | Membrane/free structural material | No | D-081 membrane reserve and repair material, not nutrient assimilation |
| `u_h/u_b`, `k_h/k_b`, templates | Template/hereditary chemistry | No | Hereditary/material-network state, not environmental nutrient provenance |
| `q_k/q_e`, `k_a/k_r/k_node_b` | D-094 catalytic-network material | No | Catalytic network chemistry, not environmental transfer history |
| D-018 provenance tracer | Observer-only structural provenance | No | Explicitly observer-only and cannot enter behavior or growth |

The inventory is based on the current source, not on names alone. In
`LumpedChem`, N/F are the only existing nutrient/fuel fields; R is documented
as stored activated-resource material and explicitly not readiness or a
division trigger. The D-081 reserve is a separate membrane ledger (`M_L` and
`M_B`) and its accepted continuation remains membrane-specific.

## Causal proof of the boundary

The current reaction path computes the frozen activation extent directly from
bulk `n*f`, then subtracts N and F and adds A and W. The growth path then uses
instantaneous A surplus when reserve is disabled, or D-091 R when reserve is
enabled. The unchanged physical fission gate reads structural mass and local
pinch conditions. No existing field preserves an environmental-origin amount
between transfer and those consumers.

The existing fission partition conserves the current interior pools by area
fraction, including N/F/A/W/R and the established template/network fields, but
it has no environmental-assimilation pool to partition. Adding an assay ledger
alone would therefore fail the required checkpoint, remesh, fission, and
daughter-continuity contract.

## Route decision

### Route A — existing composition

`GOAL_AGENT_PROVISIONALLY_CLOSED_FOR_CURRENT_COMPOSITION`.

Reusing bulk N/F is not a new causal boundary. Reusing A or R changes the
meaning of those frozen fields. Reusing membrane, template, catalytic-node,
or observer-provenance state is semantically incompatible. The source audit
therefore finds no existing carrier that can satisfy the goal's environmental
provenance requirement.

### Route B — materially different architecture

The remaining candidate is one explicit finite assimilatory material
compartment, with a physical identity distinct from bulk N/F and D-091 R. It
would need to be owned by the finite world/interface ledger, retained through
the reaction step, converted by an explicitly defined conserved process into
the existing chemistry/growth path, and partitioned through remesh,
checkpoint, fission, and daughters.

This is not implemented in this increment because the conversion law and its
semantic relationship to frozen M1 chemistry are architectural choices, not
mere wiring. Implementing it without that contract would create another
nearby assay loop and could smuggle a growth quota into the organism.

## Prior-art constraint

Nutrient-limited growth models support the adaptable systems principle that
environmental transport, intracellular processing, and biomass accumulation
are coupled across distinct timescales. No biological rate, quota, or yield is
imported by this audit. The Digital Cell implementation must still derive its
state ownership, conservation, and conversion law from project invariants.

## Required next capability increment

Before runtime implementation, independently review and freeze an
assimilation-compartment contract containing:

1. finite world ownership and shared-allocation debit;
2. local interface delivery and an explicit transfer ledger;
3. retained physical state with checkpoint and serialization identity;
4. conversion into existing N/F/A/W or a separately justified growth substrate;
5. no double counting with D-091 R, free membrane, templates, or catalysts;
6. exact remesh/fission/daughter partition rules;
7. transfer-disabled and no-resource controls;
8. preservation of the unchanged `1.35 * birth_mass` gate and frozen M1 path.

If the contract requires a new kinetic parameter, a new biological yield, or a
new growth trigger chosen from the desired fission result, stop and return the
architecture for review rather than tuning it.

## Provisional disposition

```text
RESOURCE_TO_FISSION_CAUSALITY: NOT ESTABLISHED
ROUTE_A_EXISTING_CARRIER: NOT AVAILABLE
ROUTE_B_ASSIMILATION_COMPARTMENT: ARCHITECTURE REVIEW REQUIRED
LOCAL_ACTIVE_WORK_FORMULA_SEARCH: CLOSED
NEXT_RUNTIME_EXECUTION: NOT AUTHORIZED
AUTONOMOUS_RESOURCE_ACQUISITION: NOT ESTABLISHED
SELF_ACCEPTANCE: GOAL_AGENT_PROVISIONAL_ONLY
```
