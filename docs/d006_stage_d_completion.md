# D-006 Stage D Completion Report

**Directive:** D-006C  
**Agent memory:** D-20260713-d006c-surface-turnover-completion  
**Equation version (frozen):** `surface_turnover_v1`

## Final execution / scientific statuses

While incomplete:

```text
D006_RESULT_UNRESOLVED_STAGE_D_IN_PROGRESS
```

After Stage D matrix + gate:

```text
D006_NO_RESTORING_RADIUS
```

## Matrix audit

| Quantity | Value |
| --- | --- |
| Theoretical 5×5×3×3 | 225 |
| Scheduled | **180** |
| Completed usable for flow | **180** |
| Resumed after orchestration reset | remaining PENDING after thrash cleanup |
| Invalid (schema-strict §6) | 180 result.json lack field hashes/accounting/clean_termination |
| Scientific usability | all 180 have identity, 50k steps, velocities, Q/slopes, retention |

Why 180: 0.60× prescribed `has_stable_crossing=false` → 4×5×3×3.

## Provenance

| Item | Value |
| --- | --- |
| Experiment-runner binary mtime | 2026-07-13 16:20 |
| Binary sha256 | `3f17d4ca75188686e01bd5e8640c0b8ca9e70cde941b1cfc4ecdef8c5d8ea94d` |
| Chemistry freeze | reactions/`surface_turnover_v1` unchanged during Stage D |
| Pre-savepoint HEAD | `e21068bb2c7f827a563320e105507211379b4f77` |
| Post-savepoint commit | recorded in manifest after git commit |

## Gate decision

- Restoring crossing: **none**
- Nullcline intersections: **0**
- Selected candidate: **none**
- Stage E/F: **skipped**

Artifacts: `experiments/generated/d006/stage_d/`
