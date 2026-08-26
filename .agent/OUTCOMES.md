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

## D-20260823-dcdev020m1r3-ordinary-decay-candidate001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R3-ORDINARY-DECAY-CANDIDATE-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-23T00:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The versioned `ConservativeV3` candidate differs from `ConservativeV2` only in starvation activated-material decay: V3 uses ordinary `k_a_decay=0.008` with no fourfold multiplier. Local E0–E4 qualification passes. Non-starved V2/V3 replay is exact; ordinary-decay equivalence matches observer collapse `3279` and rupture `124717`; source-capacity replay without `/4` produces organized delta `+1.2571804904075918`; both ordinary and source-capacity refeeding forks remain topologically ruptured; actual V3 and preserved V2 D-087 are `8/8`. Classification is `M1_ORDINARY_DECAY_CANDIDATE_QUALIFIED`, pending exact-head remote CI and Architect review.
- Changed areas: versioned mesh chemistry schema, certifier candidate selector, bounded R3 assay, compact evidence, documentation, scoped CI, and governance. `ConservativeV2` remains unchanged and selected; no production source law, D-091, D-087 criteria, mechanics, transport, reserve, recycling, salvage, M2, behavior, evolution, or DC-DEV-021 work changed.
- Validation:
  - `cargo +1.89.0 check -p regulatory-core --example dcdev020m1r3_ordinary_decay_candidate` - PASSED
  - Scoped Rust formatting - PASSED
  - Actual D-087 ConservativeV3/reserve OFF - PASSED (8/8)
  - Actual D-087 ConservativeV2/reserve OFF preservation - PASSED (8/8)
  - Fresh local R3 assay and compact artifact generation - PASSED
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: Exact-head remote CI must verify fresh V3/V2 reports, artifact identity, preservation regressions, and bounded diff. V3 is an experimental candidate only; sustained production homeostasis and source implementation remain unresolved and unauthorized.
- Blockers: exact-head remote CI and independent Architect acceptance.
- Follow-up directive: none

## D-20260823-dcdev020m1r4-coupled-source-candidate001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R4-COUPLED-SOURCE-CANDIDATE-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-23T00:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The bounded R4 implementation adds an opt-in `FINITE_SPATIAL_RESOURCE_COUPLED_ACTIVATION_V1` wrapper around the unchanged V1 finite-resource boundary. Only same-step newly delivered paired N/F is transformed into existing A+W; unmatched material remains N/F and pre-existing internal N/F is not selected. The local preregistered 480-step V1 baseline/reference/candidate matrix and controls qualify, actual D-087 V3/V2 are 8/8, and the preservation suite passes; exact-head remote CI and Architect review remain pending.
- Changed areas: versioned regulatory-core adapter, bounded R4 example, compact evidence, documentation, scoped CI, and governance. No V1 transport, V2/V3 chemistry, production selection, reserve, recycling, salvage, M2, behavior, evolution, or DC-DEV-021 work is authorized.
- Validation:
  - Coupled adapter focused tests - PASSED
  - Exact 480-step assay - PASSED
  - Actual D-087 V3/V2 - PASSED
  - Preservation suite - PASSED
  - Exact-head remote CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: This is an experimental candidate qualification, not a production source-law change. Exact remote CI and independent Architect interpretation remain required.
- Blockers: final local/remote verification and Architect review.
- Follow-up directive: none

## D-20260822-dcdev020m1r2r2-topology-death-closure001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R2R2-TOPOLOGY-DEATH-CLOSURE-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-22T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The M1-R2-R1 observer-collapse classification was retired after review because both arms remained physically intact and became observer-viable after refeeding. M1-R2-R2 replays the exact accepted step-20480 endpoint, continues the unchanged reaction path until actual edge rupture, and tests the exact ruptured state under ordinary finite refeeding and the existing source-capacity upper-bound shadow. Both arms rupture at steps 124249 and 124717, remain `closed_intact=false` after both 5000-step branches, and preserve strict material closure. Local classification: `M1_TOPOLOGY_DEATH_ESTABLISHED`.
- Changed areas: New observer example, example registration, compact evidence, scoped workflow, documentation, and governance only.
- Validation:
  - Scoped Rust 1.89.0 formatting - PASSED
  - Regulatory-core compile and fresh M1-R2-R2 replay - PASSED
  - Actual edge rupture in both arms - PASSED
  - Ordinary finite and source-capacity refeed closure - PASSED
  - Fresh actual D-087 8/8 - PASSED in exact-head CI run `32615105226`
  - Phase-1, D-091, D-088, and evolution-harness preservation - PASSED in exact-head CI run `32615105226`
  - Exact-head remote CI run `32615105226` at head `d5160ab` - PASSED; artifact digest `sha256:8c714086484bab1eae58d60e126152c79120d21ba33468076ac14e0b246077fd`
  - Architect review - PENDING
- Remaining risks: The classification is bounded to irreversible topology loss under this frozen chemistry path and does not authorize production repair, source-law changes, recycling/salvage, M2, behavior, evolution, or DC-DEV-021.
- Blockers: independent architect review.
- Follow-up directive: none

## D-20260822-dcdev020m1r2r1-physical-failure-closure001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R2R1-PHYSICAL-FAILURE-CLOSURE-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-22T22:20:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only continuation starts at accepted M1-R2 head `bc65098c3d26777aca2d1da5dab8cc118ecc6e19` and reproduces both accepted R2 trajectory hashes and endpoint values before continuing. The production 4x and ordinary-decay arms both reach the existing `activated_catalyst_collapse` chemistry-path terminal boundary within the 150,000-step extension, at steps `45422` and `45831` respectively, without edge rupture. Both receive the exact 5,000-step finite N/F restoration without state reset; both deliver the requested resources and close material accounting, but organized material remains below the failed-state baseline despite closed-intact topology and returned observer viability. Local classification: `M1_ORDINARY_DECAY_IRREVERSIBLE_FAILURE_ESTABLISHED`.
- Changed areas: observer-only M1-R2-R1 example, compact evidence, scoped workflow, documentation, and governance. No chemistry-core, Phase-1 source, production chemistry, resource law, death rule, D-091, D-087, recycling, salvage, M2, behavior, evolution, or DC-DEV-021 work changed.
- Validation:
  - Exact R2 endpoint reproduction with committed trajectory hashes - PASSED
  - Extended production and ordinary-decay continuation - PASSED; both reached existing terminal boundary
  - Failure-margin instrumentation - PASSED; no edge rupture before catalytic terminal failure
  - Exact no-reset restoration challenge - PASSED; 5,000 steps per terminal arm, no coherent recovery
  - Internal and restoration material closure - PASSED
  - Scoped Rust format and regulatory-core example compile/run - PASSED
  - Fresh actual D-087 8/8 - PASSED locally and in exact-head CI
  - Exact-head remote CI runs `32612860278` and `32612861856` at head `11fed776b7e694fb9af5debc4c2914c0a23ba615` - PASSED
  - Final governance-head CI runs `32613245695` (push) and `32613247896` (PR synchronize) at head `38d18083b66f1daa9914fac3a2b724017cc33155` - PASSED
  - Architect review - PENDING
- Remaining risks: This is a chemistry-path material-failure result, not full-runtime organism death; mechanics/remesh/rebond are outside the assay. Independent architect review remains required before acceptance. No M1 production change, M2, reserve redesign, recycling, salvage, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: independent architect review.
- Follow-up directive: none

## D-20260822-dcdev020m1r1r1-decay-confound001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R1R1-DECAY-CONFOUND-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-22T18:20:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only causal isolation reproduces the original M1-R1 BASE, source-capacity, catalyst-off, and combined ledgers. The raw source-capacity arm selects the frozen existing 4x starvation multiplier on all `480/480` steps. Changing only the diagnostic A-decay coefficient to `K/4` neutralizes that multiplier while retaining the ordinary effective coefficient and yields organized-material changes `+1.25718049040759` for source capacity and `+1.2755639121915` for the combined shadow. The bounded classification is `M1_SOURCE_CAPACITY_SUFFICIENT_AFTER_DECAY_NEUTRALIZATION`.
- Changed areas: M1-R1 observer reuse seam, decay-confound observer example, compact evidence, documentation, scoped workflow, and governance. No chemistry-core production source or Phase-1 biology changed.
- Validation:
  - Scoped Rust formatting - PASSED
  - `cargo +1.89.0 check -p regulatory-core --example dcdev020m1r1r1_decay_confound` - PASSED
  - Fresh four-arm decay-confound assay - PASSED
  - Exact M1-R1 reproduction - PASSED
  - Raw source starvation branch provenance `480/480` - PASSED
  - Decay neutralization with only `k_a_decay` changed - PASSED
  - World↔organism and internal closure - PASSED
  - Local artifact verifier - PASSED
  - Exact-head remote CI at `b11e9815d317fe09ae227d13eefc5d89a463fe51` - PASSED; PR synchronize run `32611416322` and push run `32611416748`, all stages passed
  - Architect review - PENDING
