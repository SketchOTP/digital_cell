# Single-generation boundary

Each replicate began with exactly one founder. The runner stopped after the
first founder `FISSION_COMPLETED`, founder `DEATH`, or the frozen horizon.

When a fission occurred, the harness would have recorded the physical fission
and two generation-1 births, checked partition metadata, and stopped before
any daughter step. The assay asserts `maximum_generation_observed <= 1` and
asserts that every birth is marked as a qualified physical-fission copy.

In the observed campaign no fission occurred, so no generation-1 daughter was
created and no daughter was advanced.

