# Gate 5 to Gate 7 execution parity

Gate 5 authority is `chemistry-core::d096_allocation::pre_fission_assay`:

- seeds 1 through 8, 1,000 steps at `dt=0.02`;
- radius 8, interior `C=1.0`, `A=0.5`, `N=F=0.8`, `R=0.5`;
- finite D-096 allocation enabled;
- reserve derived from the sealed `ReserveParams::derived(...)` contract;
- expression, transport, reactions, and growth in that order;
- fission and mutation disabled.

The Gate 7 adapter path uses radius 14, interior `C=0.8`, `A=0.5`,
`N=F=0.4`, `R=0`, exterior `N=F=2`, and `ReactionParams::default()` with
reserve disabled. It executes D-096 expression before the generic coupled
step. These are materially different initial and reaction configurations.

The shadow replay reproduced the Gate 5 effects to floating-point precision:

- H reserve effect: `0.5988859008884866`;
- B retained-material effect: `3.8114697633476453`.

The result is a configuration/physiology parity failure, not a chemistry-core
failure. No Gate 7 rerun is authorized by this audit.
