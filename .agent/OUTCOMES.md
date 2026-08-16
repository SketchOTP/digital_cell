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

## D-20260815-dcdev003-regulatory-topology-continuity - PARTIAL

- Outcome ID: OUT-DCDEV003-REGULATORY-TOPOLOGY-CONTINUITY
- Supersedes outcome: none
- Closed: 2026-08-15T15:00:00-04:00
- Acceptance: PARTIAL
- Summary: The bounded observer-only continuity layer and live assay are implemented from the exact DC-DEV-002 accepted head; final pushed-head CI and architect review remain pending.
- Changed areas: regulatory-core continuity module and immutable continuity adapter, DC-DEV-003 assay/artifacts/docs/workflow, current governance state.
- Validation:
  - Regulatory-core continuity suite - PASSED (15 tests before final event-validation additions)
  - Live growth/remesh assay - PASSED locally (3 split remesh events, 24 to 48 to 72 to 96 vertices)
  - Regulator-on/off organism trajectory comparison - PASSED locally (serialized hashes equal)
  - Governance ADOPTED validation - PASSED
  - Phase-1 focused regression - PASSED (4 tests)
  - D-088 focused regression - PASSED (4 tests)
  - Evolution-harness regression - PASSED (40 tests)
  - Exact-head remote CI run 31913029009 on fafa642c97d85566c696aad61ac57fe777ac94c0 - PASSED
  - Full workspace fixture path - NOT RUN (pre-existing D-008 fixture boundary)
- Remaining risks: final CI may expose formatting or integration defects; architect must verify the mapping semantics and exact remote head.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260815-dcdev004-energy-coupled-local-contractility - PARTIAL

- Outcome ID: OUT-DCDEV004-ENERGY-COUPLED-LOCAL-CONTRACTILITY
- Supersedes outcome: none
- Closed: 2026-08-15T19:17:08-04:00
- Acceptance: PARTIAL
- Summary: The first bounded local contractility implementation and local gate assay are complete from the accepted DC-DEV-003 head; exact-head scoped remote CI passed and architect review remains pending.
- Changed areas: bounded chemistry-core edge-tension mechanics hook, regulatory-core contractility adapter, DC-DEV-004 assay/artifacts/docs/workflow, current governance state.
- Validation:
  - Regulatory-core suite - PASSED (18 tests)
  - DC-DEV-004 local gate assay - PASSED (Gates 0 through 7)
  - D-091 reserve R selected as existing funding resource; R to W expenditure observed
  - Zero-activity exact legacy parity - PASSED
  - Zero-resource no-actuation parity - PASSED
  - Local deformation and tensile closed-loop reduction - PASSED
  - Repeated-actuation metabolic limitation - PASSED
  - DC-DEV-003 remesh continuity compatibility - PASSED locally
  - Final preservation regressions and exact-head remote CI run 31914737565 on 0d45396f394f3a41f3b5b60cc46f1ce074a66bf0 - PASSED
  - Architect review - PENDING
- Remaining risks: architect must inspect the physical interpretation and frozen resource conversion.
- Blockers: architect review.
- Follow-up directive: none

## D-20260815-dcdev005-local-experience-dependent-plasticity - PARTIAL

- Outcome ID: OUT-DCDEV005-LOCAL-EXPERIENCE-DEPENDENT-PLASTICITY
- Supersedes outcome: none
- Closed: 2026-08-15T20:33:20-04:00
- Acceptance: PARTIAL
- Summary: The preregistered one-trace local plasticity implementation, habituation/recovery assay, remesh continuity checks, and scoped regressions pass locally from the accepted DC-DEV-004 head; exact-head remote CI and architect review remain pending.
- Changed areas: regulatory-core plasticity adapter, DC-DEV-005 assay/artifacts/docs/workflow, and current governance state.
- Validation:
  - Governance ADOPTED validation - PASSED
  - Regulatory-core suite - PASSED (22 tests)
  - DC-DEV-005 gate assay - PASSED (Gates 0 through 6)
  - DC-DEV-002, DC-DEV-003, and DC-DEV-004 preservation assays - PASSED locally
  - Phase-1 metrics semantics - PASSED (4 tests)
  - D-088 focused regression - PASSED (4 tests)
  - Evolution-harness regression - PASSED (40 tests)
  - Exact-head remote CI run 31917550450 on 9fe97069185ac48d4e979fe358b12d32433eb6d7 - PASSED
  - Architect review - PENDING
- Remaining risks: the architect must inspect the first history-dependent response claim.
- Blockers: architect review.
- Follow-up directive: none

## D-20260815-dcdev006-minimal-spatial-contact-environment - PARTIAL

- Outcome ID: OUT-DCDEV006-MINIMAL-SPATIAL-CONTACT-ENVIRONMENT
- Supersedes outcome: none
- Closed: `2026-08-15T22:36:55-04:00`
- Acceptance: PARTIAL
- Summary: One deterministic static obstacle, bounded local mechanics force hook, penetration-normalized contact signal, existing regulatory/plasticity coupling, local repeated-contact assay, remesh continuity, and fission fail-closed boundaries are implemented from the accepted DC-DEV-005 head. Architect qualification remains pending.
- Changed areas: bounded `chemistry-core` mesh mechanics force hook, `regulatory-core` spatial adapter and external-force wrappers, DC-DEV-006 assay/artifacts/docs/workflow, and current governance state.
- Validation:
  - Regulatory-core focused suite - PASSED
  - DC-DEV-006 gate assay - PASSED
  - DC-DEV-005 zero-contact trajectory parity - PASSED
  - Contact force locality and deterministic transduction - PASSED
  - Repeated-contact experience dependence and recovery - PASSED
  - Remesh continuity and fission fail-closed boundaries - PASSED
  - Exact-head remote CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: remote exact-head validation and independent architect interpretation of the external-force boundary.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260815-dcdev007-active-external-contact-regulation - PARTIAL

