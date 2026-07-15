# D-008 Stage E — Prescribed-Radius Balance

## Conclusion

`D008_NO_JOINT_FIXED_POINT`

Prescribed-radius balance analysis with sequential 0.8×/1.0×/1.2× calibration
screens for membrane production, activation, catalyst reproduction, and structure
production did not yield overlapping zero-flow regions for structure, catalyst,
membrane, and activated resource across the tested radius and interior-activated grid.

## Provenance

- Source commit: `dfadb10e9ad113cdac903355ead18094d680df50`
- Governed attempt: `attempt_003`
- Experiment-runner SHA-256: see `attempt_003/result.json`
- Equation version: `membrane_metabolism_v1`
- Scientific conclusion: `D008_NO_JOINT_FIXED_POINT`
- Stage classification: `D008_STAGE_E_BALANCE_FAIL`

## Method

Fixed circular φ geometry (width 2), Stage D interior seed (C=0.4, N/F=0.2, A swept
0.05–0.50, W=0.5), all eight reaction terms evaluated at old-state fields without
transport. Sequential calibration prioritized joint overlap, then minimum balance score.

## Gate

- `joint_zero_flow_overlap` (2D radius × interior-A grid): **false**

Stages F–G remain blocked. D-008 closure not achieved.

## Artifact

- `digital-protocell/experiments/generated/d008/stage_e_balance/attempt_003/result.json`

## D-011 follow-up (2026-07-14)

Stage E failure preserved: tag `D-008-stage-e-balance-fail` at commit `2db93f6`.
Model audit: `docs/d011_stage_e_model_audit.md` classifies Stage E as `STATIC_FIELD_BALANCE`.

D-011 replays the exact `attempt_003` rates under transport-coupled constrained-radius
dynamics (`D008StageMode::ConstrainedRadius`). See:

- `docs/d011_constrained_radius_assay.md`
- `docs/d011_candidate_report.md`
- `experiments/generated/d011/`

Stage E conclusion `D008_NO_JOINT_FIXED_POINT` stands until D-011 reports
`PASS_AFTER_D011`; otherwise D-011 may confirm `D011_TRANSPORT_COUPLED_NO_SOLUTION`.
