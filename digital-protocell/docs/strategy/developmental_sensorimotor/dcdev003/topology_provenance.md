# Topology provenance

The existing chemistry-core remesh code remains the authority for topology changes. It is not modified and does not consume regulatory state.

The continuity observer receives immutable frames before and after that accepted step. A caller supplies `Stable`, `Split`, or `Merge` based on the observed vertex-count transition. The continuity layer validates that the declared event agrees with the counts, derives an independent nearest-old-vertex mapping from local geometry, records the mapping and frame hashes, and advances the state only after the mapping succeeds.

This is observer provenance, not remesh control. A fission event or an unknown event is rejected before state mutation.
