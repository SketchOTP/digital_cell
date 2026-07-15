# CURRENT.md

## Active directive
- ID: D-20260715-d012-conservative-stoichiometric-closure
- Project directive: D-012
- Goal: Finish Tasks 15–18 governed Stage E without altering network or reporting a premature conclusion
- Status: full 200k Stage E reference running; diagnostic 5k complete (NOT_CONVERGED, no restoring crossing); no Stage E conclusion yet
- Acceptance: three-window quasi-steady + four balances + restoring neighbors + throughput + accounting + robustness before any pass
- Touched files: d012_stage_e.rs, d012_analysis.rs, experiments/generated/d012/v2_stage_e_*
- Next action: await 200k reference completion; then solver only if valid sensitivity on converged state; then conditional yield; then robustness; Task 19

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Full Stage E PID: check `pgrep -af 'd012 stage-e'`
- Artifacts currently under repo-root `experiments/generated/d012/` (relative path from digital-protocell cwd), not only `digital-protocell/experiments/generated/d012/`
- Diagnostic reference max_steps=5000: R18/22/26 NOT_CONVERGED; g_structure all negative
- Stage B limitation preserved: M=0.25 failed; validated M∈{0.50,0.75}
- Do not declare NO_JOINT_FIXED_POINT until 200k horizons + solver domain + yield branch rules are satisfied
- V1 superseded; v2 only

## Last validation
- Command: d012_tests 50/50; diagnostic stage-e reference
- Result: infrastructure OK; Stage E scientifically pending full horizons

## Open blockers
- Full 200k Stage E still running (hours-scale)
- Mimir MCP unavailable
