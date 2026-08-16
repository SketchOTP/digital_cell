# DC-DEV-010 implementation contract

The only new production capability is
`digital-protocell/crates/regulatory-core/src/substrate_traction.rs`.

For each vertex, the adapter receives the pre-step internal force from the
existing mechanics and existing reserve-funded edge contractility. It converts
that local attempted velocity into a reaction opposite the motion. The
longitudinal resistance is `0.25` for positive motion on the frozen substrate
axis and `0.75` for negative motion. Transverse motion uses `0.50`. The
reaction is bounded at `0.45` per vertex before entering the existing
`mechanics_step_with_external_forces` interface.

For accepted velocity `v` and substrate reaction `F_s`, the ledger checks
`F_s dot v * dt <= 0`. Zero attempted motion returns zero reaction. The
substrate never writes coordinates, reads regulatory state, or supplies an
active force.

`contractile_force_vectors` is a read-only reconstruction of the exact funded
edge-tension force used by the existing contractility adapter. It exists only
so the substrate law can be evaluated against the same pre-step internal force
that the movement authority will accept. The existing mechanics module remains
the sole coordinate integrator.

The assay is deliberately fixed-topology and disables growth, remeshing,
fission, obstacles, resource patches, contact sensing, navigation, and
resource sensing. The isotropic arm is a matched diagnostic control, not a
second substrate architecture.

Certified Phase 1 equations were not changed. The bounded post-Phase-1
contractility adapter gained only the read-only force-audit helper needed for
the production substrate boundary.
