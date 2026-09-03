# Project Outcome Ledger Template

After adoption, this append-only ledger records results for project directives. Every live outcome must reference one local directive ID.

## Entry schema after adoption

Use live outcome headings only after adoption. The following schema is instructional and is not a live entry:

Allowed adopted-project outcome states: `COMPLETE`, `PARTIAL`, `BLOCKED`, `FAILED`, `CANCELLED`, `SUPERSEDED`. Do not rewrite earlier entries; append corrections referencing the original.

## D-20260831-dcdev021-m2-entry015-polarity-actuator-interface - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY015-POLARITY-ACTUATOR-INTERFACE-LOCAL`
- Supersedes outcome: none
- Closed: `2026-08-31T22:03:33-04:00`
- Acceptance: `PARTIAL`
- Summary: The authorized ENTRY-015 isolated assay is implemented from Architect-accepted ENTRY-014 head `7685ae33e33132452105611322dbf4d045468eec`. It reimplements the accepted 24-site M2071-derived polarity equations locally and sends only the parameter-free active fraction `u/(u+v)` to the unchanged A-funded contractility/stick-slip path; matched spatial/uniform and motor-off controls are included. Local execution and exact-head validation remain pending.
- Changed areas: additive ENTRY-015 assay/example registration, compact evidence, scoped workflow, and append-only governance only; no Digital Cell scientific runtime source or PR #44 change.
- Validation:
  - Exact ENTRY-014 accepted starting head and clean branch boundary - PASSED
  - Local release assay, interface controls, chemistry replay, timing refinement, closure, and rotation - PASSED locally
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: final remote validation may identify workflow or preservation defects; the assay does not establish production polarity, autonomous polarity initiation, or autonomous resource acquisition.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260903-dcdev021-m2-entry028-balanced-separated-resource-ecology - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY028-BALANCED-SEPARATED-RESOURCE-ECOLOGY-LOCAL`
- Supersedes outcome: none
- Closed: 2026-09-03T09:45:00-04:00
- Acceptance: `PARTIAL`
- Summary: ENTRY-028 local execution replayed the exact accepted ENTRY-027 unforced 198→78/122 fission and ran all 24 daughter/bearing spatial, same-mean, and motor-off arms. Initial gaps were exactly one daughter mean edge with zero initial contact. Physical contact occurred, but no spatial arm showed a causal N/F acquisition advantage over both controls; locomotion remained active and the bounded local classification is `M2_SEPARATED_RESOURCE_CONTACT_WITHOUT_ACQUISITION_ADVANTAGE`.
- Changed areas: additive ENTRY-028 assay/example, compact evidence, scoped workflow, append-only governance, and the explicitly authorized ENTRY-027 presentation-header correction; no accepted scientific runtime source or PR #44 modification.
- Validation:
  - Local release build and assay - PASSED
  - Exact-head Linux CI - NOT RUN
  - Independent artifact digest - NOT RUN
  - Governance validator - PASSED
  - Notion readback - NOT RUN
- Remaining risks: remote exact-head validation and Architect review remain pending; autonomous resource acquisition and environment-dependent evolution remain unestablished.
- Blockers: none.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry025-live-antagonistic-inherited-locomotion - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY025-LIVE-ANTAGONISTIC-INHERITED-LOCOMOTION-LOCAL`
- Supersedes outcome: none
- Closed: 2026-09-02T18:30:00-04:00
- Acceptance: `PARTIAL`
- Summary: Exact ENTRY-024 head `f1e13c9d001e336e1f41ef63441950c0ff893c42` was reproduced through the accepted physical fission boundary and strict zero-pool eligibility. Live post-fission antagonistic inherited-polarity arms completed without scientific runtime changes. Neither daughter A nor daughter B exceeded its same-mean control or motor-off control under the preregistered spatial-leverage comparison; the bounded local classification is `M2_LIVE_ANTAGONISTIC_INHERITED_POLARITY_COMPOSITION_INSUFFICIENT`.
- Changed areas: additive ENTRY-025 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, fission, remesh, polarity production, actuator, traction, resource, M1, restart, or PR #44 change.
- Validation:
  - Local release execution, direct-live parity, strict eligibility, full material closure, A-to-W closure, rotation/index invariance, historical preservation, production/D-087, downstream, and governance - PASSED
  - Exact-head Linux workflow - PENDING
  - Independent artifact ZIP digest - PENDING
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow, governance, or preservation defects; ENTRY-025 does not establish autonomous embodied locomotion or autonomous resource acquisition.
- Blockers: exact-head Linux validation and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry025-live-antagonistic-inherited-locomotion - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY025-LIVE-ANTAGONISTIC-INHERITED-LOCOMOTION-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY025-LIVE-ANTAGONISTIC-INHERITED-LOCOMOTION-LOCAL`
- Closed: 2026-09-02T18:30:00-04:00
- Acceptance: `MET`
- Summary: Exact-head Linux validation passed for ENTRY-025 on the final result head recorded below. The accepted ENTRY-024 metadata correction is append-only and does not rewrite the sealed ENTRY-024 artifact: `autonomous_polarity_initiation` is corrected to `QUALIFIED` in the new correction record. Live anti-fraction arms remain physically valid and active after strict eligibility, but neither daughter clears the preregistered spatial-leverage comparison against same-mean and motor-off controls. The bounded classification remains `M2_LIVE_ANTAGONISTIC_INHERITED_POLARITY_COMPOSITION_INSUFFICIENT`.
- Changed areas: additive ENTRY-025 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, fission, remesh, polarity production, actuator, traction, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact authority, ENTRY-024 correction, fission replay, zero-pool eligibility, live causal order, direct-live parity, closure, rotation/index invariance, historical classifications, production/D-087, downstream, governance, and local release execution - PASSED
  - Exact-head Linux workflow and independent artifact digest - PASSED
  - Architect review - PENDING
- Remaining risks: ENTRY-025 does not establish autonomous embodied locomotion, autonomous polarity initiation beyond the accepted inherited substrate, or autonomous resource acquisition; do not start successor work.
- Blockers: Architect review only.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry024-polarity-effector-semantic-orientation - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY024-POLARITY-EFFECTOR-SEMANTIC-ORIENTATION-R2`
- Supersedes outcome: none
- Closed: `2026-09-02T16:48:00-04:00`
- Acceptance: `MET`
- Summary: Exact-head Linux validation passed on result head `b2f7343f396830452655d42d1222d46b3ec3b469`. Direct `u/(u+v)` parity passed. The assay-only complementary `v/(u+v)` orientation showed leverage on both inherited daughters but only daughter B under the analytical reference fields, yielding `M2_EFFECTOR_ORIENTATION_DAUGHTER_DEPENDENT_UNRESOLVED`.
- Changed areas: additive isolated ENTRY-024 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, polarity production, resource, actuator, traction, M1, restart, or PR #44 change.
- Validation:
  - Exact authority, direct parity, complement identity, A-to-W closure, rotation/index invariance, historical preservation, production V4/reserve-OFF, D-087, downstream, governance, and local execution - PASSED
  - Exact-head Linux workflow `33680965538` on the exact result head - PASSED
  - Independently downloaded artifact ZIP digest `sha256:c7e56a987b34a9de10164be6e8079b2a5d53683f1796adcb9b74585d4e833cd1` - PASSED
  - Architect review - NOT RUN
- Remaining risks: the bounded result does not qualify a universal effector orientation, production polarity integration, autonomous polarity initiation, or autonomous resource acquisition.
- Blockers: Architect acceptance; no successor execution authorized.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry024-polarity-effector-semantic-orientation - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY024-POLARITY-EFFECTOR-SEMANTIC-ORIENTATION-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-02T16:20:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Local ENTRY-024 replay from exact accepted ENTRY-023 head `af3029f2ed9d3be3f31cdc6feb5eacfce6471b1e` reproduces direct `u/(u+v)` results. The assay-only complementary `v/(u+v)` orientation has leverage on both inherited daughters, but only daughter B under the analytical reference fields; the bounded classification is `M2_EFFECTOR_ORIENTATION_DAUGHTER_DEPENDENT_UNRESOLVED`.
- Changed areas: additive isolated ENTRY-024 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, polarity production, resource, actuator, traction, M1, restart, or PR #44 change.
- Validation:
  - Local release execution, direct parity, complement identity, A-to-W closure, rotation, index invariance, historical evidence, and governance scope - PASSED
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: exact-head Linux validation may identify workflow or preservation defects; the assay does not establish production polarity, autonomous polarity initiation, or autonomous resource acquisition.
- Blockers: exact-head Linux validation and Architect review; do not start successor work.
- Follow-up directive: none

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

