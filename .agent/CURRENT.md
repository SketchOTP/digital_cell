# CURRENT.md

## Active directive
- ID: D-20260724-d089-heritable-catalytic-variation
- Project directive: D-089
- Goal: Minimal heritable catalytic variation and selection after D-088
- Status: started
- Acceptance: Heritable catalytic trait with selection evidence; no fission retune
- Touched files: docs/d089_*
- Next action: Define minimal heritable catalytic specification on certified mesh lineage

## Repo facts needed now
- D-088 PASS: D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED
- y_g=0.9 frozen for reproduction path
- Runtime: PHASE1_RESEARCH_RUNTIME_QUALIFIED (90min wall-clock)
- No divide() command

## Last validation
- Command: cargo run --release -- d088 pipeline (smoke=false); 90min runtime
- Result: D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED; PHASE1_RESEARCH_RUNTIME_QUALIFIED

## Open blockers
- None

## Mimir V2
- D-088 task closing
