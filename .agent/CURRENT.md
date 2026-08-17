# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-16T00:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260816-dcdev013-resource-contact-feeding`
- External directive ID: `DC-DEV-013`
- Objective: `Determine whether actual local contact with finite N/F resource can causally increase the organism's own resource acquisition through the accepted regulator, funded contractility, and DC-DEV-011 stick-slip path.`
- Current status: `VALIDATING`
- Acceptance: `Frozen 480-step qualification produced the preregistered negative result; local production tests and exact frozen evidence are complete; stacked draft PR and remote CI are pending.`
- Current phase: `DC-DEV-013 evidence and governance package are prepared for pushed draft-PR validation; architect review is next.`
- Expected or actual touched areas: `regulatory-core spatial_resource production interface, DC-DEV-013 assay/docs/artifacts/workflow, scoped CI, current governance state`
- Immediate next action: `Push DC-DEV-013, open the stacked draft PR against DC-DEV-011, run exact-head remote CI, and return for architect review; do not begin DC-DEV-014.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `8d6fe59397cabfa47bc1d8103acd68f544acc190`.
- DC-DEV-007 is architect-accepted; its active contact chain remains preserved and is exercised by the DC-DEV-008 preservation workflow.
- DC-DEV-010 / PR #19 is closed, unmerged negative evidence and must not be imported.
- Implementation work is on `strategy/dc-dev-011-local-stick-slip-traction` based on `strategy/dc-dev-009-motility-feasibility-audit`.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 regulatory-core tests, exact DC-DEV-013 assay, evidence validation, and governance validation`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Exact-head remote CI and independent architect review remain open. The first failed scientific gate is preserved as a negative result; no tuning is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014, parameter repair, parameter screening, navigation, resource seeking, and evolution remain unauthorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
