# D-039 — Membrane Turnover Requirement and Damage-Repair Qualification

## Mission

Determine whether Phase 1 requires continuous constitutive mature-membrane `S→W` destruction, and qualify the simplest corrected architecture: conserved interfacial `S=δΓ`, reversible `P↔S` exchange, no constitutive `S→W`, declared local damage, metabolism-dependent repair, observer-only molecular tracing.

## Frozen evidence

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Starting commit | `c74dd95` |
| D-038 tag | `D-039` starts from `D-038-corrected-turnover-renewal` |
| D-038 result | `D038_NO_MEMBRANE_ARCHITECTURE_RECOVERED` |
| Architecture | `membrane_metabolism_v8_reversible_surface_exchange` |
| Record | `CONSTITUTIVE_MEMBRANE_TURNOVER_UNCERTIFIED` |

## Gate 0 — Contract audit

**Conclusion:** `MEMBRANE_MAINTENANCE_MAY_USE_EXCHANGE_PLUS_CAUSAL_DAMAGE`

| Requirement | Required? |
|-------------|-----------|
| Continuous organizational maintenance | yes |
| Material / component replacement | yes |
| Causal damage repair | yes |
| Uniform constitutive first-order `S→W` hazard | **no** (unless independently justified) |

D-008 Stage B/E require active turnover and overlapping zero-flow balances; they do not independently justify an unsupported permanent mature-membrane destruction term (D-037 `MIXED_PURPOSE_TERM`).

## Schema 3 equations

Experimental schema (not historical default):

`surface_turnover_schema_3_exchange_damage_only`

### Ordinary operation

\[
\lambda_{\mathrm{turnover}} = 0
\qquad\Rightarrow\qquad
J_{S\to W}^{\mathrm{constitutive}} = 0
\]

Reversible exchange (frozen D-031 / D-030):

\[
\alpha \approx 0.167,\quad
\beta \approx 0.00334,\quad
k_{\mathrm{exchange}}\approx 0.00334,\quad
K_{\mathrm{exchange}}\approx 50
\]

Integrator: invariant-domain BE+Strang (`InvariantDomainV2`).

### Declared damage (external intervention only)

At intervention time, select a contiguous interface arc and convert a fraction \(f\) of total \(S\) into \(W\):

\[
S \xrightarrow{\text{declared}} W
\quad\text{(exact mass transfer; no rate retune; no repair controller)}
\]

Geometry is frozen at the intervention moment and is not kept active afterward.

## Tracer

Observer-only `MembraneLabelTracer`:

- tracks `label_p`, `label_s`, `label_removed_to_w`
- transfers proportionally with gross exchange and declared damage
- does not affect chemistry, transport, dt, or candidate selection
- conservation: `label_p + label_s + label_removed_to_w = initial` (except floating point)

Pulse-chase: label all \(S\) as old; unlabeled adsorption dilutes old fraction.

## Revised Stage E membrane contract (definition only)

D-039 does **not** execute or pass Stage E.

Future Stage E membrane requirements:

- normalized net \(S\) flow \(\le 10^{-4}\)
- bounded \(S,P\); stable occupancy
- active gross adsorption and desorption
- demonstrated molecular replacement
- metabolism-dependent damage repair
- localization / retention gates
- accounting closure

**Removed:** unsupported `membrane production / constitutive membrane destruction = 1` when no constitutive destruction mechanism is present.

## Artifacts

`digital-protocell/experiments/generated/d039/`

## Status

See `result.json` / `manifest.json` for the selected `D039_*` conclusion after the gated pipeline completes.

## Pipeline results (schema 3 / v8)

**Primary conclusion:** `D039_CONTINUOUS_REPLACEMENT_NOT_ESTABLISHED`

| Gate | Result |
|------|--------|
| 0 Contract audit | PASS — `MEMBRANE_MAINTENANCE_MAY_USE_EXCHANGE_PLUS_CAUSAL_DAMAGE` |
| 1 Schema safety | PASS — schema 3 isolated; historical default unchanged |
| 2 Tracer | PASS — field parity; conservation |
| 3 Stable baseline | FAIL — A retention fixed≈0.44 / dynamic≈0.18 (<0.80); net S flow not ≤1e−4 |
| 4 Pulse-chase | FAIL — replacement≈0; S drift≈0.69 |
| 5/6 Damage 10/25% | FAIL — irreversible collapse (S recovery≈0.28/0.27) |
| 5/6 Damage 40% | irreversible_membrane_failure (non-mandatory) |
| 7–9 | Skipped after maintenance falsification |
| 10 Stage E contract | Defined; Stage E not executed |

### Scientific conclusion

Self-maintenance does **not** require unsupported constitutive mature-membrane destruction (Gate 0). However, under frozen v8 reversible exchange with schema-3 zero constitutive `S→W`, the membrane does **not** demonstrate continuous molecular replacement or metabolism-dependent damage repair. Membrane mass drifts downward via net desorption when activated-resource / precursor supply cannot sustain exchange, so labeled cohorts are not replaced and local damage is not repaired by ordinary chemistry.

Do **not** restore arbitrary constitutive `S→W`. Next: review the passive exchange law / precursor coupling before further architecture.

### Status

| Item | Status |
|------|--------|
| D-008 Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |
| Constitutive turnover | `CONSTITUTIVE_MEMBRANE_TURNOVER_UNCERTIFIED` |

### Route

`REVIEW_PASSIVE_EXCHANGE_LAW_DO_NOT_RESTORE_CONSTITUTIVE_DESTRUCTION`

Next directive: revise passive exchange / precursor coupling under schema 3; do not implement D-036; do not reintroduce unsupported constitutive destruction.

### Tests

- `d039_tests` focused suite
- Historical focused: `d024_tests`, `d029_tests`, `d031_tests`, `d038_tests` (PASS)

### Tag

`D-039-membrane-maintenance-qualification`
