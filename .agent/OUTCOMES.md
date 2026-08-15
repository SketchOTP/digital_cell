# Project Outcome Ledger Template

After adoption, this append-only ledger records results for project directives. Every live outcome must reference one local directive ID.

## Entry schema after adoption

Use live outcome headings only after adoption. The following schema is instructional and is not a live entry:

Allowed adopted-project outcome states: `COMPLETE`, `PARTIAL`, `BLOCKED`, `FAILED`, `CANCELLED`, `SUPERSEDED`. Do not rewrite earlier entries; append corrections referencing the original.

## D-20260815-dcdev001a-architecture-selection - PARTIAL

- Outcome ID: OUT-DCDEV001A-ARCHITECTURE-SELECTION
- Supersedes outcome: none
- Closed: 2026-08-15T16:00:00-04:00
- Acceptance: PARTIAL
- Summary: Clean scientific base established, governance carry-forward classified, external source-level patterns compared, and one bounded hybrid architecture selected; R1 remediation remains open.
- Changed areas: governance reconciliation, developmental/sensorimotor strategy documentation, dcdev001 machine-readable artifacts.
- Validation:
  - Governance ADOPTED validator - PASSED
  - JSON parse and required-file manifest check - PASSED
  - chemistry-core d088 tests - PASSED
  - phase1-certifier metrics_semantics - PASSED
  - full workspace test - BLOCKED by pre-existing missing d008/stage_e_balance/attempt_003/result.json
  - Rust formatting check - BLOCKED by pre-existing clean-base formatting drift
- Remaining risks: R1 exact-head workflow and architect review.
- Blockers: no implementation authorized.
- Follow-up directive: D-20260815-dcdev001a-r1

## D-20260815-dcdev001a-r1 - PARTIAL

- Outcome ID: OUT-DCDEV001A-R1
- Supersedes outcome: OUT-DCDEV001A-ARCHITECTURE-SELECTION
- Closed: 2026-08-15T07:04:51-04:00
- Acceptance: PARTIAL
- Summary: R1 package is prepared for exact-head validation; no scientific implementation has started.
- Changed areas: provenance manifests, first-slice contract, governed disposition records, source license record, current state, and scoped workflow.
- Validation:
  - JSON parse and contract-boundary checks - PASSED
  - governance ADOPTED validator - BLOCKED by active-ledger schema corrections in progress
  - remote exact-head workflow - NOT RUN
- Remaining risks: remote workflow identity/conclusion and architect re-review.
- Blockers: exact-head validation is not yet complete.
- Follow-up directive: none

## D-20260815-dcdev002-local-regulatory-substrate - PARTIAL

- Outcome ID: OUT-DCDEV002-LOCAL-REGULATORY-SUBSTRATE
- Supersedes outcome: none
- Closed: 2026-08-15T11:35:00-04:00
- Acceptance: PARTIAL
- Summary: Isolated `regulatory-core` substrate, frozen Gates -1 through 11, generated evidence, documentation, governance records, and scoped CI were completed from the exact DC-DEV-001A accepted head. The coder package is awaiting architect review.
- Changed areas: regulatory-core workspace crate, immutable material-frame adapter, DC-DEV-002 artifacts/docs/workflow, and current governance state.
- Validation:
  - Governance ADOPTED validator - PASSED
  - Regulatory-core focused suite - PASSED (12 tests)
  - Phase-1 metrics semantics - PASSED (4 tests)
  - D-088 focused regression - PASSED (4 tests)
  - Evolution-harness regression - PASSED (40 tests)
  - Exact-head GitHub Actions run 31892904994 on 8d8d637d157cd79ed4e6bf4fc8124e6ac3837275 - PASSED
  - Full workspace fixture path - NOT RUN (pre-existing D-008 fixture boundary)
- Remaining risks: architect inspection of the pushed PR and scientific interpretation of the bounded substrate.
- Blockers: architect exact-head review; DC-DEV-003 remains unauthorized.
- Follow-up directive: none
