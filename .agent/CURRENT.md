# CURRENT.md

## Active directive
- ID: D-20260715-d012-conservative-stoichiometric-closure
- Project directive: D-012
- Goal: Complete D-011 long-horizon closure and establish conservative seven-field metabolic stoichiometry
- Status: design approved; specification drafting
- Acceptance: D-011 definitively classified; v1 formally audited; nonconservative v1 cannot advance; conservative v2 passes conservation before staged D-008 revalidation
- Touched files: .agent/DIRECTIVES.md, .agent/CURRENT.md, docs/superpowers/specs/2026-07-15-d012-conservative-stoichiometric-closure-design.md
- Next action: write and self-review approved design specification

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Stage E fail tag preserved: D-008-stage-e-balance-fail
- D-011 failure tags preserved: D-011-transport-coupled-balance-fail and D-011-transport-coupled-balance-fail-corrected
- Corrected latest attempt: experiments/generated/d011/attempt_017/result.json
- Operative D-011 status: D011_LONG_HORIZON_CONFIRMATION_INCOMPLETE
- attempt_017 is quick mode at 5,000 accepted substeps; existing horizon radii are 18/24/30, not D-012's 18/22/26
- Stage E remains: D008_NO_JOINT_FIXED_POINT
- Stages F-G blocked; D-009 blocked
- Production verdict: REQUIRES REMEDIATION

## Last validation
- Command: git tag/commit verification; inspect attempt_017 and current D-011 runner/docs
- Result: governed commits/tags present; repository clean before Serena created and cleanup removed an incidental onboarding file

## Open blockers
- Mimir MCP start recall failed with `fetch failed`; retry before final outcome recording
- Serena has no active Rust language support; use cocoindex, targeted reads, Cargo, and IDE diagnostics
