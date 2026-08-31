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

## D-20260831-dcdev021-m2-entry010-post-ingestive-material-signal-substrate-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY010-POST-INGESTIVE-MATERIAL-SIGNAL-SUBSTRATE-AUDIT`
- Supersedes outcome: none
- Closed: `2026-08-31T12:06:37-04:00`
- Acceptance: `PARTIAL`
- Summary: Local observer-only ENTRY-010 execution classifies `M2_POST_INGESTIVE_MATERIAL_SIGNAL_SUBSTRATE_QUALIFIED`. The exact ENTRY-009 mechanics/contact fixture transfers `0.1474334734486514` N and F per species; the first successful transfer and first divergence from the matched no-transfer arm occur at step `0`. Reconstructed V4 N/F material amounts distinguish successful transfer from persistent contact without transfer and remain distinguishable for all `480` accepted steps. Concentrations are geometry-confounded because existing mechanics reads N/F in local osmotic pressure; amount reconstruction removes that confound. The fixture does not advance `reactions_step`, so A/W/C are not downstream changes in this assay; the existing N+F to A+W relation is recorded by source audit only.
- Changed areas: one observer-only ENTRY-010 example/registration, compact evidence, scoped workflow, and governance only; no scientific runtime source changed.
- Validation:
  - Exact ENTRY-009 base and frozen source scope - PASSED locally
  - Transfer/contact-only paired replay, empty resource control, amount reconstruction, persistence, and forbidden-information audit - PASSED locally
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: exact-head remote validation is pending; this qualifies signal substrate availability only and does not qualify local exploitation or autonomous resource acquisition.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260829-post-m1-clean-capability-baseline - PARTIAL

- Outcome ID: OUT-DCDEV020-POST-M1-CLEAN-CAPABILITY-BASELINE
- Supersedes outcome: none
- Closed: `2026-08-29T12:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Clean-baseline extraction is in progress from the pre-M1 base. Accepted V4 runtime and required downstream capability sources are retained; validation and Architect review remain pending.
- Changed areas: baseline branch tree, compact evidence, current governance, and scoped validation workflow; PR #44 and scientific source authority remain unchanged.
- Validation:
  - Exact accepted source closure head and PR provenance - PASSED
  - Dependency/reference inventory - PASSED
  - Clean baseline extraction - NOT RUN
  - Exact-head Linux validation - NOT RUN
- Remaining risks: downstream preservation and exact-head CI may identify an extraction dependency that requires returning to Architect.
- Blockers: final validation and Architect acceptance.
- Follow-up directive: none

## D-20260829-post-m1-clean-capability-baseline-r1 - PARTIAL

- Outcome ID: OUT-DCDEV020-POST-M1-CLEAN-CAPABILITY-BASELINE-R1
- Supersedes outcome: OUT-DCDEV020-POST-M1-CLEAN-CAPABILITY-BASELINE
- Closed: `2026-08-29T15:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Clean-baseline extraction from the pre-M1 base is complete. Accepted V4 runtime and required downstream capability sources are retained, the compact capability comparison is explicit, and the first exact-head Linux validation passed; final authority/Architect acceptance remains pending.
- Changed areas: baseline branch tree, compact evidence, current governance, and scoped validation workflow; PR #44 and scientific source authority remain unchanged.
- Validation:
  - Exact accepted source closure head and PR provenance - PASSED
  - Dependency/reference inventory - PASSED
  - Clean baseline extraction - PASSED
  - Retained scientific/runtime source byte identity - PASSED
  - Production selector smoke - PASSED (`MaturationCoupledV4`, reserve OFF)
  - Reusable downstream foundation preservation - PASSED
  - D-087 preservation boundary - PASSED (`8/8`, `8/8`, `7/8`)
  - Exact-head Linux validation run `33270482562` at `9b45e41c2d5f6ed908665d53051eb73b09dad420` - PASSED
- Remaining risks: final manifest-bearing exact-head CI and Architect acceptance.
- Blockers: final validation and Architect acceptance.
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