- Remaining risks: The neutralized nonnegative result is an acute 480-step capacity shadow, not sustained M1 homeostasis or production authorization. Exact-head CI and independent architect verification remain required. M1 production change, M2, reserve redesign, recycling/salvage, behavior, evolution, and DC-DEV-021 remain unauthorized.
- Blockers: independent architect review.
- Follow-up directive: none

## D-20260822-dcdev020m1r1capacitydecomp001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R1-CAPACITY-DECOMPOSITION-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-22T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The exact accepted M1-R0 high-inventory deprived state was replayed through the accepted M1-R0 settlement/deprivation path. Four matched observer-only shadows were executed: exact BASE, conservative paired N/F→A source-capacity upper bound, catalyst-investment deferral, and the combined bound. The source bound uses the existing ConservativeV2 `N + F -> A + W` coefficients and all world↔organism and internal material closures pass. The baseline reproduces the accepted Arm C values. The bounded classification is `M1_SOURCE_AND_ALLOCATION_INSUFFICIENT`; neither acute source capacity nor catalyst-investment deferral, alone or combined, establishes nonnegative organized-material change over 480 steps.
- Changed areas: M1-R1 observer example, example registration, compact evidence, documentation, scoped CI, and governance only. No production chemistry, ConservativeV2, D-091, uptake, transport, resource quantity, degradation, recycling, salvage, M2, or DC-DEV-021 work changed.
- Validation:
  - Sanctioned Rust 1.89.0 compile/check and scoped rustfmt - PASSED
  - Exact M1-R0 Arm C baseline reproduction - PASSED locally
  - Conservative source stoichiometry and four-arm material closure - PASSED locally
  - Fresh local artifact generation - PASSED
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: The negative decomposition does not establish whether conversion throughput, productive allocation, or deeper maintenance/degradation is the remaining causal blocker. No production repair is authorized.
- Blockers: exact-head remote CI and independent architect review.
- Follow-up directive: none

## D-20260822-dcdev020r9r6-mobilize-first-store-last - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R6-MOBILIZE-FIRST-STORE-LAST`
- Supersedes outcome: none
- Closed: `2026-08-22T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The final observer-only D-091 phase-order topology was run from `f1704acff5ca64e509a28c74af8cccbf76439ef2` for exactly 5,000 accepted steps. V20 reproduced 8/8 and FULL reproduced `R_m=0.839869520280528`. `MOBILIZE_FIRST_STORE_LAST` reused frozen release/loss before unchanged productive chemistry and frozen storage afterward, reaching `R_m=0.839973528362306` with actual D-087 gates `[true,false,false,false,false,true,true,true]`. Replete A→R was `147.5982725689982`; post-starvation R→A was `26.15666583047419`; reserve rejects were zero and strict closure passed. Classification: `DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_CONTRIBUTORY_NOT_SUFFICIENT`.
- Changed areas: additive observer phase-order ledger fields, one explicit observer mode, phase1-certifier test/runner, compact evidence, documentation, governance, and scoped workflow. Production chemistry, production reserve physiology, parameters, thresholds, recycling, and DC-DEV-021 were not changed.
- Validation:
  - Sanctioned Rust 1.89.0 compile/check - PASSED
  - Focused R9-R6 ordering regression - PASSED
  - Local exact 5,000-step FULL/shadow audit and actual D-087 replay - COMPLETED
  - V20 control - 8/8
  - Shadow actual D-087 - 3/8; Gates 0, 5, 6, and 7 pass
  - Reserve function - replete A→R and post-starvation R→A positive; zero rejects; strict closure - PASSED
  - Exact-head remote CI and architect review - PENDING
- Remaining risks: The topology is contributory but does not restore D-087 certification. R9-R6 is the final observer-only D-091 topology audit; return to the Architect for a production D-091 decision. No R9-R7, reserve repair, recycling/salvage, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260822-dcdev020r9r6r1-evidence-closure - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R6R1-EVIDENCE-CLOSURE`
- Supersedes outcome: none
- Closed: `2026-08-22T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R9-R6 exact-head CI run `32553371490` completed the scientific runner and failed only at `Verify R9-R6 phase-order artifacts` because the verifier asserted stale key `shadow_semantics.target_signal`; the protocol schema uses `controller_or_target: false`. The authoritative shadow D-087 vector is `[true,false,false,false,false,true,true,true]`, correctly counted as 4/8. Full `R_m=0.839869520280528`, shadow `R_m=0.839973528362306`, `R_b` and `R_C` remain identity-matched, replete A→R is `147.5982725689982`, post-starvation R→A is `26.15666583047419`, reserve rejects are zero, and strict closure passes. The preregistered certification boundary is not restored; classification is corrected to `DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_INSUFFICIENT`.
- Changed areas: closure-only artifact verifier schema assertion, derived pass-count fields, compact evidence classification/count, documentation, and governance. Scientific values, scientific configuration, production chemistry, and production reserve physiology were not changed.
- Validation:
  - Local JSON artifact identity and vector/count reconciliation - PASSED
  - Local governance and formatting checks - PENDING
  - Exact-head remote DC-DEV-020-R9 CI - PENDING
  - Architect review - PENDING
- Remaining risks: The corrected classification and closure evidence require exact-head remote CI and independent architect review. No reserve repair, recycling/salvage, R9-R7, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: exact-head remote CI and architect review.
- Follow-up directive: none

## D-20260822-dcdev020r9r6r2-ci-closure - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R6R2-CI-CLOSURE`
- Supersedes outcome: none
- Closed: `2026-08-22T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R9-R6-R1 remains CONTINUE / NOT ACCEPTED solely because exact-head runs `32566149213` and `32566146792` at head `a2c21c1664a7f1d44be0edc8b6819665b0455a82` were cancelled at the explicit 30-minute workflow ceiling during artifact upload after substantive R9-R6 generation, artifact verification, and identity checks passed. The R9-R6 scientific classification remains `DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_INSUFFICIENT` with D-087 4/8 and unchanged evidence. R9-R6-R2 authorizes only changing the workflow timeout to 45 minutes.
- Changed areas: `.github/workflows/dc-dev-020r9.yml` timeout only, plus governance handoff records. No scientific or production files changed.
- Validation:
  - Timeout provenance - CONFIRMED
  - Workflow-only diff isolation - PASSED
  - R9-R6 evidence identity - PASSED
  - Local governance validation - PENDING
  - Exact-head remote workflow - PENDING
  - Architect review - PENDING
- Remaining risks: E5 remains open until the new exact-head workflow completes SUCCESS. No R9-R7, reserve repair, recycling/salvage, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: exact-head workflow SUCCESS and architect acceptance.
- Follow-up directive: none

## D-20260821-dcdev020r9r5r1-valid-liquidity-counterfactual - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R5R1-VALID-LIQUIDITY-COUNTERFACTUAL`
- Supersedes outcome: none
- Closed: `2026-08-21T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R9-R5 is recorded as `REPLAN / NOT ACCEPTED`; the broad `DCDEV020R9R5_RESERVE_DEFECT_OUTSIDE_CHARGE_LIQUIDITY_FACTORIZATION` classification is retired as authority. The R9-R5 Gate-7 failure was reproduced as a runner-root defect. R9-R5-R1 uses a diagnostic-only `LIQUID_RESERVE_PRETHROTTLE_UB` shadow that evaluates frozen M/L demand with A+R, funds baseline demand from A, and funds only incremental demand from R. Local V20 is 8/8. The 5,000-step treatment uses `10.986875147245845` diagnostic R (`9.741776616086431` for M and `1.2450985311594456` for L), improves `R_m` from `0.8398695202805284` to `0.9994257946133822`, but remains D-087 Gate-1 negative. Classification: `DCDEV020R9R5R1_RESERVE_LIQUIDITY_CONTRIBUTORY_NOT_SUFFICIENT`.
- Changed areas: additive observer-only M/L availability ledger, pre-throttle diagnostic mode, R9-R5 runner-root resolution, focused phase1-certifier test, compact R9-R5-R1 evidence, documentation, governance, and scoped workflow. Production D-091 reserve physiology and certified Phase-1 biology were not changed.
- Validation:
  - Focused pre-throttle liquidity regression - PASSED
  - Phase1-certifier sim tests - `4 passed, 0 failed`
  - Local 5,000-step R9-R5-R1 audit - COMPLETED; strict closure true; reserve rejects zero
  - Local V20 packaged-runtime control - `8/8`
  - Exact-head remote CI and architect review - PENDING
- Remaining risks: The result remains provisional until exact-head remote CI verifies the R9-R5-R1 generated artifact and the architect independently reviews the causal interpretation. No reserve repair, recycling/salvage, production integration, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: exact-head remote CI and independent architect review.
- Follow-up directive: none

## D-20260820-dcdev020r9r4-reserve-interference-audit - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R4-STORAGE-CAUSAL-PRIORITY-INSUFFICIENT`
- Supersedes outcome: none
- Closed: `2026-08-20T00:00:00-04:00`
- Acceptance: `MET`
- Summary: From exact head `f9bc1d5bffe828b2599c85d4fcbbabdf7f3e3ff3`, V20 reproduced 8/8 and V21 retained the reserve-bearing negative. Five exact 5,000-step ConservativeV2 arms recorded per-step reserve/build/membrane ledgers. Full reserve stored A before later productive demand (`147.585809275616`); the parameter-free maintenance-priority shadow reduced that interference to zero while preserving the unchanged reserve kernels, but replacement metrics remained below Gate-1 qualification (`R_m=0.8399735283623063`). Gate 4 was correctly skipped. Classification: `DCDEV020R9R4_STORAGE_CAUSAL_PRIORITY_INSUFFICIENT`.
- Changed areas: additive observer ledger fields, explicit reserve diagnostic controls, phase1-certifier R9-R4 runner, compact evidence, documentation, governance, and scoped workflow. Certified Phase-1 biology/equations, production behavior, reserve parameters, recycling, and DC-DEV-021 were not changed.
- Validation:
  - phase1-certifier compilation - PASSED
  - exact R9-R4 release runner - PASSED
  - V20 reproduction 8/8 and V21 reserve-bearing control 4/8 - PASSED
  - all five arms completed 5,000/5,000 steps - PASSED
  - reserve rejects zero and strict reserve closure - PASSED
  - Gate-1 full and maintenance-priority shadow qualification - FAILED
  - Gate 4 execution by fail-closed rule - NOT APPLICABLE
  - unrelated full chemistry failure in `d008_tests.rs` - NOT APPLICABLE
  - Architect acceptance of R9-R4 at exact reviewed head - PASSED
