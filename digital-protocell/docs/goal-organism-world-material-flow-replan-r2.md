# Goal-mode organism/world material-flow replan R2

Status: `GOAL_AGENT_REPLAN`

This is a provisional goal-agent architecture decision. It is not independent
Architect acceptance and does not authorize implementation of the candidate
architecture.

## Decision

Route A is not currently viable. The exact existing D-091 reserve composition
was tested with the finite Route-B field and did not produce resource-causal
fission. Route B is therefore selected for architectural replan: the next
material-flow design must add a biologically meaningful assimilation/quota
boundary between environmental arrival and the existing bulk metabolic/growth
path, rather than search another motor or allocation formula.

## Evidence collapsed by this replan

The following local families are closed:

| family | result |
|---|---|
| CLOSURE-006 through CLOSURE-014 active-work/contact/material allocation | ceiling below unchanged `1.35 * birth_mass` gate |
| hard-contact runtime causality audit | fission at step 25 in active and transfer-disabled arms; first transfer at step 256 |
| Route-B finite spatial field | finite conserved transfer, survival, no fission |
| Route-C finite field plus exact D-091 reserve | active transfer at step 1, zero fission; disabled arm zero transfer and starvation collapse |

The Route-C active arm delivered `200.0863577497187` N and F with zero
world-conservation residual and remained alive. The transfer-disabled arm
delivered zero N/F and died. Neither arm crossed the physical reproduction
boundary. Transfer is therefore necessary for viability in this composition,
but the current environmental-to-bulk path does not provide a sufficient
resource-dependent developmental material rate.

## Source-level bottleneck

The current causal path is:

```text
finite environmental field
→ edge-local DC-DEV-008 N/F transfer
→ bulk intracellular N/F
→ frozen N+F → A/W metabolism
→ D-091 A → R reserve
→ R-funded structural growth
→ unchanged physical fission gate
```

The failure is downstream of finite transfer and upstream of reproductive
divergence. D-091 `R` is an activated-material reserve, not a nutrient quota:
it receives A after maintenance and other frozen reactions have consumed their
share. It therefore cannot, by itself, guarantee that environmental N/F is
retained as a growth-capable developmental history before the existing fission
gate is evaluated. Route-C proves this composition is viable but not
reproductive; it does not justify changing reserve horizons or coefficients.

## Candidate architecture boundary

The next architecture review should evaluate an explicit finite assimilatory
material-flow boundary:

```text
finite environmental N/F
→ local interfacial assimilation compartment
→ conserved resource quota / processing inventory
→ existing N/F/A/W chemistry and structural synthesis
→ unchanged growth, remesh, topology, and fission
```

The interfacial compartment is a design candidate, not an implemented state.
Before code, the review must decide whether it can be represented by an
already-existing material species or whether a new physical species is truly
required. A symbolic food flag, hunger flag, observer ledger, or motor signal
is not an acceptable substitute.

The contract must specify, for every unit of material:

1. finite environmental ownership and shared-cell depletion;
2. local transport into the interface;
3. assimilation and processing ownership;
4. the exact conserved quantity that survives maintenance timescales;
5. how that quantity reaches existing structural synthesis;
6. world, organism, reaction, reserve, and growth ledger closure;
7. checkpoint, remesh, fission, and daughter-partition continuity.

No parameter, kinetic constant, quota threshold, or fission threshold may be
chosen from a desired reproductive outcome. The unchanged `1.35 * birth_mass`
gate remains the acceptance boundary.

## Prior-art basis

Nutrient-limited growth models commonly couple extracellular diffusion and
local uptake to intracellular processing and biomass accumulation, rather than
treating uptake as a contact event whose material is immediately exposed to
all competing maintenance sinks. This is an adaptable systems principle only;
no biological rate or quota value is imported. See [nutrient-limited colony
modelling](https://pmc.ncbi.nlm.nih.gov/articles/PMC5832723/) and
[dynamic nutrient/proteome growth coupling](https://pmc.ncbi.nlm.nih.gov/articles/PMC10163005/).

## Required next work before implementation

The next goal-mode increment is a source-level contract and invariant audit,
not a runtime assay. It must compare at least:

- direct environmental-to-bulk transfer, now falsified as sufficient by
  Route-C;
- an interfacial quota/assimilation boundary;
- any existing D-088/D-091 material representation that could lawfully carry
  environmental provenance without a new semantic flag.

It must also state whether a new physical material species changes M1 or the
frozen chemistry boundary. If it does, stop and return the architecture to
external review before implementation.

## Stop condition

Stop local work if the candidate requires any of:

- another active-work/contact/material allocation formula;
- a new gain, threshold, timer, complement, ratio, or reserve horizon;
- changing the `1.35` gate;
- hidden environmental replenishment or an infinite bath;
- observer-ledger behavior;
- resource sensing, motor modulation, or post-ingestive control;
- silent reserve-on production;
- changing frozen M1 chemistry without separate authority.

```text
ROUTE_A_EXISTING_COMPOSITION: CLOSED_PROVISIONALLY_NEGATIVE
ROUTE_B_MATERIAL_FLOW_REPLAN: REQUIRED
RESOURCE_TO_FISSION_CAUSALITY: NOT_ESTABLISHED
AUTONOMOUS_RESOURCE_ACQUISITION: NOT_ESTABLISHED
NEXT_RUNTIME_EXECUTION: NOT_AUTHORIZED
SELF_ACCEPTANCE: GOAL_AGENT_PROVISIONAL_ONLY
```
