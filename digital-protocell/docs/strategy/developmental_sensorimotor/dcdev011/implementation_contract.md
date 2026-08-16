# DC-DEV-011 implementation contract

This is one bounded production substrate adapter. It does not modify certified
chemistry, mesh equations, contractility parameters, or substrate mechanics.

`StickSlipTractionParamsV1` contains exactly one static/kinetic parameter set.
`evaluate_contact` is a pure local vector law. It contains no coordinate,
vertex-index, centroid, world-axis, target, stimulus, regulatory, resource, or
reward input.

The two execution adapters are:

- `apply_stick_slip_to_legacy_mechanics`, for motor-off and zero-reserve arms;
- `apply_local_contractility_with_stick_slip`, for the existing funded local
  contractility path.

Both adapters clone the current mesh only to obtain the local attempted
velocity from the existing accepted mechanics path. They then evaluate local
reaction vectors and call the existing mechanics integrator on the real mesh.
The substrate never writes vertex coordinates. The clone is not an alternate
trajectory authority and its post-step chemistry is discarded.

For stick, the reaction cancels the presented local force and accepted local
sliding velocity is zero within solver tolerance. For slip, the reaction has
fixed magnitude `0.20`, is opposite local attempted motion, and is strictly
below the static limit. The recorded substrate work is the reaction dot the
accepted local displacement and must be non-positive within tolerance.

Direct production tests cover frozen parameters, stick, slip, passivity,
bounded reaction, rotational equivalence, deterministic replay, and the
existing mechanics adapter boundary. Qualification is not executed until this
contract and the preregistered protocol are committed.
