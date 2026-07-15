# CURRENT.md

## Active directive
- ID: D-20260714-d010-execute-d008-stage-gates
- Project directive: D-010
- Goal: Implement and scientifically close the approved D-008 seven-field membrane-metabolism protocell
- Status: Stages 0–C passed; preparing Stage D fixed-compartment coupling
- Acceptance: Execute D-008 gates in order and stop at first scientific failure or produce governed D008_MEMBRANE_METABOLIC_CLOSURE_PASS evidence
- Touched files: seven-field scaffold, transport, membrane dynamics, activated metabolism, D-008 runner/configs/tests, Stage 0–C reports
- Next action: couple selective transport to activation and catalyst reproduction on fixed circular compartments at R=16/24/32

## Repo facts needed now
- Current branch: d008-membrane-metabolic-closure
- Approved D-008 design commit: f72be6bec425cd0613a25ed2c67cc1b1f9f647da
- D-009 blocked outcome commit/tag: 6c45c568a311e7d4388861321bf0ced6700523c9 / D-009-blocked-d008-not-passed
- D-008 Stage 0 source tip: afa9b37a68a715a70c4ec31feb2725c6394afbb9
- D-008 Stage A source tip: 9236be1bf1208bcc7b709c93b8855dd48e18c55e
- D-008 Stage B source tip: 31fd993123e16ca64474f3d1176f3a8d74933eb2
- D-008 Stage C source tip: bdc2411e5fd0b8ff947835ce88bf5f02c5f1fb5e
- Stage conclusions: STAGE_0/A/B/C PASS
- Stage C rates are qualitative defaults pending Stage E calibration
- Serena configured but reports Active languages: []; Rust symbol validation unavailable
- Phase 1 remains PHASE1_SELF_MAINTENANCE_PARTIAL

## Last validation
- Command: cargo test -p chemistry-core --release --test d008_tests; cargo test -p experiment-runner --release d008::tests; governed Stage C
- Result: 45 + 8 passed; 9/9 Stage C controls pass; manifest 42068496da3a3248c5b99d5c6042647ee9bb74d8b11f671a6ddaa4f532c98a51

## Open blockers
- None
