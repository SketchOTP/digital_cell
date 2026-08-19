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

## D-20260817-dcdev020-fast-allosteric-assimilation - PARTIAL

- Outcome ID: `OUT-DCDEV020-A-PRODUCT-FEEDBACK-OBSERVER-NEGATIVE`
- Supersedes outcome: none
- Closed: `2026-08-17T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: One fixed target-free local A-product feedback law was evaluated in observer/counterfactual mode from the clean DC-DEV-016 authority. Existing passive delivery and finite-resource conservation passed. The feedback arm improved final stored material relative to the matched baseline (`55.30767262894278` versus `54.3584702923158`) but remained below the deprived pre-feed state (`60.82781514212436`), so the observer gate failed. The 8,000-step no-resource continuation also lost viability.
- Changed areas: DC-DEV-020 observer example registration, compact evidence, documentation, and governance; no production chemistry, resource boundary, mechanics, homeostat, or certified Phase-1 equations changed.
- Validation:
  - Exact clean DC-DEV-016 entry - PASSED
  - Selected finite ecology delivery - PASSED (`15.566438806699026` N/F delivered in feedback arm)
  - Resource conservation - PASSED (maximum error `0`)
  - Historical D-017 observer replay - PASSED as comparison-only provenance
  - A-product feedback finite-feed restoration - FAILED as the diagnostic result
  - 8,000-step no-resource bounded continuation - FAILED viability
  - Production integration - NOT RUN by fail-closed rule
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: independent architect interpretation of the chosen single-law negative result and whether a materially different metabolic-control topology is authorized.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260818-dcdev020r2-allosteric-requalification - PARTIAL

- Outcome ID: `OUT-DCDEV020R2-A-ONLY-COORDINATE-INSUFFICIENT`
- Supersedes outcome: `OUT-DCDEV020-A-PRODUCT-FEEDBACK-OBSERVER-NEGATIVE`
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R2 requalification audited the prior protocol, reconstructed the finite-source actuation envelope, replayed the correct August DC-DEV-017 observer, and audited post-activation A decay. The source-saturated upper bound ended at `61.6843481847883` from deprived `60.82781514212436`, but the constant-gain break-even root was `13.9482421875`; across the sampled envelope, A and required saturation gain decreased together as internal N/F accumulated, opposite the permitted A-only inhibitory family, so that family was not sufficient or identifiable. Classification: `DCDEV020_A_ONLY_ALLOSTERIC_COORDINATE_INSUFFICIENT`.
- Changed areas: new R2 observer example registration, new `dcdev020r2` evidence namespace, R2 documentation, and governance records; no production chemistry or certified Phase-1 equations changed.
- Validation:
  - Exact clean entry and prior head recorded - PASSED
  - Source-actuation envelope - PASSED
  - Post-activation A-decay sequencing audit - PASSED
  - Correct August DC-DEV-017 observer replay - PASSED
  - A-only coordinate sufficiency - FAILED as the preregistered negative result
  - Derived law, finite-feed qualification, sustained 8,000-step assay, and three-cycle assay - NOT RUN by fail-closed rule
  - Sanctioned local Rust 1.89.0 check/run - PASSED
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: exact-head remote validation and independent architect interpretation of whether a materially different allosteric topology is authorized; no new topology is authorized by this result.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260819-dcdev020r8r4-shared-affinity-autogenous-cprod - PARTIAL

- Outcome ID: `OUT-DCDEV020R8R4-SHARED-AFFINITY-ACCEPTED-NEGATIVE`
- Supersedes outcome: none
- Closed: `2026-08-19T00:00:00-04:00`
- Acceptance: `MET`
- Summary: Architect review accepted R8-R4 at exact head `37b47ec89e02418018a138f670e826c6945c8030` with exact-head CI `32254177853` passed. The tested shared-affinity topology is closed as a negative sustained-homeostasis route, while the broader catalyst-allocation capacity question remains open.
- Changed areas: governance acceptance record only; R8-R4 scientific evidence remains unchanged.
- Validation:
  - Exact R8-R4 head and closed draft PR #41 - PASSED
  - Exact-head CI run 32254177853 - PASSED
  - Architect acceptance - PASSED
- Remaining risks: The result does not reject all favorable A/C allocations and does not authorize another catalyst law or production integration.
- Blockers: R8-R5 capacity classification and architect review; DC-DEV-021 remains unauthorized.
- Follow-up directive: none

## D-20260819-dcdev020r8r5-ac-allocation-upper-bound - PARTIAL

- Outcome ID: `OUT-DCDEV020R8R5-CATALYST-ALLOCATION-ENVELOPE-MIXED`
- Supersedes outcome: none
- Closed: `2026-08-19T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R8-R5 audit reproduced the accepted R8-R2 acute and R8-R4 finite/sustained endpoints, evaluated the complete conservative constant-C interval with deterministic refinement, and found no constant allocation satisfying the original sustained homeostasis gate. The best final E_AR was approximately `57.63054549392781`. All 200 R8-R3 deferred and 200 R8-R4 shared-affinity late states retained at least one nonnegative one-step allocation region, with worst observed maximum drift approximately `0.00619040719167074`; classification is `DCDEV020R8R5_CATALYST_ALLOCATION_ENVELOPE_MIXED`.
- Changed areas: R8-R5 observer example, Cargo registration, compact append-only evidence, governed external dense-ledger manifest, documentation, governance, and scoped workflow; no production chemistry or behavior changed.
- Validation:
  - Exact R8-R2 and R8-R4 reproduction - PASSED
  - A+C conservation during repartition - PASSED
  - Exact 1:1 catalyst-turnover replacement accounting - PASSED
  - 65-point allocation mesh and deterministic refinement to relative width <= 1e-6 - PASSED
  - Original sustained qualification across constant-C arms - EXECUTED; zero complete-gate passes
  - Late-state local allocation envelopes at 40-step spacing - PASSED; 200 deferred and 200 shared states audited
  - External dense-ledger SHA-256 `afa9c26f8845f9321450ec12e7e4fe55dc54a088eb6857ff8e1e272dddc8c390` - PASSED locally and on Atlas
  - Governance, preservation, and exact-head remote CI run `32258092477` - PASSED
  - Architect review - PENDING
