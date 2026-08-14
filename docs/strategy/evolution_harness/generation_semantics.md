# Generation semantics

Founder generation is zero. A generation increment occurs only when the adapter returns a completed physical fission and the harness registers the resulting offspring. Elapsed time never creates a generation.

The tracker records maximum/mean-ready distribution data, completed births/fissions, lineage depth, first-birth time, and median generation time. A replicate with `max_generation = 0` is automatically `SELECTION_UNTESTABLE_ZERO_GENERATION`; it cannot receive a selection verdict. Rejected numerical attempts are outside this crate's accepted-step inputs.
