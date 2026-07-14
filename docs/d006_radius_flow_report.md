# D-006 Radius Flow Report

**Directive:** D-006C  
**Agent memory:** D-20260713-d006c-surface-turnover-completion  
**Equation version:** `surface_turnover_v1`  
**Stage D jobs:** 180 / 180 complete  
**Scientific conclusion:** `D006_NO_RESTORING_RADIUS`

## Method

Coupled fresh-seed runs (50,000 accepted substeps) at  
R₀ ∈ {16,20,24,28,32} × C₀ ∈ {0.275,0.35,0.425} × seeds {1,2,3}  
for the four prescribed-radius survivors (0.80×–1.40×).

Velocities use simulated time. Invalid-stabilization flags applied before medians.

## Candidate-by-candidate radius flow

All four candidates show **median v_R > 0 at every tested radius and catalyst loading**.

No ordered restoring-radius crossing exists.

Machine table: `experiments/generated/d006/stage_d/aggregate_flow.json` (`radius_flow_table`).

## Catalyst-state flow

At every Stage D macrostate point, median `v_C_inside < 0` while median `v_R > 0`:

- radius expands
- mean internal catalyst concentration declines slowly
- catalyst retention remains ≥ 0.88 (not extinction)

No radius/catalyst nullcline intersection in the tested region  
(`stage_d/nullcline_summary.json`).

## Interpretation

Prescribed-field Stage C restoring crossings **did not survive** full coupling.  
Coupled organism dynamics in the screened window are expansive in radius, without a restoring R*.

## Stage D gate

- Restoring radial sign pattern: **fail**
- Selected candidate: **none**
- Stage E/F: **not run**
