# Evolution harness architecture

The `evolution-harness` crate is a research-layer boundary above the organism. It owns protocol validation, immutable experiment identities, population lifecycle, events, generations, lineages, observer-only selection analysis, and exports. It depends on `chemistry-core`; `chemistry-core` does not depend on it. `experiment-runner` and future adapters depend on the harness.

The first concrete adapter is `DigitalCellMeshAdapter`. It calls existing `coupled_step_growth` and converts only an actual returned physical fission into `AdvanceOutcome::Fission`. No `force_reproduce`, `set_fitness`, `heal`, `kill`, `set_growth`, or survival-control method exists. Declared damage is routed through existing mesh damage functions and recorded as an intervention event.

Synthetic adapters are used for harness tests so population/generation/lineage failures are isolated from mesh biology. No external ALife source or dependency is included.

## SR-003R execution boundary

`ReplicateRunner::run_campaign` creates a fresh adapter, founder, population, ledger, and lineage tracker for every declared seed. Treatment and neutral campaigns use the same runner and are compared only after both campaigns finish.

Execution is gated by `validate_for_execution` at both the campaign and harness construction boundaries; unresolved historical fixtures fail before founder initialization. A `SelectivePressureContractV1` identifies the treatment/neutral contrast and event condition. Only that declared treatment event can establish pressure; generic resource activity and scarcity endings do not.

Each step has explicit global phases: schedule resolution, living-ID snapshot, all interventions, intervention ledger append, all organism advances, common accepted step end, outcome processing, and invariant validation. This prevents a later organism's start-of-step event from following an earlier organism's step-end fission event.

`initialize_founders` consumes the placement protocol and creates stable, independently identified founder lineages. `HeredityEvidenceV1` and `PhenotypeEvidenceV1` are mechanism-aware adapter contracts; the mesh adapter returns `HARNESS_ADAPTER_UNAVAILABLE` until D-094-specific qualifiers are adapted.

The adapter supplies the accepted `dt`; the harness accumulates `f64` simulated time and uses it for event timestamps, ecology schedules, and generation intervals. The mesh adapter executes continuous transport through the existing mesh step, pulsed/scarcity resource changes through the existing exterior resource fields, and damage through the existing damage functions. Shared and spatial-local ecology require a population/dish adapter; the single-organism mesh adapter rejects those capabilities explicitly rather than silently ignoring them.