## D-20260831-dcdev021-m2-entry011-frozen-uptake-metabolism-composition - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY011-FROZEN-UPTAKE-METABOLISM-COMPOSITION`
- Supersedes outcome: none
- Closed: `2026-08-31T16:26:37-04:00`
- Acceptance: PARTIAL
- Summary: The local observer-only ENTRY-011 composition reuses the exact public frozen production reaction kernel with `MaturationCoupledV4` and reserve OFF. It runs ENTRY-005 raw intrinsic locomotion, unchanged DC-DEV-008 finite N/F uptake, then the unchanged reaction step; metabolism is active and the local classification is `M2_FROZEN_UPTAKE_METABOLISM_COMPOSITION_QUALIFIED`.
- Changed areas: additive ENTRY-011 example registration, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry011/`, scoped workflow, and append-only governance only; no scientific runtime source changed.
- Validation:
  - Exact ENTRY-010 starting head and branch boundary - PASSED
  - Frozen metabolism authority identified as `chemistry_core::mesh_reactions::reactions_step_with_reserve_mode` with `ReactionParams::conservative_v3()` and `ReserveDiagnosticMode::Full`; production order documented as transport → reaction → mechanics/remesh/rebond - PASSED
  - N/F world conservation and N/F/A/W ledger closure - PASSED; maximum local closure residual `3.765876499528531e-13`
  - Metabolism active: cumulative N/F consumption `14.174330986478182` each, A production `14.174330986478182`, reaction W production `28.203488312042303` - OBSERVED
  - Acquisition `0.8602206124447573` versus no-metabolism `0.2948669468973028`; relative improvement `1.9173178665710457` - PASSED locally
  - Step-116 N/F driving force is higher with metabolism and reconstructed N+F buildup is lower - PASSED locally
  - Exploration retained: path `0.1804716432724705`, `3,579` slips, `12` dominant-patch changes - PASSED locally
  - Contact-without-transfer and empty-resource controls, forbidden-information audit, and resource-to-work comparison - PASSED locally
  - Historical ENTRY-005 through ENTRY-010, canonical D-087, downstream foundations, governance, and exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: the result is still in the frozen in-contact ecology and therefore does not establish autonomous resource acquisition; exact-head remote validation and Architect acceptance remain outstanding.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry011-frozen-uptake-metabolism-composition - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY011-FROZEN-UPTAKE-METABOLISM-COMPOSITION-R1`
- Supersedes outcome: `OUT-DCDEV021-ENTRY011-FROZEN-UPTAKE-METABOLISM-COMPOSITION`
- Closed: `2026-08-31T16:35:42-04:00`
- Acceptance: PARTIAL
- Summary: Exact-head Linux CI passed for the observer-only ENTRY-011 composition. The exact frozen M1/V4 production reaction kernel runs after unchanged DC-DEV-008 uptake and preserves ENTRY-005 target-free locomotion. Metabolism is active, N/F/A/W accounting closes, internal N+F buildup is reduced at step 116, and cumulative acquisition is `0.8602206124447573` versus `0.2948669468973028` without metabolism (`1.9173178665710457` relative improvement), passing the frozen 10% criterion. This qualifies metabolically live resource-exploitation composition only; autonomous resource acquisition remains `NOT_ESTABLISHED` because the fixture begins in contact.
- Changed areas: additive observer-only ENTRY-011 example, compact evidence, scoped workflow, and append-only governance; no chemistry-core, phase1-certifier, regulatory scientific runtime, uptake, metabolism, actuator, traction, M1, restart, production-selector, or PR #44 source changed.
- Validation:
  - Exact frozen production reaction authority and causal-order audit - PASSED
  - No-double-counting and full material closure - PASSED; maximum residual `1.1368683772161603e-13`
  - Frozen metabolic explorer activity - PASSED; N/F consumption `14.174330986478182` each, A production `14.174330986478182`, reaction W production `28.203488312042303`
  - Acquisition and exploration - PASSED; path `0.1804716432724705`, slips `3579`, dominant-patch changes `12`
  - Resource-to-work causal chain - `ESTABLISHED_IN_FIXTURE`
  - Historical replay, M1/D-087, downstream, governance, and restart-boundary preservation - PASSED
  - Exact-head Linux CI run `33436469789` on `6c1bfd7e139ff72dd8f342dc13c825eb8ec65405` - PASSED
  - Uploaded artifact digest `sha256:376564668d704a0b4924124bc08557c30580723fabd81b37388814e9c7d47967` - RECORDED
  - Architect review - PENDING
- Remaining risks: this is an assay-only metabolic composition and does not qualify autonomous acquisition or authorize a behavioral post-ingestive coupling.
- Blockers: Architect review.
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
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
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

## D-20260831-dcdev021-m2-entry010-post-ingestive-material-signal-substrate-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY010-POST-INGESTIVE-MATERIAL-SIGNAL-SUBSTRATE-AUDIT-R1`
- Supersedes outcome: `OUT-DCDEV021-ENTRY010-POST-INGESTIVE-MATERIAL-SIGNAL-SUBSTRATE-AUDIT`
- Closed: `2026-08-31T12:21:17-04:00`
- Acceptance: PARTIAL
- Summary: Exact-head Linux CI run `33413197919` passed on result head `5ad6e9cb8385f030a5de03941ec7a8bc6bac1d57`. The observer-only transfer/contact-without-transfer replay classifies `M2_POST_INGESTIVE_MATERIAL_SIGNAL_SUBSTRATE_QUALIFIED` with architectural boundary `EXISTING_INTERNAL_MATERIAL_SIGNAL_REUSABLE`. Unchanged DC-DEV-008 transfer first succeeds and first diverges from contact-only at step `0`; actual-area V4 reconstruction identifies N, F, and combined N+F material amounts as existing internal downstream material, while concentration-only values remain geometry-confounded. The distinction persists for all `480` accepted steps without new memory. The exact fixture does not advance activated metabolism, and no behavior is implemented.
- Changed areas: observer-only ENTRY-010 assay, durable evidence, scoped workflow, and governance only; no scientific runtime source, M1 source, production selector, or PR #44 changed.
- Validation:
  - Local assay, exact ENTRY-005 through ENTRY-009 preservation, production V4/reserve-OFF, canonical D-087, downstream foundations, governance, targeted rustfmt, and diff checks - PASSED
  - Exact-head Linux CI run `33413197919` - PASSED
  - Uploaded artifact digest `sha256:ac3b34585a38078043a8830e5e0a5664461229f72870abda7636c6c1cfce8491` - RECORDED
  - Architect review - NOT RUN
- Remaining risks: this qualifies signal-substrate availability only; local exploitation and autonomous resource acquisition remain `NOT_ESTABLISHED`.
- Blockers: Architect review.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry012-separated-resource-autonomous-acquisition - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY012-SEPARATED-RESOURCE-AUTONOMOUS-ACQUISITION-R1`
- Supersedes outcome: none
- Closed: `2026-08-31T17:36:27-04:00`
- Acceptance: PARTIAL
- Summary: The observer-only ENTRY-012 replay places the unchanged radius-`1.5`, N=`3`, F=`3` resource at `[7.636854591521829, 0.0]`, giving the settled body an exact mean-edge-length initial gap of `1.3036408078380952`. Seed 1 makes no physical encounter within the fixed `1,500` accepted steps, while frozen metabolism and ENTRY-005 locomotion remain active. The preregistered classification is `M2_SEPARATED_RESOURCE_ENCOUNTER_NOT_ESTABLISHED`; autonomous resource acquisition remains `NOT_ESTABLISHED`.
- Changed areas: additive observer-only ENTRY-012 example/registration, compact evidence, scoped workflow, and append-only governance only; frozen scientific sources, M1/V4, uptake, metabolism, actuator, traction, production selection, restart boundary, and PR #44 remain unchanged.
- Validation:
  - Exact starting head and branch creation - PASSED
  - Initial zero-contact proof and one-edge-length geometry derivation - PASSED
  - Precontact parity against no-resource metabolic twin - PASSED
  - Fixed-horizon seed-1 reachability: no encounter; path `0.33538885163612836`, net displacement `0.03988968845502883`, `9196` slips, `12` dominant-patch changes; closest remaining midpoint gap `1.3036408078380952` - PASSED
  - Rotation and material closure checks - PASSED
  - Historical ENTRY-005 through ENTRY-011 replay - PASSED
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: this result establishes a bounded reachability negative only; it does not establish autonomous acquisition, general search, navigation, or authorize a successor mechanism.
- Blockers: exact-head Linux CI and Architect review; do not move the resource or extend the horizon.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry013-search-persistence-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY013-SEARCH-PERSISTENCE-AUDIT`
- Supersedes outcome: none
- Closed: `2026-08-31T18:15:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Local observer-only ENTRY-013 execution reproduces the accepted ENTRY-012 resource-free metabolic explorer and classifies the bounded-search mechanism as `M2_SEARCH_REACH_POLARITY_DECAY_OR_HOMOGENIZATION_CONFIRMED`. Seed 1 retains active motion but has high path and low net displacement; K1 activity polarity decays strongly, with only a half-turn phase change and no complete phase cycle. An assay-only fixed-profile diagnostic shows unchanged mechanics can translate a persistent asymmetric profile. This is a diagnosis only and does not establish autonomous acquisition or authorize a motility mechanism.
- Changed areas: additive observer-only ENTRY-013 example/registration, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry013/`, scoped workflow, and append-only governance only; no scientific runtime source changed.
- Validation:
  - Exact accepted ENTRY-012 no-resource trajectory reproduction - PASSED
  - Ring-mode, polarity-persistence, kinematic, mechanical-proxy, phase-locked, fixed-profile, seed, and energetic observer audit - PASSED
  - Targeted Rust formatting and example compile/run - PASSED
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: remote exact-head validation and Architect acceptance remain pending; this diagnosis does not qualify autonomous resource acquisition or authorize persistence implementation.
- Blockers: exact-head Linux CI and Architect review.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry013-search-persistence-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY013-SEARCH-PERSISTENCE-AUDIT-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY013-SEARCH-PERSISTENCE-AUDIT`
- Closed: `2026-08-31T20:27:13-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head Linux workflow `33454498820` passed on result head `e98ed19d80066ea2f53a967b5c4275798d7665bd`. It reproduced ENTRY-005 through ENTRY-012, executed the observer-only ENTRY-013 audit, preserved canonical D-087 and downstream checks, validated governance, and uploaded the evidence artifact. Seed 1 reproduces the accepted ENTRY-012 no-resource trajectory; K1 activity polarity decays from `0.243646444843049` to `0.0020155616613880007`, with a half-turn and no complete phase cycle. The phase-locked diagnostic does not improve coherent translation, while the fixed-profile diagnostic reaches `0.09316990400571264` net displacement, showing unchanged mechanics can translate persistent asymmetry. A-to-W closure and seed-equivariant preservation pass. The result remains `M2_SEARCH_REACH_POLARITY_DECAY_OR_HOMOGENIZATION_CONFIRMED`; autonomous resource acquisition remains `NOT_ESTABLISHED`.
- Changed areas: additive observer-only ENTRY-013 assay/evidence/workflow/governance only; no scientific runtime source changed.
- Validation:
  - Exact-head Linux workflow `33454498820` - PASSED
  - Artifact digest `sha256:e549171c918c9b91fce8177b4d54076b9f42ba2efec2729885e3e4d963387981` - PASSED
  - Architect review - NOT RUN
