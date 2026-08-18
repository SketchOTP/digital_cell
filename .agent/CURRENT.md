# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-18T00:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260818-dcdev020r2-allosteric-requalification`
- External directive ID: `DC-DEV-020-R2`
- Objective: `Requalify the broader A-only allosteric architecture with a source-actuation envelope, sequencing audit, correct August DC-DEV-017 replay, and a fail-closed A-only sufficiency gate.`
- Current status: `VALIDATING`
- Acceptance: `R2 Gate 4 failed with DCDEV020_A_ONLY_ALLOSTERIC_COORDINATE_INSUFFICIENT; no derived law, production integration, downstream persistence, exploration, or DC-DEV-021 work was run.`
- Current phase: `DC-DEV-020-R2 observer-only Gate 4 negative-result package; exact-head remote CI and architect review remain pending.`
- Expected or actual touched areas: `R2 observer assay, new compact evidence namespace, R2 documentation, governance, scoped CI`
- Immediate next action: `Validate preservation and governance, push the R2 branch if authority is available, and return for architect review; do not tune, integrate, or begin DC-DEV-021.`

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
- R2 implementation work is on `strategy/dc-dev-020r2-allosteric-requalification` based on prior head `876012f8888b074285c55167613471a59d4be25d`.
- DC-DEV-015 starts exactly at `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` from `strategy/dc-dev-013-resource-contact-feeding`; DC-DEV-014/PR #23 is closed, unmerged negative evidence and is not imported.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 DC-DEV-020-R2 example check/run and evidence inspection`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Exact-head remote CI and independent architect review remain open. R2 Gate 4 failed; no production integration, parameter tuning, second controller, behavior, persistence, exploration, or DC-DEV-021 is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014, parameter repair, parameter screening, navigation, resource seeking, and evolution remain unauthorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
