# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-28T11:55:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260828-dcdev020m1replan002r1-maturation-production-candidate001`
- External directive ID: `DC-DEV-020-M1-REPLAN-002-R1-MATURATION-COUPLED-PRODUCTION-CANDIDATE-QUALIFICATION-001`
- Objective: `Qualify one versioned MaturationCoupledV4 production candidate against the accepted REPLAN-002 shadow while preserving V1/V2/V3, D-087, material closure, and the unchanged production default.`
- Current status: `VALIDATING`
- Acceptance: `Run the real V4 candidate, preserve historical V1/V2/V3 behavior, reproduce the immutable shadow, qualify fed homeostasis/recovery/starvation decline, validate lifecycle continuity and serialization, run D-087 V2/V3/V4, store dense ledgers on Atlas, and obtain exact-head Linux CI.`
- Current phase: `REPLAN-002 is Architect accepted as M1_MATURATION_COUPLED_LOAD_BEARING_FEASIBILITY_CONFIRMED at 4becff4fff7d096c70468b759ace09f747c4eb56. R1 implementation reproduces the shadow and passes fed homeostasis, recovery, starvation decline, closure, damage, serialization, deterministic replay, and fission lifecycle tests. Fresh D-087 V2/V3 pass 8/8; V4 returns 6/8 with dual-retention and starvation gates failing, so the candidate is not qualified. Exact-head Linux workflow 33186408566 passed at 3c5d0c143ea1031a987fed792269760dfdab48d2; artifact digest is sha256:e663a75f49dabf5adbe8bd86700b25ae628a36ac2b6c21a4bcb9f32b49702d45. M1 remains NOT ESTABLISHED and M2 remains unauthorized.`
- Expected or actual touched areas: `Versioned V4 lifecycle state and dispatch, R1 candidate harness/tests/workflow/evidence/docs, and append-only governance only; no V1/V2/V3 rewrite, coefficient/tolerance change, production-default switch, parameter search, reserve/recycling/salvage, M2, behavior, evolution, or DC-DEV-021 behavior; all unrelated dirty work is preserved`
- Immediate next action: `Stop for Architect review of the exact-head R1 result. Do not repair D-087, tune V4, begin a physical-death follow-up, switch production, or begin M2.`

## Temporary task-relevant facts