- Remaining risks: this diagnosis does not qualify autonomous resource acquisition or authorize a persistence/motility mechanism. Prior workflow `33454372521` failed on a workflow assertion-key defect, not a scientific or implementation result; the assertion was corrected before the passing run.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry019-conservative-life-history-polarity-initiation - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY019-CONSERVATIVE-LIFE-HISTORY-POLARITY-INITIATION-LOCAL`
- Supersedes outcome: none
- Closed: 2026-09-01T12:20:00-04:00
- Acceptance: `PARTIAL`
- Summary: The isolated ENTRY-019 audit replays the accepted D-088 pre-fission physical history from exact ENTRY-018 head `e9d64534c565662e22aa67b76c5e00735970055f`, carries initially homogeneous polarity amounts by conservative material-local arclength overlap through 134 remesh events, and advances the unchanged accepted reaction-diffusion equations. The local classification is `M2_CONSERVATIVE_LIFE_HISTORY_POLARITY_INITIATION_QUALIFIED`: geometry-frozen and no-dilution controls remain homogeneous, transport creates a deterministic nonconstant perturbation, and the accepted Polar family amplifies it. The Traveling parameter family is seeded but does not amplify in this bounded assay.
- Changed areas: additive isolated ENTRY-019 example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry019/`, scoped workflow, and append-only governance only; no accepted scientific runtime source or PR #44 change.
- Validation:
  - Exact ENTRY-018 start, accepted D-088 pre-fission replay, native conservative mapping, weighted closure, homogeneous controls, causal attribution, and forbidden-information boundary - PASSED
  - Exact-head Linux workflow `33530286463` on result head `674b39d9f57e77597b61607a9a03eb1830e55f89` - PASSED
  - Independently downloaded artifact ZIP `sha256:02f9a0c414cc1f84216d8ef4d66c50dd68a6e27c08b406b3d6d821ffbf777948` - PASSED
  - Architect review - NOT RUN
- Remaining risks: ENTRY-019 does not establish production polarity initiation or autonomous resource acquisition.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry012-separated-resource-autonomous-acquisition - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY012-SEPARATED-RESOURCE-AUTONOMOUS-ACQUISITION-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY012-SEPARATED-RESOURCE-AUTONOMOUS-ACQUISITION-R1`
- Closed: `2026-08-31T17:54:00-04:00`
- Acceptance: PARTIAL
- Summary: Exact-head Linux workflow `33443167806` passed on result head `1855169de9d9ecc9f0c5137a1a5a6eb0c91e5c5d`. It reproduced ENTRY-005 through ENTRY-011, verified the preregistered initial noncontact and precontact no-resource parity, completed the fixed-horizon ENTRY-012 assay, preservation matrix, governance validation, and artifact upload. The seed-1 primary remained locomotively active but did not encounter the separated resource, yielding `M2_SEPARATED_RESOURCE_ENCOUNTER_NOT_ESTABLISHED`.
- Changed areas: additive observer-only ENTRY-012 example/registration, compact evidence, scoped workflow, and governance only; frozen scientific sources, M1/V4, uptake, metabolism, actuator, traction, production selection, restart boundary, and PR #44 remain unchanged.
- Validation:
  - Exact-head Linux workflow `33443167806` - PASSED
  - Artifact digest `sha256:cb4ca72ed3a9dde6a6e82c84c5d4ef2fec18895be6c39f8ce001900fb08ad844` - RECORDED
  - Architect review - PENDING
- Remaining risks: this remains a bounded separated-resource reachability negative; it does not establish autonomous acquisition, general search, navigation, or authorize a successor mechanism.
- Blockers: Architect review; do not move the resource or extend the horizon.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry014-excitable-polarity-reference-transfer - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY014-EXCITABLE-POLARITY-REFERENCE-TRANSFER-R1`
- Supersedes outcome: none
- Closed: `2026-08-31T21:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The isolated mathematical reimplementation of Morpheus M2071 reproduces a stationary non-homogeneous Polar regime and a moving dominant-mode Traveling-Wave regime using the versioned supplementary XML parameters, then preserves both regimes at exactly 24 periodic sites without biological parameter search, stochastic forcing, or Digital Cell runtime coupling. The resulting local classification is `M2_EXCITABLE_POLARITY_REFERENCE_TRANSFER_FEASIBLE`; autonomous resource acquisition remains `NOT_ESTABLISHED` and Architect acceptance is pending.
- Changed areas: additive isolated ENTRY-014 solver/example registration, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry014/`, scoped workflow, and append-only governance only. Frozen scientific runtime sources, M1/V4, actuator, traction, metabolism, uptake, resource, production selector, restart boundary, and PR #44 remain unchanged.
- Validation:
  - Exact ENTRY-013 accepted starting head and frozen source hashes - PASSED
  - M2071/M2072 provenance and license audit, including retained HTML/XML Traveling-Wave `b` discrepancy - PASSED
  - Independent Polar and Traveling-Wave reproduction - PASSED
  - `u+v` reaction exchange and periodic diffusion conservation - PASSED
  - Exact 24-site Polar and Traveling-Wave transfer plus 48-site resolution diagnostic - PASSED
  - Historical ENTRY-005 through ENTRY-013 preservation and targeted governance validation - PASSED locally
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: the audit qualifies only transfer of a published isolated polarity substrate; it does not qualify Digital Cell polarity integration, locomotion improvement, encounter, navigation, or autonomous resource acquisition.
- Blockers: exact-head Linux CI and Architect review; do not start a successor or production integration.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry014-excitable-polarity-reference-transfer - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY014-EXCITABLE-POLARITY-REFERENCE-TRANSFER-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY014-EXCITABLE-POLARITY-REFERENCE-TRANSFER-R1`
- Closed: `2026-08-31T21:31:02-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head Linux workflow `33458598747` passed on result head `32f7380eedbfca063ba23fed2609dee0680d4294`. It verified authority/scope, reproduced ENTRY-005 through ENTRY-013, ran the isolated M2071 reference solver, passed the Polar and Traveling-Wave 24-site transfer and conservation checks, preserved M1/downstream gates, validated governance, and uploaded the evidence artifact. The classification remains `M2_EXCITABLE_POLARITY_REFERENCE_TRANSFER_FEASIBLE`; Architect acceptance is pending and autonomous resource acquisition remains `NOT_ESTABLISHED`.
- Changed areas: exact CI result and artifact provenance for the additive isolated ENTRY-014 solver/evidence/workflow and append-only governance only; no scientific runtime source changed and PR #44 remains untouched.
- Validation:
  - Exact-head CI `33458598747` - PASSED
  - Artifact digest `sha256:38c76a6c05963854e12b935e9077c1bdfd4215eede4e851027647b64ac2352da` - RECORDED
  - Notion SOT top-level result update - RECORDED; historical content preserved
  - PR #44 preservation - VERIFIED before final governance seal; no mutation made
