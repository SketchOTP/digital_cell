# DC-DEV-007 — Active External-Contact Regulation

DC-DEV-007 qualifies the first bounded autonomous environmental sensorimotor
behavior. It uses the accepted DC-DEV-006 inert circular obstacle and the
existing chain:

`contact -> contact_stimulus_i -> distributed regulation -> DC-DEV-005 adaptation -> D-091-funded contractility -> mechanics`

The assay does not add a sensor, actuator, trace, world primitive, planner,
reward, fitness, evolution, or DC-DEV-008 work. The environment supplies only
the already-qualified local force and non-semantic contact signal; the
mechanics solver remains the coordinate authority.

The fixed preregistered horizon is 120 accepted steps at `MechParams.dt`, or
2.4 accepted simulated-time units. The primary metric is integrated contact
penetration:

`J_contact = sum(accepted steps) sum(boundary penetration_i * dt)`

The assay compares matched active, motor-off, and zero-reserve arms, then
compares naive, experienced, and recovered active responses.