- The exact scientific base for R2 is `1e242f28152797b512e25cd56c7b718e45d6ca97`; the prior R1 head is `876012f8888b074285c55167613471a59d4be25d`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `8d6fe59397cabfa47bc1d8103acd68f544acc190`.
- DC-DEV-007 is architect-accepted; its active contact chain remains preserved and is exercised by the DC-DEV-008 preservation workflow.
- DC-DEV-010 / PR #19 is closed, unmerged negative evidence and must not be imported.
- R5 is accepted as `DCDEV020R5_ACCEPTED`, `DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT`, and `ENDPOINT_SURROGATE_NOT_LOCAL_REQUIREMENT` at `d215cfc00ce70517e25fa7c3b51b13d85d9ce521`; R6 work is isolated on `strategy/dc-dev-020r6-nf-power-law-source`.
- R6 is accepted as `DCDEV020R6_ACCEPTED_NEGATIVE` and `DCDEV020R6_FINITE_FEED_RESTORATION_FAILURE` at `f01b716d9051c9f0114f3c5c0d1b123e2df037cf`; `NF_POWER_LAW_RESTORATION_ROUTE_CLOSED` does not close NF information sufficiency.
- DC-DEV-015 starts exactly at `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` from `strategy/dc-dev-013-resource-contact-feeding`; DC-DEV-014/PR #23 is closed, unmerged negative evidence and is not imported.
- R2-R2 is architect-accepted at `1622b664a4a37b8a0ac4ea51fbc97ca71f9d853c` with exact-head CI `32615395736`; ordinary starvation reaches actual topology rupture and remains non-intact after both refeeding shadows.
- R3 is architect-accepted at `17226fb7484eb50079c1c30ce9fd8039b3f23c60` with exact-head CI `32617847392` and artifact digest `sha256:dde942ef96c37ee4e0c9882abacd89202dd4eaf245c81b53d4f0efc039fe5700`; classification `M1_ORDINARY_DECAY_CANDIDATE_QUALIFIED`.
- R4 is architect-accepted at exact head `68d1c88ec1b915a4bee86efe24e985222b529d5a`, CI `32648997395`, artifact digest `sha256:ea8e2161e0889da26a613fd95b6ffa0aa1b7bdb7e0dde23a9fa9aea26d559305`; coupled source is qualified only for the bounded 480-step candidate and remains unselected.
- R5 is preregistered with `FINITE_SPATIAL_BACKING_RESERVOIR_V1`, fixed boundary concentration `2.063914918930895`, finite N/F inventory `243.14924801053778` each, and zero replenishment. Current authoritative evidence is archived under `\\atlas\\ATLAS\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\` with manifests and hashes; the retired RPI5 endpoint is historical only, and compact CI-required artifacts remain in Git.
- R6 uses the exact R5 depleted entry but executes the packaged full-runtime order `transport -> reactions -> mechanics -> remesh -> try_local_rebond`; the candidate remains ConservativeV3/reserve OFF and is not selected.
- R6 is architect-accepted as a valid invalidation at `adea13fafa1f2a85e521a44b5d77249820d107bd` with exact-head CI `32673647585`; full-runtime closure failed, and R6-R1 is authorized only to attribute that failure without repair.
- R6-R1-R1-R1 workflow closure is pushed at `a3205d6c99cbc845d406043580f99577fa6a73e6`; exact-head CI `32686612525` passed after moving unchanged V2/V3 D-087 producers before the verifier. Artifact digest: `sha256:d7f91883db1155308d576f2c6f09d2eb2c92bf60dcd617493275ade036f7d181`. Architect acceptance remains pending.
- R6-R2 local conservation repair is blocked pending Architect disposition: mechanics-only and remesh-only candidate strict deltas are zero; integrated 8,000-step candidate closure is `4.263256414560601e-14`; actual candidate D-087 is `6/8` with Gate 1 and Gate 2 failing, while unchanged ConservativeV3 under ConservativeV2 remains `8/8`.
- R6-R2-R1 local semantics audit is complete: unchanged V3 chemistry under the V2 material contract reproduces D-087 `8/8`; GeometryConservativeV3 reproduces `6/8`. Gate 1 first fails at catalyst `f_label` (`0.39221229068962093`), while amount-based catalyst `f_label` is `0.3277186407367453` and passes. All 15 candidate basin rows pass; Gate 2's non-basin starvation predicate is false (`final A=0.10147286122118783`), while replay, membrane/structural damage, rupture, and no-respawn pass. The matched chronology first diverges at step `1`; the geometry-frozen shadow has max difference `0`. Classification: `M1_GC_D087_MIXED_REGRESSION`.

## Last validation after adoption

- Command or check: `REPLAN-002-R1 exact-head Linux workflow 33186408566`
- Result: `PASSED`

## Risks

- Atlas has no Rust toolchain; local sanctioned Rust 1.89.0 is used with the Atlas worktree mounted through SSHFS and a local NTFS target directory.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.
- M0 is architect-accepted at `4895135deee7dbd782446dbfe25662181951afe0` with exact-head CI `32592048545`; the selected production identity is ConservativeV2/reserve OFF.
- M1-R0 local replay currently classifies the accepted M0 organism as `productive_allocation_or_replacement_limitation`: high finite N/F delivery reduces but does not reverse organized-material decline, and no-resource continuation reaches observer starvation collapse at accepted step 2057. This remains pending exact-head CI and architect review.
- M1-R1-R1 is architect-accepted at `7bb48874771144795a9559f7570f5ebc77e1004a` with exact-head workflows `32603849571` and `32603852368` passed; source capacity crosses the frozen 480-step boundary only after diagnostic decay neutralization, while the 4x coefficient remains provenance/necessity unresolved.
- M1-R2 is architect-accepted at `bc65098c3d26777aca2d1da5dab8cc118ecc6e19` as `M1_STARVATION_LAW_AUDIT_INCONCLUSIVE`; exact-head workflows `32611593080` and `32611594966` passed. Both no-resource arms lost observer viability without terminal chemistry-path failure within 20,000 continuation steps.
- M1-R2-R1 local replay from the exact accepted endpoint reproduces both R2 trajectory hashes and endpoint values. Both arms reach existing `activated_catalyst_collapse` during the 150,000-step extension (production 4x step `45422`, ordinary decay step `45831`), and both fail the exact 5,000-step no-reset finite N/F restoration challenge. Local classification is `M1_ORDINARY_DECAY_IRREVERSIBLE_FAILURE_ESTABLISHED`; architect acceptance and exact-head CI remain pending.
- M1-R2-R1 architect re-plan: observer collapse was reversible while topology remained intact. Its `M1_ORDINARY_DECAY_IRREVERSIBLE_FAILURE_ESTABLISHED` classification is retired.
- M1-R2-R2 reaches actual edge rupture in both arms (steps `124249` and `124717`) and preserves `closed_intact=false` through both refeeding shadows. Exact-head CI `32615105226` passed at `d5160ab`; architect review remains pending.
- R9-R2 correction: the actual D-087 certifier, not an R9 facsimile, is classification authority. ConservativeV2 remains an orthogonal physical/material/death contract; D-091 remains the biological equation lineage. The actual 3/8 result is a certification regression and must not be tuned around.
- R9-R3 result: H0 HistoricalV1/reserve OFF and V20 ConservativeV2/reserve OFF pass scientific Gates 0–6; H1 and V21 fail Gates 1–4 with nonzero reserve flows and zero rejects. The architect-accepted classification is `DCDEV020R9R3_RESERVE_PHYSIOLOGY_CERTIFICATION_GAP_CONFIRMED`.
- R9-R3-R1 diagnosis: the prior Gate-7 `bin_ok=false` was caused locally by extensionless source/destination paths on Windows (`digital-protocell-phase1` versus the actual `.exe` artifact); the runtime now derives the platform executable suffix and records build/copy/launch diagnostics without capturing unrelated environment secrets.
- Fresh R9-R3-R1 local evidence is under `digital-protocell/experiments/generated/dcdev020r3r1/`; H0 and V20 are 8/8 with unchanged replacement metrics, while H1 and V21 retain Gates 1–4 failure and the same reserve flows. Exact-head CI run `32436117572` passed all 24 stages at head `7c6b35c5b67a798c4ff32a61c3f6cf8e4fa8b5e5`; the R9-R3-R1 package is architect-accepted.
- R9-R1 exact artifacts are under `digital-protocell/experiments/generated/dcdev020r9r1/`; R8-R2 and R8-R4 dense frames remain externalized and their compact manifests record ConservativeV2 replay mode. R8-R4 is a bounded negative replay (`DCDEV020R8R4_SHARED_AFFINITY_NO_STABLE_HOMEOSTASIS`).

## Blockers

- R1 exact-head remote qualification passed workflow `33186408566`; its artifact is `dcdev020m1replan002r1-maturation-coupled-production-candidate-evidence` with digest `sha256:e663a75f49dabf5adbe8bd86700b25ae628a36ac2b6c21a4bcb9f32b49702d45`. V4 D-087 remains 6/8, so R1 is not qualified pending Architect review.
- No D-087 repair, V4 tuning, physical-death follow-up, production switch, recycling, salvage, controller, M2, behavior, evolution, REPLAN-003, or DC-DEV-021 work is authorized.

## Pending decisions

- The frozen DC-DEV-013 geometry, inventory, horizon, and thresholds must not be changed after protocol commit `fa8a689adff8cbc3b981038c4812ebdc0623116c`.
- DC-DEV-014, parameter repair, parameter screening, navigation, resource seeking, and evolution remain unauthorized.
- R7 is accepted at `7d5f772f0db67b8d754d27c1182c933533f750fd`; R8 uses frozen `p_NF=0.0003277429681759396` and distance limit `0.0024847602445668224`.
- R5 dense input is sealed at `4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585`; R7 dense input is sealed at `abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8`.
- R8 implementation source is `6e2b03a7551409086c1a38d6cf5f62827fb91929`; its dense pair/constraint ledger is sealed externally at `12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff`.
- R8-R1 implementation source is `d50037e53d041d8b06895553933c3b0a78c7a024`; its dense demand ledger is sealed externally at `f44e8f9fa441451ee40bcbfccac5f556131e4d26868868607e9507c29e7bcf90`.
- Accepted R8-R2 head for this audit is `9fdd292bbd13f62ef9c88d08e8d887f15326d242`; its dense ledger is sealed at `e932f6ab96e34516de98c97c2cae102553db9764383af3d61abf015743c3a376`.
- R8-R3 frozen catalyst half-life is `3465.7359027997263` accepted steps from `k_c_turn=0.01` and `dt=0.02`; governed horizon is `8000`; checkpoints are `433, 480, 866, 1733, 3466, 5199, 6931`.
- R8-R3 result is `DCDEV020R8R3_CATALYST_RESERVE_SOURCE_CONTEXT_DEPENDENT`: D016 has no marginal payback, while R6 pays back only at early checkpoints and not at later checkpoints; deferred arms remain alive and finite.
- R8-R4 tests `J_C,shared=k_c_prod*A*(1-q_c(C))` using existing `K_C=q_c=0.3`; architect acceptance closed that exact topology as a negative sustained-homeostasis result.
- R8-R5 tests the full conservative constant-C interval `0 <= C_hold <= A+C` with exact turnover replacement. No constant-C arm passes sustained homeostasis; all 200 deferred and 200 shared late states retain a nonnegative one-step allocation region, so classification is `DCDEV020R8R5_CATALYST_ALLOCATION_ENVELOPE_MIXED`.
- R8-R5-R1 corrects the local metric to `ΔE_NET = ΔE_repartition + ΔE_reaction` from the actual incoming state. All 400 states retain reversible nonnegative NET drift, all 400 forward-only states are negative, and every successful reversible optimum requires C→A recovery. The classification is `DCDEV020R8R5R1_RECYCLING_ONLY_LOCAL_CAPACITY`.
- The sealed R8-R5 dense ledger has no deferred checkpoint-hash field; R1 records deterministic replay hashes and explicitly reports that direct prior-ledger hash-field matching is unavailable.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
