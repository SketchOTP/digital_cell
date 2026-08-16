# DC-DEV-006 — Minimal spatial contact environment

DC-DEV-006 places the accepted DC-DEV-005 organism in one deterministic 2D
world containing one static inert circular obstacle. The only new external
relation is `contact_stimulus_i`, a bounded local signal derived from boundary
penetration and the bounded contact force returned by the world.

The environment never writes organism coordinates. It returns local forces to
the bounded chemistry-core mechanics hook; mechanics remains authoritative for
movement. The signal enters the already accepted local regulatory frame and the
existing DC-DEV-005 adaptation trace. No new actuator, plasticity trace,
reward, fitness, semantic label, resource ecology, or controller is present.

Entry: `4da04a5cf8153e4ab31603965eeba305ad4bb721` from
`strategy/dc-dev-005-local-plasticity`.

The local package is pending exact-head remote CI and independent architect
review. DC-DEV-007 is not started.

## Package

- [`implementation_contract.md`](implementation_contract.md)
- [`contact_assay.md`](contact_assay.md)
- [`remesh_and_boundaries.md`](remesh_and_boundaries.md)
- [`gate_results.md`](gate_results.md)
- [`timescale_preregistration.md`](timescale_preregistration.md)
- [`final_conclusion.md`](final_conclusion.md)
- [`timescale_preregistration.json`](timescale_preregistration.json)

