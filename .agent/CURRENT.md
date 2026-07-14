# CURRENT.md

## Active directive
- ID: D-20260714-d010-execute-d008-stage-gates
- Project directive: D-010
- Goal: Implement and scientifically close the approved D-008 seven-field membrane-metabolism protocell
- Status: Stages 0 and A passed; preparing Stage B membrane localization
- Acceptance: Execute D-008 gates in order and stop at first scientific failure or produce governed D008_MEMBRANE_METABOLIC_CLOSURE_PASS evidence
- Touched files: seven-field scaffold, selective transport/accounting, D-008 runner/config/tests, Stage 0/A reports and manifest pointer
- Next action: implement fixed-field membrane production, diffusion, decay, detachment, and localization diagnostics

## Repo facts needed now
- Current branch: d008-membrane-metabolic-closure
- Approved D-008 design commit: f72be6bec425cd0613a25ed2c67cc1b1f9f647da
- D-009 blocked outcome commit/tag: 6c45c568a311e7d4388861321bf0ced6700523c9 / D-009-blocked-d008-not-passed
- D-008 Stage 0 source tip: afa9b37a68a715a70c4ec31feb2725c6394afbb9
- D-008 Stage 0 conclusion: D008_STAGE_0_SCHEMA_PASS
- D-008 Stage A source tip: 9236be1bf1208bcc7b709c93b8855dd48e18c55e
- D-008 Stage A conclusion: D008_STAGE_A_TRANSPORT_PASS
- Serena configured but reports Active languages: []; Rust symbol validation unavailable
- Phase 1 remains PHASE1_SELF_MAINTENANCE_PARTIAL

## Last validation
- Command: cargo test release d008 + d003-d007 + runner Stage A; governed planar sweep
- Result: 29 + 131 + 3 passed; five species meet selectivity and conservation gates; manifest 17a8bbd6ea0c1e1f79d8cbe1c28707460d39fb7af026ea41aeec4bb2cceea5f0

## Open blockers
- None
