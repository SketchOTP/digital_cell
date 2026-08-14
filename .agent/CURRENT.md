# CURRENT.md

## Active directive
- ID: D-20260814-gate7-single-generation-fitness
- Project directive: DC-SR-004C
- Goal: Measure frozen D-096 inherited physiology to single-generation physical fission consequence under preregistered H/B/Neutral forcing
- Status: coder_complete_architect_pending; Gate 8 blocked
- Acceptance: 16 paired seeds per candidate/environment, frozen 4000-step/80.0-time horizon, real first-fission-or-death stop, observer-only endpoints, artifacts/docs/scoped CI, no fitness or selection control
- Touched files: new Gate 7 assay/artifacts/docs/workflow and minimal harness support only if required; certified Phase 1 biology/equations unchanged
- Next action: architect independently inspect PR #5 and exact remote head/CI; do not begin Gate 8

## Repo facts needed now
- Preserved entry commit: 956b054a9b37f675a8b84ae0624db98853956d37
- Preserved tag: DC-SR-003-modular-evolution-harness; do not move or delete
- Branch: strategy/d096-gate7-single-generation-fitness
- PR #4 remains open, draft, unmerged; base strategy/d098-processing-repair
- Certified Phase 1 chemistry-core biology/equations remain unchanged; this PR does modify bounded post-Phase-1 D-096 heredity and fission-partition code
- D-094 translation remains non-executable and no D-094 execution is authorized

## Last validation
- Command: cargo test -p evolution-harness; cargo test -p chemistry-core --test d096_tests; cargo run -p evolution-harness --example d096_gate6_assay
- Result: local fmt, 14 D-096 tests, 46 evolution-harness tests, Gate 6 continuity regression, and optimized Gate 7 assay passed; 144/144 cells horizon-stopped with no fission/death

## Open blockers
- Mimir V2 lifecycle tools are unavailable in the current tool surface; do not claim Mimir context/evidence/close-out
- Atlas has no Rust/Cargo on PATH; local sanctioned Windows toolchain is used for verification
- Draft PR: #5 https://github.com/SketchOTP/digital_cell/pull/5 open, draft, unmerged; base strategy/d096-gate6-heredity-continuity
- Remote CI: run 31839637762 passed at head a1410fd4117418d2f6a4c32a768b38dcba3bf0ea
