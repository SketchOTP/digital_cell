# CURRENT.md

## Active directive
- ID: D-20260718-d033-activated-membrane-intermediate
- Project directive: D-033
- Goal: Two-stage activated membrane intermediate (P+A→X+W, X→S, X→P) on frozen v8 interfacial architecture
- Status: runner_wired
- Acceptance: One D033_* conclusion with Gate evidence; Stage E recovered only if all applicable gates pass
- Touched files: experiment-runner/d033.rs, main.rs
- Next action: Run full pipeline Gate5 (200k) when ready

## Repo facts needed now
- Gate1 unit tests: d033_tests 9/9 (pre-existing)
- Runner gates 2–4 smoke: PASS after compile fix
- Gate5 not run (200k isolated renewal)

## Last validation
- Command: cargo check -p experiment-runner
- Result: PASS

## Open blockers
- Mimir baseline path degraded on host mapping (task begin OK)
