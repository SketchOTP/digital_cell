# DC-DEV-020-M1-REPLAN-002-R5-R4

## V4 admissible-boundary no-resurrection qualification

This qualification isolates M1 physiology from M2 spatial resource acquisition.
It starts from the accepted R5-R2 transport-conservation head
`4c6a0020be887f66ea6cfab661ce570c730f7d90`, derives the accepted R1
per-step source-opportunity envelope, and applies that envelope through a
harness-local finite, nonspatial, membrane-mediated boundary.

The boundary makes every intact edge eligible, but still evaluates the existing
permeability law, current edge length, current interior concentration, timestep,
and `k_flux`. Applied transfer is capped by membrane capacity, remaining
finite inventory, and the accepted R1 per-step cap. No direct or unconditional
internal N/F insertion is used, and replenishment is zero.

S0 is the exact 480-step accepted R1 deprivation state. S1 is the first
completed observer-nonviable starvation state. S2 is the last completed state
before the next authoritative starvation physics step would fail. The
starvation replay stops on the authoritative mechanics result and never uses a
rejected post-step state.

Dense per-step evidence is stored on Atlas at:

`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r4/`

Compact Git evidence is stored under:

`digital-protocell/experiments/generated/dcdev020m1replan002r5r4/`

This directive does not change V4 biology, transport production code, D-087,
the production selector, or any M2 behavior. The result remains pending
Architect acceptance.
