# DC-DEV-020-R9-R2 — material-fate and ConservativeV2 requalification

Status: `PROVISIONAL / ARCHITECT REVIEW PENDING`

Authority:

- Entry head: `364599aea8d4a0def3964b1b299fe45edaaaa1b3`
- Branch: `strategy/dc-dev-020r9-mesh-contract-requalification`
- PR: `#44` (draft, unmerged)
- Scope: observer-only; no DC-DEV-021 work

## Protocol

R9-R2 uses the exact ConservativeV2 contract with the D-091 metabolic-reserve
equation lineage. The exact D-015/D-016 replay remains the 5,000-step settle,
480-step deprivation/feed horizon, `dt=0.02`, frozen resource geometry, and
`ReserveParams::derived(80,40,0.5,0.3,2,0.1,area)`. The R9-R2 sustained arms
run 8,000 accepted steps from the deprived state.

The material-fate ledger records N/F delivery and source injection, N/F
consumption, A production, A→C, C→W, A→M, M→W, A→L, L→B, B→L, A→R, R→A,
R→W, direct A decay, and other material loss. Strict material closure and
organized-material reconciliation are recorded independently.

## Actual certifier result

The actual Phase-1 certifier launcher ran Gates 0–7 under ConservativeV2 with
D-091 reserve enabled. The direct local invocation was `3/8` gates passing:

```text
Gate 0 PASS
Gate 1 FAIL — D087_D086_ACCEPTANCE_INVALID
Gate 2 FAIL — D087_D086_REPRODUCTION_FAILURE
Gate 3 FAIL — D087_HELD_OUT_REPRODUCIBILITY_FAILURE
Gate 4 FAIL — D087_PHASE1_ROBUSTNESS_FAILURE
Gate 5 PASS
Gate 6 PASS
Gate 7 FAIL — D087_LINUX_RUNTIME_QUALIFICATION_FAILURE
```

This is a certification regression, not a reason to tune biology or weaken a
gate. The fail-closed R9-R2 classification is therefore:

`DCDEV020R9R2_CONSERVATIVE_CERTIFICATION_REGRESSION`

The exact-head PR artifact from remote CI run `32317704754` also reports `3/8`
with gate vector `[false, false, false, false, false, true, true, true]` and
the merge-checkout conclusion `D087_SOURCE_OR_ARTIFACT_INTEGRITY_FAILURE`.
This differs from the direct local vector only because the remote workflow
checks out the PR merge ref; it does not alter the fail-closed classification.

## Exact replay and sustained observations

The exact D-015/D-016 replay produced seven ConservativeV2 rows with zero
reserve rejects and zero closure residuals. The D-016 derived break-even
finite arm ran 480 steps and had:

- `strict_material_delta = 22.80378792172297`
- `activation_delta = 3.309793281371199`
- `organized_retained_delta = -10.277547850163131`
- `organized_reconciliation_residual = 3.552713678800501e-14`
- `closure_residual = 4.263256414560601e-14`

All four 8,000-step sustained arms had negative final-quarter organized
material slopes. The sealed R6 normal arm had slope
`-0.0024671618089803075`; the D-016 normal arm had
`-0.006401060912727409`; D-016 cprod-deferred had
`-0.006571078286778075`; and R6 cprod-deferred had
`-0.0055143188921619665`.

The finite-arm dominant irreversible-loss route is
`CATALYST_DOMINANT`, with `C→W = 5.29017338017132`. This is an attribution
from the observer ledger only; it does not authorize a catalyst-law or
recycling change.

## Evidence

Compact authoritative artifacts are written under
`digital-protocell/experiments/generated/dcdev020r9r2/`:

- `actual_d087/manifest.json` — actual Gates 0–7 result;
- `r9r1_exact/manifest.json` — exact D-015/D-016 replay;
- `material_fate.json` — finite and sustained fate ledgers;
- `qualification.json` — fail-closed qualification and classification;
- `protocol.json` and `manifest.json` — protocol and result metadata.

No dense raw ledger is introduced. Existing R9-R1 and earlier evidence remain
preserved.

Remote verification: all 21 scoped workflow steps passed at head
`b6633d99d0f8baa7faae6d569215ec8d7ff9c8cd`. The uploaded compact artifact
`dcdev020r9r2-compact-evidence` has SHA-256
`c5de085cffceff448a7dbe20f5f8280a973700ecbf800da6b11ac19cc09e65ea`.

## Governance disposition

Certified Phase-1 equations and production behavior were not changed. The
bounded observer/accounting additions expose direct A decay and route the
actual certifier through the versioned ConservativeV2 contract with D-091
reserve composition. No recycling implementation, source/sink tuning,
controller, behavior, evolution, or DC-DEV-021 work is authorized.

`NEXT_EXECUTION_STARTED:false`
