# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-18T00:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260818-dcdev020r7-on-policy-zero-drift-audit`
- External directive ID: `DC-DEV-020-R7`
- Objective: `Determine why the accepted R6 N/F power law failed restoration by solving the physical zero-drift requirement on its own induced states and replaying the frozen R5 observers without refitting.`
- Current status: `VALIDATING`
- Acceptance: `R6 is architect-accepted negative at f01b716d9051c9f0114f3c5c0d1b123e2df037cf with exact-head CI 32187547222. R7 reproduces R6, obtains 480/480 finite monotone roots, closes drift exactly, finds frozen NF insufficient only by ambiguity 0.26505161065124994 while NFA passes, and classifies DCDEV020R7_NFA_COORDINATE_REQUIRED_ON_POLICY.`
- Current phase: `DC-DEV-020-R7 observer-only evidence packaging and exact-head validation; no production law or DC-DEV-021 work is authorized.`
- Expected or actual touched areas: `R7 observer example registration, compact evidence, external dense-ledger seal, R7 documentation, governance, and scoped CI`
- Immediate next action: `Complete preservation, push the R7 branch, open a draft PR, verify exact-head CI, and return for architect review; do not construct an NFA law or begin DC-DEV-021.`

## Temporary task-relevant facts

- The exact scientific base for R2 is `1e242f28152797b512e25cd56c7b718e45d6ca97`; the prior R1 head is `876012f8888b074285c55167613471a59d4be25d`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `8d6fe59397cabfa47bc1d8103acd68f544acc190`.
- DC-DEV-007 is architect-accepted; its active contact chain remains preserved and is exercised by the DC-DEV-008 preservation workflow.
- DC-DEV-010 / PR #19 is closed, unmerged negative evidence and must not be imported.
- R5 is accepted as `DCDEV020R5_ACCEPTED`, `DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT`, and `ENDPOINT_SURROGATE_NOT_LOCAL_REQUIREMENT` at `d215cfc00ce70517e25fa7c3b51b13d85d9ce521`; R6 work is isolated on `strategy/dc-dev-020r6-nf-power-law-source`.
- R6 is accepted as `DCDEV020R6_ACCEPTED_NEGATIVE` and `DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE` at `f01b716d9051c9f0114f3c5c0d1b123e2df037cf`; `NF_POWER_LAW_RESTORATION_ROUTE_CLOSED` does not close NF information sufficiency.
- DC-DEV-015 starts exactly at `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` from `strategy/dc-dev-013-resource-contact-feeding`; DC-DEV-014/PR #23 is closed, unmerged negative evidence and is not imported.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 R7 compile, 2 focused observer tests, exact R6 replay, 480 physical roots, frozen NF/NFA replay, drift closure, exact-root oracle, and governed external dense-ledger SHA-256`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Exact-head remote CI and independent architect review remain open. No NFA production law, refit, parameter tuning, additional kinetic family, behavior, or DC-DEV-021 is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014, parameter repair, parameter screening, navigation, resource seeking, and evolution remain unauthorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
