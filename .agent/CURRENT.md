# CURRENT.md

## Active directive
- ID: D-20260720-1534-d049-coupled-aps-collapse-feedback-decomposition
- Project directive: D-049
- Goal: Coupled A/P/S collapse feedback decomposition under frozen biology
- Status: done
- Acceptance: met — `D049_COUPLED_ACTIVATION_CAPACITY_FAILURE`; Gates 0–11 evidence; no chemistry change
- Touched files: d049_analysis.rs, d049_tests.rs, d049.rs, main.rs, docs/d049_*, experiments/generated/d049, .agent/*
- Next action: coupled activation capacity reopen (Route A); next_execution_started=false

## Repo facts needed now
- Both analytic + restored branches collapse; A ledger closes; constitutive S→W=0
- Earliest: PRECURSOR_SYNTHESIS_DECLINE; frozen-S UPSTREAM; transport NEITHER
- Fixed P does not rescue A; Route A supersedes D-047 for coupled organism only
- Tag pending: D-049-coupled-aps-collapse-audit

## Last validation
- Command: cargo test d049_tests 22/22; D049_MAX_ACCEPTED=5000 pipeline
- Result: primary=D049_COUPLED_ACTIVATION_CAPACITY_FAILURE

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
