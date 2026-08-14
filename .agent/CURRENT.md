# CURRENT.md

## Active directive
- ID: D-20260814-1456-gate6-expansion
- Project directive: DC-SR-004B
- Goal: Complete the bounded D-096 Gate 6 heredity/mutation assay, evidence, documentation, and scoped CI package after midpoint clearance
- Status: in_progress; Gate 7 blocked
- Acceptance: runnable heredity/mutation assays, committed evidence artifacts and documentation, scoped CI, final local suite, pushed draft PR #4
- Touched files: evolution-harness Gate 6 tests/docs/artifacts/CI and .agent records; no certified science changes
- Next action: inspect existing Gate 6 coverage, define the smallest missing assay package, implement and verify

## Repo facts needed now
- Preserved entry commit: 956b054a9b37f675a8b84ae0624db98853956d37
- Preserved tag: DC-SR-003-modular-evolution-harness; do not move or delete
- Branch: strategy/d096-gate6-heredity-continuity
- PR #4 remains open, draft, unmerged; base strategy/d098-processing-repair
- Certified chemistry-core source and experiment-runner source remain unchanged by this Gate 6 expansion
- D-094 translation remains non-executable and no D-094 execution is authorized

## Last validation
- Command: cargo test -p evolution-harness; cargo test -p chemistry-core --test d096_tests; cargo run -p evolution-harness --example d096_gate6_assay
- Result: 45 harness tests passed, 14 D-096 tests passed, assay passed; scoped formatting check passed

## Open blockers
- Mimir V2 lifecycle tools are unavailable in the current tool surface; do not claim Mimir context/evidence/close-out
- Atlas has no Rust/Cargo on PATH; local sanctioned Windows toolchain is used for verification
- Draft PR: #4 https://github.com/SketchOTP/digital_cell/pull/4; midpoint-cleared head 00921d7651035142850af42ff4d1dc1eedb5b437
