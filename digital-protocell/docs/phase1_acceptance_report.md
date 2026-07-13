# Phase 1 Acceptance Report (D-002)

**Generated:** 2026-07-12  
**D-001 savepoint:** `2123435` (tag `D-001-baseline`)  
**D-002 commit:** `95dbfed`  
**Frozen config:** `configs/phase1_candidate.toml`

## Final conclusion

```text
PHASE1_SELF_MAINTENANCE_PARTIAL
```

The protocell demonstrates active chemistry, mass accounting within tolerance, and measurable turnover over 250,000 substeps, but **does not** meet D-002 pass criteria: no seed reached `VIABLE` at 250k, no seed achieved whole-mass-equivalent turnover (all four ratios ≥ 1.0), and long-horizon interventions/repair/damage curves were not fully executed in this session.

---

## Implementation changes

| Component | Change |
|-----------|--------|
| `accounting.rs` | Per-field step ledgers + cumulative stoichiometric ledger |
| `simulation.rs` | Accounting integration, timing telemetry, `field_hash()`, optional observer |
| `diagnostics.rs` | 25,000-substep rolling viability window, turnover ratios, morphology sampling |
| `interventions.rs` | Spatial wound region, damage fractions, synthesis/reservoir knockouts |
| `experiment-runner` | `baseline` / `acceptance` CLI, checkpoints, artifact manifest |
| `validation_tests.rs` | 17 D-002 §5 tests (conservation, stoichiometry, observer, repair, resurrection) |

---

## Numerical validation

| Test | Result |
|------|--------|
| `test_cahn_hilliard_conserves_structural_mass` | PASS |
| `test_passive_free_energy_is_nonincreasing` | PASS |
| Diffusion conservation (C,N,F,W) | PASS (4/4) |
| Stoichiometric R1–R5 ledgers | PASS (5/5) |
| `test_observer_has_no_causal_effect` | PASS |
| Repair/resurrection (short-run) | PASS (4/4) |
| Integration smoke (22 tests) | PASS (prior session + catastrophic fix) |

**Accounting (all 5 seeds @ 250k):** cumulative unexplained residual ≤ 1×10⁻⁵ × processed mass; 250,000/250,000 steps within per-step tolerance.

---

## Baseline results (250,000 substeps)

| Seed | Wall (s) | Classification | Init struct | Final struct | Init cat | Final cat | Struct repl | Struct synth | Cat repl | Cat repro | Accounting | Pass |
|------|----------|----------------|-------------|--------------|----------|-----------|-------------|--------------|----------|-----------|------------|------|
| 1 | 5493 | Transient | 1857.7 | 1209.2 | 636.6 | 550.1 | 0.395 | 0.046 | 0.254 | 0.113 | OK | **fail** |
| 2 | 6322 | Transient | 1857.7 | 823.1 | 636.6 | 351.4 | 0.659 | 0.101 | 0.625 | 0.163 | OK | **fail** |
| 3 | 6374 | Transient | 1857.7 | 823.2 | 636.6 | 351.3 | 0.659 | 0.101 | 0.625 | 0.163 | OK | **fail** |
| 4 | 6321 | Transient | 1857.7 | 823.1 | 636.6 | 351.3 | 0.659 | 0.101 | 0.625 | 0.163 | OK | **fail** |
| 5 | 6370 | Transient | 1857.7 | 823.2 | 636.6 | 351.4 | 0.659 | 0.101 | 0.625 | 0.163 | OK | **fail** |

**Replicate pass rate:** 0/5 (need ≥4/5)

Seed 1 reached brief `Viable` windows at substeps 49k–149k but lost consecutive qualifying windows by 250k (structure/catalyst mass CV and retention degraded). Seeds 2–5 (deterministic noise variants) converged to a lower-mass steady decline with higher decay turnover but still below 1.0 synthesis/reproduction ratios.

**Turnover ratio trajectory (seed 1):**

| Substep | Struct repl | Struct synth | Cat repl | Cat repro |
|---------|-------------|--------------|----------|-----------|
| 25k | 0.048 | 0.002 | 0.017 | 0.014 |
| 50k | 0.093 | 0.005 | 0.035 | 0.028 |
| 100k | 0.179 | 0.013 | 0.077 | 0.053 |
| 150k | 0.257 | 0.023 | 0.128 | 0.076 |
| 200k | 0.329 | 0.034 | 0.188 | 0.096 |
| 250k | 0.395 | 0.046 | 0.254 | 0.113 |

---

## Intervention results

| Intervention | Status |
|--------------|--------|
| Nutrient starvation @ 50k | **Not run** (250k post-intervention pending) |
| Fuel starvation @ 50k | **Not run** |
| Catalyst reproduction knockout | **Not run** |
| Structural synthesis knockout | **Not run** |
| Reservoir shutdown | **Not run** |

Short-run intervention tests (5k–8k) in `integration_tests.rs` remain passing from D-001.

---

## Repair / damage / controls / Godot

| Item | Status |
|------|--------|
| Spatial repair (25° wedge) | Short-run tests PASS |
| Damage-response curve (10–80%) | **Not run** at 250k |
| Passive / no-rep / no-structure controls | **Not run** at 250k |
| Godot bridge equivalence | Headless reproducibility PASS; Godot 4.6 runtime not verified this session |

---

## Failures and known issues

1. **No 250k seed passes viability or turnover thresholds** — primary scientific blocker.
2. Structure and catalyst mass decline over 250k despite positive instantaneous turnover rates.
3. `k_structure` tuned for measurable short-run turnover is insufficient for whole-mass replacement at 250k.
4. Long-horizon causal interventions not executed (estimated ~7+ additional hours).
5. Mimir `memory_record_outcome`: BLOCKED (504 gateway timeout).

---

## Artifacts

- `experiments/generated/phase1_acceptance/baseline_seed_{1..5}/`
- Checkpoints: 0, 25k, 50k, 100k, 150k, 200k, 250k (field PNGs + snapshots)
- `experiments/generated/phase1_acceptance/manifest.json` (SHA-256, seed 1 bundle)

---

## Recommended next steps

1. Parameter search targeting turnover ratios ≥ 1.0 **without** tuning during acceptance runs.
2. Execute §11 intervention suite at 250k post-intervention horizon.
3. Re-run acceptance only after failed evidence preserved (current artifacts immutable).
