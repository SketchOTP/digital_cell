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
