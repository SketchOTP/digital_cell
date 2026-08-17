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

## D-20260816-dcdev008-finite-spatial-resource-acquisition - PARTIAL

- Outcome ID: OUT-DCDEV008-FINITE-SPATIAL-RESOURCE-ACQUISITION-LOCAL
- Supersedes outcome: none
- Closed: `2026-08-16T00:35:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Local DC-DEV-008 Gates 0-7 and exact-head remote CI run `31926377883` pass at commit `4e0d31dd1e991e6c983279589d6144dc73b57235`; architect review remains pending.
- Changed areas: regulatory-core assay registration, DC-DEV-008 assay, generated evidence, documentation, scoped workflow, and governance records; no chemistry-core or certified Phase-1 biology/equations changed.
- Validation:
  - DC-DEV-008 local assay Gates 0-7 - PASSED
  - Finite N/F depletion continuation - PASSED at step 543 with zero post-exhaustion uptake
  - Per-step N/F world-to-organism mass conservation - PASSED with zero maximum residual
  - Resource-free and noncontact uptake controls - PASSED
  - Existing A/R coupling and persistence measure - PASSED
  - Ordinary remeshing and fission fail-closed boundary - PASSED
  - Governance ADOPTED validation - PASSED
  - Exact-head remote CI run 31926377883 - PASSED
  - Architect review - PENDING
- Remaining risks: remote exact-head preservation matrix and independent architect interpretation of the finite-resource boundary semantics.
- Blockers: remote exact-head CI and architect review.
- Follow-up directive: none

## D-20260816-dcdev008r1-runtime-boundary-closure - PARTIAL

- Outcome ID: OUT-DCDEV008R1-RUNTIME-BOUNDARY-CLOSURE
- Supersedes outcome: OUT-DCDEV008-FINITE-SPATIAL-RESOURCE-ACQUISITION-LOCAL
- Closed: `2026-08-16T01:00:00-04:00`
- Acceptance: PARTIAL
- Summary: The qualified finite spatial N/F acquisition mechanism is being promoted from the assay into reusable `regulatory-core` production code; the assay now calls the versioned production API and the accepted scientific values remain unchanged.
- Changed areas: `regulatory-core/src/spatial_resource.rs`, regulatory-core exports/tests, DC-DEV-008 assay/workflow/docs, and active governance records.
- Validation:
  - Six direct production-module tests - PASSED
  - DC-DEV-008 assay reproduction - PASSED with accepted values
  - DC-DEV-002 through DC-DEV-007 preservation assays - PASSED locally
  - Governance ADOPTED validation - PASSED
  - Phase-1 focused regression - PASSED (4 tests)
  - D-088 focused regression - PASSED (4 tests)
  - Evolution-harness regression - PASSED (40 tests)
  - Exact-head GitHub Actions run `31938453863` at `746e12514dfdbd5dd3f8a6cd90d10900f8a6b5cf` - PASSED
- Remaining risks: architect inspection of the runtime boundary.
- Blockers: architect review; DC-DEV-009 remains blocked.
- Follow-up directive: none

## D-20260816-dcdev009-free-space-motility-feasibility-audit - PARTIAL

- Outcome ID: OUT-DCDEV009-FREE-SPACE-MOTILITY-AUDIT-LOCAL
- Supersedes outcome: none
- Closed: `2026-08-16T11:51:10-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only fixed-topology audit passes locally. Existing contractility produces shape change but no valid free-space translation attributable to contractile propulsion.
- Changed areas: regulatory-core example registration, DC-DEV-009 observer assay/artifacts/docs/workflow, and current governance state; no chemistry-core source changed.
- Validation:
  - Fixed 24-vertex, 240-step, 4.8 simulated-time active/motor-off assay - PASSED
  - Regulatory trajectory identity - PASSED
  - Equal-and-opposite contractile force accounting - PASSED; max norm `6.804363002006077e-16`
  - Contractility-only centroid displacement - PASSED; `2.473548217003853e-18`
  - Shape change without accepted locomotion - PASSED
  - Artifact audit - PASSED; active-minus-control drift matched baseline force-field integral
  - External prior-art review and three-option architecture comparison - COMPLETE
  - Governance ADOPTED validation - PENDING
  - DC-DEV-002 through DC-DEV-008 preservation - PENDING
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote exact-head validation and architect review of the artifact classification and recommended substrate experiment.
- Blockers: exact-head remote CI and architect review.
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

