# CURRENT.md

## Active directive
- ID: D-20260814-0930-harness-verification-repair
- Project directive: DC-SR-003R
- Goal: Repair executable evolution-harness semantics and close architect verification defects without changing certified science
- Status: implementation complete; remote verification and draft PR pending
- Acceptance: real time, executable supported ecology, N-seed campaigns, treatment/neutral analysis, closed-fail qualification, provenance, passing tests, pushed branch, real draft PR
- Touched files: evolution-harness, evolution-harness docs, strategy disposition, .agent records
- Next action: synchronize remote, run remote scope checks, commit/push, open draft PR

## Repo facts needed now
- Preserved entry commit: 956b054a9b37f675a8b84ae0624db98853956d37
- Preserved tag: DC-SR-003-modular-evolution-harness; do not move or delete
- Branch: strategy/modular-evolution-harness
- Certified chemistry-core and experiment-runner source must remain unchanged
- D-094 translation remains non-executable and no D-094 execution is authorized

## Last validation
- Command: cargo +1.89.0-x86_64-pc-windows-msvc test -p evolution-harness
- Result: 9 passed, 0 failed; rustfmt installed and applied

## Open blockers
- Mimir V2 lifecycle tools are unavailable in the current tool surface; do not claim Mimir context/evidence/close-out
- Atlas has no Rust/Cargo on PATH; local sanctioned Windows toolchain is used for verification
- No draft PR exists until the pushed repaired commit is created
