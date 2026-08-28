# DC-DEV-020-M1 REPLAN-002-R5-R1

## Zero-area closure and refeed semantics audit

Status: observer-only diagnostic; architect acceptance pending.

Starting authority: `0cdab2f5dcccfe6b7f41936e546b96ffe8df7c4b` on `strategy/dc-dev-020r9-mesh-contract-requalification`.

## Scope and preservation

This audit does not modify V4 biology, mechanics, chemistry, transport, topology, D-087, tolerances, or the production default. It records stage-level state before and after the existing `TRANSPORT -> REACTIONS -> MECHANICS -> REMESH -> REBOND` sequence and audits the already-committed R5 refeed code. Dense ledgers belong on Atlas; only the compact JSON package is committed.

## Findings

- R5 reproduces as `M1_V4_DEATH_QUALIFICATION_UNRESOLVED`; its reported maximum raw strict-material delta is `0.45051928554230614`.
- The maximum raw delta occurs in `TRANSPORT` at step `8177`, before the first failed mechanics return at `8566`. At that step the internal C+A+W amount removed is the reported `0.4505192855423097`; the transport ledger's expected export is larger because its existing floor/clamp semantics overstate the requested export.
- The first signed-area nonpositive state is step `7675`; the first unexplained stage residual is transport at step `7684` (`0.005153409950736487`).
- The first `mechanics_step == false` is step `8566`, transitioning from area `1.1102230246251565e-16` to `0.0`. The mechanics call mutates the mesh before returning false (`state_changed_despite_false=true`).
- The authoritative full-runtime qualification caller stops with an error when mechanics returns false. The R5/R4 diagnostic harnesses ignore the return, so the deep R5 continuation is harness-only invalid continuation.
- R5 refeeding is `SEALED_INTERNAL_DELIVERY_UPPER_BOUND`: it generates a healthy reference schedule and directly inserts `source.n / area` and `source.f / area` into clone interior concentrations. It is not a live spatial-resource opportunity test.

## Validity boundary

R5 checkpoints `5277` and `6130` are `VALID_PRE_FAILURE`. Checkpoints `10200` and `150200` are `POST_INVALID_CONTINUATION`. R4's first observer-collapse evidence at `6130` is pre-failure valid; its 150k material trajectory requires requalification. Historical evidence files are not rewritten.

## Classification

`M1_R5_ALTERNATE_CLOSURE_CAUSE_CONFIRMED`

The R5 maximum residual is attributable to the existing transport internal C/A/W export at step 8177, not to the later zero-area transition. The later failed mechanics transition independently invalidates deeper continuation, and the direct-internal-injection refeed independently fails the physical resource-opportunity requirement. This result does not establish M1 or authorize M2.

## Canonical evidence

- Compact: `digital-protocell/experiments/generated/dcdev020m1replan002r5r1/`
- Dense: `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r1/`