## D-20260816-dcdev011-passive-isotropic-stick-slip-traction - PARTIAL

- Outcome ID: OUT-DCDEV011-PASSIVE-ISOTROPIC-STICK-SLIP-LOCAL
- Supersedes outcome: none
- Closed: `2026-08-16T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The frozen local isotropic stick-slip mechanism qualified the preregistered four-arm assay locally. Active stick-slip retained `0.004404569847979622` material-centroid displacement, versus `0` motor-off and `0.0018021246021144236` active no-substrate; both stick and slip occurred, passivity and rotational equivalence passed.
- Changed areas: DC-DEV-011 stick-slip production module, assay, compact evidence, documentation, scoped workflow, and governance records; no certified chemistry-core source changed.
- Validation:
  - 35 regulatory-core tests - PASSED
  - 4 Phase-1 metrics tests - PASSED
  - 4 D-088 focused tests - PASSED
  - 40 evolution-harness tests - PASSED
  - DC-DEV-002 through DC-DEV-009 preservation assays - PASSED
  - Scoped formatting - PASSED
  - Exact-head remote CI run `31970906725` at `1a1a97ece380dc5438a568c2642d20de850576fc` - PASSED
- Remaining risks: draft PR inspection and independent architect review.
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

## D-20260816-dcdev009-free-space-motility-feasibility-audit - PARTIAL

- Outcome ID: OUT-DCDEV009-FREE-SPACE-MOTILITY-AUDIT-REMOTE
- Supersedes outcome: OUT-DCDEV009-FREE-SPACE-MOTILITY-AUDIT-LOCAL
- Closed: `2026-08-16T12:08:00-04:00`
- Acceptance: `PARTIAL`
- Summary: DC-DEV-009 observer-only motility feasibility audit is pushed at `5cbd46d34167519748b5b888fd29f4359cbf019a` with draft PR #18 stacked on `strategy/dc-dev-008-spatial-resource-acquisition`; the scientific finding remains that existing free-space motility is not established.
- Validation:
  - Local DC-DEV-009 assay and committed evidence artifacts - PASSED
  - Local DC-DEV-002 through DC-DEV-008 preservation matrix - PASSED
  - Local Phase-1 metrics semantics - PASSED
  - Local D-088 focused regression - PASSED
  - Local evolution-harness regression - PASSED
  - Exact-head remote CI run `31957461248` - PASSED at `5cbd46d34167519748b5b888fd29f4359cbf019a`
  - Draft PR #18 - OPEN and UNMERGED
  - Architect review - PENDING
- Changed areas: observer-only DC-DEV-009 assay, generated evidence, scoped workflow, developmental sensorimotor documentation, and governance records; no chemistry-core changes.
- Remaining risks: independent architect review of the finite-horizon free-space artifact classification and recommended next experiment.
- Blockers: architect review.
- Follow-up directive: none

## D-20260816-dcdev013-resource-contact-feeding - PARTIAL

- Outcome ID: OUT-DCDEV013-LOCAL-RESOURCE-CONTACT-FEEDING
- Supersedes outcome: none
- Closed: `2026-08-16T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The frozen DC-DEV-013 assay completed with the preregistered negative result. Active finite N/F acquisition was `0.354201468008014`, below sensor-off and motor-off at `0.364097551510532`; the first scientific failure was Gate 5 active acquisition benefit.
- Changed areas: regulatory-core local resource-contact observation, DC-DEV-013 assay, generated evidence, documentation, workflow, and governance; no chemistry-core source or certified Phase-1 equations changed.
- Validation:
  - Regulatory-core production tests - PASSED (36 tests)
  - Exact 5,000-step legacy settlement - PASSED
  - Exact 480-step matched five-arm assay - PASSED with negative scientific conclusion
  - Local resource physicality, conservation, empty sham, rotation, passivity, and ownership controls - PASSED
  - Gate 5 active acquisition benefit - FAILED as preregistered negative result
  - Gate 6 contact benefit - FAILED as dependent negative result
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote exact-head validation and independent architect interpretation of the negative feeding result.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260817-dcdev015-metabolic-restoration-audit - PARTIAL

