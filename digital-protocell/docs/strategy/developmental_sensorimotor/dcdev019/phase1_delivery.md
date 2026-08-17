# DC-DEV-019 Phase 1 — finite nutrient delivery

## Authority

- Entry: `1e242f28152797b512e25cd56c7b718e45d6ca97`
- Branch: `strategy/dc-dev-019-nutrient-homeostasis-persistence`
- Directive: `DC-DEV-019`
- Assay: `dcdev019_phase1_delivery`
- Horizon: 5,000 accepted settlement steps followed by 480 accepted delivery steps

The assay is observer-only. Existing mechanics, chemistry equations, finite
resource semantics, and default production behavior remain authoritative.

## Reconstructed resource semantics

The production `FiniteSpatialResourceRegionV1` stores finite N/F mass, derives
fixed boundary concentration from mass and circular material volume, exposes
an edge only when its midpoint lies inside the region, applies the existing
permeability and positive inward concentration gradient, and caps each
transfer by remaining world mass. World loss equals organism delivery in
every step.

The ideal arm is an upper-bound diagnostic using the same region geometry,
exposure, boundary concentration, internal concentration, and finite
inventory, but equilibrates the exposed material in a well-mixed transfer
calculation. It does not create material or permit uphill transport.

## Gate results

- Gate 0: passed. The run starts from the exact DC-DEV-016 clean entry and
  uses the required current resource geometry `[4.8, 0.0]`, radius `1.5`,
  equal N/F inventory, and 480-step horizon.
- Gate 1: passed. Gain-1 counterfactual reaction and resource paths reproduce
  their existing paths within the assay tolerance; no production default is
  changed.
- Gate 2: passed through bounded Phase 1B/1C. Ideal current-inventory
  delivery is insufficient, so deterministic doubling and bisection select
  `M_selected=19.878372106390554`. Passive existing delivery at that mass
  restores stored activated material above the deprived start, so
  `G_transport_max=1` and no transport extension is required.

## Results

| arm | final `E_stored` | interpretation |
| --- | ---: | --- |
| D1 current geometry, existing delivery, `M=14.588954880632265` | `58.43844935623427` | current inventory insufficient |
| D2 current geometry, ideal delivery, `M=14.588954880632265` | `57.819981156419516` | inventory remains insufficient even under upper bound |
| Phase 1C current geometry, existing delivery, `M=19.878372106390554` | `61.68434818478833` | passive delivery passes restoration |

The accepted DC-DEV-018-R1 finite result `59.14641669238137` is reproduced by
an explicit reference replay using that assay's recorded geometry (center
`[0.0, 0.0]`, radius `5.0`). It is not relabeled as the DC-DEV-019 current
geometry result. The geometry discrepancy is retained in compact evidence
instead of being hidden.

## Evidence

Compact authoritative JSON is committed under
`experiments/generated/dcdev019/phase1/`. Dense step ledgers are not
committed. The artifact manifest records the schema, source entry, assay
parameters, and reproduction command. Phase 2 and later evidence must append
to this package without rewriting the Phase 1 result.

