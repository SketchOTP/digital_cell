# DC-DEV-021 ENTRY-001 implementation contract

`ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1` is an explicit, opt-in M2
contractility adapter for `MaturationCoupledV4`. It reuses the frozen
DC-DEV-004 activity-to-edge-tension mapping and cost, but funds the requested
tension from the current absolute amount of interior `A`.

The adapter requires finite positive physical area, delegates all vertex
movement to the existing chemistry-core mechanics authority, and spends the
actual funded amount only after an accepted mechanics step. The spent amount
is removed from `A` and added to `W`; `R` is not used, created, or consumed.

`apply_local_activated_energy_contractility_with_stick_slip` composes the new
funding path with the unchanged DC-DEV-011 local isotropic stick-slip law.
Both new APIs are inactive unless explicitly called. The historical
reserve-funded APIs retain their existing schemas and behavior, and the
production selector remains `MaturationCoupledV4` with reserve OFF.

The ENTRY-001 assay is a fixed asymmetric local-activity feasibility input,
not a resource-seeking policy. It has matched active, motor-off,
no-substrate, zero-A, oracle, passive, and 180-degree rotation controls.
It establishes actuator feasibility only; autonomous resource acquisition is
not claimed.
