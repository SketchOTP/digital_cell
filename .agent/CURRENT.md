# CURRENT.md

## Active directive
- ID: D-20260714-d010-execute-d008-stage-gates
- Project directive: D-010
- Goal: Implement and scientifically close the approved D-008 seven-field membrane-metabolism protocell
- Status: Stages 0, A, and B passed; preparing Stage C activated metabolism
- Acceptance: Execute D-008 gates in order and stop at first scientific failure or produce governed D008_MEMBRANE_METABOLIC_CLOSURE_PASS evidence
- Touched files: seven-field scaffold, selective transport, membrane dynamics/accounting, D-008 runner/config/tests, Stage 0/A/B reports and manifest
- Next action: implement zero-dimensional activation, activated decay, catalyst reproduction, waste production, and stoichiometric accounting

## Repo facts needed now
- Current branch: d008-membrane-metabolic-closure
- Approved D-008 design commit: f72be6bec425cd0613a25ed2c67cc1b1f9f647da
- D-009 blocked outcome commit/tag: 6c45c568a311e7d4388861321bf0ced6700523c9 / D-009-blocked-d008-not-passed
- D-008 Stage 0 source tip: afa9b37a68a715a70c4ec31feb2725c6394afbb9
- D-008 Stage 0 conclusion: D008_STAGE_0_SCHEMA_PASS
- D-008 Stage A source tip: 9236be1bf1208bcc7b709c93b8855dd48e18c55e
- D-008 Stage A conclusion: D008_STAGE_A_TRANSPORT_PASS
- D-008 Stage B source tip: 31fd993123e16ca64474f3d1176f3a8d74933eb2
- D-008 Stage B conclusion: D008_STAGE_B_LOCALIZATION_PASS
- Serena configured but reports Active languages: []; Rust symbol validation unavailable
- Phase 1 remains PHASE1_SELF_MAINTENANCE_PARTIAL

## Last validation
- Command: cargo test -p chemistry-core --release --test d008_tests; cargo test -p experiment-runner --release d008::tests; governed Stage B run
- Result: 36 + 5 passed; Stage B 5/5 clean runs, minimum localization 0.90035, manifest 3ec24bc002e96b37843c4006c7f44f017f8a45bf73305ae9013442c945c22eda

## Open blockers
- None