- Outcome ID: OUT-DCDEV007-ACTIVE-EXTERNAL-CONTACT-REGULATION
- Supersedes outcome: none
- Closed: `2026-08-15T23:45:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The fixed-horizon DC-DEV-007 integration assay and evidence package pass locally from the architect-accepted DC-DEV-006 head. Exact-head remote CI and architect review remain pending.
- Changed areas: regulatory-core example registration, DC-DEV-007 assay/artifacts/docs/workflow, and current governance state; no chemistry-core or certified Phase-1 biology/equations changed.
- Validation:
  - DC-DEV-007 Gates 0-8 local assay - PASSED
  - Active integrated contact penetration lower than motor-off - PASSED
  - Zero-reserve passive-contact control - PASSED
  - Experience-dependent response and recovery - PASSED
  - Ordinary remesh continuity and fission fail-closed boundary - PASSED
  - Scoped preservation suite - PENDING
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote exact-head validation and independent architect interpretation of the causal sensorimotor claim.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260815-dcdev007-active-external-contact-regulation - PARTIAL

- Outcome ID: OUT-DCDEV007-ACTIVE-EXTERNAL-CONTACT-REGULATION-REMOTE
- Supersedes outcome: OUT-DCDEV007-ACTIVE-EXTERNAL-CONTACT-REGULATION
- Closed: `2026-08-15T23:34:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head remote CI passed for PR #15 at commit `ad65385c247acda681d10ce182943bb5d28afd6e`; architect review remains pending.
- Changed areas: DC-DEV-007 assay/artifacts/docs/workflow and governance records; no chemistry-core or certified Phase-1 biology/equations changed.
- Validation:
  - Governance ADOPTED validation - PASSED
  - DC-DEV-007 artifact and boundary validation - PASSED
  - Scoped formatting - PASSED
  - Regulatory-core and DC-DEV-007 assay - PASSED
  - DC-DEV-002 through DC-DEV-006 preservation assays - PASSED
  - Phase-1 metrics semantics - PASSED
  - D-088 focused regression - PASSED
  - Evolution-harness regression - PASSED
  - Exact PR-head checkout assertion - PASSED
  - Exact-head remote CI run 31924373883 - PASSED
  - Architect review - PENDING
- Remaining risks: independent architect interpretation of the causal sensorimotor claim.
- Blockers: architect review.
- Follow-up directive: none

## D-20260815-dcdev006-minimal-spatial-contact-environment - PARTIAL

- Outcome ID: OUT-DCDEV006-MINIMAL-SPATIAL-CONTACT-ENVIRONMENT-REMOTE
- Supersedes outcome: OUT-DCDEV006-MINIMAL-SPATIAL-CONTACT-ENVIRONMENT
- Closed: `2026-08-15T22:46:41-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head remote validation passed for PR #14 at commit `50eb7383d2df58395b4da906dc7436a00f1ad027`; independent architect review remains required.
- Changed areas: governance current-state and outcome records only; no scientific implementation or evidence artifacts changed.
- Validation:
  - Governance ADOPTED validation - PASSED
  - DC-DEV-006 artifact and boundary validation - PASSED
  - Scoped formatting - PASSED
  - Regulatory-core and DC-DEV-006 assay - PASSED
  - DC-DEV-002 through DC-DEV-005 preservation assays - PASSED
  - Phase-1 metrics semantics - PASSED
  - D-088 focused regression - PASSED
  - Evolution-harness regression - PASSED
  - Exact-head remote CI run 31922533764 - PASSED
  - Architect review - PENDING
- Remaining risks: independent architect review of the external-force and contact-signal boundary.
- Blockers: architect review.
- Follow-up directive: none

## D-20260815-dcdev006-minimal-spatial-contact-environment - PARTIAL

- Outcome ID: OUT-DCDEV006-MINIMAL-SPATIAL-CONTACT-ENVIRONMENT-EXACT-HEAD
- Supersedes outcome: OUT-DCDEV006-MINIMAL-SPATIAL-CONTACT-ENVIRONMENT-REMOTE
- Closed: `2026-08-15T22:59:22-04:00`
- Acceptance: `PARTIAL`
- Summary: The scoped workflow now checks out and asserts the actual PR head; exact-head remote CI run 31923037384 passed at commit `30f9b0cab792ac6742d1820ad0f5677f29af5631`; independent architect review remains required.
- Changed areas: scoped DC-DEV-006 workflow checkout/assertion and governance current-state/outcome records; no scientific implementation changed.
- Validation:
  - Governance ADOPTED validation - PASSED
  - DC-DEV-006 artifact and boundary validation - PASSED
  - Scoped formatting - PASSED
  - Regulatory-core and DC-DEV-006 assay - PASSED
  - DC-DEV-002 through DC-DEV-005 preservation assays - PASSED
  - Phase-1 metrics semantics - PASSED
  - D-088 focused regression - PASSED
  - Evolution-harness regression - PASSED
  - Exact PR-head checkout assertion - PASSED
  - Exact-head remote CI run 31923037384 - PASSED
  - Architect review - PENDING
- Remaining risks: independent architect review of the external-force and contact-signal boundary.
- Blockers: architect review.
- Follow-up directive: none