- Remaining risks: this is an isolated reference-transfer feasibility result only; it does not authorize Digital Cell polarity integration, locomotion modification, navigation, encounter, or autonomous acquisition.
- Blockers: final exact-head CI after governance seal and Architect review.
- Follow-up directive: none

## D-20260831-dcdev021-m2-entry015-polarity-actuator-interface - PARTIAL

- Outcome ID: OUT-DCDEV021-ENTRY015-POLARITY-ACTUATOR-INTERFACE-R1
- Supersedes outcome: `OUT-DCDEV021-ENTRY015-POLARITY-ACTUATOR-INTERFACE-LOCAL`
- Closed: 2026-08-31T22:21:36-04:00
- Acceptance: PARTIAL
- Summary: Exact-head Linux workflow `33461618844` passed on result head `1a4ec8f363186707adedaf70a8f4b8aa1fa6debc`. The isolated assay uses the accepted ENTRY-014 Polar and Traveling chemistry with the parameter-free local `u/(u+v)` interface, matched same-mean and motor-off controls, one-way chemistry isolation, energetic closure, and rotation. Polar spatial organization exceeds its same-mean control in coherent translation; the Traveling regime produces phase-coupled heading variation and spatial leverage. The result remains assay-only and does not establish autonomous polarity initiation or resource acquisition.
- Changed areas: additive ENTRY-015 assay/example registration, compact evidence, scoped workflow, and append-only governance only; no Digital Cell scientific runtime source or PR #44 change.
- Validation:
  - Exact ENTRY-014 accepted starting head and frozen source scope - PASSED
  - Local release assay, interface controls, chemistry replay, timing refinement, closure, and rotation - PASSED
  - Exact-head Linux workflow `33461618844` - PASSED
  - Artifact digest `sha256:40483a35cf28b3fcb71ee8c62595e92fd7142bcc4fe739576bcdf9c15f8b5009` and independent downloaded-archive hash - PASSED
  - Architect review - NOT RUN
- Remaining risks: the result qualifies only the assay-local polarity-to-effector interface; production polarity integration, autonomous polarity initiation, and autonomous resource acquisition remain out of scope.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry016-autonomous-polarity-initiation-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY016-AUTONOMOUS-POLARITY-INITIATION-AUDIT-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-01T06:48:42-04:00`
- Acceptance: `PARTIAL`
- Summary: The authorized observer-only ENTRY-016 audit runs from accepted ENTRY-015 head `4ca7d0ee7c9e135a1ecf8adfdd5525b02c67c6bd`. Exact accepted equations yield spatially unstable nonzero-mode linear growth for both Polar and Traveling parameter regimes, while exact homogeneous replay remains homogeneous. The settled 24-site MaturationCoupledV4 body has no physically nonuniform local field beyond numerical residue, so the local classification is `M2_POLARITY_INITIATION_ENDOGENOUS_ASYMMETRY_ABSENT`.
- Changed areas: additive isolated ENTRY-016 audit/example registration, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry016/`, scoped workflow, and append-only governance only; no scientific runtime source or PR #44 change.
- Validation:
  - Exact accepted ENTRY-015 starting head and frozen source scope - PASSED locally
  - Homogeneous equilibria, exact 24-site Fourier/Jacobian spectra, homogeneous replay, settled local-field inventory, provenance, rotation, forbidden-information, and mapping-boundary audit - PASSED locally
  - Exact-head Linux CI - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: exact-head remote validation and Architect acceptance remain outstanding; no polarity initialization or resource encounter has been implemented or tested.
- Blockers: exact-head Linux CI and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry016-autonomous-polarity-initiation-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY016-AUTONOMOUS-POLARITY-INITIATION-AUDIT-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY016-AUTONOMOUS-POLARITY-INITIATION-AUDIT-LOCAL`
- Closed: `2026-09-01T07:01:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head Linux workflow `33499640907` passed on implementation result head `ea482ca4f655f79967f5e0b1b99ca00974116f30`; the uploaded artifact and independently downloaded ZIP both hash to `sha256:0b452b3b3dd5b5759fd6f3fa0723704d341c9dde191a813de363778a7477a9c8`. The result remains `M2_POLARITY_INITIATION_ENDOGENOUS_ASYMMETRY_ABSENT`: both accepted polarity regimes have spatially unstable modes, but the exact settled organism contains no physically meaningful nonuniform local field.
- Changed areas: additive ENTRY-016 audit/example, compact evidence, scoped workflow, and append-only governance only; no scientific runtime source or PR #44 change.
- Validation:
  - Exact-head Linux workflow `33499640907` on implementation head - PASSED
  - Artifact digest and independent downloaded-archive hash - PASSED
  - Architect review - NOT RUN
  - Final sealed-head Linux workflow - NOT RUN
- Remaining risks: final sealed-head CI and Architect acceptance remain outstanding; no polarity initialization or resource encounter has been implemented or tested.
- Blockers: final sealed-head exact-head Linux CI and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry016-autonomous-polarity-initiation-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY016-AUTONOMOUS-POLARITY-INITIATION-AUDIT-R3`
- Supersedes outcome: `OUT-DCDEV021-ENTRY016-AUTONOMOUS-POLARITY-INITIATION-AUDIT-R2`
- Closed: `2026-09-01T07:13:05-04:00`
- Acceptance: `PARTIAL`
- Summary: Final sealed-head exact-head Linux workflow `33500356464` passed on result head `a80d4c4f4d5a92d7ba029e628977a77bc3563b5f`. The ENTRY-016 classification remains `M2_POLARITY_INITIATION_ENDOGENOUS_ASYMMETRY_ABSENT`: accepted polarity regimes have unstable spatial modes, but the exact settled organism contains no physically meaningful nonuniform local field to seed them.
- Changed areas: append-only governance correction and final validation provenance for the existing additive ENTRY-016 audit/evidence/workflow; no scientific runtime or PR #44 change.
- Validation:
  - Final sealed-head Linux workflow `33500356464` - PASSED
  - Artifact digest and independent downloaded-archive hash `sha256:4f1ffb699638f6b52826d16deb09ccecb5d5295df56b4f18c9470977e361b37d` - PASSED
  - Architect review - PENDING
- Remaining risks: Architect acceptance remains outstanding; no polarity initialization or resource encounter has been implemented or tested.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry017-post-fission-daughter-asymmetry-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY017-POST-FISSION-DAUGHTER-ASYMMETRY-AUDIT-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-01T07:59:29-04:00`
- Acceptance: `PARTIAL`
- Summary: The authorized observer-only ENTRY-017 audit replays the accepted D-088 physical growth/fission path from `bbbcc7c2bd8e25da69a36902107e7a7420c81ef0`. Fission occurs at replay step `2326`, producing a 198-site mother and 78/122-site daughters. Multiple lawful daughter-local geometry, strain, structural-material, maturation, membrane, and curvature fields are physically nonuniform, and the existing partition report passes. Because the daughters are not directly compatible with the accepted 24-site polarity ring, the local classification is `M2_POST_FISSION_ASYMMETRY_PRESENT_TOPOLOGY_MAPPING_UNRESOLVED`.
- Changed areas: additive observer-only ENTRY-017 assay/example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry017/`, scoped workflow, and append-only governance only; no scientific runtime or PR #44 change. The reproduced D-088 mother was already physically nonuniform before fission, so the assay supports preservation/partition of existing mother history, not de novo birth-generated asymmetry; no Phase-3 individuality claim is made.
- Validation:
  - Exact starting head, accepted physical fission authority, conservative partition closure, daughter snapshots, topology boundary, forbidden-information audit, rotation, and local replay - PASSED
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: exact-head remote validation may identify workflow or preservation defects; no polarity state or resource encounter was implemented or tested, and no daughter topology remapping is authorized.
- Blockers: exact-head Linux CI and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry018-native-material-ring-polarity-transfer - PARTIAL

- Outcome ID: OUT-DCDEV021-ENTRY018-NATIVE-MATERIAL-RING-POLARITY-TRANSFER-LOCAL
- Supersedes outcome: none
- Closed: 2026-09-01T09:30:00-04:00
- Acceptance: PARTIAL
- Summary: The authorized ENTRY-018 isolated audit replays the accepted continuous M2071-derived polarity equations on normalized physical arclength using conservative edge-centered control volumes. The exact accepted fission replay produces 198/78/122-site rings, regular-grid algebraic equivalence passes, weighted active+inactive conservation passes, and native Polar and Traveling-parameter spatial instabilities plus nonhomogeneous reference-pattern replays remain present on all three physical topologies. The local classification is `M2_NATIVE_MATERIAL_RING_POLARITY_TRANSFER_FEASIBLE`, pending exact-head Linux validation and Architect review.
- Changed areas: additive numerical example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry018/`, scoped workflow, and append-only governance only; no accepted scientific runtime source changed and no polarity initialization or behavior coupling was implemented.
- Validation:
  - Exact ENTRY-017 starting head and physical fission replay - PASSED
  - Regular-grid equivalence, weighted conservation, native stability, and reference-pattern replay - PASSED locally
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow or preservation defects; ENTRY-018 does not establish autonomous polarity initiation or resource acquisition.
- Blockers: exact-head Linux CI and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry017-post-fission-daughter-asymmetry-audit - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY017-POST-FISSION-DAUGHTER-ASYMMETRY-AUDIT-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY017-POST-FISSION-DAUGHTER-ASYMMETRY-AUDIT-LOCAL`
- Closed: `2026-09-01T08:16:59-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head Linux workflow `33505752560` passed on result head `d12c5ba1f6b23c412296e7c4354e1c929c0fd92b`, and the uploaded artifact plus independently downloaded ZIP both hash to `sha256:904dbbf76b470a0c16183828ca5aefbab0e9a641adea3e99f245145b059eba86`. The accepted D-088 physical fission replay reaches separation at step `2326`, with a 198-site mother and 78/122-site daughters. Lawful daughter-local geometry, strain, structural-material, maturation, membrane, and curvature fields are physically nonuniform; partition closure and rotation pass. Because daughter topology is not directly compatible with the accepted 24-site polarity ring, the classification remains `M2_POST_FISSION_ASYMMETRY_PRESENT_TOPOLOGY_MAPPING_UNRESOLVED`. The mother was already nonuniform before fission, so the result supports preservation/partition of existing life-history asymmetry rather than de novo birth-generated asymmetry; no Phase-3 individuality claim is made.
- Changed areas: additive observer-only ENTRY-017 assay/example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry017/`, scoped workflow, and append-only governance only; no scientific runtime, M1, polarity, actuator, traction, metabolism, uptake, resource, restart, or PR #44 change.
- Validation:
  - Exact starting head, accepted physical fission authority, replay, daughter snapshots, material partition closure, topology boundary, forbidden-information audit, rotation, and local preservation checks - PASSED
  - Exact-head Linux workflow `33505752560` on `d12c5ba1f6b23c412296e7c4354e1c929c0fd92b` - PASSED
  - Artifact digest and independent downloaded-archive hash - PASSED
  - Architect review - PENDING
- Remaining risks: Architect acceptance remains outstanding; no polarity state, topology remapping, or resource encounter was implemented or tested.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry018-native-material-ring-polarity-transfer - COMPLETE

- Outcome ID: OUT-DCDEV021-ENTRY018-NATIVE-MATERIAL-RING-POLARITY-TRANSFER-R1
- Supersedes outcome: OUT-DCDEV021-ENTRY018-NATIVE-MATERIAL-RING-POLARITY-TRANSFER-LOCAL
- Closed: 2026-09-01T09:45:00-04:00
- Acceptance: MET
- Summary: Exact-head ENTRY-018 validation confirms that the accepted continuous M2071-derived polarity equations transfer natively to the exact 198-site mother and 78/122-site daughter material rings. Normalized physical arclength, edge-centered conservative finite volumes, regular-grid equivalence, weighted active+inactive conservation, all accepted homogeneous equilibria, native spatial instability, and nonhomogeneous Polar/Traveling reference replays pass without resampling or biological parameter changes. The classification is `M2_NATIVE_MATERIAL_RING_POLARITY_TRANSFER_FEASIBLE`.
- Changed areas: additive numerical example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry018/`, scoped workflow, and append-only governance only; no accepted scientific runtime source changed and no polarity initialization, actuator coupling, or resource assay was implemented.
- Validation:
  - Exact ENTRY-017 starting head and ancestry - PASSED
  - Regular-grid equivalence, weighted conservation, native stability, and reference-pattern replay - PASSED
  - Exact-head Linux CI `33512289462` on `b1993be67d85ed8288c5d580c25269f4d6bf3d67` - PASSED
  - Independent downloaded artifact ZIP digest `sha256:9c48bab34a62df3c90dbeddd5892c71680a9a27ee16dacf1a6c881afd9423325` - PASSED
  - M1, downstream, governance, and restart-boundary checks - PASSED