## D-20260829-dcdev021-m2-entry001-r1-d087-preservation-harness-repair - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY001-R1-D087-PRESERVATION-HARNESS-REPAIR`
- Supersedes outcome: none
- Closed: `2026-08-29T20:18:00-04:00`
- Acceptance: MET
- Summary: The D-087 preservation harness incorrectly used `MaturationCoupledV4` as the R9-R3 chemistry selector. The repaired canonical mapping uses ConservativeV3 chemistry plus the independent V4 flag. Immutable baseline and M2 selector-repair replay both returned V2 `8/8`, V3 `8/8`, and V4 `[true,true,false,true,true,true,true,true]`. The existing A-funded actuator assay remained qualified and all required downstream preservation checks passed.
- Changed areas: scoped workflow, compact evidence, governance, and documentation only; chemistry-core, phase1-certifier, V4/M1 physiology, reserve behavior, actuator law, and traction law were unchanged by R1.
- Validation:
  - Exact source/base authority and frozen scientific-source diff - PASSED
  - Canonical D-087 baseline/M2 parity - PASSED
  - A-to-W closure and R invariance - PASSED
  - Zero-A and zero-activity passive parity - PASSED
  - R-funded oracle parity and rotational equivariance - PASSED
  - Stick-slip retained displacement over matched controls - PASSED
  - Regulator, plasticity, contact, contact regulation, finite resource, historical traction, D-088, D-091, and evolution preservation - PASSED
  - Exact-head Linux CI run `33282801415` - PASSED at `39901a01ce17f42826351278a4321e65b1a99780`; artifact `sha256:74a537d379b5ebd9d50c72daa1e09fd604ff3e1c8c4b11c2f61138d01e22d72f`
  - Architect review - PENDING
- Remaining risks: autonomous resource acquisition has not been tested or established.
- Blockers: Architect acceptance; no successor M2 directive is authorized.
- Follow-up directive: none

## D-20260829-dcdev021-m2-entry002-temporal-navigation-substrate-audit - COMPLETE

- Outcome ID: OUT-DCDEV021-ENTRY002-TEMPORAL-NAVIGATION-SUBSTRATE-AUDIT
- Supersedes outcome: none
- Closed: `2026-08-29T21:13:39-04:00`
- Acceptance: MET
- Summary: `M2_TEMPORAL_NAVIGATION_EXPLORATION_SUBSTRATE_INSUFFICIENT`. The frozen zero-resource regulator state produces no local activity, A spending, funded tension, centroid path length, or reorientation. A direct A-funded substitution preserves the DC-DEV-013 instantaneous-contact negative, so no existing target-free temporal-navigation substrate is ready for implementation.
- Changed areas: observer-only audit example/registration, compact evidence, scoped workflow, and governance; no chemistry-core, phase1-certifier, V4/M1 physiology, regulator, plasticity, contact/resource boundary, contractility law, traction law, production selector, or reserve behavior changed.
- Validation:
  - Exact ENTRY-001 authority and frozen-source boundary - PASSED
  - Historical DC-DEV-013 causal reconstruction - PASSED
  - Direct A-funded instantaneous-contact replay - PASSED as negative-route evidence
  - Resource-independent exploration audit - PASSED as insufficiency evidence
  - Accepted ENTRY-001 actuator preservation - PASSED
  - Canonical D-087 boundary V2/V3/V4 - PASSED
  - Regulator/plasticity/contact/resource/traction/D-088/D-091/evolution preservation - PASSED
  - Exact-head Linux CI 33285014072 at b754d48d23264d14559a614853bdc60a38973dd3; artifact sha256:632df5c707c982434f2571d704b356971eccdca838ccbc52a760dc368c6599bc - PASSED
  - Architect review - NOT RUN
- Remaining risks: any future navigation architecture must be separately authorized; autonomous resource acquisition is not established.
- Blockers: Architect review; no successor is authorized.
- Follow-up directive: none

## D-20260829-dcdev021-m2-entry003-intrinsic-exploration-feasibility - PARTIAL

