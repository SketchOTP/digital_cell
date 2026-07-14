# D-007 — D-006 joint kinetic diagnosis

## Governing status (frozen)

```text
D-005: D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR
D-006: D006_NO_RESTORING_RADIUS
Phase 1: PHASE1_SELF_MAINTENANCE_PARTIAL
```

## D-006 preservation gate

| Check | Value |
| --- | --- |
| Commit | `6b25d9b64d44d682518d97271e5ed92cd4475b7c` |
| Tag | `D-006-surface-turnover-stage-d` |
| Tag | `D-006-surface-turnover-closed` |
| Manifest content hash | `dd0b5e5ed5d713bf7ba6e5bee6f1abec9a7aa66ead1924fdc696f29bada97f14` |
| Working tree at gate | clean (removed untracked `scripts/__pycache__/`) |

D-006 artifacts under `digital-protocell/experiments/generated/d006/` and `docs/d006_*.md` were not modified.

## Frozen equation architecture

Equation version remains `surface_turnover_v1`. Only tunable rates in D-007:

- `k_structure_interface`
- `k_rep`

## Machine-precision D-006 reference rates

From `planar_interface/calibration.json` and survivor params (not rounded report text):

| Parameter | Value |
| --- | --- |
| derived `k_structure_interface` (1.0×) | `0.09642857142857159` |
| frozen `k_rep` | `0.014489097664708522` |
| reference configuration hash | `53c5fd482d171d8a5d20dfbc16e7fdc1f1fc782d06d98c659c1a82fd23a172bb` |

Config: `digital-protocell/configs/d007/reference_d006.toml`

## Catalyst reproduction estimate (Stage D, n=180)

Source: `experiments/generated/d006/stage_d/aggregate_flow.json`

Estimator: `required_k_rep = current_k_rep / max(Q_C, ε)` with invalid-row rejection.

Artifact: `experiments/generated/d007/diagnosis/catalyst_rate_estimate.json`

| Statistic | Value |
| --- | --- |
| n_valid | 180 |
| median required `k_rep` | `0.014908378567878227` (~1.029× D006) |
| IQR | see JSON |
| min / max | see JSON |
| outside 3× bound? | **no** |
| classification | `D007_CATALYST_RATE_WITHIN_BOUNDED_RANGE` |

```text
k_rep_center = median valid required_k_rep
```

Note: median `Q_C` is already near unity, so the estimator does not demand a large catalyst-rate correction. Coupled `v_C_inside` remained negative throughout D-006 Stage D; D-007 still explores the bounded catalyst bracket around this center after a structural crossing is established.

## Scientific hypothesis under test

```text
interface assembly is too strong
AND
catalyst reproduction is too weak
```

D-007 tests whether a bounded joint correction produces intersecting restoring nullclines without equation changes.

## Next gate

Structural bracket over factors `0.50 … 0.80` × frozen `k_rep` (63 runs × 30k substeps).


## Reference replay note

Directive-required 10k replay on the D-006 1.0× identity yields transient `v_R < 0`. Same identity/seed at 50k reproduces Stage D direction (`v_R > 0`, `v_C_inside < 0`). Hash match: `53c5fd48…`.
