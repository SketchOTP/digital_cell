# Route B: spatial material-field replan

Status: `GOAL_AGENT_PROVISIONAL_NEGATIVE`

This document records the material-flow architecture selected after the
resource-to-fission causality audit. It is a design boundary, not a biological
qualification and not an Architect acceptance.

The first integrated runtime test of this architecture is now complete. It is
provisionally negative for resource-causal reproduction: the finite field
delivered N/F before the runtime horizon, conserved the transfer, and preserved
the active organism, but neither the active nor transfer-disabled twin crossed
the unchanged physical fission gate. The active arm reached approximately
`199.786` N and F delivered, retained one living individual, and ended with no
fission after the established 12,000-step developmental bootstrap plus 3,000
runtime steps. The transfer-disabled twin lost viability and also produced no
fission. This is not an acceptance of Route B and does not authorize another
local allocation variant.

## End-goal gap

The runtime's finite circular resource can only deliver N/F when membrane
exposure exists. In the seed-2 causality audit, fission occurred at step 25 in
both the transfer-enabled and transfer-disabled arms, while the first enabled
transfer occurred at step 256. The existing fission gate and local active-work
family therefore cannot establish resource-causal reproduction in that
composition.

## Source-level findings

The current runtime order is:

1. native polarity motor and A-funded mechanics;
2. `FiniteWorldV1::exchange`;
3. frozen reactions;
4. growth;
5. remesh/topology;
6. existing `1.35 * birth_mass` eligibility and physical pinch fission.

`FiniteWorldV1` owns finite inventory and order-independent allocation, but its
resource request is derived from `FiniteSpatialResourceRegionV1` edge exposure.
That is a hard-contact source, not a spatially transported environmental field.

The repository already contains D-090's `SpatialDish`: a finite N/F/W field,
mass-conserving diffusion, local depletion, shared inventory, and a
resource-before-growth/fission step order. Its existing implementation is a
useful substrate, but it is not adopted unchanged because it samples chemistry
at the organism centroid and can optionally add uniform supply. Those choices
are not sufficient for the final organism's local membrane/material boundary.

## Candidate architecture

The opt-in, versioned spatial environmental material field implemented for this
test:

- stores finite N/F/W mass in world control volumes;
- advances diffusion with conservative pairwise fluxes;
- uses zero external N/F in the ordinary membrane transport pass;
- evaluates the unchanged DC-DEV-008 N/F flux law at each membrane edge using
  the field value at that edge's local world cell;
- allocates simultaneous edge/organism requests against the same pre-step cell
  inventories;
- subtracts exactly delivered N/F from those cells;
- routes W emission back to the local environmental cells;
- advances before the unchanged frozen reaction, growth, topology, and fission
  laws;
- permits finite initial patches but no hidden refill in the primary ecology.

This is an environmental material-flow change, not a motor formula, sensor,
reward, target, gradient-to-heading controller, or fission-gate change.

## Invariants

The implementation is eligible for an end-to-end test only if it proves:

- world N/F loss equals organism N/F delivery;
- field diffusion is mass-conservative and nonnegative under the selected
  numerical stability contract;
- uptake uses existing permeability, `k_flux`, boundary/interior driving force,
  exposed edge length, and `dt` without new biological coefficients;
- no centroid-only uptake shortcut remains in the candidate interface;
- no uniform supply is used in the finite primary arm;
- multiple organisms draw from shared cells with order-independent allocation;
- no resource coordinates, distances, bearings, gradients, or inventory values
  enter organism behavior;
- the existing `1.35 * birth_mass` gate and physical pinch law remain unchanged;
- transfer is recorded before any candidate reproductive divergence;
- transfer-disabled and field-empty controls remain valid;
- previous accepted runtime and frozen-science evidence remains preserved.

## Result and stop condition

The coupling itself is valid: it uses finite local field mass, conservative
diffusion, unchanged edge transport, and transfer before reactions/growth and
topology. The end-to-end Route-B candidate nevertheless did not establish
resource-causal fission. Stop this local implementation family here. Do not
respond by changing the fission gate, increasing the field inventory, moving
the field after inspecting the result, or restarting the CLOSURE-006 through
CLOSURE-014 active-work formula search.

The next architecture decision must be made at the material-flow boundary:
either identify an already-existing source-level composition that supplies
environmental material at a biologically useful developmental rate, or close
the current hard-contact/spatial-field boundary as insufficient and replan the
organism-world material interface from prior art and Digital Cell invariants.

## Prior-art basis

Spatial microbial models commonly represent extracellular nutrient diffusion,
local uptake, depletion, and growth as one coupled material-flow system. The
conservative moving-mesh reaction-diffusion literature provides the numerical
precedent for preserving global mass while domains evolve. These principles are
adapted here only at the environmental-material and numerical-method level;
no biological kinetic values are imported.
