# First implementation contract

This is a future implementation boundary, not implementation in DC-DEV-001A.

## Minimal contract

`LocalRegulatoryPatch` stores a versioned local state, reads a bounded local material frame and neighbor frame, applies one local non-semantic physical stimulus transducer, and returns only regulatory state, neighbor signal, local transduced input, and provenance. It is observer-coupled and non-authoritative. It cannot create or destroy material, command chemistry, transport, growth, repair, movement, behavior, birth/death, or fission, and it cannot read global population state.

The update function must be deterministic for `(initial_local_state, local_material_frame, neighbor_frame, accepted_dt, seed, implementation_version)`. Every update records input provenance and accepted time. Unsupported mapping returns `HARNESS_ADAPTER_UNAVAILABLE` rather than a silent no-op.

## First assay boundary

The first implementation may demonstrate only:

1. persistent local regulatory state;
2. bounded neighbor-to-neighbor propagation;
3. one local non-semantic physical stimulus transducer;
4. deterministic multi-step persistence;
5. perturbation response; and
6. a neutral/no-stimulus control with the same execution machinery.

The first slice exposes `regulatory_state`, `neighbor_signal`, `local_transduced_input`, and `provenance`. It has no effector intent, target direction or position, movement request, growth request, repair request, resource request, behavior command, motor output, or organism-physics effect. It must not include selection, learning, memory, reproduction qualification, or population fitness.

## Deferred architecture

Effector and motor architecture remains documented as future work only. It requires a later directive and independent causal review; it is not part of DC-DEV-002's first assay.

## Exit criteria for the next directive

The next directive may implement only this observer-coupled contract after independent review. It must add source-level tests, causal perturbation tests, replay evidence, and governance artifacts before any expansion.