- Outcome ID: OUT-DCDEV015-METABOLIC-RESTORATION-LOCAL
- Supersedes outcome: none
- Closed: `2026-08-17T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only DC-DEV-015 audit reproduced the exact 5,000-step settlement and 480-step deprivation, then found lawful finite N/F delivery and measurable N/F-to-A conversion without restoration of A, R, E_stored, or E_available toward replete references. Gate-8 classification: `DCDEV015_RESOURCE_CONVERSION_WITHOUT_HOMEOSTATIC_RESTORATION`.
- Changed areas: DC-DEV-015 assay registration, evidence, documentation, workflow, and governance records; no chemistry-core source or certified Phase-1 equations changed.
- Validation:
  - Sanctioned local Rust 1.89.0 example check - PASSED
  - DC-DEV-015 exact assay - PASSED with the required negative classification
  - Observer parity between instrumented and non-instrumented feeding trajectories - PASSED
  - Resource delivery/world-loss conservation - PASSED
  - Precursor ingress - PASSED
  - N/F-to-A conversion - PASSED (`0.01843919491375493` matched conversion fraction)
  - Activated-material restoration - FAILED as the diagnostic result
  - Precursor-inclusive restoration - FAILED as the diagnostic result
  - Material destination reconciliation - residuals reported, not hidden
  - Exact-head remote CI run 32011192247 - PASSED
  - Architect review - PENDING
- Remaining risks: independent architect interpretation of the destination residuals and rate-limiting classification.
- Blockers: architect review.
- Follow-up directive: none

## D-20260817-dcdev016-metabolic-break-even - PARTIAL

- Outcome ID: OUT-DCDEV016-METABOLIC-BREAK-EVEN-LOCAL
- Supersedes outcome: none
- Closed: `2026-08-17T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only DC-DEV-016 challenge reproduced the DC-DEV-015 settlement/deprivation and current-resource reference, then executed exactly one derived inventory of `14.588954880632265` N and F. Matched delivery reached `11.401893960861464` against the preregistered `11.387290380605897` target; E_available increased, but A, R, and E_stored did not restore from deprivation. Gate-8 classification: `DCDEV016_CONVERSION_STORAGE_BOTTLENECK_CONFIRMED`.
- Changed areas: DC-DEV-016 assay registration, evidence, documentation, workflow, and governance records; no chemistry-core source or certified Phase-1 equations changed.
- Validation:
  - Sanctioned local Rust 1.89.0 example check - PASSED
  - DC-DEV-016 exact assay - PASSED with the required classification
  - DC-DEV-015 settled/deprived/current/no-delivery parity - PASSED
  - N/F resource-world conservation for current, derived, and uptake-only arms - PASSED
  - Derived matched-delivery target - PASSED
  - E_available break-even - PASSED
  - A/R/E_stored strict restoration - FAILED as the diagnostic result
  - Legacy scalar destination reconciliation - not used; status recorded as `ACCOUNTING_CONTRACT_NOT_CLOSED`
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: exact-head remote validation and independent architect interpretation of the conversion/storage bottleneck classification.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260817-dcdev019r1-continuous-state-requalification - PARTIAL

