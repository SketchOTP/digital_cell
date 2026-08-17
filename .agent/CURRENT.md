# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-17T00:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260817-dcdev019r1-continuous-state-requalification`
- External directive ID: `DC-DEV-019-R1`
- Objective: `Requalify the unchanged one-state homeostat through continuous deprivation and finite refeeding, stopping at the first failed metabolic gate.`
- Current status: `VALIDATING`
- Acceptance: `Gate 0 passed; Gate 1 failed because carried-state finite refeeding did not restore stored material. Phases 2–6 are not authorized.`
- Current phase: `DC-DEV-019-R1 Gate 1 negative-result evidence and review package.`
- Expected or actual touched areas: `observer requalification assay, compact evidence, documentation, governance, scoped CI`
- Immediate next action: `Push the bounded negative requalification package, run exact-head remote CI, and return for architect review; do not rerun DC-DEV-019 or begin DC-DEV-020.`

## Temporary task-relevant facts

- The exact R1 entry is `59633ebcc37c936e2d04ca5d53477129ab1dca13`.
- The frozen homeostat production blob is `1172bf792292cfb50269f3f19c01034f446f1af6`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `8d6fe59397cabfa47bc1d8103acd68f544acc190`.
- DC-DEV-007 is architect-accepted; its active contact chain remains preserved and is exercised by the DC-DEV-008 preservation workflow.
- DC-DEV-010 / PR #19 is closed, unmerged negative evidence and must not be imported.
- Implementation work is on `strategy/dc-dev-016-metabolic-break-even` based on `strategy/dc-dev-015-metabolic-restoration-audit`.
- DC-DEV-015 starts exactly at `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` from `strategy/dc-dev-013-resource-contact-feeding`; DC-DEV-014/PR #23 is closed, unmerged negative evidence and is not imported.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 DC-DEV-016 example check/run and evidence inspection`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Exact-head remote CI and independent architect review remain open. DC-DEV-016 is observer-only; no metabolism repair, tuning, new hunger state, behavior, or DC-DEV-017 is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014, parameter repair, parameter screening, navigation, resource seeking, and evolution remain unauthorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
