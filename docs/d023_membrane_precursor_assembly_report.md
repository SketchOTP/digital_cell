# D-023 Membrane-Precursor Interface Assembly Report

## Mission

Replace the failed single-field membrane-localization architecture with one
explicit soluble membrane precursor (`P`) and evaluate whether internally
produced precursor can diffuse to the structural interface and assemble into
localized membrane (`M`) while preserving retention, conservation, turnover,
and autonomous causal closure.

Do not resume seven-field rate or transport tuning.

## D-022 / D-021 preservation

- D-021 commit: `16213c7` — tag `D-021-retention-localization-not-recovered`
- D-022 commit: `e54b379` — tag `D-022-localization-not-recovered`
- Frozen: `ε_M = 0.02`, D-022 χ screens / failed R22 localization evidence
- Historical equation versions and artifacts left immutable

## Architecture

```text
equation_version = membrane_metabolism_v6_precursor_assembly
fields = φ, C, N, F, W, A, P, M
field_schema = eight_field_v1
precursor_schema = 1
stoichiometric_schema = 2
membrane_transport_schema = 1   # χ_M = 0, diffusion-only M
```

Reactions (direct `A → M` disabled):

```text
A → P   r_precursor = k_precursor × A × q(C) × H(φ)
P → M   r_assembly  = k_assembly × P × I(φ) × max(0, 1 − M/M_max)
P → W   r_P_decay   = k_precursor_decay × P
M → W   D-021 interface-protected membrane loss retained
```

Frozen transport for the bounded experiment:

- `D_P = D_A`
- precursor membrane attenuation = A attenuation
- `k_precursor_decay = k_A_decay`
- `χ_M = 0`
- only independently screened parameter: `k_assembly`

## Gate results

### Gate 0 — Schema and preservation

PASS (unit-backed by `d023_tests`): eight current/next buffers and swap,
eight-field snapshots, v1–v5 readable by original versions, seven-field
cannot resume as v6, candidate hash includes precursor params.

### Gate 1 — Conservation and causal chemistry

PASS (unit-backed): P requires A and C; M requires P; direct A→M disabled;
assembly conserves P+M; turnover to W; material and activation accounting
close; chemistry independent of observer metrics.

### Gate 2 — Isolated assembly and localization

Analytical bootstrap:

| quantity | value |
| --- | --- |
| bootstrap `k_assembly` | 0.3 |
| bootstrap min localization | 0.8975 |
| analytical `k_assembly` | 0.9014 |
| membrane loss / assembly basis | 1.7145 / 1.9020 |

Screen (fixed φ/C, supplied A; promote smallest with localization ≥ 0.90):

| factor | `k_assembly` | min loc after transient | interior+exterior frac | gate2_pass |
| --- | --- | --- | --- | --- |
| 0.5× | 0.4507 | 0.8895 | 0.1093 | NO |
| 1.0× | 0.9014 | 0.8750 | 0.1236 | NO |
| 2.0× | 1.8028 | 0.8614 | 0.1369 | NO |

All candidates: clean termination, active A→P / P→M / M→W, bounded P and M,
accounting closed. None reached localization ≥ 0.90; all retained
interior/exterior M accumulation > 0.10. Higher `k_assembly` worsened
localization (more assembled M subject to bulk diffusion).

No candidate promoted.

### Gate 3 / Gate 4 / Gate 5

Not run — directive requires a passing isolated candidate before coupled
R22, fixed-compartment regression, or Stage E recovery.

## Interpretation

The eight-field precursor path is chemically causal and conservative, but
bulk-field `M` assembled from soluble `P` still fails Stage B localization
under the frozen transport choices. Best screened min localization is
~0.8895 (0.5×), below the 0.90 gate and below D-022 Stage B
(0.9074–0.9088).

Per directive: **reject the bulk-field membrane architecture** and design a
true interfacial surface-density model. Do not resume seven-field or
eight-field bulk-M localization tuning.

## Conclusion

```text
D023_PRECURSOR_LOCALIZATION_NOT_RECOVERED
```

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: not advanced
- Production verdict: reject bulk-field membrane localization; next
  architecture must be an interfacial surface-density model

## Artifacts

- `digital-protocell/experiments/generated/d023/`
  - `preservation/`, `schema/`, `conservation/`, `isolated_assembly/`
  - `r22_bootstrap/` (blocked), `fixed_compartments/` (blocked)
  - `stage_e_candidates/` (blocked), `accounting/`
  - `gate0/`, `gate1/`, `gate2/`, `manifest.json`

## Tests

```text
cargo test -p chemistry-core --release --test d023_tests --test d022_tests \
  --test d021_tests --test d020_tests --test d019_tests --test d014_tests \
  --test d012_tests --test d008_tests
```

PASS (d023 12, d022 10, d021 11, d020 8, d019 9, d014 20, d012 50, d008 50).
