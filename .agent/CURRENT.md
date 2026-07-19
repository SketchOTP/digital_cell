# CURRENT.md

## Active directive
- ID: D-20260719-1328-d039-membrane-turnover-requirement-damage-repair
- Project directive: D-039
- Goal: Qualify exchange+damage membrane maintenance without constitutive S→W
- Status: done
- Acceptance: met — `D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED`
- Touched files: chemistry-core config/surface_density/simulation/interventions/membrane_label_tracer/d039_*, experiment-runner/d039, docs/d039_*, experiments/generated/d039
- Next action: review passive exchange / precursor coupling under schema 3; do not restore constitutive destruction

## Repo facts needed now
- Schema 3: zero constitutive S→W; historical schemas unchanged
- Gate0: MEMBRANE_MAINTENANCE_MAY_USE_EXCHANGE_PLUS_CAUSAL_DAMAGE
- Frozen v8 under schema3: A retention fail; pulse-chase replacement≈0; damage unrepaired
- Stage E: still BLOCKED_NOT_RECOVERED (not certified by D-039)

## Last validation
- Command: cargo test d039_tests; focused d024/d029/d031/d038; d039 pipeline Gates0–6
- Result: unit PASS; Gate0–2 PASS; Gate3/4/6 FAIL → D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED

## Open blockers
- None for D-039 closeout
