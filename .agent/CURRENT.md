# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-17T00:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260817-dcdev018r1-closed-loop-feasibility`
- External directive ID: `DC-DEV-018-R1`
- Objective: `Resolve closed-loop N/F-to-A source feasibility, state-dependent sink coupling, finite-resource sufficiency, and failed-controller reachability without adding a controller.`
- Current status: `VALIDATING`
- Acceptance: `Local exact entry parity, gain-1 counterfactual parity, ideal source feasibility, finite-resource upper-bound result, and generated evidence are complete; pushed draft PR, exact-head remote CI, and architect review remain pending.`
- Current phase: `DC-DEV-018-R1 observer-only feasibility audit complete locally; final classification recorded.`
- Expected or actual touched areas: `assay-only counterfactual reaction entry point, regulatory-core example registration, R1 evidence/docs/workflow, governance`
- Immediate next action: `Push the R1 branch and open a draft PR against DC-DEV-016; do not start DC-DEV-018-R2 or DC-DEV-019.`

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
- Implementation work is on `strategy/dc-dev-016-metabolic-break-even` based on `strategy/dc-dev-015-metabolic-restoration-audit`.
- DC-DEV-015 starts exactly at `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` from `strategy/dc-dev-013-resource-contact-feeding`; DC-DEV-014/PR #23 is closed, unmerged negative evidence and is not imported.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 DC-DEV-018-R1 example check/run, exact entry parity, and evidence inspection`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.
- The broad chemistry-core lib suite has one unrelated clean-base failure in `d029_analysis::tests::nnls_recovers_known_alpha_beta`; the R1 assay and regulatory-core tests pass.

## Blockers

- Exact-head remote CI, draft PR creation, and independent architect review remain open. The committed DC-DEV-018 artifact lacks the per-step M4 error trace; exact trace reconstruction is not claimed. No metabolism repair, tuning, new hunger state, behavior, DC-DEV-018-R2, or DC-DEV-019 is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014, parameter repair, parameter screening, navigation, resource seeking, and evolution remain unauthorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