- Remaining risks: Architect acceptance remains outstanding; ENTRY-018 does not establish autonomous polarity initiation or resource acquisition.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry020-autonomous-polarity-embodied-locomotion - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY020-AUTONOMOUS-POLARITY-EMBODIED-LOCOMOTION-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-01T14:00:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The isolated ENTRY-020 live composition starts from the exact accepted D-088/ENTRY-019 physical state and an exactly homogeneous Polar state. Conservative native-ring transport creates a deterministic polarity seed, the closed loop remains active and locomotory, and the exact patterned Polar positive control translates. The autonomous closed loop is indistinguishable from the same-mean uniform control for the preregistered spatial-leverage criteria, so the local classification is `M2_AUTONOMOUS_POLARITY_MECHANICAL_AMPLITUDE_INSUFFICIENT`.
- Changed areas: additive assay-only ENTRY-020 example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry020/`, scoped workflow, and append-only governance only; no accepted scientific runtime or PR #44 change. The assay uses the exact `ReactionParams::conservative_v3` production authority and reports actuator-ledger A-to-W closure separately from broader D-088 geometry bookkeeping.
- Validation:
  - Exact ENTRY-019 starting head and local release assay - PASSED
  - Autonomous seed, active locomotion, positive-control translation, same-mean comparison, physical rotation equivariance, and actuator A-to-W closure - PASSED locally; an extra circular-index replay was not used for qualification
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow, governance, or preservation defects; ENTRY-020 does not establish autonomous embodied locomotion or resource acquisition.
- Blockers: exact-head Linux CI and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry020-autonomous-polarity-embodied-locomotion - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY020-AUTONOMOUS-POLARITY-EMBODIED-LOCOMOTION-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY020-AUTONOMOUS-POLARITY-EMBODIED-LOCOMOTION-LOCAL`
- Closed: `2026-09-01T19:05:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head Linux workflow `33568834067` passed on result head `e0916d65a7a3b9eb8b4dad08fcb2585d32b076f0`; the uploaded artifact digest and independently downloaded ZIP SHA-256 are `sha256:bbca560f5b50ed3b6daa0b081ff496dcbb93c981911050b687c28489569f5962`. The autonomous closed loop begins homogeneous, develops a deterministic polarity seed, remains locomotory, and preserves actuator A-to-W closure, but is indistinguishable from the same-mean uniform control; the exact patterned Polar positive control translates. The bounded classification is `M2_AUTONOMOUS_POLARITY_MECHANICAL_AMPLITUDE_INSUFFICIENT`.
- Changed areas: additive assay-only ENTRY-020 example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry020/`, scoped workflow, and append-only governance only; no accepted scientific runtime or PR #44 change.
- Validation:
  - Exact ENTRY-019 authority and local release assay - PASSED
  - Autonomous seed, active locomotion, positive-control translation, same-mean comparison, physical rotation, actuator A-to-W closure, historical preservation, M1/D-087, downstream, and governance - PASSED
  - Exact-head Linux workflow `33568834067` on `e0916d65a7a3b9eb8b4dad08fcb2585d32b076f0` - PASSED
  - Independent artifact ZIP digest `sha256:bbca560f5b50ed3b6daa0b081ff496dcbb93c981911050b687c28489569f5962` - PASSED
