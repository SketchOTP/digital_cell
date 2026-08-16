# DC-DEV-012: Endogenous Stochastic Polarity

DC-DEV-012 tests whether an internal finite stochastic polarity process can
create asymmetric regulatory activity without an environmental cue and feed
the already accepted DC-DEV-011 funded stick-slip path.

The production implementation is
`crates/regulatory-core/src/endogenous_polarity.rs`. It owns only 1,000
conserved abstract tokens on the fixed 24-patch material ring. The assay is
`examples/dcdev012_gate_assay.rs`.

The frozen reference set is `N=1000`, `k_feedback=10.0`, `k_off=9.0`,
`k_on=1e-4`, `D_membrane=1.2`, and `dt=0.02`. The four allowed events are
association, same-patch recruitment, nearest-neighbor diffusion, and
dissociation. The diffusion hop rate is derived as `q=D/h^2` from the
settled body's measured ring spacing.

Formal qualification uses exactly seeds `12001..12024`, 1,500 accepted steps,
zero environmental stimulus, fixed topology, disabled plasticity, and four
matched physical arms. No result-dependent parameter screening is allowed.

DC-DEV-012 does not add a target, navigation, resource seeking, new actuator,
new traction law, mechanics change, chemistry change, heredity, or DC-DEV-013.
