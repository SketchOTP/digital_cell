# Remesh and unsupported boundaries

Growth and remeshing remain owned by the existing chemistry and mesh mechanics
paths. The environment only re-observes the current geometry each accepted
step. DC-DEV-003 continuity maps the local regulatory activity, and
DC-DEV-005 maps the existing adaptation trace through ordinary split/merge
events.

The environment does not decide growth, remeshing, metabolism, or heredity.
Fission and unknown topology state transfer remain explicit fail-closed errors
in both the continuity and plasticity adapters. No fission inheritance is
implemented.

The bounded force hook is the only post-Phase-1 mechanics addition. Certified
Phase-1 biology and equations remain unchanged; the hook is a post-Phase-1
external-force boundary with exact zero-contact legacy parity.