- Remaining risks: Architect review is pending; ENTRY-020 does not establish autonomous embodied locomotion or autonomous resource acquisition.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260901-dcdev021-m2-entry021-conservative-polarity-fission-inheritance-amplification - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY021-CONSERVATIVE-POLARITY-FISSION-INHERITANCE-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-01T20:30:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The local ENTRY-021 observer audit replays the accepted D-088 physical fission path without forcing division, reaches step `2326`, and produces the accepted 198-site mother with 78/122-site daughters. Parent-local `u`, `v`, and `F` control-volume amounts are assigned to exact contiguous inherited parent-edge slices; synthesized closing edges receive zero inherited amount because they have no parent predecessor. Daughter partition closure passes, same-total controls are preserved, and both daughters show immediate post-fission spatial amplitude above numerical-noise homogeneous controls, while the 3,000-step trajectories decay toward homogeneity. The local classification is `M2_CONSERVATIVE_POLARITY_FISSION_INHERITANCE_AND_AMPLIFICATION_QUALIFIED`, pending exact-head Linux validation and Architect review.
- Changed areas: additive observer-only ENTRY-021 assay/example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry021/`, scoped workflow, and append-only governance only; no accepted scientific runtime, fission biology, actuator, resource, production polarity, M1, or PR #44 change.
- Validation:
  - Exact ENTRY-020 authority, physical fission replay, topology, local correspondence, polarity closure, same-total controls, rotation/index checks, and local release execution - PASSED
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow, governance, or preservation defects; ENTRY-021 does not establish production polarity, autonomous locomotion, or autonomous resource acquisition.
- Blockers: exact-head Linux validation and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry022-post-fission-transient-locomotion - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY022-POST-FISSION-TRANSIENT-LOCOMOTION-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-02T12:08:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The local ENTRY-022 assay replays the accepted unforced D-088 fission at step `2326`, producing the accepted 198-site mother and 78/122-site daughters. Birth states retain a zero-pool closing edge and are explicitly non-actuatable. One actuator-off, growth-off, additional-fission-off eligibility step makes every daughter `u+v` strictly positive, after which exact `u/(u+v)` inherited-spatial, same-mean uniform, and motor-off arms run for the remaining 2,999 steps. Both active arms remain energetic and locomotory, but neither inherited spatial arm exceeds its same-mean and motor-off controls on the preregistered displacement criteria; the local classification is `M2_POST_FISSION_TRANSIENT_MOTOR_CONTRAST_MECHANICALLY_INSUFFICIENT`.
- Changed areas: additive ENTRY-022 assay/example, compact evidence under `digital-protocell/experiments/generated/dcdev021m2entry022/`, scoped workflow, and append-only governance only; no accepted scientific runtime, fission biology, polarity production, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact physical fission replay, zero-pool boundary, strict post-fission eligibility, six daughter arms, A-to-W closure, rotation/index checks, and local release execution - PASSED
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow, preservation, or scope defects; ENTRY-022 does not establish autonomous resource acquisition or production polarity integration.
- Blockers: exact-head Linux validation and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry022-post-fission-transient-locomotion - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY022-POST-FISSION-TRANSIENT-LOCOMOTION-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY022-POST-FISSION-TRANSIENT-LOCOMOTION-LOCAL`
- Closed: `2026-09-02T12:18:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Exact-head Linux workflow `33653530285` passed on result head `b0daff3d89e5465e53f2fb4bee7ba81dd31ead69`. The independent artifact ZIP digest is `sha256:bcc2bd555c6804d3cbd522124e51a8d6da07fc4b7aa0f4d2c5885e35471d5ce5`. The sealed result remains `M2_POST_FISSION_TRANSIENT_MOTOR_CONTRAST_MECHANICALLY_INSUFFICIENT`: inherited spatial motor arms are valid and active after strict eligibility, but do not outperform same-mean and motor-off controls under the preregistered spatial-leverage comparison.
- Changed areas: additive ENTRY-022 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, fission biology, production polarity, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact authority, fission replay, zero-pool boundary, eligibility, daughter arms, closure, rotation/index, historical classifications, production/D-087, downstream, governance, and local execution - PASSED
  - Exact-head Linux workflow `33653530285` - PASSED
  - Independent artifact ZIP digest - PASSED
  - Architect review - PENDING
- Remaining risks: ENTRY-022 does not establish autonomous resource acquisition, production polarity integration, or autonomous polarity initiation beyond the already accepted upstream boundary.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry023-daughter-mechanical-transfer-attribution - PARTIAL

- Outcome ID: OUT-DCDEV021-ENTRY023-DAUGHTER-MECHANICAL-TRANSFER-ATTRIBUTION-LOCAL
- Supersedes outcome: none
- Closed: 2026-09-02T15:30:00-04:00
- Acceptance: `PARTIAL`
- Summary: Local ENTRY-023 replay completed from exact accepted ENTRY-022 head `48b313db45761552e27a34f77b7aff9b0e688f95`. The analytical reference Polar field has mechanical leverage on daughter A but not B; the frozen inherited field and its valid K1-only component show the same A-only leverage, while the residual-only decomposition is correctly marked invalid because it leaves the admissible motor range. Exact live ENTRY-022 replay is retained for decay attribution; the local bounded classification is `M2_DAUGHTER_MECHANICAL_TRANSFER_ATTRIBUTION_UNRESOLVED` because the inherited leverage differs by daughter and therefore does not support a single stronger preregistered classification.
- Changed areas: additive ENTRY-023 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, fission/remesh biology, polarity production, actuator, traction, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact fission replay, daughter remesh-compatible field transport, reference/inherited controls, live replay, modal reconstruction, A-to-W closure, rotation/index checks, and local release execution - PASSED
  - Exact-head Linux CI - PENDING
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow, governance, or preservation defects; ENTRY-023 does not establish autonomous polarity initiation, embodied locomotion, or autonomous resource acquisition.
- Blockers: exact-head Linux validation and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry023-daughter-mechanical-transfer-attribution - COMPLETE

- Outcome ID: OUT-DCDEV021-ENTRY023-DAUGHTER-MECHANICAL-TRANSFER-ATTRIBUTION-R2
- Supersedes outcome: OUT-DCDEV021-ENTRY023-DAUGHTER-MECHANICAL-TRANSFER-ATTRIBUTION-LOCAL
- Closed: 2026-09-02T15:45:00-04:00
- Acceptance: MET
- Summary: Exact-head Linux validation passed for ENTRY-023 on result head `ec60390c1c05301a949c70a851d4da7744b0b5cd`. The analytical reference Polar field, frozen inherited field, and valid K1-only reconstruction show daughter-A-only mechanical leverage; daughter B does not pass the preregistered spatial-versus-control comparison. Residual-only decomposition is invalid because it leaves the admissible motor range. The bounded classification is `M2_DAUGHTER_MECHANICAL_TRANSFER_ATTRIBUTION_UNRESOLVED`.
- Changed areas: additive ENTRY-023 observer assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, fission/remesh biology, production polarity, actuator, traction, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact authority, daughter replay, modal/phase attribution, live/frozen comparison, closure, rotation/index, historical classifications, production/D-087, downstream, governance, and local release execution - PASSED
  - Exact-head Linux workflow `33670749143` on `ec60390c1c05301a949c70a851d4da7744b0b5cd` - PASSED
  - Independent artifact ZIP digest `sha256:2157fbd0db692be8e11c55894b3b5ba9958ec1cb4267754593443602347633f0` - PASSED
  - Architect review - PENDING
- Remaining risks: ENTRY-023 does not establish autonomous embodied locomotion or autonomous resource acquisition; the unresolved mixed daughter attribution remains bounded to this audit.
- Blockers: Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry025-live-antagonistic-inherited-locomotion - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY025-LIVE-ANTAGONISTIC-INHERITED-LOCOMOTION-R3`
- Supersedes outcome: `OUT-DCDEV021-ENTRY025-LIVE-ANTAGONISTIC-INHERITED-LOCOMOTION-R2`
- Closed: 2026-09-02T18:30:00-04:00
- Acceptance: `MET`
- Summary: Exact-head Linux workflow `33688853406` passed on result head `c506837301c5cb4ed98b519b9cfc79f1033597fa`; the independently downloaded artifact ZIP digest is `sha256:3751de543f4355c4dff858f6ca5855bac592260aa2b0a94b3ed219098ee193c2`. The sealed ENTRY-024 artifact remains unchanged and the ENTRY-024 metadata correction is represented only by the new ENTRY-025 correction evidence. Live complementary inherited-polarity arms remain valid and active but do not clear the preregistered spatial-leverage comparison against same-mean and motor-off controls. The final bounded classification is `M2_LIVE_ANTAGONISTIC_INHERITED_POLARITY_COMPOSITION_INSUFFICIENT`.
- Changed areas: additive ENTRY-025 assay/example, compact evidence, scoped workflow, append-only governance, and the ENTRY-024 metadata correction only; no accepted scientific runtime, fission, remesh, polarity production, actuator, traction, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact authority, ENTRY-024 correction, fission replay, strict eligibility, live causal order, direct-live parity, full material and energetic closure, rotation/index invariance, historical classifications, production/D-087, downstream, governance, and local release execution - PASSED
  - Exact-head Linux workflow `33688853406` - PASSED
  - Independent artifact ZIP digest - PASSED
  - Architect review - PENDING
- Remaining risks: ENTRY-025 does not establish autonomous embodied locomotion or autonomous resource acquisition; no successor work is authorized.
- Blockers: Architect review only.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry026-post-fission-development-polarity-maintenance - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY026-POST-FISSION-DEVELOPMENT-POLARITY-MAINTENANCE-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-02T19:25:00-04:00`
- Acceptance: `PARTIAL`
- Summary: Local ENTRY-026 replay from exact accepted ENTRY-025 head `b6eb9f1a58155220f6dff49bd5c79152b4964ffc` shows growth-ON maintains inherited nonhomogeneous polarity for both daughters relative to matched growth-OFF decay; same-total homogeneous controls remain homogeneous and no de-novo reseed event occurs. Normal growth is physically active, remeshing remains conservative, second-fission eligibility is observed at step 25 and no second fission is executed. Provisional classification: `M2_POST_FISSION_DEVELOPMENT_MAINTAINS_INHERITED_POLARITY_WITHOUT_DE_NOVO_RESEED`.
- Changed areas: additive ENTRY-026 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, production polarity, actuator, resource, M1, restart, or PR #44 change.
- Validation:
  - Local release assay and generated evidence - PASSED
  - Exact-head Linux validation - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: ENTRY-026 does not establish autonomous embodied locomotion or autonomous resource acquisition; Architect review is pending.
- Blockers: exact-head Linux validation and Architect review; do not start successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry026-post-fission-development-polarity-maintenance - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY026-POST-FISSION-DEVELOPMENT-POLARITY-MAINTENANCE-R2`
- Supersedes outcome: `OUT-DCDEV021-ENTRY026-POST-FISSION-DEVELOPMENT-POLARITY-MAINTENANCE-LOCAL`
- Closed: `2026-09-02T19:35:00-04:00`
- Acceptance: `MET`
- Summary: Exact-head Linux workflow `33695346399` passed on result head `f842a1dc1a160015b870ce72df68e1c0f2739a94`; the independently downloaded artifact ZIP digest is `sha256:3401fb9bb664fd18ca6ff2f3c3a52a105b720033fad2e8437e08a5aa7378be75`. Growth-ON maintains inherited nonhomogeneous daughter polarity relative to matched growth-OFF decay for both daughters; same-total homogeneous controls remain homogeneous and no de-novo reseed occurs. Second-fission eligibility is observed at step 25 and no second fission is executed. The bounded classification is `M2_POST_FISSION_DEVELOPMENT_MAINTAINS_INHERITED_POLARITY_WITHOUT_DE_NOVO_RESEED`.
- Changed areas: additive ENTRY-026 assay/example, compact evidence, scoped workflow, and append-only governance only; no accepted scientific runtime, production polarity, actuator, resource, M1, restart, or PR #44 change.
- Validation:
  - Exact authority, bounded scope, ENTRY-026 assay, evidence, rotation/index, historical classifications, production/D-087, downstream, and governance - PASSED
  - Exact-head Linux workflow `33695346399` on `f842a1dc1a160015b870ce72df68e1c0f2739a94` - PASSED
  - Independent artifact ZIP digest - PASSED
  - Architect review - NOT RUN
