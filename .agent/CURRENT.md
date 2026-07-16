# CURRENT.md

## Active directive
- ID: D-20260716-d017-waste-architecture-comparison
- Project directive: D-017
- Goal: Compare activation-yield vs energy-coupled active W export
- Status: done (reject both)
- Acceptance: One D017_* selection/rejection with evidence — met as D017_REJECT_BOTH_ARCHITECTURES
- Touched files: d017_comparison, d017_tests, experiment-runner/d017, docs/d017_*, d012/d015/d016 appends
- Next action: Next directive — identify upstream mechanism beyond A/B using D-017 evidence (structure-turnover dominance)

## Repo facts needed now
- D017_REJECT_BOTH_ARCHITECTURES; tag D-017-reject-both-waste-architectures
- Direct activation W ≈5%; structure turnover ≈89% of frozen source
- Perfect-interface center W≈12.69 ≥10; internal delivery insufficient
- D-012 solver: CLOSED
- Mimir slug: digital_cell

## Last validation
- Command: cargo test -p chemistry-core --release --test d012/d013/d014/d015/d016/d017
- Result: PASS 50+32+20+32+24+17

## Open blockers
- Stage E waste unresolved; A/B architectures falsified
