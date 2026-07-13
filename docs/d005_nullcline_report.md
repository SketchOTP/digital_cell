# D-005 Nullcline Report

**Directive:** D-005 §12  
**Reduced-order plane:** x = equivalent radius, y = mean internal catalyst concentration

## Status

Nullcline intersections computed from coarse basin macrostate velocities in `experiments/generated/d005/nullclines/intersections.json`.

## Preliminary finding

Cross-state D-004 trajectories show uniformly negative structural slope on fresh seeds — consistent with **no stable radius nullcline crossing** accessible from fresh initialization.

## Fixed-point classification

Pending full coarse grid; synthetic Jacobian tests pass in `d005_tests.rs`.

## Structural failure gate

If v_R < 0 at every tested radius → `D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR` (§19). Cross-state evidence supports this branch.

## Revision 2026-07-13

Final nullcline intersections: **0**. Fixed-point classification: no stable points.
Formalized in `experiments/generated/d005/nullclines/intersections.json` via `d005 finalize`.