- Outcome ID: OUT-DCDEV021-ENTRY003-INTRINSIC-EXPLORATION-FEASIBILITY
- Supersedes outcome: none
- Closed: `2026-08-29T22:15:00-04:00`
- Acceptance: PARTIAL
- Summary: `M2_INTRINSIC_EXPLORATION_MECHANICALLY_INSUFFICIENT` locally. The explicit opt-in intrinsic state produces changing asymmetric local activity and spends current A into W with a residual below `1e-8` while preserving R, but its material-centroid path is numerical-scale and does not exceed either frozen motor-off control. The no-substrate full-dynamics control also retains deformation-related centroid drift, so retained substrate-mediated exploration is not established.
- Changed areas: new `regulatory-core` opt-in exploration module/export, feasibility example/registration, compact evidence, workflow, and governance only; chemistry-core, phase1-certifier, V4/M1 physiology, reserve behavior, existing regulator/plasticity, historical/qualified actuator APIs, and traction law are unchanged.
- Validation:
  - Exact ENTRY-002 authority and frozen-source boundary - PASSED locally
  - Intrinsic activity dynamics and dominant-patch switching - PASSED
  - A-to-W material closure and R invariance - PASSED
  - Zero-A passive parity, seed diversity, and rotational equivariance - PASSED
  - Retained stick-slip displacement over matched controls - FAILED as preregistered negative result
  - Full mesh restart hash continuity - FAILED; intrinsic-state hash continuity passed, but generic mesh JSON reconstruction differed and was not repaired under this directive
  - Exact-head Linux CI `33286398021` at `8602e4f273837fc27c69fffef0e4bd9972a4aaf2`; artifact `sha256:2b3f9b78b9b43473efe08137f994b361740c6a06188d5b253a221bc45874275f` - PASSED
  - Architect review - PENDING
- Remaining risks: the frozen actuator/traction composition does not currently establish a target-free exploration substrate; no tuning or follow-up mechanism is authorized.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260829-dcdev021-m2-entry004-intrinsic-traction-transfer-audit - COMPLETE

- Outcome ID: OUT-DCDEV021-ENTRY004-INTRINSIC-TRACTION-TRANSFER-AUDIT
- Supersedes outcome: none
- Closed: `2026-08-29T23:00:00-04:00`
- Acceptance: MET
- Summary: `M2_INTRINSIC_TRACTION_ADAPTATION_SUPPRESSION_CONFIRMED`. ENTRY-003 effective activity never crosses the frozen `0.45` static traction limit, while the raw intrinsic state would cross it 11,345 times on the identical clone trajectory.
- Changed areas: observer-only audit example/registration, compact evidence, scoped workflow, and governance only; chemistry-core, phase1-certifier, V4/M1 physiology, intrinsic explorer, contractility, traction, plasticity, reserve behavior, and production selection are unchanged.
- Validation:
  - Exact ENTRY-003 authority and frozen scientific-source boundary - PASSED locally
  - ENTRY-001 reproduction (`0.005665433467909554` displacement; `76` slips) - PASSED
  - ENTRY-003 reproduction (`0` slips; numerical-scale retained path) - PASSED
  - Free-clone clutch prediction versus actual ledger parity - PASSED
  - Effective/raw/unit-peak force decomposition - PASSED; adaptation attenuation is decisive under the preregistered rule
  - Intrinsic-state restart - PASSED; generic full-mesh JSON restart remains FAILED but does not affect uninterrupted force reconstruction
  - Regulatory-core focused tests (46) - PASSED
  - Canonical D-087 boundary V2/V3/V4 - PASSED locally (`8/8`, `8/8`, exact V4 `7/8` vector)
  - Exact-head Linux CI `33288018734` at `0bfda0cc0dd04d7b27e241e45068c124bdd808b7`; artifact `sha256:923f9028babd3e92fe1127460575468995221415c7971190e85fc3c0a70c859f` - PASSED
  - Architect review - PENDING
