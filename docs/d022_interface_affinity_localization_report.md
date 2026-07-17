# D-022 Interface-Affinity Membrane Localization Report

## Mission

Recover membrane localization under coupled R22 while preserving D-021’s restored
A retention, via conservative local interface-affinity M transport
(`membrane_metabolism_v5_interface_affinity`).

## D-021 preservation

- Commit: `16213c7` — `D-021: Interface-protected membrane; retention gate fails`
- Tag: `D-021-retention-localization-not-recovered`
- Retained: `membrane_metabolism_v4_interface_protected`, ε_M candidates,
  A retention ≈ 0.9996, R22 M localization ≈ 0.889
- Frozen for D-022: ε_M = 0.02 (best D-021 Gate3 localization among screens)

## Mechanism

```text
J_M(i→j) = −D_M · (M_j − M_i) + χ_M · mean(M_i,M_j) · (I_j − I_i)
I = local φ-interface indicator
```

- `equation_version = membrane_metabolism_v5_interface_affinity`
- `membrane_transport_schema = 2`
- Face flux antisymmetric; total M conserved by transport; local-only
- χ_M = 0 recovers v4 diffusion path
- D-021 production, decay (ε_M), and detachment unchanged
- No target radius, mass, or localization score
- Screened: χ_M / D_M ∈ {0.5, 1.0, 2.0}

## Gate results

### Gate 1 — Transport integrity

PASS (unit-backed): antisymmetric flux, M conservation, χ_M=0 ≡ v4,
local-only affinity, no forbidden targets. Schema = 2.

### Gate 2 — Localization (Stage B + short R22)

| χ_M/D_M | χ_M | Stage B loc min | R22 M loc | A ret | C ret | promote |
| ------- | --- | --------------- | --------- | ----- | ----- | ------- |
| 0.5 | 0.0005 | 0.9074 | 0.8895 | 0.9996 | 0.9992 | NO |
| 1.0 | 0.0010 | 0.9079 | 0.8899 | 0.9996 | 0.9992 | NO |
| 2.0 | 0.0020 | 0.9088 | 0.8907 | 0.9996 | 0.9992 | NO |

Stage B localization passes for all three. Coupled short R22 localization
improves slightly with χ (0.8895 → 0.8907) but remains **below 0.90**.
A/C retention remain ≥ 0.80 (≈ 0.999). No candidate promoted.

### Gate 3 / Gate 4

Not run — localization gate not met; joint-rate recovery forbidden until
localization promotes.

## Interpretation

Interface affinity moves M toward the locally generated interface without
damaging D-021 A retention, but the coupled R22 localization deficit is only
~0.01 and affinity within the screened χ/D band does not close it.

Per directive: **reject further seven-field membrane-localization tuning**.
The next directive must introduce an explicit membrane-precursor or
membrane-bound component rather than another rate or transport adjustment.

## Conclusion

```text
D022_LOCALIZATION_NOT_RECOVERED
```

D-008 Stage E remains `BLOCKED_NOT_RECOVERED`. Stage F not started.

## Artifacts

- `experiments/generated/d022/gate1/`
- `experiments/generated/d022/gate2/`
- `experiments/generated/d022/manifest.json`

## Tests

```text
cargo test -p chemistry-core --release --test d022_tests --test d021_tests --test d020_tests --test d019_tests --test d012_tests --test d011_tests --test d008_tests
```

PASS (d022 10, d021 11, d020 8, d019 9, d012 50, d011 21, d008 50).
