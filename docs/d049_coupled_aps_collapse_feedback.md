# D-049 — Coupled A/P/S Collapse and Feedback Decomposition

## Primary conclusion

`D049_COUPLED_ACTIVATION_CAPACITY_FAILURE`

Selected route: `ROUTE_A_COUPLED_ACTIVATION`

## Preservation

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Starting commit | `bdcd6bf` |
| Starting tag | `D-048-frozen-biology-membrane-fail` |
| Record | `FROZEN_COUPLED_ORGANISM_COLLAPSE_CONFIRMED` |
| D-047 status | `DIAGNOSTIC_SUPPLY_ADEQUACY_NOT_COUPLED_ATTRACTOR_PROOF` |
| Chemistry changed | **no** |
| Rates changed | **no** |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Gate 0 — D-048 evidence completeness

| Check | Result |
|-------|--------|
| Tag `D-048-frozen-biology-membrane-fail` | present |
| Starting commit `bdcd6bf` | present |
| D-048 analytic seed | fail (A retention ~1% by 10k) |
| D-048 restored snapshot | **null** (never formed mid-run) |
| D-049 analytic seed | **collapse** (A retention ≈0.022 @5k; localization ≈1.0) |
| D-049 restored branch | **ran** (bootstrap snapshot + autonomous continuation) → **collapse** (A retention ≈0.015) |
| Gate0 outcome | `D049_D048_GLOBAL_ATTRACTOR_FAILURE_REPRODUCED` |

Bootstrap note: fixed-P/A hold did not reach θ≥0.5 within bootstrap budget (`ready=false`); restored branch still executed from saved snapshot under frozen autonomous chemistry. Global attractor failure is reproduced on both branches.

## Gate 1 — Coupled ledgers

**PASS** (A field ledger closes on all windows; constitutive `S→W` ≈ 0). Soft P/S exchange residuals noted (surface vs bulk observer basis).

## Gate 2 — Spatial histories

Checkpoints saved under `spatial_histories/` (interior / interface / exterior / reservoir partitions). Localization remains high while A collapses — geometry is not lost.

## Gate 3 — Earliest causal event

`PRECURSOR_SYNTHESIS_DECLINE`

Synthesis falls early as A collapses; do not confuse with terminal deficits. Membrane localization stays high.

## Gate 4 — Frozen-membrane control

| Metric | Baseline | Frozen S |
|--------|----------|----------|
| A retention | ≈0.022 | ≈0.41 |

Classification: `UPSTREAM_OF_MEMBRANE` (freeze helps but does not reach ≥0.80). Membrane→A feedback is contributory, not primary.

## Gate 5 — Transport controls

Classification: `NEITHER`

| Control | A retention |
|---------|-------------|
| Baseline | ≈0.022 |
| Disable A transport (A/B) | ≈0.022 |
| Freeze surface (C/D) | ≈0.41 |
| No P diffusion (E) | ≈0.022 |

## Gate 6 — Demand controls

`PRECURSOR_DEMAND_CAUSAL_OVERLOAD` = **false** (no single sink reaches A retention ≥0.80)

| Control | A retention |
|---------|-------------|
| No precursor synthesis | ≈0.41 |
| Replace precursor demand | ≈0.54 |
| No structural production | ≈0.04 |
| No catalyst reproduction | ≈0.013 |
| No A decay | ≈0.013 |

Precursor demand is the strongest single demand lever but does not uniquely prevent collapse at the ≥0.80 bar.

## Gate 7 — Precursor-state controls

| Control | A retention |
|---------|-------------|
| Fixed P=0.020 | ≈0.013 |
| Fixed P=0.060 | ≈0.013 |
| No P decay | ≈0.013 |
| No P outward | ≈0.013 |

Sufficient fixed P does **not** rescue A under frozen coupled biology — D-040’s fixed-P membrane repair does not extend to activated-resource retention here.

## Gate 8 — Feedback ablations

Classification: `COUPLED_A_AND_PRECURSOR_DEFICIT`

Healthy permeability alone / with fixed P does not restore a healthy A trajectory at the retention bar.

## Gate 9 — D-040 reconciliation

Disposition: `D040_REDUCED_MODEL_VALID`

Reduced healthy fixed point still exists offline. Omitted coupled leakage/load terms matter for **basin accessibility under frozen k=0.020**, not for invalidating the reduced bistability claim. D-040 remains a reduced-model finding; it is not a coupled-organism attractor proof.

## Gate 10 — Empirical reduced model

Physical healthy fixed point: **yes** (observer model). Full-system collapse ordering remains A-led with early precursor synthesis decline.

## Gate 11 — Route

`ROUTE_A_COUPLED_ACTIVATION` → `D049_COUPLED_ACTIVATION_CAPACITY_FAILURE`

Rationale: A remains deficient under healthy-perm proxies, transport blocks, and controlled/fixed P. Route R was rejected because no_p_decay / no_p_outward / fixed-P do not restore A — ledger `p_loss > p_prod` during collapse is consequence, not cause.

This **supersedes D-047 only for the fully coupled organism** (D-047 remains valid as diagnostic supply adequacy).

## Secondary conclusions

| Item | Result |
|------|--------|
| Analytic seed | collapse |
| Restored state | collapse |
| Earliest event | `PRECURSOR_SYNTHESIS_DECLINE` |
| A loss | production ≪ productive+transport demand; retention ~2% by 5k |
| P loss | synthesis declines with A; fixed P does not rescue A |
| S flow | strongly negative net exchange; localization high |
| Transport controls | neither A nor P leakage primary |
| Demand controls | precursor demand largest lever, not ≥0.80 rescue |
| Precursor controls | fixed P fails to restore A |
| D-040 disposition | reduced model valid; not coupled attractor proof |
| Empirical FP | physical healthy FP exists offline |

## Tests and artifacts

- `cargo test -p chemistry-core --release --test d049_tests` — 22/22 PASS
- Pipeline: `D049_MAX_ACCEPTED=5000` release run — complete
- Artifacts: `digital-protocell/experiments/generated/d049/`

## Status

| Item | Status |
|------|--------|
| D-008 Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Next directive

Reopen **coupled activation capacity** for the eight-field organism (Route A). Do not redesign isolated exchange, transport, or precursor equations first. Historical activation `r_A=0.020·C·N·F` remains the frozen baseline until a coupled capacity repair is authorized.

`next_execution_started`: false
