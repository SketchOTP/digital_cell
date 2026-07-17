# D-021 Interface-Protected Membrane Retention Report

## Mission

Repair v3 membrane/A bootstrap failure via local interface-protected membrane
turnover (`membrane_metabolism_v4_interface_protected`), then recover Stage E
only after retention/localization gates pass.

## D-020 preservation

- Commit: `243e540` — `D-020: Record bounded v3 joint-rate failure`
- Tag: `D-020-v3-joint-rate-recovery-fail`
- Artifacts retained under `experiments/generated/d020/` and `docs/d020_*`

## Mechanism

```text
r_M_decay = k_M_decay × M × [ε_M + (1 − I(φ))]
```

Off-interface detachment retained. No target radius/mass. No new fields/pumps.

- `equation_version = membrane_metabolism_v4_interface_protected`
- `membrane_schema_version = 2`
- Screened: ε_M ∈ {0.02, 0.05, 0.10}
- Frozen initially: productive rates, stoichiometry schema 2, transport schema 1,
  interface-limited structure turnover, yields, environment

## Gate results

### Gate 1 — Local mechanism + Stage B

All three ε pass local mechanism checks and Stage B localization:

| ε_M | localization min | Stage B |
| --- | ---------------- | ------- |
| 0.02 | 0.9069 | PASS |
| 0.05 | 0.9068 | PASS |
| 0.10 | 0.9065 | PASS |

### Gate 2 — Fixed compartment (R16/R24/R32)

All three ε: `D021_STAGE_D_FIXED_COMPARTMENT_PASS`. No regression.

### Gate 3 — R22 pre-balance (productive rates frozen)

| ε_M | C ret | A ret | M loc | promote |
| --- | ----- | ----- | ----- | ------- |
| 0.02 | 0.999 | 0.9996 | 0.8891 | NO |
| 0.05 | 0.999 | 0.9996 | 0.8891 | NO |
| 0.10 | 0.999 | 0.9996 | 0.8891 | NO |

A retention recovers strongly vs D-020 (0.377 → ~1.0). Membrane localization on
the constrained R22 short screen remains ~0.889 < 0.90, so no ε is promoted.

### Gate 4 / Gate 5

Not run — retention/localization gate not met; rate search forbidden.

## Interpretation

Interface protection repairs activated-resource retention under frozen rates, but
does not lift constrained-radius membrane localization through the 0.90 gate.
Further productive-rate calibration would violate the governing principle.

## Conclusion

```text
D021_RETENTION_LOCALIZATION_NOT_RECOVERED
```

D-008 Stage E remains `BLOCKED_NOT_RECOVERED`. Stage F not started.

Reject continuing the seven-field membrane bootstrap via rate search alone.
Next work requires a new local membrane-localization mechanism (not joint-rate
compensation).

## Artifacts

- `experiments/generated/d021/gate1/`
- `experiments/generated/d021/gate2/`
- `experiments/generated/d021/gate3/`
- `experiments/generated/d021/manifest.json`

## Tests

```text
cargo test -p chemistry-core --release --test d021_tests --test d020_tests --test d019_tests --test d012_tests --test d011_tests --test d008_tests
```

PASS (d021 11, d020 8, d019 9, d012 50, and prior suites green).
