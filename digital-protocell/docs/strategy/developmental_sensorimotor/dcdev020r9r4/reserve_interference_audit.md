# DC-DEV-020-R9-R4 reserve interference audit

Status: observer-only diagnostic; architect review pending.

The audit starts from `f9bc1d5bffe828b2599c85d4fcbbabdf7f3e3ff3`, uses the frozen
ConservativeV2/D-091 path, seed `2`, and exactly 5,000 accepted steps per arm.
The ordinary reaction path remains `Full`; diagnostic modes are explicit and
reuse the existing reserve, structural, membrane, transport, and mechanics
kernels.

## Controls and result

The V20 control reproduces 8/8. The V21 control reproduces the reserve-bearing
negative with 4/8 gates. In the full reserve arm, A→R is `147.585809275616`,
R→A is `46.83472463956206`, R→W is `7.440433037092543`, rejected reserve
steps are zero, and reserve closure residual is `9.155664961633069e-11`.

The four required ablations were run independently. `STORE_OFF` improves the
replacement metrics to the reserve-off values, but it is not a reserve repair
and the explicit maintenance-priority shadow is the causal test required by
this directive.

The shadow keeps release and loss before maintenance and defers only A→R
storage until after structural and membrane maintenance. It reduces
A→R-before-later-demand from `147.585809275616` to `0`, while preserving
A→R=`147.5982725689982`, R→A=`46.83967808924662`, R→W=`7.441286018803014`,
zero rejects, and strict closure. It does not restore Gate-1 qualification:
`R_m=0.8399735283623063`, `R_b=5.578002846425376`, and
`R_C=1.37732517447396`.

Gate 5 confirms positive R→A during both replete and starvation/downshift
phases, zero reserve rejects, strict reserve closure, and a reserve-off
comparison. Gate 4 actual full certification was correctly skipped because
the shadow did not restore Gate 1.

## Classification and boundary

`DCDEV020R9R4_STORAGE_CAUSAL_PRIORITY_INSUFFICIENT`

This is a diagnostic negative: pre-maintenance storage is causally present but
the parameter-free maintenance-priority ordering is insufficient by itself.
No production reserve repair, recycling, salvage, parameter tuning, source or
sink change, behavior change, or DC-DEV-021 work is authorized.

Compact evidence is under `experiments/generated/dcdev020r9r4/`; dense JSONL
ledgers remain local/external to Git and are bound by the recorded SHA-256
manifest.
