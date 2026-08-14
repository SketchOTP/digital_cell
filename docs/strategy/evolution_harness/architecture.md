# Evolution harness architecture

The `evolution-harness` crate is a research-layer boundary above the organism. It owns protocol validation, immutable experiment identities, population lifecycle, events, generations, lineages, observer-only selection analysis, and exports. It depends on `chemistry-core`; `chemistry-core` does not depend on it. `experiment-runner` and future adapters depend on the harness.

The first concrete adapter is `DigitalCellMeshAdapter`. It calls existing `coupled_step_growth` and converts only an actual returned physical fission into `AdvanceOutcome::Fission`. No `force_reproduce`, `set_fitness`, `heal`, `kill`, `set_growth`, or survival-control method exists. Declared damage is routed through existing mesh damage functions and recorded as an intervention event.

Synthetic adapters are used for harness tests so population/generation/lineage failures are isolated from mesh biology. No external ALife source or dependency is included.
