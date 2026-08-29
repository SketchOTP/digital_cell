# DC-DEV-020-R9-R5-R1

This is an observer-only requalification from `f1fad5c65859f3a314102d3ec5a0751822a2f5ea` on PR #44. R9-R5 is recorded as `REPLAN / NOT ACCEPTED`; its broad `OUTSIDE_CHARGE_LIQUIDITY_FACTORIZATION` classification is retired as authority. The accepted R9-R4 storage-causality result is preserved.

## Gate 0

The R9-R5 V20 Gate 7 failure was a runner-root defect. The R9-R5 example passed `digital-protocell` as `repo_root`, while `gate7_linux_runtime` resolves build paths relative to the parent repository root. The bounded repair uses the same parent-root convention as the accepted R9-R3-R1 runner. The V20 packaged-runtime control then reproduced 8/8 with the frozen control metrics: `R_m=0.8398695202805284` and `STORE_OFF R_m=1.0180981834599838`.

## Counterfactual definition

`LIQUID_RESERVE_PRETHROTTLE_UB` is diagnostic-only. It evaluates the unchanged structural-M and membrane-L demand equations with `A+R` as activation-equivalent availability before low A suppresses candidate demand. A funds the baseline demand calculated at actual A; diagnostic R funds only the incremental demand unlocked by the additional availability. No unrelated chemistry receives the substitution, no new material is created, and production defaults remain `FULL`.

The focused regression constructs low A with positive R and verifies that the normal path remains A-limited, the shadow requests greater frozen M/L demand, diagnostic R is used for both M and L, and ConservativeV2 closure remains within tolerance.

## Result

Across 5,000 accepted steps, diagnostic R availability was `183945.65216387564`, of which `10.986875147245845` was used: `9.741776616086431` for M and `1.2450985311594456` for L. Full `R_m` was `0.8398695202805284`; the valid liquidity counterfactual reached `0.9994257946133822` but did not restore D-087 certification. Its D-087 gates were `[true,false,true,true,true,true,true,true]`.

The narrow result is:

```text
DCDEV020R9R5R1_RESERVE_LIQUIDITY_CONTRIBUTORY_NOT_SUFFICIENT
```

This does not authorize a reserve repair, salvage/recycling, or DC-DEV-021. Dense ledgers are local/external evidence and their hashes are recorded in `experiments/generated/dcdev020r9r5r1/ledger_manifest.json`.
