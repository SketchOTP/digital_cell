# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-16T22:30:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260816-dcdev014-homeostatic-exploration`
- External directive ID: `DC-DEV-014`
- Objective: `Determine whether one existing material depletion/restoration signal can causally switch direction-neutral local exploration and reduce exploration after finite N/F restoration.`
- Current status: `VALIDATING`
- Acceptance: `The existing A signal closes depletion/restoration and the fixed assay executes; Gates 1-5 and 7-11 pass, Gate 6 late relief fails, so the current scientific result is DCDEV014_HOMEOSTATIC_EXPLORATION_NOT_ESTABLISHED pending remote CI and architect review.`
- Current phase: `DC-DEV-014 bounded exploration package is prepared for pushed stacked-PR validation; architect review is next.`
- Expected or actual touched areas: `regulatory-core homeostatic exploration module, DC-DEV-014 assay/docs/artifacts/workflow, scoped CI, current governance state`
- Immediate next action: `Push DC-DEV-014, open a stacked draft PR against DC-DEV-013, run exact-head remote CI, and return for architect review; do not begin DC-DEV-015.`

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
- DC-DEV-014 entry authority is `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` on `strategy/dc-dev-013-resource-contact-feeding`; implementation branch is `strategy/dc-dev-014-homeostatic-exploration`.
- The selected existing material signal is `MaterialMesh.interior.a`; accepted replete reference is the seed `A=0.5`, and accepted no-resource maintenance decreased A to `0.303630027599798` after 480 steps.
- Finite N/F plus existing `reactions_step` restored A relative to the matched no-delivery control (`C final A=0.2502233661813926`, `D final A=0.20689179981214934`); this is restoration, not yet late homeostatic relief.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 regulatory-core tests, exact DC-DEV-014 assay, evidence validation, and governance validation`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.
- Exploration is direction-neutral and receives only normalized interior-A need, local topology size, and its deterministic provenance state; it has no resource/contact/coordinate input.

## Blockers

- Exact-head remote CI and independent architect review remain open. Gate 6 late relief is a negative result; no parameter tuning or horizon extension is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014 uses 5,000 settlement steps, 480 assay steps, 160-step analysis windows, and the existing regulator `k_decay=0.5` rate scale; these are frozen for this package.
- DC-DEV-015, parameter repair, parameter screening, navigation, resource seeking, selection, and evolution remain unauthorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