- Remaining risks: ENTRY-026 establishes continued-development polarity maintenance without de-novo reseeding in this bounded assay; it does not establish autonomous embodied locomotion or autonomous resource acquisition. Architect acceptance remains pending.
- Blockers: Architect review only; no successor work is authorized.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry026r1-population-fission-gate-requalification - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY026R1-POPULATION-FISSION-GATE-LOCAL`
- Supersedes outcome: `OUT-DCDEV021-ENTRY026-POST-FISSION-DEVELOPMENT-POLARITY-MAINTENANCE-R2`
- Closed: `2026-09-02T21:02:39-04:00`
- Acceptance: `PARTIAL`
- Summary: R1 starts exactly at `04e8b7f030118842b0ad2d8428b6f937fa9aa6c7`, replays the accepted physical first fission, and evaluates the actual population gate `grown_enough && population_tick % 25 == 0 && try_local_fission(...).is_some()` without executing a second fission. Daughter A has birth mass `212.6439090102348` and threshold `287.069277163817`; daughter B has birth mass `335.7739408039904` and threshold `453.294820085386`. The sealed ENTRY-026 step-25 raw pinch candidates are false eligibility because both are below 1.35x. Growth-ON reaches true eligibility at relative step 250 for A and 225 for B. Both daughters maintain inherited polarity relative to matched growth-OFF decay; same-total homogeneous controls show no de-novo reseed.
- Changed areas: additive R1 assay/example, compact evidence, scoped workflow, and append-only governance only; sealed ENTRY-026 evidence and accepted scientific runtime remain unchanged.
- Validation:
  - Exact authority, corrected population gate, material closure, rotation/index, and local release assay - PASSED
  - Exact-head Linux CI - PENDING
  - Independent artifact digest - PENDING
  - Architect acceptance - PENDING
- Remaining risks: exact-head validation and governance checks are still pending; this remains a bounded post-fission maintenance result and does not establish autonomous embodied locomotion or resource acquisition.
- Blockers: exact-head Linux validation and Architect review; no successor work.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry026r1-population-fission-gate-requalification - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY026R1-POPULATION-FISSION-GATE-REQUALIFICATION-FINAL`
- Supersedes outcome: `OUT-DCDEV021-ENTRY026R1-POPULATION-FISSION-GATE-LOCAL`
- Closed: `2026-09-02T21:46:00-04:00`
- Acceptance: `MET`
- Summary: R1 starts exactly at sealed ENTRY-026 head `04e8b7f030118842b0ad2d8428b6f937fa9aa6c7`, replays the accepted physical first fission, and evaluates the true daughter-specific `1.35x birth_mass` plus population cadence plus physical pinch boundary without executing a second fission. Daughter A reaches true lifecycle eligibility at relative step `250`; daughter B at `225`. Growth-ON maintains inherited polarity relative to growth-OFF for both daughters; homogeneous controls remain homogeneous and no de-novo reseed occurs. The bounded classification is `M2_POST_FISSION_DEVELOPMENT_MAINTAINS_INHERITED_POLARITY_WITHOUT_DE_NOVO_RESEED`.
- Changed areas: additive R1 assay/example, compact evidence, scoped workflow, append-only governance, and no sealed ENTRY-026 rewrite; no accepted scientific runtime source changed and PR #44 remains untouched.
- Validation:
  - Exact authority, corrected population gate, material closure, rotation/index, preservation matrix, production/D-087, downstream, governance, and local release assay - PASSED
  - Exact-head Linux workflow `33704174275` on `b8889a725784fa8d4339b9a8633977589aa5e801` - PASSED
  - Independent artifact ZIP digest `sha256:6957842c387ffb26068971cc2c09236d4f435edd3c88f963ee08547206a44a3d` - PASSED
  - Notion append-only handoff and readback - PASSED
  - Architect acceptance - PENDING
- Remaining risks: R1 establishes bounded post-fission developmental maintenance and corrects the population-gate audit boundary; it does not establish autonomous embodied locomotion or resource acquisition.
- Blockers: Architect review only; no successor work is authorized.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry027-growth-on-interfission-locomotion - PARTIAL

- Outcome ID: `OUT-DCDEV021-ENTRY027-GROWTH-ON-INTERFISSION-LOCOMOTION-LOCAL`
- Supersedes outcome: none
- Closed: 2026-09-02T22:18:00-04:00
- Acceptance: `PARTIAL`
- Summary: ENTRY-027 starts exactly at accepted ENTRY-026-R1 head `cff3340c46801aa9bbe52ea2b5e830b124ee0852`, replays the unforced physical first fission, performs the required zero-pool actuator-off step, and runs daughter-specific growth-on spatial, same-mean, and motor-off arms without executing a second fission. Daughter A does not clear the spatial-leverage comparison; Daughter B does. The local bounded classification is `M2_GROWTH_ON_INTERFISSION_LOCOMOTION_DAUGHTER_DEPENDENT`.
- Changed areas: additive ENTRY-027 assay/example, compact evidence, scoped workflow, append-only governance, and no accepted scientific runtime source; PR #44 remains untouched.
- Validation:
  - Local release assay, lifecycle boundary, material/energy closure, rotation/index, and evidence generation - PASSED
  - Exact-head Linux CI attempt `33708022659` - FAILED only because the workflow compared boolean rotation/index evidence with string `PASS`; assay and evidence validation reached that assertion, and the validator is corrected without changing the assay.
  - Independent artifact digest - NOT RUN
  - Notion readback - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: Remote exact-head preservation and Architect review remain pending; no successor work is authorized.
- Blockers: none
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry027-growth-on-interfission-locomotion - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY027-GROWTH-ON-INTERFISSION-LOCOMOTION-FINAL`
- Supersedes outcome: `OUT-DCDEV021-ENTRY027-GROWTH-ON-INTERFISSION-LOCOMOTION-LOCAL`
- Closed: 2026-09-02T22:22:00-04:00
- Acceptance: `MET`
- Summary: ENTRY-027 validates the exact unforced first physical fission and the growth-on inter-fission locomotion composition using only the assay-local `v/(u+v)` interface. Daughter A has no spatial leverage against its same-mean control; Daughter B does, yielding `M2_GROWTH_ON_INTERFISSION_LOCOMOTION_DAUGHTER_DEPENDENT`. The assay observes the true lifecycle boundary without executing a second fission.
- Changed areas: additive ENTRY-027 assay/example, compact evidence, scoped workflow, append-only governance, and no accepted scientific runtime source; PR #44 remains untouched.
- Validation:
  - Exact authority, zero-pool eligibility, physical fission, daughter controls, lifecycle, closure, rotation/index, preservation, D-087, downstream, governance, and local release assay - PASSED
  - Exact-head Linux workflow `33708162577` on `e0ec069125a64e1217723620167f95c6274906b4` - PASSED
  - Independently downloaded artifact ZIP digest `sha256:a523e92552b866546bfb7cbb7aac214be4256187b3770b751e0474e0d2914cee` - PASSED
  - Notion append-only handoff and readback - PASSED
  - Architect acceptance - PENDING
- Remaining risks: This is a bounded daughter-dependent inter-fission locomotion result; autonomous resource acquisition and environment-dependent evolution remain unestablished. No successor work is authorized.
- Blockers: Architect review only.
- Follow-up directive: none

## D-20260902-dcdev021-m2-entry027-growth-on-interfission-locomotion - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY027-GROWTH-ON-INTERFISSION-LOCOMOTION-FINAL-CORRECTION`
- Supersedes outcome: `OUT-DCDEV021-ENTRY027-GROWTH-ON-INTERFISSION-LOCOMOTION-FINAL`
- Closed: 2026-09-02T22:26:00-04:00
- Acceptance: `MET`
- Summary: Final exact-head validation of ENTRY-027 remains `M2_GROWTH_ON_INTERFISSION_LOCOMOTION_DAUGHTER_DEPENDENT`; the result head is `2dd93d1a644eb50089b7e7be43d17e9c07becc31`.
- Changed areas: additive ENTRY-027 assay/example, compact evidence, scoped workflow, append-only governance, and no accepted scientific runtime source; PR #44 remains untouched.
- Validation:
  - Exact authority, unforced physical first fission, daughter controls, lifecycle boundary without second fission, closure, rotation/index, preservation, D-087, downstream, governance, and local release assay - PASSED
  - Exact-head Linux workflow `33708546230` on `2dd93d1a644eb50089b7e7be43d17e9c07becc31` - PASSED
  - Independently downloaded artifact ZIP digest `sha256:728083dca165ba71dbba1ad43e3f4132728faa26bb2bd3514ef521619f676e0c` - PASSED
  - Notion append-only handoff and readback - PASSED
  - Architect acceptance - PENDING
