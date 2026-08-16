# DC-DEV-008 Gate Results

Local execution of `dcdev008_gate_assay` at entry `2968882769991f48c987ceb40c719fd351b2e046` passed all nine assay assertions.

- Finite active inventory: `16.0` total N/F; final inventory `10.322397050168249`.
- Active uptake: N `2.838801474915872`, F `2.838801474915872`.
- World loss equals organism gain: exact within `1e-12`.
- Noncontact uptake: exactly `0.0`.
- Resource-free uptake: exactly `0.0`.
- Final activated resource A: active `0.5113897710193833`; resource-free `0.5068165398395282`.
- Active region was locally exposed for all `120` accepted steps.

The existing global `mesh_transport` path, certified permeability function,
reaction stoichiometry, reserve chemistry, growth law, and DC-DEV-007 behavior
are preserved. Remote preservation and exact-head CI remain required before
qualification.
