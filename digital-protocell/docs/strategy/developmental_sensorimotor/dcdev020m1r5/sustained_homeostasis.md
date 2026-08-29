# DC-DEV-020-M1-R5 sustained finite-resource homeostasis

This observer-only qualification starts from accepted R4 head
`68d1c88ec1b915a4bee86efe24e985222b529d5a` and the exact depleted M1 entry
state used by R4. It runs the existing ConservativeV3 chemistry with reserve
OFF for 8,000 accepted steps.

The only new world-side contract is
`FINITE_SPATIAL_BACKING_RESERVOIR_V1`. It fixes the accepted R4 boundary
concentrations (`2.063914918930895` for both N and F), scales the R4 finite
inventory by the preregistered ratio `8000/480 = 50/3`, and performs no
replenishment. Every local exposure, permeability, segment, and `dt`
calculation is delegated to the unchanged V1 finite-resource region.

## Arms

- `COUPLED_SUSTAINED`: ConservativeV3, reserve OFF, backing reservoir, and
  accepted R4 same-step paired N/F to A+W.
- `UNCOUPLED_SUSTAINED`: the identical backing reservoir and organism, with
  ordinary V1 N/F deposition and no coupled activation.
- `NO_RESOURCE`: zero external N/F for 8,000 steps.
- `FEED_THEN_REMOVE`: exact R4 finite patch for 480 steps, then removal of only
  the remaining external N/F inventory followed by 8,000 steps without reset.

The assay records compact checkpoints at steps `0, 480, 1000, 2000, 3466,
4000, 6000, 6931, 8000`, plus the feed-removal checkpoints. Dense per-step
ledgers are not committed; authoritative local evidence is stored at the
shared-drive path recorded in the artifact manifest.

No production selection, chemistry equation, transport law, reserve,
recycling, salvage, controller, M2, or DC-DEV-021 behavior is changed.

Architect acceptance remains pending until exact-head Linux CI verifies the
fresh D-087 V2/V3 8/8 controls, the sustained criteria, and the preservation
suite.
