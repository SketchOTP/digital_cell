# CURRENT.md

## Active directive
- ID: D-20260721-d067-activation-capacity-law-identification
- Project directive: D-067
- Goal: Identify whether one local conservative activation law converts smooth N/F into enough A (observer/shadow; frozen stoichiometry)
- Status: done
- Acceptance: met — `D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW`
- Touched files: d067_analysis/tests, d050_analysis, experiment-runner/d067, main.rs, lib.rs, docs/d067_*, experiments/generated/d067, .agent/*
- Next action: audit dominant A-demand / membrane Stage E under frozen activation; next_execution_started=false

## Repo facts needed now
- Primary: Route N `D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW`
- D-066 reproduced; ordinary A≈0.355; unlimited≈1.81; χ_A≈0.117; χ_min≈1.27
- N̂/F̂ linear unclipped; ordinary product suppressed (`ORDINARY_RESPONSE_LINEAR_LOW`)
- Candidate B: m_V≳11 needed for A≥0.80 but high-N/F rejects
- Candidate C: durable ≥1200-step ordinary A peaks ≈0.65 <0.80 (600-step false positive)
- Selected activation law: none; production defaults unchanged
- Artifacts: experiments/generated/d067 → /mnt/storage1tb/.../d067

## Last validation
- Command: cargo test -p chemistry-core --test d067_tests; D067_MAX_ACCEPTED=1200 D067_SKIP_LATE_GATES=1 pipeline
- Result: 10/10 PASS; primary NoPortableActivationCapacityLaw / Route N

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded from D-067 staging