- Remaining risks: This is a bounded daughter-dependent inter-fission locomotion result; autonomous resource acquisition and environment-dependent evolution remain unestablished. No successor work is authorized.
- Blockers: Architect review only.
- Follow-up directive: none

## D-20260903-dcdev021-m2-entry028-balanced-separated-resource-ecology - COMPLETE

- Outcome ID: `OUT-DCDEV021-ENTRY028-BALANCED-SEPARATED-RESOURCE-ECOLOGY-FINAL`
- Supersedes outcome: `OUT-DCDEV021-ENTRY028-BALANCED-SEPARATED-RESOURCE-ECOLOGY-LOCAL`
- Closed: `2026-09-03T11:05:00-04:00`
- Acceptance: `MET`
- Summary: ENTRY-028 validates the exact accepted ENTRY-027 unforced first fission and the preregistered balanced separated-resource ecology across both daughters and all four bearings. Initial contact is zero; physical contact occurs on eligible arms, but all arms deliver zero N/F and no spatial arm exceeds both same-mean and motor-off controls. The bounded classification is `M2_SEPARATED_RESOURCE_CONTACT_WITHOUT_ACQUISITION_ADVANTAGE`.
- Changed areas: additive ENTRY-028 assay/example, compact evidence, scoped workflow, append-only governance, and the explicitly authorized ENTRY-027 presentation-header correction only; no accepted scientific runtime source or PR #44 modification.
- Validation:
  - Local release build, full 24-arm assay, lifecycle boundary, closure, rotation/index construction, historical preservation, production/D-087, downstream, and governance - PASSED
  - Exact-head Linux workflow `33745961902` on assay head `7dbe9b85d28b9cb65842f98c2fa41405f37f2445` - PASSED
  - Independently downloaded artifact ZIP digest `sha256:81d5de75785c60b0e811a3240033e1262af790ecdeefb7dfcf454901094249f0` - PASSED
  - Architect review - PENDING
- Remaining risks: ENTRY-028 does not establish autonomous resource acquisition or environment-dependent evolution; no successor work is authorized pending Architect review.
- Blockers: Architect review only.
- Follow-up directive: none

## D-20260903-dcdev021-m2-closure-finite-world-autonomous-ecology - PARTIAL

- Outcome ID: `OUT-DCDEV021-M2-CLOSURE-FINITE-WORLD-AUTONOMOUS-ECOLOGY-LOCAL`
- Supersedes outcome: none
- Closed: `2026-09-03T12:20:00-04:00`
- Acceptance: `PARTIAL`
- Summary: The additive closure runtime implements a shared finite world with conserved, order-independent allocation, reuses accepted inherited polarity/fission/metabolism authorities, validates positive finite transfer plus same-step metabolism in a direct contact sanity arm, and runs existing-motor and conditional local-protrusion ecology arms. Both bounded separated campaigns retain active mechanics and unforced first fission but obtain no finite resource; no descendant fission or resource-causal reproduction is established. The result classification is `M2_CURRENT_SENSORIMOTOR_ROUTE_ECOLOGICALLY_INSUFFICIENT`.
- Changed areas: additive finite-world module, assay-only contractility/stick-slip extra-force helpers, closure example/evidence, scoped workflow, and append-only governance; accepted M1 biology and PR #44 remain untouched.
- Validation:
  - Exact ENTRY-028 authority and starting head, finite-world conservation/order tests, contact transfer/metabolism sanity, six-arm closure campaign, A-to-W closure, historical preservation, D-087, downstream, and local governance validation - PASSED
  - Exact-head Linux workflow - NOT RUN
  - Artifact upload and independent digest - NOT RUN
  - Notion append-only handoff/readback - NOT RUN
  - Architect review - PENDING
- Remaining risks: remote validation may identify workflow or preservation defects; autonomous finite-resource acquisition, shared ecological contention, resource-causal reproduction, and evolution re-entry remain unestablished.
- Blockers: exact-head Linux CI, artifact verification, Notion readback, and Architect review.
- Follow-up directive: none

## D-20260903-dcdev021-m2-closure-finite-world-autonomous-ecology - COMPLETE

- Outcome ID: `OUT-DCDEV021-M2-CLOSURE-FINITE-WORLD-AUTONOMOUS-ECOLOGY-FINAL`
- Supersedes outcome: `OUT-DCDEV021-M2-CLOSURE-FINITE-WORLD-AUTONOMOUS-ECOLOGY-LOCAL`
- Closed: `2026-09-03T12:42:00-04:00`
- Acceptance: `MET`
- Summary: The bounded closure execution implements and validates a reusable shared finite world with conserved, order-independent allocation; reuses accepted polarity, fission, and frozen-metabolism authorities; proves positive direct-contact N/F transfer with exact world debit and same-step metabolism; and tests the existing motor before the conditional assay-only local protrusion fallback. Both integrated separated campaigns remain actively motile and execute unforced first fission, but neither route reaches a finite resource in the fixed ecology. The final classification is `M2_CURRENT_SENSORIMOTOR_ROUTE_ECOLOGICALLY_INSUFFICIENT`.
- Changed areas: additive finite-world module, assay-only extra-force helpers, closure assay/evidence/workflow, and append-only governance; no frozen M1 biology, resource-law source, fission source, or PR #44 modification.
- Validation:
  - Local release assay, finite-world unit tests, evidence validation, governance validation, material/energy closure, historical preservation, D-087, and downstream tests - PASSED
  - Exact-head Linux workflow `33779402628` on `328a68a4d18757b537f69c3d9e33cbd5099d6cd7` - PASSED
  - Independently downloaded GitHub artifact ZIP digest `sha256:3578659875dd9982f9b9a76f26664f42fcc8fb7042533eb6f303a6c28d14d01f` - PASSED
  - Notion append-only handoff and readback - PASSED
  - PR #44 state verification (OPEN/DRAFT/UNMERGED/UNTOUCHED) - PASSED
  - Architect acceptance - PENDING
- Remaining risks: The current sensorimotor route does not establish autonomous finite-resource acquisition, actual shared-resource contention, resource-causal reproduction, a mutable heritable causal phenotype, or evolution re-entry. Dense Atlas traces were not generated by this compact run.
- Blockers: Architect review only; no successor work is authorized.
- Follow-up directive: none

## D-20260903-dcdev021-m2-closure001-r1-polarity-clutch-migration - PARTIAL

- Outcome ID: `OUT-DCDEV021-M2-CLOSURE001-R1-POLARITY-CLUTCH-MIGRATION`
- Supersedes outcome: `OUT-DCDEV021-M2-CLOSURE-FINITE-WORLD-AUTONOMOUS-ECOLOGY-FINAL`
- Closed: 2026-09-03T16:05:00-04:00
- Acceptance: `PARTIAL`
- Summary: R1 corrects the finite-world boundary to preserve nonfeeding transport while reserving positive N/F influx to finite inventory, rechecks post-mechanics world views, solves exact polygon surface gaps, closes lineage motion segments across fission, separates passive traction from A funding, and tests the parameter-free local polarity clutch.
- Changed areas: additive R1 workflow/evidence, finite-world transport helper, assay-local corrected force composition and traction clutch, closure assay and append-only governance; sealed CLOSURE-001 evidence remains unchanged.
- Validation:
  - Corrected finite-world transport boundary, exact polygon clearance, ENTRY-027 parity, lineage accounting, passive traction semantics, local clutch, conservation, and compact evidence - PASSED
  - Exact-head Linux CI - NOT RUN
  - Artifact upload and independent digest - NOT RUN
  - Notion append-only handoff/readback - NOT RUN
  - Architect review - NOT RUN
- Remaining risks: remote validation may identify workflow or preservation defects; no second-generation fission or resource-causal reproduction is established in the bounded run.
- Blockers: exact-head Linux CI, artifact verification, Notion readback, and Architect review.
- Follow-up directive: none