- Remaining risks: the audit does not authorize changes to activity adaptation, amplitude, or the frozen traction mechanism; retained exploration and autonomous acquisition remain unestablished.
- Blockers: Architect review; no successor execution is authorized.
- Follow-up directive: none

## D-20260830-dcdev021-m2-entry005-refractory-motor-decoupling - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY005-REFRACTORY-MOTOR-DECOUPLING`
- Supersedes outcome: none
- Closed: `2026-08-30T00:40:00-04:00`
- Acceptance: MET
- Summary: `M2_REFRACTORY_MOTOR_DECOUPLING_EXPLORATION_QUALIFIED`. The opt-in refractory-only motor schema leaves the ENTRY-003 intrinsic excitation and adaptation equations unchanged, including adaptation inside self-excitation, but supplies raw `activity_after` to the existing A-funded motor. It produces 325 frozen-clutch slips and retained material-centroid path `0.04095445278706012`, while preserved ENTRY-003 remains numerical-scale with zero slips.
- Changed areas: additive `intrinsic_exploration` API/export, feasibility example/registration, compact evidence, scoped workflow, and governance only; M1/V4, chemistry-core, phase1-certifier, DC-DEV-005 plasticity, historical/qualified actuator APIs, traction, mechanics, reserve behavior, and production selection remain unchanged.
- Validation:
  - Exact ENTRY-004 authority and frozen-source boundary - PASSED
  - ENTRY-003 state/adaptation equation parity - PASSED
  - Adaptation causal role inside intrinsic dynamics - PASSED (`max adaptation 0.712303095377031`)
  - Frozen clutch engagement and predicted/actual parity - PASSED (`325` slips; maximum required force `0.4766371923900446` above frozen `0.45` limit)
  - Retained exploration, substrate artifact exclusion, rotation, and fixed seed diversity - PASSED
  - A-to-W closure and R invariance - PASSED (`6.394884621840902e-14` residual; R unchanged)
  - Zero-A and historical ENTRY-003 controls - PASSED
  - Intrinsic-state restart - PASSED; generic full-mesh JSON restart remains `KNOWN_FAIL` and does not affect this uninterrupted result
  - Canonical D-087 boundary - PASSED (`8/8`, `8/8`, V4 `[true,true,false,true,true,true,true,true]`)
  - Regulator, continuity, plasticity, contact, contact regulation, finite-resource, traction, D-088, D-091, and evolution preservation - PASSED
  - Exact-head Linux CI `33292817570` at `e8349df8c1ef839b23e97d4bfe7c5b75b00b0b5a`; artifact `sha256:654af0ac349ca13662416593bb152f581672de370eca75fcbdc382a458c80194` - PASSED
  - Architect review - PENDING
- Remaining risks: this establishes target-free retained intrinsic exploration only; resource-biased navigation and autonomous acquisition remain untested and unestablished.
- Blockers: Architect review; no successor directive is authorized.
- Follow-up directive: none

## D-20260830-dcdev021-m2-entry006-unguided-resource-acquisition - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY006-UNGUIDED-RESOURCE-ACQUISITION`
- Supersedes outcome: none
- Closed: `2026-08-30T01:30:00-04:00`
- Acceptance: PARTIAL
- Summary: The exact frozen DC-DEV-013 ecology replay returns `M2_UNGUIDED_RESOURCE_ACQUISITION_NOT_ESTABLISHED`. ENTRY-005 produces retained movement and exact A-to-W closure without reading resource contact, but cumulative N/F acquisition is `0.2948669468973028`, below both ENTRY-003 and motor-off controls at approximately `0.3550441352752`; physical exposure is unchanged.
- Changed areas: new observer-only ENTRY-006 assay/registration, compact evidence, scoped workflow, and governance only; chemistry-core, phase1-certifier, M1/V4, intrinsic explorer, actuator, traction, plasticity, resource boundary, reserve, and production selection remain unchanged.
- Validation:
  - Exact ENTRY-005 source authority and PR #44 provenance - PASSED
  - Exact 5,000-step settlement and 480-step DC-DEV-013 ecology fixture - PASSED
  - Resource signal disconnected from organism causality - PASSED
  - ENTRY-005 unguided locomotion, A-to-W closure, and R invariance - PASSED
  - Finite N/F world-to-organism conservation and empty-resource specificity - PASSED
  - Historical 10% acquisition benefit and physical contact benefit - FAILED as preregistered negative result
  - Rotation and material/vertex centroid artifact exclusion - PASSED
  - Canonical D-087, downstream preservation, governance validation, and exact-head Linux CI - PENDING
