# CURRENT.md

## Active directive
- ID: D-20260714-d008-membrane-metabolic-closure
- Project directive: D-008
- Goal: Seven-field protocell with self-produced selective membrane and activated internal metabolism
- Status: design exploration — implementation not started
- Acceptance: D-007 preserved; sequential A-G gates; one evidence-backed D008_* conclusion
- Touched files: .agent/DIRECTIVES.md, .agent/CURRENT.md, .agent/LEARNINGS.md, D-008 design spec
- Next action: user review of approved design spec, then detailed implementation plan

## Repo facts needed now
- D-007 commit/tag: bd7d5cfd5ea6a1689feae34f4285e950d61bc21d / D-007-five-field-model-rejected
- D-007 manifest SHA-256: abb9071bd26ef01604bb2a88182f8574eb687c50598b17a890ebe072858fa343
- D-007 binary SHA-256: eb91db836917baefdb90cbb8648879f0259e0841216683ceb85bc192eec98d29
- D-007 tracked tree was clean immediately after tagging
- Serena configured but unavailable for Rust symbol navigation; reported Active languages: []
- Phase1: PHASE1_SELF_MAINTENANCE_PARTIAL

## Last validation
- Command: cargo test -p chemistry-core --release --test d007_tests
- Result: 26 PASS on D-007 baseline commit

## Open blockers
- None; design spec awaits user review before implementation planning
