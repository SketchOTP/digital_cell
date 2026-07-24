# CURRENT.md

## Active directive
- ID: D-20260724-d090-ecological-timescale-selection-requalification
- Project directive: D-090
- Goal: Repair shared-dish ecology timescale; qualify or reject C_H/C_B evolution
- Status: done — no valid selection ecology
- Acceptance: met (primary conclusion established with Gates 0–3 evidence)
- Touched files: spatial_shared_dish, founder_preconditioning, ecological_timescales, shared_dish_audit, d090_*, docs/d090_*, .agent/*
- Next action: architecture review of organism–environment resource coupling (no Phase 3)

## Repo facts needed now
- D-089 seal: 6d363a7 / tag D-089-natural-selection-not-established
- D-090 conclusion: D090_VALID_SELECTION_ECOLOGY_NOT_ESTABLISHED
- Hypothesis upheld as ecology/timescale issue: EARLY_FISSION_PRECEDED_SELECTION_PRESSURE
- Organism remains frozen; trait not rejected on selection grounds

## Last validation
- Command: D090_SMOKE=1 D090_ASSUME_GATE0=1 cargo run --release -p experiment-runner -- d090 pipeline; cargo test -p chemistry-core --test d090_tests
- Result: Gates 0–2 PASS; Gate 3 FAIL all H/B candidates; unit tests PASS

## Open blockers
- Valid selection ecology not established under frozen biology

## Mimir V2
- task 75a7ae47e6204966a59d95ef7c0b48fc (closing)
