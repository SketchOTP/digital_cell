# D-005 D-004 Aggregate Report

**Directive:** D-005 §5  
**Source:** `experiments/generated/d005/d004_aggregate/aggregate.json`  
**Run count:** 21 cross-state experiments (unchanged from D-004)

## Recorded conclusions (preserved)

```text
D-004: D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT
D-003: D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH
Phase 1: PHASE1_SELF_MAINTENANCE_PARTIAL
```

## Summary by state class

| k_phi | fresh_seed (3 seeds) | D-002 aged (seed 2) | calibration_endpoint (3 seeds) |
|-------|----------------------|---------------------|--------------------------------|
| 0.5 | ContinuedDrift; Qφ 0.76–0.86; slope_φ negative | Collapse (NoActiveAttractor) | State-dependent; higher Qφ on endpoint |
| 1.0 | ContinuedDrift; seed1 Qφ≈0.86, seeds 2–3 Qφ≈0.76 | Collapse | Endpoint Qφ≈0.98 but fresh seeds fail |
| 2.0 | ContinuedDrift; similar sensitivity | Collapse | Endpoint-dependent balance |

## Behavior classification (all 21 runs)

- **fresh_seed (9 runs):** `unresolved_long_transient` — all show negative structural slope at 100k; no qualifying balance window.
- **D-002 aged (3 runs):** `continued_collapse` — loss of coherent protocell; low retention.
- **calibration_endpoint (9 runs):** mixed — endpoint states can approach Qφ≈1 on short windows but do not represent fresh-seed accessibility.

## Cross-state convergence

**None.** No candidate shows convergent active attractor across fresh, aged, and calibration-endpoint classes within D-004 tolerance bands.

## Corrected Stage B (K_phi=1.0, calibrated candidate)

| Seed | Qφ | QC | Pass |
|------|-----|-----|------|
| 1 | 0.863 | 0.898 | yes |
| 2 | 0.757 | 0.754 | no |
| 3 | 0.757 | 0.754 | no |

Evidence of **seed sensitivity**, not robust viability.

## Implication for D-005

Fresh analytical seeds do not reliably reach a bounded active organization. Calibration-endpoint and aged states are **not** acceptable initialization for accessibility testing (§9, §16).
