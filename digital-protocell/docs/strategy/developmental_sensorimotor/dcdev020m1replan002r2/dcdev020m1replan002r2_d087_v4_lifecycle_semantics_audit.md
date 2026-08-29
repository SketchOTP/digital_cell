# DC-DEV-020-M1-REPLAN-002-R2

## D-087 V4 lifecycle-semantics audit

Status: `INVESTIGATE / NOT QUALIFIED`; Architect acceptance pending.

This observer-only audit starts at `98b1104165039359bdc609898e0d0371f9ce05c4` and does not repair V4, alter D-087 thresholds or durations, change historical contracts, or change production behavior. It compares the frozen D-087 result with observer-only lifecycle and viability interpretations on immutable copies of the same trajectories.

## Authority and state inventory

The failed D-087 gates are Gate 1 dual-retention and Gate 2 starvation. Gate 1 reads catalyst/material replacement and label fractions derived from observer tracers. Under V4, total structural material remains the physical authority (`M = M_young + M_mature`), while ordinary turnover is drawn only from mature material. Gate 2 currently uses the historical `!alive || A < 0.05` predicate. V4 uses the existing conservative observer authority (`observer_viable()` and `observer_death_reason()`); the historical `alive` latch is not the conservative physical-viability authority.

## Dual-retention audit

The raw exact-head V4 Gate-1 result is:

```text
R_m = 1.2150320498067337
f_m = 0.39365548976559395
R_c = 1.2293611628840693
f_c = 0.3257709240668205
dual-retention = false
```

The structural failure is specifically the frozen label-fraction predicate (`f_m <= exp(-1)`): `0.39365548976559395` is above the threshold, while `R_m` passes. Catalyst and membrane predicates remain unchanged and pass.

The lifecycle-consistent parallel observer starts with mature structural material labeled and zero young label, adds new structural production as unlabeled young material, transfers young to mature without creating label, and removes label only in proportion to label in the mature turnover-eligible pool:

```text
lifecycle R_m = 1.2150320498067337
lifecycle f_m = 0.24710833271945795
lifecycle f_pool = 0.18353830764832416
lifecycle label initial = 87.75070322587518
lifecycle label final = 21.68392996910598
lifecycle Gate-1 structural predicate = pass
```

The lifecycle observer changes no physical state. Its maximum decomposition residual is `7.105427357601002e-14`, and the legacy/lifecycle physical hash sequences are identical. The unchanged catalyst and membrane raw predicates also pass, so the full lifecycle-consistent Gate-1 comparison passes without changing the frozen thresholds.

## Starvation audit

The exact frozen Gate-2 continuation is 200 warm-up steps followed by 6000 no-resource steps. At final step 6200, V4 reports:

```text
alive = true
A = 0.09087892901751628
observer_viable = true
observer_death_reason = null
total M = 36.62413505096765
young M = 9.02102373815217
mature M = 27.603111312815482
ruptured edges = 0
closed_intact = true
physical_runtime_valid = true
```

The first `A < 0.05`, first observer-nonviable step, and first `alive == false` are all `never`. Therefore this audit does not support a stale-latch explanation for Gate 2: V4 does not reach the existing observer-defined starvation/collapse state within the frozen D-087 continuation. Gate 2 remains a genuine V4 certification failure under the unchanged protocol.

## Exact-head result

```text
V2 D-087 = 8/8
V3 D-087 = 8/8
V4 legacy D-087 = [true, false, false, true, true, true, true, true]
Gate 1 cause = CERTIFIER_SEMANTICS
Gate 2 cause = BIOLOGICAL
physical trajectory parity = true
classification = M1_V4_D087_MIXED_REGRESSION
```

R1 biological capabilities remain preserved: shadow parity, fed homeostasis (`organized delta ≈ +1.3323122170185968`), no-reset recovery, starvation structural decline, material closure, damage, remesh, fission, and serialization all remain passing. V2/V3 D-087 remain 8/8.

## Evidence and boundaries

Compact evidence is committed under `digital-protocell/experiments/generated/dcdev020m1replan002r2/`. Dense observer ledgers are stored on Atlas under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r2/`. This package is diagnostic only. No V4 biology, D-087 threshold, historical contract, production default, physical-death work, M2, reserve, recycling, salvage, or downstream work is authorized by this result.
