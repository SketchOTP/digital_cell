# CURRENT.md

## Active directive
- ID: D-20260715-d012-conservative-stoichiometric-closure
- Project directive: D-012
- Goal: Finish Tasks 15–18 governed Stage E without altering network or reporting a premature conclusion
- Status: reference terminated `INVALID_ARTIFACT`; solver/yield/robustness not started; no Stage E scientific conclusion
- Acceptance: three-window quasi-steady + four balances + restoring neighbors + throughput + accounting + robustness before any pass
- Touched files: d012_stage_e.rs, main.rs, experiments/.../v2_stage_e_reference/
- Next action: repair rejected-step termination/checkpoint/accounting artifact capture before any new governed reference

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Canonical output: `digital-protocell/experiments/generated/d012/v2_stage_e_reference`
- Diagnostic snapshot preserved under `.../diagnostic_snapshot/` (5k NOT_CONVERGED; all g_structure negative)
- Reference source: 15f9f21; binary b044ac4083838e9ea6e21c32e093f2025f25b10876b26f9b67c917995e28e77d
- INVALID_ARTIFACT cause: rejected substep ended accepted progress, but attempted chunks appended zero-motion windows; clean=false, checkpoints and activation ledger absent
- Stage B limitation: M=0.25 failed; validated M∈{0.50,0.75}
- Do not claim pass/no-solution until full protocol completes

## Last validation
- Command: inspect reference result/ledger, verify hashes and required evidence
- Result: INVALID_ARTIFACT; manifest hash d4a7bf88b244761e054af4c12cb17afc39e6d9cad2e83f3451f412df1a744a7a

## Open blockers
- Reference runner must record true termination, atomic checkpoints, and activation-potential accounting before rerun
- Mimir MCP unavailable
