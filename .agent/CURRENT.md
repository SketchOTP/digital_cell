# CURRENT.md

## Active directive
- ID: D-20260721-d065-canonical-resource-sufficiency-topology-necessity
- Project directive: D-065
- Goal: Canonicalize accepted net resource sufficiency; audit topology necessity (shadow-only)
- Status: done
- Acceptance: met — `D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED`
- Touched files: d065_analysis/tests, experiment-runner/d065, main.rs, lib.rs, docs/d065_*, experiments/generated/d065, .agent/*
- Next action: audit frozen activation law / accepted activation capacity under smooth geometry; next_execution_started=false

## Repo facts needed now
- Primary: Route A `D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED`
- Smooth χ R22≈1.82 (≥1.05) — connected membrane not required for resource capacity
- D-064 identity reproduced: legacy static≈13.55, proxy≈0.19, gross χ≈19.03; A≈0.40; S 368→227; W ceiling @~1076
- Control C (unlimited local N/F) restores A≈1.81; perfect exterior N/F does not
- W_DESTINATION_OVERCOMMIT secondary execution defect
- Frozen k_T: 1.4346157818803311
- Artifacts: experiments/generated/d065 → /mnt/storage1tb/.../d065

## Last validation
- Command: cargo test -p chemistry-core --test d065_tests; D065_MAX_ACCEPTED=1200 D065_SKIP_LATE_GATES=1 pipeline
- Result: 12/12 PASS; primary ResourceDeliverySufficientActivationLimited

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
