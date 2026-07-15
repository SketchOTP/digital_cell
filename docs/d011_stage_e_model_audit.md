# D-011 Stage E Model Audit

## Preserved failure

| Item | Value |
| --- | --- |
| Result commit | `2db93f6a06ea1ae6c577585d9f49fc98899f027e` |
| Failure tag | `D-008-stage-e-balance-fail` |
| Source commit (documented) | `dfadb10e9ad113cdac903355ead18094d680df50` |
| Artifact | `digital-protocell/experiments/generated/d008/stage_e_balance/attempt_003/result.json` |
| Artifact SHA-256 | `3ff7f8ac0a623cce0563d9853d56d35a59c285f83c173037e11db484b71c7aee` |
| Scientific conclusion | `D008_NO_JOINT_FIXED_POINT` |

Exact calibrated rates (machine precision from artifact):

```text
k_membrane              = 0.23697878259991778
k_d008_activation       = 0.024
k_d008_reproduction     = 0.032
k_d008_structure        = 0.6788558775098147
k_d008_activated_decay  = 0.005
k_d008_catalyst_turnover= 0.002
k_structure_decay       = 0.025
```

## What Stage E actually computed

Implementation: `prescribed_balance_point` in `d008_analysis.rs`, driven by `run_stage_e`.

For each prescribed radius and interior seed:

| Field | Evolved? | How measured |
| --- | --- | --- |
| φ | **Fixed** | Analytic circular tanh profile; never advanced |
| C | **Prescribed** | Uniform interior seed; instantaneous rate only |
| N | **Prescribed** | Uniform interior seed; no transport |
| F | **Prescribed** | Uniform interior seed; no transport |
| W | **Prescribed** | Uniform interior seed; no transport |
| A | **Prescribed** (swept) | Interior A grid 0.05–0.50; not dynamic |
| M | **Prescribed** | `I(φ) × membrane_scale`; not dynamic |

Balance evaluation:

```text
instantaneous reaction rates on fixed fields
no accepted simulation substeps
no membrane-modified transport
no reservoir exchange
no time averaging
no quasi-steady relaxation
```

Sequential calibration screened `k_membrane`, `k_activation`, `k_reproduction`, `k_structure` at 0.8×/1.0×/1.2× against that static signature.

## Classification

```text
STATIC_FIELD_BALANCE
```

Not `FULLY_TRANSPORT_COUPLED`. Hypothesis A from D-011 (under-coupled prescribed-radius model) is therefore the primary repair path before any stoichiometry change.

## Implications for D-011

1. Do not treat `D008_NO_JOINT_FIXED_POINT` as confirmed network failure until the transport-coupled constrained-radius assay completes.
2. Do not duplicate Stage E’s static evaluator as the D-011 assay.
3. Replay the exact Stage E rates under constrained-radius dynamics with quasi-steady windows.
4. Only after coupled assay + bounded joint solver may a network-level failure be confirmed.
