# CURRENT.md

## Active directive
- ID: D-20260814-final-harness-acceptance-closure
- Project directive: DC-SR-003R2
- Goal: Close execution authorization, chronology, causal generation, mechanism evidence, selective-pressure, multi-founder, and remote-CI defects without changing certified science
- Status: implementation complete locally; remote sync and CI pending
- Acceptance: fail-closed execution, phased chronology, parent-causal generation intervals, mechanism-aware evidence, explicit treatment pressure, multi-founder placement, passing local tests, successful GitHub CI on exact PR head
- Touched files: evolution-harness, evolution-harness docs, CI workflow, .agent records
- Next action: sync scoped repair, push PR #1, wait for exact-head GitHub CI; do not start SR-004

## Repo facts needed now
- Preserved entry commit: 956b054a9b37f675a8b84ae0624db98853956d37
- Preserved tag: DC-SR-003-modular-evolution-harness; do not move or delete
- Branch: strategy/modular-evolution-harness
- Certified chemistry-core and experiment-runner source must remain unchanged
- D-094 translation remains non-executable and no D-094 execution is authorized

## Last validation
- Command: cargo +1.89.0-x86_64-pc-windows-msvc test -p evolution-harness
- Result: 27 passed, 0 failed; rustfmt installed and applied

## Open blockers
- Mimir V2 lifecycle tools are unavailable in the current tool surface; do not claim Mimir context/evidence/close-out
- Atlas has no Rust/Cargo on PATH; local sanctioned Windows toolchain is used for verification
- Draft PR: #1 https://github.com/SketchOTP/digital_cell/pull/1; new head pending push
