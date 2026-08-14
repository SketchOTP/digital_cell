# Shared constructor audit

Gate 5 and the corrected D-096 adapter now use the same typed construction
helpers from `chemistry-core/src/d096_allocation.rs`. The adapter no longer
recreates radius, center, chemistry, reserve, transport, or growth state.

The refactor is behavior-preserving for `pre_fission_assay`; the sealed D-096
effects are checked by the existing chemistry regression and R2 exact replay.
The machine-readable source mapping is in `shared_constructor_audit.json`.
