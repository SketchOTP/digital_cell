# DC-DEV-013 implementation contract

## Production boundary

`FiniteSpatialResourceRegionV1::local_contact_signal` in
`crates/regulatory-core/src/spatial_resource.rs` is the sole reusable sensing
interface.  It reuses the exact `edge_exposed` predicate used by `uptake` and
returns one value per material edge: `1.0` for current contact with available
N and F, otherwise `0.0`.  Empty or depleted inventory produces an all-zero
signal.  It reads no target, distance, gradient, world direction, reward, or
future state and does not mutate the mesh or resource.

## Causal composition

Each accepted step is:

1. observe current local contact;
2. construct the existing continuity frame from that signal only;
3. advance `ContinuityNetworkV1`;
4. apply unchanged reserve-funded contractility through unchanged DC-DEV-011
   stick-slip;
5. let chemistry-core mechanics accept the step;
6. execute unchanged finite N/F uptake on the resulting body;
7. record material, reserve, substrate, and conservation ledgers.

Sensor-off forces only the regulator input to zero.  Motor-off retains local
sensing but calls the existing passive stick-slip path.  Zero-reserve retains
the sensor and path but sets reserve `R` to zero.  The empty sham uses the same
geometry with N/F inventory zero.

No chemistry-core equations, mechanics equations, topology, substrate law,
or resource uptake law were changed.