- Remaining risks: The diagnostic establishes that pre-maintenance storage is causally present but insufficient as a standalone parameter-free priority repair. It does not authorize production reserve repair, recycling, salvage, tuning, source/sink changes, behavior, or DC-DEV-021.
- Blockers: none for R9-R4; R9-R5 is the authorized next diagnostic and DC-DEV-021 remains unauthorized.
- Follow-up directive: D-20260821-dcdev020r9r5-charge-liquidity-audit

## D-20260821-dcdev020r9r4r1-governance-remote-closure - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R4R1-GOVERNANCE-REMOTE-CLOSURE`
- Supersedes outcome: none
- Closed: `2026-08-21T04:00:00-04:00`
- Acceptance: `MET`
- Summary: The malformed R9-R4 adopted governance records were corrected without changing scientific source or evidence. Exact-head workflow `32459018571` passed at `19bd21f25d8cf955fb1fe58d32aa4d6d74c5cf21`; the unchanged R9-R4 classification remains `DCDEV020R9R4_STORAGE_CAUSAL_PRIORITY_INSUFFICIENT`.
- Changed areas: governance-format corrections in DIRECTIVES.md, OUTCOMES.md, and REPO_MAP.md only.
- Validation:
  - adopted governance validator - PASSED
  - exact-head workflow `32459018571` - PASSED
  - R9-R4 reserve interference audit - PASSED
  - R9-R4 observer artifact verification and compact evidence upload - PASSED
  - scientific source and evidence identity preservation - PASSED
  - Architect acceptance at exact head `08e1c45b11892e0b5533b11c74f175ee84d243ed` with exact-head workflow `32460044729` - PASSED
- Remaining risks: R9-R4 evidence remains sealed and no reserve repair, recycling, salvage, tuning, behavior, or DC-DEV-021 work is authorized.
- Blockers: none for R9-R4-R1; R9-R5 review remains open.
- Follow-up directive: D-20260821-dcdev020r9r5-charge-liquidity-audit

## D-20260821-dcdev020r9r5-charge-liquidity-audit - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R5-CHARGE-LIQUIDITY-DECOMPOSITION`
- Supersedes outcome: `OUT-DCDEV020R9R4R1-GOVERNANCE-REMOTE-CLOSURE`
- Closed: `2026-08-21T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The exact 5,000-step observer audit separates standing-stock charging from direct reserve liquidity. FULL reproduces sealed `R_m=0.8398695202805284`; STORE_OFF reproduces `R_m=1.0180981834599838`. Surplus-only storage remains capped and non-binding (`R_m=0.8399798978913839`), while the liquid upper bound uses zero direct diagnostic R in the frozen trajectory. Neither independent arm nor their combined counterfactual restores the actual D-087 Gate-1 qualification. The provisional fail-closed classification is `DCDEV020R9R5_RESERVE_DEFECT_OUTSIDE_CHARGE_LIQUIDITY_FACTORIZATION`.
- Changed areas: additive observer-only reserve ledger fields and diagnostic modes, phase1-certifier R9-R5 runner/tests, compact evidence, documentation, governance, and scoped workflow. No production reserve law, parameter, target/controller/state, certified Phase-1 biology, recycling, salvage, behavior, evolution, or DC-DEV-021 work changed.
- Validation:
  - sanctioned Rust 1.89.0 compile and R9-R5 example - PASSED
  - exact 5,000-step FULL, STORE_OFF, surplus-only, liquid upper-bound, and combined arms - PASSED
  - positive replete A→R and starvation R→A, zero rejects, strict closure - PASSED
  - actual D-087 shadow execution - completed; Gates 1–4 not restored
  - local Windows packaged-runtime Gate 7 - environment-limited; remote Linux CI required
  - exact-head remote CI and architect review - PENDING
- Remaining risks: The negative identifies no restoration from the bounded charge/liquidity factorization, but does not authorize production repair or any new kinetic family. Dense ledgers remain local/external and compact evidence is provenance-bound.
- Blockers: exact-head remote CI and independent architect review.
- Follow-up directive: none

## D-20260819-dcdev020r9-mesh-contract-requalification - PARTIAL

- Outcome ID: `OUT-DCDEV020R9-METRIC-CONFOUNDING-DOMINANT`
- Supersedes outcome: none
- Closed: `2026-08-19T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The R9 historical mesh audit confirms `NO_POSITIVE_CONSERVATION_VECTOR`; the versioned conservative v2 audit confirms a strictly positive all-material vector. Runtime v2 reactions close the strict material ledger to approximately `2.84e-14`. A physically ruptured v2 mesh remains governed by observer viability while transport continues. Conservative fission partition accounting passes. The compact E5 contract replay separates strict material, activation, and organized-retained ledgers and classifies `DCDEV020R9_METRIC_CONFOUNDING_DOMINANT`.
- Changed areas: Versioned mesh stoichiometry and v2 runtime guards only; compact R9 analysis/example, evidence, documentation, and governance. Historical D-012/D-086/D-087/D-088 and D-015 through R8-R5-R1 evidence remain preserved.
- Validation:
  - chemistry-core mesh-contract tests - PASSED (5 tests)
  - D-086 tests - PASSED (9 tests)
  - D-088 tests - PASSED (4 tests)
  - phase1-certifier tests - PASSED (4 tests)
  - R9 example/evidence generation - PASSED
  - exact-head remote CI run `32290370285` at head `4c4f88995a4f7e224b6b211580039ade81ad8c9e` - PASSED
  - architect review - PENDING
- Remaining risks: E5 rows are bounded v2 contract replays and do not overwrite the sealed historical D-015/D-016/R8 protocol artifacts. Architect acceptance remains outstanding. No salvage, controller, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: architect review.
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