- Remaining risks: target-free locomotion alone does not establish autonomous resource acquisition in the frozen ecology; no sensory-bias mechanism is authorized by this result.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260830-dcdev021-m2-entry006-unguided-resource-acquisition - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY006-UNGUIDED-RESOURCE-ACQUISITION-EXACT-HEAD`
- Supersedes outcome: `OUT-DCDEV021-ENTRY006-UNGUIDED-RESOURCE-ACQUISITION`
- Closed: `2026-08-30T09:35:00-04:00`
- Acceptance: MET
- Summary: `M2_UNGUIDED_RESOURCE_ACQUISITION_NOT_ESTABLISHED`. Under the exact frozen DC-DEV-013 ecology, target-free ENTRY-005 locomotion remains physically valid but does not improve finite N/F capture: unguided cumulative acquisition is `0.2948669468973028`, below ENTRY-003 `0.3550441352751993` and motor-off `0.35504413527520107` (about `-16.949%` versus each). Time-integrated exposure and final exposed patches are unchanged, so no contact benefit is established.
- Changed areas: observer-only ENTRY-006 assay/registration, compact evidence, scoped workflow, and governance only; frozen M1/V4, chemistry-core, phase1-certifier, intrinsic exploration, actuator, traction, plasticity, resource boundary, reserve, production selection, and PR #44 remain unchanged.
- Validation:
  - Resource signal remains observer-only; no resource center, radius, or inventory enters exploration, intrinsic dynamics, adaptation, motor, or traction - PASSED
  - ENTRY-005 locomotion, A-to-W closure, R invariance, finite-resource conservation, empty-sham specificity, rotation, and centroid artifact exclusion - PASSED
  - Historical 10% acquisition and contact-benefit criteria - FAILED as preregistered valid negative results
  - Canonical D-087 boundary and regulator, plasticity, contact, contact-regulation, finite-resource, traction, D-088, D-091, and evolution preservation - PASSED
  - Exact-head Linux CI `33314318443` at `c8df61f6fdf4a0dafe7d3b859ed57c845a535136`; artifact `sha256:a350ff42ccdd99d8b025024a5229136d8c5157b187e84d58e85a7397d3867f74` - PASSED
  - Architect review - PENDING
- Remaining risks: unguided exploration alone does not establish autonomous resource acquisition in the frozen ecology; no sensory-bias mechanism is authorized by this evidence.
- Blockers: Architect review; no successor directive is authorized.
- Follow-up directive: none

## D-20260830-dcdev021-m2-entry007-uptake-degradation-mechanism-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY007-UPTAKE-DEGRADATION-MECHANISM-AUDIT`
- Supersedes outcome: none
- Closed: `2026-08-30T09:43:00-04:00`
- Acceptance: PARTIAL
- Summary: The local observer-only Entry-007 replay completed as `M2_ENTRY007_UPTAKE_DEGRADATION_AUDIT_COMPLETE`. The diagnostic projection matches the unchanged DC-DEV-008 production ledger per step and exposed edge across the unguided ENTRY-006, ENTRY-003 pinned, and motor-off arms; autonomous resource acquisition remains `NOT_ESTABLISHED`.
- Changed areas: read-only `FiniteSpatialResourceRegionV1::uptake_diagnostic`, its focused parity test, the Entry-007 assay/registration, compact evidence, and governance only.
- Validation:
  - Diagnostic projection does not mutate mesh or region - PASSED
  - Per-step diagnostic totals equal production uptake ledger - PASSED
  - Finite N/F world-loss conservation across all three arms - PASSED
  - Fixed 5,000-step settlement and 480-step arm replay - PASSED
  - Compact per-step/per-edge decomposition evidence - PASSED
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: this is a mechanism decomposition only; it does not establish a sensory-bias mechanism or autonomous resource acquisition.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260830-dcdev021-m2-entry007-uptake-degradation-mechanism-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY007-UPTAKE-DEGRADATION-MECHANISM-AUDIT-R1`
- Supersedes outcome: `OUT-DCDEV021-ENTRY007-UPTAKE-DEGRADATION-MECHANISM-AUDIT`
- Closed: `2026-08-30T10:05:00-04:00`
- Acceptance: PARTIAL
- Summary: The assay-local clone-only reconstruction reproduces Entry-006 exactly and locates the first uptake divergence at step `116` on exposed edges `0` and `23` for both ENTRY-003 and motor-off controls. Exposed edge identities, length, occupancy, and permeability are unchanged at that step; area, interior N/F concentration, driving force, and finite-inventory trajectory diverge. Total N+F acquisition is unguided `0.2948669468973028`, ENTRY-003 `0.3550441352751993`, motor-off `0.35504413527520107`; the primary local classification is `M2_UPTAKE_DEGRADATION_CONCENTRATION_FEEDBACK_CONFIRMED`. The pre/post requested N/F values at the first divergent step are `0.0003796020752095074` and `0.0003789886454808434`; observer substitutions yield cumulative instantaneous recovered N/F of `3.0813060891968678` for segment length, `3.078078388092634` for permeability, and `3.040414616703692` for driving force. Contact remains positive for `480/480` steps with one entry, zero exits, and zero transitions after initial entry.
- Changed areas: assay-local Entry-007 reconstruction, required compact artifacts, scoped workflow, and governance only. `spatial_resource.rs`, intrinsic exploration, contractility, traction, chemistry-core, phase1-certifier, production selection, M1, and PR #44 are unchanged relative to accepted Entry-006 head.
- Validation:
  - Local replay, diagnostic-to-production ledger parity, conservation, first divergence, pre/post move, counterfactual decompositions, contact audit, scoped rustfmt, and diff checks - PASSED
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: decomposition is locally evidenced but not yet remotely accepted; it does not qualify autonomous resource acquisition or authorize a corrective mechanism.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260830-dcdev021-m2-entry007-uptake-degradation-mechanism-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY007-UPTAKE-DEGRADATION-MECHANISM-AUDIT-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY007-UPTAKE-DEGRADATION-MECHANISM-AUDIT-R1`
- Closed: `2026-08-30T15:18:39-04:00`
- Acceptance: PARTIAL
- Summary: Exact-head Linux CI run `33324714690` passed on result head `99884caa9fcbed148616b680f70734f58b1ddc41`. It completed authority, Entry-006 reproduction, Entry-007 uptake reconstruction/decomposition, canonical D-087, downstream preservation, governance validation, SHA-256 recording, and artifact upload. The primary classification remains `M2_UPTAKE_DEGRADATION_CONCENTRATION_FEEDBACK_CONFIRMED`; autonomous resource acquisition remains `NOT_ESTABLISHED`.
- Changed areas: assay-local Entry-007 reconstruction, compact evidence, scoped workflow, and governance only. Frozen scientific sources and production behavior remain unchanged relative to accepted Entry-006 head `6bfc4839b68e328bab7d89f896dd575fabb5baa7`.
- Validation:
  - Exact-head Linux CI run `33324714690` - PASSED
  - Uploaded artifact digest `sha256:b6ed78089b95301a24ce80deb812d6eefe9f439d9e78ff5550fc4aa4fea39094` - RECORDED
  - Architect review - NOT RUN
- Remaining risks: this is a mechanism decomposition only; it does not qualify autonomous resource acquisition or authorize a corrective mechanism.
- Blockers: Architect review.
- Follow-up directive: none
