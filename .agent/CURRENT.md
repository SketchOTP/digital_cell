# CURRENT.md

## Active directive
- ID: D-20260722-d072-mature-membrane-damage-refill-causal-audit
- Project directive: D-072
- Goal: Causal audit of mature-membrane damage refill failure under frozen exchange
- Status: done
- Acceptance: met — `D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE` (Route X)
- Touched files: d072_analysis/tests, experiment-runner/d072, main.rs, docs/d072_*, experiments/generated/d072, .agent/*
- Next action: D-073 exchange-architecture review; next_execution_started=false

## Repo facts needed now
- Gate0: constitutive~0.897, regulated~0.894, k_p=0~0.894; floor 0.8928
- τ≈186; 1200-step sim_time≈6 ≈0.032τ; recovery worsens over 5τ
- Local p≈0.057 ⇒ θ_eq≈0.74; fixed_P control ~0.892 (does not restore)
- Synthetic isolated exchange parity PASS
- Unrelated dirty: .cursor/rules/*, AGENTS.md — exclude

## Last validation
- Command: cargo test -p chemistry-core --test d072_tests --release; D072 pipeline
- Result: 11/11 PASS; primary D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Organism-level frozen exchange does not refill 10% mature-membrane damage despite synthetic parity
