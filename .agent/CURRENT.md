# CURRENT.md

## Active directive
- ID: D-20260714-d011-transport-coupled-joint-balance
- Project directive: D-011
- Goal: Determine if Stage E failure is under-coupled model vs genuine network incompatibility
- Status: complete — transport-coupled quick assay found no joint balance; full 200k horizon not completed
- Acceptance: partial — corrected quick D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION with 50k supplementary replay; full governed horizons remain incomplete
- Touched files: d011_analysis, constraint_accounting, simulation constrained radius, experiment-runner/d011, docs/d011_*
- Next action: if continuing, run full 200k corrected four-rate protocol or write next network-repair directive from D-011 evidence

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- Stage E fail tag preserved: D-008-stage-e-balance-fail
- D-011 failure tag exists: D-011-transport-coupled-balance-fail
- Corrected latest attempt: experiments/generated/d011/attempt_017/result.json
- Conclusion: D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION (quick corrected protocol)
- Stage E remains: D008_NO_JOINT_FIXED_POINT
- Stages F-G blocked; D-009 blocked
- Production verdict: REQUIRES REMEDIATION

## Last validation
- Command: cargo test -p chemistry-core --release --test d011_tests; cargo test -p chemistry-core --release --test d008_tests; cargo run -p experiment-runner --release -- d011 run --quick
- Result: 21 PASS; 50 PASS; attempt_017 D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION

## Open blockers
- Full 200k D-011 horizon protocol did not complete in this session; attempt_017 is quick-mode evidence