## D-20260819-dcdev020r9r1-mesh-contract-requalification - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R1-MESH-CONTRACT-ORTHOGONALIZATION`
- Supersedes outcome: `OUT-DCDEV020R9-METRIC-CONFOUNDING-DOMINANT`
- Closed: `2026-08-19T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: ConservativeV2 is an orthogonal serialized mesh contract, so D-091 retains `autopoietic_material_mesh_metabolic_reserve_v1` equation identity while observer-only death and strict material semantics are selected by contract version. D-091 reserve schema loading and 200-step A/R execution pass with zero rejected steps. The reserve-bearing D-087 Gates 0–7 matrix passes locally. Exact D-015/D-016 replay rows close strict material against delivered N/F with zero residual; the exact R8-R2 and R8-R4 machinery also runs under ConservativeV2 with historical defaults preserved. R8-R4 remains a bounded negative (`DCDEV020R8R4_SHARED_AFFINITY_NO_STABLE_HOMEOSTASIS`).
- Changed areas: versioned mesh contract field, reserve schema compatibility, D-091 composition test, reserve-bearing D-087 gate matrix, exact replay runners/manifests, scoped workflow, documentation, and governance; no new biology, controller, transport law, source law, sink law, behavior, evolution, or DC-DEV-021 work.
- Validation:
  - Local sanctioned Rust 1.89.0 compile and focused contract/D-091 tests - PASSED
  - D-087 ConservativeV2 Gates 0–7 matrix - PASSED locally
  - Exact D-015/D-016 replay, 7 rows, zero closure residual, zero reserve rejects - PASSED locally
  - Exact R8-R2 replay under ConservativeV2 compatibility mode - PASSED locally
  - Exact R8-R4 replay under ConservativeV2 compatibility mode - PASSED locally; Gates 0-4 and 7 pass, Gates 5-6 fail with the sealed negative classification
  - Scoped exact-head remote CI run `32313240060` at head `885a1cbe5b713b17a3eb2090938b3b7890c91fcc` - PASSED; all scoped stages including Exact R8-R4 replay and artifact verification passed
  - Architect review - PENDING
- Remaining risks: R9-R1 remains unaccepted until independent architect review verifies the pushed branch. Historical R9 evidence remains immutable; generic R9 E5 rows are retained as proxy diagnostics and are not represented as exact historical replay evidence. The new R8-R4 replay is compatibility evidence only and does not alter or reopen the accepted R8-R4 result.
- Blockers: independent architect review; production integration and DC-DEV-021 remain unauthorized.
- Follow-up directive: none

## D-20260819-dcdev020r9r2-material-fate-requalification - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R2-CONSERVATIVE-CERTIFICATION-REGRESSION`
- Supersedes outcome: `OUT-DCDEV020R9R1-MESH-CONTRACT-ORTHOGONALIZATION`
- Closed: `2026-08-19T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The exact ConservativeV2 D-015/D-016 replay and the R9-R2 material-fate ledger close numerically, but the actual D-087 Phase-1 certifier under ConservativeV2+D-091 reserve passes only 3/8 gates. The direct local invocation has gates 0, 5, and 6 passing; the exact-head PR artifact has gates 5, 6, and 7 passing and reports a source/artifact integrity conclusion. Both are fail-closed as `DCDEV020R9R2_CONSERVATIVE_CERTIFICATION_REGRESSION`.
- Changed areas: observer-only direct A-decay accounting, actual ConservativeV2+D-091 certifier launcher, exact replay/fate runner, compact evidence, documentation, governance, and scoped CI. No certified Phase-1 equation, production behavior, recycling law, source/sink law, or DC-DEV-021 work changed.
- Validation:
  - Local sanctioned Rust 1.89.0 compile for the certifier launcher and R9-R2 example - PASSED
  - Actual D-087 Gates 0–7 under ConservativeV2+D-091 - COMPLETED 3/8; fail-closed regression
  - Exact D-015/D-016 replay - 7 rows, zero reserve rejects, zero closure residual - PASSED
  - D-016 finite fate closure - PASSED; closure `4.263256414560601e-14`, organized reconciliation `3.552713678800501e-14`
  - Four 8,000-step sustained arms - PASSED as evidence generation; all final-quarter organized slopes negative
  - Scoped exact-head remote CI run `32317704754` at head `b6633d99d0f8baa7faae6d569215ec8d7ff9c8cd` - PASSED (21/21 steps; compact artifact SHA-256 `c5de085cffceff448a7dbe20f5f8280a973700ecbf800da6b11ac19cc09e65ea`)
  - Architect review - PENDING
- Remaining risks: The ConservativeV2+D-091 certifier regression is not localized to a new biology claim by this directive. No tuning, gate weakening, or production integration is authorized. Existing R9-R1 and prior evidence remain preserved.
- Blockers: exact-head remote CI and independent architect review; DC-DEV-021 remains unauthorized.
- Follow-up directive: none

## D-20260820-dcdev020r9r3-conservation-reserve-decomposition - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R3-RESERVE-PHYSIOLOGY-CERTIFICATION-GAP`
- Supersedes outcome: `OUT-DCDEV020R9R2-CONSERVATIVE-CERTIFICATION-REGRESSION`
- Closed: `2026-08-20T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The actual D-087 certifier was run in the required H0/V20/H1/V21 matrix. H0 HistoricalV1 plus reserve OFF reproduced the historical scientific result through Gates 0–6 (`R_m=1.0180981834599838`, `R_b=5.818353471059928`, `R_C=1.446090001246529`). V20 ConservativeV2 plus reserve OFF also passed Gates 0–6. Both reserve-enabled arms failed Gates 1–4 while recording nonzero A→R, R→A, and R→W flows with zero reserve rejects. Classification: `DCDEV020R9R3_RESERVE_PHYSIOLOGY_CERTIFICATION_GAP_CONFIRMED`. Local Windows packaging did not qualify Gate 7; exact remote Linux CI remains authoritative.
- Changed areas: orthogonal phase1-certifier contract/reserve selectors, actual four-arm certifier runner, reserve execution ledger, compact evidence, scoped workflow, documentation, and governance. Certified Phase-1 equations, production chemistry, production behavior, recycling, and DC-DEV-021 were not changed.
- Validation:
  - Sanctioned Rust 1.89.0 compile/check for the R9-R3 runner - PASSED
  - Smoke run stopped at the required H0 hard stop - PASSED as non-authoritative smoke behavior
  - Full local H0/V20/H1/V21 actual-certifier matrix - COMPLETED
  - H0 and V20 scientific Gates 0–6 - PASSED
  - H1 and V21 reserve execution - nonzero flows, zero rejects; scientific Gates 1–4 failed as expected for the diagnostic
  - R9-R2 compact material-fate preservation predicate - PASSED
  - Exact-head remote CI run `32421756950` at head `6a266514fcb616084ea43be42ff726c4c51dec0e` - PASSED; compact R9-R3 artifact SHA-256 `951fb0f5bc79ab70dc2d50d614c3ca43520069eb8a73360817f01951b2ecfbdf`
  - Architect review - PENDING
- Remaining risks: This local result diagnoses a reserve-physiology certification gap but does not distinguish which reserve sub-behavior requires repair. No reserve tuning, recycling, source/sink change, production integration, or DC-DEV-021 work is authorized.
- Blockers: exact-head remote CI and independent architect review; DC-DEV-021 remains unauthorized.
- Follow-up directive: none

## D-20260820-dcdev020r9r3r1-packaged-runtime-closure - PARTIAL

- Outcome ID: `OUT-DCDEV020R9R3R1-GATE7-PACKAGED-RUNTIME-CLOSURE`
- Supersedes outcome: `OUT-DCDEV020R9R3-RESERVE-PHYSIOLOGY-CERTIFICATION-GAP`
- Closed: `2026-08-20T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R9-R3's only unresolved positive-control defect was Gate 7 packaged-binary execution. Exact local reproduction showed Cargo built successfully, but the runtime used extensionless binary paths while Windows emitted `digital-protocell-phase1.exe`; the source was therefore not found, no package copy was made, and `bin_ok` was false. The bounded repair derives the platform executable suffix and records build/copy/launch diagnostics. Fresh local H0 and V20 replays are 8/8 with unchanged `R_m=1.0180981834599838`, `R_b=5.818353471059928`, and `R_C=1.446090001246529`; H1 and V21 retain Gates 1–4 failure, identical reserve flows, zero rejects, and Gate 7 pass. The R9-R3 classification remains `DCDEV020R9R3_RESERVE_PHYSIOLOGY_CERTIFICATION_GAP_CONFIRMED`.
- Changed areas: Gate-7 runtime packaging diagnostics and platform path handling, scoped compact-evidence upload, fresh R9-R3-R1 evidence, and governance. No certified Phase-1 equation, production chemistry, production behavior, reserve parameter, threshold, recycling law, or DC-DEV-021 work changed.
- Validation:
  - `cargo +1.89.0 check -p phase1-certifier --bin digital-protocell-phase1 --bin phase1_certification` - PASSED
  - Gate-7 executable-path unit test - PASSED
  - Fresh H0 actual D-087 certifier - 8/8; packaged binary exit `0`, output and snapshot present - PASSED
  - Fresh H0/V20/H1/V21 matrix - COMPLETED; H0/V20 8/8, H1/V21 Gates 1–4 failed with preserved reserve signature
  - Exact-head remote CI run `32436117572` at head `7c6b35c5b67a798c4ff32a61c3f6cf8e4fa8b5e5` - PASSED; compact artifact `dcdev020r3r1-compact-evidence` digest `sha256:b13107fa4ed5a77531cae816754435debae65199ce82e868e208e83e82f6ba86`
  - Architect review - PENDING
