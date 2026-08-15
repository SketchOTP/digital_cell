# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T07:14:02-04:00`

## Active state after adoption

- Local directive ID: `D-20260815-dcdev002-local-regulatory-substrate`
- External directive ID: `DC-DEV-002`
- Objective: `Implement a minimal distributed regulatory substrate with local persistence, bounded neighbor propagation, one non-semantic local physical transducer, deterministic dynamics, and complete read-only isolation from certified organism biology.`
- Current status: `VALIDATING`
- Acceptance: `DC-DEV-002 local Gates -1 through 11 pass; exact-head remote CI and architect review remain pending.`
- Current phase: `DC-DEV-002 gate package from entry 8caf5a19061b0ad34723333e979f30637bdf2c2d.`
- Expected or actual touched areas: `regulatory-core crate, DC-DEV-002 generated artifacts and documentation, scoped workflow, current governance state`
- Immediate next action: `Run exact-head remote validation, record its evidence, and request architect re-review.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- The DC-DEV-002 regulator receives only `LocalMaterialFrameV1`; topology changes fail closed and seed is provenance-only.
- Entry authority is `8caf5a19061b0ad34723333e979f30637bdf2c2d`.
- Implementation work is on `strategy/dc-dev-002-local-regulatory-substrate`.

## Last validation after adoption

- Command or check: `regulatory-core`, Phase-1 metrics semantics, D-088, and evolution-harness focused regressions using sanctioned Rust 1.89 compiler paths
- Result: `PASSED`

## Risks

- The full workspace test remains unavailable because of the pre-existing missing D-008 fixture; R1 workflow deliberately does not invoke it.
- Exact remote workflow identity and conclusion must be recorded after the final package commit.

## Blockers

- Exact-head remote CI and architect re-review are required before any later directive.

## Pending decisions

- None for architecture selection; DC-DEV-002 is authorized and Gate 7 remains outside scope.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