- Outcome ID: `OUT-DCDEV019R1-GATE1-CONTINUOUS-REFEED`
- Supersedes outcome: none
- Closed: `2026-08-17T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact candidate requalification passed Gate 0 and reproduced the accepted settled body, legacy deprived body, Phase-1 selected-mass result, original DC-DEV-019 M2 result, and original M3 result. Continuous deprivation preserved the legacy material hash while carrying `h` from `0.0` to `0.12874703683936634`. The unchanged carried-state finite refeed ended at `E_stored=56.54155735320212`, below the prefeed `60.82781514212436`; Gate 1 therefore failed and all later phases were stopped.
- Changed areas: observer-only continuous-state requalification assay, compact evidence, documentation, and governance; no production homeostat or scientific parameter changed.
- Validation:
  - Exact entry and frozen production blob - PASSED
  - Historical control reproduction - PASSED
  - Continuous deprivation material parity - PASSED
  - Carried-state finite refeed - FAILED as preregistered diagnostic
  - Reset-state control reproduction - PASSED
  - Source-saturated sufficiency control - PASSED
  - Phases 2–6 - NOT RUN after Gate 1 stop
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: the unchanged homeostat remains unqualified for the current continuous finite-persistence contract; no tuning or alternate controller is authorized.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260817-dcdev019-nutrient-homeostasis-persistence - PARTIAL

- Outcome ID: `OUT-DCDEV019-PHASE1-DELIVERY`
- Supersedes outcome: none
- Closed: `2026-08-17T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Phase 1 reconstructed the production finite-resource boundary and passed Gates 0–2. Under the required current geometry, D1 ended at `E_stored=58.43844935623427` and D2's well-mixed upper bound ended at `57.819981156419516`; deterministic Phase 1B/1C selected `M_selected=19.878372106390554`, where passive delivery ended at `61.68434818478833`. `G_transport_max=1` was therefore retained. The accepted R1 value was separately reproduced only under its recorded reference geometry.
- Changed areas: observer-only counterfactual helpers, Phase 1 assay registration, compact Phase 1 evidence, documentation, and governance; no default production trajectory change.
- Validation:
  - Exact entry and protocol parity - PASSED
  - Chemistry gain-1 observer parity - PASSED
  - Resource uptake gain-1 observer parity - PASSED
  - Current-geometry D1/D2 delivery audit - PASSED as a Gate 2 diagnostic
  - Deterministic mass root and passive selected-mass restoration - PASSED
  - M0–M5 homeostasis qualification - PENDING
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: the one-state homeostat must remain feature-off compatible and pass the frozen M0–M5 qualification before any behavior work is authorized.
- Blockers: Gate 3, exact-head remote CI, and architect review.
- Follow-up directive: none

## D-20260817-dcdev019-nutrient-homeostasis-persistence - PARTIAL

- Outcome ID: `OUT-DCDEV019-GATE3-HOMEOSTASIS`
- Supersedes outcome: `OUT-DCDEV019-PHASE1-DELIVERY`
- Closed: `2026-08-17T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The default-off one-state homeostat passed M0 feature-off parity and M1 starvation demand accumulation, but M2 finite feeding did not restore stored material and M3 sustained precursor homeostasis remained within the target band while failing the Q4 slope and Q4 mean-h bounds. Gate 3 classification: `DCDEV019_COORDINATED_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED`.
- Changed areas: post-Phase-1 default-off homeostat API, observer-only qualification example, compact evidence, documentation, and governance; no behavior/exploration implementation was started.
- Validation:
  - Homeostat unit tests - PASSED (`4/4`)
  - M0 feature-off exact parity - PASSED
  - M1 starvation source-zero/no-material-creation checks - PASSED
  - M2 finite-feeding restoration - FAILED as preregistered diagnostic
  - M3 sustained homeostasis - FAILED as preregistered diagnostic
  - M4/M5 - NOT RUN after M3 stop condition
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: this is a negative result for the frozen one-state architecture; no controller tuning or behavior work is authorized without a new directive.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none
