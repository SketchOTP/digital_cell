# D-015 clearance-law audit

## Configured law

```text
for idx in waste_sink_region:
    W[idx] += (reservoir_rate * dt) * (w_reservoir - W[idx])
```

| Property | Value |
| --- | --- |
| Target W | `w_reservoir = 0.0` |
| Coefficient | `reservoir_rate = 0.5` |
| Baseline region | `grid.reservoir_mask` (r > DISH_RADIUS − RESERVOIR_WIDTH) |
| Repaired W region | dish cells with `r >= waste_sink_inner_radius` (30 for R22) |
| N/F region | unchanged `reservoir_mask` |
| Units | model concentration / time |
| Old-state usage | applied to working copy of accepted state before reactions |
| Form | reaction-like local relaxation (not face flux) |
| dt scaling | once per accepted attempt (`rate * dt`) |
| Cap | none |
| Reverse | yes — relaxes toward target (can increase W if below target) |
| Rejected attempts | do not commit clearance |

## Classification

**CORRECT** (baseline implementation matched its equation; failure was idle sink due to empty peripheral annulus).

## Tests

`test_waste_clearance_matches_configured_law`, `test_waste_clearance_scales_once_with_dt`,
`test_waste_clearance_uses_accepted_old_state`, `test_rejected_attempt_does_not_clear_waste`,
`test_clearance_never_creates_waste_above_target`, `test_clearance_ledger_matches_field_delta`.
