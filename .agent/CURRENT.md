# CURRENT.md

## Active directive
- ID: D-20260715-d012-conservative-stoichiometric-closure
- Project directive: D-012
- Goal: Complete D-011 long-horizon closure and establish conservative seven-field metabolic stoichiometry
- Status: plan committed; beginning Task 1 preservation
- Acceptance: D-011 definitively classified or superseded by invalid stoichiometry; v1 formally audited; nonconservative v1 cannot advance; conservative v2 passes conservation before staged D-008 revalidation
- Touched files: docs/superpowers/specs/2026-07-15-d012-conservative-stoichiometric-closure-design.md, docs/superpowers/plans/2026-07-15-d012-conservative-stoichiometric-closure.md
- Next action: Task 1 preservation manifest, D-011 status normalization, tag D-011-long-horizon-incomplete

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Design commits: a04e098 then 7aa9c63 (strengthened ordering)
- Plan commit: latest with D-012 implementation plan
- Stage E fail tag preserved: D-008-stage-e-balance-fail
- D-011 failure tags preserved: D-011-transport-coupled-balance-fail and D-011-transport-coupled-balance-fail-corrected
- Corrected latest attempt: experiments/generated/d011/attempt_017/result.json (quick 5k)
- Operative D-011 status: D011_LONG_HORIZON_CONFIRMATION_INCOMPLETE until Task 1 tags it
- Scientific branch: stoichiometric audit before expensive D-011; skip exhaustive D-011 if nonconservative
- Stage E remains: D008_NO_JOINT_FIXED_POINT
- Stages F-G blocked; D-009 blocked
- Production verdict: REQUIRES REMEDIATION

## Last validation
- Command: design/plan commits; governed tag/commit verification
- Result: tags and attempt artifacts present; plan ready for Task 1

## Open blockers
- Mimir MCP start recall failed with `fetch failed`; retry before final outcome recording
- Serena has no active Rust language support; use cocoindex, targeted reads, Cargo, and IDE diagnostics
