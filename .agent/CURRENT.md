# CURRENT.md

## Active directive
- ID: D-20260717-d023-membrane-precursor-assembly
- Project directive: D-023
- Goal: Eight-field precursor assembly (v6); evaluate isolated localization then coupled gates
- Status: done — D023_PRECURSOR_LOCALIZATION_NOT_RECOVERED
- Acceptance: met (honest failure; Gate0/1 PASS; Gate2 fail; Gates3–5 blocked)
- Touched files: fields/config/membrane/snapshot/candidate_identity/accounting/simulation, d023.rs, d023_tests, docs/d023_*, experiments/generated/d023
- Next action: Design interfacial surface-density membrane model; do not resume bulk-field M localization tuning

## Repo facts needed now
- D-021 preserved: 16213c7 / tag D-021-retention-localization-not-recovered
- D-022 preserved: e54b379 / tag D-022-localization-not-recovered
- D-023: v6 eight-field; Gate2 min loc 0.861–0.889 < 0.90; analytical k_assembly≈0.901
- Mimir slug: digital_cell

## Last validation
- Command: cargo test -p chemistry-core --release --test d008–d023; `d023 pipeline`
- Result: tests PASS; conclusion D023_PRECURSOR_LOCALIZATION_NOT_RECOVERED

## Open blockers
- Bulk-field M (v1–v6) cannot meet coupled localization; need surface-density architecture
