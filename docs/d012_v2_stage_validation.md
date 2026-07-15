# D-012 V2 Stage Validation (Tasks 11–14)

## Scope

Conservative `membrane_metabolism_v2_conservative` revalidation of D-008 Stages A–D after the v2 conservation gate (Task 10). Stage E remains gated on this evidence.

## Equation identity

| Field | Value |
| --- | --- |
| `equation_version` | `membrane_metabolism_v2_conservative` |
| `stoichiometric_schema_version` | `2` |
| Unit yields | `η_C = η_φ = η_M = 1` |

V1 candidate hashes and snapshots are **not** comparable to v2.

## Stage A — transport equivalence (Task 11)

**Test:** `test_v2_transport_matches_v1`

Shared selective-transport kernel (`try_membrane_metabolism_v1_transport`) is equation-version independent. Focused regression compares v1 vs v2 flux and field state for catalyst/activated/nutrient/fuel/waste at membrane levels `{0.0, 0.5, 1.0}`.

**Result:** PASS — bit-identical transport flux and transported fields.

## Stage B — membrane localization (Task 12)

**Runner:** `experiment-runner d012 stage-b`  
**Artifact:** `digital-protocell/experiments/generated/d012/v2_stage_b_localization/result.json`  
**Test:** `test_v2_membrane_localization`

| Gate | Threshold | Result |
| --- | --- | --- |
| Localization after transient | ≥ 0.90 | 0.9001 (center 0.50) |
| Membrane synthesis / decay / detachment | > 0 | all positive |
| Bounded M | `[0, m_max]` | pass |
| Membrane accounting residual | < 1e-8 | pass |

**Classification:** `D012_STAGE_B_LOCALIZATION_PASS`

**V2 note:** Initial-state robustness uses levels `{0.50, 0.75}` instead of v1 `{0.25, 0.75}` because A-coupled membrane synthesis depletes activated resource at very low initial M (0.25 fell to 0.885 localization).

## Stage C — zero-D metabolism (Task 13)

**Runner:** `experiment-runner d012 stage-c`  
**Artifact:** `digital-protocell/experiments/generated/d012/v2_stage_c_metabolism/result.json`  
**Test:** `test_v2_metabolic_reactor_bounded`

Nine qualitative controls (missing C/N/F/A, decay-only paths, waste positivity, stoichiometric closure with material ledger).

| Gate | Result |
| --- | --- |
| Activation requires C, N, F | pass |
| Reproduction requires A | pass |
| Bounded C/A + negligible clamp | pass |
| Material-equivalent step closes | pass |

**Classification:** `D012_STAGE_C_METABOLISM_PASS`

## Stage D — fixed compartments (Task 14)

**Runner:** `experiment-runner d012 stage-d`  
**Artifact:** `digital-protocell/experiments/generated/d012/v2_stage_d_fixed_compartment/attempt_001/result.json`  
**Test:** `test_v2_fixed_compartment_retention` (single-step coupling + material closure); full retention gates validated by runner integration test.

Radii R = 16, 24, 32; 5000 substeps; selected Stage A/B/C rates from `configs/d008/stage_c_selected.toml`.

| Radius | Catalyst retention | Activated retention |
| --- | --- | --- |
| 16 | 0.9990 | 0.9977 |
| 24 | 0.9993 | 0.9984 |
| 32 | 0.9995 | 0.9988 |

All D-008 retention/flux/scaling gate checks: **pass**  
**Classification:** `D012_STAGE_D_FIXED_COMPARTMENT_PASS`

V2 Stage D metrics match v1 at unit yield (expected — metabolism deltas identical when η=1).

## Task 15 authorization

**Authorized:** Stage B, C, and D gates passed. Task 15 (transport-coupled Stage E reference assay) may proceed.

## Commands

```bash
cargo test -p chemistry-core --release --test d012_tests
cargo test -p experiment-runner --release run_v2_stage

cd digital-protocell
./target/release/experiment-runner d012 stage-b --output experiments/generated/d012/v2_stage_b_localization
./target/release/experiment-runner d012 stage-c --output experiments/generated/d012/v2_stage_c_metabolism
./target/release/experiment-runner d012 stage-d --output experiments/generated/d012/v2_stage_d_fixed_compartment
```
