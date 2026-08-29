# DC-DEV-020-M0-BASELINE-001

## Production-selection provenance

The production selector is the phase-1 certifier simulation layer:

- `crates/phase1-certifier/src/sim.rs::conservative_v2_enabled()` selects the material contract.
- `crates/phase1-certifier/src/sim.rs::reserve_enabled()` selects D-091 reserve physiology.
- `seed_mesh()` stamps the D-091 reserve equation only when `reserve_enabled()` is true.
- `reaction_params_for()` installs D-091 reserve parameters only when `reserve_enabled()` is true.

Before M0 selection, choosing `ConservativeV2` without an explicit reserve selector implicitly enabled D-091. The bounded M0 change makes the defaults independent and explicit:

```text
ordinary default: ConservativeV2 / reserve OFF
diagnostic opt-in: DCDEV020R9R3_RESERVE=1
historical contract opt-in: DCDEV020R9R3_CONTRACT=HistoricalV1
```

No D-091 source, constants, equations, or consumers were changed. The R9 workflow declares its reserve-enabled diagnostic arms explicitly so its sealed evidence remains reproducible.

## Dependency boundary

The selected M0 path has no structural dependency on reserve species `R`: reserve stamping and reserve parameter installation are both bypassed, while the existing `R` field and D-091 consumers remain available to post-M0 diagnostics and historical tests. No actuator, behavior, feeding, recycling, or DC-DEV-021 layer was rewritten.

## Fresh local qualification

The ordinary no-selector production path was executed at the M0 entry head with the sanctioned Rust 1.89.0 toolchain:

- contract: `ConservativeV2`
- reserve: `false`
- actual D-087 gates: `8/8`
- packaged runtime: alive after 5,000 steps
- reserve flows: `A→R=0`, `R→A=0`, `R→W=0`
- reserve rejected steps: `0`
- activation-equivalent closure residual: `2.580691216280684e-10`

Remote Linux CI remains the authority for packaged Linux runtime closure and final acceptance. M1 is not authorized by this record.
