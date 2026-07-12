# Phase 1 Acceptance Report (D-002)

**Status:** IN PROGRESS — 250,000-substep baseline seed 1 running at report generation.

**D-001 savepoint:** `2123435` (tag `D-001-baseline`)

**Frozen config:** `configs/phase1_candidate.toml`

## Implementation changes

| Area | Change |
|------|--------|
| Accounting | `accounting.rs` — per-field step ledgers + cumulative stoichiometric ledger |
| Simulation | Mass accounting integrated in `try_substep`; timing telemetry; `field_hash()` |
| Observer | 25,000-substep rolling viability window; turnover ratios; morphology sampling interval |
| Interventions | Spatial wound region, damage fractions, structural/reservoir knockouts |
| Experiment runner | `baseline` and `acceptance` subcommands; checkpoints; manifest |
| Validation | 17 new tests in `validation_tests.rs` |

## Numerical validation (5000-substep probe, seed 1)

| Test | Result |
|------|--------|
| Mass conservation (Cahn–Hilliard passive) | PASS (≤1e-3 relative drift) |
| Free energy non-increasing | PASS |
| Soluble diffusion conservation (C,N,F,W) | PASS |
| Stoichiometric R1–R5 ledgers | PASS |
| Observer non-causal | PASS |
| Accounting cumulative residual | PASS (1.2e-6 vs 2.6e8 processed mass) |

## Baseline results

| Seed | Substeps | Classification | Struct rep ratio | Synth ratio | Cat rep ratio | Cat repl ratio | Pass |
|------|----------|----------------|------------------|-------------|---------------|----------------|------|
| 1 | 250000 | PENDING | — | — | — | — | pending |
| 1 (probe) | 5000 | Transient | 0.010 | 0.0004 | 0.003 | 0.003 | partial |

Turnover ratios ≥ 1.0 require full 250k run (probe shows active but immature turnover).

## Intervention results

Pending full acceptance suite (`experiment-runner acceptance`).

## Repair results

Spatial repair tests PASS at 8k+5k substeps (`test_repair_is_spatially_local`).

## Controls

Pending passive / no-rep / no-structure 250k controls.

## Failures

None recorded in probe run. Starvation death at 250k not yet verified.

## Final conclusion

```text
PHASE1_SELF_MAINTENANCE_PARTIAL
```

Upgrade to `PHASE1_AUTOPOIETIC_CANDIDATE_PASS` requires completed 250k evidence per D-002 §20.
