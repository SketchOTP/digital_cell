# CURRENT.md

## Active directive
- ID: D-20260715-d012-conservative-stoichiometric-closure
- Project directive: D-012
- Goal: Finish Tasks 15–18 governed Stage E without altering network or reporting a premature conclusion
- Status: governed 200k Stage E restarted after orchestration fix (`15f9f21`); diagnostic 5k preserved; no Stage E conclusion yet
- Acceptance: three-window quasi-steady + four balances + restoring neighbors + throughput + accounting + robustness before any pass
- Touched files: d012_stage_e.rs, main.rs, experiments/.../v2_stage_e_reference/
- Next action: await 200k reference completion; then conditional solver/yield/robust; Task 19

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Full Stage E PID: see `pgrep -af 'd012 stage-e --max-steps 200000'`
- Log: `/tmp/d012_stage_e_full.log`
- Canonical output: `digital-protocell/experiments/generated/d012/v2_stage_e_reference`
- Diagnostic snapshot preserved under `.../diagnostic_snapshot/` (5k NOT_CONVERGED; all g_structure negative)
- Calibration/estimate now use diagnostic horizons; classification radii retain 200k/10k windows
- Stage B limitation: M=0.25 failed; validated M∈{0.50,0.75}
- Do not claim pass/no-solution until full protocol completes

## Last validation
- Command: orchestration fix build + d012 Stage E gate unit test
- Result: PASS; full Stage E restart launched

## Open blockers
- Full 200k Stage E still running
- Mimir MCP unavailable