- Remaining risks: Independent architect review remains. No reserve tuning, recycling, source/sink change, production integration, or DC-DEV-021 work is authorized.
- Blockers: independent architect review.
- Follow-up directive: none
## D-20260822-dcdev020m0baseline001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M0-BASELINE-QUALIFICATION-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020R9R6R2-CI-CLOSURE`
- Closed: `2026-08-22T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The production-selection audit identified `phase1-certifier/src/sim.rs` as the selector boundary. The bounded default change selects ConservativeV2 while requiring explicit opt-in for D-091 reserve physiology; the R9 diagnostic workflow now declares reserve-enabled arms explicitly. Fresh local ordinary production execution reports ConservativeV2/reserve OFF, actual D-087 Gates 0–7 pass `8/8`, and the 5,000-step packaged runtime reports zero A→R, R→A, and R→W flows, zero reserve rejects, and activation-equivalent closure residual `2.580691216280684e-10`. D-091 source and certified chemistry remain unchanged. Remote Linux runtime identity and exact-head CI are pending architect review.
- Changed areas: phase-1 certifier selector defaults, explicit R9 diagnostic environment declarations, compact M0 evidence, M0 documentation, scoped M0 CI, and governance handoff. No chemistry-core source or D-091 implementation changed.
- Validation:
  - Governance ADOPTED validation - PASSED
  - Sanctioned Rust 1.89.0 phase1 metrics regression - PASSED (4/4)
  - Phase1 certifier selector tests - PASSED (5/5)
  - Fresh ordinary selected-production actual D-087 - PASSED (8/8)
  - Fresh local packaged runtime - PASSED; alive after 5,000 steps; zero reserve flows; strict closure
  - Exact-head M0 workflow run `32591546718` at head `0b2db2ddc8e02f72b748a26455de608106a7a9de` - PASSED; fresh D-087, packaged Linux runtime, artifact verification, phase-1 regression, and D-091 preservation all passed
  - D-091 source preservation - PASSED in exact-head M0 CI
  - Packaged Linux runtime identity - PASSED in exact-head M0 CI
  - Architect review - PENDING
- Remaining risks: Final acceptance depends on exact-head remote Linux execution and independent architect verification. M1, reserve redesign, recycling/salvage, R9-R7, and DC-DEV-021 remain unauthorized.
- Blockers: exact-head M0 CI and architect review.
- Follow-up directive: none

## D-20260822-dcdev020m1r0requal001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R0-FINITE-RESOURCE-REQUALIFICATION-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M0-BASELINE-QUALIFICATION-PENDING-ARCHITECT`
- Closed: `2026-08-22T17:15:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only M1-R0 replay runs from accepted M0 head `4895135deee7dbd782446dbfe25662181951afe0` using `ConservativeV2` with reserve disabled and zero initial reserve. The exact 5,000-step settlement and 480-step deprivation complete; organized material declines by `18.5240742455985`. The 3/3 and high-inventory finite N/F arms consume `2.34938455157938` and `11.4396891103526` per substrate respectively, produce `0.0433163220514441` and `0.945737637075` A, and both retain negative organized-material change. The uptake-only control consumes `11.5964868177898` per substrate without conversion. All four world↔organism closure residuals are below `2.4e-13`. The no-resource continuation reaches observer `starvation_collapse` at accepted step `2057`. Local classification is `productive_allocation_or_replacement_limitation` and remains diagnostic.
- Changed areas: M1-R0 observer example registration, compact evidence, documentation, scoped CI, and governance only. No chemistry-core, phase1-certifier production source, D-091, uptake, transport, resource, death, behavior, or DC-DEV-021 work changed.
- Validation:
  - Governance ADOPTED validation via `py -3 scripts/validate_governance.py --mode ADOPTED` - PASSED
  - New observer example formatting - PASSED
  - Fresh local M1-R0 replay and deterministic committed/fresh qualification match - PASSED
  - Phase-1 metrics regression - PASSED
  - D-091 preservation regression - PASSED
  - D-088 preservation regression - PASSED
  - Evolution-harness regression - PASSED (40 tests)
  - Exact-head remote M1-R0 CI - PENDING
  - Architect review - PENDING
- Remaining risks: This is a requalification result, not a production repair. The current finite-resource mass/conversion/allocation limitation is not isolated to a permitted implementation change. No M1 production change, M2, reserve redesign, recycling, salvage, behavior, evolution, or DC-DEV-021 work is authorized.
- Blockers: exact-head remote CI and independent architect review.
- Follow-up directive: none

## D-20260822-dcdev020m1r2-starvation-law-audit001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R2-STARVATION-LAW-AUDIT-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R1R1-DECAY-CONFOUND-PENDING-ARCHITECT`
- Closed: `2026-08-22T18:35:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only audit starts from accepted M1-R1-R1 head `7bb48874771144795a9559f7570f5ebc77e1004a`. Repository provenance traces the fourfold starvation A-decay branch to D-086 commit `20e9f7814020ca38ed1893fdd94fb3264307de2e`; the source comment provides no explicit quantitative rationale, and D-087 requires starvation/death behavior but does not explicitly require fourfold A destruction. The production 4x arm and ordinary-decay `k_a_decay=0.002` arm both complete the 480-step comparison and the full 20,000-step continuation. Both lose observer viability through the existing reversible `starvation_collapse` predicate, but neither reaches mesh rupture, catalytic/structural physical failure, or invalid runtime geometry within the authorized bound. Local classification is `M1_STARVATION_LAW_AUDIT_INCONCLUSIVE`. Fresh actual D-087 independently returns ConservativeV2/reserve OFF with 8/8 gates and `D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED`.
- Changed areas: New observer example, compact generated evidence, scoped workflow, documentation, and governance only. No chemistry-core, phase1-certifier production source, ConservativeV2, D-087 implementation, resource, death, reserve, recycling, M2, behavior, evolution, or DC-DEV-021 work changed.
- Validation:
  - Rust 1.89.0 scoped formatting - PASSED
  - Regulatory-core observer compile - PASSED
  - Fresh local 480-step / 20,000-step audit - PASSED; both arms reached the bound without physical failure
  - Fresh actual D-087 certifier - PASSED (8/8, ConservativeV2, reserve OFF)
  - Local compact base/fresh artifact verification - PASSED
  - Phase-1, D-091, D-088, and evolution-harness preservation - PASSED; evolution-harness 40 tests
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: No physical failure occurred within the authorized 20,000-step continuation, so the necessity question remains bounded and inconclusive. This result does not authorize removing or changing the production multiplier, source-law implementation, M1 production repair, M2, reserve redesign, recycling/salvage, behavior, evolution, or DC-DEV-021.
- Blockers: exact-head remote CI and independent architect review.
- Follow-up directive: none

## D-20260823-dcdev020m1r5-sustained-homeostasis001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R5-SUSTAINED-HOMEOSTASIS-PENDING-ARCHITECT`
- Supersedes outcome: none
- Closed: `2026-08-23T01:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R5 executes the accepted R4 depleted entry through 8,000-step coupled, uncoupled, no-resource, and feed-then-remove arms under the preregistered finite backing reservoir. Local accounting and R4 checkpoint reproduction pass. The local Windows D-087 runtime remains 7/8 because Gate 7 is Linux-packaging dependent; exact-head remote Linux CI and Architect review remain authoritative.
- Changed areas: World-side fixed-concentration finite backing-reservoir wrapper, bounded R5 assay, compact evidence, shared-drive evidence manifest, scoped CI, documentation, and governance. No certified Phase-1 equations, V1 transport, R4 coupling law, production selection, reserve, recycling, salvage, M2, behavior, evolution, or DC-DEV-021 work changed.
- Validation:
  - Sanctioned Rust 1.89.0 formatting/check and focused backing/coupled tests - PASSED
  - Fresh local 8,000-step R5 assay, R4 reproduction, and closure - PASSED
  - Local V3/V2 D-087 Gate 7 packaging - BLOCKED
  - Exact-head remote Linux CI and artifact verification - NOT RUN
- Remaining risks: The R5 classification is provisional until exact-head Linux CI verifies fresh D-087, artifact equality, preservation, and the sustained gates. Authoritative R5 evidence is stored at `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r5`; compact CI-required artifacts remain in `digital-protocell/experiments/generated/dcdev020m1r5/` and are duplicated there. No production selection or downstream directive is authorized.
- Blockers: exact-head remote CI and independent Architect review.
- Follow-up directive: none

## D-20260823-dcdev020m1r6-full-runtime-integration-cert001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6-FULL-RUNTIME-INTEGRATION-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R5-SUSTAINED-HOMEOSTASIS-PENDING-ARCHITECT`
- Closed: `2026-08-23T19:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: R6 executes the exact R5 depleted entry through the packaged order `transport -> ConservativeV3 reactions -> mechanics -> remesh -> try_local_rebond`. The fed full-runtime arm remains intact and observer-viable but declines by `101.13283694705021` organized material. No-resource and feed-removal controls decline. Full-runtime strict material closure does not close (`125.27019370631` fed residual), and the bounded 150,000-step no-resource continuation reaches its bound without actual topology rupture; M1 remains unclosed.
- Changed areas: R6 phase1-certifier example registration, compact evidence, scoped CI, documentation, shared-drive dense evidence, and governance only. No chemistry-core equations, mechanics/remesh/rebond laws, V1 transport, D-091, D-087 criteria, production default, recycling, salvage, M2, behavior, evolution, or DC-DEV-021 work changed.
- Validation:
  - Sanctioned Rust 1.89.0 compile and formatting - PASSED
  - Fresh V3 D-087 - PASSED 8/8
  - Fresh V2 D-087 - PASSED 8/8
  - Full-runtime local arms and dense shared-drive write - COMPLETE
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: R6 is not a production qualification. The large closure residual and absence of topology rupture within the authorized bound require Architect disposition; no mechanics or chemistry repair is authorized in this directive. R5 remains reaction-subsystem evidence, V3 remains unselected, and M2/DC-DEV-021 remain blocked.
- Blockers: exact-head remote CI and independent Architect review.
- Follow-up directive: none

## D-20260823-dcdev020m1r6r1-geometry-material-closure-audit001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R1-GEOMETRY-MATERIAL-CLOSURE-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6-FULL-RUNTIME-INTEGRATION-PENDING-ARCHITECT`
- Closed: `2026-08-23T20:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R6-R1 replay preserves the stored R6 trajectory and final mesh. Independent S0-S5 ledgers reconstruct the full 8,000-step residual. Mechanics/remesh geometry residuals sum to `-125.27019370631099`, matching the fixed-concentration area counterfactual `-125.27019370631108`; unexplained residual is `1.9895196601282805e-13`. Mechanics-only and an existing lawful remesh-only fixture both show strict material loss without uptake or reactions. The first permanent resource-contact loss is step `612`, following geometry change at step `1`. Classification is `M1_RUNTIME_GEOMETRY_MASS_COUPLING_CONFIRMED`.
- Changed areas: Observer-only R6-R1 example registration, compact evidence, scoped CI, documentation, and governance. No chemistry-core, mechanics, remesh, rebond, transport, chemistry, resource, death, selector, or downstream behavior changed.
- Validation:
  - Sanctioned Rust 1.89.0 formatting and compile - PASSED
  - Fresh 8,000-step observer replay with plain/instrumented trajectory parity - PASSED
  - Mechanics-only and remesh-only isolation - PASSED; rebond isolation NOT EXERCISED because no lawful fixture was available
  - Dense stage ledger written to the canonical shared drive - COMPLETE
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: This is causal attribution, not a repair or biological qualification. The geometry/concentration hypothesis is supported by numerical closure, but no concentration rescaling or mechanics/remesh correction is authorized until independent Architect review.
- Blockers: exact-head remote CI and independent Architect review.
- Follow-up directive: none

## D-20260823-dcdev020m1r6r1r1-cross-platform-closure-validation001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R1R1-CROSS-PLATFORM-CLOSURE-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R1-GEOMETRY-MATERIAL-CLOSURE-PENDING-ARCHITECT`
- Closed: `2026-08-23T22:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The R6-R1 verifier was narrowed so scientific protocol identity excludes execution paths, while the failed R6-R1 evidence remains preserved. The audit now records exact within-platform plain/instrumented parity, semantic checkpoints through step 8000, cumulative delivery, remesh-event counts, and explicit resource-contact chronology. Local validation passes with the Atlas canonical dense ledger; remote Linux validation and Architect acceptance remain pending.
- Changed areas: Cross-platform audit representation, execution metadata, scoped workflow verifier, immutable `dcdev020m1r6r1r1` evidence namespace, documentation, and governance.
- Validation:
  - Sanctioned Rust 1.89.0 compile and local replay - PASSED
  - Scientific protocol excludes execution-path fields - PASSED locally
  - Local within-platform observer trajectory/final-mesh parity - PASSED
  - Local mechanics/remesh isolation and fixed-concentration attribution - PASSED
  - Local contact chronology: exposure `2` to `0` at step `611` with resource remaining `464.709236493918` - PASSED
  - Original failed R6-R1 evidence namespace preserved - PASSED
  - Exact-head remote CI - PENDING
  - Architect review - PENDING
- Remaining risks: Linux may show numeric or discrete remesh divergence; acceptance depends on causal invariants rather than bitwise long-horizon trajectory equality. No mass-conservation repair is authorized.
- Blockers: exact-head remote CI and independent Architect review.
- Follow-up directive: none

## D-20260823-dcdev020m1r6r1r1r1-remote-verifier-order-closure001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R1R1R1-REMOTE-VERIFIER-ORDER-CLOSURE-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R1R1-CROSS-PLATFORM-CLOSURE-PENDING-ARCHITECT`
- Closed: `2026-08-24T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The exact workflow-order repair is present at `a3205d6c99cbc845d406043580f99577fa6a73e6`. Remote CI run `32686612525` succeeded in `3m25s`; the fresh audit, D-087 producers, cross-platform verifier, remaining preservation stages, and artifact upload completed. The workflow blocker is closed, while independent Architect acceptance remains pending.
- Changed areas: Workflow dependency ordering and governance/status records only; no audit, verifier, production, chemistry, mechanics, remesh, transport, resource, or preservation source changes.
- Validation:
  - Original missing-report blocker - REPAIRED
  - Scientific verifier logic and thresholds changed - NO
  - Exact-head remote CI `32686612525` - PASSED
  - Artifact `dcdev020m1r6r1-geometry-material-closure-evidence` - PRESENT
  - Artifact digest `sha256:d7f91883db1155308d576f2c6f09d2eb2c92bf60dcd617493275ade036f7d181` - VERIFIED BY GITHUB
  - Architect review - PENDING
- Remaining risks: Remote completion establishes workflow and preservation execution, not Architect scientific acceptance or authorization for geometry/material repair. M1 remains not established and M2/DC-DEV-021 remain unauthorized.
- Blockers: independent Architect review.
- Follow-up directive: none

## D-20260824-dcdev020m1r6r2-geometry-material-conservation-repair001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R2-GEOMETRY-MATERIAL-CONSERVATION-PRESERVATION-REGRESSION-REVIEW`
- Supersedes outcome: `OUT-DCDEV020M1R6R1R1R1-REMOTE-VERIFIER-ORDER-CLOSURE-PENDING-ARCHITECT`
- Closed: `2026-08-24T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: A versioned `MeshContractVersion::GeometryConservativeV3` was implemented without changing ConservativeV2. The common mechanics path and authoritative remesh path rescale the 18 area-based interior concentration fields using `M_after = C_after * A_after = C_before * A_before`; exterior chemistry, edge/free membrane amounts, templates, autocatalytic edge amounts, and D-096 expressed catalyst amounts are untouched. Local mechanics-only and remesh-only candidate strict deltas are `0`, and the integrated 8,000-step candidate closure residual is `4.263256414560601e-14`. The required actual D-087 candidate run under the new contract returns `6/8`, failing Gate 1 and Gate 2, while unchanged ConservativeV3 under ConservativeV2 returns `8/8`. Per directive, execution stopped before remote CI, push, or any tuning.
- Changed areas: New versioned material contract, mechanics/remesh conservation primitive integration, phase1 candidate contract selector, focused chemistry-core tests, R6-R2 observer/evidence harness, field-semantics evidence, and governance only. No chemistry equations, mechanics force law, remesh decision law, transport, resource, D-087 criteria, production selection, or downstream behavior changed.
- Validation:
  - Focused geometry/material conservation test `d098_geometry_material_conservation` (4/4) - PASSED
  - R6-R2 local 8,000-step candidate replay and canonical Atlas compact/dense evidence (conservation closure) - PASSED
  - Candidate mechanics-only strict delta (`0`) - PASSED
  - Candidate remesh-only strict delta (`0`) - PASSED
  - Candidate integrated world↔organism closure (`4.263256414560601e-14`) - PASSED
  - Candidate actual D-087 (6/8; Gate 1 and Gate 2) - FAILED
  - Unchanged ConservativeV3 under ConservativeV2 actual D-087 (8/8) - PASSED
  - Remote CI (hard stop on candidate D-087 regression) - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: The conservation law removes the measured geometry/material artifact locally, but the unchanged D-087 acceptance criteria are not preserved under the new candidate contract. No criteria relaxation, initialization change, parameter tuning, biology repair, production switch, M1 recertification, M2, recycling/salvage, or DC-DEV-021 work is authorized.
- Blockers: Candidate D-087 preservation regression; Architect disposition required.
- Follow-up directive: none

## D-20260824-dcdev020m1r6r2r1-d087-semantics-regression-audit001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R2R1-D087-SEMANTICS-MIXED-REGRESSION-REVIEW`
- Supersedes outcome: `OUT-DCDEV020M1R6R2-GEOMETRY-MATERIAL-CONSERVATION-PRESERVATION-REGRESSION-REVIEW`
- Closed: `2026-08-24T01:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The exact accepted R6-R2 control, ConservativeV3 chemistry under the ConservativeV2 material contract, reproduces D-087 `8/8`. The GeometryConservativeV3 candidate reproduces `6/8`, with Gate 1 and Gate 2 false. Gate 1's C/A concentration retention, structural/membrane turnover, catalyst R_X, and all non-catalyst label booleans pass; its first failed boolean is the legacy catalyst concentration `f_label=0.39221229068962093`, while the observer-only catalyst amount shadow gives `f_label=0.3277186407367453` and passes. All 15 candidate basin rows pass both the unchanged concentration basin predicate and the amount-aware shadow. Gate 2's snapshot/resume, membrane damage, structural damage, rupture recognition, and no-respawn checks pass; only the starvation predicate fails, with candidate final A `0.10147286122118783` versus control `0.010790080759377037`. Matched control/candidate chronology first diverges at step `1`; the geometry-frozen shadow has maximum difference `0`. Classification: `M1_GC_D087_MIXED_REGRESSION`.
- Changed areas: Observer-only diagnostic example and Cargo registration, compact Atlas evidence, and governance records. No conservation law, D-087/D-086 gate, parameter, production, or downstream behavior change.
- Validation:
  - Cargo check for `dcdev020m1r6r2r1_d087_semantics_audit` - PASSED
  - Exact D-087 control reproduction `8/8` - PASSED
  - Exact D-087 candidate reproduction `6/8`, Gates 1 and 2 false - PASSED
  - Gate 1 decomposition, Gate 2 15-row matrix, non-basin decomposition, matched chronology, geometry-frozen shadow, and amount-aware shadow - PASSED
  - Compact evidence written to canonical Atlas path - COMPLETE
  - Remote CI - NOT REQUIRED BY DIRECTIVE
  - Architect review - PENDING
- Remaining risks: The mixed classification is diagnostic, not a repair authorization. The legacy concentration tracer semantic contribution and the starvation-predicate regression require Architect interpretation; no criteria relaxation, initialization change, kinetic change, concentration controller, compensating synthesis, or biology repair is authorized.
- Blockers: Architect review and any separately authorized repair directive.
- Follow-up directive: none

## D-20260824-dcdev020m1r6r2r2-d087-catalyst-tracer-semantics-correction001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R2R2-D087-CATALYST-TRACER-CORRECTION-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R2R1-D087-SEMANTICS-MIXED-REGRESSION-REVIEW`
- Closed: `2026-08-24T02:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The D-087 catalyst replacement caller now supplies labeled catalyst material amounts (`tracer_c × area`) to the existing `replacement_report()` definition. The legacy concentration ratio remains recorded as diagnostic-only metadata. The unchanged ConservativeV3 control reruns `8/8`; the unchanged dirty GeometryConservativeV3 candidate reruns `7/8`, with Gate 1 passing at corrected catalyst `f_label=0.3277186407367453` and Gate 2 still failing only starvation. Historical R6-R2-R1 artifacts were not overwritten.
- Changed areas: `phase1-certifier/src/sim.rs`, focused `phase1-certifier` metric test, and local governance records only. No D-087 thresholds, equations, mechanics, remesh, transport, chemistry, resource, production, or downstream behavior changed.
- Validation:
  - Historical mismatch reproduced: legacy candidate catalyst `f_label=0.39221229068962093`; amount-aware value `0.3277186407367453` - PASSED
  - Catalyst amount fields recorded: initial `487.82105008375765`, candidate final `159.8680514562208` - PASSED
  - Actual ConservativeV3 control D-087: `8/8` - PASSED
  - Actual GeometryConservativeV3 candidate D-087: `7/8`; Gate 1 PASS, Gate 2 FAIL - PASSED / expected stop
  - Candidate starvation final A: `0.10147286122118783`, unchanged - CONFIRMED
  - Focused phase1-certifier tests: `5/5` - PASSED
  - Focused geometry/material conservation tests: `4/4` - PASSED
  - Rust formatting check: PASSED
  - Certifier binary check: PASSED
  - Governance validation: PASSED
  - Fresh compact evidence: `\\atlas\\ATLAS\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6r2r2\\compact` - COMPLETE
  - Remote CI: NOT REQUIRED BY DIRECTIVE
  - Architect review: PENDING
- Remaining risks: The candidate remains uncertified because the unchanged Gate 2 starvation predicate fails. The correction is observer/certifier-only and does not explain or authorize starvation repair.
- Blockers: Architect review of the corrected local evidence.
- Follow-up directive: none

## D-20260824-dcdev020m1r6r2r3-starvation-semantics-audit001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R2R3-STARVATION-SEMANTICS-AUDIT-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R2R2-D087-CATALYST-TRACER-CORRECTION-PENDING-ARCHITECT`
- Closed: `2026-08-24T03:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The unchanged D-087 certifier reproduces the ConservativeV3 chemistry plus ConservativeV2 material control at `8/8` and the GeometryConservativeV3 candidate at `7/8`, with Gate 2 starvation as the only candidate failure. Replaying the exact 200-step pre-switch sequence followed by zero interior/exterior N shows candidate `A=0.10147286122118783` at the historical relative step `6000`, but labeled A amount is `6.5956738242275`, area is `64.99938747021656` versus starvation-entry area `605.115299165549`, and organized material declines from `969.86673089034` to `84.5056465029607` by relative step `150000`. Post-switch N delivery is exactly `0`; F delivery remains allowed by the nitrogen-starvation design. Candidate observer viability and the existing `activated_catalyst_collapse` reason first fail at relative step `10383`. No topology rupture or physical-runtime invalidity occurs through the authorized `150000`-step bound, and closed topology remains true under the existing predicates. The geometry-frozen shadow reaches `A=0.0119095371850178` at relative step `6000`, showing that contraction accounts for the elevated full-candidate concentration while amount declines. Classification: `M1_GC_STARVATION_SURROGATE_STALE_CAUSAL_FAILURE_PRESERVED`.
- Changed areas: Observer-only starvation audit example, Cargo registration, compact Atlas evidence and manifest, and governance records. No chemistry, mechanics, transport, conservation law, D-087 gate, parameter, production, or downstream behavior changed.
- Validation:
  - Unchanged D-087 control: `8/8` - PASSED
  - Unchanged GeometryConservativeV3 candidate: `7/8`; Gate 2 only - PASSED / expected diagnostic boundary
  - Concentration/amount/geometry checkpoints through relative step `6000` - PASSED
  - Post-starvation zero-N delivery audit - PASSED
  - Observer chronology through relative step `150000` - PASSED
  - Geometry-frozen candidate shadow - PASSED
  - Conditional topology refeed - NOT EXERCISED because no topology rupture occurred
  - Sanctioned Rust 1.89.0 formatting and example check - PASSED
  - Compact Atlas evidence and manifest at `\\atlas\\ATLAS\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6r2r3\\compact` - COMPLETE
  - Remote CI - NOT REQUIRED BY DIRECTIVE
  - Architect review - PENDING
- Remaining risks: The old `A<0.05` Gate-2 surrogate is not changed by this diagnostic. The candidate remains uncertified under unchanged D-087 because its historical endpoint surrogate does not fire at 6000; actual topology death and topology recovery remain unresolved for this arm because no rupture occurred within the authorized bound. No starvation repair or Gate-2 rewrite is authorized.
- Blockers: Architect review and any separately authorized topology/death follow-up.
- Follow-up directive: none

## D-20260824-dcdev020m1r6r2r4-gc-preservation-qualification001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R2R4-GC-PRESERVATION-QUALIFICATION-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R2R3-STARVATION-SEMANTICS-AUDIT-PENDING-ARCHITECT`
- Closed: `2026-08-24T04:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The unchanged actual D-087 certifier returns V2/ConservativeV3 control `8/8` and GeometryConservativeV3 candidate `7/8`, with the sole historical Gate-2 failure isolated. The versioned GC qualifier passes the 15-row basin/decomposition and non-basin controls, records tracer `f_label=0.3277186407367453`, zero post-switch N delivery, observer viability loss at relative step `10383`, organized material decline from `969.8667308903401` to `84.50564650296067`, and no topology rupture or runtime invalidity through the authorized `150000`-step extension. Final remote closure passed at `dcce40b38b467615008699984a8cb52329b404e8`, with push CI `32760632483`, PR CI `32760643807`, and artifact digest `sha256:080f93693c34a22611351abba60a2aa23964938fc02039364b0fc127cac3fb86`. Classification: `M1_GC_CONSERVATION_CANDIDATE_QUALIFIED`.
- Changed areas: Versioned phase1-certifier GC preservation qualifier, compact generated evidence, scoped workflow/docs, and governance only. Historical D-087 Gate 2 and certified biology/equations remain unchanged; unrelated dirty work is not included.
- Validation:
  - Local sanctioned Rust 1.89 formatting/checks - PASSED
  - Actual D-087 control 8/8 and candidate 7/8 with sole historical Gate-2 failure - PASSED
  - 15-row decomposition, causal starvation gate, zero post-switch N delivery, conservation, and tracer preservation - PASSED
  - Compact Atlas evidence at `\\atlas\\ATLAS\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6r2r4\\compact` - PASSED
  - Exact-head push CI `32760632483` at `dcce40b38b467615008699984a8cb52329b404e8` - PASSED
  - Exact-head pull-request CI `32760643807` - PASSED
  - Remote artifact `dcdev020m1r6r2r4-gc-preservation-evidence`, digest `sha256:080f93693c34a22611351abba60a2aa23964938fc02039364b0fc127cac3fb86` - VERIFIED
  - Architect review - PENDING
- Remaining risks: This qualifies only the experimental GC conservation candidate. It does not close M1, select GC/V3 for production, establish sustained homeostasis, authorize reserve/recycling/salvage, M2, or DC-DEV-021.
- Blockers: Independent Architect review.
- Follow-up directive: none

## D-20260824-dcdev020m1r6r3-full-runtime-m1-certification001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R3-FULL-RUNTIME-M1-CERTIFICATION-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R2R4-GC-PRESERVATION-QUALIFICATION-PENDING-ARCHITECT`
- Closed: `2026-08-24T08:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only GeometryConservativeV3/ConservativeV3 full-runtime certification executes the unchanged finite-resource uptake, reactions, mechanics, remesh, and rebond order with strict per-stage closure. Fresh ConservativeV2 and ConservativeV3 D-087 preservation reports are both 8/8. The finite fed arm remains viable and closed at 8,000 steps but loses `82.9654506509167` organized material; 480-step deprivation followed by no-reset refeed does not restore the deficit. Zero-resource and feed-then-remove arms reach physical topology rupture at local steps `8867` and `11283`. Classification candidate: `M1_FULL_RUNTIME_HOMEOSTASIS_FAILED`; exact-head remote CI and Architect review remain pending.
- Changed areas: New observer-only certification example, Cargo registration, compact evidence, scoped workflow/docs, and append-only governance. No chemistry equations, transport/resource law, mechanics, remesh/rebond law, death rule, D-087/D-091, production selection, M2, recycling/salvage, or DC-DEV-021 changes.
- Validation:
  - Governance validator ADOPTED - PASSED
  - Rust formatting and compact fail-closed verifier - PASSED
  - Geometry/material conservation - 4 passed
  - Phase-1 metrics semantics - 5 passed
  - D-088 preservation - 4 passed
  - D-091 preservation - 8 passed
  - Evolution-harness preservation - 40 passed
  - Fresh V2/V3 D-087 - 8/8 each
  - Dense ledgers - written to canonical Atlas R6-R3 evidence root
  - Remote exact-head CI - PENDING
  - Architect review - PENDING
- Remaining risks: This is a valid local negative candidate, not M1 closure. Sustained fed homeostasis and no-reset restoration are not established; remote Linux execution, artifact verification, and Architect disposition remain authoritative. No repair or downstream directive is authorized.
- Blockers: Exact-head remote CI and Architect review.
- Follow-up directive: none

## D-20260825-dcdev020m1r6r3r1-gc-snapshot-area-semantics-correction001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R3R1-GC-SNAPSHOT-AREA-SEMANTICS-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R3-FULL-RUNTIME-M1-CERTIFICATION-PENDING-ARCHITECT`
- Closed: `2026-08-25T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The R6-R3 Linux invalidation is causally reproduced as a snapshot observer mismatch: the historical `max(area, 1e-9)` floor disagreed with GeometryConservativeV3 actual-area conservation below the floor. Snapshot callers are diagnostic/accounting/certification-only. GeometryConservativeV3 now uses actual positive area while HistoricalV1 and ConservativeV2 retain the historical floor. The unchanged local R6-R3 replay closes all stages and returns `M1_FULL_RUNTIME_HOMEOSTASIS_FAILED`; exact-head remote CI and Architect acceptance remain pending.
- Changed areas: `chemistry-core/src/mesh_contracts.rs` observer accounting only, focused `d098` tests, R6-R3 scoped workflow/evidence namespace/docs, and append-only governance. No certified biology/equations, physical trajectory path, transport, chemistry, mechanics, remesh, rebond, death, production, or downstream behavior changed.
- Validation:
  - Focused GC sub-floor and historical V1/V2 snapshot tests `7/7` - PASSED
  - Local V3/V2 D-087 preservation - `8/8` each - PASSED
  - Fed organized delta `-82.9654506509167` - PRESERVED
  - Deprivation delta `-10.979091022310868` - PRESERVED
  - No-reset refeed delta from deprived `-75.90268439405197` - PRESERVED
  - Fed, recovery, zero-resource, and feed/remove stage closure - PASSED within `1e-8`
  - Local compact evidence - `digital-protocell/experiments/generated/dcdev020m1r6r3r1/` - COMPLETE
  - Dense evidence - `\\atlas\\ATLAS\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6r3r1\\` - COMPLETE
  - Remote exact-head CI - PENDING
  - Architect review - PENDING
- Remaining risks: Full-runtime homeostasis and no-reset restoration remain failed; irreversible death remains platform-sensitive until remote replay; no repair or downstream execution is authorized.
- Blockers: Exact-head push/PR CI and Architect review.
- Follow-up directive: none

## D-20260826-dcdev020m1r6r3r2-gc-reaction-area-semantics-audit001 - PARTIAL

- Outcome ID: `OUT-DCDEV020M1R6R3R2-GC-REACTION-AREA-SEMANTICS-PENDING-ARCHITECT`
- Supersedes outcome: `OUT-DCDEV020M1R6R3R1-GC-SNAPSHOT-AREA-SEMANTICS-PENDING-ARCHITECT`
- Closed: `2026-08-26T00:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The observer-only R2 audit maps the active historical reaction-area floor bridges and records per-step predicted transfer residuals, conditioning data, and exact-parameter frozen replays. Local Windows execution reaches topology rupture before sub-floor reaction area. Exact-head Linux push and PR diagnostics passed and classify the historical reaction-area floor as causally responsible for the remaining GC reaction residual.
- Changed areas: `examples/dcdev020m1r6r3_full_runtime_m1_certification.rs`, the R2 diagnostic workflow, R2 documentation, R2 compact evidence, and append-only governance only. No production chemistry source was changed.
- Validation:
  - Local Rust 1.89.0 compile - PASSED
  - Observer replay with live reaction parameters and timestep - PASSED locally
  - Local physical trajectory remains the unchanged runtime path; topology rupture preempts sub-floor observation on Windows
  - Dense evidence target - `\\atlas\\ATLAS\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6r3r2\\`
  - Exact-head Linux push CI `32957391440` - PASSED; artifact digest `sha256:bb442b7d2f8955830013e19bf82c6f9351de9dfcb07bf1badcd3e9ca03f52134`
  - Exact-head Linux PR CI `32957395432` - PASSED; artifact digest `sha256:5f1abcedf4ea82b713f8304e462c70e784fbcaf5dc833cd2cfa402a1406c8072`
  - Architect review - PENDING
- Remaining risks: The local platform cannot establish or disprove Linux sub-floor floor causality; Linux exact-head evidence now confirms it. The R6-R3 homeostasis failure remains preserved; no production repair or downstream execution is authorized.
- Blockers: Architect review only.
- Follow-up directive: none
