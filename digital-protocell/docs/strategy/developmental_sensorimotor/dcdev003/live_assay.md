# Live growth/remesh assay

The assay runs the existing `coupled_step_growth` path for 1,000 accepted steps with ordinary mechanics and growth enabled and fission disabled. The regulator is updated from an immutable frame after each material step.

Observed remesh continuity events: `24 -> 48 -> 72 -> 96` vertices, three split events survived. Activity remained valid, bounded, and nonzero after every accepted remesh. The regulator-on and regulator-off material trajectories were serialized-hash identical at every step.

The regulator is therefore an internal passenger in this bounded assay. It does not modify material fields or alter the organism trajectory.
