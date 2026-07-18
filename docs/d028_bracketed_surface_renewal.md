# D-028 — Bracketed Surface-Renewal Root Recovery

## Conclusion

`D028_ROOT_NOT_PORTABLE`

## Operative status

| Item | Status |
|------|--------|
| D-021–D-027 | Preserved (commits, tags, artifacts unchanged) |
| D-027 historical | `D027_ISOLATED_SURFACE_RENEWAL_FAILURE` (unchanged) |
| Additional record | `D027_SURFACE_BALANCE_ROOT_BRACKETED` |
| D-028 | `D028_ROOT_NOT_PORTABLE` |
| D-008 | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |

## Branch / starting commit

- Branch: `d008-membrane-metabolic-closure`
- Starting commit: `15d46f2` (`D-027: Preserve coupled surface-renewal failure`)
- Failure tag (prior): `D-027-surface-renewal-fail`

## Exact bracket (machine values)

| Endpoint | `k_ads` | Late `Q_surface` (Gate 0 live) |
|----------|--------:|-------------------------------:|
| D-027 0.5× (frozen Q) | 0.01688873302573429 | 0.6145455496162924 |
| D-027 1× | 0.03377746605146858 | 0.9064945432686394 |
| Midpoint | 0.05066619907720287 | 1.0249857798937156 |
| D-027 2× | 0.06755493210293716 | 1.0568472407802698 |

Gate 0 live 1×/2× Q matched frozen D-027 isolated artifacts bit-for-bit.

## Gate results

### Gate 0 — Bracket reproduction — PASS

- Lower: `Q < 0.98`, bounded, no saturation lock, accounting OK
- Upper: `Q > 1.02`, bounded, no saturation lock, accounting OK
- Monotonicity: frozen 0.5/1/2 and live 1× → mid → 2× all increasing
- Recorded: `D027_SURFACE_BALANCE_ROOT_BRACKETED`

### Gate 1 — Bracketed root solve — PASS

- Method: safeguarded regula-falsi (max 4 new candidates)
- First trial: `k = 0.054783922487850654` → `Q = 1.038370` (not balanced)
- Selected root: **`k_ads = 0.04867196940427757`**
  - `Q_surface = 1.0167867088768683`
  - `g_surface = 3.357335981956185e-05`
  - Localization ≈ 1.0; active ads/turnover; P/S bounded

### Gate 2 — Local ±2% robustness — PASS

| Perturbation | `k_ads` | `Q_surface` |
|--------------|--------:|------------:|
| −2% | 0.04769853001619202 | 1.012330 |
| center | 0.04867196940427757 | 1.016787 |
| +2% | 0.04964540879236312 | 1.020942 |

Ordered `Q_− < Q_0 < Q_+`; all bounded and localized; center balanced.

### Gate 3 — Six-state portability — FAIL (2/6)

Assay: D-027 Gate1 states; override `k_ads` to selected root; 4000-step settle + 2000-step common window.

| State | `Q_surface` | Pass `[0.90,1.10]` |
|-------|------------:|:------------------:|
| D-024 fixed R22 | 0.9761 | yes |
| D-025 dynamic R22 | 1.7988 | no |
| Stage E 10k | 1.0674 | yes |
| Stage E 25k | 1.2343 | no |
| Stage E 100k | 1.7217 | no |
| Stage E 200k | 1.6930 | no |

Failed states show `Q > 1.10` with `g_surface > 0` (flow away from balance under the isolated root).

### Gates 4–11

Not started (stop on Gate 3).

## Scientific reading

A balance point exists inside the D-027 1×–2× bracket on the isolated fixed-interface screen, and it is locally robust (±2%). That scalar does not remain near balance when transplanted onto dynamic R22 or late Stage E coverage-deficit states. The adsorption law is therefore not portable at the isolated root; exchange-law revision is the authorized next step (desorption / chemical-potential exchange), not further productive-rate calibration under frozen kinetics.

## Adsorption law

Unchanged (`membrane_metabolism_v7_surface_density` adsorption equation retained).

## Artifacts

`digital-protocell/experiments/generated/d028/` — preservation, bracket_reproduction, root_iterations, local_robustness, portability, manifest.json.

## Next directive

Architect-authorized bulk–surface exchange improvement (reversible adsorption/desorption or chemical-potential-based exchange). Do not begin Stage F. Do not reopen productive-rate-only repair under frozen surface kinetics.

## Terminal seal

- Commit: `0dcc6e0` (solver/runner) + this preserve commit
- Tag: `D-028-bracketed-renewal-fail`
- Conclusion: `D028_ROOT_NOT_PORTABLE`