- Remaining risks: Mixed local drift means catalyst allocation capacity is not closed as insufficient; no allocator or next catalyst law is authorized by this result.
- Blockers: final evidence seal, exact-head remote CI, and architect review; DC-DEV-021 remains unauthorized.
- Follow-up directive: none

## D-20260819-dcdev020r8r5r1-net-allocation-drift - PARTIAL

- Outcome ID: `OUT-DCDEV020R8R5R1-RECYCLING-ONLY-LOCAL-CAPACITY`
- Supersedes outcome: `OUT-DCDEV020R8R5-CATALYST-ALLOCATION-ENVELOPE-MIXED`
- Closed: `2026-08-19T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R8-R5 local capacity was requalified from the actual incoming A/R state using `ΔE_NET = ΔE_repartition + ΔE_reaction`. All 200 deferred and 200 shared states retained nonnegative reversible NET drift, while all 200 deferred and all 200 shared forward-only envelopes were negative. Every successful reversible optimum required C→A recovery. Classification: `DCDEV020R8R5R1_RECYCLING_ONLY_LOCAL_CAPACITY`.
- Changed areas: R8-R5 observer-only net-drift implementation and wrapper, compact evidence, governed external dense-ledger manifest, documentation, governance, and scoped workflow; no production chemistry or behavior changed.
- Validation:
  - Exact R8-R5 replay machinery produced 200 deferred and 200 shared checkpoint states - PASSED
  - A+C conservation and unchanged controls - PASSED
  - Incoming-state net-drift decomposition - PASSED
  - Reversible and forward-only 65-point envelopes with deterministic refinement - PASSED
  - Worst-case aggregation uses minimum, with focused regression - PASSED
  - External dense-ledger SHA-256 `bdedb478eb3c32025fc62ecbc2af538b0e594d3182ee97ebeb7c68c2bb9d8efc` - PASSED locally and on Atlas
  - Governance, preservation, and exact-head remote CI run `32270631382` - PASSED
  - Architect review - PENDING
- Remaining risks: The positive local authority depends on privileged C→A recovery and does not establish forward A→C allocation capacity. The prior R8-R5 dense ledger lacks a deferred checkpoint-hash field, so R1 records deterministic replay hashes and reports direct prior-ledger hash-field matching as unavailable.
- Blockers: architect review; dynamic allocation, catalyst recycling, and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r3-two-substrate-saturating-activation - PARTIAL

- Outcome ID: `OUT-DCDEV020R3-SATURATING-KINETICS-NOT-IDENTIFIABLE`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R2 is architect-accepted as `DCDEV020R2_ACCEPTED_NEGATIVE`, with `A_ONLY_ALLOSTERIC_ASSIMILATION_CLOSED`. The R3 observer found material bilinear low-substrate suppression, but the symmetric N=F trajectory constrained only V_max/K_S^2. Deterministic asymptotic witnesses improved continuously as both parameters grew, so no unique finite V_max and K_S pair was identifiable. Classification: `DCDEV020R3_SATURATING_KINETICS_NOT_IDENTIFIABLE`.
- Changed areas: new R3 observer example registration, append-only `dcdev020r3` evidence, R3 documentation, workflow, and governance; no production chemistry, behavior, or certified Phase-1 equations changed.
- Validation:
  - Accepted R2 and clean scientific-base authority - PASSED
  - Required per-step source ledger - PASSED
  - Gate 2 bilinear low-substrate attribution - PASSED
  - Gate 4 finite V_max/K_S identification - FAILED as the required negative result
  - Candidate, finite-feed, dose, sustained, cycle, and production gates - NOT RUN by fail-closed rule
  - Sanctioned local Rust 1.89.0 check/run - PASSED
  - Exact-head remote CI `32183126542` at implementation/evidence head `90421f9f867e16ff369f9ecf7f7fe384b66d6857` - PASSED
  - Architect review - PENDING
- Remaining risks: exact-head remote validation and independent architect interpretation; only one symmetric substrate trajectory was authorized, so the two parameters remain structurally confounded.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260818-dcdev020r4-asymmetric-two-substrate-identifiability - PARTIAL

- Outcome ID: `OUT-DCDEV020R4-SATURATING-FAMILY-STRUCTURAL-MISMATCH`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R3 is architect-accepted as `DCDEV020R3_ACCEPTED_NEGATIVE`. All five R4 asymmetric probes were finite, conservative, source-comparable, and admitted deterministic break-even roots. The reciprocal design reached rank 3 with condition number `25.682977705016`, so the R3 collinearity was broken. The P0-P2 reciprocal fit nevertheless produced alpha `0.558251618117446`, beta `-0.629755501300906`, gamma `20.9305816462364`, and family-consistency relative error `0.966058373333954`. Positive symmetric two-substrate saturation therefore cannot represent the observed required-source surface. Classification: `DCDEV020R4_SATURATING_FAMILY_STRUCTURAL_MISMATCH`.
- Changed areas: new R4 observer example registration, append-only `dcdev020r4` evidence, R4 documentation, workflow, and governance; no production chemistry, behavior, or certified Phase-1 equations changed.
- Validation:
  - Accepted R3 and clean scientific-base authority - PASSED
  - R1/R2/R3 evidence immutability - PASSED locally
  - P0-P4 conservation, paired-substrate ingress, finite break-even roots, and source-saturation comparability - PASSED
  - Reciprocal design rank and conditioning - PASSED
  - Positive reciprocal coefficients and alpha*gamma approximately beta^2 - FAILED as the required negative result
  - Finite V_max/K_S, P3/P4 finite-model holdout, boundary witnesses, qualification, and production integration - NOT RUN by fail-closed rule
  - Sanctioned local Rust 1.89.0 check/run - PASSED
  - Exact-head remote CI run `32195362719` at head `bfbfece349ec4c637b15c432388b4ddff6ab689d` - PASSED
  - Architect review - PENDING
- Remaining risks: exact-head remote validation and independent architect interpretation; the permitted symmetric family is structurally mismatched after independent-axis excitation, but no broader kinetic family is authorized.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260818-dcdev020r4-asymmetric-two-substrate-identifiability - COMPLETE

- Outcome ID: `OUT-DCDEV020R4-ARCHITECT-ACCEPTED-NEGATIVE`
- Supersedes outcome: `OUT-DCDEV020R4-SATURATING-FAMILY-STRUCTURAL-MISMATCH`
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `MET`
- Summary: Architect review accepted the R4 negative result at exact head `669a511aacb227240bd7a4698efecfb564f481d4`; PR #33 closed unmerged and exact-head CI `32171718751` passed. The tested saturation family failed against the endpoint-derived constant-gain surrogate surface; R4 did not establish a unique instantaneous source-demand surface.
- Changed areas: governance acceptance record only; R4 scientific evidence remains unchanged.
- Validation:
  - Exact R4 head and closed-unmerged PR #33 - PASSED
  - Exact-head CI run 32171718751 - PASSED
  - Architect negative-result review - PASSED
- Remaining risks: the endpoint-derived constant-gain target required local physiological requalification before selecting another activation topology.
- Blockers: production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: D-20260818-dcdev020r5-local-zero-drift-source-audit

## D-20260818-dcdev020r5-local-zero-drift-source-audit - PARTIAL

- Outcome ID: `OUT-DCDEV020R5-NF-LOCAL-COORDINATE-SUFFICIENT`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer replayed all ten R4 baseline/constant trajectories exactly and audited 4,800 post-uptake pre-reaction states. Every state had a finite physical zero-drift root; none was capacity-insufficient or non-monotonic before its first root. The endpoint constant-gain trajectories were not local requirements. A fixed P0-P2-trained, P3-P4-held-out diagnostic supported N/F as sufficient on the frozen audited manifold. Classification: `DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT`; independent surrogate classification: `ENDPOINT_SURROGATE_NOT_LOCAL_REQUIREMENT`.
- Changed areas: R5 observer example registration, compact append-only evidence, external dense-ledger manifest, R5 documentation, scoped workflow, and governance; no production chemistry or behavior changed.
- Validation:
  - Accepted R4 authority and R1-R4 evidence immutability - PASSED locally
  - Historical D-043, D-045, and D-067 guard - PASSED
  - Ten R4 trajectory hashes - PASSED
  - 4,800 bounded source-response and zero-drift audits - PASSED
  - Maximum root relative interval `9.53674316472667e-7` - PASSED
  - Maximum stored-material accounting residual `2.2849994466001e-14` - PASSED
  - External ledger SHA-256 `4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585` - PASSED locally and on Atlas
  - Sanctioned Rust 1.89.0 check/run - PASSED
  - Local full preservation - PASSED
  - Exact-head remote CI run `32207702692` at head `4f9e637f5d0dd97ed13df9266d18624538107588` - PASSED
  - Architect review - PENDING
- Remaining risks: N/F sufficiency is diagnostic and bounded to the frozen P0-P4 one-step state manifold; it does not select or qualify a production law, restoration controller, or durable metabolism.
- Blockers: architect review; production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r8r3-catalyst-reserve-horizon - PARTIAL

- Outcome ID: `OUT-DCDEV020R8R3-CATALYST-RESERVE-SOURCE-CONTEXT-DEPENDENT`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R8-R3 audit reproduced the accepted R8-R2 acute endpoints, derived the frozen catalyst half-life as `3465.7359027997263` accepted steps, and executed the fixed 8000-step sustained horizon under D016 bilinear and sealed R6 NF power-law source contexts. Deferred catalyst production retained frozen turnover and both deferred arms remained alive, finite, and conservation-closed. D016 had no marginal payback at any deterministic checkpoint; R6 paid back at `433, 480, 866, 1733, 3466` but not at `5199` or `6931`. Classification: `DCDEV020R8R3_CATALYST_RESERVE_SOURCE_CONTEXT_DEPENDENT`.
- Changed areas: R8-R3 observer example registration, compact append-only evidence, governed external dense-ledger manifest, documentation, governance, and scoped CI; no production chemistry or behavior changed.
- Validation:
  - Accepted R8-R2 authority and sealed R5/R7/R8/R8-R1/R8-R2 input hashes - PASSED
  - Accepted R8-R2 480-step normal and deferred endpoint reproduction - PASSED
  - Frozen `k_c_turn`, `dt`, half-life, deterministic checkpoints, and 8000-step horizon - PASSED
  - D016/R6 sustained NORMAL and DEFERRED trajectories - PASSED
  - D016/R6 marginal INVEST/SKIP checkpoint shadows - PASSED
  - Source-context comparison and fail-closed classification - PASSED
  - Conditional delayed-resume validation - NOT RUN; no overlapping source-context payback bracket
  - Local sanctioned Rust 1.89.0 compile and actual observer execution - PASSED
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: Payback is source-context dependent and no portable delayed-resume timing oracle was authorized by the result. The audit does not establish a production law, catalyst target, deficit signal, or DC-DEV-021 authorization.
- Blockers: exact-head remote CI and architect review; production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r5-local-zero-drift-source-audit - COMPLETE

- Outcome ID: `OUT-DCDEV020R5-ARCHITECT-ACCEPTED`
- Supersedes outcome: `OUT-DCDEV020R5-NF-LOCAL-COORDINATE-SUFFICIENT`
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `MET`
- Summary: Architect review accepted R5 at exact head `d215cfc00ce70517e25fa7c3b51b13d85d9ce521`; PR #34 closed unmerged and exact-head CI `32183498937` passed. Source capacity is sufficient on all 4,800 audited states, endpoint-derived source is not the local requirement, and N/F is information-sufficient on the tested one-step manifold without qualifying an NF controller.
- Changed areas: governance acceptance record only; R5 scientific evidence remains unchanged.
- Validation:
  - Exact R5 head and closed-unmerged PR #34 - PASSED
  - Exact-head CI run 32183498937 - PASSED
  - Architect positive diagnostic review - PASSED
- Remaining risks: N/F information sufficiency did not establish one explicit causal production law, finite-feed restoration, or durable homeostasis.
- Blockers: production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: D-20260818-dcdev020r6-nf-power-law-source

## D-20260818-dcdev020r6-nf-power-law-source - PARTIAL

- Outcome ID: `OUT-DCDEV020R6-FINITE-FEED-RESTORATION-FAILURE`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only symmetric generalized-mass-action power law passed deterministic P0-P2 identification, P3/P4 held-out local-root validation, and source-response sanity, then stopped at Gate 5. The R6 arm increased final E_stored to `60.0620310117838` versus baseline `54.3584702923158`, but remained below deprived `60.82781514212436`. Classification: `DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE`.
- Changed areas: R6 observer example registration, compact append-only evidence, R6 documentation, scoped workflow, and governance; no production chemistry or behavior changed.
- Validation:
  - Accepted R5 authority and governed dense-ledger SHA-256 - PASSED
  - Historical D-043/D-067/R4 architecture guard - PASSED
  - Closed-form fit `K_PL=0.017556661171593057`, `p=0.0003277429681759396` - PASSED
  - P3/P4 combined relative RMSE `0.05106673550084852` and p95 `0.12868568862094845` - PASSED
  - All-state capacity violations and clipping dependence - PASSED with zero
  - Selected finite-feed safety, physical bounds, and accounting - PASSED
  - Selected finite-feed stored-material restoration - FAILED as the authorized negative result
  - Balanced dose, sustained-fed, and three-cycle gates - NOT RUN by fail-closed rule
  - Three focused candidate tests - PASSED
  - Regulatory-core complete suite - PASSED (36 tests)
  - Phase-1 focused preservation - PASSED (4 tests)
  - D-088 focused preservation - PASSED (4 tests)
  - Evolution-harness preservation - PASSED (40 tests)
  - Governance ADOPTED validation - PASSED
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: The two-parameter law accurately approximates one-step local balance but does not restore stored material on the frozen finite-feed window; no broader kinetic or control architecture is authorized by this result.
- Blockers: architect review; production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r8-nfa-restorative-attractor - PARTIAL

- Outcome ID: `OUT-DCDEV020R8-PRODUCT-FEEDBACK-TOPOLOGY-INCOMPATIBLE`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R8 used the accepted R7 state, sealed R5/R7 dense ledgers, frozen N/F support, and deterministic matched pairs to test a reciprocal target-free product-feedback topology. The training set contained `2425` matched pairs; only `310` showed the required maintenance-demand decrease with A while `2115` showed the opposite sign. The reciprocal positive-feasibility region was empty, so execution stopped at Gate 3 with classification `DCDEV020R8_PRODUCT_FEEDBACK_TOPOLOGY_INCOMPATIBLE`.
- Changed areas: R8 observer example registration, compact append-only evidence, governed external dense pair/constraint ledger, R8 documentation, scoped workflow, and governance; no production chemistry or behavior changed.
- Validation:
  - Accepted R7 head and sealed R5/R7 dense inputs - PASSED locally
  - Deterministic matched-pair construction and reciprocal constraint audit - PASSED
  - Training Gate 3 feasibility - FAILED closed with the authorized negative classification
  - Holdout, P3/P4, R7, and capacity gates - NOT RUN by fail-closed rule
  - Zero-substrate control and production-scope guards - PASSED locally
  - External R8 dense-ledger SHA-256 `12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff` - PASSED locally and on Atlas
  - Focused R8 tests - PASSED locally
  - Exact-head remote CI run `32195362719` at head `bfbfece349ec4c637b15c432388b4ddff6ab689d` - PASSED
  - Architect review - PENDING
- Remaining risks: This closes only the tested reciprocal product-feedback topology on the frozen R5/R7 training support; it does not close the wider NFA route or authorize a production A-dependent law.
- Blockers: architect review; R9 and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r6-nf-power-law-source - COMPLETE

- Outcome ID: `OUT-DCDEV020R6-ARCHITECT-ACCEPTED-NEGATIVE`
- Supersedes outcome: `OUT-DCDEV020R6-FINITE-FEED-RESTORATION-FAILURE`
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `MET`
- Summary: Architect review accepted R6 at exact head `f01b716d9051c9f0114f3c5c0d1b123e2df037cf`; PR #35 closed unmerged and exact-head CI `32187547222` passed. Classification remains `DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE`. The exact generalized N/F power-law restoration route is closed, but NF information sufficiency is not closed.
- Changed areas: governance acceptance record only; R6 scientific evidence remains unchanged.
- Validation:
  - Exact R6 head and closed-unmerged PR #35 - PASSED
  - Exact-head CI run 32187547222 - PASSED
  - Architect negative-result review - PASSED
- Remaining risks: R6 one-step fit success did not identify whether its free-running failure was law-family error, induced-state ambiguity, or maintenance-only zero-drift behavior.
- Blockers: production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: D-20260818-dcdev020r7-on-policy-zero-drift-audit

## D-20260818-dcdev020r7-on-policy-zero-drift-audit - PARTIAL

- Outcome ID: `OUT-DCDEV020R7-NFA-COORDINATE-REQUIRED-ON-POLICY`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R7 reproduced the exact R6 finite-feed endpoint, solved 480/480 finite monotone physical zero-drift roots, and closed summed local drift exactly against the observed `-0.7657841303405846` endpoint change. R6 remained below every root. The unchanged NF observer passed RMSE and p95 but failed ambiguity at `0.26505161065124994`; unchanged NFA passed with RMSE `0.017325292104497104`, p95 `0.04444189891888537`, and ambiguity `0.010315383793568476`. The exact-root oracle maintained depleted stored material rather than restoring it. Classification: `DCDEV020R7_NFA_COORDINATE_REQUIRED_ON_POLICY`.
- Changed areas: R7 observer example registration, compact append-only evidence, external dense-ledger seal, R7 documentation, scoped workflow, and governance; no production chemistry or behavior changed.
- Validation:
  - Accepted R6 authority and R1-R6 evidence immutability - PASSED locally
  - Exact R6 endpoint and committed realization hash - PASSED locally
  - 480 bounded source-response and zero-drift audits - PASSED
  - Source capacity and monotonicity - PASSED with zero failures
  - Maximum root relative interval `9.53674316472667e-7` - PASSED
  - Maximum root accounting residual `1.660910992073994e-14` - PASSED
  - Summed local drift and endpoint closure - PASSED exactly
  - Frozen NF observer limits - FAILED on ambiguity only as the authorized diagnostic result
  - Frozen NFA observer limits - PASSED
  - Exact-root oracle accounting and maintenance-only result - PASSED
  - External ledger SHA-256 `abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8` - PASSED locally and on Atlas
  - Focused observer tests - PASSED (2 tests)
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: NFA is only information-sufficient under a frozen diagnostic; R7 does not identify or authorize an NFA production law. Exact zero drift is maintenance-like at the depleted state and does not supply restorative surplus.
- Blockers: exact-head remote CI and architect review; production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r8r1-causal-a-demand-elasticity - PARTIAL

- Outcome ID: `OUT-DCDEV020R8R1-A-DEMAND-ELASTICITY-POSITIVE`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R8-R1 audit held every reconstructed state fixed and perturbed only A. Across 2,880 training states, 960 P3 states, 960 P4 states, and 480 R7 on-policy states, all finite A perturbations produced positive physical zero-drift demand elasticity. Catalyst production was the dominant demand block by median magnitude. An independent 2,425-pair A-only swap audit confirmed R8 pair confounding, including 155 background-state sign reversals. Classification is `DCDEV020R8R1_A_DEMAND_ELASTICITY_POSITIVE` with pair verdict `R8_PAIR_CONFOUNDING_CONFIRMED`.
- Changed areas: observer-only R8-R1 example, compact append-only evidence, governed external dense demand ledger, documentation, scoped workflow, and governance; no production chemistry or behavior changed.
- Validation:
  - Finite A perturbation roots (10,560 roots; zero capacity failures; zero non-monotonicity failures) - PASSED
  - Accounting closure (maximum residual `2.6549075438087044e-14`, tolerance `1e-10`) - PASSED
  - R8 pair confounding audit (2,425 pairs; 155 background-state sign reversals) - PASSED
  - External dense-ledger SHA-256 `f44e8f9fa441451ee40bcbfccac5f556131e4d26868868607e9507c29e7bcf90` on local and Atlas copies - PASSED
  - Focused R8-R1 compile/execution and governance validation - PASSED
  - Exact-head remote CI run `32203590517` at head `bbf636626b0009e339d4250eb998123cb1f193fe` - PASSED
  - Architect review - PENDING
- Remaining risks: This establishes causal A-demand elasticity and explains R8 pair confounding, but authorizes no A-feedback law, production integration, or DC-DEV-021 work.
- Blockers: architect review; production changes and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260818-dcdev020r8r2-catalyst-investment-payback - PARTIAL

- Outcome ID: `OUT-DCDEV020R8R2-CATALYST-INVESTMENT-ACUTE-RECOVERY-BOTTLENECK`
- Supersedes outcome: none
- Closed: `2026-08-18T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R8-R2 audit reconstructed all 480 accepted R7 on-policy pre-reaction states and paired each with normal and one-step no-catalyst-production physical zero-drift shadows. All 480 pairs were valid, with zero capacity failures and zero pre-crossing non-monotonicity failures. Median catalyst-production burden was `0.008328836524032168` E_AR units and the burden was at least the R6 source shortfall on all 480 states. Exact sealed R6 normal replay ended at `60.06203101178377`, while whole-window catalyst-production deferral ended at `63.645566711951915` from a deprived start of `60.82781514212436`. All 24 checkpoint payback runs reported `NO_PAYBACK`. Classification: `DCDEV020R8R2_CATALYST_INVESTMENT_ACUTE_RECOVERY_BOTTLENECK`.
- Changed areas: R8-R2 observer example registration, compact append-only evidence, governed external dense-ledger manifest, documentation, scoped workflow, and governance; no production chemistry or behavior changed.
- Validation:
  - Accepted R8-R1 authority and sealed R5/R7/R8/R8-R1 input hashes - PASSED
  - Local sanctioned Rust 1.89.0 compile and focused example test - PASSED
  - 480 paired physical roots, zero capacity failures, zero non-monotonicity failures - PASSED
  - D016 bilinear and R6 power-law payback checkpoints - EXECUTED; all `NO_PAYBACK`
  - Exact normal R6 endpoint parity - PASSED
  - Whole-window cprod-deferred shadow alive/finite and above deprived start - PASSED
  - External dense-ledger SHA-256 `e932f6ab96e34516de98c97c2cae102553db9764383af3d61abf015743c3a376` - PASSED locally and on Atlas
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: The result diagnoses catalyst-production investment as an acute finite-feed recovery bottleneck; it does not establish a production repair, a different resource contract, or a sufficient alternative catalyst strategy. No implementation or DC-DEV-021 work is authorized.
- Blockers: architect review; production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: none
- D-20260819-dcdev020r8r4-shared-affinity-autogenous-cprod - PARTIAL
  - Outcome ID: `OUT-DCDEV020R8R4-SHARED-AFFINITY-NO-STABLE-HOMEOSTASIS`
  - Supersedes outcome: none
  - Closed: `2026-08-19T00:00:00-04:00`
  - Acceptance: `PARTIAL`
  - Summary: The observer-only shared-affinity law `J_C=k_c_prod*A*(1-q_c(C))`, reusing existing `K_C=q_c=0.3` and adding no parameter or state, reproduced the accepted R8-R2 acute authority and R8-R3 sustained authority. Under sealed R6, finite-feed restoration passed with final `E_AR=62.575632782724874`, all dose scales were monotonic (`62.571943751789135`, `62.575632782724874`, `62.57772981708882`), but the 8,000-step sustained gate failed at `E_AR=54.45821737181944` against target `77.91027880846893`; the conditional three-cycle assay was correctly not run. Classification: `DCDEV020R8R4_SHARED_AFFINITY_NO_STABLE_HOMEOSTASIS`.
  - Changed areas: observer example/helper, Cargo registration, compact evidence, external dense-ledger manifest, documentation, governance, and scoped workflow; no production chemistry or behavior changed.
  - Validation: local Rust 1.89.0 compile and execution passed; exact authority reproduction passed; finite-feed, dose, D016 preservation, accounting, and boundary checks passed; exact-head remote CI and architect review are pending.
  - Remaining risks: This closes only the tested shared-affinity topology as a sustained-homeostasis route. It does not reject all coupled source/allocation architectures and does not authorize production integration.
  - Blockers: exact-head remote CI and architect review; DC-DEV-021 remains unauthorized.
  - Follow-up directive: none
